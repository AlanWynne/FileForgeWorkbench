<#
.SYNOPSIS
  Recover stalled crate1 (ff-catalog-dialog) of l10n slice2: verify COMPILE-CLEAN
  (check + clippy + fmt), and if clean, COMMIT the work. All output is captured
  INSIDE this script to per-step logs (no external redirect on the invocation);
  a single SUMMARY log carries the verdict to read back.

.NOTES
  Compile-only policy (testing.md Execution split): NO cargo test/nextest/gate.
  Read-then-act: run me, then read tools\logs\slice2-crate1-summary.txt.
#>
$ErrorActionPreference = 'Continue'
$wt = 'C:\workspace\apps\FileForgeWorkbench\.worktrees\l10n-phase2-slice2'
$manifest = Join-Path $wt 'Cargo.toml'
$logDir = 'C:\workspace\apps\FileForgeWorkbench\tools\logs'
if (-not (Test-Path $logDir)) { New-Item -ItemType Directory -Path $logDir -Force | Out-Null }
$summary = Join-Path $logDir 'slice2-crate1-summary.txt'
$checkLog = Join-Path $logDir 'slice2-crate1-check.log'
$clippyLog = Join-Path $logDir 'slice2-crate1-clippy.log'
$fmtLog = Join-Path $logDir 'slice2-crate1-fmt.log'
$commitLog = Join-Path $logDir 'slice2-crate1-commit.log'

$s = New-Object System.Collections.Generic.List[string]
$s.Add("=== slice2 crate1 (ff-catalog-dialog) verify+commit @ " + (Get-Date -Format 'HH:mm:ss') + " ===")

# What changed (internal redirect)
$status = & git -C $wt status --short 2>&1
$s.Add("")
$s.Add("-- git status --")
if (-not $status) { $s.Add("  (clean -- nothing to commit)") } else { @($status) | ForEach-Object { $s.Add("  " + $_) } }

# 1. cargo check (internal redirect to its own log)
& cargo check -p ff-catalog-dialog -p ff-i18n --manifest-path $manifest *> $checkLog
$checkRc = $LASTEXITCODE
$checkTail = (Get-Content $checkLog -Tail 3 -ErrorAction SilentlyContinue) -join ' | '
$s.Add("")
$s.Add("-- cargo check rc=$checkRc --  tail: $checkTail")

# 2. cargo clippy -D warnings
& cargo clippy -p ff-catalog-dialog -p ff-i18n --manifest-path $manifest -- -D warnings *> $clippyLog
$clippyRc = $LASTEXITCODE
$clippyTail = (Get-Content $clippyLog -Tail 3 -ErrorAction SilentlyContinue) -join ' | '
$s.Add("-- cargo clippy rc=$clippyRc --  tail: $clippyTail")

# 3. cargo fmt --check
& cargo fmt -p ff-catalog-dialog -- --check *> $fmtLog
$fmtRc = $LASTEXITCODE
$s.Add("-- cargo fmt --check rc=$fmtRc --")

# Commit only if all three clean (rc 0). Note: phantom -1 is a capture artifact,
# but a REAL non-zero from the program is a genuine failure; the tool wrapper's
# -1 never reaches $LASTEXITCODE here since we run cargo directly in-process.
$allClean = ($checkRc -eq 0) -and ($clippyRc -eq 0) -and ($fmtRc -eq 0)
$s.Add("")
if ($allClean) {
    $s.Add("VERDICT: COMPILE-CLEAN -- committing.")
    & git -C $wt add -A *> $commitLog
    & git -C $wt commit -m "feat(l10n): task7 ff-catalog-dialog localise render literals" *>> $commitLog
    $commitRc = $LASTEXITCODE
    $commitTail = (Get-Content $commitLog -Tail 4 -ErrorAction SilentlyContinue) -join ' | '
    $s.Add("-- git commit rc=$commitRc --  $commitTail")
} else {
    $s.Add("VERDICT: NOT CLEAN (check=$checkRc clippy=$clippyRc fmt=$fmtRc) -- NOT committing. Read the per-step logs.")
}
$s.Add("=== end ===")
Set-Content -Path $summary -Value $s -Encoding utf8
