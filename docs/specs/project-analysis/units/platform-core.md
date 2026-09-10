# Analysis Record: platform-core

- **Wave**: 0 (foundation -- central GUI-independent orchestration layer)
- **Backing crate**: `ff-core`
- **Spec folder**: `docs/specs/platform-core/`
- **Analysed**: Wave 0, task W0.1 (CR-NR-057 re-baseline)
- **Verdict**: COMPLETE (100/100 tasks `[x]`, TCR PASS). NOT a split candidate.
  GUI-independence VERIFIED (zero GUI deps). Logging exemplary. One 400-cap
  refactor (event_bus.rs 435), one Req 4.1 crate-name-drift doc note, project-wide
  non-ASCII contribution.
- **CR-NR-057 impact**: NONE. CR-NR-057 did not touch platform-core specs; this
  spec is unchanged from the prior (superseded) pass. Findings re-verified fresh.

---

## 1. Scope summary

`ff-core` is the central orchestration layer -- owns all application state,
manages subsystem lifecycles, defines the business-logic / GUI-shell boundary,
and declares the workspace layer rules. 9 requirements:

- Req 1 core layer architecture (zero GUI deps, WorkbenchApp entry point);
  Req 2 service registry (type-safe, ordered, freezable); Req 3 event bus
  (bidirectional, subscription, bounded); Req 4 crate structure + 5-layer rules;
  Req 5 startup lifecycle (logging->config->VFS->commands->plugins->GUI);
  Req 6 shutdown lifecycle (reverse, grace period); Req 7 panic handling;
  Req 8 hot-restart of plugins; Req 9 thread model (main/core/Tokio).

196 req lines, 9 requirements, single backing crate. Cohesive.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 196 lines; 9 reqs | No |
| 3+ distinct responsibilities | registry/event-bus/lifecycle/panic/hot-restart/thread-model are facets of ONE orchestration core | No |
| 2+ crates | one crate `ff-core` | No |
| low-cohesion clusters | high cohesion; all serve WorkbenchApp orchestration | No |
| file-size pressure | `event_bus.rs` 435 non-test over cap; lifecycle.rs 360 near | Yes (1) |

0-1 criteria (only event_bus.rs size). **NOT a split candidate.** Fix is the
refactor (PA-STD-001), not a split.

## 3. Consistency / conflict

- Public types owned here (WorkbenchApp, ServiceRegistry, EventBus,
  LifecyclePhase, ThreadContext, CoreError, ConfigProvider trait) -- sole owner
  `ff-core`. Added to consistency-matrix.
- **PA-DOC-001 (crate-name drift, re-confirmed)**: Req 4.1 layer-membership lists
  use illustrative crate names that differ from actual crates: `ff-document`
  (actual `ff-document-model`), `ff-edit` (`ff-edit-operations`), `ff-undo`
  (`ff-undo-redo`), `ff-viewport` (`ff-viewport-scrolling`), `ff-display-lines`
  (`ff-display-line-mapping`), `ff-find` (`ff-find-and-replace`), `ff-nav`
  (`ff-navigation-commands`), `ff-exclude` (`ff-exclude-show-filter`). Owner
  decision: reconcile to actual names OR annotate as illustrative. Doc-only, no
  gate.
- `ConfigProvider` trait is defined here and implemented in ff-config -- correct
  dependency direction (Core layer, ff-config depends on the trait). No conflict.
- Layer rules (Req 4) are the authoritative source for the whole-workspace
  dependency direction that every later unit's consistency check relies on. The
  5-layer model (Foundation/Core/Editor/Feature/Shell) is enforced by Cargo.toml
  (Req 4.7). No conflict; this is the reference other units are checked against.

## 4. Completeness

- Tasks: 100 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-core` row PASS (service registry, event bus, lifecycle,
  startup/shutdown ordering; integration_tests.rs + property_tests.rs).
- Req 1.1 GUI-independence VERIFIED: no egui/winit/wgpu in Cargo.toml (grep 0).
- Completeness verdict: **COMPLETE.**

## 5. Logging audit -- EXEMPLARY

- 32 log call sites via `ff-logging` macros; `ff-logging` declared and used.
- Every mandated ERROR/WARN path present at the correct level:
  - Req 2.7 duplicate service registration -> `log_warn!` (service_registry.rs:65);
    frozen-registry + out-of-order also WARN. PASS.
  - Req 3.7 event-bus buffer overflow -> `log_warn!` with dropped count
    (event_bus.rs:340,372). PASS.
  - Req 5.3/5.4 critical & non-critical startup failure -> `log_error!`
    (lifecycle.rs:225,239,328,342); Req 5.5 startup timeout -> `log_warn!`
    (lifecycle.rs:307). PASS.
  - Req 6.3 shutdown grace-period exceeded -> `log_warn!`; Req 6.5 shutdown
    panic -> `log_error!` (shutdown.rs:103,111). PASS.
- No `println!`/`eprintln!`. GUI-independence upheld.
- Along with ff-plugin (prior exemplar), ff-core is a reference logging pattern
  for foundation crates.
- Logging verdict: **exemplary / fully adequate.**

## 6. Findings logged

- **PA-STD-001** (REFACTOR -- 400-line cap): `event_bus.rs` is 435 non-test
  lines, over the cap (lifecycle.rs 360 near). Split event_bus.rs by concern
  (e.g. `event_bus_dispatch.rs` / `event_bus_subscribe.rs`; keep event_bus.rs
  the EventBus core). REFACTOR, no behaviour change, no gate.
- **PA-DOC-001** (TRACKING-FIX): Req 4.1 illustrative crate names differ from
  actual crate names (see section 3). Reconcile OR annotate as illustrative.
  Doc-only, no gate.
- **PA-LOG-001** (LOGGING-GAP / ASCII, project-wide): ff-core contributes 322
  non-ASCII matches in `.rs` -- em-dashes in comments AND in log-message string
  literals (e.g. shutdown.rs:105 grace-period message, lifecycle.rs:241/344
  "reduced functionality" prefixed by an em-dash), plus box-drawing separators.
  The in-string em-dashes render in the log file; recommend ASCII `--`. Schedule
  ONE project-wide code-mode cleanup pass replacing non-ASCII in `.rs` with ASCII
  (`--`, `===`); do not fix per-unit. Low severity.
