# Implementation Plan: RC.B.5 -- ff-volume crate + ff-dscatalog schema v4

Phase RC.B STEP 5 of the Mainframe Dataset Stack Rationalisation (CR-CH-059).
OWNER-APPROVED gated work (CR-CH-059; volume-model Req 1-11 gated under
CR-NR-105 / CR-CH-057; dataset-catalog Req 32; dataset-ownership-model Req 21/22).

## Operating rules for the coder

- Edits go DIRECTLY on branch `main` in the main workspace at
  `c:\workspace\VSC\FileForgeWorkbench`. NO worktree, NO new branch, NO commit --
  leave the tree uncommitted for owner review. (The restructuring instructions'
  "commit locally" guidance is OVERRIDDEN by the task's explicit "NO commit"; do
  not create commits.)
- TDD per `testing.md`: write the failing test first (confirm red), then the
  minimum implementation (confirm green). Every test carries a
  `// Validates: Requirement X.Y` comment.
- 400-non-test-line rule per `rust-standards.md`: split by concern BEFORE a file
  exceeds 400 non-test lines. Split test modules into a sibling `*_tests.rs` when
  the test module alone exceeds ~200 lines.
- Plain ASCII only in `.rs` files; ASCII section separators
  `// === Name ===`. Watch for an em dash: the existing
  `crates/ff-dscatalog/Cargo.toml` `description` line contains a U+2014 em dash
  ("emulation —"); do NOT copy that pattern into the new `ff-volume` Cargo.toml,
  and if touching that line replace `—` with `--`.
- SCOPED checks ONLY. After each FEAT and at the end, run:
  `cargo check -p ff-volume`, `cargo test -p ff-volume`,
  `cargo clippy -p ff-volume`, then the same for `-p ff-dscatalog`, then
  `cargo fmt`. NEVER run `--workspace` or `cargo gate` -- that is the owner's
  manual step. Use the pwsh7 non-interactive wrapper per `tooling.md`:
  `C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "<cmd>"`.
- Framework-conformance: `ff-volume` is a NARROW model crate. It MUST NOT depend
  on `ff-dscatalog`, `ff-dsalloc`, or `ff-idcams` (acyclic DAG, enforced by
  construction: `ff-dscatalog -> ff-volume -> ff-vfs`). It may depend on `ff-vfs`
  (for the `StorageProvider` URI / `StorageLocator` seam), `thiserror`, `serde`,
  `chrono`.

## SCOPE BOUNDARY (do NOT build)

RC.B.5 is volume-model tasks 1-10 PLUS the ff-dscatalog schema-v4 side
(dataset-catalog tasks 33-37). Do NOT build any of:
- The `DatasetAccess` trait (RC.B.6; dataset-catalog task 40, volume-model task 11
  is reference-only).
- VSAM wiring under DatasetAccess / retiring `ff-vsam-services` (RC.B.7).
- idcams repoint, record-aware SAVE / MAINFRAME BackendEnvironment (RC.B.8).
- The Volume UI / VTOC `WorkspaceContext` and `DEFINE VOLUME` command DISPATCH
  wiring (RC.C.10). RC.B.5 delivers only the SERVICE API and the command
  CONTRACT (types + documented method set), NOT the dispatch arm or the panel.
- Deleting `ff-dataset-catalog` or `ff-vsam-services`.
- Any edit under `.worktrees/vsam-wiring`.

## Design decisions made here (not re-deciding the gated design)

The volume-model design.md fixes the entity model, geometry algorithm, two
failure points, ownership, and dual-read migration. These are the implementation
choices the design left open:

1. **ff-volume is a pure in-memory model crate, not a persistence layer.** The
   `volumes` / `dataset_volumes` SQLite tables are owned and persisted by
   `ff-dscatalog` (design.md: "The Volume ROW type is persisted by the catalog,
   but the Volume TYPE and its behaviour ... are owned by `ff-volume`").
   Therefore `ff-volume` exposes plain Rust types + behaviour + a `VolumeStore`
   trait/registry operating on in-memory collections; `ff-dscatalog` maps rows
   to/from these types. This keeps the DAG acyclic and keeps `rusqlite` out of
   `ff-volume`.
2. **Default `bytes_per_track = 56664`, `tracks_per_cylinder = 15`** (the design's
   worked 3390-style example). Fixed as documented `pub const` values.
3. **Capacity counters are stored in TRACKS** (the smallest emulated unit);
   cylinders are a derived view. `Volume` carries `total_tracks` / `used_tracks`;
   free = total - used. This matches Req 7.1 ("in tracks or cylinders") and keeps
   one canonical unit for arithmetic.
4. **IDs are newtypes** (`VolumeId(u64)`, `DatasetId(u64)`) per rust-standards
   "newtypes over bare primitives". `ff-dscatalog` supplies the row ids.
5. **Errors live in one `VolumeError` enum** (`thiserror`, `#[non_exhaustive]`)
   with the five distinct variants from design.md's failure table
   (`SpaceAbend`, `VolumeFull`, `VolumeOffline`, `VolumeReadOnly`,
   `VolumeNotFound`) plus `DuplicateVolser`. The two space failures are DISTINCT
   variants (Req 5.5, 7.3).
6. **`AccessMode` naming clash:** `ff-dscatalog::vsam_service` already exports an
   `AccessMode`. The new volume access mode is `ff_volume::AccessMode`
   {ReadWrite, ReadOnly}; it is a different type in a different crate -- no
   collision at definition, but when `ff-dscatalog` imports it, use a qualified
   path `ff_volume::AccessMode` (do NOT glob-import) to avoid shadowing the vsam
   one.
7. **Schema v4 migration is additive + dual-read:** keep the `storage_path`
   column; add `volumes` + `dataset_volumes`; the forward migration seeds rows.
   Resolution reads `dataset_volumes` first and falls back to `storage_path` for
   unmigrated rows. No `DROP COLUMN`, no byte movement.

## Module decomposition for `crates/ff-volume/src/` (400-line rule)

Split by concern from the start so no file approaches 400 non-test lines:

- `lib.rs` -- crate docs, `pub mod` declarations, curated re-exports. Thin.
- `error.rs` -- `VolumeError` enum (thiserror, non_exhaustive).
- `volume.rs` -- `Volume`, `VolumeId`, `VolumeStatus`, `AccessMode`, `Volser`
   newtype + uniqueness helper, capacity counters, status/access behaviour
   (set-online/offline, mount/unmount, allocation/write/extend guards).
- `geometry.rs` -- `GeometryProfile`, default consts, deterministic
   byte<->track<->cylinder conversions (ceil).
- `space.rs` -- `AllocationUnit` (`UnitKind` {Trk, Cyl, AvgRec}, primary,
   secondary), block/avg-record -> units via geometry.
- `extent.rs` -- `Extent` metadata type, `ExtentSet` accounting, `Max_Extents`
   (default 16), the overflow decision (fit / acquire secondary / x37 Space_Abend
   / Volume_Full), interacting with a `Volume`'s capacity.
- `dataset_volume.rs` -- `DatasetVolume` association (sequence, is_primary,
   locator), multivolume ordered sequence, uncataloged (VOL=SER + UNIT)
   resolution helpers.
- `reporting.rs` -- derived dataset/volume reporting counters and `VtocView`
   (datasets + extents on a volume + usage counters).
- `service.rs` -- `VolumeService` (DEFINE VOLUME contract: define by name/VOLSER,
   path, capacity, status; duplicate-VOLSER rejection), the resolution seam
   (Dataset -> DatasetVolume -> Volume -> locator with Online check), the
   `DEFINE VOLUME` + volume-listing command CONTRACT (a documented method set +
   a command-name/param descriptor struct; NO ff-command dispatch wiring).
- `migration.rs` -- the metadata-only migration seam: given an existing
   Repository root, construct a `Volume` + per-dataset `DatasetVolume` rows
   WITHOUT moving bytes (the pure logic `ff-dscatalog`'s v3->v4 migration calls).

Keep each test module inline under `#[cfg(test)]`; if any grows past ~200 lines,
move it to `<module>_tests.rs`.

---

## Ordered implementation items

### ff-volume crate (volume-model tasks 1-10)

- [ ] 1. Scaffold the `ff-volume` crate and wire it into the workspace.
      Create `crates/ff-volume/Cargo.toml` (package name `ff-volume`, workspace
      version/edition/authors/license; deps: `ff-vfs = { path = "../ff-vfs" }`,
      `thiserror = { workspace = true }`, `serde = { workspace = true }`,
      `chrono = { workspace = true }`; dev-deps `pretty_assertions`, `proptest`,
      `tempfile` from workspace). ASCII-only `description` (no em dash). Add
      `"crates/ff-volume"` to the `members` list in the root `Cargo.toml` (place
      it next to the other `ff-ds*` members). Create `src/lib.rs` with crate docs
      and the `pub mod` list from the decomposition above (empty modules for now).
      Files: `crates/ff-volume/Cargo.toml`, `crates/ff-volume/src/lib.rs`,
      `Cargo.toml`.
      Verify: `cargo check -p ff-volume` compiles (empty crate).

- [ ] 2. Volume entity + VOLSER uniqueness (volume-model task 1.2, 1.3; Req 1).
      In `volume.rs` define `VolumeId(u64)`, `Volser(String)` (uppercased,
      validated non-empty) newtypes; `VolumeStatus {Online, Offline}` and
      `AccessMode {ReadWrite, ReadOnly}` (both `#[non_exhaustive]`, derive Debug/
      Clone/Copy/PartialEq/Eq; Serialize/Deserialize); the `Volume` struct
      (volume_id, volser, display_name: Option<String>, storage_uri: String,
      status, access_mode, total_tracks, used_tracks, reserved_tracks). Add a
      `VolumeRegistry` (in-memory `Vec`/`HashMap`) with `define`/insert enforcing
      VOLSER uniqueness (returns `VolumeError::DuplicateVolser`).
      In `error.rs` define the `VolumeError` enum (see decision 5).
      Files: `crates/ff-volume/src/volume.rs`, `crates/ff-volume/src/error.rs`,
      `crates/ff-volume/src/lib.rs` (re-exports).
      Verify: `cargo test -p ff-volume volser` -- a test that two Volumes with the
      same VOLSER is rejected and distinct VOLSERs are accepted passes.
      Tests (all `// Validates:`): `volume_carries_required_fields` (Req 1.1),
      `duplicate_volser_is_rejected` (Req 1.2), `volser_is_stored_uppercase`.

- [ ] 3. Status / access-mode behaviour (volume-model task 2; Req 2).
      In `volume.rs` add `set_online`/`set_offline`/`mount`/`unmount`
      transitions and guard methods: `ensure_allocatable` (rejects on Offline ->
      `VolumeError::VolumeOffline`), `ensure_writable` (rejects on ReadOnly ->
      `VolumeError::VolumeReadOnly`) used for write/delete/extend, and a
      resolution helper that checks a slice of required Volumes are Online and
      returns the FIRST unavailable one.
      Files: `crates/ff-volume/src/volume.rs`.
      Verify: `cargo test -p ff-volume status` passes.
      Tests: `offline_volume_rejects_new_allocation` (Req 2.1),
      `readonly_volume_rejects_write_delete_extend` (Req 2.2),
      `set_online_offline_mount_unmount_transition` (Req 2.3),
      `resolution_reports_first_offline_volume` (Req 2.4). (Req 2.5 command
      invocability is the contract in item 10; note it, do not wire dispatch.)

- [ ] 4. Geometry profile + conversions (volume-model task 3; Req 3, 11.1).
      In `geometry.rs`: `GeometryProfile { bytes_per_track: u64,
      tracks_per_cylinder: u64 }`, `pub const DEFAULT_BYTES_PER_TRACK: u64 =
      56664`, `pub const DEFAULT_TRACKS_PER_CYLINDER: u64 = 15`, `Default` impl.
      Methods `tracks(bytes) = ceil(bytes / bytes_per_track)`,
      `cylinders(tracks) = ceil(tracks / tracks_per_cylinder)`,
      `bytes_of(tracks) = tracks * bytes_per_track`. Document determinism in
      `///` docs. Use checked/ceil integer math (no float).
      Files: `crates/ff-volume/src/geometry.rs`, `lib.rs`.
      Verify: `cargo test -p ff-volume geometry` passes.
      Tests: `bytes_to_tracks_ceils` (worked example 1_000_000 bytes -> 18 tracks)
      (Req 3.2), `tracks_to_cylinders_ceils` (18 tracks -> 2 cyl) (Req 3.2),
      `conversions_are_deterministic` (Req 3.3, 11.1),
      `defaults_are_3390_style` (Req 3.1).

- [ ] 5. Allocation-unit model (volume-model task 4; Req 4).
      In `space.rs`: `UnitKind {Trk, Cyl, AvgRec { avg_record_len: u64 }}`,
      `AllocationUnit { kind: UnitKind, primary: u64, secondary: u64 }`. Method
      `primary_tracks(&self, geom: &GeometryProfile) -> u64` and
      `secondary_tracks(...)` converting the request to tracks (TRK -> as-is;
      CYL -> *tracks_per_cylinder; AvgRec -> bytes = avg*qty then geom.tracks()).
      Document that JCL SPACE keyword PARSING stays in `ff-dsalloc` -- this is the
      unit MODEL only (Req 4.4). Do NOT add any SPACE string parser here.
      Files: `crates/ff-volume/src/space.rs`, `lib.rs`.
      Verify: `cargo test -p ff-volume space` passes.
      Tests: `trk_primary_maps_to_tracks` (Req 4.1),
      `cyl_primary_maps_via_tracks_per_cyl` (Req 4.2),
      `avgrec_maps_to_units_via_geometry` (Req 4.3),
      `secondary_quantity_recorded` (Req 4.5).

- [ ] 6. Extent model + the two overflow decisions (volume-model task 5+6;
      Req 5, 6; and the Volume_Full branch coordinating with item 7).
      In `extent.rs`: `Extent { dataset_id: DatasetId, volume_id: VolumeId,
      sequence_number: u32, allocated_tracks: u64, used_bytes: u64, locator:
      String }`; `pub const DEFAULT_MAX_EXTENTS: u32 = 16`. An `ExtentSet`
      (ordered extents for one dataset) with `allocated_tracks()`,
      `extents_used()`, and the core `accommodate(write_bytes, alloc:
      &AllocationUnit, geom, volume: &mut Volume, max_extents) ->
      Result<(), VolumeError>` implementing the design's decision tree:
      fit -> ok; else if `extents_used < max_extents` AND volume free tracks >=
      secondary -> acquire secondary (charge volume, push extent); else if
      extents_used == max_extents -> `VolumeError::SpaceAbend { dataset, reason }`
      (x37, prior content intact -- do not mutate on failure); else if volume
      lacks free space -> `VolumeError::VolumeFull { volser }` and DO NOT push an
      extent (Req 7.4). Reason enum distinguishes CapacityExceeded /
      MaxExtentsReached.
      Files: `crates/ff-volume/src/extent.rs`, `lib.rs` (and `VolumeError`
      variants in `error.rs` if not already present).
      Verify: `cargo test -p ff-volume extent` passes.
      Tests: `write_within_allocated_space_proceeds` (Req 5.1),
      `overflow_acquires_secondary_extent` (Req 5.2, 6.2),
      `max_extents_reached_returns_x37_space_abend` (Req 5.3, 6.3),
      `space_abend_leaves_prior_content_intact` (Req 5.4),
      `max_extents_is_configurable` (Req 6.4),
      `space_abend_distinct_from_volume_full` (Req 5.5, 6.4, 7.3).

- [ ] 7. Volume capacity + Volume_Full (volume-model task 6; Req 7).
      In `volume.rs` add capacity accessors `total_tracks`/`used_tracks`/
      `free_tracks` and `charge(tracks)` / `release(tracks)` that update
      `used_tracks` and reject when free < requested with
      `VolumeError::VolumeFull`. Ensure `extent.rs::accommodate` calls `charge`
      only after confirming free space, so a Volume_Full path consumes no extent
      (Req 7.4). This item makes the Volume_Full branch of item 6 real.
      Files: `crates/ff-volume/src/volume.rs`.
      Verify: `cargo test -p ff-volume capacity` passes.
      Tests: `volume_tracks_used_free_accounting` (Req 7.1),
      `allocation_beyond_free_space_reports_volume_full` (Req 7.2),
      `volume_full_does_not_consume_dataset_extent` (Req 7.4).

- [ ] 8. Reporting counters + VTOC view (volume-model task 7; Req 8, 11.2).
      In `reporting.rs`: `DatasetSpaceReport { extents_used, extents_remaining,
      tracks_in_use, cylinders_in_use }` derived from an `ExtentSet` + geometry;
      `VolumeSpaceReport { total/used/free tracks + cylinders, extents_allocated }`;
      `VtocView { volume: VolumeSpaceReport, datasets: Vec<(DatasetId,
      DatasetSpaceReport)> }` derived read view. All O(n) in extents, O(1) for a
      single-extent dataset; no byte scanning (Req 11.2).
      Files: `crates/ff-volume/src/reporting.rs`, `lib.rs`.
      Verify: `cargo test -p ff-volume reporting` passes.
      Tests: `dataset_report_extents_and_tracks` (Req 8.1),
      `volume_report_total_used_free` (Req 8.2),
      `vtoc_view_lists_datasets_and_counters` (Req 8.3),
      `reporting_is_derived_not_scanned` (Req 8.4, 11.2).

- [ ] 9. DatasetVolume association + multivolume + uncataloged (volume-model
      task 8; Req 9).
      In `dataset_volume.rs`: `DatasetVolume { dataset_id, volume_id,
      sequence_number: u32, is_primary: bool, locator: String }`. A
      `DatasetVolumeSet` (ordered by sequence) supporting multivolume; a resolver
      that, given a `VolumeRegistry` + `DatasetVolumeSet`, returns ordered
      (Volume, locator) honouring the Online check; and
      `resolve_uncataloged(volser, unit, registry) -> Result<...>` locating by
      VOL=SER + UNIT with no catalog row. Document that cross-catalog sharing is a
      model-level allowance (the registry is catalog-agnostic).
      Files: `crates/ff-volume/src/dataset_volume.rs`, `lib.rs`.
      Verify: `cargo test -p ff-volume dataset_volume` passes.
      Tests: `datasetvolume_replaces_storage_path` (Req 9.1),
      `multivolume_ordered_sequence` (Req 9.2),
      `shared_volume_across_catalogs` (Req 9.3),
      `uncataloged_resolves_by_volser_unit` (Req 9.4, 9.5).

- [ ] 10. DEFINE VOLUME service + command CONTRACT (volume-model task 9; Req 10)
      and the migration seam (volume-model task 10; Req 11.3).
      In `service.rs`: `VolumeService` holding a `VolumeRegistry` with
      `define_volume(DefineVolumeRequest { volser, display_name, path, capacity,
      status }) -> Result<VolumeId, VolumeError>` (duplicate-VOLSER rejection via
      the registry), plus a resolution method (Dataset -> DatasetVolume -> Volume
      -> locator with Online check) that `DatasetAccess` will later consume
      (document it as the RC.B.6 seam; do NOT build DatasetAccess). Define the
      command CONTRACT as data only: a `DEFINE_VOLUME_COMMAND: &str` name const +
      a `DefineVolumeParams` descriptor and a `VolumeListingParams` /
      `VtocCommand` descriptor, with `///` docs stating these resolve through the
      single command-dispatch path when wired in RC.C.10. NO `ff-command`
      registration, NO WorkspaceContext.
      In `migration.rs`: `fn volume_over_repository(root: &str, volser: &str,
      capacity_tracks: u64) -> Volume` and `fn seed_dataset_volumes(datasets:
      &[(DatasetId, String /*storage_path*/)], volume_id) ->
      Vec<DatasetVolume>` (locator = storage_path, sequence 1, is_primary true),
      moving NO bytes -- pure metadata construction.
      Files: `crates/ff-volume/src/service.rs`,
      `crates/ff-volume/src/migration.rs`, `lib.rs`.
      Verify: `cargo check -p ff-volume && cargo test -p ff-volume service` and
      `... migration` pass; `cargo clippy -p ff-volume` clean; `cargo fmt`.
      Tests: `define_volume_registers_and_rejects_duplicate_volser`
      (Req 10.1, 10.3), `define_volume_accepts_name_path_capacity_status`
      (Req 10.1), `migration_builds_volume_and_datasetvolumes_without_bytes`
      (Req 11.3). (Req 10.2/10.4/10.5 dispatch + visibility + WorkspaceContext are
      RC.C.10 -- documented in the contract, not built.)

### ff-dscatalog schema v4 side (dataset-catalog tasks 33-37; Req 32)

- [ ] 11. Depend on `ff-volume` (dataset-catalog task 35; Req 32.8).
      Add `ff-volume = { path = "../ff-volume" }` to
      `crates/ff-dscatalog/Cargo.toml` `[dependencies]`. Do NOT redefine the
      Volume type in the catalog -- import `ff_volume::{Volume, VolumeStatus,
      AccessMode as VolumeAccessMode, ...}` with qualified names (decision 6).
      Also fix the em dash in the existing `description` line (`—` -> `--`) since
      the file is being touched (documentation.md).
      Files: `crates/ff-dscatalog/Cargo.toml`.
      Verify: `cargo check -p ff-dscatalog` compiles with the new dep.

- [ ] 12. Schema v4: add `volumes` + `dataset_volumes` tables and bump
      SCHEMA_VERSION 3 -> 4 (dataset-catalog task 33; Req 32.1, 32.2).
      In `crates/ff-dscatalog/src/schema.rs`: change `SCHEMA_VERSION` to `"4"`;
      extend the version-history doc comment; add `CREATE_VOLUMES_TABLE` and
      `CREATE_DATASET_VOLUMES_TABLE` consts matching dataset-catalog design.md
      exactly (volumes: volume_id PK, volser UNIQUE NOT NULL, storage_uri NOT
      NULL, status, access_mode, total_units, used_units; dataset_volumes:
      dataset_id REFERENCES datasets(id), volume_id REFERENCES volumes(volume_id),
      sequence_number NOT NULL, is_primary NOT NULL, locator NOT NULL, PRIMARY KEY
      (dataset_id, sequence_number)). Add both to `initialize_database` (CREATE on
      fresh DBs). Append the v3->v4 forward migration to `MIGRATIONS` (index 2 ->
      `("4", <sql creating the two tables + seeding>)`); the migration SQL creates
      the tables if absent, then runs the seed (item 13).
      Note the `MIGRATIONS[idx]` arithmetic: `idx = (v - 1)`, so v3->v4 is index 2;
      the existing `migration_from_v1_to_v2_*` test asserts the chain ends at "3"
      and MUST be updated to expect "4" (and a new chained-to-v4 assertion added).
      Files: `crates/ff-dscatalog/src/schema.rs`.
      Verify: `cargo test -p ff-dscatalog schema` and `... initialize_creates`
      pass (after updating the version-string assertions in existing tests).
      Tests: `schema_v4_creates_volumes_and_dataset_volumes` (Req 32.1, 32.2),
      `fresh_db_reports_version_4`.

- [ ] 13. Forward migration v3->v4 that seeds a Volume per Repository and a
      DatasetVolume per dataset, deriving locator from storage_path, moving NO
      bytes (dataset-catalog task 33.3; Req 32.5).
      Because the plain SQL-batch migration cannot see the Repository root path or
      derive a VOLSER, implement the seed as a Rust migration step rather than a
      pure SQL string: add a `migrate_v3_to_v4(conn, repo_root, catalog_name)`
      function in schema.rs (or a new `schema_v4.rs` module if schema.rs nears 400
      lines) that (a) CREATEs the two tables, (b) INSERTs one `volumes` row
      (volser derived from `catalog_name`, uppercased; storage_uri = repo_root;
      status 'Online'; access_mode 'ReadWrite'; capacity from
      `ff_volume::migration::volume_over_repository`), (c) for each `datasets` row
      INSERTs a `dataset_volumes` row (locator = that row's `storage_path`,
      sequence_number 1, is_primary 1) using `ff_volume::migration::
      seed_dataset_volumes`. Wire this into the mount path where the Repository
      root + catalog name are known (see `repository.rs` / `catalog.rs` mount);
      the pure `apply_migrations(conn)` still bumps the version and creates tables
      so a connection-only migration never corrupts the DB, and the row-seeding
      runs when the catalog context is available. Keep `storage_path` (NO DROP).
      Files: `crates/ff-dscatalog/src/schema.rs` (or new `schema_v4.rs`),
      `crates/ff-dscatalog/src/catalog.rs` or `repository.rs` (invoke the seed on
      mount), `lib.rs` if a new module.
      Verify: `cargo test -p ff-dscatalog migration` passes.
      Tests: `migration_v3_to_v4_seeds_volume_per_repository` (Req 32.5),
      `migration_v3_to_v4_populates_dataset_volumes_from_storage_path` (Req 32.5),
      `migration_v3_to_v4_moves_no_bytes` (assert files untouched) (Req 32.5),
      `migration_v3_to_v4_preserves_storage_path_column` (dual-read) (Req 32.5).

- [ ] 14. Resolution via DatasetVolume indirection + Volume online check, with
      dual-read fallback (dataset-catalog task 34; Req 32.3, 32.4).
      In `catalog.rs` / `catalog_registry.rs`: add a resolution path that, for a
      resolved dataset, reads its `dataset_volumes` rows (ordered by
      sequence_number) -> `volumes` row -> `locator`, verifying each Volume
      `status = 'Online'` (map Offline to a reported error identifying the first
      unavailable Volume, reusing or adding a `CatalogError` variant, e.g.
      `VolumeUnavailable { volser }`). If a dataset has NO `dataset_volumes` row
      (unmigrated), FALL BACK to `storage_path` (dual-read). Keep the existing
      `ResolveResult.physical_path` populated from the resolved locator joined to
      the Volume `storage_uri` (or the legacy repo root for the fallback). Do NOT
      remove the `storage_path` reads elsewhere.
      Files: `crates/ff-dscatalog/src/catalog.rs`,
      `crates/ff-dscatalog/src/catalog_registry.rs`,
      `crates/ff-dscatalog/src/error.rs` (new variant if needed).
      Verify: `cargo test -p ff-dscatalog resolve` passes.
      Tests: `resolve_via_datasetvolume_locator` (Req 32.3),
      `resolve_offline_volume_reports_first_unavailable` (Req 32.3),
      `resolve_falls_back_to_storage_path_when_unmigrated` (dual-read, Req 32.3),
      `catalog_persists_metadata_and_locator_only` (Req 32.4).

- [ ] 15. Multivolume, shared-volume, and uncataloged (VOL=SER + UNIT)
      resolution at the catalog layer (dataset-catalog task 36; Req 32.6, 32.7).
      Ensure the schema permits (no constraint forbids) many catalogs registering
      datasets on one shared volume and one dataset spanning multiple volumes
      (multiple `dataset_volumes` rows with ascending sequence_number). Add a
      catalog-layer `resolve_uncataloged(volser, unit)` that delegates to
      `ff_volume::dataset_volume::resolve_uncataloged` against the volume registry
      without requiring a `datasets` row.
      Files: `crates/ff-dscatalog/src/catalog_registry.rs` (or a small
      `volume_binding.rs` module if catalog_registry.rs nears 400 lines),
      `lib.rs`.
      Verify: `cargo test -p ff-dscatalog uncataloged` and `... multivolume` pass.
      Tests: `two_catalogs_share_one_volume` (Req 32.6),
      `one_dataset_spans_two_volumes` (Req 32.6),
      `uncataloged_resolves_by_volser_unit_no_catalog_row` (Req 32.7).

- [ ] 16. Consolidated migration/resolution integration tests
      (dataset-catalog task 37). Place cross-cutting tests in
      `crates/ff-dscatalog/tests/schema_v4.rs` (integration test file) covering
      the four task-37 scenarios end-to-end: dual-read migration, volume
      resolution, uncataloged resolve, shared-volume registration. If any inline
      test module exceeds ~200 lines, move it to a sibling `*_tests.rs`.
      Files: `crates/ff-dscatalog/tests/schema_v4.rs`.
      Verify: `cargo test -p ff-dscatalog --test schema_v4` passes.
      Tests: mirror dataset-catalog tasks 37.1-37.4 with
      `// Validates: Requirement 32.1-32.7`.

### Doc updates (end of the run, after scoped checks are green)

- [ ] 17. Update tracking docs. Do NOT pre-mark tasks before the code is in.
      - `docs/specs/volume-model/tasks.md`: tick tasks 1-10 `[x]` (NOT task 11 --
        RC.B.6). 
      - `docs/specs/dataset-catalog/tasks.md`: tick tasks 33-37 `[x]`.
      - `docs/quality/TCR.md`: in the `ff-dscatalog` section add rows for Req
        32.1-32.8 with the test names from items 12-16 (status PASS where a test
        exists). Add an `ff-volume` crate section (follow the existing per-crate
        table format) with rows for volume-model Req 1-11 criteria mapped to the
        ff-volume test names from items 2-10 (PASS). Use the TCR emoji set.
      - `docs/project-management/project-master/tasks.md`: tick / add the RC.B.5
        deliverable line(s) for "ff-volume crate + schema v4" and update the
        Summary counts. (Edit the main-workspace copy only; ignore the
        `.worktrees/*` copies.)
      Files: the four docs above.
      Verify: ASCII check -- `rg "[\u2013\u2014\u2018\u2019\u201C\u201D]" docs/`
      finds no new matches in the edited files; re-run `cargo fmt`.

### Hand-off

- [ ] 18. Final scoped verification + owner hand-off. Run, via the pwsh7 wrapper:
      `cargo check -p ff-volume`, `cargo test -p ff-volume`,
      `cargo clippy -p ff-volume -- -D warnings`, then the same `-p ff-dscatalog`,
      then `cargo fmt -- --check`. Confirm all clean. Then STOP and hand off:
      state exactly which scoped commands were run and prompt the owner to run the
      full gate manually (`cargo gate --build`). Leave the tree UNCOMMITTED. Do
      NOT run `--workspace` or `cargo gate` yourself.

## Out-of-scope crates that may fail the owner's FULL gate (flag these)

The scoped `-p ff-volume` / `-p ff-dscatalog` checks will pass, but adding
`ff-volume` as a dependency and changing resolution may surface failures in
downstream crates during the owner's `--workspace` gate. The coder MUST note
these in `findings` so the owner is not surprised:

1. **Direct `storage_path` readers downstream of the catalog.** `ff-dsalloc`
   (`catalog_bridge`), `ff-idcams`, `ff-catalog-registry`, `ff-files-panel`,
   `ff-explorer-view`, and `ff-desktop` resolve datasets via the catalog and may
   read `ResolveResult.physical_path`. Dual-read keeps these working (storage_path
   retained), so they SHOULD still pass -- but any that constructed a path from
   `storage_path` directly need review. Do NOT change them in RC.B.5 (that is
   RC.B.6/B.8); just list them.
2. **`DatasetRecord` / `AllocParams` constructors.** Any crate that constructs a
   `DatasetRecord` literal (it still carries `storage_path`) is unaffected because
   the column is retained; confirm none assumes schema v3 exactly.
3. **Governance / fitness tests.** `ff-governance-tests` enforces the acyclic DAG
   (dataset-ownership-model Req 20). Adding `ff-volume` as a new dataset-subsystem
   crate may require its dependency-direction allow-list to learn
   `ff-dscatalog -> ff-volume -> ff-vfs` (Req 20.3 says the fitness function must
   be updated when a new subsystem crate is added). FLAG that this allow-list / 
   fitness function likely needs a new rule for `ff-volume`; if a scoped
   `cargo test -p ff-governance-tests` is quick, run it and report, else note it
   for the owner's full gate.
4. **Schema-version assertions.** Any test anywhere asserting
   `SCHEMA_VERSION == "3"` (there is at least one in `ff-dscatalog`'s schema.rs
   test module) breaks on the bump to "4"; the ff-dscatalog ones are fixed in
   item 12, but grep the workspace note for others the full gate may catch.

## Requirement -> item map (traceability)

| Spec / Req | Plan item |
|---|---|
| volume-model Req 1 | 2 |
| volume-model Req 2 | 3 |
| volume-model Req 3, 11.1 | 4 |
| volume-model Req 4 | 5 |
| volume-model Req 5, 6 | 6 |
| volume-model Req 7 | 7 |
| volume-model Req 8, 11.2 | 8 |
| volume-model Req 9 | 9 |
| volume-model Req 10, 11.3 | 10 |
| dataset-catalog Req 32.8 | 11 |
| dataset-catalog Req 32.1, 32.2 | 12 |
| dataset-catalog Req 32.5 | 13 |
| dataset-catalog Req 32.3, 32.4 | 14 |
| dataset-catalog Req 32.6, 32.7 | 15 |
| dataset-catalog task 37 (32.1-32.7) | 16 |
| doc/TCR/master updates | 17 |
| hand-off | 18 |
