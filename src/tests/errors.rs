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
    use crate::bucket::{ExtractItemError, ExtractItemErrorKind};
    use crate::builder::{BuilderError, BuilderErrorKind};
    use crate::decode::InnerItemError;
    use crate::object::{BuildInItemError, ExtractListError, ExtractListErrorKind};
    use crate::types::object::{InvalidObjectDir, InvalidObjectPath};
    use crate::types::{InvalidBucketName, InvalidEndPoint};
    use crate::Error;
    #[test]
    fn test_dotenv() {
        let err = Error::Dotenv(dotenv::Error::LineParse("abc".to_string(), 1));

        assert_eq!(
            format!("{err}"),
            "Error parsing line: 'abc', error at line index: 1"
        );
        assert_eq!(format!("{err:?}"), "Dotenv(LineParse(\"abc\", 1))");

        fn bar() -> Error {
            dotenv::Error::LineParse("abc".to_string(), 1).into()
        }
        assert_eq!(format!("{:?}", bar()), "Dotenv(LineParse(\"abc\", 1))");
    }

    #[test]
    fn test_builder() {
        let err = Error::BuilderError(BuilderError {
            kind: BuilderErrorKind::Bar,
        });
        assert_eq!(format!("{err}"), "bar");

        fn bar() -> Error {
            BuilderError {
                kind: BuilderErrorKind::Bar,
            }
            .into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "BuilderError(BuilderError { kind: Bar })"
        );
    }

    #[test]
    fn test_endpoint() {
        let err = Error::InvalidEndPoint(InvalidEndPoint { _priv: () });
        assert_eq!(
            format!("{err}"),
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        );

        fn bar() -> Error {
            InvalidEndPoint { _priv: () }.into()
        }
        assert_eq!(format!("{:?}", bar()), "InvalidEndPoint(InvalidEndPoint)");
    }

    #[test]
    fn test_bucket_name() {
        let err = Error::InvalidBucketName(InvalidBucketName { _priv: () });

        assert_eq!(
            format!("{err}"),
            "bucket name only allow `alphabet, digit, -`, and must not with `-` prefix or `-` suffix"
        );

        fn bar() -> Error {
            InvalidBucketName { _priv: () }.into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "InvalidBucketName(InvalidBucketName)"
        );
    }

    #[test]
    fn test_config() {
        use crate::config::InvalidConfig;
        let err = Error::InvalidConfig(InvalidConfig::test_bucket());

        assert_eq!(format!("{err}"), "get config faild, source: bar");

        //   assert_eq!(
        //     format!("{}", err.source().unwrap()),
        //     "bucket name only allow `alphabet, digit, -`, and must not with `-` prefix or `-` suffix"
        // );

        fn bar() -> Error {
            InvalidConfig::test_bucket().into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "InvalidConfig(InvalidConfig { source: \"bar\", kind: BucketName(InvalidBucketName) })"
        );
    }

    #[test]
    fn test_object_path() {
        let err = Error::InvalidObjectPath(InvalidObjectPath { _priv: () });

        assert_eq!(format!("{err}"), "invalid object path");

        fn bar() -> Error {
            InvalidObjectPath { _priv: () }.into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "InvalidObjectPath(InvalidObjectPath)"
        );
    }

    #[test]
    fn test_object_dir() {
        let err = Error::InvalidObjectDir(InvalidObjectDir { _priv: () });

        assert_eq!(
            format!("{err}"),
            "object-dir must end with `/`, and not start with `/`,`.`"
        );

        fn bar() -> Error {
            InvalidObjectDir { _priv: () }.into()
        }
        assert_eq!(format!("{:?}", bar()), "InvalidObjectDir(InvalidObjectDir)");
    }

    // #[test]
    // #[cfg(feature = "decode")]
    // fn test_inner_item() {
    //     use crate::decode::InnerItemError;

    //     let err = Error::InnerItemError(InnerItemError::new());

    //     assert_eq!(
    //         format!("{err}"),
    //         "decode xml to object has error, info: foo"
    //     );

    //     fn bar() -> Error {
    //       InnerItemError::new().into()
    //     }
    //     assert_eq!(
    //         format!("{:?}", bar()),
    //         "InnerItemError(InnerItemError(\"foo\"))"
    //     );
    // }
    // #[test]
    // #[cfg(feature = "decode")]
    // fn test_inner_list() {
    //     use crate::decode::InnerListError;

    //     let err = Error::InnerListError(InnerListError::from_xml());

    //     assert_eq!(format!("{err}"), "decode xml faild, quick_xml error");

    //     fn bar() -> Error {
    //         InnerListError::from_xml().into()
    //     }
    //     assert_eq!(
    //         format!("{:?}", bar()),
    //         "InnerListError(InnerListError { kind: Xml(TextNotFound) })"
    //     );
    // }

    #[test]
    fn test_build_in_iter_error() {
        let err = Error::BuildInItemError(BuildInItemError::test_new());

        assert_eq!(
            format!("{err}"),
            "parse storage-class failed, gived str: foo"
        );

        fn bar() -> Error {
            BuildInItemError::test_new().into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "BuildInItemError(BuildInItemError { source: \"foo\", kind: InvalidStorageClass })"
        );
    }

    #[test]
    fn test_extract_list() {
        let err = Error::ExtractList(ExtractListError {
            kind: ExtractListErrorKind::NoMoreFile,
        });

        assert_eq!(format!("{err}"), "no more file");

        fn bar() -> Error {
            ExtractListError {
                kind: ExtractListErrorKind::NoMoreFile,
            }
            .into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "ExtractList(ExtractListError { kind: NoMoreFile })"
        );
    }

    #[test]
    fn test_extract_item() {
        let err = Error::ExtractItem(ExtractItemError {
            kind: ExtractItemErrorKind::Decode(InnerItemError::new()),
        });

        assert_eq!(format!("{err}"), "decode xml failed");

        fn bar() -> Error {
            ExtractItemError {
                kind: ExtractItemErrorKind::Decode(InnerItemError::new()),
            }
            .into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "ExtractItem(ExtractItemError { kind: Decode(InnerItemError(MyError)) })"
        );
    }
}
