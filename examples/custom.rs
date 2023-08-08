use aliyun_oss_client::{
    decode::{RefineObject, RefineObjectList},
    object::{ExtractListError, InitObject},
    Client, DecodeListError,
};
use dotenv::dotenv;
use thiserror::Error;

#[derive(Debug)]
struct MyFile {
    key: String,
    #[allow(dead_code)]
    other: String,
}

impl RefineObject<MyError> for MyFile {
    fn set_key(&mut self, key: &str) -> Result<(), MyError> {
        self.key = key.to_string();
        Ok(())
    }
}

#[derive(Default, Debug)]
struct MyBucket {
    name: String,
    files: Vec<MyFile>,
}

impl RefineObjectList<MyFile, MyError> for MyBucket {
    fn set_name(&mut self, name: &str) -> Result<(), MyError> {
        self.name = name.to_string();
        Ok(())
    }
    fn set_list(&mut self, list: Vec<MyFile>) -> Result<(), MyError> {
        self.files = list;
        Ok(())
    }
}

#[derive(Debug, Error, DecodeListError)]
#[error("my error")]
struct MyError {}

async fn get_with_client() -> Result<(), ExtractListError> {
    dotenv().ok();

    let client = Client::from_env().unwrap();

    // 除了设置Default 外，还可以做更多设置
    let mut bucket = MyBucket {
        name: "abc".to_string(),
        files: Vec::with_capacity(20),
    };

    // 利用闭包对 MyFile 做一下初始化设置
    impl InitObject<MyFile> for MyBucket {
        fn init_object(&mut self) -> Option<MyFile> {
            Some(MyFile {
                key: String::default(),
                other: "abc".to_string(),
            })
        }
    }

    client.base_object_list([], &mut bucket).await?;

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
