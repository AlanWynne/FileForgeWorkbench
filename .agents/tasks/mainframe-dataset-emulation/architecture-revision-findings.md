# Architecture Revision Findings -- VFS and Dataset-Catalog Layering

READ-ONLY architectural investigation to inform a possible whole-architecture
revision of the FileForgeWorkbench (FFWB) VFS + dataset-catalog layering.
Workspace: `C:\workspace\VSC\FileForgeWorkbench`. This report changes no source;
it is diagnosis and design-input only. It builds on the prior
`findings.md` in this folder (which concluded "most of the emulation already
exists; wire VSAM, do not greenfield"). This report answers the deeper
"should we revise the whole architecture" question the owner raised.

---

## Summary answer (read this first)

The owner's target mental model -- `ff-vfs` as THE Virtual File System, sitting
over three provider kinds (Native host files, POSIX catalogs, Mainframe dataset
catalogs) -- is ALREADY the documented and largely-built architecture. The
`virtual-catalog-manager` spec names exactly those three kinds with exactly those
VFS schemes (`local` / `posix` / `catalog`). The two-layer seam the owner wants
(`ff-vfs` routing on top, a physical `StorageProvider` below) ALSO already exists
in `ff-vfs`. So the revision is NOT a rewrite of the architecture -- it is a
CONSOLIDATION of duplicate, drifted implementations of that architecture.

The investigation found the real problem is DUPLICATION AND DRIFT, not a missing
design. Concretely:

1. **Two catalog crates with different type systems.** `ff-dscatalog` (the full
   SQLite implementation, ~24 modules) and `ff-dataset-catalog` (a single
   trait-only file) define DIFFERENT `Dsorg` and `Recfm` enums. `ff-dscatalog` is
   the one every real consumer uses; `ff-dataset-catalog` is referenced by NO
   shipping crate -- only by `ff-governance-tests` (dev-dependency). The
   `dataset-catalog` spec explicitly says `ff-dataset-catalog` is "incorrect" and
   should be read as `ff-dscatalog`, yet the GOVERNANCE spec
   (`dataset-ownership-model`, ADR-001) still names `ff-dataset-catalog` as the
   owner. The specs contradict each other.

2. **TWO `StorageProvider` traits with different shapes** -- one in
   `ff-vfs::storage_provider` (the intended public physical seam, UUID-free,
   `StorageLocator`-based, HashSet capabilities) and one in
   `ff-dscatalog::storage` (UUID `ObjectId`, `&[ProviderCapability]`,
   `workspace_root`-threaded). The real VSAM/record backends implement the
   `ff-dscatalog` one; `PosixNativeProvider` implements the `ff-vfs` one. The two
   never meet.

3. **THREE catalog-service trait definitions and TWO VSAM-service traits.**
   `ff-dataset-catalog::CatalogService`, `ff-idcams::services::CatalogService`,
   and `ff-dscatalog`'s concrete `Catalog`/`CatalogRegistry` are three different
   surfaces for "the catalog". `ff-vsam-services::VsamService` and
   `ff-idcams::services::VsamService` are two different VSAM trait shapes. None of
   the dedicated trait crates (`ff-dataset-catalog`, `ff-vsam-services`) is in the
   shipping dependency closure -- `ff-idcams` carries its own copies.

4. **THREE "posix"/native providers.** `ff-connector-local-fs::LocalFsProvider`
   (scheme `local`), `ff-posix-provider::PosixProvider` (scheme `posix`, wraps
   LocalFs), and `ff-vfs::PosixNativeProvider` (scheme `posix`, standalone). Two
   different crates both register scheme `posix`.

5. **A FOURTH, UI-side catalog model.** `ff-desktop`'s own `catalog_registry`
   module defines `VirtualCatalog` + `CatalogType {Mainframe, Posix, Native}` --
   this is the model actually wired into the running app, and it maps 1:1 to the
   owner's three kinds. The lower `ff-vfs`/`ff-dscatalog` provider stack is built
   and unit-tested but is NOT registered into a live `ProviderRegistry` at startup
   (`ff-desktop/src/main.rs` registers config schema but no VFS providers).

**Recommendation (detail in section 7):** Do a CONSOLIDATION, not a rewrite.
Keep `ff-vfs` as THE abstraction with its existing two traits (`VfsProvider` +
`ff-vfs::StorageProvider`). Make `ff-vfs::StorageProvider` the SINGLE physical
seam. Keep `ff-dscatalog` as the mainframe catalog + record-codec + VSAM home,
but have its backends implement the `ff-vfs::StorageProvider` trait (delete the
duplicate `ff-dscatalog::storage::StorageProvider`). DELETE `ff-dataset-catalog`
and `ff-vsam-services` as separate crates (fold their trait role into
`ff-dscatalog`'s public API), and fix ADR-001 to name `ff-dscatalog`. Collapse
the three native/posix providers to ONE native crate exposing both schemes.
Align with the in-flight `ff-volume` split (CR-CH-057) so the physical seam is a
Volume->StorageProvider URI. Several of these are FRAMEWORK CHANGES (removing/
reshaping public framework types, changing a governance ADR) and REQUIRE OWNER
CONFIRMATION per `framework-conformance.md`.

---

## Q1. ff-dscatalog vs ff-dataset-catalog -- full comparison

### ff-dataset-catalog (the trait-only crate)

- **Files:** exactly TWO source files -- `src/lib.rs` (28 lines, doc + `pub mod
  traits; pub use traits::*;`) and `src/traits.rs` (~415 lines incl. tests).
- **Cargo.toml:** depends ONLY on `thiserror` (+ dev `proptest`,
  `pretty_assertions`). No `ff-vfs`, no `rusqlite`, no runtime.
- **What it DECLARES (not implements):**
  - Error types: `CatalogError`, `DsnValidationError`.
  - Data types: `DatasetId(String)`, `Dsorg {Ps, Po, Da, Vsam}`,
    `Recfm {F, Fb, V, Vb, U}`, `DatasetAttributes`, `ResolutionResult`,
    `DatasetEntry`, `DatasetFilter`, `GenerationInfo`.
  - Traits: `CatalogService` (static-dispatch) and `DynCatalogService`
    (object-safe) with a blanket `impl<T: CatalogService> DynCatalogService for
    T`.
- **What it IMPLEMENTS:** nothing but `Display` for `Dsorg`/`Recfm` and the
  blanket forwarding impl. There is NO concrete `CatalogService` implementor in
  this crate. The only test asserts `Box<dyn DynCatalogService>` compiles.
- **Public API surface:** the traits + the data/error types above.

### ff-dscatalog (the real implementation crate)

- **lib.rs modules (24):** `audit, catalog, catalog_registry, codecs, commands,
  config, context_menu, dataset, dsn, encoding, error, gdg, hierarchy, integrity,
  listcat, pds, properties, repository, schema, security, storage, transactions,
  vfs_provider` (+ `storage` has submodules `esds, isam, native, rrds,
  sqlite_record`).
- **Cargo.toml:** depends on `ff-vfs`, `ff-command`, `ff-config`, `ff-logging`,
  `rusqlite 0.31` (bundled), `tokio`, `async-trait`, `uuid`, `zip`, `walkdir`,
  `percent-encoding`, `chrono`, `serde`.
- **What it IMPLEMENTS (real code):**
  - SQLite catalog DB (`catalog.rs`, `schema.rs`), `CatalogRegistry`
    (`catalog_registry.rs`), repository layout (`repository.rs`).
  - DSN parsing/validation (`dsn.rs`: `Dsn`, `MemberName`).
  - Record codecs (`codecs/`): `FixedCodec`, `VariableCodec` (4-byte RDW),
    `BinaryCodec`, `TextCodec`.
  - Physical storage backends (`storage/`): `NativeFileProvider`,
    `NativeEsdsProvider` (ESDS), `SqliteRecordProvider` (KSDS + alternate
    indexes), `SqliteRrdsProvider` (RRDS), `IsamProvider`.
  - `CatalogVfsProvider` (scheme `catalog`, implements `ff_vfs::VfsProvider`).
  - GDG (`gdg.rs`), PDS members (`pds.rs`), LISTCAT (`listcat.rs`), audit,
    integrity/reconcile, security scrubbing.
- **Public API surface (lib.rs re-exports):** `Catalog, CatalogLocation,
  CatalogMount, CatalogRegistry, AllocParams, DatasetRecord, Dsorg,
  PartitionedSubtype, Recfm, Dsn, MemberName, CatalogError, GdgBase,
  GdgGeneration, GdgStatus, CatalogScope, PdsMemberInfo, DatasetProperties,
  Repository, CatalogVfsProvider`. There is NO `CatalogService` trait here -- the
  catalog is exposed as concrete `Catalog` / `CatalogRegistry` types.

### The DIFFERING Dsorg / Recfm definitions (exact variants)

| Type | `ff-dataset-catalog` (`traits.rs`) | `ff-dscatalog` (`dataset.rs`) |
|---|---|---|
| `Dsorg` | `Ps, Po, Da, Vsam` | `PS, PO, GDG` (`#[non_exhaustive]`) |
| `Recfm` | `F, Fb, V, Vb, U` | `F, FB, V, VB, U` (`#[non_exhaustive]`) |

They disagree on both the VARIANT SET and the CASING:
- `ff-dataset-catalog` has `Da` and `Vsam` but NOT `GDG`; GDG is handled as a
  catalog concern elsewhere, VSAM as a `Dsorg`.
- `ff-dscatalog` has `GDG` as a first-class `Dsorg` and NO `Da`/`Vsam` variant
  (VSAM sub-types live in `ff-vsam-services`/`storage`, not as a `Dsorg`).
- Casing differs (`Fb` vs `FB`, `Po` vs `PO`), so even the overlapping concepts
  are not the same Rust value.

### Which crate do the requirements docs treat as authoritative?

CONTRADICTORY across specs:
- `docs/specs/dataset-catalog/requirements.md` states verbatim: the real crate is
  `ff-dscatalog`, and any reference to `ff-dataset-catalog` in older documents "is
  incorrect and should be read as `ff-dscatalog`."
- `docs/specs/dataset-ownership-model/requirements.md` (ADR-001, the GOVERNANCE
  doc that "takes precedence") still names **`ff-dataset-catalog`** as the owner
  of dataset definitions/attributes/resolution, and `ff-dataset-catalog`'s own
  `lib.rs` claims to be "the single authority".
- `volume-model` and the amended `dataset-catalog` glossary consistently use
  `ff-dscatalog` for the implementation that depends on `ff-volume`.

So the IMPLEMENTATION authority is unambiguously `ff-dscatalog`; the GOVERNANCE
text is stale and still points at `ff-dataset-catalog`. This stale pointer is the
root cause of the two-crate ambiguity.

---

## Q2. Consumer map

### Cargo.toml dependency edges (shipping vs test)

| Crate | Depends on | How |
|---|---|---|
| `ff-catalog-registry` | `ff-dscatalog`, `ff-vfs` | normal dep |
| `ff-catalog-dialog` | `ff-dscatalog`, `ff-catalog-registry` | normal dep (UI) |
| `ff-files-panel` | `ff-dscatalog` | normal dep (UI) |
| `ff-desktop` | `ff-dscatalog`, `ff-posix-provider`, `ff-connector-local-fs`, `ff-vfs` | normal dep (shell) |
| `ff-governance-tests` | `ff-dataset-catalog`, `ff-vsam-services` | **dev-dependency ONLY** |
| `ff-nav-model` | `ff-posix-provider` | dev-dependency |

Key facts:
- **`ff-dscatalog`** is consumed by `ff-catalog-registry`, `ff-catalog-dialog`,
  `ff-files-panel`, `ff-desktop` -- i.e. it IS in the shipping `ffwb` closure.
- **`ff-dataset-catalog`** is consumed by NO shipping crate. The ONLY reference is
  `ff-governance-tests/tests/mock_compilation.rs` (dev). It is a governance/
  compile-fixture crate, not part of the running app.
- **`ff-vsam-services`** is likewise consumed ONLY by `ff-governance-tests` (dev).
  Its `StubVsamService` returns `NotImplemented` for every method and has no
  dependency on `ff-dscatalog`.

### `use` / trait references in .rs

- `use ff_dscatalog::...` -- appears across `ff-catalog-registry` (`Catalog`,
  `dataset::{AllocParams, DatasetRecord, Dsorg}`, `dsn`, `hierarchy`,
  `repository`) and `ff-files-panel` tests (`catalog::CatalogMount`, `dataset`,
  etc.). Real, broad usage of concrete types.
- `use ff_dataset_catalog::...` -- appears ONLY in
  `ff-governance-tests/tests/mock_compilation.rs` (imports `CatalogService,
  DynCatalogService, DatasetAttributes, Dsorg, ...`).
- `CatalogService` / `DynCatalogService` -- defined in `ff-dataset-catalog`
  (consumed only by governance tests) AND independently defined in
  `ff-idcams::services` (a DIFFERENT trait with different method signatures:
  `create_dataset(CreateDatasetParams)` vs `create_dataset(&str,
  DatasetAttributes)`). `ff-idcams` depends on its OWN trait, not on
  `ff-dataset-catalog`.
- `VsamService` -- defined in `ff-vsam-services` (governance-test-only) AND
  independently in `ff-idcams::services` (again a different shape). `ff-idcams`
  wires `Arc<dyn CatalogService>` / `Arc<dyn VsamService>` to ITS OWN traits via
  `IdcamsServices`, with `MockCatalogService`/`MockVsamService` test doubles.

### StorageProvider consumers

- `ff-vfs::StorageProvider` -- implemented by `ff-vfs::PosixNativeProvider`;
  registered via `ProviderRegistry::register_storage` /
  `get_storage` (the registry has a dedicated storage-provider map). No OTHER
  crate implements it today.
- `ff-dscatalog::storage::StorageProvider` -- implemented by
  `NativeFileProvider`, `NativeEsdsProvider`, `SqliteRecordProvider`,
  `SqliteRrdsProvider`, `IsamProvider` (all inside `ff-dscatalog`). Not used
  outside `ff-dscatalog`.

**Merge-cost conclusion from the map:** deleting `ff-dataset-catalog` and
`ff-vsam-services` breaks ONLY `ff-governance-tests` (one dev file) -- zero
shipping impact. Unifying the two `StorageProvider` traits is contained ENTIRELY
within `ff-dscatalog` (change which trait its 5 backends implement) plus the
`ff-vfs` registry already supporting storage registration.

---

## Q3. Merge-or-delete assessment

**Can `ff-dataset-catalog` be deleted?** Yes, with very low cost.
- Nothing in the shipping closure imports it. Its `CatalogService` trait is NOT
  used by any real consumer -- `ff-idcams` (the intended consumer per the trait's
  own doc-comment) actually defines and uses its OWN `CatalogService`. So the
  trait is dead except as a compile fixture.
- What would break: `ff-governance-tests/tests/mock_compilation.rs` (imports the
  trait + types) and `architecture_compliance.rs` references. These are
  governance tests, not product code; they would be updated to target the
  retained authority (`ff-dscatalog`).
- Its genuinely-useful ideas worth ABSORBING into `ff-dscatalog`'s public API: a
  trait-based `CatalogService`/`DynCatalogService` seam (so `ff-idcams`/
  `ff-dsalloc` can depend on an interface + mock it, which is the ADR-001 intent),
  and `DatasetAttributes`/`ResolutionResult`/`DatasetEntry` shapes. But these must
  be reconciled to `ff-dscatalog`'s `Dsorg {PS,PO,GDG}` / `Recfm {FB,...}` and its
  concrete `Dsn` type, not the divergent `ff-dataset-catalog` enums.

**Is `ff-dataset-catalog`'s trait used anywhere real?** No. Only in governance
tests. The "real" catalog-service abstraction that ships is `ff-idcams`'s own
trait pair, and the concrete `ff-dscatalog::Catalog`/`CatalogRegistry`.

**Recommended direction:** MERGE `ff-dataset-catalog` INTO `ff-dscatalog` by
(a) adding a reconciled `CatalogService`/`DynCatalogService` trait to
`ff-dscatalog` (using `ff-dscatalog`'s existing types), (b) deleting the
`ff-dataset-catalog` crate, (c) pointing `ff-idcams`/`ff-dsalloc` at the
`ff-dscatalog` trait (so there is ONE catalog-service interface), and (d)
updating ADR-001 to name `ff-dscatalog`. This is the lower-risk direction because
the implementation, all shipping consumers, and the record/VSAM code already live
in `ff-dscatalog`. (Reversing it -- moving the SQLite impl into
`ff-dataset-catalog` -- would churn every shipping consumer.) NOTE: changing
ADR-001's named owner and removing a public crate are FRAMEWORK/governance changes
requiring owner confirmation (section 7).

---

## Q4. The ff-vfs layer

Files: `crates/ff-vfs/src/{lib, provider, storage_provider, registry, posix_provider,
uri, types, error, search, watch, transaction, workspace, vfs, subsystem}.rs`.

### VfsProvider (`provider.rs`)
`#[async_trait] trait VfsProvider: Send + Sync` with `scheme() -> &str`,
`capabilities() -> VfsCapabilities`, and async `open/read/read_stream/write/
create/delete/rename/list/stat/exists` plus default-`UnsupportedOperation`
`watch`/`search`. Object-safe (compile-time assertions). This is the ROUTING /
abstraction layer that consumers call.

### StorageProvider (`storage_provider.rs`) -- the intended physical seam
`trait StorageProvider: Send + Sync` (sync, not async) with
`capabilities() -> HashSet<StorageCapability>`, `supports(cap)`, `allocate(name)
-> StorageLocator`, `open/stat/rename/delete(locator)`, `list()`, `reconcile(
catalogue_names)`, default `write()`. `StorageLocator` is OPAQUE (`as_str()`
only). `StorageCapability` is a rich 12-variant enum (`StreamRead/Write,
RecordRead/Write, KeyedAccess, RelativeAccess, AppendOnly, MemberOperations,
AtomicRename, Locking, Snapshotting, WatchNotifications`). Doc: "keeps physical
storage concerns ... out of VFS routing logic" (Requirement 9).

### Provider registry (`registry.rs`)
`ProviderRegistry` holds TWO maps: `providers: HashMap<String, Arc<dyn
VfsProvider>>` keyed by scheme, and `storage_providers: HashMap<String, Arc<dyn
StorageProvider>>`. API: `register(Arc<dyn VfsProvider>)` (scheme from
`provider.scheme()`), `deregister/get/list_schemes/list_providers`,
`default_scheme` (="local"), `has_default_provider`, plus
`register_storage(scheme, Arc<dyn StorageProvider>)` / `deregister_storage /
get_storage / list_storage_schemes`. Duplicate scheme = `VfsError::DuplicateScheme`.

### URI routing
`vfs://provider/path` (`uri.rs`, `ResourceUri`); bare paths default to
`vfs://local/...`. The `Vfs` facade (`vfs.rs`) and `VfsSubsystem` (`subsystem.rs`)
route by scheme to the registered provider.

### Is the layering clean? Mostly yes, in `ff-vfs` itself.
The intended separation is correct and well-documented: `VfsProvider` on top
(what editors/panels call), `ff-vfs::StorageProvider` below (physical backend),
registry keeps the two in separate maps. `PosixNativeProvider` demonstrates ONE
object implementing BOTH traits cleanly.

### Does it support the owner's three-kind model cleanly? Yes, structurally.
Three VFS schemes (`local`, `posix`, `catalog`) map exactly to Native / POSIX /
Mainframe. The `catalog` provider (`ff-dscatalog::CatalogVfsProvider`) can sit on
top of the mainframe record/VSAM backends via `ff-vfs::StorageProvider`. The ONLY
blur is that `ff-dscatalog` introduced a SECOND, incompatible `StorageProvider`
trait instead of implementing the `ff-vfs` one -- so today the catalog's physical
backends do NOT plug into the `ff-vfs` storage seam. Fixing that (Q7) is what
makes the layering actually clean end-to-end.

---

## Q5. Current provider implementations, mapped to the three kinds

| Provider (symbol / crate) | VFS scheme | Traits implemented | Owner's kind |
|---|---|---|---|
| `LocalFsProvider` (`ff-connector-local-fs`) | `local` | `VfsProvider` | **Native** |
| `PosixProvider` (`ff-posix-provider`) | `posix` | `VfsProvider` (wraps `LocalFsProvider`, root-jail, read-only) | **POSIX catalog** |
| `PosixNativeProvider` (`ff-vfs::posix_provider`) | `posix` | `VfsProvider` + `ff-vfs::StorageProvider` | POSIX/Native (duplicate) |
| `CatalogVfsProvider` (`ff-dscatalog::vfs_provider`) | `catalog` | `VfsProvider` | **Mainframe catalog** |
| `NativeFileProvider` (`ff-dscatalog::storage::native`) | n/a | `ff-dscatalog::StorageProvider` | Mainframe physical (PS/PDS/GDG) |
| `NativeEsdsProvider` (`storage::esds`) | n/a | `ff-dscatalog::StorageProvider` | Mainframe physical (VSAM ESDS) |
| `SqliteRecordProvider` (`storage::sqlite_record`) | n/a | `ff-dscatalog::StorageProvider` | Mainframe physical (VSAM KSDS + AIX) |
| `SqliteRrdsProvider` (`storage::rrds`) | n/a | `ff-dscatalog::StorageProvider` | Mainframe physical (VSAM RRDS) |
| `IsamProvider` (`storage::isam`) | n/a | `ff-dscatalog::StorageProvider` | Mainframe physical (ISAM) |

Observations:
- **Native files:** well-served by `LocalFsProvider` (scheme `local`). This is
  the "primary VFS provider" per `connector-local-fs`.
- **POSIX catalog:** TWO implementations both claiming scheme `posix`
  (`ff-posix-provider::PosixProvider` and `ff-vfs::PosixNativeProvider`). Only one
  can register for `posix` at a time (registry forbids duplicate scheme). This is
  a straight duplication to resolve.
- **Mainframe dataset catalog:** `CatalogVfsProvider` (routing) + five physical
  backends (storage). Rich and real. But the backends implement the WRONG
  (duplicate) `StorageProvider` trait, so they are not reachable through the
  `ff-vfs` storage seam.
- **Not yet wired live:** `ff-desktop/src/main.rs` registers config schema but
  registers NO VFS providers into a `ProviderRegistry`. The running app instead
  uses `ff-desktop::catalog_registry::{VirtualCatalog, CatalogType {Mainframe,
  Posix, Native}}` (a FOURTH catalog model) for its POM option-1 Files UI. So the
  built provider stack and the live UI catalog model are not yet connected.

---

## Q6. Spec landscape

| Sub-project | Governs | Already describes the layered 3-kind model? |
|---|---|---|
| `virtual-file-system` | `ff-vfs`: VfsProvider, URI scheme, registry, StorageProvider seam (Req 9), record codecs (Req 16), hybrid storage (Req 18-20) | YES -- defines the two-layer seam and that providers (local/catalog/future) are interchangeable. |
| `dataset-catalog` | `ff-dscatalog`: SQLite catalog, PS/PDS/GDG, DSN, codecs, `catalog` provider | YES for mainframe kind. Says `ff-dscatalog` is the real crate; `ff-dataset-catalog` is "incorrect". |
| `dataset-ownership-model` | ADR-001 single-authority ownership across ff-vfs / catalog / allocator / idcams / vsam | PARTLY -- correct principle, but still names **`ff-dataset-catalog`** (stale) as catalog owner; conflicts with implementation. |
| `volume-model` | NEW `ff-volume` crate (CR-CH-057/CR-NR-105): Volume as first-class physical container, VOLSER, geometry-as-metadata, x37 space errors, DatasetVolume | YES -- explicitly "a Volume maps to a VFS StorageProvider URI (ADR-001), building ON the existing ff-vfs StorageProvider seam". Directly relevant to the physical seam. |
| `connector-local-fs` | `ff-connector-local-fs::LocalFsProvider` (scheme `local`) -- the Native kind | YES -- the Native provider. |
| `connector-mainframe` | FUTURE `ff-connector-mainframe` (z/OS FTP/TN3270/zOSMF/USS) -- **DEFERRED**, not initial release | N/A -- real remote z/OS, out of scope for the emulation revision. |
| `connector-network-fs` | FUTURE network-FS connector -- deferred/architectural hook only | N/A for now. |
| `virtual-catalog-manager` | UI subsystem owning POM option 1 (Files); defines the FOUR catalog kinds table | YES -- names Mainframe=`catalog`, POSIX=`posix`, Native=`local` EXACTLY as the owner's model. This is the clearest statement of the target model. |
| `platform-core` | `ff-core` layer rules; places `ff-vfs` in the Core_Layer | YES -- fixes dependency direction (shell depends down; ff-vfs depends only on foundation). |

### ff-volume interaction (CR-CH-057 / CR-NR-105)
The Volume split promotes the physical Repository into a first-class `ff-volume`
Volume; the Catalog becomes a pure metadata locator (ADR-002: catalogs never hold
bytes). Schema v4 adds `volumes` + `dataset_volumes`. Crucially, `volume-model`
says a Volume maps to a VFS `StorageProvider` URI and builds ON the existing
`ff-vfs` StorageProvider seam -- which means the Volume layer EXPECTS the single
`ff-vfs::StorageProvider` to be the physical seam. This REINFORCES the Q7
recommendation (unify on `ff-vfs::StorageProvider`) and means the catalog's
`storage_path` becomes a DatasetVolume locator. Any consolidation must land
compatibly with this in-flight direction (the `vsam-wiring` worktree on branch
`feature/vsam-service-wiring` is already registered in `stream-prefixes.md`,
prefix `V`).

---

## Q7. Architecture recommendation (target design + migration)

The target IS the owner's model and it is already specified; the work is
consolidating duplicates onto it. Proposed clean architecture:

```
            consumers (editors, panels, ff-idcams, ff-dsalloc, UI)
                                 |
                        ff-vfs :: VfsProvider            (ONE routing seam)
                                 |  scheme -> provider
        +------------------------+------------------------+
        | local                  | posix                  | catalog
   LocalFsProvider          Posix provider          CatalogVfsProvider
   (Native kind)            (POSIX catalog kind)    (Mainframe catalog kind)
        |                        |                        |
        +------------------------+------------------------+
                                 |
                   ff-vfs :: StorageProvider              (ONE physical seam)
                                 |
   Native file store | POSIX store | Mainframe stores: NativeFile / ESDS /
                                     KSDS(+AIX) / RRDS / ISAM  (record codecs here)
                                 |
                        ff-volume :: Volume (VOLSER, locator)   [CR-CH-057]
```

### Crate boundaries (survive / merge / new)
- **Survive:** `ff-vfs` (THE abstraction, both traits), `ff-dscatalog` (mainframe
  catalog + record codecs + VSAM/record backends), `ff-connector-local-fs`
  (Native), `ff-idcams`, `ff-dsalloc`, `ff-plugin`, `ff-volume` (new, in flight).
- **Merge/delete:**
  - DELETE `ff-dataset-catalog`; move a RECONCILED `CatalogService`/
    `DynCatalogService` trait into `ff-dscatalog` using `ff-dscatalog` types.
  - DELETE `ff-vsam-services` as a separate crate; make `ff-dscatalog` expose the
    concrete `VsamService` backed by its existing `SqliteRecordProvider`/
    `NativeEsdsProvider`/`SqliteRrdsProvider`. (Alternatively keep
    `ff-vsam-services` as a thin trait home that `ff-dscatalog` IMPLEMENTS -- but
    only if a non-test consumer needs the trait; today none does, and `ff-idcams`
    carries its own.)
  - COLLAPSE the three native/posix providers: keep ONE Native crate
    (`ff-connector-local-fs`, scheme `local`) and ONE POSIX provider (pick
    `ff-posix-provider`'s wrapping design OR `ff-vfs::PosixNativeProvider`'s
    dual-trait design -- NOT both). Remove the duplicate `posix` registrant.
- **New:** `ff-volume` (already planned).

### The single StorageProvider seam
Adopt **`ff-vfs::StorageProvider`** as the one physical seam (it is the
UI-opaque, UUID-free, capability-rich trait the specs and `volume-model` already
target). Then:
- DELETE `ff-dscatalog::storage::StorageProvider` (the duplicate).
- Reimplement the five `ff-dscatalog` backends against `ff-vfs::StorageProvider`
  (map `ObjectId`/`workspace_root` behind `StorageLocator`; translate
  `CatalogError` <-> `VfsError`). Register them via `register_storage(scheme,
  ...)`. Record codecs stay in `ff-dscatalog::codecs` (no fs/SQLite/egui dep, per
  VFS Req 17).
- VSAM lives in `ff-dscatalog` (concrete `VsamService` over the KSDS/ESDS/RRDS
  backends) -- this is also the `vsam-wiring` worktree's job.

### Alignment with ff-volume
Make the physical seam resolve through a Volume: catalog resolves DSN -> Volume +
opaque locator (DatasetVolume), and the `StorageProvider` for that Volume
performs physical I/O. Replace `DatasetRecord.storage_path` with the
DatasetVolume locator (schema v4). This keeps the catalog metadata-only (ADR-002).

### Phased migration (FFWB builds between every phase)
1. **Spec reconciliation (docs only, gated).** Fix ADR-001 to name `ff-dscatalog`;
   add the `CatalogService` trait definition to the `dataset-catalog`/
   `dataset-ownership-model` specs with `ff-dscatalog` types; mark
   `ff-dataset-catalog`/`ff-vsam-services` crates as deprecated-for-merge.
2. **Add reconciled trait to `ff-dscatalog`** (additive; nothing breaks). Point
   `ff-governance-tests` at it. App still builds.
3. **Unify StorageProvider inside `ff-dscatalog`** (change the 5 backends to
   implement `ff-vfs::StorageProvider`; delete the local trait). Contained to one
   crate + registry; consumers unaffected.
4. **Wire VSAM** (concrete `VsamService` over backends) -- the `vsam-wiring`
   stream.
5. **Collapse the duplicate posix/native provider** to a single registrant.
6. **Register the provider stack live** in `ff-desktop` startup and bridge
   `catalog_registry::CatalogType` to the three VFS schemes (so the UI model and
   the provider stack are the same thing).
7. **DELETE `ff-dataset-catalog`** (and fold/retire `ff-vsam-services`) once no
   consumer -- including governance tests -- references them.
8. **Land `ff-volume`** locator as the physical address (CR-CH-057), replacing
   `storage_path`.

### FRAMEWORK CHANGES requiring OWNER CONFIRMATION
Per `.kiro/steering/framework-conformance.md`, these are not implementation
side-effects and MUST be confirmed before code:
- **Removing/reshaping public framework types/crates:** deleting the public
  crate `ff-dataset-catalog` and its `CatalogService`/`DynCatalogService`;
  retiring `ff-vsam-services`; DELETING `ff-dscatalog::storage::StorageProvider`
  and reshaping the five backends onto `ff-vfs::StorageProvider`.
- **Changing a governance ADR:** amending ADR-001 (`dataset-ownership-model`) to
  name `ff-dscatalog` as the catalog authority instead of `ff-dataset-catalog`.
- **Consolidating a duplicate dispatch/seam:** collapsing the two `posix`-scheme
  providers to one, and introducing live provider registration in `ff-desktop`
  startup (new wiring at the shell boundary).
- **New physical-address format:** moving from `storage_path` to a
  DatasetVolume/`ff-volume` locator (already a confirmed direction via
  CR-CH-057/CR-NR-105, but the catalog-side change should be gated with it).

Each of the above is additive-first where possible (phases 1-4 add before phase 7
deletes), so FFWB keeps building throughout. Per `rust-standards.md` the 400-line
rule applies to any new/split files (e.g. a reconciled `catalog_service.rs` in
`ff-dscatalog`), and per `wiring-standard.md` any new provider wires through the
existing registry + command seams, not a new dispatcher.

---

## Conclusions

1. The owner's three-kind VFS model is NOT missing -- it is the documented and
   substantially-built architecture (`virtual-catalog-manager`'s four-kind table
   plus `ff-vfs`'s two-layer seam). The revision is CONSOLIDATION, not a rewrite.
2. The concrete problems are: two catalog crates with incompatible `Dsorg`/`Recfm`
   enums; two `StorageProvider` traits; three catalog-service / two VSAM-service
   trait definitions; three native/posix providers; a fourth UI-side catalog
   model; a stale ADR-001 pointer; and a provider stack that is built/tested but
   not registered live.
3. The low-cost, low-risk resolution is to make `ff-dscatalog` the single catalog
   authority, make `ff-vfs::StorageProvider` the single physical seam, delete the
   governance-test-only `ff-dataset-catalog`/`ff-vsam-services` crates, collapse
   the duplicate posix providers, align the physical seam with `ff-volume`, and
   finally register the stack into the live shell -- in additive phases that keep
   FFWB building.
4. Several steps are framework/governance changes and MUST be owner-confirmed
   before any code; this report proposes but implements nothing.

---

## Key file reference

| Concern | Path |
|---|---|
| VFS routing trait | `crates/ff-vfs/src/provider.rs` |
| VFS physical seam (keep) | `crates/ff-vfs/src/storage_provider.rs` |
| Provider registry (2 maps) | `crates/ff-vfs/src/registry.rs` |
| ff-vfs posix provider (dup) | `crates/ff-vfs/src/posix_provider.rs` |
| Catalog impl crate | `crates/ff-dscatalog/src/lib.rs` (+ `dataset.rs`, `catalog.rs`, `vfs_provider.rs`) |
| Duplicate StorageProvider (delete) | `crates/ff-dscatalog/src/storage/mod.rs` |
| Record backends | `crates/ff-dscatalog/src/storage/{native,esds,sqlite_record,rrds,isam}.rs` |
| Record codecs | `crates/ff-dscatalog/src/codecs/` |
| Trait-only catalog crate (delete) | `crates/ff-dataset-catalog/src/{lib,traits}.rs` |
| VSAM trait + stub (retire) | `crates/ff-vsam-services/src/traits.rs` |
| ff-idcams's own service traits | `crates/ff-idcams/src/services.rs` |
| POSIX provider (dup) | `crates/ff-posix-provider/src/lib.rs` |
| Native provider | `crates/ff-connector-local-fs/src/lib.rs` |
| UI-side catalog model | `crates/ff-desktop/src/catalog_registry.rs` (CatalogType/VirtualCatalog) |
| Governance tests (only consumer of trait crates) | `crates/ff-governance-tests/tests/{mock_compilation,architecture_compliance}.rs` |
| Ownership ADR (stale pointer) | `docs/specs/dataset-ownership-model/requirements.md` |
| Catalog spec (says ff-dscatalog) | `docs/specs/dataset-catalog/requirements.md` |
| VFS spec | `docs/specs/virtual-file-system/requirements.md` |
| Volume split direction | `docs/specs/volume-model/requirements.md` |
| Three-kind model statement | `docs/specs/virtual-catalog-manager/requirements.md` |
| Stream prefix registry | `docs/status/stream-prefixes.md` (worktree `V` = vsam-wiring) |
```
