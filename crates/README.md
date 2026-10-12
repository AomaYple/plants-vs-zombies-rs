# Cargo package 目录

本目录已注册 `pvz-data`、`pvz-core`、`pvz-assets` 和 `pvz-app`；宿主 CLI 位于 `tools/pvz-asset-tool/`。基础设施与首个白天开发切片已完成工程验收，当前推进内容数据化，状态见 [实施计划](../docs/superpowers/plans/2026-10-12-versioned-day-content.md)，依赖方向见 [架构](../docs/architecture/README.md)。

新 package 只有在有源码入口、实际消费者和验收目的时才加入根 workspace，并同步 `Cargo.lock`、lint、CI 和阶段文档。
