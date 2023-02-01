#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::{
    borrow::Cow,
    env::{self, VarError},
    fmt::Display,
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
    types::{
        BucketName, CanonicalizedResource, EndPoint, InvalidBucketName, InvalidEndPoint, KeyId,
        KeySecret, QueryKey, QueryValue, UrlQuery,
    },
    Query,
};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
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

        let (bucket, endpoint) = domain
            .split_once('.')
            .ok_or_else(|| InvalidBucketBase::Tacitly)?;

        Ok(Self {
            name: BucketName::from_static(bucket)?,
            endpoint: EndPoint::new(endpoint)?,
        })
    }
}

impl BucketBase {
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

    pub fn set_bucket(&mut self, bucket: T::Bucket) {
        self.bucket = bucket;
    }

    pub fn set_path<P>(&mut self, path: P) -> Result<(), InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        <P as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        self.path = path.try_into().map_err(|e| e.into())?;

        Ok(())
    }

    pub fn path(&self) -> ObjectPath {
        self.path.to_owned()
    }
}

#[oss_gen_rc]
impl ObjectBase<ArcPointer> {
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

#[derive(Debug)]
pub enum InvalidObjectBase {
    Bucket(InvalidBucketBase),
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
    /// ```
    pub fn new(val: impl Into<Cow<'static, str>>) -> Result<Self, InvalidObjectPath> {
        let val = val.into();
        if val.starts_with('/') || val.starts_with('.') || val.ends_with('/') {
            return Err(InvalidObjectPath);
        }
        Ok(Self(val))
    }

    /// Const function that creates a new `ObjectPath` from a static str.
    /// ```
    /// # use aliyun_oss_client::config::ObjectPath;
    /// let path = unsafe{ ObjectPath::from_static("abc") };
    /// assert!(path == "abc");
    /// ```
    pub const unsafe fn from_static(secret: &'static str) -> Self {
        Self(Cow::Borrowed(secret))
    }

    pub fn to_str(&self) -> &str {
        &self.0
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
    /// ```
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('/') || s.starts_with('.') || s.ends_with('/') {
            return Err(InvalidObjectPath);
        }
        Ok(Self(Cow::Owned(s.to_owned())))
    }
}

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
    fn set_object_path(&mut self, path: &ObjectPath);
}

impl UrlObjectPath for Url {
    fn set_object_path(&mut self, path: &ObjectPath) {
        self.set_path(&path.to_string());
    }
}

/// 文件夹下的子文件夹名，子文件夹下递归的所有文件和文件夹不包含在这里。
pub type CommonPrefixes = Vec<ObjectPath>;

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
