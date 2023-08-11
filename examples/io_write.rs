use std::{io::Write, sync::Arc};

use aliyun_oss_client::{object::content::Content, Client};
use dotenv::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let client = Client::from_env().unwrap();
    let mut objcet = Content::from_client(Arc::new(client))
        .path("aaabbb3.txt")
        .unwrap();

    //objcet.part_size(100 * 1024).unwrap();

    {
        objcet
            .write_all(b"abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789")
            .unwrap();
        objcet.flush().unwrap()
    }

    println!("finish");
}
