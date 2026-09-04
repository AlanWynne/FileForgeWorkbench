# Requirements Document

## Introduction

This feature specifies the **Sequence Numbers** subsystem for FileForgeWorkbench (`ff-sequence-numbers` crate). It handles the detection, stripping, re-insertion, and display of legacy sequence numbers found in mainframe source files — particularly COBOL, JCL, FORTRAN, and PL/I — where fixed column ranges carry punched-card-era sequence data that is not part of the source logic.

The sequence-numbers subsystem provides:

1. **Automatic detection** of sequence number columns based on language profile configuration and heuristic content sampling.
2. **Automatic stripping** (Auto-Unnum) on file open — transparent removal of detected sequence numbers from the edit buffer before first display.
3. **`UNNUM` primary command** — explicit removal of sequence numbers from any file or scoped range.
4. **`NUMBER` primary command** — explicit insertion of sequential numbers into defined column positions.
5. **`NUMBER SHOW` display mode** — overlay rendering of original sequence numbers without modifying the edit buffer.
6. **Preserve/strip on save options** — configurable behaviour controlling whether sequence numbers are restored on save.

### Design Principles


1. **Stripping is the default** — when a language profile defines sequence columns and `auto_unnum` is true, detection and removal happen automatically on file open without operator intervention.
2. **Re-insertion is explicit** — writing sequence numbers back requires the deliberate `NUMBER` command with confirmation.
3. **GUI-independent** — all detection, stripping, and numbering logic operates on the document model without GUI framework dependency. [WB]
4. **Command-framework integrated** — UNNUM, NUMBER, and NUMBER SHOW are registered with the command framework, discoverable, and scriptable. [WB]
5. **BOUNDS-aware** — sequence columns are excluded from BOUNDS-constrained edit operations; stripping does not alter active BOUNDS settings.
6. **Language-service driven** — sequence column definitions come from language profile TOML files managed by the language service.

This crate is a Wave 11 (Display Modes) component in the workbench architecture. It depends on:
- `ff-language-service` — for language profile sequence column definitions
- `ff-document-model` — for edit buffer access and line content manipulation
- `ff-command` — for command registration and dispatch
- `ff-undo` (undo-redo-transactions) — for recording undoable sequence transactions
- `ff-config` (configuration-system) — for detection rules, save behaviour, and per-language overrides

It is consumed by:
- `ff-edit-operations` — BOUNDS interaction when sequence columns are present
- `ff-viewport` (viewport-and-scrolling) — for NUMBER SHOW overlay rendering
- `ff-file-ops` (file-operations) — for preserve/strip on save behaviour

### Source References

- **[FFE-SEQNUM]** = FileForgeEditor `sequence-numbers` specification (all 10 requirements)
- **[WB]** = Workbench Platform Architecture Brief (GUI independence, command-driven, multi-crate)

### Cross-References

- **`document-model`** — Provides the TextBuffer/Document that this subsystem reads and modifies for stripping/numbering operations.
- **`edit-operations`** — Defines BOUNDS constraint semantics; sequence columns interact with the BOUNDS editable area.
- **`navigation-commands`** — Defines the BOUNDS/BNDS command that sets active column boundaries; stripping does not alter BOUNDS state.
- **`configuration-system`** — Provides per-language and global settings for detection thresholds, auto-unnum, and save behaviour.
- **`command-framework`** — All commands (UNNUM, NUMBER, NUMBER SHOW) are registered, dispatched, and discoverable through this framework.
- **`language-service`** — Manages language profile TOML definitions including `sequence_cols_front`, `sequence_cols_back`, and `auto_unnum` keys.
- **`undo-redo-transactions`** — UNNUM and NUMBER operations are recorded as single undoable Sequence_Transactions.
- **`file-operations`** — Save operations interact with sequence number state (preserve/strip on save).

---

## Glossary


| Term | Definition | Source |
|------|-----------|--------|
| **Sequence_Number** | Numeric data occupying fixed column positions in a source line, inherited from the punched-card era. Not part of the source code logic. Examples: COBOL columns 1–6, FORTRAN columns 1–5, identification area columns 73–80. | [FFE-SEQNUM] |
| **Sequence_Cols_Front** | The column range at the start of a line used for sequence numbers, as defined in the active language profile. For COBOL fixed format: columns 1–6. For FORTRAN fixed format: columns 1–5. | [FFE-SEQNUM] |
| **Sequence_Cols_Back** | The column range at the end of a line used for sequence numbers or identification data, as defined in the active language profile. For COBOL, FORTRAN, JCL, PL/I: columns 73–80. | [FFE-SEQNUM] |
| **Language_Profile** | A language definition TOML file managed by the `ff-language-service` crate. Provides language-specific configuration including sequence number column definitions. | [FFE-SEQNUM], [WB] |
| **Auto_Unnum** | A boolean flag in the Language_Profile (default: `true`) controlling whether sequence numbers are automatically stripped when a file is opened. | [FFE-SEQNUM] |
| **Sequence_Detector** | The component that samples file content and determines whether defined sequence columns contain numeric sequence data using configurable heuristics. | [FFE-SEQNUM] |
| **Detection_Threshold** | The minimum percentage of sampled non-blank lines that must match the numeric pattern to confirm sequence number presence. Default: 80%. Configurable via configuration-system. | [FFE-SEQNUM], [WB] |
| **UNNUM** | Primary command that removes sequence numbers from the current document's edit buffer using language profile column definitions or an explicit column range. | [FFE-SEQNUM] |
| **NUMBER** | Primary command that writes sequential numbers into defined column positions. Requires confirmation before modifying content. | [FFE-SEQNUM] |
| **NUMBER_SHOW** | A display-only mode that renders sequence numbers in the viewport without modifying the edit buffer. | [FFE-SEQNUM] |
| **Sequence_Transaction** | The single undoable Transaction created when UNNUM or NUMBER modifies the edit buffer. Recorded via `ff-undo` API. | [FFE-SEQNUM] |
| **Sequence_Format** | The format used for generated sequence numbers: pure numeric (zero-padded) or alphanumeric prefix (e.g., `ABC00100`). | [WB] |
| **Standard_Text_Mode** | The editor's normal text editing mode, as opposed to FileForge Grid_Edit_Mode. Sequence number processing applies only in this mode. | [FFE-SEQNUM] |
| **Visual_Indicator** | A rendered annotation (gutter marker, column shading, or status bar indicator) that communicates detected sequence column state to the operator. | [WB] |

---

## Requirements

### Requirement 1: Language Profile — Sequence Number Column Configuration

**User Story:** As a language configuration author, I want to define sequence number column ranges in a language TOML profile, so that the editor knows where to look for and remove sequence numbers without requiring operator input per file.

**Source:** [FFE-SEQNUM] Requirement 1. Cross-references: `language-service` (TOML schema), `configuration-system` (language profile directories).

#### Acceptance Criteria


1. THE Language_Profile TOML schema SHALL support an optional `sequence_cols_front` string key specifying the front sequence column range in `"start-end"` format (e.g., `"1-6"` for COBOL, `"1-5"` for FORTRAN). Absence of the key means no front sequence columns are defined for that language. [FFE-SEQNUM]

2. THE Language_Profile TOML schema SHALL support an optional `sequence_cols_back` string key specifying the back sequence column range in `"start-end"` format (e.g., `"73-80"` for COBOL, JCL, FORTRAN, and PL/I). Absence of the key means no back sequence columns are defined for that language. [FFE-SEQNUM]

3. THE Language_Profile TOML schema SHALL support an optional `auto_unnum` boolean key. WHEN `auto_unnum` is absent, THE system SHALL treat it as `true` (auto-strip enabled by default). [FFE-SEQNUM]

4. THE system SHALL validate that `sequence_cols_front` and `sequence_cols_back` values are well-formed column range strings with a start column less than or equal to the end column and both values greater than zero. IF a value is malformed, THE system SHALL emit a WARN-level log record and ignore that key. [FFE-SEQNUM]

5. THE COBOL language profile SHALL define `sequence_cols_front = "1-6"`, `sequence_cols_back = "73-80"`, and `auto_unnum = true`. [FFE-SEQNUM]

6. THE FORTRAN language profile SHALL define `sequence_cols_front = "1-5"`, `sequence_cols_back = "73-80"`, and `auto_unnum = true`. [FFE-SEQNUM]

7. THE JCL language profile SHALL define `sequence_cols_back = "73-80"` and `auto_unnum = true`. No front sequence columns SHALL be defined for JCL. [FFE-SEQNUM]

8. THE PL/I language profile SHALL define `sequence_cols_back = "73-80"` and `auto_unnum = true`. No front sequence columns SHALL be defined for PL/I. [FFE-SEQNUM]

9. WHEN a language profile defines no sequence columns (neither `sequence_cols_front` nor `sequence_cols_back`), THE Sequence_Detector SHALL NOT run for files of that language, and no stripping SHALL occur. [FFE-SEQNUM]

10. THE configuration-system SHALL support per-language override of `auto_unnum` in user or project configuration layers, allowing operators to disable auto-stripping for specific languages without modifying the language profile TOML. [WB]

---

### Requirement 2: Sequence Number Detection

**User Story:** As an editor user opening a legacy mainframe source file, I want the editor to automatically detect whether sequence numbers are present in the defined column ranges, so that stripping only occurs when actual sequence data is found.

**Source:** [FFE-SEQNUM] Requirement 2. Cross-references: `document-model` (line content access), `language-service` (active language profile).

#### Acceptance Criteria


1. WHEN a file is opened in Standard_Text_Mode and the active Language_Profile defines at least one sequence column range, THE Sequence_Detector SHALL sample up to the first 20 non-blank lines of the file. [FFE-SEQNUM]

2. THE Sequence_Detector SHALL determine that sequence numbers are present in a column range IF at least the Detection_Threshold percentage (default 80%) of sampled non-blank lines have that column range fully populated with digit characters (0–9) or space characters, with at least one line in the sample containing all digit characters in that range. [FFE-SEQNUM]

3. WHEN the file contains fewer than 5 non-blank lines, THE Sequence_Detector SHALL require 100% of sampled lines to match the numeric criterion before reporting sequence numbers as present. [FFE-SEQNUM]

4. THE Sequence_Detector SHALL evaluate `sequence_cols_front` and `sequence_cols_back` independently. A file may have front sequence numbers, back sequence numbers, both, or neither. [FFE-SEQNUM]

5. WHEN a line in the sample is shorter than the end column of the range being checked, THE Sequence_Detector SHALL treat that line as not matching the numeric criterion for that range. [FFE-SEQNUM]

6. THE Sequence_Detector SHALL complete its sampling and detection without blocking the UI thread. IF file access is slow, THE detector SHALL defer stripping to a background step with a progress indicator. [FFE-SEQNUM], [WB]

7. THE detection algorithm SHALL be purely read-only — it SHALL NOT modify the edit buffer or the source file. [FFE-SEQNUM]

8. THE Detection_Threshold SHALL be configurable via the configuration-system (key: `editor.sequence_numbers.detection_threshold`), accepting values from 50 to 100 inclusive. IF a value outside this range is configured, THE system SHALL clamp to the nearest valid value and emit a WARN-level log record. [WB]

9. THE Sequence_Detector SHALL support an optional alphanumeric prefix pattern in addition to pure numeric detection. WHEN a column range contains a consistent alphabetic prefix followed by digits across the Detection_Threshold of sampled lines, THE detector SHALL classify it as alphanumeric sequence numbers. [WB]

---

### Requirement 3: Automatic Strip on File Open (Auto-Unnum)

**User Story:** As an editor user opening a legacy source file, I want sequence numbers to be automatically removed from the edit buffer before the file is displayed, so that I see and edit clean source text without ever having to request removal.

**Source:** [FFE-SEQNUM] Requirement 3. Cross-references: `document-model` (edit buffer mutation), `undo-redo-transactions` (non-undoable operation classification).

#### Acceptance Criteria


1. WHEN a file is opened in Standard_Text_Mode, `auto_unnum` is `true` in the active Language_Profile, AND the Sequence_Detector reports sequence numbers as present, THE system SHALL strip the detected sequence columns from every line in the edit buffer before the first viewport render. [FFE-SEQNUM]

2. THE stripping SHALL replace the sequence column bytes on each line with space characters, consistent with ISPF conventions for column-range clearing. Lines shorter than the start of the column range SHALL be left unchanged. [FFE-SEQNUM]

3. THE strip operation SHALL be applied to the edit buffer only. The source file on disk SHALL remain unchanged until the user explicitly issues SAVE. [FFE-SEQNUM]

4. WHEN auto-strip completes, THE system SHALL display a status message informing the operator that sequence numbers were detected and removed, identifying which column ranges were stripped (e.g., `SEQUENCE NUMBERS REMOVED: COLS 1-6, 73-80`). [FFE-SEQNUM]

5. THE auto-strip on open SHALL NOT be added to the Undo_Stack and SHALL NOT be reversible via UNDO. It is classified as a session initialisation operation per the undo-redo-transactions spec. [FFE-SEQNUM]

6. WHEN `auto_unnum` is `false` in the active Language_Profile and sequence numbers are detected, THE system SHALL display a status message informing the operator that sequence numbers were detected but not removed, and SHALL NOT modify the edit buffer. [FFE-SEQNUM]

7. WHEN a file is opened in Browse mode, THE Sequence_Detector SHALL still run and sequence numbers SHALL still be stripped from the display buffer for Browse mode rendering. The on-disk file is not modified regardless of mode. [FFE-SEQNUM]

8. THE auto-strip operation SHALL NOT modify any active BOUNDS settings. BOUNDS are session state and are not adjusted automatically by sequence number processing. [FFE-SEQNUM]

9. THE system SHALL store the original stripped sequence number values in an internal side-table (keyed by line number) to enable NUMBER SHOW overlay rendering and potential later restoration. [WB]

---

### Requirement 4: Visual Indication of Detected Sequence Columns

**User Story:** As an editor user, I want a clear visual indication when the editor has detected and stripped sequence numbers, so that I understand the column context of the file I am editing.

**Source:** [WB] Visual feedback principle. Cross-references: `menu-and-statusbar` (status indicators), `theme-and-appearance` (colour tokens).

#### Acceptance Criteria


1. WHEN sequence numbers have been detected and stripped from the current document, THE system SHALL display a `SEQNUM` indicator in the status bar showing the stripped column ranges (e.g., `SEQNUM 1-6,73-80`). [WB]

2. WHEN sequence numbers are detected but NOT stripped (because `auto_unnum` is `false`), THE system SHALL display a `SEQNUM?` indicator in the status bar to alert the operator that sequence data may be present. [WB]

3. THE COLS column ruler (as defined in the navigation-commands spec) SHALL always display physical column positions starting from column 1, regardless of whether sequence number stripping has occurred. The operator is responsible for interpreting which columns now contain source code. [FFE-SEQNUM]

4. WHEN NUMBER SHOW mode is active, THE system SHALL display a `SEQSHOW` indicator in the status bar. [FFE-SEQNUM]

5. THE system SHALL support optional column-range shading in the viewport for sequence number columns when configured via `editor.sequence_numbers.highlight_columns = true`. The shading SHALL use a theme-defined colour token (`sequence-column-background`). [WB]

---

### Requirement 5: UNNUM Primary Command

**User Story:** As an editor user, I want an explicit UNNUM command to remove sequence numbers from the current file or a scoped range, so that I can strip sequence numbers from files whose language profile does not define sequence columns, or re-strip after manually adding data.

**Source:** [FFE-SEQNUM] Requirement 4. Cross-references: `command-framework` (command registration), `undo-redo-transactions` (Sequence_Transaction), `line-commands` (CC block pairing).

#### Acceptance Criteria


1. THE command framework SHALL register `UNNUM` (Command_ID: `sequence.unnum`) as a primary command valid in Edit mode and Browse mode (display-only effect in Browse). [FFE-SEQNUM], [WB]

2. WHEN `UNNUM` is issued with no arguments, THE system SHALL strip sequence numbers from all lines using the `sequence_cols_front` and `sequence_cols_back` ranges defined in the active Language_Profile. IF neither range is defined, THE system SHALL display an error: `UNNUM: no sequence columns defined for this language — use UNNUM COLS to specify a range`. [FFE-SEQNUM]

3. THE system SHALL support `UNNUM COLS start end` (e.g., `UNNUM COLS 1 6`) to strip an explicit column range from all lines, regardless of Language_Profile definitions. [FFE-SEQNUM]

4. THE system SHALL support `UNNUM FRONT` to strip only the `sequence_cols_front` range defined in the active Language_Profile. IF `sequence_cols_front` is not defined, THE system SHALL display an error. [FFE-SEQNUM]

5. THE system SHALL support `UNNUM BACK` to strip only the `sequence_cols_back` range defined in the active Language_Profile. IF `sequence_cols_back` is not defined, THE system SHALL display an error. [FFE-SEQNUM]

6. THE system SHALL support `UNNUM ALL` to strip both the `sequence_cols_front` and `sequence_cols_back` ranges defined in the active Language_Profile. IF neither range is defined, THE system SHALL display an error as in criterion 5.2. [FFE-SEQNUM]

7. WHEN `UNNUM` (any form) is combined with a `CC...CC` block line command, THE system SHALL restrict the strip operation to the lines within the block range. [FFE-SEQNUM]

8. WHEN `UNNUM` operates on lines where the defined column range is already filled entirely with space characters, THE system SHALL leave those lines unchanged and SHALL NOT count them as modified. [FFE-SEQNUM]

9. THE UNNUM operation SHALL be recorded as a single Sequence_Transaction in the Undo_Stack and SHALL be fully reversible via UNDO, restoring all stripped columns to their original content. [FFE-SEQNUM]

10. WHEN `UNNUM` completes successfully, THE system SHALL display a status message reporting the number of lines modified (e.g., `UNNUM: 350 lines modified`). [FFE-SEQNUM]

11. WHEN `UNNUM` is issued in Browse mode, THE strip SHALL be applied to the display buffer only and SHALL NOT modify any persisted state. [FFE-SEQNUM]

---

### Requirement 6: NUMBER Primary Command — Explicit Sequencing

**User Story:** As an editor user, I want a NUMBER command to write sequence numbers back into defined column positions, so that I can produce numbered output for legacy tools or systems that require sequence numbers.

**Source:** [FFE-SEQNUM] Requirement 5. Cross-references: `command-framework` (command registration), `undo-redo-transactions` (Sequence_Transaction), `line-commands` (CC block pairing).

#### Acceptance Criteria


1. THE command framework SHALL register `NUMBER` (Command_ID: `sequence.number`) as a primary command valid in Edit mode only. [FFE-SEQNUM], [WB]

2. WHEN `NUMBER` is issued with no arguments, THE system SHALL display a usage summary showing all supported NUMBER sub-commands and SHALL NOT modify the edit buffer. [FFE-SEQNUM]

3. THE system SHALL support `NUMBER COLS start end` (e.g., `NUMBER COLS 73 80`) to write sequential numbers into an explicit column range on all lines. [FFE-SEQNUM]

4. THE system SHALL support `NUMBER STD` to write sequential numbers using the `sequence_cols_back` range defined in the active Language_Profile. IF `sequence_cols_back` is not defined, THE system SHALL fall back to `sequence_cols_front`. IF neither is defined, THE system SHALL display an error. [FFE-SEQNUM]

5. THE system SHALL support `NUMBER STD start_val increment` (e.g., `NUMBER STD 10 10`) to control the starting value and increment of the sequence. Both SHALL be positive integers. IF either is zero or negative, THE system SHALL display an error. [FFE-SEQNUM]

6. WHEN `NUMBER COLS` or `NUMBER STD` is issued without explicit `start_val` and `increment`, THE system SHALL use a default starting value of 1 and an increment of 1, padding numbers with leading zeros to fill the column width. [FFE-SEQNUM]

7. THE system SHALL support `NUMBER ON` to enable auto-numbering mode. WHILE auto-numbering mode is active, WHEN the operator inserts a new line via any insert operation (line command `I`/`In`, clipboard paste, or file insert), THE system SHALL automatically assign the next sequence number in the active column range. [FFE-SEQNUM]

7a. THE system SHALL treat `AUTONUM ON` as an alias for `NUMBER ON` and `AUTONUM OFF` as an alias for `NUMBER OFF`. Both forms SHALL produce identical behaviour and be interchangeable. [EARS SN-AUTONUM]

8. THE system SHALL support `NUMBER OFF` to disable auto-numbering mode. Auto-numbering SHALL be off by default. [FFE-SEQNUM]

9. WHEN `NUMBER COLS`, `NUMBER STD`, or `NUMBER ON` is issued, THE system SHALL display a confirmation prompt before modifying the edit buffer: `NUMBER will overwrite column range nn-mm on all lines. Confirm? (YES/NO)`. The operation SHALL proceed only if the operator responds YES. [FFE-SEQNUM]

10. THE NUMBER sequencing operation SHALL be recorded as a single Sequence_Transaction in the Undo_Stack and SHALL be fully reversible via UNDO. [FFE-SEQNUM]

11. WHEN NUMBER generates a sequence value wider than the target column range, THE system SHALL truncate the number to fit the column width and display a warning: `NUMBER: sequence overflow — numbers truncated to fit COLS nn-mm`. [FFE-SEQNUM]

12. WHEN `NUMBER` is combined with a `CC...CC` block line command, THE system SHALL restrict the numbering operation to the lines within the block range. The sequence counter SHALL restart from the specified starting value for the block. [FFE-SEQNUM]

---

### Requirement 7: Sequence Number Format Options

**User Story:** As an editor user working with various mainframe formats, I want to control the format of generated sequence numbers (pure numeric or alphanumeric prefix), so that output matches the conventions expected by different legacy systems.

**Source:** [WB] Format flexibility. Cross-references: `configuration-system` (format settings).

#### Acceptance Criteria


1. THE system SHALL support a `NUMERIC` sequence format (default) that generates zero-padded decimal numbers filling the entire column width (e.g., `000100` for a 6-column range with start=100). [WB]

2. THE system SHALL support an `ALPHA_PREFIX` sequence format that generates an alphabetic prefix followed by zero-padded digits (e.g., `ABC001` for prefix `ABC` in a 6-column range). The prefix and digit portions SHALL together equal the column width. [WB]

3. THE system SHALL support `NUMBER COLS start end FORMAT format_name` syntax to specify the sequence format. Valid format names SHALL be `NUMERIC` (default) and `ALPHA prefix` (e.g., `NUMBER COLS 73 80 ALPHA ABC`). [WB]

4. WHEN `ALPHA prefix` format is specified and the prefix length plus the minimum digit width (1) exceeds the column width, THE system SHALL display an error: `NUMBER: prefix too long for column range`. [WB]

5. THE configuration-system SHALL support a default format setting (`editor.sequence_numbers.default_format`) with value `"numeric"` or `"alpha:PREFIX"`. The default SHALL be `"numeric"`. [WB]

---

### Requirement 8: NUMBER SHOW Display Mode -- NUM Alias

**Alias criterion (EARS SN-NUM-alias):** THE system SHALL treat `NUM` as an alias for the `NUMBER` command. WHEN `NUM` is entered with any sub-command or argument that `NUMBER` accepts (e.g., `NUM ON`, `NUM OFF`, `NUM SHOW`, `NUM COLS start end`, `NUM STD`), THE system SHALL execute the equivalent `NUMBER` form. [EARS SN-NUM-alias]

**User Story:** As an operator who wants to inspect the original sequence numbers of a legacy file without them being stored in the edit buffer, I want a NUMBER SHOW mode that overlays sequence numbers in the viewport display.

**Source:** [FFE-SEQNUM] Requirement 6. Cross-references: `viewport-and-scrolling` (overlay rendering), `theme-and-appearance` (muted style).

#### Acceptance Criteria


1. THE command framework SHALL register `NUMBER SHOW` (Command_ID: `sequence.number_show`) as a primary command that activates or deactivates the Sequence_Number display overlay for the current session. [FFE-SEQNUM], [WB]

2. WHEN NUMBER SHOW is active and the edit buffer has sequence numbers stripped, THE system SHALL render the original sequence number values from the stored side-table in the sequence column positions within the viewport display. These displayed values SHALL be visually distinct from the source text (using a theme-defined `sequence-number-overlay` style — typically a muted colour or reduced opacity). [FFE-SEQNUM]

3. WHEN NUMBER SHOW is active, THE system SHALL NOT modify the edit buffer — the display is cosmetic only. IF the operator saves with NUMBER SHOW active, the saved file SHALL NOT contain sequence numbers unless they are in the edit buffer. [FFE-SEQNUM]

4. WHEN NUMBER SHOW is toggled off, THE edit buffer content SHALL remain unchanged — the column positions show whatever is stored in the edit buffer. [FFE-SEQNUM]

5. THE NUMBER SHOW state SHALL be displayed in the status bar as a `SEQSHOW` indicator when active. [FFE-SEQNUM]

6. THE NUMBER SHOW state SHALL NOT be added to the Undo_Stack. It is a non-undoable display state change, consistent with other display mode commands (HEX ON/OFF). [FFE-SEQNUM]

7. WHEN NUMBER SHOW is active and no sequence numbers were stripped (the edit buffer retains the original column content), THE system SHALL display the column content as-is without any visual distinction — the mode has no visual effect if stripping did not occur. [FFE-SEQNUM]

---

### Requirement 9: Interaction with Undo/Redo System

**User Story:** As an editor user, I want UNNUM and NUMBER to be undoable single-step operations, so that I can recover from accidental strip or re-sequence without manually reconstructing original content.

**Source:** [FFE-SEQNUM] Requirement 7. Cross-references: `undo-redo-transactions` (Transaction API, non-undoable classification).

#### Acceptance Criteria


1. WHEN UNNUM modifies the edit buffer, THE system SHALL wrap all line modifications in a single Sequence_Transaction and push it to the Undo_Stack. A single UNDO SHALL reverse the entire UNNUM operation regardless of how many lines were modified. [FFE-SEQNUM]

2. WHEN NUMBER (sequencing form — COLS, STD) modifies the edit buffer, THE system SHALL wrap all line modifications in a single Sequence_Transaction and push it to the Undo_Stack. A single UNDO SHALL reverse the entire NUMBER operation. [FFE-SEQNUM]

3. THE auto-strip performed at file open (Requirement 3) SHALL NOT be pushed to the Undo_Stack and SHALL NOT be reversible via UNDO. It is classified as a session initialisation operation. [FFE-SEQNUM]

4. WHEN NUMBER ON auto-numbering inserts sequence numbers into newly inserted lines, EACH such insertion SHALL be part of the same Transaction as the line insertion operation that triggered it. The line insertion and its auto-sequence number are undone together as a single step. [FFE-SEQNUM]

5. WHEN UNDO reverses a Sequence_Transaction created by UNNUM, THE system SHALL restore the exact original byte content of each stripped column — not just re-insert blank spaces. [FFE-SEQNUM]

6. WHEN UNDO reverses a Sequence_Transaction created by NUMBER, THE system SHALL restore the column content that existed in the edit buffer before the NUMBER command ran. [FFE-SEQNUM]

---

### Requirement 10: BOUNDS Interaction

**User Story:** As an editor user with active BOUNDS set, I want sequence number columns to be excluded from BOUNDS-constrained edit operations, so that column-aware editing respects both the BOUNDS and the sequence number column layout.

**Source:** [FFE-SEQNUM] Requirement 3 AC 8, [WB]. Cross-references: `navigation-commands` (BOUNDS/BNDS command), `edit-operations` (BOUNDS constraint semantics).

#### Acceptance Criteria


1. THE auto-strip operation SHALL NOT modify any active BOUNDS settings. IF the operator had set BOUNDS before opening the file, those bounds remain exactly as configured after stripping. [FFE-SEQNUM]

2. WHEN sequence columns are stripped, THE system SHALL NOT automatically adjust BOUNDS to account for the removed column content. The operator is responsible for setting BOUNDS appropriate to the post-strip column layout. [FFE-SEQNUM]

3. WHEN UNNUM or NUMBER modifies column content, THE system SHALL NOT alter the active BOUNDS. BOUNDS are session state owned by the navigation-commands subsystem. [WB]

4. WHEN the operator explicitly sets BOUNDS that overlap with defined sequence column ranges and auto-numbering (NUMBER ON) is active, THE system SHALL assign sequence numbers only to columns outside the active BOUNDS. IF the sequence column range is entirely within BOUNDS, THE system SHALL display a warning: `NUMBER ON: sequence columns overlap with active BOUNDS — auto-numbering disabled for overlapping range`. [WB]

---

### Requirement 11: Interaction with SAVE

**User Story:** As an editor user, I want confidence that saving a file after sequence number stripping does NOT write sequence numbers back to disk unless I explicitly request it, while also having the option to preserve them.

**Source:** [FFE-SEQNUM] Requirement 8, [WB]. Cross-references: `file-operations` (save pipeline).

#### Acceptance Criteria


1. WHEN the operator issues SAVE after sequence numbers have been auto-stripped or stripped via UNNUM, THE system SHALL write the edit buffer content to disk. Because the sequence column positions contain spaces, the saved file SHALL NOT contain the original sequence numbers. [FFE-SEQNUM]

2. THE system SHALL NOT re-inject sequence numbers into the saved output unless the operator has explicitly used NUMBER to write them back into the edit buffer before saving. [FFE-SEQNUM]

3. WHEN the operator issues SAVE with NUMBER ON auto-numbering active, THE system SHALL save the edit buffer including the auto-generated sequence numbers that have been written into it. [FFE-SEQNUM]

4. WHEN the operator opens a previously stripped file (saved after stripping), THE Sequence_Detector SHALL sample the file as usual. Because the sequence columns now contain spaces, the detector SHALL determine that sequence numbers are not present and SHALL NOT perform any stripping. [FFE-SEQNUM]

5. THE configuration-system SHALL support a `editor.sequence_numbers.restore_on_save` boolean setting (default: `false`). WHEN set to `true` and sequence numbers were stripped on open, THE system SHALL restore the original sequence numbers from the stored side-table into the save output (the edit buffer remains stripped). [WB]

6. WHEN `restore_on_save` is `true` and the operator has modified lines since open (new lines inserted, lines deleted), THE system SHALL generate new sequence numbers for modified/inserted lines using the format and increment detected at open time, while preserving original numbers for unmodified lines. [WB]

---

### Requirement 12: Configuration for Detection Rules Per Language

**User Story:** As a workbench administrator, I want to configure detection rules per language (threshold, sample size, column ranges) through the configuration system, so that sequence number behaviour can be tuned for specific environments without modifying language profile TOML files.

**Source:** [WB] Configuration-driven behaviour. Cross-references: `configuration-system` (layered overrides), `language-service` (language profile registry).

#### Acceptance Criteria


1. THE configuration-system SHALL support a `[editor.sequence_numbers]` table with the following global settings: `detection_threshold` (integer 50–100, default 80), `sample_size` (integer 5–100, default 20), `highlight_columns` (boolean, default false), `default_format` (string, default "numeric"), `restore_on_save` (boolean, default false). [WB]

2. THE configuration-system SHALL support per-language overrides using `[editor.sequence_numbers.languages.<language_id>]` tables, allowing any of the global settings plus `auto_unnum`, `sequence_cols_front`, and `sequence_cols_back` to be overridden for a specific language. [WB]

3. WHEN a per-language configuration override defines `sequence_cols_front` or `sequence_cols_back`, THE system SHALL use the configuration value in preference to the language profile TOML value. Configuration-system layer precedence applies (project > user > defaults). [WB]

4. WHEN a per-language configuration override sets `auto_unnum = false`, THE system SHALL suppress automatic stripping for that language regardless of the language profile TOML setting. [WB]

5. THE configuration-system SHALL support hot-reload of sequence number settings. WHEN detection or display settings change while a file is open, THE system SHALL apply the new display settings (highlight_columns, NUMBER SHOW style) immediately. Detection settings apply only to files opened after the change. [WB]

---

### Requirement 13: Interaction with Standard Text Mode Only

**User Story:** As a data engineer working with FileForge structured files, I want sequence number processing to be completely absent in Grid_Edit_Mode, so that the field-offset-based structured record model is not disrupted.

**Source:** [FFE-SEQNUM] Requirement 9. Cross-references: `fileforge-integration` (Grid_Edit_Mode).

#### Acceptance Criteria


1. WHEN a file is opened in FileForge Grid_Edit_Mode (as defined in the fileforge-integration spec), THE Sequence_Detector SHALL NOT run and no sequence number stripping SHALL occur, regardless of Language_Profile settings. [FFE-SEQNUM]

2. THE UNNUM and NUMBER primary commands SHALL display an error when issued in Grid_Edit_Mode: `UNNUM/NUMBER: not applicable in Grid Edit Mode`. [FFE-SEQNUM]

3. WHEN a file is opened in Standard_Text_Mode for a language with `auto_unnum = true` and then the user switches to a different display mode that does not activate Grid_Edit_Mode (e.g., hex display), THE sequence number stripping state SHALL be preserved — the edit buffer retains its stripped state. [FFE-SEQNUM]

---

### Requirement 14: Command Compatibility Matrix

**User Story:** As a command engine developer, I want UNNUM and NUMBER entries in the command compatibility matrix so that interactions with line commands are precisely defined.

**Source:** [FFE-SEQNUM] Requirement 10. Cross-references: `command-semantics` (compatibility matrix), `line-commands` (CC block pairing).

#### Acceptance Criteria

1. THE command-semantics compatibility matrix SHALL include an entry for `UNNUM` with no line commands that specifies: strip sequence columns from all lines using Language_Profile definitions. [FFE-SEQNUM]

2. THE command-semantics compatibility matrix SHALL include an entry for `UNNUM COLS start end` with no line commands that specifies: strip the explicit column range from all lines. [FFE-SEQNUM]

3. THE command-semantics compatibility matrix SHALL include an entry for `UNNUM` (any form) combined with `CC...CC` block line commands that specifies: restrict the strip to the marked block. [FFE-SEQNUM]

4. THE command-semantics compatibility matrix SHALL include an entry for `NUMBER COLS start end` with no line commands that specifies: write sequential numbers to the column range on all lines (with confirmation). [FFE-SEQNUM]

5. THE command-semantics compatibility matrix SHALL include an entry for `NUMBER STD [start increment]` with no line commands that specifies: write sequential numbers to the Language_Profile default columns (with confirmation). [FFE-SEQNUM]

6. THE command-semantics compatibility matrix SHALL include an entry for `NUMBER` combined with `CC...CC` block line commands that specifies: restrict the numbering operation to the marked block. [FFE-SEQNUM]

7. THE command-semantics compatibility matrix SHALL include an entry for `NUMBER ON` and `NUMBER OFF` specifying: toggle auto-numbering mode (state change, no immediate line modifications, no confirmation required). [FFE-SEQNUM]

8. THE command-semantics compatibility matrix SHALL include an entry for `NUMBER SHOW` specifying: toggle sequence number display overlay (display-only, non-undoable). [FFE-SEQNUM]
