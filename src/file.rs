//! # OSS 文件相关操作
//!
//! [`File`] 是一个文件操作的工具包，包含上传，下载，删除功能，开发者可以方便的调用使用
//!
//! ```rust
//! use std::{fs, path::Path};
//!
//! use aliyun_oss_client::{
//!     config::get_url_resource,
//!     file::{File, FileError, GetStd},
//!     types::CanonicalizedResource,
//!     BucketName, Client, EndPoint, KeyId, KeySecret,
//! };
//! use reqwest::Url;
//!
//! struct MyObject {
//!     path: String,
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
//!             path: path.to_str().unwrap().to_owned(),
//!         })
//!     }
//! }
//!
//! impl GetStd for MyObject {
//!     fn get_std(&self) -> Option<(Url, CanonicalizedResource)> {
//!         let path = self.path.clone().try_into().unwrap();
//!         Some(get_url_resource(&Self::END_POINT, &Self::BUCKET, &path))
//!     }
//! }
//!
//! impl File<Client> for MyObject {
//!     fn oss_client(&self) -> Client {
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
//!         let res = obj.put_oss(content, "application/pdf").await?;
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
    client::ClientArc,
    config::{get_url_resource, InvalidObjectPath, ObjectBase, ObjectPath},
    decode::{ItemError, RefineObject},
    object::{Object, ObjectList},
    types::{CanonicalizedResource, ContentRange},
};
#[cfg(feature = "put_file")]
use infer::Infer;
use oss_derive::oss_gen_rc;

/// # 文件的相关操作
///
/// 包括 上传，下载，删除等功能
#[async_trait]
pub trait File<Client>: GetStd
where
    Client: Files<ObjectPath>,
{
    /// 指定发起 OSS 接口调用的客户端
    fn oss_client(&self) -> Client;

    /// 上传文件内容到 OSS 上面
    async fn put_oss(&self, content: Vec<u8>, content_type: &str) -> Result<Response, FileError> {
        let (url, canonicalized) = self
            .get_std()
            .ok_or(FileError::NotFoundCanonicalizedResource)?;

        let content_length = content.len().to_string();
        let headers = vec![
            (
                CONTENT_LENGTH,
                HeaderValue::from_str(&content_length).map_err(FileError::from)?,
            ),
            (
                CONTENT_TYPE,
                content_type
                    .parse()
                    .unwrap_or_else(|_| Client::DEFAULT_CONTENT_TYPE.parse().unwrap()),
            ),
        ];

        self.oss_client()
            .builder_with_header(Method::PUT, url, canonicalized, headers)
            .map_err(FileError::from)?
            .body(content)
            .send_adjust_error()
            .await
            .map_err(FileError::from)
    }

    /// # 获取 OSS 上文件的部分或全部内容
    /// 参数可指定范围:
    /// - `..` 获取文件的所有内容，常规大小的文件，使用这个即可
    /// - `..100`, `100..200`, `200..` 可用于获取文件的部分内容，一般用于大文件
    async fn get_oss<R: Into<ContentRange> + Send + Sync>(
        &self,
        range: R,
    ) -> Result<Vec<u8>, FileError> {
        let (url, canonicalized) = self
            .get_std()
            .ok_or(FileError::NotFoundCanonicalizedResource)?;

        let list: Vec<(_, HeaderValue)> = vec![("Range".parse().unwrap(), range.into().into())];

        let content = self
            .oss_client()
            .builder_with_header(Method::GET, url, canonicalized, list)
            .map_err(FileError::from)?
            .send_adjust_error()
            .await?
            .text()
            .await
            .map_err(FileError::from)?;

        Ok(content.into_bytes())
    }

    /// # 从 OSS 中删除文件
    async fn delete_oss(&self) -> Result<(), FileError> {
        let (url, canonicalized) = self
            .get_std()
            .ok_or(FileError::NotFoundCanonicalizedResource)?;

        self.oss_client()
            .builder(Method::DELETE, url, canonicalized)
            .map_err(FileError::from)?
            .send_adjust_error()
            .await
            .map_err(FileError::from)?;

        Ok(())
    }
}

/// 获取请求 OSS 接口需要的信息
pub trait GetStd {
    /// 获取 Url 和 CanonicalizedResource
    fn get_std(&self) -> Option<(Url, CanonicalizedResource)>;
}

impl GetStd for ObjectBase<ArcPointer> {
    fn get_std(&self) -> Option<(Url, CanonicalizedResource)> {
        Some(self.get_url_resource([]))
    }
}

impl GetStd for Object<ArcPointer> {
    fn get_std(&self) -> Option<(Url, CanonicalizedResource)> {
        Some(self.base.get_url_resource([]))
    }
}

impl GetStd for &Object<ArcPointer> {
    fn get_std(&self) -> Option<(Url, CanonicalizedResource)> {
        Some(self.base.get_url_resource([]))
    }
}

/// 根据给定路径，获取请求 OSS 接口需要的信息
pub trait GetStdWithPath<Path> {
    /// 根据 path 获取 Url 和 CanonicalizedResource
    fn get_std_with_path(&self, _path: Path) -> Option<(Url, CanonicalizedResource)>;
}

#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
#[cfg(feature = "blocking")]
use crate::client::ClientRc;
#[cfg(feature = "blocking")]
use std::rc::Rc;

#[oss_gen_rc]
impl GetStdWithPath<String> for ClientArc {
    fn get_std_with_path(&self, path: String) -> Option<(Url, CanonicalizedResource)> {
        let object_path = path.try_into().ok()?;
        Some(get_url_resource(
            self.get_endpoint(),
            self.get_bucket_name(),
            &object_path,
        ))
    }
}

#[oss_gen_rc]
impl GetStdWithPath<&str> for ClientArc {
    fn get_std_with_path(&self, path: &str) -> Option<(Url, CanonicalizedResource)> {
        let object_path = path.to_owned().try_into().ok()?;
        Some(get_url_resource(
            self.get_endpoint(),
            self.get_bucket_name(),
            &object_path,
        ))
    }
}

#[oss_gen_rc]
impl GetStdWithPath<ObjectPath> for ClientArc {
    fn get_std_with_path(&self, path: ObjectPath) -> Option<(Url, CanonicalizedResource)> {
        Some(get_url_resource(
            self.get_endpoint(),
            self.get_bucket_name(),
            &path,
        ))
    }
}

#[oss_gen_rc]
impl GetStdWithPath<&ObjectPath> for ClientArc {
    fn get_std_with_path(&self, path: &ObjectPath) -> Option<(Url, CanonicalizedResource)> {
        Some(get_url_resource(
            self.get_endpoint(),
            self.get_bucket_name(),
            path,
        ))
    }
}

#[oss_gen_rc]
impl GetStdWithPath<ObjectBase> for ClientArc {
    #[inline]
    fn get_std_with_path(&self, base: ObjectBase) -> Option<(Url, CanonicalizedResource)> {
        Some(base.get_url_resource([]))
    }
}

#[oss_gen_rc]
impl GetStdWithPath<&ObjectBase> for ClientArc {
    #[inline]
    fn get_std_with_path(&self, base: &ObjectBase) -> Option<(Url, CanonicalizedResource)> {
        Some(base.get_url_resource([]))
    }
}

impl GetStdWithPath<String> for Bucket {
    fn get_std_with_path(&self, path: String) -> Option<(Url, CanonicalizedResource)> {
        let path = path.try_into().ok()?;
        Some(self.base.get_url_resource_with_path(&path))
    }
}

impl GetStdWithPath<&str> for Bucket {
    fn get_std_with_path(&self, path: &str) -> Option<(Url, CanonicalizedResource)> {
        let path = path.to_owned().try_into().ok()?;
        Some(self.base.get_url_resource_with_path(&path))
    }
}

impl GetStdWithPath<ObjectPath> for Bucket {
    #[inline]
    fn get_std_with_path(&self, path: ObjectPath) -> Option<(Url, CanonicalizedResource)> {
        Some(self.base.get_url_resource_with_path(&path))
    }
}

impl GetStdWithPath<&ObjectPath> for Bucket {
    #[inline]
    fn get_std_with_path(&self, path: &ObjectPath) -> Option<(Url, CanonicalizedResource)> {
        Some(self.base.get_url_resource_with_path(path))
    }
}

#[oss_gen_rc]
impl GetStdWithPath<ObjectBase<ArcPointer>> for Bucket {
    #[inline]
    fn get_std_with_path(
        &self,
        base: ObjectBase<ArcPointer>,
    ) -> Option<(Url, CanonicalizedResource)> {
        Some(base.get_url_resource([]))
    }
}

#[oss_gen_rc]
impl GetStdWithPath<&ObjectBase<ArcPointer>> for Bucket {
    #[inline]
    fn get_std_with_path(
        &self,
        base: &ObjectBase<ArcPointer>,
    ) -> Option<(Url, CanonicalizedResource)> {
        Some(base.get_url_resource([]))
    }
}

#[oss_gen_rc]
impl<Item: RefineObject<E> + Send + Sync, E: ItemError + Send + Sync> GetStdWithPath<String>
    for ObjectList<ArcPointer, Item, E>
{
    fn get_std_with_path(&self, path: String) -> Option<(Url, CanonicalizedResource)> {
        let object_base =
            ObjectBase::<ArcPointer>::new2(Arc::new(self.bucket.to_owned()), path.try_into().ok()?);
        Some(object_base.get_url_resource([]))
    }
}

#[oss_gen_rc]
impl<Item: RefineObject<E> + Send + Sync, E: ItemError + Send + Sync> GetStdWithPath<&str>
    for ObjectList<ArcPointer, Item, E>
{
    fn get_std_with_path(&self, path: &str) -> Option<(Url, CanonicalizedResource)> {
        let object_base = ObjectBase::<ArcPointer>::new2(
            Arc::new(self.bucket.to_owned()),
            path.to_owned().try_into().ok()?,
        );
        Some(object_base.get_url_resource([]))
    }
}

#[oss_gen_rc]
impl<Item: RefineObject<E> + Send + Sync, E: ItemError + Send + Sync> GetStdWithPath<ObjectPath>
    for ObjectList<ArcPointer, Item, E>
{
    fn get_std_with_path(&self, path: ObjectPath) -> Option<(Url, CanonicalizedResource)> {
        let object_base = ObjectBase::<ArcPointer>::new2(Arc::new(self.bucket.to_owned()), path);
        Some(object_base.get_url_resource([]))
    }
}

#[oss_gen_rc]
impl<Item: RefineObject<E> + Send + Sync, E: ItemError + Send + Sync> GetStdWithPath<&ObjectPath>
    for ObjectList<ArcPointer, Item, E>
{
    fn get_std_with_path(&self, path: &ObjectPath) -> Option<(Url, CanonicalizedResource)> {
        let object_base =
            ObjectBase::<ArcPointer>::new2(Arc::new(self.bucket.to_owned()), path.to_owned());
        Some(object_base.get_url_resource([]))
    }
}

impl<Item: RefineObject<E> + Send + Sync, E: ItemError + Send + Sync, U: GetStd> GetStdWithPath<U>
    for ObjectList<ArcPointer, Item, E>
{
    #[inline]
    fn get_std_with_path(&self, path: U) -> Option<(Url, CanonicalizedResource)> {
        path.get_std()
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
pub trait Files<Path>: AlignBuilder + GetStdWithPath<Path>
where
    Path: Send + Sync + 'static,
{
    /// # 默认的文件类型
    /// 在上传文件时，如果找不到合适的 mime 类型，可以使用
    const DEFAULT_CONTENT_TYPE: &'static str = "application/octet-stream";

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
        path: Path,
    ) -> Result<String, FileError> {
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
    ///     .put_content(
    ///         content,
    ///         "xxxxxx.msi.zip.sig",
    ///         get_content_type,
    ///     )
    ///     .await;
    /// assert!(res.is_ok());
    /// # }
    /// ```
    async fn put_content<F>(
        &self,
        content: Vec<u8>,
        path: Path,
        get_content_type: F,
    ) -> Result<String, FileError>
    where
        F: Fn(&Vec<u8>) -> Option<&'static str> + Send + Sync,
    {
        let content_type = get_content_type(&content).unwrap_or(Self::DEFAULT_CONTENT_TYPE);

        let content = self.put_content_base(content, content_type, path).await?;

        let result = content
            .headers()
            .get("ETag")
            .ok_or(FileError::EtagNotFound)?
            .to_str()
            .map_err(FileError::from)?;

        Ok(result.to_string())
    }

    /// 最核心的上传文件到 OSS 的方法
    async fn put_content_base(
        &self,
        content: Vec<u8>,
        content_type: &str,
        path: Path,
    ) -> Result<Response, FileError> {
        let (url, canonicalized) = self
            .get_std_with_path(path)
            .ok_or(FileError::NotFoundCanonicalizedResource)?;

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

    /// # 获取 OSS 上文件的部分或全部内容
    async fn get_object<R: Into<ContentRange> + Send + Sync>(
        &self,
        path: Path,
        range: R,
    ) -> Result<Vec<u8>, FileError> {
        let (url, canonicalized) = self
            .get_std_with_path(path)
            .ok_or(FileError::NotFoundCanonicalizedResource)?;

        let list: Vec<(_, HeaderValue)> = vec![("Range".parse().unwrap(), range.into().into())];

        let content = self
            .builder_with_header(Method::GET, url, canonicalized, list)
            .map_err(FileError::from)?
            .send_adjust_error()
            .await
            .map_err(FileError::from)?
            .text()
            .await
            .map_err(FileError::from)?;

        Ok(content.into_bytes())
    }

    /// # 删除 OSS 上的文件
    async fn delete_object(&self, path: Path) -> Result<(), FileError> {
        let (url, canonicalized) = self
            .get_std_with_path(path)
            .ok_or(FileError::NotFoundCanonicalizedResource)?;

        self.builder(Method::DELETE, url, canonicalized)
            .map_err(FileError::from)?
            .send_adjust_error()
            .await
            .map_err(FileError::from)?;

        Ok(())
    }
}

/// # 为更多的类型实现 上传，下载，删除等功能
///
/// 在 [`Client`]，[`Bucket`], [`ObjectList`] 等结构体中均已实现，其中 Client 是在默认的 bucket 上操作文件，
/// 而 Bucket, ObjectList 则是在当前的 bucket 上操作文件
///
/// [`Client`]: crate::client::Client
/// [`Bucket`]: crate::bucket::Bucket
/// [`ObjectList`]: crate::object::ObjectList
impl<P: Send + Sync + 'static, T: AlignBuilder + GetStdWithPath<P>> Files<P> for T {}

/// 文件模块的 Error 集合
#[derive(Debug)]
pub enum FileError {
    #[doc(hidden)]
    Path(InvalidObjectPath),
    #[doc(hidden)]
    Io(std::io::Error),
    #[doc(hidden)]
    ToStr(http::header::ToStrError),
    #[doc(hidden)]
    HeaderValue(InvalidHeaderValue),
    #[doc(hidden)]
    Build(BuilderError),
    #[doc(hidden)]
    FileTypeNotFound,
    #[doc(hidden)]
    EtagNotFound,
    #[doc(hidden)]
    NotFoundCanonicalizedResource,
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
            Self::NotFoundCanonicalizedResource => write!(f, "Not found CanonicalizedResource"),
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
/// 用于他们方便的实现 [`Files`] trait
///
/// [`Files`]: self::Files
/// [`Client`]: crate::client::Client
/// [`Bucket`]: crate::bucket::Bucket
/// [`ObjectList`]: crate::object::ObjectList
pub trait AlignBuilder: Send + Sync {
    /// 根据具体的 API 接口参数，返回请求的构建器（不带 headers）
    #[inline]
    fn builder(
        &self,
        method: Method,
        url: Url,
        resource: CanonicalizedResource,
    ) -> Result<RequestBuilder, BuilderError> {
        self.builder_with_header(method, url, resource, [])
    }

    /// 根据具体的 API 接口参数，返回请求的构建器
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

impl<Item: RefineObject<E> + Send + Sync, E: ItemError + Send + Sync> AlignBuilder
    for ObjectList<ArcPointer, Item, E>
{
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
    use std::sync::Arc;

    use crate::builder::ArcPointer;
    use crate::config::{ObjectBase, ObjectPath};
    use crate::file::{FileError, Files};
    use crate::Client;

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

        struct MyPath;
        impl TryFrom<MyPath> for ObjectBase<ArcPointer> {
            type Error = MyError;
            fn try_from(_path: MyPath) -> Result<Self, Self::Error> {
                Ok(ObjectBase::<ArcPointer>::new2(
                    Arc::new("abc".parse().unwrap()),
                    "cde".parse().unwrap(),
                ))
            }
        }

        struct MyError;

        impl Into<FileError> for MyError {
            fn into(self) -> FileError {
                FileError::FileTypeNotFound
            }
        }

        //let _ = FileAs::<ObjectPath>::delete_object_as(&client, "abc".to_string()).await;
        let _ = client
            .delete_object("abc".parse::<ObjectPath>().unwrap())
            .await;
    }
}

#[cfg(feature = "blocking")]
pub use blocking::Files as BlockingFile;

/// 同步的文件模块
#[cfg(feature = "blocking")]
pub mod blocking {

    use super::{FileError, GetStdWithPath};
    use crate::{
        blocking::builder::RequestBuilder,
        bucket::Bucket,
        builder::{BuilderError, RcPointer},
        object::{Object, ObjectList},
        types::{CanonicalizedResource, ContentRange},
    };
    use http::{
        header::{HeaderName, CONTENT_LENGTH, CONTENT_TYPE},
        HeaderValue, Method,
    };
    #[cfg(feature = "put_file")]
    use infer::Infer;
    use reqwest::{blocking::Response, Url};

    /// # 文件集合的相关操作
    /// 在对文件执行相关操作的时候，需要指定文件路径
    pub trait Files<Path>: AlignBuilder + GetStdWithPath<Path> {
        /// # 默认的文件类型
        /// 在上传文件时，如果找不到合适的 mime 类型，可以使用
        const DEFAULT_CONTENT_TYPE: &'static str = "application/octet-stream";

        /// # 上传文件到 OSS
        ///
        /// 需指定文件的路径
        #[cfg(feature = "put_file")]
        fn put_file<P: Into<std::path::PathBuf> + std::convert::AsRef<std::path::Path>>(
            &self,
            file_name: P,
            path: Path,
        ) -> Result<String, FileError> {
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
        /// let res = client.put_content(
        ///     content,
        ///     "xxxxxx.msi.zip.sig",
        ///     get_content_type,
        /// );
        /// assert!(res.is_ok());
        /// # }
        /// ```
        fn put_content<F>(
            &self,
            content: Vec<u8>,
            path: Path,
            get_content_type: F,
        ) -> Result<String, FileError>
        where
            F: Fn(&Vec<u8>) -> Option<&'static str>,
        {
            let content_type = get_content_type(&content).unwrap_or(Self::DEFAULT_CONTENT_TYPE);

            let content = self.put_content_base(content, content_type, path)?;

            let result = content
                .headers()
                .get("ETag")
                .ok_or(FileError::EtagNotFound)?
                .to_str()
                .map_err(FileError::from)?;

            Ok(result.to_string())
        }

        /// 最原始的上传文件的方法
        fn put_content_base(
            &self,
            content: Vec<u8>,
            content_type: &str,
            path: Path,
        ) -> Result<Response, FileError> {
            let (url, canonicalized) = self
                .get_std_with_path(path)
                .ok_or(FileError::NotFoundCanonicalizedResource)?;

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
                .map_err(FileError::from)?
                .body(content);

            response.send_adjust_error().map_err(FileError::from)
        }

        /// # 获取文件内容
        fn get_object<R: Into<ContentRange>>(
            &self,
            path: Path,
            range: R,
        ) -> Result<Vec<u8>, FileError> {
            let (url, canonicalized) = self
                .get_std_with_path(path)
                .ok_or(FileError::NotFoundCanonicalizedResource)?;

            let headers: Vec<(_, HeaderValue)> =
                vec![("Range".parse().unwrap(), range.into().into())];

            Ok(self
                .builder_with_header(Method::GET, url, canonicalized, headers)
                .map_err(FileError::from)?
                .send_adjust_error()
                .map_err(FileError::from)?
                .text()
                .map_err(FileError::from)?
                .into_bytes())
        }

        /// # 删除 OSS 上的文件
        fn delete_object(&self, path: Path) -> Result<(), FileError> {
            let (url, canonicalized) = self
                .get_std_with_path(path)
                .ok_or(FileError::NotFoundCanonicalizedResource)?;

            self.builder(Method::DELETE, url, canonicalized)
                .map_err(FileError::from)?
                .send_adjust_error()
                .map_err(FileError::from)?;

            Ok(())
        }
    }

    impl<P, T: AlignBuilder + GetStdWithPath<P>> Files<P> for T {}

    /// 对 Client 中的请求构建器进行抽象
    pub trait AlignBuilder {
        /// 根据具体的 API 接口参数，返回请求的构建器（不带 headers）
        #[inline]
        fn builder(
            &self,
            method: Method,
            url: Url,
            resource: CanonicalizedResource,
        ) -> Result<RequestBuilder, BuilderError> {
            self.builder_with_header(method, url, resource, [])
        }

        /// 根据具体的 API 接口参数，返回请求的构建器
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
}
