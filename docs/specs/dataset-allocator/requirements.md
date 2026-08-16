# Requirements Document

> **Governance Reference:** This specification is governed by the [Dataset Ownership Model](./../dataset-ownership-model/requirements.md) (ADR-001). Where this document conflicts with the governance document, the governance document takes precedence. The dataset allocator owns JCL-driven allocation workflows (DD parsing, DISP processing, symbolic substitution, referback resolution) and delegates all catalog operations to ff-dataset-catalog via the `CatalogService` trait interface.

## Introduction

This feature specifies the **Dataset Allocator** for FileForgeWorkbench — the `ff-dataset-allocator` crate. The dataset allocator is responsible for parsing JCL DD statements, resolving dataset names (DSN) against mounted catalog repositories, performing symbolic parameter substitution, allocating datasets from JCL specifications, handling GDG relative generation references, and providing a RESOLVE command for tracing DSN-to-physical-path mappings.

The dataset allocator bridges the JCL language constructs that mainframe developers use to reference data (DD statements with DSN=, DISP=, DCB=, SPACE= operands) with the local desktop dataset catalog emulation. It enables developers to write and test JCL locally by resolving dataset references against the workbench's mounted catalogs — without requiring a z/OS system. It is the desktop equivalent of z/OS Dynamic Allocation (DYNALLOC / SVC 99).

The `ff-dataset-allocator` crate is a Wave 13 (Dataset Catalog and Mainframe Emulation) component. It depends on:

- `ff-dataset-catalog` — for DSN resolution, catalog mount state, dataset allocation, and GDG management
- `ff-vfs` (virtual-file-system) — for provider-agnostic resource access (FFW-ARCH-001)
- `ff-command` (command-framework) — for RESOLVE command registration and dispatch
- `ff-language-service` — for JCL language definition (keyword sets, statement structure)
- `ff-config` (configuration-system) — for resolver settings (default HLQ, symbol tables, resolution preferences)
- `ff-logging` — for structured diagnostics

**Source references:**
- **[DSC]** = Dataset Catalog Brief §9 — JCL Integration, resolve_dsn API
- **[WB]** = Workbench Architecture Brief — FFW-ARCH-001, command-driven architecture

---

## Glossary

- **DD_Statement**: A JCL Data Definition statement that associates a logical file name (ddname) with a physical dataset, inline data, or output destination. Begins with `//ddname DD` on a JCL statement image. [DSC]
- **DSN (Data Set Name)**: A one-to-44-character mainframe dataset name composed of one or more qualifiers separated by dots. Each qualifier is 1–8 alphanumeric characters starting with a letter or national character (@, #, $). [DSC]
- **DISP (Disposition)**: A DD operand specifying the dataset status (NEW, OLD, SHR, MOD) and conditional disposition (KEEP, DELETE, CATLG, UNCATLG, PASS) at step start, normal end, and abnormal end. [DSC]
- **DCB (Data Control Block)**: A DD operand specifying dataset attributes: RECFM (record format), LRECL (logical record length), BLKSIZE (block size), DSORG (dataset organisation). [DSC]
- **SPACE**: A DD operand specifying allocation size for new datasets in tracks, cylinders, or block units, with primary, secondary, and directory quantities. [DSC]
- **Symbolic_Parameter**: A JCL symbolic variable prefixed with `&` (e.g., `&SYSPARM`, `&DATE`, `&USERID`) that is resolved by substitution before DD statement processing. [DSC]
- **System_Symbol**: A predefined symbolic parameter provided by the resolver environment (e.g., `&SYSDATE`, `&SYSTIME`, `&SYSJOBNAME`). [DSC]
- **GDG (Generation Data Group)**: A collection of chronologically related datasets sharing a common base name, with relative generation references (+1 = next new, 0 = current, -1 = previous). [DSC]
- **PDS_Member**: A named member within a Partitioned Data Set, referenced as `DSN=pds.name(member)`. [DSC]
- **Temporary_Dataset**: A dataset with a system-generated name using `&&` prefix (e.g., `&&TEMP`), existing only for the duration of the job. [DSC]
- **PROC (Procedure)**: A catalogued or in-stream set of JCL statements invoked by an EXEC statement, with overridable DD statements and symbolic parameters. [DSC]
- **Concatenation**: Multiple DD statements sharing the same ddname (continuation without a ddname on subsequent DDs), logically joining multiple datasets for sequential reading. [DSC]
- **SYSOUT**: A DD operand directing output to a spool class rather than a dataset (e.g., `SYSOUT=A`). [DSC]
- **Referback**: A DSN reference that refers to a DD in a previous step using `*.stepname.ddname` or `*.stepname.procstepname.ddname` syntax. [DSC]
- **RESOLVE_Command**: A workbench command (`dataset.resolve`) registered with the command framework that traces the resolution path from a DSN reference to its physical storage location. [WB]
- **Resolution_Result**: The output of resolving a DD statement's DSN: the physical file path, the catalog it was found in, the dataset type, and any applied substitutions. [DSC]
- **JCL_Job**: A complete JCL job stream starting with a JOB statement, containing one or more EXEC steps, each with DD statements. [DSC]
- **Step**: A unit of execution within a JCL job, introduced by an `EXEC` statement (PGM= or PROC=). [DSC]
- **Resolution_Panel**: A workbench output panel that displays the results of JCL resolution — resolved paths, warnings, and errors for each DD statement. [WB]
- **Lint_Diagnostic**: A warning or error produced by the JCL validation pass indicating an unresolved DSN, missing DD, invalid symbolic, or other JCL problem. [DSC]
- **Symbol_Table**: A collection of symbolic parameter definitions (name → value mappings) available for substitution during resolution. [DSC]

---

## Requirements

### Requirement 1: DD Statement Parsing

**User Story:** As a mainframe developer, I want the resolver to parse JCL DD statements including all common operands, so that I can write JCL locally and have dataset references extracted for resolution.

**Source:** [DSC] §9 JCL Integration — parser flow from DSN to catalog lookup. [DSC]

#### Acceptance Criteria

1. THE dataset allocator SHALL parse DD statements in the standard format `//ddname DD operand1,operand2,...` extracting the ddname (columns 3–10) and all operands from the operand field. [DSC]
2. THE parser SHALL extract the DSN operand value from a DD statement, supporting both unquoted names (`DSN=MY.DATA.SET`) and quoted names (`DSN='MY.DATA.SET'`). [DSC]
3. THE parser SHALL extract member references from DSN values in the format `DSN=pds.name(membername)`, separating the PDS base name from the member name. [DSC]
4. THE parser SHALL extract the DISP operand, parsing up to three positional sub-parameters: status (NEW, OLD, SHR, MOD), normal disposition (KEEP, DELETE, CATLG, UNCATLG, PASS), and abnormal disposition (KEEP, DELETE, CATLG, UNCATLG). [DSC]
5. THE parser SHALL extract the DCB operand, parsing sub-parameters RECFM, LRECL, BLKSIZE, and DSORG as key=value pairs within the DCB parenthesised list. [DSC]
6. THE parser SHALL extract the SPACE operand, parsing the allocation unit (TRK, CYL, or blocksize integer), primary quantity, secondary quantity, and directory blocks from the positional format `SPACE=(unit,(primary,secondary,directory))`. [DSC]
7. THE parser SHALL handle JCL continuation lines — when column 72 contains a non-blank character and the next record begins with `// ` (slashes followed by spaces and then operands), the parser SHALL join the continuation into the current statement. [DSC]
8. THE parser SHALL recognise SYSOUT DD statements (`SYSOUT=class`) and mark them as output-directed (no DSN resolution required). [DSC]
9. THE parser SHALL recognise DD statements with `*` or `DATA` keyword as inline data (DD *), marking them as inline (no DSN resolution required). [DSC]
10. THE parser SHALL recognise the DUMMY keyword, marking the DD as a null dataset (no allocation or resolution required). [DSC]
11. WHEN a DD statement contains syntax errors (unbalanced parentheses, invalid operand format), THE parser SHALL produce a Lint_Diagnostic at severity ERROR identifying the statement line number, the ddname, and a description of the syntax problem. [DSC]

---

### Requirement 2: Dataset Name Resolution Against Mounted Catalogs

**User Story:** As a mainframe developer, I want DSN references in my JCL resolved against locally mounted catalogs, so that I can verify dataset existence and trace the physical file path without a z/OS system.

**Source:** [DSC] §9 — resolve_dsn API, catalog mount state. Cross-references: `dataset-catalog` Requirement 1 (SQLite catalog DB), Requirement 3 (dataset resolution). [DSC]

#### Acceptance Criteria

1. WHEN a DD statement has DISP=(OLD,...) or DISP=(SHR,...), THE resolver SHALL look up the DSN in all mounted catalogs and return a Resolution_Result containing the physical file path, catalog name, and dataset type (PS, PO, GDG). [DSC]
2. WHEN a DSN is found in exactly one mounted catalog, THE resolver SHALL return a successful Resolution_Result with the resolved physical path and the catalog identifier. [DSC]
3. WHEN a DSN is found in multiple mounted catalogs, THE resolver SHALL use the catalog search order: IF `jcl.catalog_search_order` is explicitly configured, use that order; OTHERWISE, fall back to the `ff-dataset-catalog` default ordering (most recently mounted has highest priority). THE resolver SHALL emit a WARN-level diagnostic noting the ambiguity. [DSC]
4. WHEN a DSN with DISP=(OLD,...) or DISP=(SHR,...) is not found in any mounted catalog, THE resolver SHALL produce a Lint_Diagnostic at severity ERROR with message "Dataset not found: {dsn}" and the line number of the DD statement. [DSC]
5. WHEN a DSN references a PDS member (`DSN=pds.name(member)`), THE resolver SHALL verify both that the PDS base exists in a mounted catalog AND that the specified member exists within the PDS directory. [DSC]
6. WHEN a PDS member reference specifies a PDS that exists but a member that does not, THE resolver SHALL produce a Lint_Diagnostic at severity WARNING with message "Member not found: {member} in {pds_dsn}". [DSC]
7. THE resolver SHALL support wildcard-free resolution only — DSN values must be fully qualified; pattern-based resolution (e.g., `HLQ.*.DATA`) is not supported and SHALL produce a Lint_Diagnostic at severity ERROR. [DSC]
8. ALL DSN resolution SHALL flow through the `ff-dataset-catalog` crate's resolution API (specifically the `CatalogService` trait interface), not directly against the filesystem, honouring the VFS abstraction (FFW-ARCH-001). The allocator SHALL NOT contain `use rusqlite` or any direct catalog database access. [WB]

---

### Requirement 3: Symbolic Parameter Substitution

**User Story:** As a mainframe developer, I want symbolic parameters in my JCL (both system symbols and user-defined symbols) substituted before DSN resolution, so that parameterised JCL procedures resolve correctly against my local catalogs.

**Source:** [DSC] §9 — symbolic substitution pass before catalog lookup. Cross-references: `configuration-system` (symbol table persistence). [DSC]

#### Acceptance Criteria

1. THE resolver SHALL perform symbolic substitution on all DD statement operands before DSN resolution, replacing all occurrences of `&symbol` or `&symbol.` with the corresponding value from the active Symbol_Table. [DSC]
2. THE resolver SHALL support system symbols with predefined values: `&SYSDATE` (current date YYMMDD), `&SYSDATE4` (current date YYYYMMDD), `&SYSTIME` (current time HHMMSS), `&SYSJOBNAME` (job name from JOB statement), `&SYSSTEP` (current step name), and `&SYSUID` (current user ID from configuration). [DSC]
3. THE resolver SHALL support user-defined symbols declared on JCL SET statements (`// SET symbol=value`) and PROC statement KEYWORD parameters, adding them to the active Symbol_Table for the scope of the job or procedure. [DSC]
4. THE resolver SHALL support EXEC statement overrides of PROC symbolic parameters (`//step EXEC proc,symbol=value`), with override values taking precedence over PROC-declared defaults. [DSC]
5. WHEN a symbolic parameter `&symbol` is referenced but has no value in the active Symbol_Table (neither system-defined nor user-defined), THE resolver SHALL produce a Lint_Diagnostic at severity ERROR with message "Unresolved symbolic: &{symbol}" and the line number. [DSC]
6. THE resolver SHALL handle the dot-terminator convention: `&SYM.REST` substitutes the value of `&SYM` followed by literal `REST`, where the dot is consumed as a terminator and not included in the result. [DSC]
7. THE resolver SHALL handle double-ampersand (`&&`) as a literal ampersand in contexts other than temporary dataset names, and SHALL NOT attempt symbolic substitution on `&&`-prefixed temporary dataset names. [DSC]
8. THE resolver SHALL support substring notation `&symbol(start,length)` for extracting a portion of a symbolic value during substitution. [DSC]
9. THE resolver SHALL process symbolic substitution in a single left-to-right pass; nested symbolics (a symbol whose value contains another `&symbol` reference) SHALL NOT be recursively resolved unless a second explicit substitution pass is configured. [DSC]
10. USER-DEFINED symbols SHALL be configurable via the configuration-system under the `jcl.symbols` table (e.g., `[jcl.symbols]` in TOML), allowing persistent symbol definitions that apply to all resolution operations. [WB]

---

### Requirement 4: DISP Parameter Interpretation and Allocation Simulation

**User Story:** As a mainframe developer, I want the resolver to interpret DISP parameters and simulate dataset allocation (creating datasets for DISP=NEW, verifying existence for DISP=OLD/SHR), so that I can validate my JCL allocation logic locally.

**Source:** [DSC] §9 — dataset allocation from JCL. Cross-references: `dataset-catalog` (dataset allocation API). [DSC]

#### Acceptance Criteria

1. WHEN a DD statement has `DISP=(NEW,CATLG)` or `DISP=(NEW,KEEP)`, THE resolver SHALL simulate dataset allocation by invoking the `ff-dataset-catalog` allocation API with the DSN and attributes extracted from DCB and SPACE operands. [DSC]
2. WHEN allocating a new dataset (DISP=NEW), THE resolver SHALL extract dataset attributes from the DCB operand (RECFM, LRECL, BLKSIZE, DSORG) and pass them to the catalog allocation API; IF DCB is not specified, THE resolver SHALL use the allocation defaults defined by `ff-dataset-catalog` (under `[catalog.defaults]` in configuration); IF those are also not configured, THE resolver SHALL fall back to: RECFM=FB, LRECL=80, BLKSIZE=27920. [DSC]
3. WHEN a DD statement has `DISP=(NEW,...)` and the DSN already exists in a mounted catalog, THE resolver SHALL produce a Lint_Diagnostic at severity ERROR with message "Dataset already exists: {dsn} (DISP=NEW requires non-existent dataset)". [DSC]
4. WHEN a DD statement has `DISP=(OLD,...)`, THE resolver SHALL verify that the DSN exists in a mounted catalog; IF the dataset does not exist, THE resolver SHALL produce a Lint_Diagnostic at severity ERROR with message "Dataset not found: {dsn} (DISP=OLD requires existing dataset)". [DSC]
5. WHEN a DD statement has `DISP=(SHR,...)`, THE resolver SHALL verify that the DSN exists in a mounted catalog (same as OLD); the distinction between OLD and SHR is informational (exclusive vs. shared access) and does not affect resolution logic. [DSC]
6. WHEN a DD statement has `DISP=(MOD,...)`, THE resolver SHALL verify that the DSN exists in a mounted catalog for append access; IF the dataset does not exist AND a SPACE operand is provided, THE resolver SHALL treat MOD as equivalent to NEW (create the dataset). [DSC]
7. WHEN a DD statement has no DISP operand, THE resolver SHALL apply the default disposition `DISP=(NEW,DELETE)` per z/OS JCL conventions. [DSC]
8. THE resolver SHALL support the PASS disposition (`DISP=(OLD,PASS)`) by recording the passed dataset in a job-scoped pass table, making it available for referback by subsequent steps without requiring catalog lookup. [DSC]
9. WHEN allocation simulation is configured as dry-run mode (`jcl.resolve_mode = "dry-run"` in configuration), THE resolver SHALL report what allocations would occur without actually creating datasets in the catalog. [WB]
10. WHEN allocation simulation is configured as live mode (`jcl.resolve_mode = "live"`), THE resolver SHALL perform actual catalog allocations for DISP=NEW datasets, creating entries in the mounted catalog. [WB]

---

### Requirement 5: Concatenation DD Support

**User Story:** As a mainframe developer, I want the resolver to handle concatenated DD statements (multiple datasets under a single ddname), so that my JCL concatenation logic is validated and all component datasets are resolved.

**Source:** [DSC] §9 — concatenation handling during resolution. [DSC]

#### Acceptance Criteria

1. THE parser SHALL detect concatenated DD statements: when a DD statement has no ddname (columns 3–10 are blank) and follows another DD statement, it SHALL be treated as a continuation of the preceding ddname's concatenation group. [DSC]
2. THE resolver SHALL resolve each dataset in a concatenation group independently, producing a separate Resolution_Result for each component dataset. [DSC]
3. THE Resolution_Result for a concatenated DD group SHALL include the concatenation order (1-based index) for each component, preserving the sequential read order. [DSC]
4. WHEN any component dataset in a concatenation group cannot be resolved (not found in catalogs), THE resolver SHALL produce a Lint_Diagnostic at severity ERROR identifying both the ddname and the concatenation index of the failing component. [DSC]
5. THE resolver SHALL validate that all datasets in a concatenation group have compatible attributes (matching RECFM and compatible LRECL) and SHALL produce a Lint_Diagnostic at severity WARNING when attribute mismatches are detected. [DSC]
6. THE resolver SHALL support a maximum of 255 concatenated datasets per ddname, consistent with z/OS JCL limits; exceeding this limit SHALL produce a Lint_Diagnostic at severity ERROR. [DSC]

---

### Requirement 6: Temporary Dataset Handling

**User Story:** As a mainframe developer, I want the resolver to handle temporary dataset references (`&&name`), so that inter-step data passing via temporary datasets is validated without requiring permanent catalog entries.

**Source:** [DSC] §9 — temporary dataset lifecycle within a job. [DSC]

#### Acceptance Criteria

1. THE parser SHALL recognise temporary dataset names (DSN values prefixed with `&&`) and mark them as temporary in the parsed DD model. [DSC]
2. WHEN a DD statement creates a temporary dataset (`DISP=(NEW,...), DSN=&&name`), THE resolver SHALL register the temporary name in a job-scoped temporary dataset table, recording the creating step name and DD attributes. [DSC]
3. WHEN a DD statement references a temporary dataset (`DISP=(OLD,...), DSN=&&name` or `DISP=(SHR,...), DSN=&&name`), THE resolver SHALL look up the name in the job-scoped temporary dataset table and return a Resolution_Result indicating a temporary dataset with the creating step reference. [DSC]
4. WHEN a temporary dataset is referenced but was not created by a prior step in the same job, THE resolver SHALL produce a Lint_Diagnostic at severity ERROR with message "Temporary dataset not created in prior step: &&{name}". [DSC]
5. TEMPORARY datasets SHALL NOT be resolved against mounted catalogs — they exist only within the job-scoped temporary table. [DSC]
6. WHEN a temporary dataset is created without an explicit DSN (`DD` with no DSN and `DISP=(NEW,PASS)`), THE resolver SHALL assign a system-generated temporary name (format `&&SYSnnnnn`) and register it in the temporary table. [DSC]
7. THE resolver SHALL track temporary dataset lifecycle: a temporary with `DISP=(,DELETE)` at the creating step's normal-end disposition SHALL be marked as deleted and SHALL NOT be resolvable by subsequent steps. [DSC]

---

### Requirement 7: Referback Resolution

**User Story:** As a mainframe developer, I want the resolver to handle referback DSN references (`*.stepname.ddname`), so that my JCL can reference datasets defined in prior steps without repeating the full DSN.

**Source:** [DSC] §9 — referback resolution across steps. [DSC]

#### Acceptance Criteria

1. THE parser SHALL recognise referback DSN syntax: `*.stepname.ddname` (referencing a DD in a prior step) and `*.stepname.procstepname.ddname` (referencing a DD in a procedure step). [DSC]
2. WHEN resolving a referback `*.stepname.ddname`, THE resolver SHALL locate the DD statement with the specified ddname in the specified prior step and use that DD's resolved DSN as the effective DSN for the referback. [DSC]
3. WHEN resolving a referback `*.stepname.procstepname.ddname`, THE resolver SHALL locate the DD statement within the specified procedure step execution in the specified step, and use that DD's resolved DSN. [DSC]
4. WHEN a referback references a step name that does not exist in the current job, THE resolver SHALL produce a Lint_Diagnostic at severity ERROR with message "Referback target step not found: {stepname}". [DSC]
5. WHEN a referback references a ddname that does not exist in the target step, THE resolver SHALL produce a Lint_Diagnostic at severity ERROR with message "Referback target DD not found: {ddname} in step {stepname}". [DSC]
6. WHEN a referback targets a DD that is itself a referback, THE resolver SHALL follow the chain recursively (up to a configurable depth limit of 10) to find the ultimate DSN; IF the chain exceeds the depth limit, THE resolver SHALL produce a Lint_Diagnostic at severity ERROR with message "Referback chain too deep (limit: 10)". [DSC]
7. REFERBACK resolution SHALL occur after symbolic substitution but before catalog lookup — the resolved DSN from the target DD (after its own substitution) is used for the referback's catalog resolution. [DSC]

---

### Requirement 8: GDG Relative Generation Resolution

**User Story:** As a mainframe developer, I want the resolver to handle GDG relative generation references (+1, 0, -1), so that my JCL referencing generation data groups resolves to the correct generation in my local catalog.

**Source:** [DSC] §9 — GDG generation resolution. Cross-references: `dataset-catalog` (GDG management, gdg_generations table). [DSC]

#### Acceptance Criteria

1. THE parser SHALL recognise GDG relative generation syntax in DSN values: `BASE.NAME(+n)`, `BASE.NAME(0)`, and `BASE.NAME(-n)` where n is a positive integer. [DSC]
2. WHEN a DSN references generation `(0)` (current generation), THE resolver SHALL query the `ff-dataset-catalog` GDG API for the most recent active generation of the specified base name and return its physical path in the Resolution_Result. [DSC]
3. WHEN a DSN references generation `(-n)` (previous generation), THE resolver SHALL query the catalog for the nth-most-recent active generation before the current one; IF fewer than n active generations exist, THE resolver SHALL produce a Lint_Diagnostic at severity ERROR with message "GDG generation not available: {base}(-{n}) — only {available} active generations exist". [DSC]
4. WHEN a DSN references generation `(+1)` (next new generation) with `DISP=(NEW,CATLG)`, THE resolver SHALL simulate creation of a new generation by computing the next generation number from the catalog's GDG state, and SHALL include the projected generation name in the Resolution_Result. [DSC]
5. WHEN a DSN references generation `(+n)` with n > 1, THE resolver SHALL produce a Lint_Diagnostic at severity WARNING with message "Multiple forward GDG generations (+{n}) in a single step may indicate a JCL error — only (+1) is typical". [DSC]
6. WHEN a GDG base name referenced in a DSN does not exist as a registered GDG in any mounted catalog, THE resolver SHALL produce a Lint_Diagnostic at severity ERROR with message "GDG base not defined: {base_name}". [DSC]
7. THE resolver SHALL track GDG generation allocations within a job: if step 1 creates `BASE(+1)` and step 2 references `BASE(0)`, the resolver SHALL recognise that the generation created in step 1 is now the current generation for step 2's resolution. [DSC]
8. WHEN GDG generation resolution results in a roll-off (the new generation would exceed the GDG limit), THE resolver SHALL emit a Lint_Diagnostic at severity INFO with message "GDG roll-off: creating {base}(+1) will roll off generation {oldest_gen}". [DSC]

---

### Requirement 9: RESOLVE Primary Command

**User Story:** As a mainframe developer, I want a RESOLVE command that I can invoke interactively (from the command line or a keyboard shortcut) to trace how a DSN in my JCL resolves to a physical path, so that I can debug resolution problems and verify my catalog configuration.

**Source:** [WB] command-driven architecture. Cross-references: `command-framework` (command registration, dispatch, metadata). [WB]

#### Acceptance Criteria

1. THE dataset allocator SHALL register a command with ID `dataset.resolve` in the Command_Registry during crate initialization, with metadata including display name "Resolve Dataset Allocation", category "dataset", and a default keyboard shortcut. [WB]
2. WHEN `jcl.resolve` is invoked with no parameters on a JCL file, THE command SHALL resolve all DD statements in the active document and display results in the Resolution_Panel. [WB]
3. WHEN `jcl.resolve` is invoked with cursor position in a JCL file, THE command SHALL resolve only the DD statement at or nearest to the cursor position and display the single result in the Resolution_Panel. [WB]
4. WHEN `jcl.resolve` is invoked with a DSN string parameter (`jcl.resolve dsn="MY.DATA.SET"`), THE command SHALL resolve the specified DSN against mounted catalogs without requiring a JCL document context. [WB]
5. THE command SHALL accept an optional `mode` parameter with values `"dry-run"` (default — report only) or `"live"` (perform actual allocations for DISP=NEW), overriding the configuration-system setting for that invocation. [WB]
6. THE command SHALL return a Command_Result containing the resolution outcome: success count, warning count, error count, and the list of Resolution_Results for all processed DD statements. [WB]
7. THE command SHALL complete within 5 seconds for a JCL file containing up to 500 DD statements, under normal catalog sizes (up to 10,000 datasets per catalog). [WB]
8. WHEN the active document is not a JCL file (language_id is not `"jcl"`), THE command SHALL return an error result with message "Active document is not a JCL file" and SHALL NOT attempt resolution. [WB]

---

### Requirement 10: JCL Validation and Lint Diagnostics

**User Story:** As a mainframe developer, I want the resolver to produce validation diagnostics (warnings and errors) for common JCL problems — unresolved DSNs, missing DDs, invalid operands — so that I can catch JCL errors before attempting submission to a z/OS system.

**Source:** [DSC] §9 — validation pass. Cross-references: `language-service` (JCL language definition for structural validation). [DSC]

#### Acceptance Criteria

1. THE resolver SHALL produce Lint_Diagnostics at defined severity levels: ERROR (resolution failure, prevents successful execution), WARNING (potential problem that may or may not cause failure), and INFO (informational observation). [DSC]
2. THE resolver SHALL detect and report unresolved DSN references: any DD statement with a DSN that cannot be resolved against mounted catalogs (after substitution) SHALL produce an ERROR diagnostic. [DSC]
3. THE resolver SHALL detect and report unresolved symbolic parameters: any `&symbol` remaining after substitution SHALL produce an ERROR diagnostic identifying the unresolved symbol. [DSC]
4. THE resolver SHALL detect and report missing required DD statements: WHEN the JCL references well-known system DDs (SYSIN, SYSPRINT, SYSUT1, SYSUT2, SYSLIB) and they are not defined in the step, THE resolver SHALL produce a WARNING diagnostic. [DSC]
5. THE resolver SHALL detect and report duplicate ddnames within a single step (excluding intentional concatenation) as an ERROR diagnostic. [DSC]
6. THE resolver SHALL detect and report DISP conflicts: a DD with `DISP=(NEW,...)` referencing an existing dataset, or `DISP=(OLD,...)` referencing a non-existent dataset. [DSC]
7. THE resolver SHALL detect and report invalid DSN syntax: dataset names exceeding 44 characters, qualifiers exceeding 8 characters, qualifiers starting with a digit, or empty qualifiers (consecutive dots). [DSC]
8. THE resolver SHALL detect and report invalid symbolic parameter names: symbolics that contain characters other than alphanumeric and national characters (@, #, $). [DSC]
9. ALL Lint_Diagnostics SHALL include: severity level, line number in the JCL source, column range (start, end), diagnostic code (e.g., `JCL001`), and a human-readable message. [DSC]
10. THE resolver SHALL support a configurable diagnostic severity filter (`jcl.lint_level` in configuration) allowing users to suppress INFO or WARNING diagnostics. [WB]

---

### Requirement 11: Resolution Output Panel

**User Story:** As a mainframe developer, I want resolution results displayed in a dedicated output panel showing each DD statement's resolution status, resolved path, and any diagnostics, so that I can quickly see which datasets resolved successfully and which have problems.

**Source:** [WB] workbench panel architecture. Cross-references: `layout-and-docking` (panel registration), `command-framework` (command output display). [WB]

#### Acceptance Criteria

1. THE resolver SHALL display resolution results in a Resolution_Panel — a dockable output panel registered with the layout-and-docking system under panel ID `"jcl.resolution"`. [WB]
2. THE Resolution_Panel SHALL display a table with columns: Step Name, DD Name, DSN (after substitution), Status (Resolved/Error/Warning/Skipped), Physical Path (or error message), and Catalog Name. [WB]
3. WHEN a DD statement resolves successfully, THE Resolution_Panel SHALL show status "Resolved" with the physical file path and the catalog that provided the resolution. [WB]
4. WHEN a DD statement fails resolution, THE Resolution_Panel SHALL show status "Error" with the diagnostic message in the Physical Path column, highlighted in the error colour from the active theme. [WB]
5. THE Resolution_Panel SHALL display a summary header showing total DD statements processed, resolved count, warning count, and error count. [WB]
6. WHEN the user double-clicks a row in the Resolution_Panel, THE workbench SHALL navigate to the corresponding DD statement line in the JCL source document. [WB]
7. THE Resolution_Panel SHALL support sorting by any column (step name, DD name, status) and filtering by status (show only errors, show only warnings). [WB]
8. THE Resolution_Panel content SHALL persist until the next RESOLVE command is invoked or the panel is explicitly cleared. [WB]
9. WHEN resolution includes symbolic substitution, THE Resolution_Panel SHALL display both the original DSN (with symbols) and the substituted DSN in a tooltip or expandable detail row. [WB]
10. THE Resolution_Panel SHALL display concatenation groups as expandable parent rows, with each concatenated dataset as a child row showing its individual resolution status. [WB]

---

### Requirement 12: Job Structure Parsing

**User Story:** As a mainframe developer, I want the resolver to understand the overall JCL job structure (JOB, EXEC, DD hierarchy), so that step-scoped resolution, referbacks, and temporary dataset tracking work correctly across multi-step jobs.

**Source:** [DSC] §9 — job structure model for resolution context. [DSC]

#### Acceptance Criteria

1. THE parser SHALL recognise JOB statements (`//jobname JOB ...`) and extract the job name for use as the `&SYSJOBNAME` system symbol. [DSC]
2. THE parser SHALL recognise EXEC statements (`//stepname EXEC PGM=program` or `//stepname EXEC proc`) and extract the step name, program name or procedure name, and any symbolic overrides. [DSC]
3. THE parser SHALL build a hierarchical job model: Job → Steps → DD statements, preserving step ordering for referback and temporary dataset resolution. [DSC]
4. THE parser SHALL recognise PROC and PEND statements delineating in-stream procedures, and SHALL expand procedure invocations by merging procedure DD statements with EXEC-level overrides. [DSC]
5. WHEN an EXEC statement invokes a procedure and includes DD overrides (`//step.procstep DD ...`), THE parser SHALL apply the override to the corresponding DD in the procedure expansion, replacing or augmenting the procedure's original DD. [DSC]
6. THE parser SHALL recognise IF/THEN/ELSE/ENDIF conditional execution constructs and include conditionally-executed steps in the job model (resolution assumes all paths are taken for validation completeness). [DSC]
7. THE resolver SHALL process steps in declaration order (top to bottom), maintaining a cumulative state of: resolved DSNs per step, temporary dataset table, passed dataset table, and GDG generation state. [DSC]
8. WHEN a JCL file contains no JOB statement (a procedure library member or JCL fragment), THE resolver SHALL process it as a single anonymous job context with a default job name of `"NOJOB"`. [DSC]

---

### Requirement 13: Resolution Processing Pipeline

**User Story:** As a mainframe developer, I want the resolver to process JCL through a well-defined pipeline (parse → substitute → resolve → validate), so that resolution is predictable, traceable, and each stage's output can be inspected for debugging.

**Source:** [DSC] §9 — resolution pipeline stages. [WB]

#### Acceptance Criteria

1. THE resolver SHALL process JCL through the following ordered pipeline stages: (1) Parse — extract job structure and DD operands, (2) Substitute — replace symbolic parameters, (3) Resolve — look up DSNs in catalogs and handle referbacks/temporaries/GDGs, (4) Validate — produce lint diagnostics for problems detected. [DSC]
2. EACH pipeline stage SHALL produce intermediate results that are available for inspection: the parse stage produces a structured job model, the substitute stage produces substituted operand values, the resolve stage produces Resolution_Results, and the validate stage produces Lint_Diagnostics. [DSC]
3. THE resolver SHALL continue processing subsequent DD statements after encountering an error in one DD statement — errors in one DD SHALL NOT prevent resolution of other DDs in the same job. [DSC]
4. THE resolver SHALL aggregate all Lint_Diagnostics from all pipeline stages into a single ordered list, sorted by line number, for presentation in the Resolution_Panel. [DSC]
5. THE resolver SHALL emit structured log records at DEBUG level for each pipeline stage transition, including timing information (milliseconds spent in each stage). [WB]
6. THE resolver SHALL support incremental resolution: WHEN only a single DD statement has changed in an already-parsed job, THE resolver SHALL re-resolve only the affected DD and its dependents (DDs that referback to it) rather than re-resolving the entire job. [WB]

---

### Requirement 14: Configuration and Defaults

**User Story:** As a mainframe developer, I want resolver behaviour configurable through the workbench configuration system (default HLQ, symbol tables, resolution mode, lint severity), so that I can tailor resolution to match my project's conventions.

**Source:** [WB] configuration-driven architecture. Cross-references: `configuration-system` (TOML config, hot-reload). [WB]

#### Acceptance Criteria

1. THE resolver SHALL read configuration from the `[jcl]` table in the workbench configuration, supporting the following keys: `jcl.resolve_mode` (string: "dry-run" or "live"), `jcl.default_hlq` (string), `jcl.catalog_search_order` (array of catalog names), `jcl.lint_level` (string: "error", "warning", "info"), `jcl.max_referback_depth` (integer, default 10). [WB]
2. THE resolver SHALL read persistent user-defined symbols from the `[jcl.symbols]` configuration table, where each key-value pair defines a symbol name (without the `&` prefix) and its substitution value. [WB]
3. THE resolver SHALL read default dataset attributes from the `ff-dataset-catalog` configuration (`[catalog.defaults]`); these are the authoritative source for RECFM, LRECL, BLKSIZE, and DSORG defaults. The allocator SHALL NOT define its own duplicate defaults. All default retrieval SHALL flow through `CatalogService::get_allocation_defaults()` — the allocator SHALL NOT parse configuration files or access `ff-config` directly for allocation defaults. [WB]
4. WHEN configuration values change via hot-reload, THE resolver SHALL pick up the new values for subsequent resolution operations without requiring application restart. [WB]
5. THE resolver SHALL register its configuration schema with the Configuration_System during initialization, declaring all supported keys with their types, defaults, and descriptions. [WB]
6. THE `jcl.default_hlq` setting SHALL be used as a prefix for unqualified dataset names: WHEN a DSN in JCL has fewer than two qualifiers, THE resolver SHALL prepend the default HLQ before catalog lookup. [DSC]
7. THE `jcl.catalog_search_order` setting SHALL define an explicit override for catalog search priority; WHEN configured, it takes precedence over the `ff-dataset-catalog` default mount-order priority. IF not configured, THE resolver SHALL search all mounted catalogs in their mount order (as defined by `ff-dataset-catalog` Requirement 5.3). [DSC]

---

### Requirement 15: Error Handling and Diagnostics

**User Story:** As a mainframe developer, I want clear, actionable error messages when resolution fails, so that I can quickly identify and fix JCL problems or catalog configuration issues.

**Source:** [WB] error handling principles. Cross-references: `logging-subsystem` (structured logging). [WB, DSC]

#### Acceptance Criteria

1. ALL resolver errors SHALL use the `thiserror` crate and be defined in a `JclResolverError` enum with variants carrying sufficient context to diagnose the problem (line number, ddname, DSN, catalog name, reason). [WB]
2. THE resolver SHALL emit structured log records at appropriate levels: ERROR for resolution failures that prevent a DD from resolving, WARN for ambiguous or potentially problematic resolutions, INFO for successful resolution summary, DEBUG for pipeline stage details and intermediate results. [WB]
3. WHEN a catalog query fails due to a database error (SQLite I/O error, corrupt catalog), THE resolver SHALL produce a Lint_Diagnostic at severity ERROR with message "Catalog query failed: {catalog_name} — {error_detail}" and SHALL continue resolving remaining DDs against other available catalogs. [DSC]
4. WHEN the resolver encounters an internal error (programming error, unexpected state), THE resolver SHALL log the error at ERROR level with full context and return a graceful error result rather than panicking. [WB]
5. THE resolver SHALL provide a `resolve_result.diagnostics()` method returning all Lint_Diagnostics produced during resolution, ordered by line number, for consumption by the Resolution_Panel and any external tooling. [DSC]
6. THE resolver SHALL assign unique diagnostic codes to each class of problem (e.g., JCL001 = syntax error, JCL002 = unresolved DSN, JCL003 = unresolved symbolic, JCL004 = DISP conflict, JCL005 = referback target not found, JCL006 = GDG not found, JCL007 = concatenation error, JCL008 = invalid DSN syntax). [DSC]

---

### Requirement 16: Integration with Language Service

**User Story:** As a workbench developer, I want the dataset allocator to leverage the language service's JCL language definition for keyword recognition and statement structure validation, so that parsing is consistent with the syntax highlighting and the allocator can be triggered contextually when editing JCL files.

**Source:** [WB] language service integration. Cross-references: `language-service` (JCL language_id, keyword sets). [WB, DSC]

#### Acceptance Criteria

1. THE resolver SHALL query the `ff-language-service` to confirm that the active document's language_id is `"jcl"` before initiating resolution. [WB]
2. THE resolver SHALL use the JCL language definition's keyword sets to validate JCL statement types (JOB, EXEC, DD, PROC, PEND, SET, IF, ELSE, ENDIF) during parsing, rather than maintaining its own separate keyword list. [WB]
3. THE resolver SHALL support automatic resolution triggering: WHEN configured (`jcl.auto_resolve = true`), THE resolver SHALL perform a lightweight resolution pass (parse + substitute, no catalog queries) on document save to detect obvious symbolic and syntax errors. [WB]
4. THE resolver SHALL expose a programmatic API (`resolve_document(text: &str, config: &ResolverConfig) -> ResolveOutput`) that the language service or other subsystems can invoke independently of the command framework. [WB]
5. THE resolver SHALL provide hover information: WHEN the language service requests hover data for a DSN token in a JCL file, THE resolver SHALL return the resolution status, physical path (if resolved), and dataset attributes for display in an editor tooltip. [WB]

---

## Cross-Cutting Concerns

### Performance

- Resolution of a typical JCL job (10–50 steps, 100–500 DD statements) SHALL complete within 5 seconds under normal conditions (catalogs with up to 10,000 datasets each). [WB]
- The parser SHALL process JCL at a rate of at least 10,000 lines per second on a modern desktop CPU. [WB]
- Catalog lookups SHALL be batched where possible to minimize SQLite round-trips. [DSC]

### Thread Safety

- The resolver's public API SHALL be safe to invoke from any thread (Send + Sync). [WB]
- Resolution state (temporary tables, pass tables, GDG state) is scoped to a single resolution invocation and need not be shared across threads. [WB]

### Testability

- The resolver SHALL be testable without mounted catalogs by accepting a trait-based catalog interface that can be mocked in unit tests. [WB]
- The parser SHALL be independently testable with JCL text input, without requiring catalog or VFS infrastructure. [WB]

### Accessibility

- The Resolution_Panel SHALL support keyboard navigation (arrow keys to move between rows, Enter to activate a row). [WB]
- All diagnostic messages SHALL be plain text suitable for screen reader announcement. [WB]
