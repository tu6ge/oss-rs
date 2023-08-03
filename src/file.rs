//! # OSS 文件相关操作
//!
//! [`File`] 是一个文件操作的工具包，包含上传，下载，删除功能，开发者可以方便的调用使用
//!
//! ```rust
//! use std::{fs, io, path::Path};
//!
//! use aliyun_oss_client::{
//!     config::get_url_resource,
//!     errors::OssError,
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
//!     fn new(path: &Path) -> Result<MyObject, io::Error> {
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
//! async fn run() -> Result<(), OssError> {
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
//!         let res = obj
//!             .put_oss(content, "application/pdf")
//!             .await
//!             .map_err(OssError::from)?;
//!
//!         println!("result status: {}", res.status());
//!     }
//!
//!     Ok(())
//! }
//! ```
//! [`File`]: crate::file::File

use async_trait::async_trait;
use chrono::Duration;
use http::{
    header::{HeaderName, CONTENT_LENGTH, CONTENT_TYPE},
    HeaderValue, Method,
};
use reqwest::{Response, Url};

use crate::{
    bucket::Bucket,
    builder::{ArcPointer, BuilderError, RequestBuilder},
    object::{Object, ObjectList},
    types::object::{ObjectBase, ObjectPath},
    types::{CanonicalizedResource, ContentRange},
};
#[cfg(feature = "put_file")]
use infer::Infer;

#[cfg(test)]
mod test;

const ETAG: &str = "ETag";
const RANGE: &str = "Range";

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
        let (url, canonicalized) = self.get_std().ok_or(FileError {
            kind: FileErrorKind::NotFoundCanonicalizedResource,
        })?;

        let content_length = content.len().to_string();
        let headers = vec![
            (CONTENT_LENGTH, header_from_content_length(&content_length)?),
            (
                CONTENT_TYPE,
                content_type.parse().map_err(|e| FileError {
                    kind: FileErrorKind::InvalidContentType(e),
                })?,
            ),
        ];

        self.oss_client()
            .builder_with_header(Method::PUT, url, canonicalized, headers)?
            .body(content)
            .send_adjust_error()
            .await
            .map_err(FileError::from)
    }

    /// # 获取 OSS 上文件的部分或全部内容
    /// 参数可指定范围:
    /// - `..` 获取文件的所有内容，常规大小的文件，使用这个即可
    /// - `..100`, `100..200`, `200..` 可用于获取文件的部分内容，一般用于大文件
    async fn get_oss<Num, R>(&self, range: R) -> Result<Vec<u8>, FileError>
    where
        R: Into<ContentRange<Num>> + Send + Sync,
        ContentRange<Num>: Into<HeaderValue>,
    {
        let (url, canonicalized) = self.get_std().ok_or(FileError {
            kind: FileErrorKind::NotFoundCanonicalizedResource,
        })?;

        let list: Vec<(_, HeaderValue)> = vec![(
            {
                #[allow(clippy::unwrap_used)]
                RANGE.parse().unwrap()
            },
            range.into().into(),
        )];

        let content = self
            .oss_client()
            .builder_with_header(Method::GET, url, canonicalized, list)?
            .send_adjust_error()
            .await?
            .text()
            .await?;

        Ok(content.into_bytes())
    }

    /// # 从 OSS 中删除文件
    async fn delete_oss(&self) -> Result<(), FileError> {
        let (url, canonicalized) = self.get_std().ok_or(FileError {
            kind: FileErrorKind::NotFoundCanonicalizedResource,
        })?;

        self.oss_client()
            .builder(Method::DELETE, url, canonicalized)?
            .send_adjust_error()
            .await?;

        Ok(())
    }
}

/// 获取请求 OSS 接口需要的信息
pub trait GetStd {
    /// 获取 `Url` 和 `CanonicalizedResource`
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

/// 根据给定路径，获取请求 OSS 接口需要的信息
pub trait GetStdWithPath<Path> {
    /// 根据 path 获取 `Url` 和 `CanonicalizedResource`
    fn get_std_with_path(&self, _path: Path) -> Option<(Url, CanonicalizedResource)>;
}

#[doc(hidden)]
pub mod std_path_impl {
    #[cfg(feature = "blocking")]
    use crate::builder::RcPointer;
    #[cfg(feature = "blocking")]
    use crate::client::ClientRc;

    use super::{GetStd, GetStdWithPath};
    use crate::{
        bucket::Bucket,
        builder::ArcPointer,
        client::ClientArc,
        config::{get_url_resource2 as get_url_resource, BucketBase},
        decode::RefineObject,
        object::{BuildInItemError, ObjectList},
        types::{object::ObjectBase, CanonicalizedResource},
        ObjectPath,
    };
    use oss_derive::oss_gen_rc;
    use reqwest::Url;

    /// # 用于在 Client 上对文件进行操作
    ///
    /// 文件路径可以是 `String` 类型
    ///
    /// [`ObjectPath`]: crate::ObjectPath
    #[oss_gen_rc]
    impl GetStdWithPath<String> for ClientArc {
        fn get_std_with_path(&self, path: String) -> Option<(Url, CanonicalizedResource)> {
            let object_path = path.try_into().ok()?;
            Some(get_url_resource(self, self, &object_path))
        }
    }

    /// # 用于在 Client 上对文件进行操作
    ///
    /// 文件路径可以是 `&str` 类型
    ///
    /// [`ObjectPath`]: crate::ObjectPath
    #[oss_gen_rc]
    impl GetStdWithPath<&str> for ClientArc {
        fn get_std_with_path(&self, path: &str) -> Option<(Url, CanonicalizedResource)> {
            let object_path = path.try_into().ok()?;
            Some(get_url_resource(self, self, &object_path))
        }
    }

    /// # 用于在 Client 上对文件进行操作
    ///
    /// 文件路径可以是 [`ObjectPath`] 类型
    ///
    /// [`ObjectPath`]: crate::ObjectPath
    #[oss_gen_rc]
    impl GetStdWithPath<ObjectPath> for ClientArc {
        fn get_std_with_path(&self, path: ObjectPath) -> Option<(Url, CanonicalizedResource)> {
            Some(get_url_resource(self, self, &path))
        }
    }

    /// # 用于在 Client 上对文件进行操作
    ///
    /// 文件路径可以是 [`&ObjectPath`] 类型
    ///
    /// [`&ObjectPath`]: crate::ObjectPath
    #[oss_gen_rc]
    impl<Path: AsRef<ObjectPath>> GetStdWithPath<Path> for ClientArc {
        fn get_std_with_path(&self, path: Path) -> Option<(Url, CanonicalizedResource)> {
            Some(get_url_resource(self, self, path.as_ref()))
        }
    }

    /// # 用于在 Bucket 上对文件进行操作
    ///
    /// 文件路径可以是 `String` 类型
    impl<B: AsRef<BucketBase>> GetStdWithPath<String> for B {
        fn get_std_with_path(&self, path: String) -> Option<(Url, CanonicalizedResource)> {
            let path = path.try_into().ok()?;
            Some(self.as_ref().get_url_resource_with_path(&path))
        }
    }

    /// # 用于在 Bucket 上对文件进行操作
    ///
    /// 文件路径可以是 `&str` 类型
    impl<B: AsRef<BucketBase>> GetStdWithPath<&str> for B {
        fn get_std_with_path(&self, path: &str) -> Option<(Url, CanonicalizedResource)> {
            let path = path.try_into().ok()?;
            Some(self.as_ref().get_url_resource_with_path(&path))
        }
    }

    /// # 用于在 Bucket 上对文件进行操作
    ///
    /// 文件路径可以是 [`ObjectPath`] 类型
    ///
    /// [`ObjectPath`]: crate::ObjectPath
    impl<B: AsRef<BucketBase>> GetStdWithPath<ObjectPath> for B {
        #[inline]
        fn get_std_with_path(&self, path: ObjectPath) -> Option<(Url, CanonicalizedResource)> {
            Some(self.as_ref().get_url_resource_with_path(&path))
        }
    }

    /// # 用于在 Bucket 上对文件进行操作
    ///
    /// 文件路径可以是 [`&ObjectPath`] 类型
    ///
    /// [`&ObjectPath`]: crate::ObjectPath
    impl<B: AsRef<BucketBase>> GetStdWithPath<&ObjectPath> for B {
        #[inline]
        fn get_std_with_path(&self, path: &ObjectPath) -> Option<(Url, CanonicalizedResource)> {
            Some(self.as_ref().get_url_resource_with_path(path))
        }
    }

    /// # 用于在 Bucket 上对文件进行操作
    ///
    /// 文件路径可以是 [`ObjectBase`] 类型
    ///
    /// [`ObjectBase`]: crate::types::object::ObjectBase
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

    /// # 用于在 Bucket 上对文件进行操作
    ///
    /// 文件路径可以是 [`&ObjectBase`] 类型
    ///
    /// [`&ObjectBase`]: crate::types::object::ObjectBase
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

    /// # 用于在 ObjectList 上对文件进行操作
    ///
    /// 文件路径可以是实现 [`GetStd`] 特征的类型
    ///
    /// [`GetStd`]: crate::file::GetStd
    ///
    /// TODO remove Send + Sync
    impl<Item: RefineObject<BuildInItemError> + Send + Sync, U: GetStd> GetStdWithPath<U>
        for ObjectList<ArcPointer, Item>
    {
        #[inline]
        fn get_std_with_path(&self, path: U) -> Option<(Url, CanonicalizedResource)> {
            path.get_std()
        }
    }
}

/// 默认 content-type
pub const DEFAULT_CONTENT_TYPE: &'static str = "application/octet-stream";

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
    const DEFAULT_CONTENT_TYPE: &'static str = DEFAULT_CONTENT_TYPE;

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
        let file_content = std::fs::read(file_name).map_err(|e| FileError {
            kind: error_impl::FileErrorKind::FileRead(e),
        })?;

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
    ///     .put_content(content, "xxxxxx.msi.zip.sig", get_content_type)
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
            .get(ETAG)
            .ok_or(FileError {
                kind: FileErrorKind::EtagNotFound,
            })?
            .to_str()
            .map_err(|e| FileError {
                kind: FileErrorKind::InvalidEtag(e),
            })?;

        // TODO change to result[1..33].to_string()
        // 不能使用该方案，返回的etag长度不固定
        Ok(result.to_string())
    }

    /// 最核心的上传文件到 OSS 的方法
    async fn put_content_base(
        &self,
        content: Vec<u8>,
        content_type: &str,
        path: Path,
    ) -> Result<Response, FileError> {
        let (url, canonicalized) = self.get_std_with_path(path).ok_or(FileError {
            kind: FileErrorKind::NotFoundCanonicalizedResource,
        })?;

        let content_length = content.len().to_string();
        let headers = vec![
            (CONTENT_LENGTH, header_from_content_length(&content_length)?),
            (
                CONTENT_TYPE,
                content_type.parse().map_err(|e| FileError {
                    kind: FileErrorKind::InvalidContentType(e),
                })?,
            ),
        ];

        self.builder_with_header(Method::PUT, url, canonicalized, headers)?
            .body(content)
            //.timeout(std::time::Duration::new(3, 0))
            .send_adjust_error()
            .await
            .map_err(FileError::from)
    }

    /// # 获取 OSS 上文件的部分或全部内容
    async fn get_object<Num, R>(&self, path: Path, range: R) -> Result<Vec<u8>, FileError>
    where
        R: Into<ContentRange<Num>> + Send + Sync,
        ContentRange<Num>: Into<HeaderValue>,
    {
        let (url, canonicalized) = self.get_std_with_path(path).ok_or(FileError {
            kind: FileErrorKind::NotFoundCanonicalizedResource,
        })?;

        let list: Vec<(_, HeaderValue)> = vec![(
            {
                #[allow(clippy::unwrap_used)]
                RANGE.parse().unwrap()
            },
            range.into().into(),
        )];

        let content = self
            .builder_with_header(Method::GET, url, canonicalized, list)?
            .send_adjust_error()
            .await?
            .text()
            .await?;

        Ok(content.into_bytes())
    }

    /// # 删除 OSS 上的文件
    async fn delete_object(&self, path: Path) -> Result<(), FileError> {
        let (url, canonicalized) = self.get_std_with_path(path).ok_or(FileError {
            kind: FileErrorKind::NotFoundCanonicalizedResource,
        })?;

        self.builder(Method::DELETE, url, canonicalized)?
            .send_adjust_error()
            .await?;

        Ok(())
    }
}

fn header_from_content_length(content: &str) -> Result<HeaderValue, FileError> {
    HeaderValue::from_str(content).map_err(|e| FileError {
        kind: FileErrorKind::InvalidContentLength(e),
    })
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

/// 文件模块的 Error
#[derive(Debug)]
pub struct FileError {
    kind: error_impl::FileErrorKind,
}

impl FileError {
    #[cfg(test)]
    pub(crate) fn test_new() -> Self {
        Self {
            kind: error_impl::FileErrorKind::EtagNotFound,
        }
    }
}

/// 文件模块的 Error 实现方法
mod error_impl {
    use std::{error::Error, fmt::Display, io::ErrorKind};

    use http::header::InvalidHeaderValue;

    use crate::builder::{reqwest_to_io, BuilderError};

    use super::FileError;

    impl Display for FileError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use FileErrorKind::*;
            match &self.kind {
                #[cfg(feature = "put_file")]
                FileRead(_) => write!(f, "file read failed"),
                InvalidContentLength(_) => write!(f, "invalid content length"),
                InvalidContentType(_) => write!(f, "invalid content type"),
                Build(to) => write!(f, "{to}"),
                Reqwest(_) => write!(f, "reqwest error"),
                EtagNotFound => write!(f, "failed to get etag"),
                InvalidEtag(_) => write!(f, "invalid etag"),
                NotFoundCanonicalizedResource => write!(f, "not found canonicalized-resource"),
            }
        }
    }

    impl Error for FileError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            use FileErrorKind::*;
            match &self.kind {
                #[cfg(feature = "put_file")]
                FileRead(e) => Some(e),
                InvalidContentLength(e) | InvalidContentType(e) => Some(e),
                Build(e) => e.source(),
                Reqwest(e) => Some(e),
                InvalidEtag(e) => Some(e),
                EtagNotFound | NotFoundCanonicalizedResource => None,
            }
        }
    }

    #[derive(Debug)]
    pub(super) enum FileErrorKind {
        #[cfg(feature = "put_file")]
        FileRead(std::io::Error),
        InvalidContentLength(InvalidHeaderValue),
        InvalidContentType(InvalidHeaderValue),
        Build(BuilderError),
        Reqwest(reqwest::Error),
        EtagNotFound,
        InvalidEtag(http::header::ToStrError),
        NotFoundCanonicalizedResource,
    }

    impl From<BuilderError> for FileError {
        fn from(value: BuilderError) -> Self {
            Self {
                kind: FileErrorKind::Build(value),
            }
        }
    }

    impl From<reqwest::Error> for FileError {
        fn from(value: reqwest::Error) -> Self {
            Self {
                kind: FileErrorKind::Reqwest(value),
            }
        }
    }

    impl From<FileError> for std::io::Error {
        fn from(FileError { kind }: FileError) -> Self {
            kind.into()
        }
    }

    impl From<FileErrorKind> for std::io::Error {
        fn from(value: FileErrorKind) -> Self {
            let kind = match value {
                #[cfg(feature = "put_file")]
                FileErrorKind::FileRead(e) => return e,
                FileErrorKind::InvalidContentLength(e) => {
                    Self::new(ErrorKind::InvalidData, "invalid content length")
                }
                FileErrorKind::InvalidContentType(_) => {
                    Self::new(ErrorKind::InvalidData, "invalid content type")
                }
                FileErrorKind::Build(e) => return e.into(),
                FileErrorKind::Reqwest(e) => reqwest_to_io(e),
                FileErrorKind::EtagNotFound => Self::new(ErrorKind::Interrupted, "etag not found"),
                FileErrorKind::InvalidEtag(_) => Self::new(ErrorKind::Interrupted, "invalid etag"),
                FileErrorKind::NotFoundCanonicalizedResource => {
                    Self::new(ErrorKind::InvalidData, "not found canonicalized resource")
                }
            };
            kind
        }
    }
}

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

impl<Item: Send + Sync> AlignBuilder for ObjectList<ArcPointer, Item> {
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

#[cfg(feature = "blocking")]
pub use blocking::Files as BlockingFiles;

use self::error_impl::FileErrorKind;

/// 同步的文件模块
#[cfg(feature = "blocking")]
pub mod blocking {

    use super::{
        error_impl::FileErrorKind, header_from_content_length, FileError, GetStdWithPath, ETAG,
        RANGE,
    };
    use crate::{
        blocking::builder::RequestBuilder,
        bucket::Bucket,
        builder::{BuilderError, RcPointer},
        object::ObjectList,
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
        const DEFAULT_CONTENT_TYPE: &'static str = super::DEFAULT_CONTENT_TYPE;

        /// # 上传文件到 OSS
        ///
        /// 需指定文件的路径
        #[cfg(feature = "put_file")]
        fn put_file<P: Into<std::path::PathBuf> + std::convert::AsRef<std::path::Path>>(
            &self,
            file_name: P,
            path: Path,
        ) -> Result<String, FileError> {
            let file_content = std::fs::read(file_name).map_err(|e| FileError {
                kind: FileErrorKind::FileRead(e),
            })?;

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
        /// use crate::aliyun_oss_client::file::BlockingFiles;
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
                .get(ETAG)
                .ok_or(FileError {
                    kind: FileErrorKind::EtagNotFound,
                })?
                .to_str()
                .map_err(|e| FileError {
                    kind: FileErrorKind::InvalidEtag(e),
                })?;

            Ok(result.to_string())
        }

        /// 最原始的上传文件的方法
        fn put_content_base(
            &self,
            content: Vec<u8>,
            content_type: &str,
            path: Path,
        ) -> Result<Response, FileError> {
            let (url, canonicalized) = self.get_std_with_path(path).ok_or(FileError {
                kind: FileErrorKind::NotFoundCanonicalizedResource,
            })?;

            let content_length = content.len().to_string();
            let headers = vec![
                (CONTENT_LENGTH, header_from_content_length(&content_length)?),
                (
                    CONTENT_TYPE,
                    content_type.parse().map_err(|e| FileError {
                        kind: FileErrorKind::InvalidContentType(e),
                    })?,
                ),
            ];

            let response = self
                .builder_with_header(Method::PUT, url, canonicalized, headers)?
                .body(content);

            response.send_adjust_error().map_err(FileError::from)
        }

        /// # 获取文件内容
        fn get_object<Num, R>(&self, path: Path, range: R) -> Result<Vec<u8>, FileError>
        where
            R: Into<ContentRange<Num>>,
            ContentRange<Num>: Into<HeaderValue>,
        {
            let (url, canonicalized) = self.get_std_with_path(path).ok_or(FileError {
                kind: FileErrorKind::NotFoundCanonicalizedResource,
            })?;

            let headers: Vec<(_, HeaderValue)> = vec![(
                {
                    #[allow(clippy::unwrap_used)]
                    RANGE.parse().unwrap()
                },
                range.into().into(),
            )];

            Ok(self
                .builder_with_header(Method::GET, url, canonicalized, headers)?
                .send_adjust_error()?
                .text()?
                .into_bytes())
        }

        /// # 删除 OSS 上的文件
        fn delete_object(&self, path: Path) -> Result<(), FileError> {
            let (url, canonicalized) = self.get_std_with_path(path).ok_or(FileError {
                kind: FileErrorKind::NotFoundCanonicalizedResource,
            })?;

            self.builder(Method::DELETE, url, canonicalized)?
                .send_adjust_error()?;

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
