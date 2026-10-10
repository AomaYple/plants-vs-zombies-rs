# Plants vs. Zombies — Rust 重写

**plants-vs-zombies-rs** 是计划使用 Rust 与 Macroquad 实现的非官方跨平台重写项目。
目标是以版本化的原版参考资源和可观察行为为参考，重新编写游戏逻辑、动画播放器、渲染、音频接线、关卡系统与工具。

## 当前状态

仓库处于 **文档、结构与参考资源阶段**：设计、实施计划、Git 配置和预留目录已经建立；3,228 个解包资源及对应 `main.pak` 已放入 `assets/original/` 并登记清单。没有 Rust 源码、可运行游戏或已注册的 Cargo package。
根 Cargo.toml 是成员列表为空的虚拟 workspace。不要将目录存在或 CI 结构检查通过理解为游戏已经可构建、可运行。

## 已确定的路线

- 引擎：Macroquad；模拟内核保持纯 Rust，与引擎分离。
- 命名：GitHub 仓库 plants-vs-zombies-rs；crate 前缀 pvz-。
- 许可证：原创代码、文档和自制测试数据使用 MIT。
- 平台：支持目标为 Windows、macOS、Linux、Web、Android 和 iOS；桌面与 Web 先行，移动端随后接入，实际支持状态以各平台构建与运行验收为准。
- 资源：版本化资源位于 `assets/original/`，先读取已解包目录，再按需求增加转换；`main.pak` 读取不是初期功能。
- 还原：以记录了文件指纹的 EXE 与资源包为参考，以测量和回放验证行为。
- 原则：正确性第一；在此基础上尽可能优化运行速度与内存，并加快游戏开发。长期约束见 [项目原则](docs/principles/README.md)，测试、格式、中文提交和验收流程见 [贡献指南](CONTRIBUTING.md)。

## 文档入口

- [完整设计](docs/superpowers/specs/2026-10-07-repository-and-runtime-design.md)
- [近期实施计划](docs/superpowers/plans/2026-10-07-foundation-and-asset-audit.md)
- [项目原则](docs/principles/README.md)
- [工程原则](docs/principles/engineering.md)
- [游戏开发原则](docs/principles/game-development.md)
- [全项目路线图](docs/roadmap.md)
- [模块与依赖边界](docs/architecture/README.md)
- [参考版本核对记录](docs/behavior/reference-baseline.md)
- [资源说明与清单](assets/README.md)
- [贡献与验证](CONTRIBUTING.md)

## 目录

```text
crates/            pvz-core、pvz-data、pvz-assets、pvz-app 的预留目录
tools/             pvz-asset-tool 的预留目录
assets/            版本化的原版参考资源与清单
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
具体任务见实施计划，完整验收命令见贡献指南；当前没有可用的 cargo run 命令。

## 原版资源与许可

仓库包含经过记录和校验的原版参考资源，位于 `assets/original/`，清单和导入规范见 [资源说明](assets/README.md)。原版可执行文件、截图、录像、存档和临时派生产物不进入仓库。
原版参考资源保留独立的来源与许可边界；MIT 仅覆盖本项目原创代码、文档和明确原创的测试数据，不把原版资源重新声明为 MIT。资源导入时保留来源、原始指纹以及必要的格式规范化记录。
本项目与原游戏权利方无关联，也不声称已完成经过正式隔离审查的 clean-room 开发流程。
