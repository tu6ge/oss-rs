use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use reqwest::Url;
use reqwest::header::{HeaderValue,InvalidHeaderValue};

use crate::errors::{OssError, OssResult};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct KeyId(
    Cow<'static, str>
);

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
pub struct KeySecret(
    Cow<'static, str>
);

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

    pub fn as_bytes(&self) -> &[u8]{
        self.as_ref().as_bytes()
    }
}

//===================================================================================================

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct EndPoint(
    Cow<'static, str>
);

impl AsRef<str> for EndPoint {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for EndPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// 已实现，需要的时候再打开
// impl TryInto<Url> for EndPoint {
//     type Error = OssError;
//     fn try_into(self) -> Result<Url, OssError> {
//         Url::parse(self.as_ref()).map_err(|e|OssError::Input(e.to_string()))
//     }
// }

impl From<String> for EndPoint {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl From<&'static str> for EndPoint {
    fn from(url: &'static str) -> Self {
        Self::from_static(url)
    }
}

impl EndPoint {
    pub fn new(url: impl Into<Cow<'static, str>>) -> Self {
        Self(url.into())
    }

    pub const fn from_static(url: &'static str) -> Self {
        Self(Cow::Borrowed(url))
    }

    pub fn into_url(&self) -> OssResult<Url> {
        Url::parse(self.as_ref()).map_err(|e|OssError::Input(e.to_string()))
    }
}

//===================================================================================================

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BucketName(
    Cow<'static, str>
);

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

// impl TryInto<HeaderValue> for BucketName {
//     type Error = InvalidHeaderValue;
//     fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
//         HeaderValue::from_str(self.as_ref())
//     }
// }
impl From<String> for BucketName {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl From<&'static str> for BucketName {
    fn from(bucket: &'static str) -> Self {
        Self::from_static(bucket)
    }
}

impl BucketName {
    /// Creates a new `BucketName` from the given string.
    pub fn new(bucket: impl Into<Cow<'static, str>>) -> Self {
        Self(bucket.into())
    }

    /// Const function that creates a new `BucketName` from a static str.
    pub const fn from_static(bucket: &'static str) -> Self {
        Self(Cow::Borrowed(bucket))
    }
}

//===================================================================================================

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ContentMd5(
    Cow<'static, str>
);

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
pub struct CanonicalizedResource(
    Cow<'static, str>
);

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

impl CanonicalizedResource {
    /// Creates a new `CanonicalizedResource` from the given string.
    pub fn new(val: impl Into<Cow<'static, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `CanonicalizedResource` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }
}

//===================================================================================================

pub struct Query{
    inner: HashMap<Cow<'static, str>, Cow<'static, str>>,
}

impl Query {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: Cow<'static, str>, value: Cow<'static, str>){
        self.inner.insert(key, value);
    }

    pub fn get(&self, key: &Cow<'static, str>) -> Option<&Cow<'static, str>> {
        self.inner.get(key)
    }

    pub fn remove(&mut self, key: &Cow<'static, str>) -> Option<Cow<'static, str>>{
        self.inner.remove(key)
    }

    /// 将查询参数拼成 aliyun 接口需要的格式
    pub fn to_oss_string(&self) -> String{
        let mut query_str = String::new();
        for (key,value) in self.inner.iter() {
            query_str += "&";
            query_str += key;
            query_str += "=";
            query_str += value;
        }
        let query_str = "list-type=2".to_owned() + &query_str;
        query_str
    }
}

trait UrlQuery {
    fn set_search_query(self, query: Query) -> Self;
}

impl UrlQuery for Url{

    /// 将查询参数拼接到 API 的 Url 上
    fn set_search_query(mut self, query: Query) -> Self {
        let str = query.to_oss_string();
        self.set_query(Some(&str));
        self
    }
}