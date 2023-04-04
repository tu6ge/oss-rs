//! # Auth 模块
//! 计算 OSS API 的签名，并将数据收集到 `http::header::HeaderMap` 中
//!
//! ## Examples
//! ```rust
//! # use aliyun_oss_client::auth::AuthBuilder;
//! use chrono::Utc;
//! use http::Method;
//! let mut auth_builder = AuthBuilder::default();
//! auth_builder.key("my_key");
//! auth_builder.secret("my_secret");
//! auth_builder.method(&Method::GET);
//! auth_builder.date(Utc::now());
//! auth_builder.canonicalized_resource("/abc/?bucketInfo");
//!
//! let builder = reqwest::Client::default()
//!     .request(
//!         Method::GET,
//!         "https://abc.oss-cn-shanghai.aliyuncs.com/?bucketInfo",
//!     )
//!     .headers(auth_builder.get_headers().unwrap());
//! ```

use crate::{
    types::{
        object::ObjectPathInner, CanonicalizedResource, ContentMd5, ContentType, Date,
        InnerCanonicalizedResource, InnerContentMd5, InnerDate, InnerKeyId, InnerKeySecret, KeyId,
        KeySecret,
    },
    BucketName, EndPoint, Query, QueryKey, QueryValue,
};
use chrono::Utc;
#[cfg(test)]
use http::header::AsHeaderName;
use http::{
    header::{HeaderMap, HeaderValue, IntoHeaderName, InvalidHeaderValue, CONTENT_TYPE},
    Method,
};
#[cfg(test)]
use mockall::automock;
use std::convert::TryInto;
use std::fmt::Display;

/// 计算 OSS 签名的数据
#[derive(Default, Clone)]
pub struct InnerAuth<'a> {
    access_key_id: InnerKeyId<'a>,
    access_key_secret: InnerKeySecret<'a>,
    method: Method,
    content_md5: Option<InnerContentMd5<'a>>,
    date: InnerDate<'a>,
    canonicalized_resource: InnerCanonicalizedResource<'a>,
    headers: HeaderMap,
}
/// 静态作用域的 InnerAuth
pub type Auth = InnerAuth<'static>;

impl<'a> InnerAuth<'a> {
    fn set_key(&mut self, access_key_id: InnerKeyId<'a>) {
        self.access_key_id = access_key_id;
    }

    #[cfg(test)]
    pub(crate) fn get_key(self) -> InnerKeyId<'a> {
        self.access_key_id
    }

    fn set_secret(&mut self, secret: KeySecret) {
        self.access_key_secret = secret;
    }
    fn set_method(&mut self, method: Method) {
        self.method = method;
    }
    fn set_content_md5(&mut self, content_md5: ContentMd5) {
        self.content_md5 = Some(content_md5)
    }
    fn set_date(&mut self, date: Date) {
        self.date = date;
    }
    fn set_canonicalized_resource(&mut self, canonicalized_resource: CanonicalizedResource) {
        self.canonicalized_resource = canonicalized_resource;
    }
    fn set_headers(&mut self, headers: HeaderMap) {
        self.headers = headers;
    }
    fn extend_headers(&mut self, headers: HeaderMap) {
        self.headers.extend(headers);
    }
    fn header_insert<K: IntoHeaderName + 'static>(&mut self, key: K, val: HeaderValue) {
        self.headers.insert(key, val);
    }
    fn headers_clear(&mut self) {
        self.headers.clear();
    }

    #[cfg(test)]
    pub(crate) fn get_header<K>(self, key: K) -> Option<HeaderValue>
    where
        K: AsHeaderName,
    {
        self.headers.get(key).cloned()
    }

    #[cfg(test)]
    pub(crate) fn header_len(&self) -> usize {
        self.headers.len()
    }

    #[cfg(test)]
    pub(crate) fn header_contains_key<K>(&self, key: K) -> bool
    where
        K: AsHeaderName,
    {
        self.headers.contains_key(key)
    }
}

#[cfg_attr(test, automock)]
pub(crate) trait AuthToHeaderMap {
    fn get_original_header(&self) -> HeaderMap;
    fn get_header_key(&self) -> Result<HeaderValue, InvalidHeaderValue>;
    fn get_header_secret(&self) -> Result<HeaderValue, InvalidHeaderValue>;
    fn get_header_method(&self) -> Result<HeaderValue, InvalidHeaderValue>;
    fn get_header_md5(&self) -> Option<HeaderValue>;
    fn get_header_date(&self) -> Result<HeaderValue, InvalidHeaderValue>;
    fn get_header_resource(&self) -> Result<HeaderValue, InvalidHeaderValue>;
}

impl AuthToHeaderMap for InnerAuth<'_> {
    fn get_original_header(&self) -> HeaderMap {
        // 7 = 6 + 1
        let mut header = HeaderMap::with_capacity(7 + self.headers.len());
        header.extend(self.headers.clone());
        header
    }
    fn get_header_key(&self) -> Result<HeaderValue, InvalidHeaderValue> {
        self.access_key_id.as_ref().try_into()
    }
    fn get_header_secret(&self) -> Result<HeaderValue, InvalidHeaderValue> {
        self.access_key_secret.as_ref().try_into()
    }
    fn get_header_method(&self) -> Result<HeaderValue, InvalidHeaderValue> {
        self.method.as_str().try_into()
    }
    fn get_header_md5(&self) -> Option<HeaderValue> {
        self.content_md5
            .as_ref()
            .and_then(|val| TryInto::<HeaderValue>::try_into(val).ok())
    }
    fn get_header_date(&self) -> Result<HeaderValue, InvalidHeaderValue> {
        self.date.as_ref().try_into()
    }
    fn get_header_resource(&self) -> Result<HeaderValue, InvalidHeaderValue> {
        self.canonicalized_resource.as_ref().try_into()
    }
}

pub(crate) trait AuthToOssHeader {
    fn to_oss_header(&self) -> OssHeader;
}

impl AuthToOssHeader for InnerAuth<'_> {
    /// 转化成 OssHeader
    fn to_oss_header(&self) -> OssHeader {
        //return Some("x-oss-copy-source:/honglei123/file1.txt");
        let mut header: Vec<_> = self
            .headers
            .iter()
            .filter(|(k, _v)| k.as_str().starts_with("x-oss-"))
            .collect();
        if header.is_empty() {
            return OssHeader(None);
        }

        header.sort_by(|(k1, _), (k2, _)| k1.as_str().cmp(k2.as_str()));

        let header_vec: Vec<_> = header
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|value| k.as_str().to_owned() + ":" + value)
            })
            .collect();

        OssHeader(Some(header_vec.join(LINE_BREAK)))
    }
}

/// 从 auth 中提取各个字段，用于计算签名的原始字符串
pub(crate) trait AuthSignString {
    fn get_sign_info(
        &self,
    ) -> (
        &InnerKeyId,
        &InnerKeySecret,
        &Method,
        InnerContentMd5,
        ContentType,
        &InnerDate,
        &InnerCanonicalizedResource,
    );
}

impl AuthSignString for InnerAuth<'_> {
    #[inline]
    fn get_sign_info(
        &self,
    ) -> (
        &InnerKeyId,
        &InnerKeySecret,
        &Method,
        InnerContentMd5,
        ContentType,
        &InnerDate,
        &InnerCanonicalizedResource,
    ) {
        (
            &self.access_key_id,
            &self.access_key_secret,
            &self.method,
            self.content_md5.clone().unwrap_or_default(),
            self.headers
                .get(CONTENT_TYPE)
                .map_or(ContentType::default(), |ct| {
                    ct.to_owned().try_into().unwrap_or_else(|_| {
                        unreachable!("HeaderValue always is a rightful ContentType")
                    })
                }),
            &self.date,
            &self.canonicalized_resource,
        )
    }
}

impl InnerAuth<'_> {
    /// 返回携带了签名信息的 headers
    pub fn get_headers(&self) -> AuthResult<HeaderMap> {
        let mut map = HeaderMap::from_auth(self)?;

        let oss_header = self.to_oss_header();
        let sign_string = SignString::from_auth(self, oss_header);
        map.append_sign(sign_string.to_sign().map_err(AuthError::from)?)?;

        Ok(map)
    }
    /// 将 Auth 信息计算后附加到 HeaderMap 上
    fn append_headers(&self, headers: &mut HeaderMap) -> AuthResult<()> {
        headers.append_auth(self)?;
        let oss_header = self.to_oss_header();
        let sign_string = SignString::from_auth(self, oss_header);
        headers.append_sign(sign_string.to_sign().map_err(AuthError::from)?)?;

        Ok(())
    }
}

pub(crate) trait AuthHeader {
    fn from_auth(auth: &impl AuthToHeaderMap) -> Result<Self, InvalidHeaderValue>
    where
        Self: Sized;
    fn append_sign<S: TryInto<HeaderValue, Error = InvalidHeaderValue>>(
        &mut self,
        sign: S,
    ) -> Result<Option<HeaderValue>, InvalidHeaderValue>;
}

const ACCESS_KEY_ID: &str = "AccessKeyId";
const SECRET_ACCESS_KEY: &str = "SecretAccessKey";
const VERB_IDENT: &str = "VERB";
const CONTENT_MD5: &str = "Content-MD5";
const DATE: &str = "Date";
const CANONICALIZED_RESOURCE: &str = "CanonicalizedResource";
const AUTHORIZATION: &str = "Authorization";

impl AuthHeader for HeaderMap {
    fn from_auth(auth: &impl AuthToHeaderMap) -> Result<Self, InvalidHeaderValue> {
        let mut map = auth.get_original_header();

        map.insert(ACCESS_KEY_ID, auth.get_header_key()?);
        map.insert(SECRET_ACCESS_KEY, auth.get_header_secret()?);
        map.insert(VERB_IDENT, auth.get_header_method()?);

        if let Some(a) = auth.get_header_md5() {
            map.insert(CONTENT_MD5, a);
        }

        map.insert(DATE, auth.get_header_date()?);
        map.insert(CANONICALIZED_RESOURCE, auth.get_header_resource()?);

        //println!("header list: {:?}",map);
        Ok(map)
    }
    fn append_sign<S: TryInto<HeaderValue, Error = InvalidHeaderValue>>(
        &mut self,
        sign: S,
    ) -> Result<Option<HeaderValue>, InvalidHeaderValue> {
        let res = self.insert(AUTHORIZATION, sign.try_into()?);
        Ok(res)
    }
}

trait AppendAuthHeader {
    fn append_auth<'a>(&'a mut self, auth: &InnerAuth<'a>) -> Result<(), InvalidHeaderValue>;
}

impl AppendAuthHeader for HeaderMap {
    fn append_auth<'a>(&'a mut self, auth: &InnerAuth<'a>) -> Result<(), InvalidHeaderValue> {
        self.extend(auth.get_original_header());

        self.insert(ACCESS_KEY_ID, auth.get_header_key()?);
        self.insert(SECRET_ACCESS_KEY, auth.get_header_secret()?);
        self.insert(VERB_IDENT, auth.get_header_method()?);

        if let Some(a) = auth.get_header_md5() {
            self.insert(CONTENT_MD5, a);
        }

        self.insert(DATE, auth.get_header_date()?);
        self.insert(CANONICALIZED_RESOURCE, auth.get_header_resource()?);

        Ok(())
    }
}

/// # 前缀是 x-oss- 的 header 记录
///
/// 将他们按顺序组合成一个字符串，用于计算签名
pub(crate) struct OssHeader(Option<String>);

impl OssHeader {
    #[allow(dead_code)]
    pub(crate) fn new(string: Option<String>) -> Self {
        Self(string)
    }

    #[allow(dead_code)]
    pub(crate) fn is_none(&self) -> bool {
        self.0.is_none()
    }

    #[inline]
    fn len(&self) -> usize {
        self.0.as_ref().map_or(0_usize, |str| str.len())
    }
}

impl Display for OssHeader {
    /// 转化成 SignString 需要的格式
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut content = String::with_capacity({
            let len = self.len();
            if len > 0 {
                len + 2
            } else {
                0
            }
        });
        if let Some(str) = &self.0 {
            content.push_str(str);
            content.push_str(LINE_BREAK);
        }
        write!(f, "{}", content)
    }
}

/// 待签名的数据
#[derive(Debug)]
pub(crate) struct SignString<'a> {
    data: String,
    key: InnerKeyId<'a>,
    secret: InnerKeySecret<'a>,
}

const LINE_BREAK: &str = "\n";

impl<'a, 'b> SignString<'_> {
    #[allow(dead_code)]
    #[inline]
    pub(crate) fn new(
        data: &'b str,
        key: InnerKeyId<'a>,
        secret: InnerKeySecret<'a>,
    ) -> SignString<'a> {
        SignString {
            data: data.to_owned(),
            key,
            secret,
        }
    }
}

impl<'a> SignString<'a> {
    pub(crate) fn from_auth(auth: &'a impl AuthSignString, header: OssHeader) -> SignString {
        let (key, secret, verb, content_md5, content_type, date, canonicalized_resource) =
            auth.get_sign_info();
        let method = verb.to_string();

        let data = method
            + LINE_BREAK
            + content_md5.as_ref()
            + LINE_BREAK
            + content_type.as_ref()
            + LINE_BREAK
            + date.as_ref()
            + LINE_BREAK
            + &header.to_string()
            + canonicalized_resource.as_ref();

        SignString {
            data,
            key: key.clone(),
            secret: secret.clone(),
        }
    }

    #[cfg(test)]
    pub fn data(&self) -> String {
        self.data.clone()
    }

    #[cfg(test)]
    pub(crate) fn key_string(&self) -> String {
        self.key.as_ref().to_string()
    }

    #[cfg(test)]
    pub(crate) fn secret_string(&self) -> String {
        self.secret.as_ref().to_string()
    }

    // 转化成签名
    #[inline]
    pub(crate) fn to_sign(&self) -> Result<Sign, hmac::digest::crypto_common::InvalidLength> {
        use base64::engine::general_purpose::STANDARD;
        use base64::Engine;
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        type HmacSha1 = Hmac<Sha1>;

        let secret = self.secret.as_ref().as_bytes();
        let data_u8 = self.data.as_bytes();

        let mut mac = HmacSha1::new_from_slice(secret)?;

        mac.update(data_u8);

        let sha1 = mac.finalize().into_bytes();

        Ok(Sign {
            data: STANDARD.encode(sha1),
            key: self.key.clone(),
        })
    }
}

/// header 中的签名
#[derive(Debug)]
pub(crate) struct Sign<'a> {
    data: String,
    key: InnerKeyId<'a>,
}

impl Sign<'_> {
    #[cfg(test)]
    pub(crate) fn new<'a, 'b>(data: &'b str, key: InnerKeyId<'a>) -> Sign<'a> {
        Sign {
            data: data.to_owned(),
            key,
        }
    }

    #[cfg(test)]
    pub fn data(&self) -> &str {
        &self.data
    }

    #[cfg(test)]
    pub fn key_string(&self) -> String {
        self.key.as_ref().to_string()
    }
}

impl TryInto<HeaderValue> for Sign<'_> {
    type Error = InvalidHeaderValue;

    /// 转化成 header 中需要的格式
    fn try_into(self) -> Result<HeaderValue, Self::Error> {
        let sign = format!("OSS {}:{}", self.key.as_ref(), self.data);
        sign.parse()
    }
}

/// Auth 结构体的构建器
#[derive(Default, Clone)]
pub struct AuthBuilder {
    auth: Auth,
}

impl AuthBuilder {
    /// 给 key 赋值
    ///
    /// ```
    /// # use aliyun_oss_client::auth::AuthBuilder;
    /// let mut headers = AuthBuilder::default();
    /// headers.key("bar");
    /// headers.get_headers();
    /// ```
    #[inline]
    pub fn key<K: Into<KeyId>>(&mut self, key: K) {
        self.auth.set_key(key.into());
    }

    /// 给 secret 赋值
    #[inline]
    pub fn secret<S: Into<KeySecret>>(&mut self, secret: S) {
        self.auth.set_secret(secret.into());
    }

    /// 给 verb 赋值
    #[inline]
    pub fn method(&mut self, method: &Method) {
        self.auth.set_method(method.to_owned());
    }

    /// 给 content_md5 赋值
    #[inline]
    pub fn content_md5<Md5: Into<ContentMd5>>(&mut self, content_md5: Md5) {
        self.auth.set_content_md5(content_md5.into());
    }

    /// # 给 date 赋值
    ///
    /// ## Example
    /// ```
    /// use chrono::Utc;
    /// let builder = aliyun_oss_client::auth::AuthBuilder::default().date(Utc::now());
    /// ```
    #[inline]
    pub fn date<D: Into<Date>>(&mut self, date: D) {
        self.auth.set_date(date.into());
    }

    /// 给 content_md5 赋值
    #[inline]
    pub fn canonicalized_resource<Res: Into<CanonicalizedResource>>(&mut self, data: Res) {
        self.auth.set_canonicalized_resource(data.into());
    }

    /// 给 Auth 附加新的 headers 信息
    #[inline]
    pub fn with_headers(&mut self, headers: Option<HeaderMap>) {
        if let Some(headers) = headers {
            self.extend_headers(headers);
        }
    }

    /// 给 Auth 设置全新的 headers 信息
    #[inline]
    pub fn headers(&mut self, headers: HeaderMap) {
        self.auth.set_headers(headers);
    }

    /// 给 Auth 附加新的 headers 信息
    #[inline]
    pub fn extend_headers(&mut self, headers: HeaderMap) {
        self.auth.extend_headers(headers);
    }

    /// 给 header 序列添加新值
    #[inline]
    pub fn header_insert<K: IntoHeaderName + 'static>(&mut self, key: K, val: HeaderValue) {
        self.auth.header_insert(key, val);
    }

    /// 清理 headers
    #[inline]
    pub fn header_clear(&mut self) {
        self.auth.headers_clear();
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn build(self) -> Auth {
        self.auth
    }
}

impl AuthBuilder {
    /// 返回携带了签名信息的 headers
    pub fn get_headers(&self) -> AuthResult<HeaderMap> {
        self.auth.get_headers()
    }
}

/// 将 OSS 签名信息附加到 Request 中
pub trait RequestWithOSS {
    /// 输入 key，secret，以及 Request 中的 method，header,url，query
    /// 等信息，计算 OSS 签名
    /// 并把签名后的 header 信息，传递给 self
    fn with_oss(&mut self, key: KeyId, secret: KeySecret) -> AuthResult<()>;
}

use reqwest::{Request, Url};

impl RequestWithOSS for Request {
    fn with_oss(&mut self, key: KeyId, secret: KeySecret) -> AuthResult<()> {
        let mut auth = InnerAuth::default();
        auth.set_key(key);
        auth.set_secret(secret);

        auth.set_method(self.method().clone());
        auth.set_date(Utc::now().into());
        auth.set_canonicalized_resource(
            self.url()
                .canonicalized_resource()
                .ok_or(AuthError::InvalidCanonicalizedResource)?,
        );

        auth.append_headers(self.headers_mut())?;

        Ok(())
    }
}

/// 根据 Url 计算 [`CanonicalizedResource`]
///
/// [`CanonicalizedResource`]: crate::types::CanonicalizedResource
pub trait GenCanonicalizedResource {
    /// 计算并返回 [`CanonicalizedResource`]， 无法计算则返回 `None`
    ///
    /// [`CanonicalizedResource`]: crate::types::CanonicalizedResource
    fn canonicalized_resource(&self) -> Option<CanonicalizedResource>;

    /// 根据 Url 计算 bucket 名称和 Endpoint
    fn oss_host(&self) -> OssHost;

    /// 根据 Url 的 query 计算 [`Query`]
    ///
    /// [`Query`]: crate::types::Query
    fn oss_query(&self) -> Query;

    /// 根据 Url 的 path 计算当前使用的 [`ObjectPathInner`]
    ///
    /// [`ObjectPathInner`]: crate::config::ObjectPathInner
    fn object_path(&self) -> Option<ObjectPathInner>;
}

/// Oss 域名的几种状态
#[derive(PartialEq, Debug, Eq)]
pub enum OssHost {
    /// 有 bucket 的，包含 bucket 名字
    Bucket(BucketName),
    /// 只有 endpoint
    EndPoint,
    /// 其他
    None,
}

const LIST_TYPE2: &str = "list-type=2";
const LIST_TYPE2_AND: &str = "list-type=2&";
const COM: &str = "com";
const ALIYUNCS: &str = "aliyuncs";

impl GenCanonicalizedResource for Url {
    fn canonicalized_resource(&self) -> Option<CanonicalizedResource> {
        use crate::types::BUCKET_INFO;

        let bucket = match self.oss_host() {
            OssHost::None => return None,
            OssHost::EndPoint => return Some(CanonicalizedResource::from_endpoint()),
            OssHost::Bucket(bucket) => bucket,
        };

        if self.path().is_empty() {
            return None;
        }

        // 没有 object 的情况
        if self.path() == "/" {
            return match self.query() {
                // 查询单个bucket 信息
                Some(BUCKET_INFO) => Some(CanonicalizedResource::from_bucket_name(
                    &bucket,
                    Some(BUCKET_INFO),
                )),
                // 查 object_list
                Some(q) if q.ends_with(LIST_TYPE2) || q.contains(LIST_TYPE2_AND) => Some(
                    CanonicalizedResource::from_bucket_query2(&bucket, &self.oss_query()),
                ),
                // 其他情况待定
                _ => todo!("Unable to obtain can information based on existing query information"),
            };
        }

        // 获取 ObjectPath 失败，返回 None，否则根据 ObjectPath 计算 CanonicalizedResource
        self.object_path()
            .map(|path| CanonicalizedResource::from_object((bucket.as_ref(), path.as_ref()), []))
    }

    fn oss_host(&self) -> OssHost {
        use url::Host;
        let domain = match self.host() {
            Some(Host::Domain(domain)) => domain,
            _ => return OssHost::None,
        };

        let mut url_pieces = domain.rsplit('.');

        match (url_pieces.next(), url_pieces.next()) {
            (Some(COM), Some(ALIYUNCS)) => (),
            _ => return OssHost::None,
        }

        match url_pieces.next() {
            Some(endpoint) => match EndPoint::from_host_piece(endpoint) {
                Ok(_) => (),
                _ => return OssHost::None,
            },
            _ => return OssHost::None,
        };

        match url_pieces.next() {
            Some(bucket) => {
                if let Ok(b) = BucketName::from_static(bucket) {
                    OssHost::Bucket(b)
                } else {
                    OssHost::None
                }
            }
            None => OssHost::EndPoint,
        }
    }

    fn oss_query(&self) -> Query {
        self.query_pairs()
            .filter(|(_, val)| !val.is_empty())
            .collect()
    }

    fn object_path(&self) -> Option<ObjectPathInner> {
        use percent_encoding::percent_decode;

        let input = if self.path().starts_with('/') {
            &self.path()[1..]
        } else {
            self.path()
        }
        .as_bytes();
        ObjectPathInner::new(percent_decode(input).decode_utf8().ok()?).ok()
    }
}

impl GenCanonicalizedResource for Request {
    fn canonicalized_resource(&self) -> Option<CanonicalizedResource> {
        self.url().canonicalized_resource()
    }

    fn oss_host(&self) -> OssHost {
        self.url().oss_host()
    }

    fn oss_query(&self) -> Query {
        self.url().oss_query()
    }

    fn object_path(&self) -> Option<ObjectPathInner> {
        self.url().object_path()
    }
}

/// 收集 Auth 模块的错误
#[derive(Debug)]
pub enum AuthError {
    #[doc(hidden)]
    InvalidHeaderValue(http::header::InvalidHeaderValue),
    #[doc(hidden)]
    InvalidLength(hmac::digest::crypto_common::InvalidLength),
    #[doc(hidden)]
    InvalidCanonicalizedResource,
    #[doc(hidden)]
    AppendSignFailed,
}

impl std::error::Error for AuthError {}

impl Display for AuthError {
    /// Error message
    /// ```
    /// # use aliyun_oss_client::auth::AuthError;
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_str("\n");
    /// let header_error = val.unwrap_err();
    /// assert_eq!(
    ///     format!("{}", AuthError::InvalidHeaderValue(header_error)),
    ///     "failed to parse header value"
    /// );
    /// assert_eq!(
    ///     format!(
    ///         "{}",
    ///         AuthError::InvalidLength(hmac::digest::crypto_common::InvalidLength {})
    ///     ),
    ///     "Invalid hmac Length"
    /// );
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::InvalidHeaderValue(_) => f.write_str("failed to parse header value"),
            Self::InvalidLength(_) => f.write_str("Invalid hmac Length"),
            Self::InvalidCanonicalizedResource => f.write_str("Invalid CanonicalizedResource"),
            Self::AppendSignFailed => f.write_str("append signature failed"),
        }
    }
}

impl From<http::header::InvalidHeaderValue> for AuthError {
    /// ```
    /// # use aliyun_oss_client::auth::AuthError;
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_str("\n");
    /// let header_error = val.unwrap_err();
    /// let auth_error: AuthError = header_error.into();
    /// assert_eq!(format!("{}", auth_error), "failed to parse header value");
    /// ```
    fn from(value: http::header::InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValue(value)
    }
}
impl From<hmac::digest::crypto_common::InvalidLength> for AuthError {
    /// ```
    /// # use aliyun_oss_client::auth::AuthError;
    /// let hmac_error = hmac::digest::crypto_common::InvalidLength {};
    /// let auth_error: AuthError = hmac_error.into();
    /// assert_eq!(format!("{}", auth_error), "Invalid hmac Length");
    /// ```
    fn from(value: hmac::digest::crypto_common::InvalidLength) -> Self {
        Self::InvalidLength(value)
    }
}

type AuthResult<T> = Result<T, AuthError>;

#[cfg(test)]
mod builder_tests {
    use http::{header::CONTENT_LANGUAGE, HeaderMap};

    use super::AuthBuilder;

    #[test]
    fn key() {
        let builder = AuthBuilder::default();
        assert_eq!(builder.build().get_key().as_ref(), "");

        let mut builder = AuthBuilder::default();
        builder.key("bar");
        assert_eq!(builder.build().get_key().as_ref(), "bar");
    }

    #[test]
    fn with_headers() {
        let builder = AuthBuilder::default();
        let before_len = builder.build().get_headers().unwrap().len();
        assert!(before_len == 6);

        let mut builder = AuthBuilder::default();
        builder.with_headers(Some({
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_LANGUAGE, "abc".parse().unwrap());
            headers
        }));
        let len = builder.build().get_headers().unwrap().len();
        assert!(len == 7);

        let mut builder = AuthBuilder::default();
        builder.with_headers(None);
        let len = builder.build().get_headers().unwrap().len();
        assert!(len == 6);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oss_header_to_string() {
        let header = OssHeader::new(Some("foo7".to_string()));
        assert_eq!(header.to_string(), "foo7\n".to_string());

        let header = OssHeader::new(None);

        assert_eq!(header.to_string(), "".to_string());
    }

    #[test]
    fn get_sign_info() {
        let mut builder = AuthBuilder::default();
        builder.key("abc");
        let auth = builder.build();
        let (key, ..) = auth.get_sign_info();

        assert_eq!(*key, KeyId::new("abc"))
    }
}

#[cfg(test)]
mod tests_canonicalized_resource {
    use super::*;

    #[test]
    fn test_canonicalized_resource() {
        let url: Url = "https://oss2.aliyuncs.com".parse().unwrap();
        assert_eq!(url.canonicalized_resource(), None);
        let url: Url = "https://oss-cn-qingdao.aliyuncs.com".parse().unwrap();
        assert_eq!(
            url.canonicalized_resource(),
            Some(CanonicalizedResource::default())
        );

        let url: Url = "https://abc.oss-cn-qingdao.aliyuncs.com?bucketInfo"
            .parse()
            .unwrap();
        assert_eq!(
            url.canonicalized_resource(),
            Some(CanonicalizedResource::new("/abc/?bucketInfo"))
        );

        let url: Url = "https://abc.oss-cn-qingdao.aliyuncs.com?list-type=2&continuation-token=foo"
            .parse()
            .unwrap();
        assert_eq!(
            url.canonicalized_resource(),
            Some(CanonicalizedResource::new("/abc/?continuation-token=foo"))
        );

        let url: Url =
            "https://abc.oss-cn-qingdao.aliyuncs.com?continuation-token=foo&abc=def&list-type=2"
                .parse()
                .unwrap();
        assert_eq!(
            url.canonicalized_resource(),
            Some(CanonicalizedResource::new("/abc/?continuation-token=foo"))
        );

        let url: Url = "https://abc.oss-cn-qingdao.aliyuncs.com/path1"
            .parse()
            .unwrap();
        assert_eq!(
            url.canonicalized_resource(),
            Some(CanonicalizedResource::new("/abc/path1"))
        );
    }

    #[test]
    fn test_oss_host() {
        let url: Url = "https://192.168.3.10/path1?delimiter=5".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::None);

        let url: Url = "https://example.com/path1?delimiter=5".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::None);

        let url: Url = "https://oss-cn-qingdao.aliyuncs.com".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::EndPoint);

        let url: Url = "https://oss-abc.aliyuncs.com".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::EndPoint);

        let url: Url = "https://abc.aliyuncs.com".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::None);

        let url: Url = "https://abc.oss-cn-qingdao.aliyuncs.com".parse().unwrap();
        assert_eq!(
            url.oss_host(),
            OssHost::Bucket(BucketName::new("abc").unwrap())
        );
        let url: Url = "https://abc-.oss-cn-qingdao.aliyuncs.com".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::None);
    }

    #[test]
    fn test_oss_query() {
        let url: Url = "https://example.com/path1?delimiter=5".parse().unwrap();
        let query = url.oss_query();
        assert!(query[QueryKey::Delimiter] == QueryValue::new("5"));
    }

    #[test]
    fn test_object_path() {
        let url: Url = "https://example.com/path1".parse().unwrap();
        assert_eq!(
            url.object_path(),
            Some(ObjectPathInner::new("path1").unwrap())
        );

        let url: Url = "https://example.com/path1/object2".parse().unwrap();
        assert_eq!(
            url.object_path(),
            Some(ObjectPathInner::new("path1/object2").unwrap())
        );

        let url: Url = "https://example.com/路径/object2".parse().unwrap();
        assert_eq!(
            url.object_path(),
            Some(ObjectPathInner::new("路径/object2").unwrap())
        );

        let url: Url = "https://example.com/path1/object2?foo=bar".parse().unwrap();
        assert_eq!(
            url.object_path(),
            Some(ObjectPathInner::new("path1/object2").unwrap())
        );

        let url: Url = "https://example.com/path1/".parse().unwrap();
        assert_eq!(url.object_path(), None);
    }
}
