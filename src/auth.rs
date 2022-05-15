
//extern crate base64;

use sha1::Sha1;
use hmac::{Hmac, Mac};
use base64::{encode};
use reqwest::{Method};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
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
  pub fn sign(&self) -> String {
    let method = self.verb.0.to_string();
    let str: String = method
      + match self.content_md5 {
        Some(str)=> str,
        None => ""
      } 
      + "\n"
      + match self.content_type {
        Some(str) => str,
        None => ""
      }
      + "\n"
      + self.date + "\n"
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

impl<'a> From<Auth<'a>> for HeaderMap {
  fn from(auth: Auth<'a>) -> Self {
    let mut map= HeaderMap::with_capacity(7);

    map.insert(self::to_name("AccessKeyId"), self::to_value(auth.access_key_id));
    map.insert(self::to_name("SecretAccessKey"), self::to_value(auth.access_key_secret));
    map.insert(self::to_name("VERB"), auth.verb.0.to_string().parse().unwrap());
    if let Some(a) = auth.content_md5 {
      map.insert(self::to_name("Content-MD5"),self::to_value(a));
    }
    if let Some(a) = auth.content_type {
      map.insert(self::to_name("Content-Type"),self::to_value(a));
    }
    map.insert(self::to_name("Date"),self::to_value(auth.date));
    map.insert(self::to_name("CanonicalizedResource"), self::to_value(auth.canonicalized_resource));

    map
  }

  
}

fn to_name(name: &str) -> HeaderName{
  HeaderName::from_bytes(name.as_bytes()).unwrap()
}

fn to_value(value: &str) -> HeaderValue{
  value.parse().unwrap()
}