# RC.B.8 Part 2 (b)-(d) + BRC.4 -- Record-aware MAINFRAME editor SAVE

TDD implementation plan. One coherent end-to-end slice: the mainframe Command
Environment (BRC.4 / prerequisite (e)) PLUS the registry/open/provider wiring
(b)-(d) that makes it reachable. Edits go DIRECTLY on `main`, uncommitted, for
the owner's full gate. ASCII-only `.rs`; 400-non-test-line rule; build ON the
framework.

The loop stop contract: the reviewer writes
`.agents/tasks/rcb8-part2-brc4/review.json` with `verdict` == `"APPROVED"`.

---

## 0. ANTICIPATED WIRING RISKS (read first -- NOT a hard blocker)

This slice is buildable within scope, but two integration points are larger than
a trivial sibling-wiring and MUST be built deliberately. Neither is the
"ballooning blocker" that forces a STOP (each has a concrete, bounded
resolution), but both are the places this slice can grow if done carelessly.

### Risk 1 -- DSN identity is LOST at open (must be threaded)

VERIFIED from live code: `open_mainframe_dsn`
(`crates/ff-desktop/src/shell/render_body.rs`) resolves a DSN to a HOST PATH and
the tab is opened by that host path (`file.open { path }`). So today
`TabState.path` is a host path and `save_active_tab_via_backend`
(`crates/ff-desktop/src/tab_manager.rs`) builds
`StoreTarget { dsn: path.to_string_lossy(), owning_env, catalog_id: None }` -- a
HOST PATH in the `dsn` field, NOT a real DSN. A mainframe CE storing over
`DatasetAccess` resolves BY DSN, so it needs the real DSN + catalog name, which
the host-path open discards.

RESOLUTION (bounded, in this slice): carry the real dataset identity onto the
tab at open and into `StoreTarget`. The `file.open` command already carries an
optional `owning_env` param (`FileOpenHandler`, `handlers.rs`); ADD a parallel
optional `dsn` + `catalog` pair the handler stashes alongside `owning_env` in
`pending_open`, applied by `update.rs` onto a new `TabState` field
`store_identity: Option<StoreIdentity { dsn, catalog }>`. Then
`save_active_tab_via_backend` builds `StoreTarget` from `store_identity` when
present (real DSN + `catalog_id = Some(catalog)`), falling back to the host-path
placeholder when absent (host tabs unchanged). This keeps the SINGLE save seam
and the SINGLE open command; it only enriches the payload.

### Risk 2 -- the shell holds NO DatasetAccess / VolumeRegistry / StorageProvider

VERIFIED: ff-desktop's `crate::catalog_registry::CatalogRegistry` (the
`ff-catalog-registry` crate) resolves a DSN to a path via
`Catalog::physical_path` only (`crates/ff-catalog-registry/src/lib.rs`); it
exposes NO `VolumeRegistry`, NO `ff_vfs::StorageProvider`, and NO
`DatasetAccess`. `ff_dscatalog::CatalogDatasetAccess::new(provider: Arc<dyn
StorageProvider>, registry: VolumeRegistry)` needs BOTH. There are also TWO
`CatalogRegistry` types: the ff-desktop/`ff-catalog-registry` one (used by the
Files Panel) and `ff_dscatalog::catalog_registry::CatalogRegistry` (used by
`CatalogVfsProvider` and `CatalogDatasetAccess`'s resolution). The shell's
mounted-catalog state is the former.

RESOLUTION (bounded, in this slice): the mainframe CE OWNS an `Arc<dyn
ff_dscatalog::DatasetAccess>` (object-safe) and is constructed in ff-desktop at
registry-build time from a `ff_dscatalog::CatalogDatasetAccess::new(provider,
volume_registry)`. For the FIRST slice the provider is the native
`StorageProvider` over the repository root and the volume registry is the one the
dscatalog repository already seeds (the same construction the
`CatalogDatasetAccess` tests use: a `Posix/NativeNativeProvider` + a
`VolumeRegistry` from the repo). The CE RESOLVES/ALLOCATES by DSN (so the save
targets the catalog's own store, co-located with what the open read).

WHY THIS IS NOT THE HARD-STOP BLOCKER: the construction is a bounded composition
of types that already exist and are already unit-tested together
(`CatalogDatasetAccess::new` + a native provider + a volume registry). It does
NOT require a new framework mechanism, a second dispatcher, or a new persisted
format. IF, during BRC.4, it emerges that a faithful SAVE requires the shell to
share the EXACT same live `CatalogRegistry`/volume handles the Files Panel opened
(so a save is visible to a subsequent open in the same session) AND that cannot
be obtained without a new cross-crate handle the shell does not own -- STOP and
record that specific blocker in `review.json`/FEAT findings rather than inventing
a parallel store. The co-located-by-DSN construction above is the default and
should hold.

### DAG (unchanged, enforced)

`ff-idcams -> ff-dscatalog -> ff-volume -> ff-vfs`. ff-vfs/ff-volume/ff-dscatalog
gain NO dependency on ff-idcams or ff-desktop. The mainframe CE lives in
`ff-idcams`; ff-desktop constructs and registers it. ff-idcams gains a dependency
on `ff-vfs` (to implement `BackendEnvironment`) -- a new edge INTO a lower DAG
layer, which is acyclic. See the governance note in section 7.

---

## 1. What already exists (do NOT rebuild)

- `ff_vfs::BackendEnvironment` with `save_records(target, records, attrs) ->
  RecordStoreOutcome`, `record_capable()`, `StoreTarget`, `RecordAttrs`,
  `RecordFormatKind`, `RecordSource`, `RecordStoreOutcome` (byte `save` retained).
  (BRC.2, `crates/ff-vfs/src/backend_environment.rs`.)
- `TabManager::save_active_tab_via_backend(backend, runtime)` ALREADY selects the
  record path by `Document::record_format()` + `backend.record_capable()`, builds
  the `RecordImageSource` + `RecordAttrs`, calls `save_records`, maps
  `Stored { rc }` (rc 0 -> clear dirty; rc != 0 -> `Err`), and falls back to the
  byte path on `NotRecordCapable`. (BRC.3, `crates/ff-desktop/src/tab_manager.rs`.)
- `file.open` carries an OPTIONAL `owning_env` param already
  (`FileOpenHandler`, `handlers.rs`; applied in `update.rs` via
  `shell_open_file_with_env`).
- `TabState.owning_environment` (default `"HOSTFS"`) + `active_owning_environment()`.
- `ff_dscatalog::{DatasetAccess, CatalogDatasetAccess, DdRequest, DatasetHandle,
  OpenDataset, AccessIntent, StepOutcome, DatasetError, Record, Dsorg, Recfm}`
  (object-safe trait; `new(provider, registry)` ctor; x37 surfaces as
  `DatasetError::SpaceAbend`, volume-full as `VolumeFull`).
- `ff_dscatalog::CatalogVfsProvider` (scheme `"catalog"`, a `VfsProvider`).
- `build_live_provider_registry` (`construct_provider.rs`) seeds only the host-FS
  `local` provider.

## 2. Files to touch

NEW:
- `crates/ff-idcams/src/mainframe_env.rs` -- the mainframe `BackendEnvironment`
  (BRC.4). Split into `mainframe_env.rs` (+ `mainframe_env_tests.rs` if the test
  module exceeds ~200 lines).
- (if needed) `crates/ff-desktop/src/shell/mainframe_backend.rs` -- the ff-desktop
  helper that CONSTRUCTS the mainframe CE (provider + volume registry +
  `CatalogDatasetAccess`) and the `StoreIdentity` plumbing, to keep
  `environment_registry.rs` and `construct*.rs` under 400 lines.

MODIFY:
- `crates/ff-idcams/Cargo.toml` -- add `ff-vfs = { path = "../ff-vfs" }`.
- `crates/ff-idcams/src/lib.rs` -- `pub mod mainframe_env;` + re-export.
- `crates/ff-desktop/Cargo.toml` -- add `ff-idcams = { path = "../ff-idcams" }`.
- `crates/ff-desktop/src/shell/environment_registry.rs` -- (b) named-backend map +
  `MAINFRAME_NAME` + `backend_for`.
- `crates/ff-desktop/src/shell/dispatch_ffedit.rs` -- (b) owning-env-aware save
  resolution in `host_fs_save`.
- `crates/ff-desktop/src/shell/handlers.rs` + `state.rs` + `construct_commands.rs`
  + `update.rs` -- (c) thread `dsn`/`catalog` through `file.open` ->
  `pending_open` -> tab `store_identity`.
- `crates/ff-desktop/src/tab_state.rs` + `tab_state_ctors.rs` -- (c) add
  `store_identity: Option<StoreIdentity>` (default `None`).
- `crates/ff-desktop/src/tab_manager.rs` -- (c) build `StoreTarget` from
  `store_identity` when present; set `Document::set_record_format` at open for a
  mainframe tab (or do the set in the open flow).
- `crates/ff-desktop/src/shell/render_body.rs` / `render_body_arms.rs` /
  `render_nav.rs` -- (c) pass `owning_env="MAINFRAME"` + `dsn` + `catalog` to
  `file.open` at the mainframe open sites; look up the dataset RECFM/LRECL from
  the catalog to set the open RecordFormat.
- `crates/ff-desktop/src/shell/render_nav_expand.rs` -- (c) lift the "Editing
  Mainframe datasets is available in a later update." gate (two sites).
- `crates/ff-desktop/src/shell/construct_provider.rs` -- (d) also register the
  `catalog`-scheme mainframe provider.

## 3. BRC.4 -- the mainframe Command Environment (ff-idcams)

Shape (`mainframe_env.rs`):

```
pub struct MainframeEnvironment {
    access: std::sync::Arc<dyn ff_dscatalog::DatasetAccess>,
}
impl MainframeEnvironment {
    pub fn new(access: std::sync::Arc<dyn ff_dscatalog::DatasetAccess>) -> Self { ... }
}
impl ff_vfs::BackendEnvironment for MainframeEnvironment {
    fn name(&self) -> &str { "MAINFRAME" }
    fn is_case_sensitive(&self) -> bool { false }
    fn save(&self, path, bytes) -> std::io::Result<()> { /* byte save = error:
        a mainframe resource has no meaningful host-path byte save; return
        std::io::Error::other("MAINFRAME is record-only; use save_records") */ }
    fn record_capable(&self) -> bool { true }
    fn save_records(&self, target, records, attrs) -> RecordStoreOutcome { ... }
}
```

`save_records` (the heart of BRC.4, Req 18.7 / 18.8):
1. Resolve the dataset by `target.dsn` via `access.resolve(dsn,
   AccessIntent::Write)` (the dataset exists -- the open flow created/resolved
   it). On `DatasetError::NotFound` -> `allocate` a PS dataset from `attrs`
   (RECFM/LRECL mapped `RecordFormatKind -> ff_dscatalog::Recfm`: Fixed->FB,
   Variable->VB, Undefined->U). If allocation identity (volume/space) is not
   available from `attrs` alone, prefer resolve-only and map NotFound to a
   non-zero rc (document the choice in the CE doc comment).
2. `let mut open = access.open(&handle, AccessIntent::Write)?;`
3. Pull every record from `records.next_record()` and
   `access.put(&mut open, &ff_dscatalog::Record { key: vec![], data:
   rec.to_vec() })?` (sequential PS: empty key, bytes in `data`).
4. `access.close(open)?` -- the RECFM codec frames bytes on close (Req 18.7).
5. Map the result to `RecordStoreOutcome` (Req 18.8):
   - `Ok(())` -> `Stored { rc: 0 }`.
   - `Err(DatasetError::SpaceAbend { .. })` -> `Stored { rc: 37 }` (x37 abend ->
     non-zero rc; pick a stable code, document it; `37` reads as x37).
   - `Err(DatasetError::VolumeFull { .. })` -> `Stored { rc: 37 }` (space class).
   - `Err(DatasetError::ReadOnlyVolume { .. })` -> `Stored { rc: 8 }`.
   - other `Err(_)` -> `Stored { rc: 12 }` (document the mapping table).
   The CE NEVER returns `NotRecordCapable` (it IS record-capable).

Keep the whole file < 400 non-test lines; the rc-mapping table + record-pump are
small. Record attrs -> Recfm/Dsorg mapping is a private helper.

## 4. (b) Registry named-backend map + backend_for (environment_registry.rs)

- Add `pub(super) const MAINFRAME_NAME: &str = "MAINFRAME";`.
- Generalise the single `host_fs: Box<dyn BackendEnvironment>` into a NAME-keyed
  collection holding BOTH host_fs and the mainframe CE. Minimal shape that keeps
  `host_fs_backend()` working: keep `host_fs` as-is AND add `mainframe:
  Box<dyn ff_vfs::BackendEnvironment>` (or a `Vec<(&'static str, Box<dyn ...>)>`
  map). Prefer the explicit two-field form if the map complicates `with_builtins`
  construction order; either satisfies the rule.
- Add `pub(super) fn backend_for(&self, name: &str) -> &dyn
  ff_vfs::BackendEnvironment` resolving case-insensitively: `MAINFRAME` -> the
  mainframe CE; `HOSTFS` (and ANY unknown name) -> host_fs (fallback preserves
  host behaviour). Add a `RegisteredEnv::MainframePlaceholder` entry + an
  `EnvironmentEntry` so membership/`resolve`/`active_name` stay consistent (the
  active-env derivation + claim gate are UNCHANGED -- a mainframe-owned EDITOR tab
  still derives FFEDIT as its active env; MAINFRAME is only an ADDRESSING/backend
  target, never an active interactive env).
- `with_builtins` cannot construct the mainframe CE itself (it needs a
  `DatasetAccess`). Add a constructor variant, e.g. `with_builtins_and_mainframe(
  mainframe: Box<dyn ff_vfs::BackendEnvironment>)`, called from `construct.rs`
  with the CE built by the ff-desktop helper (section 6). Keep `with_builtins()`
  for tests that do not need the mainframe backend (it installs a declining /
  host fallback for MAINFRAME so `backend_for("MAINFRAME")` is always safe).

## 5. (b) Owning-env-aware save resolution (dispatch_ffedit.rs)

In `host_fs_save` (the single CR-CH-053 Task 20/21 save seam -- do NOT add a
second path): replace the hardcoded `environments.host_fs_backend()` with a
resolve by the ACTIVE TAB's owning environment:

```
let result = {
    let Self { environments, tabs, runtime, .. } = self;
    let owning = tabs.active_tab().owning_environment.clone();
    let backend = environments.backend_for(&owning);   // HOSTFS/unknown -> host_fs
    tabs.save_active_tab_via_backend(backend, runtime)
};
self.open_error = result.err();
```

(Rename the method's doc to note it now resolves the owning backend; the method
name may stay `host_fs_save` or be renamed `addressed_save` -- a rename is a pure
refactor, optional.) Host tabs resolve to `host_fs` and the byte path, BYTE-
IDENTICAL. A MAINFRAME tab resolves to the mainframe CE and the record path.

## 6. (c) Bind owning_env + DSN identity + RecordFormat at open

- `tab_state.rs`: add `pub store_identity: Option<StoreIdentity>` where
  `StoreIdentity { pub dsn: String, pub catalog: String }`; default `None` in all
  ctors (`tab_state_ctors.rs`). Host tabs stay `None` -> unchanged.
- `handlers.rs` `FileOpenHandler`: read optional `dsn` + `catalog` params
  alongside `owning_env`; stash all three in `pending_open` (widen
  `state::PendingOpen` payload from `(String, Option<String>)` to carry the
  identity, e.g. `(String, Option<String>, Option<(String,String)>)` or a small
  struct `PendingOpenReq`). Update the two tests in `tests_command.rs` that read
  the pending tuple shape.
- `update.rs`: when applying the pending open, after
  `shell_open_file_with_env(path, owning_env)`, set the active tab's
  `store_identity` from the carried `(dsn, catalog)` and set its document
  `RecordFormat` from the dataset RECFM/LRECL (see below).
- `tab_manager.rs` `save_active_tab_via_backend`: build `StoreTarget` from
  `store_identity` when present (`dsn = id.dsn`, `catalog_id = Some(id.catalog)`),
  else the existing host-path placeholder. (`owning_env` field unchanged.)
- Open sites (`render_body_arms.rs`, `render_nav.rs`): at the mainframe branch,
  look up the dataset's RECFM/LRECL via the catalog (the registry already
  resolves the record -- use `list_datasets`/the resolved `DatasetRecord`
  attributes), and pass `owning_env="MAINFRAME"`, `dsn`, `catalog`, into
  `file.open`. Set the opened document's `RecordFormat` to
  `Fixed { lrecl }` / `Variable { max_lrecl, rdw }` so BRC.3's selection fires
  (document-model Req 11.2). The RecordFormat set happens shell-side in
  `update.rs` (it owns the just-opened tab + runtime) from the identity's cataloged
  attrs; thread the RECFM/LRECL in the pending payload too, OR re-query the catalog
  in `update.rs`. Prefer threading it (avoids a second catalog read).
- `render_nav_expand.rs`: REMOVE the two `"Editing Mainframe datasets is
  available in a later update."` early-returns so a mainframe dataset routes to
  the open flow. (Leave the "Dataset member browsing" gate -- PO member browsing
  is a separate later slice; only the dataset-edit gate is lifted.)

## 7. (d) Register the live mainframe provider (construct_provider.rs)

In `build_live_provider_registry`, after the host-FS `local` provider, ALSO
register the `catalog`-scheme provider so a MAINFRAME owning-environment can
read/write its store (Req 17.4):

```
// construct the shell's dscatalog CatalogRegistry wrapped Arc<RwLock<..>> (the
// SAME mounted catalogs), then:
let catalog_provider: Arc<dyn VfsProvider> =
    Arc::new(ff_dscatalog::CatalogVfsProvider::new(registry_arc));
let _ = registry.register(catalog_provider);   // best-effort, like the host one
```

If the live `ff_dscatalog::CatalogRegistry` is not readily constructible at
provider-build time (the shell holds the `ff-catalog-registry` wrapper, not the
dscatalog registry), register the provider from whatever mounted-catalog state is
available and log a warning on failure (best-effort startup, matching the host
provider's error handling). This registration is ADDITIVE -- host open/save does
not consult the registry, so native access is unchanged whether or not the
catalog provider registers (Req 17.3).

## 8. Failing-test-first list (the three REQUIRED tests)

Write each test FIRST, confirm it FAILS (red), then implement. Every test carries
`// Validates: Requirement X.Y`.

### T1 -- ff-idcams: CE record round-trip + x37 -> rc (`mainframe_env_tests.rs`)
- `mainframe_env_stores_records_via_dataset_access_round_trip`: build a
  `MainframeEnvironment` over a real `CatalogDatasetAccess` on a `TempDir` (mirror
  `impl_access_tests.rs` setup: native provider + volume registry, allocate a PS
  FB dataset). Drive `save_records` with a `RecordSource` of N records; then
  `open`+`get` the dataset back through the same access and assert the records
  round-trip through the RECFM codec (boundaries are the codec's, not CRLF).
  Validates: command-environments 18.7.
- `mainframe_env_maps_space_abend_to_nonzero_rc`: use a volume sized to force a
  `DatasetError::SpaceAbend`/`VolumeFull` on `close`/`put` (mirror
  `put_growth_surfaces_x37_space_abend`); assert `save_records` returns
  `Stored { rc }` with `rc != 0` (x37 class). Validates: 18.8.
- `mainframe_env_is_record_capable_and_object_safe`: `record_capable()` is true;
  usable as `Box<dyn ff_vfs::BackendEnvironment>`. Validates: 18.3/18.4.

### T2 -- ff-desktop: registry `backend_for` mapping (`environment_registry.rs` tests)
- `backend_for_resolves_mainframe_to_record_capable_backend`: a registry built
  with a mainframe CE returns a `record_capable()` backend for `"MAINFRAME"`
  (case-insensitive), and the host_fs (non-record-capable) backend for `"HOSTFS"`
  and for an unknown name (fallback). Validates: command-environments 14.1, 16.4.
- Keep the existing `active_name`/`is_ffedit_active` tests GREEN (MAINFRAME does
  not change active-env derivation).

### T3 -- ff-desktop: end-to-end mainframe SAVE + HOSTFS byte-identical
(`tab_manager.rs` tests and/or a shell-level test in `tests_command.rs`)
- `save_routes_mainframe_tab_to_record_capable_backend`: a tab with
  `owning_environment = "MAINFRAME"`, a non-Delimited `RecordFormat`, and a
  `store_identity`, saved through `save_active_tab_via_backend` with a
  record-capable test backend, drives `save_records` (NOT `save`) and builds a
  `StoreTarget` whose `dsn` is the real DSN (from `store_identity`, not the host
  path) and `owning_env == "MAINFRAME"`. Assert via a capturing test backend
  (mirror the existing `RecordCapableBackend` in `backend_environment.rs` tests).
  Validates: command-environments 18.2; document-model 13.4.
- `save_hostfs_tab_is_byte_identical`: a HOSTFS tab (Delimited format, no
  store_identity) still takes the BYTE path and writes byte-identical content
  (keep/extend the existing `TestBackend` byte-write assertion). Validates:
  document-model 12.12 (native SAVE unchanged).
- A full-shell or `host_fs_save`-level test that `backend_for` is consulted by the
  save seam: a MAINFRAME active tab's SAVE resolves the mainframe backend, a
  HOSTFS active tab's SAVE resolves host_fs. Validates: command-environments
  14.4/14.5.

(If a full end-to-end shell SAVE against a real `CatalogDatasetAccess`-backed CE
is too heavy for a scoped unit test, assert the ROUTING with a capturing backend
in ff-desktop and the ACTUAL DatasetAccess round-trip in T1 -- together they cover
the end-to-end path without a multi-crate fixture in one test.)

## 9. Out-of-scope full-gate risk to call out

- `ff-governance-tests` DAG rules: the NEW `ff-idcams -> ff-vfs` dependency edge
  may be checked by the architecture/governance tests. The edge is acyclic and
  INTO a lower layer (ff-vfs is the storage-abstraction leaf every backend
  depends on), consistent with the CE trait living in ff-vfs by design. If a
  governance test enumerates allowed edges explicitly, it may need the new edge
  added -- FLAG in the hand-off; do NOT weaken a governance rule without noting it.
- The NEW `ff-desktop -> ff-idcams` edge: ff-desktop already depends on
  ff-dscatalog; adding ff-idcams (which depends only on ff-dscatalog + thiserror
  + the new ff-vfs) introduces no cycle. Flag if governance enumerates edges.
- 400-line rule: `environment_registry.rs`, `construct.rs`/`construct_provider.rs`
  are near-ish the limit; split the mainframe construction helper into
  `mainframe_backend.rs` if any file would cross 400 non-test lines.
- `tests_command.rs` pending-open tuple-shape tests WILL break when the payload
  widens (section 6) -- update them in the same change (expected, not a
  regression).

## 10. Verification (scoped; owner runs the full gate)

Per crate touched, via the pwsh7 non-interactive wrapper:
- `cargo test -p ff-idcams` (T1, red-then-green).
- `cargo test -p ff-desktop` (T2, T3; existing host-save + session tests stay
  green -- native SAVE unchanged).
- `cargo clippy -p ff-idcams` and `cargo clippy -p ff-desktop` (`-D warnings`).
- `cargo fmt`.
Then HAND OFF: the owner runs `cargo gate --build` manually and reports back.
Kiro NEVER runs the full gate or `--workspace`.
