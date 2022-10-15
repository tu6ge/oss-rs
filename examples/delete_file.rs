//! `cargo run --example delete_file --features=blocking`

use aliyun_oss_client::blocking::client;

extern crate dotenv;

use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = client::Client::new(key_id.into(),key_secret.into(), endpoint.into(), bucket.into());
    //let headers = None;
    client.delete_object("examples/bg2015071010.png").unwrap();
    println!("delet file success");
}
