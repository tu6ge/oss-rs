//! # OSS 文件相关操作
//!
//! [`File`] 是一个文件操作的工具包，包含上传，下载，删除功能，开发者可以方便的调用使用
//!
//! ```rust
//! use std::{fs, path::Path};
//!
//! use aliyun_oss_client::{
//!     config::ObjectPath,
//!     file::{File, FileError, Files},
//!     BucketName, Client, EndPoint, KeyId, KeySecret,
//! };
//!
//! struct MyObject {
//!     path: ObjectPath,
//! }
//!
//! impl MyObject {
//!     const KEY_ID: KeyId = KeyId::from_static("xxxxxxxxxxxxxxxx");
//!     const KEY_SECRET: KeySecret = KeySecret::from_static("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
//!     const END_POINT: EndPoint = EndPoint::CnShanghai;
//!     const BUCKET: BucketName = unsafe { BucketName::from_static2("xxxxxx") };
//!
//!     fn new(path: &Path) -> Result<MyObject, FileError> {
//!         Ok(MyObject {
//!             path: path.try_into()?,
//!         })
//!     }
//! }
//!
//! impl File for MyObject {
//!     type Client = Client;
//!     fn get_path(&self) -> ObjectPath {
//!         self.path.clone()
//!     }
//!
//!     fn oss_client(&self) -> Self::Client {
//!         Client::new(
//!             Self::KEY_ID,
//!             Self::KEY_SECRET,
//!             Self::END_POINT,
//!             Self::BUCKET,
//!         )
//!     }
//! }
//!
//! async fn run() -> Result<(), FileError> {
//!     for entry in fs::read_dir("examples")? {
//!         let path = entry?.path();
//!         let path = path.as_path();
//!
//!         if !path.is_file() {
//!             continue;
//!         }
//!
//!         let obj = MyObject::new(path)?;
//!         let content = fs::read(path)?;
//!
//!         let res = obj.put_oss(content, Client::DEFAULT_CONTENT_TYPE).await?;
//!
//!         println!("result status: {}", res.status());
//!     }
//!
//!     Ok(())
//! }
//! ```
//! [`File`]: crate::file::File

use std::{error::Error, fmt::Display, sync::Arc};

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

/// # 文件的相关操作
///
/// 包括 上传，下载，删除等功能
#[async_trait]
pub trait File
where
    Self::Client: Files,
{
    /// 用于发起 OSS 接口调用的客户端，如[`Client`]，[`Bucket`], [`ObjectList`] 等结构体
    ///
    /// [`Client`]: crate::client::Client
    /// [`Bucket`]: crate::bucket::Bucket
    /// [`ObjectList`]: crate::object::ObjectList
    type Client;

    /// 指定要操作的 OSS 对象的路径，需要自行实现
    fn get_path(&self) -> <Self::Client as Files>::Path;

    /// 指定发起 OSS 接口调用的客户端
    fn oss_client(&self) -> Self::Client;

    /// 上传文件内容到 OSS 上面
    #[inline]
    async fn put_oss(
        &self,
        content: Vec<u8>,
        content_type: &str,
    ) -> Result<Response, <Self::Client as Files>::Err> {
        self.oss_client()
            .put_content_base(content, content_type, self.get_path())
            .await
    }

    /// # 获取 OSS 上文件的部分或全部内容
    /// 参数可指定范围:
    /// - `..` 获取文件的所有内容，常规大小的文件，使用这个即可
    /// - `..100`, `100..200`, `200..` 可用于获取文件的部分内容，一般用于大文件
    #[inline]
    async fn get_oss<R: Into<ContentRange> + Send + Sync>(
        &self,
        range: R,
    ) -> Result<Vec<u8>, <Self::Client as Files>::Err> {
        self.oss_client().get_object(self.get_path(), range).await
    }

    /// # 从 OSS 中删除文件
    #[inline]
    async fn delete_oss(&self) -> Result<(), <Self::Client as Files>::Err> {
        self.oss_client().delete_object(self.get_path()).await
    }
}

/// # 文件集合的相关操作
/// 在对文件执行相关操作的时候，需要指定文件路径
///
/// 包括 上传，下载，删除等功能
/// 在 [`Client`]，[`Bucket`], [`ObjectList`] 等结构体中均已实现，其中 Client 是在默认的 bucket 上操作文件，
/// 而 Bucket, ObjectList 则是在当前的 bucket 上操作文件
///
/// [`Client`]: crate::client::Client
/// [`Bucket`]: crate::bucket::Bucket
/// [`ObjectList`]: crate::object::ObjectList
#[async_trait]
pub trait Files: AlignBuilder
where
    Self::Path: Send + Sync,
    Self::Err: From<FileError>,
{
    type Path;
    type Err;

    /// # 默认的文件类型
    /// 在上传文件时，如果找不到合适的 mime 类型，可以使用
    const DEFAULT_CONTENT_TYPE: &'static str = "application/octet-stream";

    /// 根据文件路径获取最终的调用接口以及相关参数
    fn get_url(&self, path: Self::Path) -> Result<(Url, CanonicalizedResource), Self::Err>;

    /// # 上传文件到 OSS
    ///
    /// 需指定文件的路径
    ///
    /// *如果获取不到文件类型，则使用默认的文件类型，如果您不希望这么做，可以使用 `put_content_base` 方法*
    #[cfg(feature = "put_file")]
    async fn put_file<
        P: Into<std::path::PathBuf> + std::convert::AsRef<std::path::Path> + Send + Sync,
    >(
        &self,
        file_name: P,
        path: Self::Path,
    ) -> Result<String, Self::Err> {
        let file_content = std::fs::read(file_name).map_err(|e| e.into())?;

        let get_content_type =
            |content: &Vec<u8>| Infer::new().get(content).map(|con| con.mime_type());

        self.put_content(file_content, path, get_content_type).await
    }

    /// # 上传文件内容到 OSS
    ///
    /// 需指定要上传的文件内容
    /// 以及根据文件内容获取文件类型的闭包
    ///
    /// *如果获取不到文件类型，则使用默认的文件类型，如果您不希望这么做，可以使用 `put_content_base` 方法*
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
    /// use aliyun_oss_client::file::Files;
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
    ///     .put_content(content, "xxxxxx.msi.zip.sig".parse().unwrap(), get_content_type)
    ///     .await;
    /// assert!(res.is_ok());
    /// # }
    /// ```
    async fn put_content<F>(
        &self,
        content: Vec<u8>,
        path: Self::Path,
        get_content_type: F,
    ) -> Result<String, Self::Err>
    where
        F: Fn(&Vec<u8>) -> Option<&'static str> + Send + Sync,
    {
        let content_type = get_content_type(&content).unwrap_or(Self::DEFAULT_CONTENT_TYPE);

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
    async fn put_content_base(
        &self,
        content: Vec<u8>,
        content_type: &str,
        path: Self::Path,
    ) -> Result<Response, Self::Err> {
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
            .map_err(|e| FileError::from(e).into())?
            .body(content)
            .send_adjust_error()
            .await
            .map_err(|e| FileError::from(e).into())
    }

    /// # 获取 OSS 上文件的部分或全部内容
    async fn get_object<R: Into<ContentRange> + Send + Sync>(
        &self,
        path: Self::Path,
        range: R,
    ) -> Result<Vec<u8>, Self::Err> {
        let (url, canonicalized) = self.get_url(path)?;

        let list: Vec<(_, HeaderValue)> = vec![("Range".parse().unwrap(), range.into().into())];

        let content = self
            .builder_with_header(Method::GET, url, canonicalized, list)
            .map_err(|e| FileError::from(e).into())?
            .send_adjust_error()
            .await
            .map_err(|e| e.into())?
            .text()
            .await
            .map_err(|e| FileError::from(e).into())?;

        Ok(content.into_bytes())
    }

    /// # 删除 OSS 上的文件
    async fn delete_object(&self, path: Self::Path) -> Result<(), Self::Err> {
        let (url, canonicalized) = self.get_url(path)?;

        self.builder(Method::DELETE, url, canonicalized)
            .map_err(|e| FileError::from(e).into())?
            .send_adjust_error()
            .await
            .map_err(|e| FileError::from(e).into())?;

        Ok(())
    }
}

impl Files for Client {
    type Path = ObjectPath;
    type Err = FileError;
    fn get_url(&self, path: Self::Path) -> Result<(Url, CanonicalizedResource), FileError> {
        let object_base = ObjectBase::<ArcPointer>::new2(Arc::new(self.get_bucket_base()), path);
        Ok(object_base.get_url_resource([]))
    }
}

impl Files for Bucket {
    type Path = ObjectPath;
    type Err = FileError;
    fn get_url(&self, path: Self::Path) -> Result<(Url, CanonicalizedResource), FileError> {
        let object_base = ObjectBase::<ArcPointer>::new2(Arc::new(self.base.to_owned()), path);
        Ok(object_base.get_url_resource([]))
    }
}

/// 可将 `Object` 实例作为参数传递给各种操作方法
impl Files for ObjectList<ArcPointer> {
    type Path = Object<ArcPointer>;
    type Err = FileError;
    fn get_url(&self, path: Self::Path) -> Result<(Url, CanonicalizedResource), FileError> {
        let object_base =
            ObjectBase::<ArcPointer>::new2(Arc::new(self.bucket.to_owned()), path.into());
        Ok(object_base.get_url_resource([]))
    }
}

use oss_derive::path_where;

#[async_trait]
pub trait FileAs: Files<Path = ObjectPath>
where
    Self::Error: From<<Self as Files>::Err>,
{
    type Error;

    /// # 上传文件到 OSS
    ///
    /// 需指定文件的路径
    #[cfg(feature = "put_file")]
    #[path_where]
    async fn put_file_as<
        P: Into<std::path::PathBuf> + std::convert::AsRef<std::path::Path> + Send + Sync,
        OP,
    >(
        &self,
        file_name: P,
        path: OP,
    ) -> Result<String, Self::Error> {
        let path = path.try_into().map_err(|e| e.into())?;
        Files::put_file(self, file_name, path)
            .await
            .map_err(Self::Error::from)
    }

    /// # 上传文件内容到 OSS
    ///
    /// 需指定要上传的文件内容
    /// 以及根据文件内容获取文件类型的闭包
    #[path_where]
    async fn put_content_as<F, OP>(
        &self,
        content: Vec<u8>,
        path: OP,
        get_content_type: F,
    ) -> Result<String, Self::Error>
    where
        F: Fn(&Vec<u8>) -> Option<&'static str> + Send + Sync,
    {
        let path = path.try_into().map_err(|e| e.into())?;
        Files::put_content(self, content, path, get_content_type)
            .await
            .map_err(Self::Error::from)
    }

    /// 上传文件
    #[path_where]
    async fn put_content_base_as<OP>(
        &self,
        content: Vec<u8>,
        content_type: &str,
        path: OP,
    ) -> Result<Response, Self::Error> {
        let path = path.try_into().map_err(|e| e.into())?;
        Files::put_content_base(self, content, content_type, path)
            .await
            .map_err(Self::Error::from)
    }

    /// 获取文件内容
    #[path_where]
    async fn get_object_as<R: Into<ContentRange> + Send + Sync, OP>(
        &self,
        path: OP,
        range: R,
    ) -> Result<Vec<u8>, Self::Error> {
        let path = path.try_into().map_err(|e| e.into())?;
        Files::get_object(self, path, range)
            .await
            .map_err(Self::Error::from)
    }

    /// # 删除 OSS 上的文件
    #[path_where]
    async fn delete_object_as<OP>(&self, path: OP) -> Result<(), Self::Error> {
        let path = path.try_into().map_err(|e| e.into())?;
        Files::delete_object(self, path)
            .await
            .map_err(Self::Error::from)
    }
}

impl FileAs for Client {
    type Error = FileError;
}
impl FileAs for Bucket {
    type Error = FileError;
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

#[cfg(test)]
mod test_try {
    use crate::config::ObjectPath;
    use crate::file::FileError;
    use crate::Client;

    use super::FileAs;
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

        struct Path;
        impl TryFrom<Path> for ObjectPath {
            type Error = MyError;
            fn try_from(_path: Path) -> Result<Self, Self::Error> {
                ObjectPath::new("abc").map_err(|_| MyError {})
            }
        }

        struct MyError;

        impl Into<FileError> for MyError {
            fn into(self) -> FileError {
                FileError::FileTypeNotFound
            }
        }

        let _ = client.delete_object_as(Path {}).await;
    }
}

#[cfg(feature = "blocking")]
pub use blocking::Files as BlockingFile;

#[cfg(feature = "blocking")]
pub mod blocking {
    use std::rc::Rc;

    use super::FileError;
    use crate::{
        blocking::builder::RequestBuilder,
        bucket::Bucket,
        builder::{BuilderError, RcPointer},
        config::{ObjectBase, ObjectPath},
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

    pub trait Files: AlignBuilder
    where
        Self::Err: From<FileError>,
    {
        type Path;
        type Err;

        /// # 默认的文件类型
        /// 在上传文件时，如果找不到合适的 mime 类型，可以使用
        const DEFAULT_CONTENT_TYPE: &'static str = "application/octet-stream";

        /// 根据文件路径获取最终的调用接口以及相关参数
        fn get_url(&self, path: Self::Path) -> Result<(Url, CanonicalizedResource), Self::Err>;

        /// # 上传文件到 OSS
        ///
        /// 需指定文件的路径
        #[cfg(feature = "put_file")]
        fn put_file<P: Into<std::path::PathBuf> + std::convert::AsRef<std::path::Path>>(
            &self,
            file_name: P,
            path: Self::Path,
        ) -> Result<String, Self::Err> {
            let file_content = std::fs::read(file_name).map_err(|e| e.into())?;

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
        /// let res = client.put_content(content, "xxxxxx.msi.zip.sig".parse().unwrap(), get_content_type);
        /// assert!(res.is_ok());
        /// # }
        /// ```
        fn put_content<F>(
            &self,
            content: Vec<u8>,
            path: Self::Path,
            get_content_type: F,
        ) -> Result<String, Self::Err>
        where
            F: Fn(&Vec<u8>) -> Option<&'static str>,
        {
            let content_type = get_content_type(&content).unwrap_or("application/octet-stream");

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
        fn put_content_base(
            &self,
            content: Vec<u8>,
            content_type: &str,
            path: Self::Path,
        ) -> Result<Response, Self::Err> {
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
                .builder_with_header(Method::PUT, url, canonicalized, headers)
                .map_err(|e| FileError::from(e).into())?
                .body(content);

            response
                .send_adjust_error()
                .map_err(|e| FileError::from(e).into())
        }

        /// # 获取文件内容
        fn get_object<R: Into<ContentRange>>(
            &self,
            path: Self::Path,
            range: R,
        ) -> Result<Vec<u8>, Self::Err> {
            let (url, canonicalized) = self.get_url(path)?;

            let headers: Vec<(_, HeaderValue)> =
                vec![("Range".parse().unwrap(), range.into().into())];

            Ok(self
                .builder_with_header(Method::GET, url, canonicalized, headers)
                .map_err(|e| FileError::from(e).into())?
                .send_adjust_error()
                .map_err(|e| e.into())?
                .text()
                .map_err(|e| FileError::from(e).into())?
                .into_bytes())
        }

        fn delete_object(&self, path: Self::Path) -> Result<(), Self::Err> {
            let (url, canonicalized) = self.get_url(path)?;

            self.builder(Method::DELETE, url, canonicalized)
                .map_err(|e| FileError::from(e).into())?
                .send_adjust_error()
                .map_err(|e| FileError::from(e).into())?;

            Ok(())
        }
    }

    impl Files for ClientRc {
        type Path = ObjectPath;
        type Err = FileError;
        fn get_url(&self, path: Self::Path) -> Result<(Url, CanonicalizedResource), FileError> {
            let object_base = ObjectBase::<RcPointer>::new2(Rc::new(self.get_bucket_base()), path);

            Ok(object_base.get_url_resource([]))
        }
    }

    impl Files for Bucket<RcPointer> {
        type Path = ObjectPath;
        type Err = FileError;
        fn get_url(&self, path: Self::Path) -> Result<(Url, CanonicalizedResource), FileError> {
            let object_base = ObjectBase::<RcPointer>::new2(Rc::new(self.base.clone()), path);

            Ok(object_base.get_url_resource([]))
        }
    }

    impl Files for ObjectList<RcPointer> {
        type Path = Object<RcPointer>;
        type Err = FileError;
        fn get_url(&self, path: Self::Path) -> Result<(Url, CanonicalizedResource), FileError> {
            let object_base =
                ObjectBase::<RcPointer>::new2(Rc::new(self.bucket.clone()), path.into());

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
            let content_type = get_content_type(&content).unwrap_or("application/octet-stream");

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
