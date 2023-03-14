use std::convert::TryInto;

use http::header::{HeaderMap, HeaderValue, InvalidHeaderValue};
use mockall::mock;

use crate::{
    auth::{AuthHeader, MockAuthToHeaderMap, OssHeader, Sign},
    types::KeyId,
};

mod to_oss_header {
    use std::convert::TryInto;

    use crate::auth::{AuthBuilder, AuthToOssHeader};

    #[test]
    fn test_none() {
        let builder = AuthBuilder::default();
        let header = builder.build().to_oss_header();

        assert!(header.is_none());

        let mut builder = AuthBuilder::default();
        builder.header_insert("abc", "def".try_into().unwrap());
        let header = builder.build().to_oss_header();
        assert!(header.is_none());
    }

    #[test]
    fn test_some() {
        let mut builder = AuthBuilder::default();
        builder.header_insert("x-oss-foo", "bar".try_into().unwrap());
        builder.header_insert("x-oss-ffoo", "barbar".try_into().unwrap());
        builder.header_insert("fffoo", "aabb".try_into().unwrap());
        let header = builder.build().to_oss_header();
        let header = header.to_string();
        assert_eq!(&header, "x-oss-ffoo:barbar\nx-oss-foo:bar\n");
    }
}

mod auth_sign_string {
    use chrono::{TimeZone, Utc};
    use http::Method;

    use crate::auth::AuthBuilder;
    use crate::auth::AuthSignString;

    #[test]
    fn auth_to_sign_string() {
        let date = Utc.with_ymd_and_hms(2022, 1, 1, 18, 1, 1).unwrap();

        let mut builder = AuthBuilder::default();
        builder.key("foo1");
        builder.secret("foo2");
        builder.method(&Method::POST);
        builder.content_md5("foo4");
        builder.date(date);
        builder.canonicalized_resource("foo5");
        builder.header_insert("Content-Type", "foo6".try_into().unwrap());

        let auth = builder.build();

        let (key, secret, verb, content_md5, content_type, date, canonicalized_resource) =
            auth.get_sign_info();

        assert_eq!(key.as_ref(), "foo1");

        assert_eq!(secret.as_ref(), "foo2");

        assert_eq!(verb.to_string(), "POST".to_owned());

        assert_eq!(content_md5.as_ref(), "foo4");

        assert_eq!(content_type.as_ref(), "foo6");

        assert_eq!(date.as_ref(), "Sat, 01 Jan 2022 18:01:01 GMT");

        assert_eq!(canonicalized_resource.as_ref(), "foo5");
    }

    #[test]
    fn auth_to_sign_string_none() {
        let date = Utc.with_ymd_and_hms(2022, 1, 1, 18, 1, 1).unwrap();

        let mut builder = AuthBuilder::default();
        builder.key("foo1");
        builder.secret("foo2");
        builder.method(&Method::POST);
        //.content_md5(ContentMd5::new("foo4"))
        builder.date(date);
        builder.canonicalized_resource("foo5");
        builder.header_insert("Content-Type", "foo6".try_into().unwrap());

        let auth = builder.build();

        let (key, secret, verb, content_md5, content_type, date, canonicalized_resource) =
            auth.get_sign_info();

        assert_eq!(key.as_ref(), "foo1");

        assert_eq!(secret.as_ref(), "foo2");

        assert_eq!(verb.to_string(), "POST".to_owned());

        assert_eq!(content_md5.as_ref(), "");

        assert_eq!(content_type.as_ref(), "foo6");

        assert_eq!(date.as_ref(), "Sat, 01 Jan 2022 18:01:01 GMT");

        assert_eq!(canonicalized_resource.as_ref(), "foo5");
    }
}

mod auth_builder {
    use std::convert::TryInto;

    use chrono::{TimeZone, Utc};
    use http::{header::HOST, HeaderMap, Method};

    use crate::auth::{AuthBuilder, AuthSignString};

    #[test]
    fn test_key() {
        let mut builder = AuthBuilder::default();
        builder.key("foo1");
        let auth = builder.build();

        let (key, ..) = auth.get_sign_info();

        assert_eq!(key.as_ref(), "foo1");
    }

    #[test]
    fn test_secret() {
        let mut builder = AuthBuilder::default();
        builder.secret("foo2");
        let auth = builder.build();

        let (_, secret, ..) = auth.get_sign_info();

        assert_eq!(secret.as_ref(), "foo2");
    }

    #[test]
    fn test_verb() {
        let mut builder = AuthBuilder::default();
        builder.method(&Method::POST);
        let auth = builder.build();

        let (_, _, verb, ..) = auth.get_sign_info();

        assert_eq!(verb, &Method::POST);
    }

    #[test]
    fn test_content_md5() {
        let mut builder = AuthBuilder::default();
        builder.content_md5("abc3");
        let auth = builder.build();

        let (_, _, _, content_md5, ..) = auth.get_sign_info();

        assert_eq!(content_md5.as_ref(), "abc3");
    }

    #[test]
    fn test_date() {
        let mut builder = AuthBuilder::default();
        let date = Utc.with_ymd_and_hms(2022, 1, 1, 18, 1, 1).unwrap();
        builder.date(date);
        let auth = builder.build();

        let (.., date, _) = auth.get_sign_info();

        assert_eq!(date.as_ref(), "Sat, 01 Jan 2022 18:01:01 GMT");
    }

    #[test]
    fn test_canonicalized_resource() {
        let mut builder = AuthBuilder::default();
        builder.canonicalized_resource("foo323");
        let auth = builder.build();

        let (.., canonicalized_resource) = auth.get_sign_info();

        assert_eq!(canonicalized_resource.as_ref(), "foo323");
    }

    #[test]
    fn test_header() {
        let mut builder = AuthBuilder::default();
        let mut header = HeaderMap::new();
        header.insert(HOST, "127.0.0.1".try_into().unwrap());
        builder.headers(header);

        let host = builder.build().get_header("HOST");
        assert!(host.is_some());

        let host = host.unwrap();
        assert_eq!(host.to_str().unwrap(), "127.0.0.1");
    }

    #[test]
    fn test_insert_header() {
        let mut builder = AuthBuilder::default();
        builder.header_insert("Content-Type", "application/json".parse().unwrap());

        let auth = builder.build();
        assert_eq!(auth.header_len(), 1);
        assert!(auth.header_contains_key("Content-Type"));
    }

    #[test]
    fn test_clear() {
        let mut builder = AuthBuilder::default();
        builder.header_insert("Content-Type", "application/json".parse().unwrap());
        builder.header_clear();

        assert_eq!(builder.build().header_len(), 0);
    }
}

mod auth_to_header_map {
    use chrono::{TimeZone, Utc};
    use http::header::HeaderValue;
    use http::Method;

    use crate::auth::AuthBuilder;
    use crate::auth::AuthToHeaderMap;

    #[test]
    fn test_to_header_map() {
        let date = Utc.with_ymd_and_hms(2022, 1, 1, 18, 1, 1).unwrap();

        let mut builder = AuthBuilder::default();
        builder.key("foo1");
        builder.secret("foo2");
        builder.method(&Method::POST);
        builder.content_md5("foo4");
        builder.date(date);
        builder.canonicalized_resource("foo5");
        builder.header_insert("Content-Type", "foo6".try_into().unwrap());

        let auth = builder.build();

        let header = auth.get_original_header();
        assert_eq!(
            header.get("Content-Type").unwrap().to_str().unwrap(),
            "foo6"
        );

        let key = auth.get_header_key().unwrap();
        let secret = auth.get_header_secret().unwrap();
        let verb = auth.get_header_method().unwrap();
        let md5 = auth.get_header_md5().unwrap();
        let date = auth.get_header_date().unwrap();
        let resource = auth.get_header_resource().unwrap();

        assert_eq!(key, HeaderValue::from_bytes(b"foo1").unwrap());
        assert_eq!(secret, HeaderValue::from_bytes(b"foo2").unwrap());
        assert_eq!(verb, HeaderValue::from_bytes(b"POST").unwrap());
        assert_eq!(md5, HeaderValue::from_bytes(b"foo4").unwrap());
        assert_eq!(
            date,
            HeaderValue::from_bytes(b"Sat, 01 Jan 2022 18:01:01 GMT").unwrap()
        );
        assert_eq!(resource, HeaderValue::from_bytes(b"foo5").unwrap());
    }

    #[test]
    fn test_to_header_map_none() {
        let date = Utc.with_ymd_and_hms(2022, 1, 1, 18, 1, 1).unwrap();

        let mut builder = AuthBuilder::default();
        builder.key("foo1");
        builder.secret("foo2");
        builder.method(&Method::POST);
        //.content_md5(ContentMd5::new("foo4"))
        builder.date(date);
        builder.canonicalized_resource("foo5");
        builder.header_insert("Content-Type", "foo6".try_into().unwrap());

        let auth = builder.build();

        let md5 = auth.get_header_md5();

        assert!(md5.is_none());
    }
}

#[test]
fn header_map_from_auth() {
    let mut auth = MockAuthToHeaderMap::default();
    auth.expect_get_original_header()
        .times(1)
        .returning(|| HeaderMap::new());
    auth.expect_get_header_key()
        .times(1)
        .returning(|| Ok("foo1".parse().unwrap()));
    auth.expect_get_header_secret()
        .times(1)
        .returning(|| Ok("foo2".parse().unwrap()));
    auth.expect_get_header_method()
        .times(1)
        .returning(|| Ok("foo3".parse().unwrap()));

    auth.expect_get_header_md5().times(1).returning(|| {
        let val: HeaderValue = "foo4".parse().unwrap();
        Some(val)
    });
    auth.expect_get_header_date()
        .times(1)
        .returning(|| Ok("foo6".parse().unwrap()));
    auth.expect_get_header_resource()
        .times(1)
        .returning(|| Ok("foo7".parse().unwrap()));

    let map = HeaderMap::from_auth(&auth);
    assert!(map.is_ok());

    let map = map.unwrap();
    assert_eq!(map.get("AccessKeyId").unwrap().to_str().unwrap(), "foo1");
    assert_eq!(
        map.get("SecretAccessKey").unwrap().to_str().unwrap(),
        "foo2"
    );
}

#[test]
fn test_append_sign() {
    mock! {
        BarStruct {}

        impl TryInto<HeaderValue> for BarStruct {
            type Error = InvalidHeaderValue;
            fn try_into(self) -> Result<HeaderValue, InvalidHeaderValue>;
        }
    }

    let mut myinto = MockBarStruct::new();
    myinto
        .expect_try_into()
        .times(1)
        .returning(|| HeaderValue::from_str("foo"));

    let mut map = HeaderMap::new();
    let res = map.append_sign(myinto);

    assert!(res.is_ok());
    let val = res.unwrap();
    assert!(val.is_none());
    assert!(!map.is_empty());

    let mut myinto = MockBarStruct::new();
    myinto.expect_try_into().times(1).returning(|| {
        let val = HeaderValue::from_str("foo").unwrap();
        Ok(val)
    });

    let mut map = HeaderMap::new();
    map.insert("Authorization", "bar".try_into().unwrap());
    let res = map.append_sign(myinto);

    assert!(res.is_ok());
    let val = res.unwrap();
    assert!(matches!(val, Some(v) if v==HeaderValue::from_bytes(b"bar").unwrap()));
}

#[test]
fn to_sign_string() {
    let header = OssHeader::new(None);
    let string = header.to_string();
    assert_eq!(&string, "");

    let header = OssHeader::new(Some(String::from("")));
    let string = header.to_string();
    assert_eq!(&string, "\n");

    let header = OssHeader::new(Some(String::from("abc")));
    let string = header.to_string();
    assert_eq!(&string, "abc\n");
}

#[test]
fn header_into_string() {
    let header = OssHeader::new(None);
    let string: String = header.to_string();
    assert_eq!(&string, "");

    let header = OssHeader::new(Some(String::from("")));
    let string: String = header.to_string();
    assert_eq!(&string, "\n");

    let header = OssHeader::new(Some(String::from("abc")));
    let string: String = header.to_string();
    assert_eq!(&string, "abc\n");
}

mod sign_string_struct {

    use http::Method;

    use crate::{
        auth::{AuthSignString, OssHeader, SignString},
        types::{CanonicalizedResource, ContentMd5, ContentType, Date, KeyId, KeySecret},
    };

    #[test]
    fn test_from_auth() {
        struct Bar {
            key: KeyId,
            secret: KeySecret,
            verb: Method,
            date: Date,
            content_md5: ContentMd5,
            content_type: ContentType,
            canonicalized_resource: CanonicalizedResource,
        }

        impl AuthSignString for Bar {
            fn get_sign_info(
                &self,
            ) -> (
                &KeyId,
                &KeySecret,
                &Method,
                ContentMd5,
                ContentType,
                &Date,
                &CanonicalizedResource,
            ) {
                (
                    &self.key,
                    &self.secret,
                    &self.verb,
                    self.content_md5.clone(),
                    self.content_type.clone(),
                    &self.date,
                    &self.canonicalized_resource,
                )
            }
        }

        let bar = Bar {
            key: KeyId::new("foo1"),
            secret: KeySecret::new("foo2"),
            verb: Method::GET,
            content_md5: ContentMd5::new("foo3"),
            content_type: ContentType::new("foo4"),
            date: Date::new("foo5"),
            canonicalized_resource: CanonicalizedResource::new("foo6"),
        };

        let header = OssHeader::new(Some("foo7".to_string()));

        let val = SignString::from_auth(&bar, header);

        assert_eq!(val.data(), "GET\nfoo3\nfoo4\nfoo5\nfoo7\nfoo6");
        assert_eq!(val.key_string(), "foo1".to_string());
        assert_eq!(val.secret_string(), "foo2".to_string());
    }

    #[test]
    fn test_to_sign() {
        let key = KeyId::from("foo1");
        let secret = KeySecret::from("foo2");
        let sign_string = SignString::new("bar", key, secret);

        let res = sign_string.to_sign();
        assert!(res.is_ok());
        let sign = res.unwrap();
        assert_eq!(sign.data(), "gTzwiN1fRQV90YcecTvo1pH+kI8=");
        assert_eq!(sign.key_string(), "foo1".to_string());
    }
}

#[test]
fn test_sign_to_headervalue() {
    let key = KeyId::from("bar");
    let sign = Sign::new("foo", key);

    let val: HeaderValue = sign.try_into().unwrap();
    assert_eq!(val.to_str().unwrap(), "OSS bar:foo");
}

mod get_headers {
    use http::Method;

    use crate::auth::AuthBuilder;

    /// 集成测试，其他的都是单元测试
    #[test]
    fn test_get_headers() {
        let mut builder = AuthBuilder::default();
        builder.key("foo1");
        builder.secret("foo2");
        builder.method(&Method::POST);
        builder.content_md5("foo4");
        builder.date("foo_date");
        builder.canonicalized_resource("foo5");
        builder.header_insert("Content-Type", "foo6".try_into().unwrap());
        let map = builder.build().get_headers();

        assert!(map.is_ok());
        let map = map.unwrap();
        assert_eq!(map.len(), 8);
        assert_eq!(map.get("Content-Type").unwrap(), &"foo6");
        assert_eq!(map.get("accesskeyid").unwrap(), &"foo1");
        assert_eq!(map.get("secretaccesskey").unwrap(), &"foo2");
        assert_eq!(map.get("verb").unwrap(), &"POST");
        assert_eq!(map.get("content-md5").unwrap(), &"foo4");
        assert_eq!(map.get("date").unwrap(), &"foo_date");
        assert_eq!(map.get("canonicalizedresource").unwrap(), &"foo5");
        assert_eq!(
            map.get("authorization").unwrap(),
            &"OSS foo1:67qpyspFaWOYrWwahWKgNN+ngUY="
        );
    }
}
