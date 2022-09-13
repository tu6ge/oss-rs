use reqwest::header::{HeaderMap, HeaderValue};

use crate::{auth::{VERB, self}, errors::OssError};

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


#[tokio::test]
async fn test_async_get_headers(){
    
    let auth = crate::auth::Auth{
        access_key_id: "foo_key",
        access_key_secret: "foo_secret",
        verb: VERB::GET,
        content_md5: None,
        content_type: Some("text/plain".into()),
        date: "Sat, 03 Sep 2022 16:04:47 GMT".into(),
        canonicalized_resource: "",
        headers: HeaderMap::new(),
    };

    let headers = auth.async_get_headers().await;

    assert!(headers.is_ok());

    let header = headers.unwrap();

    assert_eq!(header.get("AccessKeyId"), Some(&HeaderValue::from_str("foo_key").unwrap()));
    assert_eq!(header.get("SecretAccessKey"), Some(&HeaderValue::from_str("foo_secret").unwrap()));
    assert_eq!(header.get("VERB"), Some(&HeaderValue::from_str("GET").unwrap()));
    assert_eq!(header.get("Content-MD5"), None);
    assert_eq!(header.get("Content-Type"), Some(&HeaderValue::from_str("text/plain").unwrap()));
    assert_eq!(header.get("CanonicalizedResource"), Some(&HeaderValue::from_str("").unwrap()));
    assert_eq!(header.get("date"), Some(&HeaderValue::from_str("Sat, 03 Sep 2022 16:04:47 GMT").unwrap()));
    assert_eq!(header.get("Authorization"), Some(&HeaderValue::from_str("OSS foo_key:BoUvtc18Dc2q21W+sINIWidt+SE=").unwrap()));

    let auth = crate::auth::Auth{
        access_key_id: "foo_key",
        access_key_secret: "foo_secret",
        verb: VERB::GET,
        content_md5: Some("bar"),
        content_type: Some("text/plain".into()),
        date: "Sat, 03 Sep 2022 16:04:47 GMT".into(),
        canonicalized_resource: "",
        headers: HeaderMap::new(),
    };

    let headers = auth.async_get_headers().await;

    assert!(headers.is_ok());

    let header = headers.unwrap();

    assert_eq!(header.get("Content-MD5"), Some(&HeaderValue::from_str("bar").unwrap()));
}

#[tokio::test]
async fn test_sign(){
    let auth = crate::auth::Auth{
        access_key_id: "foo_key",
        access_key_secret: "foo_secret",
        verb: VERB::GET,
        content_md5: None,
        content_type: Some("text/plain".into()),
        date: "Sat, 03 Sep 2022 16:04:47 GMT".into(),
        canonicalized_resource: "",
        headers: HeaderMap::new(),
    };

    let sign = auth.sign();

    assert!(sign.is_ok());

    assert_eq!(sign.unwrap(), "BoUvtc18Dc2q21W+sINIWidt+SE=".to_string());

    let mut headers = HeaderMap::new();
    headers.insert("x-oss-test", auth::to_value("Bearer xxx").unwrap());

    let auth = crate::auth::Auth{
        access_key_id: "foo_key",
        access_key_secret: "foo_secret",
        verb: VERB::GET,
        content_md5: Some("bar_md5"),
        content_type: Some("text/plain".into()),
        date: "Sat, 03 Sep 2022 16:04:47 GMT".into(),
        canonicalized_resource: "",
        headers: headers,
    };

    let sign = auth.sign();

    assert!(sign.is_ok());

    assert_eq!(sign.unwrap(), "dHqpW+ZVuUBDMvb/4hnrxj+cniY=".to_string());
}

mod header_str{
    use reqwest::header::{HeaderMap, HOST, HeaderValue};
    use crate::auth::VERB;

    #[test]
    fn test_none(){
        let auth = crate::auth::Auth{
            headers: HeaderMap::new(),
            verb: VERB::GET,
            ..Default::default()
        };
    
        assert!(auth.header_str().is_ok());
        assert!(auth.header_str().unwrap().is_none());
    }

    #[test]
    fn test_other_header_key(){
        let mut headers = HeaderMap::new();
        headers.insert(
            HOST,
            HeaderValue::from_str("test_value").unwrap()
        );
        let auth2 = crate::auth::Auth{
            headers: headers,
            verb: VERB::GET,
            ..Default::default()
        };

        assert!(auth2.header_str().is_ok());
        assert!(auth2.header_str().unwrap().is_none());
    }

    #[test]
    fn test_oss(){
        let mut headers = HeaderMap::new();
        headers.insert(
            HOST,
            HeaderValue::from_str("test_value").unwrap()
        );
        headers.insert(
            "x-oss-test",
            HeaderValue::from_str("oss_test_value").unwrap()
        );
        let auth2 = crate::auth::Auth{
            headers: headers,
            verb: VERB::GET,
            ..Default::default()
        };

        assert!(auth2.header_str().is_ok());
        assert!(auth2.header_str().unwrap().is_some());
        assert_eq!(auth2.header_str().unwrap().unwrap(), "x-oss-test:oss_test_value".to_string());
    }
}

#[test]
fn test_to_value(){
    let value = auth::to_value("\n");

    assert!(value.is_err());

    let value_inner = value.unwrap_err();

    assert!(matches!(value_inner, OssError::Input(s) if s=="invalid HeaderValue".to_string()));
}

#[test]
fn test_string_to_value(){
    let value = auth::string_to_value("\n".to_string());

    assert!(value.is_err());

    let value_inner = value.unwrap_err();

    assert!(matches!(value_inner, OssError::Input(s) if s=="invalid HeaderValue".to_string()));
}


mod auth_builder{
    use crate::auth::AuthBuilder;


    #[test]
    fn test_content_md5(){
        let mut builder = AuthBuilder::default();
        builder = builder.content_md5("abc3");

        assert_eq!(builder.auth.content_md5, Some("abc3"));
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
}