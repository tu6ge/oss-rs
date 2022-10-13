use infer::Infer;
use reqwest::header::{HeaderMap};

use crate::auth::{VERB, AuthBuilder, AuthGetHeader};
use crate::builder::{ClientWithMiddleware, Middleware, RequestBuilder};
#[cfg(feature = "blocking")]
use crate::builder::blocking as blocking_builder;
use crate::config::{BucketBase, Config};
use chrono::prelude::*;
use reqwest::Url;
use crate::errors::{OssResult};

use std::sync::Arc;
#[cfg(feature = "plugin")]
use std::sync::Mutex;
#[cfg(feature = "plugin")]
use crate::plugin::{Plugin};
#[cfg(feature = "plugin")]
#[cfg_attr(test, mockall_double::double)]
use crate::plugin::PluginStore;

use crate::types::{KeyId, KeySecret, EndPoint, BucketName, CanonicalizedResource};

/// # 构造请求的客户端结构体
#[non_exhaustive]
#[derive(Default)]
pub struct Client{
    auth_builder: AuthBuilder,
    client_middleware: ClientWithMiddleware,
    #[cfg(feature = "blocking")]
    blocking_client_middleware: blocking_builder::ClientWithMiddleware,
    endpoint: EndPoint,
    bucket: BucketName,
    
    #[cfg(feature = "plugin")]
    pub plugins: Mutex<PluginStore>,
    pub infer: Infer,
}

#[cfg_attr(test, mockall::automock)]
impl Client {
    
    pub fn new(access_key_id: KeyId, access_key_secret: KeySecret, endpoint: EndPoint, bucket: BucketName) -> Client {
        let auth_builder = AuthBuilder::default()
            .key(access_key_id)
            .secret(access_key_secret);
        
        Client{
            auth_builder,
            client_middleware: ClientWithMiddleware::default(),
            #[cfg(feature = "blocking")]
            blocking_client_middleware: blocking_builder::ClientWithMiddleware::default(),
            endpoint,
            bucket,
            #[cfg(feature = "plugin")]
            plugins: Mutex::new(PluginStore::default()),
            infer: Infer::default(),
        }
    }
    
    pub fn from_config(config: &Config) -> Client{
        let auth_builder = AuthBuilder::default()
            .key(config.key())
            .secret(config.secret());
        
        Client{
            auth_builder,
            client_middleware: ClientWithMiddleware::default(),
            #[cfg(feature = "blocking")]
            blocking_client_middleware: blocking_builder::ClientWithMiddleware::default(),
            endpoint: config.endpoint(),
            bucket: config.bucket(),
            #[cfg(feature = "plugin")]
            plugins: Mutex::new(PluginStore::default()),
            infer: Infer::default(),
        }
    }

    pub fn set_bucket_name(&mut self, bucket: BucketName){
        self.bucket = bucket
    }

    /// # 用于模拟请求 OSS 接口
    /// 默认直接请求 OSS 接口，如果设置中间件，则可以中断请求，对 Request 做一些断言，对 Response 做一些模拟操作
    pub fn middleware(mut self, middleware: Arc<dyn Middleware>) -> Self{
        self.client_middleware.middleware(middleware);
        self
    }

    #[cfg(feature = "blocking")]
    pub fn blocking_middleware(mut self, middleware: Arc<dyn blocking_builder::Middleware>){
        self.blocking_client_middleware.middleware(middleware);
    }

    /// # 注册插件
    #[cfg(feature = "plugin")]
    pub fn plugin(mut self, mut plugin: Box<dyn Plugin>) -> OssResult<Client> {
        plugin.initialize(&mut self)?;

        self.plugins.lock().unwrap().insert(plugin);
        Ok(self)
    }

    pub fn get_bucket_base(&self) -> BucketBase {
        BucketBase::new(self.bucket.to_owned(), self.endpoint.to_owned())
    }

    pub fn get_bucket_url(&self) -> OssResult<Url>{
        self.get_bucket_base().to_url()
    }

    pub fn get_endpoint_url(&self) -> OssResult<Url>{
        self.endpoint.to_url()
    }

    /// # 向 OSS 发送请求的封装
    /// 参数包含请求的：
    /// 
    /// - method
    /// - url
    /// - [CanonicalizedResource](https://help.aliyun.com/document_detail/31951.html#section-rvv-dx2-xdb)  
    /// 
    /// 返回值是一个 reqwest 的请求创建器 `reqwest::blocking::RequestBuilder`
    /// 
    /// 返回后，可以再加请求参数，然后可选的进行发起请求
    #[cfg(feature = "blocking")]
    #[cfg_attr(not(test), inline)]
    pub fn blocking_builder(&self, method: VERB, url: &Url, resource: CanonicalizedResource) -> OssResult<blocking_builder::RequestBuilder>{
        self.blocking_builder_with_header(method, url, resource, None)
    }
    
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
    /// 
    #[cfg(feature = "blocking")]
    pub fn blocking_builder_with_header(&self, method: VERB, url: &Url, resource: CanonicalizedResource, headers: Option<HeaderMap>) -> OssResult<blocking_builder::RequestBuilder>{

        let headers = self.auth_builder.clone()
            .verb(method.to_owned())
            .date(now().into())
            .canonicalized_resource(resource)
            .with_headers(headers)
            .get_headers()?;

        Ok(self.blocking_client_middleware.request(method.into(), url.to_owned())
            .headers(headers))
    }

    /// builder 方法的异步实现
    #[cfg_attr(not(test), inline)]
    pub async fn builder(&self, method: VERB, url: &Url, resource: CanonicalizedResource)
    -> OssResult<RequestBuilder>
    {
        self.builder_with_header(method, url, resource, None).await
    }

    /// builder 方法的异步实现
    /// 带 header 参数
    pub async fn builder_with_header(&self, method: VERB, url: &Url, resource: CanonicalizedResource, headers: Option<HeaderMap>) 
    -> OssResult<RequestBuilder>
    {
        let headers = self.auth_builder.clone()
            .verb(method.to_owned())
            .date(now().into())
            .canonicalized_resource(resource)
            .with_headers(headers)
            .get_headers()?;

        Ok(self.client_middleware.request(method.into(), url.to_owned())
            .headers(headers))
    }

}

#[cfg(not(test))]
#[inline]
fn now() -> DateTime<Utc>{
    Utc::now()
}

#[cfg(test)]
fn now() -> DateTime<Utc>{
    let naive = NaiveDateTime::parse_from_str("2022/10/6 20:40:00", "%Y/%m/%d %H:%M:%S").unwrap();
    DateTime::from_utc(naive, Utc)
}
