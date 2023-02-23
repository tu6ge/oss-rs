# 贡献者指南

## 下载代码
```
git clone git@github.com:tu6ge/oss-rs.git
cd oss-rs
```

## 运行单元测试

```
cargo test --lib --all-features tests
```

## 运行单元测试 + 集成测试

1. 需要连接aliyun OSS 账号

复制根目录下的 `.env-example` 文件为 `.env` 并在里面填充 aliyun 的配置信息

2. 运行测试

```
cargo t -q --all-features
```
