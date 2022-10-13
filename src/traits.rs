use std::error::Error;
use std::fmt::{self, Debug};

use quick_xml::{Reader};
use quick_xml::events::Event;

use crate::config::BucketBase;
use crate::errors::{OssResult, OssError};
use crate::types::InvalidEndPoint;

pub trait OssIntoObject where Self: Sized{
    fn set_key(self, _key: String) -> Result<Self, InvalidObjectValue>{
        Ok(self)
    }
    fn set_last_modified(self, _last_modified: String) -> Result<Self, InvalidObjectValue>{
        Ok(self)
    }
    fn set_etag(self, _etag: String) -> Result<Self, InvalidObjectValue>{
        Ok(self)
    }
    fn set_type(self, _type: String) -> Result<Self, InvalidObjectValue>{
        Ok(self)
    }
    fn set_size(self, _size: String) -> Result<Self, InvalidObjectValue>{
        Ok(self)
    }
    fn set_storage_class(self, _storage_class: String) -> Result<Self, InvalidObjectValue>{
        Ok(self)
    }
    fn set_bucket(self, _bucket: BucketBase) -> Self{
        self
    }
}

#[derive(Debug)]
pub struct InvalidObjectValue;

impl fmt::Display for InvalidObjectValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "faild parse to object value")
    }
}

impl Error for InvalidObjectValue{}

#[derive(Debug)]
pub struct InvalidObjectListValue;

impl fmt::Display for InvalidObjectListValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "faild parse to object list value")
    }
}

impl Error for InvalidObjectListValue{}

pub trait OssIntoObjectList<T: OssIntoObject + Default>
where Self: Sized
{
    fn set_name(self, _name: String) -> Result<Self, InvalidObjectListValue>{
        Ok(self)
    }
    fn set_prefix(self, _prefix: String) -> Result<Self, InvalidObjectListValue>{
        Ok(self)
    }
    fn set_max_keys(self, _max_keys: String) -> Result<Self, InvalidObjectListValue>{
        Ok(self)
    }
    fn set_key_count(self, _key_count: String) -> Result<Self, InvalidObjectListValue>{
        Ok(self)
    }
    fn set_next_continuation_token(self, _token: Option<String>) -> Result<Self, InvalidObjectListValue>{
        Ok(self)
    }
    fn set_list(self, _list: Vec<T>) -> Result<Self, InvalidObjectListValue>{
        Ok(self)
    }

    fn from_xml(self, xml: String, bucket: &BucketBase) -> OssResult<Self> {
        //println!("from_xml: {:#}", xml);
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
                    let object = T::default()
                        .set_bucket(bucket.to_owned())
                        .set_key(key.clone()).map_err(|e|OssError::InvalidObjectValue(e))?
                        .set_last_modified(last_modified.clone()).map_err(|e|OssError::InvalidObjectValue(e))?
                        .set_etag(etag.clone()).map_err(|e|OssError::InvalidObjectValue(e))?
                        .set_type(_type.clone()).map_err(|e|OssError::InvalidObjectValue(e))?
                        .set_size(size.clone()).map_err(|e|OssError::InvalidObjectValue(e))?
                        .set_storage_class(storage_class.clone()).map_err(|e|OssError::InvalidObjectValue(e))?
                    ;
                    result.push(object);
                }
                Ok(Event::Eof) => {
                    list_object = self.set_name(name).map_err(|e|OssError::InvalidObjectListValue(e))?
                        .set_prefix(prefix).map_err(|e|OssError::InvalidObjectListValue(e))?
                        .set_max_keys(max_keys).map_err(|e|OssError::InvalidObjectListValue(e))?
                        .set_key_count(key_count).map_err(|e|OssError::InvalidObjectListValue(e))?
                        .set_list(result).map_err(|e|OssError::InvalidObjectListValue(e))?
                        .set_next_continuation_token(next_continuation_token).map_err(|e|OssError::InvalidObjectListValue(e))?;
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


pub trait OssIntoBucket where Self: Sized {
    fn set_name(self, _name: String) -> Result<Self, InvalidBucketValue>{
        Ok(self)
    }
    fn set_creation_date(self, _creation_date: String) -> Result<Self, InvalidBucketValue>{
        Ok(self)
    }
    fn set_location(self, _location: String) -> Result<Self, InvalidBucketValue>{
        Ok(self)
    }
    fn set_extranet_endpoint(self, _extranet_endpoint: String) -> Result<Self, InvalidBucketValue>{
        Ok(self)
    }
    fn set_intranet_endpoint(self, _intranet_endpoint: String) -> Result<Self, InvalidBucketValue>{
        Ok(self)
    }
    fn set_storage_class(self, _storage_class: String) -> Result<Self, InvalidBucketValue>{
        Ok(self)
    }

    fn from_xml(self, xml: String) -> OssResult<Self>{
        //println!("from_xml: {:#}", xml);
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
                    bucket = self.set_name(name).map_err(|e|OssError::InvalidBucketValue(e))?
                        .set_creation_date(creation_date).map_err(|e|OssError::InvalidBucketValue(e))?
                        .set_location(location).map_err(|e|OssError::InvalidBucketValue(e))?
                        .set_extranet_endpoint(extranet_endpoint).map_err(|e|OssError::InvalidBucketValue(e))?
                        .set_intranet_endpoint(intranet_endpoint).map_err(|e|OssError::InvalidBucketValue(e))?
                        .set_storage_class(storage_class).map_err(|e|OssError::InvalidBucketValue(e))?;
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

#[derive(Debug)]
pub struct InvalidBucketValue;

impl fmt::Display for InvalidBucketValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "faild parse to bucket value")
    }
}

impl Error for InvalidBucketValue{}

impl From<InvalidEndPoint> for InvalidBucketValue{
    //TODO 待完善
    fn from(_end: InvalidEndPoint) -> InvalidBucketValue {
        InvalidBucketValue{}
    }
}

pub trait OssIntoBucketList<T: OssIntoBucket + Default>
where Self: Sized {
    fn set_prefix(self, _prefix: String) -> Result<Self, InvalidBucketListValue>{
        Ok(self)
    }
    fn set_marker(self, _marker: String) -> Result<Self, InvalidBucketListValue>{
        Ok(self)
    }
    fn set_max_keys(self, _max_keys: String) -> Result<Self, InvalidBucketListValue>{
        Ok(self)
    }
    fn set_is_truncated(self, _is_truncated: bool) -> Result<Self, InvalidBucketListValue>{
        Ok(self)
    }
    fn set_next_marker(self, _next_marker: String) -> Result<Self, InvalidBucketListValue>{
        Ok(self)
    }
    fn set_id(self, _id: String) -> Result<Self, InvalidBucketListValue>{
        Ok(self)
    }
    fn set_display_name(self, _display_name: String) -> Result<Self, InvalidBucketListValue>{
        Ok(self)
    }
    fn set_list(self, _list: Vec<T>) -> Result<Self, InvalidBucketListValue>{
        Ok(self)
    }

    fn from_xml(self,xml: String) -> OssResult<Self>{
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
    
        let bucket_list;
    
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
                    let bucket = T::default()
                        .set_name(name.clone()).map_err(|e|OssError::InvalidBucketValue(e))?
                        .set_creation_date(creation_date.clone()).map_err(|e|OssError::InvalidBucketValue(e))?
                        .set_location(location.clone()).map_err(|e|OssError::InvalidBucketValue(e))?
                        .set_extranet_endpoint(extranet_endpoint.clone()).map_err(|e|OssError::InvalidBucketValue(e))?
                        .set_intranet_endpoint(intranet_endpoint.clone()).map_err(|e|OssError::InvalidBucketValue(e))?
                        .set_storage_class(storage_class.clone()).map_err(|e|OssError::InvalidBucketValue(e))?
                      ;
                    result.push(bucket);
                }
                Ok(Event::Eof) => {
                    bucket_list = self.set_prefix(prefix).map_err(|e|OssError::InvalidBucketListValue(e))?
                        .set_marker(marker).map_err(|e|OssError::InvalidBucketListValue(e))?
                        .set_max_keys(max_keys).map_err(|e|OssError::InvalidBucketListValue(e))?
                        .set_is_truncated(is_truncated).map_err(|e|OssError::InvalidBucketListValue(e))?
                        .set_next_marker(next_marker).map_err(|e|OssError::InvalidBucketListValue(e))?
                        .set_id(id).map_err(|e|OssError::InvalidBucketListValue(e))?
                        .set_display_name(display_name).map_err(|e|OssError::InvalidBucketListValue(e))?
                        .set_list(result)?;
                    
                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(OssError::Input(format!("Error at position {}: {:?}", reader.buffer_position(), e)))
                },
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }
        Ok(bucket_list)
    }
}

#[derive(Debug)]
pub struct InvalidBucketListValue;

impl fmt::Display for InvalidBucketListValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "faild parse to bucket list value")
    }
}

impl Error for InvalidBucketListValue{}
