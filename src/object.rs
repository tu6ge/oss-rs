use chrono::{DateTime, Utc};
use reqwest::{
    header::{HeaderMap, CONTENT_LENGTH, CONTENT_TYPE},
    Method,
};

use crate::{
    bucket::Bucket,
    client::{now, Client},
    error::OssError,
    types::CanonicalizedResource,
};

#[derive(Debug, Default)]
pub struct Objects {
    list: Vec<Object>,
    next_token: Option<String>,
}

impl Objects {
    pub fn new(list: Vec<Object>, next_token: Option<String>) -> Objects {
        Objects { list, next_token }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }
}

#[derive(Debug)]
pub struct Object {
    bucket: Bucket,
    path: String,
}

impl Object {
    pub fn new(bucket: Bucket, path: String) -> Object {
        Object { bucket, path }
    }
    pub async fn get_info(&self, client: &Client) -> Result<ObjectInfo, OssError> {
        const LINE_BREAK: &str = "\n";

        let mut url = self.bucket.to_url();
        url.set_path(&self.path);
        url.set_query(Some("objectMeta"));
        let method = Method::HEAD;
        let date = now();
        let resource = CanonicalizedResource::new(format!(
            "/{}/{}?objectMeta",
            self.bucket.as_str(),
            self.path
        ));
        let content_type = "text/xml";

        let sign = {
            let method = Method::GET;
            let mut string = method.as_str().to_owned();
            string += LINE_BREAK;
            string += LINE_BREAK;
            string += content_type;
            string += LINE_BREAK;
            string += date.as_str();
            string += LINE_BREAK;
            string += resource.as_str();

            let encry = client.secret().encryption(string.as_bytes()).unwrap();

            format!("OSS {}:{}", client.key().as_str(), encry)
        };

        let header_map = {
            let mut headers = HeaderMap::new();
            headers.insert("AccessKeyId", client.key().as_str().try_into()?);
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

        let response = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?;

        let headers = response.headers();

        let content_length = match headers.get(CONTENT_LENGTH) {
            Some(v) => v,
            None => return Err(OssError::NoFoundContentLength),
        };
        let etag = match headers.get("etag") {
            Some(v) => v,
            None => return Err(OssError::NoFoundEtag),
        };
        let last_modified = match headers.get("last-modified") {
            Some(v) => v,
            None => return Err(OssError::NoFoundLastModified),
        };
        let last_modified = last_modified.to_str()?;

        let date = DateTime::parse_from_rfc2822(last_modified)?;
        Ok(ObjectInfo {
            last_modified: date.with_timezone(&Utc),
            etag: etag.to_str()?.to_string(),
            size: content_length.to_str()?.parse()?,
        })
    }

    pub async fn upload(&self, content: Vec<u8>, client: &Client) -> Result<(), OssError> {
        const LINE_BREAK: &str = "\n";

        let mut url = self.bucket.to_url();
        url.set_path(&self.path);
        let method = Method::PUT;
        let date = now();
        let resource =
            CanonicalizedResource::new(format!("/{}/{}", self.bucket.as_str(), self.path));
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

            let encry = client.secret().encryption(string.as_bytes()).unwrap();

            format!("OSS {}:{}", client.key().as_str(), encry)
        };

        let header_map = {
            let mut headers = HeaderMap::new();
            headers.insert("AccessKeyId", client.key().as_str().try_into()?);
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

        let response = reqwest::Client::new()
            .put(url)
            .headers(header_map)
            .body(content)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let body = response.text().await?;
            Err(OssError::Upload(body))
        }
    }
    pub async fn download(&self, client: &Client) -> Result<Vec<u8>, OssError> {
        const LINE_BREAK: &str = "\n";

        let mut url = self.bucket.to_url();
        url.set_path(&self.path);
        let method = Method::GET;
        let date = now();
        let resource =
            CanonicalizedResource::new(format!("/{}/{}", self.bucket.as_str(), self.path));
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

            let encry = client.secret().encryption(string.as_bytes()).unwrap();

            format!("OSS {}:{}", client.key().as_str(), encry)
        };

        let header_map = {
            let mut headers = HeaderMap::new();
            headers.insert("AccessKeyId", client.key().as_str().try_into()?);
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

        let response = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?
            .text()
            .await?;

        Ok(response.into())
    }
}

#[derive(Debug)]
pub struct ObjectInfo {
    last_modified: DateTime<Utc>,
    etag: String,
    size: u64,
}
impl ObjectInfo {
    pub fn new(last_modified: DateTime<Utc>, etag: String, size: u64) -> Self {
        ObjectInfo {
            last_modified,
            etag,
            size,
        }
    }

    pub fn last_modified(&self) -> &DateTime<Utc> {
        &self.last_modified
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn etag(&self) -> &str {
        &self.etag
    }
}

#[cfg(test)]
mod tests {
    use super::Object;
    use crate::{bucket::Bucket, client::initClient, types::EndPoint};

    #[tokio::test]
    async fn test_object_info() {
        let object = Object::new(
            Bucket::new("honglei123".into(), EndPoint::CN_SHANGHAI),
            "app-config.json".into(),
        );

        let info = object.get_info(&initClient()).await.unwrap();

        println!("{info:?}");
    }

    #[tokio::test]
    async fn test_upload() {
        let object = Object::new(
            Bucket::new("honglei123".into(), EndPoint::CN_SHANGHAI),
            "abc.txt".into(),
        );

        let info = object.upload("aaa".into(), &initClient()).await.unwrap();

        println!("{info:?}");
    }

    #[tokio::test]
    async fn test_down() {
        let object = Object::new(
            Bucket::new("honglei123".into(), EndPoint::CN_SHANGHAI),
            "abc.txt".into(),
        );

        let info = object.download(&initClient()).await.unwrap();

        println!("{info:?}");
    }
}
