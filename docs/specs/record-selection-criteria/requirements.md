# Requirements Document

## Introduction

This feature specifies the **Record Selection Criteria** system for FileForgeWorkbench (`ff-criteria` crate). When FileForge_Mode is active (a Structure_Definition is associated with the open file), the user can define field-level filter criteria that control which records are displayed in the grid. Criteria are composed into logical expressions using AND/OR connectors, evaluated in the display layer without modifying the source file, and can be saved and reloaded from a persistent **Criteria_Catalog** managed through the configuration system.

The feature covers seven tightly related capabilities:

1. **Criteria_Set definition** — field-based filter expressions with comparison operators, logical connectors, and grouping
2. **Comparison operators** — EQ, NE, GT, GE, LT, LE, CONTAINS, STARTS_WITH, ENDS_WITH, MATCHES_REGEX, plus wildcard support
3. **Logical combination** — AND/OR groups with parenthesised sub-expressions and standard precedence
4. **CRITERIA primary command** — CRITERIA SET/CLEAR/SHOW dispatched through the command framework
5. **Criteria applied to Grid_Edit_Mode display** — filter rows in real time without modifying the file
6. **Criteria applied to FIND/CHANGE scope** — restrict find-and-replace operations to criteria-matching records
7. **Criteria persistence** — named criteria sets saved to a Criteria_Catalog with structure association

Additionally, the system provides:
- **Criteria UI panel** — an interactive builder for constructing filter expressions visually
- **Field-type-aware comparison** — numeric, string, and packed-decimal fields compared using appropriate semantics
- **Wildcard support** — glob-style wildcards (`*`, `?`) in string comparison values

This spec extends the `structure-catalog` and `fileforge-integration` specs. The `file_forge` crate continues to own all record parsing and field extraction. Criteria evaluation is performed in the display layer, not written to the source file. The criteria engine integrates with the `find-and-replace` engine to scope FIND/CHANGE operations.

**Source references:**
- **[FFE-CRITERIA]** = FileForgeEditor `record-selection-criteria` spec (Requirements 1–12)
- **[WB]** = Workbench Platform Architecture Brief (command-driven, plugin-capable, GUI-independent)

## Cross-References

- **`fileforge-integration`** — Provides FileForge_Mode, Structure_Definition, Record_Structure, field extraction, EBCDIC/COMP-3 decoding
- **`structure-catalog`** — Provides the Structure_Catalog, Grid_Edit_Mode, Grid_Browse_Mode, Record_Filter, Record_Type_Filter, Matching_Record, Non_Matching_Record
- **`find-and-replace`** — Criteria scope integration; FIND/CHANGE can be restricted to criteria-matching records
- **`command-framework`** — CRITERIA command registration, dispatch, metadata, undo integration
- **`configuration-system`** — Criteria_Catalog path configuration, hot-reload of criteria store settings
- **`layout-and-docking`** — Criteria panel can be docked as a side panel or floated

---

## Glossary

- **Criteria_Set**: An ordered list of Criterion rows that together form a complete filter expression. Named, saved, and loaded as a unit. [FFE-CRITERIA]
- **Criterion**: A single filter rule consisting of a field name, an operator, one or two values, an AND/OR connector, an enabled/disabled flag, and optional group-open/group-close markers. [FFE-CRITERIA]
- **Criteria_Operator**: A comparison operator applicable to a field value. Standard set: `EQ` (`=`), `NE` (`<>`), `GT` (`>`), `GE` (`>=`), `LT` (`<`), `LE` (`<=`), `CONTAINS`, `STARTS_WITH`, `ENDS_WITH`, `MATCHES_REGEX`. [FFE-CRITERIA, WB]
- **Criteria_Connector**: The logical connector joining one Criterion row to the next: `AND` or `OR`. The last row has no connector. [FFE-CRITERIA]
- **Criteria_Catalog**: The operator-managed directory (or set of directories) containing named `.criteria.json` files, each representing one saved Criteria_Set. [FFE-CRITERIA]
- **Criteria_Location**: The filesystem path to a Criteria_Catalog directory. Multiple locations can be configured; one is designated as the Active_Criteria_Location. [FFE-CRITERIA]
- **Criteria_Store**: The TOML/JSON configuration file recording all known Criteria_Locations, the Active_Criteria_Location, and structure association hints. Stored in the configuration system's user layer. [FFE-CRITERIA, WB]
- **Active_Criteria_Location**: The currently selected Criteria_Location from which criteria sets are loaded and to which new criteria sets are saved. [FFE-CRITERIA]
- **Active_Criteria_Set**: The Criteria_Set currently applied to the session. Its name is shown in the status bar. [FFE-CRITERIA]
- **Criteria_Panel**: The interactive builder panel (dockable or floating) for defining, enabling, disabling, and applying selection criteria rows. [FFE-CRITERIA, WB]
- **Criteria_Catalog_Dialog**: The dialog for browsing, creating, loading, duplicating, and deleting saved Criteria_Sets. [FFE-CRITERIA]
- **Criteria_Active_Indicator**: The status bar element showing criteria state (e.g., `Criteria: <name>` or `Criteria: active`). [FFE-CRITERIA]
- **Structure_Association**: An optional field in a saved `.criteria.json` recording the Structure_Definition name the criteria set was designed for. [FFE-CRITERIA]
- **Record_Type_Scope**: An optional setting restricting criteria evaluation to records of a specific Record_Structure type within a multi-type file. [FFE-CRITERIA]
- **Numeric_Comparison**: Criteria evaluation path for numeric fields (`int`, `float`, packed-decimal) where values are converted to decimal form before comparison. [FFE-CRITERIA]
- **Packed_Decimal_Comparison**: Criteria evaluation path for COMP-3 (packed-decimal) fields where the packed bytes are decoded to numeric value via the `file_forge` crate before comparison. [WB]
- **Wildcard_Pattern**: A glob-style pattern using `*` (match zero or more characters) and `?` (match exactly one character) in string comparison values. [WB]
- **Case_Sensitive_Flag**: A per-Criteria_Set boolean (default `false`) controlling case sensitivity for string comparisons. [FFE-CRITERIA]
- **Criteria_Scope**: A SearchScope modifier for the find-and-replace engine that restricts FIND/CHANGE operations to records currently passing the Active_Criteria_Set filter. [WB]

---

## Requirements

### Requirement 1: Criteria_Set Definition

**User Story:** As an editor user in FileForge mode, I want to define field-based filter expressions composed of one or more criteria rows, so that I can precisely select which records are displayed in the grid.

**Source:** [FFE-CRITERIA Req 1, 3]

#### Acceptance Criteria

1. A Criteria_Set SHALL consist of an ordered list of zero or more Criterion rows, a Case_Sensitive_Flag (default `false`), and an optional Record_Type_Scope. [FFE-CRITERIA]
2. EACH Criterion row SHALL contain the following fields: enabled (boolean), field name (string referencing a field in the active Record_Structure), operator (Criteria_Operator enum), value (string), value2 (string, used for range operators), connector (AND/OR, null on the last row), group_open (boolean), and group_close (boolean). [FFE-CRITERIA]
3. WHEN only one Criterion row is defined and enabled, THE Criteria_Evaluator SHALL evaluate that single criterion with no connector logic. [FFE-CRITERIA]
4. WHEN all Criterion rows are disabled or the Criteria_Set is empty, THE Criteria_Evaluator SHALL skip filtering entirely, returning all records without any filter constraint applied. [FFE-CRITERIA]
5. WHEN a Criterion row's enabled flag is `false`, THE Criteria_Evaluator SHALL skip that row entirely as if it were not present in the Criteria_Set. [FFE-CRITERIA]
6. THE Criteria_Set SHALL be serialisable to JSON for persistence and deserialised from JSON for loading. [FFE-CRITERIA]
7. THE Criteria_Set SHALL be representable as a displayable expression string for status bar and command output (e.g., `FIELD1 EQ 'ABC' AND FIELD2 GT '100'`). [WB]

---

### Requirement 2: Comparison Operators

**User Story:** As an editor user, I want a full set of comparison operators including pattern matching and range checks, so that I can express precise selection criteria for both numeric and string fields.

**Source:** [FFE-CRITERIA Req 2], [WB]

#### Acceptance Criteria

1. THE Criteria_Evaluator SHALL support the following comparison operators: `EQ` (equals), `NE` (not equals), `GT` (greater than), `GE` (greater than or equal), `LT` (less than), `LE` (less than or equal), `CONTAINS` (substring match), `STARTS_WITH` (prefix match), `ENDS_WITH` (suffix match), `MATCHES_REGEX` (regular expression match). [FFE-CRITERIA, WB]
2. WHEN the operator is `EQ`, THE Criteria_Evaluator SHALL return true if the field value equals the criterion value, using the appropriate comparison mode (numeric or string) based on field type. [FFE-CRITERIA]
3. WHEN the operator is `NE`, THE Criteria_Evaluator SHALL return true if the field value does not equal the criterion value. [FFE-CRITERIA]
4. WHEN the operator is `GT`, `GE`, `LT`, or `LE`, THE Criteria_Evaluator SHALL perform an ordered comparison using numeric semantics for numeric fields and lexicographic semantics for string fields. [FFE-CRITERIA]
5. WHEN the operator is `CONTAINS`, THE Criteria_Evaluator SHALL return true if the field value contains the criterion value as a substring. [FFE-CRITERIA]
6. WHEN the operator is `STARTS_WITH`, THE Criteria_Evaluator SHALL return true if the field value begins with the criterion value. [FFE-CRITERIA]
7. WHEN the operator is `ENDS_WITH`, THE Criteria_Evaluator SHALL return true if the field value ends with the criterion value. [FFE-CRITERIA]
8. WHEN the operator is `MATCHES_REGEX`, THE Criteria_Evaluator SHALL interpret the criterion value as a regular expression pattern and return true if the field value matches the pattern (partial match — the pattern need not match the entire field value). [WB]
9. IF a `MATCHES_REGEX` criterion value is not a valid regex pattern, THE Criteria_Evaluator SHALL treat the criterion as not matching and display a validation error in the Criteria_Panel identifying the invalid pattern. [WB]
10. WHEN the Case_Sensitive_Flag is `false` (the default), THE Criteria_Evaluator SHALL perform all string comparisons (`EQ`, `NE`, `CONTAINS`, `STARTS_WITH`, `ENDS_WITH`, `MATCHES_REGEX`) case-insensitively. [FFE-CRITERIA]
11. WHEN the Case_Sensitive_Flag is `true`, THE Criteria_Evaluator SHALL perform all string comparisons case-sensitively. [FFE-CRITERIA]
12. WHEN a criterion value cannot be converted to the expected numeric type and the operator requires Numeric_Comparison, THE Criteria_Evaluator SHALL treat the criterion as not matching and display a validation warning in the Criteria_Panel. [FFE-CRITERIA]

---

### Requirement 3: Field-Type-Aware Comparison

**User Story:** As an editor user working with mainframe-origin data, I want criteria comparisons to respect field data types including packed-decimal (COMP-3) fields, so that numeric ordering is correct regardless of the underlying encoding.

**Source:** [FFE-CRITERIA Req 2], [WB]

#### Acceptance Criteria

1. WHEN a field's `data_type` is `int` or `float`, THE Criteria_Evaluator SHALL perform Numeric_Comparison by converting both the field value and the criterion value to decimal form before comparison. [FFE-CRITERIA]
2. WHEN a field's `data_type` is `packed` (COMP-3 packed-decimal), THE Criteria_Evaluator SHALL decode the packed bytes to a numeric value using the `file_forge` crate's packed-decimal decoder, then perform Numeric_Comparison against the criterion value. [WB]
3. WHEN a field's `data_type` is `str` or `bool`, THE Criteria_Evaluator SHALL perform string comparison (lexicographic ordering for GT/GE/LT/LE, equality for EQ/NE). [FFE-CRITERIA]
4. WHEN a field's `data_type` indicates EBCDIC encoding, THE Criteria_Evaluator SHALL convert the field value from EBCDIC to the display character set before performing string comparison operations. [WB]
5. WHEN a field value is entirely numeric (digits, optional sign, optional decimal point) but the field's declared `data_type` is `str`, THE Criteria_Evaluator SHALL still perform string comparison — the declared type takes precedence over inferred content. [WB]
6. THE Criteria_Panel SHALL display the detected comparison mode (Numeric, String, Packed-Decimal) next to the operator dropdown when a field is selected, providing user feedback on how the comparison will be evaluated. [WB]

---

### Requirement 4: Wildcard Support

**User Story:** As an editor user, I want to use glob-style wildcards in criterion values for string operators, so that I can match patterns without needing full regular expressions.

**Source:** [WB]

#### Acceptance Criteria

1. WHEN the operator is `EQ` or `NE` and the criterion value contains wildcard characters (`*` or `?`), THE Criteria_Evaluator SHALL interpret the value as a Wildcard_Pattern where `*` matches zero or more characters and `?` matches exactly one character. [WB]
2. THE wildcard matching SHALL apply only to string-type comparisons; numeric fields SHALL NOT interpret `*` or `?` as wildcards. [WB]
3. WHEN the Case_Sensitive_Flag is `false`, THE wildcard matching SHALL be case-insensitive. [WB]
4. WHEN the criterion value contains no wildcard characters, THE `EQ` and `NE` operators SHALL perform exact equality comparison (no pattern interpretation). [WB]
5. THE Criteria_Panel SHALL indicate when a criterion value is being interpreted as a wildcard pattern (e.g., a small icon or tooltip on the value field). [WB]
6. WHEN the user needs to match a literal `*` or `?` character, THE Criteria_Evaluator SHALL support escaping with a backslash (`\*` matches literal asterisk, `\?` matches literal question mark). [WB]

---

### Requirement 5: Logical Combination (AND/OR Groups)

**User Story:** As an editor user, I want to combine multiple criteria with AND/OR connectors and parenthesised grouping, so that I can express complex filter expressions with correct logical precedence.

**Source:** [FFE-CRITERIA Req 3]

#### Acceptance Criteria

1. THE Criteria_Evaluator SHALL combine Criterion rows using their Criteria_Connectors, evaluating the expression from top to bottom with standard logical precedence (AND binds tighter than OR) unless overridden by grouping. [FFE-CRITERIA]
2. WHEN a Criterion row has its group_open flag set, THE Criteria_Evaluator SHALL treat the sub-expression starting at that row as a parenthesised group with higher precedence than surrounding connectors. [FFE-CRITERIA]
3. WHEN a Criterion row has its group_close flag set, THE Criteria_Evaluator SHALL close the current parenthesised group at that row. [FFE-CRITERIA]
4. WHEN group_open or group_close flags are unmatched (the group structure is inconsistent), THE system SHALL display a validation error in the Criteria_Panel and SHALL NOT apply the filter until the grouping is corrected. [FFE-CRITERIA]
5. THE Criteria_Panel SHALL visually indicate group nesting depth using indentation or colour-coded brackets, so the user can see the logical structure of the expression. [WB]
6. THE Criteria_Panel SHALL support up to 8 levels of nested grouping. [WB]

---

### Requirement 6: CRITERIA Primary Command

**User Story:** As a keyboard-driven user, I want to control selection criteria from the command line using CRITERIA SET/CLEAR/SHOW subcommands, so that I can apply and manage filters without the GUI panel.

**Source:** [FFE-CRITERIA Req 11], [WB]

#### Acceptance Criteria

1. THE command framework SHALL register the command `criteria` (alias `select`) with subcommands routed through the command dispatch system. [FFE-CRITERIA, WB]
2. WHEN `CRITERIA` is issued with no subcommand and FileForge_Mode is active, THE system SHALL open the Criteria_Panel. [FFE-CRITERIA]
3. WHEN `CRITERIA SET <name>` (or `CRITERIA LOAD <name>`) is issued, THE system SHALL load the named Criteria_Set from the Active_Criteria_Location and apply it immediately. IF the name is not found, THE system SHALL display an error listing available criteria set names. [FFE-CRITERIA]
4. WHEN `CRITERIA CLEAR` is issued, THE system SHALL remove the Active_Criteria_Set, display all records, and remove the Criteria_Active_Indicator from the status bar. [FFE-CRITERIA]
5. WHEN `CRITERIA SHOW` (or `CRITERIA STATUS`) is issued, THE system SHALL display the name of the Active_Criteria_Set (or a summary of the unsaved expression) in the command result area without opening the panel. [FFE-CRITERIA]
6. WHEN `CRITERIA SAVE <name>` is issued, THE system SHALL save the current in-memory Criteria_Set to the Active_Criteria_Location under the given name, overwriting an existing file of the same name without a GUI prompt. [FFE-CRITERIA]
7. WHEN the `CRITERIA` or `SELECT` command is issued and FileForge_Mode is not active, THE system SHALL trigger the Structure_Selector flow (as defined in the `structure-catalog` spec) so the user can activate FileForge_Mode before the criteria panel opens. IF the user cancels, THE system SHALL not open the Criteria_Panel. [FFE-CRITERIA]
8. ALL CRITERIA subcommands SHALL be registered with the command framework including metadata (display name, description, category `"criteria"`), enabling discovery via command palette and shortcut binding. [WB]

---

### Requirement 7: Criteria Applied to Grid Display

**User Story:** As an editor user, I want the grid to show only records that satisfy my active criteria, so that I can focus on relevant data without noise while the underlying file remains unchanged.

**Source:** [FFE-CRITERIA Req 4, 12]

#### Acceptance Criteria

1. WHEN a Criteria_Set is applied, THE system SHALL evaluate each record against the Criteria_Set using the Criteria_Evaluator and display only records that satisfy the expression. [FFE-CRITERIA]
2. WHEN a Criteria_Set is applied, records that do not satisfy the criteria SHALL be excluded from the grid display entirely — they SHALL NOT be shown as greyed-out rows or placeholder rows. [FFE-CRITERIA]
3. WHEN a Criteria_Set is applied and the Record_Type_Scope is set to a specific Record_Structure name, THE Criteria_Evaluator SHALL apply criteria only to records of that type; records of other types SHALL be displayed normally without being subject to the criteria filter. [FFE-CRITERIA]
4. WHEN a Criteria_Set is applied and the Record_Type_Scope is `ALL TYPES`, THE Criteria_Evaluator SHALL apply criteria to all records regardless of their Record_Structure type. [FFE-CRITERIA]
5. WHEN criteria are active and the user issues a SAVE command, THE system SHALL save all records (including filtered-out records) to their original byte positions — the filter affects only display, not file content. [FFE-CRITERIA]
6. WHEN the user scrolls, navigates, or resizes the grid while criteria are active, THE system SHALL maintain the filter without requiring re-application. [FFE-CRITERIA]
7. WHEN the active Structure_Definition is changed while criteria are active, THE system SHALL clear the Active_Criteria_Set because the field names referenced by the criteria may no longer be valid. [FFE-CRITERIA]
8. WHEN a Criteria_Set references a field name that does not exist in the current Record_Structure, THE system SHALL display a warning identifying the unknown field name and treat criteria rows referencing that field as disabled. [FFE-CRITERIA]
9. WHEN a Criteria_Set and a Record_Filter are both active, THE system SHALL display only records that satisfy BOTH the Criteria_Set expression AND the Record_Filter condition. [FFE-CRITERIA]
10. WHEN a Criteria_Set and a Record_Type_Filter are both active, THE system SHALL display only records that satisfy BOTH the Criteria_Set expression AND the Record_Type_Filter condition. [FFE-CRITERIA]
11. WHEN all three filters are simultaneously active (Criteria_Set, Record_Filter, Record_Type_Filter), THE system SHALL apply all three conjunctively — only records satisfying all three SHALL be displayed. [FFE-CRITERIA]
12. WHEN criteria are active, THE system SHALL display the count of visible records and the total record count in the status bar (e.g., `Showing 142 of 10,000 records`). [FFE-CRITERIA]

---

### Requirement 8: Criteria Applied to FIND/CHANGE Scope

**User Story:** As an editor user, I want FIND and CHANGE operations to optionally restrict their scope to records matching my active criteria, so that I can search and replace only within the filtered record set.

**Source:** [WB]

#### Acceptance Criteria

1. WHEN `FIND 'text' CRITERIA` is issued and an Active_Criteria_Set is in effect, THE find engine SHALL restrict the search to lines belonging to records that satisfy the Active_Criteria_Set filter. [WB]
2. WHEN `CHANGE 'old' 'new' CRITERIA` is issued and an Active_Criteria_Set is in effect, THE find engine SHALL restrict replacements to lines belonging to records that satisfy the Active_Criteria_Set filter. [WB]
3. WHEN `FIND 'text' CRITERIA` is issued and no Active_Criteria_Set is in effect, THE find engine SHALL search all eligible lines (the CRITERIA modifier has no effect when no criteria are active). [WB]
4. THE `CRITERIA` scope modifier SHALL combine with other FIND/CHANGE modifiers (TAGGED, EXCLUDED, VISIBLE, column bounds) conjunctively — a line must satisfy all active scope constraints to be searched or changed. [WB]
5. WHEN the find panel's scope dropdown includes a "Criteria-matching records" option, selecting it SHALL apply the CRITERIA scope modifier to all searches and changes executed from the panel. [WB]
6. THE Criteria_Scope SHALL be evaluated at the record level: if any part of a record's display lines satisfies the criteria, all lines belonging to that record are eligible for FIND/CHANGE within the criteria scope. [WB]
7. WHEN criteria are active but the user does NOT specify the CRITERIA modifier, FIND/CHANGE SHALL operate on all visible lines regardless of criteria — the criteria filter does not implicitly restrict FIND/CHANGE unless explicitly requested. [WB]

---

### Requirement 9: Criteria Persistence (Named Criteria Sets)

**User Story:** As an editor user, I want to save, name, load, and manage criteria sets in a catalog, so that I can reuse filter expressions across sessions and share them with colleagues.

**Source:** [FFE-CRITERIA Req 6, 7, 8]

#### Acceptance Criteria

1. THE Criteria_Store SHALL be managed through the configuration system's user layer, stored at the user-level configuration directory (e.g., `~/.config/ffworkbench/criteria_store.toml`). [FFE-CRITERIA, WB]
2. THE Criteria_Store SHALL record a list of known Criteria_Locations and the name of the Active_Criteria_Location. [FFE-CRITERIA]
3. THE default Active_Criteria_Location SHALL be `~/.config/ffworkbench/criteria/` (or the platform equivalent), created automatically on first use if it does not exist. [FFE-CRITERIA]
4. EACH saved Criteria_Set SHALL be stored as a single `.criteria.json` file in the Active_Criteria_Location. [FFE-CRITERIA]
5. THE `.criteria.json` file SHALL be a JSON object containing: `name` (string), `structure_association` (string or null), `record_type_scope` (string or null), `case_sensitive` (boolean), `criteria` (array of criterion objects). [FFE-CRITERIA]
6. EACH criterion object SHALL contain: `enabled` (boolean), `field` (string), `operator` (string — one of the Criteria_Operator values), `value` (string), `value2` (string or null), `connector` (string `AND`/`OR` or null), `group_open` (boolean), `group_close` (boolean). [FFE-CRITERIA]
7. WHEN a `.criteria.json` file is missing required keys or contains an unrecognised operator string, THE system SHALL display an error describing the parse failure and not load the corrupted set. [FFE-CRITERIA]
8. WHEN the Criteria_Store file is absent at startup, THE system SHALL initialise with the default criteria location and an empty catalog, without error. [FFE-CRITERIA]
9. WHEN the Criteria_Store file is corrupt or unparseable, THE system SHALL initialise with defaults, emit a warning, and not overwrite the corrupt file until the operator makes a change. [FFE-CRITERIA]
10. THE system SHALL provide a Criteria Location Manager where the operator can add, remove, and rename Criteria_Locations and designate the Active_Criteria_Location. [FFE-CRITERIA]

---

### Requirement 10: Criteria UI Panel (Interactive Builder)

**User Story:** As an editor user in FileForge mode, I want an interactive visual panel for building, editing, and applying selection criteria, so that I can construct complex filter expressions without writing raw command syntax.

**Source:** [FFE-CRITERIA Req 1, 2, 7], [WB]

#### Acceptance Criteria

1. WHEN FileForge_Mode is active, THE system SHALL make the Criteria_Panel accessible from the editor menu, a toolbar button, the command palette, and the `CRITERIA` command. [FFE-CRITERIA]
2. THE Criteria_Panel SHALL display a grid with one row per Criterion, and each row SHALL contain: enabled checkbox, field-name dropdown, operator dropdown, value input, second-value input (visible only for range-type operations), connector dropdown (AND/OR, hidden on last row), group-open toggle, group-close toggle. [FFE-CRITERIA]
3. THE field-name dropdown SHALL be populated with field names from the active Record_Structure's field definitions, with field data type shown alongside each name. [FFE-CRITERIA, WB]
4. THE operator dropdown SHALL offer all Criteria_Operators: `EQ`, `NE`, `GT`, `GE`, `LT`, `LE`, `CONTAINS`, `STARTS_WITH`, `ENDS_WITH`, `MATCHES_REGEX`. [FFE-CRITERIA, WB]
5. WHEN a multi-type Structure_Definition is active, THE Criteria_Panel SHALL display a Record_Type_Scope selector allowing the user to choose a specific Record_Structure type or `ALL TYPES`. [FFE-CRITERIA]
6. THE Criteria_Panel SHALL provide buttons to add a new Criterion row, delete the selected row, move a row up, and move a row down. [FFE-CRITERIA]
7. THE Criteria_Panel SHALL provide `Apply`, `Clear`, `Save`, `Load`, and `Cancel` action buttons. [FFE-CRITERIA]
8. WHEN the user clicks `Apply`, THE system SHALL evaluate the Criteria_Set and apply the resulting filter to the grid display. The filter SHALL NOT be applied automatically when criteria rows are modified; the user MUST explicitly click `Apply`. [FFE-CRITERIA]
9. WHEN the user clicks `Clear`, THE system SHALL remove the Active_Criteria_Set, display all records, and remove the Criteria_Active_Indicator from the status bar. [FFE-CRITERIA]
10. WHEN the user clicks `Cancel`, THE system SHALL close the Criteria_Panel and leave the current filter state unchanged. [FFE-CRITERIA]
11. THE Criteria_Panel SHALL expose the Case_Sensitive_Flag as a checkbox labelled "Case sensitive" at the top of the panel. [FFE-CRITERIA]
12. THE Criteria_Panel SHALL be non-modal, allowing the user to interact with the editor grid while the panel is open. [FFE-CRITERIA]
13. THE Criteria_Panel SHALL be dockable within the workbench layout system — it can be docked to any panel zone or floated as a standalone window. [WB]
14. THE Criteria_Panel SHALL validate the criteria expression in real time, highlighting errors (unmatched groups, invalid regex patterns, type mismatches) with inline indicators before the user clicks Apply. [WB]

---

### Requirement 11: Criteria Catalog Dialog

**User Story:** As an editor user, I want a catalog dialog for naming, saving, loading, duplicating, and deleting criteria sets, so that I can reuse my filter expressions across sessions and files.

**Source:** [FFE-CRITERIA Req 7, 9]

#### Acceptance Criteria

1. THE Criteria_Catalog_Dialog SHALL be accessible from within the Criteria_Panel via a `Load` or `Catalog` button, and from the editor menu. [FFE-CRITERIA]
2. THE Criteria_Catalog_Dialog SHALL display all `.criteria.json` files found in the Active_Criteria_Location, showing at minimum the criteria set name, its associated Structure_Definition name (if any), and the number of Criterion rows. [FFE-CRITERIA]
3. THE Criteria_Catalog_Dialog SHALL allow the user to load a saved Criteria_Set into the Criteria_Panel for immediate use or editing. [FFE-CRITERIA]
4. THE Criteria_Catalog_Dialog SHALL allow the user to save the current Criteria_Panel contents as a new named Criteria_Set. [FFE-CRITERIA]
5. THE Criteria_Catalog_Dialog SHALL allow the user to overwrite an existing saved Criteria_Set with the current panel contents, with a confirmation prompt before overwriting. [FFE-CRITERIA]
6. THE Criteria_Catalog_Dialog SHALL allow the user to duplicate a saved Criteria_Set under a new name. [FFE-CRITERIA]
7. THE Criteria_Catalog_Dialog SHALL allow the user to delete a saved Criteria_Set, with a confirmation prompt before deletion. [FFE-CRITERIA]
8. THE Criteria_Catalog_Dialog SHALL allow switching the Active_Criteria_Location from within the dialog without closing it. [FFE-CRITERIA]
9. WHEN saving a Criteria_Set, THE system SHALL prompt for a name if one has not yet been assigned. The name SHALL be used as the file name (with non-alphanumeric characters replaced by underscores). [FFE-CRITERIA]
10. WHEN saving a Criteria_Set, THE system SHALL offer an optional `Associate with structure` field, prepopulated with the current Structure_Definition name. [FFE-CRITERIA]

---

### Requirement 12: Structure Association and Auto-Suggestion

**User Story:** As an editor user, I want the system to offer to apply previously saved criteria when I open a file with a matching structure, so that I don't have to manually reload the same filter every session.

**Source:** [FFE-CRITERIA Req 9, 10]

#### Acceptance Criteria

1. WHEN a Structure_Definition is applied to a file and the Active_Criteria_Location contains one or more `.criteria.json` files whose `structure_association` matches the Structure_Definition name (case-insensitive), THE system SHALL display a prompt offering to apply the most recently saved matching Criteria_Set. [FFE-CRITERIA]
2. WHEN multiple matching Criteria_Sets exist, THE system SHALL present a picker listing all matches instead of silently applying one. [FFE-CRITERIA]
3. WHEN the user declines the auto-suggestion, THE system SHALL proceed with no criteria applied and SHALL NOT prompt again during the same file session. [FFE-CRITERIA]
4. WHEN the user accepts the offer and the Criteria_Set loads successfully, THE system SHALL apply the criteria and display the Criteria_Active_Indicator in the status bar. [FFE-CRITERIA]
5. THE auto-suggestion SHALL check the Active_Criteria_Location only. [FFE-CRITERIA]
6. WHEN a file session ends with a Criteria_Set applied, THE system SHALL record the criteria set name (if saved) or the full criteria expression (if unsaved) in the session history entry for that file. [FFE-CRITERIA]
7. WHEN a file that has a criteria entry in session history is reopened, THE system SHALL display a prompt asking whether the user wants to restore the previous criteria. [FFE-CRITERIA]
8. WHEN the user accepts criteria restoration and the named set no longer exists in the catalog, THE system SHALL display a message and not apply any criteria. [FFE-CRITERIA]

---

### Requirement 13: Status Bar Integration

**User Story:** As an editor user, I want the status bar to always show whether criteria are active and which named criteria set is loaded, so that I always know what filter is in effect.

**Source:** [FFE-CRITERIA Req 5]

#### Acceptance Criteria

1. WHEN a Criteria_Set is applied, THE system SHALL display the Criteria_Active_Indicator in the status bar showing `Criteria: <name>` if the set has a saved name, or `Criteria: active` if the set is unsaved. [FFE-CRITERIA]
2. WHEN no Criteria_Set is applied, THE system SHALL NOT display the Criteria_Active_Indicator in the status bar. [FFE-CRITERIA]
3. WHEN a Criteria_Set is active and a Record_Type_Scope restricts evaluation to a specific type, THE status bar SHALL also show the scoped type name (e.g., `Criteria: active | Scope: Detail`). [FFE-CRITERIA]
4. THE Criteria_Active_Indicator SHALL be displayed alongside any active Record_Filter and Record_Type_Filter indicators without overwriting them. [FFE-CRITERIA]
5. WHEN criteria are active, THE system SHALL display the count of visible records and the total record count in the status bar (e.g., `Showing 142 of 10,000 records`). [FFE-CRITERIA]
6. WHEN `CRITERIA CLEAR` is issued, THE system SHALL clear only the Active_Criteria_Set and SHALL NOT affect the Record_Filter or the Record_Type_Filter. [FFE-CRITERIA]
7. WHEN `SHOW ALL` is issued (as defined in the `structure-catalog` spec), THE system SHALL clear the Record_Filter and the Record_Type_Filter but SHALL NOT clear the Active_Criteria_Set. [FFE-CRITERIA]

---

### Requirement 14: Configuration Integration

**User Story:** As an operator, I want criteria catalog paths and defaults configurable through the standard workbench configuration system, so that deployment-specific settings follow the same layered model as all other workbench configuration.

**Source:** [FFE-CRITERIA Req 6], [WB]

#### Acceptance Criteria

1. THE configuration system SHALL accept a `[criteria]` table in any configuration layer (system, user, profile, project, workspace) with the following keys: `store_path` (string — custom path for the Criteria_Store file), `default_location` (string — default Active_Criteria_Location path), `auto_suggest` (boolean — enable/disable structure-association auto-suggestion, default `true`), `max_criteria_rows` (integer — maximum rows per Criteria_Set, default `50`). [WB]
2. WHEN `criteria.store_path` is configured, THE system SHALL use that path for the Criteria_Store instead of the default user-level location. [WB]
3. WHEN `criteria.default_location` is configured, THE system SHALL use that path as the initial Active_Criteria_Location for new installations. [WB]
4. WHEN `criteria.auto_suggest` is `false`, THE system SHALL skip structure-association auto-suggestion prompts entirely. [WB]
5. WHEN configuration keys for this feature contain invalid values (e.g., a path that does not exist), THE system SHALL emit a configuration warning via the logging subsystem and apply the default. [FFE-CRITERIA, WB]
6. THE configuration system's hot-reload mechanism SHALL detect changes to `[criteria]` settings and apply them without requiring application restart. [WB]
