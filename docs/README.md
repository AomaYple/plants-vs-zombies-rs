# 文档索引

本仓库按职责维护文档，避免同一事实在多个入口漂移。

| 文档 | 用途 |
| --- | --- |
| [项目原则](principles/README.md) | 工程原则与游戏开发原则的入口和文档职责 |
| [工程原则](principles/engineering.md) | Rust、测试、覆盖率、性能、卫生、跨平台和 Git 规则 |
| [游戏开发原则](principles/game-development.md) | 行为还原、运行时、资源、平台和开发效率规则 |
| [路线图](roadmap.md) | 阶段顺序、当前阶段摘要、下一步和平台门槛 |
| [七阶段集中开发总计划](superpowers/plans/2026-10-17-phased-development.md) | 当前产品阶段、依赖、验收标准及旧计划承接 |
| [第一阶段：完整白天战斗体验](superpowers/plans/2026-10-17-phase-1-day-combat.md) | 当前集中开发任务与整批实机测试/调整 |
| [阶段开发设计](superpowers/specs/2026-10-17-phased-development-design.md) | 经用户确认的阶段组织方式与迁移边界 |
| [原版行为测量计划](superpowers/plans/2026-10-12-reference-behavior-measurement.md) | 原版证据、重复和规则采纳 |
| [测试速度与内存优化计划](superpowers/plans/2026-10-15-test-performance-and-memory-optimization.md) | 已完成的测试反馈、测量和 CI 优化专题 |
| [完整设计](superpowers/specs/2026-10-07-repository-and-runtime-design.md) | 当前架构与运行时设计基线 |
| [版本化内容设计](superpowers/specs/2026-10-12-versioned-day-content.md) | 内容 schema、纯装配和时钟边界 |
| [原版行为设计](superpowers/specs/2026-10-12-reference-behavior-measurement.md) | 行为证据分类和替换边界 |
| [测试策略](testing/README.md) | 测试类型、隔离、覆盖率和平台边界 |
| [性能策略](performance/README.md) | 基准、时间/内存口径和已测优化结果 |
| [架构](architecture/README.md) | crate、模块、依赖方向和数据流 |
| [行为测量](behavior/README.md) | 行为合同、证据分类和回放要求 |
| [参考基线](behavior/reference-baseline.md) | 版本身份、已核对事实和待测量项 |
| [行为结果](behavior/content-foundation-results.md) | 实际命令、环境、读数、提交和限制 |
| [技术决策](adr/README.md) | 架构和资源边界的 ADR |
| [贡献指南](../CONTRIBUTING.md) | 工具链、验证、提交、PR 和交付操作 |

## 历史计划

| 文档 | 保留职责 |
| --- | --- |
| [旧 S 系列总计划](superpowers/plans/2026-10-13-minimum-playable-vertical-slices.md) | S0–S9 原规划、已交付 S1/S2-A 和当时执行记录；未完成项由七阶段计划承接 |
| [S2-B1 普通僵尸 walk](superpowers/plans/2026-10-14-zombie-walk.md) | 行走实现与原始验收记录；平台缺项迁入阶段 1，保留原勾选 |
| [规则与卫生整合](superpowers/plans/2026-10-16-repository-rules-and-hygiene.md) | 已交付工程维护的范围和验收记录 |
| [2026-10-18 仓库整理](superpowers/plans/2026-10-18-repository-cleanup.md) | 本轮规则去重、元数据、导航修正与全仓审计范围；交付状态以 PR/CI 为准 |

## 文档维护规则

- README 只说明当前能做什么和最短入口；路线图只维护阶段摘要和下一步。
- 原则文档维护长期规则；贡献指南维护操作步骤和命令；测试/性能页维护方法；总计划维护阶段与依赖，当前阶段详案维护任务，远期启动时再细化；结果页保存真实命令、环境、证据和限制。
- 历史计划和结果页不因新提交改写当时事实。发现明确导航或事实错误时，保留原始适用范围并记录修正理由。
- 公开文档不得依赖被忽略的 `.superpowers/`、个人路径、凭据或临时产物。原始证据可在忽略的 `artifacts/` 中保存，公开结论必须在跟踪文档中自洽。
- 新增规则、计划、测试方法或 CI 步骤时，同步 [AGENTS.md](../AGENTS.md)、[项目原则](principles/README.md)、[CONTRIBUTING.md](../CONTRIBUTING.md) 和受影响入口，避免复制同一命令矩阵。
