// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Length is a model of encoding information on how to format date and time by specifying the preferred length
//! of date and time fields.
//!
//! If either of the fields is omitted, the value will be formatted according to the pattern associated with the
//! preferred length of the present field in a given locale.
//!
//! If both fields are present, both parts of the value will be formatted and an additional connector pattern
//! will be used to construct a full result.
//! The type of the connector is determined by the length of the [`Date`] field.
//!
//! Additionally, the bag contains an optional set of `Preferences` which represent user preferred adjustments
//! that can be applied onto the pattern right before formatting.
//!
//! # Examples
//!
//! ```
//! use icu::datetime::DateTimeFormatOptions;
//! use icu::datetime::options::length;
//!
//! let bag = length::Bag {
//!      date: Some(length::Date::Medium), // `Medium` length connector will be used
//!      time: Some(length::Time::Short),
//!      preferences: None,
//! };
//!
//! let options = DateTimeFormatOptions::Length(bag);
//! ```
//!
//! Or the options can be inferred through the [`Into`] trait.
//!
//! ```
//! use icu::datetime::DateTimeFormatOptions;
//! use icu::datetime::options::length;
//! let options: DateTimeFormatOptions = length::Bag::default().into();
//! ```
//!
//! *Note*: The exact result returned from [`DateTimeFormat`](crate::DateTimeFormat) is a subject to change over
//! time. Formatted result should be treated as opaque and displayed to the user as-is,
//! and it is strongly recommended to never write tests that expect a particular formatted output.

use super::preferences;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A structure to represent the set of lengths in which the [`DateTimeInput`] implementer should be formatted to.
///
/// [`DateTimeInput`]: crate::date::DateTimeInput
///
/// The available lengths correspond to [`UTS #35: Unicode LDML 4. Dates`], section 2.4 [`Element dateFormats`].
///
/// # Examples
///
/// ```
/// use icu::datetime::DateTimeFormatOptions;
/// use icu::datetime::options::length;
///
/// let bag = length::Bag {
///      date: Some(length::Date::Medium),
///      time: Some(length::Time::Short),
///      preferences: None,
/// };
///
/// let options = DateTimeFormatOptions::Length(bag);
/// ```
///
/// Or the options can be inferred through the [`Into`] trait.
///
/// ```
/// use icu::datetime::DateTimeFormatOptions;
/// use icu::datetime::options::length;
/// let options: DateTimeFormatOptions = length::Bag::default().into();
/// ```
///
/// [`UTS #35: Unicode LDML 4. Dates`]: https://unicode.org/reports/tr35/tr35-dates.html
/// [`Element dateFormats`]: https://unicode.org/reports/tr35/tr35-dates.html#dateFormats
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bag {
    pub date: Option<Date>,
    pub time: Option<Time>,
    pub preferences: Option<preferences::Bag>,
}

impl Default for Bag {
    fn default() -> Self {
        Self {
            date: Some(Date::Long),
            time: Some(Time::Long),
            preferences: None,
        }
    }
}

/// Represents different lengths a [`DateTimeInput`] implementer can be formatted into.
/// Each length has associated best pattern for it for a given locale.
///
/// [`DateTimeInput`]: crate::date::DateTimeInput
///
/// # Examples
///
/// ```
/// use icu::datetime::options::length;
///
/// let bag = length::Bag {
///     date: Some(length::Date::Long),
///     time: None,
///
///     preferences: None,
/// };
/// ```
///
/// The available lengths correspond to [`UTS #35: Unicode LDML 4. Dates`], section 2.4 [`Element dateFormats`].
///
/// *Note*: The exact result returned from [`DateTimeFormat`] is a subject to change over
/// time. Formatted result should be treated as opaque and displayed to the user as-is,
/// and it is strongly recommended to never write tests that expect a particular formatted output.
///
/// [`UTS #35: Unicode LDML 4. Dates`]: https://unicode.org/reports/tr35/tr35-dates.html
/// [`Element dateFormats`]: https://unicode.org/reports/tr35/tr35-dates.html#dateFormats
/// [`DateTimeFormat`]: super::super::DateTimeFormat
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Date {
    /// Full length, usually with weekday name.
    ///
    /// # Examples
    ///
    /// ```
    /// "Tuesday, January 21, 2020";       // en-US
    /// "wtorek, 21 stycznia, 2020";       // pl
    /// "الثلاثاء، ٢١ يناير ٢٠٢٠";          // ar
    /// "вторник, 21 января 2020 г.";      // ru
    /// "2020年1月21日火曜日";               // ja
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "full"))]
    Full,
    /// Long length, with wide month name.
    ///
    /// # Examples
    ///
    /// ```
    /// "September 10, 2020";     // en-US
    /// "10 września 2020";       // pl
    /// "١٠ سبتمبر ٢٠٢٠";         // ar
    /// "10 сентября 2020 г.";    // ru
    /// "2020年9月10日";           // ja
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "long"))]
    Long,
    /// Medium length.
    ///
    /// # Examples
    ///
    /// ```
    /// "Feb 20, 2020";        // en-US
    /// "20 lut 2020";         // pl
    /// "٢٠‏/٠٢‏/٢٠٢٠";          // ar
    /// "20 февр. 2020 г.";    // ru
    /// "2020/02/20";          // ja
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "medium"))]
    Medium,
    /// Short length, usually with numeric month.
    ///
    /// # Examples
    ///
    /// ```
    /// "1/30/20";      // en-US
    /// "30.01.2020";   // pl
    /// "٣٠‏/١‏/٢٠٢٠";    // ar
    /// "30.01.2020";   // ru
    /// "2020/01/30";   // ja
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "short"))]
    Short,
}

/// Represents different length lengths a [`DateTimeInput`] implementer can be formatted into.
/// Each length has associated best pattern for it for a given locale.
///
/// [`DateTimeInput`]: crate::date::DateTimeInput
///
/// # Examples
///
/// ```
/// use icu::datetime::options::length;
///
/// let bag = length::Bag {
///     date: None,
///     time: Some(length::Time::Medium),
///
///     preferences: None,
/// };
/// ```
///
/// The available lengths correspond to [`UTS #35: Unicode LDML 4. Dates`], section 2.4 [`Element timeFormats`].
///
/// *Note*: The exact result returned from [`DateTimeFormat`] is a subject to change over
/// time. Formatted result should be treated as opaque and displayed to the user as-is,
/// and it is strongly recommended to never write tests that expect a particular formatted output.
///
/// [`UTS #35: Unicode LDML 4. Dates`]: https://unicode.org/reports/tr35/tr35-dates.html
/// [`Element dateFormats`]: https://unicode.org/reports/tr35/tr35-dates.html#timeFormats
/// [`DateTimeFormat`]: super::super::DateTimeFormat
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Time {
    /// Full length, with spelled out time zone name.
    ///
    /// # Examples
    ///
    /// ```
    /// "8:25:07 AM Pacific Standard Time";              // en-US
    /// "08:25:07 czas pacyficzny standardowy";          // pl
    /// "٨:٢٥:٠٧ ص توقيت المحيط الهادي الرسمي";          // ar
    /// "08:25:07 Тихоокеанское стандартное время";      // ru
    /// "8時25分07秒 アメリカ太平洋標準時";                  // ja
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "full"))]
    Full,
    /// Full length, usually with short time-zone code.
    ///
    /// # Examples
    ///
    /// ```
    /// "8:25:07 AM PST";       // en-US
    /// "08:25:07 GMT-8";       // pl
    /// "٨:٢٥:٠٧ ص غرينتش-٨";   // ar
    /// "08:25:07 GMT-8";       // ru
    /// "8:25:07 GMT-8";        // ja
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "long"))]
    Long,
    /// Full length, usually with seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// "8:25:07 AM";   // en-US
    /// "08:25:07";     // pl
    /// "٨:٢٥:٠٧ ص";    // ar
    /// "08:25:07";     // ru
    /// "8:25:07";      // ja
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "medium"))]
    Medium,
    /// Full length, usually without seconds.
    ///
    /// # Examples
    ///
    /// ```
    /// "8:25 AM";   // en-US
    /// "08:25";     // pl
    /// "٨:٢٥ ص";    // ar
    /// "08:25";     // ru
    /// "8:25";      // ja
    /// ```
    #[cfg_attr(feature = "serde", serde(rename = "short"))]
    Short,
}
