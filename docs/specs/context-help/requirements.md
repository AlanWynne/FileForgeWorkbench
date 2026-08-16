# Requirements Document

## Introduction

This feature specifies the **context-sensitive help system** for FileForgeWorkbench (`ff-help` crate). The help system provides an integrated, non-modal, searchable help facility inspired by the ISPF Tutorial/Help model. It delivers context-aware help content through a dockable Help Panel, keyboard-triggered context detection (F1), a searchable topic library, and a navigable topic hierarchy — all without disrupting the user's editing workflow.

The help system integrates with:
- The **command framework** (`command-framework`) — commands register help text via `CommandMetadata`, and the `HELP` primary command dispatches into this system.
- The **layout and docking system** (`layout-and-docking`) — the Help Panel participates as a `DockablePanel` trait implementor, dockable to any zone.
- The **plugin architecture** (`plugin-architecture`) — plugins can contribute additional help topics for their registered commands.
- The **command semantics** (`command-semantics`) — the HELP primary command (Requirement 7 in that spec) routes through this help infrastructure.

The `HELP` primary command defined in `command-semantics` Requirement 7 routes through this help infrastructure — typing `HELP CHANGE` on the command line and pressing F1 while typing a CHANGE command both arrive at the same help content via the Help Panel.

### Design Principles

- **F1 always works.** Regardless of mode, panel focus, or editor state, pressing F1 produces a help response. If no specific context is detected, the system opens the Help Index.
- **Context detection is best-effort.** The system inspects the current focus, command line content, and active mode to determine the most relevant topic. If ambiguous, it offers the closest match.
- **Help does not disrupt the workflow.** The Help Panel is non-modal: the user can read help while editing, and dismiss it without losing state.
- **Help content is data, not code.** Help topics are authored in Markdown files, loadable at runtime, replaceable per deployment, and translatable.
- **Help is extensible.** Commands, plugins, and macros can register help topics at runtime through the command registry and the help topic registry.

**Source references:**
- **[FFE-HELP]** = FileForgeEditor `context-help` specification (13 requirements — all incorporated and adapted)
- **[WB]** = Workbench Architecture Brief — command-driven architecture, plugin model, docking system

## Glossary

- **Help_System**: The subsystem responsible for context detection, topic resolution, content loading, and Help Panel rendering. Implemented in `ff-help` crate. [FFE-HELP]
- **Help_Panel**: A dockable panel (implementing `DockablePanel` trait from `layout-and-docking`) used to display help content. Participates in the workbench layout system. [FFE-HELP, WB]
- **Help_Topic**: A named unit of help content — one topic per command, line command, feature, mode, or configuration key. Identified by a Topic_Key. [FFE-HELP]
- **Topic_Key**: A string identifier for a Help_Topic, used in lookups (e.g., `"cmd:CHANGE"`, `"line:CC"`, `"mode:hex"`, `"feature:undo"`, `"config:help_panel_position"`). [FFE-HELP]
- **Help_Index**: The top-level help page listing all available topic categories with navigable links to individual topics. [FFE-HELP]
- **Context_Detector**: The component that inspects the current editor state (focused panel, command line text, active mode, cursor position, active line command) to determine the most relevant Topic_Key. [FFE-HELP]
- **Help_Topic_Registry**: The runtime collection of all available help topics, supporting O(1) lookup by Topic_Key and keyword search. Populated from file-based content and command-registered help text. [FFE-HELP, WB]
- **Help_Content_File**: A Markdown (`.help.md`) file containing the help content for one or more topics, stored in the `help/` content directory. [FFE-HELP]
- **Help_Search**: A keyword search facility within the Help Panel that searches across all loaded Help_Topics by title and body content. [FFE-HELP]
- **Help_Navigation_Stack**: A back/forward navigation history within the Help Panel, allowing the user to return to previously viewed topics. [FFE-HELP]
- **Help_Menu**: The "Help" top-level menu in the workbench menu bar, providing access to the Help Panel, About dialog, and reference pages. [FFE-HELP]
- **Command_Input_Context**: The state of the command input field when F1 is pressed — specifically, the command name token (if any) currently typed. [FFE-HELP]
- **Prefix_Area_Context**: The state of the prefix area when F1 is pressed — specifically, the line command text (if any) in the focused prefix cell. [FFE-HELP]
- **Mode_Context**: The active editor mode (Browse, Edit, View, Hex, Preview, FileForge Grid_Browse, FileForge Grid_Edit) when F1 is pressed. [FFE-HELP]

---

## Requirements

### Requirement 1: F1 Key — Context-Sensitive Help Activation

**User Story:** As a workbench user, I want to press F1 at any time to get help relevant to what I am currently doing, so that I can learn and confirm command syntax without leaving the editor or searching documentation manually.

**Source:** [FFE-HELP] Requirement 1. Cross-references: `command-framework` (reserved shortcuts), `command-semantics` (HELP command).

#### Acceptance Criteria

1.1. WHEN the user presses F1, THE Help_System SHALL determine the current context using the Context_Detector and open the Help_Panel displaying the most relevant Help_Topic. [FFE-HELP]

1.2. WHEN the command input field has keyboard focus and contains a recognisable command name (first whitespace-delimited token), THE Context_Detector SHALL resolve the Topic_Key to `"cmd:<COMMAND_NAME>"` and display help for that command. [FFE-HELP]

1.3. WHEN the command input field has keyboard focus and is empty or contains only whitespace, THE Context_Detector SHALL resolve the Topic_Key to `"index"` and display the Help_Index. [FFE-HELP]

1.4. WHEN a prefix area cell has keyboard focus and contains a recognisable line command, THE Context_Detector SHALL resolve the Topic_Key to `"line:<COMMAND>"` (e.g., `"line:CC"`, `"line:D"`) and display help for that line command. [FFE-HELP]

1.5. WHEN the editor is in a special mode (Hex, Preview, FileForge Grid_Edit, FileForge Grid_Browse), THE Context_Detector SHALL include the mode in context resolution. IF no more specific context is available (command line empty, no focused prefix cell), THE Help_System SHALL display the mode-specific help topic (e.g., `"mode:hex"`, `"mode:grid_edit"`). [FFE-HELP]

1.6. WHEN the Help_Panel is already open and F1 is pressed, THE Help_System SHALL re-evaluate context and navigate to the new topic if it differs from the currently displayed topic. IF the context resolves to the same topic, THE Help_System SHALL close the Help_Panel (toggle behaviour). [FFE-HELP]

1.7. WHEN F1 is pressed and the Context_Detector cannot resolve a specific topic (e.g., focus is on a generic UI element with no associated help), THE Help_System SHALL display the Help_Index. [FFE-HELP]

1.8. THE F1 key binding SHALL be hard-coded as a reserved shortcut (per `command-framework` Requirement 5.3) and SHALL NOT be overridable via the key map system or plugin shortcut registration. F1 always means Help. [FFE-HELP, WB]

1.9. THE F1 help activation SHALL work in all editor modes: Browse, Edit, View, Hex, Preview, and all FileForge special modes. [FFE-HELP]

1.10. THE F1 key press SHALL NOT be added to command history and SHALL NOT be recorded as an undoable transaction. [FFE-HELP]

---

### Requirement 2: Help Panel — Dockable Display

**User Story:** As a workbench user, I want help content displayed in a dedicated, readable, dockable panel that does not obscure my editing work, so that I can reference help while continuing to work.

**Source:** [FFE-HELP] Requirement 2. Cross-references: `layout-and-docking` (DockablePanel trait, dock zones).

#### Acceptance Criteria

2.1. THE Help_Panel SHALL implement the `DockablePanel` trait from the `layout-and-docking` system, participating in dock/undock operations, tab groups, and floating window placement. [FFE-HELP, WB]

2.2. THE Help_Panel SHALL default to the right dock zone, occupying no more than 40% of the window width. [FFE-HELP]

2.3. THE Help_Panel width (when docked to a side zone) SHALL be resizable via the standard dock zone divider provided by the layout system. [FFE-HELP, WB]

2.4. THE Help_Panel SHALL be dismissable by: pressing Escape, pressing F1 again (toggle per 1.6), clicking the panel close button, or issuing the `HELP OFF` primary command. [FFE-HELP]

2.5. WHILE the Help_Panel is open, THE editing area SHALL remain fully functional — the Help_Panel is non-modal. The user can type commands, edit text, and navigate while help is displayed. [FFE-HELP]

2.6. THE Help_Panel SHALL display the topic title at the top, followed by the help content body rendered as formatted text with support for: section headings, bullet lists, indented code examples, bold/highlighted keywords, and cross-reference links. [FFE-HELP]

2.7. THE Help_Panel SHALL be vertically scrollable for topics that exceed the visible height. [FFE-HELP]

2.8. THE Help_Panel SHALL display a breadcrumb or topic path at the top (e.g., `Help > Commands > CHANGE`) showing the current location in the topic hierarchy. [FFE-HELP]

2.9. WHEN the Help_Panel is docked in a zone that is too narrow to render content readably (below 200 pixels width), THE Help_Panel SHALL display a message suggesting undocking or resizing. [WB]

2.10. THE Help_Panel SHALL be keyboard-navigable: Up/Down arrows scroll content, Escape closes the panel, and Tab returns focus to the editing area without closing the panel. [FFE-HELP]

---

### Requirement 3: Help Navigation

**User Story:** As a workbench user, I want to navigate between help topics, go back to previously viewed topics, and follow cross-references, so that I can efficiently find the information I need.

**Source:** [FFE-HELP] Requirement 3.

#### Acceptance Criteria

3.1. THE Help_Panel SHALL maintain a Help_Navigation_Stack that records each topic visited during the current help session. [FFE-HELP]

3.2. THE Help_Panel SHALL display Back and Forward navigation controls (and respond to Alt+Left / Alt+Right keyboard shortcuts) to traverse the Help_Navigation_Stack. [FFE-HELP]

3.3. WHEN the user activates a cross-reference link within help content (e.g., "See also: FIND command"), THE Help_Panel SHALL navigate to the linked topic and push it onto the Help_Navigation_Stack. [FFE-HELP]

3.4. THE Help_Panel SHALL display a Table of Contents (TOC) sidebar or collapsible outline for topics that contain multiple sections, enabling direct navigation to any section within the current topic. [WB]

3.5. THE Help_Panel SHALL display a "Help Index" link or button that always returns to the top-level Help_Index topic regardless of current position. [FFE-HELP]

3.6. WHEN the Help_Panel is closed and reopened, THE Help_Navigation_Stack SHALL be cleared — each F1 press starts a fresh help session from the context-resolved topic. [FFE-HELP]

---

### Requirement 4: Help Search

**User Story:** As a workbench user, I want to search across all help content by keyword, so that I can find relevant topics even when I do not know the exact command name or topic title.

**Source:** [FFE-HELP] Requirement 3.4–3.5 (extracted as separate requirement for clarity).

#### Acceptance Criteria

4.1. THE Help_Panel SHALL display a Search input field at the top. WHEN the user types a search query (minimum 2 characters), THE Help_System SHALL search all loaded Help_Topics for matching keywords in titles and body content, and display a results list with topic titles and matching excerpts. [FFE-HELP]

4.2. THE Help_Search SHALL perform case-insensitive substring matching across topic titles, topic body text, and Topic_Key aliases. [FFE-HELP]

4.3. WHEN the user selects a search result, THE Help_Panel SHALL navigate to that topic and push it onto the Help_Navigation_Stack. [FFE-HELP]

4.4. THE Help_Search results SHALL be ranked by relevance: exact title matches first, then keyword matches in headings, then keyword matches in body text. [WB]

4.5. WHEN a search query produces no results, THE Help_Panel SHALL display "No help topics found for: <query>" with a suggestion to try alternative terms. [FFE-HELP]

---

### Requirement 5: Help Content Format

**User Story:** As a workbench developer or technical writer, I want help content stored in structured Markdown files that are easy to author, version-control, and translate, so that help content can be maintained independently of the application code.

**Source:** [FFE-HELP] Requirement 4 (adapted from plain text to Markdown).

#### Acceptance Criteria

5.1. THE Help_System SHALL load help content from `.help.md` files located in a `help/` directory. The Help_System SHALL search for the `help/` directory in the following locations, in order: (a) the directory containing the workbench binary, (b) the user data directory (`User_Data_Dir`), (c) a custom path specified by the `help_directory` configuration key. [FFE-HELP]

5.2. EACH `.help.md` file SHALL contain one or more Help_Topics, separated by a YAML front-matter block or a topic delimiter line of the format `<!-- TOPIC: topic_key -->` followed by `<!-- TITLE: Human Title -->`. [FFE-HELP, adapted]

5.3. THE help content body SHALL be standard Markdown with the following elements supported for rendering:
  - `# Heading` / `## Sub-heading` — section headings within a topic
  - `- item` — bullet list item
  - `` `code` `` — inline code
  - Fenced code blocks (` ``` `) — multi-line code examples
  - `**bold text**` — bold/highlighted keyword
  - `[link text](topic_key)` — cross-reference link to another Help_Topic by Topic_Key
  - Standard paragraphs and line breaks
[FFE-HELP, adapted]

5.4. THE Help_System SHALL load all `.help.md` files at startup and index them by Topic_Key in the Help_Topic_Registry for O(1) lookup. [FFE-HELP]

5.5. WHEN a referenced Topic_Key does not exist in the loaded content, THE Help_Panel SHALL display "Help topic not found: <key>" with a link back to the Help_Index. [FFE-HELP]

5.6. WHEN the `help/` directory is missing or contains no `.help.md` files, THE Help_System SHALL display a built-in minimal help page explaining that help content files are not installed, and providing the expected file locations. [FFE-HELP]

5.7. THE Help_System SHALL support hot-reload of help content files: WHEN a `.help.md` file is modified on disk while the workbench is running, THE Help_System SHALL detect the change (via VFS file-watcher) and reload the affected topics without requiring a restart. [WB]

---

### Requirement 6: Help Topic Registry

**User Story:** As a workbench developer, I want a centralised topic registry that aggregates help content from file-based sources and runtime registrations (commands, plugins), so that the help system always has a complete, up-to-date view of available topics.

**Source:** [FFE-HELP] Requirement 12 (expanded to workbench scope). Cross-references: `command-framework` (CommandMetadata), `plugin-architecture` (plugin lifecycle).

#### Acceptance Criteria

6.1. THE Help_Topic_Registry SHALL store Help_Topics indexed by Topic_Key, supporting registration from: (a) file-based `.help.md` content loaded at startup, (b) `CommandMetadata.help_text` fields from the command registry, and (c) plugin-contributed topics registered during the plugin `initialize` lifecycle phase. [FFE-HELP, WB]

6.2. THE `CommandMetadata` struct (defined in `command-framework`) SHALL include a `help_text` field (String) containing the help content body and a `help_syntax` field (String) containing the command syntax line. [FFE-HELP]

6.3. WHEN a command is registered with the Command_Registry via the command framework, THE Help_System SHALL automatically create a Help_Topic with Topic_Key `"cmd:<COMMAND_ID>"` from the command's `help_text` and `help_syntax` metadata fields. [FFE-HELP]

6.4. THE Help_Topic_Registry SHALL prefer runtime-registered help text (from Command_Registry or plugins) over file-based help content when both exist for the same Topic_Key. This ensures that dynamically registered commands always have up-to-date help. [FFE-HELP]

6.5. WHEN a command is registered without `help_text` (empty string), THE Help_System SHALL fall back to the file-based help content for that command's Topic_Key. [FFE-HELP]

6.6. WHEN a plugin is unloaded during its `shutdown` lifecycle phase, THE Help_Topic_Registry SHALL remove all topics contributed by that plugin. [WB]

6.7. THE Help_Topic_Registry SHALL be thread-safe: read and write operations SHALL be safe from any thread without requiring the caller to acquire an external lock. [WB]

---

### Requirement 7: Help Content — Primary Commands

**User Story:** As a workbench user, I want every primary command to have a help topic explaining its syntax, modifiers, examples, and related commands, so that I can learn the full capabilities of any command from within the editor.

**Source:** [FFE-HELP] Requirement 5. Cross-references: `command-semantics` (command list).

#### Acceptance Criteria

7.1. THE help content SHALL include one Help_Topic for each registered primary command, with Topic_Key `"cmd:<NAME>"` (e.g., `"cmd:FIND"`, `"cmd:CHANGE"`, `"cmd:SAVE"`). [FFE-HELP]

7.2. EACH primary command help topic SHALL contain at minimum:
  - **Syntax** — the full command syntax with all argument forms and optional modifiers
  - **Description** — a one-paragraph explanation of what the command does
  - **Modifiers** — a list of all supported modifiers with a brief explanation of each
  - **Examples** — at least two concrete usage examples showing common use cases
  - **See Also** — cross-references to related commands
[FFE-HELP]

7.3. THE help content SHALL include topics for all primary commands defined across all specs, including but not limited to: FIND, RFIND, CHANGE, RCHANGE, EXCLUDE, SHOW, INCLUDE, RESET, SORT, SAVE, CANCEL, END, LOAD, RELOAD, DELETE, COPY, MOVE, LOCATE, UP, DOWN, LEFT, RIGHT, TOP, BOTTOM, MACRO, EXEC, RUN, COLS, BOUNDS, BNDS, UNDO, REDO, HEX, PREVIEW, SHELL, TSO, CONVERT, SAVEAS, NEW, REVERT, RETRIEVE, NUMBER, UNNUM, CRITERIA, SELECT, TABS, MASK, HELP, ASA. [FFE-HELP]

7.4. THE help content for commands with aliases SHALL list all aliases (e.g., FIND topic mentions RFIND; SHELL topic mentions TSO; EXCLUDE topic mentions X). [FFE-HELP]

---

### Requirement 8: Help Content — Line Commands

**User Story:** As a workbench user, I want help topics for all line commands explaining their syntax, block forms, and interactions, so that I can learn prefix-area commands without memorising the full reference.

**Source:** [FFE-HELP] Requirement 6. Cross-references: `line-commands` (line command definitions).

#### Acceptance Criteria

8.1. THE help content SHALL include one Help_Topic for each line command family, with Topic_Key `"line:<CMD>"` (e.g., `"line:D"`, `"line:CC"`, `"line:MM"`). [FFE-HELP]

8.2. EACH line command help topic SHALL contain:
  - **Syntax** — the command letter(s), optional count suffix, and block form
  - **Description** — what the command does
  - **Block Form** — how the paired block markers work (e.g., `DD...DD`)
  - **Examples** — at least one concrete usage example
  - **Target Requirements** — whether the command needs an A/B target (for C, CC, M, MM)
  - **See Also** — related line commands and associated primary commands
[FFE-HELP]

8.3. THE help content SHALL include topics for all line commands defined in the `line-commands` spec: D, DD, I, R, RR, C, CC, M, MM, A, B, X, XX, T, TT, U, UU, >, >>, <, <<, ), )), (, ((, COLS, BNDS, TABS, MASK. [FFE-HELP]

8.4. THE Help_System SHALL provide a summary topic with Topic_Key `"line:index"` (accessible via `HELP LINECOMMANDS`) listing all line commands in a compact reference table. [FFE-HELP]

---

### Requirement 9: Help Content — Macro API

**User Story:** As a macro developer, I want comprehensive help for the Lua macro API available through the help system, so that I can write macros using the correct function names, parameters, and return values without consulting external documentation.

**Source:** [FFE-HELP] Requirement 7.3–7.4 (expanded). Cross-references: `lua-macro-engine` (macro API).

#### Acceptance Criteria

9.1. THE help content SHALL include a macro API overview topic with Topic_Key `"feature:macros"` accessible via `HELP MACRO` or `HELP API`. [FFE-HELP]

9.2. THE macro API help topic SHALL contain:
  - An overview of the scripting model (Lua runtime, per-buffer state, event hooks)
  - A categorised list of all available API functions with their signatures
  - A description of event hooks (OnChar, OnKey, OnOpen, OnSave, etc.) with usage examples
  - Cross-references to the `lua-macro-engine` feature documentation
[FFE-HELP]

9.3. THE Help_System SHALL provide per-function help topics with Topic_Key `"api:<function_name>"` for each macro API function, containing: function signature, parameter descriptions, return value, and at least one usage example. [WB]

9.4. WHEN the Lua macro engine registers API functions, THE Help_System SHALL accept help text for each function and make it accessible via the `"api:<function_name>"` Topic_Key pattern. [WB]

---

### Requirement 10: Help Content — Configuration Keys

**User Story:** As a workbench user, I want help topics for configuration keys available through the help system, so that I can understand what each setting does, what values are valid, and where to configure it.

**Source:** [WB] (new for workbench — extends FFE-HELP Requirement 13). Cross-references: `configuration-system` (TOML config, key registry).

#### Acceptance Criteria

10.1. THE help content SHALL include a configuration overview topic with Topic_Key `"feature:configuration"` listing all configuration categories and where settings are stored. [WB]

10.2. THE help content SHALL include per-key help topics with Topic_Key `"config:<key_path>"` (e.g., `"config:help_panel_position"`, `"config:theme"`) for each documented configuration key. [WB]

10.3. EACH configuration key help topic SHALL contain: the key name, the TOML section it belongs to, valid values or value range, the default value, and a description of the key's effect. [WB]

10.4. WHEN `HELP CONFIG` or `HELP CONFIGURATION` is entered as a primary command, THE Help_System SHALL display the configuration overview topic. [WB]

---

### Requirement 11: Help Content — Modes and Features

**User Story:** As a workbench user, I want help topics for each editor mode and major feature, so that I can understand how to use Hex display, the macro system, docking, and other complex capabilities.

**Source:** [FFE-HELP] Requirement 7.

#### Acceptance Criteria

11.1. THE help content SHALL include mode-specific topics with Topic_Keys: `"mode:browse"`, `"mode:edit"`, `"mode:view"`, `"mode:hex"`, `"mode:preview"`, `"mode:grid_browse"`, `"mode:grid_edit"`. [FFE-HELP]

11.2. EACH mode topic SHALL explain: how to enter the mode, what capabilities are available, what is restricted, and how to exit the mode. [FFE-HELP]

11.3. THE help content SHALL include feature topics with Topic_Keys including but not limited to: `"feature:undo"`, `"feature:transactions"`, `"feature:macros"`, `"feature:function_keys"`, `"feature:command_history"`, `"feature:file_tree"`, `"feature:syntax_highlighting"`, `"feature:selection_criteria"`, `"feature:structure_catalog"`, `"feature:sequence_numbers"`, `"feature:clipboard"`, `"feature:tabs"`, `"feature:docking"`, `"feature:vfs"`, `"feature:plugins"`, `"feature:workflows"`. [FFE-HELP, WB]

11.4. EACH feature topic SHALL provide: a concise overview, the commands and UI elements involved, configuration options, and cross-references to related command topics. [FFE-HELP]

11.5. THE help content SHALL include a `"getting_started"` topic that provides a walkthrough for first-time users covering: opening a file, basic navigation, editing, saving, using the command line, and workbench panel arrangement. [FFE-HELP]

---

### Requirement 12: Help Index

**User Story:** As a workbench user, I want a top-level help index that categorises all available help topics, so that I can browse the full help system and discover features I may not know about.

**Source:** [FFE-HELP] Requirement 8.

#### Acceptance Criteria

12.1. THE Help_Index (Topic_Key `"index"`) SHALL be the default topic displayed when F1 is pressed with no specific context or when `HELP` is issued with no arguments. [FFE-HELP]

12.2. THE Help_Index SHALL organise topics into the following categories:
  - **Getting Started** — introduction and tutorial for new users
  - **Primary Commands** — alphabetical listing of all primary commands with one-line descriptions
  - **Line Commands** — compact reference table of all prefix-area commands
  - **Modes** — list of editor modes with brief descriptions
  - **Features** — list of major features (undo, macros, file tree, docking, VFS, plugins, etc.)
  - **Configuration** — summary of configurable settings and where they are stored
  - **Function Keys** — current key map display (generated dynamically from the active Key_Map)
  - **Macro API** — entry point to the scripting API reference
[FFE-HELP, WB]

12.3. EACH category entry in the Help_Index SHALL be a clickable link that navigates to the corresponding topic. [FFE-HELP]

12.4. THE Help_Index SHALL display the workbench application name and version at the bottom. [FFE-HELP]

---

### Requirement 13: HELP Primary Command Integration

**User Story:** As a keyboard-driven user, I want the `HELP` primary command to open the Help Panel with the requested topic, so that I can access help from the command line as well as from F1.

**Source:** [FFE-HELP] Requirement 9. Cross-references: `command-semantics` Requirement 7 (HELP command dispatch).

#### Acceptance Criteria

13.1. WHEN `HELP` is issued with no arguments, THE Help_System SHALL open the Help_Panel displaying the Help_Index. [FFE-HELP]

13.2. WHEN `HELP <command_name>` is issued (e.g., `HELP CHANGE`), THE Help_System SHALL open the Help_Panel displaying the topic for that command (Topic_Key `"cmd:<COMMAND_NAME>"`). [FFE-HELP]

13.3. WHEN `HELP LINECOMMANDS` is issued, THE Help_System SHALL open the Help_Panel displaying the line command summary topic (Topic_Key `"line:index"`). [FFE-HELP]

13.4. WHEN `HELP MACRO` or `HELP API` is issued, THE Help_System SHALL open the Help_Panel displaying the macro API reference topic (Topic_Key `"feature:macros"`). [FFE-HELP]

13.5. WHEN `HELP KEYS` is issued, THE Help_System SHALL open the Help_Panel displaying the current function key assignments (dynamically generated from the active Key_Map). [FFE-HELP]

13.6. WHEN `HELP CONFIG` or `HELP CONFIGURATION` is issued, THE Help_System SHALL open the Help_Panel displaying the configuration overview topic (Topic_Key `"feature:configuration"`). [WB]

13.7. WHEN `HELP` is issued with an unrecognised topic name, THE Help_System SHALL display the Help_Index with a message: "No help available for: <topic>. Available topics are listed below." [FFE-HELP]

13.8. WHEN `HELP OFF` is issued, THE Help_System SHALL close the Help_Panel if it is currently open. [FFE-HELP]

13.9. THE `HELP` command SHALL be valid in Browse mode, Edit mode, View mode, and all special modes. [FFE-HELP]

13.10. THE `HELP` command SHALL NOT be added to command history and SHALL NOT be recorded as an undoable transaction. [FFE-HELP]

---

### Requirement 14: Help Menu Integration

**User Story:** As a desktop user, I want the Help menu to provide access to help topics, the about dialog, and key bindings reference, so that I can discover help features through the standard menu system.

**Source:** [FFE-HELP] Requirement 10. Cross-references: `menu-and-statusbar` (menu bar layout).

#### Acceptance Criteria

14.1. THE Help_Menu SHALL contain the following items: Help Index, Command Reference, Line Command Reference, Key Bindings, separator, About FileForgeWorkbench. [FFE-HELP]

14.2. WHEN "Help Index" is selected, THE Workbench SHALL open the Help_Panel displaying the Help_Index (equivalent to pressing F1 with no context or issuing `HELP`). [FFE-HELP]

14.3. WHEN "Command Reference" is selected, THE Workbench SHALL open the Help_Panel displaying the primary commands category of the Help_Index. [FFE-HELP]

14.4. WHEN "Line Command Reference" is selected, THE Workbench SHALL open the Help_Panel displaying the line command summary (equivalent to `HELP LINECOMMANDS`). [FFE-HELP]

14.5. WHEN "Key Bindings" is selected, THE Workbench SHALL open the Help_Panel displaying the current function key and keyboard shortcut reference (equivalent to `HELP KEYS`). [FFE-HELP]

14.6. WHEN "About FileForgeWorkbench" is selected, THE Workbench SHALL display a modal dialog showing the application name, version, build date, Rust compiler version, and license information. [FFE-HELP]

---

### Requirement 15: Dynamic Help Content — Function Keys

**User Story:** As a workbench user, I want the help system to show my current function key assignments dynamically, so that the help always reflects my actual configuration rather than showing a generic default.

**Source:** [FFE-HELP] Requirement 11. Cross-references: `function-keys-and-history` (Key_Map), `command-framework` (Shortcut_Registry).

#### Acceptance Criteria

15.1. THE Help_System SHALL generate the function keys help topic (Topic_Key `"feature:function_keys"`) dynamically at display time from the active Shortcut_Registry and Key_Map (Global_Key_Map or Profile_Key_Map). [FFE-HELP]

15.2. THE dynamically generated topic SHALL display a table with columns: Key, Command, and Label — listing all assigned function keys F1–F24 and common keyboard shortcuts. [FFE-HELP]

15.3. WHEN a language profile is active and provides a Profile_Key_Map, THE generated topic SHALL show the profile key map and note which profile is active. [FFE-HELP]

15.4. WHEN no key map is configured (all keys unassigned), THE generated topic SHALL display a message explaining how to configure key maps in the workbench configuration and language profiles. [FFE-HELP]

---

### Requirement 16: Help System Configuration

**User Story:** As a workbench user, I want to configure help system behaviour in the workbench TOML configuration, so that I can customise the help panel placement, content location, and rendering preferences.

**Source:** [FFE-HELP] Requirement 13. Cross-references: `configuration-system` (TOML config, hot-reload).

#### Acceptance Criteria

16.1. THE workbench configuration SHALL accept a `[help]` section containing the following keys:
  - `directory` (string) — custom path to the help content directory. WHEN absent, the default search locations SHALL be used. [FFE-HELP]
  - `panel_width_ratio` (float, range 0.2–0.5, default 0.35) — Help_Panel width as a fraction of the window width when docked to a side zone. [FFE-HELP]
  - `panel_position` (string, values `"right"` | `"left"` | `"bottom"`, default `"right"`) — default dock zone for the Help_Panel. [FFE-HELP, adapted]
  - `search_highlight` (boolean, default true) — whether to highlight search matches in help content. [WB]

16.2. WHEN configuration keys in the `[help]` section contain invalid values, THE Help_System SHALL emit a configuration warning via the logging subsystem and apply the default value. [FFE-HELP]

16.3. THE Help_System SHALL respond to hot-reload events from the configuration system: WHEN help-related configuration keys are changed at runtime, THE Help_System SHALL apply the new values without requiring a workbench restart. [WB]

---

## Cross-Reference Summary

| Dependency | Relationship |
|-----------|-------------|
| `command-framework` | Help_System reads `CommandMetadata.help_text` / `help_syntax` from the Command_Registry; F1 is a reserved shortcut (Req 5.3) |
| `command-semantics` | HELP primary command (Req 7) routes into this help infrastructure |
| `layout-and-docking` | Help_Panel implements `DockablePanel` trait; participates in dock zones, tab groups, floating |
| `plugin-architecture` | Plugins register/deregister help topics during lifecycle phases |
| `configuration-system` | Help configuration lives in `[help]` TOML section; hot-reload supported |
| `function-keys-and-history` | Dynamic help content reads Key_Map for function key display |
| `line-commands` | Context detection for prefix-area line commands; help topics for each line command |
| `lua-macro-engine` | Macro API function help registration; `"feature:macros"` topic |
| `menu-and-statusbar` | Help_Menu is part of the workbench menu bar |
