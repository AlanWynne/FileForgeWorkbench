# FileForgeWorkbench Virtual File and Dataset Storage Architecture Requirements

## Document Control

| Field | Value |
|---|---|
| Document title | Virtual File and Dataset Storage Architecture Requirements |
| Project | FileForgeWorkbench |
| Document type | Software architecture and requirements specification |
| Status | Proposed |
| Version | 0.1 |
| Date | 2026-09-03 |
| Language | British/South African English |

## 1. Purpose

This document defines the proposed architecture and requirements for representing mainframe datasets, indexed files, partitioned libraries, generation data groups, and POSIX files in FileForgeWorkbench.

The design adopts a **hybrid storage architecture**:

- SQLite shall provide the logical catalogue, metadata repository, relationships, indexes, audit information, and selected record-oriented storage services.
- The host filesystem shall store ordinary sequential datasets, partitioned dataset members, generation files, and POSIX files as native files and directories.
- Storage providers shall be accessed through common abstractions so that catalogue resolution is separated from physical storage.

The objective is to preserve useful mainframe semantics without turning SQLite into a general-purpose binary file container or hiding ordinary files from operating-system tools, editors, Git, backup utilities, and future storage providers.

## 2. Scope

This specification covers:

- Physical sequential datasets (PS).
- Partitioned datasets (PDS).
- Partitioned datasets extended (PDSE).
- Generation data groups (GDGs).
- VSAM key-sequenced datasets (KSDS).
- VSAM entry-sequenced datasets (ESDS).
- VSAM relative-record datasets (RRDS).
- ISAM-style indexed files.
- POSIX files and directories.
- Dataset catalogue services.
- Record-format handling.
- Storage abstraction, portability, integrity, migration, and traceability.

This specification does not require byte-for-byte replication of z/OS internals. FileForgeWorkbench shall emulate externally useful behaviours through documented, testable abstractions.

## 3. Architectural Decision

### 3.1 Decision

FileForgeWorkbench shall use a hybrid model in which:

1. **SQLite is the authoritative logical catalogue and metadata repository.**
2. **Native files and directories are the default physical storage for stream-oriented and library-oriented content.**
3. **SQLite-backed data stores may be used for record-oriented datasets whose defining behaviour requires keyed or relative access.**
4. **POSIX content remains native filesystem content and is not stored as SQLite BLOBs by default.**
5. **All access is mediated by catalogue and storage-provider interfaces rather than by user-interface code accessing SQLite or disk paths directly.**

### 3.2 Decision Matrix

| Logical type | Preferred physical representation | Catalogue and metadata |
|---|---|---|
| PS | Native file | SQLite |
| PDS | Native directory with member files | SQLite |
| PDSE | Native directory with member files | SQLite |
| GDG base | Logical catalogue entity | SQLite |
| GDG generation | Native file or provider-specific object | SQLite |
| VSAM KSDS | SQLite-backed keyed record store | SQLite |
| VSAM RRDS | SQLite-backed relative-record store | SQLite |
| VSAM ESDS | Native append-oriented data file, with optional sidecar index | SQLite |
| ISAM | SQLite-backed indexed record store by default | SQLite |
| POSIX file | Native file | Optional SQLite registration/pointer |
| POSIX directory | Native directory | Optional SQLite registration/pointer |

Implementations may offer alternative providers, but the logical behaviour and catalogue contract shall remain stable.

## 4. Architectural Principles

### AR-PR-001: Separation of concerns

The logical dataset model, catalogue, record codecs, physical storage, and user interface shall be separate components.

### AR-PR-002: Catalogue independence

A dataset's logical name shall not be treated as its physical path. The catalogue shall resolve a logical identifier to a storage provider and provider-specific locator.

### AR-PR-003: Native-file preference

Content that does not require database semantics shall be stored as native files or directories unless a configured storage provider explicitly requires another representation.

### AR-PR-004: Behaviour over internal replication

The product shall emulate documented dataset behaviours such as keyed retrieval, member access, generation resolution, and record formatting. It need not reproduce proprietary on-disk control structures.

### AR-PR-005: Cross-platform portability

The logical naming model shall be independent of Windows, Linux, and macOS path syntax. Provider path handling shall use platform-safe APIs.

### AR-PR-006: Recoverability

Catalogue entries and physical objects shall be recoverable, reconcilable, and diagnosable when one side is missing or inconsistent.

### AR-PR-007: Explicit transactions

Operations that affect both catalogue state and physical content shall use staged, transactional workflows with compensating recovery where a single atomic transaction cannot span SQLite and the filesystem.

## 5. Conceptual Architecture

```text
+---------------------------------------------------------+
|                   FileForgeWorkbench                    |
+---------------------------------------------------------+
| Editors | Dataset Explorer | JES/CICS Tools | Importers |
+---------------------------------------------------------+
|          Virtual File and Dataset Service API           |
+---------------------------+-----------------------------+
| Catalogue Service         | Record and Member Services  |
+---------------------------+-----------------------------+
| Storage Provider Interface                              |
+-------------+----------------+--------------------------+
| Native File | SQLite Record  | Future Providers         |
| Provider    | Provider       | Object/Remote/Archive    |
+-------------+----------------+--------------------------+
| SQLite Catalogue           | Host Filesystem            |
+----------------------------+----------------------------+
```

## 6. Logical Dataset Model

Every managed object shall have a stable internal identifier independent of its logical name and physical locator.

A catalogue entry should include, where applicable:

- Internal dataset identifier.
- Logical dataset name.
- Dataset organisation/type.
- Storage provider identifier.
- Provider-specific locator.
- Record format (`RECFM`).
- Logical record length (`LRECL`).
- Block size (`BLKSIZE`).
- Key definition and alternate-index definitions.
- Character encoding or code page.
- Binary/text handling mode.
- Created and modified timestamps.
- Owner and security metadata.
- Tags and description.
- Retention and lifecycle state.
- Checksum or integrity information.
- Parent library or GDG base identifier.
- Source, lineage, and migration information.
- Optimistic concurrency/version token.

Physical paths shall not be exposed as the permanent identity of datasets.

## 7. Functional Requirements

### 7.1 Catalogue Requirements

**FFW-VFS-CAT-001**  
The system shall maintain dataset catalogue entries in SQLite.

**FFW-VFS-CAT-002**  
The catalogue shall map each managed logical dataset name to exactly one active storage provider and provider-specific locator within a catalogue scope.

**FFW-VFS-CAT-003**  
The catalogue shall support master and user-catalogue concepts, or an equivalent scoped catalogue hierarchy.

**FFW-VFS-CAT-004**  
The catalogue shall maintain dataset organisation, record attributes, encoding, ownership, timestamps, lifecycle state, and descriptive metadata.

**FFW-VFS-CAT-005**  
The catalogue shall validate uniqueness according to the configured naming scope and collation rules.

**FFW-VFS-CAT-006**  
The catalogue shall support logical rename and physical relocation as separate operations.

**FFW-VFS-CAT-007**  
The catalogue shall detect entries whose physical objects are missing, inaccessible, duplicated, or inconsistent.

**FFW-VFS-CAT-008**  
The system shall provide a reconciliation operation that can compare catalogue state with provider state and report proposed corrective actions without automatically changing data.

**FFW-VFS-CAT-009**  
The catalogue shall record create, rename, move, delete, restore, import, export, and allocation changes in an audit trail.

**FFW-VFS-CAT-010**  
Schema changes shall be versioned and performed through forward migration scripts.

### 7.2 Physical Sequential Dataset Requirements

**FFW-VFS-PS-001**  
A physical sequential dataset shall be stored as a native file by the default provider.

**FFW-VFS-PS-002**  
The catalogue shall preserve `RECFM`, `LRECL`, `BLKSIZE`, encoding, and binary/text mode independently of the host filesystem.

**FFW-VFS-PS-003**  
The record codec shall support at least fixed-length, variable-length, and undefined or binary record modes.

**FFW-VFS-PS-004**  
The system shall distinguish logical record boundaries from host text line endings.

**FFW-VFS-PS-005**  
Import and export shall require an explicit record-format and encoding policy when the format cannot be reliably determined.

### 7.3 PDS and PDSE Requirements

**FFW-VFS-PDS-001**  
A PDS or PDSE shall be represented by a native directory under the default provider.

**FFW-VFS-PDS-002**  
Each library member shall be represented by an individually addressable native file unless a configured alternative provider is used.

**FFW-VFS-PDS-003**  
Member metadata shall be maintained in the catalogue and shall not depend solely on host filename attributes.

**FFW-VFS-PDS-004**  
The system shall validate member names according to the active mainframe naming profile while retaining a safe physical filename mapping.

**FFW-VFS-PDS-005**  
The logical member name shall not be assumed to equal the host filename. Reversible escaping or an explicit mapping shall handle case, illegal characters, reserved names, and collisions.

**FFW-VFS-PDS-006**  
PDS and PDSE differences shall be represented as logical capabilities and metadata, not by requiring unrelated physical storage structures.

**FFW-VFS-PDS-007**  
Member-level create, read, update, rename, delete, copy, move, compare, and version-control operations shall be supported.

### 7.4 GDG Requirements

**FFW-VFS-GDG-001**  
A GDG base shall be a logical catalogue entity containing its lifecycle and retention rules.

**FFW-VFS-GDG-002**  
Each GDG generation shall have a stable internal identifier, absolute generation/version identity, physical locator, creation timestamp, and lifecycle state.

**FFW-VFS-GDG-003**  
The system shall resolve relative generation references such as `0`, `-1`, and `+1` against a consistent catalogue snapshot.

**FFW-VFS-GDG-004**  
Allocation of a new generation shall prevent duplicate generation numbers during concurrent operations.

**FFW-VFS-GDG-005**  
Generation limit processing shall be configurable and shall not irreversibly delete a rolled-off generation without applying the configured retention policy.

**FFW-VFS-GDG-006**  
The catalogue shall preserve historical lineage between each generation and its GDG base.

**FFW-VFS-GDG-007**  
Rename or relocation of a GDG base shall not break relative-generation resolution.

### 7.5 VSAM KSDS Requirements

**FFW-VFS-KSDS-001**  
The default KSDS provider shall use a SQLite-backed keyed record store rather than SQLite BLOB storage in the general catalogue table.

**FFW-VFS-KSDS-002**  
Each KSDS shall define a primary key offset, key length, key type/collation, and uniqueness rule.

**FFW-VFS-KSDS-003**  
The provider shall support keyed read, ordered sequential read, insert, update, delete, and range retrieval.

**FFW-VFS-KSDS-004**  
Primary-key uniqueness shall be enforced transactionally.

**FFW-VFS-KSDS-005**  
Alternate indexes shall be represented as explicit metadata and database indexes or mapping tables.

**FFW-VFS-KSDS-006**  
Record data shall be stored independently of catalogue rows so catalogue queries do not scan dataset payloads.

**FFW-VFS-KSDS-007**  
The design shall permit a KSDS to use a dedicated SQLite database or another provider when isolation, scale, backup, or contention requirements justify it.

### 7.6 VSAM RRDS Requirements

**FFW-VFS-RRDS-001**  
The default RRDS provider shall use a SQLite-backed record store keyed by relative record number.

**FFW-VFS-RRDS-002**  
The provider shall distinguish an unallocated relative record from an allocated record containing zero or blank content.

**FFW-VFS-RRDS-003**  
The provider shall support direct retrieval, replacement, deletion, and sequential iteration by relative record number.

### 7.7 VSAM ESDS Requirements

**FFW-VFS-ESDS-001**  
The default ESDS provider shall store records in insertion order in an append-oriented native file.

**FFW-VFS-ESDS-002**  
The provider shall issue a stable record address or equivalent logical token for each appended record.

**FFW-VFS-ESDS-003**  
If an index or sidecar file is used, it shall be rebuildable from the data file or protected by an integrity and recovery mechanism.

**FFW-VFS-ESDS-004**  
Update and deletion semantics shall be explicitly documented if they differ from append-only behaviour.

### 7.8 ISAM Requirements

**FFW-VFS-ISAM-001**  
ISAM-style files shall use the common indexed-record interface.

**FFW-VFS-ISAM-002**  
The default ISAM provider shall use SQLite indexes for primary and secondary access paths.

**FFW-VFS-ISAM-003**  
ISAM implementation details shall remain encapsulated behind the storage-provider interface so a future native B-tree provider can be introduced without changing callers.

### 7.9 POSIX File Requirements

**FFW-VFS-POSIX-001**  
POSIX files and directories shall remain native host filesystem objects by default.

**FFW-VFS-POSIX-002**  
The system shall not copy POSIX file contents into SQLite merely to make them visible in FileForgeWorkbench.

**FFW-VFS-POSIX-003**  
The catalogue may register a POSIX root, file, or directory using a provider locator and optional metadata.

**FFW-VFS-POSIX-004**  
Users shall be able to work with explicitly mounted or registered filesystem roots subject to workspace and security policy.

**FFW-VFS-POSIX-005**  
External changes shall be detected through refresh, filesystem notifications where supported, or reconciliation.

**FFW-VFS-POSIX-006**  
Symlink handling shall be configurable, with loop detection and prevention of traversal beyond authorised roots.

**FFW-VFS-POSIX-007**  
Host permissions, file locking, case sensitivity, and path semantics shall be surfaced accurately and shall not be silently normalised into mainframe semantics.

## 8. Storage Provider Interface Requirements

**FFW-VFS-SPI-001**  
All physical access shall occur through a storage-provider interface.

The interface shall expose capabilities equivalent to:

```rust
trait StorageProvider {
    fn allocate(&self, specification: &DatasetSpecification) -> Result<ObjectId>;
    fn open(&self, object: &ObjectId, mode: OpenMode) -> Result<Handle>;
    fn stat(&self, object: &ObjectId) -> Result<ObjectMetadata>;
    fn rename(&self, object: &ObjectId, target: &ProviderLocator) -> Result<()>;
    fn delete(&self, object: &ObjectId, policy: DeletePolicy) -> Result<()>;
    fn list(&self, parent: &ObjectId) -> Result<Vec<DirectoryEntry>>;
    fn reconcile(&self, object: &ObjectId) -> Result<ReconciliationReport>;
}
```

The exact Rust API may differ, but the responsibilities shall remain separated.

**FFW-VFS-SPI-002**  
Providers shall declare capabilities rather than requiring callers to infer them from dataset type.

Example capabilities include:

- Stream read/write.
- Record read/write.
- Keyed access.
- Relative access.
- Append-only access.
- Member operations.
- Atomic rename.
- Locking.
- Snapshotting.
- Watch notifications.

**FFW-VFS-SPI-003**  
The native-filesystem provider and SQLite record provider shall implement a common error taxonomy.

**FFW-VFS-SPI-004**  
Provider-specific locators shall be opaque outside the provider and catalogue services.

## 9. Naming and Physical Mapping Requirements

**FFW-VFS-NAM-001**  
The logical dataset name shall be stored exactly in canonical catalogue form.

**FFW-VFS-NAM-002**  
The physical mapping algorithm shall protect against path traversal, reserved device names, illegal characters, case-folding collisions, and maximum path-length constraints.

**FFW-VFS-NAM-003**  
Mapping shall be deterministic or explicitly persisted so that a dataset can be found after restart.

**FFW-VFS-NAM-004**  
The implementation shall not rely on dots in a dataset name being translated directly into directory separators.

**FFW-VFS-NAM-005**  
Physical storage may use internal identifiers rather than logical names, with human-friendly names presented through the catalogue.

A preferred layout is:

```text
workspace/
├── catalog.db
├── datasets/
│   ├── objects/
│   │   ├── <dataset-uuid>.dat
│   │   └── <library-uuid>/
│   │       └── <member-uuid>.dat
│   └── staging/
├── indexed/
│   └── <dataset-uuid>.sqlite
└── recovery/
```

This avoids binding logical naming rules to host path rules.

## 10. Record Format and Encoding Requirements

**FFW-VFS-REC-001**  
Record codecs shall be separate from storage providers.

**FFW-VFS-REC-002**  
The system shall preserve fixed-block, variable-block, undefined/binary, and host text representations through explicit codecs.

**FFW-VFS-REC-003**  
CRLF, LF, and other host line endings shall not be assumed to represent mainframe record boundaries unless a selected import/export profile specifies that mapping.

**FFW-VFS-REC-004**  
Encoding conversion shall be explicit and shall support preservation of original bytes when round-trip fidelity is required.

**FFW-VFS-REC-005**  
Invalid record lengths or malformed variable-record descriptors shall produce diagnostic errors containing dataset identity, record position, and expected constraints.

## 11. Consistency and Transaction Requirements

SQLite and the host filesystem do not share a single transaction manager. Therefore, multi-resource operations shall use a staged protocol.

**FFW-VFS-TXN-001**  
Create operations shall stage physical content, create or reserve catalogue state, publish the physical object, and then mark the catalogue entry active.

**FFW-VFS-TXN-002**  
Delete operations shall first mark an entry pending deletion, move or tombstone physical content where practical, and then finalise catalogue state.

**FFW-VFS-TXN-003**  
Interrupted operations shall be discoverable through operation journals or transitional catalogue states.

**FFW-VFS-TXN-004**  
On start-up, the system shall detect and offer deterministic recovery for incomplete operations.

**FFW-VFS-TXN-005**  
Concurrent modification shall be controlled using locks, version tokens, SQLite transactions, provider-specific mechanisms, or a documented combination.

**FFW-VFS-TXN-006**  
The system shall not report an operation as successful until both catalogue and provider state satisfy the operation's postconditions.

## 12. Integrity, Backup, and Recovery Requirements

**FFW-VFS-INT-001**  
Managed content shall support optional checksums to detect unexpected physical modification or corruption.

**FFW-VFS-INT-002**  
Backup procedures shall capture the catalogue, SQLite record stores, native dataset files, library directories, and operation journals as one recoverable workspace.

**FFW-VFS-INT-003**  
A backup shall include a manifest containing schema version, provider configuration, object inventory, and integrity information.

**FFW-VFS-INT-004**  
Restore shall support restoration to the original workspace or remapping to a different root without changing logical dataset names.

**FFW-VFS-INT-005**  
The product shall provide diagnostics for orphaned physical objects and dangling catalogue entries.

**FFW-VFS-INT-006**  
Repair operations shall be previewable, auditable, and reversible where practical.

## 13. Security Requirements

**FFW-VFS-SEC-001**  
All resolved physical paths shall be constrained to authorised workspace roots unless the user explicitly mounts an external root.

**FFW-VFS-SEC-002**  
Path canonicalisation and traversal checks shall occur before filesystem access.

**FFW-VFS-SEC-003**  
Catalogue metadata shall not be treated as a substitute for host operating-system access controls.

**FFW-VFS-SEC-004**  
Sensitive dataset contents and credentials shall not be written to logs.

**FFW-VFS-SEC-005**  
SQLite connections shall use parameterised statements and controlled schema identifiers.

**FFW-VFS-SEC-006**  
Audit events shall identify the action, object, outcome, timestamp, and initiating principal or process where available.

## 14. Non-Functional Requirements

**FFW-VFS-NFR-001 Portability**  
The architecture shall operate on Windows, Linux, and macOS without changing the logical dataset model.

**FFW-VFS-NFR-002 Performance**  
Catalogue listing shall query metadata without loading dataset payloads.

**FFW-VFS-NFR-003 Scalability**  
The design shall permit large datasets and large libraries without placing all content into the central catalogue database.

**FFW-VFS-NFR-004 Testability**  
Catalogue, codec, and provider components shall be independently testable using temporary workspaces and deterministic fixtures.

**FFW-VFS-NFR-005 Observability**  
Storage operations shall emit structured diagnostic events with correlation identifiers.

**FFW-VFS-NFR-006 Extensibility**  
A future storage provider shall be addable without rewriting dataset editors or catalogue consumers.

**FFW-VFS-NFR-007 Git compatibility**  
Text-oriented PDS/PDSE members and selected sequential datasets shall be capable of being represented as ordinary files suitable for external version-control tooling.

**FFW-VFS-NFR-008 Data fidelity**  
The system shall not silently alter bytes, encoding, record boundaries, keys, or generation identity.

## 15. Prohibited and Discouraged Designs

The following designs shall be prohibited unless approved by an architectural decision record:

- Storing all PS, PDS, GDG, and POSIX content as BLOBs in the central catalogue database.
- Treating a logical dataset name as a raw host path.
- Allowing user-interface components to manipulate provider paths directly.
- Inferring mainframe record boundaries solely from CRLF or LF.
- Using PDS/PDSE host filenames without collision-safe logical-name mapping.
- Deleting rolled-off GDG generations without applying retention policy.
- Updating SQLite state and filesystem state without transitional states or recovery handling.
- Treating SQLite catalogue backup alone as a complete workspace backup.

## 16. Acceptance Criteria

### AC-001: Sequential dataset

Given a catalogued fixed-record PS dataset, when it is opened, edited, closed, and reopened, then its logical records, metadata, encoding policy, and physical mapping shall remain consistent.

### AC-002: PDS library

Given a PDS with multiple members, when one member is edited, then the affected member shall be independently persisted and other members shall remain unchanged.

### AC-003: Naming collision

Given two valid logical names that map to the same case-folded or escaped host filename, the allocation shall remain collision-free and both objects shall be retrievable by logical name.

### AC-004: GDG resolution

Given a GDG containing multiple active generations, when `0` and `-1` are resolved, then the catalogue shall return the correct current and previous generations from one consistent catalogue snapshot.

### AC-005: Concurrent GDG allocation

Given concurrent requests for `+1`, the system shall allocate unique absolute generation identities or fail one request without creating duplicate or ambiguous state.

### AC-006: KSDS keyed access

Given a KSDS with a defined unique key, when records are inserted and retrieved by key, then uniqueness, ordered traversal, update, deletion, and range retrieval shall conform to its dataset definition.

### AC-007: RRDS vacancy

Given an RRDS, when an unallocated relative record and an allocated blank record are read, then the two states shall be distinguishable.

### AC-008: POSIX external change

Given a registered POSIX file that is changed by an external tool, when the workspace is refreshed or reconciled, then FileForgeWorkbench shall report the changed state without overwriting it silently.

### AC-009: Interrupted allocation

Given an interruption between physical allocation and catalogue activation, when recovery runs, then the operation shall either complete deterministically or roll back without leaving an active dangling entry.

### AC-010: Workspace backup

Given a workspace containing native datasets and SQLite-backed indexed datasets, when backup and restore are performed, then catalogue resolution, dataset contents, member relationships, and generation relationships shall be restored consistently.

### AC-011: Cross-platform mapping

Given the same logical catalogue exported from one supported operating system and restored on another, then logical dataset names and relationships shall remain unchanged while physical locators are safely remapped.

### AC-012: Record fidelity

Given a binary or encoded mainframe dataset, when it is opened and saved without modification, then the resulting bytes shall remain unchanged unless an explicit conversion profile is applied.

## 17. Suggested Component Boundaries

```text
crates/
├── ffw-catalogue
│   ├── schema
│   ├── migrations
│   ├── naming
│   └── audit
├── ffw-vfs
│   ├── dataset_service
│   ├── provider_api
│   ├── transactions
│   └── reconciliation
├── ffw-provider-native
├── ffw-provider-sqlite-records
├── ffw-record-codecs
│   ├── fixed
│   ├── variable
│   ├── binary
│   └── text
└── ffw-mainframe-model
    ├── ps
    ├── pds
    ├── gdg
    ├── vsam
    └── isam
```

This is a suggested organisation, not a mandatory repository layout.

## 18. Implementation Phases

### Phase 1: Foundations

- Define catalogue schema and migrations.
- Define dataset and provider interfaces.
- Implement safe logical-name and physical-locator mapping.
- Implement native PS and POSIX providers.
- Implement operation journalling and reconciliation foundations.

### Phase 2: Libraries and generations

- Implement PDS/PDSE directory and member handling.
- Implement GDG base and generation resolution.
- Add lifecycle and retention processing.

### Phase 3: Indexed organisations

- Implement KSDS and ISAM keyed stores.
- Implement RRDS.
- Implement ESDS and stable record addressing.
- Add alternate indexes and record browsing.

### Phase 4: Governance and resilience

- Implement integrity manifests, workspace backup, restore, and repair.
- Add audit views and structured diagnostics.
- Add migration tools for any earlier all-in-SQLite representation.
- Complete cross-platform and failure-injection testing.

## 19. Migration from an Earlier SQLite-Content Model

If an earlier prototype stored complete file payloads in SQLite, the migration shall:

1. Inventory existing logical objects and their metadata.
2. Validate payload readability and calculate integrity hashes.
3. Allocate new provider objects in a staging area.
4. Export payloads using the applicable record codec without unintended conversion.
5. Repoint catalogue entries using a versioned transaction or migration state.
6. Verify logical reads against pre-migration hashes or record-level checks.
7. Retain a rollback manifest until migration is accepted.
8. Remove obsolete payload tables only through a later, explicit migration.

## 20. Risks and Mitigations

| Risk | Mitigation |
|---|---|
| SQLite and filesystem state diverge | Transitional states, operation journal, reconciliation, and recovery |
| Logical names are unsafe as paths | Stable identifiers and collision-safe persisted mapping |
| External tools modify native files | Checksums, timestamps, watchers, refresh, and conflict handling |
| Large indexed datasets contend in one database | Dedicated per-dataset stores or configurable providers |
| Host line endings alter record meaning | Explicit record codecs independent of text-line handling |
| Backup captures only part of a workspace | Manifest-driven workspace backup and restore |
| Cross-platform case and path differences | Canonical logical names and provider-specific locators |
| GDG retention causes accidental loss | Policy-based roll-off, tombstones, audit, and recoverable deletion |

## 21. Traceability Summary

| Architectural concern | Principal requirement groups |
|---|---|
| SQLite as catalogue | `FFW-VFS-CAT-*` |
| Native file storage | `FFW-VFS-PS-*`, `FFW-VFS-PDS-*`, `FFW-VFS-POSIX-*` |
| GDG semantics | `FFW-VFS-GDG-*` |
| SQLite-backed keyed/relative data | `FFW-VFS-KSDS-*`, `FFW-VFS-RRDS-*`, `FFW-VFS-ISAM-*` |
| ESDS append semantics | `FFW-VFS-ESDS-*` |
| Provider abstraction | `FFW-VFS-SPI-*` |
| Cross-resource consistency | `FFW-VFS-TXN-*`, `FFW-VFS-INT-*` |
| Record and encoding fidelity | `FFW-VFS-REC-*` |
| Security and path safety | `FFW-VFS-SEC-*`, `FFW-VFS-NAM-*` |
| Quality attributes | `FFW-VFS-NFR-*` |

## 22. Final Recommendation

FileForgeWorkbench should treat SQLite as the **catalogue and selected indexed-record engine**, not as the universal container for every file. Sequential datasets, PDS/PDSE members, GDG generation payloads, and POSIX content should normally remain as native files and directories. KSDS, RRDS, and ISAM behaviours are appropriate candidates for SQLite-backed record stores because their defining semantics require transactional keyed or relative access.

This hybrid design provides a clean separation between logical mainframe semantics and physical storage. It supports cross-platform operation, native tooling, Git-friendly member files, scalable payload handling, and future provider extensibility while preserving the governed metadata and traceability needed by FileForgeWorkbench.
