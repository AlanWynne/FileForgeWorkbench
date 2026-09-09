# Incomplete Work Register

Consolidated, living backlog of every pending/incomplete task, orphaned
requirement, NOT-COVERED TCR row without a task, tracking-marker fix, and logging
gap found during the analysis (Requirement 4.3, 5.5).

This EXTENDS `docs/specs/ears-integration/incomplete-work-audit.md` (EI-3) rather
than replacing it. EI-3 established the pending phases and inconsistencies as of
that audit; this register carries them forward and adds new findings per
sub-project, then produces a dependency-ordered sequence in Wave 6.

Status: SEEDED from EI-3; populated per sub-project during Waves 0-5.

Item kinds: PENDING-TASK, ORPHAN-REQ (requirement with no task),
TCR-GAP (NOT COVERED row with no task), TRACKING-FIX (marker correction, no gate),
LOGGING-GAP (missing/inadequate logging, fix under code mode), SUPERSEDED.

---

## Seed rows carried forward from EI-3 (ears-integration/incomplete-work-audit.md)

| ID | Kind | Unit | Item | Status per EI-3 | Notes |
|----|------|------|------|-----------------|-------|
| IWR-001 | PENDING-TASK | dataset-catalog | BS.8-BS.15 (staged txns, integrity/backup, audit trail, security, hierarchy, editor integration, non-functional, design update) | Wave 3-4 pending | Re-verify against current tasks.md/TCR during Wave 2 analysis |
| IWR-002 | PENDING-TASK | virtual-catalog-manager | BU.2-BU.9 (SQLite catalog integration) | pending; needs BS.4 (done) + BS.8 | Re-verify during Wave 2 |
| IWR-003 | PENDING-TASK | dataset-catalog | BV.1 (CatalogLocation refactor) | pending; no deps | EI-3 says can start immediately; CR-NR-017 marked DONE in change-log -- reconcile during Wave 2 |
| IWR-004 | SUPERSEDED | virtual-catalog-manager | VCM Task 17 (dataset file creation on first open) | superseded by BU.7 | Confirm marker note present |
| IWR-005 | TCR-GAP | virtual-file-system | ff-vfs Req 9-12 (StorageProvider, POSIX native, staged txns, backup/restore) had no tasks | gap; add tasks before BS.8 | Re-verify during Wave 0 (virtual-file-system) |
| IWR-006 | TRACKING-FIX | dataset-catalog | task 21 parent `[ ]` but subtasks 21.1-21.4 `[x]` | doc fix | Apply during Wave 2 |
| IWR-007 | TRACKING-FIX | project-master | BU.1 marker `[ ]` but BU.D1-BU.D4 `[x]` | doc fix | Reconcile during Wave 2/6 |
| IWR-008 | TRACKING-FIX | project-master / TCR | duplicate Phase BD `NOT COVERED` table in TCR.md | doc fix | Apply during Wave 4 (file-tree-panel) or 6 |
| IWR-009 | PENDING-TASK | compiler-toolchain-integration | Tasks beyond 5.1-5.3 (MockToolchain done; later tasks) | `[~]` in project-master | Re-verify during Wave 5 |

> Note: EI-3 predates several later phases (DB, DC-DE, CR-CH-015/016, CR-NR-054).
> Newer pending work (e.g. CR-NR-054 DF.7-DF.10 code, command-configurator Task 3
> desktop-adapter follow-ups, dockable Output_Panel view) will be added as their
> owning sub-projects are analysed in the relevant waves.

---

## New findings (added per sub-project during analysis)

| ID | Kind | Unit | Item | Proposed action | Wave |
|----|------|------|------|-----------------|------|
| PA-LOG-001 | LOGGING-GAP (ASCII) | platform-core (`ff-core`) + project-wide | `.rs` sources contain non-ASCII in comments and log-message string literals (em-dashes, box-drawing separators), violating rust-standards.md / documentation.md (Rust = plain ASCII only). Confirmed in ff-core (`thread_model.rs` separators; em-dashes in `shutdown.rs`/`lifecycle.rs`/`hot_restart.rs`/`event_bus.rs`/`service_registry.rs`/`panic_hook.rs`). | Schedule ONE project-wide code-mode cleanup pass replacing non-ASCII in `.rs` with ASCII (`--`, `===`); do not fix per-unit. Low severity, cosmetic. | later (dedicated) |
| PA-DOC-001 | TRACKING-FIX | platform-core | requirements.md Req 4.1 layer-membership lists use illustrative crate names (`ff-document`, `ff-edit`, `ff-undo`, `ff-viewport`, `ff-display-lines`, `ff-find`, `ff-nav`) that differ from actual crates (`ff-document-model`, `ff-edit-operations`, `ff-undo`(?), `ff-viewport-scrolling`, `ff-display-line-mapping`, `ff-find-and-replace`, `ff-navigation-commands`). | Owner decision: reconcile to actual crate names OR annotate as illustrative. Doc-only, no gate. | 6 (consolidation) |
| PA-STD-001 | REFACTOR (400-line cap) | configuration-system (`ff-config`) | Six non-test source files exceed the rust-standards.md 400-line non-test cap: `config_handle.rs` (809), `editorconfig/parser.rs` (626), `reload.rs` (510), `access.rs` (486), `init.rs` (446), `plugin_handle.rs` (432). | Split each by concern (`_state`/`_render`/`_commands`/helpers) per rust-standards.md. REFACTOR, no behaviour change, no gate. | 6 (consolidation) |
| PA-DOC-002 | TRACKING-FIX | configuration-system | requirements.md requirement numbering skips Req 10-14 (jumps 9 -> 15). | Confirm 10-14 are intentionally allocated to other sub-projects / cross-cutting reqs; add a one-line note to the spec Introduction. Doc-only, no gate. | 6 (consolidation) |
| PA-SPLIT-001 | SPLIT PROPOSAL | configuration-system | Spec meets 4/4 split criteria (365 req lines, 13 reqs, 2 crates, separable core vs enterprise/UI bands). | Proposal: split SPEC into core (Req 1-9) + `configuration-enterprise` (Req 16-18); fold Settings UI (Req 15, 18.6) into `menu-workspace`. Crate split NOT recommended. Needs owner approval + own gate. | later (owner-gated) |
| PA-TRACK-001 | TRACKING-FIX | logging-subsystem (`ff-logging` / `tools`) | Req 12 Logging Inventory Tool is implemented and verified (tool `tools/python/logging_inventory.py` + report `docs/quality/logging-inventory.md` present, runs deterministically -- Property 12 holds), but tasks 23-24 remain `[ ]` and TCR "Phase DD" rows Req 12.1-12.5 remain NOT COVERED (red). | Owner action: check tasks 23.1-24.3; set Phase DD TCR rows to PASS citing the tool/report as evidence. Bookkeeping, no code, no gate. | 6 (consolidation) |
| PA-LOG-002 | LOGGING DATA (project-wide) | all crates | Generated `docs/quality/logging-inventory.md` reports 56/69 crates with ZERO log call sites and 792 silent-error review candidates. | Authoritative input for the project-analysis logging audit (Req 5). Reference per-unit rather than re-scanning; owner decides which zero-log crates need instrumentation. | later (per-unit + owner-gated) |
| PA-INCOMPLETE-001 | INCOMPLETE (real code gap) | command-framework (`ff-command` + `ff-desktop`) | Req 9 Command Arguments (CR-NR-054) is unimplemented: 6 Phase DF tasks open (DF.1-DF.6), all TCR Phase DF rows red, no `parse_invocation`/`CommandInvocation`/reserved `arg` param in code. This is the user-designed `DOWN 8` / `MENU SETTINGS EDITOR` / `RETRIEVE LIST` key-forwarding feature. | Requirements gate already complete (Req 9 authored). Owner action: schedule Phase DF implementation via TDD (testing.md). Feeds the task-revision step (project-analysis Req 6). | task-revision (ready to build) |
| PA-WATCH-001 | CONSISTENCY WATCH | command-framework (`Command_Target`) | Command_Target (Req 8) is consumed by menu-workspace (Req 10/11), command-configurator (Req 1), startup-and-session (Req 21), shell-command (Req 19), lua-macro-engine (Req 5). | When each consumer unit is analysed (Waves 1-5), verify Target_Resolution is honoured consistently (no ad-hoc parsing bypassing the owner). No conflict now. | per-unit (Waves 1-5) |
| PA-WATCH-002 | FILE-SIZE WATCH | document-model (`ff-document-model`) | `document.rs` is 399 non-test lines -- exactly at the rust-standards.md 400-line cap; `text_buffer.rs` 354. | Split `document.rs` by concern (e.g. `document_nav.rs` / `document_scroll.rs`) the next time it is touched. REFACTOR, no gate. | 6 (consolidation) / on next touch |
| PA-LOG-003 | LOGGING ENHANCEMENT (optional) | document-model (`ff-document-model`) | Zero log call sites despite depending on `ff-logging`; all errors surface via Result / LoadingProgress::Failed / watcher notifications. Defensible for a pure GUI-independent library. | Optional: add `log_warn!` on streaming-load failure (Req 4.6) and `log_debug!` on read-only modify-attempt (Req 7.4) to aid bug reporting. Owner-gated; not a completeness blocker. | later (owner-gated) |
