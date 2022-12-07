use crate::auth::{AuthBuilder, AuthGetHeader, VERB};
#[cfg(feature = "blocking")]
use crate::blocking::builder::ClientWithMiddleware as BlockingClientWithMiddleware;
#[cfg(test)]
use crate::builder::Middleware;
use crate::builder::{BuilderError, ClientWithMiddleware, RequestBuilder};
use crate::config::{BucketBase, Config, InvalidConfig};
use crate::file::AlignBuilder;
use crate::types::{BucketName, CanonicalizedResource, EndPoint, KeyId, KeySecret};
use chrono::prelude::*;
use reqwest::header::HeaderMap;
use reqwest::Url;
use std::env;
#[cfg(all(feature = "blocking", test))]
use std::rc::Rc;
#[cfg(test)]
use std::sync::Arc;

/// # 构造请求的客户端结构体
/// Clone 特征不是必须的
#[non_exhaustive]
#[derive(Default, Clone)]
pub struct Client<M = ClientWithMiddleware>
where
    M: Default + Clone,
{
    auth_builder: AuthBuilder,
    client_middleware: M,
    endpoint: EndPoint,
    bucket: BucketName,
}

impl<M: Default + Clone> Client<M> {
    pub fn new(
        access_key_id: KeyId,
        access_key_secret: KeySecret,
        endpoint: EndPoint,
        bucket: BucketName,
    ) -> Self {
        let auth_builder = AuthBuilder::default()
            .key(access_key_id)
            .secret(access_key_secret);

        Self::from_builder(auth_builder, endpoint, bucket)
    }

    pub fn from_config(config: Config) -> Self {
        let (key, secret, bucket, endpoint) = config.get_all();

        let auth_builder = AuthBuilder::default().key(key).secret(secret);

        Self::from_builder(auth_builder, endpoint, bucket)
    }

    /// # 通过环境变量初始化 Client
    ///
    /// 示例
    /// ```rust
    /// use std::env::set_var;
    /// set_var("ALIYUN_KEY_ID", "foo1");
    /// set_var("ALIYUN_KEY_SECRET", "foo2");
    /// set_var("ALIYUN_ENDPOINT", "qingdao");
    /// set_var("ALIYUN_BUCKET", "foo4");
    ///
    /// # use aliyun_oss_client::client::Client;
    /// use aliyun_oss_client::builder::ClientWithMiddleware;
    /// let client = Client::<ClientWithMiddleware>::from_env();
    /// assert!(client.is_ok());
    /// ```
    pub fn from_env() -> Result<Self, InvalidConfig> {
        let key_id = env::var("ALIYUN_KEY_ID").map_err(InvalidConfig::from)?;
        let key_secret = env::var("ALIYUN_KEY_SECRET").map_err(InvalidConfig::from)?;
        let endpoint = env::var("ALIYUN_ENDPOINT").map_err(InvalidConfig::from)?;
        let bucket = env::var("ALIYUN_BUCKET").map_err(InvalidConfig::from)?;

        let auth_builder = AuthBuilder::default()
            .key(key_id.into())
            .secret(key_secret.into());

        Ok(Self::from_builder(
            auth_builder,
            endpoint.into(),
            bucket.into(),
        ))
    }

    #[inline]
    pub fn from_builder(auth_builder: AuthBuilder, endpoint: EndPoint, bucket: BucketName) -> Self {
        Self {
            auth_builder,
            client_middleware: M::default(),
            endpoint,
            bucket,
        }
    }

    #[deprecated(
        since = "0.10",
        note = "this bucket is default value, is not need change"
    )]
    pub fn set_bucket_name(&mut self, bucket: BucketName) {
        self.bucket = bucket
    }

    pub(crate) fn get_bucket_name(&self) -> &BucketName {
        &self.bucket
    }

    /// 返回默认可用区，默认 bucket 的 BucketBase
    pub fn get_bucket_base(&self) -> BucketBase {
        BucketBase::new(self.bucket.to_owned(), self.endpoint.to_owned())
    }

    pub fn get_bucket_url(&self) -> Url {
        self.get_bucket_base().to_url()
    }

    pub(crate) fn get_endpoint(&self) -> &EndPoint {
        &self.endpoint
    }
    pub fn get_endpoint_url(&self) -> Url {
        self.endpoint.to_url()
    }
}

#[cfg(not(test))]
#[inline]
fn now() -> DateTime<Utc> {
    Utc::now()
}

#[cfg(test)]
fn now() -> DateTime<Utc> {
    let naive = NaiveDateTime::parse_from_str("2022/10/6 20:40:00", "%Y/%m/%d %H:%M:%S").unwrap();
    DateTime::from_utc(naive, Utc)
}

pub type ClientArc = Client<ClientWithMiddleware>;

impl Client {
    /// # 用于模拟请求 OSS 接口
    /// 默认直接请求 OSS 接口，如果设置中间件，则可以中断请求，对 Request 做一些断言，对 Response 做一些模拟操作
    #[cfg(test)]
    pub(crate) fn middleware(mut self, middleware: Arc<dyn Middleware>) -> Self {
        self.client_middleware.middleware(middleware);
        self
    }
}

impl AlignBuilder for Client<ClientWithMiddleware> {
    /// builder 方法的异步实现
    /// 带 header 参数
    #[inline]
    fn builder_with_header<M: Into<VERB>>(
        &self,
        method: M,
        url: Url,
        resource: CanonicalizedResource,
        headers: Option<HeaderMap>,
    ) -> Result<RequestBuilder, BuilderError> {
        let method = method.into();
        let headers = self
            .auth_builder
            .clone()
            .verb(&method)
            .date(now().into())
            .canonicalized_resource(resource)
            .with_headers(headers)
            .get_headers()?;

        Ok(self.client_middleware.request(method, url).headers(headers))
    }
}

#[cfg(all(feature = "blocking", test))]
use crate::blocking::builder::Middleware as BlockingMiddleware;
#[cfg(feature = "blocking")]
use crate::blocking::builder::RequestBuilder as BlockingRequestBuilder;

#[cfg(feature = "blocking")]
pub type ClientRc = Client<BlockingClientWithMiddleware>;

#[cfg(feature = "blocking")]
impl Client<BlockingClientWithMiddleware> {
    /// # 用于模拟请求 OSS 接口
    /// 默认直接请求 OSS 接口，如果设置中间件，则可以中断请求，对 Request 做一些断言，对 Response 做一些模拟操作
    #[cfg(test)]
    pub(crate) fn middleware(mut self, middleware: Rc<dyn BlockingMiddleware>) -> Self {
        self.client_middleware.middleware(middleware);
        self
    }
}

#[cfg(feature = "blocking")]
impl crate::file::blocking::AlignBuilder for Client<BlockingClientWithMiddleware> {
    /// # 向 OSS 发送请求的封装
    /// 参数包含请求的：
    ///
    /// - method
    /// - url
    /// - headers (可选)
    /// - [CanonicalizedResource](https://help.aliyun.com/document_detail/31951.html#section-rvv-dx2-xdb)
    ///
    /// 返回值是一个 reqwest 的请求创建器 `reqwest::blocking::RequestBuilder`
    ///
    /// 返回后，可以再加请求参数，然后可选的进行发起请求
    #[inline]
    fn builder_with_header<M: Into<VERB>>(
        &self,
        method: M,
        url: Url,
        resource: CanonicalizedResource,
        headers: Option<HeaderMap>,
    ) -> Result<BlockingRequestBuilder, BuilderError> {
        let method = method.into();
        let headers = self
            .auth_builder
            .clone()
            .verb(&method)
            .date(now().into())
            .canonicalized_resource(resource)
            .with_headers(headers)
            .get_headers()?;

        Ok(self.client_middleware.request(method, url).headers(headers))
    }
}
