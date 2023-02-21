#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::{
    borrow::Cow,
    env::{self, VarError},
    fmt::Display,
    ops::{Add, AddAssign},
    path::Path,
    str::FromStr,
    sync::Arc,
};

use oss_derive::oss_gen_rc;
use reqwest::Url;
use std::fmt;
use thiserror::Error;

#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
use crate::{
    builder::{ArcPointer, PointerFamily},
    object::Object,
    types::{
        BucketName, CanonicalizedResource, EndPoint, InvalidBucketName, InvalidEndPoint, KeyId,
        KeySecret, QueryKey, QueryValue, UrlQuery,
    },
    Query,
};

/// OSS 配置信息
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Config {
    key: KeyId,
    secret: KeySecret,
    endpoint: EndPoint,
    bucket: BucketName,
}

impl Config {
    /// 初始化 OSS 配置信息
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

    /// 初始化 OSS 配置信息
    ///
    /// 支持更宽泛的输入类型
    pub fn try_new<ID, S, E, B>(
        key: ID,
        secret: S,
        endpoint: E,
        bucket: B,
    ) -> Result<Config, InvalidConfig>
    where
        ID: Into<KeyId>,
        S: Into<KeySecret>,
        E: TryInto<EndPoint>,
        <E as TryInto<EndPoint>>::Error: Into<InvalidConfig>,
        B: TryInto<BucketName>,
        <B as TryInto<BucketName>>::Error: Into<InvalidConfig>,
    {
        Ok(Config {
            key: key.into(),
            secret: secret.into(),
            endpoint: endpoint.try_into().map_err(|e| e.into())?,
            bucket: bucket.try_into().map_err(|e| e.into())?,
        })
    }

    pub(crate) fn get_all(self) -> (KeyId, KeySecret, BucketName, EndPoint) {
        (self.key, self.secret, self.bucket, self.endpoint)
    }
}

/// Config 错误信息集合
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum InvalidConfig {
    /// 非法的可用区
    #[error("{0}")]
    EndPoint(#[from] InvalidEndPoint),

    /// 非法的 bucket 名称
    #[error("{0}")]
    BucketName(#[from] InvalidBucketName),

    /// 非法的环境变量
    #[error("{0}")]
    VarError(#[from] VarError),
}

// impl Error for InvalidConfig{}

// impl fmt::Display for InvalidConfig {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "endpoint must like with https://xxx.aliyuncs.com")
//     }
// }

/// # Bucket 元信息
/// 包含所属 bucket 名以及所属的 endpoint
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BucketBase {
    endpoint: EndPoint,
    name: BucketName,
}

const HTTPS: &str = "https://";

impl FromStr for BucketBase {
    type Err = InvalidBucketBase;
    /// 通过域名获取
    /// 举例
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// # use aliyun_oss_client::types::EndPoint;
    /// let bucket: BucketBase = "abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap();
    /// assert_eq!(bucket.name(), "abc");
    /// assert_eq!(bucket.endpoint(), EndPoint::CnShanghai);
    ///
    /// assert!("abc*#!".parse::<BucketBase>().is_err());
    /// assert!("abc".parse::<BucketBase>().is_err());
    /// ```
    fn from_str(domain: &str) -> Result<Self, InvalidBucketBase> {
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

        let (bucket, endpoint) = domain.split_once('.').ok_or(InvalidBucketBase::Tacitly)?;

        Ok(Self {
            name: BucketName::from_static(bucket)?,
            endpoint: EndPoint::new(endpoint)?,
        })
    }
}

impl BucketBase {
    /// 初始化
    pub fn new(name: BucketName, endpoint: EndPoint) -> Self {
        Self { name, endpoint }
    }

    /// # 通过环境变量初始化
    /// ## 举例
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// use std::env::set_var;
    /// set_var("ALIYUN_ENDPOINT", "qingdao");
    /// set_var("ALIYUN_BUCKET", "foo1");
    /// assert!(BucketBase::from_env().is_ok());
    /// ```
    pub fn from_env() -> Result<Self, InvalidConfig> {
        let endpoint = env::var("ALIYUN_ENDPOINT").map_err(InvalidConfig::from)?;
        let bucket = env::var("ALIYUN_BUCKET").map_err(InvalidConfig::from)?;

        Ok(Self {
            name: BucketName::new(bucket)?,
            endpoint: endpoint.try_into().map_err(InvalidConfig::from)?,
        })
    }

    /// 返回 bucket 名称的引用
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// 返回 BucketName 引用
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// # use aliyun_oss_client::BucketName;
    /// use std::env::set_var;
    /// set_var("ALIYUN_ENDPOINT", "qingdao");
    /// set_var("ALIYUN_BUCKET", "foo1");
    /// assert_eq!(*BucketBase::from_env().unwrap().get_name(), BucketName::new("foo1").unwrap());
    /// ```
    #[inline]
    pub fn get_name(&self) -> &BucketName {
        &self.name
    }

    /// 获取 Bucket 元信息中的可用区
    #[inline]
    pub fn endpoint(self) -> EndPoint {
        self.endpoint
    }

    /// 设置 bucket name
    ///
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// use aliyun_oss_client::types::BucketName;
    /// let mut bucket = BucketBase::default();
    /// bucket.set_name("abc".parse::<BucketName>().unwrap());
    /// assert_eq!(bucket.name(), "abc");
    /// ```
    pub fn set_name<N: Into<BucketName>>(&mut self, name: N) {
        self.name = name.into();
    }

    /// 为 Bucket 元信息设置可用区
    pub fn set_endpoint<E: Into<EndPoint>>(&mut self, endpoint: E) {
        self.endpoint = endpoint.into();
    }

    /// 设置 bucket name
    ///
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// let mut bucket = BucketBase::default();
    /// assert!(bucket.try_set_name("abc").is_ok());
    /// assert_eq!(bucket.name(), "abc");
    /// ```
    pub fn try_set_name<N: TryInto<BucketName>>(&mut self, name: N) -> Result<(), N::Error> {
        self.name = name.try_into()?;
        Ok(())
    }

    /// 设置 endpoint
    ///
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// # use aliyun_oss_client::EndPoint;
    /// let mut bucket = BucketBase::default();
    /// assert!(bucket.try_set_endpoint("hangzhou").is_ok());
    /// assert_eq!(bucket.endpoint(), EndPoint::CnHangzhou);
    /// ```
    pub fn try_set_endpoint<E: TryInto<EndPoint>>(&mut self, endpoint: E) -> Result<(), E::Error> {
        self.endpoint = endpoint.try_into()?;
        Ok(())
    }

    /// 获取url
    /// 举例
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// use aliyun_oss_client::types::BucketName;
    /// let mut bucket = BucketBase::default();
    /// bucket.set_name("abc".parse::<BucketName>().unwrap());
    /// bucket.try_set_endpoint("shanghai").unwrap();
    /// let url = bucket.to_url();
    /// assert_eq!(url.as_str(), "https://abc.oss-cn-shanghai.aliyuncs.com/");
    ///
    /// use std::env::set_var;
    /// set_var("ALIYUN_OSS_INTERNAL", "true");
    /// let mut bucket = BucketBase::default();
    /// bucket.set_name("abc".parse::<BucketName>().unwrap());
    /// bucket.try_set_endpoint("shanghai").unwrap();
    /// let url = bucket.to_url();
    /// assert_eq!(
    ///     url.as_str(),
    ///     "https://abc.oss-cn-shanghai-internal.aliyuncs.com/"
    /// );
    /// ```
    ///
    /// > 因为 BucketName,EndPoint 声明时已做限制,所以 BucketBase 可以安全的转换成 url
    pub fn to_url(&self) -> Url {
        let endpoint = self.endpoint.to_url();
        let url = endpoint.to_string();
        let name_str = self.name.to_string();

        let mut name = String::from(HTTPS);
        name.push_str(&name_str);
        name.push('.');

        let url = url.replace(HTTPS, &name);
        Url::parse(&url).unwrap()
    }

    /// 根据查询参数，获取当前 bucket 的接口请求参数（ url 和 CanonicalizedResource）
    #[inline]
    pub fn get_url_resource(&self, query: &Query) -> (Url, CanonicalizedResource) {
        let mut url = self.to_url();
        url.set_search_query(query);

        let resource = CanonicalizedResource::from_bucket_query(self, query);

        (url, resource)
    }
}

/// Bucket 元信息的错误集
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum InvalidBucketBase {
    #[doc(hidden)]
    #[error("bucket url must like with https://yyy.xxx.aliyuncs.com")]
    Tacitly,

    #[doc(hidden)]
    #[error("{0}")]
    EndPoint(#[from] InvalidEndPoint),

    #[doc(hidden)]
    #[error("{0}")]
    BucketName(#[from] InvalidBucketName),
}

impl PartialEq<Url> for BucketBase {
    /// # 相等比较
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// use aliyun_oss_client::types::BucketName;
    /// use reqwest::Url;
    /// let mut bucket = BucketBase::default();
    /// bucket.set_name("abc".parse::<BucketName>().unwrap());
    /// bucket.try_set_endpoint("shanghai").unwrap();
    /// assert!(bucket == Url::parse("https://abc.oss-cn-shanghai.aliyuncs.com/").unwrap());
    /// ```
    #[inline]
    fn eq(&self, other: &Url) -> bool {
        &self.to_url() == other
    }
}

/// # Object 元信息
/// 包含所属 bucket endpoint 以及文件路径
#[derive(Debug, Clone)]
pub struct ObjectBase<PointerSel: PointerFamily = ArcPointer> {
    bucket: PointerSel::Bucket,
    path: ObjectPath,
}

impl<T: PointerFamily> Default for ObjectBase<T> {
    fn default() -> Self {
        Self {
            bucket: T::Bucket::default(),
            path: ObjectPath::default(),
        }
    }
}

impl<T: PointerFamily> ObjectBase<T> {
    /// 初始化 Object 元信息
    pub fn new<P>(bucket: T::Bucket, path: P) -> Result<Self, InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        <P as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        let path = path.try_into().map_err(|e| e.into())?;

        Ok(Self { bucket, path })
    }

    #[inline]
    pub(crate) fn new2(bucket: T::Bucket, path: ObjectPath) -> Self {
        Self { bucket, path }
    }

    /// 为 Object 元信息设置 bucket
    pub fn set_bucket(&mut self, bucket: T::Bucket) {
        self.bucket = bucket;
    }

    /// 为 Object 元信息设置文件路径
    pub fn set_path<P>(&mut self, path: P) -> Result<(), InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        <P as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        self.path = path.try_into().map_err(|e| e.into())?;

        Ok(())
    }

    /// 返回 Object 元信息的文件路径
    pub fn path(&self) -> ObjectPath {
        self.path.to_owned()
    }
}

#[oss_gen_rc]
impl ObjectBase<ArcPointer> {
    #[doc(hidden)]
    #[inline]
    pub fn from_bucket<P>(bucket: BucketBase, path: P) -> Result<Self, InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        <P as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        Ok(Self {
            bucket: Arc::new(bucket),
            path: path.try_into().map_err(|e| e.into())?,
        })
    }

    #[doc(hidden)]
    #[inline]
    pub fn try_from_bucket<B, P>(bucket: B, path: P) -> Result<Self, InvalidObjectBase>
    where
        B: TryInto<BucketBase>,
        P: TryInto<ObjectPath>,
        <B as TryInto<BucketBase>>::Error: Into<InvalidObjectBase>,
        <P as TryInto<ObjectPath>>::Error: Into<InvalidObjectBase>,
    {
        Ok(Self {
            bucket: Arc::new(bucket.try_into().map_err(|e| e.into())?),
            path: path.try_into().map_err(|e| e.into())?,
        })
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_ref_bucket<P>(bucket: Arc<BucketBase>, path: P) -> Result<Self, InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        <P as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        Ok(Self {
            bucket,
            path: path.try_into().map_err(|e| e.into())?,
        })
    }

    /// TODO bucket name 可能会panic
    #[inline]
    pub fn from_bucket_name<B, E, P>(
        bucket: B,
        endpoint: E,
        path: P,
    ) -> Result<Self, InvalidObjectBase>
    where
        B: TryInto<BucketName>,
        <B as TryInto<BucketName>>::Error: Into<InvalidObjectBase>,
        E: TryInto<EndPoint>,
        <E as TryInto<EndPoint>>::Error: Into<InvalidObjectBase>,
        P: TryInto<ObjectPath>,
        <P as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        let bucket = BucketBase::new(
            bucket.try_into().map_err(|e| e.into())?,
            endpoint.try_into().map_err(|e| e.into())?,
        );
        Self::from_bucket(bucket, path).map_err(|e| e.into())
    }

    #[doc(hidden)]
    #[inline]
    pub fn bucket_name(&self) -> &BucketName {
        self.bucket.get_name()
    }

    /// 根据提供的查询参数信息，获取当前 object 对应的接口请求参数（ url 和 CanonicalizedResource）
    #[inline]
    pub fn get_url_resource<Q: IntoIterator<Item = (QueryKey, QueryValue)>>(
        &self,
        query: Q,
    ) -> (Url, CanonicalizedResource) {
        let mut url = self.bucket.to_url();
        url.set_object_path(&self.path);

        let resource =
            CanonicalizedResource::from_object((self.bucket.name(), self.path.as_ref()), query);

        (url, resource)
    }
}

#[oss_gen_rc]
impl PartialEq<ObjectBase<ArcPointer>> for ObjectBase<ArcPointer> {
    #[inline]
    fn eq(&self, other: &ObjectBase<ArcPointer>) -> bool {
        *self.bucket == *other.bucket && self.path == other.path
    }
}

impl<T: PointerFamily> PartialEq<&str> for ObjectBase<T> {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::config::ObjectBase;
    /// # use aliyun_oss_client::config::BucketBase;
    /// # use aliyun_oss_client::builder::ArcPointer;
    /// # use std::sync::Arc;
    /// use aliyun_oss_client::types::BucketName;
    /// let mut path = ObjectBase::<ArcPointer>::default();
    /// path.set_path("abc");
    /// assert!(path == "abc");
    ///
    /// let mut bucket = BucketBase::default();
    /// bucket.set_name("def".parse::<BucketName>().unwrap());
    /// bucket.try_set_endpoint("shanghai").unwrap();
    /// path.set_bucket(Arc::new(bucket));
    /// assert!(path == "abc");
    /// ```
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.path == other
    }
}

/// Object 元信息的错误集
#[derive(Debug)]
pub enum InvalidObjectBase {
    #[doc(hidden)]
    Bucket(InvalidBucketBase),
    #[doc(hidden)]
    Path(InvalidObjectPath),
}

impl Display for InvalidObjectBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InvalidObjectBase::*;
        match self {
            Bucket(b) => write!(f, "{}", b),
            Path(p) => write!(f, "{}", p),
        }
    }
}

impl From<InvalidBucketBase> for InvalidObjectBase {
    fn from(value: InvalidBucketBase) -> Self {
        Self::Bucket(value)
    }
}

impl From<InvalidObjectPath> for InvalidObjectBase {
    fn from(value: InvalidObjectPath) -> Self {
        Self::Path(value)
    }
}

impl From<InvalidBucketName> for InvalidObjectBase {
    fn from(value: InvalidBucketName) -> Self {
        Self::Bucket(value.into())
    }
}

impl From<InvalidEndPoint> for InvalidObjectBase {
    fn from(value: InvalidEndPoint) -> Self {
        Self::Bucket(value.into())
    }
}

/// OSS Object 存储对象的路径
/// 不带前缀 `/`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectPath(Cow<'static, str>);

impl AsRef<str> for ObjectPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObjectPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}

impl Default for ObjectPath {
    /// 默认值
    /// ```
    /// # use aliyun_oss_client::config::ObjectPath;
    /// let path = ObjectPath::default();
    /// assert!(path == "");
    /// ```
    fn default() -> Self {
        Self(Cow::Borrowed(""))
    }
}

impl PartialEq<&str> for ObjectPath {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::config::ObjectPath;
    /// let path = ObjectPath::new("abc").unwrap();
    /// assert!(path == "abc");
    /// ```
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.0 == other
    }
}

impl PartialEq<ObjectPath> for &str {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::config::ObjectPath;
    /// let path = ObjectPath::new("abc").unwrap();
    /// assert!("abc" == path);
    /// ```
    #[inline]
    fn eq(&self, other: &ObjectPath) -> bool {
        self == &other.0
    }
}

impl PartialEq<String> for ObjectPath {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::config::ObjectPath;
    /// let path = ObjectPath::new("abc").unwrap();
    /// assert!(path == "abc".to_string());
    /// ```
    #[inline]
    fn eq(&self, other: &String) -> bool {
        &self.0.clone() == other
    }
}

impl PartialEq<ObjectPath> for String {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::config::ObjectPath;
    /// let path = ObjectPath::new("abc").unwrap();
    /// assert!("abc".to_string() == path);
    /// ```
    #[inline]
    fn eq(&self, other: &ObjectPath) -> bool {
        self == &other.0.clone()
    }
}

impl ObjectPath {
    /// Creates a new `ObjectPath` from the given string.
    /// ```
    /// # use aliyun_oss_client::config::ObjectPath;
    /// assert!(ObjectPath::new("abc.jpg").is_ok());
    /// assert!(ObjectPath::new("abc/def.jpg").is_ok());
    /// assert!(ObjectPath::new("/").is_err());
    /// assert!(ObjectPath::new("/abc").is_err());
    /// assert!(ObjectPath::new("abc/").is_err());
    /// assert!(ObjectPath::new(".abc").is_err());
    /// assert!(ObjectPath::new("../abc").is_err());
    /// assert!(ObjectPath::new(r"aaa\abc").is_err());
    /// ```
    pub fn new(val: impl Into<Cow<'static, str>>) -> Result<Self, InvalidObjectPath> {
        let val = val.into();
        if val.starts_with('/') || val.starts_with('.') || val.ends_with('/') {
            return Err(InvalidObjectPath);
        }
        if !val.chars().all(|c| c != '\\') {
            return Err(InvalidObjectPath);
        }
        Ok(Self(val))
    }

    /// # Safety
    ///
    /// Const function that creates a new `ObjectPath` from a static str.
    /// ```
    /// # use aliyun_oss_client::config::ObjectPath;
    /// let path = unsafe { ObjectPath::from_static("abc") };
    /// assert!(path == "abc");
    /// ```
    pub const unsafe fn from_static(secret: &'static str) -> Self {
        Self(Cow::Borrowed(secret))
    }
}

impl TryFrom<String> for ObjectPath {
    type Error = InvalidObjectPath;
    /// ```
    /// # use aliyun_oss_client::config::ObjectPath;
    /// let path: ObjectPath = String::from("abc").try_into().unwrap();
    /// assert!(path == "abc");
    /// ```
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Self::new(val)
    }
}

impl<'a> TryFrom<&'a str> for ObjectPath {
    type Error = InvalidObjectPath;
    fn try_from(val: &'a str) -> Result<Self, Self::Error> {
        Self::new(val.to_owned())
    }
}

impl FromStr for ObjectPath {
    type Err = InvalidObjectPath;
    /// ```
    /// # use aliyun_oss_client::config::ObjectPath;
    /// use std::str::FromStr;
    /// let path: ObjectPath = "img1.jpg".parse().unwrap();
    /// assert!(path == "img1.jpg");
    /// assert!(ObjectPath::from_str("abc.jpg").is_ok());
    /// assert!(ObjectPath::from_str("abc/def.jpg").is_ok());
    /// assert!(ObjectPath::from_str("/").is_err());
    /// assert!(ObjectPath::from_str("/abc").is_err());
    /// assert!(ObjectPath::from_str("abc/").is_err());
    /// assert!(ObjectPath::from_str(".abc").is_err());
    /// assert!(ObjectPath::from_str("../abc").is_err());
    /// assert!(ObjectPath::from_str(r"aaa\abc").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('/') || s.starts_with('.') || s.ends_with('/') {
            return Err(InvalidObjectPath);
        }

        if !s.chars().all(|c| c != '\\') {
            return Err(InvalidObjectPath);
        }
        Ok(Self(Cow::Owned(s.to_owned())))
    }
}

impl TryFrom<&Path> for ObjectPath {
    type Error = InvalidObjectPath;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let val = value.to_str().ok_or(InvalidObjectPath)?;
        if std::path::MAIN_SEPARATOR != '/' {
            val.replace(std::path::MAIN_SEPARATOR, "/").parse()
        } else {
            val.parse()
        }
    }
}

impl<T: PointerFamily> From<Object<T>> for ObjectPath {
    #[inline]
    fn from(obj: Object<T>) -> Self {
        obj.base.path
    }
}

/// 不合法的文件路径
#[derive(Debug, Error)]
pub struct InvalidObjectPath;

impl Display for InvalidObjectPath {
    /// ```
    /// # use aliyun_oss_client::config::InvalidObjectPath;
    /// assert_eq!(format!("{}", InvalidObjectPath), "invalid object path");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid object path")
    }
}

/// 将 object 的路径拼接到 Url 上去
pub trait UrlObjectPath {
    /// 为 Url 添加方法
    fn set_object_path(&mut self, path: &ObjectPath);
}

impl UrlObjectPath for Url {
    fn set_object_path(&mut self, path: &ObjectPath) {
        self.set_path(path.as_ref());
    }
}

/// OSS Object 对象路径的前缀目录
/// 不带前缀 `/`, 必须以 `/` 结尾
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDir<'a>(Cow<'a, str>);

impl AsRef<str> for ObjectDir<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObjectDir<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}

// impl Default for ObjectDir<'_> {
//     /// 默认值
//     /// ```
//     /// # use aliyun_oss_client::config::ObjectDir;
//     /// let path = ObjectDir::default();
//     /// assert!(path == "default/");
//     /// ```
//     fn default() -> Self {
//         Self(Cow::Borrowed("default/"))
//     }
// }

impl PartialEq<&str> for ObjectDir<'_> {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::config::ObjectDir;
    /// let path = ObjectDir::new("abc/").unwrap();
    /// assert!(path == "abc/");
    /// ```
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.0 == other
    }
}

impl PartialEq<ObjectDir<'_>> for &str {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::config::ObjectDir;
    /// let path = ObjectDir::new("abc/").unwrap();
    /// assert!("abc/" == path);
    /// ```
    #[inline]
    fn eq(&self, other: &ObjectDir<'_>) -> bool {
        self == &other.0
    }
}

impl PartialEq<String> for ObjectDir<'_> {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::config::ObjectDir;
    /// let path = ObjectDir::new("abc/").unwrap();
    /// assert!(path == "abc/".to_string());
    /// ```
    #[inline]
    fn eq(&self, other: &String) -> bool {
        &self.0.clone() == other
    }
}

impl PartialEq<ObjectDir<'_>> for String {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::config::ObjectDir;
    /// let path = ObjectDir::new("abc/").unwrap();
    /// assert!("abc/".to_string() == path);
    /// ```
    #[inline]
    fn eq(&self, other: &ObjectDir) -> bool {
        self == &other.0.clone()
    }
}

impl<'a, 'b> Add<ObjectDir<'b>> for ObjectDir<'a> {
    type Output = ObjectDir<'a>;

    /// # 支持 ObjectDir 相加运算
    /// ```
    /// # use aliyun_oss_client::config::ObjectDir;
    /// let dir1 = ObjectDir::new("dir1/").unwrap();
    /// let dir2 = ObjectDir::new("dir2/").unwrap();
    /// let full_dir = ObjectDir::new("dir1/dir2/").unwrap();
    ///
    /// assert_eq!(dir1 + dir2, full_dir);
    /// ```
    fn add(self, rhs: ObjectDir<'b>) -> Self::Output {
        let mut string = self.as_ref().to_string();

        string += rhs.as_ref();
        ObjectDir(Cow::Owned(string))
    }
}

impl<'a, 'b> AddAssign<ObjectDir<'b>> for ObjectDir<'a> {
    /// # 支持 ObjectDir 相加运算
    /// ```
    /// # use aliyun_oss_client::config::ObjectDir;
    /// let mut dir1 = ObjectDir::new("dir1/").unwrap();
    /// let dir2 = ObjectDir::new("dir2/").unwrap();
    /// let full_dir = ObjectDir::new("dir1/dir2/").unwrap();
    ///
    /// dir1 += dir2;
    /// assert_eq!(dir1, full_dir);
    /// ```
    fn add_assign(&mut self, rhs: ObjectDir<'b>) {
        let mut string = self.as_ref().to_string();

        string += rhs.as_ref();
        *self = ObjectDir(Cow::Owned(string));
    }
}

impl<'a> Add<ObjectPath> for ObjectDir<'a> {
    type Output = ObjectPath;

    /// # 支持 ObjectDir 与 ObjectPath 相加运算
    /// ```
    /// # use aliyun_oss_client::config::{ObjectDir, ObjectPath};
    /// let dir1 = ObjectDir::new("dir1/").unwrap();
    /// let file1 = ObjectPath::new("img1.png").unwrap();
    /// let full_file = ObjectPath::new("dir1/img1.png").unwrap();
    ///
    /// assert_eq!(dir1 + file1, full_file);
    /// ```
    fn add(self, rhs: ObjectPath) -> Self::Output {
        let mut string = self.as_ref().to_string();

        string += rhs.as_ref();
        ObjectPath(Cow::Owned(string))
    }
}

impl<'a> ObjectDir<'a> {
    /// Creates a new `ObjectPath` from the given string.
    /// ```
    /// # use aliyun_oss_client::config::ObjectDir;
    /// assert!(ObjectDir::new("abc/").is_ok());
    /// assert!(ObjectDir::new("abc/def/").is_ok());
    /// assert!(ObjectDir::new("/").is_err());
    /// assert!(ObjectDir::new("/abc/").is_err());
    /// assert!(ObjectDir::new(".abc/").is_err());
    /// assert!(ObjectDir::new("../abc/").is_err());
    /// assert!(ObjectDir::new(r"aaa\abc/").is_err());
    /// ```
    pub fn new<'b: 'a>(val: impl Into<Cow<'b, str>>) -> Result<Self, InvalidObjectDir> {
        let val = val.into();
        if val.starts_with('/') || val.starts_with('.') || !val.ends_with('/') {
            return Err(InvalidObjectDir);
        }
        if !val.chars().all(|c| c != '\\') {
            return Err(InvalidObjectDir);
        }
        Ok(Self(val))
    }

    /// # Safety
    ///
    /// Const function that creates a new `ObjectPath` from a static str.
    /// ```
    /// # use aliyun_oss_client::config::ObjectDir;
    /// let path = unsafe { ObjectDir::from_static("abc/") };
    /// assert!(path == "abc/");
    /// ```
    pub const unsafe fn from_static(secret: &'a str) -> Self {
        Self(Cow::Borrowed(secret))
    }
}

impl TryFrom<String> for ObjectDir<'_> {
    type Error = InvalidObjectDir;
    /// ```
    /// # use aliyun_oss_client::config::ObjectDir;
    /// let path: ObjectDir = String::from("abc/").try_into().unwrap();
    /// assert!(path == "abc/");
    /// ```
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Self::new(val)
    }
}

impl<'a: 'b, 'b> TryFrom<&'a str> for ObjectDir<'b> {
    type Error = InvalidObjectDir;
    fn try_from(val: &'a str) -> Result<Self, Self::Error> {
        Self::new(val)
    }
}

impl FromStr for ObjectDir<'_> {
    type Err = InvalidObjectDir;
    /// ```
    /// # use aliyun_oss_client::config::ObjectDir;
    /// use std::str::FromStr;
    /// let path: ObjectDir = "path1/".parse().unwrap();
    /// assert!(path == "path1/");
    /// assert!(ObjectDir::from_str("abc/").is_ok());
    /// assert!(ObjectDir::from_str("abc/def/").is_ok());
    /// assert!(ObjectDir::from_str("/").is_err());
    /// assert!(ObjectDir::from_str("/abc/").is_err());
    /// assert!(ObjectDir::from_str(".abc/").is_err());
    /// assert!(ObjectDir::from_str("../abc/").is_err());
    /// assert!(ObjectDir::from_str(r"aaa\abc/").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('/') || s.starts_with('.') || !s.ends_with('/') {
            return Err(InvalidObjectDir);
        }

        if !s.chars().all(|c| c != '\\') {
            return Err(InvalidObjectDir);
        }
        Ok(Self(Cow::Owned(s.to_owned())))
    }
}

impl TryFrom<&Path> for ObjectDir<'_> {
    type Error = InvalidObjectDir;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let val = value.to_str().ok_or(InvalidObjectDir)?;
        if std::path::MAIN_SEPARATOR != '/' {
            val.replace(std::path::MAIN_SEPARATOR, "/").parse()
        } else {
            val.parse()
        }
    }
}

/// 不合法的文件路径
#[derive(Debug, Error)]
pub struct InvalidObjectDir;

impl Display for InvalidObjectDir {
    /// ```
    /// # use aliyun_oss_client::config::InvalidObjectDir;
    /// assert_eq!(format!("{}", InvalidObjectDir), "ObjectDir must end with `/`");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectDir must end with `/`")
    }
}

/// 给 Url 设置一个初始化方法，根据 OSS 的配置信息，返回文件的完整 OSS Url
pub trait OssFullUrl {
    /// 根据配置信息，计算完整的 Url
    fn from_oss(endpoint: &EndPoint, bucket: &BucketName, path: &ObjectPath) -> Self;
}

impl OssFullUrl for Url {
    fn from_oss(endpoint: &EndPoint, bucket: &BucketName, path: &ObjectPath) -> Self {
        let mut end_url = endpoint.to_url();

        let host = end_url.host_str();

        let mut name_str = bucket.to_string() + ".";

        let new_host = host.map(|h| {
            name_str.push_str(h);
            &*name_str
        });
        // 因为 endpoint 都是已知字符组成，bucket 也有格式要求，所以 unwrap 是安全的
        end_url.set_host(new_host).unwrap();

        end_url.set_object_path(path);

        end_url
    }
}

/// 文件夹下的子文件夹名，子文件夹下递归的所有文件和文件夹不包含在这里。
pub type CommonPrefixes = Vec<ObjectDir<'static>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_from_ref_bucket() {
        use std::env::set_var;
        set_var("ALIYUN_ENDPOINT", "qingdao");
        set_var("ALIYUN_BUCKET", "foo1");
        let object = ObjectBase::<ArcPointer>::from_ref_bucket(
            Arc::new(BucketBase::from_env().unwrap()),
            "img1.jpg",
        )
        .unwrap();

        assert_eq!(object.path(), "img1.jpg");
    }

    #[test]
    fn object_from_bucket_name() {
        let object =
            ObjectBase::<ArcPointer>::from_bucket_name("foo1", "qingdao", "img1.jpg").unwrap();

        assert_eq!(object.path(), "img1.jpg");
    }
}

#[cfg(feature = "blocking")]
#[cfg(test)]
mod blocking_tests {
    use crate::builder::RcPointer;

    use super::ObjectBase;

    fn crate_object_base(bucket: &'static str, path: &'static str) -> ObjectBase<RcPointer> {
        use std::rc::Rc;

        let bucket = bucket.parse().unwrap();

        let object = ObjectBase::<RcPointer>::new2(Rc::new(bucket), path.try_into().unwrap());
        object
    }

    #[test]
    fn test_get_object_info() {
        let object = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "bar");

        assert_eq!(object.bucket_name(), &"abc");
        assert_eq!(object.path(), "bar");
    }

    #[test]
    fn test_object_base_eq() {
        let object1 = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "bar");
        let object2 = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "bar");
        let object3 = crate_object_base("abc.oss-cn-qingdao.aliyuncs.com", "bar");
        let object4 = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "ba2");
        assert!(object1 == object2);
        assert!(object1 != object3);
        assert!(object1 != object4);
    }
}
