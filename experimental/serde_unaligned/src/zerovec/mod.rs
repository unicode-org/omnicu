// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#[cfg(feature = "serde")]
mod serde;

use crate::ule::*;
use std::fmt;
use std::ops::Deref;

/// A zero-copy vector for fixed-length types.
///
/// `ZeroVec<T>` is designed as a drop-in replacement for `Vec<T>` in situations where it is
/// desirable to borrow data from an unaligned byte slice, such as zero-copy deserialization.
///
/// `T` must implement [`AsULE`], which is auto-implemented for a number of built-in types,
/// including all fixed-length multibyte integers.
///
/// # How it Works
///
/// `ZeroVec<T>` represents a slice of `T` as a slice of `T::ULE`. The difference between `T` and
/// `T::ULE` is that `T::ULE` must be encoded in little-endian with 1-byte alignment. When accessing
/// items from `ZeroVec<T>`, we fetch the `T::ULE`, convert it on the fly to `T`, and return `T` by
/// value.
///
/// Benchmarks can be found in the project repository. We found that for common operations on small
/// and large vectors, `ZeroVec<T>` performs from slightly faster to 15% slower than `Vec<T>`.
/// However, the main performance improvement on `ZeroVec<T>` is when deserializing from a byte
/// array; `ZeroVec<T>` deserializes 80% faster than `Vec<T>` in Serde Bincode, and it does not
/// require any heap allocations.
///
/// # Safety
///
/// `ZeroVec<T>` contains no unsafe code. However, the conversion from `&[u8]` to `&[T::ULE]` may
/// be unsafe. For more information, see the [`ule`] module.
///
/// # Example
///
/// ```
/// use serde_unaligned::ZeroVec;
///
/// // The little-endian bytes correspond to the numbers on the following line.
/// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
/// let nums: &[u16] = &[211, 281, 421, 461];
///
/// // Conversion from &[u8] to &[u16::ULE] is infallible.
/// let zerovec: ZeroVec<u16> = ZeroVec::try_from_bytes(bytes).expect("infallible");
///
/// assert!(matches!(zerovec, ZeroVec::Borrowed(_)));
/// assert_eq!(zerovec.get(2), Some(421));
/// assert_eq!(zerovec, nums);
/// ```
#[non_exhaustive]
#[derive(Clone)]
pub enum ZeroVec<'a, T>
where
    T: AsULE + ?Sized,
{
    Owned(Vec<T::ULE>),
    Borrowed(&'a [T::ULE]),
}

impl<T> fmt::Debug for ZeroVec<'_, T>
where
    T: AsULE + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ZeroVec::*;
        match self {
            Owned(_) => write!(f, "ZeroVec::Owned({:?})", self.to_vec()),
            Borrowed(_) => write!(f, "ZeroVec::Borrowed({:?})", self.to_vec()),
        }
    }
}

impl<'a, 'b, T> PartialEq<ZeroVec<'b, T>> for ZeroVec<'a, T>
where
    T: AsULE + Copy + PartialEq,
{
    #[inline(always)]
    fn eq(&self, other: &ZeroVec<'b, T>) -> bool {
        // Note: T implements PartialEq but not T::ULE
        self.iter().eq(other.iter())
    }
}

impl<T> PartialEq<&[T]> for ZeroVec<'_, T>
where
    T: AsULE + Copy + PartialEq,
{
    #[inline(always)]
    fn eq(&self, other: &&[T]) -> bool {
        self.iter().eq(other.iter().copied())
    }
}

impl<'a, T> ZeroVec<'a, T>
where
    T: AsULE + ?Sized,
{
    /// Parses a `&[u8]` buffer into a `ZeroVec<T>`.
    ///
    /// This function is infallible for built-in integer types, but fallible for other types,
    /// such as `char`. For more information, see [`ULE::parse_byte_slice`].
    ///
    /// # Endianness
    ///
    /// The byte buffer must be encoded in little-endian, even if running in a big-endian
    /// environment. This ensures a consistent representation of data across platforms.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    ///
    /// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
    /// let zerovec: ZeroVec<u16> = ZeroVec::try_from_bytes(bytes).expect("infallible");
    ///
    /// assert!(matches!(zerovec, ZeroVec::Borrowed(_)));
    /// assert_eq!(zerovec.get(2), Some(421));
    /// ```
    pub fn try_from_bytes(bytes: &'a [u8]) -> Result<Self, <<T as AsULE>::ULE as ULE>::Error> {
        let slice: &'a [T::ULE] = T::ULE::parse_byte_slice(bytes)?;
        Ok(Self::Borrowed(slice))
    }

    /// Returns a `ZeroVec<T>` as its underlying `&[u8]` byte buffer representation.
    ///
    /// Useful for serialization.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    ///
    /// // The little-endian bytes correspond to the numbers on the following line.
    /// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
    /// let nums: &[u16] = &[211, 281, 421, 461];
    ///
    /// let zerovec = ZeroVec::from_aligned(nums);
    ///
    /// assert_eq!(bytes, zerovec.as_bytes());
    /// ```
    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        T::ULE::as_byte_slice(self.as_slice())
    }

    /// Dereferences this `ZeroVec<T>` as `&[T::ULE]`. Most other functions on `ZeroVec<T>` use
    /// this function as a building block.
    #[inline(always)]
    pub fn as_slice(&self) -> &[T::ULE] {
        use ZeroVec::*;
        match self {
            Owned(vec) => vec.as_slice(),
            Borrowed(slice) => slice,
        }
    }

    /// Returns the number of elements in this `ZeroVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    /// use serde_unaligned::ule::AsULE;
    ///
    /// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
    /// let zerovec: ZeroVec<u16> = ZeroVec::try_from_bytes(bytes).expect("infallible");
    ///
    /// assert_eq!(4, zerovec.len());
    /// assert_eq!(
    ///     bytes.len(),
    ///     zerovec.len() * std::mem::size_of::<<u16 as AsULE>::ULE>()
    /// );
    /// ```
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.deref().len()
    }

    /// Returns whether the vec is empty.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    ///
    /// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
    /// let zerovec: ZeroVec<u16> = ZeroVec::try_from_bytes(bytes).expect("infallible");
    /// assert!(!zerovec.is_empty());
    ///
    /// let emptyvec: ZeroVec<u16> = ZeroVec::try_from_bytes(&[]).expect("infallible");
    /// assert!(emptyvec.is_empty());
    /// ```
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.deref().is_empty()
    }
}

impl<T> ZeroVec<'_, T>
where
    T: AsULE,
{
    /// Creates a `ZeroVec<T>` from a `&[T]`.
    ///
    /// This function allocates memory and results in an `Owned` instance of `ZeroVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    ///
    /// // The little-endian bytes correspond to the numbers on the following line.
    /// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
    /// let nums: &[u16] = &[211, 281, 421, 461];
    ///
    /// let zerovec = ZeroVec::from_aligned(nums);
    ///
    /// assert!(matches!(zerovec, ZeroVec::Owned(_)));
    /// assert_eq!(bytes, zerovec.as_bytes());
    /// ```
    #[inline(always)]
    pub fn from_aligned(other: &[T]) -> Self {
        Self::Owned(other.iter().map(T::as_unaligned).collect())
    }

    /// Creates a `Vec<T>` from a `ZeroVec<T>`.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    ///
    /// let nums: &[u16] = &[211, 281, 421, 461];
    /// let vec: Vec<u16> = ZeroVec::from_aligned(nums).to_vec();
    ///
    /// assert_eq!(nums, vec.as_slice());
    /// ```
    #[inline(always)]
    pub fn to_vec(&self) -> Vec<T> {
        self.as_slice().iter().map(T::from_unaligned).collect()
    }
}

impl<T> ZeroVec<'_, T>
where
    T: AsULE + Copy + ?Sized,
{
    /// Gets the element at the specified index. Returns None if out of range.
    ///
    /// The element is returned by value, so `T` must implement `Copy`.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    ///
    /// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
    /// let zerovec: ZeroVec<u16> = ZeroVec::try_from_bytes(bytes).expect("infallible");
    ///
    /// assert_eq!(zerovec.get(2), Some(421));
    /// assert_eq!(zerovec.get(4), None);
    /// ```
    #[inline(always)]
    pub fn get(&self, index: usize) -> Option<T> {
        self.as_slice().get(index).map(T::from_unaligned)
    }

    /// Gets the first element. Returns None if empty.
    ///
    /// The element is returned by value, so `T` must implement `Copy`.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    ///
    /// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
    /// let zerovec: ZeroVec<u16> = ZeroVec::try_from_bytes(bytes).expect("infallible");
    ///
    /// assert_eq!(zerovec.first(), Some(211));
    /// ```
    #[inline(always)]
    pub fn first(&self) -> Option<T> {
        self.as_slice().first().map(T::from_unaligned)
    }

    /// Gets the last element. Returns None if empty.
    ///
    /// The element is returned by value, so `T` must implement `Copy`.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    ///
    /// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
    /// let zerovec: ZeroVec<u16> = ZeroVec::try_from_bytes(bytes).expect("infallible");
    ///
    /// assert_eq!(zerovec.last(), Some(461));
    /// ```
    #[inline(always)]
    pub fn last(&self) -> Option<T> {
        self.as_slice().last().map(T::from_unaligned)
    }

    /// Gets an iterator over the elements.
    ///
    /// The elements are returned by value, so `T` must implement `Copy`.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    ///
    /// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
    /// let zerovec: ZeroVec<u16> = ZeroVec::try_from_bytes(bytes).expect("infallible");
    /// let mut it = zerovec.iter();
    ///
    /// assert_eq!(it.next(), Some(211));
    /// assert_eq!(it.next(), Some(281));
    /// assert_eq!(it.next(), Some(421));
    /// assert_eq!(it.next(), Some(461));
    /// assert_eq!(it.next(), None);
    /// ```
    #[inline(always)]
    pub fn iter<'b>(&'b self) -> impl Iterator<Item = T> + 'b {
        self.as_slice().iter().map(T::from_unaligned)
    }

    /// Converts a borrowed ZeroVec to an owned ZeroVec. No-op if already owned.
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    ///
    /// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
    /// let zerovec: ZeroVec<u16> = ZeroVec::try_from_bytes(bytes).expect("infallible");
    /// assert!(matches!(zerovec, ZeroVec::Borrowed(_)));
    ///
    /// let owned = zerovec.into_owned();
    /// assert!(matches!(owned, ZeroVec::Owned(_)));
    /// ```
    pub fn into_owned(self) -> ZeroVec<'static, T> {
        use ZeroVec::*;
        match self {
            Owned(vec) => ZeroVec::Owned(vec),
            Borrowed(_) => {
                let vec: Vec<T::ULE> = self.iter().map(|ule| T::as_unaligned(&ule)).collect();
                ZeroVec::Owned(vec)
            }
        }
    }
}

impl<T> ZeroVec<'_, T>
where
    T: AsULE + Ord,
{
    /// Binary searches a sorted `ZeroVec<T>` for the given element. For more information, see
    /// the primitive function [`binary_search`].
    ///
    /// # Example
    ///
    /// ```
    /// use serde_unaligned::ZeroVec;
    ///
    /// let bytes: &[u8] = &[0xD3, 0x00, 0x19, 0x01, 0xA5, 0x01, 0xCD, 0x01];
    /// let zerovec: ZeroVec<u16> = ZeroVec::try_from_bytes(bytes).expect("infallible");
    ///
    /// assert_eq!(zerovec.binary_search(&281), Ok(1));
    /// assert_eq!(zerovec.binary_search(&282), Err(2));
    /// ```
    ///
    /// [`binary_search`]: https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search
    #[inline(always)]
    pub fn binary_search(&self, x: &T) -> Result<usize, usize> {
        self.as_slice()
            .binary_search_by(|probe| T::from_unaligned(probe).cmp(x))
    }
}
