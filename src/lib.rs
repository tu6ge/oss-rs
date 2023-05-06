/*!
# 一个 aliyun OSS 的客户端

## 使用方法

1. 在 `cargo.toml` 中添加如下依赖项

```toml
[dependencies]
aliyun-oss-client = "^0.11"
```

2. 初始化配置信息

- 方式一

```rust
use std::env::set_var;
set_var("ALIYUN_KEY_ID", "foo1");
set_var("ALIYUN_KEY_SECRET", "foo2");
set_var("ALIYUN_ENDPOINT", "qingdao");
set_var("ALIYUN_BUCKET", "foo4");
let client = aliyun_oss_client::Client::from_env();
```

- 方式二

在项目根目录下创建 `.env` 文件（需将其加入 .gitignore ），内容：

```bash
ALIYUN_KEY_ID=xxxxxxx
ALIYUN_KEY_SECRET=yyyyyyyyyyyyyy
ALIYUN_ENDPOINT=https://oss-cn-shanghai.aliyuncs.com
ALIYUN_BUCKET=zzzzzzzzzz
```

在需要使用 OSS 的地方，这样设置：
```rust
use dotenv::dotenv;
dotenv().ok();
let client = aliyun_oss_client::Client::from_env();
```

- 方式三

```rust
use aliyun_oss_client::BucketName;
let bucket = BucketName::new("bbb").unwrap();
let client = aliyun_oss_client::Client::new(
    "key1".into(),
    "secret1".into(),
    "qingdao".try_into().unwrap(),
    bucket
);
```

## 支持内网访问 Version +0.9

~~在阿里云的 ECS 上请求 OSS 接口，使用内网 API 有更高的效率，只需要在 ECS 上设置 `ALIYUN_OSS_INTERNAL` 环境变量为 `true` 即可~~

从 `0.12` 版本开始，只有在 [`Client::from_env`] 和 [`BucketBase::from_env`] 这两个方法中 `ALIYUN_OSS_INTERNAL` 环境变量才起作用，
其他地方，请使用 [`EndPoint::set_internal`] 进行切换

### 查询所有的 bucket 信息

了解更多，请查看 [`get_bucket_list`]

```rust
# #[tokio::main]
# async fn main(){
    # use std::env::set_var;
    # set_var("ALIYUN_KEY_ID", "foo1");
    # set_var("ALIYUN_KEY_SECRET", "foo2");
    # set_var("ALIYUN_ENDPOINT", "qingdao");
    # set_var("ALIYUN_BUCKET", "foo4");
    # let client = aliyun_oss_client::Client::from_env().unwrap();
    client.get_bucket_list().await;
# }
```

### 获取 bucket 信息

了解更多，请查看 [`get_bucket_info`]

```rust
# #[tokio::main]
# async fn main(){
    # use std::env::set_var;
    # set_var("ALIYUN_KEY_ID", "foo1");
    # set_var("ALIYUN_KEY_SECRET", "foo2");
    # set_var("ALIYUN_ENDPOINT", "qingdao");
    # set_var("ALIYUN_BUCKET", "foo4");
    # let client = aliyun_oss_client::Client::from_env().unwrap();
    let response = client.get_bucket_info().await;
    println!("bucket info: {:?}", response);
# }
```

### 查询当前 bucket 中的 object 列表

了解更多，请查看 [`get_object_list`]

查询条件参数有多种方式，具体参考 [`Bucket::get_object_list`] 文档
```rust
# #[tokio::main]
# async fn main(){
    # use std::env::set_var;
    # set_var("ALIYUN_KEY_ID", "foo1");
    # set_var("ALIYUN_KEY_SECRET", "foo2");
    # set_var("ALIYUN_ENDPOINT", "qingdao");
    # set_var("ALIYUN_BUCKET", "foo4");
    # let client = aliyun_oss_client::Client::from_env().unwrap();
    let response = client.get_object_list([]).await;
    println!("objects list: {:?}", response);
# }
```

### 也可以使用 bucket struct 查询 object 列表

了解更多，请查看 [`Bucket::get_object_list`]
```rust,ignore
# #[tokio::main]
# async fn main(){
    # use std::env::set_var;
    # set_var("ALIYUN_KEY_ID", "foo1");
    # set_var("ALIYUN_KEY_SECRET", "foo2");
    # set_var("ALIYUN_ENDPOINT", "qingdao");
    # set_var("ALIYUN_BUCKET", "foo4");
    # let client = aliyun_oss_client::Client::from_env().unwrap();
    let query = [("max-keys".into(), "5".into()), ("prefix".into(), "babel".into())];
    let result = client.get_bucket_info().await.unwrap().get_object_list(query).await;

    println!("object list : {:?}", result);
# }
```

### 上传文件

了解更多，请查看 [`put_file`], [`put_content`], [`put_content_base`]
```rust
# #[tokio::main]
# async fn main(){
    # use std::env::set_var;
    # set_var("ALIYUN_KEY_ID", "foo1");
    # set_var("ALIYUN_KEY_SECRET", "foo2");
    # set_var("ALIYUN_ENDPOINT", "qingdao");
    # set_var("ALIYUN_BUCKET", "foo4");
    # let client = aliyun_oss_client::Client::from_env().unwrap();
    # use aliyun_oss_client::types::object::ObjectPath;

    use aliyun_oss_client::file::Files;
    client.put_file("examples/bg2015071010.png", "examples/bg2015071010.png").await;

    let path: ObjectPath = "examples/bg2015071010.png".parse().unwrap();
    client.put_file("examples/bg2015071010.png", path).await;

    // or 上传文件内容
    let file_content = std::fs::read("examples/bg2015071010.png").unwrap();
    client
        .put_content(file_content, "examples/bg2015071010.png", |_| {
            Some("image/png")
        })
        .await;

    // or 自定义上传文件 Content-Type
    let file_content = std::fs::read("examples/bg2015071010.png").unwrap();
    client
        .put_content_base(file_content, "image/png", "examples/bg2015071010.png")
        .await;
# }
```

### 下载文件

了解更多，请查看 [`get_object`]
```rust
# #[tokio::main]
# async fn main(){
    # use std::env::set_var;
    # set_var("ALIYUN_KEY_ID", "foo1");
    # set_var("ALIYUN_KEY_SECRET", "foo2");
    # set_var("ALIYUN_ENDPOINT", "qingdao");
    # set_var("ALIYUN_BUCKET", "foo4");
    # let client = aliyun_oss_client::Client::from_env().unwrap();
    # use aliyun_oss_client::types::object::ObjectPath;
    use aliyun_oss_client::file::Files;

    // 获取完整文件
    let content = client.get_object("bar.json", ..).await;

    // 获取文件一部分
    let content = client.get_object("bar.json".to_string(), ..100).await;
    let path: ObjectPath = "bar.json".parse().unwrap();
    let content = client.get_object(path, 100..).await;
    let content = client.get_object("bar.json", 100..200).await;
# }
```

### 删除文件

了解更多，请查看 [`delete_object`]
```rust
# #[tokio::main]
# async fn main(){
    # use std::env::set_var;
    # set_var("ALIYUN_KEY_ID", "foo1");
    # set_var("ALIYUN_KEY_SECRET", "foo2");
    # set_var("ALIYUN_ENDPOINT", "qingdao");
    # set_var("ALIYUN_BUCKET", "foo4");
    # let client = aliyun_oss_client::Client::from_env().unwrap();
    use aliyun_oss_client::file::Files;
    client.delete_object("examples/bg2015071010.png").await;
# }
```

[`get_bucket_list`]: crate::client::Client::get_bucket_list
[`get_bucket_info`]: crate::client::Client::get_bucket_info
[`get_object_list`]: crate::client::Client::get_object_list
[`Bucket::get_object_list`]: crate::bucket::Bucket::get_object_list
[`put_file`]: crate::file::Files::put_file
[`put_content`]: crate::file::Files::put_content
[`put_content_base`]: crate::file::Files::put_content_base
[`get_object`]: crate::file::Files::get_object
[`delete_object`]: crate::file::Files::delete_object
[`Client::from_env`]: crate::client::Client::from_env
[`BucketBase::from_env`]: crate::config::BucketBase::from_env
[`EndPoint::set_internal`]: crate::EndPoint::set_internal
*/

#![cfg_attr(all(feature = "bench", test), feature(test))]
#![warn(missing_docs)]
#![warn(clippy::unwrap_used)]

// #![doc(html_playground_url = "https://play.rust-lang.org/")]

#[cfg(all(feature = "bench", test))]
extern crate test;

#[cfg(feature = "auth")]
pub mod auth;

#[cfg(feature = "blocking")]
pub mod blocking;

#[cfg(feature = "core")]
pub mod bucket;

#[cfg(feature = "core")]
#[doc(hidden)]
pub mod builder;

#[cfg(feature = "core")]
pub mod client;

#[cfg(feature = "core")]
pub mod config;

mod consts;

#[cfg(feature = "decode")]
pub mod decode;

#[cfg(feature = "core")]
pub mod errors;

#[cfg(feature = "core")]
pub mod file;

#[cfg(feature = "core")]
pub mod object;

#[cfg(feature = "sts")]
pub mod sts;

pub mod types;

#[cfg(test)]
mod tests;

#[cfg(feature = "core")]
pub use client::ClientArc as Client;
#[cfg(feature = "blocking")]
pub use client::ClientRc;
#[cfg(feature = "core")]
pub use errors::{OssError as Error, OssResult as Result};
#[cfg(feature = "core")]
pub use http::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
#[cfg(feature = "decode")]
pub use oss_derive::DecodeListError;
#[cfg(feature = "core")]
pub use types::{
    object::{ObjectDir, ObjectPath},
    Query, QueryKey, QueryValue,
};
pub use types::{BucketName, EndPoint, KeyId, KeySecret};

#[cfg(all(doctest, not(tarpaulin)))]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

/** # 主要入口

*/
#[cfg(feature = "core")]
pub fn client<ID, S, E, B>(
    access_key_id: ID,
    access_key_secret: S,
    endpoint: E,
    bucket: B,
) -> client::Client<ClientWithMiddleware>
where
    ID: Into<KeyId>,
    S: Into<KeySecret>,
    E: Into<EndPoint>,
    B: Into<BucketName>,
{
    use config::Config;
    let config = Config::new(access_key_id, access_key_secret, endpoint, bucket);
    client::Client::<ClientWithMiddleware>::from_config(config)
}

#[cfg(feature = "core")]
use builder::ClientWithMiddleware;
