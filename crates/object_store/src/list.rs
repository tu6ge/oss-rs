use chrono::{DateTime, Utc};
use object_store::{path::Path, Error, ObjectMeta, Result};
use serde::Deserialize;

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
