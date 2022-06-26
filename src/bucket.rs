
use std::collections::HashMap;
use std::fmt;

use crate::client::{Client, ReqeustHandler};
use crate::auth::VERB;
use crate::errors::{OssResult,OssError};
use crate::object::ObjectList;
use chrono::prelude::*;
use reqwest::Url;

use quick_xml::{events::Event, Reader};

#[derive(Clone)]
pub struct ListBuckets<'a> {
  pub prefix: Option<String>,
  pub marker: Option<String>,
  pub max_keys: Option<String>,
  pub is_truncated: bool,
  pub next_marker: Option<String>,
  pub id: Option<String>,
  pub display_name: Option<String>,
  pub buckets: Vec<Bucket<'a>>,
}

impl fmt::Debug for ListBuckets<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
    f.debug_struct("ListBuckets")
      .field("prefix", &self.prefix)
      .field("marker", &self.marker)
      .field("max_keys", &self.max_keys)
      .field("is_truncated", &self.is_truncated)
      .field("next_marker", &self.next_marker)
      .field("id", &self.id)
      .field("display_name", &self.display_name)
      .field("buckets", &"bucket list")
      .finish()
  }
}

impl ListBuckets<'_> {
  pub fn new(
    prefix: Option<String>, 
    marker: Option<String>,
    max_keys: Option<String>,
    is_truncated: bool,
    next_marker: Option<String>,
    id: Option<String>,
    display_name: Option<String>,
    buckets: Vec<Bucket>,
  ) -> ListBuckets {
    ListBuckets {
      prefix,
      marker,
      max_keys,
      is_truncated,
      next_marker,
      id,
      display_name,
      buckets
    }
  }
}

impl ListBuckets<'_>  {

  fn from_xml<'a>(xml: String, client: &'a Client) -> OssResult<ListBuckets<'a>> {
    let mut result = Vec::new();
    let mut reader = Reader::from_str(xml.as_str());
    reader.trim_text(true);
    let mut buf = Vec::with_capacity(xml.len());
    let mut skip_buf = Vec::with_capacity(xml.len());

    let mut prefix = String::new();
    let mut marker = String::new();
    let mut max_keys = String::new();
    let mut is_truncated = false;
    let mut next_marker = String::new();
    let mut id = String::with_capacity(8);
    let mut display_name = String::with_capacity(8);

    let mut name = String::new();
    let mut location = String::new();
    let mut creation_date = String::with_capacity(20);
    
    // 目前最长的可用区 zhangjiakou 13 ，剩余部分总共 20 
    let mut extranet_endpoint = String::with_capacity(33);
    // 上一个长度 + 9 （-internal）
    let mut intranet_endpoint = String::with_capacity(42);
    // 最长的值 ColdArchive 11
    let mut storage_class = String::with_capacity(11);

    let list_buckets;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name() {
                b"Prefix" => prefix = reader.read_text(e.name(), &mut skip_buf)?,
                b"Marker" => marker = reader.read_text(e.name(), &mut skip_buf)?,
                b"MaxKeys" => max_keys = reader.read_text(e.name(), &mut skip_buf)?,
                b"IsTruncated" => {
                    is_truncated = reader.read_text(e.name(), &mut skip_buf)? == "true"
                }
                b"NextMarker" => next_marker = reader.read_text(e.name(), &mut skip_buf)?,
                b"ID" => id = reader.read_text(e.name(), &mut skip_buf)?,
                b"DisplayName" => display_name = reader.read_text(e.name(), &mut skip_buf)?,

                b"Bucket" => {
                    name.clear();
                    location.clear();
                    creation_date.clear();
                    extranet_endpoint.clear();
                    intranet_endpoint.clear();
                    storage_class.clear();
                }

                b"Name" => name = reader.read_text(e.name(), &mut skip_buf)?,
                b"CreationDate" => creation_date = reader.read_text(e.name(), &mut skip_buf)?,
                b"ExtranetEndpoint" => {
                    extranet_endpoint = reader.read_text(e.name(), &mut skip_buf)?
                }
                b"IntranetEndpoint" => {
                    intranet_endpoint = reader.read_text(e.name(), &mut skip_buf)?
                }
                b"Location" => location = reader.read_text(e.name(), &mut skip_buf)?,
                b"StorageClass" => {
                    storage_class = reader.read_text(e.name(), &mut skip_buf)?
                }
                _ => (),
            },
            Ok(Event::End(ref e)) if e.name() == b"Bucket" => {
              let in_creation_date = &creation_date.parse::<DateTime<Utc>>()?;
              let bucket = Bucket::new(
                  name.clone(),
                  in_creation_date.clone(),
                  location.clone(),
                  extranet_endpoint.clone(),
                  intranet_endpoint.clone(),
                  storage_class.clone(),
                  client,
              );
              result.push(bucket);
            }
            Ok(Event::Eof) => {
                list_buckets = ListBuckets::new(
                    Client::string2option(prefix),
                    Client::string2option(marker),
                    Client::string2option(max_keys),
                    is_truncated,
                    Client::string2option(next_marker),
                    Client::string2option(id),
                    Client::string2option(display_name),
                    result,
                );
                break;
            } // exits the loop when reaching end of file
            Err(e) => {
              return Err(OssError::Input(format!("Error at position {}: {:?}", reader.buffer_position(), e)))
            },
            _ => (), // There are several other `Event`s we do not consider here
        }
        buf.clear();
    }
    Ok(list_buckets)
  }
}



#[derive(Clone)]
pub struct Bucket<'a>{
  // bucket_info: Option<Bucket<'b>>,
  // bucket: Option<Bucket<'c>>,
  pub creation_date: DateTime<Utc>,
  pub extranet_endpoint: String,
  pub intranet_endpoint: String,
  pub location: String,
  pub name: String,
  // owner 	存放Bucket拥有者信息的容器。父节点：BucketInfo.Bucket
  // access_control_list;
  // pub grant: Grant,
  // pub data_redundancy_type: Option<DataRedundancyType>,
  pub storage_class: String,
  // pub versioning: &'a str,
  // ServerSideEncryptionRule,
  // ApplyServerSideEncryptionByDefault,
  // pub sse_algorithm: &'a str,
  // pub kms_master_key_id: Option<&'a str>,
  // pub cross_region_replication: &'a str,
  // pub transfer_acceleration: &'a str,
  client: &'a Client<'a>,
}

impl fmt::Debug for Bucket<'_> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Bucket")
      .field("creation_date", &self.creation_date)
      .field("extranet_endpoint", &self.extranet_endpoint)
      .field("intranet_endpoint", &self.intranet_endpoint)
      .field("location", &self.location)
      .field("name", &self.name)
      .field("storage_class", &self.storage_class)
      .finish()
  }
}

impl <'b> Bucket<'_> {
  pub fn new<'a>(
    name: String,
    creation_date: DateTime<Utc>,
    location: String,
    extranet_endpoint: String,
    intranet_endpoint: String,
    storage_class: String,
    client: &'a Client,
  ) -> Bucket<'a> {
    Bucket {
      name,
      creation_date,
      // data_redundancy_type: None,
      location,
      extranet_endpoint,
      intranet_endpoint,
      storage_class,
      client,
    }
  }

  #[cfg(feature = "blocking")]
  pub fn blocking_get_object_list(&self, query: HashMap<String, String>) -> OssResult<ObjectList>{
    let input = "https://".to_owned() + &self.name + "." + &self.extranet_endpoint;
    let mut url = Url::parse(&input).map_err(|_| OssError::Input("url parse error".to_string()))?;

    let query_str = Client::<'b>::object_list_query_generator(&query);

    url.set_query(Some(&query_str));

    let response = self.client.blocking_builder(VERB::GET, &url, None, Some(self.name.to_string()))?;
    let content = response.send()?.handle_error()?;
    ObjectList::from_xml(content.text()?, &self.client, query)
  }

  pub async fn get_object_list(&self, query: HashMap<String, String>) -> OssResult<ObjectList<'_>>{
    let input = "https://".to_owned() + &self.name + "." + &self.extranet_endpoint;
    let mut url = Url::parse(&input).map_err(|_| OssError::Input("url parse error".to_string()))?;

    let query_str = Client::<'b>::object_list_query_generator(&query);

    url.set_query(Some(&query_str));

    let response = self.client.builder(VERB::GET, &url, None, Some(self.name.to_string())).await?;
    let content = response.send().await?.handle_error()?;

    // println!("{}", &content.text()?);
    // return Err(errors::OssError::Other(anyhow!("abc")));

    ObjectList::from_xml(content.text().await?, &self.client, query)
  }

  fn from_xml<'a>(xml: String, client: &'a Client) -> OssResult<Bucket<'a>>{
    let mut reader = Reader::from_str(xml.as_str());
    reader.trim_text(true);
    let mut buf = Vec::with_capacity(xml.len());
    let mut skip_buf = Vec::with_capacity(xml.len());

    let mut name = String::new();
    let mut location = String::new();
    let mut creation_date = String::with_capacity(20);
    
    // 目前最长的可用区 zhangjiakou 13 ，剩余部分总共 20 
    let mut extranet_endpoint = String::with_capacity(33);
    // 上一个长度 + 9 （-internal）
    let mut intranet_endpoint = String::with_capacity(42);
    // 最长的值 ColdArchive 11
    let mut storage_class = String::with_capacity(11);

    let bucket;

    loop {
      match reader.read_event(&mut buf) {
          Ok(Event::Start(ref e)) => match e.name() {
              b"Name" => name = reader.read_text(e.name(), &mut skip_buf)?,
              b"CreationDate" => creation_date = reader.read_text(e.name(), &mut skip_buf)?,
              b"ExtranetEndpoint" => {
                  extranet_endpoint = reader.read_text(e.name(), &mut skip_buf)?
              }
              b"IntranetEndpoint" => {
                  intranet_endpoint = reader.read_text(e.name(), &mut skip_buf)?
              }
              b"Location" => location = reader.read_text(e.name(), &mut skip_buf)?,
              b"StorageClass" => {
                  storage_class = reader.read_text(e.name(), &mut skip_buf)?
              }
              _ => (),
          },
          Ok(Event::Eof) => {
            let in_creation_date = &creation_date.parse::<DateTime<Utc>>()?;
            bucket = Bucket::new(
              name.clone(),
              in_creation_date.clone(),
              location.clone(),
              extranet_endpoint.clone(),
              intranet_endpoint.clone(),
              storage_class.clone(),
              client,
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
    Ok(bucket)
  }
}


impl<'a> Client<'a> {

  /** # 获取 buiket 列表
  */
  #[cfg(feature = "blocking")]
  pub fn blocking_get_bucket_list(&self) -> OssResult<ListBuckets> {
    let url = Url::parse(&self.endpoint).map_err(|_| OssError::Input("endpoint url parse error".to_string()))?;
    //url.set_path(self.bucket)

    let response = self.blocking_builder(VERB::GET, &url, None, None)?;
    let content = response.send()?.handle_error()?;
    
    ListBuckets::from_xml(content.text()?, &self)
  }

  pub async fn get_bucket_list(&self) -> OssResult<ListBuckets<'_>>{
    let url = Url::parse(&self.endpoint).map_err(|_| OssError::Input("endpoint url parse error".to_string()))?;
    //url.set_path(self.bucket)

    let response = self.builder(VERB::GET, &url, None, None).await?;
    let content = response.send().await?.handle_error()?;
    
    ListBuckets::from_xml(content.text().await?, &self)
  }

  #[cfg(feature = "blocking")]
  pub fn blocking_get_bucket_info(&self) -> OssResult<Bucket> {
    let headers = None;
    let mut bucket_url = self.get_bucket_url()?;
    bucket_url.set_query(Some("bucketInfo"));

    let response = self.blocking_builder(VERB::GET, &bucket_url, headers, None)?;
    let content = response.send()?.handle_error()?;

    Bucket::from_xml(content.text()?, &self)
  }

  pub async fn get_bucket_info(&self) -> OssResult<Bucket<'_>> {
    let headers = None;
    let mut bucket_url = self.get_bucket_url()?;
    bucket_url.set_query(Some("bucketInfo"));

    let response = self.builder(VERB::GET, &bucket_url, headers, None).await?;
    let content = response.send().await?.handle_error()?;

    Bucket::from_xml(content.text().await?, &self)
  }
}

pub enum Grant{
  Private,
  PublicRead,
  PublicReadWrite,
}

impl Default for Grant {
  fn default() -> Self {
    Self::Private
  }
}

#[derive(Clone, Debug)]
pub enum DataRedundancyType{
  LRS,
  ZRS,
}

impl Default for DataRedundancyType{
  fn default() -> Self {
    Self::LRS
  }
}


#[derive(Default,Clone, Debug)]
pub struct BucketListObjectParms<'a>{
  pub list_type: u8,
  pub delimiter: &'a str,
  pub continuation_token: &'a str,
  pub max_keys: u32,
  pub prefix: &'a str,
  pub encoding_type: &'a str,
  pub fetch_owner: bool,
}

#[derive(Default,Clone, Debug)]
pub struct BucketListObject<'a>{
  //pub content:
  pub common_prefixes: &'a str,
  pub delimiter: &'a str,
  pub encoding_type: &'a str,
  pub display_name: &'a str,
  pub etag: &'a str,
  pub id: &'a str,
  pub is_truncated: bool,
  pub key: &'a str,
  pub last_modified: &'a str, // TODO 时间
  pub list_bucket_result: Option<&'a str>,
  pub start_after: Option<&'a str>,
  pub max_keys: u32,
  pub name: &'a str,
  // pub owner: &'a str,
  pub prefix: &'a str,
  pub size: u64,
  pub storage_class: &'a str,
  pub continuation_token: Option<&'a str>,
  pub key_count: i32,
  pub next_continuation_token: Option<&'a str>,
  pub restore_info: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub enum Location {
  CnHangzhou,
  CnShanghai,
  CnQingdao,
  CnBeijing,
  CnZhangjiakou, // 张家口 lenght=13
  CnHongkong,
  CnShenzhen,
  UsWest1,
  UsEast1,
  ApSouthEast1,
}

#[derive(Clone, Debug)]
pub struct BucketStat{
  pub storage: u64,
  pub object_count: u32,
  pub multipart_upload_count: u32,
  pub live_channel_count: u32,
  pub last_modified_time: u16,
  pub standard_storage: u64,
  pub standard_object_count: u32,
  pub infrequent_access_storage: u64,
  pub infrequent_access_real_storage: u64,
  pub infrequent_access_object_count: u64,
  pub archive_storage: u64,
  pub archive_real_storage: u64,
  pub archive_object_count: u64,
  pub cold_archive_storage: u64,
  pub cold_archive_real_storage: u64,
  pub cold_archive_object_count: u64,
}