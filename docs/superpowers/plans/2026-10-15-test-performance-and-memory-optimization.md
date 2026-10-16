# 测试速度与内存优化实施计划

本计划记录测试反馈、测量入口、CI 调度和局部测试实现优化；当前结果已随 PR #7 合入主线。步骤勾选只表示本地计划工作完成，远端状态以对应提交、PR 和主线 CI 核验为准。

**Goal:** 在保留全部正确性门槛的前提下，缩短本地测试反馈和 CI 关键路径，并用可重复的进程树测量评估峰值内存。

**Architecture:** 先建立可追溯的 Cargo 命令包装和 Windows 进程树测量入口，再用 `pvz-app` 的纯 feature 回归确认 `probe` 行为，最后加入按角色隔离的 CI 依赖缓存并消除已证明重复的步骤。默认 workspace 测试、all-features 的 check/test/Clippy/doc、覆盖率、基准编译和平台冒烟继续由明确门槛覆盖；不把 OS × feature 扩成六个 runner，也不在没有同条件测量前收窄 all-features 测试。

**Tech Stack:** stable Rust 1.99.0、Cargo、PowerShell 7、GitHub Actions、`actions/cache@v4`、`cargo llvm-cov` 0.9.1、Criterion 0.5、现有 Bevy 0.20.0 workspace。

**Spec:** [docs/superpowers/specs/2026-10-15-test-performance-and-memory-optimization.md](../specs/2026-10-15-test-performance-and-memory-optimization.md)

## Global Constraints

- 正确性第一、速度第二、内存第三；不得通过删除有效测试、降低覆盖范围、关闭 lint 或隐藏失败制造速度结果。
- 保留 `cargo test --workspace --locked` 的 doctest；默认与有效 feature 组合的 check、test、Clippy、doc、覆盖率、基准和平台门槛分别报告。all-features workspace test 在证据证明等价替代前不得删除。
- 共享 checkout 中的 Cargo 构建、测试和基准不得并发争抢同一个 `target/`；需要并发时使用不同 `CARGO_TARGET_DIR`。
- 测试默认并行；只对真实窗口、GPU、音频或无法隔离的外部状态局部串行，不设置全 workspace 的 `--test-threads=1`。
- 覆盖率使用独立、干净且不恢复旧插桩对象的 profile；default/all-features 的路径和分母分别核对，插桩 `target`、`profraw` 和 `profdata` 不进入缓存；doctest 由常规 workspace 测试验证，不把 llvm-cov 分母误称为 doctest 覆盖率。
- 不改变核心规则、事件顺序、确定性回放、资源合同、回放格式、平台支持声明或原版行为证据边界。
- 正式内存结果必须记录根进程与后代进程树的峰值工作集或适用分配统计、真实退出码、采样范围、预热和重复次数；启动失败、超时、树未收口或采样中断的结果作废，Criterion 计时不能替代内存证据。
- 性能比较至少使用同一提交、工具链、runner、target、feature、缓存状态和 jobs；默认只报告中位数与离散范围，不设置未经噪声测量验证的自动门槛。收益至少有三次有效重复；速度收益须有超出实测噪声的可重复墙钟下降，内存收益还须有可重复峰值下降并证明速度没有实质退化。
- 原定 10% 容差仅作为探索时的警戒线，不能把容差内的实际降速写成提速或自动接受为内存收益。
- 文本文件使用 UTF-8、LF、无 BOM、末尾换行；项目自写说明和诊断使用中文，技术标识符及标准命令保持原写法。
- 子代理在本计划中只做只读规则、CI/feature 审计和最终复核，不修改源码、脚本、CI、Cargo 配置或项目文档；所有实施、Cargo 调度和整合由主代理完成。若以后授权子代理实施，必须使用隔离 worktree 并明确文件归属。
- 每个独立目的使用 `type(scope): 中文说明` 提交。本地提交只是可回退的审计节点；推送、创建 PR、合并和删除远端分支只有在用户明确或持续授权覆盖时才执行。

---

### Task 1: 建立可靠的 Cargo 进程树测量入口

**Files:**
- Create: `tools/measure-cargo.ps1`
- Modify: `tools/README.md`
- Test: `tools/measure-cargo.ps1`（通过 PowerShell 自检命令验证）

**Interfaces:**
- Consumes: `-OutputLog` 输出前缀、`-Executable` 可执行文件路径、`-ArgumentList` 字符串数组、可选 `-WorkingDirectory`、`-SampleMilliseconds` 和 `-TreeTimeoutSeconds`（默认 30）。
- Produces: JSON stdout 摘要和 `<OutputLog>.out`、`<OutputLog>.err`；摘要字段固定为 `command`、`exit_code`、`elapsed_seconds`、`peak_working_set_mb`、`samples`、`process_count_peak`、`stdout`、`stderr`、`measurement_error`。`command` 为结构化 executable/argv，输出文件保存原始流。

- [x] **Step 1: 写进程树采样的失败自检场景**

在 `tools/README.md` 中先记录两个可重复验收命令：一个调用 `cargo.exe --version` 验证成功退出码和输出，另一个调用 PowerShell 生成退出码 7 的短命令，验证脚本保留非零退出码并仍写出 JSON。命令必须使用参数数组，不能把整个命令拼接成一个可执行文件名。

```powershell
& ./tools/measure-cargo.ps1 `
  -OutputLog artifacts/validation/test-optimization-2026-10-15/measure-version `
  -Executable cargo.exe `
  -ArgumentList @('--version')

& ./tools/measure-cargo.ps1 `
  -OutputLog artifacts/validation/test-optimization-2026-10-15/measure-failure `
  -Executable pwsh `
  -ArgumentList @('-NoProfile', '-Command', 'exit 7')
```

- [x] **Step 2: 实现安全的进程启动和退出采样**

使用 `.NET System.Diagnostics.ProcessStartInfo`，逐项调用 `ArgumentList.Add()` 传递 argv，避免 `Start-Process -ArgumentList` 的 Windows quoting 歧义；`-WorkingDirectory` 先解析为绝对 `.Path`。每个采样周期用 `Win32_Process.ParentProcessId` 递归发现子进程，将根 PID 与去重后的后代 PID 一并求 `WorkingSet64` 总和。只忽略已经退出或 PID 重用的进程读取错误；根进程退出后继续刷新并等待树收口，超过 `-TreeTimeoutSeconds` 返回专用非零退出码并在 JSON 写入 `measurement_error`。stdout/stderr 重定向文件的父目录由脚本创建，启动失败也必须产出合法 JSON。

```powershell
$startInfo = [Diagnostics.ProcessStartInfo]::new()
$startInfo.FileName = $Executable
$startInfo.WorkingDirectory = (Resolve-Path $WorkingDirectory).Path
$startInfo.RedirectStandardOutput = $true
$startInfo.RedirectStandardError = $true
$startInfo.UseShellExecute = $false
foreach ($argument in $ArgumentList) { [void]$startInfo.ArgumentList.Add($argument) }
$root = [Diagnostics.Process]::Start($startInfo)
while ($true) {
  $ids = @(Get-ProcessTreeIds -RootId $root.Id)
  $current = Get-ProcessTreeWorkingSet -Ids $ids
  $peak = [Math]::Max($peak, $current.Sum)
  $samples++
  if ($root.HasExited -and (Test-ProcessTreeExited -Ids $ids)) { break }
  if ((Get-Date) - $started -gt [TimeSpan]::FromSeconds($TreeTimeoutSeconds)) { $measurementError = 'tree_timeout'; break }
  Start-Sleep -Milliseconds $SampleMilliseconds
}
$summary = [ordered]@{ command = [ordered]@{ executable = $Executable; argv = @($ArgumentList) }; exit_code = $root.ExitCode; elapsed_seconds = ...; peak_working_set_mb = ...; samples = $samples; process_count_peak = ...; stdout = ...; stderr = ...; measurement_error = $measurementError }
$summary | ConvertTo-Json -Compress
```

实现时将示例中的 `Get-DescendantProcessIds` 和 `Test-ProcessTreeExited` 写成同文件内的私有函数，处理 PID 重用和已退出进程；不能依赖 `$LASTEXITCODE` 读取 Cargo 子进程结果。

- [x] **Step 3: 运行成功、失败和真实测试测量自检**

运行上述两个短命令，再串行运行热的 `cargo test --workspace --locked --no-run`。检查三个 JSON 的 `exit_code` 分别为 0、7、0，输出文件存在，`samples` 大于 0，测试测量在根进程和子进程都退出后才结束。失败场景不得被脚本吞掉。

- [x] **Step 4: 提交测量入口**

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
- Consumes: `-Mode`（`package`、`workspace`、`probe`、`full`）、`-Package`（`package` 模式恰好一个）、`-NoRun`、`-DryRun` 和可选 `-TargetDirectory`。
- Produces: 透传 Cargo stdout/stderr，失败时返回 Cargo 的真实退出码；不改变测试线程数，不在同一调用中启动并行 Cargo 进程。

- [x] **Step 1: 固定模式到 Cargo 参数的测试表**

在 `tools/README.md` 中写出脚本的参数映射，并用 `-DryRun` 输出结构化参数检查，不实际启动 Cargo。映射必须明确如下：

```text
package   -> cargo test -p <Package> --locked [--no-run]
workspace -> cargo test --workspace --locked [--no-run]
probe     -> cargo test -p pvz-app --features probe --locked [--no-run]
full      -> cargo test --workspace --locked --no-fail-fast
```

`-Package` 为空或多于一个时，`package` 模式直接以参数错误退出；`-TargetDirectory` 不存在时创建并转为绝对路径，只设置当前进程的 `CARGO_TARGET_DIR`，不写入全局配置；`full` 忽略 `-NoRun` 并始终运行完整测试。

- [x] **Step 2: 实现参数校验和单一 Cargo 调用**

脚本使用 `[ValidateSet]` 校验模式，以数组构造 `$cargoArgs`，先设置局部环境变量，再调用 `& cargo @cargoArgs`。调用结束后立即执行 `exit $LASTEXITCODE`；不要用字符串拆分命令，不要添加 `--test-threads=1` 或未经测量的 `--jobs`。

```powershell
$cargoArgs = @('test', '--locked')
switch ($Mode) {
  'package'   { if ($Package.Count -ne 1) { throw 'package 模式必须恰好提供一个 -Package' }; $cargoArgs += @('-p', $Package[0]); if ($NoRun) { $cargoArgs += '--no-run' } }
  'workspace' { $cargoArgs += '--workspace'; if ($NoRun) { $cargoArgs += '--no-run' } }
  'probe'     { $cargoArgs += @('-p', 'pvz-app', '--features', 'probe'); if ($NoRun) { $cargoArgs += '--no-run' } }
  'full'      { $cargoArgs += @('--workspace', '--no-fail-fast') }
}
if ($TargetDirectory) { $null = New-Item -ItemType Directory -Force -Path $TargetDirectory; $env:CARGO_TARGET_DIR = (Resolve-Path $TargetDirectory).Path }
if ($DryRun) { [ordered]@{ command = 'cargo'; argv = $cargoArgs; target_directory = $env:CARGO_TARGET_DIR } | ConvertTo-Json -Compress; exit 0 }
& cargo @cargoArgs
exit $LASTEXITCODE
```

- [x] **Step 3: 运行本地反馈自检**

串行执行 `package`、`workspace -NoRun`、`probe` 和 `full` 四个模式；检查 package 模式只接受一个已知 package，probe 模式在默认 feature 下实际构建并运行测试，full 模式仍包含 doctest。用一个不存在的 package 验证非零返回，确认脚本没有将失败转换为成功。

- [x] **Step 4: 补充贡献指南中的使用边界**

在 `CONTRIBUTING.md` 的反馈层级表旁说明：局部模式只用于开发反馈，提交仍须执行完整 L0–L3；跨 package 改动使用一次 workspace 调用；覆盖率、基准、窗口和 Web 检查不通过该脚本隐式替代。记录共享 `target/` 下不要并发运行多个脚本实例。

- [x] **Step 5: 提交本地反馈入口**

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

- [x] **Step 1: 先写默认和 probe 的失败回归测试**

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

- [x] **Step 2: 运行测试确认缺少 helper 时失败**

运行：

```powershell
cargo test -p pvz-app --locked 编译期_probe_默认值与feature一致
```

预期：编译失败并指出 `apply_compile_time_options` 或 `default_options` 尚不存在；该失败只验证测试先行，不能提交。

- [x] **Step 3: 提取纯 helper 并保持主入口行为**

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

- [x] **Step 4: 在两个 feature 组合下运行回归并审计源码**

运行默认和 probe 测试：

```powershell
cargo test -p pvz-app --locked
cargo test -p pvz-app --features probe --locked
```

再执行：

```powershell
rg -n "cfg\s*!?\(.*feature|\[features\]|build\.rs|\[build-dependencies\]" crates tests benches tools Cargo.toml
cargo metadata --no-deps --locked --format-version 1
cargo test -p pvz-app --features probe --locked --no-run
```

审计结果必须记录 metadata 中的 feature 成员、全仓 `cfg(feature)`（含 tests/benches/doctest/build.rs）及依赖 feature 传播；只有确认 `pvz-app/probe` 一个项目 feature 且没有隐藏条件时，才允许消除重复的 probe Clippy。workspace all-features test 仍保留，除非另有同等测试证据并单独记录。

- [x] **Step 5: 提交 feature 回归**

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
- Produces: 保留三 OS runner、Ubuntu 条件 all-features 步骤、按角色隔离的 registry/git cache key、原有状态检查名/coverage artifact 名称和所有适用平台门槛。

- [x] **Step 1: 为 cache key 写出隔离矩阵**

在 `CONTRIBUTING.md` 记录最终 key 的组成：`${{ runner.os }}`、`${{ runner.arch }}`、`rust-toolchain.toml` 和 `Cargo.lock` 哈希以及 job 角色。只缓存 registry/git 下载；不缓存 target、覆盖率插桩对象、profraw 或 profdata。coverage default/all 分角色隔离。

```yaml
- name: 缓存 Cargo registry 与 Git checkout
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
    key: ${{ runner.os }}-${{ runner.arch }}-rust-${{ hashFiles('rust-toolchain.toml') }}-${{ hashFiles('**/Cargo.lock') }}-implementation-registry-v1
```

coverage 使用独立 `CARGO_TARGET_DIR`，运行前清理对应插桩目录；Web 使用独立绑定输出目录，避免不同组合互相覆盖。

- [x] **Step 2: 保留三 OS，并在 Ubuntu 执行 all-features 检查**

保留现有三个 OS 的 `implementation` job 和状态检查名，不直接扩成 OS × feature 六个 runner。三个 OS 都执行默认 check、fmt、workspace test、Clippy、doc；Windows 默认路径继续执行三条冒烟。仅 Ubuntu 条件执行一次 all-features check/test/Clippy/doc，以消除已证明的跨 OS 重复；若分支保护要求每个 OS 的 all-features 状态，先保持旧步骤并把缓存优化单独落地。每个 runner 内仍按顺序运行 Cargo 命令，避免共享 `target/` 争抢。

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
name: Rust 实现检查
```

最终实现保留 `cargo test --workspace --all-features --locked`，不以专门的 probe 回归替换它。Windows/macOS 远端执行默认组合，all-features 的跨 OS 重复步骤未保留，因此不能声称有这两个 OS 的远端独立 all-features 证据；本机 Windows 的 all-features 结果另行记录。

- [x] **Step 3: 消除已证明的重复 probe Clippy 调用**

Ubuntu 的 workspace `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` 已覆盖 `pvz-app/probe`，因此删除单独的 `cargo clippy -p pvz-app --features probe` 步骤；若审计或 CI 日志显示覆盖条件不等价，恢复该步骤。默认分支不运行需要 probe feature 的冒烟。

- [x] **Step 4: 给 coverage 和 Web job 加上角色隔离缓存**

coverage 的 default/all 矩阵保留独立 LCOV/JSON artifact 名称，设置独立 `CARGO_TARGET_DIR`，运行前清理插桩目录和 profraw/profdata，且不缓存插桩 target；默认基准 `cargo bench --workspace --locked --no-run` 继续只执行一次。Web job 缓存 registry/git 下载，默认与 probe 的构建仍顺序执行，绑定输出分别位于 `target/web` 和 `target/web-probe`。最终方案不缓存任何 target；registry/git key 只包含与下载缓存相关的 OS、架构、工具链、锁文件和角色，不伪称保存了可执行构建产物。

- [x] **Step 5: 静态检查并提交 CI 重排**

运行：

```powershell
actionlint .github/workflows/ci.yml
git diff --check
```

检查所有 job 仍有 structure 依赖、失败返回码、Windows timeout、coverage artifact 上传和 Web bindgen 输出；然后提交：

```powershell
git add .github/workflows/ci.yml CONTRIBUTING.md
git diff --cached --check
git commit -m "ci: 调整 feature 调度并隔离依赖缓存"
```

### Task 5: 建立同条件性能和内存证据

**Files:**
- Create: `artifacts/validation/test-optimization-2026-10-15/`（忽略的原始日志和 JSON）
- Modify: `docs/performance/README.md`（接受的测量入口和本轮结果索引）
- Modify: `docs/superpowers/specs/2026-10-15-test-performance-and-memory-optimization.md`（仅在结果改变设计结论时更新状态）
- Test: `tools/measure-cargo.ps1`、本地反馈入口、CI job 日志

**Interfaces:**
- Consumes: Task 1 的 JSON 摘要、Task 2 的反馈模式、Task 3 的 feature 结果和 Task 4 的 CI job 日志。
- Produces: 当前样本及其提交 SHA、工具链、target、feature、缓存状态、jobs、elapsed、exit code 和峰值工作集，并单列缺少的同条件前后对照；只有达到至少三次有效重复的同条件样本才汇总比较，不把一次冷样本与热样本合并。

- [x] **Step 1: 串行建立当前测量并单列缺少的前后对照**

正式的前后比较须停止并发构建、测试、基准和本项目窗口/服务，按相同 target 隔离方式执行候选场景。冷缓存清空场景 target、保留 registry/git；热缓存复用同场景 target。每组冷/热样本各取得三次有效重复，保留 stdout/stderr 和 sidecar 元数据；启动失败、超时、树未收口或退出码缺失的样本作废。首轮并未取得完整冷/热前后对照，这一收益验收条件仍未满足，不能因本步骤的测量入口和当前样本已交付而改写成性能目标已达成。

```powershell
$measure = @{
  OutputLog = 'artifacts/validation/test-optimization-2026-10-15/baseline-default-test-01'
  Executable = 'cargo.exe'
  ArgumentList = [string[]]@('test', '--workspace', '--locked', '--no-fail-fast')
  WorkingDirectory = (Get-Location).Path
}
& ./tools/measure-cargo.ps1 @measure
```

首轮完成覆盖率、默认/all-features 正确性检查和测量入口，但 2026-10-16 复核撤销了旧 benchmark/jobs 进程树样本的正式资格，原始证据保留。Task 7 使用修复后的工具重测能正常收口的场景；Windows 完整测试根 Cargo 退出 0 后仍可能遗留 MSVC `vctip.exe`，这样的进程树测量继续作废，不按进程名排除以制造成功。没有候选收益的路径不反复进行冷构建，收益验收保持未满足。

- [x] **Step 2: 验证正确性和行为不变**

对每次候选改动至少运行 `cargo fmt --all -- --check`、`cargo check --workspace --all-targets --locked`、默认 workspace 测试、all-features check/test/Clippy/doc、`cargo deny check`、受影响 Criterion 基准编译、确定性回放/事件顺序测试和适用 Windows/Web 冒烟。覆盖率 default/all 各在干净 target 中生成并核对 JSON/LCOV 源路径和分母。

- [x] **Step 3: 比较 Cargo jobs 和 cache 方案**

在相同输入下比较 Cargo 默认 jobs 与一个受控较低值的 elapsed 和峰值工作集；仅当三次有效重复的墙钟中位数不超过基线 10% 且峰值工作集下降时才保留受控 jobs，否则回退默认 jobs。对 CI 运行记录 cache hit/miss、每个 job 的步骤耗时、排队和总墙钟；不要用不同 runner 或删减检查的结果作前后结论。

2026-10-16 修复后的热 `cargo test --workspace --locked --no-run` 比较各完成三次有效重复：默认 jobs 墙钟中位数 `0.710 s`、峰值工作集中位数 `122.84 MB`；`--jobs 1` 为 `0.747 s`、`123.57 MB`。该缓存场景没有显示降低 jobs 的速度或内存优势，不代表冷构建、完整测试或 CI runner，因此保留默认调度。旧的 `.740/.712 s` 读数仅留在历史探索记录。CI registry/git cache 按 OS、架构、工具链、锁文件和角色隔离，target 与 coverage 插桩产物未缓存；缺少同条件远端 cache hit/miss 和排队时序，不能把缓存设计写成实测墙钟收益。

- [x] **Step 4: 仅在有证据时记录内存优化**

若峰值下降来自 target 任务隔离、cache 或 jobs 调整，记录它属于验证过程 OS 工作集改善，不称为游戏运行时堆内存改善。若修改夹具/解析生命周期，追加分配统计和回放/事件回归；没有可重复收益就回退该候选改动。

- [x] **Step 5: 更新性能方法和结果边界**

在 `docs/performance/README.md` 增加本轮脚本调用、冷/热条件、重复次数、进程树口径和结果索引，明确 CI/CPU/CLI 测量不等于玩家平台或原版行为验收。原始日志只放现有忽略目录，不把缓存、截图或个人路径提交到 Git。

- [x] **Step 6: 提交测量结果文档**

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

- [x] **Step 1: 运行提交前完整检查**

在仓库根目录串行执行适用命令：

```powershell
cargo fmt --all -- --check
cargo metadata --no-deps --locked --format-version 1
cargo check --workspace --all-targets --locked
cargo test --workspace --locked --no-fail-fast
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo check --workspace --all-targets --all-features --locked
cargo test --workspace --all-features --locked --no-fail-fast
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --all-features --no-deps --locked
cargo deny check
git diff --check
git diff --cached --check
```

再按验收矩阵运行 coverage、基准编译、Windows 冒烟、WASM default/probe bindgen 和 actionlint；每一项记录真实退出码和对应提交 SHA。

- [x] **Step 2: 核对差异、文件边界和证据**

检查 `git status --short`、`git diff --stat`、`git diff --name-only origin/main...HEAD`，确认没有 `target/`、凭据、原版程序、截图或临时日志进入跟踪；检查新工具的调用方、文档链接、Cargo.lock 未被无关修改。清理只确认无用途的临时文件，不删除仍被计划、CI 或验收引用的证据。

- [x] **Step 3: 请求代码复核并修复反馈**

按用户选定的执行技能完成实施中的逐任务 review；每条意见都回到对应测试或契约验证，重新运行受影响命令。任何失败、未运行或环境限制写入最终 PR 描述，不把旧主线 CI 结果借给新提交。

- [x] **Step 4: 按授权推送、创建 PR、等待 CI 并合并**

只有用户明确或持续授权覆盖外部 Git 操作时，才推送当前短期分支、创建 PR、等待所有必需 job 并合并；核对 default/all-features、coverage、Web、Windows/macOS/Linux 和依赖审计都对应新提交。没有该授权时停在本地可审阅分支，本地提交不等于推送或合并授权。

- [x] **Step 5: 核验主线并清理分支**

仅在合并已获授权且完成后更新本地 `main`，确认合入提交与远端 CI 一致，再按同一授权删除已完成的本地/远端任务分支。否则保留短期分支，最后报告本地验证、远端 CI、合并状态、缓存/内存测量、未验证平台和原版行为边界。

### 首轮执行收口快照（2026-10-15）

- 本地实现提交为 `238b08a`；随后补充的收口修复和结果文档尚未提交时，工作区只包含本计划、设计、性能记录、工具说明和测量脚本的预期改动。
- 默认与 all-features workspace 正确性检查、独立覆盖率、三次 benchmark、默认 jobs/`--jobs 1` 各三次比较、Windows 三条冒烟、默认/probe WASM 绑定、资源审计、编码检查和 actionlint 均已执行；所有纳入统计的样本真实退出码为 0，测量错误样本保留但作废。
- 当前没有同条件优化前的三次基线，也没有远端 CI 的新提交耗时、缓存命中和排队记录；因此不把本轮读数写成速度或内存收益，不改变默认 Cargo jobs、测试线程数或覆盖率分母。
- 上述测量结论随后在 Task 7 复核：旧 benchmark/jobs 进程树样本已撤销正式资格，原始记录保留，正确性检查和独立覆盖率不受采样器缺陷影响。
- 首轮 PR #5 已合并；`cef4754` 的 push 与 pull_request 两轮共 16 个检查均成功，合入提交为 `6082b2c8a1fd3e1d4f41b0c964c19687965f9034`。随后主线 `326e0f4` 的 8 个检查均成功，原任务的本地/远端分支已删除。本节是首轮交付快照，本次测量精度修复见 Task 7；后续 PR、主线和分支的实时状态以 GitHub 与最终 Git 核验为准。

### Task 7: 收口复核发现的进程身份精度回归

2026-10-16 收口复核发现：WMI `CreationDate` 以微秒表示，`.NET Process.StartTime` 保留更细精度；直接比较 ticks 会把同一个仍运行的子进程误判为 PID 已重用。本机只读检查已观察到两者相差 5 ticks。此前没有捕获该问题的测量不得继续作为可靠进程树峰值证据。

- [x] 用根进程先退出、子进程继续存活的独立夹具复现提前收口，并保留失败输出；夹具进程由测试按 PID 和创建时间清理。
- [x] 在微秒精度下统一比较创建时间；不以新快照覆盖根句柄确定的身份，清理前再次核对实际句柄身份。
- [x] 补充成功等待、树收口超时、非零退出、启动失败和根运行超时的自动回归，接入 Windows CI；不引入额外测试依赖，本地五项已通过，远端结果按最终提交核对。
- [x] 重测受影响的 benchmark 和 jobs 样本，旧样本保留但降为探索记录；三次热 benchmark 和默认 jobs/`--jobs 1` 各三次样本正常收口，结果见[性能记录](../../performance/README.md#2026-10-16-进程树复核与重测)。完整测试和首轮编译因 `vctip.exe` 未收口作废，不计收益。
- [x] 用不存在的 package 复现调用者开启原生错误偏好时，Cargo 101 被转为脚本退出 1；在脚本作用域显式处理原生退出码，补充已有/未定义 target 环境恢复和 full 模式回归，并接入三 OS CI。
- [x] 核对脚本语法、工作流、编码、文档和适用本地验收，准备同一 PR 的修复与证据说明；默认/all-features 测试、doctest、两类工具回归、actionlint 和跟踪文件/资源检查均通过。既有游戏实现未变，复用其覆盖率、平台和依赖审计记录。

Task 7 的远端交付已完成：PR #7 的 16 个检查全部成功并以 squash 合并，合并提交为 `c958d08820b6e9b33c704e7f2b558e8d577941c6`；合并后的 `main` 8 个检查全部成功，短期分支已删除。workspace 总体和 CI 远端墙钟收益仍未完成验收，不以局部测试场景的实测下降替代。

### Task 8: 测试实现路径优化与最终复核（2026-10-16）

- [x] 共享只读资源装配和版本化内容 JSON 解析；保留所有独立变体和错误路径测试。
- [x] 增加无分配的 `state_digest_matches`，逐项覆盖摘要字段；6000 次回放继续逐 tick 比较事件和状态。
- [x] 修复 `-DryRun` 不应创建 target 目录或污染调用者环境，并补充回归断言。
- [x] 在 CI 中加入过时运行取消、格式前置和按角色隔离的 registry/git 下载缓存；不缓存 target、覆盖率插桩或 profraw/profdata。
- [x] 用同一工具链、target、feature、缓存状态和 jobs 完成三次关键场景前后测量；资源测试约下降 11.7% 墙钟和 5.7% 观察工作集，`pvz-app` 完整库测试约下降 10.0% 墙钟和 4.6% 观察工作集。
- [x] 撤回没有稳定收益且会削弱断言或增加生产路径复杂度的表现层逐帧断言移除和核心旧位置按需快照；不改变规则、事件顺序和回放合同。
- [x] 记录 workspace 全量测试和核心 Criterion 没有形成稳定收益结论，保留默认 Cargo jobs、测试线程和完整正确性门槛。

### 交付收口（2026-10-16）

- [x] 本地默认/all-features 正确性、覆盖率、基准、Web、工具、资源和结构检查完成；局部速度/观察工作集结果和 workspace 证据边界已写入性能记录。
- [x] 提交 `9d2842a` 推送到 `codex/test-optimization-final`，PR #7 全部检查成功后 squash 合并为 `c958d08`。
- [x] 合并提交的 `main` CI 8/8 成功；本地 `main` 与 `origin/main` 同步，短期分支和临时 worktree 已清理。
- [x] Android/iOS、真实浏览器手势、设备音频、macOS/Linux 实机窗口和原版有效重复仍保持未验收状态。

## 计划自审

- 设计文档中的测量脚本、分层本地反馈、feature 回归、CI cache/矩阵、内存口径、回滚条件和最终验收分别由 Tasks 1–6 覆盖。
- 计划没有把旧的失败内存采样当作结果，也没有把 CI、CPU、CLI 或 Web 构建扩展为玩家平台或原版行为证据。
- `probe` 测试收窄是条件分支：审计和专门回归通过前继续运行 workspace all-features 测试；审计失败时恢复完整测试。
- 每个代码改动任务包含先写回归、运行失败、最小实现、运行通过和提交步骤；CI/脚本任务包含静态检查、退出码检查和回滚条件。
- 计划中没有未填充的占位步骤；所有路径均为仓库现有或本计划明确创建的文件。
