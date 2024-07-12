use std::env::VarError;

use chrono::Utc;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Method,
};
use serde::{de::DeserializeOwned, Deserialize};
use serde_xml_rs::from_str;

use crate::{
    bucket::Bucket,
    error::OssError,
    types::{CanonicalizedResource, EndPoint, Key, Secret},
};

/// 存放 key, secret 以及默认 bucket 信息，几乎每个 api 都会用到它的引用
#[derive(Debug, Clone)]
pub struct Client {
    key: Key,
    secret: Secret,
    bucket: Option<Bucket>,
    security_token: Option<String>,
}

impl Client {
    pub fn new(key: Key, secret: Secret) -> Client {
        Self {
            key,
            secret,
            bucket: None,
            security_token: None,
        }
    }

    pub fn from_env() -> Result<Self, VarError> {
        let key = Key::from_env()?;
        let secret = Secret::from_env()?;
        let bucket = Bucket::from_env().ok();

        Ok(Client {
            key,
            secret,
            bucket,
            security_token: None,
        })
    }

    pub fn new_with_sts(key: Key, secret: Secret, security_token: String) -> Self {
        Self {
            key,
            secret,
            bucket: None,
            security_token: Some(security_token),
        }
    }

    /// 设置默认的 bucket(bucket 也会包含 endpoint 信息)
    /// 当设置的时候，会返回上次设置的值，默认值为 None
    /// ```
    /// # use aliyun_oss_client::{Client, Key, Secret, EndPoint, Bucket};
    /// # let mut client = Client::new(Key::new("foo"), Secret::new("bar"));
    /// # let bucket = Bucket::new("bucket1", EndPoint::CN_QINGDAO);
    /// # let bucket2 = Bucket::new("bucket2", EndPoint::CN_QINGDAO);
    /// assert!(client.bucket().is_none());
    /// let res = client.set_bucket(bucket.clone());
    /// assert!(res.is_none());
    /// assert_eq!(client.bucket(), Some(&bucket));
    ///
    /// let res2 = client.set_bucket(bucket2.clone());
    /// assert_eq!(res2, Some(bucket));
    /// assert_eq!(client.bucket(), Some(&bucket2));
    /// ```
    pub fn set_bucket(&mut self, bucket: Bucket) -> Option<Bucket> {
        self.bucket.replace(bucket)
    }

    /// 返回当前设置的 bucket 信息
    pub fn bucket(&self) -> Option<&Bucket> {
        self.bucket.as_ref()
    }

    pub fn authorization(
        &self,
        method: &Method,
        resource: CanonicalizedResource,
    ) -> Result<HeaderMap, OssError> {
        self.authorization_header(method, resource, HeaderMap::new())
    }

    pub fn authorization_header(
        &self,
        method: &Method,
        resource: CanonicalizedResource,
        mut headers: HeaderMap,
    ) -> Result<HeaderMap, OssError> {
        const LINE_BREAK: &str = "\n";

        let date = now();
        let content_type = "text/xml";

        if let Some(sts_token) = &self.security_token {
            headers.insert("x-oss-security-token", {
                let mut token: HeaderValue = sts_token.try_into()?;
                token.set_sensitive(true);
                token
            });
        }

        let oss_header_str = to_oss_header(&headers);

        let sign = {
            let mut string = method.as_str().to_owned();
            string += LINE_BREAK;
            string += LINE_BREAK;
            string += content_type;
            string += LINE_BREAK;
            string += date.as_str();
            string += LINE_BREAK;
            string += &oss_header_str;
            string += resource.as_str();

            let encry = self.secret.encryption(string.as_bytes()).unwrap();

            format!("OSS {}:{}", self.key.as_str(), encry)
        };
        let header_map = {
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

    /// 调用 api 导出 bucket 列表信息到自定义类型
    ///
    /// aliyun api 返回的 xml 是如下格式：
    /// ```xml
    /// <Bucket>
    ///  <Comment></Comment>
    ///  <CreationDate>2020-09-13T03:14:54.000Z</CreationDate>
    ///  <ExtranetEndpoint>oss-cn-shanghai.aliyuncs.com</ExtranetEndpoint>
    ///  <IntranetEndpoint>oss-cn-shanghai-internal.aliyuncs.com</IntranetEndpoint>
    ///  <Location>oss-cn-shanghai</Location>
    ///  <Name>aliyun-wb-kpbf3</Name>
    ///  <Region>cn-shanghai</Region>
    ///  <StorageClass>Standard</StorageClass>
    /// </Bucket>
    /// ```
    /// 该方法返回的类型可以是如下结构体：
    /// ```rust
    /// use serde::Deserialize;
    /// #[derive(Debug, Deserialize)]
    /// struct MyBucket {
    ///     Comment: String,
    ///     CreationDate: String,
    ///     ExtranetEndpoint: String,
    ///     IntranetEndpoint: String,
    ///     Location: String,
    ///     Name: String,
    ///     Region: String,
    ///     StorageClass: String,
    /// }
    /// // 或者根据不同的业务需要，导出不同的字段
    /// #[derive(Debug, Deserialize)]
    /// struct MyBucket2 {
    ///     Location: String,
    ///     Name: String,
    /// }
    /// ```
    pub async fn export_buckets<B: DeserializeOwned>(
        &self,
        endpoint: &EndPoint,
    ) -> Result<Vec<B>, OssError> {
        let url = endpoint.to_url();
        let method = Method::GET;
        let resource = CanonicalizedResource::default();

        let header_map = self.authorization(&method, resource)?;

        let response = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?;

        let is_success = response.status().is_success();

        let content = response.text().await?;

        if !is_success {
            return Err(OssError::Service(content));
        }

        //println!("{}", content);

        #[derive(Debug, Deserialize)]
        struct ListAllMyBucketsResult<T> {
            #[serde(rename = "Buckets")]
            buckets: Buckets<T>,
        }

        #[derive(Debug, Deserialize)]
        struct Buckets<T> {
            #[serde(rename = "Bucket")]
            bucket: Vec<T>,
        }

        let xml_res: ListAllMyBucketsResult<B> = from_str(&content)?;

        Ok(xml_res.buckets.bucket)
    }

    pub async fn get_buckets(&self, endpoint: &EndPoint) -> Result<Vec<Bucket>, OssError> {
        let url = endpoint.to_url();
        let method = Method::GET;
        let resource = CanonicalizedResource::default();

        let header_map = self.authorization(&method, resource)?;

        let response = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?;

        let is_success = response.status().is_success();
        let content = response.text().await?;
        if !is_success {
            return Err(OssError::Service(content));
        }

        // println!("{content}");

        Self::parse_xml(content, endpoint)
    }

    fn parse_xml(xml: String, endpoint: &EndPoint) -> Result<Vec<Bucket>, OssError> {
        let mut start_positions = vec![];
        let mut end_positions = vec![];
        let mut start = 0;
        let mut pattern = "<Name>";
        let pattern_len = pattern.len();

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

        debug_assert!(start_positions.len() == end_positions.len());

        let mut bucket = vec![];
        for i in 0..start_positions.len() {
            let name = &xml[start_positions[i] + pattern_len..end_positions[i]];
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

fn to_oss_header(headers: &HeaderMap) -> String {
    const X_OSS_PRE: &str = "x-oss-";
    const LINE_BREAK: &str = "\n";
    //return Some("x-oss-copy-source:/honglei123/file1.txt");
    let mut header: Vec<_> = headers
        .iter()
        .filter(|(k, _v)| k.as_str().starts_with(X_OSS_PRE))
        .collect();
    if header.is_empty() {
        return String::new();
    }

    header.sort_by(|(k1, _), (k2, _)| k1.as_str().cmp(k2.as_str()));

    let header_vec: Vec<_> = header
        .iter()
        .filter_map(|(k, v)| {
            v.to_str()
                .ok()
                .map(|value| k.as_str().to_owned() + ":" + value)
        })
        .collect();

    let mut str = header_vec.join(LINE_BREAK);
    str += LINE_BREAK;
    str
}

#[cfg(test)]
mod tests {
    use crate::{client::init_client, types::EndPoint};

    #[tokio::test]
    async fn test_get_buckets() {
        let list = init_client()
            .get_buckets(&EndPoint::CN_QINGDAO)
            .await
            .unwrap();

        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn parse_xml() {
        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        struct MyBucket {
            Comment: String,
            CreationDate: String,
            ExtranetEndpoint: EndPoint,
            IntranetEndpoint: String,
            Location: String,
            Name: String,
            Region: String,
            StorageClass: String,
        }

        let list: Vec<MyBucket> = init_client()
            .export_buckets(&EndPoint::CN_QINGDAO)
            .await
            .unwrap();

        println!("{list:?}");
    }
}

#[cfg(test)]
pub fn init_client() -> Client {
    use std::env;

    use dotenv::dotenv;

    dotenv().ok();
    let key = env::var("ALIYUN_KEY_ID").unwrap();
    let secret = env::var("ALIYUN_KEY_SECRET").unwrap();

    Client::new(Key::new(key), Secret::new(secret))
    //Client::new_with_sts(Key::new("STS."), Secret::new(""), "".to_string())
}
