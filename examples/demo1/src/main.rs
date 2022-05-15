
use oss::auth;
use oss::client;
use oss::auth::VERB;
use reqwest::Method;

fn main() {
    let client = client::Client::new("abc","cde", "bar");
    let response = client.builder(VERB(Method::GET), "https://oss-cn-hangzhou.aliyuncs.com").unwrap();
    println!("{}", response);
}
