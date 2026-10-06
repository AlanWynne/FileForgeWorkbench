# Requirements Document

> **Governance Reference:** This specification is governed by the [Dataset Ownership Model](./../dataset-ownership-model/requirements.md) (ADR-001, and the ff-volume ownership boundary added by CR-CH-057). Where this document conflicts with the governance document, the governance document takes precedence. The Volume entity, VOLSER identity, volume status/access-mode, emulated geometry, extent accounting, volume-capacity counters, and the DatasetVolume association are owned by the new `ff-volume` crate on which `ff-dscatalog` depends.

## Introduction

This feature specifies the Volume Model subsystem for FileForgeWorkbench (planned `ff-volume` crate). It introduces a first-class mainframe **Volume** layer that the "split, not rename" direction (CR-NR-105 / CR-CH-057) adds beneath the dataset catalog.

Today a FFWB "catalog" is a SQLite database bound 1:1 to a physical Repository directory that BOTH locates AND physically stores datasets. That single object conflates two distinct mainframe concepts: the **Volume** (the physical DASD-style container, identified by a VOLSER) and the **Catalog** (a name-to-location locator that holds metadata only). This specification promotes the physical Repository into a first-class Volume and leaves the Catalog as a pure metadata locator. A dataset "lives on" one or more Volumes; the Catalog merely records which Volume(s) and the opaque per-volume locator.

This design realigns FFWB with its own source design document `docs/source-documents/dataset-catalog/FFWB_Storage_and_Catalog_Data_Model.md`:
- **ADR-001** (that doc, section 23): Volumes SHALL be modelled explicitly as first-class domain objects and mapped to storage-provider URIs.
- **ADR-002** (that doc, section 23): Catalogs SHALL hold metadata references to datasets and their locations and SHALL NOT physically contain the dataset data.
- **ADR-005** (that doc, section 23): tracks and cylinders SHALL be supported as EMULATED logical units converted through a geometry profile, not physical CKD device geometry.

A central scoping decision (owner-approved): **tracks, cylinders, and extents are METADATA ONLY**. FFWB does not simulate physical disk geometry or on-disk layout. These units serve two purposes:
1. A **capacity constraint** -- a dataset has an allocated space (primary plus secondary extents); a write or append that would exceed the dataset's allocatable capacity, or exhaust its secondary extents, or exceed its maximum extent count, MUST fail as a cleanly reported space-abend-style error (emulating the z/OS x37 abend family B37/D37/E37), never a crash.
2. **Reporting** -- the system reports tracks, cylinders, and extents in use and remaining, for both datasets and Volumes (a VTOC-style volume listing).

**Framework conformance:** A Volume maps to a VFS StorageProvider URI (ADR-001), building ON the existing `ff-vfs` StorageProvider seam. Volume-management actions (DEFINE VOLUME, volume listing/VTOC, mount/vary, VOL=SER allocation) are COMMANDS resolved through the single command-dispatch path. Any Volume/VTOC listing surfaced as a Workspace Context is a `WorkspaceContext` dispatched via `render_workspace_context`; any persisted Volume view uses a `WorkspaceDescriptor`. This specification introduces no new navigation stack, no second dispatcher, and no new persistence format.

### Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `dataset-catalog` | **Consumer** | The catalog depends on `ff-volume`; it resolves a DSN to a Volume plus locator and stores `volumes` and `dataset_volumes` tables (schema v4). See dataset-catalog Requirement 32. |
| `dataset-ownership-model` | **Governance** | ADR amendment (Requirement 21) names `ff-volume` as the Volume owner and restates ADR-002 (catalogs never own bytes). |
| `virtual-file-system` | **Dependency** | A Volume maps to a VFS StorageProvider URI (ADR-001). Build ON the existing `ff-vfs` StorageProvider seam; no change to its scheme model is required by this spec. |
| `dataset-allocator` | **Future consumer** | DD/DISP/SPACE allocation records and resolves the target Volume; VOL=SER + UNIT direct (uncataloged) allocation becomes resolvable. Edited under a future gate. |
| `idcams-emulator` | **Future consumer** | `DEFINE VOLUME`, `DEFINE CLUSTER ... VOLUMES(...)`, `ALTER ADDVOLUMES/REMOVEVOLUMES`, `LISTCAT ... VOLUME` bind to real Volume entities. Edited under a future gate. |

> **Coordinates with (future gates):** `virtual-catalog-manager` (Volume-aware catalog creation + VTOC listing UI), `dataset-allocator`, `idcams-emulator`, `virtual-file-system`, `jes-emulator` (SpoolVolume VOLSER alignment), and `jcl-resolver`. Those specs are NOT edited by this gate; they are named here to record the dependency only.

> **Source:** `docs/source-documents/dataset-catalog/FFWB_Storage_and_Catalog_Data_Model.md` (ADR-001, ADR-002, ADR-005; sections 7.2 Volume, 7.3 Volume Group, 7.9 Allocation Definition, 7.10 Dataset Volume Association, 7.11 Extent, 9 Catalog Resolution Algorithm, 14 Volume Capacity, 15 Multivolume Datasets, 18.2 Volume validation, 20 Error Model, 22 Phases, 11.1 DEFINE VOLUME). Owner-approved scoping decisions 1-8 (CR-NR-105 / CR-CH-057). Content was rephrased for compliance with licensing restrictions.

## Glossary

- **Volume**: The FFWB equivalent of a mainframe DASD volume -- the first-class physical container that holds datasets. A Volume has a VOLSER, a storage URI (the host directory that is today's Repository root), a status, an access mode, and capacity counters. (ADR-001; source 7.2.)
- **VOLSER (Volume_Serial)**: The unique serial that identifies a Volume within an FFWB storage system. VOLSER uniqueness is enforced. (source 7.2, 18.2.)
- **Volume_Status**: The operational state of a Volume. For this specification the modelled values are Online and Offline (the source also lists Mounted/Unmounted/Recovering/Error as future states). An Offline Volume rejects new allocation. (source 7.2.)
- **Access_Mode**: Whether a Volume is ReadWrite or ReadOnly. A ReadOnly Volume rejects write, delete, and extend operations. (source 7.2.)
- **Volume_Group**: A named logical pool of Volumes for policy-driven allocation (first-fit, best-fit, round-robin, explicit-only). Model-level only in this specification. (source 7.3.)
- **Geometry_Profile**: A configurable conversion profile with a `bytes_per_track` value (default a 3390-style value) and `tracks_per_cylinder` (default 15). It converts bytes to tracks and cylinders for ACCOUNTING ONLY; no physical layout is simulated. (ADR-005; source 14.)
- **Track**: An emulated allocation unit equal to `bytes_per_track` bytes under the active Geometry_Profile. Accounting unit, not physical geometry.
- **Cylinder**: An emulated allocation unit equal to `tracks_per_cylinder` tracks (default 15). Accounting unit, not physical geometry.
- **Extent**: A logical allocation segment recorded as metadata for a dataset on a Volume. Extents account for allocated space; they do not describe physical on-disk placement. (source 7.11.)
- **Primary_Extent**: The initial allocation segment created when a dataset is allocated.
- **Secondary_Extent**: An additional allocation segment acquired when a dataset grows beyond its current allocated space.
- **Max_Extents**: The configurable maximum number of extents (primary plus secondary) a dataset may hold. Default 16 secondary extents.
- **Space_Abend (x37-style)**: A cleanly reported failure of a write/append/extend because the dataset exceeded its allocatable capacity or exhausted its allowed extents. It emulates the z/OS x37 abend family (B37/D37/E37) and is reported, never a crash. (source 20 InsufficientSpace.)
- **Volume_Full**: A cleanly reported failure of an allocation because the target Volume lacks sufficient free space. This is a DISTINCT failure point from the dataset-level Space_Abend. (source 14, 20.)
- **VTOC_View**: A derived, read listing of what datasets and extents reside on a Volume, together with the Volume's track/cylinder/extent usage. It plays the role of a z/OS VTOC. May be derived rather than a stored table initially. (source 7.10, 7.11.)
- **DatasetVolume**: The association from a Dataset to one or more Volumes, carrying a sequence number, an is-primary flag, and an opaque per-volume locator. It replaces the dataset's raw `storage_path`. Single-volume datasets have exactly one DatasetVolume row. (source 7.10.)
- **Uncataloged_Dataset**: A dataset that physically exists on a Volume with no catalog entry, resolvable only by explicit VOL=SER plus UNIT. (source 5, 7.10.)
- **VOL=SER + UNIT direct access**: The resolution path that locates an Uncataloged_Dataset by its Volume serial (VOL=SER) and device/UNIT without consulting a catalog.
- **Multivolume_Dataset**: A dataset associated with more than one Volume via an ordered sequence of DatasetVolume rows. (source 15.)
- **Storage_URI**: The provider-addressable location of a Volume (ADR-001). It maps to the existing `ff-vfs` StorageProvider seam and corresponds to today's Repository root host directory.

## Requirements

### Requirement 1: Volume as a First-Class Entity

**User Story:** As a mainframe developer, I want Volumes to be first-class storage containers with a unique VOLSER, so that datasets physically live on a named Volume rather than being conflated with a catalog.

**Source:** `FFWB_Storage_and_Catalog_Data_Model.md` ADR-001; sections 7.2, 18.2. Owner decisions 5, 6. Content was rephrased for compliance with licensing restrictions.

#### Acceptance Criteria

1.1 THE Volume entity SHALL carry at minimum: a volume_id, a VOLSER, an optional display name, a storage_uri (the host directory that is today's Repository root), a Volume_Status, an Access_Mode, and capacity counters (total, used, and reserved space in tracks or cylinders).
1.2 THE system SHALL enforce that a VOLSER is unique within a storage system -- no two Volumes SHALL share a VOLSER.
1.3 WHEN a Volume is defined over an existing Repository root, THE Volume SHALL own that root's physical layout (the `storage/`, `pds/`, `gdg/`, `temp/` directories, or the UUID `datasets/objects/` layout) rather than the Catalog owning it.
1.4 THE Volume entity SHALL map to a VFS StorageProvider via its storage_uri (ADR-001), building on the existing `ff-vfs` StorageProvider seam.
1.5 THE Volume entity SHALL be owned by the `ff-volume` crate; `ff-dscatalog` SHALL depend on `ff-volume` and SHALL NOT redefine the Volume type.

---

### Requirement 2: Volume Status and Access Mode

**User Story:** As a mainframe developer, I want a Volume to be Online or Offline and ReadWrite or ReadOnly, so that allocation and I/O respect volume availability as they do on z/OS.

**Source:** `FFWB_Storage_and_Catalog_Data_Model.md` sections 7.2, 9 (step 8), 10.1, 18.2. Owner decision 8. Content was rephrased for compliance with licensing restrictions.

#### Acceptance Criteria

2.1 WHEN a Volume's status is Offline, THE system SHALL reject any new allocation targeting that Volume with a reported error identifying the Volume as offline.
2.2 WHEN a Volume's access_mode is ReadOnly, THE system SHALL reject write, delete, and extend operations targeting that Volume with a reported error identifying the Volume as read-only.
2.3 THE system SHALL support status transitions set-online and set-offline, and mount and unmount, for a Volume.
2.4 WHEN resolving a dataset for I/O, THE system SHALL verify that each required Volume is Online, and SHALL return a reported error identifying the first unavailable Volume otherwise.
2.5 THE set-online, set-offline, mount, and unmount actions SHALL be invokable as commands through the single command-dispatch path.

---

### Requirement 3: Emulated Geometry (Tracks, Cylinders, Byte Conversions)

**User Story:** As a mainframe developer, I want tracks and cylinders to be emulated accounting units derived from a configurable geometry profile, so that JCL and IDCAMS space concepts work without simulating physical disk geometry.

**Source:** `FFWB_Storage_and_Catalog_Data_Model.md` ADR-005; section 14. Owner decision 2. Content was rephrased for compliance with licensing restrictions.

#### Acceptance Criteria

3.1 THE system SHALL define a Geometry_Profile with a configurable `bytes_per_track` (default a documented 3390-style value) and a `tracks_per_cylinder` value defaulting to 15.
3.2 THE system SHALL convert bytes to tracks by dividing by `bytes_per_track` and rounding up to a whole track, and SHALL convert tracks to cylinders by dividing by `tracks_per_cylinder` and rounding up to a whole cylinder.
3.3 THE byte-to-track, track-to-cylinder, and cylinder-to-byte conversions SHALL be deterministic -- the same inputs under the same Geometry_Profile SHALL always yield the same units -- and SHALL be documented in design.md.
3.4 THE geometry conversions SHALL be ACCOUNTING ONLY and SHALL NOT describe or constrain physical on-disk layout of the stored bytes.
3.5 WHEN the Geometry_Profile is changed, THE system SHALL apply the new profile to subsequent conversions and SHALL document that previously recorded unit counts reflect the profile active at allocation time.

---

### Requirement 4: Space Allocation Units and JCL SPACE Syntax

**User Story:** As a mainframe developer, I want to request dataset space using JCL-style SPACE in tracks, cylinders, or block/average-record units, so that allocation matches the syntax I use on z/OS.

**Source:** `FFWB_Storage_and_Catalog_Data_Model.md` section 7.9 (AllocationUnit). Owner decision 2. Content was rephrased for compliance with licensing restrictions.

#### Acceptance Criteria

4.1 THE system SHALL support a space-allocation request expressed in tracks, in the form `SPACE=(TRK,(primary,secondary))`.
4.2 THE system SHALL support a space-allocation request expressed in cylinders, in the form `SPACE=(CYL,(primary,secondary))`.
4.3 THE system SHALL support a space-allocation request expressed in block or average-record units, in the form `SPACE=(avgreclen,(primary,secondary))` together with an `AVGREC` modifier, and SHALL convert the requested quantity to allocation units via the Geometry_Profile.
4.4 THE `ff-volume` crate SHALL expose the allocation-unit model (unit kind plus primary and secondary quantities) that these SPACE forms map to; the parsing of the JCL SPACE keyword itself is owned by `ff-dsalloc` and consumes this model.
4.5 WHEN a SPACE request specifies a secondary quantity, THE system SHALL record it as the per-extent secondary allocation size used when the dataset grows.

---

### Requirement 5: Dataset Space Capacity Constraint (x37-style Failure)

**User Story:** As a mainframe developer, I want a dataset write that exceeds its allocated capacity to fail with a clean space-abend-style error, so that overflow is reported like a z/OS x37 abend rather than crashing or silently growing.

**Source:** `FFWB_Storage_and_Catalog_Data_Model.md` section 20 (InsufficientSpace). Owner decisions 1(a), 3. Content was rephrased for compliance with licensing restrictions.

#### Acceptance Criteria

5.1 THE allocated space of a dataset SHALL be its primary extent plus the secondary extents acquired so far, each sized from the SPACE request, measured in tracks or cylinders.
5.2 WHEN a write or append would exceed the dataset's current allocated space AND a further secondary extent is available (below Max_Extents and within Volume free space), THE system SHALL acquire a secondary extent to satisfy the write.
5.3 WHEN a write or append would exceed the dataset's allocatable capacity because no further secondary extent can be acquired (Max_Extents reached), THE system SHALL fail the operation with a cleanly reported space-abend-style error that emulates the z/OS x37 abend family (B37/D37/E37) and SHALL NOT crash.
5.4 THE reported x37-style error SHALL identify the dataset and the reason (allocatable capacity exceeded / maximum extents reached) and SHALL leave the dataset's prior committed content intact.
5.5 THE x37-style dataset-capacity failure SHALL be distinct from, and reported separately from, the Volume_Full failure defined in Requirement 7.

---

### Requirement 6: Secondary Extents and Maximum Extent Limit

**User Story:** As a mainframe developer, I want a dataset to grow through secondary extents up to a configurable maximum, so that extent exhaustion emulates the mainframe limit on dataset growth.

**Source:** `FFWB_Storage_and_Catalog_Data_Model.md` sections 7.9 (max_extents), 7.11. Owner decision 3. Content was rephrased for compliance with licensing restrictions.

#### Acceptance Criteria

6.1 THE system SHALL allocate a dataset with a primary extent plus up to a configurable maximum number of secondary extents, defaulting to 16 secondary extents.
6.2 WHEN a dataset needs to grow and the current extent count is below Max_Extents AND the Volume has free space, THE system SHALL add a secondary extent sized from the SPACE secondary quantity.
6.3 WHEN a dataset needs to grow but the extent count has reached Max_Extents, THE system SHALL fail with the x37-style space-abend error defined in Requirement 5 (maximum extents reached).
6.4 THE Max_Extents value SHALL be configurable, and the maximum-extent failure SHALL be distinct from the Volume_Full failure (Requirement 7).

---

### Requirement 7: Volume Capacity and Volume-Full Failure

**User Story:** As a mainframe developer, I want allocation to fail when a Volume lacks free space, reported distinctly from a dataset exceeding its own allocation, so that the two failure causes are not confused.

**Source:** `FFWB_Storage_and_Catalog_Data_Model.md` sections 14, 20. Owner decision 4. Content was rephrased for compliance with licensing restrictions.

#### Acceptance Criteria

7.1 THE system SHALL record a Volume's total capacity in tracks or cylinders, and SHALL track used and free capacity derived from the extents allocated on the Volume.
7.2 WHEN a requested allocation or extent acquisition requires more free space than the target Volume currently has, THE system SHALL fail with a cleanly reported Volume_Full error identifying the Volume.
7.3 THE Volume_Full failure SHALL be a DISTINCT and separately reported failure point from the dataset x37-style capacity failure (Requirement 5), so that a caller can tell which cause occurred.
7.4 WHEN an allocation would succeed against the dataset's extent limits but the Volume lacks free space, THE system SHALL report Volume_Full and SHALL NOT consume a dataset extent.

---

### Requirement 8: Reporting -- Tracks, Cylinders, and Extents In Use and Remaining

**User Story:** As a mainframe developer, I want to see how many tracks, cylinders, and extents a dataset and a Volume use and have remaining, so that I can monitor space like a VTOC listing on z/OS.

**Source:** `FFWB_Storage_and_Catalog_Data_Model.md` section 14. Owner decision 1(b). Content was rephrased for compliance with licensing restrictions.

#### Acceptance Criteria

8.1 THE system SHALL report, for a dataset: the number of extents used and remaining, and the tracks and cylinders in use.
8.2 THE system SHALL report, for a Volume: total, used, and free tracks and cylinders, and the number of allocated and available extents or space.
8.3 THE system SHALL expose a VTOC_View that lists the datasets (and their extents) residing on a Volume together with the Volume's usage counters.
8.4 THE reporting counters SHALL be derived metadata computed from recorded extents and the Geometry_Profile; they SHALL NOT require inspecting physical on-disk layout.
8.5 THE VTOC_View SHALL be invokable as a command through the single command-dispatch path (a volume listing).

---

### Requirement 9: Multivolume, Uncataloged, and Cardinality

**User Story:** As a mainframe developer, I want the model to permit multivolume datasets, shared Volumes across catalogs, and uncataloged datasets, so that FFWB can express the real relationships a JCL/DD environment needs.

**Source:** `FFWB_Storage_and_Catalog_Data_Model.md` sections 5, 7.10, 15; recommendation sections 3.3, 5.2. Owner decisions 6, 7. Content was rephrased for compliance with licensing restrictions.

#### Acceptance Criteria

9.1 THE DatasetVolume association SHALL record, for each Volume a dataset resides on, a sequence_number, an is_primary flag, and an opaque per-volume locator, replacing the dataset's raw storage_path.
9.2 THE model SHALL permit a single dataset to reside on more than one Volume (a Multivolume_Dataset) via an ordered sequence of DatasetVolume rows.
9.3 THE model SHALL permit datasets registered in different catalogs to reside on the same shared Volume -- the 1:1 catalog-to-Volume binding SHALL be broken at the model level.
9.4 THE model SHALL permit an Uncataloged_Dataset to physically exist on a Volume with no catalog entry.
9.5 WHEN a caller requests an Uncataloged_Dataset by explicit VOL=SER plus UNIT, THE system SHALL resolve it from the Volume without requiring a catalog row.
9.6 THE UI MAY default to one Volume per catalog initially, but the schema and model SHALL permit the many-catalogs-per-Volume, many-Volumes-per-dataset, and uncataloged relationships above.

---

### Requirement 10: DEFINE VOLUME and Volume Visibility

**User Story:** As a mainframe developer, I want to register a host directory as an emulated Volume and see Volumes as a visible but advanced concept, so that I can manage storage without the Volume being hidden or being the first thing a casual user meets.

**Source:** `FFWB_Storage_and_Catalog_Data_Model.md` section 11.1 (DEFINE VOLUME); recommendation section 6. Owner decision 8. Content was rephrased for compliance with licensing restrictions.

#### Acceptance Criteria

10.1 THE system SHALL provide a `DEFINE VOLUME` FFWB-extension command that registers a host directory as an emulated Volume, accepting at minimum a name/VOLSER, a path, a capacity, and a status.
10.2 THE `DEFINE VOLUME` command SHALL be resolved through the single command-dispatch path (not a bespoke intercept or a second dispatcher).
10.3 WHEN `DEFINE VOLUME` is given a VOLSER that already exists, THE system SHALL fail with a reported duplicate-VOLSER error (Requirement 1.2).
10.4 THE Volume concept SHALL be VISIBLE but ADVANCED -- surfaced during catalog and dataset creation and in a Volume report / VTOC_View listing -- and SHALL NOT be hidden from users who need it.
10.5 WHEN a Volume listing or VTOC_View is surfaced as a Workspace Context, THE Context SHALL be a `WorkspaceContext` dispatched via `render_workspace_context`, returning its interior focus; this specification SHALL introduce no bespoke focus ring or second navigation stack.

## Non-Functional Requirements

### Requirement 11: Determinism, Derived Reporting, and Metadata-Only Migration

**Source:** `FFWB_Storage_and_Catalog_Data_Model.md` ADR-003, ADR-005; section 14. Owner decisions 1, 2. Content was rephrased for compliance with licensing restrictions.

#### Acceptance Criteria

11.1 THE geometry conversions (Requirement 3) SHALL be deterministic: identical inputs under an identical Geometry_Profile SHALL always produce identical track, cylinder, and byte results.
11.2 THE reporting counters (Requirement 8) SHALL be derived metadata computed in time proportional to the number of extents (O(n) in extents, O(1) for a dataset with a single extent); they SHALL NOT scan physical content.
11.3 WHEN a Volume is defined over an existing Repository root, NO dataset bytes SHALL move -- the migration SHALL add metadata only (ADR-003: physical object names are opaque and renames are metadata-only).
