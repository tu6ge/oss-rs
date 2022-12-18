use async_trait::async_trait;
use http::Method;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Body, IntoUrl,
};
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::{sync::Arc, time::Duration};
use thiserror::Error;

#[cfg(feature = "blocking")]
use crate::blocking::builder::ClientWithMiddleware as BlockingClientWithMiddleware;
use crate::{auth::AuthError, client::Client as AliClient, config::BucketBase};
use reqwest::{Client, Request, Response};

pub trait PointerFamily
where
    Self::Bucket: std::fmt::Debug + Clone + Default,
{
    type PointerType;
    type Bucket;
}

#[derive(Default)]
pub struct ArcPointer;

impl PointerFamily for ArcPointer {
    type PointerType = Arc<AliClient<ClientWithMiddleware>>;
    type Bucket = Arc<BucketBase>;
}

#[cfg(feature = "blocking")]
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
            Some(m) => m.handle(self.inner.build().unwrap()).await,
            None => self.inner.send().await.map_err(BuilderError::from),
        }
    }

    /// 发送请求，获取响应后，解析 xml 文件，如果有错误，返回 Err 否则返回 Response
    pub async fn send_adjust_error(self) -> Result<Response, BuilderError> {
        match self.middleware {
            Some(m) => m.handle(self.inner.build().unwrap()).await,
            None => self
                .inner
                .send()
                .await
                .map_err(BuilderError::from)?
                .handle_error()
                .await
                .map_err(BuilderError::from),
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BuilderError {
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("OssService {0}")]
    OssService(#[from] OssService),

    #[error("{0}")]
    AuthError(#[from] AuthError),
}

#[async_trait]
pub(crate) trait RequestHandler {
    async fn handle_error(self) -> Result<Response, BuilderError>;
}

use crate::errors::OssService;

#[async_trait]
impl RequestHandler for Response {
    /// # 收集并处理 OSS 接口返回的错误
    async fn handle_error(self) -> Result<Response, BuilderError> {
        if self.status().is_success() {
            return Ok(self);
        }

        let status = self.status();

        Err(OssService::new(&self.text().await?, &status).into())
    }
}
