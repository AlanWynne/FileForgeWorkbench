<#
.SYNOPSIS
  ONCE-OFF: relocate git-ignored artefacts into the _ignore/ folder.

.DESCRIPTION
  A deliberate, one-time migration tool. DRY-RUN BY DEFAULT: with no -Apply it
  only PRINTS what it would move and changes nothing. Pass -Apply to perform the
  moves. This is NOT a session-start action -- run it by hand, review the dry run,
  then re-run with -Apply.

  What it moves (each into a mirrored path under _ignore/):
    - tools/logs/*        -> _ignore/logs/            (the primary relocation)
    - bootstrap/logs/*.log-> _ignore/bootstrap-logs/
    - reports/*           -> _ignore/reports/
    - loose *.log / *.tmp in the repo ROOT -> _ignore/scratch/
    - FailedTest.txt, test_output.txt, Tests.txt (root) -> _ignore/scratch/

  What it deliberately does NOT touch (left where tooling expects them):
    - target/      (cargo output; recreated every build; huge; leave in place)
    - .gate/       (the canonical gate writes here by convention)
    - .worktrees/  (parallel git worktrees)
    - .ffworkbench/logs/ (dev app logs, redirected by config)
    - anything git is TRACKING (only already-ignored artefacts are candidates)

  SAFETY:
    - Dry-run default; -Apply required to move.
    - Uses git check-ignore so it ONLY moves files git already ignores -- a
      tracked file is never relocated.
    - Never deletes; a name collision in the destination is reported and SKIPPED,
      not overwritten.
    - Writes a log to _ignore/logs/sweep-to-ignore.txt and echoes it.

.PARAMETER RepoRoot
  Absolute path to the project root. Defaults to the parent of tools/.

.PARAMETER Apply
  Perform the moves. Omit for a dry run (report only).
#>
[CmdletBinding()]
param(
    [string]$RepoRoot,
    [switch]$Apply
)

$ErrorActionPreference = 'Stop'

if (-not $RepoRoot -or $RepoRoot.Trim() -eq '') {
    $RepoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
}

$ignoreDir = Join-Path $RepoRoot '_ignore'
$logDir    = Join-Path $ignoreDir 'logs'
if (-not (Test-Path -LiteralPath $logDir)) {
    New-Item -ItemType Directory -Path $logDir -Force | Out-Null
}

$mode = if ($Apply) { 'APPLY' } else { 'DRY-RUN' }
$log  = New-Object System.Collections.Generic.List[string]
function Note([string]$m) { $log.Add($m) }

Note("sweep-to-ignore [$mode]: repo = $RepoRoot")
Note("")

# Is a path git-ignored? Only ignored paths are move candidates.
function Test-GitIgnored([string]$path) {
    Push-Location $RepoRoot
    try {
        & git check-ignore -q -- "$path" 2>$null
        return ($LASTEXITCODE -eq 0)
    } finally {
        Pop-Location
    }
}

# Plan one source-dir -> dest-dir relocation of its direct children.
function Plan-Dir([string]$srcRel, [string]$destSub, [string]$filter) {
    $src = Join-Path $RepoRoot $srcRel
    if (-not (Test-Path -LiteralPath $src)) { Note("skip ${srcRel}: not present"); return }
    $dest = Join-Path $ignoreDir $destSub
    $items = if ($filter) {
        Get-ChildItem -LiteralPath $src -Filter $filter -File -ErrorAction SilentlyContinue
    } else {
        Get-ChildItem -LiteralPath $src -ErrorAction SilentlyContinue
    }
    if (-not $items -or $items.Count -eq 0) { Note("skip ${srcRel}: empty"); return }
    if ($Apply -and -not (Test-Path -LiteralPath $dest)) {
        New-Item -ItemType Directory -Path $dest -Force | Out-Null
    }
    foreach ($it in $items) {
        if ($it.Name -eq '.gitkeep') { continue }
        $target = Join-Path $dest $it.Name
        if (Test-Path -LiteralPath $target) {
            Note("SKIP (collision) $srcRel/$($it.Name) -> $destSub/$($it.Name)")
            continue
        }
        if ($Apply) {
            Move-Item -LiteralPath $it.FullName -Destination $target -Force
            Note("MOVED $srcRel/$($it.Name) -> $destSub/$($it.Name)")
        } else {
            Note("would move $srcRel/$($it.Name) -> $destSub/$($it.Name)")
        }
    }
}

# Plan loose root-level files matching a glob, only if git-ignored.
function Plan-RootGlob([string]$glob, [string]$destSub) {
    $items = Get-ChildItem -LiteralPath $RepoRoot -Filter $glob -File -ErrorAction SilentlyContinue
    if (-not $items) { return }
    $dest = Join-Path $ignoreDir $destSub
    if ($Apply -and -not (Test-Path -LiteralPath $dest)) {
        New-Item -ItemType Directory -Path $dest -Force | Out-Null
    }
    foreach ($it in $items) {
        if (-not (Test-GitIgnored $it.FullName)) {
            Note("keep (tracked) $($it.Name)"); continue
        }
        $target = Join-Path $dest $it.Name
        if (Test-Path -LiteralPath $target) { Note("SKIP (collision) $($it.Name)"); continue }
        if ($Apply) {
            Move-Item -LiteralPath $it.FullName -Destination $target -Force
            Note("MOVED $($it.Name) -> $destSub/$($it.Name)")
        } else {
            Note("would move $($it.Name) -> $destSub/$($it.Name)")
        }
    }
}

# The primary relocation and the secondary ones.
Plan-Dir 'tools/logs'     'logs'           $null
Plan-Dir 'bootstrap/logs' 'bootstrap-logs' '*.log'
Plan-Dir 'reports'        'reports'        $null
Plan-RootGlob '*.log' 'scratch'
Plan-RootGlob '*.tmp' 'scratch'
foreach ($n in @('FailedTest.txt','test_output.txt','Tests.txt')) {
    $p = Join-Path $RepoRoot $n
    if ((Test-Path -LiteralPath $p) -and (Test-GitIgnored $p)) {
        $dest = Join-Path $ignoreDir 'scratch'
        if ($Apply -and -not (Test-Path -LiteralPath $dest)) { New-Item -ItemType Directory -Path $dest -Force | Out-Null }
        $target = Join-Path $dest $n
        if (Test-Path -LiteralPath $target) { Note("SKIP (collision) $n") }
        elseif ($Apply) { Move-Item -LiteralPath $p -Destination $target -Force; Note("MOVED $n -> scratch/$n") }
        else { Note("would move $n -> scratch/$n") }
    }
}

Note("")
if (-not $Apply) {
    Note("DRY RUN complete -- nothing moved. Re-run with -Apply to perform the moves.")
} else {
    Note("APPLY complete.")
    Note("REMINDER: update steering/tooling references from 'tools/logs/' to '_ignore/logs/' so scripts write to the new location (otherwise tools/logs/ is recreated).")
}

$logPath = Join-Path $logDir 'sweep-to-ignore.txt'
$log | Set-Content -LiteralPath $logPath -Encoding utf8
$log | ForEach-Object { Write-Output $_ }
exit 0
