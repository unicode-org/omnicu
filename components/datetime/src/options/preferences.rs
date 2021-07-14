// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Preferences is a bag of options to be associated with either [`length::Bag`] or [`components::Bag`]
//! which provides information on user preferences that can affect the result of the formatting.
//!
//! [`length::Bag`]: crate::options::length::Bag
//! [`components::Bag`]: crate::options::components::Bag
//!
//! # Unicode Extensions
//! User preferences will often be stored as part of the [`Locale`] identified as `Unicode Extensions`, but
//! for scenarios where the application stores information about user preferences they can be also provided via
//! this bag (and if they are, they will take precedence over unicode extensions from the locale).
//!
//! [`Locale`]: icu_locid::Locale
//!
//! # Examples
//!
//! ```
//! use icu::datetime::options::preferences;
//!
//! let prefs = preferences::Bag {
//!     hour_cycle: Some(preferences::HourCycle::H23)
//! };
//! ```
use crate::fields;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Stores user preferences which may affect the result of date and time formatting.
///
/// # Examples
///
/// ```
/// use icu::datetime::options::preferences;
///
/// let prefs = preferences::Bag {
///     hour_cycle: Some(preferences::HourCycle::H23)
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bag {
    #[cfg_attr(feature = "serde", serde(rename = "hourCycle"))]
    pub hour_cycle: Option<HourCycle>,
}

/// A user preference for adjusting how the hour component is displayed.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum HourCycle {
    /// Hour is formatted to be in range 1-24 where midnight is 24:00.
    ///
    /// # Examples
    ///
    /// ```
    /// "24:12";
    /// "8:23";
    /// "19:00";
    /// "23:21";
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "h24"))]
    H24,
    /// Hour is formatted to be in range 0-23 where midnight is 00:00.
    ///
    /// # Examples
    ///
    /// ```
    /// "0:12";
    /// "8:23";
    /// "19:00";
    /// "23:21";
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "h23"))]
    H23,
    /// Hour is formatted to be in range 1-12 where midnight is 12:00.
    ///
    /// # Examples
    ///
    /// ```
    /// "1:12";
    /// "8:23";
    /// "7:00";
    /// "11:21";
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "h12"))]
    H12,
    /// Hour is formatted to be in range 0-11 where midnight is 00:00.
    ///
    /// # Examples
    ///
    /// ```
    /// "0:12";
    /// "8:23";
    /// "7:00";
    /// "11:21";
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "h11"))]
    H11,
}

impl HourCycle {
    pub fn field(self) -> fields::Hour {
        match self {
            Self::H11 => fields::Hour::H11,
            Self::H12 => fields::Hour::H12,
            Self::H23 => fields::Hour::H23,
            Self::H24 => fields::Hour::H24,
        }
    }
}
