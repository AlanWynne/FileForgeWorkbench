# Mainframe Dataset Emulation -- Vision-Fit Evaluation and Rationalisation Recommendation

READ-ONLY architecture evaluation. No source or specs were changed. Workspace
root: `c:\workspace\VSC\FileForgeWorkbench`. Plain ASCII only (documentation.md).

This report evaluates the mainframe dataset / catalog / volume / VSAM work done
to date AGAINST the owner's stated vision, recommends a rationalised crate
structure and terminology, and sequences the work. It BUILDS ON (and updates
with current code state) the prior read-only analyses in this folder, chiefly
`mainframe-dataset-emulation/architecture-revision-findings.md`,
`mainframe-dataset-emulation/findings.md`,
`volume-model-recommendation/recommendation.md`, and
`dscatalog-duplicate/report.md`. Where the current tree differs from those
reports, the difference is flagged.

---

## 0. Executive summary

**One-line verdict: the work-to-date FITS the vision well in SHAPE and is
roughly 70-80% scaffolded, but it does NOT yet DELIVER the vision because the
two-layer seam is still blurred (SQLite reaches into the catalog directly, not
through a provider trait), the mainframe-interface contract a JES/JCL executor
needs is only partially present and split across drifted duplicate crates, and
the first-class Volume/DASD layer is designed but not built.**

The owner's mental model -- (1) emulate datasets but not hardware, (2) a clean
PHYSICAL layer optimised for host filesystems/SQLite under a MAINFRAME-FAITHFUL
INTERFACE layer, (3) an endgame JES/JCL executor that opens/reads/writes datasets
through that interface, (4) rationalise the crate sprawl -- is almost exactly the
documented-and-partly-built architecture. The gap is NOT a missing design; it is
DUPLICATION AND DRIFT plus two unbuilt layers (Volume, and the single
mainframe-dataset access interface). The dominant risk is that building JES/JCL,
VSAM wiring, or the Volume UI on TOP of the current drifted duplicates would
multiply the drift and have to be redone after consolidation.

Per-vision-point scorecard:

| Vision point | Verdict | One-line reason |
|---|---|---|
| 1. Emulate datasets, not hardware; Hercules-informed terminology | **Partial** | PS/PO/GDG + VSAM record semantics exist; terminology is inconsistent (no DASD/VOLSER/VTOC/DSCB anchor; `storage_path` leaks physical layout into the catalog). |
| 2. Two-layer: SQLite physical / mainframe-faithful interface | **Partial (blurred seam)** | A clean physical `StorageProvider` seam EXISTS in `ff-vfs`, but the catalog's own backends implement a SECOND, incompatible `StorageProvider` and the catalog stores a raw `storage_path` -- SQLite/physical path leaks into the interface. |
| 3. JES/JCL executor sits on the dataset interface (DD/DISP/alloc/catalog/GDG) | **Partial / Gap** | A real DD/DISP allocator (`ff-dsalloc`) and GDG/PDS/catalog exist, but there is NO single mainframe-dataset access interface (open/close/read/write by DD+DISP+DSN); the pieces are spread across three drifted service-trait definitions. |
| 4. Rationalise the crate sprawl | **Gap (not started)** | All the duplication is still present at HEAD: two catalog crates, two `StorageProvider` traits, three `CatalogService` + two `VsamService` definitions, two `Dsorg`/`Recfm` enums, no `ff-volume`. |

Positive note on recent progress (NEW since the earlier reports): the shell now
registers a LIVE `ff-vfs` `ProviderRegistry` at startup
(`crates/ff-desktop/src/shell/construct_provider.rs`), an `EnvironmentRegistry`
is built (`crates/ff-desktop/src/shell/environment_registry.rs`), and a per-FS
`BackendEnvironment` family exists (`ff-ce-posix`, `ff-ce-ntfs`,
`ff-ce-host-fs`). These close three of the "not wired live / closed env set"
gaps the earlier editor-CE report called out -- but a MAINFRAME/SQLITE backend
environment is explicitly still future.

---

## A. Vision-fit, per point (evidence-based)

### A.1 Emulate datasets, not hardware; Hercules-informed terminology -- PARTIAL

**Good fit (what emulates the mainframe reality correctly):**

- Dataset organisations and record semantics are real. `ff-dscatalog` implements
  PS, PO (PDS/PDSE), and GDG as first-class (`crates/ff-dscatalog/src/dataset.rs`
  `Dsorg {PS, PO, GDG}`, `PartitionedSubtype {PDS, PDSE}`; `gdg.rs`; `pds.rs`),
  with RECFM/LRECL/BLKSIZE attributes and validation (`AllocParams::validate`,
  LRECL 1..=32760, BLKSIZE >= LRECL).
- Record codecs correctly model mainframe record boundaries, NOT text lines:
  `FixedCodec`, `VariableCodec` (4-byte RDW), `BinaryCodec`, `TextCodec`
  (`crates/ff-dscatalog/src/codecs/`), and the design explicitly forbids CRLF/LF
  as a record delimiter (virtual-file-system Req 16.1). This is the right
  "interface emulates mainframe reality" instinct.
- VSAM organisations are modelled: KSDS (`storage/sqlite_record.rs`, keyed +
  alternate indexes), ESDS (`storage/esds.rs`, append-only), RRDS
  (`storage/rrds.rs`), ISAM (`storage/isam.rs`).
- DSN parsing/validation is mainframe-shaped (`crates/ff-dscatalog/src/dsn.rs`
  `Dsn`, `MemberName`, HLQ qualifiers).

**Gap vs correct Hercules-informed terminology (detail in section B):**

- The physical container concept (mainframe **DASD volume / VOLSER / VTOC**) has
  NO code anchor. `SCHEMA_VERSION` is still `"3"`
  (`crates/ff-dscatalog/src/schema.rs`) with NO `volumes` / `dataset_volumes`
  table and NO `volser` column; the `datasets` table carries a raw
  `storage_path TEXT NOT NULL`. The word "DASD" appears nowhere as a layer anchor.
- "Hardware not emulated" is correctly scoped: the Volume spec fixes tracks/
  cylinders/extents as METADATA-ONLY accounting units via a `Geometry_Profile`
  (volume-model Req 3, ADR-005), not CKD/ECKD device geometry. That is the right
  reading of Hercules (Hercules models real CKD/ECKD DASD device images and
  VOLSERs; FFWB deliberately emulates only the logical dataset/volume layer, not
  the device). Reference: Hercules documents supported DASD device types and DASD
  utilities ([Hercules Reference Summary, glanzmann.org mirror](https://hercdoc.glanzmann.org/V306/HerculesReferenceSummary.pdf)).
  Content was rephrased for compliance with licensing restrictions.

### A.2 Two-layer model (SQLite physical / mainframe-faithful interface) -- PARTIAL, seam blurred

**Good fit:** The intended two-layer seam EXISTS and is well-documented in
`ff-vfs`:
- Routing/interface layer: `VfsProvider` (`crates/ff-vfs/src/provider.rs`),
  scheme-keyed (`local` / `posix` / `catalog`), object-safe async open/read/
  write/list/stat.
- Physical layer: `ff-vfs::StorageProvider`
  (`crates/ff-vfs/src/storage_provider.rs`) -- a UI-opaque `StorageLocator`, a
  rich 12-variant `StorageCapability` enum (StreamRead/Write, RecordRead/Write,
  KeyedAccess, RelativeAccess, AppendOnly, MemberOperations, ...). Its doc states
  the explicit intent: "keeps physical storage concerns ... out of VFS routing
  logic" (Req 9).
- The hybrid storage decision matches the owner's "SQLite as a physical backend"
  preference: catalog METADATA in SQLite, PS/PDS/GDG as native files, keyed VSAM
  (KSDS/RRDS) in per-dataset SQLite record stores, ESDS as append-only native
  file (virtual-file-system Req 18, "Hybrid Storage Architecture"). So SQLite is
  deliberately the physical engine for keyed records, not for flat datasets.

**The blur (why this is only Partial):**

1. **TWO incompatible `StorageProvider` traits.** `ff-vfs::StorageProvider`
   (`storage_provider.rs:101`, `HashSet<StorageCapability>`, opaque
   `StorageLocator`, UUID-free) AND `ff-dscatalog::storage::StorageProvider`
   (`crates/ff-dscatalog/src/storage/mod.rs:66`, `ObjectId = uuid::Uuid`,
   `&[ProviderCapability]`, `workspace_root`-threaded). The five mainframe
   backends implement the `ff-dscatalog` one; only `PosixNativeProvider`
   implements the `ff-vfs` one. The two never meet -- so the mainframe physical
   backends do NOT plug into the intended `ff-vfs` physical seam.
2. **The catalog stores a raw physical path.** `datasets.storage_path TEXT NOT
   NULL` (`schema.rs`) is a direct relative path into the repository. That is
   physical layout leaking INTO the interface/metadata layer -- the exact
   ADR-002 violation ("catalogs shall not physically contain / own the dataset
   data"; FFWB_Storage_and_Catalog_Data_Model.md section 23). The volume-model
   spec's whole purpose is to replace `storage_path` with a DatasetVolume
   (volume + opaque locator) (volume-model glossary "DatasetVolume"; dataset-
   catalog Req 32), but that is not built (schema still v3).
3. **The catalog is a concrete SQLite type, not an interface.** `Catalog` owns a
   `Repository { root: PathBuf }` + a `rusqlite::Connection`
   (`crates/ff-dscatalog/src/catalog.rs`, `repository.rs`), so a consumer wanting
   "the mainframe interface" today binds to a concrete SQLite-backed type, not an
   abstract dataset-access trait.

Net: the physical/interface SEAM is designed and half-present, but SQLite and the
physical path currently leak through the catalog metadata layer, and the
mainframe backends sit behind the WRONG physical trait. Unifying on
`ff-vfs::StorageProvider` and replacing `storage_path` with a Volume locator is
what makes the two-layer vision actually true end-to-end.

### A.3 JES/JCL executor on the dataset interface -- PARTIAL / GAP

**Good fit (more is built than the earlier reports credited):**

- A real DD/DISP allocation engine exists in `ff-dsalloc`:
  `simulate_allocation` (`crates/ff-dsalloc/src/allocation.rs`) interprets
  `DispStatus::{New, Old, Shr, Mod}` with correct semantics (NEW rejects
  existing, OLD/SHR require existing, MOD appends-or-creates-with-SPACE), a
  `PassTable` for `DISP=PASS` across steps, `DdStatement` (`dd_statement.rs`),
  `operands.rs` (`DispAction`, `SpaceAllocation`, `DcbAttributes`),
  `gdg_resolver.rs`, `temp_registry.rs`. This is squarely the JCL DD-statement
  -> dataset-access model the vision names.
- GDG relative/absolute resolution, PDS members, LISTCAT all exist in
  `ff-dscatalog` -- the catalog surface a JCL executor needs for cataloging.

**Gap (why only Partial):**

1. **No single mainframe-dataset ACCESS interface.** The vision needs ONE
   contract: "allocate/open/close/read-record/write-record keyed by DD name +
   DISP + DSN". Today that is spread across, and DUPLICATED by, three different
   service-trait definitions that do not agree:
   - `ff-dscatalog` exposes CONCRETE `Catalog`/`CatalogRegistry` (no service
     trait).
   - `ff-idcams::services` defines its OWN `CatalogService` and `VsamService`
     (`crates/ff-idcams/src/services.rs:289,324`).
   - `ff-dataset-catalog::CatalogService` (`crates/ff-dataset-catalog/src/traits.rs:181`)
     and `ff-vsam-services::VsamService`
     (`crates/ff-vsam-services/src/traits.rs:139`) are a THIRD/second set, used
     only by `ff-governance-tests` (dev-dependency).
2. **The allocator resolves to a physical path, not through the record
   interface.** `ff-dsalloc` outcomes carry `physical_path: String`
   (`allocation.rs` `AllocationOutcome::{Verified, Allocated}`), and its
   `catalog_bridge` abstracts catalog access via a LOCAL `CatalogProvider` trait
   whose doc says "Production would delegate to `ff-dataset-catalog`"
   (`crates/ff-dsalloc/src/catalog_bridge.rs`) -- i.e. it names the WRONG crate
   and returns a path, not a record stream. So the allocator and the record/VSAM
   backends are not yet joined through one interface.
3. **VSAM is stubbed and disconnected.** `ff-vsam-services` ships only
   `StubVsamService` (returns `NotImplemented`) and has NO dependency on
   `ff-dscatalog`, so its trait and the real KSDS/ESDS/RRDS backends cannot meet.
4. **The editor save path bypasses the dataset interface entirely.** FFEDIT SAVE
   is a direct `LocalFsProvider::write` to the physical path with no record
   packing (per the sibling `address-routing-findings.md`); a dataset opened from
   a mainframe catalog is saved byte-identically to a native file. The new
   `BackendEnvironment` family (A.2 progress) is the seam that will fix this, but
   there is no MAINFRAME backend environment yet.

Conclusion: the DD/DISP front half of the JCL model is genuinely built; the back
half (a single record-aware dataset-access interface the executor and the editor
both call, backed by the real codecs/VSAM backends through a Volume) is the
missing contract. Section D defines it.

### A.4 Rationalise the crate sprawl -- GAP (not started)

Every duplication the prior report identified is still present at HEAD (verified
this pass):
- Two catalog crates: `ff-dscatalog` (live, 24 modules, in the ffwb closure) vs
  `ff-dataset-catalog` (trait-only, 2 files, dev-dep of `ff-governance-tests`
  only).
- Two `StorageProvider` traits (A.2).
- Three `CatalogService` definitions + two `VsamService` definitions (A.3).
- Two divergent `Dsorg`/`Recfm` enums: `ff-dataset-catalog` `Dsorg {Ps,Po,Da,Vsam}`
  / `Recfm {F,Fb,V,Vb,U}` vs `ff-dscatalog` `Dsorg {PS,PO,GDG}` /
  `Recfm {F,FB,V,VB,U}` (different variant SETS and casing).
- `storage_path` still in schema v3; `ff-volume` crate does NOT exist (confirmed:
  no `crates/ff-volume` directory).
- A stale governance pointer: ADR-001 (`dataset-ownership-model`) still names
  `ff-dataset-catalog` as the authority while the `dataset-catalog` spec says the
  real crate is `ff-dscatalog`.

So point 4 is entirely ahead of us. Section C gives the target.

---

## B. Terminology map (Hercules-informed) + recommendations

Hercules is the correct reference for the LOGICAL mainframe vocabulary even
though FFWB does not copy its device-level emulation. Hercules models real DASD
device images, VOLSERs, and ships DASD utilities ([Hercules Reference Summary](https://hercdoc.glanzmann.org/V306/HerculesReferenceSummary.pdf));
FFWB should borrow the NAMES (DASD volume, VOLSER, VTOC, DSCB, catalog, VSAM
cluster, GDG, CI/CA) for the INTERFACE layer while keeping the physical layer as
host files + SQLite.

Current naming assessment and recommended target:

| Concept (correct mainframe term) | Today in FFWB | Issue | Recommended name |
|---|---|---|---|
| **DASD volume / VOLSER** (physical container) | Not modelled; `Repository { root }` is the de-facto volume; `datasets.storage_path` | No VOLSER, no volume entity; "DASD" absent | `Volume` with `volser` in a new `ff-volume` crate (volume-model Req 1) |
| **VTOC** (what-is-on-this-volume) | Absent | No per-volume content listing | `VTOC_View` (derived) in `ff-volume` (volume-model glossary) |
| **DSCB** (dataset control block / per-dataset metadata) | `DatasetRecord` row | Fine conceptually; name is non-mainframe but acceptable | Keep `DatasetRecord`; optionally document it as the DSCB-equivalent |
| **Catalog** (name -> volume+locator) | `Catalog` (SQLite DB + Repository) | Conflates locator with physical store (ADR-002 violation) | Keep `Catalog`, but metadata-only; locator becomes a `DatasetVolume` |
| **VSAM cluster / KSDS/ESDS/RRDS/LDS** | `storage/{sqlite_record,esds,rrds,isam}.rs`, `VsamType {Ksds,Esds,Rrds,Lds}` | Correct; LDS not yet backed | Keep; add LDS later |
| **CI/CA, SHAREOPTIONS, keys** | Parsed by `ff-idcams` (`DEFINE CLUSTER`) | Parsed but not bound to a cluster entity | Model a `VsamCluster` entity (source 7.13) |
| **DSORG** | `Dsorg` -- TWO conflicting enums | Drift (`Da`/`Vsam` vs `GDG`; `Po` vs `PO`) | ONE `Dsorg {PS,PO,GDG}` in `ff-dscatalog` (VSAM via cluster entity, not a Dsorg variant) |
| **RECFM** | `Recfm` -- TWO conflicting enums | Casing drift (`Fb` vs `FB`) | ONE `Recfm {F,FB,V,VB,U}` in `ff-dscatalog` |
| **DD / DISP / SPACE** | `ff-dsalloc` `DdStatement`, `DispStatus`, `SpaceAllocation` | Good; references wrong crate name | Keep; retarget to `ff-dscatalog` |
| **GDG base / generation** | `GdgBase`, `GdgGeneration` | Correct | Keep |
| **Physical backend (host file / SQLite)** | Two `StorageProvider` traits | Drift | ONE `ff-vfs::StorageProvider` as the physical seam |

Headline terminology recommendations:
1. Introduce **DASD/Volume/VOLSER/VTOC** as the explicit physical-layer anchor
   (the `ff-volume` crate). This is the single biggest terminology gap.
2. Collapse the TWO `Dsorg`/`Recfm` enums to the `ff-dscatalog` casing
   (`PS/PO/GDG`, `F/FB/V/VB/U`); treat VSAM as a cluster entity, not a `Dsorg`.
3. Rename the physical seam language consistently to "StorageProvider = physical,
   VfsProvider = interface/routing"; delete the duplicate catalog-local
   `StorageProvider`.
4. Fix stale crate names in docs/comments: `ff-dataset-catalog` ->
   `ff-dscatalog`, `ff-dataset-allocator` -> `ff-dsalloc` (both wrong names appear
   in `jes-emulator/requirements.md` and `ff-dsalloc/src/catalog_bridge.rs`).

---

## C. Recommended rationalised crate structure

The target is the owner's model and it is already specified; the work is
CONSOLIDATION onto it, not a rewrite. Target layered architecture:

```
   consumers: editor (FFEDIT), navigator, ff-idcams, ff-jes/JCL executor, UI
                                 |
                     ff-vfs :: VfsProvider            (ONE interface/routing seam)
                                 |  scheme -> provider
        +------------------------+------------------------+
        | local                  | posix                  | catalog
   LocalFsProvider          Posix provider          CatalogVfsProvider
   (Native)                 (POSIX)                  (Mainframe)
        +------------------------+------------------------+
                                 |
                   ff-vfs :: StorageProvider              (ONE physical seam)
                                 |
   host files / SQLite record stores: NativeFile / ESDS / KSDS(+AIX) / RRDS / ISAM
   (record codecs live here, in ff-dscatalog)
                                 |
                        ff-volume :: Volume (VOLSER, storage_uri -> StorageProvider URI)
```

### KEEP / MERGE / RENAME / RETIRE table

| Crate | Action | Rationale |
|---|---|---|
| `ff-vfs` | **KEEP** | THE interface + physical abstraction (both traits). Already the live seam (ProviderRegistry registered at startup). |
| `ff-dscatalog` | **KEEP** (and make authoritative) | The live catalog + record codecs + VSAM/record backends. Absorb the reconciled `CatalogService`/`VsamService` traits here. |
| `ff-connector-local-fs` | **KEEP** | The Native (`local`) provider; already live-registered. |
| `ff-dsalloc` | **KEEP** | Real DD/DISP/SPACE/PASS allocator; retarget its `catalog_bridge` to `ff-dscatalog` and (later) resolve a Volume. |
| `ff-idcams` | **KEEP** | IDCAMS parser/executor; repoint its `CatalogService`/`VsamService` at the reconciled `ff-dscatalog` traits (stop carrying its own copies). |
| `ff-plugin` | **KEEP** | Plugin seam (unchanged by this). |
| `ff-volume` | **CREATE** (new) | First-class Volume/VOLSER/VTOC/geometry layer (volume-model Req 1-11). `ff-dscatalog` depends on it. Owns the physical container; catalogs never own bytes. |
| `ff-dataset-catalog` | **RETIRE (merge into `ff-dscatalog`)** | Trait-only, orphan to the binary, divergent enums. Fold a reconciled `CatalogService` into `ff-dscatalog`; delete the crate after repointing `ff-governance-tests`. GATED (removes a public crate + amends ADR-001). |
| `ff-vsam-services` | **RETIRE or thin** | Stub-only, governance-test-only, no `ff-dscatalog` dep. Either delete and expose a concrete `VsamService` from `ff-dscatalog`, or keep as a thin trait home that `ff-dscatalog` IMPLEMENTS -- only if a real consumer needs the trait (none does today). GATED. |
| `ff-posix-provider` vs `ff-vfs::PosixNativeProvider` | **MERGE to ONE `posix` registrant** | Two crates both register scheme `posix`; the registry forbids duplicate schemes. Pick one design. |
| `ff-ce-posix` / `ff-ce-ntfs` / `ff-ce-host-fs` | **KEEP** | The per-FS backend Command Environments + host decider (CR-CH-053). Light, shell-free (`ff-vfs::BackendEnvironment` only). ADD a `MAINFRAME`/`SQLITE` backend CE here later. |
| `ff-catalog-registry`, `ff-catalog-dialog`, `ff-dataset-alloc-dialog`, `ff-files-panel` | **KEEP** | UI consumers of `ff-dscatalog`; unaffected by the trait consolidation if it is additive-first. |
| `ff-desktop::catalog_registry` (`CatalogType {Mainframe,Posix,Native}`) | **KEEP, reconcile** | The live UI catalog model; it maps 1:1 to the three VFS schemes. Keep it as the UI model but ensure it drives the three schemes rather than being a 4th parallel model. |
| `ff-ce-host-fs` naming crates (`ff-ce-ntfs` etc.) | **KEEP** | Correct per-FS split; not part of the dataset drift. |

### Target dependency DAG (must stay acyclic; `ff-volume` owns Volume; catalogs never own bytes)

```
ff-volume          (Volume, VOLSER, VTOC, geometry; depends on ff-vfs StorageProvider URI)
   ^
   |
ff-dscatalog  -->  ff-vfs  (interface + physical traits)
   ^   ^   ^
   |   |   +-- ff-idcams        (uses ff-dscatalog CatalogService/VsamService)
   |   +------ ff-dsalloc       (uses ff-dscatalog via catalog_bridge; resolves Volume)
   +---------- ff-catalog-registry / ff-files-panel / dialogs (UI)
                        ^
                        |
                     ff-desktop (shell: ProviderRegistry + EnvironmentRegistry live)

ff-jes (future) --> ff-dsalloc + ff-dscatalog (through the ONE access interface, section D)
```

Invariants this preserves: single interface seam (`VfsProvider`), single physical
seam (`ff-vfs::StorageProvider`), single catalog authority (`ff-dscatalog`),
Volume owned by `ff-volume`, catalogs hold metadata + locator only (ADR-002).

---

## D. JES/JCL interface-contract readiness + the single interface to target

### What a JCL executor needs from the dataset layer

A step executor binds each DD to a dataset and performs record I/O under a
disposition. The minimal contract:

1. **Allocate/resolve** by `(DDNAME, DSN|&&temp, DISP=NEW/OLD/SHR/MOD, SPACE,
   DCB, VOL=SER/UNIT)` -> an opaque dataset handle, honouring catalog resolution,
   GDG relative refs, concatenation, and volume online/RO checks.
2. **Open/close** the handle for a given access intent (read/write/update).
3. **Record read/write** (GET/PUT/POINT) honouring RECFM/LRECL (fixed, variable
   with RDW, keyed for KSDS, relative for RRDS).
4. **Disposition at step end** (KEEP/CATLG/UNCATLG/DELETE/PASS).
5. **SYSOUT** as a spool dataset sink.

### Does any current trait approximate it?

| Need | Closest current surface | Fit |
|---|---|---|
| Allocate by DD+DISP | `ff-dsalloc::simulate_allocation` + `DdStatement` + `PassTable` | GOOD for allocation/DISP/PASS; returns a `physical_path`, not a record handle |
| Catalog resolve / GDG | `ff-dscatalog` `Catalog`/`CatalogRegistry`/`gdg` | GOOD, but concrete type not an interface |
| Record read/write | `ff-dscatalog::codecs` + `storage::*` backends | GOOD codecs/backends, but behind the DUPLICATE `ff-dscatalog::StorageProvider`, not reachable via `ff-vfs` |
| VSAM get/put/browse | `ff-vsam-services::VsamService` | DECLARED but stub-only, disconnected from backends |
| Open/close handle, disposition | none unified | GAP |

So the pieces exist but behind THREE different service traits and a duplicate
physical trait; there is no ONE handle-based access interface.

### The single interface the JES/JCL work should target

**Define ONE `DatasetAccess` (or `MvsIo`) trait as the mainframe-faithful access
contract, owned by `ff-dscatalog`, backed by `ff-vfs::StorageProvider` + the
record codecs, and resolving physical location through `ff-volume`.** Shape
(illustrative, to be gated):

```
trait DatasetAccess {
    fn allocate(&self, dd: &DdRequest) -> Result<DatasetHandle, DatasetError>; // DISP/SPACE/DCB/VOL
    fn open(&self, h: &DatasetHandle, intent: AccessIntent) -> Result<OpenDataset, DatasetError>;
    fn get(&self, od: &mut OpenDataset) -> Result<Option<Record>, DatasetError>; // RECFM-aware
    fn put(&self, od: &mut OpenDataset, rec: &Record) -> Result<(), DatasetError>;
    fn point(&self, od: &mut OpenDataset, key_or_rrn: &Positioner) -> Result<(), DatasetError>; // VSAM
    fn close(&self, od: OpenDataset) -> Result<(), DatasetError>;
    fn dispose(&self, h: DatasetHandle, outcome: StepOutcome) -> Result<(), DatasetError>; // KEEP/CATLG/...
}
```

- `ff-dsalloc` becomes the DD/DISP front-end that produces a `DdRequest` and calls
  `allocate`; it stops returning raw `physical_path`.
- `ff-idcams` and the editor's future MAINFRAME `BackendEnvironment` SAVE both
  call this ONE trait (IDCAMS DEFINE/REPRO/DELETE map to allocate/get-put/dispose).
- `ff-jes` depends on `ff-dsalloc` + this trait only -- it never touches SQLite or
  physical paths.

This is the "single interface the JES/JCL work should target". It MUST be defined
(it does not exist today); the earlier recommendation to "wire VSAM, do not
greenfield" is correct, but the wiring should land UNDER this one trait rather
than reviving `ff-vsam-services`'s disconnected trait.

---

## E. Sequenced recommendation (cheap-now / defer / owner-blocked)

Governing constraints: ROADMAP says CORE must be signed off FIRST; dataset/volume
emulation is PLUGIN phase 2, JES is PLUGIN phase 6
(`docs/project-management/ROADMAP.md` sections 3, 5, 6). A separate worktree is
doing live editor (CR-CH-058) work. So nothing here should compete with CORE
stabilisation; the dataset work is pre-positioning for phase 2.

### Cheap-and-safe NOW (docs/gated consolidation; no new behaviour, unblocks everything)

These are the consolidation steps that REDUCE drift and are prerequisites for any
later build. Each is a FRAMEWORK/governance change requiring the requirements
gate + owner confirmation per `framework-conformance.md` (they remove/reshape
public types or amend an ADR), but they are additive-first so FFWB keeps building:

1. **Spec reconciliation (docs only).** Amend ADR-001 to name `ff-dscatalog` as
   the catalog authority; mark `ff-dataset-catalog`/`ff-vsam-services` as
   deprecated-for-merge; fix the wrong crate names in `jes-emulator/requirements.md`
   (`ff-dataset-catalog` -> `ff-dscatalog`, `ff-dataset-allocator` -> `ff-dsalloc`)
   and in `ff-dsalloc/src/catalog_bridge.rs` doc comments.
2. **Add a reconciled `CatalogService`/`VsamService` to `ff-dscatalog`** using
   `ff-dscatalog`'s own types (`Dsorg {PS,PO,GDG}`, `Recfm`, `Dsn`). Additive;
   nothing breaks. Repoint `ff-governance-tests`.
3. **Unify the physical seam inside `ff-dscatalog`**: make the five backends
   implement `ff-vfs::StorageProvider` and delete `ff-dscatalog::storage::StorageProvider`.
   Contained to one crate + the (already live) registry.
4. **Collapse the duplicate `posix` provider** to a single registrant.

### Build NEXT, when phase 2 opens (gated features, in order)

5. **Land `ff-volume`** (volume-model spec is already written through Req 11):
   create the crate, schema v4 (`volumes` + `dataset_volumes`), migrate
   `storage_path` -> DatasetVolume locator with dual-read. This is the physical/
   interface seam fix (A.2) and the terminology anchor (B).
6. **Define the single `DatasetAccess` trait** (section D) in `ff-dscatalog`;
   retarget `ff-dsalloc` to it (return a handle, not a path).
7. **Wire VSAM** under `DatasetAccess` (concrete `VsamService`/get-put over the
   KSDS/ESDS/RRDS backends), then retire `ff-vsam-services`.
8. **Add a MAINFRAME `BackendEnvironment`** (the per-FS CE family already exists;
   `ff-ce-*` only need a mainframe sibling) so FFEDIT SAVE routes record-aware
   writes through `DatasetAccess` instead of `LocalFsProvider::write`.

### DEFER (owner-blocked or later phase)

- JES/JCL executor itself (ROADMAP PLUGIN phase 6) -- but it MUST target the
  section-D `DatasetAccess` trait. Do not start it before steps 5-7.
- Volume UI (virtual-catalog-manager Req 17-18 Volume picker/VTOC) -- after
  `ff-volume` lands.
- Multivolume datasets, extents, master/user catalog + alias routing, uncataloged
  VOL=SER access -- source model Phase 4; owner decisions 3-5 in
  `volume-model-recommendation/recommendation.md` section 7 are still open.
- Geometry profile tuning, Volume Groups -- model-level only for now.

### MUST NOT be built before consolidation (avoid wasted effort)

- **Do NOT wire VSAM against `ff-vsam-services`'s disconnected trait**, nor
  against `ff-dscatalog::storage::StorageProvider` -- both would be redone after
  steps 2-3/6-7. Wire it under the ONE `DatasetAccess` trait on the unified
  physical seam.
- **Do NOT build the JES/JCL executor or the Volume UI against `storage_path`**
  or against `ff-dsalloc`'s current `physical_path` return -- both disappear when
  `ff-volume` + `DatasetAccess` land (steps 5-6).
- **Do NOT add a MAINFRAME backend environment that calls the catalog's concrete
  SQLite path directly** -- it must call `DatasetAccess`, or it re-creates the
  seam blur (A.2) in the editor.
- **Do NOT extend `ff-idcams`'s private `CatalogService`/`VsamService`** further;
  repoint it at the reconciled `ff-dscatalog` traits first (step 2).

---

## Evidence index (files / symbols / docs cited)

Code (current HEAD, this pass):
- `crates/ff-vfs/src/storage_provider.rs:101` -- `ff-vfs::StorageProvider`, `StorageCapability`, `StorageLocator` (physical seam).
- `crates/ff-vfs/src/provider.rs` -- `VfsProvider` (interface/routing seam).
- `crates/ff-vfs/src/backend_environment.rs:64` -- `BackendEnvironment` trait (CR-CH-053 SAVE addressing).
- `crates/ff-dscatalog/src/storage/mod.rs:66` -- duplicate `ff-dscatalog::storage::StorageProvider`.
- `crates/ff-dscatalog/src/schema.rs` -- `SCHEMA_VERSION = "3"`; `datasets.storage_path`; no `volumes`/`volser`.
- `crates/ff-dscatalog/src/lib.rs` -- 24 modules; `Dsorg {PS,PO,GDG}`, `Recfm`; `Catalog`/`CatalogRegistry` (concrete, no service trait).
- `crates/ff-dscatalog/src/catalog.rs`, `repository.rs` -- `Catalog` owns `Repository{root}` + `rusqlite::Connection`.
- `crates/ff-dscatalog/src/codecs/` -- `FixedCodec`/`VariableCodec`(RDW)/`BinaryCodec`/`TextCodec`.
- `crates/ff-dscatalog/src/storage/{sqlite_record,esds,rrds,isam,native}.rs` -- VSAM/record backends (behind the duplicate trait).
- `crates/ff-dataset-catalog/src/traits.rs:181` -- `CatalogService`; divergent `Dsorg {Ps,Po,Da,Vsam}`/`Recfm {F,Fb,...}`.
- `crates/ff-vsam-services/src/traits.rs:139` -- `VsamService` (stub-only consumer).
- `crates/ff-idcams/src/services.rs:289,324` -- `ff-idcams`'s OWN `CatalogService`/`VsamService`.
- `crates/ff-dsalloc/src/allocation.rs` -- `simulate_allocation`, `DispStatus {New,Old,Shr,Mod}`, `AllocationOutcome{..physical_path..}`, `PassTable`.
- `crates/ff-dsalloc/src/catalog_bridge.rs` -- local `CatalogProvider` trait; doc says "would delegate to `ff-dataset-catalog`" (wrong crate name).
- `crates/ff-ce-posix/src/lib.rs`, `ff-ce-host-fs/src/lib.rs` -- per-FS backend CEs; comments name future MAINFRAME/SQLITE CE.
- `crates/ff-desktop/src/shell/construct_provider.rs` -- LIVE `ProviderRegistry` seeded with `local` (CR-CH-053 Task 22, Req 17).
- `crates/ff-desktop/src/shell/state.rs`, `construct.rs` -- `EnvironmentRegistry::with_builtins()`, `provider_registry`.
- (No `crates/ff-volume` directory -- confirmed absent.)

Docs / specs:
- `docs/source-documents/dataset-catalog/FFWB_Storage_and_Catalog_Data_Model.md` -- ADR-001 (volumes first-class), ADR-002 (catalogs never own bytes), ADR-005 (tracks/cyls emulated units); sections 5, 7.2-7.17, 9, 10, 11.1 (DEFINE VOLUME), 14, 15.
- `docs/project-management/ROADMAP.md` -- CORE-first; dataset/volume = PLUGIN phase 2; JES = phase 6; section 7a storage/catalog model.
- `docs/specs/volume-model/requirements.md` -- Req 1-4 (Volume, VOLSER, status/access, geometry, SPACE), glossary (DatasetVolume, VTOC_View, Space_Abend).
- `docs/specs/virtual-catalog-manager/requirements.md` -- three catalog kinds -> schemes `catalog`/`posix`/`local`; Req 13/16 SQLite-backed catalog; Req 17-18 (Volume UI, referenced).
- `docs/specs/dataset-catalog/requirements.md` -- "`ff-dscatalog` is the real crate; `ff-dataset-catalog` is incorrect"; Req 32 (volume split).
- `docs/specs/dataset-ownership-model/requirements.md` -- ADR-001 (stale `ff-dataset-catalog` pointer).
- `docs/specs/jes-emulator/requirements.md` -- JES plugin; references wrong crate names `ff-dataset-catalog`/`ff-dataset-allocator`; DISP NEW/OLD/SHR/MOD.
- `docs/specs/virtual-file-system/requirements.md` -- Req 9 (StorageProvider seam), Req 16 (record codecs), Req 18 (hybrid storage).

Prior analyses extended by this report:
- `.agents/tasks/mainframe-dataset-emulation/architecture-revision-findings.md` (duplication/drift; consolidation plan).
- `.agents/tasks/mainframe-dataset-emulation/findings.md` ("most exists; wire VSAM, do not greenfield").
- `.agents/tasks/mainframe-dataset-emulation/address-routing-findings.md` (FFEDIT SAVE bypasses the dataset interface).
- `.agents/tasks/mainframe-dataset-emulation/editor-ce-extension-findings.md` (CE/editor seams; now partly landed).
- `.agents/tasks/volume-model-recommendation/recommendation.md` (split-not-rename Volume).
- `.agents/tasks/dscatalog-duplicate/report.md` (ff-dscatalog authoritative; gated removal of ff-dataset-catalog).

External reference (terminology only, cited; content rephrased for licensing
compliance):
- Hercules documentation mirror -- DASD device types / DASD utilities / VOLSER:
  [Hercules Reference Summary](https://hercdoc.glanzmann.org/V306/HerculesReferenceSummary.pdf),
  [Hercules General Information](https://hercdoc.glanzmann.org/V313/HerculesGeneralInfo.pdf).

---

This is design-input only. No requirements gate was run; any resulting change
(especially the GATED consolidation items in section E, which remove/reshape
public crates and amend ADR-001) must be gated and owner-confirmed separately per
`.kiro/steering/workflow.md` and `framework-conformance.md`.
