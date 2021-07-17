// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

mod test_util;

use icu_codepointtrie::codepointtrie::*;
use icu_codepointtrie::error::Error;
use icu_codepointtrie::planes::get_planes_trie;
use icu_codepointtrie::provider::UnicodeEnumeratedProperty;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zerovec::ZeroVec;

#[test]
fn planes_trie_deserialize_check_test() {
    // Get expected planes trie from crate::planes::get_planes_trie()

    let exp_planes_trie = get_planes_trie();

    // Compute actual planes trie from planes.toml

    let path = Path::new("tests/testdata/planes.toml");
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut toml_str = String::new();

    match file.read_to_string(&mut toml_str) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        _ => {}
    }

    let planes_enum_prop: UnicodeEnumeratedProperty = toml::from_str(&toml_str).unwrap();

    let code_point_trie_struct = planes_enum_prop.code_point_trie.trie_struct;

    let trie_header = CodePointTrieHeader {
        index_length: code_point_trie_struct.index_length,
        data_length: code_point_trie_struct.data_length,
        high_start: code_point_trie_struct.high_start,
        shifted12_high_start: code_point_trie_struct.shifted12_high_start,
        index3_null_offset: code_point_trie_struct.index3_null_offset,
        data_null_offset: code_point_trie_struct.data_null_offset,
        null_value: code_point_trie_struct.null_value,
    };

    let data = ZeroVec::from_aligned(&code_point_trie_struct.data_8.unwrap());
    let index = ZeroVec::from_aligned(&code_point_trie_struct.index);
    let trie_result: Result<CodePointTrie<u8, Small>, Error> =
        CodePointTrie::try_new(trie_header, index, data);
    let act_planes_trie = trie_result.unwrap();

    // Get check ranges (inversion map-style sequence of range+value) and
    // apply the trie validation test fn on expected and actual tries

    let serialized_ranges: Vec<(u32, u32, u32)> = planes_enum_prop.code_point_map.data.ranges;
    let mut check_ranges: Vec<u32> = vec![];
    for range_tuple in serialized_ranges {
        let range_end = range_tuple.1 + 1;
        let value = range_tuple.2;
        check_ranges.push(range_end);
        check_ranges.push(value);
    }

    test_util::check_trie(&act_planes_trie, &check_ranges);
    test_util::check_trie(&exp_planes_trie, &check_ranges);
}
