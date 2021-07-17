// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::iter::Iterator;
use std::str::FromStr;
use std::u32;

use crate::support::UnicodeProperties;
use icu_uniset::provider::UnicodePropertyV1;
use icu_uniset::{UnicodeSet, UnicodeSetBuilder};
use tinystr::TinyStr16;

//
// Provider-related structs and impl functions
//

fn split_line(line: &str) -> Vec<&str> {
    line.split(';').collect::<Vec<_>>()
}

/// For lines in PPUCD that begin with "defaults", "blk", and "cp" -- the lines
/// that convey property values for code points (or ranges thereof) -- return
/// the property information as a map. Enumerated properties' names and values
/// become keys and values in the map, while the binary property strings are
/// stored as the key and value in the map. Binary property strings are stored
/// as-is, so no parsing occurs here when the "false" state of a binary
/// property is represented with a minus ("-") prefix before the name.
fn get_data_line_prop_vals<'s>(data_line_parts: &[&'s str]) -> HashMap<&'s str, &'s str> {
    let mut m = HashMap::new();
    // idx 0 = line type identifier (property, value, defaults, blk, cp)
    // idx 1 = data line code point value or code point range
    let line_parts = &data_line_parts[2..];
    for prop_str in line_parts {
        let first_equals_idx = prop_str.find('=');
        if let Some(idx) = first_equals_idx {
            // Properties with an "=" in their string have values associated
            // (ex: enumerated properties).
            m.insert(&prop_str[0..idx], &prop_str[idx + 1..]);
        } else {
            // For properties that don't take values, let their value in the map be the prop name itself
            // This applies to binary properties.
            // Also, keep the string as-is to preserve any initial minus sign that
            // indicates for binary properties that this line excludes this code
            // point range from inclusion in the property set.
            m.insert(prop_str, prop_str);
        }
    }
    m
}

fn is_skip_ppucd_line(line: &str) -> bool {
    line.starts_with('#') || line.starts_with("ucd") || line.is_empty()
}

fn is_property_line(line: &str) -> bool {
    line.starts_with("property;")
}

fn is_enum_val_line(line: &str) -> bool {
    line.starts_with("value;")
}

/// For a property definition or enumerated property value line, update the aliases map.
fn update_aliases<'s>(prop_aliases: &mut HashMap<&'s str, HashSet<&'s str>>, line: &'s str) {
    let line_parts = split_line(line);
    let mut line_parts: &[&str] = line_parts.as_slice();
    line_parts = &line_parts[2..];

    // Properties defined in UTS 18 but not in UCD may have an empty line part in PPUCD
    // because of the non-existence in UCD. See Compatibility Properties in UTS 18 Annex C
    // that provide backwards compatibility for POSIX-style properties:
    // https://unicode.org/reports/tr18/#Compatibility_Properties
    if line_parts[0].is_empty() {
        line_parts = &line_parts[1..];
    }

    let canonical_name = line_parts[0];
    let all_names_set: HashSet<&'s str> = line_parts.iter().copied().collect();
    prop_aliases.insert(canonical_name, all_names_set);
}

/// Parse property definition line for binary and enumerated properties.
fn update_property_aliases<'s>(
    binary_prop_aliases: &mut HashMap<&'s str, HashSet<&'s str>>,
    enum_prop_aliases: &mut HashMap<&'s str, HashSet<&'s str>>,
    line: &'s str,
) {
    let line_parts = split_line(line);
    assert_eq!(&"property", &line_parts[0]);
    let prop_type = &line_parts[1];
    if prop_type == &"Binary" {
        update_aliases(binary_prop_aliases, line);
    } else if prop_type == &"Enumerated" {
        update_aliases(enum_prop_aliases, line);
    }
}

/// Parse enum property value definition line.
fn update_enum_val_aliases<'s>(
    enum_val_aliases: &mut HashMap<&'s str, HashMap<&'s str, HashSet<&'s str>>>,
    line: &'s str,
) {
    let line_parts = split_line(line);
    let mut line_parts: &[&str] = line_parts.as_slice();
    assert_eq!(&"value", &line_parts[0]);
    line_parts = &line_parts[1..];
    let enum_prop_name = line_parts[0];
    let enum_prop_val = line_parts[1];
    enum_val_aliases
        .entry(enum_prop_name)
        .or_insert_with(HashMap::new);
    let enum_val_alias_map: &mut HashMap<&str, HashSet<&str>> =
        enum_val_aliases.get_mut(&enum_prop_name).unwrap();
    enum_val_alias_map
        .entry(enum_prop_val)
        .or_insert_with(HashSet::new);
    let enum_prop_val_aliases: &mut HashSet<&str> =
        enum_val_alias_map.get_mut(&enum_prop_val).unwrap();
    enum_prop_val_aliases.insert(enum_prop_val);
    line_parts = &line_parts[2..];
    // What remains of line_parts are all of the remaining aliases for this
    // enumerated property's value
    for alias in line_parts {
        enum_prop_val_aliases.insert(alias);
    }
}

/// Mutate the map so any binary exclusion values
/// (in other words, values for binary properties that are prefixed with a
/// minus sign, as described in the PPUCD documentation, ex: "-Gr_Base")
/// are not included and any existing include values for the binary property
/// (ex: "Gr_Base") are accordingly also removed from the map.
fn apply_exclude_vals_for_binary_props<'s>(prop_vals: &mut HashMap<&'s str, &'s str>) {
    let mut prop_names: HashSet<&'s str> = prop_vals.keys().copied().collect();
    // If we see "-Gr_Base", then remove both "Gr_Base" and "-Gr_Base".
    for prop_name in prop_vals.keys() {
        if let Some(orig_prop_name) = prop_name.strip_prefix('-') {
            prop_names.remove(&orig_prop_name);
            prop_names.remove(prop_name);
        }
    }
    prop_vals.retain(|prop_name, _| prop_names.contains(prop_name));
}

fn is_defaults_line(line: &str) -> bool {
    line.starts_with("defaults;")
}

/// Return the property key-value information represented in the "defaults"
/// line as a map. "defaults" is like the base level of overrides of property
/// values for all code points in PPUCD.
fn get_defaults_prop_vals(line: &str) -> HashMap<&str, &str> {
    let line_parts = split_line(line);
    assert_eq!(&"defaults", &line_parts[0]);
    get_data_line_prop_vals(&line_parts)
}

fn is_block_line(line: &str) -> bool {
    line.starts_with("block;")
}

/// Return the property key-value information represented in a "blk"
/// line as a map. "blocks" represent overrides of property values for code
/// points in PPUCD within a Unicode block above the "defaults" values.
fn get_block_range_prop_vals(line: &str) -> (UnicodeSet, HashMap<&str, &str>) {
    let line_parts = split_line(line);
    assert_eq!(&"block", &line_parts[0]);

    let range_str = &line_parts[1];
    let range_bound_strs = range_str.split("..").collect::<Vec<_>>();
    let range_start: &u32 = &u32::from_str_radix(range_bound_strs[0], 16).unwrap();
    let range_end: &u32 = &u32::from_str_radix(range_bound_strs[1], 16).unwrap(); // inclusive end val in PPUCD
    let inv_list_start: u32 = *range_start;
    let inv_list_end: u32 = *range_end + 1;
    let inv_list: Vec<u32> = vec![inv_list_start, inv_list_end];
    let range_result = UnicodeSet::from_inversion_list(inv_list);

    let props_vals = get_data_line_prop_vals(&line_parts);

    let range = if let Ok(range) = range_result {
        range
    } else {
        let inv_list: Vec<u32> = Vec::default();
        UnicodeSet::from_inversion_list(inv_list).unwrap()
    };
    (range, props_vals)
}

fn is_code_point_line(line: &str) -> bool {
    line.starts_with("cp;")
}

/// Return the property key-value information represented in a "cp"
/// line as a map. "cp" represents overrides of property values for a code
/// point (or range of code points) that are layered above "blk" and "defaults".
fn get_code_point_overrides(line: &str) -> (UnicodeSet, HashMap<&str, &str>) {
    let line_parts = split_line(line);
    assert_eq!(&"cp", &line_parts[0]);

    let range_str = &line_parts[1];
    let range_bound_strs = &range_str.split("..").collect::<Vec<_>>();
    // a "cp" line in PPUCD can either represent a single code point or a code point range
    let range_result = if range_bound_strs.len() > 1 {
        let range_start: &u32 = &u32::from_str_radix(range_bound_strs[0], 16).unwrap();
        let range_end: &u32 = &u32::from_str_radix(range_bound_strs[1], 16).unwrap(); // inclusive end val in PPUCD
        let inv_list_start: u32 = *range_start;
        let inv_list_end: u32 = *range_end + 1;
        let inv_list: Vec<u32> = vec![inv_list_start, inv_list_end];
        UnicodeSet::from_inversion_list(inv_list)
    } else {
        let code_point_str = range_str;
        let code_point: u32 = u32::from_str_radix(code_point_str, 16).unwrap();
        let inv_list: Vec<u32> = vec![code_point, code_point + 1];
        UnicodeSet::from_inversion_list(inv_list)
    };

    let props_vals = get_data_line_prop_vals(&line_parts);

    let range = if let Ok(range) = range_result {
        range
    } else {
        let inv_list: Vec<u32> = Vec::default();
        UnicodeSet::from_inversion_list(inv_list).unwrap()
    };
    (range, props_vals)
}

/// For a single code point, apply the overrides from the PPUCD lines
/// "defaults", "blk", and "cp".
fn get_code_point_prop_vals<'s>(
    code_point: u32,
    code_point_overrides: &HashMap<UnicodeSet, HashMap<&'s str, &'s str>>,
    blocks: &HashMap<UnicodeSet, HashMap<&'s str, &'s str>>,
    defaults: &HashMap<&'s str, &'s str>,
) -> HashMap<&'s str, &'s str> {
    // create the map of applicable property values, and initialize it to
    // defaults for the whole Unicode range
    let mut prop_vals: HashMap<&'s str, &'s str> = HashMap::new();
    prop_vals.clone_from(defaults);

    // determine if this code point matches any of the block-wide defaults
    // ("overrides")
    for (range, block_prop_vals) in blocks {
        if range.contains_u32(code_point) {
            prop_vals.extend(block_prop_vals);
            // Apply any exclude overrides for a binary property as indicated
            // when the binary property name is prefixed with a "-"
            apply_exclude_vals_for_binary_props(&mut prop_vals);
        }
    }

    // finally, apply the overrides for this code point
    for (range, code_point_prop_vals) in code_point_overrides {
        if range.contains_u32(code_point) {
            prop_vals.extend(code_point_prop_vals);
            // Apply any exclude overrides for a binary property as indicated
            // when the binary property name is prefixed with a "-"
            apply_exclude_vals_for_binary_props(&mut prop_vals);
        }
    }

    prop_vals
}

/// Return a map of property names to UnicodeSets when given a map of known
/// binary property names and the assigned property key-value information for
/// each code point in PPUCD. This function effectively changes the "grouping"
/// dimension of code point <-> property key-value information from the
/// grouping by code point (as given by `code_points`) to a grouping by
/// property name.
fn get_binary_prop_unisets<'s>(
    prop_aliases: &HashMap<&'s str, HashSet<&'s str>>,
    code_points: &HashMap<u32, HashMap<&'s str, &'s str>>,
) -> HashMap<&'s str, UnicodeSet> {
    let mut m: HashMap<&'s str, UnicodeSet> = HashMap::new();

    for (canonical_name, all_names) in prop_aliases {
        let mut uniset_builder = UnicodeSetBuilder::new();
        'outer: for (code_point, code_point_prop_vals) in code_points {
            for prop_key in code_point_prop_vals.keys() {
                if all_names.contains(prop_key) {
                    uniset_builder.add_u32(*code_point);
                    continue 'outer;
                }
            }
        }
        if !&uniset_builder.is_empty() {
            let uniset = uniset_builder.build();
            m.insert(canonical_name, uniset);
        }
    }

    m
}

/// Return a Map of `UnicodeSet`s for each of the enumerated properties'
/// values. The key in the map will be a string slice created from the
/// combination of the enumerated property name and property value (ex:
/// `"gc=Lo"`), and the value is the corresponding `UnicodeSet`.
fn get_enum_prop_unisets<'s>(
    enum_prop_aliases: &HashMap<&'s str, HashSet<&'s str>>,
    enum_val_aliases: &HashMap<&'s str, HashMap<&'s str, HashSet<&'s str>>>,
    code_points: &HashMap<u32, HashMap<&'s str, &'s str>>,
) -> HashMap<Cow<'s, TinyStr16>, UnicodeSet> {
    let mut m: HashMap<&str, HashMap<&str, UnicodeSetBuilder>> = HashMap::new();

    let enum_val_mappings: HashMap<&str, HashMap<&str, &str>> =
        get_enum_val_canonical_mapping(enum_val_aliases);

    for (canonical_prop_name, all_prop_name_aliases) in enum_prop_aliases {
        for (code_point, code_point_prop_key_vals) in code_points {
            let code_point_prop_names: HashSet<&str> =
                code_point_prop_key_vals.keys().copied().collect();
            if !all_prop_name_aliases.is_disjoint(&code_point_prop_names) {
                for prop_name in all_prop_name_aliases.intersection(&code_point_prop_names) {
                    let val_name: &str = code_point_prop_key_vals.get(prop_name).unwrap();
                    let canonicalized_val_name: &str = enum_val_mappings
                        .get(canonical_prop_name)
                        .unwrap()
                        .get(val_name)
                        .unwrap();

                    if !m.contains_key(canonical_prop_name) {
                        m.insert(canonical_prop_name, HashMap::new());
                    }

                    if !m
                        .get(canonical_prop_name)
                        .unwrap()
                        .contains_key(canonicalized_val_name)
                    {
                        let result_prop_val_builder_map: &mut HashMap<&str, UnicodeSetBuilder> =
                            m.get_mut(canonical_prop_name).unwrap();
                        result_prop_val_builder_map
                            .insert(canonicalized_val_name, UnicodeSetBuilder::new());
                    }

                    let enum_val_uniset_builder: &mut UnicodeSetBuilder = m
                        .get_mut(canonical_prop_name)
                        .unwrap()
                        .get_mut(canonicalized_val_name)
                        .unwrap();
                    enum_val_uniset_builder.add_char(std::char::from_u32(*code_point).unwrap());
                }
            }
        }
    }

    let mut result: HashMap<Cow<'s, TinyStr16>, UnicodeSet> = HashMap::new();

    // Insert UnicodeSets into `result`, with a key like `"gc=Lo"` that
    // represents the combination of the Unicode
    // enumerated property name (`gc` or `General_Category`) and the Unicode
    // enumerated property value (`Lo` or `Other_letter`).
    // Note: because we want to revert the use of enum integer values in the
    // key in favor of the Unicode property short aliases, the key strings
    // longer than the length of TinyStr16 will not be included in the parsed
    // result.
    for (canonical_prop_name, prop_val_builder_map) in m {
        for (canonical_val_name, uniset_builder) in prop_val_builder_map {
            let uniset_name_result =
                TinyStr16::from_str(&format!("{}={}", canonical_prop_name, canonical_val_name));
            if let Ok(uniset_name) = uniset_name_result {
                let enum_val_uniset_name: Cow<'s, TinyStr16> = Cow::Owned(uniset_name);
                let uniset = uniset_builder.build();
                result.insert(enum_val_uniset_name, uniset);
            }
        }
    }

    result
}

fn aliases_as_canonical_mappings<'s>(
    aliases_map: &HashMap<&'s str, HashSet<&'s str>>,
) -> HashMap<&'s str, &'s str> {
    let mut result: HashMap<&str, &str> = HashMap::new();
    for (canonical_name, aliases) in aliases_map {
        result.insert(<&str>::clone(canonical_name), <&str>::clone(canonical_name));
        for alias in aliases {
            result.insert(<&str>::clone(alias), <&str>::clone(canonical_name));
        }
    }

    result
}

fn get_enum_val_canonical_mapping<'s>(
    enum_val_aliases: &HashMap<&'s str, HashMap<&'s str, HashSet<&'s str>>>,
) -> HashMap<&'s str, HashMap<&'s str, &'s str>> {
    let mut result: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
    for (enum_prop_canon_name, enum_val_aliases) in enum_val_aliases {
        let enum_val_canonical_mapping = aliases_as_canonical_mappings(enum_val_aliases);
        result.insert(
            <&str>::clone(enum_prop_canon_name),
            enum_val_canonical_mapping,
        );
    }

    result
}

/// Parse a whole PPUCD file that was loaded into a string slice and return a
/// struct of the binary and enumerated property inversion lists.
/// Note: even though `UnicodeProperties` stores a sequential data structure of
/// the `UnicodePropertyV1` struct, there is no inherent ordering of the entries.
pub fn parse<'s>(s: &'s str) -> UnicodeProperties<'s> {
    let lines: std::str::Lines = s.lines();

    let parseable_lines = lines.filter(|line| !is_skip_ppucd_line(line));

    let mut binary_prop_aliases: HashMap<&'s str, HashSet<&'s str>> = HashMap::new();

    let mut enum_prop_aliases: HashMap<&'s str, HashSet<&'s str>> = HashMap::new();

    let mut enum_val_aliases: HashMap<&'s str, HashMap<&'s str, HashSet<&'s str>>> = HashMap::new();

    let mut defaults: HashMap<&'s str, &'s str> = HashMap::new();

    // UnicodeSet is used to store the code point range described in a PPUCD
    // `block` line
    let mut blocks: HashMap<UnicodeSet, HashMap<&'s str, &'s str>> = HashMap::new();

    // UnicodeSet is used to store the code point or code point range described
    // in a PPUCD `cp` line, according to the PPUCD file format spec.
    let mut code_point_overrides: HashMap<UnicodeSet, HashMap<&'s str, &'s str>> = HashMap::new();

    // current implementation uses this data structure to pull all of the code
    // point info into memory
    let mut code_points: HashMap<u32, HashMap<&'s str, &'s str>> = HashMap::new();

    // parse PPUCD to fill out data structures for info of property name aliases
    // and overrides at defaults/blocks/cp levels
    for line in parseable_lines {
        if is_property_line(line) {
            update_property_aliases(&mut binary_prop_aliases, &mut enum_prop_aliases, line);
        } else if is_enum_val_line(line) {
            update_enum_val_aliases(&mut enum_val_aliases, line);
        } else if is_defaults_line(line) {
            defaults = get_defaults_prop_vals(line);
        } else if is_block_line(line) {
            let (range, prop_vals) = get_block_range_prop_vals(line);
            blocks.insert(range, prop_vals);
        } else if is_code_point_line(line) {
            // record code point override vals directly from line
            let (code_point_range, prop_vals) = get_code_point_overrides(line);
            code_point_overrides.insert(code_point_range, prop_vals);

            // compute final code point property vals after applying all
            // levels of overrides
            // can't clone UnicodeSet, so recomputing code point range
            // TODO: can we allow UnicodeSet to derive Clone ?
            let (code_point_range, _) = get_code_point_overrides(line);
            for code_point_char in code_point_range.iter_chars() {
                let code_point = code_point_char as u32;
                let code_point_prop_vals =
                    get_code_point_prop_vals(code_point, &code_point_overrides, &blocks, &defaults);
                code_points.insert(code_point, code_point_prop_vals);
            }
        }
    }

    // This vector becomes the return value for the fn. Push each new
    // `UnicodePropertyV1` constructed from each UnicodeSet + name for all of the
    // binary properties and enumerated properties parsed from the input.
    let mut props: Vec<UnicodePropertyV1> = vec![];

    let binary_prop_unisets: HashMap<&'s str, UnicodeSet> =
        get_binary_prop_unisets(&binary_prop_aliases, &code_points);

    let enum_prop_unisets: HashMap<Cow<'s, TinyStr16>, UnicodeSet> =
        get_enum_prop_unisets(&enum_prop_aliases, &enum_val_aliases, &code_points);

    for (canonical_name, uniset) in binary_prop_unisets {
        let ppucd_prop: UnicodePropertyV1 =
            UnicodePropertyV1::from_uniset(&uniset, Cow::Borrowed(canonical_name));
        props.push(ppucd_prop);
    }

    for (key_val_tuple_name, uniset) in enum_prop_unisets {
        let key_val_tuple_name_str: Cow<'s, str> = match key_val_tuple_name {
            Cow::Borrowed(tiny_str) => Cow::Borrowed(tiny_str.as_str()),
            Cow::Owned(tiny_str) => Cow::Owned(tiny_str.to_string()),
        };
        let ppucd_prop: UnicodePropertyV1 =
            UnicodePropertyV1::from_uniset(&uniset, key_val_tuple_name_str);
        props.push(ppucd_prop);
    }

    UnicodeProperties { props }
}

#[cfg(test)]
mod gen_properties_test {
    use super::*;

    #[test]
    fn skip_ppucd_line_test() {
        assert_eq!(true, is_skip_ppucd_line("ucd;13.0.0"));
        assert_eq!(
            false,
            is_skip_ppucd_line("value;InSC;Gemination_Mark;Gemination_Mark")
        );
    }

    #[test]
    fn split_line_test() {
        let line = "cp;0020;bc=WS;gc=Zs;lb=SP;na=SPACE;Name_Alias=abbreviation=SP;Pat_WS;SB=SP;WB=WSegSpace;WSpace";
        let exp_parts = vec![
            "cp",
            "0020",
            "bc=WS",
            "gc=Zs",
            "lb=SP",
            "na=SPACE",
            "Name_Alias=abbreviation=SP",
            "Pat_WS",
            "SB=SP",
            "WB=WSegSpace",
            "WSpace",
        ];
        let line_parts = split_line(&line);
        assert_eq!(exp_parts, line_parts);
    }

    #[test]
    fn get_data_line_prop_vals_test() {
        let line = "cp;0020;bc=WS;gc=Zs;lb=SP;na=SPACE;Name_Alias=abbreviation=SP;Pat_WS;SB=SP;WB=WSegSpace;WSpace";

        let mut exp_prop_vals: HashMap<&str, &str> = HashMap::new();
        exp_prop_vals.insert("bc", "WS");
        exp_prop_vals.insert("gc", "Zs");
        exp_prop_vals.insert("lb", "SP");
        exp_prop_vals.insert("na", "SPACE");
        exp_prop_vals.insert("Name_Alias", "abbreviation=SP");
        exp_prop_vals.insert("Pat_WS", "Pat_WS");
        exp_prop_vals.insert("SB", "SP");
        exp_prop_vals.insert("WB", "WSegSpace");
        exp_prop_vals.insert("WSpace", "WSpace");

        let line_parts = split_line(&line);
        let act_props_vals = get_data_line_prop_vals(&line_parts);

        assert_eq!(exp_prop_vals, act_props_vals);
    }

    #[test]
    fn update_aliases_test() {
        let line = "property;Binary;Upper;Uppercase";
        let mut prop_aliases: HashMap<&str, HashSet<&str>> = HashMap::new();

        update_aliases(&mut prop_aliases, &line);

        let mut exp_prop_aliases: HashMap<&str, HashSet<&str>> = HashMap::new();
        let exp_aliases = vec!["Uppercase", "Upper"]; // order won't matter
        let exp_aliases: HashSet<&str> = exp_aliases.into_iter().collect();
        exp_prop_aliases.insert("Upper", exp_aliases);

        assert_eq!(&exp_prop_aliases, &prop_aliases);
    }

    #[test]
    fn block_range_prop_vals_test() {
        let line = "block;0000..007F;age=1.1;blk=ASCII;ea=Na;gc=Cc;Gr_Base;lb=AL;sc=Zyyy";

        let mut exp_prop_vals: HashMap<&str, &str> = HashMap::new();
        exp_prop_vals.insert("age", "1.1");
        exp_prop_vals.insert("blk", "ASCII");
        exp_prop_vals.insert("ea", "Na");
        exp_prop_vals.insert("gc", "Cc");
        exp_prop_vals.insert("Gr_Base", "Gr_Base");
        exp_prop_vals.insert("lb", "AL");
        exp_prop_vals.insert("sc", "Zyyy");

        let exp_range_inv_list: Vec<u32> = vec![0, 128]; // PPUCD: end val is inclusive;
                                                         // inversion list: end val exclusive
        let exp_range = UnicodeSet::from_inversion_list(exp_range_inv_list).unwrap();

        let (range, prop_vals) = get_block_range_prop_vals(&line);

        assert_eq!(exp_range, range);
        assert_eq!(exp_prop_vals, prop_vals);
    }

    #[test]
    fn code_point_overrides_test() {
        let defaults_line = "defaults;0000..10FFFF;age=NA;bc=L;blk=NB;bpt=n;cf=<code point>;dm=<code point>;dt=None;ea=N;FC_NFKC=<code point>;gc=Cn;GCB=XX;gcm=Cn;hst=NA;InPC=NA;InSC=Other;jg=No_Joining_Group;jt=U;lb=XX;lc=<code point>;NFC_QC=Y;NFD_QC=Y;NFKC_CF=<code point>;NFKC_QC=Y;NFKD_QC=Y;nt=None;SB=XX;sc=Zzzz;scf=<code point>;scx=<script>;slc=<code point>;stc=<code point>;suc=<code point>;tc=<code point>;uc=<code point>;vo=R;WB=XX";
        let block_line = "block;0000..007F;age=1.1;blk=ASCII;ea=Na;gc=Cc;Gr_Base;lb=AL;sc=Zyyy";
        let code_point_line = "cp;0020;bc=WS;gc=Zs;lb=SP;na=SPACE;Name_Alias=abbreviation=SP;Pat_WS;SB=SP;WB=WSegSpace;WSpace";

        let exp_code_point: u32 = 32;
        let mut exp_code_point_prop_vals: HashMap<&str, &str> = HashMap::new();
        exp_code_point_prop_vals.insert("WB", "WSegSpace");
        exp_code_point_prop_vals.insert("FC_NFKC", "<code point>");
        exp_code_point_prop_vals.insert("GCB", "XX");
        exp_code_point_prop_vals.insert("gcm", "Cn");
        exp_code_point_prop_vals.insert("scx", "<script>");
        exp_code_point_prop_vals.insert("bc", "WS");
        exp_code_point_prop_vals.insert("lc", "<code point>");
        exp_code_point_prop_vals.insert("NFKC_CF", "<code point>");
        exp_code_point_prop_vals.insert("InPC", "NA");
        exp_code_point_prop_vals.insert("lb", "SP");
        exp_code_point_prop_vals.insert("NFC_QC", "Y");
        exp_code_point_prop_vals.insert("SB", "SP");
        exp_code_point_prop_vals.insert("Name_Alias", "abbreviation=SP");
        exp_code_point_prop_vals.insert("dt", "None");
        exp_code_point_prop_vals.insert("ea", "Na");
        exp_code_point_prop_vals.insert("nt", "None");
        exp_code_point_prop_vals.insert("InSC", "Other");
        exp_code_point_prop_vals.insert("NFD_QC", "Y");
        exp_code_point_prop_vals.insert("bpt", "n");
        exp_code_point_prop_vals.insert("jg", "No_Joining_Group");
        exp_code_point_prop_vals.insert("gc", "Zs");
        exp_code_point_prop_vals.insert("vo", "R");
        exp_code_point_prop_vals.insert("NFKD_QC", "Y");
        exp_code_point_prop_vals.insert("NFKC_QC", "Y");
        exp_code_point_prop_vals.insert("blk", "ASCII");
        exp_code_point_prop_vals.insert("uc", "<code point>");
        exp_code_point_prop_vals.insert("suc", "<code point>");
        exp_code_point_prop_vals.insert("scf", "<code point>");
        exp_code_point_prop_vals.insert("slc", "<code point>");
        exp_code_point_prop_vals.insert("age", "1.1");
        exp_code_point_prop_vals.insert("na", "SPACE");
        exp_code_point_prop_vals.insert("cf", "<code point>");
        exp_code_point_prop_vals.insert("hst", "NA");
        exp_code_point_prop_vals.insert("dm", "<code point>");
        exp_code_point_prop_vals.insert("sc", "Zyyy");
        exp_code_point_prop_vals.insert("stc", "<code point>");
        exp_code_point_prop_vals.insert("jt", "U");
        exp_code_point_prop_vals.insert("tc", "<code point>");
        exp_code_point_prop_vals.insert("Gr_Base", "Gr_Base");
        exp_code_point_prop_vals.insert("WSpace", "WSpace");
        exp_code_point_prop_vals.insert("Pat_WS", "Pat_WS");
        let defaults: HashMap<&str, &str> = get_defaults_prop_vals(&defaults_line);
        let mut exp_code_points: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        exp_code_points.insert(exp_code_point, exp_code_point_prop_vals);

        let mut blocks: HashMap<UnicodeSet, HashMap<&str, &str>> = HashMap::new();
        let (range, prop_vals) = get_block_range_prop_vals(&block_line);
        blocks.insert(range, prop_vals);

        let mut code_point_overrides: HashMap<UnicodeSet, HashMap<&str, &str>> = HashMap::new();
        let (code_point_range, prop_vals) = get_code_point_overrides(&code_point_line);
        code_point_overrides.insert(code_point_range, prop_vals);

        let (code_point_range, _) = get_code_point_overrides(&code_point_line);
        let mut code_points: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        for code_point_char in code_point_range.iter_chars() {
            let code_point = code_point_char as u32;
            let code_point_prop_vals =
                get_code_point_prop_vals(code_point, &code_point_overrides, &blocks, &defaults);
            code_points.insert(code_point, code_point_prop_vals);
        }

        assert_eq!(&exp_code_points, &code_points);
    }

    #[test]
    fn apply_exclude_vals_for_binary_props_test() {
        let exp_binary_props1: HashMap<&str, &str> = HashMap::new();
        let mut input_binary_props1: HashMap<&str, &str> = HashMap::new();
        input_binary_props1.insert("Gr_Base", "Gr_Base");
        input_binary_props1.insert("-Gr_Base", "-Gr_Base");
        apply_exclude_vals_for_binary_props(&mut input_binary_props1);
        assert_eq!(exp_binary_props1, input_binary_props1);

        let mut exp_binary_props2: HashMap<&str, &str> = HashMap::new();
        exp_binary_props2.insert("Gr_Base", "Gr_Base");
        let mut input_binary_props2: HashMap<&str, &str> = HashMap::new();
        input_binary_props2.insert("Gr_Base", "Gr_Base");
        apply_exclude_vals_for_binary_props(&mut input_binary_props2);
        assert_eq!(exp_binary_props2, input_binary_props2);
    }

    #[test]
    fn code_point_overrides_binary_prop_minus_prefix_test() {
        let defaults_line = "defaults;0000..10FFFF;age=NA;bc=L;blk=NB;bpt=n;cf=<code point>;dm=<code point>;dt=None;ea=N;FC_NFKC=<code point>;gc=Cn;GCB=XX;gcm=Cn;hst=NA;InPC=NA;InSC=Other;jg=No_Joining_Group;jt=U;lb=XX;lc=<code point>;NFC_QC=Y;NFD_QC=Y;NFKC_CF=<code point>;NFKC_QC=Y;NFKD_QC=Y;nt=None;SB=XX;sc=Zzzz;scf=<code point>;scx=<script>;slc=<code point>;stc=<code point>;suc=<code point>;tc=<code point>;uc=<code point>;vo=R;WB=XX";
        let block_line = "block;0000..007F;age=1.1;blk=ASCII;ea=Na;gc=Cc;Gr_Base;lb=AL;sc=Zyyy";
        let code_point_line = "cp;000D;bc=B;ea=N;GCB=CR;-Gr_Base;lb=CR;Name_Alias=control=CARRIAGE RETURN,abbreviation=CR;Pat_WS;SB=CR;WB=CR;WSpace";

        let exp_code_point: u32 = 13;
        let mut exp_code_point_prop_vals: HashMap<&str, &str> = HashMap::new();
        exp_code_point_prop_vals.insert("WB", "CR");
        exp_code_point_prop_vals.insert("FC_NFKC", "<code point>");
        exp_code_point_prop_vals.insert("GCB", "CR");
        exp_code_point_prop_vals.insert("gcm", "Cn");
        exp_code_point_prop_vals.insert("scx", "<script>");
        exp_code_point_prop_vals.insert("bc", "B");
        exp_code_point_prop_vals.insert("lc", "<code point>");
        exp_code_point_prop_vals.insert("NFKC_CF", "<code point>");
        exp_code_point_prop_vals.insert("InPC", "NA");
        exp_code_point_prop_vals.insert("lb", "CR");
        exp_code_point_prop_vals.insert("NFC_QC", "Y");
        exp_code_point_prop_vals.insert("SB", "CR");
        exp_code_point_prop_vals.insert("Name_Alias", "control=CARRIAGE RETURN,abbreviation=CR");
        exp_code_point_prop_vals.insert("dt", "None");
        exp_code_point_prop_vals.insert("ea", "N");
        exp_code_point_prop_vals.insert("nt", "None");
        exp_code_point_prop_vals.insert("InSC", "Other");
        exp_code_point_prop_vals.insert("NFD_QC", "Y");
        exp_code_point_prop_vals.insert("bpt", "n");
        exp_code_point_prop_vals.insert("jg", "No_Joining_Group");
        exp_code_point_prop_vals.insert("gc", "Cc");
        exp_code_point_prop_vals.insert("vo", "R");
        exp_code_point_prop_vals.insert("NFKD_QC", "Y");
        exp_code_point_prop_vals.insert("NFKC_QC", "Y");
        exp_code_point_prop_vals.insert("blk", "ASCII");
        exp_code_point_prop_vals.insert("uc", "<code point>");
        exp_code_point_prop_vals.insert("suc", "<code point>");
        exp_code_point_prop_vals.insert("scf", "<code point>");
        exp_code_point_prop_vals.insert("slc", "<code point>");
        exp_code_point_prop_vals.insert("age", "1.1");
        // exp_code_point_prop_vals.insert("na", "SPACE");
        exp_code_point_prop_vals.insert("cf", "<code point>");
        exp_code_point_prop_vals.insert("hst", "NA");
        exp_code_point_prop_vals.insert("dm", "<code point>");
        exp_code_point_prop_vals.insert("sc", "Zyyy");
        exp_code_point_prop_vals.insert("stc", "<code point>");
        exp_code_point_prop_vals.insert("jt", "U");
        exp_code_point_prop_vals.insert("tc", "<code point>");
        // exp_code_point_prop_vals.insert("Gr_Base"), "Gr_Base"));
        exp_code_point_prop_vals.insert("WSpace", "WSpace");
        exp_code_point_prop_vals.insert("Pat_WS", "Pat_WS");
        let defaults: HashMap<&str, &str> = get_defaults_prop_vals(&defaults_line);
        let mut exp_code_points: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        exp_code_points.insert(exp_code_point, exp_code_point_prop_vals);

        let mut blocks: HashMap<UnicodeSet, HashMap<&str, &str>> = HashMap::new();
        let (range, prop_vals) = get_block_range_prop_vals(&block_line);
        blocks.insert(range, prop_vals);

        let mut code_point_overrides: HashMap<UnicodeSet, HashMap<&str, &str>> = HashMap::new();
        let (code_point_range, prop_vals) = get_code_point_overrides(&code_point_line);
        code_point_overrides.insert(code_point_range, prop_vals);

        let (code_point_range, _) = get_code_point_overrides(&code_point_line);
        let mut code_points: HashMap<u32, HashMap<&str, &str>> = HashMap::new();
        for code_point_char in code_point_range.iter_chars() {
            let code_point = code_point_char as u32;
            let code_point_prop_vals =
                get_code_point_prop_vals(code_point, &code_point_overrides, &blocks, &defaults);
            code_points.insert(code_point, code_point_prop_vals);
        }

        assert_eq!(&exp_code_points, &code_points);
    }

    #[test]
    fn update_enum_val_aliases_test() {
        let mut exp_enum_val_aliases: HashMap<&str, HashMap<&str, HashSet<&str>>> = HashMap::new();
        let exp_key = "gcm";
        let mut exp_val_alias_map: HashMap<&str, HashSet<&str>> = HashMap::new();
        let mut sc_set: HashSet<&str> = HashSet::new();
        sc_set.insert("Sc");
        sc_set.insert("Currency_Symbol");
        let mut sk_set: HashSet<&str> = HashSet::new();
        sk_set.insert("Sk");
        sk_set.insert("Modifier_Symbol");
        let mut sm_set: HashSet<&str> = HashSet::new();
        sm_set.insert("Sm");
        sm_set.insert("Math_Symbol");
        let mut so_set: HashSet<&str> = HashSet::new();
        so_set.insert("So");
        so_set.insert("Other_Symbol");
        exp_val_alias_map.insert("Sc", sc_set);
        exp_val_alias_map.insert("Sk", sk_set);
        exp_val_alias_map.insert("Sm", sm_set);
        exp_val_alias_map.insert("So", so_set);
        exp_enum_val_aliases.insert(exp_key, exp_val_alias_map);

        let mut act_enum_val_aliases: HashMap<&str, HashMap<&str, HashSet<&str>>> = HashMap::new();
        let line1 = "value;gcm;Sc;Currency_Symbol";
        let line2 = "value;gcm;Sk;Modifier_Symbol";
        let line3 = "value;gcm;Sm;Math_Symbol";
        let line4 = "value;gcm;So;Other_Symbol";
        update_enum_val_aliases(&mut act_enum_val_aliases, &line1);
        update_enum_val_aliases(&mut act_enum_val_aliases, &line2);
        update_enum_val_aliases(&mut act_enum_val_aliases, &line3);
        update_enum_val_aliases(&mut act_enum_val_aliases, &line4);

        assert_eq!(exp_enum_val_aliases, act_enum_val_aliases);
    }

    #[test]
    fn aliases_as_canonical_mappings_test() {
        let mut exp_mappings: HashMap<&str, &str> = HashMap::new();
        exp_mappings.insert("Sc", "Sc");
        exp_mappings.insert("Currency_Symbol", "Sc");
        exp_mappings.insert("Sk", "Sk");
        exp_mappings.insert("Modifier_Symbol", "Sk");
        exp_mappings.insert("Sm", "Sm");
        exp_mappings.insert("Math_Symbol", "Sm");
        exp_mappings.insert("So", "So");
        exp_mappings.insert("Other_Symbol", "So");

        let mut symbol_props_val_alias_map: HashMap<&str, HashSet<&str>> = HashMap::new();
        let mut sc_set: HashSet<&str> = HashSet::new();
        sc_set.insert("Sc");
        sc_set.insert("Currency_Symbol");
        let mut sk_set: HashSet<&str> = HashSet::new();
        sk_set.insert("Sk");
        sk_set.insert("Modifier_Symbol");
        let mut sm_set: HashSet<&str> = HashSet::new();
        sm_set.insert("Sm");
        sm_set.insert("Math_Symbol");
        let mut so_set: HashSet<&str> = HashSet::new();
        so_set.insert("So");
        so_set.insert("Other_Symbol");
        symbol_props_val_alias_map.insert("Sc", sc_set);
        symbol_props_val_alias_map.insert("Sk", sk_set);
        symbol_props_val_alias_map.insert("Sm", sm_set);
        symbol_props_val_alias_map.insert("So", so_set);

        let act_mappings = aliases_as_canonical_mappings(&symbol_props_val_alias_map);
        assert_eq!(act_mappings, exp_mappings);
    }

    #[test]
    fn get_enum_val_canonical_mapping_test() {
        // expected
        let mut exp_enum_mapping: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
        let enum_prop = "gcm";
        let mut exp_mapping: HashMap<&str, &str> = HashMap::new();
        exp_mapping.insert("Sc", "Sc");
        exp_mapping.insert("Currency_Symbol", "Sc");
        exp_mapping.insert("Sk", "Sk");
        exp_mapping.insert("Modifier_Symbol", "Sk");
        exp_mapping.insert("Sm", "Sm");
        exp_mapping.insert("Math_Symbol", "Sm");
        exp_mapping.insert("So", "So");
        exp_mapping.insert("Other_Symbol", "So");
        exp_enum_mapping.insert(enum_prop, exp_mapping);

        // input
        let mut enum_val_aliases: HashMap<&str, HashMap<&str, HashSet<&str>>> = HashMap::new();
        let enum_prop = "gcm";
        let mut val_alias_map: HashMap<&str, HashSet<&str>> = HashMap::new();
        let mut sc_set: HashSet<&str> = HashSet::new();
        sc_set.insert("Sc");
        sc_set.insert("Currency_Symbol");
        let mut sk_set: HashSet<&str> = HashSet::new();
        sk_set.insert("Sk");
        sk_set.insert("Modifier_Symbol");
        let mut sm_set: HashSet<&str> = HashSet::new();
        sm_set.insert("Sm");
        sm_set.insert("Math_Symbol");
        let mut so_set: HashSet<&str> = HashSet::new();
        so_set.insert("So");
        so_set.insert("Other_Symbol");
        val_alias_map.insert("Sc", sc_set);
        val_alias_map.insert("Sk", sk_set);
        val_alias_map.insert("Sm", sm_set);
        val_alias_map.insert("So", so_set);
        enum_val_aliases.insert(enum_prop, val_alias_map);

        // actual
        let enum_val_canonical_mapping = get_enum_val_canonical_mapping(&enum_val_aliases);

        assert_eq!(enum_val_canonical_mapping, exp_enum_mapping);
    }

    // Test the functionality of enumerated property parsing.
    //
    // As a side effect when doing so in a direct-from-PPUCD manner, other
    // functionality that gets tested again: "-" prefix for binary props,
    // cascading of binary/enum props across defaults/blk/cp lines in PPUCD,
    // only parsing binary/enum props (ignore catalog props).
    #[test]
    fn parse_with_enum_props_test() {
        use std::iter::FromIterator;

        // Input
        let ppucd_property_files_root_path = "tests/testdata/ppucd-sc-tglg-test.txt";
        let ppucd_property_file_str =
            std::fs::read_to_string(ppucd_property_files_root_path).unwrap();
        // Actual
        let uni_props: UnicodeProperties = parse(&ppucd_property_file_str);
        // Convert actual to testable form
        let act_uni_props_set: HashSet<UnicodePropertyV1> = HashSet::from_iter(uni_props.props);

        // Expected
        let mut exp_uni_props_set: HashSet<UnicodePropertyV1> = HashSet::new();
        let gc_lo = UnicodePropertyV1 {
            name: Cow::Borrowed("gc=Lo"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(gc_lo);
        let gc_mn = UnicodePropertyV1 {
            name: Cow::Borrowed("gc=Mn"),
            inv_list: UnicodeSet::from_inversion_list(vec![5906, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(gc_mn);
        let gr_base = UnicodePropertyV1 {
            name: Cow::Borrowed("Gr_Base"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(gr_base);
        let idc = UnicodePropertyV1 {
            name: Cow::Borrowed("IDC"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(idc);
        let ids = UnicodePropertyV1 {
            name: Cow::Borrowed("IDS"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(ids);
        let insc_consonant = UnicodePropertyV1 {
            name: Cow::Borrowed("InSC=Consonant"),
            inv_list: UnicodeSet::from_inversion_list(vec![5891, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(insc_consonant);
        let insc_pure_killer = UnicodePropertyV1 {
            name: Cow::Borrowed("InSC=Pure_Killer"),
            inv_list: UnicodeSet::from_inversion_list(vec![5908, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(insc_pure_killer);
        let lb_al = UnicodePropertyV1 {
            name: Cow::Borrowed("lb=AL"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(lb_al);
        let lb_cm = UnicodePropertyV1 {
            name: Cow::Borrowed("lb=CM"),
            inv_list: UnicodeSet::from_inversion_list(vec![5906, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(lb_cm);
        let sb_le = UnicodePropertyV1 {
            name: Cow::Borrowed("SB=LE"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(sb_le);
        let sb_ex = UnicodePropertyV1 {
            name: Cow::Borrowed("SB=EX"),
            inv_list: UnicodeSet::from_inversion_list(vec![5906, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(sb_ex);
        let wb_le = UnicodePropertyV1 {
            name: Cow::Borrowed("WB=LE"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(wb_le);
        let wb_extend = UnicodePropertyV1 {
            name: Cow::Borrowed("WB=Extend"),
            inv_list: UnicodeSet::from_inversion_list(vec![5906, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(wb_extend);
        let xidc = UnicodePropertyV1 {
            name: Cow::Borrowed("XIDC"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(xidc);
        let xids = UnicodePropertyV1 {
            name: Cow::Borrowed("XIDS"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(xids);
        let bc_l = UnicodePropertyV1 {
            name: Cow::Borrowed("bc=L"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(bc_l);
        let bc_nsm = UnicodePropertyV1 {
            name: Cow::Borrowed("bc=NSM"),
            inv_list: UnicodeSet::from_inversion_list(vec![5906, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(bc_nsm);
        let ci = UnicodePropertyV1 {
            name: Cow::Borrowed("CI"),
            inv_list: UnicodeSet::from_inversion_list(vec![5906, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(ci);
        let gcb_ex = UnicodePropertyV1 {
            name: Cow::Borrowed("GCB=EX"),
            inv_list: UnicodeSet::from_inversion_list(vec![5906, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(gcb_ex);
        let gcb_xx = UnicodePropertyV1 {
            name: Cow::Borrowed("GCB=XX"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(gcb_xx);
        let gr_ext = UnicodePropertyV1 {
            name: Cow::Borrowed("Gr_Ext"),
            inv_list: UnicodeSet::from_inversion_list(vec![5906, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(gr_ext);
        let inpc_na = UnicodePropertyV1 {
            name: Cow::Borrowed("InPC=NA"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(inpc_na);
        let inpc_top = UnicodePropertyV1 {
            name: Cow::Borrowed("InPC=Top"),
            inv_list: UnicodeSet::from_inversion_list(vec![5906, 5907]).unwrap(),
        };
        exp_uni_props_set.insert(inpc_top);
        let inpc_bottom = UnicodePropertyV1 {
            name: Cow::Borrowed("InPC=Bottom"),
            inv_list: UnicodeSet::from_inversion_list(vec![5907, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(inpc_bottom);

        let jt_u = UnicodePropertyV1 {
            name: Cow::Borrowed("jt=U"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5906]).unwrap(),
        };
        exp_uni_props_set.insert(jt_u);
        let jt_t = UnicodePropertyV1 {
            name: Cow::Borrowed("jt=T"),
            inv_list: UnicodeSet::from_inversion_list(vec![5906, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(jt_t);
        let alpha = UnicodePropertyV1 {
            name: Cow::Borrowed("Alpha"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5908]).unwrap(),
        };
        exp_uni_props_set.insert(alpha);
        let nfd_qc_y = UnicodePropertyV1 {
            name: Cow::Borrowed("NFD_QC=Y"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(nfd_qc_y);
        let nfc_qc_y = UnicodePropertyV1 {
            name: Cow::Borrowed("NFC_QC=Y"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(nfc_qc_y);
        let nfkd_qc_y = UnicodePropertyV1 {
            name: Cow::Borrowed("NFKD_QC=Y"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(nfkd_qc_y);
        let nfkc_qc_y = UnicodePropertyV1 {
            name: Cow::Borrowed("NFKC_QC=Y"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(nfkc_qc_y);
        let dt_none = UnicodePropertyV1 {
            name: Cow::Borrowed("dt=None"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(dt_none);
        let nt_none = UnicodePropertyV1 {
            name: Cow::Borrowed("nt=None"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(nt_none);
        let ea_n = UnicodePropertyV1 {
            name: Cow::Borrowed("ea=N"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(ea_n);
        let vo_r = UnicodePropertyV1 {
            name: Cow::Borrowed("vo=R"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(vo_r);
        let ccc_9 = UnicodePropertyV1 {
            name: Cow::Borrowed("ccc=9"),
            inv_list: UnicodeSet::from_inversion_list(vec![5908, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(ccc_9);
        let hst_na = UnicodePropertyV1 {
            name: Cow::Borrowed("hst=NA"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(hst_na);
        let gr_link = UnicodePropertyV1 {
            name: Cow::Borrowed("Gr_Link"),
            inv_list: UnicodeSet::from_inversion_list(vec![5908, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(gr_link);
        let bpt_n = UnicodePropertyV1 {
            name: Cow::Borrowed("bpt=n"),
            inv_list: UnicodeSet::from_inversion_list(vec![5888, 5901, 5902, 5909]).unwrap(),
        };
        exp_uni_props_set.insert(bpt_n);

        // partial assertion
        let exp_uni_props_names_set: HashSet<Cow<str>> =
            exp_uni_props_set.iter().map(|p| p.name.clone()).collect();
        let act_uni_props_names_set: HashSet<Cow<str>> =
            act_uni_props_set.iter().map(|p| p.name.clone()).collect();
        let names_diff = act_uni_props_names_set.difference(&exp_uni_props_names_set);
        let names_diff_2 = exp_uni_props_names_set.difference(&act_uni_props_names_set);
        let names_diff_str = names_diff
            .into_iter()
            .cloned()
            .collect::<Vec<Cow<str>>>()
            .join(", ");
        let names_diff_str_2 = names_diff_2
            .into_iter()
            .cloned()
            .collect::<Vec<Cow<str>>>()
            .join(", ");
        assert_eq!(
            exp_uni_props_names_set, act_uni_props_names_set,
            "**** prop names missing in exp but in act = {}
            **** prop names missing in act but in exp = {}",
            names_diff_str, names_diff_str_2,
        );

        // full assertion
        let diff = act_uni_props_set.difference(&exp_uni_props_set);
        let diff_str = diff
            .into_iter()
            .map(|up| format!("{:?}", up))
            .collect::<Vec<String>>()
            .join(", ");
        assert_eq!(
            exp_uni_props_set, act_uni_props_set,
            "**** UnicodePropertyV1 values missing in exp but in act = {}",
            diff_str
        );
    }
}
