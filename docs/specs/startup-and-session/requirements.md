# Requirements Document

## Introduction

This spec defines the **application startup sequence**, **configuration loading orchestration**, **session persistence and restoration**, **command-line argument handling**, **exit sequence**, **crash recovery**, and **graceful degradation** for FileForgeWorkbench (`ff-session` crate). It formalises the ordered flow from process launch through first UI frame, the persistent session state model across restarts, and the fault-tolerance guarantees that ensure no single corrupt or missing file prevents the workbench from starting.

The startup-and-session subsystem bridges platform-core initialisation, plugin loading, configuration resolution, layout restoration, and the multi-tab editor workspace. It orchestrates these subsystems into a deterministic sequence that yields a usable workbench as quickly as possible while loading secondary state in the background.

### Design Principles

1. **No single failure prevents startup.** A missing or corrupt session file, history store, catalog, plugin, or layout file degrades functionality but never blocks the workbench from reaching an interactive state. [FFE-STARTUP]
2. **CLI arguments override session restore.** Explicit user intent (command-line paths) always takes precedence over stored session state. [FFE-STARTUP]
3. **Session state is a first-class data model.** It is persisted, versioned, and validated — not a bag of ad-hoc JSON. [WB]
4. **Exit is safe by default.** The workbench never discards unsaved work without explicit user confirmation. [FFE-STARTUP]
5. **Crash recovery leverages undo-redo transactions.** Recovery files from the `undo-redo-transactions` subsystem provide the data for unsaved-change restoration after unexpected termination. [FFE-STARTUP, WB]
6. **GUI-independent orchestration.** The startup sequence logic lives in `ff-session` (platform-core layer); the GUI shell is notified when it may render but does not own the sequence. [WB]

### Source References

- **[FFE-STARTUP]** = FileForgeEditor `startup-and-session` specification (10 requirements — priority source)
- **[WB]** = Workbench Architecture Brief (GUI independence, plugin lifecycle, async I/O, layout-as-data)
- **[SCI]** = SciTE session management concepts (MRU, session files, window position persistence)

### Cross-References

- **`configuration-system`** — Owns configuration loading and hot-reload; this spec orchestrates *when* configuration is loaded within the startup sequence and defines session-specific configuration keys.
- **`plugin-architecture`** — Plugin lifecycle (initialize → activate) is a startup sequence step; this spec defines where plugin loading fits and how plugin failures are handled gracefully.
- **`layout-and-docking`** — Layout_State serialisation/deserialisation is loaded during session restore; this spec defines when that occurs and graceful fallback to default layout.
- **`file-operations`** — File open pipeline is invoked during session restore and CLI-driven open; this spec defines the trigger points.
- **`undo-redo-transactions`** — Recovery_Files provide crash recovery data; this spec defines how recovery is offered to the user on startup.
- **`multi-tab-editor`** — Tab_Collection state (open files, tab order, per-tab state) is part of the persisted session; this spec defines the serialisation contract.

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **Startup_Sequence** | The ordered set of operations the workbench performs from process launch to first interactive UI frame. | [FFE-STARTUP] |
| **Session_State** | The complete serialisable snapshot of the user's workspace at a point in time: open files, tab order, viewport positions, panel layout, active persona, window geometry. | [FFE-STARTUP], [WB] |
| **Session_File** | The persistence file (`session.toml`) in the User_Data_Dir that stores the most recent Session_State for restore on next launch. | [FFE-STARTUP], [WB] |
| **Recent_Files_List** | An ordered list of recently opened file URIs with associated metadata (timestamp, last viewport position). Distinct from the full Session_State. | [FFE-STARTUP], [SCI] |
| **Recovery_File** | A per-document periodic snapshot of undo state written by the `undo-redo-transactions` subsystem for crash recovery. | [FFE-STARTUP] |
| **User_Data_Dir** | The platform-specific directory for user-level persistent data: `~/.config/ffworkbench/` on Linux, `~/Library/Application Support/ffworkbench/` on macOS, `%APPDATA%\ffworkbench\` on Windows. | [FFE-STARTUP], [WB] |
| **CLI_Source_Arg** | A file path or VFS URI passed as a positional command-line argument when launching the workbench. | [FFE-STARTUP] |
| **Default_Root** | The process working directory at launch time, used as the File_Tree_Panel root and the base for relative CLI path resolution. | [FFE-STARTUP] |
| **Startup_Phase** | One discrete numbered step within the Startup_Sequence. Each phase has a defined purpose, inputs, outputs, and failure mode. | [WB] |
| **Degraded_Mode** | An operational state where one or more non-essential subsystems failed to initialise; the workbench remains interactive with reduced functionality. | [FFE-STARTUP] |
| **Window_Geometry** | The persisted window position (x, y), size (width, height), maximised state, and display identifier for the Primary_Window and all Floating_Windows. | [SCI], [WB] |
| **Exit_Sequence** | The ordered set of operations performed when the user requests application shutdown: unsaved-change prompts, session save, plugin shutdown, window close. | [FFE-STARTUP] |

---

## Requirements

### Requirement 1: Startup Sequence Ordering

**User Story:** As an operator, I want the workbench to start reliably and quickly with all my preferences, plugins, layout, and session ready, so that I can begin working immediately without manual reconfiguration.

**Source:** FFE Reqs 1, 2, 3, 4 — adapted for workbench plugin and layout lifecycle. [FFE-STARTUP, WB]

#### Acceptance Criteria

1. WHEN the workbench process starts, THE Workbench SHALL execute the Startup_Sequence in the following phase order:
   - Phase 1: Parse command-line arguments
   - Phase 2: Locate and load configuration via the `configuration-system` (layered merge: defaults → system → user → profile → project → workspace)
   - Phase 3: Initialise the logging subsystem using resolved configuration
   - Phase 4: Initialise User_Data_Dir (create if absent, verify subdirectories)
   - Phase 5: Load plugins via the `plugin-architecture` lifecycle (discover → initialize → activate)
   - Phase 6: Load Session_State from Session_File (open tabs, per-tab state, recent files, layout, window geometry)
   - Phase 7: Restore Layout_State via the `layout-and-docking` system (panel positions, tab groups, splitter sizes, persona)
   - Phase 8: Render the first UI frame (workbench becomes interactive)
   - Phase 9: Determine file-open targets (CLI arguments, session restore, or empty state) and open files via the `file-operations` pipeline
   - Phase 10: Check for Recovery_Files and offer crash recovery if applicable
2. PHASES 1 through 7 SHALL complete before Phase 8 renders the first interactive UI frame.
3. PHASES 9 and 10 SHALL execute after the first frame is rendered, so that startup latency is minimised and the workbench appears responsive immediately.
4. WHEN any phase in the Startup_Sequence fails non-fatally, THE Workbench SHALL log the failure at WARN level, record the failure for deferred user notification, and continue to the next phase.
5. THE Workbench SHALL NOT fail to start due to a failure in any single phase except Phase 1 (invalid command-line arguments that cannot be ignored) or a catastrophic platform error (cannot initialise the GUI framework).
6. THE total time from process launch to first interactive UI frame (Phase 8) SHALL be under 2 seconds on reference hardware for a workbench with up to 10 plugins and a session of up to 20 previously open tabs.

---

### Requirement 2: Configuration Loading Orchestration

**User Story:** As an operator, I want the workbench to load my configuration early in the startup sequence so that all subsequent subsystems (logging, plugins, layout) use my preferences from their first operation.

**Source:** FFE Req 2 — adapted to delegate to the `configuration-system` crate. [FFE-STARTUP, WB]

#### Acceptance Criteria

1. THE Workbench SHALL delegate all configuration loading to the `configuration-system` crate during Phase 2 of the Startup_Sequence — the startup-and-session subsystem does NOT parse TOML directly.
2. WHEN the `configuration-system` reports that no configuration file was found in any layer, THE Workbench SHALL proceed with all default values and log an INFO-level record indicating first-run defaults are active.
3. WHEN the `configuration-system` reports configuration warnings (unknown keys, invalid values, parse errors), THE Workbench SHALL collect all warnings and display them together in the status area after Phase 8 (UI ready) — not as modal dialogs.
4. WHEN the `configuration-system` hot-reload detects a change to session-related configuration keys after startup, THE Workbench SHALL apply the new values to subsequent session operations (e.g., changing `max_recent_files` takes effect on the next file open) without requiring restart.
5. THE startup-and-session subsystem SHALL register the following configuration keys with the `configuration-system` schema during its initialisation:
   - `session.user_data_dir` (string, optional override for User_Data_Dir path)
   - `session.max_recent_files` (integer, default 50, range 1–500)
   - `session.restore_on_startup` (boolean, default true)
   - `session.restore_tabs_on_startup` (boolean, default true)
   - `session.startup_file` (string, optional path to auto-open on every launch)
   - `session.save_window_geometry` (boolean, default true)
   - `session.crash_recovery_enabled` (boolean, default true)

---

### Requirement 3: User Data Directory Initialisation

**User Story:** As a first-time user, I want the workbench to create all required directories and default files automatically, so that the workbench works correctly without any manual setup.

**Source:** FFE Req 3 — adapted for workbench directory structure. [FFE-STARTUP]

#### Acceptance Criteria

1. WHEN the User_Data_Dir does not exist at startup (Phase 4), THE Workbench SHALL create it and all required subdirectories: `sessions/`, `recovery/`, `profiles/`, `plugins/`.
2. WHEN the User_Data_Dir exists but a required subdirectory is missing, THE Workbench SHALL create the missing subdirectory without affecting existing content.
3. WHEN the User_Data_Dir cannot be created or is not writable (permission error), THE Workbench SHALL log the error at ERROR level, display a deferred warning after Phase 8, and operate in Degraded_Mode where session persistence and recovery are disabled for the current run.
4. THE User_Data_Dir path SHALL be configurable via the `session.user_data_dir` key in the configuration system. WHEN absent, the platform default SHALL be used.
5. WHEN operating in Degraded_Mode due to User_Data_Dir failure, THE Workbench SHALL still be fully usable for file viewing and editing — only session-level persistence is affected.

---

### Requirement 4: Session State Persistence

**User Story:** As an operator, I want the workbench to remember my complete workspace state — open files, viewport positions, panel layout, window size — so that I can resume exactly where I left off after closing and reopening the workbench.

**Source:** FFE Reqs 5, 6 — enhanced with workbench layout and multi-window state. [FFE-STARTUP, WB]

#### Acceptance Criteria

1. THE Session_State SHALL record the following data for persistence in the Session_File:
   - The ordered list of open file URIs (the Tab_Collection from `multi-tab-editor`) with their tab order
   - Per-tab state for each open file: viewport position (top line, horizontal scroll offset), caret position, selection ranges, active language override (if any), document-specific settings
   - The active tab identifier (which tab was focused at save time)
   - The Layout_State (panel positions, tab group arrangement, splitter sizes, persona name) as defined by `layout-and-docking`
   - Window_Geometry for the Primary_Window and all Floating_Windows
   - The Recent_Files_List with timestamps and per-file metadata
   - The active configuration profile name (from `configuration-system`)
2. THE Session_File SHALL be stored as `session.toml` in the User_Data_Dir, using TOML format consistent with the configuration system.
3. THE Workbench SHALL persist Session_State to the Session_File at the following times:
   - During the Exit_Sequence (Requirement 9)
   - Periodically during operation at a configurable interval (default: every 5 minutes) to guard against crash data loss
   - When the user explicitly triggers a "Save Session" command
4. THE Recent_Files_List SHALL retain entries for the last N files where N is defined by the `session.max_recent_files` configuration key (default 50).
5. WHEN a file in the Recent_Files_List no longer exists on disk (checked at session load time), its entry SHALL be retained but marked as unavailable — it SHALL NOT be removed automatically.
6. THE Session_File format SHALL include a schema version number. WHEN the workbench loads a Session_File with an older schema version, THE Workbench SHALL migrate the data to the current schema, preserving all compatible state.
7. WHEN the Session_File is absent (first run or manually deleted), THE Workbench SHALL start with an empty session without error.
8. WHEN the Session_File is corrupt or unparseable, THE Workbench SHALL log a WARN-level record, discard the corrupt session, and start with an empty session — the workbench SHALL NOT fail to start.

---

### Requirement 5: Session Restore on Launch

**User Story:** As an operator, I want the workbench to restore my previous workspace automatically or offer me the choice, so that I can resume work quickly without manually reopening files and rearranging panels.

**Source:** FFE Req 7 — enhanced with multi-tab and layout restore. [FFE-STARTUP, WB]

#### Acceptance Criteria

1. WHEN `session.restore_on_startup` is `true`, no CLI_Source_Args are provided, and a valid Session_State exists, THE Workbench SHALL restore the full previous workspace: all previously open tabs, their per-tab state, the Layout_State, and Window_Geometry.
2. WHEN `session.restore_tabs_on_startup` is `true` (within a restore), THE Workbench SHALL reopen all previously open files in their recorded tab order, restoring per-tab viewport position, caret position, and selection for each.
3. WHEN `session.restore_tabs_on_startup` is `false`, THE Workbench SHALL restore the Layout_State and Window_Geometry but NOT reopen previously open files — the workbench opens in the empty state with the restored layout.
4. WHEN restoring tabs and a previously open file no longer exists on disk or cannot be resolved through the VFS, THE Workbench SHALL skip that tab, log a WARN-level record, and display a deferred notification in the status area: "Could not restore: <uri>". Remaining tabs SHALL still be restored.
5. WHEN restoring Layout_State and a referenced panel type is not available (plugin not loaded), THE Workbench SHALL substitute a placeholder or use the default layout for that dock zone, and log a WARN-level record.
6. WHEN a CLI_Source_Arg is provided, THE Workbench SHALL NOT perform session tab restore — the CLI argument takes precedence. Layout_State and Window_Geometry SHALL still be restored.
7. WHEN `session.restore_on_startup` is `false`, THE Workbench SHALL skip session restore entirely and open in the empty startup state. Layout_State and Window_Geometry SHALL still be restored from the Session_File if `session.save_window_geometry` is `true`.
8. THE session restore process for file opening SHALL be performed asynchronously after Phase 8 (first frame rendered), so that the workbench appears interactive while files are loading in the background.
9. WHILE session restore is loading files, THE Workbench SHALL display a progress indicator in the status area showing how many tabs have been restored out of the total.

---

### Requirement 6: Command-Line Argument Handling

**User Story:** As a developer or operator, I want to pass file paths, VFS URIs, and options as command-line arguments when launching the workbench, so that I can open specific files or configure behaviour directly from a terminal or shell script.

**Source:** FFE Req 6 — enhanced with VFS URIs and workbench-specific flags. [FFE-STARTUP, WB]

#### Acceptance Criteria

1. THE Workbench SHALL accept zero or more positional command-line arguments, each specifying a file path or VFS URI to open on startup.
2. WHEN a CLI_Source_Arg is a relative file path, THE Workbench SHALL resolve it against the Default_Root (process working directory).
3. WHEN a CLI_Source_Arg is a VFS URI (e.g., `vfs://local/path/to/file`), THE Workbench SHALL pass it directly to the VFS layer for resolution without filesystem-based path manipulation.
4. WHEN one or more CLI_Source_Args are provided and all resolve to existing resources, THE Workbench SHALL open each in a separate tab after Phase 8, with the last argument's tab set as the Active_Tab.
5. WHEN a CLI_Source_Arg resolves to a resource that does not exist, THE Workbench SHALL display a deferred error in the status area ("Resource not found: <path/uri>") and skip that argument. Other valid arguments SHALL still be opened.
6. THE Workbench SHALL accept the following named command-line flags:
   - `--new-window` — Force a new workbench instance even if one is already running (no single-instance enforcement in initial release)
   - `--no-session-restore` — Suppress session restore for this invocation regardless of configuration
   - `--profile <name>` — Activate the specified configuration profile for this session
   - `--project <path>` — Set the project root directory, enabling project-layer configuration
   - `--log-level <level>` — Override the configured log level for this invocation
7. WHEN `--new-window` is specified, THE Workbench SHALL start a fresh instance without attempting to communicate with any existing instance.
8. WHEN both a `session.startup_file` configuration key is set and CLI_Source_Args are provided, THE CLI_Source_Args SHALL take precedence — `session.startup_file` is only used when no explicit file arguments are given.
9. WHEN no CLI_Source_Args are provided and `session.startup_file` is set, THE Workbench SHALL open the configured startup file, overriding session restore for file opening (but Layout_State and Window_Geometry are still restored).

---

### Requirement 7: Empty Startup State

**User Story:** As a first-time user or an operator who has declined session restore, I want the workbench to open in a clean, usable state so that I can immediately begin navigating to a file or starting work.

**Source:** FFE Req 8 — adapted for workbench multi-panel layout. [FFE-STARTUP, WB]

#### Acceptance Criteria

1. WHEN the workbench opens in the empty startup state (no file open, no session restore), THE Workbench SHALL display the default Layout_State: File_Tree_Panel in the left dock zone, the editor area (center) with a welcome tab, the command line, the status bar, and the key label bar.
2. THE welcome tab in the editor area SHALL display a welcome message including the workbench version, a list of recent files (from the Recent_Files_List if available), and quick-action links (Open File, Open Folder, New File).
3. THE command line SHALL be focused and ready to accept input.
4. THE status bar SHALL show the workbench version and "No file open".
5. THE File_Tree_Panel SHALL show the Default_Root (process working directory) as its initial root.
6. THE user SHALL be able to open a file by: typing a command in the command line (e.g., `OPEN path`), double-clicking in the File_Tree_Panel, using the platform native open dialog (Ctrl+O), clicking a recent file in the welcome tab, or via drag-and-drop onto the workbench window.

---

### Requirement 8: Window Geometry Persistence

**User Story:** As an operator, I want the workbench to remember its window position and size across restarts, so that it always appears where I left it — especially important in multi-monitor setups.

**Source:** NEW — derived from SciTE window position persistence + workbench multi-window model. [SCI, WB]

#### Acceptance Criteria

1. WHEN `session.save_window_geometry` is `true`, THE Workbench SHALL persist the Window_Geometry of the Primary_Window as part of the Session_State: position (x, y), size (width, height), maximised/fullscreen state, and display identifier.
2. WHEN `session.save_window_geometry` is `true` and Floating_Windows exist, THE Workbench SHALL persist the Window_Geometry of each Floating_Window, keyed by its panel/tab content identifier.
3. WHEN restoring Window_Geometry and the target display is still connected, THE Workbench SHALL position the window at the recorded coordinates and size.
4. WHEN restoring Window_Geometry and the target display is no longer connected (e.g., laptop undocked from monitor), THE Workbench SHALL reposition the window to the primary display, centred, at the recorded size (clamped to fit the available display).
5. WHEN restoring Window_Geometry and the recorded position would place the window partially or fully off-screen on the target display (display resolution changed), THE Workbench SHALL clamp the window position and size to ensure it is fully visible.
6. WHEN `session.save_window_geometry` is `false`, THE Workbench SHALL use platform default window placement (typically centred on primary display at a default size).
7. THE Workbench SHALL persist Window_Geometry at the same times as Session_State (exit, periodic, explicit save) and SHALL NOT write geometry on every window move/resize event (to avoid disk thrashing).

---

### Requirement 9: Exit Sequence

**User Story:** As an operator, I want the workbench to safely handle shutdown — prompting for unsaved work, saving my session, and shutting down plugins cleanly — so that I never lose work and the next startup is reliable.

**Source:** FFE Req 10 (multi-tab exit) — enhanced with plugin shutdown and session save. [FFE-STARTUP, WB]

#### Acceptance Criteria

1. WHEN the user initiates application exit (File > Exit, window close button, platform shortcut, or `QUIT` command) and no documents have unsaved modifications, THE Workbench SHALL proceed directly to the shutdown sequence without prompting.
2. WHEN the user initiates application exit and one or more open documents have unsaved modifications, THE Workbench SHALL present a summary dialog listing all modified documents with options: "Save All", "Discard All", "Review Each", and "Cancel".
3. WHEN the user selects "Save All", THE Workbench SHALL save each modified document. IF any save fails, THE Workbench SHALL report the failure and abort the exit for that document (offering retry or discard).
4. WHEN the user selects "Discard All", THE Workbench SHALL discard all unsaved changes and proceed to shutdown.
5. WHEN the user selects "Review Each", THE Workbench SHALL present the unsaved-changes dialog for each modified document in tab order (Save / Discard / Cancel per document).
6. WHEN the user selects "Cancel" at any point during the exit flow, THE Workbench SHALL abort the exit and return to normal operation with all documents intact.
7. AFTER unsaved-change handling is complete (all documents saved or discarded), THE Workbench SHALL execute the shutdown sequence in order:
   - Step 1: Persist current Session_State to Session_File
   - Step 2: Clean up Recovery_Files for all documents that were saved or discarded (no longer needed)
   - Step 3: Notify all plugins of shutdown via the `plugin-architecture` lifecycle (`deactivate` → `shutdown`)
   - Step 4: Flush and close the logging subsystem
   - Step 5: Close all windows and terminate the process
8. IF the shutdown sequence encounters an error in Steps 1–4, THE Workbench SHALL log the error and continue to the next step — shutdown SHALL NOT be blocked by a non-fatal error.
9. THE Exit_Sequence SHALL complete within 5 seconds under normal conditions. IF a plugin's shutdown exceeds 3 seconds, THE Workbench SHALL log a WARN-level timeout record and proceed without waiting further.

---

### Requirement 10: Crash Recovery

**User Story:** As an operator, I want the workbench to detect when it was terminated abnormally and offer to restore my unsaved work from recovery files, so that I do not lose edits due to crashes or power failures.

**Source:** FFE Req 7 criterion 5, plus `undo-redo-transactions` Requirement 6 Recovery_File contract. [FFE-STARTUP, WB]

#### Acceptance Criteria

1. WHEN `session.crash_recovery_enabled` is `true` and the workbench starts, THE Workbench SHALL scan the `recovery/` subdirectory of User_Data_Dir for Recovery_Files that were not cleaned up by a normal Exit_Sequence.
2. WHEN one or more Recovery_Files are found (indicating a previous abnormal termination), THE Workbench SHALL present a non-modal notification offering to restore unsaved work: "The workbench was not shut down cleanly. N file(s) have recoverable unsaved changes. [Restore] [Discard] [Later]".
3. WHEN the user selects "Restore", THE Workbench SHALL open each file with a Recovery_File and apply the recovered undo state, placing the document in a modified state so the user can review and save.
4. WHEN the user selects "Discard", THE Workbench SHALL delete all Recovery_Files and proceed normally.
5. WHEN the user selects "Later" (or ignores the notification), THE Workbench SHALL retain the Recovery_Files and offer recovery again on the next startup.
6. WHEN a Recovery_File references a source file that no longer exists on disk, THE Workbench SHALL display a warning for that specific file and skip its recovery. Other recoverable files SHALL still be offered.
7. WHEN a Recovery_File is corrupt or cannot be applied (schema mismatch, data integrity failure), THE Workbench SHALL log the error, skip that file's recovery, and inform the user: "Recovery data for <file> is corrupt and cannot be restored."
8. THE Recovery_File scan and recovery offer SHALL occur in Phase 10 of the Startup_Sequence (after the UI is interactive), so that crash recovery does not delay workbench startup.

---

### Requirement 11: Graceful Degradation

**User Story:** As an operator, I want the workbench to remain usable even when non-essential startup components fail, so that a corrupt session file, failed plugin, or missing layout never prevents me from editing files.

**Source:** FFE Req 9 — enhanced with plugin and layout failure modes. [FFE-STARTUP, WB]

#### Acceptance Criteria

1. THE Workbench SHALL start successfully and be fully usable for file editing even when ALL of the following fail: Session_File loading, plugin initialisation (one or more plugins), Layout_State restoration, Recent_Files_List loading, Recovery_File scan.
2. WHEN operating in Degraded_Mode (one or more subsystems failed during startup), THE Workbench SHALL display a persistent but dismissable indicator in the status bar (e.g., "⚠ Some components not loaded — click for details").
3. THE Workbench SHALL NOT display modal error dialogs during the Startup_Sequence. ALL startup warnings SHALL be deferred to the status area after Phase 8 and be viewable in a summary notification that the user can dismiss.
4. WHEN a plugin fails to initialise during Phase 5, THE Workbench SHALL log the failure, skip that plugin, and continue loading remaining plugins. THE Workbench SHALL report the plugin failure in the deferred status notification.
5. WHEN Layout_State restoration fails (corrupt layout data, missing panel types), THE Workbench SHALL fall back to the default layout and log a WARN-level record.
6. WHEN the workbench has started in Degraded_Mode and the underlying issue is resolved (e.g., User_Data_Dir becomes writable, a plugin is manually reloaded), THE Workbench SHALL clear the degraded indicator for that subsystem.
7. WHEN a file is opened during or after startup, the full file-open pipeline SHALL execute regardless of Degraded_Mode: VFS resolution, encoding detection, language detection, Recovery_File check, and plugin hooks (for loaded plugins). Degraded_Mode does NOT skip file-processing steps — it only affects session-level persistence and failed subsystems.

---

### Requirement 14: ISPF Primary Option Menu and Tabbed Window Container

**User Story:** As an ISPF-familiar operator, I want the workbench to operate as a container of detachable tabbed windows, opening by default to an ISPF-style Primary Option Menu, so that I can navigate to any feature from a familiar home screen and manage multiple work contexts as independent tabs.

**Source:** [ISPF-POM] — IBM ISPF Primary Option Menu heritage; adapted for FileForgeWorkbench feature set.

#### Acceptance Criteria

1. WHEN the workbench application starts for the first time (no saved session), THE desktop shell SHALL open with a single tab displaying the Primary Option Menu. WHEN a saved session exists, THE desktop shell SHALL restore the session to the exact state it was in when last closed — including all open tabs, their types, and their content. [ISPF-POM]

2. THE Primary Option Menu tab SHALL display a centred title line in the format `FileForge Workbench — Primary Option Menu` followed by the application version, a numbered list of menu options, and a live calendar panel. [ISPF-POM]

3. THE Primary Option Menu SHALL display a numbered list of menu options, each with a short label and a one-line description. The built-in options SHALL be, at minimum:
   - `0 Settings` — FFWB Settings and Client Parameters
   - `1 File Catalogs` — Virtual File Catalogs — Mainframe, POSIX, Native
   - `2 Files` — File Explorer — Browse catalogs and files in a tree view
   - `3 Utilities` — Perform utility functions
   - `4 Compilers` — Interactive language processing
   - `5 Lua Scripts` — Run and manage Lua macros
   - `6 Terminals` — Enter TSO or Workstation commands
   - `7 Databases` — Database tool and query browser
   - `8 Plugins` — Vendor added plugins
   [ISPF-POM]

14.3a Option `1` SHALL be labelled `File Catalogs` with description `Virtual File Catalogs — Mainframe, POSIX, Native`. WHEN selected, it SHALL open the Files_Panel (a unified virtual catalog explorer) rather than the native OS file explorer. [ISPF-POM, WB]

14.3b Option `8` SHALL be labelled `Plugins` with description `Vendor added plugins`. WHEN selected, it SHALL open a Plugins management panel. [ISPF-POM]

4. THE Primary Option Menu SHALL display a live calendar to the right of the option list showing the current month, year, day-of-week header, and highlighted current day. [ISPF-POM]

5. THE calendar panel SHALL also display the current time (HH:MM) and the day-of-year number, updated each frame. [ISPF-POM]

6. WHEN the user types a menu option number (e.g., `1`) into the `Command ===>` field of a Primary Option Menu tab and presses Enter, THE shell SHALL transform that tab's content to the corresponding feature view (e.g., option `1` opens a file browser / editor view within the same tab). [ISPF-POM]

7. THE menu bar SHALL include top-level menus that mirror the Primary Option Menu entries: `Settings`, `File Catalogs`, `Files`, `Utilities`, `Compilers`, `Lua`, `Terminals`, `Databases`, `Plugins`, `Help`. [ISPF-POM]

8. THE workbench tab bar SHALL act as a container for all open tabbed windows. Each tab represents an independent work context (Primary Option Menu, file editor, utility panel, etc.). ALL tabs SHALL be attached by default and MAY be detached into separate floating OS windows. [ISPF-POM, WB]

9. WHEN the user right-clicks on the empty space in the tab bar (not on a tab header), THE shell SHALL display a Tab_Bar_Context_Menu with the following items:
   - `New` — opens a new Primary Option Menu tab
   - `New File` — opens a new untitled file editor tab
   [ISPF-POM]

10. WHEN the user types `START` in any `Command ===>` field and presses Enter, THE shell SHALL open a new Primary Option Menu tab. [ISPF-POM]

11. WHEN the user types `CLOSE` in any `Command ===>` field and presses Enter, THE shell SHALL close the current tab (following unsaved-changes rules). [ISPF-POM]

12. WHEN the user types `EXIT`, `=X`, or presses Ctrl+X in any `Command ===>` field, THE shell SHALL initiate the application exit sequence. [ISPF-POM]

13. THE Primary Option Menu tab title in the tab bar SHALL be displayed as `[POM]` to distinguish it from file tabs. [ISPF-POM]

14. A new Primary Option Menu tab SHALL be openable at any time via the `Settings` menu bar entry, by typing `START` in any command field, or by right-clicking the tab bar empty space and selecting `New`. [ISPF-POM]

15. WHEN the user right-clicks a Tab_Header, THE shell SHALL display a Tab_Context_Menu whose contents are determined by the kind of the right-clicked tab, as defined in criteria 14.15a–14.15c. [ISPF-POM, WB]

15a. The following items SHALL appear in the Tab_Context_Menu for ALL tab kinds:
   - Close
   - Close All BUT This
   - Close All to the Left
   - Close All to the Right
   - Close All Unchanged
   - *(separator)*
   - Clone to Other Tab
   - Move to Other View
   - *(separator)*
   - Pin Tab / Unpin Tab *(toggle based on current pin state)*
   - *(separator)*
   - Exit
   [ISPF-POM, WB]

15b. The following items SHALL appear in the Tab_Context_Menu ONLY when the right-clicked tab is a file editor tab (TabKind::FileEditor):
   - *(separator)*
   - Open Containing Folder in Explorer
   - Open Containing Folder in CMD
   - Open Containing Folder in PowerShell
   - Open Containing Folder in Terminal
   - *(separator)*
   - Rename
   - Copy Name to Clipboard
   - Copy Path to Clipboard
   - *(separator)*
   - Read-Only
   - Clear Read-Only Flag
   - *(separator)*
   - Save
   - Save As
   - Reload
   [ISPF-POM, WB]

15c. The Tab_Context_Menu for a Primary Option Menu tab (TabKind::PrimaryOptionMenu) SHALL contain ONLY the universal items listed in 14.15a. No file-specific items SHALL appear — not even in a disabled state. [ISPF-POM]

16. WHEN `Close` is selected from the Tab_Context_Menu, THE shell SHALL close the right-clicked tab following unsaved-changes confirmation rules. [ISPF-POM]

17. WHEN `Close All BUT This` is selected, THE shell SHALL close all tabs except the right-clicked tab, following unsaved-changes confirmation for each modified tab. [ISPF-POM]

18. WHEN `Close All to the Left` is selected, THE shell SHALL close all tabs to the left of the right-clicked tab, following unsaved-changes confirmation for each modified tab. [ISPF-POM]

19. WHEN `Close All to the Right` is selected, THE shell SHALL close all tabs to the right of the right-clicked tab, following unsaved-changes confirmation for each modified tab. [ISPF-POM]

20. WHEN `Close All Unchanged` is selected, THE shell SHALL close all tabs that have no unsaved modifications, leaving modified tabs open. [ISPF-POM]

21. WHEN `Clone to Other Tab` is selected, THE shell SHALL create a duplicate of the right-clicked tab (same content, same type) as a new tab appended to the tab bar. [ISPF-POM]

22. WHEN `Move to Other View` is selected, THE shell SHALL detach the right-clicked tab into a new floating OS window. [ISPF-POM, WB]

23. WHEN `Open Containing Folder in Explorer` is selected on a file tab, THE shell SHALL open the folder containing the file in the platform file explorer (Windows Explorer on Windows). [ISPF-POM]

24. WHEN `Open Containing Folder in CMD` is selected on a file tab, THE shell SHALL open a CMD window at the folder containing the file. [ISPF-POM]

25. WHEN `Open Containing Folder in PowerShell` is selected on a file tab, THE shell SHALL open a PowerShell window at the folder containing the file. [ISPF-POM]

26. WHEN `Open Containing Folder in Terminal` is selected on a file tab, THE shell SHALL open the platform default terminal at the folder containing the file. [ISPF-POM]

27. WHEN `Rename` is selected on a file tab, THE shell SHALL allow the user to rename the file on disk and update the tab title accordingly. [ISPF-POM]

28. WHEN `Copy Name to Clipboard` is selected, THE shell SHALL copy the file name (without path) to the system clipboard. [ISPF-POM]

29. WHEN `Copy Path to Clipboard` is selected, THE shell SHALL copy the full absolute path of the file to the system clipboard. [ISPF-POM]

30. WHEN `Read-Only` is selected on a writable file tab, THE shell SHALL set the tab to read-only mode, preventing edits and updating the tab header with a read-only indicator. [ISPF-POM]

31. WHEN `Clear Read-Only Flag` is selected on a read-only file tab, THE shell SHALL restore the tab to editable mode. [ISPF-POM]

32. WHEN `Pin Tab` is selected, THE shell SHALL pin the tab (immune to bulk-close operations) and display a pin indicator on the tab header. [ISPF-POM]

33. WHEN `Unpin Tab` is selected on a pinned tab, THE shell SHALL remove the pin flag. [ISPF-POM]

34. WHEN `Save` is selected on a modified file tab, THE shell SHALL save the file to disk. [ISPF-POM]

35. WHEN `Save As` is selected on a file tab, THE shell SHALL prompt the user for a new file path and save the content there, updating the tab title. [ISPF-POM]

36. WHEN `Reload` is selected on a file tab, THE shell SHALL reload the file content from disk, discarding any unsaved changes after confirmation if the tab is modified. [ISPF-POM]

37. File-specific Tab_Context_Menu items (those listed in 14.15b) SHALL be OMITTED ENTIRELY from the menu when the right-clicked tab is not a file editor tab — they SHALL NOT appear in a disabled or greyed-out state. The menu for a non-file tab SHALL contain only the universal items from 14.15a (and any kind-specific items from 14.15c). [ISPF-POM]

38. WHEN the user selects "Exit" from the Tab_Context_Menu (any tab kind), THE shell SHALL initiate the application exit sequence, closing the entire application. [ISPF-POM]

39. WHEN the Primary Option Menu is displayed, each numbered option entry (0–8) SHALL be rendered as an interactive button/hyperlink that the user can activate by mouse click or by tabbing to it and pressing Enter. WHEN an option button is activated, THE shell SHALL perform the same navigation action as typing that option number into the `Command ===>` field and pressing Enter. [ISPF-POM]

40. WHEN the Primary Option Menu is displayed, the exit line SHALL be rendered as the text `Enter X to Terminate using log/list defaults` as an interactive button/hyperlink. WHEN it is activated by mouse click or by tabbing to it and pressing Enter, THE shell SHALL initiate the application exit sequence. [ISPF-POM]
   *(Changed from "Enter X to close application" — updated to ISPF-authentic wording.)*

41. THE calendar panel header SHALL be rendered as `<   MonthName  YYYY   >` where `<` and `>` are interactive hotspot buttons flanking the centred month-and-year text. [ISPF-POM]

42. WHEN the user clicks the `<` hotspot or tabs to it and presses Enter, THE calendar SHALL navigate to the previous month. WHEN the user clicks the `>` hotspot or tabs to it and presses Enter, THE calendar SHALL navigate to the next month. The calendar grid, day-of-year, and time display SHALL update to reflect the selected month. The current day highlight SHALL only appear when the displayed month matches the current real month and year. [ISPF-POM]

---

### Requirement 13: Desktop Shell Editor Interactions

**User Story:** As an editor user, I want mouse clicks to move the cursor, Ctrl+Z to undo my last edit, and the cursor position to be clearly visible on screen, so that the editor behaves like a standard interactive text editor.

**Source:** [FFE-MVP-2], [FFE-MVP-3], [FFE-MVP-8] — desktop shell wiring of logical model behaviours into the egui render loop.

#### Acceptance Criteria

1. WHEN the user clicks the mouse within the editor text area, THE desktop shell SHALL compute the document line and column corresponding to the click position (using line height and character width metrics) and move the cursor to that position. [FFE-MVP-8]

2. WHEN the user presses Ctrl+Z, THE desktop shell SHALL undo the most recent edit operation by restoring the document content and cursor position from the most recent entry in the tab's undo stack, and SHALL clear the modified flag if the document returns to its saved state. [FFE-MVP-3]

3. WHEN the cursor is positioned on a line, THE desktop shell SHALL render a visible highlight (frame or background) on the current cursor line so that the user can clearly identify their editing position within the document. [FFE-MVP-2]

4. WHEN the cursor is positioned at a column, THE desktop shell SHALL render a visible caret (vertical bar or block) at that column position within the highlighted line. [FFE-MVP-2]

---

### Requirement 19: File Explorer Panel (POM Option 2)

**User Story:** As an ISPF-familiar operator, I want POM option 2 to open a File Explorer panel that shows all open catalogs as tree nodes with their files listed beneath them, so that I can browse and navigate the file system from a familiar tree interface.

**Source:** [ISPF-POM] option 2 re-definition; [WB] VFS-unified explorer; [FFE-TREE] file tree panel.

#### Acceptance Criteria

1. WHEN the user types `=2` into any `Command ===>` field and presses Enter, THE shell SHALL close the current context (transform the current tab to the File_Explorer_Panel view) and switch the window to the Files context. [ISPF-POM]

2. WHEN the user types `=FILES` (case-insensitive) into any `Command ===>` field and presses Enter, THE shell SHALL close the current context and switch the window to the Files context, identical to `=2`. [ISPF-POM]

3. WHEN the user types `FILES` (case-insensitive, without the `=` prefix) into any `Command ===>` field and presses Enter, THE shell SHALL open a NEW tab in the Files context without closing the current tab. [ISPF-POM]

4. WHEN the user selects option `2` from the Primary Option Menu (by clicking the option button or typing `2` in the command field of a POM tab), THE shell SHALL transform the current POM tab into a File_Explorer_Panel tab with title `[FILES]`. [ISPF-POM]

5. THE File_Explorer_Panel SHALL display a tree view where each open/mounted catalog appears as a top-level expandable node, labelled with the catalog name. [WB, FFE-TREE]

6. WHEN a catalog node is expanded, THE File_Explorer_Panel SHALL list the files and datasets belonging to that catalog as child nodes in the tree, using the same node types and icons as the `file-tree-panel` specification (sequential datasets, PDS members, directories, files). [FFE-TREE]

7. THE File_Explorer_Panel tree SHALL include a node for each catalog type registered in the Catalog_Registry: Mainframe catalogs, POSIX catalogs, and Native catalogs, each grouped under their respective section headers. [WB]

8. WHEN no catalogs are mounted, THE File_Explorer_Panel SHALL display a placeholder message "No catalogs open — use File Catalogs (option 1) to create or mount a catalog" in the tree area. [WB]

9. WHEN the user double-clicks a file node or PDS member node in the File_Explorer_Panel tree, THE shell SHALL open that file in a new editor tab. [FFE-TREE]

10. WHEN the user presses F3 or types `END` in the File_Explorer_Panel command field, THE shell SHALL return the tab to the Primary Option Menu view. [ISPF-POM]

11. THE File_Explorer_Panel tab title in the tab bar SHALL be displayed as `[FILES]` to distinguish it from file editor tabs and the POM tab. [ISPF-POM]

12. THE `[FILES]` tab kind SHALL be persisted in the session and restored on next launch as a `FileExplorerPanel` tab kind. [WB]


### Requirement 20: TSO Session Lifecycle Commands (LOGOFF, TIME, STATUS routing)

**User Story:** As a TSO-familiar operator, I want session lifecycle commands including session timestamps in the status bar, a LOGOFF command to terminate the session, a TIME command to display current date and time, and STATUS routing to the job status panel, so that the workbench matches the TSO session experience.

**Source:** EARS integration Phase CA (coverage-classification.md B08)

#### Acceptance Criteria

1. WHEN the workbench session starts, THE status bar SHALL display the session start timestamp in the format `Started: HH:MM` (or `Started: HH:MM:SS` if configured). [TSO-1.2]
2. WHEN the workbench session ends (exit sequence initiated), THE system SHALL record the session end timestamp and display a logoff message in the format `Logoff at HH:MM -- session duration: Xm Ys` in the status area before closing. [TSO-1.3]
3. WHEN the user types `LOGOFF` in any `Command ===>` field and presses Enter, THE system SHALL initiate the application exit sequence, identical to `EXIT` or `=X`. [TSO-1.4]
4. WHEN the user types `TIME` in any `Command ===>` field and presses Enter, THE system SHALL display the current date and time in the status bar or command response area in the format `Date: YYYY-MM-DD  Time: HH:MM:SS  Day: DDD`. [TSO-2.4]
5. WHEN the user types `STATUS` in any `Command ===>` field and presses Enter, THE system SHALL route to the FFW-JES job status panel (equivalent to `=JES` or the SDSF ST panel). [TSO-2.5]
6. WHEN the user types `STATUS jobname` with an optional job name argument, THE system SHALL route to the FFW-JES panel filtered to show only jobs matching `jobname`. [TSO-2.5]
