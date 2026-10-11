# 版本化参考资源

本目录保存项目使用的版本化原版参考资源。资源用于新写 Rust 实现的资源装配、格式解析、渲染比对和行为复现；它们不提供原版程序代码，也不让重写运行时依赖原版可执行文件。

## 目录布局

```text
assets/
├─ original/       已核对的目录资源与 main.pak
├─ manifest.json   每个资源的来源、转换记录和 SHA-256
├─ third-party/    中文字体等独立登记的第三方支持资源
└─ README.md       本说明
```

当前基线从本机 `main.pak` 解包目录导入，共 3,228 个解包文件，总原始大小 67,973,713 字节；`assets/original/main.pak` 作为对应资源包一并保存，大小 68,117,478 字节。包索引包含 3,228 个条目，占 143,765 字节；包内容与解包目录逐文件、逐字节一致。

解包资源来源标签为 `PvZ_Toolkit_v1.22.0/extract_1791212844`；资源包来自 `Plants vs. Zombies (AIO v1.0)/Plants vs. Zombies GOTY ZH/1.1.0.1056 2012(.07)`。这些标签记录输入来源，参考版本身份以完整文件指纹为准，公开文档不保存本机用户绝对路径。

导入时只对文本表示做仓库格式规范化：去除 UTF-8 或 UTF-16LE 编码标记，将 CRLF/CR 转换为 LF，并确保文件末尾有换行。其他字符和正文空白保持不变。XML、Reanim、TXT、trail 和 jsfl 归为文本；PNG、JPEG、GIF、OGG、MO3、AU、compiled 文件以及 `main.pak` 保持源文件字节。清单同时记录源文件与仓库文件的大小和 SHA-256。当前文本资源 290 个，二进制资源 2,938 个，另有 1 个资源包记录。

## 清单合同

`assets/manifest.json` 使用 schema version 1，`files` 中每条记录至少包含：

- `path`：相对于 `assets/original/` 的 POSIX 路径；
- `kind`：`text`、`binary` 或 `package`；
- `source_size`、`source_sha256`：导入来源的大小和指纹；
- `size`、`sha256`：仓库内文件的大小和指纹；
- `source_encoding`：所有文本记录的原始编码，为 `UTF-8`、`UTF-8-BOM` 或 `UTF-16LE-BOM`。

`package` 字段记录 `main.pak` 的条目数、索引大小和解包核对结果。资源更新必须重新生成清单，核对来源与导入后指纹，确认包索引和目录内容一致，并同步更新参考基线或新增 ADR。不能只替换文件而保留旧指纹。

CI 会确认清单与 `assets/original/` 的跟踪集合完全一致，校验每个文件的 SHA-256，并对文本资源执行 UTF-8、LF、无 BOM 和末尾换行检查。原版资源文件可以保留历史正文空白；CI 对清单登记的资源不套用项目文档的尾随空白规则。

## 使用与产物

开发与资源审计统一读取本目录的清单和资源。桌面、Web 和移动端通过各自的资源提供接口加载同一版本化输入，部署路径由平台适配层负责；运行时不要求用户另行提供原版安装目录，也不规划独立的本地资源覆盖配置。

在仓库根目录运行 `cargo run -p pvz-asset-tool --locked -- audit --format text` 或 `--format json`，只读核对全部登记资源；`inspect-reanim PeaShooter` 在校验登记与指纹后报告动画概要。命令合同、退出码和诊断见 [工具说明](../tools/pvz-asset-tool/README.md)。

`main.pak` 保留原始包与来源指纹，解包目录用于首阶段资源加载。两者承担来源核对与运行输入的不同职责；后续包读取或转换方案按实际加载、性能与打包需求引入。

缓存、图集等可重建的转换产物写入被 Git 忽略的 `target/`；原版截图、录像、参考存档和测量产物写入被忽略的 `artifacts/` 或保留在仓库外；测试使用各自隔离的临时目录。只有实际产生对应内容时才创建这些目录。运行时存档使用各平台的应用数据目录或浏览器存储，不写回版本化素材。

## 资源边界

原版 EXE 不是资源包内容，也不是重写的运行依赖，因此不导入 `assets/original/`。本机证据和临时转换产物不作为版本化资源提交。项目原创代码、文档和原创测试数据的 MIT 授权范围与参考资源的来源、商标和许可说明分开维护；本目录不把原版资源重新声明为 MIT。

参考版本的 EXE 文件指纹、版本字段差异和包一致性记录见 [参考基线](../docs/behavior/reference-baseline.md)，资源导入决策见 [ADR 0002](../docs/adr/0002-versioned-original-assets.md)。
