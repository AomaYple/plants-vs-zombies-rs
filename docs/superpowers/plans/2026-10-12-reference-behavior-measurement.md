# 原版行为基线测量实施计划

> 当前活动计划。旧的基础工程与版本化内容计划已完成并保留为历史验收索引；本计划只处理原版可观察行为证据和开发规则替换入口。

**目标：** 在继续扩展完整内容前，对已确认身份的原版白天教程/切片建立可重复的行为测量证据，把有足够置信度的参数接入版本化规则和核心回放，同时保留开发参数与原版事实的边界。

**设计：** [原版行为基线测量设计](../specs/2026-10-12-reference-behavior-measurement.md)。**前置：** [参考版本基线](../../behavior/reference-baseline.md)、[开发切片合同](../../behavior/development-slice.md)、[项目原则](../../principles/README.md)。

## 共同约束

- 正确性第一、速度第二、内存第三；先建立可观察事实和回放保护，再做任何优化或参数替换。
- 原版程序只作参考输入，不能成为生产运行依赖；原始截图、录像、存档和临时脚本只写入被忽略的 `artifacts/` 或仓库外。
- 自动化测量、测试和运行尽可能后台执行；真实窗口交互只使用最小范围，并记录无法后台完成的原因。
- 所有公开文本 UTF-8、LF、无 BOM、末尾换行；新增工具、数据格式和目录必须有实际消费者，禁止预建空实现。
- 公开仓库不提交凭据、个人绝对路径、原版可执行文件或未登记临时载荷；MIT 只覆盖本项目原创内容。

## Task 1：建立测量记录和验收入口

**文件：** `docs/behavior/measurement-protocol.md`、`docs/behavior/measurement-schema.md`、`docs/behavior/README.md`、`docs/behavior/reference-baseline.md`、`docs/roadmap.md`。

- [ ] 将参考身份、窗口资格验证和未验证事实整理为统一的案例字段，引用新的测量设计。
- [ ] 定义 JSON/表格记录的最小字段、误差写法、重复次数、排除规则和 `observed`/`inferred`/`development_parameter` 分类。
- [ ] 为 tick、间隔、碰撞、事件顺序、失败/胜利、暂停/重开建立可逐项勾选的验收矩阵；每项写清从证据到规则的接受条件。
- [ ] 检查所有入口链接、编码、BOM、换行和文档表述，不把资格验证或单次教程观察写成行为结论。

**验证：** `git diff --check`；文档链接检查；UTF-8/LF/无 BOM 检查；`cargo metadata --no-deps --locked --format-version 1`。

## Task 2：取得可重复的原版窗口证据

**文件：** 仅在确认有稳定消费者后新增 `tools/` 测量辅助；原始记录写入被忽略的 `artifacts/validation/reference-behavior-*`，不提交截图、录像或存档。

- [ ] 重新核对目标 EXE、`main.pak` 和解包资源指纹，记录运行环境、窗口尺寸、DPI、输入设备和目标进程绑定。
- [ ] 先验证连续画面和输入回放能稳定到达同一教程/切片状态，再执行固定输入序列；后台启动优先，前台交互仅用于目标窗口接口确实需要的步骤。
- [ ] 对每个案例至少完成三次有效重复，分别记录阳光产出、豌豆发射/命中、僵尸移动/啃食、割草机、波次、胜负、暂停/恢复和重开；失败案例单独记录，不以退出码代替画面证据。
- [ ] 保存原始帧号/时间、输入序列、失败重试、捕获误差和环境元数据；清理无用临时进程和未被索引的产物。

**验证：** 每个案例都有完整字段、有效重复次数和原始证据索引；无法测量的项目写明阻塞原因和影响范围。

## Task 3：把证据映射到规则与内容

**文件：** `content/levels/development-day.json`、`crates/pvz-data/src/day.rs`、`crates/pvz-core/src/content.rs`、`docs/behavior/development-slice.md`、`docs/behavior/reference-baseline.md`、必要时新增版本化内容 JSON。

- [ ] 只对满足设计接受条件的参数建立已测量来源；其余参数继续保持 `basis: development`，不得批量标记为原版。
- [ ] 若 tick、间隔、碰撞或阶段顺序改变，先更新规则版本/内容来源字段，再补齐装配边界和无效输入测试；保持 core 不依赖 JSON、I/O、引擎或墙钟。
- [ ] 用固定输入逐 tick 比较状态摘要和事件序列，覆盖正常命令、暂停/恢复、重开、胜利和失败；新旧开发回放的兼容/拒绝策略必须明确。
- [ ] 为每个替换参数新增回归测试，并运行受影响 Criterion 基准；性能报告必须与行为结果分开，不能用优化结论掩盖规则变化。

**验证：** `cargo test -p pvz-data --locked`、`cargo test -p pvz-core --locked`、相关 `cargo llvm-cov` 和 `cargo bench`；提交前执行完整 workspace 门槛。

## Task 4：核心与窗口行为验收

**文件：** `crates/pvz-core/tests/`、`docs/behavior/reference-behavior-results.md`、`docs/behavior/README.md`、`docs/roadmap.md`、`README.md`。

- [ ] 公开结果页逐项区分原版观察、候选推断、开发参数、核心回放和平台实跑；附案例 ID、重复次数、误差和未验证项。
- [ ] 复核失败边界、暂停/失焦、重开和终局冻结；浏览器失败流程、真实 DPR/触摸、Linux/macOS 和移动平台继续按路线图独立验收。
- [ ] 完成静态引用、资源边界、死代码/无用文件、锁定元数据和 Git 差异检查；删除仅为探索而创建且无消费者的文件或目录。

**验证：** `cargo fmt --all -- --check`、`cargo check --workspace --all-targets --locked`、`cargo test --workspace --locked`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo doc --workspace --no-deps --locked`、`cargo deny check`、`git diff --check`。

## Task 5：下一阶段计划收口

- [ ] 将本计划标记为已完成或保留明确未完成项，更新路线图 P3/P4 的勾选和依赖顺序。
- [ ] 若行为测量达到接受条件，创建独立的“内容与表现扩展”设计/计划，拆分 Reanim 动画、粒子、音频和更多实体；不把六平台发布合并进单一大任务。
- [ ] 保留本计划、测量结果和原始证据索引的历史职责；不删除仍有引用或审计职责的旧计划。

**交付门槛：** 一个完整目的一个提交，使用 `docs(behavior): 中文说明` 或 `docs: 中文说明`；按贡献指南完成适用验证、评审、CI 和主线同步。外部推送、合入和分支清理按当次授权执行，并分别报告本地验证与远端状态。
