use chrono::prelude::*;
use futures::Stream;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::{io::Read, iter::Iterator};
use reqwest::header::{HeaderMap,HeaderValue};

use crate::errors::{OssResult,OssError};
use crate::client::{Client, ReqeustHandler};
use crate::auth::{VERB};
use crate::traits::{ObjectTrait, ObjectListTrait};

#[derive(Clone)]
pub struct ObjectList<'a> {
  pub name: String,
  pub prefix: String,
  pub max_keys: u32,
  pub key_count: u64,
  pub object_list: Vec<Object>,
  pub next_continuation_token: Option<String>,
  client: Option<&'a Client<'a>>,
  pub search_query: Option<HashMap<String, String>>,
}

impl fmt::Debug for ObjectList<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("ObjectList")
      .field("name", &self.name)
      .field("prefix", &self.prefix)
      .field("max_keys", &self.max_keys)
      .field("key_count", &self.key_count)
      .field("next_continuation_token", &self.next_continuation_token)
      .finish()
  }
}

impl ObjectListTrait<Object> for ObjectList<'_> {
  fn from_oss(
    name: String,
    prefix: String,
    max_keys: String,
    key_count: String,
    object_list: Vec<Object>,
    next_continuation_token: Option<String>,
    // client: Option<&'a Client>,
    // search_query: Option<HashMap<String, String>>
  ) ->OssResult<ObjectList<'static>>{
    let in_max_keys = max_keys.parse::<u32>()?;
    let in_key_count = key_count.parse::<u64>()?;
    Ok(ObjectList{
      name,
      prefix,
      max_keys: in_max_keys,
      key_count: in_key_count,
      object_list,
      next_continuation_token,
      client: None,
      search_query: None,
    })
  }
}
impl<'b> ObjectList<'b> {
  
  pub fn set_client(mut self, client: &'b Client) -> Self{
    self.client = Some(client);
    self
  }
  
  pub fn set_search_query(mut self, search_query: HashMap<String, String>) -> Self{
    self.search_query = Some(search_query);
    self
  }
}

#[derive(Clone, Debug)]
pub struct Object {
  pub key: String,
  pub last_modified: DateTime<Utc>,
  pub etag: String,
  pub _type: String,
  pub size: u64,
  pub storage_class: String,
}

impl ObjectTrait for Object {
  fn from_oss(
    key: String,
    last_modified: String,
    etag: String,
    _type: String,
    size: String,
    storage_class: String
  ) -> OssResult<Object> {
    let in_last_modified = last_modified.parse::<DateTime<Utc>>()?;
    let in_size = size.parse::<u64>()?;
    Ok(Object {
      key,
      last_modified: in_last_modified,
      etag,
      _type,
      size: in_size,
      storage_class
    })
  }
}

impl <'a> Client<'a> {

  /// # 获取存储对象列表
  /// 使用的 v2 版本 API
  /// query 参数请参考 OSS 文档，注意 `list-type` 参数已固定为 `2` ，无需传
  /// 
  /// [OSS 文档](https://help.aliyun.com/document_detail/187544.html)
  #[cfg(feature = "blocking")]
  pub fn blocking_get_object_list(&self, query: HashMap<String, String>) -> OssResult<ObjectList<'_>>{
    let mut url = self.get_bucket_url()?;

    let query_str = Client::<'a>::object_list_query_generator(&query);

    url.set_query(Some(&query_str));

    let response = self.blocking_builder(VERB::GET, &url, None, None)?;
    let content = response.send()?.handle_error()?;

    Ok(
      ObjectList::from_xml(content.text()?)?.set_client(&self).set_search_query(query)
    )
  }

  pub async fn get_object_list(&self, query: HashMap<String, String>) -> OssResult<ObjectList<'_>>{

    let mut url = self.get_bucket_url()?;

    let query_str = Client::<'a>::object_list_query_generator(&query);

    url.set_query(Some(&query_str));

    let response = self.builder(VERB::GET, &url, None, None).await?;
    let content = response.send().await?.handle_error()?;

    Ok(
      ObjectList::from_xml(content.text().await?)?.set_client(&self).set_search_query(query)
    )
  }

  /// # 上传文件到 OSS 中
  /// 
  /// 提供有效的文件路径即可
  #[cfg(feature = "blocking")]
  pub fn blocking_put_file(&self, file_name: PathBuf, key: &'a str) -> OssResult<String> {
    let mut file_content = Vec::new();
    std::fs::File::open(file_name)?
      .read_to_end(&mut file_content)?;

    self.blocking_put_content(&file_content, key)
  }

  pub async fn put_file(&self, file_name: PathBuf, key: &'a str) -> OssResult<String> {
    let mut file_content = Vec::new();
    std::fs::File::open(file_name)?
      .read_to_end(&mut file_content)?;

    self.put_content(&file_content, key).await
  }

  /// # 上传文件内容到 OSS
  /// 
  /// 需要事先读取文件内容到 `Vec<u8>` 中
  /// 
  /// 并提供存储的 key 
  #[cfg(feature = "blocking")]
  pub fn blocking_put_content(&self, content: &Vec<u8>, key: &str) -> OssResult<String>{
    let kind = infer::get(content);

    let con = match kind {
      Some(con) => {
        Ok(con)
      },
      None => Err(OssError::Input("file type is known".to_string()))
    };

    let mime_type = con?.mime_type();

    let mut url = self.get_bucket_url()?;
    url.set_path(key);

    let mut headers = HeaderMap::new();
    let content_length = content.len().to_string();
    headers.insert(
      "Content-Length", 
      HeaderValue::from_str(&content_length).map_err(|_| OssError::Input("Content-Length parse error".to_string()))?);

    headers.insert(
      "Content-Type", 
      mime_type.parse().map_err(|_| OssError::Input("Content-Type parse error".to_string()))?);
    let response = self.blocking_builder(VERB::PUT, &url, Some(headers), None)?
      .body(content.clone());

    let content = response.send()?.handle_error()?;

    let result = content.headers().get("ETag")
      .ok_or(OssError::Input("get Etag error".to_string()))?
      .to_str().map_err(|_| OssError::Input("ETag parse error".to_string()))?;

    Ok(result.to_string())
  }

  pub async fn put_content(&self, content: &Vec<u8>, key: &str) -> OssResult<String>{
    let kind = infer::get(content);

    let con = match kind {
      Some(con) => {
        Ok(con)
      },
      None => Err(OssError::Input("file type is known".to_string()))
    };

    let mime_type = con?.mime_type();

    let mut url = self.get_bucket_url()?;
    url.set_path(key);

    let mut headers = HeaderMap::new();
    let content_length = content.len().to_string();
    headers.insert(
      "Content-Length", 
      HeaderValue::from_str(&content_length).map_err(|_| OssError::Input("Content-Length parse error".to_string()))?);

    headers.insert(
      "Content-Type", 
      mime_type.parse().map_err(|_| OssError::Input("Content-Type parse error".to_string()))?);
    let response = self.builder(VERB::PUT, &url, Some(headers), None).await?
      .body(content.clone());

    let content = response.send().await?.handle_error()?;

    let result = content.headers().get("ETag")
      .ok_or(OssError::Input("get Etag error".to_string()))?
      .to_str().map_err(|_| OssError::Input("ETag parse error".to_string()))?;

    Ok(result.to_string())
  }

  /// # 删除文件
  #[cfg(feature = "blocking")]
  pub fn blocking_delete_object(&self, key: &str) -> OssResult<()>{
    let mut url = self.get_bucket_url()?;
    url.set_path(key);

    let response = self.blocking_builder(VERB::DELETE, &url, None, None)?;

    response.send()?.handle_error()?;
    
    Ok(())
  }

  pub async fn delete_object(&self, key: &str) -> OssResult<()>{
    let mut url = self.get_bucket_url()?;
    url.set_path(key);

    let response = self.builder(VERB::DELETE, &url, None, None).await?;

    response.send().await?.handle_error()?;
    
    Ok(())
  }
}

#[cfg(feature = "blocking")]
impl <'a>Iterator for ObjectList<'a>{
  type Item = ObjectList<'a>;
  fn next(&mut self) -> Option<ObjectList<'a>> {
    match self.next_continuation_token.clone() {
      Some(token) => {
        let mut query = self.search_query.as_ref().unwrap().clone();
        query.insert("continuation-token".to_string(), token);
        match self.client.unwrap().blocking_get_object_list(query) {
          Ok(list) => Some(list),
          Err(_) => None,
        }
      },
      None => {
        return None
      }
    }
  }
}


// impl <'a>Stream for ObjectList<'a> {
//   type Item = ObjectList<'a>;

//   /// 未测试的
//   fn poll_next(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> core::task::Poll<Option<ObjectList<'a>>> {
//     match self.next_continuation_token.clone() {
//       Some(token) => {
//         let mut query = self.search_query.clone();
//         query.insert("continuation-token".to_string(), token);
//         match self.client.get_object_list(query) {
//           Ok(list) => core::task::Poll::Ready(Some(list)),
//           Err(_) => core::task::Poll::Ready(None),
//         }
//       },
//       None => {
//         core::task::Poll::Ready(None)
//       }
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