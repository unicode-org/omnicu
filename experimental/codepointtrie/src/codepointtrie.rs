// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

// TODO: add module-level Rust-doc with examples

use crate::error::Error;
use crate::impl_const::*;
use std::marker::PhantomData;
use zerovec::ZeroVec;

// Enums

/// The width of the elements in the data array of a [`CodePointTrie`].
/// See UCPTrieValueWidth in ICU4C.
#[derive(Clone, Copy, PartialEq)]
pub enum ValueWidthEnum {
    Bits16 = 0,
    Bits32 = 1,
    Bits8 = 2,
}

/// The type of trie represents whether the trie has an optimization that
/// would make it small or fast.
/// See UCPTrieType in ICU4C.
#[derive(Clone, Copy, PartialEq)]
pub enum TrieTypeEnum {
    Fast = 0,
    Small = 1,
}

// ValueWidth trait

// AsULE is AsUnalignedLittleEndian, i.e. "allowed in a zerovec"
pub trait ValueWidth: Copy + zerovec::ule::AsULE {
    const ENUM_VALUE: ValueWidthEnum;
    fn cast_to_widest(self) -> u32;
}

impl ValueWidth for u8 {
    const ENUM_VALUE: ValueWidthEnum = ValueWidthEnum::Bits8;

    fn cast_to_widest(self) -> u32 {
        self as u32
    }
}

impl ValueWidth for u16 {
    const ENUM_VALUE: ValueWidthEnum = ValueWidthEnum::Bits16;

    fn cast_to_widest(self) -> u32 {
        self as u32
    }
}

impl ValueWidth for u32 {
    const ENUM_VALUE: ValueWidthEnum = ValueWidthEnum::Bits32;

    fn cast_to_widest(self) -> u32 {
        self
    }
}

// TrieType trait

pub trait TrieType {
    const FAST_MAX: u32;
    const ENUM_VALUE: TrieTypeEnum;
}

pub struct Fast;
impl TrieType for Fast {
    const FAST_MAX: u32 = FAST_TYPE_FAST_INDEXING_MAX;
    const ENUM_VALUE: TrieTypeEnum = TrieTypeEnum::Fast;
}

pub struct Small;
impl TrieType for Small {
    const FAST_MAX: u32 = SMALL_TYPE_FAST_INDEXING_MAX;
    const ENUM_VALUE: TrieTypeEnum = TrieTypeEnum::Small;
}

pub struct CodePointTrie<'trie, W: ValueWidth, T: TrieType> {
    header: CodePointTrieHeader,
    index: ZeroVec<'trie, u16>,
    data: ZeroVec<'trie, W>,
    _marker_ty: PhantomData<T>,
}

pub struct CodePointTrieHeader {
    pub index_length: u32,
    pub data_length: u32,
    pub high_start: u32,
    pub shifted12_high_start: u16,
    pub index3_null_offset: u16,
    pub data_null_offset: u32,
    pub null_value: u32,
}

// TODO: add Rust-doc that includes examples

impl<'trie, W: ValueWidth, T: TrieType> CodePointTrie<'trie, W, T> {
    /// Create a new [`CodePointTrie`] backed by borrowed data for the `index`
    /// array and `data` array.
    pub fn try_new(
        header: CodePointTrieHeader,
        index: ZeroVec<'trie, u16>,
        data: ZeroVec<'trie, W>,
    ) -> Result<CodePointTrie<'trie, W, T>, Error> {
        if header.data_length < ERROR_VALUE_NEG_DATA_OFFSET {
            return Err(Error::FromDeserialized {
                reason: "Data array must be large enough to contain error value",
            });
        }

        if header.data_length < HIGH_VALUE_NEG_DATA_OFFSET {
            return Err(Error::FromDeserialized {
                reason:
                    "Data array must be large enough to contain value for range highStart..U+10FFFF",
            });
        }

        if index.len() as u32 != header.index_length {
            return Err(Error::FromDeserialized {
                reason: "Length of index array does not match corresponding header value",
            });
        }

        if data.len() as u32 != header.data_length {
            return Err(Error::FromDeserialized {
                reason: "Length of data array does not match corresponding header value",
            });
        }

        // Note: this particular constructor is "templatized" through Rust's
        // generics type system, so callers to this constructor function must
        // necessarily know ahead of time which trie type and value width their
        // desired trie instance represents. There is no runtime matching ->
        // dynamic dispatch here.

        let trie: CodePointTrie<'trie, W, T> = CodePointTrie {
            header,
            index,
            data,
            _marker_ty: PhantomData,
        };
        Ok(trie)
    }

    fn trie_internal_small_index(&self, code_point: u32) -> u32 {
        let mut index1_pos: u32 = code_point >> SHIFT_1;
        if T::ENUM_VALUE == TrieTypeEnum::Fast {
            assert!(
                FAST_TYPE_FAST_INDEXING_MAX < code_point && code_point < self.header.high_start
            );
            index1_pos = index1_pos + BMP_INDEX_LENGTH - OMITTED_BMP_INDEX_1_LENGTH;
        } else {
            assert!(code_point < self.header.high_start && self.header.high_start > SMALL_LIMIT);
            index1_pos += SMALL_INDEX_LENGTH;
        }
        let index3_block_idx: u32 = (self.index.get(index1_pos as usize).unwrap() as u32)
            + ((code_point >> SHIFT_2) & INDEX_2_MASK);
        let mut index3_block: u32 = self.index.get(index3_block_idx as usize).unwrap() as u32;
        let mut index3_pos: u32 = (code_point >> SHIFT_3) & INDEX_3_MASK;
        let mut data_block: u32;
        if index3_block & 0x8000 == 0 {
            // 16-bit indexes
            data_block = self
                .index
                .get((index3_block + index3_pos) as usize)
                .unwrap() as u32;
        } else {
            // 18-bit indexes stored in groups of 9 entries per 8 indexes.
            index3_block = (index3_block & 0x7fff) + (index3_pos & !7) + (index3_pos >> 3);
            index3_pos &= 7;
            data_block = ((self.index.get(index3_block as usize).unwrap() << (2 + (2 * index3_pos)))
                as u32
                & 0x30000) as u32;
            index3_block += 1;
            data_block |= self
                .index
                .get((index3_block + index3_pos) as usize)
                .unwrap() as u32;
        }
        // Returns data_pos == data_block (offset) +
        //     portion of code_point bit field for last (4th) lookup
        data_block + (code_point & SMALL_DATA_MASK)
    }

    /// Internal trie getter for a code point at or above the fast limit that
    /// is designed for this trie's trie type, [`T`]. Returns the position in
    /// the `data` array for the value that is associated with `code_point`.
    fn trie_above_fastmax_index(&self, code_point: u32) -> u32 {
        if code_point >= self.header.high_start {
            self.header.data_length - HIGH_VALUE_NEG_DATA_OFFSET
        } else {
            self.trie_internal_small_index(code_point) // helper fn
        }
    }

    /// Internal trie getter for a code point below the fast limit that
    /// is designed for this trie's trie type, [`T`]. Returns the position in
    /// the `data` array for the value that is associated with `code_point`.
    fn trie_below_fastmax_index(&self, c: u32) -> u32 {
        let index_array_pos: u32 = c >> FAST_TYPE_SHIFT;
        let index_array_val: u16 = self.index.get(index_array_pos as usize).unwrap(); // 1. How to specify type parameter for .index() 2. How to avoid unwrap()?
        let fast_index_val: u32 = index_array_val as u32 + (c & FAST_TYPE_DATA_MASK);
        fast_index_val
    }

    /// Returns the value that is associated with `code_point` for this [`CodePointTrie`].
    pub fn get(&self, code_point: u32) -> W {
        let data_pos: u32 = if code_point <= T::FAST_MAX {
            Self::trie_below_fastmax_index(self, code_point)
        } else if code_point <= CODE_POINT_MAX {
            Self::trie_above_fastmax_index(self, code_point)
        } else {
            self.header.data_length - ERROR_VALUE_NEG_DATA_OFFSET
        };
        // We need the unwrap because the default value is stored in the data array,
        // and getting that default value always returns an Option<W>, but need to return W
        self.data.get(data_pos as usize).unwrap()
    }

    /// Returns the value that is associated with `code_point` for this [`CodePointTrie`]
    /// as a `u32`. This API method maintains consistency with the corresponding
    /// originalICU APIs.
    pub fn get_u32(&self, code_point: u32) -> u32 {
        // CPTAuto will only expose get_u32() (probably will call it `get()`), but will not / cannot expose get() without using an enum for W
        // CPT<W1,S1> will expose get() and get_u32(). You may not want get_u32() at this point, but it is necessary in order to implement CPTAuto

        // u32 vs. char in Rust - unpaired surrogates are valid code points but are not UTF-32 code units, therefore not allowed in `char` type in Rust.
        // But for the CodePointTrie, we do want to be able to look up surrogate code points, therefore we want to use the u32 type. (Note: this is also what we do in the `uniset` crate for `UnicodeSet`.)
        // struct CodePoint(u32) ← enforce whatever invariants you'd like

        // this is the consistent API that is public-facing for users
        self.get(code_point).cast_to_widest()
    }
}

/// Convert the serialized `u8` value for the trie type into a `TrieTypeEnum`.
pub fn get_code_point_trie_type_enum(trie_type_int: u8) -> Option<TrieTypeEnum> {
    match trie_type_int {
        0 => Some(TrieTypeEnum::Fast),
        1 => Some(TrieTypeEnum::Small),
        _ => None,
    }
}
