#![feature(test)]
extern crate test;

pub mod auth;
pub mod bucket;
pub mod object;
pub mod client;


#[allow(soft_unstable)]
#[cfg(test)]
mod tests {
  use test::Bencher;

  use std::env;
  use super::*;
  extern crate dotenv;
  use dotenv::dotenv;


  #[test]
  fn test_get_object() {
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = client::Client::new(&key_id,&key_secret, &endpoint, &bucket);
  }

  #[bench]
  fn bench_get_object(b: &mut Bencher){
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = client::Client::new(&key_id,&key_secret, &endpoint, &bucket);
    b.iter(|| {
      client.get_object_list();
    });
  }

}
