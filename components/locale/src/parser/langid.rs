use std::iter::Peekable;

pub use super::errors::ParserError;
use crate::subtags;
use crate::LanguageIdentifier;

#[derive(PartialEq)]
pub enum ParserMode {
    LanguageIdentifier,
    Locale,
    Partial,
}

#[derive(PartialEq)]
enum ParserPosition {
    Script,
    Region,
    Variant,
}

pub fn parse_language_identifier_from_iter<'a>(
    iter: &mut Peekable<impl Iterator<Item = &'a [u8]>>,
    mode: ParserMode,
) -> Result<LanguageIdentifier, ParserError> {
    let language;
    let mut script = None;
    let mut region = None;
    let mut variant = None;

    if let Some(subtag) = iter.next() {
        language = subtags::Language::from_bytes(subtag)?;
    } else {
        return Err(ParserError::InvalidLanguage);
    }

    let mut position = ParserPosition::Script;

    while let Some(subtag) = iter.peek() {
        if mode != ParserMode::LanguageIdentifier && subtag.len() == 1 {
            break;
        }

        if position == ParserPosition::Script {
            if let Ok(s) = subtags::Script::from_bytes(subtag) {
                script = Some(s);
                position = ParserPosition::Region;
            } else if let Ok(s) = subtags::Region::from_bytes(subtag) {
                region = Some(s);
                position = ParserPosition::Variant;
            } else if let Ok(v) = subtags::Variant::try_from_iter(iter) {
                variant = Some(v);
                break;
            } else if mode != ParserMode::Partial {
                return Err(ParserError::InvalidSubtag);
            } else {
                break;
            }
        } else if position == ParserPosition::Region {
            if let Ok(s) = subtags::Region::from_bytes(subtag) {
                region = Some(s);
                position = ParserPosition::Variant;
            } else if let Ok(v) = subtags::Variant::try_from_iter(iter) {
                variant = Some(v);
                break;
            } else if mode != ParserMode::Partial {
                return Err(ParserError::InvalidSubtag);
            } else {
                break;
            }
        } else if let Ok(v) = subtags::Variant::try_from_iter(iter) {
            variant = Some(v);
            break;
        } else if mode != ParserMode::Partial {
            return Err(ParserError::InvalidSubtag);
        } else {
            break;
        }
        iter.next();
    }

    Ok(LanguageIdentifier {
        language,
        script,
        region,
        variant,
    })
}

pub fn parse_language_identifier(
    t: &[u8],
    mode: ParserMode,
) -> Result<LanguageIdentifier, ParserError> {
    let mut iter = t.split(|c| *c == b'-' || *c == b'_').peekable();
    parse_language_identifier_from_iter(&mut iter, mode)
}
