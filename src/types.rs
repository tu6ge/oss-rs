use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, Utc};
use reqwest::header::{HeaderValue, InvalidHeaderValue};
use reqwest::Url;

use crate::config::{BucketBase, GetObjectInfo};
use crate::errors::{OssError, OssResult};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct KeyId(Cow<'static, str>);

impl AsRef<str> for KeyId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for KeyId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryInto<HeaderValue> for KeyId {
    type Error = InvalidHeaderValue;
    fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(self.as_ref())
    }
}

impl From<String> for KeyId {
    fn from(s: String) -> KeyId {
        KeyId(Cow::Owned(s))
    }
}

impl From<&'static str> for KeyId {
    fn from(key_id: &'static str) -> Self {
        Self::from_static(key_id)
    }
}

impl KeyId {
    /// Creates a new `KeyId` from the given string.
    pub fn new(key_id: impl Into<Cow<'static, str>>) -> Self {
        Self(key_id.into())
    }

    /// Const function that creates a new `KeyId` from a static str.
    pub const fn from_static(key_id: &'static str) -> Self {
        Self(Cow::Borrowed(key_id))
    }
}

//===================================================================================================

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct KeySecret(Cow<'static, str>);

impl AsRef<str> for KeySecret {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for KeySecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryInto<HeaderValue> for KeySecret {
    type Error = InvalidHeaderValue;
    fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(self.as_ref())
    }
}

impl From<String> for KeySecret {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl From<&'static str> for KeySecret {
    fn from(secret: &'static str) -> Self {
        Self::from_static(secret)
    }
}

impl KeySecret {
    /// Creates a new `KeySecret` from the given string.
    pub fn new(secret: impl Into<Cow<'static, str>>) -> Self {
        Self(secret.into())
    }

    /// Const function that creates a new `KeySecret` from a static str.
    pub const fn from_static(secret: &'static str) -> Self {
        Self(Cow::Borrowed(secret))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref().as_bytes()
    }
}

//===================================================================================================

/// OSS 的可用区
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EndPoint {
    CnHangzhou,
    CnShanghai,
    CnQingdao,
    CnBeijing,
    CnZhangjiakou, // 张家口 lenght=13
    CnHongkong,
    CnShenzhen,
    UsWest1,
    UsEast1,
    ApSouthEast1,
}

impl AsRef<str> for EndPoint {
    fn as_ref(&self) -> &str {
        match *self {
            Self::CnHangzhou => "cn-hangzhou",
            Self::CnShanghai => "cn-shanghai",
            Self::CnQingdao => "cn-qingdao",
            Self::CnBeijing => "cn-beijing",
            Self::CnZhangjiakou => "cn-zhangjiakou",
            Self::CnHongkong => "cn-hongkong",
            Self::CnShenzhen => "cn-shenzhen",
            Self::UsWest1 => "us-west1",
            Self::UsEast1 => "us-east1",
            Self::ApSouthEast1 => "ap-south-east1",
            //_ => "custom",
        }
    }
}

impl Default for EndPoint {
    fn default() -> Self {
        Self::CnHangzhou
    }
}

impl Display for EndPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

// 已实现，需要的时候再打开
// impl TryInto<Url> for EndPoint {
//     type Error = OssError;
//     fn try_into(self) -> Result<Url, OssError> {
//         Url::parse(self.as_ref()).map_err(|e|OssError::Input(e.to_string()))
//     }
// }

impl TryFrom<String> for EndPoint {
    type Error = InvalidEndPoint;
    /// 字符串转 endpoint
    /// 举例1 - 产生恐慌
    /// ```should_panic
    /// # use aliyun_oss_client::types::EndPoint;
    /// let e: EndPoint = String::from("weifang").try_into().unwrap();
    /// ```
    /// 举例2 - 正常
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// let e: EndPoint = String::from("qingdao").try_into().unwrap();
    /// ```
    fn try_from(url: String) -> Result<Self, Self::Error> {
        if url.contains("shanghai") {
            Ok(Self::CnShanghai)
        } else if url.contains("hangzhou") {
            Ok(Self::CnHangzhou)
        } else if url.contains("qingdao") {
            Ok(Self::CnQingdao)
        } else if url.contains("beijing") {
            Ok(Self::CnBeijing)
        } else if url.contains("zhangjiakou") {
            Ok(Self::CnZhangjiakou)
        } else if url.contains("hongkong") {
            Ok(Self::CnHongkong)
        } else if url.contains("shenzhen") {
            Ok(Self::CnShenzhen)
        } else if url.contains("us-west1") {
            Ok(Self::UsWest1)
        } else if url.contains("us-east1") {
            Ok(Self::UsEast1)
        } else if url.contains("ap-south-east1") {
            Ok(Self::ApSouthEast1)
        } else {
            Err(InvalidEndPoint)
        }
    }
}

impl TryFrom<&'static str> for EndPoint {
    type Error = InvalidEndPoint;
    fn try_from(url: &'static str) -> Result<Self, Self::Error> {
        Self::new(url)
    }
}

impl EndPoint {
    /// 通过字符串字面值初始化 endpoint
    ///
    /// 举例1 - 产生恐慌
    /// ```should_panic
    /// # use aliyun_oss_client::types::EndPoint;
    /// EndPoint::from_static("weifang");
    /// ```
    /// 举例2 - 正常
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// EndPoint::from_static("qingdao");
    /// ```
    pub fn from_static(url: &'static str) -> Self {
        Self::new(url).expect(format!("Unknown Endpoint :{}", url).as_str())
    }

    /// 初始化 endpoint enum
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// assert!(matches!(EndPoint::new("shanghai"), Ok(EndPoint::CnShanghai)));
    /// assert!(EndPoint::new("weifang").is_err());
    /// ```
    pub fn new(url: &'static str) -> Result<Self, InvalidEndPoint> {
        if url.contains("shanghai") {
            Ok(Self::CnShanghai)
        } else if url.contains("hangzhou") {
            Ok(Self::CnHangzhou)
        } else if url.contains("qingdao") {
            Ok(Self::CnQingdao)
        } else if url.contains("beijing") {
            Ok(Self::CnBeijing)
        } else if url.contains("zhangjiakou") {
            Ok(Self::CnZhangjiakou)
        } else if url.contains("hongkong") {
            Ok(Self::CnHongkong)
        } else if url.contains("shenzhen") {
            Ok(Self::CnShenzhen)
        } else if url.contains("us-west1") {
            Ok(Self::UsWest1)
        } else if url.contains("us-east1") {
            Ok(Self::UsEast1)
        } else if url.contains("ap-south-east1") {
            Ok(Self::ApSouthEast1)
        } else {
            Err(InvalidEndPoint)
        }
    }

    /// 转化成 Url
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// use reqwest::Url;
    /// let endpoint = EndPoint::new("shanghai").unwrap();
    /// assert_eq!(endpoint.to_url(), Url::parse("https://oss-cn-shanghai.aliyuncs.com").unwrap());
    /// ```
    pub fn to_url(&self) -> Url {
        let mut url = String::from("https://oss-");
        url.push_str(self.as_ref());
        url.push_str(".aliyuncs.com");
        Url::parse(&url).unwrap()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct InvalidEndPoint;

impl Error for InvalidEndPoint {}

impl fmt::Display for InvalidEndPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "endpoint must like with https://xxx.aliyuncs.com")
    }
}

//===================================================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BucketName(Cow<'static, str>);

impl AsRef<str> for BucketName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for BucketName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for BucketName {
    fn default() -> BucketName {
        BucketName::new("a").unwrap()
    }
}

// impl TryInto<HeaderValue> for BucketName {
//     type Error = InvalidHeaderValue;
//     fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
//         HeaderValue::from_str(self.as_ref())
//     }
// }
impl TryFrom<String> for BucketName {
    type Error = InvalidBucketName;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<&'static str> for BucketName {
    type Error = InvalidBucketName;
    fn try_from(bucket: &'static str) -> Result<Self, Self::Error> {
        Self::from_static(bucket)
    }
}

impl BucketName {
    /// Creates a new `BucketName` from the given string.
    /// 只允许小写字母、数字、短横线（-），且不能以短横线开头或结尾
    ///
    /// ```
    /// # use aliyun_oss_client::types::BucketName;
    ///
    /// assert!(BucketName::new("").is_err());
    /// assert!(BucketName::new("abc").is_ok());
    /// assert!(BucketName::new("abc-").is_err());
    /// assert!(BucketName::new("-abc").is_err());
    /// assert!(BucketName::new("abc-def234ab").is_ok());
    /// assert!(BucketName::new("abc-def*#$%^ab").is_err());
    /// ```
    pub fn new(bucket: impl Into<Cow<'static, str>>) -> Result<Self, InvalidBucketName> {
        let bucket = bucket.into();

        fn valid_character(c: char) -> bool {
            match c {
                _ if c.is_ascii_lowercase() => true,
                _ if c.is_numeric() => true,
                '-' => true,
                _ => false,
            }
        }
        if !bucket.chars().all(valid_character) {
            return Err(InvalidBucketName);
        }

        if bucket.len() < 1 {
            return Err(InvalidBucketName);
        }

        if bucket.starts_with("-") || bucket.ends_with("-") {
            return Err(InvalidBucketName);
        }

        Ok(Self(bucket))
    }

    /// Const function that creates a new `BucketName` from a static str.
    pub fn from_static(bucket: &'static str) -> Result<Self, InvalidBucketName> {
        fn valid_character(c: char) -> bool {
            match c {
                _ if c.is_ascii_lowercase() => true,
                _ if c.is_numeric() => true,
                '-' => true,
                _ => false,
            }
        }
        if !bucket.chars().all(valid_character) {
            return Err(InvalidBucketName);
        }

        if bucket.len() < 1 {
            return Err(InvalidBucketName);
        }

        if bucket.starts_with("-") || bucket.ends_with("-") {
            return Err(InvalidBucketName);
        }

        Ok(Self(Cow::Borrowed(bucket)))
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct InvalidBucketName;

impl Error for InvalidBucketName {}

impl fmt::Display for InvalidBucketName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "bucket 名称只允许小写字母、数字、短横线（-），且不能以短横线开头或结尾"
        )
    }
}

//===================================================================================================

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ContentMd5(Cow<'static, str>);

impl AsRef<str> for ContentMd5 {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for ContentMd5 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryInto<HeaderValue> for ContentMd5 {
    type Error = InvalidHeaderValue;
    fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(self.as_ref())
    }
}
impl From<String> for ContentMd5 {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl ContentMd5 {
    /// Creates a new `ContentMd5` from the given string.
    pub fn new(val: impl Into<Cow<'static, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `ContentMd5` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }
}

//===================================================================================================

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ContentType(Cow<'static, str>);

impl AsRef<str> for ContentType {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryInto<HeaderValue> for ContentType {
    type Error = InvalidHeaderValue;
    fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(self.as_ref())
    }
}
impl TryFrom<HeaderValue> for ContentType {
    type Error = OssError;
    fn try_from(value: HeaderValue) -> OssResult<Self> {
        Ok(Self(Cow::Owned(
            value.to_str().map_err(OssError::from)?.to_owned(),
        )))
    }
}
impl From<String> for ContentType {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl ContentType {
    /// Creates a new `ContentMd5` from the given string.
    pub fn new(val: impl Into<Cow<'static, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `ContentMd5` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }
}

//===================================================================================================

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Date(Cow<'static, str>);

impl AsRef<str> for Date {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryInto<HeaderValue> for Date {
    type Error = InvalidHeaderValue;
    fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(self.as_ref())
    }
}
impl From<String> for Date {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}
impl From<&'static str> for Date {
    fn from(date: &'static str) -> Self {
        Self::from_static(date)
    }
}

impl From<DateTime<Utc>> for Date {
    fn from(d: DateTime<Utc>) -> Self {
        Self::from(d.format("%a, %d %b %Y %T GMT").to_string())
    }
}

impl Date {
    /// Creates a new `Date` from the given string.
    pub fn new(val: impl Into<Cow<'static, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `Date` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }
}

//===================================================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalizedResource(Cow<'static, str>);

impl AsRef<str> for CanonicalizedResource {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for CanonicalizedResource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryInto<HeaderValue> for CanonicalizedResource {
    type Error = InvalidHeaderValue;
    fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(self.as_ref())
    }
}
impl From<String> for CanonicalizedResource {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl Default for CanonicalizedResource {
    fn default() -> Self {
        Self::new("/")
    }
}

impl CanonicalizedResource {
    /// Creates a new `CanonicalizedResource` from the given string.
    pub fn new(val: impl Into<Cow<'static, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `CanonicalizedResource` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }

    /// 获取 bucket 的签名参数
    pub fn from_bucket(bucket: &BucketBase, query: Option<&str>) -> Self {
        match query {
            Some(q) => {
                if q == "acl" || q == "bucketInfo" {
                    return Self::from(format!("/{}/?{}", bucket.name(), q));
                }

                Self::from(format!("/{}/", bucket.name()))
            }
            None => Self::from_static("/"),
        }
    }

    /// 获取 bucket 的签名参数
    /// 带查询条件的
    ///
    /// 如果查询条件中有翻页的话，则忽略掉其他字段
    pub fn from_bucket_query(bucket: &BucketBase, query: &Query) -> Self {
        match query.get("continuation-token") {
            Some(v) => Self::from(format!(
                "/{}/?continuation-token={}",
                bucket.name(),
                v.as_ref()
            )),
            None => Self::from(format!("/{}/", bucket.name())),
        }
    }

    /// 根据 OSS 存储对象（Object）查询签名参数
    pub fn from_object<Obj: GetObjectInfo>(object: &Obj, query: Option<&Query>) -> Self {
        let bucket = object.bucket_name();
        let path = object.path();

        match query {
            Some(q) => {
                let query_value = q.to_url_query();
                Self::from(format!("/{}/{}?{}", bucket, path, query_value))
            }
            None => Self::from(format!("/{}/{}", bucket, path)),
        }
    }
}

//===================================================================================================
/// 查询条件
///
/// ```
/// use aliyun_oss_client::types::Query;
///
/// let mut query = Query::new();
/// query.insert("abc","def");
/// assert_eq!(query.len(), 1);
///
/// let value = query.get("abc");
/// assert!(value.is_some());
/// let value = value.unwrap();
/// assert_eq!(value.as_ref(), "def");
///
/// let str = query.to_oss_string();
/// assert_eq!(str.as_str(), "list-type=2&abc=def");
/// ```
#[derive(Clone, Debug, Default)]
pub struct Query {
    inner: HashMap<QueryKey, QueryValue>,
}

impl Query {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: impl Into<QueryKey>, value: impl Into<QueryValue>) {
        self.inner.insert(key.into(), value.into());
    }

    pub fn get(&self, key: impl Into<QueryKey>) -> Option<&QueryValue> {
        self.inner.get(&key.into())
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn remove(&mut self, key: impl Into<QueryKey>) -> Option<QueryValue> {
        self.inner.remove(&key.into())
    }

    /// 将查询参数拼成 aliyun 接口需要的格式
    pub fn to_oss_string(&self) -> String {
        let mut query_str = String::new();
        for (key, value) in self.inner.iter() {
            query_str += "&";
            query_str += key.as_ref();
            query_str += "=";
            query_str += value.as_ref();
        }
        let query_str = "list-type=2".to_owned() + &query_str;
        query_str
    }

    /// 转化成 url 参数的形式
    /// a=foo&b=bar
    /// 未进行 urlencode 转码
    pub fn to_url_query(&self) -> String {
        let list: Vec<String> = self
            .inner
            .iter()
            .map(|(k, v)| {
                let mut res = String::new();
                res.push_str(k.as_ref());
                res.push_str("=");
                res.push_str(v.as_ref());
                res
            })
            .collect();

        list.join("&")
    }
}

pub trait UrlQuery {
    fn set_search_query(&mut self, query: &Query);
}

impl UrlQuery for Url {
    /// 将查询参数拼接到 API 的 Url 上
    ///
    /// # 例子
    /// ```
    /// use aliyun_oss_client::types::Query;
    /// use aliyun_oss_client::types::UrlQuery;
    /// use reqwest::Url;
    ///
    /// let mut query = Query::new();
    /// query.insert("abc","def");
    /// let mut url = Url::parse("https://exapmle.com").unwrap();
    /// url.set_search_query(&query);
    /// assert_eq!(url.as_str(), "https://exapmle.com/?list-type=2&abc=def");
    /// assert_eq!(url.query(), Some("list-type=2&abc=def"));
    /// ```
    fn set_search_query(&mut self, query: &Query) {
        let str = query.to_oss_string();
        self.set_query(Some(&str));
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Hash)]
pub struct QueryKey(Cow<'static, str>);

impl AsRef<str> for QueryKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for QueryKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// TODO 需要的时候再开启
// impl TryInto<HeaderValue> for QueryKey {
//     type Error = InvalidHeaderValue;
//     fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
//         HeaderValue::from_str(self.as_ref())
//     }
// }
impl From<String> for QueryKey {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}
impl From<&'static str> for QueryKey {
    fn from(date: &'static str) -> Self {
        Self::from_static(date)
    }
}

impl QueryKey {
    /// Creates a new `QueryKey` from the given string.
    pub fn new(val: impl Into<Cow<'static, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `QueryKey` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct QueryValue(Cow<'static, str>);

impl AsRef<str> for QueryValue {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for QueryValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// TODO 需要的时候再开启
// impl TryInto<HeaderValue> for QueryValue {
//     type Error = InvalidHeaderValue;
//     fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
//         HeaderValue::from_str(self.as_ref())
//     }
// }
impl From<String> for QueryValue {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}
impl From<&'static str> for QueryValue {
    fn from(date: &'static str) -> Self {
        Self::from_static(date)
    }
}

impl QueryValue {
    /// Creates a new `QueryValue` from the given string.
    pub fn new(val: impl Into<Cow<'static, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `QueryValue` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }
}
