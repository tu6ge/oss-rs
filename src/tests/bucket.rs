use std::sync::Arc;

use async_trait::async_trait;
use chrono::{NaiveDateTime, DateTime, Utc};
use http::HeaderValue;
use reqwest::{Request, Response, Url};
use crate::bucket::Bucket;
use crate::builder::{ClientWithMiddleware, ArcPointer};

use crate::client::ClientArc;
use crate::config::BucketBase;
use crate::types::Query;
use crate::{builder::Middleware, errors::{OssResult}, client::Client};


#[tokio::test]
async fn test_get_bucket_list(){
  
    struct MyMiddleware{}

    #[async_trait]
    impl Middleware for MyMiddleware{
        async fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(*request.url(), Url::parse("https://oss-cn-shanghai.aliyuncs.com/").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/").unwrap()));
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body("foo")
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
    
    let res = client.get_bucket_list().await;

    //println!("{:?}", res);
    assert_eq!(format!("{:?}", res), r#"Ok(ListBuckets { prefix: None, marker: None, max_keys: None, is_truncated: false, next_marker: None, id: None, display_name: None, buckets: [] })"#);
}

#[cfg(feature = "blocking")]
#[test]
fn test_get_blocking_bucket_list(){
    use crate::blocking::builder::Middleware;
    use reqwest::blocking::{Request, Response};
    use crate::client::ClientRc;
    use std::rc::Rc;
  
    struct MyMiddleware{}


    impl Middleware for MyMiddleware{
        fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(*request.url(), Url::parse("https://oss-cn-shanghai.aliyuncs.com/").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/").unwrap()));
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body("foo")
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
    
    let res = client.get_bucket_list();

    //println!("{:?}", res);
    assert_eq!(format!("{:?}", res), r#"Ok(ListBuckets { prefix: None, marker: None, max_keys: None, is_truncated: false, next_marker: None, id: None, display_name: None, buckets: [] })"#);
}

#[tokio::test]
async fn test_get_bucket_info(){
    // use crate::bucket::Bucket;
    // use crate::types::{BucketName};
    // use crate::config::{BucketBase};
    struct MyMiddleware{}

    #[async_trait]
    impl Middleware for MyMiddleware{
        async fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(*request.url(), Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/?bucketInfo").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/foo4/?bucketInfo").unwrap()));
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(r#"<?xml version="1.0" encoding="UTF-8"?>
                <BucketInfo>
                  <Bucket>
                    <AccessMonitor>Disabled</AccessMonitor>
                    <Comment></Comment>
                    <CreationDate>2016-11-05T13:10:10.000Z</CreationDate>
                    <CrossRegionReplication>Disabled</CrossRegionReplication>
                    <DataRedundancyType>LRS</DataRedundancyType>
                    <ExtranetEndpoint>oss-cn-shanghai.aliyuncs.com</ExtranetEndpoint>
                    <IntranetEndpoint>oss-cn-shanghai-internal.aliyuncs.com</IntranetEndpoint>
                    <Location>oss-cn-shanghai</Location>
                    <Name>barname</Name>
                    <ResourceGroupId>aaa</ResourceGroupId>
                    <StorageClass>Standard</StorageClass>
                    <TransferAcceleration>Disabled</TransferAcceleration>
                    <Owner>
                      <DisplayName>22222</DisplayName>
                      <ID>33333</ID>
                    </Owner>
                    <AccessControlList>
                      <Grant>public-read</Grant>
                    </AccessControlList>
                    <ServerSideEncryptionRule>
                      <SSEAlgorithm>None</SSEAlgorithm>
                    </ServerSideEncryptionRule>
                    <BucketPolicy>
                      <LogBucket></LogBucket>
                      <LogPrefix></LogPrefix>
                    </BucketPolicy>
                  </Bucket>
                </BucketInfo>"#)
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
    
    let res = client.get_bucket_info().await;

    //println!("{:?}", res);
    assert_eq!(format!("{:?}", res), r#"Ok(Bucket { base: BucketBase { endpoint: CnShanghai, name: BucketName("barname") }, creation_date: 2016-11-05T13:10:10Z, intranet_endpoint: "oss-cn-shanghai-internal.aliyuncs.com", location: "oss-cn-shanghai", storage_class: "Standard" })"#);
}

#[cfg(feature = "blocking")]
#[test]
fn test_get_blocking_bucket_info(){
    use crate::blocking::builder::Middleware;
    use reqwest::blocking::{Request, Response};
    use crate::client::ClientRc;
    use std::rc::Rc;

    struct MyMiddleware{}

    impl Middleware for MyMiddleware{
        fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(*request.url(), Url::parse("https://foo4.oss-cn-shanghai.aliyuncs.com/?bucketInfo").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/foo4/?bucketInfo").unwrap()));
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(r#"<?xml version="1.0" encoding="UTF-8"?>
                <BucketInfo>
                  <Bucket>
                    <AccessMonitor>Disabled</AccessMonitor>
                    <Comment></Comment>
                    <CreationDate>2016-11-05T13:10:10.000Z</CreationDate>
                    <CrossRegionReplication>Disabled</CrossRegionReplication>
                    <DataRedundancyType>LRS</DataRedundancyType>
                    <ExtranetEndpoint>oss-cn-shanghai.aliyuncs.com</ExtranetEndpoint>
                    <IntranetEndpoint>oss-cn-shanghai-internal.aliyuncs.com</IntranetEndpoint>
                    <Location>oss-cn-shanghai</Location>
                    <Name>barname</Name>
                    <ResourceGroupId>aaa</ResourceGroupId>
                    <StorageClass>Standard</StorageClass>
                    <TransferAcceleration>Disabled</TransferAcceleration>
                    <Owner>
                      <DisplayName>22222</DisplayName>
                      <ID>33333</ID>
                    </Owner>
                    <AccessControlList>
                      <Grant>public-read</Grant>
                    </AccessControlList>
                    <ServerSideEncryptionRule>
                      <SSEAlgorithm>None</SSEAlgorithm>
                    </ServerSideEncryptionRule>
                    <BucketPolicy>
                      <LogBucket></LogBucket>
                      <LogPrefix></LogPrefix>
                    </BucketPolicy>
                  </Bucket>
                </BucketInfo>"#)
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
    
    let res = client.get_bucket_info();

    //println!("{:?}", res);
    assert_eq!(format!("{:?}", res), r#"Ok(Bucket { base: BucketBase { endpoint: CnShanghai, name: BucketName("barname") }, creation_date: 2016-11-05T13:10:10Z, intranet_endpoint: "oss-cn-shanghai-internal.aliyuncs.com", location: "oss-cn-shanghai", storage_class: "Standard" })"#);
}

#[tokio::test]
async fn test_get_object_list(){
    struct MyMiddleware{}

    #[async_trait]
    impl Middleware for MyMiddleware{
        async fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(*request.url(), Url::parse("https://abc.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/abc/").unwrap()));
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
    
    let client = ClientArc::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".try_into().unwrap(),
        "foo4".try_into().unwrap()
    ).middleware(Arc::new(MyMiddleware{}))
    ;

    let naive = NaiveDateTime::parse_from_str("2022/10/6 20:40:00", "%Y/%m/%d %H:%M:%S").unwrap();
    let creation_date = DateTime::from_utc(naive, Utc);

    let bucket = Bucket::<ArcPointer>::new(
        BucketBase::from_str("abc.oss-cn-shanghai.aliyuncs.com").unwrap(),
        creation_date,
        String::from("foo1"),
        String::from("foo2"),
        String::from("foo3"),
        Arc::new(client)
    );

    let mut query = Query::new();
    query.insert("max-keys", "5");
    let res = bucket.get_object_list(query).await;

    assert_eq!(format!("{:?}", res), r##"Ok(ObjectList { name: "barname", bucket: BucketBase { endpoint: CnShanghai, name: BucketName("abc") }, prefix: "", max_keys: 100, key_count: 23, next_continuation_token: None, search_query: Query { inner: {QueryKey("max-keys"): QueryValue("5")} } })"##);
}

#[cfg(feature = "blocking")]
#[test]
fn test_get_blocking_object_list(){
    use crate::blocking::builder::Middleware;
    use reqwest::blocking::{Request, Response};
    use crate::client::ClientRc;
    use crate::builder::RcPointer;
    use std::rc::Rc;

    struct MyMiddleware{}

    impl Middleware for MyMiddleware{
        fn handle(&self, request: Request) -> OssResult<Response>{
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(*request.url(), Url::parse("https://abc.oss-cn-shanghai.aliyuncs.com/?list-type=2&max-keys=5").unwrap());
            assert_eq!(request.headers().get("canonicalizedresource"), Some(&HeaderValue::from_str("/abc/").unwrap()));
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
    ).middleware(Rc::new(MyMiddleware{}))
    ;

    let naive = NaiveDateTime::parse_from_str("2022/10/6 20:40:00", "%Y/%m/%d %H:%M:%S").unwrap();
    let creation_date = DateTime::from_utc(naive, Utc);

    let bucket = Bucket::<RcPointer>::new(
        BucketBase::from_str("abc.oss-cn-shanghai.aliyuncs.com").unwrap(),
        creation_date,
        String::from("foo1"),
        String::from("foo2"),
        String::from("foo3"),
        Rc::new(client)
    );

    let mut query = Query::new();
    query.insert("max-keys", "5");
    let res = bucket.get_object_list(query);

    assert_eq!(format!("{:?}", res), r##"Ok(ObjectList { name: "barname", bucket: BucketBase { endpoint: CnShanghai, name: BucketName("abc") }, prefix: "", max_keys: 100, key_count: 23, next_continuation_token: None, search_query: Query { inner: {QueryKey("max-keys"): QueryValue("5")} } })"##);
}