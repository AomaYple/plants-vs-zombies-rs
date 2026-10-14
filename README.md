# Plants vs. Zombies — Rust 重写

**plants-vs-zombies-rs** 是使用 Rust 与 Bevy 实现的非官方跨平台重写项目。
目标是以版本化的原版参考资源和可观察行为为参考，重新编写游戏逻辑、动画播放器、渲染、音频接线、关卡系统与工具。

## 当前状态

仓库已完成 **首个白天开发切片及现有白天内容数据化的本地实现验收**：`pvz-data`、`pvz-core`、`pvz-assets`、`pvz-asset-tool` 和 `pvz-app` 已注册，提供确定性模拟、资源清单/XML 审计、宿主 CLI、Bevy 视口/音频探针，以及 5×9 棋盘、两种植物、普通僵尸、阳光、豌豆和割草机的可玩流程。默认切片现由严格版本化原创 JSON 驱动，core 纯装配保留默认逐 tick 摘要和事件，app 消费实际模拟频率。内容批次的任务独立复核、默认/all-features 门槛、覆盖率、七个受影响基准、Windows 隐藏胜利冒烟及 Web 指定交互已通过；整体静态复核与最终结构检查已完成，未发现生产代码缺陷，文档修复限定复核已通过。Git 交付仍按实际 PR 和 CI 记录核验，证据见 [版本化内容结果](docs/behavior/content-foundation-results.md)。

此前切片输入、截图退出、音乐生命周期及整分支复核已通过，历史 Web 胜利与 DPR 1 下 2K/4K 输入映射、Windows 音频范围见 [切片验收结果](docs/behavior/vertical-slice-results.md)。版本化内容批次没有重复全部历史平台检查。S1 已在 Windows/Web 正常入口完成开发防守关卡的失败、胜利、终局冻结、重开与再次种植，当前源码的 Windows 默认成功/失败隐藏冒烟也通过；本轮没有重跑 Web 默认十波完整胜利，也没有实听设备音频。S1 已完成开发闭环交付：双终局、错误退出码修复、两组合各 194 项测试、干净覆盖率、七基准及最终独立复核通过，修复后的 WASM 包完成启动/F失败/R重开回归；目的提交 `f1850b1ce5bad82b91338867c3f4cca8387410fc` 已推送 `main`，对应 [CI 运行 38502468453](https://github.com/AomaYple/plants-vs-zombies-rs/actions/runs/38502468453) 八个 job 全部成功，详细范围见[本轮双终局结果](docs/behavior/content-foundation-results.md#2026-10-14-白天单关双终局闭环)。原版窗口曾通过资格观察，当前三次教程重复及窗口控制恢复见[参考基线](docs/behavior/reference-baseline.md#2026-10-13-首个端到端案例登记)；原版时基/规则、完整动态表现和其余平台运行继续待验收。

当前[总计划](docs/superpowers/plans/2026-10-13-minimum-playable-vertical-slices.md)按阶段性最小可玩垂直切片推进：已有切片是 S0 历史起点，S1 已完成开发闭环，S2-A 向日葵动态 idle 已完成独立可玩交付，提交 `3e6634e` 的八项 CI 全部成功；当前进入 [S2-B1 普通僵尸 walk](docs/superpowers/plans/2026-10-14-zombie-walk.md)细化实施，随后逐对象接入动态表现、增加策略与两关进度，再逐关完善地形、冒险和模式。每个阶段保留一个完整可玩版本，六平台适配随阶段推进。[原版行为测量专题计划](docs/superpowers/plans/2026-10-12-reference-behavior-measurement.md)作为横向证据轨逐参数采纳；窗口阻塞只影响直接依赖它的任务，开发参数和工程验证继续与原版一致性证据分开记录。
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
- [原版行为基线测量设计](docs/superpowers/specs/2026-10-12-reference-behavior-measurement.md)
- [当前总计划：阶段性最小可玩垂直切片](docs/superpowers/plans/2026-10-13-minimum-playable-vertical-slices.md)
- [当前细化计划：普通僵尸 walk（S2-B1）](docs/superpowers/plans/2026-10-14-zombie-walk.md)
- [已完成增量：向日葵 idle（S2-A）](docs/superpowers/plans/2026-10-14-sunflower-idle.md)
- [横向专题计划：原版行为测量与并行开发](docs/superpowers/plans/2026-10-12-reference-behavior-measurement.md)
- [已完成：版本化白天内容计划](docs/superpowers/plans/2026-10-12-versioned-day-content.md)
- [已完成：基础与切片历史计划](docs/superpowers/plans/2026-10-07-foundation-and-asset-audit.md)
- [项目原则](docs/principles/README.md)
- [工程原则](docs/principles/engineering.md)
- [游戏开发原则](docs/principles/game-development.md)
- [全项目路线图](docs/roadmap.md)
- [模块与依赖边界](docs/architecture/README.md)
- [参考版本核对记录](docs/behavior/reference-baseline.md)
- [版本化白天内容结果](docs/behavior/content-foundation-results.md)
- [切片验收结果](docs/behavior/vertical-slice-results.md)
- [资源说明与清单](assets/README.md)
- [贡献与验证](CONTRIBUTING.md)

## 目录

```text
crates/            pvz-core、pvz-data、pvz-assets、pvz-app
tools/             pvz-asset-tool 资源审计 CLI
assets/            版本化的原版参考资源与清单
content/           本项目原创的版本化关卡与开发参数
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

Windows CI 还复用实现检查的编译产物执行隐藏窗口截图冒烟，探针拒绝缺少原创红蓝夹具的画面；最近的修复、覆盖率及诊断范围见[仓库质量复核补充](docs/behavior/content-foundation-results.md#2026-10-13-仓库质量复核补充)。
当前具体任务见[阶段性最小可玩垂直切片计划](docs/superpowers/plans/2026-10-13-minimum-playable-vertical-slices.md)，原版测量任务见[横向专题计划](docs/superpowers/plans/2026-10-12-reference-behavior-measurement.md)，完整验收命令见贡献指南。默认可玩切片使用 `cargo run -p pvz-app --locked`，此命令打开交互窗口；运行后按 `F` 进入明确开发失败场景，按 `R` 重开。后台验证使用 `cargo run -p pvz-app --locked -- --smoke 600 --slice-smoke --screenshot`。Web 构建及启动见 [Web 入口](web/README.md)，基础设施实测范围见 [平台探针结果](docs/behavior/platform-probe-results.md)，本批内容迁移与运行证据见 [版本化内容结果](docs/behavior/content-foundation-results.md)。

## 原版资源与许可

仓库包含经过记录和校验的原版参考资源，位于 `assets/original/`，清单和导入规范见 [资源说明](assets/README.md)。原版可执行文件、截图、录像、存档和临时派生产物不进入仓库。
原版参考资源保留独立的来源与许可边界；MIT 仅覆盖本项目原创代码、文档和明确原创的测试数据，不把原版资源重新声明为 MIT。资源导入时保留来源、原始指纹以及必要的格式规范化记录。
本项目与原游戏权利方无关联，也不声称已完成经过正式隔离审查的 clean-room 开发流程。
