## 2022-09-04, Version 0.7.5
### Commits
- [[`70b2f8cc26`](https://github.com/tu6ge/oss/commit/70b2f8cc269068ae491a0d02fe80da5543ca1b6c)] chore(version) (tu6ge)
- [[`5ace49ae76`](https://github.com/tu6ge/oss/commit/5ace49ae76a0935031b14d3bb28c5286d5311c87)] chore(changelog) (tu6ge)
- [[`ee97d1e845`](https://github.com/tu6ge/oss/commit/ee97d1e845bee0c24caa259fc33374a72bd0220e)] feat(aliyun): 更好的阿里云错误的处理方式 (tu6ge)
- [[`36754c4048`](https://github.com/tu6ge/oss/commit/36754c4048d19c62beb2eaaffc6cb81b7d0d4869)] docs(plugin): 小小的说明修改 (tu6ge)
- [[`6a4fc952c4`](https://github.com/tu6ge/oss/commit/6a4fc952c4aa194d397a846a26b5b4bd245099a4)] docs(plugin): 小小的示例修改 (tu6ge)
- [[`ef461139f7`](https://github.com/tu6ge/oss/commit/ef461139f79eb0ea0baa1dfbb47c7368b2b66d23)] docs(plugin): 在文档中对插件的使用方式做了一个说明 (tu6ge)

### Stats
```diff
 Cargo.toml    |  4 ++-
 Changelog.md  | 20 +++++++++++++++-
 README.md     | 46 ++++++++++++++++++++++++++++++++++-
 src/bucket.rs | 11 ++++----
 src/client.rs | 68 +++++++++++++-------------------------------------
 src/errors.rs | 82 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 src/object.rs | 17 +++++++-----
 src/plugin.rs |  2 +-
 8 files changed, 186 insertions(+), 64 deletions(-)
```


## 2022-09-04, Version 0.7.5
### Commits
- [[`70b2f8cc26`](https://github.com/tu6ge/oss/commit/70b2f8cc269068ae491a0d02fe80da5543ca1b6c)] chore(version) (tu6ge)
- [[`5ace49ae76`](https://github.com/tu6ge/oss/commit/5ace49ae76a0935031b14d3bb28c5286d5311c87)] chore(changelog) (tu6ge)
- [[`ee97d1e845`](https://github.com/tu6ge/oss/commit/ee97d1e845bee0c24caa259fc33374a72bd0220e)] feat(aliyun): 更好的阿里云错误的处理方式 (tu6ge)
- [[`36754c4048`](https://github.com/tu6ge/oss/commit/36754c4048d19c62beb2eaaffc6cb81b7d0d4869)] docs(plugin): 小小的说明修改 (tu6ge)
- [[`6a4fc952c4`](https://github.com/tu6ge/oss/commit/6a4fc952c4aa194d397a846a26b5b4bd245099a4)] docs(plugin): 小小的示例修改 (tu6ge)
- [[`ef461139f7`](https://github.com/tu6ge/oss/commit/ef461139f79eb0ea0baa1dfbb47c7368b2b66d23)] docs(plugin): 在文档中对插件的使用方式做了一个说明 (tu6ge)

### Stats
```diff
 Cargo.toml    |  4 ++-
 Changelog.md  | 20 +++++++++++++++-
 README.md     | 46 ++++++++++++++++++++++++++++++++++-
 src/bucket.rs | 11 ++++----
 src/client.rs | 68 +++++++++++++-------------------------------------
 src/errors.rs | 82 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 src/object.rs | 17 +++++++-----
 src/plugin.rs |  2 +-
 8 files changed, 186 insertions(+), 64 deletions(-)
```


## 2022-09-04, Version 0.7.5
### Commits
- [[`70b2f8cc26`](https://github.com/tu6ge/oss/commit/70b2f8cc269068ae491a0d02fe80da5543ca1b6c)] chore(version) (tu6ge)
- [[`5ace49ae76`](https://github.com/tu6ge/oss/commit/5ace49ae76a0935031b14d3bb28c5286d5311c87)] chore(changelog) (tu6ge)
- [[`ee97d1e845`](https://github.com/tu6ge/oss/commit/ee97d1e845bee0c24caa259fc33374a72bd0220e)] feat(aliyun): 更好的阿里云错误的处理方式 (tu6ge)
- [[`36754c4048`](https://github.com/tu6ge/oss/commit/36754c4048d19c62beb2eaaffc6cb81b7d0d4869)] docs(plugin): 小小的说明修改 (tu6ge)
- [[`6a4fc952c4`](https://github.com/tu6ge/oss/commit/6a4fc952c4aa194d397a846a26b5b4bd245099a4)] docs(plugin): 小小的示例修改 (tu6ge)
- [[`ef461139f7`](https://github.com/tu6ge/oss/commit/ef461139f79eb0ea0baa1dfbb47c7368b2b66d23)] docs(plugin): 在文档中对插件的使用方式做了一个说明 (tu6ge)

### Stats
```diff
 Cargo.toml    |  4 ++-
 Changelog.md  | 20 +++++++++++++++-
 README.md     | 46 ++++++++++++++++++++++++++++++++++-
 src/bucket.rs | 11 ++++----
 src/client.rs | 68 +++++++++++++-------------------------------------
 src/errors.rs | 82 +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 src/object.rs | 17 +++++++-----
 src/plugin.rs |  2 +-
 8 files changed, 186 insertions(+), 64 deletions(-)
```


## 2022-08-21, Version 0.7.3
### Commits
- [[`a54dfd4264`](https://github.com/tu6ge/oss/commit/a54dfd4264d4a221a25ec94ca7a9719489056424)] chore(version): 插件能力升级 (tu6ge)
- [[`f48957a0bc`](https://github.com/tu6ge/oss/commit/f48957a0bc3e5d4ccc31fd672094376838e3d3a8)] feat(plugin): 支持自定义扩展文件类型 (tu6ge)
- [[`8dc3d83381`](https://github.com/tu6ge/oss/commit/8dc3d8338161300b6aa6db4d26f2611f39272c7c)] fix(plugin): 解决在多线程情况下，plugin的问题 (tu6ge)
- [[`a237f2f127`](https://github.com/tu6ge/oss/commit/a237f2f1279a4da9f3186f7dd1338523a446e796)] feat(object): 上传文件的路径支持特殊字符（空格等） (tu6ge)

### Stats
```diff
 Cargo.toml         |  7 +++---
 examples/plugin.rs | 26 ++++++++++++++++++------
 src/bucket.rs      |  2 +-
 src/client.rs      | 35 +++++++++++++++++++++-----------
 src/errors.rs      |  5 ++++-
 src/object.rs      |  7 ++++--
 src/plugin.rs      | 60 ++++++++++++++++++++++++++++++++++++++++++++++++++-----
 7 files changed, 113 insertions(+), 29 deletions(-)
```


## 2022-06-26, Version 0.6.0
### Commits
- [[`fd9dd1a8a3`](https://github.com/tu6ge/oss/commit/fd9dd1a8a31e5cc92d8411799bc400b7deb5cef2)] chore(version): optional deps reqwest blocking (tu6ge)
- [[`440af2dc01`](https://github.com/tu6ge/oss/commit/440af2dc01e5089fb493d5ec2c9f35947eb9b316)] docs(async): 异步的方法去掉前缀 async 改为默认方法 (tu6ge)
- [[`7fbed1941a`](https://github.com/tu6ge/oss/commit/7fbed1941afad74e4b61d8209a6a0e276398a057)] feat(async): 异步的方法去掉前缀 async 改为默认方法 (tu6ge)
- [[`cdcc197050`](https://github.com/tu6ge/oss/commit/cdcc1970504855b034cbe68d466e6993049f8d03)] feat(blocking): reqwest 的 blocking 特征改为可选引用 (tu6ge)
- [[`7def094231`](https://github.com/tu6ge/oss/commit/7def09423198985ed746e390aaf61b82aa7d86e0)] feat(blocking): 同步方法加上 blocking 前缀 (tu6ge)
- [[`2d0679e318`](https://github.com/tu6ge/oss/commit/2d0679e3183caa37579dd07e1ac3c686266bc073)] feat(stream): 尝试实现 stream (tu6ge)
- [[`0ad71eb09b`](https://github.com/tu6ge/oss/commit/0ad71eb09b76027deb867d32a109a73cad27f855)] docs (tu6ge)
- [[`35af9df839`](https://github.com/tu6ge/oss/commit/35af9df839c35cad5464babc7b1ad229721b3b79)] Revert "refactor: wip" (tu6ge)
- [[`29f9d6d1c5`](https://github.com/tu6ge/oss/commit/29f9d6d1c53e29befa30fd3ff95f8a9b5d3d7aa5)] refactor: wip (tu6ge)
- [[`101d8b3edd`](https://github.com/tu6ge/oss/commit/101d8b3edde0612a325d7b9bb3eb464a925ddd74)] docs(async): 异步的文档说明 (tu6ge)
- [[`b794dc7620`](https://github.com/tu6ge/oss/commit/b794dc7620e28ebda1b14cb444a1fe4841d3f23f)] refatcor(deps): update tokio features (tu6ge)
- [[`f12cb27a0c`](https://github.com/tu6ge/oss/commit/f12cb27a0c7871d1c8c5b432e25077c615dc7e99)] feat(sync): 支持异步调用（所有接口） (tu6ge)
- [[`3735da7a57`](https://github.com/tu6ge/oss/commit/3735da7a57b86b32dbd6168e7d8dc41b2ca2a5d7)] style: 删除多余的引用 (tu6ge)

### Stats
```diff
 Cargo.toml              |  11 +++-
 Changelog.md            |  28 +++++++++++-
 README.md               |  69 ++++++++++++++++++++++----
 examples/bucket.rs      |   4 +-
 examples/buckets.rs     |   4 +-
 examples/delete_file.rs |   2 +-
 examples/objects.rs     |   2 +-
 examples/plugin.rs      |   2 +-
 examples/put_file.rs    |   2 +-
 src/auth.rs             |   5 ++-
 src/bucket.rs           |  51 +++++++++++++++++---
 src/client.rs           |  84 +++++++++++++++++++++++++++++++-
 src/errors.rs           |   3 +-
 src/lib.rs              |  19 ++++---
 src/object.rs           | 118 ++++++++++++++++++++++++++++++++++++++++-----
 src/tests.rs            | 128 +-------------------------------------------------
 src/tests_async.rs      | 114 ++++++++++++++++++++++++++++++++++++++++++++-
 src/tests_blocking.rs   | 127 +++++++++++++++++++++++++++++++++++++++++++++++++-
 18 files changed, 596 insertions(+), 177 deletions(-)
```


## 2022-06-19, Version 0.5.0
### Commits
- [[`73dcc85a31`](https://github.com/tu6ge/oss/commit/73dcc85a31c3bc2e6b6ae3c79ae8cd9597027e8a)] chore(version): 插件扩展能力 (tu6ge)
- [[`72b668326d`](https://github.com/tu6ge/oss/commit/72b668326d6918b824d5d93e01d7b4e07c847320)] test(plugin): ignore doctest (tu6ge)
- [[`5957beba65`](https://github.com/tu6ge/oss/commit/5957beba652d3a1bb0debb44e20b15222518b0e1)] docs(plugin): 补充文档信息 (tu6ge)
- [[`bc71a1fadf`](https://github.com/tu6ge/oss/commit/bc71a1fadf67217df601bc273555f3cd887efad7)] feat(plugin): 增加插件的能力 (tu6ge)
- [[`5b6c89450d`](https://github.com/tu6ge/oss/commit/5b6c89450ddcc542a2595e910427ff1a6b51067d)] feat(plugin): 插件可查看 client 结构体内容 (tu6ge)
- [[`f487d7050c`](https://github.com/tu6ge/oss/commit/f487d7050c01e58326f60e7cbb2ddfdc8aabf1d3)] refactor: 去掉无用的代码 (tu6ge)
- [[`502864aacd`](https://github.com/tu6ge/oss/commit/502864aacd0d1de3530b8d95d032fc7329985b0a)] refactor: 去掉无用的代码 (tu6ge)
- [[`f9b9bedd8a`](https://github.com/tu6ge/oss/commit/f9b9bedd8a8e43d7500de7eba365886ea3a48078)] chore(deps): 减少一个不必要的依赖项 (tu6ge)
- [[`fb1ac8fea8`](https://github.com/tu6ge/oss/commit/fb1ac8fea8f969a67f270dc198cad9ab80c98df1)] feat(plugin): 支持插件机制 (tu6ge)
- [[`4324d2d775`](https://github.com/tu6ge/oss/commit/4324d2d775dbef68ba04bd5c6f70681a977a0268)] refactor: 优化代码 (tu6ge)

### Stats
```diff
 Cargo.toml         |   7 ++--
 examples/plugin.rs |  25 ++++++++++++-
 src/auth.rs        |  22 ++++--------
 src/bucket.rs      |   6 +--
 src/client.rs      |  58 +++++++++++++++++++++----------
 src/errors.rs      |  30 ++++++++++++++--
 src/lib.rs         |   3 ++-
 src/object.rs      |   2 +-
 src/plugin.rs      | 102 ++++++++++++++++++++++++++++++++++++++++++++++++++++++-
 9 files changed, 214 insertions(+), 41 deletions(-)
```


## 2022-06-14, Version 0.3.1
### Commits
- [[`43bf2b7d81`](https://github.com/tu6ge/oss/commit/43bf2b7d8137204e629c9b6847cbd29768a122d2)] docs: 完善文档 (tu6ge)
- [[`25f24e3665`](https://github.com/tu6ge/oss/commit/25f24e36654a477b5733ac4110cc1b5f30c332d0)] chore(coverage): disabled (tu6ge)
- [[`da14d19ebf`](https://github.com/tu6ge/oss/commit/da14d19ebf62b491e0f6ac22ec1822ba44fefc69)] chore(coverage) (tu6ge)
- [[`0ec90459c9`](https://github.com/tu6ge/oss/commit/0ec90459c98446a2cd6f50c06ee5e24039738384)] Merge branch 'master' of github.com:tu6ge/oss (tu6ge)
- [[`eb6024ee0d`](https://github.com/tu6ge/oss/commit/eb6024ee0db7031f1a6c33a5dddf15fe4e86e5ba)] docs(object): get_object_list 支持自定义参数 (tu6ge)
- [[`d1451c185b`](https://github.com/tu6ge/oss/commit/d1451c185b2b1690fdb4f3ec5c0896a2b0d66607)] test(object): get_object_list 支持自定义参数 (tu6ge)
- [[`7057e2c011`](https://github.com/tu6ge/oss/commit/7057e2c011249883fb903af9e2875f290c2310e4)] chore(error): 处理 expect 语句 (tu6ge)

### Stats
```diff
 .github/workflows/publish.yml | 34 ++++++++++++++++++++++++++++++++++
 Cargo.toml                    |  2 +-
 README.md                     | 18 +++++++++++++++---
 src/auth.rs                   |  9 ++++-----
 src/errors.rs                 |  4 ++++
 src/lib.rs                    |  3 ++-
 src/object.rs                 | 13 ++++++++++---
 src/tests.rs                  |  5 +++--
 8 files changed, 73 insertions(+), 15 deletions(-)
```


## 2022-06-12, Version 0.3.0
### Commits
- [[`543101d37b`](https://github.com/tu6ge/oss/commit/543101d37b7b72ae7064a1604d1558cf6afaadad)] chore(version): get_object_list 支持自定义参数 (tu6ge)
- [[`f2202c4b98`](https://github.com/tu6ge/oss/commit/f2202c4b9814e380ee06139520b45522eb1a9bbf)] style: remove unused import (tu6ge)
- [[`2e6fb21bf3`](https://github.com/tu6ge/oss/commit/2e6fb21bf35653c33784b9ff111b91ed51a8c50d)] chore(depend): update anyhow version (tu6ge)
- [[`bdde53cdbf`](https://github.com/tu6ge/oss/commit/bdde53cdbf866886d9455be30c6eb4c821e94bb1)] feat(object): 获取object 列表时加上参数支持 (tu6ge)
- [[`082857d0bf`](https://github.com/tu6ge/oss/commit/082857d0bfee62208901007a045f07fd6474ce28)] feat(objects): 接收object 列表接口返回的 next token (tu6ge)
- [[`0d231c34cd`](https://github.com/tu6ge/oss/commit/0d231c34cde60b380971f1c3308209ce105c1261)] chore(depend): update dependencies version fomate (tu6ge)
- [[`55d66c3444`](https://github.com/tu6ge/oss/commit/55d66c3444d07bcf44c28ba1ee48f4f45be1b13c)] chore(action): specify version (tu6ge)
- [[`7808069cfb`](https://github.com/tu6ge/oss/commit/7808069cfb5f541d058d6b73d66dc53b861472c1)] chore: update github action configure (tu6ge)
- [[`d757eab719`](https://github.com/tu6ge/oss/commit/d757eab71911776f46cac099f8782feeeae4de74)] chore: probation cargo-local-install (tu6ge)
- [[`2c8027075a`](https://github.com/tu6ge/oss/commit/2c8027075a9ec469265d986ecafd788d86f08f50)] feat(error): 优化 oss 返回错误的处理方式 (tu6ge)
- [[`1631d9ef4b`](https://github.com/tu6ge/oss/commit/1631d9ef4bd2a9cfe35ed0f180959b4d104b4e7f)] chore(action): test action revert with Dockerfile (tu6ge)
- [[`d54bb92479`](https://github.com/tu6ge/oss/commit/d54bb924797f7face68479d65b4b86cac9e45098)] chore(action): test action with Dockerfile (tu6ge)
- [[`2c1832459a`](https://github.com/tu6ge/oss/commit/2c1832459a1644625f39cc053dbbcdcf1bbcf0e5)] chore(action): test action with Dockerfile (tu6ge)
- [[`a34b0ff071`](https://github.com/tu6ge/oss/commit/a34b0ff07105d737a2d2df150006b80dc7e2d3d6)] style: remove unwrap (tu6ge)
- [[`262b60cb8a`](https://github.com/tu6ge/oss/commit/262b60cb8a5073a030376c44266840b4d2612d98)] feat(error): supplement error handler (tu6ge)

### Stats
```diff
 .github/workflows/publish.yml | 18 +++++++++--
 Cargo.toml                    | 22 +++++++-------
 examples/buckets.rs           |  1 +-
 examples/objects.rs           |  7 ++--
 src/bucket.rs                 | 19 +++---------
 src/client.rs                 | 65 +++++++++++++++++++++++------------------
 src/errors.rs                 |  3 ++-
 src/object.rs                 | 70 +++++++++++++++++++++++++++-----------------
 8 files changed, 121 insertions(+), 84 deletions(-)
```


