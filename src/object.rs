use crate::auth::VERB;
#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
use crate::builder::{ArcPointer, PointerFamily};
use crate::client::Client;
#[cfg(feature = "blocking")]
use crate::client::ClientRc;
use crate::config::{BucketBase, ObjectBase, ObjectPath};
use crate::errors::{OssError, OssResult};
#[cfg(feature = "blocking")]
use crate::file::blocking::AlignBuilder as BlockingAlignBuilder;
use crate::file::AlignBuilder;
use crate::traits::{InvalidObjectListValue, InvalidObjectValue, OssIntoObject, OssIntoObjectList};
use crate::types::{CanonicalizedResource, Query, UrlQuery};
use async_stream::try_stream;
use chrono::prelude::*;
use futures_core::stream::Stream;
use std::fmt;
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::sync::Arc;
use std::vec::IntoIter;

/// # 存放对象列表的结构体
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

impl Default for ObjectList {
    fn default() -> Self {
        Self {
            bucket: BucketBase::default(),
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
            prefix,
            max_keys,
            key_count,
            object_list,
            next_continuation_token,
            client,
            search_query,
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
                search_query.insert("continuation-token", token.to_owned());
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

impl ObjectList {
    pub fn set_client(mut self, client: Arc<Client>) -> Self {
        self.client = client;
        self
    }

    pub fn client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }

    pub async fn get_next_list(&self) -> OssResult<Self> {
        match self.next_query() {
            None => Err(OssError::WithoutMore),
            Some(query) => {
                let mut url = self.bucket.to_url();
                url.set_search_query(&query);

                let canonicalized = CanonicalizedResource::from_bucket_query(&self.bucket, &query);

                let response = self.builder(VERB::GET, url, canonicalized)?;
                let content = response.send().await?;

                let list = ObjectList::<ArcPointer>::default()
                    .set_client(self.client().clone())
                    .set_bucket(self.bucket.clone());

                Ok(list
                    .from_xml(content.text().await?, Arc::new(self.bucket.clone()))?
                    .set_search_query(query))
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
    /// ```ignore
    /// # use dotenv::dotenv;
    /// # use aliyun_oss_client::{Client, Query};
    /// # #[tokio::main]
    /// # async fn main() {
    ///     # dotenv().ok();
    ///     use futures::{pin_mut, StreamExt};
    ///     # let client = Client::from_env().unwrap();
    ///     # let mut query = Query::new();
    ///     # query.insert("max-keys", "100");
    ///     # let object_list = client.get_object_list(query).await.unwrap();
    ///     let stream = object_list.into_stream();
    ///     pin_mut!(stream);
    ///
    ///     let second_list = stream.next().await;
    ///     let third_list = stream.next().await;
    ///     println!("second_list: {:?}", second_list);
    ///     println!("third_list: {:?}", third_list);
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
        key: P,
        last_modified: DateTime<Utc>,
        etag: String,
        _type: String,
        size: u64,
        storage_class: String,
    ) -> Self {
        let base = ObjectBase::<T>::new(bucket, key);
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

impl<T: PointerFamily + Sized> OssIntoObject<T> for Object<T> {
    fn set_bucket(mut self, bucket: T::Bucket) -> Self {
        self.base.set_bucket(bucket);
        self
    }

    fn set_key(mut self, key: String) -> Result<Self, InvalidObjectValue> {
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
    use std::sync::Arc;

    use chrono::{DateTime, NaiveDateTime, Utc};

    use crate::{
        builder::ArcPointer,
        config::BucketBase,
        object::{Object, ObjectBuilder},
        types::QueryValue,
        Client, Query,
    };

    use super::ObjectList;

    fn init_object_list(token: Option<String>, list: Vec<Object>) -> ObjectList {
        let client = Client::new(
            "foo1".into(),
            "foo2".into(),
            "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
            "foo4".try_into().unwrap(),
        );

        let mut query = Query::new();
        query.insert("key1", "value1");

        let object_list = ObjectList::<ArcPointer>::new(
            BucketBase::from_str("abc.oss-cn-shanghai.aliyuncs.com").unwrap(),
            String::from("foo2"),
            100,
            200,
            list,
            token,
            Arc::new(client),
            query,
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
