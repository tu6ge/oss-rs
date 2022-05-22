/*!
# 一个 aliyun OSS 的客户端

## 使用方法

1. 在自己的项目里添加如下依赖项

```
[dependencies]
aliyun-oss-client = "0.2.0"
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
let response = client.get_object_list().unwrap();
println!("objects list: {:?}", response);
```

### 上传文件
```
client.put_file("examples/bg2015071010.png", "examples/bg2015071010.png").expect("上传失败");

// 上传文件内容
let mut file_content = Vec::new();
std::fs::File::open(file_name)
  .expect("open file failed").read_to_end(&mut file_content)
  .expect("read_to_end failed");
client.put_file(&file_content, "examples/bg2015071010.png").expect("上传失败");
```

### 删除文件
```
client.delete_object("examples/bg2015071010.png").unwrap();

```
 * 
 */

#![feature(test,assert_matches)]
extern crate test;

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

/** # 主要入口

## 简单使用方式为： 
```
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


#[allow(soft_unstable)]
#[cfg(test)]
mod tests {
  use test::Bencher;

  use std::{env, assert_matches::assert_matches};
  use super::*;
  extern crate dotenv;
  use dotenv::dotenv;
  

  #[test]
  fn test_get_bucket_list(){
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = client(&key_id,&key_secret, &endpoint, &bucket);

    let bucket_list = client.get_bucket_list();

    assert_matches!(bucket_list, Ok(_));
  }

  #[test]
  fn test_get_bucket_info(){
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = client(&key_id,&key_secret, &endpoint, &bucket);

    let bucket_list = client.get_bucket_info();

    assert_matches!(bucket_list, Ok(_));
  }


  #[test]
  fn test_get_object() {
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = client(&key_id,&key_secret, &endpoint, &bucket);

    let object_list = client.get_object_list();

    assert_matches!(object_list, Ok(_));
  }

  #[test]
  fn test_put_and_delete_file(){
    dotenv().ok();

    let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
    let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
    let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
    let bucket      = env::var("ALIYUN_BUCKET").unwrap();

    let client = client(&key_id,&key_secret, &endpoint, &bucket);

    let object_list = client.put_file("examples/bg2015071010.png", "examples/bg2015071010.png");

    assert_matches!(object_list, Ok(_));

    let result = client.delete_object("examples/bg2015071010.png");

    assert_matches!(result, Ok(_));
  }

  // #[bench]
  // fn bench_get_object(b: &mut Bencher){
  //   dotenv().ok();

  //   let key_id      = env::var("ALIYUN_KEY_ID").unwrap();
  //   let key_secret  = env::var("ALIYUN_KEY_SECRET").unwrap();
  //   let endpoint    = env::var("ALIYUN_ENDPOINT").unwrap();
  //   let bucket      = env::var("ALIYUN_BUCKET").unwrap();

  //   let client = client::Client::new(&key_id,&key_secret, &endpoint, &bucket);
  //   b.iter(|| {
  //     client.get_object_list();
  //   });
  // }

}
