use std::pin::Pin;

use futures::{FutureExt, StreamExt};
use futures_core::{Future, Stream};

use aliyun_oss_client::Client;
use dotenv::dotenv;
use pin_project::pin_project;

#[pin_project]
struct Struct {
    a: u8,
    b: String,
    #[pin]
    client: reqwest::Client,
}

impl Stream for Struct {
    type Item = String;
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let that = self.project();
        // dotenv().ok();
        // let client = reqwest::Client::new();
        let request = that
            .client
            .as_ref()
            .get("https://course.rs")
            .build()
            .unwrap();

        let mut rsp = that.client.execute(request);

        let r = Pin::new(&mut rsp).poll(cx);

        r.map(|r| Some(format!("{:?}", r.unwrap())))
    }
}

#[tokio::main]
async fn main() {
    let mut struct1 = Struct {
        a: 12,
        b: "12".to_owned(),
        client: reqwest::Client::new(),
    };

    let second = struct1.next().await;

    println!("second: {:?}", second);
}
