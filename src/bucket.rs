use std::{str::FromStr, sync::Arc};

use async_stream::try_stream;
use chrono::{DateTime, Utc};
use futures_core::{stream::BoxStream, Stream};
use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize};
use serde_xml_rs::from_str;
use url::Url;

use crate::{
    client::Client,
    error::{BucketNameError, OssError},
    object::{Object, Objects},
    types::{CanonicalizedResource, EndPointUrl, ObjectQuery, StorageClass},
};

#[derive(Debug, Clone)]
pub struct Bucket {
    name: String,
    pub(crate) client: Arc<Client>,
    query: ObjectQuery,
}

impl PartialEq<Bucket> for Bucket {
    fn eq(&self, other: &Bucket) -> bool {
        self.name.eq(&other.name)
    }
}
impl Eq for Bucket {}

type NextContinuationToken = Option<String>;

impl Bucket {
    pub fn new<N: Into<String>>(name: N, client: Arc<Client>) -> Result<Bucket, OssError> {
        let name = name.into();
        validate_bucket_name(&name)?;

        Ok(Bucket {
            name,
            client,
            query: ObjectQuery::new(),
        })
    }

    pub fn from_env() -> Result<Bucket, OssError> {
        let name = std::env::var("ALIYUN_BUCKET").map_err(|_| OssError::InvalidBucket)?;

        let client = Arc::new(Client::from_env()?);

        Ok(Bucket {
            name,
            client,
            query: ObjectQuery::new(),
        })
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.name
    }

    /// # 返回 bucket 对应的链接地址
    /// 可以是内网地址，默认为外网地址
    /// ```
    /// # use aliyun_oss_client::{Bucket, EndPoint};
    /// # use url::Url;
    /// let bucket = Bucket::new("foo", EndPoint::CN_QINGDAO);
    /// assert_eq!(bucket.to_url(), Url::parse("https://foo.oss-cn-qingdao.aliyuncs.com").unwrap());
    ///
    /// let mut endpoint_internal = EndPoint::CN_QINGDAO;
    /// endpoint_internal.set_internal(true);
    /// let bucket_internal = Bucket::new("bar", endpoint_internal);
    /// assert_eq!(bucket_internal.to_url(), Url::parse("https://bar.oss-cn-qingdao-internal.aliyuncs.com").unwrap());
    /// ```
    pub fn to_url(&self) -> Result<Url, OssError> {
        let url = format!(
            "https://{}.{}",
            self.name.as_str(),
            EndPointUrl::new(&self.client.endpoint)?.host()
        );

        Url::parse(&url).map_err(|_| OssError::InvalidBucketUrl)
    }

    pub fn max_keys(mut self, max_keys: u32) -> Self {
        self.query = self.query.max_keys(max_keys);
        self
    }
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.query = self.query.prefix(prefix);
        self
    }
    pub fn delimiter(mut self, delimiter: &str) -> Self {
        self.query = self.query.delimiter(delimiter);
        self
    }
    pub fn continuation_token(mut self, continuation_token: &str) -> Self {
        self.query = self.query.continuation_token(continuation_token);
        self
    }
    pub fn encoding_type(mut self, encoding_type: &str) -> Self {
        self.query = self.query.encoding_type(encoding_type);
        self
    }
    pub fn start_after(mut self, start_after: &str) -> Self {
        self.query = self.query.start_after(start_after);
        self
    }
    pub fn fetch_owner(mut self, fetch_owner: bool) -> Self {
        self.query = self.query.fetch_owner(fetch_owner);
        self
    }

    pub fn object(&self, path: &str) -> Object {
        Object::new(path, Arc::new(self.clone()))
    }

    /// 调用 api 导出 bucket 详情信息到自定义类型
    ///
    /// aliyun api 返回的 xml 是如下格式：
    /// ```xml
    /// <Bucket>
    ///   <AccessMonitor>Disabled</AccessMonitor>
    ///   <BlockPublicAccess>false</BlockPublicAccess>
    ///   <Comment></Comment>
    ///   <CreationDate>2016-11-05T13:10:10.000Z</CreationDate>
    ///   <CrossRegionReplication>Disabled</CrossRegionReplication>
    ///   <DataRedundancyType>LRS</DataRedundancyType>
    ///   <ExtranetEndpoint>oss-cn-shanghai.aliyuncs.com</ExtranetEndpoint>
    ///   <IntranetEndpoint>oss-cn-shanghai-internal.aliyuncs.com</IntranetEndpoint>
    ///   <Location>oss-cn-shanghai</Location>
    ///   <Name>honglei123</Name>
    ///   <ResourceGroupId>rg-acfmoiyerp5judy</ResourceGroupId>
    ///   <StorageClass>Standard</StorageClass>
    ///   <TransferAcceleration>Disabled</TransferAcceleration>
    ///   <Owner>
    ///     <DisplayName>34773519</DisplayName>
    ///     <ID>34773519</ID>
    ///   </Owner>
    ///   <AccessControlList>
    ///     <Grant>public-read</Grant>
    ///   </AccessControlList>
    ///   <ServerSideEncryptionRule>
    ///     <SSEAlgorithm>None</SSEAlgorithm>
    ///   </ServerSideEncryptionRule>
    ///   <BucketPolicy>
    ///     <LogBucket>honglei123</LogBucket>
    ///     <LogPrefix>oss-accesslog/</LogPrefix>
    ///   </BucketPolicy>
    /// </Bucket>
    /// ```
    /// 该方法返回的类型是自定义的，根据不同的业务需要，导出不同的字段，例如导出的类型可以是如下结构体：
    ///
    /// ```rust
    /// use serde::Deserialize;
    /// #[derive(Debug, Deserialize)]
    /// struct DemoData {
    ///     Name: String,
    /// }
    /// ```

    pub async fn export_info<B: DeserializeOwned>(&self, client: &Client) -> Result<B, OssError> {
        const BUCKET_INFO: &str = "bucketInfo";

        let mut url = self.to_url()?;
        url.set_query(Some(BUCKET_INFO));
        let method = Method::GET;
        let resource = CanonicalizedResource::from_bucket_info(self);

        let header_map = client.authorization(&method, resource)?;

        let response = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?;

        let is_success = response.status().is_success();

        let content = response.text().await?;

        if !is_success {
            return Err(OssError::from_service(&content));
        }

        //println!("{}", content);

        #[derive(Debug, Deserialize)]
        struct BucketInfo<T> {
            #[serde(rename = "Bucket")]
            bucket: T,
        }
        let res: BucketInfo<B> = from_str(&content)?;

        Ok(res.bucket)
    }

    pub async fn get_info(&self, client: &Client) -> Result<BucketInfo, OssError> {
        const BUCKET_INFO: &str = "bucketInfo";

        let mut url = self.to_url()?;
        url.set_query(Some(BUCKET_INFO));
        let method = Method::GET;
        let resource = CanonicalizedResource::from_bucket_info(self);

        let header_map = client.authorization(&method, resource)?;

        let response = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?;

        let is_success = response.status().is_success();

        let content = response.text().await?;

        if !is_success {
            return Err(OssError::from_service(&content));
        }

        //println!("{content}");
        Self::parse_info_xml(content)
    }

    fn parse_info_xml(xml: String) -> Result<BucketInfo, OssError> {
        let creation_date = Self::parse_item(&xml, "CreationDate")
            .ok_or(OssError::NoFoundCreationDate)?
            .parse()?;
        let storage_class = StorageClass::new(
            Self::parse_item(&xml, "StorageClass").ok_or(OssError::NoFoundStorageClass)?,
        )
        .ok_or(OssError::NoFoundStorageClass)?;
        let data_redundancy_type = Self::parse_item(&xml, "DataRedundancyType")
            .ok_or(OssError::NoFoundDataRedundancyType)?;
        let data_redundancy_type = DataRedundancyType::from_str(data_redundancy_type)
            .map_err(|_| OssError::NoFoundDataRedundancyType)?;

        Ok(BucketInfo {
            creation_date,
            storage_class,
            data_redundancy_type,
        })
    }

    pub(crate) fn parse_item<'a>(xml: &'a str, field: &str) -> Option<&'a str> {
        let start_tag = {
            let mut s = String::from("<");
            s += field;
            s += ">";
            s
        };
        let start_index = xml.find(&start_tag);

        let end_tag = {
            let mut s = String::from("</");
            s += field;
            s += ">";
            s
        };
        let end_index = xml.find(&end_tag);

        match (start_index, end_index) {
            (Some(start), Some(end)) => {
                let s = &xml[start + field.len() + 2..end];
                Some(s)
            }
            _ => None,
        }
    }

    pub fn object_query(mut self, query: ObjectQuery) -> Self {
        self.query = query;
        self
    }

    pub fn export_objects_stream<Obj>(self) -> BoxStream<'static, Result<Obj, OssError>>
    where
        Obj: DeserializeOwned + Send + 'static,
    {
        Box::pin(self.export_objects_stream_impl::<Obj>())
    }

    pub fn export_objects_stream_impl<Obj>(mut self) -> impl Stream<Item = Result<Obj, OssError>>
    where
        Obj: DeserializeOwned,
    {
        try_stream! {
            let mut token: Option<String> = None;

            loop {
                // 设置 continuation token
                if let Some(ref t) = token {
                    self.query.insert(ObjectQuery::CONTINUATION_TOKEN, t);
                }

                let (objects, next) = self.export_objects::<Obj>().await?;

                for obj in objects {
                    yield obj;
                }

                match next {
                    Some(t) => token = Some(t),
                    None => break,
                }
            }
        }
    }

    /// 调用 aliyun api 返回 object 列表到自定义类型，它还会返回用于翻页的 `NextContinuationToken`
    ///
    /// aliyun api 返回的 xml 是如下格式：
    /// ```xml
    /// <Contents>
    ///   <Key>9AB932LY.jpeg</Key>
    ///   <LastModified>2022-06-26T09:53:21.000Z</LastModified>
    ///   <ETag>"F75A15996D0857B16FA31A3B16624C26"</ETag>
    ///   <Type>Normal</Type>
    ///   <Size>18027</Size>
    ///   <StorageClass>Standard</StorageClass>
    /// </Contents>
    /// ```
    /// 该方法返回的类型是自定义的，根据不同的业务需要，导出不同的字段，例如导出的类型可以是如下结构体：
    /// ```rust
    /// use serde::Deserialize;
    /// #[derive(Debug, Deserialize)]
    /// struct MyObject {
    ///     Key: String,
    /// }
    /// ```
    pub async fn export_objects<Obj: DeserializeOwned>(
        &self,
    ) -> Result<(Vec<Obj>, NextContinuationToken), OssError> {
        let mut url = self.to_url()?;
        url.set_query(Some(&self.query.to_oss_query()));
        let method = Method::GET;
        let resource = CanonicalizedResource::from_object_list(self, self.query.get_next_token());

        let header_map = self.client.authorization(&method, resource)?;

        let response = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?;

        let is_success = response.status().is_success();

        let content = response.text().await?;

        if !is_success {
            return Err(OssError::from_service(&content));
        }

        //println!("{content}");

        #[derive(Debug, Deserialize)]
        struct ListBucketResult<T> {
            #[serde(rename = "Contents")]
            contents: Vec<T>,
            #[serde(rename = "NextContinuationToken")]
            next_token: Option<String>,
        }
        let res: ListBucketResult<Obj> = from_str(&content)?;

        Ok((res.contents, res.next_token))
    }

    pub fn objects_into_stream(self) -> BoxStream<'static, Result<Object, OssError>> {
        Box::pin(self.objects_into_stream_impl())
    }

    fn objects_into_stream_impl(mut self) -> impl Stream<Item = Result<Object, OssError>> {
        try_stream! {
            let mut marker: Option<String> = None;

            loop {
               if let Some(token) = marker {
                    self.query.insert(ObjectQuery::CONTINUATION_TOKEN, &token);
                }
                let resp = self.get_objects().await?;

                for obj in resp.list {
                    yield obj;
                }

                match resp.next_token{
                    Some(token)=> {
                        marker = Some(token);
                    },
                    None => {
                        break;
                    }
                }

            }
        }
    }

    pub async fn get_objects(&self) -> Result<Objects, OssError> {
        let mut url = self.to_url()?;
        url.set_query(Some(&self.query.to_oss_query()));
        let method = Method::GET;
        let resource = CanonicalizedResource::from_object_list(self, self.query.get_next_token());

        let header_map = self.client.authorization(&method, resource)?;

        let response = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?;

        let is_success = response.status().is_success();

        let content = response.text().await?;

        if !is_success {
            return Err(OssError::from_service(&content));
        }

        //println!("{content}");

        let list = self.parse_xml_objects(&content)?;

        let token = Self::parse_item(&content, "NextContinuationToken").map(|t| t.to_owned());

        Ok(Objects::new(list, token))
    }

    pub(crate) fn parse_xml_objects(&self, xml: &str) -> Result<Vec<Object>, OssError> {
        let mut start_positions = vec![];
        let mut end_positions = vec![];
        let mut start = 0;
        let mut pattern = "<Key>";

        while let Some(pos) = xml[start..].find(pattern) {
            start_positions.push(start + pos);
            start += pos + pattern.len();
        }
        start = 0;
        pattern = "</Key>";
        while let Some(pos) = xml[start..].find(pattern) {
            end_positions.push(start + pos);
            start += pos + pattern.len();
        }

        let mut list = vec![];
        let arc_bucket = Arc::new(self.clone());
        for i in 0..start_positions.len() {
            let path = &xml[start_positions[i] + 5..end_positions[i]];
            list.push(Object::new(path, arc_bucket.clone()))
        }

        Ok(list)
    }
}

pub fn validate_bucket_name(name: &str) -> Result<(), BucketNameError> {
    let len = name.len();
    if len < 3 || len > 63 {
        return Err(BucketNameError::InvalidLength);
    }

    let mut chars = name.chars();

    let first = chars.next().unwrap();
    if !is_lowercase_letter_or_digit(first) {
        return Err(BucketNameError::InvalidStart);
    }

    let last = name.chars().last().unwrap();
    if !is_lowercase_letter_or_digit(last) {
        return Err(BucketNameError::InvalidEnd);
    }

    for c in name.chars() {
        if !(is_lowercase_letter_or_digit(c) || c == '-') {
            return Err(BucketNameError::InvalidCharacter(c));
        }
    }

    if looks_like_ip(name) {
        return Err(BucketNameError::LooksLikeIpAddress);
    }

    Ok(())
}

fn is_lowercase_letter_or_digit(c: char) -> bool {
    matches!(c, 'a'..='z' | '0'..='9')
}

fn looks_like_ip(name: &str) -> bool {
    let parts: Vec<&str> = name.split('.').collect();
    if parts.len() != 4 {
        return false;
    }

    parts.iter().all(|p| {
        !p.is_empty()
            && p.len() <= 3
            && p.chars().all(|c| c.is_ascii_digit())
            && p.parse::<u8>().is_ok()
    })
}

#[derive(Debug)]
pub struct BucketInfo {
    //base: Bucket,
    creation_date: DateTime<Utc>,
    storage_class: StorageClass,
    data_redundancy_type: DataRedundancyType,
}

impl BucketInfo {
    pub fn new(
        creation_date: DateTime<Utc>,
        storage_class: StorageClass,
        data_redundancy_type: DataRedundancyType,
    ) -> Self {
        BucketInfo {
            creation_date,
            storage_class,
            data_redundancy_type,
        }
    }

    pub fn creation_date(&self) -> &DateTime<Utc> {
        &self.creation_date
    }
    pub fn storage_class(&self) -> &StorageClass {
        &self.storage_class
    }
    pub fn data_redundancy_type(&self) -> &DataRedundancyType {
        &self.data_redundancy_type
    }
}

#[derive(Default, Debug)]
pub enum Grant {
    #[default]
    Private,
    PublicRead,
    PublicReadWrite,
}

#[derive(Clone, Debug, Default)]
pub enum DataRedundancyType {
    #[default]
    LRS,
    ZRS,
}

impl FromStr for DataRedundancyType {
    type Err = OssError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LRS" => Ok(DataRedundancyType::LRS),
            "ZRS" => Ok(DataRedundancyType::ZRS),
            _ => Err(OssError::NoFoundDataRedundancyType),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures_util::StreamExt;
    use serde::Deserialize;

    use crate::{client::init_client, types::ObjectQuery};

    use super::Bucket;

    fn build_bucket() -> Bucket {
        Bucket::new("honglei123", Arc::new(init_client())).unwrap()
    }

    #[tokio::test]
    async fn test_get_info() {
        let bucket = build_bucket();
        let info = bucket.get_info(&init_client()).await.unwrap();

        println!("{info:?}");
        //assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_export_info() {
        let bucket = build_bucket();
        #[derive(Debug, Deserialize)]
        struct DemoData {
            Name: String,
        }
        let res: DemoData = bucket.export_info(&init_client()).await.unwrap();

        println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_export_objects() {
        use dotenv::dotenv;

        dotenv().ok();

        #[derive(Debug, Deserialize)]
        struct MyObject {
            Key: String,
        }

        let mut stream = Bucket::from_env()
            .unwrap()
            .max_keys(5)
            .export_objects_stream::<MyObject>();

        let mut i = 0;
        while let Some(item) = stream.next().await {
            println!("{item:?}");

            i = i + 1;
            if i > 7 {
                break;
            }
        }
    }

    #[tokio::test]
    async fn test_get_objects() {
        use futures_util::StreamExt;

        let client = init_client();
        let mut stream = client
            .bucket("honglei123")
            .unwrap()
            .max_keys(5)
            .objects_into_stream();

        let mut i = 0;
        while let Some(item) = stream.next().await {
            println!("{item:?}");
            i = i + 1;
            if i > 7 {
                break;
            }
        }
    }
}
