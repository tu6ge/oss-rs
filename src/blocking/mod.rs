/*!
# 阻塞模式

## 使用方法

### 初始化配置信息

- 方式一

```rust
use std::env::set_var;
set_var("ALIYUN_KEY_ID", "foo1");
set_var("ALIYUN_KEY_SECRET", "foo2");
set_var("ALIYUN_ENDPOINT", "qingdao");
set_var("ALIYUN_BUCKET", "foo4");
let client = aliyun_oss_client::ClientRc::from_env();
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
let client = aliyun_oss_client::ClientRc::from_env();
```

- 方式三

```rust
let client = aliyun_oss_client::ClientRc::new(
    "key1".into(),
    "secret1".into(),
    "qingdao".into(),
    "my-bucket".into()
);
```

### 查询所有的 bucket 信息

```rust
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
```rust
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
```rust
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

```no_run
# use std::env::set_var;
# set_var("ALIYUN_KEY_ID", "foo1");
# set_var("ALIYUN_KEY_SECRET", "foo2");
# set_var("ALIYUN_ENDPOINT", "qingdao");
# set_var("ALIYUN_BUCKET", "foo4");
# let client = aliyun_oss_client::ClientRc::from_env().unwrap();
let query = vec![("max-keys", "5"), ("prefix", "babel")];

let mut result = client.get_bucket_info().unwrap().get_object_list(query).unwrap();

println!("object list : {:?}", result);

// 翻页功能 获取下一页数据
println!("next object list: {:?}", result.next());
```

### 上传文件
```rust
# use std::env::set_var;
# set_var("ALIYUN_KEY_ID", "foo1");
# set_var("ALIYUN_KEY_SECRET", "foo2");
# set_var("ALIYUN_ENDPOINT", "qingdao");
# set_var("ALIYUN_BUCKET", "foo4");
# let client = aliyun_oss_client::ClientRc::from_env().unwrap();
use aliyun_oss_client::file::blocking::File;
client.put_file("examples/bg2015071010.png", "examples/bg2015071010.png");

// or 上传文件内容
let file_content = std::fs::read("examples/bg2015071010.png").unwrap();
client.put_content(file_content, "examples/bg2015071010.png", |_|Some("image/png"));

// or 自定义上传文件 Content-Type
let file_content = std::fs::read("examples/bg2015071010.png").unwrap();
client.put_content_base(file_content, "image/png", "examples/bg2015071010.png");
```

### 删除文件
```rust
# use std::env::set_var;
# set_var("ALIYUN_KEY_ID", "foo1");
# set_var("ALIYUN_KEY_SECRET", "foo2");
# set_var("ALIYUN_ENDPOINT", "qingdao");
# set_var("ALIYUN_BUCKET", "foo4");
# let client = aliyun_oss_client::ClientRc::from_env().unwrap();
use aliyun_oss_client::file::blocking::File;
client.delete_object("examples/bg2015071010.png");
```
*/

pub mod builder;

use crate::config::Config;
use crate::types::{BucketName, EndPoint, KeyId, KeySecret};

use self::builder::ClientWithMiddleware;

pub fn client<ID, S, E, B>(
    access_key_id: ID,
    access_key_secret: S,
    endpoint: E,
    bucket: B,
) -> crate::client::Client<ClientWithMiddleware>
where
    ID: Into<KeyId>,
    S: Into<KeySecret>,
    E: Into<EndPoint>,
    B: Into<BucketName>,
{
    let config = Config::new(access_key_id, access_key_secret, endpoint, bucket);
    crate::client::Client::<ClientWithMiddleware>::from_config(config)
}
