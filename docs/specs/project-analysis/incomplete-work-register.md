# Incomplete Work Register

Consolidated, living backlog of every pending/incomplete task, orphaned
requirement, NOT-COVERED TCR row without a task, tracking-marker fix, and logging
gap found during the analysis (Requirement 4.3, 5.5).

This EXTENDS `docs/specs/ears-integration/incomplete-work-audit.md` (EI-3) rather
than replacing it. EI-3 established the pending phases and inconsistencies as of
that audit; this register carries them forward and adds new findings per
sub-project, then produces a dependency-ordered sequence in Wave 6.

Status: SEEDED from EI-3 (re-baselined after CR-NR-057, commit `b537737`);
populated per sub-project during Waves 0-5.

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

> Note: EI-3 predates several later phases (DB, DC-DE, CR-CH-015/016, CR-NR-054,
> CR-NR-057). Newer pending work (e.g. CR-NR-054 DF.7-DF.10 code, CR-NR-057
> command chaining / Context Navigation Stack, command-configurator Task 3
> desktop-adapter follow-ups, dockable Output_Panel view) will be added as their
> owning sub-projects are analysed in the relevant waves.

---

## New findings (added per sub-project during analysis)

| ID | Kind | Unit | Item | Proposed action | Wave |
|----|------|------|------|-----------------|------|
| _(none recorded yet -- re-baselined after CR-NR-057)_ | | | | | |
