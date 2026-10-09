# CR-CH-059 Phase RC.A -- Verification Note (first iteration)

Code-complete PENDING the owner's full `cargo gate --build`. All edits are
UNCOMMITTED on branch `main` for owner review. No worktree / branch / commit was
created. `.worktrees/` was not touched.

## What was implemented (per rca-plan.md, in dependency order)

### RC.A.1 -- ADR-001 correction + crate-name doc/comment fixes (docs/comment only)
- `docs/specs/dataset-ownership-model/requirements.md`: added CR-CH-059 pointers
  to the `ff-dataset-catalog` / `ff-vsam-services` glossary entries, the Req 3 and
  Req 5 ownership-boundary headers, and Req 18 AC 2 (fitness-function wording now
  asserts the corrected `ff-dscatalog` authority + the eventual no-live-reference
  rule, WITHOUT changing `architecture_compliance.rs`). Requirement 22 (22.1-22.7)
  and the design.md delta were already present from the gate.
- `docs/specs/jes-emulator/requirements.md`: VERIFIED (grep) -- every legacy
  `ff-dataset-catalog` / `ff-dataset-allocator` mention already sits inside a
  "corrected ... by CR-CH-059" note; no live legacy reference. No edit needed.
- `crates/ff-dsalloc/src/catalog_bridge.rs`: doc comment `ff-dataset-catalog` ->
  `ff-dscatalog` (two mentions); also fixed three pre-existing non-ASCII chars
  (em dashes / arrow) in that touched file. Comment-only; no logic change.

### RC.A.2 -- Reconciled CatalogService / VsamService in ff-dscatalog
- New `crates/ff-dscatalog/src/service.rs` (360 non-test lines): reconciled
  `CatalogService` + object-safe `DynCatalogService` (blanket impl) over
  ff-dscatalog's own `Dsn`/`Dsorg`/`Recfm`; reconciled DTOs; reuses `CatalogError`.
- New `crates/ff-dscatalog/src/vsam_service.rs` (350 non-test lines): object-safe
  `VsamService` + `VsamCluster` (VSAM is a cluster entity carrying `VsamType`, NOT
  a `Dsorg` variant), VSAM DTOs, `StubVsamService`, `VsamError`.
- `crates/ff-dscatalog/src/lib.rs`: declared + re-exported both modules.
- Repointed `crates/ff-governance-tests` (Cargo.toml dev-dep + mock_compilation.rs)
  from `ff_dataset_catalog` / `ff_vsam_services` to the reconciled `ff_dscatalog`
  traits/types (per dscatalog-duplicate/report.md step 3). `architecture_compliance.rs`
  NOT edited (crates still exist and are still legitimately named). ASCII-cleaned
  mock_compilation.rs (pre-existing em dashes / box-drawing / arrows).

### RC.A.3 -- Single ff-vfs::StorageProvider seam; duplicate trait deleted
- The five backends in `crates/ff-dscatalog/src/storage/{native,esds,sqlite_record,
  rrds,isam}.rs` now implement `ff_vfs::StorageProvider` (opaque `StorageLocator`,
  `HashSet<StorageCapability>`, `VfsError`; UUID/workspace_root carried behind a
  `root` field on each struct; `ProviderCapability` -> `StorageCapability` 1:1).
- Deleted the duplicate trait `ff-dscatalog::storage::StorageProvider` plus its
  `ProviderCapability` / `ObjectId` / `ObjectStat` support types from
  `storage/mod.rs` (grep confirmed they were used ONLY by the five backends).
- native.rs test module split into `storage/native_tests.rs` (via `#[path]`) to
  keep native.rs under 400 non-test lines (now 388). Record codecs untouched.
- `crates/ff-desktop/src/shell/construct_provider.rs`: VERIFIED unchanged (it only
  registers the host `local` provider and never named the deleted trait).

### RC.A.4 -- Single posix registrant
- `crates/ff-vfs/src/posix_provider.rs`: added a single-registrant test
  (`posix_native_provider_is_sole_posix_registrant`, Req 14.1/14.4).
- `crates/ff-posix-provider/src/lib.rs`: reduced to pure POSIX path helpers +
  `pub use ff_vfs::PosixNativeProvider as PosixProvider;`. Its own local-FS-backed
  `VfsProvider` impl was removed, so it can no longer register scheme `posix`.
  Cargo.toml deps trimmed accordingly (added `tempfile` dev-dep).
- `crates/ff-desktop/src/shell/render_nav.rs` + `render_nav_expand.rs`: switched
  the four ad-hoc `PosixProvider::new(root, ro)` call sites to the single
  infallible `ff_vfs::PosixNativeProvider` (via the re-export), adjusting the
  surrounding `Result` handling. `local` provider untouched.

## Scope guards honoured
- ADDITIVE-FIRST: the ONLY deletion was the duplicate TRAIT (RC.A.3). The crates
  `ff-dataset-catalog` and `ff-vsam-services` and their root `Cargo.toml` member
  lines (57, 58) STILL EXIST (verified).
- No JES, no Volume UI, no storage_path->DatasetVolume migration, no DatasetAccess,
  no MAINFRAME BackendEnvironment added. `.worktrees/vsam-wiring` untouched. No VSAM
  wired under the old `ff-vsam-services` trait -- only the reconciled surface added.

## SCOPED commands run (clean pwsh7 wrapper; NEVER --workspace / cargo gate)
- `cargo fmt` -> exit 0.
- `cargo check -p ff-dscatalog -p ff-vfs -p ff-posix-provider -p ff-governance-tests -p ff-desktop --all-targets` -> exit 0.
- `cargo clippy -p ff-dscatalog -p ff-vfs -- -D warnings` -> exit 0 (clean).
- `cargo test -p ff-dscatalog` -> 262 passed, 0 failed (incl. the 3 new storage
  seam tests + service/vsam_service tests).
- `cargo test -p ff-vfs -p ff-posix-provider` -> ff-vfs 163 + PBTs all pass (incl.
  `posix_native_provider_is_sole_posix_registrant`); ff-posix-provider 11 pass.
- `cargo test -p ff-governance-tests` -> architecture_compliance 10 pass +
  mock_compilation 7 pass (reconciled traits).
- `cargo test -p ff-desktop` -> 998 passed, 4 FAILED. The 4 failures
  (`close_workspace_removes_settings_from_config`,
  `set_theme_disables_follow_os_so_selection_is_not_clobbered`,
  `exit_saves_command_history_and_reloads`,
  `startup_loads_persisted_command_history`) are the PRE-EXISTING B048
  shared-env-var flake under multithreaded `cargo test` (unrelated to RC.A: they
  concern config/history/theme persistence racing on shared state). Confirmed by
  re-running the affected modules single-threaded
  (`cargo test -p ff-desktop --bin ffwb -- --test-threads=1 tests_session tests_menu_workspace`)
  -> 128 passed, 0 failed. Not fixed, per the brief.

## Line-limit note (rust-standards 400 non-test lines)
- New/changed files that are compliant: service.rs 360, vsam_service.rs 350,
  native.rs 388, rrds.rs 296, isam.rs 152, storage/mod.rs small, ff-posix-provider
  lib.rs ~190.
- `storage/esds.rs` (611 non-test) and `storage/sqlite_record.rs` (845 non-test)
  were ALREADY over the 400-non-test limit before RC.A (esds was 652 total at HEAD).
  RC.A added only a `root` field + the behaviour-preserving trait reshape (a handful
  of lines); it did not create the violation. Splitting these record backends is a
  larger refactor outside RC.A's additive scope (the plan scoped file-splitting to
  native.rs only). Flagged here as a pre-existing condition and a candidate for a
  follow-up refactor; NOT addressed in RC.A.

## ASCII
- All new/changed `.rs` files are plain ASCII (grep `[^\x00-\x7F]` returns nothing
  for service.rs, vsam_service.rs, the storage/*.rs backends, native_tests.rs,
  ff-posix-provider/lib.rs, and mock_compilation.rs after the ASCII clean-up).

## Hand-off
Kiro ran ONLY the scoped checks above and they are clean (the ff-desktop failures
are the documented pre-existing B048 flake). The owner should run the full gate
manually outside Kiro: `cargo gate --build` (fallback `pwsh -ExecutionPolicy
Bypass -File tools\ffwb-gate.ps1`) and report back.
