use quick_xml::{Reader};
use quick_xml::events::Event;

use crate::errors::{OssResult, OssError};

pub trait ObjectTrait {
  /// 使用 oss 返回的数据初始化 Object 结构体
  fn from_oss(
    key: String,
    last_modified: String,
    etag: String,
    _type: String,
    size: String,
    storage_class: String
  ) -> OssResult<Self>
  where Self: Sized;
}

pub trait ObjectListTrait<OBJ: ObjectTrait> {
  /// 使用 oss 返回的数据初始化 ObjectList 结构体
  fn from_oss(
    name: String,
    prefix: String,
    max_keys: String,
    key_count: String,
    object_list: Vec<OBJ>,
    next_continuation_token: Option<String>,
  ) -> OssResult<Self>
  where Self: Sized;

  fn from_xml(xml: String) -> OssResult<Self> where Self: Sized {
    let mut result = Vec::new();
    let mut reader = Reader::from_str(xml.as_str());
    reader.trim_text(true);
    let mut buf = Vec::with_capacity(xml.len());

    let mut key = String::new();
    let mut last_modified = String::with_capacity(20);
    let mut _type = String::new();
    let mut etag = String::with_capacity(34); // 32 位 加两位 "" 符号
    let mut size = String::new();
    let mut storage_class = String::with_capacity(11);
    // let mut is_truncated = false;

    let mut name = String::new();
    let mut prefix = String::new();
    let mut max_keys = String::new();
    let mut key_count = String::new();
    let mut next_continuation_token: Option<String> = None;

    let list_object;

    loop {
      match reader.read_event_into(&mut buf) {
          Ok(Event::Start(e)) => {
              match e.name().as_ref() {
                  b"Prefix" => {
                    prefix = reader.read_text(e.to_end().into_owned().name())?.to_string()
                  },
                  b"Name" => {
                    name = reader.read_text(e.to_end().into_owned().name())?.to_string()
                  },
                  b"MaxKeys" => {
                    max_keys = reader.read_text(e.to_end().into_owned().name())?.to_string();
                  },
                  b"KeyCount" => {
                    key_count = reader.read_text(e.to_end().into_owned().name())?.to_string();
                  },
                  b"IsTruncated" => {
                    //is_truncated = reader.read_text(e.to_end().into_owned().name())?.to_string() == "true"
                  }
                  b"NextContinuationToken" => {
                    next_continuation_token = Some(reader.read_text(e.to_end().into_owned().name())?.to_string());
                  }
                  b"Contents" => {
                    key.clear();
                    last_modified.clear();
                    etag.clear();
                    _type.clear();
                    storage_class.clear();
                  }

                  b"Key" => key = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                  b"LastModified" => last_modified = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                  b"ETag" => {
                    etag = reader.read_text(e.to_end().into_owned().name())?.to_string();
                    let str = "\"";
                    etag = etag.replace(str, "");
                  }
                  b"Type" => {
                    _type = reader.read_text(e.to_end().into_owned().name())?.to_string()
                  }
                  b"Size" => {
                    size = reader.read_text(e.to_end().into_owned().name())?.to_string();
                  },
                  b"StorageClass" => {
                    storage_class = reader.read_text(e.to_end().into_owned().name())?.to_string()
                  }
                  _ => (),
              }
          },
          Ok(Event::End(ref e)) if e.name().as_ref() == b"Contents" => {
            let object = OBJ::from_oss(
                key.clone(),
                last_modified.clone(),
                etag.clone(),
                _type.clone(),
                size.clone(),
                storage_class.clone(),
            )?;
            result.push(object);
          }
          Ok(Event::Eof) => {
              list_object = Self::from_oss(
                  string2option(name).ok_or(OssError::Input("get name failed by xml".to_string()))?,
                  prefix,
                  max_keys,
                  key_count,
                  result,
                  next_continuation_token
              )?;
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

pub trait BucketTrait {
  /// 使用 oss 返回的数据初始化 Bucket 结构体
  fn from_oss<'a>(
    name: String,
    creation_date: String,
    location: String,
    extranet_endpoint: String,
    intranet_endpoint: String,
    storage_class: String,
  ) -> OssResult<Self>
  where Self: Sized;

  fn from_xml(xml: String) -> OssResult<Self>
  where Self: Sized{
    let mut reader = Reader::from_str(xml.as_str());
    reader.trim_text(true);
    let mut buf = Vec::with_capacity(xml.len());

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
      match reader.read_event_into(&mut buf) {
          Ok(Event::Start(e)) => {
              match e.name().as_ref() {
                  b"Name" => name = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                  b"CreationDate" => creation_date = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                  b"ExtranetEndpoint" => {
                      extranet_endpoint = reader.read_text(e.to_end().into_owned().name())?.to_string()
                  }
                  b"IntranetEndpoint" => {
                      intranet_endpoint = reader.read_text(e.to_end().into_owned().name())?.to_string()
                  }
                  b"Location" => location = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                  b"StorageClass" => {
                      storage_class = reader.read_text(e.to_end().into_owned().name())?.to_string()
                  }
                  _ => (),
              }
          },
          Ok(Event::Eof) => {
            //let in_creation_date = &creation_date.parse::<DateTime<Utc>>()?;
            bucket = BucketTrait::from_oss(
              name.clone(),
              creation_date.clone(),
              location.clone(),
              extranet_endpoint.clone(),
              intranet_endpoint.clone(),
              storage_class.clone(),
            )?;
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

pub trait ListBucketTrait {
  type Bucket: BucketTrait;
  /// 使用 oss 返回的数据初始化 ListBucket 结构体
  fn from_oss(
    prefix: Option<String>, 
    marker: Option<String>,
    max_keys: Option<String>,
    is_truncated: bool,
    next_marker: Option<String>,
    id: Option<String>,
    display_name: Option<String>,
    buckets: Vec<Self::Bucket>,
  ) -> OssResult<Self>
  where Self: Sized;

  fn from_xml(xml: String) -> OssResult<Self>
  where Self: Sized {
    let mut result = Vec::new();
    let mut reader = Reader::from_str(xml.as_str());
    reader.trim_text(true);
    let mut buf = Vec::with_capacity(xml.len());

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
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"Prefix" => prefix = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                    b"Marker" => marker = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                    b"MaxKeys" => max_keys = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                    b"IsTruncated" => {
                        is_truncated = reader.read_text(e.to_end().into_owned().name())?.to_string() == "true"
                    }
                    b"NextMarker" => next_marker = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                    b"ID" => id = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                    b"DisplayName" => display_name = reader.read_text(e.to_end().into_owned().name())?.to_string(),

                    b"Bucket" => {
                        name.clear();
                        location.clear();
                        creation_date.clear();
                        extranet_endpoint.clear();
                        intranet_endpoint.clear();
                        storage_class.clear();
                    }

                    b"Name" => name = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                    b"CreationDate" => creation_date = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                    b"ExtranetEndpoint" => {
                        extranet_endpoint = reader.read_text(e.to_end().into_owned().name())?.to_string()
                    }
                    b"IntranetEndpoint" => {
                        intranet_endpoint = reader.read_text(e.to_end().into_owned().name())?.to_string()
                    }
                    b"Location" => location = reader.read_text(e.to_end().into_owned().name())?.to_string(),
                    b"StorageClass" => {
                        storage_class = reader.read_text(e.to_end().into_owned().name())?.to_string()
                    }
                    _ => (),
                }
            },
            Ok(Event::End(ref e)) if e.name().as_ref() == b"Bucket" => {
              //let in_creation_date = &creation_date.parse::<DateTime<Utc>>()?;
              let bucket = BucketTrait::from_oss(
                  name.clone(),
                  creation_date.clone(),
                  location.clone(),
                  extranet_endpoint.clone(),
                  intranet_endpoint.clone(),
                  storage_class.clone(),
              )?;
              result.push(bucket);
            }
            Ok(Event::Eof) => {
                list_buckets = ListBucketTrait::from_oss(
                    string2option(prefix),
                    string2option(marker),
                    string2option(max_keys),
                    is_truncated,
                    string2option(next_marker),
                    string2option(id),
                    string2option(display_name),
                    result,
                )?;
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

#[inline]
fn string2option(string: String) -> Option<String> {
  if string.len() == 0 {
    return None
  }
  Some(string)
}
