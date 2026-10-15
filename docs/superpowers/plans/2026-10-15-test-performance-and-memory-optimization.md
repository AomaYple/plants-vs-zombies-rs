# 测试速度与内存优化实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在保留全部正确性门槛的前提下，缩短本地测试反馈和 CI 关键路径，并用可重复的进程树测量评估峰值内存。

**Architecture:** 先建立可追溯的 Cargo 命令包装和 Windows 进程树测量入口，再用 `pvz-app` 的纯 feature 回归确认 `probe` 行为，最后隔离 CI cache key 并拆分互不依赖的 feature job。默认 workspace 测试、all-features 的 check/Clippy/doc、覆盖率、基准编译和平台冒烟继续由明确 job 覆盖；只有 feature 审计和回归通过后才收窄重复测试。

**Tech Stack:** stable Rust 1.99.0、Cargo、PowerShell 7、GitHub Actions、`actions/cache@v4`、`cargo llvm-cov` 0.9.1、Criterion 0.5、现有 Bevy 0.20.0 workspace。

**Spec:** [docs/superpowers/specs/2026-10-15-test-performance-and-memory-optimization.md](../specs/2026-10-15-test-performance-and-memory-optimization.md)

## Global Constraints

- 正确性第一、速度第二、内存第三；不得通过删除有效测试、降低覆盖范围、关闭 lint 或隐藏失败制造速度结果。
- 保留 `cargo test --workspace --locked` 的 doctest；默认与有效 feature 组合的构建、测试、Clippy、doc、覆盖率、基准和平台门槛分别报告。
- 共享 checkout 中的 Cargo 构建、测试和基准不得并发争抢同一个 `target/`；需要并发时使用不同 `CARGO_TARGET_DIR`。
- 测试默认并行；只对真实窗口、GPU、音频或无法隔离的外部状态局部串行，不设置全 workspace 的 `--test-threads=1`。
- 覆盖率使用独立、干净的插桩 profile，默认与 all-features 的路径和分母分别核对；doctest 由常规 workspace 测试验证。
- 不改变核心规则、事件顺序、确定性回放、资源合同、回放格式、平台支持声明或原版行为证据边界。
- 正式内存结果必须记录进程树峰值工作集或适用分配统计、真实退出码、采样范围、预热和重复次数；Criterion 计时不能替代内存证据。
- 文本文件使用 UTF-8、LF、无 BOM、末尾换行；项目自写说明和诊断使用中文，技术标识符及标准命令保持原写法。
- 每个独立目的使用 `type(scope): 中文说明` 提交；只在所有适用验证完成后推送、创建 PR、合并和清理分支。

---

### Task 1: 建立可靠的 Cargo 进程树测量入口

**Files:**
- Create: `tools/measure-cargo.ps1`
- Modify: `tools/README.md`
- Test: `tools/measure-cargo.ps1`（通过 PowerShell 自检命令验证）

**Interfaces:**
- Consumes: `-OutputLog` 输出前缀、`-Executable` 可执行文件路径、`-ArgumentList` 字符串数组、可选 `-WorkingDirectory` 和 `-SampleMilliseconds`。
- Produces: JSON stdout 摘要和 `<OutputLog>.out`、`<OutputLog>.err`；摘要字段固定为 `command`、`exit_code`、`elapsed_seconds`、`peak_working_set_mb`、`samples`、`stdout`、`stderr`。

- [ ] **Step 1: 写进程树采样的失败自检场景**

在 `tools/README.md` 中先记录两个可重复验收命令：一个调用 `cargo.exe --version` 验证成功退出码和输出，另一个调用 PowerShell 生成退出码 7 的短命令，验证脚本保留非零退出码并仍写出 JSON。命令必须使用参数数组，不能把整个命令拼接成一个可执行文件名。

```powershell
pwsh -NoProfile -File tools/measure-cargo.ps1 `
  -OutputLog artifacts/validation/test-optimization-2026-10-15/measure-version `
  -Executable cargo.exe `
  -ArgumentList @('--version')

pwsh -NoProfile -File tools/measure-cargo.ps1 `
  -OutputLog artifacts/validation/test-optimization-2026-10-15/measure-failure `
  -Executable pwsh `
  -ArgumentList @('-NoProfile', '-Command', 'exit 7')
```

- [ ] **Step 2: 实现安全的进程启动和退出采样**

使用 `Start-Process -PassThru -WindowStyle Hidden` 启动根进程，将 `-ArgumentList` 原样传给 PowerShell 的数组参数；每个采样周期用 `Win32_Process.ParentProcessId` 递归发现子进程，再对去重后的 PID 求 `WorkingSet64` 总和。进程树发现和进程读取失败只跳过已经退出的 PID；根进程退出后继续刷新并等待子进程全部退出，超过明确的收口期限则返回非零并在 JSON 中说明测量失败。stdout/stderr 重定向文件的父目录由脚本创建。

```powershell
$root = Start-Process -FilePath $Executable -ArgumentList $ArgumentList `
  -WorkingDirectory $WorkingDirectory -RedirectStandardOutput "$OutputLog.out" `
  -RedirectStandardError "$OutputLog.err" -PassThru -WindowStyle Hidden
while ($true) {
  $ids = @(Get-DescendantProcessIds -RootId $root.Id)
  $current = ($ids | ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue } |
    Measure-Object -Property WorkingSet64 -Sum).Sum
  $peak = [Math]::Max($peak, [int64]$current)
  if ($root.HasExited -and (Test-ProcessTreeExited -Ids $ids)) { break }
  Start-Sleep -Milliseconds $SampleMilliseconds
}
$root.Refresh()
$summary = [ordered]@{ exit_code = $root.ExitCode; peak_working_set_mb = [Math]::Round($peak / 1MB, 2) }
$summary | ConvertTo-Json -Compress
if ($root.ExitCode -ne 0) { exit $root.ExitCode }
```

实现时将示例中的 `Get-DescendantProcessIds` 和 `Test-ProcessTreeExited` 写成同文件内的私有函数，处理 PID 重用和已退出进程；不能依赖 `$LASTEXITCODE` 读取 Cargo 子进程结果。

- [ ] **Step 3: 运行成功、失败和真实测试测量自检**

运行上述两个短命令，再串行运行热的 `cargo test --workspace --locked --no-run`。检查三个 JSON 的 `exit_code` 分别为 0、7、0，输出文件存在，`samples` 大于 0，测试测量在根进程和子进程都退出后才结束。失败场景不得被脚本吞掉。

- [ ] **Step 4: 提交测量入口**

```powershell
git add tools/measure-cargo.ps1 tools/README.md
git diff --cached --check
git commit -m "perf(tool): 建立可复现的 Cargo 进程测量"
```

### Task 2: 提供不改变门槛的本地反馈入口

**Files:**
- Create: `tools/test-feedback.ps1`
- Modify: `tools/README.md`
- Modify: `CONTRIBUTING.md`（验证流程的本地反馈层级）
- Test: `tools/test-feedback.ps1`（命令分派和退出码自检）

**Interfaces:**
- Consumes: `-Mode`（`package`、`workspace`、`probe`、`full`）、`-Package` 可重复参数、`-NoRun` 和可选 `-TargetDirectory`。
- Produces: 透传 Cargo stdout/stderr，失败时返回 Cargo 的真实退出码；不改变测试线程数，不在同一调用中启动并行 Cargo 进程。

- [ ] **Step 1: 固定模式到 Cargo 参数的测试表**

在 `tools/README.md` 中写出脚本的参数映射，并用 `-WhatIf` 或等价的 dry-run 输出检查参数，不实际启动第二个 Cargo 进程。映射必须明确如下：

```text
package   -> cargo test -p <Package> --locked [--no-run]
workspace -> cargo test --workspace --locked [--no-run]
probe     -> cargo test -p pvz-app --features probe --locked [--no-run]
full      -> cargo test --workspace --locked --no-fail-fast
```

`-Package` 为空时，`package` 模式直接以参数错误退出；`-TargetDirectory` 只设置当前进程的 `CARGO_TARGET_DIR`，不写入全局配置。

- [ ] **Step 2: 实现参数校验和单一 Cargo 调用**

脚本使用 `[ValidateSet]` 校验模式，以数组构造 `$cargoArgs`，先设置局部环境变量，再调用 `& cargo @cargoArgs`。调用结束后立即执行 `exit $LASTEXITCODE`；不要用字符串拆分命令，不要添加 `--test-threads=1` 或未经测量的 `--jobs`。

```powershell
$cargoArgs = @('test', '--locked')
switch ($Mode) {
  'package'   { $cargoArgs += @('-p', $Package); if ($NoRun) { $cargoArgs += '--no-run' } }
  'workspace' { $cargoArgs += '--workspace'; if ($NoRun) { $cargoArgs += '--no-run' } }
  'probe'     { $cargoArgs += @('-p', 'pvz-app', '--features', 'probe'); if ($NoRun) { $cargoArgs += '--no-run' } }
  'full'      { $cargoArgs += @('--workspace', '--no-fail-fast') }
}
if ($TargetDirectory) { $env:CARGO_TARGET_DIR = (Resolve-Path $TargetDirectory).Path }
& cargo @cargoArgs
exit $LASTEXITCODE
```

- [ ] **Step 3: 运行本地反馈自检**

串行执行 `package`、`workspace -NoRun`、`probe` 和 `full` 四个模式；检查 package 模式只接受一个已知 package，probe 模式在默认 feature 下实际构建并运行测试，full 模式仍包含 doctest。用一个不存在的 package 验证非零返回，确认脚本没有将失败转换为成功。

- [ ] **Step 4: 补充贡献指南中的使用边界**

在 `CONTRIBUTING.md` 的反馈层级表旁说明：局部模式只用于开发反馈，提交仍须执行完整 L0–L3；跨 package 改动使用一次 workspace 调用；覆盖率、基准、窗口和 Web 检查不通过该脚本隐式替代。记录共享 `target/` 下不要并发运行多个脚本实例。

- [ ] **Step 5: 提交本地反馈入口**

```powershell
git add tools/test-feedback.ps1 tools/README.md CONTRIBUTING.md
git diff --cached --check
git commit -m "perf(tool): 增加分层本地测试反馈入口"
```

### Task 3: 锁定 `probe` feature 行为并完成 feature 审计

**Files:**
- Modify: `crates/pvz-app/src/main.rs:69-82,640-745`
- Test: `crates/pvz-app/src/main.rs` 内的 `#[cfg(test)]` 模块

**Interfaces:**
- Consumes: 当前 `Options`、native CLI 解析和 `cfg!(feature = "probe")`。
- Produces: 私有 `apply_compile_time_options(Options) -> Options`，主入口通过它应用 feature 默认值；默认构建和 `--features probe` 构建各有明确回归断言。

- [ ] **Step 1: 先写默认和 probe 的失败回归测试**

在现有 app 单元测试模块中加入两个行为断言：一个验证显式 `probe: true` 永远保持 true，另一个验证 `probe: false` 的结果等于当前编译 feature。测试通过私有 helper 的结果观察模式，不创建窗口、不加载资源。

```rust
#[test]
fn 编译期_probe_默认值与feature一致() {
    let options = apply_compile_time_options(Options {
        probe: false,
        smoke: None,
        screenshot: false,
        audio_smoke: false,
        slice_smoke: false,
        failure_smoke: false,
    });
    assert_eq!(options.probe, cfg!(feature = "probe"));
}

#[test]
fn 显式_probe不会被编译期默认值关闭() {
    assert!(apply_compile_time_options(Options { probe: true, ..default_options() }).probe);
}
```

- [ ] **Step 2: 运行测试确认缺少 helper 时失败**

运行：

```powershell
cargo test -p pvz-app --locked 编译期_probe_默认值与feature一致
```

预期：编译失败并指出 `apply_compile_time_options` 或 `default_options` 尚不存在；该失败只验证测试先行，不能提交。

- [ ] **Step 3: 提取纯 helper 并保持主入口行为**

在 `main` 前增加 `default_options() -> Options` 和 `apply_compile_time_options(mut options: Options) -> Options`。native 解析和 wasm 默认构造仍保持原字段；只把原来的 `options.probe |= cfg!(feature = "probe")` 替换为 helper 调用，不改变 CLI 冲突校验、资源加载和运行时逻辑。

```rust
fn default_options() -> Options {
    Options {
        probe: false,
        smoke: None,
        screenshot: false,
        audio_smoke: false,
        slice_smoke: false,
        failure_smoke: false,
    }
}

fn apply_compile_time_options(mut options: Options) -> Options {
    options.probe |= cfg!(feature = "probe");
    options
}
```

- [ ] **Step 4: 在两个 feature 组合下运行回归并审计源码**

运行默认和 probe 测试：

```powershell
cargo test -p pvz-app --locked
cargo test -p pvz-app --features probe --locked
```

再执行：

```powershell
rg -n "cfg\s*!?\(.*feature|\[features\]|build\.rs|\[build-dependencies\]" crates tools Cargo.toml
cargo metadata --no-deps --locked --format-version 1
```

审计结果必须记录只有 `pvz-app/probe` 一个项目 feature、没有 build script 或隐藏 feature 条件；若结果不同，保留 workspace all-features 测试，不执行后续收窄。

- [ ] **Step 5: 提交 feature 回归**

```powershell
git add crates/pvz-app/src/main.rs
git diff --cached --check
git commit -m "test(app): 锁定 probe feature 默认模式"
```

### Task 4: 重排 CI 并加入隔离缓存

**Files:**
- Modify: `.github/workflows/ci.yml:191-255,273-312,314-335`
- Modify: `CONTRIBUTING.md`（CI job 与缓存边界说明）
- Test: `.github/workflows/ci.yml`（actionlint 和本地 YAML/脚本静态核对）

**Interfaces:**
- Consumes: structure job 的 package/app outputs、matrix OS、feature 审计结论和既有 Windows/Web/coverage 命令。
- Produces: 默认和 all-features 独立 runner、显式 Cargo cache key、原有状态检查名/coverage artifact 名称和所有适用平台门槛。

- [ ] **Step 1: 为 cache key 写出隔离矩阵**

在 CI 变更前在 `CONTRIBUTING.md` 记录 key 的组成：`${{ runner.os }}`、Rust `1.99.0`、job 角色、target/架构、profile、feature 集和 `hashFiles('**/Cargo.lock')`。默认、all-features、coverage 和 Web 不得共用 target artifact；registry/git 缓存可以在同一 OS、toolchain 和 lockfile 下复用。

```yaml
- name: 缓存 Cargo registry、Git checkout 和构建产物
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-rust-1.99.0-implementation-${{ matrix.feature_set }}-${{ hashFiles('**/Cargo.lock') }}
```

对没有 `matrix.feature_set` 的 job 使用固定角色字符串；coverage 采用独立 `CARGO_TARGET_DIR` 或独立角色 key，避免复用插桩对象。

- [ ] **Step 2: 将实现 job 扩展为 OS × feature_set 矩阵**

把现有 `implementation` matrix 扩展为 `os: [ubuntu-latest, windows-latest, macos-latest]` 和 `feature_set: [default, all]`，job 名称包含 feature 集。默认分支保留 check、fmt、workspace test、Clippy、doc 和 Windows 三条冒烟；all 分支执行 workspace `--all-features` 的 check、test、Clippy、doc。每个 runner 内仍按顺序执行 Cargo 命令，Windows 冒烟只使用默认分支已生成的 `target/debug/pvz-app.exe`。

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    feature_set: [default, all]
name: Rust 实现检查 (${{ matrix.feature_set }})
```

在 feature 审计未证明测试等价前，all 分支保留 `cargo test --workspace --all-features --locked`。审计通过后才改为默认 workspace 测试加 `cargo test -p pvz-app --features probe --locked`，并继续保留 all-features check/Clippy/doc；这一步单独提交并记录测试数量和覆盖率分母未被错误替换。

- [ ] **Step 3: 消除已证明的重复 probe Clippy 调用**

all 分支的 workspace `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 已覆盖 `pvz-app/probe`，因此在 feature 审计通过后删除单独的 `cargo clippy -p pvz-app --features probe` 步骤；若审计或 CI 日志显示覆盖条件不等价，恢复该步骤。默认分支不运行需要 probe feature 的冒烟。

- [ ] **Step 4: 给 coverage 和 Web job 加上角色隔离缓存**

coverage 的 default/all 矩阵保留独立 LCOV/JSON artifact 名称，并用各自的 target 目录或 key；默认基准 `cargo bench --workspace --locked --no-run` 继续只执行一次。Web job 的 native release target、`wasm32-unknown-unknown` target 和 `web`/`web-probe` 输出使用 web 角色缓存，不与 native test 角色共用。

- [ ] **Step 5: 静态检查并提交 CI 重排**

运行：

```powershell
actionlint .github/workflows/ci.yml
git diff --check
```

检查所有 job 仍有 structure 依赖、失败返回码、Windows timeout、coverage artifact 上传和 Web bindgen 输出；然后提交：

```powershell
git add .github/workflows/ci.yml CONTRIBUTING.md
git diff --cached --check
git commit -m "ci: 并行隔离默认与全 feature 验证"
```

### Task 5: 建立同条件性能和内存证据

**Files:**
- Create: `artifacts/validation/test-optimization-2026-10-15/`（忽略的原始日志和 JSON）
- Modify: `docs/performance/README.md`（接受的测量入口和本轮结果索引）
- Modify: `docs/superpowers/specs/2026-10-15-test-performance-and-memory-optimization.md`（仅在结果改变设计结论时更新状态）
- Test: `tools/measure-cargo.ps1`、本地反馈入口、CI job 日志

**Interfaces:**
- Consumes: Task 1 的 JSON 摘要、Task 2 的反馈模式、Task 3 的 feature 结果和 Task 4 的 CI job 日志。
- Produces: 冷/热、默认/all-features、`--no-run`、coverage 和基准入口的可比较结果；每个场景至少三次重复，包含提交 SHA、工具链、target、feature、缓存状态、jobs、elapsed、exit code 和峰值工作集。

- [ ] **Step 1: 串行取得优化前后同条件本地基线**

停止其他 Cargo、测试、基准、窗口和 Web 服务；使用独立 target 目录分别运行默认 workspace 测试、`--no-run`、all-features 测试和基准 `--no-run`。冷缓存和热缓存各重复三次，保留 stdout/stderr，不把已有有缺陷脚本的 122–124 MB 读数作为基线。

```powershell
pwsh -NoProfile -File tools/measure-cargo.ps1 `
  -OutputLog artifacts/validation/test-optimization-2026-10-15/baseline-default-test-01 `
  -Executable cargo.exe `
  -ArgumentList @('test', '--workspace', '--locked', '--no-fail-fast') `
  -WorkingDirectory (Get-Location)
```

- [ ] **Step 2: 验证正确性和行为不变**

对每次候选改动至少运行 `cargo fmt --all -- --check`、`cargo check --workspace --all-targets --locked`、默认 workspace 测试、all-features check/test/Clippy/doc、`cargo deny check`、受影响 Criterion 基准编译、确定性回放/事件顺序测试和适用 Windows/Web 冒烟。覆盖率 default/all 各在干净 target 中生成并核对 JSON/LCOV 源路径和分母。

- [ ] **Step 3: 比较 Cargo jobs 和 cache 方案**

在相同输入下比较 Cargo 默认 jobs 与一个受控较低值的 elapsed 和峰值工作集；只有速度不退化且峰值下降时才保留受控 jobs。对 CI 运行记录 cache hit/miss、每个 job 的步骤耗时、排队和总墙钟；不要用不同 runner 或删减检查的结果作前后结论。

- [ ] **Step 4: 仅在有证据时记录内存优化**

若峰值下降来自 target 任务隔离、cache 或 jobs 调整，记录它属于验证过程 OS 工作集改善，不称为游戏运行时堆内存改善。若修改夹具/解析生命周期，追加分配统计和回放/事件回归；没有可重复收益就回退该候选改动。

- [ ] **Step 5: 更新性能方法和结果边界**

在 `docs/performance/README.md` 增加本轮脚本调用、冷/热条件、重复次数、进程树口径和结果索引，明确 CI/CPU/CLI 测量不等于玩家平台或原版行为验收。原始日志只放现有忽略目录，不把缓存、截图或个人路径提交到 Git。

- [ ] **Step 6: 提交测量结果文档**

```powershell
git add docs/performance/README.md docs/superpowers/specs/2026-10-15-test-performance-and-memory-optimization.md
git diff --cached --check
git commit -m "perf: 记录测试验证速度与内存测量"
```

### Task 6: 完成全仓验收和交付准备

**Files:**
- Modify: `docs/superpowers/plans/2026-10-15-test-performance-and-memory-optimization.md`（勾选实际完成步骤并记录结果）
- Modify: `docs/superpowers/specs/2026-10-15-test-performance-and-memory-optimization.md`（同步最终边界和未验证项）
- Test: 全部适用本地命令、远端 CI 和 Git 状态

**Interfaces:**
- Consumes: Tasks 1–5 的提交、原始证据、CI job 状态和缓存命中记录。
- Produces: 可审阅的最终分支、PR 所需验证摘要和明确的未验证平台/原版行为边界。

- [ ] **Step 1: 运行提交前完整检查**

在仓库根目录串行执行适用命令：

```powershell
cargo fmt --all -- --check
cargo metadata --no-deps --locked --format-version 1
cargo check --workspace --all-targets --locked
cargo test --workspace --locked --no-fail-fast
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --all-features --no-deps --locked
cargo deny check
git diff --check
git diff --cached --check
```

再按验收矩阵运行 coverage、基准编译、Windows 冒烟、WASM default/probe bindgen 和 actionlint；每一项记录真实退出码和对应提交 SHA。

- [ ] **Step 2: 核对差异、文件边界和证据**

检查 `git status --short`、`git diff --stat`、`git diff --name-only origin/main...HEAD`，确认没有 `target/`、凭据、原版程序、截图或临时日志进入跟踪；检查新工具的调用方、文档链接、Cargo.lock 未被无关修改。清理只确认无用途的临时文件，不删除仍被计划、CI 或验收引用的证据。

- [ ] **Step 3: 请求代码复核并修复反馈**

按用户选定的执行技能完成实施中的逐任务 review；每条意见都回到对应测试或契约验证，重新运行受影响命令。任何失败、未运行或环境限制写入最终 PR 描述，不把旧主线 CI 结果借给新提交。

- [ ] **Step 4: 按授权推送、创建 PR、等待 CI 并合并**

推送当前短期分支，创建 PR，等待所有必需 job 完成；核对 default/all-features、coverage、Web、Windows/macOS/Linux 和依赖审计都对应新提交。用户已授权的外部 Git 操作按仓库贡献流程执行；若授权范围只覆盖本地，则停在可审阅分支并报告待执行动作。

- [ ] **Step 5: 核验主线并清理分支**

合并后更新本地 `main`，确认合入提交与远端 CI 一致，再按授权删除已完成的本地/远端任务分支。最后报告本地验证、远端 CI、合并状态、缓存/内存测量、未验证平台和原版行为边界。

## 计划自审

- 设计文档中的测量脚本、分层本地反馈、feature 回归、CI cache/矩阵、内存口径、回滚条件和最终验收分别由 Tasks 1–6 覆盖。
- 计划没有把旧的失败内存采样当作结果，也没有把 CI、CPU、CLI 或 Web 构建扩展为玩家平台或原版行为证据。
- `probe` 测试收窄是条件分支：审计和专门回归通过前继续运行 workspace all-features 测试；审计失败时恢复完整测试。
- 每个代码改动任务包含先写回归、运行失败、最小实现、运行通过和提交步骤；CI/脚本任务包含静态检查、退出码检查和回滚条件。
- 计划中没有未填充的占位步骤；所有路径均为仓库现有或本计划明确创建的文件。
