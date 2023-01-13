use http::{header::ToStrError, StatusCode};
use std::fmt;
use thiserror::Error;

use crate::{
    bucket::InvalidBucketValue,
    builder::BuilderError,
    config::{InvalidConfig, InvalidObjectPath},
    types::{InvalidBucketName, InvalidEndPoint},
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

    #[cfg(feature = "decode")]
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

    // #[error("hmac InvalidLength: {0}")]
    // InvalidLength(#[from] crypto_common::InvalidLength),

    // #[error("FromUtf8Error: {0}")]
    // FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("aliyun response error: {0}")]
    OssService(#[from] OssService),

    #[error("{0}")]
    BuilderError(#[from] BuilderError),

    #[error("{0}")]
    InvalidEndPoint(#[from] InvalidEndPoint),

    #[error("{0}")]
    InvalidBucketValue(#[from] InvalidBucketValue),

    #[error("{0}")]
    InvalidBucketName(#[from] InvalidBucketName),

    #[error("{0}")]
    InvalidConfig(#[from] InvalidConfig),

    #[error("{0}")]
    InvalidObjectPath(#[from] InvalidObjectPath),

    /// 用于 Stream
    #[error("Without More Content")]
    WithoutMore,
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

/// # 保存并返回 OSS 服务端返回是数据
/// 当服务器返回的状态码不在 200<=x 且 x<300 范围时，则会返回此错误
///
/// 如果解析 xml 格式错误，则会返回默认值，默认值的 status = 200
#[derive(Debug, Error, PartialEq, Eq)]
pub struct OssService {
    pub code: String,
    pub status: StatusCode,
    pub message: String,
    pub request_id: String,
}

impl Default for OssService {
    fn default() -> Self {
        Self {
            code: "Undefined".to_owned(),
            status: StatusCode::default(),
            message: "Parse aliyun response xml error message failed.".to_owned(),
            request_id: "XXXXXXXXXXXXXXXXXXXXXXXX".to_owned(),
        }
    }
}

impl fmt::Display for OssService {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("OssService")
            .field("code", &self.code)
            .field("status", &self.status)
            .field("message", &self.message)
            .field("request_id", &self.request_id)
            .finish()
    }
}

impl<'a> OssService {
    /// 解析 oss 的错误信息
    pub fn new(source: &'a str, status: &StatusCode) -> Self {
        let code0 = match source.find("<Code>") {
            Some(offset) => offset,
            None => return Self::default(),
        };
        let code1 = match source.find("</Code>") {
            Some(offset) => offset,
            None => return Self::default(),
        };
        let message0 = match source.find("<Message>") {
            Some(offset) => offset,
            None => return Self::default(),
        };
        let message1 = match source.find("</Message>") {
            Some(offset) => offset,
            None => return Self::default(),
        };
        let request_id0 = match source.find("<RequestId>") {
            Some(offset) => offset,
            None => return Self::default(),
        };
        let request_id1 = match source.find("</RequestId>") {
            Some(offset) => offset,
            None => return Self::default(),
        };

        Self {
            code: source[code0 + 6..code1].to_owned(),
            status: *status,
            message: source[message0 + 9..message1].to_owned(),
            request_id: source[request_id0 + 11..request_id1].to_owned(),
        }
    }
}

pub type OssResult<T> = Result<T, OssError>;
