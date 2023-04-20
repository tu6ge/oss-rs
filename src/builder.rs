//! 封装了 reqwest::RequestBuilder 模块

use async_trait::async_trait;
use http::Method;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Body, IntoUrl,
};
use std::error::Error;
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::{fmt::Display, sync::Arc, time::Duration};

#[cfg(feature = "auth")]
use crate::auth::AuthError;
#[cfg(feature = "blocking")]
use crate::blocking::builder::ClientWithMiddleware as BlockingClientWithMiddleware;
use crate::{
    client::Client as AliClient,
    config::{BucketBase, InvalidConfig},
    errors::OssService,
};
use reqwest::{Client, Request, Response};

#[cfg(test)]
pub(crate) mod test;

pub trait PointerFamily
where
    Self::Bucket: std::fmt::Debug + Clone + Default,
{
    type PointerType;
    type Bucket;
}

#[derive(Default, Debug)]
pub struct ArcPointer;

impl PointerFamily for ArcPointer {
    type PointerType = Arc<AliClient<ClientWithMiddleware>>;
    type Bucket = Arc<BucketBase>;
}

#[cfg(feature = "blocking")]
#[derive(Default, Debug)]
pub struct RcPointer;

#[cfg(feature = "blocking")]
impl PointerFamily for RcPointer {
    type PointerType = Rc<AliClient<BlockingClientWithMiddleware>>;
    type Bucket = Rc<BucketBase>;
}

#[derive(Default, Clone)]
pub struct ClientWithMiddleware {
    inner: Client,
    middleware: Option<Arc<dyn Middleware>>,
}

#[async_trait]
pub trait Middleware: 'static + Send + Sync {
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
            Some(m) =>
            {
                #[allow(clippy::unwrap_used)]
                m.handle(self.inner.build().unwrap()).await
            }
            None => self.inner.send().await.map_err(BuilderError::from),
        }
    }

    /// 发送请求，获取响应后，解析 xml 文件，如果有错误，返回 Err 否则返回 Response
    pub async fn send_adjust_error(self) -> Result<Response, BuilderError> {
        match self.middleware {
            Some(m) =>
            {
                #[allow(clippy::unwrap_used)]
                m.handle(self.inner.build().unwrap()).await
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
}

#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum BuilderErrorKind {
    Reqwest(reqwest::Error),

    OssService(OssService),

    #[cfg(feature = "auth")]
    Auth(AuthError),

    Config(InvalidConfig),

    #[cfg(test)]
    Bar,
}

impl Display for BuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BuilderErrorKind::*;
        match &self.kind {
            Reqwest(_) => "reqwest error".fmt(f),
            OssService(_) => "http status is not success".fmt(f),
            #[cfg(feature = "auth")]
            Auth(_) => "aliyun auth failed".fmt(f),
            Config(_) => "oss config error".fmt(f),
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
            #[cfg(feature = "auth")]
            Auth(e) => Some(e),
            Config(e) => Some(e),
            #[cfg(test)]
            Bar => None,
        }
    }
}

impl From<reqwest::Error> for BuilderError {
    fn from(value: reqwest::Error) -> Self {
        Self {
            kind: BuilderErrorKind::Reqwest(value),
        }
    }
}
impl From<OssService> for BuilderError {
    fn from(value: OssService) -> Self {
        Self {
            kind: BuilderErrorKind::OssService(value),
        }
    }
}

#[cfg(feature = "auth")]
impl From<AuthError> for BuilderError {
    fn from(value: AuthError) -> Self {
        Self {
            kind: BuilderErrorKind::Auth(value),
        }
    }
}
impl From<InvalidConfig> for BuilderError {
    fn from(value: InvalidConfig) -> Self {
        Self {
            kind: BuilderErrorKind::Config(value),
        }
    }
}

pub(crate) async fn check_http_status(response: Response) -> Result<Response, BuilderError> {
    if response.status().is_success() {
        return Ok(response);
    }
    let status = response.status();
    let text = response.text().await?;
    Err(OssService::new2(text, &status).into())
}
