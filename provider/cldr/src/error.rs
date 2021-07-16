// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use icu_locid::LanguageIdentifier;
use std::path::{Path, PathBuf};
use displaydoc::Display;

#[cfg(feature = "download")]
use crate::download;

#[non_exhaustive]
#[derive(Display, Debug)]
pub enum Error {
    #[displaydoc("{0}: {1:?}")]
    Io(std::io::Error, Option<PathBuf>),
    #[displaydoc("JSON error: {0}: {1:?}")]
    Json(serde_json::error::Error, Option<PathBuf>),
    #[displaydoc("{0}: {1:?}")]
    Custom(String, Option<LanguageIdentifier>),
    #[displaydoc("{0}")]
    MissingSource(MissingSourceError),
    #[cfg(feature = "download")]
    #[displaydoc("{0}")]
    Download(download::Error),
    #[displaydoc("poisoned lock on CLDR provider")]
    Poison,
}

impl std::error::Error for Error {}


#[cfg(feature = "download")]
impl From<download::Error> for Error {
    fn from(err: download::Error) -> Error {
        match err {
            download::Error::Io(err, path) => Error::Io(err, path),
            _ => Error::Download(err),
        }
    }
}

#[derive(Display, Debug, PartialEq, Copy, Clone)]
#[displaydoc("Missing CLDR data source: {src}")]
pub struct MissingSourceError {
    pub src: &'static str,
}

impl std::error::Error for Error {}


/// To help with debugging, I/O errors should be paired with a file path.
/// If a path is unavailable, create the error directly: [`Error::Io`]`(err, `[`None`]`)`
impl<P: AsRef<Path>> From<(std::io::Error, P)> for Error {
    fn from(pieces: (std::io::Error, P)) -> Self {
        Self::Io(pieces.0, Some(pieces.1.as_ref().to_path_buf()))
    }
}

/// To help with debugging, JSON errors should be paired with a file path.
/// If a path is unavailable, create the error directly: [`Error::Json`]`(err, `[`None`]`)`
impl<P: AsRef<Path>> From<(serde_json::error::Error, P)> for Error {
    fn from(pieces: (serde_json::error::Error, P)) -> Self {
        Self::Json(pieces.0, Some(pieces.1.as_ref().to_path_buf()))
    }
}

/// To help with debugging, string errors should be paired with a locale.
/// If a locale is unavailable, create the error directly: [`Error::Custom`]`(err, `[`None`]`)`
impl<L: AsRef<LanguageIdentifier>> From<(String, L)> for Error {
    fn from(pieces: (String, L)) -> Self {
        Self::Custom(pieces.0, Some(pieces.1.as_ref().clone()))
    }
}

/// To help with debugging, string errors should be paired with a locale.
/// If a locale is unavailable, create the error directly: [`Error::Custom`]`(err, `[`None`]`)`
impl<L: AsRef<LanguageIdentifier>> From<(&'static str, L)> for Error {
    fn from(pieces: (&'static str, L)) -> Self {
        Self::Custom(pieces.0.to_string(), Some(pieces.1.as_ref().clone()))
    }
}

impl From<MissingSourceError> for Error {
    fn from(err: MissingSourceError) -> Self {
        Self::MissingSource(err)
    }
}
