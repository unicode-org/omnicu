// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use litemap::LiteMap;
use std::borrow::Cow;

/// Provides a few common map accessor methods to new-type structs that wrap a map type.
/// The methods are all pass-through calls to the internal methods of the same name.
macro_rules! map_access {
    ($outer: ty => $inner: ty: $lt: lifetime) => {
        impl<$lt> $outer {
            /// Get the data from the key.
            pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&$inner>
            where
                Q: Ord,
                Cow<'s, str>: std::borrow::Borrow<Q>,
            {
                self.0.get(key)
            }

            /// Check if the underlying data is empty.
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        impl<$lt, Q: ?Sized> std::ops::Index<&Q> for $outer
        where
            Q: Ord,
            Cow<'s, str>: std::borrow::Borrow<Q>,
        {
            type Output = $inner;
            fn index(&self, key: &Q) -> &Self::Output {
                self.0.get(key).unwrap()
            }
        }
    };
}

/// An ICU4X mapping to the CLDR timeZoneNames format strings.
/// See CLDR-JSON timeZoneNames.json for more context.
#[derive(PartialEq, Debug, Clone, Default)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct TimeZoneFormatsV1<'s> {
    /// The hour format for displaying GMT offsets.
    pub hour_format: (Cow<'s, str>, Cow<'s, str>),
    /// The localized GMT-offset format.
    pub gmt_format: Cow<'s, str>,
    /// The localized GMT format with no offset.
    pub gmt_zero_format: Cow<'s, str>,
    /// The format string for a region.
    pub region_format: Cow<'s, str>,
    /// The format strings for region format variants
    /// e.g. daylight, standard.
    pub region_format_variants: LiteMap<Cow<'s, str>, Cow<'s, str>>,
    /// The format string to fall back to if data is unavailable.
    pub fallback_format: Cow<'s, str>,
}

/// An ICU4X mapping to the CLDR timeZoneNames exemplar cities.
/// See CLDR-JSON timeZoneNames.json for more context.
#[derive(PartialEq, Debug, Clone, Default)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ExemplarCitiesV1<'s>(pub LiteMap<Cow<'s, str>, Cow<'s, str>>);
map_access!(ExemplarCitiesV1<'s> => Cow<'s, str>: 's);

/// An ICU4X mapping to the long-form generic metazone names.
/// See CLDR-JSON timeZoneNames.json for more context.
#[derive(PartialEq, Debug, Clone, Default)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct MetaZoneGenericNamesLongV1<'s>(pub LiteMap<Cow<'s, str>, Cow<'s, str>>);
map_access!(MetaZoneGenericNamesLongV1<'s> => Cow<'s, str>: 's);

/// An ICU4X mapping to the short-form generic metazone names.
/// See CLDR-JSON timeZoneNames.json for more context.
#[derive(PartialEq, Debug, Clone, Default)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct MetaZoneGenericNamesShortV1<'s>(pub LiteMap<Cow<'s, str>, Cow<'s, str>>);
map_access!(MetaZoneGenericNamesShortV1<'s> => Cow<'s, str>: 's);

/// An ICU4X mapping to the long-form specific metazone names.
/// Specific names include time variants such as "daylight."
/// See CLDR-JSON timeZoneNames.json for more context.
#[derive(PartialEq, Debug, Clone, Default)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct MetaZoneSpecificNamesLongV1<'s>(pub LiteMap<Cow<'s, str>, MetaZoneSpecificNamesV1<'s>>);
map_access!(MetaZoneSpecificNamesLongV1<'s> => MetaZoneSpecificNamesV1<'s>: 's);

/// An ICU4X mapping to the short-form specific metazone names.
/// Specific names include time variants such as "daylight."
/// See CLDR-JSON timeZoneNames.json for more context.
#[derive(PartialEq, Debug, Clone, Default)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct MetaZoneSpecificNamesShortV1<'s>(pub LiteMap<Cow<'s, str>, MetaZoneSpecificNamesV1<'s>>);
map_access!(MetaZoneSpecificNamesShortV1<'s> => MetaZoneSpecificNamesV1<'s>: 's);

/// A general struct to hold metazone specific name variants.
/// Specific names include time variants such as "daylight."
/// See CLDR-JSON timeZoneNames.json for more context.
#[derive(PartialEq, Debug, Clone, Default)]
#[cfg_attr(
    feature = "provider_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct MetaZoneSpecificNamesV1<'s>(pub LiteMap<Cow<'s, str>, Cow<'s, str>>);
map_access!(MetaZoneSpecificNamesV1<'s> => Cow<'s, str>: 's);
