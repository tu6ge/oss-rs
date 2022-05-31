
use aliyun_oss_client::client;

extern crate dotenv;

use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();

    let client = client::Client::new(&key_id,&key_secret, &endpoint, "");
    //let headers = None;
    let response = client.get_bucket_list().unwrap();
    println!("buckets list: {:?}", response);
}
