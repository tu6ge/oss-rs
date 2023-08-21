use aliyun_oss_client::{errors::OssError, file::AlignBuilder, Client, Method, TryIntoHeaders};
use dotenv::dotenv;
use http::header::InvalidHeaderValue;

#[tokio::main]
pub async fn main() -> Result<(), OssError> {
    dotenv().ok();

    let client = Client::from_env().unwrap();

    let (url, resource) = client
        .get_object_base("9AB932LY.jpeg")?
        .get_url_resource([()]);

    let builder = client.builder_with_header(
        Method::HEAD,
        url,
        resource,
        IfUnmodifiedSince {
            date: "Sat, 01 Jan 2022 18:01:01 GMT",
        },
    )?;

    let response = builder.send().await?;

    println!("status: {:?}", response.status());

    Ok(())
}

struct IfUnmodifiedSince {
    date: &'static str,
}

impl TryIntoHeaders for IfUnmodifiedSince {
    type Error = InvalidHeaderValue;
    fn try_into_headers(self) -> Result<http::HeaderMap, Self::Error> {
        let mut map = http::HeaderMap::with_capacity(1);
        map.insert("If-Unmodified-Since", self.date.parse()?);
        Ok(map)
    }
}
