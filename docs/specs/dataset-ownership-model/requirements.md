# Requirements Document

## Introduction

This specification defines the **Dataset Ownership Model** — a cross-cutting architectural governance document that establishes single-authority ownership boundaries, interface contracts, and dependency rules for all dataset-related subsystems in FileForgeWorkbench. It codifies the decisions recorded in ADR-001 (Dataset Ownership Model) as formal, testable requirements.

This is NOT a rewrite of the individual subsystem specifications. It is the authoritative source for:

1. **Ownership boundaries** — what each subsystem owns and what it must delegate
2. **Interface contracts** — the APIs through which subsystems communicate
3. **Dependency direction** — permitted and prohibited dependency relationships
4. **Conflict resolution** — explicit resolution of overlapping responsibilities identified across the `ff-vfs`, `ff-dataset-catalog`, `ff-dataset-allocator`, `ff-idcams`, and `ff-vsam-services` specifications

The individual subsystem specs (`dataset-catalog`, `dataset-allocator`, `IDCAMS-Emulator`, `virtual-file-system`) SHALL be updated to align with this governance document. Where a subsystem spec conflicts with this document, this document takes precedence.

**ADR Reference:** ADR-001 — Dataset Ownership Model

---

## Glossary

- **Ownership**: The exclusive authority to implement, persist, and expose a specific capability. The owning subsystem is the single source of truth for that capability. [ADR-001]
- **Delegation**: The act of invoking another subsystem's API to perform an operation that the caller does not own. The caller SHALL NOT implement the delegated operation internally. [ADR-001]
- **Authority_Rule**: A constraint specifying that all operations of a given type MUST flow through the owning subsystem's API. Violations of authority rules are architectural defects. [ADR-001]
- **Dependency_Direction**: The permitted call direction between subsystems. If A → B is permitted, A may invoke B's API. If A → B is prohibited, A SHALL NOT depend on or invoke B. [ADR-001]
- **ff-vfs**: The Virtual File System crate — owns resource URIs, provider registration, provider routing, resource access abstraction, provider capabilities, file watching, and search abstraction. [ADR-001]
- **ff-dataset-catalog**: The Dataset Catalog crate — owns dataset definitions, catalog entries, dataset attributes, dataset aliases, GDG catalog metadata, dataset resolution APIs, and dataset naming validation. [ADR-001]
- **ff-dataset-allocator**: The Dataset Allocator crate — owns DD statement interpretation, DISP processing, symbolic substitution, referback resolution, GDG reference resolution, and allocation workflows. [ADR-001]
- **ff-vsam-services**: The VSAM Services crate (future) — owns KSDS behaviour, ESDS behaviour, RRDS behaviour, LDS behaviour, alternate indexes, record insertion, record retrieval, and key management. [ADR-001]
- **ff-idcams**: The IDCAMS Emulator crate — owns DEFINE command parsing, LISTCAT command parsing, ALTER command parsing, DELETE command parsing, REPRO command parsing, IMPORT command parsing, and EXPORT command parsing. [ADR-001]
- **Interface_Contract**: A defined API surface through which one subsystem exposes capabilities to others. Contracts specify method signatures, error types, and behavioural guarantees. [ADR-001]
- **Single_Authority_Principle**: The architectural rule that each responsibility has exactly one authoritative owner. No two subsystems SHALL independently implement the same capability. [ADR-001]
- **Dataset_Lifecycle**: The sequence of operations from dataset creation through usage to deletion, with clear ownership at each stage. [ADR-001]
- **Resolution_Lifecycle**: The sequence of operations from DSN reference in JCL through symbolic substitution, catalog lookup, and physical path return. [ADR-001]

---

## Requirements

### Requirement 1: Single-Authority Ownership Principle

**User Story:** As a platform architect, I want each responsibility to have exactly one authoritative owner, so that there are no conflicting implementations, no ambiguous ownership, and consumers always know which subsystem to invoke.

**Source:** ADR-001 — Core architectural principle. [ADR-001]

#### Acceptance Criteria

1. THE architecture SHALL enforce that each capability listed in ADR-001 ownership tables is implemented by exactly one owning subsystem — every listed capability SHALL be implemented (not left unimplemented), and no capability SHALL be implemented by more than one subsystem.
2. WHEN a subsystem needs a capability it does not own, THE subsystem SHALL invoke the owning subsystem's public API rather than implementing the capability internally.
3. THE Single_Authority_Principle SHALL be verifiable at compile time through Rust module visibility and trait boundaries — subsystem internals SHALL NOT be exposed as public API to other crates.
4. IF a new capability is introduced that spans multiple subsystems, THEN THE architecture SHALL assign ownership to exactly one subsystem before implementation begins, documented in an ADR amendment.
5. THE ownership assignments defined in this document SHALL take precedence over any conflicting assignments in individual subsystem specifications until those specifications are updated to align.

---

### Requirement 2: ff-vfs Ownership Boundary

**User Story:** As a platform architect, I want the VFS to own only the abstraction layer (URIs, provider registration, routing, capabilities), so that it remains domain-agnostic and does not reach into dataset, catalog, or VSAM logic.

**Source:** ADR-001 — ff-vfs ownership definition. [ADR-001]

#### Acceptance Criteria

1. THE ff-vfs crate SHALL own: Resource_URI definition and parsing, Provider_Registry (registration, lookup, deregistration), provider routing (URI to provider dispatch), resource access abstraction (the VfsProvider trait), provider capability declarations, file watching abstraction, and search abstraction.
2. THE ff-vfs crate SHALL NOT own or implement: dataset metadata storage, dataset allocation logic, VSAM record structures, catalog management operations, IDCAMS command logic, or JCL parsing.
3. THE ff-vfs crate SHALL NOT contain any `use ff_dataset_catalog`, `use ff_dataset_allocator`, `use ff_idcams`, or `use ff_vsam_services` import statements — it SHALL have zero compile-time dependencies on domain subsystems. Runtime loading of domain modules through plugin systems or dynamic linking is permitted.
4. THE ff-vfs crate SHALL define the VfsProvider trait as the sole integration point for domain subsystems — domain crates integrate with VFS by implementing VfsProvider, not by VFS importing domain logic.
5. WHEN a domain subsystem registers as a VFS provider, THE registration SHALL flow from the domain subsystem calling the VFS registry API — the VFS SHALL NOT actively discover or instantiate domain providers.

---

### Requirement 3: ff-dataset-catalog Ownership Boundary

**User Story:** As a platform architect, I want the Dataset Catalog to be the single authority for dataset metadata, catalog entries, naming validation, and resolution APIs, so that all subsystems obtain dataset information from one consistent source.

**Source:** ADR-001 — ff-dataset-catalog ownership definition. [ADR-001]

#### Acceptance Criteria

1. THE ff-dataset-catalog crate SHALL own: dataset definitions (create, read, update, delete metadata), catalog entries (the SQLite catalog database), dataset attributes (RECFM, LRECL, BLKSIZE, DSORG), dataset aliases, GDG catalog metadata (base definitions, generation tracking, roll-off policy), dataset resolution APIs (DSN to physical path), and dataset naming validation (DSN syntax, qualifier rules, HLQ management).
2. THE ff-dataset-catalog crate SHALL NOT own or implement: physical dataset allocation workflows driven by JCL (that belongs to ff-dataset-allocator), dataset content I/O beyond path resolution (content flows through VFS), VSAM record storage or retrieval logic, JCL parsing, or IDCAMS command processing.
3. THE ff-dataset-catalog crate SHALL NOT contain any `use ff_idcams` or `use ff_dataset_allocator` import statement — it SHALL NOT depend on higher-level orchestration crates.
4. THE ff-dataset-catalog crate SHALL expose a public API (trait or module) that provides: `create_dataset(dsn, attributes) → Result`, `delete_dataset(dsn) → Result`, `update_dataset(dsn, attributes) → Result`, `resolve_dsn(dsn) → Result<PhysicalPath>`, `list_datasets(filter) → Result<Vec<DatasetEntry>>`, `validate_dsn(dsn) → Result`, and GDG management operations (`create_gdg_base`, `create_generation`, `list_generations`, `resolve_generation`).
5. ALL modifications to dataset metadata (creation, deletion, attribute changes, GDG generation management) SHALL be performed exclusively through the ff-dataset-catalog public API — no other subsystem SHALL directly write to the catalog SQLite database.
6. THE ff-dataset-catalog crate SHALL provide the `LISTCAT` and `LISTDS` command implementations as these are catalog query operations that read catalog-owned metadata. The ff-idcams LISTCAT command SHALL delegate to ff-dataset-catalog's query API for data retrieval and own only the IDCAMS command syntax parsing and output formatting.

---

### Requirement 4: ff-dataset-allocator Ownership Boundary

**User Story:** As a platform architect, I want the Dataset Allocator to own only JCL-driven allocation workflows (DD parsing, DISP processing, symbolic substitution, referbacks), delegating all catalog operations to ff-dataset-catalog, so that allocation logic and catalog persistence remain cleanly separated.

**Source:** ADR-001 — ff-dataset-allocator ownership definition. [ADR-001]

#### Acceptance Criteria

1. THE ff-dataset-allocator crate SHALL own: DD statement interpretation (parsing JCL DD syntax), DISP parameter processing (NEW/OLD/SHR/MOD semantics), symbolic parameter substitution, referback resolution (`*.stepname.ddname`), GDG relative reference resolution (`(+1)`, `(0)`, `(-1)` interpretation), allocation workflow orchestration (sequencing the parse→substitute→resolve→validate pipeline), and the RESOLVE command.
2. THE ff-dataset-allocator crate SHALL NOT own or implement: catalog persistence (SQLite writes), dataset metadata storage, physical storage engine interaction, or dataset content I/O.
3. THE ff-dataset-allocator SHALL obtain all dataset metadata exclusively from ff-dataset-catalog services — it SHALL NOT query the catalog SQLite database directly or maintain its own dataset metadata cache that could become inconsistent with the catalog. WHEN catalog service methods are unavailable or fail, THE allocator SHALL propagate the error rather than bypassing to direct database access.
4. WHEN the allocator needs to create a dataset (DISP=NEW), THE allocator SHALL invoke `ff-dataset-catalog::create_dataset()` with the parsed attributes — it SHALL NOT create catalog entries or physical storage directly.
5. WHEN the allocator needs to verify dataset existence (DISP=OLD/SHR), THE allocator SHALL invoke `ff-dataset-catalog::resolve_dsn()` — it SHALL NOT query the filesystem or catalog database directly.
6. WHEN the allocator resolves a GDG relative reference, THE allocator SHALL invoke `ff-dataset-catalog::resolve_generation()` to obtain the target generation — it SHALL NOT compute generation numbers by reading the catalog database directly.
7. THE ff-dataset-allocator crate SHALL depend on ff-dataset-catalog through a trait interface (e.g., `CatalogService` trait), enabling unit testing with mock catalog implementations.

---

### Requirement 5: ff-vsam-services Ownership Boundary

**User Story:** As a platform architect, I want VSAM record-level operations (KSDS, ESDS, RRDS, LDS behaviour) isolated in a dedicated ff-vsam-services crate, so that VSAM implementation details are decoupled from catalog metadata and IDCAMS command parsing.

**Source:** ADR-001 — ff-vsam-services ownership definition. [ADR-001]

#### Acceptance Criteria

1. THE ff-vsam-services crate SHALL own: KSDS behaviour (key-sequenced insertion, key lookup, key-ordered traversal), ESDS behaviour (entry-sequenced insertion, sequential access), RRDS behaviour (relative record addressing), LDS behaviour (byte-oriented linear access), alternate index management (definition, maintenance, path access), record insertion logic, record retrieval logic, and key management (uniqueness enforcement, key comparison, index maintenance).
2. THE ff-vsam-services crate SHALL NOT own or implement: catalog metadata persistence (owned by ff-dataset-catalog), IDCAMS command parsing (owned by ff-idcams), JCL parsing (owned by ff-dataset-allocator), or storage provider registration (owned by ff-vfs).
3. THE ff-vsam-services crate SHALL NOT contain any `use ff_idcams` import statement — it SHALL NOT depend on the IDCAMS emulator.
4. ALL VSAM record-level operations (insert, retrieve, update, delete records within a VSAM dataset) SHALL be performed exclusively through ff-vsam-services APIs — no other subsystem SHALL directly manipulate VSAM storage structures.
5. THE ff-vsam-services crate SHALL expose a public trait (e.g., `VsamDataset`) with methods: `open(dsn, access_mode) → Result<VsamHandle>`, `get(key) → Result<Record>`, `put(record) → Result`, `delete(key) → Result`, `browse(start_key, direction) → Result<RecordIterator>`, and dataset-type-specific operations.
6. THE ff-vsam-services crate SHALL obtain dataset metadata (type, key length, key offset, record length) from ff-dataset-catalog via its resolution API — it SHALL NOT maintain its own metadata store.
7. THE ff-vsam-services crate SHALL register as a VFS provider (scheme `vsam`) for record-level access, enabling `vfs://vsam/DSN` URIs for VSAM-aware consumers.

---

### Requirement 6: ff-idcams Ownership Boundary

**User Story:** As a platform architect, I want the IDCAMS Emulator to own only command parsing and orchestration (DEFINE, DELETE, ALTER, LISTCAT, REPRO, IMPORT, EXPORT syntax), delegating all actual operations to the appropriate service crates, so that IDCAMS is a thin command interpreter rather than a monolithic implementation.

**Source:** ADR-001 — ff-idcams ownership definition. [ADR-001]

#### Acceptance Criteria

1. THE ff-idcams crate SHALL own: DEFINE command parsing (CLUSTER, AIX, PATH, GDG syntax), LISTCAT command parsing and output formatting, ALTER command parsing, DELETE command parsing, REPRO command parsing, IMPORT command parsing, EXPORT command parsing, BLDINDEX command parsing, PRINT command parsing, and VERIFY command parsing.
2. THE ff-idcams crate SHALL NOT own or implement: catalog persistence (no direct SQLite access), dataset metadata storage, dataset allocation logic, storage provider implementations, or VSAM record-level operations.
3. THE ff-idcams crate SHALL NOT directly write to, read from, or manage the catalog database — all catalog interactions SHALL flow through ff-dataset-catalog APIs.
4. WHEN ff-idcams parses a DEFINE CLUSTER command, THE ff-idcams crate SHALL extract the dataset attributes from the command syntax and invoke `ff-dataset-catalog::create_dataset()` to create the catalog entry, then invoke `ff-vsam-services::initialize_dataset()` to set up the VSAM storage structure. The parsing and execution SHALL be atomic — IF any downstream operation (catalog creation or VSAM initialization) fails, THE entire DEFINE command SHALL fail and any partial state SHALL be rolled back.
5. WHEN ff-idcams parses a DELETE command, THE ff-idcams crate SHALL invoke `ff-vsam-services::destroy_dataset()` to clean up VSAM structures, then invoke `ff-dataset-catalog::delete_dataset()` to remove the catalog entry.
6. WHEN ff-idcams parses a LISTCAT command, THE ff-idcams crate SHALL invoke `ff-dataset-catalog::list_datasets()` and `ff-dataset-catalog::get_dataset_attributes()` to retrieve catalog information, then format the output according to IDCAMS LISTCAT conventions.
7. WHEN ff-idcams parses an ALTER command, THE ff-idcams crate SHALL extract the modified attributes and invoke `ff-dataset-catalog::update_dataset()` to apply the changes.
8. WHEN ff-idcams parses a REPRO command, THE ff-idcams crate SHALL invoke `ff-vsam-services` for record-level copy operations between VSAM datasets, or `ff-vfs` for sequential dataset copies.
9. THE ff-idcams crate SHALL depend on ff-dataset-catalog, ff-dataset-allocator, and ff-vsam-services through trait interfaces, enabling unit testing with mock implementations of all downstream services.

---

### Requirement 7: Dependency Direction Enforcement

**User Story:** As a platform architect, I want dependency directions enforced at compile time, so that prohibited dependencies cannot be accidentally introduced and the layered architecture remains intact.

**Source:** ADR-001 — Dependency direction rules. [ADR-001]

#### Acceptance Criteria

1. THE permitted dependency direction SHALL be: `ff-idcams → ff-dataset-allocator → ff-dataset-catalog → ff-vsam-services → storage providers`. Each arrow indicates that the left crate MAY depend on (invoke APIs of) the right crate.
2. ALL components SHALL be permitted to depend on ff-vfs for resource access abstraction — this is the universal infrastructure dependency.
3. THE following dependencies SHALL be prohibited and SHALL NOT appear in any crate's `Cargo.toml` `[dependencies]` section: ff-vfs SHALL NOT depend on ff-idcams, ff-dataset-catalog, ff-dataset-allocator, or ff-vsam-services; ff-dataset-catalog SHALL NOT depend on ff-idcams or ff-dataset-allocator; ff-vsam-services SHALL NOT depend on ff-idcams or ff-dataset-allocator.
4. WHEN a prohibited dependency is introduced (detectable via `cargo tree` or CI dependency analysis), THE build pipeline SHALL flag it as an architectural violation.
5. THE ff-dataset-allocator crate SHALL NOT depend on ff-idcams — allocation is a lower-level service that IDCAMS orchestrates, not the reverse.
6. TRAIT-BASED indirection SHALL be used at dependency boundaries: ff-dataset-allocator SHALL depend on a `CatalogService` trait (defined in ff-dataset-catalog or a shared interface crate), not on ff-dataset-catalog's concrete implementation types directly. This enables testing and future refactoring without circular dependencies.

---

### Requirement 8: Conflict Resolution — IDCAMS Catalog Operations (IDC-005 through IDC-008)

**User Story:** As a platform architect, I want the IDCAMS spec's catalog creation/update/deletion requirements rewritten so that IDCAMS delegates to ff-dataset-catalog APIs, resolving the ownership conflict where IDCAMS claimed direct catalog CRUD authority.

**Source:** ADR-001 — Conflict resolution for IDCAMS catalog operations. [ADR-001]

#### Acceptance Criteria

1. THE IDCAMS specification requirements IDC-005 (Catalog Creation), IDC-007 (Catalog Deletion), and IDC-008 (Catalog Update) SHALL be reinterpreted as delegation requirements: IDCAMS SHALL invoke ff-dataset-catalog APIs to perform these operations rather than implementing catalog CRUD internally.
2. WHEN IDCAMS processes a DEFINE command that results in catalog entry creation (IDC-005), THE ff-idcams crate SHALL invoke `ff-dataset-catalog::create_dataset()` passing the parsed attributes — the catalog entry creation is owned by ff-dataset-catalog.
3. WHEN IDCAMS processes a DELETE command that results in catalog entry removal (IDC-007), THE ff-idcams crate SHALL invoke `ff-dataset-catalog::delete_dataset()` — the catalog entry deletion is owned by ff-dataset-catalog.
4. WHEN IDCAMS processes an ALTER command that results in catalog attribute changes (IDC-008), THE ff-idcams crate SHALL invoke `ff-dataset-catalog::update_dataset()` — metadata modification is owned by ff-dataset-catalog.
5. THE IDCAMS specification requirement IDC-006 (LISTCAT display) SHALL be split: ff-idcams owns the LISTCAT command syntax parsing and output formatting; ff-dataset-catalog owns the data retrieval (query) API that provides the catalog information. THE ff-idcams LISTCAT implementation SHALL invoke `ff-dataset-catalog::list_datasets()` with the parsed filter criteria.
6. THE IDCAMS emulator SHALL NOT contain any SQLite database access code, catalog schema definitions, or direct file-system metadata manipulation — all such operations are delegated.

---

### Requirement 9: Conflict Resolution — IDCAMS VSAM Dataset Types (IDC-001 through IDC-004)

**User Story:** As a platform architect, I want the IDCAMS spec's VSAM dataset type requirements rewritten so that IDCAMS delegates actual VSAM behaviour to ff-vsam-services, resolving the ownership conflict where IDCAMS claimed VSAM implementation authority.

**Source:** ADR-001 — Conflict resolution for IDCAMS VSAM operations. [ADR-001]

#### Acceptance Criteria

1. THE IDCAMS specification requirements IDC-001 (KSDS Support), IDC-002 (ESDS Support), IDC-003 (RRDS Support), and IDC-004 (LDS Support) SHALL be reinterpreted as orchestration requirements: IDCAMS SHALL parse DEFINE commands specifying these types and delegate actual VSAM dataset creation and behaviour to ff-vsam-services.
2. WHEN IDCAMS parses a DEFINE CLUSTER with INDEXED organization (KSDS), THE ff-idcams crate SHALL invoke `ff-vsam-services::create_ksds()` with the parsed key length, key offset, and record attributes — the KSDS behaviour implementation is owned by ff-vsam-services.
3. WHEN IDCAMS parses a DEFINE CLUSTER with NONINDEXED organization (ESDS), THE ff-idcams crate SHALL invoke `ff-vsam-services::create_esds()` with the parsed record attributes.
4. WHEN IDCAMS parses a DEFINE CLUSTER with NUMBERED organization (RRDS), THE ff-idcams crate SHALL invoke `ff-vsam-services::create_rrds()` with the parsed slot size and capacity.
5. WHEN IDCAMS parses a DEFINE CLUSTER for a Linear Data Set (LDS), THE ff-idcams crate SHALL invoke `ff-vsam-services::create_lds()` with the parsed allocation attributes.
6. THE ff-idcams crate SHALL NOT contain any VSAM record-level logic (key comparison, index maintenance, record insertion, sequential access implementation) — these are exclusively owned by ff-vsam-services. THE ff-idcams crate SHALL NOT invoke VSAM creation functions except through proper delegation via the `VsamService` trait interface.
7. THE ff-idcams crate MAY validate DEFINE command parameters (e.g., key length > 0, record length within bounds) as a syntax-level check, but the authoritative validation of VSAM structural constraints SHALL be performed by ff-vsam-services during dataset creation.

---

### Requirement 10: Conflict Resolution — Dataset Catalog Allocation (Requirement 7)

**User Story:** As a platform architect, I want the Dataset Catalog's allocation requirement clarified so that the catalog owns only the low-level metadata create API, while the JCL-driven allocation workflow (parsing DD statements, interpreting DISP, applying defaults) is owned by ff-dataset-allocator.

**Source:** ADR-001 — Conflict resolution for catalog vs allocator responsibilities. [ADR-001]

#### Acceptance Criteria

1. THE ff-dataset-catalog Requirement 7 (Dataset Create, Delete, Rename, and Allocate) SHALL be interpreted as defining the low-level catalog CRUD API — the methods that create/delete/rename catalog entries and their associated physical storage. This is the "primitive" layer.
2. THE ff-dataset-allocator owns the high-level allocation workflow: parsing JCL DD statements, extracting DCB/SPACE parameters, interpreting DISP semantics, applying defaults, and orchestrating the allocation by invoking the catalog's create API.
3. WHEN the ff-dataset-catalog's `create_dataset()` API is invoked (by ff-dataset-allocator or ff-idcams), THE catalog SHALL create the metadata entry and physical storage — this is the catalog's owned responsibility.
4. THE ff-dataset-allocator SHALL NOT duplicate the catalog's create logic — it SHALL NOT directly create physical files or write to the catalog database. It prepares the parameters and delegates.
5. THE ff-dataset-catalog SHALL NOT parse JCL DD statements, interpret DISP parameters, or perform symbolic substitution — these are exclusively owned by ff-dataset-allocator.
6. THE boundary is: ff-dataset-allocator decides WHAT to allocate and WHEN (from JCL context); ff-dataset-catalog decides HOW to allocate (creating the entry and storage).

---

### Requirement 11: Conflict Resolution — LISTCAT Command Ownership (Catalog Req 13 vs IDCAMS IDC-006)

**User Story:** As a platform architect, I want the LISTCAT/LISTDS command ownership clarified between the Dataset Catalog and IDCAMS, resolving the overlap where both specs claim LISTCAT functionality.

**Source:** ADR-001 — Conflict resolution for LISTCAT overlap. [ADR-001]

#### Acceptance Criteria

1. THE ff-dataset-catalog SHALL own the `catalog.listcat` and `catalog.listds` commands (as defined in its Requirement 13) — these are workbench-native commands that query catalog metadata directly using the catalog's internal API. They are NOT IDCAMS commands.
2. THE ff-idcams crate SHALL own the IDCAMS `LISTCAT` command — this is the IDCAMS-syntax command (`LISTCAT ENTRIES(dsn) ALL/NAME/VOLUME`) that emulates the z/OS IDCAMS LISTCAT utility. It is registered under a separate command ID (e.g., `idcams.listcat`).
3. THE ff-idcams LISTCAT command SHALL obtain its data by invoking ff-dataset-catalog's query APIs (`list_datasets`, `get_dataset_attributes`) — it SHALL NOT query the catalog database directly.
4. THE two commands serve different personas: `catalog.listcat` is a workbench-native developer tool with tabular output; `idcams.listcat` is a mainframe-faithful emulation with IDCAMS-formatted output (cluster/data/index hierarchy).
5. WHEN both commands query the same underlying catalog data, they SHALL produce consistent results because both read from the same ff-dataset-catalog API — there SHALL be no divergence due to separate data sources.
6. THE ff-dataset-catalog's LISTCAT/LISTDS commands SHALL NOT depend on ff-idcams — they are independent catalog-native query tools that predate and do not require the IDCAMS emulator.

---

### Requirement 12: Conflict Resolution — Dataset Allocator Catalog References

**User Story:** As a platform architect, I want the Dataset Allocator's catalog operation references rewritten to flow through ff-dataset-catalog APIs exclusively, resolving the conflict where the allocator spec implied direct catalog access.

**Source:** ADR-001 — Conflict resolution for allocator catalog access. [ADR-001]

#### Acceptance Criteria

1. THE ff-dataset-allocator Requirement 2 (Dataset Name Resolution Against Mounted Catalogs) acceptance criterion 8 ("ALL DSN resolution SHALL flow through the ff-dataset-catalog crate's resolution API") SHALL be the authoritative integration pattern — the allocator SHALL NOT have any code path that bypasses this API.
2. THE ff-dataset-allocator SHALL depend on ff-dataset-catalog through a `CatalogService` trait that abstracts the catalog's public API, with methods: `resolve_dsn(dsn) → Result<ResolutionResult>`, `create_dataset(dsn, attrs) → Result`, `dataset_exists(dsn) → Result<bool>`, `resolve_gdg_generation(base, offset) → Result<GenerationInfo>`, and `get_dataset_attributes(dsn) → Result<DatasetAttributes>`.
3. THE ff-dataset-allocator SHALL NOT contain `use rusqlite` or any SQLite-related imports — all database access is encapsulated within ff-dataset-catalog.
4. THE ff-dataset-allocator SHALL NOT read any TOML configuration directly — all configuration (including allocation defaults, lint levels, and resolution preferences) SHALL flow through the `CatalogService` trait API. THE allocator SHALL NOT define its own independent defaults that could diverge from the catalog's defaults.
5. WHEN the allocator's Requirement 14 (Configuration and Defaults) references `[catalog.defaults]`, THE allocator SHALL read these values by invoking ff-dataset-catalog's defaults API. THE allocator SHALL NOT contain direct TOML parsing, file reading, or `ff-config` access for any configuration values — all configuration access is mediated through the catalog service.

---

### Requirement 13: Dataset Lifecycle — Ownership at Each Stage

**User Story:** As a platform architect, I want the complete dataset lifecycle documented with clear ownership at each stage, so that implementers know exactly which crate is responsible for each operation in the create→use→modify→delete sequence.

**Source:** ADR-001 — Lifecycle ownership mapping. [ADR-001]

#### Acceptance Criteria

1. THE dataset creation lifecycle SHALL follow this ownership sequence: (a) ff-idcams or ff-dataset-allocator parses the creation request (DEFINE command or DD with DISP=NEW); (b) the requesting crate invokes `ff-dataset-catalog::create_dataset()` with validated attributes; (c) ff-dataset-catalog creates the catalog entry and physical storage; (d) IF the dataset is VSAM, ff-dataset-catalog invokes `ff-vsam-services::initialize_dataset()` to set up record structures.
2. THE dataset access lifecycle SHALL follow this strict sequential ownership sequence: (a) a consumer provides a DSN; (b) ff-dataset-catalog resolves the DSN to a physical path via `resolve_dsn()` — this step depends on step (a); (c) content I/O flows through ff-vfs using `vfs://catalog/DSN` or `vfs://vsam/DSN` URIs — this step depends on step (b); (d) for VSAM datasets, record-level access flows through ff-vsam-services — this step depends on step (b). Each step SHALL complete before its dependent steps begin.
3. THE dataset modification lifecycle (attribute changes) SHALL follow in strict sequential order: (a) ff-idcams parses an ALTER command or the workbench UI initiates a change — parsing MUST complete before invocation; (b) the initiator invokes `ff-dataset-catalog::update_dataset()` with the new attributes — invocation MUST complete before catalog update; (c) ff-dataset-catalog updates the catalog entry.
4. THE dataset deletion lifecycle SHALL follow: (a) ff-idcams parses a DELETE command or the workbench UI initiates deletion; (b) IF the dataset is VSAM, the initiator invokes `ff-vsam-services::destroy_dataset()` to clean up record structures; (c) the initiator invokes `ff-dataset-catalog::delete_dataset()` to remove the catalog entry and physical storage.
5. THE GDG generation lifecycle SHALL follow: (a) ff-dataset-allocator resolves a `(+1)` reference or ff-idcams processes a DEFINE for a new generation; (b) the requesting crate SHALL always invoke `ff-dataset-catalog::create_generation()` which handles generation numbering, roll-off policy enforcement, and catalog entry creation — this invocation is mandatory after resolving the reference, not optional.
6. AT NO POINT in any lifecycle SHALL a subsystem bypass the owning subsystem's API to perform an operation directly — this is the fundamental architectural invariant.

---

### Requirement 14: Resolution Lifecycle — Ownership at Each Stage

**User Story:** As a platform architect, I want the DSN resolution lifecycle documented with clear ownership at each stage, so that the path from a JCL DSN reference to a physical file is unambiguous.

**Source:** ADR-001 — Resolution lifecycle ownership. [ADR-001]

#### Acceptance Criteria

1. THE resolution lifecycle SHALL follow this ownership sequence: (a) ff-dataset-allocator parses the JCL DD statement (owned: DD parsing); (b) ff-dataset-allocator performs symbolic substitution on the DSN (owned: symbolic substitution); (c) ff-dataset-allocator resolves referbacks if present (owned: referback resolution); (d) ff-dataset-allocator invokes `ff-dataset-catalog::resolve_dsn()` with the fully-substituted DSN (delegation to catalog); (e) ff-dataset-catalog performs the catalog lookup and returns the physical path (owned: resolution).
2. WHEN the resolution involves a GDG relative reference: (a) ff-dataset-allocator detects the `(+n)`/`(0)`/`(-n)` syntax (owned: GDG reference detection); (b) ff-dataset-allocator invokes `ff-dataset-catalog::resolve_generation()` with the base name and offset (delegation to catalog); (c) ff-dataset-catalog computes the target generation and returns its path (owned: generation resolution).
3. WHEN resolution results in a new allocation (DISP=NEW): (a) ff-dataset-allocator determines that allocation is required (owned: DISP interpretation); (b) ff-dataset-allocator assembles attributes from DCB/SPACE/defaults (owned: attribute assembly); (c) ff-dataset-allocator invokes `ff-dataset-catalog::create_dataset()` (delegation to catalog); (d) ff-dataset-catalog creates the entry and storage (owned: catalog CRUD).
4. THE resolution lifecycle SHALL NOT involve ff-idcams — IDCAMS is a separate entry point for DEFINE/DELETE/ALTER commands, not part of the JCL resolution pipeline.
5. THE resolution lifecycle SHALL NOT involve ff-vsam-services for path resolution — VSAM services are invoked only for record-level operations after the dataset has been located.

---

### Requirement 15: Interface Contract — ff-dataset-catalog Public API

**User Story:** As a subsystem implementer, I want the ff-dataset-catalog's public API contract formally defined, so that dependent crates (ff-dataset-allocator, ff-idcams, ff-vsam-services) can code to a stable interface.

**Source:** ADR-001 — Interface contract definitions. [ADR-001]

#### Acceptance Criteria

1. THE ff-dataset-catalog crate SHALL expose a `CatalogService` trait (or equivalent public API module) that defines the complete set of operations available to external consumers.
2. THE `CatalogService` trait SHALL include dataset CRUD operations: `create_dataset(dsn: &str, attrs: DatasetAttributes) → Result<DatasetId, CatalogError>`, `delete_dataset(dsn: &str) → Result<(), CatalogError>`, `update_dataset(dsn: &str, attrs: DatasetAttributes) → Result<(), CatalogError>`, `rename_dataset(old_dsn: &str, new_dsn: &str) → Result<(), CatalogError>`.
3. THE `CatalogService` trait SHALL include resolution operations: `resolve_dsn(dsn: &str) → Result<ResolutionResult, CatalogError>`, `dataset_exists(dsn: &str) → Result<bool, CatalogError>`, `get_dataset_attributes(dsn: &str) → Result<DatasetAttributes, CatalogError>`.
4. THE `CatalogService` trait SHALL include query operations: `list_datasets(filter: &DatasetFilter) → Result<Vec<DatasetEntry>, CatalogError>`, `validate_dsn(dsn: &str) → Result<(), DsnValidationError>`.
5. THE `CatalogService` trait SHALL include GDG operations: `create_gdg_base(dsn: &str, limit: u8, scratch: bool) → Result<(), CatalogError>`, `create_generation(base_dsn: &str, attrs: DatasetAttributes) → Result<GenerationInfo, CatalogError>`, `resolve_generation(base_dsn: &str, offset: i32) → Result<GenerationInfo, CatalogError>`, `list_generations(base_dsn: &str) → Result<Vec<GenerationInfo>, CatalogError>`.
6. THE `CatalogService` trait SHALL include defaults retrieval: `get_allocation_defaults(dsorg: Dsorg) → DatasetAttributes` — providing the configured default RECFM, LRECL, BLKSIZE for a given dataset organization type.
7. THE `CatalogService` trait SHALL prioritize ergonomic type signatures using generics and associated types. A separate object-safe wrapper trait (e.g., `DynCatalogService`) SHALL be provided for dynamic dispatch and mock implementations in tests — enabling both ergonomic production code and flexible testing.

---

### Requirement 16: Interface Contract — ff-vsam-services Public API

**User Story:** As a subsystem implementer, I want the ff-vsam-services public API contract formally defined, so that ff-idcams and other consumers can code to a stable interface even before the VSAM crate is fully implemented.

**Source:** ADR-001 — Interface contract definitions. [ADR-001]

#### Acceptance Criteria

1. THE ff-vsam-services crate SHALL expose a `VsamService` trait (or equivalent public API module) that defines the complete set of VSAM operations available to external consumers.
2. THE `VsamService` trait SHALL include dataset lifecycle operations: `create_ksds(dsn: &str, key_length: u16, key_offset: u16, record_length: u32) → Result<(), VsamError>`, `create_esds(dsn: &str, record_length: u32) → Result<(), VsamError>`, `create_rrds(dsn: &str, slot_size: u32) → Result<(), VsamError>`, `create_lds(dsn: &str) → Result<(), VsamError>`, `destroy_dataset(dsn: &str) → Result<(), VsamError>`, `initialize_dataset(dsn: &str, vsam_type: VsamType, params: VsamParams) → Result<(), VsamError>`.
3. THE `VsamService` trait SHALL include record-level operations: `open(dsn: &str, mode: AccessMode) → Result<VsamHandle, VsamError>`, `get(handle: &VsamHandle, key: &[u8]) → Result<Record, VsamError>`, `put(handle: &VsamHandle, record: &Record) → Result<(), VsamError>`, `delete(handle: &VsamHandle, key: &[u8]) → Result<(), VsamError>`, `close(handle: VsamHandle) → Result<(), VsamError>`.
4. THE `VsamService` trait SHALL include browsing operations: `start_browse(handle: &VsamHandle, start_key: &[u8], direction: BrowseDirection) → Result<BrowseHandle, VsamError>`, `next_record(browse: &BrowseHandle) → Result<Option<Record>, VsamError>`, `end_browse(browse: BrowseHandle) → Result<(), VsamError>`.
5. THE `VsamService` trait SHALL include alternate index operations: `define_aix(base_dsn: &str, aix_dsn: &str, key_field: KeyField) → Result<(), VsamError>`, `build_index(aix_dsn: &str) → Result<(), VsamError>`.
6. THE `VsamService` trait SHALL be object-safe to enable dynamic dispatch and mock implementations.
7. UNTIL ff-vsam-services is implemented, dependent crates (ff-idcams) SHALL compile against the trait definition with a no-op or error-returning stub implementation, enabling the architecture to be validated without the full VSAM implementation.

---

### Requirement 17: Interface Contract — ff-dataset-allocator Public API

**User Story:** As a subsystem implementer, I want the ff-dataset-allocator's public API contract formally defined, so that ff-idcams and other consumers can invoke allocation workflows through a stable interface.

**Source:** ADR-001 — Interface contract definitions. [ADR-001]

#### Acceptance Criteria

1. THE ff-dataset-allocator crate SHALL expose an `AllocatorService` trait (or equivalent public API module) that defines the allocation workflow operations available to external consumers.
2. THE `AllocatorService` trait SHALL include: `resolve_dd(dd_statement: &DdStatement, context: &JobContext) → Result<ResolutionResult, AllocatorError>` — resolve a single DD statement within a job context.
3. THE `AllocatorService` trait SHALL include: `resolve_job(jcl_text: &str, mode: ResolveMode) → Result<JobResolutionResult, AllocatorError>` — resolve all DD statements in a complete JCL job.
4. THE `AllocatorService` trait SHALL include: `resolve_dsn(dsn: &str) → Result<ResolutionResult, AllocatorError>` — resolve a standalone DSN against mounted catalogs (convenience method bypassing JCL parsing).
5. THE `AllocatorService` trait SHALL include: `substitute_symbols(text: &str, symbol_table: &SymbolTable) → Result<String, AllocatorError>` — perform symbolic substitution on arbitrary text (utility method for other consumers).
6. THE ff-idcams crate MAY invoke the allocator's `resolve_dsn()` for REPRO/IMPORT/EXPORT commands that need to locate datasets, rather than reimplementing resolution logic.
7. THE `AllocatorService` trait SHALL be object-safe to enable dynamic dispatch and mock implementations.

---

### Requirement 18: Architectural Compliance Verification

**User Story:** As a platform architect, I want automated verification that the ownership model is being followed, so that architectural violations are caught during development rather than discovered in production.

**Source:** ADR-001 — Governance enforcement. [ADR-001]

#### Acceptance Criteria

1. THE project CI pipeline SHALL include a dependency direction check that parses all workspace crate `Cargo.toml` files and verifies that no prohibited dependencies exist (as defined in Requirement 7).
2. THE project SHALL maintain an architectural fitness function (test or script) that verifies: (a) ff-vfs has zero dependencies on domain crates; (b) ff-dataset-catalog has zero dependencies on ff-idcams or ff-dataset-allocator; (c) ff-vsam-services has zero dependencies on ff-idcams or ff-dataset-allocator.
3. WHEN a new crate is added to the workspace that participates in the dataset subsystem, THE architectural fitness function SHALL be updated to include the new crate's permitted and prohibited dependencies. THE workspace build SHALL prevent addition of a new dataset subsystem crate until the fitness function is updated with appropriate dependency rules — omitting this update SHALL cause CI failure.
4. THE project SHALL include integration tests that verify trait-based coupling: each dependent crate SHALL compile and pass basic tests with a mock implementation of its upstream trait (e.g., ff-dataset-allocator compiles with a mock `CatalogService`), proving that no concrete-type coupling exists.
5. THE architectural fitness function SHALL be executable via `cargo test --test architecture_compliance` and SHALL return a non-zero exit code when violations are detected, causing the CI build to fail immediately.

---

### Requirement 19: Subsystem Specification Alignment

**User Story:** As a platform architect, I want clear guidance on how existing subsystem specifications must be updated to align with this governance document, so that the transition from the current (conflicting) state to the clean ownership model is orderly.

**Source:** ADR-001 — Specification migration guidance. [ADR-001]

#### Acceptance Criteria

1. THE ff-idcams specification (IDCAMS-Emulator-Requirements.md) SHALL be updated to: (a) rewrite IDC-001 through IDC-004 as delegation requirements invoking ff-vsam-services; (b) rewrite IDC-005, IDC-007, IDC-008 as delegation requirements invoking ff-dataset-catalog; (c) rewrite IDC-006 to invoke ff-dataset-catalog query APIs for data and own only output formatting.
2. THE ff-dataset-catalog specification (dataset-catalog/requirements.md) SHALL be updated to: (a) clarify Requirement 7 as the low-level catalog CRUD API (not the JCL allocation workflow); (b) clarify Requirement 13 LISTCAT/LISTDS as workbench-native commands distinct from IDCAMS LISTCAT; (c) add a note that JCL-driven allocation workflows are owned by ff-dataset-allocator.
3. THE ff-dataset-allocator specification (dataset-allocator/requirements.md) SHALL be updated to: (a) ensure all catalog access references explicitly name the `CatalogService` trait interface; (b) remove any implication of direct catalog database access; (c) confirm that allocation defaults are sourced from ff-dataset-catalog configuration.
4. THE ff-vfs specification (virtual-file-system/requirements.md) requires NO changes — it correctly owns only the abstraction layer and does not reach into domain logic. This is confirmed by ADR-001 review.
5. EACH updated specification SHALL include a cross-reference to this governance document (dataset-ownership-model) as the authoritative source for ownership boundaries.
6. SPECIFICATION updates SHALL be performed incrementally — each subsystem spec can be updated independently as long as it conforms to this governance document's requirements.

---

### Requirement 20: Future Extensibility — New Subsystem Integration

**User Story:** As a platform architect, I want a clear pattern for integrating future subsystems (e.g., ff-vsam-services, ff-jcl-runner) into the ownership model, so that the architecture scales without ad-hoc decisions.

**Source:** ADR-001 — Extensibility principle. [ADR-001]

#### Acceptance Criteria

1. WHEN a new subsystem is proposed that touches dataset operations, THE proposer SHALL produce an ADR amendment that: (a) defines what the new subsystem owns; (b) defines what it does not own; (c) defines its permitted and prohibited dependencies; (d) defines its authority rule.
2. THE new subsystem SHALL integrate with existing subsystems exclusively through their defined trait interfaces (CatalogService, VsamService, AllocatorService, VfsProvider) — it SHALL NOT introduce new direct coupling between existing crates.
3. THE new subsystem's dependency direction SHALL be appended to the dependency chain without creating cycles — the DAG property of the dependency graph SHALL be preserved.
4. THE architectural fitness function (Requirement 18) SHALL be extended to cover the new subsystem's constraints within the same PR that introduces the new crate.
5. IF the new subsystem requires capabilities not currently exposed by an existing trait interface, THE required trait methods SHALL be added to the owning subsystem's public API through a PR to that subsystem — the new subsystem SHALL NOT work around missing APIs by accessing internals directly.
