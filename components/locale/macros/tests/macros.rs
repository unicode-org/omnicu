use icu_locale_macros::*;
use icu_locale::subtags;
use icu_locale::LanguageIdentifier;

const LANG_PL: subtags::Language = language!("pL");
const SCRIPT_LATN: subtags::Script = script!("lAtN");
const REGION_US: subtags::Region = region!("us");
const VARIANT_MACOS: subtags::Variant = variant!("MACOS");
const LANGID: LanguageIdentifier = langid!("de-Arab-AT");

#[test]
fn language() {
    let lang = language!("Pl");

    assert_eq!(lang, "pl");
    assert_eq!(LANG_PL, "pl");
    assert_eq!(lang, LANG_PL);
}

#[test]
fn script() {
    let script = script!("latn");

    assert_eq!(script, "Latn");
    assert_eq!(SCRIPT_LATN, "Latn");
    assert_eq!(script, SCRIPT_LATN);
}

#[test]
fn region() {
    let region = region!("us");

    assert_eq!(region, "US");
    assert_eq!(REGION_US, "US");
    assert_eq!(region, REGION_US);
}

#[test]
fn variant() {
    let variant = variant!("macOS");

    assert_eq!(variant, "macos");
    assert_eq!(VARIANT_MACOS, "macos");
    assert_eq!(variant, VARIANT_MACOS);
}

#[test]
fn langid() {
    let langid = langid!("de_Arab_aT");

    assert_eq!(langid.to_string(), "de-Arab-AT");
    assert_eq!(LANGID.to_string(), "de-Arab-AT");
    assert_eq!(langid, LANGID);
}
