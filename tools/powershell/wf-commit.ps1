<#
.SYNOPSIS
    Bounded, logged git commit helper for workflow steps (CR-NR-103 Task 8 trial;
    see docs/status/DEFECT-terminal-execution.md section 7.2/7.3).

.DESCRIPTION
    A workflow coder step calls this as the SHORT, BOUNDED commit action at the
    end of an area, instead of running bare `git add` + `git commit` as the
    step's final shell command. It:
      - stages ONLY the explicitly-listed files (never `git add -A`),
      - commits with the given message,
      - writes its FULL output (stage + commit + resulting HEAD line) to a log,
      - echoes that log back,
    so the step verifies the commit BY READING THE LOG (verify-by-effect, never
    the reported exit code -- this machine's wrapper emits a spurious -1), then
    hands off via a NON-SHELL completion signal (send_message success / status).

    This is deliberately a QUICK git-only script: it runs NO compile/test, so it
    is never the long/quiet command that wedges the channel. Keep the heavy
    compile-verify as a separate ASYNC/backgrounded step whose log is read.

.PARAMETER Message
    The commit message (required). Must NOT contain ';' (the guard hook blocks it)
    -- use ',' or '--' instead.

.PARAMETER Files
    One or more repo-relative paths to stage and commit (required).

.PARAMETER RepoDir
    The worktree/repo root. Defaults to two parents up from this script
    (tools/powershell -> repo root), so it works from any worktree unchanged.

.PARAMETER Log
    Log file path. Defaults to <RepoDir>/tools/logs/wf-commit.log.

.EXAMPLE
    pwsh -NoProfile -NonInteractive -File tools/powershell/wf-commit.ps1 `
         -Message "feat(l10n): task8 event-log localise render literals" `
         -Files crates/ff-desktop/src/event_log_panel.rs,crates/ff-i18n/i18n/en/ff-desktop-event-log.ftl
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)] [string]   $Message,
    [Parameter(Mandatory = $true)] [string[]] $Files,
    [string] $RepoDir,
    [string] $Log
)

$ErrorActionPreference = 'Continue'

if (-not $RepoDir) {
    $RepoDir = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
}
if (-not $Log) {
    $Log = Join-Path $RepoDir 'tools\logs\wf-commit.log'
}

# Fresh log each run so stale output never misleads (tooling.md).
$logDir = Split-Path -Parent $Log
if (-not (Test-Path $logDir)) { New-Item -ItemType Directory -Path $logDir -Force | Out-Null }
Set-Content -Path $Log -Value "== wf-commit @ $(Get-Date -Format o) ==" -Encoding utf8

function Write-Log([string]$line) {
    Add-Content -Path $Log -Value $line -Encoding utf8
}

if ($Message -match ';') {
    Write-Log "ERROR: commit message contains ';' (blocked by the guard hook). Use ',' or '--'."
    Get-Content -Path $Log
    exit 2
}

Write-Log "RepoDir : $RepoDir"
Write-Log "Files   : $($Files -join ', ')"

# Stage ONLY the listed files.
$addOut = & git -C $RepoDir add -- $Files 2>&1
Write-Log "-- git add --"
if ($addOut) { Write-Log ($addOut | Out-String).TrimEnd() }

# Show what is staged (so the log proves the right files are in).
$staged = & git -C $RepoDir diff --cached --name-only 2>&1
Write-Log "-- staged files --"
if ($staged) { Write-Log ($staged | Out-String).TrimEnd() }

if (-not $staged) {
    Write-Log "ERROR: nothing staged -- aborting commit (no matching changed files?)."
    Get-Content -Path $Log
    exit 3
}

# Commit.
$commitOut = & git -C $RepoDir commit -m $Message 2>&1
Write-Log "-- git commit --"
if ($commitOut) { Write-Log ($commitOut | Out-String).TrimEnd() }

# Resulting HEAD -- the durable proof the commit landed.
$head = & git -C $RepoDir log --oneline -1 2>&1
Write-Log "-- HEAD now --"
if ($head) { Write-Log ($head | Out-String).TrimEnd() }

Write-Log "== done =="
# Echo the log back so the caller has it even if stdout is swallowed.
Get-Content -Path $Log
