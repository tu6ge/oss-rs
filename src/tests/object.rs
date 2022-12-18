use crate::builder::{BuilderError, ClientWithMiddleware};
use crate::file::File;
use crate::{builder::Middleware, client::Client};
use async_trait::async_trait;
use http::HeaderValue;
use reqwest::{Request, Response, Url};
use std::sync::Arc;

#[cfg(feature = "blocking")]
#[test]
fn object_list_get_object_list() {
    use crate::client::ClientRc;
    use crate::{blocking::builder::Middleware, builder::RcPointer, object::ObjectList};
    use reqwest::blocking::{Request, Response};
    use std::rc::Rc;

    struct MyMiddleware {}

    impl Middleware for MyMiddleware {
        fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(
                *request.url(),
                Url::parse("https://abc.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5")
                    .unwrap()
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/abc/").unwrap())
            );
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
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
                </ListBucketResult>"#,
                )
                .unwrap();
            let response = Response::from(response);
            Ok(response)
        }
    }

    let client = ClientRc::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap(),
    )
    .middleware(Rc::new(MyMiddleware {}));

    let mut object_list = ObjectList::<RcPointer>::new(
        "abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        String::from("foo2"),
        100,
        200,
        Vec::new(),
        None,
        Rc::new(client),
        vec![("max-keys".into(), 5u8.into())],
    );

    let res = object_list.get_object_list();

    //println!("{:?}", res);
    assert_eq!(
        format!("{:?}", res),
        r##"Ok(ObjectList { bucket: BucketBase { endpoint: CnShanghai, name: BucketName("abc") }, prefix: "", max_keys: 100, key_count: 23, next_continuation_token: None, search_query: Query { inner: {MaxKeys: QueryValue("5")} } })"##
    );
}

#[tokio::test]
async fn test_get_object_list() {
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(
                *request.url(),
                Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5")
                    .unwrap()
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/foo4/").unwrap())
            );
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
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
                </ListBucketResult>"#,
                )
                .unwrap();
            let response = Response::from(response);
            Ok(response)
        }
    }

    let client = Client::<ClientWithMiddleware>::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap(),
    )
    .middleware(Arc::new(MyMiddleware {}));

    let res = client
        .get_object_list(vec![("max-keys".parse().unwrap(), "5".parse().unwrap())])
        .await;

    //println!("{:?}", res);
    assert_eq!(
        format!("{:?}", res),
        r##"Ok(ObjectList { bucket: BucketBase { endpoint: CnShanghai, name: BucketName("foo4") }, prefix: "", max_keys: 100, key_count: 23, next_continuation_token: None, search_query: Query { inner: {MaxKeys: QueryValue("5")} } })"##
    );
}

#[cfg(feature = "blocking")]
#[test]
fn test_get_blocking_object_list() {
    use crate::blocking::builder::Middleware;
    use crate::client::ClientRc;
    use reqwest::blocking::{Request, Response};
    use std::rc::Rc;

    struct MyMiddleware {}

    impl Middleware for MyMiddleware {
        fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(
                *request.url(),
                Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5")
                    .unwrap()
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/foo4/").unwrap())
            );
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
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
                </ListBucketResult>"#,
                )
                .unwrap();
            let response = Response::from(response);
            Ok(response)
        }
    }

    let client = ClientRc::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap(),
    )
    .middleware(Rc::new(MyMiddleware {}));

    let res = client.get_object_list([("max-keys".into(), "5".into())]);

    //println!("{:?}", res);
    assert_eq!(
        format!("{:?}", res),
        r##"Ok(ObjectList { bucket: BucketBase { endpoint: CnShanghai, name: BucketName("foo4") }, prefix: "", max_keys: 100, key_count: 23, next_continuation_token: None, search_query: Query { inner: {MaxKeys: QueryValue("5")} } })"##
    );
}

#[tokio::test]
async fn test_put_content_base() {
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "PUT");
            assert_eq!(
                *request.url(),
                Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/abc.text").unwrap()
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/foo4/abc.text").unwrap())
            );
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
        "foo4".try_into().unwrap(),
    )
    .middleware(Arc::new(MyMiddleware {}));

    let content = String::from("Hello world");
    let content: Vec<u8> = content.into();

    let res = client
        .put_content_base(content, "application/text", "abc.text")
        .await;

    //println!("{:?}", res);
    assert!(res.is_ok());
}

#[cfg(feature = "blocking")]
#[test]
fn test_blocking_put_content_base() {
    use crate::client::ClientRc;
    use crate::{blocking::builder::Middleware, file::blocking::File};
    use reqwest::blocking::{Request, Response};
    use std::rc::Rc;

    struct MyMiddleware {}

    impl Middleware for MyMiddleware {
        fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "PUT");
            assert_eq!(
                *request.url(),
                Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/abc.text").unwrap()
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/foo4/abc.text").unwrap())
            );
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
        "foo4".try_into().unwrap(),
    )
    .middleware(Rc::new(MyMiddleware {}));

    let content = String::from("Hello world");
    let content: Vec<u8> = content.into();

    let res = client.put_content_base(content, "application/text", "abc.text");

    //println!("{:?}", res);
    assert!(res.is_ok());
}

mod get_object {
    use std::sync::Arc;

    use http::HeaderValue;
    use reqwest::{Request, Response, Url};

    use crate::builder::{BuilderError, ClientWithMiddleware};
    use crate::file::File;
    use crate::{builder::Middleware, client::Client};
    use async_trait::async_trait;

    #[tokio::test]
    async fn test_all_range() {
        struct MyMiddleware {}

        #[async_trait]
        impl Middleware for MyMiddleware {
            async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png").unwrap()
                );
                assert_eq!(
                    request.headers().get("canonicalizedresource"),
                    Some(&HeaderValue::from_str("/foo4/foo.png").unwrap())
                );
                assert_eq!(
                    request.headers().get("Range"),
                    Some(&HeaderValue::from_str("bytes=0-").unwrap())
                );
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
            "foo4".try_into().unwrap(),
        )
        .middleware(Arc::new(MyMiddleware {}));

        let res = client.get_object("foo.png", ..).await;

        //println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, String::from("content bar").into_bytes())
    }

    #[tokio::test]
    async fn test_start_range() {
        struct MyMiddleware {}

        #[async_trait]
        impl Middleware for MyMiddleware {
            async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png").unwrap()
                );
                assert_eq!(
                    request.headers().get("canonicalizedresource"),
                    Some(&HeaderValue::from_str("/foo4/foo.png").unwrap())
                );
                assert_eq!(
                    request.headers().get("Range"),
                    Some(&HeaderValue::from_str("bytes=1-").unwrap())
                );
                use http::response::Builder;
                let response = Builder::new()
                    .status(206)
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
            "foo4".try_into().unwrap(),
        )
        .middleware(Arc::new(MyMiddleware {}));

        let res = client.get_object("foo.png", 1..).await;

        //println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, String::from("content bar").into_bytes())
    }

    #[tokio::test]
    async fn test_end_range() {
        struct MyMiddleware {}

        #[async_trait]
        impl Middleware for MyMiddleware {
            async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png").unwrap()
                );
                assert_eq!(
                    request.headers().get("canonicalizedresource"),
                    Some(&HeaderValue::from_str("/foo4/foo.png").unwrap())
                );
                assert_eq!(
                    request.headers().get("Range"),
                    Some(&HeaderValue::from_str("bytes=0-10").unwrap())
                );
                use http::response::Builder;
                let response = Builder::new()
                    .status(206)
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
            "foo4".try_into().unwrap(),
        )
        .middleware(Arc::new(MyMiddleware {}));

        let res = client.get_object("foo.png", ..10).await;

        //println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, String::from("content bar").into_bytes())
    }

    #[tokio::test]
    async fn test_start_end_range() {
        struct MyMiddleware {}

        #[async_trait]
        impl Middleware for MyMiddleware {
            async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png").unwrap()
                );
                assert_eq!(
                    request.headers().get("canonicalizedresource"),
                    Some(&HeaderValue::from_str("/foo4/foo.png").unwrap())
                );
                assert_eq!(
                    request.headers().get("Range"),
                    Some(&HeaderValue::from_str("bytes=2-10").unwrap())
                );
                use http::response::Builder;
                let response = Builder::new()
                    .status(206)
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
            "foo4".try_into().unwrap(),
        )
        .middleware(Arc::new(MyMiddleware {}));

        let res = client.get_object("foo.png", 2..10).await;

        //println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, String::from("content bar").into_bytes())
    }
}

#[cfg(feature = "blocking")]
mod blocking_get_object {
    use std::rc::Rc;

    use http::HeaderValue;
    use reqwest::blocking::{Request, Response};
    use reqwest::Url;

    use crate::blocking::builder::ClientWithMiddleware;
    use crate::builder::BuilderError;
    use crate::file::blocking::File;
    use crate::{blocking::builder::Middleware, client::Client};

    #[test]
    fn test_all_range() {
        struct MyMiddleware {}

        impl Middleware for MyMiddleware {
            fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png").unwrap()
                );
                assert_eq!(
                    request.headers().get("canonicalizedresource"),
                    Some(&HeaderValue::from_str("/foo4/foo.png").unwrap())
                );
                assert_eq!(
                    request.headers().get("Range"),
                    Some(&HeaderValue::from_str("bytes=0-").unwrap())
                );
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
            "foo4".try_into().unwrap(),
        )
        .middleware(Rc::new(MyMiddleware {}));

        let res = client.get_object("foo.png", ..);

        //println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, String::from("content bar").into_bytes())
    }

    #[test]
    fn test_start_range() {
        struct MyMiddleware {}

        impl Middleware for MyMiddleware {
            fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png").unwrap()
                );
                assert_eq!(
                    request.headers().get("canonicalizedresource"),
                    Some(&HeaderValue::from_str("/foo4/foo.png").unwrap())
                );
                assert_eq!(
                    request.headers().get("Range"),
                    Some(&HeaderValue::from_str("bytes=1-").unwrap())
                );
                use http::response::Builder;
                let response = Builder::new()
                    .status(206)
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
            "foo4".try_into().unwrap(),
        )
        .middleware(Rc::new(MyMiddleware {}));

        let res = client.get_object("foo.png", 1..);

        //println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, String::from("content bar").into_bytes())
    }

    #[test]
    fn test_end_range() {
        struct MyMiddleware {}

        impl Middleware for MyMiddleware {
            fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png").unwrap()
                );
                assert_eq!(
                    request.headers().get("canonicalizedresource"),
                    Some(&HeaderValue::from_str("/foo4/foo.png").unwrap())
                );
                assert_eq!(
                    request.headers().get("Range"),
                    Some(&HeaderValue::from_str("bytes=0-10").unwrap())
                );
                use http::response::Builder;
                let response = Builder::new()
                    .status(206)
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
            "foo4".try_into().unwrap(),
        )
        .middleware(Rc::new(MyMiddleware {}));

        let res = client.get_object("foo.png", ..10);

        //println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, String::from("content bar").into_bytes())
    }

    #[test]
    fn test_start_end_range() {
        struct MyMiddleware {}

        impl Middleware for MyMiddleware {
            fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png").unwrap()
                );
                assert_eq!(
                    request.headers().get("canonicalizedresource"),
                    Some(&HeaderValue::from_str("/foo4/foo.png").unwrap())
                );
                assert_eq!(
                    request.headers().get("Range"),
                    Some(&HeaderValue::from_str("bytes=2-10").unwrap())
                );
                use http::response::Builder;
                let response = Builder::new()
                    .status(206)
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
            "foo4".try_into().unwrap(),
        )
        .middleware(Rc::new(MyMiddleware {}));

        let res = client.get_object("foo.png", 2..10);

        //println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, String::from("content bar").into_bytes())
    }
}

#[tokio::test]
async fn test_delete_object() {
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "DELETE");
            assert_eq!(
                *request.url(),
                Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/abc.png").unwrap()
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/foo4/abc.png").unwrap())
            );
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
                <ListBucketResult></ListBucketResult>"#,
                )
                .unwrap();
            let response = Response::from(response);
            Ok(response)
        }
    }

    let client = Client::<ClientWithMiddleware>::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap(),
    )
    .middleware(Arc::new(MyMiddleware {}));

    let res = client.delete_object("abc.png").await;
    //println!("{:?}", res);
    assert!(res.is_ok());
}

#[cfg(feature = "blocking")]
#[test]
fn test_blocking_delete_object() {
    use crate::client::ClientRc;
    use crate::{blocking::builder::Middleware, file::BlockingFile};
    use reqwest::blocking::{Request, Response};
    use std::rc::Rc;

    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "DELETE");
            assert_eq!(
                *request.url(),
                Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/abc.png").unwrap()
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/foo4/abc.png").unwrap())
            );
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
                <ListBucketResult></ListBucketResult>"#,
                )
                .unwrap();
            let response = Response::from(response);
            Ok(response)
        }
    }

    let client = ClientRc::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap(),
    )
    .middleware(Rc::new(MyMiddleware {}));

    let res = client.delete_object("abc.png");
    //println!("{:?}", res);
    assert!(res.is_ok());
}
