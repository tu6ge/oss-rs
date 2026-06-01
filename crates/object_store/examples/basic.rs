use aliyun_oss_object_store::AliyunOssObjectStore;
use bytes::Bytes;
use object_store::{path::Path, ObjectStoreExt as _, PutPayload};

#[tokio::main]
async fn main() -> object_store::Result<()> {
    let store = AliyunOssObjectStore::try_from_env("my-bucket")?;

    let path = Path::from("data/hello.txt");
    store
        .put(
            &path,
            PutPayload::from_bytes(Bytes::from_static(b"hello, oss")),
        )
        .await?;

    let meta = store.head(&path).await?;
    println!("size = {}", meta.size);

    Ok(())
}
