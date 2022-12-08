# aliyun-oss-client

aliyun OSS 的一个客户端

> 最开始的时候，是作为一个 rust 练手项目，渐渐的现在越来越完善了，包含了 rust 的 struct,enum, async, trait features 等特性，
> 以及自定义 error 类，同时也包含完整的的测试用例

## 使用方法

1. 在自己的项目里添加如下依赖项 (项目遵循语义化版本规则，请放心使用)

```toml
[dependencies]
aliyun-oss-client = "^0.9"
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
let client = aliyun_oss_client::Client::new("key1".into(), "secret1".into(), "qingdao".into(), "my-bucket".into());
```

或者

```rust
let client = aliyun_oss_client::client("key1", "secret1", "qingdao", "my-bucket");
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
let client = aliyun_oss_client::ClientRc::new("key1".into(),"secret1".into(),"qingdao".into(), bucket);
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
use aliyun_oss_client::Query;
let query = Query::new();
let response = client.get_object_list(query);
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
use aliyun_oss_client::Query;
let mut query = Query::new();
query.insert("max-keys", "5");
query.insert("prefix", "babel");

let mut result = client.get_bucket_info().unwrap().get_object_list(query).unwrap();

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

## 与 [官方 client](https://crates.io/crates/oss-rust-sdk) 对比

- 完整的测试用例
- 单一入口，避免泛引入导致意外的命名冲突
- 链式调用
- 对公共的参数进行了封装，每次调用的时候，只需要传递业务参数即可
- 默认支持异步调用，可选的支持同步方式调用
- 支持内置的 object bucket 等结构体
- 支持导出数据到自定义的 object bucket 结构体

## 运行 Bench

```bash
rustup run nightly cargo bench
```

## 生成 Changelog

```
conventional-changelog -p angular -i Changelog.md -s
```
