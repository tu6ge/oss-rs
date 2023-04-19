#[cfg(test)]
mod tests {
    use super::super::ObjectList;
    use crate::{
        builder::ArcPointer,
        config::BucketBase,
        object::{Object, ObjectBuilder, StorageClass},
        types::{object::ObjectPath, QueryValue},
        Client,
    };
    use chrono::{DateTime, NaiveDateTime, Utc};
    use std::sync::Arc;

    fn init_object_list(token: Option<String>, list: Vec<Object>) -> ObjectList {
        let client = Client::new(
            "foo1".into(),
            "foo2".into(),
            "https://oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            "foo4".parse().unwrap(),
        );

        let object_list = ObjectList::<ArcPointer>::new(
            "abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap(),
            Some("foo2/".parse().unwrap()),
            100,
            200,
            list,
            token,
            Arc::new(client),
            vec![("key1".into(), "value1".into())],
        );

        object_list
    }

    #[test]
    fn test_object_list_fmt() {
        let object_list = init_object_list(Some(String::from("foo3")), vec![]);
        assert_eq!(
            format!("{object_list:?}"),
            "ObjectList { bucket: BucketBase { endpoint: CnShanghai, name: BucketName(\"abc\") }, prefix: Some(ObjectDir(\"foo2/\")), max_keys: 100, key_count: 200, next_continuation_token: \"foo3\", common_prefixes: [], search_query: Query { inner: {Custom(\"key1\"): InnerQueryValue(\"value1\")} } }"
        );
    }

    #[test]
    fn test_get_bucket() {
        let object_list = init_object_list(Some(String::from("foo3")), vec![]);

        let bucket = object_list.bucket();

        assert_eq!(bucket.name(), "abc");

        assert!(object_list.prefix() == &Some("foo2/".parse().unwrap()));

        assert!(object_list.max_keys() == &100u32);
        assert_eq!(object_list.max_keys().to_owned(), 100u32);

        assert_eq!(object_list.next_continuation_token_str(), "foo3");
    }

    #[test]
    fn test_bucket_name() {
        let object_list = init_object_list(Some(String::from("foo3")), vec![]);
        let bucket_name = object_list.bucket_name();

        assert!("abc" == bucket_name);
    }

    #[test]
    fn test_next_query() {
        let object_list = init_object_list(Some(String::from("foo3")), vec![]);

        let query = object_list.next_query();

        assert!(query.is_some());
        let inner_query = query.unwrap();
        assert_eq!(
            inner_query.get("key1"),
            Some(&QueryValue::from_static("value1"))
        );
        assert_eq!(
            inner_query.get("continuation-token"),
            Some(&QueryValue::from_static("foo3"))
        );

        let object_list = init_object_list(None, vec![]);
        let query = object_list.next_query();
        assert!(query.is_none());
    }

    #[test]
    fn test_object_iter_in_list() {
        let bucket = Arc::new("abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap());
        let object_list = init_object_list(
            None,
            vec![
                Object::new(
                    Arc::clone(&bucket),
                    "key1".parse().unwrap(),
                    DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp_opt(123000, 0).unwrap(),
                        Utc,
                    ),
                    "foo3".into(),
                    "foo4".into(),
                    100,
                    StorageClass::IA,
                ),
                Object::new(
                    Arc::clone(&bucket),
                    "key2".parse().unwrap(),
                    DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp_opt(123000, 0).unwrap(),
                        Utc,
                    ),
                    "foo3".into(),
                    "foo4".into(),
                    100,
                    StorageClass::IA,
                ),
            ],
        );

        let mut iter = object_list.object_iter();
        let first = iter.next();
        assert!(first.is_some());
        assert_eq!(first.unwrap().base.path().as_ref(), "key1");

        let second = iter.next();
        assert!(second.is_some());
        assert_eq!(second.unwrap().base.path().as_ref(), "key2");

        let third = iter.next();
        assert!(third.is_none());
    }

    #[test]
    fn test_common_prefixes() {
        let mut object_list = init_object_list(None, vec![]);
        let list = object_list.common_prefixes();
        assert!(list.len() == 0);

        object_list.set_common_prefixes(["abc/".parse().unwrap(), "cde/".parse().unwrap()]);
        let list = object_list.common_prefixes();

        assert!(list.len() == 2);
        assert!(list[0] == "abc/");
        assert!(list[1] == "cde/");
    }

    #[test]
    fn test_object_new() {
        let bucket = Arc::new("abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap());
        let object = Object::<ArcPointer>::new(
            bucket,
            "foo2".parse().unwrap(),
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(123000, 0).unwrap(), Utc),
            "foo3".into(),
            "foo4".into(),
            100,
            StorageClass::IA,
        );

        assert_eq!(object.base.path().as_ref(), "foo2");
        assert_eq!(object.last_modified.to_string(), "1970-01-02 10:10:00 UTC");
        assert_eq!(object.etag, "foo3");
        assert_eq!(object._type, "foo4");
        assert_eq!(object.size, 100);
        assert_eq!(object.storage_class, StorageClass::IA);
    }

    #[test]
    fn test_object_builder() {
        let bucket = Arc::new(BucketBase::new(
            "abc".parse().unwrap(),
            "qingdao".parse().unwrap(),
        ));
        let object = ObjectBuilder::<ArcPointer>::new(bucket, "abc".parse::<ObjectPath>().unwrap())
            .last_modified(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(123000, 0).unwrap(),
                Utc,
            ))
            .etag("foo1".to_owned())
            .set_type("foo2".to_owned())
            .size(123)
            .storage_class(StorageClass::IA)
            .build();

        assert_eq!(object.base.path().as_ref(), "abc");
        assert_eq!(object.last_modified.to_string(), "1970-01-02 10:10:00 UTC");
        assert_eq!(object.etag, "foo1");
        assert_eq!(object._type, "foo2");
        assert_eq!(object.size, 123);
        assert_eq!(object.storage_class, StorageClass::IA);
    }
}

#[cfg(feature = "blocking")]
#[cfg(test)]
mod blocking_tests {
    use std::rc::Rc;

    use chrono::{DateTime, NaiveDateTime, Utc};

    use crate::builder::RcPointer;

    use super::super::{Object, StorageClass};

    fn init_object(
        bucket: &str,
        path: &'static str,
        last_modified: i64,
        etag: &'static str,
        _type: &'static str,
        size: u64,
        storage_class: StorageClass,
    ) -> Object<RcPointer> {
        let bucket = Rc::new(bucket.parse().unwrap());
        Object::<RcPointer>::new(
            bucket,
            path.parse().unwrap(),
            DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_opt(last_modified, 0).unwrap(),
                Utc,
            ),
            etag.into(),
            _type.into(),
            size,
            storage_class,
        )
    }

    #[test]
    fn test_object_eq() {
        let object1 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            StorageClass::Archive,
        );

        let object2 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            StorageClass::Archive,
        );

        assert!(object1 == object2);

        let object3 = init_object(
            "abc2.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            StorageClass::Archive,
        );

        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo2",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            StorageClass::Archive,
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123009,
            "efoo1",
            "tyfoo1",
            12,
            StorageClass::Archive,
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo2",
            "tyfoo1",
            12,
            StorageClass::Archive,
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo3",
            12,
            StorageClass::Archive,
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            256,
            StorageClass::Archive,
        );
        assert!(object1 != object3);

        let object3 = init_object(
            "abc.oss-cn-shanghai.aliyuncs.com",
            "foo1",
            123000,
            "efoo1",
            "tyfoo1",
            12,
            StorageClass::IA,
        );
        assert!(object1 != object3);
    }
}

mod list_error {
    use std::error::Error;

    use crate::types::object::InvalidObjectDir;

    use super::super::{ObjectListError, ObjectListErrorKind};

    #[test]
    fn key_count() {
        let err = i32::from_str_radix("a12", 10).unwrap_err();

        let err = ObjectListError {
            source: "foo".to_string(),
            kind: ObjectListErrorKind::KeyCount(err),
        };
        assert_eq!(format!("{err}"), "parse key-count failed, gived str: foo");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "invalid digit found in string"
        );
    }

    #[test]
    fn max_keys() {
        let err = i32::from_str_radix("a12", 10).unwrap_err();

        let err = ObjectListError {
            source: "foo".to_string(),
            kind: ObjectListErrorKind::MaxKeys(err),
        };
        assert_eq!(format!("{err}"), "parse max-keys failed, gived str: foo");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "invalid digit found in string"
        );
    }

    #[test]
    fn prefix() {
        let err = ObjectListError {
            source: "foo".to_string(),
            kind: ObjectListErrorKind::Prefix(InvalidObjectDir { _priv: () }),
        };
        assert_eq!(format!("{err}"), "parse prefix failed, gived str: foo");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "object-dir must end with `/`"
        );
    }

    #[test]
    fn common_prefix() {
        let err = ObjectListError {
            source: "foo".to_string(),
            kind: ObjectListErrorKind::CommonPrefix(InvalidObjectDir { _priv: () }),
        };
        assert_eq!(
            format!("{err}"),
            "parse common-prefix failed, gived str: foo"
        );
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "object-dir must end with `/`"
        );
    }
}
