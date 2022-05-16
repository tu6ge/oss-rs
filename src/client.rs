

use reqwest::{blocking,ClientBuilder,Method,Url, Error};
use reqwest::blocking::{Response};
use reqwest::header::{CONTENT_TYPE, DATE,HeaderMap};

use crate::auth::{Auth,VERB};
use chrono::prelude::*;

pub struct Client<'a>{
  access_key_id: &'a str,
  access_key_secret: &'a str,
  canonicalized_resource: &'a str,
  //pub headers: HashMap<String, String>,
}

impl<'a> Client<'a> {
  pub fn new(access_key_id: &'a str, access_key_secret: &'a str, canonicalized_resource: &'a str) -> Client<'a> {
    Client{
      access_key_id,
      access_key_secret,
      canonicalized_resource,
    }
  }

  /// 获取当前时间段 GMT 格式
  pub fn date(&self) -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%a, %d %b %Y %T GMT").to_string()
  }

  /// 向 OSS 发送请求的封装
  pub fn request(&self, method: VERB, url: &str, headers: Option<HeaderMap>) -> reqwest::Result<Response>{
    let client = blocking::Client::new();

    let auth = Auth{
      access_key_id: self.access_key_id,
      access_key_secret: self.access_key_secret,
      verb: method.clone(),
      date: &self.date(),
      content_type: None,
      content_md5: None,
      canonicalized_resource: self.canonicalized_resource,
      headers: headers,
    };

    let all_headers: HeaderMap = auth.get_headers();

    client.request(method.0, url)
      .headers(all_headers)
      .send()
  }
}
