use crate::error::OssError;

use serde::{
    de::{value, Visitor},
    Deserialize,
};
use url::Url;

/// # OSS 的可用区
/// [aliyun docs](https://help.aliyun.com/document_detail/31837.htm)
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct EndPoint {
    pub(crate) kind: EndPointKind,
    /// default false
    pub(crate) is_internal: bool,
}

const OSS_INTERNAL: &str = "-internal";
const OSS_DOMAIN_MAIN: &str = ".aliyuncs.com";

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

const HANGZHOU_L: &str = "hangzhou";
const SHANGHAI_L: &str = "shanghai";
const QINGDAO_L: &str = "qingdao";
const BEIJING_L: &str = "beijing";
const ZHANGJIAKOU_L: &str = "zhangjiakou";
const HONGKONG_L: &str = "hongkong";
const SHENZHEN_L: &str = "shenzhen";

impl EndPoint {
    /// 杭州
    pub const CN_HANGZHOU: Self = Self {
        kind: EndPointKind::CnHangzhou,
        is_internal: false,
    };
    /// 杭州
    pub const HANGZHOU: Self = Self::CN_HANGZHOU;

    /// 上海
    pub const CN_SHANGHAI: Self = Self {
        kind: EndPointKind::CnShanghai,
        is_internal: false,
    };
    /// 上海
    pub const SHANGHAI: Self = Self::CN_SHANGHAI;

    /// 青岛
    pub const CN_QINGDAO: Self = Self {
        kind: EndPointKind::CnQingdao,
        is_internal: false,
    };
    /// 青岛
    pub const QINGDAO: Self = Self::CN_QINGDAO;

    /// 北京
    pub const CN_BEIJING: Self = Self {
        kind: EndPointKind::CnBeijing,
        is_internal: false,
    };
    /// 北京
    pub const BEIJING: Self = Self::CN_BEIJING;

    /// 张家口
    pub const CN_ZHANGJIAKOU: Self = Self::ZHANGJIAKOU;
    /// 张家口
    pub const ZHANGJIAKOU: Self = Self {
        kind: EndPointKind::CnZhangjiakou,
        is_internal: false,
    };

    /// 香港
    pub const CN_HONGKONG: Self = Self {
        kind: EndPointKind::CnHongkong,
        is_internal: false,
    };
    /// 香港
    pub const HONGKONG: Self = Self::CN_HONGKONG;

    /// 深圳
    pub const CN_SHENZHEN: Self = Self {
        kind: EndPointKind::CnShenzhen,
        is_internal: false,
    };
    /// 深圳
    pub const SHENZHEN: Self = Self::CN_SHENZHEN;

    /// UsWest1
    pub const US_WEST_1: Self = Self {
        kind: EndPointKind::UsWest1,
        is_internal: false,
    };

    /// UsEast1
    pub const US_EAST_1: Self = Self {
        kind: EndPointKind::UsEast1,
        is_internal: false,
    };

    /// ApSouthEast1
    pub const AP_SOUTH_EAST_1: Self = Self {
        kind: EndPointKind::ApSouthEast1,
        is_internal: false,
    };

    pub fn init(kind: EndPointKind, is_internal: bool) -> Self {
        Self { kind, is_internal }
    }

    /// 初始化 endpoint enum
    /// ```rust
    /// # use aliyun_oss_client::types::EndPoint;
    /// assert!(matches!(
    ///     EndPoint::new("shanghai"),
    ///     Ok(EndPoint::SHANGHAI)
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
    pub fn new(url: &str) -> Result<Self, OssError> {
        const OSS_STR: &str = "oss";
        use EndPointKind::*;
        if url.is_empty() {
            return Err(OssError::InvalidEndPoint);
        }
        // 是否是内网
        let is_internal = url.ends_with(OSS_INTERNAL);
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
            if url.starts_with('-') || url.ends_with('-') || url.starts_with(OSS_STR) {
                return Err(OssError::InvalidEndPoint);
            }

            if !url.chars().all(valid_oss_character) {
                return Err(OssError::InvalidEndPoint);
            }

            Ok(Other(url.to_owned()))
        };

        kind.map(|kind| Self { kind, is_internal })
    }

    /// 反序列化一个 endpoint 的域名
    /// ```rust
    /// # use aliyun_oss_client::types::EndPoint;
    /// # use aliyun_oss_client::types::EndPointKind;
    /// let str = "http://oss-cn-nanjing-internal.aliyuncs.com";
    /// let end = EndPoint::deserialize(str);
    /// let end2 = EndPoint::init(EndPointKind::Other("cn-nanjing".to_string()), true);
    /// assert_eq!(end, end2);
    /// ```
    pub fn deserialize(url: &str) -> Self {
        use EndPointKind::*;
        // 是否是内网
        let is_internal = url.contains(OSS_INTERNAL);

        let kind = if url.contains(SHANGHAI_L) {
            CnShanghai
        } else if url.contains(HANGZHOU_L) {
            CnHangzhou
        } else if url.contains(QINGDAO_L) {
            CnQingdao
        } else if url.contains(BEIJING_L) {
            CnBeijing
        } else if url.contains(ZHANGJIAKOU_L) {
            CnZhangjiakou
        } else if url.contains(HONGKONG_L) {
            CnHongkong
        } else if url.contains(SHENZHEN_L) {
            CnShenzhen
        } else if url.contains(US_WEST1) {
            UsWest1
        } else if url.contains(US_EAST1) {
            UsEast1
        } else if url.contains(AP_SOUTH_EAST1) {
            ApSouthEast1
        } else {
            let part_url = url.replace("https://", "");
            let part_url = part_url.replace("http://", "");
            let part_url = part_url.replace(".aliyuncs.com", "");
            let part_url = part_url.replace(OSS_INTERNAL, "");
            let part_url = part_url.replace("oss-", "");
            Other(part_url)
        };

        Self { kind, is_internal }
    }

    /// use env init Endpoint
    pub fn from_env() -> Result<Self, OssError> {
        let endpoint = std::env::var("ALIYUN_ENDPOINT").map_err(|_| OssError::InvalidEndPoint)?;
        let mut endpoint = EndPoint::new(&endpoint)?;

        if let Ok(is_internal) = std::env::var("ALIYUN_OSS_INTERNAL") {
            if is_internal == "true"
                || is_internal == "1"
                || is_internal == "yes"
                || is_internal == "Y"
            {
                endpoint.set_internal(true);
            }
        }

        Ok(endpoint)
    }

    /// # 调整 API 指向是否为内网
    ///
    /// 当在 Aliyun ECS 上执行时，设为 true 会更高效，默认是 false
    pub fn set_internal(&mut self, is_internal: bool) {
        self.is_internal = is_internal;
    }

    /// 返回当前的 endpoint 是否为内网
    pub fn is_internal(&self) -> bool {
        self.is_internal
    }

    /// 转化成 Url
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    /// use reqwest::Url;
    /// let mut endpoint = EndPoint::CN_SHANGHAI;;
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
        const OSS_DOMAIN_PREFIX: &str = "https://oss-";
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

fn valid_oss_character(c: char) -> bool {
    match c {
        _ if c.is_ascii_lowercase() => true,
        _ if c.is_numeric() => true,
        '-' => true,
        _ => false,
    }
}

impl AsRef<str> for EndPoint {
    /// ```
    /// # use aliyun_oss_client::types::EndPoint;
    ///
    /// assert_eq!(EndPoint::HANGZHOU.as_ref(), "cn-hangzhou");
    /// assert_eq!(EndPoint::SHANGHAI.as_ref(), "cn-shanghai");
    /// assert_eq!(EndPoint::QINGDAO.as_ref(), "cn-qingdao");
    /// assert_eq!(EndPoint::BEIJING.as_ref(), "cn-beijing");
    /// assert_eq!(EndPoint::ZHANGJIAKOU.as_ref(), "cn-zhangjiakou");
    /// assert_eq!(EndPoint::HONGKONG.as_ref(), "cn-hongkong");
    /// assert_eq!(EndPoint::SHENZHEN.as_ref(), "cn-shenzhen");
    /// assert_eq!(EndPoint::US_WEST_1.as_ref(), "us-west-1");
    /// assert_eq!(EndPoint::US_EAST_1.as_ref(), "us-east-1");
    /// assert_eq!(EndPoint::AP_SOUTH_EAST_1.as_ref(), "ap-southeast-1");
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

/// # OSS 的可用区种类 enum
#[derive(Clone, Debug, PartialEq, Eq, Default, Hash)]
#[non_exhaustive]
pub enum EndPointKind {
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
    #[allow(dead_code)]
    Other(String),
}

use std::fmt;

struct StringVisitor;

impl<'de> Visitor<'de> for StringVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v.to_owned())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }
}

impl<'de> Deserialize<'de> for EndPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = deserializer.deserialize_string(StringVisitor)?;

        Ok(EndPoint::deserialize(&str))
    }
}
