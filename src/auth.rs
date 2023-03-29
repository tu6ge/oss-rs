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

use crate::types::{
    CanonicalizedResource, ContentMd5, ContentType, Date, InnerCanonicalizedResource,
    InnerContentMd5, InnerDate, InnerKeyId, InnerKeySecret, KeyId, KeySecret,
};
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

/// 收集 Auth 模块的错误
#[derive(Debug)]
pub enum AuthError {
    #[doc(hidden)]
    InvalidHeaderValue(http::header::InvalidHeaderValue),
    #[doc(hidden)]
    InvalidLength(hmac::digest::crypto_common::InvalidLength),
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
