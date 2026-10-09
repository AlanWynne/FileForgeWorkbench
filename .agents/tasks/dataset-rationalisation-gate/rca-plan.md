# CR-CH-059 Phase RC.A -- Implementation Plan (Mainframe Dataset Stack Rationalisation)

Owner-approved, gated work. This plan is to be followed verbatim by the coder.
Workspace root: `c:\workspace\VSC\FileForgeWorkbench`. All paths below are
absolute under that root unless shown relative. ASCII-only in every `.rs` and
`docs/` file (documentation.md). Every new/changed `.rs` stays under 400
non-test lines; split a test module before it passes ~200 lines
(rust-standards.md). TDD per testing.md: write the failing test FIRST (red),
confirm it fails, then implement to green. Every test carries a
`// Validates: Requirement X.Y` line.

## Scope and dependency order (do NOT reorder)

- RC.A.1 (docs/comments only) -> RC.A.2 (reconciled traits + single enums +
  repoint governance tests) -> RC.A.3 (single physical seam, delete the
  DUPLICATE TRAIT) in that order; A.3 depends on A.2 being in place.
- RC.A.4 (single posix registrant) is independent; do it LAST.
- ADDITIVE-FIRST. In RC.A the ONLY deletion permitted is the duplicate TRAIT
  `ff-dscatalog::storage::StorageProvider` (RC.A.3). Do NOT delete the crates
  `ff-dataset-catalog` or `ff-vsam-services`, nor their root `Cargo.toml` member
  lines (that is RC.B.7 / RC.B.8, OUT OF SCOPE). FFWB must build at every step.
- Respect the "DO NOT build before consolidation" waste list: do NOT touch JES,
  Volume UI, `storage_path` -> DatasetVolume migration, `DatasetAccess`, or a
  MAINFRAME BackendEnvironment here. Do NOT touch `.worktrees/vsam-wiring`. Do
  NOT wire VSAM under the old `ff-vsam-services` trait -- RC.A only ADDS the
  reconciled `VsamService` trait surface in `ff-dscatalog`.

## Key findings from exploration (ground truth the plan relies on)

- `ff-dscatalog` already depends on `ff-vfs` (Cargo.toml) -- RC.A.3 needs NO new
  dependency. `ff-dscatalog` Dsorg = `{PS,PO,GDG}` and Recfm = `{F,FB,V,VB,U}`
  (both already `#[non_exhaustive]`, correct casing) in
  `crates/ff-dscatalog/src/dataset.rs`. These are the single enums; do NOT add
  `Vsam`/`Da` variants.
- The duplicate trait `ff-dscatalog::storage::StorageProvider`
  (`crates/ff-dscatalog/src/storage/mod.rs`) uses `ObjectId = uuid::Uuid`,
  `&[ProviderCapability]`, and threads `workspace_root: &Path`. The single
  `ff-vfs::StorageProvider` (`crates/ff-vfs/src/storage_provider.rs`) uses an
  opaque `StorageLocator`, `HashSet<StorageCapability>`, is UUID-free, and maps
  errors to `VfsError`. `PosixNativeProvider` already implements it.
- The duplicate trait is implemented ONLY by the five backends in
  `crates/ff-dscatalog/src/storage/{native,esds,sqlite_record,rrds,isam}.rs`
  and is referenced via `dyn` only inside `storage/isam.rs` (a test and an
  internal `StorageProvider::delete(self.inner...)` call). NO other `ff-dscatalog`
  module (catalog.rs, vfs_provider.rs, repository.rs, integrity.rs, commands.rs)
  names the backends or the trait. The record-level operations the catalog uses
  are INHERENT methods on the concrete backend structs, not trait methods, so
  deleting the duplicate trait does not sever catalog logic. VERIFY this during
  implementation with a grep before deleting.
- `ff-governance-tests` is the ONLY consumer of `ff-dataset-catalog` /
  `ff-vsam-services`. `tests/mock_compilation.rs` imports BOTH
  `ff_dataset_catalog::{CatalogService, DynCatalogService, ...}` AND
  `ff_vsam_services::{VsamService, ...}`. `tests/architecture_compliance.rs`
  references those crate NAMES by string and asserts they EXIST
  (`all_governed_crates_exist`, `prohibited_rules_cover_all_dataset_crates`).
  Because RC.A does NOT delete the crates, `architecture_compliance.rs` stays
  valid and MUST NOT be edited in RC.A beyond what RC.A.1 Task 13.4 documents.
- The live provider registry (`crates/ff-desktop/src/shell/construct_provider.rs`)
  registers ONLY the `local` provider. Neither posix provider self-registers
  into the live `ProviderRegistry`. `ff-posix-provider::PosixProvider` is
  instantiated ad-hoc in `crates/ff-desktop/src/shell/render_nav.rs` and
  `render_nav_expand.rs`. `ff-vfs::PosixNativeProvider` implements BOTH
  `VfsProvider` (scheme "posix") and `ff-vfs::StorageProvider`.
- `ff-dsalloc` does NOT depend on `ff-dscatalog`; its `catalog_bridge.rs`
  `CatalogProvider` is a local trait with a doc comment naming the wrong crate.
  RC.A.1's fix there is comment-only.

---

## RC.A.1 -- ADR-001 correction + crate-name doc/comment fixes

DOCS + comments only. No gate, no tests, no Rust logic change. ASCII-only.
Delivers dataset-ownership-model Req 22 (tasks 13.1-13.5); jes-emulator Req 19
(tasks 35.1-35.4). Build is unaffected (comment-only in `.rs`).

- [ ] 1. Amend ADR-001 in dataset-ownership-model to name `ff-dscatalog` as the
      catalog authority and mark the two trait crates deprecated-for-merge.
      Edit `docs/specs/dataset-ownership-model/requirements.md`: in the
      Introduction glossary and Requirement 3/5 ownership text, add the
      "read-as `ff-dscatalog`" / "DEPRECATED-FOR-MERGE" governance notes
      consistent with the already-present Requirement 22 (22.1-22.7). Do NOT
      rewrite Requirements 1-21 wholesale -- Req 22's note already remaps the
      legacy names; add a short pointer where a reader would otherwise be
      misled (glossary entries for `ff-dataset-catalog` and `ff-vsam-services`,
      and the Req 3/Req 5 headers). Update the architectural fitness-function
      text (Requirement 18 AC 2) so it reads as asserting the corrected
      `ff-dscatalog` authority and the eventual no-live-reference rule, WITHOUT
      changing `architecture_compliance.rs` (that test change is RC.B when the
      crates are actually deleted; here it is documentation of intent per task
      13.4). Keep the acyclic DAG (`ff-idcams -> ff-dsalloc -> ff-dscatalog ->
      ff-volume -> storage providers`) and ADR-002 (catalogs never own bytes)
      intact. Mark the touched spec section notes with "(... by CR-CH-059)".
      Files: `docs/specs/dataset-ownership-model/requirements.md`,
      `docs/specs/dataset-ownership-model/design.md` (add the design delta that
      task 13.1-13.3 reference).
      Verify: `rg "[^\x00-\x7F\u2500-\u257F]" docs/specs/dataset-ownership-model/requirements.md docs/specs/dataset-ownership-model/design.md`
      returns no matches (ASCII check); re-read the file and confirm Req 22 and
      the new notes agree (no contradiction).

- [ ] 2. Fix the wrong crate names in `docs/specs/jes-emulator/requirements.md`
      per the gate summary's in-place edits: `ff-dataset-catalog` ->
      `ff-dscatalog`, `ff-dataset-allocator` -> `ff-dsalloc` in the
      Introduction, Req 1.7, Req 11.1/11.3/11.5, Req 12.5, each carrying a
      "(Crate names corrected ... by CR-CH-059)" note. NOTE: exploration shows
      the Introduction, Req 19.1-19.4, and the Req 11/12 lines ALREADY carry the
      corrected names and the CR-CH-059 note. Treat this as VERIFICATION: grep
      the file and confirm NO live legacy reference remains outside an explicit
      "corrected from" note; add the note/correction anywhere a bare legacy name
      still appears. Do the same read-only confirmation on
      `docs/specs/jes-emulator/design.md`.
      Files: `docs/specs/jes-emulator/requirements.md`,
      `docs/specs/jes-emulator/design.md` (only if a bare legacy name is found).
      Verify: `rg "ff-dataset-catalog|ff-dataset-allocator" docs/specs/jes-emulator/`
      -- every hit must be inside a "corrected from ... by CR-CH-059" note, none
      a live crate reference. ASCII check as in step 1.

- [ ] 3. Fix the wrong crate name in the `ff-dsalloc` catalog-bridge doc
      comment. In `crates/ff-dsalloc/src/catalog_bridge.rs` change the two
      doc-comment mentions of "Production would delegate to `ff-dataset-catalog`"
      / "Production implementation delegates to `ff-dataset-catalog`" to
      `ff-dscatalog`. Comment-only; do NOT add a dependency or change any logic
      (`ff-dsalloc` deliberately has no `ff-dscatalog` dep in RC.A).
      Files: `crates/ff-dsalloc/src/catalog_bridge.rs`.
      Verify: `cargo check -p ff-dsalloc` compiles (comment-only, must still
      build); `rg "ff-dataset-catalog" crates/ff-dsalloc/` returns no matches;
      `rg "[^\x00-\x7F]" crates/ff-dsalloc/src/catalog_bridge.rs` returns no
      matches (ASCII).

- [ ] 4. Flip the RC.A.1 docs tasks and TCR rows. In the per-spec tasks.md mark
      the RC.A.1 checkboxes complete, and set the ownership TCR rows to their
      documented status. Mark `[x]`: dataset-ownership-model tasks 13.1, 13.2,
      13.3, 13.4, 13.5; jes-emulator tasks 35.1, 35.2, 35.3, 35.4. In
      `docs/quality/TCR.md` update the Req 22.1-22.7 rows (ownership,
      documentation criteria) and the jes Req 19.1-19.5 rows (doc/contract
      criteria -- JES deferred, no runtime test) to the status those task notes
      specify (documentation-complete, not runtime-tested).
      Files: `docs/specs/dataset-ownership-model/tasks.md`,
      `docs/specs/jes-emulator/tasks.md`, `docs/quality/TCR.md`.
      Verify: `rg "\[ \] 13\.|\[ \] 35\." docs/specs/dataset-ownership-model/tasks.md docs/specs/jes-emulator/tasks.md`
      shows those lines are now `[x]`; ASCII check on TCR.md
      (`rg "[^\x00-\x7F\u2500-\u257F]" docs/quality/TCR.md` -- allow box-drawing
      and the TCR status emoji only).

---

## RC.A.2 -- Reconciled CatalogService / VsamService in ff-dscatalog + single Dsorg/Recfm + repoint ff-governance-tests

THE CORE CODE STEP. Delivers dataset-catalog Req 33 (33.1-33.7) and supports
dataset-ownership-model Req 22.2-22.5. Additive: traits are ADDED to
`ff-dscatalog` over its OWN types; nothing is removed. Design decision: model
VSAM as a cluster entity (`VsamCluster` with a KSDS/ESDS/RRDS/LDS type field) --
do NOT add `Vsam`/`Da` to `Dsorg`. Reason: Req 33.5 makes the removal of those
variants deliberate, and the backends already distinguish VSAM kind by concrete
type, not by Dsorg.

Design decision -- object safety: follow the existing dataset-ownership-model
Req 15.7 pattern (an ergonomic `CatalogService` plus an object-safe
`DynCatalogService` wrapper with a blanket `impl<T: CatalogService>
DynCatalogService for T`). This mirrors the retired `ff-dataset-catalog`
pattern the governance tests expect. `VsamService` is written object-safe
directly (Req 33.3), matching the retired `ff-vsam-services::VsamService` shape
so the mock repoint is mechanical.

Design decision -- file layout (400-line rule): put the new traits in NEW files
under `ff-dscatalog`, not in the already-large `catalog.rs`. Create
`crates/ff-dscatalog/src/service.rs` (the `CatalogService` + `DynCatalogService`
traits, their DTOs that are not already in `dataset.rs`/`dsn.rs`/`gdg.rs`, and
the blanket impl) and `crates/ff-dscatalog/src/vsam_service.rs` (the
`VsamService` trait, `VsamCluster`, VSAM type enum, and VSAM DTOs). Re-export
both from `crates/ff-dscatalog/src/lib.rs`. Keep each file under 400 non-test
lines; if the test module in either file approaches ~200 lines, split it into a
sibling `*_tests.rs`.

- [ ] 5. (RED) Write the failing trait-shape/behaviour tests for the reconciled
      `CatalogService` + `DynCatalogService` in `ff-dscatalog`. In a `#[cfg(test)]`
      module in `crates/ff-dscatalog/src/service.rs`, add tests that (a) a
      `MockCatalogService` implementing `CatalogService` compiles and can be
      boxed as `Box<dyn DynCatalogService>` (object-safety), (b) the trait
      exposes the Req 15 operation set over `ff-dscatalog`'s own types
      (`create_dataset`/`delete_dataset`/`update_dataset`/`rename_dataset`,
      `resolve_dsn`/`dataset_exists`/`get_dataset_attributes`,
      `list_datasets`/`validate_dsn`, GDG ops, allocation defaults keyed by
      `Dsorg`), (c) `get_allocation_defaults(Dsorg::PS)` round-trips the
      `ff-dscatalog` `Dsorg` enum (proves single-enum usage, not the divergent
      set). Give each test `// Validates: Requirement 33.1` / `33.2` / `33.4` as
      appropriate.
      Files: `crates/ff-dscatalog/src/service.rs` (new).
      Verify: `cargo test -p ff-dscatalog service` fails to COMPILE/pass because
      the trait does not exist yet (red). Confirm the failure is the expected
      "cannot find trait" / unimplemented, not an unrelated error.

- [ ] 6. (GREEN) Implement the reconciled `CatalogService` + `DynCatalogService`
      in `crates/ff-dscatalog/src/service.rs` over `ff-dscatalog`'s `Dsn`,
      `Dsorg`, `Recfm`, and existing DTOs (`DatasetRecord`, `AllocParams`,
      `DatasetProperties`, GDG types), reusing `CatalogError` from
      `crates/ff-dscatalog/src/error.rs` for the error type. Methods per Req
      33.2. Add the object-safe `DynCatalogService` wrapper + blanket impl
      (Req 33.1, 22.5). Re-export both from `lib.rs`. Where behaviour-preserving,
      make the concrete `Catalog` (`crates/ff-dscatalog/src/catalog.rs`) and/or
      `CatalogRegistry` (`catalog_registry.rs`) implement `CatalogService` by
      delegating to their existing inherent methods -- add the `impl` block in
      `service.rs` or a small `impl` at the bottom of `catalog.rs` ONLY if it
      stays under the 400-line limit; otherwise keep the impl in `service.rs`.
      Do NOT change existing public method behaviour; the trait is a thin facade.
      Files: `crates/ff-dscatalog/src/service.rs`,
      `crates/ff-dscatalog/src/lib.rs` (re-export),
      `crates/ff-dscatalog/src/catalog.rs` (optional thin `impl` only).
      Verify: `cargo test -p ff-dscatalog service` passes (green);
      `cargo check -p ff-dscatalog`.

- [ ] 7. (RED) Write the failing tests for the reconciled `VsamService` in
      `crates/ff-dscatalog/src/vsam_service.rs`: a `MockVsamService` compiles and
      boxes as `Box<dyn VsamService>` (object-safe, Req 33.3); the trait covers
      the Req 16 operation set (create per VSAM type, destroy/initialize, record
      get/put/delete, browse, alternate indexes); a `VsamCluster` value carries a
      VSAM type (KSDS/ESDS/RRDS/LDS) and is NOT a `Dsorg` variant (Req 33.5 --
      assert `Dsorg` still has exactly `PS`,`PO`,`GDG` by matching all variants).
      Give each test `// Validates: Requirement 33.3` / `33.5`.
      Files: `crates/ff-dscatalog/src/vsam_service.rs` (new).
      Verify: `cargo test -p ff-dscatalog vsam_service` is red (trait absent).

- [ ] 8. (GREEN) Implement `VsamService` (object-safe), `VsamCluster`, and the
      VSAM type enum in `crates/ff-dscatalog/src/vsam_service.rs`, backed by the
      existing KSDS (`SqliteRecordProvider`), ESDS (`NativeEsdsProvider`), and
      RRDS (`SqliteRrdsProvider`) backends (Req 33.3). The trait is a SURFACE in
      RC.A -- a mock implements it for the governance test; do NOT wire a concrete
      `ff-dscatalog` impl against live backends here (that is RC.B.7). Reuse
      `CatalogError`/a VSAM error from `error.rs` for the error type. Re-export
      from `lib.rs`.
      Files: `crates/ff-dscatalog/src/vsam_service.rs`,
      `crates/ff-dscatalog/src/lib.rs`.
      Verify: `cargo test -p ff-dscatalog vsam_service` passes;
      `cargo check -p ff-dscatalog`.

- [ ] 9. Repoint `ff-governance-tests` from `ff-dataset-catalog`/
      `ff-vsam-services` to the reconciled `ff-dscatalog` traits. Edit
      `crates/ff-governance-tests/Cargo.toml` `[dev-dependencies]`: replace
      `ff-dataset-catalog` and `ff-vsam-services` with
      `ff-dscatalog = { path = "../ff-dscatalog" }`. Edit
      `crates/ff-governance-tests/tests/mock_compilation.rs`: change the two
      `use ff_dataset_catalog::{...}` / `use ff_vsam_services::{...}` import
      blocks to `use ff_dscatalog::{...}` (the reconciled `CatalogService`,
      `DynCatalogService`, `VsamService`, `VsamCluster`, and the reconciled DTO
      types), and adjust `MockCatalogService` / `MockVsamService` to the
      reconciled method signatures and types (e.g. `Dsorg::Ps` -> `Dsorg::PS`;
      the retired-crate DTOs -> the `ff-dscatalog` equivalents). Follow the exact
      procedure in `.agents/tasks/dscatalog-duplicate/report.md` step 3. DO NOT
      delete `ff-dataset-catalog` / `ff-vsam-services` or their member lines
      (RC.B.8). After the repoint those crates simply have no consumer -- fine.
      DO NOT edit `tests/architecture_compliance.rs` (it still legitimately
      references the crate names by string and asserts they exist, which remains
      true in RC.A).
      Files: `crates/ff-governance-tests/Cargo.toml`,
      `crates/ff-governance-tests/tests/mock_compilation.rs`.
      Verify: `cargo test -p ff-governance-tests` passes (both
      `mock_compilation` and `architecture_compliance` suites); confirm the
      `allocator_compiles_with_mock_catalog_service`,
      `idcams_compiles_with_mock_vsam_and_catalog_services`,
      `dyn_catalog_service_can_be_boxed`, and `vsam_service_can_be_boxed` tests
      compile against the reconciled traits (Req 33.6, 33.7).

- [ ] 10. ASCII + line-length checkpoint and task/TCR flips for RC.A.2.
      Verify every new/changed `.rs` is ASCII and under 400 non-test lines; mark
      the tasks and TCR rows. Mark `[x]`: dataset-catalog tasks 38, 38.1, 38.2,
      38.3 (and any 38.x sub-items present). In `docs/quality/TCR.md` set the
      Req 33.1-33.7 rows (crate `ff-dscatalog`) to PASS with the test names from
      steps 5-9; set dataset-ownership-model Req 22.2-22.5 rows that these tests
      back to their covered status.
      Files: `docs/specs/dataset-catalog/tasks.md`, `docs/quality/TCR.md`.
      Verify: `rg "[^\x00-\x7F]" crates/ff-dscatalog/src/service.rs crates/ff-dscatalog/src/vsam_service.rs crates/ff-governance-tests/tests/mock_compilation.rs`
      returns nothing; count non-test lines of each new file is < 400;
      `cargo fmt -- --check` clean for the touched crates.

---

## RC.A.3 -- Unify onto ff-vfs::StorageProvider + delete the DUPLICATE TRAIT

Depends on RC.A.2. Delivers virtual-file-system Req 13 (13.1-13.5) and
dataset-catalog Req 35.1b (dataset-catalog task 39). Contained to
`crates/ff-dscatalog` + `crates/ff-vfs` + the live registry wiring in
`crates/ff-desktop/src/shell/construct_provider.rs`. The ONLY deletion in RC.A
is the duplicate TRAIT `ff-dscatalog::storage::StorageProvider` (NOT a crate).

Design decision -- capability mapping: map the old `ProviderCapability`
variants to `ff-vfs::StorageCapability` 1:1 where names match (StreamRead,
StreamWrite, RecordRead, RecordWrite, KeyedAccess, RelativeAccess, AppendOnly,
MemberOperations, AtomicRename); `ff-vfs` additionally has Locking,
Snapshotting, WatchNotifications (unused by these backends -- do not advertise
them). This is a lossless mapping.

Design decision -- locator and workspace_root: the old trait threaded
`workspace_root: &Path` and used `ObjectId = Uuid` + a locator string. The
`ff-vfs` trait is `workspace_root`-free and uses the opaque `StorageLocator`
(Req 13.3). Carry the UUID/workspace_root INSIDE the backend struct (store the
root at construction) and encode any UUID/relative-path inside the opaque
`StorageLocator` string. Backends are already constructed with their root
available (e.g. `IsamProvider::open(dir.path(), id, ...)`), so capture it in the
struct. Keep the record codecs in `ff-dscatalog` untouched (Req 13.4).

Design decision -- preserve inherent record methods: the catalog uses the
backends' INHERENT record methods (not trait methods). Keep those inherent
methods as-is; only the `impl StorageProvider` block changes trait + signatures.
This keeps the change behaviour-preserving for catalog logic.

- [ ] 11. (RED) Pre-check and write the failing seam test. First confirm the
      duplicate trait is only used by the backends: run
      `rg "ff_dscatalog::storage::StorageProvider|storage::StorageProvider|dyn StorageProvider" crates/`
      and `rg "ProviderCapability" crates/` and record the hit set (expected:
      only `crates/ff-dscatalog/src/storage/*`). Then add a failing test (in a
      `#[cfg(test)]` module in `crates/ff-dscatalog/src/storage/native.rs`, or a
      new `storage/seam_tests.rs` if native.rs is near the line limit) asserting
      that `NativeFileProvider` is usable as `Box<dyn ff_vfs::StorageProvider>`
      and advertises `ff_vfs::StorageCapability::StreamRead` via the HashSet API.
      `// Validates: Requirement 13.1, 13.2`.
      Files: `crates/ff-dscatalog/src/storage/native.rs` (test) or
      `crates/ff-dscatalog/src/storage/seam_tests.rs` (new).
      Verify: `cargo test -p ff-dscatalog storage` is red (the backend still
      implements the OLD trait, so `dyn ff_vfs::StorageProvider` does not apply).

- [ ] 12. (GREEN) Reimplement all five backends against `ff-vfs::StorageProvider`.
      In each of `crates/ff-dscatalog/src/storage/{native,esds,sqlite_record,
      rrds,isam}.rs`: change `impl StorageProvider for X` to
      `impl ff_vfs::StorageProvider for X`, converting the method set --
      `capabilities(&self) -> HashSet<StorageCapability>` (build from the mapped
      variants); `allocate(&self, name: &str) -> Result<StorageLocator, VfsError>`;
      `open(&self, &StorageLocator) -> Result<Vec<u8>, VfsError>`;
      `stat`/`rename`/`delete`/`list`/`reconcile`/`write` per the `ff-vfs`
      signatures -- carrying the former `workspace_root`/UUID behind the struct
      and the opaque `StorageLocator` (Req 13.3), and mapping all `CatalogError`
      outcomes to `VfsError` variants at the seam (Req 13.3). Keep the inherent
      record methods and codecs unchanged (Req 13.4). Update the `storage/isam.rs`
      internal `StorageProvider::delete(self.inner...)` call and its test to the
      `ff-vfs` trait. This is multiple files sharing ONE change pattern -- do them
      together.
      Files: `crates/ff-dscatalog/src/storage/native.rs`,
      `crates/ff-dscatalog/src/storage/esds.rs`,
      `crates/ff-dscatalog/src/storage/sqlite_record.rs`,
      `crates/ff-dscatalog/src/storage/rrds.rs`,
      `crates/ff-dscatalog/src/storage/isam.rs`.
      Verify: `cargo test -p ff-dscatalog storage` passes (step 11 test green);
      `cargo check -p ff-dscatalog`.

- [ ] 13. Delete the duplicate trait and its now-dead support types. In
      `crates/ff-dscatalog/src/storage/mod.rs` remove the `trait StorageProvider`
      definition, the `ProviderCapability` enum, `ObjectId`, and `ObjectStat`
      IF and only if nothing outside the backends still uses them (the step-11
      grep is the gate). Keep the `pub use` re-exports of the concrete backend
      types. If `ObjectStat`/`ObjectId` are still referenced by inherent methods,
      keep them; delete ONLY the trait + `ProviderCapability` that are now unused.
      Register the backends through the registry's physical-storage path exists
      already as `register_storage`/`get_storage` on `ff-vfs::ProviderRegistry`
      (Req 13.5) -- no second mechanism is added.
      Files: `crates/ff-dscatalog/src/storage/mod.rs`.
      Verify: `rg "trait StorageProvider" crates/ff-dscatalog/` returns no
      matches; `rg "ProviderCapability" crates/ff-dscatalog/` returns no matches;
      `cargo check -p ff-dscatalog` and `cargo check -p ff-vfs` build; `cargo
      clippy -p ff-dscatalog -- -D warnings` clean (no dead-code warnings).

- [ ] 14. Confirm the live registry still builds and registers correctly. Read
      `crates/ff-desktop/src/shell/construct_provider.rs`: it registers only the
      host `local` `VfsProvider` and does not reference the deleted trait, so no
      change is expected. If a signature mismatch surfaces at any registration
      site, adapt AT THE REGISTRATION SITE (e.g. wrap/convert the locator) -- do
      NOT revive the duplicate trait. VERIFY rather than assume: this is the
      "needs verification during implementation" item.
      Files: `crates/ff-desktop/src/shell/construct_provider.rs` (only if a
      mismatch is found).
      Verify: `cargo check -p ff-desktop` builds.

- [ ] 15. ASCII + line-length checkpoint and task/TCR flips for RC.A.3.
      Mark `[x]`: dataset-catalog tasks 39, 39.1, 39.2. In `docs/quality/TCR.md`
      set the virtual-file-system Req 13.1-13.5 rows (recorded under crate
      `ff-dscatalog`, per the vfs tasks.md note) to PASS with the step-11/12 test
      names.
      Files: `docs/specs/dataset-catalog/tasks.md`, `docs/quality/TCR.md`.
      Verify: `rg "[^\x00-\x7F]" crates/ff-dscatalog/src/storage/*.rs` returns
      nothing; each touched `storage/*.rs` is under 400 non-test lines (split a
      backend's test module into a `*_tests.rs` sibling if it crosses ~200
      lines); `cargo fmt -- --check` clean.

---

## RC.A.4 -- Collapse the duplicate posix registrant (independent; do last)

Delivers virtual-file-system Req 14 (14.1-14.4). Design decision (Req 14.2): the
single `posix` registrant is `ff-vfs::PosixNativeProvider` (it already
implements BOTH `VfsProvider` and `ff-vfs::StorageProvider`, demonstrating one
object spanning both seams). `ff-posix-provider::PosixProvider` is retired as a
`posix` registrant WITHOUT deleting the crate: reduce it so it can no longer
register scheme `posix` (either re-export `PosixNativeProvider` or stop its
`VfsProvider::scheme` from returning "posix"). Exploration shows the live
registry does not register either posix provider today and `PosixProvider` is
used ad-hoc in `ff-desktop` nav rendering, so the practical change is: switch
those ad-hoc usages to the single provider and guarantee only one object can
register `posix`.

- [ ] 16. (RED) Write the failing single-registrant test in `ff-vfs`. In a
      `#[cfg(test)]` test (e.g. `crates/ff-vfs/src/registry.rs` tests or
      `crates/ff-vfs/src/posix_provider.rs` tests) assert that after registering
      `PosixNativeProvider` for scheme `posix`, a second registration of ANY
      `posix` provider returns `VfsError::DuplicateScheme` (Req 14.4), and that
      the registry reports exactly one registrant per scheme for
      `local`/`posix`/`catalog`. `// Validates: Requirement 14.1, 14.4`.
      Files: `crates/ff-vfs/src/registry.rs` (test) or
      `crates/ff-vfs/src/posix_provider.rs` (test).
      Verify: `cargo test -p ff-vfs` -- the new test is red if it exercises a
      not-yet-present helper; otherwise confirm it encodes the single-registrant
      invariant and passes only after step 17.

- [ ] 17. (GREEN) Make `ff-vfs::PosixNativeProvider` the sole `posix` registrant.
      Reduce `crates/ff-posix-provider/src/lib.rs` so it no longer registers
      scheme `posix`: per Req 14.2, reduce it to a re-export of
      `ff-vfs::PosixNativeProvider` (preferred) OR remove its `VfsProvider` impl
      so it cannot self-register as `posix`. Update the ad-hoc `ff-desktop`
      usages in `crates/ff-desktop/src/shell/render_nav.rs` and
      `crates/ff-desktop/src/shell/render_nav_expand.rs` (and the thin
      re-export `crates/ff-desktop/src/posix_provider.rs`) to construct the
      single chosen provider. Leave the `local` provider
      (`ff-connector-local-fs::LocalFsProvider`) untouched (Req 14.3). Do NOT
      delete the `ff-posix-provider` crate or its member line.
      Files: `crates/ff-posix-provider/src/lib.rs`,
      `crates/ff-desktop/src/posix_provider.rs`,
      `crates/ff-desktop/src/shell/render_nav.rs`,
      `crates/ff-desktop/src/shell/render_nav_expand.rs`.
      Verify: `cargo test -p ff-vfs` green (step 16 test);
      `cargo check -p ff-posix-provider -p ff-vfs -p ff-desktop`;
      `cargo clippy -p ff-vfs -- -D warnings` clean.

- [ ] 18. ASCII + line-length checkpoint and task/TCR flips for RC.A.4.
      Mark `[x]`: virtual-file-system tasks 17, 17.1, 17.2, 17.3. In
      `docs/quality/TCR.md` set the virtual-file-system Req 14.1-14.4 rows to
      PASS with the step-16 test name.
      Files: `docs/specs/virtual-file-system/tasks.md`, `docs/quality/TCR.md`.
      Verify: `rg "[^\x00-\x7F]" crates/ff-posix-provider/src/lib.rs crates/ff-vfs/src/posix_provider.rs crates/ff-vfs/src/registry.rs`
      returns nothing; touched files under 400 non-test lines; `cargo fmt --
      --check` clean.

---

## Final verification and hand-off (whole RC.A)

- [ ] 19. Run the SCOPED checks for every crate RC.A touched and read their
      output before declaring code-complete. Do NOT run `--workspace` or
      `cargo gate` (owner's manual step).
      Verify, in order:
      `cargo fmt -- --check`;
      `cargo check -p ff-dscatalog -p ff-vfs -p ff-posix-provider -p ff-dsalloc -p ff-governance-tests -p ff-desktop`;
      `cargo clippy -p ff-dscatalog -p ff-vfs -p ff-posix-provider -- -D warnings`;
      `cargo test -p ff-dscatalog -p ff-vfs -p ff-governance-tests`.
      Expected: all clean/green. Fix any failure attributable to this change and
      rerun the scoped checks; if the same failure survives three materially
      different attempts, stop and report.

- [ ] 20. Confirm no out-of-scope deletion happened and the waste-list guards
      hold. Verify the crates `ff-dataset-catalog` and `ff-vsam-services` and
      their root `Cargo.toml` member lines STILL EXIST (RC.A deletes neither);
      verify `.worktrees/vsam-wiring` was not touched; verify no `DatasetAccess`,
      `storage_path` migration, Volume UI, or MAINFRAME BackendEnvironment code
      was added.
      Verify: `rg "crates/ff-dataset-catalog|crates/ff-vsam-services" Cargo.toml`
      shows both member lines present; `git status` shows no changes under
      `.worktrees/`.

- [ ] 21. Hand off. State that scoped checks are clean, list EXACTLY which
      scoped commands were run (step 19), and prompt the owner to run the full
      gate manually outside Kiro: `cargo gate --build` (fallback
      `pwsh -ExecutionPolicy Bypass -File tools\ffwb-gate.ps1`). Do NOT
      self-certify the full gate; wait for the owner's "clean" or failure report
      and act on it.

## Summary of artifacts to flip (consolidated)

- tasks.md checkboxes: dataset-ownership-model 13/13.1-13.5; jes-emulator
  35/35.1-35.4; dataset-catalog 38/38.1-38.3 and 39/39.1-39.2;
  virtual-file-system 17/17.1-17.3. (Do NOT flip RC.B/RC.C tasks: dataset-catalog
  40/41/42, jes contract-only beyond 35, project-master RC.B/RC.C rows.)
- TCR.md rows to flip to PASS/documented: ownership Req 22.1-22.7 (A.1 docs);
  jes Req 19.1-19.5 (A.1 doc/contract, JES deferred); dataset-catalog Req
  33.1-33.7 (A.2); virtual-file-system Req 13.1-13.5 (A.3, under crate
  `ff-dscatalog`) and Req 14.1-14.4 (A.4). Do NOT touch Req 34.x or 35.1a/c/d /
  35.2 / 35.3 rows (RC.B).
- project-master `docs/project-management/project-master/tasks.md`: mark
  RC.A.1-RC.A.4 `[ ]` -> `[x]` once each lands; leave RC.B/RC.C untouched.

## Notes / assumptions

- "Needs verification during implementation" items (do not skip, verify): the
  catalog-logic independence from the duplicate trait (step 11 grep before step
  13 deletion); the live registry needing no change (step 14); the jes-emulator
  crate-name corrections possibly already applied (step 2). None is assumed
  already-done -- each is kept in the plan as an explicit verify.
- Error-type choice for the reconciled traits reuses `ff-dscatalog`'s existing
  `CatalogError` to avoid a new public error taxonomy in RC.A; `DatasetError`
  and its `VfsError` mapping belong to `DatasetAccess` (RC.B), out of scope.
- If reducing `ff-posix-provider` to a re-export causes a downstream type
  mismatch in `ff-desktop` nav code, prefer switching those call sites to
  `ff_vfs::PosixNativeProvider` over keeping a second `posix`-capable type.
