use std::{fs::File, io::Read, sync::Arc};

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

#[tokio::test]
async fn test_multi_upload() {
    dotenv::dotenv().ok();

    let s = "aaa";
    let a = &s[0..10];
    println!("{a}");
    todo!();
    let client = aliyun_oss_client::Client::from_env().unwrap();

    let mut objcet = Content::from_client(Arc::new(client))
        .path("aaabbb2.txt")
        .unwrap();
    let res = objcet.init_multi().await;
    println!("{res:#?}");

    // let _ = objcet.upload_part(1, &[98; 200000]).await.unwrap();
    // let _ = objcet.upload_part(2, &[98; 200000]).await.unwrap();

    let _ = objcet.abort_multi().await.unwrap();
}
