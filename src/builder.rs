use async_trait::async_trait;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Body, IntoUrl,
};
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::{sync::Arc, time::Duration};

#[cfg(feature = "blocking")]
use crate::blocking::builder::ClientWithMiddleware as BlockingClientWithMiddleware;
use crate::{
    auth::VERB,
    client::Client as AliClient,
    config::BucketBase,
    errors::{OssError, OssResult},
};
use reqwest::{Client, Request, Response};

pub trait PointerFamily
where
    Self::Bucket: std::fmt::Debug + Clone + Default,
{
    type PointerType;
    type Bucket;
}

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

#[derive(Default)]
pub struct ClientWithMiddleware {
    inner: Client,
    middleware: Option<Arc<dyn Middleware>>,
}

#[async_trait]
pub trait Middleware: 'static + Send + Sync {
    async fn handle(&self, request: Request) -> OssResult<Response>;
}

impl ClientWithMiddleware {
    pub fn new(inner: Client) -> Self {
        Self {
            inner,
            middleware: None,
        }
    }

    pub fn request<U: IntoUrl>(&self, method: VERB, url: U) -> RequestBuilder {
        RequestBuilder {
            inner: self.inner.request(method.into(), url),
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
    pub fn header<K, V>(self, key: K, value: V) -> Self
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

    pub fn headers(self, headers: HeaderMap) -> Self {
        RequestBuilder {
            inner: self.inner.headers(headers),
            ..self
        }
    }

    pub fn body<T: Into<Body>>(self, body: T) -> Self {
        RequestBuilder {
            inner: self.inner.body(body),
            ..self
        }
    }

    pub fn timeout(self, timeout: Duration) -> Self {
        RequestBuilder {
            inner: self.inner.timeout(timeout),
            ..self
        }
    }

    pub fn build(self) -> reqwest::Result<Request> {
        self.inner.build()
    }

    pub async fn send(self) -> OssResult<Response> {
        match self.middleware {
            Some(m) => m.handle(self.inner.build().unwrap()).await,
            None => {
                self.inner
                    .send()
                    .await
                    .map_err(OssError::from)?
                    .handle_error()
                    .await
            }
        }
    }
}

#[async_trait]
pub trait RequestHandler {
    async fn handle_error(self) -> OssResult<Response>;
}

#[async_trait]
impl RequestHandler for Response {
    /// # 收集并处理 OSS 接口返回的错误
    async fn handle_error(self) -> OssResult<Response> {
        use crate::errors::OssService;

        let status = self.status();

        if status != 200 && status != 204 {
            return Err(OssService::new(self.text().await?).into());
        }

        Ok(self)
    }
}
