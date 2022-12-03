use crate::auth::VERB;
use crate::builder::BuilderError;
use crate::errors::{OssError, OssResult};
use reqwest::blocking::{self, Body, Request, Response};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    IntoUrl,
};
use std::{rc::Rc, time::Duration};

#[derive(Default, Clone)]
pub struct ClientWithMiddleware {
    inner: blocking::Client,
    middleware: Option<Rc<dyn Middleware>>,
}

pub trait Middleware: 'static {
    fn handle(&self, request: Request) -> Result<Response, BuilderError>;
}

impl ClientWithMiddleware {
    pub fn new(inner: blocking::Client) -> Self {
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

    pub fn middleware(&mut self, middleware: Rc<dyn Middleware>) {
        self.middleware = Some(middleware);
    }
}

pub struct RequestBuilder {
    inner: reqwest::blocking::RequestBuilder,
    middleware: Option<Rc<dyn Middleware>>,
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

    pub fn send(self) -> Result<Response, BuilderError> {
        match self.middleware {
            Some(m) => m.handle(self.inner.build().unwrap()),
            None => self
                .inner
                .send()
                .map_err(BuilderError::from)?
                .handle_error(),
        }
    }
}

pub trait BlockingReqeustHandler {
    fn handle_error(self) -> Result<Self, BuilderError>
    where
        Self: Sized;
}

impl BlockingReqeustHandler for Response {
    /// # 收集并处理 OSS 接口返回的错误
    fn handle_error(self) -> Result<Response, BuilderError> {
        use crate::builder::SUCCESS_STATUS;
        use crate::errors::OssService;

        let status = self.status();

        for item in SUCCESS_STATUS.iter() {
            if *item == status {
                return Ok(self);
            }
        }

        Err(OssService::new(&self.text()?).into())
    }
}
