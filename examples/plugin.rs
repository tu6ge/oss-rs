extern crate dotenv;

use aliyun_oss_client::plugin::Plugin;
use dotenv::dotenv;
use aliyun_oss_client::client::Client;
use aliyun_oss_client::auth::{VERB};
use reqwest::Url;
use reqwest::header::{HeaderMap};
use std::env;

fn main() {
  dotenv().ok();

  let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
  let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
  let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
  let bucket      = env::var("ALIYUN_BUCKET").unwrap();

  let my_plugin = MyPlugin{};

  let client = Client::new(&key_id,&key_secret, &endpoint, &bucket)
    .plugin(Box::new(my_plugin));

  let mut url = client.get_bucket_url().unwrap();
  url.set_path("file_copy.txt");

  let mut headers = HeaderMap::new();
  headers.insert("x-oss-copy-source", "/honglei123/file1.txt".parse().unwrap());
  headers.insert("x-oss-metadata-directive", "COPY".parse().unwrap());

  let request = client.builder(VERB::PUT, &url, Some(headers), None).unwrap();

  let response = request.send().unwrap();

  println!("copy result: {:?}", response);
}

struct MyPlugin {

}

impl Plugin for MyPlugin{
  fn name(&self) -> &'static str {
    "my_plugin"
  }

  fn canonicalized_resource(&self, _url: &Url) -> Option<String>{
    None
  }
}