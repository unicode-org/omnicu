// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! This hour cycle module represents various utilities for working with hour cycles in order
//! to apply a user preference.

#![doc(hidden)]
#![cfg(feature = "provider_transform_internals")]

use crate::fields;
use crate::pattern::{CoarseHourCycle, Pattern, PatternItem};
use crate::provider;
use crate::skeleton;

/// Figure out the coarse hour cycle given a pattern, which is useful for generating the provider
/// patterns for `length::Bag`.
pub fn determine_coarse_hour_cycle(pattern: &Pattern) -> Option<CoarseHourCycle> {
    for item in pattern.items() {
        if let PatternItem::Field(fields::Field {
            symbol: fields::FieldSymbol::Hour(pattern_hour),
            length: _,
        }) = item
        {
            return Some(match pattern_hour {
                fields::Hour::H11 | fields::Hour::H12 => CoarseHourCycle::H11H12,
                fields::Hour::H23 | fields::Hour::H24 => CoarseHourCycle::H23H24,
            });
        }
    }

    None
}

/// Invoke the pattern matching machinery to transform the hour cycle of a pattern. This provides
/// a safe mapping from a h11/h12 to h23/h24 for transforms.
#[doc(hidden)]
#[cfg(feature = "provider_transform_internals")]
pub fn apply_coarse_hour_cycle(
    datetime: &provider::gregory::patterns::DateTimeFormatsV1,
    pattern_str: &str,
    mut pattern: Pattern,
    coarse_hour_cycle: CoarseHourCycle,
) -> Option<String> {
    for item in pattern.items_mut() {
        if let PatternItem::Field(fields::Field { symbol, length: _ }) = item {
            if let fields::FieldSymbol::Hour(pattern_hour) = symbol {
                if match coarse_hour_cycle {
                    CoarseHourCycle::H11H12 => match pattern_hour {
                        fields::Hour::H11 | fields::Hour::H12 => true,
                        fields::Hour::H23 | fields::Hour::H24 => false,
                    },
                    CoarseHourCycle::H23H24 => match pattern_hour {
                        fields::Hour::H11 | fields::Hour::H12 => false,
                        fields::Hour::H23 | fields::Hour::H24 => true,
                    },
                } {
                    // The preference hour cycle matches the pattern, bail out early and
                    // return the current pattern.
                    return Some(pattern_str.into());
                } else {
                    // Mutate the pattern with the new symbol, so that it can be matched against.
                    *symbol = fields::FieldSymbol::Hour(match coarse_hour_cycle {
                        CoarseHourCycle::H11H12 => fields::Hour::H12,
                        CoarseHourCycle::H23H24 => fields::Hour::H23,
                    });
                    break;
                }
            }
        }
    }

    let skeleton = skeleton::Skeleton::from(&pattern);

    match skeleton::create_best_pattern_for_fields(
        &datetime.skeletons,
        &datetime.length_patterns,
        &skeleton.as_slice(),
    ) {
        skeleton::BestSkeleton::AllFieldsMatch(pattern)
        | skeleton::BestSkeleton::MissingOrExtraFields(pattern) => Some(format!("{}", pattern)),
        skeleton::BestSkeleton::NoMatch => None,
    }
}
