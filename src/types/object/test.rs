#[cfg(feature = "core")]
mod test_core {
    use std::path::Path;
    use std::sync::Arc;

    use reqwest::Url;

    use crate::builder::ArcPointer;
    use crate::config::{BucketBase, InvalidBucketBase};
    use crate::types::object::InvalidObjectDir;
    use crate::types::object::{InvalidObjectBase, InvalidObjectPath, ObjectBase, OssFullUrl};
    use crate::{BucketName, ObjectDir, ObjectPath};

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
        assert!(matches!(object, InvalidObjectBase::Bucket(_)));

        let object =
            ObjectBase::<ArcPointer>::from_bucket_name("foo1", "-q-", "img1.jpg").unwrap_err();
        assert!(matches!(object, InvalidObjectBase::Bucket(_)));
    }

    #[test]
    fn object_base_debug() {
        let object = ObjectBase::<ArcPointer>::default();
        assert_eq!(format!("{object:?}"), "ObjectBase { bucket: BucketBase { endpoint: CnHangzhou, name: BucketName(\"a\") }, path: ObjectPathInner(\"\") }");
    }

    #[test]
    fn test_invalid_obj_base() {
        let bucket = InvalidObjectBase::Bucket(InvalidBucketBase::Tacitly);

        assert_eq!(
            format!("{bucket}"),
            "bucket url must like with https://yyy.xxx.aliyuncs.com"
        );

        let path = InvalidObjectBase::Path(InvalidObjectPath {});
        assert_eq!(format!("{path}"), "invalid object path");
        assert_eq!(format!("{path:?}"), "Path(InvalidObjectPath)");

        let base = ObjectBase::<ArcPointer>::try_from_bucket("-a", "path1").unwrap_err();
        assert!(matches!(base, InvalidObjectBase::Bucket(_)));

        let base = ObjectBase::<ArcPointer>::try_from_bucket("abc.qingdao", "/path1").unwrap_err();
        assert!(matches!(base, InvalidObjectBase::Path(_)));
    }

    #[test]
    fn test_path2object_path() {
        let path = Path::new("path2/file_name");
        let obj_path = ObjectPath::try_from(path).unwrap();
        assert_eq!(obj_path, "path2/file_name");
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
        assert!(matches!(dir, InvalidObjectDir {}));
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
        let err = InvalidObjectDir {};
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
        let err = InvalidObjectPath {};
        assert_eq!(format!("{err:?}"), "InvalidObjectPath");
    }

    #[test]
    fn test_url_from_oss() {
        use crate::EndPoint;
        let endpoint = EndPoint::CnQingdao;
        let bucket = BucketName::new("foo").unwrap();
        let path = ObjectPath::new("file1").unwrap();

        let url = Url::from_oss(&endpoint, &bucket, &path);

        assert_eq!(
            url,
            Url::parse("https://foo.oss-cn-qingdao.aliyuncs.com/file1").unwrap()
        );
    }
}

#[cfg(feature = "blocking")]
mod blocking_tests {
    use crate::builder::RcPointer;
    use crate::types::object::ObjectBase;

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
