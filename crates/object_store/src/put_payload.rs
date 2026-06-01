use aliyun_oss_client::object::{BodyStream, IntoBody};
use futures_util::stream;
use object_store::PutPayload;

pub(crate) struct BuiltinPutPayload(PutPayload);

impl BuiltinPutPayload {
    pub fn new(payload: PutPayload) -> Self {
        Self(payload)
    }
}

impl IntoBody for BuiltinPutPayload {
    fn into_body(self) -> BodyStream {
        Box::pin(stream::iter(self.0.into_iter().map(Ok)))
    }

    fn content_length(&self) -> Option<usize> {
        Some(self.0.content_length())
    }
}
