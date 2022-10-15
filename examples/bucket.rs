extern crate dotenv;

use dotenv::dotenv;
use std::{env};
use aliyun_oss_client::blocking::client::Client;
use aliyun_oss_client::types::Query;

fn main() {
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = Client::new(key_id.into(),key_secret.into(), endpoint.into(), bucket.into());
    //let headers = None;
    let response = client.get_bucket_info().unwrap();
    println!("bucket info: {:?}", response);

    let mut query = Query::new();
    query.insert("max-keys".to_string(), "2".to_string());
    let mut result = response.get_object_list(query).unwrap();
    println!("object list: {:?}", result);
    println!("next object list: {:?}", result.next());
}
