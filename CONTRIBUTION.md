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

## 注意事项

- 请保持 commit 信息的整洁，每次 commit 只做一件事
- commit 信息，请遵守 Angular 规范
- 一次 Pull Request 可以包含多个 commit
- 新增特性时，提供必要的单元测试，也欢迎为现有的代码补充单测
- 新增特性时，请补充必要的文档，预览文档命令：

```
cargo doc --open --no-deps
```

## 感谢您的贡献
