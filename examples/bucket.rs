extern crate dotenv;

use aliyun_oss_client::blocking::builder::ClientWithMiddleware;
use aliyun_oss_client::client::Client;
use aliyun_oss_client::types::Query;
use dotenv::dotenv;

fn main() {
    dotenv().ok();

    let client = Client::<ClientWithMiddleware>::from_env().unwrap();
    //let headers = None;
    let response = client.get_bucket_info().unwrap();
    println!("bucket info: {:?}", response);

    let mut query = Query::new();
    query.insert("max-keys".to_string(), "2".to_string());
    let mut result = response.get_object_list(query).unwrap();
    println!("object list: {:?}", result);
    println!("next object list: {:?}", result.next());
}
