# Plants vs. Zombies — Rust 重写

**plants-vs-zombies-rs** 是使用 Rust 与 Bevy 实现的非官方跨平台重写项目。
目标是以版本化的原版参考资源和可观察行为为参考，重新编写游戏逻辑、动画播放器、渲染、音频接线、关卡系统与工具。

## 当前状态

仓库已实现 **首个白天开发切片，现按用户要求阶段性暂停**：`pvz-data`、`pvz-core`、`pvz-assets`、`pvz-asset-tool` 和 `pvz-app` 已注册。当前提供确定性模拟、资源清单/XML 审计、宿主 CLI、Bevy 视口/音频探针，以及 5×9 棋盘、两种植物、普通僵尸、阳光、豌豆和割草机的可玩流程。Windows 隐藏窗口已完成切片胜利冒烟，Web 已验证两卡种植、暂停、恢复和重开；任务 6 独立复核、其余 Web 交互和任务 7/8 尚未完成。完整原版行为和其余平台尚未验收，具体证据与续做入口见 [切片阶段性结果](docs/behavior/vertical-slice-results.md)。
`pvz-data`、`pvz-core` 和 `pvz-assets` 都不依赖游戏引擎；数据层不读取文件，核心层不访问资源和平台 API，资源层只在显式资源根目录审计时读取清单登记输入。

## 已确定的路线

- 引擎：Bevy；模拟内核保持纯 Rust，与引擎分离。
- 命名：GitHub 仓库 plants-vs-zombies-rs；crate 前缀 pvz-。
- Git：`main` 是唯一长期分支，独立任务使用短期分支，经验证和评审后通过 PR 合入并清理；提交使用中文 Conventional Commits，流程见 [贡献指南](CONTRIBUTING.md#提交推送与评审)。
- 许可证：原创代码、文档和自制测试数据使用 MIT。
- 平台：支持目标为 Windows、macOS、Linux、Web、Android 和 iOS；桌面与 Web 先行，移动端随后接入，实际支持状态以各平台构建与运行验收为准。
- 资源：版本化资源位于 `assets/original/`，先读取已解包目录，再按需求增加转换；`main.pak` 读取不是初期功能。
- 还原：以记录了文件指纹的 EXE 与资源包为参考，以测量和回放验证行为。
- 原则：正确性第一、速度第二、内存第三；速度包括运行速度与游戏开发、验证和迭代效率。长期约束与取舍规则见 [项目原则](docs/principles/README.md)，测试、格式、中文提交和验收流程见 [贡献指南](CONTRIBUTING.md)。

## 文档入口

- [完整设计](docs/superpowers/specs/2026-10-07-repository-and-runtime-design.md)
- [近期实施计划](docs/superpowers/plans/2026-10-07-foundation-and-asset-audit.md)
- [项目原则](docs/principles/README.md)
- [工程原则](docs/principles/engineering.md)
- [游戏开发原则](docs/principles/game-development.md)
- [全项目路线图](docs/roadmap.md)
- [模块与依赖边界](docs/architecture/README.md)
- [参考版本核对记录](docs/behavior/reference-baseline.md)
- [切片阶段性结果](docs/behavior/vertical-slice-results.md)
- [资源说明与清单](assets/README.md)
- [贡献与验证](CONTRIBUTING.md)

## 目录

```text
crates/            pvz-core、pvz-data、pvz-assets、pvz-app
tools/             pvz-asset-tool 资源审计 CLI
assets/            版本化的原版参考资源与清单
tests/             自制测试夹具与回放的约定
docs/              设计、计划、行为证据和技术决策
.github/           结构、实现与平台构建检查 CI，Issue/PR 模板
```

## 工具链与当前验证

使用 Rust 2024 Edition、Cargo resolver 3，工具链固定为 1.99.0，包含 rustfmt 与 Clippy。
项目尚未宣布支持更早的最低 Rust 版本。

当前可以验证 workspace、基础 crate、资源工具和开发切片：

```sh
cargo metadata --no-deps --locked --format-version 1
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
git diff --check
cargo run -p pvz-asset-tool --locked -- audit --format json
cargo run -p pvz-asset-tool --locked -- inspect-reanim PeaShooter
cargo run -p pvz-app --locked -- --probe --smoke 120 --screenshot
```

已生成并提交 `Cargo.lock`，CI 已启用格式、检查、测试、Clippy、文档构建及依赖审计。
具体任务见实施计划，完整验收命令见贡献指南。默认可玩切片使用 `cargo run -p pvz-app --locked`，此命令打开交互窗口；后台验证使用 `cargo run -p pvz-app --locked -- --smoke 600 --slice-smoke --screenshot`。Web 构建及启动见 [Web 入口](web/README.md)，基础设施实测范围见 [平台探针结果](docs/behavior/platform-probe-results.md)，当前切片进度见 [阶段性结果](docs/behavior/vertical-slice-results.md)。

## 原版资源与许可

仓库包含经过记录和校验的原版参考资源，位于 `assets/original/`，清单和导入规范见 [资源说明](assets/README.md)。原版可执行文件、截图、录像、存档和临时派生产物不进入仓库。
原版参考资源保留独立的来源与许可边界；MIT 仅覆盖本项目原创代码、文档和明确原创的测试数据，不把原版资源重新声明为 MIT。资源导入时保留来源、原始指纹以及必要的格式规范化记录。
本项目与原游戏权利方无关联，也不声称已完成经过正式隔离审查的 clean-room 开发流程。
