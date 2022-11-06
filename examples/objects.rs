//! `cargo run --example objects --features=blocking`

use aliyun_oss_client::builder::ClientWithMiddleware;
use aliyun_oss_client::client::Client;
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = Client::<ClientWithMiddleware>::from_env().unwrap();

    let response = client.get_object("app-config.json", 10..16).await.unwrap();
    println!(
        "objects content: {:?}",
        String::from_utf8(response).unwrap()
    );
}
