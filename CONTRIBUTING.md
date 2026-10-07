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

使用短期分支和聚焦提交。提交标题采用简单的 Conventional Commit 风格，例如：

```text
feat(core): add deterministic simulation shell
fix(assets): report case collision
docs: record reference behavior
```

一次提交应能说明一个可审阅的边界。修改接口时同时修改相关设计、计划或 ADR。

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

没有可用 package 的结构阶段只运行：

```sh
cargo metadata --no-deps --format-version 1
git diff --check
```

报告平台支持时写清目标、命令、运行方式和限制；编译通过不等于窗口、音频、输入或存档已验证。

## 资源和许可

MIT 只覆盖本项目原创内容。贡献者必须确认新增文件没有复制原版素材、文本、商标或第三方内容；依赖许可证由 cargo-deny 检查。
