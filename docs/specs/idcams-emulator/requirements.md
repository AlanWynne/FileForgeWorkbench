# Requirements Document

## Introduction

This specification defines the **IDCAMS Emulator** (`ff-idcams`) — the command parsing and orchestration layer for IBM Access Method Services (IDCAMS) within the FileForgeWorkbench ecosystem. The emulator enables mainframe developers to use familiar IDCAMS commands (DEFINE, DELETE, ALTER, LISTCAT, PRINT, REPRO, VERIFY, EXPORT, IMPORT, BLDINDEX) in a local desktop environment without requiring z/OS.

**Ownership Principle (ADR-001):** ff-idcams owns ONLY command parsing and orchestration. All actual catalog persistence, VSAM record operations, dataset allocation, and storage access are delegated to downstream service crates through trait interfaces. ff-idcams is a thin command interpreter, not a monolithic implementation.

**Governance Reference:** `.kiro/specs/dataset-ownership-model/requirements.md` — this specification aligns with the Dataset Ownership Model governance document. Where any conflict exists, the governance document takes precedence.

**Delegation Model:**

| Command | Parse Owner | Execution Delegate |
|---------|------------|-------------------|
| DEFINE CLUSTER | ff-idcams | ff-dataset-catalog (create_dataset) + ff-vsam-services (initialize_dataset) |
| DEFINE AIX | ff-idcams | ff-vsam-services (define_aix) |
| DEFINE PATH | ff-idcams | ff-vsam-services (define_path) |
| DEFINE GDG | ff-idcams | ff-dataset-catalog (create_gdg_base) |
| DELETE | ff-idcams | ff-vsam-services (destroy_dataset) + ff-dataset-catalog (delete_dataset) |
| ALTER | ff-idcams | ff-dataset-catalog (update_dataset) |
| LISTCAT | ff-idcams | ff-dataset-catalog (list_datasets, get_dataset_attributes) |
| PRINT | ff-idcams | ff-vsam-services (browse) or ff-vfs (read_stream) |
| REPRO | ff-idcams | ff-vsam-services (get/put) or ff-vfs (read/write) |
| VERIFY | ff-idcams | ff-vsam-services (verify_integrity) |
| EXPORT | ff-idcams | ff-dataset-catalog (export_dataset) |
| IMPORT | ff-idcams | ff-dataset-catalog (import_dataset) |
| BLDINDEX | ff-idcams | ff-vsam-services (build_index) |

---

## Glossary

- **ff-idcams**: The IDCAMS Emulator crate — owns DEFINE command parsing, LISTCAT command parsing, ALTER command parsing, DELETE command parsing, REPRO command parsing, IMPORT command parsing, EXPORT command parsing, BLDINDEX command parsing, PRINT command parsing, and VERIFY command parsing. [ADR-001]
- **IDCAMS_Parser**: The component within ff-idcams that tokenizes and parses IDCAMS control statements into structured command representations.
- **Command_Executor**: The component within ff-idcams that takes a parsed command and orchestrates delegation to downstream services.
- **CatalogService**: The trait interface exposed by ff-dataset-catalog through which ff-idcams performs all catalog operations. [ADR-001]
- **VsamService**: The trait interface exposed by ff-vsam-services through which ff-idcams performs all VSAM operations. [ADR-001]
- **AllocatorService**: The trait interface exposed by ff-dataset-allocator through which ff-idcams performs dataset resolution. [ADR-001]
- **ff-vfs**: The Virtual File System crate — owns resource URIs, provider registration, and content access abstraction. [ADR-001]
- **KSDS**: Key Sequenced Data Set — a VSAM dataset type where records are ordered by a primary key. Organization keyword: INDEXED.
- **ESDS**: Entry Sequenced Data Set — a VSAM dataset type where records are stored in insertion order. Organization keyword: NONINDEXED.
- **RRDS**: Relative Record Data Set — a VSAM dataset type where records are addressed by relative record number. Organization keyword: NUMBERED.
- **LDS**: Linear Data Set — a VSAM dataset type providing byte-oriented linear access with no record structure. Organization keyword: LINEAR.
- **AIX**: Alternate Index — a secondary index over a VSAM base cluster providing access by an alternate key.
- **PATH**: A named access route connecting an alternate index to its base cluster for transparent access.
- **GDG**: Generation Data Group — a collection of related non-VSAM datasets managed as a group with automatic version rollover.
- **SYSIN**: The input stream from which IDCAMS reads control statements (commands).
- **MAXCC**: The maximum condition code encountered across all commands in a single IDCAMS invocation.
- **LASTCC**: The condition code returned by the most recently executed command.
- **Condition_Code**: A numeric return value (0=success, 4=warning, 8=error, 12=severe error, 16=catastrophic) indicating command outcome.
- **Control_Statement**: A single IDCAMS command with its parameters, potentially spanning multiple lines via continuation.
- **Modal_Command**: An IF/THEN/ELSE construct that conditionally executes commands based on LASTCC or MAXCC values.
- **CLUSTER**: A VSAM dataset consisting of a DATA component and optionally an INDEX component.
- **DATA_Component**: The physical storage area holding the actual records of a VSAM cluster.
- **INDEX_Component**: The B-tree index structure for a KSDS cluster, enabling key-based record access.
- **SHAREOPTIONS**: VSAM parameter controlling cross-region and cross-system sharing behaviour (values 1-4 for each).
- **FREESPACE**: VSAM parameter specifying percentage of free space to leave in CI (control interval) and CA (control area) for future insertions.
- **Atomic_Execution**: The guarantee that a parsed IDCAMS command either fully succeeds (all delegated operations complete) or fully fails (all partial state is rolled back).

---

## Requirements

### Requirement 1: IDCAMS Control Statement Parser

**User Story:** As a mainframe developer, I want ff-idcams to parse IDCAMS control statements with the same syntax rules as z/OS IDCAMS, so that I can reuse existing IDCAMS scripts without modification.

#### Acceptance Criteria

1. THE IDCAMS_Parser SHALL tokenize input text into a sequence of commands, where each command consists of a verb (DEFINE, DELETE, ALTER, LISTCAT, PRINT, REPRO, VERIFY, EXPORT, IMPORT, BLDINDEX, SET, IF) followed by parameters.
2. THE IDCAMS_Parser SHALL recognize parameters enclosed in parentheses, supporting nested parentheses for sub-parameter lists (e.g., `KEYS(8 0)`, `RECORDSIZE(80 80)`, `FREESPACE(20 10)`).
3. THE IDCAMS_Parser SHALL support continuation across multiple lines: a line ending with a hyphen (`-`) indicates continuation on the next line, and leading/trailing whitespace on continuation lines SHALL be handled equivalently to z/OS behaviour.
4. THE IDCAMS_Parser SHALL treat the semicolon (`;`) as a command separator, allowing multiple commands on a single line.
5. THE IDCAMS_Parser SHALL treat text following `/*` through `*/` as a comment and exclude commented text from parsing.
6. THE IDCAMS_Parser SHALL recognize single-line comments starting with `//` (a full line beginning with `//` after optional whitespace) and exclude them from command parsing.
7. THE IDCAMS_Parser SHALL be case-insensitive for command verbs and parameter keywords (e.g., `DEFINE`, `Define`, and `define` are equivalent).
8. THE IDCAMS_Parser SHALL accept dataset names containing up to 44 characters, composed of qualifiers (1-8 characters each) separated by periods, where each qualifier starts with an alphabetic character or national character (@, #, $).
9. IF the IDCAMS_Parser encounters an unrecognized command verb, THEN THE IDCAMS_Parser SHALL produce an AST containing error nodes marking the failure position, and return a parse error with message code IDC0001E indicating the invalid verb and its position in the input. WHEN both unrecognized verbs and malformed parameters occur, THE parser SHALL report the error for whichever issue is detected first during left-to-right parsing.
10. IF the IDCAMS_Parser encounters a malformed parameter (missing closing parenthesis, invalid nesting), THEN THE IDCAMS_Parser SHALL produce an AST containing error nodes marking the failure position, and return a parse error with message code IDC0002E indicating the parameter position and nature of the syntax error.
11. THE IDCAMS_Parser SHALL produce a structured Abstract Syntax Tree (AST) representation of each parsed command, suitable for validation and execution by the Command_Executor.
12. FOR ALL valid IDCAMS control statements, parsing then pretty-printing then re-parsing SHALL produce an equivalent AST (round-trip property).

---

### Requirement 2: DEFINE CLUSTER Command

**User Story:** As a mainframe developer, I want to define VSAM clusters (KSDS, ESDS, RRDS, LDS) using the same DEFINE CLUSTER syntax as z/OS IDCAMS, so that I can create datasets locally that match my mainframe definitions.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses a DEFINE CLUSTER command, THE IDCAMS_Parser SHALL extract the cluster NAME, organization type (INDEXED/NONINDEXED/NUMBERED/LINEAR), and all specified parameters into a DefineClusterCommand structure.
2. WHEN a DEFINE CLUSTER command specifies INDEXED organization, THE Command_Executor SHALL invoke `CatalogService::create_dataset()` with DSORG=KSDS and the parsed attributes, then invoke `VsamService::initialize_dataset()` with VsamType::Ksds and the key parameters.
3. WHEN a DEFINE CLUSTER command specifies NONINDEXED organization, THE Command_Executor SHALL invoke `CatalogService::create_dataset()` with DSORG=ESDS, then invoke `VsamService::initialize_dataset()` with VsamType::Esds.
4. WHEN a DEFINE CLUSTER command specifies NUMBERED organization, THE Command_Executor SHALL invoke `CatalogService::create_dataset()` with DSORG=RRDS, then invoke `VsamService::initialize_dataset()` with VsamType::Rrds.
5. WHEN a DEFINE CLUSTER command specifies LINEAR organization, THE Command_Executor SHALL invoke `CatalogService::create_dataset()` with DSORG=LDS, then invoke `VsamService::initialize_dataset()` with VsamType::Lds.
6. THE IDCAMS_Parser SHALL parse the NAME parameter as a 1-44 character dataset name conforming to z/OS DSN syntax rules.
7. THE IDCAMS_Parser SHALL parse the VOLUMES parameter as one or more volume serial identifiers (1-6 characters each).
8. THE IDCAMS_Parser SHALL parse space allocation parameters in one of these forms: CYLINDERS(primary secondary), TRACKS(primary secondary), RECORDS(primary secondary), or KILOBYTES(primary secondary), where primary and secondary are positive integers.
9. THE IDCAMS_Parser SHALL parse the RECORDSIZE(average maximum) parameter as two positive integers specifying average and maximum record lengths in bytes.
10. THE IDCAMS_Parser SHALL parse the KEYS(length offset) parameter as two non-negative integers specifying key length (1-255) and key offset (0-based byte position) for INDEXED clusters.
11. THE IDCAMS_Parser SHALL parse the FREESPACE(ci_percent ca_percent) parameter as two integers (0-100) specifying free space percentages for control interval and control area.
12. THE IDCAMS_Parser SHALL parse the SHAREOPTIONS(crossregion crosssystem) parameter as two integers (1-4 each) specifying cross-region and cross-system sharing levels.
13. THE IDCAMS_Parser SHALL parse mutually exclusive parameters SPEED and RECOVERY, where SPEED skips preformat and RECOVERY preformats the dataset.
14. THE IDCAMS_Parser SHALL parse the REUSE parameter as a boolean flag indicating the cluster can be opened as a reusable dataset (equivalent to empty on open).
15. THE IDCAMS_Parser SHALL parse DATA and INDEX component sub-definitions within DEFINE CLUSTER, each accepting NAME, VOLUMES, CYLINDERS/TRACKS/RECORDS, RECORDSIZE, KEYS, CONTROLINTERVALSIZE, and FREESPACE parameters.
16. THE IDCAMS_Parser SHALL parse the CONTROLINTERVALSIZE(bytes) parameter as a positive integer specifying the CI size for DATA or INDEX components.
17. THE IDCAMS_Parser SHALL parse the BUFFERSPACE(bytes) parameter as a positive integer specifying the minimum buffer allocation.
18. IF DEFINE CLUSTER specifies INDEXED but omits the KEYS parameter, THEN THE Command_Executor SHALL return condition code 12 with message IDC0503E indicating KEYS is required for INDEXED clusters.
19. IF DEFINE CLUSTER specifies a NAME that already exists in the catalog, THEN THE Command_Executor SHALL return condition code 12 with message IDC0514E indicating a duplicate dataset name.
20. IF the CatalogService::create_dataset() invocation succeeds but VsamService::initialize_dataset() fails, THEN THE Command_Executor SHALL invoke CatalogService::delete_dataset() to roll back the catalog entry and return condition code 12 with the downstream error message. This rollback SHALL apply to any failure after catalog creation succeeds, not only VsamService failures.
21. WHEN DEFINE CLUSTER completes successfully, THE Command_Executor SHALL set LASTCC to 0 and emit message IDC0001I confirming creation with the cluster name.

---

### Requirement 3: DEFINE ALTERNATEINDEX Command

**User Story:** As a mainframe developer, I want to define alternate indexes over VSAM base clusters, so that I can access records by secondary keys.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses a DEFINE ALTERNATEINDEX command, THE IDCAMS_Parser SHALL extract the AIX NAME, RELATE (base cluster name), KEYS(length offset), and uniqueness option into a DefineAixCommand structure.
2. THE IDCAMS_Parser SHALL parse the RELATE(base_cluster_name) parameter as a mandatory 1-44 character dataset name identifying the base cluster.
3. THE IDCAMS_Parser SHALL parse the KEYS(length offset) parameter specifying the alternate key field within base cluster records.
4. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters UNIQUEKEY (default) and NONUNIQUEKEY specifying whether duplicate alternate key values are permitted.
5. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters UPGRADE (default) and NOUPGRADE specifying whether the AIX is maintained during base cluster updates.
6. THE IDCAMS_Parser SHALL parse the RECORDSIZE(average maximum) parameter for the AIX, which must account for pointer records when NONUNIQUEKEY is specified.
7. WHEN the Command_Executor processes a parsed DEFINE ALTERNATEINDEX command, THE Command_Executor SHALL invoke `VsamService::define_aix()` with the base cluster DSN, AIX DSN, key field parameters, and uniqueness/upgrade options.
8. IF DEFINE ALTERNATEINDEX specifies a RELATE name that does not exist in the catalog, THEN THE Command_Executor SHALL return condition code 12 with message IDC0510E indicating the base cluster is not found.
9. IF DEFINE ALTERNATEINDEX specifies a RELATE name that is not a VSAM cluster (e.g., it refers to a GDG or non-VSAM dataset), THEN THE Command_Executor SHALL return condition code 12 with message IDC0511E indicating invalid base cluster type.
10. WHEN DEFINE ALTERNATEINDEX completes successfully (VsamService::define_aix returns success), THE Command_Executor SHALL set LASTCC to 0 and emit message IDC0001I confirming the AIX definition. Success indicators SHALL NOT be emitted when any validation or downstream operation fails.

---

### Requirement 4: DEFINE PATH Command

**User Story:** As a mainframe developer, I want to define paths connecting alternate indexes to base clusters, so that programs can transparently access records through alternate keys.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses a DEFINE PATH command, THE IDCAMS_Parser SHALL extract the path NAME and PATHENTRY (AIX name) into a DefinePathCommand structure.
2. THE IDCAMS_Parser SHALL parse the NAME parameter as a 1-44 character dataset name for the path.
3. THE IDCAMS_Parser SHALL parse the PATHENTRY(aix_name) parameter as a mandatory 1-44 character name identifying the alternate index.
4. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters UPDATE (default) and NOUPDATE specifying whether accessing the base cluster through this path triggers AIX maintenance.
5. WHEN the Command_Executor processes a parsed DEFINE PATH command, THE Command_Executor SHALL invoke `VsamService::define_path()` with the path name, AIX name, and update mode.
6. IF DEFINE PATH specifies a PATHENTRY name that does not exist or is not an alternate index, THEN THE Command_Executor SHALL validate AIX existence during parsing and return condition code 12 with message IDC0512E indicating the AIX is not found — this check occurs before execution is attempted.
7. WHEN DEFINE PATH completes successfully, THE Command_Executor SHALL set LASTCC to 0 and emit message IDC0001I confirming the path definition.

---

### Requirement 5: DEFINE GDG Command

**User Story:** As a mainframe developer, I want to define Generation Data Group base entries, so that I can manage versioned datasets with automatic rollover.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses a DEFINE GDG command, THE IDCAMS_Parser SHALL extract the GDG NAME, LIMIT, and management options into a DefineGdgCommand structure.
2. THE IDCAMS_Parser SHALL parse the NAME parameter as a 1-44 character dataset name for the GDG base.
3. THE IDCAMS_Parser SHALL parse the LIMIT(n) parameter as a positive integer (1-255) specifying the maximum number of generations to retain.
4. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters SCRATCH (default) and NOSCRATCH specifying whether rolled-off generations are physically deleted or only uncataloged.
5. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters NOEMPTY (default) and EMPTY specifying whether all generations are rolled off when the limit is exceeded or only the oldest.
6. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters FIFO and LIFO (default LIFO) specifying generation deactivation order — FIFO deactivates oldest first, LIFO deactivates newest first.
7. WHEN the Command_Executor processes a parsed DEFINE GDG command, THE Command_Executor SHALL invoke `CatalogService::create_gdg_base()` with the base DSN, limit, scratch policy, empty policy, and ordering.
8. IF DEFINE GDG omits the LIMIT parameter, THEN THE Command_Executor SHALL return condition code 12 with message IDC0520E indicating LIMIT is required.
9. IF DEFINE GDG specifies a NAME that already exists in the catalog, THEN THE Command_Executor SHALL return condition code 12 with message IDC0514E indicating a duplicate name.
10. WHEN DEFINE GDG completes successfully (CatalogService::create_gdg_base returns success), THE Command_Executor SHALL verify the operation actually succeeded before setting LASTCC to 0 and emitting message IDC0001I confirming the GDG base definition. IF the downstream operation returns an error, success indicators SHALL NOT be emitted.


---

### Requirement 6: DELETE Command

**User Story:** As a mainframe developer, I want to delete datasets, clusters, alternate indexes, paths, and GDG bases using the same DELETE syntax as z/OS IDCAMS, so that I can manage dataset lifecycle locally.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses a DELETE command, THE IDCAMS_Parser SHALL extract the entry name(s), entry type, and option parameters into a DeleteCommand structure.
2. THE IDCAMS_Parser SHALL parse one or more entry names (1-44 characters each) as the targets for deletion, supporting a list of names enclosed in parentheses.
3. THE IDCAMS_Parser SHALL parse the entry type keyword: CLUSTER, ALTERNATEINDEX, PATH, GDG, NONVSAM, or USERCATALOG specifying the type of entry to delete.
4. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters PURGE and NOPURGE (default), where PURGE allows deletion regardless of retention period and NOPURGE respects retention period.
5. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters FORCE and NOFORCE (default), where FORCE deletes a non-empty GDG base including all generations.
6. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters ERASE and NOERASE (default), where ERASE overwrites the dataset content with binary zeros before deletion.
7. THE IDCAMS_Parser SHALL parse the SCRATCH and NOSCRATCH parameters for GDG generation handling during deletion.
8. WHEN the Command_Executor processes a DELETE command with type CLUSTER, THE Command_Executor SHALL invoke `VsamService::destroy_dataset()` to clean up VSAM structures, then invoke `CatalogService::delete_dataset()` to remove the catalog entry.
9. WHEN the Command_Executor processes a DELETE command with type ALTERNATEINDEX, THE Command_Executor SHALL invoke `VsamService::destroy_dataset()` for the AIX, then invoke `CatalogService::delete_dataset()`.
10. WHEN the Command_Executor processes a DELETE command with type PATH, THE Command_Executor SHALL invoke `VsamService::delete_path()` to remove the path association, then invoke `CatalogService::delete_dataset()` for the path entry.
11. WHEN the Command_Executor processes a DELETE command with type GDG, THE Command_Executor SHALL invoke `CatalogService::delete_gdg_base()` which handles generation cleanup based on the FORCE option.
12. WHEN the Command_Executor processes a DELETE command with type NONVSAM, THE Command_Executor SHALL invoke `CatalogService::delete_dataset()` to remove the catalog entry and associated storage.
13. IF DELETE specifies an entry name that does not exist in the catalog, THEN THE Command_Executor SHALL return condition code 8 with message IDC0550E indicating entry not found.
14. IF DELETE specifies a type that does not match the actual catalog entry type, THEN THE Command_Executor SHALL return condition code 12 with message IDC0551E indicating type mismatch.
15. IF VsamService::destroy_dataset() fails during DELETE CLUSTER, THEN THE Command_Executor SHALL NOT proceed with catalog deletion and SHALL return condition code 12 with the downstream error — deletion is atomic.
16. WHEN DELETE completes successfully for each entry, THE Command_Executor SHALL set LASTCC to 0 and emit message IDC0002I confirming deletion with the entry name.
17. WHEN DELETE is given a list of names, THE Command_Executor SHALL process each name sequentially, setting LASTCC after each, and continuing to the next entry regardless of individual failures (MAXCC tracks the highest code).

---

### Requirement 7: ALTER Command

**User Story:** As a mainframe developer, I want to modify dataset attributes using the ALTER command, so that I can adjust VSAM cluster properties after initial definition.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses an ALTER command, THE IDCAMS_Parser SHALL extract the entry name and the set of attributes to modify into an AlterCommand structure.
2. THE IDCAMS_Parser SHALL parse the entry name (1-44 characters) identifying the dataset to alter.
3. THE IDCAMS_Parser SHALL parse alterable attributes including: FREESPACE(ci_percent ca_percent), SHAREOPTIONS(crossregion crosssystem), BUFFERSPACE(bytes), RECORDSIZE(average maximum), KEYS(length offset), ADDVOLUMES(volser...), REMOVEVOLUMES(volser...).
4. THE IDCAMS_Parser SHALL parse the NEWNAME(new_dsn) parameter for renaming the entry.
5. THE IDCAMS_Parser SHALL parse the NULLIFY keyword with sub-parameters specifying attributes to remove or reset to defaults.
6. WHEN the Command_Executor processes a parsed ALTER command, THE Command_Executor SHALL invoke `CatalogService::update_dataset()` with the entry name and modified attributes.
7. IF ALTER specifies NEWNAME, THE Command_Executor SHALL invoke `CatalogService::rename_dataset()` with the old and new names.
8. IF ALTER specifies an entry name that does not exist in the catalog, THEN THE Command_Executor SHALL return condition code 8 with message IDC0560E indicating entry not found.
9. IF ALTER specifies an attribute that is not modifiable for the entry type (e.g., changing KEYS on an ESDS), THEN THE Command_Executor SHALL return condition code 12 with message IDC0561E indicating the attribute cannot be altered.
10. WHEN ALTER completes successfully, THE Command_Executor SHALL set LASTCC to 0 and emit message IDC0003I confirming the alteration.

---

### Requirement 8: LISTCAT Command

**User Story:** As a mainframe developer, I want to list catalog entries with the same LISTCAT syntax and output format as z/OS IDCAMS, so that I can inspect dataset metadata in a familiar format.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses a LISTCAT command, THE IDCAMS_Parser SHALL extract the filter criteria (ENTRIES or LEVEL), display level, and catalog specification into a ListcatCommand structure.
2. THE IDCAMS_Parser SHALL parse the ENTRIES(name...) parameter as one or more specific dataset names (with optional generic wildcard `*`) to list.
3. THE IDCAMS_Parser SHALL parse the LEVEL(qualifier) parameter as a high-level qualifier filter selecting all entries under that qualifier. THE LEVEL and ENTRIES parameters SHALL be mutually exclusive — IF both are specified, THE IDCAMS_Parser SHALL return a parse error indicating only one may be used.
4. THE IDCAMS_Parser SHALL parse the mutually exclusive display level parameters: NAME (names only), HISTORY (names + history), VOLUME (names + volume info), ALL (complete attribute display). Default is NAME.
5. THE IDCAMS_Parser SHALL parse the CATALOG(catalog_name) parameter specifying which catalog to query.
6. THE IDCAMS_Parser SHALL parse the entry type filter keywords: CLUSTER, ALTERNATEINDEX, PATH, GDG, NONVSAM, USERCATALOG, DATA, INDEX, or ALL (default ALL).
7. WHEN the Command_Executor processes a parsed LISTCAT command, THE Command_Executor SHALL invoke `CatalogService::list_datasets()` with the parsed filter criteria to retrieve matching entries.
8. WHEN the display level is ALL, THE Command_Executor SHALL invoke `CatalogService::get_dataset_attributes()` for each matched entry to retrieve full attribute details.
9. THE Command_Executor SHALL format LISTCAT output in the z/OS hierarchical format: cluster entries show DATA and INDEX components indented beneath the cluster, with AIX and PATH entries listed as associations.
10. WHEN display level is ALL, THE Command_Executor SHALL display for each cluster: NAME, HISTORY (creation date, last access), ASSOCIATIONS (AIX, PATH), ATTRIBUTES (RECFM, LRECL, KEYLEN, RKP, MAXLRECL, AVGLRECL), STATISTICS (record count, CI splits, CA splits), ALLOCATION (space, volumes), and VOLUME information.
11. WHEN display level is NAME, THE Command_Executor SHALL display only the entry names, one per line, with type indication (CLUSTER, DATA, INDEX, AIX, PATH, GDG, NONVSAM).
12. IF LISTCAT finds no matching entries, THEN THE Command_Executor SHALL return condition code 4 with message IDC0565W indicating no entries found matching the filter.
13. WHEN LISTCAT completes with results, THE Command_Executor SHALL set LASTCC to 0.
14. THE IDCAMS_Parser SHALL support generic filtering using asterisk (`*`) as a wildcard in ENTRIES parameter positions (e.g., `MY.DATA.*` matches all entries with prefix `MY.DATA.`).

---

### Requirement 9: PRINT Command

**User Story:** As a mainframe developer, I want to print (display) dataset contents using the PRINT command with support for character, hex, and dump formats, so that I can browse dataset contents without a separate tool.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses a PRINT command, THE IDCAMS_Parser SHALL extract the input dataset specification, format, and selection criteria into a PrintCommand structure.
2. THE IDCAMS_Parser SHALL parse the mutually exclusive input parameters INFILE(ddname) and INDATASET(dsn) specifying the dataset to print.
3. THE IDCAMS_Parser SHALL parse the mutually exclusive format parameters: CHARACTER (printable characters with non-printable shown as periods), HEX (hexadecimal representation), DUMP (combined character and hex display). Default is DUMP.
4. THE IDCAMS_Parser SHALL parse key-based selection: FROMKEY(key_value) and TOKEY(key_value) specifying the range of keys to print for KSDS datasets. WHEN FROMKEY is specified (with or without TOKEY), THE dataset SHALL be validated as KSDS type — specifying FROMKEY for a non-KSDS dataset SHALL produce an error.
5. THE IDCAMS_Parser SHALL parse address-based selection: FROMADDRESS(rba) and TOADDRESS(rba) specifying RBA range for ESDS datasets. WHEN FROMADDRESS is specified, THE dataset SHALL be validated as ESDS type — specifying FROMADDRESS for a non-ESDS dataset SHALL produce condition code 12 with an error message.
6. THE IDCAMS_Parser SHALL parse record-number-based selection: FROMRECORD(n) and TORECORD(n) specifying relative record number range.
7. THE IDCAMS_Parser SHALL parse the COUNT(n) parameter as a positive integer specifying the maximum number of records to print.
8. THE IDCAMS_Parser SHALL parse the SKIP(n) parameter as a non-negative integer specifying the number of records to skip before printing.
9. WHEN the Command_Executor processes a PRINT command for a VSAM dataset, THE Command_Executor SHALL invoke `VsamService::open()` followed by `VsamService::start_browse()` and `VsamService::next_record()` to retrieve records within the specified range.
10. WHEN the Command_Executor processes a PRINT command for a non-VSAM sequential dataset, THE Command_Executor SHALL invoke ff-vfs read operations to retrieve the content.
11. THE Command_Executor SHALL format each record according to the selected format (CHARACTER, HEX, or DUMP) and write to the output stream.
12. WHEN format is DUMP, THE Command_Executor SHALL display each record with: byte offset in hexadecimal, hex representation of the bytes (groups of 4 bytes separated by spaces), and the character interpretation (non-printable bytes shown as periods).
13. IF PRINT specifies a dataset that does not exist, THEN THE Command_Executor SHALL return condition code 12 with message IDC0570E indicating dataset not found.
14. IF PRINT specifies FROMKEY/TOKEY for a non-KSDS dataset, THEN THE Command_Executor SHALL return condition code 12 with message IDC0571E indicating key selection requires a KSDS.
15. WHEN PRINT completes successfully, THE Command_Executor SHALL set LASTCC to 0 and emit a summary message indicating the number of records printed.

---

### Requirement 10: REPRO Command

**User Story:** As a mainframe developer, I want to copy records between datasets using the REPRO command, so that I can load data, unload backups, and transfer records between VSAM and sequential datasets.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses a REPRO command, THE IDCAMS_Parser SHALL extract the input source, output target, selection criteria, and copy options into a ReproCommand structure.
2. THE IDCAMS_Parser SHALL parse the mutually exclusive input parameters INFILE(ddname) and INDATASET(dsn) specifying the source dataset.
3. THE IDCAMS_Parser SHALL parse the mutually exclusive output parameters OUTFILE(ddname) and OUTDATASET(dsn) specifying the target dataset.
4. THE IDCAMS_Parser SHALL parse key-based selection: FROMKEY(key_value) and TOKEY(key_value) for KSDS source datasets.
5. THE IDCAMS_Parser SHALL parse address-based selection: FROMADDRESS(rba) and TOADDRESS(rba) for ESDS source datasets.
6. THE IDCAMS_Parser SHALL parse the COUNT(n) and SKIP(n) parameters controlling how many records to copy and how many to skip.
7. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters REPLACE and NOREPLACE (default), where REPLACE overwrites existing records with matching keys in the target, and NOREPLACE skips duplicates.
8. WHEN the Command_Executor processes a REPRO between two VSAM datasets, THE Command_Executor SHALL invoke `VsamService::start_browse()` on the source and `VsamService::put()` on the target for each record within the selection range.
9. WHEN the Command_Executor processes a REPRO from a sequential dataset to a VSAM dataset, THE Command_Executor SHALL invoke ff-vfs read operations on the source and `VsamService::put()` on the target.
10. WHEN the Command_Executor processes a REPRO from a VSAM dataset to a sequential dataset, THE Command_Executor SHALL invoke `VsamService::start_browse()` on the source and ff-vfs write operations on the target.
11. WHEN the Command_Executor processes a REPRO between two sequential datasets, THE Command_Executor SHALL invoke ff-vfs read on the source and ff-vfs write on the target.
12. IF REPRO encounters a duplicate key in the target KSDS and NOREPLACE is in effect, THEN THE Command_Executor SHALL skip the record and set LASTCC to 4 with message IDC0580W for each duplicate.
13. IF REPRO encounters a duplicate key in the target KSDS and REPLACE is in effect, THEN THE Command_Executor SHALL explicitly invoke `VsamService::put()` to overwrite the existing record.
14. IF REPRO specifies a source dataset that does not exist, THEN THE Command_Executor SHALL return condition code 12 with message IDC0581E indicating source not found.
15. IF REPRO specifies a target dataset that does not exist, THEN THE Command_Executor SHALL return condition code 12 with message IDC0582E indicating target not found.
16. WHEN REPRO completes, THE Command_Executor SHALL emit a summary message indicating the number of records copied and the number of records skipped (if any).
17. THE Command_Executor SHALL process REPRO as an atomic operation — IF a write to the target fails mid-copy, THE Command_Executor SHALL report the error with the number of records successfully copied before the failure.


---

### Requirement 11: VERIFY Command

**User Story:** As a mainframe developer, I want to verify dataset integrity using the VERIFY command, so that I can detect and report corruption in VSAM datasets after abnormal termination.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses a VERIFY command, THE IDCAMS_Parser SHALL extract the dataset specification into a VerifyCommand structure.
2. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters FILE(ddname) and DATASET(dsn) specifying the dataset to verify.
3. WHEN the Command_Executor processes a VERIFY command, THE Command_Executor SHALL invoke `VsamService::verify_integrity()` with the dataset DSN.
4. THE VsamService::verify_integrity() SHALL check: (a) end-of-file pointer consistency (the logical EOF matches the physical extent), (b) index integrity for KSDS (all index entries point to valid data CIs), and (c) data component consistency (no orphaned records beyond EOF).
5. IF VERIFY detects integrity issues, THEN THE Command_Executor SHALL set LASTCC to 0 (VERIFY corrects end-of-file) and emit message IDC0001I indicating the dataset was verified and the end-of-file marker was reset.
6. IF VERIFY finds no issues, THEN THE Command_Executor SHALL set LASTCC to 0 and emit message IDC0590I indicating the dataset is consistent.
7. IF VERIFY specifies a dataset that does not exist or cannot be accessed (including permission or I/O failures), THEN THE Command_Executor SHALL return condition code 12 with message IDC0591E indicating dataset access failure with the specific reason.
8. IF VERIFY specifies a non-VSAM dataset, THEN THE Command_Executor SHALL return condition code 12 with message IDC0592E indicating VERIFY applies only to VSAM datasets.

---

### Requirement 12: EXPORT Command

**User Story:** As a mainframe developer, I want to export datasets to a portable format using the EXPORT command, so that I can create transportable copies for backup or transfer.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses an EXPORT command, THE IDCAMS_Parser SHALL extract the source entry name, output specification, and export options into an ExportCommand structure.
2. THE IDCAMS_Parser SHALL parse the entry name (1-44 characters) as the source dataset to export.
3. THE IDCAMS_Parser SHALL parse the mutually exclusive output parameters OUTFILE(ddname) and OUTDATASET(dsn) specifying the export destination. IF both are specified, THE IDCAMS_Parser SHALL return a parse error indicating only one may be used.
4. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters TEMPORARY and PERMANENT (default), where TEMPORARY indicates the export file is transient and PERMANENT indicates it is a lasting backup.
5. THE IDCAMS_Parser SHALL parse the mutually exclusive parameters INHIBITSOURCE and NOINHIBITSOURCE (default), where INHIBITSOURCE marks the source dataset as unavailable after export (for migration), and NOINHIBITSOURCE leaves the source accessible.
6. WHEN the Command_Executor processes an EXPORT command, THE Command_Executor SHALL invoke `CatalogService::export_dataset()` with the source DSN, destination, and export options.
7. IF EXPORT specifies a source entry that does not exist, THEN THE Command_Executor SHALL return condition code 12 with message IDC0600E indicating source not found.
8. IF EXPORT specifies an output destination that cannot be written, THEN THE Command_Executor SHALL return condition code 12 with message IDC0601E indicating output write failure.
9. WHEN EXPORT completes successfully, THE Command_Executor SHALL set LASTCC to 0 and emit message IDC0004I confirming the export with record count and byte count. IF export fails or does not complete successfully, THE Command_Executor SHALL set LASTCC to a non-zero condition code (8 or 12) appropriate to the failure.

---

### Requirement 13: IMPORT Command

**User Story:** As a mainframe developer, I want to import datasets from a portable export format using the IMPORT command, so that I can restore backups or receive transferred datasets.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses an IMPORT command, THE IDCAMS_Parser SHALL extract the input specification, target specification, and import options into an ImportCommand structure.
2. THE IDCAMS_Parser SHALL parse the mutually exclusive input parameters INFILE(ddname) and INDATASET(dsn) specifying the export file to import from.
3. THE IDCAMS_Parser SHALL parse the OUTDATASET(dsn) parameter specifying the target dataset name for the imported data.
4. THE IDCAMS_Parser SHALL parse the CATALOG(catalog_name) parameter specifying which catalog to register the imported dataset in.
5. THE IDCAMS_Parser SHALL parse the OBJECTS keyword with sub-parameters allowing renaming and attribute override during import: OBJECTS((old_name NEWNAME(new_name) VOLUMES(volser))).
6. WHEN the Command_Executor processes an IMPORT command, THE Command_Executor SHALL invoke `CatalogService::import_dataset()` with the input source, target DSN, catalog, and object mappings.
7. IF IMPORT specifies an input source that does not exist or is not a valid export file, THEN THE Command_Executor SHALL return condition code 12 with message IDC0610E indicating invalid import source.
8. IF IMPORT specifies a target DSN that already exists in the catalog, THEN THE Command_Executor SHALL return condition code 12 with message IDC0611E indicating the target already exists (use REPLACE option or DELETE first).
9. WHEN IMPORT completes successfully, THE Command_Executor SHALL set LASTCC to 0 and emit message IDC0005I confirming the import with record count.

---

### Requirement 14: BLDINDEX Command

**User Story:** As a mainframe developer, I want to build or rebuild alternate index entries using the BLDINDEX command, so that I can populate AIX structures after loading data into the base cluster.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses a BLDINDEX command, THE IDCAMS_Parser SHALL extract the input dataset, output dataset, and build options into a BldindexCommand structure.
2. THE IDCAMS_Parser SHALL parse the INDATASET(dsn) parameter as the base cluster whose records will be scanned to build the index.
3. THE IDCAMS_Parser SHALL parse the OUTDATASET(dsn) parameter as the alternate index to be populated.
4. THE IDCAMS_Parser SHALL parse the optional CATALOG(catalog_name) parameter.
5. WHEN the Command_Executor processes a BLDINDEX command, THE Command_Executor SHALL invoke `VsamService::build_index()` with the base cluster DSN and AIX DSN.
6. IF BLDINDEX specifies an INDATASET that does not exist, THEN THE Command_Executor SHALL return condition code 12 with message IDC0620E indicating base cluster not found.
7. IF BLDINDEX specifies an OUTDATASET that is not defined as an alternate index, THEN THE Command_Executor SHALL return condition code 12 with message IDC0621E indicating the output is not a valid AIX.
8. IF BLDINDEX encounters duplicate alternate keys when the AIX is defined with UNIQUEKEY, THEN THE Command_Executor SHALL return condition code 8 with message IDC0622W indicating duplicate keys were found and report the count. This warning SHALL only be emitted when the actual duplicate count is greater than zero.
9. WHEN BLDINDEX completes successfully, THE Command_Executor SHALL set LASTCC to 0 and emit message IDC0006I confirming the index build with the number of index entries created.

---

### Requirement 15: Return Code Management

**User Story:** As a mainframe developer, I want IDCAMS to track and propagate return codes (LASTCC, MAXCC) identically to z/OS IDCAMS, so that I can use conditional execution and rely on familiar return code semantics.

#### Acceptance Criteria

1. THE Command_Executor SHALL maintain two condition code registers: LASTCC (the return code of the most recently executed command) and MAXCC (the highest return code encountered across all commands in the current invocation).
2. WHEN a command completes execution, THE Command_Executor SHALL set LASTCC to the command's return code and update MAXCC to be the maximum of the current MAXCC and LASTCC. MAXCC SHALL never decrease during an invocation — once set to a high value, it remains at that value even if all subsequent commands succeed with code 0.
3. THE Command_Executor SHALL support these condition code values: 0 (successful completion), 4 (warning — operation completed with minor issues), 8 (error — operation failed but processing continues), 12 (severe error — the specific command failed), 16 (catastrophic error — processing should terminate).
4. WHEN the IDCAMS_Parser parses a SET MAXCC(n) command, THE Command_Executor SHALL set MAXCC to the specified value n (0-16).
5. WHEN the IDCAMS_Parser parses a SET LASTCC(n) command, THE Command_Executor SHALL set LASTCC to the specified value n (0-16).
6. WHEN the entire IDCAMS invocation completes (all commands processed), THE Command_Executor SHALL return MAXCC as the overall process return code.
7. IF a command returns condition code 16, THEN THE Command_Executor SHALL terminate processing of subsequent commands immediately, emit the final summary message (IDC0002I), and return 16 as the process return code.
8. THE Command_Executor SHALL emit a final summary message indicating MAXCC at the end of the invocation: IDC0002I `IDCAMS PROCESSING COMPLETE. MAXIMUM CONDITION CODE WAS n`.

---

### Requirement 16: Modal Commands (IF/THEN/ELSE)

**User Story:** As a mainframe developer, I want to conditionally execute IDCAMS commands based on return codes using IF/THEN/ELSE, so that I can build resilient multi-step IDCAMS scripts.

#### Acceptance Criteria

1. WHEN the IDCAMS_Parser parses an IF statement, THE IDCAMS_Parser SHALL extract the condition expression, the THEN command(s), and optionally the ELSE command(s) into a ModalCommand structure.
2. THE IDCAMS_Parser SHALL parse condition expressions testing LASTCC or MAXCC against a numeric value using comparison operators: EQ (equal), NE (not equal), GT (greater than), LT (less than), GE (greater than or equal), LE (less than or equal).
3. THE IDCAMS_Parser SHALL parse compound conditions using logical operators AND and OR, with parentheses for grouping.
4. THE IDCAMS_Parser SHALL parse the THEN clause containing one or more commands to execute when the condition is true.
5. THE IDCAMS_Parser SHALL parse the optional ELSE clause containing one or more commands to execute when the condition is false.
6. THE IDCAMS_Parser SHALL parse the DO/END block syntax for multiple commands within THEN or ELSE clauses: `THEN DO ... END` or `ELSE DO ... END`.
7. WHEN the Command_Executor evaluates an IF statement, THE Command_Executor SHALL evaluate the condition against the current LASTCC and MAXCC values, then execute either the THEN or ELSE clause.
8. WHEN the THEN or ELSE clause contains multiple commands (DO/END block), THE Command_Executor SHALL execute them sequentially, updating LASTCC and MAXCC after each.
9. IF an IF condition references an undefined register (neither LASTCC nor MAXCC), THEN THE IDCAMS_Parser SHALL return a parse error with message IDC0630E indicating invalid condition operand.
10. THE Command_Executor SHALL support nested IF statements — an IF within a THEN or ELSE clause.

---

### Requirement 17: SYSIN Processing and Command Input

**User Story:** As a mainframe developer, I want IDCAMS to read commands from SYSIN (or an equivalent input stream), so that I can drive IDCAMS through JCL, scripts, or programmatic API.

#### Acceptance Criteria

1. WHEN ff-idcams is invoked, THE Command_Executor SHALL read control statements from the configured input source (SYSIN stream, string buffer, or file).
2. THE Command_Executor SHALL process control statements sequentially, executing each parsed command in order from the input.
3. THE Command_Executor SHALL support three input modes: (a) SYSIN DD (text read from a DD allocation, used in JCL execution), (b) String buffer (commands passed as a string, used in scripting API), (c) File input (commands read from a file path, used in standalone execution).
4. WHEN reading from SYSIN DD, THE Command_Executor SHALL resolve the SYSIN DD name through the AllocatorService to locate the input stream. IF SYSIN or SYSPRINT DD access fails, THE Command_Executor SHALL fail immediately with condition code 16 rather than attempting alternative I/O. WHEN using string buffer or file input modes, DD resolution SHALL be skipped entirely.
5. THE Command_Executor SHALL strip sequence numbers from columns 73-80 if the input is in fixed 80-byte record format.
6. THE Command_Executor SHALL ignore blank lines in the input stream.
7. IF the input source is empty (no commands), THEN THE Command_Executor SHALL set MAXCC to 0 and emit message IDC0640I indicating no commands to process.

---

### Requirement 18: Command Chaining

**User Story:** As a mainframe developer, I want to execute multiple IDCAMS commands in a single invocation, so that I can batch related operations (define, load, verify) together.

#### Acceptance Criteria

1. THE Command_Executor SHALL process multiple commands from a single input stream, where commands are separated by: (a) newlines (each command starts on a new line after the previous command ends), (b) semicolons (`;` separating commands on the same line), or (c) explicit end of the previous command's parameters.
2. THE Command_Executor SHALL execute chained commands sequentially in input order.
3. WHEN a command in the chain sets LASTCC to 8 or 12, THE Command_Executor SHALL continue processing subsequent commands in the chain (unless LASTCC is 16 or a modal command directs otherwise).
4. THE Command_Executor SHALL maintain LASTCC and MAXCC across all commands in the chain — MAXCC accumulates the worst return code from any command in the chain.
5. WHEN all commands in the chain have been processed, THE Command_Executor SHALL return MAXCC as the invocation return code.
6. THE Command_Executor SHALL support at least 100 commands in a single chained invocation without degradation.

---

### Requirement 19: Error Handling and Message Format

**User Story:** As a mainframe developer, I want IDCAMS to produce error messages in the same IDCnnnX format as z/OS IDCAMS, so that I can use existing troubleshooting documentation and scripts that parse IDCAMS output.

#### Acceptance Criteria

1. THE Command_Executor SHALL format all messages using the pattern `IDCnnnnX text` where: `nnnn` is a 4-digit message number (0000-9999), and `X` is the severity indicator (I=Informational, W=Warning, E=Error, S=Severe).
2. THE Command_Executor SHALL emit informational messages (I) only for successful operations (condition code 0) — informational severity SHALL NOT be used for failed operations.
3. THE Command_Executor SHALL emit warning messages (W) for operations that completed with minor issues — these correspond to condition code 4.
4. THE Command_Executor SHALL emit error messages (E) for operations that failed — these correspond to condition code 8 or 12.
5. THE Command_Executor SHALL emit severe messages (S) for catastrophic failures — these correspond to condition code 16.
6. THE Command_Executor SHALL include contextual information in error messages: the command verb, the entry name (if applicable), and a description of the failure cause.
7. THE Command_Executor SHALL write all messages to an output stream (SYSPRINT equivalent) in the order they are generated during execution.
8. IF a downstream service (CatalogService, VsamService) returns an error, THEN THE Command_Executor SHALL map the downstream error to an appropriate IDC message code and include the downstream error detail in the message text.
9. THE Command_Executor SHALL prefix each command's output with the command text (echoed input) before emitting result messages, matching z/OS IDCAMS behaviour.
10. THE Command_Executor SHALL number output lines sequentially within each invocation for reference.

---

### Requirement 20: Integration Points and Invocation

**User Story:** As a mainframe developer, I want to invoke IDCAMS from JCL (EXEC PGM=IDCAMS), from the workbench command palette, and from the scripting API, so that IDCAMS is available in all execution contexts.

#### Acceptance Criteria

1. THE ff-idcams crate SHALL register with the FileForgeWorkbench command framework, exposing IDCAMS as an invocable program via `EXEC PGM=IDCAMS` in JCL execution.
2. THE ff-idcams crate SHALL expose a public Rust API: `fn execute_idcams(input: &str, services: &IdcamsServices) -> IdcamsResult` that accepts control statements as a string and returns structured results including output messages, LASTCC, and MAXCC.
3. THE ff-idcams crate SHALL expose a command palette integration allowing individual IDCAMS commands to be invoked interactively from the workbench UI (e.g., `idcams.define`, `idcams.listcat`, `idcams.delete`).
4. WHEN invoked via JCL (EXEC PGM=IDCAMS), THE Command_Executor SHALL read input from the SYSIN DD and write output to the SYSPRINT DD, using the AllocatorService to resolve these DDs.
5. WHEN invoked via the scripting API, THE Command_Executor SHALL accept input as a string parameter and return output as a structured result (no DD resolution needed). This applies regardless of whether the system is currently in a JCL execution context — the scripting API always uses string input/output.
6. THE ff-idcams crate SHALL accept its downstream service dependencies (CatalogService, VsamService, AllocatorService) through constructor injection via an `IdcamsServices` struct, enabling unit testing with mock implementations.
7. THE ff-idcams crate SHALL implement the workbench command handler trait to receive invocations from the command framework.


---

### Requirement 21: Ownership Boundary Enforcement

**User Story:** As a platform architect, I want ff-idcams to strictly enforce its ownership boundary (parsing and orchestration only), so that the crate remains a thin command interpreter and all actual operations are delegated to downstream services.

**Governance Reference:** Dataset Ownership Model — Requirement 6 (ff-idcams Ownership Boundary). [ADR-001]

#### Acceptance Criteria

1. THE ff-idcams crate SHALL NOT contain any SQLite import statements, database connection code, or direct catalog persistence logic — all catalog operations SHALL flow through the CatalogService trait.
2. THE ff-idcams crate SHALL NOT contain any VSAM record-level logic including key comparison, index maintenance, record insertion algorithms, B-tree operations, or sequential access implementation — all VSAM operations SHALL flow through the VsamService trait.
3. THE ff-idcams crate SHALL NOT directly access the filesystem for dataset content — all content access SHALL flow through ff-vfs or VsamService.
4. THE ff-idcams crate SHALL NOT contain any JCL parsing logic — DD statement resolution SHALL flow through the AllocatorService trait when SYSIN DD resolution is required.
5. THE ff-idcams crate's `Cargo.toml` SHALL NOT list `rusqlite`, `rocksdb`, `lmdb`, or any storage engine as a direct or transitive dependency — the entire dependency tree of ff-idcams SHALL be free of storage engine crates.
6. THE ff-idcams crate SHALL depend on ff-dataset-catalog, ff-vsam-services, and ff-dataset-allocator exclusively through trait interfaces defined in those crates (CatalogService, VsamService, AllocatorService).
7. THE ff-idcams crate SHALL validate command parameters at the syntax level (e.g., key length > 0, DSN within 44 characters) but SHALL delegate authoritative semantic validation to the owning downstream service.
8. THE ff-idcams crate MAY cache parsed command structures for performance but SHALL NOT cache catalog state, dataset metadata, or VSAM structural information.

---

### Requirement 22: Atomic Execution Guarantee

**User Story:** As a mainframe developer, I want every IDCAMS command to execute atomically (all-or-nothing), so that a failure in any downstream operation does not leave the system in an inconsistent state.

**Governance Reference:** Dataset Ownership Model — Requirement 6, Acceptance Criterion 4. [ADR-001]

#### Acceptance Criteria

1. WHEN the Command_Executor invokes multiple downstream services for a single command (e.g., DEFINE CLUSTER requires CatalogService::create_dataset + VsamService::initialize_dataset), THE Command_Executor SHALL execute them in a defined sequence and roll back completed operations if a subsequent operation fails.
2. IF CatalogService::create_dataset() succeeds but VsamService::initialize_dataset() fails during DEFINE CLUSTER, THEN THE Command_Executor SHALL attempt to invoke CatalogService::delete_dataset() to remove the partial catalog entry before returning the error. The system obligation is satisfied by attempting the rollback — if the rollback itself fails, it is handled by criterion 5.
3. IF VsamService::destroy_dataset() succeeds but CatalogService::delete_dataset() fails during DELETE, THEN THE Command_Executor SHALL log a warning message IDC0700W indicating potential inconsistency and return condition code 12 — the VSAM destruction cannot be rolled back but the inconsistency is reported.
4. THE Command_Executor SHALL implement a compensation pattern for rollback: each step in a multi-service command SHALL record its compensating action, and on failure, compensating actions SHALL be executed in reverse order.
5. IF a rollback (compensating action) itself fails, THEN THE Command_Executor SHALL emit message IDC0701S indicating a severe inconsistency requiring manual intervention and return condition code 16.
6. SINGLE-SERVICE commands (e.g., ALTER which only calls CatalogService::update_dataset) are inherently atomic — the downstream service owns the transactional semantics.

---

### Requirement 23: Non-Functional — Performance

**User Story:** As a mainframe developer working with large datasets, I want IDCAMS command parsing and execution orchestration to be efficient, so that batch operations complete within acceptable time bounds.

#### Acceptance Criteria

1. THE IDCAMS_Parser SHALL parse a single control statement (up to 1024 characters including continuations) within 1 millisecond on a modern desktop processor.
2. THE IDCAMS_Parser SHALL parse a batch of 1000 commands from a SYSIN stream within 500 milliseconds, excluding downstream execution time.
3. THE Command_Executor overhead (time spent in ff-idcams orchestration logic, excluding downstream service call time) SHALL be less than 5 milliseconds per command.
4. THE REPRO Command_Executor SHALL support streaming record copy without buffering the entire source dataset in memory — records SHALL be processed in a streaming fashion, one at a time or in bounded batches. IF streaming is unavailable (e.g., the downstream service does not support streaming), THE Command_Executor SHALL fall back to buffered processing with bounded batch sizes and emit a warning message.
5. THE LISTCAT Command_Executor SHALL support pagination or streaming output for catalogs containing more than 10,000 entries without loading all entries into memory simultaneously.
6. THE IDCAMS_Parser SHALL allocate less than 64 KB of heap memory for parsing a single command (excluding the input text itself).

---

### Requirement 24: Non-Functional — Thread Safety

**User Story:** As a workbench developer, I want ff-idcams to be safe for concurrent invocation, so that multiple JCL jobs or workbench commands can use IDCAMS simultaneously.

#### Acceptance Criteria

1. THE ff-idcams crate SHALL be safe to invoke concurrently from multiple threads — the IDCAMS_Parser SHALL be stateless and the Command_Executor SHALL hold no global mutable state.
2. EACH IDCAMS invocation (call to `execute_idcams`) SHALL maintain its own LASTCC and MAXCC registers, output buffer, and execution context — there SHALL be no shared mutable state between concurrent invocations.
3. THE ff-idcams crate's public API types SHALL implement `Send + Sync` where appropriate, enabling safe sharing across threads.
4. THE ff-idcams crate SHALL NOT use global mutable statics (`static mut`, lazy_static with interior mutability, or equivalent) for any operational state.
5. CONCURRENT IDCAMS invocations targeting the same dataset SHALL rely on the downstream services (CatalogService, VsamService) for serialization and conflict detection — ff-idcams SHALL NOT implement its own locking for dataset access.

---

### Requirement 25: Non-Functional — Testability

**User Story:** As a developer working on ff-idcams, I want the crate to be fully testable with mock downstream services, so that I can validate parsing and orchestration logic without requiring a real catalog, VSAM engine, or filesystem.

#### Acceptance Criteria

1. THE ff-idcams crate SHALL accept all downstream service dependencies through trait objects (CatalogService, VsamService, AllocatorService), enabling injection of mock implementations in tests.
2. THE IDCAMS_Parser SHALL be independently testable: given an input string, it SHALL produce a deterministic AST or error without requiring any service dependencies.
3. THE Command_Executor SHALL be testable with mock service implementations that return configurable success or error responses, enabling validation of orchestration logic, rollback behaviour, and error handling.
4. THE ff-idcams crate SHALL expose its parsed command types (DefineClusterCommand, DeleteCommand, etc.) as public types, enabling external crates to construct commands programmatically for testing.
5. THE ff-idcams crate SHALL provide a test helper module or builder pattern for constructing `IdcamsServices` with mock implementations, reducing test boilerplate.
6. EVERY acceptance criterion in this specification SHALL be testable through the public API of ff-idcams with mock downstream services — no criterion shall require a real database, filesystem, or VSAM engine to validate.

---

### Requirement 26: Pretty Printer

**User Story:** As a mainframe developer, I want ff-idcams to format parsed commands back into valid IDCAMS control statements, so that tools can round-trip commands (parse → modify → emit) and produce readable output.

#### Acceptance Criteria

1. THE ff-idcams crate SHALL provide a Pretty_Printer module that formats any parsed command AST back into a valid IDCAMS control statement string.
2. THE Pretty_Printer SHALL produce output that, when re-parsed by the IDCAMS_Parser, yields an equivalent AST (round-trip property: parse → print → parse produces same structure).
3. THE Pretty_Printer SHALL format commands with consistent indentation: parameters indented under the command verb, sub-parameters indented under their parent parameter.
4. THE Pretty_Printer SHALL insert continuation characters (hyphen at end of line) when a command exceeds 72 characters per line.
5. THE Pretty_Printer SHALL preserve parameter ordering consistent with z/OS IDCAMS conventions (NAME first, then type-specific parameters, then common parameters).
6. THE Pretty_Printer SHALL support a compact mode (minimal whitespace, single line where possible) and a verbose mode (one parameter per line for readability).
7. FOR ALL valid command ASTs, THE Pretty_Printer SHALL produce syntactically valid IDCAMS control statements — the output SHALL always be parseable without error.

