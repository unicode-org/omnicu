use crate::manifest::Manifest;
use crate::Error;
use icu_data_provider::prelude::*;
use icu_data_provider::structs;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub struct FsDataProvider {
    root: PathBuf,
    manifest: Manifest,
}

impl FsDataProvider {
    pub fn try_new(root: PathBuf) -> Result<Self, Error> {
        let manifest_path = root.clone().join("manifest.json");
        let file = File::open(&manifest_path)?;
        let reader = BufReader::new(file);
        let manifest: Manifest = serde_json::from_reader(reader)?;
        Ok(Self { root, manifest })
    }
}

impl DataProvider<'_> for FsDataProvider {
    fn load(
        &self,
        req: &data_provider::Request,
    ) -> Result<data_provider::Response<'static>, data_provider::Error> {
        type Error = data_provider::Error;
        let mut path_buf = self.root.clone();
        path_buf.extend(req.data_key.get_components().iter());
        if !path_buf.exists() {
            path_buf.pop();
            if !path_buf.exists() {
                return Err(Error::UnsupportedCategory(req.data_key.category));
            } else {
                return Err(Error::UnsupportedDataKey(req.data_key));
            }
        }
        // TODO: Implement proper locale fallback
        path_buf.extend(req.data_entry.get_components().iter());
        path_buf.set_extension(self.manifest.syntax.get_file_extension());
        if !path_buf.exists() {
            return Err(Error::UnavailableEntry(req.clone()));
        }
        let file = match File::open(&path_buf) {
            Ok(file) => file,
            Err(err) => return Err(Error::ResourceError(Box::new(err))),
        };
        let reader = BufReader::new(file);
        // TODO: Eliminate this dispatch.
        // https://github.com/unicode-org/icu4x/issues/196
        if req.data_key.category == data_key::Category::Plurals {
            let obj: structs::plurals::PluralRuleStringsV1 = match serde_json::from_reader(reader) {
                Ok(obj) => obj,
                Err(err) => return Err(Error::ResourceError(Box::new(err))),
            };
            let response = data_provider::ResponseBuilder {
                // TODO: Return the actual locale when fallbacks are implemented.
                data_langid: req.data_entry.langid.clone(),
            }
            .with_owned_payload(obj);
            Ok(response)
        } else {
            panic!("Don't know how to parse this data key, but it is on the filesystem");
        }
    }
}
