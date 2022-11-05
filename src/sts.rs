use crate::{auth::AuthBuilder, client::Client, BucketName, EndPoint, KeyId, KeySecret};

pub trait STS {
    fn new_with_sts(
        access_key_id: KeyId,
        access_key_secret: KeySecret,
        endpoint: EndPoint,
        bucket: BucketName,
        security_token: String,
    ) -> Self;
}

impl<M: Default> STS for Client<M> {
    fn new_with_sts(
        access_key_id: KeyId,
        access_key_secret: KeySecret,
        endpoint: EndPoint,
        bucket: BucketName,
        security_token: String,
    ) -> Self {
        let auth_builder = AuthBuilder::default()
            .key(access_key_id)
            .secret(access_key_secret)
            .header_insert("x-oss-security-token", security_token.try_into().unwrap());

        Self::from_builder(auth_builder, endpoint, bucket)
    }
}

#[cfg(test)]
mod tests {
    use http::HeaderValue;

    use crate::{types::CanonicalizedResource, BucketName, Client, EndPoint};

    use super::STS;

    #[tokio::test]
    async fn test_sts() {
        let client = Client::new_with_sts(
            "foo1".into(),
            "foo2".into(),
            EndPoint::CnShanghai,
            BucketName::new("abc").unwrap(),
            "bar".to_string(),
        );

        let builder = client
            .builder(
                "GET",
                "https://abc.oss-cn-shanghai.aliyuncs.com/"
                    .try_into()
                    .unwrap(),
                CanonicalizedResource::default(),
            )
            .unwrap();

        let request = builder.build().unwrap();

        let headers = request.headers();
        let sts_token = headers.get("x-oss-security-token");

        assert_eq!(sts_token, Some(&HeaderValue::from_static("bar")));
    }
}
