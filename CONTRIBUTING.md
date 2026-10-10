# 贡献指南

当前仓库处于文档与结构阶段。未经明确任务授权，不要创建游戏实现或把原版素材复制进仓库。

## 开始前

阅读：

1. `README.md`
2. `docs/superpowers/specs/2026-10-07-repository-and-runtime-design.md`
3. `docs/superpowers/plans/2026-10-07-foundation-and-asset-audit.md`
4. `docs/architecture/README.md`
5. `assets-private/README.md`

用户资源仅可从本机显式路径读取。不要提交资源、缓存、截图、录像、绝对路径、本地配置或生成的包。

## 提交流程

使用短期分支和聚焦提交。Git 提交遵循 Conventional Commits，标题格式为 `type(scope): 中文说明`，scope 可省略。type 和 scope 使用标准英文标记，标题说明和正文使用中文；Rust、Macroquad、Cargo.lock、API 名称等技术名称保留原写法。例如：

```text
feat(core): 添加确定性模拟基础
fix(assets): 修复资源路径大小写冲突的诊断
docs: 记录原版行为核对结果
```

- type 按变更性质选择：`feat` 新功能、`fix` 修复、`docs` 文档、`refactor` 重构、`perf` 性能、`test` 测试、`build` 构建、`ci` 持续集成、`chore` 维护、`revert` 回退。
- scope 标识主要受影响的模块，优先使用 `data`、`core`、`assets`、`app`、`tool`；仓库级变更可省略 scope。
- 标题简洁、具体，直接描述变更；避免“更新”“修改一下”等无法说明内容的消息。正文按复杂度添加，与标题空一行，用中文说明必要的原因、行为影响和验证结果。
- 破坏性变更在 type/scope 后、冒号前加 `!`，或在页脚使用 `BREAKING CHANGE:`；用中文说明不兼容之处和迁移方式。关联 issue 可使用 `Refs:` 或 `Closes:` 等标准页脚标记。
- 一次提交应能说明一个可审阅的边界，不混入无关改动；提交前完成当前阶段适用的检查。修改接口时同时修改相关设计、计划或 ADR。

## 编码、命名与现代 Rust

- 所有可读文本文件使用 UTF-8、LF、无 BOM；提交前检查新增和修改文件。
- crate/目录使用小写 kebab-case，Rust 模块使用 snake_case；文件必须有明确用途，过时文件随同改动删除。
- 使用当前 stable Rust 支持的现代实践和最小 API。避免不必要的 `unsafe`，公开 API 写清错误、所有权和生命周期约定。
- 代码必须在 `rustfmt` 和 Clippy 质量门槛下通过；死代码、未使用依赖、未使用导入和未使用变量不能以警告形式长期保留。

## 验证

注册 package 后，提交前运行：

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --no-deps
cargo deny check
git diff --check
git ls-files
```

默认测试并行运行。只有测试确实依赖共享外部状态时，才对最小范围使用串行测试，并在测试注释或文档中写明原因。

覆盖率和性能检查是质量门槛的一部分。注册 package 后，建议在本地安装 `cargo-llvm-cov`，运行：

```sh
cargo llvm-cov --workspace --all-features --summary-only
```

核心算法和资源热路径必须提供可重复的 Criterion 基准（通常放在对应 package 的 `benches/`），运行：

```sh
cargo bench --workspace
```

基准用于发现回归和指导优化；先保证测试正确，再解释基准结果和优化收益。性能优化不得改变已验证的行为、事件顺序或回放结果。

没有可用 package 的结构阶段只运行：

```sh
cargo metadata --no-deps --format-version 1
git diff --check
```

报告平台支持时写清目标、命令、运行方式和限制；编译通过不等于窗口、音频、输入或存档已验证。

## 资源和许可

MIT 只覆盖本项目原创内容。贡献者必须确认新增文件没有复制原版素材、文本、商标或第三方内容；依赖许可证由 cargo-deny 检查。
