use async_trait::async_trait;
use http::{
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    HeaderMap, HeaderValue,
};
use reqwest::{Response, Url};

use crate::{
    auth::VERB,
    bucket::Bucket,
    builder::{ArcPointer, BuilderError, RequestBuilder},
    config::{ObjectBase, ObjectPath},
    errors::{OssError, OssResult},
    object::{Object, ObjectList},
    types::{CanonicalizedResource, ContentRange},
    Client,
};
#[cfg(feature = "put_file")]
use infer::Infer;

use oss_derive::oss_file;

/// # 文件相关功能
///
/// 包括 上传，下载，删除等功能
/// 在 [`Client`]，[`Bucket`], [`ObjectList`] 等结构体中均已实现，其中 Client 是在默认的 bucket 上操作文件，
/// 而 Bucket, ObjectList 则是在当前的 bucket 上操作文件
///
/// [`Client`]: crate::client::Client
/// [`Bucket`]: crate::bucket::Bucket
/// [`ObjectList`]: crate::object::ObjectList
#[oss_file(ASYNC)]
#[async_trait]
pub trait File: AlignBuilder {
    /// 根据文件路径获取最终的调用接口以及相关参数
    fn get_url<OP: Into<ObjectPath> + Send + Sync>(&self, path: OP)
        -> (Url, CanonicalizedResource);

    /// # 上传文件到 OSS
    ///
    /// 需指定文件的路径
    #[cfg(feature = "put_file")]
    async fn put_file<
        P: Into<std::path::PathBuf> + std::convert::AsRef<std::path::Path> + Send + Sync,
        OP: Into<ObjectPath> + Send + Sync,
    >(
        &self,
        file_name: P,
        path: OP,
    ) -> OssResult<String> {
        let file_content = std::fs::read(file_name)?;

        let get_content_type = |content: &Vec<u8>| match Infer::new().get(content) {
            Some(con) => Some(con.mime_type()),
            None => None,
        };

        self.put_content(file_content, path, get_content_type).await
    }

    /// # 上传文件内容到 OSS
    ///
    /// 需指定要上传的文件内容
    /// 以及根据文件内容获取文件类型的闭包
    ///
    /// # Examples
    ///
    /// 上传 tauri 升级用的签名文件
    /// ```no_run
    /// # #[tokio::main]
    /// # async fn main(){
    /// use infer::Infer;
    /// # use dotenv::dotenv;
    /// # dotenv().ok();
    /// # let client = aliyun_oss_client::Client::from_env().unwrap();
    /// use aliyun_oss_client::file::File;
    ///
    /// fn sig_match(buf: &[u8]) -> bool {
    ///     return buf.len() >= 3 && buf[0] == 0x64 && buf[1] == 0x57 && buf[2] == 0x35;
    /// }
    /// let mut infer = Infer::new();
    /// infer.add("application/pgp-signature", "sig", sig_match);
    ///
    /// let get_content_type = |content: &Vec<u8>| match infer.get(content) {
    ///     Some(con) => Some(con.mime_type()),
    ///     None => None,
    /// };
    /// let content: Vec<u8> = String::from("dW50cnVzdGVkIGNvbW1lbnQ6IHNpxxxxxxxxx").into_bytes();
    /// let res = client
    ///     .put_content(content, "xxxxxx.msi.zip.sig", get_content_type)
    ///     .await;
    /// assert!(res.is_ok());
    /// # }
    /// ```
    async fn put_content<F, OP: Into<ObjectPath> + Send + Sync>(
        &self,
        content: Vec<u8>,
        path: OP,
        get_content_type: F,
    ) -> OssResult<String>
    where
        F: Fn(&Vec<u8>) -> Option<&'static str> + Send + Sync,
    {
        let content_type = get_content_type(&content)
            .ok_or(OssError::Input("Failed to get file type".to_string()))?;

        let content = self.put_content_base(content, content_type, path).await?;

        let result = content
            .headers()
            .get("ETag")
            .ok_or(OssError::Input("get Etag error".to_string()))?
            .to_str()
            .map_err(OssError::from)?;

        Ok(result.to_string())
    }

    /// 最核心的上传文件到 OSS 的方法
    async fn put_content_base<OP: Into<ObjectPath> + Send + Sync>(
        &self,
        content: Vec<u8>,
        content_type: &str,
        path: OP,
    ) -> OssResult<Response> {
        let (url, canonicalized) = self.get_url(path);

        let mut headers = HeaderMap::with_capacity(2);
        let content_length = content.len().to_string();
        headers.insert(
            CONTENT_LENGTH,
            HeaderValue::from_str(&content_length).map_err(OssError::from)?,
        );

        headers.insert(CONTENT_TYPE, content_type.parse().map_err(OssError::from)?);

        self.builder_with_header(VERB::PUT, url, canonicalized, headers)?
            .body(content)
            .send_adjust_error()
            .await
            .map_err(OssError::from)
    }

    /// # 获取 OSS 上的文件内容
    async fn get_object<R: Into<ContentRange> + Send + Sync, OP: Into<ObjectPath> + Send + Sync>(
        &self,
        path: OP,
        range: R,
    ) -> OssResult<Vec<u8>> {
        let (url, canonicalized) = self.get_url(path);

        let headers = {
            let mut headers = HeaderMap::with_capacity(1);
            headers.insert("Range", range.into().into());
            headers
        };

        let content = self
            .builder_with_header("GET", url, canonicalized, headers)?
            .send_adjust_error()
            .await?
            .text()
            .await?;

        Ok(content.into_bytes())
    }

    /// # 删除 OSS 上的文件
    async fn delete_object<OP: Into<ObjectPath> + Send + Sync>(&self, path: OP) -> OssResult<()> {
        let (url, canonicalized) = self.get_url(path);

        self.builder(VERB::DELETE, url, canonicalized)?
            .send_adjust_error()
            .await?;

        Ok(())
    }
}

impl File for Client {
    fn get_url<OP: Into<ObjectPath> + Send + Sync>(
        &self,
        path: OP,
    ) -> (Url, CanonicalizedResource) {
        let object_base = ObjectBase::<ArcPointer>::from_bucket(self.get_bucket_base(), path);

        object_base.get_url_resource([])
    }
}

impl File for Bucket {
    fn get_url<OP: Into<ObjectPath> + Send + Sync>(
        &self,
        path: OP,
    ) -> (Url, CanonicalizedResource) {
        let object_base = ObjectBase::<ArcPointer>::from_bucket(self.base.to_owned(), path);

        object_base.get_url_resource([])
    }
}

impl File for ObjectList<ArcPointer> {
    fn get_url<OP: Into<ObjectPath> + Send + Sync>(
        &self,
        path: OP,
    ) -> (Url, CanonicalizedResource) {
        let object_base = ObjectBase::<ArcPointer>::from_bucket(self.bucket.to_owned(), path);
        object_base.get_url_resource([])
    }
}

/// # 对齐 [`Client`]，[`Bucket`], [`ObjectList`] 等结构体的 trait
///
/// 用于他们方便的实现 [`File`] trait
///
/// [`File`]: self::File
/// [`Client`]: crate::client::Client
/// [`Bucket`]: crate::bucket::Bucket
/// [`ObjectList`]: crate::object::ObjectList
pub trait AlignBuilder: Send + Sync {
    #[inline]
    fn builder<M: Into<VERB>>(
        &self,
        method: M,
        url: Url,
        resource: CanonicalizedResource,
    ) -> Result<RequestBuilder, BuilderError> {
        self.builder_with_header(method, url, resource, HeaderMap::with_capacity(0))
    }

    fn builder_with_header<M: Into<VERB>>(
        &self,
        method: M,
        url: Url,
        resource: CanonicalizedResource,
        headers: HeaderMap,
    ) -> Result<RequestBuilder, BuilderError>;
}

impl AlignBuilder for Bucket {
    #[inline]
    fn builder_with_header<M: Into<VERB>>(
        &self,
        method: M,
        url: Url,
        resource: CanonicalizedResource,
        headers: HeaderMap,
    ) -> Result<RequestBuilder, BuilderError> {
        self.client()
            .builder_with_header(method, url, resource, headers)
    }
}

impl AlignBuilder for ObjectList<ArcPointer> {
    #[inline]
    fn builder_with_header<M: Into<VERB>>(
        &self,
        method: M,
        url: Url,
        resource: CanonicalizedResource,
        headers: HeaderMap,
    ) -> Result<RequestBuilder, BuilderError> {
        self.client()
            .builder_with_header(method, url, resource, headers)
    }
}

#[cfg(test)]
mod tests_macro {
    use std::sync::Arc;

    use chrono::{DateTime, NaiveDateTime, Utc};

    use crate::{config::BucketBase, object::Object, Client};

    fn init_object() -> Object {
        let bucket = Arc::new(BucketBase::from_str("abc.oss-cn-shanghai.aliyuncs.com").unwrap());
        Object::new(
            bucket,
            "foo2",
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(123000, 0), Utc),
            "foo3".into(),
            "foo4".into(),
            100,
            "foo5".into(),
        )
    }

    fn init_client() -> Client {
        use std::env::set_var;
        set_var("ALIYUN_KEY_ID", "foo1");
        set_var("ALIYUN_KEY_SECRET", "foo2");
        set_var("ALIYUN_ENDPOINT", "qingdao");
        set_var("ALIYUN_BUCKET", "foo4");
        Client::from_env().unwrap()
    }

    #[cfg(feature = "put_file")]
    #[tokio::test]
    async fn test_object_put_file() {
        let object = init_object();

        let client = init_client();

        let res = object.put_file("abc", &client).await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_object_put_content() {
        let object = init_object();

        let client = init_client();

        let content: Vec<u8> = String::from("dW50cnVzdGVkIGNvbW1lbnQ6IHNpxxxxxxxxx").into_bytes();

        let res = object
            .put_content(content, |_| Some("image/png"), &client)
            .await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_object_put_content_base() {
        let object = init_object();

        let client = init_client();

        let content: Vec<u8> = String::from("dW50cnVzdGVkIGNvbW1lbnQ6IHNpxxxxxxxxx").into_bytes();

        let res = object.put_content_base(content, "image/png", &client).await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_object_get_object() {
        let object = init_object();

        let client = init_client();

        let res = object.get_object(.., &client).await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_object_delete() {
        let object = init_object();

        let client = init_client();

        let res = object.delete_object(&client).await;

        assert!(res.is_err());
    }
}

#[cfg(feature = "blocking")]
pub use blocking::File as BlockingFile;

#[cfg(feature = "blocking")]
pub mod blocking {
    use crate::{
        auth::VERB,
        blocking::builder::RequestBuilder,
        bucket::Bucket,
        builder::{BuilderError, RcPointer},
        config::{ObjectBase, ObjectPath},
        errors::{OssError, OssResult},
        object::{Object, ObjectList},
        types::{CanonicalizedResource, ContentRange},
        ClientRc,
    };
    use http::{
        header::{CONTENT_LENGTH, CONTENT_TYPE},
        HeaderMap, HeaderValue,
    };
    #[cfg(feature = "put_file")]
    use infer::Infer;
    use oss_derive::oss_file;
    use reqwest::{blocking::Response, Url};

    #[oss_file]
    pub trait File: AlignBuilder {
        /// 根据文件路径获取最终的调用接口以及相关参数
        fn get_url<OP: Into<ObjectPath>>(&self, path: OP) -> (Url, CanonicalizedResource);

        /// # 上传文件到 OSS
        ///
        /// 需指定文件的路径
        #[cfg(feature = "put_file")]
        fn put_file<
            P: Into<std::path::PathBuf> + std::convert::AsRef<std::path::Path>,
            OP: Into<ObjectPath>,
        >(
            &self,
            file_name: P,
            path: OP,
        ) -> OssResult<String> {
            let file_content = std::fs::read(file_name)?;

            let get_content_type = |content: &Vec<u8>| match Infer::new().get(content) {
                Some(con) => Some(con.mime_type()),
                None => None,
            };

            self.put_content(file_content, path, get_content_type)
        }

        /// # 上传文件内容到 OSS
        ///
        /// 需指定要上传的文件内容
        /// 以及根据文件内容获取文件类型的闭包
        ///
        /// # Examples
        ///
        /// 上传 tauri 升级用的签名文件
        /// ```no_run
        /// # fn main(){
        /// use infer::Infer;
        /// # use dotenv::dotenv;
        /// # dotenv().ok();
        /// # let client = aliyun_oss_client::ClientRc::from_env().unwrap();
        /// use crate::aliyun_oss_client::file::BlockingFile;
        ///
        /// fn sig_match(buf: &[u8]) -> bool {
        ///     return buf.len() >= 3 && buf[0] == 0x64 && buf[1] == 0x57 && buf[2] == 0x35;
        /// }
        /// let mut infer = Infer::new();
        /// infer.add("application/pgp-signature", "sig", sig_match);
        ///
        /// let get_content_type = |content: &Vec<u8>| match infer.get(content) {
        ///     Some(con) => Some(con.mime_type()),
        ///     None => None,
        /// };
        /// let content: Vec<u8> = String::from("dW50cnVzdGVkIGNvbW1lbnQ6IHNpxxxxxxxxx").into_bytes();
        /// let res = client.put_content(content, "xxxxxx.msi.zip.sig", get_content_type);
        /// assert!(res.is_ok());
        /// # }
        /// ```
        fn put_content<F, OP: Into<ObjectPath>>(
            &self,
            content: Vec<u8>,
            path: OP,
            get_content_type: F,
        ) -> OssResult<String>
        where
            F: Fn(&Vec<u8>) -> Option<&'static str>,
        {
            let content_type = get_content_type(&content)
                .ok_or(OssError::Input("Failed to get file type".to_string()))?;

            let content = self.put_content_base(content, content_type, path)?;

            let result = content
                .headers()
                .get("ETag")
                .ok_or(OssError::Input("get Etag error".to_string()))?
                .to_str()
                .map_err(OssError::from)?;

            Ok(result.to_string())
        }

        /// 最原始的上传文件的方法
        fn put_content_base<OP: Into<ObjectPath>>(
            &self,
            content: Vec<u8>,
            content_type: &str,
            path: OP,
        ) -> OssResult<Response> {
            let (url, canonicalized) = self.get_url(path);

            let mut headers = HeaderMap::with_capacity(2);
            let content_length = content.len().to_string();
            headers.insert(
                CONTENT_LENGTH,
                HeaderValue::from_str(&content_length).map_err(OssError::from)?,
            );

            headers.insert(CONTENT_TYPE, content_type.parse().map_err(OssError::from)?);

            let response = self
                .builder_with_header(VERB::PUT, url, canonicalized, headers)?
                .body(content);

            let content = response.send_adjust_error()?;
            Ok(content)
        }

        /// # 获取文件内容
        fn get_object<R: Into<ContentRange>, OP: Into<ObjectPath>>(
            &self,
            path: OP,
            range: R,
        ) -> OssResult<Vec<u8>> {
            let (url, canonicalized) = self.get_url(path);

            let headers = {
                let mut headers = HeaderMap::with_capacity(1);
                headers.insert("Range", range.into().into());
                headers
            };

            Ok(self
                .builder_with_header("GET", url, canonicalized, headers)?
                .send_adjust_error()?
                .text()?
                .into_bytes())
        }

        fn delete_object<OP: Into<ObjectPath>>(&self, path: OP) -> OssResult<()> {
            let (url, canonicalized) = self.get_url(path);

            self.builder(VERB::DELETE, url, canonicalized)?
                .send_adjust_error()?;

            Ok(())
        }
    }

    impl File for ClientRc {
        fn get_url<OP: Into<ObjectPath>>(&self, path: OP) -> (Url, CanonicalizedResource) {
            let object_base = ObjectBase::<RcPointer>::from_bucket(self.get_bucket_base(), path);

            object_base.get_url_resource([])
        }
    }

    impl File for Bucket<RcPointer> {
        fn get_url<OP: Into<ObjectPath>>(&self, path: OP) -> (Url, CanonicalizedResource) {
            let object_base = ObjectBase::<RcPointer>::from_bucket(self.base.clone(), path);

            object_base.get_url_resource([])
        }
    }

    impl File for ObjectList<RcPointer> {
        fn get_url<OP: Into<ObjectPath>>(&self, path: OP) -> (Url, CanonicalizedResource) {
            let object_base = ObjectBase::<RcPointer>::from_bucket(self.bucket.clone(), path);

            object_base.get_url_resource([])
        }
    }

    pub trait AlignBuilder {
        #[inline]
        fn builder<M: Into<VERB>>(
            &self,
            method: M,
            url: Url,
            resource: CanonicalizedResource,
        ) -> Result<RequestBuilder, BuilderError> {
            self.builder_with_header(method, url, resource, HeaderMap::with_capacity(0))
        }

        fn builder_with_header<M: Into<VERB>>(
            &self,
            method: M,
            url: Url,
            resource: CanonicalizedResource,
            headers: HeaderMap,
        ) -> Result<RequestBuilder, BuilderError>;
    }

    /// # 对齐 Client, Bucket, ObjectList 等结构体的 trait
    ///
    /// 用于他们方便的实现 [`File`](./trait.File.html) trait
    impl AlignBuilder for Bucket<RcPointer> {
        fn builder_with_header<M: Into<VERB>>(
            &self,
            method: M,
            url: Url,
            resource: CanonicalizedResource,
            headers: HeaderMap,
        ) -> Result<RequestBuilder, BuilderError> {
            self.client()
                .builder_with_header(method, url, resource, headers)
        }
    }

    impl AlignBuilder for ObjectList<RcPointer> {
        fn builder_with_header<M: Into<VERB>>(
            &self,
            method: M,
            url: Url,
            resource: CanonicalizedResource,
            headers: HeaderMap,
        ) -> Result<RequestBuilder, BuilderError> {
            self.client()
                .builder_with_header(method, url, resource, headers)
        }
    }

    #[cfg(test)]
    mod tests_macro {
        use chrono::{DateTime, NaiveDateTime, Utc};

        use crate::{builder::RcPointer, config::BucketBase, object::Object, ClientRc};
        use std::rc::Rc;

        fn init_object() -> Object<RcPointer> {
            let bucket = Rc::new(BucketBase::from_str("abc.oss-cn-shanghai.aliyuncs.com").unwrap());
            Object::<RcPointer>::new(
                bucket,
                "foo2",
                DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(123000, 0), Utc),
                "foo3".into(),
                "foo4".into(),
                100,
                "foo5".into(),
            )
        }

        fn init_client() -> ClientRc {
            use std::env::set_var;
            set_var("ALIYUN_KEY_ID", "foo1");
            set_var("ALIYUN_KEY_SECRET", "foo2");
            set_var("ALIYUN_ENDPOINT", "qingdao");
            set_var("ALIYUN_BUCKET", "foo4");
            ClientRc::from_env().unwrap()
        }

        #[test]
        #[cfg(feature = "put_file")]
        fn test_object_put_file() {
            let object = init_object();

            let client = init_client();

            let res = object.put_file("abc", &client);

            assert!(res.is_err());
        }

        #[test]
        fn test_object_put_content() {
            let object = init_object();

            let client = init_client();

            let content: Vec<u8> =
                String::from("dW50cnVzdGVkIGNvbW1lbnQ6IHNpxxxxxxxxx").into_bytes();

            let res = object.put_content(content, |_| Some("image/png"), &client);

            assert!(res.is_err());
        }

        #[test]
        fn test_object_put_content_base() {
            let object = init_object();

            let client = init_client();

            let content: Vec<u8> =
                String::from("dW50cnVzdGVkIGNvbW1lbnQ6IHNpxxxxxxxxx").into_bytes();

            let res = object.put_content_base(content, "image/png", &client);

            assert!(res.is_err());
        }

        #[test]
        fn test_object_get_object() {
            let object = init_object();

            let client = init_client();

            let res = object.get_object(.., &client);

            assert!(res.is_err());
        }

        #[test]
        fn test_object_delete() {
            let object = init_object();

            let client = init_client();

            let res = object.delete_object(&client);

            assert!(res.is_err());
        }
    }
}
