# Cargo package 目录

本目录已注册 [pvz-data](pvz-data/README.md)、[pvz-core](pvz-core/README.md)、[pvz-assets](pvz-assets/README.md) 和 [pvz-app](pvz-app/README.md)；宿主 CLI 见[工具目录](../tools/README.md)。各包说明维护局部接口与验证入口，依赖方向见[架构](../docs/architecture/README.md)。基础设施、首个白天开发切片和内容数据化的历史验收，以及 S1/S2-A 交付与 B1 工程/部分平台证据见[增量结果](../docs/behavior/content-foundation-results.md)；当前阶段与待验项统一见[总计划](../docs/superpowers/plans/2026-10-17-phased-development.md)。

新 package 只有在有源码入口、实际消费者和验收目的时才加入根 workspace，并同步 `Cargo.lock`、lint、CI 和阶段文档。
