use std::{convert::TryInto, error::Error};

use http::{
    header::{HeaderMap, HeaderValue, InvalidHeaderValue},
    Method,
};
use mockall::mock;

use crate::types::{Date, KeyId};

use super::{
    AppendAuthHeader, AuthBuilder, AuthError, AuthErrorKind, AuthHeader, MockAuthToHeaderMap,
    OssHeader, Sign, SignString,
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

    use crate::auth::{AuthBuilder, InnerAuth};

    #[test]
    fn auth_to_sign_string() {
        use http::header::CONTENT_TYPE;
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

        let InnerAuth {
            access_key_id,
            access_key_secret,
            method,
            content_md5,
            headers,
            date,
            canonicalized_resource,
            ..
        } = auth;

        assert_eq!(access_key_id.as_ref(), "foo1");

        assert_eq!(access_key_secret.as_str(), "foo2");

        assert_eq!(method.to_string(), "POST".to_owned());

        assert_eq!(content_md5.unwrap().as_ref(), "foo4");

        assert_eq!(headers.get(CONTENT_TYPE).unwrap().as_bytes(), b"foo6");

        assert_eq!(date.as_ref(), "Sat, 01 Jan 2022 18:01:01 GMT");

        assert_eq!(canonicalized_resource.as_ref(), "foo5");
    }

    #[test]
    fn auth_to_sign_string_none() {
        use http::header::CONTENT_TYPE;
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

        let InnerAuth {
            access_key_id,
            access_key_secret,
            method,
            content_md5,
            headers,
            date,
            canonicalized_resource,
            ..
        } = auth;

        assert_eq!(access_key_id.as_ref(), "foo1");

        assert_eq!(access_key_secret.as_str(), "foo2");

        assert_eq!(method.to_string(), "POST".to_owned());

        assert!(content_md5.is_none());

        assert_eq!(headers.get(CONTENT_TYPE).unwrap().as_bytes(), b"foo6");

        assert_eq!(date.as_ref(), "Sat, 01 Jan 2022 18:01:01 GMT");

        assert_eq!(canonicalized_resource.as_ref(), "foo5");
    }
}

mod auth_builder {
    use std::convert::TryInto;

    use chrono::{TimeZone, Utc};
    use http::{header::HOST, HeaderMap, Method};

    use crate::auth::AuthBuilder;

    #[test]
    fn test_key() {
        let mut builder = AuthBuilder::default();
        builder.key("foo1");
        let auth = builder.build();

        assert_eq!(auth.access_key_id.as_ref(), "foo1");
    }

    #[test]
    fn test_secret() {
        let mut builder = AuthBuilder::default();
        builder.secret("foo2");
        let auth = builder.build();

        assert_eq!(auth.access_key_secret.as_str(), "foo2");
    }

    #[test]
    fn test_verb() {
        let mut builder = AuthBuilder::default();
        builder.method(&Method::POST);
        let auth = builder.build();

        assert_eq!(auth.method, &Method::POST);
    }

    #[test]
    fn test_content_md5() {
        let mut builder = AuthBuilder::default();
        builder.content_md5("abc3");
        let auth = builder.build();

        assert_eq!(auth.content_md5.unwrap().as_ref(), "abc3");
    }

    #[test]
    fn test_date() {
        let mut builder = AuthBuilder::default();
        let date = Utc.with_ymd_and_hms(2022, 1, 1, 18, 1, 1).unwrap();
        builder.date(date);
        let auth = builder.build();

        assert_eq!(auth.date.as_ref(), "Sat, 01 Jan 2022 18:01:01 GMT");
    }

    #[test]
    fn test_canonicalized_resource() {
        let mut builder = AuthBuilder::default();
        builder.canonicalized_resource("foo323");
        let auth = builder.build();

        assert_eq!(auth.canonicalized_resource.as_ref(), "foo323");
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
        //let secret = auth.get_header_secret().unwrap();
        let verb = auth.get_header_method().unwrap();
        let md5 = auth.get_header_md5().unwrap();
        let date = auth.get_header_date().unwrap();
        let resource = auth.get_header_resource().unwrap();

        assert_eq!(key, HeaderValue::from_bytes(b"foo1").unwrap());
        //assert_eq!(secret, HeaderValue::from_bytes(b"foo2").unwrap());
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
    // auth.expect_get_header_secret()
    //     .times(1)
    //     .returning(|| Ok("foo2".parse().unwrap()));
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
    assert!(map.get("SecretAccessKey").is_none());
}

#[test]
fn test_append_auth() {
    let mut builder = AuthBuilder::default();
    builder.key("foo1");
    builder.secret("foo2");
    builder.method(&Method::POST);
    builder.content_md5("foo4");
    builder.date(unsafe { Date::from_static("foo_date") });
    builder.canonicalized_resource("foo5");
    builder.header_insert("Content-Type", "foo6".try_into().unwrap());
    let auth = builder.build();

    let mut map = HeaderMap::new();
    map.append_auth(&auth).unwrap();

    assert_eq!(map.len(), 6);
    assert_eq!(map.get("Content-Type").unwrap(), &"foo6");
    assert_eq!(map.get("accesskeyid").unwrap(), &"foo1");
    assert!(map.get("secretaccesskey").is_none());
    assert_eq!(map.get("verb").unwrap(), &"POST");
    assert_eq!(map.get("content-md5").unwrap(), &"foo4");
    assert_eq!(map.get("date").unwrap(), &"foo_date");
    assert_eq!(map.get("canonicalizedresource").unwrap(), &"foo5");
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
    use crate::{
        auth::SignString,
        types::{KeyId, KeySecret},
    };

    #[test]
    fn test_into_sign() {
        let key = KeyId::from("foo1");
        let secret = KeySecret::from("foo2");
        let sign_string = SignString::new("bar", key, secret);

        let res = sign_string.into_sign();
        assert!(res.is_ok());
        let sign = res.unwrap();
        assert_eq!(sign.data(), "gTzwiN1fRQV90YcecTvo1pH+kI8=");
        assert_eq!(sign.key_string(), "foo1".to_string());
    }
}

#[test]
fn test_sign_string_debug() {
    let sign = SignString::new("abc", "key".into(), "secret".into());

    assert_eq!(
        format!("{sign:?}"),
        "SignString { data: \"abc\", key: InnerKeyId(\"key\"), secret: KeySecret }"
    );
}

#[test]
fn test_sign_debug() {
    let sign = Sign::new("abc", "key".into());
    assert_eq!(
        format!("{sign:?}"),
        "Sign { data: \"abc\", key: InnerKeyId(\"key\") }"
    );
}

#[test]
fn test_sign_to_headervalue() {
    let key = KeyId::from("bar");
    let sign = Sign::new("foo", key);

    let val: HeaderValue = sign.try_into().unwrap();
    assert_eq!(val.to_str().unwrap(), "OSS bar:foo");
}

mod get_headers {
    use http::{HeaderMap, Method};

    use crate::{auth::AuthBuilder, types::Date};

    /// 集成测试，其他的都是单元测试
    #[test]
    fn test_get_headers() {
        let mut builder = AuthBuilder::default();
        builder.key("foo1");
        builder.secret("foo2");
        builder.method(&Method::POST);
        builder.content_md5("foo4");
        builder.date(unsafe { Date::from_static("foo_date") });
        builder.canonicalized_resource("foo5");
        builder.header_insert("Content-Type", "foo6".try_into().unwrap());
        let map = builder.build().get_headers();

        assert!(map.is_ok());
        let map = map.unwrap();
        assert_eq!(map.len(), 7);
        assert_eq!(map.get("Content-Type").unwrap(), &"foo6");
        assert_eq!(map.get("accesskeyid").unwrap(), &"foo1");
        assert!(map.get("secretaccesskey").is_none());
        assert_eq!(map.get("verb").unwrap(), &"POST");
        assert_eq!(map.get("content-md5").unwrap(), &"foo4");
        assert_eq!(map.get("date").unwrap(), &"foo_date");
        assert_eq!(map.get("canonicalizedresource").unwrap(), &"foo5");
        assert_eq!(
            map.get("authorization").unwrap(),
            &"OSS foo1:67qpyspFaWOYrWwahWKgNN+ngUY="
        );
    }

    #[test]
    fn test_append_headers() {
        let mut builder = AuthBuilder::default();
        builder.key("foo1");
        builder.secret("foo2");
        builder.method(&Method::POST);
        builder.content_md5("foo4");
        builder.date(unsafe { Date::from_static("foo_date") });
        builder.canonicalized_resource("foo5");
        builder.header_insert("Content-Type", "foo6".try_into().unwrap());
        let auth = builder.build();

        let mut map = HeaderMap::new();
        auth.append_headers(&mut map).unwrap();

        assert_eq!(map.len(), 7);
        assert_eq!(map.get("Content-Type").unwrap(), &"foo6");
        assert_eq!(map.get("accesskeyid").unwrap(), &"foo1");
        assert!(map.get("secretaccesskey").is_none());
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

#[test]
fn oss_header_to_string() {
    let header = OssHeader::new(Some("foo7".to_string()));
    assert_eq!(header.to_string(), "foo7\n".to_string());

    let header = OssHeader::new(None);

    assert_eq!(header.to_string(), "".to_string());
}

#[test]
fn test_error_display() {
    let val = HeaderValue::from_str("\n");
    let header_error = val.unwrap_err();
    let err = AuthError {
        kind: AuthErrorKind::HeaderValue(header_error),
    };
    assert_eq!(format!("{}", err), "failed to parse header value");

    let err = AuthError {
        kind: AuthErrorKind::Hmac(hmac::digest::crypto_common::InvalidLength {}),
    };
    assert_eq!(format!("{}", err), "invalid aliyun secret length");

    let err = AuthError {
        kind: AuthErrorKind::InvalidCanonicalizedResource,
    };
    assert_eq!(format!("{}", err), "invalid canonicalized-resource");
}

#[test]
fn test_error_source() {
    let val = HeaderValue::from_str("\n");
    let header_error = val.unwrap_err();
    let err = AuthError {
        kind: AuthErrorKind::HeaderValue(header_error),
    };
    assert_eq!(
        format!("{}", err.source().unwrap()),
        "failed to parse header value"
    );

    let err = AuthError {
        kind: AuthErrorKind::Hmac(hmac::digest::crypto_common::InvalidLength {}),
    };
    assert_eq!(format!("{}", err.source().unwrap()), "Invalid Length");

    let err = AuthError {
        kind: AuthErrorKind::InvalidCanonicalizedResource,
    };
    assert!(err.source().is_none());
}

mod with_oss {
    use http::{HeaderValue, Method};

    use crate::auth::{AuthError, AuthErrorKind, RequestWithOSS};

    #[test]
    fn test() {
        let client = reqwest::Client::default();
        let mut request = client
            .request(
                Method::GET,
                "https://foo.oss-cn-shanghai.aliyuncs.com/?bucketInfo",
            )
            .build()
            .unwrap();

        request.with_oss("key1".into(), "secret2".into()).unwrap();
        let header = request.headers();

        assert!(header.len() == 5);

        assert_eq!(
            header.get("accesskeyid").unwrap(),
            "key1".parse::<HeaderValue>().unwrap()
        );
        assert!(header.get("secretaccesskey").is_none());
        assert_eq!(
            header.get("verb").unwrap(),
            "GET".parse::<HeaderValue>().unwrap()
        );
        assert_eq!(
            header.get("canonicalizedresource").unwrap(),
            "/foo/?bucketInfo".parse::<HeaderValue>().unwrap()
        );
        assert!(header.get("date").is_some());
        assert!(header.get("authorization").is_some());
    }

    #[test]
    fn test_err() {
        let client = reqwest::Client::default();
        let mut request = client
            .request(
                Method::GET,
                "https://foo.oss-cn-shanghai.aliyunxx.com/?bucketInfo",
            )
            .build()
            .unwrap();

        let err = request
            .with_oss("key1".into(), "secret2".into())
            .unwrap_err();

        assert!(matches!(
            err,
            AuthError {
                kind: AuthErrorKind::InvalidCanonicalizedResource,
            }
        ));
    }
}

mod tests_canonicalized_resource {
    use http::Method;
    use reqwest::Url;

    use crate::{auth::OssHost, types::CanonicalizedResource, BucketName};

    use super::super::GenCanonicalizedResource;

    #[test]
    fn test_canonicalized_resource() {
        let url: Url = "https://oss2.aliyuncs.com".parse().unwrap();
        assert_eq!(url.canonicalized_resource(), None);
        let url: Url = "https://oss-cn-qingdao.aliyuncs.com".parse().unwrap();
        assert_eq!(
            url.canonicalized_resource(),
            Some(CanonicalizedResource::default())
        );

        let url: Url = "https://abc.oss-cn-qingdao.aliyuncs.com?bucketInfo"
            .parse()
            .unwrap();
        assert_eq!(
            url.canonicalized_resource(),
            Some(CanonicalizedResource::new("/abc/?bucketInfo"))
        );

        let url: Url = "https://abc.oss-cn-qingdao.aliyuncs.com?list-type=2&continuation-token=foo"
            .parse()
            .unwrap();
        assert_eq!(
            url.canonicalized_resource(),
            Some(CanonicalizedResource::new("/abc/?continuation-token=foo"))
        );

        let url: Url =
            "https://abc.oss-cn-qingdao.aliyuncs.com?continuation-token=foo&abc=def&list-type=2"
                .parse()
                .unwrap();
        assert_eq!(
            url.canonicalized_resource(),
            Some(CanonicalizedResource::new("/abc/?continuation-token=foo"))
        );

        let url: Url = "https://abc.oss-cn-qingdao.aliyuncs.com/path1"
            .parse()
            .unwrap();
        assert_eq!(
            url.canonicalized_resource(),
            Some(CanonicalizedResource::new("/abc/path1"))
        );
    }

    #[test]
    fn test_oss_host() {
        let url: Url = "https://192.168.3.10/path1?delimiter=5".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::None);

        let url: Url = "https://example.com/path1?delimiter=5".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::None);

        let url: Url = "https://aliyuncs.com".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::None);

        let url: Url = "https://oss-cn-qingdao.aliyuncs.com".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::EndPoint);

        let url: Url = "https://oss-abc.aliyuncs.com".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::EndPoint);

        let url: Url = "https://abc.aliyuncs.com".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::None);

        let url: Url = "https://abc.oss-cn-qingdao.aliyuncs.com".parse().unwrap();
        assert_eq!(
            url.oss_host(),
            OssHost::Bucket(BucketName::new("abc").unwrap())
        );
        let url: Url = "https://abc-.oss-cn-qingdao.aliyuncs.com".parse().unwrap();
        assert_eq!(url.oss_host(), OssHost::None);
    }

    #[test]
    fn test_object_list_resource() {
        let url: Url = "https://example.com/path1?delimiter=5".parse().unwrap();
        let bucket = "abc".parse().unwrap();
        let resource = url.object_list_resource(&bucket);
        assert!(resource == "/abc/");

        let url: Url = "https://example.com/path1?continuation-token=bar&delimiter=5"
            .parse()
            .unwrap();
        let bucket = "abc".parse().unwrap();
        let resource = url.object_list_resource(&bucket);
        assert!(resource == "/abc/?continuation-token=bar");
    }

    #[test]
    fn test_object_path() {
        let url: Url = "https://example.com/path1".parse().unwrap();
        assert_eq!(url.object_path().unwrap(), "path1");

        let url: Url = "https://example.com/path1/object2".parse().unwrap();
        assert_eq!(url.object_path().unwrap(), "path1/object2");

        let url: Url = "https://example.com/路径/object2".parse().unwrap();
        assert_eq!(url.object_path().unwrap(), "路径/object2");

        let url: Url = "https://example.com/path1/object2?foo=bar".parse().unwrap();
        assert_eq!(url.object_path().unwrap(), "path1/object2");

        let url: Url = "https://example.com/path1/".parse().unwrap();
        assert!(url.object_path().is_none());
    }

    #[test]
    fn test_request() {
        let client = reqwest::Client::default();
        let request = client
            .request(
                Method::GET,
                "https://foo.oss-cn-shanghai.aliyuncs.com/?bucketInfo",
            )
            .build()
            .unwrap();

        assert_eq!(
            request.canonicalized_resource(),
            request.url().canonicalized_resource(),
        );
        assert_eq!(request.oss_host(), request.url().oss_host(),);

        let bucket = "abc".parse().unwrap();
        assert_eq!(
            request.object_list_resource(&bucket),
            request.url().object_list_resource(&bucket),
        );
        assert_eq!(request.object_path(), request.url().object_path(),);
    }
}
