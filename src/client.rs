


use std::error::Error;

use reqwest::blocking::{self,RequestBuilder};
use reqwest::header::{HeaderMap};

use crate::auth::{Auth,VERB};
use chrono::prelude::*;

/// # 构造请求的客户端结构体
pub struct Client<'a>{
  access_key_id: &'a str,
  access_key_secret: &'a str,
  pub endpoint: &'a str,
  pub bucket: &'a str,
  //pub headers: HashMap<String, String>,
}

impl<'a> Client<'a> {
  pub fn new(access_key_id: &'a str, access_key_secret: &'a str, endpoint: &'a str, bucket: &'a str) -> Client<'a> {
    Client{
      access_key_id,
      access_key_secret,
      endpoint,
      bucket,
    }
  }

  // TODO
  pub fn canonicalized_resource(&self) -> String{
    if self.bucket.len()>0 {
      format!("/{}/", self.bucket)
    }else{
      "/".to_string()
    }
  }

  /// # 获取当前时间段 GMT 格式
  pub fn date(&self) -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%a, %d %b %Y %T GMT").to_string()
  }

  /// # 向 OSS 发送请求的封装
  /// 参数包含请求的：
  /// 
  /// - method
  /// - url
  /// - headers (可选)
  /// 
  /// 返回值是一个 reqwest 的请求创建器 `reqwest::blocking::RequestBuilder`
  /// 
  /// 返回后，可以再加请求参数，然后可选的进行发起请求
  /// 
  pub fn builder(&self, method: VERB, url: &str, headers: Option<HeaderMap>) -> RequestBuilder{
    let client = blocking::Client::new();

    let auth = Auth{
      access_key_id: self.access_key_id,
      access_key_secret: self.access_key_secret,
      verb: method.clone(),
      date: &self.date(),
      content_type: None,
      content_md5: None,
      canonicalized_resource: &self.canonicalized_resource(),
      headers: headers,
    };

    let all_headers: HeaderMap = auth.get_headers();

    client.request(method.0, url)
      .headers(all_headers)
  }
}

/// # OSS 对象的特征
/// 里面包含对象必须实现的接口
pub trait OssObject {

  /// # 将 xml 转换成 OSS 结构体的接口
  fn from_xml(xml: String) -> Result<Self, Box<dyn Error>> where Self: Sized;
}