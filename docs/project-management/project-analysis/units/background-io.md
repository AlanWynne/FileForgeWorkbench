# Analysis Record: background-io

- **Wave**: 0 (foundation -- async I/O coordination layer over VFS)
- **Backing crate**: `ff-background-io`
- **Spec folder**: `docs/specs/background-io/`
- **Analysed**: Wave 0, task W0.17 (CR-NR-057 re-baseline)
- **Verdict**: INCOMPLETE (tracking says done, code does not match). Req 6.6-6.9
  (error logging, retry, resume-from-position) are NOT integrated into the runtime
  load path despite tasks 9.1-9.4/13.7 marked `[x]` (PA-INCOMPLETE-002, PERSISTS).
  NO TCR rows exist (PA-TCR-001, persists). NOT a split candidate.
- **CR-NR-057 impact**: NONE (background-io specs untouched). Re-verified.

---

## 1. Scope summary

`ff-background-io` is the async load/save coordination layer. 8 requirements:
Req 1 async streaming load; Req 2 progress reporting; Req 3 cancellation; Req 4
background save with atomic rename; Req 5 large-file streaming; Req 6 error
propagation + retry/recovery; Req 7 concurrency/task management; Req 8 VFS
provider async integration.

200 req lines, 8 requirements, single backing crate.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 200 lines; 8 reqs | No |
| 3+ distinct responsibilities | load/save/progress/cancel/concurrency are facets of ONE I/O coordinator | No |
| 2+ crates | one crate | No |
| low-cohesion clusters | high cohesion | No |
| file-size pressure | largest non-test `save.rs` ~302 -- all under cap | No |

0 criteria. **NOT a split candidate.**

## 3. Consistency / conflict

- Public types owned here (IoError, ProgressState, IoPhase, IoTaskHandle,
  RetryPolicy, LoadOptions, SaveOptions, DocumentChunkSource, BackgroundIoService,
  ChunkSize, LargeFileThreshold) -- sole owner. Added to consistency-matrix.
- FFW-ARCH-001 VERIFIED UPHELD: only `std::fs`/`tokio::fs` mention is lib.rs:10
  inside the module doc-comment stating the crate never calls them. All I/O via
  ff-vfs (Req 1.8/4.10/8.1-8.3).
- IoCancellationToken distinct from ff-workflow CancellationToken (W0.15). io.*
  config keys consistent with configuration-system `io` namespace. Consumer
  relationships (document-model, file-operations, workflow-engine) match spec
  Cross-References. No conflict.

## 4. Completeness -- DISCREPANCY (tracking vs code), PERSISTS

- Tasks: 133 `[x]`, 0 `[ ]` -- tracked as fully complete.
- BUT the runtime load path (`load.rs::execute_load`, sole load impl, called only
  from `service.rs:313`) does NOT contain the claimed-done work:
  - **Task 9.1 (retry on transient errors)**: `execute_load` reads chunks in a
    plain loop; on read error returns `IoError::ReadChunkFailed` IMMEDIATELY, no
    retry. `RetryPolicy` exists in retry.rs but is invoked ONLY in its own unit
    tests -- never from the load path (grep 0 in load.rs). Req 6.7 UNMET.
  - **Task 9.2 (resume-from-position)**: no retry -> no resume. Req 6.8 UNMET.
  - **Task 9.3 (log all I/O errors at ERROR)**: execute_load returns IoError
    variants WITHOUT any ff_logging ERROR call. Req 6.6 UNMET on the load path.
  - **Task 9.4 (WARN per retry)**: no retry loop -> no retry WARN. Req 6.9 UNMET.
  - **Task 13.7 ("integration test: transient error with retry")**: NO such test
    in tests/ (grep 0). Retry "tests" are retry.rs unit tests on the policy struct
    + a property test that recomputes backoff arithmetic -- neither exercises the
    load path. FALSE CONFIDENCE.
- What IS correct: Req 4.6 atomic-rename fallback WARN (save.rs), Req 7.7
  incomplete-save-at-shutdown ERROR (service.rs), subsystem lifecycle Info; atomic
  save, cancellation, progress, large-file chunking.
- **NO TCR rows for `ff-background-io`** at all (searched TCR.md: 0 matches).
- Completeness verdict: **INCOMPLETE** -- Req 6.6/6.7/6.8/6.9 effectively
  unimplemented, mis-tracked as done, with tests that do not exercise the real
  path. Logged PA-INCOMPLETE-002. TCR absence PA-TCR-001. Both PERSIST from prior
  pass.

## 5. Logging audit

- 8 log call sites via `ff_logging::log(LogLevel::X, ...)`. Correct: Req 4.6
  fallback WARN, Req 7.7 shutdown ERROR, lifecycle Info.
- MISSING (part of PA-INCOMPLETE-002): Req 6.6 load-error ERROR and Req 6.9 retry
  WARN absent because the load path neither logs errors nor retries.
- No println!/eprintln!.
- Logging verdict: **partially adequate** -- save/shutdown logged; load
  error/retry paths unlogged (unimplemented).

## 6. Findings logged

- **PA-INCOMPLETE-002** (INCOMPLETE + tracking discrepancy + false-positive tests;
  PERSISTS, HIGH): Req 6.6 (load-error ERROR), 6.7 (retry), 6.8 (resume), 6.9
  (retry WARN) NOT integrated into `load.rs::execute_load`; `RetryPolicy` never
  invoked at runtime. Tasks 9.1-9.4/9.9/13.7 `[x]` but the code and the claimed
  integration test (13.7) do not exist / do not exercise the path. Owner action:
  (a) implement retry loop + resume + ERROR/WARN logging (wire RetryPolicy from
  config); (b) add the real mock-VFS integration test; (c) then re-mark tasks.
  Criteria exist (Req 6) -- code-mode, no gate. HIGH priority (silent unretried
  failures + unlogged errors undermine bug reporting).
- **PA-TCR-001** (TCR-GAP, persists): `ff-background-io` has ZERO rows in TCR.md.
  Add rows for Req 1-8 once PA-INCOMPLETE-002 is fixed; Req 6.6-6.9 rows stay NOT
  COVERED/FAIL until then.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-background-io contributes
  matches (doc-comment em-dashes, box-drawing separators). Rolled into PA-LOG-001.
