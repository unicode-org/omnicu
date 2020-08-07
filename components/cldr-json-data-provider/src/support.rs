use crate::CldrPaths;
use icu_data_provider::iter::DataEntryCollection;
use icu_data_provider::prelude::*;
use std::convert::TryFrom;
use std::sync::RwLock;

pub(crate) trait DataKeySupport {
    fn supports_key(data_key: &DataKey) -> Result<(), DataError>;
}

pub(crate) struct LazyCldrProvider<T> {
    data_provider: RwLock<Option<T>>,
}

impl<'b, 'd, T> LazyCldrProvider<T>
where
    T: DataProvider<'d> + DataKeySupport + DataEntryCollection + TryFrom<&'b CldrPaths>,
    <T as TryFrom<&'b CldrPaths>>::Error: 'static + std::error::Error,
{
    pub fn new() -> Self {
        Self {
            data_provider: RwLock::new(None),
        }
    }

    pub fn try_load(
        &self,
        req: &DataRequest,
        cldr_paths: &'b CldrPaths,
    ) -> Result<Option<DataResponse<'d>>, DataError> {
        if T::supports_key(&req.data_key).is_err() {
            return Ok(None);
        }
        if self.data_provider.read().unwrap().is_none() {
            self.data_provider.write().unwrap().replace(
                T::try_from(cldr_paths).map_err(|e| DataError::ResourceError(Box::new(e)))?,
            );
        };
        self.data_provider
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .load(req)
            .map(Some)
    }

    pub fn try_iter(
        &self,
        data_key: &DataKey,
        cldr_paths: &'b CldrPaths,
    ) -> Result<Option<Box<dyn Iterator<Item = DataEntry>>>, DataError> {
        if T::supports_key(data_key).is_err() {
            return Ok(None);
        }
        if self.data_provider.read().unwrap().is_none() {
            self.data_provider.write().unwrap().replace(
                T::try_from(cldr_paths).map_err(|e| DataError::ResourceError(Box::new(e)))?,
            );
        };
        self.data_provider
            .read()
            .unwrap()
            .as_ref()
            .unwrap()
            .iter_for_key(data_key)
            .map(Some)
    }
}
