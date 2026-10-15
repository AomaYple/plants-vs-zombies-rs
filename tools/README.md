# 宿主工具

[`pvz-asset-tool`](pvz-asset-tool/README.md) 已实现仓库清单审计和 Reanim 概要命令，输入保持只读，不进入 Web 应用依赖图。当前没有转换或打包命令；以后按实际需求引入，可重建输出写入被忽略的 `target/`，不修改来源资源。

## Cargo 进程测量

`measure-cargo.ps1` 用逐项 argv 启动一个 Cargo 或其他宿主进程，采样根进程和后代进程的工作集，并等待整棵进程树收口。它不会把 PowerShell 的参数数组重新拼接成命令字符串，因此可用于含空格路径、引号和嵌套 `-Command` 的自检。

```powershell
& ./tools/measure-cargo.ps1 `
  -OutputLog artifacts/validation/test-optimization-2026-10-15/measure-version `
  -Executable cargo.exe `
  -ArgumentList @('--version')

$measure = @{
  OutputLog = 'artifacts/validation/test-optimization-2026-10-15/measure-failure'
  Executable = 'pwsh'
  ArgumentList = [string[]]@('-NoProfile', '-Command', 'exit 7')
}
& ./tools/measure-cargo.ps1 @measure
```

在 PowerShell 7.3 及以上版本中直接用 `& ./tools/measure-cargo.ps1 @measure` 调用。参数 splatting 保留 argv 的数组边界；不要把数组展开后跨越 `pwsh -File` 原生进程边界传入，否则 `-Command`、`-File` 等项可能被误当成测量脚本参数。

标准输出和 `<OutputLog>.json` 都保存一个紧凑 JSON 摘要；原始 stdout/stderr 分别流式写入 `<OutputLog>.out` 和 `<OutputLog>.err`，不会在测量脚本进程中无界积累。摘要至少包含 `command`、`exit_code`、`elapsed_seconds`、`peak_working_set_mb`、`samples`、`process_count_peak`、`stdout`、`stderr` 和 `measurement_error`。`exit_code` 是被测根进程真实退出码；启动失败返回 125，采样失败、输出收口失败、`-RunTimeoutSeconds` 运行超时或根进程退出后整棵已观察进程树超过 `-TreeTimeoutSeconds` 未收口时返回 124，`measurement_error` 非空，样本不得进入性能统计。脚本记录 PID 的创建时间以防止重用，并在超时或异常时按身份清理已观察进程。正式测量前应把 `-WorkingDirectory` 解析为绝对路径，并为每个场景使用独立 `CARGO_TARGET_DIR`。

Windows 的 WMI 创建时间只到微秒，句柄时间在比较前统一截断到相同精度；根进程身份以启动句柄为准，清理前再核对实际句柄。采样仍可能遗漏两个快照之间启动并退出的短命后代，工作集也包含共享页，故该值是采样口径下的进程树峰值下界，不是精确堆分配或显存。以下回归验证成功等待、树收口超时、非零退出、启动失败和根运行超时；临时夹具及证据在忽略目录中按每次运行隔离，失败时按身份清理夹具进程：

树收口超时诊断列出仍存活的进程名与 PID。Windows MSVC 的编译或 doctest 可能启动长驻的 `vctip.exe`；根 Cargo 退出 0 不能使这样的进程树样本有效，不按进程名排除它来制造成功。需要完整树峰值的场景应先取得能正常收口的工具链环境；失败样本和根 Cargo 的正确性结果分别保留。

```powershell
pwsh -NoProfile -File tools/tests/measure-cargo.ps1
```

该回归同时在 Windows implementation job 中执行，不创建可见窗口，不依赖额外测试框架。

## 分层本地测试反馈

`test-feedback.ps1` 只选择既有 Cargo 命令、透传 Cargo 输出和退出码，不改变测试线程数或隐式删减门槛。`package` 模式恰好接受一个 package；`-TargetDirectory` 不存在时会创建，并只对子进程设置当前调用的 `CARGO_TARGET_DIR`。`-DryRun` 只打印结构化参数，不启动 Cargo。

| 模式 | Cargo 参数 |
| --- | --- |
| `package -Package pvz-core` | `cargo test -p pvz-core --locked` |
| `workspace` | `cargo test --workspace --locked` |
| `probe` | `cargo test -p pvz-app --features probe --locked` |
| `full` | `cargo test --workspace --locked --no-fail-fast` |

需要只验证编译时，在前三个模式追加 `-NoRun`；`full` 始终运行完整测试并保留 doctest。快速反馈不能替代提交前的 workspace、feature、覆盖率、基准、窗口和 Web 验收；共享 target 下不要并发运行多个反馈脚本实例。

```powershell
pwsh -NoProfile -File tools/test-feedback.ps1 -Mode package -Package pvz-core -NoRun -DryRun
pwsh -NoProfile -File tools/test-feedback.ps1 -Mode workspace -NoRun -TargetDirectory artifacts/validation/test-optimization-2026-10-15/local-target
```
