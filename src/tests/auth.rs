use std::convert::TryInto;

use reqwest::header::{HeaderMap, HeaderValue};

use crate::{auth::{VERB, AuthHeader, OssHeader, HeaderToSign}, types::{KeyId, KeySecret, CanonicalizedResource, ContentMd5, ContentType}};

#[test]
fn test_verb2string(){
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
fn test_str2verb(){
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
fn test_verb2headervalue(){
    let verb = VERB::GET;
    let header_value: HeaderValue = verb.try_into().unwrap();

    assert_eq!(header_value.to_str().unwrap(), "GET");
}

#[test]
fn test_verb_default(){
    let verb = VERB::default();
    assert_eq!(verb, VERB::GET);
}

mod to_oss_header{
    use std::convert::TryInto;

    use crate::auth::{AuthBuilder, AuthToOssHeader};

    #[test]
    fn test_none(){
        let builder = AuthBuilder::default();
        let header = builder.auth.to_oss_header();
        assert!(header.is_ok());
        let header = header.unwrap();
        assert!(header.is_none());

        let builder = AuthBuilder::default()
            .header_insert("abc", "def".try_into().unwrap());
        let header = builder.auth.to_oss_header();
        assert!(header.is_ok());
        let header = header.unwrap();
        assert!(header.is_none());
    }

    #[test]
    fn test_some(){
        let builder = AuthBuilder::default()
            .header_insert("x-oss-foo", "bar".try_into().unwrap())
            .header_insert("x-oss-ffoo", "barbar".try_into().unwrap())
            .header_insert("fffoo", "aabb".try_into().unwrap())
            ;
        let header = builder.auth.to_oss_header();
        assert!(header.is_ok());
        let header = header.unwrap();
        let header: String = header.into();
        assert_eq!(&header, "x-oss-ffoo:barbar\nx-oss-foo:bar\n");
    }
}

// TODO TEST get_headers()



mod auth_builder{
    use std::convert::TryInto;

    use chrono::{Utc, TimeZone};
    use http::{header::{HOST, CONTENT_TYPE}, HeaderMap};

    use crate::{auth::{AuthBuilder, VERB, Auth}, types::{KeySecret, CanonicalizedResource}};

    #[test]
    fn test_key(){
        let mut builder = AuthBuilder::default();
        builder = builder.key("foo1".to_owned());

        assert_eq!(builder.auth.access_key_id.as_ref(), "foo1");
    }

    #[test]
    fn test_secret(){
        let mut builder = AuthBuilder::default();
        builder = builder.secret("foo2".to_owned());

        assert_eq!(builder.auth.access_key_secret.as_ref(), "foo2");
    }

    #[test]
    fn test_verb(){
        let mut builder = AuthBuilder::default();
        builder = builder.verb("POST");

        assert!(matches!(builder.auth.verb, VERB::POST));
    }

    #[test]
    fn test_content_md5(){
        let mut builder = AuthBuilder::default();
        builder = builder.content_md5("abc3".to_owned());

        assert!(matches!(builder.auth.content_md5, Some(v) if v.as_ref()=="abc3"));
    }

    #[test]
    fn test_date(){
        let mut builder = AuthBuilder::default();
        let date = Utc.ymd(2022, 1, 1).and_hms(18, 1, 1);
        builder = builder.date(date);

        assert_eq!(builder.auth.date.as_ref(), "Sat, 01 Jan 2022 18:01:01 GMT");
    }

    #[test]
    fn test_canonicalized_resource(){
        let mut builder = AuthBuilder::default();
        builder = builder.canonicalized_resource("foo323".to_string());

        assert_eq!(builder.auth.canonicalized_resource.as_ref(), "foo323");
    }

    #[test]
    fn test_type_with_header(){
        let mut builder = AuthBuilder::default();
        let auth = Auth{
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
    fn test_header(){
        let mut builder = AuthBuilder::default();
        let mut header = HeaderMap::new();
        header.insert(HOST, "127.0.0.1".try_into().unwrap());
        builder = builder.headers(header);

        let host = builder.auth.headers.get("HOST");
        assert!(host.is_some());

        let host = host.unwrap();
        assert_eq!(host.to_str().unwrap(), "127.0.0.1");

        let content_type = builder.auth.content_type;
        assert!(content_type.is_none());

        let mut builder2 = AuthBuilder::default();
        let mut header2 = HeaderMap::new();
        header2.insert(HOST, "127.0.0.1".try_into().unwrap());
        header2.insert(CONTENT_TYPE, "bar".try_into().unwrap());
        builder2 = builder2.headers(header2);

        assert!(matches!(builder2.auth.content_type, Some(v) if v.as_ref()=="bar"));
    }

    #[test]
    fn test_insert_header(){
        let mut builder = AuthBuilder::default();
        builder = builder.header_insert("Content-Type", "application/json".parse().unwrap());

        assert_eq!(builder.auth.headers.len(), 1);
        assert!(builder.auth.headers.contains_key("Content-Type"));
    }

    #[test]
    fn test_clear(){
        let mut builder = AuthBuilder::default();
        builder = builder.header_insert("Content-Type", "application/json".parse().unwrap());
        builder = builder.header_clear();

        assert_eq!(builder.auth.headers.len(), 0);
    }

    // #[test]
    // TODO 
    // fn test_get_headers(){
    //     #[mockall_double::double]
    //     use crate::auth::Auth;
    //     let mut auth = MockAuth::default();
    //     auth.expect_get_headers().times(1).returning(|| {
    //         let mut headers = HeaderMap::new();
    //         headers.insert(HOST, "example.com".parse().unwrap());
    //         Ok(headers)
    //     });

    //     let builder = AuthBuilder{
    //         auth,
    //     };

    // }
}

#[test]
fn header_map_from_auth(){
    let auth = crate::auth::Auth{
        access_key_id: KeyId::from_static("foo_key"),
        access_key_secret: KeySecret::from_static("foo_secret"),
        verb: VERB::GET,
        content_md5: None,
        content_type: Some(ContentType::from_static("text/plain")),
        date: "Sat, 03 Sep 2022 16:04:47 GMT".into(),
        canonicalized_resource: CanonicalizedResource::from_static(""),
        headers: HeaderMap::new(),
    };

    let headers = HeaderMap::from_auth(&auth);

    assert!(headers.is_ok());

    let header = headers.unwrap();

    assert_eq!(header.get("AccessKeyId"), Some(&HeaderValue::from_str("foo_key").unwrap()));
    assert_eq!(header.get("SecretAccessKey"), Some(&HeaderValue::from_str("foo_secret").unwrap()));
    assert_eq!(header.get("VERB"), Some(&HeaderValue::from_str("GET").unwrap()));
    assert_eq!(header.get("Content-MD5"), None);
    assert_eq!(header.get("Content-Type"), Some(&HeaderValue::from_str("text/plain").unwrap()));
    assert_eq!(header.get("CanonicalizedResource"), Some(&HeaderValue::from_str("").unwrap()));
    assert_eq!(header.get("date"), Some(&HeaderValue::from_str("Sat, 03 Sep 2022 16:04:47 GMT").unwrap()));

    let auth = crate::auth::Auth{
        access_key_id: KeyId::from_static("foo_key"),
        access_key_secret: "foo_secret".to_owned().into(),
        verb: VERB::GET,
        content_md5: Some(ContentMd5::from_static("bar")),
        content_type: Some(ContentType::from_static("text/plain")),
        date: "Sat, 03 Sep 2022 16:04:47 GMT".into(),
        canonicalized_resource: CanonicalizedResource::new(""),
        headers: HeaderMap::new(),
    };

    let headers = HeaderMap::from_auth(&auth);

    assert!(headers.is_ok());

    let header = headers.unwrap();

    assert_eq!(header.get("Content-MD5"), Some(&HeaderValue::from_str("bar").unwrap()));
}

#[test]
fn to_sign_string(){
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
fn header_into_string(){
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

mod sign_string_struct{
    use crate::auth::AuthBuilder;

    #[test]
    fn test_new(){
        let auth_builder = AuthBuilder::default();
    }
}