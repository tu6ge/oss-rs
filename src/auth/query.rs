//! # 在 Url (即 query) 中包含签名
//!
//! [aliyun docs](https://help.aliyun.com/document_detail/31952.html)
//!
//! ## 用法
//! ```
//! # use aliyun_oss_client::{auth::QueryAuth, EndPoint};
//! # use chrono::Utc;
//! let key = "key".into();
//! let secret = "secret".into();
//! let bucket = "bucket".parse().unwrap();
//! let auth = QueryAuth::new(
//!     &key,
//!     &secret,
//!     &EndPoint::CN_QINGDAO,
//!     &bucket
//! );
//! let time = Utc::now().timestamp() + 3600;
//! let url = auth.to_url(&"pretty.png".parse().unwrap(), time);
//! ```

use url::Url;

use crate::{
    types::{object::SetObjectPath, url_from_bucket, CanonicalizedResource},
    BucketName, EndPoint, KeyId, KeySecret, ObjectPath,
};

/// Query 签名器
pub struct QueryAuth<'a> {
    access_key_id: &'a KeyId,
    access_secret_key: &'a KeySecret,
    endpoint: &'a EndPoint,
    bucket: &'a BucketName,
}

#[cfg(feature = "core")]
use crate::{
    client::Client,
    config::{BucketBase, Config},
};

#[cfg(feature = "core")]
impl<'a> From<&'a Config> for QueryAuth<'a> {
    #[inline]
    fn from(config: &'a Config) -> Self {
        Self::new(
            config.as_ref(),
            config.as_ref(),
            config.as_ref(),
            config.as_ref(),
        )
    }
}
#[cfg(feature = "core")]
impl<'a, M: Default + Clone> From<&'a Client<M>> for QueryAuth<'a> {
    #[inline]
    fn from(client: &'a Client<M>) -> Self {
        Self::new(
            client.get_key(),
            client.get_secret(),
            client.as_ref(),
            client.as_ref(),
        )
    }
}

impl<'a> QueryAuth<'a> {
    /// 初始化 QueryAuth
    #[inline]
    pub fn new(
        access_key_id: &'a KeyId,
        access_secret_key: &'a KeySecret,
        endpoint: &'a EndPoint,
        bucket: &'a BucketName,
    ) -> Self {
        Self {
            access_key_id,
            access_secret_key,
            endpoint,
            bucket,
        }
    }

    /// 通过 BucketBase 初始化
    #[cfg(feature = "core")]
    #[inline]
    pub fn new_with_bucket(
        access_key_id: &'a KeyId,
        access_secret_key: &'a KeySecret,
        base: &'a BucketBase,
    ) -> Self {
        Self::new(
            access_key_id,
            access_secret_key,
            base.as_ref(),
            base.as_ref(),
        )
    }
    fn get_resource(&self, path: &ObjectPath) -> CanonicalizedResource {
        CanonicalizedResource::from_object_str(self.bucket.as_ref(), path.as_ref())
    }
    fn get_url(&self, path: &ObjectPath) -> Url {
        let mut url = url_from_bucket(self.endpoint, self.bucket);
        url.set_object_path(path);
        url
    }

    fn sign_string(&self, path: &ObjectPath, expires: i64) -> String {
        const METHOD: &str = "GET";
        const LN3: &str = "\n\n\n";
        const LN: &str = "\n";

        let p = self.get_resource(path);

        const fn len(path: &str) -> usize {
            METHOD.len() + LN.len() + LN3.len() + 10 + path.len()
        }

        let mut string = String::with_capacity(len(p.as_ref()));
        string += METHOD;
        string += LN3;
        string += &expires.to_string();
        string += LN;
        string += p.as_ref();
        string
    }
    fn signature(&self, path: &ObjectPath, expires: i64) -> String {
      #![allow(clippy::unwrap_used)]
        self.access_secret_key
            .encryption_string(self.sign_string(path, expires))
            .unwrap()
    }

    /// 转化为带签名完整 url
    pub fn to_url(&self, path: &ObjectPath, expires: i64) -> Url {
        let mut url = self.get_url(path);
        self.signature_url(&mut url, path, expires);
        url
    }

    /// 为指定的 url 附加签名信息
    pub fn signature_url(&self, url: &mut Url, path: &ObjectPath, expires: i64) {
        const KEY: &str = "OSSAccessKeyId";
        const EXPIRES: &str = "Expires";
        const SIGNATURE: &str = "Signature";

        url.query_pairs_mut()
            .clear()
            .append_pair(KEY, self.access_key_id.as_ref())
            .append_pair(EXPIRES, &expires.to_string())
            .append_pair(SIGNATURE, &self.signature(path, expires));
    }
}

#[cfg(feature = "core")]
#[cfg(test)]
mod test {
    use url::Url;

    use crate::{
        config::{BucketBase, Config},
        BucketName, Client, EndPoint,
    };

    use super::QueryAuth;

    fn init_config() -> Config {
        Config::new(
            "foo",
            "foo2",
            EndPoint::CN_QINGDAO,
            BucketName::new("aaa").unwrap(),
        )
    }

    #[test]
    fn from_client() {
        let client = Client::new(
            "foo".into(),
            "foo2".into(),
            EndPoint::CN_QINGDAO,
            "aaa".parse().unwrap(),
        );

        let auth = QueryAuth::from(&client);
        assert_eq!(auth.access_key_id.as_ref(), "foo");
        assert_eq!(auth.access_secret_key.as_str(), "foo2");
        assert_eq!(auth.endpoint, &EndPoint::CN_QINGDAO);
        assert_eq!(auth.bucket.as_ref(), "aaa");
    }

    #[test]
    fn new_with_bucket() {
        let key = "foo".into();
        let secret = "foo2".into();
        let base = BucketBase::new("aaa".parse().unwrap(), EndPoint::CN_QINGDAO);

        let auth = QueryAuth::new_with_bucket(&key, &secret, &base);
        assert_eq!(auth.access_key_id.as_ref(), "foo");
        assert_eq!(auth.access_secret_key.as_str(), "foo2");
        assert_eq!(auth.endpoint, &EndPoint::CN_QINGDAO);
        assert_eq!(auth.bucket.as_ref(), "aaa");
    }

    #[test]
    fn get_resource() {
        let config = init_config();
        let auth = QueryAuth::from(&config);
        let res = auth.get_resource(&"img.png".parse().unwrap());
        assert_eq!(res.as_ref(), "/aaa/img.png");
    }

    #[test]
    fn get_url() {
        let config = init_config();
        let auth = QueryAuth::from(&config);
        let url = auth.get_url(&"img.png".parse().unwrap());
        assert_eq!(
            url.as_str(),
            "https://aaa.oss-cn-qingdao.aliyuncs.com/img.png"
        );
    }

    #[test]
    fn sign_string() {
        let config = init_config();
        let auth = QueryAuth::from(&config);
        let string = auth.sign_string(&"img.png".parse().unwrap(), 1200);
        assert_eq!(string, "GET\n\n\n1200\n/aaa/img.png");
    }

    #[test]
    fn signature() {
        let config = init_config();
        let auth = QueryAuth::from(&config);
        let string = auth.signature(&"img.png".parse().unwrap(), 1200);
        assert_eq!(string, "EQQzNJZptBDl8xJ6n2mQRG7oxkY=");
    }

    #[test]
    fn to_url() {
        let config = init_config();
        let auth = QueryAuth::from(&config);
        let string = auth.to_url(&"img.png".parse().unwrap(), 1200);
        assert_eq!(string.as_str(), "https://aaa.oss-cn-qingdao.aliyuncs.com/img.png?OSSAccessKeyId=foo&Expires=1200&Signature=EQQzNJZptBDl8xJ6n2mQRG7oxkY%3D");
    }

    #[test]
    fn signature_url() {
        let config = init_config();
        let auth = QueryAuth::from(&config);
        let mut url: Url = "https://example.com/image2.png".parse().unwrap();
        auth.signature_url(&mut url, &"img.png".parse().unwrap(), 1200);
        assert_eq!(url.as_str(), "https://example.com/image2.png?OSSAccessKeyId=foo&Expires=1200&Signature=EQQzNJZptBDl8xJ6n2mQRG7oxkY%3D");
    }
}
