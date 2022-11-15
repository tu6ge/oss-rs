use chrono::prelude::*;

use crate::auth::VERB;
#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
use crate::builder::{ArcPointer, PointerFamily};
use crate::client::Client;
#[cfg(feature = "blocking")]
use crate::client::ClientRc;
use crate::config::{BucketBase, ObjectBase};
use crate::errors::{OssError, OssResult};
use crate::traits::{InvalidObjectListValue, InvalidObjectValue, OssIntoObject, OssIntoObjectList};
use crate::types::{CanonicalizedResource, ContentRange, Query, UrlQuery};
#[cfg(feature = "put_file")]
use infer::Infer;
#[cfg(feature = "blocking")]
use reqwest::blocking::Response as BResponse;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Response;
use std::fmt;
use std::path::PathBuf;
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Default)]
#[non_exhaustive]
pub struct ObjectList<PointerSel: PointerFamily = ArcPointer> {
    bucket: BucketBase,
    name: String,
    prefix: String,
    max_keys: u32,
    key_count: u64,
    pub object_list: Vec<Object<PointerSel>>,
    next_continuation_token: Option<String>,
    client: PointerSel::PointerType,
    search_query: Query,
}

impl<T: PointerFamily> fmt::Debug for ObjectList<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ObjectList")
            .field("name", &self.name)
            .field("bucket", &self.bucket)
            .field("prefix", &self.prefix)
            .field("max_keys", &self.max_keys)
            .field("key_count", &self.key_count)
            .field("next_continuation_token", &self.next_continuation_token)
            .field("search_query", &self.search_query)
            .finish()
    }
}

impl Default for ObjectList {
    fn default() -> Self {
        Self {
            bucket: BucketBase::default(),
            name: String::default(),
            prefix: String::default(),
            max_keys: u32::default(),
            key_count: u64::default(),
            object_list: Vec::new(),
            next_continuation_token: None,
            client: Arc::new(Client::default()),
            search_query: Query::default(),
        }
    }
}

#[cfg(feature = "blocking")]
impl Default for ObjectList<RcPointer> {
    fn default() -> Self {
        Self {
            bucket: BucketBase::default(),
            name: String::default(),
            prefix: String::default(),
            max_keys: u32::default(),
            key_count: u64::default(),
            object_list: Vec::new(),
            next_continuation_token: None,
            client: Rc::new(ClientRc::default()),
            search_query: Query::default(),
        }
    }
}

impl<T: PointerFamily> ObjectList<T> {
    pub fn new(
        bucket: BucketBase,
        name: String,
        prefix: String,
        max_keys: u32,
        key_count: u64,
        object_list: Vec<Object<T>>,
        next_continuation_token: Option<String>,
        client: T::PointerType,
        search_query: Query,
    ) -> Self {
        Self {
            bucket,
            name,
            prefix,
            max_keys,
            key_count,
            object_list,
            next_continuation_token,
            client,
            search_query,
        }
    }
}

impl ObjectList {
    pub fn set_client(mut self, client: Arc<Client>) -> Self {
        self.client = client;
        self
    }

    pub fn client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }
}

#[cfg(feature = "blocking")]
impl ObjectList<RcPointer> {
    pub fn set_client(mut self, client: Rc<ClientRc>) -> Self {
        self.client = client;
        self
    }

    pub fn client(&self) -> Rc<ClientRc> {
        Rc::clone(&self.client)
    }

    pub fn get_object_list(&mut self) -> OssResult<Self> {
        let mut url = self.bucket.to_url();

        url.set_search_query(&self.search_query);

        let canonicalized =
            CanonicalizedResource::from_bucket_query(&self.bucket, &self.search_query);

        let client = self.client();
        let response = client.builder(VERB::GET, url, canonicalized)?;
        let content = response.send()?;

        let list = Self::default()
            .set_client(Rc::clone(&client))
            .set_bucket(self.bucket.clone());
        Ok(list
            .from_xml(content.text()?, Rc::new(self.bucket.clone()))?
            .set_search_query(self.search_query.clone()))
    }
}

impl<T: PointerFamily> ObjectList<T> {
    pub fn set_search_query(mut self, search_query: Query) -> Self {
        self.search_query = search_query;
        self
    }

    pub fn set_bucket(mut self, bucket: BucketBase) -> Self {
        self.bucket = bucket;
        self
    }

    pub fn bucket_name(&self) -> &str {
        self.bucket.name()
    }

    pub fn len(&self) -> usize {
        self.object_list.len()
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Object<PointerSel: PointerFamily = ArcPointer> {
    base: ObjectBase<PointerSel>,
    key: String, // 打算弃用的字段
    last_modified: DateTime<Utc>,
    etag: String,
    _type: String,
    size: u64,
    storage_class: String,
}

impl<T: PointerFamily> Default for Object<T> {
    fn default() -> Self {
        Object {
            base: ObjectBase::<T>::default(),
            last_modified: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
            key: String::default(),
            etag: String::default(),
            _type: String::default(),
            size: 0,
            storage_class: String::default(),
        }
    }
}

impl<T: PointerFamily> Object<T> {
    #[inline]
    pub fn base(self) -> ObjectBase<T> {
        self.base
    }

    #[inline]
    pub fn set_base(mut self, base: ObjectBase<T>) {
        self.base = base;
    }

    #[inline]
    pub fn last_modified(self) -> DateTime<Utc> {
        self.last_modified
    }

    #[inline]
    pub fn set_last_modified(mut self, last_modified: DateTime<Utc>) {
        self.last_modified = last_modified;
    }

    #[inline]
    pub fn etag(self) -> String {
        self.etag
    }

    #[inline]
    pub fn set_etag(mut self, etag: String) {
        self.etag = etag
    }

    #[inline]
    pub fn get_type(self) -> String {
        self._type
    }

    #[inline]
    pub fn set_type(mut self, _type: String) {
        self._type = _type;
    }

    #[inline]
    pub fn size(self) -> u64 {
        self.size
    }

    #[inline]
    pub fn set_size(mut self, size: u64) {
        self.size = size;
    }

    #[inline]
    pub fn storage_class(self) -> String {
        self.storage_class
    }

    #[inline]
    pub fn set_storage_class(mut self, storage_class: String) {
        self.storage_class = storage_class;
    }

    /// 获取一部分数据
    pub fn pieces(self) -> (ObjectBase<T>, DateTime<Utc>, String, String, u64, String) {
        (
            self.base,
            self.last_modified,
            self.etag,
            self._type,
            self.size,
            self.storage_class,
        )
    }
}

impl<T: PointerFamily + Sized> OssIntoObject<T> for Object<T> {
    fn set_bucket(mut self, bucket: T::Bucket) -> Self {
        self.base.set_bucket(bucket);
        self
    }

    fn set_key(mut self, key: String) -> Result<Self, InvalidObjectValue> {
        self.key = key.clone();
        self.base.set_path(key);
        Ok(self)
    }

    fn set_last_modified(mut self, value: String) -> Result<Self, InvalidObjectValue> {
        let last_modified = value
            .parse::<DateTime<Utc>>()
            .map_err(|_| InvalidObjectValue {})?;
        self.last_modified = last_modified;
        Ok(self)
    }

    fn set_etag(mut self, value: String) -> Result<Self, InvalidObjectValue> {
        self.etag = value;
        Ok(self)
    }

    fn set_type(mut self, value: String) -> Result<Self, InvalidObjectValue> {
        self._type = value;
        Ok(self)
    }

    fn set_size(mut self, size: String) -> Result<Self, InvalidObjectValue> {
        self.size = size.parse::<u64>().map_err(|_| InvalidObjectValue {})?;
        Ok(self)
    }

    fn set_storage_class(mut self, value: String) -> Result<Self, InvalidObjectValue> {
        self.storage_class = value;
        Ok(self)
    }
}

impl<T: PointerFamily> OssIntoObjectList<Object<T>, T> for ObjectList<T> {
    fn set_key_count(mut self, key_count: String) -> Result<Self, InvalidObjectListValue> {
        self.key_count = key_count
            .parse::<u64>()
            .map_err(|_| InvalidObjectListValue {})?;
        Ok(self)
    }

    fn set_name(mut self, name: String) -> Result<Self, InvalidObjectListValue> {
        self.name = name;
        Ok(self)
    }

    fn set_prefix(mut self, prefix: String) -> Result<Self, InvalidObjectListValue> {
        self.prefix = prefix;
        Ok(self)
    }

    fn set_max_keys(mut self, max_keys: String) -> Result<Self, InvalidObjectListValue> {
        self.max_keys = max_keys
            .parse::<u32>()
            .map_err(|_| InvalidObjectListValue {})?;
        Ok(self)
    }

    fn set_next_continuation_token(
        mut self,
        token: Option<String>,
    ) -> Result<Self, InvalidObjectListValue> {
        self.next_continuation_token = token;
        Ok(self)
    }

    fn set_list(mut self, list: Vec<Object<T>>) -> Result<Self, InvalidObjectListValue> {
        self.object_list = list;
        Ok(self)
    }
}

impl Client {
    pub async fn get_object_list(self, query: Query) -> OssResult<ObjectList> {
        let mut url = self.get_bucket_url();

        url.set_search_query(&query);

        let bucket = self.get_bucket_base();

        let canonicalized = CanonicalizedResource::from_bucket_query(&bucket, &query);

        let response = self.builder(VERB::GET, url, canonicalized)?;
        let content = response.send().await?;

        let list = ObjectList::<ArcPointer>::default()
            .set_client(Arc::new(self))
            .set_bucket(bucket.clone());

        Ok(list
            .from_xml(content.text().await?, Arc::new(bucket))?
            .set_search_query(query))
    }

    /// # 上传文件
    ///
    /// 需指定文件的路径
    #[cfg(feature = "put_file")]
    pub async fn put_file<P: Into<PathBuf> + std::convert::AsRef<std::path::Path>>(
        &self,
        file_name: P,
        key: &'static str,
    ) -> OssResult<String> {
        let file_content = std::fs::read(file_name)?;

        let get_content_type = |content: &Vec<u8>| match Infer::new().get(content) {
            Some(con) => Some(con.mime_type()),
            None => None,
        };

        self.put_content(file_content, key, get_content_type).await
    }

    /// # 上传文件内容
    ///
    /// 需指定要上传的文件内容
    /// 以及获取文件类型的闭包
    ///
    /// # Examples
    ///
    /// 上传 tauri 升级用的签名文件
    /// ```ignore
    /// # #[tokio::main]
    /// # async fn main(){
    /// use infer::Infer;
    /// # use dotenv::dotenv;
    /// # dotenv().ok();
    /// # let client = aliyun_oss_client::Client::from_env().unwrap();
    ///
    /// fn sig_match(buf: &[u8]) -> bool {
    ///     return buf.len() >= 3 && buf[0] == 0x64 && buf[1] == 0x57 && buf[2] == 0x35;
    /// }
    /// let mut infer = Infer::new();
    /// infer.add("application/pgp-signature", "sig", sig_match);
    ///
    /// let get_content_type = |content: &Vec<u8>| match infer.get(content) {
    ///     Some(con) => Some(con.mime_type()),
    ///     None => None,
    /// };
    /// let content: Vec<u8> = String::from("dW50cnVzdGVkIGNvbW1lbnQ6IHNpxxxxxxxxx").into_bytes();
    /// let res = client
    ///     .put_content(content, "xxxxxx.msi.zip.sig", get_content_type)
    ///     .await;
    /// assert!(res.is_ok());
    /// # }
    /// ```
    pub async fn put_content<F>(
        &self,
        content: Vec<u8>,
        key: &str,
        get_content_type: F,
    ) -> OssResult<String>
    where
        F: Fn(&Vec<u8>) -> Option<&'static str>,
    {
        let content_type =
            get_content_type(&content).ok_or(OssError::Input("file type is known".to_string()))?;

        let content = self.put_content_base(content, content_type, key).await?;

        let result = content
            .headers()
            .get("ETag")
            .ok_or(OssError::Input("get Etag error".to_string()))?
            .to_str()
            .map_err(OssError::from)?;

        Ok(result.to_string())
    }

    /// 最原始的上传文件的方法
    pub async fn put_content_base(
        &self,
        content: Vec<u8>,
        content_type: &str,
        key: &str,
    ) -> OssResult<Response> {
        let mut url = self.get_bucket_url();
        url.set_path(key);

        let mut headers = HeaderMap::new();
        let content_length = content.len().to_string();
        headers.insert(
            "Content-Length",
            HeaderValue::from_str(&content_length).map_err(OssError::from)?,
        );

        headers.insert(
            "Content-Type",
            content_type.parse().map_err(OssError::from)?,
        );

        let object_base =
            ObjectBase::<ArcPointer>::new(Arc::new(self.get_bucket_base()), key.to_owned());

        let canonicalized = CanonicalizedResource::from_object(&object_base, None);

        let response = self
            .builder_with_header(VERB::PUT, url, canonicalized, Some(headers))?
            .body(content);

        let content = response.send().await?;
        Ok(content)
    }

    /// # 获取文件内容
    pub async fn get_object<R: Into<ContentRange>>(
        &self,
        key: &str,
        range: R,
    ) -> OssResult<Vec<u8>> {
        let mut url = self.get_bucket_url();
        url.set_path(key);

        let object_base =
            ObjectBase::<ArcPointer>::new(Arc::new(self.get_bucket_base()), key.to_owned());

        let canonicalized = CanonicalizedResource::from_object(&object_base, None);

        let headers = {
            let mut headers = HeaderMap::new();
            headers.insert("Range", range.into().into());
            headers
        };

        let builder = self.builder_with_header("GET", url, canonicalized, Some(headers))?;

        let response = builder.send().await?;

        let content = response.text().await?;

        Ok(content.into_bytes())
    }

    pub async fn delete_object(&self, key: &str) -> OssResult<()> {
        let mut url = self.get_bucket_url();
        url.set_path(key);

        let object_base =
            ObjectBase::<ArcPointer>::new(Arc::new(self.get_bucket_base()), key.to_owned());

        let canonicalized = CanonicalizedResource::from_object(&object_base, None);

        let response = self.builder(VERB::DELETE, url, canonicalized)?;

        response.send().await?;

        Ok(())
    }
}

#[cfg(feature = "blocking")]
impl ClientRc {
    pub fn get_object_list(self, query: Query) -> OssResult<ObjectList<RcPointer>> {
        let mut url = self.get_bucket_url();

        url.set_search_query(&query);

        let bucket = self.get_bucket_base();

        let canonicalized = CanonicalizedResource::from_bucket_query(&bucket, &query);

        let response = self.builder(VERB::GET, url, canonicalized)?;
        let content = response.send()?;

        let list = ObjectList::<RcPointer>::default()
            .set_client(Rc::new(self))
            .set_bucket(bucket.clone());

        Ok(list
            .from_xml(content.text()?, Rc::new(bucket))?
            .set_search_query(query))
    }

    #[cfg(feature = "put_file")]
    pub fn put_file<P: Into<PathBuf> + std::convert::AsRef<std::path::Path>>(
        &self,
        file_name: P,
        key: &'static str,
    ) -> OssResult<String> {
        let file_content = std::fs::read(file_name)?;

        let get_content_type = |content: &Vec<u8>| match Infer::new().get(content) {
            Some(con) => Some(con.mime_type()),
            None => None,
        };

        self.put_content(file_content, key, get_content_type)
    }

    pub fn put_content<F>(
        &self,
        content: Vec<u8>,
        key: &str,
        get_content_type: F,
    ) -> OssResult<String>
    where
        F: Fn(&Vec<u8>) -> Option<&'static str>,
    {
        let content_type =
            get_content_type(&content).ok_or(OssError::Input("file type is known".to_string()))?;

        let content = self.put_content_base(content, content_type, key)?;

        let result = content
            .headers()
            .get("ETag")
            .ok_or(OssError::Input("get Etag error".to_string()))?
            .to_str()
            .map_err(OssError::from)?;

        Ok(result.to_string())
    }

    /// 最原始的上传文件的方法
    pub fn put_content_base(
        &self,
        content: Vec<u8>,
        content_type: &str,
        key: &str,
    ) -> OssResult<BResponse> {
        let mut url = self.get_bucket_url();
        url.set_path(key);

        let mut headers = HeaderMap::new();
        let content_length = content.len().to_string();
        headers.insert(
            "Content-Length",
            HeaderValue::from_str(&content_length).map_err(OssError::from)?,
        );

        headers.insert(
            "Content-Type",
            content_type.parse().map_err(OssError::from)?,
        );

        let object_base =
            ObjectBase::<RcPointer>::new(Rc::new(self.get_bucket_base()), key.to_owned());

        let canonicalized = CanonicalizedResource::from_object(&object_base, None);

        let response = self
            .builder_with_header(VERB::PUT, url, canonicalized, Some(headers))?
            .body(content);

        let content = response.send()?;
        Ok(content)
    }

    pub fn delete_object(&self, key: &str) -> OssResult<()> {
        let mut url = self.get_bucket_url();
        url.set_path(key);

        let object_base =
            ObjectBase::<RcPointer>::new(Rc::new(self.get_bucket_base()), key.to_owned());

        let canonicalized = CanonicalizedResource::from_object(&object_base, None);

        let response = self.builder(VERB::DELETE, url, canonicalized)?;

        response.send()?;

        Ok(())
    }
}

#[cfg(feature = "blocking")]
impl Iterator for ObjectList<RcPointer> {
    type Item = ObjectList<RcPointer>;
    fn next(&mut self) -> Option<Self> {
        match self.next_continuation_token.clone() {
            Some(token) => {
                self.search_query.insert("continuation-token", token);

                match self.get_object_list() {
                    Ok(v) => Some(v),
                    _ => None,
                }
            }
            None => None,
        }
    }
}

// use futures::Stream;
// use std::iter::Iterator;
// use std::pin::{Pin};
// use std::task::Poll;
// impl <'a>Stream for ObjectList<'a> {
//   type Item = Vec<Object>;

//   /// 未测试的
//   fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> core::task::Poll<Option<Vec<Object>>> {

//     if let None = self.next_continuation_token {
//       return Poll::Ready(None);
//     }

//     let mut pinned = pin!(self.next_continuation_token);
//     match pinned.as_mut().poll(cx) {
//       Poll::Ready(token) => {
//         let mut query = self.search_query.clone();
//         query.insert("continuation-token".to_string(), token);
//         match self.client.get_object_list(query) {
//           Ok(list) => core::task::Poll::Ready(Some(list)),
//           Err(_) => core::task::Poll::Ready(None),
//         }
//       },
//       Poll::Pending => Poll::Pending
//     }
//   }
// }

#[derive(Default)]
pub struct PutObject<'a> {
    pub forbid_overwrite: bool,
    pub server_side_encryption: Option<Encryption>,
    pub server_side_data_encryption: Option<Encryption>,
    pub server_side_encryption_key_id: Option<&'a str>,
    pub object_acl: ObjectAcl,
    pub storage_class: StorageClass,
    pub tagging: Option<&'a str>,
}

#[derive(Default)]
pub enum Encryption {
    #[default]
    Aes256,
    Kms,
    Sm4,
}

#[derive(Default)]
pub enum ObjectAcl {
    #[default]
    Default,
    Private,
    PublicRead,
    PublicReadWrite,
}

#[derive(Default)]
pub enum StorageClass {
    #[default]
    Standard,
    IA,
    Archive,
    ColdArchive,
}

#[derive(Default)]
pub struct CopyObject<'a> {
    pub forbid_overwrite: bool,
    pub copy_source: &'a str,
    pub copy_source_if_match: Option<&'a str>,
    pub copy_source_if_none_match: Option<&'a str>,
    pub copy_source_if_unmodified_since: Option<&'a str>,
    pub copy_source_if_modified_since: Option<&'a str>,
    pub metadata_directive: CopyDirective,
    pub server_side_encryption: Option<Encryption>,
    pub server_side_encryption_key_id: Option<&'a str>,
    pub object_acl: ObjectAcl,
    pub storage_class: StorageClass,
    pub tagging: Option<&'a str>,
    pub tagging_directive: CopyDirective,
}

#[derive(Default)]
pub enum CopyDirective {
    #[default]
    Copy,
    Replace,
}

#[cfg(test)]
mod tests {}
