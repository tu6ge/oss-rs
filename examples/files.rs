use std::fs;

use aliyun_oss_client::builder::{BuilderError, RequestBuilder};
use aliyun_oss_client::file::{AlignBuilder, FileError, Files};
use aliyun_oss_client::types::CanonicalizedResource;
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

impl Files for MyClient {
    type Err = MyError;
    type Path = MyPath;
    fn get_url(&self, path: Self::Path) -> Result<(Url, CanonicalizedResource), Self::Err> {
        use aliyun_oss_client::config::OssFullUrl;

        dotenv::dotenv().ok();
        let bucket = std::env::var("ALIYUN_BUCKET").unwrap();

        let end_point = EndPoint::CnShanghai;
        let bucket = BucketName::new(bucket).unwrap();

        let resource = format!("/{}/{}", bucket, path.0);

        let p = path
            .0
            .try_into()
            .map_err(|_| MyError("路径格式错误".to_string()))?;
        let url = Url::from_oss(&end_point, &bucket, &p);

        Ok((url, CanonicalizedResource::new(resource)))
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
