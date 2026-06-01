use aliyun_oss_client::object::{BodyStream, IntoBody};
use object_store::PutPayload;

pub(crate) struct BuiltinPutPayload(PutPayload);

impl BuiltinPutPayload {
    pub fn new(payload: PutPayload) -> Self {
        Self(payload)
    }
}

impl IntoBody for BuiltinPutPayload {
    fn into_body(self) -> BodyStream {
        Box::pin(futures_util::stream::once(async move { Ok(self.0.into()) }))
    }
}
