//! object 相关的类型

use url::Url;

use std::{
    borrow::Cow,
    convert::Infallible,
    fmt::{self, Debug, Display},
    ops::{Add, AddAssign},
    path::Path,
    str::FromStr,
};

use crate::{BucketName, EndPoint};

#[cfg(test)]
mod test;

#[cfg(feature = "core")]
pub mod base;
#[cfg(feature = "core")]
pub use base::{InvalidObjectBase, ObjectBase};

/// OSS Object 存储对象的路径
/// 不带前缀 `/`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
            return Err(InvalidObjectPath::new());
        }
        if !val.chars().all(|c| c != '\\') {
            return Err(InvalidObjectPath::new());
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

impl TryFrom<&String> for ObjectPathInner<'_> {
    type Error = InvalidObjectPath;
    /// ```
    /// # use aliyun_oss_client::types::object::ObjectPath;
    /// let str = String::from("abc");
    /// let path: ObjectPath = &str.try_into().unwrap();
    /// assert!(path == "abc");
    /// ```
    fn try_from(val: &String) -> Result<Self, Self::Error> {
        Self::new(val.to_owned())
    }
}

impl TryFrom<Box<String>> for ObjectPathInner<'_> {
    type Error = InvalidObjectPath;
    fn try_from(val: Box<String>) -> Result<Self, Self::Error> {
        Self::new(*val)
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
            return Err(InvalidObjectPath::new());
        }

        if !s.chars().all(|c| c != '\\') {
            return Err(InvalidObjectPath::new());
        }
        Ok(Self(Cow::Owned(s.to_owned())))
    }
}

impl TryFrom<&[u8]> for ObjectPathInner<'_> {
    type Error = InvalidObjectPath;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        use std::str;

        str::from_utf8(value)
            .map_err(|_| InvalidObjectPath::new())
            .and_then(str::parse)
    }
}

impl TryFrom<&Path> for ObjectPathInner<'_> {
    type Error = InvalidObjectPath;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let val = value.to_str().ok_or(InvalidObjectPath::new())?;
        if std::path::MAIN_SEPARATOR != '/' {
            val.replace(std::path::MAIN_SEPARATOR, "/").parse()
        } else {
            val.parse()
        }
    }
}

/// 不合法的文件路径
pub struct InvalidObjectPath {
    _priv: (),
}

impl InvalidObjectPath {
    pub(crate) fn new() -> Self {
        Self { _priv: () }
    }
}

impl Display for InvalidObjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid object path")
    }
}

impl Debug for InvalidObjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InvalidObjectPath").finish()
    }
}

impl std::error::Error for InvalidObjectPath {}

impl From<Infallible> for InvalidObjectPath {
    fn from(_: Infallible) -> Self {
        Self { _priv: () }
    }
}

/// 将 object 的路径拼接到 Url 上去
pub trait SetObjectPath: private::Sealed {
    /// 为 Url 添加方法
    fn set_object_path(&mut self, path: &ObjectPathInner);
}

mod private {
    pub trait Sealed {}
}

impl private::Sealed for Url {}

impl SetObjectPath for Url {
    fn set_object_path(&mut self, path: &ObjectPathInner) {
        self.set_path(path.as_ref());
    }
}

/// OSS Object 对象路径的前缀目录
/// 不带前缀 `/`, 且必须以 `/` 结尾
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
            return Err(InvalidObjectDir::new());
        }
        if !val.chars().all(|c| c != '\\') {
            return Err(InvalidObjectDir::new());
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
            return Err(InvalidObjectDir::new());
        }

        if !s.chars().all(|c| c != '\\') {
            return Err(InvalidObjectDir::new());
        }
        Ok(Self(Cow::Owned(s.to_owned())))
    }
}

impl TryFrom<&Path> for ObjectDir<'_> {
    type Error = InvalidObjectDir;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let val = value.to_str().ok_or(InvalidObjectDir::new())?;
        if std::path::MAIN_SEPARATOR != '/' {
            val.replace(std::path::MAIN_SEPARATOR, "/").parse()
        } else {
            val.parse()
        }
    }
}

/// 不合法的文件目录路径
pub struct InvalidObjectDir {
    _priv: (),
}

impl InvalidObjectDir {
    pub(crate) fn new() -> InvalidObjectDir {
        InvalidObjectDir { _priv: () }
    }
}

impl Display for InvalidObjectDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "object-dir must end with `/`, and not start with `/`,`.`"
        )
    }
}

impl Debug for InvalidObjectDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InvalidObjectDir").finish()
    }
}

impl std::error::Error for InvalidObjectDir {}

/// 根据 OSS 的配置信息初始化外部类型
pub trait FromOss {
    /// 根据配置信息，计算外部类型的具体实现
    fn from_oss(endpoint: &EndPoint, bucket: &BucketName, path: &ObjectPath) -> Self;
}

/// 给 Url 设置一个初始化方法，根据 OSS 的配置信息，返回文件的完整 OSS Url
impl FromOss for Url {
    /// 根据配置信息，计算完整的 Url
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
