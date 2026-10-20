<#
.SYNOPSIS
  Delete spent one-off Python patch/fix/split/read scripts from tools/python,
  keeping only the documented reusable tools.

.DESCRIPTION
  One command, one job (per .kiro/steering/tooling.md). The reusable keepers are
  listed in $Keep; every other *.py in tools/python plus __pycache__ is removed.
  Writes the deleted-file manifest to tools/logs/prune-oneoff-scripts.txt.

  Repo root is derived from this script's own location (two parents up), so the
  tool is portable across machines and relocations.

.NOTES
  Reusable project tool. Destructive by design but scoped to tools/python/*.py
  minus the keep-list; recoverable via git for any tracked file.
#>
param(
    [switch]$WhatIfOnly
)

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$pyDir = Join-Path $repoRoot 'tools\python'
$logDir = Join-Path $repoRoot 'tools\logs'
$log = Join-Path $logDir 'prune-oneoff-scripts.txt'

if (-not (Test-Path $logDir)) { New-Item -ItemType Directory -Path $logDir -Force | Out-Null }

$keep = @(
    'logging_inventory.py',
    'measure_crate_lines.py',
    'pwsh_command_guard.py',
    'pwsh_command_guard_selftest.py'
)

$targets = Get-ChildItem -Path $pyDir -Filter *.py -File |
    Where-Object { $keep -notcontains $_.Name }

$names = $targets | ForEach-Object { $_.Name }

if ($WhatIfOnly) {
    $header = "WHATIF: would delete $($names.Count) files (keeping $($keep.Count))"
    Set-Content -Path $log -Value (@($header) + $names) -Encoding utf8
    Write-Output $header
    return
}

$targets | Remove-Item -Force
$cache = Join-Path $pyDir '__pycache__'
if (Test-Path $cache) { Remove-Item -Path $cache -Recurse -Force }

$header = "deleted $($names.Count) one-off scripts (kept $($keep.Count) reusable)"
Set-Content -Path $log -Value (@($header) + $names) -Encoding utf8
Write-Output $header
