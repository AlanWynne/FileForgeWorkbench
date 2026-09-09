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

$timingLog = Join-Path $logs "verify.timing.log"
$overallStart = Get-Date
$mode = if ($Fast) { "FAST" } else { "FULL" }
$runner = if ($haveNextest) { "nextest" } else { "cargo test (fallback)" }
"Verify run started: $($overallStart.ToString('yyyy-MM-dd HH:mm:ss'))  mode=$mode  runner=$runner" | Out-File -FilePath $timingLog

function Invoke-TimedStep {
    param(
        [string]$Name,
        [string]$Command
    )
    $stepStart = Get-Date
    cmd /c $Command
    $stepEnd = Get-Date
    $elapsed = $stepEnd - $stepStart
    $line = "{0,-16} {1:hh\:mm\:ss\.fff} ({2:N1}s)" -f $Name, $elapsed, $elapsed.TotalSeconds
    $line | Tee-Object -FilePath $timingLog -Append | Out-Null
}

$fmtOut = Join-Path $logs "cargo.fmt.stdout.log"
$fmtErr = Join-Path $logs "cargo.fmt.stderr.log"
$clpOut = Join-Path $logs "cargo.clippy.stdout.log"
$clpErr = Join-Path $logs "cargo.clippy.stderr.log"
$tstOut = Join-Path $logs "cargo.test.stdout.log"
$tstErr = Join-Path $logs "cargo.test.stderr.log"

Invoke-TimedStep -Name "cargo fmt" -Command "cargo fmt --check 1>`"$fmtOut`" 2>`"$fmtErr`""
Invoke-TimedStep -Name "cargo clippy" -Command "cargo clippy --workspace 1>`"$clpOut`" 2>`"$clpErr`""

if ($haveNextest) {
    Invoke-TimedStep -Name "cargo nextest" -Command "cargo nextest run --workspace 1>`"$tstOut`" 2>`"$tstErr`""
} else {
    Invoke-TimedStep -Name "cargo test" -Command "cargo test --workspace 1>`"$tstOut`" 2>`"$tstErr`""
}

# Accumulate problems into ai-review.log. Covers:
#   - clippy/rustc/fmt: lines beginning with "error" or "warning"
#   - test failures: nextest "FAIL"/"failed" markers and libtest "FAILED"
# An empty ai-review.log means the gate is clean.
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
("{0,-16} {1:hh\:mm\:ss\.fff} ({2:N1}s)" -f "TOTAL", $overallElapsed, $overallElapsed.TotalSeconds) |
    Tee-Object -FilePath $timingLog -Append | Out-Null
"Verify run finished: $($overallEnd.ToString('yyyy-MM-dd HH:mm:ss'))" | Tee-Object -FilePath $timingLog -Append | Out-Null

# Surface a one-line verdict for the caller.
$reviewLines = @(Get-Content $review -ErrorAction SilentlyContinue)
if ($reviewLines.Count -eq 0) {
    Write-Host "verify.ps1: CLEAN ($mode, $runner). See tools\logs\verify.timing.log."
} else {
    Write-Host "verify.ps1: ISSUES FOUND ($($reviewLines.Count) line(s)) -- see tools\logs\ai-review.log"
}
