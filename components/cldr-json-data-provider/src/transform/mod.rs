mod plurals;

pub use plurals::PluralsProvider;

use crate::support::LazyCldrProvider;
use crate::CldrPaths;
use icu_data_provider::data_provider::DataProvider;
use icu_data_provider::iter::DataEntryCollection;
use icu_data_provider::prelude::*;

pub struct CldrJsonDataProvider<'a, 'd> {
    pub cldr_paths: &'a CldrPaths,
    plurals: LazyCldrProvider<PluralsProvider<'d>>,
}

impl<'a, 'd> CldrJsonDataProvider<'a, 'd> {
    pub fn new(cldr_paths: &'a CldrPaths) -> Self {
        CldrJsonDataProvider {
            cldr_paths,
            plurals: LazyCldrProvider::new(),
        }
    }
}

impl<'a, 'd> DataProvider<'d> for CldrJsonDataProvider<'a, 'd> {
    fn load(
        &self,
        req: &data_provider::Request,
    ) -> Result<data_provider::Response<'d>, data_provider::Error> {
        if let Some(resp) = self.plurals.try_load(req, &self.cldr_paths)? {
            return Ok(resp);
        }
        Err(data_provider::Error::UnsupportedDataKey(req.data_key))
    }
}

impl<'a, 'd> DataEntryCollection for CldrJsonDataProvider<'a, 'd> {
    fn iter_for_key(
        &self,
        data_key: &DataKey,
    ) -> Result<Box<dyn Iterator<Item = DataEntry>>, data_provider::Error> {
        if let Some(resp) = self.plurals.try_iter(data_key, &self.cldr_paths)? {
            return Ok(resp);
        }
        Err(data_provider::Error::UnsupportedDataKey(*data_key))
    }
}
