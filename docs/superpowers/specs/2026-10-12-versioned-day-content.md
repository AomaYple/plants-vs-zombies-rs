# 版本化白天内容设计

日期：2026-10-12。本文在[总体设计](2026-10-07-repository-and-runtime-design.md)和[开发切片合同](../../behavior/development-slice.md)上增加内容装配边界，不改变原版还原目标。

## 问题与交付范围

首个开发切片已经完成工程验收，默认规则、seed、初始阳光和关卡标识仍写在 Rust 构造函数内，应用还重复使用固定 60 Hz。下一步将现有白天参数和有序波次放入版本化原创 JSON，使内容修改有明确格式、错误诊断、确定性回归和实际应用消费者。本批只覆盖现有两种植物、普通僵尸、豌豆、阳光与割草机的参数；护甲、地形、更多实体、完整动画和模式另按路线图推进。

## 数据与装配

- 新增 `content/levels/development-day.json`，由本项目原创，MIT 覆盖；不得混入原版资源清单。它是默认应用的内容输入，编译时嵌入，因此 native/Web 使用同一字节输入，不另建文件加载器或部署复制流程。
- `pvz-data` 定义 `DayLevelDefinition`、`DayRulesDefinition`、`DaySpawnDefinition` 和 `ContentBasis`，只依赖现有 serde。字段使用明确类型，所有结构使用 `deny_unknown_fields`，不使用会掩盖遗漏的默认值、别名或展平字段。
- 顶层字段为 `schema_version: SchemaVersion`、`rules_version: RulesVersion`、`basis: ContentBasis`、`level: LevelId`、`seed: u64`、`initial_sun: u32`、`ticks_per_second: u32`、`rules: DayRulesDefinition`。当前 basis 仅接受 `development`，不提供无证据的原版已测量标记。
- rules 逐项对应规则 2.0 的 23 个标量和 wave；整数类型与现有 `SliceRules` 一致。wave 项为 `at: u64`、`row: u8`、`x: i32`，数组顺序不能排序或去重。同 tick 的生成顺序属于合同。
- JSON 默认 seed=7、level=`development-day-slice`、initial_sun=150、ticks_per_second=60，rules 与基线 `SliceRules::development()` 完全一致。保留这个有测试和基准消费者的现有工厂，作为独立迁移对照；不在生产路径静默回退到它。
- `pvz-core` 新增 `GameState::from_day_definition(&DayLevelDefinition) -> Result<GameState, InvalidDayLevel>`。纯类型装配严格只接收规则 2.0，频率范围为 1..=1000（schema 1 的有限工作量合同），逐项转换后调用既有规则校验，保持唯一规则有效性判断。core 不增加 serde_json 的生产依赖，不读文件或资源。
- 显式时钟必须存入当前与初始合同；重开恢复同一内容、seed、初始状态与时钟。摘要继续包含实际影响演化的完整值，无需新增散列、序列化状态或修改规则版本。
- `pvz-app` 新增 `Runtime::from_day_json(&str) -> Result<Runtime, DayContentError>`，复用成熟 serde_json，先拒绝超过 1 MiB 的输入，再解析和调用纯装配。自定义诊断为中文，保留第三方错误作为原因；损坏、未知字段/版本或非法参数明确失败。
- `Runtime::day()` 消费编译嵌入 JSON；项目自带内容无效时明确失败，探针仍使用独立无资源空棋盘入口。`advance` 从实际模拟时钟取得频率，保留 250ms 帧上限、暂停/终局不追赶和有序输入合同。不新增 CLI 选关、热重载、存档、配置菜单或暂时无消费者的 API。

## 验收与证据

默认 JSON 与旧工厂逐字段相等，并以同一 seed/初始状态和每步相同命令运行至少 6000 次，逐步比较完整摘要及事件序列；包括正常游戏命令、暂停/恢复和重开。独立覆盖更改参数确实改变模拟、同 tick 波次顺序、非法范围、未知 schema/rules/basis/字段、遗漏/重复字段、损坏 JSON、超大输入，以及 30/120 Hz 推进和重开恢复。不得仅比较最终胜利标记。

schema、解析/装配和核心推进使用现有测试/基准机制；新增原创 JSON 解析装配 Criterion 场景。完整默认/all-features 门槛、覆盖率、原有核心场景与新增场景计时，以及默认/probe Web 构建按贡献指南完成。实际 Windows 隐藏窗口与 Web 默认入口确认仍消费资源并运行切片；平台证据和原版行为测量分别记录。仅在有相同条件比较时讨论性能改善，本轮不预设优化收益。

## 计划与历史

旧基础计划和本批计划作为已完成工程范围的历史验收索引保留，当前活动入口切换到[原版行为基线测量计划](../plans/2026-10-12-reference-behavior-measurement.md)。清理文档中的过时暂停/待复核描述；原则继续按原有单一职责维护，不把本批进度写成新原则。原版数值测量仍是规则还原的前置条件，内容数据化可以独立推进。
