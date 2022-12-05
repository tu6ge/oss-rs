use std::env;

use aliyun_oss_client::{
    builder::BuilderError,
    traits::{RefineObject, RefineObjectList},
    Client,
};
use dotenv::dotenv;
use thiserror::Error;

#[derive(Debug)]
struct MyFile {
    key: String,
    #[allow(dead_code)]
    other: String,
}

impl RefineObject for MyFile {
    type Error = MyError;

    fn set_key(&mut self, key: &str) -> Result<(), Self::Error> {
        self.key = key.to_string();
        Ok(())
    }
}

#[derive(Default, Debug)]
struct MyBucket {
    name: String,
    files: Vec<MyFile>,
}

impl RefineObjectList<MyFile> for MyBucket {
    type Error = MyError;

    fn set_name(&mut self, name: &str) -> Result<(), Self::Error> {
        self.name = name.to_string();
        Ok(())
    }
    fn set_list(&mut self, list: Vec<MyFile>) -> Result<(), Self::Error> {
        self.files = list;
        Ok(())
    }
}

#[derive(Debug, Error)]
enum MyError {
    #[error(transparent)]
    QuickXml(#[from] quick_xml::Error),

    #[error(transparent)]
    BuilderError(#[from] BuilderError),
}

async fn get_with_client() -> Result<(), MyError> {
    dotenv().ok();

    let client = Client::from_env().unwrap();

    // 除了设置Default 外，还可以做更多设置
    let mut bucket = MyBucket::default();

    // 利用闭包对 MyFile 做一下初始化设置
    let init_file = || MyFile {
        key: String::default(),
        other: "abc".to_string(),
    };
    let bucket_name = env::var("ALIYUN_BUCKET").unwrap();

    let res: Result<_, MyError> = client
        .base_object_list(bucket_name, [], &mut bucket, init_file)
        .await;

    res?;

    println!("bucket: {:?}", bucket);

    Ok(())
}

#[tokio::main]
pub async fn main() {
    let res = get_with_client().await;

    if let Err(err) = res {
        eprintln!("{}", err);
    }
}
