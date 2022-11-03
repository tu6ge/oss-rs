/*!
# 一个 aliyun OSS 的客户端

## 使用方法

1. 在自己的项目里添加如下依赖项

```ignore
[dependencies]
aliyun-oss-client = "0.2.0"
```

2. 初始化配置信息

```rust
use std::env::set_var;
set_var("ALIYUN_KEY_ID", "foo1");
set_var("ALIYUN_KEY_SECRET", "foo2");
set_var("ALIYUN_ENDPOINT", "qingdao");
set_var("ALIYUN_BUCKET", "foo4");
let client = aliyun_oss_client::Client::from_env();
```

或者

```rust
use aliyun_oss_client::BucketName;
let bucket = BucketName::new("bbb").unwrap();
let client = aliyun_oss_client::Client::new("key1".into(),"secret1".into(),"qingdao".try_into().unwrap(), bucket);
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
 *
 */

// #![feature(test)]

// extern crate test;

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

#[cfg(feature = "blocking")]
pub mod blocking;

pub mod builder;

/// 定义 trait 们
pub mod traits;

pub mod errors;

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
