use reqwest::Url;
use chrono::prelude::*;
use quick_xml::{events::Event, Reader};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;
use reqwest::header::{HeaderMap,HeaderValue};

use crate::client::{Client, OssObject, Result};
use crate::auth::{self, VERB};

#[derive(Clone, Debug)]
pub struct ObjectList {
  pub name: String,
  pub prefix: String,
  pub max_keys: u32,
  pub key_count: u64,
  pub object_list: Vec<Object>,
}

impl ObjectList {
  pub fn new(name: String, prefix: String, max_keys: u32, key_count: u64, object_list: Vec<Object>) ->Self {
    ObjectList {
      name,
      prefix,
      max_keys,
      key_count,
      object_list
    }
  }
}

impl OssObject for ObjectList {
  
  fn from_xml(xml: String) -> Result<ObjectList> {
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
                  Client::string2option(name).unwrap(),
                  Client::string2option(prefix).unwrap_or("".to_owned()),
                  max_keys,
                  key_count,
                  result,
              );
              break;
          } // exits the loop when reaching end of file
          Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
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
  pub fn get_object_list(&self) -> Result<ObjectList>{

    let mut url = self.get_bucket_url().unwrap();
    url.set_query(Some("list-type=2"));

    let response = self.builder(VERB::GET, &url, None);
    //println!("get_bucket_list {}", response.send().unwrap().text().unwrap());
    let mut content = response.send().expect(Client::ERROR_REQUEST_ALIYUN_API);

    Client::handle_error(&mut content);
    //println!("get_bucket_list: {}", content.text().unwrap());

    ObjectList::from_xml(content.text().unwrap())
  }

  /// # 上传文件到 OSS 中
  /// 
  /// 提供有效的文件路径即可
  pub fn put_file(&self, file_name: &'a str, key: &'a str) -> Result<String> {
    let mut file_content = Vec::new();
    std::fs::File::open(file_name)
      .expect("open file failed").read_to_end(&mut file_content)
      .expect("read_to_end failed");

    self.put_content(&file_content, key)
  }

  /// # 上传文件内容到 OSS
  /// 
  /// 需要事先读取文件内容到 `Vec<u8>` 中
  /// 
  /// 并提供存储的 key 
  pub fn put_content(&self, content: &Vec<u8>, key: &str) -> Result<String>{
    let mime_type = infer::get(content)
      .expect("file read successfully")
      .mime_type();

    let mut url = self.get_bucket_url().unwrap();
    url.set_path(key);

    let mut headers = HeaderMap::new();
    let content_length = content.len().to_string();
    headers.insert("Content-Length", HeaderValue::from_str(&content_length).unwrap());

    headers.insert(auth::to_name("Content-Type"), mime_type.parse().unwrap());
    let response = self.builder(VERB::PUT, &url, Some(headers))
      .body(content.clone());

    let mut content = response.send().expect(Client::ERROR_REQUEST_ALIYUN_API);

    // let text = content.text().unwrap().clone();
    // println!("{}", text);
    // return Ok(text);

    Client::handle_error(&mut content);

    let result = content.headers().get("ETag").unwrap().to_str().unwrap();

    Ok(result.to_string())
  }

  /// # 删除文件
  pub fn delete_object(&self, key: &str) -> Result<()>{
    let mut url = self.get_bucket_url().unwrap();
    url.set_path(key);

    let response = self.builder(VERB::DELETE, &url, None);

    let mut content = response.send().expect(Client::ERROR_REQUEST_ALIYUN_API);

    // let text = content.text().unwrap().clone();
    // println!("{}", text);
    // return Ok(text);

    Client::handle_error(&mut content);
    
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