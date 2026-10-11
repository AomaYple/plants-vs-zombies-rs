# 第三方支持资源

`noto-sans-sc.ttf` 是 Noto Sans SC 的官方可变字体，供应用中文文字使用；不依赖维护者的系统字体。来源为 Google Fonts 的 `ofl/notosanssc/NotoSansSC[wght].ttf`，来源 Git blob 为 `fb0637bafbcd804fe32152370a1225990745b4bc`。下载后实际核对 Git 对象及 SHA-256，没有修改字体字节。

此字体使用 SIL Open Font License 1.1，原文见 [OFL.txt](OFL.txt)，MIT 不覆盖字体。许可文本仅规范化 LF 和末尾换行。文件大小、SHA-256、来源和许可登记在 [manifest.json](manifest.json)，CI 核对实际跟踪集合及指纹；应用是二进制的明确消费者。

当前完整字体为 17,772,300 字节，增加构建和浏览器加载体积。先建立中文显示与实际运行基线；若后续通过成熟子集工具减少体积，须维护可重复生成流程、字体许可、字形覆盖和资源指纹，不以缺字换取体积改善。
