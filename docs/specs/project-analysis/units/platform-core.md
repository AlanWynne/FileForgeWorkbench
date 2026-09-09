# Analysis Record -- platform-core

- Wave: 0 (Foundations)
- Backing crate(s): `ff-core`
- Upstream deps analysed: `ff-logging` only (Foundation Layer). Analysed next in
  this wave; platform-core depends only on it, so no ordering violation.
- Prior artifacts consulted: EI-3 audit (no platform-core pending items);
  api-consistency-report.md (WorkbenchApp/PluginContext ownership).

## 1. Split Analysis (Req 2)

- Size: 9 Requirements, ~230 requirement lines. Responsibilities: architecture
  boundary, service registry, event bus, layer rules, startup, shutdown, panic
  handling, hot-restart, thread model -- all facets of ONE cohesive concern
  (the GUI-independent orchestration core). Crates: 1 (`ff-core`).
- Split_Candidate: **NO**.
- Rationale: meets at most one split criterion (responsibility count), but the
  responsibilities are tightly cohesive facets of a single crate's job
  (own state + manage subsystem lifecycles). Size is moderate (< 350 lines).
  Backed by exactly one crate with clear internal module separation
  (`app`, `service_registry`, `event_bus`, `lifecycle`, `shutdown`, `panic_hook`,
  `hot_restart`, `thread_model`, `layer_rules`, `error`). Splitting would
  fragment the orchestration contract. Keep as one Logical_Unit.

## 2. Consistency and Conflicts (Req 3)

Artifacts this unit owns (feed the Consistency_Matrix):
- Types: `WorkbenchApp`, `ServiceRegistry`, `EventBus`, `LifecyclePhase`,
  `ThreadContext`, `CoreError`, `SubsystemCriticality`.
- Concepts/terms: Platform_Core, Service_Registry, Event_Bus, Startup_Sequence,
  Shutdown_Sequence, the five-layer model (Foundation/Core/Editor/Feature/Shell).
- No Command_IDs, config keys, or panel names owned (GUI-independent by design).

Conflicts found:
- **Layer-name drift (LOW):** requirements.md uses illustrative crate names
  (`ff-document`, `ff-edit`, `ff-undo`, `ff-viewport`, `ff-display-lines`,
  `ff-find`, `ff-nav`) that differ from the actual crate names
  (`ff-document-model`, `ff-edit-operations`, `ff-undo-redo` / `ff-undo`,
  `ff-viewport-scrolling`, `ff-display-line-mapping`, `ff-find-and-replace`,
  `ff-navigation-commands`). Not a behaviour conflict; a documentation
  consistency item. Proposed resolution: reconcile the layer-membership lists in
  a later doc pass against the actual workspace crates (owner: platform-core doc;
  cross-ref api-consistency-report.md). Recorded, not changed.
- No dangling cross-references; no contradictions with other units found.

## 3. Completeness (Req 4)

- tasks.md: Tasks 1-15 all `[x]` (scaffold, WorkbenchApp, event interface,
  service registry x2, event bus x3, startup, shutdown, panic, hot-restart,
  thread model, layer rules, PBT). Full Acceptance-Criteria Coverage Matrix
  present covering Req 1-9.
- TCR: `ff-core` row = PASS (`integration_tests.rs`, `property_tests.rs`;
  "Service registry, event bus, lifecycle, startup/shutdown ordering").
- Code + tests: `crates/ff-core/{src,tests}` present; all specced modules exist
  (`app.rs`, `service_registry.rs`, `event_bus.rs`, `lifecycle.rs`, `shutdown.rs`,
  `panic_hook.rs`, `hot_restart.rs`, `thread_model.rs`, plus error). Verified
  under the clean `verify.ps1` baseline (full nextest green).
- Classification: **COMPLETE**. No discrepancies among tasks / TCR / code.
- No orphaned requirements; no tracking-marker fixes needed for this unit.

## 4. Logging Audit (Req 5)

- ff-logging usage: **strong**. `log_info!` for lifecycle/startup/shutdown/
  hot-restart success and signals; `log_warn!` for grace-period timeout,
  event-bus overflow (with dropped count), duplicate/out-of-order/frozen registry
  registration; `log_error!` for critical/non-critical subsystem failures, plugin
  hot-restart failures, and panic capture. Levels are appropriate per
  logging-subsystem.
- Error paths without a log: none material -- the `let _ = catch_unwind(...)` in
  `panic_hook.rs` is intentional (AC 7.5: the hook must never panic; best-effort
  logging is deliberately abandoned on failure), correctly commented.
- `println!`/`eprintln!` in lib: none.
- Two-phase init: N/A here (ff-core receives a `LoggingStatus`; the two-phase
  init is a ff-desktop/ff-logging concern, CR-CH-015).
- Finding **PA-LOG-001 (LOW):** `ff-core` source contains non-ASCII characters
  in comments and log-message string literals -- em-dashes (`--` intended) and
  box-drawing section separators (`thread_model.rs`, and em-dashes in log strings
  across `shutdown.rs`, `lifecycle.rs`, `hot_restart.rs`, `event_bus.rs`,
  `service_registry.rs`, `panic_hook.rs`). This violates rust-standards.md /
  documentation.md (Rust source is plain ASCII only; box-drawing not allowed in
  `.rs`). Cosmetic, no behaviour impact; fix under code mode by replacing with
  ASCII (`--`, `===` separators). This is a codebase-wide pattern (many crates
  predate the ASCII rule); recorded once here and tracked project-wide in the
  register.

## 5. Task Revision (Req 6)

- No incomplete tasks for platform-core; nothing to re-order.
- No supersessions.
- project-master ordering delta: none (Phase not pending).

## 6. Decisions for Owner (Req 8.5)

1. **Layer-membership doc reconciliation** (Section 2): update platform-core
   requirements.md Req 4.1 layer lists to the actual crate names, OR leave as
   illustrative with a note. Low priority; owner call. Not changed during
   analysis.
2. **Project-wide ASCII-in-Rust cleanup** (PA-LOG-001): whether to schedule a
   dedicated code-mode cleanup pass replacing non-ASCII in `.rs` files across the
   workspace. Recommend a single tracked task rather than per-unit fixes.
