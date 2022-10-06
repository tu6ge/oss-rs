use infer::Infer;
#[cfg(feature = "blocking")]
use reqwest::blocking::{self,RequestBuilder,Response};
use reqwest::{Client as AsyncClient, RequestBuilder as AsyncRequestBuilder, Response as AsyncResponse};
use reqwest::header::{HeaderMap};

use crate::auth::{VERB, AuthBuilder};
use crate::config::{BucketBase, Config};
use chrono::prelude::*;
use reqwest::Url;
use crate::errors::{OssResult,OssError};

#[cfg(feature = "plugin")]
use std::sync::Mutex;
#[cfg(feature = "plugin")]
use crate::plugin::{Plugin};
#[cfg(feature = "plugin")]
#[cfg_attr(test, mockall_double::double)]
use crate::plugin::PluginStore;

use async_trait::async_trait;
use crate::types::{KeyId, KeySecret, EndPoint, BucketName, CanonicalizedResource};

/// # 构造请求的客户端结构体
#[non_exhaustive]
#[derive(Default)]
pub struct Client{
    auth_builder: AuthBuilder,
    endpoint: EndPoint,
    bucket: BucketName,
    
    #[cfg(feature = "plugin")]
    pub plugins: Mutex<PluginStore>,
    pub infer: Infer,
}

impl Client {
    
    pub fn new(access_key_id: KeyId, access_key_secret: KeySecret, endpoint: EndPoint, bucket: BucketName) -> Client {
        let auth_builder = AuthBuilder::default()
            .key(access_key_id)
            .secret(access_key_secret);
        
        Client{
            auth_builder,
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
            endpoint: config.endpoint(),
            bucket: config.bucket(),
            #[cfg(feature = "plugin")]
            plugins: Mutex::new(PluginStore::default()),
            infer: Infer::default(),
        }
    }

    #[inline]
    pub fn set_bucket_name(&mut self, bucket: BucketName){
        self.bucket = bucket
    }

    #[inline]
    pub fn get_bucket_base(&self) -> BucketBase {
        BucketBase::new(self.bucket.to_owned(), self.endpoint.to_owned())
    }

    #[inline]
    pub fn get_bucket_url(&self) -> OssResult<Url>{
        self.get_bucket_base().to_url()
    }

    #[inline]
    pub fn get_endpoint_url(&self) -> OssResult<Url>{
        self.endpoint.to_url()
    }

    /// # 注册插件
    #[cfg(feature = "plugin")]
    pub fn plugin(mut self, mut plugin: Box<dyn Plugin>) -> OssResult<Client> {
        plugin.initialize(&mut self)?;

        self.plugins.lock().unwrap().insert(plugin);
        Ok(self)
    }
    
    /// # 向 OSS 发送请求的封装
    /// 参数包含请求的：
    /// 
    /// - method
    /// - url
    /// - headers (可选)
    /// - bucket 要操作的 bucket ，默认为当前配置的 bucket 
    /// 
    /// 返回值是一个 reqwest 的请求创建器 `reqwest::blocking::RequestBuilder`
    /// 
    /// 返回后，可以再加请求参数，然后可选的进行发起请求
    /// 
    #[cfg(feature = "blocking")]
    pub fn blocking_builder(&self, method: VERB, url: &Url, headers: Option<HeaderMap>, resource: CanonicalizedResource) -> OssResult<RequestBuilder>{
        let client = blocking::Client::new();

        let builder = self.auth_builder.clone()
            .verb(method.to_owned())
            .date(Utc::now())
            .canonicalized_resource(resource)
            .with_headers(headers)
            ;

        let all_headers = builder.get_headers()?;

        Ok(client.request(method.0, url.to_owned())
            .headers(all_headers))
    }

    /// builder 方法的异步实现
    pub async fn builder(&self, method: VERB, url: &Url, headers: Option<HeaderMap>, resource: CanonicalizedResource) -> OssResult<AsyncRequestBuilder>{
        let client = AsyncClient::new();

        let builder = self.auth_builder.clone()
            .verb(method.to_owned())
            .date(Utc::now())
            .canonicalized_resource(resource)
            .with_headers(headers)
            ;

        let all_headers = builder.get_headers()?;

        Ok(client.request(method.0, url.to_owned())
            .headers(all_headers))
    }

}

#[cfg(feature = "blocking")]
pub trait ReqeustHandler {
    fn handle_error(self) -> OssResult<Self> where Self: Sized;
}

#[cfg(feature = "blocking")]
impl ReqeustHandler for Response {

    /// # 收集并处理 OSS 接口返回的错误
    fn handle_error(self) -> OssResult<Response>
    {
        #[cfg_attr(test, mockall_double::double)]
        use crate::errors::OssService;

        let status = self.status();
    
        if status != 200 && status != 204{
            let body = self.text()?;
            let error = OssService::new(body);
            return Err(OssError::OssService(error));
        }

        Ok(self)
    }
}

#[async_trait]
pub trait AsyncRequestHandle {
    async fn handle_error(self) -> OssResult<AsyncResponse>;
}

#[async_trait]
impl AsyncRequestHandle for AsyncResponse{
    /// # 收集并处理 OSS 接口返回的错误
    async fn handle_error(self) -> OssResult<AsyncResponse>
    {
        #[cfg_attr(test, mockall_double::double)]
        use crate::errors::OssService;

        let status = self.status();
        
        if status != 200 && status != 204 {
            let body = self.text().await?;
            let error = OssService::new(body);
            return Err(OssError::OssService(error));
        }

        Ok(self)
    }
}