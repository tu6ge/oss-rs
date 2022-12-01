use crate::errors::{OssError, OssService};

#[test]
fn test_message() {
    let err1 = OssError::Input("bar".to_string());

    assert_eq!(err1.message(), "input error: bar".to_string());

    let oss_err = OssError::OssService(OssService {
        code: "OSS_TEST_CODE".to_string(),
        message: "foo_msg".to_string(),
        request_id: "foo_req_id".to_string(),
    });

    assert_eq!(oss_err.message(), "foo_msg".to_string());
}

#[test]
fn test_oss_service_fmt() {
    let oss_err = OssService {
        code: "OSS_TEST_CODE".to_string(),
        message: "foo_msg".to_string(),
        request_id: "foo_req_id".to_string(),
    };

    assert_eq!(
        format!("{}", oss_err),
        "OssService { code: \"OSS_TEST_CODE\", message: \"foo_msg\", request_id: \"foo_req_id\" }"
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
    let service = OssService::new(content);
    assert_eq!(service.code, format!("RequestTimeTooSkewed"));
    assert_eq!(service.message, format!("bar"));
    assert_eq!(service.request_id, format!("63145DB90BFD85303279D56B"))
}

//use test::Bencher;
