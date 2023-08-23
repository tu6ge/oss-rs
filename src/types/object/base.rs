//! ObjectBase 定义

use oss_derive::oss_gen_rc;
use url::Url;

use super::{
    super::{CanonicalizedResource, InvalidBucketName},
    InvalidObjectPath, ObjectPath, SetObjectPath,
};
use crate::{
    auth::query::QueryAuth,
    builder::{ArcPointer, PointerFamily},
    types::core::IntoQuery,
    EndPoint, KeyId, KeySecret,
};
use crate::{config::BucketBase, BucketName};

#[cfg(feature = "blocking")]
use crate::builder::RcPointer;
#[cfg(feature = "blocking")]
use std::rc::Rc;
use std::sync::Arc;

/// # Object 元信息
/// 包含所属 bucket endpoint 以及文件路径
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ObjectBase<PointerSel: PointerFamily = ArcPointer> {
    pub(super) bucket: PointerSel::Bucket,
    pub(crate) path: ObjectPath,
}

impl<T: PointerFamily> Default for ObjectBase<T> {
    fn default() -> Self {
        Self {
            bucket: T::Bucket::default(),
            path: ObjectPath::default(),
        }
    }
}

impl<T: PointerFamily> AsRef<ObjectPath> for ObjectBase<T> {
    fn as_ref(&self) -> &ObjectPath {
        &self.path
    }
}

impl<T: PointerFamily> ObjectBase<T> {
    /// 初始化 Object 元信息
    #[cfg(test)]
    pub(crate) fn new<P>(bucket: T::Bucket, path: P) -> Result<Self, InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        P::Error: Into<InvalidObjectPath>,
    {
        let path = path.try_into().map_err(|e| e.into())?;

        Ok(Self { bucket, path })
    }

    #[inline]
    pub(crate) fn new2(bucket: T::Bucket, path: ObjectPath) -> Self {
        Self { bucket, path }
    }

    /// 为 Object 元信息设置 bucket
    pub fn set_bucket(&mut self, bucket: T::Bucket) {
        self.bucket = bucket;
    }

    pub(crate) fn init_with_bucket(bucket: T::Bucket) -> Self {
        Self {
            bucket,
            ..Default::default()
        }
    }

    /// 为 Object 元信息设置文件路径
    pub fn set_path<P>(&mut self, path: P) -> Result<(), InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        P::Error: Into<InvalidObjectPath>,
    {
        self.path = path.try_into().map_err(|e| e.into())?;

        Ok(())
    }

    /// 返回 Object 元信息的文件路径
    pub fn path(&self) -> ObjectPath {
        self.path.to_owned()
    }
}

#[oss_gen_rc]
impl ObjectBase<ArcPointer> {
    #[doc(hidden)]
    #[inline]
    pub fn from_bucket<P>(bucket: BucketBase, path: P) -> Result<Self, InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        P::Error: Into<InvalidObjectPath>,
    {
        Ok(Self {
            bucket: Arc::new(bucket),
            path: path.try_into().map_err(|e| e.into())?,
        })
    }

    #[doc(hidden)]
    #[inline]
    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn try_from_bucket<B>(bucket: B, path: &str) -> Result<Self, InvalidObjectBase>
    where
        B: TryInto<BucketBase>,
        B::Error: Into<InvalidObjectBase>,
    {
        Ok(Self {
            bucket: Arc::new(bucket.try_into().map_err(|e| e.into())?),
            path: path
                .parse()
                .map_err(|e| InvalidObjectBase::from_path(path, e))?,
        })
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_ref_bucket<P>(bucket: Arc<BucketBase>, path: P) -> Result<Self, InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        P::Error: Into<InvalidObjectPath>,
    {
        Ok(Self {
            bucket,
            path: path.try_into().map_err(|e| e.into())?,
        })
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn from_bucket_name(
        bucket: &str,
        endpoint: &str,
        path: &str,
    ) -> Result<Self, InvalidObjectBase> {
        let bucket = BucketBase::new(
            bucket
                .parse()
                .map_err(|e| InvalidObjectBase::from_bucket_name(bucket, e))?,
            endpoint
                .parse()
                .map_err(|e| InvalidObjectBase::from_endpoint(endpoint, e))?,
        );
        Ok(Self {
            bucket: Arc::new(bucket),
            path: path
                .parse()
                .map_err(|e| InvalidObjectBase::from_path(path, e))?,
        })
    }

    #[doc(hidden)]
    #[inline]
    pub fn bucket_name(&self) -> &BucketName {
        self.bucket.get_name()
    }

    /// 获取 EndPoint 引用
    pub fn endpoint(&self) -> &EndPoint {
        self.bucket.endpoint_ref()
    }

    /// 根据提供的查询参数信息，获取当前 object 对应的接口请求参数（ url 和 CanonicalizedResource）
    #[inline]
    pub fn get_url_resource<Q: IntoQuery>(&self, query: Q) -> (Url, CanonicalizedResource) {
        let mut url = self.bucket.to_url();
        url.set_object_path(&self.path);

        let resource =
            CanonicalizedResource::from_object((self.bucket.name(), self.path.as_ref()), query);

        (url, resource)
    }

    /// 带签名的 Url 链接
    pub fn to_sign_url(&self, key: &KeyId, secret: &KeySecret, expires: i64) -> Url {
        let auth = QueryAuth::new_with_bucket(key, secret, &self.bucket);
        auth.to_url(&self.path, expires)
    }
}

#[oss_gen_rc]
impl PartialEq<ObjectBase<ArcPointer>> for ObjectBase<ArcPointer> {
    #[inline]
    fn eq(&self, other: &ObjectBase<ArcPointer>) -> bool {
        *self.bucket == *other.bucket && self.path == other.path
    }
}

impl<T: PointerFamily> PartialEq<&str> for ObjectBase<T> {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectBase;
    /// # use aliyun_oss_client::config::BucketBase;
    /// # use aliyun_oss_client::builder::ArcPointer;
    /// # use std::sync::Arc;
    /// use aliyun_oss_client::types::BucketName;
    /// let mut path = ObjectBase::<ArcPointer>::default();
    /// path.set_path("abc");
    /// assert!(path == "abc");
    ///
    /// let mut bucket = BucketBase::default();
    /// bucket.set_name("def".parse::<BucketName>().unwrap());
    /// bucket.try_set_endpoint("shanghai").unwrap();
    /// path.set_bucket(Arc::new(bucket));
    /// assert!(path == "abc");
    /// ```
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.path == other
    }
}

pub use invalid::InvalidObjectBase;

/// 定义 InvalidObjectBase
pub mod invalid {
    use std::{
        error::Error,
        fmt::{self, Display},
    };

    use crate::{
        config::InvalidBucketBase,
        types::{InvalidBucketName, InvalidEndPoint},
    };

    use super::InvalidObjectPath;

    /// Object 元信息的错误集
    #[derive(Debug)]
    #[non_exhaustive]
    pub struct InvalidObjectBase {
        pub(crate) source: String,
        pub(crate) kind: InvalidObjectBaseKind,
    }

    impl InvalidObjectBase {
        pub(super) fn from_bucket_name(name: &str, err: InvalidBucketName) -> Self {
            InvalidObjectBase {
                source: name.to_string(),
                kind: InvalidObjectBaseKind::BucketName(err),
            }
        }
        pub(super) fn from_endpoint(name: &str, err: InvalidEndPoint) -> Self {
            InvalidObjectBase {
                source: name.to_string(),
                kind: InvalidObjectBaseKind::EndPoint(err),
            }
        }

        pub(super) fn from_path(name: &str, err: InvalidObjectPath) -> Self {
            InvalidObjectBase {
                source: name.to_string(),
                kind: InvalidObjectBaseKind::Path(err),
            }
        }
    }

    /// Object 元信息的错误集
    #[derive(Debug)]
    #[non_exhaustive]
    pub(crate) enum InvalidObjectBaseKind {
        #[doc(hidden)]
        Bucket(InvalidBucketBase),
        #[doc(hidden)]
        BucketName(InvalidBucketName),
        #[doc(hidden)]
        EndPoint(InvalidEndPoint),
        #[doc(hidden)]
        Path(InvalidObjectPath),
        #[cfg(test)]
        Bar,
    }

    impl Display for InvalidObjectBase {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "get object base faild, source: {}", self.source)
        }
    }

    impl Error for InvalidObjectBase {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            use InvalidObjectBaseKind::*;
            match &self.kind {
                Bucket(b) => Some(b),
                BucketName(e) => Some(e),
                EndPoint(e) => Some(e),
                Path(p) => Some(p),
                #[cfg(test)]
                Bar => None,
            }
        }
    }

    impl From<InvalidBucketBase> for InvalidObjectBase {
        fn from(value: InvalidBucketBase) -> Self {
            Self {
                source: value.clone().source_string(),
                kind: InvalidObjectBaseKind::Bucket(value),
            }
        }
    }
}
