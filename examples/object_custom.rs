use aliyun_oss_client::{
    builder::{ArcPointer, BuilderError},
    config::{InvalidObjectDir, ObjectDir, ObjectPath},
    decode::RefineObject,
    object::ObjectList,
    BucketName, Client,
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
            _ => {
                let re = key.parse::<ObjectDir>();
                MyObject::Dir(re.unwrap())
            }
        };

        Ok(())
    }
}

struct MyError(String);

impl From<quick_xml::Error> for MyError {
    fn from(value: quick_xml::Error) -> Self {
        Self(value.to_string())
    }
}

impl From<BuilderError> for MyError {
    fn from(value: BuilderError) -> Self {
        Self(value.to_string())
    }
}

impl From<std::num::ParseIntError> for MyError {
    fn from(value: std::num::ParseIntError) -> Self {
        Self(value.to_string())
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
