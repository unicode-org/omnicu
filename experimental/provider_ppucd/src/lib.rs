// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

mod error;
pub mod parse_ppucd;
pub mod support;

pub use support::PpucdDataProvider;

/// Returns a list of all keys supported by PPUCD.
pub fn get_all_ppucd_keys() -> &'static [icu_provider::ResourceKey] {
    &icu_uniset::provider::key::ALL_KEYS
}
