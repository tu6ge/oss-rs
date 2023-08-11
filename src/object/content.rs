//! # 读写 object 内容 （实现 std Write/Read）
//!
//! ## 写入数据
//! ```rust,no_run
//! # use aliyun_oss_client::{Client, object::Content};
//! # use std::sync::Arc;
//! use std::io::Write;
//! fn main() -> std::io::Result<()> {
//!     dotenv::dotenv().ok();
//!     let client = Client::from_env().unwrap();
//!
//!     let mut object = Content::from_client(Arc::new(client)).path("path1.txt").unwrap();
//!     object.write(b"abc")?;
//!     object.flush();
//!
//!     Ok(())
//! }
//! ```
//!
//! ## 读取数据
//! ```rust,no_run
//! # use aliyun_oss_client::{Client, Query};
//! use aliyun_oss_client::object::content::List;
//! use std::io::Read;
//! #[tokio::main]
//! async fn main() -> std::io::Result<()> {
//!     dotenv::dotenv().ok();
//!     let client = Client::from_env().unwrap();
//!
//!     let list: List = client
//!         .get_custom_object(&Query::default())
//!         .await
//!         .unwrap();
//!     let mut vec = list.to_vec();
//!     let mut object = &mut vec[0];
//!     let mut buffer = [0; 10];
//!     object.read(&mut buffer)?;
//!
//!     println!("{:?}", buffer);
//!
//!     Ok(())
//! }
//! ```

use std::{
    error::Error,
    fmt::Display,
    io::{Read, Result as IoResult, Seek, SeekFrom, Write},
    ops::{Deref, DerefMut},
    sync::Arc,
};

use futures::executor::block_on;
use http::{header::CONTENT_LENGTH, HeaderValue, Method};
use url::Url;

use crate::{
    builder::BuilderError,
    decode::RefineObject,
    file::{AlignBuilder, DEFAULT_CONTENT_TYPE},
    types::{
        object::{InvalidObjectPath, SetObjectPath},
        CanonicalizedResource,
    },
    Client, ObjectPath,
};

use super::{BuildInItemError, InitObject, Objects};

#[cfg(feature = "blocking")]
pub mod blocking;

#[cfg(not(test))]
use crate::file::Files;
#[cfg(test)]
use mock::Files;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test_suite;
#[cfg(all(test, feature = "blocking"))]
mod test_suite_block;

/// # object 内容
/// [OSS 分片上传文档](https://help.aliyun.com/zh/oss/user-guide/multipart-upload-12)
#[derive(Default)]
pub struct Content {
    client: Arc<Client>,
    inner: Inner,
}

/// # 内部
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inner {
    path: ObjectPath,
    content_size: u64,
    current_pos: u64,
    content_part: Vec<Vec<u8>>,
    content_type: &'static str,
    upload_id: String,
    /// 分片上传返回的 etag
    etag_list: Vec<(u16, HeaderValue)>,
    part_size: usize,
}

impl Write for Content {
    // 写入缓冲区
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.inner.write(buf)
    }

    // 按分片数量选择上传 OSS 的方式
    fn flush(&mut self) -> IoResult<()> {
        let len = self.content_part.len();

        //println!("len: {}", len);

        if len == 0 {
            return Ok(());
        }
        if len == 1 {
            return block_on(self.upload());
        }

        block_on(self.upload_multi())
    }
}

impl Read for Content {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let len = buf.len();
        if len as u64 > Inner::MAX_SIZE {
            return Err(ContentError::new(ContentErrorKind::OverflowMaxSize).into());
        }

        let end = self.current_pos + len as u64;
        let vec = block_on(
            self.client
                .get_object(self.path.clone(), self.current_pos..end - 1),
        )?;

        let len = vec.len().min(buf.len());
        buf[..len].copy_from_slice(&vec[..len]);

        Ok(len)
    }
}

impl Seek for Content {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        self.inner.seek(pos)
    }
}

impl Deref for Content {
    type Target = Inner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Content {
    fn deref_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }
}

/// 带内容的 object 列表
pub type List = Objects<Content>;

impl InitObject<Content> for List {
    fn init_object(&mut self) -> Option<Content> {
        Some(Content {
            client: self.client(),
            ..Default::default()
        })
    }
}

impl Content {
    /// 从 client 创建
    pub fn from_client(client: Arc<Client>) -> Content {
        Content {
            client,
            ..Default::default()
        }
    }
    /// 设置 ObjectPath
    pub fn path<P>(mut self, path: P) -> Result<Self, InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        P::Error: Into<InvalidObjectPath>,
    {
        self.path = path.try_into().map_err(Into::into)?;
        self.content_type_with_path();
        Ok(self)
    }

    fn part_canonicalized(&self, query: &str) -> (Url, CanonicalizedResource) {
        let mut url = self.client.get_bucket_url();
        url.set_object_path(&self.path);
        url.set_query(Some(query));

        let bucket = self.client.get_bucket_name();
        (
            url,
            CanonicalizedResource::new(format!("/{}/{}?{}", bucket, self.path.as_ref(), query)),
        )
    }
    async fn upload(&mut self) -> IoResult<()> {
        let content = self.content_part.pop().expect("content_part len is not 1");
        self.client
            .put_content_base(content, self.content_type, self.path.clone())
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn upload_multi(&mut self) -> IoResult<()> {
        self.init_multi().await?;

        let mut i = 1;
        let mut size: u64 = 0;
        self.content_part.reverse();
        while let Some(item) = self.content_part.pop() {
            size += item.len() as u64;
            self.upload_part(i, item).await?;
            i += 1;
        }

        self.complete_multi().await?;
        self.content_size = size;
        Ok(())
    }

    /// 初始化批量上传
    async fn init_multi(&mut self) -> Result<(), ContentError> {
        const UPLOADS: &str = "uploads";

        let (url, resource) = self.part_canonicalized(UPLOADS);
        let xml = self
            .client
            .builder(Method::POST, url, resource)?
            .send_adjust_error()
            .await?
            .text()
            .await?;

        self.parse_upload_id(&xml)
    }
    /// 上传分块
    async fn upload_part(&mut self, index: u16, buf: Vec<u8>) -> Result<(), ContentError> {
        const ETAG: &str = "ETag";

        if self.upload_id.is_empty() {
            return Err(ContentError::new(ContentErrorKind::NoFoundUploadId));
        }

        if self.etag_list.len() >= Inner::MAX_PARTS_COUNT as usize {
            return Err(ContentError::new(ContentErrorKind::OverflowMaxPartsCount));
        }
        if buf.len() > Inner::PART_SIZE_MAX {
            return Err(ContentError::new(ContentErrorKind::OverflowPartSize));
        }

        let (url, resource) =
            self.part_canonicalized(&format!("partNumber={}&uploadId={}", index, self.upload_id));

        let content_length = buf.len().to_string();
        let headers = vec![(
            CONTENT_LENGTH,
            HeaderValue::from_str(&content_length)
                .expect("content length must be a valid header value"),
        )];

        let resp = self
            .client
            .builder_with_header(Method::PUT, url, resource, headers)?
            .body(buf)
            .send_adjust_error()
            .await?;

        let etag = resp
            .headers()
            .get(ETAG)
            .ok_or(ContentError::new(ContentErrorKind::NoFoundEtag))?;

        //println!("etag: {:?}", etag);

        // 59A2A10DD1686F679EE885FC1EBA5183
        //let etag = &(etag.to_str().unwrap())[1..33];

        self.etag_list.push((index, etag.to_owned()));

        Ok(())
    }
    /// 完成分块上传
    async fn complete_multi(&mut self) -> Result<(), ContentError> {
        if self.upload_id.is_empty() {
            return Err(ContentError::new(ContentErrorKind::NoFoundUploadId));
        }

        let xml = self.etag_list_xml()?;

        let (url, resource) = self.part_canonicalized(&format!("uploadId={}", self.upload_id));

        let content_length = xml.len().to_string();
        let headers = vec![(
            CONTENT_LENGTH,
            HeaderValue::from_str(&content_length)
                .expect("content length must be a valid header value"),
        )];

        let _resp = self
            .client
            .builder_with_header(Method::POST, url, resource, headers)?
            .body(xml)
            .send_adjust_error()
            .await?;

        self.etag_list.clear();
        self.upload_id = String::default();

        Ok(())
    }
    /// 取消分块上传
    pub async fn abort_multi(&mut self) -> Result<(), ContentError> {
        if self.upload_id.is_empty() {
            return Err(ContentError::new(ContentErrorKind::NoFoundUploadId));
        }
        let query = format!("uploadId={}", self.upload_id);

        let (url, resource) = self.part_canonicalized(&query);
        let _resp = self
            .client
            .builder(Method::DELETE, url, resource)?
            .send_adjust_error()
            .await?;

        //println!("resp: {:?}", resp);
        self.etag_list.clear();
        self.upload_id = String::default();

        Ok(())
    }
}

impl Seek for Inner {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        use std::io::{Error, ErrorKind};
        let n = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::End(p) => {
                if self.content_size == 0 {
                    return Err(Error::new(ErrorKind::InvalidData, "content size is 0"));
                }
                // self.content_size - p
                i64::try_from(self.content_size)
                    .and_then(|v| u64::try_from(v - p))
                    .map_err(|e| Error::new(ErrorKind::InvalidData, e))?
            }
            // self.current_pos + n
            SeekFrom::Current(n) => i64::try_from(self.current_pos)
                .and_then(|v| u64::try_from(v + n))
                .map_err(|e| Error::new(ErrorKind::InvalidData, e))?,
        };
        self.current_pos = n;
        Ok(n)
    }
}

// impl Drop for Content {
//     fn drop(&mut self) {
//         if self.upload_id.is_empty() == false {
//             self.abort_multi();
//         }
//     }
// }

mod private {
    use super::Content;

    pub trait Sealed {}

    impl Sealed for Content {}

    #[cfg(feature = "blocking")]
    impl Sealed for super::blocking::Content {}
}

impl<T: DerefMut<Target = Inner> + private::Sealed> RefineObject<BuildInItemError> for T {
    #[inline]
    fn set_key(&mut self, key: &str) -> Result<(), BuildInItemError> {
        self.path = key.parse().map_err(|e| BuildInItemError::new(e, key))?;
        self.content_type_with_path();
        Ok(())
    }
    /// 提取 size
    fn set_size(&mut self, size: &str) -> Result<(), BuildInItemError> {
        if let Ok(size) = size.parse() {
            self.content_size = size;
        }
        Ok(())
    }
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            path: ObjectPath::default(),
            content_size: u64::default(),
            current_pos: 0,
            content_part: Vec::default(),
            content_type: Self::DEFAULT_CONTENT_TYPE,
            upload_id: String::default(),
            etag_list: Vec::default(),
            part_size: 200 * 1024 * 1024, // 200M
        }
    }
}

fn get_content_type(filename: &str) -> &'static str {
    match filename.rsplit('.').next() {
        Some(str) => match str.to_lowercase().as_str() {
            "jpg" => "image/jpeg",
            "pdf" => "application/pdf",
            "png" => "image/png",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "gz" => "application/gzip",
            "txt" => "text/plain",
            "mp3" => "audio/mpeg",
            "wav" => "audio/wave",
            "mp4" => "video/mp4",
            "mov" => "video/quicktime",
            "avi" => "video/x-msvideo",
            "wmv" => "video/x-ms-wmv",
            "html" => "text/html",
            "js" => "application/javascript",
            "css" => "text/css",
            "php" => "application/x-httpd-php",
            _ => DEFAULT_CONTENT_TYPE,
        },
        None => DEFAULT_CONTENT_TYPE,
    }
}

impl Inner {
    const DEFAULT_CONTENT_TYPE: &str = DEFAULT_CONTENT_TYPE;

    /// 最大存储容量 48.8 TB, 49664 = 1024 * 48.5
    #[cfg(not(test))]
    const MAX_SIZE: u64 = 1024 * 1024 * 1024 * 49_664;
    /// 最大 part 数量
    #[cfg(not(test))]
    const MAX_PARTS_COUNT: u16 = 10000;
    /// 单个 part 的最小尺寸 100K
    #[cfg(not(test))]
    const PART_SIZE_MIN: usize = 102400;
    /// 单个 part 的最大尺寸 5G
    #[cfg(not(test))]
    const PART_SIZE_MAX: usize = 1024 * 1024 * 1024 * 5;

    /// 最大存储容量 48.8 TB, 49664 = 1024 * 48.5
    #[cfg(test)]
    const MAX_SIZE: u64 = 200;
    /// 最大 part 数量
    #[cfg(test)]
    const MAX_PARTS_COUNT: u16 = 10;
    /// 单个 part 的最小尺寸 100K
    #[cfg(test)]
    const PART_SIZE_MIN: usize = 10;
    /// 单个 part 的最大尺寸 5G
    #[cfg(test)]
    const PART_SIZE_MAX: usize = 20;

    /// 设置 ObjectPath
    pub fn path<P>(mut self, path: P) -> Result<Self, InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        P::Error: Into<InvalidObjectPath>,
    {
        self.path = path.try_into().map_err(Into::into)?;
        self.content_type_with_path();
        Ok(self)
    }

    fn content_type_with_path(&mut self) {
        self.content_type = get_content_type(self.path.as_ref());
    }

    /// 设置 content_type
    pub fn content_type(&mut self, content_type: &'static str) {
        self.content_type = content_type;
    }

    // 写入缓冲区
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        let part_total = self.content_part.len();
        if part_total >= Inner::MAX_PARTS_COUNT as usize {
            return Err(ContentError::new(ContentErrorKind::OverflowMaxPartsCount).into());
        }

        let part_size = self.part_size;
        if let Some(part) = self.content_part.last_mut() {
            let part_len = part.len();
            if part_len < part_size {
                let mid = part_size - part_len;
                let left = &buf[..mid.min(buf.len())];

                part.append(&mut left.to_vec());

                return Ok(left.len());
            }
        }

        let con = &buf[..buf.len().min(self.part_size)];
        self.content_part.push({
            let mut vec = Vec::with_capacity(self.part_size);
            vec.extend(con);
            vec
        });

        Ok(con.len())
    }

    /// 设置分块的尺寸
    pub fn part_size(&mut self, size: usize) -> Result<(), ContentError> {
        if (Self::PART_SIZE_MIN..=Self::PART_SIZE_MAX).contains(&size) {
            self.part_size = size;
            Ok(())
        } else {
            Err(ContentError::new(ContentErrorKind::OverflowPartSize))
        }
    }
    fn parse_upload_id(&mut self, xml: &str) -> Result<(), ContentError> {
        if let (Some(start), Some(end)) = (xml.find("<UploadId>"), xml.find("</UploadId>")) {
            self.upload_id = (&xml[start + 10..end]).to_owned();
            Ok(())
        } else {
            Err(ContentError::new(ContentErrorKind::NoFoundUploadId))
        }
    }
    fn etag_list_xml(&self) -> Result<String, ContentError> {
        if self.etag_list.is_empty() {
            return Err(ContentError::new(ContentErrorKind::EtagListEmpty));
        }
        let mut list = String::new();
        for (index, etag) in self.etag_list.iter() {
            list.push_str(&format!(
                "<Part><PartNumber>{}</PartNumber><ETag>{}</ETag></Part>",
                index,
                etag.to_str().expect("etag covert str failed")
            ));
        }

        Ok(format!(
            "<CompleteMultipartUpload>{}</CompleteMultipartUpload>",
            list
        ))
    }

    /// 清空缓冲区
    pub fn part_clear(&mut self) {
        self.content_part.clear();
    }
}

impl From<Client> for Content {
    fn from(value: Client) -> Self {
        Content {
            client: Arc::new(value),
            ..Default::default()
        }
    }
}

/// object Content 的错误信息
#[derive(Debug)]
#[non_exhaustive]
pub struct ContentError {
    kind: ContentErrorKind,
}

/// object Content 的错误信息
#[derive(Debug)]
#[non_exhaustive]
enum ContentErrorKind {
    /// not found upload id
    NoFoundUploadId,

    /// builder request failed
    Builder(BuilderError),

    /// not found etag
    NoFoundEtag,

    /// overflow max parts count
    OverflowMaxPartsCount,

    /// etag list is empty
    EtagListEmpty,

    /// part size must be between 100k and 5G
    OverflowPartSize,

    /// max size must be lt 48.8TB
    OverflowMaxSize,
}

impl ContentError {
    fn new(kind: ContentErrorKind) -> Self {
        Self { kind }
    }
}

impl Display for ContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

impl Error for ContentError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.kind.source()
    }
}

impl From<BuilderError> for ContentError {
    fn from(value: BuilderError) -> Self {
        Self {
            kind: ContentErrorKind::Builder(value),
        }
    }
}
impl From<reqwest::Error> for ContentError {
    fn from(value: reqwest::Error) -> Self {
        Self {
            kind: ContentErrorKind::Builder(BuilderError::from_reqwest(value)),
        }
    }
}
impl From<ContentError> for std::io::Error {
    fn from(ContentError { kind }: ContentError) -> Self {
        use std::io::ErrorKind::*;
        use ContentErrorKind::*;
        match kind {
            Builder(e) => e.into(),
            NoFoundUploadId => Self::new(NotFound, kind),
            NoFoundEtag => Self::new(NotFound, kind),
            OverflowMaxPartsCount => Self::new(InvalidInput, kind),
            EtagListEmpty => Self::new(NotFound, kind),
            OverflowPartSize => Self::new(Unsupported, kind),
            OverflowMaxSize => Self::new(Unsupported, kind),
        }
    }
}

impl Display for ContentErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::NoFoundUploadId => "not found upload id".fmt(f),
            Self::Builder(_) => "builder request failed".fmt(f),
            Self::NoFoundEtag => "not found etag".fmt(f),
            Self::OverflowMaxPartsCount => "overflow max parts count".fmt(f),
            Self::EtagListEmpty => "etag list is empty".fmt(f),
            Self::OverflowPartSize => "part size must be between 100k and 5G".fmt(f),
            Self::OverflowMaxSize => "max size must be lt 48.8TB".fmt(f),
        }
    }
}

impl Error for ContentErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Builder(e) => Some(e),
            Self::NoFoundUploadId
            | Self::NoFoundEtag
            | Self::OverflowMaxPartsCount
            | Self::EtagListEmpty
            | Self::OverflowPartSize
            | Self::OverflowMaxSize => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Seek, Write},
        ops::Deref,
        sync::Arc,
    };

    use super::{
        get_content_type,
        test_suite::{AbortMulti, CompleteMulti, InitMulti, UploadMulti, UploadPart},
        Content, Inner, List,
    };

    use crate::{decode::RefineObject, object::InitObject, Client, ObjectPath};

    #[test]
    fn default() {
        let inner = Inner::default();

        assert_eq!(inner.path, ObjectPath::default());
        assert_eq!(inner.content_size, 0);
        assert_eq!(inner.current_pos, 0);
        assert_eq!(inner.content_part.len(), 0);
        assert_eq!(inner.content_type, "application/octet-stream");
        assert_eq!(inner.upload_id, "");
        assert_eq!(inner.etag_list.len(), 0);
        assert_eq!(inner.part_size, 200 * 1024 * 1024);
    }

    #[test]
    fn read() {
        let client = Client::test_init();
        let mut con = Content::from_client(Arc::new(client))
            .path("aaa.txt")
            .unwrap();

        let mut buf = [0u8; 201];
        let err = con.read(&mut buf).unwrap_err();
        assert_eq!(err.to_string(), "max size must be lt 48.8TB");

        let mut buf = [0u8; 10];
        let len = con.read(&mut buf).unwrap();
        assert_eq!(buf, [1u8, 2, 3, 4, 5, 0, 0, 0, 0, 0]);
        assert_eq!(len, 5);

        let mut buf = [0u8; 3];
        let len = con.read(&mut buf).unwrap();
        assert_eq!(buf, [1u8, 2, 3]);
        assert_eq!(len, 3);

        con.current_pos = 10;
        let mut buf = [0u8; 3];
        let len = con.read(&mut buf).unwrap();
        assert_eq!(buf, [1u8, 2, 3]);
        assert_eq!(len, 3);
    }

    #[test]
    fn init_object() {
        let mut list = List::default();
        let client = Client::test_init();
        list.set_client(Arc::new(client.clone()));

        let con = list.init_object().unwrap();

        assert_eq!(con.client.bucket, client.bucket);
        assert_eq!(con.inner, Inner::default());
    }

    #[test]
    fn from_client() {
        let client = Client::test_init();

        let con = Content::from_client(Arc::new(client.clone()));

        assert_eq!(con.client.bucket, client.bucket);
        assert_eq!(con.inner, Inner::default());
    }

    #[test]
    fn path() {
        let con = Content::default().path("aaa.txt").unwrap();
        assert_eq!(con.path, "aaa.txt");
    }

    #[test]
    fn part_canonicalized() {
        let client = Client::test_init();
        let con = Content::from_client(Arc::new(client))
            .path("aaa.txt")
            .unwrap();

        let (url, can) = con.part_canonicalized("first=1&second=2");
        assert_eq!(
            url.as_str(),
            "https://bar.oss-cn-qingdao.aliyuncs.com/aaa.txt?first=1&second=2"
        );
        assert_eq!(can.to_string(), "/bar/aaa.txt?first=1&second=2");
    }

    #[tokio::test]
    async fn upload() {
        let client = Client::test_init();
        let mut con = Content::from_client(Arc::new(client))
            .path("aaa.txt")
            .unwrap();
        con.content_part.push(b"bbb".to_vec());
        con.upload().await.unwrap();
    }

    #[tokio::test]
    async fn init_multi() {
        let client = Client::test_init().middleware(Arc::new(InitMulti {}));
        let mut con = Content::from_client(Arc::new(client))
            .path("aaa.txt")
            .unwrap();

        con.init_multi().await.unwrap();

        assert_eq!(con.upload_id, "foo_upload_id");
    }

    #[tokio::test]
    async fn upload_part() {
        let client = Client::test_init().middleware(Arc::new(UploadPart {}));
        let mut con = Content::from_client(Arc::new(client))
            .path("aaa.txt")
            .unwrap();

        let err = con.upload_part(1, b"bbb".to_vec()).await.unwrap_err();
        assert_eq!(err.to_string(), "not found upload id");

        con.upload_id = "foo_upload_id".to_string();
        for _i in 0..10 {
            con.etag_list.push((1, "a".parse().unwrap()));
        }
        let err = con.upload_part(1, b"bbb".to_vec()).await.unwrap_err();
        assert_eq!(err.to_string(), "overflow max parts count");
        con.etag_list.clear();

        let err = con
            .upload_part(1, b"012345678901234567890".to_vec())
            .await
            .unwrap_err();
        assert_eq!(err.to_string(), "part size must be between 100k and 5G");

        con.upload_part(2, b"bbb".to_vec()).await.unwrap();
        let (index, value) = con.etag_list.pop().unwrap();
        assert_eq!(index, 2);
        assert_eq!(value.to_str().unwrap(), "foo_etag");
    }

    #[tokio::test]
    async fn complete_multi() {
        let client = Client::test_init().middleware(Arc::new(CompleteMulti {}));
        let mut con = Content::from_client(Arc::new(client))
            .path("aaa.txt")
            .unwrap();
        let err = con.complete_multi().await.unwrap_err();
        assert_eq!(err.to_string(), "not found upload id");

        con.upload_id = "foo_upload_id".to_string();
        con.etag_list.push((1, "aaa".parse().unwrap()));
        con.etag_list.push((2, "bbb".parse().unwrap()));
        con.complete_multi().await.unwrap();
        assert!(con.etag_list.is_empty());
        assert!(con.upload_id.is_empty());
    }

    #[tokio::test]
    async fn upload_multi() {
        let client = Client::test_init().middleware(Arc::new(UploadMulti {}));
        let mut con = Content::from_client(Arc::new(client))
            .path("aaa.txt")
            .unwrap();

        con.content_part.push(b"aaa".to_vec());
        con.content_part.push(b"bbb".to_vec());

        con.upload_multi().await.unwrap();

        assert_eq!(con.content_size, 6);
    }

    #[tokio::test]
    async fn abort_multi() {
        let client = Client::test_init().middleware(Arc::new(AbortMulti {}));
        let mut con = Content::from_client(Arc::new(client))
            .path("aaa.txt")
            .unwrap();
        let err = con.complete_multi().await.unwrap_err();
        assert_eq!(err.to_string(), "not found upload id");

        con.upload_id = "foo_upload_id".to_string();
        con.etag_list.push((1, "aaa".parse().unwrap()));
        con.abort_multi().await.unwrap();
        assert!(con.etag_list.is_empty());
        assert!(con.upload_id.is_empty());
    }

    #[test]
    fn seek() {
        let mut inner = Inner::default();
        let pos = inner.seek(std::io::SeekFrom::Start(5)).unwrap();
        assert_eq!(pos, 5);
        assert_eq!(inner.current_pos, 5);

        let err = inner.seek(std::io::SeekFrom::End(10)).unwrap_err();
        assert_eq!(err.to_string(), "content size is 0");

        inner.content_size = 11;
        let pos = inner.seek(std::io::SeekFrom::End(5)).unwrap();
        assert_eq!(pos, 6);
        inner.current_pos = 6;

        let pos = inner.seek(std::io::SeekFrom::Current(-1)).unwrap();
        assert_eq!(pos, 5);
        inner.current_pos = 5;
    }

    #[test]
    fn test_get_content_type() {
        assert_eq!(get_content_type("aaa.jpg"), "image/jpeg");
        assert_eq!(get_content_type("aaa.pdf"), "application/pdf");
        assert_eq!(get_content_type("aaa.png"), "image/png");
        assert_eq!(get_content_type("aaa.gif"), "image/gif");
        assert_eq!(get_content_type("aaa.bmp"), "image/bmp");
        assert_eq!(get_content_type("aaa.zip"), "application/zip");
        assert_eq!(get_content_type("aaa.tar"), "application/x-tar");
        assert_eq!(get_content_type("aaa.gz"), "application/gzip");
        assert_eq!(get_content_type("aaa.txt"), "text/plain");
        assert_eq!(get_content_type("aaa.mp3"), "audio/mpeg");
        assert_eq!(get_content_type("aaa.wav"), "audio/wave");
        assert_eq!(get_content_type("aaa.mp4"), "video/mp4");
        assert_eq!(get_content_type("aaa.mov"), "video/quicktime");
        assert_eq!(get_content_type("aaa.avi"), "video/x-msvideo");
        assert_eq!(get_content_type("aaa.wmv"), "video/x-ms-wmv");
        assert_eq!(get_content_type("aaa.html"), "text/html");
        assert_eq!(get_content_type("aaa.js"), "application/javascript");
        assert_eq!(get_content_type("aaa.css"), "text/css");
        assert_eq!(get_content_type("aaa.php"), "application/x-httpd-php");
        assert_eq!(get_content_type("aaa.doc"), "application/octet-stream");
        assert_eq!(get_content_type("file"), "application/octet-stream");
    }

    #[test]
    fn test_path() {
        let mut inner = Inner::default();
        inner = inner.path("bbb.txt").unwrap();
        assert_eq!(inner.path.as_ref(), "bbb.txt");
        assert_eq!(inner.content_type, "text/plain");
    }

    #[test]
    fn content_type_with_path() {
        let mut inner = Inner::default();
        inner.path = "ccc.html".parse().unwrap();
        inner.content_type_with_path();
        assert_eq!(inner.content_type, "text/html");
    }

    #[test]
    fn test_content_type() {
        let mut inner = Inner::default();
        inner.content_type("bar");
        assert_eq!(inner.content_type, "bar");
    }

    #[test]
    fn part_size() {
        let mut inner = Inner::default();
        let err = inner.part_size(5).unwrap_err();
        assert_eq!(err.to_string(), "part size must be between 100k and 5G");

        let err = inner.part_size(21).unwrap_err();
        assert_eq!(err.to_string(), "part size must be between 100k and 5G");

        inner.part_size(11).unwrap();
        assert_eq!(inner.part_size, 11);
    }

    #[test]
    fn test_parse_upload_id() {
        let mut content = Inner::default();

        let xml = r#"<InitiateMultipartUploadResult>
        <Bucket>bucket_name</Bucket>
        <Key>aaa</Key>
        <UploadId>AC3251A13464411D8691F271CE33A300</UploadId>
      </InitiateMultipartUploadResult>"#;
        content.parse_upload_id(xml).unwrap();

        assert_eq!(content.upload_id, "AC3251A13464411D8691F271CE33A300");

        content.parse_upload_id("abc").unwrap_err();
    }

    #[test]
    fn etag_list_xml() {
        let mut inner = Inner::default();

        let err = inner.etag_list_xml().unwrap_err();
        assert_eq!(err.to_string(), "etag list is empty");

        inner.etag_list.push((1, "aaa".parse().unwrap()));
        inner.etag_list.push((2, "bbb".parse().unwrap()));

        let xml = inner.etag_list_xml().unwrap();
        assert_eq!(xml, "<CompleteMultipartUpload><Part><PartNumber>1</PartNumber><ETag>aaa</ETag></Part><Part><PartNumber>2</PartNumber><ETag>bbb</ETag></Part></CompleteMultipartUpload>");
    }

    #[test]
    fn part_clear() {
        let mut inner = Inner::default();
        assert_eq!(inner.content_part.len(), 0);

        inner.content_part.push(vec![1u8, 2, 3]);
        inner.content_part.push(vec![1u8, 2, 3]);
        inner.part_clear();
        assert_eq!(inner.content_part.len(), 0);
    }

    #[test]
    fn assert_impl() {
        fn impl_fn<T: RefineObject<E>, E: std::error::Error + 'static>(_: T) {}

        impl_fn(Content::default());

        fn impl_deref<T: Deref<Target = Inner>>(_: T) {}

        impl_deref(Content::default());
    }

    #[test]
    fn test_write() {
        let mut content = Content::default();
        content.part_size = 5;
        content.write_all(b"abcdefg").unwrap();

        assert!(content.content_part.len() == 2);
        assert_eq!(content.content_part[0], b"abcde");
        assert_eq!(content.content_part[1], b"fg");

        content.part_clear();

        content.write(b"mn").unwrap();
        assert!(content.content_part.len() == 1);
        assert_eq!(content.content_part[0], b"mn");

        content.part_clear();

        let len = content.write(b"efghijklmn").unwrap();
        assert!(content.content_part.len() == 1);
        assert_eq!(content.content_part[0], b"efghi");
        assert_eq!(len, 5);

        content.part_clear();

        content.write(b"o").unwrap();
        content.write(b"p").unwrap();
        content.write(b"q").unwrap();
        assert!(content.content_part.len() == 1);
        assert_eq!(content.content_part[0], b"opq");
    }
}
