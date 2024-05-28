use chrono::Utc;
use reqwest::{
    header::{HeaderMap, CONTENT_TYPE},
    Method,
};

use crate::{
    bucket::Bucket,
    error::OssError,
    types::{CanonicalizedResource, EndPoint, Key, Secret},
};

pub struct Client {
    key: Key,
    secret: Secret,
    bucket: Option<Bucket>,
}

impl Client {
    pub fn new(key: Key, secret: Secret) -> Client {
        Self {
            key,
            secret,
            bucket: None,
        }
    }

    pub(crate) fn key(&self) -> &Key {
        &self.key
    }
    pub(crate) fn secret(&self) -> &Secret {
        &self.secret
    }

    pub fn set_bucket(&mut self, bucket: Bucket) -> Option<Bucket> {
        self.bucket.replace(bucket)
    }

    pub fn bucket(&self) -> Option<&Bucket> {
        self.bucket.as_ref()
    }

    pub(crate) fn authorization(
        &self,
        method: Method,
        resource: CanonicalizedResource,
    ) -> Result<HeaderMap, OssError> {
        const LINE_BREAK: &str = "\n";

        let date = now();
        let content_type = "text/xml";

        let sign = {
            let mut string = method.as_str().to_owned();
            string += LINE_BREAK;
            string += LINE_BREAK;
            string += content_type;
            string += LINE_BREAK;
            string += date.as_str();
            string += LINE_BREAK;
            string += resource.as_str();

            let encry = self.secret.encryption(string.as_bytes()).unwrap();

            format!("OSS {}:{}", self.key.as_str(), encry)
        };
        let header_map = {
            let mut headers = HeaderMap::new();
            headers.insert("AccessKeyId", self.key.as_str().try_into()?);
            headers.insert("VERB", method.as_str().try_into()?);
            headers.insert("Date", date.try_into()?);
            headers.insert("Authorization", sign.try_into()?);
            headers.insert(CONTENT_TYPE, content_type.try_into()?);
            headers.insert(
                "CanonicalizedResource",
                resource.as_str().try_into().unwrap(),
            );
            headers
        };

        Ok(header_map)
    }
    pub async fn get_buckets(&self, endpoint: EndPoint) -> Result<Vec<Bucket>, OssError> {
        let url = endpoint.to_url();
        let method = Method::GET;
        let resource = CanonicalizedResource::default();

        let header_map = self.authorization(method, resource)?;

        let content = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?
            .text()
            .await?;

        Self::parse_xml(content, endpoint)
    }

    fn parse_xml(xml: String, endpoint: EndPoint) -> Result<Vec<Bucket>, OssError> {
        let mut start_positions = vec![];
        let mut end_positions = vec![];
        let mut start = 0;
        let mut pattern = "<Name>";

        while let Some(pos) = xml[start..].find(pattern) {
            start_positions.push(start + pos);
            start += pos + pattern.len();
        }
        start = 0;
        pattern = "</Name>";
        while let Some(pos) = xml[start..].find(pattern) {
            end_positions.push(start + pos);
            start += pos + pattern.len();
        }

        let mut bucket = vec![];
        for i in 0..start_positions.len() {
            let name = &xml[start_positions[i] + 6..end_positions[i]];
            bucket.push(Bucket::new(name.to_owned(), endpoint.clone()))
        }

        Ok(bucket)
    }
}

fn now() -> String {
    Utc::now().format("%a, %d %b %Y %T GMT").to_string()
}

// fn now() -> DateTime<Utc> {
//     use chrono::NaiveDateTime;
//     let naive = NaiveDateTime::parse_from_str("2022/10/6 20:40:00", "%Y/%m/%d %H:%M:%S").unwrap();
//     DateTime::from_utc(naive, Utc)
// }

#[cfg(test)]
mod tests {
    use crate::{client::initClient, types::EndPoint};

    #[tokio::test]
    async fn test_get_buckets() {
        let list = initClient()
            .get_buckets(EndPoint::CN_QINGDAO)
            .await
            .unwrap();

        assert_eq!(list.len(), 2);
    }
}

#[cfg(test)]
pub fn initClient() -> Client {
    use std::env;

    use dotenv::dotenv;

    dotenv().ok();
    let key = env::var("ALIYUN_KEY_ID").unwrap();
    let secret = env::var("ALIYUN_KEY_SECRET").unwrap();

    Client::new(Key::new(key), Secret::new(secret))
}
