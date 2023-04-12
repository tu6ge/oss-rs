mod tests_get_std {
    use reqwest::Url;
    use std::sync::Arc;

    use crate::file::GetStd;
    use crate::{
        builder::ArcPointer,
        object::Object,
        types::{object::ObjectBase, CanonicalizedResource},
    };

    #[test]
    fn test_object_base() {
        let mut path = ObjectBase::<ArcPointer>::default();
        path.set_path("path1").unwrap();
        let bucket = Arc::new("abc.qingdao".parse().unwrap());
        path.set_bucket(bucket);

        let res = path.get_std();

        assert!(res.is_some());
        let (url, resource) = res.unwrap();

        assert_eq!(
            url,
            Url::parse("https://abc.oss-cn-qingdao.aliyuncs.com/path1").unwrap()
        );
        assert_eq!(resource, CanonicalizedResource::new("/abc/path1"));
    }

    #[test]
    fn test_object() {
        let object = Object::<ArcPointer>::default();
        let res = object.get_std();
        assert!(res.is_some());
        let (url, resource) = res.unwrap();

        assert_eq!(
            url,
            Url::parse("https://a.oss-cn-hangzhou.aliyuncs.com/").unwrap()
        );
        assert_eq!(resource, CanonicalizedResource::new("/a/"));
    }

    #[test]
    fn test_object_ref() {
        let object = Object::<ArcPointer>::default();
        let res = GetStd::get_std(&object);
        assert!(res.is_some());
        let (url, resource) = res.unwrap();

        assert_eq!(
            url,
            Url::parse("https://a.oss-cn-hangzhou.aliyuncs.com/").unwrap()
        );
        assert_eq!(resource, CanonicalizedResource::new("/a/"));
    }
}

mod test_get_std_with_path {
    use reqwest::Url;

    use crate::{
        bucket::Bucket,
        builder::ArcPointer,
        client::ClientArc,
        file::GetStdWithPath,
        object::Object,
        types::{object::ObjectBase, CanonicalizedResource},
        ObjectPath,
    };

    fn assert_url_resource(
        result: Option<(Url, CanonicalizedResource)>,
        url: &str,
        resource: &str,
    ) {
        let (u, r) = result.unwrap();

        assert_eq!(u, Url::parse(url).unwrap());
        assert_eq!(r, resource);
    }

    #[test]
    fn test_client() {
        let client = ClientArc::test_init();
        assert_url_resource(
            client.get_std_with_path("path1".to_owned()),
            "https://bar.oss-cn-qingdao.aliyuncs.com/path1",
            "/bar/path1",
        );

        assert_url_resource(
            client.get_std_with_path("path1"),
            "https://bar.oss-cn-qingdao.aliyuncs.com/path1",
            "/bar/path1",
        );

        assert_url_resource(
            client.get_std_with_path("path1".parse::<ObjectPath>().unwrap()),
            "https://bar.oss-cn-qingdao.aliyuncs.com/path1",
            "/bar/path1",
        );

        assert_url_resource(
            client.get_std_with_path(Object::<ArcPointer>::test_path("path1")),
            "https://bar.oss-cn-qingdao.aliyuncs.com/path1",
            "/bar/path1",
        );
    }

    #[test]
    fn test_as_bucket_base() {
        let bucket = Bucket::<ArcPointer>::default();
        assert_url_resource(
            bucket.get_std_with_path("path1".to_string()),
            "https://a.oss-cn-hangzhou.aliyuncs.com/path1",
            "/a/path1",
        );

        assert_url_resource(
            bucket.get_std_with_path("path1"),
            "https://a.oss-cn-hangzhou.aliyuncs.com/path1",
            "/a/path1",
        );
        assert_url_resource(
            bucket.get_std_with_path("path1".parse::<ObjectPath>().unwrap()),
            "https://a.oss-cn-hangzhou.aliyuncs.com/path1",
            "/a/path1",
        );

        let path = "path1".parse::<ObjectPath>().unwrap();
        assert_url_resource(
            bucket.get_std_with_path(&path),
            "https://a.oss-cn-hangzhou.aliyuncs.com/path1",
            "/a/path1",
        );
    }

    #[test]
    fn test_bucket() {
        let bucket = Bucket::<ArcPointer>::default();
        assert_url_resource(
            bucket.get_std_with_path({
                let mut obj = ObjectBase::<ArcPointer>::default();
                obj.set_path("path1").unwrap();
                obj
            }),
            "https://a.oss-cn-hangzhou.aliyuncs.com/path1",
            "/a/path1",
        );

        let mut obj = ObjectBase::<ArcPointer>::default();
        obj.set_path("path1").unwrap();

        assert_url_resource(
            bucket.get_std_with_path(&obj),
            "https://a.oss-cn-hangzhou.aliyuncs.com/path1",
            "/a/path1",
        );
    }
}

mod test_try {
    use std::sync::Arc;

    use crate::builder::ArcPointer;
    use crate::file::{FileError, Files};
    use crate::types::object::{ObjectBase, ObjectPath};
    use crate::Client;

    fn init_client() -> Client {
        use std::env::set_var;
        set_var("ALIYUN_KEY_ID", "foo1");
        set_var("ALIYUN_KEY_SECRET", "foo2");
        set_var("ALIYUN_ENDPOINT", "qingdao");
        set_var("ALIYUN_BUCKET", "foo4");
        Client::from_env().unwrap()
    }

    #[tokio::test]
    async fn try_delete() {
        let client = init_client();

        struct MyPath;
        impl TryFrom<MyPath> for ObjectBase<ArcPointer> {
            type Error = MyError;
            fn try_from(_path: MyPath) -> Result<Self, Self::Error> {
                Ok(ObjectBase::<ArcPointer>::new2(
                    Arc::new("abc".parse().unwrap()),
                    "cde".parse().unwrap(),
                ))
            }
        }

        struct MyError;

        impl Into<FileError> for MyError {
            fn into(self) -> FileError {
                FileError::FileTypeNotFound
            }
        }

        //let _ = FileAs::<ObjectPath>::delete_object_as(&client, "abc".to_string()).await;
        let _ = client
            .delete_object("abc".parse::<ObjectPath>().unwrap())
            .await;
    }
}

mod error {
    use std::io::ErrorKind;

    use http::HeaderValue;

    use crate::{builder::BuilderError, file::FileError, types::object::InvalidObjectPath};

    #[test]
    fn test_path() {
        let err = FileError::Path(InvalidObjectPath { _priv: () });

        assert_eq!(format!("{err}"), "invalid object path");

        fn bar() -> FileError {
            InvalidObjectPath { _priv: () }.into()
        }
        assert_eq!(format!("{:?}", bar()), "Path(InvalidObjectPath)");
    }

    #[test]
    fn test_io() {
        let err = FileError::Io(std::io::Error::new(ErrorKind::Other, "oh no!"));

        assert_eq!(format!("{err}"), "oh no!");

        fn bar() -> FileError {
            std::io::Error::new(ErrorKind::Other, "oh no!").into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "Io(Custom { kind: Other, error: \"oh no!\" })"
        );
    }

    #[test]
    fn test_header_value() {
        let err = HeaderValue::from_bytes(b"\n").unwrap_err();
        let err = FileError::HeaderValue(err);

        assert_eq!(format!("{err}"), "failed to parse header value");

        fn bar() -> FileError {
            let err = HeaderValue::from_bytes(b"\n").unwrap_err();
            err.into()
        }
        assert_eq!(format!("{:?}", bar()), "HeaderValue(InvalidHeaderValue)");
    }

    #[test]
    fn test_build() {
        let err = FileError::Build(BuilderError::Bar);

        assert_eq!(format!("{err}"), "bar");

        fn bar() -> FileError {
            BuilderError::Bar.into()
        }
        assert_eq!(format!("{:?}", bar()), "Build(Bar)");
    }

    #[test]
    fn test_display() {
        assert_eq!(
            format!("{}", FileError::FileTypeNotFound),
            "Failed to get file type"
        );
        assert_eq!(format!("{}", FileError::EtagNotFound), "Failed to get etag");
        assert_eq!(
            format!("{}", FileError::NotFoundCanonicalizedResource),
            "Not found CanonicalizedResource"
        );
    }
}
