// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Skeletons are used for pattern matching. See the [`Skeleton`] struct for more information.

use smallvec::SmallVec;
use std::convert::TryFrom;
use thiserror::Error;

use crate::{
    fields::{self, Field, FieldLength, FieldSymbol},
    options::length,
    pattern::{Pattern, PatternItem},
    provider::gregory::patterns::{LengthPatternsV1, PatternV1, SkeletonV1, SkeletonsV1},
};

#[cfg(feature = "provider_serde")]
use serde::{
    de,
    ser::{self, SerializeSeq},
    Deserialize, Deserializer, Serialize,
};

#[derive(Debug, PartialEq)]
struct FieldIndex(usize);

/// A [`Skeleton`] is used to represent what types of `Field`s are present in a [`Pattern`]. The
/// ordering of the [`Skeleton`]'s `Field`s have no bearing on the ordering of the `Field`s and
/// `Literal`s in the [`Pattern`].
///
/// A [`Skeleton`] is a [`Vec`]`<Field>`, but with the invariant that it is sorted according to the canonical
/// sort order. This order is sorted according to the most significant `Field` to the least significant.
/// For example, a field with a `Minute` symbol would preceed a field with a `Second` symbol.
/// This order is documented as the order of fields as presented in the
/// [UTS 35 Date Field Symbol Table](https://unicode.org/reports/tr35/tr35-dates.html#Date_Field_Symbol_Table)
///
/// The `Field`s are only sorted in the [`Skeleton`] in order to provide a deterministic
/// serialization strategy, and to provide a faster [`Skeleton`] matching operation.
#[derive(Debug, Eq, PartialEq, Clone, Ord, PartialOrd)]
pub struct Skeleton(SmallVec<[fields::Field; 5]>);

impl Skeleton {
    fn fields_iter<'a>(&'a self) -> impl Iterator<Item = &Field> + 'a {
        self.0.iter()
    }

    fn fields_len(&self) -> usize {
        self.0.len()
    }
}

/// This is an implementation of the serde deserialization visitor pattern.
#[cfg(feature = "provider_serde")]
#[allow(clippy::upper_case_acronyms)]
struct DeserializeSkeletonFieldsUTS35String;

#[cfg(feature = "provider_serde")]
impl<'de> de::Visitor<'de> for DeserializeSkeletonFieldsUTS35String {
    type Value = Skeleton;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Expected to find a valid skeleton.")
    }

    /// A [`Skeleton`] serialized into a string follows UTS-35.
    /// https://unicode.org/reports/tr35/tr35-dates.html#Date_Field_Symbol_Table
    /// This string consists of a symbol that is repeated N times. This string is
    /// deserialized here into the Skeleton format which is used in memory
    /// when working with formatting datetimes.
    fn visit_str<E>(self, skeleton_string: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Skeleton::try_from(skeleton_string).map_err(|err| {
            de::Error::invalid_value(
                de::Unexpected::Other(&format!("{:?} {}", skeleton_string, err)),
                &"field symbols representing a skeleton",
            )
        })
    }
}

#[cfg(feature = "provider_serde")]
struct DeserializeSkeletonBincode;

#[cfg(feature = "provider_serde")]
impl<'de> de::Visitor<'de> for DeserializeSkeletonBincode {
    type Value = Skeleton;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Unable to deserialize a bincode Pattern.")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Skeleton, V::Error>
    where
        V: de::SeqAccess<'de>,
    {
        let mut items: SmallVec<[fields::Field; 5]> = SmallVec::new();
        while let Some(item) = seq.next_element()? {
            if let Some(prev_item) = items.last() {
                if prev_item >= &item {
                    return Err(de::Error::invalid_value(
                        de::Unexpected::Other(&format!(
                            "field item out of order or duplicate: {:?}",
                            item
                        )),
                        &"ordered field symbols representing a skeleton",
                    ));
                }
            }
            items.push(item)
        }
        Ok(Skeleton(items))
    }
}

#[cfg(feature = "provider_serde")]
impl<'de> Deserialize<'de> for Skeleton {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_str(DeserializeSkeletonFieldsUTS35String)
        } else {
            deserializer.deserialize_seq(DeserializeSkeletonBincode)
        }
    }
}

#[cfg(feature = "provider_serde")]
impl Serialize for Skeleton {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            // Serialize into the UTS 35 string representation.
            let mut string = String::new();

            for field in self.0.iter() {
                let ch: char = field.symbol.into();
                for _ in 0..field.length as usize {
                    string.push(ch);
                }
            }

            serializer.serialize_str(&string)
        } else {
            // Serialize into a bincode-friendly representation. This means that pattern parsing
            // will not be needed when deserializing.
            let mut seq = serializer.serialize_seq(Some(self.fields_len()))?;
            for item in self.fields_iter() {
                seq.serialize_element(item)?;
            }
            seq.end()
        }
    }
}

/// Parse a string into a list of fields. This trait implementation validates the input string to
/// verify that fields are correct. If the fields are out of order, this returns an error that
/// contains the fields, which gives the callee a chance to sort the fields with the
/// `From<SmallVec<[fields::Field; 5]>> for Skeleton` trait.
impl TryFrom<&str> for Skeleton {
    type Error = SkeletonError;
    fn try_from(skeleton_string: &str) -> Result<Self, Self::Error> {
        let mut fields: SmallVec<[fields::Field; 5]> = SmallVec::new();

        let mut iter = skeleton_string.bytes().peekable();
        while let Some(byte) = iter.next() {
            // Convert the byte to a valid field symbol.
            let field_symbol = FieldSymbol::try_from(byte)?;

            // Go through the bytes to count how often it's repeated.
            let mut field_length: u8 = 1;
            while let Some(next_byte) = iter.peek() {
                if *next_byte != byte {
                    break;
                }
                field_length += 1;
                iter.next();
            }

            let field = Field::from((field_symbol, FieldLength::try_from(field_length)?));

            match fields.binary_search(&field) {
                Ok(_) => return Err(SkeletonError::DuplicateField),
                Err(pos) => fields.insert(pos, field),
            }
        }

        Ok(Self(fields))
    }
}

/// Represents a specific pattern that is available for a given locale.
/// A [`Skeleton`] is used to match against to find the best pattern.
#[derive(Debug, PartialEq, Clone)]
pub struct AvailableFormatPattern<'a> {
    /// The skeleton that is used to match against.
    skeleton: &'a Skeleton,
    pub pattern: &'a Pattern,
}

impl<'a> From<(&'a SkeletonV1, &'a PatternV1)> for AvailableFormatPattern<'a> {
    fn from(tuple: (&'a SkeletonV1, &'a PatternV1)) -> Self {
        let (skeleton_v1, pattern_v1) = tuple;

        AvailableFormatPattern {
            skeleton: &skeleton_v1.0,
            pattern: &pattern_v1.0,
        }
    }
}

/// These strings follow the recommendations for the serde::de::Unexpected::Other type.
/// https://docs.serde.rs/serde/de/enum.Unexpected.html#variant.Other
///
/// Serde will generate an error such as:
/// "invalid value: unclosed literal in pattern, expected a valid UTS 35 pattern string at line 1 column 12"
#[derive(Error, Debug)]
pub enum SkeletonError {
    #[error("field too long in skeleton")]
    InvalidFieldLength,
    #[error("duplicate field in skeleton")]
    DuplicateField,
    #[error("symbol unknown {0} in skeleton")]
    SymbolUnknown(char),
    #[error("symbol invalid {0} in skeleton")]
    SymbolInvalid(char),
    #[error("symbol unimplemented {0} in skeleton")]
    SymbolUnimplemented(char),
    #[error("unimplemented field {0} in skeleton")]
    UnimplementedField(char),
    #[error(transparent)]
    Fields(#[from] fields::Error),
}

impl From<fields::LengthError> for SkeletonError {
    fn from(_: fields::LengthError) -> Self {
        Self::InvalidFieldLength
    }
}

impl From<fields::SymbolError> for SkeletonError {
    fn from(symbol_error: fields::SymbolError) -> Self {
        match symbol_error {
            fields::SymbolError::Invalid(ch) => Self::SymbolInvalid(ch),
            fields::SymbolError::Unknown(byte) => {
                // NOTE: If you remove a symbol due to it now being supported,
                //       make sure to regenerate the test data.
                //       https://github.com/unicode-org/icu4x/blob/main/provider/testdata/README.md
                match byte {
                    // TODO(#487) - Flexible day periods
                    b'B'
                    // TODO(#486) - Era
                    | b'G'
                    // TODO(#502) - Week of month
                    | b'W'
                    // TODO(#501) - Quarters
                    | b'Q'
                    // TODO (#488) - Week of year
                    | b'w'
                    => Self::SymbolUnimplemented(byte.into()),
                    _ => Self::SymbolUnknown(byte.into()),
                }
            }
        }
    }
}

// The following scalar values are for testing the suitability of a skeleton's field for the
// given input. Per UTS 35, the better the fit of a pattern, the "lower the distance". In this
// implementation each distance type is separated by an order of magnitiude. This magnitude needs
// to be at minimum a multiple of the max length of fields. As of CLDR 38 (2021-01), the max length
// of a skeleton in the "availableFormats" contained a total of 4 fields. The scores use a multiple
// of 10, as a number that will contain the range, and be easy to reason with.
//
// The only exception is on the largest magnitude of values (MISSING_OR_SKELETON_EXTRA_SYMBOL). The
// missing or extra count BOTH the requested fields and skeleton fields. This is fine since there
// is no higher magnitude.

const MAX_SKELETON_FIELDS: u32 = 10;

// Per the skeleton matching algorithm:
// https://unicode.org/reports/tr35/tr35-dates.html#Matching_Skeletons

// > 1. "Input skeleton symbols" are replaced with the best match for a given locale.
// >   - Hour: j → {H, k, h, K} + {a, b, B}
// >           J → {H, k, h, K}
// >           C → j + day period

// The components::Bag does not support step 1

// > 2. For fields with symbols representing the same type (year, month, day, etc):
// >   A. Most symbols have a small distance from each other.
// >     - Months: M ≅ L           (9 ≅ 9)  conjuction, vs stand-alone
// >       Week:   E ≅ c           (Tue ≅ 2)
// >       Period: a ≅ b ≅ B       (am. ≅ mid. ≅ at night)
// >       Hour:   H ≅ k ≅ h ≅ K   (23, 24, 12, 11)

// For step 2, the components::Bag will not produce "stand-alone" months, as no skeletons
// contain stand-alone months.

const NO_DISTANCE: u32 = 0;

// B. Width differences among fields, other than those marking text vs numeric, are given small
// distance from each other.
// - MMM ≅ MMMM  (Sep ≅ September)
//   MM ≅ M      (09 ≅ 9)
const WIDTH_MISMATCH_DISTANCE: u32 = 1;

// C. Numeric and text fields are given a larger distance from each other.
// - MMM ≈ MM    (Sep ≈ 09)
//   MMM
const TEXT_VS_NUMERIC_DISTANCE: u32 = 10;

// D. Symbols representing substantial differences (week of year vs week of month) are given much
// larger a distances from each other.
// - d ≋ D;     (12 ≋ 345) Day of month vs Day of year
const SUBSTANTIAL_DIFFERENCES_DISTANCE: u32 = 100;

// A skeleton had more symbols than what was requested.
const SKELETON_EXTRA_SYMBOL: u32 = 1000;

// A requested symbol is missing in the skeleton. Note that this final value can be more than
// MAX_SKELETON_FIELDS, as it's counting the missing requested fields, which can be longer than
// the stored skeletons. There cannot be any cases higher than this one.
const REQUESTED_SYMBOL_MISSING: u32 = 10000;

/// According to the [UTS 35 skeleton matching algorithm](https://unicode.org/reports/tr35/tr35-dates.html#Matching_Skeletons)
/// there will be a guaranteed match for a skeleton. However, with this initial implementation,
/// there is no attempt to add on missing fields. This enum encodes the variants for the current
/// search for a best skeleton.
#[derive(Debug, PartialEq, Clone)]
pub enum BestSkeleton<T> {
    AllFieldsMatch(T),
    MissingOrExtraFields(T),
    NoMatch,
}

// TODO - This could return a Cow<'a, Pattern>, but it affects every other part of the API to
// add a lifetime here. The pattern returned here could be one that we've already constructed in
// the CLDR as an exotic type, or it could be one that was modified to meet the requirements of
// the components bag.
pub fn create_best_pattern_for_fields<'a>(
    skeletons: &'a SkeletonsV1,
    length_patterns: &LengthPatternsV1,
    fields: &[Field],
) -> BestSkeleton<Pattern> {
    let first_pattern_match = get_best_available_format_pattern(skeletons, fields);

    // Try to match a skeleton to all of the fields.
    if let BestSkeleton::AllFieldsMatch(pattern) = first_pattern_match {
        return BestSkeleton::AllFieldsMatch(pattern);
    }

    let FieldsByType { date, time, other } = group_fields_by_type(fields);

    if !other.is_empty() {
        // These require "append items" support, see #586.
        // TODO(#583) - TimeZones
        // TODO(#486) - Eras,
        // ... etc.
        unimplemented!(
            "There are no \"other\" fields supported, these need to be appended to the pattern. {:?}", other
        );
    }

    if date.is_empty() || time.is_empty() {
        return match first_pattern_match {
            BestSkeleton::AllFieldsMatch(_) => {
                unreachable!("Logic error in implementation. AllFieldsMatch handled above.")
            }
            BestSkeleton::MissingOrExtraFields(pattern) => {
                BestSkeleton::MissingOrExtraFields(pattern)
            }
            BestSkeleton::NoMatch => BestSkeleton::NoMatch,
        };
    }

    // Match the date and time, and then simplify the combinatorial logic of the results into
    // an optional values of the results, and a boolean value.
    let (date_pattern, date_missing_or_extra) =
        match get_best_available_format_pattern(skeletons, &date) {
            BestSkeleton::MissingOrExtraFields(fields) => (Some(fields), true),
            BestSkeleton::AllFieldsMatch(fields) => (Some(fields), false),
            BestSkeleton::NoMatch => (None, true),
        };

    let (time_pattern, time_missing_or_extra) =
        match get_best_available_format_pattern(skeletons, &time) {
            BestSkeleton::MissingOrExtraFields(fields) => (Some(fields), true),
            BestSkeleton::AllFieldsMatch(fields) => (Some(fields), false),
            BestSkeleton::NoMatch => (None, true),
        };

    // Determine how to combine the date and time.
    let pattern: Option<Pattern> = match (date_pattern, time_pattern) {
        (Some(date_pattern), Some(time_pattern)) => {
            let month_field = fields
                .iter()
                .find(|f| matches!(f.symbol, FieldSymbol::Month(_)));

            // Per UTS-35, choose a "length" pattern for combining the date and time.
            // https://unicode.org/reports/tr35/tr35-dates.html#Matching_Skeletons
            //
            // 1. If the requested date fields include Wide month and weekday name of any length, use length::Date::Full
            // 2. Otherwise, if the requested date fields include wide month, use length::Date::Long
            // 3. Otherwise, if the requested date fields include abbreviated month, use length::Date::Medium
            // 4. Otherwise use length::Date::Short
            let length = match month_field {
                Some(field) => match field.length {
                    FieldLength::Wide => {
                        let weekday = fields
                            .iter()
                            .find(|f| matches!(f.symbol, FieldSymbol::Weekday(_)));

                        if weekday.is_some() {
                            length::Date::Full
                        } else {
                            length::Date::Long
                        }
                    }
                    FieldLength::Abbreviated => length::Date::Medium,
                    _ => length::Date::Short,
                },
                None => length::Date::Short,
            };

            let bytes = match length {
                length::Date::Full => &length_patterns.full,
                length::Date::Long => &length_patterns.long,
                length::Date::Medium => &length_patterns.medium,
                length::Date::Short => &length_patterns.short,
            };

            Some(Pattern::from_bytes_combination(bytes, date_pattern, time_pattern).expect("TODO"))
        }
        (None, Some(pattern)) => Some(pattern),
        (Some(pattern), None) => Some(pattern),
        (None, None) => None,
    };

    match pattern {
        Some(pattern) => {
            if date_missing_or_extra || time_missing_or_extra {
                BestSkeleton::MissingOrExtraFields(pattern)
            } else {
                BestSkeleton::AllFieldsMatch(pattern)
            }
        }
        None => BestSkeleton::NoMatch,
    }
}

struct FieldsByType {
    pub date: Vec<Field>,
    pub time: Vec<Field>,
    pub other: Vec<Field>,
}

fn group_fields_by_type(fields: &[Field]) -> FieldsByType {
    let mut date = Vec::new();
    let mut time = Vec::new();
    let mut other = Vec::new();

    for field in fields {
        match field.symbol {
            // Date components:
            // Note: Weekdays are included in both time and date skeletons.
            //  - Time examples: "EBhm" "EBhms" "Ed" "Ehm" "EHm" "Ehms" "EHms"
            //  - Date examples: "GyMMMEd" "MEd" "MMMEd" "MMMMEd" "yMEd" "yMMMEd"
            //  - Solo example: "E"
            FieldSymbol::Year(_)
            | FieldSymbol::Month(_)
            | FieldSymbol::Day(_)
            | FieldSymbol::Weekday(_) => date.push(*field),

            // Time components:
            FieldSymbol::DayPeriod(_)
            | FieldSymbol::Hour(_)
            | FieldSymbol::Minute
            | FieldSymbol::Second(_) => time.push(*field),

            // Other components
            FieldSymbol::TimeZone(_) => other.push(*field),
            // TODO(#486)
            // FieldSymbol::Era(_) => other.push(*field),
            // Plus others...
        };
    }

    FieldsByType { date, time, other }
}

/// A partial implementation of the [UTS 35 skeleton matching algorithm](https://unicode.org/reports/tr35/tr35-dates.html#Matching_Skeletons).
///
/// The following is implemented:
///
///  * Compute a score based on the best possible match for the given fields.
///  * Select the skeleton with highest score.
///  * Modify the resulting pattern to have fields of the same length. For example requesting
///      a skeleton "yMMMMd" can have a best match of ["yMMMd", "d MMM y"]. This pattern should
///      then be modified to use the requested length to produce a pattern "d MMMM y".
///      However, fields should not be changed from numeric to text.
///
/// The following is not implemented:
///
///  * 2.6.2.2 Missing Skeleton Fields
///    - TODO(#586) - Using the CLDR appendItems field. Note: There is not agreement yet on how
///      much of this step to implement. See the issue for more information.
pub fn get_best_available_format_pattern(
    skeletons: &SkeletonsV1,
    fields: &[Field],
) -> BestSkeleton<Pattern> {
    let mut closest_format_pattern = None;
    let mut closest_distance: u32 = u32::MAX;
    let mut closest_missing_fields = 0;

    for available_format_pattern in get_available_format_patterns(skeletons) {
        let skeleton = &available_format_pattern.skeleton;
        debug_assert!(
            skeleton.fields_len() <= MAX_SKELETON_FIELDS as usize,
            "The distance mechanism assumes skeletons are less than MAX_SKELETON_FIELDS in length."
        );
        let mut missing_fields = 0;
        let mut distance: u32 = 0;
        // The distance should fit into a u32.

        let mut requested_fields = fields.iter().peekable();
        let mut skeleton_fields = skeleton.fields_iter().peekable();
        loop {
            let next = (requested_fields.peek(), skeleton_fields.peek());

            // Try to find matching symbols.
            match next {
                (Some(requested_field), Some(skeleton_field)) => {
                    debug_assert!(
                        // As of the time of this writing, stand-alone months are not in the CLDR
                        // skeleton data. The components::Bag could produce stand-alone month fields,
                        // but since the CLDR does not have them, only Month::Format symbols are
                        // used for matching.
                        skeleton_field.symbol != FieldSymbol::Month(fields::Month::StandAlone)
                    );

                    if skeleton_field.symbol > requested_field.symbol {
                        // Keep searching for a matching skeleton field.
                        skeleton_fields.next();
                        distance += SKELETON_EXTRA_SYMBOL;
                        continue;
                    }

                    if skeleton_field.symbol < requested_field.symbol {
                        // The requested field symbol is missing from the skeleton.
                        distance += REQUESTED_SYMBOL_MISSING;
                        missing_fields += 1;
                        requested_fields.next();
                        continue;
                    }

                    distance += if requested_field == skeleton_field {
                        NO_DISTANCE
                    } else if requested_field.symbol != skeleton_field.symbol {
                        SUBSTANTIAL_DIFFERENCES_DISTANCE
                    } else if requested_field.get_length_type() != skeleton_field.get_length_type()
                    {
                        TEXT_VS_NUMERIC_DISTANCE
                    } else {
                        WIDTH_MISMATCH_DISTANCE
                    };

                    requested_fields.next();
                    skeleton_fields.next();
                }
                (None, Some(_)) => {
                    // The skeleton has additional fields that we are not matching.
                    distance += SKELETON_EXTRA_SYMBOL;
                    skeleton_fields.next();
                }
                (Some(_), None) => {
                    // The skeleton is missing requested fields.
                    distance += REQUESTED_SYMBOL_MISSING;
                    requested_fields.next();
                    missing_fields += 1;
                }
                (None, None) => {
                    break;
                }
            }
        }

        if distance < closest_distance {
            closest_format_pattern = Some(available_format_pattern.pattern);
            closest_distance = distance;
            closest_missing_fields = missing_fields;
        }
    }

    let closest_format_pattern =
        closest_format_pattern.expect("At least one closest format pattern will always be found.");

    if closest_missing_fields == fields.len() {
        return BestSkeleton::NoMatch;
    }

    if closest_distance == NO_DISTANCE {
        return BestSkeleton::AllFieldsMatch(closest_format_pattern.clone());
    }

    // Modify the resulting pattern to have fields of the same length.
    let expanded_pattern = Pattern::from(
        closest_format_pattern
            .items()
            .iter()
            .map(|item| {
                if let PatternItem::Field(pattern_field) = item {
                    if let Some(requested_field) = fields
                        .iter()
                        .find(|field| field.symbol == pattern_field.symbol)
                    {
                        if requested_field.length != pattern_field.length
                            && requested_field.get_length_type() == pattern_field.get_length_type()
                        {
                            return PatternItem::Field(*requested_field);
                        }
                    }
                }
                // There's no match, or this is a string literal return the original item.
                item.clone()
            })
            .collect::<Vec<PatternItem>>(),
    );

    if closest_distance >= SKELETON_EXTRA_SYMBOL {
        return BestSkeleton::MissingOrExtraFields(expanded_pattern);
    }

    BestSkeleton::AllFieldsMatch(expanded_pattern)
}

pub fn get_available_format_patterns<'a>(
    skeletons: &'a SkeletonsV1,
) -> impl Iterator<Item = AvailableFormatPattern> + 'a {
    skeletons.0.iter().map(AvailableFormatPattern::from)
}

#[cfg(all(test, feature = "provider_serde"))]
mod test {
    use super::*;

    use icu_locid_macros::langid;
    use icu_provider::prelude::*;

    use crate::{
        fields::{Day, Field, FieldLength, Month, Weekday},
        options::components,
        provider::{gregory::DatePatternsV1Marker, key::GREGORY_DATE_PATTERNS_V1},
    };

    fn get_data_payload() -> DataPayload<'static, 'static, DatePatternsV1Marker> {
        let provider = icu_testdata::get_provider();
        let langid = langid!("en");
        provider
            .load_payload(&DataRequest {
                resource_path: ResourcePath {
                    key: GREGORY_DATE_PATTERNS_V1,
                    options: ResourceOptions {
                        variant: None,
                        langid: Some(langid),
                    },
                },
            })
            .unwrap()
            .take_payload()
            .unwrap()
    }

    /// This is an initial smoke test to verify the skeleton machinery is working. For more in-depth
    /// testing see components/datetime/tests/fixtures/tests/components-*.json
    #[test]
    fn test_skeleton_matching() {
        let components = components::Bag {
            year: Some(components::Numeric::Numeric),
            month: Some(components::Month::Long),
            day: Some(components::Numeric::Numeric),

            hour: Some(components::Numeric::Numeric),
            minute: Some(components::Numeric::Numeric),
            second: Some(components::Numeric::Numeric),

            ..Default::default()
        };
        let requested_fields = components.to_vec_fields();
        let data_provider = get_data_payload();

        match get_best_available_format_pattern(
            &data_provider.get().datetime.skeletons,
            &requested_fields,
        ) {
            BestSkeleton::AllFieldsMatch(available_format_pattern)
            | BestSkeleton::MissingOrExtraFields(available_format_pattern) => {
                assert_eq!(
                    available_format_pattern.to_string(),
                    String::from("MMMM d, y")
                )
            }
            BestSkeleton::NoMatch => {
                panic!("No skeleton was found.")
            }
        };
    }

    #[test]
    fn test_skeleton_matching_missing_fields() {
        let components = components::Bag {
            month: Some(components::Month::Numeric),
            weekday: Some(components::Text::Short),
            ..Default::default()
        };
        let requested_fields = components.to_vec_fields();
        let data_provider = get_data_payload();

        match get_best_available_format_pattern(
            &data_provider.get().datetime.skeletons,
            &requested_fields,
        ) {
            BestSkeleton::MissingOrExtraFields(available_format_pattern) => {
                assert_eq!(available_format_pattern.to_string(), String::from("L"))
            }
            best => panic!("Unexpected {:?}", best),
        };
    }

    // TODO(#586) - Append items support needs to be added.
    #[test]
    fn test_missing_append_items_support() {
        let components = components::Bag {
            year: Some(components::Numeric::Numeric),
            month: Some(components::Month::Long),
            day: Some(components::Numeric::Numeric),
            // This will be appended.
            time_zone_name: Some(components::TimeZoneName::Long),
            ..Default::default()
        };
        let requested_fields = components.to_vec_fields();
        let data_provider = get_data_payload();

        match create_best_pattern_for_fields(
            &data_provider.get().datetime.skeletons,
            &data_provider.get().datetime.length_patterns,
            &requested_fields,
        ) {
            BestSkeleton::AllFieldsMatch(available_format_pattern) => {
                // TODO - This needs to support the "Z" pattern. This test will begin to fail
                // once support is added.
                assert_eq!(
                    available_format_pattern.to_string(),
                    String::from("MMMM d, y")
                )
            }
            best => panic!("Unexpected {:?}", best),
        };
    }

    #[test]
    fn test_skeleton_empty_bag() {
        let components: components::Bag = Default::default();
        let requested_fields = components.to_vec_fields();
        let data_provider = get_data_payload();

        assert_eq!(
            get_best_available_format_pattern(
                &data_provider.get().datetime.skeletons,
                &requested_fields
            ),
            BestSkeleton::NoMatch,
            "No match was found"
        );
    }

    /// There are no skeletons that match just the time zone. They all rely on the appendItems
    /// data from the CLDR.
    #[test]
    fn test_skeleton_no_match() {
        let components = components::Bag {
            time_zone_name: Some(components::TimeZoneName::Long),
            ..Default::default()
        };
        let requested_fields = components.to_vec_fields();
        let data_provider = get_data_payload();

        assert_eq!(
            get_best_available_format_pattern(
                &data_provider.get().datetime.skeletons,
                &requested_fields
            ),
            BestSkeleton::NoMatch,
            "No match was found"
        );
    }

    // These were all of the skeletons from the "available formats" in the CLDR as of 2021-01
    // Generated with:
    // https://gist.github.com/gregtatum/1d76bbdb87132f71a969a10f0c1d2d9c

    #[rustfmt::skip]
    const SUPPORTED_STRING_SKELETONS: [&str; 60] = [
        "E", "dEEEE", "EHm", "EHms", "dE", "Ehm", "Ehms", "H", "HHmm", "HHmmss", "Hm", "Hms", "M",
        "MdEEEE", "MdE", "MMM", "MMMdEEEE", "MMMdE", "MMMM", "MMMMdEEEE", "MMMMdE", "MMMMd",
        "MMMMdd", "MMMd", "MMMdd", "MMd", "MMdd", "Md", "Mdd", "d", "h", "hm", "hms", "mmss", "ms",
        "y", "yM", "yMdEEEE", "yMdE", "yMM", "yMMM", "yMMMdEEEE", "yMMMdE", "yMMMM", "yMMMMdEEEE",
        "yMMMMdE", "yMMMMdcccc", "yMMMMd", "yMMMd", "yMMdd", "yMd",
        // Timezones
        "HHmmZ", "Hmsv", "Hmsvvvv", "Hmv", "Hmvvvv", "hmsv", "hmsvvvv", "hmv", "hmvvvv",
    ];

    // NOTE: If you are moving this to the SUPPORTED section, make sure to remove the match
    //       on your symbol from impl From<fields::SymbolError> for SkeletonError
    //       and then regenerate the test data.
    //       https://github.com/unicode-org/icu4x/blob/main/provider/testdata/README.md
    #[rustfmt::skip]
    const UNSUPPORTED_STRING_SKELETONS: [&str; 19] = [
        // TODO(#487) - Flexible day periods
        "Bh", "Bhm", "Bhms", "EBhm", "EBhms",
        // TODO(#486) - Era
        "Gy", "GyM", "GyMMM", "GyMMMdEEEE", "GyMMMdE", "GyMMMM", "GyMMMMdE", "GyMMMMd", "GyMMMd",
        // TODO(#502) - Week of month
        "MMMMW",
        // TODO(#501) - Quarters
        "yQ", "yQQQ", "yQQQQ",
        // TODO (#488) - Week of year
        "yw"
    ];

    #[test]
    fn test_known_skeletons_ok() {
        for string_skeleton in &SUPPORTED_STRING_SKELETONS {
            match Skeleton::try_from(*string_skeleton) {
                Ok(_) => {}
                Err(err) => {
                    panic!(
                        "Unable to parse string_skeleton {:?} with error, {:?}",
                        string_skeleton, err
                    )
                }
            }
        }
    }

    #[test]
    fn test_unsupported_skeletons_skeletons_err() {
        for string_skeleton in &UNSUPPORTED_STRING_SKELETONS {
            match Skeleton::try_from(*string_skeleton) {
                Ok(_) => {
                    panic!(
                        "An unsupported field is now supported, consider moving {:?} to the \
                         supported skeletons, and ensure the skeleton is properly implemented.",
                        string_skeleton
                    )
                }
                Err(err) => match err {
                    SkeletonError::SymbolUnimplemented(_) => {
                        // Every skeleton should return this error.
                    }
                    _ => panic!("{}", err),
                },
            }
        }
    }

    #[test]
    fn test_skeleton_deserialization() {
        assert_eq!(
            Skeleton::try_from("MMMMdEEEE").unwrap(),
            Skeleton(
                vec![
                    Field {
                        symbol: Month::Format.into(),
                        length: FieldLength::Wide
                    },
                    Field {
                        symbol: Day::DayOfMonth.into(),
                        length: FieldLength::One
                    },
                    Field {
                        symbol: Weekday::Format.into(),
                        length: FieldLength::Wide
                    },
                ]
                .into()
            )
        );
    }

    #[test]
    fn test_skeleton_tuple_ordering() {
        let skeletons_strings = Vec::from([
            "y", "yM", "yMdE", "yMdEEEE", "yMMM", "M", "Md", "Mdd", "MMd", "MMdd", "d", "h", "hm",
            "hms", "Hm", "Hms", "ms", "mmss",
        ]);

        let skeleton_fields: Vec<Skeleton> = skeletons_strings
            .iter()
            .map(|skeleton_string| Skeleton::try_from(*skeleton_string).unwrap())
            .collect();

        for (strings, fields) in skeletons_strings.windows(2).zip(skeleton_fields.windows(2)) {
            if fields[0].cmp(&fields[1]) != std::cmp::Ordering::Less {
                panic!("Expected {:?} < {:?}", strings[0], strings[1]);
            }
        }
    }

    #[test]
    fn test_skeleton_json_reordering() {
        let unordered_skeleton = "EEEEyMd";
        let ordered_skeleton = "yMdEEEE";

        // Wrap the string in quotes so it's a JSON string.
        let json: String = serde_json::to_string(unordered_skeleton).unwrap();

        // Wrap the string in quotes so it's a JSON string.
        let skeleton = serde_json::from_str::<Skeleton>(&json)
            .expect("Unable to parse an unordered skeletons.");

        assert_eq!(
            serde_json::to_string(&skeleton).unwrap(),
            serde_json::to_string(ordered_skeleton).unwrap()
        );
    }

    /// This test handles a branch in the skeleton serialization code that takes into account
    /// duplicate field errors when deserializing from string.
    #[test]
    fn test_skeleton_json_duplicate_fields() {
        // Wrap the string in quotes so it's a JSON string.
        let json: String = serde_json::to_string("EEEEyMdEEEE").unwrap();
        let err =
            serde_json::from_str::<Skeleton>(&json).expect_err("Expected a duplicate field error.");

        assert_eq!(
            format!("{}", err),
            "invalid value: \"EEEEyMdEEEE\" duplicate field in skeleton, expected field symbols representing a skeleton at line 1 column 13"
        );
    }

    /// Skeletons are represented in bincode as a vec of field, but bincode shouldn't be completely
    /// trusted, test that the bincode gets validated correctly.
    struct TestInvalidSkeleton(Vec<Field>);

    #[cfg(feature = "provider_serde")]
    impl Serialize for TestInvalidSkeleton {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            let fields = &self.0;
            let mut seq = serializer.serialize_seq(Some(fields.len()))?;
            for item in fields.iter() {
                seq.serialize_element(item)?;
            }
            seq.end()
        }
    }

    #[test]
    fn test_skeleton_bincode_reordering() {
        let unordered_skeleton = TestInvalidSkeleton(vec![
            Field::from((FieldSymbol::Day(Day::DayOfMonth), FieldLength::One)),
            Field::from((FieldSymbol::Month(Month::Format), FieldLength::One)),
        ]);

        let mut buffer: Vec<u8> = Vec::new();

        bincode::serialize_into(&mut buffer, &unordered_skeleton).unwrap();

        let err =
            bincode::deserialize::<Skeleton>(&buffer).expect_err("Expected an unordered error");

        assert_eq!(
            format!("{}", err),
            "invalid value: field item out of order or duplicate: Field { symbol: Month(Format), length: One }, expected ordered field symbols representing a skeleton"
        );
    }

    #[test]
    fn test_skeleton_bincode_duplicate_field() {
        let unordered_skeleton = TestInvalidSkeleton(vec![
            Field::from((FieldSymbol::Month(Month::Format), FieldLength::One)),
            Field::from((FieldSymbol::Day(Day::DayOfMonth), FieldLength::One)),
            Field::from((FieldSymbol::Day(Day::DayOfMonth), FieldLength::One)),
        ]);

        let mut buffer: Vec<u8> = Vec::new();

        bincode::serialize_into(&mut buffer, &unordered_skeleton).unwrap();

        let err = bincode::deserialize::<Skeleton>(&buffer)
            .expect_err("Expected a duplicate field error");

        assert_eq!(
            format!("{}", err),
            "invalid value: field item out of order or duplicate: Field { symbol: Day(DayOfMonth), length: One }, expected ordered field symbols representing a skeleton"
        );
    }
}
