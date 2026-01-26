use crate::error::OssError;

use serde::Deserialize;
use std::fmt;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Region {
    Known(KnownRegion),
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum KnownRegion {
    CnHangzhou,
    CnShanghai,
    CnQingdao,
    CnBeijing,
    CnZhangjiakou,
    CnHongkong,
    CnShenzhen,
    UsWest1,
    UsEast1,
    ApSoutheast1,
}

impl Region {
    pub fn parse(s: &str) -> Result<Self, OssError> {
        if s.is_empty() {
            return Err(OssError::InvalidRegion);
        }

        let s = s.trim();

        let known = match s {
            "cn-hangzhou" => Some(KnownRegion::CnHangzhou),
            "cn-shanghai" => Some(KnownRegion::CnShanghai),
            "cn-qingdao" => Some(KnownRegion::CnQingdao),
            "cn-beijing" => Some(KnownRegion::CnBeijing),
            "cn-zhangjiakou" => Some(KnownRegion::CnZhangjiakou),
            "cn-hongkong" => Some(KnownRegion::CnHongkong),
            "cn-shenzhen" => Some(KnownRegion::CnShenzhen),
            "us-west-1" => Some(KnownRegion::UsWest1),
            "us-east-1" => Some(KnownRegion::UsEast1),
            "ap-southeast-1" => Some(KnownRegion::ApSoutheast1),
            _ => None,
        };

        if let Some(k) = known {
            return Ok(Region::Known(k));
        }

        // custom region 校验
        if !s.chars().all(valid_oss_character)
            || s.starts_with('-')
            || s.ends_with('-')
            || s.starts_with("oss")
        {
            return Err(OssError::InvalidRegion);
        }

        Ok(Region::Custom(s.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Region::Known(k) => k.as_str(),
            Region::Custom(s) => s,
        }
    }
}

impl KnownRegion {
    pub fn as_str(&self) -> &'static str {
        match self {
            KnownRegion::CnHangzhou => "cn-hangzhou",
            KnownRegion::CnShanghai => "cn-shanghai",
            KnownRegion::CnQingdao => "cn-qingdao",
            KnownRegion::CnBeijing => "cn-beijing",
            KnownRegion::CnZhangjiakou => "cn-zhangjiakou",
            KnownRegion::CnHongkong => "cn-hongkong",
            KnownRegion::CnShenzhen => "cn-shenzhen",
            KnownRegion::UsWest1 => "us-west-1",
            KnownRegion::UsEast1 => "us-east-1",
            KnownRegion::ApSoutheast1 => "ap-southeast-1",
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EndPoint {
    region: Region,
    internal: bool,
}

impl TryFrom<String> for EndPoint {
    type Error = OssError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        EndPoint::infer_from_oss_url(&value)
    }
}
impl TryFrom<&str> for EndPoint {
    type Error = OssError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        EndPoint::infer_from_oss_url(value)
    }
}
impl TryFrom<Url> for EndPoint {
    type Error = OssError;
    fn try_from(url: Url) -> Result<Self, Self::Error> {
        Self::from_url(&url)
    }
}

impl EndPoint {
    pub fn new(region: Region) -> Self {
        Self {
            region,
            internal: false,
        }
    }

    pub fn internal(mut self) -> Self {
        self.internal = true;
        self
    }

    pub fn is_internal(&self) -> bool {
        self.internal
    }

    pub fn region(&self) -> &Region {
        &self.region
    }

    pub fn from_env() -> Result<Self, OssError> {
        let region = std::env::var("ALIYUN_ENDPOINT").map_err(|_| OssError::InvalidRegion)?;

        let mut endpoint = EndPoint::infer_from_oss_url(&region)?;

        if let Ok(v) = std::env::var("ALIYUN_OSS_INTERNAL") {
            if matches!(v.as_str(), "1" | "true" | "yes" | "Y") {
                endpoint = endpoint.internal();
            }
        }

        Ok(endpoint)
    }

    pub fn to_url(&self) -> Result<EndPointUrl, OssError> {
        EndPointUrl::new(self)
    }

    /// 从 oss endpoint url 推断（用于响应 / 反序列化）
    pub fn infer_from_oss_url(url: &str) -> Result<Self, OssError> {
        let url = if url.contains("://") {
            url.to_string()
        } else {
            format!("https://{}", url)
        };
        let url = Url::parse(&url).map_err(|_| OssError::InvalidEndPoint)?;

        Self::from_url(&url)
    }

    fn from_url(url: &Url) -> Result<Self, OssError> {
        let host = url.host_str().ok_or(OssError::InvalidEndPoint)?;

        if !host.ends_with(OSS_DOMAIN) {
            return Err(OssError::InvalidEndPoint);
        }

        let host = host.trim_end_matches(OSS_DOMAIN).trim_end_matches('.');
        let host = host.trim_start_matches("oss-");

        let internal = host.ends_with("-internal");
        let region = if internal {
            host.trim_end_matches("-internal")
        } else {
            host
        };

        Ok(EndPoint {
            region: Region::parse(region)?,
            internal,
        })
    }
}

const OSS_DOMAIN: &str = "aliyuncs.com";
const OSS_PREFIX: &str = "oss";

#[derive(Clone, Debug)]
pub struct EndPointUrl {
    url: Url,
}

impl EndPointUrl {
    pub fn new(endpoint: &EndPoint) -> Result<Self, OssError> {
        let mut host = String::new();

        host.push_str(OSS_PREFIX);
        host.push('-');
        host.push_str(endpoint.region().as_str());

        if endpoint.is_internal() {
            host.push_str("-internal");
        }

        host.push('.');
        host.push_str(OSS_DOMAIN);

        let url =
            Url::parse(&format!("https://{}", host)).map_err(|_| OssError::InvalidEndPoint)?;

        Ok(Self { url })
    }

    pub fn as_url(&self) -> &Url {
        &self.url
    }

    pub(crate) fn host(&self) -> &str {
        self.url.host_str().unwrap_or_default()
    }
}

use serde::de::{self, Deserializer};

impl<'de> Deserialize<'de> for EndPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        EndPoint::infer_from_oss_url(&s).map_err(|_| de::Error::custom("invalid oss endpoint"))
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
