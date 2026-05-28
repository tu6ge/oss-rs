use aliyun_oss_client::{types::ObjectQuery, Client, EndPoint};
use serde::Deserialize;

#[allow(unused)]
async fn run() -> Result<(), aliyun_oss_client::Error> {
    let client = Client::from_env()?;

    let buckets = client.get_buckets().await?;

    let objects = buckets[0].get_objects().await?;

    let _obj_info = objects[0].get_info().await?;

    #[derive(Debug, Deserialize)]
    struct MyBucket {
        #[serde(rename = "Comment")]
        #[allow(unused)]
        comment: String,

        #[serde(rename = "CreationDate")]
        #[allow(unused)]
        creation_date: String,

        #[serde(rename = "ExtranetEndpoint")]
        #[allow(unused)]
        extranet_endpoint: EndPoint,

        #[serde(rename = "IntranetEndpoint")]
        #[allow(unused)]
        intranet_endpoint: String,

        #[serde(rename = "Location")]
        #[allow(unused)]
        location: String,

        #[serde(rename = "Name")]
        #[allow(unused)]
        name: String,

        #[serde(rename = "Region")]
        #[allow(unused)]
        region: String,

        #[serde(rename = "StorageClass")]
        #[allow(unused)]
        storage_class: String,
    }

    let _list: Vec<MyBucket> = client.export_buckets().await?;

    #[derive(Debug, Deserialize)]
    struct MyBucketInfo {
        #[serde(rename = "Name")]
        #[allow(unused)]
        name: String,
    }
    let _res: MyBucketInfo = buckets[0].export_info(&client).await?;

    let condition = {
        let mut map = ObjectQuery::new();
        map.insert(ObjectQuery::MAX_KEYS, "5");
        map
    };

    #[derive(Debug, Deserialize)]
    struct MyObject {
        #[serde(rename = "Key")]
        #[allow(unused)]
        key: String,
    }

    let (_list, _next_token): (Vec<MyObject>, _) = buckets[0]
        .clone()
        .object_query(condition)
        .export_objects()
        .await?;
    Ok(())
}

pub fn main() {
    // run().unwrap();
}
