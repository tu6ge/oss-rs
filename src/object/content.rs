//! 读写 object 内容

use std::{
    error::Error,
    fmt::Display,
    io::{Read, Result as IoResult, Seek, SeekFrom, Write},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use http::{header::CONTENT_LENGTH, HeaderValue, Method};
use url::Url;

use crate::{
    builder::BuilderError,
    decode::RefineObject,
    file::{blocking::AlignBuilder, BlockingFiles, DEFAULT_CONTENT_TYPE},
    types::{
        object::{InvalidObjectPath, SetObjectPath},
        CanonicalizedResource,
    },
    ClientRc as Client, ObjectPath,
};

use super::{BuildInItemError, BuildInItemErrorKind, ObjectsBlocking};

/// # object 内容
/// [OSS 分片上传文档](https://help.aliyun.com/zh/oss/user-guide/multipart-upload-12)
//#[derive(Debug)]
pub struct Content {
    client: Rc<Client>,
    inner: Inner,
}

/// # 内部
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
        if self.content_part.len() >= Inner::MAX_PARTS_COUNT as usize {
            return Err(ContentError::OverflowMaxPartsCount.into());
        }
        let con = if buf.len() < self.part_size {
            &buf[..]
        } else {
            &buf[..self.part_size]
        };
        self.content_part.push(con.to_vec());

        Ok(con.len())
    }

    // 按分片数量选择上传 OSS 的方式
    fn flush(&mut self) -> IoResult<()> {
        let len = self.content_part.len();

        //println!("len: {}", len);

        if len == 0 {
            return Ok(());
        }
        if len == 1 {
            let con = self.content_part.pop().unwrap();
            return self.upload(con);
        }

        self.init_multi()?;

        let mut i = 1;
        let mut size: u64 = 0;
        self.content_part.reverse();
        while let Some(item) = self.content_part.pop() {
            size += item.len() as u64;
            self.upload_part(i, item)?;
            i += 1;
        }

        self.complete_multi()?;
        self.content_size = size;

        Ok(())
    }
}

impl Read for Content {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let len = buf.len();
        if len as u64 > Inner::MAX_SIZE {
            return Err(ContentError::OverflowMaxSize.into());
        }

        let end = self.current_pos + len as u64;
        let vec = self
            .client
            .get_object(self.path.clone(), self.current_pos..end - 1)?;

        let len = std::cmp::min(vec.len(), buf.len());
        buf[..len].copy_from_slice(&vec[..len]);

        Ok(len)
    }
}
impl Seek for Content {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        let n = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::End(p) => (self.content_size as i64 - p) as u64,
            SeekFrom::Current(n) => (self.current_pos as i64 + n) as u64,
        };
        self.current_pos = n;
        Ok(n)
    }
}

impl Default for Content {
    fn default() -> Self {
        Self {
            client: Rc::default(),
            inner: Inner::default(),
        }
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
pub type List = ObjectsBlocking<Content>;

impl Content {
    fn init_object(list: &mut List) -> Option<Content> {
        Some(Content {
            client: list.client(),
            ..Default::default()
        })
    }
    /// 从 client 创建
    pub fn from_client(client: Rc<Client>) -> Content {
        Content {
            client,
            ..Default::default()
        }
    }

    fn part_canonicalized<'q>(&self, query: &'q str) -> (Url, CanonicalizedResource) {
        let mut url = self.client.get_bucket_url();
        url.set_object_path(&self.path);
        url.set_query(Some(query));

        let bucket = self.client.get_bucket_name();
        (
            url,
            CanonicalizedResource::new(format!("/{}/{}?{}", bucket, self.path.as_ref(), query)),
        )
    }

    fn upload(&self, content: Vec<u8>) -> IoResult<()> {
        self.client
            .put_content_base(content, self.content_type, self.path.clone())
            .map(|_| ())
            .map_err(Into::into)
    }

    /// 初始化批量上传
    pub fn init_multi(&mut self) -> Result<(), ContentError> {
        const UPLOADS: &str = "uploads";

        let (url, resource) = self.part_canonicalized(UPLOADS);
        let xml = self
            .client
            .builder(Method::POST, url, resource)?
            .send_adjust_error()?
            .text()?;

        self.parse_upload_id(&xml)
    }

    /// 上传分块
    pub fn upload_part(&mut self, index: u16, buf: Vec<u8>) -> Result<(), ContentError> {
        const ETAG: &str = "ETag";

        if self.upload_id.is_empty() {
            return Err(ContentError::NoFoundUploadId);
        }

        if self.etag_list.len() >= Inner::MAX_PARTS_COUNT as usize {
            return Err(ContentError::OverflowMaxPartsCount);
        }
        if buf.len() > Inner::PART_SIZE_MAX {
            return Err(ContentError::OverflowPartSize);
        }

        let query = format!("partNumber={}&uploadId={}", index, self.upload_id);

        let (url, resource) = self.part_canonicalized(&query);

        let content_length = buf.len().to_string();
        let headers = vec![(
            CONTENT_LENGTH,
            HeaderValue::from_str(&content_length).unwrap(),
        )];

        let resp = self
            .client
            .builder_with_header(Method::PUT, url, resource, headers)?
            .body(buf)
            .send_adjust_error()?;

        let etag = resp.headers().get(ETAG).ok_or(ContentError::NoFoundEtag)?;

        //println!("etag: {:?}", etag);

        // 59A2A10DD1686F679EE885FC1EBA5183
        //let etag = &(etag.to_str().unwrap())[1..33];

        self.etag_list.push((index, etag.to_owned()));

        Ok(())
    }

    /// 完成分块上传
    pub fn complete_multi(&mut self) -> Result<(), ContentError> {
        if self.upload_id.is_empty() {
            return Err(ContentError::NoFoundUploadId);
        }

        let xml = self.etag_list_xml()?;

        let query = format!("uploadId={}", self.upload_id);

        let (url, resource) = self.part_canonicalized(&query);

        let content_length = xml.len().to_string();
        let headers = vec![(
            CONTENT_LENGTH,
            HeaderValue::from_str(&content_length).unwrap(),
        )];

        let _resp = self
            .client
            .builder_with_header(Method::POST, url, resource, headers)?
            .body(xml)
            .send_adjust_error()
            ?
            // .text()
            // .await?
            ;

        //println!("resp: {}", resp);
        self.etag_list.clear();
        self.upload_id = String::default();

        Ok(())
    }

    /// 取消分块上传
    pub fn abort_multi(&mut self) -> Result<(), ContentError> {
        if self.upload_id.is_empty() {
            return Err(ContentError::NoFoundUploadId);
        }
        let query = format!("uploadId={}", self.upload_id);

        let (url, resource) = self.part_canonicalized(&query);
        let _resp = self
            .client
            .builder(Method::DELETE, url, resource)?
            .send_adjust_error()?;

        //println!("resp: {:?}", resp);
        self.upload_id = String::default();

        Ok(())
    }
}

// impl Drop for Content {
//     fn drop(&mut self) {
//         if self.upload_id.is_empty() == false {
//             block_on(self.abort_multi());
//         }
//     }
// }

impl RefineObject<BuildInItemError> for Inner {
    #[inline]
    fn set_key(&mut self, key: &str) -> Result<(), BuildInItemError> {
        self.path = key.parse().map_err(|e| BuildInItemError {
            source: key.to_string(),
            kind: BuildInItemErrorKind::BasePath(e),
        })?;

        self.content_type_from_key(key);
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

impl Inner {
    const DEFAULT_CONTENT_TYPE: &str = DEFAULT_CONTENT_TYPE;

    /// 最大存储容量 48.8 TB, 49664 = 1024 * 48.5
    const MAX_SIZE: u64 = 1024 * 1024 * 1024 * 49_664;
    /// 最大 part 数量
    const MAX_PARTS_COUNT: u16 = 10000;
    /// 单个 part 的最小尺寸 100K
    const PART_SIZE_MIN: usize = 102400;
    /// 单个 part 的最大尺寸 5G
    const PART_SIZE_MAX: usize = 1024 * 1024 * 1024 * 5;
    /// 设置 ObjectPath
    pub fn path<P>(mut self, path: P) -> Result<Self, InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        P::Error: Into<InvalidObjectPath>,
    {
        self.path = path.try_into().map_err(Into::into)?;
        Ok(self)
    }
    fn content_type_from_key(&mut self, key: &str) {
        self.content_type = match key.rsplit(".").next() {
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

    /// 设置分块的尺寸
    pub fn part_size(&mut self, size: usize) -> Result<(), ContentError> {
        if size > Self::PART_SIZE_MAX || size < Self::PART_SIZE_MIN {
            return Err(ContentError::OverflowPartSize);
        }
        self.part_size = size;

        Ok(())
    }
    fn parse_upload_id(&mut self, xml: &str) -> Result<(), ContentError> {
        if let (Some(start), Some(end)) = (xml.find("<UploadId>"), xml.find("</UploadId>")) {
            let upload_id = &xml[start + 10..end];
            self.upload_id = upload_id.to_owned();
            //println!("upload_id {}", upload_id);
            return Ok(());
        }

        Err(ContentError::NoFoundUploadId)
    }
    fn etag_list_xml(&self) -> Result<String, ContentError> {
        if self.etag_list.is_empty() {
            return Err(ContentError::EtagListEmpty);
        }
        let mut list = String::new();
        for (index, etag) in self.etag_list.iter() {
            list.push_str(&format!(
                "<Part><PartNumber>{}</PartNumber><ETag>{}</ETag></Part>",
                index,
                etag.to_str().unwrap()
            ));
        }

        Ok(format!(
            "<CompleteMultipartUpload>{}</CompleteMultipartUpload>",
            list
        ))
    }
}

#[cfg(test)]
#[tokio::test]
async fn main() {
    dotenv::dotenv().ok();
    let client = Client::from_env().unwrap();

    // let mut list = client
    //     .get_custom_object(&Query::default(), Content::init_object)
    //     .await
    //     .unwrap();

    // let second = list.get_next_base(Content::init_object).await;

    // let mut objcet = Content::from_client(Arc::new(client)).path("aaa").unwrap();
    // let res = objcet.init_multi().await;
    // println!("{res:#?}");
}

impl From<Client> for Content {
    fn from(value: Client) -> Self {
        Content {
            client: Rc::new(value),
            ..Default::default()
        }
    }
}

/// object Content 的错误信息
#[derive(Debug)]
#[non_exhaustive]
pub enum ContentError {
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

impl Display for ContentError {
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

impl Error for ContentError {
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

impl From<BuilderError> for ContentError {
    fn from(value: BuilderError) -> Self {
        Self::Builder(value)
    }
}
impl From<reqwest::Error> for ContentError {
    fn from(value: reqwest::Error) -> Self {
        Self::Builder(BuilderError::from_reqwest(value))
    }
}
impl From<ContentError> for std::io::Error {
    fn from(value: ContentError) -> Self {
        use std::io::ErrorKind::*;
        use ContentError::*;
        match value {
            Builder(e) => e.into(),
            NoFoundUploadId => Self::new(NotFound, value),
            NoFoundEtag => Self::new(NotFound, value),
            OverflowMaxPartsCount => Self::new(InvalidInput, value),
            EtagListEmpty => Self::new(NotFound, value),
            OverflowPartSize => Self::new(Unsupported, value),
            OverflowMaxSize => Self::new(Unsupported, value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_upload_id() {
        let mut content = Content::default();

        let xml = r#"<InitiateMultipartUploadResult>
        <Bucket>honglei123</Bucket>
        <Key>aaa</Key>
        <UploadId>AC3251A13464411D8691F271CE33A300</UploadId>
      </InitiateMultipartUploadResult>"#;
        content.parse_upload_id(xml).unwrap();

        assert_eq!(content.upload_id, "AC3251A13464411D8691F271CE33A300");

        content.parse_upload_id("abc").unwrap_err();
    }
}
