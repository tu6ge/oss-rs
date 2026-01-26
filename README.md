# aliyun_oss_client 打算采用一种全新的方式来实现

**巨大更新，之前使用过的，慎重升级 0.13**

目前的实现，代码将大大简化，api 接口也更加起清晰，使用了有限的 rust 语言特性，所以接口更加统一，
大大减少了外部依赖

> 使用本 lib 创建的 oss 命令行工具：[tu6ge/oss-cli](https://github.com/tu6ge/oss-cli)，也算是本库的一个演示项目

详细使用案例如下：

1. 基本操作

```rust
use aliyun_oss_client::{types::ObjectQuery, Client, EndPoint};
use futures_util::StreamExt;

async fn run() -> Result<(), aliyun_oss_client::Error> {
    let client = Client::from_env()?;

    // 获取 buckets 列表
    let buckets = client.get_buckets(&EndPoint::CN_QINGDAO).await?;

    // 获取文件列表
    // 接口每次请求只读取5个文件，随着 next() 函数的不断调用，每隔五个会重新调用一次接口，获取下一页的文件
    let stream = Client::from_env()?
        .bucket("honglei123")?
        .max_keys(5)
        .objects_into_stream();

    pin_mut!(stream);

    while let Some(item) = stream.next().await {
        println!("{item:?}");
    }

    // 获取文件的详细信息
    let obj_info = objects[0].get_info(&client).await?;

    // 上传文件
    let res = Client::from_env()?
        .bucket("honglei123")?
        .object("abc2.txt")
        .content_type("text/plain;charset=utf-8")
        .upload("aaab")
        .await?;

    // 使用文件句柄上传文件
    let mut f = tokio::fs::File::open("example_file.txt").await?;
    let info = Client::from_env()?
        .bucket("honglei123")?
        .object("abc_file.txt")
        .content_type("text/plain;charset=utf-8")
        .upload(f)
        .await?;

     // 使用目录上传文件
    let res = Client::from_env()?
        .bucket("honglei123")?
        .object("abc2.txt")
        .content_type("text/plain;charset=utf-8")
        .upload_file("local.txt")
        .await?;

    // 下载文件到文件句柄
    // 使用流式下载，支持边下载边解压/压缩
    let mut file = tokio::fs::File::create("aaa.txt").await?;
    let res = Client::from_env()?
        .bucket("honglei123")?
        .object("download1.jpg")
        .download(&mut file)
        .await?;

    //下载文件到指定目录
    let res = Client::from_env()?
        .bucket("honglei123")?
        .object("download1.jpg")
        .download_to_file("local.jpg")
        .await?;

    //获取下载文件的 Vec<u8> 内容
    let content = Client::from_env()?
        .bucket("honglei123")?
        .object("download1.jpg")
        .download_to_bytes()
        .await?;

    //获取下载文件的 String 内容
    let content = Client::from_env()?
        .bucket("honglei123")?
        .object("download1.jpg")
        .download_to_string()
        .await?;

    // 复制文件
    let res = Client::from_env()?
        .bucket("honglei123")?
        .object("new_file.txt")
        .copy_source("/bucket_name/source_file.txt")
        .content_type("text/plain;charset=utf-8")
        .copy()
        .await?;

    // 分片上传（大文件）
    let result = PartsUpload::new("myvideo23.mov")
        .file_path("./video.mov".into())
        .upload(&client)
        .await;

    // 删除文件
    let result = Client::from_env()?
        .bucket("honglei123")?
        .object("abc.txt")
        .delete()
        .await;

    Ok(())
}
```

2. 导出 bucket 到自定义类型

```rust
#[derive(Debug, Deserialize)]
struct MyBucket {
    Comment: String,
    CreationDate: String,
    ExtranetEndpoint: EndPoint,
    IntranetEndpoint: String,
    Location: String,
    Name: String,
    Region: String,
    StorageClass: String,
}

let list: Vec<MyBucket> = client.export_buckets(&EndPoint::CN_QINGDAO).await?;
```

3. 导出 bucket 详细信息到自定义类型

```rust
#[derive(Debug, Deserialize)]
struct MyBucketInfo {
    Name: String,
}
let res: MyBucketInfo = buckets[0].export_info(&client).await?;
```

4. 导出 object 列表到自定义

```rust
let condition = {
    let mut map = ObjectQuery::new();
    map.insert(ObjectQuery::MAX_KEYS, "5");
    map
};

#[derive(Debug, Deserialize)]
struct MyObject {
    Key: String,
}

let (list, next_token): (Vec<MyObject>, _) =
    buckets[0].export_objects(&condition, &client).await?;
```

# RFC

get Buckets

```rust
struct Client {
   key: String,
   secret: String,
}

impl Client {
    async fn get_buckets(&self, endpoint: EndPoint) -> Vec<Bucket> {
        todo!()
    }

    // 导出到自定义的类型
    pub async fn export_buckets<B: DeserializeOwned>(
        &self,
        endpoint: &EndPoint,
    ) -> Result<Vec<B>, OssError> {
        //...
    }
}
```

get bucket info;

```rust
struct Bucket {
    name: String,
    endpoint: EndPoint,
}
impl Bucket{
    async fn get_info(&self, client: &Client) -> BucketInfo {
        todo!()
    }

    // 导出到自定义的类型
    pub async fn export_info<B: DeserializeOwned>(&self, client: &Client) -> Result<B, OssError> {
        //...
    }

    async fn get_object(&self, client: &Client) -> Vec<Object> {
        todo!()
    }

    // 导出到自定义的类型
    pub async fn export_objects<Obj: DeserializeOwned>(
        &self,
        query: &ObjectQuery,
        client: &Client,
    ) -> Result<(Vec<Obj>, NextContinuationToken), OssError> {
        //...
    }
}
```

get object info

```rust
struct Object {
    bucket: Bucket,
    path: ObjectPath,
}
impl Object {
    async fn get_info(&self, client: &Client) -> ObjectInfo {
        todo!()
    }

    async fn upload(&self, client: &Client, content: Vec<u8>) -> Result<(), Error>{
        todo!()
    }
    async fn download(&self, client: &Client) -> Result<Vec<u8>, Error>{
        todo!()
    }
}
```
