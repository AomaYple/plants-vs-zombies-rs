# 文档索引

| 文档 | 用途 |
| --- | --- |
| [项目原则](principles/README.md) | 工程原则与游戏开发原则的分类入口 |
| [工程原则](principles/engineering.md) | Rust、测试、覆盖率、性能、卫生、Git 与交付规则 |
| [游戏开发原则](principles/game-development.md) | 行为还原、开发效率、运行时、资源、渲染与平台规则 |
| [完整设计](superpowers/specs/2026-10-07-repository-and-runtime-design.md) | 当前仓库与运行架构的设计基线 |
| [版本化白天内容设计](superpowers/specs/2026-10-12-versioned-day-content.md) | 严格内容格式、纯装配与实际时钟消费边界 |
| [原版行为基线测量设计](superpowers/specs/2026-10-12-reference-behavior-measurement.md) | 可观察行为、证据分类与规则替换边界 |
| [当前实施计划：原版行为基线测量](superpowers/plans/2026-10-12-reference-behavior-measurement.md) | 原版可观察行为证据、规则替换与回放验收 |
| [已完成：版本化白天内容计划](superpowers/plans/2026-10-12-versioned-day-content.md) | 已完成的白天内容数据化与应用装配验收 |
| [已完成：基础与切片历史计划](superpowers/plans/2026-10-07-foundation-and-asset-audit.md) | 已完成任务及历史验收索引 |
| [路线图](roadmap.md) | 分阶段交付、依赖关系与完成标准 |
| [架构](architecture/README.md) | crate、模块、数据流与依赖方向 |
| [测试策略](testing/README.md) | 测试类型、状态隔离、覆盖率与平台验收 |
| [性能策略](performance/README.md) | 时间、内存与分配测量，优化比较与行为保护 |
| [贡献指南](../CONTRIBUTING.md) | 工具链、命令、提交与评审流程 |
| [参考基线](behavior/reference-baseline.md) | EXE、资源包、拆包一致性及画布证据 |
| [参考资源](../assets/README.md) | 版本化资源、导入格式与清单校验 |
| [行为测量](behavior/README.md) | 如何记录观察、控制变量、形成规则测试 |
| [开发切片合同](behavior/development-slice.md) | 原创开发参数、确定性与表现边界，不代表原版规格 |
| [版本化内容结果](behavior/content-foundation-results.md) | 本批测试、干净覆盖率、受影响基准与 Windows/Web 实测范围 |
| [切片阶段性结果](behavior/vertical-slice-results.md) | 首个开发切片的测试、运行、覆盖率与基准历史证据 |
| [平台探针结果](behavior/platform-probe-results.md) | 任务 5 的窗口、输入、图形与音频历史验证 |
| [技术决策](adr/README.md) | 已采纳决策与重新评估条件 |

文档职责与更新约定见 [原则索引](principles/README.md#文档职责与维护)。现有白天参数、初始状态与有序波次的数据化已通过本地实现验收，整体文档复核与 Git 交付按近期计划收口。文档中“计划”“待接入”“待测量”表示工作尚未完成；开发切片可以使用明确标记的开发参数，参数作为原版规格使用或验收前须完成相应测量。
