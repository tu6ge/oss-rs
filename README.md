# aliyun_oss_client 打算采用一种全新的方式来实现

**巨大更新，之前使用过的，慎重升级 0.13**

目前的实现，代码将大大简化，api接口也更加起清晰，使用了有限的 rust 语言特性，所以接口更加统一，
大大减少了外部依赖

> 使用本 lib 创建的 oss 命令行工具：https://github.com/tu6ge/oss-cli，里面用比较实际的一些用法，可供参考

详细使用案例如下：

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

