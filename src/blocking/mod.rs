pub mod client;

pub mod builder;

pub mod bucket;

pub mod object;

use crate::config::Config;
use crate::types::{KeyId, KeySecret, EndPoint, BucketName};

pub fn client<ID, S, E, B>(access_key_id: ID, access_key_secret: S, endpoint: E, bucket: B) -> client::Client
where ID: Into<KeyId>,
    S: Into<KeySecret>,
    E: Into<EndPoint>,
    B: Into<BucketName>,
{
    let config = Config::new(access_key_id, access_key_secret, endpoint, bucket);
    client::Client::from_config(&config)
}
