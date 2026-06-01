use std::fmt::Display;

use aliyun_oss_client::Bucket;
use async_trait::async_trait;
use futures_util::stream::BoxStream;
use object_store::{
    path::Path, CopyOptions, Error, GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta,
    ObjectStore, PutMultipartOptions, PutOptions, PutPayload, PutResult, Result,
};

#[derive(Debug)]
pub struct AliyunOssObjectStore {
    bucket: Bucket,
}

impl Display for AliyunOssObjectStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AliyunOssObjectStore")
    }
}

impl AliyunOssObjectStore {
    pub fn new(bucket: Bucket) -> Self {
        Self { bucket }
    }
}

#[async_trait]
impl ObjectStore for AliyunOssObjectStore {
    async fn put_opts(
        &self,
        location: &Path,
        payload: PutPayload,
        opts: PutOptions,
    ) -> Result<PutResult, Error> {
        todo!()
    }
    async fn put_multipart_opts(
        &self,
        location: &Path,
        opts: PutMultipartOptions,
    ) -> Result<Box<dyn MultipartUpload>> {
        todo!()
    }
    async fn get_opts(&self, location: &Path, opts: GetOptions) -> Result<GetResult, Error> {
        todo!()
    }

    fn delete_stream(
        &self,
        locations: BoxStream<'static, Result<Path, Error>>,
    ) -> BoxStream<'static, Result<Path, Error>> {
        todo!()
    }

    fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, Result<ObjectMeta>> {
        todo!()
    }

    async fn list_with_delimiter(&self, prefix: Option<&Path>) -> Result<ListResult> {
        todo!()
    }

    async fn copy_opts(&self, from: &Path, to: &Path, options: CopyOptions) -> Result<()> {
        todo!()
    }
}
