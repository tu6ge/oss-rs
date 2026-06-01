use reqwest::header::HeaderMap;

use crate::error::OssError;

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for reqwest::Response {}

fn parse_etag(headers: &HeaderMap) -> Result<String, OssError> {
    let etag = headers
        .get("etag")
        .ok_or(OssError::NoFoundEtag)?
        .to_str()
        .map_err(OssError::ToStrError)?;
    Ok(etag.trim_matches('"').to_string())
}

/// 将 OSS HTTP 响应转为 `Result<(), OssError>`；仅 crate 内可用，且不可在外部实现。
pub(crate) trait OssResponseExt: sealed::Sealed {
    async fn into_oss_empty_result(self) -> Result<(), OssError>;
    async fn into_oss_put_result(self) -> Result<String, OssError>;
}

impl OssResponseExt for reqwest::Response {
    async fn into_oss_empty_result(self) -> Result<(), OssError> {
        if self.status().is_success() {
            Ok(())
        } else {
            let body = self.text().await?;
            Err(OssError::from_service(&body))
        }
    }

    async fn into_oss_put_result(self) -> Result<String, OssError> {
        let is_success = self.status().is_success();
        let headers = self.headers().clone();
        if is_success {
            parse_etag(&headers)
        } else {
            let body = self.text().await?;
            Err(OssError::from_service(&body))
        }
    }
}
