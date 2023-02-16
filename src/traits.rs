//! # 解析 aliyun OSS 接口返回的 xml 原始数据的 trait
//! 开发者可利用该 trait 将 xml 高效地转化为 rust 的 struct 或者 enum 类型
//!
//! 本 trait 是零拷贝的，所以可以做到很高效
//!
//! ## 示例
//! ```
//! use aliyun_oss_client::decode::{RefineObject, RefineObjectList};
//! use thiserror::Error;
//!
//! struct MyFile {
//!     key: String,
//!     #[allow(dead_code)]
//!     other: String,
//! }
//! impl RefineObject<MyError> for MyFile {
//!
//!     fn set_key(&mut self, key: &str) -> Result<(), MyError> {
//!         self.key = key.to_string();
//!         Ok(())
//!     }
//! }
//!
//! #[derive(Default)]
//! struct MyBucket {
//!     name: String,
//!     files: Vec<MyFile>,
//! }
//!
//! impl RefineObjectList<MyFile, MyError> for MyBucket {
//!
//!     fn set_name(&mut self, name: &str) -> Result<(), MyError> {
//!         self.name = name.to_string();
//!         Ok(())
//!     }
//!     fn set_list(&mut self, list: Vec<MyFile>) -> Result<(), MyError> {
//!         self.files = list;
//!         Ok(())
//!     }
//! }
//!
//! #[derive(Debug, Error)]
//! enum MyError {
//!     #[error(transparent)]
//!     QuickXml(#[from] quick_xml::Error),
//! }
//!
//! fn get_with_xml() -> Result<(), MyError> {
//!     // 这是阿里云接口返回的原始数据
//!     let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
//!         <ListBucketResult>
//!           <Name>foo_bucket</Name>
//!           <Prefix></Prefix>
//!           <MaxKeys>100</MaxKeys>
//!           <Delimiter></Delimiter>
//!           <IsTruncated>false</IsTruncated>
//!           <NextContinuationToken>CiphcHBzL1RhdXJpIFB1Ymxpc2ggQXBwXzAuMS42X3g2NF9lbi1VUy5tc2kQAA--</NextContinuationToken>
//!           <Contents>
//!             <Key>9AB932LY.jpeg</Key>
//!             <LastModified>2022-06-26T09:53:21.000Z</LastModified>
//!             <ETag>"F75A15996D0857B16FA31A3B16624C26"</ETag>
//!             <Type>Normal</Type>
//!             <Size>18027</Size>
//!             <StorageClass>Standard</StorageClass>
//!           </Contents>
//!           <KeyCount>3</KeyCount>
//!         </ListBucketResult>"#;
//!
//!     // 除了设置Default 外，还可以做更多设置
//!     let mut bucket = MyBucket::default();
//!
//!     // 利用闭包对 MyFile 做一下初始化设置
//!     let init_file = || MyFile {
//!         key: String::default(),
//!         other: "abc".to_string(),
//!     };
//!
//!     bucket.decode(xml, init_file)?;
//!
//!     assert!(bucket.name == "foo_bucket");
//!     assert!(bucket.files[0].key == "9AB932LY.jpeg");
//!
//!     Ok(())
//! }
//!
//! let res = get_with_xml();
//!
//! if let Err(err) = res {
//!     eprintln!("{}", err);
//! }
//! ```

use std::borrow::Cow;

use quick_xml::{events::Event, Reader};

pub trait RefineObject<Error> {
    /// 提取 key
    fn set_key(&mut self, _key: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取最后修改时间
    fn set_last_modified(&mut self, _last_modified: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_etag(&mut self, _etag: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_type(&mut self, _type: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_size(&mut self, _size: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_storage_class(&mut self, _storage_class: &str) -> Result<(), Error> {
        Ok(())
    }
}

const PREFIX: &[u8] = b"Prefix";
const COMMON_PREFIX: &[u8] = b"CommonPrefixes";
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
const CONTENTS: &[u8] = b"Contents";

pub trait RefineObjectList<T, Error>
where
    T: RefineObject<Error>,
    Error: From<quick_xml::Error>,
{
    /// 提取 bucket 名
    fn set_name(&mut self, _name: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取前缀
    fn set_prefix(&mut self, _prefix: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取文件目录
    fn set_common_prefix(&mut self, _list: &Vec<Cow<'_, str>>) -> Result<(), Error> {
        Ok(())
    }

    fn set_max_keys(&mut self, _max_keys: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_key_count(&mut self, _key_count: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取翻页信息，有下一页，返回 Some, 否则返回 None
    fn set_next_continuation_token(&mut self, _token: Option<&str>) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 object 列表
    fn set_list(&mut self, _list: Vec<T>) -> Result<(), Error> {
        Ok(())
    }

    /// # 由 xml 转 struct 的底层实现
    /// - `init_object` 用于初始化 object 结构体的方法
    fn decode<F>(&mut self, xml: &str, mut init_object: F) -> Result<(), Error>
    where
        F: FnMut() -> T,
    {
        //println!("from_xml: {:#}", xml);
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::with_capacity(xml.len());

        let mut key = Cow::from("");
        let mut last_modified = Cow::from(String::with_capacity(20));
        let mut _type = Cow::from("");
        let mut etag = Cow::from(String::with_capacity(34));
        let mut size = Cow::from("");
        let mut storage_class = Cow::from(String::with_capacity(11));

        let mut is_common_pre = false;
        let mut prefix_vec = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        COMMON_PREFIX => {
                            is_common_pre = true;
                            prefix_vec = Vec::new();
                        }
                        PREFIX => {
                            if is_common_pre {
                                prefix_vec.push(reader.read_text(e.to_end().name())?);
                            } else {
                                self.set_prefix(&reader.read_text(e.to_end().name())?)?;
                            }
                        }
                        NAME => self.set_name(&reader.read_text(e.to_end().name())?)?,
                        MAX_KEYS => self.set_max_keys(&reader.read_text(e.to_end().name())?)?,
                        KEY_COUNT => self.set_key_count(&reader.read_text(e.to_end().name())?)?,
                        IS_TRUNCATED => {
                            //is_truncated = reader.read_text(e.to_end().name())?.to_string() == "true"
                        }
                        NEXT_CONTINUATION_TOKEN => {
                            let next_continuation_token = reader.read_text(e.to_end().name())?;
                            self.set_next_continuation_token(
                                if next_continuation_token.len() > 0 {
                                    Some(&next_continuation_token)
                                } else {
                                    None
                                },
                            )?;
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
                            etag = Cow::Owned((*tag.trim_matches('"')).to_owned());
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
                Ok(Event::End(ref e)) => match e.name().as_ref() {
                    COMMON_PREFIX => {
                        self.set_common_prefix(&prefix_vec)?;
                        is_common_pre = false;
                    }
                    CONTENTS => {
                        let mut object = init_object();
                        object.set_key(&key)?;
                        object.set_last_modified(&last_modified)?;
                        object.set_etag(&etag)?;
                        object.set_type(&_type)?;
                        object.set_size(&size)?;
                        object.set_storage_class(&storage_class)?;
                        result.push(object);
                    }
                    _ => (),
                },
                Ok(Event::Eof) => {
                    self.set_list(result)?;
                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(Error::from(e));
                }
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }

        Ok(())
    }
}

pub trait RefineBucket<Error>
where
    Error: From<quick_xml::Error>,
{
    fn set_name(&mut self, _name: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_creation_date(&mut self, _creation_date: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_location(&mut self, _location: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_extranet_endpoint(&mut self, _extranet_endpoint: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_intranet_endpoint(&mut self, _intranet_endpoint: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_storage_class(&mut self, _storage_class: &str) -> Result<(), Error> {
        Ok(())
    }

    fn decode(&mut self, xml: &str) -> Result<(), Error> {
        //println!("from_xml: {:#}", xml);
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::with_capacity(xml.len());

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    NAME => self.set_name(&reader.read_text(e.to_end().name())?)?,
                    CREATION_DATE => {
                        self.set_creation_date(&reader.read_text(e.to_end().name())?)?
                    }
                    EXTRANET_ENDPOINT => {
                        self.set_extranet_endpoint(&reader.read_text(e.to_end().name())?)?
                    }
                    INTRANET_ENDPOINT => {
                        self.set_intranet_endpoint(&reader.read_text(e.to_end().name())?)?
                    }
                    LOCATION => self.set_location(&reader.read_text(e.to_end().name())?)?,
                    STORAGE_CLASS => {
                        self.set_storage_class(&reader.read_text(e.to_end().name())?)?
                    }
                    _ => (),
                },
                Ok(Event::Eof) => {
                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(Error::from(e));
                }
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }
        Ok(())
    }
}

const TRUE: &str = "true";

pub trait RefineBucketList<T: RefineBucket<Error>, Error>
where
    Error: From<quick_xml::Error>,
{
    fn set_prefix(&mut self, _prefix: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_marker(&mut self, _marker: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_max_keys(&mut self, _max_keys: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_is_truncated(&mut self, _is_truncated: bool) -> Result<(), Error> {
        Ok(())
    }

    fn set_next_marker(&mut self, _next_marker: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_id(&mut self, _id: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_display_name(&mut self, _display_name: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_list(&mut self, _list: Vec<T>) -> Result<(), Error> {
        Ok(())
    }

    fn decode<F>(&mut self, xml: &str, mut init_bucket: F) -> Result<(), Error>
    where
        F: FnMut() -> T,
    {
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::with_capacity(xml.len());

        let mut name = Cow::from("");
        let mut location = Cow::from("");
        let mut creation_date = Cow::from(String::with_capacity(20));

        // 目前最长的可用区 zhangjiakou 13 ，剩余部分总共 20
        let mut extranet_endpoint = Cow::from(String::with_capacity(33));
        // 上一个长度 + 9 （-internal）
        let mut intranet_endpoint = Cow::from(String::with_capacity(42));
        // 最长的值 ColdArchive 11
        let mut storage_class = Cow::from(String::with_capacity(11));

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    PREFIX => self.set_prefix(&reader.read_text(e.to_end().name())?)?,
                    MARKER => self.set_marker(&reader.read_text(e.to_end().name())?)?,
                    MAX_KEYS => self.set_max_keys(&reader.read_text(e.to_end().name())?)?,
                    IS_TRUNCATED => {
                        self.set_is_truncated(reader.read_text(e.to_end().name())? == TRUE)?;
                    }
                    NEXT_MARKER => self.set_next_marker(&reader.read_text(e.to_end().name())?)?,
                    ID => self.set_id(&reader.read_text(e.to_end().name())?)?,
                    DISPLAY_NAME => self.set_display_name(&reader.read_text(e.to_end().name())?)?,
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
                    let mut bucket = init_bucket();
                    bucket.set_name(&name)?;
                    bucket.set_creation_date(&creation_date)?;
                    bucket.set_location(&location)?;
                    bucket.set_extranet_endpoint(&extranet_endpoint)?;
                    bucket.set_intranet_endpoint(&intranet_endpoint)?;
                    bucket.set_storage_class(&storage_class)?;
                    result.push(bucket);
                }
                Ok(Event::Eof) => {
                    self.set_list(result)?;
                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(Error::from(e));
                }
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_common_prefixes() {
        struct ObjectA {}
        impl RefineObject<quick_xml::Error> for ObjectA {}

        struct ListA {}

        impl RefineObjectList<ObjectA, quick_xml::Error> for ListA {
            fn set_prefix(&mut self, prefix: &str) -> Result<(), quick_xml::Error> {
                assert!(prefix == "bar");
                Ok(())
            }

            fn set_common_prefix(
                &mut self,
                list: &Vec<Cow<'_, str>>,
            ) -> Result<(), quick_xml::Error> {
                assert!(list[0] == "foo1/");
                assert!(list[1] == "foo2/");
                Ok(())
            }
        }

        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ListBucketResult>
          <Prefix>bar</Prefix>
          <Contents>
            <Key>9AB932LY.jpeg</Key>
          </Contents>
          <Contents>
            <Key>9AB932LY.jpeg</Key>
          </Contents>
          <CommonPrefixes>
            <Prefix>foo1/</Prefix>
            <Prefix>foo2/</Prefix>
          </CommonPrefixes>
        </ListBucketResult>
        "#;

        let mut list = ListA {};

        let init_object = || ObjectA {};

        let res = list.decode(xml, init_object);

        assert!(res.is_ok());
    }
}
