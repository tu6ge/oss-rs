use crate::error::OssError;

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for reqwest::Response {}

/// 将 OSS HTTP 响应转为 `Result<(), OssError>`；仅 crate 内可用，且不可在外部实现。
pub(crate) trait OssResponseExt: sealed::Sealed {
    async fn into_oss_empty_result(self) -> Result<(), OssError>;
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
}
