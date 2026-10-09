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
- RC.B.7 (wire VSAM under DatasetAccess, retire ff-vsam-services) -- after B.6.
- RC.B.8 (repoint ff-idcams + record-aware SAVE, delete ff-dataset-catalog) -- after B.7.

## NEXT ACTIONS (in order)

1. OWNER (manual, outside Kiro): re-run `cargo gate --build` from the workspace
   root. Confirm a NEW `started` timestamp (later than 22:01). Clean =
   `.gate/gate.review.log` is EMPTY.
   - If the full test run surfaces MORE out-of-scope crates that referenced the
     old `PosixProvider::new -> Result` API, they are the SAME trivial fix (drop
     `.expect(...)`; the constructor is now infallible). Kiro can clear each with
     scoped checks and hand off again.
2. If clean: the shell file split AND RC.A are both DONE. Mark the shell-split
   hand-off closed (RESUME-ffdesktop-simplification.md) and RC.A done in
   project-master.
3. THEN STOP -- do not auto-start RC.B. RC.B (land ff-volume, define DatasetAccess,
   wire VSAM, mainframe record-aware SAVE) is PLUGIN phase 2 per ROADMAP, gated on
   CORE sign-off, and the editor work (CR-CH-058) is live in another worktree.
   RC.B begins only on an explicit owner "start RC.B".

## OTHER OPEN ITEMS (unchanged, lower priority)

- Shell file split (construct.rs/commands.rs/state.rs -> under 400 via
  construct_commands.rs/commands_environment.rs/commands_line.rs + NavUiState in
  state_groups.rs) -- reviewer-APPROVED, uncommitted, folded into the SAME gate
  run above.
- ff-dataset-catalog vs ff-vsam-services crate deletion -- deferred to RC.B.7/8
  (gated consolidation; procedure in dscatalog-duplicate/report.md).
- Volume UI gate (virtual-catalog-manager Req 17-18) -- valid as spec, build
  deferred into RC.C.10 after ff-volume lands.

## GUARDRAILS REMINDER

- Kiro runs SCOPED `-p` checks only; the full `cargo gate --build` is the OWNER's
  manual step. Known pre-existing B048 shared-env-var flake fails only under
  multithreaded `cargo test`, passes under nextest (which the gate uses).
- ASCII-only .rs; 400-non-test-line rule; build ON the framework; additive-first
  until RC.B.
