# Plants vs. Zombies — Rust 重写

**plants-vs-zombies-rs** 是使用 Rust 与 Bevy 实现的非官方跨平台重写项目。项目以版本化的原版参考资源和可观察行为为参考，重新实现游戏逻辑、动画播放器、渲染、音频接线、关卡系统和工具。

## 当前状态

截至 2026-10-18：

- 当前按[七阶段总计划](docs/superpowers/plans/2026-10-17-phased-development.md)推进：阶段内集中开发多项关联需求，持续自动化验证，整批具备后集中实机测试与调整。阶段 1“完整白天战斗体验”的计划已建立，游戏开发待启动。
- S1 白天单关开发闭环已完成；S2-A 向日葵 idle 已完成独立可玩交付。
- S2-B1 普通僵尸 walk 已完成源码、测试、工程门槛和 Web 功能证据，但 Windows 正常窗口动态图、Web 诊断、完整平台运行和原版行为重复测量仍待完成；S2-B2 啃食/死亡反馈尚未开始。
- 原版有效重复仍为 `0/3`。当前内容和规则中的开发参数不等于原版规格。
- 最新主线已合入测试资源复用、无分配状态比较、DryRun 副作用修复、CI 并发/下载缓存和测试反馈调度优化。局部 `pvz-app` 测试取得同条件三次前后样本；workspace 整体没有足够的三次纯热样本，因此没有宣称全仓总体提速。
- 测试优化与规则/仓库卫生整合已交付；B1 遗留平台验收及 B2 反馈需求由阶段 1 统一承接。工程检查和 CI 通过不等于六平台玩家验收或原版行为还原完成。

## 能运行什么

从仓库根目录启动开发切片：

```powershell
cargo run -p pvz-app --locked
```

当前切片包含 5×9 草坪、阳光、向日葵、豌豆射手、普通僵尸 walk、基础战斗、割草机、胜利/失败、终局冻结、暂停/恢复和重开。运行后可使用：

- `F`：进入无防守失败场景。
- `R`：重开当前关卡。
- `P` 或空格：暂停/恢复。

后台工程冒烟：

```powershell
cargo run -p pvz-app --locked -- --probe --smoke 30 --screenshot
cargo run -p pvz-app --locked -- --smoke 30 --slice-smoke --screenshot
```

这些命令只验证对应的本地入口、最小绘制或开发合同；真实浏览器手势、设备音频、移动端实机、完整平台安装包和原版行为仍按结果页单独验收。

## 路线和文档入口

- [路线图](docs/roadmap.md)：阶段顺序、当前平台门槛和下一项产品工作。
- [七阶段集中开发总计划](docs/superpowers/plans/2026-10-17-phased-development.md)：当前阶段、依赖、验收标准和历史承接。
- [第一阶段：完整白天战斗体验](docs/superpowers/plans/2026-10-17-phase-1-day-combat.md)：集中开发任务与 Windows/Web 实机矩阵。
- [阶段开发设计](docs/superpowers/specs/2026-10-17-phased-development-design.md)：阶段内开发、自动化与集中实机的组织依据。
- [原版行为测量专题计划](docs/superpowers/plans/2026-10-12-reference-behavior-measurement.md)：原版证据和规则采纳。
- [项目原则](docs/principles/README.md)：长期工程和游戏开发规则。
- [架构](docs/architecture/README.md)：crate、模块、数据流和依赖方向。
- [测试策略](docs/testing/README.md)：测试类型、覆盖率和平台边界。
- [性能策略](docs/performance/README.md)：基准、内存口径和优化结果。
- [行为结果](docs/behavior/README.md)：可核查的事实、命令、环境和限制。
- [资源说明](assets/README.md)：参考资源、清单、许可和公开边界。
- [贡献指南](CONTRIBUTING.md)：工具链、验证、提交、PR 和交付。

## 目录

```text
crates/            pvz-data、pvz-core、pvz-assets、pvz-app
tools/             pvz-asset-tool 和宿主测试/测量工具
assets/            版本化原版参考资源、第三方资源和清单
content/           本项目原创的版本化关卡与开发参数
tests/             跨 package 的原创测试夹具和回放约定
docs/              设计、原则、计划、行为证据和技术决策
.github/           结构、实现、覆盖率、依赖和 Web CI
web/               Web 入口和启动回归
```

## 工具链与基本验证

仓库使用 Rust 2024 Edition、Cargo resolver 3 和 `rust-toolchain.toml` 固定的 Rust 1.99.0，包含 rustfmt 与 Clippy。项目不声明更早 Rust 版本兼容性。常用完整检查见 [CONTRIBUTING.md](CONTRIBUTING.md)：

```powershell
cargo metadata --no-deps --locked --format-version 1
cargo fmt --all -- --check
cargo test --workspace --locked --no-fail-fast
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo deny check
```

需要兼容 feature 时追加 `--all-features` 的 check、test、Clippy 和 doc。Web 构建和 Node 启动回归见 [web/README.md](web/README.md)。

## 资源与许可

`assets/original/` 包含经过清单、来源和指纹记录的原版参考资源；原版程序、截图、录像、存档和临时派生产物不进入 Git、CI 或发行包。`assets/third-party/` 单独记录第三方许可和哈希。MIT 只覆盖本项目原创代码、文档和明确原创的测试数据，不重新声明原版资源、名称、商标或第三方依赖的许可。本项目与原游戏权利方无关联，也不声称已完成正式法律意义上的 clean-room 审查。
