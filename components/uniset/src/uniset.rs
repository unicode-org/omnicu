// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use icu_provider::yoke::{self, *};
use std::{char, ops::RangeBounds, ops::RangeInclusive, slice::Chunks};

#[cfg(feature = "serde")]
use serde::ser::SerializeSeq;

use super::UnicodeSetError;
use crate::utils::{deconstruct_range, is_valid};

/// Represents the end code point of the Basic Multilingual Plane range, starting from code point 0, inclusive
const BMP_MAX: u32 = 0xFFFF;

/// A membership wrapper for [`UnicodeSet`].
///
/// Provides exposure to membership functions and constructors from serialized [`UnicodeSets`](UnicodeSet)
/// and predefined ranges.
#[derive(Debug, PartialEq, Hash, Eq, Clone, Yokeable, ZeroCopyFrom)]
#[yoke(cloning_zcf)]
pub struct UnicodeSet {
    // TODO: need advice - how should we remove Hash and Eq from UnicodeSet unless we need it?

    // If we wanted to use an array to keep the memory on the stack, there is an unsafe nightly feature
    // https://doc.rust-lang.org/nightly/core/array/trait.FixedSizeArray.html
    // Allows for traits of fixed size arrays

    // Implements an [inversion list.](https://en.wikipedia.org/wiki/Inversion_list)
    inv_list: Vec<u32>,
    size: usize,
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for UnicodeSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let parsed_inv_list = Vec::<u32>::deserialize(deserializer)?;

        UnicodeSet::from_inversion_list(parsed_inv_list).map_err(|e| Error::custom(format!("Cannot deserialize invalid inversion list for UnicodeSet: {:?}", e)))
    }
}

// Note: serde(flatten) currently does not promote a struct field of type Vec
// to replace the struct when serializing. The error message from the default
// serialization is: "can only flatten structs and maps (got a sequence)".

#[cfg(feature = "serde")]
impl serde::Serialize for UnicodeSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.inv_list.len()))?;
        for e in &self.inv_list {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

impl UnicodeSet {
    /// Returns [`UnicodeSet`] from an [inversion list.](https://en.wikipedia.org/wiki/Inversion_list)
    /// represented by a [`Vec`]`<`[`u32`]`>` of codepoints.
    ///
    /// The inversion list must be of even length, sorted ascending non-overlapping,
    /// and within the bounds of `0x0 -> 0x10FFFF` inclusive, and end points being exclusive.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::uniset::UnicodeSet;
    /// use icu::uniset::UnicodeSetError;
    /// let invalid: Vec<u32> = vec![0x0, 0x80, 0x3];
    /// let result = UnicodeSet::from_inversion_list(invalid.clone());
    /// assert!(matches!(result, Err(UnicodeSetError::InvalidSet(_))));
    /// if let Err(UnicodeSetError::InvalidSet(actual)) = result {
    ///     assert_eq!(invalid, actual);
    /// }
    /// ```
    pub fn from_inversion_list(inv_list: Vec<u32>) -> Result<Self, UnicodeSetError> {
        if is_valid(&inv_list) {
            let size: usize = inv_list.chunks(2).map(|end_points| end_points[1] - end_points[0]).sum::<u32>() as usize;
            Ok(Self { inv_list, size })
        } else {
            Err(UnicodeSetError::InvalidSet(inv_list))
        }
    }

    /// Returns an owned inversion list representing the current [`UnicodeSet`]
    pub fn get_inversion_list(&self) -> Vec<u32> {
        let result: Vec<u32> = self
            .as_inversion_list() // Only crate public, to not leak impl
            .to_vec();
        result
    }

    /// Returns [`UnicodeSet`] spanning entire Unicode range
    ///
    /// The range spans from `0x0 -> 0x10FFFF` inclusive
    pub fn all() -> Self {
        Self {
            inv_list: vec![0x0, (char::MAX as u32) + 1],
            size: (char::MAX as usize) + 1,
        }
    }

    /// Returns [`UnicodeSet`] spanning BMP range
    ///
    /// The range spans from `0x0 -> 0xFFFF` inclusive
    pub fn bmp() -> Self {
        Self {
            inv_list: vec![0x0, BMP_MAX + 1],
            size: (BMP_MAX as usize) + 1,
        }
    }

    /// Returns the inversion list as a slice
    ///
    /// Public only to the crate, not exposed to public
    pub(crate) fn as_inversion_list(&self) -> &[u32] {
        &self.inv_list
    }

    /// Yields an [`Iterator`] going through the character set in the [`UnicodeSet`]
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::uniset::UnicodeSet;
    /// let example_list = vec![0x41, 0x44, 0x45, 0x46];
    /// let example = UnicodeSet::from_inversion_list(example_list).unwrap();
    /// let mut ex_iter_chars = example.iter_chars();
    /// assert_eq!(Some('A'), ex_iter_chars.next());
    /// assert_eq!(Some('B'), ex_iter_chars.next());
    /// assert_eq!(Some('C'), ex_iter_chars.next());
    /// assert_eq!(Some('E'), ex_iter_chars.next());
    /// assert_eq!(None, ex_iter_chars.next());
    /// ```
    pub fn iter_chars(&self) -> impl Iterator<Item = char> + '_ {
        self.inv_list.chunks(2).flat_map(|pair| (pair[0]..pair[1])).filter_map(char::from_u32)
    }

    /// Yields an [`Iterator`] returning the ranges of the code points that are
    /// included in the [`UnicodeSet`]
    ///
    /// Ranges are returned as [`RangeInclusive`], which is inclusive of its
    /// `end` bound value. An end-inclusive behavior matches the ICU4C/J
    /// behavior of ranges, ex: `UnicodeSet::contains(UChar32 start, UChar32 end)`.
    ///
    /// # Example
    ///
    /// ```
    /// use icu::uniset::UnicodeSet;
    /// let example_list = vec![0x41, 0x44, 0x45, 0x46];
    /// let example = UnicodeSet::from_inversion_list(example_list).unwrap();
    /// let mut example_iter_ranges = example.iter_ranges();
    /// assert_eq!(Some(0x41..=0x43), example_iter_ranges.next());
    /// assert_eq!(Some(0x45..=0x45), example_iter_ranges.next());
    /// assert_eq!(None, example_iter_ranges.next());
    /// ```
    pub fn iter_ranges(&self) -> impl ExactSizeIterator<Item = RangeInclusive<u32>> + '_ {
        self.inv_list.chunks(2).map(|pair| RangeInclusive::new(pair[0], pair[1] - 1))
    }

    /// Returns the number of ranges contained in this [`UnicodeSet`]
    pub fn get_range_count(&self) -> usize {
        self.inv_list.len() / 2
    }

    /// Returns the number of elements of the [`UnicodeSet`]
    pub fn size(&self) -> usize {
        if self.is_empty() {
            return 0;
        }
        self.size
    }

    /// Returns whether or not the [`UnicodeSet`] is empty
    pub fn is_empty(&self) -> bool {
        self.inv_list.is_empty()
    }

    /// Wrapper for contains
    ///
    /// Returns an [`Option`] as to whether or not it is possible for the query to be contained.
    /// The value in the [`Option`] is the start index of the range that contains the query.
    fn contains_query(&self, query: u32) -> Option<usize> {
        match self.inv_list.binary_search(&query) {
            Ok(pos) => {
                if pos % 2 == 0 {
                    Some(pos)
                } else {
                    None
                }
            }
            Err(pos) => {
                if pos % 2 != 0 && pos < self.inv_list.len() {
                    Some(pos - 1)
                } else {
                    None
                }
            }
        }
    }

    /// Checks to see the query is in the [`UnicodeSet`]
    ///
    /// Runs a binary search in `O(log(n))` where `n` is the number of start and end points
    /// in the set using [`std`] implementation
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::uniset::UnicodeSet;
    /// let example_list = vec![0x41, 0x43, 0x44, 0x45];
    /// let example = UnicodeSet::from_inversion_list(example_list).unwrap();
    /// assert!(example.contains('A'));
    /// assert!(!example.contains('C'));
    /// ```
    pub fn contains(&self, query: char) -> bool {
        self.contains_query(query as u32).is_some()
    }

    /// Checks to see the unsigned int is in the [`UnicodeSet::all()`](UnicodeSet::all())
    ///
    /// Note: Even though [`u32`] and [`prim@char`] in Rust are non-negative 4-byte
    /// values, there is an important difference. A [`u32`] can take values up to
    /// a very large integer value, while a [`prim@char`] in Rust is defined to be in
    /// the range from 0 to the maximum valid Unicode Scalar Value.
    ///
    /// Runs a binary search in `O(log(n))` where `n` is the number of start and end points
    /// in the set using [`std`] implementation
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::uniset::UnicodeSet;
    /// let example_list = vec![0x41, 0x43, 0x44, 0x45];
    /// let example = UnicodeSet::from_inversion_list(example_list).unwrap();
    /// assert!(example.contains_u32(0x41));
    /// assert!(!example.contains_u32(0x43));
    /// ```
    pub fn contains_u32(&self, query: u32) -> bool {
        self.contains_query(query).is_some()
    }

    /// Checks to see if the range is in the [`UnicodeSet`], returns a [`Result`]
    ///
    /// Runs a binary search in `O(log(n))` where `n` is the number of start and end points
    /// in the set using [`Vec`] implementation. Only runs the search once on the `start`
    /// parameter, while the `end` parameter is checked in a single `O(1)` step.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::uniset::UnicodeSet;
    /// let example_list = vec![0x41, 0x43, 0x44, 0x45];
    /// let example = UnicodeSet::from_inversion_list(example_list).unwrap();
    /// assert!(example.contains_range(&('A'..'C')));
    /// assert!(example.contains_range(&('A'..='B')));
    /// assert!(!example.contains_range(&('A'..='C')));
    /// ```
    ///
    /// Surrogate points (`0xD800 -> 0xDFFF`) will return [`false`] if the Range contains them but the
    /// [`UnicodeSet`] does not.
    ///
    /// Note: when comparing to ICU4C/J, keep in mind that `Range`s in Rust are
    /// constructed inclusive of start boundary and exclusive of end boundary.
    /// The ICU4C/J `UnicodeSet::contains(UChar32 start, UChar32 end)` method
    /// differs by including the end boundary.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::uniset::UnicodeSet;
    /// use std::char;
    /// let check = char::from_u32(0xD7FE).unwrap() .. char::from_u32(0xE001).unwrap();
    /// let example_list = vec![0xD7FE, 0xD7FF, 0xE000, 0xE001];
    /// let example = UnicodeSet::from_inversion_list(example_list).unwrap();
    /// assert!(!example.contains_range(&(check)));
    /// ```
    pub fn contains_range(&self, range: &impl RangeBounds<char>) -> bool {
        let (from, till) = deconstruct_range(range);
        if from >= till {
            return false;
        }
        match self.contains_query(from) {
            Some(pos) => (till) <= self.inv_list[pos + 1],
            None => false,
        }
    }

    /// Check if the calling [`UnicodeSet`] contains all the characters of the given [`UnicodeSet`]
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::uniset::UnicodeSet;
    /// let example_list = vec![0x41, 0x46, 0x55, 0x5B]; // A - E, U - Z
    /// let example = UnicodeSet::from_inversion_list(example_list).unwrap();
    /// let a_to_d = UnicodeSet::from_inversion_list(vec![0x41, 0x45]).unwrap();
    /// let f_to_t = UnicodeSet::from_inversion_list(vec![0x46, 0x55]).unwrap();
    /// let r_to_x = UnicodeSet::from_inversion_list(vec![0x52, 0x58]).unwrap();
    /// assert!(example.contains_set(&a_to_d)); // contains all
    /// assert!(!example.contains_set(&f_to_t)); // contains none
    /// assert!(!example.contains_set(&r_to_x)); // contains some
    /// ```
    pub fn contains_set(&self, set: &Self) -> bool {
        if set.size() > self.size() {
            return false;
        }
        let mut set_ranges: Chunks<u32> = set.as_inversion_list().chunks(2);
        let mut check = set_ranges.next();
        for range in self.inv_list.chunks(2) {
            match check {
                Some(r) => {
                    if r[0] >= range[0] && r[1] <= range[1] {
                        check = set_ranges.next();
                    }
                }
                _ => break,
            }
        }
        check.is_none()
    }

    /// Returns the end of the initial substring where the characters are either contained/not contained
    /// in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::uniset::UnicodeSet;
    /// let example_list = vec![0x41, 0x44]; // {A, B, C}
    /// let example = UnicodeSet::from_inversion_list(example_list).unwrap();
    /// assert_eq!(example.span("CABXYZ", true), 3);
    /// assert_eq!(example.span("XYZC", false), 3);
    /// assert_eq!(example.span("XYZ", true), 0);
    /// assert_eq!(example.span("ABC", false), 0);
    /// ```
    pub fn span(&self, span_str: &str, contained: bool) -> usize {
        span_str.chars().take_while(|&x| self.contains(x) == contained).count()
    }

    /// Returns the start of the trailing substring (starting from end of string) where the characters are
    /// either contained/not contained in the set. Returns the length of the string if no valid return.
    ///
    /// # Examples
    ///
    /// ```
    /// use icu::uniset::UnicodeSet;
    /// let example_list = vec![0x41, 0x44]; // {A, B, C}
    /// let example = UnicodeSet::from_inversion_list(example_list).unwrap();
    /// assert_eq!(example.span_back("XYZCAB", true), 3);
    /// assert_eq!(example.span_back("ABCXYZ", true), 6);
    /// assert_eq!(example.span_back("CABXYZ", false), 3);
    /// ```
    pub fn span_back(&self, span_str: &str, contained: bool) -> usize {
        span_str.len() - span_str.chars().rev().take_while(|&x| self.contains(x) == contained).count()
    }
}

#[cfg(test)]
mod tests {
    use super::{UnicodeSet, UnicodeSetError, BMP_MAX};
    use std::{char, vec::Vec};

    #[test]
    fn test_unicodeset_try_from_vec() {
        let ex = vec![0x2, 0x3, 0x4, 0x5];
        let check = UnicodeSet::from_inversion_list(ex.clone()).unwrap();
        assert_eq!(ex, check.inv_list);
        assert_eq!(0x2, check.size());
    }

    #[test]
    fn test_unicodeset_try_from_vec_error() {
        let check = vec![0x1, 0x1, 0x2, 0x3, 0x4];
        let set = UnicodeSet::from_inversion_list(check.clone());
        assert!(matches!(set, Err(UnicodeSetError::InvalidSet(_))));
        if let Err(UnicodeSetError::InvalidSet(actual)) = set {
            assert_eq!(check, actual);
        }
    }

    #[test]
    fn test_unicodeset_all() {
        let expected = vec![0x0, (char::MAX as u32) + 1];
        assert_eq!(UnicodeSet::all().inv_list, expected);
        assert_eq!(UnicodeSet::all().size(), (expected[1] - expected[0]) as usize)
    }

    #[test]
    fn test_unicodeset_bmp() {
        let expected = vec![0x0, BMP_MAX + 1];
        assert_eq!(UnicodeSet::bmp().inv_list, expected);
        assert_eq!(UnicodeSet::bmp().size(), (expected[1] - expected[0]) as usize);
    }

    // UnicodeSet membership functions
    #[test]
    fn test_unicodeset_contains_query() {
        let ex = vec![0x41, 0x46, 0x4B, 0x55];
        let check = UnicodeSet::from_inversion_list(ex).unwrap();
        assert!(check.contains_query(0x40).is_none());
        assert_eq!(check.contains_query(0x41).unwrap(), 0);
        assert_eq!(check.contains_query(0x44).unwrap(), 0);
        assert!(check.contains_query(0x46).is_none());
        assert_eq!(check.contains_query(0x4C).unwrap(), 2);
        assert!(check.contains_query(0x56).is_none());
    }

    #[test]
    fn test_unicodeset_contains() {
        let ex = vec![0x2, 0x5, 0xA, 0xF];
        let check = UnicodeSet::from_inversion_list(ex).unwrap();
        assert!(check.contains(0x2 as char));
        assert!(check.contains(0x4 as char));
        assert!(check.contains(0xA as char));
        assert!(check.contains(0xE as char));
    }

    #[test]
    fn test_unicodeset_contains_false() {
        let ex = vec![0x2, 0x5, 0xA, 0xF];
        let check = UnicodeSet::from_inversion_list(ex).unwrap();
        assert!(!check.contains(0x1 as char));
        assert!(!check.contains(0x5 as char));
        assert!(!check.contains(0x9 as char));
        assert!(!check.contains(0xF as char));
        assert!(!check.contains(0x10 as char));
    }

    #[test]
    fn test_unicodeset_contains_range() {
        let ex = vec![0x41, 0x46, 0x4B, 0x55];
        let check = UnicodeSet::from_inversion_list(ex).unwrap();
        assert!(check.contains_range(&('A'..='E'))); // 65 - 69
        assert!(check.contains_range(&('C'..'D'))); // 67 - 67
        assert!(check.contains_range(&('L'..'P'))); // 76 - 80
        assert!(!check.contains_range(&('L'..='U'))); // 76 - 85
    }

    #[test]
    fn test_unicodeset_contains_range_false() {
        let ex = vec![0x41, 0x46, 0x4B, 0x55];
        let check = UnicodeSet::from_inversion_list(ex).unwrap();
        assert!(!check.contains_range(&('!'..'A'))); // 33 - 65
        assert!(!check.contains_range(&('F'..'K'))); // 70 - 74
        assert!(!check.contains_range(&('U'..))); // 85 - ..
    }

    #[test]
    fn test_unicodeset_contains_range_invalid() {
        let check = UnicodeSet::all();
        assert!(!check.contains_range(&('A'..'!'))); // 65 - 33
        assert!(!check.contains_range(&('A'..'A'))); // 65 - 65
    }

    #[test]
    fn test_unicodeset_contains_set_u() {
        let ex = vec![0xA, 0x14, 0x28, 0x32, 0x46, 0x50, 0x64, 0x6E];
        let u = UnicodeSet::from_inversion_list(ex).unwrap();
        let inside = vec![0xF, 0x14, 0x2C, 0x31, 0x46, 0x50, 0x64, 0x6D];
        let s = UnicodeSet::from_inversion_list(inside).unwrap();
        assert!(u.contains_set(&s));
    }

    #[test]
    fn test_unicodeset_contains_set_u_false() {
        let ex = vec![0xA, 0x14, 0x28, 0x32, 0x46, 0x50, 0x64, 0x78];
        let u = UnicodeSet::from_inversion_list(ex).unwrap();
        let outside = vec![0x0, 0xA, 0x16, 0x2C, 0x32, 0x46, 0x4F, 0x51, 0x6D, 0x6F];
        let s = UnicodeSet::from_inversion_list(outside).unwrap();
        assert!(!u.contains_set(&s));
    }

    #[test]
    fn test_unicodeset_size() {
        let ex = vec![0x2, 0x5, 0xA, 0xF];
        let check = UnicodeSet::from_inversion_list(ex).unwrap();
        assert_eq!(8, check.size());
        let check = UnicodeSet::all();
        let expected = (char::MAX as u32) + 1;
        assert_eq!(expected as usize, check.size());
        let check = UnicodeSet { inv_list: Vec::new(), size: 0 };
        assert_eq!(check.size(), 0);
    }

    #[test]
    fn test_unicodeset_is_empty() {
        let check = UnicodeSet { inv_list: vec![], size: 0 };
        assert!(check.is_empty());
    }

    #[test]
    fn test_unicodeset_is_not_empty() {
        let check = UnicodeSet::all();
        assert!(!check.is_empty());
    }

    #[test]
    fn test_unicodeset_iter_chars() {
        let ex = vec![0x41, 0x44, 0x45, 0x46, 0xD800, 0xD801];
        let check = UnicodeSet::from_inversion_list(ex).unwrap();
        let mut iter = check.iter_chars();
        assert_eq!(Some('A'), iter.next());
        assert_eq!(Some('B'), iter.next());
        assert_eq!(Some('C'), iter.next());
        assert_eq!(Some('E'), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_unicodeset_iter_ranges() {
        let ex = vec![0x41, 0x44, 0x45, 0x46, 0xD800, 0xD801];
        let set = UnicodeSet::from_inversion_list(ex).unwrap();
        let mut ranges = set.iter_ranges();
        assert_eq!(Some(0x41..=0x43), ranges.next());
        assert_eq!(Some(0x45..=0x45), ranges.next());
        assert_eq!(Some(0xD800..=0xD800), ranges.next());
        assert_eq!(None, ranges.next());
    }

    #[test]
    fn test_unicodeset_iter_ranges_exactsizeiter_trait() {
        let ex = vec![0x41, 0x44, 0x45, 0x46, 0xD800, 0xD801];
        let set = UnicodeSet::from_inversion_list(ex).unwrap();
        let ranges = set.iter_ranges();
        assert_eq!(3, ranges.len());
    }

    #[test]
    fn test_unicodeset_range_count() {
        let ex = vec![0x41, 0x44, 0x45, 0x46, 0xD800, 0xD801];
        let set = UnicodeSet::from_inversion_list(ex).unwrap();
        assert_eq!(3, set.get_range_count());
    }

    // Range<char> cannot represent the upper bound (non-inclusive) for
    // char::MAX, whereas Range<u32> can.
    #[test]
    fn test_unicodeset_iter_ranges_with_max_code_point() {
        let ex = vec![0x80, (char::MAX as u32) + 1];
        let set = UnicodeSet::from_inversion_list(ex).unwrap();
        let mut ranges = set.iter_ranges();
        assert_eq!(Some(0x80..=(char::MAX as u32)), ranges.next());
        assert_eq!(None, ranges.next());
    }

    #[test]
    fn test_unicodeset_span_contains() {
        let ex = vec![0x41, 0x44, 0x46, 0x4B]; // A - D, F - K
        let check = UnicodeSet::from_inversion_list(ex).unwrap();
        assert_eq!(check.span("ABCDE", true), 3);
        assert_eq!(check.span("E", true), 0);
    }

    #[test]
    fn test_unicodeset_span_does_not_contain() {
        let ex = vec![0x41, 0x44, 0x46, 0x4B]; // A - D, F - K
        let check = UnicodeSet::from_inversion_list(ex).unwrap();
        assert_eq!(check.span("DEF", false), 2);
        assert_eq!(check.span("KLMA", false), 3);
    }

    #[test]
    fn test_unicodeset_span_back_contains() {
        let ex = vec![0x41, 0x44, 0x46, 0x4B]; // A - D, F - K
        let check = UnicodeSet::from_inversion_list(ex).unwrap();
        assert_eq!(check.span_back("XYZABFH", true), 3);
        assert_eq!(check.span_back("ABCXYZ", true), 6);
    }

    #[test]
    fn test_unicodeset_span_back_does_not_contain() {
        let ex = vec![0x41, 0x44, 0x46, 0x4B]; // A - D, F - K
        let check = UnicodeSet::from_inversion_list(ex).unwrap();
        assert_eq!(check.span_back("ABCXYZ", false), 3);
        assert_eq!(check.span_back("XYZABC", false), 6);
    }

    #[test]
    fn test_uniset_to_inv_list() {
        let inv_list: Vec<u32> = vec![
            0x9, 0xE, 0x20, 0x21, 0x85, 0x86, 0xA0, 0xA1, 0x1626, 0x1627, 0x2000, 0x2003, 0x2028, 0x202A, 0x202F, 0x2030, 0x205F, 0x2060, 0x3000, 0x3001,
        ];
        let inv_list_clone = (&inv_list).clone();
        let s: UnicodeSet = UnicodeSet::from_inversion_list(inv_list_clone).unwrap();
        let round_trip_inv_list = s.get_inversion_list();
        assert_eq!(round_trip_inv_list, inv_list);
    }

    #[test]
    fn test_serde_serialize() {
        let inv_list = vec![0x41, 0x46, 0x4B, 0x55];
        let uniset = UnicodeSet::from_inversion_list(inv_list).unwrap();
        let json_str = serde_json::to_string(&uniset).unwrap();
        assert_eq!(json_str, "[65,70,75,85]");
    }

    #[test]
    fn test_serde_deserialize() {
        let inv_list_str = "[65,70,75,85]";
        let exp_inv_list = vec![0x41, 0x46, 0x4B, 0x55];
        let exp_uniset = UnicodeSet::from_inversion_list(exp_inv_list).unwrap();
        let act_uniset: UnicodeSet = serde_json::from_str(inv_list_str).unwrap();
        assert_eq!(act_uniset, exp_uniset);
    }

    #[test]
    fn test_serde_deserialize_invalid() {
        let inv_list_str = "[65,70,98775,85]";
        let act_result: Result<UnicodeSet, serde_json::Error> = serde_json::from_str(inv_list_str);
        assert!(matches!(act_result, Err(_)));
    }
}
