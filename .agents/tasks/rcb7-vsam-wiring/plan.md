# Implementation Plan -- RC.B.7: Wire VSAM under DatasetAccess, close resolve-by-DSN, retire ff-vsam-services

CR-CH-059 Phase RC.B STEP 7. All edits go DIRECTLY on `main`, uncommitted (owner reviews + runs the full gate). This plan builds ON the existing framework (DatasetAccess + the single `ff-vfs::StorageProvider` seam + the existing KSDS/ESDS/RRDS backends); it introduces no new backend and no second dispatcher.

## Context discovered during exploration (ground truth)

- `DatasetAccess` trait + `CatalogDatasetAccess` live in `crates/ff-dscatalog/src/dataset_access/` (`trait_def.rs`, `types.rs`, `impl_access.rs`, `impl_io.rs`, `error.rs`, `mod.rs`, `impl_access_tests.rs`, `impl_io_tests.rs`). Sequential PS/PO paths are wired end-to-end over the RECFM codecs and ONE `Arc<dyn ff_vfs::StorageProvider>` held on `CatalogDatasetAccess`.
- `impl_io.rs::point_impl` resolves the Volume+locator for `Positioner::Key`/`Positioner::Rrn` then returns `DatasetError::NotYetWired { operation }`. Tests `point_on_ksds_returns_not_yet_wired` / `point_relative_rrds_returns_not_yet_wired` (in `impl_io_tests.rs`) assert that. These are the two stubs RC.B.7 replaces.
- The reconciled `VsamService` trait + `VsamCluster`/`VsamType`/`VsamParams`/`Record`/`VsamHandle`/`BrowseHandle`/`AccessMode`/`BrowseDirection`/`KeyField`/`VsamError` + `StubVsamService` are in `crates/ff-dscatalog/src/vsam_service.rs` (RC.A.2). `Record { key: Vec<u8>, data: Vec<u8> }` is re-exported as `crate::Record` and is the DTO `DatasetAccess::get/put` already use.
- The VSAM backends already exist and already implement `ff_vfs::StorageProvider` (RC.A.3) AND carry their own typed keyed/relative APIs:
  - `storage/sqlite_record.rs` `SqliteRecordProvider`: KSDS. Typed API `insert(key,&[u8])`/`read(key)->Option<KsdsRecord>`/`update`/`delete`/`sequential_read()->Vec<KsdsRecord{key:String,data}>`/`range*` + alternate-index ops (`add_alternate_index`/`rebuild_alternate_index`/`lookup_by_alternate_key`/`list_alternate_indexes`). Keys are `String` (text). `KsdsKeyDefinition::new(offset,length)`. Constructor `SqliteRecordProvider::open(repository_root, Uuid, KsdsKeyDefinition)`.
  - `storage/esds.rs` `NativeEsdsProvider`: ESDS. Typed API `append(&[u8])->u64 (stable address)`/`read(addr)->Option<EsdsRecord{address,data}>`/`update`/`delete_record`/`sequential_read()->Vec<EsdsRecord>` (insertion order). Constructor `NativeEsdsProvider::open(repository_root, Uuid)`.
  - `storage/rrds.rs` `SqliteRrdsProvider`: RRDS. Typed API `read(rrn:u64)->RrdsSlot{Unallocated|Allocated(Vec<u8>)}`/`write(rrn,&[u8])`/`delete_record`/`sequential_read()->Vec<RrdsRecord{record_number,data}>`. RRN starts at 1 (0 rejected). Constructor `SqliteRrdsProvider::open(repository_root, Uuid)`.
  - All exported from `storage/mod.rs`.
- CRITICAL seam fact: `ff_vfs::StorageProvider` is BYTE-oriented only (`open(locator)->Vec<u8>`, `write(locator,&[u8])`); keyed/relative access is advertised only via `StorageCapability::{KeyedAccess,RelativeAccess,AppendOnly}` flags, NOT as trait methods. The typed keyed/relative methods above are concrete-type methods, unreachable through `Arc<dyn StorageProvider>`. The design decision in FEAT-001 resolves how `DatasetAccess` reaches them.
- `ff-dsalloc`: `AllocationOutcome::{Verified{physical_path,...}, Allocated{handle,..}, WouldAllocate{dsn}, Passed{physical_path,passing_step}, Skipped}` in `src/allocation.rs`. RC.B.6 already made `Allocated` carry `ff_dscatalog::DatasetHandle`. `Verified`/`Passed` still carry `physical_path` (sourced from `CatalogProvider::lookup_dsn` and the `PassTable`, NOT from DatasetAccess). `src/catalog_bridge.rs` has the mockable `DatasetAllocator` trait (`allocate(&req)->DatasetHandle`) + `MockDatasetAllocator` (drives a real `CatalogDatasetAccess::in_memory`). `src/pipeline.rs` maps `AllocationOutcome`->`ResolutionOutcome` (+ `dsn_display_path()` helper); `src/panel.rs` renders `ResolutionOutcome`; `src/command.rs` `resolve_single_dsn` builds a `Resolved{physical_path}` from `CatalogProvider::lookup_dsn`.
- No `ff-desktop` code consumes `AllocationOutcome`/`ResolutionOutcome` directly (grep clean) -- `cargo check -p ff-desktop` is a regression guard only.
- `ff-vsam-services` (`crates/ff-vsam-services`) is STUB-only (`StubVsamService`), listed in root `Cargo.toml` `[workspace].members` at `"crates/ff-vsam-services"`. NO shipping crate depends on it as a code dependency: `ff-governance-tests/Cargo.toml` dev-depends only on `ff-dscatalog`; the only references are governance TEST STRINGS + a doc comment and `ff-idcams`'s own PRIVATE trait doc. `ff-idcams` carries its OWN private `VsamService` (repoint is RC.B.8 -- OUT OF SCOPE).
- Governance coupling that will FAIL the gate once the crate is gone:
  - `crates/ff-governance-tests/tests/architecture_compliance.rs`: `all_governed_crates_exist` ASSERTS `ff-vsam-services` exists (hard fail); `prohibited_rules_cover_all_dataset_crates` requires a rule naming it; `vsam_services_has_no_upstream_dependencies` has a `crate_exists` skip-guard (benign). `required_crates`/`dataset_crates` lists include `"ff-vsam-services"`.
  - `crates/ff-governance-tests/src/compliance.rs`: `PROHIBITED_DEPENDENCIES` has two `crate_name:"ff-vsam-services"` rules + one `prohibited_dependency:"ff-vsam-services"` (on ff-vfs). `check_compliance` skips crates whose Cargo.toml is absent (benign). These are `&'static str`, so leaving/renaming them does not break compilation; the plan re-expresses them against `ff-dscatalog` per ownership Req 22.3.
  - BOTH governance files currently use non-ASCII box-drawing (U+2500 `-`) in comment separators. Per `documentation.md` `.rs` files are STRICT ASCII. Any region edited MUST be converted to ASCII `// === ... ===`.
- Stale doc comments naming ff-vsam-services as the VSAM authority: `crates/ff-dataset-catalog/src/lib.rs` ~line 21, `crates/ff-idcams/src/services.rs` ~line 321 (comment ONLY -- do NOT touch ff-idcams's private `VsamService` trait CODE; that is RC.B.8).
- ACYCLIC DAG to preserve: `ff-idcams -> ff-dsalloc -> ff-dscatalog -> ff-volume -> ff-vfs`. `ff-volume` gains NO new deps. Concrete VSAM stays in `ff-dscatalog`.
- File-size: `rust-standards.md` 400 non-test lines; split tests to a sibling `_tests.rs` before ~200 lines. `catalog.rs` ~640, `esds.rs`/`sqlite_record.rs` are pre-existing >400 -- DO NOT grow them; put new code in NEW modules under `dataset_access`/`vsam_service` or new sibling files.
- Waste list (dataset-catalog Req 35.5): do NOT wire VSAM against `ff-vsam-services`'s dead trait or `ff-dscatalog::storage::StorageProvider`; do NOT build against raw `storage_path`/`physical_path`. Do NOT touch `.worktrees/`.

## Scoped verification commands (NEVER --workspace / cargo gate -- owner runs the full gate)

Run each via the clean pwsh7 non-interactive wrapper, ONE command per invocation, redirect to `tools/logs/` and read the log back (harness shows cosmetic `Exit Code -1`; trust the log):

- `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo fmt"`
- `... "cargo check -p ff-dscatalog -p ff-dsalloc -p ff-governance-tests --all-targets"`
- `... "cargo clippy -p ff-dscatalog -p ff-dsalloc -- -D warnings"`
- `... "cargo test -p ff-dscatalog -p ff-dsalloc"`
- `... "cargo test -p ff-governance-tests"`
- `... "cargo check -p ff-desktop --all-targets"`  (regression guard -- no consumer of the reshaped outcomes)
- `... "cargo check -p ff-volume"`  (confirm ff-volume gained no new dep / still builds)
- confirm root `Cargo.toml` no longer lists `ff-vsam-services` (read the file)
- `C:\tools\python\python.exe tools\python\check_line_limits.py` (redirect to `tools/logs/`, read back)

TDD per `testing.md`: for every criterion write the failing test FIRST, run the scoped `cargo test -p <crate>` to confirm RED, then minimal implementation to GREEN. Every test carries `// Validates: Requirement X.Y`.

---

## FEATURE 1 -- Concrete VSAM record ops under DatasetAccess (dataset-catalog Task 41.1; Req 34.1/34.2/35.3; ownership Req 22.3)

### Design decision (recorded here; chosen, not deferred)

The byte-only `StorageProvider` seam cannot carry keyed/relative ops, so `CatalogDatasetAccess` (which holds `Arc<dyn StorageProvider>`) cannot reach them by downcast. Decision: introduce an internal, in-crate `VsamBackend` abstraction in `ff-dscatalog` that OWNS the concrete typed backends (`SqliteRecordProvider`/`NativeEsdsProvider`/`SqliteRrdsProvider`) directly -- legal because it lives in `ff-dscatalog` alongside them and preserves the DAG. The concrete reconciled `VsamService` impl is built over `VsamBackend`. `DatasetAccess::point/get/put` for VSAM positioners delegate to the same `VsamBackend` for the open cluster. This keeps VSAM a CLUSTER ENTITY (`VsamCluster`), NOT a `Dsorg` variant, and keeps all physical record I/O inside `ff-dscatalog` on the backends from RC.A (no new backend). Rationale: it is the only approach that (a) reuses the existing typed backends, (b) does not widen the public `ff_vfs::StorageProvider` trait, (c) keeps object-safety of `DatasetAccess`, and (d) does not cross into ff-idcams.

- [ ] 1. Add the internal `VsamBackend` enum + constructor in a NEW module `crates/ff-dscatalog/src/vsam_backend.rs` (declared `mod vsam_backend;` in `lib.rs`).
      `enum VsamBackend { Ksds(SqliteRecordProvider), Esds(NativeEsdsProvider), Rrds(SqliteRrdsProvider) }` with `fn open_for(root: &Path, dataset_id: Uuid, vsam_type: VsamType, params: &VsamParams) -> Result<Self, VsamError>` (KSDS uses `KsdsKeyDefinition::new(key_offset,key_length)` from `params`; LDS -> `VsamError::UnsupportedOperation` for now, documented). Add thin typed methods the service/point need: `get_keyed(key:&[u8])`, `put_keyed(rec:&Record)`, `delete_keyed(key)`, `browse_from(start:&[u8],dir)`, `get_relative(rrn:u64)`, `put_relative(rrn,&[u8])`, `append_entry(&[u8])->u64`, `sequential()->Vec<Record>`. Map `CatalogError`->`VsamError` locally. Keep this file < 400 non-test lines; put its unit tests in `vsam_backend_tests.rs` if they approach ~200 lines.
      Files: `crates/ff-dscatalog/src/vsam_backend.rs` (new), `crates/ff-dscatalog/src/lib.rs` (add `pub(crate) mod vsam_backend;` or `mod`), optional `crates/ff-dscatalog/src/vsam_backend_tests.rs` (new).
      Verify: `cargo test -p ff-dscatalog vsam_backend` -- new backend unit tests (KSDS keyed round-trip, RRDS RRN round-trip incl. rejecting RRN 0, ESDS append order, alt-index lookup) pass.

- [ ] 2. Implement the concrete `VsamService` over `VsamBackend` in a NEW module `crates/ff-dscatalog/src/vsam_service_impl.rs` (keep the trait + stub in `vsam_service.rs`; add `mod vsam_service_impl;` + re-export in `lib.rs`).
      Add `struct CatalogVsamService { root: PathBuf, state: Mutex<..open handles map..> }` implementing `VsamService`: `create_ksds/create_esds/create_rrds` and `initialize_dataset` provision a `VsamBackend`; `open` returns a `VsamHandle` keyed into the handle map; `get/put/delete` dispatch to the handle's `VsamBackend` keyed/relative methods; `start_browse/next_record/end_browse` iterate `sequential()`/`browse_from`; `define_aix/build_index` delegate to `SqliteRecordProvider` alt-index ops for KSDS (else `UnsupportedOperation`). Record boundaries come from the backend, never host text lines. Write failing tests FIRST in `vsam_service_impl_tests.rs`.
      Files: `crates/ff-dscatalog/src/vsam_service_impl.rs` (new), `crates/ff-dscatalog/src/vsam_service_impl_tests.rs` (new), `crates/ff-dscatalog/src/lib.rs` (declare + re-export `CatalogVsamService`).
      Verify: `cargo test -p ff-dscatalog vsam_service_impl` -- KSDS keyed read-after-write round-trip; RRDS RRN positioning; ESDS stable-address append; browse returns records in sequential order; alt-index lookup (KSDS) all pass (RED first, then GREEN). Keep `vsam_service.rs` unchanged except any re-export; it stays < 400 lines.

- [ ] 3. Wire `DatasetAccess` VSAM `point`/`get`/`put` to the backend by replacing the `NotYetWired` branches in `impl_io.rs::point_impl`.
      Teach `CatalogDatasetAccess` to hold/resolve a `VsamBackend` for a VSAM cluster open. Since `OpenDataset` currently models sequential records, extend it (in `types.rs`) with an optional VSAM positioning state (e.g. `vsam: Option<VsamOpenState>` carrying the resolved `VsamType`, the current `Positioner`, and the backend handle/key cursor) WITHOUT breaking the sequential path or object-safety. `point_impl`: `Positioner::Key` positions a KSDS cluster by key; `Positioner::Rrn` positions an RRDS cluster by RRN (reject RRN 0 -> `BadPositioner`); after a successful `point`, `get_impl`/`put_impl` operate at that position via the backend (KSDS keyed read/write; RRDS relative read/write; ESDS append-with-stable-address). A VSAM open must select the backend from the cluster's `VsamType` (resolved via the catalog/handle), reusing the existing locator resolution for the Volume Online check. Keep every signature object-safe (`&self`, no generics, no `Self` by value). Keep `impl_io.rs` under 400 non-test lines -- if it grows, extract the VSAM record path into a new `dataset_access/impl_vsam.rs` sibling module and delegate.
      Files: `crates/ff-dscatalog/src/dataset_access/impl_io.rs`, `crates/ff-dscatalog/src/dataset_access/types.rs`, `crates/ff-dscatalog/src/dataset_access/impl_access.rs` (open path selecting VSAM backend), optional new `crates/ff-dscatalog/src/dataset_access/impl_vsam.rs` + `mod` line in `dataset_access/mod.rs`.
      Verify: `cargo test -p ff-dscatalog dataset_access` -- the updated point/get/put tests pass (see step 4), sequential tests in `impl_access_tests.rs` still pass unchanged.

- [ ] 4. Replace the two deferral tests with real-behaviour assertions and add VSAM record-op tests.
      In `impl_io_tests.rs`: rewrite `point_on_ksds_returns_not_yet_wired` -> a KSDS keyed read-after-write round-trip through `point`+`put`+`get` (assert the record read back by key equals what was written, boundaries from the codec/backend not CRLF); rewrite `point_relative_rrds_returns_not_yet_wired` -> an RRDS `point(Rrn)` + put/get round-trip and an RRN-0 `BadPositioner` assertion. Add a browse/sequential-order read test and, if the backend exposes it, a KSDS alt-index test. Each carries `// Validates: Requirement 34.1`/`34.2`/`35.3`. If the test file approaches ~200 lines, split the new VSAM tests into a sibling (e.g. `impl_vsam_tests.rs` with a `mod` line in `dataset_access/mod.rs`). Write these FIRST (RED) before steps 1-3's implementation is complete, per TDD.
      Files: `crates/ff-dscatalog/src/dataset_access/impl_io_tests.rs` (rewrite the two tests + add new), optional `crates/ff-dscatalog/src/dataset_access/impl_vsam_tests.rs` (new) + `mod` in `dataset_access/mod.rs`.
      Verify: `cargo test -p ff-dscatalog dataset_access` -- no test asserts `NotYetWired` for the keyed/relative point path anymore; all new VSAM round-trip/positioning/browse tests pass.

- [ ] 5. Remove the now-dead `NotYetWired`-related wording and keep the error variant only if still used.
      Update doc comments in `trait_def.rs` (`get`/`put`/`point` `# Errors` sections) and `mod.rs`/`impl_io.rs` module headers that say the VSAM op is deferred to RC.B.7 to describe the now-concrete behaviour. If `DatasetError::NotYetWired` has NO remaining producer after step 3, either keep it (it is `#[non_exhaustive]`, harmless) with an updated doc note, or remove it and the mapping -- but ONLY if `cargo clippy -p ff-dscatalog -- -D warnings` flags it as dead; do not force removal that breaks other matches. Keep all text ASCII.
      Files: `crates/ff-dscatalog/src/dataset_access/trait_def.rs`, `crates/ff-dscatalog/src/dataset_access/mod.rs`, `crates/ff-dscatalog/src/dataset_access/impl_io.rs`, possibly `crates/ff-dscatalog/src/dataset_access/error.rs`.
      Verify: `cargo clippy -p ff-dscatalog -- -D warnings` clean; `cargo fmt -- --check` clean.

---

## FEATURE 2 -- Close resolve-by-DSN; reshape Verified/Passed to carry a DatasetHandle (dataset-allocator Req 19.3 / Task 21.3; dataset-catalog Req 34)

Depends on FEATURE 1 only in sequencing (shared crate), not in code. The RESOLVE panel WILL now show the DSN identity instead of a raw path for Verified/Passed -- this is the intended Req 19.3 behaviour change; panel tests must assert the new display.

- [ ] 6. Add an object-safe `resolve`-by-DSN capability to `DatasetAccess`.
      Add `fn resolve(&self, dsn: &str, intent: AccessIntent) -> Result<DatasetHandle, DatasetError>;` to the `DatasetAccess` trait and implement it on `CatalogDatasetAccess`: resolve the dataset via the catalog -> `ff-volume` -> locator seam with the SAME Online check as `open`/`allocate` (reuse `resolve_locator`-style logic), returning an opaque `DatasetHandle` (no raw path). Keep object-safe (`&self`, `&str`, no generics). Write the failing test FIRST in `impl_access_tests.rs` (allocate a dataset, then `resolve(dsn)` returns a handle whose `dsn()` matches; resolving an unknown DSN returns `NotFound`; a `dyn DatasetAccess` can call `resolve`). Also add a convenience on `CatalogDatasetAccess` if needed so `ff-dsalloc` (depends only on ff-dscatalog) can resolve without naming ff-volume types.
      Files: `crates/ff-dscatalog/src/dataset_access/trait_def.rs`, `crates/ff-dscatalog/src/dataset_access/impl_access.rs` (or `impl_io.rs`), `crates/ff-dscatalog/src/dataset_access/impl_access_tests.rs`.
      Verify: `cargo test -p ff-dscatalog dataset_access` -- new `resolve`-by-DSN tests pass; the `_assert_object_safe(&dyn DatasetAccess)` guard still compiles.

- [ ] 7. Add a `resolve` seam to the allocator's `DatasetAllocator` bridge and the mock.
      In `crates/ff-dsalloc/src/catalog_bridge.rs` add `fn resolve(&self, dsn: &str) -> Result<DatasetHandle, CatalogError>;` to the `DatasetAllocator` trait; implement it on `MockDatasetAllocator` by driving the real `CatalogDatasetAccess` (allocate-then-resolve, or resolve of a pre-allocated DSN) so it yields a genuine opaque `DatasetHandle`. Write the failing test FIRST (`dataset_allocator_resolve_yields_handle`). Keep the trait object-safe (it is already used as `dyn`).
      Files: `crates/ff-dsalloc/src/catalog_bridge.rs`.
      Verify: `cargo test -p ff-dsalloc catalog_bridge` -- the new resolve test passes; existing `dataset_allocator_*` tests still pass.

- [ ] 8. Reshape `AllocationOutcome::{Verified, Passed}` to carry a `DatasetHandle` instead of `physical_path`, and obtain it via the allocator `resolve` seam.
      In `crates/ff-dsalloc/src/allocation.rs`: change `Verified { handle: ff_dscatalog::DatasetHandle, catalog_name: String, dataset_type: CatalogDatasetType }` and `Passed { handle: ff_dscatalog::DatasetHandle, passing_step: String }` (drop `physical_path`). In `simulate_allocation`, the DISP=OLD/SHR/MOD verified path and the PASS path call `allocator.resolve(&dsn_str)` to acquire the handle (instead of reading `first.physical_path` / `pass_entry.physical_path`). The `PassTable` may keep tracking a DSN (not a path) for lookup; its stored identity becomes the DSN. Update `allocation.rs` tests: add `verified_outcome_carries_handle_not_path` and `passed_outcome_carries_handle` (DISP=OLD on an existing dataset -> `Verified{handle}` whose `dsn()` matches; a PASS then OLD across steps -> `Passed{handle}`). Write failing tests FIRST. This closes the DEFERRED half of Task 21.3.
      Files: `crates/ff-dsalloc/src/allocation.rs` (enum + both producer arms + `PassTable` + tests).
      Verify: `cargo test -p ff-dsalloc allocation` -- new handle tests pass; `live_new_allocation_returns_handle_not_path` and `dry_run_acquires_no_handle` still pass.

- [ ] 9. Propagate the handle through `pipeline.rs` and `command.rs`; prefer DSN identity in the RESOLVE panel.
      In `crates/ff-dsalloc/src/pipeline.rs`: the `Verified`->`Resolved` and `Passed`->`Resolved` arms now read `handle.dsn()` and build the panel display from the DSN (reuse `dsn_display_path(handle.dsn())`, the Option-A helper RC.B.6 already uses for `Allocated`) rather than a raw `physical_path`. Keep `ResolutionOutcome` as the pipeline/panel DTO (it is purely a display model) -- the handle stays inside ff-dsalloc; the panel shows DSN-derived text. In `crates/ff-dsalloc/src/command.rs` `resolve_single_dsn`, build the `Resolved` display from the resolved DSN (via `allocator.resolve` + `dsn_display_path`) rather than `CatalogProvider::lookup_dsn().physical_path`; thread a `&dyn DatasetAllocator` into `resolve_single_dsn` (it is already available in `execute_resolve_command`). Preserve all existing ASCII; if touching the `GdgResolved` arm note its pre-existing non-ASCII arrow is out of scope (do not edit that line).
      Files: `crates/ff-dsalloc/src/pipeline.rs`, `crates/ff-dsalloc/src/command.rs`.
      Verify: `cargo test -p ff-dsalloc` -- pipeline + command tests pass with the DSN-derived display.

- [ ] 10. Update `panel.rs` to assert the new DSN-based display for Verified/Passed-derived rows.
      `panel.rs` consumes `ResolutionOutcome` (unchanged shape), so the row builder likely needs no structural change, but the DISPLAYED string for formerly-raw-path rows is now DSN-derived. Update the `panel.rs` tests that assert a raw `physical_path` (e.g. `panel_model_from_resolve_output` uses `physical_path:"/data/my/data"`) to assert the new DSN-derived `path_or_message` the pipeline now produces (this is the intended Req 19.3 user-visible change). Add/adjust a test proving a Verified-sourced resolved row shows the DSN identity, not a raw storage path.
      Files: `crates/ff-dsalloc/src/panel.rs` (tests; `result_to_row` only if a display tweak is needed).
      Verify: `cargo test -p ff-dsalloc panel` -- panel tests assert the new DSN-based display and pass.

---

## FEATURE 3 -- Retire ff-vsam-services; fix governance + stale docs (dataset-catalog Task 41.2; ownership Req 22.3; Req 35.3)

MUST come AFTER FEATURE 1 (VSAM wired under DatasetAccess first -- Req 35.3 ordering). Grep-confirm no code dependency before removing.

- [ ] 11. Grep-confirm no live code dependency on ff-vsam-services, then remove the crate directory and its workspace member line.
      Confirm (read, do not assume) that no shipping crate's `Cargo.toml` lists `ff-vsam-services` and no `.rs` has `use ff_vsam_services`. Then delete the `crates/ff-vsam-services/` directory and remove the `"crates/ff-vsam-services",` line from the root `Cargo.toml` `[workspace].members`.
      Files: delete `crates/ff-vsam-services/` (whole dir), edit root `Cargo.toml`.
      Verify: read root `Cargo.toml` and confirm `ff-vsam-services` is absent; `cargo check -p ff-volume` and the ff-dscatalog/ff-dsalloc scoped checks still build (the crate was unreferenced).

- [ ] 12. Re-express the governance rules/tests against ff-dscatalog (no dangling ff-vsam-services references) and convert edited comment separators to ASCII.
      In `crates/ff-governance-tests/src/compliance.rs`: replace the two `crate_name:"ff-vsam-services"` `DependencyRule`s and the `ff-vfs`->`ff-vsam-services` prohibition with rules expressing that `ff-dscatalog` OWNS VSAM and the DAG stays acyclic (e.g. `ff-dscatalog` must not depend on `ff-idcams`/`ff-dsalloc`; `ff-vfs` must not depend on `ff-dscatalog` -- consistent with ownership Req 22.3/22.6). In `crates/ff-governance-tests/tests/architecture_compliance.rs`: remove/replace `ff-vsam-services` from `required_crates` (in `all_governed_crates_exist`) and `dataset_crates` (in `prohibited_rules_cover_all_dataset_crates`); either delete `vsam_services_has_no_upstream_dependencies` or re-point it at `ff-dscatalog`. Ensure `prohibited_rules_cover_all_dataset_crates` still finds a rule for every crate in its (updated) list. Convert any comment separator lines you touch from U+2500 box-drawing to ASCII `// === ... ===` (documentation.md: `.rs` strict ASCII).
      Files: `crates/ff-governance-tests/src/compliance.rs`, `crates/ff-governance-tests/tests/architecture_compliance.rs`.
      Verify: `cargo test -p ff-governance-tests` -- all compliance tests pass with no reference to a non-existent crate; `cargo check -p ff-governance-tests --all-targets` clean.

- [ ] 13. Fix the stale doc comments naming ff-vsam-services as the VSAM authority (comment-only).
      In `crates/ff-dataset-catalog/src/lib.rs` (~line 21) change the "owned by ff-vsam-services" note to name `ff-dscatalog` as the reconciled VSAM authority. In `crates/ff-idcams/src/services.rs` (~line 321) update the doc comment that says the VSAM trait is "implemented by ff-vsam-services" to reference the reconciled `ff-dscatalog` `VsamService` -- COMMENT ONLY; do NOT touch ff-idcams's private `VsamService` trait CODE or any method (that repoint is RC.B.8). Keep edits ASCII; do not introduce em dashes/curly quotes.
      Files: `crates/ff-dataset-catalog/src/lib.rs`, `crates/ff-idcams/src/services.rs`.
      Verify: `grep` for `ff-vsam-services`/`ff_vsam_services` returns only intentional historical/spec references (no code dependency, no "authority" claim); `cargo check -p ff-dscatalog -p ff-dsalloc --all-targets` clean.

- [ ] 14. STOP-AND-REPORT guard (do not cross into RC.B.8).
      If closing resolve-by-DSN (FEATURE 2) or retiring ff-vsam-services (FEATURE 3) appears to FORCE an `ff-idcams` CODE change (beyond the comment in step 13), or to require repointing ff-idcams's private `CatalogService`/`VsamService`, or to touch the editor SAVE path, or to delete `ff-dataset-catalog` -- do NOT proceed. Record the blocker and surface it (send_message warning) rather than entering RC.B.8 scope.
      Files: none (control gate).
      Verify: n/a.

---

## TCR / tasks / docs updates the implementation MUST make (after scoped checks are green)

- [ ] 15. Flip the RC.B.7 TCR rows to PASS and update the stale RC.B.6 deferral rows in place.
      In `docs/quality/TCR.md`: flip the `ff-dscatalog` "Req 34 (VSAM keyed/relative point ops): DEFERRED to RC.B.7 ... NotYetWired" row to a PASS row naming the new VSAM round-trip/positioning tests; add a PASS row for Req 35.3 (VSAM wired under DatasetAccess + ff-vsam-services retired); flip the `ff-dsalloc` "Req 19.3 PARTIAL ... DEFERRED to RC.B.7" row to a PASS row (Req 19.3 now FULL -- Verified/Passed carry a handle; cite the new tests). Use the TCR emoji convention (allowed in TCR.md).
      Files: `docs/quality/TCR.md`.
      Verify: the three rows no longer say DEFERRED/PARTIAL/NotYetWired and name real passing tests.

- [ ] 16. Mark the spec tasks complete.
      `docs/specs/dataset-catalog/tasks.md`: mark Task 41, 41.1, 41.2 `[x]`. `docs/specs/dataset-allocator/tasks.md`: mark Task 21.3 `[x]` and Task 21 overall `[x]` if now complete. Do NOT mark RC.B.8 / Task 42. `docs/project-management/project-master/tasks.md`: mark the RC.B.7 row code-complete-pending the owner's full gate (do NOT mark RC.B.8).
      Files: `docs/specs/dataset-catalog/tasks.md`, `docs/specs/dataset-allocator/tasks.md`, `docs/project-management/project-master/tasks.md`.
      Verify: checkbox markers are only `[ ]`/`[x]`; RC.B.8 untouched.

- [ ] 17. Update the RESUME doc.
      `docs/status/RESUME-dataset-rationalisation.md`: record RC.B.7 as code-complete pending the owner's full `cargo gate --build`: VSAM wired under DatasetAccess over the KSDS/ESDS/RRDS backends; Req 19.3 now FULL; ff-vsam-services removed + governance re-expressed against ff-dscatalog; next is RC.B.8. Keep ASCII.
      Files: `docs/status/RESUME-dataset-rationalisation.md`.
      Verify: the RC.B.7 bullet reflects the delivered state and names the owner full-gate hand-off.

---

## Final hand-off (per testing.md / workflow.md)

- [ ] 18. Run the full scoped verification set (all commands in the "Scoped verification" section), read each log, fix any failures attributable to this change, and re-run until clean. Then STOP and hand off: state exactly which scoped commands ran clean and prompt the owner to run the full gate manually outside Kiro (`cargo gate --build`). Do NOT run `cargo gate` or any `--workspace` build/test here; do NOT commit or push.
      Files: none.
      Verify: every scoped command's log is clean (fmt check, ff-dscatalog/ff-dsalloc/ff-governance-tests check+test, clippy -D warnings, ff-desktop check, ff-volume check, line-limit tool clean, root Cargo.toml confirmed).

## Assumptions / gaps (made explicit)

- Assumed the concrete VSAM backends are reached by an in-crate `VsamBackend` owning the typed providers (FEATURE 1 design decision), because the `ff_vfs::StorageProvider` seam is byte-only and widening it is out of scope. If the implementer finds an existing in-crate mechanism that already bridges open-cluster -> typed backend, prefer reusing it over adding `VsamBackend`.
- Assumed `ResolutionOutcome` (pipeline/panel display DTO) stays string-shaped and only `AllocationOutcome` carries the handle; the handle does not need to reach `ff-desktop` (grep shows no consumer). If a downstream consumer of `ResolutionOutcome` surfaces, thread the DSN (not a raw path).
- Assumed LDS has no concrete backend yet; `VsamBackend::open_for(VsamType::Lds, ..)` returns `UnsupportedOperation` with a documented reason (no capability is lost vs the stub, which returned `NotImplemented`).
