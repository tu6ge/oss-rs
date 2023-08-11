use std::{io::Write, sync::Arc};

use aliyun_oss_client::{auth::query::QueryAuth, config::Config, object::content::Content};
use chrono::Utc;
use dotenv::dotenv;
use reqwest::Client;

#[tokio::test]
async fn run() {
    dotenv().ok();
    let config = Config::from_env().unwrap();

    let auth = QueryAuth::from(&config);

    let time = Utc::now().timestamp() + 3600;
    let url = auth.to_url(&"babel.config.js".parse().unwrap(), time);

    let client = Client::new().get(url).send().await.unwrap();
    assert!(client.status().is_success());
}

#[test]
fn test_multi_upload() {
    dotenv::dotenv().ok();

    let client = aliyun_oss_client::Client::from_env().unwrap();

    let mut objcet = Content::from_client(Arc::new(client))
        .path("aaabbb3.txt")
        .unwrap();

    //objcet.part_size(100 * 1024).unwrap();

    {
        objcet
            .write_all(b"abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789")
            .unwrap();
        objcet.flush().unwrap()
    }

    println!("finish");

    // let mut buf = [0u8; 11];
    // objcet.set_size("72").unwrap();
    // objcet.seek(std::io::SeekFrom::Start(26)).unwrap();
    // objcet.read(&mut buf).unwrap();

    // println!("buf: {:?}", buf);
}
