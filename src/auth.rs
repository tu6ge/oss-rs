
//extern crate base64;

use std::convert::TryInto;
use reqwest::{Method};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, IntoHeaderName, CONTENT_TYPE};
use crate::types::{KeyId,KeySecret,ContentMd5,CanonicalizedResource, Date, ContentType};
use crate::errors::{OssResult, OssError};
// use http::Method;
// #[cfg(test)]
// use mockall::{automock, mock, predicate::*};

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(test, derive(Debug))]
#[non_exhaustive]
pub struct VERB(pub Method);

#[derive(Default, Clone)]
pub struct Auth{
  pub access_key_id: KeyId,
  pub access_key_secret: KeySecret,
  pub verb: VERB,
  pub content_md5: Option<ContentMd5>,
  pub content_type: Option<ContentType>,
  pub date: Date,
  // pub canonicalized_oss_headers: &'a str, // TODO
  pub canonicalized_resource: CanonicalizedResource,
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

  #[inline]
  pub fn to_string(&self) -> String {
    self.0.to_string()
  }
}

impl TryInto<HeaderValue> for VERB {
    type Error = OssError;
    fn try_into(self) -> OssResult<HeaderValue> {
        self.0.to_string().parse::<HeaderValue>()
        .map_err(|_| OssError::Input("VERB parse error".to_string()))
    }
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

impl Auth {

  /// 转化成 OssHeader
  pub fn to_oss_header(&self) -> OssResult<OssHeader> {
    //return Some("x-oss-copy-source:/honglei123/file1.txt");
    let mut header: Vec<(&HeaderName, &HeaderValue)> = self.headers.iter().filter(|(k,_v)|{
      k.as_str().starts_with("x-oss-")
    }).collect();
    if header.len()==0{
      return Ok(OssHeader(None));
    }

    header.sort_by(|(k1,_),(k2,_)| k1.to_string().cmp(&k2.to_string()));
    let header_vec: Vec<String> = header.into_iter().map(|(k,v)| -> OssResult<String> {
      let val = v.to_str().map_err(|e| OssError::ToStr(e.to_string()));

      let value = k.as_str().to_owned() + ":" 
        + val?;
      Ok(value)
    }).filter(|res|res.is_ok())
    // 这里的 unwrap 不会 panic
    .map(|res|res.unwrap())
    .collect();

    Ok(OssHeader(Some(header_vec.join("\n"))))
  }

}

#[derive(Default, Clone)]
pub struct AuthBuilder{
  pub auth: Auth,
}

impl AuthBuilder{
  /// 给 key 赋值
  /// 
  /// ```
  /// use aliyun_oss_client::auth::AuthBuilder;
  /// 
  /// let mut builder = AuthBuilder::default();
  /// assert_eq!(builder.auth.access_key_id.as_ref(), "");
  /// builder = builder.key("bar");
  /// assert_eq!(builder.auth.access_key_id.as_ref(), "bar");
  /// ```
  pub fn key<K: Into<KeyId>>(mut self, key: K) -> Self {
    self.auth.access_key_id = key.into();
    self
  }

  /// 给 secret 赋值
  pub fn secret<K: Into<KeySecret>>(mut self, secret: K) -> Self {
    self.auth.access_key_secret = secret.into();
    self
  }

  /// 给 verb 赋值
  pub fn verb<T: Into<VERB>>(mut self, verb: T) -> Self {
    self.auth.verb = verb.into();
    self
  }

  /// 给 content_md5 赋值
  pub fn content_md5<M: Into<ContentMd5>>(mut self, content_md5: M) -> Self {
    self.auth.content_md5 = Some(content_md5.into());
    self
  }

  /// 给 date 赋值
  /// 
  /// example
  /// ```
  /// use chrono::Utc;
  /// let builder = aliyun_oss_client::auth::AuthBuilder::default()
  ///    .date(Utc::now());
  /// ```
  pub fn date<D: Into<Date>>(mut self, date: D) -> Self {
    self.auth.date = date.into();
    self
  }

  /// 给 content_md5 赋值
  pub fn canonicalized_resource<C: Into<CanonicalizedResource>>(mut self, data: C) -> Self {
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
  /// 
  /// TODO 需要处理异常的情况
  pub fn type_with_header(mut self) -> Self {
    let content_type = self.auth.headers.get(CONTENT_TYPE);

    if let Some(ct) = content_type {
      let t: OssResult<ContentType> = ct.clone().try_into();
      if let Ok(value) = t {
        self.auth.content_type = Some(value);
      }
    }
    self
  }

  /// 清理 headers
  pub fn header_clear(mut self) -> Self {
    self.auth.headers.clear();
    self
  }

  pub fn get_headers(&self) -> OssResult<HeaderMap>{
    let mut map = HeaderMap::from_auth(&self.auth)?;

    let oss_header = self.auth.to_oss_header()?;
    let sign_string = SignString::new(&self.auth, &oss_header)?;
    let sign = sign_string.to_sign()?;
    map.append_sign(sign)?;

    Ok(map)
  }
}

pub trait AuthHeader {
  fn from_auth(auth: &Auth) -> OssResult<Self> where Self: Sized;
  fn append_sign(&mut self, sign: Sign) -> OssResult<Option<HeaderValue>>;
}

impl AuthHeader for HeaderMap {
  fn from_auth(auth: &Auth) -> OssResult<Self> {
    let mut map= auth.headers.clone();

    map.insert("AccessKeyId", auth.access_key_id.as_ref().try_into()?);
    map.insert("SecretAccessKey", auth.access_key_secret.as_ref().try_into()?);
    map.insert("VERB",auth.verb.clone().try_into()?);

    if let Some(a) = auth.content_md5.clone() {
      map.insert("Content-MD5",a.try_into()?);
    }
    if let Some(a) = &auth.content_type {
      map.insert("Content-Type",a.as_ref().try_into()?);
    }
    map.insert("Date",auth.date.as_ref().try_into()?);
    map.insert("CanonicalizedResource", auth.canonicalized_resource.as_ref().try_into()?);

    //println!("header list: {:?}",map);
    Ok(map)
  }
  fn append_sign(&mut self, sign: Sign) -> OssResult<Option<HeaderValue>>{
      let res = self.insert("Authorization", sign.try_into()?);
      Ok(res)
  }
}

/// 前缀是 x-oss- 的 header 记录
/// 
/// 将他们按顺序组合成一个字符串，用于计算签名
/// TODO String 可以改成 Cow
pub struct OssHeader(Option<String>);

impl OssHeader {
    fn to_sign_string(&self) -> String {
      let mut content = String::new();
      match self.0.clone() {
          Some(str) => {
            content.push_str(&str);
            content.push_str("\n");
          },
          None => (),
      }
      content
    }
}

/// 待签名的数据
pub struct SignString{
    data: String,
    key: KeyId,
    secret: KeySecret,
}

impl SignString {
  pub fn new(auth: &Auth, oss_header: &OssHeader) -> OssResult<SignString> {
    let method = auth.verb.to_string();

    let str: String = method
      + "\n"
      + match auth.content_md5.as_ref() {
        Some(str)=> {
          str.as_ref()
        },
        None => ""
      }
      + "\n"
      + match &auth.content_type {
        Some(str) => {
          str.as_ref()
        },
        None => ""
      }
      + "\n"
      + auth.date.as_ref() 
      + "\n"
      + oss_header.to_sign_string().as_ref()
      + auth.canonicalized_resource.as_ref();
    
    Ok(SignString{
      data: str,
      key: auth.access_key_id.clone(),
      secret: auth.access_key_secret.clone(),
    })
  }

  // 转化成签名
  fn to_sign(&self) -> OssResult<Sign> {
    use base64::encode;
    use sha1::Sha1;
    use hmac::{Hmac, Mac};
    type HmacSha1 = Hmac<Sha1>;

    let secret = self.secret.as_bytes();
    let data_u8 = self.data.as_bytes();
    
    let mut mac = HmacSha1::new_from_slice(secret)?;

    mac.update(data_u8);

    let sha1 = mac.finalize().into_bytes();

    Ok(Sign{
      data: encode(sha1),
      key: self.key.clone(),
    })
  }
}

/// header 中的签名
pub struct Sign{
    data: String,
    key: KeyId,
}

impl TryInto<HeaderValue> for Sign {
    type Error = OssError;

    /// 转化成 header 中需要的格式
    fn try_into(self) -> OssResult<HeaderValue> {
        let sign = format!("OSS {}:{}", self.key, self.data);
        Ok(sign.parse()?)
    }
}