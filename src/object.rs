use chrono::prelude::*;
use reqwest::Response;

use std::fmt;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;

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
#[cfg(feature = "blocking")]
use reqwest::blocking::Response as BResponse;
use reqwest::header::{HeaderMap, HeaderValue};
#[cfg(feature = "blocking")]
use std::rc::Rc;

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
    key: String,
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

    pub async fn put_file<P: Into<PathBuf> + std::convert::AsRef<std::path::Path>>(
        &self,
        file_name: P,
        key: &'static str,
    ) -> OssResult<String> {
        let mut file_content = Vec::new();
        std::fs::File::open(file_name)?.read_to_end(&mut file_content)?;

        self.put_content(file_content, key).await
    }

    pub async fn put_content(&self, content: Vec<u8>, key: &str) -> OssResult<String> {
        let kind = self.infer.get(&content);

        let con = match kind {
            Some(con) => Ok(con),
            None => Err(OssError::Input("file type is known".to_string())),
        };

        let content_type = con?.mime_type();

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
    pub fn put_file<P: Into<PathBuf> + std::convert::AsRef<std::path::Path>>(
        &self,
        file_name: P,
        key: &'static str,
    ) -> OssResult<String> {
        let mut file_content = Vec::new();
        std::fs::File::open(file_name)?.read_to_end(&mut file_content)?;

        self.put_content(file_content, key)
    }

    pub fn put_content(&self, content: Vec<u8>, key: &str) -> OssResult<String> {
        let kind = self.infer.get(&content);

        let con = match kind {
            Some(con) => Ok(con),
            None => Err(OssError::Input("file type is known".to_string())),
        };

        let content_type = con?.mime_type();

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

pub enum Encryption {
    Aes256,
    Kms,
    Sm4,
}

impl Default for Encryption {
    fn default() -> Encryption {
        Self::Aes256
    }
}

pub enum ObjectAcl {
    Default,
    Private,
    PublicRead,
    PublicReadWrite,
}

impl Default for ObjectAcl {
    fn default() -> Self {
        Self::Default
    }
}

pub enum StorageClass {
    Standard,
    IA,
    Archive,
    ColdArchive,
}

impl Default for StorageClass {
    fn default() -> Self {
        Self::Standard
    }
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

pub enum CopyDirective {
    Copy,
    Replace,
}

impl Default for CopyDirective {
    fn default() -> Self {
        Self::Copy
    }
}
