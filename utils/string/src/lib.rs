// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! `icu_string` is a utility crate of the [`ICU4X`] project.
//!
//! It includes [`Slice`], a core trait that allows to build parsers that can be
//! generalized over different ownership and lifetime models of the input string and produce
//! a range of slice outputs.
//! This allows to generate parsers that depending on the desired input can be zero-copy, or
//! can perpetuate the ownership model of the input.
//!
//! # Example
//!
//! ```
//! use icu_string::Slice;
//!
//! /// Parse an input string slice and return a name parsed out of it.
//! ///
//! /// # Type parameters
//! ///
//! /// - `S`: A type of a slice provided as an input: `&str`, `String` or `Cow<str>`.
//! /// - `RS`: Return slice: `&str`, `String` or `Cow<str>`.
//! ///
//! /// # Lifetimes
//! ///
//! /// - `slice`: The life time used for scenarios where the input slice is borrowed,
//! ///            or when the output slice has to live for as long as the input slice.
//! fn parse_name<'slice, S, RS>(input: &'slice S) -> RS
//! where
//!     S: Slice<'slice, RS> + ?Sized,
//! {
//!     let len = input.length();
//!     input.get_slice(6..len)
//! }
//!
//! let input = "name: Example Name";
//! let name: &str = parse_name(&input);
//!
//! assert_eq!(name, "Example Name");
//! ```
//!
//! The example above will work over any combination of input and output `str`, `String` or
//! `Cow<str>` types allowing the author to write a parser and the caller decide what ownership
//! and life time models to use.
//!
//! For a more complete example of how the input/output pairs work, see `parse_slice` example.
//!
//! [`ICU4X`]: ../icu/index.html
mod cow;
mod str;
mod string;

use std::ops::Range;

/// A trait used to allow producing parsers generic over different ownership models of a string.
///
/// # Example
///
/// ```
/// use icu_string::Slice;
///
/// /// Parse an input string slice and return a name parsed out of it.
/// ///
/// /// # Type parameters
/// ///
/// /// - `S`: A type of a slice provided as an input: `&str`, `String` or `Cow<str>`.
/// /// - `RS`: Return slice: `&str`, `String` or `Cow<str>`.
/// ///
/// /// # Lifetimes
/// ///
/// /// - `slice`: The life time used for scenarios where the input slice is borrowed,
/// ///            or when the output slice has to live for as long as the input slice.
/// fn parse_name<'slice, S, RS>(input: &'slice S) -> RS
/// where
///     S: Slice<'slice, RS> + ?Sized,
/// {
///     let len = input.length();
///     input.get_slice(6..len)
/// }
///
/// let input = "name: Example Name";
/// let name: &str = parse_name(&input);
///
/// assert_eq!(name, "Example Name");
/// ```
pub trait Slice<'s, RS> {
    /// Retrieves a single byte at a given position, or `None` if the requested
    /// `idx` is out of string bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_string::Slice;
    ///
    /// fn is_first_byte_H<'s, S: Slice<'s, &'s str>>(input: S) -> bool {
    ///     input.get_byte(0) == Some(b'H')
    /// }
    ///
    /// assert_eq!(is_first_byte_H("Hello World"), true);
    /// ```
    fn get_byte(&self, idx: usize) -> Option<u8>;

    /// Retrieves an `&str` of a range of bytes.
    ///
    /// This is useful for scenarios where the function operating on the slice
    /// needs to perform an internal operation that doesn't need to return the sliced
    /// value.
    ///
    /// For cases where the output is itended to preserve the ownership/lifetime model,
    /// see `get_slice`.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_string::Slice;
    ///
    /// fn starts_with_hello<'s, S: Slice<'s, &'s str>>(input: S) -> bool {
    ///     input.get_str_slice(0..5) == "Hello"
    /// }
    ///
    /// assert_eq!(starts_with_hello("Hello World"), true);
    /// ```
    fn get_str_slice(&self, range: Range<usize>) -> &str;

    /// Retrieves a slice of the input string perpetuating the
    /// ownership and lifetime model.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_string::Slice;
    ///
    /// fn get_word<'s, S: Slice<'s, &'s str>>(input: &'s S) -> &'s str {
    ///     input.get_slice(0..5)
    /// }
    ///
    /// assert_eq!(get_word(&"Hello World"), "Hello");
    /// ```
    fn get_slice(&'s self, range: Range<usize>) -> RS;

    /// Retrieves a length of the input slice.
    ///
    /// # Example
    ///
    /// ```
    /// use icu_string::Slice;
    ///
    /// fn iterate<'s, S: Slice<'s, &'s str>>(input: S) -> impl Iterator<Item = u8> {
    ///     let mut idx = 0;
    ///     std::iter::from_fn(move || {
    ///         let result = input.get_byte(idx)?;
    ///         idx += 1;
    ///         Some(result)
    ///     })
    /// }
    ///
    /// assert_eq!(iterate("Hello").collect::<Vec<_>>(), vec![b'H', b'e', b'l', b'l', b'o']);
    /// ```
    fn length(&self) -> usize;
}

pub use crate::cow::*;
pub use crate::str::*;
pub use crate::string::*;
