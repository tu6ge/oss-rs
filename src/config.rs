//! 配置类型

use crate::{
    consts::{TRUE1, TRUE2, TRUE3, TRUE4},
    types::{
        core::SetOssQuery,
        object::{ObjectPathInner, SetObjectPath},
        url_from_bucket, BucketName, CanonicalizedResource, EndPoint, InvalidBucketName,
        InvalidEndPoint, KeyId, KeySecret,
    },
    Query,
};
use reqwest::Url;
use std::{
    env::{self, VarError},
    error::Error,
    fmt::Display,
    str::FromStr,
};
use thiserror::Error;

const HTTPS: &str = "https://";
const OSS_HYPHEN: &str = "oss-";

/// OSS 配置信息
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Config {
    key: KeyId,
    secret: KeySecret,
    endpoint: EndPoint,
    bucket: BucketName,
}

impl AsRef<KeyId> for Config {
    fn as_ref(&self) -> &KeyId {
        &self.key
    }
}

impl AsRef<KeySecret> for Config {
    fn as_ref(&self) -> &KeySecret {
        &self.secret
    }
}

impl AsRef<EndPoint> for Config {
    fn as_ref(&self) -> &EndPoint {
        &self.endpoint
    }
}

impl AsRef<BucketName> for Config {
    fn as_ref(&self) -> &BucketName {
        &self.bucket
    }
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

    /// use env init Config
    pub fn from_env() -> Result<Self, InvalidConfig> {
        let key_id = get_env("ALIYUN_KEY_ID")?;
        let key_secret = get_env("ALIYUN_KEY_SECRET")?;

        let endpoint = EndPoint::from_env().map_err(|e| InvalidConfig {
            source: String::default(),
            kind: InvalidConfigKind::EndPoint(e),
        })?;
        Ok(Config {
            key: key_id.into(),
            secret: key_secret.into(),
            endpoint,
            bucket: BucketName::from_env().map_err(|e| InvalidConfig {
                source: String::default(),
                kind: InvalidConfigKind::BucketName(e),
            })?,
        })
    }

    /// 初始化 OSS 配置信息
    ///
    /// [未稳定] 暂不公开
    ///
    /// 支持更宽泛的输入类型
    #[cfg(test)]
    pub(crate) fn try_new<ID, S, E, B>(
        key: ID,
        secret: S,
        endpoint: E,
        bucket: B,
    ) -> Result<Config, InvalidConfig>
    where
        ID: Into<KeyId>,
        S: Into<KeySecret>,
        E: TryInto<EndPoint> + Display + Clone,
        E::Error: Into<InvalidEndPoint>,
        B: TryInto<BucketName> + Display + Clone,
        B::Error: Into<InvalidBucketName>,
    {
        Ok(Config {
            key: key.into(),
            secret: secret.into(),
            endpoint: endpoint.clone().try_into().map_err(|e| InvalidConfig {
                source: endpoint.to_string(),
                kind: InvalidConfigKind::EndPoint(e.into()),
            })?,
            bucket: bucket.clone().try_into().map_err(|e| InvalidConfig {
                source: bucket.to_string(),
                kind: InvalidConfigKind::BucketName(e.into()),
            })?,
        })
    }

    pub(crate) fn get_all(self) -> (KeyId, KeySecret, BucketName, EndPoint) {
        (self.key, self.secret, self.bucket, self.endpoint)
    }

    #[allow(dead_code)]
    pub(crate) fn get_all_ref(&self) -> (&KeyId, &KeySecret, &BucketName, &EndPoint) {
        (&self.key, &self.secret, &self.bucket, &self.endpoint)
    }
}

pub(crate) fn get_env(name: &str) -> Result<String, InvalidConfig> {
    env::var(name).map_err(|e| InvalidConfig {
        source: name.to_owned(),
        kind: InvalidConfigKind::VarError(e),
    })
}

pub(crate) fn get_endpoint(name: &str) -> Result<EndPoint, InvalidConfig> {
    EndPoint::try_from(name).map_err(|e| InvalidConfig {
        source: name.to_string(),
        kind: InvalidConfigKind::EndPoint(e),
    })
}

pub(crate) fn get_bucket(name: &str) -> Result<BucketName, InvalidConfig> {
    BucketName::try_from(name).map_err(|e| InvalidConfig {
        source: name.to_string(),
        kind: InvalidConfigKind::BucketName(e),
    })
}

/// Config 错误信息集合
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct InvalidConfig {
    source: String,
    kind: InvalidConfigKind,
}

impl InvalidConfig {
    #[cfg(test)]
    pub(crate) fn test_bucket() -> Self {
        Self {
            source: "bar".into(),
            kind: InvalidConfigKind::BucketName(InvalidBucketName::new()),
        }
    }
}

impl Display for InvalidConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InvalidConfigKind::*;
        match &self.kind {
            EndPoint(_) | BucketName(_) => write!(f, "get config failed, source: {}", self.source),
            VarError(_) => write!(f, "get config failed, env name: {}", self.source),
        }
    }
}
impl Error for InvalidConfig {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use InvalidConfigKind::*;
        match &self.kind {
            EndPoint(e) => Some(e),
            BucketName(e) => Some(e),
            VarError(e) => Some(e),
        }
    }
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
enum InvalidConfigKind {
    /// 非法的可用区
    EndPoint(InvalidEndPoint),

    /// 非法的 bucket 名称
    BucketName(InvalidBucketName),

    /// 非法的环境变量
    VarError(VarError),
}

/// # Bucket 元信息
/// 包含所属 bucket 名以及所属的 endpoint
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BucketBase {
    endpoint: EndPoint,
    name: BucketName,
}

impl AsMut<EndPoint> for BucketBase {
    fn as_mut(&mut self) -> &mut EndPoint {
        &mut self.endpoint
    }
}

impl AsMut<BucketName> for BucketBase {
    fn as_mut(&mut self) -> &mut BucketName {
        &mut self.name
    }
}

impl AsRef<EndPoint> for BucketBase {
    fn as_ref(&self) -> &EndPoint {
        &self.endpoint
    }
}

impl AsRef<BucketName> for BucketBase {
    fn as_ref(&self) -> &BucketName {
        &self.name
    }
}

impl FromStr for BucketBase {
    type Err = InvalidBucketBase;
    /// 通过域名获取
    /// 举例
    /// ```
    /// # use aliyun_oss_client::config::BucketBase;
    /// # use aliyun_oss_client::types::EndPoint;
    /// # use std::borrow::Cow;
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
            return Err(InvalidBucketBase::new_with_str(
                domain,
                InvalidBucketBaseKind::Tacitly,
            ));
        }

        let (bucket, endpoint) = domain
            .split_once('.')
            .ok_or(InvalidBucketBase::new_with_str(
                domain,
                InvalidBucketBaseKind::Tacitly,
            ))?;
        let endpoint = match endpoint.find('.') {
            Some(s) => &endpoint[0..s],
            None => endpoint,
        };

        Ok(Self {
            name: BucketName::from_static(bucket)
                .map_err(|e| InvalidBucketBase::from_kind(bucket, e))?,
            endpoint: EndPoint::new(endpoint.trim_start_matches(OSS_HYPHEN))
                .map_err(|e| InvalidBucketBase::from_kind(endpoint, e))?,
        })
    }
}

impl TryFrom<&str> for BucketBase {
    type Error = InvalidBucketBase;
    fn try_from(str: &str) -> Result<Self, Self::Error> {
        str.parse()
    }
}

/// Bucket 元信息的错误集
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct InvalidBucketBase {
    source: String,
    kind: InvalidBucketBaseKind,
}

impl InvalidBucketBase {
    pub(crate) fn new(source: String, kind: InvalidBucketBaseKind) -> Self {
        Self { source, kind }
    }

    pub(crate) fn new_with_str(source: &str, kind: InvalidBucketBaseKind) -> Self {
        Self::new(source.to_string(), kind)
    }

    pub(crate) fn from_kind<K: Into<InvalidBucketBaseKind>>(source: &str, kind: K) -> Self {
        Self::new(source.to_string(), kind.into())
    }

    /// 原始字符串
    pub fn source_string(self) -> String {
        self.source
    }
}

impl Display for InvalidBucketBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "get bucket base faild, source: {}", self.source)
    }
}
impl Error for InvalidBucketBase {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use InvalidBucketBaseKind::*;
        match &self.kind {
            Tacitly => None,
            EndPoint(e) => Some(e),
            BucketName(e) => Some(e),
        }
    }
}

/// Bucket 元信息的错误集
#[derive(Error, Debug, Clone)]
#[non_exhaustive]
pub(crate) enum InvalidBucketBaseKind {
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

impl BucketBase {
    /// 初始化
    pub fn new(name: BucketName, endpoint: EndPoint) -> Self {
        Self { name, endpoint }
    }

    /// # 调整 API 指向是否为内网
    ///
    /// 当在 Aliyun ECS 上执行时，设为 true 会更高效，默认是 false
    pub fn set_internal(&mut self, is_internal: bool) {
        self.endpoint.is_internal = is_internal;
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
    ///
    /// 如果在 Aliyun ECS 上，可将环境变量 `ALIYUN_OSS_INTERNAL`
    /// 设置为 `true` / `1` / `yes` / `Y` ，即可使用 internal 网络请求 OSS 接口
    pub fn from_env() -> Result<Self, InvalidConfig> {
        let endpoint = env::var("ALIYUN_ENDPOINT").map_err(|e| InvalidConfig {
            source: "ALIYUN_ENDPOINT".to_string(),
            kind: InvalidConfigKind::VarError(e),
        })?;
        let mut endpoint = EndPoint::from_str(&endpoint).map_err(|e| InvalidConfig {
            source: endpoint,
            kind: InvalidConfigKind::EndPoint(e),
        })?;

        if let Ok(is_internal) = env::var("ALIYUN_OSS_INTERNAL") {
            if is_internal == TRUE1
                || is_internal == TRUE2
                || is_internal == TRUE3
                || is_internal == TRUE4
            {
                endpoint.set_internal(true);
            }
        }

        let bucket = env::var("ALIYUN_BUCKET").map_err(|e| InvalidConfig {
            source: "ALIYUN_BUCKET".to_string(),
            kind: InvalidConfigKind::VarError(e),
        })?;
        Ok(Self {
            name: BucketName::from_str(&bucket).map_err(|e| InvalidConfig {
                source: bucket,
                kind: InvalidConfigKind::BucketName(e),
            })?,
            endpoint,
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
    /// assert_eq!(
    ///     *BucketBase::from_env().unwrap().get_name(),
    ///     BucketName::new("foo1").unwrap()
    /// );
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

    /// 获取 EndPoint 引用
    #[inline]
    pub fn endpoint_ref(&self) -> &EndPoint {
        &self.endpoint
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
        Url::parse(&url).unwrap_or_else(|_| panic!("covert to url failed, url string: {}", url))
    }

    /// 根据查询参数，获取当前 bucket 的接口请求参数（ url 和 CanonicalizedResource）
    #[inline]
    pub fn get_url_resource(&self, query: &Query) -> (Url, CanonicalizedResource) {
        let mut url = self.to_url();
        url.set_oss_query(query);

        let resource = CanonicalizedResource::from_bucket_query(self, query);

        (url, resource)
    }

    /// 根据查询参数，获取当前 bucket 的接口请求参数（ url 和 CanonicalizedResource）
    pub fn get_url_resource_with_path(
        &self,
        path: &ObjectPathInner,
    ) -> (Url, CanonicalizedResource) {
        let mut url = self.to_url();
        url.set_object_path(path);

        let resource = CanonicalizedResource::from_object((self.name(), path.as_ref()), []);

        (url, resource)
    }
}

pub use crate::types::{get_url_resource, get_url_resource2};

#[doc(hidden)]
pub(crate) fn get_url_resource_with_bucket(
    endpoint: &EndPoint,
    bucket: &BucketName,
    query: &Query,
) -> (Url, CanonicalizedResource) {
    let url = url_from_bucket(endpoint, bucket);

    let resource = CanonicalizedResource::from_bucket_query2(bucket, query);

    (url, resource)
}

#[doc(hidden)]
#[allow(dead_code)]
pub(crate) fn get_url_resource_with_bucket2<E: AsRef<EndPoint>, B: AsRef<BucketName>>(
    endpoint: E,
    bucket: B,
    query: &Query,
) -> (Url, CanonicalizedResource) {
    get_url_resource_with_bucket(endpoint.as_ref(), bucket.as_ref(), query)
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

#[cfg(test)]
mod tests {
    use std::{
        borrow::Cow,
        env::{remove_var, set_var},
    };

    use crate::types::EndPointKind;

    use super::*;

    #[test]
    fn test_config_try_new() {
        let err = Config::try_new("foo", "foo", "_aa", "abc");
        let err = err.unwrap_err();
        assert!(matches!(
            err,
            InvalidConfig {
                kind: InvalidConfigKind::EndPoint(_),
                ..
            }
        ));

        let err = Config::try_new("foo", "foo", "qingdao", "-abc");
        let err = err.unwrap_err();
        assert!(matches!(
            err,
            InvalidConfig {
                kind: InvalidConfigKind::BucketName(_),
                ..
            }
        ));
    }

    fn assert_as_ref_keyid<K: AsRef<KeyId>>(k: K) {
        k.as_ref();
    }
    fn assert_as_ref_key_secret<K: AsRef<KeySecret>>(k: K) {
        k.as_ref();
    }
    fn assert_as_ref_endpoint<K: AsRef<EndPoint>>(k: K) {
        k.as_ref();
    }
    fn assert_as_ref_bucket<K: AsRef<BucketName>>(k: K) {
        k.as_ref();
    }

    #[test]
    fn test_config_as_ref() {
        let config = Config::default();
        assert_as_ref_keyid(&config);
        assert_as_ref_key_secret(&config);
        assert_as_ref_endpoint(&config);
        assert_as_ref_bucket(&config);
    }

    #[test]
    fn test_set_internal() {
        let mut base = BucketBase::new("abc".try_into().unwrap(), "qingdao".try_into().unwrap());
        let BucketBase { endpoint, .. } = base.clone();
        assert!(endpoint.is_internal == false);

        base.set_internal(true);
        let BucketBase { endpoint, .. } = base;
        assert!(endpoint.is_internal == true);
    }

    #[test]
    fn test_from_env() {
        set_var("ALIYUN_KEY_ID", "foo");
        set_var("ALIYUN_KEY_SECRET", "foo2");
        set_var("ALIYUN_ENDPOINT", "qingdao");
        set_var("ALIYUN_BUCKET", "foo3");
        remove_var("ALIYUN_OSS_INTERNAL");
        let config = Config::from_env().unwrap();
        assert_eq!(config.key.as_ref(), "foo");
        assert_eq!(config.secret.as_str(), "foo2");
        assert_eq!(&config.endpoint, &EndPoint::CN_QINGDAO);
        assert_eq!(config.bucket.as_ref(), "foo3");

        set_var("ALIYUN_ENDPOINT", "ossqd");
        let config = Config::from_env().unwrap_err();
        assert!(config.source.len() == 0);
        assert!(matches!(config.kind, InvalidConfigKind::EndPoint(_)));

        set_var("ALIYUN_ENDPOINT", "hangzhou");
        set_var("ALIYUN_BUCKET", "foo3-");
        let config = Config::from_env().unwrap_err();
        assert!(config.source.len() == 0);
        assert!(matches!(config.kind, InvalidConfigKind::BucketName(_)));
    }

    #[test]
    fn test_invalid_config() {
        let error = get_endpoint("oss").unwrap_err();
        assert_eq!(format!("{error}"), "get config failed, source: oss");
        assert_eq!(
            format!("{}", error.source().unwrap()),
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        );

        let error = get_bucket("-oss").unwrap_err();
        assert_eq!(format!("{error}"), "get config failed, source: -oss");
        assert_eq!(
            format!("{}", error.source().unwrap()),
            "bucket name only allow `alphabet, digit, -`, and must not with `-` prefix or `-` suffix"
        );

        let err = get_env("aaa").unwrap_err();
        assert_eq!(format!("{}", err), "get config failed, env name: aaa");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "environment variable not found"
        );
    }

    #[test]
    fn test_base_as() {
        fn assert_as_mut_endpoint<E: AsMut<EndPoint>>(e: &mut E) {
            e.as_mut();
        }
        fn assert_as_mut_name<E: AsMut<BucketName>>(e: &mut E) {
            e.as_mut();
        }
        fn assert_as_endpoint<E: AsRef<EndPoint>>(e: &E) {
            e.as_ref();
        }
        fn assert_as_name<E: AsRef<BucketName>>(e: &E) {
            e.as_ref();
        }

        let mut base = BucketBase::default();

        assert_as_mut_endpoint(&mut base);
        assert_as_mut_name(&mut base);
        assert_as_endpoint(&base);
        assert_as_name(&base);
    }

    #[test]
    fn test_get_url_resource_with_path() {
        let base = BucketBase::new("abc".try_into().unwrap(), EndPoint::CN_BEIJING);

        let path = "path".try_into().unwrap();
        let (url, resource) = base.get_url_resource_with_path(&path);

        assert_eq!(
            url,
            Url::parse("https://abc.oss-cn-beijing.aliyuncs.com/path").unwrap()
        );
        assert_eq!(resource, "/abc/path");
    }

    #[test]
    fn test_get_url_resource_with_bucket() {
        let endpoint = EndPoint::CN_BEIJING;
        let bucket = BucketName::new("abc").unwrap();
        let query = Query::new();

        let (url, resource) = get_url_resource_with_bucket(&endpoint, &bucket, &query);
        assert_eq!(
            url,
            Url::parse("https://abc.oss-cn-beijing.aliyuncs.com").unwrap()
        );
        assert_eq!(resource, "/abc/");
    }

    #[test]
    fn test_bucketbase_to_url() {
        use std::env::{remove_var, set_var};
        set_var("ALIYUN_ENDPOINT", "qingdao");
        set_var("ALIYUN_BUCKET", "foo1");
        remove_var("ALIYUN_OSS_INTERNAL");
        let base = BucketBase::from_env().unwrap();
        let url = base.to_url();
        assert_eq!(
            url,
            Url::parse("https://foo1.oss-cn-qingdao.aliyuncs.com").unwrap()
        );

        set_var("ALIYUN_OSS_INTERNAL", "true");
        let base = BucketBase::from_env().unwrap();
        let url = base.to_url();
        assert_eq!(
            url,
            Url::parse("https://foo1.oss-cn-qingdao-internal.aliyuncs.com").unwrap()
        );

        set_var("ALIYUN_OSS_INTERNAL", "0");
        let base = BucketBase::from_env().unwrap();
        let url = base.to_url();
        assert_eq!(
            url,
            Url::parse("https://foo1.oss-cn-qingdao.aliyuncs.com").unwrap()
        );

        set_var("ALIYUN_OSS_INTERNAL", "1");
        let base = BucketBase::from_env().unwrap();
        let url = base.to_url();
        assert_eq!(
            url,
            Url::parse("https://foo1.oss-cn-qingdao-internal.aliyuncs.com").unwrap()
        );

        set_var("ALIYUN_OSS_INTERNAL", "yes");
        let base = BucketBase::from_env().unwrap();
        let url = base.to_url();
        assert_eq!(
            url,
            Url::parse("https://foo1.oss-cn-qingdao-internal.aliyuncs.com").unwrap()
        );

        set_var("ALIYUN_OSS_INTERNAL", "Y");
        let base = BucketBase::from_env().unwrap();
        let url = base.to_url();
        assert_eq!(
            url,
            Url::parse("https://foo1.oss-cn-qingdao-internal.aliyuncs.com").unwrap()
        );

        remove_var("ALIYUN_OSS_INTERNAL");
        remove_var("ALIYUN_ENDPOINT");
        let base = BucketBase::from_env().unwrap_err();
        assert_eq!(base.source, "ALIYUN_ENDPOINT");
        assert!(matches!(base.kind, InvalidConfigKind::VarError(_)));

        set_var("ALIYUN_ENDPOINT", "ossqd");
        let base = BucketBase::from_env().unwrap_err();
        assert_eq!(base.source, "ossqd");
        assert!(matches!(base.kind, InvalidConfigKind::EndPoint(_)));

        set_var("ALIYUN_ENDPOINT", "qingdao");
        remove_var("ALIYUN_BUCKET");
        let base = BucketBase::from_env().unwrap_err();
        assert_eq!(base.source, "ALIYUN_BUCKET");
        assert!(matches!(base.kind, InvalidConfigKind::VarError(_)));

        set_var("ALIYUN_BUCKET", "abc-");
        let base = BucketBase::from_env().unwrap_err();
        assert_eq!(base.source, "abc-");
        assert!(matches!(base.kind, InvalidConfigKind::BucketName(_)));
    }

    #[test]
    fn test_invalid_bucket_base() {
        let error = InvalidEndPoint::new();
        let base_err = InvalidBucketBase {
            source: "abc".to_string(),
            kind: error.into(),
        };
        assert_eq!(format!("{base_err}"), "get bucket base faild, source: abc");
        assert_eq!(
            format!("{}", base_err.source().unwrap()),
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        );

        let error = InvalidBucketName::new();
        let error2 = InvalidBucketBase {
            source: "abc".to_string(),
            kind: error.into(),
        };
        assert_eq!(format!("{error2}"), "get bucket base faild, source: abc");
        assert_eq!(
            format!("{}", error2.source().unwrap()),
            "bucket name only allow `alphabet, digit, -`, and must not with `-` prefix or `-` suffix"
        );

        let error2 = InvalidBucketBase {
            source: "abc".to_string(),
            kind: InvalidBucketBaseKind::Tacitly,
        };
        assert_eq!(format!("{error2}"), "get bucket base faild, source: abc");
        assert!(error2.source().is_none());
    }

    #[test]
    fn test_bucket_base_from_str() {
        let err = BucketBase::from_str("-abc.oss-cn-qingdao");
        let err = err.unwrap_err();
        assert!(matches!(
            err,
            InvalidBucketBase {
                kind: InvalidBucketBaseKind::BucketName(_),
                ..
            }
        ));

        let err = BucketBase::from_str("abc.oss-cn-qing-");
        let err = err.unwrap_err();
        assert!(matches!(
            err,
            InvalidBucketBase {
                kind: InvalidBucketBaseKind::EndPoint(_),
                ..
            }
        ));

        let bucket: BucketBase = "abc.oss-cn-jinan.aliyuncs.com".parse().unwrap();
        assert_eq!(bucket.name(), "abc");
        assert_eq!(
            bucket.endpoint(),
            EndPoint {
                kind: EndPointKind::Other(Cow::Borrowed("cn-jinan")),
                is_internal: false,
            }
        );

        let bucket: BucketBase = "abc.oss-cn-jinan".parse().unwrap();
        assert_eq!(bucket.name(), "abc");
        assert_eq!(
            bucket.endpoint(),
            EndPoint {
                kind: EndPointKind::Other(Cow::Borrowed("cn-jinan")),
                is_internal: false,
            }
        );
    }

    #[test]
    fn test_bucket_base_eq_url() {
        let base = BucketBase::default();
        let url = Url::parse("https://a.oss-cn-hangzhou.aliyuncs.com/").unwrap();
        assert!(base == url);
    }
}
