use std::sync::Arc;

use async_trait::async_trait;
use http::HeaderValue;
use reqwest::{Request, Response, Url};

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

    let client = Client::new(
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

    let client = Client::new(
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