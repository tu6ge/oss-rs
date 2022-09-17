use hmac::digest::crypto_common;
use thiserror::Error;
use std::fmt;
use regex::Regex;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum OssError{
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

  #[error("ParseIntError: {0}")]
  ParseIntError(#[from] std::num::ParseIntError),

  #[error("hmac InvalidLength: {0}")]
  InvalidLength(#[from] crypto_common::InvalidLength),

  #[error("FromUtf8Error: {0}")]
  FromUtf8Error(#[from] std::string::FromUtf8Error),

  #[error("aliyun response error: {0}")]
  OssService(#[from] OssService),

  #[cfg(feature = "plugin")]
  #[error("plugin : {0}")]
  Plugin(#[from] self::plugin::PluginError),

  #[error(transparent)]
  Other(#[from] anyhow::Error),
}

impl OssError{
  /// 返回 oss 服务端的错误信息
  pub fn message(self) -> String{
    match self {
      OssError::OssService(e) => e.message,
      _ => self.to_string(),
    }
  }
}


#[derive(Debug, Error, Default)]
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
impl OssService{
    /// 解析 oss 的错误信息
    pub fn new(source: String) -> OssService {
        let re = Regex::new(
          r"(?x)<Code>(?P<code>\w+)</Code>
          [\n]?[\s]+<Message>(?P<message>[\w\s.]+)</Message>
          [\n]?[\s]+<RequestId>(?P<request_id>[\w]+)</RequestId>
          "
        ).unwrap();
        let caps = re.captures(&source).unwrap();
        OssService{
          code: (&caps["code"]).to_string(),
          message: (&caps["message"]).to_string(),
          request_id: (&caps["request_id"]).to_string(),
        }
    }
}

#[cfg(feature = "plugin")]
pub mod plugin {
  use std::fmt;

  #[derive(Debug)]
  pub struct PluginError {
    pub name: &'static str,
    pub message: String,
  }

  impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
      f.debug_struct("PluginError")
        .field("name", &self.name)
        .field("message", &self.message)
        .finish()
    }
  }

  impl std::error::Error for PluginError {}
}


pub type OssResult<T> = Result<T,OssError>;
