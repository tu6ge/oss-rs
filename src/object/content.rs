//! 读写 object 内容

use std::{io::Write, sync::Arc};

use futures::executor::block_on;

use crate::{
    decode::RefineObject,
    file::{Files, DEFAULT_CONTENT_TYPE},
    types::object::InvalidObjectPath,
    Client, ObjectPath, Query,
};

use super::{BuildInItemError, BuildInItemErrorKind, Objects};

/// # object 内容
/// [OSS 分片上传文档](https://help.aliyun.com/zh/oss/user-guide/multipart-upload-12)
pub struct Content {
    client: Arc<Client>,
    path: ObjectPath,
    content: Vec<u8>,
    content_type: &'static str,
}

/// 带内容的 object 列表
pub type List = Objects<Content>;

impl Content {
    const DEFAULT_CONTENT_TYPE: &str = DEFAULT_CONTENT_TYPE;
    fn init_object(list: &mut List) -> Option<Content> {
        Some(Content {
            client: list.client(),
            path: ObjectPath::default(),
            content: Vec::default(),
            content_type: Self::DEFAULT_CONTENT_TYPE,
        })
    }
    /// 从 client 创建
    pub fn from_client(client: Arc<Client>) -> Content {
        Content {
            client,
            path: ObjectPath::default(),
            content: Vec::default(),
            content_type: Self::DEFAULT_CONTENT_TYPE,
        }
    }
    /// 设置 ObjectPath
    pub fn path<P>(mut self, path: P) -> Result<Self, InvalidObjectPath>
    where
        P: TryInto<ObjectPath>,
        P::Error: Into<InvalidObjectPath>,
    {
        self.path = path.try_into().map_err(Into::into)?;
        Ok(self)
    }
    fn content_type_from_key(&mut self, key: &str) {
        self.content_type = match key.rsplit(".").next() {
            Some(str) => match str.to_lowercase().as_str() {
                "jpg" => "image/jpeg",
                "pdf" => "application/pdf",
                "png" => "image/png",
                "gif" => "image/gif",
                "bmp" => "image/bmp",
                "zip" => "application/zip",
                "tar" => "application/x-tar",
                "gz" => "application/gzip",
                "txt" => "text/plain",
                "mp3" => "audio/mpeg",
                "wav" => "audio/wave",
                "mp4" => "video/mp4",
                "mov" => "video/quicktime",
                "avi" => "video/x-msvideo",
                "wmv" => "video/x-ms-wmv",
                "html" => "text/html",
                "js" => "application/javascript",
                "css" => "text/css",
                "php" => "application/x-httpd-php",
                _ => DEFAULT_CONTENT_TYPE,
            },
            None => DEFAULT_CONTENT_TYPE,
        }
    }
}

impl RefineObject<BuildInItemError> for Content {
    #[inline]
    fn set_key(&mut self, key: &str) -> Result<(), BuildInItemError> {
        self.path = key.parse().map_err(|e| BuildInItemError {
            source: key.to_string(),
            kind: BuildInItemErrorKind::BasePath(e),
        })?;

        self.content_type_from_key(key);
        Ok(())
    }
    /// 提取 size
    fn set_size(&mut self, size: &str) -> Result<(), BuildInItemError> {
        if let Ok(size) = size.parse() {
            self.content.reserve(size);
        }
        Ok(())
    }
}

async fn main() {
    let client = Client::from_env().unwrap();

    let mut list = client
        .get_custom_object(&Query::default(), Content::init_object)
        .await
        .unwrap();

    let second = list.get_next_base(Content::init_object).await;

    let objcet = Content::from_client(Arc::new(client)).path("aaa").unwrap();
}

impl Write for Content {
    // 普通大小的文件写入
    // 或大文件的部分写入
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // TODO: block_on
        let _ = block_on(self.client.put_content_base(
            buf.to_vec(),
            self.content_type,
            self.path.clone(),
        ))?;

        Ok(buf.len())
    }

    /// 大文件分片写入(完整写入)
    fn write_all(&mut self, mut buf: &[u8]) -> std::io::Result<()> {
        // 如果是小文件，则一次写入
        // 如果是大文件，则调用 write 批量写入
        Ok(())
    }

    // TODO
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl From<Client> for Content {
    fn from(value: Client) -> Self {
        Content {
            client: Arc::new(value),
            path: ObjectPath::default(),
            content: Vec::default(),
            content_type: "",
        }
    }
}
