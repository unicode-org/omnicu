use super::*;

#[test]
fn locale_test() {
    let data = r#"
{"test_name": "locale_parser_test",
 "test_feature": "locale.parser",
 "op": "equal",
 "msg": "Parsed locale mismatched expected",
 "test_data": [
   {"locale_test_data": {"input": "en-US-u-hc-buddhist",
                         "output": {"lang": "en",
                                    "region": "US",
                                    "subtags": [["u", "hc", "buddhist"]]}}}]}
"#;

    let act_test_base: TestBase = serde_json::from_str(data).expect("cannot parse sample TestBase");
    let act_test_cases_data: Vec<TestData> = act_test_base.test_data;

    assert_eq!(act_test_cases_data.len(), 1);

    let act_test_data = &act_test_cases_data[0];
    let act_locale_test_data: &LocaleTestData = &act_test_data.locale_test_data;

    run_locale_test(&act_locale_test_data);
}
