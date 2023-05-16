use crate::{builder::BuilderError, errors::OssService};
use http::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
use reqwest::{
    blocking::{self, Body, Request, Response},
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

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        RequestBuilder {
            inner: self.inner.request(method, url),
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
    pub fn send(self) -> Result<Response, BuilderError> {
        match self.middleware {
            Some(m) => m.handle(self.inner.build().unwrap()),
            None => self.inner.send().map_err(BuilderError::from),
        }
    }

    /// 发送请求，获取响应后，解析 xml 文件，如果有错误，返回 Err 否则返回 Response
    pub fn send_adjust_error(self) -> Result<Response, BuilderError> {
        match self.middleware {
            Some(m) => m.handle(self.inner.build().unwrap()),
            None => check_http_status(self.inner.send().map_err(BuilderError::from)?),
        }
    }
}

pub(crate) fn check_http_status(response: Response) -> Result<Response, BuilderError> {
    if response.status().is_success() {
        return Ok(response);
    }
    let url = response.url().clone();
    let status = response.status();
    Err(OssService::new2(response.text()?, &status, url).into())
}
