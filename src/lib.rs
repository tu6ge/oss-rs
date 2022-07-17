/*!
# 一个 aliyun OSS 的客户端

## 使用方法

1. 在自己的项目里添加如下依赖项

```ignore
[dependencies]
aliyun-oss-client = "0.2.0"
```

2. 打开你需要使用 oss 的文件，在里面添加如下内容，即可使用：

```ignore
// dotenv 是用于获取配置信息的，可以不使用
extern crate dotenv;
use dotenv::dotenv;
use std::env;

# fn main() -> Result<(), aliyun_oss_client::errors::OssError> {

// 需要提供四个配置信息
let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
let bucket      = env::var("ALIYUN_BUCKET").unwrap();

// 获取客户端实例
let client = aliyun_oss_client::client(&key_id,&key_secret, &endpoint, &bucket);
# Ok(())
# }
```


### 查询所有的 bucket 信息
```ignore
let response = client.blocking_get_bucket_list().unwrap();
println!("buckets list: {:?}", response);
```

### 获取 bucket 信息
```ignore
let response = client.blocking_get_bucket_info().unwrap();
println!("bucket info: {:?}", response);
```

### 查询当前 bucket 中的 object 列表
```ignore
let query: HashMap<String,String> = HashMap::new();
let result = client.blocking_get_object_list(query).unwrap();

println!("object list : {:?}", result);

// 翻页功能 获取下一页数据
println!("next object list: {:?}", result.next().unwrap());
```

### 上传文件
```ignore
client.blocking_put_file("examples/bg2015071010.png", "examples/bg2015071010.png").expect("上传失败");

// 上传文件内容
let mut file_content = Vec::new();
std::fs::File::open(file_name)
  .expect("open file failed").read_to_end(&mut file_content)
  .expect("read_to_end failed");
client.blocking_put_content(&file_content, "examples/bg2015071010.png").expect("上传失败");
```

### 删除文件
```ignore
client.blocking_delete_object("examples/bg2015071010.png").unwrap();

```
 * 
 */


/// # 验证模块
/// 包含了签名验证的一些方法，header 以及参数的封装
pub mod auth;

/// # bucket 操作模块
/// 包含查询账号下的所有bucket ，bucket明细
pub mod bucket;

/// # 存储对象模块
/// 包含查询当前 bucket 下所有存储对象的方法
pub mod object;

/// # 对 reqwest 进行了简单的封装，加上了 OSS 的签名验证功能
pub mod client;

/// 定义 trait 们
pub mod traits;

pub mod errors;

#[cfg(feature = "plugin")]
pub mod plugin;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

#[allow(soft_unstable)]
#[cfg(test)]
mod tests;

/** # 主要入口

## 简单使用方式为： 
```ignore
let result = aliyun_oss_client::client("key_id_xxx","key_secret_xxxx", "my_endpoint", "my_bucket");
```

## 推荐的使用方式为

1. 使用 cargo 安装 dotenv 

2. 在项目根目录创建 .env 文件，并添加 git 忽略，

然后在 .env 文件中填入阿里云的配置信息
```ignore
ALIYUN_KEY_ID=key_id_xxx
ALIYUN_KEY_SECRET=key_secret_xxxx
ALIYUN_ENDPOINT=my_endpoint
ALIYUN_BUCKET=my_bucket
```

3. 在自己项目里写入如下信息

```ignore
extern crate dotenv;
use dotenv::dotenv;
use std::env;

// 需要提供四个配置信息
let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
let bucket      = env::var("ALIYUN_BUCKET").unwrap();

let result = aliyun_oss_client::client(&key_id,&key_secret, &endpoint, &bucket);
```
*/
pub fn client<'a>(access_key_id: &'a str, access_key_secret: &'a str, endpoint: &'a str, bucket: &'a str) -> client::Client<'a>{
  client::Client::new(access_key_id,access_key_secret, endpoint, bucket)
}

