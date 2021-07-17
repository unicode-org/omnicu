// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

#[cfg_attr(any(feature = "serde", test), derive(serde::Deserialize))]
pub struct UnicodeEnumeratedProperty {
    pub code_point_map: EnumPropCodePointMap,
    pub code_point_trie: EnumPropSerializedCPT,
}

#[cfg_attr(any(feature = "serde", test), derive(serde::Deserialize))]
pub struct EnumPropCodePointMap {
    pub data: EnumPropCodePointMapData,
}

#[cfg_attr(any(feature = "serde", test), derive(serde::Deserialize))]
pub struct EnumPropCodePointMapData {
    pub long_name: String,
    pub name: String,
    pub ranges: Vec<(u32, u32, u32)>,
}

#[allow(clippy::upper_case_acronyms)]
#[cfg_attr(any(feature = "serde", test), derive(serde::Deserialize))]
pub struct EnumPropSerializedCPT {
    #[cfg_attr(any(feature = "serde", test), serde(rename = "struct"))]
    pub trie_struct: EnumPropSerializedCPTStruct,
}

#[allow(clippy::upper_case_acronyms)]
#[cfg_attr(any(feature = "serde", test), derive(serde::Deserialize))]
pub struct EnumPropSerializedCPTStruct {
    #[cfg_attr(any(feature = "serde", test), serde(skip))]
    pub long_name: String,
    pub name: String,
    pub index: Vec<u16>,
    pub data_8: Option<Vec<u8>>,
    pub data_16: Option<Vec<u16>>,
    pub data_32: Option<Vec<u32>>,
    #[cfg_attr(any(feature = "serde", test), serde(rename = "indexLength"))]
    pub index_length: u32,
    #[cfg_attr(any(feature = "serde", test), serde(rename = "dataLength"))]
    pub data_length: u32,
    #[cfg_attr(any(feature = "serde", test), serde(rename = "highStart"))]
    pub high_start: u32,
    #[cfg_attr(any(feature = "serde", test), serde(rename = "shifted12HighStart"))]
    pub shifted12_high_start: u16,
    #[cfg_attr(any(feature = "serde", test), serde(rename = "type"))]
    pub trie_type_enum_val: u8,
    #[cfg_attr(any(feature = "serde", test), serde(rename = "valueWidth"))]
    pub value_width_enum_val: u8,
    #[cfg_attr(any(feature = "serde", test), serde(rename = "index3NullOffset"))]
    pub index3_null_offset: u16,
    #[cfg_attr(any(feature = "serde", test), serde(rename = "dataNullOffset"))]
    pub data_null_offset: u32,
    #[cfg_attr(any(feature = "serde", test), serde(rename = "nullValue"))]
    pub null_value: u32,
}
