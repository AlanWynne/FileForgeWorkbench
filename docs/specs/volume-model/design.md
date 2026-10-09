# Design Document: Volume Model (`ff-volume`)

## Overview

The Volume Model introduces a first-class mainframe **Volume** layer beneath the dataset catalog, delivered as a new thin crate `ff-volume`. It realises the "split, not rename" direction (CR-NR-105 / CR-CH-057): the physical Repository that each catalog currently owns is promoted into a Volume (gaining a VOLSER, status, access mode, and capacity counters), and the Catalog is reduced to a pure metadata locator that maps a DSN to a Volume plus an opaque locator.

This design implements:
- The Volume entity, VOLSER identity, status, and access mode (Requirements 1, 2).
- A configurable, deterministic emulated Geometry_Profile for byte/track/cylinder accounting (Requirement 3).
- A space-allocation unit model (TRK/CYL/block forms) consumed by `ff-dsalloc` (Requirement 4).
- Extent accounting with primary plus up to a configurable maximum of secondary extents, and the two distinct failure points -- the dataset x37-style capacity failure and the Volume_Full failure (Requirements 5, 6, 7).
- Tracks/cylinders/extents reporting for datasets and Volumes, and a VTOC_View (Requirement 8).
- The DatasetVolume association, multivolume support, shared-Volume cardinality, and uncataloged (VOL=SER + UNIT) resolution (Requirement 9).
- The `DEFINE VOLUME` command contract and Volume visibility (Requirement 10).

> **Source:** adapted from `docs/source-documents/dataset-catalog/FFWB_Storage_and_Catalog_Data_Model.md` (ADR-001, ADR-002, ADR-003, ADR-005; sections 7.2, 7.3, 7.9, 7.10, 7.11, 7.12, 9, 14, 15, 18.2, 20, 22) and the approved recommendation (`.agents/tasks/volume-model-recommendation/recommendation.md`, sections 4, 5.1, 5.2, 5.3, 6). Content was rephrased for compliance with licensing restrictions.

## Entity Model

The split introduces a layer between the Catalog (locator) and the host filesystem (bytes). The responsibilities are:

- **StorageSystem** (optional, one per workspace) -- names the environment and the default master catalog. May be implicit initially.
- **Volume** -- the physical container. Fields: volume_id, VOLSER (unique), optional display name, storage_uri (today's Repository root), Volume_Status (Online/Offline), Access_Mode (ReadWrite/ReadOnly), capacity counters (total/used/reserved in tracks or cylinders), optional volume_group_id. Owns the physical `storage/ pds/ gdg/ temp/` (or UUID `datasets/objects/`) layout. This is the promoted Repository.
- **Geometry_Profile** -- configurable `bytes_per_track` (default a 3390-style value) and `tracks_per_cylinder` (default 15). Deterministic conversions for ACCOUNTING ONLY.
- **Extent** (logical) -- a recorded allocation segment: dataset_id, volume_id, sequence_number, allocated_units, used_bytes, opaque physical-object locator. Accounting metadata, not physical layout.
- **DatasetVolume** -- association from a Dataset to one or more Volumes: dataset_id, volume_id, sequence_number, is_primary, opaque per-volume locator. Replaces the dataset's raw `storage_path`.
- **VTOC_View** -- a derived read view (not necessarily a stored table initially) of the datasets and extents on a Volume plus the Volume's usage counters.
- **Dataset** (catalog entry) -- the logical identity (DSN, DSORG, RECFM/LRECL/BLKSIZE, lifecycle). It no longer carries a raw storage_path; location is via DatasetVolume. Owned by `ff-dscatalog`, not `ff-volume`.
- **Catalog** -- metadata locator. Maps DSN to a Dataset entry and records which Volume(s) via DatasetVolume. Never owns bytes (ADR-002). Owned by `ff-dscatalog`.
- **DD binding** -- a runtime bind of DDNAME to a Dataset (or temporary dataset) plus DISP and access intent. Owned by `ff-dsalloc` / an execution context at runtime, not persisted by `ff-volume`.

### Entity Relationship Diagram (proposed)

Adapted from recommendation section 5.2. Box-drawing appears only inside this fenced code block.

```text
StorageSystem
  |
  |--< Volume (VOLSER, storage_uri, status, access_mode, capacity)   [promoted Repository]
  |       ^   ^
  |       |   |  (VTOC_View = datasets + extents on this volume, derived)
  |       |   |
  |--< Catalog (master/user, metadata only) ----< CatalogAlias (HLQ -> catalog)
  |       |
  |       |--< Dataset (DSN, DSORG, RECFM/LRECL/BLKSIZE, lifecycle)
  |               |
  |               |--< DatasetVolume (seq, is_primary, locator) >---- Volume   [many-to-many]
  |               |--< Extent (seq, allocated_units, used_bytes) >--- Volume   [accounting]
  |               |--o GdgBase --< GdgGeneration
  |               |--o VsamCluster (KSDS/ESDS/RRDS/LDS)
  |               |--o PartitionedDataset --< Member
  |
  (runtime) DD binding: DDNAME -> Dataset|TempDataset + DISP + intent   [ff-dsalloc]
```

Key relationships: a Dataset is registered in exactly one Catalog scope but MAY reside on one or more Volumes; a Volume MAY hold datasets registered in different catalogs; an Uncataloged_Dataset resides on a Volume with no Catalog row; the Catalog never owns bytes (ADR-002).

## Crate Ownership (owner decision 5)

- The Volume entity, VOLSER identity, Volume_Status/Access_Mode, Geometry_Profile, Extent accounting, Volume capacity counters, and the DatasetVolume association model are owned by a NEW thin crate `ff-volume`.
- `ff-dscatalog` SHALL depend on `ff-volume` and obtain Volume data through its API; it SHALL NOT redefine the Volume type.
- `ff-volume` SHALL depend only on narrow model crates (and `ff-vfs` for the StorageProvider URI seam); it SHALL NEVER depend on `ff-dscatalog`, `ff-dsalloc`, or `ff-idcams`. This keeps the dependency DAG acyclic and makes ADR-002 (catalogs never own bytes) enforceable by construction: the catalog depends on volumes, volumes never depend on the catalog.
- **DO NOT create the `ff-volume` crate in this gate.** This is a documentation-only gate; the owner confirms the crate again at implementation. The dataset-ownership-model Requirement 21 (CR-CH-057) records the ownership boundary.

## Deterministic Geometry and Space Accounting

### Geometry convention

- `Geometry_Profile { bytes_per_track, tracks_per_cylinder }` with default `tracks_per_cylinder = 15` and a documented default `bytes_per_track` (a 3390-style value). The exact default byte value is a configurable constant fixed in `ff-volume`.
- Conversions (ACCOUNTING ONLY, deterministic):
  - `tracks(bytes) = ceil(bytes / bytes_per_track)`
  - `cylinders(tracks) = ceil(tracks / tracks_per_cylinder)`
  - `bytes_of(tracks) = tracks * bytes_per_track`
- Worked example (illustrative; the shipped default byte value is documented at implementation): with `bytes_per_track = 56664` and `tracks_per_cylinder = 15`, a 1,000,000-byte dataset needs `ceil(1000000 / 56664) = 18` tracks, i.e. `ceil(18 / 15) = 2` cylinders.

### Extent accounting

- A dataset's allocated space is the sum of its extents' `allocated_units` (converted to tracks/cylinders via the Geometry_Profile). The primary extent is sized from the SPACE primary quantity; each secondary extent from the SPACE secondary quantity.
- Used space (`used_bytes` summed across extents) is compared against allocated space to detect overflow.
- Reporting (Requirement 8) is derived: extents-used = count of extent rows; extents-remaining = `Max_Extents - extents_used`; tracks/cylinders in use = geometry conversion of allocated/used units.

### How an overflowing write becomes a reported failure

1. A write/append computes the new `used_bytes` for the dataset.
2. IF new used space fits within the current allocated space, the write proceeds.
3. ELSE the system attempts to acquire a secondary extent:
   - IF `extents_used < Max_Extents` AND the target Volume has free space for the secondary quantity, a secondary extent is added and the write proceeds (Requirement 5.2, 6.2).
   - IF `extents_used == Max_Extents`, the system returns a reported x37-style Space_Abend (B37/D37/E37 analogue) identifying the dataset and the maximum-extents reason; prior committed content is intact (Requirement 5.3, 5.4, 6.3).
   - IF an extent could be added by count but the Volume lacks free space, the system returns a reported Volume_Full error identifying the Volume and does NOT consume a dataset extent (Requirement 7.2, 7.4).

### Failure model mapping

The two distinct failure points map onto the source Error Model (`FFWB_Storage_and_Catalog_Data_Model.md` section 20):

| Failure | Cause | Source error (section 20) |
|---|---|---|
| x37-style Space_Abend | Dataset exceeded allocatable capacity / reached Max_Extents | `StorageError::InsufficientSpace` (dataset scope) |
| Volume_Full | Target Volume lacks free space | `StorageError::InsufficientSpace` (volume scope) |
| Volume offline | Allocation on an Offline Volume | `StorageError::VolumeOffline` |
| Volume read-only | Write/delete/extend on a ReadOnly Volume | `StorageError::VolumeReadOnly` |
| VOLSER not found | VOL=SER resolution misses | `StorageError::VolumeNotFound` |

`ff-volume` exposes these as distinct variants so callers (and IDCAMS/JCL return-code mapping) can tell the two space causes apart.

## Volume-to-VFS Mapping (ADR-001)

A Volume maps to a VFS StorageProvider URI through its `storage_uri`, building ON the existing `ff-vfs` StorageProvider seam (see `crates/ff-vfs`, read-only; this spec does not modify it). The Volume is a storage locator target, not a new navigation stack or dispatch path. No change to the `ff-vfs` scheme model is required by this specification; a confirming "possibly changed" review of `virtual-file-system` is a future gate (recommendation section 6).

## Framework-Conformance Statement

- **Single command dispatch:** `DEFINE VOLUME`, volume listing / VTOC_View, mount/vary, and VOL=SER allocation are COMMANDS resolved through the single command-dispatch path (`resolve_target` / `dispatch_command_target`, or the `handle_command` ladder until the verb table lands). No bespoke `if upper == "..."` intercept and no second dispatcher.
- **WorkspaceContext:** any Volume/VTOC listing surfaced as a Workspace Context is a `WorkspaceContext` dispatched via `render_workspace_context`, returning `InteriorFocus` (stable first-control id). No per-arm focus ring.
- **Navigation and persistence:** navigation uses the existing per-tab `navigate_to`; a persisted Volume view uses a `WorkspaceDescriptor` (`CustomWorkspace { kind, params }`). No second navigation stack, no new persistence format.
- Most Volume UI criteria (Volume-aware catalog creation, VTOC listing panel) live in `virtual-catalog-manager`, which is OUT OF SCOPE for this gate and edited under a future gate.

## Migration Approach

From recommendation section 5.3 and dataset-catalog Requirement 32:

1. Schema v4 (owned by `ff-dscatalog`) adds a `volumes` table and a `dataset_volumes` table.
2. For each existing mounted catalog/Repository, define a Volume whose `storage_uri` is the current Repository root and assign it a VOLSER (derived from the catalog name or prompted once); status = Online.
3. For every existing `datasets` row, insert a `dataset_volumes` row pointing at the new Volume with `locator = storage_path`.
4. Keep `storage_path` readable during a transition window (dual-read) so existing resolution keeps working; switch resolution to go Dataset -> DatasetVolume -> Volume -> locator.
5. NO dataset bytes move during migration -- it is metadata only (ADR-003; Requirement 11.3).

## Deferred / Later Phase

Aligned with the source phase split (`FFWB_Storage_and_Catalog_Data_Model.md` section 22, Phase 1 vs Phase 4):
- Phase-1 Volume is a named directory with a VOLSER, status, access mode, and capacity counters; single-volume datasets.
- Full extent-object mapping (one physical object per extent) and multivolume spanning of a single logical record stream are a later phase. The model records multivolume DatasetVolume rows now; the spanning access layer is Phase 4.
- Volume_Group allocation policies (first-fit, best-fit, round-robin) are model-level only in this specification and are elaborated later.

## No Design Change Needed

- No change to the `ff-vfs` StorageProvider scheme model is required by this spec (confirm-only, future gate).

---

## Design Delta: DatasetAccess Resolves Location Through ff-volume (CR-CH-059, Requirement 12)

This delta is a REFERENCE, not a new Volume model. The Volume entity, VOLSER, status/access mode, geometry, extents, capacity, VTOC_View, DatasetVolume association, and schema v4 migration are already designed in the sections above and in [dataset-catalog](./../dataset-catalog/design.md) (Volume Split, CR-CH-057). CR-CH-059 adds only the statement that the new `DatasetAccess` contract (dataset-catalog Requirement 34) resolves a dataset's physical location through the existing `ff-volume` structures:

- `DatasetAccess::allocate`/`open` resolve location via Dataset -> DatasetVolume (by sequence) -> Volume -> locator (dataset-catalog Requirement 32.3; this spec Requirement 9), honouring the Volume Online check (Requirement 2.4) and charging SPACE against the Volume (dataset-allocator Requirement 17) with the two distinct failures (dataset x37-style per Requirements 5-6; Volume_Full per Requirement 7) unchanged.
- The legacy `storage_path` migrates to the `DatasetVolume` locator at schema v4 (dataset-catalog Requirement 32.5); `DatasetAccess` reads the `DatasetVolume` locator, not `storage_path`.
- `ff-volume` stays the Volume owner and does NOT depend on `ff-dscatalog`/`ff-dsalloc`/`ff-idcams` (dataset-ownership-model Requirement 21.3); `DatasetAccess` lives in `ff-dscatalog` and CONSUMES `ff-volume`, keeping the DAG acyclic.

The DatasetAccess trait shape itself is in [dataset-catalog](./../dataset-catalog/design.md); this spec adds no new Volume behaviour or failure mode.
- No change to the single command-dispatch mechanism, the per-tab navigation stack, the `WorkspaceContext`/`InteriorFocus` focus latch, or the `WorkspaceDescriptor` persistence model -- this spec builds ON them unchanged.
