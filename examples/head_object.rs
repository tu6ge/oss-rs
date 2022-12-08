use aliyun_oss_client::{
    builder::ArcPointer, config::ObjectBase, errors::OssError, file::AlignBuilder, Client,
};
use dotenv::dotenv;

#[tokio::main]
pub async fn main() -> Result<(), OssError> {
    dotenv().ok();

    let client = Client::from_env().unwrap();

    let object_base =
        ObjectBase::<ArcPointer>::from_bucket(client.get_bucket_base(), "9AB932LY.jpeg");

    let (url, resource) = object_base.get_url_resource([]);

    let headers = vec![(
        "If-Unmodified-Since".parse().unwrap(),
        "Sat, 01 Jan 2022 18:01:01 GMT".parse().unwrap(),
    )];

    let builder = client.builder_with_header("HEAD", url, resource, headers)?;

    let response = builder.send().await?;

    println!("status: {:?}", response.status());

    Ok(())
}
