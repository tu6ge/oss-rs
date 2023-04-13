//! 核心功能用到的类型 Query ContentRange 等

use std::borrow::Cow;
use std::collections::HashMap;

//===================================================================================================
/// 查询条件
///
/// ```
/// use aliyun_oss_client::types::Query;
///
/// let query: Query = [("abc", "def")].into_iter().collect();
/// assert_eq!(query.len(), 1);
///
/// let value = query.get("abc");
/// assert!(value.is_some());
/// let value = value.unwrap();
/// assert_eq!(value.as_ref(), "def");
///
/// let str = query.to_oss_string();
/// assert_eq!(str.as_str(), "list-type=2&abc=def");
/// let str = query.to_url_query();
/// assert_eq!(str.as_str(), "abc=def");
/// ```
#[derive(Clone, Debug, Default)]
pub struct Query {
    inner: HashMap<QueryKey, QueryValue>,
}

impl AsMut<HashMap<QueryKey, QueryValue>> for Query {
    fn as_mut(&mut self) -> &mut HashMap<QueryKey, QueryValue> {
        &mut self.inner
    }
}

impl AsRef<HashMap<QueryKey, QueryValue>> for Query {
    fn as_ref(&self) -> &HashMap<QueryKey, QueryValue> {
        &self.inner
    }
}

impl Query {
    /// Creates an empty `Query`.
    ///
    /// The hash map is initially created with a capacity of 0, so it will not allocate until it
    /// is first inserted into.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Creates an empty `Query` with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Inserts a key-value pair into the map.
    pub fn insert(
        &mut self,
        key: impl Into<QueryKey>,
        value: impl Into<QueryValue>,
    ) -> Option<QueryValue> {
        self.as_mut().insert(key.into(), value.into())
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: impl Into<QueryKey>) -> Option<&QueryValue> {
        self.as_ref().get(&key.into())
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    pub fn remove(&mut self, key: impl Into<QueryKey>) -> Option<QueryValue> {
        self.as_mut().remove(&key.into())
    }

    /// 将查询参数拼成 aliyun 接口需要的格式
    pub fn to_oss_string(&self) -> String {
        let mut query_str = String::from("list-type=2");
        for (key, value) in self.as_ref().iter() {
            query_str += "&";
            query_str += key.as_ref();
            query_str += "=";
            query_str += value.as_ref();
        }
        query_str
    }

    /// 转化成 url 参数的形式
    /// a=foo&b=bar
    pub fn to_url_query(&self) -> String {
        self.as_ref()
            .iter()
            .map(|(k, v)| {
                let mut res = String::with_capacity(k.as_ref().len() + v.as_ref().len() + 1);
                res.push_str(k.as_ref());
                res.push('=');
                res.push_str(v.as_ref());
                res
            })
            .collect::<Vec<_>>()
            .join("&")
    }
}

impl Index<QueryKey> for Query {
    type Output = QueryValue;

    fn index(&self, index: QueryKey) -> &Self::Output {
        self.get(index).expect("no found query key")
    }
}

impl IntoIterator for Query {
    type Item = (QueryKey, QueryValue);
    type IntoIter = std::vec::IntoIter<Self::Item>;
    /// # 使用 Vec 转 Query
    /// 例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// # use aliyun_oss_client::QueryValue;
    /// # use assert_matches::assert_matches;
    /// let query = Query::from_iter(vec![("foo", "bar")]);
    /// let list: Vec<_> = query.into_iter().collect();
    /// assert_eq!(list.len(), 1);
    /// assert_matches!(&list[0].0, &QueryKey::Custom(_));
    /// let value: QueryValue = "bar".parse().unwrap();
    /// assert_eq!(list[0].1, value);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter().collect::<Vec<_>>().into_iter()
    }
}

impl FromIterator<(QueryKey, QueryValue)> for Query {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (QueryKey, QueryValue)>,
    {
        let mut map = Query::default();
        map.as_mut().extend(iter);
        map
    }
}

impl<'a> FromIterator<(&'a str, &'a str)> for Query {
    /// 转化例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query: Query = [("max-keys", "123")].into_iter().collect();
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let inner = iter.into_iter().map(|(k, v)| {
            (
                k.parse().expect("invalid QueryKey"),
                v.parse().expect("invalid QueryValue"),
            )
        });

        let mut map = Query::default();
        map.as_mut().extend(inner);
        map
    }
}

impl<'a> FromIterator<(Cow<'a, str>, Cow<'a, str>)> for Query {
    /// 转化例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query: Query = [("max-keys", "123")].into_iter().collect();
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Cow<'a, str>, Cow<'a, str>)>,
    {
        let inner = iter.into_iter().map(|(k, v)| {
            (
                k.as_ref().parse().expect("invalid QueryKey"),
                v.as_ref().parse().expect("invalid QueryValue"),
            )
        });

        let mut map = Query::default();
        map.as_mut().extend(inner);
        map
    }
}

impl<'a> FromIterator<(&'a str, u8)> for Query {
    /// 转化例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query = Query::from_iter([("max-keys", 123u8)]);
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, u8)>,
    {
        let inner = iter
            .into_iter()
            .map(|(k, v)| (k.parse().expect("invalid QueryKey"), v.into()));

        let mut map = Query::default();
        map.as_mut().extend(inner);
        map
    }
}

impl<'a> FromIterator<(&'a str, u16)> for Query {
    /// 转化例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query = Query::from_iter([("max-keys", 123u16)]);
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, u16)>,
    {
        let inner = iter
            .into_iter()
            .map(|(k, v)| (k.parse().expect("invalid QueryKey"), v.into()));

        let mut map = Query::default();
        map.as_mut().extend(inner);
        map
    }
}

impl<'a> FromIterator<(QueryKey, &'a str)> for Query {
    /// 转化例子
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query = Query::from_iter([(QueryKey::MaxKeys, "123")]);
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
    /// assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    /// ```
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (QueryKey, &'a str)>,
    {
        let inner = iter
            .into_iter()
            .map(|(k, v)| (k, v.parse().expect("invalid QueryValue")));

        let mut map = Query::default();
        map.as_mut().extend(inner);
        map
    }
}

macro_rules! impl_from_iter {
    ($key:ty, $val:ty, $convert:expr) => {
        impl FromIterator<($key, $val)> for Query {
            fn from_iter<I>(iter: I) -> Self
            where
                I: IntoIterator<Item = ($key, $val)>,
            {
                let inner = iter.into_iter().map($convert);

                let mut map = Query::default();
                map.as_mut().extend(inner);
                map
            }
        }
    };
}

impl_from_iter!(QueryKey, u8, |(k, v)| (k, v.into()));
impl_from_iter!(QueryKey, u16, |(k, v)| (k, v.into()));

#[cfg(test)]
mod tests_query_from_iter {
    use super::*;
    #[test]
    fn test() {
        let query = Query::from_iter([(QueryKey::MaxKeys, 123u8)]);
        assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
        assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));

        let query = Query::from_iter([(QueryKey::MaxKeys, 123u16)]);
        assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u8.into()));
        assert_eq!(query.get(QueryKey::MaxKeys), Some(&123u16.into()));
    }
}

impl PartialEq<Query> for Query {
    fn eq(&self, other: &Query) -> bool {
        self.as_ref() == other.as_ref()
    }
}

/// 为 Url 拼接 [`Query`] 数据
/// [`Query`]: crate::types::Query
pub trait UrlQuery {
    /// 给 Url 结构体增加 `set_search_query` 方法
    fn set_search_query(&mut self, query: &Query);
}

impl UrlQuery for Url {
    /// 将查询参数拼接到 API 的 Url 上
    ///
    /// # 例子
    /// ```
    /// use aliyun_oss_client::types::Query;
    /// use aliyun_oss_client::types::UrlQuery;
    /// use reqwest::Url;
    ///
    /// let query = Query::from_iter([("abc", "def")]);
    /// let mut url = Url::parse("https://exapmle.com").unwrap();
    /// url.set_search_query(&query);
    /// assert_eq!(url.as_str(), "https://exapmle.com/?list-type=2&abc=def");
    /// assert_eq!(url.query(), Some("list-type=2&abc=def"));
    /// ```
    fn set_search_query(&mut self, query: &Query) {
        self.set_query(Some(&query.to_oss_string()));
    }
}

/// 查询条件的键
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InnerQueryKey<'a> {
    /// 对Object名字进行分组的字符。所有Object名字包含指定的前缀，第一次出现delimiter字符之间的Object作为一组元素（即CommonPrefixes）
    /// 示例值 `/`
    Delimiter,

    /// 设定从start-after之后按字母排序开始返回Object。
    /// start-after用来实现分页显示效果，参数的长度必须小于1024字节。
    /// 做条件查询时，即使start-after在列表中不存在，也会从符合start-after字母排序的下一个开始打印。
    StartAfter,

    /// 指定List操作需要从此token开始。您可从ListObjectsV2（GetBucketV2）结果中的NextContinuationToken获取此token。
    /// 用于分页，返回下一页的数据
    ContinuationToken,

    /// 指定返回Object的最大数。
    /// 取值：大于0小于等于1000
    MaxKeys,

    /// # 限定返回文件的Key必须以prefix作为前缀。
    /// 如果把prefix设为某个文件夹名，则列举以此prefix开头的文件，即该文件夹下递归的所有文件和子文件夹。
    ///
    /// 在设置prefix的基础上，将delimiter设置为正斜线（/）时，返回值就只列举该文件夹下的文件，文件夹下的子文件夹名返回在CommonPrefixes中，
    /// 子文件夹下递归的所有文件和文件夹不显示。
    ///
    /// 例如，一个Bucket中有三个Object，分别为fun/test.jpg、fun/movie/001.avi和fun/movie/007.avi。如果设定prefix为fun/，
    /// 则返回三个Object；如果在prefix设置为fun/的基础上，将delimiter设置为正斜线（/），则返回fun/test.jpg和fun/movie/。
    /// ## 要求
    /// - 参数的长度必须小于1024字节。
    /// - 设置prefix参数时，不能以正斜线（/）开头。如果prefix参数置空，则默认列举Bucket内的所有Object。
    /// - 使用prefix查询时，返回的Key中仍会包含prefix。
    Prefix,

    /// 对返回的内容进行编码并指定编码的类型。
    EncodingType,

    /// 指定是否在返回结果中包含owner信息。
    FetchOwner,

    /// 自定义
    Custom(Cow<'a, str>),
}

/// 查询条件的键
pub type QueryKey = InnerQueryKey<'static>;

impl AsRef<str> for InnerQueryKey<'_> {
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// # use std::borrow::Cow;
    /// assert_eq!(QueryKey::Delimiter.as_ref(), "delimiter");
    /// assert_eq!(QueryKey::StartAfter.as_ref(), "start-after");
    /// assert_eq!(QueryKey::ContinuationToken.as_ref(), "continuation-token");
    /// assert_eq!(QueryKey::MaxKeys.as_ref(), "max-keys");
    /// assert_eq!(QueryKey::Prefix.as_ref(), "prefix");
    /// assert_eq!(QueryKey::EncodingType.as_ref(), "encoding-type");
    /// assert_eq!(QueryKey::Custom(Cow::Borrowed("abc")).as_ref(), "abc");
    /// ```
    fn as_ref(&self) -> &str {
        use InnerQueryKey::*;

        match self {
            Delimiter => "delimiter",
            StartAfter => "start-after",
            ContinuationToken => "continuation-token",
            MaxKeys => "max-keys",
            Prefix => "prefix",
            EncodingType => "encoding-type",
            // TODO
            FetchOwner => unimplemented!("parse xml not support fetch owner"),
            Custom(str) => str.as_ref(),
        }
    }
}

impl Display for InnerQueryKey<'_> {
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// assert_eq!(format!("{}", QueryKey::Delimiter), "delimiter");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl From<String> for InnerQueryKey<'_> {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}
impl<'a: 'b, 'b> From<&'a str> for InnerQueryKey<'b> {
    fn from(date: &'a str) -> Self {
        Self::new(date)
    }
}

impl FromStr for QueryKey {
    type Err = InvalidQueryKey;
    /// 示例
    /// ```
    /// # use aliyun_oss_client::types::QueryKey;
    /// let value: QueryKey = "abc".into();
    /// assert!(value == QueryKey::from_static("abc"));
    /// ```
    fn from_str(s: &str) -> Result<Self, InvalidQueryKey> {
        Ok(Self::from_static(s))
    }
}

/// 异常的查询条件键
#[derive(Debug)]
pub struct InvalidQueryKey {
    _priv: (),
}

impl Error for InvalidQueryKey {}

impl Display for InvalidQueryKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid query key")
    }
}

impl<'a> InnerQueryKey<'a> {
    /// # Examples
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// # use assert_matches::assert_matches;
    /// let key = QueryKey::new("delimiter");
    /// assert!(key == QueryKey::Delimiter);
    /// assert!(QueryKey::new("start-after") == QueryKey::StartAfter);
    /// assert!(QueryKey::new("continuation-token") == QueryKey::ContinuationToken);
    /// assert!(QueryKey::new("max-keys") == QueryKey::MaxKeys);
    /// assert!(QueryKey::new("prefix") == QueryKey::Prefix);
    /// assert!(QueryKey::new("encoding-type") == QueryKey::EncodingType);
    ///
    /// let key = QueryKey::new("abc");
    /// assert_matches!(key, QueryKey::Custom(_));
    /// ```
    /// *`fetch-owner` 功能未实现，特殊说明*
    pub fn new(val: impl Into<Cow<'a, str>>) -> Self {
        use InnerQueryKey::*;

        let val = val.into();
        if val.contains("delimiter") {
            Delimiter
        } else if val.contains("start-after") {
            StartAfter
        } else if val.contains("continuation-token") {
            ContinuationToken
        } else if val.contains("max-keys") {
            MaxKeys
        } else if val.contains("prefix") {
            Prefix
        } else if val.contains("encoding-type") {
            EncodingType
        } else if val.contains("fetch-owner") {
            unimplemented!("parse xml not support fetch owner");
        } else {
            Custom(val)
        }
    }

    /// # Examples
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// # use assert_matches::assert_matches;
    /// let key = QueryKey::from_static("delimiter");
    /// assert!(key == QueryKey::Delimiter);
    /// assert!(QueryKey::from_static("start-after") == QueryKey::StartAfter);
    /// assert!(QueryKey::from_static("continuation-token") == QueryKey::ContinuationToken);
    /// assert!(QueryKey::from_static("max-keys") == QueryKey::MaxKeys);
    /// assert!(QueryKey::from_static("prefix") == QueryKey::Prefix);
    /// assert!(QueryKey::from_static("encoding-type") == QueryKey::EncodingType);
    ///
    /// let key = QueryKey::from_static("abc");
    /// assert_matches!(key, QueryKey::Custom(_));
    /// ```
    /// *`fetch-owner` 功能未实现，特殊说明*
    pub fn from_static(val: &str) -> Self {
        use InnerQueryKey::*;

        if val.contains("delimiter") {
            Delimiter
        } else if val.contains("start-after") {
            StartAfter
        } else if val.contains("continuation-token") {
            ContinuationToken
        } else if val.contains("max-keys") {
            MaxKeys
        } else if val.contains("prefix") {
            Prefix
        } else if val.contains("encoding-type") {
            EncodingType
        } else if val.contains("fetch-owner") {
            unimplemented!("parse xml not support fetch owner");
        } else {
            Custom(Cow::Owned(val.to_owned()))
        }
    }
}

#[cfg(test)]
mod test_query_key {
    use super::*;

    #[test]
    #[should_panic]
    fn test_fetch_owner() {
        QueryKey::new("fetch-owner");
    }
}

/// 查询条件的值
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InnerQueryValue<'a>(Cow<'a, str>);
/// 查询条件的值
pub type QueryValue = InnerQueryValue<'static>;

impl AsRef<str> for InnerQueryValue<'_> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for InnerQueryValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for InnerQueryValue<'_> {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}
impl<'a: 'b, 'b> From<&'a str> for InnerQueryValue<'b> {
    fn from(date: &'a str) -> Self {
        Self::new(date)
    }
}

impl PartialEq<&str> for InnerQueryValue<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        &self.0 == other
    }
}

impl From<u8> for InnerQueryValue<'_> {
    /// 数字转 Query 值
    ///
    /// ```
    /// # use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::QueryKey;
    /// let query = Query::from_iter([("max_keys", 100u8)]);
    /// let query = Query::from_iter([(QueryKey::MaxKeys, 100u8)]);
    /// ```
    fn from(num: u8) -> Self {
        Self(Cow::Owned(num.to_string()))
    }
}

impl PartialEq<u8> for InnerQueryValue<'_> {
    #[inline]
    fn eq(&self, other: &u8) -> bool {
        self.to_string() == other.to_string()
    }
}

impl From<u16> for InnerQueryValue<'_> {
    /// 数字转 Query 值
    ///
    /// ```
    /// use aliyun_oss_client::Query;
    /// let query = Query::from_iter([("max_keys", 100u16)]);
    /// ```
    fn from(num: u16) -> Self {
        Self(Cow::Owned(num.to_string()))
    }
}

impl PartialEq<u16> for InnerQueryValue<'_> {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        self.to_string() == other.to_string()
    }
}

impl From<bool> for QueryValue {
    /// bool 转 Query 值
    ///
    /// ```
    /// use aliyun_oss_client::Query;
    /// let query = Query::from_iter([("abc", "false")]);
    /// ```
    fn from(b: bool) -> Self {
        if b {
            Self::from_static("true")
        } else {
            Self::from_static("false")
        }
    }
}

impl FromStr for InnerQueryValue<'_> {
    type Err = InvalidQueryValue;
    /// 示例
    /// ```
    /// # use aliyun_oss_client::types::QueryValue;
    /// let value: QueryValue = "abc".parse().unwrap();
    /// assert!(value == "abc");
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_static2(s))
    }
}

/// 异常的查询值
#[derive(Debug)]
pub struct InvalidQueryValue {
    _priv: (),
}

impl Error for InvalidQueryValue {}

impl Display for InvalidQueryValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid query value")
    }
}

impl<'a> InnerQueryValue<'a> {
    /// Creates a new `QueryValue` from the given string.
    pub fn new(val: impl Into<Cow<'a, str>>) -> Self {
        Self(val.into())
    }

    /// Const function that creates a new `QueryValue` from a static str.
    pub const fn from_static(val: &'static str) -> Self {
        Self(Cow::Borrowed(val))
    }

    /// Const function that creates a new `QueryValue` from a static str.
    pub fn from_static2(val: &str) -> Self {
        Self(Cow::Owned(val.to_owned()))
    }
}

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::{Index, Range, RangeFrom, RangeFull, RangeTo};
use std::str::FromStr;

use http::HeaderValue;
use reqwest::Url;

/// 用于指定返回内容的区域的 type
pub struct ContentRange {
    start: Option<u32>,
    end: Option<u32>,
}

impl From<Range<u32>> for ContentRange {
    fn from(r: Range<u32>) -> Self {
        Self {
            start: Some(r.start),
            end: Some(r.end),
        }
    }
}

impl From<RangeFull> for ContentRange {
    fn from(_f: RangeFull) -> Self {
        Self {
            start: None,
            end: None,
        }
    }
}

impl From<RangeFrom<u32>> for ContentRange {
    fn from(f: RangeFrom<u32>) -> Self {
        Self {
            start: Some(f.start),
            end: None,
        }
    }
}

impl From<RangeTo<u32>> for ContentRange {
    fn from(t: RangeTo<u32>) -> Self {
        Self {
            start: None,
            end: Some(t.end),
        }
    }
}

impl From<ContentRange> for HeaderValue {
    /// # 转化成 OSS 需要的格式
    /// @link [OSS 文档](https://help.aliyun.com/document_detail/31980.html)
    ///
    /// ```
    /// use reqwest::header::HeaderValue;
    /// # use aliyun_oss_client::types::ContentRange;
    /// fn abc<R: Into<ContentRange>>(range: R) -> HeaderValue {
    ///     range.into().into()
    /// }
    ///
    /// assert_eq!(abc(..), HeaderValue::from_str("bytes=0-").unwrap());
    /// assert_eq!(abc(1..), HeaderValue::from_str("bytes=1-").unwrap());
    /// assert_eq!(abc(10..20), HeaderValue::from_str("bytes=10-20").unwrap());
    /// assert_eq!(abc(..20), HeaderValue::from_str("bytes=0-20").unwrap());
    /// ```
    fn from(con: ContentRange) -> HeaderValue {
        let string = match (con.start, con.end) {
            (Some(ref start), Some(ref end)) => format!("bytes={}-{}", start, end),
            (Some(ref start), None) => format!("bytes={}-", start),
            (None, Some(ref end)) => format!("bytes=0-{}", end),
            (None, None) => "bytes=0-".to_string(),
        };

        HeaderValue::from_str(&string).unwrap_or_else(|_| {
            panic!(
                "content-range into header-value failed, content-range is : {}",
                string
            )
        })
    }
}
