//! 读写 object 内容

use std::{
    io::{Read, Result as IoResult, Seek, SeekFrom, Write},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use http::Method;
use url::Url;

#[cfg(test)]
use super::mock::blocking::Files;
#[cfg(not(test))]
use crate::file::BlockingFiles;
use crate::{
    file::blocking::AlignBuilder,
    object::InitObject,
    types::{
        header::HeaderVal,
        object::{InvalidObjectPath, SetObjectPath},
        CanonicalizedResource,
    },
    ClientRc as Client, ObjectPath,
};

use super::{super::ObjectsBlocking, ContentError, ContentErrorKind, Inner};

/// # object 内容
/// [OSS 分片上传文档](https://help.aliyun.com/zh/oss/user-guide/multipart-upload-12)
//#[derive(Debug)]
pub struct Content {
    client: Rc<Client>,
    inner: Inner,
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
            return self.upload();
        }

        self.upload_multi()
    }
}

impl Read for Content {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let len = buf.len();
        if len as u64 > Inner::MAX_SIZE {
            return Err(ContentError::new(ContentErrorKind::OverflowMaxSize).into());
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
        self.inner.seek(pos)
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
    pub fn from_client(client: Rc<Client>) -> Content {
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

    fn upload(&mut self) -> IoResult<()> {
        assert!(self.content_part.len() == 1);
        let content = self.content_part.pop().unwrap();
        self.client
            .put_content_base(content, self.content_type, self.path.clone())
            .map(|_| ())
            .map_err(Into::into)
    }

    fn upload_multi(&mut self) -> IoResult<()> {
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

    /// 初始化批量上传
    fn init_multi(&mut self) -> Result<(), ContentError> {
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
    fn upload_part(&mut self, index: u16, buf: Vec<u8>) -> Result<(), ContentError> {
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

        let query = format!("partNumber={}&uploadId={}", index, self.upload_id);

        let (url, resource) = self.part_canonicalized(&query);

        let resp = self
            .client
            .builder_with_header(
                Method::PUT,
                url,
                resource,
                HeaderVal::ContentLength(buf.len()),
            )?
            .body(buf)
            .send_adjust_error()?;

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
    fn complete_multi(&mut self) -> Result<(), ContentError> {
        if self.upload_id.is_empty() {
            return Err(ContentError::new(ContentErrorKind::NoFoundUploadId));
        }

        let xml = self.etag_list_xml()?;

        let query = format!("uploadId={}", self.upload_id);

        let (url, resource) = self.part_canonicalized(&query);

        let _resp = self
            .client
            .builder_with_header(
                Method::POST,
                url,
                resource,
                HeaderVal::ContentLength(xml.len()),
            )?
            .body(xml)
            .send_adjust_error()?;

        //println!("resp: {}", resp);
        self.etag_list.clear();
        self.upload_id = String::default();

        Ok(())
    }

    /// 取消分块上传
    pub fn abort_multi(&mut self) -> Result<(), ContentError> {
        if self.upload_id.is_empty() {
            return Err(ContentError::new(ContentErrorKind::NoFoundUploadId));
        }
        let query = format!("uploadId={}", self.upload_id);

        let (url, resource) = self.part_canonicalized(&query);
        let _resp = self
            .client
            .builder(Method::DELETE, url, resource)?
            .send_adjust_error()?;

        //println!("resp: {:?}", resp);
        self.etag_list.clear();
        self.upload_id = String::default();

        Ok(())
    }
}

// impl Drop for Content {
//     fn drop(&mut self) {
//         if self.upload_id.is_empty() == false {
//             self.abort_multi();
//         }
//     }
// }

impl From<Client> for Content {
    fn from(value: Client) -> Self {
        Content {
            client: Rc::new(value),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::decode::RefineObject;

    use super::super::test_suite_block::{
        AbortMulti, CompleteMulti, InitMulti, UploadMulti, UploadPart,
    };
    use super::*;

    #[test]
    fn assert_impl() {
        fn impl_fn<T: RefineObject<E>, E: std::error::Error + 'static>(_: T) {}

        impl_fn(Content::default());

        fn impl_deref<T: Deref<Target = Inner>>(_: T) {}

        impl_deref(Content::default());
    }

    #[test]
    fn read() {
        let client = Client::test_init();
        let mut con = Content::from_client(Rc::new(client))
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
        list.set_client(Rc::new(client.clone()));

        let con = list.init_object().unwrap();

        assert_eq!(con.client.bucket, client.bucket);
        assert_eq!(con.inner, Inner::default());
    }

    #[test]
    fn from_client() {
        let client = Client::test_init();

        let con = Content::from_client(Rc::new(client.clone()));

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
        let con = Content::from_client(Rc::new(client))
            .path("aaa.txt")
            .unwrap();

        let (url, can) = con.part_canonicalized("first=1&second=2");
        assert_eq!(
            url.as_str(),
            "https://bar.oss-cn-qingdao.aliyuncs.com/aaa.txt?first=1&second=2"
        );
        assert_eq!(can.to_string(), "/bar/aaa.txt?first=1&second=2");
    }

    #[test]
    fn upload() {
        let client = Client::test_init();
        let mut con = Content::from_client(Rc::new(client))
            .path("aaa.txt")
            .unwrap();
        con.content_part.push(b"bbb".to_vec());
        con.upload().unwrap();
    }

    #[test]
    fn init_multi() {
        let client = Client::test_init().middleware(Rc::new(InitMulti {}));
        let mut con = Content::from_client(Rc::new(client))
            .path("aaa.txt")
            .unwrap();

        con.init_multi().unwrap();

        assert_eq!(con.upload_id, "foo_upload_id");
    }

    #[test]
    fn upload_part() {
        let client = Client::test_init().middleware(Rc::new(UploadPart {}));
        let mut con = Content::from_client(Rc::new(client))
            .path("aaa.txt")
            .unwrap();

        let err = con.upload_part(1, b"bbb".to_vec()).unwrap_err();
        assert_eq!(err.to_string(), "not found upload id");

        con.upload_id = "foo_upload_id".to_string();
        for _i in 0..10 {
            con.etag_list.push((1, "a".parse().unwrap()));
        }
        let err = con.upload_part(1, b"bbb".to_vec()).unwrap_err();
        assert_eq!(err.to_string(), "overflow max parts count");
        con.etag_list.clear();

        let err = con
            .upload_part(1, b"012345678901234567890".to_vec())
            .unwrap_err();
        assert_eq!(err.to_string(), "part size must be between 100k and 5G");

        con.upload_part(2, b"bbb".to_vec()).unwrap();
        let (index, value) = con.etag_list.pop().unwrap();
        assert_eq!(index, 2);
        assert_eq!(value.to_str().unwrap(), "foo_etag");
    }

    #[test]
    fn complete_multi() {
        let client = Client::test_init().middleware(Rc::new(CompleteMulti {}));
        let mut con = Content::from_client(Rc::new(client))
            .path("aaa.txt")
            .unwrap();
        let err = con.complete_multi().unwrap_err();
        assert_eq!(err.to_string(), "not found upload id");

        con.upload_id = "foo_upload_id".to_string();
        con.etag_list.push((1, "aaa".parse().unwrap()));
        con.etag_list.push((2, "bbb".parse().unwrap()));
        con.complete_multi().unwrap();
        assert!(con.etag_list.is_empty());
        assert!(con.upload_id.is_empty());
    }

    #[test]
    fn upload_multi() {
        let client = Client::test_init().middleware(Rc::new(UploadMulti {}));
        let mut con = Content::from_client(Rc::new(client))
            .path("aaa.txt")
            .unwrap();

        con.content_part.push(b"aaa".to_vec());
        con.content_part.push(b"bbb".to_vec());

        con.upload_multi().unwrap();

        assert_eq!(con.content_size, 6);
    }

    #[test]
    fn abort_multi() {
        let client = Client::test_init().middleware(Rc::new(AbortMulti {}));
        let mut con = Content::from_client(Rc::new(client))
            .path("aaa.txt")
            .unwrap();
        let err = con.complete_multi().unwrap_err();
        assert_eq!(err.to_string(), "not found upload id");

        con.upload_id = "foo_upload_id".to_string();
        con.etag_list.push((1, "aaa".parse().unwrap()));
        con.abort_multi().unwrap();
        assert!(con.etag_list.is_empty());
        assert!(con.upload_id.is_empty());
    }
}
