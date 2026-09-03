# Implementation Plan: IDCAMS Emulator (`ff-idcams`)

## Overview

This task plan implements the `ff-idcams` crate — the IDCAMS (Access Method Services) command interpreter and orchestration layer for FileForgeWorkbench. The crate is a thin command parser and executor that delegates all actual catalog, VSAM, allocation, and filesystem operations to downstream services through trait interfaces.

**Crate location:** `crates/ff-idcams`
**Upstream dependencies:** `ff-dataset-catalog` (CatalogService trait), `ff-vsam-services` (VsamService trait), `ff-dataset-allocator` (AllocatorService trait), `ff-vfs` (content I/O), `ff-command` (workbench command registration), `ff-logging` (diagnostics)
**Downstream consumers:** JCL execution engine (EXEC PGM=IDCAMS), workbench command palette, scripting API

---

## Tasks

- [x] 1. Project scaffold, error types, and service injection
  - [x] 1.1 Create `crates/ff-idcams/Cargo.toml` with dependencies (thiserror, tracing) and dev-dependencies (proptest, pretty_assertions, tokio-test); NO rusqlite, rocksdb, or storage engine deps
  - [x] 1.2 Create `crates/ff-idcams/src/lib.rs` with crate-level doc comment and public module declarations
  - [x] 1.3 Implement `src/error.rs` — define `IdcamsError` enum with variants: ParseError, ExecutionError, RollbackFailed, ServiceUnavailable, InputError
  - [x] 1.4 Implement `src/services.rs` — define `IdcamsServices` struct holding `Arc<dyn CatalogService>`, `Arc<dyn VsamService>`, `Arc<dyn AllocatorService>`; implement constructor and test helper builder
    - Validates: Requirement 20 AC 6; Requirement 25 AC 1, AC 5
  - [x] 1.5 Implement `src/messages.rs` — define `MessageCode` enum with all IDC message codes, `Severity` enum (I, W, E, S), `IdcamsMessage` struct with code, severity, text, line_number
    - Validates: Requirement 19 AC 1
  - [x] 1.6 Implement `ConditionCode` enum (0, 4, 8, 12, 16) and `IdcamsResult` struct (lastcc, maxcc, messages)
    - Validates: Requirement 15 AC 3
  - [x] 1.7 Write unit tests for error Display formatting, message code string conversion, ConditionCode ordering
    - Validates: Requirement 19 AC 1; Requirement 15 AC 3

- [x] 2. Lexer and token types
  - [x] 2.1 Implement `src/parser/token.rs` — define `Token` enum (Verb, Keyword, OpenParen, CloseParen, Number, StringLit, Semicolon, Hyphen, Comment, Wildcard, CompareOp, LogicalOp, Eof), `Verb` enum, `CmpOp` enum, `LogOp` enum
  - [x] 2.2 Implement `src/parser/lexer.rs` — define `Lexer` struct with `tokenize(input: &str) -> Result<Vec<Token>, ParseError>` method
  - [x] 2.3 Implement case-insensitive verb and keyword recognition — DEFINE, DELETE, ALTER, LISTCAT, PRINT, REPRO, VERIFY, EXPORT, IMPORT, BLDINDEX, SET, IF all case-insensitive
    - Validates: Requirement 1 AC 7
  - [x] 2.4 Implement parenthesised parameter tokenisation — track nesting depth, support nested parentheses
    - Validates: Requirement 1 AC 2
  - [x] 2.5 Implement continuation line handling — line ending with hyphen joins to next line, handling whitespace
    - Validates: Requirement 1 AC 3
  - [x] 2.6 Implement semicolon command separator
    - Validates: Requirement 1 AC 4
  - [x] 2.7 Implement comment handling — `/* ... */` block comments and `//` single-line comments
    - Validates: Requirement 1 AC 5, AC 6
  - [x] 2.8 Implement dataset name tokenisation — 1-44 characters with qualifier rules
    - Validates: Requirement 1 AC 8
  - [x] 2.9 Write unit tests for lexer: all token types, continuation lines, comments, semicolons, case insensitivity, DSN parsing
    - Validates: Requirement 1 AC 1–8

- [x] 3. AST definitions and parser framework
  - [x] 3.1 Implement `src/parser/ast.rs` — define `Command` enum with all variants (DefineCluster, DefineAix, DefinePath, DefineGdg, Delete, Alter, Listcat, Print, Repro, Verify, Export, Import, Bldindex, Set, If, Error), plus all command structs
    - Validates: Requirement 1 AC 11
  - [x] 3.2 Implement `src/parser/error.rs` — define `ParseError` struct with message code (IDC0001E, IDC0002E), position, and description
    - Validates: Requirement 1 AC 9, AC 10
  - [x] 3.3 Implement `src/parser/mod.rs` — define `Parser` struct with `parse(tokens: &[Token]) -> Vec<Command>` method using recursive descent; produce Error AST nodes on failure rather than aborting
    - Validates: Requirement 1 AC 11
  - [x] 3.4 Implement unrecognised verb error — produce Error node with IDC0001E
    - Validates: Requirement 1 AC 9
  - [x] 3.5 Implement malformed parameter error — produce Error node with IDC0002E for unbalanced parens
    - Validates: Requirement 1 AC 10
  - [x] 3.6 Write unit tests for AST construction, error node production, and parser framework
    - Validates: Requirement 1 AC 9–11
  - [x] 3.7 Write property test: round-trip (Property 1) — for all valid commands, parse → pretty-print → re-parse produces equivalent AST
    - Validates: Requirement 1 AC 12; Requirement 26 AC 2

- [x] 4. DEFINE CLUSTER parser
  - [x] 4.1 Implement DEFINE CLUSTER parsing — extract NAME, organization (INDEXED/NONINDEXED/NUMBERED/LINEAR), all parameters into DefineClusterCommand
    - Validates: Requirement 2 AC 1
  - [x] 4.2 Implement NAME parameter parsing (1-44 char DSN)
    - Validates: Requirement 2 AC 6
  - [x] 4.3 Implement VOLUMES parameter parsing (one or more 1-6 char volume serials)
    - Validates: Requirement 2 AC 7
  - [x] 4.4 Implement space allocation parsing — CYLINDERS, TRACKS, RECORDS, KILOBYTES (primary, secondary)
    - Validates: Requirement 2 AC 8
  - [x] 4.5 Implement RECORDSIZE(average maximum) parsing
    - Validates: Requirement 2 AC 9
  - [x] 4.6 Implement KEYS(length offset) parsing — length 1-255, offset 0-based
    - Validates: Requirement 2 AC 10
  - [x] 4.7 Implement FREESPACE(ci_percent ca_percent) parsing — 0-100 each
    - Validates: Requirement 2 AC 11
  - [x] 4.8 Implement SHAREOPTIONS(crossregion crosssystem) parsing — 1-4 each
    - Validates: Requirement 2 AC 12
  - [x] 4.9 Implement SPEED/RECOVERY, REUSE, CONTROLINTERVALSIZE, BUFFERSPACE parsing
    - Validates: Requirement 2 AC 13, AC 14, AC 16, AC 17
  - [x] 4.10 Implement DATA and INDEX component sub-definition parsing
    - Validates: Requirement 2 AC 15
  - [x] 4.11 Write unit tests for DEFINE CLUSTER parsing — all organisation types, all parameters, component definitions
    - Validates: Requirement 2 AC 1–17

- [x] 5. DEFINE AIX, PATH, and GDG parsers
  - [x] 5.1 Implement DEFINE ALTERNATEINDEX parsing — NAME, RELATE, KEYS, UNIQUEKEY/NONUNIQUEKEY, UPGRADE/NOUPGRADE, RECORDSIZE
    - Validates: Requirement 3 AC 1–6
  - [x] 5.2 Implement DEFINE PATH parsing — NAME, PATHENTRY, UPDATE/NOUPDATE
    - Validates: Requirement 4 AC 1–4
  - [x] 5.3 Implement DEFINE GDG parsing — NAME, LIMIT, SCRATCH/NOSCRATCH, EMPTY/NOEMPTY, FIFO/LIFO
    - Validates: Requirement 5 AC 1–6
  - [x] 5.4 Write unit tests for DEFINE AIX, PATH, GDG parsing with all parameter combinations
    - Validates: Requirement 3 AC 1–6; Requirement 4 AC 1–4; Requirement 5 AC 1–6

- [x] 6. DELETE, ALTER, and LISTCAT parsers
  - [x] 6.1 Implement DELETE parsing — entry names (list), entry type keyword, PURGE/NOPURGE, FORCE/NOFORCE, ERASE/NOERASE, SCRATCH/NOSCRATCH
    - Validates: Requirement 6 AC 1–7
  - [x] 6.2 Implement ALTER parsing — entry name, all alterable attributes (FREESPACE, SHAREOPTIONS, BUFFERSPACE, RECORDSIZE, KEYS, ADDVOLUMES, REMOVEVOLUMES, NEWNAME, NULLIFY)
    - Validates: Requirement 7 AC 1–5
  - [x] 6.3 Implement LISTCAT parsing — ENTRIES/LEVEL filter (mutually exclusive), display level (NAME/HISTORY/VOLUME/ALL), CATALOG, entry type filter, wildcard in ENTRIES
    - Validates: Requirement 8 AC 1–6, AC 14
  - [x] 6.4 Implement ENTRIES/LEVEL mutual exclusion check — parse error if both specified
    - Validates: Requirement 8 AC 3
  - [x] 6.5 Write unit tests for DELETE, ALTER, LISTCAT parsing with all parameter permutations
    - Validates: Requirement 6 AC 1–7; Requirement 7 AC 1–5; Requirement 8 AC 1–6, AC 14

- [x] 7. PRINT, REPRO, VERIFY, EXPORT, IMPORT, BLDINDEX parsers
  - [x] 7.1 Implement PRINT parsing — INFILE/INDATASET, CHARACTER/HEX/DUMP format, FROMKEY/TOKEY, FROMADDRESS/TOADDRESS, FROMRECORD/TORECORD, COUNT, SKIP
    - Validates: Requirement 9 AC 1–8
  - [x] 7.2 Implement REPRO parsing — INFILE/INDATASET, OUTFILE/OUTDATASET, key/address selection, COUNT, SKIP, REPLACE/NOREPLACE
    - Validates: Requirement 10 AC 1–7
  - [x] 7.3 Implement VERIFY parsing — FILE/DATASET (mutually exclusive)
    - Validates: Requirement 11 AC 1–2
  - [x] 7.4 Implement EXPORT parsing — entry name, OUTFILE/OUTDATASET (mutually exclusive), TEMPORARY/PERMANENT, INHIBITSOURCE/NOINHIBITSOURCE
    - Validates: Requirement 12 AC 1–5
  - [x] 7.5 Implement mutual exclusion checks for EXPORT OUTFILE/OUTDATASET
    - Validates: Requirement 12 AC 3
  - [x] 7.6 Implement IMPORT parsing — INFILE/INDATASET, OUTDATASET, CATALOG, OBJECTS with NEWNAME/VOLUMES
    - Validates: Requirement 13 AC 1–5
  - [x] 7.7 Implement BLDINDEX parsing — INDATASET, OUTDATASET, CATALOG
    - Validates: Requirement 14 AC 1–4
  - [x] 7.8 Write unit tests for PRINT, REPRO, VERIFY, EXPORT, IMPORT, BLDINDEX parsing
    - Validates: Requirements 9–14 (parser aspects)

- [x] 8. SET and IF/THEN/ELSE parsers (modal commands)
  - [x] 8.1 Implement SET parsing — SET MAXCC(n) and SET LASTCC(n) where n is 0-16
    - Validates: Requirement 15 AC 4, AC 5
  - [x] 8.2 Implement IF parsing — condition expression (LASTCC/MAXCC, comparison operator, numeric value)
    - Validates: Requirement 16 AC 1, AC 2
  - [x] 8.3 Implement compound condition parsing — AND, OR operators with parentheses for grouping
    - Validates: Requirement 16 AC 3
  - [x] 8.4 Implement THEN clause parsing — single command or DO/END block with multiple commands
    - Validates: Requirement 16 AC 4, AC 6
  - [x] 8.5 Implement ELSE clause parsing (optional) — single command or DO/END block
    - Validates: Requirement 16 AC 5, AC 6
  - [x] 8.6 Implement nested IF detection — IF within THEN or ELSE clause
    - Validates: Requirement 16 AC 10
  - [x] 8.7 Implement invalid condition operand error — IDC0630E for non-LASTCC/MAXCC registers
    - Validates: Requirement 16 AC 9
  - [x] 8.8 Write unit tests for SET parsing, IF/THEN/ELSE with compound conditions, DO/END blocks, nested IFs, error on invalid operand
    - Validates: Requirement 15 AC 4, AC 5; Requirement 16 AC 1–10

- [x] 9. Execution context and return code management
  - [x] 9.1 Implement `src/executor/context.rs` — define `ExecutionState` struct with lastcc, maxcc, messages vec, line_counter; implement `set_lastcc(cc)` that also updates maxcc
    - Validates: Requirement 15 AC 1, AC 2
  - [x] 9.2 Implement MAXCC monotonicity — maxcc never decreases; update only when new cc > current maxcc
    - Validates: Requirement 15 AC 2
  - [x] 9.3 Implement CC=16 termination — when a command sets LASTCC to 16, stop processing subsequent commands immediately
    - Validates: Requirement 15 AC 7
  - [x] 9.4 Implement SET MAXCC/LASTCC execution — directly set the register to the specified value
    - Validates: Requirement 15 AC 4, AC 5
  - [x] 9.5 Implement final summary message emission — IDC0002I with MAXCC value at end of invocation
    - Validates: Requirement 15 AC 8
  - [x] 9.6 Implement invocation return — return MAXCC as overall process return code
    - Validates: Requirement 15 AC 6
  - [x] 9.7 Write unit tests for return code management: monotonicity, CC=16 termination, SET override, final summary
    - Validates: Requirement 15 AC 1–8
  - [x] 9.8 Write property test: MAXCC monotonicity (Property 2) — execute random command sequences, assert maxcc never decreases across commands
    - Validates: Requirement 15 AC 2

- [x] 10. Command executor dispatch and command chaining
  - [x] 10.1 Implement `src/executor/mod.rs` — define `CommandExecutor` struct with `execute(commands: Vec<Command>, services: &IdcamsServices) -> IdcamsResult`
  - [x] 10.2 Implement sequential command execution — process commands in input order, updating LASTCC/MAXCC after each
    - Validates: Requirement 18 AC 2
  - [x] 10.3 Implement continuation after error — when LASTCC is 8 or 12, continue to next command (unless 16)
    - Validates: Requirement 18 AC 3
  - [x] 10.4 Implement IF/THEN/ELSE evaluation — evaluate condition against current LASTCC/MAXCC, execute appropriate branch
    - Validates: Requirement 16 AC 7, AC 8
  - [x] 10.5 Implement DO/END block sequential execution within THEN/ELSE clauses
    - Validates: Requirement 16 AC 8
  - [x] 10.6 Implement command echo — prefix each command's output with the command text
    - Validates: Requirement 19 AC 9
  - [x] 10.7 Implement output line numbering — sequential within invocation
    - Validates: Requirement 19 AC 10
  - [x] 10.8 Write unit tests for command dispatch, chaining, continuation after error, IF evaluation, DO/END blocks
    - Validates: Requirement 18 AC 1–6; Requirement 16 AC 7–10

- [x] 11. Compensation/rollback pattern
  - [x] 11.1 Implement `src/executor/rollback.rs` — define `CompensatingAction` enum and `execute_with_rollback` function
    - Validates: Requirement 22 AC 4
  - [x] 11.2 Implement compensating action recording — each successful step records its reverse operation
    - Validates: Requirement 22 AC 1, AC 4
  - [x] 11.3 Implement reverse-order rollback execution on failure
    - Validates: Requirement 22 AC 4
  - [x] 11.4 Implement rollback failure handling — emit IDC0701S and return CC=16 when compensation itself fails
    - Validates: Requirement 22 AC 5
  - [x] 11.5 Implement partial inconsistency warning — emit IDC0700W when VSAM destroy succeeds but catalog delete fails
    - Validates: Requirement 22 AC 3
  - [x] 11.6 Write unit tests for rollback: success path (no rollback needed), failure with successful rollback, failure with failed rollback (IDC0701S), partial inconsistency (IDC0700W)
    - Validates: Requirement 22 AC 1–6

- [x] 12. DEFINE CLUSTER executor
  - [x] 12.1 Implement `src/executor/define.rs` — DEFINE CLUSTER handler: invoke CatalogService::create_dataset then VsamService::initialize_dataset
    - Validates: Requirement 2 AC 2–5
  - [x] 12.2 Implement INDEXED dispatch — CatalogService DSORG=KSDS + VsamService VsamType::Ksds
    - Validates: Requirement 2 AC 2
  - [x] 12.3 Implement NONINDEXED dispatch — CatalogService DSORG=ESDS + VsamService VsamType::Esds
    - Validates: Requirement 2 AC 3
  - [x] 12.4 Implement NUMBERED dispatch — CatalogService DSORG=RRDS + VsamService VsamType::Rrds
    - Validates: Requirement 2 AC 4
  - [x] 12.5 Implement LINEAR dispatch — CatalogService DSORG=LDS + VsamService VsamType::Lds
    - Validates: Requirement 2 AC 5
  - [x] 12.6 Implement KEYS validation — INDEXED without KEYS produces CC=12, IDC0503E
    - Validates: Requirement 2 AC 18
  - [x] 12.7 Implement duplicate name detection — existing DSN produces CC=12, IDC0514E
    - Validates: Requirement 2 AC 19
  - [x] 12.8 Implement atomic rollback — if VsamService fails after CatalogService succeeds, invoke CatalogService::delete_dataset
    - Validates: Requirement 2 AC 20
  - [x] 12.9 Implement success path — set LASTCC=0, emit IDC0001I
    - Validates: Requirement 2 AC 21
  - [x] 12.10 Write unit tests with mock services: all org types, missing KEYS error, duplicate error, rollback on VSAM failure, success message
    - Validates: Requirement 2 AC 2–21

- [x] 13. DEFINE AIX, PATH, and GDG executors
  - [x] 13.1 Implement DEFINE ALTERNATEINDEX executor — invoke VsamService::define_aix with parsed params
    - Validates: Requirement 3 AC 7
  - [x] 13.2 Implement RELATE validation — base cluster not found produces CC=12, IDC0510E
    - Validates: Requirement 3 AC 8
  - [x] 13.3 Implement RELATE type validation — non-VSAM base produces CC=12, IDC0511E
    - Validates: Requirement 3 AC 9
  - [x] 13.4 Implement DEFINE AIX success — LASTCC=0, IDC0001I only on VsamService success
    - Validates: Requirement 3 AC 10
  - [x] 13.5 Implement DEFINE PATH executor — invoke VsamService::define_path
    - Validates: Requirement 4 AC 5
  - [x] 13.6 Implement PATHENTRY validation — AIX not found produces CC=12, IDC0512E (validated before execution)
    - Validates: Requirement 4 AC 6
  - [x] 13.7 Implement DEFINE PATH success — LASTCC=0, IDC0001I
    - Validates: Requirement 4 AC 7
  - [x] 13.8 Implement DEFINE GDG executor — invoke CatalogService::create_gdg_base with limit, scratch, empty, ordering
    - Validates: Requirement 5 AC 7
  - [x] 13.9 Implement LIMIT missing validation — CC=12, IDC0520E
    - Validates: Requirement 5 AC 8
  - [x] 13.10 Implement DEFINE GDG duplicate name check — CC=12, IDC0514E
    - Validates: Requirement 5 AC 9
  - [x] 13.11 Implement DEFINE GDG success — verify downstream success before emitting IDC0001I
    - Validates: Requirement 5 AC 10
  - [x] 13.12 Write unit tests for DEFINE AIX/PATH/GDG executors: success paths, validation failures, and success-only emission
    - Validates: Requirement 3 AC 7–10; Requirement 4 AC 5–7; Requirement 5 AC 7–10

- [x] 14. DELETE executor
  - [x] 14.1 Implement `src/executor/delete.rs` — DELETE handler dispatching by entry type
  - [x] 14.2 Implement DELETE CLUSTER — VsamService::destroy_dataset then CatalogService::delete_dataset
    - Validates: Requirement 6 AC 8
  - [x] 14.3 Implement DELETE ALTERNATEINDEX — VsamService::destroy_dataset then CatalogService::delete_dataset
    - Validates: Requirement 6 AC 9
  - [x] 14.4 Implement DELETE PATH — VsamService::delete_path then CatalogService::delete_dataset
    - Validates: Requirement 6 AC 10
  - [x] 14.5 Implement DELETE GDG — CatalogService::delete_gdg_base with force option
    - Validates: Requirement 6 AC 11
  - [x] 14.6 Implement DELETE NONVSAM — CatalogService::delete_dataset
    - Validates: Requirement 6 AC 12
  - [x] 14.7 Implement entry not found — CC=8, IDC0550E
    - Validates: Requirement 6 AC 13
  - [x] 14.8 Implement type mismatch — CC=12, IDC0551E
    - Validates: Requirement 6 AC 14
  - [x] 14.9 Implement atomic deletion — if VSAM destroy fails, do not proceed with catalog delete (CC=12)
    - Validates: Requirement 6 AC 15
  - [x] 14.10 Implement success message — IDC0002I per entry
    - Validates: Requirement 6 AC 16
  - [x] 14.11 Implement multi-entry sequential processing — process each name, continue regardless of individual failures, track MAXCC
    - Validates: Requirement 6 AC 17
  - [x] 14.12 Write unit tests for DELETE: all types, not-found, type mismatch, atomic failure, multi-entry processing
    - Validates: Requirement 6 AC 8–17

- [x] 15. ALTER executor
  - [x] 15.1 Implement `src/executor/alter.rs` — ALTER handler invoking CatalogService::update_dataset
    - Validates: Requirement 7 AC 6
  - [x] 15.2 Implement NEWNAME handling — invoke CatalogService::rename_dataset
    - Validates: Requirement 7 AC 7
  - [x] 15.3 Implement entry not found — CC=8, IDC0560E
    - Validates: Requirement 7 AC 8
  - [x] 15.4 Implement non-modifiable attribute error — CC=12, IDC0561E
    - Validates: Requirement 7 AC 9
  - [x] 15.5 Implement ALTER success — LASTCC=0, IDC0003I
    - Validates: Requirement 7 AC 10
  - [x] 15.6 Write unit tests for ALTER: attribute update, rename, not-found, non-modifiable, success
    - Validates: Requirement 7 AC 6–10

- [x] 16. LISTCAT executor
  - [x] 16.1 Implement `src/executor/listcat.rs` — LISTCAT handler invoking CatalogService::list_datasets and get_dataset_attributes
    - Validates: Requirement 8 AC 7, AC 8
  - [x] 16.2 Implement wildcard matching — `*` matches zero or more characters within ENTRIES filter
    - Validates: Requirement 8 AC 14
  - [x] 16.3 Implement display level NAME — output names with type indication only
    - Validates: Requirement 8 AC 11
  - [x] 16.4 Implement display level ALL — output hierarchical format with cluster/data/index/AIX/PATH structure, attributes, statistics, allocation info
    - Validates: Requirement 8 AC 9, AC 10
  - [x] 16.5 Implement no-entries-found warning — CC=4, IDC0565W
    - Validates: Requirement 8 AC 12
  - [x] 16.6 Implement LISTCAT success — LASTCC=0
    - Validates: Requirement 8 AC 13
  - [x] 16.7 Write unit tests for LISTCAT: NAME/ALL display, wildcard matching, LEVEL filter, no results warning, type filtering
    - Validates: Requirement 8 AC 7–14

- [x] 17. PRINT executor
  - [x] 17.1 Implement `src/executor/print.rs` — PRINT handler dispatching to VSAM browse or VFS read
  - [x] 17.2 Implement VSAM dataset print — VsamService::open + start_browse + next_record loop
    - Validates: Requirement 9 AC 9
  - [x] 17.3 Implement non-VSAM sequential print — ff-vfs read operations
    - Validates: Requirement 9 AC 10
  - [x] 17.4 Implement CHARACTER format — printable chars with non-printable as periods
    - Validates: Requirement 9 AC 11
  - [x] 17.5 Implement HEX format — hexadecimal representation
    - Validates: Requirement 9 AC 11
  - [x] 17.6 Implement DUMP format — offset + hex groups + character interpretation
    - Validates: Requirement 9 AC 12
  - [x] 17.7 Implement dataset not found — CC=12, IDC0570E
    - Validates: Requirement 9 AC 13
  - [x] 17.8 Implement FROMKEY on non-KSDS error — CC=12, IDC0571E
    - Validates: Requirement 9 AC 14
  - [x] 17.9 Implement PRINT success — LASTCC=0, summary with record count
    - Validates: Requirement 9 AC 15
  - [x] 17.10 Write unit tests for PRINT: all formats, VSAM and non-VSAM paths, key validation, not-found, COUNT/SKIP
    - Validates: Requirement 9 AC 1–15

- [x] 18. REPRO executor
  - [x] 18.1 Implement `src/executor/repro.rs` — REPRO handler with four copy paths (VSAM→VSAM, SEQ→VSAM, VSAM→SEQ, SEQ→SEQ)
  - [x] 18.2 Implement VSAM-to-VSAM copy — browse source, put to target
    - Validates: Requirement 10 AC 8
  - [x] 18.3 Implement SEQ-to-VSAM copy — VFS read source, VsamService put to target
    - Validates: Requirement 10 AC 9
  - [x] 18.4 Implement VSAM-to-SEQ copy — browse source, VFS write to target
    - Validates: Requirement 10 AC 10
  - [x] 18.5 Implement SEQ-to-SEQ copy — VFS read source, VFS write target
    - Validates: Requirement 10 AC 11
  - [x] 18.6 Implement duplicate key handling — NOREPLACE skips (CC=4, IDC0580W), REPLACE overwrites
    - Validates: Requirement 10 AC 12, AC 13
  - [x] 18.7 Implement source/target not found — CC=12, IDC0581E / IDC0582E
    - Validates: Requirement 10 AC 14, AC 15
  - [x] 18.8 Implement completion summary — record count copied and skipped
    - Validates: Requirement 10 AC 16
  - [x] 18.9 Implement streaming processing — one record at a time, report failure point
    - Validates: Requirement 10 AC 17
  - [x] 18.10 Write unit tests for REPRO: all four paths, duplicate handling, not-found errors, streaming behaviour
    - Validates: Requirement 10 AC 8–17

- [x] 19. VERIFY, EXPORT, IMPORT, BLDINDEX executors
  - [x] 19.1 Implement `src/executor/verify.rs` — invoke VsamService::verify_integrity
    - Validates: Requirement 11 AC 3
  - [x] 19.2 Implement VERIFY success — LASTCC=0, IDC0590I (consistent) or IDC0001I (corrected)
    - Validates: Requirement 11 AC 5, AC 6
  - [x] 19.3 Implement VERIFY errors — not found/access failure (IDC0591E), non-VSAM (IDC0592E)
    - Validates: Requirement 11 AC 7, AC 8
  - [x] 19.4 Implement `src/executor/export.rs` — invoke CatalogService::export_dataset
    - Validates: Requirement 12 AC 6
  - [x] 19.5 Implement EXPORT errors — source not found (IDC0600E), output write failure (IDC0601E)
    - Validates: Requirement 12 AC 7, AC 8
  - [x] 19.6 Implement EXPORT success — LASTCC=0, IDC0004I with record/byte count; non-zero CC on failure
    - Validates: Requirement 12 AC 9
  - [x] 19.7 Implement `src/executor/import.rs` — invoke CatalogService::import_dataset
    - Validates: Requirement 13 AC 6
  - [x] 19.8 Implement IMPORT errors — invalid source (IDC0610E), target exists (IDC0611E)
    - Validates: Requirement 13 AC 7, AC 8
  - [x] 19.9 Implement IMPORT success — LASTCC=0, IDC0005I with record count
    - Validates: Requirement 13 AC 9
  - [x] 19.10 Implement `src/executor/bldindex.rs` — invoke VsamService::build_index
    - Validates: Requirement 14 AC 5
  - [x] 19.11 Implement BLDINDEX errors — base not found (IDC0620E), output not AIX (IDC0621E), duplicate keys warning (IDC0622W only when count > 0)
    - Validates: Requirement 14 AC 6, AC 7, AC 8
  - [x] 19.12 Implement BLDINDEX success — LASTCC=0, IDC0006I with entry count
    - Validates: Requirement 14 AC 9
  - [x] 19.13 Write unit tests for VERIFY, EXPORT, IMPORT, BLDINDEX: success paths, all error conditions
    - Validates: Requirements 11–14 (executor aspects)

- [x] 20. SYSIN processing and input modes
  - [x] 20.1 Implement `src/sysin.rs` — define `InputSource` enum (SysinDd, StringBuffer, FileInput), `InputReader` trait
  - [x] 20.2 Implement string buffer mode — direct parsing from &str
    - Validates: Requirement 17 AC 3b
  - [x] 20.3 Implement file input mode — read from file path
    - Validates: Requirement 17 AC 3c
  - [x] 20.4 Implement SYSIN DD mode — resolve DD via AllocatorService, fail with CC=16 on access failure
    - Validates: Requirement 17 AC 4
  - [x] 20.5 Implement sequence number stripping — columns 73-80 in fixed 80-byte format
    - Validates: Requirement 17 AC 5
  - [x] 20.6 Implement blank line skipping
    - Validates: Requirement 17 AC 6
  - [x] 20.7 Implement empty input handling — MAXCC=0, IDC0640I
    - Validates: Requirement 17 AC 7
  - [x] 20.8 Write unit tests for all input modes, sequence stripping, blank skipping, empty input
    - Validates: Requirement 17 AC 1–7

- [x] 21. Message formatting and error mapping
  - [x] 21.1 Implement IDCnnnnX message formatting — pattern `IDCnnnnX text` with severity indicator (I/W/E/S)
    - Validates: Requirement 19 AC 1
  - [x] 21.2 Implement severity-to-CC mapping — I→0, W→4, E→8/12, S→16
    - Validates: Requirement 19 AC 2–5
  - [x] 21.3 Implement contextual error information — include command verb, entry name, failure cause
    - Validates: Requirement 19 AC 6
  - [x] 21.4 Implement message ordering — output stream in generation order
    - Validates: Requirement 19 AC 7
  - [x] 21.5 Implement downstream error mapping — CatalogError/VsamError → IDC message code with detail text
    - Validates: Requirement 19 AC 8
  - [x] 21.6 Write unit tests for message formatting, severity mapping, contextual info, error mapping
    - Validates: Requirement 19 AC 1–10

- [x] 22. Pretty printer
  - [x] 22.1 Implement `src/pretty_printer/mod.rs` — `pretty_print(command: &Command, mode: PrintMode) -> String`
    - Validates: Requirement 26 AC 1
  - [x] 22.2 Implement verbose mode — one parameter per line, indented under verb, sub-parameters indented under parent
    - Validates: Requirement 26 AC 3
  - [x] 22.3 Implement continuation insertion — hyphen at end of line when exceeding 72 characters
    - Validates: Requirement 26 AC 4
  - [x] 22.4 Implement parameter ordering — NAME first, then type-specific, then common (z/OS conventions)
    - Validates: Requirement 26 AC 5
  - [x] 22.5 Implement compact mode — minimal whitespace, single line where possible
    - Validates: Requirement 26 AC 6
  - [x] 22.6 Implement output validity guarantee — pretty-printed output is always parseable without error
    - Validates: Requirement 26 AC 7
  - [x] 22.7 Write unit tests for both modes, continuation handling, parameter ordering
    - Validates: Requirement 26 AC 1–7
  - [x] 22.8 Write property test: pretty-print validity (Property 3) — for all generated command ASTs, pretty_print output re-parses without error
    - Validates: Requirement 26 AC 7

- [x] 23. Workbench integration and invocation modes
  - [x] 23.1 Implement `execute_idcams(input: &str, services: &IdcamsServices) -> IdcamsResult` public API
    - Validates: Requirement 20 AC 2
  - [x] 23.2 Implement command framework registration — expose IDCAMS as invocable program (EXEC PGM=IDCAMS)
    - Validates: Requirement 20 AC 1
  - [x] 23.3 Implement command palette integration — individual commands (idcams.define, idcams.listcat, idcams.delete) invocable from UI
    - Validates: Requirement 20 AC 3
  - [x] 23.4 Implement JCL invocation mode — read SYSIN DD, write SYSPRINT DD via AllocatorService
    - Validates: Requirement 20 AC 4
  - [x] 23.5 Implement scripting API mode — string input/output, no DD resolution regardless of context
    - Validates: Requirement 20 AC 5
  - [x] 23.6 Implement command handler trait — implement workbench CommandHandler for ff-command integration
    - Validates: Requirement 20 AC 7
  - [x] 23.7 Write unit tests for all invocation modes with mock services
    - Validates: Requirement 20 AC 1–7

- [x] 24. Ownership boundary enforcement verification
  - [x] 24.1 Verify Cargo.toml contains no rusqlite, rocksdb, or lmdb dependency
    - Validates: Requirement 21 AC 5
  - [x] 24.2 Verify no SQLite imports in source — grep for `use rusqlite` or database connection code
    - Validates: Requirement 21 AC 1
  - [x] 24.3 Verify no VSAM record-level logic — no key comparison, index maintenance, B-tree code
    - Validates: Requirement 21 AC 2
  - [x] 24.4 Verify no direct filesystem access — no std::fs calls for dataset content
    - Validates: Requirement 21 AC 3
  - [x] 24.5 Verify no JCL parsing logic — DD resolution flows through AllocatorService
    - Validates: Requirement 21 AC 4
  - [x] 24.6 Verify all downstream access through trait interfaces only
    - Validates: Requirement 21 AC 6
  - [x] 24.7 Verify syntax-level validation only — authoritative semantic validation delegated downstream
    - Validates: Requirement 21 AC 7
  - [x] 24.8 Write compilation test: construct IdcamsServices with all mock traits, execute a DEFINE, verify no concrete-type coupling
    - Validates: Requirement 21 AC 6; Requirement 25 AC 1

- [x] 25. Thread safety and testability
  - [x] 25.1 Verify Parser is stateless — no mutable state between parse calls
    - Validates: Requirement 24 AC 1
  - [x] 25.2 Verify ExecutionState is per-invocation — no global mutable statics
    - Validates: Requirement 24 AC 2, AC 4
  - [x] 25.3 Verify public API types implement Send + Sync where appropriate
    - Validates: Requirement 24 AC 3
  - [x] 25.4 Implement test helper builder for IdcamsServices with configurable mock responses
    - Validates: Requirement 25 AC 5
  - [x] 25.5 Verify Parser independently testable — no service dependencies needed
    - Validates: Requirement 25 AC 2
  - [x] 25.6 Verify Command_Executor testable with mocks — configurable success/error responses
    - Validates: Requirement 25 AC 3
  - [x] 25.7 Verify parsed command types are public — external crates can construct commands programmatically
    - Validates: Requirement 25 AC 4
  - [x] 25.8 Write thread safety compilation tests — spawn concurrent execute_idcams from multiple threads
    - Validates: Requirement 24 AC 1–5
  - [x] 25.9 Write property test: concurrent invocation isolation (Property 4) — run N parallel invocations, verify each has independent LASTCC/MAXCC
    - Validates: Requirement 24 AC 2

- [x] 26. Performance validation
  - [x] 26.1 Write benchmark: single statement parse time — verify < 1ms for 1024-char statement
    - Validates: Requirement 23 AC 1
  - [x] 26.2 Write benchmark: batch parse time — verify 1000 commands parse within 500ms
    - Validates: Requirement 23 AC 2
  - [x] 26.3 Write benchmark: executor overhead — verify < 5ms per command excluding downstream time
    - Validates: Requirement 23 AC 3
  - [x] 26.4 Verify REPRO streaming — records processed one at a time, not buffered entirely; fallback with warning if unavailable
    - Validates: Requirement 23 AC 4
  - [x] 26.5 Verify LISTCAT pagination/streaming — catalogs >10,000 entries not fully loaded into memory
    - Validates: Requirement 23 AC 5
  - [x] 26.6 Write benchmark: parser memory — verify < 64KB heap allocation per command parse
    - Validates: Requirement 23 AC 6

- [x] 27. Integration tests
  - [x] 27.1 Write integration test: full invocation — multi-command SYSIN with DEFINE, LISTCAT, DELETE, IF/THEN/ELSE, verify MAXCC propagation
    - Validates: Requirements 15, 16, 18 (end-to-end)
  - [x] 27.2 Write integration test: atomic rollback — DEFINE CLUSTER where VSAM init fails, verify catalog entry rolled back
    - Validates: Requirement 22 AC 2
  - [x] 27.3 Write integration test: REPRO round-trip — create KSDS, REPRO data in, REPRO data out, verify content matches
    - Validates: Requirement 10 (end-to-end)
  - [x] 27.4 Write integration test: mock testability — full invocation with all-mock services, verify no real I/O
    - Validates: Requirement 25 AC 6
  - [x] 27.5 Write integration test: command palette invocation — register command, invoke via framework, verify result
    - Validates: Requirement 20 AC 3, AC 7

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Parser | AC 1 (tokenize) | 2.1–2.9 |
| Req 1: Parser | AC 2 (nested parens) | 2.4 |
| Req 1: Parser | AC 3 (continuation) | 2.5 |
| Req 1: Parser | AC 4 (semicolons) | 2.6 |
| Req 1: Parser | AC 5 (block comments) | 2.7 |
| Req 1: Parser | AC 6 (line comments) | 2.7 |
| Req 1: Parser | AC 7 (case insensitive) | 2.3 |
| Req 1: Parser | AC 8 (DSN format) | 2.8 |
| Req 1: Parser | AC 9 (unrecognized verb) | 3.4 |
| Req 1: Parser | AC 10 (malformed param) | 3.5 |
| Req 1: Parser | AC 11 (AST output) | 3.1, 3.3 |
| Req 1: Parser | AC 12 (round-trip) | 3.7 |
| Req 2: DEFINE CLUSTER | AC 1–17 (parsing) | 4.1–4.11 |
| Req 2: DEFINE CLUSTER | AC 18–21 (execution) | 12.6–12.10 |
| Req 3: DEFINE AIX | AC 1–6 (parsing) | 5.1 |
| Req 3: DEFINE AIX | AC 7–10 (execution) | 13.1–13.4 |
| Req 4: DEFINE PATH | AC 1–4 (parsing) | 5.2 |
| Req 4: DEFINE PATH | AC 5–7 (execution) | 13.5–13.7 |
| Req 5: DEFINE GDG | AC 1–6 (parsing) | 5.3 |
| Req 5: DEFINE GDG | AC 7–10 (execution) | 13.8–13.11 |
| Req 6: DELETE | AC 1–7 (parsing) | 6.1 |
| Req 6: DELETE | AC 8–17 (execution) | 14.1–14.12 |
| Req 7: ALTER | AC 1–5 (parsing) | 6.2 |
| Req 7: ALTER | AC 6–10 (execution) | 15.1–15.6 |
| Req 8: LISTCAT | AC 1–6, 14 (parsing) | 6.3, 6.4 |
| Req 8: LISTCAT | AC 7–13 (execution) | 16.1–16.7 |
| Req 9: PRINT | AC 1–8 (parsing) | 7.1 |
| Req 9: PRINT | AC 9–15 (execution) | 17.1–17.10 |
| Req 10: REPRO | AC 1–7 (parsing) | 7.2 |
| Req 10: REPRO | AC 8–17 (execution) | 18.1–18.10 |
| Req 11: VERIFY | AC 1–2 (parsing) | 7.3 |
| Req 11: VERIFY | AC 3–8 (execution) | 19.1–19.3 |
| Req 12: EXPORT | AC 1–5 (parsing) | 7.4, 7.5 |
| Req 12: EXPORT | AC 6–9 (execution) | 19.4–19.6 |
| Req 13: IMPORT | AC 1–5 (parsing) | 7.6 |
| Req 13: IMPORT | AC 6–9 (execution) | 19.7–19.9 |
| Req 14: BLDINDEX | AC 1–4 (parsing) | 7.7 |
| Req 14: BLDINDEX | AC 5–9 (execution) | 19.10–19.12 |
| Req 15: Return Codes | AC 1–2 (registers) | 9.1, 9.2 |
| Req 15: Return Codes | AC 3 (CC values) | 1.6 |
| Req 15: Return Codes | AC 4–5 (SET) | 8.1, 9.4 |
| Req 15: Return Codes | AC 6 (return MAXCC) | 9.6 |
| Req 15: Return Codes | AC 7 (CC=16 terminate) | 9.3 |
| Req 15: Return Codes | AC 8 (summary message) | 9.5 |
| Req 16: Modal Commands | AC 1–6 (parsing) | 8.2–8.6 |
| Req 16: Modal Commands | AC 7–8 (execution) | 10.4, 10.5 |
| Req 16: Modal Commands | AC 9 (invalid operand) | 8.7 |
| Req 16: Modal Commands | AC 10 (nested IF) | 8.6 |
| Req 17: SYSIN | AC 1–7 | 20.1–20.8 |
| Req 18: Chaining | AC 1–6 | 10.1–10.8 |
| Req 19: Messages | AC 1–10 | 21.1–21.6 |
| Req 20: Integration | AC 1–7 | 23.1–23.7 |
| Req 21: Ownership | AC 1–8 | 24.1–24.8 |
| Req 22: Atomicity | AC 1–6 | 11.1–11.6 |
| Req 23: Performance | AC 1–6 | 26.1–26.6 |
| Req 24: Thread Safety | AC 1–5 | 25.1–25.3, 25.8, 25.9 |
| Req 25: Testability | AC 1–6 | 25.4–25.7, 24.8, 27.4 |
| Req 26: Pretty Printer | AC 1–7 | 22.1–22.8 |

---

## Property-Based Test Summary

| Property | Statement | Task | Validates |
|----------|-----------|------|-----------|
| P1 | Round-trip: parse → pretty-print → re-parse produces equivalent AST | 3.7 | Req 1 AC 12; Req 26 AC 2 |
| P2 | MAXCC monotonicity: across any command sequence, maxcc never decreases | 9.8 | Req 15 AC 2 |
| P3 | Pretty-print validity: all generated ASTs produce parseable output | 22.8 | Req 26 AC 7 |
| P4 | Concurrent isolation: N parallel invocations have independent LASTCC/MAXCC | 25.9 | Req 24 AC 2 |

---

## Task Dependency Graph

```json
{
  "waves": [
    {
      "id": 0,
      "label": "Scaffold, errors, services, messages",
      "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7"]
    },
    {
      "id": 1,
      "label": "Lexer and token types",
      "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7", "2.8", "2.9"],
      "dependsOn": [0]
    },
    {
      "id": 2,
      "label": "AST and parser framework",
      "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7"],
      "dependsOn": [1]
    },
    {
      "id": 3,
      "label": "Command parsers (DEFINE, DELETE, ALTER, LISTCAT, PRINT, REPRO, etc.)",
      "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "4.8", "4.9", "4.10", "4.11", "5.1", "5.2", "5.3", "5.4", "6.1", "6.2", "6.3", "6.4", "6.5", "7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "7.8", "8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8"],
      "dependsOn": [2]
    },
    {
      "id": 4,
      "label": "Execution context, return codes, command dispatch, rollback",
      "tasks": ["9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "9.8", "10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8", "11.1", "11.2", "11.3", "11.4", "11.5", "11.6"],
      "dependsOn": [3]
    },
    {
      "id": 5,
      "label": "Command executors (DEFINE, DELETE, ALTER, LISTCAT, PRINT, REPRO, VERIFY, EXPORT, IMPORT, BLDINDEX)",
      "tasks": ["12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7", "12.8", "12.9", "12.10", "13.1", "13.2", "13.3", "13.4", "13.5", "13.6", "13.7", "13.8", "13.9", "13.10", "13.11", "13.12", "14.1", "14.2", "14.3", "14.4", "14.5", "14.6", "14.7", "14.8", "14.9", "14.10", "14.11", "14.12", "15.1", "15.2", "15.3", "15.4", "15.5", "15.6", "16.1", "16.2", "16.3", "16.4", "16.5", "16.6", "16.7", "17.1", "17.2", "17.3", "17.4", "17.5", "17.6", "17.7", "17.8", "17.9", "17.10", "18.1", "18.2", "18.3", "18.4", "18.5", "18.6", "18.7", "18.8", "18.9", "18.10", "19.1", "19.2", "19.3", "19.4", "19.5", "19.6", "19.7", "19.8", "19.9", "19.10", "19.11", "19.12", "19.13"],
      "dependsOn": [4]
    },
    {
      "id": 6,
      "label": "SYSIN, messages, pretty printer",
      "tasks": ["20.1", "20.2", "20.3", "20.4", "20.5", "20.6", "20.7", "20.8", "21.1", "21.2", "21.3", "21.4", "21.5", "21.6", "22.1", "22.2", "22.3", "22.4", "22.5", "22.6", "22.7", "22.8"],
      "dependsOn": [4]
    },
    {
      "id": 7,
      "label": "Integration, ownership verification, thread safety, performance",
      "tasks": ["23.1", "23.2", "23.3", "23.4", "23.5", "23.6", "23.7", "24.1", "24.2", "24.3", "24.4", "24.5", "24.6", "24.7", "24.8", "25.1", "25.2", "25.3", "25.4", "25.5", "25.6", "25.7", "25.8", "25.9", "26.1", "26.2", "26.3", "26.4", "26.5", "26.6", "27.1", "27.2", "27.3", "27.4", "27.5"],
      "dependsOn": [5, 6]
    }
  ]
}
```

---

## Notes

- Waves 5 and 6 are independent and can proceed in parallel once Wave 4 is complete
- The parser (Waves 1–3) is fully testable without any downstream services — only strings in, AST out
- All executor tests use mock implementations of CatalogService, VsamService, and AllocatorService
- The `StubVsamService` from ff-vsam-services enables compilation before full VSAM implementation exists
- Property tests use the `proptest` crate with minimum 100 iterations
- The round-trip property test (P1) is the most important — it validates both parser and pretty printer together
- Performance benchmarks (Task 26) use `criterion` or `std::time::Instant` — they validate but do not block CI
