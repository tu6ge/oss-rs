use std::borrow::Cow;

use chrono::{TimeZone, Utc};
use http::HeaderValue;
use reqwest::Url;

use crate::{
    types::{CanonicalizedResource, InvalidBucketName},
    BucketName, EndPoint, KeyId, KeySecret,
};

use super::{ContentMd5, ContentType, Date, InvalidEndPoint};

#[test]
fn key_id() {
    let key = KeyId::new("aaa");
    assert_eq!(format!("{key}"), "aaa");

    assert!(TryInto::<HeaderValue>::try_into(key).is_ok());

    const KEY2: KeyId = KeyId::from_static("aaa");
    assert_eq!(format!("{KEY2}"), "aaa");
}

#[test]
fn secret() {
    let secret = KeySecret::new("aaa");
    assert_eq!(format!("{secret}"), "aaa");

    const KEY2: KeySecret = KeySecret::from_static("aaa");
    assert_eq!(format!("{KEY2}"), "aaa");
    assert_eq!(KEY2.as_bytes(), b"aaa");
}

#[test]
fn endpoint() {
    const END: EndPoint = unsafe { EndPoint::from_static2("aaa") };

    assert_eq!(END.as_ref(), "aaa");

    let end1 = EndPoint::Other(Cow::Borrowed("aaa"));
    assert_eq!(end1.as_ref(), "aaa");

    assert!(EndPoint::new("").is_err());

    assert!(END == end1);
    assert!(END == "aaa");
    assert!("aaa" == END);

    let endpoint = EndPoint::new("shanghai").unwrap();
    assert!(endpoint == Url::parse("https://oss-cn-shanghai.aliyuncs.com").unwrap());
}

#[test]
fn invalid_endpoint() {
    let err1 = InvalidEndPoint { _priv: () };
    let err2 = InvalidEndPoint { _priv: () };

    assert!(err1 == err2);
}

#[test]
fn bucket_name() {
    let name = unsafe { BucketName::from_static2("aaa") };
    assert_eq!(format!("{name}"), "aaa");

    let invalid = InvalidBucketName { _priv: () };
    let invalid2 = InvalidBucketName { _priv: () };
    assert!(invalid == invalid2);
}

#[test]
fn content_md5() {
    const VALUE: ContentMd5 = ContentMd5::from_static("aaa");
    assert!(TryInto::<HeaderValue>::try_into(VALUE).is_ok());
}

#[test]
fn content_type() {
    let content = ContentType::from("aaa".to_string());
    assert_eq!(format!("{content}"), "aaa");
    assert!(TryInto::<HeaderValue>::try_into(content).is_ok());

    let content = ContentType::from_static("aaa");
    assert_eq!(format!("{content}"), "aaa");
}

#[test]
fn date() {
    const DATE: Date = unsafe { Date::from_static("Sat, 01 Jan 2022 18:01:01 GMT") };

    assert_eq!(format!("{DATE}"), "Sat, 01 Jan 2022 18:01:01 GMT");

    let utc = Utc.with_ymd_and_hms(2022, 1, 1, 18, 1, 1).unwrap();

    let date = Date::from(utc);

    assert_eq!(date.as_ref(), "Sat, 01 Jan 2022 18:01:01 GMT");

    assert!(TryInto::<HeaderValue>::try_into(date).is_ok());
}

#[test]
fn canonicalized_resource() {
    const VALUE: CanonicalizedResource = CanonicalizedResource::from_static("aaa");
    assert_eq!(format!("{VALUE}"), "aaa");

    assert!(TryInto::<HeaderValue>::try_into(VALUE).is_ok());

    let value = CanonicalizedResource::from("aaa".to_string());
    assert_eq!(format!("{value}"), "aaa");
}

mod tests_canonicalized_resource {

    #[cfg(feature = "auth")]
    #[test]
    fn canonicalized_from_bucket_name() {
        use crate::{types::CanonicalizedResource, BucketName};

        let name = BucketName::new("aaa").unwrap();
        let value = CanonicalizedResource::from_bucket_name(&name, Some("bucketInfo"));
        assert!(value == "/aaa/?bucketInfo");

        let value = CanonicalizedResource::from_bucket_name(&name, Some("bbb"));
        assert!(value == "/aaa/");

        let value = CanonicalizedResource::from_bucket_name(&name, None);
        assert!(value == "/");
    }

    #[cfg(feature = "core")]
    #[test]
    fn test_from_bucket() {
        use crate::{config::BucketBase, types::CanonicalizedResource};

        let base: BucketBase = "abc.jinan".parse().unwrap();
        let resource = CanonicalizedResource::from_bucket(&base, Some("bucketInfo"));
        assert_eq!(resource, "/abc/?bucketInfo");

        let base: BucketBase = "abc.jinan".parse().unwrap();
        let resource = CanonicalizedResource::from_bucket(&base, Some("bar"));
        assert_eq!(resource, "/abc/");

        let base: BucketBase = "abc.jinan".parse().unwrap();
        let resource = CanonicalizedResource::from_bucket(&base, None);
        assert_eq!(resource, "/");
    }

    #[cfg(feature = "core")]
    #[test]
    fn test_from_bucket_query2() {
        use crate::{types::CanonicalizedResource, BucketName, Query, QueryKey};

        let bucket = BucketName::new("abc").unwrap();
        let query = Query::new();
        let resource = CanonicalizedResource::from_bucket_query2(&bucket, &query);
        assert_eq!(resource, CanonicalizedResource::new("/abc/"));

        let mut query = Query::new();
        query.insert("list-type", "2");
        query.insert(QueryKey::ContinuationToken, "foo");
        let resource = CanonicalizedResource::from_bucket_query2(&bucket, &query);
        assert_eq!(
            resource,
            CanonicalizedResource::new("/abc/?continuation-token=foo")
        );
    }

    #[cfg(feature = "core")]
    #[test]
    fn test_from_object() {
        use super::CanonicalizedResource;

        let resource = CanonicalizedResource::from_object(("foo", "bar"), []);
        assert!(resource == "/foo/bar");

        let resource =
            CanonicalizedResource::from_object(("foo", "bar"), [("foo2".into(), "bar2".into())]);
        assert!(resource == "/foo/bar?foo2=bar2");
    }
}
