//! `cargo run --example put_file --features=blocking`
use aliyun_oss_client::blocking::client::Client;

extern crate dotenv;

use dotenv::dotenv;
use std::{path::PathBuf};

fn main() {
    dotenv().ok();

    let client = Client::from_env().unwrap();
    //let headers = None;
    let response = client.put_file(PathBuf::from("examples/bg2015071010.png"), "examples/bg2015071010.png").unwrap();
    println!("put file result: {:?}", response);
}
