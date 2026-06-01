# aliyun-oss-object-store

将 [aliyun-oss-client](https://github.com/tu6ge/oss-rs) 适配到 [Apache `object_store`](https://docs.rs/object_store) 0.13 的 `ObjectStore` trait，便于与 DataFusion、Parquet、Delta Lake 等基于统一对象存储抽象的生态集成。

## 安装

```toml
[dependencies]
aliyun-oss-object-store = "0.0.2"
aliyun-oss-client = { version = "0.13", features = ["tokio"] }
object_store = "0.13"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## 快速开始

### 环境变量

设置 `ALIYUN_KEY_ID`、`ALIYUN_KEY_SECRET`、`ALIYUN_ENDPOINT`（区域 ID 或完整 Endpoint URL，与主库一致）：

```rust
let store = AliyunOssObjectStore::try_from_env("my-bucket")?;
```

### 显式传参

```rust
let store = AliyunOssObjectStore::try_new(
    "your-access-key-id",
    "your-access-key-secret",
    "cn-qingdao", // 或 "https://oss-cn-qingdao.aliyuncs.com"
    "my-bucket",
)?;
```

### 使用已有 Client（如 STS）

```rust
use aliyun_oss_client::{Client, EndPoint, Key, Secret};

let client = Client::new_with_sts(key, secret, security_token, endpoint);
let store = AliyunOssObjectStore::try_from_client(client, "my-bucket")?;
```

### 读写示例

```rust
use aliyun_oss_object_store::AliyunOssObjectStore;
use bytes::Bytes;
use object_store::{path::Path, ObjectStoreExt as _, PutPayload};

#[tokio::main]
async fn main() -> object_store::Result<()> {
    let store = AliyunOssObjectStore::try_from_env("my-bucket")?;

    let path = Path::from("data/hello.txt");
    store
        .put(&path, PutPayload::from_bytes(Bytes::from_static(b"hello, oss")))
        .await?;

    let meta = store.head(&path).await?;
    println!("size = {}", meta.size);

    Ok(())
}
```

## 已实现的能力

| `ObjectStore` 方法 | 说明 |
| --- | --- |
| `put_opts` | 上传对象，返回 OSS `ETag`；支持 `ContentType` 属性 |
| `put_multipart_opts` | 分片上传（OSS 分片号从 1 开始） |
| `get_opts` | 读取与 HEAD；支持前置条件检查 |
| `delete_stream` | 并发删除（缓冲 10） |
| `list` | 按前缀列举对象元数据 |
| `list_with_delimiter` | 非递归目录列举（`/` 分隔） |
| `copy_opts` | 桶内拷贝；`CopyMode::Create` 时目标已存在会报错 |

## 错误映射

- OSS `NoSuchKey`（对象操作）→ `object_store::Error::NotFound`
- 其余 OSS 错误 → `Error::Generic { store: "AliyunOssObjectStore", ... }`

初始化请优先使用 `try_from_env`、`try_new` 或 `try_from_client`；若直接调用主库 API，用 `map_oss_error` / `map_oss_error(e.into())` 转换错误。

## 已知限制

- **Range 读取**：非全量 Range 会先下载完整对象再在本地切片，大对象慎用。
- **版本**：不支持带版本的读写（`version` 相关选项会返回 `NotImplemented`）。
- **分片上传选项**：`PutMultipartOptions` 中的 tags / attributes 尚未映射到 OSS。
- **拷贝源路径**：使用 `/{bucket}/{key}` 格式，仅适用于同桶拷贝。

## 与主库的关系

本 crate 是对 `aliyun-oss-client` 的薄适配层，复杂能力（生命周期、ACL、图片处理等）请直接使用主库 API。主库文档见仓库根目录 [README](https://github.com/tu6ge/oss-rs)。

## License

MIT
