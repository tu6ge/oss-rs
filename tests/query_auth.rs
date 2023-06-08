use aliyun_oss_client::{auth::query::QueryAuth, config::Config};
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
