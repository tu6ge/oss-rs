use std::{
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_LENGTH},
    Method,
};
use url::Url;

use crate::{types::CanonicalizedResource, Bucket, Client, Error as OssError, Object};

#[derive(Debug)]
pub struct PartsUpload {
    path: String,
    bucket: Arc<Bucket>,
    upload_id: String,
    etags: Vec<(usize, String)>,
    file_path: PathBuf,
    part_size: usize,
}

impl PartsUpload {
    pub fn new<P: Into<String>>(path: P, bucket: Arc<Bucket>) -> PartsUpload {
        PartsUpload {
            path: path.into(),
            bucket,
            upload_id: String::new(),
            etags: Vec::new(),
            file_path: PathBuf::new(),
            part_size: 1024 * 1024,
        }
    }

    pub fn to_url(&self) -> Result<Url, OssError> {
        let mut url = self.bucket.to_url()?;
        url.set_path(&self.path);
        Ok(url)
    }

    pub fn from_file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.file_path = path.as_ref().to_path_buf();
        self
    }
    pub fn part_size(mut self, part_size: usize) -> Self {
        self.part_size = part_size;
        self
    }
    pub async fn upload(&mut self) -> Result<(), OssError> {
        let file = File::open(self.file_path.clone())?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; self.part_size];

        self.init_mulit().await?;

        let mut index = 1_usize;

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break; // EOF
            }

            self.upload_part(index, buffer[..bytes_read].to_vec())
                .await?;

            index += 1;
        }

        self.complete().await
    }

    pub async fn init_mulit(&mut self) -> Result<(), OssError> {
        let mut url = self.to_url()?;
        url.set_query(Some("uploads"));
        let method = Method::POST;

        let resource =
            CanonicalizedResource::new(format!("/{}/{}?uploads", self.bucket.as_str(), self.path));

        let header_map = self.bucket.client.authorization(&method, resource)?;

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
            self.upload_id = xml[start + 10..end].to_owned();
            Ok(())
        } else {
            Err(OssError::NoFoundUploadId)
        }
    }

    pub async fn upload_part(&mut self, index: usize, content: Vec<u8>) -> Result<(), OssError> {
        if self.upload_id.is_empty() {
            return Err(OssError::NoFoundUploadId);
        }

        let mut url = self.bucket.to_url()?;
        url.set_query(Some(&format!(
            "partNumber={}&uploadId={}",
            index, self.upload_id
        )));
        let resource = CanonicalizedResource::new(format!(
            "/{}/{}?partNumber={}&uploadId={}",
            self.bucket.as_str(),
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

        let header_map = self
            .bucket
            .client
            .authorization_header(&method, resource, headers)?;

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

    pub async fn complete(&mut self) -> Result<(), OssError> {
        self.complete_with_etag().await.map(|_| ())
    }

    /// 完成分片上传，返回合并后对象的 ETag（已去掉引号）。
    pub async fn complete_with_etag(&mut self) -> Result<String, OssError> {
        if self.upload_id.is_empty() {
            return Err(OssError::NoFoundUploadId);
        }
        if self.etags.is_empty() {
            return Err(OssError::NoFoundEtag);
        }

        let mut url = self.bucket.to_url()?;
        url.set_query(Some(&format!("uploadId={}", self.upload_id)));
        let resource = CanonicalizedResource::new(format!(
            "/{}/{}?uploadId={}",
            self.bucket.as_str(),
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

        let header_map = self
            .bucket
            .client
            .authorization_header(&method, resource, headers)?;

        let response = reqwest::Client::new()
            .request(method, url)
            .headers(header_map)
            .body(xml)
            .send()
            .await?;

        let is_success = response.status().is_success();
        let headers = response.headers().clone();
        let body = response.text().await?;

        if !is_success {
            return Err(OssError::from_service(&body));
        }

        let etag = Self::parse_complete_etag(&headers, &body)?;

        self.upload_id = String::new();
        self.etags = Vec::new();
        Ok(etag)
    }

    fn parse_complete_etag(
        headers: &HeaderMap,
        body: &str,
    ) -> Result<String, OssError> {
        if let Some(value) = headers.get("etag") {
            if let Ok(etag) = value.to_str() {
                return Ok(etag.trim_matches('"').to_string());
            }
        }

        if let (Some(start), Some(end)) = (body.find("<ETag>"), body.find("</ETag>")) {
            let etag = &body[start + 6..end];
            return Ok(etag.trim_matches('"').to_string());
        }

        Err(OssError::NoFoundEtag)
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
        let mut url = self.bucket.to_url()?;
        url.set_query(Some(&format!("uploadId={}", self.upload_id)));
        let resource = CanonicalizedResource::new(format!(
            "/{}/{}?uploadId={}",
            self.bucket.as_str(),
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

    /// 取消分片上传（使用 bucket 内嵌的 client）。
    pub async fn abort_multipart(&mut self) -> Result<(), OssError> {
        let client = self.bucket.client.clone();
        self.abort(&client).await
    }
}

impl From<&Object> for PartsUpload {
    fn from(object: &Object) -> Self {
        PartsUpload::new(&object.path, object.bucket.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::client::init_client;

    #[tokio::test]
    async fn test_upload() {
        let res = init_client()
            .bucket("honglei123")
            .unwrap()
            .object("myvideo23.mov")
            .multipart()
            .from_file("./video.mov")
            .upload()
            .await;

        println!("{res:?}");
    }
}
