//! # 在 Url (即 query) 中包含签名
//!
//! [aliyun docs](https://help.aliyun.com/document_detail/31952.html)

use url::Url;

use crate::{
    config::get_url_resource, types::CanonicalizedResource, BucketName, EndPoint, KeyId, KeySecret,
    ObjectPath,
};

/// Object struct
pub struct Object {
    endpoint: EndPoint,
    bucket: BucketName,
    path: ObjectPath,
}

/// Query 签名器
pub struct QueryAuth<CanRes: AsRef<str> = CanonicalizedResource> {
    url: Option<Url>,
    resource: CanRes,
    access_key_id: KeyId,
    access_secret_key: KeySecret,
    expires: i64,
}

impl Object {
    /// 初始化 Object
    pub fn new(endpoint: EndPoint, bucket: BucketName, path: ObjectPath) -> Self {
        Self {
            endpoint,
            bucket,
            path,
        }
    }
    fn get_url_resource(&self) -> (Url, CanonicalizedResource) {
        get_url_resource(&self.endpoint, &self.bucket, &self.path)
    }
}

impl QueryAuth {
    /// 根据 object 初始化 struct
    #[inline]
    pub fn new_with_object(
        object: Object,
        access_key_id: KeyId,
        access_secret_key: KeySecret,
        expires: i64,
    ) -> Self {
        let (url, resource) = object.get_url_resource();
        Self::new(
            Some(url),
            resource,
            access_key_id,
            access_secret_key,
            expires,
        )
    }

    /// 初始化方法
    #[inline]
    pub fn new(
        url: Option<Url>,
        resource: CanonicalizedResource,
        access_key_id: KeyId,
        access_secret_key: KeySecret,
        expires: i64,
    ) -> Self {
        Self {
            url,
            resource,
            access_key_id,
            access_secret_key,
            expires,
        }
    }
    fn sign_string(&self) -> String {
        const METHOD: &str = "GET";
        const LN3: &str = "\n\n\n";
        const LN: &str = "\n";

        let mut string = String::with_capacity({
            METHOD.len() + LN.len() + LN3.len() + 10 + self.resource.as_ref().len()
        });
        string += METHOD;
        string += LN3;
        string += &self.expires.to_string();
        string += LN;
        string += self.resource.as_ref();
        string
    }
    fn signature(&self) -> String {
        self.access_secret_key
            .encryption_string(self.sign_string())
            .unwrap()
    }

    /// 转化为带签名完整 url
    pub fn to_url(&self) -> Option<Url> {
        self.url.clone().map(|u| {
            let mut url = u;
            self.signature_url(&mut url);
            url
        })
    }

    /// 为指定的 url 附加签名信息
    pub fn signature_url(&self, url: &mut Url) {
        const KEY: &str = "OSSAccessKeyId";
        const EXPIRES: &str = "Expires";
        const SIGNATURE: &str = "Signature";

        url.query_pairs_mut()
            .clear()
            .append_pair(KEY, self.access_key_id.as_ref())
            .append_pair(EXPIRES, &self.expires.to_string())
            .append_pair(SIGNATURE, &self.signature());
    }
}

#[cfg(test)]
mod test {
    use crate::{EndPoint, ObjectPath};

    use super::{Object, QueryAuth};

    fn init_object(path: ObjectPath) -> crate::Result<Object> {
        let bucket = "foo";

        Ok(Object::new(
            EndPoint::CN_QINGDAO,
            bucket.parse().unwrap(),
            path,
        ))
    }

    fn init_auth(path: ObjectPath, expires: i64) -> crate::Result<QueryAuth> {
        let object = init_object(path)?;
        let key_id = "key_id";
        let key_secret = "secret_id";

        Ok(QueryAuth::new_with_object(
            object,
            key_id.into(),
            key_secret.into(),
            expires,
        ))
    }

    #[test]
    fn get_url_resource() {
        let object = init_object("abc.png".parse().unwrap()).unwrap();

        let (url, res) = object.get_url_resource();
        assert_eq!(
            url.as_str(),
            "https://foo.oss-cn-qingdao.aliyuncs.com/abc.png"
        );
        assert_eq!(res.as_ref(), "/foo/abc.png");
    }

    #[test]
    fn sign_string() {
        let auth = init_auth("abc.png".parse().unwrap(), 100).unwrap();
        assert_eq!(auth.sign_string(), "GET\n\n\n100\n/foo/abc.png");
    }

    #[test]
    fn signature() {
        let auth = init_auth("abc.png".parse().unwrap(), 123).unwrap();
        assert_eq!(auth.signature(), "kcbz1nvZ9LwdlKC33Ml03K5DHkk=");
    }

    #[test]
    fn to_url() {
        let auth = init_auth("abc.png".parse().unwrap(), 123).unwrap();
        let url = auth.to_url().unwrap();
        assert_eq!(url.as_str(), "https://foo.oss-cn-qingdao.aliyuncs.com/abc.png?OSSAccessKeyId=key_id&Expires=123&Signature=kcbz1nvZ9LwdlKC33Ml03K5DHkk%3D");

        let auth = QueryAuth::new(None, "res".into(), "key".into(), "secret".into(), 10);
        assert!(auth.to_url().is_none());
    }

    #[test]
    fn signature_url() {
        let mut url = "https://example.com/abc.png".parse().unwrap();

        let auth = QueryAuth::new(None, "res".into(), "key".into(), "secret".into(), 10);
        auth.signature_url(&mut url);

        assert_eq!(url.as_str(), "https://example.com/abc.png?OSSAccessKeyId=key&Expires=10&Signature=zZriP4gLmCJ6WlkdVl4WPzsImkg%3D");
    }
}
