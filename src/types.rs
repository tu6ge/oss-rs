use std::borrow::Cow;
use std::collections::HashMap;
#[cfg(feature = "core")]
use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use chrono::{DateTime, Utc};
use http::header::{HeaderValue, InvalidHeaderValue, ToStrError};
#[cfg(feature = "core")]
use reqwest::Url;

/// 阿里云 OSS 的签名 key
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InnerKeyId<'a>(Cow<'a, str>);

/// 静态作用域的 InnerKeyId
pub type KeyId = InnerKeyId<'static>;

impl AsRef<str> for InnerKeyId<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for InnerKeyId<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryInto<HeaderValue> for InnerKeyId<'_> {
    type Error = InvalidHeaderValue;
    fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(self.as_ref())
    }
}

impl From<String> for KeyId {
    fn from(s: String) -> KeyId {
        Self(Cow::Owned(s))
    }
}

impl<'a: 'b, 'b> From<&'a str> for InnerKeyId<'b> {
    fn from(key_id: &'a str) -> Self {
        Self(Cow::Borrowed(key_id))
    }
}

impl<'a> InnerKeyId<'a> {
    /// Creates a new `KeyId` from the given string.
    pub fn new(key_id: impl Into<Cow<'a, str>>) -> Self {
        Self(key_id.into())
    }

    /// Const function that creates a new `KeyId` from a static str.
    pub const fn from_static(key_id: &'static str) -> Self {
        Self(Cow::Borrowed(key_id))
    }
}

//===================================================================================================

/// 阿里云 OSS 的签名 secret
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InnerKeySecret<'a>(Cow<'a, str>);

/// 静态作用域的 InnerKeySecret
pub type KeySecret = InnerKeySecret<'static>;

impl AsRef<str> for InnerKeySecret<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for InnerKeySecret<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryInto<HeaderValue> for InnerKeySecret<'_> {
    type Error = InvalidHeaderValue;

    /// ```
    /// # use aliyun_oss_client::types::KeySecret;
    /// # use http::header::HeaderValue;
    /// let secret = KeySecret::new("foo");
    /// let value: HeaderValue = secret.try_into().unwrap();
    /// assert_eq!(value.to_str().unwrap(), "foo");
    /// ```
    fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(self.as_ref())
    }
}

impl From<String> for KeySecret {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl<'a: 'b, 'b> From<&'a str> for InnerKeySecret<'b> {
    fn from(secret: &'a str) -> Self {
        Self(Cow::Borrowed(secret))
    }
}

impl<'a> InnerKeySecret<'a> {
    /// Creates a new `KeySecret` from the given string.
    pub fn new(secret: impl Into<Cow<'a, str>>) -> Self {
        Self(secret.into())
    }

    /// Const function that creates a new `KeySecret` from a static str.
    pub const fn from_static(secret: &'static str) -> Self {
        Self(Cow::Borrowed(secret))
    }

    /// 转化成 bytes
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref().as_bytes()
    }
}

//===================================================================================================

/// # OSS 的可用区
/// [aliyun docs](https://help.aliyun.com/document_detail/31837.htm)
#[cfg(feature = "core")]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum EndPoint {
    /// 杭州可用区
    #[default]
    CnHangzhou,
    /// 上海可用区
    CnShanghai,
    /// 青岛可用区
    CnQingdao,
    /// 北京可用区
    CnBeijing,
    /// 张家口可用区
    CnZhangjiakou, // 张家口 lenght=13
    /// 香港
    CnHongkong,
    /// 深圳
    CnShenzhen,
    /// 美国西部
    UsWest1,
    /// 美国东部
    UsEast1,
    /// 新加坡
    ApSouthEast1,
    /// 其他可用区 fuzhou，ap-southeast-6 等
    Other(Cow<'static, str>),
}

#[cfg(feature = "core")]
const HANGZHOU: &str = "cn-hangzhou";
#[cfg(feature = "core")]
const SHANGHAI: &str = "cn-shanghai";
#[cfg(feature = "core")]
const QINGDAO: &str = "cn-qingdao";
#[cfg(feature = "core")]
const BEIJING: &str = "cn-beijing";
#[cfg(feature = "core")]
const ZHANGJIAKOU: &str = "cn-zhangjiakou";
#[cfg(feature = "core")]
const HONGKONG: &str = "cn-hongkong";
#[cfg(feature = "core")]
const SHENZHEN: &str = "cn-shenzhen";
#[cfg(feature = "core")]
const US_WEST1: &str = "us-west-1";
#[cfg(feature = "core")]
const US_EAST1: &str = "us-east-1";
#[cfg(feature = "core")]
const AP_SOUTH_EAST1: &str = "ap-southeast-1";

#[cfg(feature = "core")]
impl AsRef<str> for EndPoint {
    /// ```
    /// # use aliyun_oss_client::types::EndPoint::*;
    ///
    /// assert_eq!(CnHangzhou.as_ref(), "cn-hangzhou");
    /// assert_eq!(CnShanghai.as_ref(), "cn-shanghai");
    /// assert_eq!(CnQingdao.as_ref(), "cn-qingdao");
    /// assert_eq!(CnBeijing.as_ref(), "cn-beijing");
    /// assert_eq!(CnZhangjiakou.as_ref(), "cn-zhangjiakou");
    /// assert_eq!(CnHongkong.as_ref(), "cn-hongkong");
    /// assert_eq!(CnShenzhen.as_ref(), "cn-shenzhen");
    /// assert_eq!(UsWest1.as_ref(), "us-west-1");
    /// assert_eq!(UsEast1.as_ref(), "us-east-1");
    /// assert_eq!(ApSouthEast1.as_ref(), "ap-southeast-1");
    /// ```
    fn as_ref(&self) -> &str {
        use EndPoint::*;
        match self {
            CnHangzhou => HANGZHOU,
            CnShanghai => SHANGHAI,
            CnQingdao => QINGDAO,
            CnBeijing => BEIJING,
            CnZhangjiakou => ZHANGJIAKOU,
            CnHongkong => HONGKONG,
            CnShenzhen => SHENZHEN,
            UsWest1 => US_WEST1,
            UsEast1 => US_EAST1,
            ApSouthEast1 => AP_SOUTH_EAST1,
            Other(str) => str,
        }
    }
}

#[cfg(feature = "core")]
impl Display for EndPoint {
    /// ```
    /// # use aliyun_oss_client::types::EndPoint::*;
    /// assert_eq!(format!("{}", CnHangzhou), "cn-hangzhou");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

#[cfg(feature = "core")]
const HANGZHOU_L: &str = "hangzhou";
#[cfg(feature = "core")]
const SHANGHAI_L: &str = "shanghai";
#[cfg(feature = "core")]
const QINGDAO_L: &str = "qingdao";
#[cfg(feature = "core")]
const BEIJING_L: &str = "beijing";
#[cfg(feature = "core")]
const ZHANGJIAKOU_L: &str = "zhangjiakou";
#[cfg(feature = "core")]
const HONGKONG_L: &str = "hongkong";
#[cfg(feature = "core")]
const SHENZHEN_L: &str = "shenzhen";

#[cfg(feature = "core")]
impl TryFrom<String> for EndPoint {
    type Error = InvalidEndPoint;
    /// 字符串转 endpoint
    ///
    /// 举例
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// let e: EndPoint = String::from("qingdao").try_into().unwrap();
    /// assert_eq!(e, EndPoint::CnQingdao);
    /// ```
    fn try_from(url: String) -> Result<Self, Self::Error> {
        Self::new(&url)
    }
}

#[cfg(feature = "core")]
impl<'a> TryFrom<&'a str> for EndPoint {
    type Error = InvalidEndPoint;
    /// 字符串字面量转 endpoint
    ///
    /// 举例
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// let e: EndPoint = "qingdao".try_into().unwrap();
    /// assert_eq!(e, EndPoint::CnQingdao);
    /// ```
    fn try_from(url: &'a str) -> Result<Self, Self::Error> {
        Self::new(url)
    }
}

#[cfg(feature = "core")]
impl FromStr for EndPoint {
    type Err = InvalidEndPoint;
    fn from_str(url: &str) -> Result<Self, Self::Err> {
        Self::new(url)
    }
}

#[cfg(feature = "core")]
const OSS_DOMAIN_PREFIX: &str = "https://oss-";
#[cfg(feature = "core")]
#[allow(dead_code)]
const OSS_INTERNAL: &str = "-internal";
#[cfg(feature = "core")]
const OSS_DOMAIN_MAIN: &str = ".aliyuncs.com";

#[cfg(feature = "core")]
impl<'a> EndPoint {
    /// 通过字符串字面值初始化 endpoint
    ///
    /// 例如
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// EndPoint::from_static("qingdao");
    /// ```
    pub fn from_static(url: &'a str) -> Self {
        Self::new(url).unwrap_or_else(|_| panic!("Unknown Endpoint :{}", url))
    }

    /// # Safety
    /// 用于静态定义其他可用区
    pub const unsafe fn from_static2(url: &'static str) -> Self {
        EndPoint::Other(Cow::Borrowed(url))
    }

    /// 初始化 endpoint enum
    /// ```rust
    /// # use aliyun_oss_client::types::EndPoint;
    /// # use std::borrow::Cow;
    /// assert!(matches!(
    ///     EndPoint::new("shanghai"),
    ///     Ok(EndPoint::CnShanghai)
    /// ));
    ///
    /// let weifang = "weifang".to_string();
    /// assert!(matches!(
    ///     EndPoint::new("weifang"),
    ///     Ok(EndPoint::Other(Cow::Owned(weifang)))
    /// ));
    ///
    /// assert!(EndPoint::new("abc-").is_err());
    /// assert!(EndPoint::new("-abc").is_err());
    /// assert!(EndPoint::new("abc-def234ab").is_ok());
    /// assert!(EndPoint::new("abc-def*#$%^ab").is_err());
    /// assert!(EndPoint::new("cn-jinan").is_ok());
    /// assert!(EndPoint::new("oss-cn-jinan").is_err());
    /// ```
    pub fn new(url: &'a str) -> Result<Self, InvalidEndPoint> {
        use EndPoint::*;

        if url.contains(SHANGHAI_L) {
            Ok(CnShanghai)
        } else if url.contains(HANGZHOU_L) {
            Ok(CnHangzhou)
        } else if url.contains(QINGDAO_L) {
            Ok(CnQingdao)
        } else if url.contains(BEIJING_L) {
            Ok(CnBeijing)
        } else if url.contains(ZHANGJIAKOU_L) {
            Ok(CnZhangjiakou)
        } else if url.contains(HONGKONG_L) {
            Ok(CnHongkong)
        } else if url.contains(SHENZHEN_L) {
            Ok(CnShenzhen)
        } else if url.contains(US_WEST1) {
            Ok(UsWest1)
        } else if url.contains(US_EAST1) {
            Ok(UsEast1)
        } else if url.contains(AP_SOUTH_EAST1) {
            Ok(ApSouthEast1)
        } else {
            if url.is_empty() {
                return Err(InvalidEndPoint);
            }

            if url.starts_with('-') || url.ends_with('-') {
                return Err(InvalidEndPoint);
            }

            if url.starts_with("oss") {
                return Err(InvalidEndPoint);
            }

            fn valid_character(c: char) -> bool {
                match c {
                    _ if c.is_ascii_lowercase() => true,
                    _ if c.is_numeric() => true,
                    '-' => true,
                    _ => false,
                }
            }
            if !url.chars().all(valid_character) {
                return Err(InvalidEndPoint);
            }

            Ok(Other(Cow::Owned(url.to_owned())))
        }
    }

    /// 转化成 Url
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// use reqwest::Url;
    /// let endpoint = EndPoint::new("shanghai").unwrap();
    /// assert_eq!(
    ///     endpoint.to_url(),
    ///     Url::parse("https://oss-cn-shanghai.aliyuncs.com").unwrap()
    /// );
    ///
    /// use std::env::set_var;
    /// set_var("ALIYUN_OSS_INTERNAL", "true");
    /// let endpoint = EndPoint::new("shanghai").unwrap();
    /// assert_eq!(
    ///     endpoint.to_url(),
    ///     Url::parse("https://oss-cn-shanghai-internal.aliyuncs.com").unwrap()
    /// );
    /// ```
    pub fn to_url(&self) -> Url {
        let mut url = String::from(OSS_DOMAIN_PREFIX);
        url.push_str(self.as_ref());

        // internal
        if env::var("ALIYUN_OSS_INTERNAL").is_ok() {
            url.push_str(OSS_INTERNAL);
        }

        url.push_str(OSS_DOMAIN_MAIN);
        Url::parse(&url).unwrap_or_else(|_| panic!("covert to url failed, endpoint: {}", url))
    }
}

#[cfg(all(test, feature = "core"))]
mod test_endpoint {
    use super::*;

    #[test]
    #[should_panic]
    fn test_endpoint_painc() {
        EndPoint::from_static("-weifang");
    }

    #[test]
    fn test_new() {
        assert!(matches!(
            EndPoint::new("hangzhou"),
            Ok(EndPoint::CnHangzhou)
        ));

        assert!(matches!(EndPoint::new("qingdao"), Ok(EndPoint::CnQingdao)));

        assert!(matches!(EndPoint::new("beijing"), Ok(EndPoint::CnBeijing)));

        assert!(matches!(
            EndPoint::new("zhangjiakou"),
            Ok(EndPoint::CnZhangjiakou)
        ));

        assert!(matches!(
            EndPoint::new("hongkong"),
            Ok(EndPoint::CnHongkong)
        ));

        assert!(matches!(
            EndPoint::new("shenzhen"),
            Ok(EndPoint::CnShenzhen)
        ));

        assert!(matches!(EndPoint::new("us-west-1"), Ok(EndPoint::UsWest1)));

        assert!(matches!(EndPoint::new("us-east-1"), Ok(EndPoint::UsEast1)));

        assert!(matches!(
            EndPoint::new("ap-southeast-1"),
            Ok(EndPoint::ApSouthEast1)
        ));
    }
}

/// 无效的可用区
#[cfg(feature = "core")]
#[derive(Debug)]
#[non_exhaustive]
pub struct InvalidEndPoint;

#[cfg(feature = "core")]
impl Error for InvalidEndPoint {}

#[cfg(feature = "core")]
impl fmt::Display for InvalidEndPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        )
    }
}

#[cfg(feature = "core")]
impl PartialEq<&str> for EndPoint {
    /// # 相等比较
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// let e: EndPoint = String::from("qingdao").try_into().unwrap();
    /// assert!(e == "cn-qingdao");
    /// ```
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.as_ref() == other
    }
}

#[cfg(feature = "core")]
impl PartialEq<EndPoint> for &str {
    /// # 相等比较
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// let e: EndPoint = String::from("qingdao").try_into().unwrap();
    /// assert!("cn-qingdao" == e);
    /// ```
    #[inline]
    fn eq(&self, other: &EndPoint) -> bool {
        self == &other.as_ref()
    }
}

#[cfg(feature = "core")]
impl PartialEq<Url> for EndPoint {
    /// # 相等比较
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// use reqwest::Url;
    /// let endpoint = EndPoint::new("shanghai").unwrap();
    /// assert!(endpoint == Url::parse("https://oss-cn-shanghai.aliyuncs.com").unwrap());
    /// ```
    #[inline]
    fn eq(&self, other: &Url) -> bool {
        &self.to_url() == other
    }
}

//===================================================================================================

/// 存储 bucket 名字的类型
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
    #![allow(clippy::unwrap_used)]
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
    /// ```
    /// # use aliyun_oss_client::types::BucketName;
    /// let b: BucketName = String::from("abc").try_into().unwrap();
    /// assert_eq!(b, BucketName::new("abc").unwrap());
    /// ```
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl<'a> TryFrom<&'a str> for BucketName {
    type Error = InvalidBucketName;
    /// ```
    /// # use aliyun_oss_client::types::BucketName;
    /// let b: BucketName = "abc".try_into().unwrap();
    /// assert_eq!(b, BucketName::new("abc").unwrap());
    /// ```
    fn try_from(bucket: &'a str) -> Result<Self, Self::Error> {
        Self::from_static(bucket)
    }
}

impl FromStr for BucketName {
    type Err = InvalidBucketName;
    /// ```
    /// # use aliyun_oss_client::types::BucketName;
    /// let b: BucketName = "abc".parse().unwrap();
    /// assert_eq!(b, BucketName::new("abc").unwrap());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_static(s)
    }
}

impl<'a> BucketName {
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

        if bucket.is_empty() {
            return Err(InvalidBucketName);
        }

        if bucket.starts_with('-') || bucket.ends_with('-') {
            return Err(InvalidBucketName);
        }

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

        Ok(Self(bucket))
    }

    /// Const function that creates a new `BucketName` from a static str.
    /// ```
    /// # use aliyun_oss_client::types::BucketName;
    ///
    /// assert!(BucketName::from_static("").is_err());
    /// assert!(BucketName::from_static("abc").is_ok());
    /// assert!(BucketName::from_static("abc-").is_err());
    /// assert!(BucketName::from_static("-abc").is_err());
    /// assert!(BucketName::from_static("abc-def234ab").is_ok());
    /// assert!(BucketName::from_static("abc-def*#$%^ab").is_err());
    /// ```
    pub fn from_static(bucket: &'a str) -> Result<Self, InvalidBucketName> {
        if bucket.is_empty() {
            return Err(InvalidBucketName);
        }

        if bucket.starts_with('-') || bucket.ends_with('-') {
            return Err(InvalidBucketName);
        }

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

        Ok(Self(Cow::Owned(bucket.to_owned())))
    }

    /// # Safety
    pub const unsafe fn from_static2(bucket: &'static str) -> Self {
        Self(Cow::Borrowed(bucket))
    }
}

impl PartialEq<&str> for BucketName {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::types::BucketName;
    /// let path = BucketName::new("abc").unwrap();
    /// assert!(path == "abc");
    /// ```
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.0 == other
    }
}

impl PartialEq<BucketName> for &str {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::types::BucketName;
    /// let path = BucketName::new("abc").unwrap();
    /// assert!("abc" == path);
    /// ```
    #[inline]
    fn eq(&self, other: &BucketName) -> bool {
        self == &other.0
    }
}

/// 无效的 bucket 名称
#[derive(Debug)]
#[non_exhaustive]
pub struct InvalidBucketName;

impl Error for InvalidBucketName {}

impl fmt::Display for InvalidBucketName {
    /// ```
    /// # use aliyun_oss_client::types::BucketName;
    ///
    /// let err = BucketName::from_static("").unwrap_err();
    /// assert_eq!(format!("{}", err), "bucket 名称只允许小写字母、数字、短横线（-），且不能以短横线开头或结尾");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "bucket 名称只允许小写字母、数字、短横线（-），且不能以短横线开头或结尾"
        )
    }
}

//===================================================================================================

/// aliyun OSS 的配置 ContentMd5
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InnerContentMd5<'a>(Cow<'a, str>);
/// 静态作用域的 InnerContentMd5
pub type ContentMd5 = InnerContentMd5<'static>;

impl AsRef<str> for InnerContentMd5<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for InnerContentMd5<'_> {
    /// ```
    /// # use aliyun_oss_client::types::ContentMd5;
    /// let md5 = ContentMd5::new("abc");
    /// assert_eq!(format!("{md5}"), "abc");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryInto<HeaderValue> for InnerContentMd5<'_> {
    type Error = InvalidHeaderValue;
    fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(self.as_ref())
    }
}

impl TryInto<HeaderValue> for &InnerContentMd5<'_> {
    type Error = InvalidHeaderValue;
    fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(self.as_ref())
    }
}
impl From<String> for ContentMd5 {
    /// ```
    /// # use aliyun_oss_client::types::ContentMd5;
    /// let md5: ContentMd5 = String::from("abc").into();
    /// assert_eq!(format!("{md5}"), "abc");
    /// ```
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl<'a: 'b, 'b> From<&'a str> for InnerContentMd5<'b> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl<'a> InnerContentMd5<'a> {
    /// Creates a new `ContentMd5` from the given string.
    pub fn new(val: impl Into<Cow<'a, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `ContentMd5` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }
}

//===================================================================================================

/// aliyun OSS 的配置 ContentType
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
    type Error = ToStrError;
    fn try_from(value: HeaderValue) -> Result<Self, Self::Error> {
        Ok(Self(Cow::Owned(value.to_str()?.to_owned())))
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

/// 用于计算签名的 Date
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InnerDate<'a>(Cow<'a, str>);
/// 静态作用域的 InnerDate
pub type Date = InnerDate<'static>;

impl AsRef<str> for InnerDate<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for InnerDate<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryInto<HeaderValue> for InnerDate<'_> {
    type Error = InvalidHeaderValue;
    fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(self.as_ref())
    }
}
impl From<String> for InnerDate<'_> {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}
impl<'a: 'b, 'b> From<&'a str> for InnerDate<'b> {
    fn from(date: &'a str) -> Self {
        Self::new(date)
    }
}

impl From<DateTime<Utc>> for Date {
    fn from(d: DateTime<Utc>) -> Self {
        Self(Cow::Owned(d.format("%a, %d %b %Y %T GMT").to_string()))
    }
}

impl<'a> InnerDate<'a> {
    /// Creates a new `Date` from the given string.
    pub fn new(val: impl Into<Cow<'a, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `Date` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }
}

//===================================================================================================

/// 计算方式，参考 [aliyun 文档](https://help.aliyun.com/document_detail/31951.htm)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InnerCanonicalizedResource<'a>(Cow<'a, str>);
/// 静态作用域的 InnerCanonicalizedResource
pub type CanonicalizedResource = InnerCanonicalizedResource<'static>;

impl AsRef<str> for InnerCanonicalizedResource<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for InnerCanonicalizedResource<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryInto<HeaderValue> for InnerCanonicalizedResource<'_> {
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

impl<'a: 'b, 'b> From<&'a str> for InnerCanonicalizedResource<'b> {
    fn from(value: &'a str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl Default for InnerCanonicalizedResource<'_> {
    fn default() -> Self {
        InnerCanonicalizedResource(Cow::Owned("/".to_owned()))
    }
}

#[cfg(feature = "core")]
pub(crate) const CONTINUATION_TOKEN: &str = "continuation-token";
#[cfg(feature = "core")]
pub(crate) const BUCKET_INFO: &str = "bucketInfo";
#[cfg(feature = "core")]
const QUERY_KEYWORD: [&str; 2] = ["acl", BUCKET_INFO];

impl<'a> InnerCanonicalizedResource<'a> {
    /// Creates a new `CanonicalizedResource` from the given string.
    pub fn new(val: impl Into<Cow<'a, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `CanonicalizedResource` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }

    /// 获取 bucket 的签名参数
    #[cfg(feature = "core")]
    pub fn from_bucket<B: AsRef<BucketName>>(bucket: B, query: Option<&str>) -> Self {
        match query {
            Some(q) => {
                for k in QUERY_KEYWORD.iter() {
                    if *k == q {
                        return Self::new(format!("/{}/?{}", bucket.as_ref().as_ref(), q));
                    }
                }

                Self::new(format!("/{}/", bucket.as_ref().as_ref()))
            }
            None => Self::default(),
        }
    }

    /// 获取 bucket 的签名参数
    /// 带查询条件的
    ///
    /// 如果查询条件中有翻页的话，则忽略掉其他字段
    #[cfg(feature = "core")]
    pub fn from_bucket_query<B: AsRef<BucketName>>(bucket: B, query: &Query) -> Self {
        match query.get(CONTINUATION_TOKEN) {
            Some(v) => Self::new(format!(
                "/{}/?continuation-token={}",
                bucket.as_ref().as_ref(),
                v.as_ref()
            )),
            None => Self::new(format!("/{}/", bucket.as_ref().as_ref())),
        }
    }

    /// 根据 OSS 存储对象（Object）查询签名参数
    #[cfg(feature = "core")]
    pub(crate) fn from_object<
        Q: IntoIterator<Item = (QueryKey, QueryValue)>,
        B: AsRef<str>,
        P: AsRef<str>,
    >(
        (bucket, path): (B, P),
        query: Q,
    ) -> Self {
        let query = Query::from_iter(query);
        if query.is_empty() {
            Self::new(format!("/{}/{}", bucket.as_ref(), path.as_ref()))
        } else {
            Self::new(format!(
                "/{}/{}?{}",
                bucket.as_ref(),
                path.as_ref(),
                query.to_url_query()
            ))
        }
    }
}

impl PartialEq<&str> for InnerCanonicalizedResource<'_> {
    /// # 相等比较
    /// ```
    /// # use aliyun_oss_client::types::CanonicalizedResource;
    /// let res = CanonicalizedResource::new("abc");
    /// assert!(res == "abc");
    /// ```
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.0 == other
    }
}

impl PartialEq<InnerCanonicalizedResource<'_>> for &str {
    /// # 相等比较
    /// ```
    /// # use aliyun_oss_client::types::CanonicalizedResource;
    /// let res = CanonicalizedResource::new("abc");
    /// assert!("abc" == res);
    /// ```
    #[inline]
    fn eq(&self, other: &InnerCanonicalizedResource<'_>) -> bool {
        self == &other.0
    }
}

// #[cfg(test)]
// mod tests_canonicalized_resource {

//     #[cfg(feature = "core")]
//     #[test]
//     fn test_from_bucket() {

//     }
// }

//===================================================================================================
/// 查询条件
///
/// ```
/// use aliyun_oss_client::types::Query;
///
/// let query: Query = [("abc", "def")].into_iter().collect();
/// assert_eq!(query.len(), 1);
///
/// let value = query.get("abc");
/// assert!(value.is_some());
/// let value = value.unwrap();
/// assert_eq!(value.as_ref(), "def");
///
/// let str = query.to_oss_string();
/// assert_eq!(str.as_str(), "list-type=2&abc=def");
/// let str = query.to_url_query();
/// assert_eq!(str.as_str(), "abc=def");
/// ```
#[derive(Clone, Debug, Default)]
pub struct Query {
    inner: HashMap<QueryKey, QueryValue>,
}

impl AsMut<HashMap<QueryKey, QueryValue>> for Query {
    fn as_mut(&mut self) -> &mut HashMap<QueryKey, QueryValue> {
        &mut self.inner
    }
}

impl AsRef<HashMap<QueryKey, QueryValue>> for Query {
    fn as_ref(&self) -> &HashMap<QueryKey, QueryValue> {
        &self.inner
    }
}

impl Query {
    /// Creates an empty `Query`.
    ///
    /// The hash map is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Creates an empty `Query` with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Inserts a key-value pair into the map.
    pub fn insert(&mut self, key: impl Into<QueryKey>, value: impl Into<QueryValue>) {
        self.inner.insert(key.into(), value.into());
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: impl Into<QueryKey>) -> Option<&QueryValue> {
        self.inner.get(&key.into())
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    pub fn remove(&mut self, key: impl Into<QueryKey>) -> Option<QueryValue> {
        self.inner.remove(&key.into())
    }

    /// 将查询参数拼成 aliyun 接口需要的格式
    pub fn to_oss_string(&self) -> String {
        let mut query_str = String::from("list-type=2");
        for (key, value) in self.inner.iter() {
            query_str += "&";
            query_str += key.as_ref();
            query_str += "=";
            query_str += value.as_ref();
        }
        query_str
    }

    /// 转化成 url 参数的形式
    /// a=foo&b=bar
    pub fn to_url_query(&self) -> String {
        self.inner
            .iter()
            .map(|(k, v)| {
                let mut res = String::with_capacity(k.as_ref().len() + v.as_ref().len() + 1);
                res.push_str(k.as_ref());
                res.push('=');
                res.push_str(v.as_ref());
                res
            })
            .collect::<Vec<_>>()
            .join("&")
    }
}

impl IntoIterator for Query {
    type Item = (QueryKey, QueryValue);
    type IntoIter = std::vec::IntoIter<Self::Item>;
    /// # 使用 Vec 转 Query
    /// 例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// # use aliyun_oss_client::QueryValue;
    /// # use assert_matches::assert_matches;
    /// let query = Query::from_iter(vec![("foo", "bar")]);
    /// let list: Vec<_> = query.into_iter().collect();
    /// assert_eq!(list.len(), 1);
    /// assert_matches!(&list[0].0, &QueryKey::Custom(_));
    /// let value: QueryValue = "bar".parse().unwrap();
    /// assert_eq!(list[0].1, value);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter().collect::<Vec<_>>().into_iter()
    }
}

impl FromIterator<(QueryKey, QueryValue)> for Query {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (QueryKey, QueryValue)>,
    {
        let mut map = Query::default();
        map.inner.extend(iter);
        map
    }
}

impl<'a> FromIterator<(&'a str, &'a str)> for Query {
    /// 转化例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query: Query = [("max-keys", "123")].into_iter().collect();
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let inner = iter.into_iter().map(|(k, v)| {
            (
                k.parse().expect("invalid QueryKey"),
                v.parse().expect("invalid QueryValue"),
            )
        });

        let mut map = Query::default();
        map.inner.extend(inner);
        map
    }
}

impl<'a> FromIterator<(&'a str, u8)> for Query {
    /// 转化例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query = Query::from_iter([("max-keys", 123u8)]);
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, u8)>,
    {
        let inner = iter
            .into_iter()
            .map(|(k, v)| (k.parse().expect("invalid QueryKey"), v.into()));

        let mut map = Query::default();
        map.inner.extend(inner);
        map
    }
}

impl<'a> FromIterator<(&'a str, u16)> for Query {
    /// 转化例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query = Query::from_iter([("max-keys", 123u16)]);
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, u16)>,
    {
        let inner = iter
            .into_iter()
            .map(|(k, v)| (k.parse().expect("invalid QueryKey"), v.into()));

        let mut map = Query::default();
        map.inner.extend(inner);
        map
    }
}

impl<'a> FromIterator<(QueryKey, &'a str)> for Query {
    /// 转化例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query = Query::from_iter([(QueryKey::MaxKeys, "123")]);
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (QueryKey, &'a str)>,
    {
        let inner = iter
            .into_iter()
            .map(|(k, v)| (k, v.parse().expect("invalid QueryValue")));

        let mut map = Query::default();
        map.inner.extend(inner);
        map
    }
}

impl FromIterator<(QueryKey, u8)> for Query {
    /// 转化例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query = Query::from_iter([(QueryKey::MaxKeys, 123u8)]);
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (QueryKey, u8)>,
    {
        let inner = iter.into_iter().map(|(k, v)| (k, v.into()));

        let mut map = Query::default();
        map.inner.extend(inner);
        map
    }
}

impl FromIterator<(QueryKey, u16)> for Query {
    /// 转化例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query = Query::from_iter([(QueryKey::MaxKeys, 123u16)]);
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (QueryKey, u16)>,
    {
        let inner = iter.into_iter().map(|(k, v)| (k, v.into()));

        let mut map = Query::default();
        map.inner.extend(inner);
        map
    }
}

impl PartialEq<Query> for Query {
    fn eq(&self, other: &Query) -> bool {
        self.inner == other.inner
    }
}

// impl<K, V, const N: usize> From<[(K, V); N]> for Query
// where
//     K: Into<QueryKey>,
//     V: Into<QueryValue>,
// {
//     fn from(arr: [(K, V); N]) -> Self {
//         arr.into_iter().map(|(k, v)| (k.into(), v.into())).collect()
//     }
// }

/// 为 Url 拼接 [`Query`] 数据
/// [`Query`]: crate::types::Query
#[cfg(feature = "core")]
pub trait UrlQuery {
    /// 给 Url 结构体增加 `set_search_query` 方法
    fn set_search_query(&mut self, query: &Query);
}

#[cfg(feature = "core")]
impl UrlQuery for Url {
    /// 将查询参数拼接到 API 的 Url 上
    ///
    /// # 例子
    /// ```
    /// use aliyun_oss_client::types::Query;
    /// use aliyun_oss_client::types::UrlQuery;
    /// use reqwest::Url;
    ///
    /// let query = Query::from_iter([("abc", "def")]);
    /// let mut url = Url::parse("https://exapmle.com").unwrap();
    /// url.set_search_query(&query);
    /// assert_eq!(url.as_str(), "https://exapmle.com/?list-type=2&abc=def");
    /// assert_eq!(url.query(), Some("list-type=2&abc=def"));
    /// ```
    fn set_search_query(&mut self, query: &Query) {
        self.set_query(Some(&query.to_oss_string()));
    }
}

/// 查询条件的键
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InnerQueryKey<'a> {
    /// 对Object名字进行分组的字符。所有Object名字包含指定的前缀，第一次出现delimiter字符之间的Object作为一组元素（即CommonPrefixes）
    /// 示例值 `/`
    Delimiter,

    /// 设定从start-after之后按字母排序开始返回Object。
    /// start-after用来实现分页显示效果，参数的长度必须小于1024字节。
    /// 做条件查询时，即使start-after在列表中不存在，也会从符合start-after字母排序的下一个开始打印。
    StartAfter,

    /// 指定List操作需要从此token开始。您可从ListObjectsV2（GetBucketV2）结果中的NextContinuationToken获取此token。
    /// 用于分页，返回下一页的数据
    ContinuationToken,

    /// 指定返回Object的最大数。
    /// 取值：大于0小于等于1000
    MaxKeys,

    /// # 限定返回文件的Key必须以prefix作为前缀。
    /// 如果把prefix设为某个文件夹名，则列举以此prefix开头的文件，即该文件夹下递归的所有文件和子文件夹。
    ///
    /// 在设置prefix的基础上，将delimiter设置为正斜线（/）时，返回值就只列举该文件夹下的文件，文件夹下的子文件夹名返回在CommonPrefixes中，
    /// 子文件夹下递归的所有文件和文件夹不显示。
    ///
    /// 例如，一个Bucket中有三个Object，分别为fun/test.jpg、fun/movie/001.avi和fun/movie/007.avi。如果设定prefix为fun/，
    /// 则返回三个Object；如果在prefix设置为fun/的基础上，将delimiter设置为正斜线（/），则返回fun/test.jpg和fun/movie/。
    /// ## 要求
    /// - 参数的长度必须小于1024字节。
    /// - 设置prefix参数时，不能以正斜线（/）开头。如果prefix参数置空，则默认列举Bucket内的所有Object。
    /// - 使用prefix查询时，返回的Key中仍会包含prefix。
    Prefix,

    /// 对返回的内容进行编码并指定编码的类型。
    EncodingType,

    /// 指定是否在返回结果中包含owner信息。
    FetchOwner,

    /// 自定义
    #[deprecated(since = "0.12.0", note = "Please use QueryKey::new() replace it")]
    Custom(Cow<'a, str>),
}

/// 查询条件的键
pub type QueryKey = InnerQueryKey<'static>;

impl AsRef<str> for InnerQueryKey<'_> {
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// # use std::borrow::Cow;
    /// assert_eq!(QueryKey::Delimiter.as_ref(), "delimiter");
    /// assert_eq!(QueryKey::StartAfter.as_ref(), "start-after");
    /// assert_eq!(QueryKey::ContinuationToken.as_ref(), "continuation-token");
    /// assert_eq!(QueryKey::MaxKeys.as_ref(), "max-keys");
    /// assert_eq!(QueryKey::Prefix.as_ref(), "prefix");
    /// assert_eq!(QueryKey::EncodingType.as_ref(), "encoding-type");
    /// assert_eq!(QueryKey::Custom(Cow::Borrowed("abc")).as_ref(), "abc");
    /// ```
    fn as_ref(&self) -> &str {
        use InnerQueryKey::*;

        match self {
            Delimiter => "delimiter",
            StartAfter => "start-after",
            ContinuationToken => "continuation-token",
            MaxKeys => "max-keys",
            Prefix => "prefix",
            EncodingType => "encoding-type",
            // TODO
            FetchOwner => unimplemented!("parse xml not support fetch owner"),
            Custom(str) => str.as_ref(),
        }
    }
}

impl Display for InnerQueryKey<'_> {
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// assert_eq!(format!("{}", QueryKey::Delimiter), "delimiter");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl From<String> for InnerQueryKey<'_> {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}
impl<'a: 'b, 'b> From<&'a str> for InnerQueryKey<'b> {
    fn from(date: &'a str) -> Self {
        Self::new(date)
    }
}

impl FromStr for QueryKey {
    type Err = InvalidQueryKey;
    /// 示例
    /// ```
    /// # use aliyun_oss_client::types::QueryKey;
    /// let value: QueryKey = "abc".into();
    /// assert!(value == QueryKey::from_static("abc"));
    /// ```
    fn from_str(s: &str) -> Result<Self, InvalidQueryKey> {
        Ok(Self::from_static(s))
    }
}

/// 异常的查询条件键
#[derive(Debug)]
pub struct InvalidQueryKey;

impl Error for InvalidQueryKey {}

impl Display for InvalidQueryKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid query key")
    }
}

impl<'a> InnerQueryKey<'a> {
    /// # Examples
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// # use assert_matches::assert_matches;
    /// let key = QueryKey::new("delimiter");
    /// assert!(key == QueryKey::Delimiter);
    /// assert!(QueryKey::new("start-after") == QueryKey::StartAfter);
    /// assert!(QueryKey::new("continuation-token") == QueryKey::ContinuationToken);
    /// assert!(QueryKey::new("max-keys") == QueryKey::MaxKeys);
    /// assert!(QueryKey::new("prefix") == QueryKey::Prefix);
    /// assert!(QueryKey::new("encoding-type") == QueryKey::EncodingType);
    ///
    /// let key = QueryKey::new("abc");
    /// assert_matches!(key, QueryKey::Custom(_));
    /// ```
    /// *`fetch-owner` 功能未实现，特殊说明*
    pub fn new(val: impl Into<Cow<'a, str>>) -> Self {
        use InnerQueryKey::*;

        let val = val.into();
        if val.contains("delimiter") {
            Delimiter
        } else if val.contains("start-after") {
            StartAfter
        } else if val.contains("continuation-token") {
            ContinuationToken
        } else if val.contains("max-keys") {
            MaxKeys
        } else if val.contains("prefix") {
            Prefix
        } else if val.contains("encoding-type") {
            EncodingType
        } else if val.contains("fetch-owner") {
            unimplemented!("parse xml not support fetch owner");
        } else {
            Custom(val)
        }
    }

    /// # Examples
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// # use assert_matches::assert_matches;
    /// let key = QueryKey::from_static("delimiter");
    /// assert!(key == QueryKey::Delimiter);
    /// assert!(QueryKey::from_static("start-after") == QueryKey::StartAfter);
    /// assert!(QueryKey::from_static("continuation-token") == QueryKey::ContinuationToken);
    /// assert!(QueryKey::from_static("max-keys") == QueryKey::MaxKeys);
    /// assert!(QueryKey::from_static("prefix") == QueryKey::Prefix);
    /// assert!(QueryKey::from_static("encoding-type") == QueryKey::EncodingType);
    ///
    /// let key = QueryKey::from_static("abc");
    /// assert_matches!(key, QueryKey::Custom(_));
    /// ```
    /// *`fetch-owner` 功能未实现，特殊说明*
    pub fn from_static<'b>(val: &'b str) -> Self {
        use InnerQueryKey::*;

        if val.contains("delimiter") {
            Delimiter
        } else if val.contains("start-after") {
            StartAfter
        } else if val.contains("continuation-token") {
            ContinuationToken
        } else if val.contains("max-keys") {
            MaxKeys
        } else if val.contains("prefix") {
            Prefix
        } else if val.contains("encoding-type") {
            EncodingType
        } else if val.contains("fetch-owner") {
            unimplemented!("parse xml not support fetch owner");
        } else {
            Custom(Cow::Owned(val.to_owned()))
        }
    }
}

#[cfg(test)]
mod test_query_key {
    use super::*;

    #[test]
    #[should_panic]
    fn test_fetch_owner() {
        QueryKey::new("fetch-owner");
    }
}

/// 查询条件的值
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InnerQueryValue<'a>(Cow<'a, str>);
/// 查询条件的值
pub type QueryValue = InnerQueryValue<'static>;

impl AsRef<str> for InnerQueryValue<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for InnerQueryValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for InnerQueryValue<'_> {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}
impl<'a: 'b, 'b> From<&'a str> for InnerQueryValue<'b> {
    fn from(date: &'a str) -> Self {
        Self::new(date)
    }
}

impl PartialEq<&str> for InnerQueryValue<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.0 == other
    }
}

impl From<u8> for InnerQueryValue<'_> {
    /// 数字转 Query 值
    ///
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query = Query::from_iter([("max_keys", 100u8)]);
    /// let query = Query::from_iter([(QueryKey::MaxKeys, 100u8)]);
    /// ```
    fn from(num: u8) -> Self {
        Self(Cow::Owned(num.to_string()))
    }
}

impl PartialEq<u8> for InnerQueryValue<'_> {
    #[inline]
    fn eq(&self, other: &u8) -> bool {
        self.to_string() == other.to_string()
    }
}

impl From<u16> for InnerQueryValue<'_> {
    /// 数字转 Query 值
    ///
    /// ```
    /// use aliyun_oss_client::Query;
    /// let query = Query::from_iter([("max_keys", 100u16)]);
    /// ```
    fn from(num: u16) -> Self {
        Self(Cow::Owned(num.to_string()))
    }
}

impl PartialEq<u16> for InnerQueryValue<'_> {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        self.to_string() == other.to_string()
    }
}

impl From<bool> for QueryValue {
    /// bool 转 Query 值
    ///
    /// ```
    /// use aliyun_oss_client::Query;
    /// let query = Query::from_iter([("abc", "false")]);
    /// ```
    fn from(b: bool) -> Self {
        if b {
            Self::from_static("true")
        } else {
            Self::from_static("false")
        }
    }
}

impl FromStr for InnerQueryValue<'_> {
    type Err = InvalidQueryValue;
    /// 示例
    /// ```
    /// # use aliyun_oss_client::types::QueryValue;
    /// let value: QueryValue = "abc".parse().unwrap();
    /// assert!(value == "abc");
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_static2(s))
    }
}

/// 异常的查询值
#[derive(Debug)]
pub struct InvalidQueryValue;

impl Error for InvalidQueryValue {}

impl Display for InvalidQueryValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid query value")
    }
}

impl<'a> InnerQueryValue<'a> {
    /// Creates a new `QueryValue` from the given string.
    pub fn new(val: impl Into<Cow<'a, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `QueryValue` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }

    /// Const function that creates a new `QueryValue` from a static str.
    pub fn from_static2<'b>(val: &'b str) -> Self {
        Self(Cow::Owned(val.to_owned()))
    }
}

use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

/// 用于指定返回内容的区域的 type
pub struct ContentRange {
    start: Option<u32>,
    end: Option<u32>,
}

impl From<Range<u32>> for ContentRange {
    fn from(r: Range<u32>) -> Self {
        Self {
            start: Some(r.start),
            end: Some(r.end),
        }
    }
}

impl From<RangeFull> for ContentRange {
    fn from(_f: RangeFull) -> Self {
        Self {
            start: None,
            end: None,
        }
    }
}

impl From<RangeFrom<u32>> for ContentRange {
    fn from(f: RangeFrom<u32>) -> Self {
        Self {
            start: Some(f.start),
            end: None,
        }
    }
}

impl From<RangeTo<u32>> for ContentRange {
    fn from(t: RangeTo<u32>) -> Self {
        Self {
            start: None,
            end: Some(t.end),
        }
    }
}

impl From<ContentRange> for HeaderValue {
    /// # 转化成 OSS 需要的格式
    /// @link [OSS 文档](https://help.aliyun.com/document_detail/31980.html)
    ///
    /// ```
    /// use reqwest::header::HeaderValue;
    /// # use aliyun_oss_client::types::ContentRange;
    /// fn abc<R: Into<ContentRange>>(range: R) -> HeaderValue {
    ///     range.into().into()
    /// }
    ///
    /// assert_eq!(abc(..), HeaderValue::from_str("bytes=0-").unwrap());
    /// assert_eq!(abc(1..), HeaderValue::from_str("bytes=1-").unwrap());
    /// assert_eq!(abc(10..20), HeaderValue::from_str("bytes=10-20").unwrap());
    /// assert_eq!(abc(..20), HeaderValue::from_str("bytes=0-20").unwrap());
    /// ```
    fn from(con: ContentRange) -> HeaderValue {
        let string = match (con.start, con.end) {
            (Some(ref start), Some(ref end)) => format!("bytes={}-{}", start, end),
            (Some(ref start), None) => format!("bytes={}-", start),
            (None, Some(ref end)) => format!("bytes=0-{}", end),
            (None, None) => "bytes=0-".to_string(),
        };

        HeaderValue::from_str(&string).unwrap_or_else(|_| {
            panic!(
                "content-range into header-value failed, content-range is : {}",
                string
            )
        })
    }
}
