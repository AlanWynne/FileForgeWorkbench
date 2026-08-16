# Requirements Document

## Introduction

This spec defines the **Function Keys and Command History** subsystem for FileForgeWorkbench (`ff-function-keys` crate). It covers three closely related capabilities inspired by IBM ISPF/PDF workflows:

1. **Configurable Function Keys** — Function keys F1–F24 can be assigned to any registered command or macro invocation. A global default key map provides consistent bindings regardless of file type. Individual language profiles may define their own key maps that fully replace the global map when that profile is active.

2. **Key Label Bar** — A visual display region showing the current function key assignments, rendered in the workbench footer area. This mirrors ISPF's bottom-of-screen key label display and serves as an always-visible command reference.

3. **RETRIEVE Command and Command History** — A persistent, bounded, deduplicated history of previously entered primary commands. The RETRIEVE command cycles backward through this history one entry at a time. A dropdown UI provides random-access selection from the history list. Certain commands (UNDO, REDO, RETRIEVE) are excluded from history to prevent recall pollution.

All function key assignments route through the command framework — pressing a function key dispatches its assigned command through `command-framework` exactly as if typed on the command line. History persistence leverages the startup-and-session infrastructure for graceful degradation and cross-session continuity.

### Design Principles

1. **Command-framework integration** — Function key presses and RETRIEVE are dispatched through the command framework like all other command invocations. [WB]
2. **GUI-independent logic** — Key map resolution, history management, and RETRIEVE pointer logic reside in the platform-core layer with no GUI framework dependency. [WB]
3. **Configuration-system integration** — All key map and history settings live in the TOML configuration hierarchy and obey the layered override model. [FFE-FKEYS, WB]
4. **Full-replacement key map model** — When a profile key map is active, it fully replaces the global key map. Keys not defined in the profile map are unassigned, not inherited. This is a deliberate design choice matching ISPF semantics. [FFE-FKEYS]
5. **Graceful degradation** — A missing or corrupt history file never prevents startup; the workbench initialises with an empty history. [WB, FFE-FKEYS]

### Source References

- **[FFE-FKEYS]** = FileForgeEditor `function-keys-and-command-history` specification (10 requirements — priority source)
- **[WB]** = Workbench Platform Architecture Brief (command-driven architecture, GUI independence, configuration-as-data)

### Cross-References

- **`command-framework`** — Function key presses dispatch commands through the command registry and execution pipeline. RETRIEVE is a registered command. Key execution produces Command_History entries (via the standard history rules).
- **`configuration-system`** — Key maps and history settings are stored in TOML configuration files and obey the layered precedence model (Defaults → System → User → Profile → Project → Workspace). Language profile key maps live in `languages/*.toml` files.
- **`startup-and-session`** — History_Store loading occurs during the startup sequence. History persistence occurs during the exit sequence. Graceful degradation rules apply to corrupt/missing History_Store files.
- **`menu-and-statusbar`** — The Key_Label_Bar occupies the footer region alongside (or adjacent to) the Status_Bar. The Primary_Command_Field is the target for RETRIEVE recall and history dropdown interaction.

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **Key_Map** | A named collection of function-key-to-command assignments. Each entry maps a Function_Key to a command string and an optional display label. | [FFE-FKEYS] |
| **Global_Key_Map** | The Key_Map loaded from the workbench configuration that applies when no Profile_Key_Map overrides it. | [FFE-FKEYS] |
| **Profile_Key_Map** | A Key_Map associated with a specific Language_Profile that fully replaces the Global_Key_Map when that profile is active. | [FFE-FKEYS] |
| **Language_Profile** | A per-language configuration file (e.g., `languages/cobol.toml`) that controls syntax, keywords, and may optionally define a Profile_Key_Map via a `[key_map]` section. | [FFE-FKEYS] |
| **Function_Key** | A keyboard key in the set F1–F24, subject to platform key availability. | [FFE-FKEYS] |
| **Key_Label_Bar** | The UI region rendered in the workbench footer that displays the current function key assignments as labelled slots. | [FFE-FKEYS] |
| **Key_Map_Resolver** | The subsystem that selects the active Key_Map by evaluating the Global_Key_Map and any Profile_Key_Map for the active Language_Profile. | [FFE-FKEYS] |
| **Command_History** | The ordered, deduplicated, bounded list of previously entered primary commands maintained per user across sessions. | [FFE-FKEYS] |
| **History_Store** | The on-disk file where Command_History is persisted between workbench sessions. | [FFE-FKEYS] |
| **RETRIEVE** | A primary command registered in the command framework that recalls a previously entered command into the Primary_Command_Field without executing it. | [FFE-FKEYS] |
| **Retrieve_Pointer** | An internal cursor into Command_History that advances backward on each successive RETRIEVE invocation and resets when a non-RETRIEVE command is submitted. | [FFE-FKEYS] |
| **History_Dropdown** | The interactive list control attached to the Primary_Command_Field that exposes Command_History for mouse or keyboard selection. | [FFE-FKEYS] |
| **Primary_Command_Field** | The single-line text input labelled "Command ===>" positioned in the command area above the editor, used for direct ISPF-style command entry. Defined in `menu-and-statusbar`. | [FFE-FKEYS] |
| **Excluded_Command** | A command that is never added to Command_History (UNDO, REDO, RETRIEVE). | [FFE-FKEYS, WB] |

---

## Requirements

### Requirement 1: Global Default Key Map

**User Story:** As a workbench user, I want a global set of function key assignments that works regardless of which file I am editing, so that I have consistent keyboard shortcuts for common commands without any per-language configuration.

**Source:** FFE Reqs 1, 10.1. [FFE-FKEYS]

#### Acceptance Criteria

1. THE Key_Map_Resolver SHALL load a Global_Key_Map from the workbench configuration at startup, reading the `[global_key_map]` section from the effective configuration (layered model).
2. WHEN no Profile_Key_Map exists for the active Language_Profile, THE Key_Map_Resolver SHALL apply the Global_Key_Map for all function key lookups.
3. THE Global_Key_Map SHALL support assignment of any Function_Key in the range F1–F24 to any valid registered Command_ID string or macro invocation string.
4. WHEN the Global_Key_Map configuration is absent or empty, THE Key_Map_Resolver SHALL apply an empty key map, leaving all function keys unassigned.
5. IF the Global_Key_Map configuration contains a key identifier outside the F1–F24 range, THEN THE Key_Map_Resolver SHALL reject that entry, emit a configuration warning via the logging subsystem, and continue loading remaining entries without preventing startup.

---

### Requirement 2: Profile-Specific Key Map

**User Story:** As a COBOL or ABAP developer, I want language-specific function key assignments that activate automatically when I open a file of that type, so that frequently used language-specific commands are one key away.

**Source:** FFE Reqs 2, 10.2. [FFE-FKEYS]

#### Acceptance Criteria

1. WHEN a Language_Profile is active, THE Key_Map_Resolver SHALL check whether a Profile_Key_Map is defined for that Language_Profile (presence of a `[key_map]` section in the language TOML file).
2. WHEN a Profile_Key_Map is defined, THE Key_Map_Resolver SHALL apply the Profile_Key_Map in place of the Global_Key_Map for all Function_Key lookups. This is a full-replacement model: the entire Global_Key_Map is inactive while a Profile_Key_Map is in effect.
3. THE Profile_Key_Map SHALL be defined within the Language_Profile's TOML configuration file (e.g., `languages/cobol.toml`) using a `[key_map]` section with the same schema as the `[global_key_map]` section.
4. WHEN a Profile_Key_Map is removed (its `[key_map]` section deleted or the language profile file changed), THE Key_Map_Resolver SHALL fall back to the Global_Key_Map without requiring a workbench restart. This leverages the configuration-system hot-reload mechanism.
5. THE Profile_Key_Map MAY define only a subset of function keys; any function keys not mentioned in the Profile_Key_Map SHALL be unassigned (not inherited from the Global_Key_Map).
6. WHEN the active Language_Profile changes due to a file type switch or active tab change, THE Key_Map_Resolver SHALL recompute the active Key_Map and update the Key_Label_Bar within the same rendering frame.

---

### Requirement 3: Function Key Execution

**User Story:** As a workbench user, I want to press a function key to immediately execute its assigned command, so that I can perform frequent operations without typing on the command line.

**Source:** FFE Req 3. [FFE-FKEYS]

#### Acceptance Criteria

1. WHEN a Function_Key is pressed and that key has an assignment in the active Key_Map, THE workbench SHALL dispatch the assigned command string through the command framework as if the user had typed it on the Primary_Command_Field and pressed Enter.
2. WHEN a Function_Key is pressed and that key has no assignment in the active Key_Map, THE workbench SHALL produce no action and display no error.
3. THE assigned command string for a Function_Key SHALL support the full primary command syntax, including arguments and modifiers (e.g., `FIND 'ERROR' ALL`).
4. THE assigned command string for a Function_Key SHALL support macro invocation syntax (e.g., `MACRO myfix`).
5. WHEN a Function_Key executes a command that is not in the Excluded_Command set, THE workbench SHALL add the executed command to Command_History using the standard deduplication and ordering rules.
6. WHEN a Function_Key executes a command that IS in the Excluded_Command set (UNDO, REDO, RETRIEVE), THE workbench SHALL NOT add that command to Command_History.

---

### Requirement 4: Key Label Bar Display

**User Story:** As a workbench user, I want to see the current function key assignments displayed at the bottom of the workbench window, so that I can discover and remember my key bindings without consulting documentation.

**Source:** FFE Req 4. [FFE-FKEYS]

#### Acceptance Criteria

1. THE Key_Label_Bar SHALL be rendered in the workbench footer region, below the main editing surface. It SHALL be simultaneously visible with the Status_Bar defined in `menu-and-statusbar`. The exact relative positioning (separate row, combined row, or stacked) is an implementation decision.
2. THE Key_Label_Bar SHALL display the key name (e.g., "F3") and a short label for each assigned Function_Key in the active Key_Map.
3. WHEN a Function_Key has no assignment in the active Key_Map, THE Key_Label_Bar SHALL display that key's slot as blank or omit it entirely.
4. THE Key_Label_Bar label for each key SHALL be derived from the first token of the assigned command string (e.g., command `"FIND 'ERROR' ALL"` yields label `"FIND"`) unless an explicit label is configured.
5. WHERE an explicit short label is configured for a Function_Key assignment (via the `label` field in the key map table), THE Key_Label_Bar SHALL display that explicit label instead of the derived label.
6. WHEN the active Key_Map changes (due to profile switch, configuration hot-reload, or tab change), THE Key_Label_Bar SHALL update its display in the same rendering frame as the Key_Map change.

---

### Requirement 5: RETRIEVE Command — Single-Step Recall

**User Story:** As a command-line user, I want to type RETRIEVE to recall my previous command into the command field, so that I can quickly reuse or edit a recently typed command without retyping it.

**Source:** FFE Req 5. [FFE-FKEYS]

#### Acceptance Criteria

1. WHEN the user submits the primary command `RETRIEVE` on the Primary_Command_Field, THE workbench SHALL place the command at the current Retrieve_Pointer position into the Primary_Command_Field without executing it.
2. WHEN `RETRIEVE` is invoked and the Retrieve_Pointer is at its initial position (no prior retrieval in the current cycle), THE workbench SHALL set the Retrieve_Pointer to the most recent entry in Command_History and display that entry in the Primary_Command_Field.
3. WHEN `RETRIEVE` is invoked again without an intervening non-RETRIEVE command submission, THE workbench SHALL advance the Retrieve_Pointer one step further back (older) in Command_History and display that older entry.
4. WHEN `RETRIEVE` is invoked and the Retrieve_Pointer has already reached the oldest entry in Command_History, THE workbench SHALL display a status message indicating no older history entry exists and SHALL NOT modify the Primary_Command_Field content.
5. WHEN the user submits any command other than `RETRIEVE`, THE Retrieve_Pointer SHALL reset to its initial position. Subsequent RETRIEVE invocations start from the most recent entry again.
6. THE `RETRIEVE` command itself SHALL NOT be added to Command_History (it is an Excluded_Command).
7. IF Command_History is empty and the user submits `RETRIEVE`, THEN THE workbench SHALL display a status message indicating that Command_History is empty and SHALL NOT modify the Primary_Command_Field.

---

### Requirement 6: Command History Storage and Persistence

**User Story:** As a daily workbench user, I want my command history to survive when I close and reopen the workbench, so that I can recall commands from previous work sessions.

**Source:** FFE Req 6. [FFE-FKEYS]

#### Acceptance Criteria

1. THE History_Store SHALL persist Command_History to disk in a human-readable TOML format (consistent with the configuration-system's file format choice).
2. WHEN the workbench starts, THE Command_History subsystem SHALL load Command_History from the History_Store if the History_Store file exists. This loading occurs during the startup sequence (see `startup-and-session`).
3. WHEN the workbench exits normally, THE Command_History subsystem SHALL write the current Command_History to the History_Store as part of the exit sequence.
4. THE History_Store file path SHALL be configurable via the configuration-system using the `history_file` key. The default location SHALL be within the User_Data_Dir (as defined in `startup-and-session`), ensuring history is shared across all projects opened by the same user.
5. IF the History_Store file is absent at startup, THEN THE Command_History subsystem SHALL initialize an empty Command_History without error (graceful degradation).
6. IF the History_Store file is corrupt or unparseable at startup, THEN THE Command_History subsystem SHALL initialize an empty Command_History, emit a WARN-level log record via the logging subsystem identifying the file path and parse error, and continue startup without failure.
7. THE History_Store SHALL record entries in most-recent-first order.

---

### Requirement 7: Command History Deduplication

**User Story:** As a workbench user, I want repeated commands to appear only once in my history, so that my history list stays clean and the most recent use is always at the top.

**Source:** FFE Req 7. [FFE-FKEYS]

#### Acceptance Criteria

1. WHEN a command is added to Command_History and an identical entry already exists anywhere in Command_History, THE Command_History subsystem SHALL remove the existing duplicate and insert the new entry at the front of Command_History (most-recent-first promotion).
2. THE deduplication comparison SHALL be case-insensitive on the command name (first token) and case-preserving on arguments (remaining tokens). Example: `"find 'ERROR'"` and `"FIND 'ERROR'"` are considered duplicates; `"FIND 'ERROR'"` and `"FIND 'error'"` are NOT duplicates (argument case differs).
3. WHEN a command is added to Command_History and no duplicate exists, THE Command_History subsystem SHALL insert the new entry at the front of Command_History.

---

### Requirement 8: Command History Exclusion Rules

**User Story:** As a workbench user, I want utility commands like UNDO, REDO, and RETRIEVE excluded from my history, so that my history contains only substantive commands I might want to recall.

**Source:** Derived from FFE Req 5.6, Req 3, WB command framework history logging rules. [FFE-FKEYS, WB]

#### Acceptance Criteria

1. THE Command_History subsystem SHALL maintain a set of Excluded_Commands that are never added to Command_History regardless of invocation source (typed, function key, macro, menu).
2. THE default Excluded_Command set SHALL contain: `RETRIEVE`, `UNDO`, `REDO`.
3. THE Excluded_Command set SHALL be configurable via the configuration-system, allowing users to add additional commands to the exclusion list.
4. WHEN an Excluded_Command is submitted on the Primary_Command_Field or dispatched via a function key, THE Command_History subsystem SHALL NOT record it in Command_History and SHALL NOT affect the Retrieve_Pointer position.

---

### Requirement 9: Configurable History Capacity

**User Story:** As a power user, I want to configure the maximum number of history entries the workbench retains, so that I can tune memory and persistence behaviour to match my workflow.

**Source:** FFE Req 8. [FFE-FKEYS]

#### Acceptance Criteria

1. THE configuration-system SHALL accept a `max_history_entries` integer setting (under the appropriate configuration namespace) that controls the maximum number of entries retained in Command_History.
2. WHEN `max_history_entries` is not configured, THE Command_History subsystem SHALL apply a default maximum of 200 entries.
3. WHEN adding a new entry would cause Command_History to exceed `max_history_entries`, THE Command_History subsystem SHALL remove the oldest entry (tail of the list) before inserting the new one at the front.
4. IF `max_history_entries` is set to zero or a negative value in the configuration, THEN THE Command_History subsystem SHALL apply the default maximum of 200 entries and emit a configuration warning via the logging subsystem.

---

### Requirement 10: History Dropdown on Primary Command Field

**User Story:** As a workbench user, I want to open a dropdown on the command input field to browse and select from my command history, so that I can recall any previous command without cycling through it one step at a time with RETRIEVE.

**Source:** FFE Req 9. [FFE-FKEYS]

#### Acceptance Criteria

1. THE Primary_Command_Field SHALL provide a History_Dropdown control that exposes Command_History as a selectable list.
2. WHEN the user navigates the History_Dropdown using the up/down arrow keys, THE Primary_Command_Field SHALL update to show the highlighted history entry without submitting it.
3. WHEN the user selects a history entry from the History_Dropdown (via mouse click or Enter key while dropdown is focused), THE Primary_Command_Field SHALL populate with the selected entry without executing it.
4. WHEN a history entry is selected via the History_Dropdown, THE Retrieve_Pointer SHALL be set to point at the selected entry, so that subsequent RETRIEVE invocations continue backward from that position.
5. WHEN Command_History is empty, THE History_Dropdown SHALL display an empty state indicator (e.g., "No history") rather than showing nothing.
6. THE History_Dropdown SHALL display entries in most-recent-first order, consistent with Command_History ordering.

---

### Requirement 11: Configuration Schema for Function Keys and History

**User Story:** As a system administrator or power user, I want all key map and history settings specified in the existing TOML configuration files, so that the feature fits naturally into the workbench's established configuration model.

**Source:** FFE Req 10. [FFE-FKEYS]

#### Acceptance Criteria

1. THE workbench configuration SHALL accept a `[global_key_map]` section where each key is a function key name (e.g., `F3`, `F12`) and each value is either a plain command string or a TOML table with `command` (required) and `label` (optional) fields.
2. THE language profile TOML files (e.g., `languages/cobol.toml`) SHALL accept an optional `[key_map]` section using the same schema as the `[global_key_map]` section.
3. THE workbench configuration SHALL accept a `max_history_entries` integer field controlling the maximum history size.
4. THE workbench configuration SHALL accept a `history_file` string field specifying the path to the History_Store file. Relative paths SHALL be resolved relative to the User_Data_Dir.
5. THE workbench configuration SHALL accept a `history_excluded_commands` array-of-strings field allowing additional commands to be added to the Excluded_Command set beyond the defaults.
6. WHEN a configuration field for this feature contains an invalid value type, THE Configuration_System SHALL emit a descriptive warning identifying the field name and expected type, and SHALL apply the default value for that field without preventing startup.
7. ALL configuration keys defined in this spec SHALL participate in the configuration-system's hot-reload mechanism. Changes to `[global_key_map]` SHALL take effect without workbench restart. Changes to `max_history_entries` SHALL take effect on the next command addition (existing entries beyond the new limit are trimmed).

---

### Requirement 12: PFSHOW Command — Key Label Bar Visibility Toggle

**User Story:** As a workbench user, I want to show or hide the Key Label Bar with a command, so that I can reclaim screen space when I know my key assignments or reveal them when I need a reminder.

**Source:** New requirement — ISPF-style PFSHOW command.

#### Acceptance Criteria

12.1. WHEN the user submits the primary command `PFSHOW ON`, THE workbench SHALL make the Key_Label_Bar visible in the footer region if it is not already visible.
12.2. WHEN the user submits the primary command `PFSHOW OFF`, THE workbench SHALL hide the Key_Label_Bar from the footer region.
12.3. WHEN the user submits the primary command `PFSHOW` with no argument, THE workbench SHALL toggle the Key_Label_Bar visibility: if currently visible it SHALL be hidden; if currently hidden it SHALL be made visible.
12.4. THE PFSHOW visibility state SHALL be persisted in the session state so that the Key_Label_Bar is restored to its last-known visibility on the next workbench launch.
12.5. THE `PFSHOW` command SHALL be registered in the command framework with Command_ID `"keys.pfshow"` and SHALL be invocable from the Primary_Command_Field.
12.6. WHEN `PFSHOW ON` is issued and the bar is already visible, THE workbench SHALL produce no visible change and SHALL NOT emit an error.
12.7. WHEN `PFSHOW OFF` is issued and the bar is already hidden, THE workbench SHALL produce no visible change and SHALL NOT emit an error.

---

### Requirement 13: Key Label Bar — Two-Row Layout for 24 Keys

**User Story:** As a workbench user, I want the Key Label Bar to display all 24 function key assignments across two rows at the bottom of the window, so that I can see the full set of available shortcuts at a glance.

**Source:** New requirement — extension of Requirement 4 to support F1–F24 in a two-row layout.

#### Acceptance Criteria

13.1. THE Key_Label_Bar SHALL display function key assignments in two rows of up to 12 slots each: the first row SHALL display F1–F12 and the second row SHALL display F13–F24.
13.2. WHEN a function key has no assignment in the active Key_Map, THE Key_Label_Bar SHALL display that key's slot as blank (key name shown, label area empty) rather than omitting the slot entirely, so that the two-row grid layout is preserved.
13.3. THE Key_Label_Bar SHALL display each slot as a pair: the key name (e.g., "F3") followed by the short label (e.g., "END"), separated by a space or visual divider consistent with the active theme.
13.4. THE two-row layout SHALL be rendered in the workbench footer region below the main editing surface, occupying at most two lines of display height.
13.5. WHEN the Key_Label_Bar is visible and the active Key_Map changes, THE two-row display SHALL update within the same rendering frame.

---

### Requirement 14: Per-Context Key Map

**User Story:** As a workbench user, I want each window context (POM, editor, settings panel, file browser, etc.) to have its own function key assignments that load automatically when that context becomes active, so that the most relevant shortcuts are always available for the current task.

**Source:** New requirement — extends Requirement 2 (Profile-Specific Key Map) to cover all window contexts, not just language profiles.

#### Acceptance Criteria

14.1. THE Key_Map_Resolver SHALL support a Context_Key_Map for each named window context. A Context_Key_Map is defined in the workbench configuration under a `[context_key_maps.<context_name>]` section using the same schema as `[global_key_map]`.
14.2. WHEN a window context is loaded into a tab (e.g., POM, SettingsPanel, FilesPanel, FileEditor, HexDisplay), THE Key_Map_Resolver SHALL check whether a Context_Key_Map is defined for that context name and, if so, activate it as the effective key map for that tab.
14.3. WHEN no Context_Key_Map is defined for the active context, THE Key_Map_Resolver SHALL apply the Global_Key_Map as the effective key map.
14.4. WHEN the active tab changes, THE Key_Map_Resolver SHALL recompute the effective key map for the newly active tab's context and update the Key_Label_Bar within the same rendering frame.
14.5. THE Context_Key_Map model SHALL use the same full-replacement semantics as the Profile_Key_Map: when a Context_Key_Map is active, the Global_Key_Map is entirely inactive for that context; keys not defined in the Context_Key_Map are unassigned.
14.6. THE context name used for lookup SHALL be a stable string identifier assigned to each tab kind: `"pom"` for the Primary Option Menu, `"editor"` for file editor tabs, `"settings"` for the Settings Panel, `"files"` for the Files Panel, `"hex"` for hex display mode, `"toolchain"` for the Toolchain Panel.
14.7. THE configuration system SHALL accept a `[context_key_maps]` section at the top level of the workbench configuration, containing one sub-table per context name.

---

### Requirement 15: Default 24-Key Assignment Set

**User Story:** As a new workbench user, I want a sensible default set of 24 function key assignments pre-configured out of the box, so that common operations are immediately accessible without any manual configuration.

**Source:** New requirement — defines the initial default key map for the workbench.

#### Acceptance Criteria

15.1. THE workbench SHALL ship with a built-in default Global_Key_Map containing the following assignments as the baseline when no user configuration overrides them:

| Key | Command | Label |
|-----|---------|-------|
| F1  | HELP    | Help  |
| F3  | END     | End   |
| F7  | UP MAX  | Up    |
| F8  | DOWN MAX | Down |
| F12 | RETRIEVE | Retrieve |

15.2. THE built-in default assignments for F2, F4–F6, F9–F11, F13–F24 SHALL be unassigned in the baseline default map, leaving those slots blank in the Key_Label_Bar until the user configures them.
15.3. THE built-in default key map SHALL be overridable in full by providing a `[global_key_map]` section in the user configuration file; user-provided entries replace the built-in defaults entirely (full-replacement model).
15.4. THE built-in default key map SHALL be documented in the workbench help system under Topic_Key `"feature:function_keys"`.

---

### Requirement 16: Key Label Bar Hotspots

**User Story:** As a workbench user, I want to click on a function key label in the Key Label Bar to execute that key's assigned command, so that I can trigger function key actions with the mouse without pressing the physical key.

**Source:** New requirement — mouse-clickable Key_Label_Bar slots.

#### Acceptance Criteria

16.1. WHEN the Key_Label_Bar is visible and the user clicks on a slot that has an assigned command, THE workbench SHALL dispatch that slot's command through the command framework exactly as if the corresponding function key had been pressed.
16.2. WHEN the user clicks on a slot that has no assignment (blank label), THE workbench SHALL produce no action and no error.
16.3. THE clickable area for each slot SHALL encompass both the key name and the label text, providing a generous hit target.
16.4. WHEN the user hovers over an assigned slot, THE workbench SHALL display a tooltip showing the full command string assigned to that key (e.g., "UP MAX" for a slot labelled "Up").
16.5. THE hotspot click SHALL follow the same history and exclusion rules as a physical function key press: the dispatched command is recorded in Command_History unless it is an Excluded_Command.

---

### Requirement 17: END and RETURN Navigation Commands

**User Story:** As a workbench user, I want END to close the current context and return to the previous screen, and RETURN to jump directly back to the Primary Option Menu from anywhere, so that I can navigate the workbench hierarchy efficiently.

**Source:** New requirement — ISPF-style END and RETURN navigation semantics.

#### Acceptance Criteria

17.1. WHEN the user submits the primary command `END` (or presses the key assigned to END), THE workbench SHALL close the current context tab and navigate to the tab that was active immediately before the current context was opened. If no prior tab exists, the workbench SHALL navigate to the POM tab.
17.2. WHEN `END` is issued from the POM tab, THE workbench SHALL treat it as equivalent to `EXIT` and terminate the application (after any unsaved-changes prompts).
17.3. WHEN the user submits the primary command `RETURN` (or presses the key assigned to RETURN), THE workbench SHALL navigate directly to the POM tab, making it the active tab, regardless of the current context depth.
17.4. WHEN `RETURN` is issued from the POM tab, THE workbench SHALL treat it as equivalent to `EXIT` and terminate the application (after any unsaved-changes prompts).
17.5. THE `END` command SHALL be registered in the command framework with Command_ID `"nav.end"` and SHALL be invocable from the Primary_Command_Field and via function key assignment.
17.6. THE `RETURN` command SHALL be registered in the command framework with Command_ID `"nav.return"` and SHALL be invocable from the Primary_Command_Field and via function key assignment.
17.7. NEITHER `END` NOR `RETURN` SHALL be added to Command_History (they are navigation meta-commands, not substantive editing commands). Both SHALL be added to the Excluded_Command set.

---

### Requirement 18: Contextual Help — "Not Available Yet" Fallback

**User Story:** As a workbench user, I want pressing F1 (or the key assigned to HELP) to always produce a response, even when no specific help content exists for the current cursor position, so that I am never left wondering whether the key worked.

**Source:** New requirement — extends context-help Requirement 1 with an explicit "not available yet" fallback dialog.

#### Acceptance Criteria

18.1. WHEN F1 is pressed (or the HELP command is dispatched) and the Context_Detector resolves a specific Topic_Key but no help content exists for that key in the Help_Topic_Registry, THE workbench SHALL display a non-modal informational message reading: "Help not available yet for: <context>. Press F1 again or type HELP for the Help Index."
18.2. THE "not available yet" message SHALL be displayed in the status bar or as a brief overlay notification — it SHALL NOT open the full Help_Panel.
18.3. WHEN F1 is pressed and the Context_Detector cannot resolve any specific context (generic UI element), THE workbench SHALL open the Help_Panel displaying the Help_Index (existing behaviour per context-help Requirement 1.7).

---

### Requirement 19: RETRIEVE with LIST — History Browser

**User Story:** As a workbench user, I want to type "LIST" in the command field and press the RETRIEVE key to see a deduplicated list of my previously typed commands, so that I can browse and select from my full history without cycling through it one entry at a time.

**Source:** New requirement — ISPF-style history list triggered by LIST + RETRIEVE.

#### Acceptance Criteria

19.1. WHEN the Primary_Command_Field contains the text `LIST` (case-insensitive) AND the user invokes the RETRIEVE command (by typing RETRIEVE, pressing the key assigned to RETRIEVE, or pressing the RETRIEVE function key), THE workbench SHALL display the Command_History as a selectable list rather than performing single-step recall.
19.2. THE history list display SHALL show Command_History entries in most-recent-first order, deduplicated per the standard deduplication rules (Requirement 7).
19.3. WHEN the user selects an entry from the history list (via mouse click or keyboard navigation + Enter), THE workbench SHALL populate the Primary_Command_Field with the selected command text without executing it, and SHALL close the history list.
19.4. WHEN the user dismisses the history list without selecting an entry (via Escape or clicking outside), THE Primary_Command_Field SHALL be cleared and the history list SHALL close.
19.5. WHEN Command_History is empty and the LIST+RETRIEVE trigger is activated, THE workbench SHALL display the history list with an empty-state message: "No command history."
19.6. THE `LIST` text in the command field SHALL NOT itself be added to Command_History when used as the RETRIEVE trigger.
19.7. THE history list SHALL be rendered as a modal or near-modal overlay anchored to the Primary_Command_Field, consistent with the History_Dropdown defined in Requirement 10 but triggered by the LIST keyword rather than a dropdown control.

---

### Requirement 20: Key Configuration Dialog

**User Story:** As a workbench user, I want a graphical dialog where I can view and edit all function key assignments — for the default global map and for each named context — including plain, Shift, Ctrl, and Alt modifier variants, with a command string and a description for each binding, so that I can configure my key maps without editing TOML files manually.

**Source:** New requirement — Phase AN.

#### Acceptance Criteria

20.1. THE workbench SHALL provide a Key_Configuration_Dialog accessible via the command `KEYS` entered in the Primary_Command_Field, and via a menu item (e.g., `Edit > Key Assignments…`).

20.2. THE Key_Configuration_Dialog SHALL display a tab or selector for each configurable key map scope: one tab labelled **Default (Global)** and one tab per named context (`pom`, `editor`, `settings`, `files`, `hex`, `toolchain`).

20.3. WITHIN each scope tab, THE dialog SHALL display a grid of 24 rows — one per function key F1–F24 — with the following columns:

| Column | Content |
|--------|---------|
| Key | Key name (e.g., `F3`) — read-only |
| Command | Editable text field for the plain (unmodified) key command string |
| Description | Editable text field for a human-readable description of what the command does |
| Shift+Key Command | Editable text field for the Shift+Fn command string |
| Shift+Key Description | Editable text field for the Shift+Fn description |
| Ctrl+Key Command | Editable text field for the Ctrl+Fn command string |
| Ctrl+Key Description | Editable text field for the Ctrl+Fn description |
| Alt+Key Command | Editable text field for the Alt+Fn command string |
| Alt+Key Description | Editable text field for the Alt+Fn description |

20.4. WHEN the user edits a Command field and moves focus away (or presses Enter), THE dialog SHALL validate that the command string is non-empty if provided; an empty string SHALL be treated as "unassigned" (clearing the binding).

20.5. THE dialog SHALL provide **Save** and **Cancel** buttons. WHEN **Save** is clicked, THE dialog SHALL write all changes to the workbench configuration (user-layer TOML) and close. WHEN **Cancel** is clicked, THE dialog SHALL discard all unsaved changes and close.

20.6. WHEN the dialog opens, THE dialog SHALL pre-populate all fields from the currently effective key map for each scope (global map for the Default tab; the registered context map for each context tab), showing blank fields for unassigned keys.

20.7. THE dialog SHALL display the current effective label for each plain key binding in a read-only **Label** column adjacent to the Command column, derived using the same label-derivation rules as the Key_Label_Bar (explicit label if set, otherwise first token of command).

20.8. WHEN the user saves changes to the Default (Global) scope, THE workbench SHALL update the `[global_key_map]` section in the user-layer configuration file. WHEN the user saves changes to a context scope, THE workbench SHALL update the corresponding `[context_key_maps.<name>]` section.

20.9. THE Key_Configuration_Dialog SHALL support modifier-key bindings (Shift+Fn, Ctrl+Fn, Alt+Fn) as independent assignments stored alongside the plain binding. Each modifier variant has its own command string and description, independent of the plain binding.

20.10. WHEN a modifier-key binding is assigned in the dialog and the user presses that modifier+key combination in the workbench, THE workbench SHALL dispatch the modifier binding's command string through the command framework, following the same history and exclusion rules as plain function key presses.

20.11. THE modifier key bindings SHALL be stored in the TOML configuration using an extended key name syntax: `SF1`–`SF24` for Shift, `CF1`–`CF24` for Ctrl, `AF1`–`AF24` for Alt, within the same `[global_key_map]` or `[context_key_maps.<name>]` section.

20.12. THE `FunctionKey` type (or a new `ModifiedKey` type) SHALL be extended to represent the four modifier variants (plain, Shift, Ctrl, Alt) for each of F1–F24, giving a total of 96 addressable key slots per key map.

20.13. THE Key_Label_Bar SHALL continue to display only the plain (unmodified) F1–F24 bindings in its two-row layout. Modifier bindings are not shown in the Key_Label_Bar but are accessible via the Key_Configuration_Dialog and active at runtime.

20.14. WHEN the Key_Configuration_Dialog is open, THE workbench SHALL continue to process function key presses normally (the dialog is non-blocking with respect to the rest of the workbench).

20.15. THE dialog SHALL include a **Reset to Defaults** button per scope tab. WHEN clicked, THE dialog SHALL restore all fields in that tab to the built-in defaults (for the Default tab) or clear all fields (for context tabs), without saving until **Save** is clicked.
