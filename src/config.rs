use std::{
    borrow::Cow
};

use reqwest::Url;

use crate::{types::{KeyId, KeySecret, EndPoint, BucketName, InvalidEndPoint}, errors::{OssResult,OssError}};

pub struct Config{
    key: KeyId,
    secret: KeySecret,
    endpoint: EndPoint,
    bucket: BucketName,
}

impl Config {
    pub fn new<ID, S, E, B>(
        key: ID, 
        secret: S, 
        endpoint: E, 
        bucket: B,
    ) ->Config
    where ID: Into<KeyId>,
    S: Into<KeySecret>,
    E: Into<EndPoint>,
    B: Into<BucketName>,
    {
        Config{
            key: key.into(),
            secret: secret.into(),
            endpoint: endpoint.into(),
            bucket: bucket.into(),
        }
    }

    pub fn key(&self) -> KeyId {
        self.key.clone()
    }

    pub fn secret(&self) -> KeySecret{
        self.secret.clone()
    }

    pub fn bucket(&self) -> BucketName{
        self.bucket.clone()
    }

    pub fn endpoint(&self) -> EndPoint{
        self.endpoint.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct BucketBase{
    endpoint: EndPoint,
    name: BucketName,
}

impl BucketBase {
    pub fn new(
        name: BucketName,
        endpoint: EndPoint,
    ) -> Self{
        Self{
            name,
            endpoint,
        }
    }

    pub fn name(&self) -> &str{
        self.name.as_ref()
    }

    pub fn set_name<N: Into<BucketName>>(&mut self, name: N) {
        self.name = name.into();
    }

    pub fn set_endpoint<S: Into<Cow<'static, str>>>(&mut self, endpoint: S) -> Result<(), InvalidEndPoint>{
        self.endpoint = EndPoint::new(endpoint)?;
        Ok(())
    }

    /// 获取url
    /// 举例
    /// ```
    /// use aliyun_oss_client::config::BucketBase;
    /// let bucket = BucketBase::new("abc".into(), "https://oss-cn-shanghai.aliyuncs.com".into());
    /// let url = bucket.to_url();
    /// assert!(url.is_ok());
    /// let url = url.unwrap();
    /// assert_eq!(url.as_str(), "https://abc.oss-cn-shanghai.aliyuncs.com/");
    /// ```
    pub fn to_url(&self) -> OssResult<Url>{
        let mut url = self.endpoint.to_url()?;

        let host = url.host_str().unwrap();
        let host = self.name.to_string() + "." + host;
        let res = url.set_host(Some(&host));

        if let Err(e) = res{
            return Err(OssError::Input(format!("set bucket url failed: {}", e)));
        }

        Ok(url)
    }
}

#[derive(Debug, Clone)]
pub struct ObjectBase {
    bucket: BucketBase,
    path: ObjectPath,
}

impl Default for ObjectBase{
    fn default() -> Self{
        Self::new(BucketBase::new("".into(),"".into()), "")
    }
}

impl ObjectBase {
    pub fn new<P>(bucket: BucketBase, path: P) -> Self
    where P: Into<ObjectPath>
    {
        Self{
            bucket,
            path: path.into(),
        }
    }

    pub fn set_bucket(&mut self, bucket: BucketBase){
        self.bucket = bucket;
    }

    pub fn bucket_name(&self) -> &str{
        self.bucket.name()
    }

    pub fn path(&self) -> &str {
        self.path.as_ref()
    }

    pub fn set_path<P: Into<ObjectPath>>(&mut self, path: P){
        self.path = path.into();
    }
}

/// OSS Object 存储对象的路径
/// 不带前缀 `/`  
#[derive(Debug, Clone)]
pub struct ObjectPath(
    Cow<'static, str>
);

impl AsRef<str> for ObjectPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ObjectPath {
    /// Creates a new `ObjectPath` from the given string.
    pub fn new(val: impl Into<Cow<'static, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `KeySecret` from a static str.
    pub const fn from_static(secret: &'static str) -> Self {
        Self(Cow::Borrowed(secret))
    }
}

impl From<String> for ObjectPath {
    fn from(val: String) -> Self {
        Self(val.into())
    }
}

impl From<&'static str> for ObjectPath {
    fn from(url: &'static str) -> Self {
        Self::from_static(url)
    }
}