use chrono::{DateTime, Utc};
use reqwest::{header::CONTENT_LENGTH, Method};
use url::Url;

use crate::{
    client::Client,
    error::OssError,
    types::{CanonicalizedResource, ObjectQuery},
    Bucket,
};

#[derive(Debug)]
pub struct Objects {
    //bucket: Bucket,
    list: Vec<Object>,
    next_token: Option<String>,
}

impl Objects {
    pub fn new(list: Vec<Object>, next_token: Option<String>) -> Objects {
        Objects { list, next_token }
    }

    pub fn next_token(&self) -> Option<&String> {
        self.next_token.as_ref()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub async fn next_list(
        self,
        query: &ObjectQuery,
        client: &Client,
    ) -> Result<Objects, OssError> {
        let mut q = query.clone();
        if let Some(token) = self.next_token {
            q.insert(ObjectQuery::CONTINUATION_TOKEN, token);
        }
        match client.bucket() {
            Some(bucket) => bucket.get_objects(&q, client).await,
            None => Err(OssError::NoFoundBucket),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Object {
    path: String,
}

impl Object {
    pub fn new<P: Into<String>>(path: P) -> Object {
        Object { path: path.into() }
    }

    /// 确认文件是否在目录里面
    ///
    /// ```rust
    /// # use aliyun_oss_client::Object;
    /// let obj1 = Object::new("foo.txt");
    /// assert!(!obj1.in_dir());
    ///
    /// let obj2 = Object::new("path/foo.txt")
    /// assert!(obj2.in_dir());
    /// ```
    pub fn in_dir(&self) -> bool {
        self.path.find('/').is_some()
    }

    /// 获取文件袋各级目录
    /// ```rust
    /// # use aliyun_oss_client::Object;
    /// let obj1 = Object::new("foo.txt");
    /// let dirs = obj1.get_dirs();
    /// assert!(dirs.len()==0);
    /// let obj2 = Object::new("path1/path2/foo.txt");
    /// let dirs2 = obj2.get_dirs();
    /// assert_eq!(dirs2[0], "path1".to_string());
    /// assert_eq!(dirs2[1], "path2".to_string());
    /// assert!(dirs2.len() ==2);
    /// ```
    pub fn get_dirs(&self) -> Vec<String> {
        let mut dirs: Vec<&str> = self.path.split('/').collect();
        dirs.pop();

        dirs.iter().map(|&d| d.to_owned()).collect()
    }

    /// 根据目录层级，获取绝对路径
    /// ```rust
    /// # use aliyun_oss_client::Object;
    /// let obj1 = Object::new("foo.txt");
    /// let path1 = obj1.absolute_dir_nth(10);
    /// assert!(path1.is_none());
    /// let obj2 = Object::new("path3/path22/bar.txt");
    /// let path21 = obj2.absolute_dir_nth(1);
    /// assert_eq!(path21, Some("path3".to_string()));
    /// let path22 = obj2.absolute_dir_nth(2);
    /// assert_eq!(path22, Some("path3/path22".to_string()));
    /// let path23 = obj2.absolute_dir_nth(3);
    /// assert_eq!(path23, Some("path3/path22".to_string()));
    /// ```
    pub fn absolute_dir_nth(&self, num: usize) -> Option<String> {
        let dirs = self.get_dirs();
        if dirs.len() == 0 {
            return None;
        }
        let n = if num > dirs.len() { dirs.len() } else { num };
        let mut dir = String::new();
        for i in 0..n {
            if i == 0 {
                dir.push_str(&dirs[i]);
            } else {
                dir.push('/');
                dir.push_str(&dirs[i]);
            }
        }

        Some(dir)
    }

    pub fn to_url(&self, bucket: &Bucket) -> Url {
        let mut url = bucket.to_url();
        url.set_path(&self.path);
        url
    }

    pub async fn get_info(&self, client: &Client) -> Result<ObjectInfo, OssError> {
        let bucket = client.bucket().ok_or(OssError::NoFoundBucket)?;
        let mut url = self.to_url(bucket);
        url.set_query(Some("objectMeta"));
        let method = Method::GET;
        let resource =
            CanonicalizedResource::new(format!("/{}/{}?objectMeta", bucket.as_str(), self.path));

        let header_map = client.authorization(method, resource)?;

        let response = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?;

        let headers = response.headers();

        let content_length = headers
            .get(CONTENT_LENGTH)
            .ok_or(OssError::NoFoundContentLength)?;
        let etag = headers.get("etag").ok_or(OssError::NoFoundEtag)?;

        let date = DateTime::parse_from_rfc2822(
            headers
                .get("last-modified")
                .ok_or(OssError::NoFoundLastModified)?
                .to_str()?,
        )?;
        Ok(ObjectInfo {
            last_modified: date.with_timezone(&Utc),
            etag: etag.to_str()?.to_string(),
            size: content_length.to_str()?.parse()?,
        })
    }

    pub async fn upload(&self, content: Vec<u8>, client: &Client) -> Result<(), OssError> {
        let bucket = client.bucket().ok_or(OssError::NoFoundBucket)?;
        let url = self.to_url(bucket);
        let method = Method::PUT;
        let resource = CanonicalizedResource::new(format!("/{}/{}", bucket.as_str(), self.path));

        let header_map = client.authorization(method, resource)?;

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
        let bucket = client.bucket().ok_or(OssError::NoFoundBucket)?;
        let url = self.to_url(bucket);
        let method = Method::GET;
        let resource = CanonicalizedResource::new(format!("/{}/{}", bucket.as_str(), self.path));

        let header_map = client.authorization(method, resource)?;

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
    use crate::{
        bucket::Bucket,
        client::{init_client, Client},
        types::{EndPoint, ObjectQuery},
    };

    fn set_client() -> Client {
        let mut client = init_client();
        client.set_bucket(Bucket::new("honglei123", EndPoint::CN_SHANGHAI));

        client
    }

    #[tokio::test]
    async fn test_object_info() {
        let object = Object::new("app-config.json");

        let info = object.get_info(&set_client()).await.unwrap();

        println!("{info:?}");
    }

    #[tokio::test]
    async fn test_upload() {
        let object = Object::new("abc.txt");

        let info = object.upload("aaa".into(), &set_client()).await.unwrap();

        println!("{info:?}");
    }

    #[tokio::test]
    async fn test_down() {
        let object = Object::new("abc.txt");

        let info = object.download(&set_client()).await.unwrap();

        println!("{:?}", std::str::from_utf8(&info).unwrap());
    }

    #[tokio::test]
    async fn test_next_list() {
        let client = set_client();
        let condition = {
            let mut map = ObjectQuery::new();
            map.insert(ObjectQuery::MAX_KEYS, "5");
            map
        };
        let first_list = client
            .bucket()
            .unwrap()
            .get_objects(&condition, &client)
            .await
            .unwrap();

        let second_list = first_list.next_list(&condition, &client).await.unwrap();

        println!("{:?}", second_list);
    }
}
