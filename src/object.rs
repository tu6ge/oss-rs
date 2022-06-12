use chrono::prelude::*;
use quick_xml::{events::Event, Reader};
use std::collections::HashMap;
use std::io::Read;
use reqwest::header::{HeaderMap,HeaderValue};

use crate::errors::{OssResult,OssError, self};
use crate::client::{Client, OssObject, ReqeustHandler};
use crate::auth::{self, VERB};

#[macro_use]
use anyhow::anyhow;

#[derive(Clone, Debug)]
pub struct ObjectList {
  pub name: String,
  pub prefix: String,
  pub max_keys: u32,
  pub key_count: u64,
  pub object_list: Vec<Object>,
  pub next_continuation_token: Option<String>,
}

impl ObjectList {
  pub fn new(
    name: String,
    prefix: String,
    max_keys: u32,
    key_count: u64,
    object_list: Vec<Object>,
    next_continuation_token: Option<String>
  ) ->Self {
    ObjectList {
      name,
      prefix,
      max_keys,
      key_count,
      object_list,
      next_continuation_token,
    }
  }
}

impl OssObject for ObjectList {
  
  fn from_xml(xml: String) -> OssResult<ObjectList> {
    let mut result = Vec::new();
    let mut reader = Reader::from_str(xml.as_str());
    reader.trim_text(true);
    let mut buf = Vec::with_capacity(xml.len());
    let mut skip_buf = Vec::with_capacity(xml.len());

    let mut key = String::new();
    let mut last_modified = String::with_capacity(20);
    let mut _type = String::new();
    let mut etag = String::with_capacity(34); // 32 位 加两位 "" 符号
    let mut size: u64 = 0;
    let mut storage_class = String::with_capacity(11);
    // let mut is_truncated = false;

    let mut name = String::new();
    let mut prefix = String::new();
    let mut max_keys: u32 = 0;
    let mut key_count: u64 = 0;
    let mut next_continuation_token: Option<String> = None;

    let list_object;

    loop {
      match reader.read_event(&mut buf) {
          Ok(Event::Start(ref e)) => match e.name() {
              b"Prefix" => prefix = reader.read_text(e.name(), &mut skip_buf)?,
              b"Name" => name = reader.read_text(e.name(), &mut skip_buf)?,
              b"MaxKeys" => {
                max_keys = reader.read_text(e.name(), &mut skip_buf)?.parse::<u32>()?;
              },
              b"KeyCount" => {
                key_count = reader.read_text(e.name(), &mut skip_buf)?.parse::<u64>()?;
              },
              b"IsTruncated" => {
                //is_truncated = reader.read_text(e.name(), &mut skip_buf)? == "true"
              }
              b"NextContinuationToken" => {
                next_continuation_token = Some(reader.read_text(e.name(), &mut skip_buf)?);
              }
              b"Contents" => {
                key.clear();
                last_modified.clear();
                etag.clear();
                _type.clear();
                storage_class.clear();
              }

              b"Key" => key = reader.read_text(e.name(), &mut skip_buf)?,
              b"LastModified" => last_modified = reader.read_text(e.name(), &mut skip_buf)?,
              b"ETag" => {
                etag = reader.read_text(e.name(), &mut skip_buf)?;
                let str = "\"";
                etag = etag.replace(str, "");
              }
              b"Type" => {
                _type = reader.read_text(e.name(), &mut skip_buf)?
              }
              b"Size" => {
                size = reader.read_text(e.name(), &mut skip_buf)?.parse::<u64>()?;
              },
              b"StorageClass" => {
                storage_class = reader.read_text(e.name(), &mut skip_buf)?
              }
              _ => (),
          },
          Ok(Event::End(ref e)) if e.name() == b"Contents" => {
            let in_last_modified = &last_modified.parse::<DateTime<Utc>>()?;
            let object = Object::new(
                key.clone(),
                in_last_modified.clone(),
                etag.clone(),
                _type.clone(),
                size,
                storage_class.clone(),
            );
            result.push(object);
          }
          Ok(Event::Eof) => {
              list_object = ObjectList::new(
                  Client::string2option(name).ok_or(OssError::Input("get name failed by xml".to_string()))?,
                  prefix,
                  max_keys,
                  key_count,
                  result,
                  next_continuation_token,
              );
              break;
          } // exits the loop when reaching end of file
          Err(e) => {
            return Err(OssError::Input(format!("Error at position {}: {:?}", reader.buffer_position(), e)));
          },
          _ => (), // There are several other `Event`s we do not consider here
      }
      buf.clear();
    }


    Ok(list_object)
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

impl Object {
  pub fn new(
    key: String,
    last_modified: DateTime<Utc>,
    etag: String,
    _type: String,
    size: u64,
    storage_class: String
  ) -> Object {
    Object {
      key,
      last_modified,
      etag,
      _type,
      size,
      storage_class
    }
  }
}

impl <'a> Client<'a> {

  /// # 获取存储对象列表
  /// 使用的 v2 版本 API
  /// query 参数请参考 OSS 文档，注意 `list-type` 参数已固定为 `2` ，无需传
  /// 
  /// [OSS 文档](https://help.aliyun.com/document_detail/187544.html)
  pub fn get_object_list(&self, query: HashMap<String, String>) -> OssResult<ObjectList>{

    let mut url = self.get_bucket_url()?;

    let mut query_str = String::new();
    for (key,value) in query.iter() {
      query_str += "&";
      query_str += key;
      query_str += "=";
      query_str += value;
    }
    let query_str = "list-type=2".to_owned() + &query_str;

    url.set_query(Some(&query_str));

    let response = self.builder(VERB::GET, &url, None)?;
    let content = response.send()?.handle_error()?;

    // println!("{}", &content.text()?);
    // return Err(errors::OssError::Other(anyhow!("abc")));

    ObjectList::from_xml(content.text()?)
  }

  /// # 上传文件到 OSS 中
  /// 
  /// 提供有效的文件路径即可
  pub fn put_file(&self, file_name: &'a str, key: &'a str) -> OssResult<String> {
    let mut file_content = Vec::new();
    std::fs::File::open(file_name)?
      .read_to_end(&mut file_content)?;

    self.put_content(&file_content, key)
  }

  /// # 上传文件内容到 OSS
  /// 
  /// 需要事先读取文件内容到 `Vec<u8>` 中
  /// 
  /// 并提供存储的 key 
  pub fn put_content(&self, content: &Vec<u8>, key: &str) -> OssResult<String>{
    let mime_type = infer::get(content)
      .expect("file read successfully")
      .mime_type();

    let mut url = self.get_bucket_url()?;
    url.set_path(key);

    let mut headers = HeaderMap::new();
    let content_length = content.len().to_string();
    headers.insert("Content-Length", HeaderValue::from_str(&content_length)?);

    headers.insert(auth::to_name("Content-Type")?, mime_type.parse()?);
    let response = self.builder(VERB::PUT, &url, Some(headers))?
      .body(content.clone());

    let content = response.send()?.handle_error()?;

    let result = content.headers().get("ETag")
      .ok_or(OssError::Input("get Etag error".to_string()))?
      .to_str()?;

    Ok(result.to_string())
  }

  /// # 删除文件
  pub fn delete_object(&self, key: &str) -> OssResult<()>{
    let mut url = self.get_bucket_url()?;
    url.set_path(key);

    let response = self.builder(VERB::DELETE, &url, None)?;

    response.send()?.handle_error()?;
    
    Ok(())
  }
}


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