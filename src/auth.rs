
//extern crate base64;

use sha1::Sha1;
use hmac::{Hmac, Mac};
use base64::{encode};
use reqwest::{Method};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
// use http::Method;

#[derive(Clone)]
pub struct VERB(pub Method);

pub struct Auth<'a>{
  pub access_key_id: &'a str,
  pub access_key_secret: &'a str,
  pub verb: VERB,
  pub content_md5: Option<&'a str>,
  pub content_type: Option<&'a str>,
  pub date: &'a str, // TODO
  // pub canonicalized_oss_headers: &'a str, // TODO
  pub canonicalized_resource: &'a str,
  pub headers: Option<HeaderMap>,
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

type HmacSha1 = Hmac<Sha1>;

impl<'a> Auth<'a> {

  /// # 获取所有 header 信息
  /// 
  /// 包含 *公共 header*, *业务 header* 以及 **签名**
  pub fn get_headers(&self) -> HeaderMap {
    let mut map= match self.headers.to_owned() {
      Some(v) => v,
      None => HeaderMap::new(),
    };

    map.insert(self::to_name("AccessKeyId"), self::to_value(self.access_key_id));
    map.insert(self::to_name("SecretAccessKey"), self::to_value(self.access_key_secret));
    map.insert(self::to_name("VERB"), self.verb.0.to_string().parse().unwrap());
    if let Some(a) = self.content_md5 {
      map.insert(self::to_name("Content-MD5"),self::to_value(a));
    }
    if let Some(a) = self.content_type {
      map.insert(self::to_name("Content-Type"),self::to_value(a));
    }
    map.insert(self::to_name("Date"),self::to_value(self.date));
    map.insert(self::to_name("CanonicalizedResource"), self::to_value(self.canonicalized_resource));

    let sign = self.sign();
    let sign = "OSS ".to_owned() + &sign;
    map.insert(self::to_name("Authorization"), sign.parse().unwrap());

    map
  }

  /// # 业务 header
  /// 
  /// 将 header 中除了共同部分的，转换成字符串，一般是 `x-oss-` 开头的
  /// 
  /// 用于生成签名 
  pub fn header_str(&self) -> Option<&'a str> {
    None // TODO
  }

  /// 计算签名
  pub fn sign(&self) -> String {
    let method = self.verb.0.to_string();
    let mut content = String::new();

    let str: String = method
      + match self.content_md5 {
        Some(str)=> {
          content.clear();
          content.push_str(str);
          content.push_str("\n");
          &content
        },
        None => ""
      }
      
      + match self.content_type {
        Some(str) => {
          content.clear();
          content.push_str(str);
          content.push_str("\n");
          &content
        },
        None => ""
      }
      + self.date + "\n"
      + match self.header_str() {
        Some(str) => {
          content.clear();
          content.push_str(str);
          content.push_str("\n");
          &content
        },
        None => ""
      } 
      + self.canonicalized_resource;
    
    let secret = self.access_key_secret.as_bytes();
    let str_u8 = str.as_bytes();
    
    let mut mac = HmacSha1::new_from_slice(secret)
    .expect("HMAC can take key of any size");

    mac.update(str_u8);

    let sha1 = mac.finalize().into_bytes();

    encode(sha1)
  }

}


fn to_name(name: &str) -> HeaderName{
  HeaderName::from_bytes(name.as_bytes()).unwrap()
}

fn to_value(value: &str) -> HeaderValue{
  value.parse().unwrap()
}