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
    use crate::bucket::ExtractItemError;
    use crate::builder::BuilderError;
    use crate::decode::InnerItemError;
    use crate::object::{BuildInItemError, ExtractListError};
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
        let err = Error::BuilderError(BuilderError::Bar);
        assert_eq!(format!("{err}"), "bar");

        fn bar() -> Error {
            BuilderError::Bar.into()
        }
        assert_eq!(format!("{:?}", bar()), "BuilderError(Bar)");
    }

    #[test]
    fn test_endpoint() {
        let err = Error::InvalidEndPoint(InvalidEndPoint {});
        assert_eq!(
            format!("{err}"),
            "endpoint must not with `-` prefix or `-` suffix or `oss-` prefix"
        );

        fn bar() -> Error {
            InvalidEndPoint {}.into()
        }
        assert_eq!(format!("{:?}", bar()), "InvalidEndPoint(InvalidEndPoint)");
    }

    #[test]
    fn test_bucket_name() {
        let err = Error::InvalidBucketName(InvalidBucketName {});

        assert_eq!(
            format!("{err}"),
            "bucket 名称只允许小写字母、数字、短横线（-），且不能以短横线开头或结尾"
        );

        fn bar() -> Error {
            InvalidBucketName {}.into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "InvalidBucketName(InvalidBucketName)"
        );
    }

    #[test]
    fn test_config() {
        use crate::config::InvalidConfig::BucketName;
        let err = Error::InvalidConfig(BucketName(InvalidBucketName {}));

        assert_eq!(
            format!("{err}"),
            "bucket 名称只允许小写字母、数字、短横线（-），且不能以短横线开头或结尾"
        );

        fn bar() -> Error {
            BucketName(InvalidBucketName {}).into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "InvalidConfig(BucketName(InvalidBucketName))"
        );
    }

    #[test]
    fn test_object_path() {
        let err = Error::InvalidObjectPath(InvalidObjectPath {});

        assert_eq!(format!("{err}"), "invalid object path");

        fn bar() -> Error {
            InvalidObjectPath {}.into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "InvalidObjectPath(InvalidObjectPath)"
        );
    }

    #[test]
    fn test_object_dir() {
        let err = Error::InvalidObjectDir(InvalidObjectDir {});

        assert_eq!(format!("{err}"), "ObjectDir must end with `/`");

        fn bar() -> Error {
            InvalidObjectDir {}.into()
        }
        assert_eq!(format!("{:?}", bar()), "InvalidObjectDir(InvalidObjectDir)");
    }

    #[test]
    #[cfg(feature = "decode")]
    fn test_inner_item() {
        use crate::decode::InnerItemError;

        let err = Error::InnerItemError(InnerItemError("foo".to_string()));

        assert_eq!(
            format!("{err}"),
            "decode xml to object has error, info: foo"
        );

        fn bar() -> Error {
            InnerItemError("foo".to_string()).into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "InnerItemError(InnerItemError(\"foo\"))"
        );
    }
    #[test]
    #[cfg(feature = "decode")]
    fn test_inner_list() {
        use crate::decode::InnerListError;

        let err = Error::InnerListError(InnerListError::Custom("aaa".to_string()));

        assert_eq!(
            format!("{err}"),
            "decode xml to object list has error, info: aaa"
        );

        fn bar() -> Error {
            InnerListError::Custom("aaa".to_string()).into()
        }
        assert_eq!(format!("{:?}", bar()), "InnerListError(Custom(\"aaa\"))");
    }

    #[test]
    fn test_build_in_iter_error() {
        use BuildInItemError::*;

        let err = Error::BuildInItemError(InvalidStorageClass);

        assert_eq!(format!("{err}"), "invalid storage class");

        fn bar() -> Error {
            InvalidStorageClass.into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "BuildInItemError(InvalidStorageClass)"
        );
    }

    #[test]
    fn test_extract_list() {
        use ExtractListError::*;

        let err = Error::ExtractList(WithoutMore);

        assert_eq!(format!("{err}"), "Without More Content");

        fn bar() -> Error {
            WithoutMore.into()
        }
        assert_eq!(format!("{:?}", bar()), "ExtractList(WithoutMore)");
    }

    #[test]
    fn test_extract_item() {
        let err = Error::ExtractItem(ExtractItemError::Item(InnerItemError("foo".to_string())));

        assert_eq!(
            format!("{err}"),
            "decode xml to object has error, info: foo"
        );

        fn bar() -> Error {
            ExtractItemError::Item(InnerItemError("foo".to_string())).into()
        }
        assert_eq!(
            format!("{:?}", bar()),
            "ExtractItem(Item(InnerItemError(\"foo\")))"
        );
    }
}
