use aliyun_oss_client::{Client, Query};
use dotenv::dotenv;
use futures::{pin_mut, StreamExt};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = Client::from_env().unwrap();

    let mut query = Query::new();
    query.insert("max-keys", 1000u16);

    let object_list = client.get_object_list(query).await.unwrap();

    println!(
        "list: {:?}, token: {:?}",
        object_list,
        object_list.next_continuation_token()
    );

    let stream = object_list.into_stream();

    pin_mut!(stream);

    let second_list = stream.next().await;
    let third_list = stream.next().await;

    println!("second_list: {:?}", second_list);
    println!("third_list: {:?}", third_list);
}
