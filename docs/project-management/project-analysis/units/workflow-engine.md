# Analysis Record: workflow-engine

- **Wave**: 0 (foundation -- platform-core subsystem for multi-step operations)
- **Backing crate**: `ff-workflow`
- **Spec folder**: `docs/specs/workflow-engine/`
- **Analysed**: Wave 0, task W0.15 (CR-NR-057 re-baseline)
- **Verdict**: Tracking-COMPLETE (124/124 tasks, TCR PASS), BUT with a LOGGING GAP
  (Req 7.6 unmet, checkpoint silent errors -- PA-LOG-005) and an FFW-ARCH-001
  consistency question (checkpoint tokio::fs -- PA-WATCH-004), both PERSISTING
  from the prior pass. NOT a split candidate. Two 400-cap refactors.
- **CR-NR-057 impact**: NONE (workflow specs untouched). Re-verified from code.

---

## 1. Scope summary

`ff-workflow` is the state-machine execution framework. 7 requirements: Req 1
workflow definition (declarative; sequential/parallel/conditional; built-ins);
Req 2 execution (runner, context, async, pause/resume); Req 3 cancellation; Req 4
progress reporting; Req 5 error handling + recovery (fail-fast/continue/retry,
compensating actions); Req 6 workflow registry; Req 7 persistence (checkpoint/
resume, storage directory).

159 req lines, 7 requirements, single backing crate.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 159 lines; 7 reqs | No |
| 3+ distinct responsibilities | definition/runner/progress/error/registry/checkpoint are facets of ONE engine | No |
| 2+ crates | one crate | No |
| low-cohesion clusters | high cohesion | No |
| file-size pressure | runner.rs 483, definition.rs 463 over cap | Yes |

0-1 criteria. **NOT a split candidate.** Only two oversized files.

## 3. Consistency / conflict

- Public types owned here (Workflow, WorkflowDefinition, WorkflowStep,
  WorkflowState, WorkflowContext, WorkflowRunner, CancellationToken, ProgressEvent,
  WorkflowRegistry, ErrorPolicy, CompensatingAction, Checkpoint, WorkflowError) --
  sole owner. Added to consistency-matrix.
- Consumer relationships (correct direction): ff-logging, command-framework (Req
  6.5 invocation), plugin-architecture (Req 6.3 plugin workflows), event bus (Req
  4.5). CancellationToken here is distinct from ff-background-io's (W0.17).
- **PA-WATCH-004 (FFW-ARCH-001, persists)**: `checkpoint.rs` performs direct
  `tokio::fs` I/O (9 sites) rather than routing through ff-vfs. Req 7.3 specifies a
  "platform-appropriate location" and does NOT mention VFS, so not a stated
  violation -- but inconsistent with document-model Req 4.8 and virtual-file-system
  Req 1.1; ff-workflow is NOT a documented FFW-ARCH-001 init-order exception. Owner
  decision: route checkpoint I/O through VFS OR document an explicit exception.

## 4. Completeness

- Tasks: 124 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-workflow` row PASS.
- Req 5.6 (compensating-action failure -> ERROR) satisfied (runner.rs log_error!
  in execute_rollback).
- **Req 7.6 partially UNMET** (see section 5): checkpoint deserialize failure does
  not log ERROR and does not remove the invalid checkpoint.
- Completeness verdict: **feature-tracked complete; Req 7.6 behaviour partially
  unmet.** Logged PA-LOG-005.

## 5. Logging audit -- GAP (persists)

- Only 1 log call site: `log_error!` in runner.rs (Req 5.6 rollback failure).
  `ff-logging` declared but otherwise unused.
- **Req 7.6 unmet**: `checkpoint.rs::load_checkpoint` returns
  CheckpointError/CheckpointSchemaMismatch on deserialize/schema failure but logs
  NOTHING and does NOT remove the invalid checkpoint. Req 7.6 requires "log an
  ERROR-level record, remove the invalid checkpoint, and present the failure".
- **Silent-error swallowing** (PA-LOG-002 pattern, concrete):
  - `scan_resumable` checkpoint.rs:145: `Err(_) => continue` -- unreadable files
    skipped, no log.
  - `cleanup_expired` checkpoint.rs:195: `let _ = tokio::fs::remove_file(...)` --
    delete failures discarded silently.
- No println!/eprintln!. GUI-independence upheld.
- Logging verdict: **partially adequate -- checkpoint layer under-logged, Req 7.6
  unmet.** PA-LOG-005 persists.

## 6. Findings logged

- **PA-LOG-005** (LOGGING-GAP + Req 7.6 partial violation -- code fix, no gate;
  PERSISTS): checkpoint.rs under-logs. Fix: (a) load_checkpoint -> `log_error!` +
  remove invalid checkpoint (Req 7.6); (b) scan_resumable -> `log_warn!` the
  unreadable file; (c) cleanup_expired -> log the remove result. Criterion exists.
- **PA-WATCH-004** (CONSISTENCY WATCH, persists): checkpoint.rs uses `tokio::fs`
  directly (9 sites) vs ff-vfs. Not a stated violation (Req 7.3 omits VFS) but
  inconsistent with document-model / virtual-file-system. Owner: route through VFS
  OR document an explicit FFW-ARCH-001 exception for workflow checkpoints.
- **PA-STD-005** (REFACTOR -- 400-line cap): `runner.rs` (483) and `definition.rs`
  (463) over the cap. Split by concern. REFACTOR, no gate.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-workflow contributes matches
  (doc prose, separators). Rolled into project-wide PA-LOG-001.
