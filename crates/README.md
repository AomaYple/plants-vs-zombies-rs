# Cargo package 预留目录

当前仓库处于文档、结构与参考资源阶段，尚未注册或创建 Rust package。设计中的 package 及边界见 [架构](../docs/architecture/README.md)。

当某个实现任务获得授权时，创建对应的 `crates/<name>/Cargo.toml` 与 `src/lib.rs`/`src/main.rs`，再把成员加入根 workspace，并生成 Cargo.lock。
