use http::header::CONTENT_TYPE;
use http::{HeaderValue, Method};

use crate::builder::ClientWithMiddleware;
use crate::file::AlignBuilder;
use crate::{client::Client, types::CanonicalizedResource, EndPoint};

#[test]
fn test_get_bucket_url() {
    let client = Client::<ClientWithMiddleware>::new(
        "foo1".into(),
        "foo2".into(),
        EndPoint::CnQingdao,
        "foo4".parse().unwrap(),
    );
    let url = client.get_bucket_url();
    assert_eq!(url.as_str(), "https://foo4.oss-cn-qingdao.aliyuncs.com/");
}

#[test]
fn test_builder_with_header() {
    let client = Client::<ClientWithMiddleware>::new(
        "foo1".into(),
        "foo2".into(),
        EndPoint::CnQingdao,
        "foo4".parse().unwrap(),
    );
    let url = "http://foo.example.net/foo".parse().unwrap();
    let resource = CanonicalizedResource::new("bar");
    let headers = vec![(CONTENT_TYPE, HeaderValue::from_static("application/json"))];
    let builder = client.builder_with_header(Method::POST, url, resource, headers);

    assert!(builder.is_ok());

    let request = builder.unwrap().build().unwrap();

    assert_eq!(request.method(), "POST");
    assert!(request.url().host().is_some());
    assert_eq!(request.url().path(), "/foo");
    assert_eq!(
        request.headers().get("content-type"),
        Some(&HeaderValue::from_str("application/json").unwrap())
    );
    assert_eq!(
        request.headers().get("accesskeyid"),
        Some(&HeaderValue::from_str("foo1").unwrap())
    );
    assert_eq!(
        request.headers().get("secretaccesskey"),
        Some(&HeaderValue::from_str("foo2").unwrap())
    );
    assert_eq!(
        request.headers().get("verb"),
        Some(&HeaderValue::from_str("POST").unwrap())
    );
    assert_eq!(
        request.headers().get("date"),
        Some(&HeaderValue::from_str("Thu, 06 Oct 2022 20:40:00 GMT").unwrap())
    );
    assert_eq!(
        request.headers().get("canonicalizedresource"),
        Some(&HeaderValue::from_str("bar").unwrap())
    );
    assert_eq!(
        request.headers().get("authorization"),
        Some(&HeaderValue::from_str("OSS foo1:FUrk4hgj2yIB8lJpnsSub+CTC9M=").unwrap())
    );
}

#[cfg(feature = "blocking")]
#[test]
fn test_blocking_builder_with_header() {
    use crate::blocking::builder::ClientWithMiddleware;
    use crate::client::Client;
    use crate::file::blocking::AlignBuilder;
    let client = Client::<ClientWithMiddleware>::new(
        "foo1".into(),
        "foo2".into(),
        EndPoint::CnQingdao,
        "foo4".parse().unwrap(),
    );
    let url = "http://foo.example.net/foo".parse().unwrap();
    let resource = CanonicalizedResource::new("bar");
    let headers = vec![(CONTENT_TYPE, HeaderValue::from_static("application/json"))];
    let builder = client.builder_with_header(Method::POST, url, resource, headers);

    assert!(builder.is_ok());

    let request = builder.unwrap().build().unwrap();

    assert_eq!(request.method(), "POST");
    assert!(request.url().host().is_some());
    assert_eq!(request.url().path(), "/foo");
    assert_eq!(
        request.headers().get("content-type"),
        Some(&HeaderValue::from_str("application/json").unwrap())
    );
    assert_eq!(
        request.headers().get("accesskeyid"),
        Some(&HeaderValue::from_str("foo1").unwrap())
    );
    assert_eq!(
        request.headers().get("secretaccesskey"),
        Some(&HeaderValue::from_str("foo2").unwrap())
    );
    assert_eq!(
        request.headers().get("verb"),
        Some(&HeaderValue::from_str("POST").unwrap())
    );
    assert_eq!(
        request.headers().get("date"),
        Some(&HeaderValue::from_str("Thu, 06 Oct 2022 20:40:00 GMT").unwrap())
    );
    assert_eq!(
        request.headers().get("canonicalizedresource"),
        Some(&HeaderValue::from_str("bar").unwrap())
    );
    assert_eq!(
        request.headers().get("authorization"),
        Some(&HeaderValue::from_str("OSS foo1:FUrk4hgj2yIB8lJpnsSub+CTC9M=").unwrap())
    );
}

mod handle_error {
    use crate::builder::{BuilderError, RequestHandler};
    use crate::errors::OssService;
    use http::Response as HttpResponse;
    use reqwest::Response;

    #[tokio::test]
    async fn test_async_has_error() {
        let http = HttpResponse::builder()
            .status(302)
            //.header("X-Custom-Foo", "Bar")
            .body(
                r#"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
            <Error>
                <Code>foo_code</Code>
                <Message>bar</Message>
                <RequestId>63145DB90BFD85303279D56B</RequestId>
                <HostId>honglei123.oss-cn-shanghai.aliyuncs.com</HostId>
                <MaxAllowedSkewMilliseconds>900000</MaxAllowedSkewMilliseconds>
                <RequestTime>2022-09-04T07:11:33.000Z</RequestTime>
                <ServerTime>2022-09-04T08:11:37.000Z</ServerTime>
            </Error>
            "#,
            )
            .unwrap();
        let response: Response = http.into();

        let res = response.handle_error().await;

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(err, BuilderError::OssService(OssService{code,..}) if code=="foo_code"));

        //mock.checkpoint();
    }

    #[tokio::test]
    async fn test_async_ok() {
        let http = HttpResponse::builder()
            .status(200)
            //.header("X-Custom-Foo", "Bar")
            .body("body_abc")
            .unwrap();
        let response: Response = http.into();

        let res = response.handle_error().await;
        assert!(res.is_ok());
        let ok = res.unwrap();
        assert_eq!(ok.status(), 200);
        assert_eq!(&ok.text().await.unwrap(), "body_abc");

        let http = HttpResponse::builder()
            .status(204)
            //.header("X-Custom-Foo", "Bar")
            .body("body_abc")
            .unwrap();
        let response: Response = http.into();

        let res = response.handle_error().await;
        assert!(res.is_ok());
        let ok = res.unwrap();
        assert_eq!(ok.status(), 204);
        assert_eq!(&ok.text().await.unwrap(), "body_abc");
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn test_blocking_has_error() {
        use crate::blocking::builder::BlockingReqeustHandler;
        use reqwest::blocking::Response as BlockingResponse;

        let http = HttpResponse::builder()
            .status(302)
            //.header("X-Custom-Foo", "Bar")
            .body(
                r#"<?xml version=\"1.0\" encoding=\"UTF-8\"?>
            <Error>
                <Code>foo_code</Code>
                <Message>bar</Message>
                <RequestId>63145DB90BFD85303279D56B</RequestId>
                <HostId>honglei123.oss-cn-shanghai.aliyuncs.com</HostId>
                <MaxAllowedSkewMilliseconds>900000</MaxAllowedSkewMilliseconds>
                <RequestTime>2022-09-04T07:11:33.000Z</RequestTime>
                <ServerTime>2022-09-04T08:11:37.000Z</ServerTime>
            </Error>
            "#,
            )
            .unwrap();
        let response: BlockingResponse = http.into();

        let res = response.handle_error();

        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(matches!(err, BuilderError::OssService(OssService{code,..}) if code=="foo_code"));
    }

    #[cfg(feature = "blocking")]
    #[test]
    fn test_blocking_ok() {
        use crate::blocking::builder::BlockingReqeustHandler;
        use reqwest::blocking::Response as BlockingResponse;

        let http = HttpResponse::builder()
            .status(200)
            //.header("X-Custom-Foo", "Bar")
            .body("body_abc")
            .unwrap();
        let response: BlockingResponse = http.into();

        let res = response.handle_error();
        assert!(res.is_ok());
        let ok = res.unwrap();
        assert_eq!(ok.status(), 200);
        assert_eq!(&ok.text().unwrap(), "body_abc");

        let http = HttpResponse::builder()
            .status(204)
            //.header("X-Custom-Foo", "Bar")
            .body("body_abc")
            .unwrap();
        let response: BlockingResponse = http.into();

        let res = response.handle_error();
        assert!(res.is_ok());
        let ok = res.unwrap();
        assert_eq!(ok.status(), 204);
        assert_eq!(&ok.text().unwrap(), "body_abc");
    }
}

// blocking mock 有错误
