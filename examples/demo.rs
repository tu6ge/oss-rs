use aliyun_oss_client::{types::ObjectQuery, Client, EndPoint};

async fn run() -> Result<(), aliyun_oss_client::Error> {
    let client = Client::from_env()?;

    let buckes = client.get_buckets(&EndPoint::CN_QINGDAO).await?;

    let objects = buckes[0].get_objects(&ObjectQuery::new(), &client).await?;

    let obj_info = objects[0].get_info(&client).await?;

    Ok(())
}

pub fn main() {
    // run().unwrap();
}
