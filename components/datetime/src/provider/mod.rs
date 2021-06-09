// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Data provider struct definitions for this ICU4X component.
//!
//! Read more about data providers: [`icu_provider`]

/// Data providers for the Gregorian Calendar.
pub mod gregory;

pub(crate) mod helpers;

/// Data providers for time zones.
pub mod time_zones;

/// A collection of [`ResourceKey`](icu_provider::prelude::ResourceKey) structs for DateTime providers.
pub mod key {
    use icu_provider::{resource_key, ResourceKey};

    /// A [`ResourceKey`](icu_provider::prelude::ResourceKey) to
    /// [`gregory::DatesV1`](crate::provider::gregory::DatesV1).
    pub const GREGORY_V1: ResourceKey = resource_key!(dates, "gregory", 1);

    /// A [`ResourceKey`](icu_provider::prelude::ResourceKey) to
    /// [`time_zones::TimeZoneFormatsV1`](crate::provider::time_zones::TimeZoneFormatsV1).
    pub const TIMEZONE_FORMATS_V1: ResourceKey = resource_key!(time_zones, "formats", 1);

    /// A [`ResourceKey`](icu_provider::prelude::ResourceKey) to
    /// [`time_zones::ExemplarCitiesV1`](crate::provider::time_zones::ExemplarCitiesV1).
    pub const TIMEZONE_EXEMPLAR_CITIES_V1: ResourceKey =
        resource_key!(time_zones, "exemplar-cities", 1);

    /// A [`ResourceKey`](icu_provider::prelude::ResourceKey) to
    /// [`time_zones::MetaZoneGenericNamesLongV1`](crate::provider::time_zones::MetaZoneGenericNamesLongV1).
    pub const TIMEZONE_GENERIC_NAMES_LONG_V1: ResourceKey =
        resource_key!(time_zones, "generic-long", 1);

    /// A [`ResourceKey`](icu_provider::prelude::ResourceKey) to
    /// [`time_zones::MetaZoneGenericNamesShortV1`](crate::provider::time_zones::MetaZoneGenericNamesShortV1).
    pub const TIMEZONE_GENERIC_NAMES_SHORT_V1: ResourceKey =
        resource_key!(time_zones, "generic-short", 1);

    /// A [`ResourceKey`](icu_provider::prelude::ResourceKey) to
    /// [`time_zones::MetaZoneSpecificNamesLongV1`](crate::provider::time_zones::MetaZoneSpecificNamesLongV1).
    pub const TIMEZONE_SPECIFIC_NAMES_LONG_V1: ResourceKey =
        resource_key!(time_zones, "specific-long", 1);

    /// A [`ResourceKey`](icu_provider::prelude::ResourceKey) to
    /// [`time_zones::MetaZoneSpecificNamesShortV1`](crate::provider::time_zones::MetaZoneSpecificNamesShortV1).
    pub const TIMEZONE_SPECIFIC_NAMES_SHORT_V1: ResourceKey =
        resource_key!(time_zones, "specific-short", 1);
}
