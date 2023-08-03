use std::{io::Write, rc::Rc, sync::Arc};

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

    let client = aliyun_oss_client::ClientRc::from_env().unwrap();

    let mut objcet = Content::from_client(Rc::new(client))
        .path("aaabbb3.txt")
        .unwrap();

    objcet.part_size(100 * 1024).unwrap();

    objcet.write_all(&[97; 200 * 1024]).unwrap();

    objcet.flush().unwrap();
}
