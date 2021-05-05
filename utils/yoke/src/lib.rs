// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! This crate provides [`Yoke`], a data structure that allows one
//! to "yoke" Cow-like borrowed data types to their backing storage,
//! enabling one to use Cow (etc) in zero-copy deserialization
//! with dynamic lifetimes for the borrowed data, for example caching it.
//!
//! See the documentation of [`Yoke`] for more details.

// The lifetimes here are important for safety and explicitly writing
// them out is good even when redundant
#![allow(clippy::needless_lifetimes)]

mod cart;
mod yoke;
mod yokeable;

#[cfg(feature = "zerovec")]
mod zerovec_impls;

pub use crate::cart::{Cart, Cartable};
pub use crate::yoke::Yoke;
pub use crate::yokeable::Yokeable;