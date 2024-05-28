use std::collections::HashMap;

use url::Url;

use crate::bucket::Bucket;

pub struct Key(String);

impl Key {
    pub fn new(key: String) -> Key {
        Key(key)
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub struct Secret(pub(crate) String);

impl Secret {
    pub fn new(secret: String) -> Secret {
        Secret(secret)
    }
    /// # 加密数据
    /// 这种加密方式可保证秘钥明文只会存在于 `Secret` 类型内，不会被读取或复制
    pub fn encryption(
        &self,
        data: &[u8],
    ) -> Result<String, hmac::digest::crypto_common::InvalidLength> {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        type HmacSha1 = Hmac<Sha1>;

        let secret = self.0.as_bytes();

        let mut mac = HmacSha1::new_from_slice(secret)?;

        mac.update(data);

        let sha1 = mac.finalize().into_bytes();

        Ok(STANDARD.encode(sha1))
    }
}

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

// const HANGZHOU_L: &str = "hangzhou";
// const SHANGHAI_L: &str = "shanghai";
// const QINGDAO_L: &str = "qingdao";
// const BEIJING_L: &str = "beijing";
// const ZHANGJIAKOU_L: &str = "zhangjiakou";
// const HONGKONG_L: &str = "hongkong";
// const SHENZHEN_L: &str = "shenzhen";

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
    Other(String),
}

pub struct CanonicalizedResource(String);

impl Default for CanonicalizedResource {
    fn default() -> Self {
        CanonicalizedResource("/".to_owned())
    }
}

impl CanonicalizedResource {
    pub fn new(str: String) -> CanonicalizedResource {
        CanonicalizedResource(str)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn from_bucket_info(bucket: &Bucket) -> CanonicalizedResource {
        Self(format!("/{}/?bucketInfo", bucket.as_str()))
    }

    pub fn from_object_list(
        bucket: &Bucket,
        continuation_token: Option<String>,
    ) -> CanonicalizedResource {
        // "/{}/?continuation-token={}"
        match continuation_token {
            Some(token) => Self(format!(
                "/{}/?continuation-token={}",
                bucket.as_str(),
                token
            )),
            None => Self(format!("/{}/", bucket.as_str())),
        }
    }
}

/// 存储类型
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct StorageClass {
    kind: StorageClassKind,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
enum StorageClassKind {
    /// Standard 默认
    #[default]
    Standard,
    /// IA
    IA,
    /// Archive
    Archive,
    /// ColdArchive
    ColdArchive,
}

impl StorageClass {
    /// Archive
    pub const ARCHIVE: Self = Self {
        kind: StorageClassKind::Archive,
    };
    /// IA
    pub const IA: Self = Self {
        kind: StorageClassKind::IA,
    };
    /// Standard
    pub const STANDARD: Self = Self {
        kind: StorageClassKind::Standard,
    };
    /// ColdArchive
    pub const COLD_ARCHIVE: Self = Self {
        kind: StorageClassKind::ColdArchive,
    };

    /// init StorageClass
    pub fn new(s: &str) -> Option<StorageClass> {
        let start_char = s.chars().next()?;

        let kind = match start_char {
            'a' | 'A' => StorageClassKind::Archive,
            'i' | 'I' => StorageClassKind::IA,
            's' | 'S' => StorageClassKind::Standard,
            'c' | 'C' => StorageClassKind::ColdArchive,
            _ => return None,
        };
        Some(Self { kind })
    }
}

#[derive(Debug, Default)]
pub struct ObjectQuery {
    map: HashMap<String, String>,
}

impl ObjectQuery {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn insert<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> Option<String> {
        self.map.insert(key.into(), value.into())
    }

    pub(crate) fn to_oss_query(&self) -> String {
        const LIST_TYPE2: &str = "list-type=2";
        let mut query_str = String::from(LIST_TYPE2);
        for (key, value) in self.map.iter() {
            query_str += "&";
            query_str += key;
            query_str += "=";
            query_str += value;
        }
        query_str
    }

    pub fn append_next_token(&mut self, token: Option<String>) {
        if let Some(token) = token {
            self.insert("continuation-token", token);
        }
    }
}
