use std::env;

use aliyun_oss_client::{
    auth::query::{Object, QueryAuth},
    ObjectPath, Result,
};
use chrono::Utc;
use dotenv::dotenv;
use reqwest::Client;

fn init_object(path: ObjectPath) -> Result<Object> {
    dotenv().ok();
    let endpoint = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket = env::var("ALIYUN_BUCKET").unwrap();

    Ok(Object::new(
        endpoint.parse().unwrap(),
        bucket.parse().unwrap(),
        path,
    ))
}

fn init_auth(path: ObjectPath, expires: i64) -> Result<QueryAuth> {
    let object = init_object(path)?;
    dotenv().ok();
    let key_id = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret = env::var("ALIYUN_KEY_SECRET").unwrap();

    Ok(QueryAuth::new_with_object(
        object,
        key_id.into(),
        key_secret.into(),
        expires,
    ))
}

#[tokio::test]
async fn run() {
    let time = Utc::now().timestamp() + 3600;
    let auth = init_auth("babel.config.js".parse().unwrap(), time).unwrap();

    let url = auth.to_url().unwrap();

    let client = Client::new().get(url).send().await.unwrap();
    assert!(client.status().is_success());
}
