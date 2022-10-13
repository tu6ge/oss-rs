
use std::{sync::Arc, time::Duration};
use async_trait::async_trait;
use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}, IntoUrl, Body};

use reqwest::{Client, Response, Request};
use crate::{errors::{OssResult, OssError}, auth::VERB};

#[derive(Default)]
pub struct ClientWithMiddleware{
    inner: Client,
    middleware: Option<Arc<dyn Middleware>>,
}

#[async_trait]
pub trait Middleware: 'static + Send + Sync{
    async fn handle(&self, request: Request) -> OssResult<Response>;
}

impl ClientWithMiddleware{
    pub fn new(inner: Client) ->Self{
        Self{
            inner,
            middleware: None,
        }
    }

    pub fn request<U: IntoUrl>(&self, method: VERB, url: U) -> RequestBuilder{
        RequestBuilder{
            inner: self.inner.request(method.into(), url),
            middleware: self.middleware.clone(),
        }
        
    }

    pub fn middleware(&mut self, middleware: Arc<dyn Middleware>){
        self.middleware = Some(middleware);
    }
}

pub struct RequestBuilder{
    inner: reqwest::RequestBuilder,
    middleware: Option<Arc<dyn Middleware>>
}

impl RequestBuilder{
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
            Some(m) =>{
                m.handle(self.inner.build().unwrap()).await
            },
            None => {
                // TODO map_err 照这个改
                self.inner.send().await.map_err(OssError::from)?.handle_error().await
            }
        }
    }
}

#[async_trait]
pub trait RequestHandler {
    async fn handle_error(self) -> OssResult<Response>;
}

#[async_trait]
impl RequestHandler for Response{
    /// # 收集并处理 OSS 接口返回的错误
    async fn handle_error(self) -> OssResult<Response>
    {
        #[cfg_attr(test, mockall_double::double)]
        use crate::errors::OssService;

        let status = self.status();
        
        if status != 200 && status != 204 {
            return Err(OssService::new(self.text().await?).into());
        }

        Ok(self)
    }
}

#[cfg(feature = "blocking")]
pub mod blocking{
    use reqwest::blocking::{self,Response,Request, Body};
    use std::time::Duration;
    use crate::errors::{OssResult, OssError};
    use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}, IntoUrl};
    use std::sync::Arc;
    use crate::auth::VERB;

    #[derive(Default)]
    pub struct ClientWithMiddleware{
        inner: blocking::Client,
        middleware: Option<Arc<dyn Middleware>>,
    }

    pub trait Middleware: 'static{
        fn handle(&self, request: Request) -> OssResult<Response>;
    }

    impl ClientWithMiddleware{
        pub fn new(inner: blocking::Client) ->Self{
            Self{
                inner,
                middleware: None,
            }
        }

        pub fn request<U: IntoUrl>(&self, method: VERB, url: U) -> RequestBuilder{
            RequestBuilder{
                inner: self.inner.request(method.into(), url),
                middleware: self.middleware.clone(),
            }
            
        }

        pub fn middleware(&mut self, middleware: Arc<dyn Middleware>){
            self.middleware = Some(middleware);
        }
    }

    pub struct RequestBuilder{
        inner: reqwest::blocking::RequestBuilder,
        middleware: Option<Arc<dyn Middleware>>
    }

    impl RequestBuilder{
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
                Some(m) =>{
                    m.handle(self.inner.build().unwrap())
                },
                None => {
                    // TODO map_err 照这个改
                    self.inner.send().map_err(OssError::from)?.handle_error()
                }
            }
        }
    }

    pub trait BlockingReqeustHandler {
        fn handle_error(self) -> OssResult<Self> where Self: Sized;
    }
    
    impl BlockingReqeustHandler for Response {
    
        /// # 收集并处理 OSS 接口返回的错误
        fn handle_error(self) -> OssResult<Response>
        {
            #[cfg_attr(test, mockall_double::double)]
            use crate::errors::OssService;
    
            let status = self.status();
        
            if status != 200 && status != 204{
                return Err(OssService::new(self.text()?).into());
            }
    
            Ok(self)
        }
    }
}



