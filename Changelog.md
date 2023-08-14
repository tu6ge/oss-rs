##  (2023-08-14)

### [0.12.7](https://github.com/tu6ge/oss-rs/compare/0.12.6...0.12.7) (2023-08-11)

### [0.12.6](https://github.com/tu6ge/oss-rs/compare/0.12.5...0.12.6) (2023-08-11)


### Features

* **file:** more trait impl ([8d7f64a](https://github.com/tu6ge/oss-rs/commit/8d7f64a2d608b1b8b06ac4d0019b7cd18c6f9077))
* **types:** add convert ([5459d58](https://github.com/tu6ge/oss-rs/commit/5459d58e29247cd42866b8ccd5f4a8f920072df1))


### Bug Fixes

* **types:** impl Send Sync ([292a44d](https://github.com/tu6ge/oss-rs/commit/292a44d909515c2a5538ab73e06d4509ab26da96))

### [0.12.5](https://github.com/tu6ge/oss-rs/compare/0.12.4...0.12.5) (2023-07-19)

### [0.12.4](https://github.com/tu6ge/oss-rs/compare/0.12.3...0.12.4) (2023-07-18)


### Features

* **client:** Reduce constraints on generic param ([400d928](https://github.com/tu6ge/oss-rs/commit/400d928fd7d39068baca19f7275e5ba18211c314))
* **object:** add help fn in objects ([4d1ac3f](https://github.com/tu6ge/oss-rs/commit/4d1ac3f738598685fe350b01d01e99931baca5ea))


### Bug Fixes

* **types:** changed BucketName default fn ([2ea507c](https://github.com/tu6ge/oss-rs/commit/2ea507c7ca7ba57486e24ebb2217ac50d415139e))

### [0.12.3](https://github.com/tu6ge/oss-rs/compare/0.12.2...0.12.3) (2023-06-26)


### Features

* **deps:** upgrade quick-xml.infer ([1d9924c](https://github.com/tu6ge/oss-rs/commit/1d9924caf39471ee8a3bf030d90beb2388b541fb))

### [0.12.2](https://github.com/tu6ge/oss-rs/compare/0.12.1...0.12.2) (2023-06-26)


### Bug Fixes

* **types:** debug KeySecret hidden content ([4956412](https://github.com/tu6ge/oss-rs/commit/4956412a8a99d285342f6d8b426ee8d5d8b4ccf0))

### [0.12.1](https://github.com/tu6ge/oss-rs/compare/0.12.0...0.12.1) (2023-06-08)


### Features

* **auth:** 在 Url 中包含签名 ([#21](https://github.com/tu6ge/oss-rs/issues/21)) ([a15e644](https://github.com/tu6ge/oss-rs/commit/a15e64486769004f42bedbb2729e6cf5e530e14a)), closes [#20](https://github.com/tu6ge/oss-rs/issues/20)

##  (2023-06-01)

## [0.12.0](https://github.com/tu6ge/oss-rs/compare/0.11.2...0.12.0) (2023-06-01)


### ⚠ BREAKING CHANGES

* **object:** object 构建器调整
* **object:** object 构建器调整
* **decode:** remove set_next_continuation_token
* **error:** set field to private
* **types:** change QueryKey to struct
* **types:** change EndPoint to struct
* **decode:** remove ItemError trait
* **types:** change Query insert method
* **decode:** next_token remove Option warp

### Features

* **auth:** 为 Request 附加 with_oss 方法 ([114c1e0](https://github.com/tu6ge/oss-rs/commit/114c1e0f2e90cfe4e458631ab0a63765c5c9fe8c))
* **auth:** change AuthError to struct ([a4b7962](https://github.com/tu6ge/oss-rs/commit/a4b79626d400cc7b650676b11a6d4e148ce2a4fc))
* **auth:** change with_oss args type ([21820c7](https://github.com/tu6ge/oss-rs/commit/21820c7439c2ca1755681a346fc6e0b9735f6e87))
* **auth:** set OssHost trait to private ([2befdb5](https://github.com/tu6ge/oss-rs/commit/2befdb538f0de5920a795ace2296146d3c0650a0))
* **auth:** set_sensitive with secret ([7a02e84](https://github.com/tu6ge/oss-rs/commit/7a02e840597b6a5e7eddfa7ca1af7da795b08f41))
* **bucket:** add to_vec method in blocking ([2d63089](https://github.com/tu6ge/oss-rs/commit/2d63089f6d5bae6dd5ee8d369d9ff26ac04a536f))
* **bucket:** change BucketError to struct ([70dfe5b](https://github.com/tu6ge/oss-rs/commit/70dfe5bfd0d8cd2ef30affaf94a21df39661083f))
* **bucket:** change ExtractItemError to struct ([ff317e9](https://github.com/tu6ge/oss-rs/commit/ff317e915893456d4a78ecb6db5f52203ea08123))
* **builder:** add Box in Error enum ([5117b52](https://github.com/tu6ge/oss-rs/commit/5117b5272a346629f5ae528d65403317fee280ae))
* **builder:** change BuilderError to struct ([cd7f123](https://github.com/tu6ge/oss-rs/commit/cd7f123e7f8aec46c97f871e6e88899c0ee1d581))
* **config:** add set_internal in BucketBase ([576de60](https://github.com/tu6ge/oss-rs/commit/576de60b980612583f025f6369f086fe2aa3f067))
* **config:** change InvalidConfig to struct ([c80355d](https://github.com/tu6ge/oss-rs/commit/c80355dce74e73e0ec20bbd7078c34ff308f67ea))
* **config:** fixed InvalidConfig display info ([7f31892](https://github.com/tu6ge/oss-rs/commit/7f31892fa22da97d9968f80a07f682fc04ce133e))
* **core:** decode xml return more info ([0c3170f](https://github.com/tu6ge/oss-rs/commit/0c3170f6a6598ee3857ef28176cffcbff8532240))
* **core:** decoupling in Errors ([9610ad6](https://github.com/tu6ge/oss-rs/commit/9610ad6d9e5aa616800c9f84084e92e59cdba0fc))
* **core:** default datetime value update ([a8c2bed](https://github.com/tu6ge/oss-rs/commit/a8c2bed9b7c2b7f00caaaaef21aa24bc31440c6a))
* **core:** renamed ObjectList, ListBuckets ([8c17aa9](https://github.com/tu6ge/oss-rs/commit/8c17aa96b62148460ebc46c5943cfcd5f7efd3db))
* **decode:** 解析 xml 错误时，返回更多信息 ([c1dad3d](https://github.com/tu6ge/oss-rs/commit/c1dad3d2e93a93a61e1aadaa6d6c2289063d437f))
* **decode:** add Box in Error enum ([985cb45](https://github.com/tu6ge/oss-rs/commit/985cb453b020a3937d050b027b8066add9bec856))
* **decode:** Add non_exhaustive in Error type ([f39f76b](https://github.com/tu6ge/oss-rs/commit/f39f76bfc73d6a6f0ad4fe643b8a9a6c22ccf556))
* **decode:** Add non_exhaustive in InnerListError ([72f33c9](https://github.com/tu6ge/oss-rs/commit/72f33c904a0ebb6ccf69282027fc3b055d699b76))
* **decode:** change enum to struct InnerListError ([5ea1a06](https://github.com/tu6ge/oss-rs/commit/5ea1a06b4eda9c6378c6d09052355216db1fc59c))
* **decode:** change Error in trait ([0dd2c1b](https://github.com/tu6ge/oss-rs/commit/0dd2c1bf916b3e7fd1e1981be87910764808f7ce))
* **decode:** next_token remove Option warp ([d91f473](https://github.com/tu6ge/oss-rs/commit/d91f473eba2c2ea1d7e2285da06d8ac73483b5c3))
* **decode:** remove ItemError trait ([e0c8424](https://github.com/tu6ge/oss-rs/commit/e0c8424d5367deeef5297986f3ba1588b224416c))
* **decode:** remove set_next_continuation_token ([dd005f8](https://github.com/tu6ge/oss-rs/commit/dd005f88dcb9f84319418809be9c7c41a17ac527))
* **decode:** set ListErrorKind to private ([5adfb2a](https://github.com/tu6ge/oss-rs/commit/5adfb2a732f5462866ebea96c3d265cd902e0adc))
* **derive:** remove #[derive(CustomItemError)] ([db6cca3](https://github.com/tu6ge/oss-rs/commit/db6cca3243ec5c1caed2b6931af16c825d91c09a))
* **error:** change OssError to struct ([e089790](https://github.com/tu6ge/oss-rs/commit/e0897909f0b747caefc828f284a5e91bc11000d7))
* **error:** remove Input in Error enum ([e8c6d4e](https://github.com/tu6ge/oss-rs/commit/e8c6d4e10fc170cb77b3aef9840e2499e06e0a83))
* **error:** set field to private ([e0bbc43](https://github.com/tu6ge/oss-rs/commit/e0bbc4398c7bd247db2360e67b04b5ba6d09081a))
* **file:** change FileError to struct ([7fb1a0a](https://github.com/tu6ge/oss-rs/commit/7fb1a0aa0b34033b75f7370174a6421d94dcb6de))
* **file:** change OssError enum item ([a489952](https://github.com/tu6ge/oss-rs/commit/a489952831c7111ea7ad7470f73c0086356520d5))
* **file:** get_object support more num type ([c8c111b](https://github.com/tu6ge/oss-rs/commit/c8c111bd5323979cb738ced9c4f087aa10dccf27))
* **file:** rename BlockingFiles ([25bb539](https://github.com/tu6ge/oss-rs/commit/25bb5395d60acbf00d857f02d74989b037c9a1a0))
* **lint:** remove warning lint ([9915737](https://github.com/tu6ge/oss-rs/commit/9915737e7199e99d2226ac5a92bc0746966d6497))
* **object:** add ListBucketsSync type alias ([8f567ae](https://github.com/tu6ge/oss-rs/commit/8f567aec82e1b3c872e66bd2ca3fc0a5df862957))
* **object:** add new method of StorageClass ([11ebb58](https://github.com/tu6ge/oss-rs/commit/11ebb589cc42a2cb22348092eee5d59a6422708d))
* **object:** add ObjectListSync type ([e8012be](https://github.com/tu6ge/oss-rs/commit/e8012be630e1fc34c007cae60c8ec572fa350f9f))
* **object:** change BuildInItemError to struct ([5a8b4b5](https://github.com/tu6ge/oss-rs/commit/5a8b4b5b33dec4e8de96230f6eb0c5d228f48fd3))
* **object:** change ExtractListError to struct ([8af9d72](https://github.com/tu6ge/oss-rs/commit/8af9d7286d1756b98425a72961e4262158ab9030))
* **object:** change ObjectListError to struct ([d3cfe3c](https://github.com/tu6ge/oss-rs/commit/d3cfe3c14406e82155d694b6b7d85ab8335c0d63))
* **object:** object 构建器调整 ([d693e50](https://github.com/tu6ge/oss-rs/commit/d693e50fe575fba3bcbeac3ca4155900d3f299bc))
* **object:** object 构建器调整 ([5a094b3](https://github.com/tu6ge/oss-rs/commit/5a094b3ea8f6891d628c0c050731ac03bff166dd))
* **object:** set BuildInItemErrorKind private ([6f6b198](https://github.com/tu6ge/oss-rs/commit/6f6b19833b92c88a013d6aa67a229d0baef05b51))
* **object:** set object_list to private in struct ([ba66764](https://github.com/tu6ge/oss-rs/commit/ba667644ff0b57a8eeae8ba9ff2e0803c04a0ace))
* **sts:** set sensitive security-token ([b9dad10](https://github.com/tu6ge/oss-rs/commit/b9dad10af1246891693bb1d1cc9a361532a385ec))
* **type:** change InvalidObjectBase inner ([0e37837](https://github.com/tu6ge/oss-rs/commit/0e37837b3404b89d50ace749d9761fa6e798067f))
* **types:** 从 env 转化配置的时候，处理 endpoint 的情况 ([8e95b37](https://github.com/tu6ge/oss-rs/commit/8e95b3758f63605e2c6a25d9088a2a315ba86da2))
* **types:** 升级 KeySecret 类型的安全性 ([29c44fc](https://github.com/tu6ge/oss-rs/commit/29c44fc4d07aaac38016dadc0f255503c7d12cf7))
* **types:** add non_exhaustive in QueryKey ([2601b03](https://github.com/tu6ge/oss-rs/commit/2601b037efc5332793b9232845605aaa9155058a))
* **types:** Add priv in inner type ([671cb12](https://github.com/tu6ge/oss-rs/commit/671cb12ea54b132897e12767993880e9308e6b23))
* **types:** change EndPoint to struct ([32628ee](https://github.com/tu6ge/oss-rs/commit/32628eec1224f7f8ad9cfa938e05f0e6b185b5c6))
* **types:** change InvalidBucketName display info ([71df709](https://github.com/tu6ge/oss-rs/commit/71df7093407a61818d4c5621ed1f6f5b4cfb6308))
* **types:** change InvalidObjectDir display info ([8da7edf](https://github.com/tu6ge/oss-rs/commit/8da7edf3a30613759e439f1c8991b7b0a26caece))
* **types:** change Query insert method ([8e041c8](https://github.com/tu6ge/oss-rs/commit/8e041c807e66d8dae31fadf91a86e6a3288b9f31))
* **types:** change QueryKey to struct ([12fe959](https://github.com/tu6ge/oss-rs/commit/12fe959ec5a4373e0e86fad05d2fea3de2de8e16))
* **types:** Date only from DateTime ([bdfed8c](https://github.com/tu6ge/oss-rs/commit/bdfed8c3b09f5788b0881c50b719ce96c7288ae4))
* **types:** rename Trait ([4fc1b87](https://github.com/tu6ge/oss-rs/commit/4fc1b87767abf34cc6875767c63f61ab95360301))
* **types:** set ObjectBase::new to pub(crate) ([7746c73](https://github.com/tu6ge/oss-rs/commit/7746c73fe63f24f7170f88bbb471066b29e70424))


### Bug Fixes

* **auth:** 解决模块之间的依赖问题 ([3965c77](https://github.com/tu6ge/oss-rs/commit/3965c77cfe0b6d29e2489601d940e5c89ead9d6b))
* no default features ([98220d4](https://github.com/tu6ge/oss-rs/commit/98220d46d4f0ddf43c39e1100c5e92b492b6cca7))
* **object:** fixed macro error ([c7d4b4e](https://github.com/tu6ge/oss-rs/commit/c7d4b4efc9d12f74cddc74b76ec810dd962077c9))


### Reverts

* Revert "feat(file)!: GetStd GetStdWithPath remove method" ([799cbff](https://github.com/tu6ge/oss-rs/commit/799cbff462b2bec0afbe72a11907ca73dd8971f3))
* Revert "fix(test)" ([9a3a7c5](https://github.com/tu6ge/oss-rs/commit/9a3a7c5356a60cef505b398caacf1e30d1fc9d80))

#  (2023-03-24)



## [0.11.2](https://github.com/tu6ge/oss-rs/compare/0.11.1...0.11.2) (2023-03-24)


### Bug Fixes

* **types:** 解决 Endpoint 匹配错误 ([0a65ad2](https://github.com/tu6ge/oss-rs/commit/0a65ad2d581efef14d9b9b697f7e79d6c4e34247)), closes [#15](https://github.com/tu6ge/oss-rs/issues/15)


### Features

* **config:** Add AsMut of ObjectDir ([499de79](https://github.com/tu6ge/oss-rs/commit/499de79393324e161486b642030618529614f5e5))
* **core:** Add AsRef in some type ([2b72ba0](https://github.com/tu6ge/oss-rs/commit/2b72ba07ea8ac18feaa88396b7dddd241dcb2c31))
* re-export Error Result ([f1cdae3](https://github.com/tu6ge/oss-rs/commit/f1cdae3668909cb4c4a50e1c402d4ba2ee4c523a))



#  (2023-03-10)



## [0.11.1](https://github.com/tu6ge/oss-rs/compare/0.11.0...0.11.1) (2023-03-10)


### Features

* **types:** Custom endpoint deny `oss` prefix ([4aa7bbc](https://github.com/tu6ge/oss-rs/commit/4aa7bbc6d3e4b18a59d87d87a8096b9905cb66ad))
* **types:** From<&'static str> change to From<&'a str> ([848f149](https://github.com/tu6ge/oss-rs/commit/848f1499b00daa4bfaf66bf5e453d56fd9f2e028))



#  (2023-03-09)



# [0.11.0](https://github.com/tu6ge/oss-rs/compare/0.10.1...0.11.0) (2023-03-09)


### Features

* **auth:** AuthBuilder method 参数签名更改 ([b0c5182](https://github.com/tu6ge/oss-rs/commit/b0c5182ef99e231f29cbea092f2535edef15a6cd))
* **bucket:** 读取列表和详情内部的信息 ([c006da4](https://github.com/tu6ge/oss-rs/commit/c006da4afe08bb63c2132160b089a72c6d9f6693))
* **bucket:** 更改 BucketList 内部字段类型 ([03edbd3](https://github.com/tu6ge/oss-rs/commit/03edbd37f7cd7ebf6ca652776be0d9b9a5153566))
* **bucket:** BucketList Add Item generic ([0e85c82](https://github.com/tu6ge/oss-rs/commit/0e85c82bc9105afbaf1550db3752dbaa73df501a))
* **bucket:** remove Option wrapper in ListBuckets ([f9e2a3d](https://github.com/tu6ge/oss-rs/commit/f9e2a3d9f9e78e00035068a2694604332948ae15))
* **builder:** 更改方法的可见性 ([5fe7326](https://github.com/tu6ge/oss-rs/commit/5fe7326fa02a35409423172a7dfa63c0470f9cb0))
* **config:** 增加内部类型 ObjectPathInner ([de94aee](https://github.com/tu6ge/oss-rs/commit/de94aee58c313b0b020683cd53285f7ab381d182))
* **config:** Add ObjectDir type ([84a89cd](https://github.com/tu6ge/oss-rs/commit/84a89cdc265be74116af4c6d2479ceb31c3ea4ea)), closes [#12](https://github.com/tu6ge/oss-rs/issues/12)
* **config:** ObjectDir Support + operator ([b95faed](https://github.com/tu6ge/oss-rs/commit/b95faedc977671bcfc3d9a71165d7debdd8c6be9))
* **config:** remove repeat method ([f740008](https://github.com/tu6ge/oss-rs/commit/f7400086ff0dde35c82d7452751e69c99a2f9cf1))
* **config:** update ObjectDir new method ([2d8f842](https://github.com/tu6ge/oss-rs/commit/2d8f842afad4171957ef7d829a1ec1ddec14243b))
* **decode:** 对导出的 trait 改名 ([dc9c20c](https://github.com/tu6ge/oss-rs/commit/dc9c20ca55f8e8e0db3fdf93cf5dd7f8fe7ae639))
* **decode:** 减少对自定义类型的限制条件 ([2c6e445](https://github.com/tu6ge/oss-rs/commit/2c6e44596d9508dba58ad8c6cb9d0b5eadc0f984))
* **decode:** 减少对自定义类型的限制条件 ([8d8a639](https://github.com/tu6ge/oss-rs/commit/8d8a6396af8e215ad74e036e92efc80ad35e4309))
* **decode:** 减少对自定义类型的限制条件 ([4fe2441](https://github.com/tu6ge/oss-rs/commit/4fe2441a634e3f0c745f30d497240cccb032412c)), closes [#12](https://github.com/tu6ge/oss-rs/issues/12)
* **decode:** 内部 trait 增加默认实现 ([7acaec3](https://github.com/tu6ge/oss-rs/commit/7acaec37396a28dce53c88c39e3b7bd737056806))
* **decode:** traits change to decode ([a9b3a8d](https://github.com/tu6ge/oss-rs/commit/a9b3a8d61237a01387c5692a928cb7d1291e10f1))
* **file:** 对文件操作改为更加灵活的方式 ([f8cf9ea](https://github.com/tu6ge/oss-rs/commit/f8cf9ea739e210354b402bc452447750b37c0225))
* **file:** 将 blocking 的 File trait 改名为 Files ([977db3f](https://github.com/tu6ge/oss-rs/commit/977db3f7ea0638154bf0f556722cdf44bc33ab5a))
* **file:** 将 File trait 改名为 Files,另外新增 File trait ([023c320](https://github.com/tu6ge/oss-rs/commit/023c320626b495320eddeb81119dea494c85dd4e))
* **file:** remove put_file and more method ([5a95a8f](https://github.com/tu6ge/oss-rs/commit/5a95a8f1692c72e547acc04396a3ec7ccdf37cd6))
* **lib:** remove traits mod name ([35c1773](https://github.com/tu6ge/oss-rs/commit/35c177348b6db696da7ed339e28a9eefb28832bc))
* **macro:** add derive with decode ([f51865d](https://github.com/tu6ge/oss-rs/commit/f51865d69de76743e0b1df1fac8109da3e0bde55))
* **objcet:** change ObjectList prefix type ([9f07f34](https://github.com/tu6ge/oss-rs/commit/9f07f34bb800312dbe058553c9a28fdc1b08cefa))
* **object:** add get_next_base method ([c77b379](https://github.com/tu6ge/oss-rs/commit/c77b379a456908ad9428fea1b5b182c0debcd9c2))
* **object:** ObjectList Add Item generic ([63d85e2](https://github.com/tu6ge/oss-rs/commit/63d85e295bdae7f07ca13a04a5cef707426564a2)), closes [#12](https://github.com/tu6ge/oss-rs/issues/12)
* **object:** Support CommonPrefix ([c3e54c1](https://github.com/tu6ge/oss-rs/commit/c3e54c1aa665617b9a2ef7fec083b4ba4da14bf3)), closes [#9](https://github.com/tu6ge/oss-rs/issues/9)
* **sts:** STS 秘钥支持更多类型 ([f2e1531](https://github.com/tu6ge/oss-rs/commit/f2e153120c4601c62864d07261babb8c212bda63))
* **type:** 支持更多的可用区 ([8e65f01](https://github.com/tu6ge/oss-rs/commit/8e65f01bf67e0241ebc7b306f4819d2630d15bb0))
* **types:** 提升 BucketName EndPoint 等类型的安全性 ([895e373](https://github.com/tu6ge/oss-rs/commit/895e373f71f78eb4dc2ec25289c46cbc2bc24398))
* **types:** Support FromStr for more buildin type ([e56afe8](https://github.com/tu6ge/oss-rs/commit/e56afe83d700d037092b27fe8b8a5f5cfb4f9bd0))
* **types:** unwrap changed to expect ([066813b](https://github.com/tu6ge/oss-rs/commit/066813b4dfd095d4cbbd3babde578e0385b36318))


# [0.10.0](https://github.com/tu6ge/oss-rs/compare/0.9.7...0.10.0) (2022-12-10)


### Features

* **auth:** remove VERB ,use http::Method ([06ed16b](https://github.com/tu6ge/oss-rs/commit/06ed16b08db653435149270314a810a889138a84))
* **bucket:** deprecated intranet_endpoint field ([68f1fc0](https://github.com/tu6ge/oss-rs/commit/68f1fc02971049829f52b15bdff0994f8e4152c7))
* **builder:** Support Response without xml error ([cd49a01](https://github.com/tu6ge/oss-rs/commit/cd49a017fdfbc50c3a4bb8a021ae2edf2ec27877)), closes [#7](https://github.com/tu6ge/oss-rs/issues/7)
* **client:** 添加获取 object 元信息的方法 ([4c9d8e3](https://github.com/tu6ge/oss-rs/commit/4c9d8e341d68b96fce68161d442075e59e8623a1))
* **client:** deprecated set_bucket_name method ([acc281a](https://github.com/tu6ge/oss-rs/commit/acc281ab103516123903aeae2296533058531f52))
* **core:** builder_with_header 签名更改 ([2104d5c](https://github.com/tu6ge/oss-rs/commit/2104d5cc87d962a4193c64a28dc263c6f62ec54f))
* **error:** changed OssService ([7bc42ac](https://github.com/tu6ge/oss-rs/commit/7bc42acde7c383b2e48071a551206e41cc445244))
* **error:** Enhance OssService ([e74deec](https://github.com/tu6ge/oss-rs/commit/e74deec568c7f39d5ca6996b0d5d7365d61c70d8))
* **file:** 为 ObjectBase 增加了几个方法 ([0e48c21](https://github.com/tu6ge/oss-rs/commit/0e48c2113e81283e121c07e25f0ae08e7ff6f285))
* **object:** 读取和设置 object 信息更改 ([57796d3](https://github.com/tu6ge/oss-rs/commit/57796d370b96279b5ac3913c009bed0c4bbf421c))
* **object:** head_object example ([b8669b2](https://github.com/tu6ge/oss-rs/commit/b8669b23b379a36ec9c43a096767db6ddec199b6))
* **types:** 支持 &str 转 Query ([597b530](https://github.com/tu6ge/oss-rs/commit/597b5309ae6417ddc93e594153af89dd3aa716c2))
* **types:** BucketBase 添加 get_url_resource 方法 ([2197750](https://github.com/tu6ge/oss-rs/commit/2197750745bd0a66b18909b171677b41f76ad7b0))
* **types:** Query 添加 IntoIterator ([910f565](https://github.com/tu6ge/oss-rs/commit/910f565282b9300d370e5a56e0c0e65308133b9b))
* **types:** Query 支持更多的生成方式 ([5634669](https://github.com/tu6ge/oss-rs/commit/56346692a93ed388823cdf6cd4e14d897fac1bc9))



## [0.9.7](https://github.com/tu6ge/oss-rs/compare/0.9.6...0.9.7) (2022-12-05)



## [0.9.6](https://github.com/tu6ge/oss-rs/compare/0.9.5...0.9.6) (2022-12-05)



## [0.9.5](https://github.com/tu6ge/oss-rs/compare/0.9.4...0.9.5) (2022-12-05)


### Bug Fixes

* **file:** Fixed tip message ([20582dd](https://github.com/tu6ge/oss-rs/commit/20582dd417ce8ad7aabd23433a339aab06aac415)), closes [#5](https://github.com/tu6ge/oss-rs/issues/5)


### Features

* **client:** 可导出 bucket 列表到自定义类型 ([246042c](https://github.com/tu6ge/oss-rs/commit/246042c05c041d5514ca43c72b8813cf62c27c4e))
* **client:** 可导出 bucket 信息到自定义类型 ([225c7e5](https://github.com/tu6ge/oss-rs/commit/225c7e5db43e82a9e41f5f7856275798e46b29dd))
* **client:** 可导出自定义object列表 ([d1e95be](https://github.com/tu6ge/oss-rs/commit/d1e95be8f15c8afd583dade73757f174a50de680))
* **client:** 可导出自定义object列表 ([b254d5c](https://github.com/tu6ge/oss-rs/commit/b254d5ce27bd94268608b69992e1f69ef21e2367))
* **client:** 可导出自定义object列表 ([e2c701f](https://github.com/tu6ge/oss-rs/commit/e2c701fccabec0f34b1af6f545b8d2c8428d23fd))
* **init:** 支持更便捷的初始化方式 ([c9bb523](https://github.com/tu6ge/oss-rs/commit/c9bb52394bf89b761326d640dad3d2a461334f31))
* **object:** base_object_list 可传递自定义bucket名称 ([8b7ef6e](https://github.com/tu6ge/oss-rs/commit/8b7ef6e906315653b51b7d507746d95e6ff56094))
* **query:** Enhance Query type ([db1675f](https://github.com/tu6ge/oss-rs/commit/db1675f79318890ff4713061438278fdbe68eea8))
* **traits:** 减少特征要求 ([b28015f](https://github.com/tu6ge/oss-rs/commit/b28015fc1f98f272c2c9cb9af24e8355de5199bf))
* **traits:** 解析xml方法改为更加通用的方式 ([99fabb0](https://github.com/tu6ge/oss-rs/commit/99fabb0373ba04e87c94ffbec95d3a0e9ddc3d8c))
* **traits:** 解析xml方法改为更加通用的方式 ([7ef563f](https://github.com/tu6ge/oss-rs/commit/7ef563ffebf534ee221477b9ee2dbb859879cb91))
* **traits:** trait 名称中 OssInto 更改为 Refine ([37c5e13](https://github.com/tu6ge/oss-rs/commit/37c5e13caf5cdfca4f975922acb4107c47933dff))
* **types:** Support vec! into Query ([be6fb16](https://github.com/tu6ge/oss-rs/commit/be6fb16babeac7f8e77f3d7e5146d7d149812873))
* **xml:** Parse xml Reduced copy ([bea5f89](https://github.com/tu6ge/oss-rs/commit/bea5f89caa7fd1016545a12c14115f0190c9ee24))



## [0.9.4](https://github.com/tu6ge/oss-rs/compare/0.9.3...0.9.4) (2022-11-27)


### Bug Fixes

* **auth:** remove default in accident ([f11122d](https://github.com/tu6ge/oss-rs/commit/f11122d5ac019c5e6d2fb16148f7acc3ccb08699))
* **config:** InvalidBucketBase changed to enum ([a91e8f1](https://github.com/tu6ge/oss-rs/commit/a91e8f131d928a02b08d36abbd725631a4d64a5e))
* **error:** remove println code ([15a9d5d](https://github.com/tu6ge/oss-rs/commit/15a9d5df7ada1ac52693f0f76ce2bfcd190a8b39))
* **macro:** syntex fixed ([ad5f89f](https://github.com/tu6ge/oss-rs/commit/ad5f89f45642a62af96c5be279282c9979bea4e4))
* **styles:** fix PartialEq in ObjectBase ([c243ea4](https://github.com/tu6ge/oss-rs/commit/c243ea49865113d3b7cc982f61775a7d94f0fb86))


### Features

* **deprecated:** intranet_endpoint in bucket ([a200da9](https://github.com/tu6ge/oss-rs/commit/a200da935e2af72cb3c0ffb64b3bc7ac1ff05269))
* **file:** 路径参数统一 ObjectPath 类型 ([00676a5](https://github.com/tu6ge/oss-rs/commit/00676a51697a0687968ec84d21de32f4fd439e94))
* **file:** 文件相关功能增强 ([c4bdd62](https://github.com/tu6ge/oss-rs/commit/c4bdd622ef06ce279c1fcb13d411e471c2408f1a))
* **file:** 文件相关功能增强 ([7bf7566](https://github.com/tu6ge/oss-rs/commit/7bf756600263a41ec61a4a33d7ba97b5979b44f5))
* **macro:** 区分 async 和同步 ([06bfbf4](https://github.com/tu6ge/oss-rs/commit/06bfbf4971c60bd6d4906c61dc9988a8a9a92c8d))
* **macro:** Add oss_gen_rc Macro ([795add1](https://github.com/tu6ge/oss-rs/commit/795add16ed35a6ed901ad18bd3e3d24d1eeea860))
* **macro:** blocking finished ([271c8eb](https://github.com/tu6ge/oss-rs/commit/271c8eb74eae16b922d0ac63323fb5d2ae03d390))
* **macro:** support async ([0c82d47](https://github.com/tu6ge/oss-rs/commit/0c82d47240eb39b53eb899f3d3d757890f062c93))
* **macro:** support attribute, eg. cfg ([3a72dda](https://github.com/tu6ge/oss-rs/commit/3a72dda55399aa9deb08fb3410c309bc03b49603))
* **macro:** support ClientRc,Rc::clone ([35b29a6](https://github.com/tu6ge/oss-rs/commit/35b29a63683975e04565c6d6d727d920e40e9827))
* **macro:** support inline ([1c35164](https://github.com/tu6ge/oss-rs/commit/1c35164642e22ca1119930d26a403f93cb518bb6))
* **marco:** init 可以导入代码并原样导出 ([d373793](https://github.com/tu6ge/oss-rs/commit/d373793903440606edb2b6ca2da5f062e7d1c99c))
* **object:** 移除多余的字段 ([a79cc1b](https://github.com/tu6ge/oss-rs/commit/a79cc1b59e90474cdf655719f7913c190dbe1ff3))
* **object:** add blocking get_object method ([ce93a7a](https://github.com/tu6ge/oss-rs/commit/ce93a7a4be8ca07226285d3bc263c2ec83df7be0))
* **object:** object 结构体支持文件操作 ([d6f5d22](https://github.com/tu6ge/oss-rs/commit/d6f5d22713901f944d497687d98e62dbef6bc020))
* **object:** object 结构体支持文件操作 ([efb7c17](https://github.com/tu6ge/oss-rs/commit/efb7c1778affdf5f93078b87222b440ce064646f))
* **object:** ObjectList Support Steam ([4dfaa30](https://github.com/tu6ge/oss-rs/commit/4dfaa30caac7096077b966fc4d78e0026bf0fa17))
* **object:** Objects Support Iterator ([b1dda87](https://github.com/tu6ge/oss-rs/commit/b1dda87864179535369a336403d531d6247c6c47))
* **object:** try steam get next ([ead7f5c](https://github.com/tu6ge/oss-rs/commit/ead7f5cdd8b2f768acf54120249d350d7e461747))
* **types:** buildin types support PartialEq ([cb88290](https://github.com/tu6ge/oss-rs/commit/cb88290ff53130b98e14a253ba99217313f96501))
* support internal ([05c8f85](https://github.com/tu6ge/oss-rs/commit/05c8f85d896b804ae46257689575dc7bdb2547bb))
* **types:** CR::from_object Arg update ([5bd6344](https://github.com/tu6ge/oss-rs/commit/5bd6344adbe02b3487cc71a8158d4c96c31d2653))
* **types:** Query support u16 value ([249725f](https://github.com/tu6ge/oss-rs/commit/249725f9b4e0c0039811bea6fe7ecf3edfe4cb88))
* **types:** Query support u8 value ([4ba8eac](https://github.com/tu6ge/oss-rs/commit/4ba8eac3eb3eaaafaa253da54f1cca9dedebeb50))



## [0.9.3](https://github.com/tu6ge/oss-rs/compare/0.9.2...0.9.3) (2022-11-16)


### Features

* **object:** read object_list field ([0bb48b2](https://github.com/tu6ge/oss-rs/commit/0bb48b259362edca32508338e0589c02d6d6a3bd)), closes [#4](https://github.com/tu6ge/oss-rs/issues/4)



## [0.9.2](https://github.com/tu6ge/oss-rs/compare/0.9.1...0.9.2) (2022-11-15)


### Bug Fixes

* **object:** fix obejct field read or written ([cd5a3e9](https://github.com/tu6ge/oss-rs/commit/cd5a3e9e4d8debc326e95af5d0197897244e019d)), closes [#2](https://github.com/tu6ge/oss-rs/issues/2)



## [0.9.1](https://github.com/tu6ge/oss-rs/compare/0.9.0...0.9.1) (2022-11-15)


### Bug Fixes

* **object:** fix obejct field read or written ([f4b0bf5](https://github.com/tu6ge/oss-rs/commit/f4b0bf50668414858432a2cf49a28bf90b15c669)), closes [#2](https://github.com/tu6ge/oss-rs/issues/2)



# [0.9.0](https://github.com/tu6ge/oss-rs/compare/0.8.0...0.9.0) (2022-11-08)


### Bug Fixes

* **auth:** Prevent secret exposure ([e4553f7](https://github.com/tu6ge/oss-rs/commit/e4553f7d74fce682d802f8fb073943387796df29))
* **client:** ignore put_content example ([f3ebb70](https://github.com/tu6ge/oss-rs/commit/f3ebb70748ee0ce6234f42b51ae3a1415148cbd8))
* **types:** ContentRange i32 update to u32 ([d7f32bf](https://github.com/tu6ge/oss-rs/commit/d7f32bf0ea8dc468b4f5c12627cba115fdf13021))


### Features

* **auth:** set all field to private ([61da9b2](https://github.com/tu6ge/oss-rs/commit/61da9b2e29a9500f571ea5a1ceed6cf997b295b6))
* **client:** remove infer field ([04b1cff](https://github.com/tu6ge/oss-rs/commit/04b1cff27c2dcf2a4fd233858ef6b18ab813f352))
* **client:** set middleware pub(crate) ([6073ad8](https://github.com/tu6ge/oss-rs/commit/6073ad83654d048c842a59fb62b4f91e8c169469))
* **object:** Add get_object method ([be828e1](https://github.com/tu6ge/oss-rs/commit/be828e136a90facf71a695270cc5e67a57262917))



# [0.8.0](https://github.com/tu6ge/oss-rs/compare/0.7.6...0.8.0) (2022-11-05)


### Bug Fixes

* **object:** 解决 object struct 默认值导致的问题 ([a5b5b86](https://github.com/tu6ge/oss-rs/commit/a5b5b86110e5903ea8f69f84921cee473a21a5e6))
* **plugin:** 修复几个过时代码的错误 ([4429681](https://github.com/tu6ge/oss-rs/commit/442968154f0412ba6a9c90b18728b81c18346d4f))
* **test:** 测试用例更改 ([98433a4](https://github.com/tu6ge/oss-rs/commit/98433a473cf100e09a1ef5faca2721de6625b68e))


### Features

* support STS ([676935f](https://github.com/tu6ge/oss-rs/commit/676935fad116faed65e70d501c19b0a95006be72)), closes [#1](https://github.com/tu6ge/oss-rs/issues/1)
* **auth:** 加上了几个 trait ([527fd7a](https://github.com/tu6ge/oss-rs/commit/527fd7a0834d990cced0aa24248373237ca24788))
* **auth:** 减少 auth 中多个 trait 之间的耦合性 ([356d8e1](https://github.com/tu6ge/oss-rs/commit/356d8e10206fc33009a397985d867d3e9aa4fce3))
* **auth:** 使用 Cow 减少 clone ([9917fd3](https://github.com/tu6ge/oss-rs/commit/9917fd327587569c75bf3e4eaedcd9dba7d28b13))
* **auth:** 整理 auth 模块的代码 ([550f9ef](https://github.com/tu6ge/oss-rs/commit/550f9ef7de1e5b4ead05d05e2527ecd302a1b4ab))
* **auth:** builder 实现 trait ([23e377d](https://github.com/tu6ge/oss-rs/commit/23e377d4aef16717954292069ed47bd52304eb0a))
* **blocking:** add bucket,object struct in block ([17e5b32](https://github.com/tu6ge/oss-rs/commit/17e5b32a9e4ddf593004811c34f4e2e5c0e77a57))
* **blocking:** add client fn ([a0ff03d](https://github.com/tu6ge/oss-rs/commit/a0ff03d35b4fdac649e737f381a0dfd25a9f15fb))
* **bucket,object:** 新增了 BucketBase 等 ([7860fb8](https://github.com/tu6ge/oss-rs/commit/7860fb8f6ee0139e18f7a969ce7bb0e177183a0e))
* **client:** 对 reqwest 中的struct 进行包裹封装 ([2473331](https://github.com/tu6ge/oss-rs/commit/24733310b100006e7b2f63b0359714839620ca72))
* **client:** 去掉多余的 async 标记 ([fabfe98](https://github.com/tu6ge/oss-rs/commit/fabfe987037291d886211d998fccaacdd92e34e7))
* **client:** 删除了过去的获取 canonicalized_resource 的方法 ([6c8acf4](https://github.com/tu6ge/oss-rs/commit/6c8acf4b11ade74ad1c272b6fc0821aa7255524a))
* **client:** 支持用 config 初始化 client ([7f50de3](https://github.com/tu6ge/oss-rs/commit/7f50de366f4f9637b62dbcb5f40b9211e9dc32e2))
* **client:** add part in Client struct ([46e17e1](https://github.com/tu6ge/oss-rs/commit/46e17e12b45c40c43bf24b53ca4a95faca7cc261))
* **client:** client init method update ([855e3ef](https://github.com/tu6ge/oss-rs/commit/855e3efc132b96e9838d21a6a79d87b5fb0d359d))
* **client:** upgrade builder method ([5ce269d](https://github.com/tu6ge/oss-rs/commit/5ce269d364a47ad6a01398002a7b2fcc8efdfe52))
* **config:** add P in ObjectBase ([6638e0a](https://github.com/tu6ge/oss-rs/commit/6638e0a592cfe477a0778214eea15022d9b0d0ab))
* **error:** 解析错误，去掉regex方案 ([9329ee2](https://github.com/tu6ge/oss-rs/commit/9329ee2026d81519094b030d86a36356d2d5d6c0))
* **object:** add put_content_base method ([b1caf7e](https://github.com/tu6ge/oss-rs/commit/b1caf7ea7effb2aacb234982b05bf12b59c1e32b))
* **plugin:** 清理 plugin 的残余内容 ([c27a1de](https://github.com/tu6ge/oss-rs/commit/c27a1dee5d988dc78bf7c8b6df63f2cd8ae576af))
* 替代 object_list_query_generator 方法的方案 ([120c343](https://github.com/tu6ge/oss-rs/commit/120c3432c4c4c60d96714137357a8095525d37e6))
* 添加了几个新类型 ([b1f2932](https://github.com/tu6ge/oss-rs/commit/b1f29324c62b24a35af3e21b1b22672208047d20))
* 引入 Cow 智能指针 ([4e8613e](https://github.com/tu6ge/oss-rs/commit/4e8613e56a79535900a66934c2de449e10a26cf2))
* add BucketBase,ObjectBase struct ([ce97f54](https://github.com/tu6ge/oss-rs/commit/ce97f54fc59ac9385bf115e79f425b48ae1c7679))
* unite Arc and Rc in bucket,object ([8460f33](https://github.com/tu6ge/oss-rs/commit/8460f3374a2f9fb0afbb98d229c0f903e75487ab))
* **client:** client结构体不再保持key ,secret ([54838aa](https://github.com/tu6ge/oss-rs/commit/54838aa561e6083fde8e75b257fdcf1132596c19))
* **plugin:** 移除 plugin 模块 ([4460faf](https://github.com/tu6ge/oss-rs/commit/4460faff028f9856598a1a1ff7bffda793fbf4b2))
* **plugin:** remove plugin example ([664ad06](https://github.com/tu6ge/oss-rs/commit/664ad06ab3ddcc98f61c7233584fcb831f803d93))
* **types:** 更改方法名称 ([7f9c19b](https://github.com/tu6ge/oss-rs/commit/7f9c19b1fe106f89678b7499decf96d5d4cfb1e5))
* **types:** BucketName 变得更安全 ([e3daec4](https://github.com/tu6ge/oss-rs/commit/e3daec482b7b599a844dbd00b47102c755687601))
* **types:** EndPoint update to enum type ([b218e76](https://github.com/tu6ge/oss-rs/commit/b218e761ca9d99fc5ffd208224839cd0a6c47101))



## [0.7.6](https://github.com/tu6ge/oss-rs/compare/0.7.5...0.7.6) (2022-09-07)


### Bug Fixes

* **error:** 解析oss错误信息修改 ([009b7b2](https://github.com/tu6ge/oss-rs/commit/009b7b276bf22aa3e63dce3c4c49ca1914022e0c))


### Features

* **auth:** add auth builder ([8151576](https://github.com/tu6ge/oss-rs/commit/8151576a80b78638dba671024e805e100e81567f))
* **error:** OssError 添加 message 方法 ([d0938b9](https://github.com/tu6ge/oss-rs/commit/d0938b979c7d5174d0c5b26613ab47c041ad1b8c))



## [0.7.5](https://github.com/tu6ge/oss-rs/compare/0.7.4...0.7.5) (2022-09-04)


### Features

* **aliyun:** 更好的阿里云错误的处理方式 ([ee97d1e](https://github.com/tu6ge/oss-rs/commit/ee97d1e845bee0c24caa259fc33374a72bd0220e))



## [0.7.4](https://github.com/tu6ge/oss-rs/compare/0.7.3...0.7.4) (2022-08-25)



## [0.7.3](https://github.com/tu6ge/oss-rs/compare/0.7.2...0.7.3) (2022-08-21)


### Bug Fixes

* **plugin:** 解决在多线程情况下，plugin的问题 ([8dc3d83](https://github.com/tu6ge/oss-rs/commit/8dc3d8338161300b6aa6db4d26f2611f39272c7c))


### Features

* **object:** 上传文件的路径支持特殊字符（空格等） ([a237f2f](https://github.com/tu6ge/oss-rs/commit/a237f2f1279a4da9f3186f7dd1338523a446e796))
* **plugin:** 支持自定义扩展文件类型 ([f48957a](https://github.com/tu6ge/oss-rs/commit/f48957a0bc3e5d4ccc31fd672094376838e3d3a8))



## [0.7.2](https://github.com/tu6ge/oss-rs/compare/0.7.1...0.7.2) (2022-08-20)


### Bug Fixes

* **plugin:** 解决不使用 plugin 特征时导致的问题 ([99c2544](https://github.com/tu6ge/oss-rs/commit/99c2544ed8b52f862caf8823a6388611a628268f))


### Features

* **object:** 上传文件时，传递路径的方式更加灵活 ([1cce936](https://github.com/tu6ge/oss-rs/commit/1cce9363ec1dbd10d75459bcbd5f7db3630f805a))



## [0.7.1](https://github.com/tu6ge/oss-rs/compare/0.7.0...0.7.1) (2022-07-17)



# [0.7.0](https://github.com/tu6ge/oss-rs/compare/0.6.0...0.7.0) (2022-07-17)


### Features

* **client:** Method 类型支持更灵活的赋值方式 ([7c5b436](https://github.com/tu6ge/oss-rs/commit/7c5b436f97817d3d869385f751752fc1a81025a6))
* **object:** put_content arg type update ([f6f4864](https://github.com/tu6ge/oss-rs/commit/f6f4864fccf2b3681adae578243f2a9e3cd1f90f))
* **trait:** 支持导出数据到自定义 object bucket 结构体 ([4451322](https://github.com/tu6ge/oss-rs/commit/445132277d86d9974b5bbc14fa3e634d92d8272c))



# [0.6.0](https://github.com/tu6ge/oss-rs/compare/0.5.0...0.6.0) (2022-06-26)


### Features

* **async:** 异步的方法去掉前缀 async 改为默认方法 ([7fbed19](https://github.com/tu6ge/oss-rs/commit/7fbed1941afad74e4b61d8209a6a0e276398a057))
* **blocking:** 同步方法加上 blocking 前缀 ([7def094](https://github.com/tu6ge/oss-rs/commit/7def09423198985ed746e390aaf61b82aa7d86e0))
* **blocking:** reqwest 的 blocking 特征改为可选引用 ([cdcc197](https://github.com/tu6ge/oss-rs/commit/cdcc1970504855b034cbe68d466e6993049f8d03))
* **stream:** 尝试实现 stream ([2d0679e](https://github.com/tu6ge/oss-rs/commit/2d0679e3183caa37579dd07e1ac3c686266bc073))
* **sync:** 支持异步调用（所有接口） ([f12cb27](https://github.com/tu6ge/oss-rs/commit/f12cb27a0c7871d1c8c5b432e25077c615dc7e99))


### Reverts

* Revert "refactor: wip" ([35af9df](https://github.com/tu6ge/oss-rs/commit/35af9df839c35cad5464babc7b1ad229721b3b79))



# [0.5.0](https://github.com/tu6ge/oss-rs/compare/0.4.4...0.5.0) (2022-06-19)


### Features

* **plugin:** 插件可查看 client 结构体内容 ([5b6c894](https://github.com/tu6ge/oss-rs/commit/5b6c89450ddcc542a2595e910427ff1a6b51067d))
* **plugin:** 增加插件的能力 ([bc71a1f](https://github.com/tu6ge/oss-rs/commit/bc71a1fadf67217df601bc273555f3cd887efad7))
* **plugin:** 支持插件机制 ([fb1ac8f](https://github.com/tu6ge/oss-rs/commit/fb1ac8fea8f969a67f270dc198cad9ab80c98df1))



## [0.4.4](https://github.com/tu6ge/oss-rs/compare/0.4.3...0.4.4) (2022-06-17)


### Bug Fixes

* **object:** 解决put时的签名错误 ([5ea71b4](https://github.com/tu6ge/oss-rs/commit/5ea71b4ee516f6859cf5453b0b68189c1dcabceb))



## [0.4.3](https://github.com/tu6ge/oss-rs/compare/0.4.2...0.4.3) (2022-06-16)


### Features

* **copy:** 复制文件功能完成 ([2114c7f](https://github.com/tu6ge/oss-rs/commit/2114c7f115f83a988fb8a6c0b1046d57cf6467fb))
* 测试复制object 功能 ([3e9de65](https://github.com/tu6ge/oss-rs/commit/3e9de65909257a9dd3388bf923b8093822fc41e6))
* 测试复制object 功能 ([d72fae9](https://github.com/tu6ge/oss-rs/commit/d72fae9f31b058ae9c1d0d662a2a8f5e46e00307))



## [0.4.2](https://github.com/tu6ge/oss-rs/compare/0.4.1...0.4.2) (2022-06-15)


### Features

* **object:** 列表加上迭代器功能 ([1bd43d6](https://github.com/tu6ge/oss-rs/commit/1bd43d6682e76ca888e498da023da0ad05f31fab))



## [0.4.1](https://github.com/tu6ge/oss-rs/compare/0.4.0...0.4.1) (2022-06-14)



# [0.4.0](https://github.com/tu6ge/oss-rs/compare/0.3.1...0.4.0) (2022-06-14)


### Features

* **bucket:** bucket struct 添加 get_object_list 方法 ([8f7d4bb](https://github.com/tu6ge/oss-rs/commit/8f7d4bbe0d438e49f5f72ff9acf682e7c3afa7a1))



## [0.3.1](https://github.com/tu6ge/oss-rs/compare/0.3.0...0.3.1) (2022-06-14)



# [0.3.0](https://github.com/tu6ge/oss-rs/compare/0.2.6...0.3.0) (2022-06-12)


### Features

* **error:** 优化 oss 返回错误的处理方式 ([2c80270](https://github.com/tu6ge/oss-rs/commit/2c8027075a9ec469265d986ecafd788d86f08f50))
* **error:** supplement error handler ([262b60c](https://github.com/tu6ge/oss-rs/commit/262b60cb8a5073a030376c44266840b4d2612d98))
* **object:** 获取object 列表时加上参数支持 ([bdde53c](https://github.com/tu6ge/oss-rs/commit/bdde53cdbf866886d9455be30c6eb4c821e94bb1))
* **objects:** 接收object 列表接口返回的 next token ([082857d](https://github.com/tu6ge/oss-rs/commit/082857d0bfee62208901007a045f07fd6474ce28))



## [0.2.6](https://github.com/tu6ge/oss-rs/compare/0.2.5...0.2.6) (2022-05-30)



## [0.2.5](https://github.com/tu6ge/oss-rs/compare/0.2.4...0.2.5) (2022-05-30)



## [0.2.4](https://github.com/tu6ge/oss-rs/compare/0.2.3...0.2.4) (2022-05-30)



## [0.2.3](https://github.com/tu6ge/oss-rs/compare/0.2.2...0.2.3) (2022-05-30)



## [0.2.2](https://github.com/tu6ge/oss-rs/compare/0.2.1...0.2.2) (2022-05-23)



## [0.2.1](https://github.com/tu6ge/oss-rs/compare/0.2.0...0.2.1) (2022-05-22)



# [0.2.0](https://github.com/tu6ge/oss-rs/compare/0.1.0...0.2.0) (2022-05-22)


### Features

* 入口方式调整 ([8b8f3b0](https://github.com/tu6ge/oss-rs/commit/8b8f3b09b62a31cab96186312f39f8185a217e9c))
* 删除文件完成 ([25c5891](https://github.com/tu6ge/oss-rs/commit/25c58915caf38b3bf5d90d4bd4630fc77464083b))
* 上传文件完成 ([999b3e6](https://github.com/tu6ge/oss-rs/commit/999b3e602478aac5bf2c4569e6d2d721502f1f44))
* 上传文件完成 ([f63d408](https://github.com/tu6ge/oss-rs/commit/f63d408f2f5d8c6dc5299b51b8176a3135e505ad))



# [0.1.0](https://github.com/tu6ge/oss-rs/compare/43decad21bf8cfe0246a39996ef1e04c737538d8...0.1.0) (2022-05-21)


### Features

* 获取对象列表 ([2cca3d1](https://github.com/tu6ge/oss-rs/commit/2cca3d16359db240e3f9ddb7c35198d72d626fde))
* **bucket:** 时间格式化 ([dd9399e](https://github.com/tu6ge/oss-rs/commit/dd9399eeaab0eefc632df2c7c87d05b553fdfb14))
* 获取 bucket 列表成功 ([7af9fef](https://github.com/tu6ge/oss-rs/commit/7af9fef50bb41d290a838ea91e19d4134329a759))
* 签名验证通过 ([3f867b7](https://github.com/tu6ge/oss-rs/commit/3f867b79fa7039f216ed17a48d4697dc3e4ee806))
* auth 和 http 简单封装 ([4539583](https://github.com/tu6ge/oss-rs/commit/453958348bfb57c06dae06b75eef03fbeb9a9cb6))
* auth 和 http 简单封装2 ([6cb8940](https://github.com/tu6ge/oss-rs/commit/6cb89400de25a733a164d485555b82b14d6ce98e))
* auth struct 初步完成 ([43decad](https://github.com/tu6ge/oss-rs/commit/43decad21bf8cfe0246a39996ef1e04c737538d8))
* object and bucket struct 初步完成 ([b2eb7ce](https://github.com/tu6ge/oss-rs/commit/b2eb7ce9722d8e9059e09de9b52c12e838990ed4))


### Performance Improvements

* 尝试优化 xml 读取时的性能 ([bafdbd4](https://github.com/tu6ge/oss-rs/commit/bafdbd424c1eeebc4027b95096c669182501bcbd))



