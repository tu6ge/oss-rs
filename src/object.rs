use crate::auth::VERB;
#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
use crate::builder::{ArcPointer, BuilderError, PointerFamily};
use crate::client::ClientArc;
#[cfg(feature = "blocking")]
use crate::client::ClientRc;
use crate::config::{BucketBase, ObjectBase, ObjectPath};
use crate::errors::{OssError, OssResult};
#[cfg(feature = "blocking")]
use crate::file::blocking::AlignBuilder as BlockingAlignBuilder;
use crate::file::AlignBuilder;
use crate::traits::{RefineObject, RefineObjectList};
use crate::types::{CanonicalizedResource, Query, UrlQuery, CONTINUATION_TOKEN};
use crate::{BucketName, Client};
use async_stream::try_stream;
use chrono::prelude::*;
use futures_core::stream::Stream;
use oss_derive::oss_gen_rc;

use std::fmt;
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::sync::Arc;
use std::vec::IntoIter;

/// # 存放对象列表的结构体
/// TODO impl core::ops::Index
#[derive(Clone)]
#[non_exhaustive]
pub struct ObjectList<PointerSel: PointerFamily = ArcPointer> {
    pub(crate) bucket: BucketBase,
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
            .field("bucket", &self.bucket)
            .field("prefix", &self.prefix)
            .field("max_keys", &self.max_keys)
            .field("key_count", &self.key_count)
            .field("next_continuation_token", &self.next_continuation_token)
            .field("search_query", &self.search_query)
            .finish()
    }
}

#[oss_gen_rc]
impl Default for ObjectList<ArcPointer> {
    fn default() -> Self {
        Self {
            bucket: BucketBase::default(),
            prefix: String::default(),
            max_keys: u32::default(),
            key_count: u64::default(),
            object_list: Vec::new(),
            next_continuation_token: None,
            client: Arc::new(ClientArc::default()),
            search_query: Query::default(),
        }
    }
}

impl<T: PointerFamily> ObjectList<T> {
    pub fn new<Q: Into<Query>>(
        bucket: BucketBase,
        prefix: String,
        max_keys: u32,
        key_count: u64,
        object_list: Vec<Object<T>>,
        next_continuation_token: Option<String>,
        client: T::PointerType,
        search_query: Q,
    ) -> Self {
        Self {
            bucket,
            prefix,
            max_keys,
            key_count,
            object_list,
            next_continuation_token,
            client,
            search_query: search_query.into(),
        }
    }

    pub fn bucket(&self) -> &BucketBase {
        &self.bucket
    }

    pub fn prefix(&self) -> &String {
        &self.prefix
    }

    pub fn max_keys(&self) -> &u32 {
        &self.max_keys
    }

    pub fn key_count(&self) -> &u64 {
        &self.key_count
    }

    pub fn next_continuation_token(&self) -> &Option<String> {
        &self.next_continuation_token
    }

    /// # 下一页的查询条件
    ///
    /// 如果有下一页，返回 Some(Query)
    /// 如果没有下一页，则返回 None
    pub fn next_query(&self) -> Option<Query> {
        match &self.next_continuation_token {
            Some(token) => {
                let mut search_query = self.search_query.clone();
                search_query.insert(CONTINUATION_TOKEN, token.to_owned());
                Some(search_query)
            }
            None => None,
        }
    }

    /// 将 object 列表转化为迭代器
    pub fn object_iter(self) -> IntoIter<Object<T>> {
        self.object_list.into_iter()
    }
}

#[oss_gen_rc]
impl ObjectList<ArcPointer> {
    pub fn set_client(&mut self, client: Arc<ClientArc>) {
        self.client = client;
    }

    pub fn client(&self) -> Arc<ClientArc> {
        Arc::clone(&self.client)
    }
}

impl ObjectList {
    pub async fn get_next_list(&self) -> OssResult<Self> {
        match self.next_query() {
            None => Err(OssError::WithoutMore),
            Some(query) => {
                let mut url = self.bucket.to_url();
                url.set_search_query(&query);

                let canonicalized = CanonicalizedResource::from_bucket_query(&self.bucket, &query);

                let response = self.builder(VERB::GET, url, canonicalized)?;
                let content = response.send().await?;

                let mut list = ObjectList::<ArcPointer>::default();
                list.set_client(self.client().clone());
                list.set_bucket(self.bucket.clone());

                let bucket_arc = Arc::new(self.bucket.clone());

                let init_object = || {
                    let mut object = Object::<ArcPointer>::default();
                    object.base.set_bucket(bucket_arc.clone());
                    object
                };

                list.from_xml(&content.text().await?, init_object)?;

                list.set_search_query(query);
                Ok(list)
            }
        }
    }

    /// # 将 object_list 转化为 stream, 返回第二页，第三页... 的内容
    ///
    /// *不够完善，最后一次迭代返回的是 `Some(Err(OssError::WithoutMore))`，而不是 `None`*
    ///
    /// ## 用法
    ///
    /// 1. 添加依赖
    /// ```toml
    /// [dependencies]
    /// futures="0.3"
    /// ```
    /// 2. 将返回结果 pin 住
    /// ```no_run
    /// # use dotenv::dotenv;
    /// # use aliyun_oss_client::Client;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # dotenv().ok();
    /// use futures::{pin_mut, StreamExt};
    /// # let client = Client::from_env().unwrap();
    /// # let query = [("max-keys", 100u8)];
    /// # let object_list = client.get_object_list(query).await.unwrap();
    /// let stream = object_list.into_stream();
    /// pin_mut!(stream);
    ///
    /// let second_list = stream.next().await;
    /// let third_list = stream.next().await;
    /// println!("second_list: {:?}", second_list);
    /// println!("third_list: {:?}", third_list);
    /// # }
    /// ```
    pub fn into_stream(self) -> impl Stream<Item = OssResult<Self>> {
        try_stream! {
            let result = self.get_next_list().await?;
            yield result;
        }
    }
}

#[cfg(feature = "blocking")]
impl ObjectList<RcPointer> {
    pub fn get_object_list(&mut self) -> OssResult<Self> {
        let name = self.bucket.get_name();

        let client = self.client();

        let mut list = Self::default();
        list.set_client(Rc::clone(&client));
        list.set_bucket(self.bucket.clone());
        list.set_search_query(self.search_query.clone());

        let bucket_arc = Rc::new(self.bucket.clone());
        let init_object = || {
            let mut object = Object::<RcPointer>::default();
            object.base.set_bucket(bucket_arc.clone());
            object
        };

        let result: Result<_, OssError> = client.base_object_list(
            name.to_owned(),
            self.search_query.clone(),
            &mut list,
            init_object,
        );
        result?;

        Ok(list)
    }
}

impl<T: PointerFamily> ObjectList<T> {
    #[inline]
    pub fn set_search_query(&mut self, search_query: Query) {
        self.search_query = search_query;
    }

    pub fn set_bucket(&mut self, bucket: BucketBase) {
        self.bucket = bucket;
    }

    pub fn bucket_name(&self) -> &str {
        self.bucket.name()
    }

    pub fn len(&self) -> usize {
        self.object_list.len()
    }
}

/// 存放单个对象的结构体
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Object<PointerSel: PointerFamily = ArcPointer> {
    pub(crate) base: ObjectBase<PointerSel>,
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
            etag: String::default(),
            _type: String::default(),
            size: 0,
            storage_class: String::default(),
        }
    }
}

impl<T: PointerFamily> Object<T> {
    /// 初始化 Object 结构体
    pub fn new<P: Into<ObjectPath>>(
        bucket: T::Bucket,
        path: P,
        last_modified: DateTime<Utc>,
        etag: String,
        _type: String,
        size: u64,
        storage_class: String,
    ) -> Self {
        let base = ObjectBase::<T>::new(bucket, path);
        Self {
            base,
            last_modified,
            etag,
            _type,
            size,
            storage_class,
        }
    }

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

    pub fn path(&self) -> ObjectPath {
        self.base.path()
    }

    pub fn path_string(&self) -> String {
        self.base.path().to_string()
    }
}

/// Object 结构体的构建器
pub struct ObjectBuilder<T: PointerFamily = ArcPointer> {
    object: Object<T>,
}

impl<T: PointerFamily> ObjectBuilder<T> {
    /// TODO 有待进一步优化
    pub fn new<P: Into<ObjectPath>>(bucket: T::Bucket, key: P) -> Self {
        let base = ObjectBase::<T>::new(bucket, key);
        Self {
            object: Object {
                base,
                last_modified: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
                etag: String::default(),
                _type: String::default(),
                size: 0,
                storage_class: String::default(),
            },
        }
    }

    pub fn last_modified(mut self, date: DateTime<Utc>) -> Self {
        self.object.last_modified = date;
        self
    }

    pub fn etag(mut self, etag: String) -> Self {
        self.object.etag = etag;
        self
    }

    pub fn set_type(mut self, _type: String) -> Self {
        self.object._type = _type;
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.object.size = size;
        self
    }

    pub fn storage_class(mut self, storage_class: String) -> Self {
        self.object.storage_class = storage_class;
        self
    }

    pub fn build(self) -> Object<T> {
        self.object
    }
}

impl<T: PointerFamily + Sized> RefineObject for Object<T> {
    type Error = OssError;

    #[inline]
    fn set_key(&mut self, key: &str) -> Result<(), Self::Error> {
        self.base.set_path(key);
        Ok(())
    }

    #[inline]
    fn set_last_modified(&mut self, value: &str) -> Result<(), Self::Error> {
        self.last_modified = value.parse::<DateTime<Utc>>().map_err(OssError::from)?;
        Ok(())
    }

    #[inline]
    fn set_etag(&mut self, value: &str) -> Result<(), Self::Error> {
        self.etag = value.to_string();
        Ok(())
    }

    #[inline]
    fn set_type(&mut self, value: &str) -> Result<(), Self::Error> {
        self._type = value.to_string();
        Ok(())
    }

    #[inline]
    fn set_size(&mut self, size: &str) -> Result<(), Self::Error> {
        self.size = size.parse::<u64>().map_err(OssError::from)?;
        Ok(())
    }

    #[inline]
    fn set_storage_class(&mut self, value: &str) -> Result<(), Self::Error> {
        self.storage_class = value.to_string();
        Ok(())
    }
}

impl<T: PointerFamily> RefineObjectList<Object<T>> for ObjectList<T> {
    type Error = OssError;

    #[inline]
    fn set_key_count(&mut self, key_count: &str) -> Result<(), Self::Error> {
        self.key_count = key_count.parse::<u64>().map_err(OssError::from)?;
        Ok(())
    }

    #[inline]
    fn set_prefix(&mut self, prefix: &str) -> Result<(), Self::Error> {
        self.prefix = prefix.to_owned();
        Ok(())
    }

    #[inline]
    fn set_max_keys(&mut self, max_keys: &str) -> Result<(), Self::Error> {
        self.max_keys = max_keys.parse::<u32>().map_err(OssError::from)?;
        Ok(())
    }

    #[inline]
    fn set_next_continuation_token(&mut self, token: Option<&str>) -> Result<(), Self::Error> {
        self.next_continuation_token = token.map(|t| t.to_owned());
        Ok(())
    }

    #[inline]
    fn set_list(&mut self, list: Vec<Object<T>>) -> Result<(), Self::Error> {
        self.object_list = list;
        Ok(())
    }
}

impl Client {
    /// 查询默认 bucket 的文件列表
    ///
    /// 查询条件参数有多种方式，具体参考 [`get_object_list`](../bucket/struct.Bucket.html#method.get_object_list) 文档
    pub async fn get_object_list<Q: Into<Query>>(self, query: Q) -> OssResult<ObjectList> {
        let name = self.get_bucket_name();
        let bucket = BucketBase::new(name.clone(), self.get_endpoint().to_owned());

        let mut list = ObjectList::<ArcPointer>::default();
        list.set_bucket(bucket.clone());

        let bucket_arc = Arc::new(bucket.clone());
        let init_object = || {
            let mut object = Object::<ArcPointer>::default();
            object.base.set_bucket(bucket_arc.clone());
            object
        };

        let query = query.into();

        let result: Result<_, OssError> = self
            .base_object_list(name.to_owned(), query.clone(), &mut list, init_object)
            .await;
        result?;

        list.set_client(Arc::new(self));
        list.set_search_query(query);

        Ok(list)
    }

    /// # 可将 object 列表导出到外部类型（不仅仅是 struct）
    /// 可以参考下面示例，或者项目中的 `examples/custom.rs`
    /// ## 示例
    /// ```rust
    /// use aliyun_oss_client::{
    ///     builder::BuilderError,
    ///     traits::{RefineObject, RefineObjectList},
    ///     Client,
    /// };
    /// use dotenv::dotenv;
    /// use thiserror::Error;
    ///
    /// struct MyFile {
    ///     key: String,
    ///     #[allow(dead_code)]
    ///     other: String,
    /// }
    /// impl RefineObject for MyFile {
    ///     type Error = MyError;
    ///
    ///     fn set_key(&mut self, key: &str) -> Result<(), Self::Error> {
    ///         self.key = key.to_string();
    ///         Ok(())
    ///     }
    /// }
    ///
    /// #[derive(Default)]
    /// struct MyBucket {
    ///     name: String,
    ///     files: Vec<MyFile>,
    /// }
    ///
    /// impl RefineObjectList<MyFile> for MyBucket {
    ///     type Error = MyError;
    ///
    ///     fn set_name(&mut self, name: &str) -> Result<(), Self::Error> {
    ///         self.name = name.to_string();
    ///         Ok(())
    ///     }
    ///     fn set_list(&mut self, list: Vec<MyFile>) -> Result<(), Self::Error> {
    ///         self.files = list;
    ///         Ok(())
    ///     }
    /// }
    ///
    /// #[derive(Debug, Error)]
    /// enum MyError {
    ///     #[error(transparent)]
    ///     QuickXml(#[from] quick_xml::Error),
    ///     #[error(transparent)]
    ///     BuilderError(#[from] BuilderError),
    /// }
    ///
    /// async fn run() -> Result<(), MyError> {
    ///     dotenv().ok();
    ///
    ///     let client = Client::from_env().unwrap();
    ///
    ///     // 除了设置Default 外，还可以做更多设置
    ///     let mut bucket = MyBucket::default();
    ///
    ///     // 利用闭包对 MyFile 做一下初始化设置
    ///     let init_file = || MyFile {
    ///         key: String::default(),
    ///         other: "abc".to_string(),
    ///     };
    ///     //let bucket_name = env::var("ALIYUN_BUCKET").unwrap();
    ///     let bucket_name = "abc";
    ///
    ///     let res: Result<_, MyError> = client
    ///         .base_object_list(bucket_name, [], &mut bucket, init_file)
    ///         .await;
    ///
    ///     res?;
    ///
    ///     println!("bucket: {:?}", bucket);
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub async fn base_object_list<Name: Into<BucketName>, Q: Into<Query>, List, Item, F, E>(
        &self,
        name: Name,
        query: Q,
        list: &mut List,
        init_object: F,
    ) -> Result<(), E>
    where
        List: RefineObjectList<Item>,
        Item: RefineObject,
        E: From<BuilderError> + From<List::Error>,
        F: FnMut() -> Item,
    {
        let bucket = BucketBase::new(name.into(), self.get_endpoint().to_owned());

        let mut bucket_url = bucket.to_url();
        let query = query.into();
        bucket_url.set_search_query(&query);

        let canonicalized = CanonicalizedResource::from_bucket_query(&bucket, &query);

        let response = self.builder(VERB::GET, bucket_url, canonicalized)?;
        let content = response.send().await?;

        list.from_xml(
            &content.text().await.map_err(BuilderError::from)?,
            init_object,
        )?;

        Ok(())
    }
}

#[cfg(feature = "blocking")]
impl ClientRc {
    /// 查询默认 bucket 的文件列表
    ///
    /// 查询条件参数有多种方式，具体参考 [`get_object_list`](../bucket/struct.Bucket.html#method.get_object_list) 文档
    pub fn get_object_list<Q: Into<Query>>(self, query: Q) -> OssResult<ObjectList<RcPointer>> {
        let name = self.get_bucket_name();
        let bucket = BucketBase::new(name.clone(), self.get_endpoint().to_owned());

        let mut list = ObjectList::<RcPointer>::default();
        list.set_bucket(bucket.clone());

        let bucket_arc = Rc::new(bucket.clone());

        let init_object = || {
            let mut object = Object::<RcPointer>::default();
            object.base.set_bucket(bucket_arc.clone());
            object
        };

        let query = query.into();

        let result: Result<_, OssError> =
            self.base_object_list(name.to_owned(), query.clone(), &mut list, init_object);
        result?;

        list.set_client(Rc::new(self));
        list.set_search_query(query);

        Ok(list)
    }

    /// 可将 object 列表导出到外部 struct
    #[inline]
    pub fn base_object_list<Name: Into<BucketName>, Q: Into<Query>, List, Item, F, E>(
        &self,
        name: Name,
        query: Q,
        list: &mut List,
        init_object: F,
    ) -> Result<(), E>
    where
        List: RefineObjectList<Item>,
        Item: RefineObject,
        E: From<BuilderError> + From<List::Error>,
        F: FnMut() -> Item,
    {
        let bucket = BucketBase::new(name.into(), self.get_endpoint().to_owned());

        let mut bucket_url = bucket.to_url();
        let query = query.into();
        bucket_url.set_search_query(&query);

        let canonicalized = CanonicalizedResource::from_bucket_query(&bucket, &query);

        let response = self.builder(VERB::GET, bucket_url, canonicalized)?;
        let content = response.send()?;

        list.from_xml(&content.text().map_err(BuilderError::from)?, init_object)?;

        Ok(())
    }
}

#[cfg(feature = "blocking")]
impl Iterator for ObjectList<RcPointer> {
    type Item = ObjectList<RcPointer>;
    fn next(&mut self) -> Option<Self> {
        match self.next_continuation_token.clone() {
            Some(token) => {
                self.search_query.insert(CONTINUATION_TOKEN, token);

                match self.get_object_list() {
                    Ok(v) => Some(v),
                    _ => None,
                }
            }
            None => None,
        }
    }
}

// use futures::future::Pending;
// use futures::FutureExt;
// use reqwest::Response;
// use std::task::Poll::{self, Ready};

// impl Stream for ObjectList<ArcPointer> {
//     type Item = ObjectList<ArcPointer>;

//     fn poll_next(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> Poll<Option<Self::Item>> {
//         match self.next_query() {
//             Some(query) => {
//                 let mut url = self.bucket.to_url();
//                 url.set_search_query(&query);

//                 let canonicalized = CanonicalizedResource::from_bucket_query(&self.bucket, &query);

//                 let builder = self.builder(VERB::GET, url, canonicalized);
//                 match builder {
//                     Err(err) => return Ready(None),
//                     Ok(builder) => {
//                         let content = match builder.send().poll_unpin(cx) {
//                             Ready(res) => res,
//                             Poll::Pending => return Poll::Pending,
//                         };

//                         let response = match content {
//                             Ok(res) => res,
//                             Err(_) => return Ready(None),
//                         };

//                         let text = match response.text().poll_unpin(cx) {
//                             Ready(res) => res,
//                             Poll::Pending => return Poll::Pending,
//                         };

//                         let text = match text {
//                             Ok(res) => res,
//                             Err(_) => return Ready(None),
//                         };

//                         let list = ObjectList::<ArcPointer>::default()
//                             .set_client(self.client().clone())
//                             .set_bucket(self.bucket.clone());

//                         let result = list.from_xml(text, Arc::new(self.bucket.clone()));

//                         let mut result = match result {
//                             Ok(data) => data,
//                             Err(_) => return Ready(None),
//                         };

//                         result.set_search_query(query);

//                         if result.len() == 0 {
//                             Ready(None)
//                         } else {
//                             Ready(Some(result))
//                         }
//                     }
//                 }
//             }
//             None => Ready(None),
//         }
//     }
// }

#[oss_gen_rc]
impl PartialEq<Object<ArcPointer>> for Object<ArcPointer> {
    #[inline]
    fn eq(&self, other: &Object<ArcPointer>) -> bool {
        self.base == other.base
            && self.last_modified == other.last_modified
            && self.etag == other.etag
            && self._type == other._type
            && self.size == other.size
            && self.storage_class == other.storage_class
    }
}

impl<T: PointerFamily> PartialEq<DateTime<Utc>> for Object<T> {
    #[inline]
    fn eq(&self, other: &DateTime<Utc>) -> bool {
        &self.last_modified == other
    }
}

impl<T: PointerFamily> PartialEq<u64> for Object<T> {
    #[inline]
    fn eq(&self, other: &u64) -> bool {
        &self.size == other
    }
}

#[oss_gen_rc]
impl PartialEq<ObjectBase<ArcPointer>> for Object<ArcPointer> {
    #[inline]
    fn eq(&self, other: &ObjectBase<ArcPointer>) -> bool {
        &self.base == other
    }
}

/// 未来计划支持的功能
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

/// 未来计划支持的功能
#[derive(Default)]
pub enum Encryption {
    #[default]
    Aes256,
    Kms,
    Sm4,
}

/// 未来计划支持的功能
#[derive(Default)]
pub enum ObjectAcl {
    #[default]
    Default,
    Private,
    PublicRead,
    PublicReadWrite,
}

/// 未来计划支持的功能
#[derive(Default)]
pub enum StorageClass {
    #[default]
    Standard,
    IA,
    Archive,
    ColdArchive,
}

/// 未来计划支持的功能
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

/// 未来计划支持的功能
#[derive(Default)]
pub enum CopyDirective {
    #[default]
    Copy,
    Replace,
}

#[cfg(test)]
mod tests {
    use super::ObjectList;
    use crate::{
        builder::ArcPointer,
        config::BucketBase,
        object::{Object, ObjectBuilder},
        types::QueryValue,
        Client,
    };
    use chrono::{DateTime, NaiveDateTime, Utc};
    use std::sync::Arc;

    fn init_object_list(token: Option<String>, list: Vec<Object>) -> ObjectList {
        let client = Client::new(
            "foo1".into(),
            "foo2".into(),
            "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
            "foo4".try_into().unwrap(),
        );

        let object_list = ObjectList::<ArcPointer>::new(
            BucketBase::from_str("abc.oss-cn-shanghai.aliyuncs.com").unwrap(),
            String::from("foo2"),
            100,
            200,
            list,
            token,
            Arc::new(client),
            vec![("key1", "value1")],
        );

        object_list
    }

    #[test]
    fn test_get_bucket() {
        let object_list = init_object_list(Some(String::from("foo3")), vec![]);

        let bucket = object_list.bucket();

        assert_eq!(bucket.name(), "abc");

        assert!(object_list.prefix() == "foo2");
        assert_eq!(object_list.prefix(), "foo2");

        assert!(object_list.max_keys() == &100u32);
        assert_eq!(object_list.max_keys().to_owned(), 100u32);

        match &object_list.next_continuation_token() {
            Some(a) => {
                assert!(a == "foo3");
                assert_eq!(a, "foo3");
            }
            None => {
                panic!("token is valid value");
            }
        }
    }

    #[test]
    fn test_next_query() {
        let object_list = init_object_list(Some(String::from("foo3")), vec![]);

        let query = object_list.next_query();

        assert!(query.is_some());
        let inner_query = query.unwrap();
        assert_eq!(
            inner_query.get("key1"),
            Some(&QueryValue::from_static("value1"))
        );
        assert_eq!(
            inner_query.get("continuation-token"),
            Some(&QueryValue::from_static("foo3"))
        );

        let object_list = init_object_list(None, vec![]);
        let query = object_list.next_query();
        assert!(query.is_none());
    }

    #[test]
    fn test_object_iter_in_list() {
        let bucket = Arc::new(BucketBase::from_str("abc.oss-cn-shanghai.aliyuncs.com").unwrap());
        let object_list = init_object_list(
            None,
            vec![
                Object::new(
                    bucket.clone(),
                    "key1",
                    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(123000, 0), Utc),
                    "foo3".into(),
                    "foo4".into(),
                    100,
                    "foo5".into(),
                ),
                Object::new(
                    bucket.clone(),
                    "key2",
                    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(123000, 0), Utc),
                    "foo3".into(),
                    "foo4".into(),
                    100,
                    "foo5".into(),
                ),
            ],
        );

        let mut iter = object_list.object_iter();
        let first = iter.next();
        assert!(first.is_some());
        assert_eq!(first.unwrap().base.path().to_str(), "key1");

        let second = iter.next();
        assert!(second.is_some());
        assert_eq!(second.unwrap().base.path().to_str(), "key2");

        let third = iter.next();
        assert!(third.is_none());
    }

    #[test]
    fn test_object_new() {
        let bucket = Arc::new(BucketBase::from_str("abc.oss-cn-shanghai.aliyuncs.com").unwrap());
        let object = Object::<ArcPointer>::new(
            bucket,
            "foo2",
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(123000, 0), Utc),
            "foo3".into(),
            "foo4".into(),
            100,
            "foo5".into(),
        );

        assert_eq!(object.base.path().to_str(), "foo2");
        assert_eq!(object.last_modified.to_string(), "1970-01-02 10:10:00 UTC");
        assert_eq!(object.etag, "foo3");
        assert_eq!(object._type, "foo4");
        assert_eq!(object.size, 100);
        assert_eq!(object.storage_class, "foo5");
    }

    #[test]
    fn test_object_builder() {
        let bucket = Arc::new(BucketBase::new(
            "abc".try_into().unwrap(),
            "qingdao".try_into().unwrap(),
        ));
        let object = ObjectBuilder::<ArcPointer>::new(bucket, "abc")
            .last_modified(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(123000, 0),
                Utc,
            ))
            .etag("foo1".to_owned())
            .set_type("foo2".to_owned())
            .size(123)
            .storage_class("foo3".to_owned())
            .build();

        assert_eq!(object.base.path().to_str(), "abc");
        assert_eq!(object.last_modified.to_string(), "1970-01-02 10:10:00 UTC");
        assert_eq!(object.etag, "foo1");
        assert_eq!(object._type, "foo2");
        assert_eq!(object.size, 123);
        assert_eq!(object.storage_class, "foo3");
    }
}

#[cfg(feature = "blocking")]
#[cfg(test)]
mod blocking_tests {
    use std::rc::Rc;

    use chrono::{DateTime, NaiveDateTime, Utc};

    use crate::{builder::RcPointer, config::BucketBase};

    use super::Object;

    fn init_object(
        bucket: &'static str,
        path: &'static str,
        last_modified: i64,
        etag: &'static str,
        _type: &'static str,
        size: u64,
        storage_class: &'static str,
    ) -> Object<RcPointer> {
        let bucket = Rc::new(BucketBase::from_str(bucket).unwrap());
        Object::<RcPointer>::new(
            bucket,
            path,
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(last_modified, 0), Utc),
            etag.into(),
            _type.into(),
            size,
            storage_class.into(),
        )
    }

    #[test]
    fn test_object_eq() {
        let object1 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            "sc_foo",
        );

        let object2 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            "sc_foo",
        );

        assert!(object1 == object2);

        let object3 = init_object(
            "abc2.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            "sc_foo",
        );

        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo2",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            "sc_foo",
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123009,
            "efoo1",
            "tyfoo1",
            12,
            "sc_foo",
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo2",
            "tyfoo1",
            12,
            "sc_foo",
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo3",
            12,
            "sc_foo",
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            256,
            "sc_foo",
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            "sc_fo2323",
        );
        assert!(object1 != object3);
    }
}
