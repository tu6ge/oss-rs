get Buckets

```rust
// client 内部是 Arc<Inner>，克隆成本很低
let bucket = client.bucket("my-bucket");

// 不需要再传 &client，因为 bucket 内部持有 client 的引用
let objects = bucket.get_objects().await?;

// 查询条件 builder
let query = ObjectQuery::new().max_keys(10).prefix("dir1/").delimiter("/");
```

get bucket info;

```rust
use futures::StreamExt;

let mut stream = bucket.list_objects_stream();

while let Some(item) = stream.next().await {
    match item {
        Ok(object) => println!("Got object: {:?}", object.key),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

流的实现细节

```rust
pub struct ListObjectsBuilder<'a> {
    bucket: &'a Bucket,
    prefix: Option<String>,
    delimiter: Option<String>,
    max_keys: Option<u32>,
}

impl<'a> ListObjectsBuilder<'a> {
    pub fn into_stream(
        self,
    ) -> impl Stream<Item = Result<Object, OssError>> + 'a {
        let bucket = self.bucket;
        let prefix = self.prefix;
        let delimiter = self.delimiter;

        try_stream! {
            let mut marker = None;

            loop {
                let resp = bucket
                    .list_objects_page(prefix.clone(), marker.clone())
                    .await?;

                for obj in resp.objects {
                    yield obj;
                }

                if !resp.is_truncated {
                    break;
                }

                marker = resp.next_marker;
            }
        }
    }
}
```

get object info

```rust
// 定义：
// fn upload<T: Into<ByteStream>>(&self, content: T) -> ...

// 用法：
// 传字符串
bucket.object("a.txt").upload("hello world").await?;
// 传字节
bucket.object("b.txt").upload(vec![1, 2, 3]).await?;
// 传文件（异步）
let file = tokio::fs::File::open("...").await?;
bucket.object("c.txt").upload(file).await?;

// 复制文件
let src = client.bucket("b1").object("k1");
dest_obj.copy_from(&src).await?;
```

上传文件的设计细节

```rust
// 定义一个 Trait，用于转换各种输入为 HTTP Body
pub trait IntoBody {
    fn into_body(self) -> Vec<u8>; // 简化示例，实际可能是 Bytes 或 Stream
}

impl IntoBody for String { ... }
impl IntoBody for &'static str { ... }
impl IntoBody for Vec<u8> { ... }
// 甚至可以为 File 实现异步转换（如果你的架构支持）

// 结构体实现
impl ObjectBuilder {
    pub async fn upload<T: IntoBody>(self, content: T) -> Result<()> {
        let body = content.into_body();
        // 发送请求...
    }
}
```

另外一版 IntoBody

```rust
use bytes::Bytes;
use futures_core::Stream;
use std::pin::Pin;

pub type BodyStream =
    Pin<Box<dyn Stream<Item = Result<Bytes, std::io::Error>> + Send>>;

pub trait IntoBody {
    fn into_body(self) -> BodyStream;
}

use futures_util::stream;

impl IntoBody for String {
    fn into_body(self) -> BodyStream {
        Box::pin(stream::once(async move {
            Ok(Bytes::from(self))
        }))
    }
}

impl IntoBody for &'static str {
    fn into_body(self) -> BodyStream {
        Box::pin(stream::once(async move {
            Ok(Bytes::from_static(self.as_bytes()))
        }))
    }
}

impl IntoBody for Vec<u8> {
    fn into_body(self) -> BodyStream {
        Box::pin(stream::once(async move {
            Ok(Bytes::from(self))
        }))
    }
}

use tokio::fs::File;
use tokio_util::io::ReaderStream;

impl IntoBody for File {
    fn into_body(self) -> BodyStream {
        let stream = ReaderStream::new(self)
            .map(|res| res.map(Bytes::from));
        Box::pin(stream)
    }
}
```
