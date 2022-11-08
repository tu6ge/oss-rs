use crate::errors::{OssError, OssResult};
use crate::types::{CanonicalizedResource, ContentMd5, ContentType, Date, KeyId, KeySecret};
#[cfg(test)]
use http::header::AsHeaderName;
#[cfg(test)]
use mockall::automock;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, IntoHeaderName};
use reqwest::Method;
use std::borrow::Cow;
use std::convert::TryInto;

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
#[non_exhaustive]
pub struct VERB(pub Method);

#[derive(Default, Clone)]
pub struct Auth {
    access_key_id: KeyId,
    access_key_secret: KeySecret,
    verb: VERB,
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
    fn set_verb(&mut self, verb: VERB) {
        self.verb = verb;
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

impl VERB {
    /// GET
    pub const GET: VERB = VERB(Method::GET);

    /// POST
    pub const POST: VERB = VERB(Method::POST);

    /// PUT
    pub const PUT: VERB = VERB(Method::PUT);

    /// DELETE
    pub const DELETE: VERB = VERB(Method::DELETE);

    /// HEAD
    pub const HEAD: VERB = VERB(Method::HEAD);

    /// OPTIONS
    pub const OPTIONS: VERB = VERB(Method::OPTIONS);

    /// CONNECT
    pub const CONNECT: VERB = VERB(Method::CONNECT);

    /// PATCH
    pub const PATCH: VERB = VERB(Method::PATCH);

    /// TRACE
    pub const TRACE: VERB = VERB(Method::TRACE);

    #[inline]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl TryInto<HeaderValue> for VERB {
    type Error = OssError;
    fn try_into(self) -> OssResult<HeaderValue> {
        self.0
            .to_string()
            .parse::<HeaderValue>()
            .map_err(OssError::from)
    }
}

impl From<VERB> for String {
    fn from(verb: VERB) -> Self {
        match verb.0 {
            Method::GET => "GET".into(),
            Method::POST => "POST".into(),
            Method::PUT => "PUT".into(),
            Method::DELETE => "DELETE".into(),
            Method::HEAD => "HEAD".into(),
            Method::OPTIONS => "OPTIONS".into(),
            Method::CONNECT => "CONNECT".into(),
            Method::PATCH => "PATCH".into(),
            Method::TRACE => "TRACE".into(),
            _ => "".into(),
        }
    }
}

impl From<&str> for VERB {
    fn from(str: &str) -> Self {
        match str {
            "POST" => VERB(Method::POST),
            "GET" => VERB(Method::GET),
            "PUT" => VERB(Method::PUT),
            "DELETE" => VERB(Method::DELETE),
            "HEAD" => VERB(Method::HEAD),
            "OPTIONS" => VERB(Method::OPTIONS),
            "CONNECT" => VERB(Method::CONNECT),
            "PATCH" => VERB(Method::PATCH),
            "TRACE" => VERB(Method::TRACE),
            _ => VERB(Method::GET),
        }
    }
}

impl Into<Method> for VERB {
    fn into(self) -> Method {
        self.0
    }
}

impl Default for VERB {
    fn default() -> Self {
        Self::GET
    }
}

#[cfg_attr(test, automock)]
pub(crate) trait AuthToHeaderMap {
    fn get_original_header(&self) -> HeaderMap;
    fn get_header_key(&self) -> OssResult<HeaderValue>;
    fn get_header_secret(&self) -> OssResult<HeaderValue>;
    fn get_header_verb(&self) -> OssResult<HeaderValue>;
    fn get_header_md5(&self) -> OssResult<Option<HeaderValue>>;
    fn get_header_date(&self) -> OssResult<HeaderValue>;
    fn get_header_resource(&self) -> OssResult<HeaderValue>;
}

impl AuthToHeaderMap for Auth {
    fn get_original_header(&self) -> HeaderMap {
        self.headers.clone()
    }
    fn get_header_key(&self) -> OssResult<HeaderValue> {
        let val: HeaderValue = self.access_key_id.as_ref().try_into()?;
        Ok(val)
    }
    fn get_header_secret(&self) -> OssResult<HeaderValue> {
        let val: HeaderValue = self.access_key_secret.as_ref().try_into()?;
        Ok(val)
    }
    fn get_header_verb(&self) -> OssResult<HeaderValue> {
        let val: HeaderValue = self.verb.clone().try_into()?;
        Ok(val)
    }
    fn get_header_md5(&self) -> OssResult<Option<HeaderValue>> {
        let res = match self.content_md5.clone() {
            Some(val) => {
                let val: HeaderValue = val.try_into()?;
                Some(val)
            }
            None => None,
        };
        Ok(res)
    }
    fn get_header_date(&self) -> OssResult<HeaderValue> {
        let val: HeaderValue = self.date.as_ref().try_into()?;
        Ok(val)
    }
    fn get_header_resource(&self) -> OssResult<HeaderValue> {
        let val: HeaderValue = self.canonicalized_resource.as_ref().try_into()?;
        Ok(val)
    }
}

pub trait AuthToOssHeader {
    fn to_oss_header(&self) -> OssResult<OssHeader>;
}

impl AuthToOssHeader for Auth {
    /// 转化成 OssHeader
    fn to_oss_header(&self) -> OssResult<OssHeader> {
        //return Some("x-oss-copy-source:/honglei123/file1.txt");
        let mut header: Vec<(&HeaderName, &HeaderValue)> = self
            .headers
            .iter()
            .filter(|(k, _v)| k.as_str().starts_with("x-oss-"))
            .collect();
        if header.len() == 0 {
            return Ok(OssHeader(None));
        }

        header.sort_by(|(k1, _), (k2, _)| k1.to_string().cmp(&k2.to_string()));

        let header_vec: Vec<String> = header
            .into_iter()
            .filter_map(|(k, v)| match v.to_str() {
                Ok(val) => Some(k.as_str().to_owned() + ":" + val),
                _ => None,
            })
            .collect();

        Ok(OssHeader(Some(header_vec.join("\n"))))
    }
}

/// 从 auth 中提取各个字段，用于计算签名的原始字符串
pub(crate) trait AuthSignString {
    fn key(&self) -> Cow<'_, KeyId>;
    fn secret(&self) -> Cow<'_, KeySecret>;
    fn verb(&self) -> String;
    fn content_md5(&self) -> Cow<'_, ContentMd5>;
    fn content_type(&self) -> Cow<'_, ContentType>;
    fn date(&self) -> Cow<'_, Date>;
    fn canonicalized_resource(&self) -> Cow<'_, CanonicalizedResource>;
}

impl AuthSignString for Auth {
    fn key(&self) -> Cow<'_, KeyId> {
        Cow::Borrowed(&self.access_key_id)
    }
    fn secret(&self) -> Cow<'_, KeySecret> {
        Cow::Borrowed(&self.access_key_secret)
    }
    fn verb(&self) -> String {
        self.verb.to_string()
    }
    fn content_md5(&self) -> Cow<'_, ContentMd5> {
        match self.content_md5.clone() {
            Some(md5) => Cow::Owned(md5),
            None => Cow::Owned(ContentMd5::new("")),
        }
    }
    fn content_type(&self) -> Cow<'_, ContentType> {
        match self.headers.get("Content-Type") {
            Some(ct) => Cow::Owned(ct.to_owned().try_into().unwrap()),
            None => Cow::Owned(ContentType::new("")),
        }
    }
    fn date(&self) -> Cow<'_, Date> {
        Cow::Borrowed(&self.date)
    }
    fn canonicalized_resource(&self) -> Cow<'_, CanonicalizedResource> {
        Cow::Borrowed(&self.canonicalized_resource)
    }
}

pub trait AuthGetHeader {
    fn get_headers(&self) -> OssResult<HeaderMap>;
}

impl AuthGetHeader for Auth {
    fn get_headers(&self) -> OssResult<HeaderMap> {
        let mut map = HeaderMap::from_auth(self)?;

        let oss_header = self.to_oss_header()?;
        let sign_string = SignString::from_auth(self, oss_header)?;
        let sign = sign_string.to_sign()?;
        map.append_sign(sign)?;

        Ok(map)
    }
}

pub(crate) trait AuthHeader {
    fn from_auth(auth: &impl AuthToHeaderMap) -> OssResult<Self>
    where
        Self: Sized;
    fn append_sign<S: TryInto<HeaderValue, Error = OssError>>(
        &mut self,
        sign: S,
    ) -> OssResult<Option<HeaderValue>>;
}

impl AuthHeader for HeaderMap {
    fn from_auth(auth: &impl AuthToHeaderMap) -> OssResult<Self> {
        let mut map = auth.get_original_header();

        map.insert("AccessKeyId", auth.get_header_key()?);
        map.insert("SecretAccessKey", auth.get_header_secret()?);
        map.insert("VERB", auth.get_header_verb()?);

        if let Some(a) = auth.get_header_md5()? {
            map.insert("Content-MD5", a);
        }
        map.insert("Date", auth.get_header_date()?);
        map.insert("CanonicalizedResource", auth.get_header_resource()?);

        //println!("header list: {:?}",map);
        Ok(map)
    }
    fn append_sign<S: TryInto<HeaderValue, Error = OssError>>(
        &mut self,
        sign: S,
    ) -> OssResult<Option<HeaderValue>> {
        let res = self.insert("Authorization", sign.try_into()?);
        Ok(res)
    }
}

/// # 前缀是 x-oss- 的 header 记录
///
/// 将他们按顺序组合成一个字符串，用于计算签名
pub struct OssHeader(Option<String>);

impl OssHeader {
    pub fn new(string: Option<String>) -> Self {
        Self(string)
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

#[cfg_attr(test, automock)]
pub trait HeaderToSign {
    fn to_sign_string(self) -> String;
}

impl HeaderToSign for OssHeader {
    fn to_sign_string(self) -> String {
        let mut content = String::new();
        match self.0.clone() {
            Some(str) => {
                content.push_str(&str);
                content.push_str("\n");
            }
            None => (),
        }
        content
    }
}

impl Into<String> for OssHeader {
    fn into(self) -> String {
        self.to_sign_string()
    }
}

/// 待签名的数据
pub struct SignString {
    data: String,
    key: KeyId,
    secret: KeySecret,
}

impl SignString {
    pub fn new(data: String, key: KeyId, secret: KeySecret) -> SignString {
        SignString { data, key, secret }
    }

    pub(crate) fn from_auth(
        auth: &impl AuthSignString,
        header: impl HeaderToSign,
    ) -> OssResult<SignString> {
        let method = auth.verb();

        let str: String = method
            + "\n"
            + auth.content_md5().as_ref().as_ref()
            + "\n"
            + auth.content_type().as_ref().as_ref()
            + "\n"
            + auth.date().as_ref().as_ref()
            + "\n"
            + header.to_sign_string().as_ref()
            + auth.canonicalized_resource().as_ref().as_ref();

        Ok(SignString {
            data: str,
            key: auth.key().into_owned(),
            secret: auth.secret().into_owned(),
        })
    }

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
    pub fn to_sign(self) -> OssResult<Sign> {
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
            key: self.key.clone(),
        })
    }
}

/// header 中的签名
pub struct Sign {
    data: String,
    key: KeyId,
}

impl Sign {
    pub fn new(data: String, key: KeyId) -> Sign {
        Sign { data, key }
    }

    pub fn data(&self) -> String {
        self.data.clone()
    }

    pub fn key_string(&self) -> String {
        self.key.clone().to_string()
    }
}

impl TryInto<HeaderValue> for Sign {
    type Error = OssError;

    /// 转化成 header 中需要的格式
    fn try_into(self) -> OssResult<HeaderValue> {
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
    /// use aliyun_oss_client::auth::AuthGetHeader;
    /// let headers = AuthBuilder::default()
    ///     .key("bar".into())
    ///     .get_headers();
    /// ```
    pub fn key(mut self, key: KeyId) -> Self {
        self.auth.set_key(key);
        self
    }

    /// 给 secret 赋值
    pub fn secret(mut self, secret: KeySecret) -> Self {
        self.auth.set_secret(secret);
        self
    }

    /// 给 verb 赋值
    pub fn verb(mut self, verb: &VERB) -> Self {
        self.auth.set_verb(verb.to_owned());
        self
    }

    /// 给 content_md5 赋值
    pub fn content_md5(mut self, content_md5: ContentMd5) -> Self {
        self.auth.set_content_md5(content_md5);
        self
    }

    /// # 给 date 赋值
    ///
    /// ## Example
    /// ```
    /// use chrono::Utc;
    /// let builder = aliyun_oss_client::auth::AuthBuilder::default()
    ///    .date(Utc::now().into());
    /// ```
    pub fn date(mut self, date: Date) -> Self {
        self.auth.set_date(date);
        self
    }

    /// 给 content_md5 赋值
    pub fn canonicalized_resource(mut self, data: CanonicalizedResource) -> Self {
        self.auth.set_canonicalized_resource(data);
        self
    }

    pub fn with_headers(mut self, headers: Option<HeaderMap>) -> Self {
        if let Some(headers) = headers {
            self = self.extend_headers(headers);
        }
        self
    }

    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.auth.set_headers(headers);
        self
    }

    pub fn extend_headers(mut self, headers: HeaderMap) -> Self {
        self.auth.extend_headers(headers);
        self
    }

    /// 给 header 序列添加新值
    pub fn header_insert<K: IntoHeaderName + 'static>(mut self, key: K, val: HeaderValue) -> Self {
        self.auth.header_insert(key, val);
        self
    }

    /// 清理 headers
    pub fn header_clear(mut self) -> Self {
        self.auth.headers_clear();
        self
    }

    #[allow(dead_code)]
    pub(crate) fn build(self) -> Auth {
        self.auth
    }
}

impl AuthGetHeader for AuthBuilder {
    fn get_headers(&self) -> OssResult<HeaderMap> {
        self.auth.get_headers()
    }
}

#[cfg(test)]
mod builder_tests {
    use super::AuthBuilder;

    #[test]
    fn key() {
        let builder = AuthBuilder::default();
        assert_eq!(builder.build().get_key().as_ref(), "");

        let mut builder = AuthBuilder::default();
        builder = builder.key("bar".into());
        assert_eq!(builder.build().get_key().as_ref(), "bar");
    }
}
