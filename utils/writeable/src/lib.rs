// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#![cfg_attr(not(test), no_std)]

//! `writeable` is a utility crate of the [`ICU4X`] project.
//!
//! It includes [`Writeable`], a core trait representing an object that can be written to a
//! sink implementing std::fmt::Write. It is an alternative to std::fmt::Display with the
//! addition of a function indicating the number of bytes to be written.
//!
//! Writeable improves upon std::fmt::Display in two ways:
//!
//! 1. More efficient, since the sink can pre-allocate bytes.
//! 2. Smaller code, since the format machinery can be short-circuited.
//!
//! Types implementing Writeable have a defaulted writeable_to_string function.
//! If desired, types implementing Writeable can manually implement ToString
//! to wrap writeable_to_string.
//!
//! # Examples
//!
//! ```
//! use writeable::Writeable;
//! use writeable::LengthHint;
//! use writeable::assert_writeable_eq;
//! use std::fmt;
//!
//! struct WelcomeMessage<'s>{
//!     pub name: &'s str,
//! }
//!
//! impl<'s> Writeable for WelcomeMessage<'s> {
//!     fn write_to<W: fmt::Write + ?Sized>(&self, sink: &mut W) -> fmt::Result {
//!         sink.write_str("Hello, ")?;
//!         sink.write_str(self.name)?;
//!         sink.write_char('!')?;
//!         Ok(())
//!     }
//!
//!     fn write_len(&self) -> LengthHint {
//!         // "Hello, " + '!' + length of name
//!         LengthHint::Exact(8 + self.name.len())
//!     }
//! }
//!
//! let message = WelcomeMessage { name: "Alice" };
//! assert_writeable_eq!("Hello, Alice!", &message);
//! ```
//!
//! [`ICU4X`]: ../icu/index.html

extern crate alloc;

mod ops;

use alloc::string::String;
use core::fmt;

/// A hint to help consumers of Writeable pre-allocate bytes before they call write_to.
///
/// LengthHint implements std::ops::Add and similar traits for easy composition.
///
/// See this issue for more info: <https://github.com/unicode-org/icu4x/issues/370>.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum LengthHint {
    /// Default value: no hint is provided.
    Undefined,

    /// An exact length hint. This value is expected to equal the actual length from write_to.
    Exact(usize),
}

impl LengthHint {
    /// Returns a recommendation for the number of bytes to pre-allocate.
    ///
    /// # Examples
    ///
    /// ```
    /// use writeable::Writeable;
    ///
    /// fn pre_allocate_string(w: &impl Writeable) -> String {
    ///     String::with_capacity(w.write_len().capacity())
    /// }
    /// ```
    pub fn capacity(&self) -> usize {
        match self {
            Self::Undefined => 0,
            Self::Exact(len) => *len,
        }
    }

    /// Returns whether the LengthHint indicates that the string is exactly 0 bytes long.
    pub fn is_zero(&self) -> bool {
        match self {
            Self::Undefined => false,
            Self::Exact(len) => *len == 0,
        }
    }
}

/// Writeable is an alternative to std::fmt::Display with the addition of a length function.
pub trait Writeable {
    /// Writes bytes to the given sink. Errors from the sink are bubbled up.
    fn write_to<W: fmt::Write + ?Sized>(&self, sink: &mut W) -> fmt::Result;

    /// Returns a hint for the number of bytes that will be written to the sink.
    ///
    /// Override this method if it can be computed quickly.
    fn write_len(&self) -> LengthHint {
        LengthHint::Undefined
    }

    /// Creates a new String with the data from this Writeable. Like ToString,
    /// but smaller and faster.
    ///
    /// Not intended to be overriden.
    fn writeable_to_string(&self) -> String {
        let mut output = String::with_capacity(self.write_len().capacity());
        self.write_to(&mut output)
            .expect("impl Write for String is infallible");
        output
    }
}

/// Testing macro for types implementing Writeable. The first argument should be a string, and
/// the second argument should be a `&dyn Writeable`.
///
/// The macro tests for equality of both string content and string length. If your Writeable
/// implementation returns an inexact string length, don't use this macro.
///
/// # Examples
///
/// ```
/// use writeable::Writeable;
/// use writeable::LengthHint;
/// use writeable::assert_writeable_eq;
/// use std::fmt;
///
/// struct Demo;
/// impl Writeable for Demo {
///     fn write_to<W: fmt::Write + ?Sized>(&self, sink: &mut W) -> fmt::Result {
///         sink.write_str("foo")
///     }
///     fn write_len(&self) -> LengthHint {
///         LengthHint::Exact(3)
///     }
/// }
///
/// assert_writeable_eq!("foo", &Demo);
/// assert_writeable_eq!("foo", &Demo, "Message: {}", "Hello World");
/// ```
#[macro_export]
macro_rules! assert_writeable_eq {
    ($expected_str:expr, $actual_writeable:expr $(,)?) => {
        {
            use $crate::Writeable;
            let writeable = $actual_writeable;
            assert_eq!($expected_str, writeable.writeable_to_string());
            if let $crate::LengthHint::Exact(len) = writeable.write_len() {
                assert_eq!($expected_str.len(), len);
            }
        }
    };

    ($expected_str:expr, $actual_writeable:expr, $($arg:tt)+) => {
        {
            use $crate::Writeable;
            let writeable = $actual_writeable;
            assert_eq!($expected_str, writeable.writeable_to_string(), $($arg)+);
            if let $crate::LengthHint::Exact(len) = writeable.write_len() {
                assert_eq!($expected_str.len(), len, $($arg)+);
            }
        }
    };
}
