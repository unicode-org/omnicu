// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! [`icu_locale_canonicalizer`](crate) is one of the [`ICU4X`] components.
//!
//! This API provides functionality to canonicalize locale identifiers based
//! upon [`CLDR`] data.
//!
//! It currently supports the minimize and maximize likely subtags algorithms
//! as described in [`UTS 35`].
//!
//! The maximize method potentially updates a passed in locale in place
//! depending up the results of running the 'Add Likely Subtags' algorithm
//! from [`UTS 35`].
//!
//! This minimize method returns a new Locale that is the result of running the
//! 'Remove Likely Subtags' algorithm from [`UTS 35`].
//!
//! # Examples
//!
//! ```
//! # #[cfg(feature = "provider_serde")] {
//! use icu_locale_canonicalizer::{CanonicalizationResult, LocaleCanonicalizer};
//! use icu_locid::Locale;
//!
//! let provider = icu_testdata::get_provider();
//! let lc = LocaleCanonicalizer::new(&provider).unwrap();
//!
//! let mut locale : Locale = "zh-CN".parse().unwrap();
//! assert_eq!(lc.maximize(&mut locale), CanonicalizationResult::Modified);
//! assert_eq!(locale.to_string(), "zh-Hans-CN");
//!
//! let mut locale : Locale = "zh-Hant-TW".parse().unwrap();
//! assert_eq!(lc.maximize(&mut locale), CanonicalizationResult::Unmodified);
//! assert_eq!(locale.to_string(), "zh-Hant-TW");
//! # } // feature = "provider_serde"
//! ```
//!
//! ```
//! # #[cfg(feature = "provider_serde")] {
//! use icu_locale_canonicalizer::{CanonicalizationResult, LocaleCanonicalizer};
//! use icu_locid::Locale;
//!
//! let provider = icu_testdata::get_provider();
//! let lc = LocaleCanonicalizer::new(&provider).unwrap();
//!
//! let mut locale : Locale = "zh-Hans-CN".parse().unwrap();
//! assert_eq!(lc.minimize(&mut locale), CanonicalizationResult::Modified);
//! assert_eq!(locale.to_string(), "zh");
//!
//! let mut locale : Locale = "zh".parse().unwrap();
//! assert_eq!(lc.minimize(&mut locale), CanonicalizationResult::Unmodified);
//! assert_eq!(locale.to_string(), "zh");
//! # } // feature = "provider_serde"
//! ```
//!
//! [`ICU4X`]: ../icu/index.html
//! [`CLDR`]: http://cldr.unicode.org/
//! [`UTS 35`]: https://www.unicode.org/reports/tr35/#Likely_Subtags.

pub mod locale_canonicalizer;
pub mod provider;

pub use locale_canonicalizer::{CanonicalizationResult, LocaleCanonicalizer};
