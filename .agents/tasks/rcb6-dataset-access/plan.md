# Implementation Plan -- RC.B.6: DatasetAccess contract + ff-dsalloc handle retarget (CR-CH-059)

Scope: dataset-catalog Task 40 (Req 34.1-34.8) and dataset-allocator Task 21
(Req 19.1-19.6), coordinating volume-model Req 12 (Task 11). Edits go DIRECTLY on
branch `main` in the main workspace -- NO worktree, NO branch, NO commit. Do NOT
touch `.worktrees/vsam-wiring` (dataset-catalog Req 35.5 waste list). This run
delivers a WORKING concrete `DatasetAccess` impl for the NON-VSAM paths
(PS/PO sequential + members) end-to-end; VSAM point()/keyed/relative record ops
are DEFINED with seams reachable but return a typed `NotYetWired` error (RC.B.7
owns the concrete VSAM wiring). Do NOT do RC.B.7 or RC.B.8.

## Design decisions (grounded in the current code)

These are decided here so the implementer does not re-decide them. Each is backed
by a file I read.

1. **New module, not catalog.rs.** `crates/ff-dscatalog/src/catalog.rs` is the
   pre-existing ~640-line >400 violation (rust-standards.md). All new code lands
   in NEW modules under a `dataset_access/` directory so NOTHING grows catalog.rs
   and every file stays < 400 non-test lines. Tests split to a sibling
   `*_tests.rs` once they near ~200 lines (testing.md).

2. **Reuse the existing `Record` type.** `ff_dscatalog::vsam_service::Record`
   (`{ key: Vec<u8>, data: Vec<u8> }`, re-exported from `lib.rs`) is already the
   record DTO. `DatasetAccess::get/put` use THIS `Record` (key empty for
   non-keyed records) -- do NOT introduce a second record struct. Confirmed in
   `vsam_service.rs` and `lib.rs` re-exports.

3. **`DatasetError` is a NEW thin enum that wraps/maps the existing taxonomies.**
   It maps onto `CatalogError` (error.rs), `VolumeError` (ff-volume error.rs),
   `CodecError` (codecs/mod.rs), and across the physical seam onto
   `ff_vfs::VfsError`. `From` impls provide the mapping (Req 34.7). It carries a
   `NotYetWired { operation }` variant for the deferred VSAM record ops. No
   `rusqlite::Error` or `StorageLocator`-internal string ever appears in its
   public surface (Req 34.3, 34.7).

4. **Object-safe via a `dyn`-friendly trait plus a compile-time assertion.** The
   trait takes `&self` / `&mut OpenDataset` (never `self` by value except
   `close`/`dispose` which take the owned handle/open) and uses only object-safe
   signatures, mirroring the existing `ff_vfs::StorageProvider` and
   `VsamService` traits which already compile as `dyn`. A
   `fn _assert_object_safe(_: &dyn DatasetAccess) {}` guard is added (same pattern
   as storage_provider.rs). This satisfies Req 34.8 without a separate companion
   trait; a `DynDatasetAccess` wrapper is NOT needed because no generic methods
   are introduced (unlike CatalogService which needed `DynCatalogService`).

5. **`DatasetHandle` is opaque.** A newtype wrapping a private struct
   (dsn + resolved `DatasetVolume` locator + dsorg/recfm/lrecl snapshot + a
   `VolumeId`), with NO public fields and NO accessor that exposes a raw
   `storage_path` (Req 34.5, 34.3). Consumers (ff-dsalloc, future JES) hold it
   opaquely.

6. **Resolution seam = `ff_volume::VolumeService::resolve_dataset` /
   `DatasetVolumeSet::resolve`.** Confirmed present in
   `ff-volume/src/service.rs` and `dataset_volume.rs` (returns `Vec<ResolvedVolume>`
   honouring the Online check, Req 32.3 / volume-model Req 2.4). DatasetAccess
   takes the primary `ResolvedVolume`'s `locator` string, wraps it as
   `ff_vfs::StorageLocator::new(locator)`, and performs physical I/O through the
   `ff_vfs::StorageProvider` seam (`open` / `write` / `allocate`). ff-volume gains
   NO dependency on ff-dscatalog -- DatasetAccess lives in ff-dscatalog and
   consumes ff-volume (Req 12.3; acyclic DAG unchanged).

7. **SPACE charging seam = `ff_volume::ExtentSet` + `AllocationUnit` + `Volume`.**
   Confirmed in `ff-volume/src/extent.rs` / `space.rs`. `allocate` builds an
   `AllocationUnit` from the DD SPACE request, calls
   `ExtentSet::allocate_primary(dataset_id, &alloc, &geom, &mut volume)` which
   charges the Volume and surfaces `VolumeError::VolumeFull` /
   `SpaceAbend` / `VolumeReadOnly` / `VolumeOffline` unchanged (Req 34.4, 12.4).
   `allocate` first calls `volume.ensure_allocatable()` + `ensure_writable()`
   (via allocate_primary) so an Offline/ReadOnly Volume is rejected
   (Req 34.4, volume-model Req 2.1/2.2).

8. **RECFM codecs = existing `codecs::{FixedCodec, VariableCodec, BinaryCodec}`.**
   Confirmed in `codecs/mod.rs` + `codecs/fixed.rs`. `get`/`put` select the codec
   by the handle's `Recfm`: F/FB -> `FixedCodec::new(lrecl, dsn)`; V/VB ->
   `VariableCodec` (4-byte RDW); U -> `BinaryCodec` (pass-through). Record
   boundaries ALWAYS come from the codec, NEVER a CRLF/LF host text line
   (Req 34.2). `TextCodec` is import/export only and is NOT used by get/put.

9. **Non-VSAM physical backend = `ff_vfs::StorageProvider`.** The concrete
   `DatasetAccess` impl holds an `Arc<dyn ff_vfs::StorageProvider>` (the
   `NativeFileProvider` registered for PS/PO/GDG content, confirmed in
   `storage/native.rs`). PS get reads the whole object via
   `StorageProvider::open(&locator)` then codec-decodes to records; PS put
   codec-encodes records then `StorageProvider::write(&locator, &bytes)`. PO
   members address the member locator. This is the end-to-end working path this
   run delivers.

10. **VSAM paths DEFINED, concrete record op deferred.** `point()` and the
    keyed/relative get/put branches resolve the Volume + locator + reach the KSDS
    (`SqliteRecordProvider`) / ESDS (`NativeEsdsProvider`) / RRDS
    (`SqliteRrdsProvider`) backends, but the concrete VSAM record operation
    returns `DatasetError::NotYetWired { operation }` with a test asserting that.
    This keeps RC.B.7 scope out while proving the plumbing (Req 34.1 method set
    complete, trait object-safe).

11. **ff-dsalloc keeps its own mockable `CatalogProvider` trait; no SQLite, no
    hard ff-dscatalog type dependency in logic.** The current `catalog_bridge.rs`
    is self-contained (trait + `MockCatalog`) and does NOT import
    ff-dataset-catalog -- the "wrong crate name" is only in DOC COMMENTS
    (catalog_bridge.rs header + lib.rs module doc). Req 19.1 is satisfied by
    (a) correcting those doc comments to `ff-dscatalog`, and (b) adding a
    `DatasetAccess` trait object seam that production wires to ff-dscatalog while
    tests mock. ff-dsalloc already has NO `rusqlite` dep (Cargo.toml confirmed);
    Req 19.4 is preserved by NOT adding one. ff-dsalloc does NOT need a new
    `ff-dscatalog` Cargo dependency for the handle type if the handle is defined
    behind the allocator's own trait seam -- but see item 12.

12. **`DatasetHandle` crosses the crate boundary via a dep on ff-dscatalog.**
    `AllocationOutcome` must carry an opaque `DatasetHandle` (Req 19.3). The
    cleanest decision: add `ff-dscatalog = { path = "../ff-dscatalog" }` to
    `ff-dsalloc/Cargo.toml` and re-export `ff_dscatalog::DatasetHandle`. This is
    acyclic (ff-dsalloc -> ff-dscatalog already the intended DAG per design.md)
    and introduces NO rusqlite into ff-dsalloc (ff-dscatalog's rusqlite stays
    internal). The allocator still depends on the catalog through a MOCKABLE
    trait (`CatalogProvider` + a new `DatasetAllocator` trait wrapping
    `DatasetAccess::allocate`), satisfying Req 19.4. Verified ff-dsalloc currently
    has no path deps, so this is the first and is DAG-safe.

## Ordered implementation items

- [ ] 1. Add the `DatasetAccess` supporting types module.
      Create `crates/ff-dscatalog/src/dataset_access/types.rs`: `DatasetHandle`
      (opaque newtype over a private `HandleInner`), `OpenDataset` (holds the
      handle, `AccessIntent`, decoded record cursor / byte buffer, dirty flag),
      `AccessIntent { Read, Write, Update }`, `Positioner { Key(Vec<u8>),
      Rrn(u64) }`, `StepOutcome { Keep, Catlg, Uncatlg, Delete, Pass }`, and
      `DdRequest` (dsn, dsorg, recfm, lrecl, blksize, DISP status/action, SPACE
      as `ff_volume::AllocationUnit` + geometry, optional VOL=SER/UNIT). Re-use
      `ff_dscatalog::Record` (do not redefine). All types `#[non_exhaustive]`
      where they may grow (StepOutcome, AccessIntent, Positioner). ASCII only.
      Files: `crates/ff-dscatalog/src/dataset_access/types.rs`
      Verify: `cargo check -p ff-dscatalog` compiles the new module (added to a
      `dataset_access` mod in step 3).

- [ ] 2. Add the `DatasetError` taxonomy with mapping `From` impls.
      Create `crates/ff-dscatalog/src/dataset_access/error.rs`: `DatasetError`
      (`#[non_exhaustive]`, `thiserror`) with variants covering NotFound,
      AlreadyExists, VolumeUnavailable, SpaceAbend, VolumeFull, ReadOnlyVolume,
      Codec, Vfs, Catalog, InvalidIntent, BadPositioner, and
      `NotYetWired { operation: String }`. Add `From<CatalogError>`,
      `From<VolumeError>`, `From<CodecError>`, `From<ff_vfs::VfsError>`, and
      `From<DatasetError> for ff_vfs::VfsError` (Req 34.7). No `rusqlite` / no
      `StorageLocator` string in the public surface.
      Files: `crates/ff-dscatalog/src/dataset_access/error.rs`
      Verify: `cargo test -p ff-dscatalog dataset_access::error` -- the mapping
      unit tests (step 10) pass; `cargo check -p ff-dscatalog`.

- [ ] 3. Define the object-safe `DatasetAccess` trait.
      Create `crates/ff-dscatalog/src/dataset_access/mod.rs` declaring submodules
      (`types`, `error`, `trait_def`, `impl_access`) and re-exporting the public
      API. Create `crates/ff-dscatalog/src/dataset_access/trait_def.rs` with:
      `allocate(&self, dd: &DdRequest) -> Result<DatasetHandle, DatasetError>`,
      `open(&self, h: &DatasetHandle, intent: AccessIntent) -> Result<OpenDataset,
      DatasetError>`, `get(&self, od: &mut OpenDataset) -> Result<Option<Record>,
      DatasetError>`, `put(&self, od: &mut OpenDataset, rec: &Record) ->
      Result<(), DatasetError>`, `point(&self, od: &mut OpenDataset, pos:
      &Positioner) -> Result<(), DatasetError>`, `close(&self, od: OpenDataset) ->
      Result<(), DatasetError>`, `dispose(&self, h: DatasetHandle, outcome:
      StepOutcome) -> Result<(), DatasetError>`. Trait bound `Send + Sync`. Add
      `fn _assert_object_safe(_: &dyn DatasetAccess) {}` (Req 34.8).
      Files: `crates/ff-dscatalog/src/dataset_access/mod.rs`,
      `crates/ff-dscatalog/src/dataset_access/trait_def.rs`,
      and add `pub mod dataset_access;` + re-exports to
      `crates/ff-dscatalog/src/lib.rs`.
      Verify: `cargo check -p ff-dscatalog`; the object-safety test (step 10)
      holds a `Box<dyn DatasetAccess>`.

- [ ] 4. Implement the concrete `CatalogDatasetAccess` -- allocate + dispose.
      Create `crates/ff-dscatalog/src/dataset_access/impl_access.rs` with a
      struct holding the `VolumeService` (or `&mut VolumeRegistry` + geometry),
      an `Arc<dyn ff_vfs::StorageProvider>`, and the catalog handle it needs for
      metadata. Implement `allocate`: validate Offline/ReadOnly via
      `ExtentSet::allocate_primary` (charges SPACE against the Volume, Req 34.4,
      12.4), create the `DatasetVolume` locator, allocate the physical object via
      `StorageProvider::allocate(dsn)`, and return an opaque `DatasetHandle`.
      Implement `dispose`: apply `StepOutcome` (KEEP/CATLG = retain; UNCATLG =
      drop catalog assoc keep bytes; DELETE = `StorageProvider::delete` + release
      extents; PASS = retain for job scope). Map every error through
      `DatasetError` (Req 34.7). Keep this file < 400 non-test lines -- if it
      grows, split allocate/dispose into `impl_alloc.rs` and open/get/put into
      `impl_io.rs`.
      Files: `crates/ff-dscatalog/src/dataset_access/impl_access.rs`
      Verify: `cargo test -p ff-dscatalog dataset_access` -- allocate/dispose
      tests (step 10) pass.

- [ ] 5. Implement open / get / put / close for the NON-VSAM (PS/PO) paths.
      In `impl_access.rs` (or `impl_io.rs` if splitting): `open` resolves the
      Volume via the resolution seam (item 6 decision), builds an `OpenDataset`
      carrying the `StorageLocator` and the codec selected by RECFM. `get`:
      read bytes via `StorageProvider::open(&locator)`, codec-decode to records,
      advance the cursor, return `Some(Record)` / `None` at EOF. `put`:
      append the record to the open buffer, codec-encode, write via
      `StorageProvider::write(&locator, &bytes)`; honour Volume capacity by
      calling `ExtentSet::accommodate(target_used_bytes)` so growth charges the
      Volume (surfaces VolumeFull / x37 unchanged). `close` flushes and releases.
      Record boundaries come ONLY from the codec (Req 34.2).
      Files: `crates/ff-dscatalog/src/dataset_access/impl_access.rs`
      (or new `crates/ff-dscatalog/src/dataset_access/impl_io.rs`)
      Verify: `cargo test -p ff-dscatalog dataset_access` -- the per-RECFM
      allocate->open->put->get->close round-trip tests (step 10) pass.

- [ ] 6. Define the VSAM `point()` + keyed/relative branches as NotYetWired.
      In the get/put/point code, when the handle is a VSAM cluster
      (`Positioner::Key` for KSDS / `Positioner::Rrn` for RRDS/ESDS): resolve the
      Volume + locator and reach the backend (SqliteRecordProvider /
      NativeEsdsProvider / SqliteRrdsProvider) so the SEAM is proven reachable,
      then return `DatasetError::NotYetWired { operation: "point"/"get(keyed)"/
      "put(keyed)" }`. Do NOT implement the concrete VSAM record op (that is
      RC.B.7). Add a doc comment on each marking it RC.B.7.
      Files: `crates/ff-dscatalog/src/dataset_access/impl_access.rs`
      (or `impl_io.rs`)
      Verify: `cargo test -p ff-dscatalog dataset_access` -- the
      `point_on_ksds_returns_not_yet_wired` test (step 10) asserts the typed
      error.

- [ ] 7. Reconcile the volume-model Req 12 resolution seam usage (no new ff-volume code).
      Confirm `ff_volume::VolumeService::resolve_dataset` /
      `DatasetVolumeSet::resolve` is the resolution path DatasetAccess calls, and
      that ff-volume gains NO dependency on ff-dscatalog (Req 12.3). No source
      change in ff-volume is expected; if a small accessor is missing (e.g. a
      public getter already exists per service.rs), prefer using the existing
      API. This item is a verification + wiring item, not new ff-volume code.
      Files: (read-only) `crates/ff-volume/src/service.rs`,
      `crates/ff-volume/src/dataset_volume.rs`
      Verify: `cargo check -p ff-volume -p ff-dscatalog` -- DAG stays acyclic;
      `ff-volume/Cargo.toml` has no `ff-dscatalog` dep.

- [ ] 8. Correct the ff-dsalloc crate-name doc comments and add the ff-dscatalog dep.
      In `crates/ff-dsalloc/src/catalog_bridge.rs` header and
      `crates/ff-dsalloc/src/lib.rs` module doc, replace every `ff-dataset-catalog`
      / `ff_dataset_catalog` mention with `ff-dscatalog` (Req 19.1). Add
      `ff-dscatalog = { path = "../ff-dscatalog" }` to `crates/ff-dsalloc/Cargo.toml`
      `[dependencies]`. Do NOT add `rusqlite` (Req 19.4).
      Files: `crates/ff-dsalloc/src/catalog_bridge.rs`,
      `crates/ff-dsalloc/src/lib.rs`, `crates/ff-dsalloc/Cargo.toml`
      Verify: `cargo check -p ff-dsalloc`; `grep` finds no live
      `ff[-_]dataset[-_]catalog` reference in ff-dsalloc sources.

- [ ] 9. Reshape `AllocationOutcome` to carry a `DatasetHandle` and add the allocator seam.
      Add a mockable `DatasetAllocator` trait to
      `crates/ff-dsalloc/src/catalog_bridge.rs` with
      `allocate(&self, dd: &DdRequest-equivalent) -> Result<DatasetHandle,
      CatalogError>` (production wires to `ff_dscatalog::DatasetAccess::allocate`;
      a `MockDatasetAllocator` returns a canned handle). Change
      `AllocationOutcome::{Verified, Allocated}` in
      `crates/ff-dsalloc/src/allocation.rs` to carry
      `handle: ff_dscatalog::DatasetHandle` INSTEAD of `physical_path: String`
      (Req 19.3). Preserve `WouldAllocate { dsn }` for dry-run (NO handle
      acquired -- Req 19.5). Preserve `PassTable`/DISP/PASS logic and the Task 19
      Volume charging/failure semantics (Req 19.5); DISP/SPACE parsing stays in
      ff-dsalloc. Update `simulate_allocation` to call the allocator seam in Live
      mode and to NOT acquire a handle in DryRun. Re-export
      `ff_dscatalog::DatasetHandle` from `ff-dsalloc/src/lib.rs`.
      Files: `crates/ff-dsalloc/src/catalog_bridge.rs`,
      `crates/ff-dsalloc/src/allocation.rs`, `crates/ff-dsalloc/src/lib.rs`
      Verify: `cargo test -p ff-dsalloc` -- existing allocation tests updated to
      assert a handle (not a path) pass; dry-run test asserts no handle.

- [ ] 10. Write the full TDD test suite (red before green), each mapped to a Req.
      Add tests (split into `dataset_access/*_tests.rs` sibling files so no source
      file exceeds 400 non-test lines; test files split before ~200 lines).
      ff-dscatalog tests:
      - `allocate_open_put_get_close_round_trips_fixed_fb` (Req 34.1, 34.2)
      - `round_trips_variable_vb_with_rdw` (Req 34.2)
      - `round_trips_recfm_u_passthrough` (Req 34.2)
      - `po_member_round_trip` (Req 34.1, 34.2)
      - `get_put_use_codec_boundaries_not_crlf` (Req 34.2)
      - `resolves_via_volume_datasetvolume_locator` and
        `physical_io_through_storage_provider_seam` (Req 34.3, volume-model 12.1,
        12.2)
      - `allocate_rejects_offline_volume` / `allocate_rejects_readonly_volume`
        (Req 34.4, volume-model 2.1/2.2)
      - `allocate_charges_space_against_volume_and_surfaces_volume_full` and
        `..._surfaces_x37_space_abend` (Req 34.4, volume-model 12.4)
      - `allocate_returns_opaque_handle` + a compile-time check that
        `DatasetHandle` exposes no raw path (Req 34.5)
      - `dataset_error_maps_to_vfs_error` + `..._from_catalog_volume_codec`
        (Req 34.7)
      - `dataset_access_is_object_safe_as_dyn` (Req 34.8)
      - `point_on_ksds_returns_not_yet_wired` /
        `keyed_get_returns_not_yet_wired` (Req 34.1 defined; RC.B.7 deferred)
      - `dispose_keep_catlg_uncatlg_delete_pass` (Req 34.1 dispose)
      ff-dsalloc tests:
      - `catalog_bridge_names_ff_dscatalog` (doc/grep + trait binding) (Req 19.1)
      - `allocation_returns_handle_not_path` (Req 19.2, 19.3)
      - `dry_run_acquires_no_handle` (Req 19.5)
      - `outcome_carries_opaque_handle` (Req 19.3)
      - `allocator_has_no_rusqlite_and_mockable_trait` (Req 19.4)
      Each test carries `// Validates: Requirement X.Y`. Run red first.
      Files: `crates/ff-dscatalog/src/dataset_access/impl_access_tests.rs` (+
      `types_tests.rs`, `error_tests.rs` as needed),
      `crates/ff-dsalloc/src/allocation.rs` (#[cfg(test)]),
      `crates/ff-dsalloc/src/catalog_bridge.rs` (#[cfg(test)])
      Verify: `cargo test -p ff-dscatalog -p ff-dsalloc` -- all new tests pass
      green; `cargo test -- --nocapture` shows no CRLF record delimiters.

- [ ] 11. Enforce the 400-line rule and ASCII; split if needed.
      After implementation, check every new/changed `.rs` is < 400 non-test
      lines; if `impl_access.rs` exceeds, split into `impl_alloc.rs` +
      `impl_io.rs` with `mod.rs` as the thin coordinator. Confirm ASCII-only.
      Files: all new `crates/ff-dscatalog/src/dataset_access/*.rs`
      Verify: `rg -c "" crates/ff-dscatalog/src/dataset_access/*.rs` (via the
      allowed tool) shows each under 400 excluding tests; `cargo fmt -- --check`
      and `cargo clippy -p ff-dscatalog -p ff-dsalloc -- -D warnings` clean.

- [ ] 12. Flip the TCR rows to PASS and reconcile the volume-model Req 12 duplicate-row hygiene.
      In `docs/quality/TCR.md` (Phase dataset-stack-rationalisation section):
      flip dataset-catalog Req 34.1-34.8 (ff-dscatalog) and dataset-allocator
      Req 19.1-19.6 (ff-dsalloc) from the current state to the test name(s) added
      in step 10. For volume-model Req 12.1-12.4: these rows are currently
      attributed to crate `ff-volume` but the behaviour they assert
      (DatasetAccess resolves via the locator; charges SPACE) is now EXERCISED by
      the ff-dscatalog DatasetAccess tests -- update each row IN PLACE (append-only
      rule: do not delete) to point at the ff-dscatalog DatasetAccess test names
      and note the test crate is `ff-dscatalog` (the resolution/charge consumer),
      keeping ff-volume as the model owner. Do NOT create a second set of Req 12
      rows -- reconcile the existing ones (this is the RC.B.5 duplicate-row
      hygiene the brief calls for). Do NOT touch RC.B.7 (Req 35.3) / RC.B.8 rows.
      Files: `docs/quality/TCR.md`
      Verify: `rg "Req 34\.|Req 19\.|volume-model Req 12\."` shows no remaining
      red circle for the RC.B.6 criteria and no duplicate Req 12 rows.

- [ ] 13. Mark the tasks.md / project-master rows.
      Mark `docs/specs/dataset-catalog/tasks.md` Task 40 (40.1-40.5) and
      `docs/specs/dataset-allocator/tasks.md` Task 21 (21.1-21.7) items `[x]`
      ONLY for what this run completes (leave VSAM-concrete sub-points honest if
      deferred; note the NotYetWired VSAM path). Mark
      `docs/specs/volume-model/tasks.md` Task 11 as satisfied by DatasetAccess.
      In `docs/project-management/project-master/tasks.md` mark RC.B.6 as
      code-complete pending the owner's full gate (do NOT mark DONE -- owner runs
      the full gate). Do NOT pre-mark RC.B.7/B.8.
      Files: `docs/specs/dataset-catalog/tasks.md`,
      `docs/specs/dataset-allocator/tasks.md`,
      `docs/specs/volume-model/tasks.md`,
      `docs/project-management/project-master/tasks.md`
      Verify: visual check -- every checkbox is `[ ]` or `[x]` with a title;
      no RC.B.7/B.8 row changed.

- [ ] 14. Scoped verification + hand off the full gate.
      Run the SCOPED checks only (per testing.md / tooling.md; use the
      non-interactive pwsh7 wrapper): `cargo fmt`, `cargo check -p ff-dscatalog
      -p ff-dsalloc`, `cargo clippy -p ff-dscatalog -p ff-dsalloc -- -D warnings`,
      `cargo test -p ff-dscatalog -p ff-dsalloc`. Do NOT run `--workspace` or
      `cargo gate`. Confirm clean, then STATE which scoped commands ran and PROMPT
      the owner to run `cargo gate --build` outside Kiro.
      Verify: all four scoped commands report clean; hand-off message emitted.

## Constraints checklist (must hold at completion)

- Acyclic DAG unchanged: ff-dsalloc -> ff-dscatalog -> ff-volume -> ff-vfs;
  ff-volume gains NO dep on ff-dscatalog (item 7, 8, 12).
- `DatasetAccess` object-safe (`dyn DatasetAccess` holds in a test) (item 3, 10).
- ASCII-only `.rs`; every new/changed `.rs` < 400 non-test lines; `catalog.rs`
  NOT grown (item 1, 11).
- No raw `storage_path` / SQLite / `StorageLocator`-internal leaks through the
  `DatasetAccess` surface or `DatasetHandle` (item 1, 2, 4).
- `.worktrees/vsam-wiring` untouched; no VSAM concrete record op implemented
  (RC.B.7 stays out) (item 6).
- No crate deleted; no `rusqlite` added to ff-dsalloc (item 8, 9).
- TDD: every implementation item is preceded by a failing test from item 10;
  each test maps to a Requirement.

## Notes / assumptions

- The allocator's existing `catalog_bridge.rs` does NOT currently import the
  wrong crate in code -- only doc comments reference `ff-dataset-catalog`
  (confirmed by reading the file). Req 19.1 is therefore primarily a doc-comment
  correction plus the trait-seam binding; no removed-crate reference exists to
  break (Req 19.6 ordering is already satisfied -- ff-dataset-catalog is not
  deleted until RC.B.8).
- `DdRequest` carries SPACE as an `ff_volume::AllocationUnit` + geometry rather
  than re-parsing JCL in ff-dscatalog -- the JCL SPACE parse stays in ff-dsalloc
  (dataset-allocator owns it) and is passed through, preserving Req 17/18.
- If `OpenDataset` for PS needs the whole object in memory, that is acceptable
  for this emulation (NativeFileProvider::open returns the full byte vector);
  streaming is not required by Req 34.
