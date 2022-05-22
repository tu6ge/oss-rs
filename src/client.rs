


use std::error::Error;

use reqwest::blocking::{self,RequestBuilder,Response};
use reqwest::header::{HeaderMap,HeaderValue};

use crate::auth::{Auth,VERB};
use chrono::prelude::*;
use url::Url;

/// # 构造请求的客户端结构体
pub struct Client<'a>{
  access_key_id: &'a str,
  access_key_secret: &'a str,
  pub endpoint: &'a str,
  pub bucket: &'a str,
  //pub headers: HashMap<String, String>,
}

impl<'a> Client<'a> {
  pub const ERROR_REQUEST_ALIYUN_API: &'a str = "request aliyun api fail";

  pub fn new(access_key_id: &'a str, access_key_secret: &'a str, endpoint: &'a str, bucket: &'a str) -> Client<'a> {
    Client{
      access_key_id,
      access_key_secret,
      endpoint,
      bucket,
    }
  }

  /// # 返回用于签名的 canonicalized_resource 值
  pub fn canonicalized_resource(&self, url: &Url) -> String{
    if self.bucket.len()==0 {
      return "/".to_string()
    }

    //println!("url.path(): {}", url.path());

    // 有 path 的情况
    if url.path().is_empty() == false && url.path() != "/" {
      match url.query() {
        Some(query_value) if query_value.is_empty() == false => {
          return format!("/{}{}?{}", self.bucket, url.path(), query_value);
        },
        _ => return format!("/{}{}", self.bucket, url.path())
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
          
          return format!("/{}/?{}", self.bucket, query)
        }else{
          // println!("匹配到的 query");
          return format!("/{}/", self.bucket)
        }
      },
      None => {
        return format!("/");
      }
    }
  }

  pub fn get_bucket_url(&self) -> Result<Url>{
    let mut url = Url::parse(self.endpoint).ok().expect("Invalid endpoint");
    
    let bucket_url = self.bucket.to_string() + "." + &url.host().unwrap().to_string();

    url.set_host(Some(&bucket_url)).expect("get bucket url failed");
    
    Ok(url)
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
  pub fn builder(&self, method: VERB, url: &Url, headers: Option<HeaderMap>) -> RequestBuilder{
    let client = blocking::Client::new();

    let auth = Auth{
      access_key_id: self.access_key_id,
      access_key_secret: self.access_key_secret,
      verb: method.clone(),
      date: &self.date(),
      content_type: match &headers {
        Some(head) => {
          let value  = head.get("Content-Type").unwrap();
          Some(value.to_str().unwrap().to_string())
        },
        None => None,
      },
      content_md5: None,
      canonicalized_resource: &self.canonicalized_resource(&url),
      headers: headers,
    };

    let all_headers: HeaderMap = auth.get_headers();

    client.request(method.0, url.to_string())
      .headers(all_headers)
  }

  /// # 错误处理
  /// 如果请求接口没有返回 200 状态，则触发 panic 
  /// 
  /// 并打印状态码和 x-oss-request-id
  pub fn handle_error(response: &mut Response)
  {
    let status = response.status();
    
    if status != 200 {
      let headers = response.headers();
      let request_id = headers.get("x-oss-request-id").unwrap().to_str().unwrap();
      panic!("aliyun response error, http status: {}, x-oss-request-id: {}, content", status, request_id);
    }
  }

  #[inline]
  pub fn string2option(string: String) -> Option<String> {
    if string.len() == 0 {
      return None
    }
    Some(string)
  }
}

/// # OSS 对象的特征
/// 里面包含对象必须实现的接口
pub trait OssObject {

  /// # 将 xml 转换成 OSS 结构体的接口
  fn from_xml(xml: String) -> Result<Self> where Self: Sized;
}



pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
