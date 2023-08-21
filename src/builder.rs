//! 封装了 reqwest::RequestBuilder 模块

use async_trait::async_trait;
use http::{
    header::{InvalidHeaderValue, CONTENT_LENGTH},
    Method,
};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Body, IntoUrl,
};
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::{
    convert::Infallible,
    error::Error,
    io::{self, ErrorKind},
};
use std::{fmt::Display, sync::Arc, time::Duration};

#[cfg(feature = "blocking")]
use crate::blocking::builder::ClientWithMiddleware as BlockingClientWithMiddleware;
use crate::{auth::AuthError, types::ContentRange};
use crate::{
    client::Client as AliClient,
    config::{BucketBase, InvalidConfig},
    errors::OssService,
};
use reqwest::{Client, Request, Response};

#[cfg(test)]
pub(crate) mod test;

pub trait PointerFamily: private::Sealed
where
    Self::Bucket: std::fmt::Debug + Clone + Default + std::hash::Hash,
    Self::PointerType: Default,
{
    type PointerType;
    type Bucket;
}

mod private {
    pub trait Sealed {}
}

#[derive(Default, Debug)]
pub struct ArcPointer;

impl private::Sealed for ArcPointer {}

impl PointerFamily for ArcPointer {
    type PointerType = Arc<AliClient<ClientWithMiddleware>>;
    type Bucket = Arc<BucketBase>;
}

#[cfg(feature = "blocking")]
#[derive(Default, Debug)]
pub struct RcPointer;

#[cfg(feature = "blocking")]
impl private::Sealed for RcPointer {}

#[cfg(feature = "blocking")]
impl PointerFamily for RcPointer {
    type PointerType = Rc<AliClient<BlockingClientWithMiddleware>>;
    type Bucket = Rc<BucketBase>;
}

#[derive(Default, Clone, Debug)]
pub struct ClientWithMiddleware {
    inner: Client,
    middleware: Option<Arc<dyn Middleware>>,
}

#[async_trait]
pub trait Middleware: 'static + Send + Sync + std::fmt::Debug {
    async fn handle(&self, request: Request) -> Result<Response, BuilderError>;
}

impl ClientWithMiddleware {
    pub fn new(inner: Client) -> Self {
        Self {
            inner,
            middleware: None,
        }
    }

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        RequestBuilder {
            inner: self.inner.request(method, url),
            middleware: self.middleware.clone(),
        }
    }

    pub fn middleware(&mut self, middleware: Arc<dyn Middleware>) {
        self.middleware = Some(middleware);
    }
}

pub struct RequestBuilder {
    inner: reqwest::RequestBuilder,
    middleware: Option<Arc<dyn Middleware>>,
}

impl RequestBuilder {
    #[allow(dead_code)]
    pub(crate) fn header<K, V>(self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        RequestBuilder {
            inner: self.inner.header(key, value),
            ..self
        }
    }

    pub(crate) fn headers(self, headers: HeaderMap) -> Self {
        RequestBuilder {
            inner: self.inner.headers(headers),
            ..self
        }
    }

    pub(crate) fn body<T: Into<Body>>(self, body: T) -> Self {
        RequestBuilder {
            inner: self.inner.body(body),
            ..self
        }
    }

    pub(crate) fn timeout(self, timeout: Duration) -> Self {
        RequestBuilder {
            inner: self.inner.timeout(timeout),
            ..self
        }
    }

    #[allow(dead_code)]
    pub(crate) fn build(self) -> reqwest::Result<Request> {
        self.inner.build()
    }

    /// 发送请求，获取响应后，直接返回 Response
    pub async fn send(self) -> Result<Response, BuilderError> {
        match self.middleware {
            Some(m) => {
                m.handle(self.inner.build().map_err(BuilderError::from)?)
                    .await
            }
            None => self.inner.send().await.map_err(BuilderError::from),
        }
    }

    /// 发送请求，获取响应后，解析 xml 文件，如果有错误，返回 Err 否则返回 Response
    pub async fn send_adjust_error(self) -> Result<Response, BuilderError> {
        match self.middleware {
            Some(m) => {
                m.handle(self.inner.build().map_err(BuilderError::from)?)
                    .await
            }
            None => check_http_status(self.inner.send().await.map_err(BuilderError::from)?)
                .await
                .map_err(BuilderError::from),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct BuilderError {
    pub(crate) kind: BuilderErrorKind,
}

impl BuilderError {
    #[cfg(test)]
    pub(crate) fn bar() -> Self {
        Self {
            kind: BuilderErrorKind::Bar,
        }
    }

    pub(crate) fn from_reqwest(reqwest: reqwest::Error) -> Self {
        Self {
            kind: BuilderErrorKind::Reqwest(Box::new(reqwest)),
        }
    }

    pub(crate) fn header() -> Self {
        Self {
            kind: BuilderErrorKind::InvalidHeader,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum BuilderErrorKind {
    Reqwest(Box<reqwest::Error>),

    OssService(Box<OssService>),

    Auth(Box<AuthError>),

    Config(Box<InvalidConfig>),

    InvalidHeader,

    #[cfg(test)]
    Bar,
}

impl Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BuilderErrorKind::*;
        match &self.kind {
            Reqwest(_) => "reqwest error".fmt(f),
            OssService(_) => "http status is not success".fmt(f),
            Auth(_) => "aliyun auth failed".fmt(f),
            Config(_) => "oss config error".fmt(f),
            InvalidHeader => "invalid header".fmt(f),
            #[cfg(test)]
            Bar => "bar".fmt(f),
        }
    }
}

impl Error for BuilderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use BuilderErrorKind::*;
        match &self.kind {
            Reqwest(e) => Some(e),
            OssService(e) => Some(e),
            Auth(e) => Some(e),
            Config(e) => Some(e),
            InvalidHeader => None,
            #[cfg(test)]
            Bar => None,
        }
    }
}

impl From<reqwest::Error> for BuilderError {
    fn from(value: reqwest::Error) -> Self {
        Self {
            kind: BuilderErrorKind::Reqwest(Box::new(value)),
        }
    }
}
impl From<OssService> for BuilderError {
    fn from(value: OssService) -> Self {
        Self {
            kind: BuilderErrorKind::OssService(Box::new(value)),
        }
    }
}

impl From<AuthError> for BuilderError {
    fn from(value: AuthError) -> Self {
        Self {
            kind: BuilderErrorKind::Auth(Box::new(value)),
        }
    }
}
impl From<InvalidConfig> for BuilderError {
    fn from(value: InvalidConfig) -> Self {
        Self {
            kind: BuilderErrorKind::Config(Box::new(value)),
        }
    }
}

impl From<BuilderError> for io::Error {
    fn from(BuilderError { kind }: BuilderError) -> Self {
        match kind {
            BuilderErrorKind::Reqwest(req) => reqwest_to_io(*req),
            BuilderErrorKind::OssService(e) => Self::from(*e),
            BuilderErrorKind::Auth(auth) => Self::new(ErrorKind::PermissionDenied, auth),
            BuilderErrorKind::Config(conf) => Self::new(ErrorKind::InvalidInput, conf),
            BuilderErrorKind::InvalidHeader => Self::new(ErrorKind::InvalidInput, "invalid header"),
            #[cfg(test)]
            BuilderErrorKind::Bar => unreachable!("only used in tests"),
        }
    }
}

pub(crate) fn reqwest_to_io(req: reqwest::Error) -> io::Error {
    let kind = if req.is_timeout() {
        ErrorKind::TimedOut
    } else if req.is_connect() {
        ErrorKind::ConnectionAborted
    } else {
        ErrorKind::Other
    };
    io::Error::new(kind, req)
}

pub(crate) async fn check_http_status(response: Response) -> Result<Response, BuilderError> {
    if response.status().is_success() {
        return Ok(response);
    }
    let url = response.url().clone();
    let status = response.status();
    let text = response.text().await?;
    Err(OssService::new2(text, &status, url).into())
}

pub trait TryIntoHeaders {
    type Error;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error>;
}

impl TryIntoHeaders for HeaderMap {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        Ok(self)
    }
}

impl TryIntoHeaders for () {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        Ok(HeaderMap::new())
    }
}
impl TryIntoHeaders for (HeaderName, HeaderValue) {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::with_capacity(1);
        map.insert(self.0, self.1);
        Ok(map)
    }
}
impl<const N: usize> TryIntoHeaders for [(HeaderName, HeaderValue); N] {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        Ok(HeaderMap::from_iter(self.into_iter()))
    }
}
impl TryIntoHeaders for Vec<(HeaderName, HeaderValue)> {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        Ok(HeaderMap::from_iter(self.into_iter()))
    }
}

impl TryIntoHeaders for (HeaderKey, HeaderVal) {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::with_capacity(1);
        map.insert(HeaderName::from(self.0), self.1.into());
        Ok(map)
    }
}
impl<const N: usize> TryIntoHeaders for [(HeaderKey, HeaderVal); N] {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::new();
        for (k, v) in self.into_iter() {
            map.insert(HeaderName::from(k), v.into());
        }
        Ok(map)
    }
}
impl TryIntoHeaders for Vec<(HeaderKey, HeaderVal)> {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::new();
        for (k, v) in self.into_iter() {
            map.insert(HeaderName::from(k), v.into());
        }
        Ok(map)
    }
}

impl TryIntoHeaders for (HeaderName, HeaderVal) {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::with_capacity(1);
        map.insert(self.0, self.1.into());
        Ok(map)
    }
}
impl<const N: usize> TryIntoHeaders for [(HeaderName, HeaderVal); N] {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::new();
        for (k, v) in self.into_iter() {
            map.insert(k, v.into());
        }
        Ok(map)
    }
}
impl TryIntoHeaders for Vec<(HeaderName, HeaderVal)> {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::new();
        for (k, v) in self.into_iter() {
            map.insert(k, v.into());
        }
        Ok(map)
    }
}

pub(crate) enum HeaderKey {
    // /// If-Unmodified-Since
    // IfUnmodifiedSince,
    /// range
    Range,
    ContentLength,
    ContentType,
}

impl From<HeaderKey> for HeaderName {
    fn from(value: HeaderKey) -> Self {
        use http::header::CONTENT_TYPE;
        const RANGE: &str = "Range";
        match value {
            //HeaderKey::IfUnmodifiedSince => HeaderName::from_static("If-Unmodified-Since"),
            HeaderKey::Range => HeaderName::from_static(RANGE),
            HeaderKey::ContentLength => CONTENT_LENGTH,
            HeaderKey::ContentType => CONTENT_TYPE,
        }
    }
}

pub(crate) enum HeaderVal {
    Range(HeaderValue),
    ContentLength(usize),
    ContentType(HeaderValue),
}

impl HeaderVal {
    pub fn content_type(str: &str) -> Result<(HeaderKey, Self), InvalidHeaderValue> {
        Ok((HeaderKey::ContentType, Self::ContentType(str.parse()?)))
    }

    pub fn len(len: usize) -> (HeaderKey, Self) {
        (HeaderKey::ContentLength, Self::ContentLength(len))
    }

    pub fn range<Num, R>(range: R) -> (HeaderKey, Self)
    where
        R: Into<ContentRange<Num>>,
        ContentRange<Num>: Into<HeaderValue>,
    {
        (HeaderKey::Range, Self::Range(range.into().into()))
    }
}

impl From<HeaderVal> for HeaderValue {
    fn from(value: HeaderVal) -> Self {
        match value {
            HeaderVal::Range(r) => r,
            HeaderVal::ContentLength(n) => n.into(),
            HeaderVal::ContentType(con) => con,
        }
    }
}

#[test]
fn test_into_header() {
    use http::header::CONTENT_TYPE;
    fn get<M: TryIntoHeaders>(_m: M) {}

    get(());
    get((CONTENT_TYPE, HeaderValue::from_static("application/json")));
    get([(CONTENT_TYPE, HeaderValue::from_static("application/json"))]);
    get(vec![(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    )]);
    get([
        (CONTENT_TYPE, HeaderValue::from_static("application/json")),
        (CONTENT_LENGTH, HeaderValue::from_static("12")),
    ]);
    get([(
        HeaderKey::Range,
        HeaderVal::ContentType("application/json".parse().unwrap()),
    )]);
    get((HeaderKey::Range, HeaderVal::ContentLength(10)));
    get([(HeaderKey::Range, HeaderVal::ContentLength(10))]);
    get(vec![(HeaderKey::Range, HeaderVal::ContentLength(10))]);

    get((CONTENT_TYPE, HeaderVal::ContentLength(10)));
    get([(CONTENT_TYPE, HeaderVal::ContentLength(10))]);
    get(vec![(CONTENT_TYPE, HeaderVal::ContentLength(10))]);
}
