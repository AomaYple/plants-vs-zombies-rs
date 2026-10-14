# 文档索引

| 文档 | 用途 |
| --- | --- |
| [项目原则](principles/README.md) | 工程原则与游戏开发原则的分类入口 |
| [工程原则](principles/engineering.md) | Rust、测试、覆盖率、性能、卫生、Git 与交付规则 |
| [游戏开发原则](principles/game-development.md) | 行为还原、开发效率、运行时、资源、渲染与平台规则 |
| [完整设计](superpowers/specs/2026-10-07-repository-and-runtime-design.md) | 当前仓库与运行架构的设计基线 |
| [版本化白天内容设计](superpowers/specs/2026-10-12-versioned-day-content.md) | 严格内容格式、纯装配与实际时钟消费边界 |
| [原版行为基线测量设计](superpowers/specs/2026-10-12-reference-behavior-measurement.md) | 可观察行为、证据分类与规则替换边界 |
| [当前总计划：阶段性最小可玩垂直切片](superpowers/plans/2026-10-13-minimum-playable-vertical-slices.md) | S1 已完成开发闭环、S2-A 已交付、当前 S2-B1、后续阶段与证据边界 |
| [当前细化计划：普通僵尸 walk（S2-B1）](superpowers/plans/2026-10-14-zombie-walk.md) | 缓存/真实 Scene 与维护后工程验证已完成；源码已纳入维护提交，Windows 缺项、Web 诊断和平台最终复核待完成 |
| [已完成：向日葵 idle（S2-A）](superpowers/plans/2026-10-14-sunflower-idle.md) | 可玩增量、实跑与提交 `3e6634e` 对应八项 CI 通过；流程偏差与限制保留 |
| [横向专题计划：原版行为测量与并行开发](superpowers/plans/2026-10-12-reference-behavior-measurement.md) | 案例依赖、窗口阻塞处理、独立开发与逐参数采纳 |
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
| [版本化内容结果](behavior/content-foundation-results.md) | 内容、S1/S2-A 历史交付与 S2-B1 冻结源码的测试/覆盖率/基准、Windows/Web 实测及待验范围 |
| [切片阶段性结果](behavior/vertical-slice-results.md) | 首个开发切片的测试、运行、覆盖率与基准历史证据 |
| [平台探针结果](behavior/platform-probe-results.md) | 任务 5 的窗口、输入、图形与音频历史验证 |
| [技术决策](adr/README.md) | 已采纳决策与重新评估条件 |

文档职责与更新约定见 [原则索引](principles/README.md#文档职责与维护)。当前阶段与后续顺序只在总计划和路线图维护；结果页保存各批源码、命令、环境及实际验收范围。S1 与 S2-A 已完成交付；S2-B1 冻结源码的历史证据保留，维护后工程验证已完成，源码已纳入维护提交；Git/CI 记录见[维护结果](behavior/content-foundation-results.md#2026-10-15-全仓审计与维护阶段性收尾)。Windows 正常窗口缺项、Web 诊断和平台最终复核仍未闭合，整个 S2 保持未完成。文档中“计划”“待接入”“待测量”表示工作尚未完成；开发参数作为原版规格使用或验收前须完成相应测量。
