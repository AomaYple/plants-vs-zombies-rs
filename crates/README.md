# Cargo package 预留目录

当前阶段只落地仓库结构，不注册或创建 Rust package。设计中的 package 及边界见 `docs/architecture/README.md`。

当某个实现任务获得授权时，创建对应的 `crates/<name>/Cargo.toml` 与 `src/lib.rs`/`src/main.rs`，再把成员加入根 workspace，并生成 Cargo.lock。
