
//extern crate base64;

use chrono::{DateTime, Utc};
use sha1::Sha1;
use hmac::{Hmac, Mac};
use base64::{encode};
use reqwest::{Method};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, IntoHeaderName, CONTENT_TYPE};
use crate::errors::{OssResult, OssError};
use futures::executor::block_on;
// use http::Method;

#[derive(Clone)]
#[non_exhaustive]
pub struct VERB(pub Method);

#[derive(Default)]
pub struct Auth<'a>{
  pub access_key_id: &'a str,
  pub access_key_secret: &'a str,
  pub verb: VERB,
  pub content_md5: Option<&'a str>,
  pub content_type: Option<String>,
  pub date: String,
  // pub canonicalized_oss_headers: &'a str, // TODO
  pub canonicalized_resource: &'a str,
  pub headers: HeaderMap,
}

impl VERB {
  /// GET
  pub const GET: VERB = VERB(Method::GET);

  /// POST
  pub const POST: VERB = VERB(Method::POST);

  /// PUT
  pub const PUT: VERB = VERB(Method::PUT);

  /// DELETE
  pub const DELETE: VERB = VERB(Method::DELETE);

  /// HEAD
  pub const HEAD: VERB = VERB(Method::HEAD);

  /// OPTIONS
  pub const OPTIONS: VERB = VERB(Method::OPTIONS);

  /// CONNECT
  pub const CONNECT: VERB = VERB(Method::CONNECT);

  /// PATCH
  pub const PATCH: VERB = VERB(Method::PATCH);

  /// TRACE
  pub const TRACE: VERB = VERB(Method::TRACE);
}

impl From<VERB> for String {
  fn from(verb: VERB) -> Self {
    match verb.0 {
      Method::GET => "GET".into(),
      Method::POST => "POST".into(),
      Method::PUT => "PUT".into(),
      Method::DELETE => "DELETE".into(),
      Method::HEAD => "HEAD".into(),
      Method::OPTIONS => "OPTIONS".into(),
      Method::CONNECT => "CONNECT".into(),
      Method::PATCH => "PATCH".into(),
      Method::TRACE => "TRACE".into(),
      _ => "".into(),
    }
  }
}

impl From<&str> for VERB {
  fn from(str: &str) -> Self {
      match str {
          "POST"    => VERB(Method::POST),
          "GET"     => VERB(Method::GET),
          "PUT"     => VERB(Method::PUT),
          "DELETE"  => VERB(Method::DELETE),
          "HEAD"    => VERB(Method::HEAD),
          "OPTIONS" => VERB(Method::OPTIONS),
          "CONNECT" => VERB(Method::CONNECT),
          "PATCH"   => VERB(Method::PATCH),
          "TRACE"   => VERB(Method::TRACE),
          _ => VERB(Method::GET),
      }
  }
}

impl Default for VERB {
  fn default() -> Self {
      Self::GET
  }
}

type HmacSha1 = Hmac<Sha1>;

impl<'a> Auth<'a> {

  /// # 获取所有 header 信息
  /// 
  /// 包含 *公共 header*, *业务 header* 以及 **签名**
  pub fn get_headers(self) -> OssResult<HeaderMap> {
    block_on(self.async_get_headers())
  }

  pub async fn async_get_headers(self) -> OssResult<HeaderMap> {
    let mut map= self.headers.clone();

    map.insert("AccessKeyId", self::to_value(self.access_key_id)?);
    map.insert("SecretAccessKey", self::to_value(self.access_key_secret)?);
    map.insert(
      "VERB", 
      self.verb.0.to_string()
        .parse().map_err(|_| OssError::Input("VERB parse error".to_string()))?);
    if let Some(a) = self.content_md5 {
      map.insert("Content-MD5",self::to_value(a)?);
    }
    if let Some(a) = &self.content_type {
      map.insert(
        "Content-Type",
        a.parse().map_err(|_| OssError::Input("Content-Type parse error".to_string()))?);
    }
    map.insert("Date",self::string_to_value(self.date.clone())?);
    map.insert("CanonicalizedResource", self::to_value(self.canonicalized_resource)?);

    let sign = self.sign()?;
    let sign = format!("OSS {}:{}", self.access_key_id, &sign);
    map.insert(
      "Authorization", 
      sign.parse().map_err(|_| OssError::Input("Authorization parse error".to_string()))?);

    //println!("header list: {:?}",map);
    Ok(map)
  }

  /// # 业务 header
  /// 
  /// 将 header 中除了共同部分的，转换成字符串，一般是 `x-oss-` 开头的
  /// 
  /// 用于生成签名 
  pub fn header_str(&self) -> OssResult<Option<String>> {
    //return Some("x-oss-copy-source:/honglei123/file1.txt");
    let mut header: Vec<(&HeaderName, &HeaderValue)> = self.headers.iter().filter(|(k,_v)|{
      k.as_str().starts_with("x-oss-")
    }).collect();
    if header.len()==0{
      return Ok(None);
    }
    header.sort_by(|(k1,_),(k2,_)| k1.to_string().cmp(&k2.to_string()));
    let header_vec: Vec<String> = header.into_iter().map(|(k,v)|{
      let value = k.as_str().to_owned() + ":" 
        + v.to_str().unwrap();
      value
    }).collect();

    Ok(Some(header_vec.join("\n")))
  }

  /// 计算签名
  pub fn sign(&self) -> OssResult<String> {
    let method = self.verb.0.to_string();
    let mut content = String::new();

    let content_type_str;

    let str: String = method
      + "\n"
      + match self.content_md5 {
        Some(str)=> {
          content.clear();
          content.push_str(str);
          &content
        },
        None => ""
      }
      + "\n"
      + match &self.content_type {
        Some(str) => {
          content_type_str = str.clone();
          &content_type_str
        },
        None => ""
      }
      + "\n"
      + &self.date + "\n"
      + match self.header_str()? {
        Some(str) => {
          content.clear();
          content.push_str(&str);
          content.push_str("\n");
          &content
        },
        None => ""
      }
      + self.canonicalized_resource;
    
    #[cfg(test)]
    println!("auth str: {}", str);
    
    let secret = self.access_key_secret.as_bytes();
    let str_u8 = str.as_bytes();
    
    let mut mac = HmacSha1::new_from_slice(secret)?;

    mac.update(str_u8);

    let sha1 = mac.finalize().into_bytes();

    Ok(encode(sha1))
  }

}

pub fn to_value(value: &str) -> OssResult<HeaderValue>{
  value.parse()
    .map_err(|_| OssError::Input("invalid HeaderValue".to_string()))
}

pub fn string_to_value(value: String) -> OssResult<HeaderValue>{
  value.as_str().parse()
    .map_err(|_| OssError::Input("invalid HeaderValue".to_string()))
}

#[derive(Default)]
pub struct AuthBuilder<'a>{
  pub auth: Auth<'a>,
}

impl<'a> AuthBuilder<'a> {
  /// 给 key 赋值
  /// 
  /// ```
  /// use aliyun_oss_client::auth::AuthBuilder;
  /// 
  /// let mut builder = AuthBuilder::default();
  /// assert_eq!(builder.auth.access_key_id, "");
  /// builder = builder.key("bar");
  /// assert_eq!(builder.auth.access_key_id, "bar");
  /// ```
  pub fn key<T: Into<&'a str>>(mut self, key: T) -> Self {
    self.auth.access_key_id = key.into();
    self
  }

  /// 给 secret 赋值
  pub fn secret<T: Into<&'a str>>(mut self, secret: T) -> Self {
    self.auth.access_key_secret = secret.into();
    self
  }

  /// 给 verb 赋值
  pub fn verb<T: Into<VERB>>(mut self, verb: T) -> Self {
    self.auth.verb = verb.into();
    self
  }

  /// 给 content_md5 赋值
  pub fn content_md5<T: Into<&'a str>>(mut self, content_md5: T) -> Self {
    self.auth.content_md5 = Some(content_md5.into());
    self
  }

  /// 给 date 赋值
  pub fn date(mut self, date: DateTime<Utc>) -> Self {
    self.auth.date = date.format("%a, %d %b %Y %T GMT").to_string();
    self
  }

  /// 给 content_md5 赋值
  pub fn canonicalized_resource<T: Into<&'a str>>(mut self, data: T) -> Self {
    self.auth.canonicalized_resource = data.into();
    self
  }

  pub fn headers(mut self, headers: HeaderMap) -> Self {
    self.auth.headers = headers;
    self.type_with_header()
  }

  /// 给 header 序列添加新值
  pub fn header_insert<K: IntoHeaderName>(mut self, key: K, val: HeaderValue ) -> Self
  {
    self.auth.headers.insert(key, val);
    self
  }

  /// 通过 headers 给 content_type 赋值
  pub fn type_with_header(mut self) -> Self {
    let content_type = self.auth.headers.get(CONTENT_TYPE);

    if let Some(content_type) = content_type {
      if let Ok(value) = content_type.to_str() {
        self.auth.content_type = Some(String::from(value));
      }
    }
    self
  }

  /// 清理 headers
  pub fn header_clear(mut self) -> Self {
    self.auth.headers.clear();
    self
  }
}