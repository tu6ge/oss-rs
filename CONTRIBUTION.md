# 贡献者指南

下载代码
```
git clone git@github.com:tu6ge/oss-rs.git
cd oss-rs
```

运行单元测试

```
cargo test --lib --all-features tests
```

运行单元测试 + 集成测试（需要连接aliyun OSS 账号）

```
cargo t -q --all-features
```
