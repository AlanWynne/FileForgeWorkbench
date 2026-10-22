<#
.SYNOPSIS
  Liveness probe: list the most recently written files under a path, newest
  first, with age in seconds. Build output (target\) is EXCLUDED by default so
  compiler churn does not mask whether real SOURCE/doc work is happening (this
  is the liveness-rule distinction: WORK writes, not BUILD writes, signal life).

.DESCRIPTION
  Writes the report to a log you read back (terminal stdout is unreliable on this
  machine). Based on the owner's Get-ChildItem | Sort LastWriteTime pattern.

.PARAMETER Path
  Root to scan. Default: the FileForgeWorkbench workspace.
.PARAMETER Top
  How many newest files to list. Default 25.
.PARAMETER IncludeTarget
  Include target\ build output too (off by default).
.PARAMETER Log
  Output log path. Default tools\logs\last-writes.txt under the workspace.

.EXAMPLE
  C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -File tools\powershell\last-writes.ps1 -Path C:\workspace\apps\FileForgeWorkbench\.worktrees\l10n-phase2-slice2
#>
param(
    [string]$Path = 'C:\workspace\apps\FileForgeWorkbench',
    [int]$Top = 25,
    [switch]$IncludeTarget,
    [string]$Log = 'C:\workspace\apps\FileForgeWorkbench\tools\logs\last-writes.txt'
)

$ErrorActionPreference = 'Continue'
$now = Get-Date
$logDir = Split-Path -Parent $Log
if (-not (Test-Path $logDir)) { New-Item -ItemType Directory -Path $logDir -Force | Out-Null }

$files = Get-ChildItem -Path $Path -File -Recurse -ErrorAction SilentlyContinue
if (-not $IncludeTarget) {
    $files = $files | Where-Object { $_.FullName -notmatch '\\target\\' -and $_.FullName -notmatch '\\\.git\\' }
}

$ranked = $files | Sort-Object LastWriteTime -Descending | Select-Object -First $Top

$out = New-Object System.Collections.Generic.List[string]
$out.Add("=== last-writes probe @ " + $now.ToString('yyyy-MM-dd HH:mm:ss') + " ===")
$out.Add("scan root : $Path")
$out.Add("exclude   : " + $(if ($IncludeTarget) { 'none' } else { 'target\, .git\' }))
$out.Add("")
$out.Add(("{0,8}  {1,10}  {2}" -f 'AGE(s)', 'BYTES', 'FILE'))
$out.Add(("{0,8}  {1,10}  {2}" -f '------', '-----', '----'))
foreach ($f in $ranked) {
    $age = [int]($now - $f.LastWriteTime).TotalSeconds
    $rel = $f.FullName
    if ($rel.StartsWith($Path)) { $rel = $rel.Substring($Path.Length).TrimStart('\') }
    $out.Add(("{0,8}  {1,10}  {2}" -f $age, $f.Length, $rel))
}

# simple verdict from the freshest non-target write
if ($ranked) {
    $newest = [int]($now - ($ranked[0].LastWriteTime)).TotalSeconds
    $verdict = if ($newest -le 120) { 'ALIVE' } elseif ($newest -le 300) { 'SLOWING' } else { 'STALLED' }
    $out.Add("")
    $out.Add("newest WORK write: ${newest}s ago  ->  $verdict  (ALIVE<=120, SLOWING<=300, else STALLED)")
} else {
    $out.Add("")
    $out.Add("no non-target files found under the scan root")
}
$out.Add("=== end ===")

Set-Content -Path $Log -Value $out -Encoding utf8
