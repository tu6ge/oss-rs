use std::str::FromStr;

use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{de::DeserializeOwned, Deserialize};
use serde_xml_rs::from_str;
use url::Url;

use crate::{
    client::Client,
    error::OssError,
    object::{Object, Objects},
    types::{CanonicalizedResource, EndPoint, ObjectQuery, StorageClass},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bucket {
    name: String,
    endpoint: EndPoint,
}

type NextContinuationToken = String;

impl Bucket {
    pub fn new<N: Into<String>>(name: N, endpoint: EndPoint) -> Bucket {
        Bucket {
            name: name.into(),
            endpoint,
        }
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
    /// ```
    pub fn to_url(&self) -> Url {
        const HTTPS: &str = "https://";
        let url = self.endpoint.to_url().to_string();
        let name_str = self.name.to_string();

        let mut name = String::from(HTTPS);
        name.push_str(&name_str);
        name.push('.');

        let url = url.replace(HTTPS, &name);

        Url::parse(&url).unwrap()
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
    /// 该方法返回的类型可以是如下结构体：
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

        let mut url = self.to_url();
        url.set_query(Some(BUCKET_INFO));
        let method = Method::GET;
        let resource = CanonicalizedResource::from_bucket_info(self);

        let header_map = client.authorization(method, resource)?;

        let content = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?
            .text()
            .await?;

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

        let mut url = self.to_url();
        url.set_query(Some(BUCKET_INFO));
        let method = Method::GET;
        let resource = CanonicalizedResource::from_bucket_info(self);

        let header_map = client.authorization(method, resource)?;

        let content = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?
            .text()
            .await?;

        println!("{content}");
        Self::parse_info_xml(content)
    }

    fn parse_info_xml(xml: String) -> Result<BucketInfo, OssError> {
        let creation_date = match Self::parse_item(&xml, "CreationDate") {
            Some(d) => d,
            None => return Err(OssError::NoFoundCreationDate),
        };
        let creation_date = creation_date.parse()?;
        let storage_class = match Self::parse_item(&xml, "StorageClass") {
            Some(s) => s,
            None => return Err(OssError::NoFoundStorageClass),
        };
        let storage_class = match StorageClass::new(storage_class) {
            Some(s) => s,
            None => return Err(OssError::NoFoundStorageClass),
        };
        let data_redundancy_type = match Self::parse_item(&xml, "DataRedundancyType") {
            Some(s) => s,
            None => return Err(OssError::NoFoundDataRedundancyType),
        };
        let data_redundancy_type = match DataRedundancyType::from_str(data_redundancy_type) {
            Ok(d) => d,
            Err(_) => return Err(OssError::NoFoundDataRedundancyType),
        };

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
    /// 该方法返回的类型可以是如下结构体：
    /// ```rust
    /// use serde::Deserialize;
    /// #[derive(Debug, Deserialize)]
    /// struct MyObject {
    ///     Key: String,
    /// }
    /// ```
    pub async fn export_objects<Obj: DeserializeOwned>(
        &self,
        query: &ObjectQuery,
        client: &Client,
    ) -> Result<(Vec<Obj>, NextContinuationToken), OssError> {
        let mut url = self.to_url();
        url.set_query(Some(&query.to_oss_query()));
        let method = Method::GET;
        let resource = CanonicalizedResource::from_object_list(self, query.get_next_token());

        let header_map = client.authorization(method, resource)?;

        let content = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?
            .text()
            .await?;

        //println!("{content}");

        #[derive(Debug, Deserialize)]
        struct ListBucketResult<T> {
            #[serde(rename = "Contents")]
            contents: Vec<T>,
            #[serde(rename = "NextContinuationToken")]
            next_token: String,
        }
        let res: ListBucketResult<Obj> = from_str(&content)?;

        Ok((res.contents, res.next_token))
    }

    pub async fn get_objects(
        &self,
        query: &ObjectQuery,
        client: &Client,
    ) -> Result<Objects, OssError> {
        let mut url = self.to_url();
        url.set_query(Some(&query.to_oss_query()));
        let method = Method::GET;
        let resource = CanonicalizedResource::from_object_list(self, query.get_next_token());

        let header_map = client.authorization(method, resource)?;

        let content = reqwest::Client::new()
            .get(url)
            .headers(header_map)
            .send()
            .await?
            .text()
            .await?;

        //println!("{content}");

        let list = Self::parse_xml_objects(&content)?;

        let token = Self::parse_item(&content, "NextContinuationToken").map(|t| t.to_owned());

        Ok(Objects::new(list, token))
    }

    pub(crate) fn parse_xml_objects(xml: &str) -> Result<Vec<Object>, OssError> {
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
        for i in 0..start_positions.len() {
            let path = &xml[start_positions[i] + 5..end_positions[i]];
            list.push(Object::new(path.to_owned()))
        }

        Ok(list)
    }
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
    use serde::Deserialize;

    use crate::{
        client::init_client,
        types::{EndPoint, ObjectQuery},
    };

    use super::Bucket;

    #[tokio::test]
    async fn test_get_info() {
        let bucket = Bucket::new("honglei123", EndPoint::CN_SHANGHAI);
        let info = bucket.get_info(&init_client()).await.unwrap();

        //assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_export_info() {
        let bucket = Bucket::new("honglei123", EndPoint::CN_SHANGHAI);

        #[derive(Debug, Deserialize)]
        struct DemoData {
            Name: String,
        }
        let res: DemoData = bucket.export_info(&init_client()).await.unwrap();

        println!("{:?}", res);
    }

    #[tokio::test]
    async fn test_export_objects() {
        let bucket = Bucket::new("honglei123", EndPoint::CN_SHANGHAI);
        let condition = {
            let mut map = ObjectQuery::new();
            map.insert(ObjectQuery::MAX_KEYS, "5");
            map
        };

        #[derive(Debug, Deserialize)]
        struct MyObject {
            Key: String,
        }

        let (list, token): (Vec<MyObject>, String) = bucket
            .export_objects(&condition, &init_client())
            .await
            .unwrap();

        println!("{list:?}, token:{token}");
    }

    #[tokio::test]
    async fn test_get_objects() {
        let bucket = Bucket::new("honglei123", EndPoint::CN_SHANGHAI);
        let mut condition = {
            let mut map = ObjectQuery::new();
            map.insert(ObjectQuery::MAX_KEYS, "5");
            map
        };

        let list = bucket
            .get_objects(&condition, &init_client())
            .await
            .unwrap();

        println!("{list:?}");
        condition.insert_next_token(list.next_token().unwrap().to_owned());
        let second_list2 = bucket
            .get_objects(&condition, &init_client())
            .await
            .unwrap();
        println!("second_list: {:?}", second_list2);
        // let second_list = list.next_list(&condition, &init_client()).await.unwrap();
        // println!("second_list: {:?}", second_list);
    }
}
