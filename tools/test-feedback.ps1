[CmdletBinding()]
param(
    [ValidateSet('package', 'workspace', 'probe', 'full')]
    [string]$Mode = 'workspace',

    [string[]]$Package,

    [switch]$NoRun,

    [switch]$DryRun,

    [string]$TargetDirectory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$previousTargetDirectory = $env:CARGO_TARGET_DIR
$targetDirectoryWasDefined = Test-Path Env:CARGO_TARGET_DIR
$targetDirectoryChanged = $false

$cargoArgs = [Collections.Generic.List[string]]::new()
[void]$cargoArgs.Add('test')
[void]$cargoArgs.Add('--locked')

switch ($Mode) {
    'package' {
        $packages = @($Package | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        if ($packages.Count -ne 1) {
            throw 'package 模式必须恰好提供一个 -Package'
        }
        [void]$cargoArgs.Add('-p')
        [void]$cargoArgs.Add($packages[0])
        if ($NoRun) { [void]$cargoArgs.Add('--no-run') }
    }
    'workspace' {
        [void]$cargoArgs.Add('--workspace')
        if ($NoRun) { [void]$cargoArgs.Add('--no-run') }
    }
    'probe' {
        [void]$cargoArgs.Add('-p')
        [void]$cargoArgs.Add('pvz-app')
        [void]$cargoArgs.Add('--features')
        [void]$cargoArgs.Add('probe')
        if ($NoRun) { [void]$cargoArgs.Add('--no-run') }
    }
    'full' {
        [void]$cargoArgs.Add('--workspace')
        [void]$cargoArgs.Add('--no-fail-fast')
    }
}

$result = $null
$cargoExitCode = 0
$dryRunResult = $false
try {
    if ($TargetDirectory) {
        $null = New-Item -ItemType Directory -Force -Path $TargetDirectory
        $env:CARGO_TARGET_DIR = (Resolve-Path -LiteralPath $TargetDirectory).Path
        $targetDirectoryChanged = $true
    }

    $result = [ordered]@{
        command = 'cargo'
        argv = @($cargoArgs)
        target_directory = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { $null }
    }
    if ($DryRun) {
        $result | ConvertTo-Json -Compress -Depth 4
        $dryRunResult = $true
    } else {
        & cargo @cargoArgs
        $cargoExitCode = $LASTEXITCODE
    }
} finally {
    if ($targetDirectoryChanged) {
        if ($targetDirectoryWasDefined) { $env:CARGO_TARGET_DIR = $previousTargetDirectory }
        else { Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue }
    }
}

if ($dryRunResult) { exit 0 }
exit $cargoExitCode
