use std::fmt::{self, Display};

use aliyun_oss_client::{
    builder::ArcPointer,
    config::{InvalidObjectDir, ObjectDir, ObjectPath},
    decode::RefineObject,
    object::ObjectList,
    BucketName, Client, DecodeItemError,
};
use dotenv::dotenv;

#[derive(Debug)]
enum MyObject {
    File(ObjectPath),
    Dir(ObjectDir<'static>),
}

impl RefineObject<MyError> for MyObject {
    fn set_key(&mut self, key: &str) -> Result<(), MyError> {
        let res = key.parse::<ObjectPath>();

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

type MyList = ObjectList<ArcPointer, MyObject, MyError>;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = Client::from_env().unwrap();

    let mut list = MyList::default();

    let init_object = || MyObject::File(ObjectPath::default());

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
