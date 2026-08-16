# Implementation Plan: Dataset Allocator (`ff-dataset-allocator`)

## Overview

This task plan implements the `ff-dataset-allocator` crate — the JCL Dataset Allocation subsystem for FileForgeWorkbench. It parses JCL DD statements, resolves dataset names against mounted catalogs, performs symbolic parameter substitution, simulates dataset allocation, handles GDG relative generations, concatenation, temporary datasets, referbacks, and provides a RESOLVE command with a resolution output panel.

**Crate location:** `crates/ff-dataset-allocator`
**Upstream dependencies:** `ff-dataset-catalog` (Wave 12), `ff-vfs` (Wave 3), `ff-command` (Wave 5), `ff-language-service` (Wave 8), `ff-config` (Wave 1), `ff-logging` (Wave 0)
**Downstream consumers:** JCL editing workflows, dataset management UI

---

## Tasks

- [ ] 1. Project scaffold, error types, and configuration model
  - [ ] 1.1 Create `crates/ff-dataset-allocator/Cargo.toml` with dependencies (thiserror, tracing, serde, toml) and dev-dependencies (proptest, pretty_assertions, tempfile)
  - [ ] 1.2 Create `crates/ff-dataset-allocator/src/lib.rs` with crate-level doc comment and public module declarations
  - [ ] 1.3 Implement `src/error.rs` — define `JclResolverError` enum with variants: SyntaxError, UnresolvedDsn, UnresolvedSymbolic, DispConflict, ReferbackNotFound, GdgNotFound, ConcatenationError, InvalidDsnSyntax, CatalogQueryFailed, InternalError; each variant carries line number, ddname, DSN, and reason context
    - Validates: Requirement 15 AC 1, AC 6
  - [ ] 1.4 Implement `src/config.rs` — define `ResolverConfig` struct with fields: `resolve_mode` (DryRun/Live enum), `default_hlq`, `catalog_search_order`, `lint_level` (Error/Warning/Info enum), `max_referback_depth`, `auto_resolve`; implement `Default` trait with documented defaults
    - Validates: Requirement 14 AC 1, AC 5
  - [ ] 1.5 Implement `src/diagnostic.rs` — define `LintDiagnostic` struct with fields: severity (Error/Warning/Info), line_number, col_start, col_end, code (String, e.g. "JCL001"), message; implement Display and ordering by line number
    - Validates: Requirement 10 AC 1, AC 9
  - [ ] 1.6 Write unit tests for `JclResolverError` Display output: all variants carry context (line, ddname, reason), unique diagnostic codes map correctly
    - Validates: Requirement 15 AC 1, AC 6
  - [ ] 1.7 Write unit tests for `ResolverConfig` default values and TOML deserialization from `[jcl]` table
    - Validates: Requirement 14 AC 1, AC 2

- [ ] 2. DSN model and DD statement operand models
  - [ ] 2.1 Implement `src/dsn.rs` — define `DatasetName` struct (validated 1–44 char name, qualifier validation: 1–8 chars, alpha/national start), `PdsMemberRef` struct, `GdgRelativeRef` struct (base + offset), `TempDsnRef` struct (&&-prefixed names), `ReferbackRef` struct (stepname.ddname / stepname.procstepname.ddname)
    - Validates: Requirement 1 AC 2, AC 3; Requirement 10 AC 7
  - [ ] 2.2 Implement `DatasetName::parse()` — validate DSN syntax rules (max 44 chars, qualifiers 1–8 chars starting alpha/national, no empty qualifiers); return `LintDiagnostic` on invalid syntax
    - Validates: Requirement 10 AC 7
  - [ ] 2.3 Implement `src/operands.rs` — define `DispStatus` enum (New, Old, Shr, Mod), `DispAction` enum (Keep, Delete, Catlg, Uncatlg, Pass), `Disposition` struct (status, normal_disp, abnormal_disp)
    - Validates: Requirement 1 AC 4; Requirement 4 AC 7
  - [ ] 2.4 Implement DCB model in `src/operands.rs` — define `Dcb` struct with optional fields: recfm (String), lrecl (u32), blksize (u32), dsorg (String)
    - Validates: Requirement 1 AC 5
  - [ ] 2.5 Implement SPACE model in `src/operands.rs` — define `SpaceUnit` enum (Trk, Cyl, Blksize(u32)), `SpaceAllocation` struct (unit, primary, secondary, directory)
    - Validates: Requirement 1 AC 6
  - [ ] 2.6 Implement `src/dd_statement.rs` — define `DdStatement` struct aggregating: ddname, dsn (enum: Explicit/Referback/Temporary/None), disp, dcb, space, sysout_class, is_dummy, is_inline, concatenation_index; implement builder-style construction
    - Validates: Requirement 1 AC 1–10
  - [ ] 2.7 Write unit tests for DSN validation: valid names, too-long names, invalid qualifier start, empty qualifiers, PDS member extraction, GDG relative syntax
    - Validates: Requirement 1 AC 2, AC 3; Requirement 8 AC 1; Requirement 10 AC 7
  - [ ] 2.8 Write property test: DSN validation (Property 1) — generate random strings (1–50 chars), verify parse accepts only valid DSN syntax per z/OS rules (≤44 chars, qualifiers ≤8 chars, alpha/national start, no consecutive dots)
    - Validates: Requirement 10 AC 7, AC 8

- [ ] 3. DD statement parser
  - [ ] 3.1 Implement `src/parser.rs` — define `JclParser` struct with methods: `parse_dd_statement(line: &str, line_number: usize) -> Result<DdStatement, LintDiagnostic>` and `parse_job(text: &str) -> ParseResult`
  - [ ] 3.2 Implement ddname extraction (columns 3–10), operand field tokenisation (comma-separated, respecting parenthesised groups and quoted strings)
    - Validates: Requirement 1 AC 1
  - [ ] 3.3 Implement DSN operand parsing — unquoted and quoted names, PDS member references `DSN=name(member)`, GDG relative `DSN=name(+n)`
    - Validates: Requirement 1 AC 2, AC 3; Requirement 8 AC 1
  - [ ] 3.4 Implement DISP operand parsing — up to 3 positional sub-parameters within parentheses
    - Validates: Requirement 1 AC 4
  - [ ] 3.5 Implement DCB operand parsing — key=value pairs within parenthesised list
    - Validates: Requirement 1 AC 5
  - [ ] 3.6 Implement SPACE operand parsing — nested positional format `SPACE=(unit,(primary,secondary,directory))`
    - Validates: Requirement 1 AC 6
  - [ ] 3.7 Implement JCL continuation line handling — detect non-blank column 72, join with next `// ` prefixed line
    - Validates: Requirement 1 AC 7
  - [ ] 3.8 Implement SYSOUT, DD *, DD DATA, and DUMMY keyword recognition
    - Validates: Requirement 1 AC 8, AC 9, AC 10
  - [ ] 3.9 Implement syntax error detection — unbalanced parentheses, invalid operand format; emit LintDiagnostic at ERROR severity
    - Validates: Requirement 1 AC 11
  - [ ] 3.10 Implement concatenation detection — blank ddname columns following a DD statement
    - Validates: Requirement 5 AC 1
  - [ ] 3.11 Write unit tests for DD statement parsing: basic DD, DSN extraction (quoted/unquoted), member refs, DISP parsing, DCB parsing, SPACE parsing, continuation lines, SYSOUT, DD *, DUMMY, syntax errors
    - Validates: Requirement 1 AC 1–11
  - [ ] 3.12 Write property test: parser round-trip (Property 2) — generate valid DD statement strings from model, parse them, assert extracted operands match generated inputs
    - Validates: Requirement 1 AC 1–6

- [ ] 4. Job structure parsing
  - [ ] 4.1 Implement `src/job_model.rs` — define `JclJob` struct (job_name, steps: Vec<JclStep>), `JclStep` struct (step_name, program/proc, dd_statements: Vec<DdStatement>, overrides), `ProcDefinition` struct
    - Validates: Requirement 12 AC 3
  - [ ] 4.2 Implement JOB statement parsing — extract job name for `&SYSJOBNAME`; handle missing JOB (default `"NOJOB"`)
    - Validates: Requirement 12 AC 1, AC 8
  - [ ] 4.3 Implement EXEC statement parsing — extract step name, PGM= or proc name, symbolic overrides
    - Validates: Requirement 12 AC 2
  - [ ] 4.4 Implement PROC/PEND parsing and in-stream procedure expansion with DD override merging (`//step.procstep DD ...`)
    - Validates: Requirement 12 AC 4, AC 5
  - [ ] 4.5 Implement IF/THEN/ELSE/ENDIF construct recognition (include all conditional paths in model)
    - Validates: Requirement 12 AC 6
  - [ ] 4.6 Implement step ordering and cumulative state tracking (resolved DSNs, temp table, pass table, GDG state)
    - Validates: Requirement 12 AC 7
  - [ ] 4.7 Write unit tests for job structure parsing: multi-step job, proc expansion, DD overrides, IF/THEN/ELSE, no-JOB fragment
    - Validates: Requirement 12 AC 1–8
  - [ ] 4.8 Write property test: job structure hierarchy (Property 3) — generate random multi-step jobs, assert step count matches EXEC count and DD assignment is correct
    - Validates: Requirement 12 AC 3, AC 7

- [ ] 5. Symbolic substitution engine and symbol table management
  - [ ] 5.1 Implement `src/symbol_table.rs` — define `SymbolTable` struct with HashMap<String, String> storage; implement `define()`, `get()`, `contains()`, `merge()`, `scope_push()`, `scope_pop()` for nested proc scopes
    - Validates: Requirement 3 AC 1, AC 3
  - [ ] 5.2 Implement system symbol population — `&SYSDATE`, `&SYSDATE4`, `&SYSTIME`, `&SYSJOBNAME`, `&SYSSTEP`, `&SYSUID`; values derived from environment/config at resolution time
    - Validates: Requirement 3 AC 2
  - [ ] 5.3 Implement SET statement parsing (`// SET symbol=value`) and PROC parameter default extraction to populate symbol table
    - Validates: Requirement 3 AC 3
  - [ ] 5.4 Implement EXEC override merging — `//step EXEC proc,symbol=value` overrides take precedence over PROC defaults
    - Validates: Requirement 3 AC 4
  - [ ] 5.5 Implement `src/substitution.rs` — define `substitute(text: &str, symbols: &SymbolTable) -> SubstitutionResult`; single left-to-right pass replacing `&symbol` and `&symbol.` references
    - Validates: Requirement 3 AC 1, AC 9
  - [ ] 5.6 Implement dot-terminator convention — `&SYM.REST` → value of SYM + literal REST (dot consumed)
    - Validates: Requirement 3 AC 6
  - [ ] 5.7 Implement double-ampersand handling — `&&` as literal ampersand (non-temp context) or temp DSN prefix; skip substitution for `&&`-prefixed temp names
    - Validates: Requirement 3 AC 7
  - [ ] 5.8 Implement substring notation — `&symbol(start,length)` extracts portion of symbol value
    - Validates: Requirement 3 AC 8
  - [ ] 5.9 Implement unresolved symbolic detection — any `&symbol` remaining after substitution produces ERROR diagnostic
    - Validates: Requirement 3 AC 5
  - [ ] 5.10 Implement persistent symbol loading from `[jcl.symbols]` configuration table
    - Validates: Requirement 3 AC 10; Requirement 14 AC 2
  - [ ] 5.11 Write unit tests for symbolic substitution: basic replacement, dot terminator, double-ampersand, substring, unresolved detection, SET parsing, EXEC overrides, system symbols
    - Validates: Requirement 3 AC 1–10
  - [ ] 5.12 Write property test: substitution idempotence (Property 4) — generate text with known symbols, substitute once, assert no `&known_symbol` remains; substitute again, assert output unchanged
    - Validates: Requirement 3 AC 1, AC 9
  - [ ] 5.13 Write property test: dot terminator correctness (Property 5) — generate `&SYM.suffix` patterns, assert output equals value(SYM) + suffix with dot consumed
    - Validates: Requirement 3 AC 6

- [ ] 6. Catalog resolution bridge
  - [ ] 6.1 Implement `src/catalog_bridge.rs` — define `CatalogResolver` trait with methods: `resolve_dsn(dsn: &DatasetName) -> Result<ResolutionResult>`, `dataset_exists(dsn: &DatasetName) -> bool`, `member_exists(pds: &DatasetName, member: &str) -> bool`, `allocate_dataset(dsn: &DatasetName, attrs: &DatasetAttributes) -> Result<()>`
    - Validates: Requirement 2 AC 8
  - [ ] 6.2 Implement `CatalogBridgeImpl` struct wrapping `ff-dataset-catalog` API — delegate `resolve_dsn` to catalog's resolution API, honouring VFS abstraction
    - Validates: Requirement 2 AC 1, AC 2, AC 8
  - [ ] 6.3 Implement catalog search order logic — if `jcl.catalog_search_order` configured, use that order; otherwise use ff-dataset-catalog default mount-order priority; emit WARN on multi-catalog ambiguity
    - Validates: Requirement 2 AC 3; Requirement 14 AC 7
  - [ ] 6.4 Implement DSN not-found handling — produce ERROR diagnostic "Dataset not found: {dsn}" when DISP=OLD/SHR and no catalog match
    - Validates: Requirement 2 AC 4
  - [ ] 6.5 Implement PDS member verification — check base PDS exists AND member exists in PDS directory; produce WARNING if member missing
    - Validates: Requirement 2 AC 5, AC 6
  - [ ] 6.6 Implement wildcard/pattern rejection — DSN with `*` produces ERROR diagnostic
    - Validates: Requirement 2 AC 7
  - [ ] 6.7 Implement default HLQ prefixing — prepend `jcl.default_hlq` to unqualified DSNs (fewer than 2 qualifiers) before lookup
    - Validates: Requirement 14 AC 6
  - [ ] 6.8 Implement catalog query error handling — on SQLite/IO error, produce ERROR diagnostic and continue with remaining catalogs
    - Validates: Requirement 15 AC 3
  - [ ] 6.9 Write unit tests with mock catalog: successful resolution, not-found, multi-catalog ambiguity, PDS member check, wildcard rejection, default HLQ prepend, catalog error handling
    - Validates: Requirement 2 AC 1–8; Requirement 14 AC 6, AC 7
  - [ ] 6.10 Write property test: catalog resolution consistency (Property 6) — generate DSNs present in mock catalog, assert resolve always returns same physical path for same DSN
    - Validates: Requirement 2 AC 1, AC 2

- [ ] 7. DISP interpretation and allocation simulator
  - [ ] 7.1 Implement `src/allocation.rs` — define `AllocationSimulator` struct with reference to `CatalogResolver` trait and `ResolverConfig`
  - [ ] 7.2 Implement DISP=NEW handling — invoke catalog allocation API with DCB/SPACE attributes; produce ERROR if DSN already exists
    - Validates: Requirement 4 AC 1, AC 3
  - [ ] 7.3 Implement DCB attribute extraction and default fallback chain: DD-level DCB → `[catalog.defaults]` config → hardcoded RECFM=FB, LRECL=80, BLKSIZE=27920
    - Validates: Requirement 4 AC 2
  - [ ] 7.4 Implement DISP=OLD/SHR handling — verify DSN existence via catalog bridge; produce ERROR if not found
    - Validates: Requirement 4 AC 4, AC 5
  - [ ] 7.5 Implement DISP=MOD handling — verify existence for append; if not found AND SPACE provided, treat as NEW
    - Validates: Requirement 4 AC 6
  - [ ] 7.6 Implement default DISP logic — no DISP operand defaults to `DISP=(NEW,DELETE)`
    - Validates: Requirement 4 AC 7
  - [ ] 7.7 Implement PASS disposition — record passed dataset in job-scoped pass table for referback by subsequent steps
    - Validates: Requirement 4 AC 8
  - [ ] 7.8 Implement dry-run mode — report allocations without creating catalog entries
    - Validates: Requirement 4 AC 9
  - [ ] 7.9 Implement live mode — perform actual catalog allocations for DISP=NEW
    - Validates: Requirement 4 AC 10
  - [ ] 7.10 Write unit tests for allocation simulation: NEW creates, NEW duplicate error, OLD not-found, SHR not-found, MOD with/without SPACE, default DISP, PASS recording, dry-run vs live mode, DCB fallback chain
    - Validates: Requirement 4 AC 1–10
  - [ ] 7.11 Write property test: DISP default application (Property 7) — generate DD statements without explicit DISP, assert resolved disposition is always (NEW, DELETE)
    - Validates: Requirement 4 AC 7

- [ ] 8. Concatenation handler
  - [ ] 8.1 Implement `src/concatenation.rs` — define `ConcatenationGroup` struct (ddname, components: Vec<DdStatement>, resolved: Vec<ResolutionResult>)
  - [ ] 8.2 Implement concatenation group assembly — collect consecutive blank-ddname DDs following a named DD into a group
    - Validates: Requirement 5 AC 1
  - [ ] 8.3 Implement independent resolution of each concatenation component with 1-based index tracking
    - Validates: Requirement 5 AC 2, AC 3
  - [ ] 8.4 Implement component failure handling — produce ERROR diagnostic identifying ddname and failing concatenation index
    - Validates: Requirement 5 AC 4
  - [ ] 8.5 Implement attribute compatibility validation — check RECFM match and LRECL compatibility; produce WARNING on mismatch
    - Validates: Requirement 5 AC 5
  - [ ] 8.6 Implement 255-dataset concatenation limit enforcement — produce ERROR if exceeded
    - Validates: Requirement 5 AC 6
  - [ ] 8.7 Write unit tests for concatenation: group assembly, independent resolution, component failure, attribute mismatch warning, 255 limit
    - Validates: Requirement 5 AC 1–6
  - [ ] 8.8 Write property test: concatenation ordering preservation (Property 8) — generate N-component concatenations (1–255), resolve all, assert order indices match declaration order
    - Validates: Requirement 5 AC 3

- [ ] 9. Temporary dataset registry
  - [ ] 9.1 Implement `src/temp_registry.rs` — define `TempDatasetRegistry` struct with HashMap<String, TempEntry> (TempEntry: creating_step, attributes, deleted flag)
  - [ ] 9.2 Implement temp dataset creation registration — `DISP=(NEW,...), DSN=&&name` inserts into registry with step name and attributes
    - Validates: Requirement 6 AC 2
  - [ ] 9.3 Implement temp dataset reference lookup — `DISP=(OLD/SHR,...), DSN=&&name` resolves from registry, returns ResolutionResult with temp indicator
    - Validates: Requirement 6 AC 3
  - [ ] 9.4 Implement temp not-created error — reference to temp not in registry produces ERROR diagnostic
    - Validates: Requirement 6 AC 4
  - [ ] 9.5 Implement catalog isolation — temp datasets never resolved against mounted catalogs
    - Validates: Requirement 6 AC 5
  - [ ] 9.6 Implement system-generated temp name assignment — DD with no DSN and DISP=(NEW,PASS) gets `&&SYSnnnnn` name
    - Validates: Requirement 6 AC 6
  - [ ] 9.7 Implement temp lifecycle tracking — DISP=(,DELETE) marks temp as deleted; subsequent references produce ERROR
    - Validates: Requirement 6 AC 7
  - [ ] 9.8 Write unit tests for temp registry: create, reference, not-created error, catalog isolation, system name generation, delete lifecycle
    - Validates: Requirement 6 AC 1–7
  - [ ] 9.9 Write property test: temp dataset isolation (Property 9) — generate temp names (&&prefix), assert they never trigger catalog resolution calls
    - Validates: Requirement 6 AC 5

- [ ] 10. Referback resolver
  - [ ] 10.1 Implement `src/referback.rs` — define `ReferbackResolver` struct with access to job model step results
  - [ ] 10.2 Implement `*.stepname.ddname` resolution — locate DD in specified prior step, use its resolved DSN
    - Validates: Requirement 7 AC 1, AC 2
  - [ ] 10.3 Implement `*.stepname.procstepname.ddname` resolution — locate DD in procedure step within specified step
    - Validates: Requirement 7 AC 3
  - [ ] 10.4 Implement step-not-found error — produce ERROR diagnostic when referback target step does not exist
    - Validates: Requirement 7 AC 4
  - [ ] 10.5 Implement dd-not-found error — produce ERROR diagnostic when referback target ddname does not exist in target step
    - Validates: Requirement 7 AC 5
  - [ ] 10.6 Implement recursive referback chain following — follow chains up to configurable depth (default 10); produce ERROR on depth exceeded
    - Validates: Requirement 7 AC 6
  - [ ] 10.7 Implement referback ordering constraint — referback resolution occurs after symbolic substitution, before catalog lookup
    - Validates: Requirement 7 AC 7
  - [ ] 10.8 Write unit tests for referback: simple referback, proc-step referback, step not found, dd not found, recursive chain, depth limit exceeded, ordering after substitution
    - Validates: Requirement 7 AC 1–7
  - [ ] 10.9 Write property test: referback chain depth (Property 10) — generate referback chains of length 1–15, assert chains ≤10 resolve successfully and chains >10 produce depth error
    - Validates: Requirement 7 AC 6

- [ ] 11. GDG relative generation resolver
  - [ ] 11.1 Implement `src/gdg_resolver.rs` — define `GdgResolver` struct with access to `CatalogResolver` trait and job-scoped GDG state
  - [ ] 11.2 Implement generation(0) resolution — query catalog for most recent active generation, return physical path
    - Validates: Requirement 8 AC 2
  - [ ] 11.3 Implement generation(-n) resolution — query catalog for nth-most-recent; produce ERROR if fewer than n generations exist
    - Validates: Requirement 8 AC 3
  - [ ] 11.4 Implement generation(+1) with DISP=NEW — compute next generation number from catalog state, include projected name in result
    - Validates: Requirement 8 AC 4
  - [ ] 11.5 Implement generation(+n) warning for n > 1 — produce WARNING diagnostic about atypical multi-forward reference
    - Validates: Requirement 8 AC 5
  - [ ] 11.6 Implement GDG base not-found error — produce ERROR when base name is not registered as GDG in any catalog
    - Validates: Requirement 8 AC 6
  - [ ] 11.7 Implement intra-job GDG state tracking — step 1 creates (+1), step 2 sees it as current (0)
    - Validates: Requirement 8 AC 7
  - [ ] 11.8 Implement GDG roll-off detection — emit INFO diagnostic when new generation would exceed GDG limit
    - Validates: Requirement 8 AC 8
  - [ ] 11.9 Write unit tests for GDG resolution: generation(0), generation(-1), generation(+1) create, +n>1 warning, base not found, intra-job state, roll-off
    - Validates: Requirement 8 AC 1–8
  - [ ] 11.10 Write property test: GDG intra-job state consistency (Property 11) — generate multi-step jobs with sequential GDG creates and references, assert (0) always resolves to most recently created in prior steps
    - Validates: Requirement 8 AC 7

- [ ] 12. Resolution processing pipeline
  - [ ] 12.1 Implement `src/pipeline.rs` — define `ResolutionPipeline` struct orchestrating the four-stage pipeline: Parse → Substitute → Resolve → Validate
    - Validates: Requirement 13 AC 1
  - [ ] 12.2 Implement `ResolutionResult` model — define struct with fields: dd_name, step_name, dsn_original, dsn_substituted, physical_path, catalog_name, dataset_type, status (Resolved/Error/Warning/Skipped), diagnostics, concatenation_index
  - [ ] 12.3 Implement `ResolveOutput` aggregate — define struct holding: job_model (parse output), substituted_operands, resolution_results (Vec<ResolutionResult>), diagnostics (Vec<LintDiagnostic>), timing per stage
    - Validates: Requirement 13 AC 2
  - [ ] 12.4 Implement error isolation — errors in one DD do not prevent resolution of other DDs; collect all results
    - Validates: Requirement 13 AC 3
  - [ ] 12.5 Implement diagnostic aggregation — merge diagnostics from all stages, sort by line number
    - Validates: Requirement 13 AC 4
  - [ ] 12.6 Implement pipeline stage timing — emit DEBUG log records with millisecond timing for each stage
    - Validates: Requirement 13 AC 5
  - [ ] 12.7 Implement incremental resolution — detect single-DD changes, re-resolve only affected DD and dependents (referback targets)
    - Validates: Requirement 13 AC 6
  - [ ] 12.8 Implement `resolve_document(text: &str, config: &ResolverConfig) -> ResolveOutput` public API
    - Validates: Requirement 16 AC 4
  - [ ] 12.9 Write unit tests for pipeline: full pipeline pass, intermediate results inspection, error isolation, diagnostic ordering, timing presence, incremental resolution
    - Validates: Requirement 13 AC 1–6
  - [ ] 12.10 Write property test: error isolation (Property 12) — generate jobs with N DDs where some have invalid DSNs, assert resolution count always equals total DD count (no DDs dropped)
    - Validates: Requirement 13 AC 3

- [ ] 13. JCL validation and lint diagnostic emitter
  - [ ] 13.1 Implement `src/lint.rs` — define `LintEmitter` struct that collects diagnostics from all pipeline stages and applies severity filtering per `jcl.lint_level` config
    - Validates: Requirement 10 AC 10
  - [ ] 13.2 Implement unresolved DSN detection — any DD with DSN that cannot resolve produces ERROR
    - Validates: Requirement 10 AC 2
  - [ ] 13.3 Implement unresolved symbolic detection — any remaining `&symbol` after substitution produces ERROR
    - Validates: Requirement 10 AC 3
  - [ ] 13.4 Implement missing well-known DD detection — step references SYSIN/SYSPRINT/SYSUT1/SYSUT2/SYSLIB without defining them produces WARNING
    - Validates: Requirement 10 AC 4
  - [ ] 13.5 Implement duplicate ddname detection within a step (excluding concatenation) — produce ERROR
    - Validates: Requirement 10 AC 5
  - [ ] 13.6 Implement DISP conflict detection — NEW with existing DSN or OLD with non-existent DSN produce ERROR
    - Validates: Requirement 10 AC 6
  - [ ] 13.7 Implement invalid DSN syntax detection — >44 chars, >8 char qualifier, digit-start qualifier, empty qualifier (consecutive dots) produce ERROR
    - Validates: Requirement 10 AC 7
  - [ ] 13.8 Implement invalid symbolic name detection — non-alphanumeric/national chars produce ERROR
    - Validates: Requirement 10 AC 8
  - [ ] 13.9 Implement diagnostic code assignment — map each class to unique code (JCL001–JCL008)
    - Validates: Requirement 10 AC 9; Requirement 15 AC 6
  - [ ] 13.10 Write unit tests for lint: unresolved DSN, unresolved symbolic, missing DDs, duplicate ddname, DISP conflicts, invalid DSN, invalid symbolic, severity filter, diagnostic codes
    - Validates: Requirement 10 AC 1–10
  - [ ] 13.11 Write property test: diagnostic completeness (Property 13) — generate JCL with known N errors injected, assert diagnostic count ≥ N (all injected errors detected)
    - Validates: Requirement 10 AC 2, AC 3, AC 6, AC 7

- [ ] 14. RESOLVE command handler
  - [ ] 14.1 Implement `src/command.rs` — define `ResolveCommandHandler` struct implementing command-framework `CommandHandler` trait
  - [ ] 14.2 Implement command registration — register `dataset.resolve` with ID, display name "Resolve Dataset Allocation", category "dataset", and default keyboard shortcut during crate initialization
    - Validates: Requirement 9 AC 1
  - [ ] 14.3 Implement full-document resolution mode — resolve all DD statements in active document when invoked with no parameters
    - Validates: Requirement 9 AC 2
  - [ ] 14.4 Implement cursor-position resolution mode — resolve only the DD at/nearest cursor position
    - Validates: Requirement 9 AC 3
  - [ ] 14.5 Implement DSN parameter mode — resolve a specified DSN string against catalogs without JCL context
    - Validates: Requirement 9 AC 4
  - [ ] 14.6 Implement mode parameter override — accept `"dry-run"` or `"live"` parameter overriding config setting
    - Validates: Requirement 9 AC 5
  - [ ] 14.7 Implement Command_Result return — include success count, warning count, error count, list of ResolutionResults
    - Validates: Requirement 9 AC 6
  - [ ] 14.8 Implement language_id guard — if active document is not `"jcl"`, return error "Active document is not a JCL file"
    - Validates: Requirement 9 AC 8
  - [ ] 14.9 Write unit tests for RESOLVE command: full-doc mode, cursor mode, DSN param mode, mode override, language guard, result structure
    - Validates: Requirement 9 AC 1–8
  - [ ] 14.10 Write property test: command result counts consistency (Property 14) — generate jobs with known good/bad DDs, assert success+warning+error counts equal total DD count
    - Validates: Requirement 9 AC 6

- [ ] 15. Resolution output panel model
  - [ ] 15.1 Implement `src/panel.rs` — define `ResolutionPanelModel` struct with panel_id `"jcl.resolution"`, rows: Vec<PanelRow>, summary (total, resolved, warnings, errors)
    - Validates: Requirement 11 AC 1, AC 5
  - [ ] 15.2 Implement `PanelRow` struct — step_name, dd_name, dsn_substituted, status (Resolved/Error/Warning/Skipped), physical_path_or_message, catalog_name, original_dsn (for tooltip), concatenation_children
    - Validates: Requirement 11 AC 2, AC 3, AC 4, AC 9
  - [ ] 15.3 Implement `from_resolve_output(output: &ResolveOutput) -> ResolutionPanelModel` conversion — map ResolutionResults to PanelRows, compute summary counts
  - [ ] 15.4 Implement sorting support — by step name, DD name, or status column
    - Validates: Requirement 11 AC 7
  - [ ] 15.5 Implement status filter — show only errors, only warnings, or all
    - Validates: Requirement 11 AC 7
  - [ ] 15.6 Implement concatenation group display — parent row expandable with child rows for each component
    - Validates: Requirement 11 AC 10
  - [ ] 15.7 Implement navigate-to-source — row double-click produces line number for editor navigation
    - Validates: Requirement 11 AC 6
  - [ ] 15.8 Implement persistence — panel content retained until next RESOLVE or explicit clear
    - Validates: Requirement 11 AC 8
  - [ ] 15.9 Write unit tests for panel model: conversion from ResolveOutput, summary counts, sorting, filtering, concatenation grouping, navigation data, persistence behaviour
    - Validates: Requirement 11 AC 1–10

- [ ] 16. Language service integration and configuration hot-reload
  - [ ] 16.1 Implement language_id check — query `ff-language-service` to confirm document is `"jcl"` before resolution
    - Validates: Requirement 16 AC 1
  - [ ] 16.2 Implement keyword set consumption — use JCL language definition keyword sets for statement type validation (JOB, EXEC, DD, PROC, PEND, SET, IF, ELSE, ENDIF) instead of maintaining separate lists
    - Validates: Requirement 16 AC 2
  - [ ] 16.3 Implement auto-resolve on save — when `jcl.auto_resolve = true`, perform lightweight parse+substitute pass on document save (no catalog queries)
    - Validates: Requirement 16 AC 3
  - [ ] 16.4 Implement hover information provider — return resolution status, physical path, and dataset attributes for DSN tokens
    - Validates: Requirement 16 AC 5
  - [ ] 16.5 Implement configuration hot-reload subscription — subscribe to config change events; update ResolverConfig on `[jcl]` table changes without restart
    - Validates: Requirement 14 AC 4
  - [ ] 16.6 Implement configuration schema registration — register all `[jcl]` keys with types, defaults, and descriptions during initialization
    - Validates: Requirement 14 AC 5
  - [ ] 16.7 Write unit tests for language integration: language_id check, keyword consumption, auto-resolve trigger, hover data, config reload, schema registration
    - Validates: Requirement 16 AC 1–5; Requirement 14 AC 4, AC 5

- [ ] 17. Error handling, logging, and thread safety
  - [ ] 17.1 Implement structured logging — ERROR for resolution failures, WARN for ambiguous resolutions, INFO for summary, DEBUG for pipeline stage details with timing
    - Validates: Requirement 15 AC 2
  - [ ] 17.2 Implement graceful internal error handling — log ERROR with full context, return error result without panicking
    - Validates: Requirement 15 AC 4
  - [ ] 17.3 Implement `resolve_result.diagnostics()` method — return all LintDiagnostics ordered by line number
    - Validates: Requirement 15 AC 5
  - [ ] 17.4 Implement Send + Sync bounds on public API types — ensure resolver is safe to invoke from any thread
    - Validates: Cross-cutting: Thread Safety
  - [ ] 17.5 Write unit tests for error handling: catalog query failure recovery, internal error graceful return, diagnostic ordering, Send+Sync compilation check
    - Validates: Requirement 15 AC 1–6; Cross-cutting: Thread Safety
  - [ ] 17.6 Write property test: diagnostic ordering (Property 15) — generate diagnostics with random line numbers, assert `diagnostics()` output is always sorted by line_number ascending
    - Validates: Requirement 15 AC 5

- [ ] 18. Integration tests and performance validation
  - [ ] 18.1 Write integration test: full pipeline end-to-end — multi-step JCL job with symbolics, referbacks, GDG refs, concatenation, and temp datasets; verify all ResolutionResults correct
    - Validates: Requirement 13 AC 1; Cross-cutting: Testability
  - [ ] 18.2 Write integration test: RESOLVE command invocation — register command, invoke on sample JCL, assert panel model populated correctly
    - Validates: Requirement 9 AC 1–6
  - [ ] 18.3 Write integration test: mock catalog testability — verify resolver works with trait-based mock catalog (no real SQLite)
    - Validates: Cross-cutting: Testability
  - [ ] 18.4 Write integration test: parser-only testability — verify parser works independently without catalog or VFS infrastructure
    - Validates: Cross-cutting: Testability
  - [ ] 18.5 Write performance benchmark — resolve 500-DD JCL file against 10,000-dataset mock catalog; assert completes within 5 seconds
    - Validates: Requirement 9 AC 7; Cross-cutting: Performance

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: DD Statement Parsing | AC 1 (format extraction) | 3.2, 3.11 |
| Req 1: DD Statement Parsing | AC 2 (DSN extraction) | 2.1, 3.3, 3.11 |
| Req 1: DD Statement Parsing | AC 3 (member references) | 2.1, 3.3, 3.11 |
| Req 1: DD Statement Parsing | AC 4 (DISP extraction) | 2.3, 3.4, 3.11 |
| Req 1: DD Statement Parsing | AC 5 (DCB extraction) | 2.4, 3.5, 3.11 |
| Req 1: DD Statement Parsing | AC 6 (SPACE extraction) | 2.5, 3.6, 3.11 |
| Req 1: DD Statement Parsing | AC 7 (continuation lines) | 3.7, 3.11 |
| Req 1: DD Statement Parsing | AC 8 (SYSOUT recognition) | 3.8, 3.11 |
| Req 1: DD Statement Parsing | AC 9 (DD * / DATA) | 3.8, 3.11 |
| Req 1: DD Statement Parsing | AC 10 (DUMMY) | 3.8, 3.11 |
| Req 1: DD Statement Parsing | AC 11 (syntax errors) | 3.9, 3.11 |
| Req 2: DSN Resolution | AC 1 (OLD/SHR lookup) | 6.2, 6.9 |
| Req 2: DSN Resolution | AC 2 (single catalog success) | 6.2, 6.9 |
| Req 2: DSN Resolution | AC 3 (multi-catalog order) | 6.3, 6.9 |
| Req 2: DSN Resolution | AC 4 (not-found error) | 6.4, 6.9 |
| Req 2: DSN Resolution | AC 5 (PDS member verify) | 6.5, 6.9 |
| Req 2: DSN Resolution | AC 6 (member not found) | 6.5, 6.9 |
| Req 2: DSN Resolution | AC 7 (no wildcards) | 6.6, 6.9 |
| Req 2: DSN Resolution | AC 8 (via catalog API) | 6.1, 6.2 |
| Req 3: Symbolic Substitution | AC 1 (replace all) | 5.5, 5.11 |
| Req 3: Symbolic Substitution | AC 2 (system symbols) | 5.2, 5.11 |
| Req 3: Symbolic Substitution | AC 3 (SET/PROC) | 5.3, 5.11 |
| Req 3: Symbolic Substitution | AC 4 (EXEC overrides) | 5.4, 5.11 |
| Req 3: Symbolic Substitution | AC 5 (unresolved error) | 5.9, 5.11 |
| Req 3: Symbolic Substitution | AC 6 (dot terminator) | 5.6, 5.11 |
| Req 3: Symbolic Substitution | AC 7 (double ampersand) | 5.7, 5.11 |
| Req 3: Symbolic Substitution | AC 8 (substring) | 5.8, 5.11 |
| Req 3: Symbolic Substitution | AC 9 (single pass) | 5.5, 5.11 |
| Req 3: Symbolic Substitution | AC 10 (config symbols) | 5.10, 5.11 |
| Req 4: DISP/Allocation | AC 1 (NEW allocates) | 7.2, 7.10 |
| Req 4: DISP/Allocation | AC 2 (DCB defaults) | 7.3, 7.10 |
| Req 4: DISP/Allocation | AC 3 (NEW dup error) | 7.2, 7.10 |
| Req 4: DISP/Allocation | AC 4 (OLD verify) | 7.4, 7.10 |
| Req 4: DISP/Allocation | AC 5 (SHR verify) | 7.4, 7.10 |
| Req 4: DISP/Allocation | AC 6 (MOD logic) | 7.5, 7.10 |
| Req 4: DISP/Allocation | AC 7 (default DISP) | 7.6, 7.10 |
| Req 4: DISP/Allocation | AC 8 (PASS table) | 7.7, 7.10 |
| Req 4: DISP/Allocation | AC 9 (dry-run) | 7.8, 7.10 |
| Req 4: DISP/Allocation | AC 10 (live mode) | 7.9, 7.10 |
| Req 5: Concatenation | AC 1 (detection) | 3.10, 8.2, 8.7 |
| Req 5: Concatenation | AC 2 (independent resolve) | 8.3, 8.7 |
| Req 5: Concatenation | AC 3 (order index) | 8.3, 8.7 |
| Req 5: Concatenation | AC 4 (component failure) | 8.4, 8.7 |
| Req 5: Concatenation | AC 5 (attribute compat) | 8.5, 8.7 |
| Req 5: Concatenation | AC 6 (255 limit) | 8.6, 8.7 |
| Req 6: Temp Datasets | AC 1 (recognition) | 2.1, 9.1 |
| Req 6: Temp Datasets | AC 2 (creation register) | 9.2, 9.8 |
| Req 6: Temp Datasets | AC 3 (reference lookup) | 9.3, 9.8 |
| Req 6: Temp Datasets | AC 4 (not-created error) | 9.4, 9.8 |
| Req 6: Temp Datasets | AC 5 (catalog isolation) | 9.5, 9.8 |
| Req 6: Temp Datasets | AC 6 (system name) | 9.6, 9.8 |
| Req 6: Temp Datasets | AC 7 (lifecycle tracking) | 9.7, 9.8 |
| Req 7: Referback | AC 1 (syntax recognition) | 2.1, 10.2 |
| Req 7: Referback | AC 2 (step.dd resolve) | 10.2, 10.8 |
| Req 7: Referback | AC 3 (step.proc.dd resolve) | 10.3, 10.8 |
| Req 7: Referback | AC 4 (step not found) | 10.4, 10.8 |
| Req 7: Referback | AC 5 (dd not found) | 10.5, 10.8 |
| Req 7: Referback | AC 6 (chain depth) | 10.6, 10.8 |
| Req 7: Referback | AC 7 (ordering) | 10.7, 10.8 |
| Req 8: GDG Resolution | AC 1 (syntax) | 2.1, 3.3 |
| Req 8: GDG Resolution | AC 2 (gen 0) | 11.2, 11.9 |
| Req 8: GDG Resolution | AC 3 (gen -n) | 11.3, 11.9 |
| Req 8: GDG Resolution | AC 4 (gen +1 create) | 11.4, 11.9 |
| Req 8: GDG Resolution | AC 5 (+n>1 warning) | 11.5, 11.9 |
| Req 8: GDG Resolution | AC 6 (base not found) | 11.6, 11.9 |
| Req 8: GDG Resolution | AC 7 (intra-job state) | 11.7, 11.9 |
| Req 8: GDG Resolution | AC 8 (roll-off) | 11.8, 11.9 |
| Req 9: RESOLVE Command | AC 1 (registration) | 14.2, 14.9 |
| Req 9: RESOLVE Command | AC 2 (full-doc mode) | 14.3, 14.9 |
| Req 9: RESOLVE Command | AC 3 (cursor mode) | 14.4, 14.9 |
| Req 9: RESOLVE Command | AC 4 (DSN param) | 14.5, 14.9 |
| Req 9: RESOLVE Command | AC 5 (mode override) | 14.6, 14.9 |
| Req 9: RESOLVE Command | AC 6 (result structure) | 14.7, 14.9 |
| Req 9: RESOLVE Command | AC 7 (5s performance) | 18.5 |
| Req 9: RESOLVE Command | AC 8 (language guard) | 14.8, 14.9 |
| Req 10: Lint Diagnostics | AC 1 (severity levels) | 1.5, 13.10 |
| Req 10: Lint Diagnostics | AC 2 (unresolved DSN) | 13.2, 13.10 |
| Req 10: Lint Diagnostics | AC 3 (unresolved symbolic) | 13.3, 13.10 |
| Req 10: Lint Diagnostics | AC 4 (missing DDs) | 13.4, 13.10 |
| Req 10: Lint Diagnostics | AC 5 (dup ddnames) | 13.5, 13.10 |
| Req 10: Lint Diagnostics | AC 6 (DISP conflicts) | 13.6, 13.10 |
| Req 10: Lint Diagnostics | AC 7 (invalid DSN) | 13.7, 13.10 |
| Req 10: Lint Diagnostics | AC 8 (invalid symbolic) | 13.8, 13.10 |
| Req 10: Lint Diagnostics | AC 9 (diagnostic fields) | 1.5, 13.9, 13.10 |
| Req 10: Lint Diagnostics | AC 10 (severity filter) | 13.1, 13.10 |
| Req 11: Resolution Panel | AC 1 (panel registration) | 15.1, 15.9 |
| Req 11: Resolution Panel | AC 2 (table columns) | 15.2, 15.9 |
| Req 11: Resolution Panel | AC 3 (success display) | 15.2, 15.9 |
| Req 11: Resolution Panel | AC 4 (error display) | 15.2, 15.9 |
| Req 11: Resolution Panel | AC 5 (summary header) | 15.1, 15.9 |
| Req 11: Resolution Panel | AC 6 (navigate to source) | 15.7, 15.9 |
| Req 11: Resolution Panel | AC 7 (sort/filter) | 15.4, 15.5, 15.9 |
| Req 11: Resolution Panel | AC 8 (persistence) | 15.8, 15.9 |
| Req 11: Resolution Panel | AC 9 (substitution tooltip) | 15.2, 15.9 |
| Req 11: Resolution Panel | AC 10 (concatenation groups) | 15.6, 15.9 |
| Req 12: Job Structure | AC 1 (JOB statement) | 4.2, 4.7 |
| Req 12: Job Structure | AC 2 (EXEC statement) | 4.3, 4.7 |
| Req 12: Job Structure | AC 3 (hierarchy) | 4.1, 4.7 |
| Req 12: Job Structure | AC 4 (PROC/PEND) | 4.4, 4.7 |
| Req 12: Job Structure | AC 5 (DD overrides) | 4.4, 4.7 |
| Req 12: Job Structure | AC 6 (IF/THEN/ELSE) | 4.5, 4.7 |
| Req 12: Job Structure | AC 7 (step ordering) | 4.6, 4.7 |
| Req 12: Job Structure | AC 8 (no JOB default) | 4.2, 4.7 |
| Req 13: Pipeline | AC 1 (four stages) | 12.1, 12.9 |
| Req 13: Pipeline | AC 2 (intermediate results) | 12.3, 12.9 |
| Req 13: Pipeline | AC 3 (error isolation) | 12.4, 12.9 |
| Req 13: Pipeline | AC 4 (diagnostic aggregation) | 12.5, 12.9 |
| Req 13: Pipeline | AC 5 (stage timing) | 12.6, 12.9 |
| Req 13: Pipeline | AC 6 (incremental) | 12.7, 12.9 |
| Req 14: Configuration | AC 1 (config keys) | 1.4, 1.7 |
| Req 14: Configuration | AC 2 (symbols table) | 5.10, 1.7 |
| Req 14: Configuration | AC 3 (catalog defaults) | 7.3 |
| Req 14: Configuration | AC 4 (hot-reload) | 16.5, 16.7 |
| Req 14: Configuration | AC 5 (schema registration) | 16.6, 16.7 |
| Req 14: Configuration | AC 6 (default HLQ) | 6.7, 6.9 |
| Req 14: Configuration | AC 7 (catalog search order) | 6.3, 6.9 |
| Req 15: Error Handling | AC 1 (thiserror enum) | 1.3, 1.6 |
| Req 15: Error Handling | AC 2 (structured logging) | 17.1, 17.5 |
| Req 15: Error Handling | AC 3 (catalog failure) | 6.8, 6.9 |
| Req 15: Error Handling | AC 4 (graceful internal) | 17.2, 17.5 |
| Req 15: Error Handling | AC 5 (diagnostics method) | 17.3, 17.5 |
| Req 15: Error Handling | AC 6 (diagnostic codes) | 1.3, 13.9 |
| Req 16: Language Service | AC 1 (language_id check) | 16.1, 16.7 |
| Req 16: Language Service | AC 2 (keyword sets) | 16.2, 16.7 |
| Req 16: Language Service | AC 3 (auto-resolve) | 16.3, 16.7 |
| Req 16: Language Service | AC 4 (programmatic API) | 12.8, 12.9 |
| Req 16: Language Service | AC 5 (hover info) | 16.4, 16.7 |
| Cross-cutting: Performance | 5s / 500 DD | 18.5 |
| Cross-cutting: Thread Safety | Send + Sync | 17.4, 17.5 |
| Cross-cutting: Testability | Mock catalog | 18.3 |
| Cross-cutting: Testability | Parser standalone | 18.4 |

---

## Property-Based Test Summary

| Property | Statement | Task | Validates |
|----------|-----------|------|-----------|
| P1 | DSN validation: generated strings accepted only if ≤44 chars, qualifiers ≤8, alpha/national start, no consecutive dots | 2.8 | Req 10 AC 7, AC 8 |
| P2 | Parser round-trip: generate valid DD statements from model, parse, assert operands match | 3.12 | Req 1 AC 1–6 |
| P3 | Job structure hierarchy: generated multi-step jobs have step count matching EXEC count | 4.8 | Req 12 AC 3, AC 7 |
| P4 | Substitution idempotence: after one pass, no known &symbol remains; second pass is no-op | 5.12 | Req 3 AC 1, AC 9 |
| P5 | Dot terminator: &SYM.suffix → value(SYM) + suffix, dot consumed | 5.13 | Req 3 AC 6 |
| P6 | Catalog resolution consistency: same DSN always resolves to same path | 6.10 | Req 2 AC 1, AC 2 |
| P7 | DISP default: DD without DISP always resolves to (NEW, DELETE) | 7.11 | Req 4 AC 7 |
| P8 | Concatenation ordering: N-component groups preserve declaration order indices | 8.8 | Req 5 AC 3 |
| P9 | Temp dataset isolation: &&-prefixed names never trigger catalog resolution | 9.9 | Req 6 AC 5 |
| P10 | Referback chain depth: chains ≤10 resolve, chains >10 produce depth error | 10.9 | Req 7 AC 6 |
| P11 | GDG intra-job state: (0) always resolves to most recently created generation in prior steps | 11.10 | Req 8 AC 7 |
| P12 | Error isolation: jobs with N DDs always produce N resolution results regardless of errors | 12.10 | Req 13 AC 3 |
| P13 | Diagnostic completeness: N injected errors produce ≥ N diagnostics | 13.11 | Req 10 AC 2, AC 3, AC 6, AC 7 |
| P14 | Command result counts: success + warning + error = total DD count | 14.10 | Req 9 AC 6 |
| P15 | Diagnostic ordering: diagnostics() output always sorted by line_number ascending | 17.6 | Req 15 AC 5 |

---

## Notes

- Tasks 2 and 3 can be partially parallelised (DSN model in task 2 is needed by parser in task 3)
- Tasks 8, 9, 10, and 11 are independent of each other and can be implemented in parallel once task 6 (catalog bridge) and task 5 (substitution) are complete
- The `CatalogResolver` trait (task 6.1) enables all testing without real SQLite catalogs
- All property tests use the `proptest` crate with a minimum of 100 iterations
- The mock catalog for testing should be defined in `tests/mock_catalog.rs` shared across integration tests
- `ff-dataset-catalog` types are consumed via the trait boundary — only task 6.2 is tightly coupled to catalog internals
- All tests use `#[test]` (synchronous) unless async behaviour is under test

## Task Dependency Graph

```json
{
  "waves": [
    {
      "id": 0,
      "label": "Project scaffold, error types, config, and diagnostic model",
      "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7"]
    },
    {
      "id": 1,
      "label": "DSN model and DD operand models",
      "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7", "2.8"],
      "dependsOn": [0]
    },
    {
      "id": 2,
      "label": "DD statement parser and job structure parsing",
      "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7", "3.8", "3.9", "3.10", "3.11", "3.12", "4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "4.8"],
      "dependsOn": [1]
    },
    {
      "id": 3,
      "label": "Symbolic substitution engine and symbol table",
      "tasks": ["5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8", "5.9", "5.10", "5.11", "5.12", "5.13"],
      "dependsOn": [2]
    },
    {
      "id": 4,
      "label": "Catalog resolution bridge and allocation simulator",
      "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "6.8", "6.9", "6.10", "7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "7.8", "7.9", "7.10", "7.11"],
      "dependsOn": [3]
    },
    {
      "id": 5,
      "label": "Concatenation, temp datasets, referbacks, and GDG resolution",
      "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8", "9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "9.8", "9.9", "10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8", "10.9", "11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7", "11.8", "11.9", "11.10"],
      "dependsOn": [4]
    },
    {
      "id": 6,
      "label": "Resolution pipeline and lint emitter",
      "tasks": ["12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7", "12.8", "12.9", "12.10", "13.1", "13.2", "13.3", "13.4", "13.5", "13.6", "13.7", "13.8", "13.9", "13.10", "13.11"],
      "dependsOn": [5]
    },
    {
      "id": 7,
      "label": "RESOLVE command, panel model, language integration, and error handling",
      "tasks": ["14.1", "14.2", "14.3", "14.4", "14.5", "14.6", "14.7", "14.8", "14.9", "14.10", "15.1", "15.2", "15.3", "15.4", "15.5", "15.6", "15.7", "15.8", "15.9", "16.1", "16.2", "16.3", "16.4", "16.5", "16.6", "16.7", "17.1", "17.2", "17.3", "17.4", "17.5", "17.6"],
      "dependsOn": [6]
    },
    {
      "id": 8,
      "label": "Integration tests and performance validation",
      "tasks": ["18.1", "18.2", "18.3", "18.4", "18.5"],
      "dependsOn": [7]
    }
  ]
}
```
