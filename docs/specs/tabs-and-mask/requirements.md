# Requirements Document

## Introduction

This feature specifies the **TABS and MASK** display-helper commands for FileForgeWorkbench (`ff-tabs-and-mask` crate). These two closely related features provide visual aids and session-level state for tab stop management and insert-mask templates, following the same Display_Artifact_Line pattern established by `COLS` and `BNDS` in the `navigation-commands` spec.

Both commands insert a synthetic display-only line into the viewport as a visual aid and carry session-level state that is non-undoable and is cleared (display-wise) by `RESET`. Neither the TABS line nor the MASK line is real document content — they are never saved to disk.

**TABS** manages tab stop positions. The Tab key uses tab stops when inserting text; the TABS line makes the configured positions visible in the viewport. Tab stops can be set globally via the configuration system (`editor.default_tab_stops`), and language definitions (TOML files managed by `language-service`) can provide per-language defaults via a `default_tab_stops` key.

**MASK** manages an insert mask — a template string that pre-fills newly inserted blank lines (created by the `I`/`In` line commands) with boilerplate content. This is especially useful for fixed-format languages such as COBOL, where new lines may need sequence-number placeholders at specific column positions. The mask is edited by typing directly into the MASK line in the viewport. Language definitions can provide a per-language default mask via a `default_mask` key. The mask is cleared with `MASK OFF`.

Both TABS and MASK are available as:
- **Primary commands** — entered on the command line to display, configure, or clear the respective state.
- **Line commands** — entered in the prefix area to insert the display line at a specific document position.

The `ff-tabs-and-mask` crate is a Wave 11 (Display Mode) component. It depends on `ff-command` (command-framework) for command registration and dispatch, `ff-config` (configuration-system) for default tab stop and mask settings, `ff-language-service` (language-service) for per-language defaults, and integrates with `ff-edit-operations` (edit-operations) for Tab key behaviour and mask-filled line insertion.

**Source references:**
- **[FFE-TABSMASK]** = FileForgeEditor `tabs-and-mask` spec (Requirements 1–14)
- **[WB]** = Workbench Platform Architecture Brief (GUI-independent core, command-driven architecture, configuration as data, per-language TOML definitions)

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `edit-operations` | **Integration** | Tab key insertion uses the active Tab_Stop list. Mask-filled line insertion occurs through the I/In line command execution path. Tab behaviour defined in Requirement 5 integrates with insert-mode semantics in `edit-operations` Requirement 1.8. |
| `auto-indentation` | **Coordination** | Indent/Unindent commands (Tab/Shift+Tab on selected lines) are owned by `auto-indentation`; single-cursor Tab key behaviour for advancing to tab stops is owned here. The two subsystems coordinate via the active Tab_Stop list in Session_State. |
| `navigation-commands` | **Pattern** | TABS and MASK follow the same Display_Artifact_Line pattern as COLS_Line and BNDS_Line defined in `navigation-commands`. COLS/BNDS established the synthetic-line model; TABS/MASK extend it. |
| `command-framework` | **Dependency** | All TABS and MASK commands are registered with the command registry, dispatched through the command execution pipeline, and carry metadata for discoverability. |
| `configuration-system` | **Dependency** | Provides the `editor.default_tab_stops` configuration key and the language-profile directory structure for per-language TOML files. Hot-reload propagates tab stop and mask changes without restart. |
| `language-service` | **Dependency** | Provides the active language definition for the current document, from which `default_tab_stops` and `default_mask` values are read at session start. |
| `command-semantics` | **Integration** | TABS/MASK commands pass through the ISPF command engine for parsing and dispatch. The RESET command (owned by `command-semantics`) clears TABS/MASK display artifacts. |
| `line-commands` | **Integration** | TABS and MASK line commands are entered in the prefix area and processed through the line-command execution pipeline. The I/In line commands in `line-commands` trigger mask application. |

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **TABS_Line** | A synthetic display-only line inserted into the viewport to show the active tab stop column positions. Not a real document line; never saved to disk. | [FFE-TABSMASK] |
| **Tab_Stop** | A column position (1-based) at which the Tab key advances the cursor when inserting text. Tab stops are stored in Session_State as an ordered list of distinct positive integers. | [FFE-TABSMASK] |
| **Insert_Mask** | A template string stored in Session_State that is applied to every blank line inserted by the `I` or `In` line commands. Non-space characters indicate pre-filled content at those column positions. | [FFE-TABSMASK] |
| **MASK_Line** | A synthetic display-only line inserted into the viewport to show and allow in-place editing of the active Insert_Mask. Not a real document line; never saved to disk. | [FFE-TABSMASK] |
| **Display_Artifact_Line** | Any synthetic viewport line that is not real document content — including COLS_Line, BNDS_Line, TABS_Line, and MASK_Line. Display artifact lines are never saved to disk, are not part of the document model, and are not included in any command scope. | [FFE-TABSMASK, WB] |
| **Session_State** | Transient in-memory editor state (excluded lines, tags, bounds, COLS markers, BNDS markers, tab stops, TABS line, insert mask, MASK line) that is not persisted to disk. Non-undoable. | [FFE-TABSMASK] |
| **Language_Definition** | A TOML configuration file in the `languages/` directory that defines language-specific behaviour including, optionally, `default_tab_stops` and `default_mask`. Managed by `language-service`. | [FFE-TABSMASK, WB] |
| **Primary_Command** | A command entered on the command line (Command ===> area) and dispatched through the command framework. | [FFE-TABSMASK] |
| **Line_Command** | A command entered in the prefix area next to a document line, processed through the line-command execution pipeline. | [FFE-TABSMASK] |

---

## Requirements

---

### Requirement 1: TABS Primary Command — Display and Toggle [FFE-TABSMASK]

**User Story:** As an editor user, I want to type `TABS` on the command line to display the current tab stop positions in the viewport as a non-editable ruler line, so that I can confirm where the Tab key will advance before I start editing.

**Source:** [FFE-TABSMASK] Requirement 1. Cross-references: `navigation-commands` (COLS_Line pattern), `command-framework` (dispatch), `command-semantics` (RESET).

#### Acceptance Criteria

1.1 WHEN `TABS` is issued as a Primary_Command with no arguments, THE command framework SHALL insert a TABS_Line into the viewport at the current cursor position (or at the top of the visible area if no cursor line is defined). [FFE-TABSMASK]

1.2 THE TABS_Line SHALL show a tab stop indicator character (e.g., `T`) at each column position that is a configured Tab_Stop, and a filler character (e.g., `-` or space) at all other column positions, up to the maximum configured line width. [FFE-TABSMASK]

1.3 THE TABS_Line SHALL be formatted so that column positions visually align with the corresponding characters in the document lines above and below it. [FFE-TABSMASK]

1.4 WHEN `TABS` is issued a second time while a TABS_Line is already displayed, THE command framework SHALL remove all TABS_Lines from the viewport (toggle behaviour). [FFE-TABSMASK]

1.5 WHEN `RESET` or `RESET ALL` is issued, THE command framework SHALL remove all TABS_Lines from the viewport. [FFE-TABSMASK]

1.6 THE TABS_Line SHALL scroll with the document such that it remains visually anchored to the document lines it was inserted between. [FFE-TABSMASK]

1.7 WHEN multiple `TABS` Primary_Commands are issued at different cursor positions, THE command framework SHALL display a separate TABS_Line at each requested position. [FFE-TABSMASK]

1.8 THE prefix area cell adjacent to a TABS_Line SHALL be non-editable and SHALL display a fixed indicator (e.g., `TABS`). [FFE-TABSMASK]

1.9 THE command framework SHALL NOT record TABS display changes as undoable transactions; the TABS_Line is a Display_Artifact_Line and its visibility is Session_State only. [FFE-TABSMASK]

1.10 THE `TABS` command SHALL be valid in Browse mode, Edit mode, and View mode. [FFE-TABSMASK]

---

### Requirement 2: TABS Primary Command — Configure Tab Stops [FFE-TABSMASK]

**User Story:** As an editor user, I want to specify explicit tab stop column positions on the TABS command line so that I can customise where the Tab key advances without editing a configuration file.

**Source:** [FFE-TABSMASK] Requirement 2. Cross-references: `configuration-system` (default_tab_stops key), `command-framework` (dispatch).

#### Acceptance Criteria

2.1 WHEN `TABS col1 col2 ...` is issued with one or more positive integer column arguments, THE command framework SHALL replace the active Tab_Stop list in Session_State with the provided column positions. [FFE-TABSMASK]

2.2 WHEN tab stops are set via `TABS col1 col2 ...`, THE command framework SHALL update any existing TABS_Line(s) in the viewport to reflect the new tab stop positions immediately. [FFE-TABSMASK]

2.3 WHEN `TABS col1 col2 ...` is issued, THE command framework SHALL insert a TABS_Line into the viewport at the current cursor position if no TABS_Line is currently displayed. [FFE-TABSMASK]

2.4 THE command framework SHALL store the new Tab_Stop list in Session_State, replacing any previously active tab stops. [FFE-TABSMASK]

2.5 THE command framework SHALL NOT persist session tab stop changes to configuration files or any Language_Definition automatically; the change is session-only. [FFE-TABSMASK, WB]

2.6 THE command framework SHALL NOT record tab stop configuration changes as undoable transactions; tab stop state is Session_State only. [FFE-TABSMASK]

2.7 IF any column argument is not a positive integer, or if a column argument is zero, THEN THE command framework SHALL display an error message "Invalid tab stop: column positions must be positive integers" and SHALL NOT update the Tab_Stop list. [FFE-TABSMASK]

2.8 IF duplicate column values are provided, THE command framework SHALL deduplicate them and store only distinct column positions, sorted in ascending order. [FFE-TABSMASK]

---

### Requirement 3: TABS Line Command — Insert TABS_Line at Position [FFE-TABSMASK]

**User Story:** As an editor user, I want to enter `TABS` in the prefix area next to a specific line to insert a tab ruler at that exact position, so that I can visualise tab stops within a particular block of code without moving the cursor.

**Source:** [FFE-TABSMASK] Requirement 3. Cross-references: `line-commands` (prefix area processing), `navigation-commands` (COLS line command pattern).

#### Acceptance Criteria

3.1 WHEN the `TABS` Line_Command is entered in the prefix area next to a document line, THE command framework SHALL insert a TABS_Line immediately above that document line. [FFE-TABSMASK]

3.2 THE TABS_Line inserted by the line command SHALL reflect the currently active Tab_Stop list in Session_State. [FFE-TABSMASK]

3.3 WHEN `RESET` or `RESET ALL` is issued, THE command framework SHALL remove TABS_Lines inserted by line commands, consistent with removal of all Display_Artifact_Lines. [FFE-TABSMASK]

3.4 THE prefix area cell adjacent to the inserted TABS_Line SHALL be non-editable and SHALL display a fixed indicator (e.g., `TABS`). [FFE-TABSMASK]

3.5 THE command framework SHALL NOT record insertion of a TABS_Line as an undoable transaction. [FFE-TABSMASK]

---

### Requirement 4: Default Tab Stops and Language Profile Integration [FFE-TABSMASK, WB]

**User Story:** As an editor user, I want the editor to load sensible tab stop defaults from the global configuration and from the active language definition, so that I can start editing a COBOL or JCL file and get correct tab behaviour without manually configuring stops every session.

**Source:** [FFE-TABSMASK] Requirement 4. Cross-references: `configuration-system` (global config keys, hot-reload), `language-service` (language definitions, per-language TOML files).

#### Acceptance Criteria

4.1 THE configuration system SHALL support an `editor.default_tab_stops` key whose value is an array of positive integers representing default column positions. [FFE-TABSMASK, WB]

4.2 WHEN `editor.default_tab_stops` is absent or empty in the effective configuration, THE editor SHALL initialise tab stops to a built-in default of every 8 columns (columns 9, 17, 25, 33, ...). [FFE-TABSMASK]

4.3 WHEN a file is opened and the active Language_Definition contains a `default_tab_stops` key, THE editor SHALL use the Language_Definition's tab stops as the initial Session_State Tab_Stop list for that session, overriding the global default. [FFE-TABSMASK]

4.4 WHEN a file is opened and the active Language_Definition does not contain a `default_tab_stops` key, THE editor SHALL use the global `editor.default_tab_stops` from the configuration system. [FFE-TABSMASK]

4.5 THE Language_Definition `default_tab_stops` key SHALL be an array of positive integers in the language TOML file (e.g., `default_tab_stops = [7, 12, 72]`). [FFE-TABSMASK, WB]

4.6 IF the `editor.default_tab_stops` value in the configuration or a Language_Definition contains any value that is not a positive integer, THE editor SHALL ignore that invalid value, log a warning via the logging subsystem, and continue with the remaining valid values. [FFE-TABSMASK, WB]

4.7 THE active Tab_Stop list in Session_State SHALL be ordered in ascending order of column position after loading from any source. [FFE-TABSMASK]

---

### Requirement 5: Tab Key Behaviour with Tab Stops [FFE-TABSMASK]

**User Story:** As an editor user, I want the Tab key to advance the cursor to the next configured tab stop column when I am inserting text, so that I can align content to language-specific column positions by pressing a single key.

**Source:** [FFE-TABSMASK] Requirement 5. Cross-references: `edit-operations` (insert mode Tab handling, Requirement 1.8), `auto-indentation` (Indent/Unindent commands for selected lines).

#### Acceptance Criteria

5.1 WHEN the Tab key is pressed while the cursor is in an editable document line in Edit mode with no selection active, THE editor SHALL advance the cursor to the next column position in the active Tab_Stop list that is greater than the current cursor column. [FFE-TABSMASK]

5.2 IF the cursor is already at or past the last Tab_Stop in the active list, THE editor SHALL advance the cursor to the next tab stop computed by repeating the last interval (or to the end of the line, whichever comes first). [FFE-TABSMASK]

5.3 WHEN no Tab_Stop list is configured (the list is empty), THE editor SHALL fall back to advancing the cursor by the configured `editor.tab_size` value from the configuration system. [FFE-TABSMASK, WB]

5.4 THE Tab key behaviour described in this requirement applies only in Edit mode with no selection; in Browse mode and View mode, the Tab key SHALL use its standard navigation behaviour. WHEN a selection is active, Tab SHALL delegate to the Indent command in `auto-indentation`. [FFE-TABSMASK]

5.5 WHEN the Tab key advances the cursor in Insert mode, THE editor SHALL insert space characters (not a literal tab character) to fill from the current column to the target tab stop column, consistent with the ISPF fixed-column editing model. [FFE-TABSMASK, WB]

5.6 WHEN the Tab key advances the cursor in Overstrike mode, THE editor SHALL move the cursor to the target tab stop column without inserting or modifying any characters. [FFE-TABSMASK]

---

### Requirement 6: MASK Primary Command — Display and Toggle [FFE-TABSMASK]

**User Story:** As an editor user, I want to type `MASK` on the command line to display the current insert mask in the viewport as an editable template line, so that I can see and modify the boilerplate content that will be applied to newly inserted blank lines.

**Source:** [FFE-TABSMASK] Requirement 6. Cross-references: `navigation-commands` (Display_Artifact_Line pattern), `command-framework` (dispatch), `command-semantics` (RESET).

#### Acceptance Criteria

6.1 WHEN `MASK` is issued as a Primary_Command with no arguments and an Insert_Mask is currently active, THE command framework SHALL insert a MASK_Line into the viewport at the current cursor position (or at the top of the visible area if no cursor line is defined). [FFE-TABSMASK]

6.2 WHEN `MASK` is issued as a Primary_Command with no arguments and no Insert_Mask is currently active, THE command framework SHALL display "No active mask — use MASK to set one or check the language profile" in the status area. [FFE-TABSMASK]

6.3 THE MASK_Line SHALL display the full content of the active Insert_Mask, character by character, aligned with document line columns. [FFE-TABSMASK]

6.4 THE MASK_Line SHALL be directly editable in place: the user can type any characters into the MASK_Line to modify the Insert_Mask content. Changes made to the MASK_Line SHALL update the Insert_Mask in Session_State immediately. [FFE-TABSMASK]

6.5 WHEN `MASK` is issued a second time while a MASK_Line is already displayed, THE command framework SHALL remove all MASK_Lines from the viewport (toggle behaviour). [FFE-TABSMASK]

6.6 WHEN `RESET` or `RESET ALL` is issued, THE command framework SHALL remove all MASK_Lines from the viewport but SHALL NOT clear the active Insert_Mask; `MASK OFF` is required to clear the mask content. [FFE-TABSMASK]

6.7 THE MASK_Line SHALL scroll with the document such that it remains visually anchored to the document lines it was inserted between. [FFE-TABSMASK]

6.8 WHEN multiple `MASK` Primary_Commands are issued at different cursor positions, THE command framework SHALL display a separate MASK_Line at each requested position. [FFE-TABSMASK]

6.9 THE prefix area cell adjacent to a MASK_Line SHALL be non-editable and SHALL display a fixed indicator (e.g., `MASK`). [FFE-TABSMASK]

6.10 THE command framework SHALL NOT record MASK display state changes as undoable transactions; MASK_Line visibility and Insert_Mask content are Session_State only. [FFE-TABSMASK]

6.11 THE `MASK` command SHALL be valid in Browse mode (display-only, not editable) and in Edit mode (display and editable). [FFE-TABSMASK]

---

### Requirement 7: MASK OFF — Clear the Active Insert Mask [FFE-TABSMASK]

**User Story:** As an editor user, I want to issue `MASK OFF` to clear the current insert mask so that newly inserted lines are blank rather than pre-filled with boilerplate content.

**Source:** [FFE-TABSMASK] Requirement 7. Cross-references: `command-framework` (dispatch).

#### Acceptance Criteria

7.1 WHEN `MASK OFF` is issued, THE command framework SHALL clear the active Insert_Mask from Session_State, making it empty. [FFE-TABSMASK]

7.2 WHEN `MASK OFF` is issued, THE command framework SHALL remove all MASK_Lines from the viewport. [FFE-TABSMASK]

7.3 WHEN `MASK OFF` is issued and no Insert_Mask is currently active, THE command framework SHALL display "No active mask to clear" in the status area and SHALL NOT modify Session_State. [FFE-TABSMASK]

7.4 THE command framework SHALL NOT record `MASK OFF` as an undoable transaction; Insert_Mask state is Session_State only. [FFE-TABSMASK]

---

### Requirement 8: MASK Line Command — Insert MASK_Line at Position [FFE-TABSMASK]

**User Story:** As an editor user, I want to enter `MASK` in the prefix area next to a specific line to insert the mask template display at that exact position, so that I can inspect or edit the mask template inline without issuing a primary command.

**Source:** [FFE-TABSMASK] Requirement 8. Cross-references: `line-commands` (prefix area processing), `navigation-commands` (COLS line command pattern).

#### Acceptance Criteria

8.1 WHEN the `MASK` Line_Command is entered in the prefix area next to a document line, THE command framework SHALL insert a MASK_Line immediately above that document line. [FFE-TABSMASK]

8.2 THE MASK_Line inserted by the line command SHALL display the currently active Insert_Mask in Session_State. [FFE-TABSMASK]

8.3 THE MASK_Line inserted by the line command SHALL be directly editable in place, consistent with Requirement 6.4. [FFE-TABSMASK]

8.4 WHEN `RESET` or `RESET ALL` is issued, THE command framework SHALL remove MASK_Lines inserted by line commands from the viewport, but SHALL NOT clear the active Insert_Mask. [FFE-TABSMASK]

8.5 THE prefix area cell adjacent to the inserted MASK_Line SHALL be non-editable and SHALL display a fixed indicator (e.g., `MASK`). [FFE-TABSMASK]

8.6 THE command framework SHALL NOT record insertion of a MASK_Line as an undoable transaction. [FFE-TABSMASK]

---

### Requirement 9: Insert Mask Applied to Newly Inserted Lines [FFE-TABSMASK]

**User Story:** As an editor user, I want blank lines inserted by the `I` or `In` line commands to be pre-filled with the active insert mask so that COBOL or other fixed-format lines start with the correct structural boilerplate.

**Source:** [FFE-TABSMASK] Requirement 9. Cross-references: `line-commands` (I/In line command execution), `edit-operations` (line insertion), `undo-redo-transactions` (transaction grouping).

#### Acceptance Criteria

9.1 WHEN the `I` line command is executed and an Insert_Mask is active in Session_State, THE command framework SHALL fill the newly inserted blank line with the Insert_Mask content rather than leaving it empty. [FFE-TABSMASK]

9.2 WHEN the `In` line command is executed (inserting n lines) and an Insert_Mask is active, THE command framework SHALL apply the Insert_Mask to every one of the n inserted blank lines. [FFE-TABSMASK]

9.3 WHEN no Insert_Mask is active, THE command framework SHALL insert blank lines as usual, without any pre-filling. [FFE-TABSMASK]

9.4 THE mask application SHALL be part of the `I`/`In` insert transaction so that undoing the insert also removes the mask-filled content; the mask application itself is not a separate transaction and does not add to the undo stack independently. [FFE-TABSMASK]

9.5 WHEN the Insert_Mask is shorter than the document line width, THE command framework SHALL pad the inserted line with spaces to the right of the mask content. [FFE-TABSMASK]

9.6 WHEN the Insert_Mask is longer than the document line width, THE command framework SHALL truncate the mask at the document line width. [FFE-TABSMASK]

---

### Requirement 10: Default Mask and Language Definition Integration [FFE-TABSMASK, WB]

**User Story:** As an editor user, I want the editor to load a default insert mask from the active language definition when I open a COBOL or other fixed-format file, so that new lines are automatically pre-filled with the correct structural content for that language.

**Source:** [FFE-TABSMASK] Requirement 10. Cross-references: `language-service` (language definitions, TOML files), `configuration-system` (hot-reload).

#### Acceptance Criteria

10.1 WHEN a file is opened and the active Language_Definition contains a `default_mask` key, THE editor SHALL use its value as the initial Insert_Mask in Session_State for that session. [FFE-TABSMASK]

10.2 WHEN a file is opened and the active Language_Definition does not contain a `default_mask` key, THE editor SHALL start the session with no active Insert_Mask (empty mask). [FFE-TABSMASK]

10.3 THE Language_Definition `default_mask` key SHALL be a plain string value in the language TOML file (e.g., `default_mask = "      *"`). [FFE-TABSMASK, WB]

10.4 THE `default_mask` string value SHALL be used verbatim as the Insert_Mask template without any transformation; tab characters and special characters SHALL be preserved as-is. [FFE-TABSMASK]

10.5 WHEN `MASK OFF` is issued during a session, THE editor SHALL clear the Insert_Mask for the remainder of the session, even if the Language_Definition defines a `default_mask`; the language default is applied only on file open, not re-applied automatically. [FFE-TABSMASK]

10.6 IF the `default_mask` value in a Language_Definition is not a string, THE editor SHALL log a warning via the logging subsystem, treat the definition's `default_mask` as absent, and start the session with no active Insert_Mask. [FFE-TABSMASK, WB]

---

### Requirement 11: RESET Interaction for TABS and MASK [FFE-TABSMASK]

**User Story:** As an editor user, I want the `RESET` command to clear TABS and MASK display lines from the viewport along with other display artifacts, so that a single command restores a clean editing view.

**Source:** [FFE-TABSMASK] Requirement 11. Cross-references: `command-semantics` (RESET command ownership), `navigation-commands` (COLS/BNDS RESET behaviour).

#### Acceptance Criteria

11.1 WHEN `RESET` is issued with no arguments, THE command framework SHALL remove all TABS_Lines and all MASK_Lines from the viewport. [FFE-TABSMASK]

11.2 WHEN `RESET ALL` is issued, THE command framework SHALL remove all TABS_Lines and all MASK_Lines from the viewport. [FFE-TABSMASK]

11.3 WHEN `RESET` or `RESET ALL` is issued, THE command framework SHALL NOT clear the active Tab_Stop list from Session_State; tab stops persist across RESET. [FFE-TABSMASK]

11.4 WHEN `RESET` or `RESET ALL` is issued, THE command framework SHALL NOT clear the active Insert_Mask from Session_State; the mask persists across RESET. Only `MASK OFF` clears the Insert_Mask. [FFE-TABSMASK]

11.5 WHEN `RESET COMMANDS` is issued, THE command framework SHALL clear any pending TABS or MASK line commands from the prefix area but SHALL NOT remove already-inserted TABS_Lines or MASK_Lines and SHALL NOT clear the Tab_Stop list or Insert_Mask. [FFE-TABSMASK]

---

### Requirement 12: RESET TABS — Clear Custom Tab Stops [FFE-TABSMASK, WB]

**User Story:** As an editor user, I want a `RESET TABS` command to restore the default tab stops (from configuration or language definition), so that I can undo custom session tab stop changes without restarting the session.

**Source:** New workbench requirement extending [FFE-TABSMASK]. Cross-references: `configuration-system` (default_tab_stops), `language-service` (language definition defaults).

#### Acceptance Criteria

12.1 WHEN `RESET TABS` is issued, THE command framework SHALL replace the active Tab_Stop list in Session_State with the default tab stops determined by the precedence rules in Requirement 4 (Language_Definition > global config > built-in every-8-columns). [WB]

12.2 WHEN `RESET TABS` is issued, THE command framework SHALL update any displayed TABS_Lines to reflect the restored default tab stop positions. [WB]

12.3 WHEN `RESET TABS` is issued, THE command framework SHALL NOT remove TABS_Lines from the viewport; only the tab stop values change. [WB]

12.4 THE command framework SHALL NOT record `RESET TABS` as an undoable transaction; tab stop state is Session_State only. [WB]

---

### Requirement 13: Configurable Default Tab Stops per Language [FFE-TABSMASK, WB]

**User Story:** As a system administrator or advanced user, I want to configure global default tab stops and per-language tab stops in TOML configuration files, so that all sessions start with sensible defaults without requiring per-session configuration.

**Source:** [FFE-TABSMASK] Requirement 12. Cross-references: `configuration-system` (TOML format, hot-reload, layer precedence), `language-service` (language TOML files).

#### Acceptance Criteria

13.1 THE configuration system SHALL support an `editor.default_tab_stops` key whose value is a TOML array of positive integers (e.g., `default_tab_stops = [9, 17, 25]`). [FFE-TABSMASK, WB]

13.2 WHEN `editor.default_tab_stops` is missing from the effective configuration, THE editor SHALL behave as if `default_tab_stops = []` and SHALL fall back to every-8-columns tab stop behaviour as defined in Requirement 4.2. [FFE-TABSMASK]

13.3 WHEN `editor.default_tab_stops` contains invalid entries (non-integer or non-positive values), THE editor SHALL log a warning per invalid entry via the logging subsystem and continue with the remaining valid values, consistent with fault-tolerant config loading defined in `configuration-system`. [FFE-TABSMASK, WB]

13.4 THE Language_Definition TOML files SHALL support an optional `default_tab_stops` key whose value is an array of positive integers. [FFE-TABSMASK, WB]

13.5 THE Language_Definition TOML files SHALL support an optional `default_mask` key whose value is a plain string. [FFE-TABSMASK, WB]

13.6 WHEN both a global `editor.default_tab_stops` and a Language_Definition `default_tab_stops` are defined, THE Language_Definition value SHALL take precedence for sessions opened on files of that language type. [FFE-TABSMASK, WB]

13.7 WHEN the configuration system hot-reloads and the `editor.default_tab_stops` value changes, THE editor SHALL NOT retroactively change tab stops for existing sessions; the new value SHALL apply only to newly opened sessions. [WB]

---

### Requirement 14: TABS Interaction with Indent/Shift Commands [FFE-TABSMASK, WB]

**User Story:** As an editor user, I want the TABS settings to interact correctly with indent and shift line commands, so that column-based operations respect my configured tab stops and produce predictable alignment.

**Source:** New workbench requirement. Cross-references: `auto-indentation` (Indent/Unindent), `line-commands` (shift left/right `<`/`>`), `edit-operations` (Tab key in selection).

#### Acceptance Criteria

14.1 WHEN the `>` (shift right) line command is executed on a line, THE command framework SHALL shift the line content rightward to the next Tab_Stop position relative to the current first-non-space column of that line. [WB]

14.2 WHEN the `<` (shift left) line command is executed on a line, THE command framework SHALL shift the line content leftward to the previous Tab_Stop position relative to the current first-non-space column of that line. [WB]

14.3 IF no Tab_Stop exists to the left when `<` is executed, THE command framework SHALL shift the content to column 1 (leftmost position). [WB]

14.4 WHEN `>n` or `<n` line commands are executed (shift by n positions), THE command framework SHALL use the Tab_Stop list to determine the target column by advancing n stops from the current first-non-space column. [WB]

14.5 WHEN the Indent command (Tab with selection active) is invoked on selected lines, THE command framework SHALL delegate to the `auto-indentation` subsystem which uses `editor.indent_size` — NOT the TABS tab stop list. The TABS tab stop list is for single-cursor Tab navigation only. [WB]

---

### Requirement 15: Per-Session TABS State (Non-Undoable) [FFE-TABSMASK]

**User Story:** As an editor user, I want tab stop changes and TABS display state to be session-scoped and non-undoable, so that undo/redo operations do not unexpectedly alter my tab configuration.

**Source:** [FFE-TABSMASK] Requirements 1.9, 2.6, 3.5. Cross-references: `undo-redo-transactions` (transaction scope), `command-framework` (undo classification).

#### Acceptance Criteria

15.1 THE Tab_Stop list SHALL be stored per-session in Session_State and SHALL NOT be part of the document model or the undo/redo transaction history. [FFE-TABSMASK]

15.2 THE Insert_Mask SHALL be stored per-session in Session_State and SHALL NOT be part of the document model or the undo/redo transaction history. [FFE-TABSMASK]

15.3 WHEN undo or redo operations are performed, THE editor SHALL NOT modify the active Tab_Stop list or Insert_Mask. [FFE-TABSMASK]

15.4 WHEN a document is saved, THE editor SHALL NOT persist the active Tab_Stop list or Insert_Mask to the file; these are session-only state. [FFE-TABSMASK]

15.5 WHEN a document is closed and reopened, THE editor SHALL reinitialise Tab_Stop list and Insert_Mask from their default sources (configuration system and Language_Definition), not from any previously active session state. [FFE-TABSMASK, WB]

---

### Requirement 16: MASK as Visual Aid for Fixed-Format Editing [FFE-TABSMASK]

**User Story:** As a COBOL or JCL developer, I want the MASK line to serve as a visual guide showing the fixed-format column layout, so that I can see field boundaries while editing without memorising column positions.

**Source:** [FFE-TABSMASK] Requirement 6. Cross-references: `navigation-commands` (COLS display for column numbers).

#### Acceptance Criteria

16.1 THE MASK_Line SHALL visually indicate fixed-format field boundaries by displaying the mask template characters at their respective column positions, making column alignment immediately visible. [FFE-TABSMASK]

16.2 THE MASK_Line SHALL use a distinct visual style (colour and/or font weight) that differentiates it from document content lines, COLS_Lines, BNDS_Lines, and TABS_Lines. [FFE-TABSMASK]

16.3 WHEN the user edits the MASK_Line content, THE changes SHALL be reflected immediately in all subsequently inserted lines (via I/In), without requiring a separate apply command. [FFE-TABSMASK]

16.4 THE MASK_Line SHALL support the full document line width, allowing fixed-format templates up to the maximum configured line width to be defined. [FFE-TABSMASK]

---

### Requirement 17: TABS Display — Ruler Showing Active Tab Stops [FFE-TABSMASK]

**User Story:** As an editor user, I want the TABS ruler line to clearly show which columns are tab stops, so that I can visually confirm alignment positions before and during editing.

**Source:** [FFE-TABSMASK] Requirements 1.2, 1.3. Cross-references: `navigation-commands` (COLS ruler format).

#### Acceptance Criteria

17.1 THE TABS_Line SHALL render a distinct indicator character at each configured tab stop column, providing a visual ruler of tab positions. [FFE-TABSMASK]

17.2 THE TABS_Line indicator characters SHALL be clearly distinguishable from filler characters and from document content. [FFE-TABSMASK]

17.3 THE TABS_Line SHALL use a distinct visual style (colour and/or font weight) that differentiates it from document content lines, MASK_Lines, COLS_Lines, and BNDS_Lines. [FFE-TABSMASK]

17.4 WHEN tab stops are modified via `TABS col1 col2 ...`, ALL displayed TABS_Lines SHALL update their indicator positions immediately to reflect the new configuration. [FFE-TABSMASK]

17.5 THE TABS_Line SHALL extend to the full visible width of the viewport, providing ruler coverage across the entire visible editing area. [FFE-TABSMASK]

---

### Requirement 18: Display_Artifact_Line Compatibility [FFE-TABSMASK]

**User Story:** As a developer implementing TABS and MASK, I want both commands to conform to the same Display_Artifact_Line conventions used by COLS and BNDS so that the command engine and viewport treat all display artifacts consistently.

**Source:** [FFE-TABSMASK] Requirements 13–14. Cross-references: `navigation-commands` (COLS/BNDS Display_Artifact_Line rules), `find-and-replace` (scope exclusion), `command-semantics` (scope resolution), `line-commands` (compatibility matrix).

#### Acceptance Criteria

18.1 THE TABS_Line SHALL NOT be a real document line: it SHALL NOT be included in any command scope, SHALL NOT be counted in line number calculations, and SHALL NOT be saved to disk. [FFE-TABSMASK]

18.2 THE MASK_Line SHALL NOT be a real document line: it SHALL NOT be included in any command scope, SHALL NOT be counted in line number calculations, and SHALL NOT be saved to disk. [FFE-TABSMASK]

18.3 WHEN a primary command that operates on scope (FIND, CHANGE, SORT, DELETE, EXCLUDE, etc.) is executed while TABS_Lines or MASK_Lines are displayed, THE command framework SHALL skip those Display_Artifact_Lines and SHALL NOT include them in the resolved scope. [FFE-TABSMASK]

18.4 WHEN `FIND` is executed with a search term that matches the visual content of a TABS_Line or MASK_Line, THE command framework SHALL NOT report a match on those lines. [FFE-TABSMASK]

18.5 THE command framework SHALL include TABS and MASK (line command forms) in the list of Display Helper Line Commands in the HELP LINECOMMANDS output. [FFE-TABSMASK]

18.6 WHEN `TABS` or `MASK` are entered as line commands alongside any primary command other than blank, THE command framework SHALL execute the primary command first and then insert the Display_Artifact_Line, unless the primary command itself modifies Display_Artifact_Lines (e.g., RESET). [FFE-TABSMASK]

18.7 THE `TABS` and `MASK` commands SHALL be registered in the command framework with metadata including: command ID, display name, description, category ("display"), undo classification (non-undoable), and applicable modes. [WB]
