#[cfg(feature = "core")]
mod test_core {
    use std::error::Error;
    use std::path::Path;

    use std::sync::Arc;

    use reqwest::Url;

    use crate::builder::ArcPointer;
    use crate::config::BucketBase;
    use crate::types::object::base::invalid::InvalidObjectBaseKind;
    use crate::types::object::{FromOss, InvalidObjectDir, ObjectPathInner};
    use crate::types::object::{InvalidObjectBase, InvalidObjectPath, ObjectBase};
    use crate::{BucketName, EndPoint, ObjectDir, ObjectPath};

    #[test]
    fn object_from_ref_bucket() {
        use std::env::set_var;
        set_var("ALIYUN_ENDPOINT", "qingdao");
        set_var("ALIYUN_BUCKET", "foo1");
        let object = ObjectBase::<ArcPointer>::from_ref_bucket(
            Arc::new(BucketBase::from_env().unwrap()),
            "img1.jpg",
        )
        .unwrap();

        assert_eq!(object.path(), "img1.jpg");
    }

    #[test]
    fn object_from_bucket_name() {
        let object =
            ObjectBase::<ArcPointer>::from_bucket_name("foo1", "qingdao", "img1.jpg").unwrap();

        assert_eq!(object.path(), "img1.jpg");

        let object =
            ObjectBase::<ArcPointer>::from_bucket_name("-foo1", "qingdao", "img1.jpg").unwrap_err();
        assert!(matches!(object.kind, InvalidObjectBaseKind::BucketName(_)));

        let object =
            ObjectBase::<ArcPointer>::from_bucket_name("foo1", "-q-", "img1.jpg").unwrap_err();
        assert!(matches!(object.kind, InvalidObjectBaseKind::EndPoint(_)));
    }

    #[test]
    fn init_with_bucket() {
        let bucket = Arc::new("abc.qingdao".parse().unwrap());
        let base = ObjectBase::<ArcPointer>::init_with_bucket(bucket);

        assert!(base.bucket.get_name().as_ref() == "abc");
    }

    #[test]
    fn object_base_debug() {
        let object = ObjectBase::<ArcPointer>::default();
        assert_eq!(format!("{object:?}"), "ObjectBase { bucket: BucketBase { endpoint: EndPoint { kind: CnHangzhou, is_internal: false }, name: BucketName(\"a\") }, path: ObjectPathInner(\"\") }");
    }

    #[test]
    fn test_invalid_obj_base() {
        let bucket = InvalidObjectBase {
            source: "aaa".to_string(),
            kind: InvalidObjectBaseKind::Bar,
        };

        assert_eq!(format!("{bucket}"), "get object base faild, source: aaa");
        assert!(bucket.source().is_none());

        let path = InvalidObjectBase {
            source: "aaa".to_string(),
            kind: InvalidObjectBaseKind::Path(InvalidObjectPath::new()),
        };
        assert_eq!(format!("{}", path.source().unwrap()), "invalid object path");
        assert_eq!(
            format!("{path:?}"),
            "InvalidObjectBase { source: \"aaa\", kind: Path(InvalidObjectPath) }"
        );

        let base = ObjectBase::<ArcPointer>::try_from_bucket("-a", "path1").unwrap_err();
        assert!(matches!(base.kind, InvalidObjectBaseKind::Bucket(_)));

        let base = ObjectBase::<ArcPointer>::try_from_bucket("abc.qingdao", "/path1").unwrap_err();
        assert!(matches!(base.kind, InvalidObjectBaseKind::Path(_)));
    }

    #[test]
    fn test_path2object_path() {
        use std::str::FromStr;

        let path = Path::new("path2/file_name");
        let obj_path = ObjectPath::try_from(path).unwrap();
        assert_eq!(obj_path, "path2/file_name");

        let str = Box::new(String::from_str("abc").unwrap());
        let path = ObjectPath::try_from(str).unwrap();
        assert_eq!(path, "abc");

        let buff = "abc".as_bytes();
        let path = ObjectPathInner::try_from(buff).unwrap();
        assert_eq!(path, "abc");
    }

    #[test]
    fn test_obj_dir_display() {
        let dir = ObjectDir::new("path/").unwrap();

        assert_eq!(format!("{dir}"), "path/");
    }

    #[test]
    fn test_obj_dir_eq() {
        assert!("path/" == ObjectDir::new("path/").unwrap());
        assert!("path/".to_string() == ObjectDir::new("path/").unwrap());
        assert!(ObjectDir::new("path/").unwrap() == "path/".to_string());
    }

    #[test]
    fn test_dir_from_str() {
        let dir = ObjectDir::try_from("path/").unwrap();
        assert_eq!(dir, ObjectDir::new("path/").unwrap());

        let dir = ObjectDir::try_from("path").unwrap_err();
        assert!(matches!(dir, InvalidObjectDir { .. }));
    }
    #[test]
    fn test_dir_from_path() {
        use std::path::Path;
        let path = Path::new("path/");
        let obj_path = ObjectDir::try_from(path).unwrap();
        assert_eq!(obj_path, ObjectDir::new("path/").unwrap());
    }

    #[test]
    fn test_invalid_dir_debug() {
        let err = InvalidObjectDir::new();
        assert_eq!(
            format!("{err}"),
            "object-dir must end with `/`, and not start with `/`,`.`"
        );
        assert_eq!(format!("{err:?}"), "InvalidObjectDir");
    }

    #[test]
    fn test_object_path_display() {
        let path = ObjectPath::new("path1").unwrap();
        assert_eq!(format!("{path}"), "path1");
    }
    #[test]
    fn test_object_path_eq() {
        assert!(ObjectPath::new("path1").unwrap() == "path1");
        assert!(ObjectPath::new("path1").unwrap() == "path1".to_string());
        assert!("path1" == ObjectPath::new("path1").unwrap());
        assert!("path1".to_string() == ObjectPath::new("path1").unwrap());
    }

    #[test]
    fn test_invalid_path_debug() {
        let err = InvalidObjectPath::new();
        assert_eq!(format!("{err}"), "invalid object path");
        assert_eq!(format!("{err:?}"), "InvalidObjectPath");
    }

    #[test]
    fn test_url_from_oss() {
        use crate::EndPoint;
        let endpoint = EndPoint::CN_QINGDAO;
        let bucket = BucketName::new("foo").unwrap();
        let path = ObjectPath::new("file1").unwrap();

        let url = Url::from_oss(&endpoint, &bucket, &path);

        assert_eq!(
            url,
            Url::parse("https://foo.oss-cn-qingdao.aliyuncs.com/file1").unwrap()
        );
    }

    #[test]
    fn to_sign_url() {
        let object = ObjectBase::<ArcPointer>::from_bucket(
            BucketBase::new("abc".parse().unwrap(), EndPoint::CN_BEIJING),
            "path1.png",
        )
        .unwrap();

        let url = object.to_sign_url(&"key".into(), &"secret".into(), 1234567890);
        assert_eq!(url.as_str(), "https://abc.oss-cn-beijing.aliyuncs.com/path1.png?OSSAccessKeyId=key&Expires=1234567890&Signature=Kpqvd4gWgHNlkCfcYzRiHmDO%2Fvw%3D");
    }
}

#[cfg(feature = "blocking")]
mod blocking_tests {
    use crate::builder::RcPointer;
    use crate::types::object::ObjectBase;
    use crate::EndPoint;

    fn crate_object_base(bucket: &'static str, path: &'static str) -> ObjectBase<RcPointer> {
        use std::rc::Rc;

        let bucket = bucket.parse().unwrap();

        let object = ObjectBase::<RcPointer>::new2(Rc::new(bucket), path.try_into().unwrap());
        object
    }

    #[test]
    fn test_get_object_info() {
        let object = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "bar");

        assert_eq!(object.bucket_name(), &"abc");
        assert_eq!(object.endpoint(), &EndPoint::CN_SHANGHAI);
        assert_eq!(object.path(), "bar");
    }

    #[test]
    fn test_object_base_eq() {
        let object1 = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "bar");
        let object2 = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "bar");
        let object3 = crate_object_base("abc.oss-cn-qingdao.aliyuncs.com", "bar");
        let object4 = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "ba2");
        assert!(object1 == object2);
        assert!(object1 != object3);
        assert!(object1 != object4);
    }
}
