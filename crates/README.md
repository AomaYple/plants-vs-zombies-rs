# Cargo package 目录

`pvz-data`、`pvz-core` 和 `pvz-assets` 已注册并按职责独立验收；`pvz-app` 仍待窗口与渲染探针任务。设计中的 package 及依赖边界见 [架构](../docs/architecture/README.md)。

新 package 只有在有源码入口、实际消费者和验收目的时才加入根 workspace，并同步 `Cargo.lock`、lint、CI 和阶段文档。
