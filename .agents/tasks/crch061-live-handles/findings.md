# CR-CH-061 Findings -- Threading live catalog/volume handles into the MAINFRAME CE SAVE path

READ-ONLY investigation. No source modified. All citations are `path:line`
against `main` at the current working tree.

## Summary (the key finding)

The same-session SAVE-resolves-`NotFound` (rc 12) symptom is caused by **two
completely disjoint dataset stores that never meet**:

1. **Files Panel allocate path** -> an **on-disk, SQLite-backed `ff-dscatalog`
   repository**. `ff_catalog_registry::CatalogRegistry::allocate` opens
   `ff_dscatalog::Catalog::mount(path)` and `INSERT`s a `datasets` row into the
   catalog's SQLite DB, creating the physical file under the repository
   (`crates/ff-catalog-registry/src/lib.rs:296-325`;
   `crates/ff-dscatalog/src/catalog.rs:188-266`).

2. **MAINFRAME CE SAVE path** -> an **in-process, in-memory
   `CatalogDatasetAccess`**. `MainframeEnvironment::save_records` calls
   `access.resolve(dsn, Write)`, which looks the DSN up in
   `AccessState.datasets: HashMap<String, DatasetState>` -- a map populated
   ONLY by `allocate_impl`/the in-process API, never from SQLite
   (`crates/ff-idcams/src/mainframe_env.rs:154-170`;
   `crates/ff-dscatalog/src/dataset_access/impl_access.rs:50-55, 182-205`).

The `CatalogDatasetAccess` the shell builds for the CE
(`build_mainframe_backend`) is a **fresh fixture**: a `PosixNativeProvider`
rooted at the home dir plus a single synthetic `FFWB01` volume, with an **empty**
`datasets` map (`crates/ff-desktop/src/shell/mainframe_backend.rs:46-55,
63-78`). A dataset the Files Panel just created exists only as a SQLite row + a
physical file in the repository; the CE's `resolve` never consults SQLite, so it
returns `DatasetError::NotFound`, which `rc_for_error` maps to rc 12
(`crates/ff-idcams/src/mainframe_env.rs:95-101, 157-162`).

**Important correction to the brief's framing.** The gap is NOT that the shell
fails to *share a live in-memory handle* the Files Panel holds. The Files Panel
does **not** hold a live `ff_dscatalog` registry at all -- its
`FilesPanelState.registry` is the TOML wrapper type
`ff_catalog_registry::CatalogRegistry` (`crates/ff-files-panel/src/resolve.rs:11,
24-26`), a plain owned value that opens a *fresh* SQLite `Catalog` per call
(`open_dscatalog`, `crates/ff-catalog-registry/src/lib.rs:277-294`). The durable
shared truth is the **on-disk SQLite repository**. The real fix is therefore to
make the CE's `DatasetAccess` resolve against that **same on-disk catalog**, not
to pass an in-memory handle around. This widens the solution space (see Q4).

## Q1 -- How the Files Panel owns its catalog + volume state today

- The shell field is `files_panel: FilesPanelState`
  (`crates/ff-desktop/src/shell/state.rs:260`), a **plain owned value** -- no
  `Arc`, no `RwLock`, no `Mutex`.
- `FilesPanelState.registry` is `ff_catalog_registry::CatalogRegistry`
  (the TOML wrapper), constructed empty in `FilesPanelState::new`
  (`crates/ff-files-panel/src/resolve.rs:22-26, 48-60`). `crate::catalog_registry`
  in ff-desktop is a thin re-export of this crate
  (`crates/ff-desktop/src/catalog_registry.rs:12`).
- This wrapper holds **no live SQLite connection and no `VolumeRegistry`**. Each
  mainframe operation opens a throwaway `ff_dscatalog::Catalog`:
  `open_dscatalog` -> `Repository::new(path)` (self-heal `initialize` if needed)
  -> `Catalog::mount(path, 1)` (`crates/ff-catalog-registry/src/lib.rs:277-294`).
  `allocate`/`list_datasets`/`resolve_dsn` each do this and drop the catalog
  (`:296-333`).
- `Catalog::mount` opens the SQLite connection (WAL), validates schema, and runs
  the v3->v4 Volume seed `schema_v4::migrate_v3_to_v4`
  (`crates/ff-dscatalog/src/catalog.rs:112-172`). So the **Volume model for
  Files-Panel datasets lives inside the catalog's own SQLite DB**, seeded one
  Volume per repository -- NOT in any `ff_volume::VolumeRegistry` the shell owns.
- The `ff_dscatalog::catalog_registry::CatalogRegistry` (the SQLite,
  priority-ordered, live multi-catalog registry with `resolve -> ResolveResult`,
  `get_catalog`) is a **different type** (`crates/ff-dscatalog/src/catalog_registry.rs:26-30,
  161-190, 228-231`) that the shell does **not** persistently own. It appears
  only as a borrowed argument in `resolve_and_open_dataset` /
  `load_entries_from_catalog` (`crates/ff-files-panel/src/resolve.rs:95-116,
  124-170`).

Net: there is **no live, shell-held, long-lived catalog or volume handle** today.
Both the Files Panel and the CE reconstruct their view each time; only the
on-disk SQLite repository persists across them.

## Q2 -- How `CatalogDatasetAccess` is constructed and what it needs

- Public constructors (`crates/ff-dscatalog/src/dataset_access/impl_access.rs:78-109`):
  - `new(provider: Arc<dyn StorageProvider>, registry: VolumeRegistry) -> Self`
  - `with_vsam_root(provider, registry, vsam_root: PathBuf) -> Self`
  - `in_memory(storage_dir) -> Self` (test/consumer convenience)
  Both primary constructors take the `VolumeRegistry` **by value (owned)** and an
  `Arc<dyn StorageProvider>`. There is **no** constructor accepting a shared
  catalog/SQLite handle or a shared `VolumeRegistry`.
- Internal state (`impl_access.rs:49-73`): `provider: Arc<dyn StorageProvider>`,
  `state: Mutex<AccessState>`, `vsam_root: PathBuf`. `AccessState` is
  `{ registry: VolumeRegistry, datasets: HashMap<String, DatasetState>,
  next_dataset_id: u64 }` (`:50-55`). The `datasets` map **IS** the catalog from
  this type's point of view -- it is the only thing `resolve`/`open`/`close`
  consult.
- `resolve_impl` reads purely from `state.datasets` + `state.registry`
  (`impl_access.rs:182-205`); `open_impl` reads bytes through the provider using
  the locator held in `DatasetState.volumes` (`impl_io.rs:45-74`); `close_impl`
  writes bytes through the provider and re-charges extents against the in-memory
  `VolumeRegistry` (`impl_io.rs:114-176`). **Nothing in this type ever touches
  SQLite or the `ff-dscatalog::Catalog`/`Repository` layer.** It is a
  self-contained volume+provider record store whose catalog is its own HashMap.
- The trait `DatasetAccess` is `Send + Sync` and object-safe
  (`crates/ff-dscatalog/src/dataset_access/trait_def.rs:28, 115-117`). All
  methods take `&self` (interior mutability via the `Mutex<AccessState>`), so a
  shared `Arc<dyn DatasetAccess>` is already the sharing unit
  (`MainframeEnvironment` holds exactly that, `mainframe_env.rs:59-70`).

## Q3 -- Where exactly the gap is

- `build_mainframe_backend` (`crates/ff-desktop/src/shell/mainframe_backend.rs:46-55`)
  builds `CatalogDatasetAccess::new(fresh PosixNativeProvider(home), single
  FFWB01 volume)`. Its `datasets` map starts empty and is only ever filled by an
  in-process `allocate` call -- which the editor-open flow does NOT make against
  THIS access instance.
- A Files-Panel-created dataset lands in the **catalog repository's SQLite DB**
  (`Catalog::allocate` INSERT at `crates/ff-dscatalog/src/catalog.rs:214-240`) and
  a physical file under `repository.storage_dir()` (`catalog.rs:258-266`). Its
  Volume lives as a `schema_v4` row inside that same SQLite DB.
- At SAVE time, `MainframeEnvironment::save_records` calls
  `self.access.resolve(&target.dsn, Write)` (`mainframe_env.rs:157`). That hits
  `CatalogDatasetAccess::resolve_impl`, whose `state.datasets.get(&key)` misses
  (the dataset was never allocated through THIS instance) and returns
  `NotFound` -> rc 12 (`impl_access.rs:187-190`; `mainframe_env.rs:95-101`).
- The fresh CE access and the Files Panel's SQLite catalog are **different
  stores rooted differently** (CE: home dir + synthetic FFWB01 volume; Files
  Panel: the catalog's `path` repository with its own seeded volume). Even the
  physical bytes live in different directory trees, so there is no accidental
  overlap. This is why resolve fails deterministically rather than intermittently.

## Q4 -- Minimal seam options to close the gap (tradeoffs, no decision)

Framing note (from Q1): because the durable shared store is the on-disk SQLite
repository and NOT an in-memory handle, "share the live handle" is only ONE of
the viable shapes, and arguably not the simplest.

### Option (a) -- Shell holds catalog/volume in `Arc<RwLock<...>>`, clones into both paths
Have the shell own one long-lived handle and pass a clone to both the Files
Panel operations and `build_mainframe_backend`.
- Problem 1: today **neither** path holds a long-lived in-memory handle to share
  (Q1). The Files Panel opens throwaway `Catalog`s per call; the CE owns its own
  HashMap. Adopting this means first INTRODUCING a shared long-lived registry
  and rewiring the Files Panel to use it -- a large change beyond CR-CH-061.
- Problem 2: `ff_dscatalog::Catalog` holds a `rusqlite::Connection`
  (`catalog.rs:106-112`), which is **not `Sync`**. A shared
  `Arc<dyn DatasetAccess>` must be `Send + Sync` (trait_def.rs:28). An
  `Arc<RwLock<CatalogRegistry>>` over the SQLite registry is **non-Send/Sync**
  -- exactly the constraint already called out and `#[allow]`-ed in
  `build_catalog_provider` (`mainframe_backend.rs:85-96`). So a SQLite-backed
  shared handle **cannot** satisfy the `DatasetAccess: Send + Sync` bound without
  per-call connection opening or a connection pool.
- Verdict: highest blast radius; fights the `Send + Sync` contract. Not minimal.

### Option (b) -- Add a `CatalogDatasetAccess` constructor accepting shared live handles
E.g. `CatalogDatasetAccess::with_shared(provider, Arc<RwLock<VolumeRegistry>>)`
or pre-seeded `datasets`.
- Still does not help the real miss: the CE's `datasets` HashMap would remain
  empty unless something replays every Files-Panel allocation into it. The
  catalog of record is SQLite, which this type never reads.
- Would require a public API change to `CatalogDatasetAccess` and to
  `AccessState` (widening the gate; see Q6), and still leaves the two-store
  divergence unless combined with option (c)'s SQLite read.
- Verdict: public API churn without solving resolution; only useful as a
  sub-part of (c).

### Option (c) -- Make the CE's `DatasetAccess` resolve through the SAME on-disk catalog the Files Panel writes
Give the MAINFRAME CE a `DatasetAccess` implementation (new type, or an enriched
`CatalogDatasetAccess`) whose `resolve`/`open`/`close` consult the **on-disk
`ff_dscatalog` repository** (SQLite + physical files) that the Files Panel's
`allocate` already populates -- i.e. resolve the DSN to its `physical_path`
through `ff_dscatalog::catalog_registry::CatalogRegistry::resolve` /
`Catalog::physical_path` (`catalog_registry.rs:161-190`;
`catalog.rs:579-583`), then read/write the physical file via the existing
`StorageProvider` seam.
- This matches the durable truth: SQLite round-trips across both paths today
  (proven by `catalog_registry_allocate_writes_to_sqlite` and the Files Panel's
  own `resolve_and_open_dataset`, `crates/ff-files-panel/src/resolve.rs:95-116`).
- `Send + Sync`: a `rusqlite::Connection` is not `Sync`; the CE's
  `DatasetAccess` must stay `Send + Sync`. Resolve the catalog `path` to the
  physical file **per call** (open a short-lived `Catalog`, look up, drop) rather
  than holding a live connection -- exactly what the Files Panel already does
  (`open_dscatalog`). The access type then holds only the catalog `path`(s) +
  the `StorageProvider` + volume accounting, all `Send + Sync`.
- The CE needs to know WHICH catalog/repository path backs the DSN. The
  editor-open flow already carries `(dsn, catalog)` identity into the tab's
  `StoreTarget` (`PendingOpenReq.identity`, `crates/ff-desktop/src/shell/state.rs:56-64`;
  `tab_state.rs:208-212`). The shell can resolve `catalog -> repository path` via
  `files_panel.registry.get_by_name(catalog).path`
  (`crates/ff-catalog-registry/src/lib.rs:245-247`) and hand that path to the CE
  at build time (or thread it per-save).
- Verdict: lowest-divergence, matches the existing persistence model, keeps
  `Send + Sync`. Main cost: a new/extended `DatasetAccess` that bridges to the
  `ff_dscatalog::Catalog` physical path, and deciding how the catalog path(s)
  reach the CE. Likely a public `ff-dscatalog` addition (a catalog-backed
  `DatasetAccess`), which widens the gate (Q6).

### Option (d) -- (noted, not in brief) Thread the catalog repository root into `build_mainframe_backend`
Narrowest variant of (c): root the CE's `PosixNativeProvider` at the catalog
repository `storage` dir and resolve DSNs through a short-lived `Catalog`
lookup, so the CE reads/writes the SAME physical files the Files Panel created.
- Pro: smallest change if a single active mainframe catalog can be assumed.
- Con: the current CE is built ONCE at shell construction
  (`construct.rs` env registry init) with a fixed home-dir root, before any
  catalog is known; multiple mainframe catalogs each have their own repository,
  so a single fixed root cannot serve all. Needs per-save catalog resolution
  anyway, converging on (c).

Constraint cross-check for every option:
- `DatasetAccess: Send + Sync` (trait_def.rs:28) rules out holding a live
  `rusqlite::Connection` or a non-Send/Sync `Arc<RwLock<CatalogRegistry>>` inside
  the shared `Arc<dyn DatasetAccess>` -- forces per-call catalog opening in (a)/(c).
- Object-safety (trait_def.rs:115-117) is preserved by all options (they add
  constructors/impls, not generic trait methods).

## Q5 -- Constraints to respect, and which options risk them

- **Native/host SAVE must stay byte-identical.** All options touch ONLY the
  MAINFRAME backend resolution; the byte `save` path and the host backend
  (`ff-ce-ntfs`/`ff-ce-posix`) are untouched. `host_fs_save` already selects the
  backend by owning-env and only MAINFRAME tabs reach the CE
  (`crates/ff-desktop/src/shell/dispatch_ffedit.rs:51-80`;
  `crates/ff-desktop/src/shell/commands_environment.rs:72-78`). SAFE for all
  options provided the host arm is left alone.
- **SAVE keeps riding the single CR-CH-053 Task 20/21 seam (no new dispatcher).**
  The fix changes only what the CE's `DatasetAccess.resolve/open/close` DO; it
  adds no dispatch path. `save_active_tab_via_backend` ->
  `backend.save_records` -> `access` stays the one seam
  (`crates/ff-desktop/src/tab_manager.rs:1297-1322`;
  `mainframe_env.rs:148-175`). SAFE for all options.
- **Crate DAG must stay acyclic (ff-idcams -> ff-dscatalog -> ff-volume ->
  ff-vfs; ff-desktop on top).** `ff-idcams` already depends on `ff-dscatalog`
  (it imports `DatasetAccess`, `mainframe_env.rs:47-52`). Option (c)/(d) add a
  catalog-backed `DatasetAccess` INSIDE `ff-dscatalog` (which already depends on
  `ff-vfs`/`ff-volume`/`rusqlite`, `crates/ff-dscatalog/Cargo.toml:11-24`), so no
  new edge and no cycle. Threading the catalog path from the shell stays in
  `ff-desktop` (top of the DAG). SAFE. A design that made `ff-dscatalog` depend
  on `ff-catalog-registry` or `ff-files-panel` WOULD invert the DAG -- AVOID; use
  the `ff_dscatalog::catalog_registry`/`Catalog` types directly instead.
- **400-non-test-line-per-file rule.** `impl_access.rs` is already ~330 lines;
  adding a shared-handle constructor + a catalog-backed resolve risks the limit
  -- a new sibling file (e.g. `dataset_access/impl_catalog.rs`) is the clean
  split. `mainframe_backend.rs` is small and has headroom. FLAG option (b)/(c)
  if they grow `impl_access.rs` past 400.
- **ASCII-only `.rs`.** No option inherently introduces non-ASCII; standard care
  applies.

## Q6 -- Exact files/functions a fix would touch + public-API impact

Shell-side (ff-desktop, top of DAG -- no API widening):
- `crates/ff-desktop/src/shell/mainframe_backend.rs`:`build_mainframe_backend`
  (lines 46-55) -- construct the CE's access against the real catalog(s) instead
  of the fresh fixture; and `mainframe_root` (`construct.rs:290-296`) if rooting
  changes.
- `crates/ff-desktop/src/shell/construct.rs` -- the `environments:` init
  (lines ~209-215) that calls `build_mainframe_backend`, if the catalog path /
  handle must be threaded in at build time.
- Possibly `crates/ff-desktop/src/shell/dispatch_ffedit.rs:host_fs_save`
  (51-80) and `crates/ff-desktop/src/tab_manager.rs:save_active_tab_via_backend`
  (1297+) IF the catalog path must be resolved per-save from the active tab's
  identity rather than fixed at build time (the per-save variant of option c).
- Catalog-path lookup uses existing API `ff_catalog_registry::CatalogRegistry::get_by_name`
  (`crates/ff-catalog-registry/src/lib.rs:245-247`) -- no change.

ff-dscatalog (library -- **public API change, widens the gate**):
- A **new** catalog-backed `DatasetAccess` (recommended as a new file
  `crates/ff-dscatalog/src/dataset_access/impl_catalog.rs` to respect the 400-line
  rule), OR new public constructor(s) on `CatalogDatasetAccess`
  (`impl_access.rs:78-109`) that accept the catalog repository path(s) and resolve
  via `ff_dscatalog::catalog_registry::CatalogRegistry::resolve` /
  `Catalog::physical_path`. Either way this is a **new public surface in
  ff-dscatalog** -> the gate scope includes a dataset-catalog requirement/criterion.
- No change needed to the `DatasetAccess` trait itself (trait_def.rs) -- it stays
  object-safe and `Send + Sync`; the new type implements the existing trait.

ff-idcams (library -- **no API change expected**):
- `MainframeEnvironment` already consumes `Arc<dyn DatasetAccess>`
  (`mainframe_env.rs:59-70`); it is agnostic to which concrete access it wraps.
  The resolve-only policy (`:154-162`) still holds; a catalog-backed access makes
  `resolve` SUCCEED for Files-Panel datasets, turning the rc-12 miss into rc 0.
  No signature change.

ff-volume / ff-vfs:
- **No public API change required.** The catalog-backed access reuses the
  existing `StorageProvider` seam and the catalog's own schema-v4 Volume rows;
  it does not need a new `VolumeRegistry` surface. (If option b's shared
  `VolumeRegistry` were pursued it WOULD need an ff-volume API change -- another
  reason to prefer option c.)

## Recommendation

Pursue **Option (c)** -- a catalog-backed `DatasetAccess` in `ff-dscatalog` that
resolves a DSN through the on-disk SQLite catalog the Files Panel already writes
(opening a short-lived `Catalog` per operation, mirroring the Files Panel's
`open_dscatalog`), with the shell threading the active mainframe catalog's
repository path (from `files_panel.registry.get_by_name(catalog).path`, available
via the tab's existing `(dsn, catalog)` store identity) into the CE.

Rationale:
- It targets the ACTUAL store of record (SQLite + physical files), so a
  same-session SAVE resolves the exact dataset the user sees in the Files Panel
  -- which neither (a) nor (b) achieves on its own.
- It respects `DatasetAccess: Send + Sync` by opening the SQLite connection
  per-call (no live non-Send/Sync handle held in the shared `Arc`), exactly the
  pattern already proven in `ff_catalog_registry::open_dscatalog` and the Files
  Panel's `resolve_and_open_dataset`.
- It keeps the single Task 20/21 SAVE seam, leaves native/host byte SAVE
  byte-identical, and adds no DAG edge (`ff-dscatalog` already owns the catalog +
  volume + vfs layers).

Gate-scope implication to flag to the owner: Option (c) adds **public API to
`ff-dscatalog`** (a new catalog-backed `DatasetAccess` type / constructor). That
is a deliberate, gated library addition under CR-CH-061, not an incidental
change -- the requirements gate should include a dataset-catalog (or
command-environments Req 16/18) criterion for "the MAINFRAME CE resolves and
stores a Files-Panel-allocated dataset in the same session (rc 0)", plus an
`ff-idcams` round-trip test against a catalog-backed access and a scoped
ff-desktop routing test. Also plan a file split
(`dataset_access/impl_catalog.rs`) to stay under the 400-line rule.
