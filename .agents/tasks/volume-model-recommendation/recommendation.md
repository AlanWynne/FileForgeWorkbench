# Recommendation: Introduce a First-Class Volume Layer for FFWB Dataset Emulation

Status: READ-ONLY investigation and recommendation. No source or spec files were
modified. This report enumerates recommended changes only.

Author context: produced from the source design documents under
`docs/source-documents/dataset-catalog/`, the current spec sub-projects under
`docs/specs/`, and the live Rust crates under `crates/`.

---

## 1. Summary answer (read this first)

The owner's instinct is correct, with one refinement.

- YES -- FFWB needs a distinct, first-class **Volume** layer. The current build
  has no Volume concept at all in code or in the ratified specs: a "catalog" is
  implemented as a SQLite database bound 1:1 to a physical **Repository**
  directory that both locates AND physically stores the datasets. That single
  object is doing the job of a VOLSER/volume (physical container) AND a catalog
  (name locator) at the same time. That is the conflation the owner suspected.

- NUANCED -- the current "catalog" should NOT simply be renamed to "Volume".
  The right move is to **split** the current object into two: keep the Catalog as
  a pure name-to-location locator (metadata only), and promote the physical
  Repository that currently backs each catalog into a **Volume** (the thing that
  physically holds datasets, identified by a VOLSER). A dataset then "lives on" a
  Volume, and the Catalog merely records which Volume it lives on. This is
  exactly the model the project's own source design documents already propose but
  that was never carried into the ratified `dataset-catalog` spec or the code.

The good news: the target design is already written. The source document
`FFWB_Storage_and_Catalog_Data_Model.md` defines Volumes as first-class
(its ADR-001), defines catalogs as pure locators (its ADR-002), and gives a full
entity model. The gap is that the *implemented* spec (`docs/specs/dataset-catalog`)
and the shipping crate (`ff-dscatalog`) implemented a simpler, flattened model
that dropped the Volume layer. The recommendation is to realign the specs and code
with the storage data model that already exists on paper.

---

## 2. The real mainframe model (from the source documents)

The source documents describe the authentic z/OS layering. Citations are to files
under `docs/source-documents/dataset-catalog/`.

Layered storage model:

- **Volume / VOLSER** -- the physical DASD container. A volume has a unique serial
  (VOLSER), capacity, online/offline status, and holds datasets physically.
  Datasets physically reside on one or more volumes.
  (`FFWB_Storage_and_Catalog_Data_Model.md`, sections 7.2 Volume, 7.3 Volume Group,
  14 Volume Capacity, 15 Multivolume Datasets; glossary section 25.)
- **VTOC-equivalent** -- the source model does not use the term "VTOC" but models
  the same role through **Extent** and **Dataset-Volume Association** records that
  record which allocation segments of which dataset sit on which volume
  (`FFWB_Storage_and_Catalog_Data_Model.md` sections 7.10 DatasetVolume, 7.11 Extent).
  The per-volume list of what it contains is the VTOC role.
- **Catalog as name -> volume locator** -- a catalog stores metadata that resolves
  a logical dataset name to its assigned volume(s) and physical objects. It does
  NOT physically contain the dataset records. Master catalog + user catalogs +
  aliases route a high-level qualifier to the catalog that owns it.
  (`FFWB_Storage_and_Catalog_Data_Model.md` sections 5, 7.4-7.6, 9 Catalog
  Resolution Algorithm, and explicitly ADR-002 in section 23.)
- **Dataset types** -- PS (sequential), PO (PDS/PDSE), VSAM (KSDS/ESDS/RRDS/LDS),
  GDG base + generations, temporary/work.
  (`FileForgeWorkbench_Mainframe_Dataset_Architecture.md` sections 4-8;
  `FFWB_Storage_and_Catalog_Data_Model.md` section 7.7 DatasetType.)
- **Uncataloged datasets** -- the authentic model allows a dataset to physically
  exist on a volume without a catalog entry (located only by explicit VOL=SER +
  UNIT). The source model supports this implicitly because the dataset-to-volume
  association is separate from the catalog entry
  (`FFWB_Storage_and_Catalog_Data_Model.md` sections 7.10, 5).
- **DD-name to dataset binding, DISP** -- a running step binds a DDNAME to a
  dataset (or temporary dataset) with a disposition (NEW/OLD/SHR/MOD) and
  normal/abnormal disposition actions (KEEP/CATLG/UNCATLG/DELETE/PASS).
  (`FFWB_Storage_and_Catalog_Data_Model.md` section 7.17 DdAllocation;
  `FileForgeWorkbench_DDNAME_Mapping_Design_v1.1.md` sections 3-6;
  `dataset-lifecycle.md` sections 1-6.)
- **GDG** -- a GDG base holds generation/retention rules; generations resolve via
  relative references `(0)`, `(-1)`, `(+1)` and absolute `GnnnnVnn`.
  (`FFWB_Storage_and_Catalog_Data_Model.md` sections 7.14-7.15;
  `FileForgeWorkbench_Mainframe_Dataset_Architecture.md` section 7;
  `virtual-file-catalogs.md` GDG generations.)

The source model's own resolution flow makes the layering explicit: name ->
(alias) -> catalog -> dataset entry -> dataset-volume association(s) -> extents ->
physical objects, with a check that the required volumes are online
(`FFWB_Storage_and_Catalog_Data_Model.md` section 9, steps 1-10).

Two of the source model's Architectural Decision Records are the crux of this whole
report and are worth quoting in paraphrase (content rephrased for licensing
compliance):

- ADR-001: volumes shall be modelled explicitly as first-class domain objects and
  mapped to storage-provider URIs
  (`FFWB_Storage_and_Catalog_Data_Model.md` section 23).
- ADR-002: catalogs shall hold metadata references to datasets and their
  locations, and shall NOT physically contain the dataset data
  (`FFWB_Storage_and_Catalog_Data_Model.md` section 23).

The companion requirements document reinforces this with principle AR-PR-002
("Catalogue independence"): a dataset's logical name shall not be treated as its
physical path; the catalogue resolves a logical identifier to a storage provider
and provider-specific locator
(`FileForgeWorkbench_Virtual_File_and_Dataset_Storage_Requirements.md` section 4,
AR-PR-002; the whole document is a hybrid-storage architecture where SQLite is the
catalogue/metadata and native files are the payload).

---

## 3. FFWB's CURRENT vision and implementation (specs + code)

### 3.1 What the ratified spec says a catalog is

The implemented spec `docs/specs/dataset-catalog/requirements.md` defines, in its
Glossary (content rephrased for compliance):

- Catalog: a SQLite database that maps dataset names to physical file locations
  within a Repository. A session can mount several catalogs at once.
- Repository: a directory structure on the local filesystem that physically
  stores dataset content, organised into `storage/`, `pds/`, `gdg/`, and `temp/`
  subdirectories.
- Dataset_Resolution: look up a dataset name in mounted catalogs and return the
  physical path to the underlying file.

So in the current ratified vision, one Catalog == one Catalog_Database (`catalog.db`)
== one Repository directory. The catalog both locates the dataset AND is the
physical container it lives in. There is no layer between "catalog" and "disk". The
word "Volume" does not appear in the dataset-catalog, virtual-file-system,
virtual-catalog-manager, dataset-allocator, or dataset-ownership-model specs
(verified by search; the only hit was an IDCAMS `LISTCAT ... VOLUME` display-level
option in `dataset-ownership-model/requirements.md`, which is a display keyword with
no backing volume entity).

The UI spec `docs/specs/virtual-catalog-manager/requirements.md` reinforces the
flattening: a "catalog" is both a UI tree root and a VFS scheme (`catalog`, `posix`,
`local`), and a Mainframe catalog is created by pointing at a Repository Path
(see also `virtual-file-catalogs.md`, "Mainframe-specific fields" -> Repository Path,
and "How the Repository is Laid Out").

### 3.2 What the code actually implements

Primary crate: `crates/ff-dscatalog`.

- `crates/ff-dscatalog/src/schema.rs` -- the SQLite schema. The `datasets` table
  columns are `id, dsn, dsorg, storage_path, recfm, lrecl, blksize, subtype,
  scope, created, modified, accessed`. There is a `storage_path` column that is a
  direct relative path into the repository. There is NO `volser`, NO volume table,
  NO extent table, NO dataset-volume association table. `SCHEMA_VERSION` is "3";
  the migrations added an audit_log table and a `scope` (master/user) column -- no
  volume work has ever been done.
- `crates/ff-dscatalog/src/repository.rs` -- `Repository { root: PathBuf }` with
  `storage_dir()`, `pds_dir()`, `gdg_dir()`, `temp_dir()`, `catalog_db_path()`.
  This directory IS the physical container. It is the de-facto "volume" but is
  owned by, and 1:1 with, the catalog.
- `crates/ff-dscatalog/src/catalog.rs` -- `Catalog { name, repository, conn,
  priority }`. A `Catalog` owns one `Repository` and one SQLite `Connection`.
  `CatalogLocation` is `Local { path }` or `Remote { scheme, uri }` (remote is a
  stub). So a catalog is physically anchored to one path.
- `crates/ff-dscatalog/src/lib.rs` -- documents the model directly: datasets are
  stored in `storage/`, `pds/`, `gdg/`; the catalog maps DSNs "to physical files
  stored in a structured repository layout on the local filesystem."

Supporting crates:

- `crates/ff-vfs` -- the VFS abstraction (`provider.rs`, `registry.rs` keyed by
  scheme, `storage_provider.rs`, `transaction.rs`, `workspace.rs`). This is the
  right seam for a future Volume/StorageProvider split but currently has no volume
  notion.
- `crates/ff-dsalloc` -- JCL DD-statement parsing and allocation
  (`dd_statement.rs`, `allocation.rs`, `operands.rs`, `gdg_resolver.rs`,
  `temp_registry.rs`, `catalog_bridge.rs`). It parses `VOL=SER`-style operands into
  strings but there is no volume entity to resolve them against; it bridges to the
  catalog for `resolve_dsn` / `create_dataset`.
- `crates/ff-dataset-catalog/src/traits.rs` -- the ownership-model trait surface.
  It carries `volser: Option<String>` as a free-text attribute on dataset info, not
  a relationship to a Volume entity. (Note the two similarly named crates:
  `ff-dataset-catalog` is the trait/ownership layer named by ADR-001 in
  `dataset-lifecycle.md`; `ff-dscatalog` is the concrete SQLite implementation. The
  dataset-catalog spec notes this rename.)
- `crates/ff-idcams` -- parses `DEFINE CLUSTER ... VOLUMES(...)`, `ALTER
  ADDVOLUMES/REMOVEVOLUMES`, `LISTCAT ... VOLUME`. Again, VOLSERs are captured as
  `Vec<String>` and never resolved to a volume object (`src/parser/mod.rs`,
  `src/parser/ast.rs`, `src/services.rs`, `src/executor/handlers.rs`).
- `crates/ff-jes` -- the ONLY place a volume-like struct exists: `SpoolVolume
  { volser, total_tracks, used_tracks }` for the SDSF SP panel
  (`src/sdsf_system_panels.rs`). It is a JES spool display concept, unrelated to
  the dataset storage layer, but it shows the team already finds VOLSER natural.

### 3.3 Where the conflation is, precisely

The conflation is in the 1:1 identity Catalog == Repository (physical container):

- The catalog's own `storage_path` column points straight at bytes on disk. There
  is nothing in between to say "this dataset is on volume PROD01" vs "on WORK01".
- You cannot have one dataset catalogued in catalog A but physically on a volume
  shared with catalog B. In the real model, many catalogs can register datasets
  that live on the same volume, and one dataset can span several volumes
  (`FFWB_Storage_and_Catalog_Data_Model.md` sections 5, 15). The current code
  cannot express either.
- You cannot have an uncataloged dataset (one that exists on a volume with no
  catalog entry), because the catalog IS the volume -- no entry means no file.
- Volume-level operations (online/offline, mount/vary, capacity, VTOC listing,
  VOL=SER direct allocation, space/extents) have nowhere to attach.

---

## 4. Is the owner right? YES, with a split (not a rename)

Direct answer to question 3: YES, a distinct Volume layer is needed; NUANCED on the
"convert the catalog into a Volume" wording.

Do not rename Catalog -> Volume. Instead split today's single object:

- The physical **Repository** that each catalog currently owns becomes a **Volume**
  (gains a VOLSER, capacity, status; keeps the `storage/ pds/ gdg/` physical layout).
- The **Catalog** stays, but becomes metadata only: it maps DSN -> (VOLSER, locator)
  and no longer owns the bytes. Dataset `storage_path` is replaced by a
  (volume, locator) pair.

Why split rather than rename:

- A straight rename would keep the 1:1 Catalog/Volume binding and so would NOT fix
  the multi-catalog-per-volume, multi-volume-per-dataset, or uncataloged-dataset
  gaps. It would just relabel the problem.
- The source design already prescribes the split (ADR-001 + ADR-002). Renaming
  contradicts ADR-002 (catalogs must not contain data).
- The split is the smaller long-term change: it lets GDG, VSAM, DD/DISP allocation,
  IDCAMS `DEFINE VOLUME`/`DEFINE CLUSTER VOLUMES(...)`, and SDSF-style volume
  utilisation all hang off one coherent model instead of being faked with strings.

The one place to be pragmatic: for a single-user desktop, the Volume can start as a
"logical volume" that is simply a named Repository directory with a VOLSER and a
status flag. Full track/cylinder geometry, extents, and multivolume spanning can be
phased in later (the source model's Phase 1 vs Phase 4 split,
`FFWB_Storage_and_Catalog_Data_Model.md` section 22, supports exactly this).

---

## 5. Proposed target model

### 5.1 Entities and responsibilities

- **StorageSystem** (optional, 1 per workspace) -- names the environment and the
  default master catalog. Can be implicit initially.
- **Volume** -- the physical container. Fields: `volume_id`, `volser` (unique),
  `storage_uri` (the host directory that is today's Repository root), `status`
  (Online/Offline), `access_mode` (RW/RO), capacity counters. Owns the physical
  `storage/ pds/ gdg/ temp/` layout. This is the promoted Repository.
- **VTOC view** (not necessarily a stored table at first) -- "what datasets/extents
  live on this volume", derived from DatasetVolume + Extent rows. Backs a future
  volume-listing / VTOC panel and VOL=SER direct allocation.
- **Dataset** (catalog entry) -- the logical identity: `dataset_id`, `dsn` (unique
  in catalog scope), `dsorg`, `recfm/lrecl/blksize`, lifecycle state. No longer
  carries a raw `storage_path`.
- **DatasetVolume** -- association from a Dataset to one or more Volumes with a
  sequence number and an opaque per-volume locator (replaces today's
  `storage_path`). Single-volume datasets have exactly one row.
- **Catalog** -- metadata locator: maps DSN -> Dataset entry; records which
  Volume(s) via DatasetVolume. Master/user types + aliases route HLQ -> catalog
  (phase them in; `scope` column already exists as a seed).
- **DD binding** -- a runtime bind of DDNAME -> Dataset (or temp dataset) + DISP +
  access intent, owned by `ff-dsalloc` / an execution context (already partly
  modelled; `dd_statement.rs`, `temp_registry.rs`).

### 5.2 Entity relationship diagram (proposed)

```text
StorageSystem
  |
  |--< Volume (VOLSER, storage_uri, status, capacity)        [promoted Repository]
  |       ^   ^
  |       |   |  (VTOC view = datasets/extents on this volume)
  |       |   |
  |--< Catalog (master/user, metadata only) ----< CatalogAlias (HLQ -> catalog)
  |       |
  |       |--< Dataset (DSN, DSORG, RECFM/LRECL/BLKSIZE, lifecycle)
  |               |
  |               |--< DatasetVolume (seq, locator) >---- Volume     [many-to-many]
  |               |--< Extent (seq, size, locator) >----- Volume     [phase 4]
  |               |--o GdgBase --< GdgGeneration
  |               |--o VsamCluster (KSDS/ESDS/RRDS/LDS)
  |               |--o PartitionedDataset --< Member
  |
  (runtime) DD binding: DDNAME -> Dataset|TempDataset + DISP + intent  [ff-dsalloc]
```

Key relationships: a Dataset is registered in exactly one Catalog scope but may
reside on one or more Volumes; a Volume may hold datasets registered in different
catalogs; the Catalog never owns bytes (ADR-002).

### 5.3 Migration from the current model

The current `catalog.db` + Repository maps cleanly onto the new model, so migration
is mechanical and low-risk:

1. For each existing mounted catalog/Repository, define a Volume whose
   `storage_uri` is the existing Repository root and assign it a VOLSER (default:
   derive from the catalog name, or prompt once). Status = Online.
2. Add schema v4: a `volumes` table and a `dataset_volumes` table; add
   `nullable volser` + migrate: for every row in `datasets`, insert a
   `dataset_volumes` row pointing at the new Volume with `locator = storage_path`.
3. Keep `storage_path` readable during a transition window (dual-read) so existing
   resolution keeps working; switch `resolve_dsn` to go Dataset -> DatasetVolume ->
   Volume -> locator.
4. Later (phase 4) introduce Extent rows and multivolume support; `storage_path`
   can then be dropped.

Because physical object names in the source model are opaque and renames are
metadata-only (ADR-003, `FFWB_Storage_and_Catalog_Data_Model.md` section 23), no
dataset bytes need to move during migration -- only metadata rows are added.

---

## 6. Spec sub-projects that need new/changed requirements

Recommendations only -- no spec text written here. Each item would go through the
requirements gate.

- **NEW sub-project `volume-model`** -- WARRANTED. Owns the Volume entity, VOLSER
  rules, volume status (online/offline, RW/RO), capacity accounting, the
  VTOC/volume-listing view, `DEFINE VOLUME`, and VOL=SER direct allocation. This is
  a cohesive new domain that does not belong inside the catalog spec. It should
  cite ADR-001/ADR-002 from `FFWB_Storage_and_Catalog_Data_Model.md` as its source.
  (Create the folder under `docs/specs/` and add it to the list in
  `.kiro/steering/specs.md` per the workflow.)

- **`dataset-catalog`** (`docs/specs/dataset-catalog/requirements.md`) -- CHANGED.
  Redefine Catalog as a metadata locator (not a SQLite-DB-plus-Repository); redefine
  Repository's physical role as belonging to a Volume; replace dataset
  `storage_path` with a Dataset -> Volume(+locator) association; add schema v4
  requirements for `volumes` and `dataset_volumes`. This is the largest change and
  touches `ff-dscatalog`'s schema, repository, catalog, and vfs_provider modules.

- **`virtual-catalog-manager`** (`docs/specs/virtual-catalog-manager`) -- CHANGED.
  The Files UI needs a Volume concept: creating a Mainframe catalog should let the
  user pick/define the Volume it registers datasets on (instead of "Repository
  Path" being the whole story); add a volume view / VTOC listing; dataset
  allocation gains an optional VOLSER field.

- **`dataset-allocator`** (`docs/specs/dataset-allocator`) and the `ff-dsalloc`
  crate -- CHANGED. DD allocation and DISP should resolve/record the target Volume;
  VOL=SER + UNIT direct (uncataloged) allocation becomes resolvable; temp/work
  datasets allocate on a configured work Volume (or volume group).

- **`idcams-emulator`** (`docs/specs/idcams-emulator`) -- CHANGED. `DEFINE
  CLUSTER ... VOLUMES(...)`, `ALTER ADDVOLUMES/REMOVEVOLUMES`, `LISTCAT ... VOLUME`,
  and a new `DEFINE VOLUME` (FFWB extension) should bind to real Volume entities
  rather than storing VOLSER strings.

- **`virtual-file-system`** (`docs/specs/virtual-file-system`) -- POSSIBLY CHANGED
  / confirm only. The VFS StorageProvider seam is where a Volume maps to a
  storage-provider URI (ADR-001). Likely a small addition so a provider can be
  addressed per-Volume; confirm it does not need to change its scheme model.

- **`dataset-ownership-model`** (`docs/specs/dataset-ownership-model`) -- CHANGED
  (governance). ADR-001 for FFWB currently assigns ownership of catalog + physical
  storage to `ff-dscatalog`. Introducing Volumes means an ADR amendment naming the
  owner of the Volume entity (either a new `ff-volume` crate or `ff-dscatalog`), its
  permitted dependencies, and the invariant that catalogs never own bytes. The doc
  already defines the ADR-amendment process for exactly this
  (`dataset-lifecycle.md`, "Future Extensibility").

- **`jes-emulator`** (`docs/specs/jes-emulator`) -- MINOR / align. `ff-jes` already
  has `SpoolVolume`; worth aligning naming so spool volumes and storage volumes
  share VOLSER conventions, but no functional dependency.

- **`jcl-resolver`** (`docs/specs/jcl-resolver`) -- currently an empty stub
  (`.gitkeep` only). When it is fleshed out it will consume the Volume model via
  DD/DISP resolution; no change needed now beyond noting the dependency.

- Crates expected to change if the above gates pass: `ff-dscatalog` (schema,
  repository, catalog, resolution, vfs_provider), `ff-dsalloc`, `ff-idcams`,
  possibly a new `ff-volume` crate, and `ff-dataset-catalog` traits.

---

## 7. Open questions / decisions for the owner (resolve before any gate)

1. **Logical vs geometric volume.** Start Volumes as named directories with a
   VOLSER + status + byte capacity (recommended for phase 1), or model
   tracks/cylinders/extents and a geometry profile from day one? Recommendation:
   logical first, geometry in a later phase (matches source Phase 1 vs Phase 4).

2. **Separate crate or not.** Put the Volume entity in a new `ff-volume` crate, or
   extend `ff-dscatalog`? Recommendation: a thin `ff-volume` crate so the catalog
   depends on volumes (keeping the DAG acyclic and ADR-002 enforceable), but this is
   an architecture decision that needs owner confirmation per the ownership model.

3. **Catalog-to-volume cardinality for phase 1.** Allow many catalogs per volume and
   many volumes per dataset immediately, or keep 1 catalog : 1 volume at first and
   only break the binding later? Recommendation: break the binding in the schema
   (so uncataloged + shared volumes become possible) but it is fine for the UI to
   default to one-volume-per-catalog initially.

4. **Master catalog + aliases.** Introduce master/user catalog + alias routing now
   (the `scope` column is already seeded), or defer? This affects how `volume-model`
   and `dataset-catalog` requirements are scoped.

5. **Uncataloged datasets.** Confirm FFWB should support a dataset existing on a
   volume with no catalog entry (VOL=SER + UNIT access). This is authentic but adds
   a resolution path; it is the clearest justification for the split over a rename.

6. **Migration window.** Confirm the dual-read transition (keep `storage_path`
   while `dataset_volumes` is populated) is acceptable, and who assigns VOLSERs to
   existing catalogs on upgrade (auto-derived vs prompted).

7. **Naming in the UI.** Decide user-facing terminology: do end users see
   "Volume" and "Catalog" as separate things in the Files panel, or is Volume an
   advanced/admin concept hidden behind catalog creation? This shapes the
   `virtual-catalog-manager` change.

---

## 8. Documents and code actually read (citations)

Source documents (all under `docs/source-documents/dataset-catalog/`):

- `FFWB_Storage_and_Catalog_Data_Model.md` (full) -- the authoritative target
  model; Volume/catalog entities, resolution algorithm, ADR-001..006, phases.
- `FileForgeWorkbench_Virtual_File_and_Dataset_Storage_Requirements.md` (full) --
  hybrid SQLite-catalogue/native-file architecture; AR-PR-002 catalogue
  independence; FFW-VFS-* requirements.
- `FileForgeWorkbench_Mainframe_Dataset_Architecture.md` (full) -- record
  semantics, dataset-type physical storage matrix, GDG/VSAM.
- `dataset-lifecycle.md` (full) -- create/use/modify/delete ownership and the
  DSN -> physical resolution lifecycle; ADR amendment process.
- `FileForgeWorkbench_DDNAME_Mapping_Design_v1.1.md` (sections 1-6 read) -- DDNAME
  -> dataset/stream binding, DISP, standard DD registry.
- `virtual-file-catalogs.md` (full) -- current three-catalog-type UI model and the
  Repository-backed Mainframe catalog layout.

Specs (under `docs/specs/`):

- `dataset-catalog/requirements.md` (intro + glossary read) -- current ratified
  definition of Catalog/Repository/Resolution.
- `virtual-catalog-manager/requirements.md` (intro read) -- Files UI, catalog-as-
  scheme model.
- `dataset-ownership-model/requirements.md` (searched; LISTCAT VOLUME reference).
- Folder presence confirmed for `virtual-file-system`, `dataset-allocator`,
  `idcams-emulator` (full specs), `jcl-resolver` (empty stub, `.gitkeep` only).

Code (under `crates/`):

- `ff-dscatalog/src/schema.rs` -- datasets table has `storage_path`, no volume
  table/column; SCHEMA_VERSION 3.
- `ff-dscatalog/src/repository.rs` -- Repository = physical container directory.
- `ff-dscatalog/src/catalog.rs` -- Catalog owns one Repository + SQLite conn;
  CatalogLocation Local/Remote.
- `ff-dscatalog/src/lib.rs` -- documents DSN -> physical file mapping.
- `ff-vfs/src/*` -- VFS provider/registry/storage_provider seam (no volume notion).
- `ff-dsalloc/src/*` -- DD statement + allocation; VOL operands parsed as strings.
- `ff-dataset-catalog/src/traits.rs` -- `volser: Option<String>` as a free-text
  attribute, not a relationship.
- `ff-idcams/src/parser,ast,services,executor` -- VOLUMES parsed into Vec<String>.
- `ff-jes/src/sdsf_system_panels.rs` -- `SpoolVolume { volser, ... }` (JES spool
  display only).

---

## 9. One-paragraph recommendation

Promote the physical Repository that each Mainframe catalog currently owns into a
first-class Volume (VOLSER, status, capacity, the `storage/ pds/ gdg/` layout), and
reduce the Catalog to a pure DSN -> (Volume, locator) metadata locator, exactly as
the project's own `FFWB_Storage_and_Catalog_Data_Model.md` ADR-001 and ADR-002
already prescribe. Do this as a split, not a rename, so FFWB can finally express
many-catalogs-per-volume, multivolume datasets, and uncataloged (VOL=SER) datasets
-- the capabilities a JCL/DD/DISP environment needs. Create a new `volume-model`
spec sub-project, change the `dataset-catalog`, `virtual-catalog-manager`,
`dataset-allocator`, `idcams-emulator`, and `dataset-ownership-model` specs, and
migrate by adding `volumes`/`dataset_volumes` tables while dual-reading the existing
`storage_path` so no dataset bytes move. Start logical (named volume directories),
add geometry/extents/multivolume later.
