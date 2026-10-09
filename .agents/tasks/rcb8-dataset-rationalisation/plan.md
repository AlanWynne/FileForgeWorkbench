# RC.B.8 Implementation Plan -- Mainframe Dataset Stack Rationalisation (CR-CH-059), final RC.B step

OWNER-APPROVED gated work. Edits go DIRECTLY on branch `main`, uncommitted, for
the owner's full `cargo gate --build`. NO worktree/branch/commit. TDD per
`.kiro/steering/testing.md`; ASCII-only `.rs`; 400-non-test-line rule (split
tests to a sibling `_tests.rs`); SCOPED `-p <crate>` checks only (NEVER
`--workspace` or the full gate -- that is the owner's manual hand-off step).

This plan is authored from the live code, not in the abstract. Key verified
facts that shape it are recorded inline.

---

## Dependency order (three parts)

1. **PART 1 -- Repoint `ff-idcams` traits at `ff-dscatalog`** (idcams-emulator
   Req 28; Task 29). The hard, self-contained slice: add the Cargo dep, verify
   the DAG stays acyclic, delete `ff-idcams`'s private `CatalogService` /
   `VsamService` trait DECLARATIONS, and reconcile every executor call site onto
   `ff-dscatalog`'s trait shapes. Must land FIRST because Part 3's grep pre-check
   depends on it (and because Req 28.6 sequences the repoint before any crate
   deletion).

2. **PART 2 -- Record-aware MAINFRAME editor SAVE** (idcams-emulator Req 28
   composes; command-environments Task 20/21, Req 16). CONCLUSION FROM THE CODE:
   this is a **STOP-and-report candidate** -- it CANNOT be delivered by merely
   wiring a `BackendEnvironment` sibling at the existing seam without a larger
   editor/registry/provider build that the current CR does not fund. Part 2 is
   therefore a thin slice that RECORDS the finding and the precise prerequisites,
   defers the build, and leaves native SAVE untouched. Evidence below.

3. **PART 3 -- Delete `ff-dataset-catalog` + re-express governance**
   (dataset-catalog Task 42; ownership Req 22.1/22.2). Mirrors the RC.B.7
   `ff-vsam-services` retirement EXACTLY. Runs LAST: its grep pre-check must find
   zero Cargo deps on `ff-dataset-catalog` after Part 1.

---

## PART 1 -- Repoint ff-idcams traits (Req 28.1-28.7, Task 29.1-29.8)

### What is actually there (verified)

- `crates/ff-idcams/Cargo.toml` depends ONLY on `thiserror` (+ dev-deps
  proptest, pretty_assertions). No `ff-dscatalog` dep yet.
- `crates/ff-idcams/src/services.rs` declares its OWN private `CatalogService`
  and `VsamService` traits over its own rich param structs (`CreateDatasetParams`,
  `CreateGdgParams`, `UpdateAttrs`, `ListFilter`, `ExportParams`/`ExportResult`,
  `ImportParams`/`ImportResult`, `VsamType`, `VsamInitParams`, `DefineAixParams`,
  `DefinePathParams`, `DatasetHandle`, `OpenMode`, `BrowsePosition`,
  `BrowseCursor`, `Record`, `VerifyResult`, `BuildIndexResult`) and its AST
  `DatasetName`. It also hosts `IdcamsServices` + `mocks::{MockCatalogService,
  MockVsamService, MockAllocatorService, TestServicesBuilder}`.
- `crates/ff-idcams/src/executor/handlers.rs` is the ONLY caller of the traits
  (verified by grep `services.(catalog|vsam|allocator).`): DEFINE CLUSTER/AIX/
  PATH/GDG, DELETE, ALTER, LISTCAT, PRINT, REPRO, VERIFY, EXPORT, IMPORT,
  BLDINDEX.
- `crates/ff-idcams/src/error.rs` defines LOCAL `CatalogError` / `VsamError` /
  `AllocatorError` used by the private traits.

### The reconciliation gap (the crux of Part 1)

`ff-dscatalog`'s reconciled traits have a DIFFERENT shape and types from
`ff-idcams`'s private ones (read `crates/ff-dscatalog/src/service.rs`,
`vsam_service.rs`, `dataset_access/trait_def.rs`):

- `ff_dscatalog::CatalogService`: `&str` DSNs; `create_dataset(&str,
  DatasetAttributes) -> DatasetId`; `delete_dataset(&str)`;
  `update_dataset(&str, DatasetAttributes)`; `rename_dataset(&str,&str)`;
  `list_datasets(&DatasetFilter) -> Vec<DatasetEntry>`;
  `get_dataset_attributes(&str)`; `create_gdg_base(&str,u8,bool)`; plus
  resolve/exists/generation methods. Error type `ff_dscatalog::CatalogError`.
  It has NO `delete_gdg_base`, NO `export_dataset`, NO `import_dataset`.
- `ff_dscatalog::VsamService`: `&str` DSNs; `create_ksds/esds/rrds/lds`,
  `destroy_dataset`, `initialize_dataset(&str, VsamType, VsamParams)`,
  `open(&str, AccessMode) -> VsamHandle`, `get/put/delete`, `close`,
  `start_browse(&VsamHandle,&[u8],BrowseDirection) -> BrowseHandle`,
  `next_record(&BrowseHandle)`, `end_browse`, `define_aix(&str,&str,KeyField)`,
  `build_index(&str)`. Error type `ff_dscatalog::VsamError`. It has NO
  `define_path` / `delete_path`, NO `verify_integrity`.
- `ff_dscatalog::DatasetAccess` (object-safe): `allocate` / `resolve` / `open` /
  `get` / `put` / `point` / `close` / `dispose` over `DatasetHandle` /
  `OpenDataset` / `Record` / `Positioner` / `AccessIntent` / `StepOutcome` /
  `DatasetError`. This is the record-aware I/O contract REPRO's get/put and the
  DEFINE record-storage init map onto (Req 28.3, 28.4).

Req 28 and the task brief resolve this gap EXPLICITLY: adopt ONE trait definition
(ff-dscatalog's); where ff-dscatalog provides an equivalent, USE IT; where
`ff-idcams` has IDCAMS-specific command-param types ff-dscatalog does NOT model
(`DefinePathParams`, `delete_path`, `verify_integrity`, `export`/`import`,
`DefineAixParams` detail, `BuildIndexResult`), KEEP those local param types in
`ff-idcams` but MAP them onto the ff-dscatalog trait calls (or keep them as
orchestration-local helpers where ff-dscatalog models no equivalent at all).

PRECEDENT TO COPY: `ff-dsalloc` already did exactly this (read
`crates/ff-dsalloc/src/catalog_bridge.rs` + `allocation.rs`): it depends on
`ff-dscatalog`, keeps a thin local DTO (`DatasetAllocationRequest`) at the
boundary, and wraps `DatasetAccess`/`CatalogService` behind a small mockable
bridge trait, holding opaque `ff_dscatalog::DatasetHandle`s. Follow that shape.

### Part 1 steps (ordered)

1. **Add the Cargo dep + prove acyclicity BEFORE touching trait code.**
   - Edit `crates/ff-idcams/Cargo.toml`: add `ff-dscatalog = { path =
     "../ff-dscatalog" }` under `[dependencies]`.
   - DAG verification approach (see the dedicated section below): the target DAG
     is `ff-idcams -> ff-dsalloc -> ff-dscatalog -> ff-volume -> ff-vfs`, and
     `ff-idcams -> ff-dscatalog` DIRECTLY is allowed. The ONLY cycle risk is
     `ff-dscatalog` (or `ff-volume` / `ff-vfs`) gaining a dep on `ff-idcams` /
     `ff-dsalloc`. Verified today: `ff-dscatalog/Cargo.toml` depends on ff-vfs,
     ff-volume, ff-command, ff-config, ff-logging -- NONE on ff-idcams/ff-dsalloc.
     Run `cargo check -p ff-idcams` (Cargo itself rejects a dependency cycle at
     resolve time with error E0464/"cyclic package dependency", so a clean
     `cargo check` IS the acyclicity proof); additionally keep the
     `ff-governance-tests` `architecture_compliance.rs` rules green (they assert
     ff-vfs/catalog have no upstream-orchestrator deps).
   - Files: `crates/ff-idcams/Cargo.toml`.
   - Verify: `cargo check -p ff-idcams` compiles (dep resolves, no cycle).

2. **Write the failing reconciliation tests FIRST (TDD red).** In a new
   `crates/ff-idcams/src/services_tests.rs` (sibling test module; keep
   `services.rs` under 400 non-test lines) OR extend the existing executor tests,
   assert against the RECONCILED surface:
   - DEFINE CLUSTER routes through `ff_dscatalog::CatalogService::create_dataset`
     + `VsamService` init (Req 28.3); on init failure the catalog entry is rolled
     back via `delete_dataset` (preserves Req 2.20 / Req 22).
   - REPRO record get/put flows through the reconciled `VsamService` /
     `DatasetAccess` (Req 28.4) -- assert the mock's get/put were driven, not a
     local copy loop.
   - DELETE flows through `CatalogService::delete_dataset` + `VsamService`
     teardown / `DatasetAccess::dispose` (Req 28.5), atomic on failure.
   - CC/output/LASTCC/MAXCC unchanged for a representative multi-command SYSIN
     (Req 28.7) -- reuse the existing integration-test expectations.
   Each test carries `// Validates: Requirement 28.x`. Confirm RED first.
   - Files: `crates/ff-idcams/src/services_tests.rs` (new) and/or
     `crates/ff-idcams/src/executor/handlers.rs` test module; existing
     `crates/ff-idcams/tests/*` integration tests re-pointed if they constructed
     the old mocks.
   - Verify: `cargo test -p ff-idcams` -- the new tests FAIL to compile/assert.

3. **Repoint the trait source (Req 28.1, 28.2).** In
   `crates/ff-idcams/src/services.rs`:
   - DELETE the private `pub trait CatalogService` and `pub trait VsamService`
     declarations.
   - `use ff_dscatalog::{CatalogService, DynCatalogService, VsamService,
     DatasetAccess, DatasetAttributes, DatasetFilter, DatasetEntry, DatasetId,
     VsamType, VsamParams, VsamHandle, BrowseHandle, BrowseDirection, AccessMode,
     KeyField, Record, DatasetHandle, OpenDataset, Positioner, AccessIntent,
     StepOutcome, CatalogError, VsamError, DatasetError};` (import exactly what is
     used).
   - Change `IdcamsServices` to hold `Arc<dyn DynCatalogService>` (object-safe
     wrapper, Req 22.5) + `Arc<dyn VsamService>` + (for record I/O) a handle to a
     `dyn DatasetAccess`, plus the existing `AllocatorService` (which stays local
     -- ff-dscatalog models DD resolution via `DatasetAccess::resolve`, but the
     IDCAMS DD->DSN step is orchestration; keep `AllocatorService` local or map it
     onto `DatasetAccess::resolve`, documented in the code).
   - KEEP the IDCAMS-specific param types ff-dscatalog does not model
     (`DefinePathParams`/`define_path`, `verify_integrity`, `ExportParams`/
     `ImportParams`, `DefineAixParams`, `BuildIndexResult`, `DeleteEntryType`
     dispatch) as `ff-idcams`-local orchestration helpers; do NOT re-add them to a
     private trait. Map them onto `ff-dscatalog` calls where an equivalent exists
     (`define_aix` -> `VsamService::define_aix(&str,&str,KeyField)`;
     `build_index` -> `VsamService::build_index(&str)`; DELETE PATH -> catalog
     `delete_dataset`), and where ff-dscatalog models no equivalent (PATH as a
     first-class VSAM op, VERIFY, EXPORT/IMPORT), keep the behaviour inside
     `ff-idcams`'s executor as an orchestration-only path with a code comment
     stating ff-dscatalog models no trait method for it (Req 21 thin-orchestrator
     boundary preserved; no new storage logic).
   - Rewrite `mocks::MockCatalogService` / `MockVsamService` to implement the
     RECONCILED traits (Req 28.2). The reconciled method set differs, so the mock
     response-vec fields change shape -- model them on the simple
     `ff-dscatalog`/`ff-dsalloc` mocks (read `service.rs` test `MockCatalogService`
     and `catalog_bridge.rs` `MockDatasetAllocator`).
   - Files: `crates/ff-idcams/src/services.rs`, `crates/ff-idcams/src/lib.rs`
     (re-exports: stop re-exporting the deleted local traits; re-export the
     ff-dscatalog ones or drop the re-export), `crates/ff-idcams/src/error.rs`
     (the local `CatalogError`/`VsamError` are superseded by ff-dscatalog's for
     the trait boundary -- keep `IdcamsError` + any executor-only error, remove
     or `#[deprecated]`-then-remove the now-unused local trait error enums; adjust
     `handlers.rs` match arms to ff-dscatalog's error variants).
   - ASCII FIX (do while here): `services.rs` and `lib.rs` currently contain
     box-drawing section separators (`---`) and em dashes -- PROHIBITED in `.rs`
     per `documentation.md`. Replace box-drawing with ASCII `// === ... ===` and
     em dashes with `--` in every file this part touches.
   - Verify: compiles after step 4.

4. **Reconcile the executor call sites (Req 28.3-28.5, 28.7).** In
   `crates/ff-idcams/src/executor/handlers.rs` map each call onto the reconciled
   trait:
   - DEFINE CLUSTER: build `ff_dscatalog::DatasetAttributes` from the parsed
     command and call `catalog.create_dataset(dsn_str, attrs)`; then
     `vsam.initialize_dataset(dsn_str, VsamType::_, VsamParams{..})` (or the
     per-type `create_ksds/esds/rrds/lds`). Record-storage init goes through
     `DatasetAccess` where allocated (Req 28.3). Rollback via `delete_dataset`.
   - DEFINE GDG: `catalog.create_gdg_base(dsn_str, limit, scratch)`. (ff-dscatalog
     has no empty/fifo params -- keep those IDCAMS-local, documented.)
   - DEFINE AIX/PATH: `vsam.define_aix(base,aix,KeyField)`; PATH stays
     orchestration-local (no ff-dscatalog trait method).
   - DELETE: `vsam.destroy_dataset(dsn)` then `catalog.delete_dataset(dsn)`;
     GDG via `catalog.delete_dataset` of the base (no `delete_gdg_base` in the
     reconciled trait -- document); dispose via `DatasetAccess::dispose` where a
     handle is held (Req 28.5).
   - ALTER: `catalog.update_dataset(dsn, attrs)` / `rename_dataset`.
   - LISTCAT: `catalog.list_datasets(&DatasetFilter{..})` +
     `get_dataset_attributes`.
   - PRINT/REPRO: `vsam.open(dsn, AccessMode) -> VsamHandle`,
     `start_browse(&handle,&[u8],Forward) -> BrowseHandle`,
     `next_record(&browse)`, `put(&handle,&Record)`; REPRO record copy flows
     through these / `DatasetAccess` get/put (Req 28.4), NOT a local loop that
     re-implements record handling.
   - VERIFY / EXPORT / IMPORT / BLDINDEX: VERIFY and EXPORT/IMPORT have no
     reconciled trait method -- keep as orchestration-local behaviour with a
     comment; BLDINDEX -> `vsam.build_index(aix_dsn)`.
   - Map error variants to `ff_dscatalog::CatalogError` / `VsamError` in the
     match arms; keep the SAME IDC message codes and CC so Req 28.7 holds.
   - Files: `crates/ff-idcams/src/executor/handlers.rs` (watch the 400-line rule;
     if it grows past 400 non-test lines, split by command group into
     `handlers_define.rs` / `handlers_delete.rs` / `handlers_io.rs` as a REFACTOR).
   - Verify: `cargo test -p ff-idcams` GREEN (step-2 tests pass); CC/output
     integration tests unchanged.

5. **Grep + behaviour confirmation (Req 28.6, 28.7).**
   - `grep` for `ff-vsam-services`/`ff_vsam_services` and for any removed-crate
     reference inside `ff-idcams` -- expect none.
   - Confirm no private `trait CatalogService`/`trait VsamService` remains in
     `ff-idcams`.
   - Files: none (verification only).
   - Verify: `grep_search` returns zero live references; `cargo test -p ff-idcams`
     and `cargo clippy -p ff-idcams -- -D warnings` clean; `cargo fmt`.

### DAG-acyclicity verification approach (Req 28.1, ownership Req 22.6) -- top risk

- TARGET DAG: `ff-idcams -> ff-dsalloc -> ff-dscatalog -> ff-volume -> ff-vfs`;
  a DIRECT `ff-idcams -> ff-dscatalog` edge is explicitly allowed. The invariant:
  `ff-dscatalog`, `ff-volume`, `ff-vfs` must gain NO dependency on `ff-idcams`
  or `ff-dsalloc`.
- MECHANICAL PROOF 1 (Cargo): a dependency cycle is a hard Cargo resolve error.
  After adding the dep, `cargo check -p ff-idcams` succeeding IS the proof that
  no cycle was introduced (Cargo refuses to build a cyclic graph).
- MECHANICAL PROOF 2 (governance): `cargo test -p ff-governance-tests` must stay
  green -- `tests/architecture_compliance.rs` asserts `ff-vfs` has no
  domain/orchestrator deps and (today) that catalog has no upstream deps;
  `src/compliance.rs` `PROHIBITED_DEPENDENCIES` encodes the same intent. (Part 3
  re-expresses the `ff-dataset-catalog`-named rules; the ACYCLIC intent must
  remain covered -- see Part 3.)
- PROOF 3 (read): confirm `crates/ff-dscatalog/Cargo.toml`,
  `crates/ff-volume/Cargo.toml`, `crates/ff-vfs/Cargo.toml` list NO
  `ff-idcams`/`ff-dsalloc` dep (verified today for ff-dscatalog).

### Part 1 TDD tests (file -> what it asserts -> Req)

- `crates/ff-idcams/src/services_tests.rs` (new):
  - `idcams_services_hold_reconciled_ff_dscatalog_traits` -- `IdcamsServices`
    constructs over `dyn DynCatalogService` + `dyn VsamService` (Req 28.1, 28.2).
  - `define_cluster_routes_through_reconciled_catalog_and_vsam` -- create_dataset
    + initialize_dataset driven; rollback on init error (Req 28.3).
  - `repro_record_copy_flows_through_dataset_access_or_vsam` -- get/put driven via
    the reconciled seam, no local record logic (Req 28.4).
  - `delete_flows_through_delete_dataset_and_dispose_atomically` (Req 28.5).
  - `no_private_service_traits_remain` -- a compile-level assertion / doc test
    that `ff_idcams::services` re-exports the ff-dscatalog traits, not local ones
    (Req 28.1, 28.6).
- Reuse/retarget `crates/ff-idcams/tests/*` integration tests for CC/output
  invariance (Req 28.7); each gets `// Validates: Requirement 28.7`.

### Part 1 scoped verification
`cargo fmt`; `cargo check -p ff-idcams --all-targets`;
`cargo test -p ff-idcams`; `cargo clippy -p ff-idcams -- -D warnings`;
`cargo test -p ff-governance-tests` (acyclicity + mock still green);
`cargo check -p ff-dsalloc` (shares ff-dscatalog, confirm no break).

---

## PART 2 -- Record-aware MAINFRAME editor SAVE -- STOP-AND-REPORT (with evidence)

### Determination

The record-aware MAINFRAME SAVE is **NOT achievable by merely wiring a
`BackendEnvironment` sibling at the existing seam**. It requires a larger
editor/registry/provider build that this CR does not scope. Part 2 is therefore
a documentation-only slice: record the finding + prerequisites, defer the build,
leave native SAVE byte-identical. This matches the task brief's explicit
instruction to FLAG it as a STOP-and-report candidate rather than force it, and
it is corroborated by the spec itself.

### Evidence from the live CE / SAVE code

1. **The `BackendEnvironment` contract is a BYTE write, not record-aware.**
   `crates/ff-vfs/src/backend_environment.rs`:
   `fn save(&self, path: &Path, bytes: &[u8]) -> std::io::Result<()>`. A
   record-aware save through `ff_dscatalog::DatasetAccess` needs the DSN, the
   dataset's RECFM/LRECL/catalog identity, and `Record` boundaries -- NONE of
   which `(path, bytes)` carries. Making the MAINFRAME CE route through
   `DatasetAccess` would require CHANGING this `ff-vfs` trait signature (a core
   framework type) -- a FRAMEWORK CHANGE requiring express owner confirmation
   (framework-conformance mechanism; `ff-vfs::BackendEnvironment` is load-bearing).

2. **The registry is CLOSED to arbitrary named backends.**
   `crates/ff-desktop/src/shell/environment_registry.rs`: `EnvironmentRegistry`
   holds a closed `enum RegisteredEnv { FfCmdBase, FfEdit, HostFsPlaceholder }`
   plus a SINGLE `host_fs: Box<dyn BackendEnvironment>`. There is no map of named
   boxed backends and no "MAINFRAME" entry.
   `crates/ff-desktop/src/shell/commands_environment.rs`:
   `dispatch_to_environment` matches that closed enum -- only `HostFsPlaceholder`
   performs a `save`. Addressing SAVE to a MAINFRAME backend needs this registry +
   dispatcher extended to resolve and invoke additional named boxed backends.

3. **A mainframe-catalog dataset is NOT bound to a MAINFRAME environment today.**
   `crates/ff-desktop/src/shell/render_body.rs::open_mainframe_dsn` resolves the
   DSN to a HOST PATH and (in `render_body_arms.rs`) opens it via `file.open`
   with ONLY `path` -- NO `owning_env`. So the tab binds to
   `DEFAULT_OWNING_ENVIRONMENT = "HOSTFS"` and SAVE already (correctly) goes to
   the host-FS byte write. The editor nav explicitly rejects mainframe dataset
   editing ("available in a later update",
   `crates/ff-desktop/src/shell/render_nav_expand.rs`). There is no live code path
   that produces a MAINFRAME-owned editable tab to save.

4. **The provider prerequisite (Req 17) is only partially met.** The live
   `ff-vfs::ProviderRegistry` IS registered at startup
   (`construct_provider.rs::build_live_provider_registry`) but seeded ONLY with
   the host-FS `local` provider; the mainframe VFS provider is a later ("V"-stream)
   phase (`shell/actions.rs` `#[allow(dead_code)]` note). command-environments
   Req 17.4 states the non-host parts of Req 14-16 DEPEND ON a mainframe provider
   being registered -- not yet done.

5. **The spec itself defers this.** command-environments Req 16 (Marking):
   "the mainframe CE (housed in `ff-idcams`) is the first CE that genuinely
   diverges and **is built in a later phase**." Req 16.1/16.4 house the mainframe
   CE in `ff-idcams`, NOT a `ff-ce-mainframe` sibling (no such crate exists -- file
   search confirms). So even the crate shape in the task hint ("sibling to
   ff-ce-posix/ntfs/host-fs") contradicts the spec's "housed in ff-idcams" and
   would itself be a design decision for the owner.

### What Part 2 delivers (the thin, safe slice)

- A short finding note `.agents/tasks/rcb8-dataset-rationalisation/part2-mainframe-save-stop.md`
  recording the five evidence points above and the concrete PREREQUISITES a
  future slice needs: (a) owner-confirmed `BackendEnvironment` record-aware
  contract (or a record-aware addressing variant that carries DSN + attributes +
  records, not `(path,bytes)`); (b) an open named-backend registry +
  `dispatch_to_environment` arm; (c) `open_mainframe_dsn` passing
  `owning_env = "MAINFRAME"` and binding the tab; (d) the mainframe VFS provider
  registered live (Req 17.4); (e) the mainframe CE housed in `ff-idcams`
  implementing the contract over `DatasetAccess`.
- NO `.rs` behaviour change: native/host SAVE stays byte-identical; no new crate;
  no `ff-vfs` trait change.
- A `send_message` severity `warning` STOP to the owner summarising the finding
  and asking whether to (i) accept the deferral and proceed to Part 3, or
  (ii) open a separate owner-confirmed FRAMEWORK-change slice to build it.

### Part 2 TDD tests

None (documentation-only; no behaviour change). The guard is the NEGATIVE
evidence above plus the existing host-save tests
(`crates/ff-desktop/src/tab_manager.rs` `save_active_tab_via_backend` tests and
`shell/tests_session.rs` `host_path_open_is_unchanged_by_the_live_provider_registry`)
which must remain green and prove native SAVE is untouched. If the owner elects
to build it later, that slice writes the first-fail test:
"a MAINFRAME-owned dataset SAVE drives `DatasetAccess::put`, not a raw byte
write" -- which cannot be written today because no MAINFRAME-owned tab exists.

### Part 2 scoped verification
`cargo test -p ff-desktop` (host-save + provider-registry tests still green --
proves no regression). No other change.

---

## PART 3 -- Delete ff-dataset-catalog + re-express governance (Task 42; ownership 22.1/22.2)

### Pre-check (must pass before any deletion)

- `grep_search` `ff-dataset-catalog`/`ff_dataset_catalog` across `**/*.toml` and
  `**/*.rs`. VERIFIED TODAY: the ONLY references are (a) root `Cargo.toml`
  `[workspace].members` line `"crates/ff-dataset-catalog",`; (b) the crate's own
  `crates/ff-dataset-catalog/**`; (c) governance in
  `ff-governance-tests/src/compliance.rs` + `tests/architecture_compliance.rs`;
  (d) stale doc prose. NO shipping crate (incl. `ff-idcams` after Part 1) has it
  as a Cargo dependency. This is the SAME situation RC.B.7 had for
  `ff-vsam-services` -- safe to retire, no STOP.
- `mock_compilation.rs` already imports `ff_dscatalog` (verified: no residual
  `ff_dataset_catalog` import).

### Part 3 steps (mirror RC.B.7 exactly -- see
`.agents/tasks/rcb7-vsam-wiring/feat003-finish-note.md`)

1. **Delete the crate.** Remove dir `crates/ff-dataset-catalog/` (entire tree)
   and its `[workspace].members` entry `"crates/ff-dataset-catalog",` from root
   `Cargo.toml` (around line 57). `Cargo.lock` regenerates on the next scoped
   `cargo check`.
   - Files: `Cargo.toml`; delete `crates/ff-dataset-catalog/`.

2. **Re-express governance -- `crates/ff-governance-tests/src/compliance.rs`:**
   remove the TWO `ff-dataset-catalog` `DependencyRule`s (the
   `ff-dataset-catalog -> ff-idcams` and `-> ff-dsalloc` prohibitions, ~lines
   93-105) AND the `ff-vfs -> ff-dataset-catalog` prohibition (~lines 80-92). The
   catalog/acyclic intent remains covered by the surviving `ff-vfs -> ff-idcams`,
   `ff-vfs -> ff-dsalloc`, and `ff-dsalloc -> ff-idcams` rules (same reasoning
   RC.B.7 used: catalog is a lower-level service already protected by the ff-vfs
   and ff-dsalloc rules). Do NOT invent a new rule unless removing these leaves a
   governed crate with ZERO rules (it does not -- `ff-dscatalog` is not in the
   governed dataset-crate list; the acyclic DAG for the live crates stays
   asserted by the ff-vfs/ff-dsalloc rules).

3. **Re-express governance -- `crates/ff-governance-tests/tests/architecture_compliance.rs`:**
   - `vfs_has_no_domain_dependencies` (~line 33): remove the
     `!deps.contains_key("ff-dataset-catalog")` assertion block.
   - DELETE the whole `dataset_catalog_has_no_upstream_dependencies` test
     (~lines 44-66) -- it targets the removed crate (RC.B.7 deleted the analogous
     `vsam_services_has_no_upstream_dependencies`).
   - `prohibited_rules_cover_all_dataset_crates` (~line 165): remove
     `"ff-dataset-catalog"` from the `dataset_crates` array (else it asserts a
     rule exists for a crate that now has none).
   - `all_governed_crates_exist` (~line 230): remove the
     `("ff-dataset-catalog", true)` entry from `required_crates` (the crate no
     longer exists).

4. **Confirm `mock_compilation.rs`** has no residual `ff_dataset_catalog` import
   (already true; re-verify after the deletion compiles).

5. **Fix stale doc prose** naming `ff-dataset-catalog` as the catalog AUTHORITY
   (not historical narrative): any `.rs` doc comment / `docs/specs` prose that
   calls `ff-dataset-catalog` the single authority -> `ff-dscatalog`. Leave
   genuine historical record (RC.B.7 note, RESUME history, change-log) untouched,
   exactly as RC.B.7 did. Target the idcams-emulator `requirements.md` Delegation
   Model table (it still names `ff-dataset-catalog` / `ff-vsam-services` as
   delegates) ONLY if the gate owner wants it; otherwise note it as prose to
   reconcile. (The ownership-model Req 22 note already instructs readers to read
   `ff-dataset-catalog` as `ff-dscatalog`, so spec prose is covered by that note;
   prioritise `.rs` doc comments.)

### Part 3 scoped verification
`cargo fmt`;
`cargo check -p ff-governance-tests -p ff-dscatalog -p ff-dsalloc -p ff-idcams --all-targets`;
`cargo test -p ff-governance-tests` (THE KEY CHECK -- architecture_compliance +
mock_compilation GREEN after rule edits);
`cargo check -p ff-desktop --all-targets` (app closure still builds without the
crate);
final `grep_search` -- NO live `ff-dataset-catalog`/`ff_dataset_catalog` in any
`*.toml`/`*.rs`.

---

## Anticipated out-of-scope / full-gate risks (owner's manual `cargo gate --build`)

- `ff-idcams` integration tests under `crates/ff-idcams/tests/` may construct the
  OLD mocks and need retargeting; scoped `cargo test -p ff-idcams --all-targets`
  catches this (run `--all-targets`).
- `Cargo.lock` churn from the removed member + the new `ff-idcams -> ff-dscatalog`
  edge: regenerated by scoped `cargo check`; the full gate confirms the whole
  workspace still resolves.
- Orphan crates / proptest >=100-iteration runs are only exercised by the full
  `--workspace` gate -- hence the owner hand-off.
- `ff-dscatalog` is a heavier dependency (rusqlite, tokio, zip, chrono). Adding
  it to `ff-idcams` enlarges `ff-idcams`'s build but introduces no cycle; confirm
  `idcams_has_no_storage_engine_dependencies` governance test semantics -- it
  checks `ff-idcams`'s OWN Cargo.toml for rusqlite/rocksdb/lmdb (TRANSITIVE deps
  via ff-dscatalog are NOT a direct dep, so the test still passes; verify this
  assumption with `cargo test -p ff-governance-tests` -- if it inspects the lock
  graph it may flag, which would be an owner FLAG).

## TCR rows to flip (docs/quality/TCR.md), tasks to mark

- TCR `ff-idcams` Req 28.1-28.7 (lines ~1752-1758): 🔴 NOT COVERED -> ✅ with the
  Part 1 test names, EXCEPT any criterion Part 2's STOP defers -- Req 28 itself is
  satisfied by Part 1 (the repoint); the MAINFRAME SAVE is a COMPOSING concern
  recorded as deferred, not a 28.x criterion, so all 28.1-28.7 can flip to ✅ once
  Part 1 is green. Cite the `services_tests.rs` tests + CC-invariance integration
  tests.
- TCR dataset-catalog Req 35.1a/c/d + 35.2 and ownership Req 22.1/22.2: update to
  reflect `ff-dataset-catalog` deleted + governance re-expressed (Part 3), mirror
  the RC.B.7 wording for Req 35.3/22.3.
- Tasks: idcams-emulator `tasks.md` Task 29.1-29.8 `[ ]` -> `[x]` (29.8 = TCR +
  hand-off). dataset-catalog `tasks.md` Task 42 (42.1, 42.2) `[ ]` -> `[x]`.
- `docs/project-management/project-master/tasks.md` RC.B.8 row (line ~3451)
  `[ ]` -> `[x]` with a "code-complete pending the owner's full `cargo gate
  --build`" note; record the Part 2 STOP/deferral explicitly.
- `docs/status/RESUME-dataset-rationalisation.md`: mark RC.B.8 done-pending-gate;
  note Part 2 deferred with the prerequisites.

## Documentation / character rules

Every `.rs` file touched must be ASCII-only (fix the existing box-drawing +
em-dash violations in `services.rs` / `lib.rs` while editing them). `docs/`
Markdown: ASCII substitutes per `.kiro/steering/documentation.md` (no em dash /
curly quotes).

## Hand-off

After each part's SCOPED checks are clean, STOP and hand off to the owner to run
the full `cargo gate --build` manually (fallback `pwsh -ExecutionPolicy Bypass
-File tools\ffwb-gate.ps1`). Do NOT run `--workspace` or `cargo gate` from Kiro.
