//! 核心功能用到的类型 Query ContentRange 等

use std::borrow::Cow;
use std::collections::HashMap;

const DELIMITER: &str = "delimiter";
const START_AFTER: &str = "start-after";
const CONTINUATION_TOKEN: &str = "continuation-token";
const MAX_KEYS: &str = "max-keys";
const PREFIX: &str = "prefix";
const ENCODING_TYPE: &str = "encoding-type";
const FETCH_OWNER: &str = "fetch-owner";
const DEFAULT_MAX_KEYS: usize = 100;

//===================================================================================================
/// 查询条件
///
/// ```
/// use aliyun_oss_client::types::Query;
///
/// let query: Query = Query::new2([("abc", "def")]);
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Creates a new `Query` with trait
    #[must_use]
    pub fn new2<Q: IntoQuery>(into: Q) -> Self {
        into.into_query()
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
        const LIST_TYPE2: &str = "list-type=2";
        let mut query_str = String::from(LIST_TYPE2);
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

    pub(crate) fn get_max_keys(&self) -> usize {
        match self.get(QueryKey::MAX_KEYS) {
            Some(capacity) => capacity.try_into().unwrap_or(DEFAULT_MAX_KEYS),
            None => DEFAULT_MAX_KEYS,
        }
    }
}

#[cfg(test)]
mod test_query {
    use super::*;
    #[test]
    fn get_max_keys() {
        let query = Query::new();
        assert_eq!(query.get_max_keys(), 100);

        let mut query = Query::new();
        query.insert(QueryKey::MAX_KEYS, "10");
        assert_eq!(query.get_max_keys(), 10);

        let mut query = Query::new();
        query.insert(QueryKey::MAX_KEYS, "str");
        assert_eq!(query.get_max_keys(), 100);
    }
}

impl Index<QueryKey> for Query {
    type Output = QueryValue;

    fn index(&self, index: QueryKey) -> &Self::Output {
        self.get(index).expect("no found query key")
    }
}

/// convert query trait
///
/// 在构造查询条件时，更符合人体工程学
pub trait IntoQuery {
    /// convert query method
    fn into_query(self) -> Query;
}

impl IntoQuery for () {
    fn into_query(self) -> Query {
        Query::default()
    }
}
impl IntoQuery for [(); 1] {
    fn into_query(self) -> Query {
        Query::default()
    }
}
impl IntoQuery for Query {
    fn into_query(self) -> Query {
        self
    }
}

impl<'a, 'b, const N: usize> IntoQuery for [(Cow<'a, str>, Cow<'b, str>); N] {
    fn into_query(self) -> Query {
        let mut query = Query::with_capacity(N);
        for (k, v) in self.into_iter() {
            query.insert(
                k.as_ref().parse::<QueryKey>().expect("invalid QueryKey"),
                v.as_ref()
                    .parse::<QueryValue>()
                    .expect("invalid QueryValue"),
            );
        }
        query
    }
}
impl<'a, 'b> IntoQuery for Vec<(Cow<'a, str>, Cow<'b, str>)> {
    fn into_query(self) -> Query {
        let mut query = Query::with_capacity(self.len());
        for (k, v) in self.into_iter() {
            query.insert(
                k.as_ref().parse::<QueryKey>().expect("invalid QueryKey"),
                v.as_ref()
                    .parse::<QueryValue>()
                    .expect("invalid QueryValue"),
            );
        }
        query
    }
}

macro_rules! into_query_impl {
    ($key:ty, $val:ty) => {
        impl IntoQuery for ($key, $val) {
            fn into_query(self) -> Query {
                let mut query = Query::with_capacity(1);
                query.insert(self.0, self.1);
                query
            }
        }

        impl<const N: usize> IntoQuery for [($key, $val); N] {
            fn into_query(self) -> Query {
                let mut query = Query::with_capacity(N);
                for (k, v) in self.into_iter() {
                    query.insert(k, v);
                }
                query
            }
        }

        impl IntoQuery for Vec<($key, $val)> {
            fn into_query(self) -> Query {
                let mut query = Query::with_capacity(self.len());
                for (k, v) in self.into_iter() {
                    query.insert(k, v);
                }
                query
            }
        }
    };
}

into_query_impl!(QueryKey, QueryValue);

into_query_impl!(&'static str, &'static str);
into_query_impl!(&'static str, u8);
into_query_impl!(&'static str, u16);
into_query_impl!(&'static str, i32);
into_query_impl!(&'static str, String);

into_query_impl!(String, &'static str);
into_query_impl!(String, u8);
into_query_impl!(String, u16);
into_query_impl!(String, i32);
into_query_impl!(String, String);

into_query_impl!(QueryKey, &'static str);
into_query_impl!(QueryKey, u8);
into_query_impl!(QueryKey, u16);
into_query_impl!(QueryKey, i32);
into_query_impl!(QueryKey, String);

impl IntoIterator for Query {
    type Item = (QueryKey, QueryValue);
    type IntoIter = std::vec::IntoIter<Self::Item>;
    /// # 使用 Vec 转 Query
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter().collect::<Vec<_>>().into_iter()
    }
}

#[cfg(test)]
mod tests_query_from_iter {
    use super::*;

    #[test]
    fn test_into_trait() {
        fn search<Q: IntoQuery>(_q: Q) {}

        search(());

        search((QueryKey::MAX_KEYS, 1_u8));
        search((QueryKey::MAX_KEYS, 1_u16));
        search((QueryKey::MAX_KEYS, "foo"));
        search((QueryKey::MAX_KEYS, QueryValue::from_static("val")));
        search((QueryKey::MAX_KEYS, "foo".to_string()));
        search(("max-keys", "foo"));
        search(("max-keys".to_string(), "foo"));
        search(("max-keys".to_string(), "foo".to_string()));

        search([("abc", "def")]);
        search([("abc", 1_u8)]);
        search([("abc", 1_u16)]);
        search([(Cow::Borrowed("key"), Cow::Borrowed("val"))]);

        search([(QueryKey::MAX_KEYS, 1_u8)]);
        search([(QueryKey::MAX_KEYS, 1_u16)]);
        search([(QueryKey::MAX_KEYS, "foo")]);
        search(Query::default());

        search([(QueryKey::MAX_KEYS, QueryValue::from_static("val"))]);

        search(vec![("abc", "def")]);
        search(vec![("abc", 1_u8)]);
        search(vec![("abc", 1_u16)]);
        search(vec![(Cow::Borrowed("key"), Cow::Borrowed("val"))]);
        search(vec![(QueryKey::MAX_KEYS, 1_u8)]);
        search(vec![(QueryKey::MAX_KEYS, 1_u16)]);
        search(vec![(QueryKey::MAX_KEYS, "foo")]);
        search(vec![(QueryKey::MAX_KEYS, QueryValue::from_static("val"))]);

        let q = [("abc", "def"), ("aaa", "ccc")];
        let query = q.into_query();
        assert_eq!(query.len(), 2);
    }
}

impl PartialEq<Query> for Query {
    fn eq(&self, other: &Query) -> bool {
        self.as_ref() == other.as_ref()
    }
}

/// 为 Url 拼接 [`Query`] 数据
/// [`Query`]: crate::types::Query
pub trait SetOssQuery: private::Sealed {
    /// 给 Url 结构体增加 `set_search_query` 方法
    fn set_oss_query(&mut self, query: &Query);
}

mod private {
    pub trait Sealed {}
}

impl private::Sealed for Url {}

impl SetOssQuery for Url {
    /// 将查询参数拼接到 API 的 Url 上
    ///
    /// # 例子
    /// ```
    /// use aliyun_oss_client::types::Query;
    /// use aliyun_oss_client::types::SetOssQuery;
    /// use reqwest::Url;
    /// # use aliyun_oss_client::types::core::IntoQuery;
    ///
    /// let query = [("abc", "def")].into_query();
    /// let mut url = Url::parse("https://exapmle.com").unwrap();
    /// url.set_oss_query(&query);
    /// assert_eq!(url.as_str(), "https://exapmle.com/?list-type=2&abc=def");
    /// assert_eq!(url.query(), Some("list-type=2&abc=def"));
    /// ```
    fn set_oss_query(&mut self, query: &Query) {
        self.set_query(Some(&query.to_oss_string()));
    }
}

/// 查询条件的键
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct InnerQueryKey<'a> {
    kind: QueryKeyEnum<'a>,
}

impl InnerQueryKey<'_> {
    /// 对Object名字进行分组的字符。所有Object名字包含指定的前缀，第一次出现delimiter字符之间的Object作为一组元素（即CommonPrefixes）
    /// 示例值 `/`
    pub const DELIMITER: Self = Self {
        kind: QueryKeyEnum::Delimiter,
    };

    /// 设定从start-after之后按字母排序开始返回Object。
    /// start-after用来实现分页显示效果，参数的长度必须小于1024字节。
    /// 做条件查询时，即使start-after在列表中不存在，也会从符合start-after字母排序的下一个开始打印。
    pub const START_AFTER: Self = Self {
        kind: QueryKeyEnum::StartAfter,
    };

    /// 指定List操作需要从此token开始。您可从ListObjectsV2（GetBucketV2）结果中的NextContinuationToken获取此token。
    /// 用于分页，返回下一页的数据
    pub const CONTINUATION_TOKEN: Self = Self {
        kind: QueryKeyEnum::ContinuationToken,
    };

    /// 指定返回Object的最大数。
    /// 取值：大于0小于等于1000
    pub const MAX_KEYS: Self = Self {
        kind: QueryKeyEnum::MaxKeys,
    };

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
    pub const PREFIX: Self = Self {
        kind: QueryKeyEnum::Prefix,
    };

    /// 对返回的内容进行编码并指定编码的类型。
    pub const ENCODING_TYPE: Self = Self {
        kind: QueryKeyEnum::EncodingType,
    };

    /// 指定是否在返回结果中包含owner信息。
    pub const FETCH_OWNER: Self = Self {
        kind: QueryKeyEnum::FetchOwner,
    };
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
enum QueryKeyEnum<'a> {
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
    /// assert_eq!(QueryKey::DELIMITER.as_ref(), "delimiter");
    /// assert_eq!(QueryKey::START_AFTER.as_ref(), "start-after");
    /// assert_eq!(QueryKey::CONTINUATION_TOKEN.as_ref(), "continuation-token");
    /// assert_eq!(QueryKey::MAX_KEYS.as_ref(), "max-keys");
    /// assert_eq!(QueryKey::PREFIX.as_ref(), "prefix");
    /// assert_eq!(QueryKey::ENCODING_TYPE.as_ref(), "encoding-type");
    /// assert_eq!(QueryKey::new("abc").as_ref(), "abc");
    /// ```
    fn as_ref(&self) -> &str {
        use QueryKeyEnum::*;

        match &self.kind {
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
    /// assert_eq!(format!("{}", QueryKey::DELIMITER), "delimiter");
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
    /// let key = QueryKey::new("delimiter");
    /// assert!(key == QueryKey::DELIMITER);
    /// assert!(QueryKey::new("start-after") == QueryKey::START_AFTER);
    /// assert!(QueryKey::new("continuation-token") == QueryKey::CONTINUATION_TOKEN);
    /// assert!(QueryKey::new("max-keys") == QueryKey::MAX_KEYS);
    /// assert!(QueryKey::new("prefix") == QueryKey::PREFIX);
    /// assert!(QueryKey::new("encoding-type") == QueryKey::ENCODING_TYPE);
    /// ```
    /// *`fetch-owner` 功能未实现，特殊说明*
    pub fn new(val: impl Into<Cow<'a, str>>) -> Self {
        use QueryKeyEnum::*;

        let val = val.into();
        let kind = if val.contains(DELIMITER) {
            Delimiter
        } else if val.contains(START_AFTER) {
            StartAfter
        } else if val.contains(CONTINUATION_TOKEN) {
            ContinuationToken
        } else if val.contains(MAX_KEYS) {
            MaxKeys
        } else if val.contains(PREFIX) {
            Prefix
        } else if val.contains(ENCODING_TYPE) {
            EncodingType
        } else if val.contains(FETCH_OWNER) {
            unimplemented!("parse xml not support fetch owner");
        } else {
            Custom(val)
        };
        Self { kind }
    }

    /// # Examples
    /// ```
    /// # use aliyun_oss_client::QueryKey;
    /// let key = QueryKey::from_static("delimiter");
    /// assert!(key == QueryKey::DELIMITER);
    /// assert!(QueryKey::from_static("start-after") == QueryKey::START_AFTER);
    /// assert!(QueryKey::from_static("continuation-token") == QueryKey::CONTINUATION_TOKEN);
    /// assert!(QueryKey::from_static("max-keys") == QueryKey::MAX_KEYS);
    /// assert!(QueryKey::from_static("prefix") == QueryKey::PREFIX);
    /// assert!(QueryKey::from_static("encoding-type") == QueryKey::ENCODING_TYPE);
    /// ```
    /// *`fetch-owner` 功能未实现，特殊说明*
    pub fn from_static(val: &str) -> Self {
        use QueryKeyEnum::*;

        let kind = if val.contains(DELIMITER) {
            Delimiter
        } else if val.contains(START_AFTER) {
            StartAfter
        } else if val.contains(CONTINUATION_TOKEN) {
            ContinuationToken
        } else if val.contains(MAX_KEYS) {
            MaxKeys
        } else if val.contains(PREFIX) {
            Prefix
        } else if val.contains(ENCODING_TYPE) {
            EncodingType
        } else if val.contains(FETCH_OWNER) {
            unimplemented!("parse xml not support fetch owner");
        } else {
            Custom(Cow::Owned(val.to_owned()))
        };
        Self { kind }
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

    #[test]
    fn test_custom() {
        let key = QueryKey::new("abc");
        assert!(matches!(key.kind, QueryKeyEnum::Custom(_)));
    }

    #[test]
    fn test_into_iter() {
        let query = vec![("foo", "bar")].into_query();
        let list: Vec<_> = query.into_iter().collect();
        assert_eq!(list.len(), 1);
        assert!(matches!(&list[0].0.kind, &QueryKeyEnum::Custom(_)));
        let value: QueryValue = "bar".parse().unwrap();
        assert_eq!(list[0].1, value);
    }

    #[test]
    fn test_from_static() {
        let key = QueryKey::from_static("abc");
        assert!(matches!(key.kind, QueryKeyEnum::Custom(_)));
    }
}

/// 查询条件的值
#[derive(Clone, Debug, PartialEq, Eq, Default, Hash)]
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
    /// # use aliyun_oss_client::types::core::IntoQuery;
    /// let query = [("max_keys", 100u8)].into_query();
    /// let query = [(QueryKey::MAX_KEYS, 100u8)].into_query();
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
    /// # use aliyun_oss_client::types::core::IntoQuery;
    /// let query = [("max_keys", 100u16)].into_query();
    /// ```
    fn from(num: u16) -> Self {
        Self(Cow::Owned(num.to_string()))
    }
}

impl From<i32> for InnerQueryValue<'_> {
    /// 数字转 Query 值
    ///
    /// ```
    /// use aliyun_oss_client::Query;
    /// # use aliyun_oss_client::types::core::IntoQuery;
    /// let query = [("max_keys", 100_i32)].into_query();
    /// ```
    fn from(num: i32) -> Self {
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
    /// # use aliyun_oss_client::types::core::IntoQuery;
    /// let query = [("abc", false)].into_query();
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

impl TryFrom<&InnerQueryValue<'_>> for usize {
    type Error = ParseIntError;
    fn try_from(value: &InnerQueryValue<'_>) -> Result<Self, Self::Error> {
        value.0.parse()
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
use std::num::ParseIntError;
use std::ops::{Index, Range, RangeFrom, RangeFull, RangeTo};
use std::str::FromStr;

use http::HeaderValue;
use reqwest::Url;

/// 用于指定返回内容的区域的 type
pub struct ContentRange<Num> {
    start: Option<Num>,
    end: Option<Num>,
}

unsafe impl<Num: Send> Send for ContentRange<Num> {}
unsafe impl<Num: Sync> Sync for ContentRange<Num> {}

impl<Num> From<Range<Num>> for ContentRange<Num> {
    fn from(r: Range<Num>) -> Self {
        Self {
            start: Some(r.start),
            end: Some(r.end),
        }
    }
}

impl From<RangeFull> for ContentRange<u32> {
    fn from(_: RangeFull) -> Self {
        Self {
            start: None,
            end: None,
        }
    }
}

impl<Num> From<RangeFrom<Num>> for ContentRange<Num> {
    fn from(f: RangeFrom<Num>) -> Self {
        Self {
            start: Some(f.start),
            end: None,
        }
    }
}

impl<Num> From<RangeTo<Num>> for ContentRange<Num> {
    fn from(t: RangeTo<Num>) -> Self {
        Self {
            start: None,
            end: Some(t.end),
        }
    }
}

macro_rules! generate_range {
    ($($t:ty)*) => ($(
        impl From<ContentRange<$t>> for HeaderValue {
            /// # 转化成 OSS 需要的格式
            /// @link [OSS 文档](https://help.aliyun.com/document_detail/31980.html)
            fn from(con: ContentRange<$t>) -> HeaderValue {
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
    )*)
}

generate_range!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);

#[cfg(test)]
mod test_range {
    #[test]
    fn test() {
        use super::ContentRange;
        use reqwest::header::HeaderValue;
        fn abc<R: Into<ContentRange<u32>>>(range: R) -> HeaderValue {
            range.into().into()
        }

        assert_eq!(abc(..), HeaderValue::from_str("bytes=0-").unwrap());
        assert_eq!(abc(1..), HeaderValue::from_str("bytes=1-").unwrap());
        assert_eq!(abc(10..20), HeaderValue::from_str("bytes=10-20").unwrap());
        assert_eq!(abc(..20), HeaderValue::from_str("bytes=0-20").unwrap());
    }
}
