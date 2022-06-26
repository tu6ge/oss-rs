# aliyun-oss-client

aliyun OSS 的一个客户端

> 本不想重复造轮子，但是发现官方提供的 sdk 还有优化的空间，然后就尝试着写一写，就当是练习 rust 的项目了

## 使用方法

1. 在自己的项目里添加如下依赖项 (项目遵循语义化版本规则)

```
[dependencies]
aliyun-oss-client = "^0.3"
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
let client = aliyun_oss_client::client(&key_id,&key_secret, &endpoint, &bucket);
```

## 同步（阻塞模式）

### 查询所有的 bucket 信息
```
let response = client.blocking_get_bucket_list().unwrap();
println!("buckets list: {:?}", response);
```

### 获取 bucket 信息
```
let response = client.blocking_get_bucket_info().unwrap();
println!("bucket info: {:?}", response);
```

### 查询当前 bucket 中的 object 列表
```
let query: HashMap<String,String> = HashMap::new();
let response = client.blocking_get_object_list(query).unwrap();
println!("objects list: {:?}", response);
```

### 也可以使用 bucket struct 查询 object 列表

```
let mut query:HashMap<String,String> = HashMap::new();
query.insert("max-keys".to_string(), "5".to_string());
query.insert("prefix".to_string(), "babel".to_string());

let result = client.blocking_get_bucket_info().unwrap().blocking_get_object_list(query).unwrap();

println!("object list : {:?}", result);

// 翻页功能 获取下一页数据
println!("next object list: {:?}", result.next().unwrap());
```

### 上传文件
```
client.blocking_put_file("examples/bg2015071010.png", "examples/bg2015071010.png").expect("上传失败");

// or 上传文件内容
let mut file_content = Vec::new();
std::fs::File::open(file_name)
  .expect("open file failed").read_to_end(&mut file_content)
  .expect("read_to_end failed");
client.blocking_put_content(&file_content, "examples/bg2015071010.png").expect("上传失败");
```

### 删除文件
```
client.blocking_delete_object("examples/bg2015071010.png").unwrap();

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
client.put_content(&file_content, "examples/bg2015071010.png").await.expect("上传失败");
```

### 删除文件
```
client.delete_object("examples/bg2015071010.png").await.unwrap();

```

## 与 [官方 client](https://crates.io/crates/oss-rust-sdk) 对比

- 完整的测试用例
- 单一入口，避免泛引入导致意外的命名冲突
- 链式调用
- 对公共的参数进行了封装，每次调用的时候，只需要传递业务参数即可
- 默认支持异步调用，可选的支持同步方式调用

