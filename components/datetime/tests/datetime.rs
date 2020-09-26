mod fixtures;

use icu_cldr_json_data_provider::{CldrJsonDataProvider, CldrPaths};
use icu_datetime::DateTimeFormat;
use std::fmt::Write;

#[test]
fn test_fixtures() {
    let mut cldr_paths = CldrPaths::default();

    cldr_paths.cldr_dates = Ok("./tests/fixtures/data/cldr/cldr-dates-modern".into());

    let provider = CldrJsonDataProvider::new(&cldr_paths);

    for fx in fixtures::get_fixture("styles").unwrap().0 {
        let langid = fx.input.locale.parse().unwrap();
        let options = fixtures::get_options(&fx.input.options);
        let dtf = DateTimeFormat::try_new(langid, &provider, &options).unwrap();

        let value = fixtures::parse_date(&fx.input.value).unwrap();

        let result = dtf.format_to_string(&value);
        assert_eq!(result, fx.output.value);

        let mut s = String::new();
        dtf.format_to_write(&mut s, &value).unwrap();
        assert_eq!(s, fx.output.value);

        let fdt = dtf.format(&value);
        assert_eq!(fdt.to_string(), fx.output.value);

        let mut s = String::new();
        write!(s, "{}", fdt).unwrap();
        assert_eq!(s, fx.output.value);
    }
}
