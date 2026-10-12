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

后续按[原版行为基线测量计划](../superpowers/plans/2026-10-12-reference-behavior-measurement.md)推进：

1. 先取得可靠目标画面与输入时间记录，再重复测量可见间隔和碰撞；记录采集误差，以证据替换开发参数并同步回归。窗口资格验证不代表原版失焦推进合同已确认。
2. 逐对象接入已解析 Reanim 子集，先 idle/walk，再射击、啃食、死亡混合、粒子与 MO3；以原版画面对照确认稀疏字段、插值和特殊帧语义。
3. 依据行为证据扩展护甲、减速、特殊攻击与地形，随后主菜单、选卡、冒险进度、其他模式和跨平台存档。首批白天数据化不等于完整内容系统。
4. 补 Web 失败、真实DPR、触摸和其他浏览器，以及 Linux/macOS 实际运行，随后移动生命周期、打包和六平台发布。构建、实际运行和原版还原分别验收。

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

文档中的“已通过”仅覆盖本页明确列出的本地证据。整体静态复核、root 最终结构检查、限定文档修复复核和本批 Git 交付已完成；后续实现仍按路线图的独立任务和平台验收推进。
