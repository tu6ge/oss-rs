extern crate dotenv;

use dotenv::dotenv;
use std::env;
use aliyun_oss_client::client;

fn main() {
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = client::Client::new(&key_id,&key_secret, &endpoint, &bucket);
    //let headers = None;
    let response = client.get_bucket_info().unwrap();
    println!("bucket info: {:?}", response);
}
