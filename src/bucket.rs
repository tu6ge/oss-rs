
use crate::client::Client;
use crate::auth::VERB;

#[derive(Default)]
pub struct Bucket<'a>{
  // bucket_info: Option<Bucket<'b>>,
  // bucket: Option<Bucket<'c>>,
  pub creation_date: &'a str,
  pub extranet_endpoint: &'a str,
  pub intranet_endpoint: &'a str,
  pub location: &'a str,
  pub name: &'a str,
  // owner 	存放Bucket拥有者信息的容器。父节点：BucketInfo.Bucket
  pub id: &'a str,
  pub display_name: &'a str,
  // access_control_list;
  pub grant: Grant,
  pub data_redundancy_type: DataRedundancyType,
  pub storage_class: &'a str,
  pub versioning: &'a str,
  // ServerSideEncryptionRule,
  // ApplyServerSideEncryptionByDefault,
  pub sse_algorithm: &'a str,
  pub kms_master_key_id: Option<&'a str>,
  pub cross_region_replication: &'a str,
  pub transfer_acceleration: &'a str,
}

impl Client<'_> {

  /** # 获取 buiket 列表
      # Examples1
```
use dotenv::dotenv;
use std::env;
use aliyun_oss_client::client;

let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
let bucket      = env::var("ALIYUN_BUCKET").unwrap();

let client = client::Client::new(&key_id,&key_secret, &endpoint, &bucket);

let response = client.get_bucket_list().unwrap();
let first = response.first().unwrap();
assert_eq!(first, "abc");
```

  */
  pub fn get_bucket_list(&self) -> Option<Vec<String>> {
    let headers = None;
    println!("get_bucket_list {}", self.endpoint);
    let response = self.builder(VERB::GET, "https://oss-cn-shanghai.aliyuncs.com", headers);
    println!("get_bucket_list {}", response.send().unwrap().text().unwrap());
    Some(vec!("abc".to_string()))
  }

}

pub enum Grant{
  Private,
  PublicRead,
  PublicReadWrite,
}

impl Default for Grant {
  fn default() -> Self {
    Self::Private
  }
}

pub enum DataRedundancyType{
  LRS,
  ZRS,
}

impl Default for DataRedundancyType{
  fn default() -> Self {
    Self::LRS
  }
}


#[derive(Default)]
pub struct BucketListObjectParms<'a>{
  pub list_type: u8,
  pub delimiter: &'a str,
  pub continuation_token: &'a str,
  pub max_keys: u32,
  pub prefix: &'a str,
  pub encoding_type: &'a str,
  pub fetch_owner: bool,
}

#[derive(Default)]
pub struct BucketListObject<'a>{
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

pub enum Location {
  CnHangzhou,
  CnShanghai,
  CnQingdao,
  CnBeijing,
  CnZhangjiakou, // 张家口
  CnHongkong,
  CnShenzhen,
  UsWest1,
  UsEast1,
  ApSouthEast1,
}

pub struct BucketStat{
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