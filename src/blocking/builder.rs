use crate::auth::VERB;
use crate::errors::{OssError, OssResult};
use reqwest::blocking::{self, Body, Request, Response};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    IntoUrl,
};
use std::{rc::Rc, time::Duration};

#[derive(Default)]
pub struct ClientWithMiddleware {
    inner: blocking::Client,
    middleware: Option<Rc<dyn Middleware>>,
}

pub trait Middleware: 'static {
    fn handle(&self, request: Request) -> OssResult<Response>;
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

    pub fn send(self) -> OssResult<Response> {
        match self.middleware {
            Some(m) => m.handle(self.inner.build().unwrap()),
            None => {
                // TODO map_err 照这个改
                self.inner.send().map_err(OssError::from)?.handle_error()
            }
        }
    }
}

pub trait BlockingReqeustHandler {
    fn handle_error(self) -> OssResult<Self>
    where
        Self: Sized;
}

impl BlockingReqeustHandler for Response {
    /// # 收集并处理 OSS 接口返回的错误
    fn handle_error(self) -> OssResult<Response> {
        #[cfg_attr(test, mockall_double::double)]
        use crate::errors::OssService;

        let status = self.status();

        if status != 200 && status != 204 {
            return Err(OssService::new(self.text()?).into());
        }

        Ok(self)
    }
}
