# 用户本地资源

本目录只提交此说明和 .gitkeep。其余内容均被 Git 忽略。

```text
assets-private/
├─ README.md
├─ .gitkeep
├─ original/       用户自行提供的已解包资源
├─ cache/          本机缓存、图集等转换产物
└─ packs/          本机生成的资源包
```

也可以保持现有资源在项目目录外，只在 config/local.toml 中指定其位置，无须复制进项目。
本次仓库初始化不复制或生成任何原版资源。

预期输入包括 images、reanim、particles、sounds、properties、data、compiled。
资源工具先检查 resources.xml 的引用、文件尺寸、格式和路径，再扩展转换功能。
main.pak 格式兼容是后续可选能力，当前已有一致的解包输入。

不得提交原版贴图、音频、字体、Reanim、粒子、LawnStrings 文本、EXE、DLL、main.pak、截图夹具或它们转换产生的内容。
截图比对与参考录像放在本机 artifacts/，不作为公开 CI 输入。
自制的小型格式夹具放在 tests/fixtures，并在该目录说明原创来源。

新增贡献前用 git status --ignored 检查素材目录，用 git ls-files 检查实际跟踪列表。
忽略规则仅影响未跟踪文件，不能自动移除历史上已经提交的素材。
