use bytes::Bytes;
use futures_core::Stream;
use futures_util::stream;
use std::pin::Pin;

#[cfg(feature = "tokio")]
use tokio::fs::File;
#[cfg(feature = "tokio")]
use tokio_util::io::ReaderStream;

pub type BodyStream = Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>;

pub trait IntoBody {
    fn into_body(self) -> BodyStream;
    fn content_length(&self) -> Option<usize> {
        None
    }
}

impl IntoBody for String {
    fn into_body(self) -> BodyStream {
        Box::pin(stream::once(async move { Ok(Bytes::from(self)) }))
    }
    fn content_length(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl IntoBody for &'static str {
    fn into_body(self) -> BodyStream {
        Box::pin(stream::once(async move {
            Ok(Bytes::from_static(self.as_bytes()))
        }))
    }
    fn content_length(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl IntoBody for Vec<u8> {
    fn into_body(self) -> BodyStream {
        Box::pin(stream::once(async move { Ok(Bytes::from(self)) }))
    }
    fn content_length(&self) -> Option<usize> {
        Some(self.len())
    }
}

#[cfg(feature = "tokio")]
impl IntoBody for File {
    fn into_body(self) -> BodyStream {
        use futures_util::StreamExt;

        let stream = ReaderStream::new(self).map(|res| res.map(Bytes::from));
        Box::pin(stream)
    }
    fn content_length(&self) -> Option<usize> {
        None
    }
}
