# Requirements Document

> **Governance Reference:** This specification is governed by the [Dataset Ownership Model](./../dataset-ownership-model/requirements.md) (ADR-001). Where this document conflicts with the governance document, the governance document takes precedence. The dataset-catalog crate is the single authority for dataset metadata, catalog entries, naming validation, and resolution APIs.

## Introduction

This feature specifies the Dataset Catalog subsystem for FileForgeWorkbench (`ff-dscatalog` crate). The Dataset Catalog provides **mainframe dataset filesystem emulation on the local desktop** — enabling developers to work with mainframe-style dataset naming, organization, and management without access to a z/OS system.

The subsystem implements a SQLite-backed catalog database that maps mainframe dataset names (HLQ.qualifier format) to physical files stored in a structured repository layout on the local filesystem. It supports sequential datasets (PS), partitioned datasets (PDS/PDSE), and Generation Data Groups (GDG). The catalog integrates with the VFS layer as a dedicated provider (scheme `catalog`), making datasets addressable as `vfs://catalog/HLQ.QUALIFIER.NAME` throughout the workbench.

The Dataset Catalog provides catalog lifecycle management (mount, unmount, add, remove, export, import), dataset CRUD operations (create, delete, rename, resolve, allocate), PDS member navigation, a properties panel displaying dataset attributes (LRECL, RECFM, BLKSIZE, DSORG, creation date), context menus for the catalog tree, dataset allocation parameters (space, directory blocks), and GDG generation management (rolling limits, generation creation).

**Architectural integration:**
- Implements the `VfsProvider` trait from `ff-vfs` (virtual-file-system), registering under scheme `catalog`
- All dataset I/O flows through the VFS abstraction (FFW-ARCH-001)
- Catalog configuration (mounted catalogs, default HLQ) persisted via `ff-config` (configuration-system)
- Dataset operations exposed as commands via `ff-command` (command-framework)
- File tree integration provided by `file-tree-panel` consuming the VFS provider

**Source references:**
- **[DSC]** = Dataset Catalog Brief (primary source)
- **[WB]** = Workbench Platform Architecture Brief (FFW-ARCH-001 VFS principle)
- **[FFE]** = FileForgeEditor file operations (adapted for dataset paradigm)

### Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `virtual-file-system` | **Dependency** | Implements `VfsProvider` trait; registers under scheme `catalog`. All operations route through VFS. |
| `connector-extensibility` | **Implements** | The catalog provider conforms to the connector extensibility framework, advertising capabilities. |
| `file-tree-panel` | **Consumer** | File tree renders catalog content under the "Catalogs" root node via VFS list/stat. |
| `fileforge-integration` | **Integration** | FileForge-mode datasets (EBCDIC, COMP-3) are stored in and resolved from the catalog. |
| `command-framework` | **Dependency** | All catalog/dataset operations are registered as commands dispatched through the framework. |
| `configuration-system` | **Dependency** | Mounted catalogs, default HLQ, and repository paths stored in `[catalog]` config namespace. |

> **Crate name:** The actual workspace crate is `ff-dscatalog`. Any reference to `ff-dataset-catalog` in older documents is incorrect and should be read as `ff-dscatalog`.

## Glossary

- **Dataset**: A named data resource in the mainframe naming convention. Identified by a Dataset_Name (DSN) composed of qualifiers separated by dots. [DSC]
- **Dataset_Name (DSN)**: A string of 1–8 character qualifiers separated by dots, maximum 44 characters total. Each qualifier starts with an alphabetic or national character (A-Z, @, #, $) followed by alphanumeric or national characters. [DSC]
- **High_Level_Qualifier (HLQ)**: The first qualifier in a Dataset_Name, typically representing an owner or project (e.g., `PAYROLL` in `PAYROLL.INPUT.FILE`). [DSC]
- **Catalog**: A SQLite database that maps Dataset_Names to physical file locations within a Repository. A workbench session can have multiple catalogs mounted simultaneously. [DSC]
- **Catalog_Database**: The SQLite file (`catalog.db`) at the root of a Repository, containing the metadata for all datasets in that catalog. [DSC]
- **Repository**: A directory structure on the local filesystem that physically stores dataset content, organized into `storage/`, `pds/`, `gdg/`, and `temp/` subdirectories. [DSC]
- **Sequential_Dataset (PS)**: A dataset type representing a single flat file — equivalent to a regular file. Stored as one physical file in the repository's `storage/` directory. [DSC]
- **Partitioned_Dataset (PDS)**: A dataset type representing a library of members — equivalent to a directory of files. Each member is an independently addressable unit. Stored as a directory in the repository's `pds/` directory. [DSC]
- **Partitioned_Dataset_Extended (PDSE)**: A modern variant of PDS with enhanced capabilities (no directory block limit, member-level locking, dynamic space release). Functionally treated identically to PDS in the local emulation. [DSC]
- **PDS_Member**: An individually named unit within a PDS or PDSE. Member names are 1–8 characters following the same naming rules as a single qualifier. [DSC]
- **Generation_Data_Group (GDG)**: A collection of chronologically versioned datasets (generations) sharing a base name. Managed with a rolling limit — oldest generations are automatically deleted when the limit is exceeded. [DSC]
- **GDG_Generation**: A single versioned instance within a GDG, identified by a generation number in the format `GnnnnVnn` (e.g., `G0001V00`). [DSC]
- **GDG_Limit**: The maximum number of active generations maintained in a GDG. When a new generation is created and the limit is reached, the oldest generation is rolled off (deleted or uncataloged). [DSC]
- **Dataset_Allocation**: The act of creating a new dataset with specified attributes (type, LRECL, RECFM, BLKSIZE, space, directory blocks). Analogous to the z/OS ALLOCATE command or JCL DD with DISP=NEW. [DSC]
- **LRECL (Logical_Record_Length)**: The length of each logical record in a dataset, in bytes. For variable-length records, this is the maximum record length. [DSC]
- **RECFM (Record_Format)**: The format of records in a dataset: F (fixed), V (variable), FB (fixed blocked), VB (variable blocked), U (undefined). [DSC]
- **BLKSIZE (Block_Size)**: The physical block size in bytes for dataset I/O. In local emulation, this is metadata only (no actual blocking). [DSC]
- **DSORG (Dataset_Organization)**: The organization of the dataset: PS (sequential), PO (partitioned — PDS/PDSE), GDG (generation data group). [DSC]
- **Mount**: The act of making a Catalog available for use in the current session — its datasets become visible in the file tree and resolvable by DSN. [DSC]
- **Unmount**: The act of hiding a Catalog from the current session without deleting it — its datasets become invisible and unresolvable. [DSC]
- **Catalog_Export**: Packaging a catalog's database and repository into a portable archive (ZIP) for sharing or backup. [DSC]
- **Catalog_Import**: Restoring a catalog from a previously exported archive into a new repository location. [DSC]
- **Dataset_Resolution**: Looking up a Dataset_Name in mounted catalogs and returning the physical path to the underlying file content. [DSC]
- **Properties_Panel**: A UI panel displaying dataset attributes (DSN, type, RECFM, LRECL, BLKSIZE, DSORG, creation date, modification date, physical path). [DSC]
- **Context_Menu**: A right-click menu on catalog tree nodes offering operations appropriate to the node type (catalog, dataset, PDS member). [DSC]

## Requirements

### Requirement 1: SQLite Catalog Database

**User Story:** As a workbench user, I want dataset metadata stored in a reliable, queryable database so that dataset lookups are fast, consistent, and survive application restarts.

**Source:** [DSC] §6 — Catalog Database Design. [DSC, WB]

#### Acceptance Criteria

1. THE Catalog_Database SHALL be implemented as a SQLite database file named `catalog.db` located at the root of each Repository directory.
2. THE Catalog_Database SHALL store a `datasets` table containing at minimum the following columns: `id` (INTEGER PRIMARY KEY), `dsn` (TEXT UNIQUE NOT NULL), `dsorg` (TEXT NOT NULL — one of PS, PO, GDG), `storage_path` (TEXT NOT NULL — relative path from repository root to physical content), `recfm` (TEXT — record format), `lrecl` (INTEGER — logical record length), `blksize` (INTEGER — block size), `created` (TEXT — ISO 8601 timestamp), `modified` (TEXT — ISO 8601 timestamp), `accessed` (TEXT — ISO 8601 timestamp).
3. THE Catalog_Database SHALL store a `gdg_bases` table for GDG definitions containing: `id` (INTEGER PRIMARY KEY), `dsn` (TEXT UNIQUE NOT NULL — the GDG base name), `limit` (INTEGER NOT NULL — maximum active generations), `scratch` (BOOLEAN NOT NULL DEFAULT TRUE — whether rolled-off generations are physically deleted), `created` (TEXT — ISO 8601 timestamp).
4. THE Catalog_Database SHALL store a `gdg_generations` table containing: `id` (INTEGER PRIMARY KEY), `base_id` (INTEGER FOREIGN KEY referencing gdg_bases), `generation_number` (INTEGER NOT NULL), `version` (INTEGER NOT NULL DEFAULT 0), `dataset_id` (INTEGER FOREIGN KEY referencing datasets), `status` (TEXT — active, rolled_off, deferred).
5. THE Catalog_Database SHALL enforce a UNIQUE constraint on `dsn` within the `datasets` table — no two datasets in the same catalog SHALL have identical names.
6. THE Catalog_Database SHALL use WAL (Write-Ahead Logging) journal mode for concurrent read access during write operations.
7. WHEN the catalog database file does not exist at the repository root, THE system SHALL create it with the correct schema upon first mount or catalog creation.
8. THE Catalog_Database SHALL store a `catalog_metadata` table containing: `key` (TEXT PRIMARY KEY), `value` (TEXT) — for catalog-level properties (catalog name, version, creation date, description).
9. ALL database operations SHALL use parameterized queries to prevent SQL injection from user-supplied dataset names or paths.

---

### Requirement 2: Dataset Naming Conventions

**User Story:** As a mainframe developer, I want dataset names to follow standard mainframe naming rules (HLQ.qualifier format) so that my local development environment faithfully represents the naming constraints I encounter on z/OS.

**Source:** [DSC] §5 — Mainframe-style dataset naming. [DSC]

#### Acceptance Criteria

1. A Dataset_Name SHALL consist of one or more qualifiers separated by dots (`.`), with a maximum total length of 44 characters including the dot separators.
2. EACH qualifier SHALL be 1–8 characters in length, starting with an alphabetic character (A–Z) or a national character (`@`, `#`, `$`), followed by zero or more alphanumeric characters (A–Z, 0–9) or national characters.
3. THE first qualifier in a Dataset_Name SHALL be the High_Level_Qualifier (HLQ), representing the dataset owner or project grouping.
4. WHEN a dataset name is submitted that does not conform to the naming rules (invalid characters, qualifier too long, total length exceeded, empty qualifier between dots), THE system SHALL return an error describing the specific validation failure and the position of the offending character or qualifier.
5. THE system SHALL perform case-insensitive comparison of Dataset_Names — `PAYROLL.INPUT` and `payroll.input` SHALL resolve to the same dataset. Internally, all Dataset_Names SHALL be stored in uppercase.
6. THE system SHALL support an alias resolution mechanism: a default HLQ may be configured per user profile, and when a bare qualifier is provided without a leading HLQ, THE system SHALL prepend the configured default HLQ.
7. THE system SHALL reject Dataset_Names that begin or end with a dot, or contain consecutive dots (`..`).
8. THE system SHALL validate PDS member names using the same rules as a single qualifier: 1–8 characters, starting with alphabetic or national character, followed by alphanumeric or national characters.
9. THE system SHALL support referencing a PDS member using the syntax `DSN(MEMBER)` — parenthesized member name appended to the dataset name — and parse this into separate DSN and member components.

---

### Requirement 3: Dataset Types

**User Story:** As a mainframe developer, I want to create and work with different dataset organizations (sequential, PDS, PDSE, GDG) so that the local emulation matches the data structures I use on z/OS.

**Source:** [DSC] §4 — Dataset organization types. [DSC]

#### Acceptance Criteria

1. THE system SHALL support creating datasets with the following organization types: `PS` (sequential — single flat file), `PO` (partitioned — PDS or PDSE, a library of members), and `GDG` (generation data group — versioned dataset collection).
2. WHEN a dataset with `DSORG=PS` is created, THE system SHALL create a single physical file in the repository's `storage/` directory and record its relative path in the catalog database.
3. WHEN a dataset with `DSORG=PO` is created, THE system SHALL create a directory in the repository's `pds/` directory to contain member files; each PDS member SHALL be stored as an individual file within that directory.
4. WHEN a dataset with `DSORG=GDG` is created, THE system SHALL create a GDG base entry in the `gdg_bases` table specifying the generation limit and scratch policy; no physical storage is allocated until individual generations are created.
5. THE system SHALL distinguish between PDS and PDSE via a `subtype` field in the catalog: PDS is the default; PDSE is indicated by `subtype=PDSE`. Functionally, both are treated identically in the local emulation (no directory block limit for either, dynamic space release for both).
6. EACH dataset SHALL carry the following metadata attributes (stored in the catalog database): `RECFM`, `LRECL`, `BLKSIZE`, `DSORG`, `creation_date`, `last_modified_date`, `last_access_date`, and `allocated_space` (informational).
7. THE system SHALL validate dataset type consistency: a PS dataset SHALL NOT be opened as a PDS (member access), and a PDS SHALL NOT be opened as a sequential flat file.
8. WHEN a dataset is opened for reading, THE system SHALL update its `last_access_date` in the catalog database.

---

### Requirement 4: Repository Layout (Legacy DSN-Derived Layout)

> **SUPERSEDED FOR NEW ALLOCATIONS by Requirement 20 (UUID-Based Physical Object Layout).**
> All new dataset allocations SHALL use the UUID-based layout defined in Requirement 20.
> Requirement 4 describes the legacy DSN-derived layout and is retained solely as a reference
> for import/export compatibility with catalogs created before Requirement 20 was adopted.
> Implementations SHALL NOT use DSN-derived paths for new allocations.

**User Story:** As a workbench user, I want dataset content stored in a well-defined directory structure on my local filesystem, so that I can understand where files are physically stored, back them up with standard tools, and verify content outside the workbench if needed.

**Source:** [DSC] §6 — Repository directory structure. [DSC, WB]

#### Acceptance Criteria

1. A Repository SHALL be a directory with the following structure at its root: `catalog.db` (SQLite catalog database), `storage/` (sequential dataset files), `pds/` (partitioned dataset directories), `gdg/` (GDG base directories with generation subdirectories), and `temp/` (temporary allocations, automatically cleaned on startup).
2. SEQUENTIAL datasets SHALL be stored as files within `storage/`, using a filesystem-safe encoding of the DSN as the filename (dots replaced with directory separators or a configurable encoding scheme that preserves the qualifier structure). NOTE: new allocations use UUID layout (Req 20.2) instead.
3. PARTITIONED datasets SHALL be stored as directories within `pds/`, with the directory name derived from the DSN; each member SHALL be stored as an individual file within that directory using the member name as the filename (uppercased). NOTE: new allocations use UUID layout (Req 20.2) instead.
4. GDG bases SHALL be stored as directories within `gdg/`, with each generation stored as a file or directory (depending on generation DSORG) within the GDG base directory, named using the generation identifier format `GnnnnVnn`. NOTE: new allocations use UUID layout (Req 20.2) instead.
5. THE mapping from DSN to physical path SHALL be stored in the catalog database (`storage_path` column), allowing the physical layout to be reconstructed or relocated.
6. WHEN a repository is created (first catalog initialization), THE system SHALL create all required subdirectories (`storage/`, `pds/`, `gdg/`, `temp/`) and the catalog database with the correct schema.
7. THE system SHALL handle filesystem-safe name encoding for DSNs containing national characters (`@`, `#`, `$`): these SHALL be percent-encoded in physical directory names (e.g., `#` → `%23`) to ensure cross-platform compatibility.
8. THE system SHALL support configuring the repository root path, defaulting to `~/.ffworkbench/catalogs/{catalog-name}/` if not explicitly specified.
9. THE `temp/` directory SHALL be purged of stale allocations (files older than 24 hours or from previous sessions) when the catalog is mounted.

---

### Requirement 5: Catalog Mount and Unmount

**User Story:** As a workbench user, I want to mount and unmount catalogs during my session, so that I can work with multiple project catalogs selectively and keep my file tree focused on relevant datasets.

**Source:** [DSC] §7 — Catalog lifecycle management. [DSC, WB]

#### Acceptance Criteria

1. THE system SHALL support mounting a catalog by specifying the path to its repository root directory; upon mounting, the catalog's datasets become visible in the file tree and resolvable by DSN through the VFS provider.
2. THE system SHALL support multiple simultaneously mounted catalogs — datasets from all mounted catalogs are visible and resolvable concurrently.
3. WHEN multiple mounted catalogs contain datasets with the same DSN, THE system SHALL resolve using catalog priority order (most recently mounted has highest priority); the resolution result SHALL include which catalog provided the dataset.
4. THE system SHALL support unmounting a catalog, which removes its datasets from visibility and resolution without deleting the catalog or its data; any open files from the unmounted catalog SHALL remain open but further resolves to that catalog SHALL fail until remounted.
5. WHEN a catalog is mounted, THE system SHALL validate the repository structure and catalog database schema; IF validation fails (missing directories, corrupt database, schema mismatch), THE system SHALL return an error describing the problem and not mount the catalog.
6. THE system SHALL persist the list of mounted catalogs and their priority order across sessions via the configuration system (`ff-config`), restoring mounts on application startup.
7. WHEN a catalog is mounted, THE system SHALL register its content with the VFS provider registry so that URIs of the form `vfs://catalog/DSN` resolve to datasets in the mounted catalog.
8. THE system SHALL expose catalog mount and unmount operations as commands registered with the command framework: `catalog.mount` (params: repository path) and `catalog.unmount` (params: catalog name or path).

---

### Requirement 6: Catalog Add, Remove, Export, and Import

**User Story:** As a workbench user, I want to create new catalogs, remove existing ones, and export/import catalogs as portable archives, so that I can share dataset collections with team members, back up my work, and set up new environments quickly.

**Source:** [DSC] §7 — Catalog creation, removal, and portability. [DSC, WB]

#### Acceptance Criteria

1. THE system SHALL support creating a new empty catalog by specifying a name and repository root path; creation SHALL initialize the repository directory structure and an empty catalog database.
2. WHEN creating a catalog, THE system SHALL accept optional parameters: `default_hlq` (default High Level Qualifier for the catalog), `description` (human-readable description), and `auto_mount` (whether to mount automatically on startup).
3. THE system SHALL support removing a catalog, which unmounts it (if mounted) and optionally deletes the repository directory and all physical dataset content; removal with deletion SHALL require explicit user confirmation.
4. THE system SHALL support exporting a catalog to a portable ZIP archive containing the catalog database and all repository content, preserving the directory structure.
5. WHEN exporting, THE system SHALL include a manifest file (`manifest.json`) within the archive containing: catalog name, description, export timestamp, dataset count, total size, and schema version.
6. THE system SHALL support importing a catalog from a previously exported ZIP archive into a specified target directory, restoring the repository structure and catalog database.
7. WHEN importing, THE system SHALL validate the archive integrity: verify the manifest exists and is valid, confirm the schema version is compatible, and check that all files referenced in the catalog database exist in the archive.
8. IF an import archive has an incompatible schema version, THEN THE system SHALL return an error indicating the version mismatch and the supported schema version range.
9. THE system SHALL expose catalog operations as commands: `catalog.create` (params: name, path, options), `catalog.remove` (params: catalog name, delete_files: bool), `catalog.export` (params: catalog name, output path), `catalog.import` (params: archive path, target directory).
10. WHEN export or import operations involve large repositories, THE system SHALL report progress through the workflow engine's progress reporting mechanism, showing files processed and total size.

---

### Requirement 7: Dataset Create, Delete, Rename, and Allocate

**User Story:** As a mainframe developer, I want to create, delete, rename, and allocate datasets with mainframe-style parameters (RECFM, LRECL, BLKSIZE, space), so that I can manage my local dataset collection using familiar concepts.

**Source:** [DSC] §8 — Dataset CRUD operations. [DSC, WB]

> **Ownership Clarification (ADR-001):** This requirement defines the **low-level catalog CRUD API** — the primitives that create/delete/rename catalog entries and their associated physical storage. JCL-driven allocation workflows (parsing DD statements, interpreting DISP semantics, applying defaults, symbolic substitution) are owned by `ff-dsalloc` (Dataset Allocator). The allocator invokes these catalog primitives to execute allocation; it does not duplicate them.

#### Acceptance Criteria

1. THE system SHALL support creating (allocating) a new dataset by specifying: DSN, DSORG (PS, PO, or GDG), RECFM (F, FB, V, VB, U), LRECL (integer), BLKSIZE (integer), and optionally: directory blocks (for PDS), GDG limit (for GDG), and description.
2. WHEN a dataset is allocated with valid parameters, THE system SHALL create the catalog entry, allocate the physical storage in the appropriate repository subdirectory, and return success.
3. IF a dataset with the specified DSN already exists in any mounted catalog, THEN THE system SHALL return an error indicating the dataset already exists and identify the catalog containing the duplicate.
4. THE system SHALL support deleting a dataset by DSN: removing the catalog entry, deleting the physical storage (file for PS, directory and members for PDS, all generations for GDG), and confirming deletion.
5. WHEN a PDS is deleted, ALL its members SHALL be deleted along with the PDS directory; the operation SHALL be atomic (either all members and the directory are removed, or none are).
6. THE system SHALL support renaming a dataset by specifying the current DSN and new DSN; renaming SHALL update the catalogue entry only -- the physical object SHALL NOT be moved or renamed (see Requirement 20.6). For legacy DSN-derived repositories (Requirement 4), a migration path to UUID layout is required before rename can be performed without physical move.
7. WHEN renaming a dataset, THE system SHALL validate that the new DSN conforms to naming rules and does not already exist in any mounted catalog.
8. THE system SHALL support a resolve operation: given a DSN, return the physical filesystem path to the dataset's content, the catalog that provided the resolution, and the dataset's metadata.
9. WHEN a resolve is attempted for a DSN that does not exist in any mounted catalog, THE system SHALL return a `VfsError::NotFound` error containing the DSN.
10. THE system SHALL validate allocation parameters: LRECL must be > 0 and ≤ 32760; BLKSIZE must be ≥ LRECL; RECFM must be one of the supported values; GDG limit must be between 1 and 255.
11. THE system SHALL expose dataset operations as commands: `dataset.allocate` (params: DSN, DSORG, RECFM, LRECL, BLKSIZE, options), `dataset.delete` (params: DSN), `dataset.rename` (params: old DSN, new DSN).

---

### Requirement 8: PDS Member Operations

**User Story:** As a mainframe developer, I want to list, open, create, delete, and rename members within a PDS, so that I can manage partitioned dataset libraries as I would on a mainframe system.

**Source:** [DSC] §9 — PDS member management. [DSC, WB]

#### Acceptance Criteria

1. THE system SHALL support listing all members of a PDS given its DSN, returning a list of member names sorted alphabetically, each with metadata (size, last modified date).
2. THE system SHALL support opening a PDS member for reading or writing using the `DSN(MEMBER)` syntax, resolving to the physical file within the PDS directory.
3. THE system SHALL support creating a new member in an existing PDS by specifying the PDS DSN and the new member name; the member name SHALL be validated against member naming rules (1–8 characters, same rules as a qualifier).
4. WHEN a member is created in a PDS that does not exist or is not of type PO, THE system SHALL return an error indicating the target dataset is not a partitioned dataset.
5. THE system SHALL support deleting a member from a PDS by specifying `DSN(MEMBER)`; the physical file SHALL be removed from the PDS directory.
6. THE system SHALL support renaming a member within a PDS by specifying the PDS DSN, old member name, and new member name; the physical file SHALL be renamed within the PDS directory.
7. WHEN a member operation targets a non-existent member (open, delete, rename source), THE system SHALL return a `VfsError::NotFound` error containing the DSN and member name.
8. WHEN a member is created with a name that already exists in the PDS, THE system SHALL return a `VfsError::AlreadyExists` error unless an overwrite option is explicitly specified.
9. THE system SHALL update the PDS dataset's `last_modified_date` in the catalog whenever a member is created, deleted, or renamed.
10. THE system SHALL expose member operations as commands: `member.create` (params: DSN, member name), `member.delete` (params: DSN, member name), `member.rename` (params: DSN, old name, new name).

---

### Requirement 9: Generation Data Group (GDG) Management

**User Story:** As a mainframe developer, I want to create GDG bases with rolling generation limits and create/access generations using relative references (+1, 0, -1), so that I can emulate the versioned dataset workflow used in mainframe batch processing.

**Source:** [DSC] §10 — GDG management. [DSC]

#### Acceptance Criteria

1. THE system SHALL support creating a GDG base with a specified DSN, generation limit (1–255), and scratch policy (whether to physically delete rolled-off generations or merely uncatalog them).
2. WHEN a new generation is created for a GDG (via `+1` relative reference or explicit `catalog.gdg.create_generation`), THE system SHALL assign the next sequential generation number, create the generation's physical storage, add a catalog entry with the fully qualified generation name (`BASE.GnnnnVnn`), and link it to the GDG base in the `gdg_generations` table.
3. WHEN creating a new generation causes the active generation count to exceed the GDG limit, THE system SHALL roll off the oldest active generation: IF the scratch policy is `true`, THE physical storage SHALL be deleted; IF `false`, the generation SHALL be marked as `rolled_off` in the catalog but its physical storage SHALL be preserved.
4. THE system SHALL support accessing GDG generations using relative references: `(0)` for the current (most recently created) generation, `(-1)` for the previous generation, `(+1)` for allocating a new generation.
5. THE system SHALL support accessing GDG generations using absolute references: `GnnnnVnn` format appended to the GDG base name (e.g., `PAYROLL.MONTHLY.G0003V00`).
6. THE system SHALL support listing all active generations of a GDG base, returning generation numbers, creation dates, and sizes, sorted from newest to oldest.
7. THE system SHALL support modifying a GDG base's properties: changing the generation limit (with automatic roll-off if the new limit is smaller) and changing the scratch policy.
8. WHEN a GDG base is deleted, ALL its active and rolled-off generations SHALL be deleted (physical and catalog entries), and the GDG base entry SHALL be removed.
9. THE system SHALL expose GDG operations as commands: `gdg.create_base` (params: DSN, limit, scratch), `gdg.create_generation` (params: base DSN, DSORG, RECFM, LRECL, BLKSIZE), `gdg.delete_base` (params: DSN), `gdg.list_generations` (params: base DSN).
10. THE system SHALL validate GDG relative references: `(+1)` is only valid in allocation contexts (creating a new generation); `(0)` and negative references are only valid in read/access contexts and SHALL return an error if no generation exists at that offset.

---

### Requirement 10: VFS Provider Integration

**User Story:** As a workbench developer, I want the dataset catalog to integrate with the VFS as a registered provider, so that datasets are accessible using standard `vfs://catalog/DSN` URIs and all VFS operations (open, read, write, list, stat, exists) work transparently on catalog-managed datasets.

**Source:** [DSC] §3 — VFS integration; [WB] Architecture Brief FFW-ARCH-001. [DSC, WB]

#### Acceptance Criteria

1. THE dataset catalog crate SHALL implement the `VfsProvider` trait from `ff-vfs`, registering with the scheme identifier `catalog`.
2. WHEN the VFS receives a request for a URI of the form `vfs://catalog/DSN`, THE catalog provider SHALL resolve the DSN against mounted catalogs and route the operation to the physical storage path.
3. THE catalog provider SHALL implement `list(path)` as follows: when path is empty or `/`, list all mounted catalogs; when path is a catalog name, list all HLQs in that catalog; when path is a partial DSN, list datasets matching that prefix; when path is a PDS DSN, list its members.
4. THE catalog provider SHALL implement `stat(path)` by returning a `VfsMetadata` struct populated with dataset attributes: size, last modified time, resource type (file for PS/GDG generation, directory for PDS), and provider-specific metadata containing RECFM, LRECL, BLKSIZE, DSORG as key-value pairs.
5. THE catalog provider SHALL implement `open(path, options)` by resolving the DSN to a physical path and delegating to the local filesystem for actual I/O, returning an async reader/writer.
6. THE catalog provider SHALL implement `read(path)` and `write(path, data)` for sequential datasets and PDS members by resolving the physical path and performing the I/O.
7. THE catalog provider SHALL implement `create(path, options)` as dataset allocation — creating a new dataset entry in the catalog and physical storage in the repository.
8. THE catalog provider SHALL implement `delete(path)` as dataset deletion — removing the catalog entry and physical storage.
9. THE catalog provider SHALL implement `rename(old_path, new_path)` as dataset rename — updating the catalog entry and physical storage path.
10. THE catalog provider SHALL implement `exists(path)` by checking whether the DSN exists in any mounted catalog, returning `true`/`false` without error for non-existent datasets.
11. THE catalog provider SHALL advertise the following VFS capabilities: `Read`, `Write`, `List`, `Metadata`, `Create`, `Delete`, `Rename`. It SHALL NOT advertise `Watch` or `Search` in the initial release.
12. THE catalog provider SHALL map its internal errors to `VfsError` variants: DSN not found → `NotFound`, duplicate DSN → `AlreadyExists`, invalid DSN format → `InvalidUri`, PDS member not found → `NotFound`, catalog not mounted → `ProviderUnavailable`.

---

### Requirement 11: Properties Panel

**User Story:** As a workbench user, I want to view dataset attributes (RECFM, LRECL, BLKSIZE, DSORG, dates, physical path) in a dedicated properties panel, so that I can inspect dataset characteristics without executing commands.

**Source:** [DSC] §11 — Dataset properties display. [DSC, WB]

#### Acceptance Criteria

1. WHEN a dataset node is selected in the file tree panel, THE system SHALL make its properties available for display in a Properties_Panel.
2. THE Properties_Panel SHALL display the following attributes for a sequential dataset: DSN, DSORG (PS), RECFM, LRECL, BLKSIZE, creation date, last modified date, last access date, physical file size, physical path, and containing catalog name.
3. THE Properties_Panel SHALL display the following attributes for a partitioned dataset: DSN, DSORG (PO), subtype (PDS or PDSE), RECFM, LRECL, BLKSIZE, creation date, last modified date, member count, physical directory path, and containing catalog name.
4. THE Properties_Panel SHALL display the following attributes for a GDG base: DSN, DSORG (GDG), generation limit, scratch policy, active generation count, creation date, and containing catalog name.
5. THE Properties_Panel SHALL display the following attributes for a GDG generation: generation name (BASE.GnnnnVnn), generation number, RECFM, LRECL, BLKSIZE, creation date, file size, physical path, and parent GDG base DSN.
6. THE Properties_Panel SHALL display the following attributes for a PDS member: member name, parent PDS DSN, file size, last modified date, and physical file path.
7. WHEN a property value is not applicable to the dataset type (e.g., member count for a PS dataset), THE field SHALL be omitted from the display rather than shown as empty or N/A.
8. THE Properties_Panel SHALL update dynamically when the selected node changes in the file tree — no explicit refresh action required from the user.
9. THE system SHALL expose a command `dataset.properties` (params: DSN) that retrieves dataset properties programmatically, returning a structured result containing all applicable attributes.

---

### Requirement 12: Context Menus

**User Story:** As a workbench user, I want right-click context menus on catalog tree nodes that offer operations appropriate to the node type (catalog, dataset, PDS, member, GDG), so that I can perform common actions without memorizing commands.

**Source:** [DSC] §12 — Context menu actions. [DSC, WB]

#### Acceptance Criteria

1. WHEN the user right-clicks on a mounted catalog node in the file tree, THE system SHALL display a context menu containing: "Unmount Catalog", "New Dataset…", "Properties", "Export Catalog…", and "Refresh".
2. WHEN the user right-clicks on a sequential dataset (PS) node, THE system SHALL display a context menu containing: "Open", "Rename…", "Delete", "Properties", "Copy DSN", and "Allocate Like…" (create a new dataset with same attributes).
3. WHEN the user right-clicks on a partitioned dataset (PDS/PDSE) node, THE system SHALL display a context menu containing: "Expand" (show members), "New Member…", "Rename…", "Delete", "Properties", "Copy DSN", and "Allocate Like…".
4. WHEN the user right-clicks on a PDS member node, THE system SHALL display a context menu containing: "Open", "Rename…", "Delete", "Copy Member Name", and "Properties".
5. WHEN the user right-clicks on a GDG base node, THE system SHALL display a context menu containing: "New Generation…", "List Generations", "Properties", "Delete GDG", "Copy DSN", and "Modify Limit…".
6. WHEN the user right-clicks on a GDG generation node, THE system SHALL display a context menu containing: "Open", "Delete", "Properties", and "Copy DSN".
7. WHEN the user right-clicks on the "Catalogs" root node (no catalog selected), THE system SHALL display a context menu containing: "Mount Catalog…", "Create New Catalog…", and "Import Catalog…".
8. ALL context menu actions SHALL be dispatched as commands through the command framework — each menu item invokes the corresponding registered command with appropriate parameters derived from the clicked node.
9. CONTEXT menu items SHALL be dynamically enabled/disabled based on the current state: "Unmount" is disabled for catalogs that are not mounted; "Delete" is disabled if the catalog is read-only.

---

### Requirement 13: LISTCAT and LISTDS Commands

**User Story:** As a mainframe developer, I want LISTCAT and LISTDS equivalent commands that display catalog contents and dataset details in a familiar format, so that I can query my local catalog using the same mental model as on z/OS.

**Source:** [DSC] §13 — Command-line catalog query. [DSC, WB]

> **Ownership Clarification (ADR-001):** The `catalog.listcat` and `catalog.listds` commands defined here are **workbench-native** developer tools with tabular output. They are distinct from the IDCAMS `LISTCAT` command (registered as `idcams.listcat`) which provides mainframe-faithful IDCAMS-formatted output. The IDCAMS LISTCAT command is owned by `ff-idcams` and obtains its data by invoking ff-dataset-catalog's query APIs. Both commands read from the same catalog API and produce consistent results.

#### Acceptance Criteria

1. THE system SHALL register a `LISTCAT` command that lists datasets in mounted catalogs matching a specified filter pattern (DSN prefix, wildcard with `*` and `%` characters).
2. THE `LISTCAT` command SHALL accept the following parameters: `filter` (DSN pattern with wildcards — `*` matches any string, `%` matches a single qualifier), `type` (optional — PS, PO, GDG to filter by DSORG), and `catalog` (optional — limit search to a specific mounted catalog).
3. THE `LISTCAT` command SHALL display results in a tabular format showing: DSN, DSORG, RECFM, LRECL, creation date, and containing catalog name.
4. THE system SHALL register a `LISTDS` command that displays detailed information about a specific dataset, equivalent to the z/OS LISTDS command.
5. THE `LISTDS` command SHALL accept parameters: `dsn` (required — the dataset name to query), `members` (optional boolean — if true and the dataset is a PDS, include the member list), `history` (optional boolean — include creation, last-access, and modification dates).
6. THE `LISTDS` command output SHALL include: DSN, DSORG, RECFM, LRECL, BLKSIZE, creation date, last modified date, physical size, physical path, catalog name, and (for PDS) member count.
7. WHEN `LISTDS` is called with `members=true` on a PDS, THE output SHALL include a member list showing each member's name, size, and last modified date.
8. WHEN `LISTDS` is called with a DSN that does not exist in any mounted catalog, THE system SHALL return an error: "DATASET NOT FOUND: {dsn}".
9. THE `LISTCAT` wildcard patterns SHALL follow mainframe conventions: `*` matches zero or more characters within or across qualifiers; `%` matches exactly one qualifier. For example, `PAY.*` matches `PAY.INPUT`, `PAY.OUTPUT.FILE`; `PAY.%` matches only `PAY.INPUT`, `PAY.OUTPUT` (single qualifier after PAY).
10. BOTH commands SHALL be registered with the command framework under IDs `catalog.listcat` and `catalog.listds` respectively, with appropriate metadata (category: "catalog", description, no default shortcut).

---

### Requirement 14: Configuration Integration

**User Story:** As a workbench user, I want catalog settings (mounted catalogs, default HLQ, repository paths) persisted in the workbench configuration system, so that my catalog environment is automatically restored on each application startup.

**Source:** [DSC] §14 — Configuration persistence. [DSC, WB]

#### Acceptance Criteria

1. THE dataset catalog subsystem SHALL store its configuration under the `[catalog]` TOML table in the workbench configuration, using the `ff-config` namespace scoping mechanism.
2. THE `[catalog]` configuration table SHALL include the following keys: `default_hlq` (string — prepended to bare qualifiers), `mounted_catalogs` (array of tables — each containing `name`, `path`, `priority`, `auto_mount`), and `repository_root` (string — default root directory for new catalogs).
3. WHEN the application starts, THE system SHALL read the `mounted_catalogs` configuration and automatically mount all catalogs where `auto_mount=true`, in priority order.
4. WHEN a catalog is mounted or unmounted during a session, THE system SHALL update the `mounted_catalogs` configuration entry to reflect the current state, ensuring persistence across restarts.
5. THE system SHALL support hot-reload of catalog configuration: when the configuration file changes externally, THE system SHALL detect changes to `[catalog]` settings via the configuration system's reload callback and apply them (mounting/unmounting catalogs as needed).
6. THE `default_hlq` setting SHALL be configurable at both user level (applies to all projects) and project level (overrides user level for the specific project).
7. THE system SHALL validate configuration values on load: repository paths must exist (or be creatable), HLQ values must conform to naming rules, priority values must be positive integers.

---

### Requirement 15: Dataset Allocation Defaults

**User Story:** As a mainframe developer, I want sensible default allocation parameters based on dataset type, so that I can create datasets quickly without specifying every attribute when the defaults are appropriate.

**Source:** [DSC] §8 — Allocation convenience. [DSC]

#### Acceptance Criteria

1. WHEN a sequential dataset (PS) is allocated without explicit RECFM, LRECL, or BLKSIZE, THE system SHALL apply defaults: RECFM=FB, LRECL=80, BLKSIZE=27920.
2. WHEN a partitioned dataset (PDS/PDSE) is allocated without explicit RECFM, LRECL, or BLKSIZE, THE system SHALL apply defaults: RECFM=FB, LRECL=80, BLKSIZE=27920.
3. WHEN a GDG generation is allocated without explicit RECFM, LRECL, or BLKSIZE, THE system SHALL inherit the last generation's attributes if one exists; if no previous generation exists, apply sequential defaults.
4. THE system SHALL support an "Allocate Like" operation that copies all attributes (DSORG, RECFM, LRECL, BLKSIZE) from an existing dataset to serve as defaults for a new dataset, requiring only the new DSN to be specified.
5. THE system SHALL allow all default values to be overridden by explicit parameters at allocation time — explicit values always take precedence over defaults.
6. THE system SHALL provide configurable allocation defaults in the `[catalog.defaults]` configuration table, allowing users to customise the default RECFM, LRECL, and BLKSIZE for each DSORG type.

---

## Non-Functional Requirements

### Performance

- Catalog mount (opening the SQLite database and validating schema) SHALL complete within 1 second for catalogs with up to 10,000 datasets.
- DSN resolution (lookup by name) SHALL complete within 50 milliseconds for catalogs with up to 10,000 datasets.
- LISTCAT with a wildcard filter SHALL return results within 500 milliseconds for catalogs with up to 10,000 datasets.

### Reliability

- THE catalog database SHALL use WAL journal mode to support concurrent read access during write operations without blocking.
- WHEN the catalog database file is corrupt or unreadable, THE system SHALL return a descriptive error and SHALL NOT crash or corrupt other mounted catalogs.

### Scalability

- THE catalog subsystem SHALL support up to 50 simultaneously mounted catalogs without performance degradation.
- A single catalog SHALL support up to 100,000 dataset entries.

### Data Integrity

- ALL dataset CRUD operations SHALL be atomic: a failed operation SHALL leave the catalog in its pre-operation state with no partial writes.
- THE catalog database SHALL enforce the UNIQUE constraint on DSN at the database level, not only in application code.

---

## Requirements Added by CR-NR-016 — Mainframe Dataset Architecture

> **Source documents:** `docs/source-documents/dataset-catalog/FileForgeWorkbench_Mainframe_Dataset_Architecture.md` and
> `docs/source-documents/dataset-catalog/FileForgeWorkbench_Virtual_File_and_Dataset_Storage_Requirements.md`

---

### Requirement 16: Record-Oriented Storage — No Text-Line Boundaries

**User Story:** As a mainframe developer, I want datasets stored as record-oriented binary objects so that mainframe record semantics are preserved exactly and no CRLF or LF byte is ever silently inserted as a record delimiter.

**Source:** [ARCH] §2 Principle 2; [VFS-REQ] §10 FFW-VFS-REC-001 to REC-005.

#### Acceptance Criteria

16.1 WHEN a mainframe dataset is written, THE system SHALL NOT use CRLF, LF, or any host text-line terminator as a record boundary — record boundaries SHALL be derived solely from RECFM, LRECL, RDW, or VSAM key structure.

16.2 WHEN a fixed-length (F or FB) dataset is stored, THE system SHALL pack records contiguously as `N × LRECL` bytes with no inter-record delimiters; record `n` SHALL be located at byte offset `n × LRECL`.

16.3 WHEN a variable-length (V or VB) dataset is stored, EACH record SHALL be preceded by a 4-byte Record Descriptor Word (RDW) encoding the total record length including the RDW itself; no CRLF or LF delimiter SHALL follow the data bytes.

16.4 WHEN a dataset with RECFM=U (undefined) is stored, THE system SHALL treat the content as an opaque binary stream and preserve bytes exactly without interpretation.

16.5 WHEN a dataset is opened for display in the editor, THE system SHALL present records as lines using the applicable codec without altering the underlying binary storage.

16.6 WHEN a dataset is saved after editing, THE system SHALL re-encode the displayed lines back to the binary record format using the same codec, preserving RECFM and LRECL exactly.

16.7 WHEN a record codec encounters an invalid record length or malformed RDW, THE system SHALL return a diagnostic error containing the dataset identity, record position, and the expected constraint.

---

### Requirement 17: Record Codecs as Independent Components

**User Story:** As a platform developer, I want record codecs separated from storage providers so that encoding logic can be tested independently and reused across different storage backends.

**Source:** [VFS-REQ] §10 FFW-VFS-REC-001 to REC-005; [ARCH] §11 DatasetProvider trait.

#### Acceptance Criteria

17.1 THE system SHALL implement record codecs as a separate module (or crate) independent of any storage provider; codecs SHALL have no dependency on SQLite, the filesystem, or egui.

17.2 THE system SHALL provide a `FixedCodec` that encodes and decodes fixed-length records given an LRECL value.

17.3 THE system SHALL provide a `VariableCodec` that encodes and decodes variable-length records using 4-byte RDW headers.

17.4 THE system SHALL provide a `BinaryCodec` that passes bytes through unchanged for RECFM=U datasets.

17.5 THE system SHALL provide a `TextCodec` that maps host text lines to/from fixed-length records using a configurable encoding profile, used only for explicit import/export operations — never applied silently during normal dataset I/O.

17.6 WHEN encoding or decoding, EACH codec SHALL be independently testable using in-memory byte buffers without any filesystem or database dependency.

17.7 WHEN an import or export operation requires encoding conversion, THE system SHALL require an explicit codec and encoding policy to be specified; the system SHALL NOT infer a codec from file extension or host line endings alone.

---

### Requirement 18: Hybrid Storage Architecture — SQLite Catalogue + Native Files

**User Story:** As a platform architect, I want the catalogue to use SQLite for metadata and native files for sequential/library content so that datasets are accessible to external tools, Git, and backup utilities without requiring workbench-specific extraction.

**Source:** [VFS-REQ] §3 Decision Matrix; [ARCH] §4 Physical Storage Strategy; [VFS-REQ] §15 Prohibited Designs.

#### Acceptance Criteria

18.1 THE system SHALL store PS dataset content as native files on the host filesystem; the catalogue SHALL record the provider locator but SHALL NOT store PS payload bytes as SQLite BLOBs.

18.2 THE system SHALL store PDS and PDSE member content as individual native files within a native directory; the catalogue SHALL record member metadata but SHALL NOT store member payload bytes as SQLite BLOBs.

18.3 THE system SHALL store GDG generation content as native files; the catalogue SHALL record generation lineage and lifecycle state.

18.4 THE system SHALL store VSAM KSDS records in a SQLite-backed keyed record store (separate from the catalogue database) because keyed access semantics require transactional database support.

18.5 THE system SHALL store VSAM RRDS records in a SQLite-backed relative-record store for the same reason.

18.6 THE system SHALL store VSAM ESDS records in an append-oriented native file; an optional sidecar index SHALL be rebuildable from the data file.

18.7 THE system SHALL store POSIX files as native host filesystem objects; the catalogue MAY register a POSIX root as a provider locator but SHALL NOT copy POSIX file contents into SQLite.

18.8 THE system SHALL NOT store PS, PDS, GDG, or POSIX content as BLOBs in the central catalogue database — this design is explicitly prohibited.

---

### Requirement 19: StorageProvider Abstraction Layer

**User Story:** As a platform developer, I want a StorageProvider interface that separates physical access from catalogue resolution so that alternative storage backends can be added without changing dataset editors or catalogue consumers.

**Source:** [VFS-REQ] §8 FFW-VFS-SPI-001 to SPI-004; [ARCH] §11 DatasetProvider trait.

#### Acceptance Criteria

19.1 THE system SHALL define a `StorageProvider` trait exposing at minimum: `allocate`, `open`, `stat`, `rename`, `delete`, `list`, and `reconcile` operations; the exact Rust API may differ from the specification sketch but the responsibilities SHALL remain separated.

19.2 EACH provider SHALL declare its capabilities (stream read/write, record read/write, keyed access, relative access, append-only, member operations, atomic rename, locking, snapshotting, watch notifications) rather than requiring callers to infer them from dataset type.

19.3 THE native-filesystem provider and the SQLite record provider SHALL implement a common error taxonomy mapping to `VfsError` variants.

19.4 Provider-specific locators SHALL be opaque outside the provider and catalogue services; user-interface code SHALL NOT construct or parse raw provider paths.

19.5 THE system SHALL provide a `NativeFileProvider` implementing `StorageProvider` for PS, PDS/PDSE, GDG, and POSIX content stored as native files and directories.

19.6 THE system SHALL provide a `SqliteRecordProvider` implementing `StorageProvider` for VSAM KSDS, RRDS, and ISAM content requiring keyed or relative access.

19.7 WHEN a future storage provider is added, THE system SHALL not require changes to dataset editors, catalogue consumers, or the VFS layer — only a new `StorageProvider` implementation and registration are needed.

---

### Requirement 20: UUID-Based Physical Object Layout

**User Story:** As a platform developer, I want physical dataset objects identified by stable UUIDs rather than DSN-derived paths so that logical renames do not require physical file moves and path-safety issues are eliminated.

**Source:** [VFS-REQ] §9 FFW-VFS-NAM-001 to NAM-005; preferred layout in §9.

#### Acceptance Criteria

20.1 THE system SHALL assign each managed physical object a stable internal UUID at allocation time; this UUID SHALL be stored in the catalogue and used as the physical filename or directory name.

20.2 THE preferred repository layout SHALL be:
```
workspace/
  catalog.db
  datasets/
    objects/
      <dataset-uuid>.dat
      <library-uuid>/
        <member-uuid>.dat
    staging/
  indexed/
    <dataset-uuid>.sqlite
  recovery/
```

20.3 THE logical dataset name SHALL NOT be used as the physical path; the catalogue SHALL be the sole authority mapping logical names to physical locators.

20.4 THE physical mapping SHALL be deterministic and persisted so that a dataset can be found after restart without scanning the filesystem.

20.5 THE system SHALL NOT rely on dots in a dataset name being translated directly into directory separators for the UUID-based layout.

20.6 WHEN a dataset is renamed, THE physical object SHALL NOT be moved or renamed — only the catalogue entry SHALL be updated.

20.7 THE system SHALL protect against path traversal, reserved device names, illegal characters, case-folding collisions, and maximum path-length constraints when constructing physical paths.

---

### Requirement 21: VSAM KSDS Support

**User Story:** As a mainframe developer, I want VSAM Key-Sequenced Dataset emulation so that applications using keyed record access work correctly in the local environment.

**Source:** [VFS-REQ] §7.5 FFW-VFS-KSDS-001 to KSDS-007; [ARCH] §8 VSAM Architecture.

#### Acceptance Criteria

21.1 THE system SHALL implement a KSDS provider using a dedicated SQLite database (one per KSDS dataset) with a table keyed by the primary key field.

21.2 EACH KSDS SHALL define a primary key offset, key length, key type/collation, and uniqueness rule stored in the catalogue.

21.3 THE KSDS provider SHALL support: keyed read by primary key, ordered sequential read, insert, update, delete, and range retrieval.

21.4 Primary-key uniqueness SHALL be enforced transactionally within the SQLite database.

21.5 Alternate indexes SHALL be represented as explicit metadata and additional SQLite indexes or mapping tables within the KSDS database.

21.6 Record data SHALL be stored independently of catalogue rows so that catalogue queries do not scan dataset payloads.

21.7 THE design SHALL permit a KSDS to use a dedicated SQLite database or another provider when isolation, scale, backup, or contention requirements justify it.

> **Implementation scope (Phase BS.4):** This phase implements the dedicated
> SQLite record-provider base and the primary KSDS operations in criteria
> 21.1-21.4 and 21.6. Alternate indexes (21.5) and provider substitution
> policy (21.7) remain documented extension points for later phases.

---

### Requirement 22: VSAM RRDS Support

**User Story:** As a mainframe developer, I want VSAM Relative-Record Dataset emulation so that applications using relative record number access work correctly.

**Source:** [VFS-REQ] §7.6 FFW-VFS-RRDS-001 to RRDS-003; [ARCH] §8 VSAM Architecture.

#### Acceptance Criteria

22.1 THE system SHALL implement an RRDS provider using a SQLite-backed record store keyed by relative record number.

22.2 THE provider SHALL distinguish an unallocated relative record slot from an allocated record containing zero or blank content — these two states SHALL be distinguishable by the caller.

22.3 THE provider SHALL support: direct retrieval by relative record number, replacement, deletion, and sequential iteration.

---

### Requirement 23: VSAM ESDS Support

**User Story:** As a mainframe developer, I want VSAM Entry-Sequenced Dataset emulation so that append-oriented workloads are supported.

**Source:** [VFS-REQ] §7.7 FFW-VFS-ESDS-001 to ESDS-004; [ARCH] §8 VSAM Architecture.

#### Acceptance Criteria

23.1 THE system SHALL implement an ESDS provider storing records in insertion order in an append-oriented native file.

23.2 THE provider SHALL issue a stable record address or equivalent logical token for each appended record.

23.3 WHEN a sidecar index is used, it SHALL be rebuildable from the data file or protected by an integrity and recovery mechanism.

23.4 Update and deletion semantics SHALL be explicitly documented; if they differ from append-only behaviour the documentation SHALL describe the exact behaviour.

---

### Requirement 24: ISAM Support

**User Story:** As a mainframe developer, I want ISAM-style indexed file emulation so that legacy indexed file access patterns are supported.

**Source:** [VFS-REQ] §7.8 FFW-VFS-ISAM-001 to ISAM-003.

#### Acceptance Criteria

24.1 ISAM-style files SHALL use the common indexed-record interface shared with KSDS.

24.2 THE default ISAM provider SHALL use SQLite indexes for primary and secondary access paths.

24.3 ISAM implementation details SHALL remain encapsulated behind the `StorageProvider` interface so a future native B-tree provider can be introduced without changing callers.

---

### Requirement 25: Staged Transaction Protocol

**User Story:** As a platform developer, I want a staged transaction protocol for operations that span both SQLite and the filesystem so that interrupted operations leave the system in a recoverable state rather than a corrupt one.

**Source:** [VFS-REQ] §11 FFW-VFS-TXN-001 to TXN-006.

#### Acceptance Criteria

25.1 WHEN a dataset is created, THE system SHALL: (a) stage the physical content in the `datasets/staging/` area, (b) create or reserve the catalogue entry, (c) publish the physical object to its final location, (d) mark the catalogue entry active — in that order.

25.2 WHEN a dataset is deleted, THE system SHALL: (a) mark the catalogue entry pending-deletion, (b) move or tombstone the physical content where practical, (c) finalise catalogue state — in that order.

25.3 Interrupted operations SHALL be discoverable through operation journals or transitional catalogue states on the next startup.

25.4 WHEN the system starts, THE system SHALL detect incomplete operations and offer deterministic recovery — either completing or rolling back each incomplete operation.

25.5 Concurrent modification SHALL be controlled using SQLite transactions, version tokens, provider-specific locking, or a documented combination.

25.6 THE system SHALL NOT report an operation as successful until both catalogue and provider state satisfy the operation's postconditions.

---

### Requirement 26: Integrity, Backup, and Restore

**User Story:** As a workbench user, I want workspace backup and restore to capture all catalogue and physical content together so that I can recover a complete, consistent workspace after a failure or migration.

**Source:** [VFS-REQ] §12 FFW-VFS-INT-001 to INT-006.

#### Acceptance Criteria

26.1 THE system SHALL support optional checksums on managed content to detect unexpected physical modification or corruption.

26.2 A workspace backup SHALL capture: the catalogue database, all SQLite record stores, all native dataset files, all library directories, and operation journals — as one recoverable unit.

26.3 A backup SHALL include a manifest containing: schema version, provider configuration, object inventory, and integrity information.

26.4 Restore SHALL support restoration to the original workspace or remapping to a different root without changing logical dataset names.

26.5 THE system SHALL provide diagnostics for orphaned physical objects (present on disk but absent from catalogue) and dangling catalogue entries (in catalogue but missing on disk).

26.6 Repair operations SHALL be previewable, auditable, and reversible where practical.

---

### Requirement 27: Catalogue Reconciliation

**User Story:** As a workbench administrator, I want a reconciliation operation that compares catalogue state with provider state and reports discrepancies without automatically changing data, so that I can review and approve corrections.

**Source:** [VFS-REQ] §7.1 FFW-VFS-CAT-007 to CAT-008; [VFS-REQ] §8 FFW-VFS-SPI-001 reconcile.

#### Acceptance Criteria

27.1 THE system SHALL provide a reconciliation operation that compares catalogue entries with the physical objects reported by each provider.

27.2 THE reconciliation operation SHALL detect: entries whose physical objects are missing, inaccessible, duplicated, or inconsistent.

27.3 THE reconciliation operation SHALL report proposed corrective actions without automatically changing data — the user SHALL approve each correction.

27.4 THE catalogue SHALL record create, rename, move, delete, restore, import, export, and allocation changes in an audit trail.

27.5 Schema changes SHALL be versioned and performed through forward migration scripts; no schema change SHALL be applied without a corresponding migration.

---

### Requirement 28: Security — Path Safety and Audit

**User Story:** As a security-conscious operator, I want all physical paths constrained to authorised workspace roots and all sensitive data excluded from logs so that the workbench cannot be used to traverse or leak filesystem content.

**Source:** [VFS-REQ] §13 FFW-VFS-SEC-001 to SEC-006.

#### Acceptance Criteria

28.1 ALL resolved physical paths SHALL be constrained to authorised workspace roots unless the user explicitly mounts an external root.

28.2 Path canonicalisation and traversal checks SHALL occur before any filesystem access.

28.3 Catalogue metadata SHALL NOT be treated as a substitute for host operating-system access controls.

28.4 Sensitive dataset contents and credentials SHALL NOT be written to logs.

28.5 ALL SQLite connections SHALL use parameterised statements; schema identifiers SHALL be controlled and not interpolated from user input.

28.6 Audit events SHALL identify: action, object, outcome, timestamp, and initiating principal or process where available.

---

### Requirement 29: Catalogue Hierarchy — Master and User Catalogues

**User Story:** As a mainframe developer, I want master and user catalogue concepts so that the catalogue hierarchy mirrors z/OS conventions and multi-project environments can be organised cleanly.

**Source:** [VFS-REQ] §7.1 FFW-VFS-CAT-003; [ARCH] §3 Catalogue Architecture.

#### Acceptance Criteria

29.1 THE catalogue SHALL support master and user-catalogue concepts, or an equivalent scoped catalogue hierarchy, so that datasets can be organised by project or ownership scope.

29.2 THE catalogue SHALL map each managed logical dataset name to exactly one active storage provider and provider-specific locator within a catalogue scope.

29.3 THE catalogue SHALL support logical rename and physical relocation as separate operations — renaming a dataset SHALL NOT require moving its physical content.

29.4 THE catalogue SHALL validate uniqueness according to the configured naming scope and collation rules.

---

### Requirement 30: Non-Functional — Portability, Git Compatibility, and Data Fidelity

**User Story:** As a developer, I want the storage architecture to work identically on Windows, Linux, and macOS, to be compatible with Git for text-oriented members, and to never silently alter bytes or record boundaries.

**Source:** [VFS-REQ] §14 FFW-VFS-NFR-001 to NFR-008.

#### Acceptance Criteria

30.1 THE architecture SHALL operate on Windows, Linux, and macOS without changing the logical dataset model.

30.2 Catalogue listing SHALL query metadata without loading dataset payloads.

30.3 THE design SHALL permit large datasets and large libraries without placing all content into the central catalogue database.

30.4 Catalogue, codec, and provider components SHALL be independently testable using temporary workspaces and deterministic fixtures.

30.5 Storage operations SHALL emit structured diagnostic events with correlation identifiers.

30.6 A future storage provider SHALL be addable without rewriting dataset editors or catalogue consumers.

30.7 Text-oriented PDS/PDSE members and selected sequential datasets SHALL be capable of being represented as ordinary files suitable for external version-control tooling (Git compatibility).

30.8 THE system SHALL NOT silently alter bytes, encoding, record boundaries, keys, or generation identity — any conversion SHALL require an explicit codec and encoding policy.
