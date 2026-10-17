# 版本化白天内容实施计划

> 状态：已完成，作为历史验收索引保留。阶段性开发计划见[七阶段集中开发总计划](2026-10-17-phased-development.md)，原版证据计划见[行为测量专题计划](2026-10-12-reference-behavior-measurement.md)。

> 本文保留当批实现与复核顺序，所有勾选仅对应已记录的内容迁移验收；后续实施范围与授权以当次任务及现行计划为准。

**目标：** 以严格版本化原创内容驱动现有白天切片，消除应用初始化和频率硬编码，同时保持默认规则 2.0 的逐 tick 行为。
**架构：** data 提供 serde DTO，core 只消费纯类型并校验装配，app 解析编译嵌入的 JSON；原版资源装配与探针继续沿用现有入口。
**技术栈：** Rust 1.99.0、Edition 2024、Bevy =0.20.0、现有 serde/serde_json、Criterion 0.5。
**设计：** [版本化白天内容设计](../specs/2026-10-12-versioned-day-content.md)。

## 共同约束

- 遵守[项目原则](../../principles/README.md)及[贡献指南](../../../CONTRIBUTING.md)全部适用验收；文本 UTF-8/LF/无 BOM/末尾换行，原创说明、注释、诊断使用中文。
- 正确性第一、速度第二、内存第三。测试默认并行；Cargo/实际窗口/性能测量由主代理统一协调，基准与其他项目活动隔离。
- core 无引擎、JSON 解析、I/O、墙钟或资源路径生产依赖；不改变命令/事件顺序、RNG、几何、碰撞和默认参数。
- 六平台依赖边界不变；Windows 与 Web 本批实际验收，其他环境继续分别保留限制。
- 独立目的提交使用 `type(scope): 中文说明`，实现带测试和必要文档。短期 `codex/` 分支，经复核/CI 后 PR 合入，保留完整目的的独立提交。
- 旧任务 1–8 不重做；旧记录保持历史用途。当前规则参数均为 development，无原版测量不能标记原版已还原。

## 仓库审计与依赖裁决

基线 main 为 `545a2b2b6338850b1ed5c35e21a7a62b70715b70`，五个 package、156 项测试（含 doctest）及原有 CI 构成历史基线；本批仍须重跑适用检查。发现架构、crates 总入口、docs 总入口有过时暂停或待复核文字，活动计划入口仍指向旧基础计划。实施前同步活动入口与路线图，旧计划保留已完成任务和验收索引职责。

优先复核原版 EXE/main.pak 身份与旧测量失败原因，原始路径不进入生产依赖。历史窗口捕获取得了无关前台画面，不能据此猜测原版时基/碰撞。本轮工具发现后经 Computer Use 取得有效标题、菜单和 1-1 教程画面，正常选卡种植观察到余额 150→50；窗口观察入口已资格验证，规则时序和碰撞仍待重复测量。本批内容装配不依赖这些未知数值，继续保持 development 标记。

### 任务 1：严格内容 schema 与原创默认关卡

**文件：** 新增 `crates/pvz-data/src/day.rs`、`crates/pvz-data/tests/day_content.rs`、`content/levels/development-day.json`、`content/README.md`；修改 `crates/pvz-data/src/lib.rs`、对应 README。
**接口：** 导出设计中的四个 DTO，所有字段公开，derive Debug/Clone/Eq/PartialEq/Serialize/Deserialize；basis 使用 snake_case 枚举，只接受 development。rules 类型与当前 SliceRules 完全一致，不引入规则执行或重复验证算法。

- [x] 先写严格格式测试并确认新增类型缺失时失败。测试使用 serde_json，从真实原创内容加载，覆盖稳定往返、23 个标量及十项有序波次，所有层级未知/遗漏/重复字段、整数溢出/负 unsigned/小数、schema/basis/ID 错误。
- [x] 定义纯 DTO，加入原创 JSON，数值逐项取自基线 board.rs；新 JSON 顶层严格使用设计规定字段。
- [x] 测试示例必须断言实际内容：`assert_eq!(definition.level.as_str(), "development-day-slice"); assert_eq!(definition.rules.wave[0].at, 600); assert_eq!(definition.rules.wave[1].row, 0);`，并使用序列化后再次解码核对全量相等。格式版本语法验证不代替 core 支持版本判定。
- [x] 运行 `cargo test -p pvz-data --locked`、相关 Clippy/格式；主代理完成 workspace 适用验收与独立复核后按完整目的提交 `feat(data): 定义版本化白天内容与默认关卡`。

### 任务 2：纯模拟装配与逐 tick 迁移回归

**文件：** 新增 `crates/pvz-core/src/content.rs`、`crates/pvz-core/tests/day_content.rs`；修改 `lib.rs`、`world.rs`、Cargo.toml（仅 serde_json dev-dependency）及 core README。
**消费：** 任务 1 DTO。**产出：** `GameState::from_day_definition(&DayLevelDefinition) -> Result<Self, InvalidDayLevel>`，中文错误实现 std::error::Error；严格规则2.0、频率1..=1000、既有 SliceRules::validate。

- [x] 先建立构造/支持版本/频率/无效规则回归，运行 `cargo test -p pvz-core --test day_content --locked` 确认新增接口缺失导致失败。
- [x] 用显式字段转换保持 23 标量和 wave 顺序；仅既有 validate 执行规则有效性判断。以给定时钟构造当前和初始状态，不引入 JSON 生产依赖或文件读取。
- [x] 默认 JSON 构造与旧 `GameState::day(7, InitialState::new(LevelId::new("development-day-slice").unwrap(), 150), SliceRules::development()).unwrap()` 跑至少6000步，在每一步 `assert_eq!(new_events, old_events); assert_eq!(new.state_digest(), old.state_digest());`，包含选择/种植/暂停恢复/重开并明确比较阶段。
- [x] 测试每类参数/初始值/seed/频率在摘要与实际规则中的生效、重开恢复、同 tick 波次顺序、不支持规则版本和频率0/1001、既有数值/波次非法边界；保留旧确定性与基准场景测试。
- [x] 运行 core 测试/Clippy、受影响覆盖率及原有切片 Criterion 场景；主代理统一完整验收与独立复核，按完整目的提交 `feat(core): 装配版本化关卡并保护切片回放`。

### 任务 3：应用内容入口与实际时钟消费

**文件：** 新增 `crates/pvz-app/src/content.rs`、`crates/pvz-app/tests/day_content.rs`；修改 `lib.rs`、`runtime.rs`、Cargo.toml（serde_json 生产依赖）、`benches/presentation.rs`及 app README/开发合同。
**消费：** 任务 2 构造。**产出：** `Runtime::from_day_json(&str) -> Result<Self, DayContentError>`、`Runtime::day()`嵌入默认JSON，advance读取实际clock。

- [x] 先建立默认入口、损坏/未知/超大输入无回退、30/120Hz推进与重开回归，运行 app 聚焦测试确认缺失接口失败。
- [x] 复用 serde_json 并在解析前拒绝超过1048576字节；包装中文错误，normal/probe 入口保持明确，不新增热重载、文件配置器或选关UI。
- [x] advance 以实际 clock 计算 tick，保持既有250ms上限、暂停/终局无追赶、有序命令和重开动画清零。测试0.25秒分别推进30Hz的7tick/120Hz的30tick，暂停冻结、恢复不补停顿、重开时钟保持。
- [x] 加入默认原创JSON解析+校验+Runtime装配 Criterion 场景（字节数/版本明确，计时范围包含解析与装配，不包括文件I/O）；运行相关测试、覆盖率、基准及默认/all-features workspace门槛。
- [x] Windows隐藏窗口和Web默认入口实跑，核对seed/初始阳光/波次及玩法仍正确；默认/probe WASM构建与最小探针分别记录。不能仅凭构建声称运行。
- [x] 独立复核、完整验收及文档同步后按目的提交 `feat(app): 使用版本化内容驱动白天切片`。

### 任务 4：证据收口与后续路线整合

**文件：** 新增 `docs/behavior/content-foundation-results.md`，更新 README、docs/README、架构、路线图、当前计划和必要专题链接。
**消费：** Tasks 1–3实现、复核和真实日志。**产出：** 可核查验收、活动计划状态与下一批依赖。

- [x] 汇总实际命令/版本/测试数/覆盖率分母与遗漏/基准均值及95%CI/Windows与Web运行结果；保留原始证据在本批忽略目录，不覆盖旧resume记录。
- [x] 整体独立复核，修复有效问题，核对无死代码/未使用依赖/无消费者文件以及文本、链接、清单哈希、locked metadata；唯一文档 Minor 已修复并通过限定复核。
- [x] P4只勾选现有白天参数和波次的数据化；更多植物/僵尸/地形、完整动画/MO3、模式、存档与平台发布继续列待办，原版测量仍列优先门槛。
本批实现、任务复核与本地实测见[版本化内容结果](../../behavior/content-foundation-results.md)。整体独立静态复核、限定修复复核和 root 的最终文本/链接/清单哈希/locked metadata 检查均已通过，未发现生产代码缺陷。PR #3 已以 rebase 合入 `main`，合入提交为 `36f6e10280415fd6005bbd142ff80b8d42ba9f3c`，PR 与主线 push CI 各 8 个 job 均已成功；本地分支和实施 worktree 已清理。

### Git 交付门槛

依据授权提交/推送、创建 PR、等待必要 CI，保留完整目的提交合入 main；核对主线 CI、同步本地 main，保存本机账本并清理完成分支与工作区。每次交付均须执行这些门槛，状态以对应 PR、CI 和实际清理记录核验，不以一次性勾选表示流程永久完成，也不得把已排队操作当作完成。

## 后续入口

内容数据化只提供参数替换和内容扩展入口，不能替代原版证据。后续开发与平台任务统一由[总计划](2026-10-17-phased-development.md)维护；原版受控测量和逐参数采纳由[专题计划](2026-10-12-reference-behavior-measurement.md)维护。Web 双终局、向日葵 idle 和本地普通僵尸 walk 的后续范围见[增量结果](../../behavior/content-foundation-results.md)，不改写为本批重测，也不再保留第二份活动待办。
