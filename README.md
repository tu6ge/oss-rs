# aliyun-oss-client

aliyun OSS 的一个客户端

> 本不想重复造轮子，但是发现官方提供的 sdk 还有优化的空间，然后就尝试着写一写，就当是练习 rust 的项目了

## 使用方法

1. 在自己的项目里添加如下依赖项

```
[dependencies]
aliyun-oss-client = "0.1.0"
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

// 查询所有的 bucket 信息
let response = client.get_bucket_list().unwrap();
println!("buckets list: {:?}", response);

// 获取 bucket 信息
let response = client.get_bucket_info().unwrap();
println!("bucket info: {:?}", response);

// 查询当前 bucket 中的 object 列表
let response = client.get_object_list().unwrap();
println!("objects list: {:?}", response);

// 上传文件
client.put_file("examples/bg2015071010.png", "examples/bg2015071010.png").expect("上传失败");

// 上传文件内容
let mut file_content = Vec::new();
std::fs::File::open(file_name)
  .expect("open file failed").read_to_end(&mut file_content)
  .expect("read_to_end failed");
client.put_file(&file_content, "examples/bg2015071010.png").expect("上传失败");

```