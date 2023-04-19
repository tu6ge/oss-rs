use std::error::Error;
use std::sync::Arc;

use crate::bucket::{Bucket, BucketError, BucketErrorKind};
use crate::builder::{ArcPointer, BuilderError, ClientWithMiddleware};
use crate::decode::{RefineBucket, RefineBucketList};
use crate::object::StorageClass;
use crate::tests::object::assert_object_list;
use crate::types::object::CommonPrefixes;
use crate::{EndPoint, Query, QueryKey};

use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use http::HeaderValue;
use reqwest::{Request, Response};

use crate::client::ClientArc;
use crate::{builder::Middleware, client::Client};

use super::ListBuckets;

#[tokio::test]
async fn test_get_bucket_list() {
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(
                *request.url(),
                "https://oss-cn-shanghai.aliyuncs.com/".parse().unwrap()
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/").unwrap())
            );
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
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Arc::new(MyMiddleware {}));

    let res = client.get_bucket_list().await;

    //println!("{:?}", res);
    assert_eq!(
        format!("{:?}", res),
        r#"Ok(ListBuckets { prefix: "", marker: "", max_keys: 0, is_truncated: false, next_marker: "", id: "", display_name: "", buckets: [] })"#
    );
}

#[cfg(feature = "blocking")]
#[test]
fn test_get_blocking_bucket_list() {
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
                "https://oss-cn-shanghai.aliyuncs.com/".parse().unwrap()
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/").unwrap())
            );
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
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Rc::new(MyMiddleware {}));

    let res = client.get_bucket_list();

    //println!("{:?}", res);
    assert_eq!(
        format!("{:?}", res),
        r#"Ok(ListBuckets { prefix: "", marker: "", max_keys: 0, is_truncated: false, next_marker: "", id: "", display_name: "", buckets: [] })"#
    );
}

#[tokio::test]
async fn test_get_bucket_info() {
    // use crate::bucket::Bucket;
    // use crate::types::{BucketName};
    // use crate::config::{BucketBase};
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        async fn handle(&self, request: Request) -> Result<Response, BuilderError> {
            //println!("request {:?}", request);
            assert_eq!(request.method(), "GET");
            assert_eq!(
                *request.url(),
                "https://foo4.oss-cn-shanghai.aliyuncs.com/?bucketInfo"
                    .parse()
                    .unwrap()
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/foo4/?bucketInfo").unwrap())
            );
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
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
                </BucketInfo>"#,
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

    let res = client.get_bucket_info().await;

    //println!("{:?}", res);
    assert_eq!(
        format!("{:?}", res),
        r#"Ok(Bucket { base: BucketBase { endpoint: CnShanghai, name: BucketName("barname") }, creation_date: 2016-11-05T13:10:10Z, storage_class: Standard })"#
    );
}

#[cfg(feature = "blocking")]
#[test]
fn test_get_blocking_bucket_info() {
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
                "https://foo4.oss-cn-shanghai.aliyuncs.com/?bucketInfo"
                    .parse()
                    .unwrap()
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/foo4/?bucketInfo").unwrap())
            );
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
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
                </BucketInfo>"#,
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

    let res = client.get_bucket_info();

    //println!("{:?}", res);
    assert_eq!(
        format!("{:?}", res),
        r#"Ok(Bucket { base: BucketBase { endpoint: CnShanghai, name: BucketName("barname") }, creation_date: 2016-11-05T13:10:10Z, storage_class: Standard })"#
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

    let client = ClientArc::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Arc::new(MyMiddleware {}));

    let bucket = bucket(client);

    let res = bucket
        .get_object_list(vec![("max-keys".into(), "5".into())])
        .await;

    assert!(res.is_ok());
    let list = res.unwrap();
    assert_object_list::<ArcPointer>(
        list,
        EndPoint::CnShanghai,
        "abc".parse().unwrap(),
        None,
        100,
        23,
        String::default(),
        CommonPrefixes::from_iter([]),
        Query::from_iter([(QueryKey::MaxKeys, 5u16)]),
    );
}

#[tokio::test]
async fn test_error_get_object_list() {
    struct MyMiddleware {}

    #[async_trait]
    impl Middleware for MyMiddleware {
        async fn handle(&self, _request: Request) -> Result<Response, BuilderError> {
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                //.url(url.clone())
                .body(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
                <ListBucketResult>
                  <Name>barname</Name>
                  <Prefix></Prefix>
                  <MaxKeys>aaa</MaxKeys>
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

    let client = ClientArc::new(
        "foo1".into(),
        "foo2".into(),
        "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        "foo4".parse().unwrap(),
    )
    .middleware(Arc::new(MyMiddleware {}));

    let bucket = bucket(client);

    let res = bucket
        .get_object_list(vec![("max-keys".into(), "5".into())])
        .await;

    let err = res.unwrap_err();
    assert_eq!(format!("{err}"), "decode xml failed");
    assert_eq!(
        format!("{}", err.source().unwrap()),
        "parse max-keys failed, gived str: aaa"
    );
}

fn bucket(client: Client) -> Bucket {
    let naive = NaiveDateTime::parse_from_str("2022/10/6 20:40:00", "%Y/%m/%d %H:%M:%S").unwrap();
    let creation_date = DateTime::from_utc(naive, Utc);

    Bucket::<ArcPointer>::new(
        "abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        creation_date,
        StorageClass::Archive,
        Arc::new(client),
    )
}

#[cfg(feature = "blocking")]
#[test]
fn test_get_blocking_object_list() {
    use crate::blocking::builder::Middleware;
    use crate::builder::RcPointer;
    use crate::client::ClientRc;
    use crate::tests::object::assert_object_list;
    use crate::types::object::CommonPrefixes;
    use crate::{EndPoint, Query, QueryKey};
    use reqwest::blocking::{Request, Response};
    use std::rc::Rc;

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

    let naive = NaiveDateTime::parse_from_str("2022/10/6 20:40:00", "%Y/%m/%d %H:%M:%S").unwrap();
    let creation_date = DateTime::from_utc(naive, Utc);

    let bucket = Bucket::<RcPointer>::new(
        "abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
        creation_date,
        StorageClass::Archive,
        Rc::new(client),
    );

    let res = bucket.get_object_list(vec![("max-keys".into(), "5".into())]);

    assert!(res.is_ok());
    let list = res.unwrap();
    assert_object_list::<RcPointer>(
        list,
        EndPoint::CnShanghai,
        "abc".parse().unwrap(),
        None,
        100,
        23,
        String::default(),
        CommonPrefixes::from_iter([]),
        Query::from_iter([(QueryKey::MaxKeys, 5u16)]),
    );
}

#[test]
fn test_set_storage_class() {
    let mut bucket = Bucket::<ArcPointer>::default();

    bucket.set_storage_class("archive").unwrap();
    assert_eq!(bucket.storage_class, StorageClass::Archive);
    bucket.set_storage_class("Archive").unwrap();
    assert_eq!(bucket.storage_class, StorageClass::Archive);

    bucket.set_storage_class("IA").unwrap();
    assert_eq!(bucket.storage_class, StorageClass::IA);
    bucket.set_storage_class("ia").unwrap();
    assert_eq!(bucket.storage_class, StorageClass::IA);

    bucket.set_storage_class("standard").unwrap();
    assert_eq!(bucket.storage_class, StorageClass::Standard);
    bucket.set_storage_class("Standard").unwrap();
    assert_eq!(bucket.storage_class, StorageClass::Standard);

    bucket.set_storage_class("cold_archive").unwrap();
    assert_eq!(bucket.storage_class, StorageClass::ColdArchive);
    bucket.set_storage_class("ColdArchive").unwrap();
    assert_eq!(bucket.storage_class, StorageClass::ColdArchive);

    let err = bucket.set_storage_class("eeeeee").unwrap_err();
    assert!(matches!(
        err,
        BucketError {
            kind: BucketErrorKind::InvalidStorageClass,
            ..
        }
    ));
}

#[test]
fn test_refine_bucket() {
    let mut list = ListBuckets::<ArcPointer>::default();
    list.set_prefix("foo1").unwrap();
    list.set_marker("foo2").unwrap();
    list.set_max_keys("10").unwrap();
    list.set_is_truncated(true).unwrap();
    list.set_next_marker("foo3").unwrap();
    list.set_id("foo4").unwrap();
    list.set_display_name("foo5").unwrap();

    assert_eq!(list.prefix, "foo1");
    assert_eq!(list.marker, "foo2");
    assert_eq!(list.max_keys, 10);
    assert_eq!(list.is_truncated, true);
    assert_eq!(list.next_marker, "foo3");
    assert_eq!(list.id, "foo4");
    assert_eq!(list.display_name, "foo5");
}

#[cfg(feature = "blocking")]
#[test]
fn test_default_list_bucket() {
    use crate::builder::RcPointer;

    use super::ListBuckets;

    let list = ListBuckets::<RcPointer>::default();

    assert!(list.buckets.len() == 0);
}

mod test_extract_item_error {
    use std::error::Error;

    use crate::{bucket::ExtractItemError, builder::BuilderError, decode::InnerItemError};

    #[test]
    fn from_builder() {
        let err = ExtractItemError::from(BuilderError::bar());

        assert_eq!(format!("{err}"), "builder error");
        assert_eq!(format!("{}", err.source().unwrap()), "bar");
        assert_eq!(
            format!("{:?}", err),
            "ExtractItemError { kind: Builder(BuilderError { kind: Bar }) }"
        );
    }

    #[test]
    fn from_decode() {
        let err = ExtractItemError::from(InnerItemError::new());

        assert_eq!(format!("{err}"), "decode xml failed");
        assert_eq!(format!("{}", err.source().unwrap()), "demo");
        assert_eq!(
            format!("{:?}", err),
            "ExtractItemError { kind: Decode(InnerItemError(MyError)) }"
        );
    }
}

mod test_bucket_error {

    use super::super::*;
    #[test]
    fn display() {
        let error = BucketError {
            kind: BucketErrorKind::BucketName(InvalidBucketName { _priv: () }),
            source: "abc".to_string(),
        };
        assert_eq!(error.to_string(), "decode bucket xml faild, gived str: abc");
        assert_eq!(
            format!("{}", error.source().unwrap()),
            "bucket name only allow `alphabet, digit, -`, and must not with `-` prefix or `-` suffix"
        );

        let error = BucketError {
            kind: BucketErrorKind::EndPoint(InvalidEndPoint { _priv: () }),
            source: "abc".to_string(),
        };
        assert_eq!(error.to_string(), "decode bucket xml faild, gived str: abc");
        assert_eq!(
            format!("{}", error.source().unwrap()),
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        );

        let error = "aaa".parse::<DateTime<Utc>>().unwrap_err();
        let error = BucketError {
            kind: BucketErrorKind::Chrono(error),
            source: "bar".to_string(),
        };
        assert_eq!(error.to_string(), "decode bucket xml faild, gived str: bar");
        assert_eq!(
            format!("{}", error.source().unwrap()),
            "input contains invalid characters"
        );

        let error = BucketError {
            kind: BucketErrorKind::InvalidStorageClass,
            source: "abc".to_string(),
        };
        assert_eq!(error.to_string(), "decode bucket xml faild, gived str: abc");
        assert!(error.source().is_none());
        assert_eq!(
            format!("{error:?}"),
            "BucketError { source: \"abc\", kind: InvalidStorageClass }"
        );
    }
}

mod test_bucket_list_error {
    use super::super::*;

    fn assert_impl<T: ListError>() {}

    #[test]
    fn test_bucket_list_error() {
        assert_impl::<BucketListError>();

        let err = i32::from_str_radix("a12", 10).unwrap_err();
        let err = BucketListError::ParseInt(err);
        assert_eq!(format!("{err}"), "covert max_keys failed");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "invalid digit found in string"
        );
        assert_eq!(
            format!("{err:?}"),
            "ParseInt(ParseIntError { kind: InvalidDigit })"
        );
    }
}
