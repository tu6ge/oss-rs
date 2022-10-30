pub mod builder;

use crate::config::Config;
use crate::types::{BucketName, EndPoint, KeyId, KeySecret};

use self::builder::ClientWithMiddleware;

pub fn client<ID, S, E, B>(
    access_key_id: ID,
    access_key_secret: S,
    endpoint: E,
    bucket: B,
) -> crate::client::Client<ClientWithMiddleware>
where
    ID: Into<KeyId>,
    S: Into<KeySecret>,
    E: Into<EndPoint>,
    B: Into<BucketName>,
{
    let config = Config::new(access_key_id, access_key_secret, endpoint, bucket);
    crate::client::Client::<ClientWithMiddleware>::from_config(&config)
}
