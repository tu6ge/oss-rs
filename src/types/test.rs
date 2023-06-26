use std::env::set_var;

use chrono::{TimeZone, Utc};
use http::HeaderValue;
use reqwest::Url;

use crate::{
    types::{CanonicalizedResource, EndPointKind, InvalidBucketName},
    BucketName, EndPoint, KeyId, KeySecret,
};

use super::{ContentMd5, ContentType, Date, InvalidEndPoint};

#[test]
fn key_id() {
    let key = KeyId::new("aaa");
    assert_eq!(format!("{key}"), "aaa");

    assert!(TryInto::<HeaderValue>::try_into(key).is_ok());

    let key2 = KeyId::from_static("aaa");
    assert_eq!(format!("{key2}"), "aaa");
}

#[test]
fn secret() {
    let secret = KeySecret::new("aaa");
    assert_eq!(format!("{secret}"), "******secret******");
    assert_eq!(secret.as_str(), "aaa");

    let key2 = KeySecret::from_static("bbb");
    assert_eq!(format!("{key2}"), "******secret******");
    assert_eq!(key2.as_str(), "bbb");

    assert_eq!(format!("{key2:?}"), "KeySecret");
}

#[test]
fn encryption() {
    let secret = KeySecret::new("aaa");
    let result = secret.encryption(b"bbb").unwrap();
    assert_eq!(result, "nGG1u/pDDcCCgw9xP87bPPJxWTY=");

    let secret = KeySecret::new("aaa");
    let result = secret.encryption_string("bbb".into()).unwrap();
    assert_eq!(result, "nGG1u/pDDcCCgw9xP87bPPJxWTY=");
}

#[test]
fn endpoint() {
    let end = unsafe { EndPoint::from_static2("aaa") };

    assert_eq!(end.as_ref(), "aaa");

    let end1 = EndPoint {
        kind: EndPointKind::Other("aaa".into()),
        is_internal: false,
    };
    assert_eq!(end1.as_ref(), "aaa");

    assert!(EndPoint::new("").is_err());

    assert!(end == end1);
    assert!(end == "aaa");
    assert!("aaa" == end);

    let endpoint = EndPoint::new("shanghai").unwrap();
    assert!(endpoint == Url::parse("https://oss-cn-shanghai.aliyuncs.com").unwrap());
}

mod test_endpoint {
    use std::{
        borrow::Cow,
        env::{remove_var, set_var},
    };

    use super::*;

    #[test]
    #[should_panic]
    fn test_endpoint_painc() {
        EndPoint::from_static("-weifang");
    }

    #[test]
    fn test_new() {
        assert!(matches!(
            EndPoint::new("hangzhou"),
            Ok(EndPoint::CN_HANGZHOU)
        ));

        assert!(matches!(EndPoint::new("qingdao"), Ok(EndPoint::CN_QINGDAO)));

        assert!(matches!(EndPoint::new("beijing"), Ok(EndPoint::CN_BEIJING)));

        assert!(matches!(
            EndPoint::new("zhangjiakou"),
            Ok(EndPoint::CN_ZHANGJIAKOU)
        ));

        assert!(matches!(
            EndPoint::new("hongkong"),
            Ok(EndPoint::CN_HONGKONG)
        ));

        assert!(matches!(
            EndPoint::new("shenzhen"),
            Ok(EndPoint::CN_SHENZHEN)
        ));

        assert!(matches!(
            EndPoint::new("us-west-1"),
            Ok(EndPoint::US_WEST_1)
        ));

        assert!(matches!(
            EndPoint::new("us-east-1"),
            Ok(EndPoint::US_EAST_1)
        ));

        assert!(matches!(
            EndPoint::new("ap-southeast-1"),
            Ok(EndPoint::AP_SOUTH_EAST_1)
        ));

        assert!(matches!(
            EndPoint::new("weifang"),
            Ok(EndPoint {
                kind: EndPointKind::Other(Cow::Owned(_)),
                ..
            })
        ));

        assert!(matches!(
            EndPoint::new("https://oss-cn-qingdao-internal.aliyuncs.com"),
            Ok(EndPoint {
                kind: EndPointKind::CnQingdao,
                is_internal: false,
            })
        ));
        assert!(matches!(
            EndPoint::new("https://oss-cn-qingdao.aliyuncs.com"),
            Ok(EndPoint {
                kind: EndPointKind::CnQingdao,
                is_internal: false,
            })
        ));

        let res = EndPoint::new("abc-internal").unwrap();
        assert_eq!(res.is_internal, true);
        assert_eq!(res.as_ref(), "abc");
    }

    #[test]
    fn test_from_env() {
        let has_err = EndPoint::from_env();
        assert!(has_err.is_err());

        set_var("ALIYUN_ENDPOINT", "ossaa");
        let has_err = EndPoint::from_env();
        assert!(has_err.is_err());

        set_var("ALIYUN_ENDPOINT", "qingdao");
        remove_var("ALIYUN_OSS_INTERNAL");
        let endpoint = EndPoint::from_env().unwrap();
        assert_eq!(endpoint.kind, EndPointKind::CnQingdao);
        assert!(!endpoint.is_internal);

        set_var("ALIYUN_OSS_INTERNAL", "true");
        let endpoint = EndPoint::from_env().unwrap();
        assert_eq!(endpoint.kind, EndPointKind::CnQingdao);
        assert!(endpoint.is_internal);

        set_var("ALIYUN_OSS_INTERNAL", "0");
        let endpoint = EndPoint::from_env().unwrap();
        assert!(!endpoint.is_internal);

        set_var("ALIYUN_OSS_INTERNAL", "1");
        let endpoint = EndPoint::from_env().unwrap();
        assert!(endpoint.is_internal);

        set_var("ALIYUN_OSS_INTERNAL", "yes");
        let endpoint = EndPoint::from_env().unwrap();
        assert!(endpoint.is_internal);

        set_var("ALIYUN_OSS_INTERNAL", "Y");
        let endpoint = EndPoint::from_env().unwrap();
        assert!(endpoint.is_internal);
    }

    #[test]
    fn test_from_host_piece() {
        assert!(EndPoint::from_host_piece("qingdao").is_err());

        assert_eq!(
            EndPoint::from_host_piece("oss-cn-qingdao"),
            Ok(EndPoint::CN_QINGDAO)
        );
        assert_eq!(
            EndPoint::from_host_piece("oss-qingdao"),
            Ok(EndPoint {
                kind: EndPointKind::CnQingdao,
                is_internal: false,
            })
        );
        assert_eq!(
            EndPoint::from_host_piece("oss-qingdao-internal"),
            Ok(EndPoint {
                kind: EndPointKind::CnQingdao,
                is_internal: true,
            })
        );
    }

    #[test]
    fn test_from_url() {
        let url = Url::parse("https://oss-cn-qingdao.aliyuncs.com/").unwrap();
        let endpoint = EndPoint::try_from(url).unwrap();

        assert!(matches!(endpoint.kind, EndPointKind::CnQingdao));
        assert_eq!(endpoint.is_internal, false);

        let url = Url::parse("https://oss-cn-qingdao-internal.aliyuncs.com/").unwrap();
        let endpoint = EndPoint::try_from(url).unwrap();

        assert!(matches!(endpoint.kind, EndPointKind::CnQingdao));
        assert_eq!(endpoint.is_internal, true);

        let url = Url::parse("https://192.168.3.1/").unwrap();
        assert!(EndPoint::try_from(url).is_err());

        let url = Url::parse("https://oss-cn-qingdao-internal.aliyuncs.cn/").unwrap();
        assert!(EndPoint::try_from(url).is_err());

        let url = Url::parse("https://oss-cn-qingdao-internal.aliyun.com/").unwrap();
        assert!(EndPoint::try_from(url).is_err());

        let url = Url::parse("https://aliyuncs.com/").unwrap();
        assert!(EndPoint::try_from(url).is_err());

        let url = Url::parse("https://-cn-qingdao.aliyuncs.com/").unwrap();
        assert!(EndPoint::try_from(url).is_err());
    }
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
fn bucket_name_from_env() {
    let bucket = BucketName::from_env();
    assert!(bucket.is_err());

    set_var("ALIYUN_BUCKET", "abc");
    let name = BucketName::from_env().unwrap();
    assert_eq!(name, "abc");
}

#[test]
fn content_md5() {
    let value: crate::types::InnerContentMd5 = ContentMd5::from_static("aaa");
    assert!(TryInto::<HeaderValue>::try_into(value).is_ok());
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
    let date = unsafe { Date::from_static("Sat, 01 Jan 2022 18:01:01 GMT") };

    assert_eq!(format!("{date}"), "Sat, 01 Jan 2022 18:01:01 GMT");

    let utc = Utc.with_ymd_and_hms(2022, 1, 1, 18, 1, 1).unwrap();

    let date = Date::from(utc);

    assert_eq!(date.as_ref(), "Sat, 01 Jan 2022 18:01:01 GMT");

    assert!(TryInto::<HeaderValue>::try_into(date).is_ok());
}

#[test]
fn canonicalized_resource() {
    let value = CanonicalizedResource::from_static("aaa");
    assert_eq!(format!("{value}"), "aaa");

    assert!(TryInto::<HeaderValue>::try_into(value).is_ok());

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
        assert!("/" == value);
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
        query.insert(QueryKey::CONTINUATION_TOKEN, "foo");
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

    #[test]
    fn test_from_object_str() {
        use super::CanonicalizedResource;
        let resource = CanonicalizedResource::from_object_str("foo", "bar");
        assert!(resource.as_ref() == "/foo/bar");
    }
}
