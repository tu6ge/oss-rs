use std::fs;

use aliyun_oss_client::builder::{BuilderError, RequestBuilder};
use aliyun_oss_client::config::BucketBase;
use aliyun_oss_client::file::{AlignBuilder, FileError, Files, GetStdWithPath};
use aliyun_oss_client::types::{object::ObjectPath, CanonicalizedResource};
use aliyun_oss_client::{BucketName, Client, EndPoint, HeaderName, HeaderValue, Method};
use reqwest::Url;

struct MyClient;

#[derive(Debug)]
struct MyError(String);

impl From<FileError> for MyError {
    fn from(value: FileError) -> Self {
        Self(value.to_string())
    }
}

struct MyPath(String);

impl AlignBuilder for MyClient {
    fn builder_with_header<H: IntoIterator<Item = (HeaderName, HeaderValue)>>(
        &self,
        method: Method,
        url: Url,
        resource: CanonicalizedResource,
        headers: H,
    ) -> Result<RequestBuilder, BuilderError> {
        dotenv::dotenv().ok();
        Client::from_env()?.builder_with_header(method, url, resource, headers)
    }
}

impl GetStdWithPath<MyPath> for MyClient {
    fn get_std_with_path(&self, path: MyPath) -> Option<(Url, CanonicalizedResource)> {
        let bucket = std::env::var("ALIYUN_BUCKET").unwrap();

        let end_point = EndPoint::CnShanghai;
        let bucket = BucketName::new(bucket).unwrap();
        let base = BucketBase::new(bucket, end_point);
        let obj_path = ObjectPath::try_from(path.0).unwrap();
        Some(base.get_url_resource_with_path(&obj_path))
    }
}

impl GetStdWithPath<MyPath> for Client {
    fn get_std_with_path(&self, path: MyPath) -> Option<(Url, CanonicalizedResource)> {
        let bucket = std::env::var("ALIYUN_BUCKET").unwrap();

        let end_point = EndPoint::CnShanghai;
        let bucket = BucketName::new(bucket).unwrap();
        let base = BucketBase::new(bucket, end_point);
        let obj_path = ObjectPath::try_from(path.0).unwrap();
        Some(base.get_url_resource_with_path(&obj_path))
    }
}

#[tokio::main]
async fn main() {
    let client = MyClient {};

    let file = fs::read("rustfmt.toml").unwrap();
    let res = client
        .put_content_base(file, "application/json", MyPath("rustfmt.toml".to_string()))
        .await;

    println!("{res:?}");
}
