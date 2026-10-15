#requires -Version 7.3
[CmdletBinding()]
param(
    [ValidateSet('suite', 'parent', 'child')][string]$Mode = 'suite',
    [string]$OutputDirectory = 'artifacts/validation/measure-cargo-tests',
    [string]$MarkerPath,
    [string]$ChildRecord,
    [int]$DurationMilliseconds = 4000
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw '进程树测量回归目前只支持 Windows' }
$pwsh = (Get-Process -Id $PID).Path

if ($Mode -eq 'child') {
    Start-Sleep -Milliseconds $DurationMilliseconds
    [IO.File]::WriteAllText($MarkerPath, "子进程已完成`n")
    exit 0
}

if ($Mode -eq 'parent') {
    # 定向构造 WMI 微秒时间与句柄 ticks 不完全相等的进程，避免靠概率触发回归。
    for ($attempt = 0; $attempt -lt 10; $attempt++) {
        $info = [Diagnostics.ProcessStartInfo]::new()
        $info.FileName = $pwsh
        $info.UseShellExecute = $false
        $info.CreateNoWindow = $true
        $info.RedirectStandardOutput = $true
        $info.RedirectStandardError = $true
        foreach ($argument in @('-NoProfile', '-File', $PSCommandPath, '-Mode', 'child',
                '-MarkerPath', $MarkerPath, '-DurationMilliseconds', "$DurationMilliseconds")) {
            [void]$info.ArgumentList.Add($argument)
        }
        $child = [Diagnostics.Process]::Start($info)
        $created = $child.StartTime.ToUniversalTime().Ticks
        if ($created % 10 -ne 0) { break }
        $child.Kill($true)
        [void]$child.WaitForExit(5000)
        $child.Dispose()
        $child = $null
    }
    if (-not $child) { throw '未能生成带亚微秒精度的回归夹具' }
    $record = @{ pid = $child.Id; creation_ticks = $created } | ConvertTo-Json -Compress
    [IO.File]::WriteAllText($ChildRecord, "$record`n")
    # 让采样器观察到父子关系；随后父进程退出，子进程继续运行。
    Start-Sleep -Milliseconds 1000
    $child.Dispose()
    exit 0
}

$measurementScript = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../measure-cargo.ps1'))
$runDirectory = Join-Path ([IO.Path]::GetFullPath($OutputDirectory)) ([guid]::NewGuid().ToString('N'))
$null = [IO.Directory]::CreateDirectory($runDirectory)

function Invoke-Measurement {
    param([string]$Name, [string]$Executable, [string[]]$Arguments,
        [int]$TreeTimeout = 10, [int]$RunTimeout = 30)
    $parameters = @{
        OutputLog = Join-Path $runDirectory $Name
        Executable = $Executable
        ArgumentList = $Arguments
        TreeTimeoutSeconds = $TreeTimeout
        RunTimeoutSeconds = $RunTimeout
    }
    $output = & $measurementScript @parameters
    $wrapperExitCode = $LASTEXITCODE
    $summary = ($output -join "`n") | ConvertFrom-Json
    [pscustomobject]@{ WrapperExitCode = $wrapperExitCode; Summary = $summary }
}

function Get-OwnedChild {
    param([string]$RecordPath)
    if (-not (Test-Path -LiteralPath $RecordPath)) { return $null }
    $record = Get-Content -LiteralPath $RecordPath -Raw | ConvertFrom-Json
    $process = $null
    try {
        $process = [Diagnostics.Process]::GetProcessById($record.pid)
        if (-not $process.HasExited -and $process.StartTime.ToUniversalTime().Ticks -eq $record.creation_ticks) {
            return $process
        }
    } catch [ArgumentException] {
        # 夹具进程已退出。
    } catch [InvalidOperationException] {
        # 夹具在查询期间退出。
    }
    if ($process) { $process.Dispose() }
    return $null
}

foreach ($case in @('wait', 'timeout')) {
    $marker = Join-Path $runDirectory "$case.done"
    $recordPath = Join-Path $runDirectory "$case.child.json"
    $duration = if ($case -eq 'wait') { 4000 } else { 30000 }
    $treeTimeout = if ($case -eq 'wait') { 10 } else { 1 }
    try {
        $result = Invoke-Measurement -Name $case -Executable $pwsh -TreeTimeout $treeTimeout -Arguments @(
            '-NoProfile', '-File', $PSCommandPath, '-Mode', 'parent', '-MarkerPath', $marker,
            '-ChildRecord', $recordPath, '-DurationMilliseconds', "$duration"
        )
        if ($case -eq 'wait') {
            if ($result.WrapperExitCode -ne 0 -or $result.Summary.exit_code -ne 0 -or
                $result.Summary.measurement_error -or -not (Test-Path -LiteralPath $marker)) {
                throw '等待回归失败：测量必须在子进程完成之后才成功收口'
            }
            if ($result.Summary.process_count_peak -lt 2 -or $result.Summary.samples -lt 1) {
                throw '等待回归失败：没有观察到父子进程树'
            }
        } elseif ($result.WrapperExitCode -ne 124 -or
            $result.Summary.measurement_error -notmatch '后代进程收口超时') {
            throw '超时回归失败：仍存活的子进程必须触发非零测量错误'
        }
        $remaining = Get-OwnedChild -RecordPath $recordPath
        if ($remaining) {
            $remaining.Dispose()
            throw '进程清理回归失败：测量结束后夹具子进程仍在运行'
        }
        "进程树回归通过：$case"
    } finally {
        # 即使旧实现导致提前成功或断言失败，也只清理本次夹具的实际进程。
        $remaining = Get-OwnedChild -RecordPath $recordPath
        if ($remaining) {
            $remaining.Kill($true)
            [void]$remaining.WaitForExit(5000)
            $remaining.Dispose()
        }
    }
}

$failure = Invoke-Measurement -Name 'exit-seven' -Executable $pwsh -Arguments @('-NoProfile', '-Command', 'exit 7')
if ($failure.WrapperExitCode -ne 7 -or $failure.Summary.exit_code -ne 7 -or $failure.Summary.measurement_error) {
    throw '退出码回归失败：被测进程的退出码 7 必须原样保留'
}
$missing = Invoke-Measurement -Name 'missing' -Executable (Join-Path $runDirectory 'missing.exe') -Arguments @()
if ($missing.WrapperExitCode -ne 125 -or $null -ne $missing.Summary.exit_code -or -not $missing.Summary.measurement_error) {
    throw '启动失败回归失败：必须报告非零退出和测量错误'
}
$runTimeout = Invoke-Measurement -Name 'run-timeout' -Executable $pwsh -RunTimeout 1 -Arguments @(
    '-NoProfile', '-Command', 'Start-Sleep -Seconds 30'
)
if ($runTimeout.WrapperExitCode -ne 124 -or $runTimeout.Summary.measurement_error -notmatch '被测命令运行超时') {
    throw '运行超时回归失败：根进程未结束时必须按运行期限报错'
}
"全部五项测量回归通过；证据目录：$runDirectory"
exit 0
