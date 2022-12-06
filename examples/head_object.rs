use std::{env, sync::Arc};

use aliyun_oss_client::{
    auth::{AuthBuilder, AuthGetHeader},
    builder::ArcPointer,
    config::{ObjectBase, ObjectPath, UrlObjectPath},
    errors::OssError,
    types::CanonicalizedResource,
    Client,
};
use chrono::Utc;
use dotenv::dotenv;
use http::HeaderMap;

#[tokio::main]
pub async fn main() -> Result<(), OssError> {
    dotenv().ok();

    let key_id = env::var("ALIYUN_KEY_ID").map_err(OssError::from)?;
    let key_secret = env::var("ALIYUN_KEY_SECRET").map_err(OssError::from)?;

    let client = Client::from_env().unwrap();

    let path = ObjectPath::new("9AB932LY.jpeg");

    let mut url = client.get_bucket_url();
    url.set_object_path(&path);

    let object_base = ObjectBase::<ArcPointer>::new(Arc::new(client.get_bucket_base()), path);

    let resource = CanonicalizedResource::from_object(object_base, []);

    let mut headers = HeaderMap::new();

    headers.insert(
        "If-Unmodified-Since",
        "Sat, 01 Jan 2022 18:01:01 GMT".try_into().unwrap(),
    );

    let reqw_header = AuthBuilder::default()
        .key(key_id.into())
        .secret(key_secret.into())
        .verb(&"HEAD".into())
        .date(Utc::now().into())
        .canonicalized_resource(resource)
        .with_headers(Some(headers))
        .get_headers()
        .unwrap();

    let client = reqwest::Client::new();
    let response = client.head(url).headers(reqw_header).send().await?;

    println!("result status: {:?}", response.status());

    Ok(())
}
