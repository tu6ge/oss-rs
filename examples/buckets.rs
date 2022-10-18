//! `cargo run --example buckets --features=blocking`
#![deny(warnings)]

use aliyun_oss_client::blocking::client;
use aliyun_oss_client::types::Query;

extern crate dotenv;

use dotenv::dotenv;

fn main() {
    dotenv().ok();

    let client = client::Client::from_env().unwrap();
    //let headers = None;
    let response = client.get_bucket_list().unwrap();
    println!("buckets list: {:?}", response.buckets.first().unwrap());

    let mut query = Query::new();
    query.insert("max-keys".to_string(), "5".to_string());
    query.insert("prefix".to_string(), "babel".to_string());

    let buckets = response.buckets;
    let the_bucket = &buckets[1];
    println!("bucket object list: {:?}", the_bucket.get_object_list(query));
}
