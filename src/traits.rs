use std::borrow::Cow;
use std::error::Error;

use quick_xml::events::Event;
use quick_xml::Reader;

pub trait OssIntoObject
where
    Self: Sized,
    Self::Bucket: Clone,
    Self::Error: Error,
{
    type Bucket;
    type Error;

    fn set_key(self, _key: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_last_modified(self, _last_modified: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_etag(self, _etag: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_type(self, _type: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_size(self, _size: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_storage_class(self, _storage_class: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_bucket(self, _bucket: Self::Bucket) -> Self {
        self
    }
}

const PREFIX: &[u8] = b"Prefix";
const NAME: &[u8] = b"Name";
const MAX_KEYS: &[u8] = b"MaxKeys";
const KEY_COUNT: &[u8] = b"KeyCount";
const IS_TRUNCATED: &[u8] = b"IsTruncated";
const NEXT_CONTINUATION_TOKEN: &[u8] = b"NextContinuationToken";
const KEY: &[u8] = b"Key";
const LAST_MODIFIED: &[u8] = b"LastModified";
const E_TAG: &[u8] = b"ETag";
const TYPE: &[u8] = b"Type";
const SIZE: &[u8] = b"Size";
const STORAGE_CLASS: &[u8] = b"StorageClass";
const BUCKET: &[u8] = b"Bucket";

const CREATION_DATE: &[u8] = b"CreationDate";
const EXTRANET_ENDPOINT: &[u8] = b"ExtranetEndpoint";
const INTRANET_ENDPOINT: &[u8] = b"IntranetEndpoint";
const LOCATION: &[u8] = b"Location";

const MARKER: &[u8] = b"Marker";
const NEXT_MARKER: &[u8] = b"NextMarker";
const ID: &[u8] = b"ID";
const DISPLAY_NAME: &[u8] = b"DisplayName";

pub trait OssIntoObjectList<T>
where
    Self: Sized,
    T: OssIntoObject + Default,
    Self::Error: Error + From<quick_xml::Error> + From<T::Error>,
{
    type Error;
    fn set_name(self, _name: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_prefix(self, _prefix: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_max_keys(self, _max_keys: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_key_count(self, _key_count: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_next_continuation_token(self, _token: Option<&str>) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_list(self, _list: Vec<T>) -> Result<Self, Self::Error> {
        Ok(self)
    }

    fn from_xml(self, xml: &str, bucket: T::Bucket) -> Result<Self, Self::Error> {
        //println!("from_xml: {:#}", xml);
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::with_capacity(xml.len());

        let mut key = Cow::from("");
        let mut last_modified = Cow::from(""); //::with_capacity(20);
        let mut _type = Cow::from("");
        let mut etag = Cow::from(""); //String::with_capacity(34); // 32 位 加两位 "" 符号
        let mut size = Cow::from("");
        let mut storage_class = Cow::from(""); //String::with_capacity(11);
                                               // let mut is_truncated = false;

        let mut name = Cow::from("");
        let mut prefix = Cow::from("");
        let mut max_keys = Cow::from("");
        let mut key_count = Cow::from("");
        let mut next_continuation_token = Cow::from("");

        let list_object;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        PREFIX => prefix = reader.read_text(e.to_end().name())?,
                        NAME => name = reader.read_text(e.to_end().name())?,
                        MAX_KEYS => {
                            max_keys = reader.read_text(e.to_end().name())?;
                        }
                        KEY_COUNT => {
                            key_count = reader.read_text(e.to_end().name())?;
                        }
                        IS_TRUNCATED => {
                            //is_truncated = reader.read_text(e.to_end().name())?.to_string() == "true"
                        }
                        NEXT_CONTINUATION_TOKEN => {
                            next_continuation_token = reader.read_text(e.to_end().name())?;
                        }
                        // b"Contents" => {
                        //     // key.clear();
                        //     // last_modified.clear();
                        //     // etag.clear();
                        //     // //_type.clear();
                        //     // storage_class.clear();
                        // }
                        KEY => key = reader.read_text(e.to_end().name())?,
                        LAST_MODIFIED => last_modified = reader.read_text(e.to_end().name())?,
                        E_TAG => {
                            let tag = reader.read_text(e.to_end().name())?;

                            let new_tag = tag.into_owned();
                            let new_tag = &new_tag.trim_matches('"');
                            etag = Cow::Owned((*new_tag).to_owned());
                        }
                        TYPE => _type = reader.read_text(e.to_end().name())?,
                        SIZE => {
                            size = reader.read_text(e.to_end().name())?;
                        }
                        STORAGE_CLASS => {
                            storage_class = reader.read_text(e.to_end().name())?;
                        }
                        _ => (),
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"Contents" => {
                    let object = T::default()
                        .set_bucket(bucket.clone())
                        .set_key(&key)?
                        .set_last_modified(&last_modified)?
                        .set_etag(&etag)?
                        .set_type(&_type)?
                        .set_size(&size)?
                        .set_storage_class(&storage_class)?;
                    result.push(object);
                }
                Ok(Event::Eof) => {
                    list_object = self
                        .set_name(&name)?
                        .set_prefix(&prefix)?
                        .set_max_keys(&max_keys)?
                        .set_key_count(&key_count)?
                        .set_list(result)?
                        .set_next_continuation_token(if next_continuation_token.len() > 0 {
                            Some(&next_continuation_token)
                        } else {
                            None
                        })?;
                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(Self::Error::from(e));
                }
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }

        Ok(list_object)
    }
}

pub trait OssIntoBucket
where
    Self: Sized,
    Self::Error: Error + From<quick_xml::Error>,
{
    type Error;
    fn set_name(self, _name: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_creation_date(self, _creation_date: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_location(self, _location: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_extranet_endpoint(self, _extranet_endpoint: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_intranet_endpoint(self, _intranet_endpoint: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_storage_class(self, _storage_class: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }

    fn from_xml(self, xml: &str) -> Result<Self, Self::Error> {
        //println!("from_xml: {:#}", xml);
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::with_capacity(xml.len());

        let mut name = Cow::from("");
        let mut location = Cow::from("");
        let mut creation_date = Cow::from(""); // String::with_capacity(20);

        // 目前最长的可用区 zhangjiakou 13 ，剩余部分总共 20
        let mut extranet_endpoint = Cow::from(""); // String::with_capacity(33);
                                                   // 上一个长度 + 9 （-internal）
        let mut intranet_endpoint = Cow::from(""); // String::with_capacity(42);
                                                   // 最长的值 ColdArchive 11
        let mut storage_class = Cow::from(""); // String::with_capacity(11);

        let bucket;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    NAME => name = reader.read_text(e.to_end().name())?,
                    CREATION_DATE => creation_date = reader.read_text(e.to_end().name())?,
                    EXTRANET_ENDPOINT => extranet_endpoint = reader.read_text(e.to_end().name())?,
                    INTRANET_ENDPOINT => intranet_endpoint = reader.read_text(e.to_end().name())?,
                    LOCATION => location = reader.read_text(e.to_end().name())?,
                    STORAGE_CLASS => storage_class = reader.read_text(e.to_end().name())?,
                    _ => (),
                },
                Ok(Event::Eof) => {
                    bucket = self
                        .set_name(&name)?
                        .set_creation_date(&creation_date)?
                        .set_location(&location)?
                        .set_extranet_endpoint(&extranet_endpoint)?
                        .set_intranet_endpoint(&intranet_endpoint)?
                        .set_storage_class(&storage_class)?;
                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(Self::Error::from(e));
                }
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }
        Ok(bucket)
    }
}

pub trait OssIntoBucketList<T: OssIntoBucket + Default>
where
    Self: Sized,
    Self::Error: Error + From<quick_xml::Error> + From<T::Error>,
{
    type Error;
    fn set_prefix(self, _prefix: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_marker(self, _marker: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_max_keys(self, _max_keys: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_is_truncated(self, _is_truncated: bool) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_next_marker(self, _next_marker: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_id(self, _id: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_display_name(self, _display_name: &str) -> Result<Self, Self::Error> {
        Ok(self)
    }
    fn set_list(self, _list: Vec<T>) -> Result<Self, Self::Error> {
        Ok(self)
    }

    fn from_xml(self, xml: &str) -> Result<Self, Self::Error> {
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::with_capacity(xml.len());

        let mut prefix = Cow::from("");
        let mut marker = Cow::from("");
        let mut max_keys = Cow::from("");
        let mut is_truncated = false;
        let mut next_marker = Cow::from("");
        let mut id = Cow::from(""); //String::with_capacity(8);
        let mut display_name = Cow::from(""); //String::with_capacity(8);

        let mut name = Cow::from("");
        let mut location = Cow::from("");
        let mut creation_date = Cow::from(""); //String::with_capacity(20);

        // 目前最长的可用区 zhangjiakou 13 ，剩余部分总共 20
        let mut extranet_endpoint = Cow::from(""); //String::with_capacity(33);
                                                   // 上一个长度 + 9 （-internal）
        let mut intranet_endpoint = Cow::from(""); //String::with_capacity(42);
                                                   // 最长的值 ColdArchive 11
        let mut storage_class = Cow::from(""); //String::with_capacity(11);

        let bucket_list;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    PREFIX => prefix = reader.read_text(e.to_end().name())?,
                    MARKER => marker = reader.read_text(e.to_end().name())?,
                    MAX_KEYS => max_keys = reader.read_text(e.to_end().name())?,
                    IS_TRUNCATED => {
                        is_truncated = reader.read_text(e.to_end().name())?.to_string() == "true"
                    }
                    NEXT_MARKER => next_marker = reader.read_text(e.to_end().name())?,
                    ID => id = reader.read_text(e.to_end().name())?,
                    DISPLAY_NAME => display_name = reader.read_text(e.to_end().name())?,

                    // b"Bucket" => {
                    //     // name.clear();
                    //     // location.clear();
                    //     // creation_date.clear();
                    //     // extranet_endpoint.clear();
                    //     // intranet_endpoint.clear();
                    //     // storage_class.clear();
                    // }
                    NAME => name = reader.read_text(e.to_end().name())?,
                    CREATION_DATE => creation_date = reader.read_text(e.to_end().name())?,
                    EXTRANET_ENDPOINT => extranet_endpoint = reader.read_text(e.to_end().name())?,
                    INTRANET_ENDPOINT => intranet_endpoint = reader.read_text(e.to_end().name())?,
                    LOCATION => location = reader.read_text(e.to_end().name())?,
                    STORAGE_CLASS => storage_class = reader.read_text(e.to_end().name())?,
                    _ => (),
                },
                Ok(Event::End(ref e)) if e.name().as_ref() == BUCKET => {
                    //let in_creation_date = &creation_date.parse::<DateTime<Utc>>()?;
                    let bucket = T::default()
                        .set_name(&name)?
                        .set_creation_date(&creation_date)?
                        .set_location(&location)?
                        .set_extranet_endpoint(&extranet_endpoint)?
                        .set_intranet_endpoint(&intranet_endpoint)?
                        .set_storage_class(&storage_class)?;
                    result.push(bucket);
                }
                Ok(Event::Eof) => {
                    bucket_list = self
                        .set_prefix(&prefix)?
                        .set_marker(&marker)?
                        .set_max_keys(&max_keys)?
                        .set_is_truncated(is_truncated)?
                        .set_next_marker(&next_marker)?
                        .set_id(&id)?
                        .set_display_name(&display_name)?
                        .set_list(result)?;

                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(Self::Error::from(e));
                }
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }
        Ok(bucket_list)
    }
}
