/*!
# 一个 aliyun OSS 的客户端

## 使用方法

1. 在自己的项目里添加如下依赖项

```toml
[dependencies]
aliyun-oss-client = "^0.8"
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

在阿里云的 ECS 上请求 OSS 接口，使用内网 API 有更高的效率，只需要在 ECS 上设置 `ALIYUN_OSS_INTERNAL` 环境变量为 `true` 即可

### 查询所有的 bucket 信息

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

查询条件参数有多种方式，具体参考 [`get_object_list`](./bucket/struct.Bucket.html#method.get_object_list) 文档
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
    client.put_file("examples/bg2015071010.png", "examples/bg2015071010.png".parse().unwrap()).await;

    use aliyun_oss_client::file::FileAs;
    client.put_file_as("examples/bg2015071010.png", "examples/bg2015071010.png").await;

    // or 上传文件内容
    let file_content = std::fs::read("examples/bg2015071010.png").unwrap();
    client
        .put_content_as(file_content, "examples/bg2015071010.png", |_| {
            Some("image/png")
        })
        .await;

    // or 自定义上传文件 Content-Type
    let file_content = std::fs::read("examples/bg2015071010.png").unwrap();
    client
        .put_content_base_as(file_content, "image/png", "examples/bg2015071010.png")
        .await;
# }
```

### 下载文件
```rust
# #[tokio::main]
# async fn main(){
    # use std::env::set_var;
    # set_var("ALIYUN_KEY_ID", "foo1");
    # set_var("ALIYUN_KEY_SECRET", "foo2");
    # set_var("ALIYUN_ENDPOINT", "qingdao");
    # set_var("ALIYUN_BUCKET", "foo4");
    # let client = aliyun_oss_client::Client::from_env().unwrap();
    use aliyun_oss_client::file::FileAs;

    // 获取完整文件
    let content = client.get_object_as("bar.json", ..).await;

    // 获取文件一部分
    let content = client.get_object_as("bar.json", ..100).await;
    let content = client.get_object_as("bar.json", 100..).await;
    let content = client.get_object_as("bar.json", 100..200).await;
# }
```

### 删除文件
```rust
# #[tokio::main]
# async fn main(){
    # use std::env::set_var;
    # set_var("ALIYUN_KEY_ID", "foo1");
    # set_var("ALIYUN_KEY_SECRET", "foo2");
    # set_var("ALIYUN_ENDPOINT", "qingdao");
    # set_var("ALIYUN_BUCKET", "foo4");
    # let client = aliyun_oss_client::Client::from_env().unwrap();
    use aliyun_oss_client::file::FileAs;
    client.delete_object_as("examples/bg2015071010.png").await;
# }
```
*/

#![cfg_attr(all(feature = "bench", test), feature(test))]
#![warn(missing_docs)]

// #![doc(html_playground_url = "https://play.rust-lang.org/")]

#[cfg(all(feature = "bench", test))]
extern crate test;

/// 库内置类型的定义模块
#[cfg(any(feature = "core", feature = "auth"))]
pub mod types;
#[cfg(feature = "core")]
use builder::ClientWithMiddleware;
#[cfg(feature = "core")]
use config::Config;

/// 重新导出 http 库的一些方法，便于开发者调用 lib 未提供的 api
#[cfg(feature = "core")]
pub use http::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
#[cfg(feature = "core")]
pub use types::{BucketName, EndPoint, KeyId, KeySecret, Query, QueryKey, QueryValue};

/// # 验证模块
/// 包含了签名验证的一些方法，header 以及参数的封装
#[cfg(feature = "auth")]
pub mod auth;

/// # bucket 操作模块
/// 包含查询账号下的所有bucket ，bucket明细
#[cfg(feature = "core")]
pub mod bucket;

/// # 存储对象模块
/// 包含查询当前 bucket 下所有存储对象的方法
#[cfg(feature = "core")]
pub mod object;

/// OSS 文件相关操作
#[cfg(feature = "core")]
pub mod file;

/// 配置类型
#[cfg(feature = "core")]
pub mod config;

/// # 对 reqwest 进行了简单的封装，加上了 OSS 的签名验证功能
#[cfg(feature = "core")]
pub mod client;
#[cfg(feature = "core")]
pub use client::ClientArc as Client;

#[cfg(feature = "blocking")]
pub use client::ClientRc;

/// 阻塞模式（无需 async await）
#[cfg(feature = "blocking")]
pub mod blocking;

/// 封装了 reqwest::RequestBuilder 模块
#[cfg(feature = "core")]
#[doc(hidden)]
pub mod builder;

/// 解析 aliyun OSS 接口返回的 xml
#[cfg(feature = "decode")]
#[path = "traits.rs"]
pub mod decode;

/// 异常处理模块
#[cfg(feature = "core")]
pub mod errors;

/// 临时访问权限管理服务
#[cfg(feature = "sts")]
pub mod sts;

#[allow(soft_unstable)]
#[cfg(test)]
mod tests;

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
    let config = Config::new(access_key_id, access_key_secret, endpoint, bucket);
    client::Client::<ClientWithMiddleware>::from_config(config)
}
