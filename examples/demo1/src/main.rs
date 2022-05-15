use oss::abc;

use oss::auth;

fn main() {
    
    let auth = auth::Auth{
        access_key_id:"abc",
        access_key_secret:"cde",
        verb: auth::VERB::POST,
        content_md5:"test_md5",
        content_type:"application/octet-stream",
        date: "2022-11-22 22:33:33",
        canonicalized_resource: "abc",
    };
    println!("Hello, {}!", auth.sign());
}
