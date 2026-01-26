use std::{
    fmt::Debug,
    io::Cursor,
    ops::{Index, IndexMut},
    path::Path,
    sync::Arc,
};

use chrono::{DateTime, Utc};
use reqwest::{
    header::{HeaderMap, CONTENT_LENGTH, CONTENT_TYPE},
    Body, Method,
};
use tokio::io::AsyncWrite;
use url::Url;

use crate::{error::OssError, types::CanonicalizedResource, Bucket};

mod body;
mod parts_upload;
pub use body::IntoBody;
pub use parts_upload::PartsUpload;

#[derive(Debug)]
pub struct Objects {
    //bucket: Bucket,
    pub(crate) list: Vec<Object>,
    pub(crate) next_token: Option<String>,
}

impl Objects {
    pub fn new(list: Vec<Object>, next_token: Option<String>) -> Objects {
        Objects { list, next_token }
    }

    pub fn next_token(&self) -> Option<&String> {
        self.next_token.as_ref()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn get_vec(&self) -> &Vec<Object> {
        &self.list
    }
}

impl Index<usize> for Objects {
    type Output = Object;
    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}

impl IndexMut<usize> for Objects {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.list[index]
    }
}

pub struct Object {
    path: String,
    bucket: Arc<Bucket>,
    content_type: String,
    copy_source: Option<String>,
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

impl Eq for Object {}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Object").field("path", &self.path).finish()
    }
}

impl Object {
    pub fn new<P: Into<String>>(path: P, bucket: Arc<Bucket>) -> Object {
        Object {
            path: path.into(),
            bucket,
            content_type: String::new(),
            copy_source: None,
        }
    }

    /// 确认文件是否在目录里面
    ///
    /// ```rust
    /// # use aliyun_oss_client::Object;
    /// let obj1 = Object::new("foo.txt");
    /// assert!(!obj1.in_dir());
    ///
    /// let obj2 = Object::new("path/foo.txt");
    /// assert!(obj2.in_dir());
    /// ```
    pub fn in_dir(&self) -> bool {
        self.path.find('/').is_some()
    }

    /// 获取文件的各级目录
    /// ```rust
    /// # use aliyun_oss_client::Object;
    /// let obj1 = Object::new("foo.txt");
    /// let dirs = obj1.get_dirs();
    /// assert!(dirs.len()==0);
    /// let obj2 = Object::new("path1/path2/foo.txt");
    /// let dirs2 = obj2.get_dirs();
    /// assert_eq!(dirs2[0], "path1".to_string());
    /// assert_eq!(dirs2[1], "path2".to_string());
    /// assert!(dirs2.len() ==2);
    /// ```
    pub fn get_dirs(&self) -> Vec<String> {
        let mut dirs: Vec<&str> = self.path.split('/').collect();
        dirs.pop();

        dirs.iter().map(|&d| d.to_owned()).collect()
    }

    /// 根据目录层级，获取绝对路径
    /// ```rust
    /// # use aliyun_oss_client::Object;
    /// let obj1 = Object::new("foo.txt");
    /// let path1 = obj1.absolute_dir_nth(10);
    /// assert!(path1.is_none());
    /// let obj2 = Object::new("path3/path22/bar.txt");
    /// let path21 = obj2.absolute_dir_nth(1);
    /// assert_eq!(path21, Some("path3".to_string()));
    /// let path22 = obj2.absolute_dir_nth(2);
    /// assert_eq!(path22, Some("path3/path22".to_string()));
    /// let path23 = obj2.absolute_dir_nth(3);
    /// assert_eq!(path23, Some("path3/path22".to_string()));
    /// ```
    pub fn absolute_dir_nth(&self, num: usize) -> Option<String> {
        let dirs = self.get_dirs();
        if dirs.is_empty() {
            return None;
        }
        let n = if num > dirs.len() { dirs.len() } else { num };
        let mut dir = String::new();
        for i in 0..n {
            if i == 0 {
                dir.push_str(&dirs[i]);
            } else {
                dir.push('/');
                dir.push_str(&dirs[i]);
            }
        }

        Some(dir)
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn to_url(&self) -> Result<Url, OssError> {
        let mut url = self.bucket.to_url()?;
        url.set_path(&self.path);
        Ok(url)
    }

    /// 获取 object 的 meta 信息
    pub async fn get_info(&self) -> Result<ObjectInfo, OssError> {
        let mut url = self.to_url()?;
        url.set_query(Some("objectMeta"));
        let method = Method::GET;
        let resource = CanonicalizedResource::new(format!(
            "/{}/{}?objectMeta",
            self.bucket.as_str(),
            self.path
        ));

        let header_map = self.bucket.client.authorization(&method, resource)?;

        let response = reqwest::Client::new()
            .request(method, url)
            .headers(header_map)
            .send()
            .await?;

        let headers = response.headers();

        let content_length = headers
            .get(CONTENT_LENGTH)
            .ok_or(OssError::NoFoundContentLength)?;
        let etag = headers.get("etag").ok_or(OssError::NoFoundEtag)?;

        let date = DateTime::parse_from_rfc2822(
            headers
                .get("last-modified")
                .ok_or(OssError::NoFoundLastModified)?
                .to_str()?,
        )?;
        Ok(ObjectInfo {
            last_modified: date.with_timezone(&Utc),
            etag: etag.to_str()?.to_string(),
            size: content_length.to_str()?.parse()?,
        })
    }

    pub fn content_type<T: Into<String>>(mut self, content_type: T) -> Self {
        self.content_type = content_type.into();
        self
    }

    /// 上传文件
    pub async fn upload(&self, body: impl IntoBody) -> Result<(), OssError> {
        let url = self.to_url()?;
        let method = Method::PUT;
        let resource = CanonicalizedResource::from_object(&self.bucket, &self);

        let mut header_map = HeaderMap::new();
        if !self.content_type.is_empty() {
            header_map.insert(CONTENT_TYPE, self.content_type.clone().try_into()?);
        }

        header_map = self
            .bucket
            .client
            .authorization_header(&method, resource, header_map)?;
        if let Some(content_length) = body.content_length() {
            if content_length == 0 {
                header_map.insert(CONTENT_LENGTH, 0.into());
            }
        }
        let body_stream = body.into_body();
        let req_body = Body::wrap_stream(body_stream);

        let response = reqwest::Client::new()
            .request(method, url)
            .headers(header_map)
            .body(req_body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let body = response.text().await?;
            Err(OssError::from_service(&body))
        }
    }

    #[cfg(feature = "tokio")]
    pub async fn upload_file<P: AsRef<Path>>(&self, path: P) -> Result<(), OssError> {
        let file = tokio::fs::File::open(path).await?;
        self.upload(file).await
    }

    /// 下载文件
    #[cfg(feature = "tokio")]
    pub async fn download<W>(&self, writer: &mut W) -> Result<(), OssError>
    where
        W: AsyncWrite + Unpin,
    {
        use futures_util::TryStreamExt;
        use tokio_util::io::StreamReader;

        let url = self.to_url()?;
        let method = Method::GET;
        let resource = CanonicalizedResource::from_object(&self.bucket, self);

        let header_map = self.bucket.client.authorization(&method, resource)?;

        let resp = reqwest::Client::new()
            .request(method, url)
            .headers(header_map)
            .send()
            .await
            .map_err(OssError::Reqwest)?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await?;
            return Err(OssError::from_service(&body));
        }

        let stream = resp
            .bytes_stream()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

        let mut reader = StreamReader::new(stream);

        tokio::io::copy(&mut reader, writer).await?;

        Ok(())
    }

    #[cfg(feature = "tokio")]
    pub async fn download_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), OssError> {
        let mut file = tokio::fs::File::create(path).await?;
        self.download(&mut file).await
    }

    #[cfg(feature = "tokio")]
    pub async fn download_to_bytes(&self) -> Result<Vec<u8>, OssError> {
        let mut buf = Cursor::new(Vec::new());
        self.download(&mut buf).await?;
        Ok(buf.into_inner())
    }

    #[cfg(feature = "tokio")]
    pub async fn download_to_string(&self) -> Result<String, OssError> {
        let bytes = self.download_to_bytes().await?;
        String::from_utf8(bytes).map_err(OssError::InvalidUtf8)
    }

    pub fn copy_source<T: Into<String>>(mut self, source: T) -> Self {
        self.copy_source = Some(source.into());
        self
    }

    /// 复制文件
    pub async fn copy(self) -> Result<(), OssError> {
        let url = self.to_url()?;
        let method = Method::PUT;
        let resource = CanonicalizedResource::from_object(&self.bucket, &self);

        let mut headers = HeaderMap::new();
        let source = self.copy_source.ok_or(OssError::CopySourceNotFound)?;
        headers.insert("x-oss-copy-source", source.try_into()?);

        if !self.content_type.is_empty() {
            headers.insert(CONTENT_TYPE, self.content_type.try_into()?);
        }

        let header_map = self
            .bucket
            .client
            .authorization_header(&method, resource, headers)?;

        let response = reqwest::Client::new()
            .request(method, url)
            .headers(header_map)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let body = response.text().await?;
            Err(OssError::from_service(&body))
        }
    }

    /// 删除文件
    pub async fn delete(&self) -> Result<(), OssError> {
        let url = self.to_url()?;
        let method = Method::DELETE;
        let resource = CanonicalizedResource::from_object(&self.bucket, self);

        let header_map = self.bucket.client.authorization(&method, resource)?;

        let response = reqwest::Client::new()
            .request(method, url)
            .headers(header_map)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let body = response.text().await?;
            Err(OssError::from_service(&body))
        }
    }
}

#[derive(Debug)]
pub struct ObjectInfo {
    last_modified: DateTime<Utc>,
    etag: String,
    size: u64,
}
impl ObjectInfo {
    pub fn new(last_modified: DateTime<Utc>, etag: String, size: u64) -> Self {
        ObjectInfo {
            last_modified,
            etag,
            size,
        }
    }

    pub fn last_modified(&self) -> &DateTime<Utc> {
        &self.last_modified
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn etag(&self) -> &str {
        &self.etag
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{bucket::Bucket, client::init_client};

    fn build_bucket() -> Bucket {
        Bucket::new("honglei123", Arc::new(init_client())).unwrap()
    }

    #[tokio::test]
    async fn test_object_info() {
        let object = build_bucket().object("aaabbc.txt");

        let info = object.get_info().await.unwrap();

        println!("{info:?}");
    }

    #[tokio::test]
    async fn test_upload() {
        let info = build_bucket()
            .object("abc.txt")
            .content_type("text/plain;charset=utf-8")
            .upload("aaab")
            .await
            .unwrap();

        println!("{info:?}");
    }

    #[tokio::test]
    async fn test_upload_file() {
        let f = tokio::fs::File::open("example_file.txt").await.unwrap();
        let info = build_bucket()
            .object("abc_file.txt")
            .content_type("text/plain;charset=utf-8")
            .upload(f)
            .await
            .unwrap();

        println!("{info:?}");
    }

    #[tokio::test]
    async fn test_down() {
        use tokio::fs::File;
        let object = build_bucket().object("abc.txt");

        let mut file = File::create("aaa.txt").await.unwrap();
        let result: () = object.download(&mut file).await.unwrap();

        std::fs::remove_file("aaa.txt").unwrap();
    }

    #[tokio::test]
    async fn test_copy() {
        let object = build_bucket().object("def2.txt");
        let _ = object
            .copy_source("/honglei123/abc2.txt")
            .content_type("text/plain;charset=utf-8")
            .copy()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_delete() {
        let object = build_bucket().object("abc.txt");

        let info = object.delete().await.unwrap();
    }

    #[tokio::test]
    async fn test_upload_empty_file() {
        let object = build_bucket().object("empty.txt");

        let info = object.upload("").await;
        assert!(info.is_ok())
    }
}
