use crate::types::{CanonicalizedResource, ContentMd5, ContentType, Date, KeyId, KeySecret};
#[cfg(test)]
use http::header::AsHeaderName;
use http::header::CONTENT_TYPE;
#[cfg(test)]
use mockall::automock;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, IntoHeaderName};
use reqwest::Method;
use std::convert::TryInto;
use std::fmt::Display;
use thiserror::Error;

#[derive(Default, Clone)]
pub struct Auth {
    access_key_id: KeyId,
    access_key_secret: KeySecret,
    method: Method,
    content_md5: Option<ContentMd5>,
    date: Date,
    // pub canonicalized_oss_headers: &'a str, // TODO
    canonicalized_resource: CanonicalizedResource,
    headers: HeaderMap,
}

impl Auth {
    fn set_key(&mut self, access_key_id: KeyId) {
        self.access_key_id = access_key_id;
    }

    #[cfg(test)]
    pub(crate) fn get_key(self) -> KeyId {
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
    fn get_header_key(&self) -> AuthResult<HeaderValue>;
    fn get_header_secret(&self) -> AuthResult<HeaderValue>;
    fn get_header_method(&self) -> AuthResult<HeaderValue>;
    fn get_header_md5(&self) -> AuthResult<Option<HeaderValue>>;
    fn get_header_date(&self) -> AuthResult<HeaderValue>;
    fn get_header_resource(&self) -> AuthResult<HeaderValue>;
}

impl AuthToHeaderMap for Auth {
    fn get_original_header(&self) -> HeaderMap {
        self.headers.clone()
    }
    fn get_header_key(&self) -> AuthResult<HeaderValue> {
        let val: HeaderValue = self.access_key_id.as_ref().try_into()?;
        Ok(val)
    }
    fn get_header_secret(&self) -> AuthResult<HeaderValue> {
        let val: HeaderValue = self.access_key_secret.as_ref().try_into()?;
        Ok(val)
    }
    fn get_header_method(&self) -> AuthResult<HeaderValue> {
        let val: HeaderValue = self.method.as_str().try_into()?;
        Ok(val)
    }
    fn get_header_md5(&self) -> AuthResult<Option<HeaderValue>> {
        let res = match self.content_md5.clone() {
            Some(val) => {
                let val: HeaderValue = val.try_into()?;
                Some(val)
            }
            None => None,
        };
        Ok(res)
    }
    fn get_header_date(&self) -> AuthResult<HeaderValue> {
        let val: HeaderValue = self.date.as_ref().try_into()?;
        Ok(val)
    }
    fn get_header_resource(&self) -> AuthResult<HeaderValue> {
        let val: HeaderValue = self.canonicalized_resource.as_ref().try_into()?;
        Ok(val)
    }
}

pub(crate) trait AuthToOssHeader {
    fn to_oss_header(&self) -> AuthResult<OssHeader>;
}

impl AuthToOssHeader for Auth {
    /// 转化成 OssHeader
    fn to_oss_header(&self) -> AuthResult<OssHeader> {
        //return Some("x-oss-copy-source:/honglei123/file1.txt");
        let mut header: Vec<(&HeaderName, &HeaderValue)> = self
            .headers
            .iter()
            .filter(|(k, _v)| k.as_str().starts_with("x-oss-"))
            .collect();
        if header.is_empty() {
            return Ok(OssHeader(None));
        }

        header.sort_by(|(k1, _), (k2, _)| k1.as_str().cmp(k2.as_str()));

        let header_vec: Vec<_> = header
            .iter()
            .filter_map(|(k, v)| match v.to_str() {
                Ok(val) => Some(k.as_str().to_owned() + ":" + val),
                _ => None,
            })
            .collect();

        Ok(OssHeader(Some(header_vec.join(LINE_BREAK))))
    }
}

/// 从 auth 中提取各个字段，用于计算签名的原始字符串
pub(crate) trait AuthSignString {
    fn get_sign_info(
        &self,
    ) -> (
        &KeyId,
        &KeySecret,
        &Method,
        ContentMd5,
        ContentType,
        &Date,
        &CanonicalizedResource,
    );
}

impl AuthSignString for Auth {
    #[inline]
    fn get_sign_info(
        &self,
    ) -> (
        &KeyId,
        &KeySecret,
        &Method,
        ContentMd5,
        ContentType,
        &Date,
        &CanonicalizedResource,
    ) {
        (
            &self.access_key_id,
            &self.access_key_secret,
            &self.method,
            self.content_md5.clone().unwrap_or_default(),
            match self.headers.get(CONTENT_TYPE) {
                Some(ct) => ct.to_owned().try_into().unwrap(),
                None => ContentType::default(),
            },
            &self.date,
            &self.canonicalized_resource,
        )
    }
}

pub trait AuthGetHeader {
    fn get_headers(&self) -> AuthResult<HeaderMap>;
}

impl AuthGetHeader for Auth {
    /// 返回携带了签名信息的 headers
    fn get_headers(&self) -> AuthResult<HeaderMap> {
        let mut map = HeaderMap::from_auth(self)?;

        let oss_header = self.to_oss_header()?;
        let sign_string = SignString::from_auth(self, oss_header)?;
        let sign = sign_string.to_sign()?;
        map.append_sign(sign)?;

        Ok(map)
    }
}

pub(crate) trait AuthHeader {
    fn from_auth(auth: &impl AuthToHeaderMap) -> AuthResult<Self>
    where
        Self: Sized;
    fn append_sign<S: TryInto<HeaderValue, Error = AuthError>>(
        &mut self,
        sign: S,
    ) -> AuthResult<Option<HeaderValue>>;
}

const ACCESS_KEY_ID: &str = "AccessKeyId";
const SECRET_ACCESS_KEY: &str = "SecretAccessKey";
const VERB_IDENT: &str = "VERB";
const CONTENT_MD5: &str = "Content-MD5";
const DATE: &str = "Date";
const CANONICALIZED_RESOURCE: &str = "CanonicalizedResource";
const AUTHORIZATION: &str = "Authorization";

impl AuthHeader for HeaderMap {
    fn from_auth(auth: &impl AuthToHeaderMap) -> AuthResult<Self> {
        let mut map = auth.get_original_header();

        map.insert(ACCESS_KEY_ID, auth.get_header_key()?);
        map.insert(SECRET_ACCESS_KEY, auth.get_header_secret()?);
        map.insert(VERB_IDENT, auth.get_header_method()?);

        if let Some(a) = auth.get_header_md5()? {
            map.insert(CONTENT_MD5, a);
        }
        map.insert(DATE, auth.get_header_date()?);
        map.insert(CANONICALIZED_RESOURCE, auth.get_header_resource()?);

        //println!("header list: {:?}",map);
        Ok(map)
    }
    fn append_sign<S: TryInto<HeaderValue, Error = AuthError>>(
        &mut self,
        sign: S,
    ) -> AuthResult<Option<HeaderValue>> {
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
        match &self.0 {
            Some(str) => str.len(),
            None => 0,
        }
    }
}

impl Display for OssHeader {
    /// 转化成 SignString 需要的格式
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut content = String::with_capacity(self.len() + 2);
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
    key: &'a KeyId,
    secret: &'a KeySecret,
}

const LINE_BREAK: &str = "\n";

impl<'a, 'b> SignString<'_> {
    #[allow(dead_code)]
    pub(crate) fn new(data: &'b str, key: &'a KeyId, secret: &'a KeySecret) -> SignString<'a> {
        SignString {
            data: data.to_owned(),
            key,
            secret,
        }
    }
}

impl<'a> SignString<'a> {
    pub(crate) fn from_auth(
        auth: &impl AuthSignString,
        header: OssHeader,
    ) -> AuthResult<SignString> {
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
            + header.to_string().as_str()
            + canonicalized_resource.as_ref();

        Ok(SignString { data, key, secret })
    }

    #[cfg(test)]
    pub fn data(&self) -> String {
        self.data.clone()
    }

    #[cfg(test)]
    pub(crate) fn key_string(&self) -> String {
        self.key.to_string()
    }

    #[cfg(test)]
    pub(crate) fn secret_string(&self) -> String {
        self.secret.to_string()
    }

    // 转化成签名
    #[inline]
    pub(crate) fn to_sign(&self) -> AuthResult<Sign<'a>> {
        use base64::encode;
        use hmac::{Hmac, Mac};
        use sha1::Sha1;
        type HmacSha1 = Hmac<Sha1>;

        let secret = self.secret.as_bytes();
        let data_u8 = self.data.as_bytes();

        let mut mac = HmacSha1::new_from_slice(secret)?;

        mac.update(data_u8);

        let sha1 = mac.finalize().into_bytes();

        Ok(Sign {
            data: encode(sha1),
            key: self.key,
        })
    }
}

/// header 中的签名
pub(crate) struct Sign<'a> {
    data: String,
    key: &'a KeyId,
}

impl Sign<'_> {
    #[cfg(test)]
    pub(crate) fn new<'a, 'b>(data: &'b str, key: &'a KeyId) -> Sign<'a> {
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
        self.key.clone().to_string()
    }
}

impl TryInto<HeaderValue> for Sign<'_> {
    type Error = AuthError;

    /// 转化成 header 中需要的格式
    fn try_into(self) -> AuthResult<HeaderValue> {
        let sign = format!("OSS {}:{}", self.key, self.data);
        Ok(sign.parse()?)
    }
}

#[derive(Default, Clone)]
pub struct AuthBuilder {
    auth: Auth,
}

#[cfg_attr(test, mockall::automock)]
impl AuthBuilder {
    /// 给 key 赋值
    ///
    /// ```
    /// # use aliyun_oss_client::auth::AuthBuilder;
    /// # use aliyun_oss_client::auth::AuthGetHeader;
    /// let mut headers = AuthBuilder::default();
    /// headers.key("bar".into());
    /// headers.get_headers();
    /// ```
    pub fn key(&mut self, key: KeyId) {
        self.auth.set_key(key);
    }

    /// 给 secret 赋值
    pub fn secret(&mut self, secret: KeySecret) {
        self.auth.set_secret(secret);
    }

    /// 给 verb 赋值
    pub fn method(&mut self, method: &Method) {
        self.auth.set_method(method.to_owned());
    }

    /// 给 content_md5 赋值
    pub fn content_md5(&mut self, content_md5: ContentMd5) {
        self.auth.set_content_md5(content_md5);
    }

    /// # 给 date 赋值
    ///
    /// ## Example
    /// ```
    /// use chrono::Utc;
    /// let builder = aliyun_oss_client::auth::AuthBuilder::default().date(Utc::now().into());
    /// ```
    pub fn date(&mut self, date: Date) {
        self.auth.set_date(date);
    }

    /// 给 content_md5 赋值
    pub fn canonicalized_resource(&mut self, data: CanonicalizedResource) {
        self.auth.set_canonicalized_resource(data);
    }

    pub fn with_headers(&mut self, headers: Option<HeaderMap>) {
        if let Some(headers) = headers {
            self.extend_headers(headers);
        }
    }

    pub fn headers(&mut self, headers: HeaderMap) {
        self.auth.set_headers(headers);
    }

    pub fn extend_headers(&mut self, headers: HeaderMap) {
        self.auth.extend_headers(headers);
    }

    /// 给 header 序列添加新值
    pub fn header_insert<K: IntoHeaderName + 'static>(&mut self, key: K, val: HeaderValue) {
        self.auth.header_insert(key, val);
    }

    /// 清理 headers
    pub fn header_clear(&mut self) {
        self.auth.headers_clear();
    }

    #[allow(dead_code)]
    pub(crate) fn build(self) -> Auth {
        self.auth
    }
}

impl AuthGetHeader for AuthBuilder {
    /// 返回携带了签名信息的 headers
    fn get_headers(&self) -> AuthResult<HeaderMap> {
        self.auth.get_headers()
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("invalid header value msg: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("hmac InvalidLength: {0}")]
    InvalidLength(#[from] hmac::digest::crypto_common::InvalidLength),
}

type AuthResult<T> = Result<T, AuthError>;

#[cfg(test)]
mod builder_tests {
    use super::AuthBuilder;

    #[test]
    fn key() {
        let builder = AuthBuilder::default();
        assert_eq!(builder.build().get_key().as_ref(), "");

        let mut builder = AuthBuilder::default();
        builder.key("bar".into());
        assert_eq!(builder.build().get_key().as_ref(), "bar");
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
}
