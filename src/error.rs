use std::{
    env::VarError,
    fmt::{self, Display},
    io,
    num::ParseIntError,
};

use reqwest::header::{InvalidHeaderValue, ToStrError};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OssError {
    Reqwest(#[from] reqwest::Error),

    HeaderValue(#[from] InvalidHeaderValue),

    Chrono(#[from] chrono::ParseError),

    ToStrError(#[from] ToStrError),

    VarError(#[from] VarError),

    IoError(#[from] io::Error),

    NoFoundCreationDate,

    NoFoundStorageClass,

    NoFoundDataRedundancyType,

    NoFoundContentLength,

    NoFoundEtag,

    NoFoundLastModified,

    ParseIntError(#[from] ParseIntError),

    Service(ServiceXML),

    NoFoundBucket,

    ParseXml(#[from] serde_xml_rs::Error),

    InvalidEndPoint,

    InvalidRegion,

    InvalidBucket,

    NotSetDefaultBucket,

    BucketName(BucketNameError),

    InvalidBucketUrl,

    InvalidOssError(String),

    CopySourceNotFound,

    NoFoundUploadId,

    NoFoundContinuationToken,
}

impl OssError {
    pub(crate) fn from_service(xml: &str) -> Self {
        match ServiceXML::new(xml) {
            Ok(xml) => Self::Service(xml),
            Err(e) => Self::InvalidOssError(e.to_string()),
        }
    }
}

impl Display for OssError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "oss error".fmt(f)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Error")]
#[allow(dead_code)]
pub struct ServiceXML {
    #[serde(rename = "Code")]
    code: String,

    #[serde(rename = "Message")]
    message: String,

    #[serde(rename = "RequestId")]
    request_id: String,

    #[serde(rename = "RecommendDoc")]
    recommend_doc: String,
}
impl ServiceXML {
    fn new(xml: &str) -> Result<Self, serde_xml_rs::Error> {
        //println!("{xml}");
        serde_xml_rs::from_str(xml)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BucketNameError {
    #[error("bucket name length must be between 3 and 63 characters")]
    InvalidLength,

    #[error("bucket name contains invalid character: '{0}'")]
    InvalidCharacter(char),

    #[error("bucket name must start with a letter or digit")]
    InvalidStart,

    #[error("bucket name must end with a letter or digit")]
    InvalidEnd,

    #[error("bucket name must not be formatted like an IP address")]
    LooksLikeIpAddress,
}

impl From<BucketNameError> for OssError {
    fn from(value: BucketNameError) -> Self {
        Self::BucketName(value)
    }
}
