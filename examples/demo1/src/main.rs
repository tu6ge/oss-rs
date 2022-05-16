
use oss::auth;
use oss::client;
use oss::auth::VERB;

fn main() {
    let client = client::Client::new("abc","cde", "bar");
    let headers = None;
    let response = client.request(VERB::GET, "https://oss-cn-hangzhou.aliyuncs.com",headers).unwrap();
    println!("{}", response.text().unwrap());
}
