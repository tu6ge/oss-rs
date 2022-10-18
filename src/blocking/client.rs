use infer::Infer;
use reqwest::header::{HeaderMap};

use crate::auth::{VERB, AuthBuilder, AuthGetHeader};
use super::builder::{ClientWithMiddleware, Middleware, RequestBuilder};
use crate::config::{BucketBase, Config, InvalidConfig};
use chrono::prelude::*;
use reqwest::Url;
use crate::errors::{OssResult};

use std::env;
use std::{rc::Rc};

use crate::types::{KeyId, KeySecret, EndPoint, BucketName, CanonicalizedResource};

/// # 构造请求的客户端结构体
#[non_exhaustive]
#[derive(Default)]
pub struct Client{
    auth_builder: AuthBuilder,
    client_middleware: ClientWithMiddleware,
    endpoint: EndPoint,
    bucket: BucketName,

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
            endpoint,
            bucket,
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
            endpoint: config.endpoint(),
            bucket: config.bucket(),
            infer: Infer::default(),
        }
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
    /// # use aliyun_oss_client::blocking::client::Client;
    /// let client = Client::from_env();
    /// assert!(client.is_ok());
    /// ```
    pub fn from_env() -> Result<Self, InvalidConfig> {
        let key_id      = env::var("ALIYUN_KEY_ID").map_err(InvalidConfig::from)?;
        let key_secret  = env::var("ALIYUN_KEY_SECRET").map_err(InvalidConfig::from)?;
        let endpoint    = env::var("ALIYUN_ENDPOINT").map_err(InvalidConfig::from)?;
        let bucket      = env::var("ALIYUN_BUCKET").map_err(InvalidConfig::from)?;

        let auth_builder = AuthBuilder::default()
            .key(key_id.into())
            .secret(key_secret.into());
        
        Ok(Client{
            auth_builder,
            client_middleware: ClientWithMiddleware::default(),
            endpoint: endpoint.try_into().map_err(InvalidConfig::from)?,
            bucket: bucket.try_into().map_err(InvalidConfig::from)?,
            infer: Infer::default(),
        })
    }

    pub fn set_bucket_name(&mut self, bucket: BucketName){
        self.bucket = bucket
    }

    /// # 用于模拟请求 OSS 接口
    /// 默认直接请求 OSS 接口，如果设置中间件，则可以中断请求，对 Request 做一些断言，对 Response 做一些模拟操作
    pub fn middleware(mut self, middleware: Rc<dyn Middleware>) -> Self{
        self.client_middleware.middleware(middleware);
        self
    }

    pub fn get_bucket_base(&self) -> BucketBase {
        BucketBase::new(self.bucket.to_owned(), self.endpoint.to_owned())
    }

    pub fn get_bucket_url(&self) -> Url{
        self.get_bucket_base().to_url()
    }

    pub fn get_endpoint_url(&self) -> Url{
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
    #[cfg_attr(not(test), inline)]
    pub fn builder(&self, method: VERB, url: &Url, resource: CanonicalizedResource) -> OssResult<RequestBuilder>{
        self.builder_with_header(method, url, resource, None)
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
    pub fn builder_with_header(&self, method: VERB, url: &Url, resource: CanonicalizedResource, headers: Option<HeaderMap>) -> OssResult<RequestBuilder>{

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
