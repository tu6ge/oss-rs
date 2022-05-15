
//extern crate base64;

use sha1::Sha1;
use hmac::{Hmac, Mac};
use base64::{encode, decode};
// use http::Method;

pub struct Auth<'a>{
  pub access_key_id: &'a str,
  pub access_key_secret: &'a str,
  pub verb: VERB,
  pub content_md5: &'a str,
  pub content_type: &'a str,
  pub date: &'a str, // TODO
  // pub canonicalized_oss_headers: &'a str, // TODO
  pub canonicalized_resource: &'a str,
}

#[derive(Copy,Clone)]
pub enum VERB {
  PUT,
  GET,
  POST,
  HEAD,
  DELETE,
}

impl From<VERB> for String {
  fn from(verb: VERB) -> Self {
    match verb {
      VERB::PUT => "PUT".into(),
      VERB::GET => "GET".into(),
      VERB::POST => "POST".into(),
      VERB::HEAD => "HEAD".into(),
      VERB::DELETE => "DELETE".into(),
    }
  }
}

type HmacSha1 = Hmac<Sha1>;

impl<'a> Auth<'a> {
  pub fn sign(&self) -> String {
    let str: String = String::from(self.verb)
      + self.content_md5 + "\n"
      + self.content_type + "\n"
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