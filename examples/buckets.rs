//! `cargo run --example buckets --features=blocking`
#![deny(warnings)]

use aliyun_oss_client::client;
use aliyun_oss_client::types::BucketName;

extern crate dotenv;

use dotenv::dotenv;
use std::{env, collections::HashMap};

fn main() {
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();

    let client = client::Client::new(key_id.into(),key_secret.into(), endpoint.into(), BucketName::from_static(""));
    //let headers = None;
    let response = client.blocking_get_bucket_list().unwrap();
    println!("buckets list: {:?}", response.buckets.first().unwrap());

    let mut query:HashMap<String,String> = HashMap::new();
    query.insert("max-keys".to_string(), "5".to_string());
    query.insert("prefix".to_string(), "babel".to_string());

    let buckets = response.buckets;
    let the_bucket = &buckets[1];
    println!("bucket object list: {:?}", the_bucket.blocking_get_object_list(query));
}
