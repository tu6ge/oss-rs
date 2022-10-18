use http::{HeaderMap, HeaderValue};
use reqwest::Url;

use crate::{client::{Client}, types::CanonicalizedResource, EndPoint};

#[test]
#[cfg(not(feature = "plugin"))]
fn init_client_without_plugin(){
    use crate::{client, EndPoint};
    let client = client("foo1", "foo2", EndPoint::CnQingdao, "foo4");

    let buf = [0x10, 0x11, 0x12, 0x13];
    assert!(!client.infer.is_custom(&buf));
}

#[test]
fn set_bucket_name(){
    use crate::client;
    let mut client = client("a","b",EndPoint::CnQingdao,"d".try_into().unwrap());
    client.set_bucket_name("abcaaa".try_into().unwrap());

    assert_eq!(client.get_bucket_base().name(), "abcaaa");
}

mod test_use_plugin{
    #[cfg(feature = "plugin")]
    #[test]
    fn test_install_plugin(){
        use std::sync::Mutex;
        use crate::{client, EndPoint};

        //#[mockall_double::double]
        use crate::plugin::{MockPlugin, MockPluginStore};

        let mut plugin_store = MockPluginStore::new();

        plugin_store.expect_insert().times(1).returning(|_|());
        
        let mut client = client("foo1","foo2",EndPoint::CnQingdao,"foo4".try_into().unwrap());

        client.plugins = Mutex::new(plugin_store);

        let mut plugin = MockPlugin::new();
        plugin.expect_initialize().times(1)
            .returning(|_|Ok(()));
        
        plugin.expect_name().times(0).returning(||"foo_plugin");
      
        let res = client.plugin(Box::new(plugin));
        assert!(res.is_ok());
    }
}


#[test]
fn test_get_bucket_url(){
    // TODO expect is_err()
    // let client = Client::new(
    //     "foo1".into(),
    //     "foo2".into(),
    //     "qingdao".into(),
    //     "foo4".into()
    // );
    // let result = client.get_bucket_url();
    // assert!(result.is_err());

    let client = Client::new(
        "foo1".to_owned().into(),
        "foo2".to_owned().into(),
        EndPoint::CnQingdao,
        "foo4".try_into().unwrap()
    );
    let url = client.get_bucket_url();
    assert_eq!(url.as_str(), "https://foo4.oss-cn-qingdao.aliyuncs.com/");
}

#[tokio::test]
async fn test_builder_with_header(){
    let client = Client::new("foo1".into(), "foo2".into(), EndPoint::CnQingdao, "foo4".try_into().unwrap());
    let url = Url::parse("http://foo.example.net/foo").unwrap();
    let resource = CanonicalizedResource::new("bar");
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let builder = client.builder_with_header("POST".into(), &url, resource, Some(headers)).await;

    assert!(builder.is_ok());

    let request = builder.unwrap().build().unwrap();
    
    assert_eq!(request.method(), "POST");
    assert!(request.url().host().is_some());
    assert_eq!(request.url().path(), "/foo");
    assert_eq!(request.headers().get("content-type"), Some(&HeaderValue::from_str("application/json").unwrap()));
    assert_eq!(request.headers().get("accesskeyid"), Some(&HeaderValue::from_str("foo1").unwrap()));
    assert_eq!(request.headers().get("secretaccesskey"), Some(&HeaderValue::from_str("foo2").unwrap()));
    assert_eq!(request.headers().get("verb"), Some(&HeaderValue::from_str("POST").unwrap()));
    assert_eq!(request.headers().get("date"), Some(&HeaderValue::from_str("Thu, 06 Oct 2022 20:40:00 GMT").unwrap()));
    assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("bar").unwrap()));
    assert_eq!(request.headers().get("authorization"), Some(&HeaderValue::from_str("OSS foo1:FUrk4hgj2yIB8lJpnsSub+CTC9M=").unwrap()));
}


#[cfg(feature = "blocking")]
#[test]
fn test_blocking_builder_with_header(){
    use crate::blocking::client::Client;
    let client = Client::new("foo1".into(), "foo2".into(), EndPoint::CnQingdao, "foo4".try_into().unwrap());
    let url = Url::parse("http://foo.example.net/foo").unwrap();
    let resource = CanonicalizedResource::new("bar");
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse().unwrap());
    let builder = client.builder_with_header("POST".into(), &url, resource, Some(headers));

    assert!(builder.is_ok());

    let request = builder.unwrap().build().unwrap();

    assert_eq!(request.method(), "POST");
    assert!(request.url().host().is_some());
    assert_eq!(request.url().path(), "/foo");
    assert_eq!(request.headers().get("content-type"), Some(&HeaderValue::from_str("application/json").unwrap()));
    assert_eq!(request.headers().get("accesskeyid"), Some(&HeaderValue::from_str("foo1").unwrap()));
    assert_eq!(request.headers().get("secretaccesskey"), Some(&HeaderValue::from_str("foo2").unwrap()));
    assert_eq!(request.headers().get("verb"), Some(&HeaderValue::from_str("POST").unwrap()));
    assert_eq!(request.headers().get("date"), Some(&HeaderValue::from_str("Thu, 06 Oct 2022 20:40:00 GMT").unwrap()));
    assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("bar").unwrap()));
    assert_eq!(request.headers().get("authorization"), Some(&HeaderValue::from_str("OSS foo1:FUrk4hgj2yIB8lJpnsSub+CTC9M=").unwrap()));
}


mod handle_error{
    use reqwest::Response;
    use http::Response as HttpResponse;
    use crate::builder::RequestHandler;
    use crate::errors::{OssError, OssService};
    
    #[tokio::test]
    async fn test_async_has_error(){

        let http = HttpResponse::builder()
            .status(302)
            //.header("X-Custom-Foo", "Bar")
            .body(r#"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
            <Error>
                <Code>foo_code</Code>
                <Message>bar</Message>
                <RequestId>63145DB90BFD85303279D56B</RequestId>
                <HostId>honglei123.oss-cn-shanghai.aliyuncs.com</HostId>
                <MaxAllowedSkewMilliseconds>900000</MaxAllowedSkewMilliseconds>
                <RequestTime>2022-09-04T07:11:33.000Z</RequestTime>
                <ServerTime>2022-09-04T08:11:37.000Z</ServerTime>
            </Error>
            "#)
            .unwrap();
        let response: Response = http.into();

        let res = response.handle_error().await;

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(err, OssError::OssService(OssService{code,..}) if code=="foo_code"));

        //mock.checkpoint();
    }

    #[tokio::test]
    async fn test_async_ok(){
        
        let http = HttpResponse::builder()
            .status(200)
            //.header("X-Custom-Foo", "Bar")
            .body("body_abc")
            .unwrap();
        let response: Response = http.into();

        let res = response.handle_error().await;
        assert!(res.is_ok());
        let ok = res.unwrap();
        assert_eq!(ok.status(), 200);
        assert_eq!(ok.text().await.unwrap(), "body_abc".to_string());

        let http = HttpResponse::builder()
            .status(204)
            //.header("X-Custom-Foo", "Bar")
            .body("body_abc")
            .unwrap();
        let response: Response = http.into();

        let res = response.handle_error().await;
        assert!(res.is_ok());
        let ok = res.unwrap();
        assert_eq!(ok.status(), 204);
        assert_eq!(ok.text().await.unwrap(), "body_abc".to_string());

    }

    #[cfg(feature = "blocking")]
    #[test]
    fn test_blocking_has_error(){
        use reqwest::blocking::Response as BlockingResponse;
        use crate::blocking::builder::BlockingReqeustHandler;

        let http = HttpResponse::builder()
            .status(302)
            //.header("X-Custom-Foo", "Bar")
            .body(r#"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
            <Error>
                <Code>foo_code</Code>
                <Message>bar</Message>
                <RequestId>63145DB90BFD85303279D56B</RequestId>
                <HostId>honglei123.oss-cn-shanghai.aliyuncs.com</HostId>
                <MaxAllowedSkewMilliseconds>900000</MaxAllowedSkewMilliseconds>
                <RequestTime>2022-09-04T07:11:33.000Z</RequestTime>
                <ServerTime>2022-09-04T08:11:37.000Z</ServerTime>
            </Error>
            "#)
            .unwrap();
        let response: BlockingResponse = http.into();

        let res = response.handle_error();

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(err, OssError::OssService(OssService{code,..}) if code=="foo_code"));

    }

    #[cfg(feature = "blocking")]
    #[test]
    fn test_blocking_ok(){
        use reqwest::blocking::Response as BlockingResponse;
        use crate::blocking::builder::BlockingReqeustHandler;
        
        let http = HttpResponse::builder()
            .status(200)
            //.header("X-Custom-Foo", "Bar")
            .body("body_abc")
            .unwrap();
        let response: BlockingResponse = http.into();

        let res = response.handle_error();
        assert!(res.is_ok());
        let ok = res.unwrap();
        assert_eq!(ok.status(), 200);
        assert_eq!(ok.text().unwrap(), "body_abc".to_string());

        let http = HttpResponse::builder()
            .status(204)
            //.header("X-Custom-Foo", "Bar")
            .body("body_abc")
            .unwrap();
        let response: BlockingResponse = http.into();

        let res = response.handle_error();
        assert!(res.is_ok());
        let ok = res.unwrap();
        assert_eq!(ok.status(), 204);
        assert_eq!(ok.text().unwrap(), "body_abc".to_string());

    }

}

// blocking mock 有错误
