// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use std::borrow::Cow;
use std::fmt;

fn deserialize_option_cow_str<'de, D>(deserializer: D) -> Result<Option<Cow<'de, str>>, D::Error>
where
    D: serde::de::Deserializer<'de>
{
    use serde::de::Error;
    use serde::de::Unexpected;

    // Note: The following visitor is taken from serde::private::de::borrow_cow_str
    struct CowStrVisitor;

    impl<'a> serde::de::Visitor<'a> for CowStrVisitor {
        type Value = Cow<'a, str>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Owned(v.to_owned()))
        }

        fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Borrowed(v))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(Cow::Owned(v))
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match std::str::from_utf8(v) {
                Ok(s) => Ok(Cow::Owned(s.to_owned())),
                Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
            }
        }

        fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match std::str::from_utf8(v) {
                Ok(s) => Ok(Cow::Borrowed(s)),
                Err(_) => Err(Error::invalid_value(Unexpected::Bytes(v), &self)),
            }
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: Error,
        {
            match String::from_utf8(v) {
                Ok(s) => Ok(Cow::Owned(s)),
                Err(e) => Err(Error::invalid_value(
                    Unexpected::Bytes(&e.into_bytes()),
                    &self,
                )),
            }
        }
    }

    struct OptionCowStrVisitor;

    impl<'a> serde::de::Visitor<'a> for OptionCowStrVisitor {
        type Value = Option<Cow<'a, str>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::de::Deserializer<'a>,
        {
            Ok(Some(deserializer.deserialize_str(CowStrVisitor)?))
        }
    }

    deserializer.deserialize_option(OptionCowStrVisitor)
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct Demo<'s> {
    #[serde(borrow, deserialize_with = "deserialize_option_cow_str")]
    message: Option<Cow<'s, str>>,
}

#[test]
fn demo() {
    let data_orig = Demo {
        message: Some(Cow::Borrowed("Hello World")),
    };
    let json_buf = serde_json::to_string(&data_orig).expect("serialize");
    assert_eq!("{\"message\":\"Hello World\"}", json_buf);
    let data_new: Demo = serde_json::from_str(&json_buf).expect("deserialize from buffer");
    assert_eq!(data_orig, data_new);
    assert!(matches!(data_new.message, Some(Cow::Borrowed(_))));
}
