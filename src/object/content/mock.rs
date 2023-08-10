use async_trait::async_trait;
use http::HeaderValue;
use reqwest::Response;

use crate::{file::FileError, types::ContentRange, Client, ObjectPath};

#[async_trait]
pub(super) trait Files {
    async fn put_content_base(
        &self,
        content: Vec<u8>,
        content_type: &str,
        path: ObjectPath,
    ) -> Result<Response, FileError>;

    async fn get_object<Num, R>(&self, path: ObjectPath, range: R) -> Result<Vec<u8>, FileError>
    where
        R: Into<ContentRange<Num>> + Send + Sync,
        ContentRange<Num>: Into<HeaderValue>;
}

static mut READ_FILE_NUM: u8 = 1;

#[async_trait]
impl Files for Client {
    async fn put_content_base(
        &self,
        content: Vec<u8>,
        content_type: &str,
        path: ObjectPath,
    ) -> Result<Response, FileError> {
        use http::response::Builder;
        assert_eq!(content, b"bbb".to_vec());
        assert_eq!(content_type, "text/plain");
        assert_eq!(path.as_ref(), "aaa.txt");

        let resp = Builder::new().status(200).body("").unwrap();

        Ok(resp.into())
    }
    async fn get_object<Num, R>(&self, path: ObjectPath, range: R) -> Result<Vec<u8>, FileError>
    where
        R: Into<ContentRange<Num>> + Send + Sync,
        ContentRange<Num>: Into<HeaderValue>,
    {
        if unsafe { READ_FILE_NUM == 1 } {
            unsafe {
                READ_FILE_NUM += 1;
            }
            assert_eq!(path.as_ref(), "aaa.txt");
            let range: HeaderValue = range.into().into();
            assert_eq!(range.to_str().unwrap(), "bytes=0-9");

            let vec = vec![1u8, 2, 3, 4, 5];

            return Ok(vec);
        } else if unsafe { READ_FILE_NUM == 2 } {
            unsafe {
                READ_FILE_NUM += 1;
            }
            assert_eq!(path.as_ref(), "aaa.txt");
            let range: HeaderValue = range.into().into();
            assert_eq!(range.to_str().unwrap(), "bytes=0-2");

            let vec = vec![1u8, 2, 3, 4, 5];

            return Ok(vec);
        } else if unsafe { READ_FILE_NUM == 3 } {
            unsafe {
                READ_FILE_NUM += 1;
            }
            assert_eq!(path.as_ref(), "aaa.txt");
            let range: HeaderValue = range.into().into();
            assert_eq!(range.to_str().unwrap(), "bytes=10-12");

            let vec = vec![1u8, 2, 3, 4, 5];

            return Ok(vec);
        }

        panic!("error");
    }
}

#[cfg(feature = "blocking")]
pub(super) mod blocking {
    use http::HeaderValue;
    use reqwest::Response;

    use crate::{client::ClientRc as Client, file::FileError, types::ContentRange, ObjectPath};

    pub trait Files {
        fn put_content_base(
            &self,
            content: Vec<u8>,
            content_type: &str,
            path: ObjectPath,
        ) -> Result<Response, FileError>;
        /// # 获取文件内容
        fn get_object<Num, R>(&self, path: ObjectPath, range: R) -> Result<Vec<u8>, FileError>
        where
            R: Into<ContentRange<Num>>,
            ContentRange<Num>: Into<HeaderValue>;
    }

    static mut READ_FILE_NUM: u8 = 1;

    impl Files for Client {
        fn put_content_base(
            &self,
            content: Vec<u8>,
            content_type: &str,
            path: ObjectPath,
        ) -> Result<Response, FileError> {
            use http::response::Builder;
            assert_eq!(content, b"bbb".to_vec());
            assert_eq!(content_type, "text/plain");
            assert_eq!(path.as_ref(), "aaa.txt");

            let resp = Builder::new().status(200).body("").unwrap();

            Ok(resp.into())
        }
        fn get_object<Num, R>(&self, path: ObjectPath, range: R) -> Result<Vec<u8>, FileError>
        where
            R: Into<ContentRange<Num>>,
            ContentRange<Num>: Into<HeaderValue>,
        {
            if unsafe { READ_FILE_NUM == 1 } {
                unsafe {
                    READ_FILE_NUM += 1;
                }
                assert_eq!(path.as_ref(), "aaa.txt");
                let range: HeaderValue = range.into().into();
                assert_eq!(range.to_str().unwrap(), "bytes=0-9");

                let vec = vec![1u8, 2, 3, 4, 5];

                return Ok(vec);
            } else if unsafe { READ_FILE_NUM == 2 } {
                unsafe {
                    READ_FILE_NUM += 1;
                }
                assert_eq!(path.as_ref(), "aaa.txt");
                let range: HeaderValue = range.into().into();
                assert_eq!(range.to_str().unwrap(), "bytes=0-2");

                let vec = vec![1u8, 2, 3, 4, 5];

                return Ok(vec);
            } else if unsafe { READ_FILE_NUM == 3 } {
                unsafe {
                    READ_FILE_NUM += 1;
                }
                assert_eq!(path.as_ref(), "aaa.txt");
                let range: HeaderValue = range.into().into();
                assert_eq!(range.to_str().unwrap(), "bytes=10-12");

                let vec = vec![1u8, 2, 3, 4, 5];

                return Ok(vec);
            }

            panic!("error");
        }
    }
}
