#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
use crate::builder::{ArcPointer, BuilderError, PointerFamily};
use crate::client::ClientArc;
#[cfg(feature = "blocking")]
use crate::client::ClientRc;
use crate::config::BucketBase;
use crate::decode::{
    InnerItemError, ItemError, ListError, RefineBucket, RefineBucketList, RefineObjectList,
};
use crate::errors::OssError;
#[cfg(feature = "blocking")]
use crate::file::blocking::AlignBuilder as BlockingAlignBuilder;
use crate::file::AlignBuilder;
use crate::object::{ExtractListError, Object, ObjectList, StorageClass};
use crate::types::{CanonicalizedResource, Query, QueryKey, QueryValue, BUCKET_INFO};
use crate::{BucketName, EndPoint};

use chrono::{DateTime, NaiveDateTime, Utc};
use http::Method;
use oss_derive::oss_gen_rc;
use std::fmt;
use std::marker::PhantomData;
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::sync::Arc;

/// # 存储 Bucket 列表的 struct
#[derive(Clone)]
#[non_exhaustive]
pub struct ListBuckets<
    PointerSel: PointerFamily = ArcPointer,
    Item: RefineBucket<E> = Bucket<PointerSel>,
    E: ItemError = OssError,
> {
    prefix: String,
    marker: String,
    max_keys: u16,
    is_truncated: bool,
    next_marker: String,
    id: String,
    display_name: String,
    /// 存放单个 bucket 类型的 vec 集合
    pub buckets: Vec<Item>,
    client: PointerSel::PointerType,
    ph_err: PhantomData<E>,
}

impl<T: PointerFamily, Item: RefineBucket<E> + std::fmt::Debug, E: ItemError> fmt::Debug
    for ListBuckets<T, Item, E>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ListBuckets")
            .field("prefix", &self.prefix)
            .field("marker", &self.marker)
            .field("max_keys", &self.max_keys)
            .field("is_truncated", &self.is_truncated)
            .field("next_marker", &self.next_marker)
            .field("id", &self.id)
            .field("display_name", &self.display_name)
            .field("buckets", &self.buckets)
            .finish()
    }
}

#[oss_gen_rc]
impl<Item: RefineBucket<E>, E: ItemError> ListBuckets<ArcPointer, Item, E> {
    pub(crate) fn set_client(&mut self, client: Arc<ClientArc>) {
        self.client = Arc::clone(&client);
    }
}

#[oss_gen_rc]
impl<Item: RefineBucket<E>, E: ItemError> Default for ListBuckets<ArcPointer, Item, E> {
    fn default() -> Self {
        Self {
            prefix: String::default(),
            marker: String::default(),
            max_keys: 0,
            is_truncated: false,
            next_marker: String::default(),
            id: String::default(),
            display_name: String::default(),
            buckets: Vec::default(),
            client: Arc::default(),
            ph_err: PhantomData,
        }
    }
}

impl<Item: RefineBucket<E>, E: ItemError> ListBuckets<ArcPointer, Item, E> {
    /// 获取 prefix
    pub fn prefix_string(&self) -> &String {
        &self.prefix
    }

    /// 获取 marker
    pub fn marker_string(&self) -> &String {
        &self.marker
    }

    /// 获取 next_marker
    pub fn next_marker_string(&self) -> &String {
        &self.next_marker
    }

    /// 获取 id 和 display_name
    pub fn info_string(&self) -> (&String, &String) {
        (&self.id, &self.display_name)
    }
}

/// 内置的存放单个 bucket 的类型
#[derive(Clone)]
#[non_exhaustive]
pub struct Bucket<PointerSel: PointerFamily = ArcPointer> {
    pub(crate) base: BucketBase,
    // bucket_info: Option<Bucket<'b>>,
    // bucket: Option<Bucket<'c>>,
    creation_date: DateTime<Utc>,
    //pub extranet_endpoint: String,
    // owner 	存放Bucket拥有者信息的容器。父节点：BucketInfo.Bucket
    // access_control_list;
    // pub grant: Grant,
    // pub data_redundancy_type: Option<DataRedundancyType>,
    storage_class: StorageClass,
    // pub versioning: &'a str,
    // ServerSideEncryptionRule,
    // ApplyServerSideEncryptionByDefault,
    // pub sse_algorithm: &'a str,
    // pub kms_master_key_id: Option<&'a str>,
    // pub cross_region_replication: &'a str,
    // pub transfer_acceleration: &'a str,
    client: PointerSel::PointerType,
}

impl<T: PointerFamily> fmt::Debug for Bucket<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bucket")
            .field("base", &self.base)
            .field("creation_date", &self.creation_date)
            //.field("extranet_endpoint", &self.extranet_endpoint)
            .field("storage_class", &self.storage_class)
            .finish()
    }
}

#[oss_gen_rc]
impl Default for Bucket<ArcPointer> {
    fn default() -> Self {
        Self {
            base: BucketBase::default(),
            creation_date: DateTime::<Utc>::from_utc(
                #[allow(clippy::unwrap_used)]
                NaiveDateTime::from_timestamp_opt(61, 0).unwrap(),
                Utc,
            ),
            //extranet_endpoint: String::default(),
            storage_class: StorageClass::default(),
            client: Arc::default(),
        }
    }
}

impl<T: PointerFamily> AsRef<BucketBase> for Bucket<T> {
    fn as_ref(&self) -> &BucketBase {
        &self.base
    }
}

impl<T: PointerFamily> AsRef<BucketName> for Bucket<T> {
    fn as_ref(&self) -> &BucketName {
        self.base.as_ref()
    }
}

impl<T: PointerFamily> AsRef<EndPoint> for Bucket<T> {
    fn as_ref(&self) -> &EndPoint {
        self.base.as_ref()
    }
}

impl<T: PointerFamily> RefineBucket<OssError> for Bucket<T> {
    fn set_name(&mut self, name: &str) -> Result<(), OssError> {
        self.base.set_name(name.parse::<BucketName>()?);
        Ok(())
    }

    fn set_location(&mut self, location: &str) -> Result<(), OssError> {
        self.base.set_endpoint(location.parse::<EndPoint>()?);
        Ok(())
    }

    fn set_creation_date(&mut self, creation_date: &str) -> Result<(), OssError> {
        self.creation_date = creation_date.parse()?;
        Ok(())
    }

    fn set_storage_class(&mut self, storage_class: &str) -> Result<(), OssError> {
        let start_char = storage_class
            .chars()
            .next()
            .ok_or(OssError::InvalidStorageClass)?;

        match start_char {
            'a' | 'A' => self.storage_class = StorageClass::Archive,
            'i' | 'I' => self.storage_class = StorageClass::IA,
            's' | 'S' => self.storage_class = StorageClass::Standard,
            'c' | 'C' => self.storage_class = StorageClass::ColdArchive,
            _ => return Err(OssError::InvalidStorageClass),
        }

        Ok(())
    }
}

impl<T: PointerFamily> Bucket<T> {
    /// 初始化 Bucket
    pub fn new(
        base: BucketBase,
        creation_date: DateTime<Utc>,
        storage_class: StorageClass,
        client: T::PointerType,
    ) -> Self {
        Self {
            base,
            creation_date,
            storage_class,
            client,
        }
    }

    /// 获取 bucket 创建时间
    pub fn creation_date(&self) -> &DateTime<Utc> {
        &self.creation_date
    }

    /// 获取 storage_class
    pub fn storage_class(&self) -> &StorageClass {
        &self.storage_class
    }

    /// 读取 bucket 基本信息
    pub fn base(&self) -> &BucketBase {
        &self.base
    }
}

#[oss_gen_rc]
impl Bucket<ArcPointer> {
    /// 为 Bucket struct 设置 Client
    fn set_client(&mut self, client: Arc<ClientArc>) {
        self.client = client;
    }

    /// 获取 Bucket 的 Client 信息
    pub(crate) fn client(&self) -> Arc<ClientArc> {
        Arc::clone(&self.client)
    }
}

impl Bucket {
    /// # 查询 Object 列表
    ///
    /// 参数 query 有多种写法：
    /// - `[]` 查所有
    /// - `[("max-keys".into(), "5".into())]` 数组（不可变长度），最大可支持 size 为 8 的数组
    /// - `[("max-keys".into(), "5".into()), ("prefix".into(), "babel".into())]` 数组（不可变长度）
    /// - `vec![("max-keys".into(), "5".into())]` Vec(可变长度)
    /// - `vec![("max-keys".into(), 5u8.into())]` 数字类型
    /// - `vec![("max-keys".into(), 1000u16.into())]` u16 数字类型
    pub async fn get_object_list<Q: IntoIterator<Item = (QueryKey, QueryValue)>>(
        &self,
        query: Q,
    ) -> Result<ObjectList, ExtractListError> {
        let query = Query::from_iter(query);

        let bucket_arc = Arc::new(self.base.clone());

        let init_object = || {
            let mut object = Object::<ArcPointer>::default();
            object.base.set_bucket(bucket_arc.clone());
            object
        };
        let mut list = ObjectList::<ArcPointer>::default();

        let (bucket_url, resource) = bucket_arc.get_url_resource(&query);

        let response = self.builder(Method::GET, bucket_url, resource)?;
        let content = response.send_adjust_error().await?;

        list.decode(
            &content.text().await.map_err(BuilderError::from)?,
            init_object,
        )?;

        list.set_bucket(self.base.clone());
        list.set_client(self.client());
        list.set_search_query(query);

        Ok(list)
    }
}

#[cfg(feature = "blocking")]
impl Bucket<RcPointer> {
    /// 查询默认 bucket 的文件列表
    ///
    /// 查询条件参数有多种方式，具体参考 [`get_object_list`](#method.get_object_list) 文档
    pub fn get_object_list<Q: IntoIterator<Item = (QueryKey, QueryValue)>>(
        &self,
        query: Q,
    ) -> Result<ObjectList<RcPointer>, ExtractListError> {
        let query = Query::from_iter(query);

        let bucket_arc = Rc::new(self.base.clone());

        let init_object = || {
            let mut object = Object::<RcPointer>::default();
            object.base.set_bucket(bucket_arc.clone());
            object
        };

        let mut list = ObjectList::<RcPointer>::default();

        let (bucket_url, resource) = bucket_arc.get_url_resource(&query);

        let response = self.builder(Method::GET, bucket_url, resource)?;
        let content = response.send_adjust_error()?;

        list.decode(&content.text().map_err(BuilderError::from)?, init_object)?;

        list.set_bucket(self.base.clone());
        list.set_client(self.client());
        list.set_search_query(query);

        Ok(list)
    }
}

impl<T: PointerFamily, Item: RefineBucket<E>, E: ItemError> RefineBucketList<Item, OssError, E>
    for ListBuckets<T, Item, E>
{
    fn set_prefix(&mut self, prefix: &str) -> Result<(), OssError> {
        self.prefix = prefix.to_owned();
        Ok(())
    }

    fn set_marker(&mut self, marker: &str) -> Result<(), OssError> {
        self.marker = marker.to_owned();
        Ok(())
    }

    fn set_max_keys(&mut self, max_keys: &str) -> Result<(), OssError> {
        self.max_keys = max_keys.parse()?;
        Ok(())
    }

    fn set_is_truncated(&mut self, is_truncated: bool) -> Result<(), OssError> {
        self.is_truncated = is_truncated;
        Ok(())
    }

    fn set_next_marker(&mut self, marker: &str) -> Result<(), OssError> {
        self.next_marker = marker.to_owned();
        Ok(())
    }

    fn set_id(&mut self, id: &str) -> Result<(), OssError> {
        self.id = id.to_owned();
        Ok(())
    }

    fn set_display_name(&mut self, display_name: &str) -> Result<(), OssError> {
        self.display_name = display_name.to_owned();
        Ok(())
    }

    fn set_list(&mut self, list: Vec<Item>) -> Result<(), OssError> {
        self.buckets = list;
        Ok(())
    }
}

impl ClientArc {
    /// 从 OSS 获取 bucket 列表
    pub async fn get_bucket_list(self) -> Result<ListBuckets, ExtractListError> {
        let client_arc = Arc::new(self);

        let init_bucket = || {
            let mut bucket = Bucket::<ArcPointer>::default();
            bucket.set_client(client_arc.clone());
            bucket
        };

        let mut bucket_list = ListBuckets::<ArcPointer>::default();
        client_arc
            .base_bucket_list(&mut bucket_list, init_bucket)
            .await?;

        bucket_list.set_client(client_arc.clone());

        Ok(bucket_list)
    }

    /// 从 OSS 获取 bucket 列表，并存入自定义类型中
    pub async fn base_bucket_list<List, Item, F, E, ItemErr>(
        &self,
        list: &mut List,
        init_bucket: F,
    ) -> Result<(), ExtractListError>
    where
        List: RefineBucketList<Item, E, ItemErr>,
        Item: RefineBucket<ItemErr>,
        E: ListError,
        ItemErr: ItemError,
        F: FnMut() -> Item,
    {
        let url = self.get_endpoint_url();

        let canonicalized = CanonicalizedResource::default();

        let response = self.builder(Method::GET, url, canonicalized)?;
        let content = response.send_adjust_error().await?;

        list.decode(
            &content
                .text()
                .await
                .map_err(BuilderError::from)
                .map_err(ExtractListError::from)?,
            init_bucket,
        )
        .map_err(ExtractListError::from)?;

        Ok(())
    }

    /// 从 OSS 上获取默认的 bucket 信息
    pub async fn get_bucket_info(self) -> Result<Bucket, ExtractItemError> {
        let name = self.get_bucket_name();

        let mut bucket = Bucket::<ArcPointer>::default();

        self.base_bucket_info(name.to_owned(), &mut bucket).await?;

        bucket.set_client(Arc::new(self));

        Ok(bucket)
    }

    /// 从 OSS 上获取某一个 bucket 的信息，并存入自定义的类型中
    pub async fn base_bucket_info<Bucket, Name: Into<BucketName>, E>(
        &self,
        name: Name,
        bucket: &mut Bucket,
    ) -> Result<(), ExtractItemError>
    where
        Bucket: RefineBucket<E>,
        E: ItemError,
    {
        let mut bucket_url = BucketBase::new(name.into(), self.get_endpoint().to_owned()).to_url();
        let query = Some(BUCKET_INFO);
        bucket_url.set_query(query);

        let canonicalized = CanonicalizedResource::from_bucket(&self.get_bucket_base(), query);

        let response = self.builder(Method::GET, bucket_url, canonicalized)?;
        let content = response.send_adjust_error().await?;

        bucket
            .decode(&content.text().await.map_err(BuilderError::from)?)
            .map_err(ExtractItemError::from)?;

        Ok(())
    }
}

/// 为 [`base_bucket_info`] 方法，返回一个统一的 Error
///
/// [`base_bucket_info`]: crate::client::Client::base_bucket_info
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ExtractItemError {
    #[doc(hidden)]
    #[error("{0}")]
    Builder(#[from] BuilderError),

    #[doc(hidden)]
    #[error("{0}")]
    Item(#[from] InnerItemError),
}

#[cfg(feature = "blocking")]
impl ClientRc {
    /// 获取 bucket 列表
    pub fn get_bucket_list(self) -> Result<ListBuckets<RcPointer>, ExtractListError> {
        let client_arc = Rc::new(self);

        let init_bucket = || {
            let mut bucket = Bucket::<RcPointer>::default();
            bucket.set_client(client_arc.clone());
            bucket
        };

        let mut bucket_list = ListBuckets::<RcPointer>::default();
        client_arc.base_bucket_list(&mut bucket_list, init_bucket)?;
        bucket_list.set_client(client_arc.clone());

        Ok(bucket_list)
    }

    /// 获取 bucket 列表，可存储为自定义的类型
    #[inline]
    pub fn base_bucket_list<List, Item, F, E, ItemErr>(
        &self,
        list: &mut List,
        init_bucket: F,
    ) -> Result<(), ExtractListError>
    where
        List: RefineBucketList<Item, E, ItemErr>,
        Item: RefineBucket<ItemErr>,
        E: ListError,
        ItemErr: ItemError,
        F: FnMut() -> Item,
    {
        let url = self.get_endpoint_url();

        let canonicalized = CanonicalizedResource::default();

        let response = self.builder(Method::GET, url, canonicalized)?;
        let content = response.send_adjust_error()?;

        list.decode(&content.text().map_err(BuilderError::from)?, init_bucket)
            .map_err(ExtractListError::from)?;

        Ok(())
    }

    /// 获取当前的 bucket 的信息
    pub fn get_bucket_info(self) -> Result<Bucket<RcPointer>, ExtractItemError> {
        let name = self.get_bucket_name();

        let mut bucket = Bucket::<RcPointer>::default();

        self.base_bucket_info(name.to_owned(), &mut bucket)?;

        bucket.set_client(Rc::new(self));

        Ok(bucket)
    }

    /// 获取某一个 bucket 的信息，并存储到自定义的类型
    #[inline]
    pub fn base_bucket_info<Bucket, Name: Into<BucketName>, E>(
        &self,
        name: Name,
        bucket: &mut Bucket,
    ) -> Result<(), ExtractItemError>
    where
        Bucket: RefineBucket<E>,
        E: ItemError,
    {
        let mut bucket_url = BucketBase::new(name.into(), self.get_endpoint().to_owned()).to_url();
        let query = Some(BUCKET_INFO);
        bucket_url.set_query(query);

        let canonicalized = CanonicalizedResource::from_bucket(&self.get_bucket_base(), query);

        let response = self.builder(Method::GET, bucket_url, canonicalized)?;
        let content = response.send_adjust_error()?;

        bucket
            .decode(&content.text().map_err(BuilderError::from)?)
            .map_err(ExtractItemError::from)?;

        Ok(())
    }
}

impl<T: PointerFamily> PartialEq<Bucket<T>> for Bucket<T> {
    #[inline]
    fn eq(&self, other: &Bucket<T>) -> bool {
        self.base == other.base
            && self.creation_date == other.creation_date
            && self.storage_class == other.storage_class
    }
}

impl<T: PointerFamily> PartialEq<DateTime<Utc>> for Bucket<T> {
    #[inline]
    fn eq(&self, other: &DateTime<Utc>) -> bool {
        &self.creation_date == other
    }
}

impl<T: PointerFamily> PartialEq<BucketBase> for Bucket<T> {
    #[inline]
    fn eq(&self, other: &BucketBase) -> bool {
        &self.base == other
    }
}

#[doc(hidden)]
#[derive(Default)]
pub enum Grant {
    #[default]
    Private,
    PublicRead,
    PublicReadWrite,
}

#[doc(hidden)]
#[derive(Clone, Debug, Default)]
pub enum DataRedundancyType {
    #[default]
    LRS,
    ZRS,
}

#[doc(hidden)]
#[derive(Default, Clone, Debug)]
pub struct BucketListObjectParms<'a> {
    pub list_type: u8,
    pub delimiter: &'a str,
    pub continuation_token: &'a str,
    pub max_keys: u32,
    pub prefix: &'a str,
    pub encoding_type: &'a str,
    pub fetch_owner: bool,
}

#[doc(hidden)]
#[derive(Default, Clone, Debug)]
pub struct BucketListObject<'a> {
    //pub content:
    pub common_prefixes: &'a str,
    pub delimiter: &'a str,
    pub encoding_type: &'a str,
    pub display_name: &'a str,
    pub etag: &'a str,
    pub id: &'a str,
    pub is_truncated: bool,
    pub key: &'a str,
    pub last_modified: &'a str,
    pub list_bucket_result: Option<&'a str>,
    pub start_after: Option<&'a str>,
    pub max_keys: u32,
    pub name: &'a str,
    // pub owner: &'a str,
    pub prefix: &'a str,
    pub size: u64,
    pub storage_class: &'a str,
    pub continuation_token: Option<&'a str>,
    pub key_count: i32,
    pub next_continuation_token: Option<&'a str>,
    pub restore_info: Option<&'a str>,
}

#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct BucketStat {
    pub storage: u64,
    pub object_count: u32,
    pub multipart_upload_count: u32,
    pub live_channel_count: u32,
    pub last_modified_time: u16,
    pub standard_storage: u64,
    pub standard_object_count: u32,
    pub infrequent_access_storage: u64,
    pub infrequent_access_real_storage: u64,
    pub infrequent_access_object_count: u64,
    pub archive_storage: u64,
    pub archive_real_storage: u64,
    pub archive_object_count: u64,
    pub cold_archive_storage: u64,
    pub cold_archive_real_storage: u64,
    pub cold_archive_object_count: u64,
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "blocking")]
    #[test]
    fn test_default_list_bucket() {
        use crate::builder::RcPointer;

        use super::ListBuckets;

        let list = ListBuckets::<RcPointer>::default();

        assert!(list.buckets.len() == 0);
    }
}
