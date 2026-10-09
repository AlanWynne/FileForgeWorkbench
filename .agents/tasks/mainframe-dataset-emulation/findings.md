# Mainframe Dataset Emulation -- Investigation Findings

READ-ONLY investigation of how a mainframe dataset emulation plugin would
integrate with FileForgeWorkbench (FFWB). Workspace:
`C:\workspace\VSC\FileForgeWorkbench`, branch `main`, single worktree.

---

## Summary answer (read this first)

The headline finding is that **most of what the owner is proposing already
exists in the FFWB workspace as real, tested Rust code.** FFWB is not at the
"should we use SQLite / should we build record support" stage -- it has already:

- A **VFS abstraction** (`ff-vfs`) with a `VfsProvider` trait, a provider
  registry, URI routing (`vfs://catalog/DSN`), and -- below it -- a second
  `StorageProvider` trait layer explicitly designed so new physical backends
  drop in without touching VFS routing.
- A **SQLite-backed dataset catalog** (`ff-dscatalog`) using `rusqlite`
  (bundled SQLite) with PS/PO(PDS/PDSE)/GDG support, DSN validation, RECFM /
  LRECL / BLKSIZE / DSORG attributes, repository layout, and a `catalog`
  VfsProvider.
- **Record-oriented storage already built**: record codecs (`FixedCodec`,
  `VariableCodec` with 4-byte RDW, `BinaryCodec`, `TextCodec`) that explicitly
  forbid CRLF/LF as record delimiters -- exactly the mainframe record semantics
  the owner describes.
- **VSAM storage backends already present** in `ff-dscatalog/src/storage/`:
  `SqliteRecordProvider` (KSDS, keyed, with alternate indexes), `NativeEsdsProvider`
  (ESDS, append-only native file + rebuildable sidecar index),
  `SqliteRrdsProvider` (RRDS), and `IsamProvider`.
- A **plugin architecture** (`ff-plugin`) with a `FileForgePlugin` trait,
  `PluginContext`, capability registry, and -- critically -- a `Providers`
  capability variant intended for VFS/data-source providers.

**Recommendation on the three big questions:**

1. **TAR-as-catalog vs ZIP-migration vs SQLite-per-file**: FFWB has already
   decided this and codified it as requirements. The adopted design is a
   **hybrid**: SQLite stores catalog *metadata only* (never dataset payload
   BLOBs), PS/PDS/GDG content lives as *native files* (UUID-named), and only
   *keyed/relative* VSAM data (KSDS/RRDS) lives in dedicated per-dataset SQLite
   record stores. ESDS uses an append-only native file. This is Requirement 18
   ("Hybrid Storage Architecture") and it explicitly *prohibits* storing
   PS/PDS/GDG/POSIX content as SQLite BLOBs. The owner's "SQLite file stores
   each file internally" idea is already the design *for the VSAM keyed cases*,
   and already rejected for the flat cases (for Git/backup/tooling compatibility).
   Offline migration via ZIP already exists as `catalog.export` / `catalog.import`
   (Requirement 6).

2. **Separate repo vs worktree, and building it as an FFWB plugin**: A fully
   separate repository would be the wrong move -- it would have to duplicate
   `ff-vfs` (the `VfsProvider` + `StorageProvider` traits), `ff-dscatalog`'s
   dataset/DSN/codec types, `ff-vsam-services` traits, and `ff-plugin`. These
   are workspace-internal path crates with no published versions. The emulation
   work is *continuation of existing crates*, not a greenfield project. If
   isolation from day-to-day FFWB churn is the goal, a **git worktree off a
   feature branch** (keeping the same workspace `Cargo.toml` and shared crates)
   is the right tool. The "plugin" framing is already satisfied architecturally:
   the catalog registers as a VFS provider and advertises a `Providers`
   capability, which is FFWB's plugin seam.

3. **Use Turso's Rust rewrite of SQLite?** Not now. FFWB already depends on
   `rusqlite` 0.31 with bundled SQLite (battle-tested C SQLite). Turso (the
   project formerly called Limbo; the `turso` / `turso_sync_engine` crates, and
   the older `libsql` fork) is a ground-up Rust rewrite that is **still in beta**
   per its own docs. Swapping the storage engine is orthogonal to building VSAM
   emulation and would add risk to the exact layer (keyed record integrity) where
   correctness matters most. Keep `rusqlite`; the `StorageProvider` trait already
   isolates the engine choice so Turso can be evaluated later behind that seam
   without disturbing consumers.

The practical conclusion: the next step is almost certainly **finishing and
wiring the VSAM services** (`ff-vsam-services` is currently only traits + a
`StubVsamService`) to the already-built storage backends in
`ff-dscatalog/src/storage/`, not standing up a new project.

---

## Evidence

### 1. Existing plugin architecture (`ff-plugin`, `docs/specs/plugin-architecture/`)

Source files: `crates/ff-plugin/src/{traits,capability,context,registry/*,loader,lifecycle,capability_registry}.rs`.

- **Primary trait** `FileForgePlugin` (`traits.rs`): object-safe
  (`Box<dyn FileForgePlugin>`), `Send + Sync`, lifecycle methods
  `initialize(Arc<PluginContext>)`, `activate`, `deactivate`, `shutdown`, all
  returning `Result<(), PluginError>`, plus `metadata()`, `plugin_capabilities()`,
  and `supports_hot_reload()`.
- **Loading** (per `docs/specs/plugin-architecture/requirements.md` Req 3): the
  `Plugin_Registry` scans a `Plugin_Directory`, resolves a `Dependency_Graph`
  topologically, and runs discover -> load -> validate -> initialize -> activate.
  Panics are caught with `catch_unwind` (Req 5.3) so a plugin cannot crash the
  host. API-version compat is checked against `PLUGIN_API_VERSION` (Req 6).
- **Can a plugin provide a new storage backend? Yes.** The `Capability` enum
  (`capability.rs`) includes `Providers(ProvidersCapability)` whose
  `provider_type` is a free string documented with the example `"vfs"`. Service
  traits in `traits.rs` include `PluginVfsAccess` (read/write/exists/list through
  VFS URIs). So a plugin's path to adding storage is: advertise a `Providers`
  capability and register a `VfsProvider` with the VFS registry. This matches
  the dataset-ownership rule (below) that domain crates integrate by *implementing*
  `VfsProvider`, not by VFS importing them.
- Note: the plugin model is **in-process, trait-based, same-address-space**
  (Req 7.1) -- not a dynamically-loaded `.so`/`.dll` ABI. The
  `design.md` shows a `libsql_connector.so` only as an illustrative directory
  sketch; there is no stable C-ABI plugin loader. In practice today, "plugins"
  like the catalog are compiled-in crates wired through `ff-desktop`.

### 2. Current VFS and dataset model

**VFS trait** (`crates/ff-vfs/src/provider.rs`): `VfsProvider` is `async_trait`,
`Send + Sync`, object-safe, with `scheme()`, `capabilities() -> VfsCapabilities`,
and async `open/read/read_stream/write/create/delete/rename/list/stat/exists`
plus default-`UnsupportedOperation` `watch`/`search`. URI model
(`virtual-file-system/requirements.md` Req 2): `vfs://provider/path`, with the
catalog provider using `vfs://catalog/HLQ.QUALIFIER.NAME` and query params like
`?recfm=fb`.

**Two-layer provider design.** There is a deliberate split:
- `ff-vfs::VfsProvider` -- the routing/abstraction layer (what consumers call).
- `ff-vfs::storage_provider::StorageProvider` (`crates/ff-vfs/src/storage_provider.rs`)
  -- a lower physical-storage trait with a richer `StorageCapability` set
  (`StreamRead/Write`, `RecordRead/Write`, `KeyedAccess`, `RelativeAccess`,
  `AppendOnly`, `MemberOperations`, `AtomicRename`, `Locking`, `Snapshotting`,
  `WatchNotifications`) and an opaque `StorageLocator`. This exists specifically
  so "new backends can be added without touching the VFS API"
  (`virtual-file-system/requirements.md` Req 9, 19).

**A second `StorageProvider` lives inside `ff-dscatalog`** (`src/storage/mod.rs`):
a catalog-local trait (`allocate/open/stat/rename/delete/list/reconcile`,
`ObjectId = uuid::Uuid`, `ProviderCapability`). This is the one the concrete
VSAM backends implement. (Observation: there are now *two* `StorageProvider`
traits -- one in `ff-vfs`, one in `ff-dscatalog`. See "Observations" below.)

**Dataset attributes** are first-class (`crates/ff-dscatalog/src/dataset.rs`):
`Dsorg {PS, PO, GDG}`, `Recfm {F, FB, V, VB, U}`, `PartitionedSubtype {PDS, PDSE}`,
`AllocParams` (dsn, dsorg, recfm, lrecl, blksize, dir_blocks, gdg_limit,
gdg_scratch, subtype, scope) with `validate()` (LRECL 1..=32760, BLKSIZE >= LRECL,
GDG limit) and `with_defaults()` (RECFM=FB, LRECL=80, BLKSIZE=27920).
`DatasetRecord` is the stored row. A *separate* attribute model also exists in
`ff-dataset-catalog/src/traits.rs` (`DatasetAttributes`, `Dsorg {Ps,Po,Da,Vsam}`,
`Recfm`) -- see "Two catalog crates" below.

**Catalog DB schema** (`dataset-catalog/requirements.md` Req 1): `catalog.db`
SQLite, WAL mode, tables `datasets`, `gdg_bases`, `gdg_generations`,
`catalog_metadata`; schema v4 adds `volumes` + `dataset_volumes`
(CR-CH-057 Volume split). Parameterised queries are mandated (Req 1.9; enforced
note in `lib.rs`).

### 3. Record-oriented storage and VSAM -- already implemented

This directly answers the owner's core concern (fixed-length vs
length-prefixed variable records).

- **Codecs** (`crates/ff-dscatalog/src/codecs/`): `RecordCodec` trait with
  `FixedCodec` (packs N x LRECL, pads with 0x40), `VariableCodec` (4-byte RDW
  per `virtual-file-system/requirements.md` Req 16.3), `BinaryCodec` (RECFM=U
  passthrough), `TextCodec` (import/export only). Codecs have **no** SQLite /
  fs / egui dependency (Req 17.1). `requirements.md` Req 16.1 explicitly forbids
  CRLF/LF as a record boundary.
- **VSAM backends** (`crates/ff-dscatalog/src/storage/`):
  - `sqlite_record.rs` -> `SqliteRecordProvider` for **KSDS**: per-dataset
    SQLite DB under `indexed/<uuid>.sqlite`, `KsdsKeyDefinition`
    (offset/length/type/collation/unique), `AlternateIndex` support
    (`virtual-file-system`/`dataset-catalog` Req 21.5), key collation
    BINARY/NOCASE.
  - `esds.rs` -> `NativeEsdsProvider` for **ESDS**: append-only `.esds` native
    file (`datasets/objects/<uuid>.esds`), byte-offset = stable record address,
    updates append replacement frames, deletes append tombstones, sidecar index
    rebuildable from data (matches `requirements.md` Req 18.6).
  - `rrds.rs` -> `SqliteRrdsProvider` for **RRDS**: per-dataset SQLite under
    `relative/<uuid>.sqlite`, slot-based.
  - `isam.rs` -> `IsamProvider`.
  - `native.rs` -> `NativeFileProvider` for PS/PDS/PDSE/GDG/POSIX
    (`requirements.md` Req 19.5).
- **VSAM service layer** (`crates/ff-vsam-services/src/traits.rs`): a complete
  `VsamService` trait (create_ksds/esds/rrds/lds, destroy, open/get/put/delete/
  close, browse start/next/end, alternate index define/build) with `VsamType
  {Ksds, Esds, Rrds, Lds}`, `VsamParams`, `AccessMode`, `BrowseDirection`,
  `KeyField`. **BUT** the only implementation shipped is `StubVsamService`,
  which returns `VsamError::NotImplemented` for every method. The crate has no
  dependency on `ff-dscatalog` -- so the trait layer and the concrete storage
  backends are **not yet wired together**. This is the clearest gap and the most
  likely "real work" target.

Search confirmation: `grep` for `VSAM/KSDS/ESDS/RRDS/Recfm/RECFM` returns dense
hits across `ff-dscatalog/src/storage/*` and `ff-vsam-services`, plus
`idcams-emulator` / `dataset-ownership-model` specs.

### 4. Crate dependency graph and how `ff-desktop` pulls it in

Root `Cargo.toml` is one Cargo workspace (resolver 2) with ~85 member crates.
Relevant members: `ff-vfs`, `ff-dscatalog`, `ff-dataset-catalog`,
`ff-vsam-services`, `ff-dsalloc`, `ff-idcams`, `ff-plugin`,
`ff-catalog-registry`, `ff-catalog-dialog`, `ff-dataset-alloc-dialog`,
`ff-posix-provider`, `ff-governance-tests`, `ff-desktop`.

Pure-model/logic crates (no egui): `ff-vfs`, `ff-dscatalog`, `ff-dataset-catalog`,
`ff-vsam-services`, `ff-dsalloc`, `ff-idcams`, `ff-plugin`. UI crates:
`ff-desktop`, `ff-catalog-dialog`, `ff-dataset-alloc-dialog`, `ff-explorer-view`,
`ff-files-panel`.

`ff-dscatalog/Cargo.toml` depends on `ff-vfs`, `ff-command`, `ff-config`,
`ff-logging`, and `rusqlite 0.31` (bundled), `tokio`, `async-trait`, `uuid`,
`zip`, `walkdir`, `percent-encoding`.

`ff-desktop/Cargo.toml` pulls the stack in by path: `ff-vfs`, `ff-dscatalog`,
`ff-plugin`, `ff-catalog-registry`, `ff-catalog-dialog`,
`ff-dataset-alloc-dialog`, `ff-posix-provider`, `ff-files-panel`,
`ff-explorer-view`. It does **not** directly depend on `ff-vsam-services`,
`ff-dataset-catalog`, `ff-dsalloc`, or `ff-idcams` -- those are reached (or
not yet reached) transitively. A new storage/VSAM capability therefore needs no
new interface shape at the `ff-desktop` boundary: it integrates by implementing
`VfsProvider` / the catalog `StorageProvider` and registering through the
existing catalog provider that `ff-desktop` already wires in.

Dependents of the catalog crates (from `Cargo.toml` grep):
`ff-dscatalog` <- `ff-catalog-registry`, `ff-catalog-dialog`, `ff-files-panel`,
`ff-desktop`. `ff-dataset-catalog` and `ff-vsam-services` are currently only
consumed by `ff-governance-tests` (dev-dependency) -- i.e. the trait-only crates
are governance-tested but not yet in the shipping dependency closure of `ffwb`.

### 5. Separate project vs worktree

- Git state: single worktree at the workspace root on branch `main`
  (`git worktree list` -> one entry; `git branch --show-current` -> `main`).
- The emulation feature's dependencies (`ff-vfs` traits, `ff-dscatalog` types,
  `ff-vsam-services` traits, `ff-plugin`) are **path dependencies inside this
  workspace**, versioned `0.1.0`, unpublished. A separate repo would have to
  either vendor/duplicate them (guaranteed drift -- the governance model in
  `dataset-ownership-model` exists precisely to stop duplicate implementations)
  or publish them to a registry (not set up).
- A **worktree off a feature branch** shares the identical workspace
  `Cargo.toml` and all shared crates while isolating commits on a branch, which
  is the standard FFWB pattern. This directly satisfies the owner's
  "does not impact heavily on FFWB project work" goal without the duplication
  cost of a separate repo.
- The "plugin" deployment framing is already met by the capability/provider
  model (Section 1). No new plugin mechanism is required to "deploy" the
  emulation -- it is a VFS provider advertising a `Providers` capability.

### 6. Turso / libSQL assessment (web, cited)

- Turso's own posts describe the project (formerly "Limbo") as a *complete
  ground-up rewrite of SQLite in Rust*, currently in **beta**
  ([turso.tech intro](https://turso.tech/blog/introducing-limbo-a-complete-rewrite-of-sqlite-in-rust),
  [libsql README](https://github.com/libsql/libsql/blob/main/README.md),
  [Turso Rust quickstart](https://docs.turso.tech/sdk/rust/quickstart)).
  `libSQL` is the earlier *fork* of SQLite; Turso has since stated it is going
  "all-in" on the rewrite and that independent libSQL-as-SQLite-replacement usage
  stayed low
  ([turso.tech](https://turso.tech/blog/we-will-rewrite-sqlite-and-we-are-going-all-in)).
  (Content was rephrased for compliance with licensing restrictions.)
- FFWB currently uses `rusqlite 0.31` with the `bundled` feature (upstream C
  SQLite), confirmed in `Cargo.lock` (`libsqlite3-sys 0.28.0`).
- Verdict: a beta engine is a poor fit for the integrity-critical keyed-record
  layer, and the engine choice is already abstracted behind `StorageProvider`,
  so there is no need to adopt it now. It can be trialled later behind that trait
  without changing catalog consumers.

---

## Observations worth surfacing (potential cleanup / clarification)

These are not blockers but affect how the next phase should be scoped:

1. **Two catalog crates coexist.** `ff-dscatalog` is the full implementation
   (SQLite, codecs, storage backends, VFS provider). `ff-dataset-catalog` is a
   trait-only crate (`CatalogService` / `DynCatalogService`,
   `DatasetAttributes`, `Dsorg {Ps,Po,Da,Vsam}`). They use *different* `Dsorg`
   and `Recfm` type definitions. The `dataset-catalog/requirements.md` header
   says the real crate is `ff-dscatalog` and any `ff-dataset-catalog` reference
   is "incorrect". Before VSAM wiring, confirm which crate is authoritative for
   the service trait so the VSAM implementation targets the right boundary.
2. **Two `StorageProvider` traits** exist (`ff-vfs::storage_provider` vs
   `ff-dscatalog::storage`). The concrete VSAM backends implement the
   `ff-dscatalog` one. Clarify whether the `ff-vfs` one is the intended public
   seam (per Volume-model Req 1.4 and `virtual-file-system` Req 9/19) and whether
   the catalog-local trait should ultimately implement/forward to it.
3. **`ff-vsam-services` is stubbed and disconnected.** It has no path dependency
   on `ff-dscatalog`, so its `VsamService` trait and the real KSDS/ESDS/RRDS
   providers cannot currently meet. Wiring these together (a concrete
   `VsamService` impl backed by `SqliteRecordProvider` / `NativeEsdsProvider` /
   `SqliteRrdsProvider`) is the obvious high-value, self-contained next task and
   fits the worktree-off-FFWB approach cleanly.
4. **Volume split (CR-CH-057 / CR-NR-105)** is in flight: a planned `ff-volume`
   crate (per `docs/specs/volume-model/requirements.md`) moves physical ownership
   from catalog to Volume, schema v4 adds `volumes`/`dataset_volumes`. Any new
   storage work should align with this direction (locator via DatasetVolume, not
   raw `storage_path`).

---

## Conclusions and recommendations

1. **Do not start a greenfield project.** The emulation is ~70-80% scaffolded in
   `ff-vfs` + `ff-dscatalog` + `ff-vsam-services`. Treat this as *completion and
   wiring*, governed by the existing specs and the dataset-ownership model.
2. **Use a git worktree off a feature branch**, not a separate repo. It gives
   the requested isolation from mainline FFWB churn while keeping the shared
   workspace crates (no duplication, no drift against the single-authority
   governance).
3. **Keep `rusqlite`/bundled SQLite.** Defer Turso/libSQL -- it is beta and the
   `StorageProvider` seam already lets it be swapped later without consumer churn.
4. **Adopt the already-decided hybrid storage model** (Req 18): metadata in
   `catalog.db`, PS/PDS/GDG as native UUID-named files, KSDS/RRDS in per-dataset
   SQLite record stores, ESDS as append-only native file. The owner's
   "SQLite-per-file" instinct is right *for keyed VSAM* and already built; it is
   deliberately *not* used for flat datasets (Git/backup/tooling access).
5. **Most impactful next task (gate-worthy):** implement a concrete
   `VsamService` in/for `ff-vsam-services` (or a bridge crate) backed by the
   existing `ff-dscatalog` storage providers, resolve the "two catalog crates"
   and "two StorageProvider traits" ambiguities first, and align with the
   `ff-volume` locator direction. This is a clean, isolatable unit of work
   well-suited to a worktree.

Per the workflow gate: implementing any of the above (new behaviour) requires
running the requirements gate (requirements.md -> design.md -> tasks.md -> master
-> TCR -> confirm) before touching source. This report implements nothing; it is
diagnosis and recommendation only.

---

## Key file reference

| Concern | Path |
|---|---|
| VFS provider trait | `crates/ff-vfs/src/provider.rs` |
| VFS physical storage trait | `crates/ff-vfs/src/storage_provider.rs` |
| Catalog impl crate | `crates/ff-dscatalog/src/lib.rs` (+ `dataset.rs`, `catalog.rs`, `schema.rs`, `vfs_provider.rs`) |
| Record codecs | `crates/ff-dscatalog/src/codecs/{mod,fixed,variable,binary,text}.rs` |
| VSAM/record storage backends | `crates/ff-dscatalog/src/storage/{mod,sqlite_record,esds,rrds,isam,native}.rs` |
| VSAM service trait (stub only) | `crates/ff-vsam-services/src/traits.rs` |
| Catalog service trait (separate crate) | `crates/ff-dataset-catalog/src/traits.rs` |
| Plugin trait + capabilities | `crates/ff-plugin/src/{traits,capability}.rs` |
| Ownership governance (ADR-001) | `docs/specs/dataset-ownership-model/requirements.md` |
| Hybrid storage + record reqs | `docs/specs/virtual-file-system/requirements.md` (Req 9-20) |
| Catalog reqs | `docs/specs/dataset-catalog/requirements.md` |
| Volume split direction | `docs/specs/volume-model/requirements.md` |
| ff-desktop wiring | `crates/ff-desktop/Cargo.toml` |

Sources: web search results cited inline above (turso.tech, github.com/libsql,
docs.turso.tech). Content was rephrased for compliance with licensing restrictions.
