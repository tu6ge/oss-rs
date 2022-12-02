use std::convert::TryInto;

use mockall::mock;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::{
    auth::{AuthHeader, HeaderToSign, MockAuthToHeaderMap, OssHeader, Sign, VERB},
    errors::{OssError, OssResult},
    types::KeyId,
};

#[test]
fn test_verb2string() {
    let verb = VERB::GET;
    let string: String = verb.into();
    assert_eq!(string, "GET".to_owned());

    let verb = VERB::POST;
    let string: String = verb.into();
    assert_eq!(string, "POST".to_owned());

    let verb = VERB::PUT;
    let string: String = verb.into();
    assert_eq!(string, "PUT".to_owned());

    let verb = VERB::DELETE;
    let string: String = verb.into();
    assert_eq!(string, "DELETE".to_owned());

    let verb = VERB::HEAD;
    let string: String = verb.into();
    assert_eq!(string, "HEAD".to_owned());

    let verb = VERB::OPTIONS;
    let string: String = verb.into();
    assert_eq!(string, "OPTIONS".to_owned());

    let verb = VERB::CONNECT;
    let string: String = verb.into();
    assert_eq!(string, "CONNECT".to_owned());

    let verb = VERB::PATCH;
    let string: String = verb.into();
    assert_eq!(string, "PATCH".to_owned());

    let verb = VERB::TRACE;
    let string: String = verb.into();
    assert_eq!(string, "TRACE".to_owned());
}

#[test]
fn test_str2verb() {
    let str = "GET";
    let verb: VERB = str.into();
    assert_eq!(verb, VERB::GET);

    let str = "POST";
    let verb: VERB = str.into();
    assert_eq!(verb, VERB::POST);

    let str = "PUT";
    let verb: VERB = str.into();
    assert_eq!(verb, VERB::PUT);

    let str = "DELETE";
    let verb: VERB = str.into();
    assert_eq!(verb, VERB::DELETE);

    let str = "HEAD";
    let verb: VERB = str.into();
    assert_eq!(verb, VERB::HEAD);

    let str = "OPTIONS";
    let verb: VERB = str.into();
    assert_eq!(verb, VERB::OPTIONS);

    let str = "CONNECT";
    let verb: VERB = str.into();
    assert_eq!(verb, VERB::CONNECT);

    let str = "PATCH";
    let verb: VERB = str.into();
    assert_eq!(verb, VERB::PATCH);

    let str = "TRACE";
    let verb: VERB = str.into();
    assert_eq!(verb, VERB::TRACE);
}

#[test]
fn test_verb2headervalue() {
    let verb = VERB::GET;
    let header_value: HeaderValue = verb.try_into().unwrap();

    assert_eq!(header_value.to_str().unwrap(), "GET");
}

#[test]
fn test_verb_default() {
    let verb = VERB::default();
    assert_eq!(verb, VERB::GET);
}

mod to_oss_header {
    use std::convert::TryInto;

    use crate::auth::{AuthBuilder, AuthToOssHeader};

    #[test]
    fn test_none() {
        let builder = AuthBuilder::default();
        let header = builder.build().to_oss_header();
        assert!(header.is_ok());
        let header = header.unwrap();
        assert!(header.is_none());

        let builder = AuthBuilder::default().header_insert("abc", "def".try_into().unwrap());
        let header = builder.build().to_oss_header();
        assert!(header.is_ok());
        let header = header.unwrap();
        assert!(header.is_none());
    }

    #[test]
    fn test_some() {
        let builder = AuthBuilder::default()
            .header_insert("x-oss-foo", "bar".try_into().unwrap())
            .header_insert("x-oss-ffoo", "barbar".try_into().unwrap())
            .header_insert("fffoo", "aabb".try_into().unwrap());
        let header = builder.build().to_oss_header();
        assert!(header.is_ok());
        let header = header.unwrap();
        let header: String = header.into();
        assert_eq!(&header, "x-oss-ffoo:barbar\nx-oss-foo:bar\n");
    }
}

mod auth_sign_string {
    use chrono::{TimeZone, Utc};

    use crate::auth::{AuthSignString, VERB};
    use crate::{
        auth::AuthBuilder,
        types::{CanonicalizedResource, ContentMd5},
    };

    #[test]
    fn auth_to_sign_string() {
        let date = Utc.ymd(2022, 1, 1).and_hms(18, 1, 1);

        let builder = AuthBuilder::default()
            .key("foo1".into())
            .secret("foo2".into())
            .verb(&VERB::POST)
            .content_md5(ContentMd5::new("foo4"))
            .date(date.into())
            .canonicalized_resource(CanonicalizedResource::new("foo5"))
            .header_insert("Content-Type", "foo6".try_into().unwrap());

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
        let date = Utc.ymd(2022, 1, 1).and_hms(18, 1, 1);

        let builder = AuthBuilder::default()
            .key("foo1".into())
            .secret("foo2".into())
            .verb(&VERB::POST)
            //.content_md5(ContentMd5::new("foo4"))
            .date(date.into())
            .canonicalized_resource(CanonicalizedResource::new("foo5"))
            .header_insert("Content-Type", "foo6".try_into().unwrap());

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
    use http::{header::HOST, HeaderMap};

    use crate::auth::{AuthBuilder, AuthSignString, VERB};

    #[test]
    fn test_key() {
        let mut builder = AuthBuilder::default();
        builder = builder.key("foo1".to_owned().into());
        let auth = builder.build();

        let (key, ..) = auth.get_sign_info();

        assert_eq!(key.as_ref(), "foo1");
    }

    #[test]
    fn test_secret() {
        let mut builder = AuthBuilder::default();
        builder = builder.secret("foo2".to_owned().into());
        let auth = builder.build();

        let (_, secret, ..) = auth.get_sign_info();

        assert_eq!(secret.as_ref(), "foo2");
    }

    #[test]
    fn test_verb() {
        let mut builder = AuthBuilder::default();
        builder = builder.verb(&VERB::POST);
        let auth = builder.build();

        let (_, _, verb, ..) = auth.get_sign_info();

        assert_eq!(verb, &VERB::POST);
    }

    #[test]
    fn test_content_md5() {
        let mut builder = AuthBuilder::default();
        builder = builder.content_md5("abc3".to_owned().into());
        let auth = builder.build();

        let (_, _, _, content_md5, ..) = auth.get_sign_info();

        assert_eq!(content_md5.as_ref(), "abc3");
    }

    #[test]
    fn test_date() {
        let mut builder = AuthBuilder::default();
        let date = Utc.ymd(2022, 1, 1).and_hms(18, 1, 1);
        builder = builder.date(date.into());
        let auth = builder.build();

        let (.., date, _) = auth.get_sign_info();

        assert_eq!(date.as_ref(), "Sat, 01 Jan 2022 18:01:01 GMT");
    }

    #[test]
    fn test_canonicalized_resource() {
        let mut builder = AuthBuilder::default();
        builder = builder.canonicalized_resource("foo323".to_string().into());
        let auth = builder.build();

        let (.., canonicalized_resource) = auth.get_sign_info();

        assert_eq!(canonicalized_resource.as_ref(), "foo323");
    }

    #[test]
    fn test_header() {
        let mut builder = AuthBuilder::default();
        let mut header = HeaderMap::new();
        header.insert(HOST, "127.0.0.1".try_into().unwrap());
        builder = builder.headers(header);

        let host = builder.build().get_header("HOST");
        assert!(host.is_some());

        let host = host.unwrap();
        assert_eq!(host.to_str().unwrap(), "127.0.0.1");
    }

    #[test]
    fn test_insert_header() {
        let mut builder = AuthBuilder::default();
        builder = builder.header_insert("Content-Type", "application/json".parse().unwrap());

        let auth = builder.build();
        assert_eq!(auth.header_len(), 1);
        assert!(auth.header_contains_key("Content-Type"));
    }

    #[test]
    fn test_clear() {
        let mut builder = AuthBuilder::default();
        builder = builder.header_insert("Content-Type", "application/json".parse().unwrap());
        builder = builder.header_clear();

        assert_eq!(builder.build().header_len(), 0);
    }
}

mod auth_to_header_map {
    use chrono::{TimeZone, Utc};
    use http::header::HeaderValue;

    use crate::auth::{AuthToHeaderMap, VERB};
    use crate::{
        auth::AuthBuilder,
        types::{CanonicalizedResource, ContentMd5},
    };

    #[test]
    fn test_to_header_map() {
        let date = Utc.ymd(2022, 1, 1).and_hms(18, 1, 1);

        let builder = AuthBuilder::default()
            .key("foo1".into())
            .secret("foo2".into())
            .verb(&VERB::POST)
            .content_md5(ContentMd5::new("foo4"))
            .date(date.into())
            .canonicalized_resource(CanonicalizedResource::new("foo5"))
            .header_insert("Content-Type", "foo6".try_into().unwrap());

        let auth = builder.build();

        let header = auth.get_original_header();
        assert_eq!(
            header.get("Content-Type").unwrap().to_str().unwrap(),
            "foo6"
        );

        let key = auth.get_header_key().unwrap();
        let secret = auth.get_header_secret().unwrap();
        let verb = auth.get_header_verb().unwrap();
        let md5 = auth.get_header_md5().unwrap();
        let date = auth.get_header_date().unwrap();
        let resource = auth.get_header_resource().unwrap();

        assert_eq!(key, HeaderValue::from_bytes(b"foo1").unwrap());
        assert_eq!(secret, HeaderValue::from_bytes(b"foo2").unwrap());
        assert_eq!(verb, HeaderValue::from_bytes(b"POST").unwrap());
        assert!(matches!(md5, Some(v) if v==HeaderValue::from_bytes(b"foo4").unwrap()));
        assert_eq!(
            date,
            HeaderValue::from_bytes(b"Sat, 01 Jan 2022 18:01:01 GMT").unwrap()
        );
        assert_eq!(resource, HeaderValue::from_bytes(b"foo5").unwrap());
    }

    #[test]
    fn test_to_header_map_none() {
        let date = Utc.ymd(2022, 1, 1).and_hms(18, 1, 1);

        let builder = AuthBuilder::default()
            .key("foo1".into())
            .secret("foo2".into())
            .verb(&VERB::POST)
            //.content_md5(ContentMd5::new("foo4"))
            .date(date.into())
            .canonicalized_resource(CanonicalizedResource::new("foo5"))
            .header_insert("Content-Type", "foo6".try_into().unwrap());

        let auth = builder.build();

        let md5 = auth.get_header_md5().unwrap();

        assert!(matches!(md5, None));
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
    auth.expect_get_header_verb()
        .times(1)
        .returning(|| Ok("foo3".parse().unwrap()));

    auth.expect_get_header_md5().times(1).returning(|| {
        let val: HeaderValue = "foo4".parse().unwrap();
        Ok(Some(val))
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
            type Error = OssError;
            fn try_into(self) -> OssResult<HeaderValue>;
        }
    }

    let mut myinto = MockBarStruct::new();
    myinto.expect_try_into().times(1).returning(|| {
        let val = HeaderValue::from_str("foo").unwrap();
        Ok(val)
    });

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
    let string = header.to_sign_string();
    assert_eq!(&string, "");

    let header = OssHeader::new(Some(String::from("")));
    let string = header.to_sign_string();
    assert_eq!(&string, "\n");

    let header = OssHeader::new(Some(String::from("abc")));
    let string = header.to_sign_string();
    assert_eq!(&string, "abc\n");
}

#[test]
fn header_into_string() {
    let header = OssHeader::new(None);
    let string: String = header.try_into().unwrap();
    assert_eq!(&string, "");

    let header = OssHeader::new(Some(String::from("")));
    let string: String = header.try_into().unwrap();
    assert_eq!(&string, "\n");

    let header = OssHeader::new(Some(String::from("abc")));
    let string: String = header.try_into().unwrap();
    assert_eq!(&string, "abc\n");
}

mod sign_string_struct {

    use crate::{
        auth::{AuthSignString, MockHeaderToSign, SignString, VERB},
        types::{CanonicalizedResource, ContentMd5, ContentType, Date, KeyId, KeySecret},
    };

    #[test]
    fn test_from_auth() {
        struct Bar {
            key: KeyId,
            secret: KeySecret,
            verb: VERB,
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
                &VERB,
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
            verb: VERB::GET,
            content_md5: ContentMd5::new("foo3"),
            content_type: ContentType::new("foo4"),
            date: Date::new("foo5"),
            canonicalized_resource: CanonicalizedResource::new("foo6"),
        };

        let mut header = MockHeaderToSign::new();

        header
            .expect_to_sign_string()
            .times(1)
            .returning(|| "foo7".to_string());

        let res = SignString::from_auth(&bar, header);

        assert!(res.is_ok());
        let val = res.unwrap();
        assert_eq!(val.data(), "GET\nfoo3\nfoo4\nfoo5\nfoo7foo6");
        assert_eq!(val.key_string(), "foo1".to_string());
        assert_eq!(val.secret_string(), "foo2".to_string());
    }

    #[test]
    fn test_to_sign() {
        let key = KeyId::from("foo1");
        let secret = KeySecret::from("foo2");
        let sign_string = SignString::new("bar", &key, &secret);

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
    let sign = Sign::new("foo", &key);

    let val: HeaderValue = sign.try_into().unwrap();
    assert_eq!(val.to_str().unwrap(), "OSS bar:foo");
}

mod get_headers {
    use crate::{
        auth::{AuthBuilder, AuthGetHeader, VERB},
        types::{CanonicalizedResource, ContentMd5},
    };

    /// 集成测试，其他的都是单元测试
    #[test]
    fn test_get_headers() {
        let builder = AuthBuilder::default()
            .key("foo1".into())
            .secret("foo2".into())
            .verb(&VERB::POST)
            .content_md5(ContentMd5::new("foo4"))
            .date("foo_date".into())
            .canonicalized_resource(CanonicalizedResource::new("foo5"))
            .header_insert("Content-Type", "foo6".try_into().unwrap());
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
