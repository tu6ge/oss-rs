
use std::collections::HashMap;
use reqwest::{blocking,ClientBuilder,Method,Url,header};

use crate::auth::{Auth,VERB};

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

  pub fn builder(&self, method: VERB, url: &str) -> reqwest::Result<String>{
    let client = blocking::Client::new();
    let auth = Auth{
      access_key_id: self.access_key_id,
      access_key_secret: self.access_key_secret,
      verb: method.clone(),
      date: "abc", // TODO 
      content_type: None,
      content_md5: None,
      canonicalized_resource: self.canonicalized_resource,
    };

    let headers: header::HeaderMap = auth.try_into().unwrap();

    client.request(method.0,url)
      .headers(headers)
      .send().unwrap().text()
  }
}
