use std::{fmt::Display, io::Cursor, sync::Arc};

use aliyun_oss_client::{Bucket, Error as OssError, Object};
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::{stream::BoxStream, StreamExt as _};
use object_store::{
    path::Path, Attribute, Attributes, CopyOptions, Error, GetOptions, GetResult, GetResultPayload,
    ListResult, MultipartUpload, ObjectMeta, ObjectStore, PutMultipartOptions, PutOptions,
    PutPayload, PutResult, Result,
};

mod list;
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

fn to_object_store_error(err: OssError, path: &Path) -> Error {
    match err.service_code() {
        Some("NoSuchKey") => Error::NotFound {
            path: path.to_string(),
            source: Box::new(err),
        },
        _ => Error::Generic {
            store: "AliyunOssObjectStore",
            source: Box::new(err),
        },
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
            .map_err(|e| to_object_store_error(e, location))?;

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

    async fn get_opts(&self, location: &Path, opts: GetOptions) -> Result<GetResult> {
        let object = self.object(location);

        let info = object
            .get_info()
            .await
            .map_err(|e| to_object_store_error(e, location))?;

        let meta = ObjectMeta {
            location: location.clone(),
            last_modified: *info.last_modified(),
            size: info.size(),
            e_tag: Some(info.etag().to_string()),
            version: None,
        };

        opts.check_preconditions(&meta)?;

        if opts.version.is_some() {
            return Err(Error::NotImplemented {
                operation: "get with version".into(),
                implementer: "AliyunOssObjectStore".into(),
            });
        }

        let range = match &opts.range {
            Some(r) => r.as_range(meta.size).map_err(|source| Error::Generic {
                store: "AliyunOssObjectStore",
                source: Box::new(source),
            })?,
            None => 0..meta.size,
        };

        if opts.head {
            return Ok(GetResult {
                payload: GetResultPayload::Stream(futures_util::stream::empty().boxed()),
                meta,
                range,
                attributes: Attributes::new(),
            });
        }

        let stream = if range == (0..meta.size) {
            object
                .download_stream()
                .await
                .map_err(|e| to_object_store_error(e, location))?
                .map(|chunk| {
                    chunk.map_err(|e| Error::Generic {
                        store: "AliyunOssObjectStore",
                        source: Box::new(e),
                    })
                })
                .boxed()
        } else {
            let mut buf = Cursor::new(Vec::with_capacity(meta.size as usize));
            object
                .download(&mut buf)
                .await
                .map_err(|e| to_object_store_error(e, location))?;

            let bytes = Bytes::from(buf.into_inner());
            let start = range.start as usize;
            let end = range.end as usize;
            let data = bytes.slice(start..end.min(bytes.len()));
            futures_util::stream::once(futures_util::future::ready(Ok(data))).boxed()
        };

        Ok(GetResult {
            payload: GetResultPayload::Stream(stream),
            meta,
            range,
            attributes: Attributes::new(),
        })
    }

    fn delete_stream(
        &self,
        locations: BoxStream<'static, Result<Path, Error>>,
    ) -> BoxStream<'static, Result<Path, Error>> {
        let bucket = self.bucket.clone();
        locations
            .map(move |location| {
                let bucket = bucket.clone();
                async move {
                    let location = location?;
                    Object::new(location.to_string(), Arc::new(bucket))
                        .delete()
                        .await
                        .map_err(|e| to_object_store_error(e, &location))?;
                    Ok(location)
                }
            })
            .buffered(10)
            .boxed()
    }

    fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, Result<ObjectMeta>> {
        use async_stream::try_stream;
        use list::{should_include, to_meta, ListedObject};

        let prefix_len = prefix.map(|p| p.as_ref().len()).unwrap_or_default();
        let prefix_filter = prefix.cloned();

        let mut bucket = self.bucket.clone();
        if let Some(ref p) = prefix_filter {
            bucket = bucket.prefix(p.as_ref());
        }

        try_stream! {
            let mut objects = std::pin::pin!(bucket.objects_as_impl::<ListedObject>());
            while let Some(item) = objects.next().await {
                let obj = item.map_err(|e| Error::Generic {
                    store: "AliyunOssObjectStore",
                    source: Box::new(e),
                })?;

                let Some(meta) = to_meta(obj)? else {
                    continue;
                };

                if should_include(&meta.location, prefix_filter.as_ref(), prefix_len) {
                    yield meta;
                }
            }
        }
        .boxed()
    }

    async fn list_with_delimiter(&self, prefix: Option<&Path>) -> Result<ListResult> {
        list::fetch_list_with_delimiter(self.bucket.clone(), prefix).await
    }

    async fn copy_opts(&self, from: &Path, to: &Path, options: CopyOptions) -> Result<()> {
        todo!()
    }
}
