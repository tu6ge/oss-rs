use aliyun_oss_client::{sts::STS, BucketName, Client, EndPoint};

#[tokio::main]
async fn main() {
    let client = Client::new_with_sts(
        "STS.xxxxxxxx".into(),
        "EVd6dXew6xxxxxxxxxxxxxxxxxxxxxxxxxxx".into(),
        EndPoint::CnShanghai,
        BucketName::new("yyyyyy").unwrap(),
        "CAIS4gF1q6Ft5Bxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_string(),
    )
    .unwrap();

    let builder = client.get_bucket_list().await.unwrap();
    println!("{:?}", builder);
}
