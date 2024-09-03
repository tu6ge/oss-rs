# aliyun_oss_client 打算采用一种全新的方式来实现

**巨大更新，之前使用过的，慎重升级 0.13**

目前的实现，代码将大大简化，api 接口也更加起清晰，使用了有限的 rust 语言特性，所以接口更加统一，
大大减少了外部依赖

> 使用本 lib 创建的 oss 命令行工具：[tu6ge/oss-cli](https://github.com/tu6ge/oss-cli)，也算是本库的一个演示项目

详细使用案例如下：

1. 基本操作

```rust
use aliyun_oss_client::{types::ObjectQuery, Client, EndPoint};

async fn run() -> Result<(), aliyun_oss_client::Error> {
    let client = Client::from_env()?;

    // 获取 buckets 列表
    let buckets = client.get_buckets(&EndPoint::CN_QINGDAO).await?;

    // 获取某一个 bucket 的文件列表
    let objects = buckets[0].get_objects(&ObjectQuery::new(), &client).await?;

    // 获取文件的详细信息
    let obj_info = objects[0].get_info(&client).await?;

    let object = Object::new("filename.txt");
    // 上传文件
    let info = object.upload("content".into(), &client).await?;
    // 下载文件内容
    let content = object.download(&client).await?;

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
