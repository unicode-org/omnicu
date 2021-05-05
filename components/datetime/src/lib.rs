// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#![warn(missing_docs)]

//! `icu_datetime` is one of the [`ICU4X`] components.
//!
//! This API provides necessary functionality for formatting date and time to user readable textual representation.
//!
//! [`DateTimeFormat`] is the main structure of the component. It accepts a set of arguments which
//! allow it to collect necessary data from the [`DataProvider`], and once instantiated, can be
//! used to quickly format any date and time provided.
//!
//! # Examples
//!
//! ```
//! use icu::locid::Locale;
//! use icu::locid::macros::langid;
//! use icu::datetime::{DateTimeFormat, DateTimeFormatOptions, mock::datetime::MockDateTime, options::length};
//!
//! let provider = icu_testdata::get_provider();
//!
//! let locale: Locale = langid!("en").into();
//!
//! // See the next code example for a more ergonomic example with .into().
//! let options = DateTimeFormatOptions::Length(length::Bag {
//!     date: Some(length::Date::Medium),
//!     time: Some(length::Time::Short),
//!     ..Default::default()
//! });
//!
//! let dtf = DateTimeFormat::try_new(locale, &provider, &options)
//!     .expect("Failed to create DateTimeFormat instance.");
//!
//!
//! let date: MockDateTime = "2020-09-12T12:35:00".parse()
//!     .expect("Failed to parse date.");
//!
//! let formatted_date = dtf.format(&date);
//! assert_eq!(formatted_date.to_string(), "Sep 12, 2020, 12:35 PM");
//! ```
//!
//! The options can be created more ergonomically using the `Into` trait to automatically
//! convert a [`options::length::Bag`] into a [`DateTimeFormatOptions::Length`].
//!
//! ```
//! use icu::locid::Locale;
//! use icu::locid::macros::langid;
//! use icu::datetime::{DateTimeFormat, DateTimeFormatOptions, mock::datetime::MockDateTime, options::length};
//! # let provider = icu_testdata::get_provider();
//! # let locale: Locale = langid!("en").into();
//! let options = length::Bag {
//!     date: Some(length::Date::Medium),
//!     time: Some(length::Time::Short),
//!     ..Default::default()
//! }.into();
//!
//! let dtf = DateTimeFormat::try_new(locale, &provider, &options);
//! ```
//!
//! At the moment, the crate provides only options using the [`Length`] bag, but in the future,
//! we expect to add more ways to customize the output, like skeletons, and components.
//!
//! *Notice:* Rust at the moment does not have a canonical way to represent date and time. We are introducing
//! [`MockDateTime`] as an example of the data necessary for ICU [`DateTimeFormat`] to work, and
//! [we hope to work with the community](https://github.com/unicode-org/icu4x/blob/main/docs/research/datetime.md)
//! to develop core date and time APIs that will work as an input for this component.
//!
//! [`DataProvider`]: icu_provider::DataProvider
//! [`ICU4X`]: ../icu/index.html
//! [`Length`]: options::length
//! [`MockDateTime`]: mock::datetime::MockDateTime
mod arithmetic;
pub mod date;
pub mod datetime;
mod error;
mod fields;
mod format;
pub mod mock;
pub mod options;
#[allow(missing_docs)]
#[doc(hidden)]
pub mod pattern;
pub mod provider;
#[allow(missing_docs)]
#[doc(hidden)]
pub mod skeleton;
// TODO(#622) make the time_zone module public once TimeZoneFormat is public.
pub(crate) mod time_zone;
pub mod zoned_datetime;

pub use datetime::DateTimeFormat;
pub use error::DateTimeFormatError;
pub use format::datetime::FormattedDateTime;
pub use format::zoned_datetime::FormattedZonedDateTime;
pub use options::DateTimeFormatOptions;
pub use zoned_datetime::ZonedDateTimeFormat;
// TODO(#622) re-export TimeZoneFormat once it is made public.
