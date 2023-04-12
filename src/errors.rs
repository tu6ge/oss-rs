use http::StatusCode;
use std::fmt;
use thiserror::Error;

use crate::{
    bucket::ExtractItemError,
    builder::BuilderError,
    config::InvalidConfig,
    object::ExtractListError,
    types::{
        object::{InvalidObjectDir, InvalidObjectPath},
        InvalidBucketName, InvalidEndPoint,
    },
};

/// 内置的 Error 集合
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum OssError {
    #[doc(hidden)]
    #[error("{0}")]
    #[cfg(test)]
    Dotenv(#[from] dotenv::Error),

    #[doc(hidden)]
    #[error("{0}")]
    BuilderError(#[from] BuilderError),

    #[doc(hidden)]
    #[error("{0}")]
    InvalidEndPoint(#[from] InvalidEndPoint),

    #[doc(hidden)]
    #[error("{0}")]
    InvalidBucketName(#[from] InvalidBucketName),

    #[doc(hidden)]
    #[error("{0}")]
    InvalidConfig(#[from] InvalidConfig),

    #[doc(hidden)]
    #[error("{0}")]
    InvalidObjectPath(#[from] InvalidObjectPath),

    #[doc(hidden)]
    #[error("{0}")]
    InvalidObjectDir(#[from] InvalidObjectDir),

    #[doc(hidden)]
    #[error("{0}")]
    BuildInItemError(#[from] crate::object::BuildInItemError),

    #[cfg(feature = "decode")]
    #[doc(hidden)]
    #[error("{0}")]
    InnerListError(#[from] crate::decode::InnerListError),

    #[doc(hidden)]
    #[error("{0}")]
    ExtractList(#[from] ExtractListError),

    #[doc(hidden)]
    #[error("{0}")]
    ExtractItem(#[from] ExtractItemError),
}

/// # 保存并返回 OSS 服务端返回是数据
/// 当服务器返回的状态码不在 200<=x 且 x<300 范围时，则会返回此错误
///
/// 如果解析 xml 格式错误，则会返回默认值，默认值的 status = 200
#[derive(Debug, Error, PartialEq, Eq)]
pub struct OssService {
    #[doc(hidden)]
    pub code: String,

    #[doc(hidden)]
    pub status: StatusCode,

    #[doc(hidden)]
    pub message: String,

    #[doc(hidden)]
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
        match (
            source.find("<Code>"),
            source.find("</Code>"),
            source.find("<Message>"),
            source.find("</Message>"),
            source.find("<RequestId>"),
            source.find("</RequestId>"),
        ) {
            (
                Some(code0),
                Some(code1),
                Some(message0),
                Some(message1),
                Some(request_id0),
                Some(request_id1),
            ) => Self {
                code: source[code0 + 6..code1].to_owned(),
                status: *status,
                message: source[message0 + 9..message1].to_owned(),
                request_id: source[request_id0 + 11..request_id1].to_owned(),
            },
            _ => Self::default(),
        }
    }
}

/// 内置的 Result
pub type OssResult<T> = Result<T, OssError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oss_service_display() {
        assert_eq!(
            format!(
                "{}",
                OssService {
                    code: "abc".to_owned(),
                    status: StatusCode::OK,
                    message: "mes1".to_owned(),
                    request_id: "xx".to_owned(),
                }
            ),
            "OssService { code: \"abc\", status: 200, message: \"mes1\", request_id: \"xx\" }"
        );
    }

    #[test]
    fn oss_service_default() {
        let oss = OssService::default();
        assert_eq!(oss.code, "Undefined".to_string());
        assert_eq!(oss.status, StatusCode::OK);
        assert_eq!(
            oss.message,
            "Parse aliyun response xml error message failed.".to_owned()
        );
        assert_eq!(oss.request_id, "XXXXXXXXXXXXXXXXXXXXXXXX".to_owned());
    }

    #[test]
    fn oss_service_new() {
        assert_eq!(
            OssService::new("abc", &StatusCode::OK),
            OssService::default()
        );
    }
}
