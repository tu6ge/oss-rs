use std::sync::Arc;

use async_trait::async_trait;
use http::HeaderValue;
use reqwest::{Request, Response, Url};

use crate::{builder::Middleware, errors::{OssResult}, client::Client, types::Query};
use crate::builder::ClientWithMiddleware;

#[tokio::test]
async fn test_get_object_list(){
    struct MyMiddleware{}

    #[async_trait]
    impl Middleware for MyMiddleware{
        async fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(*request.url(), Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/foo4/").unwrap()));
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(r#"<?xml version="1.0" encoding="UTF-8"?>
                <ListBucketResult>
                  <Name>barname</Name>
                  <Prefix></Prefix>
                  <MaxKeys>100</MaxKeys>
                  <Delimiter></Delimiter>
                  <IsTruncated>false</IsTruncated>
                  <Contents>
                    <Key>9AB932LY.jpeg</Key>
                    <LastModified>2022-06-26T09:53:21.000Z</LastModified>
                    <ETag>"F75A15996D0857B16FA31A3B16624C26"</ETag>
                    <Type>Normal</Type>
                    <Size>18027</Size>
                    <StorageClass>Standard</StorageClass>
                  </Contents>
                  <KeyCount>23</KeyCount>
                </ListBucketResult>"#)
                .unwrap();
            let response = Response::from(response);
            Ok(response)
        }
    }

    let client = Client::<ClientWithMiddleware>::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap()
    )
    .middleware(Arc::new(MyMiddleware{}))
    ;

    let mut query = Query::new();
    query.insert("max-keys", "5");
    let res = client.get_object_list(query).await;

    //println!("{:?}", res);
    assert_eq!(format!("{:?}", res), r##"Ok(ObjectList { name: "barname", bucket: BucketBase { endpoint: CnShanghai, name: BucketName("foo4") }, prefix: "", max_keys: 100, key_count: 23, next_continuation_token: None, search_query: Query { inner: {QueryKey("max-keys"): QueryValue("5")} } })"##);
}

#[cfg(feature = "blocking")]
#[test]
fn test_get_blocking_object_list(){
    use crate::blocking::builder::Middleware;
    use reqwest::blocking::{Request, Response};
    use crate::client::ClientRc;
    use std::rc::Rc;

    struct MyMiddleware{}

    impl Middleware for MyMiddleware{
        fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(*request.url(), Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/foo4/").unwrap()));
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(r#"<?xml version="1.0" encoding="UTF-8"?>
                <ListBucketResult>
                  <Name>barname</Name>
                  <Prefix></Prefix>
                  <MaxKeys>100</MaxKeys>
                  <Delimiter></Delimiter>
                  <IsTruncated>false</IsTruncated>
                  <Contents>
                    <Key>9AB932LY.jpeg</Key>
                    <LastModified>2022-06-26T09:53:21.000Z</LastModified>
                    <ETag>"F75A15996D0857B16FA31A3B16624C26"</ETag>
                    <Type>Normal</Type>
                    <Size>18027</Size>
                    <StorageClass>Standard</StorageClass>
                  </Contents>
                  <KeyCount>23</KeyCount>
                </ListBucketResult>"#)
                .unwrap();
            let response = Response::from(response);
            Ok(response)
        }
    }

    let client = ClientRc::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap()
    )
    .middleware(Rc::new(MyMiddleware{}))
    ;

    let mut query = Query::new();
    query.insert("max-keys", "5");
    let res = client.get_object_list(query);

    //println!("{:?}", res);
    assert_eq!(format!("{:?}", res), r##"Ok(ObjectList { name: "barname", bucket: BucketBase { endpoint: CnShanghai, name: BucketName("foo4") }, prefix: "", max_keys: 100, key_count: 23, next_continuation_token: None, search_query: Query { inner: {QueryKey("max-keys"): QueryValue("5")} } })"##);
}

#[tokio::test]
async fn test_put_content_base(){
    struct MyMiddleware{}

    #[async_trait]
    impl Middleware for MyMiddleware{
        async fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "PUT");
            assert_eq!(*request.url(), Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/abc.text").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/foo4/abc.text").unwrap()));
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(r#"content bar"#)
                .unwrap();
            let response = Response::from(response);
            Ok(response)
        }
    }

    let client = Client::<ClientWithMiddleware>::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap()
    )
    .middleware(Arc::new(MyMiddleware{}))
    ;

    let content = String::from("Hello world");
    let content: Vec<u8> = content.into();

    let res = client.put_content_base(content, "application/text", "abc.text").await;

    //println!("{:?}", res);
    assert!(res.is_ok());
}

#[cfg(feature = "blocking")]
#[test]
fn test_blocking_put_content_base(){
    use crate::blocking::builder::Middleware;
    use reqwest::blocking::{Request, Response};
    use crate::client::ClientRc;
    use std::rc::Rc;
    
    struct MyMiddleware{}

    impl Middleware for MyMiddleware{
        fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "PUT");
            assert_eq!(*request.url(), Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/abc.text").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/foo4/abc.text").unwrap()));
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(r#"content bar"#)
                .unwrap();
            let response = Response::from(response);
            Ok(response)
        }
    }

    let client = ClientRc::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap()
    )
    .middleware(Rc::new(MyMiddleware{}))
    ;

    let content = String::from("Hello world");
    let content: Vec<u8> = content.into();

    let res = client.put_content_base(content, "application/text", "abc.text");

    //println!("{:?}", res);
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_delete_object(){
    struct MyMiddleware{}

    #[async_trait]
    impl Middleware for MyMiddleware{
        async fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "DELETE");
            assert_eq!(*request.url(), Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/abc.png").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/foo4/abc.png").unwrap()));
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(r#"<?xml version="1.0" encoding="UTF-8"?>
                <ListBucketResult></ListBucketResult>"#)
                .unwrap();
            let response = Response::from(response);
            Ok(response)
        }
    }

    let client = Client::<ClientWithMiddleware>::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap()
    )
    .middleware(Arc::new(MyMiddleware{}))
    ;

    let res = client.delete_object("abc.png").await;
    //println!("{:?}", res);
    assert!(res.is_ok());
}

#[cfg(feature = "blocking")]
#[test]
fn test_blocking_delete_object(){
    use crate::blocking::builder::Middleware;
    use reqwest::blocking::{Request, Response};
    use crate::client::ClientRc;
    use std::rc::Rc;

    struct MyMiddleware{}

    #[async_trait]
    impl Middleware for MyMiddleware{
        fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "DELETE");
            assert_eq!(*request.url(), Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/abc.png").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/foo4/abc.png").unwrap()));
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(r#"<?xml version="1.0" encoding="UTF-8"?>
                <ListBucketResult></ListBucketResult>"#)
                .unwrap();
            let response = Response::from(response);
            Ok(response)
        }
    }

    let client = ClientRc::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap()
    )
    .middleware(Rc::new(MyMiddleware{}))
    ;

    let res = client.delete_object("abc.png");
    //println!("{:?}", res);
    assert!(res.is_ok());
}
