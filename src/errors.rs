use thiserror::Error;

#[derive(Debug, Error)]
pub enum OssError{
  #[error("reqwest error: {0}")]
  Request(#[from] reqwest::Error),

  #[error("url parse error: {0}")]
  UrlParse(#[from] url::ParseError),

  #[error("{0}")]
  #[cfg(test)]
  Dotenv(#[from] dotenv::Error),

  #[error("var error: {0}")]
  VarError(#[from] std::env::VarError),

  #[error("input data is not valid: {0}")]
  Input(String),

  #[error("io error: {0}")]
  Io(#[from] std::io::Error),

  #[error("InvalidHeaderName: {0}")]
  InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),

  #[error("InvalidHeaderValue: {0}")]
  InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

  #[error("QuickXml error: {0}")]
  QuickXml(#[from] quick_xml::Error),

  #[error("chrono error: {0}")]
  Chrono(#[from] chrono::ParseError),

  #[error("ParseIntError: {0}")]
  ParseIntError(#[from] std::num::ParseIntError),

  #[error("ToStrError: {0}")]
  ToStrError(#[from] reqwest::header::ToStrError),

  #[error(transparent)]
  Other(#[from] anyhow::Error),
}

pub type OssResult<T> = Result<T,OssError>;