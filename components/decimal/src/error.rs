// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

//! Error types for decimal formatting.

use displaydoc::Display;

#[derive(Display, Debug)]
pub enum Error {
    #[displaydoc("error loading data: {0}")]
    Data(#[from] icu_provider::DataError),
}
