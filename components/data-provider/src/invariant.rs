use crate::error::Error;
use crate::iter::DataEntryCollection;
use crate::prelude::*;
use crate::structs;
use icu_locale::LanguageIdentifier;
use std::fmt;

pub(crate) fn make_inv_response<T: 'static + Clone + erased_serde::Serialize + fmt::Debug>(
    t: T,
) -> data_provider::Response<'static> {
    data_provider::ResponseBuilder {
        data_langid: LanguageIdentifier::default(),
    }
    .with_owned_payload(t)
}

/// A locale-invariant data provider. Sometimes useful for testing. Not intended to be used in
/// production environments.
pub struct InvariantDataProvider;

impl DataProvider<'static> for InvariantDataProvider {
    fn load<'a>(
        &'a self,
        req: &data_provider::Request,
    ) -> Result<data_provider::Response<'static>, Error> {
        structs::get_invariant(&req.data_key).ok_or(Error::UnsupportedDataKey(req.data_key))
    }
}

impl DataEntryCollection for InvariantDataProvider {
    fn iter_for_key(
        &self,
        data_key: &DataKey,
    ) -> Result<Box<dyn Iterator<Item = DataEntry>>, Error> {
        structs::get_invariant(data_key).ok_or(Error::UnsupportedDataKey(*data_key))?;
        let list: Vec<DataEntry> = vec![DataEntry {
            variant: None,
            langid: LanguageIdentifier::default(),
        }];
        Ok(Box::new(list.into_iter()))
    }
}

#[test]
fn test_basic() {
    let provider = InvariantDataProvider;
    let response = provider
        .load(&data_provider::Request {
            data_key: icu_data_key!(plurals: cardinal@1),
            data_entry: DataEntry {
                variant: None,
                langid: LanguageIdentifier::default(),
            },
        })
        .unwrap();
    let plurals_data: &structs::plurals::PluralRuleStringsV1 = response.borrow_payload().unwrap();
    assert_eq!(
        plurals_data,
        &structs::plurals::PluralRuleStringsV1 {
            zero: None,
            one: None,
            two: None,
            few: None,
            many: None,
        }
    );
}
