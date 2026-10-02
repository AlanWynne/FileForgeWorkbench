#!/usr/bin/env bash
#
# FileForgeWorkbench unified verification gate (Linux / macOS).
#
# THE single verification gate for the workspace. Replaces the former
# tools/allcargo.bat driver and tools/powershell/verify.ps1 -- which duplicated
# each other's work (allcargo ran the whole suite and THEN invoked verify.ps1,
# recompiling and re-running the tests a second time). This script runs each
# phase EXACTLY ONCE:
#
#   1. cargo fmt --check          (formatting; no compile)
#   2. cargo clippy --workspace   (compile-check + lint)
#   3. cargo nextest run <scope>  (build + run tests; or `cargo test`)
#
# clippy does the compile-check and the test step builds the test binaries, so
# there is no separate `cargo check` / `cargo build` pass.
#
# LOG FILTERING (matches the old allcargo.bat intent): the combined log keeps the
# signal and drops the noise. It FILTERS OUT:
#     per-test success lines  ("<name> ... ok" from cargo test;
#                              "        PASS [   ...]" from cargo nextest)
#     blank / whitespace-only lines
# It KEEPS group/summary success lines (e.g. "test result: ok. N passed",
# nextest "Summary [...] N tests run: N passed", "Finished", "Compiling",
# "Running", "Doc-tests"), and every warning and error.
#
# PORTABLE: the repo root is derived from this script's own location
# (tools/ffwb-gate.sh -> repo root is its parent's parent), so the script can be
# pulled from GitHub and run unchanged on any machine. No absolute paths baked in.
# This bash script is the Linux/macOS counterpart to tools/ffwb-gate.ps1.
#
# OUTPUTS under tools/logs/ (git-ignored):
#   gate.combined.log   : all messages, filtered as described above
#   ai-review.log       : only errors/warnings/failures (empty == clean gate)
#   cargo.*.log         : the raw, unfiltered per-step output
#   verify.history.csv  : one appended row per run (survives *.log cleanup)
#   verify.timing.log   : per-step timing for the run
#   verify.diag.log     : append-only phase/watchdog diagnostics
#
# Usage:
#   tools/ffwb-gate.sh                 # full --workspace completion gate
#   tools/ffwb-gate.sh --fast          # PROPTEST_CASES=32 quick signal (PARTIAL)
#   tools/ffwb-gate.sh --app-only      # app dependency closure only (PARTIAL)
#   tools/ffwb-gate.sh --crate ff-keys # single crate (PARTIAL)

set -u

# --- Portable repo root ------------------------------------------------------
# Resolve this script's own path, following symlinks. Prefer bash's BASH_SOURCE
# but fall back to $0 so the script also works when run as `sh ffwb-gate.sh`.
# Uses `case` instead of the bash-only [[ ]] so it stays POSIX-portable.
if [ -n "${BASH_SOURCE:-}" ]; then
    SOURCE="${BASH_SOURCE}"
else
    SOURCE="$0"
fi
while [ -h "$SOURCE" ]; do
    DIR="$(cd -P "$(dirname "$SOURCE")" >/dev/null 2>&1 && pwd)"
    SOURCE="$(readlink "$SOURCE")"
    case "$SOURCE" in
        /*) ;;
        *) SOURCE="$DIR/$SOURCE" ;;
    esac
done
TOOLS_DIR="$(cd -P "$(dirname "$SOURCE")" >/dev/null 2>&1 && pwd)"
REPO="$(cd -P "$TOOLS_DIR/.." >/dev/null 2>&1 && pwd)"
LOGS="$TOOLS_DIR/logs"

cd "$REPO" || { echo "ffwb-gate: cannot cd to repo root $REPO" >&2; exit 2; }
mkdir -p "$LOGS"

# --- Parse args --------------------------------------------------------------
FAST=0
APP_ONLY=0
CRATE=""
while [ $# -gt 0 ]; do
    case "$1" in
        --fast)      FAST=1 ;;
        --app-only)  APP_ONLY=1 ;;
        --crate)     shift; CRATE="${1:-}" ;;
        *) echo "ffwb-gate: unknown argument '$1'" >&2; exit 2 ;;
    esac
    shift
done
if [ "$APP_ONLY" -eq 1 ] && [ -n "$CRATE" ]; then
    echo "ffwb-gate: --app-only and --crate are mutually exclusive. Pick one." >&2
    exit 2
fi

# Clean only *.log so the .csv history survives across runs.
rm -f "$LOGS"/*.log

export CARGO_TERM_COLOR=never

MODE="FULL"
if [ "$FAST" -eq 1 ]; then
    export PROPTEST_CASES=32
    MODE="FAST"
    echo "ffwb-gate: --fast mode (PROPTEST_CASES=32) -- quick signal, NOT the full gate."
else
    unset PROPTEST_CASES 2>/dev/null || true
fi

# Detect cargo-nextest once; fall back to `cargo test` when absent.
if cargo nextest --version >/dev/null 2>&1; then
    HAVE_NEXTEST=1
    RUNNER="nextest"
else
    HAVE_NEXTEST=0
    RUNNER="cargo-test"
fi

# --- Scope resolution --------------------------------------------------------
# Derive the -app-only exclude list at runtime: (all members) MINUS (ff-desktop
# closure), so a crate wired into ff-desktop automatically re-enters the gate.
derive_orphans() {
    local members closure closure_file
    members="$(cargo metadata --no-deps --format-version 1 2>/dev/null \
        | tr ',' '\n' | grep -oE '"name":"[^"]+"' | sed 's/"name":"//; s/"//' | sort -u)" || return 1
    [ -z "$members" ] && return 1
    closure="$(cargo tree -p ff-desktop --edges normal --prefix none 2>/dev/null \
        | sed -E 's/[[:space:]]+v.*$//' | grep -E '^ff-' | sort -u)" || return 1
    [ -z "$closure" ] && return 1
    # Orphans = members NOT in the ff-desktop closure. Use a temp file + grep
    # -vxF (fixed-string, whole-line) instead of process substitution so the
    # function works under any POSIX-compatible bash.
    closure_file="$(mktemp)"
    printf '%s\n' "$closure" >"$closure_file"
    printf '%s\n' "$members" | grep -vxF -f "$closure_file"
    rm -f "$closure_file"
}

# SCOPE_ARGS is a plain space-separated string of safe single-word tokens
# (crate names never contain spaces), NOT a bash array -- so the script runs
# unchanged on any POSIX /bin/sh as well as bash.
SCOPE="workspace"
SCOPE_ARGS="--workspace"
SCOPE_LABEL="workspace (full completion gate)"
if [ -n "$CRATE" ]; then
    SCOPE="crate"
    SCOPE_ARGS="-p $CRATE"
    SCOPE_LABEL="crate:$CRATE"
elif [ "$APP_ONLY" -eq 1 ]; then
    ORPHANS="$(derive_orphans || true)"
    if [ -z "$ORPHANS" ]; then
        echo "ffwb-gate: --app-only could not derive the orphan list; falling back to full --workspace." >&2
        SCOPE_LABEL="workspace (app-only-fallback)"
    else
        SCOPE="app-only"
        SCOPE_ARGS="--workspace"
        ORPHAN_COUNT=0
        for o in $ORPHANS; do
            [ -z "$o" ] && continue
            SCOPE_ARGS="$SCOPE_ARGS --exclude $o"
            ORPHAN_COUNT=$((ORPHAN_COUNT + 1))
        done
        SCOPE_LABEL="app-only (excludes $ORPHAN_COUNT orphan crates)"
    fi
fi

COMBINED="$LOGS/gate.combined.log"
REVIEW="$LOGS/ai-review.log"
TIMING="$LOGS/verify.timing.log"
DIAG="$LOGS/verify.diag.log"
HISTORY="$LOGS/verify.history.csv"

# Per-step watchdog seconds (override via VERIFY_STEP_TIMEOUT_SECS).
STEP_TIMEOUT="${VERIFY_STEP_TIMEOUT_SECS:-1800}"

write_diag() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >>"$DIAG"
}

# --- Combined-log filter -----------------------------------------------------
# Drop per-test ok lines, nextest PASS lines, and blank lines. Keep the rest.
DROP_REGEX='(\.\.\. ok[[:space:]]*$)|(^[[:space:]]*PASS[[:space:]]*\[)|(^[[:space:]]*$)'

add_section() {
    local title="$1" raw="$2"
    echo "===== $title =====" >>"$COMBINED"
    if [ -f "$raw" ]; then
        grep -Ev "$DROP_REGEX" "$raw" >>"$COMBINED" || true
    else
        echo "(no output captured)" >>"$COMBINED"
    fi
}

# Run one cargo step with a background watchdog. Captures stdout+stderr verbatim
# to the raw log. Writes the step exit code to the global RUN_STEP_CODE and
# echoes the elapsed seconds on stdout. (Uses only portable constructs: plain
# globals, no `local`, no `printf -v`, no arrays -- so it runs on any POSIX sh.)
run_step() {
    RUN_STEP_NAME="$1"
    RUN_STEP_RAW="$2"
    shift 2
    RUN_STEP_START=$(date +%s)
    write_diag "PHASE START  $RUN_STEP_NAME"
    # Progress line goes to stderr: run_step's STDOUT must carry ONLY the elapsed
    # seconds, because the caller captures it with $(...). (A stray stdout line
    # here previously corrupted the history CSV.)
    echo "ffwb-gate: $RUN_STEP_NAME ..." >&2
    cargo "$@" >"$RUN_STEP_RAW" 2>&1 &
    RUN_STEP_PID=$!
    RUN_STEP_TIMED=0
    while kill -0 "$RUN_STEP_PID" 2>/dev/null; do
        RUN_STEP_NOW=$(date +%s)
        if [ $((RUN_STEP_NOW - RUN_STEP_START)) -ge "$STEP_TIMEOUT" ]; then
            write_diag "WATCHDOG  $RUN_STEP_NAME exceeded ${STEP_TIMEOUT}s -- terminating (assumed hung)."
            kill -TERM "$RUN_STEP_PID" 2>/dev/null || true
            sleep 2
            kill -KILL "$RUN_STEP_PID" 2>/dev/null || true
            pkill -KILL -P "$RUN_STEP_PID" 2>/dev/null || true
            RUN_STEP_TIMED=1
            break
        fi
        sleep 3
    done
    if [ "$RUN_STEP_TIMED" -eq 1 ]; then
        RUN_STEP_CODE=124
        STEP_TIMED_OUT=1
    else
        wait "$RUN_STEP_PID"
        RUN_STEP_CODE=$?
    fi
    RUN_STEP_END=$(date +%s)
    printf '%-16s %ss (code=%s)\n' "$RUN_STEP_NAME" "$((RUN_STEP_END - RUN_STEP_START))" "$RUN_STEP_CODE" >>"$TIMING"
    write_diag "PHASE END    $RUN_STEP_NAME  code=$RUN_STEP_CODE  elapsed=$((RUN_STEP_END - RUN_STEP_START))s"
    # Write the exit code to a file so it survives even when run_step is called
    # inside a $(...) command substitution (which is its own subshell). The
    # elapsed seconds go to stdout for the caller to capture.
    echo "$RUN_STEP_CODE" >"$LOGS/.last_step_code"
    echo "$STEP_TIMED_OUT" >"$LOGS/.step_timed_out"
    echo $((RUN_STEP_END - RUN_STEP_START))
}

# --- Run ---------------------------------------------------------------------
OVERALL_START=$(date +%s)
echo "Verify run started: $(date '+%Y-%m-%d %H:%M:%S')  mode=$MODE  runner=$RUNNER  scope=$SCOPE_LABEL" >"$TIMING"
write_diag "RUN START  mode=$MODE  runner=$RUNNER  scope=$SCOPE_LABEL  step_timeout=${STEP_TIMEOUT}s"
if [ "$SCOPE" != "workspace" ] || [ "$SCOPE_LABEL" = "workspace (app-only-fallback)" ]; then
    echo "ffwb-gate: SCOPE = $SCOPE_LABEL. PARTIAL gate, NOT the completion gate -- declaring done/releasing REQUIRES a clean plain 'ffwb-gate.sh' (full --workspace)." >&2
fi

STEP_TIMED_OUT=0
FMT_CODE=0; CLP_CODE=0; TST_CODE=0

# run_step echoes elapsed seconds (captured here) and writes its exit code and
# timed-out flag to small files (so they survive the $(...) subshell). Read them
# back after each call. $SCOPE_ARGS is deliberately unquoted so it word-splits
# into separate cargo arguments.
read_step_outcome() {
    RUN_STEP_CODE="$(cat "$LOGS/.last_step_code" 2>/dev/null || echo 1)"
    if [ "$(cat "$LOGS/.step_timed_out" 2>/dev/null || echo 0)" = "1" ]; then
        STEP_TIMED_OUT=1
    fi
}

FMT_SECS="$(run_step "cargo fmt" "$LOGS/cargo.fmt.log" fmt --check)"
read_step_outcome; FMT_CODE=$RUN_STEP_CODE
CLP_SECS="$(run_step "cargo clippy" "$LOGS/cargo.clippy.log" clippy --workspace)"
read_step_outcome; CLP_CODE=$RUN_STEP_CODE

# shellcheck disable=SC2086  # $SCOPE_ARGS is intentionally word-split into args.
if [ "$HAVE_NEXTEST" -eq 1 ]; then
    TST_SECS="$(run_step "cargo nextest" "$LOGS/cargo.test.log" nextest run $SCOPE_ARGS)"
else
    TST_SECS="$(run_step "cargo test" "$LOGS/cargo.test.log" test $SCOPE_ARGS)"
fi
read_step_outcome; TST_CODE=$RUN_STEP_CODE

# --- Build the combined, filtered log ---------------------------------------
echo "ffwb-gate combined log -- $(date '+%Y-%m-%d %H:%M:%S')  mode=$MODE  runner=$RUNNER  scope=$SCOPE_LABEL" >"$COMBINED"
add_section "cargo fmt --check" "$LOGS/cargo.fmt.log"
add_section "cargo clippy --workspace" "$LOGS/cargo.clippy.log"
if [ "$HAVE_NEXTEST" -eq 1 ]; then
    add_section "cargo nextest run" "$LOGS/cargo.test.log"
else
    add_section "cargo test" "$LOGS/cargo.test.log"
fi

# --- Parse final test counts -------------------------------------------------
TESTS_RUN=""; TESTS_PASSED=""; TESTS_FAILED=""
if [ "$HAVE_NEXTEST" -eq 1 ]; then
    SUMMARY_LINE="$(grep -E 'Summary \[.*\][[:space:]]+[0-9]+ tests run:' "$LOGS/cargo.test.log" 2>/dev/null | tail -1)"
    if [ -n "$SUMMARY_LINE" ]; then
        TESTS_RUN="$(echo "$SUMMARY_LINE" | grep -oE '[0-9]+ tests run' | grep -oE '[0-9]+')"
        TESTS_PASSED="$(echo "$SUMMARY_LINE" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+')"
        TESTS_FAILED="$(echo "$SUMMARY_LINE" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+')"
        [ -z "$TESTS_FAILED" ] && TESTS_FAILED=0
    fi
else
    TESTS_PASSED=0; TESTS_FAILED=0
    # Grep the "test result:" lines into a temp file and read the loop's stdin
    # from it (file redirect, not process substitution) so the counter updates
    # happen in THIS shell and the script stays portable to a POSIX bash.
    RESULT_TMP="$(mktemp)"
    grep -E 'test result:.*[0-9]+ passed; [0-9]+ failed' "$LOGS/cargo.test.log" 2>/dev/null >"$RESULT_TMP" || true
    while IFS= read -r m; do
        p="$(echo "$m" | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+')"
        f="$(echo "$m" | grep -oE '[0-9]+ failed' | grep -oE '[0-9]+')"
        TESTS_PASSED=$((TESTS_PASSED + ${p:-0}))
        TESTS_FAILED=$((TESTS_FAILED + ${f:-0}))
    done <"$RESULT_TMP"
    rm -f "$RESULT_TMP"
    TESTS_RUN=$((TESTS_PASSED + TESTS_FAILED))
fi

# --- Accumulate problems into ai-review.log (empty == clean) ----------------
grep -hE '^error|^warning|FAILED|^[[:space:]]*FAIL[[:space:]]|tests? run:.*failed|test result: FAILED' \
    "$LOGS"/cargo.*.log 2>/dev/null \
    | grep -Ev '0 failed|failed:[[:space:]]*0' \
    >"$REVIEW" || true

OVERALL_END=$(date +%s)
ELAPSED=$((OVERALL_END - OVERALL_START))
REVIEW_LINES=$(wc -l <"$REVIEW" 2>/dev/null | tr -d ' ')
[ -z "$REVIEW_LINES" ] && REVIEW_LINES=0

if [ -n "$TESTS_RUN" ]; then
    COUNT_SUMMARY="$TESTS_RUN run, $TESTS_PASSED passed, $TESTS_FAILED failed"
else
    COUNT_SUMMARY="counts unavailable"
fi
{
    echo ""
    echo "tests: $COUNT_SUMMARY"
    printf '%-16s %ss\n' "TOTAL" "$ELAPSED"
    echo "Verify run finished: $(date '+%Y-%m-%d %H:%M:%S')"
} >>"$TIMING"

# --- Verdict -----------------------------------------------------------------
if [ "$STEP_TIMED_OUT" -eq 1 ]; then
    VERDICT="TIMEOUT"
elif [ "$REVIEW_LINES" -eq 0 ] && [ "$FMT_CODE" -eq 0 ] && [ "$CLP_CODE" -eq 0 ] && [ "$TST_CODE" -eq 0 ]; then
    VERDICT="CLEAN"
else
    VERDICT="ISSUES"
fi
write_diag "VERDICT  $VERDICT  review_lines=$REVIEW_LINES  tests=$COUNT_SUMMARY"

{
    echo ""
    echo "===== VERDICT ====="
    echo "verdict: $VERDICT   tests: $COUNT_SUMMARY   review_lines: $REVIEW_LINES   elapsed: ${ELAPSED}s"
} >>"$COMBINED"

# --- History row (CSV survives *.log cleanup) -------------------------------
HEADER="timestamp,mode,scope,runner,fmt_secs,clippy_secs,test_secs,total_secs,tests_run,tests_passed,tests_failed,verdict,review_lines"
if [ ! -f "$HISTORY" ]; then
    echo "$HEADER" >"$HISTORY"
fi
echo "$(date '+%Y-%m-%d %H:%M:%S'),$MODE,$SCOPE,$RUNNER,$FMT_SECS,$CLP_SECS,$TST_SECS,$ELAPSED,${TESTS_RUN},${TESTS_PASSED},${TESTS_FAILED},$VERDICT,$REVIEW_LINES" >>"$HISTORY"

# Remove the internal per-step scratch files (exit code / timed-out flag).
rm -f "$LOGS/.last_step_code" "$LOGS/.step_timed_out"

if [ "$VERDICT" = "CLEAN" ]; then
    echo "ffwb-gate: CLEAN ($MODE, $RUNNER, $SCOPE) -- $COUNT_SUMMARY. See tools/logs/gate.combined.log / verify.timing.log."
    exit 0
elif [ "$VERDICT" = "TIMEOUT" ]; then
    echo "ffwb-gate: TIMEOUT -- a step ran past the ${STEP_TIMEOUT}s watchdog and was terminated (assumed hung). See tools/logs/verify.diag.log." >&2
    exit 1
else
    echo "ffwb-gate: ISSUES FOUND ($REVIEW_LINES line(s)) -- see tools/logs/ai-review.log and tools/logs/gate.combined.log ($COUNT_SUMMARY)." >&2
    exit 1
fi
