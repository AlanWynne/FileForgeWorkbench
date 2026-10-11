<#
.SYNOPSIS
    Liveness probe + scoped-check helper -- answers "is a background task still
    working or has it stalled?" and prints a one-word VERDICT, and also runs the
    scoped cargo checks that produce that evidence. The SINGLE tool for both.

.DESCRIPTION
    This script is the merge of the proven Walrus liveness probe with this
    project's additions. It absorbs the former tools/powershell/ff-check.ps1
    (now removed): its check / test / clippy / fmt tasks live here, so there is
    ONE liveness + scoped-check path, not two overlapping ones.

    EVERY task writes its full output to a log under <root>\tools\logs\
    (git-ignored, ephemeral) and THEN echoes that same log back to the terminal,
    so the caller always has a durable copy even when the interactive terminal
    mangles stdout (see .kiro/steering/tooling.md).

    Tasks (-Task, default probe):

      probe   -- the liveness VERDICT. It buckets recent writes so signal is not
                 drowned by build noise:
                   WORK  -- source / docs / tasks / configs (real work product)
                   GATE  -- .gate\ workflow logs (red/green/review/build traces)
                   BUILD -- target\ compiler output (churns constantly; shown so
                            a build-in-progress is visible, but NOT a liveness
                            signal on its own)
                 The liveness signal is the FRESHER of newest-WORK, newest-GATE,
                 and (when -RunLabel is given) the heartbeat log's last write.
                 Verdict from that signal's age vs -StaleSeconds:
                   ALIVE   -- within 1x the threshold (actively writing)
                   SLOWING -- 1x..2x the threshold (quiet; may be mid build/review
                              -- re-check before calling it, do NOT abort yet)
                   STALLED -- beyond 2x, or nothing written at all
                 The verdict is ALSO the EXIT CODE (0 ALIVE / 3 SLOWING /
                 4 STALLED), so the probe can gate a script, not just inform a
                 human. It also reads the latest git commit so the common
                 "work done but not committed yet" stall shape is obvious.

      status  -- the recent-write bucket snapshot only (no verdict, exit 0).
      check   -- scoped cargo check -p ff-idcams -p ff-desktop -p ff-dscatalog
                 -p ff-vfs -p ff-volume --all-targets.
      test    -- scoped cargo test -p ff-idcams; cargo test -p ff-desktop.
      clippy  -- cargo clippy -p ff-idcams -p ff-desktop -- -D warnings.
      fmt     -- cargo fmt (writes), result note.

    PORTABLE: the repo root is derived from this script's own location
    (tools/powershell/liveness-probe.ps1 -> repo root is three parents up), so
    the script runs unchanged on any machine and from any worktree. No absolute
    paths are baked in. The cargo tasks always run against the derived repo root;
    the probe/status tasks scan -Path (default: the repo root).

.PARAMETER Task
    Which job to run. Default: probe.

.PARAMETER Path
    Root to scan for probe/status. Defaults to the repo root derived from the
    script location. Point it at a worktree to scan that worktree.

.PARAMETER Top
    How many recent files to list PER BUCKET. Default 10.

.PARAMETER StaleSeconds
    Liveness threshold in seconds. Newest work/gate/heartbeat write within this
    = ALIVE; within 2x = SLOWING; beyond 2x (or nothing) = STALLED. Default 300
    (tuned for a build+test+review loop). Lower it for a quick model step (e.g.
    60); raise it for a long gate.

.PARAMETER RunLabel
    Optional heartbeat label. When given, the probe folds the last-write time of
    <root>\tools\logs\<RunLabel>-progress.txt into the liveness signal and prints
    its last line. This is the heartbeat convention from the orchestration rule.

.PARAMETER NoGit
    Skip the git-commit check (use when -Path is not inside a repo).

.EXAMPLE
    # Liveness verdict for a worktree, 5-min baseline, with a heartbeat log:
    C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -File tools\powershell\liveness-probe.ps1 -Path .worktrees\editor-f3 -RunLabel editor-f3

.EXAMPLE
    # Scoped compile check (replaces the old ff-check.ps1 check):
    C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -File tools\powershell\liveness-probe.ps1 -Task check

.NOTES
    Reusable project tool. Read-only except `fmt` (formats) and the cargo build
    cache. Safe to re-run. Logs are overwritten (per task) each run. The probe's
    exit code encodes the verdict (0 ALIVE / 3 SLOWING / 4 STALLED); the other
    tasks exit 0.
#>
[CmdletBinding()]
param(
    [ValidateSet('probe', 'status', 'check', 'test', 'clippy', 'fmt')]
    [string]$Task = 'probe',
    [string]$Path,
    [int]$Top = 10,
    [int]$StaleSeconds = 300,
    [string]$RunLabel,
    [switch]$NoGit
)

$ErrorActionPreference = 'Stop'

# --- Repo root (portable) ----------------------------------------------------
# This file lives at <repo>\tools\powershell\liveness-probe.ps1, so the repo
# root is three parents up. Derived, never hardcoded.
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot  = (Resolve-Path -LiteralPath (Join-Path $scriptDir '..\..')).Path

# Scan root defaults to the repo root; the user may point it at a worktree.
if ($Path) {
    $root = (Resolve-Path -LiteralPath $Path).Path
} else {
    $root = $repoRoot
}

$now     = Get-Date
$logsDir = Join-Path $root 'tools\logs'
if (-not (Test-Path -LiteralPath $logsDir)) {
    try { New-Item -ItemType Directory -Path $logsDir -Force | Out-Null } catch { $logsDir = $root }
}

# Collected report lines. Deliberately NO -f format operator with file paths --
# paths can contain characters that break format strings. Plain concatenation
# and PadRight/PadLeft only, so the report can never fail to format.
$lines = New-Object System.Collections.Generic.List[string]
function Add-Line([string]$s) { [void]$lines.Add($s) }

# Write the collected report to a log file, then echo it to the terminal.
function Emit([string]$logName) {
    $logFile = Join-Path $logsDir $logName
    [void]$lines.Add('')
    [void]$lines.Add("Report written to: " + $logFile)
    $text = ($lines -join [Environment]::NewLine)
    try { Set-Content -LiteralPath $logFile -Value $text -Encoding utf8 } catch {}
    Write-Output $text
}

# Age in whole seconds of a mtime vs now.
function Get-AgeSeconds($time) { [int]($now - $time).TotalSeconds }

# === Liveness helpers (probe / status) =======================================

# Classify a relative path into a bucket: 'build', 'gate', 'vcs', or 'work'.
$vcsDirs = @('.git', 'node_modules')
function Get-Bucket([string]$rel) {
    $seg = $rel -split '[\\/]'
    if ($seg | Where-Object { $vcsDirs -contains $_ }) { return 'vcs' }
    if ($seg -contains 'target') { return 'build' }
    if ($seg -contains '.gate')  { return 'gate' }
    return 'work'
}

# Newest entry across a bucket list, or $null if empty.
function Get-Newest($list) {
    if ($list.Count -eq 0) { return $null }
    $list | Sort-Object Time -Descending | Select-Object -First 1
}

# Latest git commit under $root (best-effort). Returns a PSObject with Line and
# AgeSeconds, or Line = $null when not a repo / git absent / -NoGit.
function Get-LastCommit {
    $result = [pscustomobject]@{ Line = $null; AgeSeconds = $null }
    if ($NoGit) { return $result }
    try {
        $raw = & git -C $root log -1 --format="%cd|%h|%s" --date=format:"%Y-%m-%d %H:%M:%S" 2>$null
        if ($LASTEXITCODE -eq 0 -and $raw) {
            $parts = $raw -split '\|', 3
            $result.Line = $parts[1] + "  " + $parts[0] + "  " + $parts[2]
            try { $result.AgeSeconds = Get-AgeSeconds ([datetime]::ParseExact($parts[0], "yyyy-MM-dd HH:mm:ss", $null)) } catch {}
        }
    } catch {}
    return $result
}

# Heartbeat: last line + mtime of tools/logs/<RunLabel>-progress.txt.
function Get-Heartbeat([string]$label) {
    $result = [pscustomobject]@{ Found = $false; LastLine = ''; Time = $null; Path = '' }
    if (-not $label) { return $result }
    $hb = Join-Path $logsDir ($label + '-progress.txt')
    $result.Path = $hb
    if (-not (Test-Path -LiteralPath $hb)) { return $result }
    $result.Found = $true
    $content = @(Get-Content -LiteralPath $hb -ErrorAction SilentlyContinue | Where-Object { $_ -ne '' })
    if ($content.Count -gt 0) { $result.LastLine = $content[-1] }
    try { $result.Time = (Get-Item -LiteralPath $hb).LastWriteTime } catch {}
    return $result
}

# Gather + bucket recent writes. Returns a hashtable of lists.
#
# PERFORMANCE: this is a Rust workspace -- target\ holds hundreds of thousands of
# compiler artefacts, and node_modules / .git are likewise huge; nested
# worktrees under .worktrees\ carry their OWN target\ / .git. A naive
# `Get-ChildItem -Recurse` over the whole root HANGS walking those. So this is a
# MANUAL stack-based walk that PRUNES the heavy dir NAMES before ever descending
# into them -- they are never enumerated at any depth. BUILD is excluded from
# the verdict and is shown only for visibility, so its headline is a cheap,
# bounded one-level stat of target\ (never a deep walk).
function Get-Buckets {
    $prune = @('.git', 'node_modules', 'target')  # never descended into
    $work  = New-Object System.Collections.Generic.List[object]
    $gate  = New-Object System.Collections.Generic.List[object]
    $build = New-Object System.Collections.Generic.List[object]

    # Manual DFS: push dirs, pop and process, pruning heavy dir names so they are
    # never opened. This bounds the walk to real work product.
    $stack = New-Object System.Collections.Generic.Stack[string]
    $stack.Push($root)
    while ($stack.Count -gt 0) {
        $dir = $stack.Pop()
        $children = Get-ChildItem -LiteralPath $dir -Force -ErrorAction SilentlyContinue
        foreach ($c in $children) {
            if ($c.PSIsContainer) {
                if ($prune -contains $c.Name) { continue }   # never descend
                $stack.Push($c.FullName)
            } else {
                $rel = $c.FullName.Substring($root.Length).TrimStart('\', '/')
                switch (Get-Bucket $rel) {
                    'gate' { [void]$gate.Add([pscustomobject]@{ Rel = $rel; Time = $c.LastWriteTime }) }
                    'work' { [void]$work.Add([pscustomobject]@{ Rel = $rel; Time = $c.LastWriteTime }) }
                    default { }  # 'vcs'/'build' cannot occur (pruned above)
                }
            }
        }
    }

    # BUILD headline only, WITHOUT walking target\ (a full recursive walk of a
    # Rust target\ is exactly what hangs). Approximate "newest build write" from
    # the shallow mtimes of target\ itself and its immediate children (profile
    # dirs like debug\ / release\, whose mtime bumps when their contents change).
    # One-level stat; BUILD never feeds the verdict.
    $targetDir = Join-Path $root 'target'
    if (Test-Path -LiteralPath $targetDir) {
        $shallow = @(Get-Item -LiteralPath $targetDir -ErrorAction SilentlyContinue)
        $shallow += Get-ChildItem -LiteralPath $targetDir -Force -ErrorAction SilentlyContinue
        $newestBuild = $shallow | Where-Object { $_ } | Sort-Object LastWriteTime -Descending | Select-Object -First 1
        if ($newestBuild) {
            $rel = $newestBuild.FullName.Substring($root.Length).TrimStart('\', '/')
            [void]$build.Add([pscustomobject]@{ Rel = ($rel + '  (shallow mtime)'); Time = $newestBuild.LastWriteTime })
        }
    }

    return @{ Work = $work; Gate = $gate; Build = $build }
}

function Add-BucketHeadline([string]$label, $newest, $count) {
    if ($null -eq $newest) {
        Add-Line ($label.PadRight(7) + ": (none)")
    } else {
        $age = Get-AgeSeconds $newest.Time
        Add-Line ($label.PadRight(7) + ": " + $newest.Time.ToString("yyyy-MM-dd HH:mm:ss") +
                  "  (" + $age + "s ago)  " + $newest.Rel + "   [" + $count + " files]")
    }
}

function Add-BucketList([string]$title, $list) {
    if ($list.Count -eq 0) { return }
    Add-Line ''
    Add-Line ("---- " + $title + " (top " + $Top + ") ----")
    Add-Line ("LAST WRITE".PadRight(21) + "  " + "AGE(s)".PadLeft(10) + "  " + "RELATIVE PATH")
    Add-Line ('-' * 78)
    $list | Sort-Object Time -Descending | Select-Object -First $Top | ForEach-Object {
        $age = Get-AgeSeconds $_.Time
        Add-Line ($_.Time.ToString("yyyy-MM-dd HH:mm:ss").PadRight(21) + "  " +
                  ([string]$age).PadLeft(10) + "  " + $_.Rel)
    }
}

# === Cargo helper (check / test / clippy / fmt) ===============================

# Run a cargo command from the repo root, capture combined output + exit code.
function Invoke-Cargo([string[]]$cargoArgs) {
    Push-Location $repoRoot
    try {
        $out = & cargo @cargoArgs 2>&1
        $code = $LASTEXITCODE
    } finally {
        Pop-Location
    }
    foreach ($o in $out) { Add-Line ([string]$o) }
    Add-Line ''
    Add-Line ('=== cargo ' + ($cargoArgs -join ' ') + ' exit code: ' + $code + ' ===')
    return $code
}

# === Task dispatch ============================================================

$exitCode = 0

switch ($Task) {

    { $_ -in 'probe', 'status' } {
        $buckets     = Get-Buckets
        $newestWork  = Get-Newest $buckets.Work
        $newestGate  = Get-Newest $buckets.Gate
        $newestBuild = Get-Newest $buckets.Build

        # Liveness signal: the freshest of newest-WORK, newest-GATE, and the
        # heartbeat's last write. Build output is deliberately excluded.
        $hb = Get-Heartbeat $RunLabel
        $signalTime = $null
        foreach ($cand in @($newestWork, $newestGate)) {
            if ($cand -and (-not $signalTime -or $cand.Time -gt $signalTime)) { $signalTime = $cand.Time }
        }
        if ($hb.Found -and $hb.Time -and (-not $signalTime -or $hb.Time -gt $signalTime)) { $signalTime = $hb.Time }

        $commit = Get-LastCommit

        if ($Task -eq 'probe') {
            # Verdict from the signal age vs the threshold.
            if ($null -eq $signalTime) {
                $verdict = "STALLED"; $exitCode = 4
                $verdictWhy = "no work/gate/heartbeat writes found under the scan root -- nothing is being written here"
            } else {
                $sigAge = Get-AgeSeconds $signalTime
                if ($sigAge -le $StaleSeconds) {
                    $verdict = "ALIVE"; $exitCode = 0
                    $verdictWhy = "newest work/gate/heartbeat write is " + $sigAge + "s old (<= " + $StaleSeconds + "s threshold)"
                } elseif ($sigAge -le (2 * $StaleSeconds)) {
                    $verdict = "SLOWING"; $exitCode = 3
                    $verdictWhy = "newest write is " + $sigAge + "s old (between 1x and 2x the " + $StaleSeconds + "s threshold) -- may be mid build/review; re-check before calling it, do NOT abort yet"
                } else {
                    $verdict = "STALLED"; $exitCode = 4
                    $verdictWhy = "newest write is " + $sigAge + "s old (> 2x the " + $StaleSeconds + "s threshold) -- treat as hung or restart-paused; recover, do not keep polling"
                }
            }

            Add-Line "================ LIVENESS PROBE ================"
            Add-Line ("VERDICT           : " + $verdict + " -- " + $verdictWhy)
            Add-Line ("Probe run at (NOW): " + $now.ToString("yyyy-MM-dd HH:mm:ss"))
            Add-Line ("Scan root         : " + $root)
            Add-Line ("Stale threshold   : " + $StaleSeconds + "s (ALIVE <= 1x, SLOWING <= 2x, STALLED > 2x)")
        } else {
            Add-Line "================ STATUS SNAPSHOT ================"
            Add-Line ("Snapshot at (NOW) : " + $now.ToString("yyyy-MM-dd HH:mm:ss"))
            Add-Line ("Scan root         : " + $root)
        }

        Add-Line ''
        Add-Line "---- NEWEST WRITE PER BUCKET ----"
        Add-BucketHeadline "WORK"  $newestWork  $buckets.Work.Count
        Add-BucketHeadline "GATE"  $newestGate  $buckets.Gate.Count
        Add-BucketHeadline "BUILD" $newestBuild $buckets.Build.Count

        Add-Line ''
        Add-Line "---- HEARTBEAT ----"
        if (-not $RunLabel) {
            Add-Line "heartbeat        : no -RunLabel given (skipped)"
        } elseif (-not $hb.Found) {
            Add-Line ("heartbeat        : none at " + $hb.Path)
        } else {
            $hbAge = if ($hb.Time) { Get-AgeSeconds $hb.Time } else { '?' }
            Add-Line ("heartbeat file   : " + $hb.Path)
            Add-Line ("last line        : " + $hb.LastLine)
            Add-Line ("last line age    : " + $hbAge + "s")
        }

        Add-Line ''
        if ($NoGit) {
            Add-Line "GIT last commit  : (skipped -- -NoGit)"
        } elseif ($commit.Line) {
            $gitAgeTxt = if ($null -ne $commit.AgeSeconds) { "  (" + $commit.AgeSeconds + "s ago)" } else { "" }
            Add-Line ("GIT last commit  : " + $commit.Line + $gitAgeTxt)
            if ($null -ne $signalTime -and $null -ne $commit.AgeSeconds) {
                $sigAge = Get-AgeSeconds $signalTime
                if ($commit.AgeSeconds -gt $sigAge + 60) {
                    Add-Line ("GIT note         : work is NEWER than the last commit -- uncommitted work in flight" +
                              " (the 'produced output but has not committed' stall shape -- check this if STALLED).")
                }
            }
        } else {
            Add-Line "GIT last commit  : (no commit found / not a git repo)"
        }

        Add-BucketList "WORK files"  $buckets.Work
        Add-BucketList "GATE files"  $buckets.Gate
        Add-BucketList "BUILD files" $buckets.Build

        if ($Task -eq 'probe') {
            Add-Line ''
            Add-Line ("================ END PROBE (" + $verdict + ") ================")
            Emit 'liveness-probe.txt'
        } else {
            Add-Line ''
            Add-Line "================ END SNAPSHOT ================"
            Emit 'liveness-status.txt'
        }
    }

    'check' {
        Add-Line ('=== scoped cargo check @ ' + $now.ToString('yyyy-MM-dd HH:mm:ss') + ' ===')
        Add-Line ''
        [void](Invoke-Cargo @('check', '-p', 'ff-idcams', '-p', 'ff-desktop', '-p', 'ff-dscatalog', '-p', 'ff-vfs', '-p', 'ff-volume', '--all-targets'))
        Emit 'ff-check.txt'
    }

    'test' {
        Add-Line ('=== scoped cargo test @ ' + $now.ToString('yyyy-MM-dd HH:mm:ss') + ' ===')
        Add-Line ''
        [void](Invoke-Cargo @('test', '-p', 'ff-idcams'))
        Add-Line ''
        Add-Line '================ ff-desktop ================'
        [void](Invoke-Cargo @('test', '-p', 'ff-desktop'))
        Emit 'ff-test.txt'
    }

    'clippy' {
        Add-Line ('=== scoped cargo clippy @ ' + $now.ToString('yyyy-MM-dd HH:mm:ss') + ' ===')
        Add-Line ''
        [void](Invoke-Cargo @('clippy', '-p', 'ff-idcams', '-p', 'ff-desktop', '--', '-D', 'warnings'))
        Emit 'ff-clippy.txt'
    }

    'fmt' {
        Add-Line ('=== cargo fmt @ ' + $now.ToString('yyyy-MM-dd HH:mm:ss') + ' ===')
        Add-Line ''
        [void](Invoke-Cargo @('fmt'))
        Emit 'ff-fmt.txt'
    }
}

exit $exitCode
