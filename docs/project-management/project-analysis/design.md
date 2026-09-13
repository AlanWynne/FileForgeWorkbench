# Design Document -- Project Analysis and Re-Structuring

## 1. Overview

This document defines the method for a systematic, dependency-ordered analysis of
all 79 sub-projects. It is a process design, not a software design: it specifies
the sequence, the per-unit analysis template, the decision criteria, the output
artifacts, and the per-sub-project Git checkpoint protocol. No crate is built.

The analysis is deliberately non-destructive (requirements Requirement 8): it
reads specs, TCR, and code; it writes analysis records and revised task ordering;
it never edits `crates/` source, never deletes criteria, and never executes a
split. Every material change is surfaced as a proposal for owner approval.

Baseline: the tree compiles and `verify.ps1` is CLEAN as of the start of this
analysis (`main` at the pre-analysis checkpoint). This gives a trustworthy code
state to audit against.

---

## 2. Output Artifacts

```
docs/specs/project-analysis/
  requirements.md              -- this analysis's requirements
  design.md                    -- this document
  tasks.md                     -- the ordered task list (one commit per sub-project)
  checklist.md                 -- every sub-project: PENDING / IN PROGRESS / DONE
  consistency-matrix.md        -- cross-unit types/commands/config/terms + conflicts
  incomplete-work-register.md  -- consolidated backlog (extends EI-3 audit)
  units/<name>.md              -- one Analysis_Record per sub-project
```

The three shared registers (`checklist.md`, `consistency-matrix.md`,
`incomplete-work-register.md`) are living documents: each per-sub-project pass
appends to them, and each per-sub-project commit includes the register rows it
added.

---

## 3. Analysis Waves (Sequencing)

Waves run in order; within a wave, order is by dependency then by size (largest
split-candidates analysed with most care). A unit is not started until its
upstream dependencies are analysed or explicitly deferred (Requirement 1.3).

### Wave 0 -- Foundations
platform-core, configuration-system, logging-subsystem, command-framework,
document-model, virtual-file-system, plugin-architecture, workflow-engine,
background-io, encoding-and-characters.

### Wave 1 -- Core editing and model
undo-redo-transactions, viewport-and-scrolling, caret-and-selection,
display-line-mapping, edit-operations, line-commands, navigation-commands,
find-and-replace, sequence-numbers, syntax-highlighting, exclude-show-filter,
whitespace-and-guides, auto-indentation, line-wrap-toggle, text-decorations.

### Wave 2 -- Catalog and dataset (largest cluster)
dataset-catalog, virtual-catalog-manager, dataset-allocator,
dataset-ownership-model, structure-catalog, record-selection-criteria,
tabs-and-mask.

### Wave 3 -- Shell, commands, menus, session
command-semantics, command-completion, command-palette, command-configurator,
menu-workspace, menu-and-statusbar, function-keys-and-history, shell-command,
startup-and-session, workspace-model.

### Wave 4 -- UI, panels, layout
layout-and-docking, multi-tab-editor, file-tree-panel, theme-and-appearance,
view-zoom, hex-display, notification-system, plugin-manager-ui, accessibility,
context-help, clipboard-operations, idle-processing, large-file-performance,
external-modification, file-operations.

### Wave 5 -- Emulators, tools, connectors, integrations
jes-emulator, idcams-emulator, jcl-resolver, database-tool,
compiler-toolchain-integration, batch-execution, asa-report-preview,
lua-macro-engine, language-service, custom-file-viewers, compare-and-merge,
global-search, structure-catalog (if not in Wave 2), connector-extensibility,
connector-local-fs, connector-network-fs, connector-ftp-sftp, connector-cloud,
connector-mainframe, fileforge-integration.

### Wave 6 -- Meta and consolidation
project-master, workbench-requirements-merge, ears-integration (review only --
these are analysis artifacts, not shippable units). Then finalize the three
registers and the revised master task ordering.

> The wave membership above is the initial plan; a sub-project may be re-slotted
> if the analysis of an earlier unit reveals a dependency. Any re-slot is noted
> in `checklist.md`.

---

## 4. Per-Sub-Project Analysis Record Template

Each `units/<name>.md` follows this template. Sections map directly to the
requirements criteria they satisfy.

```markdown
# Analysis Record -- <sub-project>

- Wave: <n>
- Backing crate(s): <crate list or "none / inline in ff-desktop">
- Upstream deps analysed: <yes/list or deferred reason>
- Prior artifacts consulted: <links to EI-3 / RR rows used>

## 1. Split Analysis (Req 2)
- Size: <requirement-line count>, responsibilities: <count + list>, crates: <count>
- Split_Candidate: YES / NO
- If NO: rationale (cohesive single responsibility, acceptable size)
- If YES: proposed Logical_Units (name -> single responsibility), and a
  Split_Plan table mapping every Requirement/criterion -> exactly one target unit
- Crate impact + downstream-reference impact; gate-required? YES/NO

## 2. Consistency and Conflicts (Req 3)
- Types/Command_IDs/config keys/panel names/terms this unit owns (feeds the matrix)
- Conflicts found (shared ownership, contradictions, dangling cross-refs) + proposed resolution

## 3. Completeness (Req 4)
- tasks.md state vs TCR rows vs actual code+tests -- discrepancies
- Classification: COMPLETE / PARTIAL / NOT STARTED
- Incomplete items -> rows added to incomplete-work-register.md
- Tracking fixes (marker corrections) with exact edit

## 4. Logging Audit (Req 5)
- ff-logging usage present? Levels appropriate (ERROR/WARN/INFO/DEBUG)?
- Swallowed errors / `let _ =` discards / println!/eprintln! in lib/bin -> list with file:line
- Two-phase init respected?
- Logging gaps -> rows added to incomplete-work-register.md

## 5. Task Revision (Req 6)
- Revised, dependency-ordered sequence for this unit's incomplete tasks
- Supersessions (task -> superseded by)
- project-master ordering delta

## 6. Decisions for Owner (Req 8.5)
- Any split / conflict reconciliation / new-requirement finding needing approval
```

---

## 5. Split-Decision Criteria (Req 2.1)

A Sub_Project is flagged a Split_Candidate when it meets two or more of:

1. **Size**: `requirements.md` exceeds ~350 lines OR more than ~12 numbered
   Requirements.
2. **Responsibilities**: it covers three or more clearly distinct responsibilities
   that could each stand alone (e.g. a "catalog" unit that also owns transactions,
   backup/restore, and security).
3. **Crates**: it is backed by two or more crates, or its criteria clearly target
   different crates.
4. **Concern clusters**: its requirements form groups with little cross-reference
   between groups (low internal cohesion).

Meeting one criterion is a note, not a split. Splitting is always a proposal
(Requirement 2.6); no split is executed here. Known likely candidates from the
size scan (to be confirmed by analysis): file-tree-panel, dataset-catalog,
virtual-catalog-manager, idcams-emulator, jes-emulator, startup-and-session,
database-tool.

---

## 6. Consistency Matrix Structure (Req 3.5)

`consistency-matrix.md` holds tables keyed by artifact kind, each row naming the
owning unit and every referencing unit; a Conflict is any row with more than one
claimed owner or a contradiction:

```
## Public Types        | Type | Owner unit | Referencing units | Conflict? | Resolution
## Command_IDs         | id   | Owner unit | Referencing units | Conflict? | Resolution
## Config Keys         | key  | Owner unit | Referencing units | Conflict? | Resolution
## Panels / Contexts   | name | Owner unit | Referencing units | Conflict? | Resolution
## Glossary Terms      | term | Owner unit | Definition source  | Conflict? | Resolution
## Dangling Cross-Refs | from-unit -> missing target | Correction
```

Where `docs/specs/project-master/api-consistency-report.md` and
`docs/reviews/requirements-review/terminology-map.md` already establish an owner,
the matrix cites them rather than re-deriving.

---

## 7. Completeness Method (Req 4)

For each unit, cross-check three sources and record any disagreement:

1. **tasks.md** -- count `[ ]` vs `[x]`; note parent/subtask marker mismatches.
2. **TCR.md** -- the unit's rows: PASS / MANUAL / NOT COVERED counts.
3. **Code + tests** -- does the backing crate exist; do the referenced test
   files/functions exist (grep), do they run under the clean `verify.ps1` baseline.

Classification:
- **COMPLETE** = all tasks `[x]` AND all criteria PASS/MANUAL in TCR AND tests exist.
- **PARTIAL** = any `[ ]` task, any NOT COVERED criterion, or a code/test discrepancy.
- **NOT STARTED** = no backing implementation.

Every PARTIAL/NOT-STARTED item and every discrepancy becomes a row in
`incomplete-work-register.md`, extending (not replacing) the EI-3 audit.

---

## 8. Logging Audit Checklist (Req 5)

Per backing crate, using read-only grep/read (no edits):

1. Does the crate depend on `ff-logging` and emit records? (grep for `ff_logging`,
   `log_warn!`, `log_error!`, `log_info!`, `log_debug!` / `log(`).
2. Error-return / error-swallow sites without a log record: grep for `Err(`,
   `?` at module boundaries, `let _ =`, `.ok()` discards, silent fallbacks.
3. `println!` / `eprintln!` / `print!` / `eprint!` in `src/` (excluding `tests/`
   and `tools/`).
4. Level appropriateness spot-check (ERROR for failures, WARN for degraded, INFO
   for state transitions, DEBUG for detail).
5. Two-phase-init respect (only relevant to ff-desktop startup / ff-logging).

Findings are recorded as one register entry per unit describing the gap and
suggested fix; NO source is changed (Requirement 5.6, 8.1).

---

## 9. Git Checkpoint Protocol (Req 7)

At the end of each Sub_Project's analysis:

1. Stage only that unit's files: `units/<name>.md`, and the appended rows in
   `checklist.md` / `consistency-matrix.md` / `incomplete-work-register.md`, plus
   any tracking-fix edits to that unit's `tasks.md` and the `project-master`
   ordering delta for it.
2. Commit: `analysis(<name>): project-analysis pass -- split/consistency/completeness/logging`.
3. Push to `main`.
4. Never stage unrelated in-progress code (Requirement 7.3). If the tree has
   interleaved changes, stage the analysis files explicitly by path.

A final consolidation commit (Requirement 7.4) records the completed registers
and the revised master ordering after Wave 6.

---

## 10. Handling Findings That Need a Decision (Req 8.5)

The analysis pass itself only records. When a finding implies a change:

- **Split proposal** -> recorded as a Split_Plan in the unit's record + a decision
  entry; execution is a later, separately-gated task.
- **Conflict reconciliation** (two owners, contradiction) -> recorded with a
  proposed single-owner resolution; applying it is a later change request.
- **Superseded task** -> recorded in the register; the marker edit is a tracking
  fix (no gate).
- **New requirement discovered** (behaviour with no criterion) -> logged to the
  change-log as a new CR and routed through the normal requirements gate; not
  written into a spec during analysis.

This keeps the analysis fast and safe while producing a concrete, owner-reviewable
set of proposals and a re-ordered backlog.
