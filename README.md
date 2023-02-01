# aliyun-oss-client

aliyun OSS 的一个异步/同步客户端，包含以下功能：

- `auth` 模块，处理 OSS 验证，可以抽离出来，独立与 `reqwest` 库配合使用
- `traits` 模块，包含 OSS 接口返回的原始 `xml` 数据的解析方式，可以将数据方便的导入到自定义的 rust 类型中，可以独立使用
- `client` 模块，基础部分，封装了 `reqwest` `auth` 模块，并提供了一些便捷方法
- `bucket` 模块，包含 bucket 以及其列表的结构体
- `object` 模块，包含 object 以及其列表的结构体
- `file` 模块，文件上传，下载，删除等功能，可在 client, bucket, object 等结构体中复用
- `config` 模块，OSS 配置信息，可用于从数据库读取配置等操作

[![Coverage Status](https://coveralls.io/repos/github/tu6ge/oss-rs/badge.svg?branch=master)](https://coveralls.io/github/tu6ge/oss-rs?branch=master) [![Test and Publish](https://github.com/tu6ge/oss-rs/actions/workflows/publish.yml/badge.svg)](https://github.com/tu6ge/oss-rs/actions/workflows/publish.yml)

## 使用方法

1. 在自己的项目里添加如下依赖项 (项目遵循语义化版本规则，请放心使用)

```toml
[dependencies]
aliyun-oss-client = "[last_version]"
```

2. 打开你需要使用 oss 的文件，在里面添加如下内容，即可使用：

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
let client = aliyun_oss_client::Client::new("key1".into(), "secret1".into(), "qingdao".parse().unwrap(), "my-bucket".parse().unwrap());
```

或者

不推荐使用

```rust
// let client = aliyun_oss_client::client("key1", "secret1", "qingdao", "my-bucket");
```

## 支持内网访问 Version +0.9

在阿里云的 ECS 上请求 OSS 接口，使用内网 API 有更高的效率，只需要在 ECS 上设置 `ALIYUN_OSS_INTERNAL` 环境变量为 `true` 即可

## 异步

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
    let response = client.get_object_list([]).await;
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
    use aliyun_oss_client::QueryKey;

    let result = client.get_bucket_info().await.unwrap().get_object_list([
        (QueryKey::MaxKeys, 5u8.into()),
        (QueryKey::Prefix, "babel".into()),
    ]).await;

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
    use aliyun_oss_client::file::File;
    client
        .put_file("examples/bg2015071010.png", "examples/bg2015071010.png")
        .await;

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
    use aliyun_oss_client::file::File;
    client.delete_object("examples/bg2015071010.png").await;
# }
```

## 同步（阻塞模式）

> 如需使用，需要启用 `blocking` 特征

### 获取 client
```ignore
// dotenv 是用于获取配置信息的，可以不使用
extern crate dotenv;
use dotenv::dotenv;
use std::env;

// 需要提供四个配置信息
use aliyun_oss_client::BucketName;
let bucket = BucketName::new("bbb").unwrap();
// 获取客户端实例
let client = aliyun_oss_client::ClientRc::new("key1".into(), "secret1".into(), "qingdao".into(), bucket);
```

### 查询所有的 bucket 信息
```ignore
# use std::env::set_var;
# set_var("ALIYUN_KEY_ID", "foo1");
# set_var("ALIYUN_KEY_SECRET", "foo2");
# set_var("ALIYUN_ENDPOINT", "qingdao");
# set_var("ALIYUN_BUCKET", "foo4");
# let client = aliyun_oss_client::ClientRc::from_env().unwrap();
let response = client.get_bucket_list();
println!("buckets list: {:?}", response);
```

### 获取 bucket 信息
```ignore
# use std::env::set_var;
# set_var("ALIYUN_KEY_ID", "foo1");
# set_var("ALIYUN_KEY_SECRET", "foo2");
# set_var("ALIYUN_ENDPOINT", "qingdao");
# set_var("ALIYUN_BUCKET", "foo4");
# let client = aliyun_oss_client::ClientRc::from_env().unwrap();
let response = client.get_bucket_info();
println!("bucket info: {:?}", response);
```

### 查询当前 bucket 中的 object 列表
```ignore
# use std::env::set_var;
# set_var("ALIYUN_KEY_ID", "foo1");
# set_var("ALIYUN_KEY_SECRET", "foo2");
# set_var("ALIYUN_ENDPOINT", "qingdao");
# set_var("ALIYUN_BUCKET", "foo4");
# let client = aliyun_oss_client::ClientRc::from_env().unwrap();
let response = client.get_object_list([]);
println!("objects list: {:?}", response);
```

### 也可以使用 bucket struct 查询 object 列表

```ignore
# use std::env::set_var;
# set_var("ALIYUN_KEY_ID", "foo1");
# set_var("ALIYUN_KEY_SECRET", "foo2");
# set_var("ALIYUN_ENDPOINT", "qingdao");
# set_var("ALIYUN_BUCKET", "foo4");
# let client = aliyun_oss_client::ClientRc::from_env().unwrap();
use aliyun_oss_client::QueryKey;

let mut result = client.get_bucket_info().unwrap().get_object_list([
    (QueryKey::MaxKeys, 5u8.into()),
    (QueryKey::Prefix, "babel".into()),
]).unwrap();

println!("object list : {:?}", result);

// 翻页功能 获取下一页数据
println!("next object list: {:?}", result.next());
```

### 上传文件
```ignore
# use std::env::set_var;
# set_var("ALIYUN_KEY_ID", "foo1");
# set_var("ALIYUN_KEY_SECRET", "foo2");
# set_var("ALIYUN_ENDPOINT", "qingdao");
# set_var("ALIYUN_BUCKET", "foo4");
# let client = aliyun_oss_client::ClientRc::from_env().unwrap();
use aliyun_oss_client::file::blocking::File;
client
    .put_file("examples/bg2015071010.png", "examples/bg2015071010.png");

// or 上传文件内容
let file_content = std::fs::read("examples/bg2015071010.png").unwrap();
client
    .put_content(file_content, "examples/bg2015071010.png", |_| {
        Some("image/png")
    });

// or 自定义上传文件 Content-Type
let file_content = std::fs::read("examples/bg2015071010.png").unwrap();
client
    .put_content_base(file_content, "image/png", "examples/bg2015071010.png");
```

### 删除文件
```ignore
# use std::env::set_var;
# set_var("ALIYUN_KEY_ID", "foo1");
# set_var("ALIYUN_KEY_SECRET", "foo2");
# set_var("ALIYUN_ENDPOINT", "qingdao");
# set_var("ALIYUN_BUCKET", "foo4");
# let client = aliyun_oss_client::ClientRc::from_env().unwrap();
use aliyun_oss_client::file::blocking::File;
client.delete_object("examples/bg2015071010.png");
```

## 运行 Bench

```bash
rustup run nightly cargo bench
```

## 生成 Changelog

```bash
conventional-changelog -p angular -i Changelog.md -s
```

## 贡献代码

欢迎各种 PR 贡献，[贡献者指南](https://github.com/tu6ge/oss-rs/blob/master/CONTRIBUTION.md)
