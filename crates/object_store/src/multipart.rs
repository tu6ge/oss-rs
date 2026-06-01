use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use aliyun_oss_client::object::PartsUpload;
use async_trait::async_trait;
use bytes::Bytes;
use object_store::{
    path::Path, Error, MultipartUpload, PutPayload, PutResult, Result, UploadPart,
};
use tokio::sync::Mutex;

fn oss_error(err: aliyun_oss_client::Error) -> Error {
    Error::Generic {
        store: "AliyunOssObjectStore",
        source: Box::new(err),
    }
}

fn payload_to_bytes(payload: PutPayload) -> Vec<u8> {
    let bytes: Bytes = payload.into();
    bytes.to_vec()
}

/// OSS 分片编号从 1 开始。
#[derive(Debug)]
pub(crate) struct OssMultipartUpload {
    next_part_number: AtomicUsize,
    upload: Arc<Mutex<PartsUpload>>,
}

impl OssMultipartUpload {
    pub(crate) async fn new(
        location: Path,
        bucket: Arc<aliyun_oss_client::Bucket>,
    ) -> Result<Self, Error> {
        let mut upload = PartsUpload::new(location.as_ref(), bucket);
        upload
            .init_mulit()
            .await
            .map_err(oss_error)?;

        Ok(Self {
            next_part_number: AtomicUsize::new(1),
            upload: Arc::new(Mutex::new(upload)),
        })
    }
}

#[async_trait]
impl MultipartUpload for OssMultipartUpload {
    fn put_part(&mut self, data: PutPayload) -> UploadPart {
        let part_number = self.next_part_number.fetch_add(1, Ordering::Relaxed);
        let upload = Arc::clone(&self.upload);
        let body = payload_to_bytes(data);

        Box::pin(async move {
            upload
                .lock()
                .await
                .upload_part(part_number, body)
                .await
                .map_err(oss_error)?;
            Ok(())
        })
    }

    async fn complete(&mut self) -> Result<PutResult> {
        let etag = self
            .upload
            .lock()
            .await
            .complete_with_etag()
            .await
            .map_err(oss_error)?;

        Ok(PutResult {
            e_tag: Some(etag),
            version: None,
        })
    }

    async fn abort(&mut self) -> Result<()> {
        self.upload
            .lock()
            .await
            .abort_multipart()
            .await
            .map_err(oss_error)?;
        Ok(())
    }
}
