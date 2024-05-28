use std::str::FromStr;

use chrono::{DateTime, Utc};
use reqwest::Method;
use url::Url;

use crate::{
    client::Client,
    error::OssError,
    object::{Object, Objects},
    types::{CanonicalizedResource, EndPoint, ObjectQuery, StorageClass},
};

#[derive(Debug, Clone)]
pub struct Bucket {
    name: String,
    endpoint: EndPoint,
}

impl Bucket {
    pub fn new(name: String, endpoint: EndPoint) -> Bucket {
        Bucket { name, endpoint }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.name
    }

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

        //println!("{content}");
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

    pub async fn get_objects(
        &self,
        query: &ObjectQuery,
        client: &Client,
    ) -> Result<Objects, OssError> {
        let mut url = self.to_url();
        url.set_query(Some(&query.to_oss_query()));
        let method = Method::GET;
        let resource = CanonicalizedResource::from_object_list(&self, query.get_next_token());

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
    use crate::{
        client::initClient,
        types::{EndPoint, ObjectQuery},
    };

    use super::Bucket;

    #[tokio::test]
    async fn test_get_info() {
        let bucket = Bucket::new("honglei123".into(), EndPoint::CN_SHANGHAI);
        let info = bucket.get_info(&initClient()).await.unwrap();

        //assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_get_objects() {
        let bucket = Bucket::new("honglei123".into(), EndPoint::CN_SHANGHAI);
        let mut condition = {
            let mut map = ObjectQuery::new();
            map.insert("max-keys", "5");
            map
        };

        let list = bucket.get_objects(&condition, &initClient()).await.unwrap();

        println!("{list:?}");
        condition.set_next_token(&list);
        let second_list2 = bucket.get_objects(&condition, &initClient()).await.unwrap();
        println!("second_list: {:?}", second_list2);
        // let second_list = list.next_list(&condition, &initClient()).await.unwrap();
        // println!("second_list: {:?}", second_list);
    }
}
