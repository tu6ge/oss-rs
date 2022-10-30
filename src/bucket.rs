use crate::auth::VERB;
#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
use crate::builder::{ArcPointer, PointerFamily};
use crate::client::Client;
#[cfg(feature = "blocking")]
use crate::client::ClientRc;
use crate::config::BucketBase;
use crate::errors::OssResult;
use crate::object::ObjectList;
use crate::traits::{
    InvalidBucketListValue, InvalidBucketValue, OssIntoBucket, OssIntoBucketList, OssIntoObjectList,
};
use crate::types::{CanonicalizedResource, Query, UrlQuery};
use chrono::prelude::*;
use std::fmt;
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Default)]
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

impl ListBuckets {
    pub fn set_client(&mut self, client: Arc<Client>) {
        self.client = Arc::clone(&client);
        for i in self.buckets.iter_mut() {
            i.set_client(Arc::clone(&client));
        }
    }
}

#[cfg(feature = "blocking")]
impl ListBuckets<RcPointer> {
    pub fn set_client(&mut self, client: Rc<ClientRc>) {
        self.client = Rc::clone(&client);
        for i in self.buckets.iter_mut() {
            i.set_client(Rc::clone(&client));
        }
    }
}

impl Default for ListBuckets {
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

#[cfg(feature = "blocking")]
impl Default for ListBuckets<RcPointer> {
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
            client: Rc::default(),
        }
    }
}

#[derive(Clone)]
#[non_exhaustive]
pub struct Bucket<PointerSel: PointerFamily = ArcPointer> {
    base: BucketBase,
    // bucket_info: Option<Bucket<'b>>,
    // bucket: Option<Bucket<'c>>,
    creation_date: DateTime<Utc>,
    //pub extranet_endpoint: String,
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

impl Default for Bucket {
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

#[cfg(feature = "blocking")]
impl Default for Bucket<RcPointer> {
    fn default() -> Self {
        Self {
            base: BucketBase::default(),
            creation_date: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc),
            //extranet_endpoint: String::default(),
            intranet_endpoint: String::default(),
            location: String::default(),
            storage_class: String::default(),
            client: Rc::default(),
        }
    }
}

impl<T: PointerFamily> OssIntoBucket for Bucket<T> {
    fn set_name(mut self, name: String) -> Result<Self, InvalidBucketValue> {
        self.base.set_name(name).map_err(|_| InvalidBucketValue)?;
        Ok(self)
    }

    fn set_creation_date(mut self, creation_date: String) -> Result<Self, InvalidBucketValue> {
        self.creation_date = creation_date
            .parse::<DateTime<Utc>>()
            .map_err(|_| InvalidBucketValue {})?;
        Ok(self)
    }

    fn set_location(mut self, location: String) -> Result<Self, InvalidBucketValue> {
        self.location = location;
        Ok(self)
    }

    fn set_extranet_endpoint(
        mut self,
        extranet_endpoint: String,
    ) -> Result<Self, InvalidBucketValue> {
        if let Err(e) = self.base.set_endpoint(extranet_endpoint) {
            return Err(InvalidBucketValue::from(e));
        }
        Ok(self)
    }

    fn set_intranet_endpoint(
        mut self,
        intranet_endpoint: String,
    ) -> Result<Self, InvalidBucketValue> {
        self.intranet_endpoint = intranet_endpoint;
        Ok(self)
    }

    fn set_storage_class(mut self, storage_class: String) -> Result<Self, InvalidBucketValue> {
        self.storage_class = storage_class;
        Ok(self)
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

impl Bucket {
    pub fn set_client(&mut self, client: Arc<Client>) {
        self.client = client;
    }

    pub fn client(&self) -> Arc<Client> {
        Arc::clone(&self.client)
    }

    pub async fn get_object_list(&self, query: Query) -> OssResult<ObjectList> {
        let mut url = self.base.to_url();

        url.set_search_query(&query);

        let canonicalized = CanonicalizedResource::from_bucket_query(&self.base, &query);

        let client = self.client();

        let response = client.builder(VERB::GET, &url, canonicalized)?;
        let content = response.send().await?;

        Ok(ObjectList::<ArcPointer>::default()
            .from_xml(content.text().await?, &self.base)?
            .set_bucket(self.base.clone())
            .set_client(client)
            .set_search_query(query))
    }
}

#[cfg(feature = "blocking")]
impl Bucket<RcPointer> {
    pub fn set_client(&mut self, client: Rc<ClientRc>) {
        self.client = client;
    }

    pub fn client(&self) -> Rc<ClientRc> {
        Rc::clone(&self.client)
    }

    pub fn get_object_list(&self, query: Query) -> OssResult<ObjectList<RcPointer>> {
        let mut url = self.base.to_url();

        url.set_search_query(&query);

        let canonicalized = CanonicalizedResource::from_bucket_query(&self.base, &query);

        let client = self.client();

        let response = client.builder(VERB::GET, &url, canonicalized)?;
        let content = response.send()?;

        Ok(ObjectList::<RcPointer>::default()
            .from_xml(content.text()?, &self.base)?
            .set_bucket(self.base.clone())
            .set_client(client)
            .set_search_query(query))
    }
}

impl<T: PointerFamily> OssIntoBucketList<Bucket<T>> for ListBuckets<T>
where
    Bucket<T>: std::default::Default,
{
    fn set_prefix(mut self, prefix: String) -> Result<Self, InvalidBucketListValue> {
        self.prefix = if prefix.len() > 0 { Some(prefix) } else { None };
        Ok(self)
    }

    fn set_marker(mut self, marker: String) -> Result<Self, InvalidBucketListValue> {
        self.marker = if marker.len() > 0 { Some(marker) } else { None };
        Ok(self)
    }

    fn set_max_keys(mut self, max_keys: String) -> Result<Self, InvalidBucketListValue> {
        self.max_keys = if max_keys.len() > 0 {
            Some(max_keys)
        } else {
            None
        };
        Ok(self)
    }

    fn set_is_truncated(mut self, is_truncated: bool) -> Result<Self, InvalidBucketListValue> {
        self.is_truncated = is_truncated;
        Ok(self)
    }

    fn set_next_marker(mut self, marker: String) -> Result<Self, InvalidBucketListValue> {
        self.next_marker = if marker.is_empty() {
            None
        } else {
            Some(marker)
        };
        Ok(self)
    }

    fn set_id(mut self, id: String) -> Result<Self, InvalidBucketListValue> {
        self.id = if id.is_empty() { None } else { Some(id) };
        Ok(self)
    }

    fn set_display_name(mut self, display_name: String) -> Result<Self, InvalidBucketListValue> {
        self.display_name = if display_name.is_empty() {
            None
        } else {
            Some(display_name)
        };
        Ok(self)
    }

    fn set_list(mut self, list: Vec<Bucket<T>>) -> Result<Self, InvalidBucketListValue> {
        self.buckets = list;
        Ok(self)
    }
}

impl Client {
    pub async fn get_bucket_list(self) -> OssResult<ListBuckets> {
        let url = self.get_endpoint_url();

        let canonicalized = CanonicalizedResource::default();

        let response = self.builder(VERB::GET, &url, canonicalized)?;
        let content = response.send().await?;

        let mut bucket_list =
            ListBuckets::<ArcPointer>::default().from_xml(content.text().await?)?;
        bucket_list.set_client(Arc::new(self));

        Ok(bucket_list)
    }

    pub async fn get_bucket_info(self) -> OssResult<Bucket> {
        let query = Some("bucketInfo");
        let mut bucket_url = self.get_bucket_url();
        bucket_url.set_query(query);

        let canonicalized = CanonicalizedResource::from_bucket(&self.get_bucket_base(), query);

        let response = self.builder(VERB::GET, &bucket_url, canonicalized)?;
        let content = response.send().await?;

        let mut bucket = Bucket::<ArcPointer>::default().from_xml(content.text().await?)?;
        bucket.set_client(Arc::new(self));

        Ok(bucket)
    }
}

#[cfg(feature = "blocking")]
impl ClientRc {
    pub fn get_bucket_list(self) -> OssResult<ListBuckets<RcPointer>> {
        let url = self.get_endpoint_url();

        let canonicalized = CanonicalizedResource::default();

        let response = self.builder(VERB::GET, &url, canonicalized)?;
        let content = response.send()?;

        let mut bucket_list = ListBuckets::<RcPointer>::default().from_xml(content.text()?)?;
        bucket_list.set_client(Rc::new(self));

        Ok(bucket_list)
    }

    pub fn get_bucket_info(self) -> OssResult<Bucket<RcPointer>> {
        let query = Some("bucketInfo");
        let mut bucket_url = self.get_bucket_url();
        bucket_url.set_query(query);

        let canonicalized = CanonicalizedResource::from_bucket(&self.get_bucket_base(), query);

        let response = self.builder(VERB::GET, &bucket_url, canonicalized)?;
        let content = response.send()?;

        let mut bucket = Bucket::<RcPointer>::default().from_xml(content.text()?)?;
        bucket.set_client(Rc::new(self));

        Ok(bucket)
    }
}

pub enum Grant {
    Private,
    PublicRead,
    PublicReadWrite,
}

impl Default for Grant {
    fn default() -> Self {
        Self::Private
    }
}

#[derive(Clone, Debug)]
pub enum DataRedundancyType {
    LRS,
    ZRS,
}

impl Default for DataRedundancyType {
    fn default() -> Self {
        Self::LRS
    }
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
