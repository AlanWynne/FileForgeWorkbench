# Implementation Plan: Dataset Catalog (`ff-dscatalog`)

## Overview

This task plan implements the `ff-dscatalog` crate ? the mainframe dataset filesystem emulation subsystem for FileForgeWorkbench. It provides SQLite-backed catalog management, mainframe-style dataset naming (HLQ.qualifier format), dataset CRUD operations, PDS member management, GDG lifecycle, and VFS provider integration under the `catalog` scheme.

**Crate location:** `crates/ff-dscatalog`
**Upstream dependencies:** `ff-vfs` (Wave 3), `ff-config` (Wave 2), `ff-command` (Wave 2), `ff-logging` (Wave 0)
**Downstream consumers:** `file-tree-panel`, `fileforge-integration`, `dataset-allocator`, `FFW-JES`

---

## Tasks

- [x] 1. Project scaffold and SQLite schema
  - [x] 1.1 Create `crates/ff-dataset-catalog/Cargo.toml` with dependencies (rusqlite with bundled feature, tokio, async-trait, thiserror, chrono, serde, serde_json, zip, walkdir) and dev-dependencies (proptest, tempfile, pretty_assertions, tokio-test)
  - [x] 1.2 Create `crates/ff-dataset-catalog/src/lib.rs` with crate-level doc comment and public module declarations
  - [x] 1.3 Implement `src/error.rs` ? define `CatalogError` enum with variants (DsnValidation, DuplicateDataset, DatasetNotFound, CatalogNotMounted, CatalogAlreadyMounted, RepositoryCorrupt, SchemaVersionMismatch, IoError, SqliteError, GdgLimitExceeded, MemberNotFound, MemberAlreadyExists, InvalidAllocationParams, ExportError, ImportError)
  - [x] 1.4 Implement `src/schema.rs` ? define SQL migration strings for `catalog_metadata`, `datasets`, `gdg_bases`, `gdg_generations` tables with correct constraints and indices
  - [x] 1.5 Implement `initialize_database(path)` function ? create SQLite database with WAL journal mode, execute schema migrations, insert default metadata
  - [x] 1.6 Write unit tests for schema creation, WAL mode verification, and idempotent migration execution
    - Validates: Requirement 1 AC 1, AC 5, AC 6, AC 7, AC 8, AC 9

- [x] 2. Dataset Name (DSN) validation
  - [x] 2.1 Implement `src/dsn.rs` ? define `Dsn` newtype struct wrapping a validated, uppercased String; derive Clone, Eq, Hash, Debug
  - [x] 2.2 Implement `Dsn::parse(input)` ? validate total length =44, split by `.`, validate each qualifier (1?8 chars, starts A-Z/@/#/$, followed by A-Z/0-9/@/#/$), reject leading/trailing/consecutive dots; return `CatalogError::DsnValidation` with position info on failure
  - [x] 2.3 Implement `Dsn::with_default_hlq(bare, hlq)` ? prepend configured default HLQ to a bare qualifier
  - [x] 2.4 Implement `Dsn::parse_member_ref(input)` ? parse `DSN(MEMBER)` syntax into `(Dsn, MemberName)` tuple
  - [x] 2.5 Implement `MemberName` newtype ? validate 1?8 characters with same single-qualifier rules
  - [x] 2.6 Implement `Display` for `Dsn` and `MemberName`, `FromStr` delegating to `parse()`
  - [x] 2.7 Write unit tests for valid/invalid DSN parsing, case normalization, member reference parsing, default HLQ prepend
    - Validates: Requirement 2 AC 1?9
  - [x] 2.8 Write property test: DSN round-trip (Property 1) ? generate valid DSN strings, parse into Dsn, Display back, assert equality
    - Validates: Requirement 2 AC 1, AC 2, AC 5
  - [x] 2.9 Write property test: DSN validation rejects invalid inputs (Property 2) ? generate strings violating naming rules, assert DsnValidation error with position info
    - Validates: Requirement 2 AC 4, AC 7
  - [x] 2.10 Write property test: case-insensitive equivalence (Property 3) ? generate DSN in random case, parse both upper and lower, assert equal
    - Validates: Requirement 2 AC 5

- [x] 3. Repository layout management
  - [x] 3.1 Implement `src/repository.rs` ? define `Repository` struct with `root: PathBuf` and methods for subdirectory access (`storage_dir()`, `pds_dir()`, `gdg_dir()`, `temp_dir()`, `catalog_db_path()`)
  - [x] 3.2 Implement `Repository::initialize(root)` ? create all required subdirectories and catalog database; return error if root exists and is non-empty (unless force flag)
  - [x] 3.3 Implement `Repository::validate()` ? check subdirectory existence and catalog.db presence; return structured errors for each missing element
  - [x] 3.4 Implement `Repository::purge_temp()` ? delete stale files in `temp/` (older than 24 hours or from previous sessions based on PID file)
  - [x] 3.5 Implement DSN-to-path encoding: `dsn_to_storage_path(dsn)` ? dots as directory separators, percent-encode national chars (`@`?`%40`, `#`?`%23`, `$`?`%24`)
  - [x] 3.6 Implement `path_to_dsn(path)` ? reverse the encoding for display/debugging
  - [x] 3.7 Write unit tests for repository initialization, validation, temp purge, path encoding/decoding round-trips
    - Validates: Requirement 4 AC 1?9
  - [x] 3.8 Write property test: DSN-to-path round-trip (Property 4) ? generate valid DSNs, encode to path, decode back, assert equal
    - Validates: Requirement 4 AC 2, AC 5, AC 7

- [x] 4. Catalog lifecycle ? mount and unmount
  - [x] 4.1 Implement `src/catalog.rs` ? define `Catalog` struct holding `name: String`, `repository: Repository`, `db: Connection` (rusqlite), `priority: u32`
  - [x] 4.2 Implement `Catalog::mount(path)` ? open repository, validate structure and schema, open SQLite connection with WAL mode, return mounted Catalog instance
  - [x] 4.3 Implement `src/catalog_registry.rs` ? define `CatalogRegistry` managing multiple mounted catalogs with priority ordering
  - [x] 4.4 Implement `CatalogRegistry::mount(path, priority)` ? mount catalog, add to registry in priority order, emit log event
  - [x] 4.5 Implement `CatalogRegistry::unmount(name_or_path)` ? remove from registry, close SQLite connection, emit log event; open files remain accessible but new resolves fail
  - [x] 4.6 Implement `CatalogRegistry::resolve(dsn)` ? search mounted catalogs in priority order (highest first), return first match with catalog identity
  - [x] 4.7 Implement `CatalogRegistry::list_mounted()` ? return names, paths, priorities of all mounted catalogs
  - [x] 4.8 Write unit tests for mount/unmount lifecycle, priority ordering, resolve precedence with duplicate DSNs, validation failure on corrupt repos
    - Validates: Requirement 5 AC 1?8
  - [x] 4.9 Write property test: catalog priority resolution (Property 5) ? mount N catalogs with same DSN at different priorities, assert highest-priority wins
    - Validates: Requirement 5 AC 3

- [x] 5. Catalog add, remove, export, and import
  - [x] 5.1 Implement `CatalogRegistry::create(name, path, options)` ? initialize repository, create empty catalog database, optionally auto-mount; store metadata (description, default_hlq)
  - [x] 5.2 Implement `CatalogRegistry::remove(name, delete_files)` ? unmount if mounted, optionally delete repository directory tree; require explicit confirmation flag for deletion
  - [x] 5.3 Implement `src/export.rs` ? define `CatalogExporter` that walks the repository, creates a ZIP archive with `manifest.json` (name, description, timestamp, dataset count, total size, schema version)
  - [x] 5.4 Implement `CatalogExporter::export(catalog, output_path, progress_cb)` ? stream repository into ZIP with progress reporting
  - [x] 5.5 Implement `src/import.rs` ? define `CatalogImporter` that validates archive integrity (manifest presence, schema version, referenced files), extracts to target directory
  - [x] 5.6 Implement `CatalogImporter::import(archive_path, target_dir, progress_cb)` ? extract archive, validate, optionally auto-mount the imported catalog
  - [x] 5.7 Write unit tests for create/remove lifecycle, export/import round-trip, manifest validation, schema version mismatch rejection, progress callback invocation
    - Validates: Requirement 6 AC 1?10
  - [x] 5.8 Write property test: export-import round-trip (Property 6) ? create catalog with N datasets, export, import to new location, verify all datasets resolve identically
    - Validates: Requirement 6 AC 4, AC 6, AC 7

- [x] 6. Dataset CRUD ? allocate, delete, rename, resolve
  - [x] 6.1 Implement `src/dataset.rs` ? define `DatasetRecord` struct (id, dsn, dsorg, storage_path, recfm, lrecl, blksize, created, modified, accessed), `Dsorg` enum (PS, PO, GDG), `Recfm` enum (F, FB, V, VB, U), `AllocParams` struct
  - [x] 6.2 Implement `Catalog::allocate(params)` ? validate DSN uniqueness across all mounted catalogs, validate allocation params (LRECL>0 =32760, BLKSIZE=LRECL, valid RECFM, GDG limit 1?255), create physical storage, insert catalog entry
  - [x] 6.3 Implement physical storage creation: PS ? file in `storage/`, PO ? directory in `pds/`, GDG ? entry in `gdg_bases` table (no physical allocation)
  - [x] 6.4 Implement `Catalog::delete(dsn)` ? remove catalog entry, delete physical storage (file for PS, directory+members for PDS atomically, all generations for GDG)
  - [x] 6.5 Implement `Catalog::rename(old_dsn, new_dsn)` ? validate new DSN, check uniqueness, update catalog entry, rename physical path if DSN-derived
  - [x] 6.6 Implement `CatalogRegistry::resolve(dsn)` ? search mounted catalogs by priority, return physical path + metadata + catalog identity; return VfsError::NotFound if not in any catalog
  - [x] 6.7 Implement allocation defaults: PS/PO without explicit params ? RECFM=FB, LRECL=80, BLKSIZE=27920; GDG generation inherits from last generation or falls back to sequential defaults
  - [x] 6.8 Write unit tests for allocate/delete/rename/resolve lifecycle, uniqueness enforcement, parameter validation, defaults application, atomic PDS deletion
    - Validates: Requirement 7 AC 1?11; Requirement 15 AC 1?5
  - [x] 6.9 Write property test: allocation parameter validation (Property 7) ? generate random LRECL/BLKSIZE/RECFM combinations, verify only valid combinations succeed
    - Validates: Requirement 7 AC 10
  - [x] 6.10 Write property test: DSN uniqueness across catalogs (Property 8) ? allocate dataset in catalog A, attempt same DSN in catalog B, assert DuplicateDataset error
    - Validates: Requirement 7 AC 3

- [x] 7. PDS member operations
  - [x] 7.1 Implement `src/pds.rs` ? define `PdsMemberInfo` struct (name, size, modified), implement `Catalog::list_members(dsn)` returning sorted member list with metadata
  - [x] 7.2 Implement `Catalog::open_member(dsn, member)` ? resolve PDS directory, construct member file path, validate dataset is PO type, return physical path
  - [x] 7.3 Implement `Catalog::create_member(dsn, member_name, overwrite)` ? validate member name, check PDS exists and is PO, check member uniqueness (unless overwrite=true), create file, update PDS modified date
  - [x] 7.4 Implement `Catalog::delete_member(dsn, member_name)` ? validate member exists, delete physical file, update PDS modified date
  - [x] 7.5 Implement `Catalog::rename_member(dsn, old_name, new_name)` ? validate both names, check source exists, check target doesn't exist, rename physical file, update PDS modified date
  - [x] 7.6 Write unit tests for list/open/create/delete/rename member operations, type consistency validation (reject non-PO), member name validation, overwrite semantics
    - Validates: Requirement 8 AC 1?10
  - [x] 7.7 Write property test: member name validation (Property 9) ? generate random strings, verify only valid member names (1?8 chars, correct charset) are accepted
    - Validates: Requirement 8 AC 3; Requirement 2 AC 8

- [x] 8. GDG management
  - [x] 8.1 Implement `src/gdg.rs` ? define `GdgBase` struct (dsn, limit, scratch, created), `GdgGeneration` struct (number, version, dataset_id, status), `GdgStatus` enum (Active, RolledOff, Deferred)
  - [x] 8.2 Implement `Catalog::create_gdg_base(dsn, limit, scratch)` ? validate limit 1?255, insert into `gdg_bases` table, create directory in `gdg/`
  - [x] 8.3 Implement `Catalog::create_generation(base_dsn, alloc_params)` ? assign next generation number, create storage as `GnnnnVnn`, add catalog entry, link to gdg_generations, trigger roll-off if limit exceeded
  - [x] 8.4 Implement roll-off logic ? when generation count > limit: if scratch=true, delete oldest physical storage and catalog entry; if scratch=false, mark as rolled_off but preserve physical storage
  - [x] 8.5 Implement relative reference resolution: `(+1)` ? allocate new generation (allocation context only); `(0)` ? most recent active; `(-N)` ? Nth from most recent (read context only)
  - [x] 8.6 Implement absolute reference resolution: `GnnnnVnn` ? lookup by generation number and version in gdg_generations table
  - [x] 8.7 Implement `Catalog::list_generations(base_dsn)` ? return active generations sorted newest-to-oldest with metadata
  - [x] 8.8 Implement `Catalog::modify_gdg_base(dsn, new_limit, new_scratch)` ? update properties, trigger additional roll-off if new limit is smaller than current active count
  - [x] 8.9 Implement `Catalog::delete_gdg_base(dsn)` ? delete all generations (active + rolled_off), remove gdg_base entry, clean physical storage
  - [x] 8.10 Write unit tests for GDG base creation, generation creation with roll-off, relative/absolute references, limit modification with cascading roll-off, base deletion
    - Validates: Requirement 9 AC 1?10
  - [x] 8.11 Write property test: GDG roll-off invariant (Property 10) ? create GDG with limit L, add N>L generations, assert active count never exceeds L
    - Validates: Requirement 9 AC 3
  - [x] 8.12 Write property test: GDG relative reference consistency (Property 11) ? create K generations, verify (0) always points to newest, (-1) to second newest, etc.
    - Validates: Requirement 9 AC 4, AC 10

- [x] 9. VFS provider implementation
  - [x] 9.1 Implement `src/vfs_provider.rs` ? define `CatalogVfsProvider` struct wrapping `Arc<CatalogRegistry>`, implement `VfsProvider` trait with scheme `"catalog"`
  - [x] 9.2 Implement `capabilities()` ? advertise Read, Write, List, Metadata, Create, Delete, Rename; do NOT advertise Watch or Search
  - [x] 9.3 Implement `list(path)` ? route based on path depth: empty/root ? list mounted catalogs; catalog name ? list HLQs; partial DSN ? list matching datasets; PDS DSN ? list members
  - [x] 9.4 Implement `stat(path)` ? resolve DSN, return VfsMetadata with size, modified time, resource type (File for PS/GDG gen, Directory for PDS), provider-specific RECFM/LRECL/BLKSIZE/DSORG
  - [x] 9.5 Implement `open(path, options)` ? resolve DSN to physical path, delegate to local filesystem I/O, update last_access_date in catalog
  - [x] 9.6 Implement `read(path)` and `write(path, data)` ? resolve and perform I/O on physical files for PS datasets and PDS members
  - [x] 9.7 Implement `create(path, options)` ? delegate to dataset allocation (parse DSN, extract allocation params from options)
  - [x] 9.8 Implement `delete(path)` ? delegate to dataset deletion
  - [x] 9.9 Implement `rename(old, new)` ? delegate to dataset rename
  - [x] 9.10 Implement `exists(path)` ? check DSN exists in any mounted catalog, return bool without error for non-existent
  - [x] 9.11 Implement error mapping: CatalogError ? VfsError (DatasetNotFound?NotFound, DuplicateDataset?AlreadyExists, DsnValidation?InvalidUri, MemberNotFound?NotFound, CatalogNotMounted?ProviderUnavailable)
  - [x] 9.12 Write unit tests for all VFS operations, error mapping, list hierarchy, stat metadata population, exists semantics
    - Validates: Requirement 10 AC 1?12
  - [x] 9.13 Write property test: VFS error mapping completeness (Property 12) ? generate all CatalogError variants, verify each maps to a valid VfsError variant
    - Validates: Requirement 10 AC 12

- [x] 10. Configuration integration
  - [x] 10.1 Implement `src/config.rs` ? define `CatalogConfig` struct (default_hlq, mounted_catalogs vec, repository_root) deserializable from `[catalog]` TOML table via ff-config
  - [x] 10.2 Implement `MountedCatalogEntry` struct (name, path, priority, auto_mount) for persisted catalog list
  - [x] 10.3 Implement `CatalogConfig::load(config_service)` ? read from ff-config `[catalog]` namespace, validate values (paths exist or creatable, HLQ conforms, priorities positive)
  - [x] 10.4 Implement `CatalogConfig::save(config_service)` ? persist current mount state back to configuration
  - [x] 10.5 Implement auto-mount on startup: iterate `mounted_catalogs` where `auto_mount=true`, mount in priority order, log warnings for failures
  - [x] 10.6 Implement hot-reload callback: subscribe to ff-config reload events for `[catalog]` namespace, apply mount/unmount changes when config changes externally
  - [x] 10.7 Implement `[catalog.defaults]` sub-table for configurable allocation defaults (recfm, lrecl, blksize per dsorg type)
  - [x] 10.8 Write unit tests for config loading, validation, save/restore round-trip, auto-mount sequencing, defaults override
    - Validates: Requirement 14 AC 1?7; Requirement 15 AC 6

- [x] 11. Commands registration
  - [x] 11.1 Implement `src/commands/mod.rs` ? module structure for all catalog commands
  - [x] 11.2 Implement `catalog.mount` command ? params: repository path; mounts catalog and updates config
  - [x] 11.3 Implement `catalog.unmount` command ? params: catalog name or path; unmounts and updates config
  - [x] 11.4 Implement `catalog.create` command ? params: name, path, options (default_hlq, description, auto_mount)
  - [x] 11.5 Implement `catalog.remove` command ? params: catalog name, delete_files bool
  - [x] 11.6 Implement `catalog.export` command ? params: catalog name, output path; invokes CatalogExporter with progress
  - [x] 11.7 Implement `catalog.import` command ? params: archive path, target directory; invokes CatalogImporter with progress
  - [x] 11.8 Implement `dataset.allocate` command ? params: DSN, DSORG, RECFM, LRECL, BLKSIZE, options
  - [x] 11.9 Implement `dataset.delete` command ? params: DSN
  - [x] 11.10 Implement `dataset.rename` command ? params: old DSN, new DSN
  - [x] 11.11 Implement `dataset.properties` command ? params: DSN; returns structured properties result
  - [x] 11.12 Implement `member.create` command ? params: DSN, member name
  - [x] 11.13 Implement `member.delete` command ? params: DSN, member name
  - [x] 11.14 Implement `member.rename` command ? params: DSN, old name, new name
  - [x] 11.15 Implement `gdg.create_base` command ? params: DSN, limit, scratch
  - [x] 11.16 Implement `gdg.create_generation` command ? params: base DSN, DSORG, RECFM, LRECL, BLKSIZE
  - [x] 11.17 Implement `gdg.delete_base` command ? params: DSN
  - [x] 11.18 Implement `gdg.list_generations` command ? params: base DSN
  - [x] 11.19 Write unit tests for command registration, parameter validation, dispatch to correct catalog operations
    - Validates: Requirement 5 AC 8; Requirement 6 AC 9; Requirement 7 AC 11; Requirement 8 AC 10; Requirement 9 AC 9

- [x] 12. LISTCAT and LISTDS commands
  - [x] 12.1 Implement `src/commands/listcat.rs` ? register `catalog.listcat` command; params: filter (DSN pattern with `*` and `%` wildcards), type (optional DSORG filter), catalog (optional catalog name)
  - [x] 12.2 Implement wildcard matching engine: `*` matches zero or more chars across qualifiers, `%` matches exactly one qualifier; convert patterns to matching predicates
  - [x] 12.3 Implement `catalog.listcat` execution ? query mounted catalogs, apply filter, format tabular output (DSN, DSORG, RECFM, LRECL, created, catalog name)
  - [x] 12.4 Implement `src/commands/listds.rs` ? register `catalog.listds` command; params: dsn (required), members (optional bool), history (optional bool)
  - [x] 12.5 Implement `catalog.listds` execution ? resolve DSN, format detailed output including optional member list and history dates; return "DATASET NOT FOUND: {dsn}" on miss
  - [x] 12.6 Write unit tests for LISTCAT wildcard matching, type filtering, LISTDS with and without member/history options, not-found error
    - Validates: Requirement 13 AC 1?10
  - [x] 12.7 Write property test: wildcard matching correctness (Property 13) ? generate DSN patterns and dataset lists, verify `*` and `%` semantics match specification
    - Validates: Requirement 13 AC 9

- [x] 13. Properties panel data provider
  - [x] 13.1 Implement `src/properties.rs` ? define `DatasetProperties` enum with variants for each dataset type (Sequential, Partitioned, GdgBase, GdgGeneration, PdsMember) carrying appropriate fields
  - [x] 13.2 Implement `Catalog::get_properties(dsn)` ? resolve DSN, populate properties based on DSORG, omit non-applicable fields
  - [x] 13.3 Implement `Catalog::get_member_properties(dsn, member)` ? return member-specific properties (name, parent PDS DSN, size, modified, physical path)
  - [x] 13.4 Write unit tests for properties retrieval for each dataset type, field omission for non-applicable attributes
    - Validates: Requirement 11 AC 1?9

- [x] 14. Context menu definitions
  - [x] 14.1 Implement `src/context_menu.rs` ? define `CatalogContextMenu` struct with methods that return command descriptors for each node type
  - [x] 14.2 Implement menu generation per node type: catalog node (unmount, new dataset, properties, export, refresh), PS node (open, rename, delete, properties, copy DSN, allocate like), PO node (expand, new member, rename, delete, properties, copy DSN, allocate like), member node (open, rename, delete, copy name, properties), GDG base (new generation, list, properties, delete, copy DSN, modify limit), GDG generation (open, delete, properties, copy DSN), root node (mount, create, import)
  - [x] 14.3 Implement dynamic enable/disable logic: unmount disabled for non-mounted, delete disabled for read-only
  - [x] 14.4 Write unit tests for menu item generation per node type, enable/disable state logic
    - Validates: Requirement 12 AC 1?9

- [x] 15. Dataset type consistency and access-date tracking
  - [x] 15.1 Implement type-consistency guards in `Catalog::open_member` and VFS open ? reject PS opened as PDS (member access on sequential), reject PDS opened as flat file (sequential read on partitioned)
  - [x] 15.2 Implement access-date tracking: update `last_access_date` in catalog database on every read/open operation
  - [x] 15.3 Write unit tests for type-consistency rejection scenarios, access-date update on read
    - Validates: Requirement 3 AC 7, AC 8

- [x] 16. Integration tests and end-to-end validation
  - [x] 16.1 Write integration test: full lifecycle ? create catalog, mount, allocate PS/PO/GDG datasets, perform CRUD, unmount, remount, verify state persistence
  - [x] 16.2 Write integration test: VFS provider ? register CatalogVfsProvider, perform list/stat/open/read/write/delete via VFS facade, verify correct delegation
  - [x] 16.3 Write integration test: export-import ? create catalog with mixed dataset types, export to ZIP, import to new location, verify full fidelity
  - [x] 16.4 Write integration test: GDG lifecycle ? create base, add generations exceeding limit, verify roll-off, test relative references, modify limit, delete base
  - [x] 16.5 Write integration test: multi-catalog resolution ? mount two catalogs with overlapping DSNs, verify priority-based resolution, unmount higher-priority, verify fallback
    - Validates: All requirements end-to-end

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: SQLite Catalog Database | AC 1 (catalog.db location) | 1.4, 1.5, 3.1 |
| Req 1: SQLite Catalog Database | AC 2 (datasets table) | 1.4, 1.5, 1.6 |
| Req 1: SQLite Catalog Database | AC 3 (gdg_bases table) | 1.4, 1.5 |
| Req 1: SQLite Catalog Database | AC 4 (gdg_generations table) | 1.4, 1.5 |
| Req 1: SQLite Catalog Database | AC 5 (UNIQUE dsn constraint) | 1.4, 1.6 |
| Req 1: SQLite Catalog Database | AC 6 (WAL journal mode) | 1.5, 1.6 |
| Req 1: SQLite Catalog Database | AC 7 (create on first mount) | 1.5, 4.2 |
| Req 1: SQLite Catalog Database | AC 8 (catalog_metadata table) | 1.4, 1.5 |
| Req 1: SQLite Catalog Database | AC 9 (parameterized queries) | 1.4, 6.2 (all SQL operations) |
| Req 2: Dataset Naming | AC 1 (qualifier format) | 2.1?2.7 |
| Req 2: Dataset Naming | AC 2 (qualifier rules) | 2.2, 2.7 |
| Req 2: Dataset Naming | AC 3 (HLQ) | 2.3, 2.7 |
| Req 2: Dataset Naming | AC 4 (validation errors) | 2.2, 2.9 |
| Req 2: Dataset Naming | AC 5 (case-insensitive) | 2.2, 2.10 |
| Req 2: Dataset Naming | AC 6 (default HLQ alias) | 2.3, 10.1 |
| Req 2: Dataset Naming | AC 7 (reject bad dots) | 2.2, 2.9 |
| Req 2: Dataset Naming | AC 8 (member name rules) | 2.5, 7.7 |
| Req 2: Dataset Naming | AC 9 (DSN(MEMBER) syntax) | 2.4, 2.7 |
| Req 3: Dataset Types | AC 1 (PS, PO, GDG) | 6.1, 6.2, 6.3 |
| Req 3: Dataset Types | AC 2 (PS physical storage) | 6.3, 3.5 |
| Req 3: Dataset Types | AC 3 (PO directory storage) | 6.3, 7.1?7.5 |
| Req 3: Dataset Types | AC 4 (GDG base entry) | 8.2 |
| Req 3: Dataset Types | AC 5 (PDS vs PDSE subtype) | 6.1, 6.2 |
| Req 3: Dataset Types | AC 6 (metadata attributes) | 6.1, 9.4 |
| Req 3: Dataset Types | AC 7 (type consistency) | 15.1, 15.3 |
| Req 3: Dataset Types | AC 8 (access date update) | 15.2, 15.3 |
| Req 4: Repository Layout | AC 1 (directory structure) | 3.1, 3.2 |
| Req 4: Repository Layout | AC 2 (PS in storage/) | 3.5, 6.3 |
| Req 4: Repository Layout | AC 3 (PDS in pds/) | 3.5, 6.3 |
| Req 4: Repository Layout | AC 4 (GDG in gdg/) | 8.2, 8.3 |
| Req 4: Repository Layout | AC 5 (storage_path mapping) | 3.5, 6.2 |
| Req 4: Repository Layout | AC 6 (initialize subdirs) | 3.2, 3.7 |
| Req 4: Repository Layout | AC 7 (national char encoding) | 3.5, 3.8 |
| Req 4: Repository Layout | AC 8 (configurable root) | 10.1, 3.1 |
| Req 4: Repository Layout | AC 9 (temp purge) | 3.4, 4.2 |
| Req 5: Mount/Unmount | AC 1 (mount by path) | 4.2, 4.4 |
| Req 5: Mount/Unmount | AC 2 (multiple catalogs) | 4.3, 4.4 |
| Req 5: Mount/Unmount | AC 3 (priority resolution) | 4.6, 4.9 |
| Req 5: Mount/Unmount | AC 4 (unmount behaviour) | 4.5, 4.8 |
| Req 5: Mount/Unmount | AC 5 (validation on mount) | 4.2, 4.8 |
| Req 5: Mount/Unmount | AC 6 (persist mounts) | 10.4, 10.5 |
| Req 5: Mount/Unmount | AC 7 (VFS registration) | 9.1, 4.4 |
| Req 5: Mount/Unmount | AC 8 (commands) | 11.2, 11.3 |
| Req 6: Catalog Add/Remove/Export/Import | AC 1 (create) | 5.1 |
| Req 6: Catalog Add/Remove/Export/Import | AC 2 (create options) | 5.1 |
| Req 6: Catalog Add/Remove/Export/Import | AC 3 (remove) | 5.2 |
| Req 6: Catalog Add/Remove/Export/Import | AC 4 (export ZIP) | 5.3, 5.4 |
| Req 6: Catalog Add/Remove/Export/Import | AC 5 (manifest.json) | 5.3, 5.7 |
| Req 6: Catalog Add/Remove/Export/Import | AC 6 (import) | 5.5, 5.6 |
| Req 6: Catalog Add/Remove/Export/Import | AC 7 (import validation) | 5.5, 5.7 |
| Req 6: Catalog Add/Remove/Export/Import | AC 8 (schema version) | 5.5, 5.7 |
| Req 6: Catalog Add/Remove/Export/Import | AC 9 (commands) | 11.4?11.7 |
| Req 6: Catalog Add/Remove/Export/Import | AC 10 (progress) | 5.4, 5.6 |
| Req 7: Dataset CRUD | AC 1 (allocate params) | 6.1, 6.2 |
| Req 7: Dataset CRUD | AC 2 (allocate success) | 6.2, 6.3 |
| Req 7: Dataset CRUD | AC 3 (duplicate DSN error) | 6.2, 6.10 |
| Req 7: Dataset CRUD | AC 4 (delete) | 6.4 |
| Req 7: Dataset CRUD | AC 5 (PDS atomic delete) | 6.4, 6.8 |
| Req 7: Dataset CRUD | AC 6 (rename) | 6.5 |
| Req 7: Dataset CRUD | AC 7 (rename validation) | 6.5, 6.8 |
| Req 7: Dataset CRUD | AC 8 (resolve) | 6.6 |
| Req 7: Dataset CRUD | AC 9 (resolve not found) | 6.6, 6.8 |
| Req 7: Dataset CRUD | AC 10 (param validation) | 6.2, 6.9 |
| Req 7: Dataset CRUD | AC 11 (commands) | 11.8?11.10 |
| Req 8: PDS Members | AC 1 (list members) | 7.1 |
| Req 8: PDS Members | AC 2 (open member) | 7.2 |
| Req 8: PDS Members | AC 3 (create member) | 7.3, 7.7 |
| Req 8: PDS Members | AC 4 (non-PO error) | 7.3, 7.6 |
| Req 8: PDS Members | AC 5 (delete member) | 7.4 |
| Req 8: PDS Members | AC 6 (rename member) | 7.5 |
| Req 8: PDS Members | AC 7 (not-found error) | 7.2, 7.4, 7.5, 7.6 |
| Req 8: PDS Members | AC 8 (already-exists error) | 7.3, 7.6 |
| Req 8: PDS Members | AC 9 (update PDS modified) | 7.3, 7.4, 7.5 |
| Req 8: PDS Members | AC 10 (commands) | 11.12?11.14 |
| Req 9: GDG Management | AC 1 (create base) | 8.2 |
| Req 9: GDG Management | AC 2 (create generation) | 8.3 |
| Req 9: GDG Management | AC 3 (roll-off) | 8.4, 8.11 |
| Req 9: GDG Management | AC 4 (relative refs) | 8.5, 8.12 |
| Req 9: GDG Management | AC 5 (absolute refs) | 8.6 |
| Req 9: GDG Management | AC 6 (list generations) | 8.7 |
| Req 9: GDG Management | AC 7 (modify base) | 8.8 |
| Req 9: GDG Management | AC 8 (delete base) | 8.9 |
| Req 9: GDG Management | AC 9 (commands) | 11.15?11.18 |
| Req 9: GDG Management | AC 10 (ref validation) | 8.5, 8.12 |
| Req 10: VFS Provider | AC 1 (implement trait) | 9.1 |
| Req 10: VFS Provider | AC 2 (URI resolution) | 9.3, 9.5 |
| Req 10: VFS Provider | AC 3 (list hierarchy) | 9.3, 9.12 |
| Req 10: VFS Provider | AC 4 (stat metadata) | 9.4, 9.12 |
| Req 10: VFS Provider | AC 5 (open delegation) | 9.5 |
| Req 10: VFS Provider | AC 6 (read/write) | 9.6 |
| Req 10: VFS Provider | AC 7 (create as allocate) | 9.7 |
| Req 10: VFS Provider | AC 8 (delete) | 9.8 |
| Req 10: VFS Provider | AC 9 (rename) | 9.9 |
| Req 10: VFS Provider | AC 10 (exists) | 9.10 |
| Req 10: VFS Provider | AC 11 (capabilities) | 9.2 |
| Req 10: VFS Provider | AC 12 (error mapping) | 9.11, 9.13 |
| Req 11: Properties Panel | AC 1 (selected node) | 13.2, 14.1 |
| Req 11: Properties Panel | AC 2 (PS properties) | 13.1, 13.2 |
| Req 11: Properties Panel | AC 3 (PO properties) | 13.1, 13.2 |
| Req 11: Properties Panel | AC 4 (GDG base properties) | 13.1, 13.2 |
| Req 11: Properties Panel | AC 5 (GDG gen properties) | 13.1, 13.2 |
| Req 11: Properties Panel | AC 6 (member properties) | 13.3, 13.4 |
| Req 11: Properties Panel | AC 7 (omit non-applicable) | 13.2, 13.4 |
| Req 11: Properties Panel | AC 8 (dynamic update) | UI integration (file-tree-panel) |
| Req 11: Properties Panel | AC 9 (properties command) | 11.11 |
| Req 12: Context Menus | AC 1 (catalog node menu) | 14.2, 14.4 |
| Req 12: Context Menus | AC 2 (PS node menu) | 14.2, 14.4 |
| Req 12: Context Menus | AC 3 (PO node menu) | 14.2, 14.4 |
| Req 12: Context Menus | AC 4 (member node menu) | 14.2, 14.4 |
| Req 12: Context Menus | AC 5 (GDG base menu) | 14.2, 14.4 |
| Req 12: Context Menus | AC 6 (GDG gen menu) | 14.2, 14.4 |
| Req 12: Context Menus | AC 7 (root node menu) | 14.2, 14.4 |
| Req 12: Context Menus | AC 8 (command dispatch) | 14.2, 14.4 |
| Req 12: Context Menus | AC 9 (enable/disable) | 14.3, 14.4 |
| Req 13: LISTCAT/LISTDS | AC 1 (LISTCAT command) | 12.1 |
| Req 13: LISTCAT/LISTDS | AC 2 (LISTCAT params) | 12.1, 12.3 |
| Req 13: LISTCAT/LISTDS | AC 3 (LISTCAT output) | 12.3, 12.6 |
| Req 13: LISTCAT/LISTDS | AC 4 (LISTDS command) | 12.4 |
| Req 13: LISTCAT/LISTDS | AC 5 (LISTDS params) | 12.4, 12.5 |
| Req 13: LISTCAT/LISTDS | AC 6 (LISTDS output) | 12.5, 12.6 |
| Req 13: LISTCAT/LISTDS | AC 7 (members option) | 12.5, 12.6 |
| Req 13: LISTCAT/LISTDS | AC 8 (not found error) | 12.5, 12.6 |
| Req 13: LISTCAT/LISTDS | AC 9 (wildcard semantics) | 12.2, 12.7 |
| Req 13: LISTCAT/LISTDS | AC 10 (command IDs) | 12.1, 12.4 |
| Req 14: Configuration | AC 1 (TOML namespace) | 10.1 |
| Req 14: Configuration | AC 2 (config keys) | 10.1, 10.2 |
| Req 14: Configuration | AC 3 (auto-mount startup) | 10.5, 10.8 |
| Req 14: Configuration | AC 4 (persist mount state) | 10.4, 10.8 |
| Req 14: Configuration | AC 5 (hot-reload) | 10.6 |
| Req 14: Configuration | AC 6 (user/project level) | 10.1, 10.3 |
| Req 14: Configuration | AC 7 (validation) | 10.3, 10.8 |
| Req 15: Allocation Defaults | AC 1 (PS defaults) | 6.7, 6.8 |
| Req 15: Allocation Defaults | AC 2 (PO defaults) | 6.7, 6.8 |
| Req 15: Allocation Defaults | AC 3 (GDG gen inheritance) | 6.7, 8.3 |
| Req 15: Allocation Defaults | AC 4 (allocate like) | 6.2, 11.8 |
| Req 15: Allocation Defaults | AC 5 (explicit override) | 6.7, 6.8 |
| Req 15: Allocation Defaults | AC 6 (configurable defaults) | 10.7, 10.8 |

---

## Property-Based Test Summary

| Property | Statement | Task | Validates |
|----------|-----------|------|-----------|
| P1 | DSN round-trip: parse ? Display ? parse ? equal | 2.8 | Req 2 AC 1, 2, 5 |
| P2 | DSN validation rejects invalid inputs with position info | 2.9 | Req 2 AC 4, 7 |
| P3 | Case-insensitive DSN equivalence: mixed-case parses equal | 2.10 | Req 2 AC 5 |
| P4 | DSN-to-path encoding round-trip: encode ? decode ? equal | 3.8 | Req 4 AC 2, 5, 7 |
| P5 | Catalog priority resolution: highest-priority wins | 4.9 | Req 5 AC 3 |
| P6 | Export-import round-trip fidelity: all datasets survive | 5.8 | Req 6 AC 4, 6, 7 |
| P7 | Allocation param validation: only valid combos succeed | 6.9 | Req 7 AC 10 |
| P8 | DSN uniqueness enforced across mounted catalogs | 6.10 | Req 7 AC 3 |
| P9 | Member name validation: 1?8 chars, correct charset | 7.7 | Req 8 AC 3; Req 2 AC 8 |
| P10 | GDG roll-off invariant: active count = limit | 8.11 | Req 9 AC 3 |
| P11 | GDG relative reference consistency: (0)=newest, (-N)=Nth oldest | 8.12 | Req 9 AC 4, 10 |
| P12 | VFS error mapping: all CatalogError variants map to valid VfsError | 9.13 | Req 10 AC 12 |
| P13 | LISTCAT wildcard correctness: `*` and `%` match spec semantics | 12.7 | Req 13 AC 9 |

---

## Notes

- Tasks 2 and 3 are independent and can be developed in parallel (both depend only on task 1)
- Task 4 (mount/unmount) depends on tasks 1?3 since mounting requires schema + repository validation + DSN resolution
- Tasks 6 and 7 can be developed in parallel once task 4 is complete
- Task 8 (GDG) depends on task 6 since GDG generations are allocated via the dataset CRUD layer
- Task 9 (VFS provider) depends on tasks 6, 7, and 8 since it delegates to all operation types
- Task 10 (config) can be developed in parallel with tasks 6?8 since it only needs the mount/unmount registry from task 4
- Tasks 11 and 12 (commands) depend on their respective operation implementations being complete
- Tasks 13 and 14 (properties/context menus) are presentation-layer code and depend on the full data layer (tasks 6?8)
- Task 15 (type consistency) can be slotted anywhere after task 7 since it guards PDS member access
- Task 16 (integration tests) runs last as it exercises the full stack
- All property tests use the `proptest` crate with a minimum of 100 iterations
- All async tests use `#[tokio::test]` where applicable
- Physical file operations use `tempfile::TempDir` in tests to avoid polluting the real filesystem
- The mock ff-config and ff-command interfaces should be defined in `tests/support/` for integration tests

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Project scaffold and SQLite schema", "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6"] },
    { "id": 1, "label": "DSN validation and repository layout", "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7", "2.8", "2.9", "2.10", "3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7", "3.8"], "dependsOn": [0] },
    { "id": 2, "label": "Catalog lifecycle ? mount/unmount", "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "4.8", "4.9"], "dependsOn": [1] },
    { "id": 3, "label": "Dataset CRUD, PDS members, and configuration", "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "6.8", "6.9", "6.10", "7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8"], "dependsOn": [2] },
    { "id": 4, "label": "GDG management and export/import", "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8", "8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8", "8.9", "8.10", "8.11", "8.12"], "dependsOn": [3] },
    { "id": 5, "label": "VFS provider and type consistency", "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "9.8", "9.9", "9.10", "9.11", "9.12", "9.13", "15.1", "15.2", "15.3"], "dependsOn": [4] },
    { "id": 6, "label": "Commands, LISTCAT/LISTDS, properties, context menus", "tasks": ["11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7", "11.8", "11.9", "11.10", "11.11", "11.12", "11.13", "11.14", "11.15", "11.16", "11.17", "11.18", "11.19", "12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7", "13.1", "13.2", "13.3", "13.4", "14.1", "14.2", "14.3", "14.4"], "dependsOn": [5] },
    { "id": 7, "label": "Integration tests and end-to-end validation", "tasks": ["16.1", "16.2", "16.3", "16.4", "16.5"], "dependsOn": [6] }
  ]
}
```

---

## Tasks Added by CR-NR-016 ? Mainframe Dataset Architecture

---

- [x] 31. Resolve pre-BS requirements inconsistencies (MUST complete before any BS code)
  - [x] 31.1 Update Requirement 4 (Repository Layout) to note it is superseded by Requirement 20
    (UUID-based layout) for all new allocations; add a cross-reference note at the top of Req 4
    stating: "Physical layout for new datasets is defined by Requirement 20. Requirement 4
    describes the legacy DSN-derived layout retained only for import compatibility."
    - Validates: Resolves CRITICAL conflict between Req 4 AC 2-5 and Req 20 AC 1-5
  - [x] 31.2 Update Requirement 7 AC 6 (rename) to remove the clause "rename the physical storage
    path accordingly" and replace with: "rename SHALL update the catalogue entry only; the
    physical object SHALL NOT be moved or renamed (see Requirement 20.6)."
    - Validates: Resolves CRITICAL contradiction between Req 7 AC 6 and Req 20.6
  - [x] 31.3 Fix crate name throughout this file: replace all occurrences of `ff-dataset-catalog`
    with `ff-dscatalog` to match the actual workspace crate name.
    - Validates: Resolves MEDIUM crate name inconsistency
  - [x] 31.4 Fix crate name in the Introduction section of requirements.md: replace
    `ff-dataset-catalog` with `ff-dscatalog` in the intro paragraph and cross-reference table.
    - Validates: Resolves MEDIUM crate name inconsistency in requirements.md
  - [x] 31.5 Update virtual-catalog-manager Requirement 16.1 to align with UUID layout: replace
    the DSN-to-path mapping rule (dots -> directory separators) with a note that physical path
    resolution delegates to the StorageProvider via the catalogue locator, not DSN-derived paths.
    (Edit docs/specs/virtual-catalog-manager/requirements.md Req 16.1.)
    - Validates: Resolves CRITICAL conflict between VCM Req 16.1 and dataset-catalog Req 20.3/20.5
  - [x] 31.6 Update virtual-catalog-manager Requirement 16.3 to align with UUID layout: the
    "create file and parent directories" behaviour applies only to the legacy DSN-path model;
    under UUID layout, dataset creation goes through the staged protocol (Req 25.1). Add a note
    clarifying which model applies.
    - Validates: Resolves MEDIUM inconsistency in VCM Req 16.3

- [x] 17. Record codecs ? independent module
  - [x] 17.1 Create `src/codecs/mod.rs` ? define `RecordCodec` trait with `encode(records) -> Vec<u8>` and `decode(bytes) -> Vec<Vec<u8>>` methods; no filesystem or SQLite dependency
    - Validates: Requirement 17.1, 17.6
  - [x] 17.2 Implement `FixedCodec` ? encode/decode fixed-length records given LRECL; record `n` at offset `n ? LRECL`; reject bytes not a multiple of LRECL
    - Validates: Requirement 16.2, 17.2
  - [x] 17.3 Implement `VariableCodec` ? encode/decode variable-length records with 4-byte RDW; validate RDW length field; return diagnostic error on malformed RDW with record position
    - Validates: Requirement 16.3, 16.7, 17.3
  - [x] 17.4 Implement `BinaryCodec` ? pass bytes through unchanged for RECFM=U
    - Validates: Requirement 16.4, 17.4
  - [x] 17.5 Implement `TextCodec` ? map host text lines to/from fixed-length records using a configurable encoding profile; mark as import/export only, never applied silently
    - Validates: Requirement 17.5, 17.7
  - [x] 17.6 Write unit tests for all codecs using in-memory byte buffers; include round-trip property tests for FixedCodec and VariableCodec
    - Validates: Requirement 17.6

- [x] 18. StorageProvider trait and native file provider
  - [x] 18.1 Define `StorageProvider` trait in `src/storage/mod.rs` ? `allocate`, `open`, `stat`, `rename`, `delete`, `list`, `reconcile`; declare capabilities enum
    - Validates: Requirement 19.1, 19.2
  - [x] 18.2 Implement `NativeFileProvider` ? allocates PS as `<uuid>.dat`, PDS as `<uuid>/` directory with `<member-uuid>.dat` files, GDG generations as `<uuid>.dat`; uses `datasets/objects/` layout
    - Validates: Requirement 18.1, 18.2, 18.3, 19.5, 20.2
  - [x] 18.3 Implement UUID-based physical object allocation ? generate UUID at allocation time, store in catalogue `physical_locator` column, never derive path from DSN
    - Validates: Requirement 20.1, 20.3, 20.4, 20.6
  - [x] 18.4 Implement path-safety guards in `NativeFileProvider` ? canonicalise all paths, reject traversal outside workspace root, handle reserved names and length limits
    - Validates: Requirement 20.7, 28.1, 28.2
  - [x] 18.5 Write unit tests for `NativeFileProvider` allocation, open, stat, rename, delete, list, and reconcile; use `tempfile::TempDir`
    - Validates: Requirement 19.5

- [x] 19. SQLite record provider ? VSAM KSDS
  - Scope: Phase BS.4; uses `rusqlite` with the `bundled` feature, so no system SQLite installation is required.
  - [x] 19.1 Implement `SqliteRecordProvider` base ? open or create a per-dataset SQLite file in `indexed/<uuid>.sqlite`; WAL mode; parameterised statements only
    - Validates: Requirement 18.4, 21.1, 28.5
  - [x] 19.2 Implement KSDS table schema ? `VSAM_KEY TEXT PRIMARY KEY, RECORD_DATA BLOB`; persist key definition metadata with the indexed database pending catalogue-layer wiring
    - Validates: Requirement 21.1, 21.2, 21.6
  - [x] 19.3 Implement KSDS operations ? keyed read, ordered sequential read, insert, update, delete, range retrieval; enforce primary-key uniqueness transactionally
    - Validates: Requirement 21.3, 21.4
  - [x] 19.4 Implement alternate index support ? additional SQLite indexes or mapping tables within the KSDS database
    - Validates: Requirement 21.5
  - [x] 19.5 Write unit and property tests for KSDS operations; verify uniqueness invariant and ordered traversal
    - Validates: Requirement 21.3, 21.4

- [x] 20. SQLite record provider ? VSAM RRDS
  - [x] 20.1 Implement RRDS table schema ? `RECNO INTEGER PRIMARY KEY, RECORD_DATA BLOB, ALLOCATED BOOLEAN`; distinguish unallocated slot from allocated blank record
    - Validates: Requirement 22.1, 22.2
  - [x] 20.2 Implement RRDS operations ? direct retrieval, replacement, deletion, sequential iteration by relative record number
    - Validates: Requirement 22.3
  - [x] 20.3 Write unit tests for RRDS; verify unallocated vs allocated blank distinction
    - Validates: Requirement 22.2

- [x] 21. Native file provider ? VSAM ESDS
  - [x] 21.1 Implement ESDS provider ? append-oriented native file; issue stable record address (byte offset) for each appended record
    - Validates: Requirement 23.1, 23.2
  - [x] 21.2 Implement optional sidecar index ? rebuildable from data file; integrity check on open
    - Validates: Requirement 23.3
  - [x] 21.3 Document update and deletion semantics explicitly in code comments and design.md
    - Validates: Requirement 23.4
  - [x] 21.4 Write unit tests for ESDS append, address stability, and sidecar rebuild
    - Validates: Requirement 23.1, 23.2, 23.3

- [x] 22. ISAM support
  - [x] 22.1 Implement ISAM provider using `SqliteRecordProvider` with SQLite indexes for primary and secondary access paths; share the common indexed-record interface with KSDS
    - Validates: Requirement 24.1, 24.2
  - [x] 22.2 Encapsulate ISAM implementation behind `StorageProvider` interface ? no ISAM-specific types leak to callers
    - Validates: Requirement 24.3
  - [x] 22.3 Write unit tests for ISAM primary and secondary access
    - Validates: Requirement 24.1, 24.2

- [x] 23. Staged transaction protocol
  - [x] 23.1 Implement `src/transactions.rs` -- define `OperationJournal` that records in-progress operations with transitional states (staging, reserved, published, active, pending-delete, tombstoned)
    - Validates: Requirement 25.3, 25.4
  - [x] 23.2 Implement staged create protocol -- stage to `datasets/staging/`, reserve catalogue entry, publish to `datasets/objects/`, mark active; roll back on any step failure
    - Validates: Requirement 25.1
  - [x] 23.3 Implement staged delete protocol -- mark pending-deletion in catalogue, tombstone/move physical content to `recovery/`, finalise catalogue state
    - Validates: Requirement 25.2
  - [x] 23.4 Implement startup recovery scan -- detect incomplete operations from journal, offer complete-or-rollback for each
    - Validates: Requirement 25.4
  - [x] 23.5 Implement version tokens for concurrent modification control -- optimistic locking on catalogue rows
    - Validates: Requirement 25.5
  - [x] 23.6 Write unit and integration tests for staged create, staged delete, interrupted-create recovery, interrupted-delete recovery
    - Validates: Requirement 25.1, 25.2, 25.4, 25.6

- [x] 24. Integrity, backup, and restore
  - [x] 24.1 Implement optional checksums -- SHA-256 of physical object content stored in catalogue; verify on open when enabled
    - Validates: Requirement 26.1
  - [x] 24.2 Implement `workspace.backup` command -- capture catalogue DB, all `indexed/*.sqlite` files, all `datasets/objects/` content, operation journal; write manifest with schema version, provider config, object inventory, checksums
    - Validates: Requirement 26.2, 26.3
  - [x] 24.3 Implement `workspace.restore` command -- validate manifest, restore to original root or remap to new root without changing logical names
    - Validates: Requirement 26.4
  - [x] 24.4 Implement `workspace.diagnose` command -- report orphaned physical objects and dangling catalogue entries
    - Validates: Requirement 26.5
  - [x] 24.5 Implement `workspace.reconcile` command -- compare catalogue state with provider state, report proposed corrections without auto-applying
    - Validates: Requirement 27.1, 27.2, 27.3
  - [x] 24.6 Write unit and integration tests for backup/restore round-trip, diagnose output, reconcile report
    - Validates: Requirement 26.2, 26.3, 26.4, 26.5

- [x] 25. Catalogue audit trail and schema migrations
  - [x] 25.1 Add `audit_log` table to catalogue schema ? columns: `id`, `action`, `object_dsn`, `outcome`, `timestamp`, `principal`; insert row for every create/rename/move/delete/restore/import/export/allocate
    - Validates: Requirement 27.4, 28.6
  - [x] 25.2 Implement forward migration scripts ? version the schema; apply migrations on mount when schema version is behind current
    - Validates: Requirement 27.5
  - [x] 25.3 Write unit tests for audit log insertion and schema migration application
    - Validates: Requirement 27.4, 27.5

- [x] 26. Security hardening
  - [x] 26.1 Audit all SQLite operations ? replace any string-interpolated queries with parameterised statements; add `#[deny(clippy::format_collect)]` guard
    - Validates: Requirement 28.5
  - [x] 26.2 Implement log scrubbing ? ensure no dataset payload bytes or credentials appear in `log_info!`/`log_debug!` output; add test that verifies log output for a write operation contains no payload
    - Validates: Requirement 28.4
  - [x] 26.3 Write property test: path traversal rejection ? generate random strings containing `..`, `//`, Windows reserved names; verify all are rejected by path-safety guard
    - Validates: Requirement 28.1, 28.2, 20.7

- [x] 27. Master catalogue hierarchy
  - [x] 27.1 Implement scoped catalogue hierarchy -- `CatalogScope` enum (Master, User); catalogue entries carry scope; resolution checks scope before priority order
    - Validates: Requirement 29.1, 29.2
  - [x] 27.2 Implement logical rename as catalogue-only operation -- update DSN in catalogue without moving physical object
    - Validates: Requirement 29.3, 20.6
  - [x] 27.3 Implement uniqueness validation per scope and collation rule
    - Validates: Requirement 29.4
  - [x] 27.4 Write unit tests for scoped resolution, logical rename, uniqueness enforcement
    - Validates: Requirement 29.1, 29.2, 29.3, 29.4

- [x] 28. Record-oriented editor integration
  - [x] 28.1 Wire `FixedCodec` and `VariableCodec` into the dataset open/save path in `CatalogVfsProvider` ? decode on read, encode on write; never apply `TextCodec` silently
    - Validates: Requirement 16.1, 16.5, 16.6
  - [x] 28.2 Wire codec selection from RECFM metadata ? F/FB ? FixedCodec, V/VB ? VariableCodec, U ? BinaryCodec
    - Validates: Requirement 16.2, 16.3, 16.4
  - [x] 28.3 Write integration tests: open FB dataset, edit a record, save, reopen ? verify bytes match expected fixed-length encoding with no CRLF
    - Validates: Requirement 16.1, 16.2, 16.6
  - [x] 28.4 Write integration tests: open VB dataset, edit a record, save, reopen ? verify RDW headers are correct and no CRLF present
    - Validates: Requirement 16.3, 16.6

- [x] 29. Non-functional validation
  - [x] 29.1 Write cross-platform path tests ? verify UUID-based layout produces identical logical results on Windows, Linux, and macOS path conventions using `std::path::Path` abstractions
    - Validates: Requirement 30.1, 20.7
  - [x] 29.2 Write performance test ? catalogue listing of 1,000 datasets completes without loading any payload bytes; verify via mock provider that `stat` is called but `read` is not
    - Validates: Requirement 30.2
  - [x] 29.3 Write Git-compatibility test ? allocate a PDS, create two members, verify member files are plain text files readable by `git diff` without workbench involvement
    - Validates: Requirement 30.7
  - [x] 29.4 Write data-fidelity property test ? generate random binary content, write to a PS dataset, read back, assert byte-for-byte equality with no alteration
    - Validates: Requirement 30.8

- [x] 30. Update design.md for CR-NR-016
  - [x] 30.1 Add section to `docs/specs/dataset-catalog/design.md` documenting: UUID-based layout, StorageProvider layer, codec separation, staged transaction protocol, VSAM/ISAM provider map, audit trail schema
    - Validates: All CR-NR-016 requirements (design documentation)

## Tasks Added by CR-NR-017 -- Catalog Location Discriminant

---

- [x] 32. CatalogLocation enum and CatalogMount refactor
  - [x] 32.1 Define `CatalogLocation` enum in `src/catalog.rs` (or `src/location.rs`) with
    `Local { path: PathBuf }` and `Remote { scheme: String, uri: String }` variants;
    mark `#[non_exhaustive]`; derive `Debug`, `Clone`, `PartialEq`, `Eq`.
    - Validates: Requirement 31.1, 31.7
  - [x] 32.2 Add `local_path() -> Option<&Path>` method to `CatalogLocation`.
    - Validates: Requirement 31.8
  - [x] 32.3 Replace `path: PathBuf` field on `CatalogMount` with `location: CatalogLocation`;
    add `local_path()` convenience method on `CatalogMount` delegating to `location.local_path()`.
    - Validates: Requirement 31.2, 31.8
  - [x] 32.4 Update all `CatalogMount` construction sites to use
    `CatalogLocation::Local { path }` -- no behaviour change for local catalogs.
    - Validates: Requirement 31.3
  - [x] 32.5 Update `CatalogManager::mount()` to extract the local path via `local_path()`;
    return `CatalogError::UnsupportedOperation` when location is `Remote`.
    - Validates: Requirement 31.4
  - [x] 32.6 Update TOML serialisation/deserialisation in `src/config.rs`:
    - Add `location: String` field (default `"local"`) and `uri: Option<String>` to
      `MountedCatalogEntry`; deserialise into `CatalogLocation`.
    - WHEN `location` field is absent, default to `Local` for backward compatibility.
    - Validates: Requirement 31.5, 31.6
  - [x] 32.7 Write unit tests:
    - `catalog_location_local_path_returns_path` -- Local variant returns Some(path).
    - `catalog_location_remote_local_path_returns_none` -- Remote variant returns None.
    - `catalog_mount_local_path_delegates_to_location` -- CatalogMount.local_path() works.
    - `catalog_mount_toml_round_trip_local` -- local entry serialises and deserialises correctly.
    - `catalog_mount_toml_missing_location_defaults_to_local` -- absent field defaults to Local.
    - `catalog_mount_remote_returns_unsupported_operation` -- mount with Remote location errors.
    - Validates: Requirement 31.3, 31.4, 31.6, 31.9
  - [x] 32.8 Run `cargo test -p ff-dscatalog` -- all existing tests must continue to pass.
    - Validates: Requirement 31.9
