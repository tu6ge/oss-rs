# aliyun-oss-client

aliyun OSS 的一个异步/同步客户端，包含以下功能：

- `auth` 模块，处理 OSS 验证，可以抽离出来，独立与 `reqwest` 库配合使用
- `traits` 模块，包含 OSS 接口返回的原始 `xml` 数据的解析方式，可以将数据方便的导入到自定义的 rust 类型中，可以独立使用
- `client` 模块，基础部分，封装了 `reqwest` `auth` 模块，并提供了一些便捷方法
- `bucket` 模块，包含 bucket 以及其列表的结构体
- `object` 模块，包含 object 以及其列表的结构体
- `file` 模块，文件上传，下载，删除等功能，可在 client, bucket, object 等结构体中复用
- `config` 模块，OSS 配置信息，可用于从数据库读取配置等操作

[![Coverage Status](https://coveralls.io/repos/github/tu6ge/oss-rs/badge.svg?branch=master)](https://coveralls.io/github/tu6ge/oss-rs?branch=master) [![Test and Publish](https://github.com/tu6ge/oss-rs/actions/workflows/publish.yml/badge.svg)](https://github.com/tu6ge/oss-rs/actions/workflows/publish.yml)

## 使用方法

[查看文档](https://docs.rs/aliyun-oss-client/)

## 运行 Bench

```bash
rustup run nightly cargo bench
```

## 生成 Changelog

```bash
conventional-changelog -p angular -i Changelog.md -s
```

## 贡献代码

欢迎各种 PR 贡献，[贡献者指南](https://github.com/tu6ge/oss-rs/blob/master/CONTRIBUTION.md)
