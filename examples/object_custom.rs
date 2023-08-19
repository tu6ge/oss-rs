use aliyun_oss_client::{
    decode::RefineObject,
    object::{InitObject, Objects},
    types::object::{InvalidObjectDir, ObjectDir, ObjectPathInner},
    Client,
};
use dotenv::dotenv;

#[derive(Debug)]
enum MyObject<'a> {
    File(ObjectPathInner<'a>),
    Dir(ObjectDir<'a>),
}

impl RefineObject<InvalidObjectDir> for MyObject<'_> {
    fn set_key(&mut self, key: &str) -> Result<(), InvalidObjectDir> {
        *self = match key.parse() {
            Ok(file) => MyObject::File(file),
            _ => MyObject::Dir(key.parse()?),
        };
        Ok(())
    }
}

type MyList<'a> = Objects<MyObject<'a>>;

impl<'a> InitObject<MyObject<'a>> for MyList<'a> {
    fn init_object(&mut self) -> Option<MyObject<'a>> {
        Some(MyObject::File(ObjectPathInner::default()))
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = Client::from_env().unwrap();

    let mut list = MyList::default();

    let _ = client.base_object_list([()], &mut list).await;

    let _second = list.get_next_base().await;

    println!("list: {:?}", list.to_vec());
}
