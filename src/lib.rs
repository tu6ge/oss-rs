/*!
# 一个 aliyun OSS 的客户端

## 使用方法

1. 在自己的项目里添加如下依赖项

```ignore
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
```rust
# #[tokio::main]
# async fn main(){
    # use std::env::set_var;
    # set_var("ALIYUN_KEY_ID", "foo1");
    # set_var("ALIYUN_KEY_SECRET", "foo2");
    # set_var("ALIYUN_ENDPOINT", "qingdao");
    # set_var("ALIYUN_BUCKET", "foo4");
    # let client = aliyun_oss_client::Client::from_env().unwrap();
    use aliyun_oss_client::Query;
    let query = Query::new();
    let response = client.get_object_list(query).await;
    println!("objects list: {:?}", response);
# }
```

### 也可以使用 bucket struct 查询 object 列表
```ignore
# #[tokio::main]
# async fn main(){
    # use std::env::set_var;
    # set_var("ALIYUN_KEY_ID", "foo1");
    # set_var("ALIYUN_KEY_SECRET", "foo2");
    # set_var("ALIYUN_ENDPOINT", "qingdao");
    # set_var("ALIYUN_BUCKET", "foo4");
    # let client = aliyun_oss_client::Client::from_env().unwrap();
    use aliyun_oss_client::Query;
    let mut query = Query::new();
    query.insert("max-keys", "5");
    query.insert("prefix", "babel");

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
    client.put_file("examples/bg2015071010.png", "examples/bg2015071010.png").await;

    // or 上传文件内容
    let file_content = std::fs::read("examples/bg2015071010.png").unwrap();
    client.put_content(file_content, "examples/bg2015071010.png").await;

    // or 自定义上传文件 Content-Type
    let file_content = std::fs::read("examples/bg2015071010.png").unwrap();
    client.put_content_base(file_content, "image/png", "examples/bg2015071010.png").await;
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

    // 获取完整文件
    let content = client.get_object("bar.json", ..).await;

    // 获取文件一部分
    let content = client.get_object("bar.json", ..100).await;
    let content = client.get_object("bar.json", 100..).await;
    let content = client.get_object("bar.json", 100..200).await;
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
    client.delete_object("examples/bg2015071010.png").await;
# }
```
*/

// #![feature(test)]

// extern crate test;

/// 库内置类型的定义模块
pub mod types;

use builder::ClientWithMiddleware;
use config::Config;
pub use types::{BucketName, EndPoint, KeyId, KeySecret, Query};

/// # 验证模块
/// 包含了签名验证的一些方法，header 以及参数的封装
pub mod auth;

/// # bucket 操作模块
/// 包含查询账号下的所有bucket ，bucket明细
pub mod bucket;

/// # 存储对象模块
/// 包含查询当前 bucket 下所有存储对象的方法
pub mod object;

pub mod config;

/// # 对 reqwest 进行了简单的封装，加上了 OSS 的签名验证功能
pub mod client;
pub use client::ClientArc as Client;

#[cfg(feature = "blocking")]
pub use client::ClientRc;

/// 阻塞模式（无需 async await）
#[cfg(feature = "blocking")]
pub mod blocking;

/// 封装了 reqwest::RequestBuilder 模块
pub mod builder;

/// 定义 trait 们
pub mod traits;

/// 异常处理模块
pub mod errors;

/// 临时访问权限管理服务
#[cfg(feature = "sts")]
pub mod sts;

#[allow(soft_unstable)]
#[cfg(test)]
mod tests;

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

/** # 主要入口

*/
pub fn client<ID, S>(
    access_key_id: ID,
    access_key_secret: S,
    endpoint: EndPoint,
    bucket: BucketName,
) -> client::Client<ClientWithMiddleware>
where
    ID: Into<KeyId>,
    S: Into<KeySecret>,
{
    let config = Config::new(access_key_id, access_key_secret, endpoint, bucket);
    client::Client::<ClientWithMiddleware>::from_config(&config)
}
