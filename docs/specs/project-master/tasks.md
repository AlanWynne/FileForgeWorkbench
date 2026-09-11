# Project Master -- Implementation Status Dashboard

## How to use this file

- This file is a **one-row-per-crate dashboard** only. It shows what is genuinely done.
- Detailed tasks for each crate live in `docs/specs/<sub-project>/tasks.md`.
- When starting work on a crate, open its `tasks.md` for the numbered step list.
- A crate is `[x]` only when: all tasks in its `tasks.md` are `[x]` AND `cargo test -p <crate>` passes with real (non-zero) test coverage.

## Status key

| Mark | Meaning |
|------|---------|
| `[x]` | Complete -- real tests pass |
| `[ ]` | Scaffolded -- compiles, but 0 unit tests (TDD not done) |
| `[~]` | Partial -- some tasks done, some remaining |

---

## Full Workspace Audit -- cargo test --workspace (all pass, 0 failures)

### Phase A -- Foundation

- [x] 1.1 `ff-logging` -- logging subsystem

### Phase B -- Platform Architecture

- [x] 2.1 `ff-core` -- platform core
- [x] 2.2 `ff-config` -- configuration system
- [x] 2.3 `ff-command` -- command framework
- [x] 2.4 `ff-plugin` -- plugin architecture (124 unit + 7 integration + 10 PBTs)
- [x] 2.5 `ff-workflow` -- workflow engine (86 unit + 13 PBTs)
- [x] 2.6 `ff-layout` -- layout and docking (116 unit + 20 integration/PBTs)

### Phase C -- Virtual File System

- [x] 3.1 `ff-vfs` -- virtual file system (102 unit + PBTs)
- [x] 3.2 `ff-connector-local-fs` -- local filesystem connector
- [x] 3.3 `ff-connector-extensibility` -- connector extensibility

### Phase D -- Core Editor

- [x] 4.1 `ff-document-model` -- document model
- [x] 4.2 `ff-edit-operations` -- edit operations
- [x] 4.3 `ff-undo-redo` -- undo/redo transactions
- [x] 4.4 `ff-viewport-scrolling` -- viewport and scrolling (21 unit + 8 integration + 14 PBTs)
- [x] 4.5 `ff-display-line-mapping` -- display line mapping

### Phase E -- Command Engine

- [x] 5.1 `ff-command-semantics` -- primary command parser + pipeline
- [x] 5.2 `ff-find-and-replace` -- FIND/RFIND/CHANGE/RCHANGE
- [x] 5.3 `ff-line-commands` -- D/I/R/C/M/A/B/X/T/>/<
- [x] 5.4 `ff-exclude-show-filter` -- EXCLUDE/SHOW/RESET
- [x] 5.5 `ff-navigation-commands` -- LOCATE/SORT/UP/DOWN/BOUNDS/COLS

### Phase F -- UI and Rendering

- [x] 6.1 `ff-menu` -- menu and status bar
- [x] 6.2 `ff-theme` -- theme and appearance
- [x] 6.3 `ff-text-decorations` -- text decorations
- [x] 6.4 `ff-whitespace-guides` -- whitespace and guides (69 unit + 7 integration + 7 PBTs)
- [x] 6.5 `ff-caret-selection` -- caret and selection

### Phase G -- Language and Highlighting

- [x] 7.1 `ff-language-service` -- language service
- [x] 7.2 `ff-syntax-highlighting` -- syntax highlighting
- [x] 7.3 `ff-auto-indent` -- auto-indentation

### Phase H -- File I/O and Session

- [x] 8.1 `ff-file-ops` -- file operations
- [x] 8.2 `ff-background-io` -- background I/O
- [x] 8.3 `ff-encoding` -- encoding and characters
- [x] 8.4 `ff-external-mod` -- external modification detection
- [x] 8.5 `ff-session` -- startup and session
- [x] 8.6 `ff-tabs` -- multi-tab editor

### Phase I -- Desktop Integration

- [x] 9.1 `ff-clipboard` -- clipboard operations
- [x] 9.2 `ff-keys` -- function keys and history
- [x] 9.3 `ff-shell` -- shell command
- [x] 9.4 `ff-help` -- context help
- [x] 9.5 `ff-zoom` -- view zoom (75 unit + 18 integration + 19 PBTs)
- [x] 9.6 `ff-wrap` -- line wrap toggle (122 unit + 23 integration + 27 PBTs)

### Phase J -- Extensions and Macros

- [x] 10.1 `ff-lua` -- Lua macro engine
- [x] 10.2 `ff-completion` -- command completion

### Phase K -- Display Modes

- [x] 11.1 `ff-hex` -- hex display
- [x] 11.2 `ff-seqnum` -- sequence numbers
- [x] 11.3 `ff-tabmask` -- tabs and mask

### Phase L -- FileForge Domain

- [x] 12.1 `ff-forge` -- FileForge integration
- [x] 12.2 `ff-structure-catalog` -- structure catalog
- [x] 12.3 `ff-select` -- record selection criteria
- [x] 12.4 `ff-asa` -- ASA report preview
- [x] 12.5 `ff-viewers` -- custom file viewers (128 unit + 10 PBTs)

### Phase M -- Dataset Catalog and Mainframe Emulation

- [x] 13.0 `ff-governance-tests` -- dataset ownership model governance
- [x] 13.1 `ff-dscatalog` -- dataset catalog
- [x] 13.2 `ff-dsalloc` -- dataset allocator
- [x] 13.3 `ff-idcams` -- IDCAMS emulator

### Phase N -- Job Entry Subsystem

- [x] 14.1 `ff-jes` -- JES emulation

### Phase O -- File Explorer

- [x] 15.1 `ff-file-tree` -- file tree panel
- [x] 15.2 `ff-compare-merge` -- compare and merge

### Phase P -- Performance

- [x] 16.1 `ff-idle-processing` -- idle processing
- [x] 16.2 `ff-large-file-performance` -- large file performance

### Phase Q -- Database Tool

- [x] 17.1 `ff-database-tool` -- database tool

### Phase R -- Binary Integration (`ff-desktop` / `ffwb` binary)

- [x] 18.1 Boot sequence -- logging → config → Tokio → WorkbenchApp → eframe window
- [x] 18.2 egui shell -- WorkbenchShell, theme, menu bar, status bar, ISPF command field
- [x] 18.3 Editor panel -- line rendering, mouse-wheel scroll
- [x] 18.4 File open -- File > Open wired to ff-connector-local-fs
- [x] 18.5 Tab bar -- tab headers, switching restores per-tab viewport state
- [x] 18.6 Command field dispatch -- EDIT/EXIT/QUIT/=X/1/FILES
- [x] 18.7 Live status bar -- cursor line/col, encoding, modified indicator, line count
- [x] 18.8 Keyboard navigation -- arrow keys, Page Up/Down through ViewportModel
- [x] 18.9 CLI file arguments -- open files from command line as tabs
- [x] 18.10 Session save/restore -- persist and restore open tabs on exit/launch

### Phase S -- Binary Polish (`ff-desktop`)

- [x] 19.1 Fix ff-dsalloc property test compile failure
- [x] 19.2 Native file-open dialog (rfd, File > Open…)
- [x] 19.3 Keyboard text input -- typed chars insert, Backspace deletes, Enter splits line
- [x] 19.4 Save to disk -- File > Save and Ctrl+S

### Phase T -- Bug Fixes

- [x] 20.1 Mouse click moves cursor (Req 13.1)
- [x] 20.2 Ctrl+Z undo (Req 13.2)
- [x] 20.3 Current-line highlight (Req 13.3)
- [x] 20.4 Caret bar rendering (Req 13.4)

### Phase V -- ISPF Primary Option Menu

- [x] 22.1–22.8 All POM tasks complete (50 tests pass in ff-desktop)

### Phase U -- ISPF Command Engine Integration (next phase)

- [x] 21.1 Wire `ff-command-semantics` into `ff-desktop` command field (replace hard-coded handle_command)
- [x] 21.2 Wire `ff-line-commands` prefix area into editor panel gutter
- [x] 21.3 Wire `ff-find-and-replace` into command field (FIND/CHANGE commands)
- [x] 21.4 Wire `ff-navigation-commands` into command field (LOCATE/SORT/UP/DOWN)
- [x] 21.5 Wire `ff-exclude-show-filter` into editor panel (EXCLUDE/SHOW/RESET)
- [x] 21.6 Render interactive prefix area (gutter) in editor panel
- [x] 21.7 Wire `ff-keys` -- function key bar, RETRIEVE, command history

### Phase W -- Compiler Toolchain Integration

- [x] W.1 Create `ff-toolchain-api` crate -- shared `ToolchainPlugin` trait, `ToolchainState`, `Diagnostic`, `BuildProfile`, `BuildEvent` (Tasks 1.1–1.9)
- [x] W.2 Create `ff-gcc-toolchain` plugin crate -- GCC detection, platform install (winget/apt/brew), build invocation, diagnostic parser (Tasks 2.1–2.12)
- [x] W.3 Create `ff-rust-toolchain` plugin crate -- rustup/rustc/cargo detection, rustup-init install, cargo build/check/test, JSON diagnostic parser (Tasks 3.1–3.11)
- [x] W.4 Toolchain_Panel UI in `ff-desktop` -- status rows, install buttons, progress, build output, clickable diagnostics, Compilers menu wiring (Tasks 4.1–4.7)
- [x] W.5 Validate generic ToolchainPlugin trait contract -- MockToolchain test double in `ff-toolchain-api`, confirm no GCC/Rust-specific assumptions in trait, confirm no dev-dep on plugin crates (Tasks 5.1-5.3 in compiler-toolchain-integration/tasks.md)

### Phase X -- POM Floating Window (superseded by Phase Z)

- [x] X.1 Converted POM to egui::Window -- superseded; Phase Z converts to tabbed window container

### Phase Z -- Tabbed Window Container + Full Tab Context Menu (B001 re-specified)

- [x] Z.1 Implement TabKind enum, POM as first-class tab, full 27-item tab context menu, tab bar empty-space menu, START/CLOSE/EXIT command routing (Tasks 19.1-19.23 in startup-and-session/tasks.md)

### Phase Z.1 -- Tab Context Menu Exit Item (Req 14.38)

- [x] Z.1.1 Add "Exit" as last universal item in tab header context menu for all tab kinds (Task 20.1-20.5 in startup-and-session/tasks.md)

### Phase AA.11 -- POM Option Buttons (Req 14.39, 14.40)

- [x] AA.11 Make each POM option row and the Exit line interactive buttons (Tasks 21.1-21.7 in startup-and-session/tasks.md)

### Phase AC -- POM Option List Reorganisation (Req 14.3, 14.3a, 14.3b)

- [x] AC.1 Reorganise POM built-in options to 9 entries (0–8): Settings, File Catalogs, Files, Utilities, Compilers, Lua Scripts, Terminals, Databases, Plugins (Tasks 23.1–23.5 in startup-and-session/tasks.md)

### Phase AD -- Menu Bar Alignment with 9-Option POM (Req 14.7)

- [x] AD.1 Add `File Catalogs` and `Plugins` top-level menus to the menu bar to mirror all 9 POM options (Tasks 24.1–24.6 in startup-and-session/tasks.md)

### Phase AE -- POM Exit Line Text Update (Req 14.40)

- [x] AE.1 Update POM exit line text to ISPF-authentic wording "Enter X to Terminate using log/list defaults" (Tasks 25.1–25.4 in startup-and-session/tasks.md)

### Phase AF -- Calendar Month Navigation (Req 14.41, 14.42)

- [x] AF.1 Add `<` / `>` hotspot buttons to calendar header; support forward/backward month navigation with per-POM-tab offset state (Tasks 26.1–26.10 in startup-and-session/tasks.md)

### Phase AH -- Settings Panel (Req 15)

- [x] AH.1 Add `set_user_value` / `remove_user_value` to `ConfigHandle` in `ff-config` (Task 25)
- [x] AH.2 Add `TabKind::SettingsPanel`, route `0` / `SETTINGS` / `=0` commands, wire POM option 0
        button, session persistence (Task 26)
- [x] AH.3 Create `settings_panel.rs` -- namespace groups, collapsible sections, filter input,
        source file footer, F3/END routing (Task 27)
- [x] AH.4 Per-key value widgets (checkbox, slider, combo, text) and provenance badge (Task 28)
- [x] AH.5 Write path (validate → `set_user_value`), inline errors, Reset to Default button (Task 29)

### Phase AG -- Help > About Dialog (Req 13)

- [x] AG.1 Create `about_dialog.rs` in `ff-desktop`; wire `Help > About` menu item; display app name,
        version, creator credit (Alan R Wynne), AI assistant credit (Amazon Q Developer / AWS),
        copyright, and description (Tasks 18.1–18.6 in menu-and-statusbar/tasks.md)

### Phase AE-2 -- Legacy Theme Colour Semantics (Req 13 theme-and-appearance)

- [x] AE.1 Add `primary_menu_bg` to `UiColours`; update `ColourToken`; update Legacy palette defaults
- [x] AE.2 Wire `PomColours` from shell into `primary_option_menu::render()` for Legacy theme
- [x] AE.3 Implement per-element colour rendering in POM (key=white, label=turquoise, desc=green, calendar=turquoise, today=reversed)

### Phase AJ -- Tab-Order Focus Cycle (Req 16 menu-and-statusbar)

- [x] AJ.1 Add `FocusStop` enum with `CommandField`, `PomOption`, `PomExit`, `CalendarPrev`, `CalendarNext`, `MenuBar` variants; wire Tab/Shift+Tab cycle in `WorkbenchShell`
- [x] AJ.2 Pass `focused_pom_option` into POM render; draw reversed-colour highlight on focused option row
- [x] AJ.3 Handle Enter/Space activation on all focus stops; apply menu bar focus indicator

### Phase AI -- User-Configurable Theme Colours and Custom Themes (Req 14 theme-and-appearance)

- [x] AI.1 Implement themes directory scanning and `ThemeInfo` / `list_themes()` API in `ff-theme`
- [x] AI.2 Register directory watch on themes directory for hot-reload of new/modified user theme files
- [x] AI.3 Implement `export_theme(name)` serialisation helper; audit full token coverage; write tests

### Phase AB -- Catalog Storage Default Paths (Req 12)

- [x] AB.1 Register `catalogs.default_mainframe_root` and `catalogs.default_posix_root` config
        schema keys; pre-populate Catalog Manager Dialog path fields from config (Tasks 11.1–11.6
        in virtual-catalog-manager/tasks.md)

### Phase AA -- Virtual Catalog Manager (POM Option 1 -- Files Panel)

- [x] AA.1 Add `FilesPanel` TabKind, route POM option 1 to Files panel, update option 1 label (Task 1.1–1.6 in virtual-catalog-manager/tasks.md)
- [x] AA.2 Catalog Registry -- persist/restore all four catalog types (Task 2.1–2.6)
- [x] AA.3 POSIX VFS Provider -- scheme `posix`, root-jail, read-only enforcement (Task 3.1–3.6)
- [x] AA.4 Files Panel skeleton -- split layout, four section headers, toolbar (Task 4.1–4.7)
- [x] AA.5 Catalog Manager Dialog -- Create (all four types) (Task 5.1–5.10)
- [x] AA.6 Catalog Manager Dialog -- Edit and Delete (Task 6.1–6.6)
- [x] AA.7 Dataset Allocation Dialog -- ISPF-style fields, Allocate Like (Task 7.1–7.7)
- [x] AA.8 Context menus -- Mainframe, POSIX, Windows/Local (Task 8.1–8.7)
- [x] AA.9 Content area -- columns, sort, breadcrumb, filter (Task 9.1–9.7)
- [x] AA.10 Session persistence for FilesPanel tab and catalog registry (Task 10.1–10.7)

---

### Phase AJ -- Tab-Order Focus Cycle (Req 16 menu-and-statusbar)

- [x] AJ.1 Add `FocusStop` enum and `focus_stop` field to `WorkbenchShell`; wire Tab/Shift+Tab cycle through command field → menu bar items → calendar `<`/`>` → wrap (Task 19.1–19.6 in menu-and-statusbar/tasks.md)

### Phase AK -- Tab-Header Focus Stops + Command Field Focus Fix (Req 16 menu-and-statusbar)

- [x] AK.1 Add `TabHeader { index }` variant to `FocusStop`; update `next()`/`prev()` with `tab_count`; wire tab header focus in `update()`
- [x] AK.2 Fix command field focus: request egui focus every frame when `focus_stop == CommandField`
- [x] AK.3 Write 6 unit tests covering new tab-header cycle paths

### Phase AL -- Tab Window Chrome: Title Line (Req 17, 18)

- [x] AL.1 Add `render_title_line()` to `WorkbenchShell`; derive text from active tab kind/path; apply Legacy theme blue/white styling (Tasks 21.1–21.3 in menu-and-statusbar/tasks.md)
- [x] AL.2 Write unit tests for Title_Line text derivation (Task 21.4 in menu-and-statusbar/tasks.md)
- [x] AL.3 Stub "Move to Other View" with status message; add `title_line_text()` helper for future floating window use (Tasks 22.1–22.2 in menu-and-statusbar/tasks.md)

### Phase AO -- Detachable Tab Windows (Req 18.1–18.7)

- [x] AO.1 Wire "Move to Other View" context menu item to set detach_pending with 16-window limit (Task 23.4)
- [x] AO.2 Process detach_pending in update(); skip floating tabs in render_tab_bar(); show_viewport_deferred per tab (Tasks 23.5-23.7)
- [x] AO.3 Detect close event -> redock_pending; process redock with clamped restore (Tasks 23.8-23.9)
- [x] AO.4 5 unit tests: is_floating flag, 16-window limit, origin index, redock clamp, title format (Task 23.10)
- [x] AO.5 Fix pre-existing clippy lints in ff-keys/src/function_key.rs (strip_prefix, question_mark)

### Phase AM -- Per-Context Key Maps, PFSHOW, 24-Key Bar, Hotspots, END/RETURN, LIST+RETRIEVE (Req 12–19 function-keys-and-history)

- [x] AM.1 PFSHOW command -- register `keys.pfshow`, session persistence, shell wiring (Task 15)
- [x] AM.2 Two-row Key Label Bar -- 24-slot model, two-row render in ff-desktop (Task 16)
- [x] AM.3 Per-context key map -- `KeyMapResolver` context support, tab-switch wiring, TOML config (Task 17)
- [x] AM.4 Built-in default 24-key set -- `KeyMap::default_global()`, fallback wiring (Task 18)
- [x] AM.5 Key Label Bar hotspots -- clickable slots, tooltip, dispatch on click (Task 19)
- [x] AM.6 END and RETURN commands -- register, tab-history stack, exit semantics (Task 20)
- [x] AM.7 Contextual help fallback -- "not available yet" status message in ff-help (Task 21)
- [x] AM.8 LIST + RETRIEVE history browser -- `ShowList` variant, modal overlay, selection (Task 22)

### Phase AN -- Key Configuration Dialog (Req 20 function-keys-and-history)

### Phase AN -- Final Status (implementation complete)

- [x] AN.1 `KeyModifier` + `ModifiedKey` in `ff-keys`; `KeyMap` uses `ModifiedKey`; `description` on `KeyBinding`; TOML parser extended for `SF`/`CF`/`AF` (Tasks 23–24)
- [x] AN.2 `KeyMapResolver` call sites updated; modifier dispatch in `ff-desktop` shell (Tasks 25–26)
- [x] AN.3 `key_config_dialog.rs` -- scope tabs, 24-row grid, Save/Cancel/Reset (Tasks 27–29)
- [x] AN.4 `KEYS` command + `Edit > Key Assignments…` menu item wired (Task 28)
- [x] AN.5 Property-based tests for `ModifiedKey` round-trip -- deferred (Task 30)

### Phase AP -- PFSHOW Session Persistence (Req 12.4 function-keys-and-history)

- [x] AP.1 Add `key_bar_visible: bool` to `SessionState` in `ff-session` with `serde(default = "default_true")`
- [x] AP.2 Pass `key_bar_visible` to `session.save()` in `on_exit()`; restore in startup session load
- [x] AP.3 Unit test: `key_bar_visible_round_trips_through_session` in `session_manager.rs`

## Phase AQ -- Key Map TOML Persistence (Req 20.8)

- [x] AQ.1 Add to_config_table() to ScopeRows; add save_to_config() to KeyConfigDialog
- [x] AQ.2 Update render_if_open/render to accept ConfigHandle; wire Save button
- [x] AQ.3 Add 3 unit tests for Req 20.8 serialisation logic

### Phase AR -- [context_key_maps] TOML Config Parsing (Req 14.7 function-keys-and-history)

- [x] AR.1 Add load_context_maps_from_config() in ff-desktop shell.rs; call at startup; 2 unit tests (Task 32)

### Phase AS -- File Explorer Panel (Req 19 startup-and-session)

- [x] AS.1 Add `FileExplorerPanel` TabKind; route `=2`, `=FILES`, `FILES` commands; implement tree view showing open catalogs and their files; session persistence (Tasks 27.1–27.21 in startup-and-session/tasks.md)

### Phase AT -- Allocated Dataset Display (Req 13 virtual-catalog-manager)

- [x] AT.1 Add `AllocatedDataset` store to `FilesPanelState`; wire `AllocOutcome::Confirmed` to insert into store; populate content area from store on catalog select; persist/restore via session TOML (Tasks 12.1–12.14 in virtual-catalog-manager/tasks.md)

### Phase AX -- Default Home Catalog on First Launch (Req 14 virtual-catalog-manager)

- [x] AX.1 Add `ensure_default_home_catalog()` helper; wire into startup block in `update.rs`; deletion guard in `execute_delete()`; 5 unit tests (Tasks 14.1–14.11 in virtual-catalog-manager/tasks.md)

### Phase AU -- Catalog Registry Persistence (B010 fix, Req 2.1, 2.2 virtual-catalog-manager)

- [x] AU.1 Add `save_catalog_registry()` / `load_catalog_registry()` to `SessionManager`; wire into `on_exit()` and startup; 2 unit tests (Tasks 13.1–13.9 in virtual-catalog-manager/tasks.md)

### Phase AY -- File Explorer: Expandable Subdirectories and Scrollable Panel (Req 15 file-tree-panel)

- [x] AY.1 Wrap File Explorer Panel content in `ScrollArea::vertical()`; replace flat directory entries with recursive `CollapsingHeader` nodes in `render_native_children()` (Tasks 18.1–18.3 in file-tree-panel/tasks.md)

### Phase AZ -- File Explorer Context Menu (Req 16 file-tree-panel)

- [x] AZ.1 `NodeKind` + `MenuItem` enums; `build_context_menu()` for all 8 node kinds; egui context_menu wiring; Git/Submit JCL greyed-out (Tasks 19.1–19.3)
- [x] AZ.2 Inline rename with Enter/Escape; Mainframe 8-char uppercase enforcement (Task 19.4)
- [x] AZ.3 Copy to clipboard (full path/DSN); all Copy path variants (Tasks 19.5–19.6)
- [x] AZ.4 Reveal in Explorer -- platform-appropriate OS file manager launch (Task 19.7)
- [x] AZ.5 Copy To… / Move To… dialog with naming-rule transformation and ff-bgio progress (Task 19.8)
- [x] AZ.6 Unit tests for all above (Task 19.9)

### Phase BA -- Open With Default Application (Req 17 file-tree-panel)

- [x] BA.1 `FileClass` enum + `EXTERNAL_EXTENSIONS` table covering Office, PDF, images, audio/video, archives, executables, databases (Task 20.1)
- [x] BA.2 `classify_file()` with extension lookup and magic-byte fallback (Task 20.2)
- [x] BA.3 `launch_default_app()` platform dispatch Windows/macOS/Linux non-blocking (Task 20.3)
- [x] BA.4 `open_file_node()` routing Text→editor, External→OS launch; Mainframe bypass (Task 20.4)
- [x] BA.5 Status-bar error display for failed launches (Task 20.5)
- [x] BA.6 Unit tests for all above (Task 20.6)

### Phase BB -- Native Catalog Sorted Listing and File Attributes (CR-NR-008, B017, B018)

- [x] BB.1 Refactor `render_native_children()` -- sort dirs-first/alpha, silent-skip inaccessible entries (junction points, locked files), build `FileEntryRow` with metadata (Req 18.1, 18.7, B017)
- [x] BB.2 Implement `format_size`, `format_timestamp`, `format_permissions` helpers (Req 18.2–18.6)
- [x] BB.3 Render attribute columns per row: Size, Modified, Created, Accessed, Permissions (Req 18.9)
- [x] BB.4 Catch OS error 32 in `open_file_node()` -- status-bar message, no editor tab (Req 18.8, B018)
- [x] BB.5 Unit tests for all helpers and sort/skip behaviour (Task 21.8)

### Phase BC -- File Explorer content area: directories alphabetically sorted (CR-CH-004)

- [x] BC.1 Fix `visible_entries()` sort in `files_panel.rs` -- containers always before non-containers when sorting by Name; each group sorted case-insensitively; 3 new unit tests (Req 10.7)

### Phase BD -- File Explorer tree: drag-select and copy as text tree (CR-NR-009, Req 19 file-tree-panel)

- [x] BD.1 Add `selected_nodes: HashSet<String>` and `anchor_node: Option<String>` to `FileExplorerPanelState`; wire plain-click, Shift+click, Ctrl+click, and drag-select input handling (Tasks 22.1–22.2)
- [x] BD.2 Render selected nodes with selection background tint (Task 22.3)
- [x] BD.3 Implement `build_text_tree()` pure function -- indented ASCII tree with `[DIR]` prefix and tree connectors for hierarchical selections (Task 22.4)
- [x] BD.4 Wire Ctrl+C and "Copy as Text Tree" context menu item to `build_text_tree` + clipboard write (Tasks 22.5–22.6)
- [x] BD.5 Wire Escape to clear multi-selection (Task 22.7)
- [x] BD.6 Unit tests for `build_text_tree` (Task 22.8)

### Phase BJ -- Catalog Repository Path Display + VFS Dataset Path Resolution (CR-NR-012)

- [x] BJ.1 Show repository path as read-only field in Edit Catalog dialog for all catalog types (Task 15.1–15.3 in virtual-catalog-manager/tasks.md)
- [x] BJ.2 Add `resolve_dataset_path()` pure function; wire into Files Panel and File Explorer open handlers (Tasks 16.1–16.6 in virtual-catalog-manager/tasks.md)

### Phase BI -- Default BLKSIZE=0 in Dataset Allocation Dialog (CR-CH-005)

- [x] BI.1 Change default BLKSIZE to `"0"` in `AllocDatasetForm::default()`; update validate() to accept 0; update tests (Task 7.8 in virtual-catalog-manager/tasks.md)

### Phase BE -- File Explorer keyboard navigation + file copy/paste (CR-NR-010, CR-NR-011, Req 20–21 file-tree-panel)

- [x] BE.1 Add `FocusStop::FileExplorer` variant; wire Tab from CommandField to Explorer_Focus; `cursor_node` field on `FileExplorerPanelState` (Task 23.1)
- [x] BE.2 Implement `collect_visible_node_paths()` pure function (Task 23.2)
- [x] BE.3 Wire Tab, Arrow, Shift+Arrow, Ctrl+Arrow, Ctrl+Space, Escape keyboard handling in Explorer render loop (Tasks 23.3–23.8)
- [x] BE.4 Render cursor focus ring distinct from selection highlight (Task 23.9)
- [x] BE.5 Unit tests for all keyboard navigation behaviours (Task 23.10)
- [x] BE.6 Add `FileCopyClipboard`, `PasteProgress`, `PasteConflict` types; wire Ctrl+C, Ctrl+V (file list), Ctrl+V (editor), conflict modal, POSIX guard, Mainframe transform, pending-paste indicator (Tasks 24.1–24.8)
- [x] BE.7 Unit tests for file copy/paste operations (Task 24.9)

### Phase BL -- B024 Tab Cycle Fix: exit tree + visual cursor highlight

- [x] BL.1 Fix Tab cycle exit: Tab past last tree node exits `explorer_focused`, clears `cursor_node`, returns `focus_stop` to `CommandField` -- previously wrapped infinitely
- [x] BL.2 Add visual cursor highlight on catalog-level nodes (`rect_filled` behind `CollapsingHeader` when `cursor_node == "cat:NAME"`)
- [x] BL.3 Extend file node `is_selected` to include `cursor_node` match so file rows show highlight when tabbed to
- [x] BL.4 Consolidate two separate Tab branches into single unified branch handling enter/advance/exit cases

### Phase BM -- File Explorer Panel: egui-file-dialog look-and-feel with catalog mount points (CR-NR-014, Req 23 file-tree-panel)

- [x] BM.1 Add `selected_catalog` + `sidebar_width` fields to `FileExplorerPanelState`; refactor `render()` into `render_sidebar()` + `render_content_pane()` two-pane layout (Tasks 26.1–26.4)
- [x] BM.2 Implement `render_mainframe_content()` -- dot-qualified dataset listing, PDS expandable, PS leaf, VFS open routing (Task 26.5)
- [x] BM.3 Implement `render_posix_content()` -- forward-slash path display, directory/file tree from `read_dir` (Task 26.6)
- [x] BM.4 Empty sidebar placeholder; sidebar width persistence (Tasks 26.7–26.8)
- [x] BM.5 Unit tests + `cargo test` green (Tasks 26.9–26.10)

### Phase BK -- Native File Browser: egui-file-dialog Integration (CR-NR-013, Req 22 file-tree-panel)

- [x] BK.1 Add `egui-file-dialog = "0.6"` to `crates/ff-desktop/Cargo.toml`; vendored patch resolves egui 0.29 mismatch; `cargo check` passes (Task 25.1)
- [x] BK.2 Add `NativeDialogSlot` newtype (manual `Debug`/`Clone`); `native_dialogs: HashMap<String, NativeDialogSlot>` on `FileExplorerPanelState` (Task 25.2)
- [x] BK.3 Implement `render_native_dialog()` -- lazy init, scoped borrow, `dialog.update(ctx)`, `take_selected()` → `open_file_node()` (Task 25.3)
- [x] BK.4 Replace `render_native_children()` call in Native branch of `render()` with `render_native_dialog()`; dead-code helpers annotated `#[allow(dead_code)]` (Task 25.4)
- [x] BK.5 Confirmed Mainframe/POSIX branches untouched; all 8 Mainframe/POSIX tests pass (Task 25.5)
- [x] BK.6 `THIRD_PARTY_CREDITS.md` created at workspace root with full MIT licence text (Task 25.6)
- [x] BK.7 `cargo test` 486 passing 0 failures; `cargo clippy` clean; `cargo build --release` succeeds (Tasks 25.7–25.9)

### Phase BO -- Bug Fix Sprint: Persistence Gaps (B020, B021, B022)

- [x] BO.1 B020 -- Call `save_catalog_registry()` immediately after `DialogOutcome::Confirmed` in NewCatalog and DeleteCatalog handlers in `update.rs` (not only in `on_exit()`)
- [x] BO.2 B021 -- Pre-populate `repository_path` with `mainframe_root.clone()` in `NewCatalogForm::with_defaults()` so the field is non-empty on dialog open
- [x] BO.3 B022 -- `save_datasets()`/`load_datasets()` added to `SessionManager`; wired into startup restore, `AllocateDataset` confirmed handler, and `on_exit()`
- [x] BO.4 Mark task 26 (Phase BM) `[x]` in `file-tree-panel/tasks.md`; 496 tests passing 0 failures

### Phase BP -- Vendor Warning Elimination (egui-file-dialog future_incompatible + deprecated)

- [x] BP.1 Fix `float_literal_f32_fallback` in `vendor/egui-file-dialog/src/file_dialog.rs` lines 1223 and 1360 -- `1.0` → `1.0_f32` in two `egui::Stroke::new()` calls
- [x] BP.2 Fix deprecated `ComboBox::from_id_source` → `from_id_salt` in `file_dialog.rs` line 1805
- [x] BP.3 Fix `mismatched_lifetime_syntaxes` in `vendor/egui-file-dialog/src/data/directory_content.rs` -- add `'s` to elided return-type lifetimes on `filtered_iter` and `filtered_iter_mut`
- [x] BP.4 `cargo build -p egui-file-dialog` -- 0 warnings; `cargo test -p ff-desktop` -- 496 passed 0 failed

### Phase BQ -- Requirements Review and Modernisation (CR-NR-015)

- [x] BQ.T1 Task 1 -- Inventory & Baseline Audit (`docs/reviews/requirements-review/inventory.md`)
- [x] BQ.T2 Task 2 -- Terminology Standardisation (`docs/reviews/requirements-review/terminology-map.md`)
- [x] BQ.T3 Task 3 -- Architectural Domain Classification (`docs/reviews/requirements-review/domain-classification.md`)
- [x] BQ.T4 Task 4 -- Gap Analysis (`docs/reviews/requirements-review/gap-analysis.md`)
- [x] BQ.T5 Task 5 -- Rewrite: Core Platform & UX Layer Specs (10 specs)
- [x] BQ.T6 Task 6 -- Rewrite: Explorer & Content Layer Specs (15 specs)
- [x] BQ.T7 Task 7 -- Rewrite: Task Layer, Integration Layer & Domain Specs (14 specs)
- [x] BQ.T8 Task 8 -- Traceability Matrix (`docs/reviews/requirements-review/traceability-matrix.md`)
- [x] BQ.T9 Task 9 -- Consolidation Report (`docs/reviews/requirements-review/consolidation-report.md`)
- [x] BQ.T10 Task 10 -- Executive Assessment & Strategic Recommendations (`docs/reviews/requirements-review/executive-assessment.md`)

### Phase BT -- Pre-BS Requirements Consistency Fixes (MUST complete before Phase BS code)

> Resolves the four critical and two medium inconsistencies identified in the full requirements
> review. No Phase BS source code may be written until all BT tasks are marked [x].

- [x] BT.1 dataset-catalog/requirements.md Req 4 -- add superseded-by note pointing to Req 20
        (UUID layout); retain Req 4 for import-compatibility reference only
        (Task 31.1 in dataset-catalog/tasks.md)
- [x] BT.2 dataset-catalog/requirements.md Req 7 AC 6 -- remove physical-rename clause;
        replace with catalogue-only update cross-referencing Req 20.6
        (Task 31.2 in dataset-catalog/tasks.md)
- [x] BT.3 dataset-catalog/tasks.md + requirements.md -- replace all `ff-dataset-catalog`
        references with `ff-dscatalog` to match actual workspace crate name
        (Tasks 31.3, 31.4 in dataset-catalog/tasks.md)
- [x] BT.4 virtual-catalog-manager/requirements.md Req 16.1 -- replace DSN-to-path mapping
        rule with StorageProvider delegation note; align with UUID layout
        (Task 31.5 in dataset-catalog/tasks.md)
- [x] BT.5 virtual-catalog-manager/requirements.md Req 16.3 -- clarify staged-protocol
        applies under UUID layout; legacy DSN-path note retained for import compat
        (Task 31.6 in dataset-catalog/tasks.md)

### Phase BS -- Mainframe Dataset Architecture (CR-NR-016)

> Implements the hybrid storage architecture, record codecs, StorageProvider layer, VSAM/ISAM support,
> staged transactions, integrity/backup/restore, audit trail, and security hardening defined in
> `docs/source-documents/dataset-catalog/FileForgeWorkbench_Mainframe_Dataset_Architecture.md` and
> `docs/source-documents/dataset-catalog/FileForgeWorkbench_Virtual_File_and_Dataset_Storage_Requirements.md`.

#### Wave 1 -- Foundations (no dependencies on later waves)

- [x] BS.1 Record codecs -- `FixedCodec`, `VariableCodec`, `BinaryCodec`, `TextCodec` as independent module with full unit + property tests (Tasks 17.1–17.6 in dataset-catalog/tasks.md)
- [x] BS.2 `StorageProvider` trait definition + capability enum (Task 18.1)
- [x] BS.3 `NativeFileProvider` -- UUID-based allocation, path-safety guards, PS/PDS/GDG layout (Tasks 18.2–18.5)

#### Wave 2 -- VSAM and ISAM Providers (depends on Wave 1)

- [x] BS.4 `SqliteRecordProvider` base + VSAM KSDS -- keyed read/write, uniqueness, alternate-index extension point (Tasks 19.1–19.5; 19.4 alternate indexes remains; no system SQLite install required because `rusqlite` uses `bundled`)
- [x] BS.5 VSAM RRDS -- relative-record store, unallocated vs blank distinction (Tasks 20.1–20.3)
- [x] BS.6 VSAM ESDS -- append-oriented native file, stable record address, sidecar index (Tasks 21.1–21.4)
- [x] BS.7 ISAM -- SQLite-backed, shared indexed-record interface with KSDS (Tasks 22.1--22.3)

#### Wave 3 -- Transactions, Integrity, and Governance (depends on Wave 2)

- [x] BS.8 Staged transaction protocol -- `OperationJournal`, staged create/delete, startup recovery (Tasks 23.1–23.6)
- [x] BS.9 Integrity, backup, restore -- checksums, `workspace.backup/restore/diagnose/reconcile` commands (Tasks 24.1–24.6)
- [x] BS.10 Catalogue audit trail + schema migrations (Tasks 25.1-25.3)
- [x] BS.11 Security hardening -- parameterised SQL audit, log scrubbing, path-traversal property test (Tasks 26.1–26.3)

#### Wave 4 -- Catalogue Hierarchy and Editor Integration (depends on Wave 3)

- [x] BS.12 Master/user catalogue hierarchy, logical rename, scoped uniqueness (Tasks 27.1-27.4)
- [x] BS.13 Record-oriented editor integration -- wire codecs into open/save path, integration tests (Tasks 28.1–28.4)
- [x] BS.14 Non-functional validation -- cross-platform, performance, Git-compat, data-fidelity tests (Tasks 29.1–29.4)
- [x] BS.15 Update `dataset-catalog/design.md` for CR-NR-016 (Task 30.1)

---

### Phase CQ -- Enterprise Features (CR-NR-042)

> Adds audit logging, settings export/import, and locked config keys to `ff-config`.
> Extends configuration-system/requirements.md with Requirements 16-18.

- [x] CQ.1 Requirements gate -- configuration-system/requirements.md Req 16-18, design.md Section 12, tasks.md Tasks 30-32, TCR rows
- [x] CQ.2 Audit logging -- AuditEntry, AuditLog ring buffer, file persistence, query API (Task 30)
- [x] CQ.3 Settings export/import -- ExportScope, ImportTarget, ImportSummary, export/import pipeline (Task 31)
- [x] CQ.4 Locked config keys -- KeyLocked error, locked_keys enforcement in merger, is_locked API, Settings panel lock indicator (Task 32)
- [x] CQ.5 TCR update + cargo test --workspace green (Task 32.11-32.12)

---

### Phase CR -- OS Theme Follow + Macro Library Management (CR-NR-043)

> Adds OS dark/light mode follow to ff-theme/ff-desktop and a Macro Library
> management panel (POM option 6) to ff-desktop.
> Extends theme-and-appearance/requirements.md Req 16 and
> lua-macro-engine/requirements.md Req 12.

- [x] CR.1 Requirements gate -- theme-and-appearance Req 16, lua-macro-engine Req 12, tasks, TCR rows
- [x] CR.2 OS dark/light mode follow -- theme.follow_os config key, frame-level OS detection, Settings panel checkbox
- [x] CR.3 Macro Library panel -- TabKind::MacroLibrary, MACROS command, =6 fastpath, list/run/edit/delete/filter
- [x] CR.4 TCR update + cargo test --workspace green


### Phase CS -- Test Warning Cleanup (CR-NR-044)

> REFACTOR -- no requirements gate. Eliminates all compiler warnings in test code
> across the workspace. Categories: unused imports, unused variables, unnecessary mut,
> unused doc comments, dead code in test helpers. All pre-existing; none introduced
> by Phase CR. Uses `cargo fix` where safe; manual edits for the remainder.

- [x] CS.1 Run `cargo fix --tests --workspace` and review changes
- [x] CS.2 Fix remaining warnings not covered by cargo fix (unused doc comments, dead code)
- [x] CS.3 Run verify.ps1 -- ai-review.log must contain zero warning lines


### Phase CT -- Workbench/Workspace/Context Terminology Standardisation (CR-CH-009)

> Documentation-only pass. Applies the three-level Workbench/Workspace/Context model
> agreed in the post-CS terminology discussion across all 69 sub-project specifications,
> the canonical source documents, and the .amazonq/rules/ steering files.
> No source code changes. No requirements gate (documentation refactor only).

- [x] CT.1 Update canonical source documents: terminology-map.md, architecture-brief.md, README.md
- [x] CT.2 Update Workbench Shell specs: startup-and-session, menu-and-statusbar, function-keys-and-history, layout-and-docking, theme-and-appearance, view-zoom, line-wrap-toggle
- [x] CT.3 Update Explorer/Catalog specs: virtual-catalog-manager, file-tree-panel, dataset-catalog, dataset-allocator, virtual-file-system
- [x] CT.4 Update UX/Shell-adjacent specs: accessibility, plugin-manager-ui, notification-system, plugin-architecture, configuration-system, logging-subsystem, platform-core
- [x] CT.5 Update Content Editor specs: edit-operations, find-and-replace, undo-redo-transactions, caret-and-selection, clipboard-operations, viewport-and-scrolling, display-line-mapping, document-model, encoding-and-characters, background-io, file-operations, external-modification, multi-tab-editor
- [x] CT.6 Update remaining specs (bulk pass): all remaining sub-projects not covered in CT.2-CT.5
- [x] CT.7 Update .amazonq/rules/ steering files: specs.md, tdd-and-testing.md, rust-coding-standards.md, new-requirements-gate.md



| Status | Count |
|--------|-------|
| `[x]` Complete with real tests | 62 library crates + ff-desktop binary |
| `[x]` Complete -- Phase BT | Cross-File Replace + Search History (BT.1-BT.6) |
| `[x]` Phase CO complete | Accessibility, Plugin Manager UI, Notification System (CO.1-CO.7) |
| `[x]` Phase CP complete | Batch Command Execution (CP.1-CP.11) |
| `[x]` Phase W.5 complete | Generic ToolchainPlugin trait -- MockToolchain, audit, CI constraint |
| Test count | 731 passing, 0 failures |
| Active work | Phase CQ -- Enterprise Features COMPLETE. All 5 deliverables done. |

### Phase CO -- Accessibility, Plugin Manager UI, and Notification System (CR-NR-040)

> Implements the three highest-priority remaining gaps from the Phase BQ executive assessment.
> New sub-projects: `accessibility`, `plugin-manager-ui`, `notification-system`.
> Gate in progress -- requirements.md, design.md, tasks.md to be written before any code.

- [x] CO.1 Requirements gate -- accessibility/requirements.md, design.md, tasks.md, TCR rows
- [x] CO.2 Requirements gate -- plugin-manager-ui/requirements.md, design.md, tasks.md, TCR rows
- [x] CO.3 Requirements gate -- notification-system/requirements.md, design.md, tasks.md, TCR rows
- [x] CO.4 Accessibility implementation -- WCAG AA contrast, focus ring token+rendering, keyboard audit (Escape handlers), OS reduce-motion detection, TCR updated
- [x] CO.5 Plugin Manager UI -- PluginManager TabKind, routing (8/=8/PLUGINS), panel with filter+sort+detail, session persistence, 11 tests
- [x] CO.6 Notification System -- NotificationLevel/Queue/Sender, EventLog TabKind+panel, LOG command, channel drain in update(), 18 tests
- [x] CO.7 Integration tests + TCR rows updated to PASS

### Phase CP -- Batch Command Execution (CR-NR-041)

> Adds a headless batch execution mode to ffwb analogous to z/OS IKJEFT01 batch.
> New sub-project: `batch-execution`.
> Gate complete -- requirements.md, design.md, tasks.md written.

- [x] CP.1 Requirements gate -- batch-execution/requirements.md, design.md, tasks.md, TCR rows
- [x] CP.2 BatchInputSource -- file/stdin reader, comment/blank skip, continuation (Tasks 2.1-2.3)
- [x] CP.3 BatchOutputSink -- stdout/file/append routing, echo prefix (Tasks 3.1-3.2)
- [x] CP.4 Return code model -- StepReturnCode, BatchReturnCode, AbortPolicy (Tasks 4.1-4.2)
- [x] CP.5 BatchSession -- headless session context, config + catalog load (Tasks 5.1-5.4)
- [x] CP.6 BatchRunner orchestration -- command loop, abort policy, summary (Tasks 6.1-6.4)
- [x] CP.7 CLI entry point wiring -- --batch flag, branch in main.rs (Tasks 7.1-7.4)
- [x] CP.8 Dry-run mode (Tasks 8.1-8.4)
- [x] CP.9 FFCMD compatibility confirmation (Tasks 9.1-9.2)
- [x] CP.10 Logging integration (Tasks 10.1, 10.3-10.5)
- [x] CP.11 TCR update + cargo test --workspace green (Task 11.1-11.4)

### Phase BU -- SQLite Catalog Integration for Options 1 and 2 (CR-CH-006)

> Aligns the Files Panel (Option 1) and File Explorer Panel (Option 2) with the SQLite
> catalog architecture introduced in Phase BS. Replaces the in-memory AllocatedDataset
> HashMap and session-TOML persistence with direct reads/writes to the ff-dscatalog
> SQLite database. Replaces the DSN-derived path resolver with a catalog lookup.
>
> Sequence enforced: design docs -> failing tests -> implementation -> cleanup.

- [x] BU.1 Design docs updated -- VCM requirements.md Req 13 and 16 revised; design.md
        sections 7 and 10 revised (Tasks BU.D1-BU.D4 -- DONE)
- [x] BU.2 Failing tests written -- CatalogRegistry API, resolve_and_open_dataset,
        content area population (Tasks 18-20)
- [x] BU.3 CatalogRegistry::allocate() and list_datasets() implemented and tests green
        (Task 21)
- [x] BU.4 AllocOutcome::Confirmed handler wired to SQLite (Task 22)
- [x] BU.5 Files Panel content area reads from SQLite (Task 23)
- [x] BU.6 File Explorer Panel Mainframe content reads from SQLite (Task 24)
- [x] BU.7 resolve_and_open_dataset() replaces resolve_dataset_path() (Task 25)
- [x] BU.8 AllocatedDataset struct, datasets HashMap, and TOML persistence removed (Task 26)
- [x] BU.9 TCR.md and project-master updated; cargo test --workspace green (Task 27)

### Phase CH -- FFW-JES P2 EARS Integration (CR-NR-030)

- [x] CH.1 Overtype fields: visual distinction, direct overtype, command-line syntax, extension pop-up (Tasks 30.1-30.4)
- [x] CH.2 Help system: HELP/PF1, ACTH, COLH, CMDH, SEARCH (Tasks 31.1-31.5)
- [x] CH.3 Log panels: LOG, ULOG, NEXT/PREV, SNAPSHOT (Tasks 32.1-32.4)
- [x] CH.4 System panels: SYS, DASH, INIT, JC, SP (Tasks 32.5-32.9)
- [x] CH.5 Browse settings, PRINT action, COLS command (Tasks 33.1-33.3)
- [x] CH.6 SET P2 commands: BCOLOR, CONFIRM, CURSOR, DATE, DELAY, HEX, SCHARS, SCREEN + persistence (Tasks 34.1-34.9)

### Phase CI -- command-semantics P2 EARS Integration (CR-NR-031)

- [x] CI.1 OUTPUT and CANCEL commands -- routing to FFW-JES, PURGE operand (Tasks 25.1-25.3)
- [x] CI.2 SEND, PROFILE, PRINTDS commands -- messaging, session profile, file-ops routing (Tasks 26.1-26.4)
- [x] CI.3 TCR update for Requirement 10 (Task 27.1)

### Phase CG -- lua-macro-engine P2 EARS Integration (CR-NR-029)

- [x] CG.1 ISREDIT/ISPEXEC host command environments and IMACRO initial macro (Tasks 21.1-21.4)
- [x] CG.2 LINENUM function and CURSOR get/set extension (Tasks 21.5-21.6)
- [x] CG.3 REXX exec invocation: EXEC command, implicit invocation, % prefix, argument passing (Tasks 22.1-22.4)
- [x] CG.4 TSO host environment, ADDRESS switching, ISPEXEC/ISREDIT environments, RC variable (Tasks 22.5-22.8)
- [x] CG.5 REXX built-in functions: LISTDSI, MSG, MVSVAR, OUTTRAP, PROMPT, SYSDSN, SYSVAR, USERID (Tasks 23.1-23.8)
- [x] CG.6 EXECIO DISKR/DISKW/FINIS/SKIP and FFCMD command files (Tasks 24.1-24.7)

### Phase CF -- syntax-highlighting P2 EARS Integration (CR-NR-028)

> Adds EARS-derived criteria to syntax-highlighting: HILITE ON/OFF command (toggle
> syntax highlighting per document), HILITE LOGIC (boolean/comparison operator highlighting),
> HILITE PAREN (delimiter-pair matching at cursor with error style), HILITE FIND (persistent
> find-match highlights), and combined operand support.
> Requirement 16 in syntax-highlighting/requirements.md.

- [x] CF.1 HILITE ON/OFF command and HILITE LOGIC mode (Tasks 21.1-21.5)
- [x] CF.2 HILITE PAREN, HILITE FIND, and combined operands (Tasks 22.1-22.5)

---

### Phase CE -- undo-redo-transactions P2 EARS Integration (CR-NR-027)

> Adds EARS-derived criteria to undo-redo-transactions: SETUNDO primary command
> (ON/OFF/n operands, immediate effect) and RECOVERY primary command (ON/OFF/n operands).
> Requirement 19 in undo-redo-transactions/requirements.md.

- [x] CE.1 SETUNDO command -- ON/OFF/n operands, immediate effect on max_levels (Tasks 19.1-19.5)
- [x] CE.2 RECOVERY command -- ON/OFF/n operands, immediate effect on recovery interval (Tasks 20.1-20.5)

---

### Phase CD -- FFW-JES P1 extended EARS Integration (CR-NR-026)

> Adds EARS-derived criteria to FFW-JES: ST panel (all-jobs status view), FILTER command
> (advanced filter expressions with AND/OR/wildcard), FIND command (panel search with
> NEXT/PREV), LOCATE command (scroll to matching row), SDSF scroll commands (UP/DOWN/LEFT/RIGHT
> with n/HALF/PAGE/MAX), SET ACTION/MAIN/ROWNUM commands, WHO command, QUERY AUTH command,
> and PERSIST-1 (SET settings persistence).
> Requirement 17 in jes-emulator/requirements.md.

- [x] CD.1 ST panel and advanced FILTER/FIND/LOCATE commands (Tasks 26.1-26.5)
- [x] CD.2 SDSF scroll commands UP/DOWN/LEFT/RIGHT with SCROLL field sync (Tasks 27.1-27.3)
- [x] CD.3 SET ACTION/MAIN/ROWNUM commands, WHO, QUERY AUTH (Tasks 28.1-28.6)
- [x] CD.4 SET settings persistence and integration tests (Tasks 29.1-29.5)

---

### Phase CC -- FFW-JES P1 core EARS Integration (CR-NR-025)

> Adds EARS-derived criteria to FFW-JES: SDSF panel framework (action bar, title line,
> SCROLL field, filter lines, NP column, fixed column), action character system (S/?/C/H/A/P/D/E/J/W,
> = repeat, // block, command-line syntax, SET ROWNUM), main panel (MENU, groups, SET MAIN GROUP),
> PREFIX/OWNER/DEST filter commands, full column set, SORT command.
> Requirement 16 in jes-emulator/requirements.md.

- [x] CC.1 SDSF panel chrome -- action bar, title line, SCROLL field, filter lines, message area, COMMAND INPUT field (Tasks 20.1-20.7)
- [x] CC.2 NP column and action character system -- S/?/C/H/A/P/D/E/J/W, = repeat, // block, command-line syntax, SET ROWNUM (Tasks 21.1-21.8)
- [x] CC.3 Main panel (MENU command), command groups, S action, SET MAIN GROUP (Tasks 22.1-22.5)
- [x] CC.4 PREFIX/OWNER/DEST filter commands (Tasks 23.1-23.4)
- [x] CC.5 Full column set (JOBNAME through PROCSTEP), column hide/reorder, SORT command (Tasks 24.1-24.4)
- [x] CC.6 Integration tests for SDSF panel framework (Tasks 25.1-25.4)

---

### Phase CB -- command-semantics EARS Integration (CR-NR-024)

> Adds EARS-derived criteria to command-semantics: TSO dataset commands (ALLOCATE through
> STATUS), EDIT routing extension, FTSO operand parsing, session prefix, continuation,
> ds:// URI, namespace conflict, capability model, secret operands, audit events.
> Requirement 9 in command-semantics/requirements.md.

- [x] CB.1 TSO dataset management commands ALLOCATE/FREE/DELETE/RENAME/LISTCAT/LISTDS/LISTALC (Tasks 19.1-19.8)
- [x] CB.2 TSO job commands SUBMIT/STATUS and EDIT routing extension (Tasks 20.1-20.4)
- [x] CB.3 TSO-style operand parsing and session prefix (Tasks 21.1-21.4)
- [x] CB.4 Command continuation, ds:// URI, namespace conflict resolution (Tasks 22.1-22.4)
- [x] CB.5 Capability model, secret operands, audit events (Tasks 23.1-23.5)
- [x] CB.6 TCR update for Requirement 9 (Task 24.1)

---

Phase CA -- startup-and-session EARS Integration (CR-NR-023)

> Adds EARS-derived criteria to startup-and-session: session start timestamp (TSO-1.2),
> session end timestamp and logoff message (TSO-1.3), LOGOFF command (TSO-1.4),
> TIME command (TSO-2.4), STATUS routing to FFW-JES (TSO-2.5).
> Requirement 20 in startup-and-session/requirements.md.

- [x] CA.1 Session start timestamp in status bar (Tasks 28.1-28.3)
- [x] CA.2 Session end timestamp and logoff message (Tasks 29.1-29.3)
- [x] CA.3 LOGOFF command alias for exit sequence (Tasks 30.1-30.2)
- [x] CA.4 TIME command -- date/time display (Tasks 31.1-31.2)
- [x] CA.5 STATUS command routing to FFW-JES (Tasks 32.1-32.3)
- [x] CA.6 TCR update for Requirement 20 (Task 33.1)

---

Phase BZ -- menu-and-statusbar EARS Integration (CR-NR-022)

> Adds EARS-derived criteria to menu-and-statusbar: SCROLL ===> field, fastpath notation,
> data entry/list panel layout, list panel LOCATE, extended scroll amounts, split screen.
> Requirement 19 in menu-and-statusbar/requirements.md.

- [x] BZ.1 SCROLL ===> field adjacent to Command ===> (Tasks 24.1-24.5)
- [x] BZ.2 Fastpath dotted notation navigation (Tasks 25.1-25.3)
- [x] BZ.3 Data entry and list panel layout conformance (Tasks 26.1-26.4)
- [x] BZ.4 List panel LOCATE command (Tasks 27.1-27.3)
- [x] BZ.5 Extended scroll amounts HALF/CSR/MAX/DATA (Tasks 28.1-28.3)
- [x] BZ.6 Split screen PF2/PF9/PF3 (Tasks 29.1-29.6)
- [x] BZ.7 TCR update for Requirement 19 (Task 30.1)

---

Phase BY -- sequence-numbers EARS Integration (CR-NR-021)

> Extends sequence-numbers: AUTONUM ON/OFF as alias for NUMBER ON/OFF (Req 6.7a),
> NUM as alias for NUMBER command (Req 8 alias criterion).

- [x] BY.1 AUTONUM alias -- parser extension, unit tests (Task 20.1-20.2)
- [x] BY.2 NUM alias -- command framework alias registration, unit tests (Task 21.1-21.3)
- [x] BY.3 TCR update for alias criteria (Task 22.1)

---

Phase BX -- line-commands EARS Integration (CR-NR-020)

> Adds EARS-derived criteria to line-commands: Overlay (O/On), clipboard copy (W/WW),
> first-of-excluded (F), last-of-excluded (L), single-column shift right (]), show-excluded (S).
> Requirement 15 in line-commands/requirements.md.

- [x] BX.1 Overlay line command (O, On) -- LineCommandKind variants, parser, execution (Tasks 22.1-22.6)
- [x] BX.2 Clipboard copy line command (W, WW) -- variants, parser, ff-clipboard integration (Tasks 23.1-23.6)
- [x] BX.3 First-of-excluded (F) -- ShowFirst variant, parser, execution (Tasks 24.1-24.6)
- [x] BX.4 Last-of-excluded (L) -- ShowLast variant, parser, execution (Tasks 25.1-25.6)
- [x] BX.5 Single-column shift right (]) -- ShiftRightOne variant, parser, delegate to shift_right(1) (Tasks 26.1-26.6)
- [x] BX.6 Show-excluded (S) -- ShowLine variant, parser, execution (Tasks 27.1-27.6)
- [x] BX.7 TCR update for Requirement 15 (Task 28.1)

---

Phase BW -- edit-operations EARS Integration (CR-NR-019)

> Adds EARS-derived criteria to edit-operations: CAPS mode, NULLS mode, PROFILE command,
> STATS mode, LOCK setting, edit profile persistence, AUTONUM/NUM aliases, HILITE delegation,
> SUBMIT/CREATE/REPLACE/EDIT/BROWSE/VIEW/COMPARE primary commands.
> Requirements 16-17 in edit-operations/requirements.md.

- [x] BW.1 CAPS mode -- CapsMode flag, insert_char integration, CAPS command (Tasks 28.1-28.6)
- [x] BW.2 NULLS mode -- NullsMode flag, display/edit integration, NULLS command (Tasks 29.1-29.5)
- [x] BW.3 PROFILE command -- EditProfile struct, display and update handlers (Tasks 30.1-30.6)
- [x] BW.4 STATS mode -- StatsMode flag, prefix area rendering (Tasks 31.1-31.5)
- [x] BW.5 LOCK setting -- ProfileLock flag, profile mutation guard (Tasks 32.1-32.5)
- [x] BW.6 Edit profile persistence -- TOML round-trip via ff-session (Tasks 33.1-33.4)
- [x] BW.7 AUTONUM and NUM aliases -- command framework aliases (Tasks 34.1-34.4)
- [x] BW.8 HILITE delegation -- HILITE command handler to ff-syntax (Tasks 35.1-35.5)
- [x] BW.9 SUBMIT command -- JES subsystem dispatch (Tasks 36.1-36.5)
- [x] BW.10 CREATE and REPLACE commands (Tasks 37.1-37.6)
- [x] BW.11 Nested EDIT, BROWSE, VIEW, COMPARE commands (Tasks 38.1-38.8)
- [x] BW.12 TCR.md and project-master updated; cargo test --workspace green (Task 39)

### Phase BV -- Catalog Location Discriminant (CR-NR-017)

> Adds `CatalogLocation` enum to `CatalogMount` so each catalog declares local vs remote
> transport. Only `Local` is implemented; `Remote` returns `UnsupportedOperation`. Zero
> behaviour change for existing local catalogs.

- [x] BV.1 `CatalogLocation` enum + `CatalogMount` refactor in `ff-dscatalog`
        (Tasks 32.1-32.8 in dataset-catalog/tasks.md)

### Phase BR -- Requirements Maintenance (CA-01, CA-02, housekeeping)

> Low-effort documentation fixes that clear the path for Phase BS.
> No source code changes -- all changes are in docs/ and .amazonq/rules/.

- [x] BR.1 CA-01: Fix compiler-toolchain-integration/tasks.md requirement annotations
        (Req 15.x/16.x/17.x/18.x -> Req 1.x/2.x/3.x/4.x to match actual requirements.md)
- [x] BR.2 CA-02: Add Requirement 5 (Generic ToolchainPlugin Extension Point, FR-0971)
        to compiler-toolchain-integration/requirements.md and tasks.md (Tasks 5.1-5.3)
- [x] BR.3 Rename docs/specs/FFW-JES/ to docs/specs/jes-emulator/ (kebab-case convention);
        update .amazonq/rules/specs.md sub-project list; update naming note in requirements.md
- [x] BR.4 B009 marked SUPERSEDED in bugs.md (resolved by Phase BU SQLite integration)
- [x] BR.5 CR-NR-035 status updated to DONE in change-log.md (Phase CN completed it)

---


> EI-5 is 100% complete (all 16 batches done, phases BW-CI gated).
> The sections below list all pending implementation work in correct
> dependency order. EARS phases BW-CI have requirements and tasks but
> no implementation yet.

### Stream 1 -- Dataset Architecture (BS Wave 3-4 -> ff-vfs -> BU)

Dependency chain: BV.1 -> BS.8 -> BS.9 -> BS.10 -> BS.11 -> BS.12 -> BS.13 -> BS.14 -> BS.15 -> ff-vfs tasks -> BU.2-BU.9

- [x] BV.1 `CatalogLocation` enum + `CatalogMount` refactor in `ff-dscatalog` (Tasks 32.1-32.8)
- [x] BS.8 Staged transaction protocol -- `OperationJournal`, staged create/delete, startup recovery (Tasks 23.1-23.6)
- [x] BS.9 Integrity, backup, restore -- checksums, `workspace.backup/restore/diagnose/reconcile` (Tasks 24.1-24.6)
- [x] BS.10 Catalogue audit trail + schema migrations (Tasks 25.1-25.3)
- [x] BS.11 Security hardening -- parameterised SQL audit, log scrubbing, path-traversal PBT (Tasks 26.1-26.3)
- [x] BS.12 Master/user catalogue hierarchy, logical rename, scoped uniqueness (Tasks 27.1-27.4)
- [x] BS.13 Record-oriented editor integration -- wire codecs into open/save path (Tasks 28.1-28.4)
- [x] BS.14 Non-functional validation -- cross-platform, performance, Git-compat, data-fidelity (Tasks 29.1-29.4)
- [x] BS.15 Update `dataset-catalog/design.md` for CR-NR-016 (Task 30.1)
- [x] ff-vfs.13 StorageProvider trait in ff-vfs (Tasks 13.1-13.5 in virtual-file-system/tasks.md)
- [x] ff-vfs.14 POSIX files as native objects (Tasks 14.1-14.6)
- [x] ff-vfs.15 VFS staged transaction protocol (Tasks 15.1-15.5)
- [x] ff-vfs.16 workspace.backup/restore/reconcile/diagnose (Tasks 16.1-16.5)
- [x] BU.1 Design docs updated (DONE)
- [x] BU.2 Failing tests -- CatalogRegistry API, resolve_and_open_dataset, content area (Tasks 18-20)
- [x] BU.3 CatalogRegistry::allocate() and list_datasets() implemented and tests green (Task 21)
- [x] BU.4 AllocOutcome::Confirmed handler wired to SQLite (Task 22)
- [x] BU.5 Files Panel content area reads from SQLite (Task 23)
- [x] BU.6 File Explorer Panel Mainframe content reads from SQLite (Task 24)
- [x] BU.7 resolve_and_open_dataset() replaces resolve_dataset_path() (Task 25)
- [x] BU.8 AllocatedDataset struct, datasets HashMap, and TOML persistence removed (Task 26)
- [x] BU.9 TCR.md and project-master updated; cargo test --workspace green (Task 27)

### Stream 2 -- EARS P1 Implementation (independent of Stream 1)

- [x] BW.impl edit-operations: CAPS/NULLS/PROFILE/SUBMIT/CREATE/REPLACE/BROWSE/VIEW/nested EDIT/COMPARE/LOCK/STATS (Tasks 28-39 in edit-operations/tasks.md)
- [x] BX.impl line-commands: O/W/F/L/]/S line commands (Tasks 22-28 in line-commands/tasks.md)
- [x] BY.impl sequence-numbers: AUTONUM and NUM alias extensions (Tasks 20-22 in sequence-numbers/tasks.md)
- [x] BZ.impl menu-and-statusbar: SCROLL field, fastpath, split screen, list panel LOCATE (Tasks 24-30 in menu-and-statusbar/tasks.md)

- [x] CA.impl startup-and-session: session timestamps, LOGOFF, TIME, STATUS routing (Tasks 28-33 in startup-and-session/tasks.md)
- [x] CB.impl command-semantics P1: ALLOCATE through STATUS + FTSO operand parsing (Tasks 19-24 in command-semantics/tasks.md)
- [x] CC.impl FFW-JES P1 core: SDSF panel framework, NP column, action chars, main panel (Tasks 20-25 in jes-emulator/tasks.md)
- [x] CD.impl FFW-JES P1 extended: ST panel, FILTER/FIND/LOCATE, SET P1 commands (Tasks 26-29 in jes-emulator/tasks.md)

### Stream 3 -- EARS P2 Implementation (follows Stream 2)

- [x] CE.impl undo-redo-transactions: SETUNDO command, RECOVERY ON/OFF (Tasks 19-20 in undo-redo-transactions/tasks.md)
- [x] CF.impl syntax-highlighting: HILITE ON/OFF/LOGIC/PAREN/FIND (Tasks 21-22 in syntax-highlighting/tasks.md)
- [x] CG.impl lua-macro-engine: ISREDIT/ISPEXEC/IMACRO, REXX bridge, FFCMD (Tasks 21-24 in lua-macro-engine/tasks.md)
- [x] CH.impl FFW-JES P2: overtype, help, log/system panels, browse/print, SET P2 (Tasks 30-34 in jes-emulator/tasks.md)
- [x] CI.impl command-semantics P2: OUTPUT/CANCEL/SEND/PROFILE/PRINTDS (Tasks 25-27 in command-semantics/tasks.md)

### Phase CJ -- Bootstrap Scripts (CR-NR-032)

> Provides platform-specific scripts in `bootstrap/` that install the Rust
> stable toolchain without admin rights and guide a new contributor from
> `git clone` to a passing `cargo build` on Windows, Linux, and macOS.

- [x] CJ.1 Create `bootstrap/` directory, `logs/.gitkeep`, `.gitignore` entry (Task 1-2)
- [x] CJ.2 Write `bootstrap/bootstrap-windows.ps1` (Task 3)
- [x] CJ.3 Write `bootstrap/bootstrap-linux.sh` (Task 4)
- [x] CJ.4 Write `bootstrap/bootstrap-macos.sh` (Task 5)
- [x] CJ.5 Write `bootstrap/README.md` (Task 6)
- [x] CJ.6 Update root `README.md` Building section to reference `bootstrap/` (Task 7)

### Phase CL -- POM Guaranteed on Startup (CR-CH-007, Req 14.1a, 14.1b)

- [x] CL.1 Amend startup block in `shell/update.rs`: after session restore, if no POM tab exists prepend one at index 0; 3 unit tests; B001 FIXED (Tasks 34.1-34.9 in startup-and-session/tasks.md)

### Phase CM -- Mouse Text Selection and Clipboard Copy (CR-NR-034)

> Adds mouse-driven text selection to the editor canvas and Ctrl+C copy to the OS clipboard.
> Also makes read-only panels (POM, Settings, status bar) use egui selectable labels.
> Requirements 13-14 in caret-and-selection/requirements.md; Requirement 20 in clipboard-operations/requirements.md.

- [x] CM.1 Editor canvas mouse selection -- drag tracking, highlight rendering, Ctrl+C copy, Escape clear (Tasks 17.1-17.10 in caret-and-selection/tasks.md)
- [x] CM.2 Read-only panel selectable labels -- POM calendar, Settings key/desc/badge, status bar fields (Tasks 18.1-18.5 in caret-and-selection/tasks.md)

### Phase CN -- Editor Scroll Amount Integration (CR-NR-035)

> Wires the existing SCROLL ===> field value (PAGE/HALF/CSR/MAX/DATA/N) into the editor
> panel Page Up/Down handler so paging behaviour matches the ISPF convention.
> Requirement 14 in viewport-and-scrolling/requirements.md.

- [x] CN.1 Pass scroll_amount into editor_panel::render(); implement scroll_by_amount helper; wire all ScrollAmount variants; 5 unit tests (Task 16.1-16.6 in viewport-and-scrolling/tasks.md)

### Phase CK -- FFTest Automated Dialog Testing Framework (CR-NR-033)

> Introduces the native FFTest Automated Dialog Testing Framework. Covers
> Automation ID infrastructure, FFTest script parser and runner, headless
> execution, HTML/JSON reporting, and visual regression. Requirement 1-10
> in automated-dialog-testing/requirements.md.

- [x] CK.1 Requirements gate -- requirements.md, design.md, tasks.md, TCR rows (Phase CK-1)
- [x] CK.2 Automation ID infrastructure -- ff-fftest crate scaffold, AutomationRegistry trait,
        Automation ID constants, ff-desktop registration (Phase CK-2, Tasks 8-15)
- [x] CK.3 FFTest parser, runner, assertion engine (Phase CK-3, Tasks 16-24)
- [x] CK.4 Headless runner, reporting, visual regression, plugin testing (Phase CK-4, Tasks 25-38)

---

## Summary (superseded -- see final summary below)

> This table was current after Phase BS. See the final summary at the bottom of this file for the up-to-date state.

### Phase BS -- Productivity Core (CR-NR-036, CR-NR-037, CR-NR-038)

> Implements the three highest-impact productivity features identified in the Phase BQ
> executive assessment: Workspace Model (foundational), Command Palette, and Global Search.
> All three sub-project specs are gated and approved. Implementation follows TDD.

#### BS-A: Workspace Model (workspace-model sub-project)

- [x] BS-A.1 WorkspaceState data model and serialisation -- ff-session workspace.rs (Tasks 1.1-1.4)
- [x] BS-A.2 Session persistence for active workspace path (Tasks 2.1-2.4)
- [x] BS-A.3 Workspace lifecycle commands: OPEN/SAVE/SAVE AS/CLOSE in ff-desktop (Tasks 3.1-3.5)
- [x] BS-A.4 Workspace root management: catalog registration, ADD/REMOVE ROOT commands (Tasks 4.1-4.6)
- [x] BS-A.5 Workspace-scoped settings: inject/remove Workspace config layer (Tasks 5.1-5.3)
- [x] BS-A.6 Workspace-scoped recent files MRU list (Tasks 6.1-6.4)

#### BS-B: Command Palette (command-palette sub-project)

- [x] BS-B.1 Fuzzy match engine -- fuzzy_match() and fuzzy_score() pure functions (Tasks 1.1-1.2)
- [x] BS-B.2 Palette state and data model; recent commands in SessionState (Tasks 2.1-2.4)
- [x] BS-B.3 Palette rendering -- centered egui::Window, entry list, detail area (Tasks 3.1-3.6)
- [x] BS-B.4 Activation (Ctrl+Shift+P), keyboard nav, command execution (Tasks 4.1-4.8)

#### BS-C: Global Search (global-search sub-project)

- [x] BS-C.1 ff-global-search crate scaffold -- types and SearchEvent (Tasks 1.1-1.5)
- [x] BS-C.2 Search engine: file enumeration, per-file FindEngine delegation, cancellation (Tasks 2.1-2.5)
- [x] BS-C.3 Replace engine: cross-file replace via ff-file-ops (Tasks 3.1-3.3)
- [x] BS-C.4 Search Results panel -- TabKind, state, rendering, keyboard nav (Tasks 4.1-4.7)
- [x] BS-C.5 Replace UI, activation (Ctrl+Shift+F), search history (Tasks 5.1-5.7)

---

### Phase BT -- Cross-File Search and Replace (global-search Req 5, Req 6)

> Implements the replace pipeline and search history in ff-global-search and the
> Search Results panel in ff-desktop. All requirements already exist in
> global-search/requirements.md (Req 5.1-5.7, Req 6.1-6.3).

- [x] BT.1 GlobalReplaceEngine::replace_all() -- read file, apply FindEngine::replace_all(),
        write via ff-file-ops; unsaved-changes conflict detection (Req 5.3, 5.6)
- [x] BT.2 Replace input field, Replace All button, per-file Replace buttons in Search
        Results panel (Req 5.1)
- [x] BT.3 Replace_Preview confirmation dialog -- file/match counts before writing (Req 5.2)
- [x] BT.4 Wire Replace All: spawn replace task via ff-bgio, show summary on completion;
        regex group substitution support (Req 5.4, 5.5, 5.7)
- [x] BT.5 Search history dropdown -- last 20 queries, persisted in session state,
        restored on launch, options round-trip (Req 6.1, 6.2, 6.3)
- [x] BT.6 Integration tests: replace modifies files, history persists, unsaved-changes
        guard fires; TCR rows updated to PASS (Req 5.3, 5.6, 6.2)

---

## Summary (current -- updated after full sub-project audit)

### Phase CU -- Menu Workspace Pattern (CR-NR-045) -- SPEC ONLY

> Defines the Menu Workspace as a first-class pattern: a Workspace whose Context is a
> list of options loaded from a TOML config file. The POM becomes an instance of this
> pattern. No source code changes until spec is approved and Phase CU implementation
> is explicitly started.

- [x] CU.1 Create `docs/specs/menu-workspace/requirements.md` (Reqs 1-4: definition, chaining, POM-as-menu, per-menu config file)
- [x] CU.2 Create `docs/specs/menu-workspace/design.md` (architecture: MenuWorkspaceState, TOML loader, hot-reload, chained path resolver)
- [x] CU.3 Create `docs/specs/menu-workspace/tasks.md` (implementation tasks, numbered from 1)
- [x] CU.4 Update `docs/specs/startup-and-session/requirements.md` Req 14.3 to reference Menu Workspace pattern
- [x] CU.5 Update `docs/quality/TCR.md` with CR-NR-045 NOT COVERED rows
- [x] CU.6 Add `menu-workspace` to `.amazonq/rules/specs.md` sub-project list

---

### Phase CV -- POM Redesign Spec (CR-NR-048) -- SPEC ONLY, depends on CU

> Reviews and revises the POM option list. Defines the default menus/pom.toml content.
> Depends on Phase CU (Menu Workspace pattern) being approved first.

- [x] CV.1 Review current POM options (0-8) against all implemented functionality
- [x] CV.2 Define revised 12-option list (0-8 unchanged, add 9/S/B for Jobs/Search/Batch)
- [x] CV.3 Create `docs/specs/menu-workspace/cv-requirements.md` (Reqs 6-8: option list, pom.toml content, Req 14.3 update)
- [x] CV.4 Update `docs/specs/startup-and-session/requirements.md` Req 14.3 with revised 12-option list
- [x] CV.5 Update `docs/quality/TCR.md` with CR-NR-048 NOT COVERED rows

---

### Phase CW -- Settings as a Menu Workspace (CR-NR-047) -- SPEC ONLY, depends on CU

> Restructures the Settings Context as a Menu Workspace. Defines the default
> menus/settings.toml content with one option per major config namespace.
> Depends on Phase CU (Menu Workspace pattern) being approved first.

- [x] CW.1 Create `docs/specs/menu-workspace/cw-requirements.md` (Reqs 9-12: Settings_Menu, namespace view, settings.toml content, Req 15 update)
- [x] CW.2 Define 10-option Settings_Menu (E/T/C/V/L/K/S/P/X/A) with Namespaces and All groups
- [x] CW.3 Update `docs/specs/configuration-system/requirements.md` Req 15 with Phase CW two-level navigation note
- [x] CW.4 Update `docs/quality/TCR.md` with CR-NR-047 NOT COVERED rows

---

### Phase CX -- Named Workspaces and KEYS + SPLIT Command (CR-NR-046, CR-CH-010)

> Adds user-visible names to Workspaces, extends the KEYS command to accept a name
> argument, and adds the SPLIT command as an ISPF-heritage alias for Workspace detach.
> This phase includes both spec updates and implementation.

- [x] CX.1 Update `docs/specs/function-keys-and-history/requirements.md` Req 20 with named Workspace KEYS extension (CR-NR-046)
- [x] CX.2 Update `docs/specs/layout-and-docking/requirements.md` Req 3 with SPLIT command alias (CR-CH-010)
- [x] CX.3 Update `docs/quality/TCR.md` with CR-NR-046 and CR-CH-010 NOT COVERED rows
- [x] CX.4 Implement Workspace name property in `ff-desktop` (TabState gains a `workspace_name: Option<String>` field)
- [x] CX.5 Implement `KEYS <name>` routing in `ff-desktop` shell command handler
- [x] CX.6 Implement `SPLIT` command alias in `ff-desktop` (maps to existing detach logic)
- [x] CX.7 Update `docs/quality/TCR.md` rows to PASS after implementation

---

### Phase CZ -- FFTest Script Suite and Context Inspection (CR-NR-049)

> Extends the FFTest framework with Workspace context inspection assertions and
> automatic bug report generation. Writes the full FFTest script suite covering
> all major functional areas.

- [x] CZ.1 Update `docs/specs/automated-dialog-testing/requirements.md` with Reqs 11-13 (context inspection, bug logging, script suite)
- [x] CZ.2 Update `docs/quality/TCR.md` with CR-NR-049 NOT COVERED rows
- [x] CZ.3 Implement `ASSERT CONTEXT IS` and `ASSERT WORKSPACE COUNT IS` in `ff-fftest`
- [x] CZ.4 Implement automatic bug report generation (`reports/bugs-from-tests.md`) in `ff-fftest`
- [x] CZ.5 Write FFTest script suite: POM navigation, file open/save/close, editor input/undo (tests/dialog/)
- [x] CZ.6 Write FFTest script suite: catalog create/edit/delete, dataset allocation, settings navigation (tests/dialog/)
- [x] CZ.7 Write FFTest script suite: key config dialog, compiler context, plugin manager, notification system (tests/dialog/)
- [x] CZ.8 Write FFTest workflow scripts: batch execution, global search, command palette (tests/workflow/)
- [x] CZ.9 Update `docs/quality/TCR.md` rows to PASS after implementation

---

### Phase DA -- Configurable Menu Option Limits (CR-NR-050) -- SPEC ONLY, depends on CU

> Adds configurable soft (default 64) and hard (default 256) limits on the
> number of options in a Menu_File, replacing the implicit "no cap" behaviour.
> Soft limit warns and advises sub-menus; hard limit rejects the file as a load
> error. Both are configuration keys resolved through the layered config system.

- [x] DA.1 Add Requirement 9 (Configurable Menu Option Limits) to `docs/specs/menu-workspace/requirements.md`
- [x] DA.2 Add design section 12A to `docs/specs/menu-workspace/design.md`
- [x] DA.3 Add Phase DA-impl tasks (16-20) to `docs/specs/menu-workspace/tasks.md`
- [x] DA.4 Add CR-NR-050 rows to `docs/quality/TCR.md` (now PASS)
- [x] DA.5 Register `menu.soft_option_limit` / `menu.hard_option_limit` config keys (impl)
- [x] DA.6 Implement loader limit evaluation and advisory (impl)
- [x] DA.7 Render advisory line and re-evaluate on hot-reload (impl)

---

### Phase DB -- Unified Command Target, Command Configurator, Descriptor Persistence (CR-NR-051, CR-NR-052, CR-CH-012) -- SPEC ONLY, depends on CU

> Introduces one `CommandTarget` abstraction (menu / custom-workspace / function
> / macro / external) that menu options and keybindings both point at; a
> Command Configurator Custom Workspace storing user-defined commands in
> `commands/commands.toml`; external execution in Detached (fire-and-forget
> Started Task) and Captured (async, output shown in the Output_Panel) modes
> gated by `shell.mode`; and descriptor-based session persistence that restores
> every visible Workspace and excludes Started Tasks -- structurally fixing the
> non-file-tab restore gap and folding in the superseded CR-CH-011 (Task 14.5).

- [ ] DB.1 Add Requirement 8 (Unified Command Target) to `docs/specs/command-framework/requirements.md`
- [ ] DB.2 Add the Unified Command Target design section to `docs/specs/command-framework/design.md`
- [ ] DB.3 Create `docs/specs/command-configurator/` (requirements.md, design.md, tasks.md); register in `specs.md`
- [x] DB.4 Add Requirement 10 (options reference a Command_Target) + design section 13 to `docs/specs/menu-workspace/` (spec was already present) AND wire the runtime binding (Option B): MenuOption gains inline `[options.target]` (Req 10.6); shell `ShellTargetResolver` (user defs + registry) + `command_store` loaded at startup; menu-option dispatch (typed + click) and keyboard-shortcut/label-bar dispatch route through `resolve_target` -> `dispatch_command_target`, preserving bare-string behaviour via fall-through (Req 10.2); Function targets run via the existing pipeline; `run_command_definition` reports `Command '<id>' is not defined.` (cc Req 4.5). 11 tests. Req 10.1-10.3, 10.6, cmd-framework 8.3-8.6, cc 4.3-4.5 PASS. DEFERRED (separate steps): `MENU <name>` command for full Menu_Target execution (menu-workspace Req 11, 10.4); External/CustomWorkspace/Macro binding execution (needs the ff-shell desktop adapter, cc Req 3) -- these report a deferred status when bound.
- [ ] DB.5 Add Requirement 19 (External Program Execution: Detached/Captured) to `docs/specs/shell-command/requirements.md`
- [ ] DB.6 Add Requirement 21 (Descriptor-Based Persistence) to `docs/specs/startup-and-session/requirements.md`; annotate 14.1a / 19.12; note workspace-model Req 5
- [ ] DB.7 Add CR-NR-051 / CR-NR-052 / CR-CH-012 NOT COVERED rows to `docs/quality/TCR.md`
- [x] DB.8 Implement `CommandTarget` type + `resolve_target` + `execute_target` in `ff-command` (5 variants, TargetResolver/TargetExecutor seams, TOML round-trip, visible-workspace classification; 16 tests; Req 8.1-8.4, 8.7-8.9. Req 8.5/8.6 wiring deferred to DB.4)
- [~] DB.9 Implement Command_Store loader/save/hot-reload and the Command Configurator Context. DATA LAYER DONE: command_config/{mod,store}.rs -- CommandDefinition/CommandStore load/save/validate/duplicate-skip/hot-reload-poll, UserCommandStore TargetResolver view, validation incl. reserved-id (Req 1.1-1.7, 4.1-4.3, 4.6; 18 tests). DEFERRED: Context UI (Req 2.1/2.2/2.6/2.7/2.8, Edit UI), Req 1.8 default content, Req 4.4/4.5 binding (DB.4). External execution (Req 3) is DB.10.
- [x] DB.10 (impl) Implement External Detached spawn + Captured async into Output_Panel in `ff-shell` (ExecutionMode/TaskHandle/ExternalOutcome + spawn_detached seam in executor/external.rs; ShellEngine::execute_external + spawn_detached with shell.mode gate, working_dir fallback, captured output to Output_Panel, SpawnFailed on launch failure; 9 tests. shell-command Req 19.1-19.6, 19.8 PASS, 19.7 timeout MANUAL. command-configurator Req 3.2-3.5, 3.7, 3.9 PASS. Placeholder expansion Req 3.6, prompt-confirm Req 3.8, and Req 3.10 classification are the ff-desktop adapter's share, deferred to DB.4/desktop wiring.)
- [x] DB.11 Implement Workspace_Descriptor persistence + backward-compatible load; wire descriptor restore loop; resolve Task 14.5. (ff-session: WorkspaceKind/WorkspaceDescriptor/DescriptorParams + TabState.descriptor + effective_descriptor legacy mapping, 8 tests. ff-desktop: descriptor_for_tab/session_tab_for save helpers, restore_workspace_descriptors, settings_namespace threaded into save, 7 restore tests. Req 21.1-21.3, 21.5, 21.6, 21.8, 21.9, 21.10 PASS; 21.4 menu-restore and 21.7 captured-output assertion land with DB.4/DB.10.)

---

### Phase DC -- Two-Phase Logging Init (CR-CH-015, bug B033) -- SPEC ONLY

> Fixes B033: `logging.directory` is ignored at runtime because `ff-desktop`
> calls `init_default()` before config loads and never reconfigures. Adds
> logging-subsystem Requirement 11 (Runtime Reconfiguration) and a new
> `ff_logging::reconfigure(LogConfig)` API, then wires the desktop startup to
> re-apply the loaded `logging.*` settings. Makes config-only log redirection
> work with no recompile.

- [x] DC.1 Requirements gate -- logging-subsystem/requirements.md Req 11 (AC 11.1-11.10), design.md Section 11, tasks.md Tasks 21-22, TCR rows
- [x] DC.2 Implement `ff_logging::reconfigure` -- ChannelMessage::Reconfigure, writer-thread file swap (`switch_directory`), atomic level update, WARN on failure, INFO on switch (logging-subsystem Task 21)
- [x] DC.3 Wire two-phase init in `ff-desktop` main.rs -- keep init_default() first, build LogConfig from resolved config keys (`apply_logging_config`), call reconfigure after config load and before GUI shell (logging-subsystem Task 22)
- [x] DC.4 Tests -- reconfigure unit/integration + Property 11 (ff-logging); desktop tests asserting project logging.directory resolves and apply_logging_config is panic-safe
- [x] DC.5 Update `docs/quality/TCR.md` rows to PASS after implementation; verify.ps1 clean (9008 tests pass, ai-review.log empty)

---

### Phase DD -- Logging Inventory and Gap Report Tool (CR-NR-055)

> Adds a read-only Python maintenance tool that scans the workspace and
> regenerates a logging inventory (every log call site by crate/level) plus a
> gap report (crates with no logging, silent-error candidates). Output is a
> tracked artefact under `docs/quality/`; stdout mirrored to `tools/logs/`.
> Adds logging-subsystem Requirement 12 and Tasks 23-24. No `ff-logging`
> crate source changes.

- [ ] DD.1 Requirements gate -- logging-subsystem/requirements.md Req 12 (AC 12.1-12.9), design.md Section 12 + Property 12, tasks.md Tasks 23-24, TCR rows
- [ ] DD.2 Implement `tools/python/logging_inventory.py` -- scan crates, detect call sites + gaps + silent-error candidates, write `docs/quality/logging-inventory.md`, mirror log (logging-subsystem Task 23)
- [ ] DD.3 Run and verify -- report generated, run-twice determinism, usage note in `tools/README.md` (logging-subsystem Task 24)
- [ ] DD.4 Update `docs/quality/TCR.md` Req 12 rows to their correct status

---

### Phase DE-fix -- END/RETURN Workspace-Close Semantics (CR-CH-016) -- depends on function-keys-and-history

> Revises END/RETURN so that issuing either from a POM tab closes only that POM
> Workspace when other Workspaces remain open, and terminates the application only
> when the POM is the last Workspace (new criterion 17.2a). Reconciles the
> Key_Label_Bar blank-slot rule (Req 4.3 aligned to 13.2) and the default
> Excluded_Command set (Req 8.2 + glossary add END, RETURN). Touches
> `ff-desktop` shell command handlers only.

- [ ] DE-fix.1 Requirements gate -- function-keys-and-history/requirements.md Req 17.2/17.2a/17.4/4.3/8.2 + glossary revised; tasks 33-37 added; TCR rows (DONE for docs; awaiting code)
- [ ] DE-fix.2 Revise END-from-POM handler: close POM Workspace when others open, exit only when sole Workspace (function-keys-and-history Task 33)
- [ ] DE-fix.3 Revise RETURN-from-POM handler for consistency with END (function-keys-and-history Task 34)
- [ ] DE-fix.4 Verify Key_Label_Bar never omits blank slots; verify END/RETURN in default Excluded_Command set (function-keys-and-history Tasks 35-36)
- [ ] DE-fix.5 Update `docs/quality/TCR.md` rows for Req 17.2, 17.2a, 17.4, 4.3, 8.2 (function-keys-and-history Task 37)

---

### Phase DF -- Command Arguments and Command Chaining (CR-NR-054) -- SPEC ONLY

> Adds a general "commands accept an argument" capability: a `Command ===>` line
> is parsed into a verb + argument, and pressing a function key forwards the
> command-field contents to the bound command as its argument (type `8`, press
> F8 -> `DOWN 8`; type `LIST`, press RETRIEVE -> `RETRIEVE LIST`). Scroll commands
> gain `M`/`MAX` and line/column counts; RETRIEVE becomes argument-driven
> (empty / `LIST` numbered overlay / recall-by-number); `MENU <name> <key>` opens
> a menu and activates an option by key (== `=0.E`). Chaining is key-match only.

- [ ] DF.1 Add command-framework Requirement 9 (Command Arguments) -- DONE (spec appended)
- [ ] DF.2 Add navigation-commands Requirement 20 (Scroll Amount Arguments) -- DONE (spec appended)
- [ ] DF.3 Revise function-keys-and-history Requirement 19 (RETRIEVE argument dispatch) -- DONE (spec appended)
- [ ] DF.4 Extend menu-workspace Requirement 5 (5.5-5.6) and Requirement 11 (11.7-11.10) for MENU chaining -- DONE (spec appended)
- [ ] DF.5 Design deltas in all four sub-project design.md files -- DONE (appended)
- [ ] DF.6 Add TCR NOT COVERED rows for every new criterion (command-framework 9.1-9.10, navigation-commands 20.1-20.5, function-keys Req 19.1-19.9, menu-workspace 5.5-5.6/11.7-11.10)
- [ ] DF.7 (impl) `ff-command`: `parse_invocation` verb/arg + `arg` param at the dispatch boundary; fixed-arg on bindings
- [ ] DF.8 (impl) `ff-navigation-commands`: `parse_scroll_amount`; UP/DOWN/LEFT/RIGHT read `arg` (M/MAX/n)
- [ ] DF.9 (impl) `ff-desktop`: function-key forwarding of the command field as the argument; scroll field-clear; RETRIEVE overlay UI; MENU `<name> <key>` chaining
- [ ] DF.10 (impl) `function-keys-and-history` / `ff-keys`: `recall_list` + `recall_by_number`; RETRIEVE argument dispatch; RETRIEVE excluded from history

> NOTE: DF.7-DF.10 (code) are blocked until the CR-CH-015 (Phase DC) ff-logging
> changes compile, since ff-desktop depends on ff-logging.

---

### Phase PA-W0 -- Wave 0 Analysis Remediation (project-analysis CR-NR-056) -- PROPOSAL

> Dependency-ordered re-ordering of the incomplete Wave 0 (foundation) work found
> by the project-analysis re-baseline pass (W0.1-W0.19, commits `7330b06`..`df79cd0`
> against the CR-NR-057/CR-NR-058 specs). RECORDED per project-analysis Req 8
> (non-destructive) -- surfaced here for owner scheduling, not yet executed. Each
> item cites its incomplete-work-register ID + owning sub-project; items already
> tracked by an existing phase (DF/DH/DI/DD) are cross-referenced, not duplicated.
> Ordering rationale: the dev/debug-logging foundation FIRST (it is the highest-
> leverage bug-reporting improvement and a prerequisite for per-command
> instrumentation), then the other foundation code/behaviour defects, ready-to-
> build gated features, tracking fixes, refactors, and owner-gated proposals.

Group A -- dev/debug logging foundation (CR-NR-058, HIGHEST leverage; do first):
- [ ] PA-W0.1 (PA-CR058 / logging Req 13, `ff-logging`) Implement the build-profile
      compile-time gate: `dev-logging` cargo feature, `BUILD_PROFILE_LEVEL` const,
      cfg-split `log_trace!`/`log_debug!` (no-op in release), workspace wiring so
      debug/test builds enable it by default. Cross-ref: Phase DI Task 25. This is
      the prerequisite for PA-W0.2 and the vehicle for all dev/debug logging below.
- [ ] PA-W0.2 (PA-INCOMPLETE-004 / command Req 11, `ff-command`) Implement uniform
      per-command instrumentation at the `execute_command` boundary: start (id +
      redacted/bounded params) + completion (success/failure + result + duration)
      at DEBUG, failures at WARN/ERROR; `redact_and_bound`; shared by sync + async
      paths. Cross-ref: Phase DI Task 26. Depends on PA-W0.1. Instruments EVERY
      command invocation project-wide -- the core "log while testing/debugging".

Group B -- other foundation defects (code fixes, criteria already exist, no gate):
- [ ] PA-W0.3 (PA-INCOMPLETE-002, `ff-background-io`) Implement Req 6.6-6.9 in
      `load.rs::execute_load`: wire `RetryPolicy` from config (retry transient
      VfsError, resume-from-position), add ERROR log on I/O failure + WARN per
      retry; add the real mock-VFS retry integration test; then re-mark tasks
      9.1-9.4/9.9/13.7. HIGH priority (silent unretried failures + unlogged errors).
- [ ] PA-W0.4 (PA-LOG-004, `ff-vfs`) Add `ff-logging` dependency; replace the 3
      `eprintln!` (registry.rs:77, subsystem.rs:87/105) with `log_info!`; add
      `log_warn!` on the DuplicateScheme path to satisfy Req 3.3. Follow the
      ff-plugin / ff-core exemplar (PA-LOG-REF-001).
- [ ] PA-W0.5 (PA-LOG-005, `ff-workflow`) In `checkpoint.rs`: on deserialize/
      schema-mismatch failure emit `log_error!` + remove the invalid checkpoint
      (Req 7.6); replace `scan_resumable` `Err(_) => continue` with a `log_warn!`;
      log the `cleanup_expired` remove result.

Group C -- ready-to-build gated features (cross-ref existing phases):
- [ ] PA-W0.6 (PA-INCOMPLETE-001, command Req 9) Command Arguments = Phase DF.7-DF.10
      (already tracked); listed here for Wave 0 dependency ordering.
- [ ] PA-W0.7 (PA-INCOMPLETE-003, command Req 10) Context Navigation Stack
      (CR-NR-057) = Phase DH / Task 25 (already tracked); paired with
      command-semantics Req 11 (W3.1).

Group D -- tracking fixes (bookkeeping, no code):
- [ ] PA-W0.8 (PA-TRACK-001, configuration-system) Check Phase CQ tasks 30-31 (Req
      16 Audit / Req 17 Export/Import) -- code + TCR done, checkboxes stale.
- [ ] PA-W0.9 (PA-TRACK-002, logging-subsystem) Check tasks 23-24 + set TCR Phase DD
      Req 12.1-12.5 to PASS (tool + report verified present/deterministic).
- [ ] PA-W0.10 (PA-TCR-001, `ff-background-io`) Add TCR rows Req 1-8 AFTER PA-W0.3;
      Req 6.6-6.9 stay NOT COVERED/FAIL until then.
- [ ] PA-W0.11 (PA-DOC-001, platform-core) Reconcile Req 4.1 illustrative crate
      names OR annotate as illustrative (doc-only).
- [ ] PA-W0.12 (PA-DOC-002, configuration-system) Confirm Req 10-14 allocated
      elsewhere; add a note to the spec Introduction (doc-only).

Group E -- refactors (400-line cap, REFACTOR, no gate; do when the file is touched):
- [ ] PA-W0.13 (PA-STD-001, `ff-core`) Split `event_bus.rs` (435).
- [ ] PA-W0.14 (PA-STD-002, `ff-config`) Split 6 files over the cap (config_handle
      809, editorconfig/parser 626, reload 510, access 486, init 446, plugin_handle
      432).
- [ ] PA-W0.15 (PA-STD-003, `ff-logging`) Split `init.rs` (640).
- [ ] PA-W0.16 (PA-STD-004, `ff-plugin`) Split `registry.rs` (698).
- [ ] PA-W0.17 (PA-STD-005, `ff-workflow`) Split `runner.rs` (483) / `definition.rs`
      (463).
- [ ] PA-W0.18 (PA-STD-006, `ff-encoding`) Split `convert.rs` (536); also `ff-vfs`
      posix_provider.rs (444) / workspace.rs (410) per PA-SPLIT-002;
      `ff-document-model` document.rs (399 at cap) per PA-WATCH-002 on next touch.

Group F -- proposals requiring owner approval + own gate (NOT scheduled here):
- [ ] PA-W0.19 (PA-SPLIT-001, configuration-system) Spec split: core Req 1-9 +
      `configuration-enterprise` Req 16-18; fold Settings UI (Req 15, 18.6) into
      menu-workspace. Owner-gated.
- [ ] PA-W0.20 (PA-SPLIT-003, encoding-and-characters) Spec split: encoding-I/O
      (Req 1-5,11,14) + new `character-classification` (Req 6,7,12,13). Owner-gated.
- [ ] PA-W0.21 (PA-WATCH-004, `ff-workflow`) Decide checkpoint I/O: route through
      VFS OR document an explicit FFW-ARCH-001 exception. Owner-gated.
- [ ] PA-W0.22 (PA-LOG-002 + PA-LOG-003, project-wide) After PA-W0.1/PA-W0.2 land,
      decide which of the 56 zero-log crates still need manual instrumentation
      (many are covered automatically by per-command instrumentation); add
      remaining dev diagnostics via the `dev-logging` convention. PA-LOG-003
      (`ff-document-model`) folds in here.
- [ ] PA-W0.23 (PA-LOG-001, project-wide) One code-mode pass replacing non-ASCII in
      `.rs` with ASCII, EXCLUDING genuine Unicode test/data literals (ff-encoding,
      ff-document-model).

> Consistency watches carried to later waves (no Wave 0 action): PA-WATCH-001
> (Command_Target + Context-Navigation-Stack fan-out, Waves 3-5), PA-WATCH-003
> (workspace-backup manifest vs dataset-catalog, Wave 2), PA-WATCH-005
> (ff-encoding Unicode-data generation, Wave 6).

### Phase PA-W1 -- Wave 1 Analysis Remediation (project-analysis CR-NR-056) -- PROPOSAL

> Dependency-ordered re-ordering of the incomplete Wave 1 (editor-core) work found
> by the project-analysis re-baseline pass (W1.1-W1.15, commits `fea70d6`..`ec744e2`).
> RECORDED per project-analysis Req 8 (non-destructive) -- surfaced for owner
> scheduling, not executed. Each item cites its incomplete-work-register ID +
> owning sub-project. Wave 1 outcome: 15 sub-projects analysed, ALL tracking-complete
> except auto-indentation (PA-INCOMPLETE-006, mandated logging stubbed). NO spec
> split forced (all 15 cohesive). Three model crates (sequence-numbers,
> whitespace-guides, auto-indentation) stay OUT of PA-CONFLICT-002 by returning
> decision data and letting the caller wrap the transaction. Ordering rationale:
> the mandated-logging + dev-logging items first (CR-NR-058 leverage), then the new
> cross-unit conflict, then refactors and bookkeeping.

Group A -- mandated logging + CR-NR-058 dev-logging (depends on Phase PA-W0.1 gate):
- [ ] PA-W1.1 (PA-INCOMPLETE-006, `ff-auto-indent`) HIGH. Implement the two MANDATED
      logs currently stubbed while tasks read 137/137 `[x]`: Req 9.7 invalid-regex
      WARN (uncomment + wire `patterns.rs:47`); Req 10.7 per-decision DEBUG record
      (ref line + matched pattern + resulting indent level) in the decision/service
      path, GATED behind `dev-logging` so it is release-stripped. Re-open the
      wrongly-`[x]` tasks. Clearest Wave-1 case of CR-NR-058 dev-logging as a written
      acceptance criterion. Depends on PA-W0.1.
- [ ] PA-W1.2 (PA-LOG-010, `ff-wrap`) MEDIUM. Resolve the mandated config-warning gap
      (Req 4.7/5.8/11.3/12.2 say "via the logging-subsystem" but the crate has no
      ff-logging dep): add ff-logging + emit the warnings here OR make caller-logging
      explicit in the spec; plus dev-logging for WRAP command state changes.
- [ ] PA-W1.3 (PA-LOG-011, `ff-text-decorations`) LOW-MED. Wire ff-logging (dep is
      DEAD) to emit the Req 15.8 mandated WARN on invalid theme value; add
      dev-logging edit-sync/allocation-exhaustion trace under the `dev-logging` gate.
- [ ] PA-W1.4 (PA-LOG-006/007/008/009, seqnum/syntax/exclude/whitespace) LOW. Per the
      Group-F PA-W0.22 pattern: after PA-W0.1/PA-W0.2 land, decide dev-logging for
      these pure models. PA-LOG-006 (ff-seqnum: Req 1.4/2.8 WARN + command trace),
      PA-LOG-007 (ff-syntax-highlighting: Req 10.7 DEBUG), PA-LOG-008 (ff-exclude-show-filter:
      dead dep -> EXCLUDE/SHOW/RESET dev trace), PA-LOG-009 (ff-whitespace-guides:
      dead dep -> config-coercion WARN + toggle trace). Drop dead deps or wire.

Group B -- new cross-unit conflict (owner decision + code):
- [ ] PA-W1.5 (PA-CONFLICT-004, `ff-wrap` / `ff-whitespace-guides`) HIGH, owner-gated.
      `WrapIndentMode` + wrap-visual-flag types are DUPLICATED and unbridged across
      the two crates (visual-flag shapes even diverge: enum vs bitfield), plus two
      config surfaces (`[view.wrap]` vs `editor.wrap_*`). Make whitespace-and-guides
      the SOLE owner (its Req 6-7 + line-wrap Req 10.7 point that way); ff-wrap
      consumes. Reconcile shape + unify config keys. Code change + owner decision.

Group C -- refactors (400-line cap, REFACTOR, no gate; do when the file is touched):
- [ ] PA-W1.6 (PA-STD-014, `ff-syntax-highlighting`) Split `engine/highlight_engine.rs`
      (462); `hilite.rs` (387) is a WATCH.
- [ ] PA-W1.7 (PA-STD-016, `ff-exclude-show-filter`) Split `exclusion_engine.rs` (535).
- [ ] PA-W1.8 (PA-STD-021, `ff-text-decorations`) Split `run_styles.rs` (410).

Group D -- ASCII source cleanup (REFACTOR; several include NON-ASCII IN RUNTIME STRINGS):
- [ ] PA-W1.9 (PA-STD-013/015/017/018/019/020/022) One code-mode pass replacing
      non-ASCII in Wave-1 `.rs` with ASCII. PRIORITY sub-set: NON-ASCII CHARS INSIDE
      RUNTIME `#[error]`/warning strings are genuine output defects, not just comment
      style -- ff-exclude-show-filter (error.rs:18), ff-whitespace-guides (error.rs
      12-59, 7 msgs), ff-auto-indent (error.rs:12), ff-wrap (config.rs + error.rs,
      ~8 msgs), ff-text-decorations (error.rs:26,30). Fix these first; then the
      doc-comment em/en-dash + box-drawing banners in ff-seqnum (PA-STD-013) and
      ff-syntax-highlighting (PA-STD-015). Fold into the PA-W0.23 project-wide pass.

Group E -- TCR coverage enumeration (bookkeeping, no code):
- [ ] PA-W1.10 (PA-TCR-005/006/009) THREE TOTAL-ABSENCE crates (0 TCR rows):
      ff-exclude-show-filter (10 reqs), ff-whitespace-guides (9 reqs),
      ff-text-decorations (15 reqs). Add per-requirement TCR rows citing existing tests.
- [ ] PA-W1.11 (PA-TCR-003/004/007/008) THIN-coverage crates: ff-seqnum (3 rows/14),
      ff-syntax-highlighting (Req 16 only/16), ff-auto-indent (1 row/10 -- add Req
      9.7/10.7 AFTER PA-W1.1), ff-wrap (1 row/13). Enumerate per-requirement rows.

Group F -- doc-only naming reconciliation:
- [ ] PA-W1.12 (PA-DOC naming) Align spec crate names with actual dirs:
      sequence-numbers spec says `ff-sequence-numbers` (actual `ff-seqnum`);
      line-wrap-toggle spec says `ff-line-wrap-toggle` (actual `ff-wrap`);
      exclude-show-filter earlier summary said `ff-filter` (actual
      `ff-exclude-show-filter`). Doc-only.

> Wave-1 consistency CONFIRMATIONS (no action -- resolved during analysis):
> PA-CONFLICT-002 clean seams (seqnum/whitespace/auto-indent return decision data);
> fold-level ownership (syntax-highlighting owns, display-line-mapping consumes,
> W1.10); SHOW-restore (exclude-show-filter owns semantics, consumes canonical
> DisplayLineMapping -- positive counter-example to PA-CONFLICT-001, W1.11);
> wrap-inactive->no-markers gate (W1.14); syntax-highlighting/text-decorations peer
> boundary (independent RunStyles, W1.15). Carried WATCHES: PA-WATCH-009 (HILITE
> delegation edit-ops->syntax->find, Wave 3/4), PA-WATCH-010 (exclusion+folding
> shared visibility bit, Wave 4).

### Phase PA-W2 -- Wave 2 Analysis Remediation (project-analysis CR-NR-056) -- PROPOSAL

> Dependency-ordered re-ordering of the incomplete Wave 2 (catalog/dataset) work
> found by the project-analysis pass (W2.1-W2.8, commits `a78ff62`..`337e096`).
> RECORDED per project-analysis Req 8 (non-destructive) -- surfaced for owner
> scheduling, not executed. Wave 2 outcome: 7 sub-projects analysed, ALL
> tracking-complete; NO functional PA-INCOMPLETE. The cluster is ADR-001-governed
> with a real fitness function (`ff-governance-tests`) -- one of the best-enforced
> areas. Dominant theme: DOMAIN-TYPE FRAGMENTATION -- specs say crates should share
> a single owner but each redefines the type (DSN, VSAM, field-model x3). Ordering:
> the ADR-aligned structural conflicts first (they unblock the split), then the
> catalog-domain logging, then refactors and bookkeeping.

Group A -- ADR-001 structural conflicts + the big split (owner-gated, HIGH; do first):
- [ ] PA-W2.1 (PA-SPLIT-008 + PA-CONFLICT-007, `ff-dscatalog` / `ff-vsam-services`)
      Migrate the VSAM implementation (storage/ esds/rrds/isam/sqlite_record 843/native)
      OUT of ff-dscatalog INTO the already-existing (trait-only) ff-vsam-services,
      behind its `VsamService` trait; ff-dscatalog then deps ff-vsam-services
      (Req 7.1 direction). Also extract record codecs to `ff-record-codec`
      (Req 17.1 mandates independence). This is the PA-SPLIT-008 catalog split =
      ADR-001 Req 5 compliance. Resolves 5 of the 9 PA-STD-023 over-cap files.
- [ ] PA-W2.2 (PA-CONFLICT-006, `ff-dsalloc` / `ff-dscatalog`) Route DSN naming
      validation through `ff-dscatalog::validate_dsn` (add to the CatalogProvider/
      CatalogService trait); delete ff-dsalloc's own `DatasetName::parse` OR
      document it as a permitted pre-parse convenience matching catalog rules.
      Consolidate the 2-3 DSN parsers under the catalog (ADR-001 Req 3.1).
- [ ] PA-W2.3 (PA-CONFLICT-008 + extension, `ff-forge` / `ff-structure-catalog` /
      `ff-select`) Unify the record-structure/field-type model under ff-forge
      (fileforge-integration): ONE `FieldDefinition`/`RecordStructure`/field-type
      enum owned by ff-forge; structure-catalog adds only the `.ffs` catalog
      library layer; ff-select consumes ff-forge's field-type + COMP-3 decode
      (it currently has 3rd parallel enum + no ff-forge dep). Confirm exact
      divergence at fileforge-integration (Wave 5). Owner-gated.
- [ ] PA-W2.4 (PA-CONFLICT-005, `ff-desktop` / `ff-vfs`) Delete the duplicate,
      MISPLACED `ff-desktop/src/posix_provider.rs` (a VfsProvider in the shell --
      layering violation); keep the ff-vfs one; VCM UI consumes it via the VFS
      registry.

Group B -- catalog-domain logging (dead deps + mandated WARN/INFO; CR-NR-058, depends on PA-W0.1):
- [ ] PA-W2.5 (PA-LOG-012, `ff-dscatalog`) HIGHEST-value Wave-2 logging: wire
      ff-logging (dead dep) across the disk/SQLite/TRANSACTIONAL paths -- ERROR on
      I/O + DB failures, WARN on GDG roll-off/reconcile/import-mismatch, dev-logging
      DEBUG on command start/params/result (staged txns Req 25, integrity Req 26,
      mount Req 5.5, import Req 6.7, codec Req 16.7).
- [ ] PA-W2.6 (PA-LOG-013, `ff-desktop` VCM) Wire ff-logging on destructive catalog/
      dataset ops (recursive file delete Req 4.5, allocate/delete, POSIX errors) +
      dev-logging dialog dispatch. Consolidate with PA-W2.5 (same domain).
- [ ] PA-W2.7 (PA-LOG-015, `ff-structure-catalog`) Wire the FOUR mandated logs
      (Req 1.5 INFO dir-create; 1.6/2.5/2.6 WARN inaccessible-location / invalid
      TOML / schema-fail) + dev-logging command dispatch.
- [ ] PA-W2.8 (PA-LOG-016, `ff-select`) Wire the Req 9.9 corrupt-store WARN +
      config-coercion warnings + `.criteria.json` load failures + dev-logging
      CRITERIA dispatch (ff-logging is the DEAD sole dep).
- [ ] PA-W2.9 (PA-LOG-014 + PA-LOG-017, `ff-dsalloc` + `ff-tabmask`) LOW: reconcile
      ff-dsalloc's ff-logging spec-vs-impl drift (spec lists dep, crate lacks it) --
      add dev-logging on RESOLVE pipeline OR correct the spec; drop/justify
      ff-tabmask's dead sole-dep + optional config-coercion WARN.

Group C -- architecture watches to verify (mostly downstream waves):
- [ ] PA-W2.10 (PA-WATCH-015, `ff-select`) Route the Criteria_Store through ff-config
      (spec Req 9.1 says config-managed; impl uses raw std::fs + no ff-config dep);
      `.criteria.json` through ff-vfs per FFW-ARCH-001. Contained (2 fs sites).
- [ ] PA-W2.11 (PA-WATCH-013, `ff-governance-tests`) Extend architecture_compliance.rs
      to assert prohibited deps for `ff-dscatalog` (impl) + `ff-dsalloc`, not only the
      `ff-dataset-catalog` interface crate (fitness-function coverage gap).
- [ ] PA-W2.12 (PA-WATCH-011 remaining, ff-idcams) Verify ff-idcams `idcams.listcat`
      delegates to the catalog query API (Wave 5). CRUD half already RESOLVED (W2.3).
- [ ] PA-W2.13 (PA-WATCH-012 + PA-WATCH-014, config prefixes) Confirm `[virtual_catalogs]`
      (VCM) vs `[catalog].mounted_catalogs` (ff-dscatalog) are not competing
      persistence stores; confirm `catalog.*` (structure-catalog) vs `[catalog]`
      (dataset-catalog) config prefixes do not collide. Wave 3 (startup/config).
- [ ] PA-W2.14 (PA-WATCH-016, Display_Artifact_Line) Consider a shared synthetic-
      display-line abstraction (viewport/display-line-mapping owned) unifying COLS/
      BNDS/Placeholder/TABS/MASK. Wave 4. Not blocking.

Group D -- refactors (400-line cap, REFACTOR; several resolved by PA-W2.1 split):
- [ ] PA-W2.15 (PA-STD-023, `ff-dscatalog`) 9 files over cap; ~5 resolved by the
      PA-W2.1 VSAM/codec split. Residual concern-splits: catalog.rs 609,
      transactions.rs 503, integrity.rs 501, gdg.rs 459, dsn.rs 454.
- [ ] PA-W2.16 (PA-STD-025, `ff-desktop` VCM) SEVERE: split files_panel.rs (1195),
      file_explorer_panel.rs (935), catalog_manager_dialog.rs (645),
      dataset_alloc_dialog.rs (419) per the ff-desktop `_state/_render/_commands/
      _dialogs` shell layout. Worst cap violations in the analysis.
- [ ] PA-W2.17 (PA-STD-027, `ff-dsalloc`) Split pipeline.rs (412) by stage.

Group E -- ASCII source cleanup (REFACTOR; runtime-string defects first):
- [ ] PA-W2.18 (PA-STD-024/026/028/030/031 + 029) One code-mode pass. PRIORITY:
      NON-ASCII IN RUNTIME strings -- ff-dscatalog context-menu ellipsis (PA-STD-024),
      ff-desktop posix_provider BOM/MOJIBAKE (PA-STD-026, worst), ff-dsalloc/ff-select/
      ff-tabmask `#[error]`+config-warning em-dashes (028/030/031). Then comment-only
      ff-structure-catalog (PA-STD-029, 206 em-dashes + box-drawing). Fold into the
      PA-W0.23/PA-W1.9 project-wide ASCII sweep.

Group F -- TCR + doc bookkeeping (no code):
- [ ] PA-W2.19 (PA-TCR-010/011/012/013) Enumerate per-requirement TCR rows: ff-dsalloc
      (1/16), ff-structure-catalog (0/15 -- TOTAL absence), ff-select (1/14),
      ff-tabmask (1/18). Contrast ff-dscatalog's exemplary 99 rows.
- [ ] PA-W2.20 (PA-DEP-002 RECLASSIFIED + naming drift) DOC-only: clarify
      ff-dataset-catalog = shared-interface crate (CatalogService), ff-dscatalog =
      impl (do NOT remove); reconcile crate-name drift across the cluster specs
      (ff-dataset-catalog->ff-dscatalog, ff-dataset-allocator->ff-dsalloc,
      ff-criteria->ff-select, ff-tabs-and-mask->ff-tabmask, ff-fileforge->ff-forge);
      note that several specs list trait-injected integrations as "dependencies".

> Wave-2 consistency CONFIRMATIONS (no action -- resolved during analysis):
> ADR-001 is enforced by a real fitness function (ff-governance-tests, Req 18);
> ff-dsalloc CRUD delegation is CLEAN (CatalogProvider trait, no dup -- PA-WATCH-011
> CRUD half resolved W2.3); ff-dscatalog's 49 fs calls are LEGITIMATE (it IS the
> catalog VfsProvider + NativeFileProvider, not an FFW-ARCH-001 violation);
> ff-dscatalog TCR (99 rows) is the analysis exemplar; PA-DEP-002 WITHDRAWN
> (ff-dataset-catalog is the intentional interface crate). Minimal-dep + injection-
> trait design (ff-select, ff-tabmask) is a clean recurring seam.

### Phase PA-W3 -- Wave 3 Analysis Remediation (project-analysis CR-NR-056) -- PROPOSAL

> Dependency-ordered re-ordering of the incomplete Wave 3 (shell/commands/menus/
> session) work found by the project-analysis pass (W3.1-W3.11, commits
> `0cf0afb`..`263525c`). RECORDED per project-analysis Req 8 (non-destructive) --
> surfaced for owner scheduling, not executed. Wave 3 outcome: 10 sub-projects
> analysed; command/menu family shows CLEAN Command_Target routing + two POSITIVE
> exemplars (shell-command enforced shell.mode gate + real logging; command-completion
> injection-trait consumer). Dominant theme: the CR-NR-057 COMMAND-CHAINING BUNDLE --
> three interdependent UNBUILT legs + an END/RETURN revision, all honestly tracked.
> Ordering: the CR-NR-057 bundle first (highest-value, interdependent), then the
> two high-value logging gaps, then the shell refactors, watches, and bookkeeping.

Group A -- CR-NR-057 command-chaining bundle (HIGH, interdependent; ONE Phase DH):
- [ ] PA-W3.1 (PA-INCOMPLETE-003 + PA-INCOMPLETE-007 + PA-INCOMPLETE-008 + PA-WATCH-017)
      Implement the CR-NR-057 command chaining as ONE coordinated bundle:
      (a) command-framework Req 10 Context Navigation Stack (STOP `.`/PUSH `;`
      behaviour) = existing Phase DH (PA-INCOMPLETE-003);
      (b) command-semantics Req 11 = `chain.rs` split_chain + execute_chain +
      max_chain_length + fail-stop (Task 28, PA-INCOMPLETE-007);
      (c) menu-workspace Req 5 separator-aware Chained_Path = Vec<PathStep> +
      Navigation_Origin + PathStep dispatch + MENU chained form (Task 21 + DF.1-5,
      PA-INCOMPLETE-008);
      (d) the SHARED `split_chain`/navigation-activation helper (PA-WATCH-017) --
      implement ONCE in command-semantics; menu fastpath consumes it (tasks 28.6/DF.3).
      Req 11.5 makes (b)/(c) depend on (a). Build (a) then (b)+(c)+(d) together.
- [ ] PA-W3.2 (PA-INCOMPLETE-009, `ff-keys`) Finish function-keys Req 17 (END/RETURN-
      from-POM handler revision -- close Workspace vs exit; coordinate with the
      PA-W3.1 POM/nav model) + Req 19 (RETRIEVE recall API + History_List overlay,
      DF.1-6; LIST logic already exists) + verification/TCR (35/36/37). MEDIUM.

Group B -- high-value operational logging (dead deps on critical subsystems; CR-NR-058):
- [ ] PA-W3.3 (PA-LOG-026, `ff-session`) MEDIUM-HIGH. Wire ff-logging across the
      FAULT-TOLERANCE paths: WARN on each skipped/corrupt/missing file (Req 11
      graceful degradation), ERROR on unrecoverable, INFO on startup milestones +
      crash-recovery (Req 10). A silently-degrading startup is undebuggable. 2nd only
      to ff-dscatalog PA-LOG-012. Includes workspace-file load/save WARN.
- [ ] PA-W3.4 (PA-LOG-018, `ff-command-semantics`) MEDIUM. Command-pipeline
      dev-logging: bulk covered by PA-W0.2 (execute_command instrumentation); add
      scope-resolution + Req 11 chain-step + error-path WARN under `dev-logging`.
- [ ] PA-W3.5 (PA-LOG-024 + PA-LOG-023 + PA-LOG-019 + PA-LOG-021 + PA-LOG-025 +
      PA-LOG-020 + PA-LOG-022) LOW/refinement cluster: ff-keys history/config errors
      (024); ff-menu recent-files WARN + dead dep (023); ff-completion Req 10.5
      provider-failure catch (019); command-configurator store events (021);
      ff-shell audit gate-denial logging (025, already logs); command-palette
      optional (020); menu-workspace dispatch dev-logging (022, already logs).
      Resolve dead ff-logging deps or wire dev-logging per unit.

Group C -- shell/POM refactors (400-line cap, REFACTOR; largest in the codebase):
- [ ] PA-W3.6 (PA-STD-042, `ff-desktop` shell/) SEVERE: split the WorkbenchShell
      coordinator per state/render/commands/dialogs -- commands.rs 1255, update.rs
      1011, mod.rs 922, render.rs 873, render_chrome.rs 580. Worst caps in codebase
      (with VCM PA-STD-025 files_panel 1195).
- [ ] PA-W3.7 (PA-STD-035 + PA-STD-038 + PA-STD-040 + PA-STD-032 + PA-STD-033)
      Other over-cap files: primary_option_menu.rs 537 (menu-workspace),
      engine.rs 523 + emulator.rs 535 (ff-shell), session_state.rs 529 (ff-session),
      tso.rs 447 (cmd-semantics), engine.rs 478 (ff-completion). REFACTOR, no gate.

Group D -- ASCII source cleanup (REFACTOR; runtime-string defects first):
- [ ] PA-W3.8 (PA-STD-036/037/039/041 + 034) One code-mode pass. PRIORITY: runtime
      `#[error]`/warning-string em-dashes -- ff-menu (036), ff-keys (037), ff-shell
      (039), ff-session (041). Then comment-only ff-completion (034). Fold into the
      PA-W0.23 project-wide ASCII sweep.

Group E -- owner-gated splits + fuzzy de-duplication:
- [ ] PA-W3.9 (PA-CONFLICT-009, ff-completion / ff-desktop palette) Extract a shared
      fuzzy-match engine (`ff-fuzzy` or expose ff-completion's) consumed by both
      command-completion and command-palette (two diverged fuzzy matchers). LOW-MED.
- [ ] PA-W3.10 (PA-SPLIT-011, `ff-session`) MEDIUM: split session-state + workspace-
      model (persistence) from startup-lifecycle. PA-SPLIT-009 (ff-keys function-keys
      vs command-history) + PA-SPLIT-010 (ff-shell emulator) are LOW, deferred.

Group F -- architecture watches (mostly Wave 4/5 verification):
- [ ] PA-W3.11 (PA-WATCH-020 + PA-WATCH-023) SPEC scope-creep: menu-and-statusbar
      Reqs 17-19 (tab chrome/detach/split) + ff-session Reqs 13/14/19/20 (POM/File-
      Explorer/TSO) belong to Wave-4 layout/tab units functionally. At Wave 4,
      relocate/cross-ref the requirements to their owning specs.
- [ ] PA-W3.12 (PA-WATCH-012 REFINED + PA-WATCH-024) Confirm the 3-layer catalog-
      persistence restore ordering (ff-session tab descriptor -> VCM registry ->
      ff-dscatalog mount); confirm Command Palette + Global Search consume the
      WorkspaceState root set (Wave 5).
- [ ] PA-W3.13 (PA-WATCH-021 + PA-WATCH-022 + PA-WATCH-019) UI/pattern consistency:
      4 command-field overlays share positioning/keyboard-nav (021, Wave 4);
      `TSO` verb disambiguation ff-shell vs cmd-semantics (022); shared raw-TOML
      store loader for the menus/ family (menus/commands.toml/criteria/history/
      recent-files/session/workspace, 019).

Group G -- TCR + tracking bookkeeping (no code):
- [ ] PA-W3.14 (PA-TCR-014/015/016/017) Enumerate/extend TCR rows: ff-completion
      (1/10), command-palette (0/5), ff-menu (2/16), workspace-model (0/6). Contrast
      ff-command-semantics (39), ff-shell (25), menu-workspace (63), ff-keys (33).
- [ ] PA-W3.15 (PA-TRACK-003 + PA-TRACK-004) Bookkeeping: command-configurator
      6.2/6.3 (TCR + project-master); menu-workspace 1.3/8.1/8.2/21.5. Verify +
      check off; NOT feature work.

> Wave-3 consistency CONFIRMATIONS (no action -- resolved during analysis):
> shell-command shell.mode security gate ENFORCED at engine entry (PA-WATCH-018
> RESOLVED, exemplary; configurator inherits it); command-completion is a clean
> injection-trait consumer (zero duplication, 0 fs); command-semantics NOT in the
> HILITE chain (PA-WATCH-009 narrowed); Command_Target routing clean across the
> menu/command family (menu-workspace/command-configurator/menu-and-statusbar/
> function-keys); PA-WATCH-012 REFINED to a clean 3-layer catalog persistence;
> ff-shell + menu-workspace ACTUALLY LOG (not dead deps). Carried: PA-WATCH-020/023
> (spec scope-creep, Wave 4), PA-WATCH-021/024 (Wave 4/5 UI + unblock).

---

### Phase PA-W4 -- Wave 4 Analysis Remediation (project-analysis CR-NR-056) -- PROPOSAL

Wave 4 (UI, panels, layout: layout-and-docking, multi-tab-editor, file-tree-panel,
theme-and-appearance, view-zoom, hex-display, notification-system, plugin-manager-ui,
accessibility, context-help, clipboard-operations, idle-processing,
large-file-performance, external-modification, file-operations). Every unit's
tracking was COMPLETE; the findings below are PROPOSALS (no source changed during
analysis). All tasks `[ ]`. Cross-references: `docs/specs/project-analysis/units/`,
`consistency-matrix.md`, `incomplete-work-register.md`.

THE HEADLINE FINDING: FOUR complete, well-tested infrastructure crates are ORPHANS
(used by no crate) -- the dominant Wave-4 theme.

- [ ] PA-W4.1 (PA-CONFLICT-011 + PA-CONFLICT-012 + PA-CONFLICT-013 + PA-CONFLICT-014)
  ORPHAN-CRATE cluster (owner-gated, MEDIUM-HIGH). FOUR complete/tested infra crates
  used by NO crate: `ff-file-tree` (shell reimplements file-explorer inline --
  files_panel.rs 1195 + file_explorer.rs 935), `ff-idle-processing` (syntax-highlighting
  reimplements idle inline -- idle_styling.rs, same 10ms budget), `ff-large-file-performance`
  (render path neither wires nor reimplements -- 60fps>1M-line promise unrealized),
  `ff-external-mod` (spec never resolved placement; not wired to shell/document lifecycle).
  Per crate, owner-decide: REWIRE the consumer/shell onto the crate (preferred -- all are
  tested + well-decomposed) OR delete + reconcile spec. NOTE the pairing: idle-scheduler
  drives large-file layout work -- one integration effort wires BOTH. Code + owner decision.
- [ ] PA-W4.2 (PA-CONFLICT-010, multi-tab-editor / tabs-and-mask / menu-workspace) Tab-chrome
  triple-spec: reconcile the three specs that each define tab rendering/behaviour so a single
  owner drives the tab bar. Owner-gated. Code + spec.
- [ ] PA-W4.3 (PA-CONFLICT-005 + PA-SPLIT-012, file-tree-panel `ff-desktop`) Shell
  file-explorer tangle: posix-path assumption (PA-CONFLICT-005) + split the oversized
  files_panel.rs (1195) / file_explorer.rs (935) by concern. Ties PA-W4.1 (ff-file-tree
  orphan) -- ideally the split lands as the rewire onto ff-file-tree. Code (REFACTOR + fix).
- [ ] PA-W4.4 (PA-INCOMPLETE-010 + PA-INCOMPLETE-011 + PA-INCOMPLETE-012 + PA-INCOMPLETE-013)
  Partial UI surfaces: theme Req 16 (PA-INCOMPLETE-010), notification toast/bell
  (PA-INCOMPLETE-011), plugin enable/disable (PA-INCOMPLETE-012), accesskit/screen-reader
  wiring (PA-INCOMPLETE-013). Finish each UI surface; NOTE accessibility conformance also
  needs MANUAL assistive-tech testing (code presence necessary not sufficient). Code + tests.
- [ ] PA-W4.5 (PA-LOG-039 + PA-LOG-040) MEDIUM -- data-safety logging (strong CR-NR-058):
  `ff-external-mod` file-watcher (watch lifecycle, fs-event -> ChangeType classification,
  batch-coalescing, focus-gained re-check) and `ff-file-ops` persistence (save / atomic
  rename-on-write / backup / read-only / revert -- silent partial writes are dangerous).
  Both have DEAD ff-logging deps. Resolve dead deps + add dev-logging under the `dev-logging`
  gate. Code.
- [ ] PA-W4.6 (PA-LOG-027 through PA-LOG-038, excl. 039/040 in PA-W4.5) LOW dev-logging +
  dead-ff-logging cluster across Wave-4 UI crates (layout, multi-tab, file-tree, theme,
  view-zoom, hex, notification, plugin, context-help, clipboard, idle, large-file). Resolve
  dead ff-logging deps; add gated dev-logging where it aids debugging (idle/large-file
  deferred to their PA-W4.1 wiring). Code, LOW priority.
- [ ] PA-W4.7 (PA-DEP-003 + PA-DEP-004) Dead/over-declared Cargo deps (LOW):
  `ff-clipboard` declares ff-edit-operations + ff-undo-redo (0 usages -- clean-seam design
  needs neither); `ff-file-ops` declares ff-undo-redo (0 usages). Audit + prune workspace-wide
  (a cargo-machete-style unused-dependency sweep would catch the whole family, incl. the
  dead-ff-logging cluster). Code (Cargo cleanup).
- [ ] PA-W4.8 (PA-STD-043 through PA-STD-055) ASCII runtime-string cluster: replace em-dashes/
  arrows in runtime `#[error]` + status strings across Wave-4 crates (layout, multi-tab,
  theme, view-zoom, hex, context-help, clipboard, idle [+ Cargo-desc mojibake],
  large-file, external-mod, file-ops). PA-STD-042 (shell split) is already PA-W3.6.
  REFACTOR, no gate.
- [ ] PA-W4.9 (PA-DOC-006) Crate-name-drift reconciliation (docs): `ff-external-modification`
  (spec) vs `ff-external-mod` (dir). Add to the naming-reconciliation set with the other
  drifts (ff-hex/ff-select/ff-tabmask/ff-forge/ff-keys/ff-completion/ff-wrap/ff-seqnum/
  ff-dscatalog). Docs only.
- [ ] PA-W4.10 (PA-TCR-018 through PA-TCR-027) TCR enumeration for Wave-4 crates. NOTE the
  THREE orphan infra crates with TOTAL TCR absence (0 rows): ff-idle-processing (PA-TCR-024),
  ff-large-file-performance (PA-TCR-025), ff-external-mod (PA-TCR-026). Add per-requirement
  rows citing existing tests. No code.
- [ ] PA-W4.11 (PA-WATCH-021 + PA-WATCH-022 + PA-WATCH-024) Carried Wave-4 watches: resolve
  the remaining UI/pattern-consistency + unblock items observed in Waves 3-4. Confirm or
  downgrade each during Wave 5/6. Analysis + targeted code.

> Wave-4 consistency CONFIRMATIONS (no action -- resolved during analysis):
> ff-theme is a CLEAN single-owner exemplar (sole StyleSlotTable + colour groups, 0 dups --
> counter-example to Wave-2 field-model fragmentation); PA-WATCH-020 RESOLVED (menu-statusbar
> Reqs 17-19 misfiled -> ff-layout owns); PA-WATCH-025 RESOLVED (WCAG verifier + high-contrast
> producer both in ff-theme); PA-WATCH-023 RESOLVED (menu-and-statusbar scope-creep folded);
> FIVE clean-seam crates stay OUT of PA-CONFLICT-002 (seqnum, whitespace-guides, auto-indent,
> hex-display ByteReader, clipboard ClipboardEntry) -- and file-ops joins them (reads
> document.is_dirty, no transactions); EXEMPLARY VFS discipline in ff-external-mod + ff-file-ops
> (0 real fs); ff-file-ops is NOT an orphan (consumed by ff-global-search) -- breaks the streak;
> file-ops CONFIRMS ff-external-mod was built standalone (0 ff-external-mod dep) so the
> PA-CONFLICT-014 fix is wiring, not re-housing.
> CORRECTION (W5.12): the "ff-file-ops NOT an orphan (consumed by global-search)" note
> above is DECLARATION-ONLY -- global-search declares ff-file-ops but uses it 0 times
> (raw std::fs instead). See PA-W5.3 / PA-DEP-005.

---

### Phase PA-W5 -- Wave 5 Analysis Remediation (project-analysis CR-NR-056) -- PROPOSAL

Wave 5 (emulators, tools, connectors: jes-emulator, idcams-emulator, jcl-resolver,
database-tool, compiler-toolchain-integration, batch-execution, asa-report-preview,
lua-macro-engine, language-service, custom-file-viewers, compare-and-merge, global-search,
connector-extensibility, connector-local-fs, connector-network-fs/-ftp-sftp/-cloud/
-mainframe, fileforge-integration, automated-dialog-testing, bootstrap-scripts). Findings
below are PROPOSALS (no source changed during analysis). All tasks `[ ]`. Cross-refs:
`docs/specs/project-analysis/units/`, `consistency-matrix.md`, `incomplete-work-register.md`.

TWO dominant Wave-5 themes: (A) REIMPLEMENT-INSTEAD-OF-REUSE duplication, and (B)
DATA-SAFETY raw-fs write bypasses. Plus the orphan tally continues + a false-complete.

- [ ] PA-W5.1 (PA-CONFLICT-019 + PA-CONFLICT-022 + PA-CONFLICT-018) DUPLICATION cluster
  (owner-gated, MEDIUM-HIGH). Consolidate reimplemented capabilities onto single owners:
  ASA carriage control is in THREE crates (ff-asa [wired owner] + ff-viewers/asa_report.rs
  + ff-forge/asa.rs) -> consolidate onto ff-asa; EBCDIC in TWO (ff-encoding [owner] +
  ff-forge/ebcdic.rs) -> ff-forge reuse ff-encoding; hex in TWO (ff-hex + ff-viewers/hex.rs);
  language detection duplicated (ff-language-service [orphan] vs ff-syntax-highlighting
  inline) -> rewire syntax-highlighting onto ff-language-service (also its idle-styling ->
  ff-idle-processing, PA-CONFLICT-012); PREVIEW command DOUBLE-OWNER (ff-viewers + ff-asa)
  -> one owner (framework dispatches). Decide framework-vs-point per capability; delete the
  losing copies OR document intentional divergence + share low-level tables. Code + owner.
- [ ] PA-W5.2 (PA-CONFLICT-015 + PA-CONFLICT-020) DATA-SAFETY raw-fs WRITE bypasses
  (owner-gated, MEDIUM-HIGH). Route workbench-data writes through the safe path (ff-vfs /
  ff-file-ops atomic-rename + backup), not raw std::fs: JES job-queue persistence
  (queue.rs read/create_dir_all/write, no ff-vfs dep) and global-search cross-file REPLACE
  (replace.rs raw write, bulk mutation with no atomic/backup). Both risk partial/corrupt
  writes. NOTE: legitimate raw fs (ff-connector-local-fs = the fs layer; ff-fftest test
  artifacts; toolchain locating OS compilers; lua macro-dir scan PA-WATCH-026) is NOT in
  scope. Code + owner.
- [ ] PA-W5.3 (PA-DEP-005 + W4.15 correction) ff-file-ops dead dep in global-search
  (declared, 0 uses -- raw std::fs instead). Either USE ff-file-ops for the replace writes
  (preferred -- also resolves PA-W5.2's global-search half + makes the W4.15 "consumed"
  claim true) or drop the dep. Correct the W4.15 record's "consumed by global-search" note.
  Code + docs.
- [ ] PA-W5.4 (PA-CONFLICT-016) idcams-emulator orphan + MISSING DUAL integration
  (owner-gated, MEDIUM-HIGH). ff-idcams is complete but wired to nothing; spec Req 20
  designs (a) command-framework registration + (b) JES EXEC PGM=IDCAMS batch-step
  invocation -- NEITHER built (ff-jes has 0 idcams refs). Register with command framework +
  wire the JES batch-step so IDCAMS runs inside JES jobs. Code + owner.
- [ ] PA-W5.5 (PA-INCOMPLETE-014) database-tool FALSE-POSITIVE-COMPLETE (HIGH, tracking
  integrity). 157/157 tasks `[x]` but the impl is a foundation SKELETON (thiserror+serde
  only; ~14 of 17 reqs -- panels/pooling/async/ER/transfer/admin/integrations -- unbuilt).
  Re-open the unbuilt tasks as `[ ]`; keep only the foundation done; correct the "complete"
  framing. Then an owner-prioritised implementation effort. Tracking + honest re-scope.
- [ ] PA-W5.6 (PA-INCOMPLETE-015 + PA-INCOMPLETE-016) honestly-tracked open work:
  batch-execution task 10.2 (`--batch-log <file>` parsed but not wired to an ff-logging
  file sink -- small); lua-macro Macro Library panel (Req 12, task 25) + FFCMD-sequence
  chaining (tasks 26.x = ANOTHER CR-NR-057 leg -> Phase DH, reuse the shared chain
  executor). Code + tests.
- [ ] PA-W5.7 (PA-STD-057 + PA-STD-066) SEVERE / notable cap violations: idcams
  parser/mod.rs = 1372 non-test (the WORST file in the project; a mod.rs holding 41 fns)
  + handlers.rs 833 + services.rs 765 + ast.rs 538; and compare-and-merge session.rs = 557
  which IS the DiffEngine (cap + MISLEADING filename -> rename to diff_engine.rs). Split by
  concern. REFACTOR (comparable to the shell split PA-W3.6).
- [ ] PA-W5.8 (PA-STD-059/068/069/062/063/073) other 400-cap splits: toolchain plugins
  (gcc 421 + rust 462), connector-extensibility registry.rs 460, connector-local-fs
  provider.rs 542, lua-macro engine.rs 506 (+tso_builtins 431/ispf 405), language-service
  definition.rs 464, fftest parser.rs 406. Split each by concern. REFACTOR.
- [ ] PA-W5.9 (PA-LOG-041/042/046/050 + 044/047/048/049/051/052/043/045/037/038/039/040)
  Wave-5 logging. HIGH-VALUE (MEDIUM+): job engines log NOTHING (JES PA-LOG-041, idcams
  PA-LOG-042); lua-macro security-gate decisions are audit-relevant + ff-logging DEAD
  (PA-LOG-046); global-search bulk replace should log changes (PA-LOG-050); toolchain
  install steps (PA-LOG-044). LOWER: asa/compare/viewers/language-service/fileforge pure
  transforms + database-tool deferred to build. Add ff-logging + gated dev-logging per unit
  (resolve dead deps). Code.
- [ ] PA-W5.10 (PA-STD ASCII cluster: 052/053/054/055/056/058/060/061/064/065/067/070/071/072)
  runtime-string + comment ASCII across Wave-5 crates: em-dashes / multiplication signs
  (U+00D7, ff-asa) / minus signs (U+2212, compare-merge) in runtime strings; mojibake
  comment separators in .rs (ff-jes/ffjcl, database-tool, toolchain, connector-local-fs);
  Cargo-desc mojibake (idle); design.md mojibake STATUS lines (deferred connectors,
  PA-STD-071). Replace with ASCII. REFACTOR, no gate.
- [ ] PA-W5.11 (PA-TCR-024..036, excl. exemplars) TCR enumeration for Wave-5 crates. NOTE
  the TOTAL-ABSENCE crates (0 rows): idle/large-file/external-mod (W4) + language-service
  (PA-TCR-031) + connector-extensibility (PA-TCR-034); THIN: idcams 1/26 (widest by spec
  size), asa 1/12, compare 1/17, fileforge 1/16, local-fs 1/7. Add per-req rows. NOTE
  EXEMPLARS (no gap): JES 74, fftest 62, toolchain 38 -- do NOT touch. No code.
- [ ] PA-W5.12 (PA-DOC-006..010 + PA-TRACK-005/006) DOCS: crate-name-drift reconciliation
  (ff-external-modification->ff-external-mod, ff-asa-report-preview->ff-asa, ff-macro->
  ff-lua, ff-compare->ff-compare-merge, ff-fileforge->ff-forge -- extend the naming set);
  jcl-resolver stub (PA-CONFLICT-017: JCL fragmented across ff-dsalloc + ff-jes/ffjcl;
  PA-TRACK-005 readiness-summary wrongly says folded into FFW-JES Req 11); mark the 4
  DEFERRED connector specs clearly (PA-TRACK-006). Docs only.
- [ ] PA-W5.13 (PA-CONFLICT-021, DOWNGRADED to LOW-MED/WATCH) connector-extensibility is
  the INTENTIONAL base for the DEFERRED remote connectors (network-fs/ftp-sftp/cloud/
  mainframe specs state they implement its trait; local-fs uses ff-vfs directly by design)
  -- NOT an abandoned orphan. KEEP it; add an implementers-are-deferred note to its spec;
  re-confirm the trait API against real FTP/SFTP/z-OS semantics when the first remote
  connector is built (cloud OAuth + z/OS auth will test the auth layer). Docs now; code
  later.

> Wave-5 consistency CONFIRMATIONS (no action -- resolved / positive during analysis):
> POSITIVE exemplars -- compiler-toolchain-integration (genuinely complete, wired 99x,
> clean VFS test-only fs, 38 TCR), batch-execution (uses ff-logging -- the CR-NR-058
> behaviour), compare-and-merge (single diff owner, clean-seam, VFS-by-interface),
> connector-local-fs (correct primary VFS provider, 51 LEGIT fs, logs), automated-dialog-
> testing (egui-DECOUPLED AutomationId model, 62 TCR), bootstrap-scripts (cleanest unit,
> zero findings); SECURITY-GATE consistency -- lua-macro SecurityMode ENFORCED (mirrors
> shell.mode PA-WATCH-018), the two code-execution surfaces both gate untrusted code with a
> safe default; CLEAN scope boundary -- local mainframe EMULATION (dataset-catalog/JES/
> IDCAMS) vs REMOTE connectivity (deferred connector-mainframe), complementary not
> duplicative; ff-jes is a PLUGIN not an orphan (wired 19x, 74 TCR -- exemplary); TWO
> self-corrections this wave (ff-file-ops "consumed" claim -> declaration-only PA-W5.3;
> connector-extensibility orphan -> intentional deferred-base PA-W5.13).

---

## Summary

| Status | Count |
|--------|-------|
| `[x]` Complete with real tests | 62 library crates (incl. ff-global-search) + ff-desktop binary |
| `[x]` Stream 1 complete | BV, BS.1-BS.15, ff-vfs.13-16, BU.1-BU.9 |
| `[x]` Stream 2 complete | BW, BX, BY, BZ, CA, CB, CC, CD (all EARS P1) |
| `[x]` Stream 3 complete | CE, CF, CG, CH, CI (all EARS P2) |
| `[x]` Phase CJ complete | Bootstrap scripts (CJ.1-CJ.6) |
| `[x]` Phase CK complete | FFTest framework (CK.1-CK.4) |
| `[x]` Phase BS-A complete | Workspace Model (BS-A.1-BS-A.6) |
| `[x]` Phase BS-B complete | Command Palette (BS-B.1-BS-B.4) |
| `[x]` Phase BS-C complete | Global Search (BS-C.1-BS-C.5) |
| `[x]` Phase BT complete | Cross-File Replace + Search History (BT.1-BT.6) |
| `[~]` compiler-toolchain-integration | Tasks 5.1-5.3 complete -- MockToolchain test double written, trait audited, CI constraint documented (Req 5.1-5.4) |
| `[x]` Phase CU complete | Menu Workspace Pattern -- spec only (CU.1-CU.6) |
| `[x]` Phase CU-impl complete | Menu Workspace Implementation -- TabKind, loader, hot-reload, render, dispatch, defaults (Tasks 1-8) |
| `[x]` Phase CV complete | POM Redesign Spec -- 12-option list, cv-requirements.md, Req 14.3 updated (CV.1-CV.5) |
| `[x]` Phase CW complete | Settings Menu Spec -- 10-option Settings_Menu, cw-requirements.md, Req 15 updated (CW.1-CW.4) |
| `[x]` Phase CX complete | Named Workspaces + KEYS name + SPLIT alias -- spec + implementation (CX.1-CX.7) |
| `[x]` Phase CZ complete | FFTest Script Suite + Context Inspection (CZ.1-CZ.9) |
| `[x]` Phase DA complete | Configurable Menu Option Limits -- spec + impl (DA.1-DA.7, Tasks 16-20) |
| `[~]` Phase DB | Unified Command Target + Command Configurator + Descriptor Persistence -- spec (DB.1-DB.7) done. Impl: DB.8 DONE (CommandTarget), DB.11 DONE (descriptor persistence, unblocks Task 14.5), DB.9 DONE (Command_Store + resolver + Command Configurator Context UI -- command-configurator Task 4 + 6.1), DB.10 DONE (ff-shell external Detached/Captured execution), DB.4 DONE (menu-option + shortcut binding via Target_Resolution; inline `[options.target]`), MENU command DONE (menu-workspace Req 11 + Menu_Target Req 10.4), external desktop adapter DONE (command-configurator Task 3: ff-shell wired into ff-desktop, ${workspace_root}/${file_dir} expansion, shell.mode prompt-confirm dialog, External target execution). Remaining: CustomWorkspace/Macro binding execution and a dockable Output_Panel view are follow-up UI tasks |
| Sub-project audit | 67 of 69 sub-projects with tasks.md are ALL DONE; 2 have pending items |
| Test count | 759 passing (ff-desktop), 0 failures (cargo test --workspace after Phase CX) |
| `[x]` Phase DC complete | Two-Phase Logging Init (CR-CH-015, B033) -- logging-subsystem Req 11 + `ff_logging::reconfigure` + desktop `apply_logging_config` wiring (DC.1-DC.5); 9008 tests pass, verify.ps1 clean |
| `[ ]` Phase DD | Logging Inventory and Gap Report Tool (CR-NR-055) -- adds logging-subsystem Req 12 + `tools/python/logging_inventory.py` -> `docs/quality/logging-inventory.md` (DD.1-DD.4) |
| `[ ]` Phase DE-fix | END/RETURN from POM close the Workspace, exit only when sole Workspace (CR-CH-016) -- SPEC DONE (Req 17.2/17.2a/17.4/4.3/8.2), impl pending (DE-fix.1-DE-fix.5) |
| Active work | Phase CZ -- FFTest Script Suite + Context Inspection (next step) |
| `[ ]` Phase DH (spec) | Command Chaining and Context Navigation Stack (CR-NR-057) -- SPEC DONE: command-semantics Req 11 (Command_Chain parse + fail-stop sequential execute, `commands.max_chain_length` default 16), command-framework Req 10 (Context_Navigation_Stack, `=`-origin rule, `.` STOP vs `;` PUSH, END/RETURN chainable, `navigation.stack_max_depth` default 32), menu-workspace Req 5.7-5.12 (separator semantics + `=` origin in the chained-path resolver), lua-macro-engine Req 5.8-5.11 (newline == `;`, shared chain executor, no piping). Impl pending: command-semantics Task 28, command-framework Task 25, menu-workspace Task 21, lua-macro-engine Task 26 |

## Phase DI -- Build-Profile Logging + Uniform Command Instrumentation (CR-NR-058)

- [ ] DI.1 logging-subsystem Requirement 13 (compile-time build-profile level gating) -- SPEC DONE
- [ ] DI.2 command-framework Requirement 11 (uniform per-command instrumentation) -- SPEC DONE
- [ ] DI.3 (impl) `ff-logging`: `dev-logging` cargo feature, public `BUILD_PROFILE_LEVEL` const, cfg-split `log_trace!`/`log_debug!` (no-op in release), workspace debug-vs-release wiring (logging-subsystem Task 25)
  - PARTIAL (commit `4bf6fbc`): feature + const (re-exported) + cfg-split macros DONE + verified (both profiles compile, 21 doctests pass, verify.ps1 clean; Task 25.1-25.4). REMAINING: release-vs-debug wiring so release drops `dev-logging` without a manual flag (Task 25.5, currently default-on for both); Req 13 unit tests + PBT (Task 25.6/25.7); TCR Req 13 rows (Task 25.8).
- [ ] DI.4 (impl) `ff-command`: single Instrumentation_Point in `execute_command` -- start (DEBUG) + completion (DEBUG ok / WARN err) with redacted/bounded params and handler-only duration; rejection paths traced (command-framework Task 26)
- [ ] DI.5 TCR rows for logging-subsystem Req 13.1-13.8 and command-framework Req 11.1-11.9 set to their correct status

| Status | Count |
|--------|-------|
| `[ ]` Phase DI (spec) | Build-Profile Logging + Uniform Command Instrumentation (CR-NR-058) -- SPEC DONE: logging-subsystem Req 13 (compile-time `dev-logging` gate, `BUILD_PROFILE_LEVEL` const, TRACE/DEBUG stripped from release), command-framework Req 11 (uniform start/params/completion instrumentation at the `execute_command` boundary, DEBUG for start/success, WARN for failure, redaction + duration). Impl pending: logging-subsystem Task 25, command-framework Task 26 |
| `[ ]` Phase PA-W0 (PROPOSAL) | Wave 0 Analysis Remediation (project-analysis CR-NR-056, re-baselined for CR-NR-057/CR-NR-058) -- dependency-ordered Wave 0 findings (PA-W0.1-PA-W0.23). Group A: dev-logging foundation (CR-NR-058 Req 13 gate + Req 11 instrumentation = Phase DI). Group B defects: PA-INCOMPLETE-002 (ff-background-io Req 6.6-6.9), PA-LOG-004 (ff-vfs Req 3.3), PA-LOG-005 (ff-workflow Req 7.6). Group C features: command Req 9 = Phase DF, Req 10 = Phase DH. Groups D-F: tracking, refactors, owner-gated proposals. RECORDED, not executed |
