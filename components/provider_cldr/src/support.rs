// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/master/LICENSE ).
use crate::CldrPaths;
use icu_provider::iter::DataEntryCollection;
use icu_provider::prelude::*;
use std::convert::TryFrom;
use std::sync::RwLock;

pub trait DataKeySupport {
    fn supports_key(data_key: &DataKey) -> Result<(), DataError>;
}

#[derive(Debug)]
pub struct LazyCldrProvider<T> {
    src: RwLock<Option<T>>,
}

impl<T> Default for LazyCldrProvider<T> {
    fn default() -> Self {
        Self {
            src: RwLock::new(None),
        }
    }
}

fn map_poison<E>(_err: E) -> DataError {
    // Can't return the Poison directly because it has lifetime parameters.
    DataError::new_resc_error(crate::error::Error::Poison)
}

/// A lazy-initialized CLDR JSON data provider.
impl<'b, 'd, T> LazyCldrProvider<T>
where
    T: DataProviderV2<'d> + DataKeySupport + DataEntryCollection + TryFrom<&'b dyn CldrPaths>,
    <T as TryFrom<&'b dyn CldrPaths>>::Error: 'static + std::error::Error,
{
    /// Call `T::load`, initializing T if necessary.
    pub fn try_load(
        &self,
        req: &DataRequest,
        receiver: &mut dyn DataReceiver<'d, 'static>,
        cldr_paths: &'b dyn CldrPaths,
    ) -> Result<Option<DataResponseV2>, DataError> {
        if T::supports_key(&req.data_key).is_err() {
            return Ok(None);
        }
        if let Some(data_provider) = self.src.read().map_err(map_poison)?.as_ref() {
            return data_provider.load_v2(req, receiver).map(Some);
        }
        let mut src = self.src.write().map_err(map_poison)?;
        if src.is_none() {
            src.replace(T::try_from(cldr_paths).map_err(DataError::new_resc_error)?);
        }
        let data_provider = src
            .as_ref()
            .expect("The RwLock must be populated at this point.");
        data_provider.load_v2(req, receiver).map(Some)
    }

    /// Call `T::iter_for_key`, initializing T if necessary.
    pub fn try_iter(
        &self,
        data_key: &DataKey,
        cldr_paths: &'b dyn CldrPaths,
    ) -> Result<Option<Box<dyn Iterator<Item = DataEntry>>>, DataError> {
        if T::supports_key(data_key).is_err() {
            return Ok(None);
        }
        if let Some(data_provider) = self.src.read().map_err(map_poison)?.as_ref() {
            return data_provider.iter_for_key(data_key).map(Some);
        }
        let mut src = self.src.write().map_err(map_poison)?;
        if src.is_none() {
            src.replace(T::try_from(cldr_paths).map_err(DataError::new_resc_error)?);
        }
        let data_provider = src
            .as_ref()
            .expect("The RwLock must be populated at this point.");
        data_provider.iter_for_key(data_key).map(Some)
    }
}
