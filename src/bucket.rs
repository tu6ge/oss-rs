use crate::auth::VERB;
#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
use crate::builder::{ArcPointer, BuilderError, PointerFamily};
use crate::client::ClientArc;
#[cfg(feature = "blocking")]
use crate::client::ClientRc;
use crate::config::BucketBase;
use crate::errors::{OssError, OssResult};
#[cfg(feature = "blocking")]
use crate::file::blocking::AlignBuilder as BlockingAlignBuilder;
use crate::file::AlignBuilder;
use crate::object::{Object, ObjectList};
use crate::traits::{RefineBucket, RefineBucketList};
use crate::types::{CanonicalizedResource, InvalidEndPoint, Query};
use crate::BucketName;
use chrono::prelude::*;
use oss_derive::oss_gen_rc;
use std::error::Error;
use std::fmt;
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
#[non_exhaustive]
pub struct ListBuckets<PointerSel: PointerFamily = ArcPointer> {
    prefix: Option<String>,
    marker: Option<String>,
    max_keys: Option<String>,
    is_truncated: bool,
    next_marker: Option<String>,
    id: Option<String>,
    display_name: Option<String>,
    pub buckets: Vec<Bucket<PointerSel>>,
    client: PointerSel::PointerType,
}

impl<T: PointerFamily> fmt::Debug for ListBuckets<T> {
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
impl ListBuckets<ArcPointer> {
    pub(crate) fn set_client(&mut self, client: Arc<ClientArc>) {
        self.client = Arc::clone(&client);
    }
}

#[oss_gen_rc]
impl Default for ListBuckets<ArcPointer> {
    fn default() -> Self {
        Self {
            prefix: None,
            marker: None,
            max_keys: None,
            is_truncated: false,
            next_marker: None,
            id: None,
            display_name: None,
            buckets: Vec::default(),
            client: Arc::default(),
        }
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub struct Bucket<PointerSel: PointerFamily = ArcPointer> {
    pub(crate) base: BucketBase,
    // bucket_info: Option<Bucket<'b>>,
    // bucket: Option<Bucket<'c>>,
    creation_date: DateTime<Utc>,
    //pub extranet_endpoint: String,
    #[deprecated(since = "0.10", note = "base field has intranet endpoint info")]
    intranet_endpoint: String,
    location: String,
    // owner 	存放Bucket拥有者信息的容器。父节点：BucketInfo.Bucket
    // access_control_list;
    // pub grant: Grant,
    // pub data_redundancy_type: Option<DataRedundancyType>,
    storage_class: String,
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
            .field("intranet_endpoint", &self.intranet_endpoint)
            .field("location", &self.location)
            .field("storage_class", &self.storage_class)
            .finish()
    }
}

#[oss_gen_rc]
impl Default for Bucket<ArcPointer> {
    fn default() -> Self {
        Self {
            base: BucketBase::default(),
            creation_date: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
            //extranet_endpoint: String::default(),
            intranet_endpoint: String::default(),
            location: String::default(),
            storage_class: String::default(),
            client: Arc::default(),
        }
    }
}

impl<T: PointerFamily> RefineBucket for Bucket<T> {
    type Error = OssError;
    fn set_name(&mut self, name: &str) -> Result<(), Self::Error> {
        self.base.set_name(name);
        Ok(())
    }

    fn set_creation_date(&mut self, creation_date: &str) -> Result<(), Self::Error> {
        self.creation_date = creation_date
            .parse::<DateTime<Utc>>()
            .map_err(|_| InvalidBucketValue)?;
        Ok(())
    }

    fn set_location(&mut self, location: &str) -> Result<(), Self::Error> {
        self.location = location.to_string();
        Ok(())
    }

    fn set_extranet_endpoint(&mut self, extranet_endpoint: &str) -> Result<(), Self::Error> {
        self.base.set_endpoint(extranet_endpoint);
        Ok(())
    }

    fn set_storage_class(&mut self, storage_class: &str) -> Result<(), Self::Error> {
        self.storage_class = storage_class.to_string();
        Ok(())
    }
}

#[derive(Debug)]
pub struct InvalidBucketValue;

impl fmt::Display for InvalidBucketValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "faild parse to bucket value")
    }
}

impl Error for InvalidBucketValue {}

impl From<InvalidEndPoint> for InvalidBucketValue {
    //TODO 待完善
    fn from(_end: InvalidEndPoint) -> InvalidBucketValue {
        InvalidBucketValue {}
    }
}

impl<T: PointerFamily> Bucket<T> {
    pub fn new(
        base: BucketBase,
        creation_date: DateTime<Utc>,
        intranet_endpoint: String,
        location: String,
        storage_class: String,
        client: T::PointerType,
    ) -> Self {
        Self {
            base,
            creation_date,
            intranet_endpoint,
            location,
            storage_class,
            client,
        }
    }
}

#[oss_gen_rc]
impl Bucket<ArcPointer> {
    pub fn set_client(&mut self, client: Arc<ClientArc>) {
        self.client = client;
    }

    pub fn client(&self) -> Arc<ClientArc> {
        Arc::clone(&self.client)
    }
}

impl Bucket {
    /// # 查询 Object 列表
    ///
    /// 参数 query 有多种写法：
    /// - `[]` 查所有
    /// - `[("max-keys", "5")]` 数组（不可变长度），最大可支持 size 为 8 的数组
    /// - `[("max-keys", "5"), ("prefix", "babel")]` 数组（不可变长度）
    /// - `vec![("max-keys", "5")]` Vec(可变长度)
    /// - `vec![("max-keys", 5u8)]` 数字类型
    /// - `vec![("max-keys", 1000u16)]` u16 数字类型
    pub async fn get_object_list<Q: Into<Query>>(&self, query: Q) -> OssResult<ObjectList> {
        let query = query.into();

        let base = self.base.clone();

        let bucket_arc = Arc::new(base.clone());

        let init_object = || {
            let mut object = Object::<ArcPointer>::default();
            object.base.set_bucket(bucket_arc.clone());
            object
        };

        let client = self.client();

        let mut list = ObjectList::<ArcPointer>::default();

        let result: Result<_, OssError> = client
            .base_object_list(
                self.base.get_name().to_owned(),
                query.clone(),
                &mut list,
                init_object,
            )
            .await;
        result?;

        list.set_bucket(base);
        list.set_client(Arc::clone(&client));
        list.set_search_query(query);

        Ok(list)
    }
}

#[cfg(feature = "blocking")]
impl Bucket<RcPointer> {
    /// 查询默认 bucket 的文件列表
    ///
    /// 查询条件参数有多种方式，具体参考 [`get_object_list`](#method.get_object_list) 文档
    pub fn get_object_list<Q: Into<Query>>(&self, query: Q) -> OssResult<ObjectList<RcPointer>> {
        let query = query.into();

        let base = self.base.clone();

        let bucket_arc = Rc::new(base.clone());

        let init_object = || {
            let mut object = Object::<RcPointer>::default();
            object.base.set_bucket(bucket_arc.clone());
            object
        };

        let client = self.client();

        let mut list = ObjectList::<RcPointer>::default();

        let result: Result<_, OssError> = client.base_object_list(
            self.base.get_name().to_owned(),
            query.clone(),
            &mut list,
            init_object,
        );
        result?;

        list.set_bucket(base);
        list.set_client(Rc::clone(&client));
        list.set_search_query(query);

        Ok(list)
    }
}

impl<T: PointerFamily> RefineBucketList<Bucket<T>> for ListBuckets<T> {
    type Error = OssError;

    fn set_prefix(&mut self, prefix: &str) -> Result<(), Self::Error> {
        self.prefix = if prefix.len() > 0 {
            Some(prefix.to_owned())
        } else {
            None
        };
        Ok(())
    }

    fn set_marker(&mut self, marker: &str) -> Result<(), Self::Error> {
        self.marker = if marker.len() > 0 {
            Some(marker.to_owned())
        } else {
            None
        };
        Ok(())
    }

    fn set_max_keys(&mut self, max_keys: &str) -> Result<(), Self::Error> {
        self.max_keys = if max_keys.len() > 0 {
            Some(max_keys.to_owned())
        } else {
            None
        };
        Ok(())
    }

    fn set_is_truncated(&mut self, is_truncated: bool) -> Result<(), Self::Error> {
        self.is_truncated = is_truncated;
        Ok(())
    }

    fn set_next_marker(&mut self, marker: &str) -> Result<(), Self::Error> {
        self.next_marker = if marker.is_empty() {
            None
        } else {
            Some(marker.to_owned())
        };
        Ok(())
    }

    fn set_id(&mut self, id: &str) -> Result<(), Self::Error> {
        self.id = if id.is_empty() {
            None
        } else {
            Some(id.to_owned())
        };
        Ok(())
    }

    fn set_display_name(&mut self, display_name: &str) -> Result<(), Self::Error> {
        self.display_name = if display_name.is_empty() {
            None
        } else {
            Some(display_name.to_owned())
        };
        Ok(())
    }

    fn set_list(&mut self, list: Vec<Bucket<T>>) -> Result<(), Self::Error> {
        self.buckets = list;
        Ok(())
    }
}

impl ClientArc {
    pub async fn get_bucket_list(self) -> OssResult<ListBuckets> {
        let client_arc = Arc::new(self.clone());

        let init_bucket = || {
            let mut bucket = Bucket::<ArcPointer>::default();
            bucket.set_client(client_arc.clone());
            bucket
        };

        let mut bucket_list = ListBuckets::<ArcPointer>::default();
        let res: Result<_, OssError> = self.base_bucket_list(&mut bucket_list, init_bucket).await;
        res?;

        bucket_list.set_client(client_arc.clone());

        Ok(bucket_list)
    }

    #[inline]
    pub async fn base_bucket_list<List, Item, F, E>(
        &self,
        list: &mut List,
        init_bucket: F,
    ) -> Result<(), E>
    where
        List: RefineBucketList<Item>,
        Item: RefineBucket,
        E: From<BuilderError> + From<List::Error>,
        F: FnMut() -> Item,
    {
        let url = self.get_endpoint_url();

        let canonicalized = CanonicalizedResource::default();

        let response = self.builder(VERB::GET, url, canonicalized)?;
        let content = response.send().await?;

        list.from_xml(
            &content.text().await.map_err(BuilderError::from)?,
            init_bucket,
        )?;

        Ok(())
    }

    pub async fn get_bucket_info(self) -> OssResult<Bucket> {
        let name = self.get_bucket_name();

        let mut bucket = Bucket::<ArcPointer>::default();

        let res: Result<_, OssError> = self.base_bucket_info(name.to_owned(), &mut bucket).await;
        res?;

        bucket.set_client(Arc::new(self));

        Ok(bucket)
    }

    #[inline]
    pub async fn base_bucket_info<Bucket, Name: Into<BucketName>, E>(
        &self,
        name: Name,
        bucket: &mut Bucket,
    ) -> Result<(), E>
    where
        Bucket: RefineBucket,
        E: From<BuilderError> + From<Bucket::Error>,
    {
        let mut bucket_url = BucketBase::new(name.into(), self.get_endpoint().to_owned()).to_url();
        let query = Some("bucketInfo");
        bucket_url.set_query(query);

        let canonicalized = CanonicalizedResource::from_bucket(&self.get_bucket_base(), query);

        let response = self.builder(VERB::GET, bucket_url, canonicalized)?;
        let content = response.send().await?;

        bucket.from_xml(&content.text().await.map_err(BuilderError::from)?)?;

        Ok(())
    }
}

#[cfg(feature = "blocking")]
impl ClientRc {
    pub fn get_bucket_list(self) -> OssResult<ListBuckets<RcPointer>> {
        let client_arc = Rc::new(self.clone());

        let init_bucket = || {
            let mut bucket = Bucket::<RcPointer>::default();
            bucket.set_client(client_arc.clone());
            bucket
        };

        let mut bucket_list = ListBuckets::<RcPointer>::default();
        let res: Result<_, OssError> = self.base_bucket_list(&mut bucket_list, init_bucket);
        res?;
        bucket_list.set_client(Rc::new(self));

        Ok(bucket_list)
    }

    #[inline]
    pub fn base_bucket_list<List, Item, F, E>(
        &self,
        list: &mut List,
        init_bucket: F,
    ) -> Result<(), E>
    where
        List: RefineBucketList<Item>,
        Item: RefineBucket,
        E: From<BuilderError> + From<List::Error>,
        F: FnMut() -> Item,
    {
        let url = self.get_endpoint_url();

        let canonicalized = CanonicalizedResource::default();

        let response = self.builder(VERB::GET, url, canonicalized)?;
        let content = response.send()?;

        list.from_xml(&content.text().map_err(BuilderError::from)?, init_bucket)?;

        Ok(())
    }

    pub fn get_bucket_info(self) -> OssResult<Bucket<RcPointer>> {
        let name = self.get_bucket_name();

        let mut bucket = Bucket::<RcPointer>::default();

        let res: Result<_, OssError> = self.base_bucket_info(name.to_owned(), &mut bucket);
        res?;

        bucket.set_client(Rc::new(self));

        Ok(bucket)
    }

    #[inline]
    pub fn base_bucket_info<Bucket, Name: Into<BucketName>, E>(
        &self,
        name: Name,
        bucket: &mut Bucket,
    ) -> Result<(), E>
    where
        Bucket: RefineBucket,
        E: From<BuilderError> + From<Bucket::Error>,
    {
        let mut bucket_url = BucketBase::new(name.into(), self.get_endpoint().to_owned()).to_url();
        let query = Some("bucketInfo");
        bucket_url.set_query(query);

        let canonicalized = CanonicalizedResource::from_bucket(&self.get_bucket_base(), query);

        let response = self.builder(VERB::GET, bucket_url, canonicalized)?;
        let content = response.send()?;

        bucket.from_xml(&content.text().map_err(BuilderError::from)?)?;

        Ok(())
    }
}

impl<T: PointerFamily> PartialEq<Bucket<T>> for Bucket<T> {
    #[inline]
    fn eq(&self, other: &Bucket<T>) -> bool {
        self.base == other.base
            && self.creation_date == other.creation_date
            && self.location == other.location
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

#[derive(Default)]
pub enum Grant {
    #[default]
    Private,
    PublicRead,
    PublicReadWrite,
}

#[derive(Clone, Debug, Default)]
pub enum DataRedundancyType {
    #[default]
    LRS,
    ZRS,
}

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
    pub last_modified: &'a str, // TODO 时间
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

#[derive(Clone, Debug)]
pub enum Location {
    CnHangzhou,
    CnShanghai,
    CnQingdao,
    CnBeijing,
    CnZhangjiakou, // 张家口 lenght=13
    CnHongkong,
    CnShenzhen,
    UsWest1,
    UsEast1,
    ApSouthEast1,
}

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
