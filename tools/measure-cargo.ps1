#requires -Version 7.3
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$OutputLog,
    [Parameter(Mandatory)][string]$Executable,
    [string[]]$ArgumentList = @(),
    [string]$WorkingDirectory = (Get-Location).Path,
    [ValidateRange(10, 60000)][int]$SampleMilliseconds = 100,
    [ValidateRange(1, 86400)][int]$TreeTimeoutSeconds = 30,
    [ValidateRange(1, 86400)][int]$RunTimeoutSeconds = 7200
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$watch = [Diagnostics.Stopwatch]::StartNew()
$root = $null
$exitCode = $null
$measurementErrors = [Collections.Generic.List[string]]::new()
$tracked = @{}
$lastSnapshot = @{}
$outputTasks = @()
$outputStreams = @()
$peak = [int64]0
$samples = 0
$countPeak = 0
$drainStarted = $null
$startedUtc = [datetime]::UtcNow
$stdoutPath = [IO.Path]::GetFullPath("$OutputLog.out")
$stderrPath = [IO.Path]::GetFullPath("$OutputLog.err")

function Get-Snapshot {
    # 查询失败必须上抛，不能静默把整棵树缩成根进程。
    @(Get-CimInstance Win32_Process -Property ProcessId, ParentProcessId, CreationDate, WorkingSetSize)
}

function Update-Tree {
    param([object[]]$Snapshot)
    $current = @{}
    foreach ($item in $Snapshot) { $current[[int]$item.ProcessId] = $item }
    $script:lastSnapshot = $current
    if ($current.ContainsKey($root.Id) -and $tracked.ContainsKey($root.Id)) {
        $tracked[$root.Id].Created = ([datetime]$current[$root.Id].CreationDate).ToUniversalTime()
    }
    $changed = $true
    while ($changed) {
        $changed = $false
        foreach ($item in $Snapshot) {
            $processId = [int]$item.ProcessId
            $parentId = [int]$item.ParentProcessId
            $created = ([datetime]$item.CreationDate).ToUniversalTime()
            if (-not $tracked.ContainsKey($processId) -and $tracked.ContainsKey($parentId)) {
                $parent = $tracked[$parentId]
                # 只接受父进程本次生命周期内出生的成员，防止 PID 重用。
                $parentMatches = -not $current.ContainsKey($parentId) -or
                    ([datetime]$current[$parentId].CreationDate).ToUniversalTime().Ticks -eq $parent.Created.Ticks
                if ($parentMatches -and $created -ge $parent.Created -and -not $parent.Exited) {
                    $tracked[$processId] = [pscustomobject]@{ Created = $created; Exited = $false; WorkingSet = [int64]0 }
                    $changed = $true
                }
            }
        }
    }
    foreach ($processId in @($tracked.Keys)) {
        $record = $tracked[$processId]
        if ($current.ContainsKey($processId) -and
            ([datetime]$current[$processId].CreationDate).ToUniversalTime().Ticks -eq $record.Created.Ticks) {
            $record.WorkingSet = [int64]$current[$processId].WorkingSetSize
        } else {
            $record.Exited = $true
            $record.WorkingSet = [int64]0
        }
    }
}

function Stop-Tree {
    # 停止前重新核对出生时间；绝不终止复用了同一 PID 的无关进程。
    foreach ($processId in @($tracked.Keys)) {
        $process = $null
        try {
            if (-not $lastSnapshot.ContainsKey($processId)) { continue }
            if (([datetime]$lastSnapshot[$processId].CreationDate).ToUniversalTime().Ticks -ne $tracked[$processId].Created.Ticks) { continue }
            $process = [Diagnostics.Process]::GetProcessById($processId)
            if (-not $process.HasExited) {
                $process.Kill($true)
                [void]$process.WaitForExit(5000)
            }
        } catch [ArgumentException] {
            # 成员已退出。
        } catch {
            [void]$measurementErrors.Add("清理进程失败：$processId：$($_.Exception.Message)")
        } finally {
            if ($process) { $process.Dispose() }
        }
    }
    if ($root -and -not $root.HasExited) {
        $root.Kill($true)
        [void]$root.WaitForExit(5000)
    }
}

function Confirm-TrackedExit {
    # WMI 进程快照可能在根进程退出后短暂滞后；用句柄再次确认，避免把已经退出的成员判成树未收口。
    foreach ($processId in @($tracked.Keys)) {
        $record = $tracked[$processId]
        if ($record.Exited) { continue }
        $process = $null
        try {
            $process = [Diagnostics.Process]::GetProcessById($processId)
            $process.Refresh()
            if ($process.StartTime.ToUniversalTime().Ticks -ne $record.Created.Ticks -or $process.HasExited) {
                $record.Exited = $true
                $record.WorkingSet = [int64]0
            }
        } catch [ArgumentException] {
            $record.Exited = $true
            $record.WorkingSet = [int64]0
        } catch [InvalidOperationException] {
            # 进程在刷新期间退出。
            $record.Exited = $true
            $record.WorkingSet = [int64]0
        } catch {
            [void]$measurementErrors.Add("确认进程退出失败：$processId：$($_.Exception.Message)")
        } finally {
            if ($process) { $process.Dispose() }
        }
    }
}

try {
    if (-not $IsWindows) { throw '进程树工作集采样目前只支持 Windows' }
    $directory = (Resolve-Path -LiteralPath $WorkingDirectory).Path
    $null = [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($stdoutPath))
    $startInfo = [Diagnostics.ProcessStartInfo]::new()
    $startInfo.FileName = $Executable
    $startInfo.WorkingDirectory = $directory
    $startInfo.UseShellExecute = $false
    $startInfo.CreateNoWindow = $true
    $startInfo.RedirectStandardOutput = $true
    $startInfo.RedirectStandardError = $true
    foreach ($argument in $ArgumentList) { [void]$startInfo.ArgumentList.Add($argument) }
    $outputStreams = @(
        [IO.File]::Open($stdoutPath, [IO.FileMode]::Create, [IO.FileAccess]::Write, [IO.FileShare]::Read),
        [IO.File]::Open($stderrPath, [IO.FileMode]::Create, [IO.FileAccess]::Write, [IO.FileShare]::Read)
    )
    $root = [Diagnostics.Process]::Start($startInfo)
    $outputTasks = @(
        $root.StandardOutput.BaseStream.CopyToAsync($outputStreams[0]),
        $root.StandardError.BaseStream.CopyToAsync($outputStreams[1])
    )
    $tracked[$root.Id] = [pscustomobject]@{
        Created = $root.StartTime.ToUniversalTime(); Exited = $false; WorkingSet = [int64]0
    }
    try {
        $root.Refresh()
        $peak = [int64]$root.WorkingSet64
        $countPeak = 1
    } catch {
        [void]$measurementErrors.Add("初始工作集不可读：$($_.Exception.Message)")
    }
    while ($true) {
        Update-Tree -Snapshot @(Get-Snapshot)
        if ($root.HasExited) { Confirm-TrackedExit }
        $usage = [int64]0
        $active = 0
        foreach ($record in $tracked.Values) {
            if (-not $record.Exited) { $usage += $record.WorkingSet; $active++ }
        }
        $samples++
        $peak = [Math]::Max($peak, $usage)
        $countPeak = [Math]::Max($countPeak, $active)
        if ($root.HasExited) {
            $exitCode = $root.ExitCode
            if ($null -eq $drainStarted) { $drainStarted = $watch.Elapsed.TotalSeconds }
            if ($active -eq 0) { break }
            if ($watch.Elapsed.TotalSeconds - $drainStarted -ge $TreeTimeoutSeconds) {
                throw '根进程退出后后代进程收口超时'
            }
        }
        if ($watch.Elapsed.TotalSeconds -ge $RunTimeoutSeconds) { throw '被测命令运行超时' }
        Start-Sleep -Milliseconds $SampleMilliseconds
    }
    # 短命进程可能在首次快照前退出；根句柄保留的峰值仅是采样下界。
    $peak = [Math]::Max($peak, $root.PeakWorkingSet64)
    $countPeak = [Math]::Max($countPeak, 1)
    $allOutput = [Threading.Tasks.Task]::WhenAll([Threading.Tasks.Task[]]$outputTasks)
    if (-not $allOutput.Wait(5000)) { throw '输出流收口超时' }
} catch {
    [void]$measurementErrors.Add($_.Exception.Message)
    try { Stop-Tree } catch { [void]$measurementErrors.Add("清理失败：$($_.Exception.Message)") }
} finally {
    if ($measurementErrors.Count -gt 0 -and $root) {
        # 关闭管道解除 CopyToAsync 等待，保留已经写入的原始字节。
        $root.StandardOutput.Dispose()
        $root.StandardError.Dispose()
    }
    foreach ($task in $outputTasks) {
        try { [void]$task.Wait(5000) } catch { [void]$measurementErrors.Add("输出未完整保存：$($_.Exception.Message)") }
    }
    foreach ($stream in $outputStreams) { $stream.Dispose() }
    if ($root) {
        if ($root.HasExited) { $exitCode = $root.ExitCode }
        $root.Dispose()
    }
}
$watch.Stop()
$summary = [ordered]@{
    command = [ordered]@{ executable = $Executable; argv = @($ArgumentList) }
    exit_code = $exitCode
    elapsed_seconds = [Math]::Round($watch.Elapsed.TotalSeconds, 3)
    peak_working_set_mb = [Math]::Round($peak / 1MB, 2)
    samples = $samples
    process_count_peak = $countPeak
    started_utc = $startedUtc.ToString('o')
    sample_milliseconds = $SampleMilliseconds
    run_timeout_seconds = $RunTimeoutSeconds
    tree_timeout_seconds = $TreeTimeoutSeconds
    stdout = $stdoutPath
    stderr = $stderrPath
    measurement_error = if ($measurementErrors.Count) { $measurementErrors -join '; ' } else { $null }
}
$json = $summary | ConvertTo-Json -Compress -Depth 5
[IO.File]::WriteAllText("$([IO.Path]::GetFullPath($OutputLog)).json", "$json`n", [Text.UTF8Encoding]::new($false))
$json
if ($measurementErrors.Count) { if ($null -eq $root) { exit 125 }; exit 124 }
if ($null -eq $exitCode) { exit 125 }
exit $exitCode
