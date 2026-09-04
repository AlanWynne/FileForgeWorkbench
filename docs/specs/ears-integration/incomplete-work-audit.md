# Incomplete Work Audit
# EI-3 Output -- Pending Tasks, Orphaned Requirements, and TCR Gaps

**Status:** EI-3 complete -- analysis only, no source files modified
**Input:** project-master/tasks.md, dataset-catalog/tasks.md,
           virtual-catalog-manager/tasks.md, docs/quality/TCR.md,
           all pending sub-project requirements.md files
**Purpose:** Identify all pending/incomplete work that must be understood
before EI-4 (integration plan) assigns new phase labels and sequences
new EARS-derived requirements.

---

## Section 1: Pending Phases in project-master/tasks.md

The following phases have one or more `[ ]` tasks as of the EI-3 audit.

### Phase BS -- Mainframe Dataset Architecture (CR-NR-016)

**Sub-project:** dataset-catalog
**Status:** Wave 1-2 complete (BS.1-BS.7 all `[x]`); Wave 3-4 pending

| Task | Description | Dependency |
|------|-------------|------------|
| BS.8 | Staged transaction protocol -- OperationJournal, staged create/delete, startup recovery | Wave 2 complete |
| BS.9 | Integrity, backup, restore -- checksums, workspace commands | BS.8 |
| BS.10 | Catalogue audit trail + schema migrations | BS.9 |
| BS.11 | Security hardening -- parameterised SQL audit, log scrubbing, path-traversal PBT | BS.10 |
| BS.12 | Master/user catalogue hierarchy, logical rename, scoped uniqueness | BS.11 |
| BS.13 | Record-oriented editor integration -- wire codecs into open/save path | BS.12 |
| BS.14 | Non-functional validation -- cross-platform, performance, Git-compat, data-fidelity | BS.13 |
| BS.15 | Update dataset-catalog/design.md for CR-NR-016 | BS.14 |

**Note:** Task 21 in dataset-catalog/tasks.md (VSAM ESDS) has subtasks 21.1-21.4
all marked `[x]` but the parent task `[ ]` marker was not updated. This is a
tracking inconsistency -- the parent should be `[x]`. No code work needed.

### Phase BU -- SQLite Catalog Integration (CR-CH-006)

**Sub-project:** virtual-catalog-manager
**Status:** Design docs complete (BU.D1-BU.D4 `[x]`); all implementation pending

| Task | Description | Dependency |
|------|-------------|------------|
| BU.1 | Design docs updated (DONE -- BU.D1-BU.D4 marked `[x]` in VCM tasks.md) | -- |
| BU.2 | Failing tests written -- CatalogRegistry API, resolve_and_open_dataset, content area | BS.4 (SQLite provider) |
| BU.3 | CatalogRegistry::allocate() and list_datasets() implemented and tests green | BU.2 |
| BU.4 | AllocOutcome::Confirmed handler wired to SQLite | BU.3 |
| BU.5 | Files Panel content area reads from SQLite | BU.4 |
| BU.6 | File Explorer Panel Mainframe content reads from SQLite | BU.5 |
| BU.7 | resolve_and_open_dataset() replaces resolve_dataset_path() | BU.6 |
| BU.8 | AllocatedDataset struct, datasets HashMap, and TOML persistence removed | BU.7 |
| BU.9 | TCR.md and project-master updated; cargo test --workspace green | BU.8 |

**Dependency note:** BU requires BS.4 (SqliteRecordProvider) to be complete.
BS.4 is `[x]`. However BU also requires BS.8 (staged transactions) for
production-quality allocation. BU can proceed with BS.4 only for the
basic allocate/list path; staged transactions are a BS.8 concern.

### Phase BV -- Catalog Location Discriminant (CR-NR-017)

**Sub-project:** dataset-catalog
**Status:** All pending

| Task | Description | Dependency |
|------|-------------|------------|
| BV.1 | CatalogLocation enum + CatalogMount refactor (Tasks 32.1-32.8) | None (pure refactor) |

**Note:** BV.1 is a pure refactor with no behaviour change for local catalogs.
It has no dependency on BS Wave 3-4 or BU. It can proceed independently.

### Phase VCM Task 17 -- Dataset file creation on first open

**Sub-project:** virtual-catalog-manager
**Status:** Pending (Tasks 17.1-17.8 all `[ ]`)

| Task | Description |
|------|-------------|
| 17.1 | Failing test: opening missing dataset creates file and parent dirs |
| 17.2 | Failing test: opening missing dataset creates parent dirs |
| 17.3 | Add create_dataset_file() pure helper |
| 17.4 | Wire into Files Panel OpenFile handler |
| 17.5 | Wire into File Explorer double-click handler |
| 17.6 | cargo test green |
| 17.7 | cargo clippy clean |
| 17.8 | Update TCR.md Req 16.3 and 16.6 rows |

**Note:** This task is superseded by BU.7 (resolve_and_open_dataset replaces
resolve_dataset_path). Task 17 implements the legacy DSN-path approach;
BU.7 implements the SQLite-backed approach. Task 17 should be marked
SUPERSEDED BY BU.7 rather than implemented as written.

---

## Section 2: Task Tracking Inconsistencies

The following inconsistencies were found between task markers and actual
completion state. These are documentation fixes only -- no code changes needed.

| File | Issue | Resolution |
|------|-------|------------|
| dataset-catalog/tasks.md task 21 | Parent `[ ]` but all subtasks 21.1-21.4 are `[x]` (ESDS provider) | Mark parent `[x]` |
| project-master/tasks.md BU.1 | Listed as `[ ]` but BU.D1-BU.D4 in VCM tasks.md are all `[x]` | Mark BU.1 `[x]` |
| project-master/tasks.md Phase BD | Appears twice -- once as a status table (with all `🔴`) and once as a completed phase with `[x]` tasks | Remove the duplicate `🔴` status table; the `[x]` entries are correct |

---

## Section 3: Requirements Without Corresponding Tasks

These are requirements in existing specs that have no task in the
sub-project tasks.md to implement them. They represent gaps where
requirements were written but implementation was never planned.

### dataset-catalog/requirements.md

| Requirement | Status | Gap |
|-------------|--------|-----|
| Req 16-30 (CR-NR-016) | Tasks 17-30 exist in tasks.md | COVERED -- tasks exist |
| Req 31 (CR-NR-017) | Task 32 exists in tasks.md | COVERED -- task exists |

No gaps found in dataset-catalog. All requirements have corresponding tasks.

### virtual-catalog-manager/requirements.md

| Requirement | Status | Gap |
|-------------|--------|-----|
| Req 1-12 | Tasks 1-12 exist | COVERED |
| Req 13 (revised by CR-CH-006) | Tasks 18-27 (BU phase) exist | COVERED |
| Req 14 | Tasks 14.1-14.11 exist | COVERED |
| Req 15 | Tasks 15.1-15.3 exist | COVERED |
| Req 16 (revised by CR-CH-006) | Tasks 19, 25 (BU phase) exist | COVERED |

No gaps found in virtual-catalog-manager.

### Other sub-projects (all phases complete)

All other sub-projects (edit-operations, find-and-replace, undo-redo-transactions,
syntax-highlighting, tabs-and-mask, sequence-numbers, hex-display, menu-and-statusbar,
startup-and-session, function-keys-and-history, FFW-JES, command-semantics,
line-commands, navigation-commands, lua-macro-engine) have all tasks marked `[x]`
in project-master/tasks.md. No requirement-without-task gaps were found in these
sub-projects for their currently specified requirements.

---

## Section 4: Tasks Referencing Non-Existent or Renumbered Requirements

Checked all pending tasks (BS.8-BS.15, BU.1-BU.9, BV.1, VCM Task 17) against
their referenced requirements.

| Task | Referenced Req | Status |
|------|---------------|--------|
| BS.8 | dataset-catalog Req 25 | EXISTS -- Req 25 Staged Transaction Protocol |
| BS.9 | dataset-catalog Req 26 | EXISTS -- Req 26 Integrity, Backup, Restore |
| BS.10 | dataset-catalog Req 27 | EXISTS -- Req 27 Catalogue Reconciliation |
| BS.11 | dataset-catalog Req 28 | EXISTS -- Req 28 Security |
| BS.12 | dataset-catalog Req 29 | EXISTS -- Req 29 Catalogue Hierarchy |
| BS.13 | dataset-catalog Req 16 | EXISTS -- Req 16 Record-Oriented Storage |
| BS.14 | dataset-catalog Req 30 | EXISTS -- Req 30 Non-Functional |
| BS.15 | design.md update | N/A |
| BU.2-BU.9 | VCM Req 13, 16 (revised) | EXISTS -- revised by CR-CH-006 |
| BV.1 | dataset-catalog Req 31 | EXISTS -- Req 31 CatalogLocation |
| VCM Task 17 | VCM Req 16.3, 16.6 | EXISTS but SUPERSEDED by BU.7 |

No orphaned task references found.

---

## Section 5: TCR.md Rows That Are NOT COVERED With No Corresponding Task

Scanned TCR.md for all `🔴 NOT COVERED` rows and checked whether a pending
task exists to address each one.

### ff-dscatalog -- BS Wave 3-4 criteria (all 🔴)

All `🔴` rows for ff-dscatalog Req 16-30 (excluding Req 21-24 which are
partially covered by BS.4-BS.7) map to tasks BS.8-BS.15. COVERED by pending tasks.

### ff-vfs -- BS-related criteria (all 🔴)

All `🔴` rows for ff-vfs Req 9-12 (StorageProvider, POSIX, staged transactions,
backup/restore) have no corresponding task in any tasks.md.

**GAP IDENTIFIED:** ff-vfs requirements added by CR-NR-016 have no tasks.

| TCR Row | Requirement | Missing task |
|---------|-------------|-------------|
| ff-vfs Req 9.1-9.5 | StorageProvider trait in ff-vfs | No task in virtual-file-system/tasks.md |
| ff-vfs Req 10.1-10.6 | POSIX files as native objects | No task |
| ff-vfs Req 11.1-11.5 | VFS staged transaction protocol | No task |
| ff-vfs Req 12.1-12.5 | workspace.backup/restore/reconcile/diagnose | No task |

**Resolution:** These ff-vfs requirements are architectural extensions that
depend on BS Wave 3-4 completion. They should be added to
virtual-file-system/tasks.md as a new task group, sequenced after BS.15.
This is a task gap to be resolved in EI-4 or as a standalone gate before
the relevant BS wave.

### ff-desktop -- Phase BD duplicate 🔴 table

TCR.md contains a duplicate `🔴` status table for Phase BD (Req 19.1-19.10)
that was not removed when the implementation was completed. The `[x]` entries
in the Phase BD final status section are correct. The duplicate `🔴` table
is a documentation artefact.

**Resolution:** Remove the duplicate `🔴` table from TCR.md (documentation fix).

### ff-desktop -- Phase BU criteria (all 🔴)

All `🔴` rows for BU (Req 13.1-13.5, 16.1-16.6) map to BU.2-BU.9 tasks.
COVERED by pending tasks.

### ff-desktop -- Phase BV criteria (all 🔴)

All `🔴` rows for BV (Req 31.1-31.9) map to BV.1 task.
COVERED by pending task.

### Summary of TCR gaps

| Gap | Severity | Action |
|-----|----------|--------|
| ff-vfs Req 9-12 have no tasks | Medium | Add tasks to virtual-file-system/tasks.md before BS Wave 3 |
| Duplicate Phase BD 🔴 table in TCR.md | Low | Remove duplicate table (doc fix) |
| dataset-catalog task 21 parent marker | Low | Mark `[x]` (doc fix) |
| project-master BU.1 marker | Low | Mark `[x]` (doc fix) |

---

## Section 6: Dependency Order for Pending Phases

The correct execution order for all pending phases, accounting for dependencies:

```
BV.1 (CatalogLocation refactor)
  -- no dependencies; can start immediately

BS.8 (staged transactions)
  -- depends on BS.7 (ISAM) which is [x]

BS.9 (integrity/backup/restore)
  -- depends on BS.8

BS.10 (audit trail + migrations)
  -- depends on BS.9

BS.11 (security hardening)
  -- depends on BS.10

BS.12 (master/user catalogue hierarchy)
  -- depends on BS.11

BS.13 (record-oriented editor integration)
  -- depends on BS.12

BS.14 (non-functional validation)
  -- depends on BS.13

BS.15 (design.md update)
  -- depends on BS.14

ff-vfs tasks (new -- to be created)
  -- depends on BS.15

BU.2-BU.9 (SQLite catalog integration)
  -- depends on BS.4 (done) for basic path
  -- depends on BS.8 for staged-transaction-safe allocation
  -- recommend sequencing after BS.8 at minimum

EI-5 batches (new EARS requirements)
  -- B01-B11 (P1): can start after BV.1 is complete
  -- B12-B16 (P2): can start after B01-B11 are complete
  -- No dependency on BS Wave 3-4 for most batches
  -- B10 (command-semantics TSO commands) has no dependency on BS
  -- B11 (FFW-JES SDSF) has no dependency on BS
```

---

## Section 7: Recommended Documentation Fixes (No Gate Required)

These are pure documentation corrections with no observable behaviour change.
They can be applied immediately without a requirements gate.

1. **dataset-catalog/tasks.md task 21:** Change `- [ ] 21. Native file provider -- VSAM ESDS`
   to `- [x] 21. Native file provider -- VSAM ESDS` (all subtasks are `[x]`).

2. **project-master/tasks.md BU.1:** Change `- [ ] BU.1` to `- [x] BU.1`
   (design docs BU.D1-BU.D4 are all `[x]` in VCM tasks.md).

3. **TCR.md:** Remove the duplicate Phase BD `🔴` status table (the one that
   appears before the "Phase BE" section). The final status table with `[x]`
   entries is the correct record.

4. **virtual-catalog-manager/tasks.md Task 17:** Add a note:
   `SUPERSEDED BY BU.7 -- do not implement; resolve_and_open_dataset() covers this`.

---

## Section 8: Impact on EI-4 (Integration Plan)

The following constraints from this audit must be respected in EI-4:

1. **New EARS phases (BW onwards) do not block on BS Wave 3-4.** The EARS
   integration work (edit-operations, line-commands, command-semantics, FFW-JES,
   etc.) is independent of the mainframe dataset architecture work. Both streams
   can proceed in parallel.

2. **BV.1 should be completed before any EI-5 batch that touches ff-dscatalog.**
   BV.1 is a pure refactor with no gate required. It should be done first to
   avoid merge conflicts.

3. **BU requires BS.8 for production-quality allocation.** EI-4 should sequence
   BU after BS.8, not after BS.15. The basic allocate/list path (BU.2-BU.5) can
   proceed after BS.4 (done), but the full BU should wait for BS.8.

4. **ff-vfs tasks need to be created.** Before BS Wave 3 begins, a new task
   group should be added to virtual-file-system/tasks.md covering Req 9-12.
   This is a gap that EI-4 should flag as a prerequisite for BS.8.

5. **EI-5 batch B11 (FFW-JES SDSF) is the largest single batch (43 criteria).**
   EI-4 should consider splitting B11 into two sub-batches:
   - B11a: SDSF panel framework + job queue panels (P1 core)
   - B11b: SDSF filter/sort/search + SET commands (P1 extended)

---

## Summary

| Category | Count | Action |
|----------|------:|--------|
| Pending phases with `[ ]` tasks | 3 (BS Wave 3-4, BU, BV) | Sequence in EI-4 |
| Task tracking inconsistencies | 3 | Doc fixes (no gate) |
| Requirements without tasks | 0 | None |
| Tasks with orphaned requirement refs | 0 | None |
| TCR gaps (no task for 🔴 row) | 1 (ff-vfs Req 9-12) | Add tasks before BS.8 |
| TCR documentation artefacts | 1 (duplicate BD table) | Doc fix |
| EI-4 constraints identified | 5 | See Section 8 |
