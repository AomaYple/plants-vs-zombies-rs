# pvz-assets

资源清单审计与 CPU 格式解析 crate：清单路径、大小和 SHA-256 审计，以及 Reanim、粒子 XML 的最小模型。

`AssetRoot::open` 只接收调用方提供的仓库根目录；`AssetIndex::audit` 只读清单登记的输入并报告缺失、大小、哈希和额外文件。`parse_reanim` 与 `parse_particle` 只接收文本，不读取文件系统，因此可以由桌面、Web 和移动端资源提供层复用。当前支持的格式子集不代表完整原版格式，未知扩展在扩展解析合同前应显式记录。
