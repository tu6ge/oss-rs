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


