# icu [![crates.io](http://meritbadge.herokuapp.com/icu)](https://crates.io/crates/icu)

`icu` is the main meta-package of the `ICU4X` project.

It provides a comprehensive selection of
[Unicode Internationalization Components](http://site.icu-project.org/)
in their canonical configurations intended to enable software
internationalization capabilities.

The package is provided for convenience and users are encouraged
to fine-tune components with the features they need.

The package does not bring any unique functionality. Users
can achieve the exact same by manually including the dependent
components with pre-selected features.

## Data Management

Most of Unicode functionality relies on data which has to be provided
to the APIs.

`ICU4X` project uses a concept of [`DataProvider`] - a service used to
handle data management.

There can be many different heuristics for handling data management and
this meta-package does not supply any default [`DataProvider`].

When using `ICU4X` users are expected to decide which provider they want to use
and instrument it to point at the correct location where the data files are stored.

In the following examples an [`icu_testdata`] package is used which wraps
an [`FsDataProvider`] with locally available subset of data.

## Features

ICU4X components share a set of common features that control whether core pieces of
functionality are compiled. These features are:

- `provider_serde`: Whether to include Serde Serialize/Deserialize implementations for
  ICU4X locale data structs, such as [`SymbolsV1`]. (On by default)
- `serde`: Whether to include Serde Serialize/Deserialize implementations for core libary
  types, such as [`Locale`].
- `bench`: Whether to enable exhaustive benchmarks. This can be enabled on individual crates
  when running `cargo bench`.
- `experimental`: Whether to enable experimental preview features. Modules enabled with
  this feature may not be production-ready and could change at any time.
- `provider_transform_internals`: This is code that is useful for transforming CLDR data, but
  may be risky to ship as a public API. For instance in DateTimeFormat code we may want to
  manipulate patterns, but not allow end users to do the same.

## Example

```rust
use icu::locid::Locale;
use icu::locid::macros::langid;
use icu::datetime::{DateTimeFormat, mock::datetime::MockDateTime, options::length};

let provider = icu_testdata::get_provider();

let locale: Locale = langid!("en").into();

let options = length::Bag {
    date: Some(length::Date::Long),
    time: Some(length::Time::Medium),
    ..Default::default()
}.into();

let dtf = DateTimeFormat::try_new(locale, &provider, &options)
    .expect("Failed to create DateTimeFormat instance.");

let date: MockDateTime = "2020-09-12T12:35:00".parse()
    .expect("Failed to parse date.");

let formatted_date = dtf.format(&date);
assert_eq!(formatted_date.to_string(), "September 12, 2020 at 12:35:00 PM");
```

[`DataProvider`]: ../icu_provider/prelude/trait.DataProvider.html
[`FsDataProvider`]: ../icu_provider_fs/struct.FsDataProvider.html
[`icu_testdata`]: ../icu_testdata/index.html
[`Locale`]: crate::locid::Locale
[`SymbolsV1`]: crate::decimal::provider::DecimalSymbolsV1

## More Information

For more information on development, authorship, contributing etc. please visit [`ICU4X home page`](https://github.com/unicode-org/icu4x).
