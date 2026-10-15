# 测试速度与内存优化设计

日期：2026-10-15
状态：已实施；本地正确性、覆盖率、benchmark、Windows 冒烟与 Web 构建已验证，远端 CI/合并状态待交付阶段核对

## 目标和边界

本任务优化本地测试反馈和 GitHub Actions 持续集成的验证调度，在保持既有正确性门槛的前提下减少等待时间，并在速度不下降时降低测试与验证过程的峰值内存。优化对象是测试、编译复用、缓存和任务调度；游戏规则、资源内容、回放格式、平台支持声明和原版行为合同不在本轮改变范围内。

本轮的成功条件同时满足以下三点：

1. 默认 workspace 测试、`probe` feature 测试、doctest、Clippy、文档生成、覆盖率、基准编译、资源审计和适用平台冒烟仍有明确且可重复的门槛。
2. 在相同提交、工具链、目标平台、feature、缓存冷热状态和负载下，本地相关反馈或 CI 关键路径有可复核的墙钟改善；没有把减少检查、删测试或隐藏失败计入速度收益。
3. 需要声称内存改善的路径提供根进程与后代进程树峰值工作集或适用的分配统计，并说明测量范围、采样间隔、预热和限制。启动失败、超时、树未收口或采样中断的结果作废；Criterion 时间结果不能单独作为内存证据。

本轮不预设自动性能门槛。只有在同一提交、工具链、runner、target、feature、缓存状态和 jobs 下完成至少三次有效重复，且墙钟中位数不高于基线加 10% 容差时，才可把结果写成速度收益；内存收益还须有可重复的峰值下降。否则只记录测量结果，不据此改变默认调度。

本地实施结果已写入[性能测量记录](../performance/README.md#2026-10-15-测试反馈优化执行结果)。默认与 all-features 覆盖率均使用独立插桩 target 并完成 JSON/LCOV 路径核对；实际 workspace benchmark 完成三次有效测量，冷 target 与热 target 分开记录。热 benchmark 墙钟中位数约 182.041 秒、进程树峰值工作集中位数约 166.06 MB，但本轮没有同条件的优化前三次基线，因此不把它们写成速度或内存收益。Windows 冒烟和默认/probe WebAssembly 绑定已完成；浏览器手势、设备音频、Android/iOS 和原版行为仍按正确性边界单独保留为未验证项。

不在本轮目标中的内容包括：改写核心模拟、改变事件顺序或确定性、引入无必要的测试框架、把所有测试强制串行、删除有效的兼容性组合、把 CI 通过扩展为平台玩家验收，或把开发参数扩展为原版行为结论。

## 当前基线和证据边界

当前主线提交为 `63572b8 chore: 合并仓库阶段性维护收尾 (#4)`，五个 workspace package 为 `pvz-data`、`pvz-core`、`pvz-assets`、`pvz-app` 和 `pvz-asset-tool`。本设计所在分支从与 `origin/main` 同步的 `main` 创建。`docs/handoff.md` 已按此前任务删除，不恢复。

首轮本地基线使用同一 Windows checkout、stable 1.99.0、锁定依赖和共享默认 `target/`，结果如下：

| 场景 | 命令 | 实测读数 | 解释 |
| --- | --- | ---: | --- |
| 冷的 workspace 测试 | `cargo test --workspace --locked --no-fail-fast` | 约 379.085 秒 | 首次编译约 5 分 48 秒，不能把编译等待归入断言执行 |
| 热的完整测试 | 同上 | 约 12.341 秒 | 当前缓存条件下的开发反馈基线 |
| 热的编译检查 | `cargo test --workspace --locked --no-run` | 约 2.52 秒 | 说明重复编译和缓存复用是主要候选方向 |
| 测试执行部分 | `cargo test --workspace --locked --no-fail-fast` 的热日志 | 约 10 秒量级 | 约 223 个执行目标/测试条目全部通过；数字用于范围定位，不作为永久覆盖率门槛 |

近期主线 CI 历史记录 `artifacts/validation/repository-maintenance-2026-10-14/ci-main-jobs.json` 的墙钟读数约为：结构检查 25 秒、Web 构建 331 秒、依赖审计 24 秒、默认覆盖率与基准入口 598 秒、all-features 覆盖率 182 秒、Ubuntu 实现检查 291 秒、macOS 实现检查 510 秒。此前 PR 的 Windows 实现检查约 1410 秒，是历史上最突出的等待项；它不是本次主线的同提交基线，只作为瓶颈线索。

当前 CI 的实现 job 在每个操作系统中顺序执行默认检查、测试、Clippy、文档、Windows 冒烟、`probe` Clippy，再执行 workspace `--all-features` 的 check/test/Clippy/doc。覆盖率 job 已按默认和 all-features 拆分，但各 job 目前没有项目级 Cargo 缓存。所有 Cargo 工作共享一个 job 内的 `target/`，因此不应在同一 checkout 中并发争抢它。

`probe` 是 `pvz-app` 的唯一项目 feature。源码中唯一的 feature 条件是 `options.probe |= cfg!(feature = "probe")`；没有发现其他 workspace package 的项目 feature 条件。当前 `cargo test --workspace --all-features` 因而很可能重复运行未受 feature 影响的四个 package，但这一判断在实施前要由 feature 行为测试和静态审计确认。没有确认前，不收窄 all-features 测试范围。

探索阶段的 `measure-working-set.ps1` 曾记录约 122–124 MB，但读取到的 Cargo 退出码为 `null`，进程树收口不完整。该数据只作为待修正脚本的探索记录，不能作为正式峰值或内存改善证据。

## 正确性不变量

优化前后必须保持以下合同：

- 默认和所有有效 feature 组合都能编译；`cargo test --workspace --locked` 保留 doctest。
- 核心规则、事件顺序、暂停/终局/重开、回放摘要和确定性结果逐批一致。
- 资源清单、路径边界、原创夹具、错误诊断脱敏、编码和锁文件检查保持原门槛。
- 覆盖率默认与 all-features 使用各自干净的插桩 profile，分母、路径和遗漏独立记录；不以更换分母制造改善。
- Criterion 基准的输入规模、seed、规则版本和计时范围保持一致；终局冻结的廉价调用不计作完整推进。
- Windows、macOS、Linux、Web、Android、iOS 的支持状态仍按各自实际构建或运行证据报告。CPU/CI/CLI 结果不升级为窗口、设备音频或原版行为验收。
- 测试默认并行；真实窗口、GPU、音频或共享外部状态只在最小范围串行。不能用整个 workspace 的 `--test-threads=1` 掩盖隔离问题。
- 任一测试失败都以非零结果暴露；超时、进程未收口、缓存损坏和工具缺失分别报告，不通过重试次数或忽略失败制造通过。
- 子代理只做只读审计和复核，主代理负责源码、脚本、CI、Cargo 配置和文档修改；任何授权实施的子代理都必须使用隔离 worktree 并明确文件归属。

## 方案一：先固定可重复的测量和本地反馈层级

第一步只增加测量入口和文档，不改生产代码。修正 PowerShell 进程树采样脚本，使其记录根 Cargo 进程、子进程、峰值工作集、开始/结束时间和真实退出码；等待整个进程树退出后再写结果。每次测量使用独立 `CARGO_TARGET_DIR`，避免历史产物和并行 Cargo 任务污染结果。至少重复冷编译、热测试、`--no-run`、all-features 测试、覆盖率插桩和基准编译，并分别保存原始 stdout/stderr。

本地反馈按风险选择最小充分层级：

| 层级 | 入口 | 适用场景 | 保留的门槛 |
| --- | --- | --- | --- |
| 快速包级 | `cargo test -p <受影响包> --locked` | 单个规则、解析或表现改动 | 受影响 package 的全部测试和 doctest |
| 快速组合 | 一次 Cargo 调用选择受影响的多个 package | 跨 crate 接口改动 | 共享一次编译，避免多个 Cargo 进程争抢同一 target |
| feature 回归 | `cargo test -p pvz-app --features probe --locked` | `probe` 或 feature 条件改动 | 专门的 feature 行为测试 |
| 提交验收 | 贡献指南列出的 workspace、覆盖率、基准和平台命令 | 完整目的交付 | 全部适用 L0–L3 门槛 |

快速层级只缩小已知影响范围，不改变提交验收。以后若增加包装脚本，脚本只负责选择上述命令、记录参数和统一退出码，不隐藏 Cargo 输出，也不默认改变测试线程数或编译 jobs。

## 方案二：验证 `probe` feature 后消除可证明的重复测试

在 `pvz-app` 增加一个不依赖窗口的纯 feature 回归，直接锁定编译期默认模式：默认构建的 `Options` 不自动进入 probe，`--features probe` 构建自动进入 probe；同时保留现有运行时参数冲突和截图/资源测试。这样 feature 行为有独立断言，不再依赖“all-features 全 workspace 测试碰巧经过了 `cfg!`”。

实施时先做源码和 metadata 审计：确认项目 feature 只有 `pvz-app/probe`，确认没有隐藏的 `cfg(feature = ...)`、build script 或 feature 专属测试。首轮只删除可证明重复的 probe Clippy；workspace `--all-features` test 仍保留。未来若要收窄测试矩阵，必须补充同等测试数量、行为和覆盖率分母证据，并同步更新 CI 说明和本设计对应的验收矩阵。

如果审计发现新的 feature 条件、依赖 feature 传播或平台差异，立即恢复 workspace all-features 测试作为该组合的门槛，并把新组合加入矩阵；在此之前不以测试时间减少宣称收益。

## 方案三：本地编译复用和内存控制

本地优化优先复用同一 Cargo 调用和已有缓存：受影响 package 组合在一次命令中执行，`--no-run` 只用于需要先验证编译的场景，完整测试随后消费同一 profile。覆盖率使用独立插桩 target，基准不与构建、测试、窗口或 Web 服务重叠。不同 checkout 或 profile 切换前按现有测试策略清理会污染统计的插桩产物，保留 registry/git 依赖缓存。

内存优化只在正式测量显示不降低速度时采用。候选项按以下顺序评估：

1. 防止同一共享 target 上启动多个 Cargo 进程，先消除重复 rustc 和链接进程的峰值叠加。
2. 比较默认 Cargo 编译 jobs 与受限 jobs 的时间/峰值工作集曲线；只有在相同正确性门槛下速度不退化，才为受内存约束的本地或 CI 环境设置受控值。
3. 保持 Rust 测试线程默认并行；只有定位到测试本身的外部资源冲突时，才对最小测试目标局部串行。
4. 若夹具或重复解析被证明确实占用主要内存，再复用只读输入或缩短生命周期，并用回放、事件顺序和资源错误测试保护行为。

不预先设置全局 `--jobs 1`、`--test-threads=1`、全局可变缓存或共享临时目录。它们可能降低峰值，却会把速度或隔离问题转移到不可见的等待和顺序依赖。

## 方案四：CI 缓存和调度

CI 缓存使用 `actions/cache@v4` 的显式 key，至少包含 runner OS、Rust 1.99.0、`Cargo.lock` 哈希、命令角色、target/架构、profile 和 feature 集；默认、all-features、覆盖率插桩和 Web target 不共用可执行构建产物。registry/git 依赖缓存可跨同一 OS 和锁文件复用，`target` 缓存按 job 角色隔离。缓存命中只减少编译等待，仍执行完整命令；缓存恢复失败自动视为冷缓存继续运行。

CI 任务安排按依赖关系保持最大安全并行：结构检查结束后，依赖审计、覆盖率矩阵和 Web 构建继续独立运行；implementation 保留三个 OS 和原状态检查名，三个 OS 都执行默认门槛，all-features check/test/Clippy/doc 只在 Ubuntu 条件执行一次，除非分支保护明确要求各 OS 重复状态。每个 runner 内仍按单一 target 顺序运行 Cargo 命令，Windows GPU/截图/失败冒烟继续局部顺序，并复用同一默认构建产物。不得未经同 SHA 冷热测量把 OS × feature 六 runner 当作已证实优化。

在 all-features 测试收窄前，矩阵保留现有 workspace all-features 测试；确认 `probe` 审计和专门回归只作为补充证据，不自动替代 workspace all-features test。任何未来矩阵变更都同时更新 CI 说明和本设计对应的验收矩阵。

CI 速度比较至少记录：提交 SHA、runner image、工具链、缓存命中/未命中、job 阶段、每步墙钟、失败重试和总墙钟。不同 runner、不同缓存状态或删减检查不能直接作前后优化结论。若缓存命中率低、key 造成磁盘超限或构建产物出现跨 feature 污染，立即回退到依赖缓存或旧调度。

## 内存测量口径

正式内存结果分为编译验证和测试运行两类。编译验证记录 Cargo 进程树中所有相关进程的峰值工作集，测试运行至少记录测试二进制及其子进程峰值；若无法可靠区分系统共享页，明确标记为 OS 工作集而非 Rust 堆。测量脚本必须输出真实退出码，出现采样中断或树未收口时结果作废。

每个候选方案在冷缓存和热缓存各重复至少三次，使用同一输入、同一 `target` 隔离方式和同一 jobs 参数，报告中位数及离散范围。冷缓存清空场景 target 但保留 registry/git，热缓存使用同一场景连续第二次运行；启动失败、超时、树未收口或退出码缺失的样本不得进入统计。只有峰值下降且墙钟中位数不超过基线 10% 容差时才接受内存取舍；核心结构或分配策略改变时追加分配次数/字节数，GPU 资源另行记录，不能用进程工作集替代显存。

## 验收矩阵

实施完成后按下表逐项核验，结果写入忽略的原始证据目录并在 PR 描述中引用：

| 领域 | 必须执行 | 通过含义 | 不可推出的结论 |
| --- | --- | --- | --- |
| 格式/结构 | `cargo fmt --all -- --check`、`cargo metadata --no-deps --locked --format-version 1`、`git diff --check`、资源/编码检查、actionlint | 入口和跟踪内容符合仓库合同 | 不代表游戏可玩或原版一致 |
| 默认正确性 | `cargo check --workspace --all-targets --locked`、`cargo test --workspace --locked`、Clippy、doc | 默认代码、doctest、lint、文档通过 | 不代表 all-features 或其他平台运行 |
| feature 正确性 | all-features check/Clippy/doc；默认 workspace 测试及专门 `probe` 测试（或审计未通过时全 workspace all-features 测试） | feature 条件有明确覆盖 | 不代表窗口、音频或 Web 手势验收 |
| 行为保护 | 确定性回放摘要、事件序列、暂停/终局/重开和资源错误回归 | 优化未改可观察核心行为 | 不代表与原版行为相同 |
| 覆盖率 | 默认/all-features 独立干净插桩，JSON/LCOV 路径和分母核对 | 覆盖率范围可追溯 | 不代表分支完整或平台运行 |
| 性能/内存 | 受影响 Criterion、基准 `--no-run`、三次以上时间/工作集测量 | 同条件下速度与内存结论有证据 | 不能把不同机器绝对时间互比 |
| 平台/交付 | Windows 冒烟、Web 构建/绑定、适用 OS CI、Git 状态与远端 CI | 对应平台证据和提交链完整 | 不代表未运行的 Android/iOS 或原版验收 |

## 回滚条件和风险

出现以下任一情况，撤销对应优化并保留失败证据：缓存命中后出现与冷构建不同的编译/测试结果；feature 收窄后专门回归不能覆盖实际 `cfg` 行为；覆盖率分母或源路径漂移；默认或 all-features 任何门槛被跳过；进程树退出码、artifact 或截图回调无法可靠判断；墙钟改善来自降低 jobs、测试线程或检查范围而没有等价正确性证明；峰值工作集下降伴随速度退化、测试隔离变差或 CI 资源耗尽。

主要风险是 CI 缓存 key 过宽导致跨 feature 污染、矩阵拆分增加 runner 排队、独立 target 缓存占用磁盘、测量脚本把子进程漏算，或“重复测试”判断遗漏未来新增 feature。缓解办法是 key 显式隔离、保留完整状态检查、每次提交重新执行适用门槛、源码 feature 审计与回归测试并行维护。

## 交付顺序

本设计审阅通过后，先提交修正后的测量入口和实施计划，再实施本地反馈、feature 回归、CI 缓存/调度和内存候选项。每个阶段都先验证正确性，再进行同条件前后测量；只有全部适用门槛通过并且证据可追溯，才进入推送、PR、合并和分支清理流程。
