//! `cargo run --example objects --features=blocking`

use aliyun_oss_client::blocking::client;

extern crate dotenv;

use dotenv::dotenv;
use aliyun_oss_client::types::Query;

fn main() {
    dotenv().ok();

    let client = client::Client::from_env().unwrap();
    //let headers = None;
    let mut query = Query::new();
    query.insert("max-keys".to_string(), "5".to_string());
    //query.insert("prefix".to_string(), "babel".to_string());
    let response = client.get_object_list(query).unwrap();
    println!("objects list: {:?}", response);
}
