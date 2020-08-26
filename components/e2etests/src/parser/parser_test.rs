use super::*;

#[test]
fn locale_test_output_deser() {
    let data = r#"
{"lang": "en",
 "region": "US",
 "subtags": [["u", "hc", "buddhist"]]}
"#;

    let act_test_output: LocaleTestOutput =
        serde_json::from_str(data).expect("cannot parse sample LocaleTestOutput");

    let lang = String::from("en");
    let region = String::from("US");
    let subtags = vec![vec![
        String::from("u"),
        String::from("hc"),
        String::from("buddhist"),
    ]];
    let exp_test_output = LocaleTestOutput {
        lang,
        region,
        subtags,
    };

    assert_eq!(act_test_output, exp_test_output);
}

#[test]
fn locale_test_data_deser() {
    let data = r#"
{"input": "en-US-u-hc-buddhist",
 "output": {"lang": "en",
            "region": "US",
            "subtags": [["u", "hc", "buddhist"]]}}
"#;

    let act_test_data: LocaleTestData =
        serde_json::from_str(data).expect("cannot parse sample LocaleTestData");

    let input = String::from("en-US-u-hc-buddhist");
    let lang = String::from("en");
    let region = String::from("US");
    let subtags = vec![vec![
        String::from("u"),
        String::from("hc"),
        String::from("buddhist"),
    ]];
    let output = Some(LocaleTestOutput {
        lang,
        region,
        subtags,
    });
    let exp_test_data = LocaleTestData { input, output };

    assert_eq!(act_test_data, exp_test_data);
}

#[test]
fn test_data_deser() {
    let data = r#"
{"locale_test_data": {"input": "en-US-u-hc-buddhist",
                      "output": {"lang": "en",
                                 "region": "US",
                                 "subtags": [["u", "hc", "buddhist"]]}}}
"#;

    let act_test_data: TestData = serde_json::from_str(data).expect("cannot parse sample TestData");

    let input = String::from("en-US-u-hc-buddhist");
    let lang = String::from("en");
    let region = String::from("US");
    let subtags = vec![vec![
        String::from("u"),
        String::from("hc"),
        String::from("buddhist"),
    ]];
    let output = Some(LocaleTestOutput {
        lang,
        region,
        subtags,
    });
    let locale_test_data = LocaleTestData { input, output };
    let exp_test_data = TestData { locale_test_data };

    assert_eq!(act_test_data, exp_test_data);
}

#[test]
fn test_base_data_deser() {
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

    let input = String::from("en-US-u-hc-buddhist");
    let lang = String::from("en");
    let region = String::from("US");
    let subtags = vec![vec![
        String::from("u"),
        String::from("hc"),
        String::from("buddhist"),
    ]];
    let output = Some(LocaleTestOutput {
        lang,
        region,
        subtags,
    });
    let locale_test_data = LocaleTestData { input, output };
    let test_data1 = TestData { locale_test_data };
    let test_name = String::from("locale_parser_test");
    let test_feature = String::from("locale.parser");
    let op = String::from("equal");
    let msg = String::from("Parsed locale mismatched expected");
    let exp_test_base = TestBase {
        test_name,
        test_feature,
        op,
        msg,
        test_data: vec![test_data1],
    };

    assert_eq!(act_test_base, exp_test_base);
}
