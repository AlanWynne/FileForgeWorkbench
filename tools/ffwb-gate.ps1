#Requires -Version 5.1
<#
.SYNOPSIS
    FileForgeWorkbench unified verification gate (cross-platform PowerShell).

.DESCRIPTION
    THE single verification gate for the workspace. It replaces the former
    `tools\allcargo.bat` driver and `tools\powershell\verify.ps1` -- which
    duplicated each other's work (allcargo ran the whole suite and THEN invoked
    verify.ps1, recompiling and re-running the tests a second time). This script
    runs each phase EXACTLY ONCE:

      1. cargo fmt --check          (formatting; no compile)
      2. cargo clippy --workspace   (compile-check + lint)
      3. cargo nextest run <scope>  (build + run tests; or `cargo test`)

    clippy performs the compile-check and the test step builds the test
    binaries, so there is no separate `cargo check` / `cargo build` pass.

    LOG FILTERING (matches the old allcargo.bat intent): the combined log keeps
    the signal and drops the noise. It FILTERS OUT:
      - per-test success lines   ("<name> ... ok" from cargo test;
                                  "        PASS [   ...]" from cargo nextest)
      - blank / whitespace-only lines
    It KEEPS:
      - group / summary success lines ("test result: ok. N passed; ...",
        nextest "Summary [...] N tests run: N passed", "Finished", "Compiling",
        "Running", "Doc-tests ...")
      - every warning and error
    So the combined log reads as section headers, group outcomes, warnings, and
    errors -- not thousands of individual passing-test lines.

    PORTABLE: the repo root is derived from this script's own location
    (tools/ffwb-gate.ps1 -> repo root is its parent's parent), so the script can
    be pulled from GitHub and run unchanged on any machine. No absolute paths are
    baked in. pwsh runs this on Windows, Linux, and macOS; a sibling
    `tools/ffwb-gate.sh` provides the same gate for plain bash on Linux/macOS.

    OUTPUTS under tools/logs/ (git-ignored):
      - gate.combined.log   : all messages, filtered as described above
      - ai-review.log       : only errors/warnings/failures (empty == clean gate)
      - cargo.*.log         : the raw, unfiltered per-step output
      - verify.progress.txt : live single-file progress snapshot (overwritten)
      - verify.history.csv  : one appended row per run (survives *.log cleanup)
      - verify.timing.log   : per-step timing for the run
      - verify.diag.log     : append-only phase/watchdog diagnostics

.PARAMETER Fast
    Developer inner-loop mode: PROPTEST_CASES=32 for a quick signal. NOT the full
    gate (the canonical run keeps proptest's configured >=100 iteration count).

.PARAMETER AppOnly
    Fast ROUTINE gate. Tests the ff-desktop application dependency closure only,
    EXCLUDING workspace "orphan" crates not yet wired into ff-desktop. The exclude
    list is derived at runtime (all members MINUS the ff-desktop closure) so it
    never drifts. PARTIAL -- not the completion gate.

.PARAMETER Crate
    Inner-loop gate for a SINGLE named crate (`cargo nextest run -p <name>`).
    PARTIAL -- not the completion gate.

.EXAMPLE
    pwsh -ExecutionPolicy Bypass -File tools/ffwb-gate.ps1
    pwsh -ExecutionPolicy Bypass -File tools/ffwb-gate.ps1 -Fast
    pwsh -ExecutionPolicy Bypass -File tools/ffwb-gate.ps1 -AppOnly
    pwsh -ExecutionPolicy Bypass -File tools/ffwb-gate.ps1 -Crate ff-keys
#>
param(
    [switch]$Fast,
    [switch]$AppOnly,
    [string]$Crate
)

if ($AppOnly -and $Crate) {
    Write-Host "ffwb-gate: -AppOnly and -Crate are mutually exclusive. Pick one." -ForegroundColor Red
    exit 2
}

$ErrorActionPreference = "Continue"

# --- Portable repo root ------------------------------------------------------
# This script lives at <repo>/tools/ffwb-gate.ps1, so the repo root is the parent
# of the tools/ directory. Derived from $PSScriptRoot so the script is
# location-independent and works on any machine that pulls the repo.
$toolsDir = $PSScriptRoot
$repo     = Split-Path -Parent $toolsDir
$logs     = Join-Path $toolsDir "logs"

Set-Location $repo
New-Item -ItemType Directory -Force -Path $logs | Out-Null

# Clean only *.log so the .csv history and .txt progress markers persist across
# runs. (ai-review.log, gate.combined.log, and the cargo.*.log step logs are all
# *.log and are recreated below.)
Remove-Item -Path (Join-Path $logs "*.log") -ErrorAction SilentlyContinue

# Cargo emits ANSI-coloured status to stderr; disable colour so the captured
# logs are clean plain text on every platform.
$env:CARGO_TERM_COLOR = "never"

if ($Fast) {
    $env:PROPTEST_CASES = "32"
    Write-Host "ffwb-gate: -Fast mode (PROPTEST_CASES=32) -- quick signal, NOT the full gate."
} else {
    Remove-Item Env:\PROPTEST_CASES -ErrorAction SilentlyContinue
}

# Detect cargo-nextest once; fall back to `cargo test` when absent.
$null = & cargo nextest --version 2>$null
$haveNextest = ($LASTEXITCODE -eq 0)

# --- Scope resolution --------------------------------------------------------
# Decide which packages the TEST step covers. Three scopes:
#   (default)  full --workspace                 -- the canonical completion gate
#   -AppOnly   --workspace --exclude <orphans>  -- app dependency closure only
#   -Crate <n> -p <n>                           -- single crate inner loop
# The -AppOnly exclude list is DERIVED at runtime (never hardcoded) as
# (all workspace members) MINUS (the ff-desktop dependency closure), so a crate
# wired into ff-desktop automatically re-enters the gate and none is silently
# skipped once it ships in the binary.
function Get-AppOrphanExcludes {
    $members = @()
    try {
        $meta = & cargo metadata --no-deps --format-version 1 2>$null | ConvertFrom-Json
        $members = @($meta.packages | ForEach-Object { $_.name })
    } catch { return @() }
    if ($members.Count -eq 0) { return @() }
    $closure = @()
    try {
        $closure = @(& cargo tree -p ff-desktop --edges normal --prefix none 2>$null |
            ForEach-Object { ($_ -replace '\s+v.*$', '').Trim() } |
            Where-Object { $_ -like 'ff-*' } |
            Sort-Object -Unique)
    } catch { return @() }
    if ($closure.Count -eq 0) { return @() }
    @($members | Where-Object { $_ -notin $closure } | Sort-Object)
}

if ($Crate) {
    $scope = "crate"
    $scopeArgs = @("-p", $Crate)
    $scopeLabel = "crate:$Crate"
} elseif ($AppOnly) {
    $orphans = Get-AppOrphanExcludes
    if ($orphans.Count -eq 0) {
        Write-Host "ffwb-gate: -AppOnly could not derive the orphan list; falling back to full --workspace." -ForegroundColor Yellow
        $scope = "workspace"
        $scopeArgs = @("--workspace")
        $scopeLabel = "workspace (AppOnly-fallback)"
    } else {
        $scope = "app-only"
        $scopeArgs = @("--workspace") + ($orphans | ForEach-Object { @("--exclude", $_) })
        $scopeLabel = "app-only (excludes $($orphans.Count) orphan crates)"
    }
} else {
    $scope = "workspace"
    $scopeArgs = @("--workspace")
    $scopeLabel = "workspace (full completion gate)"
}

$timingLog   = Join-Path $logs "verify.timing.log"
$progressTxt = Join-Path $logs "verify.progress.txt"
$historyCsv  = Join-Path $logs "verify.history.csv"
$diagLog     = Join-Path $logs "verify.diag.log"
$combined    = Join-Path $logs "gate.combined.log"
$review      = Join-Path $logs "ai-review.log"

# Per-step watchdog. If a step runs longer than this without finishing, assume it
# is hung (stuck package-cache lock, never-terminating test, or a cargo prompt)
# and forcibly terminate it. Overridable via VERIFY_STEP_TIMEOUT_SECS.
$stepTimeoutSecs = 1800
if ($env:VERIFY_STEP_TIMEOUT_SECS) {
    $parsed = 0
    if ([int]::TryParse($env:VERIFY_STEP_TIMEOUT_SECS, [ref]$parsed) -and $parsed -gt 0) {
        $stepTimeoutSecs = $parsed
    }
}

function Write-Diag {
    param([string]$Message)
    $stamp = (Get-Date).ToString('yyyy-MM-dd HH:mm:ss.fff')
    "[$stamp] $Message" | Out-File -FilePath $diagLog -Append -Encoding ascii
}

$overallStart = Get-Date
$mode   = if ($Fast) { "FAST" } else { "FULL" }
$runner = if ($haveNextest) { "nextest" } else { "cargo test (fallback)" }
"Verify run started: $($overallStart.ToString('yyyy-MM-dd HH:mm:ss'))  mode=$mode  runner=$runner  scope=$scopeLabel" | Out-File -FilePath $timingLog
Write-Diag "RUN START  mode=$mode  runner=$runner  scope=$scopeLabel  step_timeout=${stepTimeoutSecs}s  pid=$PID"
if ($scope -ne "workspace") {
    Write-Host "ffwb-gate: SCOPE = $scopeLabel. This is a PARTIAL gate, NOT the completion gate -- declaring a task/phase done or releasing REQUIRES a clean plain 'ffwb-gate.ps1' (full --workspace)." -ForegroundColor Yellow
}

# --- Combined-log filter -----------------------------------------------------
# Drop per-test success lines and blank lines; keep group/summary successes,
# warnings, and errors. (Matches the old allcargo.bat intent.)
#   per-test ok   : "test some::path ... ok"
#   nextest PASS  : "        PASS [   0.123s] crate name::test"
# A blank line is anything that is empty or only whitespace.
$dropLineRegex = '(\.\.\. ok\s*$)|(^\s*PASS\s*\[)|(^\s*$)'

function Add-Section {
    param(
        [string]$Title,
        [string]$RawLog
    )
    "===== $Title =====" | Out-File -FilePath $combined -Append -Encoding utf8
    if (Test-Path $RawLog) {
        Get-Content -Path $RawLog -ErrorAction SilentlyContinue |
            Where-Object { $_ -notmatch $dropLineRegex } |
            Out-File -FilePath $combined -Append -Encoding utf8
    } else {
        "(no output captured)" | Out-File -FilePath $combined -Append -Encoding utf8
    }
}

$fmtOut = Join-Path $logs "cargo.fmt.stdout.log"
$fmtErr = Join-Path $logs "cargo.fmt.stderr.log"
$clpOut = Join-Path $logs "cargo.clippy.stdout.log"
$clpErr = Join-Path $logs "cargo.clippy.stderr.log"
$tstOut = Join-Path $logs "cargo.test.stdout.log"
$tstErr = Join-Path $logs "cargo.test.stderr.log"

# --- Helpers -----------------------------------------------------------------

function Format-Hms {
    param([TimeSpan]$Span)
    "{0:hh\:mm\:ss}" -f $Span
}

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
        "mode         : $mode   runner: $runner   scope: $scope",
        "phase_elapsed: $(Format-Hms $elapsed)",
        "total_elapsed: $(Format-Hms $overall)",
        "updated      : $((Get-Date).ToString('HH:mm:ss'))"
    )
    if ($Extra) { $lines += $Extra }
    try { $lines -join "`r`n" | Out-File -FilePath $progressTxt -Encoding ascii } catch {}
}

# Median test-phase seconds from history rows of the same mode AND scope (ETA).
function Get-EtaSeconds {
    if (-not (Test-Path $historyCsv)) { return $null }
    try {
        $rows = @(Import-Csv $historyCsv | Where-Object {
            $rowScope = if ($_.PSObject.Properties.Name -contains 'scope' -and $_.scope) { $_.scope } else { 'workspace' }
            $_.mode -eq $mode -and $_.test_secs -and $rowScope -eq $scope
        })
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

# Terminate a background job and its child process tree, then remove it.
function Stop-JobTree {
    param($Job)
    if (-not $Job) { return }
    try { Stop-Job $Job -ErrorAction SilentlyContinue } catch {}
    try { Remove-Job $Job -Force -ErrorAction SilentlyContinue } catch {}
    foreach ($p in @("cargo", "cargo-nextest", "rustc")) {
        Get-Process -Name $p -ErrorAction SilentlyContinue |
            Where-Object { $_.StartTime -ge $overallStart } |
            ForEach-Object { try { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue } catch {} }
    }
}

# Run a step with no useful streaming progress (fmt, clippy) in a background job,
# ticking the progress file while it runs. Captures stdout/stderr verbatim.
function Invoke-SimpleStep {
    param(
        [string]$Name,
        [string]$Command
    )
    $stepStart = Get-Date
    Write-Diag "PHASE START  $Name  cmd={$Command}"
    $job = Start-Job -ScriptBlock {
        param($repo, $cmd)
        Set-Location $repo
        $env:CARGO_TERM_COLOR = "never"
        cmd /c $cmd
        exit $LASTEXITCODE
    } -ArgumentList $repo, $Command
    $timedOut = $false
    while ($job.State -eq "Running") {
        $inPhase = ((Get-Date) - $stepStart).TotalSeconds
        if ($inPhase -ge $stepTimeoutSecs) {
            Write-Diag "WATCHDOG  $Name exceeded ${stepTimeoutSecs}s -- terminating (assumed hung)."
            Write-Progress-Snapshot -Phase "$Name (TIMED OUT)" -PhaseStart $stepStart -Extra "watchdog     : killed after ${stepTimeoutSecs}s"
            Stop-JobTree $job
            $timedOut = $true
            break
        }
        Write-Progress-Snapshot -Phase $Name -PhaseStart $stepStart
        Start-Sleep -Seconds 3
    }
    if (-not $timedOut) {
        Receive-Job $job | Out-Null
        Remove-Job $job -Force
    }
    $elapsed = (Get-Date) - $stepStart
    $line = "{0,-16} {1:hh\:mm\:ss\.fff} ({2:N1}s)" -f $Name, $elapsed, $elapsed.TotalSeconds
    $line | Tee-Object -FilePath $timingLog -Append | Out-Null
    if ($timedOut) {
        $script:StepTimedOut = $true
        Write-Diag "PHASE END    $Name  result=TIMED_OUT  elapsed=$(Format-Hms $elapsed)"
    } else {
        Write-Diag "PHASE END    $Name  result=finished  elapsed=$(Format-Hms $elapsed)"
    }
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
    Write-Diag "PHASE START  $Name  cmd={$Command}"
    $job = Start-Job -ScriptBlock {
        param($repo, $cmd)
        Set-Location $repo
        $env:CARGO_TERM_COLOR = "never"
        cmd /c $cmd
        exit $LASTEXITCODE
    } -ArgumentList $repo, $Command

    $total = $null
    $done  = $null
    $lastLoggedDone = -1
    $sawStarting = $false
    $timedOut = $false
    while ($job.State -eq "Running") {
        $inPhase = ((Get-Date) - $stepStart).TotalSeconds
        if ($inPhase -ge $stepTimeoutSecs) {
            Write-Diag "WATCHDOG  $Name exceeded ${stepTimeoutSecs}s at count=$(if($done){$done}else{0})/$(if($total){$total}else{'?'}) -- terminating (assumed hung)."
            Write-Progress-Snapshot -Phase "$Name (TIMED OUT)" -PhaseStart $stepStart -Extra "watchdog     : killed after ${stepTimeoutSecs}s (see verify.diag.log)"
            Stop-JobTree $job
            $timedOut = $true
            break
        }

        $done = $null
        try {
            $tail = Get-Content -Path $tstErr -Tail 60 -ErrorAction SilentlyContinue
            if ($tail) {
                foreach ($l in $tail) {
                    if ($l -match 'Starting\s+(\d+)\s+tests') { $total = [int]$Matches[1]; $sawStarting = $true }
                    if ($l -match '\(\s*(\d+)\s*/\s*(\d+)\s*\)') {
                        $done  = [int]$Matches[1]
                        $total = [int]$Matches[2]
                    }
                }
            }
        } catch {}

        if ($done -and $done -ne $lastLoggedDone) {
            Write-Diag "PROGRESS  $Name  tests $done/$total"
            $lastLoggedDone = $done
        }

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
    if (-not $timedOut) {
        Receive-Job $job | Out-Null
        Remove-Job $job -Force
    }
    $elapsed = (Get-Date) - $stepStart
    $line = "{0,-16} {1:hh\:mm\:ss\.fff} ({2:N1}s)" -f $Name, $elapsed, $elapsed.TotalSeconds
    $line | Tee-Object -FilePath $timingLog -Append | Out-Null

    if ($timedOut) {
        $script:StepTimedOut = $true
        Write-Diag "PHASE END    $Name  result=TIMED_OUT  elapsed=$(Format-Hms $elapsed)"
    } elseif (-not $sawStarting) {
        Write-Diag "PHASE END    $Name  result=NO_TESTS_RAN (build/manifest error before tests; see cargo.test.stderr.log)  elapsed=$(Format-Hms $elapsed)"
        Write-Host "ffwb-gate: test step exited WITHOUT running any tests -- a build or manifest error occurred. See tools/logs/cargo.test.stderr.log and verify.diag.log." -ForegroundColor Yellow
    } else {
        Write-Diag "PHASE END    $Name  result=finished  elapsed=$(Format-Hms $elapsed)"
    }
    return $elapsed
}

# --- Steps -------------------------------------------------------------------

$script:StepTimedOut = $false

Write-Progress-Snapshot -Phase "starting" -PhaseStart $overallStart
$fmtElapsed = Invoke-SimpleStep -Name "cargo fmt"    -Command "cargo fmt --check 1>`"$fmtOut`" 2>`"$fmtErr`""
$clpElapsed = Invoke-SimpleStep -Name "cargo clippy" -Command "cargo clippy --workspace 1>`"$clpOut`" 2>`"$clpErr`""

$eta = Get-EtaSeconds
if ($eta) {
    Write-Host ("ffwb-gate: test phase expected ~{0} (median of recent {1} runs). Watch tools/logs/verify.progress.txt." -f (Format-Hms ([TimeSpan]::FromSeconds($eta))), $mode)
} else {
    Write-Host "ffwb-gate: no history yet for an ETA; watch tools/logs/verify.progress.txt for live test count."
}

$scopeArgsStr = ($scopeArgs -join " ")
if ($haveNextest) {
    $tstElapsed = Invoke-TestStep -Name "cargo nextest" -Command "cargo nextest run $scopeArgsStr 1>`"$tstOut`" 2>`"$tstErr`"" -EtaSeconds $eta
} else {
    $tstElapsed = Invoke-TestStep -Name "cargo test" -Command "cargo test $scopeArgsStr 1>`"$tstOut`" 2>`"$tstErr`"" -EtaSeconds $eta
}

# --- Build the combined, filtered log ---------------------------------------
# (The raw per-step logs are the *.stdout/*.stderr files captured above; fold
# filtered copies into gate.combined.log so one file carries the whole run.)
"ffwb-gate combined log -- $($overallStart.ToString('yyyy-MM-dd HH:mm:ss'))  mode=$mode  runner=$runner  scope=$scopeLabel" |
    Out-File -FilePath $combined -Encoding utf8
Add-Section -Title "cargo fmt --check (stdout)"       -RawLog $fmtOut
Add-Section -Title "cargo fmt --check (stderr)"       -RawLog $fmtErr
Add-Section -Title "cargo clippy --workspace (stdout)" -RawLog $clpOut
Add-Section -Title "cargo clippy --workspace (stderr)" -RawLog $clpErr
Add-Section -Title "$runner tests (stdout)"            -RawLog $tstOut
Add-Section -Title "$runner tests (stderr)"            -RawLog $tstErr

# --- Parse final test counts -------------------------------------------------
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

# --- Accumulate problems into ai-review.log (empty == clean) ----------------
$patterns = @("^error", "^warning", "\bFAILED\b", "^\s*FAIL\s", "tests? run:.*failed", "test result: FAILED")
Get-ChildItem -Path $logs -Filter "cargo.*.log" |
    Select-String -Pattern $patterns |
    ForEach-Object { $_.Line } |
    Where-Object { $_ -notmatch "0 failed" -and $_ -notmatch "failed:\s*0" } |
    Out-File $review -Encoding utf8

$overallEnd = Get-Date
$overallElapsed = $overallEnd - $overallStart
"" | Tee-Object -FilePath $timingLog -Append | Out-Null
$countSummary = if ($testsRun -ne $null) { "$testsRun run, $testsPassed passed, $testsFailed failed" } else { "counts unavailable" }
"tests: $countSummary" | Tee-Object -FilePath $timingLog -Append | Out-Null
("{0,-16} {1:hh\:mm\:ss\.fff} ({2:N1}s)" -f "TOTAL", $overallElapsed, $overallElapsed.TotalSeconds) |
    Tee-Object -FilePath $timingLog -Append | Out-Null
"Verify run finished: $($overallEnd.ToString('yyyy-MM-dd HH:mm:ss'))" | Tee-Object -FilePath $timingLog -Append | Out-Null

# --- Verdict + history row ---------------------------------------------------
$reviewLines = @(Get-Content $review -ErrorAction SilentlyContinue)
$verdict = if ($script:StepTimedOut) { "TIMEOUT" } elseif ($reviewLines.Count -eq 0) { "CLEAN" } else { "ISSUES" }
Write-Diag "VERDICT  $verdict  review_lines=$($reviewLines.Count)  tests=$countSummary"

"" | Out-File -FilePath $combined -Append -Encoding utf8
"===== VERDICT =====" | Out-File -FilePath $combined -Append -Encoding utf8
"verdict: $verdict   tests: $countSummary   review_lines: $($reviewLines.Count)   elapsed: $(Format-Hms $overallElapsed)" |
    Out-File -FilePath $combined -Append -Encoding utf8

# Append one history row (create/migrate header on first run). CSV survives the
# *.log cleanup. Numbers use the INVARIANT culture (period decimal) so a
# comma-decimal locale does not inject stray commas that break the columns.
$currentHeader = "timestamp,mode,scope,runner,fmt_secs,clippy_secs,test_secs,total_secs,tests_run,tests_passed,tests_failed,verdict,review_lines"
if (-not (Test-Path $historyCsv)) {
    $currentHeader | Out-File -FilePath $historyCsv -Encoding ascii
} else {
    $existing = @(Get-Content $historyCsv -ErrorAction SilentlyContinue)
    $needsMigration = ($existing.Count -eq 0) -or ($existing[0] -ne $currentHeader)
    if ($needsMigration) {
        $migrated = New-Object System.Collections.Generic.List[string]
        $migrated.Add($currentHeader)
        foreach ($line in $existing) {
            if ($line -eq $currentHeader) { continue }
            if ($line -match '^timestamp,mode,') { continue }
            if ([string]::IsNullOrWhiteSpace($line)) { continue }
            $f = $line.Split(",")
            if ($f.Count -eq 12) {
                $rebuilt = @($f[0], $f[1], "workspace") + $f[2..($f.Count - 1)]
                $migrated.Add(($rebuilt -join ","))
            } else {
                $migrated.Add($line)
            }
        }
        $migrated -join "`r`n" | Out-File -FilePath $historyCsv -Encoding ascii
    }
}
$inv = [System.Globalization.CultureInfo]::InvariantCulture
$fmtSecsStr = $fmtElapsed.TotalSeconds.ToString("F1", $inv)
$clpSecsStr = $clpElapsed.TotalSeconds.ToString("F1", $inv)
$tstSecsStr = $tstElapsed.TotalSeconds.ToString("F1", $inv)
$totSecsStr = $overallElapsed.TotalSeconds.ToString("F1", $inv)
$row = @(
    $overallStart.ToString('yyyy-MM-dd HH:mm:ss'),
    $mode,
    $scope,
    $(if ($haveNextest) { "nextest" } else { "cargo-test" }),
    $fmtSecsStr, $clpSecsStr, $tstSecsStr, $totSecsStr,
    $(if ($testsRun -ne $null) { $testsRun } else { "" }),
    $(if ($testsPassed -ne $null) { $testsPassed } else { "" }),
    $(if ($testsFailed -ne $null) { $testsFailed } else { "" }),
    $verdict,
    $reviewLines.Count
) -join ","
$row | Out-File -FilePath $historyCsv -Encoding ascii -Append

Write-Progress-Snapshot -Phase "finished ($verdict)" -PhaseStart $overallStart -Extra "tests_run   : $countSummary"

if ($verdict -eq "CLEAN") {
    Write-Host "ffwb-gate: CLEAN ($mode, $runner, $scope) -- $countSummary. See tools/logs/gate.combined.log / verify.timing.log." -ForegroundColor Green
    exit 0
} elseif ($verdict -eq "TIMEOUT") {
    Write-Host "ffwb-gate: TIMEOUT -- a step ran past the ${stepTimeoutSecs}s watchdog and was terminated (assumed hung). See tools/logs/verify.diag.log." -ForegroundColor Red
    exit 1
} else {
    Write-Host "ffwb-gate: ISSUES FOUND ($($reviewLines.Count) line(s)) -- see tools/logs/ai-review.log and tools/logs/gate.combined.log ($countSummary)." -ForegroundColor Red
    exit 1
}
