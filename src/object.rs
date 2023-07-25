//! # Object 相关功能
//! `ObjectList` 对应文件列表，`Object` 对应的是单个文件对象
//!
//! `ObjectList` 也可以支持自定义的类型存储单个文件对象，例如 [issue 12] 提到的，有些时候，
//! Oss 接口不仅返回文件路径，还会返回目录路径，可以使用如下例子进行适配
//!
//! ```rust,no_run
//! use aliyun_oss_client::{
//!     decode::RefineObject,
//!     object::Objects,
//!     types::object::{InvalidObjectDir, ObjectDir, ObjectPath},
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
//! impl RefineObject<InvalidObjectDir> for MyObject {
//!     fn set_key(&mut self, key: &str) -> Result<(), InvalidObjectDir> {
//!         *self = match key.parse() {
//!             Ok(file) => MyObject::File(file),
//!             _ => MyObject::Dir(key.parse()?),
//!         };
//!
//!         Ok(())
//!     }
//! }
//!
//! type MyList = Objects<MyObject>;
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
//!         .base_object_list([], &mut list, init_object)
//!         .await;
//!     // 第二页数据
//!     let second = list.get_next_base(init_object).await;
//!
//!     println!("list: {:?}", list.to_vec());
//! }
//! ```
//! [issue 12]: https://github.com/tu6ge/oss-rs/issues/12

use crate::bucket::Bucket;
#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
use crate::builder::{ArcPointer, BuilderError, PointerFamily};
use crate::client::ClientArc;
#[cfg(feature = "blocking")]
use crate::client::ClientRc;
use crate::config::BucketBase;
use crate::decode::{InnerListError, ListError, RefineObject, RefineObjectList};
#[cfg(feature = "blocking")]
use crate::file::blocking::AlignBuilder as BlockingAlignBuilder;
use crate::file::AlignBuilder;
use crate::types::object::ObjectPathInner;
use crate::types::{
    core::SetOssQuery,
    object::{
        CommonPrefixes, InvalidObjectDir, InvalidObjectPath, ObjectBase, ObjectDir, ObjectPath,
    },
    CanonicalizedResource, Query, QueryKey, QueryValue, CONTINUATION_TOKEN,
};
use crate::{BucketName, Client, EndPoint, KeyId, KeySecret};
use async_stream::try_stream;
use chrono::{DateTime, NaiveDateTime, Utc};
use futures_core::stream::Stream;
use http::Method;
use oss_derive::oss_gen_rc;
use url::Url;

#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::{
    error::Error,
    fmt::{self, Display},
    num::ParseIntError,
    sync::Arc,
    vec::IntoIter,
};

#[cfg(test)]
mod test;

/// # 存放对象列表的结构体
/// before name is `ObjectList`
/// TODO impl core::ops::Index
#[derive(Clone)]
#[non_exhaustive]
pub struct ObjectList<P: PointerFamily = ArcPointer, Item = Object<P>> {
    pub(crate) bucket: BucketBase,
    prefix: Option<ObjectDir<'static>>,
    max_keys: u32,
    key_count: u64,
    /// 存放单个文件对象的 Vec 集合
    object_list: Vec<Item>,
    next_continuation_token: String,
    common_prefixes: CommonPrefixes,
    client: P::PointerType,
    search_query: Query,
}

/// sync ObjectList alias
pub type Objects<Item = Object<ArcPointer>> = ObjectList<ArcPointer, Item>;
/// blocking ObjectList alias
#[cfg(feature = "blocking")]
pub type ObjectsBlocking<Item = Object<RcPointer>> = ObjectList<RcPointer, Item>;

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

/// 异步的 Object struct
pub type ObjectArc = Object<ArcPointer>;

impl<T: PointerFamily, Item> fmt::Debug for ObjectList<T, Item> {
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

impl<P: PointerFamily, Item> Default for ObjectList<P, Item> {
    fn default() -> Self {
        Self {
            bucket: BucketBase::default(),
            prefix: Option::default(),
            max_keys: u32::default(),
            key_count: u64::default(),
            object_list: Vec::new(),
            next_continuation_token: String::default(),
            common_prefixes: CommonPrefixes::default(),
            client: P::PointerType::default(),
            search_query: Query::default(),
        }
    }
}

impl<T: PointerFamily, Item> AsMut<Query> for ObjectList<T, Item> {
    fn as_mut(&mut self) -> &mut Query {
        &mut self.search_query
    }
}

impl<T: PointerFamily, Item> AsRef<BucketBase> for ObjectList<T, Item> {
    fn as_ref(&self) -> &BucketBase {
        &self.bucket
    }
}

impl<T: PointerFamily, Item> AsRef<BucketName> for ObjectList<T, Item> {
    fn as_ref(&self) -> &BucketName {
        self.bucket.as_ref()
    }
}

impl<T: PointerFamily, Item> AsRef<EndPoint> for ObjectList<T, Item> {
    fn as_ref(&self) -> &EndPoint {
        self.bucket.as_ref()
    }
}

impl<T: PointerFamily, Item> ObjectList<T, Item> {
    /// 文件列表的初始化方法
    #[allow(clippy::too_many_arguments)]
    pub fn new<Q: IntoIterator<Item = (QueryKey, QueryValue)>>(
        bucket: BucketBase,
        prefix: Option<ObjectDir<'static>>,
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
            next_continuation_token: next_continuation_token.unwrap_or_default(),
            common_prefixes: CommonPrefixes::default(),
            client,
            search_query: Query::from_iter(search_query),
        }
    }

    /// 返回 bucket 元信息的引用
    pub fn bucket(&self) -> &BucketBase {
        &self.bucket
    }

    /// 返回 prefix 的引用
    pub fn prefix(&self) -> &Option<ObjectDir<'static>> {
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
    pub fn next_continuation_token_str(&self) -> &String {
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
        if !self.next_continuation_token.is_empty() {
            let mut search_query = self.search_query.clone();
            search_query.insert(CONTINUATION_TOKEN, self.next_continuation_token.to_owned());
            Some(search_query)
        } else {
            None
        }
    }

    /// 将 object 列表转化为迭代器
    pub fn object_iter(self) -> IntoIter<Item> {
        self.object_list.into_iter()
    }
}

#[oss_gen_rc]
impl<Item> ObjectList<ArcPointer, Item> {
    /// 设置 Client
    pub(crate) fn set_client(&mut self, client: Arc<ClientArc>) {
        self.client = client;
    }

    pub(crate) fn from_bucket(
        bucket: &Bucket<ArcPointer>,
        capacity: usize,
    ) -> ObjectList<ArcPointer> {
        ObjectList::<ArcPointer> {
            bucket: bucket.base.clone(),
            client: Arc::clone(&bucket.client),
            object_list: Vec::with_capacity(capacity),
            ..Default::default()
        }
    }

    /// 获取 Client 引用
    pub(crate) fn client(&self) -> Arc<ClientArc> {
        Arc::clone(&self.client)
    }

    fn clone_base(&self) -> Self {
        Self {
            client: Arc::clone(&self.client),
            bucket: self.bucket.clone(),
            search_query: self.search_query.clone(),
            max_keys: self.max_keys,
            object_list: Vec::with_capacity(self.max_keys as usize),
            ..Default::default()
        }
    }
}

impl ObjectList<ArcPointer> {
    /// 异步获取下一页的数据
    pub async fn get_next_list(&self) -> Result<ObjectList<ArcPointer>, ExtractListError> {
        match self.next_query() {
            None => Err(ExtractListError {
                kind: ExtractListErrorKind::NoMoreFile,
            }),
            Some(query) => {
                let mut url = self.bucket.to_url();
                url.set_oss_query(&query);

                let canonicalized = CanonicalizedResource::from_bucket_query(&self.bucket, &query);

                let response = self
                    .builder(Method::GET, url, canonicalized)?
                    .send_adjust_error()
                    .await?;

                let mut list = ObjectList::<ArcPointer> {
                    client: self.client(),
                    bucket: self.bucket.clone(),
                    object_list: Vec::with_capacity(query.get_max_keys()),
                    ..Default::default()
                };

                list.decode(&response.text().await?, init_object_with_list)?;

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
    pub fn into_stream(self) -> impl Stream<Item = Result<Self, ExtractListError>> {
        try_stream! {
            let result = self.get_next_list().await?;
            yield result;
        }
    }
}

impl<Item> ObjectList<ArcPointer, Item> {
    /// 自定义 Item 时，获取下一页数据
    pub async fn get_next_base<F, E>(&self, init_object: F) -> Result<Self, ExtractListError>
    where
        F: Fn(&Self) -> Item,
        Item: RefineObject<E>,
        E: Error + 'static,
    {
        match self.next_query() {
            None => Err(ExtractListError {
                kind: ExtractListErrorKind::NoMoreFile,
            }),
            Some(query) => {
                let mut list = self.clone_base();
                list.search_query = query.clone();
                self.client()
                    .base_object_list(query, &mut list, init_object)
                    .await?;

                Ok(list)
            }
        }
    }
}

#[cfg(feature = "blocking")]
impl ObjectList<RcPointer> {
    /// 从 OSS 获取 object 列表信息
    pub fn get_object_list(&self) -> Result<Self, ExtractListError> {
        let mut list = ObjectList::<RcPointer>::clone_base(self);

        let (bucket_url, resource) = self.bucket.get_url_resource(&self.search_query);

        let response = self
            .builder(Method::GET, bucket_url, resource)?
            .send_adjust_error()?;

        list.decode(&response.text()?, init_object_with_list_rc)
            .map_err(ExtractListError::from)?;

        Ok(list)
    }
}

impl<T: PointerFamily, Item> ObjectList<T, Item> {
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

    /// 返回 object 的 Vec 集合
    pub fn to_vec(self) -> Vec<Item> {
        self.object_list
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

impl<T: PointerFamily> Default for Object<T> {
    fn default() -> Self {
        Object {
            base: ObjectBase::<T>::default(),
            last_modified: DateTime::<Utc>::from_utc(
                #[allow(clippy::unwrap_used)]
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            ),
            etag: String::default(),
            _type: String::default(),
            size: 0,
            storage_class: StorageClass::default(),
        }
    }
}

impl<T: PointerFamily> AsRef<ObjectPath> for Object<T> {
    fn as_ref(&self) -> &ObjectPath {
        self.base.as_ref()
    }
}

impl<T: PointerFamily> AsRef<DateTime<Utc>> for Object<T> {
    fn as_ref(&self) -> &DateTime<Utc> {
        &self.last_modified
    }
}

impl<T: PointerFamily> AsRef<StorageClass> for Object<T> {
    fn as_ref(&self) -> &StorageClass {
        &self.storage_class
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

    pub(crate) fn from_bucket(bucket: T::Bucket) -> Self {
        Self {
            base: ObjectBase::<T>::init_with_bucket(bucket),
            ..Default::default()
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

fn init_object_with_list(list: &ObjectList) -> Object {
    Object::from_bucket(Arc::new(list.bucket.clone()))
}

#[cfg(feature = "blocking")]
fn init_object_with_list_rc(list: &ObjectList<RcPointer>) -> Object<RcPointer> {
    Object::<RcPointer>::from_bucket(Rc::new(list.bucket.clone()))
}

impl Object<ArcPointer> {
    #[cfg(test)]
    pub fn test_path(path: &'static str) -> Self {
        let mut object = Self::default();
        object.set_base(ObjectBase::<ArcPointer>::new2(
            Arc::new(BucketBase::default()),
            path.try_into().unwrap(),
        ));
        object
    }
}

impl<T: PointerFamily> From<Object<T>> for ObjectPathInner<'static> {
    #[inline]
    fn from(obj: Object<T>) -> Self {
        obj.base.path
    }
}

#[oss_gen_rc]
impl Object<ArcPointer> {
    /// # Object 构建器
    /// 用例
    /// ```
    /// # use aliyun_oss_client::{config::BucketBase, ObjectPath, object::{ObjectArc, StorageClass},EndPoint};
    /// # use chrono::{DateTime, NaiveDateTime, Utc};
    /// let bucket = BucketBase::new(
    ///     "bucket-name".parse().unwrap(),
    ///     EndPoint::CN_QINGDAO,
    /// );
    /// let mut builder = ObjectArc::builder("abc".parse::<ObjectPath>().unwrap());
    ///
    /// builder
    ///     .bucket_base(bucket)
    ///     .last_modified(DateTime::<Utc>::from_utc(
    ///         NaiveDateTime::from_timestamp_opt(123000, 0).unwrap(),
    ///         Utc,
    ///     ))
    ///     .etag("foo1".to_owned())
    ///     .set_type("foo2".to_owned())
    ///     .size(123)
    ///     .storage_class(StorageClass::IA);
    ///
    /// let object = builder.build();
    /// ```
    pub fn builder(path: ObjectPath) -> ObjectBuilder<ArcPointer> {
        ObjectBuilder::<ArcPointer>::new(Arc::default(), path)
    }

    /// 带签名的 Url 链接
    pub fn to_sign_url(&self, key: &KeyId, secret: &KeySecret, expires: i64) -> Url {
        self.base.to_sign_url(key, secret, expires)
    }
}

/// Object 结构体的构建器
pub struct ObjectBuilder<T: PointerFamily = ArcPointer> {
    object: Object<T>,
}

impl<T: PointerFamily> ObjectBuilder<T> {
    /// 初始化 Object 构建器
    pub fn new(bucket: T::Bucket, path: ObjectPath) -> Self {
        let base = ObjectBase::<T>::new2(bucket, path);
        Self {
            object: Object {
                base,
                last_modified: DateTime::<Utc>::from_utc(
                    #[allow(clippy::unwrap_used)]
                    NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                    Utc,
                ),
                ..Default::default()
            },
        }
    }

    /// 设置元信息
    pub fn bucket(&mut self, bucket: T::Bucket) -> &mut Self {
        self.object.base.set_bucket(bucket);
        self
    }

    /// 设置 last_modified
    pub fn last_modified(&mut self, date: DateTime<Utc>) -> &mut Self {
        self.object.last_modified = date;
        self
    }

    /// 设置 etag
    pub fn etag(&mut self, etag: String) -> &mut Self {
        self.object.etag = etag;
        self
    }

    /// 设置 type
    pub fn set_type(&mut self, _type: String) -> &mut Self {
        self.object._type = _type;
        self
    }

    /// 设置 size
    pub fn size(&mut self, size: u64) -> &mut Self {
        self.object.size = size;
        self
    }

    /// 设置 storage_class
    pub fn storage_class(&mut self, storage_class: StorageClass) -> &mut Self {
        self.object.storage_class = storage_class;
        self
    }

    /// 返回 object
    pub fn build(self) -> Object<T> {
        self.object
    }
}

#[oss_gen_rc]
impl ObjectBuilder<ArcPointer> {
    /// 设置元信息
    pub fn bucket_base(&mut self, base: BucketBase) -> &mut Self {
        self.object.base.set_bucket(Arc::new(base));
        self
    }
}

impl<T: PointerFamily> RefineObject<BuildInItemError> for Object<T> {
    #[inline]
    fn set_key(&mut self, key: &str) -> Result<(), BuildInItemError> {
        self.base
            .set_path(key.to_owned())
            .map_err(|e| BuildInItemError {
                source: key.to_string(),
                kind: BuildInItemErrorKind::BasePath(e),
            })
    }

    #[inline]
    fn set_last_modified(&mut self, value: &str) -> Result<(), BuildInItemError> {
        self.last_modified = value.parse().map_err(|e| BuildInItemError {
            source: value.to_string(),
            kind: BuildInItemErrorKind::LastModified(e),
        })?;
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
        self.size = size.parse().map_err(|e| BuildInItemError {
            source: size.to_string(),
            kind: BuildInItemErrorKind::Size(e),
        })?;
        Ok(())
    }

    #[inline]
    fn set_storage_class(&mut self, storage_class: &str) -> Result<(), BuildInItemError> {
        self.storage_class = StorageClass::new(storage_class).ok_or(BuildInItemError {
            source: storage_class.to_string(),
            kind: BuildInItemErrorKind::InvalidStorageClass,
        })?;
        Ok(())
    }
}

/// Xml 转化为内置 Object 时的错误集合
#[derive(Debug)]
#[non_exhaustive]
pub struct BuildInItemError {
    source: String,
    kind: BuildInItemErrorKind,
}

impl BuildInItemError {
    #[cfg(test)]
    pub(crate) fn test_new() -> Self {
        Self {
            source: "foo".to_string(),
            kind: BuildInItemErrorKind::InvalidStorageClass,
        }
    }
}

impl Display for BuildInItemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BuildInItemErrorKind::*;
        let kind = match &self.kind {
            Size(_) => "size",
            BasePath(_) => "base-path",
            LastModified(_) => "last-modified",
            InvalidStorageClass => "storage-class",
        };
        write!(f, "parse {kind} failed, gived str: {}", self.source)
    }
}

impl Error for BuildInItemError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use BuildInItemErrorKind::*;
        match &self.kind {
            Size(e) => Some(e),
            BasePath(e) => Some(e),
            LastModified(e) => Some(e),
            InvalidStorageClass => None,
        }
    }
}

/// Xml 转化为内置 Object 时的错误集合
#[derive(Debug)]
#[non_exhaustive]
enum BuildInItemErrorKind {
    /// 转换数字类型的错误
    Size(ParseIntError),

    /// 转换为 ObjectPath 时的错误
    BasePath(InvalidObjectPath),

    /// 转换日期格式的错误
    LastModified(chrono::ParseError),

    // /// 接收 Xml 转换时的错误
    // Xml(quick_xml::Error),
    /// 非法的 StorageClass
    InvalidStorageClass,
}

impl<P: PointerFamily, Item: RefineObject<E>, E: Error + 'static>
    RefineObjectList<Item, ObjectListError, E> for ObjectList<P, Item>
{
    #[inline]
    fn set_key_count(&mut self, key_count: &str) -> Result<(), ObjectListError> {
        self.key_count = key_count.parse().map_err(|e| ObjectListError {
            source: key_count.to_owned(),
            kind: ObjectListErrorKind::KeyCount(e),
        })?;
        Ok(())
    }

    #[inline]
    fn set_prefix(&mut self, prefix: &str) -> Result<(), ObjectListError> {
        if prefix.is_empty() {
            self.prefix = None;
        } else {
            let mut string = String::from(prefix);
            string += "/";
            self.prefix = Some(string.parse().map_err(|e| ObjectListError {
                source: prefix.to_owned(),
                kind: ObjectListErrorKind::Prefix(e),
            })?)
        }
        Ok(())
    }

    #[inline]
    fn set_common_prefix(
        &mut self,
        list: &[std::borrow::Cow<'_, str>],
    ) -> Result<(), ObjectListError> {
        self.common_prefixes = Vec::with_capacity(list.len());
        for val in list.iter() {
            self.common_prefixes
                .push(val.parse().map_err(|e| ObjectListError {
                    source: val.to_string(),
                    kind: ObjectListErrorKind::CommonPrefix(e),
                })?);
        }
        Ok(())
    }

    #[inline]
    fn set_max_keys(&mut self, max_keys: &str) -> Result<(), ObjectListError> {
        self.max_keys = max_keys.parse().map_err(|e| ObjectListError {
            source: max_keys.to_string(),
            kind: ObjectListErrorKind::MaxKeys(e),
        })?;
        Ok(())
    }

    #[inline]
    fn set_next_continuation_token_str(&mut self, token: &str) -> Result<(), ObjectListError> {
        self.next_continuation_token = token.to_owned();
        Ok(())
    }

    #[inline]
    fn set_list(&mut self, list: Vec<Item>) -> Result<(), ObjectListError> {
        self.object_list = list;
        Ok(())
    }
}

/// decode xml to object list error collection
#[derive(Debug)]
#[non_exhaustive]
pub struct ObjectListError {
    source: String,
    kind: ObjectListErrorKind,
}

impl ObjectListError {
    #[cfg(test)]
    pub(crate) fn test_new() -> Self {
        Self {
            source: "foo".to_string(),
            kind: ObjectListErrorKind::Bar,
        }
    }
}

impl Display for ObjectListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ObjectListErrorKind::*;
        let kind: &str = match &self.kind {
            KeyCount(_) => "key-count",
            Prefix(_) => "prefix",
            CommonPrefix(_) => "common-prefix",
            MaxKeys(_) => "max-keys",
            #[cfg(test)]
            Bar => "bar",
        };
        write!(f, "parse {kind} failed, gived str: {}", self.source)
    }
}

impl Error for ObjectListError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use ObjectListErrorKind::*;
        match &self.kind {
            KeyCount(e) | MaxKeys(e) => Some(e),
            Prefix(e) | CommonPrefix(e) => Some(e),
            #[cfg(test)]
            Bar => None,
        }
    }
}

impl ListError for ObjectListError {}

/// decode xml to object list error collection
#[derive(Debug)]
#[non_exhaustive]
enum ObjectListErrorKind {
    /// when covert key_count failed ,return this error
    KeyCount(ParseIntError),
    /// when covert prefix failed ,return this error
    Prefix(InvalidObjectDir),
    /// when covert common_prefix failed ,return this error
    CommonPrefix(InvalidObjectDir),
    /// when covert max_keys failed ,return this error
    MaxKeys(ParseIntError),
    #[cfg(test)]
    Bar,
}

impl Client {
    /// 查询默认 bucket 的文件列表
    ///
    /// 查询条件参数有多种方式，具体参考 [`get_object_list`] 文档
    ///
    /// [`get_object_list`]: crate::bucket::Bucket::get_object_list
    #[inline(always)]
    pub async fn get_object_list<Q: IntoIterator<Item = (QueryKey, QueryValue)>>(
        &self,
        query: Q,
    ) -> Result<ObjectList, ExtractListError> {
        self.get_object_list2(Query::from_iter(query)).await
    }

    /// 查询默认 bucket 的文件列表
    pub async fn get_object_list2(&self, query: Query) -> Result<ObjectList, ExtractListError> {
        let bucket = BucketBase::new(self.bucket.to_owned(), self.endpoint.to_owned());

        let (bucket_url, resource) = bucket.get_url_resource(&query);

        let mut list = ObjectList::<ArcPointer> {
            object_list: Vec::with_capacity(query.get_max_keys()),
            bucket,
            ..Default::default()
        };

        let response = self.builder(Method::GET, bucket_url, resource)?;
        let content = response.send_adjust_error().await?;

        list.decode(&content.text().await?, init_object_with_list)?;

        list.set_client(Arc::new(self.clone()));
        list.set_search_query(query);

        Ok(list)
    }

    /// # 可将 object 列表导出到外部类型（关注便捷性）
    /// 可以参考下面示例，或者项目中的 `examples/custom.rs`
    /// ## 示例
    /// ```rust
    /// use aliyun_oss_client::{
    ///     decode::{ListError, RefineObject, RefineObjectList},
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
    ///     fn init_file(_list: &MyBucket) -> MyFile {
    ///         MyFile {
    ///             key: String::default(),
    ///             other: "abc".to_string(),
    ///         }
    ///     }
    ///
    ///     client
    ///         .base_object_list([], &mut bucket, init_file)
    ///         .await?;
    ///
    ///     println!("bucket: {:?}", bucket);
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub async fn base_object_list<
        Q: IntoIterator<Item = (QueryKey, QueryValue)>,
        List,
        Item,
        F,
        E: ListError,
        ItemErr: Error + 'static,
    >(
        &self,
        query: Q,
        list: &mut List,
        init_object: F,
    ) -> Result<(), ExtractListError>
    where
        List: RefineObjectList<Item, E, ItemErr>,
        Item: RefineObject<ItemErr>,
        F: Fn(&List) -> Item,
    {
        let query = Query::from_iter(query);

        self.base_object_list2(&query, list, init_object).await
    }

    /// # 可将 object 列表导出到外部类型（关注性能）
    pub async fn base_object_list2<List, Item, F, E: ListError, ItemErr: Error + 'static>(
        &self,
        query: &Query,
        list: &mut List,
        init_object: F,
    ) -> Result<(), ExtractListError>
    where
        List: RefineObjectList<Item, E, ItemErr>,
        Item: RefineObject<ItemErr>,
        F: Fn(&List) -> Item,
    {
        let bucket = self.get_bucket_base();
        let (bucket_url, resource) = bucket.get_url_resource(query);

        let response = self.builder(Method::GET, bucket_url, resource)?;
        let content = response.send_adjust_error().await?;

        list.decode(&content.text().await?, init_object)?;

        Ok(())
    }
}

/// 为 [`base_object_list`] 方法，返回一个统一的 Error
///
/// [`base_object_list`]: crate::client::Client::base_object_list
#[derive(Debug)]
#[non_exhaustive]
pub struct ExtractListError {
    pub(crate) kind: ExtractListErrorKind,
}

/// [`ExtractListError`] 类型的枚举
///
/// [`ExtractListError`]: crate::object::ExtractListError
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum ExtractListErrorKind {
    #[doc(hidden)]
    Builder(BuilderError),

    #[doc(hidden)]
    Reqwest(reqwest::Error),

    /// 解析 xml 错误
    Decode(InnerListError),

    /// 用于 Stream
    NoMoreFile,
}

impl From<InnerListError> for ExtractListError {
    fn from(value: InnerListError) -> Self {
        use ExtractListErrorKind::*;
        Self {
            kind: Decode(value),
        }
    }
}
impl From<BuilderError> for ExtractListError {
    fn from(value: BuilderError) -> Self {
        use ExtractListErrorKind::*;
        Self {
            kind: Builder(value),
        }
    }
}
impl From<reqwest::Error> for ExtractListError {
    fn from(value: reqwest::Error) -> Self {
        use ExtractListErrorKind::*;
        Self {
            kind: Reqwest(value),
        }
    }
}
impl Display for ExtractListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ExtractListErrorKind::*;
        match &self.kind {
            Builder(_) => "builder error".fmt(f),
            Reqwest(_) => "reqwest error".fmt(f),
            Decode(_) => "decode xml failed".fmt(f),
            NoMoreFile => "no more file".fmt(f),
        }
    }
}
impl Error for ExtractListError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use ExtractListErrorKind::*;
        match &self.kind {
            Builder(e) => Some(e),
            Reqwest(e) => Some(e),
            Decode(e) => e.get_source(),
            NoMoreFile => None,
        }
    }
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

        let query = Query::from_iter(query);

        let (bucket_url, resource) = bucket_arc.get_url_resource(&query);

        let response = self.builder(Method::GET, bucket_url, resource)?;
        let content = response.send_adjust_error()?;

        list.decode(&content.text()?, init_object_with_list_rc)?;

        list.set_client(Rc::new(self));
        list.set_search_query(query);

        Ok(list)
    }

    /// 可将 object 列表导出到外部 struct
    #[inline]
    pub fn base_object_list<
        Q: IntoIterator<Item = (QueryKey, QueryValue)>,
        List,
        Item,
        F,
        E: ListError,
        ItemErr: Error + 'static,
    >(
        &self,
        query: Q,
        list: &mut List,
        init_object: F,
    ) -> Result<(), ExtractListError>
    where
        List: RefineObjectList<Item, E, ItemErr>,
        Item: RefineObject<ItemErr>,
        F: Fn(&List) -> Item,
    {
        let bucket = BucketBase::new(self.bucket.clone(), self.get_endpoint().to_owned());

        let query = Query::from_iter(query);
        let (bucket_url, resource) = bucket.get_url_resource(&query);

        let response = self.builder(Method::GET, bucket_url, resource)?;
        let content = response.send_adjust_error()?;

        list.decode(&content.text()?, init_object)?;

        Ok(())
    }
}

#[cfg(feature = "blocking")]
impl Iterator for ObjectList<RcPointer> {
    type Item = ObjectList<RcPointer>;
    fn next(&mut self) -> Option<Self> {
        if !self.next_continuation_token.is_empty() {
            self.search_query
                .insert(CONTINUATION_TOKEN, self.next_continuation_token.to_owned());
            self.get_object_list().ok()
        } else {
            None
        }
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
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct StorageClass {
    kind: StorageClassKind,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
enum StorageClassKind {
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

impl StorageClass {
    /// Archive
    pub const ARCHIVE: Self = Self {
        kind: StorageClassKind::Archive,
    };
    /// IA
    pub const IA: Self = Self {
        kind: StorageClassKind::IA,
    };
    /// Standard
    pub const STANDARD: Self = Self {
        kind: StorageClassKind::Standard,
    };
    /// ColdArchive
    pub const COLD_ARCHIVE: Self = Self {
        kind: StorageClassKind::ColdArchive,
    };

    /// init StorageClass
    pub fn new(s: &str) -> Option<StorageClass> {
        let start_char = s.chars().next()?;

        let kind = match start_char {
            'a' | 'A' => StorageClassKind::Archive,
            'i' | 'I' => StorageClassKind::IA,
            's' | 'S' => StorageClassKind::Standard,
            'c' | 'C' => StorageClassKind::ColdArchive,
            _ => return None,
        };
        Some(Self { kind })
    }
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
