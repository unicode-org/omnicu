use std::borrow::Cow;
use std::prelude::v1::*;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use icu_locale::LanguageIdentifier;

use icu_data_provider::icu_data_key;
use icu_data_provider::prelude::*;
use icu_data_provider::validator::DataProviderValidator;
use icu_data_provider::structs;
use icu_data_provider::error::Error;

// This file tests DataProvider borrow semantics with a dummy data provider based on a JSON string.

#[derive(Serialize, Deserialize, Debug)]
struct DecimalJsonSchema {
    pub symbols_v1_a: structs::decimal::SymbolsV1,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonSchema {
    pub decimal: DecimalJsonSchema,
}

#[derive(Debug)]
struct JsonDataWarehouse {
    data: JsonSchema,
}

/// A data provider reading from a JSON file.
#[derive(Debug)]
struct JsonDataProvider<'d> {
    borrowed_data: &'d JsonSchema,
}

impl JsonDataWarehouse {
    pub fn provider(&self) -> JsonDataProvider {
        self.into()
    }
}

impl FromStr for JsonDataWarehouse {
    type Err = Error;

    /// Create a JsonDataProvider from a JSON string slice.
    fn from_str(data: &str) -> Result<Self, Error> {
        let data: JsonSchema = match serde_json::from_str(data) {
            Ok(data) => data,
            Err(err) => return Err(Error::ResourceError(Box::new(err))),
        };
        Ok(Self { data })
    }
}

impl<'d> From<&'d JsonDataWarehouse> for JsonDataProvider<'d> {
    fn from(warehouse: &'d JsonDataWarehouse) -> JsonDataProvider<'d> {
        JsonDataProvider {
            borrowed_data: &warehouse.data,
        }
    }
}

impl<'d> DataProvider<'d> for JsonDataProvider<'d> {
    /// Loads JSON data. Returns borrowed data.
    fn load(
        &self,
        _request: &data_provider::Request,
    ) -> Result<data_provider::Response<'d>, data_provider::Error> {
        let response = data_provider::ResponseBuilder {
            data_langid: LanguageIdentifier::default(),
        }
        .with_borrowed_payload(&self.borrowed_data.decimal.symbols_v1_a);
        Ok(response)
    }
}

#[allow(clippy::redundant_static_lifetimes)]
const DATA: &'static str = r#"{
    "decimal": {
        "symbols_v1_a": {
            "zero_digit": "0",
            "decimal_separator": ".",
            "grouping_separator": ","
        }
    }
}"#;

fn get_warehouse() -> JsonDataWarehouse {
    JsonDataWarehouse::from_str(DATA).unwrap()
}

fn get_response(warehouse: &JsonDataWarehouse) -> data_provider::Response {
    warehouse
        .provider()
        .load(&data_provider::Request {
            data_key: icu_data_key!(decimal: symbols@1),
            data_entry: DataEntry {
                variant: None,
                langid: "en-US".parse().unwrap(),
            },
        })
        .unwrap()
}

fn get_response_with_validator(warehouse: &JsonDataWarehouse) -> data_provider::Response {
    let validation_provider = DataProviderValidator {
        data_provider: &warehouse.provider(),
    };
    validation_provider
        .load(&data_provider::Request {
            data_key: icu_data_key!(decimal: symbols@1),
            data_entry: DataEntry {
                variant: None,
                langid: "en-US".parse().unwrap(),
            },
        })
        .unwrap()
}

fn check_data(decimal_data: &structs::decimal::SymbolsV1) {
    assert_eq!(
        decimal_data,
        &structs::decimal::SymbolsV1 {
            zero_digit: '0',
            decimal_separator: ".".into(),
            grouping_separator: ",".into(),
        }
    );
}

#[test]
fn test_read_string() {
    let warehouse = get_warehouse();
    let response = get_response(&warehouse);
    let decimal_data: &structs::decimal::SymbolsV1 = response.borrow_payload().unwrap();
    check_data(decimal_data);
}

#[test]
fn test_validator() {
    let warehouse = get_warehouse();
    let response = get_response_with_validator(&warehouse);
    let decimal_data: &structs::decimal::SymbolsV1 = response.borrow_payload().unwrap();
    check_data(decimal_data);
}

#[test]
fn test_borrow_payload_mut() {
    let warehouse = get_warehouse();
    let mut response = get_response(&warehouse);
    let decimal_data: &mut structs::decimal::SymbolsV1 = response.borrow_payload_mut().unwrap();
    check_data(decimal_data);
}

#[test]
fn test_take_payload() {
    let warehouse = get_warehouse();
    let response = get_response(&warehouse);
    let decimal_data: Cow<structs::decimal::SymbolsV1> = response.take_payload().unwrap();
    check_data(&decimal_data);
}

#[test]
fn test_clone_payload() {
    let final_data = {
        let warehouse = get_warehouse();
        let response = get_response(&warehouse);
        let decimal_data: Cow<structs::decimal::SymbolsV1> = response.take_payload().unwrap();
        decimal_data.into_owned()
    };
    check_data(&final_data);
}
