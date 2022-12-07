use aliyun_oss_client::{config::ObjectBase, errors::OssError, file::AlignBuilder, Client};
use dotenv::dotenv;
use http::HeaderMap;

#[tokio::main]
pub async fn main() -> Result<(), OssError> {
    dotenv().ok();

    let client = Client::from_env().unwrap();

    let object_base = ObjectBase::from_bucket(client.get_bucket_base(), "9AB932LY.jpeg");

    let (url, resource) = object_base.get_url_resource([]);

    let mut headers = HeaderMap::new();
    headers.insert(
        "If-Unmodified-Since",
        "Sat, 01 Jan 2022 18:01:01 GMT".try_into().unwrap(),
    );

    let builder = client.builder_with_header("HEAD", url, resource, Some(headers))?;

    let response = builder.send().await?;

    println!("status: {:?}", response.status());

    Ok(())
}
