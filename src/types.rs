use std::{collections::HashMap, env::VarError};

use crate::{bucket::Bucket, Object};

mod endpoint;
pub use endpoint::{EndPoint, EndPointKind};
use serde::{de::Visitor, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key(String);

impl Key {
    pub fn new<K: Into<String>>(key: K) -> Key {
        Key(key.into())
    }

    pub fn from_env() -> Result<Key, VarError> {
        let key = std::env::var("ALIYUN_KEY_ID")?;
        Ok(Key(key))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Secret(pub(crate) String);

impl Secret {
    pub fn new<S: Into<String>>(secret: S) -> Secret {
        Secret(secret.into())
    }

    pub fn from_env() -> Result<Secret, VarError> {
        let key = std::env::var("ALIYUN_KEY_SECRET")?;
        Ok(Secret(key))
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

    pub fn from_object(bucket: &Bucket, object: &Object) -> CanonicalizedResource {
        CanonicalizedResource::new(format!("/{}/{}", bucket.as_str(), object.get_path()))
    }

    pub fn from_object_list(
        bucket: &Bucket,
        continuation_token: Option<&String>,
    ) -> CanonicalizedResource {
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

struct StorageClassVisitor;

impl<'de> Visitor<'de> for StorageClassVisitor {
    type Value = StorageClass;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Archive,IA,Standard or ColdArchive")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        StorageClass::new(&v).ok_or(E::custom(format!("{} is not StorageClass", v)))
    }
}
impl<'de> Deserialize<'de> for StorageClass {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(StorageClassVisitor)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ObjectQuery {
    map: HashMap<String, String>,
}

impl ObjectQuery {
    pub const DELIMITER: &'static str = "delimiter";
    pub const START_AFTER: &'static str = "start-after";
    pub const CONTINUATION_TOKEN: &'static str = "continuation-token";
    pub const MAX_KEYS: &'static str = "max-keys";
    pub const PREFIX: &'static str = "prefix";
    pub const ENCODING_TYPE: &'static str = "encoding-type";
    pub const FETCH_OWNER: &'static str = "fetch-owner";
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn insert<K: Into<String>, V: Into<String>>(&mut self, key: K, value: V) -> Option<String> {
        self.map.insert(key.into(), value.into())
    }

    pub(crate) fn get_next_token(&self) -> Option<&String> {
        self.map.get(Self::CONTINUATION_TOKEN)
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

    pub fn insert_next_token(&mut self, token: String) -> Option<String> {
        self.map.insert(Self::CONTINUATION_TOKEN.into(), token)
    }
}
