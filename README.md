# Plants VS Zombies — Rust Reimplementation

**plants-vs-zombies-rs** 是计划使用 Rust 与 Macroquad 实现的非官方跨平台重写项目。
目标是以用户自行提供的原版资源和可观察行为为参考，重新编写游戏逻辑、动画播放器、渲染、音频接线、关卡系统与工具。

An unofficial, independent Rust reimplementation using Macroquad. Original game assets are user supplied and are not included in this repository.

## 当前状态

仓库处于 **文档与结构阶段**：设计、实施计划、Git 配置和预留目录已经建立；没有 Rust 源码、可运行游戏或已注册的 Cargo package。
根 Cargo.toml 是成员列表为空的虚拟 workspace。不要将目录存在或 CI 结构检查通过理解为游戏已经可构建、可运行。

## 已确定的路线

- 引擎：Macroquad；模拟内核保持纯 Rust，与引擎分离。
- 命名：GitHub 仓库 plants-vs-zombies-rs；crate 前缀 pvz-。
- 许可证：原创代码、文档和自制测试数据使用 MIT。
- 平台：Windows、Linux、macOS 与 Web 先行，Android、iOS 随后接入。
- 资源：先读取已解包目录，再按需求增加本地转换；main.pak 读取不是初期功能。
- 还原：以记录了文件指纹的 EXE 与资源包为参考，以测量和回放验证行为。
- 质量：文本文件统一 UTF-8/LF/无 BOM；测试默认并行，覆盖率与性能基准随实现阶段加入；正确性优先于速度。

## 文档入口

- [完整设计](docs/superpowers/specs/2026-10-07-repository-and-runtime-design.md)
- [近期实施计划](docs/superpowers/plans/2026-10-07-foundation-and-asset-audit.md)
- [全项目路线图](docs/roadmap.md)
- [模块与依赖边界](docs/architecture/README.md)
- [参考版本核对记录](docs/behavior/reference-baseline.md)
- [资源接入约定](assets-private/README.md)
- [贡献与验证](CONTRIBUTING.md)

## 目录

```text
crates/            pvz-core、pvz-data、pvz-assets、pvz-app 的预留目录
tools/             pvz-asset-tool 的预留目录
config/            可提交的本地配置示例
assets-private/    被 Git 忽略的用户资源和缓存
tests/             自制测试夹具与回放的约定
docs/              设计、计划、行为证据和技术决策
.github/           当前阶段的结构检查 CI 与 PR 模板
```

## 工具链与当前验证

使用 Rust 2024 Edition、Cargo resolver 3，工具链固定为 1.99.0，包含 rustfmt 与 Clippy。
项目尚未宣布支持更早的最低 Rust 版本。

当前可以验证 workspace 元数据：

```sh
cargo metadata --no-deps --format-version 1
git diff --check
```

第一个 package 注册后再生成并提交 Cargo.lock，同时启用格式、检查、测试、Clippy、文档构建及依赖审计。
具体执行顺序见实施计划；当前没有可用的 cargo run 命令。

## 原版资源与许可

本仓库不包含原版图片、音频、字体、Reanim、粒子定义、中文文本、可执行文件、main.pak 或转换后的资源包。
资源由用户在本机提供；路径配置不提交到 Git，详见 assets-private/README.md。
读取用户提供的资源不会自动赋予其再分发权，MIT 仅覆盖本项目原创内容；原版资源、名称和商标不属于本项目 MIT 授权范围。
本项目与原游戏权利方无关联，也不声称已完成经过正式隔离审查的 clean-room 开发流程。
