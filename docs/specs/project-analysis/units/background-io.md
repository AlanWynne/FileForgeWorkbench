# Analysis Record: background-io

- **Wave**: 0 (foundation -- async I/O coordination layer over VFS)
- **Backing crate**: `ff-background-io`
- **Spec folder**: `docs/specs/background-io/`
- **Analysed**: Wave 0, task W0.17
- **Verdict**: INCOMPLETE (tracking says done, code does not match). Req 6.6-6.9
  (error logging, retry, resume-from-position) are NOT integrated into the
  runtime load path despite tasks 9.1-9.4/13.7 marked `[x]`. Also NO TCR rows
  exist for this crate. NOT a split candidate.

---

## 1. Scope summary

`ff-background-io` is the async load/save coordination layer. 8 requirements:
Req 1 async streaming load; Req 2 progress reporting; Req 3 cancellation;
Req 4 background save with atomic rename; Req 5 large-file streaming; Req 6
error propagation + retry/recovery; Req 7 concurrency/task management; Req 8
VFS provider async integration.

200 req lines, 8 requirements, single backing crate. Cohesive coordination
layer.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 200 lines; 8 reqs | No |
| 3+ distinct responsibilities | load/save/progress/cancel/concurrency are facets of ONE I/O coordinator | No |
| 2+ crates | one crate `ff-background-io` | No |
| low-cohesion clusters | high cohesion | No |
| file-size pressure | largest non-test file `save.rs` 302 -- all under 400 cap | No |

0 criteria. **NOT a split candidate.** Well-sized (12 files, all under cap).

## 3. Consistency / conflict

- Public types owned here (IoError, ProgressState, IoPhase, IoTaskHandle,
  RetryPolicy, LoadOptions, SaveOptions, DocumentChunkSource,
  BackgroundIoService, ChunkSize, LargeFileThreshold) -- sole owner
  `ff-background-io`. Added to consistency-matrix.
- FFW-ARCH-001 VERIFIED UPHELD: the only `std::fs`/`tokio::fs` mention is
  lib.rs:10, inside the module doc-comment stating the crate never calls them.
  No actual direct fs I/O; all I/O routes through `ff-vfs` (Req 1.8/4.10/8.1-8.3).
- CancellationToken here (`IoCancellationToken`, wrapping Tokio
  CancellationToken) is distinct from workflow-engine's CancellationToken
  (confirms the PA-WATCH note from W0.15 -- two separate cancellation types,
  each owned by its crate, no conflict).
- `io.*` config keys (chunk_size_kb, large_file_threshold_mb,
  max_concurrent_tasks, retry_count, retry_backoff_ms, shutdown_timeout_secs)
  consistent with configuration-system `io` namespace. Consumer direction correct.
- Consumer relationships (document-model streaming delivery, file-operations,
  workflow-engine orchestration) match the spec Cross-References. No conflict.

## 4. Completeness -- DISCREPANCY (tracking vs code)

- Tasks: 133 `[x]`, 0 `[ ]` -- tracked as fully complete.
- BUT verification of the runtime load path (`load.rs::execute_load`, the sole
  load implementation, called only from `service.rs:313`) shows the following
  claimed-done work is NOT present in code:
  - **Task 9.1 (retry policy in LoadTask on transient errors)**: `execute_load`
    reads chunks in a plain loop; on a read error it returns
    `IoError::ReadChunkFailed` IMMEDIATELY with no retry. `RetryPolicy`
    (retry.rs) exists but `is_transient`/`backoff_for_attempt` are invoked ONLY
    in retry.rs's own unit tests -- never from the load path. Req 6.7 UNMET.
  - **Task 9.2 (resume-from-position on retry)**: no retry, so no resume. Req 6.8
    UNMET.
  - **Task 9.3 (log all I/O errors at ERROR with full chain)**: `execute_load`
    returns IoError variants WITHOUT any `ff_logging` ERROR call. Req 6.6 UNMET
    on the load path.
  - **Task 9.4 (WARN per retry attempt)**: no retry loop -> no retry WARN. Req
    6.9 UNMET.
  - **Task 13.7 ("integration test: transient error with retry -- mock VFS fails
    first 2 reads then succeeds ...")**: NO such integration test exists in
    `tests/`. The only retry "tests" are (a) retry.rs unit tests on the policy
    struct in isolation and (b) property_tests.rs Property 8, which recomputes
    `backoff * 2` IN THE TEST ITSELF and never exercises the crate's load path.
    These give FALSE CONFIDENCE -- they pass without any runtime retry existing.
- What IS correctly implemented and logged:
  - Req 4.6 atomic-rename fallback WARN (save.rs:234, LogLevel::Warn). PASS.
  - Req 7.7 incomplete-save-at-shutdown ERROR (service.rs:218, LogLevel::Error).
    PASS.
  - subsystem lifecycle Info logs (subsystem.rs:59/76).
  - Req 4 atomic save (temp + fsync + rename), Req 3 cancellation, Req 2
    progress, Req 5 large-file chunking -- present.
- **NO TCR rows exist for `ff-background-io`** at all (searched TCR.md: zero
  matches). So none of its 8 requirements are recorded in the Test Coverage
  Report, despite tasks marked complete.
- Completeness verdict: **INCOMPLETE** -- Req 6.6/6.7/6.8/6.9 are effectively
  unimplemented in the runtime, mis-tracked as done, with tests that do not
  exercise the real path. Logged PA-INCOMPLETE-002. TCR absence logged
  PA-TCR-001.

## 5. Logging audit

- 8 log call sites via `ff_logging::log(LogLevel::X, ...)` (explicit-level form).
  `ff-logging` declared and used. No println/eprintln.
- Correct: Req 4.6 fallback WARN, Req 7.7 shutdown ERROR, lifecycle Info.
- MISSING (part of PA-INCOMPLETE-002): Req 6.6 load-error ERROR and Req 6.9
  retry WARN are absent because the load path neither logs errors nor retries.
- Logging verdict: **partially adequate** -- save/shutdown paths logged; load
  error/retry paths unlogged (because unimplemented).

## 6. Findings logged

- **PA-INCOMPLETE-002** (INCOMPLETE + tracking discrepancy + false-positive
  tests): Req 6.6 (load-error ERROR log), Req 6.7 (retry transient errors),
  Req 6.8 (resume-from-position), Req 6.9 (retry WARN) are NOT integrated into
  `load.rs::execute_load` -- `RetryPolicy` is never invoked at runtime. Tasks
  9.1-9.4, 9.9, 13.7 are marked `[x]` but the code and the claimed integration
  test (13.7) do not exist / do not exercise the path; Property 8 only tests
  arithmetic. Owner action: (a) implement the retry loop + resume + ERROR/WARN
  logging in execute_load (wire in RetryPolicy from config); (b) add the real
  mock-VFS integration test (13.7); (c) then re-mark tasks. Criteria already
  exist (Req 6) -- code-mode, no gate. HIGH priority: silent unretried failures
  and unlogged errors directly undermine bug reporting.
- **PA-TCR-001** (TCR-GAP): `ff-background-io` has ZERO rows in
  `docs/quality/TCR.md`. Add rows for Req 1-8 with correct PASS/NOT-COVERED
  status once PA-INCOMPLETE-002 is resolved (Req 6.6-6.9 rows should be
  NOT COVERED / FAIL until then, NOT PASS).
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-background-io contributes 34
  matches (doc-comment em-dashes, box-drawing separators in test headers).
  Rolled into project-wide PA-LOG-001.
