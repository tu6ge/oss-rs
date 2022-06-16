
//extern crate base64;

use sha1::Sha1;
use hmac::{Hmac, Mac};
use base64::{encode};
use reqwest::{Method};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use crate::errors::{OssResult, OssError};
// use http::Method;

#[derive(Clone)]
pub struct VERB(pub Method);

pub struct Auth<'a>{
  pub access_key_id: &'a str,
  pub access_key_secret: &'a str,
  pub verb: VERB,
  pub content_md5: Option<&'a str>,
  pub content_type: Option<String>,
  pub date: &'a str,
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
  pub fn get_headers(&self) -> OssResult<HeaderMap> {
    let mut map= match self.headers.to_owned() {
      Some(v) => v,
      None => HeaderMap::new(),
    };

    map.insert(self::to_name("AccessKeyId")?, self::to_value(self.access_key_id)?);
    map.insert(self::to_name("SecretAccessKey")?, self::to_value(self.access_key_secret)?);
    map.insert(
      self::to_name("VERB")?, 
      self.verb.0.to_string()
        .parse().map_err(|_| OssError::Input("VERB parse error".to_string()))?);
    if let Some(a) = self.content_md5 {
      map.insert(self::to_name("Content-MD5")?,self::to_value(a)?);
    }
    if let Some(a) = &self.content_type {
      map.insert(
        self::to_name("Content-Type")?,
        a.parse().map_err(|_| OssError::Input("Content-Type parse error".to_string()))?);
    }
    map.insert(self::to_name("Date")?,self::to_value(self.date)?);
    map.insert(self::to_name("CanonicalizedResource")?, self::to_value(self.canonicalized_resource)?);

    let sign = self.sign()?;
    let sign = format!("OSS {}:{}", self.access_key_id, &sign);
    map.insert(
      self::to_name("Authorization")?, 
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
    match self.headers.clone() {
      Some(header) => {
        let mut header: Vec<(&HeaderName, &HeaderValue)> = header.iter().filter(|(k,_v)|{
          k.as_str().starts_with("x-oss-")
        }).collect();
        header.sort_by(|(k1,_),(k2,_)| k1.to_string().cmp(&k2.to_string()));
        let header_vec: Vec<String> = header.into_iter().map(|(k,v)|{
          let value = k.as_str().to_owned() + ":" 
            + v.to_str().unwrap();
          value
        }).collect();
        Ok(Some(header_vec.join("\n")))
      },
      None => Ok(None),
    }
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
      + self.date + "\n"
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


pub fn to_name(name: &str) -> OssResult<HeaderName>{
  Ok(HeaderName::from_bytes(name.as_bytes())
    .map_err(|_| OssError::Input("invalid HeaderName".to_string()))?)
}

pub fn to_value(value: &str) -> OssResult<HeaderValue>{
  Ok(value.parse()
    .map_err(|_| OssError::Input("invalid HeaderValue".to_string()))?)
}