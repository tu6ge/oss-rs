//! lib 内置类型的定义模块

use std::{
    borrow::Cow,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    str::FromStr,
};

use chrono::{DateTime, TimeZone};
use http::header::{HeaderValue, InvalidHeaderValue, ToStrError};
use url::Url;

#[cfg(feature = "core")]
pub mod core;
#[cfg(feature = "core")]
pub mod object;
#[cfg(test)]
mod test;

#[cfg(feature = "core")]
pub use self::core::{ContentRange, Query, QueryKey, QueryValue, SetOssQuery};

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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct EndPoint {
    pub(crate) kind: EndPointKind,
    /// default false
    pub(crate) is_internal: bool,
}

impl EndPoint {
    /// 杭州
    #[deprecated(since = "0.13.0", note = "replace with EndPoint::CN_HANGZHOU")]
    pub const CnHangzhou: Self = Self {
        kind: EndPointKind::CnHangzhou,
        is_internal: false,
    };
    /// 杭州
    pub const CN_HANGZHOU: Self = Self {
        kind: EndPointKind::CnHangzhou,
        is_internal: false,
    };

    /// 上海
    #[deprecated(since = "0.13.0", note = "replace with EndPoint::CN_SHANGHAI")]
    pub const CnShanghai: Self = Self {
        kind: EndPointKind::CnShanghai,
        is_internal: false,
    };
    /// 上海
    pub const CN_SHANGHAI: Self = Self {
        kind: EndPointKind::CnShanghai,
        is_internal: false,
    };

    /// 青岛
    #[deprecated(since = "0.13.0", note = "replace with EndPoint::CN_QINGDAO")]
    pub const CnQingdao: Self = Self {
        kind: EndPointKind::CnQingdao,
        is_internal: false,
    };
    /// 青岛
    pub const CN_QINGDAO: Self = Self {
        kind: EndPointKind::CnQingdao,
        is_internal: false,
    };

    /// 北京
    #[deprecated(since = "0.13.0", note = "replace with EndPoint::CN_BEIJING")]
    pub const CnBeijing: Self = Self {
        kind: EndPointKind::CnBeijing,
        is_internal: false,
    };
    /// 北京
    pub const CN_BEIJING: Self = Self {
        kind: EndPointKind::CnBeijing,
        is_internal: false,
    };

    /// 张家口
    #[deprecated(since = "0.13.0", note = "replace with EndPoint::CN_ZHANGJIAKOU")]
    pub const CnZhangjiakou: Self = Self {
        kind: EndPointKind::CnZhangjiakou,
        is_internal: false,
    };
    /// 张家口
    pub const CN_ZHANGJIAKOU: Self = Self {
        kind: EndPointKind::CnZhangjiakou,
        is_internal: false,
    };

    /// 香港
    #[deprecated(since = "0.13.0", note = "replace with EndPoint::CN_HONGKONG")]
    pub const CnHongkong: Self = Self {
        kind: EndPointKind::CnHongkong,
        is_internal: false,
    };
    /// 香港
    pub const CN_HONGKONG: Self = Self {
        kind: EndPointKind::CnHongkong,
        is_internal: false,
    };

    /// 深圳
    #[deprecated(since = "0.13.0", note = "replace with EndPoint::CN_SHENZHEN")]
    pub const CnShenzhen: Self = Self {
        kind: EndPointKind::CnShenzhen,
        is_internal: false,
    };
    /// 深圳
    pub const CN_SHENZHEN: Self = Self {
        kind: EndPointKind::CnShenzhen,
        is_internal: false,
    };

    /// UsWest1
    #[deprecated(since = "0.13.0", note = "replace with EndPoint::US_WEST_1")]
    pub const UsWest1: Self = Self {
        kind: EndPointKind::UsWest1,
        is_internal: false,
    };
    /// UsWest1
    pub const US_WEST_1: Self = Self {
        kind: EndPointKind::UsWest1,
        is_internal: false,
    };

    /// UsEast1
    #[deprecated(since = "0.13.0", note = "replace with EndPoint::US_EAST_1")]
    pub const UsEast1: Self = Self {
        kind: EndPointKind::UsEast1,
        is_internal: false,
    };
    /// UsEast1
    pub const US_EAST_1: Self = Self {
        kind: EndPointKind::UsEast1,
        is_internal: false,
    };

    /// ApSouthEast1
    #[deprecated(since = "0.13.0", note = "replace with EndPoint::AP_SOUTH_EAST_1")]
    pub const ApSouthEast1: Self = Self {
        kind: EndPointKind::ApSouthEast1,
        is_internal: false,
    };
    /// ApSouthEast1
    pub const AP_SOUTH_EAST_1: Self = Self {
        kind: EndPointKind::ApSouthEast1,
        is_internal: false,
    };
}

/// # OSS 的可用区种类 enum
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub(crate) enum EndPointKind {
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

const HANGZHOU: &str = "cn-hangzhou";
const SHANGHAI: &str = "cn-shanghai";
const QINGDAO: &str = "cn-qingdao";
const BEIJING: &str = "cn-beijing";
const ZHANGJIAKOU: &str = "cn-zhangjiakou";
const HONGKONG: &str = "cn-hongkong";
const SHENZHEN: &str = "cn-shenzhen";
const US_WEST1: &str = "us-west-1";
const US_EAST1: &str = "us-east-1";
const AP_SOUTH_EAST1: &str = "ap-southeast-1";

impl AsRef<str> for EndPoint {
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    ///
    /// assert_eq!(EndPoint::CnHangzhou.as_ref(), "cn-hangzhou");
    /// assert_eq!(EndPoint::CnShanghai.as_ref(), "cn-shanghai");
    /// assert_eq!(EndPoint::CnQingdao.as_ref(), "cn-qingdao");
    /// assert_eq!(EndPoint::CnBeijing.as_ref(), "cn-beijing");
    /// assert_eq!(EndPoint::CnZhangjiakou.as_ref(), "cn-zhangjiakou");
    /// assert_eq!(EndPoint::CnHongkong.as_ref(), "cn-hongkong");
    /// assert_eq!(EndPoint::CnShenzhen.as_ref(), "cn-shenzhen");
    /// assert_eq!(EndPoint::UsWest1.as_ref(), "us-west-1");
    /// assert_eq!(EndPoint::UsEast1.as_ref(), "us-east-1");
    /// assert_eq!(EndPoint::ApSouthEast1.as_ref(), "ap-southeast-1");
    /// ```
    fn as_ref(&self) -> &str {
        use EndPointKind::*;
        match &self.kind {
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

impl Display for EndPoint {
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// assert_eq!(format!("{}", EndPoint::CnHangzhou), "cn-hangzhou");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

const HANGZHOU_L: &str = "hangzhou";
const SHANGHAI_L: &str = "shanghai";
const QINGDAO_L: &str = "qingdao";
const BEIJING_L: &str = "beijing";
const ZHANGJIAKOU_L: &str = "zhangjiakou";
const HONGKONG_L: &str = "hongkong";
const SHENZHEN_L: &str = "shenzhen";

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

impl FromStr for EndPoint {
    type Err = InvalidEndPoint;
    fn from_str(url: &str) -> Result<Self, Self::Err> {
        Self::new(url)
    }
}

const COM: &str = "com";
const ALIYUNCS: &str = "aliyuncs";
impl TryFrom<Url> for EndPoint {
    type Error = InvalidEndPoint;
    fn try_from(url: Url) -> Result<Self, Self::Error> {
        use url::Host;
        let domain = if let Some(Host::Domain(domain)) = url.host() {
            domain
        } else {
            return Err(InvalidEndPoint { _priv: () });
        };
        let mut url_pieces = domain.rsplit('.');

        match (url_pieces.next(), url_pieces.next()) {
            (Some(COM), Some(ALIYUNCS)) => (),
            _ => return Err(InvalidEndPoint { _priv: () }),
        }

        match url_pieces.next() {
            Some(endpoint) => match EndPoint::from_host_piece(endpoint) {
                Ok(end) => Ok(end),
                _ => Err(InvalidEndPoint { _priv: () }),
            },
            _ => Err(InvalidEndPoint { _priv: () }),
        }
    }
}

const OSS_DOMAIN_PREFIX: &str = "https://oss-";
#[allow(dead_code)]
const OSS_INTERNAL: &str = "-internal";
const OSS_DOMAIN_MAIN: &str = ".aliyuncs.com";

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
        Self {
            kind: EndPointKind::Other(Cow::Borrowed(url)),
            is_internal: false,
        }
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
    /// assert!(EndPoint::new("abc-").is_err());
    /// assert!(EndPoint::new("-abc").is_err());
    /// assert!(EndPoint::new("abc-def234ab").is_ok());
    /// assert!(EndPoint::new("abc-def*#$%^ab").is_err());
    /// assert!(EndPoint::new("cn-jinan").is_ok());
    /// assert!(EndPoint::new("cn-jinan").is_ok());
    /// assert!(EndPoint::new("oss-cn-jinan").is_err());
    /// ```
    pub fn new(url: &'a str) -> Result<Self, InvalidEndPoint> {
        use EndPointKind::*;
        if url.is_empty() {
            return Err(InvalidEndPoint { _priv: () });
        }
        // 是否是内网
        let is_internal = url.ends_with("-internal");
        let url = if is_internal {
            let len = url.len();
            &url[..len - 9]
        } else {
            url
        };

        let kind = if url.contains(SHANGHAI_L) {
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
            if url.starts_with('-') || url.ends_with('-') {
                return Err(InvalidEndPoint { _priv: () });
            }

            if url.starts_with("oss") {
                return Err(InvalidEndPoint { _priv: () });
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
                return Err(InvalidEndPoint { _priv: () });
            }

            Ok(Other(Cow::Owned(url.to_owned())))
        };

        kind.map(|kind| Self { kind, is_internal })
    }

    /// 从 oss 域名中提取 Endpoint 信息
    pub(crate) fn from_host_piece(url: &'a str) -> Result<Self, InvalidEndPoint> {
        if !url.starts_with("oss-") {
            return Err(InvalidEndPoint { _priv: () });
        }
        Self::new(&url[4..])
    }

    /// 设置 internal，当在 Aliyun ECS 上执行时，设为 true 会更高效，默认是 false
    pub fn set_internal(&mut self, is_internal: bool) {
        self.is_internal = is_internal;
    }

    /// 转化成 Url
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// use reqwest::Url;
    /// let mut endpoint = EndPoint::new("shanghai").unwrap();
    /// assert_eq!(
    ///     endpoint.to_url(),
    ///     Url::parse("https://oss-cn-shanghai.aliyuncs.com").unwrap()
    /// );
    ///
    /// endpoint.set_internal(true);
    /// assert_eq!(
    ///     endpoint.to_url(),
    ///     Url::parse("https://oss-cn-shanghai-internal.aliyuncs.com").unwrap()
    /// );
    /// ```
    pub fn to_url(&self) -> Url {
        let mut url = String::from(OSS_DOMAIN_PREFIX);
        url.push_str(self.as_ref());

        // internal
        if self.is_internal {
            url.push_str(OSS_INTERNAL);
        }

        url.push_str(OSS_DOMAIN_MAIN);
        Url::parse(&url).unwrap_or_else(|_| panic!("covert to url failed, endpoint: {}", url))
    }
}

/// 无效的可用区
#[derive(PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct InvalidEndPoint {
    pub(crate) _priv: (),
}

impl Debug for InvalidEndPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InvalidEndPoint").finish()
    }
}

impl Error for InvalidEndPoint {}

impl fmt::Display for InvalidEndPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        )
    }
}

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
            return Err(InvalidBucketName { _priv: () });
        }

        if bucket.starts_with('-') || bucket.ends_with('-') {
            return Err(InvalidBucketName { _priv: () });
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
            return Err(InvalidBucketName { _priv: () });
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
            return Err(InvalidBucketName { _priv: () });
        }

        if bucket.starts_with('-') || bucket.ends_with('-') {
            return Err(InvalidBucketName { _priv: () });
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
            return Err(InvalidBucketName { _priv: () });
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
#[derive(PartialEq)]
#[non_exhaustive]
pub struct InvalidBucketName {
    pub(crate) _priv: (),
}

impl Debug for InvalidBucketName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InvalidBucketName").finish()
    }
}

impl Error for InvalidBucketName {}

impl fmt::Display for InvalidBucketName {
    /// ```
    /// # use aliyun_oss_client::types::BucketName;
    ///
    /// let err = BucketName::from_static("").unwrap_err();
    /// assert_eq!(
    ///     format!("{}", err),
    ///     "bucket name only allow `alphabet, digit, -`, and must not with `-` prefix or `-` suffix"
    /// );
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "bucket name only allow `alphabet, digit, -`, and must not with `-` prefix or `-` suffix"
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

impl<Tz: TimeZone> From<DateTime<Tz>> for Date
where
    Tz::Offset: fmt::Display,
{
    fn from(d: DateTime<Tz>) -> Self {
        Self(Cow::Owned(d.format("%a, %d %b %Y %T GMT").to_string()))
    }
}

impl<'a> InnerDate<'a> {
    /// # Safety
    /// Const function that creates a new `Date` from a static str.
    pub const unsafe fn from_static(val: &'static str) -> Self {
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

#[cfg(any(feature = "core", feature = "auth"))]
pub(crate) const CONTINUATION_TOKEN: &str = "continuation-token";
#[cfg(any(feature = "core", feature = "auth"))]
pub(crate) const BUCKET_INFO: &str = "bucketInfo";
#[cfg(any(feature = "core", feature = "auth"))]
const QUERY_KEYWORD: [&str; 2] = ["acl", BUCKET_INFO];

impl<'a> InnerCanonicalizedResource<'a> {
    /// Creates a new `CanonicalizedResource` from the given string.
    pub fn new(val: impl Into<Cow<'a, str>>) -> Self {
        Self(val.into())
    }

    /// 只有 endpoint ，而没有 bucket 的时候
    #[inline(always)]
    pub fn from_endpoint() -> Self {
        Self::default()
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
                if QUERY_KEYWORD.iter().any(|&str| str == q) {
                    return Self::new(format!("/{}/?{}", bucket.as_ref().as_ref(), q));
                }

                Self::new(format!("/{}/", bucket.as_ref().as_ref()))
            }
            None => Self::default(),
        }
    }

    /// 获取 bucket 的签名参数
    #[cfg(feature = "auth")]
    pub fn from_bucket_name(bucket: &BucketName, query: Option<&str>) -> Self {
        match query {
            Some(q) => {
                if QUERY_KEYWORD.iter().any(|&str| str == q) {
                    return Self::new(format!("/{}/?{}", bucket.as_ref(), q));
                }

                Self::new(format!("/{}/", bucket.as_ref()))
            }
            None => Self::default(),
        }
    }

    /// 获取 bucket 的签名参数
    /// 带查询条件的
    ///
    /// 如果查询条件中有翻页的话，则忽略掉其他字段
    #[cfg(feature = "core")]
    #[inline]
    pub fn from_bucket_query<B: AsRef<BucketName>>(bucket: B, query: &Query) -> Self {
        Self::from_bucket_query2(bucket.as_ref(), query)
    }

    #[cfg(feature = "core")]
    #[doc(hidden)]
    pub fn from_bucket_query2(bucket: &BucketName, query: &Query) -> Self {
        match query.get(QueryKey::ContinuationToken) {
            Some(v) => Self::new(format!(
                "/{}/?continuation-token={}",
                bucket.as_ref(),
                v.as_ref()
            )),
            None => Self::new(format!("/{}/", bucket.as_ref())),
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

    #[cfg(feature = "auth")]
    pub(crate) fn from_object_without_query<B: AsRef<str>, P: AsRef<str>>(
        bucket: B,
        path: P,
    ) -> Self {
        Self::new(format!("/{}/{}", bucket.as_ref(), path.as_ref()))
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
