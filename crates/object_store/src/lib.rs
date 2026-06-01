use std::{fmt::Display, sync::Arc};

use aliyun_oss_client::{Bucket, Object};
use async_trait::async_trait;
use futures_util::stream::BoxStream;
use object_store::{
    path::Path, Attribute, CopyOptions, Error, GetOptions, GetResult, ListResult, MultipartUpload,
    ObjectMeta, ObjectStore, PutMultipartOptions, PutOptions, PutPayload, PutResult, Result,
};

mod put_payload;
use put_payload::BuiltinPutPayload;

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

    pub fn object(&self, path: &Path) -> Object {
        Object::new(path.to_string(), Arc::new(self.bucket.clone()))
    }
}

#[async_trait]
impl ObjectStore for AliyunOssObjectStore {
    async fn put_opts(
        &self,
        location: &Path,
        payload: PutPayload,
        opts: PutOptions,
    ) -> Result<PutResult> {
        let mut object = self.object(location);

        if let Some(content_type) = opts.attributes.get(&Attribute::ContentType) {
            object = object.content_type(content_type.as_ref());
        }

        object
            .upload(BuiltinPutPayload::new(payload))
            .await
            .map_err(|e| Error::Generic {
                store: "AliyunOssObjectStore",
                source: Box::new(e),
            })?;

        Ok(PutResult {
            e_tag: None,
            version: None,
        })
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
