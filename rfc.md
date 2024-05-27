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
    async fn get_object(&self, client: &Client) -> Vec<Object> {
        todo!()
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

