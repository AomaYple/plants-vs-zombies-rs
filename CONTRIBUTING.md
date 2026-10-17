# 贡献指南

本指南面向人类贡献者和自动化代理，说明如何在本仓库提交可审阅、可验证、可回退的改动。代理还必须遵守执行环境提供的全局 `AGENTS.md` 和仓库 [AGENTS.md](AGENTS.md)；普通贡献者无需配置维护者的个人全局文件。长期约束见 [项目原则](docs/principles/README.md)，当前阶段见 [路线图](docs/roadmap.md)，行为结论见 [行为结果](docs/behavior/README.md)。

## 开始前

1. 从最新 `main` 开始，检查 `git status --short --branch`、适用规则、当前计划和目录消费者。
2. 读取[当前总计划](docs/superpowers/plans/2026-10-17-phased-development.md)与对应阶段详案，明确本次目的、行为合同、受影响 package/feature/平台，以及必须保留的错误和回放边界。
3. 先查阅已有模块、测试、夹具、基准和成熟依赖；不为测试便利扩大公开 API，不预建没有消费者的目录或 package。
4. 将实现、测试、文档、配置和计划保持在同一个可独立回退的目的内。原版行为测量、开发参数、工程门槛和玩家平台验收分别记录。

当前 canonical 文档名是 `CONTRIBUTING.md`；不要新建或引用 `CONTRIBUTE.md`。工作区中的 `target/`、`artifacts/` 和本地验证日志是忽略的生成目录，原版程序、截图、录像、存档、凭据和临时派生产物不进入 Git。

## 阶段开发与集中实机验收

按[游戏开发效率](docs/principles/game-development.md#游戏开发效率)组织较大的关联需求批次，具体任务在当前阶段计划维护，使用以下步骤：

1. **确定范围：** 锁定阶段的必做/可延后需求、直接依赖、受影响平台、验收场景和已知问题；远期阶段进入实施前再细化文件与接口。
2. **集中开发：** 连续推进阶段内部任务，同步完成相关测试、回放、覆盖率和基准。日常使用下述 L1/L2 短反馈，完整目的提交前完成全部适用门槛；无需为每个动作或接口组织完整实机交付。
3. **形成待测版本：** 整批开发具备后，通过全部适用工程检查，记录代码版本或差异指纹、锁文件/资源/内容身份、target/feature、启动命令和已知问题。
4. **集中实机测试与调整：** 在同一待测版本上覆盖正常玩法、错误与边界、控制状态、诊断和音画；同一次有效运行可验证多个需求。统一记录问题，修复后先自动回归，再复测受影响的实机场景；新改动影响的旧证据重新取得。
5. **验收交付：** 核对阶段必做项和实际平台矩阵，更新任务/结果及入口摘要，按授权完成 PR、CI、合并和清理。开发完成、工程通过、实机待验、调整中与交付完成分别记录。

阶段内部可以形成多个完整目的提交/PR，不维护长期阶段分支；每个合入仍满足适用验收并保留既有可玩流程。集中实机安排不推迟自动化测试或降低 CI；真实设备才能回答的风险可提前做最小探针。缺少必要实机环境时记录影响和恢复条件，独立开发可以继续，对应平台和阶段保持待验。

## 工具链与目录

- 使用 [rust-toolchain.toml](rust-toolchain.toml) 声明的 Rust 1.99.0、rustfmt、Clippy 和需要的 target；Edition、resolver、workspace lint 和依赖以 [Cargo.toml](Cargo.toml) 为准。验收命令使用 `--locked`，不要手改 `Cargo.lock`。
- `crates/` 放游戏 package，`tools/` 放宿主 CLI，package 集成测试和基准分别放各自的 `tests/`、`benches/`；根目录 `tests/` 只放跨 package 的原创夹具和回放约定。
- `assets/original/` 是清单登记的版本化参考资源，`assets/third-party/` 是有许可和指纹的第三方资源，`content/` 是本项目原创内容，`docs/` 保存规则、设计、计划和证据索引。
- 额外工具按需安装并核对版本：`cargo-deny 0.20.2`、`cargo-llvm-cov 0.9.1`、`wasm-bindgen-cli 0.2.129`、`actionlint 1.7.12`。工具输出与项目依赖审计分开记录。

## 编码、命名与仓库卫生

- 可读文本必须是 UTF-8、LF、无 BOM、末尾有换行；遵守 `.editorconfig` 和 `.gitattributes`。项目自写正文、注释、任务说明、表单提示、显示名称和诊断使用中文；命令、标识符、资源 ID、标准格式和来源原文保留原写法。
- 使用标准 Cargo 布局和 Rust 命名。代码、模块、目录和文档必须有实际消费者、生成入口、验收目的或职责；删除或移动前检查调用方、测试、配置、文档和生成入口。
- 资源、锁文件、生成物和二进制分别按清单和忽略规则处理。新增或变更参考资源必须同步来源、大小、SHA-256、编码和清单；不提交未登记载荷、缓存、凭据或个人路径。
- 工作流改动使用 actionlint 或等价语法检查；Cargo 配置改动核对元数据和锁定依赖；文档变更核对链接、标题、状态和命令。

## 测试、覆盖率与性能

正确性第一，速度第二，内存第三。测试默认并行，先隔离临时目录、资源输入和共享状态；确实不能隔离时只对最小范围串行并说明原因。性能优化不得改变状态摘要、事件顺序、确定性回放、错误边界或平台合同。

每个实现或缺陷修复都交付能捕获目标问题的测试；探索探针进入正式交付前补齐回归。规则、解析、资源和热路径变更按影响接入覆盖率、回放和代表性基准。覆盖率不替代行为测试、基准、实际运行或原版行为证据。

性能结论必须使用相同提交、工具链、runner/主机、target、feature、缓存状态、jobs、输入和测量方式；至少三次有效重复。超时、非零退出、进程树未收口、采样中断、路径污染和失败重试样本作废。进程树工作集只能称为已观察工作集下界，不能称为精确 Rust 堆或显存。没有同条件对照时只记录读数，不改变默认调度。

## 反馈层级

开发时选择能覆盖当前风险的最低层级；提交前完成所有适用层级。较低层级不能替代高层级门槛。

| 层级 | 适用场景 | 主要检查 |
| --- | --- | --- |
| L0 | 文档、计划、结构、入口 | 链接、编码、资源/跟踪集合、`git diff --check`、元数据 |
| L1 | 局部 Rust、解析、工具或配置 | fmt、受影响 package 测试和 Clippy、受影响脚本 |
| L2 | 规则、回放、资源合同、热路径 | 相关覆盖率、确定性回放、Criterion 和平台/窗口检查 |
| L3 | 完整目的交付 | 适用 L0–L2、workspace、feature、doc、deny、Web/平台、Git/CI |

本地反馈入口：

```powershell
pwsh -NoProfile -File tools/test-feedback.ps1 -Mode package -Package pvz-core
pwsh -NoProfile -File tools/test-feedback.ps1 -Mode workspace
pwsh -NoProfile -File tools/test-feedback.ps1 -Mode probe
pwsh -NoProfile -File tools/test-feedback.ps1 -Mode full
```

脚本只复用既有 Cargo 参数并保留退出码，不强制全局串行，不替代覆盖率、基准、窗口、Web 或平台验收。需要并发构建时为调用设置独立 `CARGO_TARGET_DIR`；不要让多个 Cargo 进程争抢同一 target。使用 `tools/measure-cargo.ps1` 时，任何身份校验、收口或清理失败都使样本无效。

## 验证流程

以下命令从仓库根目录执行。按实际改动补充局部检查；Rust、依赖、构建配置和热路径变更至少完成默认与适用 feature 的 workspace 门槛。

### 文档、结构和资源

```powershell
cargo metadata --no-deps --locked --format-version 1
git diff --check
git diff --cached --check
git ls-files
```

检查修改后的文档链接、阶段描述、计划状态、工作流语法、资源清单、跟踪文件集合、UTF-8/LF/无 BOM/末尾换行和尾随空白。结构 CI 在 [.github/workflows/ci.yml](.github/workflows/ci.yml) 中执行完整清单 schema、来源指纹、路径分类和资源哈希检查；不要用简化脚本替代它。

### Rust、依赖和文档

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --locked --no-fail-fast
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo deny check
```

当 feature 可同时启用时追加：

```powershell
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-features --locked --no-fail-fast
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --all-features --no-deps --locked
```

`cargo test --workspace` 保留 doctest；不能用 `--all-targets` 代替它。平台互斥 feature 必须按实际有效矩阵分别验证，不用一次全开冒充平台支持。

### 覆盖率、基准和平台

涉及规则、解析、资源或热路径时运行受影响覆盖率和基准：

```powershell
cargo llvm-cov --workspace --locked --summary-only
cargo bench --workspace --locked
```

覆盖率默认与 all-features 使用独立、干净的插桩 target；核对报告路径属于当前 checkout、分母和遗漏范围。基准独占构建和运行资源，记录输入规模、seed、规则版本、预热、计时范围和未覆盖路径。

Web 入口见 [web/README.md](web/README.md)。常用构建检查：

```powershell
cargo build -p pvz-app --target wasm32-unknown-unknown --release --locked
wasm-bindgen --target web --out-name pvz_app --out-dir target/web target/wasm32-unknown-unknown/release/pvz-app.wasm
node --experimental-vm-modules --test web/startup.test.mjs
```

Windows 实现检查还包含后台无资源/登记资源截图、失败场景原生退出码和进程树工具回归；这些只证明对应 runner 的最小绘制、工具合同和退出码，不替代真实浏览器手势、设备音频、移动端实机或原版行为验收。macOS/Linux 的 CI 构建也不等于目标窗口运行。

## Git、评审和交付

- `main` 是唯一长期分支。独立目的从最新 `main` 创建短期分支，代理使用 `codex/<purpose>`；不在共享 checkout 中并发切换分支。
- 提交围绕一个完整目的，标题使用 `type(scope): 中文说明`，必要正文说明原因、行为影响、验证和限制。只暂存本次目的所需的代码、测试、文档和配置。
- 提交前检查 `git status`、暂存差异、文件来源和适用命令。未执行、失败、跳过或作废的检查必须写清楚；不得删除有效测试、隐藏失败或改写基线制造通过。
- 默认通过 PR 合入。用户明确授权时，代理可推送、创建 PR、等待必要 CI、合并、核验主线 CI、删除短期分支和清理 worktree；这些步骤仍分别记录。合入后执行 `git fetch --prune origin`、`git pull --ff-only origin main`，确认 `main` 与 `origin/main` 同步、工作区干净。
- 推送、合并和清理不得带入凭据、缓存、构建产物、旧 worktree、未登记资源或无关改动。共享历史改写和强推需要单独明确授权。

## 结果报告

最终报告分开说明：代码/文档/配置变化；本地验证命令和结果；覆盖率、基准与内存口径；平台和原版未验证项；提交、推送、PR、合并、主线 CI 和分支清理状态。工程通过、开发参数、后台冒烟和 CI 结果不能升级为玩家平台或原版行为结论。

规则、方法或 CI 变化时同步 [AGENTS.md](AGENTS.md)、[docs/principles/](docs/principles/README.md)、[docs/testing/README.md](docs/testing/README.md)、[docs/performance/README.md](docs/performance/README.md) 和当前计划；历史结果页只修正导航或明确事实错误，不重写当时的验收范围。旧计划由新计划接管时增加迁移说明并指向承接项，保留历史勾选和证据，避免两处同时维护当前待办。
