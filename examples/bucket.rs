extern crate dotenv;

use dotenv::dotenv;
use std::{env, collections::HashMap};

fn main() {
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = aliyun_oss_client::client(&key_id,&key_secret, &endpoint, &bucket);
    //let headers = None;
    let response = client.get_bucket_info().unwrap();
    println!("bucket info: {:?}", response);

    let mut query:HashMap<String,String> = HashMap::new();
    query.insert("max-keys".to_string(), "5".to_string());
    query.insert("prefix".to_string(), "babel".to_string());
    println!("bucket object list: {:?}", response.get_object_list(query));
}
