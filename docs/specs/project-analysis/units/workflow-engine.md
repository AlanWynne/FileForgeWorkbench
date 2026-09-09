# Analysis Record: workflow-engine

- **Wave**: 0 (foundation -- platform-core subsystem for multi-step operations)
- **Backing crate**: `ff-workflow`
- **Spec folder**: `docs/specs/workflow-engine/`
- **Analysed**: Wave 0, task W0.15
- **Verdict**: COMPLETE for tracking (124/124 tasks `[x]`, TCR PASS), BUT with a
  real LOGGING GAP (Req 7.6 checkpoint-failure log+remove unmet; silent-error
  swallowing in checkpoint.rs) and a FFW-ARCH-001 consistency question
  (checkpoint uses `tokio::fs` directly). NOT a split candidate. Two 400-cap
  refactors.

---

## 1. Scope summary

`ff-workflow` is the state-machine execution framework for long-running
operations. 7 requirements:

- Req 1 workflow definition (declarative state machine; sequential/parallel/
  conditional; validation; built-ins: data transfer, import/export,
  compare-merge, bulk rename); Req 2 execution (runner, context passing, async
  steps, pause/resume); Req 3 cancellation (cooperative token, cleanup);
  Req 4 progress reporting (determinate/indeterminate, event-bus, throttled);
  Req 5 error handling + recovery (fail-fast/continue/retry, compensating
  actions, error report); Req 6 workflow registry (name/category, plugin
  registration); Req 7 persistence (checkpoint/resume, storage directory).

159 req lines, 7 requirements, single backing crate. Cohesive.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 159 lines; 7 reqs | No |
| 3+ distinct responsibilities | definition/runner/progress/error/registry/checkpoint are facets of ONE engine | No |
| 2+ crates | one crate `ff-workflow` | No |
| low-cohesion clusters | high cohesion; 13-file split is clean per-concern | No |
| file-size pressure | `runner.rs` 483, `definition.rs` 463 over 400 cap | Yes (not a split driver) |

0-1 criteria. **NOT a split candidate.** Only two oversized files.

## 3. Consistency / conflict

- Public types owned here (Workflow, WorkflowDefinition, WorkflowStep,
  WorkflowState, WorkflowContext, WorkflowRunner, CancellationToken,
  ProgressEvent, WorkflowRegistry, ErrorPolicy, CompensatingAction,
  Checkpoint, WorkflowError) -- sole owner `ff-workflow`. Added to
  consistency-matrix.
- Consumer relationships (correct direction): ff-logging (diagnostics),
  command-framework (Req 6.5 workflow invocation by command handler),
  plugin-architecture (Req 6.3 plugin-registered workflows), event bus
  (Req 4.5 progress emission). No conflict.
- **FFW-ARCH-001 question (PA-WATCH-004)**: `checkpoint.rs` performs direct
  `tokio::fs` I/O (create_dir_all/write/read_to_string/read_dir/remove_file)
  rather than routing through `ff-vfs`. Req 7.3 specifies a "platform-appropriate
  location" and does NOT mention VFS, so this is not a stated violation -- but
  it is INCONSISTENT with document-model (which was required to use VFS for all
  I/O, Req 4.8) and with virtual-file-system Req 1.1 ("no consuming crate SHALL
  contain direct std::fs/tokio::fs"). ff-logging and ff-config are documented
  FFW-ARCH-001 exceptions (they init before VFS); ff-workflow is NOT such an
  exception (it runs after VFS is available). Owner decision needed: either route
  checkpoint I/O through VFS, or add an explicit FFW-ARCH-001 exception note for
  workflow checkpoints in the spec. Recorded as PA-WATCH-004.

## 4. Completeness

- Tasks: 124 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-workflow` row PASS (workflow definition, step execution, state
  transitions).
- Req 5.6 (compensating-action failure -> ERROR log) IS satisfied:
  `runner.rs:424` `ff_logging::log_error!` in `execute_rollback`, continues
  remaining compensations and collects failures. Correct.
- **Req 7.6 is NOT fully satisfied** (see section 5): on checkpoint deserialize
  failure the code returns an error but does NOT log ERROR and does NOT remove
  the invalid checkpoint as the criterion mandates.
- Completeness verdict: **feature-tracked complete, but Req 7.6 behaviour is
  partially unmet** -- logged PA-LOG-005.

## 5. Logging audit -- GAP

- Only 1 log call site in the whole crate: `ff_logging::log_error!` at
  runner.rs:424 (Req 5.6 rollback failure). `ff-logging` IS a declared
  dependency but is otherwise unused.
- **Req 7.6 unmet**: `checkpoint.rs::load_checkpoint` returns
  `WorkflowError::CheckpointError` / `CheckpointSchemaMismatch` on
  deserialize/schema failure but logs NOTHING and does NOT remove the invalid
  checkpoint. Req 7.6 explicitly requires "log an ERROR-level record, remove the
  invalid checkpoint from storage, and present the failure to the user".
- **Silent-error swallowing** (the PA-LOG-002 pattern, concretely):
  - `scan_resumable` (checkpoint.rs:145): `Err(_) => continue` -- unreadable
    checkpoint files are skipped with no log.
  - `cleanup_expired` (checkpoint.rs:195): `let _ = tokio::fs::remove_file(...)`
    -- delete failures discarded silently.
  These hide exactly the bug-report signal the logging audit targets.
- No `println!`/`eprintln!`. GUI-independence upheld.
- Contrast: unlike ff-vfs (PA-LOG-004, no dep + stderr), ff-workflow HAS the dep
  and logs the one hardest path (rollback) correctly -- but under-instruments
  the checkpoint layer. Follow the ff-plugin exemplar (PA-LOG-REF-001).
- Logging verdict: **partially adequate -- checkpoint layer is under-logged and
  Req 7.6 is unmet.**

## 6. Findings logged

- **PA-LOG-005** (LOGGING-GAP + Req 7.6 partial violation -- code fix, no gate):
  `checkpoint.rs` under-logs. Fix: (a) in `load_checkpoint`, on
  deserialize/schema-mismatch failure emit `log_error!` and remove the invalid
  checkpoint file (satisfies Req 7.6 log+remove); (b) in `scan_resumable` replace
  `Err(_) => continue` with a `log_warn!` naming the unreadable file before
  continuing; (c) in `cleanup_expired` replace `let _ = remove_file` with a
  logged result. Criterion already exists (Req 7.6) -- code-mode, no gate.
- **PA-WATCH-004** (FFW-ARCH-001 consistency): `checkpoint.rs` uses `tokio::fs`
  directly rather than `ff-vfs`. Not a stated violation (Req 7.3 omits VFS) but
  inconsistent with document-model Req 4.8 and virtual-file-system Req 1.1.
  Owner decision: route checkpoint I/O through VFS OR document an explicit
  FFW-ARCH-001 exception for workflow checkpoints. Cross-cutting; resolve with
  the platform-core / VFS owner.
- **PA-STD-003** (REFACTOR -- 400-line cap): `runner.rs` (483) and
  `definition.rs` (463) exceed the 400 non-test cap. Split by concern
  (`runner.rs` -> extract `runner_rollback.rs` / `runner_step.rs`;
  `definition.rs` -> extract `definition_builder.rs` / `definition_validate.rs`).
  REFACTOR, no gate.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-workflow contributes 52
  matches (doc-comment prose, separators). Rolled into project-wide PA-LOG-001.
