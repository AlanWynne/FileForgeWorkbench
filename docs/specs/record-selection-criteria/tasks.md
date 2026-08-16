# Implementation Plan: Record Selection Criteria (`ff-record-criteria`)

## Overview

This plan covers the complete implementation of the `ff-record-criteria` crate — the field-level record filtering engine for FileForgeWorkbench. The crate provides the data model, evaluation logic, persistence, and command integration for selection criteria that control which records are displayed in Grid_Edit_Mode and Grid_Browse_Mode when FileForge_Mode is active.

This is a **Wave 12 (FileForge Domain)** sub-project that depends on `ff-fileforge` (Wave 12) for field extraction and packed-decimal decoding, `ff-structure-catalog` (Wave 12) for structure definitions, `ff-command` (Wave 2) for command registration, `ff-config` (Wave 2) for configuration, `ff-document-model` (Wave 4) for record byte access, and `ff-find-replace` (Wave 5) for criteria scope integration.

---

## Tasks

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-record-criteria/Cargo.toml` with dependencies (thiserror, serde, serde_json, regex, proptest dev-dep) and deps on `ff-fileforge`, `ff-structure-catalog`, `ff-document-model`, `ff-command`, `ff-config`, `ff-logging`
  - [ ] 1.2 Create `crates/ff-record-criteria/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `model.rs`, `evaluator.rs`, `comparison.rs`, `logical.rs`, `wildcard.rs`, `persistence.rs`, `location.rs`, `commands.rs`, `filter_state.rs`, `scope.rs`, `validator.rs`, `config.rs`, `association.rs`, `types.rs`, `error.rs`
  - [ ] 1.4 Add `ff-record-criteria` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [ ] 2. Data model and core types
  - [ ] 2.1 Define `CriteriaOperator` enum (Eq, Ne, Gt, Ge, Lt, Le, Contains, StartsWith, EndsWith, MatchesRegex) with serde rename attributes and Display impl
  - [ ] 2.2 Define `CriteriaConnector` enum (And, Or) with serde rename attributes
  - [ ] 2.3 Define `ComparisonMode` enum (String, Numeric, PackedDecimal) for field-type dispatch
  - [ ] 2.4 Define `Criterion` struct with all fields (enabled, field, operator, value, value2, connector, group_open, group_close) and serde attributes
  - [ ] 2.5 Define `CriteriaSet` struct (name, structure_association, record_type_scope, case_sensitive, criteria vec) with serde attributes
  - [ ] 2.6 Implement `CriteriaSet::empty()`, `CriteriaSet::single()`, `CriteriaSet::enabled_criteria()`
  - [ ] 2.7 Implement `CriteriaSet::to_expression_string()` for status bar display formatting
  - [ ] 2.8 Implement `CriteriaSet::sanitise_name()` for filename derivation
  - [ ] 2.9 Implement `CriteriaSet::from_json()` and `CriteriaSet::to_json()` round-trip serialisation
  - [ ] 2.10 Write unit tests for CriteriaSet construction, expression string formatting, name sanitisation, and JSON round-trip
  - Covers: Requirement 1 (AC 1.1–1.7), Requirement 9 (AC 9.5, 9.6)

- [ ] 3. Error types and validation issues
  - [ ] 3.1 Define `CriteriaError` enum with all variants (FieldNotFound, InvalidRegex, NumericParseFailed, UnmatchedGroup, CriteriaNotFound, ParseFailed, Io, StoreCorrupt, InvalidCommandArg, FileForgeNotActive, InvalidConfig, MaxRowsExceeded, NameCollision) using thiserror derives
  - [ ] 3.2 Define `ValidationIssue` enum (UnknownField, UnmatchedGroup, InvalidRegex, TypeMismatch, NestingDepthExceeded, MaxRowsExceeded) with #[non_exhaustive]
  - [ ] 3.3 Define `CriteriaResult` and `RowResult` structs for per-record evaluation output
  - [ ] 3.4 Write unit tests for error Display formatting and ValidationIssue construction
  - Covers: Requirement 2 (AC 2.9, 2.12), Requirement 5 (AC 5.4), Requirement 9 (AC 9.7)

- [ ] 4. Wildcard matcher
  - [ ] 4.1 Implement `WildcardMatcher::has_wildcards()` detecting unescaped `*` and `?` characters
  - [ ] 4.2 Implement `WildcardMatcher::matches()` with glob-style matching (`*` = zero or more, `?` = exactly one)
  - [ ] 4.3 Implement backslash escape support (`\*` matches literal asterisk, `\?` matches literal question mark)
  - [ ] 4.4 Implement case-insensitive wildcard matching when case_sensitive is false
  - [ ] 4.5 Implement no-wildcard passthrough: when value has no wildcards, behave as exact equality
  - [ ] 4.6 Write unit tests for wildcard patterns, escape sequences, case sensitivity, and passthrough behaviour
  - Covers: Requirement 4 (AC 4.1–4.6)

- [ ] 5. Comparison engine
  - [ ] 5.1 Implement `ComparisonEngine::determine_mode()` mapping field data type to ComparisonMode (int/float → Numeric, packed → PackedDecimal, str/bool → String)
  - [ ] 5.2 Implement numeric comparison: parse field value and criterion value to f64, compare with operator semantics
  - [ ] 5.3 Implement string comparison: EQ/NE as equality, GT/GE/LT/LE as lexicographic ordering
  - [ ] 5.4 Implement case-insensitive string comparison via lowercasing when case_sensitive is false
  - [ ] 5.5 Implement CONTAINS operator: substring check with case sensitivity
  - [ ] 5.6 Implement STARTS_WITH operator: prefix check with case sensitivity
  - [ ] 5.7 Implement ENDS_WITH operator: suffix check with case sensitivity
  - [ ] 5.8 Implement MATCHES_REGEX operator: compile regex pattern, partial match against field value
  - [ ] 5.9 Implement invalid regex handling: return CriteriaError::InvalidRegex with pattern and error detail
  - [ ] 5.10 Implement wildcard integration: when operator is EQ/NE and value has wildcards, delegate to WildcardMatcher
  - [ ] 5.11 Implement numeric parse failure handling: return CriteriaError::NumericParseFailed
  - [ ] 5.12 Implement packed-decimal comparison path (delegate decoding to ff-fileforge's COMP-3 decoder)
  - [ ] 5.13 Implement EBCDIC field handling: convert field value from EBCDIC to display charset before string comparison
  - [ ] 5.14 Write unit tests for all operators across String, Numeric, and PackedDecimal modes, case sensitivity, wildcards, and error cases
  - Covers: Requirement 2 (AC 2.1–2.12), Requirement 3 (AC 3.1–3.6), Requirement 4 (AC 4.1–4.4)

- [ ] 6. Logical combiner
  - [ ] 6.1 Define `LogicalRow` struct (result: bool, connector: Option<CriteriaConnector>, group_open: bool, group_close: bool)
  - [ ] 6.2 Implement `LogicalCombiner::combine()` with AND binding tighter than OR (standard precedence)
  - [ ] 6.3 Implement parenthesised group handling: group_open/group_close override default precedence
  - [ ] 6.4 Implement nested group support up to 8 levels depth
  - [ ] 6.5 Implement unmatched group detection: return error when open/close flags are inconsistent
  - [ ] 6.6 Write unit tests for AND/OR combinations, grouping, nesting depth, precedence override, and error cases
  - Covers: Requirement 5 (AC 5.1–5.6)

- [ ] 7. Criteria evaluator
  - [ ] 7.1 Implement `CriteriaEvaluator::new()` constructing with ComparisonEngine and LogicalCombiner
  - [ ] 7.2 Implement `CriteriaEvaluator::is_passthrough()` detecting empty or all-disabled criteria
  - [ ] 7.3 Implement `CriteriaEvaluator::evaluate()` orchestrating: skip disabled rows, compare each enabled row, collect LogicalRows, combine results
  - [ ] 7.4 Implement per-row evaluation: resolve field value from field_values map, determine ComparisonMode from field_types, invoke ComparisonEngine
  - [ ] 7.5 Implement unknown field handling: treat rows referencing unknown fields as non-matching with ValidationIssue
  - [ ] 7.6 Implement `CriteriaEvaluator::evaluate_all()` for bulk record filtering returning matching indices
  - [ ] 7.7 Implement single-criterion evaluation (no connector logic needed when only one enabled row)
  - [ ] 7.8 Write unit tests for passthrough, single criterion, multi-criteria, disabled rows, unknown fields, and bulk evaluation
  - Covers: Requirement 1 (AC 1.3–1.5), Requirement 7 (AC 7.1–7.2, 7.8)

- [ ] 8. Criteria validator
  - [ ] 8.1 Implement `CriteriaValidator::validate()` checking all validation rules against available field list
  - [ ] 8.2 Implement unknown field detection: report UnknownField for criterion fields not in available_fields
  - [ ] 8.3 Implement `CriteriaValidator::validate_groups()` checking matched group_open/group_close flags
  - [ ] 8.4 Implement nesting depth check: report NestingDepthExceeded when groups nest beyond 8 levels
  - [ ] 8.5 Implement `CriteriaValidator::validate_regex_patterns()` compiling each MATCHES_REGEX value
  - [ ] 8.6 Implement type mismatch detection: check criterion values parseable as numeric for numeric fields
  - [ ] 8.7 Implement max rows check: report MaxRowsExceeded when criteria count exceeds configured limit
  - [ ] 8.8 Write unit tests for all validation scenarios: unknown fields, unmatched groups, invalid regex, type mismatches, depth exceeded, max rows
  - Covers: Requirement 2 (AC 2.9, 2.12), Requirement 5 (AC 5.4), Requirement 7 (AC 7.8), Requirement 10 (AC 10.14)

- [ ] 9. Filter state management
  - [ ] 9.1 Implement `FilterState::inactive()` creating an empty filter state
  - [ ] 9.2 Implement `FilterState::apply()` transitioning to active state with criteria, visible count, and total count
  - [ ] 9.3 Implement `FilterState::clear()` removing active criteria and resetting counts
  - [ ] 9.4 Implement `FilterState::is_active()` returning whether criteria are currently applied
  - [ ] 9.5 Implement `FilterState::format_indicator()` returning `Some("Criteria: <name>")` or `Some("Criteria: active")` when active, None when inactive
  - [ ] 9.6 Implement `FilterState::format_count()` returning `Some("Showing N of M records")` when active
  - [ ] 9.7 Implement Record_Type_Scope display in indicator (e.g., `Criteria: active | Scope: Detail`)
  - [ ] 9.8 Write unit tests for state transitions, indicator formatting, count formatting, and scope display
  - Covers: Requirement 7 (AC 7.12), Requirement 13 (AC 13.1–13.7)

- [ ] 10. Criteria persistence (JSON load/save)
  - [ ] 10.1 Implement `CriteriaPersistence::save()` serialising CriteriaSet to `.criteria.json` file with sanitised filename
  - [ ] 10.2 Implement `CriteriaPersistence::load()` deserialising CriteriaSet from `.criteria.json` file by name
  - [ ] 10.3 Implement parse failure handling: return CriteriaError::ParseFailed for invalid JSON or missing required keys
  - [ ] 10.4 Implement unrecognised operator handling: return error for unknown operator strings
  - [ ] 10.5 Implement `CriteriaPersistence::list()` scanning criteria location directory for all `.criteria.json` files and returning metadata
  - [ ] 10.6 Implement `CriteriaPersistence::delete()` removing a named criteria file
  - [ ] 10.7 Implement `CriteriaPersistence::duplicate()` copying a criteria file under a new name
  - [ ] 10.8 Write unit tests for save/load round-trip, list, delete, duplicate, and error handling with temp directories
  - Covers: Requirement 9 (AC 9.4–9.7), Requirement 11 (AC 11.2–11.7)

- [ ] 11. Criteria location manager
  - [ ] 11.1 Define `CriteriaStore` and `CriteriaLocation` structs with TOML serde support
  - [ ] 11.2 Implement `CriteriaLocationManager::new()` initialising with defaults from CriteriaConfig
  - [ ] 11.3 Implement `CriteriaLocationManager::load()` reading CriteriaStore from TOML file, handling absent/corrupt file gracefully
  - [ ] 11.4 Implement `CriteriaLocationManager::save()` persisting CriteriaStore to TOML file
  - [ ] 11.5 Implement `active_location()` returning the current Active_Criteria_Location path
  - [ ] 11.6 Implement `set_active_location()` changing the active location
  - [ ] 11.7 Implement `add_location()` and `remove_location()` for CRUD on Criteria_Locations
  - [ ] 11.8 Implement default location auto-creation on first use
  - [ ] 11.9 Implement corrupt store recovery: initialise with defaults, emit warning, do not overwrite corrupt file until operator makes a change
  - [ ] 11.10 Write unit tests for store load/save, location CRUD, default initialisation, and corrupt file handling
  - Covers: Requirement 9 (AC 9.1–9.3, 9.8–9.10)

- [ ] 12. Configuration integration
  - [ ] 12.1 Define `CriteriaConfig` struct with fields: store_path, default_location, auto_suggest, max_criteria_rows and Default impl
  - [ ] 12.2 Implement config loading from `[criteria]` TOML namespace via ff-config typed access
  - [ ] 12.3 Implement validation: invalid store_path → use default + WARN; invalid default_location → create + INFO; invalid auto_suggest → default true + WARN; out-of-range max_criteria_rows → clamp to [1, 200] + WARN
  - [ ] 12.4 Implement hot-reload callback registration for `criteria` namespace
  - [ ] 12.5 Write unit tests for config parsing, validation, defaults, and reload semantics
  - Covers: Requirement 14 (AC 14.1–14.6)

- [ ] 13. CRITERIA command registration and parsing
  - [ ] 13.1 Define `CriteriaCommand` enum (OpenPanel, Set{name}, Clear, Show, Save{name}) with `parse()` method
  - [ ] 13.2 Implement command argument parsing: empty → OpenPanel, SET/LOAD <name> → Set, CLEAR → Clear, SHOW/STATUS → Show, SAVE <name> → Save
  - [ ] 13.3 Implement invalid argument error: return CriteriaError::InvalidCommandArg
  - [ ] 13.4 Implement `CriteriaCommandRegistrar::register_commands()` registering `criteria` (alias `select`) with subcommands in command framework
  - [ ] 13.5 Implement command metadata: display name, description, category `"criteria"` for command palette discovery
  - [ ] 13.6 Implement FileForge_Mode guard: when mode not active, trigger Structure_Selector flow; if cancelled, do not open panel
  - [ ] 13.7 Write unit tests for command parsing (all subcommands, aliases, invalid inputs) and registration metadata
  - Covers: Requirement 6 (AC 6.1–6.8)

- [ ] 14. Criteria scope (FIND/CHANGE integration)
  - [ ] 14.1 Implement `CriteriaScope::new()` constructing from a vec of matching record indices
  - [ ] 14.2 Implement `CriteriaScope::contains_record()` checking if a record index is in scope
  - [ ] 14.3 Implement `CriteriaScope::contains_line()` mapping a line to its parent record via `LineToRecordMap` trait
  - [ ] 14.4 Define `LineToRecordMap` trait with `record_for_line(line: usize) -> Option<usize>` method
  - [ ] 14.5 Implement `CriteriaScope::is_effective()` returning true when not all records match (scope has filtering effect)
  - [ ] 14.6 Implement no-criteria-active passthrough: CRITERIA modifier has no effect when no criteria are active
  - [ ] 14.7 Write unit tests for scope containment, line mapping, effectiveness check, and passthrough behaviour
  - Covers: Requirement 8 (AC 8.1–8.7)

- [ ] 15. Structure association and auto-suggestion
  - [ ] 15.1 Implement `StructureAssociation::find_matching()` scanning Active_Criteria_Location for criteria sets with matching structure_association
  - [ ] 15.2 Implement case-insensitive structure name matching
  - [ ] 15.3 Implement `StructureAssociation::most_recent_match()` returning the most recently modified matching set
  - [ ] 15.4 Implement multi-match handling: when multiple sets match, return all for picker display
  - [ ] 15.5 Implement auto-suggest disabled check: skip suggestion when config auto_suggest is false
  - [ ] 15.6 Implement session history criteria recording: store criteria name or expression in session entry
  - [ ] 15.7 Implement session restore prompt: offer to restore previous criteria when reopening a file
  - [ ] 15.8 Implement missing named set handling: display message when a named set no longer exists in catalog
  - [ ] 15.9 Write unit tests for matching logic, multi-match, disabled auto-suggest, and missing set scenarios
  - Covers: Requirement 12 (AC 12.1–12.8)

- [ ] 16. Grid display filtering integration
  - [ ] 16.1 Implement record-level filter application: evaluate each record and expose matching indices to grid display layer
  - [ ] 16.2 Implement Record_Type_Scope filtering: apply criteria only to records of specified type; pass through other types unfiltered
  - [ ] 16.3 Implement ALL TYPES scope: apply criteria to all records regardless of type
  - [ ] 16.4 Implement conjunctive filter combination: when Criteria_Set, Record_Filter, and Record_Type_Filter are all active, return intersection
  - [ ] 16.5 Implement structure change handling: clear Active_Criteria_Set when Structure_Definition changes
  - [ ] 16.6 Implement unknown field graceful degradation: disable criterion rows referencing unknown fields with warning
  - [ ] 16.7 Implement SAVE safety: criteria filter affects only display; save operations write all records to original positions
  - [ ] 16.8 Write unit tests for scope filtering, conjunctive combination, structure change clearing, and unknown field handling
  - Covers: Requirement 7 (AC 7.1–7.11)

- [ ] 17. Property-based tests
  - [ ] 17.1 Write PBT: empty/all-disabled criteria passthrough (Property 1)
  - [ ] 17.2 Write PBT: disabled row skip equivalence (Property 2)
  - [ ] 17.3 Write PBT: EQ/NE symmetry (Property 3)
  - [ ] 17.4 Write PBT: ordering consistency GT/GE/LT/LE (Property 4)
  - [ ] 17.5 Write PBT: case sensitivity toggle (Property 5)
  - [ ] 17.6 Write PBT: wildcard no-op without pattern characters (Property 6)
  - [ ] 17.7 Write PBT: logical AND strictness (Property 7)
  - [ ] 17.8 Write PBT: logical OR leniency (Property 8)
  - [ ] 17.9 Write PBT: group override precedence (Property 9)
  - [ ] 17.10 Write PBT: JSON round-trip preservation (Property 10)
  - [ ] 17.11 Write PBT: filter state indicator consistency (Property 11)
  - [ ] 17.12 Write PBT: criteria scope record containment (Property 12)
  - Covers: Requirements 1–5, 8, 13 (see Property-Based Test Definitions below)

- [ ] 18. Integration tests
  - [ ] 18.1 Write integration test: full criteria lifecycle — define → validate → evaluate → filter state update → indicator formatting
  - [ ] 18.2 Write integration test: persistence cycle — save → list → load → duplicate → delete
  - [ ] 18.3 Write integration test: command dispatch — register commands, invoke CRITERIA SET/CLEAR/SHOW/SAVE, verify filter state mutations
  - [ ] 18.4 Write integration test: FIND/CHANGE scope — create CriteriaScope from evaluator results, verify line containment against record mapping
  - [ ] 18.5 Write integration test: config hot-reload — simulate config change mid-session, verify settings applied without restart
  - [ ] 18.6 Write integration test: structure association — activate structure, find matching criteria, auto-suggest flow
  - Covers: End-to-end validation across Requirements 1–14

---

## Property-Based Test Definitions

### Property 1: Empty/All-Disabled Criteria Passthrough

**Validates: Requirements 1.4, 1.5**

- **Statement:** When a CriteriaSet is empty or all criteria rows are disabled, evaluation returns `matches: true` for every record (no filtering occurs).
- **Strategy:** Generate:
  - CriteriaSets: either empty criteria vec, or vec of 1–10 Criterion rows all with `enabled: false`
  - Records: arbitrary field value maps (1–20 fields with random string/numeric values)
  - Field types: random assignment of String/Numeric/PackedDecimal per field
- **Invariant:** `CriteriaEvaluator::evaluate(&cs, &fields, &types).matches == true` for all generated records

### Property 2: Disabled Row Skip Equivalence

**Validates: Requirements 1.5**

- **Statement:** Evaluating a CriteriaSet with a disabled row produces the same result as evaluating the CriteriaSet with that row removed entirely.
- **Strategy:** Generate:
  - CriteriaSets: 2–8 Criterion rows with valid fields, operators, and connectors
  - One random row index to disable
  - Records: field value maps covering the referenced fields
- **Invariant:** `evaluate(cs_with_disabled_row, record).matches == evaluate(cs_with_row_removed, record).matches`

### Property 3: Operator Correctness — EQ Symmetry with NE

**Validates: Requirements 2.2, 2.3**

- **Statement:** For any field value and criterion value, `EQ` returns the logical negation of `NE` (and vice versa), regardless of comparison mode.
- **Strategy:** Generate:
  - Field values: arbitrary strings (0–100 chars) and numeric strings
  - Criterion values: arbitrary strings (0–100 chars) and numeric strings
  - ComparisonMode: random (String, Numeric)
  - Case sensitive: random boolean
- **Invariant:** `compare(v, c, EQ, mode, cs) == !compare(v, c, NE, mode, cs)` for all inputs where both comparisons succeed (no parse errors)

### Property 4: Ordering Consistency (GT/GE/LT/LE)

**Validates: Requirements 2.4**

- **Statement:** The ordering operators form a consistent total order. For any two values, exactly one of GT, EQ, LT holds, and GE ≡ GT ∨ EQ, LE ≡ LT ∨ EQ.
- **Strategy:** Generate:
  - Field values: numeric strings and arbitrary alpha strings
  - Criterion values: matching type strings
  - ComparisonMode: String or Numeric (consistent with value types)
  - Case sensitive: random boolean
- **Invariant:** `(eq as u8 + gt as u8 + lt as u8) == 1` AND `ge == (gt || eq)` AND `le == (lt || eq)`

### Property 5: Case Sensitivity Toggle

**Validates: Requirements 2.10, 2.11**

- **Statement:** When case_sensitive is false, string comparison results are identical regardless of the case of the input values.
- **Strategy:** Generate:
  - Field values: arbitrary ASCII/Unicode strings (1–50 chars)
  - Criterion values: arbitrary ASCII/Unicode strings (1–50 chars)
  - Operators: random from {EQ, NE, CONTAINS, STARTS_WITH, ENDS_WITH}
- **Invariant:** `compare(v, c, op, String, false) == compare(v.to_lowercase(), c.to_lowercase(), op, String, true)`

### Property 6: Wildcard No-Op Without Pattern Characters

**Validates: Requirements 4.4**

- **Statement:** When a criterion value contains no wildcard characters (`*`, `?`), EQ with that value produces the same result as exact equality.
- **Strategy:** Generate:
  - Field values: arbitrary strings (0–100 chars) not containing `*` or `?`
  - Criterion values: arbitrary strings (0–100 chars) not containing `*` or `?`
  - Case sensitive: random boolean
- **Invariant:** `WildcardMatcher::matches(v, c, cs) == (normalize(v) == normalize(c))` where normalize applies lowercasing if not case-sensitive

### Property 7: Logical AND Strictness

**Validates: Requirements 5.1**

- **Statement:** Combining two criterion results with AND produces true only when both individual results are true.
- **Strategy:** Generate:
  - Boolean pairs (a, b): all four combinations via proptest booleans
- **Invariant:** `combine([LogicalRow{result: a, connector: Some(And)}, LogicalRow{result: b, connector: None}]) == (a && b)`

### Property 8: Logical OR Leniency

**Validates: Requirements 5.1**

- **Statement:** Combining two criterion results with OR produces true when at least one individual result is true.
- **Strategy:** Generate:
  - Boolean pairs (a, b): all four combinations via proptest booleans
- **Invariant:** `combine([LogicalRow{result: a, connector: Some(Or)}, LogicalRow{result: b, connector: None}]) == (a || b)`

### Property 9: Group Override Precedence

**Validates: Requirements 5.2, 5.3**

- **Statement:** Parenthesised groups override default AND/OR precedence. `A OR (B AND C)` evaluates the group first.
- **Strategy:** Generate:
  - Boolean triples (a, b, c): all eight combinations via proptest booleans
- **Invariant:** `combine([row(a, Or, false, false), row(b, And, true, false), row(c, None, false, true)]) == (a || (b && c))`

### Property 10: JSON Round-Trip Preservation

**Validates: Requirements 1.6**

- **Statement:** Serialising a CriteriaSet to JSON and deserialising back produces an identical CriteriaSet.
- **Strategy:** Generate:
  - CriteriaSets: arbitrary well-formed sets with 0–10 criteria rows, random operators, connectors, field names (alphanumeric 1–30 chars), values, group flags, case_sensitive flag, optional name and structure_association
- **Invariant:** `CriteriaSet::from_json(&cs.to_json().unwrap()).unwrap() == cs`

### Property 11: Filter State Indicator Consistency

**Validates: Requirements 13.1, 13.2**

- **Statement:** `FilterState::format_indicator()` returns `Some(...)` if and only if a CriteriaSet is active.
- **Strategy:** Generate:
  - FilterState instances: some with applied criteria (random names, counts), some inactive
- **Invariant:** `fs.format_indicator().is_some() == fs.is_active()`

### Property 12: Criteria Scope Record Containment

**Validates: Requirements 8.1, 8.6**

- **Statement:** A CriteriaScope constructed from matching record indices correctly reports containment for exactly those indices and no others.
- **Strategy:** Generate:
  - Index sets: sorted unique Vec<usize> of size 0–500, values in range [0, 1000]
  - Query indices: random usize values in range [0, 1000]
- **Invariant:** `CriteriaScope::new(indices.clone()).contains_record(query) == indices.contains(&query)`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and Errors", "tasks": ["2", "3"], "dependsOn": [0] },
    { "id": 2, "label": "Wildcard Engine", "tasks": ["4"], "dependsOn": [1] },
    { "id": 3, "label": "Comparison and Logic", "tasks": ["5", "6"], "dependsOn": [2] },
    { "id": 4, "label": "Evaluator and Validator", "tasks": ["7", "8"], "dependsOn": [3] },
    { "id": 5, "label": "State and Persistence", "tasks": ["9", "10", "11", "12"], "dependsOn": [4] },
    { "id": 6, "label": "Commands and Scope", "tasks": ["13", "14"], "dependsOn": [5] },
    { "id": 7, "label": "Integration Features", "tasks": ["15", "16"], "dependsOn": [6] },
    { "id": 8, "label": "Validation and PBT", "tasks": ["17", "18"], "dependsOn": [7] }
  ]
}
```

---

## Notes

- This is a Wave 12 (FileForge Domain) crate with multiple upstream dependencies across earlier waves
- The Criteria_Panel and Criteria_Catalog_Dialog UI rendering is shell-side (ff-desktop) — this crate provides only the data model, evaluation, and persistence logic
- The `ff-find-replace` crate is a downstream consumer of `CriteriaScope` — the interface is defined in this crate but consumed there
- Status bar indicator rendering is read by `menu-and-statusbar` from `FilterState` — this crate exposes the formatted strings
- Property-based tests use the `proptest` crate with a minimum of 256 iterations per property
- All file I/O in persistence tests uses `tempfile::TempDir` for isolation
- EBCDIC and packed-decimal decoding delegate to `ff-fileforge` — this crate does not implement encoding logic
- Configuration hot-reload leverages `ff-config`'s callback mechanism registered at crate initialisation

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Criteria_Set Definition | AC 1.1–1.7 | Tasks 2, 7 |
| Req 2: Comparison Operators | AC 2.1–2.12 | Tasks 5, 8 |
| Req 3: Field-Type-Aware Comparison | AC 3.1–3.6 | Task 5 |
| Req 4: Wildcard Support | AC 4.1–4.6 | Tasks 4, 5 |
| Req 5: Logical Combination (AND/OR Groups) | AC 5.1–5.6 | Tasks 6, 8 |
| Req 6: CRITERIA Primary Command | AC 6.1–6.8 | Task 13 |
| Req 7: Criteria Applied to Grid Display | AC 7.1–7.12 | Tasks 7, 9, 16 |
| Req 8: Criteria Applied to FIND/CHANGE Scope | AC 8.1–8.7 | Task 14 |
| Req 9: Criteria Persistence | AC 9.1–9.10 | Tasks 10, 11 |
| Req 10: Criteria UI Panel | AC 10.1–10.14 | Tasks 8, 13 (logic only; rendering is shell-side 🔲 MANUAL) |
| Req 11: Criteria Catalog Dialog | AC 11.1–11.10 | Tasks 10, 11 (logic only; dialog rendering is shell-side 🔲 MANUAL) |
| Req 12: Structure Association and Auto-Suggestion | AC 12.1–12.8 | Task 15 |
| Req 13: Status Bar Integration | AC 13.1–13.7 | Task 9 |
| Req 14: Configuration Integration | AC 14.1–14.6 | Task 12 |
