# 用户本地资源

本目录只提交此说明；README 同时保留目录并说明用途，无须额外占位文件。其余内容均被 Git 忽略。

```text
assets-private/
├─ README.md
├─ original/       用户自行提供的已解包资源
├─ cache/          本机缓存、图集等转换产物
└─ packs/          本机生成的资源包
```

资源可以保持在项目目录外；未来实现配置读取后，在被忽略的 `config/local.toml` 中指定位置，无须复制进项目。配置格式与当前状态见 [配置约定](../config/README.md)。

预期输入包括 images、reanim、particles、sounds、properties、data、compiled。
资源工具先检查 resources.xml 的引用、文件尺寸、格式和路径，再扩展转换功能。
main.pak 格式兼容是后续可选能力，当前已有一致的解包输入。

原版内容与 MIT 范围遵守 [公开边界](../docs/principles/game-development.md#原版资源与公开边界)。截图比对、参考录像和存档放在被忽略的本机 `artifacts/`，不作为公开 CI 输入；原创小型格式夹具放在 [tests/fixtures](../tests/fixtures/README.md)并记录来源。

提交前用 `git status --ignored` 核对本机素材和缓存，用 `git ls-files` 核对实际跟踪列表；公开前按变更范围检查历史、LFS 与构建产物。忽略规则只影响未跟踪文件，不能移除已经提交的内容。
