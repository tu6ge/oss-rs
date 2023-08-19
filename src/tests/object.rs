use crate::builder::ArcPointer;
#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
use crate::builder::{BuilderError, ClientWithMiddleware, PointerFamily};
use crate::file::Files;
use crate::object::ObjectList;
use crate::types::core::IntoQuery;
use crate::types::object::{CommonPrefixes, ObjectPath};
use crate::{builder::Middleware, client::Client};
use crate::{BucketName, EndPoint, Query, QueryKey};
use async_trait::async_trait;
use http::HeaderValue;
use reqwest::{Request, Response};
use std::error::Error;
use std::sync::Arc;

pub(crate) fn assert_object_list<T: PointerFamily>(
    list: ObjectList<T>,
    endpoint: EndPoint,
    name: BucketName,
    prefix: Option<crate::ObjectDir>,
    max_keys: u32,
    key_count: u64,
    next_continuation_token: String,
    common_prefixes: CommonPrefixes,
    search_query: Query,
) {
    assert!(list.bucket().clone().endpoint() == endpoint);
    assert!(list.bucket().clone().name() == name);
    assert!(*list.prefix() == prefix);
    assert!(*list.max_keys() == max_keys);
    assert!(*list.key_count() == key_count);
    assert!(*list.next_continuation_token_str() == next_continuation_token);
    assert!(*list.common_prefixes() == common_prefixes);
    assert!(*list.search_query() == search_query);
}

#[cfg(feature = "blocking")]
#[test]
fn object_list_get_object_list() {
    use crate::client::ClientRc;
    use crate::{blocking::builder::Middleware, builder::RcPointer, object::ObjectList};
    use reqwest::blocking::{Request, Response};
    use std::rc::Rc;

    #[derive(Debug)]
    struct MyMiddleware {}

    impl Middleware for MyMiddleware {
        fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(
                *request.url(),
                "https://abc.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5"
                    .parse()
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
                  <Prefix>foo2</Prefix>
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
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Rc::new(MyMiddleware {}));

    let object_list = ObjectList::<RcPointer>::new(
        "abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        Some("foo2/".parse().unwrap()),
        100,
        200,
        Vec::new(),
        None,
        Rc::new(client),
        vec![("max-keys".into(), 5u8.into())],
    );

    let res = object_list.get_object_list();

    assert!(res.is_ok());
    let list = res.unwrap();
    assert_object_list::<RcPointer>(
        list,
        EndPoint::CN_SHANGHAI,
        "abc".parse().unwrap(),
        Some("foo2/".parse().unwrap()),
        100,
        23,
        String::default(),
        CommonPrefixes::from_iter([]),
        [(QueryKey::MAX_KEYS, 5u16)].into_query(),
    );
}

#[tokio::test]
async fn test_get_object_list() {
    #[derive(Debug)]
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(
                *request.url(),
                "https://foo4.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5"
                    .parse()
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
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Arc::new(MyMiddleware {}));

    let res = client.get_object_list([("max-keys", "5")]).await;

    assert!(res.is_ok());
    let list = res.unwrap();
    assert_object_list::<ArcPointer>(
        list,
        EndPoint::CN_SHANGHAI,
        "foo4".parse().unwrap(),
        None,
        100,
        23,
        String::default(),
        CommonPrefixes::from_iter([]),
        [(QueryKey::MAX_KEYS, 5u16)].into_query(),
    );
}

#[tokio::test]
async fn test_error_object_list() {
    #[derive(Debug)]
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(
                *request.url(),
                "https://foo4.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5"
                    .parse()
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
                  <KeyCount>foo</KeyCount>
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
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Arc::new(MyMiddleware {}));

    let res = client.get_object_list([("max-keys", "5")]).await;
    let err = res.unwrap_err();

    assert_eq!(format!("{err}"), "decode xml failed");
    assert_eq!(
        format!("{}", err.source().unwrap()),
        "parse key-count failed, gived str: foo"
    );
}

#[tokio::test]
async fn test_item_error_object_list() {
    #[derive(Debug)]
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(
                *request.url(),
                "https://foo4.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5"
                    .parse()
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
                    <Size>aaa</Size>
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
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Arc::new(MyMiddleware {}));

    let res = client.get_object_list([("max-keys", "5")]).await;
    let err = res.unwrap_err();

    assert_eq!(format!("{err}"), "decode xml failed");
    assert_eq!(
        format!("{}", err.source().unwrap()),
        "parse size failed, gived str: aaa"
    );
}

#[cfg(feature = "blocking")]
#[test]
fn test_get_blocking_object_list() {
    use crate::blocking::builder::Middleware;
    use crate::client::ClientRc;
    use reqwest::blocking::{Request, Response};
    use std::rc::Rc;

    #[derive(Debug)]
    struct MyMiddleware {}

    impl Middleware for MyMiddleware {
        fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(
                *request.url(),
                "https://foo4.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5"
                    .parse()
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
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Rc::new(MyMiddleware {}));

    let res = client.get_object_list([("max-keys", "5")]);

    assert!(res.is_ok());
    let list = res.unwrap();
    assert_object_list::<RcPointer>(
        list,
        EndPoint::CN_SHANGHAI,
        "foo4".parse().unwrap(),
        None,
        100,
        23,
        String::default(),
        CommonPrefixes::from_iter([]),
        [(QueryKey::MAX_KEYS, 5u16)].into_query(),
    );
}

#[tokio::test]
async fn test_put_content_base() {
    #[derive(Debug)]
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "PUT");
            assert_eq!(
                *request.url(),
                "https://foo4.oss-cn-shanghai.aliyuncs.com/abc.text"
                    .parse()
                    .unwrap()
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
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Arc::new(MyMiddleware {}));

    let content = String::from("Hello world");
    let content: Vec<u8> = content.into();

    let res = client
        .put_content_base(
            content,
            "application/text",
            "abc.text".parse::<ObjectPath>().unwrap(),
        )
        .await;

    //println!("{:?}", res);
    assert!(res.is_ok());
}

#[cfg(feature = "blocking")]
#[test]
fn test_blocking_put_content_base() {
    use crate::client::ClientRc;
    use crate::{blocking::builder::Middleware, file::blocking::Files};
    use reqwest::blocking::{Request, Response};
    use std::rc::Rc;

    #[derive(Debug)]
    struct MyMiddleware {}

    impl Middleware for MyMiddleware {
        fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "PUT");
            assert_eq!(
                *request.url(),
                "https://foo4.oss-cn-shanghai.aliyuncs.com/abc.text"
                    .parse()
                    .unwrap()
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
    use reqwest::{Request, Response};

    use crate::builder::{BuilderError, ClientWithMiddleware};
    use crate::file::Files;
    use crate::types::object::ObjectPath;
    use crate::{builder::Middleware, client::Client};
    use async_trait::async_trait;

    #[tokio::test]
    async fn test_all_range() {
        #[derive(Debug)]
        struct MyMiddleware {}

        #[async_trait]
        impl Middleware for MyMiddleware {
            async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    "https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png"
                        .parse()
                        .unwrap()
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
            "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            "foo4".parse().unwrap(),
        )
        .middleware(Arc::new(MyMiddleware {}));

        let res = client
            .get_object("foo.png".parse::<ObjectPath>().unwrap(), ..)
            .await;

        //println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, String::from("content bar").into_bytes())
    }

    #[tokio::test]
    async fn test_start_range() {
        #[derive(Debug)]
        struct MyMiddleware {}

        #[async_trait]
        impl Middleware for MyMiddleware {
            async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    "https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png"
                        .parse()
                        .unwrap()
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
            "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            "foo4".parse().unwrap(),
        )
        .middleware(Arc::new(MyMiddleware {}));

        let res = client
            .get_object("foo.png".parse::<ObjectPath>().unwrap(), 1..)
            .await;

        //println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, String::from("content bar").into_bytes())
    }

    #[tokio::test]
    async fn test_end_range() {
        #[derive(Debug)]
        struct MyMiddleware {}

        #[async_trait]
        impl Middleware for MyMiddleware {
            async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    "https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png"
                        .parse()
                        .unwrap()
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
            "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            "foo4".parse().unwrap(),
        )
        .middleware(Arc::new(MyMiddleware {}));

        let res = client
            .get_object("foo.png".parse::<ObjectPath>().unwrap(), ..10)
            .await;

        //println!("{:?}", res);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res, String::from("content bar").into_bytes())
    }

    #[tokio::test]
    async fn test_start_end_range() {
        #[derive(Debug)]
        struct MyMiddleware {}

        #[async_trait]
        impl Middleware for MyMiddleware {
            async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    "https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png"
                        .parse()
                        .unwrap()
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
            "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            "foo4".parse().unwrap(),
        )
        .middleware(Arc::new(MyMiddleware {}));

        let res = client
            .get_object("foo.png".parse::<ObjectPath>().unwrap(), 2..10)
            .await;

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

    use crate::blocking::builder::ClientWithMiddleware;
    use crate::builder::BuilderError;
    use crate::file::blocking::Files;
    use crate::{blocking::builder::Middleware, client::Client};

    #[test]
    fn test_all_range() {
        #[derive(Debug)]
        struct MyMiddleware {}

        impl Middleware for MyMiddleware {
            fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    "https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png"
                        .parse()
                        .unwrap()
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
            "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            "foo4".parse().unwrap(),
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
        #[derive(Debug)]
        struct MyMiddleware {}

        impl Middleware for MyMiddleware {
            fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    "https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png"
                        .parse()
                        .unwrap()
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
            "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            "foo4".parse().unwrap(),
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
        #[derive(Debug)]
        struct MyMiddleware {}

        impl Middleware for MyMiddleware {
            fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    "https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png"
                        .parse()
                        .unwrap()
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
            "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            "foo4".parse().unwrap(),
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
        #[derive(Debug)]
        struct MyMiddleware {}

        impl Middleware for MyMiddleware {
            fn handle(&self, request: Request) -> Result<Response, BuilderError> {
                //println!("request {:?}", request);
                assert_eq!(request.method(), "GET");
                assert_eq!(
                    *request.url(),
                    "https://foo4.oss-cn-shanghai.aliyuncs.com/foo.png"
                        .parse()
                        .unwrap()
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
            "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            "foo4".parse().unwrap(),
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
    #[derive(Debug)]
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "DELETE");
            assert_eq!(
                *request.url(),
                "https://foo4.oss-cn-shanghai.aliyuncs.com/abc.png"
                    .parse()
                    .unwrap()
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
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Arc::new(MyMiddleware {}));

    let res = client
        .delete_object("abc.png".parse::<ObjectPath>().unwrap())
        .await;
    //println!("{:?}", res);
    assert!(res.is_ok());
}

#[cfg(feature = "blocking")]
#[test]
fn test_blocking_delete_object() {
    use crate::client::ClientRc;
    use crate::{blocking::builder::Middleware, file::BlockingFiles};
    use reqwest::blocking::{Request, Response};
    use std::rc::Rc;

    #[derive(Debug)]
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "DELETE");
            assert_eq!(
                *request.url(),
                "https://foo4.oss-cn-shanghai.aliyuncs.com/abc.png"
                    .parse()
                    .unwrap()
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
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Rc::new(MyMiddleware {}));

    let res = client.delete_object("abc.png");
    //println!("{:?}", res);
    assert!(res.is_ok());
}
