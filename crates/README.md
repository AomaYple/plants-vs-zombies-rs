# Cargo package 目录

本目录已注册 `pvz-data`、`pvz-core`、`pvz-assets` 和 `pvz-app`；宿主 CLI 位于 `tools/pvz-asset-tool/`。基础设施、首个白天开发切片和现有白天内容数据化已通过本地实现验收，本批证据见[版本化内容结果](../docs/behavior/content-foundation-results.md)，收口状态见 [实施计划](../docs/superpowers/plans/2026-10-12-versioned-day-content.md)，依赖方向见 [架构](../docs/architecture/README.md)。

新 package 只有在有源码入口、实际消费者和验收目的时才加入根 workspace，并同步 `Cargo.lock`、lint、CI 和阶段文档。
