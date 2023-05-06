use aliyun_oss_client::{
    decode::RefineObject,
    object::Objects,
    types::object::{InvalidObjectDir, ObjectDir, ObjectPathInner},
    BucketName, Client,
};
use dotenv::dotenv;
use thiserror::Error;

#[derive(Debug)]
enum MyObject<'a> {
    File(ObjectPathInner<'a>),
    Dir(ObjectDir<'a>),
}

impl RefineObject<MyError> for MyObject<'_> {
    fn set_key(&mut self, key: &str) -> Result<(), MyError> {
        let res = key.parse::<ObjectPathInner>();

        *self = match res {
            Ok(file) => MyObject::File(file),
            _ => MyObject::Dir(key.parse()?),
        };

        Ok(())
    }
}

#[derive(Debug, Error)]
#[error("my error")]
struct MyError(String);

impl From<InvalidObjectDir> for MyError {
    fn from(value: InvalidObjectDir) -> Self {
        Self(value.to_string())
    }
}

type MyList<'a> = Objects<MyObject<'a>, MyError>;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = Client::from_env().unwrap();

    let mut list = MyList::default();

    let init_object = || MyObject::File(ObjectPathInner::default());

    let _ = client
        .base_object_list(
            "xxxxxx".parse::<BucketName>().unwrap(),
            [],
            &mut list,
            init_object,
        )
        .await;

    println!("list: {:?}", list.to_vec());
}
