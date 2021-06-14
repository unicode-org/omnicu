// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use std::{borrow::Cow, fmt};

use crate::{
    date::TimeZoneInput, format::time_zone::FormattedTimeZone, pattern::Error as PatternError,
    provider, DateTimeFormatError,
};
use crate::{format::time_zone, provider::time_zones::TimeZoneFormatsV1Marker};
use icu_locid::{LanguageIdentifier, Locale};
use icu_provider::prelude::*;

use crate::fields::{FieldSymbol, TimeZone};
use crate::pattern::{Pattern, PatternItem};

/// Loads a resource into its destination if the destination has not already been filled.
fn load_resource<'d, D, L, P>(
    locale: &L,
    resource_key: ResourceKey,
    destination: &mut Option<DataPayload<'d, 'd, D>>,
    provider: &P,
) -> Result<(), DateTimeFormatError>
where
    D: DataMarker<'d>,
    L: Clone + Into<LanguageIdentifier>,
    P: DataProvider<'d, 'd, D> + ?Sized,
{
    if destination.is_none() {
        *destination = Some(
            provider
                .load_payload(&DataRequest {
                    resource_path: ResourcePath {
                        key: resource_key,
                        options: ResourceOptions {
                            variant: None,
                            langid: Some(locale.clone().into()),
                        },
                    },
                })?
                .take_payload()?,
        );
    }
    Ok(())
}

/// [`TimeZoneFormat`] uses data from the [`DataProvider`], the selected [`Locale`], and the provided
/// pattern to collect all data necessary to format time zones into that locale.
///
/// The various time-zone pattern symbols specified in UTS-35 require different sets of data for
/// formatting. As such,[`TimeZoneFormat`] will pull in only the resources needed to format the
/// pattern that it is given upon construction.
///
/// For that reason, one should think of the process of formatting a time zone in two steps:
/// first, a computationally heavy construction of [`TimeZoneFormat`], and then fast formatting
/// of the time-zone data using the instance.
///
/// # Examples
///
/// // TODO(#622) Uncomment and update example once TimeZoneFormat is public.
/// // ```
/// // use icu_locid::Locale;
/// // use icu_locid::macros::langid;
/// // use icu_datetime::TimeZoneFormat;
/// // use icu_datetime::date::GmtOffset;
/// // use icu_datetime::mock::time_zone::MockTimeZone;
/// // use icu_provider::inv::InvariantDataProvider;
///
/// // let locale: Locale = langid!("en").into();
/// // let pattern = std::iter::empty().collect();
/// // let provider = InvariantDataProvider;
///
/// // let tzf = TimeZoneFormat::try_new(locale, pattern, &provider)
/// //     .expect("Failed to create TimeZoneFormat");
///
/// // let time_zone = MockTimeZone::new(
/// //        GmtOffset::default(),
/// //        None,
/// //        None,
/// //        None,
/// // );
///
/// // let value = tzf.format_to_string(&time_zone);
/// // ```
// TODO(#622) Make TimeZoneFormat public once we have a clean way to provide it options.
pub(super) struct TimeZoneFormat<'d> {
    /// The pattern to format.
    pub(super) pattern: Pattern<'d>,
    /// The data that contains meta information about how to display content.
    pub(super) zone_formats: DataPayload<'d, 'd, provider::time_zones::TimeZoneFormatsV1Marker>,
    /// The exemplar cities for time zones.
    pub(super) exemplar_cities:
        Option<DataPayload<'d, 'd, provider::time_zones::ExemplarCitiesV1Marker>>,
    /// The generic long metazone names, e.g. Pacific Time
    pub(super) mz_generic_long:
        Option<DataPayload<'d, 'd, provider::time_zones::MetaZoneGenericNamesLongV1Marker>>,
    /// The generic short metazone names, e.g. PT
    pub(super) mz_generic_short:
        Option<DataPayload<'d, 'd, provider::time_zones::MetaZoneGenericNamesShortV1Marker>>,
    /// The specific long metazone names, e.g. Pacific Daylight Time
    pub(super) mz_specific_long:
        Option<DataPayload<'d, 'd, provider::time_zones::MetaZoneSpecificNamesLongV1Marker>>,
    /// The specific short metazone names, e.g. Pacific Daylight Time
    pub(super) mz_specific_short:
        Option<DataPayload<'d, 'd, provider::time_zones::MetaZoneSpecificNamesShortV1Marker>>,
}

impl<'d> TimeZoneFormat<'d> {
    /// Constructor that selectively loads data based on what is required to
    /// format the given pattern into the given locale.
    ///
    /// # Examples
    ///
    /// // TODO(#622) Uncomment and update example once TimeZoneFormat is public.
    /// // ```
    /// // use icu_locid::Locale;
    /// // use icu_locid::macros::langid;
    /// // use icu_datetime::TimeZoneFormat;
    /// // use icu_datetime::mock::time_zone::MockTimeZone;
    /// // use icu_provider::inv::InvariantDataProvider;
    ///
    /// // let locale: Locale = langid!("en").into();
    /// // let pattern = std::iter::empty().collect();
    /// // let provider = InvariantDataProvider;
    ///
    /// // let tzf = TimeZoneFormat::try_new(locale, pattern, &provider);
    ///
    /// // assert!(tzf.is_ok());
    /// // ```
    // TODO(#622) Make this public once TimeZoneFormat is public.
    pub(super) fn try_new<L, ZP>(
        locale: L,
        pattern: Pattern<'d>,
        zone_provider: &ZP,
    ) -> Result<Self, DateTimeFormatError>
    where
        L: Into<Locale>,
        ZP: DataProvider<'d, 'd, provider::time_zones::TimeZoneFormatsV1Marker>
            + DataProvider<'d, 'd, provider::time_zones::ExemplarCitiesV1Marker>
            + DataProvider<'d, 'd, provider::time_zones::MetaZoneGenericNamesLongV1Marker>
            + DataProvider<'d, 'd, provider::time_zones::MetaZoneGenericNamesShortV1Marker>
            + DataProvider<'d, 'd, provider::time_zones::MetaZoneSpecificNamesLongV1Marker>
            + DataProvider<'d, 'd, provider::time_zones::MetaZoneSpecificNamesShortV1Marker>
            + ?Sized,
    {
        let locale = locale.into();

        let zone_formats: DataPayload<TimeZoneFormatsV1Marker> = zone_provider
            .load_payload(&DataRequest {
                resource_path: ResourcePath {
                    key: provider::key::TIMEZONE_FORMATS_V1,
                    options: ResourceOptions {
                        variant: None,
                        langid: Some(locale.clone().into()),
                    },
                },
            })?
            .take_payload()?;

        let mut time_zone_format = Self {
            pattern,
            zone_formats,
            exemplar_cities: None,
            mz_generic_long: None,
            mz_generic_short: None,
            mz_specific_long: None,
            mz_specific_short: None,
        };

        let zone_symbols = time_zone_format
            .pattern
            .items()
            .iter()
            .filter_map(|item| match item {
                PatternItem::Field(field) => Some(field),
                _ => None,
            })
            .filter_map(|field| match field.symbol {
                FieldSymbol::TimeZone(zone) => Some((u8::from(field.length), zone)),
                _ => None,
            });

        for (length, symbol) in zone_symbols {
            match symbol {
                TimeZone::LowerZ => match length {
                    1..=3 => load_resource(
                        &locale,
                        provider::key::TIMEZONE_SPECIFIC_NAMES_SHORT_V1,
                        &mut time_zone_format.mz_specific_short,
                        zone_provider,
                    )?,
                    4 => load_resource(
                        &locale,
                        provider::key::TIMEZONE_SPECIFIC_NAMES_LONG_V1,
                        &mut time_zone_format.mz_specific_long,
                        zone_provider,
                    )?,
                    _ => {
                        return Err(DateTimeFormatError::Pattern(
                            PatternError::FieldLengthInvalid(FieldSymbol::TimeZone(symbol)),
                        ))
                    }
                },
                TimeZone::LowerV => match length {
                    1 => {
                        load_resource(
                            &locale,
                            provider::key::TIMEZONE_GENERIC_NAMES_SHORT_V1,
                            &mut time_zone_format.mz_generic_short,
                            zone_provider,
                        )?;
                        load_resource(
                            &locale,
                            provider::key::TIMEZONE_EXEMPLAR_CITIES_V1,
                            &mut time_zone_format.exemplar_cities,
                            zone_provider,
                        )?;
                    }
                    4 => {
                        load_resource(
                            &locale,
                            provider::key::TIMEZONE_GENERIC_NAMES_LONG_V1,
                            &mut time_zone_format.mz_generic_long,
                            zone_provider,
                        )?;
                        load_resource(
                            &locale,
                            provider::key::TIMEZONE_EXEMPLAR_CITIES_V1,
                            &mut time_zone_format.exemplar_cities,
                            zone_provider,
                        )?;
                    }
                    _ => {
                        return Err(DateTimeFormatError::Pattern(
                            PatternError::FieldLengthInvalid(FieldSymbol::TimeZone(symbol)),
                        ))
                    }
                },
                TimeZone::UpperV => match length {
                    1 => (), // BCP-47 identifier, no CLDR-data necessary.
                    2 => (), // IANA time-zone ID, no CLDR data necessary.
                    3 | 4 => load_resource(
                        &locale,
                        provider::key::TIMEZONE_EXEMPLAR_CITIES_V1,
                        &mut time_zone_format.exemplar_cities,
                        zone_provider,
                    )?,
                    _ => {
                        return Err(DateTimeFormatError::Pattern(
                            PatternError::FieldLengthInvalid(FieldSymbol::TimeZone(symbol)),
                        ))
                    }
                },
                // ISO-8601 or localized GMT formats. CLDR data is either unneeded or required by default.
                TimeZone::LowerX | TimeZone::UpperX | TimeZone::UpperZ | TimeZone::UpperO => (),
            }
        }

        Ok(time_zone_format)
    }

    /// Takes a [`TimeZoneInput`] implementer and returns an instance of a [`FormattedTimeZone`]
    /// that contains all information necessary to display a formatted time zone and operate on it.
    ///
    /// # Examples
    ///
    /// // TODO(#622) Uncomment and update example once TimeZoneFormat is public.
    /// // ```
    /// // use icu_locid::Locale;
    /// // use icu_locid::macros::langid;
    /// // use icu_datetime::TimeZoneFormat;
    /// // use icu_datetime::date::GmtOffset;
    /// // use icu_datetime::mock::time_zone::MockTimeZone;
    /// // use icu_provider::inv::InvariantDataProvider;
    ///
    /// // # let locale: Locale = langid!("en").into();
    /// // # let pattern = std::iter::empty().collect();
    /// // # let provider = InvariantDataProvider;
    ///
    /// // let tzf = TimeZoneFormat::try_new(locale, pattern, &provider)
    /// //     .expect("Failed to create TimeZoneFormat");
    ///
    /// // let time_zone = MockTimeZone::new(
    /// //        GmtOffset::default(),
    /// //        None,
    /// //        None,
    /// //        None,
    /// // );
    ///
    /// // let _ = tzf.format(&time_zone);
    /// // ```
    // TODO(#622) Make this public once TimeZoneFormat is public.
    //           And remove #[allow(unused)]
    #[allow(unused)]
    pub(super) fn format<'l: 'd, T>(&'l self, value: &'l T) -> FormattedTimeZone<'l, T>
    where
        T: TimeZoneInput,
    {
        FormattedTimeZone {
            time_zone_format: self,
            time_zone: value,
        }
    }

    /// Takes a mutable reference to anything that implements the [`Write`](std::fmt::Write)
    /// trait and a [`TimeZoneInput`] implementer that populates the buffer with a formatted value.
    ///
    /// # Examples
    ///
    /// // TODO(#622) Uncomment and update example once TimeZoneFormat is public.
    /// // ```
    /// // use icu_locid::Locale;
    /// // use icu_locid::macros::langid;
    /// // use icu_datetime::TimeZoneFormat;
    /// // use icu_datetime::date::GmtOffset;
    /// // use icu_datetime::mock::time_zone::MockTimeZone;
    /// // use icu_provider::inv::InvariantDataProvider;
    ///
    /// // # let locale: Locale = langid!("en").into();
    /// // # let pattern = std::iter::empty().collect();
    /// // # let provider = InvariantDataProvider;
    ///
    /// // let tzf = TimeZoneFormat::try_new(locale, pattern, &provider)
    /// //     .expect("Failed to create TimeZoneFormat");
    ///
    /// // let time_zone = MockTimeZone::new(
    /// //        GmtOffset::default(),
    /// //        None,
    /// //        None,
    /// //        None,
    /// // );
    ///
    /// // let mut buffer = String::new();
    /// // tzf.format_to_write(&mut buffer, &time_zone)
    /// //     .expect("Failed to write to a buffer.");
    ///
    /// // let _ = format!("Time Zone: {}", buffer);
    /// // ```
    // TODO(#622) Make this public once TimeZoneFormat is public.
    //            And remove #[allow(unused)]
    #[allow(unused)]
    pub(super) fn format_to_write(
        &self,
        w: &mut impl std::fmt::Write,
        value: &impl TimeZoneInput,
    ) -> fmt::Result {
        time_zone::write_pattern(self, value, w).map_err(|_| std::fmt::Error)
    }

    /// Takes a [`TimeZoneInput`] implementer and returns a string with the formatted value.
    ///
    /// # Examples
    ///
    /// // TODO(#622) Uncomment and update example once TimeZoneFormat is public.
    /// // ```
    /// // use icu_locid::Locale;
    /// // use icu_locid::macros::langid;
    /// // use icu_datetime::TimeZoneFormat;
    /// // use icu_datetime::date::GmtOffset;
    /// // use icu_datetime::mock::time_zone::MockTimeZone;
    /// // use icu_provider::inv::InvariantDataProvider;
    ///
    /// // # let locale: Locale = langid!("en").into();
    /// // # let pattern = std::iter::empty().collect();
    /// // # let provider = InvariantDataProvider;
    ///
    /// // let tzf = TimeZoneFormat::try_new(locale, pattern, &provider)
    /// //     .expect("Failed to create TimeZoneFormat");
    ///
    /// // let time_zone = MockTimeZone::new(
    /// //        GmtOffset::default(),
    /// //        None,
    /// //        None,
    /// //        None,
    /// // );
    ///
    /// // let _ = tzf.format_to_string(&time_zone);
    /// // ```
    // TODO(#622) Make this public once TimeZoneFormat is public.
    //           And remove #[allow(unused)]
    #[allow(unused)]
    pub(super) fn format_to_string(&self, value: &impl TimeZoneInput) -> String {
        let mut s = String::new();
        self.format_to_write(&mut s, value)
            .expect("Failed to write to a String.");
        s
    }

    /// Writes the time zone in generic location format as defined by the UTS-35 spec.
    /// e.g. France Time
    /// https://unicode.org/reports/tr35/tr35-dates.html#Time_Zone_Format_Terminology
    pub(super) fn generic_location_format<W: fmt::Write + ?Sized>(
        &self,
        sink: &mut W,
        time_zone: &impl TimeZoneInput,
    ) -> Result<(), DateTimeFormatError> {
        // TODO(blocked on #277) Use formatter utility instead of replacing "{0}".
        sink.write_str(
            &self
                .exemplar_cities
                .as_ref()
                .map(|p| p.get())
                .and_then(|cities| time_zone.time_zone_id().and_then(|id| cities.get(id)))
                .map(|location| {
                    self.zone_formats
                        .get()
                        .region_format
                        .replace("{0}", location)
                })
                .ok_or(fmt::Error)?,
        )
        .map_err(DateTimeFormatError::from)
    }

    /// Writes the time zone in short generic non-location format as defined by the UTS-35 spec.
    /// e.g. PT
    /// https://unicode.org/reports/tr35/tr35-dates.html#Time_Zone_Format_Terminology
    pub(super) fn short_generic_non_location_format<W: fmt::Write + ?Sized>(
        &self,
        sink: &mut W,
        time_zone: &impl TimeZoneInput,
    ) -> Result<(), DateTimeFormatError> {
        sink.write_str(
            self.mz_generic_short
                .as_ref()
                .map(|p| p.get())
                .and_then(|metazones| time_zone.metazone_id().and_then(|mz| metazones.get(mz)))
                .ok_or(fmt::Error)?,
        )
        .map_err(DateTimeFormatError::from)
    }

    /// Writes the time zone in long generic non-location format as defined by the UTS-35 spec.
    /// e.g. Pacific Time
    /// https://unicode.org/reports/tr35/tr35-dates.html#Time_Zone_Format_Terminology
    pub(super) fn long_generic_non_location_format<W: fmt::Write + ?Sized>(
        &self,
        sink: &mut W,
        time_zone: &impl TimeZoneInput,
    ) -> Result<(), DateTimeFormatError> {
        sink.write_str(
            self.mz_generic_long
                .as_ref()
                .map(|p| p.get())
                .and_then(|metazones| time_zone.metazone_id().and_then(|mz| metazones.get(mz)))
                .ok_or(fmt::Error)?,
        )
        .map_err(DateTimeFormatError::from)
    }

    /// Writes the time zone in short specific non-location format as defined by the UTS-35 spec.
    /// e.g. PDT
    /// https://unicode.org/reports/tr35/tr35-dates.html#Time_Zone_Format_Terminology
    pub(super) fn short_specific_non_location_format<W: fmt::Write + ?Sized>(
        &self,
        sink: &mut W,
        time_zone: &impl TimeZoneInput,
    ) -> Result<(), DateTimeFormatError> {
        sink.write_str(
            self.mz_specific_short
                .as_ref()
                .map(|p| p.get())
                .and_then(|metazones| time_zone.metazone_id().and_then(|mz| metazones.get(mz)))
                .and_then(|specific_names| {
                    time_zone
                        .time_variant()
                        .and_then(|variant| specific_names.get(variant))
                })
                .ok_or_else(|| DateTimeFormatError::from(fmt::Error))?,
        )
        .map_err(DateTimeFormatError::from)
    }

    /// Writes the time zone in long specific non-location format as defined by the UTS-35 spec.
    /// e.g. Pacific Daylight Time
    /// https://unicode.org/reports/tr35/tr35-dates.html#Time_Zone_Format_Terminology
    pub(super) fn long_specific_non_location_format<W: fmt::Write + ?Sized>(
        &self,
        sink: &mut W,
        time_zone: &impl TimeZoneInput,
    ) -> Result<(), DateTimeFormatError> {
        sink.write_str(
            self.mz_specific_long
                .as_ref()
                .map(|p| p.get())
                .and_then(|metazones| time_zone.metazone_id().and_then(|mz| metazones.get(mz)))
                .and_then(|specific_names| {
                    time_zone
                        .time_variant()
                        .and_then(|variant| specific_names.get(variant))
                })
                .ok_or(fmt::Error)?,
        )
        .map_err(DateTimeFormatError::from)
    }

    /// Writes the time zone in localized GMT format according to the CLDR localized hour format.
    /// This goes explicitly against the UTS-35 spec, which specifies long or short localized
    /// GMT formats regardless of locale.
    ///
    /// You can see more information about our decision to resolve this conflict here:
    /// https://docs.google.com/document/d/16GAqaDRS6hzL8jNYjus5MglSevGBflISM-BrIS7bd4A/edit?usp=sharing
    pub(super) fn localized_gmt_format<W: fmt::Write + ?Sized>(
        &self,
        sink: &mut W,
        time_zone: &impl TimeZoneInput,
    ) -> Result<(), DateTimeFormatError> {
        let gmt_offset = time_zone.gmt_offset();
        if gmt_offset.is_zero() {
            sink.write_str(&self.zone_formats.get().gmt_zero_format.clone())
                .map_err(DateTimeFormatError::from)
        } else {
            // TODO(blocked on #277) Use formatter utility instead of replacing "{0}".
            sink.write_str(
                &self
                    .zone_formats
                    .get()
                    .gmt_format
                    .replace(
                        "{0}",
                        if gmt_offset.is_positive() {
                            &self.zone_formats.get().hour_format.0
                        } else {
                            &self.zone_formats.get().hour_format.1
                        },
                    )
                    // support all combos of "(HH|H):mm" by replacing longest patterns first.
                    .replace("HH", &self.format_offset_hours(time_zone, ZeroPadding::On))
                    .replace("mm", &self.format_offset_minutes(time_zone))
                    .replace("H", &self.format_offset_hours(time_zone, ZeroPadding::Off)),
            )
            .map_err(DateTimeFormatError::from)
        }
    }

    /// Writes the exemplar city associated with this time zone.
    pub(super) fn exemplar_city<W: fmt::Write + ?Sized>(
        &self,
        sink: &mut W,
        time_zone: &impl TimeZoneInput,
    ) -> Result<(), DateTimeFormatError> {
        sink.write_str(
            self.exemplar_cities
                .as_ref()
                .map(|p| p.get())
                .and_then(|cities| time_zone.time_zone_id().and_then(|id| cities.get(id)))
                .ok_or(fmt::Error)?,
        )
        .map_err(DateTimeFormatError::from)
    }

    /// Writes the unknown city "Etc/Unknown" for the current locale.
    ///
    /// If there is no localized form of "Etc/Unknown" for the current locale,
    /// returns the "Etc/Uknown" value of the `und` locale as a hard-coded string.
    ///
    /// This can be used as a fallback if [`exemplar_city()`](TimeZoneFormat::exemplar_city())
    /// is unable to produce a localized form of the time zone's exemplar city in the current locale.
    pub(super) fn unknown_city<W: fmt::Write + ?Sized>(
        &self,
        sink: &mut W,
    ) -> Result<(), DateTimeFormatError> {
        sink.write_str(
            self.exemplar_cities
                .as_ref()
                .map(|p| p.get())
                .and_then(|cities| cities.get("Etc/Unknown"))
                .unwrap_or(&Cow::Borrowed("Unknown")),
        )
        .map_err(DateTimeFormatError::from)
    }

    /// Formats a time segment with optional zero-padding.
    fn format_time_segment(n: u8, padding: ZeroPadding) -> String {
        debug_assert!((0..60).contains(&n));
        match padding {
            ZeroPadding::On => format!("{:>02}", n),
            ZeroPadding::Off => format!("{}", n),
        }
    }

    /// Formats the hours as a [`String`] with optional zero-padding.
    pub(super) fn format_offset_hours(
        &self,
        time_zone: &impl TimeZoneInput,
        padding: ZeroPadding,
    ) -> String {
        Self::format_time_segment(
            (time_zone.gmt_offset().raw_offset_seconds() / 3600).abs() as u8,
            padding,
        )
    }

    /// Formats the minutes as a [`String`] with zero-padding.
    pub(super) fn format_offset_minutes(&self, time_zone: &impl TimeZoneInput) -> String {
        Self::format_time_segment(
            (time_zone.gmt_offset().raw_offset_seconds() % 3600 / 60).abs() as u8,
            ZeroPadding::On,
        )
    }

    /// Formats the seconds as a [`String`] with zero-padding.
    pub(super) fn format_offset_seconds(&self, time_zone: &impl TimeZoneInput) -> String {
        Self::format_time_segment(
            (time_zone.gmt_offset().raw_offset_seconds() % 3600 % 60).abs() as u8,
            ZeroPadding::On,
        )
    }

    /// Writes a [`GmtOffset`](crate::date::GmtOffset) in ISO-8601 format according to the
    /// given formatting options.
    ///
    /// [`IsoFormat`] determines whether the format should be Basic or Extended,
    /// and whether a zero-offset should be formatted numerically or with
    /// The UTC indicator: "Z"
    /// - Basic    e.g. +0800
    /// - Extended e.g. +08:00
    ///
    /// [`IsoMinutes`] can be required or optional.
    /// [`IsoSeconds`] can be optional or never.
    pub(super) fn iso8601_format<W: fmt::Write + ?Sized>(
        &self,
        sink: &mut W,
        time_zone: &impl TimeZoneInput,
        format: IsoFormat,
        minutes: IsoMinutes,
        seconds: IsoSeconds,
    ) -> Result<(), DateTimeFormatError> {
        let gmt_offset = time_zone.gmt_offset();
        if gmt_offset.is_zero() && matches!(format, IsoFormat::UtcBasic | IsoFormat::UtcExtended) {
            return sink.write_char('Z').map_err(|err| err.into());
        }

        let extended_format = matches!(format, IsoFormat::Extended | IsoFormat::UtcExtended);
        sink.write_char(if gmt_offset.is_positive() { '+' } else { '-' })?;
        sink.write_str(&self.format_offset_hours(time_zone, ZeroPadding::On))?;

        match minutes {
            IsoMinutes::Required => {
                if extended_format {
                    sink.write_char(':')?;
                }
                sink.write_str(&self.format_offset_minutes(time_zone))?;
            }
            IsoMinutes::Optional => {
                if gmt_offset.has_minutes() {
                    if extended_format {
                        sink.write_char(':')?;
                    }
                    sink.write_str(&self.format_offset_minutes(time_zone))?;
                }
            }
        }

        if let IsoSeconds::Optional = seconds {
            if gmt_offset.has_seconds() {
                if extended_format {
                    sink.write_char(':')?;
                }
                sink.write_str(&self.format_offset_seconds(time_zone))?;
            }
        }

        Ok(())
    }
}

/// Determines which ISO-8601 format should be used to format a [`GmtOffset`](crate::date::GmtOffset).
pub(super) enum IsoFormat {
    /// ISO-8601 Basic Format.
    /// Formats zero-offset numerically.
    /// e.g. +0500, +0000
    Basic,

    /// ISO-8601 Extended Format.
    /// Formats zero-offset numerically.
    /// e.g. +05:00, +00:00
    Extended,

    /// ISO-8601 Basic Format.
    /// Formats zero-offset with the ISO-8601 UTC indicator: "Z"
    /// e.g. +0500, Z
    UtcBasic,

    /// ISO-8601 Extended Format.
    /// Formats zero-offset with the ISO-8601 UTC indicator: "Z"
    /// e.g. +05:00, Z
    UtcExtended,
}

/// Whether the minutes field should be optional or required in ISO-8601 format.
pub(super) enum IsoMinutes {
    /// Minutes are always displayed.
    Required,

    /// Minutes are displayed only if they are non-zero.
    Optional,
}

/// Whether the seconds field should be optional or excluded in ISO-8601 format.
pub(super) enum IsoSeconds {
    /// Seconds are displayed only if they are non-zero.
    Optional,

    /// Seconds are not displayed.
    Never,
}

/// Whether a field should be zero-padded in ISO-8601 format.
pub(super) enum ZeroPadding {
    /// Add zero-padding.
    On,

    /// Do not add zero-padding.
    Off,
}
