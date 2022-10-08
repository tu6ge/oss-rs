//! `cargo run --example plugin --features=blocking,plugin`
extern crate dotenv;

use aliyun_oss_client::errors::{OssResult, OssError};
use aliyun_oss_client::plugin::Plugin;
use aliyun_oss_client::types::{EndPoint};
use dotenv::dotenv;
use aliyun_oss_client::client::Client;
use aliyun_oss_client::auth::{VERB};
use reqwest::Url;
use reqwest::header::{HeaderMap};
use std::env;

#[tokio::main]
async fn main() {
  dotenv().ok();
  use futures::try_join;
  use aliyun_oss_client::config::{ObjectBase, BucketBase};
  use aliyun_oss_client::types::CanonicalizedResource;

  let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
  let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
  let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
  let bucket      = env::var("ALIYUN_BUCKET").unwrap();

  let my_plugin = MyPlugin{bucket:"abc".to_string()};

  let client = aliyun_oss_client::client(key_id,key_secret, endpoint.clone(), bucket.clone())
    .plugin(Box::new(my_plugin)).unwrap()
    ;

  let mut url = client.get_bucket_url().unwrap();
  url.set_path("file_copy.txt");

  let mut headers = HeaderMap::new();
  headers.insert("x-oss-copy-source", "/honglei123/file1.txt".parse().unwrap());
  headers.insert("x-oss-metadata-directive", "COPY".parse().unwrap());

  let object_base = ObjectBase::new(BucketBase::new(bucket.clone().into(), endpoint.clone().into()), "file1.txt");
    
  let canonicalized = CanonicalizedResource::from_object(&object_base, None);

  let request = client.builder_with_header(VERB::PUT, &url, canonicalized, Some(headers.clone())).await.unwrap();

  //let response = request.send().await.unwrap();

  let mut url2 = client.get_bucket_url().unwrap();
  url2.set_path("file_copy2.txt");

  let object_base = ObjectBase::new(BucketBase::new(bucket.into(), endpoint.into()), "file2.txt");
    
  let canonicalized = CanonicalizedResource::from_object(&object_base, None);

  let request2 = client.builder_with_header(VERB::PUT, &url2, canonicalized, Some(headers)).await.unwrap();

  //let response2 = request2.send().await.unwrap();

  let result = try_join!(request.send(), request2.send());
  println!("result {:?}", result);
  //println!("copy result1: {:?}, result2: {:?}", response, response2);
}

struct MyPlugin {
  bucket: String,
}

impl Plugin for MyPlugin{
  fn name(&self) -> &'static str {
    "my_plugin"
  }
  
  fn initialize(&mut self, client: &mut Client) -> OssResult<()> {
    // // 插件可以读取 client 结构体中的值
    // self.bucket = client.endpoint.to_string();

    // // 插件可以修改 client 结构体中的值
    // client.endpoint = EndPoint::new("https://oss-cn-shanghai.aliyuncs.com")
    //   .map_err(|e|OssError::InvalidEndPoint(e))?;
    Ok(())
  }
}