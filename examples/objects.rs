//! `cargo run --example objects --features=blocking`

use aliyun_oss_client::client;

extern crate dotenv;

use dotenv::dotenv;
use std::{env};
use aliyun_oss_client::types::Query;

fn main() {
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = client::Client::new(key_id.into(),key_secret.into(), endpoint.into(), bucket.into());
    //let headers = None;
    let mut query = Query::new();
    query.insert("max-keys".to_string(), "5".to_string());
    //query.insert("prefix".to_string(), "babel".to_string());
    let response = client.blocking_get_object_list(query).unwrap();
    println!("objects list: {:?}", response);
}
