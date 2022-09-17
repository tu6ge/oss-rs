use infer::Infer;
#[cfg(feature = "blocking")]
use reqwest::blocking::{self,RequestBuilder,Response};
use reqwest::{Client as AsyncClient, RequestBuilder as AsyncRequestBuilder, Response as AsyncResponse};
use reqwest::header::{HeaderMap};

use crate::auth::{VERB, AuthBuilder};
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
  access_key_id: KeyId,
  access_key_secret: KeySecret,
  pub endpoint: EndPoint,
  pub bucket: BucketName,
  
  #[cfg(feature = "plugin")]
  pub plugins: Mutex<PluginStore>,

  pub infer: Infer,
}

impl Client {

  #[cfg(not(feature = "plugin"))]
  pub fn new(access_key_id: KeyId, access_key_secret: KeySecret, endpoint: EndPoint, bucket: BucketName) -> Client {
    Client{
      access_key_id,
      access_key_secret,
      endpoint,
      bucket,
      infer: Infer::default(),
    }
  }

  #[cfg(feature = "plugin")]
  pub fn new(access_key_id: KeyId, access_key_secret: KeySecret, endpoint: EndPoint, bucket: BucketName) -> Client {
    Client{
      access_key_id,
      access_key_secret,
      endpoint,
      bucket,
      plugins: Mutex::new(PluginStore::default()),
      infer: Infer::default(),
    }
  }

  pub fn set_bucket(&mut self, bucket: BucketName){
    self.bucket = bucket
  }

  /// # 注册插件
  #[cfg(feature = "plugin")]
  pub fn plugin(mut self, mut plugin: Box<dyn Plugin>) -> OssResult<Client> {
    plugin.initialize(&mut self)?;

    self.plugins.lock().unwrap().insert(plugin);
    Ok(self)
  }

  /// # 返回用于签名的 canonicalized_resource 值
  #[cfg(feature = "blocking")]
  pub fn canonicalized_resource(&self, url: &Url, bucket: Option<String>) -> OssResult<String> {
    use futures::executor::block_on;
    block_on(self.async_canonicalized_resource(url, bucket))
  }

  pub async fn async_canonicalized_resource(&self, url: &Url, bucket: Option<String>) -> OssResult<String> {
    #[cfg(feature = "plugin")]
    {
      let plugin_result = 
        self.plugins.lock().unwrap()
          .get_canonicalized_resource(
            url
          )?;
      if let Some(result) = plugin_result {
        return Ok(result);
      }
    }

    let bucket = match bucket {
      Some(val) => val,
      None => self.bucket.to_string(),
    };
    if bucket.len()==0 {
      return Ok(format!("/"));
    }

    //println!("url.path(): {}", url.path());
    let path = urlencoding::decode(url.path());

    if let Err(_) = path {
      return Ok(format!("/"));
    }

    let path = path.unwrap();

    // 有 path 的情况
    if url.path().is_empty() == false && url.path() != "/" {
      match url.query() {
        Some(query_value) if query_value.is_empty() == false => {
          return Ok(format!("/{}{}?{}", bucket, path, query_value));
        },
        _ => return Ok(format!("/{}{}", bucket, path))
      }
    }

    // 无 path 的情况
    match url.query() {
      Some(query) => {
        // acl、uploads、location、cors、logging、website、referer、lifecycle、delete、append、tagging、objectMeta、uploadId、
        // partNumber、security-token、position、img、style、styleName、replication、replicationProgress、replicationLocation、cname、bucketInfo、
        // comp、qos、live、status、vod、startTime、endTime、symlink、x-oss-process
        if query == "acl"
        || query == "bucketInfo"{
          return Ok(format!("/{}/?{}", bucket, query));
        }else if self.is_bucket_url(url, &bucket) {
          // 基于某个 bucket 调用api 时
          // 如果查询条件中有翻页的话，则忽略掉其他字段
          let query_pairs = url.query_pairs();
          for (key,value) in query_pairs {
            if key.into_owned().starts_with("continuation-token") {
              return Ok(format!("/{}/?continuation-token={}", bucket, value.into_owned()))
            }
          }
        }
        return Ok(format!("/{}/", bucket))
      },
      None => {
        return Ok(format!("/"));
      }
    }
  }

  pub fn get_bucket_url(&self) -> OssResult<Url>{
    let mut url = self.endpoint.into_url()?;
    
    let bucket_url = self.bucket.to_string() + "."
       + &url.host().ok_or(OssError::Input("parse host faied".to_string()))?.to_string();

    url.set_host(Some(&bucket_url)).map_err(|_| OssError::Input("set_host error".to_string()))?;
    
    Ok(url)
  }

  pub fn is_bucket_url(&self, url: &Url, bucket: &String) -> bool {
    match url.host() {
      Some(host) => {
        let mut pre_host = String::from(bucket).to_owned();
        pre_host.push_str(".");
        host.to_string().starts_with(&pre_host)
      }, 
      None => false,
    }
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
  pub fn blocking_builder(&self, method: VERB, url: &Url, headers: Option<HeaderMap>, bucket: Option<String>) -> OssResult<RequestBuilder>{
    let client = blocking::Client::new();

    let canonicalized_resource = self.canonicalized_resource(&url, bucket)?;

    let mut builder = AuthBuilder::default()
      .key(self.access_key_id.clone())
      .secret(self.access_key_secret.clone())
      .verb(method.to_owned())
      .date(Utc::now())
      .canonicalized_resource(CanonicalizedResource::new(canonicalized_resource))
    ;
    
    if let Some(headers) = headers {
      builder = builder.headers(headers);
    };

    let all_headers: HeaderMap = builder.auth.get_headers()?;

    Ok(client.request(method.0, url.to_owned())
      .headers(all_headers))
  }

  /// builder 方法的异步实现
  pub async fn builder(&self, method: VERB, url: &Url, headers: Option<HeaderMap>, bucket: Option<String>) -> OssResult<AsyncRequestBuilder>{
    let client = AsyncClient::new();

    let canonicalized_resource = self.async_canonicalized_resource(&url, bucket).await?;

    let mut builder = AuthBuilder::default()
      .key(self.access_key_id.clone())
      .secret(self.access_key_secret.clone())
      .verb(method.to_owned())
      .date(Utc::now())
      .canonicalized_resource(CanonicalizedResource::new(canonicalized_resource))
    ;
    
    if let Some(headers) = headers {
      builder = builder.headers(headers);
    };

    let all_headers: HeaderMap = builder.auth.async_get_headers().await?;

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
    
    if status != 200 && status != 204{
      let body = self.text().await?;

      let error = OssService::new(body);

      return Err(OssError::OssService(error));
    }

    Ok(self)
  }
}