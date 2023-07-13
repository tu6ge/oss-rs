//! # 对 reqwest 进行了简单的封装，加上了 OSS 的签名验证功能

use crate::auth::AuthBuilder;
#[cfg(feature = "blocking")]
use crate::blocking::builder::ClientWithMiddleware as BlockingClientWithMiddleware;
#[cfg(test)]
use crate::builder::Middleware;
use crate::builder::{ArcPointer, BuilderError, ClientWithMiddleware, RequestBuilder};
use crate::config::{get_bucket, get_endpoint, get_env, BucketBase, Config, InvalidConfig};
use crate::consts::{TRUE1, TRUE2, TRUE3, TRUE4};
use crate::file::AlignBuilder;
use crate::types::{
    object::{InvalidObjectPath, ObjectBase, ObjectPath},
    BucketName, CanonicalizedResource, EndPoint, KeyId, KeySecret,
};

use chrono::{DateTime, Utc};
use http::{
    header::{HeaderMap, HeaderName},
    HeaderValue, Method,
};
use reqwest::Url;
use std::env;
#[cfg(all(feature = "blocking", test))]
use std::rc::Rc;
#[cfg(test)]
use std::sync::Arc;
use std::time::Duration;

/// # 构造请求的客户端结构体
#[non_exhaustive]
pub struct Client<M = ClientWithMiddleware> {
    auth_builder: AuthBuilder,
    client_middleware: M,
    pub(crate) endpoint: EndPoint,
    pub(crate) bucket: BucketName,
    timeout: Option<Duration>,
}

impl<M: Default> Default for Client<M> {
    fn default() -> Self {
        Self {
            auth_builder: AuthBuilder::default(),
            client_middleware: M::default(),
            endpoint: EndPoint::default(),
            bucket: BucketName::default(),
            timeout: Option::default(),
        }
    }
}

impl<M: Clone> Clone for Client<M> {
    fn clone(&self) -> Self {
        Self {
            auth_builder: self.auth_builder.clone(),
            client_middleware: self.client_middleware.clone(),
            endpoint: self.endpoint.clone(),
            bucket: self.bucket.clone(),
            timeout: self.timeout.clone(),
        }
    }
}

impl<M> AsMut<Option<Duration>> for Client<M> {
    fn as_mut(&mut self) -> &mut Option<Duration> {
        &mut self.timeout
    }
}

impl<M> AsRef<EndPoint> for Client<M> {
    fn as_ref(&self) -> &EndPoint {
        &self.endpoint
    }
}
impl<M> AsRef<BucketName> for Client<M> {
    fn as_ref(&self) -> &BucketName {
        &self.bucket
    }
}

impl<M: Default> Client<M> {
    /// 使用基本配置信息初始化 Client
    pub fn new(
        access_key_id: KeyId,
        access_key_secret: KeySecret,
        endpoint: EndPoint,
        bucket: BucketName,
    ) -> Self {
        let mut auth_builder = AuthBuilder::default();
        auth_builder.key(access_key_id);
        auth_builder.secret(access_key_secret);

        Self::from_builder(auth_builder, endpoint, bucket)
    }

    /// - bucket: bar
    /// - endpoint: qingdao
    #[cfg(test)]
    pub fn test_init() -> Self {
        Self::new(
            "foo1".into(),
            "foo2".into(),
            EndPoint::CN_QINGDAO,
            "bar".try_into().unwrap(),
        )
    }

    /// 使用 [`Config`] 中的配置初始化 Client
    ///
    /// [`Config`]: crate::config::Config
    pub fn from_config(config: Config) -> Self {
        let (key, secret, bucket, endpoint) = config.get_all();

        let mut auth_builder = AuthBuilder::default();
        auth_builder.key(key);
        auth_builder.secret(secret);

        Self::from_builder(auth_builder, endpoint, bucket)
    }

    /// # 通过环境变量初始化 Client
    ///
    /// 如果在 Aliyun ECS 上，可将环境变量 `ALIYUN_OSS_INTERNAL`
    /// 设置为 `true` / `1` / `yes` / `Y` ，即可使用 internal 网络请求 OSS 接口
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
        let key_id = get_env("ALIYUN_KEY_ID")?;
        let key_secret = get_env("ALIYUN_KEY_SECRET")?;
        let endpoint = get_env("ALIYUN_ENDPOINT")?;
        let bucket = get_env("ALIYUN_BUCKET")?;

        let mut auth_builder = AuthBuilder::default();
        auth_builder.key(key_id);
        auth_builder.secret(key_secret);

        let mut endpoint = get_endpoint(&endpoint)?;

        if let Ok(is_internal) = env::var("ALIYUN_OSS_INTERNAL") {
            if is_internal == TRUE1
                || is_internal == TRUE2
                || is_internal == TRUE3
                || is_internal == TRUE4
            {
                endpoint.set_internal(true);
            }
        }

        Ok(Self::from_builder(
            auth_builder,
            endpoint,
            get_bucket(&bucket)?,
        ))
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_builder(auth_builder: AuthBuilder, endpoint: EndPoint, bucket: BucketName) -> Self {
        Self {
            auth_builder,
            client_middleware: M::default(),
            endpoint,
            bucket,
            timeout: None,
        }
    }
}

impl<M> Client<M> {
    pub(crate) fn get_bucket_name(&self) -> &BucketName {
        &self.bucket
    }

    /// 返回默认可用区，默认 bucket 的 BucketBase
    pub fn get_bucket_base(&self) -> BucketBase {
        BucketBase::new(self.bucket.to_owned(), self.endpoint.to_owned())
    }

    /// 获取默认的 bucket 的 url
    pub fn get_bucket_url(&self) -> Url {
        self.get_bucket_base().to_url()
    }

    pub(crate) fn get_key(&self) -> &KeyId {
        &self.auth_builder.get_key()
    }
    pub(crate) fn get_secret(&self) -> &KeySecret {
        &self.auth_builder.get_secret()
    }

    pub(crate) fn get_endpoint(&self) -> &EndPoint {
        &self.endpoint
    }

    /// 获取默认的可用区的 url
    pub fn get_endpoint_url(&self) -> Url {
        self.endpoint.to_url()
    }

    /// 设置 timeout
    pub fn timeout(&mut self, timeout: Duration) {
        self.timeout = Some(timeout);
    }

    /// 根据默认的 bucket，endpoint 和提供的文件路径，获取 ObjectBase
    #[inline]
    pub fn get_object_base<P>(&self, path: P) -> Result<ObjectBase, InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        P::Error: Into<InvalidObjectPath>,
    {
        ObjectBase::<ArcPointer>::from_bucket(self.get_bucket_base(), path)
    }
}

#[cfg(not(test))]
#[inline]
fn now() -> DateTime<Utc> {
    Utc::now()
}

#[cfg(test)]
fn now() -> DateTime<Utc> {
    use chrono::NaiveDateTime;
    let naive = NaiveDateTime::parse_from_str("2022/10/6 20:40:00", "%Y/%m/%d %H:%M:%S").unwrap();
    DateTime::from_utc(naive, Utc)
}

/// 异步 Client 别名
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
    /// # 构造自定义的接口请求方法
    /// 比如在上传完文件时，返回自己期望的数据，而不是仅返回 etag 信息
    ///
    /// ## 例子是一个获取 object 元信息的接口
    /// ```
    /// use aliyun_oss_client::{errors::OssError, file::AlignBuilder, Client, Method};
    /// use dotenv::dotenv;
    ///
    /// async fn run() -> Result<(), OssError> {
    ///     dotenv().ok();
    ///     let client = Client::from_env().unwrap();
    ///
    ///     let (url, resource) = client
    ///         .get_object_base("9AB932LY.jpeg")?
    ///         .get_url_resource([]);
    ///
    ///     let headers = vec![(
    ///         "If-Unmodified-Since".parse().unwrap(),
    ///         "Sat, 01 Jan 2022 18:01:01 GMT".parse().unwrap(),
    ///     )];
    ///
    ///     let builder = client.builder_with_header(Method::HEAD, url, resource, headers)?;
    ///
    ///     let response = builder.send().await?;
    ///
    ///     println!("status: {:?}", response.status());
    ///
    ///     Ok(())
    /// }
    /// ```
    /// ## 参数
    /// - method 接口请求方式
    /// - url 要请求的接口，包含 query 参数等信息
    /// - resource 阿里云接口需要提供的统一的信息，[`CanonicalizedResource`] 提供了 bucket ，object 等各种生成方式，如果无法满足
    /// 还可以自己用 trait 来自定义
    ///
    /// ## 返回值
    /// 返回值是一个封装了 reqwest::Builder 构造器，[`RequestBuilder`], 提供两个方法 `send` 和 `send_adjust_error`
    ///
    /// - `send` 方法，直接返回 `reqwest::Response`
    /// - `send_adjust_error` 方法，会对 api 返回结果进行处理，如果 HTTP 状态码正常（200>= && <300) 则，返回 Ok,
    /// 否则，会对返回的 xml 异常数据进行解析，返回 Err([`OssService`])
    ///
    /// [`RequestBuilder`]: crate::builder::RequestBuilder
    /// [`OssService`]: crate::errors::OssService
    /// [`CanonicalizedResource`]: crate::types::CanonicalizedResource
    fn builder_with_header<H: IntoIterator<Item = (HeaderName, HeaderValue)>>(
        &self,
        method: Method,
        url: Url,
        resource: CanonicalizedResource,
        headers: H,
    ) -> Result<RequestBuilder, BuilderError> {
        // dbg!(url.clone());
        // dbg!(resource.clone());
        let mut auth_builder = self.auth_builder.clone();
        auth_builder.method(&method);
        auth_builder.date(now());
        auth_builder.canonicalized_resource(resource);
        auth_builder.extend_headers(HeaderMap::from_iter(headers));

        let mut builder = self
            .client_middleware
            .request(method, url)
            .headers(auth_builder.get_headers()?);

        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        };

        Ok(builder)
    }
}

#[cfg(all(feature = "blocking", test))]
use crate::blocking::builder::Middleware as BlockingMiddleware;
#[cfg(feature = "blocking")]
use crate::blocking::builder::RequestBuilder as BlockingRequestBuilder;

/// 同步的 Client 别名
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
    fn builder_with_header<H: IntoIterator<Item = (HeaderName, HeaderValue)>>(
        &self,
        method: Method,
        url: Url,
        resource: CanonicalizedResource,
        headers: H,
    ) -> Result<BlockingRequestBuilder, BuilderError> {
        let method = method;
        let mut auth_builder = self.auth_builder.clone();
        auth_builder.method(&method);
        auth_builder.date(now());
        auth_builder.canonicalized_resource(resource);
        auth_builder.extend_headers(HeaderMap::from_iter(headers));

        let mut builder = self
            .client_middleware
            .request(method, url)
            .headers(auth_builder.get_headers()?);

        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        };

        Ok(builder)
    }
}

#[cfg(test)]
mod tests {
    use http::Method;

    use super::*;
    use crate::{
        builder::ArcPointer,
        config::{BucketBase, Config},
        file::AlignBuilder,
        types::object::ObjectBase,
        BucketName,
    };

    use std::time::Duration;

    #[test]
    fn from_config() {
        let config = Config::try_new("foo1", "foo2", "qingdao", "foo4").unwrap();
        let client = ClientArc::from_config(config);

        assert_eq!(client.bucket, "foo4".parse::<BucketName>().unwrap());
    }

    #[test]
    fn timeout() {
        let config = Config::try_new("foo1", "foo2", "qingdao", "foo4").unwrap();
        let mut client = ClientArc::from_config(config);

        assert!(client.timeout.is_none());

        client.timeout(Duration::new(10, 0));

        assert!(client.timeout.is_some());

        assert_eq!(client.timeout, Some(Duration::new(10, 0)));
    }

    #[test]
    fn test_timeout_with_builder() {
        let mut client = ClientArc::test_init();
        client.timeout(Duration::new(11, 0));
        let (url, resource) = client
            .get_object_base("9AB932LY.jpeg")
            .unwrap()
            .get_url_resource([]);
        let builder = client.builder_with_header(Method::HEAD, url, resource, []);
        let builder = builder.unwrap();

        let request = builder.build().unwrap();
        let timeout = request.timeout().unwrap().to_owned();
        assert_eq!(timeout, Duration::new(11, 0));
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn test_timeout_with_builder_blocking() {
        use crate::file::blocking::AlignBuilder;

        let mut client = ClientRc::test_init();
        client.timeout(Duration::new(11, 0));
        let (url, resource) = client
            .get_object_base("9AB932LY.jpeg")
            .unwrap()
            .get_url_resource([]);
        let builder = client.builder_with_header(Method::HEAD, url, resource, []);
        let builder = builder.unwrap();

        let request = builder.build().unwrap();
        let timeout = request.timeout().unwrap().to_owned();
        assert_eq!(timeout, Duration::new(11, 0));
    }

    #[test]
    fn get_object_base() {
        use std::sync::Arc;

        let config = Config::try_new("foo1", "foo2", "qingdao", "foo4").unwrap();
        let client = ClientArc::from_config(config);

        let base = client.get_object_base("file111").unwrap();

        let base2 = ObjectBase::<ArcPointer>::new(
            Arc::new(BucketBase::new(
                "foo4".parse().unwrap(),
                "qingdao".parse().unwrap(),
            )),
            "file111",
        )
        .unwrap();
        assert!(base == base2);
    }
}
