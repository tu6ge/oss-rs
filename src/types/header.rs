//! # convert headers trait
//!
//! ## Examples
//! ```rust,no_run
//! struct IfUnmodifiedSince {
//!     date: &'static str,
//! }
//!
//! impl TryIntoHeaders for IfUnmodifiedSince {
//!     type Error = InvalidHeaderValue;
//!     fn try_into_headers(self) -> Result<http::HeaderMap, Self::Error> {
//!         let mut map = http::HeaderMap::with_capacity(1);
//!         map.insert("If-Unmodified-Since", self.date.parse()?);
//!         Ok(map)
//!     }
//! }
//! # use dotenv::dotenv;
//! # fn run() {
//! # let client = Client::from_env().unwrap();
//! # let (url, resource) = client
//! #      .get_object_base("9AB932LY.jpeg")?
//! #      .get_url_resource([()]);
//! let builder = client.builder_with_header(
//!     Method::HEAD,
//!     url,
//!     resource,
//!     IfUnmodifiedSince {
//!        date: "Sat, 01 Jan 2022 18:01:01 GMT",
//!     },
//! ).unwrap();
//! # }
//! ```

use std::convert::Infallible;

use http::{
    header::{HeaderName, InvalidHeaderValue, CONTENT_LENGTH},
    HeaderMap, HeaderValue,
};

use super::ContentRange;

/// convert headers trait
///
/// 在构造请求头时，方式更灵活
pub trait TryIntoHeaders {
    /// 自定义错误类型
    type Error;

    /// 尝试将某个类型转化成 `HeaderMap`
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error>;
}

impl TryIntoHeaders for HeaderMap {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        Ok(self)
    }
}

impl TryIntoHeaders for () {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        Ok(HeaderMap::new())
    }
}
impl TryIntoHeaders for (HeaderName, HeaderValue) {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::with_capacity(1);
        map.insert(self.0, self.1);
        Ok(map)
    }
}
impl<const N: usize> TryIntoHeaders for [(HeaderName, HeaderValue); N] {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        Ok(HeaderMap::from_iter(self.into_iter()))
    }
}
impl TryIntoHeaders for Vec<(HeaderName, HeaderValue)> {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        Ok(HeaderMap::from_iter(self.into_iter()))
    }
}

impl TryIntoHeaders for (HeaderKey, HeaderVal) {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::with_capacity(1);
        map.insert(HeaderName::from(self.0), self.1.into());
        Ok(map)
    }
}
impl<const N: usize> TryIntoHeaders for [(HeaderKey, HeaderVal); N] {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::new();
        for (k, v) in self.into_iter() {
            map.insert(HeaderName::from(k), v.into());
        }
        Ok(map)
    }
}
impl TryIntoHeaders for Vec<(HeaderKey, HeaderVal)> {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::new();
        for (k, v) in self.into_iter() {
            map.insert(HeaderName::from(k), v.into());
        }
        Ok(map)
    }
}

impl TryIntoHeaders for (HeaderName, HeaderVal) {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::with_capacity(1);
        map.insert(self.0, self.1.into());
        Ok(map)
    }
}
impl<const N: usize> TryIntoHeaders for [(HeaderName, HeaderVal); N] {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::new();
        for (k, v) in self.into_iter() {
            map.insert(k, v.into());
        }
        Ok(map)
    }
}
impl TryIntoHeaders for Vec<(HeaderName, HeaderVal)> {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let mut map = HeaderMap::new();
        for (k, v) in self.into_iter() {
            map.insert(k, v.into());
        }
        Ok(map)
    }
}

impl<T> TryIntoHeaders for Result<T, T::Error>
where
    T: TryIntoHeaders,
{
    type Error = T::Error;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        self.and_then(|head| head.try_into_headers())
    }
}
impl<T> TryIntoHeaders for Box<T>
where
    T: TryIntoHeaders,
{
    type Error = T::Error;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        (*self).try_into_headers()
    }
}

pub(crate) enum HeaderKey {
    // /// If-Unmodified-Since
    // IfUnmodifiedSince,
    /// range
    Range,
    ContentLength,
    ContentType,
}

impl From<HeaderKey> for HeaderName {
    fn from(value: HeaderKey) -> Self {
        use http::header::CONTENT_TYPE;
        const RANGE: &[u8] = b"Range";
        match value {
            //HeaderKey::IfUnmodifiedSince => HeaderName::from_static("If-Unmodified-Since"),
            HeaderKey::Range =>
            {
                #[allow(clippy::unwrap_used)]
                HeaderName::from_bytes(RANGE).unwrap()
            }
            HeaderKey::ContentLength => CONTENT_LENGTH,
            HeaderKey::ContentType => CONTENT_TYPE,
        }
    }
}

pub(crate) enum HeaderVal {
    Range(HeaderValue),
    ContentLength(usize),
    ContentType(HeaderValue),
}

impl HeaderVal {
    pub fn content_type(str: &str) -> Result<(HeaderKey, Self), InvalidHeaderValue> {
        Ok((HeaderKey::ContentType, Self::ContentType(str.parse()?)))
    }

    pub fn len(len: usize) -> (HeaderKey, Self) {
        (HeaderKey::ContentLength, Self::ContentLength(len))
    }

    pub fn range<Num, R>(range: R) -> (HeaderKey, Self)
    where
        R: Into<ContentRange<Num>>,
        ContentRange<Num>: Into<HeaderValue>,
    {
        (HeaderKey::Range, Self::Range(range.into().into()))
    }
}

impl From<HeaderVal> for HeaderValue {
    fn from(value: HeaderVal) -> Self {
        match value {
            HeaderVal::Range(r) => r,
            HeaderVal::ContentLength(n) => n.into(),
            HeaderVal::ContentType(con) => con,
        }
    }
}
impl TryIntoHeaders for HeaderVal {
    type Error = Infallible;
    fn try_into_headers(self) -> Result<HeaderMap, Self::Error> {
        let key = match &self {
            HeaderVal::Range(_) => HeaderKey::Range,
            HeaderVal::ContentLength(_) => HeaderKey::ContentLength,
            HeaderVal::ContentType(_) => HeaderKey::ContentType,
        };
        (key, self).try_into_headers()
    }
}

#[test]
fn test_into_header() {
    use http::header::CONTENT_TYPE;
    fn get<M: TryIntoHeaders>(_m: M) {}

    get(());
    get((CONTENT_TYPE, HeaderValue::from_static("application/json")));
    get([(CONTENT_TYPE, HeaderValue::from_static("application/json"))]);
    get(vec![(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    )]);
    get([
        (CONTENT_TYPE, HeaderValue::from_static("application/json")),
        (CONTENT_LENGTH, HeaderValue::from_static("12")),
    ]);
    get([(
        HeaderKey::Range,
        HeaderVal::ContentType("application/json".parse().unwrap()),
    )]);
    get((HeaderKey::Range, HeaderVal::ContentLength(10)));
    get([(HeaderKey::Range, HeaderVal::ContentLength(10))]);
    get(vec![(HeaderKey::Range, HeaderVal::ContentLength(10))]);

    get((CONTENT_TYPE, HeaderVal::ContentLength(10)));
    get([(CONTENT_TYPE, HeaderVal::ContentLength(10))]);
    get(vec![(CONTENT_TYPE, HeaderVal::ContentLength(10))]);
    get(HeaderVal::len(10));
    get(HeaderVal::ContentLength(10));

    struct IfUnmodifiedSince {
        date: &'static str,
    }

    impl TryIntoHeaders for IfUnmodifiedSince {
        type Error = InvalidHeaderValue;
        fn try_into_headers(self) -> Result<http::HeaderMap, Self::Error> {
            let mut map = http::HeaderMap::with_capacity(1);
            map.insert("If-Unmodified-Since", self.date.parse()?);
            Ok(map)
        }
    }

    fn since() -> Result<IfUnmodifiedSince, InvalidHeaderValue> {
        Ok(IfUnmodifiedSince { date: "foo" })
    }
    get(since());

    get(Box::new(HeaderVal::len(10)));
}