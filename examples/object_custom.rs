use aliyun_oss_client::{
    decode::RefineObject,
    object::Objects,
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

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = Client::from_env().unwrap();

    let mut list = MyList::default();

    fn init_object<'a, 'b>(_list: &'a mut MyList<'b>) -> Option<MyObject<'b>> {
        Some(MyObject::File(ObjectPathInner::default()))
    }

    let _ = client.base_object_list([], &mut list, init_object).await;

    let _second = list.get_next_base(init_object).await;

    println!("list: {:?}", list.to_vec());
}
