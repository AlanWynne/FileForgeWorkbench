# RESUME -- Dataset Rationalisation + Simplify-Core (session end 2026-10-08)

Saved at owner's request ("going to bed"). This captures the full state of the
work done this session so the next session can resume cleanly.

## ONE-LINE STATUS

RC.A (Mainframe Dataset Stack Rationalisation, CR-CH-059) and the earlier shell
file-size split are BOTH DONE -- owner-confirmed: the full `cargo gate --build`
ran CLEAN on 2026-10-08 22:41 (run id db515797; fmt/clippy/build/test/app-build
all ok; 9648 tests pass, 0 fail; empty gate.review.log). The ff-nav-model
build-phase failure was fixed before that run. Both bodies of work are still
UNCOMMITTED on `main` (owner's working style; commit at owner's discretion).
NEXT: nothing required -- RC.B is NOT auto-started (PLUGIN phase 2, gated on CORE
sign-off); it begins only on an explicit owner "start RC.B".

## WHERE THE WORK LIVES

- Branch: `main`, main workspace root `c:\workspace\VSC\FileForgeWorkbench`.
- NO worktree, NO branch, NO commit (project's current working style; everything
  uncommitted for owner review). The `.worktrees/vsam-wiring` stream is ON HOLD
  -- REDIRECT and was NOT touched.

## WHAT WAS DONE THIS SESSION (in order)

1. Status investigation (`.agents/tasks/core-dataset-status/report.md`):
   simplify-core effectively complete; catalog built (ff-dscatalog tasks 1-32);
   volumes specified but zero code; file navigation DONE.
2. Volume UI gate (CR-NR-105 second gate): authored docs-only, verified, then
   APPROVAL HELD -- deferred into the rationalisation plan (see change-log
   CR-NR-105 note). Owner answered the three open questions A/B/C (dedicated
   Volume management context + picker; two-level allocation; hard-cap capacity),
   recorded in `RESUME-volume-model.md`.
3. Duplicate-crate investigation (`.agents/tasks/dscatalog-duplicate/report.md`):
   ff-dscatalog is LIVE; ff-dataset-catalog is a trait-only governance fixture;
   removal is a gated consolidation, NOT a drop-in delete.
4. Vision-fit evaluation (`.agents/tasks/dataset-vision-fit/report.md`): the work
   fits the owner's vision in shape (~70-80%) but the two-layer seam is blurred,
   there is no single DatasetAccess interface, and ff-volume is unbuilt. The real
   problem is duplication/drift, not a missing design. Recommends consolidation.
5. Rationalisation requirements gate (CR-CH-059): DOCS-ONLY, authored +
   conformance-APPROVED, owner-APPROVED as the final plan (pending implementation).
   Summary at `.agents/tasks/dataset-rationalisation-gate/gate-summary.md`.
   - vsam-wiring (V) stream REDIRECTED (its ff-vsam-services work accepted as
     throwaway; VSAM to be wired under DatasetAccess at RC.B.7).
6. RC.A IMPLEMENTED (this is the code that is uncommitted on `main`):
   - RC.A.1 docs: ADR-001 names ff-dscatalog authority; crate-name fixes
     (jes-emulator, ff-dsalloc catalog_bridge.rs doc comment -- BOTH mentions now
     fixed, incl. the one the reviewer flagged as missed at catalog_bridge.rs:83).
   - RC.A.2 code: new reconciled object-safe CatalogService/DynCatalogService
     (`crates/ff-dscatalog/src/service.rs`) + VsamService/VsamCluster
     (`crates/ff-dscatalog/src/vsam_service.rs`) over ff-dscatalog's own
     Dsorg{PS,PO,GDG}/Recfm/Dsn; ff-governance-tests repointed off both trait
     crates (Cargo.toml + tests/mock_compilation.rs).
   - RC.A.3 code: all five storage backends (native/esds/isam/rrds/sqlite_record)
     reimplemented onto the single `ff-vfs::StorageProvider` seam; the duplicate
     `ff-dscatalog::storage::StorageProvider` TRAIT deleted; native.rs tests split
     to native_tests.rs for the 400-line rule.
   - RC.A.4 code: ff-posix-provider reduced to path helpers + a re-export of
     `ff-vfs::PosixNativeProvider` (the sole posix registrant); ff-desktop nav
     call sites switched.
   - ADDITIVE-FIRST honoured: only the duplicate TRAIT deleted; ff-dataset-catalog
     and ff-vsam-services crates + their root Cargo.toml member lines still exist
     (their deletion is RC.B.7/RC.B.8, NOT done).
   - TCR flipped to PASS: ownership Req 22.1-22.7, VFS 13.1-13.5 + 14.1-14.4,
     dataset-catalog 33.1-33.7 + 35.1 + 35.4; jes Req 19.1-19.5 -> MANUAL (JES
     deferred). RC.A tasks marked [x] in the specs + project-master.
   - Review: APPROVED (`.agents/tasks/dataset-rationalisation-gate/rca-review.md`),
     two non-blocking findings, both now resolved/benign (the missed doc comment
     is fixed; esds.rs/sqlite_record.rs >400 lines are PRE-EXISTING at HEAD,
     out of RC.A scope).

## THE FULL-GATE FAILURE + FIX (the only open loop)

- Owner ran `cargo gate --build`. fmt + clippy clean; BUILD phase FAILED: the
  workspace-wide test-binary build hit E0599 in `crates/ff-nav-model/src/lib.rs`
  (5 call sites) -- `ff_posix_provider::PosixProvider::new(path, false).expect(...)`
  no longer valid because RC.A.4 made the re-exported `PosixNativeProvider::new`
  INFALLIBLE (`-> Self`, not `-> Result`). The scoped `-p` checks missed it
  because ff-nav-model's tests were not in the scoped set -- exactly what the full
  gate exists to catch.
- FIX APPLIED (this session): dropped the `.expect("provider")` line at all 5
  sites in `crates/ff-nav-model/src/lib.rs` (test module only; no production
  change). The path `ff_posix_provider::PosixProvider` still resolves (it is the
  re-export).
- VERIFIED (scoped): `cargo check -p ff-nav-model --all-targets` CLEAN;
  `cargo test -p ff-nav-model` = 23 passed, 0 failed.
- NOT YET RE-GATED: the owner's second paste was STALE scrollback (same 22:01:05
  started timestamp; gate.history.csv shows the last real run ended 22:02:58,
  BEFORE the fix; gate.review.log still shows the old build error). A fresh
  `cargo gate --build` has not been run since the fix.

## RC.B PROGRESS

- RC.B.5 (land ff-volume + ff-dscatalog schema v4) -- DONE, owner-gate CONFIRMED:
  full `cargo gate --build` CLEAN on 2026-10-09 10:11 (run 0b7ea2b2; all phases
  ok; 9705 tests pass, 0 fail; empty gate.review.log). The three anticipated
  downstream risks (governance DAG allow-list, SCHEMA_VERSION=='3' assertions,
  storage_path readers) did NOT bite -- dual-read held. Uncommitted on `main`.
  Delivered: new `ff-volume` crate (volume-model Req 1-11; 10 modules; the two
  distinct failures SpaceAbend vs VolumeFull; acyclic DAG) + ff-dscatalog schema
  v4 (volumes + dataset_volumes tables; dual-read bytes-never-move v3->v4
  migration; DatasetVolume-indirected resolution with Online check + storage_path
  fallback). NON-BLOCKING doc hygiene to tidy: TCR has duplicate rows for the
  volume-model criteria (new PASS section + old NOT COVERED rows) -- fold into the
  next doc-touching run.
- RC.B.6 (define DatasetAccess + retarget ff-dsalloc) -- DONE, owner-gate
  CONFIRMED: full `cargo gate --build` CLEAN on 2026-10-09 14:59 (run 9b24afae;
  all phases ok; 9728 tests pass, 0 fail; empty gate.review.log). The anticipated
  governance-DAG risk did NOT bite (acyclic check accepted ff-dsalloc -> ff-volume).
  The keystone JES/JCL contract: object-safe `DatasetAccess`
  (allocate/open/get/put/point/close/dispose) in `ff-dscatalog`, RECFM-aware I/O
  via the codecs (no CRLF), location via `ff-volume`, I/O via the single
  `ff-vfs::StorageProvider` seam, allocate honours Volume status + charges SPACE;
  `ff-dsalloc` `AllocationOutcome::Allocated` carries the handle. Scoped tests:
  `ff-dscatalog` 287 lib pass (incl. `dataset_access::impl_access_tests` /
  `impl_io_tests`); `ff-dsalloc` 118 + 6 pass. DEFERRED to RC.B.7 (Option A owner
  decision): Req 19.3 `Verified`/`Passed` handle-isation needs DatasetAccess
  resolve-by-DSN; VSAM keyed/relative `point()` returns typed NotYetWired.
  NOTE: two RC.B.6 workflow attempts WEDGED on infra/ENOTFOUND network faults
  (not code problems); the code was already on disk + reviewer-shaped, so the
  housekeeping (TCR flips incl. the RC.B.5 duplicate-row reconciliation, task
  marks, Req 19.3 PARTIAL) was completed INLINE by the orchestrator. NEXT: owner
  runs full `cargo gate --build`. Anticipated out-of-scope gate risks: a
  governance-tests dependency-direction check may need the ff-dsalloc -> ff-volume
  edge; any other crate on the old AllocationOutcome shape (grep showed consumers
  are internal to ff-dsalloc).
- RC.B.6 (define DatasetAccess + retarget ff-dsalloc) -- done above.
- RC.B.7 (wire VSAM under DatasetAccess, retire ff-vsam-services) -- DONE,
  CODE-COMPLETE PENDING the owner's full `cargo gate --build`. Delivered across
  three slices:
  - FEAT-001 (completed by a prior workflow that WEDGED on an infra fault, then
    recovered -- code verified on disk): concrete `VsamService` get/put/browse +
    `point()` wired over the KSDS/ESDS/RRDS backends through `DatasetAccess`;
    scoped `cargo test -p ff-dscatalog` = 309 pass incl.
    `point_on_ksds_round_trips_keyed_record`,
    `point_relative_rrds_round_trips_and_rejects_rrn_zero` and resolve-by-DSN.
  - FEAT-002 (same prior run; verified on disk): `AllocationOutcome`
    `Verified`/`Passed` handle-ised via DatasetAccess resolve-by-DSN; the RESOLVE
    panel shows DSN identity; scoped `cargo test -p ff-dsalloc` = 123+6 pass incl.
    `verified_outcome_carries_handle_not_path`, `passed_outcome_carries_handle`.
  - FEAT-003 (completed THIS run): retired `ff-vsam-services` -- deleted
    `crates/ff-vsam-services`, removed its root `Cargo.toml` member line,
    reconciled the governance rules/tests (removed the 3 ff-vsam-services
    DependencyRules in compliance.rs; removed the `vsam_services_has_no_upstream_dependencies`
    test and the ff-vsam-services entries in the crate-list arrays in
    architecture_compliance.rs), and fixed stale doc comments
    (ff-dataset-catalog lib.rs/traits.rs, ff-idcams services.rs doc only -- trait
    CODE untouched, that repoint is RC.B.8). Scoped checks clean:
    `cargo test -p ff-governance-tests` GREEN (9 architecture + 7 mock compilation);
    `cargo check` on ff-governance-tests/ff-dscatalog/ff-dsalloc/ff-idcams and
    ff-desktop clean; grep shows NO live `ff-vsam-services`/`ff_vsam_services`
    references in any *.toml/*.rs. ff-idcams trait repoint deliberately NOT
    crossed into (that is RC.B.8).
  TCR flipped to PASS: dataset-catalog Req 35.3, ownership Req 22.3, and the
  now-closed RC.B.6 deferrals Req 34 (VSAM point ops) + Req 19.3 (Verified/Passed
  handle-isation). Tasks marked: dataset-catalog Task 41 (41.1+41.2) [x],
  dataset-allocator Task 21 + 21.3 [x], master RC.B.7 [x] code-complete-pending-gate.
  OWNER-GATE CONFIRMED: full `cargo gate --build` CLEAN on 2026-10-09 21:09
  (run 942d619a; all phases ok; 9752 tests pass, 0 fail; empty gate.review.log).
  Uncommitted on `main`. NOTE: the RC.B.7 workflow wedged twice on infra (an agent
  restart, then a ~52-min silent stall); FEAT-001/002 were already complete+tested
  on disk when found, FEAT-003 finished via a focused coder run.
- RC.B.8 (repoint ff-idcams + record-aware SAVE, delete ff-dataset-catalog) -- DONE, CODE-COMPLETE PENDING the owner's full `cargo gate --build` (Part 2 deferred, owner-accepted).
  - Part 1 (repoint ff-idcams -> ff-dscatalog, Req 28 / task 29): code-complete pending the owner's full gate.
  - Part 2 (record-aware MAINFRAME SAVE): NO LONGER A LOOSE DEFERRAL -- it has been ABSORBED into three CRs and is now an active, sequenced work stream (owner-directed 2026-10-09 "continue working RC.B.8 Part 2 into CR-CH-058 / CR-CH-059 / CR-CH-060"):
    - prerequisite (a) the record-aware `ff-vfs::BackendEnvironment` store contract -> **CR-CH-060**, APPROVED (Shape 2), PENDING IMPLEMENTATION.
    - the editor SAVE-walk that produces re-framed records -> **CR-CH-058** (document-model Req 13; its F-phase SAVE walk).
    - prerequisites (b)-(e) the open named-backend registry, `owning_env="MAINFRAME"` tab binding, live mainframe VFS provider, and the mainframe CE over `DatasetAccess` -> **CR-CH-059** RC.B stream (idcams-emulator Req 28 composing; master phase BRC.4).
    The five prerequisites remain documented in `.agents/tasks/rcb8-dataset-rationalisation/part2-mainframe-save-stop.md` (now headed RESOLVED/ABSORBED). NO .rs change has regressed native/host SAVE (byte-identical). The HARD build-order dependency: CR-CH-058 F1 (the editor record model) must land on `main` before the SAVE-walk/mainframe-CE code (BRC.3/BRC.4) can be written -- today SAVE flattens to one byte buffer, so there are no records to pack. See the SEQUENCED PLAN below.
  - Part 3 (delete `ff-dataset-catalog` + re-express governance) -- DONE (FEAT-003, completed THIS run), code-complete pending the owner's full gate. Mirrored the RC.B.7 ff-vsam-services retirement EXACTLY. Grep PRE-CHECK PASSED (no shipping crate had ff-dataset-catalog as a Cargo dep after the Part 1 repoint). Deleted `crates/ff-dataset-catalog/` + its root `Cargo.toml` member line; re-expressed the governance rules (removed the `ff-vfs -> ff-dataset-catalog` and the two `ff-dataset-catalog -> ff-idcams`/`-> ff-dsalloc` DependencyRules in `compliance.rs`; removed the `!deps.contains_key("ff-dataset-catalog")` assertion, DELETED the whole `dataset_catalog_has_no_upstream_dependencies` test, and removed the ff-dataset-catalog entries from the `dataset_crates` array + `required_crates` list in `architecture_compliance.rs`). The acyclic/single-authority intent stays covered by the surviving ff-vfs -> ff-idcams, ff-vfs -> ff-dsalloc, and ff-dsalloc -> ff-idcams rules. `mock_compilation.rs` had no residual `ff_dataset_catalog` import (already on ff-dscatalog). No stale `.rs` authority prose remained (the only mentions are the historical narrative in `ff-dscatalog/src/service.rs` and the new retirement comment). Scoped checks clean: `cargo fmt`; `cargo check -p ff-governance-tests -p ff-dscatalog -p ff-dsalloc -p ff-idcams --all-targets`; `cargo test -p ff-governance-tests` GREEN (architecture_compliance 8 + mock_compilation 7); `cargo check -p ff-desktop --all-targets` clean; final grep finds NO live `ff-dataset-catalog`/`ff_dataset_catalog` in any `*.toml`/`*.rs`. TCR flipped: dataset-catalog Req 35.1 (step d) + 35.2, ownership Req 22.1/22.2 (and the 22.6 citation retargeted off the deleted test). Tasks marked: dataset-catalog Task 42.1/42.2 [x], master RC.B.8 [x] code-complete-pending-gate.

## RESUME POINT (2026-10-10 ~13:30) -- RC.B.8 Part 2 (b)-(d) + BRC.4: compiling, tests pending

Record-aware MAINFRAME SAVE is WIRED END-TO-END and COMPILES CLEAN, but NOT yet
test-verified or gated, and it carries one documented live-session limitation.
Everything below is UNCOMMITTED on `main`.

WHAT LANDED (on disk, read + confirmed; `cargo check` exit 0 across ff-idcams /
ff-desktop / ff-dscatalog / ff-vfs / ff-volume via tools/logs/ff-check.txt):
- BRC.4 (FEAT-001): `crates/ff-idcams/src/mainframe_env.rs` -- `MainframeEnvironment`,
  a record-capable `ff_vfs::BackendEnvironment` storing over an owned
  `Arc<dyn ff_dscatalog::DatasetAccess>` (resolve -> open(Write) -> put -> close);
  `name()="MAINFRAME"`, `record_capable()=true`, byte `save()`=error; rc map
  Ok=0 / SpaceAbend=VolumeFull=37 / ReadOnly=8 / other=12. ff-idcams 37+22 tests
  passed in the FEAT-001 run; new acyclic edge ff-idcams -> ff-vfs.
- RC.B.8 (b): `shell/environment_registry.rs` -- `MAINFRAME_NAME`,
  `MainframePlaceholder`, a `mainframe: Option<Box<dyn BackendEnvironment>>` field,
  `with_builtins_and_mainframe(..)` ctor, and `backend_for(name)` (MAINFRAME ->
  mainframe CE when built; HOSTFS/unknown -> host_fs fallback). `shell/dispatch_ffedit.rs`
  `host_fs_save` now resolves the backend by the active tab's `owning_environment`
  via `backend_for` (the single SAVE seam -- only the SELECTION became owning-env
  aware; HOSTFS stays byte-identical).
- RC.B.8 (c)/(d) construction: `shell/mainframe_backend.rs` (NEW) --
  `build_mainframe_backend(root)` composes a `PosixNativeProvider` + a single-Volume
  `VolumeRegistry` into a `CatalogDatasetAccess`, wraps it as the mainframe CE;
  `build_catalog_provider()` builds the `catalog`-scheme `CatalogVfsProvider`.
  `shell/construct_provider.rs` additively registers the catalog provider (Req 17.4).

KNOWN LIMITATION (documented in mainframe_backend.rs header; NOT hidden; owner
decision needed): the mainframe CE builds a FRESH `CatalogDatasetAccess` that does
NOT share the Files Panel's LIVE SQLite catalog + volume handles. So a same-session
SAVE of a dataset the Files Panel created resolves `NotFound` (rc 12). The wiring /
routing / identity threading / record-store contract are ALL in place and
unit-testable; only live same-session store-visibility is bounded. Closing it needs
the shell to share the exact live catalog/volume handles (a cross-crate handle the
shell does not currently own). This is the gap between "wired + gate-clean" and
"a user edits a mainframe dataset and SAVE persists in-session".

NEXT ACTIONS (in order) when the terminal is healthy again:
1. Run the reusable helper `tools/powershell/ff-check.ps1` (NEW this session --
   tasks: status | check | test | clippy | fmt; writes tools/logs/ff-*.txt):
   `C:\tools\powershell7\pwsh.exe -NoProfile -File C:\workspace\VSC\FileForgeWorkbench\tools\powershell\ff-check.ps1 test`
   then `... clippy`. Read tools/logs/ff-test.txt + ff-clippy.txt. (`check` already
   PASSED, exit 0.)
2. If tests + clippy clean: flip TCR to PASS for command-environments Req 16 (mainframe
   CE) / 17.4 (provider) / 18.7 / 18.8, idcams Req 28, and mark project-master
   Phase (backendenv-record-contract) BRC.4 [x] + the RC.B.8 Part 2 (b)-(e) deferral
   RESOLVED -- BUT record the live-handle limitation as an explicit OPEN FOLLOW-UP,
   do not claim full live functionality.
3. Hand off the full `cargo gate --build` to the owner.
4. OWNER DECISION: fix the live catalog-handle sharing now (a follow-up slice: thread
   the Files Panel's live ff-catalog-registry catalog + ff-volume handles into the
   mainframe CE's DatasetAccess instead of building a fresh one), or accept it as a
   documented known limitation for this phase.
5. The BRC.4 workflow (wf_37f8655592cf4e3b) was ABORTED after its deliverables landed
   + compiled (step stayed 'running' amid infra/terminal flakiness). Finish inline.

TERMINAL NOTE: this session hit intermittent PSReadLine wedging (dangling `>`,
commands not executing). The reusable `ff-check.ps1` + short clean invocations are
the mitigation; a Kiro restart (owner doing it now) + pointing the terminal at
pwsh 7 (not Windows PowerShell 5.1, whose OneDrive profile prints the Postgres
banner + a Start-Service error) should clear it.

## STATUS (2026-10-09): RC.A through RC.B.8 (Part 1) COMMITTED + PUSHED + full-gate CLEAN

RC.A, RC.B.5, RC.B.6, RC.B.7, RC.B.8 (Parts 1+3) are all on `origin/main`, each
full-gate clean. The dataset stack rationalisation (CR-CH-059) is COMPLETE except
record-aware MAINFRAME SAVE, which is now the CR-CH-060/058/059 joint work stream
below.

## SEQUENCED PLAN -- record-aware MAINFRAME SAVE (RC.B.8 Part 2, absorbed into CR-CH-060 + CR-CH-058 + CR-CH-059)

Requirements are COMPLETE and APPROVED (CR-CH-060 Shape 2; command-environments
Req 18; document-model Req 13; idcams-emulator Req 28). The only gate to CODE is a
build-ordering dependency, not a missing requirement.

1. **Land CR-CH-058 F1 on `main`** (THE blocker for all SAVE-walk code). F1 (the
   editor record model + piece-table + byte-identical native SAVE) is built and
   gate-clean but UNMERGED in `.worktrees/wrf-foundation` (branch
   `feature/windowed-record-foundation`). **DONE (2026-10-10): the owner assigned F1
   to THIS session; it was safety-tagged (`wrf-foundation-pre-rebase`), rebased
   onto current `main` TWICE (clean, no conflicts -- the second rebase because the
   CR-CH-060 docs commit advanced main under it), scoped-verified (ff-document-model
   201 tests pass incl. byte-identical SAVE; ff-desktop closure compiles), and
   FAST-FORWARDED onto `main` (`6471b3a`). Full `cargo gate --build` CLEAN
   (run bc55ddf5, 9839/9839) and PUSHED to origin/main. F1 is landed.**
2. **BRC.2** -- the additive record-aware `BackendEnvironment` store entry in
   `ff-vfs` (Shape 2: byte `save` retained + `save_records` + `record_capable()`;
   object-safe; host CEs inherit the declining default). command-environments
   Req 18.1-18.4. **DONE (2026-10-10): code-complete pending the owner's full gate.**
   Added `save_records`/`record_capable` with declining defaults + ff-vfs-local
   types `StoreTarget`/`RecordAttrs`/`RecordFormatKind`/`RecordSource`/
   `RecordStoreOutcome` (NO ff-dscatalog/ff-document-model dep -- acyclic DAG held);
   byte `save` untouched. Scoped checks clean: ff-vfs 169+PBT+doc tests pass incl.
   6 new Req-18 tests; the three host CEs compile UNCHANGED (Req 18.3 proven);
   clippy -D warnings clean; ff-desktop --all-targets compiles (dyn object-safety
   holds). TCR Req 18.1-18.4 PASS, 18.8 outcome-type PARTIAL. Completed inline
   after the BRC.2 workflow wedged (impl on disk; orchestrator verified+finished).
   NEXT GATE: owner runs `cargo gate --build` to confirm BRC.2 on the full workspace.
   (BRC.2 gate CONFIRMED CLEAN: run ea8c99d1, 9845/9845. Committed `0a7da61`, pushed.)
3b. **BRC.3** -- the editor SAVE-walk byte-vs-record selection (ff-desktop
   `save_active_tab_via_backend`). command-environments Req 18.5-18.6; document-model
   Req 13.1-13.6. **DONE 2026-10-10: code-complete, scoped-verified, UNCOMMITTED on
   main, pending the owner's full gate.** Selection by `Document::record_format()` +
   `backend.record_capable()`; byte path verbatim (byte-identical, proven); record
   path via `RecordImageSource` + `StoreTarget`(placeholder) + `RecordAttrs` ->
   `save_records`, outcome mapped (rc0->Ok / rc!=0->Err / NotRecordCapable->byte
   fallback). Scoped checks: `tab_manager::tests` 55/55 (4 new BRC.3 tests);
   `cargo clippy -p ff-desktop --bins -- -D warnings` clean; `cargo fmt` done. TCR
   document-model Req 13.1-13.6 + command-environments 18.5/18.6 -> PASS. Completed
   INLINE after the BRC.3 workflow hung (~21 min, impl on disk + compiling; the
   orchestrator added the 4 tests, fixed one `DelimiterTerminator` type error in a
   test, ran the scoped checks, flipped docs). The record path is unit-tested against
   a record-capable TEST backend; it is NOT live end-to-end yet (needs RC.B.8 (b)-(d)
   + BRC.4). **NEXT GATE: owner runs `cargo gate --build` to confirm BRC.3 (test
   count should rise above 9845); if clean, commit + push BRC.3, then proceed to
   RC.B.8 (b)-(d) and BRC.4.**
3. **RC.B.8 Part 2 (b)-(d)** -- open the closed `RegisteredEnv` registry to a
   named-backend map + dispatch arm; `open_mainframe_dsn` passes
   `owning_env="MAINFRAME"`; register the mainframe VFS provider live
   (`build_live_provider_registry`, Req 17.4). Shell/registry/provider wiring,
   parallelisable with step 2 and CR-CH-058 F2-F5.
4. **BRC.3** -- the editor SAVE-walk byte-vs-record store-call selection on the
   CR-CH-053 Task 20/21 owning-CE seam (document-model Req 13); the walk re-frames
   the Piece_List per the OPEN-supplied RecordFormat, never re-derived. Needs F1
   (step 1) + the contract (step 2).
5. **BRC.4** -- the mainframe CE in `ff-idcams` implements the record-aware entry
   over `ff_dscatalog::DatasetAccess` (open->put->close; RECFM codec frames bytes
   on close; x37 surfaced). Needs steps 2+3+4. idcams-emulator Req 28 composing;
   lift the editor-nav gate that currently refuses mainframe editing. Now a
   MAINFRAME-owned editable tab exists and the first-fail SAVE test is writable.

What is PARALLEL: steps 2 and 3 (and CR-CH-058 F2-F5) once the step-1 contract
shape is settled (it is -- Shape 2 approved). The one serialisation point is the
shell SAVE path (`save_active_tab_via_backend` / `host_fs_save`): CR-CH-058
reshapes HOW records are produced, RC.B.8 reshapes WHERE they are addressed -- land
CR-CH-058's SAVE-walk refactor first, then RC.B.8 adds the mainframe record form.

The ONLY owner input needed to proceed: confirm who merges CR-CH-058 F1 (step 1).
Everything after is specified and can be driven as scoped-and-gated code work.

## SUPERSEDED NEXT ACTIONS (historical -- all DONE)

1. (DONE) Owner re-ran `cargo gate --build` after the ff-nav-model fix -- clean.
2. (DONE) Shell file split + RC.A both landed and were committed/pushed.
3. (DONE) RC.B ran to completion (RC.B.5-RC.B.8) and is committed/pushed. The old
   "do not auto-start RC.B" note is obsolete; RC.B is finished.

## OTHER OPEN ITEMS (unchanged, lower priority)

- Shell file split (construct.rs/commands.rs/state.rs -> under 400 via
  construct_commands.rs/commands_environment.rs/commands_line.rs + NavUiState in
  state_groups.rs) -- reviewer-APPROVED, uncommitted, folded into the SAME gate
  run above.
- ff-dataset-catalog vs ff-vsam-services crate deletion -- deferred to RC.B.7/8
  (gated consolidation; procedure in dscatalog-duplicate/report.md).
- Volume UI gate (virtual-catalog-manager Req 17-18) -- valid as spec, build
  deferred into RC.C.10 after ff-volume lands.
- Record-aware MAINFRAME SAVE -- DEFERRED from RC.B.8 (CR-CH-059, Part 2,
  owner-accepted). Needs an owner-confirmed `ff-vfs::BackendEnvironment` record
  contract (or a record-aware addressing variant) + an open `EnvironmentRegistry`
  + mainframe `owning_env` binding + the mainframe VFS provider registered live.
  Prerequisites a-e in `.agents/tasks/rcb8-dataset-rationalisation/part2-mainframe-save-stop.md`;
  a future owner-gated framework-change slice.

## GUARDRAILS REMINDER

- Kiro runs SCOPED `-p` checks only; the full `cargo gate --build` is the OWNER's
  manual step. Known pre-existing B048 shared-env-var flake fails only under
  multithreaded `cargo test`, passes under nextest (which the gate uses).
- ASCII-only .rs; 400-non-test-line rule; build ON the framework; additive-first
  until RC.B.
