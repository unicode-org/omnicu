// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use crate::blob_schema::BlobSchema;
use crate::path_util;
use icu_provider::{
    prelude::*,
    serde::{SerdeDeDataProvider, SerdeDeDataReceiver},
};
use serde::de::Deserialize;

/// A data provider loading data statically baked in to the binary. Useful for testing in situations
/// where setting up a filesystem is tricky (e.g. WASM).
///
/// This should probably not be used in production code since it bloats the binary.
pub struct StaticDataProvider {
    blob: BlobSchema<'static>,
}

/// TODO(#837): De-duplicate this code from icu_provider_fs.
macro_rules! get_bincode_deserializer_zc {
    ($bytes:tt) => {{
        use bincode::Options;
        let options = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .allow_trailing_bytes();
        bincode::de::Deserializer::from_slice($bytes, options)
    }};
}

impl StaticDataProvider {
    pub fn new_from_static_blob(blob: &'static [u8]) -> Result<Self, DataError> {
        Ok(StaticDataProvider {
            blob: BlobSchema::deserialize(&mut get_bincode_deserializer_zc!(blob))
                .map_err(DataError::new_resc_error)?,
        })
    }

    fn get_file(&self, req: &DataRequest) -> Result<&'static [u8], DataError> {
        let path = path_util::resource_path_to_string(&req.resource_path);
        let BlobSchema::V001(blob) = &self.blob;
        blob.resources
            .get(&*path)
            .ok_or(DataError::UnsupportedResourceKey(req.resource_path.key))
            .map(|v| *v)
    }
}

impl<'d, 's, M> DataProvider<'d, 's, M> for StaticDataProvider
where
    M: DataMarker<'s>,
    // 'static is what we want here, because we are deserializing from a static buffer.
    M::Yokeable: serde::de::Deserialize<'static>,
{
    fn load_payload(&self, req: &DataRequest) -> Result<DataResponse<'d, 's, M>, DataError> {
        let file = self.get_file(req)?;
        let data = M::Yokeable::deserialize(&mut get_bincode_deserializer_zc!(file))
            .map_err(DataError::new_resc_error)?;
        Ok(DataResponse {
            metadata: DataResponseMetadata {
                data_langid: req.resource_path.options.langid.clone(),
            },
            payload: Some(DataPayload::from_owned(data)),
        })
    }
}

impl SerdeDeDataProvider for StaticDataProvider {
    fn load_to_receiver(
        &self,
        req: &DataRequest,
        receiver: &mut dyn SerdeDeDataReceiver,
    ) -> Result<DataResponseMetadata, DataError> {
        let file = self.get_file(req)?;
        receiver.receive_static(&mut erased_serde::Deserializer::erase(
            &mut get_bincode_deserializer_zc!(file),
        ))?;

        Ok(DataResponseMetadata {
            data_langid: req.resource_path.options.langid.clone(),
        })
    }
}
