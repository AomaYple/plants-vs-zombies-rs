#requires -Version 7.3
[CmdletBinding()]
param([string]$OutputDirectory = 'artifacts/validation/test-feedback-tests')

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$feedbackScript = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../test-feedback.ps1'))
$runDirectory = Join-Path ([IO.Path]::GetFullPath($OutputDirectory)) ([guid]::NewGuid().ToString('N'))
$null = [IO.Directory]::CreateDirectory($runDirectory)
$previousTarget = $env:CARGO_TARGET_DIR
$targetWasDefined = Test-Path Env:CARGO_TARGET_DIR
$previousNativePreference = $PSNativeCommandUseErrorActionPreference

try {
    # Cargo 参数错误固定返回 101；调用者启用原生错误偏好时也必须保留该退出码。
    $PSNativeCommandUseErrorActionPreference = $true
    $originalTarget = Join-Path $runDirectory 'original target'
    $env:CARGO_TARGET_DIR = $originalTarget
    & $feedbackScript -Mode package -Package 'pvz-feedback-missing-package-a84dce24' `
        -TargetDirectory (Join-Path $runDirectory 'failure target') *> (Join-Path $runDirectory 'failure.log')
    if ($LASTEXITCODE -ne 101) { throw '测试反馈回归失败：没有保留 Cargo 的真实退出码 101' }
    if ($env:CARGO_TARGET_DIR -ne $originalTarget) { throw '测试反馈回归失败：失败后未恢复已有 target 环境变量' }
    if (-not $PSNativeCommandUseErrorActionPreference) { throw '测试反馈回归失败：改变了调用者的原生错误偏好' }

    Remove-Item Env:CARGO_TARGET_DIR
    $dryRunTarget = Join-Path $runDirectory 'dry run target'
    $dryRun = & $feedbackScript -Mode full -NoRun -DryRun `
        -TargetDirectory $dryRunTarget | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0 -or $dryRun.argv -contains '--no-run' -or
        $dryRun.argv -notcontains '--no-fail-fast' -or $dryRun.argv -notcontains '--workspace' -or
        $dryRun.target_directory -ne [IO.Path]::GetFullPath($dryRunTarget) -or
        (Test-Path -LiteralPath $dryRunTarget)) {
        throw '测试反馈回归失败：full 模式必须保留完整 workspace 测试'
    }
    if (Test-Path Env:CARGO_TARGET_DIR) { throw '测试反馈回归失败：没有恢复原先未定义的 target 环境变量' }
} finally {
    $PSNativeCommandUseErrorActionPreference = $previousNativePreference
    if ($targetWasDefined) { $env:CARGO_TARGET_DIR = $previousTarget }
    else { Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue }
}
"测试反馈退出码与环境隔离回归通过；证据目录：$runDirectory"
exit 0
