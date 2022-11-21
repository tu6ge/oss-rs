use hmac::digest::crypto_common;
use http::header::ToStrError;
use std::fmt;
use thiserror::Error;

#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::{
    config::InvalidConfig,
    traits::{
        InvalidBucketListValue, InvalidBucketValue, InvalidObjectListValue, InvalidObjectValue,
    },
    types::InvalidEndPoint,
};

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum OssError {
    #[error("reqwest error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("invalid header value msg: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("{0}")]
    #[cfg(test)]
    Dotenv(#[from] dotenv::Error),

    #[error("var error: {0}")]
    VarError(#[from] std::env::VarError),

    #[error("input error: {0}")]
    Input(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("QuickXml error: {0}")]
    QuickXml(#[from] quick_xml::Error),

    #[error("chrono error: {0}")]
    Chrono(#[from] chrono::ParseError),

    #[error("toStrError: {0}")]
    ToStr(String),

    #[error("{0}")]
    ToStrError(#[from] ToStrError),

    #[error("ParseIntError: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("hmac InvalidLength: {0}")]
    InvalidLength(#[from] crypto_common::InvalidLength),

    #[error("FromUtf8Error: {0}")]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error("aliyun response error: {0}")]
    OssService(#[from] OssService),

    #[error("{0}")]
    InvalidEndPoint(#[from] InvalidEndPoint),

    #[error("{0}")]
    InvalidObjectValue(#[from] InvalidObjectValue),

    #[error("{0}")]
    InvalidObjectListValue(#[from] InvalidObjectListValue),

    #[error("{0}")]
    InvalidBucketValue(#[from] InvalidBucketValue),

    #[error("{0}")]
    InvalidBucketListValue(#[from] InvalidBucketListValue),

    #[error("{0}")]
    InvalidConfig(#[from] InvalidConfig),

    /// 用于 Stream
    #[error("Without More Content")]
    WithoutMore,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl OssError {
    /// 返回 oss 服务端的错误信息
    pub fn message(self) -> String {
        match self {
            OssError::OssService(e) => e.message,
            _ => self.to_string(),
        }
    }
}

#[derive(Debug, Error, Default, PartialEq)]
pub struct OssService {
    pub code: String,
    pub message: String,
    pub request_id: String,
}

impl fmt::Display for OssService {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("OssService")
            .field("code", &self.code)
            .field("message", &self.message)
            .field("request_id", &self.request_id)
            .finish()
    }
}

#[cfg_attr(test, automock)]
impl OssService {
    /// 解析 oss 的错误信息
    pub fn new(source: String) -> Self {
        println!("{}", source);
        let code0 = source.find("<Code>").unwrap();
        let code1 = source.find("</Code>").unwrap();
        let message0 = source.find("<Message>").unwrap();
        let message1 = source.find("</Message>").unwrap();
        let request_id0 = source.find("<RequestId>").unwrap();
        let request_id1 = source.find("</RequestId>").unwrap();

        Self {
            code: (&source[code0 + 6..code1]).to_string(),
            message: (&source[message0 + 9..message1]).to_string(),
            request_id: (&source[request_id0 + 11..request_id1]).to_string(),
        }
    }
}

pub type OssResult<T> = Result<T, OssError>;
