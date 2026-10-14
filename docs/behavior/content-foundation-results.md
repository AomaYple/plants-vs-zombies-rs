# 版本化白天内容验收结果

日期：2026-10-12（UTC；本机 America/New_York 为 2026-10-11）。范围为[版本化白天内容设计](../superpowers/specs/2026-10-12-versioned-day-content.md)与[本批实施计划](../superpowers/plans/2026-10-12-versioned-day-content.md)：将现有开发切片的参数、初始状态和有序波次迁移到原创 JSON，由纯核心装配并由实际应用消费。本页集中维护本批读数；[首个切片结果](vertical-slice-results.md)和[平台探针结果](platform-probe-results.md)保留各自历史用途。

本批 Tasks 1–3 已通过各自独立规范与质量复核，默认/all-features 门槛、干净重测覆盖率、受影响基准以及 Windows/Web 指定流程已在本地通过。整体独立静态复核已完成，规格通过、无生产代码缺陷；唯一文档 Minor 已修复，限定修复复核已通过。root 的最终文本/链接/清单哈希/locked metadata 检查已通过，文档提交后 185 项测试再次通过。PR #3 已以 rebase 合入 `main`，合入提交为 `36f6e10280415fd6005bbd142ff80b8d42ba9f3c`；对应 PR CI 和主线 push CI 各 8 个 job 全部成功。本地验收不代表原版规则已还原或六平台已经发布。

## 内容与行为合同

- [原创默认关卡](../../content/levels/development-day.json)为 1241 字节，schema 1、规则 2.0、依据 `development`、seed 7、关卡 `development-day-slice`、初始阳光 150、60 tick/秒，包含 23 个规则标量和 10 项有序波次。来源与 MIT 边界见[内容说明](../../content/README.md)，它不进入原版资源清单。
- data 的四个 DTO 表达严格格式，所有对象拒绝未知、遗漏和重复字段；规则支持范围和有效性由 core 判断。`GameState::from_day_definition` 只接受规则 2.0 和 1..=1000 Hz，以显式字段转换保留同 tick 波次顺序，并复用既有 `SliceRules::validate`。core 不增加 JSON 生产解析、I/O、资源或引擎依赖。
- `Runtime::from_day_json` 在解析前拒绝超过 1048576 个 UTF-8 字节的输入，复用 serde_json 并保留错误原因；损坏内容、未知版本/字段和非法参数明确失败。`Runtime::day()` 消费编译时嵌入的同一 JSON，native/Web 输入字节相同；probe 仍使用独立空棋盘入口。
- 应用帧推进读取实际模拟频率，以 tick 单位累积小数余量，保持 250ms 帧上限、命令顺序、暂停/终局无追赶及重开清零动画和余量的合同。30 Hz 下 0.25 秒推进 7 tick，120 Hz 下推进 30 tick；重开恢复原 seed、初始状态、全部规则及初始时钟。
- 旧 `SliceRules::development()` 保留为独立迁移对照与既有测试/基准工厂。默认新旧入口比较至少 6000 次调用的完整摘要与事件序列，包含选择、种植、收集、暂停、恢复和重开；实际推进 5989 tick，明确暂停与重开阶段，排除终局冻结造成的空回归。参数作用、seed、同 tick 重复波次及非法边界另有独立测试。

这些数值仍是开发参数。内容迁移不改变规则版本、RNG、碰撞、事件阶段或默认演化；原版时基和规则的证据边界见[参考基线](reference-baseline.md)与[开发切片合同](development-slice.md)。

## 环境、命令与测试

本批起点为 `main` 的 `545a2b2b6338850b1ed5c35e21a7a62b70715b70`，实现范围为 `68cacfe`、`b60cbaf..27b9fce`、`27b9fce..a0c0d6c`。测量在隔离 worktree 完成，构建缓存使用主目录 `target`；Windows 11 10.0.26300、AMD64、Intel Family 6 Model 141、16 个逻辑 CPU，Rust/Cargo 1.99.0、Edition 2024、Bevy 0.20.0、Criterion 0.5、cargo-llvm-cov 0.9.1、wasm-bindgen 0.2.129。Cargo 构建 jobs=2，自动测试保持默认并行，窗口与基准由主代理统一顺序协调。

正式门槛的实际命令如下，全部退出码为 0；完整参数、耗时和逐项日志见本批 `final-commands.json`：

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo deny check
cargo check --workspace --all-features --locked --all-targets
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-features --locked --all-targets -- -D warnings
cargo doc --workspace --all-features --locked --no-deps
cargo build -p pvz-app --locked
```

默认与 all-features 各通过 185 项测试（含 doctest），各 25 个测试入口，零失败。相对本批 156 项基线新增 29 项：data 5、core 15、app 9，覆盖严格格式、真实参数作用、迁移回放及实际时钟/输入边界。各任务先取得缺失接口或旧行为的真实红测，再完成实现与绿测；任务复核未发现未解决的 Critical、Important 或 Minor 问题。各项报告和裁决保留在本机账本，不把静态评审当作重新运行测试。

`cargo deny check` 保留 26 类既有传递依赖重复版本 WARN，advisories/bans/licenses/sources 检查通过；本批仅复用已有 serde_json 依赖版本，没有把退出 0 写成无告警。

## 覆盖率

正式报告来自 Windows native 纯测试，分别运行默认与 all-features 插桩并导出 JSON/LCOV。本批没有追加窗口运行 profile；实际窗口冒烟属于独立运行证据。稳定工具链插桩未统计 doctest，doctest 由上述常规测试验证；Web 条件代码不在 native 分母内，branch count 0 表示未启用分支统计。

首次共享插桩缓存混入已归档 worktree 对象，默认/all 的探索分母为 8272；该报告及阶段性 `coverage-data-core.json` 不作为正式验收。主代理执行 `cargo llvm-cov clean --locked`（退出 0），完整重编译并确认正式 JSON 仅包含本批源码路径；探索记录保存在 `coverage-shared-cache-exploration/`，清理原因和日志保留为 `coverage-clean.json`、`coverage-clean.log`。分母变化来自统计输入清理，不能据此声称覆盖率提升。

实际命令为 `cargo llvm-cov --workspace --locked --no-report` 和加 `--all-features` 的对应入口，随后分别执行 `cargo llvm-cov report --json --output-path …` 与 `cargo llvm-cov report --lcov --output-path …`，均退出 0。完整顺序及导出路径见 `coverage-commands.json`。

| native 纯测试范围 | 默认命中行/总行 | all-features 命中行/总行 | 行覆盖率 |
| --- | --- | --- | --- |
| workspace | 3182/3983 | 3182/3983 | 79.89% |
| pvz-data | 148/173 | 148/173 | 85.55% |
| pvz-core | 822/842 | 822/842 | 97.62% |
| pvz-assets | 676/745 | 676/745 | 90.74% |
| pvz-app | 1362/2041 | 1362/2041 | 66.73% |
| pvz-asset-tool | 174/182 | 174/182 | 95.60% |

新增 app `content.rs` 为 13/13 行；core `content.rs` 为 68/69 行，未命中的是频率范围先行验证后 `SimulationClock::new` 的防御性错误映射。data `day.rs` 是纯 DTO/derive，报告没有单列其可执行源码行，不能为它编造 100% 行覆盖率；严格 JSON 格式由实际解析测试验证。

主要既有遗漏是设备/窗口/绘制接线：`render.rs` 为 0/361，`main.rs` 为 438/713；此外仍有资源错误处理、音频、输入、ID/诊断和核心防御路径未命中。新增内容入口的解析、超限、装配错误及真实频率有测试，后续针对设备丢失、截图 observer、Web 条件路径、真实高 DPI/触摸和未运行平台补证。历史切片追加 Windows 运行的分母保留在原页，不能与本批纯测试直接比较。

## 受影响基准

实际运行 `cargo bench -p pvz-core --locked --bench stepping` 和 `cargo bench -p pvz-app --locked --bench presentation`，退出码均为 0。七个场景均使用 3 秒预热、100 个样本、约 5 秒采样；下表从 Criterion `mean` 和 95% 置信区间提取，计时单位为每次样本所表示的完整工作量。

| 场景与计时范围 | 均值 | 95% 置信区间 |
| --- | --- | --- |
| schema1/规则2.0 原创开发白天 JSON，1241 字节，解析、校验及 Runtime 装配；无文件 I/O | 2.3519 µs | 2.3423–2.3624 µs |
| 规则1.0 空棋盘，1000 次 step/实际1000 tick，每 tick 0 命令，seed7 | 88.9349 µs | 88.0407–90.2626 µs |
| 规则1.0 阳光命令，1000 次 step/实际1000 tick，每 tick 2 命令，seed7 | 93.7678 µs | 92.5601–95.9101 µs |
| 规则2.0 五排，预热45植物（5向日葵/40射手），波次总量100，1000次 step/实际1000 tick，seed7 | 2.9889 ms | 2.9789–2.9995 ms |
| 800×600逻辑视口，3840×2160输入1点的反变换 | 4.8214 ns | 4.8041–4.8394 ns |
| 原创8轨道128帧、12fps开发动画时基索引 | 7.6332 ns | 7.5942–7.6745 ns |
| 双轴角度合同，1个100×90四边形与800×600裁剪 | 373.3414 ns | 371.4342–375.3839 ns |

切片初始化与种植预热在计时外，预热后 tick45、余额5750、初始无僵尸；100是波次总量，非同时存活数量。计时后 tick1045 且仍 Running，终局冻结另有测试，不能把冻结调用当完整更新。场景规模与消费者见[性能方法](../performance/README.md)。

基准与本项目测试、构建、覆盖率及窗口活动隔离；未控制维护者其他活动和 CPU 电源/温度。原始 `estimates.json`、`benchmark.json` 已复制到本批 `criterion/`，避免共享 target 后续覆盖。本批未改 assets 解析，因此没有重测四个资源解析场景，它们继续仅作[切片历史基线](vertical-slice-results.md)。本次没有同条件前后对照或内存/分配测量，不作速度、开发耗时或内存改善主张。

## 平台构建与实际运行

| 范围 | 本批实际证据 | 结论边界 |
| --- | --- | --- |
| Windows 默认切片 | `target/debug/pvz-app.exe --smoke 600 --slice-smoke --screenshot`，隐藏运行600帧，退出0；2次种植、27次收阳光、10次僵尸生成、20次发射/命中、4次割草机、暂停/恢复各1次、Victory1次、Defeat0次；诊断为空 | 默认 JSON 已被实际消费者装配，原版资源和玩法仍运行；1000×750 GPU截图已实际查看，不以生成文件代替视觉核对 |
| Web 默认构建 | `cargo build -p pvz-app --target wasm32-unknown-unknown --release --locked`，随后 `wasm-bindgen --target web --out-name pvz_app --out-dir …/target/web …/pvz-app.wasm`，均退出0 | 构建与 JS/WASM 生成通过，运行证据另列 |
| Web probe 构建 | 同一目标构建加 `--features probe`，生成目录 `target/web-probe`，均退出0 | probe 构建独立，不替代默认入口 |
| Web 默认实跑 | Codex In-app Browser/Chromium WebGL2，ANGLE NVIDIA RTX3060 Laptop；视口与 framebuffer 均1280×720、DPR1；重开暂停显示150与0/10波次，种植向日葵row1 col0和射手row2 col0后为0；恢复观察2/10及命中，收集后暂停显示25与5/10 | 实际原版资源绘制、两卡种植、命中、收阳光、暂停/恢复和重开通过；本批未完整重复胜负流程 |
| Web probe 实跑 | 初始50，中心点击后51，逻辑点(400,300)，暂停tick964、动画16.07秒 | 独立空棋盘最小输入/暂停入口通过，不代表完整切片或原版时基 |

Web 日志保留两个既有可选3D管线能力 WARN 与 CPU 预处理回退信息，二维实际绘制；实际页面与日志分别保存。测试结束已关闭本批浏览器标签，服务器会话退出0。

本批未复验完整 Web 胜负、2K/4K、音乐/设备音频、高 DPI、触摸或其他浏览器。历史 Web 胜利、DPR1 下2K/4K输入及 Windows 音频证据仍在[切片结果](vertical-slice-results.md)/[平台探针结果](platform-probe-results.md)，不改写为本批重测。Linux/macOS、Android/iOS 的构建、实际运行、生命周期、打包与发布继续待验。

## 原版观察与后续依赖

原版 EXE/main.pak 指纹再次核对与[参考身份](reference-baseline.md)一致。经过目标进程绑定和连续标题、菜单、教程场景变化，窗口观察入口本批已通过资格验证；早先无关前台捕获未用作行为证据。冒险1-1新用户教程单次观察到初始阳光150、豌豆射手卡片100、正常种植后余额50。重复次数为1，未录制连续时间轴，未测时基、间隔、碰撞或内部事件顺序，不能推广到全部关卡或全部规则。

2026-10-13 的[阶段性总计划](../superpowers/plans/2026-10-13-minimum-playable-vertical-slices.md)将可玩阶段与横向证据轨分开：原版案例依证据采纳，独立表现、测试与平台缺口分别推进。本页保留本批结果与原始索引；当前任务排序统一维护在总计划和[路线图](../roadmap.md)，不复制另一份后续任务清单。

本机 `web-defeat-exploration.json` 记录默认内容无种植时出现胜利；这个探索没有完成 Web 失败验收，也没有证明所有默认输入都不可能失败。后续先核对受控失败场景，再验证浏览器失败、终局冻结和重开；原创失败夹具与默认内容实跑分别记录。原版失焦推进、完整内容、动态表现和其他平台缺口继续保持未验证。

## 2026-10-14 白天单关双终局闭环

本节记录 S1 在 `a2f16c9` 起点工作区的新增验收，完整目的提交 `f1850b1ce5bad82b91338867c3f4cca8387410fc` 已推送 `main`，其对应 CI 已核验通过，交付详情见本节后文。失败场景为版本化原创内容 `content/levels/development-failure.json`，关卡 ID `development-failure-slice`、规则 2.0、依据 `development`，通过正常入口的 `F` 键进入；默认入口仍加载 `development-day.json`。默认内容无种植时可由割草机完成胜利，等待不能称作默认失败。开发关卡和实跑结果不代表原版规则事实。

### 当前运行证据

当前 Windows 源码的隐藏窗口命令如下，两条均退出 0：

```sh
cargo run -p pvz-app --locked -- --slice-smoke --smoke 600 --screenshot
cargo run -p pvz-app --locked -- --failure-smoke --smoke 400 --screenshot
```

默认成功冒烟记录向日葵/豌豆射手种植、暂停/恢复、阳光生成与收集、豌豆命中和僵尸死亡、割草机清理，于 tick 5767 产生 `Victory`。失败冒烟使用与 F 键共用的入口，首只僵尸由割草机清除，第二只突破，于 tick 4049 产生 `Defeat`。日志中的“帧末 tick”属于批次诊断，不把批内事件都当作发生于该 tick；终局 tick 单独核对。这是加速开发运行，独立于下面的正常输入实跑。

| 平台与入口 | 本轮实际输入及观察 | 证据范围 |
| --- | --- | --- |
| Windows 默认 feature 的正常窗口 | F 进入开发防守关卡，无种植达到失败；持续观察失败冻结，R 重开；正常点击射手卡片和草坪，阳光 150→50、种植成功并实际获胜；持续观察胜利冻结，再 R 重开、正常卡片/草坪点击再次种植，阳光再由 150→50 | 双终局、正常输入、重开与再次种植均来自本轮实际窗口；`native-lost.png`、`native-replanted.png`、`native-won.png`、`native-won-restart-replant.png`，进程已关闭 |
| 退出码修复前同轮 WASM 包的正常浏览器入口 | 启动及资源初始化通过；F 进入同一开发防守关卡并失败，两次失败截图逐字节相同；R 重开、P 暂停，正常点击射手与草坪，阳光 150→50；P 恢复后实际获胜；继续观察胜利视觉冻结，再 R 重开、正常点击再次种植成功，并以 P 暂停确认 | 失败逐字节一致与胜利视觉冻结分别记录，胜利截图字节不同；warn/error 查询为 `[]`；自有标签和回环服务已关闭 |
| Windows 默认关卡与 Web 历史默认关卡 | 当前 Windows 默认成功为上述隐藏冒烟；Web 默认十波胜利、重开后种植沿用[切片历史结果](vertical-slice-results.md) | 本轮没有完整重跑 Web 默认十波胜利；当前 Web 双终局来自正常 F 开发防守关卡，不把历史默认成功改写为当前实跑 |

core/app 回归验证失败内容的胜负、终局拒绝命令、状态/动画冻结与重开恢复合同；Windows GPU 和浏览器正常输入观察分别验证实际消费者。画面冻结不单独证明模拟摘要或音频游标冻结；设备音频本轮没有实听，不扩大历史音频验收范围。当前浏览器日志为空也不能证明历史间歇性 shader unresolved-import 诊断已根治。真实 DPR 2/4、触摸、更多浏览器、其他目标平台与原版一致性继续待验。

### 本地检查与覆盖率

退出码修复后，默认与 all-features workspace 各 194 项测试（含 doctest）通过，比修复前192项新增两条期限边界回归；fmt、两组合 all-targets check、Clippy `-D warnings`、doc、`cargo deny check`、actionlint 及 native 构建均退出 0，deny 仍保留重复传递依赖 WARN。实际命令和结果见 `exit-fix-commands.json`；干净覆盖率及新 WASM 实际回归如下单列，局部静态复核不替代实际运行。

退出码修复后的正式 Windows native 纯测试覆盖率对默认及 all-features 分别先执行 `cargo llvm-cov clean --workspace --locked`，再运行对应 feature 的 `cargo llvm-cov --workspace --locked --no-report`，独立导出 JSON；八条命令全部退出0。两份报告均为3325/4134行，即80.4306%，全部源码路径属于当前 checkout；源码、锁文件、工具链与 CI 文件的 SHA 与 `exit-fix-source-state.json` 一致，未追加窗口 profile。分包读数如下：

| 包 | 命中行/总行 |
| --- | --- |
| pvz-app | 1505/2192 |
| pvz-core | 822/842 |
| pvz-data | 148/173 |
| pvz-assets | 676/745 |
| pvz-asset-tool | 174/182 |

本轮退出码修复后的第一批增量插桩报告曾出现 default-final 分母4231、all-final分母4134的不一致，归为探索，不作为正式验收；随后对两组合分别 clean 重测才取得上表一致分母。修复前3260/4089行也只保留为当时记录，不作为最终报告。没有通过过滤源码、修改分母或隐藏失败制造通过，分母变化不构成覆盖率改善、提速或内存改善证据。

Web 条件代码、GPU 实际绘制和设备路径不在 native 分母；stable 插桩未统计 doctest，doctest 由常规测试单独验证，没有启用分支统计。早期 `coverage-commands.json` 最后一条 `cargo llvm-cov report --all-features --json ...` 因 report 子命令不支持该选项而退出1；去掉导出子命令的 `--all-features` 后成功导出对应插桩结果，原始失败与 `coverage-export-repair-0.log` 的成功记录同时保留。最终两组合 feature 参数仅传给插桩测试，报告导出分别消费其独立 profile；正式结果见 `coverage-clean-commands.json`、`coverage-clean-summary.json` 和两份 `coverage-clean-*.json`。

### 受影响基准

本轮独占运行 `cargo bench -p pvz-core --bench stepping --locked` 与 `cargo bench -p pvz-app --bench presentation --locked`，两条退出 0；本项目窗口、浏览器、服务、测试与其他构建已停止后计时，未控制维护者其他活动及 CPU 电源/温度。七个实际运行场景均为 3 秒预热、100 样本、约 5 秒采样，下表提取 Criterion `mean` 的点估计及 95% 置信区间：

| 场景与完整计时工作量 | 均值 | 95% 置信区间 |
| --- | --- | --- |
| 规则1.0 空棋盘，1000 次 step/实际1000 tick，每 tick 0 命令，seed7 | 274.0954 µs | 272.0996–276.3963 µs |
| 规则1.0 阳光命令，1000 次 step/实际1000 tick，每 tick 2 命令，seed7 | 93.1387 µs | 91.0806–95.9232 µs |
| 规则2.0 五排，预热45植物（5向日葵/40射手）、波次总量100，1000 次 step/实际1000 tick，seed7 | 2.9891 ms | 2.9789–3.0000 ms |
| schema1/规则2.0 开发白天 JSON，1241 字节，解析/校验/Runtime 装配，无文件 I/O | 2.3455 µs | 2.3326–2.3609 µs |
| 800×600 逻辑视口，3840×2160 输入1点反变换 | 4.7915 ns | 4.7787–4.8052 ns |
| 原创8轨道128帧、12fps 开发动画时基索引 | 7.4913 ns | 7.4776–7.5058 ns |
| 双轴角度合同，1个100×90四边形与800×600裁剪 | 375.7892 ns | 371.2178–381.6209 ns |

五排初始化/种植预热在计时外，预热后 tick45、余额5750、初始无僵尸；100是波次总量，非同时存活数量；计时后 tick1045 且仍 Running，不把终局冻结当完整更新。共享 Criterion 缓存比较把空棋盘判为回退、动画索引判为改善；这些诊断保留在日志，但未完成同条件前后对照及归因，不能据此声称已确认固定性能回退、优化收益或开发耗时改善。本轮没有内存/分配测量或内存改善主张。复制的 `criterion/` 包含旧缓存场景，本表只引用 `bench-0.log`、`bench-1.log` 本轮实际运行的七项；其他条目不算本轮重测。

### 退出码缺陷修复与交付状态

附加失败边界实跑 `target/debug/pvz-app.exe --failure-smoke --smoke 1` 被 Clap 参数范围拒绝、退出 2，只证明参数边界；合法的 `--smoke 2` 在预算内不能达到失败终局，日志显示“资源状态=失败”并产生内部错误 AppExit，OS 进程却退出 0。原因是 `main` 丢弃 `app.run()` 的返回值；`failure-bounded.json/log` 保留真实的预期非零与实际零。这说明成功冒烟退出 0 不能替代错误退出传播验收。

修复直接让 `main` 返回 Bevy `app.run()` 的 `AppExit`，复用其标准 `Termination` 实现向 OS 传播错误；新增两条真实 Runtime/冒烟消费者期限边界测试。限定独立静态复核通过、未发现新增阻断，未更改 core/runtime、内容、规则版本或热路径。修复后真实进程 `--failure-smoke --smoke 400 --screenshot` 观察到 tick4049 `Defeat`、退出0；合法期限2负例退出1；`--slice-smoke --smoke 600 --screenshot` 观察到 tick5767 `Victory`、退出0。正例、负例与旧错误退出0分别保留，当前证据不再只依赖内部 AppExit。

Windows CI 已在现有 probe30/slice30 之后配置失败400成功冒烟和期限2 OS 退出码回归：前者须观察到 `Lost` 且退出0，后者必须非零退出。actionlint 通过，本机上述实际进程正/负例通过；目的提交 `f1850b1` 对应 Windows 实现检查及其余七个 CI job 均已成功，具体运行见下文交付记录。修复后 WASM release 构建与 wasm-bindgen 生成也退出0（`exit-fix-15.log/16.log`）。新包随后在后台浏览器通过启动按钮加载默认登记画面，正常 F 输入进入失败，画面显示阳光150、2/2波次及僵尸突破防线；正常 R 重开恢复阳光150、1/2波次及守住草坪。warn/error 查询为 `[]`，截图 `web-exit-fix-failure.png` 已核对；自有 tab3 和服务 PID23732 已关闭。该新包未完整重跑胜利及再次种植，main 退出传播修复没有改变玩法、Runtime 或时钟，完整双终局/冻结/再种植合同沿用上表此前同轮正常输入证据，不能把它改写成新包全流程重测。观察和清理记录为 `web-exit-fix-observation.json`。

S1 全部适用本地 L3 验收及最终独立复核完成：双终局、冻结、重开/再种植证据、两组合各194项测试、七基准、实际进程0/1/0、actionlint、修复后两组合干净覆盖率和新 WASM 回归已齐。完整目的提交 `f1850b1ce5bad82b91338867c3f4cca8387410fc` 已推送 `main`；对应 [CI 运行 38502468453](https://github.com/AomaYple/plants-vs-zombies-rs/actions/runs/38502468453) 的 sha 与该提交一致，status 为 completed、conclusion 为 success，仓库结构、依赖审计、Web 构建、Windows/macOS/Linux 实现检查及 default/all 覆盖率与基准入口共八个 job 全部成功，账本见本轮原始索引。S1 已完成开发闭环；当时进入 S2-A 实施准备，后续向日葵增量独立记录在下节，不把 S1 的成功当作 S2-A 提交验收。原版行为测量仍为独立证据轨，不因 S1 工程进展变为已还原。

### 本轮原始索引

本轮新增证据在忽略目录 `artifacts/validation/s1-2026-10-14/`，与内容迁移及仓库质量复核历史分开；命令/日志、截图和设备观察分别解释，不进入 Git、CI 或发行包。

| 证据 | 文件/目录 |
| --- | --- |
| Windows 正常输入与终局/再种植图像 | `native-interaction*.out.log/err.log`、`native-lost.png`、`native-replanted.png`、`native-won.png`、`native-won-restart-replant.png` |
| 修复后质量门槛、原生进程及 WASM 构建生成 | `exit-fix-commands.json`、`exit-fix-00.log` 至 `exit-fix-16.log`；修复前 `remaining-gates.json` 与对应日志保留 |
| 修复后正式干净覆盖率 | `coverage-clean-commands.json`、`coverage-clean-0.log` 至 `coverage-clean-7.log`、`coverage-clean-default.json`、`coverage-clean-all.json`、`coverage-clean-summary.json`、`exit-fix-source-state.json` |
| 覆盖率历史/探索与失败导出修复 | 修复前 `coverage-commands.json` 与对应报告、分母不一致的 `coverage-default-final.json`/`coverage-all-final.json` 探索报告、`coverage-export-repair-0.log/1.log` |
| 修复后新 WASM 回归 | `web-exit-fix-observation.json`、`web-exit-fix-failure.png` |
| 基准命令与原始估计 | `bench-commands.json`、`bench-0.log`、`bench-1.log`、`criterion/*/new/estimates.json` 与 `benchmark.json`，只按本轮日志筛选 |
| 失败边界探索 | `failure-bounded.json`、`failure-bounded.log` |
| S1 目的提交对应 CI | `ci.json`；sha=`f1850b1ce5bad82b91338867c3f4cca8387410fc`，run=`38502468453`，completed/success，八个 job 全部成功 |
| 结构检查快照 | `structure.json`；最终文档及交付检查由主代理继续核验 |

## 2026-10-14 向日葵动态待机增量

S2-A 在 `35bfd8cd8e4485a810bb634a64e558ea92e12505` 起点完成向日葵动态 idle、原创回归与本地可玩验收；最终独立复核、文档检查及目的提交的 Git/对应 CI 已收口。只修改 app 资源、构图、入口、测试与基准，core/runtime、内容、规则 2.0 和锁文件没有变化。`SunFlower.reanim` 的 12fps、29 轨道、`anim_idle` 半开区段 `4..29` 是资源事实；采用全局表现时钟和离散采样是开发合同，不代表原版播放器、独立实体相位、插值或混合已验证。原版 `tutorial-plant-01` 合格重复仍为 0/3。

### 实现和正确性保护

资源装配一次解析向日葵完整 clip、验证全部图像映射并解码，把唯一 `SunflowerIdle` 缓存移动到 `Scene`；原向日葵静态 pose 已移除，其余五类继续消费静态姿态。更新借用采样帧和已有纹理，保留源层顺序、alpha、仿射、裁剪与网格/材质池；main 消费 `Runtime.animation_seconds`。原创夹具验证短轨道缺帧、隐藏/空图、纹理槽位换绑、部分/完全裁剪、固定区段 4/5/28/4、异常时间、移除及 45 株共享，暂停、双终局与重开测试直接读取实际 Scene 网格。

同 seed/规则/命令回放对照逐批保存完整核心摘要、事件及表现时间，包含恢复当批、两终局、大 elapsed、重开与再植；额外 Scene 采样即时断言 core 摘要不变。成功/失败夹具分别断言初始余额 0/100、植物数 2/1、种植数和支出 150/50，重开再植后具体顶点由 `[-313,104,0]` 变为 `[-311,104,0]`。这些测试不使用截图像素判断 core 回放。

既有 native 冒烟截图等待现在冻结 Runtime、Scene、脚本和输入推进，Bevy 渲染与 `Time<Real>` 继续泵动；恰好 30 秒仅一次 `AppExit::Error`，保持超时与错误退出传播。固定预算 4/5 的实际 GPU 图分别记录 tick60/75、表现时间 1.0/1.25 秒、render_frames4/5；1000×750 图像中向日葵区域变化 5286 像素，边界框 `[116,254,199,343]`，逐张核对主体完整。CPU 构图、GPU 实跑及原版行为各自登记。

### 实际平台流程

Windows 11 原生窗口内容为 1000×750、DPI 1.25，截图含边框为 1002×782；Web 使用本批 release 包、wasm-bindgen 0.2.129、隐藏的 Codex In-app Browser，1280×720 画面、800×600 逻辑区域缩放 1.2。均以正常 F/R/P 与卡片/草坪点击完成，不注入规则或隐藏游戏状态。

| 流程 | Windows | Web |
| --- | --- | --- |
| 动态、暂停和恢复 | 安全 row1/column0 向日葵，阳光150→100；P 暂停图对主体逐像素一致，恢复及重开再植有可见姿态变化 | F 后同一位置种植；暂停图对逐像素一致，恢复有可见变化；失败/胜利重开再植后间隔400ms主体区域各变化8058像素 |
| 失败冻结 | row0 无防守，第二只僵尸突破，向日葵在安全行存活；终局前延后种植以避免阳光遮挡，无遮挡主体图对差异0像素 | 失败前正常再植安全行向日葵，明确 Lost，无遮挡主体图对差异0像素；R 后重新选卡种植恢复动态 |
| 胜利冻结 | F 同一关卡 row0 射手、row1 向日葵，总费用150、余额0；明确 Won，无遮挡主体图对差异0像素；R 清空、再植动态另有本轮正常输入证据 | R 后 row0 射手，正常防守11秒后种 row1 向日葵，连续运行到 Won；光标移至画布外后，无遮挡主体图对差异0像素；R 后再植动态已验证 |

Windows 最终补证 PID4012 运行忽略目录 `native-ui/pvz-app.exe`，SHA-256 `04711041ee79d7532b4fc97d6c23923ea76ae090212fe214225954bc343ca4f5` 与最终 `target/debug` 一致。psutil 7.1.0 只暂停/恢复该进程在模型决策间隔的 OS 调度，所有输入仍经正常键鼠；准备完成后连续运行到终局，冻结图对之间没有 OS 暂停。失败截对捕获始于 UTC07:45:10.645 与07:46:32.383，胜利始于07:57:05.011 与07:58:27.895；每次结束时间也保留，不用文件写入时间冒充捕获时间。此前 PID27572 的正常暂停/双终局/重开/再植证据保留，但其输入准备采用 OS 控制、只有部分写文件时刻，作为补充而非精确时间轴。

Web 无遮挡 Lost 截对始于 UTC07:54:45.442 与07:55:50.220；最终无光标 Won 截对始于07:59:27.587 与08:00:21.945。首次无遮挡胜利图对有251个像素差异，框局限光标位置，保留为探索；之后把光标移到画布外重新捕获，主体差异0。旧阳光遮挡终局图只证明状态与存活，不作为完整主体冻结主要证据。当前浏览器 warn/error 查询为 `[]`，Windows stderr 为空；不据此宣称历史 GPU shader 诊断已根治。本次自有 native 进程、Web 标签5与回环服务PID18912均已关闭。

当前 Windows 默认成功600/失败400隐藏冒烟均退出0；短失败预算4实际退出1，正确拒绝未达终局，不把探索的预期0写成通过。没有重新实听设备音频、默认 Web 十波完整胜利、真实 DPR2/4、触摸或其他浏览器；macOS/Linux 实际窗口及 Android/iOS 仍待对应阶段。WASM default/probe 构建和绑定生成通过，构建不能替代其他平台运行。

### 本地检查与覆盖率

最终源码 `cargo fmt --all -- --check`；默认/all-features 分别执行 workspace all-targets check、workspace test（保留 doctest）、all-targets Clippy `-D warnings` 和 doc；`cargo deny check`、actionlint、native build 共12条命令全部退出0。两组合各210项测试；Cargo jobs=2、测试线程默认并行。deny 四类检查通过，重复传递依赖 WARN 保留。锁定 Rust1.99.0、Edition2024、Bevy0.20；没有新增依赖或 feature。

覆盖率分别 `cargo llvm-cov clean --workspace --locked` → 对应 feature 的 `cargo llvm-cov --workspace --locked --no-report` → `cargo llvm-cov report --json --output-path ...`，feature 参数只传测试。两报告均为4546/5189行（87.6084%）、463/539函数（85.8998%）、7130/8045区域（88.6265%），28个文件全部位于当前 checkout，没有追加窗口 profile。部分 cfg(test) 源码计入分母，新增测试使分母变化，不能当生产覆盖提升证据；未启用 branch/MCDC，count0不表示100%。doctest由常规测试单独验证。

| 受影响源码 | 命中行/总行 | 范围与遗漏 |
| --- | --- | --- |
| app/animation.rs | 81/81 | 离散采样和准备全部行命中；不能称分支/原版语义100% |
| app/resources.rs | 491/521 | idle 构造/采样、映射与登记资源失败有覆盖；平台装配/未触发读取错误仍按报告保留 |
| app/render.rs | 902/1093 | 真实 Scene CPU 构图、池、可见性、纹理和时钟已命中；相机、标签、Scene GPU上传等没有纯测试profile，由独立窗口/GPU证据验证 |
| app/main.rs | 695/979 | update 时钟接线、截图等待、期限/退出消费者已命中；真实初始化/设备/音频脚本及窗口输入不在纯测试profile，Web条件代码不在native分母 |

资源/Scene 红测按真实缺失接口/静态消费者失败登记。截图等待首次测试失败来自夹具前置条件，不是 guard 缺失红测，明确保留该流程偏差；最终恰好30秒实际 update 回归已通过。一次局部测试因运行中的 exe 被 Windows 锁住而无法替换，停止自有进程后重跑 bin33+presentation11通过，不把构建锁失败归为行为失败。其他编译/夹具失败与修复记录保留在命令索引，未删有效测试或降低门槛。

### 基准和缓存策略

独占运行 `cargo bench -p pvz-app --bench presentation --locked` 退出0、391.566秒。每场景3秒预热、100样本、约5秒采样；Windows11、i7-11800H、8核16线程，未控制维护者其他活动或CPU电源/温度。原创29轨道夹具含真正6帧短轨道，准备与合同断言在计时外；计时完整遍历采样输出。以下采用 Criterion JSON `mean` 和95%区间：

| 场景 | 均值 | 95%置信区间 |
| --- | --- | --- |
| JSON解析校验/Runtime装配，1241字节 | 2.30059 µs | 2.29552–2.30589 µs |
| 3840×2160输入1点反变换 | 4.75576 ns | 4.74812–4.76386 ns |
| 原创29轨道/25帧/12fps所选idle借用采样 | 47.11351 ns | 47.00620–47.22696 ns |
| 原创8轨道/128帧/12fps索引 | 7.51104 ns | 7.48016–7.55239 ns |
| 双轴仿射裁剪1四边形 | 370.81153 ns | 369.02186–373.10559 ns |
| 1株采样与仿射裁剪 | 5.20120 µs | 5.18622–5.21768 µs |
| 45株采样与仿射裁剪 | 237.56819 µs | 236.73711–238.52310 µs |

构图计时包含几何输出分配，不含 Bevy 网格池/GPU 提交，core/assets 生产逻辑未变，本批不重复其无关基准。真实45株Scene另验证只有一份缓存、纹理/网格/材质更新不累积。登记缓存结构为841展开帧、图像字符串24862字节、名称519字节、font/text为0；外层轨道/names与每轨道Vec为29/29，可绘索引28/32。仅统计结构，不包含String额外容量、分配器元数据、峰值、分配次数或GPU显存。没有同条件旧基线，不宣称速度、内存或开发耗时改善。

### S2-A 原始索引与交付边界

原始证据位于忽略目录 `artifacts/validation/s2-a-2026-10-14/`，不进入 Git/CI/发行包；公开测试和基准夹具原创，原版资源保持原清单指纹。

| 证据 | 本批文件 |
| --- | --- |
| 环境、最终源码/程序包身份 | `environment.json`、`final-source-state.json`、`final-package-state.json` |
| 最终本地命令和原始输出 | `final-l3-commands.json`、`final-l3-*.log`、`final-local-tests-fixed.json/log` |
| 干净覆盖率和路径核对 | `final-cov-coverage-commands.json`、`final-cov-coverage-default.json`、`final-cov-coverage-all.json`、`final-coverage-summary.json`、`coverage-zero-regions.json` |
| 独占基准及结构统计 | `task4-bench.json/log`、`bench-summary.json`、本轮Criterion估计、`cache-structure.json` |
| GPU固定相位及成功/失败冒烟 | `gpu-slice-1s-final.json/log`、`gpu-slice-125s-final.json/log`、`gpu-1s.png`、`gpu-125s.png`、`gpu-image-comparison.json`、`windows-success-smoke-600.json/log`、`windows-failure-smoke-400.json/log` |
| Windows最终无遮挡双终局 | `windows-unoccluded-lost-a/b.jpg`、`windows-unoccluded-won-a/b.jpg`、`windows-unoccluded-captures.jsonl`、`windows-process-control.jsonl`；旧暂停/重开/再植图单列补充 |
| Web暂停/终局/重开再植 | `web-paused-a/b.png`、`web-unoccluded-lost-a/b.png`、`web-won-nocursor-a/b.png`、`web-replant-dynamic-a/b.png`、`web-won-replant-dynamic-a/b.png`、`web-captures.jsonl`、`web-final-logs.json` |
| 图像比较、操作与清理 | `final-pixel-comparison.json`、`platform-observation-index.json` |
| 独立复核与执行账本 | `.superpowers/sdd/2026-10-14-sunflower-idle/` 内任务报告、限定复核和progress；Git/对应CI收口后另追加交付记录 |

最终限定独立复核在本地交付范围内规格与质量通过，无新的实质阻塞；核查源码/日志与图像，没有重跑 Cargo/UI。原 Windows 遮挡、操作索引及 Web/覆盖率/文档缺口均已关闭。主线程核对 73 个源码/配置和 5 个程序包指纹一致，L0 检查 3353 个文件、3229 条原版资源、328 条本地文档链接和 5 个 workspace 成员，均通过；目的提交与 CI 交付详情如下，不借用 S1 或旧计划提交的 CI。

S2-A 目的提交 `3e6634e5103de15b3f8f8ffe5d6c363895e47a3b` 已直接推送 `main`；[CI 运行 38536185894](https://github.com/AomaYple/plants-vs-zombies-rs/actions/runs/38536185894) 的 head_sha 完全一致，completed/success，结构、依赖审计、Windows/macOS/Linux 实现检查、Web 构建及两组覆盖率/基准入口八个 job 均成功。原始记录为本批 `ci.json`，独立可玩阶段完成；本次随后状态/后续计划文档提交的 CI 另行核对。

本节工程与可玩运行不表示整个 S2、其他平台或原版一致性完成；S2-B 源码仍未实施，只有当前子阶段完整交付后才继续。

## 2026-10-13 仓库质量复核补充

本次复核针对提交 `3bdf7b471baa3c399fdb8c32995a037745827921` 在 Windows 工作区重新执行了格式、workspace 检查、默认/all-features 测试、Clippy、文档、依赖审计、资源审计、Reanim 检查、编码与链接检查，以及无资源和登记资源后台冒烟；这些检查均以退出码 0 完成。对应远端 CI 运行 [38330592532](https://github.com/AomaYple/plants-vs-zombies-rs/actions/runs/38330592532) 的已有 8 个任务也全部成功。

复核发现异步渲染管线在 30 帧时可能尚未完成，短时 GPU 截图会只有背景和文字而缺少探针构图；同入口 120 帧探索复验实际包含网格、仿射夹具和裁剪结果。修复采用 Bevy 的同步管线编译能力，仅在冒烟模式启用；探针截图还检查右下角原创红蓝夹具，缺少任一颜色时不保存成功截图，并经现有截图等待消费者返回非零退出。正常游戏保留异步管线编译。CI 使用 `--smoke 30 --screenshot`，复用现有 Windows 实现检查的编译产物并限定每步 10 分钟，不再把固定帧数或截图文件存在单独当作完整画面通过。

修复增量的默认/all-features workspace 各通过 187 项测试（含 doctest），fmt、all-targets 检查、Clippy、文档、依赖审计与两种后台截图冒烟均退出 0。两项新增截图回归先取得真实红测：旧保存入口接受缺少夹具的画面；修复后缺少任一颜色时拒绝保存，完整红蓝夹具与普通截图通过。两种 30 帧 GPU 截图均为 1000×750，已经逐张查看，探针网格/夹具与登记资源切片可见；切片日志包含种植、暂停、恢复、阳光生成及收集。

本轮 Windows native 纯测试的默认/all-features 行覆盖率均为 3229/4040（79.93%），app 为 1409/2098（67.16%），core 保持 822/842（97.62%）。没有追加窗口 profile，Web 条件路径、GPU 实际绘制及分支覆盖率仍不属于这些百分比。首次默认报告混入旧 worktree 路径，保留为 `coverage-cache-exploration/`；通过 `cargo llvm-cov clean --workspace --locked` 清理后重测，确认两个正式报告只包含当前 checkout 的源码。编译并发限定为 2，测试保持默认并行；中断的高并发探索和被污染的报告不算正式验收。

本次独占 Criterion 复测只记录当前环境读数，不把它写成优化收益：空棋盘场景相对本机旧 Criterion 基线出现约 3.47%–7.51% 的统计回退，阳光命令场景无显著变化，白天切片场景约 0.53%–2.57% 且被 Criterion 判为噪声范围。该结果尚未完成同条件基线归因，后续优化前必须复现并结合行为回放验证。复核早期仍出现 Bevy 0.20 的 `mesh2d.wesl` unresolved-import 警告，随后截图复验没有再次出现；未证明该间歇性诊断已修复，保持日志可见，后续继续核对不同 GPU 和目标平台影响。

随后对未修改的 core 再次独占复测，三个场景的均值分别为 87.645 µs、89.014 µs 和 2.993 ms，Criterion 与上一轮的比较方向反转；这进一步限制了旧缓存比较的解释范围，不能据此声称优化成功或已经确认固定性能回退。本轮没有内存改善主张。

本轮新增原始命令、日志和截图位于忽略的 `artifacts/validation/repo-quality-2026-10-13/`，与下面的内容数据化历史索引分开保存。原版行为测量、Web 失败流程和未运行平台继续按活动计划推进，不因本次工程整改改为已完成。

## 本机证据索引

原始日志、报告、截图和账本保存在被忽略的 `artifacts/validation/content-foundation-2026-10-12/`，不进入 Git、CI 或发行包；旧 `artifacts/validation/resume/` 保留历史用途，未覆盖。公开本页提供原创文字与读数，原版 EXE、截图、录像和存档遵守[资源说明](../../assets/README.md)。

| 证据 | 本批文件/目录 |
| --- | --- |
| 环境与正式汇总 | `environment.json`、`summary.json`、`receipts.json` |
| 正式默认/all门槛、原始日志 | `final-commands.json`、`final-00.log` 至 `final-10.log` |
| 干净覆盖率与探索隔离 | `coverage-clean.json/log`、`coverage-commands.json`、`coverage-default.json/lcov`、`coverage-all.json/lcov`、`coverage-shared-cache-exploration/`；阶段性 `coverage-data-core.json` 仅作探索 |
| 计时命令、原始均值/区间 | `bench-commands.json`、`bench-00.log`、`bench-01.log`、`criterion/*/new/estimates.json` 与 `benchmark.json` |
| Windows实际切片 | `native-slice.json`、`native-slice.log`、`native-slice.png` |
| WASM构建与生成 | `wasm-commands.json`、`wasm-00.log` 至 `wasm-03.log` |
| Web观察、页面与日志 | `web-observation.json`、`web-*.jpg`、`web-default-logs.json`、`web-probe-logs.json`、`web-server.json/log` |
| 原版身份与窗口资格 | `reference-audit.md`、`reference-process.json`、`reference-capture-observation.json`、`reference-*.png` |
| 内容/结构/元数据检查 | `content-audit.md`、`structure.json`、`metadata.json`；root 最终检查通过，3348 文件、3229 资源、228 本地链接、5 个成员；限定文档修复复核已通过 |
| 各任务实施、独立复核与裁决 | 当前 `.superpowers/sdd/2026-10-12-versioned-day-content/` 账本，清理工作区前由主代理保存到本批 `reviews/` |

文档中的“已通过”仅覆盖本页明确列出的本地或对应提交的 CI 证据，两者范围分别记录。整体静态复核、root 最终结构检查、限定文档修复复核和本批 Git 交付已完成；后续实现仍按路线图的独立任务和平台验收推进。
