use reqwest::header::{HeaderMap, HeaderValue};

use crate::auth::VERB;


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
