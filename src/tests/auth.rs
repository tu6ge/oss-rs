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
        let header = builder.auth.to_oss_header();
        assert!(header.is_ok());
        let header = header.unwrap();
        assert!(header.is_none());

        let builder = AuthBuilder::default().header_insert("abc", "def".try_into().unwrap());
        let header = builder.auth.to_oss_header();
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
        let header = builder.auth.to_oss_header();
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
            .header_insert("Content-Type", "foo6".try_into().unwrap())
            .type_with_header();

        let auth = builder.auth;

        let key = auth.key();
        assert_eq!(key.into_owned().as_ref(), "foo1");

        let secret = auth.secret();
        assert_eq!(secret.into_owned().as_ref(), "foo2");

        let verb = auth.verb();
        assert_eq!(verb.to_string(), "POST".to_owned());

        let md5 = auth.content_md5();
        assert_eq!(md5.into_owned().as_ref(), "foo4");

        let content_type = auth.content_type();
        assert_eq!(content_type.into_owned().as_ref(), "foo6");

        let inner_date = auth.date();
        assert_eq!(
            inner_date.into_owned().as_ref(),
            "Sat, 01 Jan 2022 18:01:01 GMT"
        );

        let canonicalized_resource = auth.canonicalized_resource();
        assert_eq!(canonicalized_resource.into_owned().as_ref(), "foo5");
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
        .header_insert("Content-Type", "foo6".try_into().unwrap())
        //.type_with_header()
        ;

        let auth = builder.auth;

        let key = auth.key();
        assert_eq!(key.into_owned().as_ref(), "foo1");

        let secret = auth.secret();
        assert_eq!(secret.into_owned().as_ref(), "foo2");

        let verb = auth.verb();
        assert_eq!(verb.to_string(), "POST".to_owned());

        let md5 = auth.content_md5();
        assert_eq!(md5.into_owned().as_ref(), "");

        let content_type = auth.content_type();
        assert_eq!(content_type.into_owned().as_ref(), "");

        let inner_date = auth.date();
        assert_eq!(
            inner_date.into_owned().as_ref(),
            "Sat, 01 Jan 2022 18:01:01 GMT"
        );

        let canonicalized_resource = auth.canonicalized_resource();
        assert_eq!(canonicalized_resource.into_owned().as_ref(), "foo5");
    }
}

mod auth_builder {
    use std::convert::TryInto;

    use chrono::{TimeZone, Utc};
    use http::{
        header::{CONTENT_TYPE, HOST},
        HeaderMap,
    };

    use crate::{
        auth::{Auth, AuthBuilder, VERB},
        types::{CanonicalizedResource, KeySecret},
    };

    #[test]
    fn test_key() {
        let mut builder = AuthBuilder::default();
        builder = builder.key("foo1".to_owned().into());

        assert_eq!(builder.auth.access_key_id.as_ref(), "foo1");
    }

    #[test]
    fn test_secret() {
        let mut builder = AuthBuilder::default();
        builder = builder.secret("foo2".to_owned().into());

        assert_eq!(builder.auth.access_key_secret.as_ref(), "foo2");
    }

    #[test]
    fn test_verb() {
        let mut builder = AuthBuilder::default();
        builder = builder.verb(&VERB::POST);

        assert!(matches!(builder.auth.verb, VERB::POST));
    }

    #[test]
    fn test_content_md5() {
        let mut builder = AuthBuilder::default();
        builder = builder.content_md5("abc3".to_owned().into());

        assert!(matches!(builder.auth.content_md5, Some(v) if v.as_ref()=="abc3"));
    }

    #[test]
    fn test_date() {
        let mut builder = AuthBuilder::default();
        let date = Utc.ymd(2022, 1, 1).and_hms(18, 1, 1);
        builder = builder.date(date.into());

        assert_eq!(builder.auth.date.as_ref(), "Sat, 01 Jan 2022 18:01:01 GMT");
    }

    #[test]
    fn test_canonicalized_resource() {
        let mut builder = AuthBuilder::default();
        builder = builder.canonicalized_resource("foo323".to_string().into());

        assert_eq!(builder.auth.canonicalized_resource.as_ref(), "foo323");
    }

    #[test]
    fn test_type_with_header() {
        let mut builder = AuthBuilder::default();
        let auth = Auth {
            access_key_id: "foo1".to_owned().into(),
            access_key_secret: KeySecret::new("foo2"),
            verb: VERB::GET,
            content_md5: None,
            content_type: None,
            date: "foo3".into(),
            canonicalized_resource: CanonicalizedResource::new("foo4"),
            headers: HeaderMap::new(),
        };

        builder.auth = auth;

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "bar".try_into().unwrap());

        builder.auth.headers = headers;
        builder = builder.type_with_header();

        assert!(matches!(builder.auth.content_type, Some(v) if v.as_ref()=="bar"));
    }

    #[test]
    fn test_header() {
        let mut builder = AuthBuilder::default();
        let mut header = HeaderMap::new();
        header.insert(HOST, "127.0.0.1".try_into().unwrap());
        builder = builder.headers(header);

        let host = builder.auth.headers.get("HOST");
        assert!(host.is_some());

        let host = host.unwrap();
        assert_eq!(host.to_str().unwrap(), "127.0.0.1");
    }

    #[test]
    fn test_insert_header() {
        let mut builder = AuthBuilder::default();
        builder = builder.header_insert("Content-Type", "application/json".parse().unwrap());

        assert_eq!(builder.auth.headers.len(), 1);
        assert!(builder.auth.headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_clear() {
        let mut builder = AuthBuilder::default();
        builder = builder.header_insert("Content-Type", "application/json".parse().unwrap());
        builder = builder.header_clear();

        assert_eq!(builder.auth.headers.len(), 0);
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
            .header_insert("Content-Type", "foo6".try_into().unwrap())
            .type_with_header();

        let auth = builder.auth;

        let header = auth.get_original_header();
        assert_eq!(
            header.get("Content-Type").unwrap().to_str().unwrap(),
            "foo6"
        );

        let key = auth.get_header_key().unwrap();
        let secret = auth.get_header_secret().unwrap();
        let verb = auth.get_header_verb().unwrap();
        let md5 = auth.get_header_md5().unwrap();
        let content_type = auth.get_header_content_type().unwrap();
        let date = auth.get_header_date().unwrap();
        let resource = auth.get_header_resource().unwrap();

        assert_eq!(key, HeaderValue::from_bytes(b"foo1").unwrap());
        assert_eq!(secret, HeaderValue::from_bytes(b"foo2").unwrap());
        assert_eq!(verb, HeaderValue::from_bytes(b"POST").unwrap());
        assert!(matches!(md5, Some(v) if v==HeaderValue::from_bytes(b"foo4").unwrap()));
        assert!(matches!(content_type, Some(v) if v==HeaderValue::from_bytes(b"foo6").unwrap()));
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
        .header_insert("Content-Type", "foo6".try_into().unwrap())
        //.type_with_header()
        ;

        let auth = builder.auth;

        let md5 = auth.get_header_md5().unwrap();
        let content_type = auth.get_header_content_type().unwrap();

        assert!(matches!(md5, None));
        assert!(matches!(content_type, None));
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
    auth.expect_get_header_content_type()
        .times(1)
        .returning(|| {
            let val: HeaderValue = "foo5".parse().unwrap();
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
    use http::header::{HeaderMap, HeaderValue};
    use mockall::mock;
    use std::borrow::Cow;

    use crate::{
        auth::{
            AuthSignString, AuthToHeaderMap, AuthToOssHeader, MockHeaderToSign, OssHeader,
            SignString,
        },
        errors::OssResult,
        types::{CanonicalizedResource, ContentMd5, ContentType, Date, KeyId, KeySecret},
    };

    #[test]
    fn test_from_auth() {
        mock! {
            Bar{}

            impl AuthSignString for Bar{
                fn key(&self) -> Cow<'_, KeyId>;
                fn secret(&self) -> Cow<'_, KeySecret>;
                fn verb(&self) -> String;
                fn content_md5(&self) -> Cow<'_, ContentMd5>;
                fn content_type(&self) -> Cow<'_, ContentType>;
                fn date(&self) -> Cow<'_, Date>;
                fn canonicalized_resource(&self) -> Cow<'_, CanonicalizedResource>;
            }

            impl AuthToOssHeader for Bar{
                fn to_oss_header(&self) -> OssResult<OssHeader>;
            }

            impl AuthToHeaderMap for Bar{
                fn get_original_header(&self) -> HeaderMap;
                fn get_header_key(&self) -> OssResult<HeaderValue>;
                fn get_header_secret(&self) -> OssResult<HeaderValue>;
                fn get_header_verb(&self) -> OssResult<HeaderValue>;
                fn get_header_md5(&self) -> OssResult<Option<HeaderValue>>;
                fn get_header_content_type(&self) -> OssResult<Option<HeaderValue>>;
                fn get_header_date(&self) -> OssResult<HeaderValue>;
                fn get_header_resource(&self) -> OssResult<HeaderValue>;
            }
        }

        let mut auth = MockBar::new();

        auth.expect_key()
            .times(1)
            .returning(|| Cow::Owned(KeyId::new("foo1")));
        auth.expect_secret()
            .times(1)
            .returning(|| Cow::Owned(KeySecret::new("foo2")));

        auth.expect_verb().times(1).returning(|| "GET".to_string());

        auth.expect_content_md5()
            .times(1)
            .returning(|| Cow::Owned(ContentMd5::new("foo3")));

        auth.expect_content_type()
            .times(1)
            .returning(|| Cow::Owned(ContentType::new("foo4")));

        auth.expect_date()
            .times(1)
            .returning(|| Cow::Owned(Date::new("foo5")));

        auth.expect_canonicalized_resource()
            .times(1)
            .returning(|| Cow::Owned(CanonicalizedResource::new("foo6")));

        let mut header = MockHeaderToSign::new();

        header
            .expect_to_sign_string()
            .times(1)
            .returning(|| "foo7".to_string());

        let res = SignString::from_auth(&auth, header);

        assert!(res.is_ok());
        let val = res.unwrap();
        assert_eq!(val.data(), "GET\nfoo3\nfoo4\nfoo5\nfoo7foo6".to_string());
        assert_eq!(val.key_string(), "foo1".to_string());
        assert_eq!(val.secret_string(), "foo2".to_string());
    }

    #[test]
    fn test_to_sign() {
        let sign_string = SignString::new(
            "bar".to_string(),
            KeyId::from("foo1"),
            KeySecret::from("foo2"),
        );

        let res = sign_string.to_sign();
        assert!(res.is_ok());
        let sign = res.unwrap();
        assert_eq!(sign.data(), "gTzwiN1fRQV90YcecTvo1pH+kI8=".to_string());
        assert_eq!(sign.key_string(), "foo1".to_string());
    }
}

#[test]
fn test_sign_to_headervalue() {
    let sign = Sign::new("foo".to_string(), KeyId::from("bar"));

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
            .header_insert("Content-Type", "foo6".try_into().unwrap())
            .type_with_header();
        let map = builder.auth.get_headers();

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
