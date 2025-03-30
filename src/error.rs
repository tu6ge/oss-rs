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

    InvalidBucket,

    InvalidOssError(String),

    CopySourceNotFound,
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
