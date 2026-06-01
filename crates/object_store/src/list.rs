use aliyun_oss_client::{Bucket, Error as OssError};
use chrono::{DateTime, Utc};
use object_store::{path::Path, Error, ListResult, ObjectMeta, Result};
use serde::Deserialize;

/// object_store 目录浏览使用的路径分隔符，与 OSS `delimiter` 一致。
const LIST_DELIMITER: &str = "/";

#[derive(Deserialize)]
pub(crate) struct ListedObject {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "LastModified")]
    last_modified: String,
    #[serde(rename = "ETag")]
    etag: String,
    #[serde(rename = "Size")]
    size: u64,
}

pub(crate) fn to_meta(obj: ListedObject) -> Result<Option<ObjectMeta>> {
    if obj.key.ends_with('/') {
        return Ok(None);
    }

    let location = Path::from(obj.key);
    let last_modified = DateTime::parse_from_rfc3339(&obj.last_modified)
        .map_err(|e| Error::Generic {
            store: "AliyunOssObjectStore",
            source: Box::new(e),
        })?
        .with_timezone(&Utc);
    let e_tag = obj.etag.trim_matches('"').to_string();

    Ok(Some(ObjectMeta {
        location,
        last_modified,
        size: obj.size,
        e_tag: Some(e_tag),
        version: None,
    }))
}

#[derive(Deserialize)]
struct CommonPrefix {
    #[serde(rename = "Prefix")]
    prefix: String,
}

fn common_prefix_to_path(prefix: String) -> Path {
    Path::from(prefix.trim_end_matches('/'))
}

fn oss_error(err: OssError) -> Error {
    Error::Generic {
        store: "AliyunOssObjectStore",
        source: Box::new(err),
    }
}

pub(crate) async fn fetch_list_with_delimiter(
    mut bucket: Bucket,
    prefix: Option<&Path>,
) -> Result<ListResult> {
    let prefix_len = prefix.map(|p| p.as_ref().len()).unwrap_or(0);

    if let Some(p) = prefix {
        bucket = bucket.prefix(p.as_ref());
    }
    bucket = bucket.delimiter(LIST_DELIMITER);

    let mut objects = Vec::new();
    let mut common_prefixes = Vec::new();
    let mut token: Option<String> = None;

    loop {
        if let Some(ref t) = token {
            bucket = bucket.continuation_token(t);
        }

        let (contents, prefixes, next) = bucket
            .list_objects_page::<ListedObject, CommonPrefix>()
            .await
            .map_err(oss_error)?;

        for obj in contents {
            let Some(meta) = to_meta(obj)? else {
                continue;
            };
            if meta.location.as_ref().len() > prefix_len {
                objects.push(meta);
            }
        }

        for cp in prefixes {
            let path = common_prefix_to_path(cp.prefix);
            if path.as_ref().len() > prefix_len {
                common_prefixes.push(path);
            }
        }

        if next.is_none() {
            break;
        }
        token = next;
    }

    Ok(ListResult {
        objects,
        common_prefixes,
    })
}

pub(crate) fn should_include(location: &Path, prefix: Option<&Path>, prefix_len: usize) -> bool {
    if location.as_ref().len() <= prefix_len {
        return false;
    }

    match prefix {
        Some(p) => location
            .prefix_match(p)
            .map(|mut it| it.next().is_some())
            .unwrap_or(false),
        None => true,
    }
}
