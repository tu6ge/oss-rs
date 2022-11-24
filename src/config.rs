use std::{
    borrow::Cow,
    env::{self, VarError},
};

use reqwest::Url;
use std::fmt;
use thiserror::Error;

#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
use crate::{
    builder::{ArcPointer, PointerFamily},
    types::{BucketName, EndPoint, InvalidBucketName, InvalidEndPoint, KeyId, KeySecret},
};

pub struct Config {
    key: KeyId,
    secret: KeySecret,
    endpoint: EndPoint,
    bucket: BucketName,
}

impl Config {
    pub fn new<ID, S, E, B>(key: ID, secret: S, endpoint: E, bucket: B) -> Config
    where
        ID: Into<KeyId>,
        S: Into<KeySecret>,
        E: Into<EndPoint>,
        B: Into<BucketName>,
    {
        Config {
            key: key.into(),
            secret: secret.into(),
            endpoint: endpoint.into(),
            bucket: bucket.into(),
        }
    }

    pub(crate) fn get_all(self) -> (KeyId, KeySecret, BucketName, EndPoint) {
        (self.key, self.secret, self.bucket, self.endpoint)
    }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum InvalidConfig {
    #[error("{0}")]
    EndPoint(#[from] InvalidEndPoint),

    #[error("{0}")]
    BucketName(#[from] InvalidBucketName),

    #[error("{0}")]
    VarError(#[from] VarError),
}

// impl Error for InvalidConfig{}

// impl fmt::Display for InvalidConfig {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "endpoint must like with https://xxx.aliyuncs.com")
//     }
// }

#[derive(Debug, Clone, Default)]
pub struct BucketBase {
    endpoint: EndPoint,
    name: BucketName,
}

impl BucketBase {
    pub fn new(name: BucketName, endpoint: EndPoint) -> Self {
        Self { name, endpoint }
    }

    /// 通过域名获取
    /// 举例
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// # use aliyun_oss_client::types::EndPoint;
    /// let bucket = BucketBase::from_str("abc.oss-cn-shanghai.aliyuncs.com");
    /// assert!(bucket.is_ok());
    /// let bucket = bucket.unwrap();
    /// assert_eq!(bucket.name(), "abc");
    /// assert_eq!(bucket.endpoint(), EndPoint::CnShanghai);
    /// ```
    pub fn from_str(domain: &'static str) -> Result<Self, InvalidBucketBase> {
        fn valid_character(c: char) -> bool {
            match c {
                _ if c.is_ascii_lowercase() => true,
                _ if c.is_numeric() => true,
                '-' => true,
                '.' => true,
                _ => false,
            }
        }
        if !domain.chars().all(valid_character) {
            return Err(InvalidBucketBase::Tacitly);
        }

        let (bucket, endpoint) = match domain.split_once('.') {
            Some(v) => v,
            None => return Err(InvalidBucketBase::Tacitly),
        };

        Ok(Self {
            name: BucketName::new(bucket)?,
            endpoint: EndPoint::new(endpoint)?,
        })
    }

    /// 通过环境变量初始化
    pub fn from_env() -> Result<Self, InvalidConfig> {
        let endpoint = env::var("ALIYUN_ENDPOINT").map_err(InvalidConfig::from)?;
        let bucket = env::var("ALIYUN_BUCKET").map_err(InvalidConfig::from)?;

        Ok(Self {
            name: BucketName::new(bucket)?,
            endpoint: endpoint.try_into().map_err(InvalidConfig::from)?,
        })
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn endpoint(self) -> EndPoint {
        self.endpoint
    }

    /// 设置 bucket name
    ///
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// let mut bucket = BucketBase::default();
    /// assert!(bucket.set_name("abc").is_ok());
    /// assert_eq!(bucket.name(), "abc");
    /// ```
    pub fn set_name<N: TryInto<BucketName>>(&mut self, name: N) -> Result<(), N::Error> {
        self.name = name.try_into()?;
        Ok(())
    }

    pub fn set_endpoint<E: TryInto<EndPoint>>(&mut self, endpoint: E) -> Result<(), E::Error> {
        self.endpoint = endpoint.try_into()?;
        Ok(())
    }

    /// 获取url
    /// 举例
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// let mut bucket = BucketBase::default();
    /// bucket.set_name("abc");
    /// bucket.set_endpoint("shanghai");
    /// let url = bucket.to_url();
    /// assert_eq!(url.as_str(), "https://abc.oss-cn-shanghai.aliyuncs.com/");
    ///
    /// use std::env::set_var;
    /// set_var("ALIYUN_OSS_INTERNAL", "true");
    /// let mut bucket = BucketBase::default();
    /// bucket.set_name("abc");
    /// bucket.set_endpoint("shanghai");
    /// let url = bucket.to_url();
    /// assert_eq!(url.as_str(), "https://abc.oss-cn-shanghai-internal.aliyuncs.com/");
    /// ```
    ///
    /// > 因为 BucketName,EndPoint 声明时已做限制,所以 BucketBase 可以安全的转换成 url
    pub fn to_url(&self) -> Url {
        let endpoint = self.endpoint.to_url();
        let url = endpoint.to_string();
        let name_str = self.name.to_string();

        let mut name = String::from("https://");
        name.push_str(&name_str);
        name.push('.');

        let url = url.replace("https://", &name);
        Url::parse(&url).unwrap()
    }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum InvalidBucketBase {
    #[error("bucket url must like with https://yyy.xxx.aliyuncs.com")]
    Tacitly,

    #[error("{0}")]
    EndPoint(#[from] InvalidEndPoint),

    #[error("{0}")]
    BucketName(#[from] InvalidBucketName),
}

#[derive(Debug, Clone)]
pub struct ObjectBase<PointerSel: PointerFamily = ArcPointer> {
    bucket: PointerSel::Bucket,
    path: ObjectPath,
}

impl<T: PointerFamily> Default for ObjectBase<T> {
    fn default() -> Self {
        Self::new(T::Bucket::default(), "")
    }
}

impl<T: PointerFamily> ObjectBase<T> {
    pub fn new<P>(bucket: T::Bucket, path: P) -> Self
    where
        P: Into<ObjectPath>,
    {
        Self {
            bucket,
            path: path.into(),
        }
    }

    pub fn set_bucket(&mut self, bucket: T::Bucket) {
        self.bucket = bucket;
    }

    pub fn set_path<P: Into<ObjectPath>>(&mut self, path: P) {
        self.path = path.into();
    }

    pub fn path(&self) -> ObjectPath {
        self.path.to_owned()
    }
}

pub trait GetObjectInfo {
    fn bucket_name(&self) -> &str;
    fn path(&self) -> &str;
}

impl GetObjectInfo for ObjectBase {
    fn bucket_name(&self) -> &str {
        self.bucket.name()
    }

    fn path(&self) -> &str {
        self.path.as_ref()
    }
}

#[cfg(feature = "blocking")]
impl GetObjectInfo for ObjectBase<RcPointer> {
    fn bucket_name(&self) -> &str {
        self.bucket.name()
    }

    fn path(&self) -> &str {
        self.path.as_ref()
    }
}

/// OSS Object 存储对象的路径
/// 不带前缀 `/`  
/// TODO PartialEq
#[derive(Debug, Clone)]
pub struct ObjectPath(Cow<'static, str>);

impl AsRef<str> for ObjectPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObjectPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.to_owned())
    }
}

impl Default for ObjectPath {
    fn default() -> Self {
        Self(Cow::Borrowed(""))
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

    pub fn to_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ObjectPath {
    fn from(val: String) -> Self {
        Self(val.into())
    }
}

impl Into<String> for ObjectPath {
    fn into(self) -> String {
        self.0.to_owned().to_string()
    }
}

impl From<&'static str> for ObjectPath {
    fn from(url: &'static str) -> Self {
        Self::from_static(url)
    }
}

/// 将 object 的路径拼接到 Url 上去
pub trait UrlObjectPath {
    fn set_object_path(&mut self, path: &ObjectPath);
}

impl UrlObjectPath for Url {
    fn set_object_path(&mut self, path: &ObjectPath) {
        let inner_path: String = path.clone().into();
        self.set_path(&inner_path);
    }
}
