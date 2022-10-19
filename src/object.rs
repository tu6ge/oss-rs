use chrono::prelude::*;
use reqwest::Response;

use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;
use std::{io::Read};

use reqwest::header::{HeaderMap,HeaderValue};
use crate::config::{ObjectBase, BucketBase};
use crate::errors::{OssResult,OssError};
use crate::client::{Client};
use crate::auth::{VERB};
use crate::traits::{ OssIntoObject, InvalidObjectValue, OssIntoObjectList, InvalidObjectListValue};
use crate::types::{Query, UrlQuery, CanonicalizedResource};

#[derive(Clone,Default)]
#[non_exhaustive]
pub struct ObjectList {
    bucket: BucketBase,
    name: String,
    prefix: String,
    max_keys: u32,
    key_count: u64,
    pub object_list: Vec<Object>,
    next_continuation_token: Option<String>,
    client: Arc<Client>,
    search_query: Query,
}

impl fmt::Debug for ObjectList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ObjectList")
          .field("name", &self.name)
          .field("bucket", &self.bucket)
          .field("prefix", &self.prefix)
          .field("max_keys", &self.max_keys)
          .field("key_count", &self.key_count)
          .field("next_continuation_token", &self.next_continuation_token)
          .field("search_query", &self.search_query)
          .finish()
    }
}

impl ObjectList {
    pub fn set_client(mut self, client: Arc<Client>) -> Self{
        self.client = client;
        self
    }

    pub fn client(&self) -> Arc<Client>{
        Arc::clone(&self.client)
    }

    pub fn set_search_query(mut self, search_query: Query) -> Self{
        self.search_query = search_query;
        self
    }

    pub fn set_bucket(mut self, bucket: BucketBase) -> Self{
        self.bucket = bucket;
        self
    }

    pub fn bucket_name(&self) -> &str{
        self.bucket.name()
    }

    pub fn len(&self) -> usize{
        self.object_list.len()
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Object {
    base: ObjectBase,
    key: String,
    last_modified: DateTime<Utc>,
    etag: String,
    _type: String,
    size: u64,
    storage_class: String,
}

impl Default for Object {
    fn default() -> Self {
        Object {
            base: ObjectBase::default(),
            last_modified: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
            key: String::default(),
            etag: String::default(),
            _type: String::default(),
            size: 0,
            storage_class: String::default(),
        }
    }
}

impl OssIntoObject for Object {
    fn set_bucket(mut self, bucket: BucketBase) -> Self {
        self.base.set_bucket(bucket);
        self
    }

    fn set_key(mut self, key: String) -> Result<Self, InvalidObjectValue> {
        self.key = key.clone();
        self.base.set_path(key);
        Ok(self)
    }
    fn set_last_modified(mut self, value: String) -> Result<Self, InvalidObjectValue> {
        let last_modified = value.parse::<DateTime<Utc>>().map_err(|_|InvalidObjectValue{})?;
        self.last_modified = last_modified;
        Ok(self)
    }
    fn set_etag(mut self, value: String) -> Result<Self, InvalidObjectValue> {
        self.etag = value;
        Ok(self)
    }
    fn set_type(mut self, value: String) -> Result<Self, InvalidObjectValue> {
        self._type = value;
        Ok(self)
    }
    fn set_size(mut self, size: String) -> Result<Self, InvalidObjectValue> {
        self.size = size.parse::<u64>().map_err(|_|InvalidObjectValue{})?;
        Ok(self)
    }

    fn set_storage_class(mut self, value: String) -> Result<Self, InvalidObjectValue> {
        self.storage_class = value;
        Ok(self)
    }
}

impl OssIntoObjectList<Object> for ObjectList{
    fn set_key_count(mut self, key_count: String) -> Result<Self, InvalidObjectListValue> {
        self.key_count = key_count.parse::<u64>().map_err(|_|InvalidObjectListValue{})?;
        Ok(self)
    }

    fn set_name(mut self, name: String) -> Result<Self, InvalidObjectListValue> {
        self.name = name;
        Ok(self)
    }

    fn set_prefix(mut self, prefix: String) -> Result<Self, InvalidObjectListValue> {
        self.prefix = prefix;
        Ok(self)
    }

    fn set_max_keys(mut self, max_keys: String) -> Result<Self, InvalidObjectListValue> {
        self.max_keys = max_keys.parse::<u32>().map_err(|_|InvalidObjectListValue{})?;
        Ok(self)
    }

    fn set_next_continuation_token(mut self, token: Option<String>) -> Result<Self, InvalidObjectListValue> {
        self.next_continuation_token = token;
        Ok(self)
    }

    fn set_list(mut self, list: Vec<Object>) -> Result<Self, InvalidObjectListValue> {
        self.object_list = list;
        Ok(self)
    }
}

impl Client {

  pub async fn get_object_list(self, query: Query) -> OssResult<ObjectList>{
    let mut url = self.get_bucket_url();

    url.set_search_query(&query);

    let bucket = self.get_bucket_base();

    let canonicalized = CanonicalizedResource::from_bucket_query(&bucket, &query);

    let response = self.builder(VERB::GET, &url, canonicalized)?;
    let content = response.send().await?;

    let list = ObjectList::default().set_client(Arc::new(self))
        .set_bucket(bucket.clone());

    Ok(
      list.from_xml(content.text().await?, &bucket)?.set_search_query(query)
    )
  }

  pub async fn put_file<P: Into<PathBuf> + std::convert::AsRef<std::path::Path>>(&self, file_name: P, key: &'static str) -> OssResult<String> {
    let mut file_content = Vec::new();
    std::fs::File::open(file_name)?
      .read_to_end(&mut file_content)?;

    self.put_content(file_content, key).await
  }

  pub async fn put_content(&self, content: Vec<u8>, key: &str) -> OssResult<String>{
    let kind = self.infer.get(&content);

    let con = match kind {
      Some(con) => {
        Ok(con)
      },
      None => Err(OssError::Input("file type is known".to_string()))
    };

    let content_type = con?.mime_type();

    let content = self.put_content_base(content, content_type, key).await?;

    let result = content.headers().get("ETag")
      .ok_or(OssError::Input("get Etag error".to_string()))?
      .to_str().map_err(|_| OssError::Input("ETag parse error".to_string()))?;

    Ok(result.to_string())
  }

  /// 最原始的上传文件的方法
  pub async fn put_content_base(&self, content: Vec<u8>, content_type: &str, key: &str) -> OssResult<Response>{
    let mut url = self.get_bucket_url();
    url.set_path(key);

    let mut headers = HeaderMap::new();
    let content_length = content.len().to_string();
    headers.insert(
      "Content-Length", 
      HeaderValue::from_str(&content_length).map_err(|_| OssError::Input("Content-Length parse error".to_string()))?
    );

    headers.insert(
      "Content-Type", 
      content_type.parse().map_err(|_| OssError::Input("Content-Type parse error".to_string()))?
    );

    let object_base = ObjectBase::new(self.get_bucket_base(), key.to_owned());
  
    let canonicalized = CanonicalizedResource::from_object(&object_base, None);

    let response = self.builder_with_header(VERB::PUT, &url, canonicalized, Some(headers))?
      .body(content);

    let content = response.send().await?;
    Ok(content)
  }

  pub async fn delete_object(&self, key: &str) -> OssResult<()>{
    let mut url = self.get_bucket_url();
    url.set_path(key);

    let object_base = ObjectBase::new(self.get_bucket_base(), key.to_owned());
    
    let canonicalized = CanonicalizedResource::from_object(&object_base, None);

    let response = self.builder(VERB::DELETE, &url, canonicalized)?;

    response.send().await?;
    
    Ok(())
  }
}

// use futures::Stream;
// use std::iter::Iterator;
// use std::pin::{Pin};
// use std::task::Poll;
// impl <'a>Stream for ObjectList<'a> {
//   type Item = Vec<Object>;
  

//   /// 未测试的
//   fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> core::task::Poll<Option<Vec<Object>>> {
    

//     if let None = self.next_continuation_token {
//       return Poll::Ready(None);
//     }
    
//     let mut pinned = pin!(self.next_continuation_token);
//     match pinned.as_mut().poll(cx) {
//       Poll::Ready(token) => {
//         let mut query = self.search_query.clone();
//         query.insert("continuation-token".to_string(), token);
//         match self.client.get_object_list(query) {
//           Ok(list) => core::task::Poll::Ready(Some(list)),
//           Err(_) => core::task::Poll::Ready(None),
//         }
//       },
//       Poll::Pending => Poll::Pending
//     }
//   }
// }

#[derive(Default)]
pub struct PutObject<'a>{
  pub forbid_overwrite: bool,
  pub server_side_encryption: Option<Encryption>,
  pub server_side_data_encryption: Option<Encryption>,
  pub server_side_encryption_key_id: Option<&'a str>,
  pub object_acl: ObjectAcl,
  pub storage_class: StorageClass,
  pub tagging: Option<&'a str>,
}

pub enum Encryption{
  Aes256,
  Kms,
  Sm4
}

impl Default for Encryption{
  fn default() -> Encryption{
    Self::Aes256
  }
}

pub enum ObjectAcl{
  Default,
  Private,
  PublicRead,
  PublicReadWrite,
}

impl Default for ObjectAcl {
  fn default() -> Self {
    Self::Default
  }
}

pub enum StorageClass{
  Standard,
  IA,
  Archive,
  ColdArchive,
}

impl Default for StorageClass {
  fn default() -> Self {
    Self::Standard
  }
}

#[derive(Default)]
pub struct CopyObject<'a>{
  pub forbid_overwrite: bool,
  pub copy_source: &'a str,
  pub copy_source_if_match: Option<&'a str>,
  pub copy_source_if_none_match: Option<&'a str>,
  pub copy_source_if_unmodified_since: Option<&'a str>,
  pub copy_source_if_modified_since: Option<&'a str>,
  pub metadata_directive: CopyDirective,
  pub server_side_encryption: Option<Encryption>,
  pub server_side_encryption_key_id: Option<&'a str>,
  pub object_acl: ObjectAcl,
  pub storage_class: StorageClass,
  pub tagging: Option<&'a str>,
  pub tagging_directive: CopyDirective,
}

pub enum CopyDirective {
  Copy,
  Replace,
}

impl Default for CopyDirective{
  fn default() -> Self{
    Self::Copy
  }
}