//! # Object 相关功能
//! `ObjectList` 对应文件列表，`Object` 对应的是单个文件对象
//!
//! `ObjectList` 也可以支持自定义的类型存储单个文件对象，例如 [issue 12](https://github.com/tu6ge/oss-rs/issues/12) 提到的，有些时候，
//! Oss 接口不仅返回文件路径，还会返回目录路径，可以使用如下例子进行适配
//!
//! ```rust,no_run
//! use aliyun_oss_client::{
//!     builder::{ArcPointer, BuilderError},
//!     config::{InvalidObjectDir, ObjectDir, ObjectPath},
//!     decode::{ItemError, RefineObject},
//!     object::ObjectList,
//!     BucketName, Client,
//! };
//! use dotenv::dotenv;
//!
//! #[derive(Debug)]
//! enum MyObject {
//!     File(ObjectPath),
//!     Dir(ObjectDir<'static>),
//! }
//!
//! impl RefineObject<MyError> for MyObject {
//!     fn set_key(&mut self, key: &str) -> Result<(), MyError> {
//!         let res = key.parse::<ObjectPath>();
//!
//!         *self = match res {
//!             Ok(file) => MyObject::File(file),
//!             _ => {
//!                 let re = key.parse::<ObjectDir>();
//!                 MyObject::Dir(re.unwrap())
//!             }
//!         };
//!
//!         Ok(())
//!     }
//! }
//!
//! struct MyError(String);
//!
//! use std::fmt::{self, Display};
//!
//! impl Display for MyError {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         f.write_fmt(format_args!("{}", self.0))
//!     }
//! }
//! impl ItemError for MyError {}
//!
//! impl From<InvalidObjectDir> for MyError {
//!     fn from(value: InvalidObjectDir) -> Self {
//!         Self(value.to_string())
//!     }
//! }
//!
//! type MyList = ObjectList<ArcPointer, MyObject, MyError>;
//!
//! #[tokio::main]
//! async fn main() {
//!     dotenv().ok();
//!
//!     let client = Client::from_env().unwrap();
//!
//!     let mut list = MyList::default();
//!
//!     let init_object = || MyObject::File(ObjectPath::default());
//!
//!     let _ = client
//!         .base_object_list(
//!             "xxxxxx".parse::<BucketName>().unwrap(),
//!             [],
//!             &mut list,
//!             init_object,
//!         )
//!         .await;
//!
//!     println!("list: {:?}", list.object_list);
//! }
//! ```

#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
use crate::builder::{ArcPointer, BuilderError, PointerFamily};
use crate::client::ClientArc;
#[cfg(feature = "blocking")]
use crate::client::ClientRc;
use crate::config::{
    BucketBase, CommonPrefixes, InvalidObjectPath, ObjectBase, ObjectDir, ObjectPath,
};
use crate::decode::{InnerListError, ItemError, ListError, RefineObject, RefineObjectList};
use crate::errors::{OssError, OssResult};
#[cfg(feature = "blocking")]
use crate::file::blocking::AlignBuilder as BlockingAlignBuilder;
use crate::file::AlignBuilder;
use crate::types::{
    CanonicalizedResource, Query, QueryKey, QueryValue, UrlQuery, CONTINUATION_TOKEN,
};
use crate::{BucketName, Client};
use async_stream::try_stream;
use chrono::{DateTime, NaiveDateTime, Utc};
use futures_core::stream::Stream;
use http::Method;
use oss_derive::oss_gen_rc;

use std::fmt;
use std::marker::PhantomData;
use std::num::ParseIntError;
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::sync::Arc;
use std::vec::IntoIter;

/// # 存放对象列表的结构体
/// TODO impl core::ops::Index
#[derive(Clone)]
#[non_exhaustive]
pub struct ObjectList<
    P: PointerFamily = ArcPointer,
    Item: RefineObject<E> = Object<P>,
    E: ItemError = BuildInItemError,
> {
    pub(crate) bucket: BucketBase,
    prefix: String,
    max_keys: u32,
    key_count: u64,
    /// 存放单个文件对象的 Vec 集合
    pub object_list: Vec<Item>,
    next_continuation_token: Option<String>,
    common_prefixes: CommonPrefixes,
    client: P::PointerType,
    search_query: Query,
    ph_err: PhantomData<E>,
}

impl<T: PointerFamily, Item: RefineObject<E>, E: ItemError> fmt::Debug for ObjectList<T, Item, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ObjectList")
            .field("bucket", &self.bucket)
            .field("prefix", &self.prefix)
            .field("max_keys", &self.max_keys)
            .field("key_count", &self.key_count)
            .field("next_continuation_token", &self.next_continuation_token)
            .field("common_prefixes", &self.common_prefixes)
            .field("search_query", &self.search_query)
            .finish()
    }
}

#[oss_gen_rc]
impl<Item: RefineObject<E>, E: ItemError> Default for ObjectList<ArcPointer, Item, E> {
    fn default() -> Self {
        Self {
            bucket: BucketBase::default(),
            prefix: String::default(),
            max_keys: u32::default(),
            key_count: u64::default(),
            object_list: Vec::new(),
            next_continuation_token: None,
            common_prefixes: CommonPrefixes::default(),
            client: Arc::new(ClientArc::default()),
            search_query: Query::default(),
            ph_err: PhantomData,
        }
    }
}

impl<T: PointerFamily, Item: RefineObject<E>, E: ItemError> ObjectList<T, Item, E> {
    /// 文件列表的初始化方法
    #[allow(clippy::too_many_arguments)]
    pub fn new<Q: IntoIterator<Item = (QueryKey, QueryValue)>>(
        bucket: BucketBase,
        prefix: String,
        max_keys: u32,
        key_count: u64,
        object_list: Vec<Item>,
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
            common_prefixes: CommonPrefixes::default(),
            client,
            search_query: Query::from_iter(search_query),
            ph_err: PhantomData,
        }
    }

    /// 返回 bucket 元信息的引用
    pub fn bucket(&self) -> &BucketBase {
        &self.bucket
    }

    /// 返回 prefix 的引用
    pub fn prefix(&self) -> &String {
        &self.prefix
    }

    /// 获取文件夹下的子文件夹名，子文件夹下递归的所有文件和文件夹不包含在这里。
    pub fn common_prefixes(&self) -> &CommonPrefixes {
        &self.common_prefixes
    }

    /// 设置 common_prefixes 信息
    pub fn set_common_prefixes<P: IntoIterator<Item = ObjectDir<'static>>>(&mut self, prefixes: P) {
        self.common_prefixes = CommonPrefixes::from_iter(prefixes);
    }

    /// 返回 max_keys
    pub fn max_keys(&self) -> &u32 {
        &self.max_keys
    }

    /// 返回 key_count
    pub fn key_count(&self) -> &u64 {
        &self.key_count
    }

    /// # 返回下一个 continuation_token
    /// 用于翻页使用
    pub fn next_continuation_token(&self) -> &Option<String> {
        &self.next_continuation_token
    }

    /// 返回查询条件
    pub fn search_query(&self) -> &Query {
        &self.search_query
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
    pub fn object_iter(self) -> IntoIter<Item> {
        self.object_list.into_iter()
    }
}

#[oss_gen_rc]
impl<Item: RefineObject<E>, E: ItemError> ObjectList<ArcPointer, Item, E> {
    /// 设置 Client
    pub(crate) fn set_client(&mut self, client: Arc<ClientArc>) {
        self.client = client;
    }

    /// 获取 Client 引用
    pub(crate) fn client(&self) -> Arc<ClientArc> {
        Arc::clone(&self.client)
    }
}

impl ObjectList<ArcPointer> {
    /// 异步获取下一页的数据
    pub async fn get_next_list(&self) -> OssResult<ObjectList<ArcPointer>> {
        match self.next_query() {
            None => Err(OssError::WithoutMore),
            Some(query) => {
                let mut url = self.bucket.to_url();
                url.set_search_query(&query);

                let canonicalized = CanonicalizedResource::from_bucket_query(&self.bucket, &query);

                let response = self.builder(Method::GET, url, canonicalized)?;
                let content = response.send_adjust_error().await?;

                let mut list = ObjectList::<ArcPointer>::default();
                list.set_client(self.client());
                list.set_bucket(self.bucket.clone());

                let bucket_arc = Arc::new(self.bucket.clone());

                let init_object = || {
                    let mut object = Object::<ArcPointer>::default();
                    object.base.set_bucket(bucket_arc.clone());
                    object
                };

                list.decode(&content.text().await?, init_object)?;

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
    /// # let query = [("max-keys".into(), 100u8.into())];
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

impl<Item: RefineObject<E>, E: ItemError> ObjectList<ArcPointer, Item, E> {
    /// 自定义 Item 时，获取下一页数据
    pub async fn get_next_base<F>(&self, f: F) -> OssResult<Self>
    where
        F: FnMut() -> Item,
    {
        match self.next_query() {
            None => Err(OssError::WithoutMore),
            Some(query) => {
                let mut list = Self::default();
                let name = self.bucket.get_name().clone();
                self.client()
                    .base_object_list(name, query, &mut list, f)
                    .await?;

                Ok(list)
            }
        }
    }
}

#[cfg(feature = "blocking")]
impl ObjectList<RcPointer> {
    /// 从 OSS 获取 object 列表信息
    pub fn get_object_list(&mut self) -> Result<Self, ExtractListError> {
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

        client
            .base_object_list(
                name.to_owned(),
                self.search_query.clone(),
                &mut list,
                init_object,
            )
            .map_err(ExtractListError::from)?;

        Ok(list)
    }
}

impl<T: PointerFamily, Item: RefineObject<E>, E: ItemError> ObjectList<T, Item, E> {
    /// 设置查询条件
    #[inline]
    pub fn set_search_query(&mut self, search_query: Query) {
        self.search_query = search_query;
    }

    /// 设置 bucket 元信息
    pub fn set_bucket(&mut self, bucket: BucketBase) {
        self.bucket = bucket;
    }

    /// 获取 bucket 名称
    pub fn bucket_name(&self) -> &str {
        self.bucket.name()
    }

    /// 返回文件数量
    pub fn len(&self) -> usize {
        self.object_list.len()
    }

    /// 返回是否存在文件
    pub fn is_empty(&self) -> bool {
        self.object_list.is_empty()
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
    storage_class: StorageClass,
}

impl<T: PointerFamily> Default for Object<T> {
    fn default() -> Self {
        Object {
            base: ObjectBase::<T>::default(),
            last_modified: DateTime::<Utc>::from_utc(
                #[allow(clippy::unwrap_used)]
                NaiveDateTime::from_timestamp_opt(61, 0).unwrap(),
                Utc,
            ),
            etag: String::default(),
            _type: String::default(),
            size: 0,
            storage_class: StorageClass::default(),
        }
    }
}

impl<T: PointerFamily> Object<T> {
    /// 初始化 Object 结构体
    pub fn new(
        bucket: T::Bucket,
        path: ObjectPath,
        last_modified: DateTime<Utc>,
        etag: String,
        _type: String,
        size: u64,
        storage_class: StorageClass,
    ) -> Self {
        let base = ObjectBase::<T>::new2(bucket, path);
        Self {
            base,
            last_modified,
            etag,
            _type,
            size,
            storage_class,
        }
    }

    /// 读取 Object 元信息
    #[inline]
    pub fn base(&self) -> &ObjectBase<T> {
        &self.base
    }

    /// 设置 Object 元信息
    #[inline]
    pub fn set_base(&mut self, base: ObjectBase<T>) {
        self.base = base;
    }

    /// 读取最后修改时间
    #[inline]
    pub fn last_modified(&self) -> &DateTime<Utc> {
        &self.last_modified
    }

    /// 设置最后修改时间
    #[inline]
    pub fn set_last_modified(&mut self, last_modified: DateTime<Utc>) {
        self.last_modified = last_modified;
    }

    /// 读取 etag 信息
    #[inline]
    pub fn etag(&self) -> &String {
        &self.etag
    }

    /// 设置 etag
    #[inline]
    pub fn set_etag(&mut self, etag: String) {
        self.etag = etag
    }

    /// 读取 type
    #[inline]
    pub fn get_type_string(&self) -> &String {
        &self._type
    }

    /// 设置 type
    #[inline]
    pub fn set_type_string(&mut self, _type: String) {
        self._type = _type;
    }

    /// 读取文件 size
    #[inline]
    pub fn size(&self) -> u64 {
        self.size
    }

    /// 设置文件 size
    #[inline]
    pub fn set_size(&mut self, size: u64) {
        self.size = size;
    }

    /// 读取 storage_class
    #[inline]
    pub fn storage_class(&self) -> &StorageClass {
        &self.storage_class
    }

    /// 设置 storage_class
    #[inline]
    pub fn set_storage_class(&mut self, storage_class: StorageClass) {
        self.storage_class = storage_class;
    }

    /// 获取一部分数据
    pub fn pieces(
        self,
    ) -> (
        ObjectBase<T>,
        DateTime<Utc>,
        String,
        String,
        u64,
        StorageClass,
    ) {
        (
            self.base,
            self.last_modified,
            self.etag,
            self._type,
            self.size,
            self.storage_class,
        )
    }

    /// 读取 文件路径
    pub fn path(&self) -> ObjectPath {
        self.base.path()
    }

    #[doc(hidden)]
    pub fn path_string(&self) -> String {
        self.base.path().to_string()
    }
}

/// Object 结构体的构建器
pub struct ObjectBuilder<T: PointerFamily = ArcPointer> {
    object: Object<T>,
}

impl<T: PointerFamily> ObjectBuilder<T> {
    /// 初始化 Object 构建器
    pub fn new<P: Into<ObjectPath>>(bucket: T::Bucket, key: P) -> Self {
        let base = ObjectBase::<T>::new2(bucket, key.into());
        Self {
            object: Object {
                base,
                last_modified: DateTime::<Utc>::from_utc(
                    #[allow(clippy::unwrap_used)]
                    NaiveDateTime::from_timestamp_opt(61, 0).unwrap(),
                    Utc,
                ),
                etag: String::default(),
                _type: String::default(),
                size: 0,
                storage_class: StorageClass::default(),
            },
        }
    }

    /// 设置 last_modified
    pub fn last_modified(mut self, date: DateTime<Utc>) -> Self {
        self.object.last_modified = date;
        self
    }

    /// 设置 etag
    pub fn etag(mut self, etag: String) -> Self {
        self.object.etag = etag;
        self
    }

    /// 设置 type
    pub fn set_type(mut self, _type: String) -> Self {
        self.object._type = _type;
        self
    }

    /// 设置 size
    pub fn size(mut self, size: u64) -> Self {
        self.object.size = size;
        self
    }

    /// 设置 storage_class
    pub fn storage_class(mut self, storage_class: StorageClass) -> Self {
        self.object.storage_class = storage_class;
        self
    }

    /// 返回 object
    pub fn build(self) -> Object<T> {
        self.object
    }
}

impl<T: PointerFamily + Sized> RefineObject<BuildInItemError> for Object<T> {
    #[inline]
    fn set_key(&mut self, key: &str) -> Result<(), BuildInItemError> {
        self.base
            .set_path(key.to_owned())
            .map_err(BuildInItemError::from)
    }

    #[inline]
    fn set_last_modified(&mut self, value: &str) -> Result<(), BuildInItemError> {
        self.last_modified = value.parse().map_err(BuildInItemError::from)?;
        Ok(())
    }

    #[inline]
    fn set_etag(&mut self, value: &str) -> Result<(), BuildInItemError> {
        self.etag = value.to_string();
        Ok(())
    }

    #[inline]
    fn set_type(&mut self, value: &str) -> Result<(), BuildInItemError> {
        self._type = value.to_string();
        Ok(())
    }

    #[inline]
    fn set_size(&mut self, size: &str) -> Result<(), BuildInItemError> {
        self.size = size.parse().map_err(BuildInItemError::from)?;
        Ok(())
    }

    #[inline]
    fn set_storage_class(&mut self, storage_class: &str) -> Result<(), BuildInItemError> {
        let start_char = storage_class
            .chars()
            .next()
            .ok_or(BuildInItemError::InvalidStorageClass)?;

        match start_char {
            'a' | 'A' => self.storage_class = StorageClass::Archive,
            'i' | 'I' => self.storage_class = StorageClass::IA,
            's' | 'S' => self.storage_class = StorageClass::Standard,
            'c' | 'C' => self.storage_class = StorageClass::ColdArchive,
            _ => return Err(BuildInItemError::InvalidStorageClass),
        }
        Ok(())
    }
}

impl<P: PointerFamily, Item: RefineObject<E>, E: ItemError> RefineObjectList<Item, OssError, E>
    for ObjectList<P, Item, E>
{
    #[inline]
    fn set_key_count(&mut self, key_count: &str) -> Result<(), OssError> {
        self.key_count = key_count.parse().map_err(OssError::from)?;
        Ok(())
    }

    #[inline]
    fn set_prefix(&mut self, prefix: &str) -> Result<(), OssError> {
        self.prefix = prefix.to_owned();
        Ok(())
    }

    #[inline]
    fn set_common_prefix(&mut self, list: &[std::borrow::Cow<'_, str>]) -> Result<(), OssError> {
        for val in list.iter() {
            self.common_prefixes
                .push(val.parse().map_err(OssError::from)?);
        }
        Ok(())
    }

    #[inline]
    fn set_max_keys(&mut self, max_keys: &str) -> Result<(), OssError> {
        self.max_keys = max_keys.parse().map_err(OssError::from)?;
        Ok(())
    }

    #[inline]
    fn set_next_continuation_token(&mut self, token: Option<&str>) -> Result<(), OssError> {
        self.next_continuation_token = token.map(|t| t.to_owned());
        Ok(())
    }

    #[inline]
    fn set_list(&mut self, list: Vec<Item>) -> Result<(), OssError> {
        self.object_list = list;
        Ok(())
    }
}

/// Xml 转化为内置 Object 时的错误集合
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BuildInItemError {
    /// 转换数字类型的错误
    #[error("{0}")]
    ParseInt(#[from] ParseIntError),

    /// 转换为 ObjectPath 时的错误
    #[error("{0}")]
    Path(#[from] InvalidObjectPath),

    /// 转换日期格式的错误
    #[error("{0}")]
    ParseDate(#[from] chrono::ParseError),

    /// 接收 Xml 转换时的错误
    #[error("{0}")]
    Xml(#[from] quick_xml::Error),

    /// 非法的 StorageClass
    #[error("invalid storage class")]
    InvalidStorageClass,
}

impl ItemError for BuildInItemError {}

impl Client {
    /// 查询默认 bucket 的文件列表
    ///
    /// 查询条件参数有多种方式，具体参考 [`get_object_list`] 文档
    ///
    /// [`get_object_list`]: crate::bucket::Bucket::get_object_list
    pub async fn get_object_list<Q: IntoIterator<Item = (QueryKey, QueryValue)>>(
        self,
        query: Q,
    ) -> Result<ObjectList, ExtractListError> {
        let bucket = BucketBase::new(
            self.get_bucket_name().to_owned(),
            self.get_endpoint().to_owned(),
        );

        let mut list = ObjectList::<ArcPointer>::default();
        list.set_bucket(bucket.clone());

        let bucket_arc = Arc::new(bucket.clone());
        let init_object = || {
            let mut object = Object::<ArcPointer>::default();
            object.base.set_bucket(bucket_arc.clone());
            object
        };

        let query = Query::from_iter(query);

        let (bucket_url, resource) = bucket.get_url_resource(&query);

        let response = self.builder(Method::GET, bucket_url, resource)?;
        let content = response.send_adjust_error().await?;

        list.decode(
            &content.text().await.map_err(BuilderError::from)?,
            init_object,
        )
        .map_err(ExtractListError::from)?;

        list.set_client(Arc::new(self));
        list.set_search_query(query);

        Ok(list)
    }

    /// # 可将 object 列表导出到外部类型
    /// 可以参考下面示例，或者项目中的 `examples/custom.rs`
    /// ## 示例
    /// ```rust
    /// use aliyun_oss_client::{
    ///     decode::{ItemError, ListError, RefineObject, RefineObjectList},
    ///     object::ExtractListError,
    ///     Client,
    /// };
    /// use dotenv::dotenv;
    /// use thiserror::Error;
    ///
    /// #[derive(Debug)]
    /// struct MyFile {
    ///     key: String,
    ///     #[allow(dead_code)]
    ///     other: String,
    /// }
    /// impl RefineObject<MyError> for MyFile {
    ///     fn set_key(&mut self, key: &str) -> Result<(), MyError> {
    ///         self.key = key.to_string();
    ///         Ok(())
    ///     }
    /// }
    ///
    /// #[derive(Default, Debug)]
    /// struct MyBucket {
    ///     name: String,
    ///     files: Vec<MyFile>,
    /// }
    ///
    /// impl RefineObjectList<MyFile, MyError> for MyBucket {
    ///     fn set_name(&mut self, name: &str) -> Result<(), MyError> {
    ///         self.name = name.to_string();
    ///         Ok(())
    ///     }
    ///     fn set_list(&mut self, list: Vec<MyFile>) -> Result<(), MyError> {
    ///         self.files = list;
    ///         Ok(())
    ///     }
    /// }
    ///
    /// #[derive(Debug, Error)]
    /// #[error("my error")]
    /// enum MyError {}
    ///
    /// impl ItemError for MyError {}
    /// impl ListError for MyError {}
    ///
    /// async fn run() -> Result<(), ExtractListError> {
    ///     dotenv().ok();
    ///     use aliyun_oss_client::BucketName;
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
    ///     let bucket_name = "abc".parse::<BucketName>().unwrap();
    ///
    ///     client
    ///         .base_object_list(bucket_name, [], &mut bucket, init_file)
    ///         .await?;
    ///
    ///     println!("bucket: {:?}", bucket);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn base_object_list<
        Name: Into<BucketName>,
        Q: IntoIterator<Item = (QueryKey, QueryValue)>,
        List,
        Item,
        F,
        E: ListError,
        ItemErr: ItemError,
    >(
        &self,
        name: Name,
        query: Q,
        list: &mut List,
        init_object: F,
    ) -> Result<(), ExtractListError>
    where
        List: RefineObjectList<Item, E, ItemErr>,
        Item: RefineObject<ItemErr>,
        F: FnMut() -> Item,
    {
        let query = Query::from_iter(query);

        let (bucket_url, resource) =
            BucketBase::new(name.into(), self.get_endpoint().to_owned()).get_url_resource(&query);

        let response = self.builder(Method::GET, bucket_url, resource)?;
        let content = response.send_adjust_error().await?;

        list.decode(
            &content.text().await.map_err(BuilderError::from)?,
            init_object,
        )
        .map_err(ExtractListError::from)?;

        Ok(())
    }
}

/// 为 [`base_object_list`] 方法，返回一个统一的 Error
///
/// [`base_object_list`]: crate::client::Client::base_object_list
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ExtractListError {
    #[doc(hidden)]
    #[error("{0}")]
    Builder(#[from] BuilderError),

    #[doc(hidden)]
    #[error("{0}")]
    List(#[from] InnerListError),
}

#[cfg(feature = "blocking")]
impl ClientRc {
    /// 查询默认 bucket 的文件列表
    ///
    /// 查询条件参数有多种方式，具体参考 [`get_object_list`](../bucket/struct.Bucket.html#method.get_object_list) 文档
    pub fn get_object_list<Q: IntoIterator<Item = (QueryKey, QueryValue)>>(
        self,
        query: Q,
    ) -> Result<ObjectList<RcPointer>, ExtractListError> {
        let name = self.get_bucket_name();
        let bucket = BucketBase::new(name.clone(), self.get_endpoint().to_owned());

        let mut list = ObjectList::<RcPointer>::default();
        list.set_bucket(bucket.clone());

        let bucket_arc = Rc::new(bucket);

        let init_object = || {
            let mut object = Object::<RcPointer>::default();
            object.base.set_bucket(bucket_arc.clone());
            object
        };

        let query = Query::from_iter(query);

        let (bucket_url, resource) = bucket_arc.get_url_resource(&query);

        let response = self.builder(Method::GET, bucket_url, resource)?;
        let content = response.send_adjust_error()?;

        list.decode(&content.text().map_err(BuilderError::from)?, init_object)?;

        list.set_client(Rc::new(self));
        list.set_search_query(query);

        Ok(list)
    }

    /// 可将 object 列表导出到外部 struct
    #[inline]
    pub fn base_object_list<
        Name: Into<BucketName>,
        Q: IntoIterator<Item = (QueryKey, QueryValue)>,
        List,
        Item,
        F,
        E: ListError,
        ItemErr: ItemError,
    >(
        &self,
        name: Name,
        query: Q,
        list: &mut List,
        init_object: F,
    ) -> Result<(), ExtractListError>
    where
        List: RefineObjectList<Item, E, ItemErr>,
        Item: RefineObject<ItemErr>,
        F: FnMut() -> Item,
    {
        let bucket = BucketBase::new(name.into(), self.get_endpoint().to_owned());

        let query = Query::from_iter(query);
        let (bucket_url, resource) = bucket.get_url_resource(&query);

        let response = self.builder(Method::GET, bucket_url, resource)?;
        let content = response.send_adjust_error()?;

        list.decode(&content.text().map_err(BuilderError::from)?, init_object)
            .map_err(ExtractListError::from)?;

        Ok(())
    }
}

#[cfg(feature = "blocking")]
impl Iterator for ObjectList<RcPointer> {
    type Item = ObjectList<RcPointer>;
    fn next(&mut self) -> Option<Self> {
        self.next_continuation_token.clone().and_then(|token| {
            self.search_query.insert(CONTINUATION_TOKEN, token);
            self.get_object_list().ok()
        })
    }
}

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

//                 let builder = self.builder(Method::GET, url, canonicalized);
//                 match builder {
//                     Err(err) => Ready(None),
//                     Ok(builder) => {
//                         let waker = cx.waker().clone();

//                         std::thread::spawn(move || {
//                             let response = builder.send_adjust_error();

//                             let response = futures::executor::block_on(response);
//                             let text = response.unwrap().text();
//                             let text = futures::executor::block_on(text);

//                             let text = text.unwrap();

//                             let bucket_arc = Arc::new(self.bucket);

//                             let init_object = || {
//                                 let object = Object::<ArcPointer>::default();
//                                 object.base.set_bucket(bucket_arc.clone());
//                                 object
//                             };

//                             self.decode(&text, init_object).unwrap();

//                             self.set_search_query(query);

//                             waker.wake();
//                         });

//                         Poll::Pending
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
#[doc(hidden)]
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
#[doc(hidden)]
pub enum Encryption {
    #[default]
    Aes256,
    Kms,
    Sm4,
}

/// 未来计划支持的功能
#[derive(Default)]
#[doc(hidden)]
pub enum ObjectAcl {
    #[default]
    Default,
    Private,
    PublicRead,
    PublicReadWrite,
}

/// 存储类型
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum StorageClass {
    /// Standard 默认
    #[default]
    Standard,
    /// IA
    IA,
    /// Archive
    Archive,
    /// ColdArchive
    ColdArchive,
}

/// 未来计划支持的功能
#[derive(Default)]
#[doc(hidden)]
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
#[doc(hidden)]
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
        config::{BucketBase, ObjectPath},
        object::{Object, ObjectBuilder, StorageClass},
        types::QueryValue,
        Client,
    };
    use chrono::{DateTime, NaiveDateTime, Utc};
    use std::sync::Arc;

    fn init_object_list(token: Option<String>, list: Vec<Object>) -> ObjectList {
        let client = Client::new(
            "foo1".into(),
            "foo2".into(),
            "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            "foo4".parse().unwrap(),
        );

        let object_list = ObjectList::<ArcPointer>::new(
            "abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            String::from("foo2"),
            100,
            200,
            list,
            token,
            Arc::new(client),
            vec![("key1".into(), "value1".into())],
        );

        object_list
    }

    #[test]
    fn test_object_list_fmt() {
        let object_list = init_object_list(Some(String::from("foo3")), vec![]);
        assert_eq!(
            format!("{object_list:?}"),
            "ObjectList { bucket: BucketBase { endpoint: CnShanghai, name: BucketName(\"abc\") }, prefix: \"foo2\", max_keys: 100, key_count: 200, next_continuation_token: Some(\"foo3\"), common_prefixes: [], search_query: Query { inner: {Custom(\"key1\"): QueryValue(\"value1\")} } }"
        );
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
    fn test_bucket_name() {
        let object_list = init_object_list(Some(String::from("foo3")), vec![]);
        let bucket_name = object_list.bucket_name();

        assert!("abc" == bucket_name);
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
        let bucket = Arc::new("abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap());
        let object_list = init_object_list(
            None,
            vec![
                Object::new(
                    Arc::clone(&bucket),
                    "key1".parse().unwrap(),
                    DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp_opt(123000, 0).unwrap(),
                        Utc,
                    ),
                    "foo3".into(),
                    "foo4".into(),
                    100,
                    StorageClass::IA,
                ),
                Object::new(
                    Arc::clone(&bucket),
                    "key2".parse().unwrap(),
                    DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp_opt(123000, 0).unwrap(),
                        Utc,
                    ),
                    "foo3".into(),
                    "foo4".into(),
                    100,
                    StorageClass::IA,
                ),
            ],
        );

        let mut iter = object_list.object_iter();
        let first = iter.next();
        assert!(first.is_some());
        assert_eq!(first.unwrap().base.path().as_ref(), "key1");

        let second = iter.next();
        assert!(second.is_some());
        assert_eq!(second.unwrap().base.path().as_ref(), "key2");

        let third = iter.next();
        assert!(third.is_none());
    }

    #[test]
    fn test_common_prefixes() {
        let mut object_list = init_object_list(None, vec![]);
        let list = object_list.common_prefixes();
        assert!(list.len() == 0);

        object_list.set_common_prefixes(["abc/".parse().unwrap(), "cde/".parse().unwrap()]);
        let list = object_list.common_prefixes();

        assert!(list.len() == 2);
        assert!(list[0] == "abc/");
        assert!(list[1] == "cde/");
    }

    #[test]
    fn test_object_new() {
        let bucket = Arc::new("abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap());
        let object = Object::<ArcPointer>::new(
            bucket,
            "foo2".parse().unwrap(),
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(123000, 0).unwrap(), Utc),
            "foo3".into(),
            "foo4".into(),
            100,
            StorageClass::IA,
        );

        assert_eq!(object.base.path().as_ref(), "foo2");
        assert_eq!(object.last_modified.to_string(), "1970-01-02 10:10:00 UTC");
        assert_eq!(object.etag, "foo3");
        assert_eq!(object._type, "foo4");
        assert_eq!(object.size, 100);
        assert_eq!(object.storage_class, StorageClass::IA);
    }

    #[test]
    fn test_object_builder() {
        let bucket = Arc::new(BucketBase::new(
            "abc".parse().unwrap(),
            "qingdao".parse().unwrap(),
        ));
        let object = ObjectBuilder::<ArcPointer>::new(bucket, "abc".parse::<ObjectPath>().unwrap())
            .last_modified(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(123000, 0).unwrap(),
                Utc,
            ))
            .etag("foo1".to_owned())
            .set_type("foo2".to_owned())
            .size(123)
            .storage_class(StorageClass::IA)
            .build();

        assert_eq!(object.base.path().as_ref(), "abc");
        assert_eq!(object.last_modified.to_string(), "1970-01-02 10:10:00 UTC");
        assert_eq!(object.etag, "foo1");
        assert_eq!(object._type, "foo2");
        assert_eq!(object.size, 123);
        assert_eq!(object.storage_class, StorageClass::IA);
    }
}

#[cfg(feature = "blocking")]
#[cfg(test)]
mod blocking_tests {
    use std::rc::Rc;

    use chrono::{DateTime, NaiveDateTime, Utc};

    use crate::builder::RcPointer;

    use super::{Object, StorageClass};

    fn init_object(
        bucket: &str,
        path: &'static str,
        last_modified: i64,
        etag: &'static str,
        _type: &'static str,
        size: u64,
        storage_class: StorageClass,
    ) -> Object<RcPointer> {
        let bucket = Rc::new(bucket.parse().unwrap());
        Object::<RcPointer>::new(
            bucket,
            path.parse().unwrap(),
            DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(last_modified, 0).unwrap(),
                Utc,
            ),
            etag.into(),
            _type.into(),
            size,
            storage_class,
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
            StorageClass::Archive,
        );

        let object2 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            StorageClass::Archive,
        );

        assert!(object1 == object2);

        let object3 = init_object(
            "abc2.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            StorageClass::Archive,
        );

        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo2",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            StorageClass::Archive,
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123009,
            "efoo1",
            "tyfoo1",
            12,
            StorageClass::Archive,
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo2",
            "tyfoo1",
            12,
            StorageClass::Archive,
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo3",
            12,
            StorageClass::Archive,
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            256,
            StorageClass::Archive,
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            StorageClass::IA,
        );
        assert!(object1 != object3);
    }
}
