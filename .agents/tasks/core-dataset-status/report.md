# Status Reconciliation: Simplify-the-Core, File Navigation, Dataset Volumes, Catalog

Read-only investigation, FileForgeWorkbench. Produced against the authoritative
status docs and verified against the crate tree. Where docs and code disagree it
is flagged with **[DOC/CODE DISAGREEMENT]**.

---

## Summary answer (read this first)

1. **Dataset VOLUMES work: specified and owner-approved, but ZERO code.** The
   first-class Volume layer (CR-NR-105 / CR-CH-057) passed its requirements gate
   (volume-model Req 1-11 authored, dataset-catalog Req 32 + tasks 33-37 added).
   No `ff-volume` crate exists; every volume task is `[ ]`. It is also blocked on
   three open owner UI decisions and a not-yet-run second (UI) gate.

2. **CATALOG work: substantially COMPLETE as a locator+storage subsystem.** The
   `ff-dscatalog` crate implements tasks 1-32 (DSN, repository, mount/unmount,
   CRUD, PDS, GDG, VFS provider, config, commands, LISTCAT/LISTDS, properties,
   context menus, UUID layout, StorageProvider, VSAM/ISAM providers, staged
   transactions, backup/restore, audit, security) -- all `[x]`. The only
   OUTSTANDING catalog work is the Volume split (tasks 33-37), which is the
   catalog side of the Volume effort above.

3. **Simplify-the-Core: effectively COMPLETE.** Phases 1 and 2 are owner-verified
   (full gate clean). Phase 3 (B080 dispatch unification) is done -- all Steps
   0-7 either implemented or deferred-to-CR-CH-053, and tasks 3.2/3.3 done. The
   only residual is three shell files slightly over the 400-line rule, which is a
   housekeeping refactor, not architectural work.

---

## (A) Simplify-the-Core -- complete? what remains

### Status: effectively COMPLETE (one minor housekeeping item)

**Evidence (docs):** `docs/status/RESUME-ffdesktop-simplification.md`
- Phase 1 (safe simplifications): "DONE, owner-verified", workflow
  wf_e26563bfd4c8c342 reviewer-APPROVED, full verify.ps1 clean. Deleted
  `panel_layout.rs` + superseded TabManager openers; split the 10,999-line
  `shell/tests.rs` into `tests_common.rs` + 8 modules (454 tests before = 454
  after).
- Phase 2 (shell structure): "FINAL STATUS: DONE" -- owner ran full gate, 9495
  tests pass. Task 2.1 (god-struct grouping into `state_groups.rs` +
  `state.rs`) and 2.2 (file splits to <=400 lines) both complete.
- Phase 3 = the B080 dispatch-unify migration. The task list marks Steps 0-7 all
  DONE or deferred: Steps 0-2 implemented (single front door
  `dispatch_command_string`); Steps 3-6 DEFERRED to CR-CH-053 (editor-action and
  window/standalone verbs that do not fit an existing `CommandTarget` variant
  behaviour-preservingly); Step 7 "done by prior removal". Tasks 3.2 (kind-state
  off `TabState` behind `TabKind`) and 3.3 (remove wiring-standard caveat) are
  `[x]`.

**Evidence (code -- verified against the tree):**
- The Phase 2 split files all exist in `crates/ff-desktop/src/shell/`:
  `state.rs`, `state_groups.rs`, `dispatch.rs`, `dispatch_ffedit.rs`,
  `update.rs` + `update_dialogs.rs` / `update_dialogs_catalog.rs` /
  `update_floating.rs` / `update_input.rs` / `update_keys.rs` /
  `update_startup.rs`, `render.rs` + `render_body.rs` / `render_chrome.rs` /
  `render_command_line.rs` / `render_nav*.rs` / `render_split*.rs` /
  `render_status.rs`, `commands.rs` + `commands_fastpath.rs` /
  `commands_ladder_a.rs` / `_b.rs` / `_c.rs` / `commands_menu.rs` /
  `commands_scrm.rs` / `commands_session.rs` / `commands_theme.rs`.
- **B080 unification confirmed in code.** `shell/dispatch.rs` documents and
  implements `dispatch_command_string` as the single front door: it runs the
  `=` prefix step once, then `run_command_prelude`, then `resolve_target`, then
  `run_command_ladder` -- the typed line and the key/menu FallThrough both route
  here. (`crates/ff-desktop/src/shell/dispatch.rs` module doc + method body.)
- **`commands_ladder_b2.rs` no longer exists** (not in the shell directory
  listing) -- confirms the resume's claim that CR-CH-053 E7 retired it.

### What remains for Simplify-the-Core

- **Three shell files exceed the 400 non-test-line rule** (`rust-standards.md`).
  Verified by running `tools/python/check_line_limits.py`
  (log: `tools/logs/linecheck.txt`):
  - `shell/construct.rs` -- 453
  - `shell/commands.rs` -- 446
  - `shell/state.rs` -- 432
  These are behaviour-preserving splits (no gate needed).

- **[DOC/CODE DISAGREEMENT]** The resume names FOUR pre-existing over-limit files
  (`main.rs` 632, `config_panel/render.rs` 590, `tab_state.rs` 404,
  `menu_workspace/loader.rs` 404) as the known residual. The checker (which walks
  the whole `ff-desktop/src` tree, not just `shell/`) reports NONE of those four
  today -- they are now at or under 400 non-test lines. Instead it finds three
  DIFFERENT shell files over the limit (above), which likely grew during the B080
  work after the resume was written. Net: the resume's "4 pre-existing" list is
  STALE; the real residual is the three files listed above.

- The non-blocking nits recorded in the resume (a pre-existing non-ASCII comment
  sweep; a few `pub(crate)` fields that could tighten to `pub(super)`; the
  ff-mdx-app/installer profile warnings) remain low priority.

**Bottom line:** the architectural simplification is done. What is left is a
~3-file line-limit refactor plus cosmetic nits -- maintenance, not design.

---

## (B) File navigation -- status + remaining

### Status: DONE for the core capability (modern explorer shipped)

**Evidence (docs):**
- `ROADMAP.md` Section 3, Core item 3 (File directory navigator): status
  **DONE** -- "modern ff-file-tree explorer, CR-NR-060; edit ops incl. catalog
  subtrees, B042".
- `project-master/tasks.md` Phase (navigation-modernization): the summary row is
  `[x]` -- "Modern ff-file-tree/NavModel explorer is the SOLE File Explorer
  (NAV.1-12): browse/open(files+datasets)/keyboard/Tab-focus/context-menu/
  multi-select/copy-as-text-tree/file-copy-paste ... DONE".
- `docs/specs/file-tree-panel/tasks.md`: tasks 1-26 are ALL `[x]` (crate
  scaffold, multi-root tree, async loading, rendering, file watching, context
  menus, keyboard nav, search/filter, dataset-catalog browsing, path bar,
  refresh, config, accessibility, command registration, integration tests, and
  the Phase AY-BM extensions through Requirement 23).
- Three navigation bugs from testing are FIXED (`docs/status/bugs.md`): B040
  (preview-toggle crash), B041 (catalog repository init), B042 (catalog-subtree
  edit ops).

**Evidence (code):** the navigation crates exist -- `ff-file-tree`,
`ff-file-explorer`, `ff-files-panel`, `ff-explorer-view`, `ff-nav-model`,
`ff-navigation-commands`, plus `ff-vfs` and `ff-connector-local-fs`.

### What remains for file navigation

- **Slice B (mainframe qualifier/dataset duality)** is explicitly deferred: the
  navigation-modernization phase note says "Mainframe qualifier/dataset duality
  (D2) is Slice B" -- Slice A (CR-NR-060) is the DONE part. Slice B is not yet
  gated/scheduled.
- **Volume surfacing in the tree.** Once the Volume layer lands (section C), the
  explorer will need a VTOC/volume view and volume nodes. `virtual-catalog-manager`
  has NO volume content yet (grep for "volume/VTOC/picker" returned no matches),
  so this is future UI-gate work, not built.
- No open file-navigation bugs found; the three testing bugs are all FIXED.

**Bottom line:** core file navigation is complete and is the sole explorer.
Remaining navigation work is downstream of the Volume effort (VTOC/volume nodes)
and the deferred mainframe-duality Slice B.

---

## (C) Dataset VOLUME emulation -- status + remaining + open owner questions

### Status: GATE AUTHORED + owner-APPROVED direction; NO code yet

**Evidence (docs):** `docs/status/RESUME-volume-model.md` and
`docs/status/change-log.md` (CR-NR-105 / CR-CH-057, both **IN PROGRESS -- gate
authored; owner approved direction**).
- The owner-approved model is "split, not rename": promote today's physical
  Repository into a first-class **Volume** (new thin `ff-volume` crate), and
  reduce the Catalog to a pure DSN -> (Volume, locator) metadata locator.
- Deliverables ALREADY authored (docs-only): `docs/specs/volume-model/`
  {requirements (Req 1-11), design, tasks (Tasks 1-10)}; dataset-catalog Req 32
  (32.1-32.8) + tasks 33-37; dataset-ownership-model Req 21 + Req 7.7;
  project-master Phase (volume-model) VM.1-VM.5; TCR NOT COVERED rows.

**Evidence (code -- verified):**
- **`ff-volume` crate does NOT exist.** `file_search` for "ff-volume" returned no
  files; it is absent from the `crates/` directory listing. Consistent with the
  resume ("Crate NOT created yet").
- `docs/specs/volume-model/tasks.md`: all 10 tasks are `[ ]` (the file header
  states "These are FUTURE tasks ... All tasks are `[ ]`").
- `project-master/tasks.md` Phase (volume-model): VM.1-VM.5 are all `[ ]`,
  including VM.1 the gate itself (authored but the master row is still `[ ]`).

### Open owner questions (BLOCKING -- from RESUME-volume-model.md)

These were deliberately left to a future (UI) gate and are NOT yet answered:
- **A. Volume UI shape:** dedicated Volume management context vs one shared
  dialog? Kiro recommended a dedicated Volume WorkspaceContext (VTOC/volume list
  + DEFINE VOLUME / vary online-offline / set RW-RO) plus a volume PICKER in the
  catalog-creation dialog. **OWNER NOT CONFIRMED.**
- **B. Allocation model:** confirm the two-level allocation -- DEFINE VOLUME
  (admin: VOLSER + host path + capacity + status) vs automatic SPACE-on-volume
  allocation charged as extents. Clarified by Kiro; **owner confirmation
  pending.**
- **C. Capacity:** hard cap fixed at DEFINE time (resizable later, no
  over-commit) vs elastic? Kiro recommended hard cap. **OWNER NOT CONFIRMED.**

Owner decisions ALREADY settled (do not re-litigate): tracks/cylinders/extents
are metadata-only (capacity constraint + reporting); SPACE in TRK/CYL/block;
primary + up to configurable max secondary extents (default 16), exhaustion =
x37-style failure; Volume has its own capacity, Volume_Full is a distinct
failure; `ff-volume` owns the Volume entity and `ff-dscatalog` depends on it (never
the reverse); many-catalogs-per-volume / many-volumes-per-dataset / uncataloged
all permitted; Volume is visible-but-advanced in the UI.

### What remains for Volume emulation (in order)

1. Get owner answers to open questions A, B (confirm), C.
2. Run the SECOND (UI/flow) requirements gate, docs-only: `virtual-catalog-manager`
   (dedicated Volume management context + catalog-dialog volume picker),
   `dataset-allocator` (SPACE-against-volume + VOL=SER+UNIT uncataloged), possibly
   `idcams-emulator` (DEFINE VOLUME).
3. Code build via project-master phases:
   - VM.2 `ff-volume` crate (Volume entity, VOLSER uniqueness, status/access,
     geometry, byte/track/cyl conversions) -- volume-model Req 1-3.
   - VM.3 SPACE + extents + both failures -- Req 4-7.
   - VM.4 reporting/VTOC + DatasetVolume + uncataloged + DEFINE VOLUME -- Req 8-11.
   - VM.5 dataset-catalog integration (schema v4, resolution indirection,
     dual-read bytes-never-move migration, depend on ff-volume) -- dataset-catalog
     Req 32; dataset-ownership-model Req 21 / Req 7.7.
   (Before VM.2: confirm the default `bytes_per_track` constant -- design uses
   ~56664 in the worked example -- and re-confirm the `ff-volume` crate per the
   ownership ADR.)

**Bottom line:** Volume emulation is fully specified and approved in principle,
but not started. It is BLOCKED on three owner UI decisions and a second gate
before any code.

---

## (D) Catalog emulation -- status + remaining

### Status: substantially COMPLETE; only the Volume split remains

**Evidence (docs):** `docs/specs/dataset-catalog/tasks.md`
- Tasks **1-16** all `[x]`: SQLite schema, DSN validation, repository layout,
  mount/unmount + registry, create/remove/export/import, dataset CRUD, PDS
  members, GDG management, VFS provider (scheme `catalog`), configuration
  integration, commands registration, LISTCAT/LISTDS, properties provider,
  context menus, type-consistency + access-date, integration tests. 13
  property-based tests defined.
- CR-NR-016 tasks **17-30** all `[x]`: record codecs (Fixed/Variable/Binary/
  Text), StorageProvider trait + NativeFileProvider, UUID-based physical layout,
  VSAM KSDS/RRDS/ESDS + ISAM providers, staged transaction protocol with startup
  recovery, integrity/backup/restore/diagnose/reconcile, audit trail + schema
  migrations, security hardening, master-catalogue hierarchy, record-oriented
  editor integration.
- CR-NR-017 task **32** (CatalogLocation Local/Remote discriminant) all `[x]`.
- Task **31** (resolve pre-BS requirements inconsistencies, incl. crate-name fix
  `ff-dataset-catalog` -> `ff-dscatalog`) all `[x]`.
- Catalog-related bugs FIXED: B041 (repository init at catalog creation + mount
  self-heal), B042 (catalog-subtree edit ops).

**Evidence (code -- verified):** the catalog subsystem crates exist:
`ff-dscatalog`, `ff-dataset-catalog`, `ff-catalog-registry`, `ff-catalog-dialog`,
`ff-dsalloc`, `ff-dataset-alloc-dialog`, `ff-idcams`, plus `ff-vfs`.

- **[DOC/CODE DISAGREEMENT -- minor/naming]** The task file was authored against a
  crate scaffolded as `ff-dataset-catalog` (tasks 1.1-1.2), then task 31.3/31.4
  renamed references to `ff-dscatalog` to match the real workspace crate. The
  tree currently contains BOTH `ff-dscatalog` AND `ff-dataset-catalog`
  directories. This did not change my status conclusion (the authoritative crate
  is `ff-dscatalog`, confirmed by the dependency references in the volume tasks),
  but the presence of two similarly-named directories is worth a cleanup check so
  a stale/duplicate crate is not left in the tree.

### What remains for catalog emulation

- **CR-CH-057 tasks 33-37 (the Volume split) -- all `[ ]`:**
  - 33 schema v4 (`volumes` + `dataset_volumes` tables, forward migration seeding
    a Volume per Repository, moving no bytes),
  - 34 resolution via DatasetVolume indirection + Volume online check,
  - 35 depend on `ff-volume`,
  - 36 multivolume / shared-volume / uncataloged (VOL=SER+UNIT),
  - 37 migration/resolution/uncataloged/cardinality tests.
  These are the catalog side of the Volume effort (section C) and are blocked on
  the same `ff-volume` crate and owner decisions.

**Bottom line:** the catalog is built and tested as a locator+storage subsystem.
The only outstanding catalog work is to turn it into a PURE locator by landing
the Volume split (tasks 33-37), which depends on VM.2 `ff-volume` first.

---

## Cross-cutting documentation note (staleness)

- **`docs/status/current-work.md` is STALE.** Its "Active work item" still reads
  "Phase CV-impl -- POM Redesign implementation" and its work table stops at
  Phase CV/CW. The product has moved far past this (B080 dispatch unification,
  CR-CH-053 Command Environments, CR-CH-056 egui theme rework, CR-NR-098 SCRM
  waves 0-3, CR-NR-103 localization Phase 1, the volume-model gate). Treat
  `ROADMAP.md` + the `RESUME-*.md` files + `project-master/tasks.md` as
  authoritative; do NOT rely on `current-work.md` for current state.

---

## Prioritized NEXT STEPS

Distinguishing docs-gate work from code work, and noting owner-decision blocks.

### Blocked on owner decisions (resolve first -- cheap, unblocks the biggest item)
1. **Answer Volume open questions A, B (confirm), C** (RESUME-volume-model.md).
   This unblocks the entire Volume + catalog-split effort. No code or docs needed
   from the owner -- just three decisions.

### Docs-gate work (no source changes; reversible)
2. **Run the SECOND (UI/flow) requirements gate** for the Volume UI once A/B/C are
   answered: `virtual-catalog-manager` (Volume management context + catalog-dialog
   volume picker), `dataset-allocator` (SPACE-against-volume, VOL=SER+UNIT),
   optionally `idcams-emulator` (DEFINE VOLUME). ASCII-only, per workflow gate.
3. **Refresh `current-work.md`** (or formally retire it in favour of ROADMAP +
   RESUME files) so the dashboard stops pointing at Phase CV. Docs-only.

### Code work -- Simplify-the-Core (behaviour-preserving, no gate)
4. **Split the three over-limit shell files** (`construct.rs` 453, `commands.rs`
   446, `state.rs` 432) back under 400 non-test lines. Update the resume's stale
   "4 pre-existing files" note while doing so.
5. **Investigate the `ff-dscatalog` vs `ff-dataset-catalog` duplicate directory**
   and remove/merge the stale one (confirm references first; this is cleanup,
   not a feature).

### Code work -- Volume + catalog split (gated; starts only after step 2 + owner TASK instruction)
6. **VM.2** build `ff-volume` crate (Volume entity, VOLSER uniqueness,
   status/access, geometry, conversions) -- volume-model Req 1-3. Confirm the
   default `bytes_per_track` constant first.
7. **VM.3** SPACE unit model + primary/secondary extents + max-extents + the two
   distinct failures (dataset x37-style and Volume_Full) -- Req 4-7.
8. **VM.4** reporting/VTOC_View + DatasetVolume association + multivolume +
   uncataloged (VOL=SER+UNIT) + DEFINE VOLUME contract -- Req 8-11.
9. **VM.5 / dataset-catalog tasks 33-37** schema v4, resolution via DatasetVolume
   indirection + online check, dual-read bytes-never-move migration, depend on
   `ff-volume` -- dataset-catalog Req 32; dataset-ownership-model Req 21 / 7.7.

### Downstream (after Volume lands)
10. **File-navigation:** add the VTOC/volume view and volume nodes to the explorer
    (future UI gate), and schedule the deferred Slice B mainframe
    qualifier/dataset duality (CR-NR-060 Slice B).

### Verification method note
All Volume/catalog code work uses TDD and SCOPED checks only
(`cargo check/test/clippy -p ff-volume` / `-p ff-dscatalog`), then hands off the
full `cargo gate --build` to the owner -- per `testing.md`. No `--workspace`
runs from the agent.

---

## Evidence index (files cited)

- `docs/project-management/ROADMAP.md` -- CORE vs PLUGIN split; Core item 3 DONE.
- `docs/status/RESUME-ffdesktop-simplification.md` -- Phases 1/2 DONE; Phase 3
  B080 Steps 0-7.
- `docs/status/RESUME-volume-model.md` -- gate authored, owner-approved, open
  questions A/B/C, next actions.
- `docs/status/change-log.md` -- CR-NR-105 / CR-CH-057 IN PROGRESS; CR-NR-060.
- `docs/status/bugs.md` -- B040/B041/B042 FIXED.
- `docs/status/current-work.md` -- STALE (stops at Phase CV/CW).
- `docs/specs/volume-model/tasks.md` -- Tasks 1-10 all `[ ]`.
- `docs/specs/dataset-catalog/tasks.md` -- Tasks 1-32 `[x]`; tasks 33-37 `[ ]`.
- `docs/specs/file-tree-panel/tasks.md` -- Tasks 1-26 `[x]`.
- `docs/specs/virtual-catalog-manager/requirements.md` -- no volume content yet.
- `docs/project-management/project-master/tasks.md` -- Phase (volume-model)
  VM.1-VM.5 all `[ ]`; Phase (navigation-modernization) `[x]`.
- `crates/` directory listing -- `ff-volume` ABSENT; catalog/nav crates present;
  both `ff-dscatalog` and `ff-dataset-catalog` present.
- `crates/ff-desktop/src/shell/` listing -- Phase 2 split files present;
  `commands_ladder_b2.rs` ABSENT.
- `crates/ff-desktop/src/shell/dispatch.rs` -- single front door
  `dispatch_command_string` (B080) confirmed.
- `tools/python/check_line_limits.py` + `tools/logs/linecheck.txt` -- 3 shell
  files over 400 lines (construct.rs 453, commands.rs 446, state.rs 432).
