use crate::cldr_langid::CldrLanguage;
use crate::error::Error;
use crate::reader::open_reader;
use crate::support::DataKeySupport;
use crate::CldrPaths;
use icu_data_provider::iter::DataEntryCollection;
use icu_data_provider::prelude::*;
use icu_data_provider::structs::plurals::*;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::marker::PhantomData;

/// A data provider reading from CLDR JSON plural rule files.
#[derive(PartialEq, Debug)]
pub struct PluralsProvider<'d> {
    cardinal_rules: Option<cldr_json::Rules>,
    ordinal_rules: Option<cldr_json::Rules>,
    _phantom: PhantomData<&'d ()>, // placeholder for when we need the lifetime param
}

impl TryFrom<&CldrPaths> for PluralsProvider<'_> {
    type Error = Error;
    fn try_from(cldr_paths: &CldrPaths) -> Result<Self, Self::Error> {
        let cardinal_rules = {
            let path = cldr_paths
                .cldr_core
                .clone()?
                .join("supplemental")
                .join("plurals.json");
            let data: cldr_json::Resource = serde_json::from_reader(open_reader(path)?)?;
            data.supplemental
                .plurals_type_cardinal
                .map(|r| r.normalize())
        };
        let ordinal_rules = {
            let path = cldr_paths
                .cldr_core
                .clone()?
                .join("supplemental")
                .join("ordinals.json");
            let data: cldr_json::Resource = serde_json::from_reader(open_reader(path)?)?;
            data.supplemental
                .plurals_type_ordinal
                .map(|r| r.normalize())
        };
        Ok(PluralsProvider {
            cardinal_rules,
            ordinal_rules,
            _phantom: PhantomData,
        })
    }
}

impl<'d> TryFrom<&'d str> for PluralsProvider<'d> {
    type Error = serde_json::error::Error;
    /// Attempt to parse a JSON string.
    fn try_from(s: &'d str) -> Result<Self, Self::Error> {
        let data: cldr_json::Resource = serde_json::from_str(s)?;
        Ok(PluralsProvider {
            cardinal_rules: data
                .supplemental
                .plurals_type_cardinal
                .map(|r| r.normalize()),
            ordinal_rules: data
                .supplemental
                .plurals_type_ordinal
                .map(|r| r.normalize()),
            _phantom: PhantomData,
        })
    }
}

impl<'d> DataKeySupport for PluralsProvider<'d> {
    fn supports_key(data_key: &DataKey) -> Result<(), DataError> {
        if data_key.category != DataCategory::Plurals {
            return Err((&data_key.category).into());
        }
        if data_key.version != 1 {
            return Err(data_key.into());
        }
        Ok(())
    }
}

impl<'d> PluralsProvider<'d> {
    fn get_rules_for(&self, data_key: &DataKey) -> Result<&cldr_json::Rules, DataError> {
        PluralsProvider::supports_key(data_key)?;
        match data_key.sub_category.as_str() {
            "cardinal" => self.cardinal_rules.as_ref(),
            "ordinal" => self.ordinal_rules.as_ref(),
            _ => return Err(data_key.into()),
        }
        .ok_or_else(|| data_key.into())
    }
}

impl<'d> DataProvider<'d> for PluralsProvider<'d> {
    fn load(&self, req: &DataRequest) -> Result<DataResponse<'d>, DataError> {
        let cldr_rules = self.get_rules_for(&req.data_key)?;
        // TODO: Implement language fallback?
        // TODO: Avoid the clone
        let cldr_langid = CldrLanguage(req.data_entry.langid.clone());
        let (_, r) = match cldr_rules.0.binary_search_by_key(&&cldr_langid, |(l, _)| l) {
            Ok(idx) => &cldr_rules.0[idx],
            Err(_) => return Err(req.clone().into()),
        };
        Ok(DataResponseBuilder {
            data_langid: req.data_entry.langid.clone(),
        }
        .with_owned_payload(PluralRuleStringsV1::from(r)))
    }
}

impl<'d> DataEntryCollection for PluralsProvider<'d> {
    fn iter_for_key(
        &self,
        data_key: &DataKey,
    ) -> Result<Box<dyn Iterator<Item = DataEntry>>, DataError> {
        let cldr_rules = self.get_rules_for(data_key)?;
        let list: Vec<DataEntry> = cldr_rules
            .0
            .iter()
            .map(|(l, _)| DataEntry {
                variant: None,
                // TODO: Avoid the clone
                langid: l.0.clone(),
            })
            .collect();
        Ok(Box::new(list.into_iter()))
    }
}

impl From<&cldr_json::LocalePluralRules> for PluralRuleStringsV1 {
    fn from(other: &cldr_json::LocalePluralRules) -> PluralRuleStringsV1 {
        #[allow(clippy::ptr_arg)]
        fn convert(s: &Cow<'static, str>) -> Cow<'static, str> {
            s.clone()
        }
        PluralRuleStringsV1 {
            zero: other.zero.as_ref().map(convert),
            one: other.one.as_ref().map(convert),
            two: other.two.as_ref().map(convert),
            few: other.few.as_ref().map(convert),
            many: other.many.as_ref().map(convert),
        }
    }
}

/// Serde structs for the CLDR JSON plurals files.
pub(self) mod cldr_json {
    use crate::cldr_langid::CldrLanguage;
    use serde::Deserialize;
    use std::borrow::Cow;

    // TODO: Use Serde Borrow throughout these structs. Blocked by:
    // https://stackoverflow.com/q/63201624/1407170

    #[derive(PartialEq, Debug, Deserialize)]
    pub struct LocalePluralRules {
        #[serde(rename = "pluralRule-count-zero")]
        pub zero: Option<Cow<'static, str>>,
        #[serde(rename = "pluralRule-count-one")]
        pub one: Option<Cow<'static, str>>,
        #[serde(rename = "pluralRule-count-two")]
        pub two: Option<Cow<'static, str>>,
        #[serde(rename = "pluralRule-count-few")]
        pub few: Option<Cow<'static, str>>,
        #[serde(rename = "pluralRule-count-many")]
        pub many: Option<Cow<'static, str>>,
    }

    #[derive(PartialEq, Debug, Deserialize)]
    pub struct Rules(#[serde(with = "tuple_vec_map")] pub Vec<(CldrLanguage, LocalePluralRules)>);

    impl Rules {
        pub fn normalize(mut self) -> Self {
            // NOTE: We need to sort in order to put "root" -> "und" into place.
            // TODO: Avoid the clone
            self.0.sort_by_key(|(l, _)| l.0.clone());
            self
        }
    }

    #[derive(PartialEq, Debug, Deserialize)]
    pub struct Supplemental {
        #[serde(rename = "plurals-type-cardinal")]
        pub plurals_type_cardinal: Option<Rules>,
        #[serde(rename = "plurals-type-ordinal")]
        pub plurals_type_ordinal: Option<Rules>,
    }

    #[derive(PartialEq, Debug, Deserialize)]
    pub struct Resource {
        pub supplemental: Supplemental,
    }
}

#[test]
fn test_basic() {
    use std::borrow::Borrow;

    let json_str = std::fs::read_to_string("tests/testdata/plurals.json").unwrap();
    let provider = PluralsProvider::try_from(json_str.as_str()).unwrap();

    // Spot-check locale 'cs' since it has some interesting entries
    let cs_rules: Cow<PluralRuleStringsV1> = provider
        .load(&DataRequest {
            data_key: icu_data_key!(plurals: cardinal@1),
            data_entry: DataEntry {
                variant: None,
                langid: "cs".parse().unwrap(),
            },
        })
        .unwrap()
        .take_payload()
        .unwrap();

    assert_eq!(None, cs_rules.zero);
    assert_eq!(
        Some("i = 1 and v = 0 @integer 1"),
        cs_rules.one.as_ref().map(|v| v.borrow())
    );
    assert_eq!(None, cs_rules.two);
    assert_eq!(
        Some("i = 2..4 and v = 0 @integer 2~4"),
        cs_rules.few.as_ref().map(|v| v.borrow())
    );
    assert_eq!(
        Some("v != 0   @decimal 0.0~1.5, 10.0, 100.0, 1000.0, 10000.0, 100000.0, 1000000.0, …"),
        cs_rules.many.as_ref().map(|v| v.borrow())
    );
}
