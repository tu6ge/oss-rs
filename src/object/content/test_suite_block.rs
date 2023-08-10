use http::HeaderValue;
use reqwest::blocking::{Body, Request, Response};

use crate::{blocking::builder::Middleware, builder::BuilderError};

#[derive(Debug)]
pub(super) struct InitMulti {}

impl Middleware for InitMulti {
    fn handle(&self, request: Request) -> Result<Response, BuilderError> {
        //println!("request {:?}", request);
        assert_eq!(request.method(), "POST");
        assert_eq!(
            request.url().as_str(),
            "https://bar.oss-cn-qingdao.aliyuncs.com/aaa.txt?uploads"
        );
        assert_eq!(
            request.headers().get("canonicalizedresource"),
            Some(&HeaderValue::from_str("/bar/aaa.txt?uploads").unwrap())
        );
        use http::response::Builder;
        let response = Builder::new()
            .status(200)
            .body(
                r#"<InitiateMultipartUploadResult>
        <Bucket>bucket_name</Bucket>
        <Key>aaa</Key>
        <UploadId>foo_upload_id</UploadId>
      </InitiateMultipartUploadResult>"#,
            )
            .unwrap();
        Ok(response.into())
    }
}

#[derive(Debug)]
pub(super) struct UploadPart {}

impl Middleware for UploadPart {
    fn handle(&self, request: Request) -> Result<Response, BuilderError> {
        //println!("request {:?}", request);
        assert_eq!(request.method(), "PUT");
        assert_eq!(
            request.url().as_str(),
            "https://bar.oss-cn-qingdao.aliyuncs.com/aaa.txt?partNumber=2&uploadId=foo_upload_id"
        );
        assert_eq!(
            request.headers().get("canonicalizedresource"),
            Some(
                &HeaderValue::from_str("/bar/aaa.txt?partNumber=2&uploadId=foo_upload_id").unwrap()
            )
        );
        let body = request.body().unwrap().clone();
        let xml = "bbb";
        let xml = Body::from(xml);
        assert_eq!(body.as_bytes(), xml.as_bytes());
        use http::response::Builder;
        let response = Builder::new()
            .status(200)
            .header("ETag", "foo_etag")
            .body("")
            .unwrap();
        Ok(response.into())
    }
}

#[derive(Debug)]
pub(super) struct CompleteMulti {}

impl Middleware for CompleteMulti {
    fn handle(&self, request: Request) -> Result<Response, BuilderError> {
        //println!("request {:?}", request);
        assert_eq!(request.method(), "POST");
        assert_eq!(
            request.url().as_str(),
            "https://bar.oss-cn-qingdao.aliyuncs.com/aaa.txt?uploadId=foo_upload_id"
        );
        assert_eq!(
            request.headers().get("canonicalizedresource"),
            Some(&HeaderValue::from_str("/bar/aaa.txt?uploadId=foo_upload_id").unwrap())
        );
        let body = request.body().unwrap().clone();
        let xml = "<CompleteMultipartUpload><Part><PartNumber>1</PartNumber><ETag>aaa</ETag></Part><Part><PartNumber>2</PartNumber><ETag>bbb</ETag></Part></CompleteMultipartUpload>";
        let xml = Body::from(xml);
        assert_eq!(body.as_bytes(), xml.as_bytes());
        use http::response::Builder;
        let response = Builder::new().status(200).body("").unwrap();
        Ok(response.into())
    }
}

#[derive(Debug)]
pub(super) struct AbortMulti {}

impl Middleware for AbortMulti {
    fn handle(&self, request: Request) -> Result<Response, BuilderError> {
        //println!("request {:?}", request);
        assert_eq!(request.method(), "DELETE");
        assert_eq!(
            request.url().as_str(),
            "https://bar.oss-cn-qingdao.aliyuncs.com/aaa.txt?uploadId=foo_upload_id"
        );
        assert_eq!(
            request.headers().get("canonicalizedresource"),
            Some(&HeaderValue::from_str("/bar/aaa.txt?uploadId=foo_upload_id").unwrap())
        );
        use http::response::Builder;
        let response = Builder::new().status(200).body("").unwrap();
        Ok(response.into())
    }
}

#[derive(Debug)]
pub(super) struct UploadMulti {}

static mut UPLOAD_MULTI_ORDER: i8 = 1;

impl Middleware for UploadMulti {
    fn handle(&self, request: Request) -> Result<Response, BuilderError> {
        if unsafe { UPLOAD_MULTI_ORDER == 1 } {
            assert_eq!(request.method(), "POST");
            assert_eq!(
                request.url().as_str(),
                "https://bar.oss-cn-qingdao.aliyuncs.com/aaa.txt?uploads"
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/bar/aaa.txt?uploads").unwrap())
            );
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                .body(
                    r#"<InitiateMultipartUploadResult>
        <Bucket>bucket_name</Bucket>
        <Key>aaa</Key>
        <UploadId>foo_upload_id2</UploadId>
      </InitiateMultipartUploadResult>"#,
                )
                .unwrap();
            unsafe {
                UPLOAD_MULTI_ORDER += 1;
            }
            return Ok(response.into());
        } else if unsafe { UPLOAD_MULTI_ORDER == 2 } {
            assert_eq!(request.method(), "PUT");
            assert_eq!(
              request.url().as_str(),
              "https://bar.oss-cn-qingdao.aliyuncs.com/aaa.txt?partNumber=1&uploadId=foo_upload_id2"
          );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(
                    &HeaderValue::from_str("/bar/aaa.txt?partNumber=1&uploadId=foo_upload_id2")
                        .unwrap()
                )
            );
            let body = request.body().unwrap().clone();
            let xml = "aaa";
            let xml = Body::from(xml);
            assert_eq!(body.as_bytes(), xml.as_bytes());
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                .header("ETag", "foo_etag1")
                .body("")
                .unwrap();
            unsafe {
                UPLOAD_MULTI_ORDER += 1;
            }
            return Ok(response.into());
        } else if unsafe { UPLOAD_MULTI_ORDER == 3 } {
            assert_eq!(request.method(), "PUT");
            assert_eq!(
              request.url().as_str(),
              "https://bar.oss-cn-qingdao.aliyuncs.com/aaa.txt?partNumber=2&uploadId=foo_upload_id2"
          );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(
                    &HeaderValue::from_str("/bar/aaa.txt?partNumber=2&uploadId=foo_upload_id2")
                        .unwrap()
                )
            );
            let body = request.body().unwrap().clone();
            let xml = "bbb";
            let xml = Body::from(xml);
            assert_eq!(body.as_bytes(), xml.as_bytes());
            use http::response::Builder;
            let response = Builder::new()
                .status(200)
                .header("ETag", "foo_etag2")
                .body("")
                .unwrap();
            unsafe {
                UPLOAD_MULTI_ORDER += 1;
            }
            return Ok(response.into());
        } else if unsafe { UPLOAD_MULTI_ORDER == 4 } {
            //unsafe {UPLOAD_MULTI_ORDER += 1;}
            assert_eq!(request.method(), "POST");
            assert_eq!(
                request.url().as_str(),
                "https://bar.oss-cn-qingdao.aliyuncs.com/aaa.txt?uploadId=foo_upload_id2"
            );
            assert_eq!(
                request.headers().get("canonicalizedresource"),
                Some(&HeaderValue::from_str("/bar/aaa.txt?uploadId=foo_upload_id2").unwrap())
            );
            let body = request.body().unwrap().clone();
            let xml = "<CompleteMultipartUpload><Part><PartNumber>1</PartNumber><ETag>foo_etag1</ETag></Part><Part><PartNumber>2</PartNumber><ETag>foo_etag2</ETag></Part></CompleteMultipartUpload>";
            let xml = Body::from(xml);
            assert_eq!(body.as_bytes(), xml.as_bytes());
            use http::response::Builder;
            let response = Builder::new().status(200).body("").unwrap();
            return Ok(response.into());
        }

        panic!("error");
    }
}
