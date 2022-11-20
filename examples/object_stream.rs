use aliyun_oss_client::{object::get_next_list, Client, Query};
use dotenv::dotenv;
use futures::{pin_mut, StreamExt};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = Client::from_env().unwrap();

    let mut query = Query::new();
    query.insert("max-keys", "2");

    let object_list = client.get_object_list(query).await.unwrap();

    println!(
        "list: {:?}, token: {:?}",
        object_list,
        object_list.next_continuation_token()
    );

    let list = get_next_list(object_list);

    pin_mut!(list);

    let second_list = list.next().await;

    println!("second_list: {:?}", second_list);
}
