# aliyun-oss-client

aliyun OSS 的一个客户端

> 最开始的时候，是作为一个 rust 练手项目，渐渐的现在越来越完善了，包含了 rust 的 struct,enum, async, trait features 等特性，
> 以及自定义 error 类，同时也包含完整的的测试用例

## 使用方法

1. 在自己的项目里添加如下依赖项 (项目遵循语义化版本规则，请放心使用)

```
[dependencies]
aliyun-oss-client = "^0.7"
```

2. 打开你需要使用 oss 的文件，在里面添加如下内容，即可使用：

```
// dotenv 是用于获取配置信息的，可以不使用
extern crate dotenv;
use dotenv::dotenv;
use std::env;

// 需要提供四个配置信息
let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
let bucket      = env::var("ALIYUN_BUCKET").unwrap();

// 获取客户端实例
let client = aliyun_oss_client::client(key_id,key_secret, endpoint, bucket);
```

## 异步

### 查询所有的 bucket 信息
```
let response = client.get_bucket_list().await.unwrap();
println!("buckets list: {:?}", response);
```

### 获取 bucket 信息
```
let response = client.get_bucket_info().await.unwrap();
println!("bucket info: {:?}", response);
```

### 查询当前 bucket 中的 object 列表
```
let query: HashMap<String,String> = HashMap::new();
let response = client.get_object_list(query).await.unwrap();
println!("objects list: {:?}", response);
```

### 也可以使用 bucket struct 查询 object 列表

```
let mut query:HashMap<String,String> = HashMap::new();
query.insert("max-keys".to_string(), "5".to_string());
query.insert("prefix".to_string(), "babel".to_string());

let result = client.get_bucket_info().await.unwrap().get_object_list(query).await.unwrap();

println!("object list : {:?}", result);

```

### 上传文件
```
client.put_file("examples/bg2015071010.png", "examples/bg2015071010.png").expect("上传失败");

// or 上传文件内容
let mut file_content = Vec::new();
std::fs::File::open(file_name)
  .expect("open file failed").read_to_end(&mut file_content)
  .expect("read_to_end failed");
client.put_content(file_content, "examples/bg2015071010.png").await.expect("上传失败");

// or 自定义上传文件 Content-Type
client.put_content_base(file_content, "image/png", "examples/bg2015071010.png")
```

### 删除文件
```
client.delete_object("examples/bg2015071010.png").await.unwrap();
```

## 同步（阻塞模式）

> 如需使用，需要启用 `blocking` 特征

### 获取 client
```
// dotenv 是用于获取配置信息的，可以不使用
extern crate dotenv;
use dotenv::dotenv;
use std::env;

// 需要提供四个配置信息
let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
let bucket      = env::var("ALIYUN_BUCKET").unwrap();

// 获取客户端实例
let client = aliyun_oss_client::blocking::client(key_id,key_secret, endpoint, bucket);
```

### 查询所有的 bucket 信息
```
let response = client.get_bucket_list().unwrap();
println!("buckets list: {:?}", response);
```

### 获取 bucket 信息
```
let response = client.get_bucket_info().unwrap();
println!("bucket info: {:?}", response);
```

### 查询当前 bucket 中的 object 列表
```
let query: HashMap<String,String> = HashMap::new();
let response = client.get_object_list(query).unwrap();
println!("objects list: {:?}", response);
```

### 也可以使用 bucket struct 查询 object 列表

```
let mut query:HashMap<String,String> = HashMap::new();
query.insert("max-keys".to_string(), "5".to_string());
query.insert("prefix".to_string(), "babel".to_string());

let result = client.get_bucket_info().unwrap().get_object_list(query).unwrap();

println!("object list : {:?}", result);

// 翻页功能 获取下一页数据
println!("next object list: {:?}", result.next().unwrap());
```

### 上传文件
```
client.put_file("examples/bg2015071010.png", "examples/bg2015071010.png").expect("上传失败");

// or 上传文件内容
let mut file_content = Vec::new();
std::fs::File::open(file_name)
  .expect("open file failed").read_to_end(&mut file_content)
  .expect("read_to_end failed");
client.put_content(&file_content, "examples/bg2015071010.png").expect("上传失败");

// or 自定义上传文件 Content-Type
client.put_content_base(file_content, "image/png", "examples/bg2015071010.png");
```

### 删除文件
```
client.delete_object("examples/bg2015071010.png").unwrap();
```

## Plugin *已弃用*

> Rust 的类型系统足够好，不需要此插件进行扩展了

插件机制，可以在保持项目本身不变动的情况下，提供更多功能

- 对签名中需要的 `canonicalized_resource` 计算规则进行扩展
- 对上传文件的类型进行扩展

### 扩展文件类型

举个例子 Tauri 打包的升级包的签名文件，不在常用的文件类型中，可以使用如下的扩展，进行使用

```
use aliyun_oss_client::plugin::Plugin;

// 创建一个扩展 struct
struct SigFile;

// 实现 Plugin trait 中的方法，
impl Plugin for SigFile {
    fn name(&self) -> &'static str {
      "sig_file_ext"
    }

    // 这是扩展的初始化方法，在插件挂载时会运行
    // 具体实现，请参考 https://docs.rs/infer/0.9.0 文档
    fn initialize(&mut self, client: &mut Client) -> OssResult<()> {
        let mime_type = "application/pgp-signature";
        let extension = "sig";
        fn m(buf: &[u8]) -> bool {
            return buf.len() >= 3 && buf[0] == 0x64 && buf[1] == 0x57 && buf[2] == 0x35;
        }
        client.infer.add(mime_type, extension, m);
    
        Ok(())
    }
}

// 在 lib 初始化时挂载该插件，
let client_has_plugin = aliyun_oss_client::client("abc", "abc", "abc", "abc")
          .plugin(Box::new(SigFile{})).unwrap();
```

## 与 [官方 client](https://crates.io/crates/oss-rust-sdk) 对比

- 完整的测试用例
- 单一入口，避免泛引入导致意外的命名冲突
- 链式调用
- 对公共的参数进行了封装，每次调用的时候，只需要传递业务参数即可
- 默认支持异步调用，可选的支持同步方式调用
- 支持内置的 object bucket 等结构体
- 支持导出数据到自定义的 object bucket 结构体

