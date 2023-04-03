use std::{
    borrow::Cow,
    fmt::{self, Display},
    ops::{Add, AddAssign},
    path::Path,
    str::FromStr,
    sync::Arc,
};

use oss_derive::oss_gen_rc;
use reqwest::Url;
use thiserror::Error;

use crate::{
    builder::{ArcPointer, PointerFamily},
    config::{BucketBase, InvalidBucketBase},
    object::Object,
    BucketName, EndPoint, QueryKey, QueryValue,
};

use super::{CanonicalizedResource, InvalidBucketName, InvalidEndPoint};

/// # Object 元信息
/// 包含所属 bucket endpoint 以及文件路径
#[derive(Debug, Clone)]
pub struct ObjectBase<PointerSel: PointerFamily = ArcPointer> {
    bucket: PointerSel::Bucket,
    path: ObjectPath,
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
    pub fn new<P>(bucket: T::Bucket, path: P) -> Result<Self, InvalidObjectPath>
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
    pub fn try_from_bucket<B, P>(bucket: B, path: P) -> Result<Self, InvalidObjectBase>
    where
        B: TryInto<BucketBase>,
        P: TryInto<ObjectPath>,
        B::Error: Into<InvalidObjectBase>,
        P::Error: Into<InvalidObjectBase>,
    {
        Ok(Self {
            bucket: Arc::new(bucket.try_into().map_err(|e| e.into())?),
            path: path.try_into().map_err(|e| e.into())?,
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
    pub(crate) fn from_bucket_name<B, E, P>(
        bucket: B,
        endpoint: E,
        path: P,
    ) -> Result<Self, InvalidObjectBase>
    where
        B: TryInto<BucketName>,
        B::Error: Into<InvalidObjectBase>,
        E: TryInto<EndPoint>,
        E::Error: Into<InvalidObjectBase>,
        P: TryInto<ObjectPath>,
        P::Error: Into<InvalidObjectPath>,
    {
        let bucket = BucketBase::new(
            bucket.try_into().map_err(|e| e.into())?,
            endpoint.try_into().map_err(|e| e.into())?,
        );
        Self::from_bucket(bucket, path).map_err(|e| e.into())
    }

    #[doc(hidden)]
    #[inline]
    pub fn bucket_name(&self) -> &BucketName {
        self.bucket.get_name()
    }

    /// 根据提供的查询参数信息，获取当前 object 对应的接口请求参数（ url 和 CanonicalizedResource）
    #[inline]
    pub fn get_url_resource<Q: IntoIterator<Item = (QueryKey, QueryValue)>>(
        &self,
        query: Q,
    ) -> (Url, CanonicalizedResource) {
        let mut url = self.bucket.to_url();
        url.set_object_path(&self.path);

        let resource =
            CanonicalizedResource::from_object((self.bucket.name(), self.path.as_ref()), query);

        (url, resource)
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

/// Object 元信息的错误集
#[derive(Debug)]
pub enum InvalidObjectBase {
    #[doc(hidden)]
    Bucket(InvalidBucketBase),
    #[doc(hidden)]
    Path(InvalidObjectPath),
}

impl Display for InvalidObjectBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use InvalidObjectBase::*;
        match self {
            Bucket(b) => write!(f, "{}", b),
            Path(p) => write!(f, "{}", p),
        }
    }
}

impl From<InvalidBucketBase> for InvalidObjectBase {
    fn from(value: InvalidBucketBase) -> Self {
        Self::Bucket(value)
    }
}

impl From<InvalidObjectPath> for InvalidObjectBase {
    fn from(value: InvalidObjectPath) -> Self {
        Self::Path(value)
    }
}

impl From<InvalidBucketName> for InvalidObjectBase {
    fn from(value: InvalidBucketName) -> Self {
        Self::Bucket(value.into())
    }
}

impl From<InvalidEndPoint> for InvalidObjectBase {
    fn from(value: InvalidEndPoint) -> Self {
        Self::Bucket(value.into())
    }
}

/// OSS Object 存储对象的路径
/// 不带前缀 `/`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectPathInner<'a>(Cow<'a, str>);

/// OSS Object 存储对象的路径
/// 不带前缀 `/`
pub type ObjectPath = ObjectPathInner<'static>;

impl AsRef<str> for ObjectPathInner<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ObjectPathInner<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ObjectPathInner<'_> {
    /// 默认值
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectPath;
    /// let path = ObjectPath::default();
    /// assert!(path == "");
    /// ```
    fn default() -> Self {
        Self(Cow::Borrowed(""))
    }
}

impl PartialEq<&str> for ObjectPathInner<'_> {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectPath;
    /// let path = ObjectPath::new("abc").unwrap();
    /// assert!(path == "abc");
    /// ```
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.0 == other
    }
}

impl PartialEq<ObjectPathInner<'_>> for &str {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectPath;
    /// let path = ObjectPath::new("abc").unwrap();
    /// assert!("abc" == path);
    /// ```
    #[inline]
    fn eq(&self, other: &ObjectPathInner) -> bool {
        self == &other.0
    }
}

impl PartialEq<String> for ObjectPathInner<'_> {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectPath;
    /// let path = ObjectPath::new("abc").unwrap();
    /// assert!(path == "abc".to_string());
    /// ```
    #[inline]
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

impl PartialEq<ObjectPathInner<'_>> for String {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectPath;
    /// let path = ObjectPath::new("abc").unwrap();
    /// assert!("abc".to_string() == path);
    /// ```
    #[inline]
    fn eq(&self, other: &ObjectPathInner) -> bool {
        self == &other.0
    }
}

impl<'a> ObjectPathInner<'a> {
    /// Creates a new `ObjectPath` from the given string.
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectPath;
    /// assert!(ObjectPath::new("abc.jpg").is_ok());
    /// assert!(ObjectPath::new("abc/def.jpg").is_ok());
    /// assert!(ObjectPath::new("/").is_err());
    /// assert!(ObjectPath::new("/abc").is_err());
    /// assert!(ObjectPath::new("abc/").is_err());
    /// assert!(ObjectPath::new(".abc").is_err());
    /// assert!(ObjectPath::new("../abc").is_err());
    /// assert!(ObjectPath::new(r"aaa\abc").is_err());
    /// ```
    pub fn new(val: impl Into<Cow<'a, str>>) -> Result<Self, InvalidObjectPath> {
        let val = val.into();
        if val.starts_with('/') || val.starts_with('.') || val.ends_with('/') {
            return Err(InvalidObjectPath);
        }
        if !val.chars().all(|c| c != '\\') {
            return Err(InvalidObjectPath);
        }
        Ok(Self(val))
    }

    /// # Safety
    ///
    /// Const function that creates a new `ObjectPath` from a static str.
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectPath;
    /// let path = unsafe { ObjectPath::from_static("abc") };
    /// assert!(path == "abc");
    /// ```
    pub const unsafe fn from_static(secret: &'static str) -> Self {
        Self(Cow::Borrowed(secret))
    }
}

impl TryFrom<String> for ObjectPathInner<'_> {
    type Error = InvalidObjectPath;
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectPath;
    /// let path: ObjectPath = String::from("abc").try_into().unwrap();
    /// assert!(path == "abc");
    /// ```
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Self::new(val)
    }
}

impl<'a: 'b, 'b> TryFrom<&'a str> for ObjectPathInner<'b> {
    type Error = InvalidObjectPath;
    fn try_from(val: &'a str) -> Result<Self, Self::Error> {
        Self::new(val)
    }
}

impl FromStr for ObjectPathInner<'_> {
    type Err = InvalidObjectPath;
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectPath;
    /// use std::str::FromStr;
    /// let path: ObjectPath = "img1.jpg".parse().unwrap();
    /// assert!(path == "img1.jpg");
    /// assert!(ObjectPath::from_str("abc.jpg").is_ok());
    /// assert!(ObjectPath::from_str("abc/def.jpg").is_ok());
    /// assert!(ObjectPath::from_str("/").is_err());
    /// assert!(ObjectPath::from_str("/abc").is_err());
    /// assert!(ObjectPath::from_str("abc/").is_err());
    /// assert!(ObjectPath::from_str(".abc").is_err());
    /// assert!(ObjectPath::from_str("../abc").is_err());
    /// assert!(ObjectPath::from_str(r"aaa\abc").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('/') || s.starts_with('.') || s.ends_with('/') {
            return Err(InvalidObjectPath);
        }

        if !s.chars().all(|c| c != '\\') {
            return Err(InvalidObjectPath);
        }
        Ok(Self(Cow::Owned(s.to_owned())))
    }
}

impl TryFrom<&Path> for ObjectPathInner<'_> {
    type Error = InvalidObjectPath;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let val = value.to_str().ok_or(InvalidObjectPath)?;
        if std::path::MAIN_SEPARATOR != '/' {
            val.replace(std::path::MAIN_SEPARATOR, "/").parse()
        } else {
            val.parse()
        }
    }
}

impl<T: PointerFamily> From<Object<T>> for ObjectPathInner<'static> {
    #[inline]
    fn from(obj: Object<T>) -> Self {
        obj.base.path
    }
}

/// 不合法的文件路径
#[derive(Debug, Error)]
pub struct InvalidObjectPath;

impl Display for InvalidObjectPath {
    /// ```
    /// # use aliyun_oss_client::types::object::InvalidObjectPath;
    /// assert_eq!(format!("{}", InvalidObjectPath), "invalid object path");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid object path")
    }
}

/// 将 object 的路径拼接到 Url 上去
pub trait UrlObjectPath {
    /// 为 Url 添加方法
    fn set_object_path(&mut self, path: &ObjectPathInner);
}

impl UrlObjectPath for Url {
    fn set_object_path(&mut self, path: &ObjectPathInner) {
        self.set_path(path.as_ref());
    }
}

/// OSS Object 对象路径的前缀目录
/// 不带前缀 `/`, 且必须以 `/` 结尾
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectDir<'a>(Cow<'a, str>);

impl AsRef<str> for ObjectDir<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsMut<String> for ObjectDir<'_> {
    fn as_mut(&mut self) -> &mut String {
        self.0.to_mut()
    }
}

impl fmt::Display for ObjectDir<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// impl Default for ObjectDir<'_> {
//     /// 默认值
//     /// ```
//     /// # use aliyun_oss_client::types::object::ObjectDir;
//     /// let path = ObjectDir::default();
//     /// assert!(path == "default/");
//     /// ```
//     fn default() -> Self {
//         Self(Cow::Borrowed("default/"))
//     }
// }

impl PartialEq<&str> for ObjectDir<'_> {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectDir;
    /// let path = ObjectDir::new("abc/").unwrap();
    /// assert!(path == "abc/");
    /// ```
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.0 == other
    }
}

impl PartialEq<ObjectDir<'_>> for &str {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectDir;
    /// let path = ObjectDir::new("abc/").unwrap();
    /// assert!("abc/" == path);
    /// ```
    #[inline]
    fn eq(&self, other: &ObjectDir<'_>) -> bool {
        self == &other.0
    }
}

impl PartialEq<String> for ObjectDir<'_> {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectDir;
    /// let path = ObjectDir::new("abc/").unwrap();
    /// assert!(path == "abc/".to_string());
    /// ```
    #[inline]
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

impl PartialEq<ObjectDir<'_>> for String {
    /// 相等比较
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectDir;
    /// let path = ObjectDir::new("abc/").unwrap();
    /// assert!("abc/".to_string() == path);
    /// ```
    #[inline]
    fn eq(&self, other: &ObjectDir) -> bool {
        self == &other.0
    }
}

impl<'dir1, 'dir2: 'dir1> Add<ObjectDir<'dir2>> for ObjectDir<'dir1> {
    type Output = ObjectDir<'dir1>;

    /// # 支持 ObjectDir 相加运算
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectDir;
    /// let dir1 = ObjectDir::new("dir1/").unwrap();
    /// let dir2 = ObjectDir::new("dir2/").unwrap();
    /// let full_dir = ObjectDir::new("dir1/dir2/").unwrap();
    ///
    /// assert_eq!(dir1 + dir2, full_dir);
    /// ```
    fn add(self, rhs: ObjectDir<'dir2>) -> Self::Output {
        let mut string = self.0;

        string += rhs.0;
        ObjectDir(string)
    }
}

impl<'dir1, 'dir2: 'dir1> AddAssign<ObjectDir<'dir2>> for ObjectDir<'dir1> {
    /// # 支持 ObjectDir 相加运算
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectDir;
    /// let mut dir1 = ObjectDir::new("dir1/").unwrap();
    /// let dir2 = ObjectDir::new("dir2/").unwrap();
    /// let full_dir = ObjectDir::new("dir1/dir2/").unwrap();
    ///
    /// dir1 += dir2;
    /// assert_eq!(dir1, full_dir);
    /// ```
    fn add_assign(&mut self, rhs: ObjectDir<'dir2>) {
        *self.as_mut() += rhs.as_ref();
    }
}

impl<'file, 'dir: 'file> Add<ObjectPathInner<'file>> for ObjectDir<'dir> {
    type Output = ObjectPathInner<'file>;

    /// # 支持 ObjectDir 与 ObjectPath 相加运算
    /// ```
    /// # use aliyun_oss_client::types::object::{ObjectDir, ObjectPath};
    /// let dir1 = ObjectDir::new("dir1/").unwrap();
    /// let file1 = ObjectPath::new("img1.png").unwrap();
    /// let full_file = ObjectPath::new("dir1/img1.png").unwrap();
    ///
    /// assert_eq!(dir1 + file1, full_file);
    /// ```
    fn add(self, rhs: ObjectPathInner<'file>) -> Self::Output {
        let mut string = self.0;

        string += rhs.0;
        ObjectPathInner(string)
    }
}

impl<'a> ObjectDir<'a> {
    /// Creates a new `ObjectPath` from the given string.
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectDir;
    /// assert!(ObjectDir::new("abc/").is_ok());
    /// assert!(ObjectDir::new("abc/def/").is_ok());
    /// assert!(ObjectDir::new("/").is_err());
    /// assert!(ObjectDir::new("/abc/").is_err());
    /// assert!(ObjectDir::new(".abc/").is_err());
    /// assert!(ObjectDir::new("../abc/").is_err());
    /// assert!(ObjectDir::new(r"aaa\abc/").is_err());
    /// ```
    pub fn new<'b: 'a>(val: impl Into<Cow<'b, str>>) -> Result<Self, InvalidObjectDir> {
        let val = val.into();
        if val.starts_with('/') || val.starts_with('.') || !val.ends_with('/') {
            return Err(InvalidObjectDir);
        }
        if !val.chars().all(|c| c != '\\') {
            return Err(InvalidObjectDir);
        }
        Ok(Self(val))
    }

    /// # Safety
    ///
    /// Const function that creates a new `ObjectPath` from a static str.
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectDir;
    /// let path = unsafe { ObjectDir::from_static("abc/") };
    /// assert!(path == "abc/");
    /// ```
    pub const unsafe fn from_static(secret: &'a str) -> Self {
        Self(Cow::Borrowed(secret))
    }
}

impl TryFrom<String> for ObjectDir<'_> {
    type Error = InvalidObjectDir;
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectDir;
    /// let path: ObjectDir = String::from("abc/").try_into().unwrap();
    /// assert!(path == "abc/");
    /// ```
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Self::new(val)
    }
}

impl<'a: 'b, 'b> TryFrom<&'a str> for ObjectDir<'b> {
    type Error = InvalidObjectDir;
    fn try_from(val: &'a str) -> Result<Self, Self::Error> {
        Self::new(val)
    }
}

impl FromStr for ObjectDir<'_> {
    type Err = InvalidObjectDir;
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectDir;
    /// use std::str::FromStr;
    /// let path: ObjectDir = "path1/".parse().unwrap();
    /// assert!(path == "path1/");
    /// assert!(ObjectDir::from_str("abc/").is_ok());
    /// assert!(ObjectDir::from_str("abc/def/").is_ok());
    /// assert!(ObjectDir::from_str("/").is_err());
    /// assert!(ObjectDir::from_str("/abc/").is_err());
    /// assert!(ObjectDir::from_str(".abc/").is_err());
    /// assert!(ObjectDir::from_str("../abc/").is_err());
    /// assert!(ObjectDir::from_str(r"aaa\abc/").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('/') || s.starts_with('.') || !s.ends_with('/') {
            return Err(InvalidObjectDir);
        }

        if !s.chars().all(|c| c != '\\') {
            return Err(InvalidObjectDir);
        }
        Ok(Self(Cow::Owned(s.to_owned())))
    }
}

impl TryFrom<&Path> for ObjectDir<'_> {
    type Error = InvalidObjectDir;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let val = value.to_str().ok_or(InvalidObjectDir)?;
        if std::path::MAIN_SEPARATOR != '/' {
            val.replace(std::path::MAIN_SEPARATOR, "/").parse()
        } else {
            val.parse()
        }
    }
}

/// 不合法的文件目录路径
#[derive(Debug, Error)]
pub struct InvalidObjectDir;

impl Display for InvalidObjectDir {
    /// ```
    /// # use aliyun_oss_client::types::object::InvalidObjectDir;
    /// assert_eq!(
    ///     format!("{}", InvalidObjectDir),
    ///     "ObjectDir must end with `/`"
    /// );
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectDir must end with `/`")
    }
}

/// 给 Url 设置一个初始化方法，根据 OSS 的配置信息，返回文件的完整 OSS Url
pub trait OssFullUrl {
    /// 根据配置信息，计算完整的 Url
    fn from_oss(endpoint: &EndPoint, bucket: &BucketName, path: &ObjectPath) -> Self;
}

impl OssFullUrl for Url {
    fn from_oss(endpoint: &EndPoint, bucket: &BucketName, path: &ObjectPath) -> Self {
        let mut end_url = endpoint.to_url();

        let host = end_url.host_str();

        let mut name_str = bucket.to_string() + ".";

        let new_host = host.map(|h| {
            name_str.push_str(h);
            &*name_str
        });
        // 因为 endpoint 都是已知字符组成，bucket 也有格式要求，所以 unwrap 是安全的
        end_url
            .set_host(new_host)
            .unwrap_or_else(|_| panic!("set host failed: host: {}", new_host.unwrap_or("none")));

        end_url.set_object_path(path);

        end_url
    }
}

/// 文件夹下的子文件夹名，子文件夹下递归的所有文件和文件夹不包含在这里。
pub type CommonPrefixes = Vec<ObjectDir<'static>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_from_ref_bucket() {
        use std::env::set_var;
        set_var("ALIYUN_ENDPOINT", "qingdao");
        set_var("ALIYUN_BUCKET", "foo1");
        let object = ObjectBase::<ArcPointer>::from_ref_bucket(
            Arc::new(BucketBase::from_env().unwrap()),
            "img1.jpg",
        )
        .unwrap();

        assert_eq!(object.path(), "img1.jpg");
    }

    #[test]
    fn object_from_bucket_name() {
        let object =
            ObjectBase::<ArcPointer>::from_bucket_name("foo1", "qingdao", "img1.jpg").unwrap();

        assert_eq!(object.path(), "img1.jpg");
    }
}

#[cfg(feature = "blocking")]
#[cfg(test)]
mod blocking_tests {
    use crate::builder::RcPointer;

    use super::ObjectBase;

    fn crate_object_base(bucket: &'static str, path: &'static str) -> ObjectBase<RcPointer> {
        use std::rc::Rc;

        let bucket = bucket.parse().unwrap();

        let object = ObjectBase::<RcPointer>::new2(Rc::new(bucket), path.try_into().unwrap());
        object
    }

    #[test]
    fn test_get_object_info() {
        let object = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "bar");

        assert_eq!(object.bucket_name(), &"abc");
        assert_eq!(object.path(), "bar");
    }

    #[test]
    fn test_object_base_eq() {
        let object1 = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "bar");
        let object2 = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "bar");
        let object3 = crate_object_base("abc.oss-cn-qingdao.aliyuncs.com", "bar");
        let object4 = crate_object_base("abc.oss-cn-shanghai.aliyuncs.com", "ba2");
        assert!(object1 == object2);
        assert!(object1 != object3);
        assert!(object1 != object4);
    }
}
