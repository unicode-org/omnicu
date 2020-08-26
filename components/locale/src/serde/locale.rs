use crate::Locale;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Locale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Locale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LocaleVisitor;

        impl<'de> serde::de::Visitor<'de> for LocaleVisitor {
            type Value = Locale;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a valid Unicode Locale")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                s.parse::<Locale>().map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_string(LocaleVisitor)
    }
}

#[test]
fn serialize() -> Result<(), Box<dyn std::error::Error>> {
    let fr = serde_json::json!("fr".parse::<Locale>()?);
    let en = serde_json::json!("en-US".parse::<Locale>()?);

    assert_eq!(serde_json::to_string(&fr)?, r#""fr""#);
    assert_eq!(serde_json::to_string(&en)?, r#""en-US""#);

    Ok(())
}

#[test]
fn deserialize() -> Result<(), Box<dyn std::error::Error>> {
    let fr = serde_json::from_str::<Locale>(r#""fr""#)?;
    let en = serde_json::from_str::<Locale>(r#""en-US""#)?;

    assert_eq!(fr, "fr".parse::<Locale>()?);
    assert_eq!(en, "en-US".parse::<Locale>()?);

    let failed = serde_json::from_str::<Locale>(r#""2Xs""#);
    assert!(failed.is_err());
    let err = failed.unwrap_err();
    assert!(err.is_data());
    assert_eq!(
        err.to_string(),
        "The given language subtag is invalid at line 1 column 5".to_string()
    );

    let failed_lang_subtag = serde_json::from_str::<Locale>(r#""en-US-invalidsubtag""#);
    assert!(failed_lang_subtag.is_err());
    let err_lang_subtag = failed_lang_subtag.unwrap_err();
    assert!(err_lang_subtag.is_data());
    assert_eq!(
        err_lang_subtag.to_string(),
        "Invalid subtag at line 1 column 21".to_string()
    );

    let failed_subtag = serde_json::from_str::<Locale>(r#""en-US-u-kn-invalidsubtag""#);
    assert!(failed_subtag.is_err());
    let err_subtag = failed_subtag.unwrap_err();
    assert!(err_subtag.is_data());
    assert_eq!(
        err_subtag.to_string(),
        "Invalid extension at line 1 column 26".to_string()
    );

    let failed_extension = serde_json::from_str::<Locale>(r#""en-US-e-kn-something""#);
    assert!(failed_extension.is_err());
    let err_extension = failed_extension.unwrap_err();
    assert!(err_extension.is_data());
    assert_eq!(
        err_extension.to_string(),
        "Invalid extension at line 1 column 22".to_string()
    );

    Ok(())
}
