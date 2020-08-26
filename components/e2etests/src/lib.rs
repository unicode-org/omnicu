use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum TestOp {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    EqualsAnyOrder,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TestBase {
    test_name: String,
    test_feature: String,
    // might need to customize Serde to create Enums
    //op: TestOp,
    op: String,
    msg: String,
    test_data: Vec<TestData>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
// need to see how to make this behave like Protobuf "oneof".
// maybe trait (interface) and derive?
pub struct TestData {
    locale_test_data: LocaleTestData,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LocaleTestData {
    input: String,
    output: Option<LocaleTestOutput>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LocaleTestOutput {
    lang: String,
    region: String,
    subtags: Vec<Vec<String>>,
}

//
// modules
//

#[path = "./runner/runner.rs"]
mod runner;

//
// tests
//

#[cfg(test)]
#[path = "./parser/parser_test.rs"]
mod parser_tests;
