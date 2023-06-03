//! # 在 Url (即 query) 中包含签名
//!
//! [aliyun docs](https://help.aliyun.com/document_detail/31952.html)

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
use crate::config::Config;

#[cfg(feature = "core")]
impl<'a> From<&'a Config> for QueryAuth<'a> {
    fn from(config: &'a Config) -> Self {
        let (access_key_id, access_secret_key, bucket, endpoint) = config.get_all_ref();

        Self::new(access_key_id, access_secret_key, endpoint, bucket)
    }
}

impl<'a> QueryAuth<'a> {
    /// 初始化 QueryAuth
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
    fn get_resource(&self, path: &ObjectPath) -> CanonicalizedResource {
        CanonicalizedResource::from_object_str(self.bucket.as_ref(), path.as_ref())
    }
    fn get_url(&self, path: &ObjectPath) -> Url {
        let mut url = url_from_bucket(&self.endpoint, &self.bucket);
        url.set_object_path(path);
        url
    }

    fn sign_string(&self, path: &ObjectPath, expires: i64) -> String {
        const METHOD: &str = "GET";
        const LN3: &str = "\n\n\n";
        const LN: &str = "\n";

        let p = self.get_resource(path);

        let mut string =
            String::with_capacity(METHOD.len() + LN.len() + LN3.len() + 10 + p.as_ref().len());
        string += METHOD;
        string += LN3;
        string += &expires.to_string();
        string += LN;
        string += p.as_ref();
        string
    }
    fn signature(&self, path: &ObjectPath, expires: i64) -> String {
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

#[cfg(test)]
mod test {
    use crate::{EndPoint, ObjectPath};

    use super::QueryAuth;
}
