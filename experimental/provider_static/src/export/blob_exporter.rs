// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use icu_provider::export::DataExporter;
use icu_provider::prelude::*;
use icu_provider::serde::SerdeSeDataStructMarker;
use crate::path_util;
use litemap::LiteMap;
use crate::blob_schema::*;

pub struct BlobExporter<'w> {
    resources: LiteMap<String, Vec<u8>>,
    sink: &'w mut dyn std::io::Write,
}

impl<'w> BlobExporter<'w> {
    pub fn new_with_sink(sink: &'w mut dyn std::io::Write) -> Self {
        Self {
            resources: LiteMap::new(),
            sink
        }
    }
}

/// TODO(#837): De-duplicate this code from icu_provider_fs.
fn serialize(
    obj: &dyn erased_serde::Serialize,
    mut sink: &mut (impl std::io::Write + ?Sized),
) -> Result<(), DataError> {
    use bincode::Options;
    obj.erased_serialize(&mut <dyn erased_serde::Serializer>::erase(
        &mut bincode::Serializer::new(
            &mut sink,
            bincode::config::DefaultOptions::new().with_fixint_encoding(),
        ),
    ))?;
    Ok(())
}

impl<'d, 's: 'd, 'w> DataExporter<'d, 's, SerdeSeDataStructMarker> for BlobExporter<'w> {
    fn put_payload(
        &mut self,
        req: DataRequest,
        obj: DataPayload<'d, 's, SerdeSeDataStructMarker>,
    ) -> Result<(), DataError> {
        let path = path_util::resource_path_to_string(&req.resource_path);
        log::trace!("Adding: {}", path);
        let mut buffer: Vec<u8> = Vec::new();
        serialize(obj.get().as_serialize(), &mut buffer)?;
        self.resources.insert(path, buffer);
        Ok(())
    }

    fn close(&mut self) -> Result<(), DataError> {
        // Convert from LiteMap<String, Vec> to LiteMap<&str, &[]>
        let mut schema = BlobSchemaV1 {
            resources: LiteMap::with_capacity(self.resources.len())
        };
        for (k, v) in self.resources.iter() {
            schema.resources.try_append(k, v).expect("Same order");
        }
        let blob = BlobSchema::V001(schema);
        serialize(&blob, self.sink)?;
        Ok(())
    }
}
