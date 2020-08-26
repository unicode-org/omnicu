use super::*;

use icu_locale::extensions::unicode::Key;
use icu_locale::extensions::unicode::Value;
use icu_locale::extensions::Extensions;
use icu_locale::extensions::Unicode;
use icu_locale::subtags::Region;
use icu_locale::subtags::Variant;
use icu_locale::Locale;

fn concat_unicode_subtags(variants: &mut Vec<Vec<String>>, unicode_exts: &Unicode) {
    match unicode_exts {
        Unicode { keywords, .. } => {
            let key_vals: Vec<(Key, Value)> = keywords.to_vec();
            for (key, value) in key_vals {
                let key_str = String::from(key.as_str());
                let value_str: String = format!("{}", value);
                let flattened_subtag: Vec<String> = vec![String::from("u"), key_str, value_str];
                variants.push(flattened_subtag);
            }
        }
    }
}

impl From<&Locale> for LocaleTestOutput {
    fn from(input: &Locale) -> Self {
        let language = input.language;
        let _script = input.script;
        let variants = &input.variants;

        let region: Option<Region> = input.region;
        let region: &str = match &region {
            Some(r) => r.as_str(),
            None => "",
        };
        let region: String = String::from(region);

        let lang: String = String::from(language.as_str());

        let _script: String = match _script {
            Some(s) => String::from(s.as_str()),
            None => String::from(""),
        };

        let variants: Vec<Variant> = variants.to_vec();
        let variants: Vec<String> = variants
            .into_iter()
            .map(|v: Variant| String::from(v.as_str()))
            .collect::<Vec<String>>();

        // Create `subtags` as a "flattened" version of Variants. A nested
        // sequential Vec<Vec<_>> should help preserve order of subtags.
        let mut subtags: Vec<Vec<String>> = if variants.len() > 0 {
            vec![variants]
        } else {
            vec![]
        };
        if !input.extensions.is_empty() {
            match &input.extensions {
                Extensions { unicode, .. } => concat_unicode_subtags(&mut subtags, unicode),
            }
        }

        let locale_test_output = LocaleTestOutput {
            lang,
            region,
            subtags,
        };

        locale_test_output
    }
}

pub fn run_locale_test(locale_test_data: &LocaleTestData) {
    let input: &String = &locale_test_data.input;
    let output: &Option<LocaleTestOutput> = &locale_test_data.output;

    let actual_locale: Locale = input
        .parse()
        .expect("Failed to parse test input locale string");
    let actual_locale_test_output = LocaleTestOutput::from(&actual_locale);

    let expected_locale_test_output: &LocaleTestOutput = output
        .as_ref()
        .expect("Expected output data required for test");

    assert_eq!(&actual_locale_test_output, expected_locale_test_output);
}

//
// tests
//

#[cfg(test)]
#[path = "./runner_test.rs"]
mod runner_tests;
