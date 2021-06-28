// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! # Implementation status
//!
//! This is currently only a partial implementation of the UTS-35 skeleton matching algorithm.
//!
//! | Algorithm step | Status |
//! |----------------|--------|
//! | Match skeleton fields according to a ranking             | Implemented |
//! | Adjust the matched pattern to have certain widths        | Implemented |
//! | Match date and times separately, and them combine them   | Implemented |
//! | Use appendItems to fill in a pattern with missing fields | Not yet, and may not be fully implemented. See [issue #586](https://github.com/unicode-org/icu4x/issues/586) |
//!
//! # Description
//!
//! A [`components::Bag`](struct.Bag.html) is a model of encoding information on how to format date
//! and time by specifying a list of components the user wants to be visible in the formatted string
//! and how each field should be displayed.
//!
//! This model closely corresponds to `ECMA402` API and allows for high level of customization
//! compared to `Length` model.
//!
//! Additionally, the bag contains an optional set of `Preferences` which represent user
//! preferred adjustments that can be applied onto the pattern right before formatting.
//!
//! ## Pattern Selection
//!
//! The [`components::Bag`](struct.Bag.html) is a way for the developer to describe which components
//! should be included in in a datetime, and how they should be displayed. There is not a strict
//! guarantee in how the final date will be displayed to the end user. The user's preferences and
//! locale information can override the developer preferences.
//!
//! The fields in the [`components::Bag`](struct.Bag.html) are matched against available patterns in
//! the `CLDR` locale data. A best fit is found, and presented to the user. This means that in
//! certain situations, and component combinations, fields will not have a match, or the match will
//! have a different type of presentation for a given locale.
//!
//!
//! # Examples
//!
//! ```
//! use icu::datetime::DateTimeFormatOptions;
//! use icu::datetime::options::components;
//!
//! let bag = components::Bag {
//!     year: Some(components::Numeric::Numeric),
//!     month: Some(components::Month::Long),
//!     day: Some(components::Numeric::Numeric),
//!
//!     hour: Some(components::Numeric::TwoDigit),
//!     minute: Some(components::Numeric::TwoDigit),
//!
//!     preferences: None,
//!
//!     ..Default::default()
//! };
//!
//! // The options can be created manually.
//! let options = DateTimeFormatOptions::Components(bag);
//! ```
//!
//! Or the options can be inferred through the `.into()` trait.
//!
//! ```
//! use icu::datetime::DateTimeFormatOptions;
//! use icu::datetime::options::components;
//! let options: DateTimeFormatOptions = components::Bag::default().into();
//! ```
//!
//! *Note*: The exact result returned from [`DateTimeFormat`](crate::DateTimeFormat) is a subject to change over
//! time. Formatted result should be treated as opaque and displayed to the user as-is,
//! and it is strongly recommended to never write tests that expect a particular formatted output.
use crate::fields::{self, Field, FieldLength, FieldSymbol};

use super::preferences;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// See the [module-level](./index.html) docs for more information.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bag {
    pub era: Option<Text>,
    pub year: Option<Numeric>,
    pub month: Option<Month>,
    pub day: Option<Numeric>,
    pub weekday: Option<Text>,

    pub hour: Option<Numeric>,
    pub minute: Option<Numeric>,
    pub second: Option<Numeric>,

    pub time_zone_name: Option<TimeZoneName>,

    pub preferences: Option<preferences::Bag>,
}

impl Bag {
    /// Converts the components::Bag into a Vec<Field>. The fields will be ordered in from most
    /// significant field to least significant. This is the order the fields are listed in
    /// the UTS 35 table - https://unicode.org/reports/tr35/tr35-dates.html#Date_Field_Symbol_Table
    pub(crate) fn to_vec_fields(&self) -> Vec<Field> {
        let mut fields = Vec::new();
        if let Some(_era) = self.era {
            unimplemented!("FieldSymbol::Era is needed. See issue #486.")
        }

        if let Some(year) = self.year {
            // Unimplemented year fields:
            // Y - Week of Year
            // u - Extended year
            // U - Cyclic year name
            // r - Related Gregorian year
            fields.push(Field {
                symbol: FieldSymbol::Year(fields::Year::Calendar),
                length: match year {
                    // Calendar year (numeric).
                    // y       2, 20, 201, 2017, 20173
                    // yy      02, 20, 01, 17, 73
                    // yyy     002, 020, 201, 2017, 20173    (not implemented)
                    // yyyy    0002, 0020, 0201, 2017, 20173 (not implemented)
                    // yyyyy+  ...                           (not implemented)
                    Numeric::Numeric => FieldLength::One,
                    Numeric::TwoDigit => FieldLength::TwoDigit,
                },
            });
        }

        // TODO(#501) - Unimplemented quarter fields:
        // Q - Quarter number/name
        // q - Stand-alone quarter

        if let Some(month) = self.month {
            fields.push(Field {
                // Always choose Month::Format as Month::StandAlone is not used in skeletons.
                symbol: FieldSymbol::Month(fields::Month::Format),
                length: match month {
                    // (intended to be used in conjunction with ‘d’ for day number).
                    // M      9, 12      Numeric: minimum digits
                    // MM     09, 12     Numeric: 2 digits, zero pad if needed
                    // MMM    Sep        Abbreviated
                    // MMMM   September  Wide
                    // MMMMM  S          Narrow
                    Month::Numeric => FieldLength::One,
                    Month::TwoDigit => FieldLength::TwoDigit,
                    Month::Long => FieldLength::Wide,
                    Month::Short => FieldLength::Abbreviated,
                    Month::Narrow => FieldLength::Narrow,
                },
            });
        }

        // TODO(#502) - Unimplemented week fields:
        // w - Week of year
        // W - Week of month

        if let Some(day) = self.day {
            // TODO(#591,#592) Unimplemented day fields:
            // D - Day of year
            // F - Day of week in month
            // g - Modified Julian day.
            fields.push(Field {
                symbol: FieldSymbol::Day(fields::Day::DayOfMonth),
                length: match day {
                    // Day of month (numeric).
                    // d    1 	  Numeric: minimum digits
                    // dd   01 	  Numeric: 2 digits, zero pad if needed
                    Numeric::Numeric => FieldLength::One,
                    Numeric::TwoDigit => FieldLength::TwoDigit,
                },
            });
        }

        if let Some(weekday) = self.weekday {
            // TODO(#593) Unimplemented fields
            // e - Local day of week.
            // c - Stand-alone local day of week.
            fields.push(Field {
                symbol: FieldSymbol::Weekday(fields::Weekday::Format),
                length: match weekday {
                    // Day of week name, format length.
                    //
                    // E..EEE   Tue      Abbreviated
                    // EEEE     Tuesday  Wide
                    // EEEEE    T 	     Narrow
                    // EEEEEE   Tu       Short
                    Text::Long => FieldLength::Wide,
                    Text::Short => FieldLength::One,
                    Text::Narrow => FieldLength::Narrow,
                },
            });
        }

        // The period fields are not included in skeletons:
        // a - AM, PM
        // b - am, pm, noon, midnight
        // c - flexible day periods

        if let Some(hour) = self.hour {
            // fields::Hour::H11
            // fields::Hour::H12
            // fields::Hour::H23
            // fields::Hour::H24

            // When used in skeleton data or in a skeleton passed in an API for flexible date
            // pattern generation, it should match the 12-hour-cycle format preferred by the
            // locale (h or K); it should not match a 24-hour-cycle format (H or k).
            fields.push(Field {
                symbol: FieldSymbol::Hour(match self.preferences {
                    Some(preferences::Bag {
                        hour_cycle: Some(hour_cycle),
                    }) => match hour_cycle {
                        // k - symbol
                        preferences::HourCycle::H24 => fields::Hour::H24,
                        // H - symbol
                        preferences::HourCycle::H23 => fields::Hour::H23,
                        // h - symbol
                        preferences::HourCycle::H12 => fields::Hour::H12,
                        // K - symbol
                        preferences::HourCycle::H11 => fields::Hour::H11,
                    },
                    // TODO(#594) - This should default should be the locale default, which is
                    // region-based (h12 for US, h23 for GB, etc). This is in CLDR, but we need
                    // to load it as well as think about the best architecture for where that
                    // data loading code should reside.
                    _ => fields::Hour::H24,
                }),
                length: match hour {
                    // Example for h: (note that this is the same for k, K, and H)
                    // h     1, 12  Numeric: minimum digits
                    // hh   01, 12  Numeric: 2 digits, zero pad if needed
                    Numeric::Numeric => FieldLength::One,
                    Numeric::TwoDigit => FieldLength::TwoDigit,
                },
            });
        }

        if let Some(minute) = self.minute {
            // m   8, 59    Numeric: minimum digits
            // mm  08, 59   Numeric: 2 digits, zero pad if needed
            fields.push(Field {
                symbol: FieldSymbol::Minute,
                length: match minute {
                    Numeric::Numeric => FieldLength::One,
                    Numeric::TwoDigit => FieldLength::TwoDigit,
                },
            });
        }

        if let Some(second) = self.second {
            // s    8, 12    Numeric: minimum digits
            // ss  08, 12    Numeric: 2 digits, zero pad if needed
            fields.push(Field {
                symbol: FieldSymbol::Second(fields::Second::Second),
                length: match second {
                    Numeric::Numeric => FieldLength::One,
                    Numeric::TwoDigit => FieldLength::TwoDigit,
                },
            });
            // S - Not used in skeletons.
            // A - Milliseconds in day. Not used in skeletons.
        }

        // TODO(#583) - Implement:
        // if self.time_zone_name.is_some() {
        //     unimplemented!();
        // }

        debug_assert!(
            fields.windows(2).all(|f| f[0] < f[1]),
            "The fields are sorted and unique."
        );

        fields
    }
}

impl Default for Bag {
    fn default() -> Self {
        Self {
            era: None,
            year: None,
            month: None,
            day: None,
            weekday: None,

            hour: None,
            minute: None,
            second: None,

            time_zone_name: None,

            preferences: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Numeric {
    #[cfg_attr(feature = "serde", serde(rename = "numeric"))]
    Numeric,
    #[cfg_attr(feature = "serde", serde(rename = "two-digit"))]
    TwoDigit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Text {
    #[cfg_attr(feature = "serde", serde(rename = "long"))]
    Long,
    #[cfg_attr(feature = "serde", serde(rename = "short"))]
    Short,
    #[cfg_attr(feature = "serde", serde(rename = "narrow"))]
    Narrow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Month {
    #[cfg_attr(feature = "serde", serde(rename = "numeric"))]
    Numeric,
    #[cfg_attr(feature = "serde", serde(rename = "two-digit"))]
    TwoDigit,
    #[cfg_attr(feature = "serde", serde(rename = "long"))]
    Long,
    #[cfg_attr(feature = "serde", serde(rename = "short"))]
    Short,
    #[cfg_attr(feature = "serde", serde(rename = "narrow"))]
    Narrow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TimeZoneName {
    #[cfg_attr(feature = "serde", serde(rename = "long"))]
    Long,
    #[cfg_attr(feature = "serde", serde(rename = "short"))]
    Short,
}

#[cfg(test)]
mod test {
    use super::*;

    // Shorten these for terser tests.
    type Symbol = FieldSymbol;
    type Length = FieldLength;

    #[test]
    fn test_component_bag_to_vec_field() {
        let bag = Bag {
            year: Some(Numeric::Numeric),
            month: Some(Month::Long),
            day: Some(Numeric::Numeric),

            hour: Some(Numeric::Numeric),
            minute: Some(Numeric::Numeric),
            second: Some(Numeric::Numeric),

            ..Default::default()
        };
        assert_eq!(
            bag.to_vec_fields(),
            vec![
                (Symbol::Year(fields::Year::Calendar), Length::One).into(),
                (Symbol::Month(fields::Month::Format), Length::Wide).into(),
                (Symbol::Day(fields::Day::DayOfMonth), Length::One).into(),
                (Symbol::Hour(fields::Hour::H24), Length::One).into(),
                (Symbol::Minute, Length::One).into(),
                (Symbol::Second(fields::Second::Second), Length::One).into(),
            ]
        );
    }

    #[test]
    fn test_component_bag_to_vec_field2() {
        let bag = Bag {
            year: Some(Numeric::Numeric),
            month: Some(Month::TwoDigit),
            day: Some(Numeric::Numeric),
            ..Default::default()
        };
        assert_eq!(
            bag.to_vec_fields(),
            vec![
                (Symbol::Year(fields::Year::Calendar), Length::One).into(),
                (Symbol::Month(fields::Month::Format), Length::TwoDigit).into(),
                (Symbol::Day(fields::Day::DayOfMonth), Length::One).into(),
            ]
        );
    }
}
