# Requirements Document

> **Governance Reference:** This specification is governed by the [Dataset Ownership Model](./../dataset-ownership-model/requirements.md) (ADR-001). Where this document conflicts with the governance document, the governance document takes precedence. The dataset-catalog crate is the single authority for dataset metadata, catalog entries, naming validation, and resolution APIs.

## Introduction

This feature specifies the Dataset Catalog subsystem for FileForgeWorkbench (`ff-dataset-catalog` crate). The Dataset Catalog provides **mainframe dataset filesystem emulation on the local desktop** — enabling developers to work with mainframe-style dataset naming, organization, and management without access to a z/OS system.

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

### Requirement 4: Repository Layout

**User Story:** As a workbench user, I want dataset content stored in a well-defined directory structure on my local filesystem, so that I can understand where files are physically stored, back them up with standard tools, and verify content outside the workbench if needed.

**Source:** [DSC] §6 — Repository directory structure. [DSC, WB]

#### Acceptance Criteria

1. A Repository SHALL be a directory with the following structure at its root: `catalog.db` (SQLite catalog database), `storage/` (sequential dataset files), `pds/` (partitioned dataset directories), `gdg/` (GDG base directories with generation subdirectories), and `temp/` (temporary allocations, automatically cleaned on startup).
2. SEQUENTIAL datasets SHALL be stored as files within `storage/`, using a filesystem-safe encoding of the DSN as the filename (dots replaced with directory separators or a configurable encoding scheme that preserves the qualifier structure).
3. PARTITIONED datasets SHALL be stored as directories within `pds/`, with the directory name derived from the DSN; each member SHALL be stored as an individual file within that directory using the member name as the filename (uppercased).
4. GDG bases SHALL be stored as directories within `gdg/`, with each generation stored as a file or directory (depending on generation DSORG) within the GDG base directory, named using the generation identifier format `GnnnnVnn`.
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
6. THE system SHALL support renaming a dataset by specifying the current DSN and new DSN; renaming SHALL update the catalog entry and, if physical paths are DSN-derived, rename the physical storage path accordingly.
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
