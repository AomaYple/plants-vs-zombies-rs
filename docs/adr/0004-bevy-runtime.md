# ADR 0004：运行后端迁移到 Bevy

日期：2026-10-11。状态：用户已选择 Bevy，替代 [ADR 0001](0001-repository-runtime-and-assets.md) 中的 Macroquad 后端决定；其他仓库、模拟和资源边界继续有效。

## 依据

注册运行后端后，实际 `cargo deny check` 报告 Macroquad 0.4.16 受 [RUSTSEC-2025-0035](https://rustsec.org/advisories/RUSTSEC-2025-0035.html) 影响：安全 API 可以触发内存损坏，公告没有修复版本；字体链的 `ttf-parser` 0.25.1 另有 [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html) 停止维护公告。核对了 RustSec 公告原文和锁定依赖链，没有添加忽略项。

结合正确性第一和成熟库优先原则，用户明确选择转向 Bevy。资源格式、游戏规则、输入命令和确定性合同继续由独立模块承担；引擎仅作为应用的窗口、调度、绘制、中文文字和音频后端。

## 决定

锁定 Bevy 0.20.0，官方 package 声明最低 Rust 1.97.1、MIT OR Apache-2.0，兼容仓库固定的 Rust 1.99.0。关闭默认 feature，仅声明当前 2D、窗口、文字、图片、OGG 音频、并行执行器及 WebGL/X11 所需能力，实际列表在根 Cargo.toml；不启用完整 3D、游戏手柄或开发远程协议。

重新生成锁文件并复核实际依赖。当前字体链使用 Parley/Fontique/Skrifa，没有旧 `ttf-parser`；图片解码与引擎统一使用 image 0.25。字体为单独登记、保留 OFL 原文的 Noto Sans SC，不依赖系统字体。

实际隐藏窗口运行发现默认的非复杂文字分词配置反复报告缺少中文模型。启用 Bevy 提供的 `complex_script_segmentation`，让 Parley 使用成熟的字典分词实现；复验中文显示及错误日志，不屏蔽诊断。

核对 `encase`、`encase_derive` 和 `encase_derive_impl` 0.12.1 的 MIT No Attribution 原文后，将 `MIT-0` 加入依赖许可允许列表：它允许使用、修改、分发、再许可，与项目原创代码的 MIT 策略相容。此项是具体许可兼容性评估，不是安全公告豁免。传递依赖多版本仍按现有 warning 策略报告，不通过强制覆写不兼容版本去消除警告。

Web 包由与 Cargo.lock 匹配的 wasm-bindgen CLI 生成到 target，不维护手写引擎 glue；原有 Macroquad loader、调用及依赖移除。Bevy 的隐藏窗口和持续运行配置用于后台冒烟验证，实际平台构建、运行、音频和截图分别记录。

## 成本与验收

Bevy 增加依赖、首次编译成本和应用接线复杂度；不把迁移本身称为运行性能优化。已写的纯视口、输入、仿射、资源校验与模拟模块保留并复用。六平台支持依据实际验收，Windows/Web 先行，macOS/Linux/Android/iOS 仍需要各自环境。

重新运行依赖审计、相关测试、图形探针、覆盖率与时间基准；原版 tick、碰撞、完整 Reanim 与 MO3 音乐的待测量状态不因后端变化而自动完成。
