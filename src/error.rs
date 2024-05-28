use std::{
    fmt::{self, Display},
    num::ParseIntError,
};

use reqwest::header::{InvalidHeaderValue, ToStrError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OssError {
    Reqwest(#[from] reqwest::Error),

    HeaderValue(#[from] InvalidHeaderValue),

    Chrono(#[from] chrono::ParseError),

    ToStrError(#[from] ToStrError),

    NoFoundCreationDate,

    NoFoundStorageClass,

    NoFoundDataRedundancyType,

    NoFoundContentLength,

    NoFoundEtag,

    NoFoundLastModified,

    ParseIntError(#[from] ParseIntError),

    Upload(String),

    NoFoundBucket,
}

impl Display for OssError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "oss error".fmt(f)
    }
}
