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
//! use aliyun_oss_client::DecodeListError;
//!
//! // 自定义的 Error 需要实现这两个 Trait，用于内部解析方法在调用时，统一处理异常
//! #[derive(Debug, Error, DecodeListError)]
//! #[error("my error")]
//! struct MyError {}
//!
//! fn get_with_xml() -> Result<(), aliyun_oss_client::decode::InnerListError> {
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
use std::error::Error as StdError;
use std::fmt::Display;
use std::num::ParseIntError;

use quick_xml::{events::Event, Reader};

use crate::types::InvalidEndPoint;
#[cfg(feature = "core")]
use crate::{
    errors::OssError,
    types::object::{InvalidObjectDir, InvalidObjectPath},
};

#[cfg(test)]
mod test;

/// 将一个 object 的数据写入到 rust 类型
pub trait RefineObject<Error: StdError + 'static> {
    /// 提取 key
    fn set_key(&mut self, _key: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取最后修改时间
    fn set_last_modified(&mut self, _last_modified: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 etag
    fn set_etag(&mut self, _etag: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 type
    fn set_type(&mut self, _type: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 size
    fn set_size(&mut self, _size: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 storage_class
    fn set_storage_class(&mut self, _storage_class: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 对单个 objcet 部分的 xml 内容进行解析
    fn decode(&mut self, xml: &str) -> Result<(), InnerItemError> {
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::with_capacity(xml.len());
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    KEY => self.set_key(&reader.read_text(e.to_end().name())?)?,
                    LAST_MODIFIED => {
                        self.set_last_modified(&reader.read_text(e.to_end().name())?)?
                    }
                    E_TAG => {
                        let tag = reader.read_text(e.to_end().name())?;
                        self.set_etag(tag.trim_matches('"'))?;
                    }
                    TYPE => self.set_type(&reader.read_text(e.to_end().name())?)?,
                    SIZE => {
                        self.set_size(&reader.read_text(e.to_end().name())?)?;
                    }
                    STORAGE_CLASS => {
                        self.set_storage_class(&reader.read_text(e.to_end().name())?)?;
                    }
                    _ => (),
                },
                Ok(Event::Eof) => {
                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(InnerItemError::from(e));
                }
                _ => (), //
            }
            buf.clear();
        }
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

/// 将 object 列表写入到 rust 类型
pub trait RefineObjectList<T, Error, ItemErr = Error>
where
    T: RefineObject<ItemErr>,
    Error: ListError,
    ItemErr: StdError + 'static,
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
    fn set_common_prefix(&mut self, _list: &[Cow<'_, str>]) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 max_keys
    fn set_max_keys(&mut self, _max_keys: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 key_count
    fn set_key_count(&mut self, _key_count: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取翻页信息，有下一页，返回 Some, 否则返回 None
    #[deprecated(
        since = "0.12.0",
        note = "Option is redundant, replace with set_next_continuation_token_str"
    )]
    fn set_next_continuation_token(&mut self, _token: Option<&str>) -> Result<(), Error> {
        Ok(())
    }

    /// 提取翻页信息 token
    fn set_next_continuation_token_str(&mut self, _token: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 object 列表
    fn set_list(&mut self, _list: Vec<T>) -> Result<(), Error> {
        Ok(())
    }

    /// 用于解析 common prefix
    fn decode_common_prefix(&mut self, xml: &str) -> Result<(), InnerListError> {
        let mut reader = Reader::from_str(xml);
        let mut buf = Vec::with_capacity(xml.len());
        let mut prefix_vec = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    if e.name().as_ref() == PREFIX {
                        prefix_vec.push(reader.read_text(e.to_end().name())?);
                    }
                }
                Ok(Event::Eof) => {
                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(InnerListError::from(e));
                }
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }
        self.set_common_prefix(&prefix_vec)?;

        Ok(())
    }

    /// # 由 xml 转 struct 的底层实现
    /// - `init_object` 用于初始化 object 结构体的方法
    fn decode<F>(&mut self, xml: &str, mut init_object: F) -> Result<(), InnerListError>
    where
        F: FnMut() -> T,
    {
        //println!("from_xml: {:#}", xml);
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::with_capacity(xml.len());

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        COMMON_PREFIX => {
                            self.decode_common_prefix(&reader.read_text(e.to_end().name())?)?;
                        }
                        PREFIX => {
                            self.set_prefix(&reader.read_text(e.to_end().name())?)?;
                        }
                        NAME => self.set_name(&reader.read_text(e.to_end().name())?)?,
                        MAX_KEYS => self.set_max_keys(&reader.read_text(e.to_end().name())?)?,
                        KEY_COUNT => self.set_key_count(&reader.read_text(e.to_end().name())?)?,
                        IS_TRUNCATED => {
                            //is_truncated = reader.read_text(e.to_end().name())?.to_string() == "true"
                        }
                        NEXT_CONTINUATION_TOKEN => {
                            let next_continuation_token = reader.read_text(e.to_end().name())?;
                            self.set_next_continuation_token_str(&next_continuation_token)?;
                            self.set_next_continuation_token(
                                if !next_continuation_token.is_empty() {
                                    Some(&next_continuation_token)
                                } else {
                                    None
                                },
                            )?;
                        }
                        CONTENTS => {
                            // <Contents></Contents> 标签内部的数据对应单个 object 信息
                            let mut object = init_object();
                            object.decode(&reader.read_text(e.to_end().name())?)?;
                            result.push(object);
                        }
                        _ => (),
                    }
                }
                Ok(Event::Eof) => {
                    self.set_list(result)?;
                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(InnerListError::from(e));
                }
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }

        Ok(())
    }
}

/// 将一个 bucket 的数据写入到 rust 类型
pub trait RefineBucket<Error: StdError + 'static> {
    /// 提取 bucket name
    fn set_name(&mut self, _name: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 bucket 创建时间
    fn set_creation_date(&mut self, _creation_date: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 location
    fn set_location(&mut self, _location: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 extranet_endpoint
    fn set_extranet_endpoint(&mut self, _extranet_endpoint: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 intranet_endpoint
    fn set_intranet_endpoint(&mut self, _intranet_endpoint: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 storage_class
    fn set_storage_class(&mut self, _storage_class: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 解析 OSS 接口返回的 xml 数据
    fn decode(&mut self, xml: &str) -> Result<(), InnerItemError> {
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
                    return Err(InnerItemError::from(e));
                }
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }
        Ok(())
    }
}

const TRUE: &str = "true";

/// 将 bucket 列表的数据写入到 rust 类型
pub trait RefineBucketList<T: RefineBucket<ItemErr>, Error, ItemErr = Error>
where
    Error: ListError,
    ItemErr: StdError + 'static,
{
    /// 提取 prefix
    fn set_prefix(&mut self, _prefix: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 marker
    fn set_marker(&mut self, _marker: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 max_keys
    fn set_max_keys(&mut self, _max_keys: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 is_truncated
    fn set_is_truncated(&mut self, _is_truncated: bool) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 next_marker
    fn set_next_marker(&mut self, _next_marker: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 id
    fn set_id(&mut self, _id: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 display_name
    fn set_display_name(&mut self, _display_name: &str) -> Result<(), Error> {
        Ok(())
    }

    /// 提取 bucket 列表
    fn set_list(&mut self, _list: Vec<T>) -> Result<(), Error> {
        Ok(())
    }

    /// 解析 OSS 接口返回的 xml 数据
    fn decode<F>(&mut self, xml: &str, mut init_bucket: F) -> Result<(), InnerListError>
    where
        F: FnMut() -> T,
    {
        let mut result = Vec::new();
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        let mut buf = Vec::with_capacity(xml.len());

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
                    BUCKET => {
                        // <Bucket></Bucket> 标签内部的数据对应单个 bucket 信息
                        let mut bucket = init_bucket();
                        bucket.decode(&reader.read_text(e.to_end().name())?)?;
                        result.push(bucket);
                    }
                    _ => (),
                },
                Ok(Event::Eof) => {
                    self.set_list(result)?;
                    break;
                } // exits the loop when reaching end of file
                Err(e) => {
                    return Err(InnerListError::from(e));
                }
                _ => (), // There are several other `Event`s we do not consider here
            }
            buf.clear();
        }
        Ok(())
    }
}

/// # Object 的 Error 中间层
/// 当外部实现 [`RefineObject`] 时，所使用的 Error ,可先转换为这个，
/// 变成一个已知的 Error 类型
///
/// [`RefineObject`]: crate::decode::RefineObject
#[derive(Debug)]
#[doc(hidden)]
pub struct InnerItemError(Box<dyn StdError + 'static>);

impl<T: StdError + 'static> From<T> for InnerItemError {
    fn from(err: T) -> Self {
        Self(Box::new(err))
    }
}

impl Display for InnerItemError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.0)
    }
}

impl InnerItemError {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        #[derive(Debug)]
        struct MyError;
        impl Display for MyError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                "demo".fmt(f)
            }
        }
        impl StdError for MyError {}

        Self(Box::new(MyError {}))
    }

    pub fn get_source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.0.as_ref())
    }
}

/// 当外部要实现 [`RefineObjectList`] 时，Error 类需要实现此 Trait
///
/// [`RefineObjectList`]: crate::decode::RefineObjectList
pub trait ListError: StdError + 'static {}

impl ListError for ParseIntError {}

impl ListError for InvalidEndPoint {}

#[cfg(feature = "core")]
impl ListError for InvalidObjectPath {}
#[cfg(feature = "core")]
impl ListError for InvalidObjectDir {}
#[cfg(feature = "core")]
impl ListError for chrono::ParseError {}
#[cfg(feature = "core")]
impl ListError for OssError {}

impl<T: ListError> From<T> for InnerListError {
    fn from(err: T) -> InnerListError {
        Self {
            kind: ListErrorKind::Custom(Box::new(err)),
        }
    }
}

/// # ObjectList 的 Error 中间层
/// 当外部实现 [`RefineObjectList`] 时，所使用的 Error ,可先转换为这个，
/// 变成一个已知的 Error 类型
///
/// [`RefineObjectList`]: crate::decode::RefineObjectList
#[derive(Debug)]
pub struct InnerListError {
    kind: ListErrorKind,
}

impl Display for InnerListError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use ListErrorKind::*;
        match &self.kind {
            Item(item) => write!(fmt, "{}", item.0),
            Xml(xml) => write!(fmt, "{xml}"),
            Custom(out) => write!(fmt, "{out}"),
        }
    }
}

impl InnerListError {
    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn from_xml() -> Self {
        Self {
            kind: ListErrorKind::Xml(quick_xml::Error::TextNotFound),
        }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn from_custom() -> Self {
        #[derive(Debug)]
        struct MyError;
        impl Display for MyError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                "custom".fmt(f)
            }
        }
        impl StdError for MyError {}
        Self {
            kind: ListErrorKind::Custom(Box::new(MyError {})),
        }
    }

    /// 获取更详细的错误信息
    pub fn get_source(&self) -> Option<&(dyn StdError + 'static)> {
        use ListErrorKind::*;
        match &self.kind {
            Item(item) => item.get_source(),
            Xml(xml) => Some(xml),
            Custom(out) => Some(out.as_ref()),
        }
    }
}

impl From<InnerItemError> for InnerListError {
    fn from(value: InnerItemError) -> Self {
        Self {
            kind: ListErrorKind::Item(value),
        }
    }
}

impl From<quick_xml::Error> for InnerListError {
    fn from(value: quick_xml::Error) -> Self {
        Self {
            kind: ListErrorKind::Xml(value),
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
#[non_exhaustive]
pub enum ListErrorKind {
    #[non_exhaustive]
    Item(InnerItemError),

    #[non_exhaustive]
    Xml(quick_xml::Error),

    #[non_exhaustive]
    Custom(Box<dyn StdError + 'static>),
}
