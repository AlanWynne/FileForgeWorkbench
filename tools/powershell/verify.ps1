#Requires -Version 5.1
<#
.SYNOPSIS
    Workspace verification gate for FileForgeWorkbench.

.DESCRIPTION
    Runs formatting, lint, and tests, capturing each step's output to
    tools\logs\ and accumulating any errors/warnings into
    tools\logs\ai-review.log (empty == clean).

    Faster than the previous five-step flow: it drops the separate `cargo check`
    and `cargo build` passes (clippy performs the compile-check; the test step
    builds the test binaries), and runs tests with cargo-nextest, which executes
    every test binary across all cores in parallel and prints one aggregated
    summary. If cargo-nextest is not installed it falls back to `cargo test`.

    Steps:
      1. cargo fmt --check         (formatting; no compile)
      2. cargo clippy --workspace   (compile-check + lint; lib/bin scope, matching the prior gate)
      3. cargo nextest run --workspace  (build + run tests; or `cargo test`)

    Progress + history (added CR-NR-096 follow-up):
      - tools\logs\verify.progress.txt is OVERWRITTEN every few seconds with the
        current phase, elapsed-in-phase, live tests-run/total (during the test
        phase, parsed from nextest's "(n/N)" lines), and an ETA. Read this ONE
        file to see live progress instead of blind-polling.
        NOTE: nextest block-buffers its per-test "(n/N)" output when its stderr
        is redirected to a file (no TTY), so the live count typically only starts
        appearing partway through the test phase and then tracks near-real-time
        to the end. The phase name and elapsed timers update from the very start
        regardless, and the ETA (below) tells you roughly how long to wait.
      - tools\logs\verify.history.csv gets ONE appended row per completed run
        (durations + test counts + verdict). It is a .csv so the "*.log" cleanup
        at the top of each run does NOT wipe it -- the trend survives. Use it to
        spot a run that suddenly takes much longer, or a drop in test count.
      - Before the test phase an ETA is printed from the median test-phase
        duration of recent history rows (same mode), so the caller knows roughly
        how long to wait before checking verify.progress.txt.

.PARAMETER Fast
    Developer inner-loop mode. Sets PROPTEST_CASES=32 so property tests run far
    fewer iterations, giving a quick signal. This is NOT the full gate: the
    canonical verification (no -Fast) keeps proptest's configured iteration
    count (>=100 per testing.md) for real coverage.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File tools\powershell\verify.ps1
    powershell -ExecutionPolicy Bypass -File tools\powershell\verify.ps1 -Fast
#>
param(
    [switch]$Fast
)

$ErrorActionPreference = "Continue"
$repo = "C:\workspace\VSC\FileForgeWorkbench"
$logs = Join-Path $repo "tools\logs"
Set-Location $repo
New-Item -ItemType Directory -Force -Path $logs | Out-Null
# Clean only *.log so the .csv history and .txt progress markers persist across
# runs. (ai-review.log and the cargo.*.log step logs are recreated below.)
Remove-Item -Path (Join-Path $logs "*.log") -ErrorAction SilentlyContinue

# Cargo emits ANSI-coloured status text to stderr. PowerShell stream redirection
# (2>file) wraps native stderr in ErrorRecord objects and re-formats them to the
# console width, producing mangled, hard-wrapped logs. Disabling colour and using
# cmd byte-level redirection captures cargo output verbatim instead.
$env:CARGO_TERM_COLOR = "never"

# -Fast: shrink proptest iteration counts for a quick developer signal. The full
# gate leaves this unset so proptest uses its configured (>=100) case count.
if ($Fast) {
    $env:PROPTEST_CASES = "32"
    Write-Host "verify.ps1: -Fast mode (PROPTEST_CASES=32) -- quick signal, NOT the full gate."
} else {
    Remove-Item Env:\PROPTEST_CASES -ErrorAction SilentlyContinue
}

# Detect cargo-nextest once; fall back to `cargo test` when absent.
$null = & cargo nextest --version 2>$null
$haveNextest = ($LASTEXITCODE -eq 0)

$timingLog   = Join-Path $logs "verify.timing.log"
$progressTxt = Join-Path $logs "verify.progress.txt"
$historyCsv  = Join-Path $logs "verify.history.csv"

$overallStart = Get-Date
$mode   = if ($Fast) { "FAST" } else { "FULL" }
$runner = if ($haveNextest) { "nextest" } else { "cargo test (fallback)" }
"Verify run started: $($overallStart.ToString('yyyy-MM-dd HH:mm:ss'))  mode=$mode  runner=$runner" | Out-File -FilePath $timingLog

$fmtOut = Join-Path $logs "cargo.fmt.stdout.log"
$fmtErr = Join-Path $logs "cargo.fmt.stderr.log"
$clpOut = Join-Path $logs "cargo.clippy.stdout.log"
$clpErr = Join-Path $logs "cargo.clippy.stderr.log"
$tstOut = Join-Path $logs "cargo.test.stdout.log"
$tstErr = Join-Path $logs "cargo.test.stderr.log"

# ── Helpers ─────────────────────────────────────────────────────────────────

function Format-Hms {
    param([TimeSpan]$Span)
    "{0:hh\:mm\:ss}" -f $Span
}

# Write the single-file progress snapshot (overwritten each tick).
function Write-Progress-Snapshot {
    param(
        [string]$Phase,
        [datetime]$PhaseStart,
        [string]$Extra = ""
    )
    $elapsed = (Get-Date) - $PhaseStart
    $overall = (Get-Date) - $overallStart
    $lines = @(
        "phase        : $Phase",
        "mode         : $mode   runner: $runner",
        "phase_elapsed: $(Format-Hms $elapsed)",
        "total_elapsed: $(Format-Hms $overall)",
        "updated      : $((Get-Date).ToString('HH:mm:ss'))"
    )
    if ($Extra) { $lines += $Extra }
    # Best-effort; never let a transient file lock abort the run.
    try { $lines -join "`r`n" | Out-File -FilePath $progressTxt -Encoding ascii } catch {}
}

# Median test-phase seconds from history rows of the same mode (for the ETA).
function Get-EtaSeconds {
    if (-not (Test-Path $historyCsv)) { return $null }
    try {
        $rows = @(Import-Csv $historyCsv | Where-Object { $_.mode -eq $mode -and $_.test_secs })
    } catch { return $null }
    if ($rows.Count -eq 0) { return $null }
    $inv = [System.Globalization.CultureInfo]::InvariantCulture
    $vals = @($rows | ForEach-Object {
        $v = 0.0
        if ([double]::TryParse([string]$_.test_secs, [System.Globalization.NumberStyles]::Float, $inv, [ref]$v)) { $v }
    } | Sort-Object) | Select-Object -Last 10
    if ($vals.Count -eq 0) { return $null }
    $mid = [int][math]::Floor($vals.Count / 2)
    if ($vals.Count % 2 -eq 1) { return $vals[$mid] }
    return [math]::Round((($vals[$mid - 1] + $vals[$mid]) / 2), 1)
}

# Run a step that produces no useful streaming progress (fmt, clippy). Ticks the
# progress file on a timer while the external command runs in a background job.
function Invoke-SimpleStep {
    param(
        [string]$Name,
        [string]$Command
    )
    $stepStart = Get-Date
    $job = Start-Job -ScriptBlock {
        param($repo, $cmd)
        Set-Location $repo
        $env:CARGO_TERM_COLOR = "never"
        cmd /c $cmd
    } -ArgumentList $repo, $Command
    while ($job.State -eq "Running") {
        Write-Progress-Snapshot -Phase $Name -PhaseStart $stepStart
        Start-Sleep -Seconds 3
    }
    Receive-Job $job | Out-Null
    Remove-Job $job -Force
    $elapsed = (Get-Date) - $stepStart
    $line = "{0,-16} {1:hh\:mm\:ss\.fff} ({2:N1}s)" -f $Name, $elapsed, $elapsed.TotalSeconds
    $line | Tee-Object -FilePath $timingLog -Append | Out-Null
    return $elapsed
}

# Run the test step in a background job (streaming stderr verbatim to $tstErr)
# while this loop parses that log for the live "(n/N)" count and updates the
# progress file. Returns the elapsed TimeSpan.
function Invoke-TestStep {
    param(
        [string]$Name,
        [string]$Command,
        [Nullable[double]]$EtaSeconds
    )
    $stepStart = Get-Date
    $job = Start-Job -ScriptBlock {
        param($repo, $cmd)
        Set-Location $repo
        $env:CARGO_TERM_COLOR = "never"
        cmd /c $cmd
    } -ArgumentList $repo, $Command

    $total = $null
    while ($job.State -eq "Running") {
        $done = $null
        # Parse the tail of the streaming stderr for the newest "(n/N)" marker
        # and the "Starting N tests" line. Best-effort; ignore read races.
        try {
            $tail = Get-Content -Path $tstErr -Tail 60 -ErrorAction SilentlyContinue
            if ($tail) {
                foreach ($l in $tail) {
                    if ($l -match 'Starting\s+(\d+)\s+tests') { $total = [int]$Matches[1] }
                    if ($l -match '\(\s*(\d+)\s*/\s*(\d+)\s*\)') {
                        $done  = [int]$Matches[1]
                        $total = [int]$Matches[2]
                    }
                }
            }
        } catch {}

        $extra = ""
        if ($total) {
            $countTxt = if ($done) { "$done/$total" } else { "0/$total" }
            $extra = "tests_run   : $countTxt"
        } else {
            $extra = "tests_run   : (compiling test binaries...)"
        }
        if ($EtaSeconds) {
            $remain = [math]::Max(0, [int]($EtaSeconds - ((Get-Date) - $stepStart).TotalSeconds))
            $extra += "`r`neta_test    : ~$(Format-Hms ([TimeSpan]::FromSeconds($EtaSeconds))) (median); ~${remain}s remaining"
        }
        Write-Progress-Snapshot -Phase $Name -PhaseStart $stepStart -Extra $extra
        Start-Sleep -Seconds 5
    }
    Receive-Job $job | Out-Null
    Remove-Job $job -Force
    $elapsed = (Get-Date) - $stepStart
    $line = "{0,-16} {1:hh\:mm\:ss\.fff} ({2:N1}s)" -f $Name, $elapsed, $elapsed.TotalSeconds
    $line | Tee-Object -FilePath $timingLog -Append | Out-Null
    return $elapsed
}

# ── Steps ───────────────────────────────────────────────────────────────────

Write-Progress-Snapshot -Phase "starting" -PhaseStart $overallStart
$fmtElapsed = Invoke-SimpleStep -Name "cargo fmt"    -Command "cargo fmt --check 1>`"$fmtOut`" 2>`"$fmtErr`""
$clpElapsed = Invoke-SimpleStep -Name "cargo clippy" -Command "cargo clippy --workspace 1>`"$clpOut`" 2>`"$clpErr`""

$eta = Get-EtaSeconds
if ($eta) {
    Write-Host ("verify.ps1: test phase expected ~{0} (median of recent {1} runs). Watch tools\logs\verify.progress.txt." -f (Format-Hms ([TimeSpan]::FromSeconds($eta))), $mode)
} else {
    Write-Host "verify.ps1: no history yet for an ETA; watch tools\logs\verify.progress.txt for live test count."
}

if ($haveNextest) {
    $tstElapsed = Invoke-TestStep -Name "cargo nextest" -Command "cargo nextest run --workspace 1>`"$tstOut`" 2>`"$tstErr`"" -EtaSeconds $eta
} else {
    $tstElapsed = Invoke-TestStep -Name "cargo test" -Command "cargo test --workspace 1>`"$tstOut`" 2>`"$tstErr`"" -EtaSeconds $eta
}

# ── Parse final test counts ──────────────────────────────────────────────────
# nextest: "Summary [ 212.800s] 9301 tests run: 9301 passed, 0 skipped"
#          (with failures: "... N passed, M failed, K skipped")
# cargo test: one or more "test result: ok. N passed; M failed; ..." lines.
$testsRun = $null; $testsPassed = $null; $testsFailed = $null
$allTestOut = @()
$allTestOut += @(Get-Content $tstErr -ErrorAction SilentlyContinue)
$allTestOut += @(Get-Content $tstOut -ErrorAction SilentlyContinue)
if ($haveNextest) {
    $summary = $allTestOut | Select-String -Pattern 'Summary \[.*\]\s+(\d+)\s+tests run:\s+(\d+)\s+passed(?:,\s+(\d+)\s+failed)?' | Select-Object -Last 1
    if ($summary) {
        $testsRun    = [int]$summary.Matches[0].Groups[1].Value
        $testsPassed = [int]$summary.Matches[0].Groups[2].Value
        $testsFailed = if ($summary.Matches[0].Groups[3].Success) { [int]$summary.Matches[0].Groups[3].Value } else { 0 }
    }
} else {
    $passed = 0; $failed = 0; $any = $false
    foreach ($m in ($allTestOut | Select-String -Pattern 'test result:.*?(\d+)\s+passed;\s+(\d+)\s+failed')) {
        $passed += [int]$m.Matches[0].Groups[1].Value
        $failed += [int]$m.Matches[0].Groups[2].Value
        $any = $true
    }
    if ($any) { $testsPassed = $passed; $testsFailed = $failed; $testsRun = $passed + $failed }
}

# ── Accumulate problems into ai-review.log (empty == clean) ──────────────────
$review = Join-Path $logs "ai-review.log"
$patterns = @("^error", "^warning", "\bFAILED\b", "^\s*FAIL\s", "tests? run:.*failed", "test result: FAILED")
Get-ChildItem -Path $logs -Filter "cargo.*.log" |
    Select-String -Pattern $patterns |
    ForEach-Object { $_.Line } |
    Where-Object { $_ -notmatch "0 failed" -and $_ -notmatch "failed:\s*0" } |
    Out-File $review

$overallEnd = Get-Date
$overallElapsed = $overallEnd - $overallStart
"" | Tee-Object -FilePath $timingLog -Append | Out-Null
$countSummary = if ($testsRun -ne $null) { "$testsRun run, $testsPassed passed, $testsFailed failed" } else { "counts unavailable" }
"tests: $countSummary" | Tee-Object -FilePath $timingLog -Append | Out-Null
("{0,-16} {1:hh\:mm\:ss\.fff} ({2:N1}s)" -f "TOTAL", $overallElapsed, $overallElapsed.TotalSeconds) |
    Tee-Object -FilePath $timingLog -Append | Out-Null
"Verify run finished: $($overallEnd.ToString('yyyy-MM-dd HH:mm:ss'))" | Tee-Object -FilePath $timingLog -Append | Out-Null

# ── Verdict + history row ────────────────────────────────────────────────────
$reviewLines = @(Get-Content $review -ErrorAction SilentlyContinue)
$verdict = if ($reviewLines.Count -eq 0) { "CLEAN" } else { "ISSUES" }

# Append one history row (create header on first run). CSV survives *.log cleanup.
# Numbers are formatted with the INVARIANT culture (period decimal) so a locale
# whose decimal separator is a comma (e.g. de-DE, where 15.2 prints as "15,2")
# does NOT inject stray commas that break the CSV column layout.
if (-not (Test-Path $historyCsv)) {
    "timestamp,mode,runner,fmt_secs,clippy_secs,test_secs,total_secs,tests_run,tests_passed,tests_failed,verdict,review_lines" |
        Out-File -FilePath $historyCsv -Encoding ascii
}
$inv = [System.Globalization.CultureInfo]::InvariantCulture
$fmtSecsStr = $fmtElapsed.TotalSeconds.ToString("F1", $inv)
$clpSecsStr = $clpElapsed.TotalSeconds.ToString("F1", $inv)
$tstSecsStr = $tstElapsed.TotalSeconds.ToString("F1", $inv)
$totSecsStr = $overallElapsed.TotalSeconds.ToString("F1", $inv)
$row = @(
    $overallStart.ToString('yyyy-MM-dd HH:mm:ss'),
    $mode,
    $(if ($haveNextest) { "nextest" } else { "cargo-test" }),
    $fmtSecsStr, $clpSecsStr, $tstSecsStr, $totSecsStr,
    $(if ($testsRun -ne $null) { $testsRun } else { "" }),
    $(if ($testsPassed -ne $null) { $testsPassed } else { "" }),
    $(if ($testsFailed -ne $null) { $testsFailed } else { "" }),
    $verdict,
    $reviewLines.Count
) -join ","
$row | Out-File -FilePath $historyCsv -Encoding ascii -Append

# Final progress snapshot so a reader of the progress file sees the outcome.
Write-Progress-Snapshot -Phase "finished ($verdict)" -PhaseStart $overallStart -Extra "tests_run   : $countSummary"

if ($verdict -eq "CLEAN") {
    Write-Host "verify.ps1: CLEAN ($mode, $runner) -- $countSummary. See tools\logs\verify.timing.log / verify.history.csv."
} else {
    Write-Host "verify.ps1: ISSUES FOUND ($($reviewLines.Count) line(s)) -- see tools\logs\ai-review.log ($countSummary)"
}
