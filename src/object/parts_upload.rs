use std::{
    fs::File,
    io::{BufReader, Read},
};

use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_LENGTH},
    Method,
};
use url::Url;

use crate::{types::CanonicalizedResource, Bucket, Client, Error as OssError};

pub struct PartsUpload {
    path: String,
    upload_id: String,
    etags: Vec<(usize, String)>,
    file_path: String,
    part_size: usize,
}

impl PartsUpload {
    pub fn new<P: Into<String>>(path: P) -> PartsUpload {
        PartsUpload {
            path: path.into(),
            upload_id: String::new(),
            etags: Vec::new(),
            file_path: String::new(),
            part_size: 1024 * 1024,
        }
    }

    pub fn to_url(&self, bucket: &Bucket) -> Result<Url, OssError> {
        let mut url = bucket.to_url()?;
        url.set_path(&self.path);
        Ok(url)
    }

    pub fn file_path(mut self, file_path: String) -> Self {
        self.file_path = file_path;
        self
    }
    pub fn part_size(mut self, part_size: usize) -> Self {
        self.part_size = part_size;
        self
    }
    pub async fn upload(&mut self, client: &Client) -> Result<(), OssError> {
        let file = File::open(self.file_path.clone())?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; self.part_size];

        self.init_mulit(client).await?;

        let mut index = 1_usize;

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break; // EOF
            }

            self.upload_part(index, buffer[..bytes_read].to_vec(), client)
                .await?;

            index += 1;
        }

        self.complete(client).await
    }

    pub async fn init_mulit(&mut self, client: &Client) -> Result<(), OssError> {
        let bucket = client.bucket().ok_or(OssError::NoFoundBucket)?;
        let mut url = self.to_url(bucket)?;
        url.set_query(Some("uploads"));
        let method = Method::POST;
        let resource =
            CanonicalizedResource::new(format!("/{}/{}?uploads", bucket.as_str(), self.path));

        let header_map = client.authorization(&method, resource)?;

        let xml = reqwest::Client::new()
            .request(method, url)
            .headers(header_map)
            .send()
            .await?
            .text()
            .await?;

        self.parse_upload_id(&xml)
    }
    fn parse_upload_id(&mut self, xml: &str) -> Result<(), OssError> {
        if let (Some(start), Some(end)) = (xml.find("<UploadId>"), xml.find("</UploadId>")) {
            self.upload_id = (&xml[start + 10..end]).to_owned();
            Ok(())
        } else {
            Err(OssError::NoFoundUploadId)
        }
    }

    pub async fn upload_part(
        &mut self,
        index: usize,
        content: Vec<u8>,
        client: &Client,
    ) -> Result<(), OssError> {
        if self.upload_id.is_empty() {
            return Err(OssError::NoFoundUploadId);
        }

        let bucket = client.bucket().ok_or(OssError::NoFoundBucket)?;
        let mut url = self.to_url(bucket)?;
        url.set_query(Some(&format!(
            "partNumber={}&uploadId={}",
            index, self.upload_id
        )));
        let resource = CanonicalizedResource::new(format!(
            "/{}/{}?partNumber={}&uploadId={}",
            bucket.as_str(),
            self.path,
            index,
            self.upload_id
        ));
        let content_length = content.len().to_string();
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_LENGTH,
            HeaderValue::from_str(&content_length).unwrap(),
        );

        let method = Method::PUT;

        let header_map = client.authorization_header(&method, resource, headers)?;

        let response = reqwest::Client::new()
            .request(method, url)
            .headers(header_map)
            .body(content)
            .send()
            .await?;
        let headers = response.headers();

        if let Some(value) = headers.get("ETag") {
            if let Ok(str) = value.to_str() {
                self.etags.push((index, str.to_string()));
                return Ok(());
            }
        }

        Err(OssError::NoFoundEtag)
    }

    pub async fn complete(&mut self, client: &Client) -> Result<(), OssError> {
        if self.upload_id.is_empty() {
            return Err(OssError::NoFoundUploadId);
        }
        if self.etags.is_empty() {
            return Err(OssError::NoFoundEtag);
        }

        let bucket = client.bucket().ok_or(OssError::NoFoundBucket)?;
        let mut url = self.to_url(bucket)?;
        url.set_query(Some(&format!("uploadId={}", self.upload_id)));
        let resource = CanonicalizedResource::new(format!(
            "/{}/{}?uploadId={}",
            bucket.as_str(),
            self.path,
            self.upload_id
        ));

        let xml = self.etag_list_xml();
        let content_length = xml.len().to_string();
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_LENGTH,
            HeaderValue::from_str(&content_length).unwrap(),
        );

        let method = Method::POST;

        let header_map = client.authorization_header(&method, resource, headers)?;

        let response = reqwest::Client::new()
            .request(method, url)
            .headers(header_map)
            .body(xml)
            .send()
            .await?;

        if response.status().is_success() {
            self.upload_id = String::new();
            self.etags = Vec::new();
            Ok(())
        } else {
            let body = response.text().await?;
            Err(OssError::from_service(&body))
        }
    }

    fn etag_list_xml(&self) -> String {
        let mut list = String::new();
        for (index, etag) in self.etags.iter() {
            list.push_str(&format!(
                "<Part><PartNumber>{}</PartNumber><ETag>{}</ETag></Part>",
                index, etag
            ));
        }

        format!(
            "<CompleteMultipartUpload>{}</CompleteMultipartUpload>",
            list
        )
    }

    pub async fn abort(&mut self, client: &Client) -> Result<(), OssError> {
        if self.upload_id.is_empty() {
            return Err(OssError::NoFoundUploadId);
        }
        let bucket = client.bucket().ok_or(OssError::NoFoundBucket)?;
        let mut url = self.to_url(bucket)?;
        url.set_query(Some(&format!("uploadId={}", self.upload_id)));
        let resource = CanonicalizedResource::new(format!(
            "/{}/{}?uploadId={}",
            bucket.as_str(),
            self.path,
            self.upload_id
        ));
        let method = Method::DELETE;
        let header_map = client.authorization(&method, resource)?;

        let _response = reqwest::Client::new()
            .request(method, url)
            .headers(header_map)
            .send()
            .await?;

        self.upload_id = String::new();
        self.etags = Vec::new();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{client::init_client, object::PartsUpload, Bucket, Client, EndPoint};

    fn build_bucket() -> Bucket {
        use crate::types::KnownRegion;
        use crate::types::Region;
        Bucket::new(
            "honglei123",
            EndPoint::new(Region::Known(KnownRegion::CnShanghai)),
        )
    }

    fn set_client() -> Client {
        let mut client = init_client();
        client.set_bucket(build_bucket());
        client
    }

    #[tokio::test]
    async fn test_upload() {
        let object = PartsUpload::new("myvideo23.mov");

        let info = object
            .file_path("./video.mov".into())
            .upload(&set_client())
            .await;

        println!("{info:?}");
    }
}
