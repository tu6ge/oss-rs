use std::{error::Error, fmt::Display};

use async_trait::async_trait;
use http::{
    header::{HeaderName, InvalidHeaderValue, CONTENT_LENGTH, CONTENT_TYPE},
    HeaderValue, Method,
};
use reqwest::{Response, Url};

use crate::{
    bucket::Bucket,
    builder::{ArcPointer, BuilderError, RequestBuilder},
    config::{InvalidObjectPath, ObjectBase, ObjectPath},
    object::{Object, ObjectList},
    types::{CanonicalizedResource, ContentRange},
    Client,
};
#[cfg(feature = "put_file")]
use infer::Infer;

/// # 文件相关功能
///
/// 包括 上传，下载，删除等功能
/// 在 [`Client`]，[`Bucket`], [`ObjectList`] 等结构体中均已实现，其中 Client 是在默认的 bucket 上操作文件，
/// 而 Bucket, ObjectList 则是在当前的 bucket 上操作文件
///
/// [`Client`]: crate::client::Client
/// [`Bucket`]: crate::bucket::Bucket
/// [`ObjectList`]: crate::object::ObjectList
#[async_trait]
pub trait File: AlignBuilder {
    /// 根据文件路径获取最终的调用接口以及相关参数
    fn get_url<OP>(&self, path: OP) -> Result<(Url, CanonicalizedResource), FileError>
    where
        OP: TryInto<ObjectPath> + Send + Sync,
        <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>;

    /// # 上传文件到 OSS
    ///
    /// 需指定文件的路径
    #[cfg(feature = "put_file")]
    async fn put_file<
        P: Into<std::path::PathBuf> + std::convert::AsRef<std::path::Path> + Send + Sync,
        OP,
    >(
        &self,
        file_name: P,
        path: OP,
    ) -> Result<String, FileError>
    where
        OP: TryInto<ObjectPath> + Send + Sync,
        <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        let file_content = std::fs::read(file_name)?;

        let get_content_type =
            |content: &Vec<u8>| Infer::new().get(content).map(|con| con.mime_type());

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
    async fn put_content<F, OP>(
        &self,
        content: Vec<u8>,
        path: OP,
        get_content_type: F,
    ) -> Result<String, FileError>
    where
        F: Fn(&Vec<u8>) -> Option<&'static str> + Send + Sync,
        OP: TryInto<ObjectPath> + Send + Sync,
        <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        let content_type = get_content_type(&content).ok_or_else(|| FileError::FileTypeNotFound)?;

        let content = self.put_content_base(content, content_type, path).await?;

        let result = content
            .headers()
            .get("ETag")
            .ok_or_else(|| FileError::EtagNotFound)?
            .to_str()
            .map_err(FileError::from)?;

        Ok(result.to_string())
    }

    /// 最核心的上传文件到 OSS 的方法
    async fn put_content_base<OP>(
        &self,
        content: Vec<u8>,
        content_type: &str,
        path: OP,
    ) -> Result<Response, FileError>
    where
        OP: TryInto<ObjectPath> + Send + Sync,
        <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        let (url, canonicalized) = self.get_url(path)?;

        let content_length = content.len().to_string();
        let headers = vec![
            (
                CONTENT_LENGTH,
                HeaderValue::from_str(&content_length).map_err(FileError::from)?,
            ),
            (CONTENT_TYPE, content_type.parse().map_err(FileError::from)?),
        ];

        self.builder_with_header(Method::PUT, url, canonicalized, headers)
            .map_err(FileError::from)?
            .body(content)
            .send_adjust_error()
            .await
            .map_err(FileError::from)
    }

    /// # 获取 OSS 上的文件内容
    async fn get_object<R: Into<ContentRange> + Send + Sync, OP>(
        &self,
        path: OP,
        range: R,
    ) -> Result<Vec<u8>, FileError>
    where
        OP: TryInto<ObjectPath> + Send + Sync,
        <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        let (url, canonicalized) = self.get_url(path)?;

        let list: Vec<(_, HeaderValue)> = vec![("Range".parse().unwrap(), range.into().into())];

        let content = self
            .builder_with_header(Method::GET, url, canonicalized, list)?
            .send_adjust_error()
            .await?
            .text()
            .await
            .map_err(FileError::from)?;

        Ok(content.into_bytes())
    }

    /// # 删除 OSS 上的文件
    async fn delete_object<OP>(&self, path: OP) -> Result<(), FileError>
    where
        OP: TryInto<ObjectPath> + Send + Sync,
        <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        let (url, canonicalized) = self.get_url(path)?;

        self.builder(Method::DELETE, url, canonicalized)?
            .send_adjust_error()
            .await?;

        Ok(())
    }
}

impl File for Client {
    fn get_url<OP>(&self, path: OP) -> Result<(Url, CanonicalizedResource), FileError>
    where
        OP: TryInto<ObjectPath> + Send + Sync,
        <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        let object_base = ObjectBase::<ArcPointer>::from_bucket(self.get_bucket_base(), path)
            .map_err(FileError::from)?;

        Ok(object_base.get_url_resource([]))
    }
}

impl File for Bucket {
    fn get_url<OP>(&self, path: OP) -> Result<(Url, CanonicalizedResource), FileError>
    where
        OP: TryInto<ObjectPath> + Send + Sync,
        <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        let object_base = ObjectBase::<ArcPointer>::from_bucket(self.base.to_owned(), path)?;

        Ok(object_base.get_url_resource([]))
    }
}

impl File for ObjectList<ArcPointer> {
    fn get_url<OP>(&self, path: OP) -> Result<(Url, CanonicalizedResource), FileError>
    where
        OP: TryInto<ObjectPath> + Send + Sync,
        <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
    {
        let object_base = ObjectBase::<ArcPointer>::from_bucket(self.bucket.to_owned(), path)?;
        Ok(object_base.get_url_resource([]))
    }
}

#[derive(Debug)]
pub enum FileError {
    Path(InvalidObjectPath),
    Io(std::io::Error),
    ToStr(http::header::ToStrError),
    HeaderValue(InvalidHeaderValue),
    Build(BuilderError),
    FileTypeNotFound,
    EtagNotFound,
}

impl Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(p) => write!(f, "{p}"),
            Self::Io(p) => write!(f, "{p}"),
            Self::ToStr(to) => write!(f, "{to}"),
            Self::HeaderValue(to) => write!(f, "{to}"),
            Self::Build(to) => write!(f, "{to}"),
            Self::FileTypeNotFound => write!(f, "Failed to get file type"),
            Self::EtagNotFound => write!(f, "Failed to get etag"),
        }
    }
}

impl From<InvalidObjectPath> for FileError {
    fn from(value: InvalidObjectPath) -> Self {
        Self::Path(value)
    }
}

impl From<std::io::Error> for FileError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<http::header::ToStrError> for FileError {
    fn from(value: http::header::ToStrError) -> Self {
        Self::ToStr(value)
    }
}

impl From<InvalidHeaderValue> for FileError {
    fn from(value: InvalidHeaderValue) -> Self {
        Self::HeaderValue(value)
    }
}

impl From<BuilderError> for FileError {
    fn from(value: BuilderError) -> Self {
        Self::Build(value)
    }
}

impl From<reqwest::Error> for FileError {
    fn from(value: reqwest::Error) -> Self {
        Self::Build(value.into())
    }
}

impl Error for FileError {}

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
    fn builder(
        &self,
        method: Method,
        url: Url,
        resource: CanonicalizedResource,
    ) -> Result<RequestBuilder, BuilderError> {
        self.builder_with_header(method, url, resource, [])
    }

    fn builder_with_header<H: IntoIterator<Item = (HeaderName, HeaderValue)>>(
        &self,
        method: Method,
        url: Url,
        resource: CanonicalizedResource,
        headers: H,
    ) -> Result<RequestBuilder, BuilderError>;
}

impl AlignBuilder for Bucket {
    #[inline]
    fn builder_with_header<H: IntoIterator<Item = (HeaderName, HeaderValue)>>(
        &self,
        method: Method,
        url: Url,
        resource: CanonicalizedResource,
        headers: H,
    ) -> Result<RequestBuilder, BuilderError> {
        self.client()
            .builder_with_header(method, url, resource, headers)
    }
}

impl AlignBuilder for ObjectList<ArcPointer> {
    #[inline]
    fn builder_with_header<H: IntoIterator<Item = (HeaderName, HeaderValue)>>(
        &self,
        method: Method,
        url: Url,
        resource: CanonicalizedResource,
        headers: H,
    ) -> Result<RequestBuilder, BuilderError> {
        self.client()
            .builder_with_header(method, url, resource, headers)
    }
}

impl Object<ArcPointer> {
    /// # 将当前 Object 的文件上传到 OSS
    ///
    /// 需指定要上传的本地文件路径
    #[cfg(feature = "put_file")]
    pub async fn put_file<
        P: Into<std::path::PathBuf> + std::convert::AsRef<std::path::Path> + Send + Sync,
        B: AlignBuilder,
    >(
        &self,
        file_name: P,
        builder: &B,
    ) -> Result<String, FileError> {
        let file_content = std::fs::read(file_name)?;

        let get_content_type =
            |content: &Vec<u8>| Infer::new().get(content).map(|con| con.mime_type());

        self.put_content(file_content, get_content_type, builder)
            .await
    }

    /// # 上传文件内容到 OSS
    ///
    /// 需指定要上传的文件内容
    /// 以及根据文件内容获取文件类型的闭包
    pub async fn put_content<F, B>(
        &self,
        content: Vec<u8>,
        get_content_type: F,
        builder: &B,
    ) -> Result<String, FileError>
    where
        F: Fn(&Vec<u8>) -> Option<&'static str> + Send + Sync,
        B: AlignBuilder,
    {
        let content_type = get_content_type(&content).ok_or_else(|| FileError::FileTypeNotFound)?;

        let content = self
            .put_content_base(content, content_type, builder)
            .await?;

        let result = content
            .headers()
            .get("ETag")
            .ok_or_else(|| FileError::EtagNotFound)?
            .to_str()
            .map_err(FileError::from)?;

        Ok(result.to_string())
    }

    pub async fn put_content_base<B: AlignBuilder>(
        &self,
        content: Vec<u8>,
        content_type: &str,
        builder: &B,
    ) -> Result<Response, FileError> {
        let (url, canonicalized) = self.base.get_url_resource([]);

        let content_length = content.len().to_string();
        let headers = vec![
            (
                CONTENT_LENGTH,
                HeaderValue::from_str(&content_length).map_err(FileError::from)?,
            ),
            (CONTENT_TYPE, content_type.parse().map_err(FileError::from)?),
        ];

        builder
            .builder_with_header(Method::PUT, url, canonicalized, headers)
            .map_err(FileError::from)?
            .body(content)
            .send_adjust_error()
            .await
            .map_err(FileError::from)
    }

    /// # 获取 Object 对应的 OSS 上的资源文件
    /// 可以获取一部分
    pub async fn get_object<R: Into<ContentRange> + Send + Sync, B: AlignBuilder>(
        &self,
        range: R,
        builder: &B,
    ) -> Result<Vec<u8>, FileError> {
        let (url, canonicalized) = self.base.get_url_resource([]);

        let list: Vec<(_, HeaderValue)> = vec![("Range".parse().unwrap(), range.into().into())];

        let content = builder
            .builder_with_header(Method::GET, url, canonicalized, list)?
            .send_adjust_error()
            .await?
            .text()
            .await
            .map_err(FileError::from)?;

        Ok(content.into_bytes())
    }

    /// 删除 Object 对应的 OSS 上的资源文件
    pub async fn delete_object<B: AlignBuilder>(&self, builder: &B) -> Result<(), FileError> {
        let (url, canonicalized) = self.base.get_url_resource([]);

        builder
            .builder(Method::DELETE, url, canonicalized)?
            .send_adjust_error()
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests_macro {
    use std::sync::Arc;

    use chrono::{DateTime, NaiveDateTime, Utc};

    use crate::{object::Object, Client};

    fn init_object() -> Object {
        let bucket = Arc::new("abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap());
        Object::new(
            bucket,
            "foo2".parse().unwrap(),
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp_opt(123000, 0).unwrap(), Utc),
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
    use super::FileError;
    use crate::{
        blocking::builder::RequestBuilder,
        bucket::Bucket,
        builder::{BuilderError, RcPointer},
        config::{InvalidObjectPath, ObjectBase, ObjectPath},
        object::{Object, ObjectList},
        types::{CanonicalizedResource, ContentRange},
        ClientRc,
    };
    use http::{
        header::{HeaderName, CONTENT_LENGTH, CONTENT_TYPE},
        HeaderValue, Method,
    };
    #[cfg(feature = "put_file")]
    use infer::Infer;
    use reqwest::{blocking::Response, Url};

    pub trait File: AlignBuilder {
        /// 根据文件路径获取最终的调用接口以及相关参数
        fn get_url<OP>(&self, path: OP) -> Result<(Url, CanonicalizedResource), FileError>
        where
            OP: TryInto<ObjectPath>,
            <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>;

        /// # 上传文件到 OSS
        ///
        /// 需指定文件的路径
        #[cfg(feature = "put_file")]
        fn put_file<P: Into<std::path::PathBuf> + std::convert::AsRef<std::path::Path>, OP>(
            &self,
            file_name: P,
            path: OP,
        ) -> Result<String, FileError>
        where
            OP: TryInto<ObjectPath>,
            <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
        {
            let file_content = std::fs::read(file_name)?;

            let get_content_type =
                |content: &Vec<u8>| Infer::new().get(content).map(|con| con.mime_type());

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
        fn put_content<F, OP>(
            &self,
            content: Vec<u8>,
            path: OP,
            get_content_type: F,
        ) -> Result<String, FileError>
        where
            F: Fn(&Vec<u8>) -> Option<&'static str>,
            OP: TryInto<ObjectPath>,
            <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
        {
            let content_type =
                get_content_type(&content).ok_or_else(|| FileError::FileTypeNotFound)?;

            let content = self.put_content_base(content, content_type, path)?;

            let result = content
                .headers()
                .get("ETag")
                .ok_or_else(|| FileError::EtagNotFound)?
                .to_str()
                .map_err(FileError::from)?;

            Ok(result.to_string())
        }

        /// 最原始的上传文件的方法
        fn put_content_base<OP>(
            &self,
            content: Vec<u8>,
            content_type: &str,
            path: OP,
        ) -> Result<Response, FileError>
        where
            OP: TryInto<ObjectPath>,
            <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
        {
            let (url, canonicalized) = self.get_url(path)?;

            let content_length = content.len().to_string();
            let headers = vec![
                (
                    CONTENT_LENGTH,
                    HeaderValue::from_str(&content_length).map_err(FileError::from)?,
                ),
                (CONTENT_TYPE, content_type.parse().map_err(FileError::from)?),
            ];

            let response = self
                .builder_with_header(Method::PUT, url, canonicalized, headers)?
                .body(content);

            let content = response.send_adjust_error()?;
            Ok(content)
        }

        /// # 获取文件内容
        fn get_object<R: Into<ContentRange>, OP>(
            &self,
            path: OP,
            range: R,
        ) -> Result<Vec<u8>, FileError>
        where
            OP: TryInto<ObjectPath>,
            <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
        {
            let (url, canonicalized) = self.get_url(path)?;

            let headers: Vec<(_, HeaderValue)> =
                vec![("Range".parse().unwrap(), range.into().into())];

            Ok(self
                .builder_with_header(Method::GET, url, canonicalized, headers)?
                .send_adjust_error()?
                .text()?
                .into_bytes())
        }

        fn delete_object<OP>(&self, path: OP) -> Result<(), FileError>
        where
            OP: TryInto<ObjectPath>,
            <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
        {
            let (url, canonicalized) = self.get_url(path)?;

            self.builder(Method::DELETE, url, canonicalized)?
                .send_adjust_error()?;

            Ok(())
        }
    }

    impl File for ClientRc {
        fn get_url<OP>(&self, path: OP) -> Result<(Url, CanonicalizedResource), FileError>
        where
            OP: TryInto<ObjectPath>,
            <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
        {
            let object_base = ObjectBase::<RcPointer>::from_bucket(self.get_bucket_base(), path)
                .map_err(FileError::from)?;

            Ok(object_base.get_url_resource([]))
        }
    }

    impl File for Bucket<RcPointer> {
        fn get_url<OP>(&self, path: OP) -> Result<(Url, CanonicalizedResource), FileError>
        where
            OP: TryInto<ObjectPath>,
            <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
        {
            let object_base = ObjectBase::<RcPointer>::from_bucket(self.base.clone(), path)
                .map_err(FileError::from)?;

            Ok(object_base.get_url_resource([]))
        }
    }

    impl File for ObjectList<RcPointer> {
        fn get_url<OP>(&self, path: OP) -> Result<(Url, CanonicalizedResource), FileError>
        where
            OP: TryInto<ObjectPath>,
            <OP as TryInto<ObjectPath>>::Error: Into<InvalidObjectPath>,
        {
            let object_base = ObjectBase::<RcPointer>::from_bucket(self.bucket.clone(), path)
                .map_err(FileError::from)?;

            Ok(object_base.get_url_resource([]))
        }
    }

    pub trait AlignBuilder {
        #[inline]
        fn builder(
            &self,
            method: Method,
            url: Url,
            resource: CanonicalizedResource,
        ) -> Result<RequestBuilder, BuilderError> {
            self.builder_with_header(method, url, resource, [])
        }

        fn builder_with_header<H: IntoIterator<Item = (HeaderName, HeaderValue)>>(
            &self,
            method: Method,
            url: Url,
            resource: CanonicalizedResource,
            headers: H,
        ) -> Result<RequestBuilder, BuilderError>;
    }

    /// # 对齐 Client, Bucket, ObjectList 等结构体的 trait
    ///
    /// 用于他们方便的实现 [`File`](./trait.File.html) trait
    impl AlignBuilder for Bucket<RcPointer> {
        fn builder_with_header<H: IntoIterator<Item = (HeaderName, HeaderValue)>>(
            &self,
            method: Method,
            url: Url,
            resource: CanonicalizedResource,
            headers: H,
        ) -> Result<RequestBuilder, BuilderError> {
            self.client()
                .builder_with_header(method, url, resource, headers)
        }
    }

    impl AlignBuilder for ObjectList<RcPointer> {
        fn builder_with_header<H: IntoIterator<Item = (HeaderName, HeaderValue)>>(
            &self,
            method: Method,
            url: Url,
            resource: CanonicalizedResource,
            headers: H,
        ) -> Result<RequestBuilder, BuilderError> {
            self.client()
                .builder_with_header(method, url, resource, headers)
        }
    }

    impl Object<RcPointer> {
        /// # 将当前 Object 的文件上传到 OSS
        ///
        /// 需指定要上传的本地文件路径
        #[cfg(feature = "put_file")]
        pub fn put_file<
            P: Into<std::path::PathBuf> + std::convert::AsRef<std::path::Path>,
            B: AlignBuilder,
        >(
            &self,
            file_name: P,
            builder: &B,
        ) -> Result<String, FileError> {
            let file_content = std::fs::read(file_name)?;

            let get_content_type =
                |content: &Vec<u8>| Infer::new().get(content).map(|con| con.mime_type());

            self.put_content(file_content, get_content_type, builder)
        }

        /// # 上传文件内容到 OSS
        ///
        /// 需指定要上传的文件内容
        /// 以及根据文件内容获取文件类型的闭包
        pub fn put_content<F, B>(
            &self,
            content: Vec<u8>,
            get_content_type: F,
            builder: &B,
        ) -> Result<String, FileError>
        where
            F: Fn(&Vec<u8>) -> Option<&'static str>,
            B: AlignBuilder,
        {
            let content_type =
                get_content_type(&content).ok_or_else(|| FileError::FileTypeNotFound)?;

            let content = self.put_content_base(content, content_type, builder)?;

            let result = content
                .headers()
                .get("ETag")
                .ok_or_else(|| FileError::EtagNotFound)?
                .to_str()
                .map_err(FileError::from)?;

            Ok(result.to_string())
        }

        /// 最原始的上传文件的方法
        pub fn put_content_base<B: AlignBuilder>(
            &self,
            content: Vec<u8>,
            content_type: &str,
            builder: &B,
        ) -> Result<Response, FileError> {
            let (url, canonicalized) = self.base.get_url_resource([]);

            let content_length = content.len().to_string();
            let headers = vec![
                (
                    CONTENT_LENGTH,
                    HeaderValue::from_str(&content_length).map_err(FileError::from)?,
                ),
                (CONTENT_TYPE, content_type.parse().map_err(FileError::from)?),
            ];

            builder
                .builder_with_header(Method::PUT, url, canonicalized, headers)
                .map_err(FileError::from)?
                .body(content)
                .send_adjust_error()
                .map_err(FileError::from)
        }

        /// # 获取 Object 对应的 OSS 上的资源文件
        /// 可以获取一部分
        pub fn get_object<R: Into<ContentRange>, B: AlignBuilder>(
            &self,
            range: R,
            builder: &B,
        ) -> Result<Vec<u8>, FileError> {
            let (url, canonicalized) = self.base.get_url_resource([]);

            let list: Vec<(_, HeaderValue)> = vec![("Range".parse().unwrap(), range.into().into())];

            let content = builder
                .builder_with_header(Method::GET, url, canonicalized, list)?
                .send_adjust_error()?
                .text()
                .map_err(FileError::from)?;

            Ok(content.into_bytes())
        }

        /// 删除 Object 对应的 OSS 上的资源文件
        pub fn delete_object<B: AlignBuilder>(&self, builder: &B) -> Result<(), FileError> {
            let (url, canonicalized) = self.base.get_url_resource([]);

            builder
                .builder(Method::DELETE, url, canonicalized)?
                .send_adjust_error()?;

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests_macro {
        use chrono::{DateTime, NaiveDateTime, Utc};

        use crate::{builder::RcPointer, object::Object, ClientRc};
        use std::rc::Rc;

        fn init_object() -> Object<RcPointer> {
            let bucket = Rc::new("abc.oss-cn-shanghai.aliyuncs.com".parse().unwrap());
            Object::<RcPointer>::new(
                bucket,
                "foo2".parse().unwrap(),
                DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(123000, 0).unwrap(),
                    Utc,
                ),
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
