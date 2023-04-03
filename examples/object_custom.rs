use std::fmt::{self, Display};

use aliyun_oss_client::{
    builder::ArcPointer,
    decode::RefineObject,
    object::ObjectList,
    types::object::{InvalidObjectDir, ObjectDir, ObjectPathInner},
    BucketName, Client, DecodeItemError,
};
use dotenv::dotenv;

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

#[derive(DecodeItemError)]
struct MyError(String);

impl Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

impl From<InvalidObjectDir> for MyError {
    fn from(value: InvalidObjectDir) -> Self {
        Self(value.to_string())
    }
}

type MyList<'a> = ObjectList<ArcPointer, MyObject<'a>, MyError>;

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

    println!("list: {:?}", list.object_list);
}
