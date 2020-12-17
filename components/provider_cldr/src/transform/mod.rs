// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/master/LICENSE ).
mod dates;
mod plurals;

pub use dates::DatesProvider;
pub use plurals::PluralsProvider;

use crate::support::LazyCldrProvider;
use crate::CldrPaths;
use icu_provider::iter::DataEntryCollection;
use icu_provider::prelude::*;

/// Returns a list of all DataKeys that this provider can produce.
pub fn get_all_data_keys() -> Vec<DataKey> {
    let mut result: Vec<DataKey> = vec![];
    result.extend(&plurals::ALL_KEYS);
    result.extend(&dates::ALL_KEYS);
    result
}

#[derive(Debug)]
pub struct CldrJsonDataProvider<'a, 'd> {
    pub cldr_paths: &'a dyn CldrPaths,
    plurals: LazyCldrProvider<PluralsProvider<'d>>,
    dates: LazyCldrProvider<DatesProvider<'d>>,
}

impl<'a, 'd> CldrJsonDataProvider<'a, 'd> {
    pub fn new(cldr_paths: &'a dyn CldrPaths) -> Self {
        CldrJsonDataProvider {
            cldr_paths,
            plurals: Default::default(),
            dates: Default::default(),
        }
    }
}

impl<'a, 'd> DataProviderV2<'d> for CldrJsonDataProvider<'a, 'd> {
    fn load_v2(
        &self,
        req: &DataRequest,
        receiver: &mut dyn DataReceiver<'d, 'static>,
    ) -> Result<DataResponseV2, DataError> {
        if let Some(result) = self.plurals.try_load(req, receiver, self.cldr_paths)? {
            return Ok(result);
        }
        if let Some(result) = self.dates.try_load(req, receiver, self.cldr_paths)? {
            return Ok(result);
        }
        Err(DataError::UnsupportedDataKey(req.data_key))
    }
}

impl<'a, 'd> DataEntryCollection for CldrJsonDataProvider<'a, 'd> {
    fn iter_for_key(
        &self,
        data_key: &DataKey,
    ) -> Result<Box<dyn Iterator<Item = DataEntry>>, DataError> {
        if let Some(resp) = self.plurals.try_iter(data_key, self.cldr_paths)? {
            return Ok(resp);
        }
        if let Some(resp) = self.dates.try_iter(data_key, self.cldr_paths)? {
            return Ok(resp);
        }
        Err(DataError::UnsupportedDataKey(*data_key))
    }
}
