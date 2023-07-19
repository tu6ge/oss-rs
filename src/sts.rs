//! # STS 临时访问权限管理服务
//! 阿里云STS（Security Token Service）是阿里云提供的一种临时访问权限管理服务。RAM提供RAM用户和RAM角色两种身份。
//! 其中，RAM角色不具备永久身份凭证，而只能通过STS获取可以自定义时效和访问权限的临时身份凭证，即安全令牌（STS Token）。
//! @link [文档](https://help.aliyun.com/document_detail/28756.html)
//!
//! ## 用法
//!
//! ```
//! # async fn run() {
//! use aliyun_oss_client::{sts::STS, BucketName, Client, EndPoint};
//! let client = Client::new_with_sts(
//!     "STS.xxxxxxxx".into(),                         // KeyId
//!     "EVd6dXew6xxxxxxxxxxxxxxxxxxxxxxxxxxx".into(), // KeySecret
//!     EndPoint::CnShanghai,
//!     BucketName::new("yyyyyy").unwrap(),
//!     "CAIS4gF1q6Ft5Bxxxxxxxxxxx".to_string(), // STS Token, type should be string, &str or more
//! )
//! .unwrap();
//!
//! let builder = client.get_bucket_list().await;
//! println!("{:?}", builder);
//! # }
//! ```

use http::{header::InvalidHeaderValue, HeaderValue};

use crate::{auth::AuthBuilder, client::Client, BucketName, EndPoint, KeyId, KeySecret};

/// 给 Client 增加 STS 能力
pub trait STS: private::Sealed
where
    Self: Sized,
{
    /// 用 STS 配置信息初始化 [`Client`]
    ///
    /// [`Client`]: crate::client::Client
    fn new_with_sts<ST>(
        access_key_id: KeyId,
        access_key_secret: KeySecret,
        endpoint: EndPoint,
        bucket: BucketName,
        security_token: ST,
    ) -> Result<Self, InvalidHeaderValue>
    where
        ST: TryInto<HeaderValue>,
        ST::Error: Into<InvalidHeaderValue>;
}

mod private {
    pub trait Sealed {}
}

impl<M: Default + Clone> private::Sealed for Client<M> {}

const SECURITY_TOKEN: &str = "x-oss-security-token";

impl<M: Default + Clone> STS for Client<M> {
    fn new_with_sts<ST>(
        access_key_id: KeyId,
        access_key_secret: KeySecret,
        endpoint: EndPoint,
        bucket: BucketName,
        security_token: ST,
    ) -> Result<Self, InvalidHeaderValue>
    where
        ST: TryInto<HeaderValue>,
        ST::Error: Into<InvalidHeaderValue>,
    {
        let mut auth_builder = AuthBuilder::default();
        auth_builder.key(access_key_id);
        auth_builder.secret(access_key_secret);
        auth_builder.header_insert(SECURITY_TOKEN, {
            let mut token = security_token.try_into().map_err(|e| e.into())?;
            token.set_sensitive(true);
            token
        });

        Ok(Self::from_builder(auth_builder, endpoint, bucket))
    }
}

#[cfg(test)]
mod tests {
    use http::{HeaderValue, Method};

    use crate::{file::AlignBuilder, types::CanonicalizedResource, BucketName, Client, EndPoint};

    use super::STS;

    #[tokio::test]
    async fn test_sts() {
        let client = Client::new_with_sts(
            "foo1".into(),
            "foo2".into(),
            EndPoint::CN_SHANGHAI,
            BucketName::new("abc").unwrap(),
            "bar",
        )
        .unwrap();

        let builder = client
            .builder(
                Method::GET,
                "https://abc.oss-cn-shanghai.aliyuncs.com/"
                    .try_into()
                    .unwrap(),
                CanonicalizedResource::default(),
            )
            .unwrap();

        let request = builder.build().unwrap();

        let headers = request.headers();
        let sts_token = headers.get("x-oss-security-token");

        assert_eq!(sts_token, Some(&HeaderValue::from_static("bar")));
    }
}
