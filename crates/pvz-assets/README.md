# pvz-assets

资源清单审计与 CPU 格式解析 crate：清单来源、包元数据、路径、大小和 SHA-256 校验，以及 Reanim 稀疏帧与粒子描述树。

`Manifest::parse(&[u8])` 不读取文件系统；`records()`、`record()` 和 `validate_bytes()` 用于浏览器或应用资源提供器。记录保留内容与来源大小、指纹及来源编码，文本字节额外校验 UTF-8、LF、无 BOM 和末尾换行。未知清单字段、非小写指纹、来源不一致、禁止载荷及包合同冲突会被拒绝。

`AssetRoot::open` 接收调用方提供的仓库根目录；`AssetIndex::manifest()` 暴露纯模型，`read()` 只读登记普通文件，`audit()` 同时报告缺失、目录读取错误、大小、哈希、文本合同和额外文件。宿主入口拒绝路径组件、目录树与根目录中的符号链接（Windows 包含 junction），错误保留相对输入名称与 `ErrorKind`，避免披露用户绝对路径。这些检查不保证对抗同时替换路径的外部进程；并发修改的输入不属于只读审计合同。

`parse_reanim` 接受原版顶层片段或原创 `root` 包装，保存浮点帧率和 `x/y/sx/sy/kx/ky/f/a/i/font/text` 稀疏字段；缺省值不在解析时补继承。`parse_particle` 保留所有发射器与 `Field/SystemField` 子树，曲线只保存文本；Name、Image、EmitterType 在实际资源中允许缺省。粒子系统通过 `emitters()` 保留发射器顺序，发射器通过 `properties()`、`property()` 和属性的 `children()` 读取完整树；不提供只读首发射器的系统便捷访问器。解析器校验闭合、上下文、重复字段与有限数，拒绝未知标签、属性、DTD 和处理指令；粒子曲线、动画继承与运行时行为仍由后续应用层实现和验收。
