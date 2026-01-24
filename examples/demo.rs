use aliyun_oss_client::{types::ObjectQuery, Client, EndPoint};
use serde::Deserialize;

fn build_endpoint() -> EndPoint {
    use aliyun_oss_client::types::KnownRegion;
    use aliyun_oss_client::types::Region;
    EndPoint::new(Region::Known(KnownRegion::CnShanghai))
}

async fn run() -> Result<(), aliyun_oss_client::Error> {
    let client = Client::from_env()?;

    let buckets = client.get_buckets(&build_endpoint()).await?;

    let objects = buckets[0].get_objects().await?;

    let _obj_info = objects[0].get_info(&client).await?;

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

    let list: Vec<MyBucket> = client.export_buckets(&build_endpoint()).await?;

    #[derive(Debug, Deserialize)]
    struct MyBucketInfo {
        Name: String,
    }
    let res: MyBucketInfo = buckets[0].export_info(&client).await?;

    let condition = {
        let mut map = ObjectQuery::new();
        map.insert(ObjectQuery::MAX_KEYS, "5");
        map
    };

    #[derive(Debug, Deserialize)]
    struct MyObject {
        Key: String,
    }

    let (list, next_token): (Vec<MyObject>, _) = buckets[0]
        .clone()
        .object_query(condition)
        .export_objects(&client)
        .await?;
    Ok(())
}

pub fn main() {
    // run().unwrap();
}
