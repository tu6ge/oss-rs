use http::StatusCode;

use crate::errors::OssService;

#[test]
fn test_oss_service_fmt() {
    let oss_err = OssService {
        code: "OSS_TEST_CODE".to_string(),
        status: StatusCode::default(),
        message: "foo_msg".to_string(),
        request_id: "foo_req_id".to_string(),
    };

    assert_eq!(
        format!("{}", oss_err),
        "OssService { code: \"OSS_TEST_CODE\", status: 200, message: \"foo_msg\", request_id: \"foo_req_id\" }"
            .to_string()
    );
}

#[test]
fn test_oss_service_new() {
    let content = r#"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
    <Error>
        <Code>RequestTimeTooSkewed</Code>
        <Message>bar</Message>
        <RequestId>63145DB90BFD85303279D56B</RequestId>
        <HostId>honglei123.oss-cn-shanghai.aliyuncs.com</HostId>
        <MaxAllowedSkewMilliseconds>900000</MaxAllowedSkewMilliseconds>
        <RequestTime>2022-09-04T07:11:33.000Z</RequestTime>
        <ServerTime>2022-09-04T08:11:37.000Z</ServerTime>
    </Error>
    "#;
    let service = OssService::new(content, &StatusCode::default());
    assert_eq!(service.code, format!("RequestTimeTooSkewed"));
    assert_eq!(service.message, format!("bar"));
    assert_eq!(service.request_id, format!("63145DB90BFD85303279D56B"))
}

mod debug {
    use std::error::Error as StdError;
    use std::io::{self, ErrorKind};

    use crate::auth::AuthError;
    use crate::bucket::{BucketError, ExtractItemError, ExtractItemErrorKind};
    use crate::builder::{BuilderError, BuilderErrorKind};
    use crate::decode::InnerItemError;
    use crate::file::FileError;
    use crate::object::{
        BuildInItemError, ExtractListError, ExtractListErrorKind, ObjectListError,
    };
    use crate::types::object::{InvalidObjectDir, InvalidObjectPath};
    use crate::types::{InvalidBucketName, InvalidEndPoint};
    use crate::Error;

    #[test]
    fn test_io() {
        let err = Error::from(io::Error::new(ErrorKind::Other, "foo"));
        assert_eq!(format!("{}", err), "io error");
        assert_eq!(format!("{}", err.source().unwrap()), "foo");
        assert_eq!(
            format!("{:?}", err),
            "OssError { kind: Io(Custom { kind: Other, error: \"foo\" }) }"
        );
    }
    #[test]
    fn test_dotenv() {
        let err = Error::from(dotenv::Error::LineParse("abc".to_string(), 1));

        assert_eq!(format!("{err}"), "dotenv error");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "Error parsing line: 'abc', error at line index: 1"
        );

        fn bar() -> Error {
            dotenv::Error::LineParse("abc".to_string(), 1).into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: Dotenv(LineParse(\"abc\", 1)) }"
        );
    }

    #[test]
    fn test_builder() {
        let err = Error::from(BuilderError {
            kind: BuilderErrorKind::Bar,
        });
        assert_eq!(format!("{err}"), "builder error");
        assert_eq!(format!("{}", err.source().unwrap()), "bar");

        fn bar() -> Error {
            BuilderError {
                kind: BuilderErrorKind::Bar,
            }
            .into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: Builder(BuilderError { kind: Bar }) }"
        );
    }

    #[test]
    fn test_endpoint() {
        let err = Error::from(InvalidEndPoint { _priv: () });
        assert_eq!(format!("{err}"), "invalid endpoint");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        );

        fn bar() -> Error {
            InvalidEndPoint { _priv: () }.into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: EndPoint(InvalidEndPoint) }"
        );
    }

    #[test]
    fn test_bucket_name() {
        let err = Error::from(InvalidBucketName { _priv: () });

        assert_eq!(format!("{err}"), "invalid bucket name");
        assert_eq!(
          format!("{}", err.source().unwrap()),
          "bucket name only allow `alphabet, digit, -`, and must not with `-` prefix or `-` suffix"
      );

        fn bar() -> Error {
            InvalidBucketName { _priv: () }.into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: BucketName(InvalidBucketName) }"
        );
    }

    #[test]
    fn test_config() {
        use crate::config::InvalidConfig;
        let err = Error::from(InvalidConfig::test_bucket());

        assert_eq!(format!("{err}"), "invalid config");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "get config faild, source: bar"
        );

        fn bar() -> Error {
            InvalidConfig::test_bucket().into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: Config(InvalidConfig { source: \"bar\", kind: BucketName(InvalidBucketName) }) }"
        );
    }

    #[test]
    fn test_object_path() {
        let err = Error::from(InvalidObjectPath { _priv: () });

        assert_eq!(format!("{err}"), "invalid object path");
        assert_eq!(format!("{}", err.source().unwrap()), "invalid object path");

        fn bar() -> Error {
            InvalidObjectPath { _priv: () }.into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: ObjectPath(InvalidObjectPath) }"
        );
    }

    #[test]
    fn test_object_dir() {
        let err = Error::from(InvalidObjectDir { _priv: () });

        assert_eq!(format!("{err}"), "invalid object dir");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "object-dir must end with `/`, and not start with `/`,`.`"
        );

        fn bar() -> Error {
            InvalidObjectDir { _priv: () }.into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: ObjectDir(InvalidObjectDir) }"
        );
    }

    #[test]
    #[cfg(feature = "decode")]
    fn test_inner_item() {
        use std::error::Error as StdError;

        use crate::decode::InnerItemError;

        let err = Error::from(InnerItemError::new());

        assert_eq!(format!("{err}"), "decode into list error");
        assert_eq!(format!("{}", err.source().unwrap()), "demo");

        fn bar() -> Error {
            InnerItemError::new().into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: InnerItem(InnerItemError(MyError)) }"
        );
    }
    #[test]
    #[cfg(feature = "decode")]
    fn test_inner_list() {
        use crate::decode::InnerListError;

        let err = Error::from(InnerListError::from_xml());

        assert_eq!(format!("{err}"), "decode into list error");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "Cannot read text, expecting Event::Text"
        );

        fn bar() -> Error {
            InnerListError::from_xml().into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: InnerList(InnerListError { kind: Xml(TextNotFound) }) }"
        );
    }

    #[test]
    fn test_build_in_iter_error() {
        let err = Error::from(BuildInItemError::test_new());

        assert_eq!(format!("{err}"), "build in item error");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "parse storage-class failed, gived str: foo"
        );

        fn bar() -> Error {
            BuildInItemError::test_new().into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: BuildInItemError(BuildInItemError { source: \"foo\", kind: InvalidStorageClass }) }"
        );
    }

    #[test]
    fn test_extract_list() {
        let err = Error::from(ExtractListError {
            kind: ExtractListErrorKind::NoMoreFile,
        });

        assert_eq!(format!("{err}"), "extract list error");
        assert_eq!(format!("{}", err.source().unwrap()), "no more file");

        fn bar() -> Error {
            ExtractListError {
                kind: ExtractListErrorKind::NoMoreFile,
            }
            .into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: ExtractList(ExtractListError { kind: NoMoreFile }) }"
        );
    }

    #[test]
    fn test_extract_item() {
        let err = Error::from(ExtractItemError {
            kind: ExtractItemErrorKind::Decode(InnerItemError::new()),
        });

        assert_eq!(format!("{err}"), "extract item error");
        assert_eq!(format!("{}", err.source().unwrap()), "decode xml failed");

        fn bar() -> Error {
            ExtractItemError {
                kind: ExtractItemErrorKind::Decode(InnerItemError::new()),
            }
            .into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "OssError { kind: ExtractItem(ExtractItemError { kind: Decode(InnerItemError(MyError)) }) }"
        );
    }

    #[test]
    fn file() {
        let err = Error::from(FileError::test_new());

        assert_eq!(format!("{err}"), "file error");
        assert_eq!(format!("{}", err.source().unwrap()), "failed to get etag");
        assert_eq!(
            format!("{err:?}"),
            "OssError { kind: File(FileError { kind: EtagNotFound }) }"
        );
    }

    #[test]
    fn auth() {
        let err = Error::from(AuthError::test_new());

        assert_eq!(format!("{err}"), "auth error");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "invalid canonicalized-resource"
        );
        assert_eq!(
            format!("{err:?}"),
            "OssError { kind: Auth(AuthError { kind: InvalidCanonicalizedResource }) }"
        );
    }

    #[test]
    fn bucket() {
        let err = Error::from(BucketError::test_new());

        assert_eq!(format!("{err}"), "bucket error");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "decode bucket xml faild, gived str: foo"
        );
        assert_eq!(
            format!("{err:?}"),
            "OssError { kind: Bucket(BucketError { source: \"foo\", kind: InvalidStorageClass }) }"
        );
    }

    #[test]
    fn object_list() {
        let err = Error::from(ObjectListError::test_new());

        assert_eq!(format!("{err}"), "object list error");
        assert_eq!(
            format!("{}", err.source().unwrap()),
            "parse bar failed, gived str: foo"
        );
        assert_eq!(
            format!("{err:?}"),
            "OssError { kind: ObjectList(ObjectListError { source: \"foo\", kind: Bar }) }"
        );
    }
}
