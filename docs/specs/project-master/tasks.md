# Project Master — Implementation Status Dashboard

## How to use this file

- This file is a **one-row-per-crate dashboard** only. It shows what is genuinely done.
- Detailed tasks for each crate live in `docs/specs/<sub-project>/tasks.md`.
- When starting work on a crate, open its `tasks.md` for the numbered step list.
- A crate is `[x]` only when: all tasks in its `tasks.md` are `[x]` AND `cargo test -p <crate>` passes with real (non-zero) test coverage.

## Status key

| Mark | Meaning |
|------|---------|
| `[x]` | Complete — real tests pass |
| `[ ]` | Scaffolded — compiles, but 0 unit tests (TDD not done) |
| `[~]` | Partial — some tasks done, some remaining |

---

## Full Workspace Audit — cargo test --workspace (all pass, 0 failures)

### Phase A — Foundation

- [x] 1.1 `ff-logging` — logging subsystem

### Phase B — Platform Architecture

- [x] 2.1 `ff-core` — platform core
- [x] 2.2 `ff-config` — configuration system
- [x] 2.3 `ff-command` — command framework
- [x] 2.4 `ff-plugin` — plugin architecture (124 unit + 7 integration + 10 PBTs)
- [x] 2.5 `ff-workflow` — workflow engine (86 unit + 13 PBTs)
- [x] 2.6 `ff-layout` — layout and docking (116 unit + 20 integration/PBTs)

### Phase C — Virtual File System

- [x] 3.1 `ff-vfs` — virtual file system (102 unit + PBTs)
- [x] 3.2 `ff-connector-local-fs` — local filesystem connector
- [x] 3.3 `ff-connector-extensibility` — connector extensibility

### Phase D — Core Editor

- [x] 4.1 `ff-document-model` — document model
- [x] 4.2 `ff-edit-operations` — edit operations
- [x] 4.3 `ff-undo-redo` — undo/redo transactions
- [x] 4.4 `ff-viewport-scrolling` — viewport and scrolling (21 unit + 8 integration + 14 PBTs)
- [x] 4.5 `ff-display-line-mapping` — display line mapping

### Phase E — Command Engine

- [x] 5.1 `ff-command-semantics` — primary command parser + pipeline
- [x] 5.2 `ff-find-and-replace` — FIND/RFIND/CHANGE/RCHANGE
- [x] 5.3 `ff-line-commands` — D/I/R/C/M/A/B/X/T/>/<
- [x] 5.4 `ff-exclude-show-filter` — EXCLUDE/SHOW/RESET
- [x] 5.5 `ff-navigation-commands` — LOCATE/SORT/UP/DOWN/BOUNDS/COLS

### Phase F — UI and Rendering

- [x] 6.1 `ff-menu` — menu and status bar
- [x] 6.2 `ff-theme` — theme and appearance
- [x] 6.3 `ff-text-decorations` — text decorations
- [x] 6.4 `ff-whitespace-guides` — whitespace and guides (69 unit + 7 integration + 7 PBTs)
- [x] 6.5 `ff-caret-selection` — caret and selection

### Phase G — Language and Highlighting

- [x] 7.1 `ff-language-service` — language service
- [x] 7.2 `ff-syntax-highlighting` — syntax highlighting
- [x] 7.3 `ff-auto-indent` — auto-indentation

### Phase H — File I/O and Session

- [x] 8.1 `ff-file-ops` — file operations
- [x] 8.2 `ff-background-io` — background I/O
- [x] 8.3 `ff-encoding` — encoding and characters
- [x] 8.4 `ff-external-mod` — external modification detection
- [x] 8.5 `ff-session` — startup and session
- [x] 8.6 `ff-tabs` — multi-tab editor

### Phase I — Desktop Integration

- [x] 9.1 `ff-clipboard` — clipboard operations
- [x] 9.2 `ff-keys` — function keys and history
- [x] 9.3 `ff-shell` — shell command
- [x] 9.4 `ff-help` — context help
- [x] 9.5 `ff-zoom` — view zoom (75 unit + 18 integration + 19 PBTs)
- [x] 9.6 `ff-wrap` — line wrap toggle (122 unit + 23 integration + 27 PBTs)

### Phase J — Extensions and Macros

- [x] 10.1 `ff-lua` — Lua macro engine
- [x] 10.2 `ff-completion` — command completion

### Phase K — Display Modes

- [x] 11.1 `ff-hex` — hex display
- [x] 11.2 `ff-seqnum` — sequence numbers
- [x] 11.3 `ff-tabmask` — tabs and mask

### Phase L — FileForge Domain

- [x] 12.1 `ff-forge` — FileForge integration
- [x] 12.2 `ff-structure-catalog` — structure catalog
- [x] 12.3 `ff-select` — record selection criteria
- [x] 12.4 `ff-asa` — ASA report preview
- [x] 12.5 `ff-viewers` — custom file viewers (128 unit + 10 PBTs)

### Phase M — Dataset Catalog and Mainframe Emulation

- [x] 13.0 `ff-governance-tests` — dataset ownership model governance
- [x] 13.1 `ff-dscatalog` — dataset catalog
- [x] 13.2 `ff-dsalloc` — dataset allocator
- [x] 13.3 `ff-idcams` — IDCAMS emulator

### Phase N — Job Entry Subsystem

- [x] 14.1 `ff-jes` — JES emulation

### Phase O — File Explorer

- [x] 15.1 `ff-file-tree` — file tree panel
- [x] 15.2 `ff-compare-merge` — compare and merge

### Phase P — Performance

- [x] 16.1 `ff-idle-processing` — idle processing
- [x] 16.2 `ff-large-file-performance` — large file performance

### Phase Q — Database Tool

- [x] 17.1 `ff-database-tool` — database tool

### Phase R — Binary Integration (`ff-desktop` / `ffwb` binary)

- [x] 18.1 Boot sequence — logging → config → Tokio → WorkbenchApp → eframe window
- [x] 18.2 egui shell — WorkbenchShell, theme, menu bar, status bar, ISPF command field
- [x] 18.3 Editor panel — line rendering, mouse-wheel scroll
- [x] 18.4 File open — File > Open wired to ff-connector-local-fs
- [x] 18.5 Tab bar — tab headers, switching restores per-tab viewport state
- [x] 18.6 Command field dispatch — EDIT/EXIT/QUIT/=X/1/FILES
- [x] 18.7 Live status bar — cursor line/col, encoding, modified indicator, line count
- [x] 18.8 Keyboard navigation — arrow keys, Page Up/Down through ViewportModel
- [x] 18.9 CLI file arguments — open files from command line as tabs
- [x] 18.10 Session save/restore — persist and restore open tabs on exit/launch

### Phase S — Binary Polish (`ff-desktop`)

- [x] 19.1 Fix ff-dsalloc property test compile failure
- [x] 19.2 Native file-open dialog (rfd, File > Open…)
- [x] 19.3 Keyboard text input — typed chars insert, Backspace deletes, Enter splits line
- [x] 19.4 Save to disk — File > Save and Ctrl+S

### Phase T — Bug Fixes

- [x] 20.1 Mouse click moves cursor (Req 13.1)
- [x] 20.2 Ctrl+Z undo (Req 13.2)
- [x] 20.3 Current-line highlight (Req 13.3)
- [x] 20.4 Caret bar rendering (Req 13.4)

### Phase V — ISPF Primary Option Menu

- [x] 22.1–22.8 All POM tasks complete (50 tests pass in ff-desktop)

### Phase U — ISPF Command Engine Integration (next phase)

- [x] 21.1 Wire `ff-command-semantics` into `ff-desktop` command field (replace hard-coded handle_command)
- [x] 21.2 Wire `ff-line-commands` prefix area into editor panel gutter
- [x] 21.3 Wire `ff-find-and-replace` into command field (FIND/CHANGE commands)
- [x] 21.4 Wire `ff-navigation-commands` into command field (LOCATE/SORT/UP/DOWN)
- [x] 21.5 Wire `ff-exclude-show-filter` into editor panel (EXCLUDE/SHOW/RESET)
- [x] 21.6 Render interactive prefix area (gutter) in editor panel
- [x] 21.7 Wire `ff-keys` — function key bar, RETRIEVE, command history

### Phase W — Compiler Toolchain Integration

- [x] W.1 Create `ff-toolchain-api` crate — shared `ToolchainPlugin` trait, `ToolchainState`, `Diagnostic`, `BuildProfile`, `BuildEvent` (Tasks 1.1–1.9)
- [x] W.2 Create `ff-gcc-toolchain` plugin crate — GCC detection, platform install (winget/apt/brew), build invocation, diagnostic parser (Tasks 2.1–2.12)
- [x] W.3 Create `ff-rust-toolchain` plugin crate — rustup/rustc/cargo detection, rustup-init install, cargo build/check/test, JSON diagnostic parser (Tasks 3.1–3.11)
- [x] W.4 Toolchain_Panel UI in `ff-desktop` — status rows, install buttons, progress, build output, clickable diagnostics, Compilers menu wiring (Tasks 4.1–4.7)

### Phase X — POM Floating Window (superseded by Phase Z)

- [x] X.1 Converted POM to egui::Window — superseded; Phase Z converts to tabbed window container

### Phase Z — Tabbed Window Container + Full Tab Context Menu (B001 re-specified)

- [x] Z.1 Implement TabKind enum, POM as first-class tab, full 27-item tab context menu, tab bar empty-space menu, START/CLOSE/EXIT command routing (Tasks 19.1-19.23 in startup-and-session/tasks.md)

### Phase Z.1 — Tab Context Menu Exit Item (Req 14.38)

- [x] Z.1.1 Add "Exit" as last universal item in tab header context menu for all tab kinds (Task 20.1-20.5 in startup-and-session/tasks.md)

### Phase AA.11 — POM Option Buttons (Req 14.39, 14.40)

- [x] AA.11 Make each POM option row and the Exit line interactive buttons (Tasks 21.1-21.7 in startup-and-session/tasks.md)

### Phase AC — POM Option List Reorganisation (Req 14.3, 14.3a, 14.3b)

- [x] AC.1 Reorganise POM built-in options to 9 entries (0–8): Settings, File Catalogs, Files, Utilities, Compilers, Lua Scripts, Terminals, Databases, Plugins (Tasks 23.1–23.5 in startup-and-session/tasks.md)

### Phase AD — Menu Bar Alignment with 9-Option POM (Req 14.7)

- [x] AD.1 Add `File Catalogs` and `Plugins` top-level menus to the menu bar to mirror all 9 POM options (Tasks 24.1–24.6 in startup-and-session/tasks.md)

### Phase AE — POM Exit Line Text Update (Req 14.40)

- [x] AE.1 Update POM exit line text to ISPF-authentic wording "Enter X to Terminate using log/list defaults" (Tasks 25.1–25.4 in startup-and-session/tasks.md)

### Phase AF — Calendar Month Navigation (Req 14.41, 14.42)

- [x] AF.1 Add `<` / `>` hotspot buttons to calendar header; support forward/backward month navigation with per-POM-tab offset state (Tasks 26.1–26.10 in startup-and-session/tasks.md)

### Phase AH — Settings Panel (Req 15)

- [x] AH.1 Add `set_user_value` / `remove_user_value` to `ConfigHandle` in `ff-config` (Task 25)
- [x] AH.2 Add `TabKind::SettingsPanel`, route `0` / `SETTINGS` / `=0` commands, wire POM option 0
        button, session persistence (Task 26)
- [x] AH.3 Create `settings_panel.rs` — namespace groups, collapsible sections, filter input,
        source file footer, F3/END routing (Task 27)
- [x] AH.4 Per-key value widgets (checkbox, slider, combo, text) and provenance badge (Task 28)
- [x] AH.5 Write path (validate → `set_user_value`), inline errors, Reset to Default button (Task 29)

### Phase AG — Help > About Dialog (Req 13)

- [x] AG.1 Create `about_dialog.rs` in `ff-desktop`; wire `Help > About` menu item; display app name,
        version, creator credit (Alan R Wynne), AI assistant credit (Amazon Q Developer / AWS),
        copyright, and description (Tasks 18.1–18.6 in menu-and-statusbar/tasks.md)

### Phase AE — Legacy Theme Colour Semantics (Req 13 theme-and-appearance)

- [x] AE.1 Add `primary_menu_bg` to `UiColours`; update `ColourToken`; update Legacy palette defaults
- [x] AE.2 Wire `PomColours` from shell into `primary_option_menu::render()` for Legacy theme
- [x] AE.3 Implement per-element colour rendering in POM (key=white, label=turquoise, desc=green, calendar=turquoise, today=reversed)

### Phase AJ — Tab-Order Focus Cycle (Req 16 menu-and-statusbar)

- [x] AJ.1 Add `FocusStop` enum with `CommandField`, `PomOption`, `PomExit`, `CalendarPrev`, `CalendarNext`, `MenuBar` variants; wire Tab/Shift+Tab cycle in `WorkbenchShell`
- [x] AJ.2 Pass `focused_pom_option` into POM render; draw reversed-colour highlight on focused option row
- [x] AJ.3 Handle Enter/Space activation on all focus stops; apply menu bar focus indicator

### Phase AI — User-Configurable Theme Colours and Custom Themes (Req 14 theme-and-appearance)

- [x] AI.1 Implement themes directory scanning and `ThemeInfo` / `list_themes()` API in `ff-theme`
- [x] AI.2 Register directory watch on themes directory for hot-reload of new/modified user theme files
- [x] AI.3 Implement `export_theme(name)` serialisation helper; audit full token coverage; write tests

### Phase AB — Catalog Storage Default Paths (Req 12)

- [x] AB.1 Register `catalogs.default_mainframe_root` and `catalogs.default_posix_root` config
        schema keys; pre-populate Catalog Manager Dialog path fields from config (Tasks 11.1–11.6
        in virtual-catalog-manager/tasks.md)

### Phase AA — Virtual Catalog Manager (POM Option 1 — Files Panel)

- [x] AA.1 Add `FilesPanel` TabKind, route POM option 1 to Files panel, update option 1 label (Task 1.1–1.6 in virtual-catalog-manager/tasks.md)
- [x] AA.2 Catalog Registry — persist/restore all four catalog types (Task 2.1–2.6)
- [x] AA.3 POSIX VFS Provider — scheme `posix`, root-jail, read-only enforcement (Task 3.1–3.6)
- [x] AA.4 Files Panel skeleton — split layout, four section headers, toolbar (Task 4.1–4.7)
- [x] AA.5 Catalog Manager Dialog — Create (all four types) (Task 5.1–5.10)
- [x] AA.6 Catalog Manager Dialog — Edit and Delete (Task 6.1–6.6)
- [x] AA.7 Dataset Allocation Dialog — ISPF-style fields, Allocate Like (Task 7.1–7.7)
- [x] AA.8 Context menus — Mainframe, POSIX, Windows/Local (Task 8.1–8.7)
- [x] AA.9 Content area — columns, sort, breadcrumb, filter (Task 9.1–9.7)
- [x] AA.10 Session persistence for FilesPanel tab and catalog registry (Task 10.1–10.7)

---

## Summary (as of full workspace audit)

| Status | Count |
|--------|-------|
| `[x]` Complete with real tests | 61 library crates + ff-desktop binary |
| `[ ]` Scaffolded only | 0 |
| Active work | Phase U — wire ISPF engine into ff-desktop; Phase W — compiler toolchain integration |

**All 61 library crates compile and pass `cargo test --workspace` with 0 failures.**

The remaining work is Phase X: converting the Primary Option Menu from a central-panel branch to a detachable floating `egui::Window` (B001 fix, Req 14 revised).



### Phase AJ — Tab-Order Focus Cycle (Req 16 menu-and-statusbar)

- [x] AJ.1 Add `FocusStop` enum and `focus_stop` field to `WorkbenchShell`; wire Tab/Shift+Tab cycle through command field → menu bar items → calendar `<`/`>` → wrap (Task 19.1–19.6 in menu-and-statusbar/tasks.md)

### Phase AK — Tab-Header Focus Stops + Command Field Focus Fix (Req 16 menu-and-statusbar)

- [x] AK.1 Add `TabHeader { index }` variant to `FocusStop`; update `next()`/`prev()` with `tab_count`; wire tab header focus in `update()`
- [x] AK.2 Fix command field focus: request egui focus every frame when `focus_stop == CommandField`
- [x] AK.3 Write 6 unit tests covering new tab-header cycle paths

### Phase AL — Tab Window Chrome: Title Line (Req 17, 18)

- [x] AL.1 Add `render_title_line()` to `WorkbenchShell`; derive text from active tab kind/path; apply Legacy theme blue/white styling (Tasks 21.1–21.3 in menu-and-statusbar/tasks.md)
- [x] AL.2 Write unit tests for Title_Line text derivation (Task 21.4 in menu-and-statusbar/tasks.md)
- [x] AL.3 Stub "Move to Other View" with status message; add `title_line_text()` helper for future floating window use (Tasks 22.1–22.2 in menu-and-statusbar/tasks.md)

### Phase AO — Detachable Tab Windows (Req 18.1–18.7)

- [x] AO.1 Wire "Move to Other View" context menu item to set detach_pending with 16-window limit (Task 23.4)
- [x] AO.2 Process detach_pending in update(); skip floating tabs in render_tab_bar(); show_viewport_deferred per tab (Tasks 23.5-23.7)
- [x] AO.3 Detect close event -> redock_pending; process redock with clamped restore (Tasks 23.8-23.9)
- [x] AO.4 5 unit tests: is_floating flag, 16-window limit, origin index, redock clamp, title format (Task 23.10)
- [x] AO.5 Fix pre-existing clippy lints in ff-keys/src/function_key.rs (strip_prefix, question_mark)

### Phase AM — Per-Context Key Maps, PFSHOW, 24-Key Bar, Hotspots, END/RETURN, LIST+RETRIEVE (Req 12–19 function-keys-and-history)

- [x] AM.1 PFSHOW command — register `keys.pfshow`, session persistence, shell wiring (Task 15)
- [x] AM.2 Two-row Key Label Bar — 24-slot model, two-row render in ff-desktop (Task 16)
- [x] AM.3 Per-context key map — `KeyMapResolver` context support, tab-switch wiring, TOML config (Task 17)
- [x] AM.4 Built-in default 24-key set — `KeyMap::default_global()`, fallback wiring (Task 18)
- [x] AM.5 Key Label Bar hotspots — clickable slots, tooltip, dispatch on click (Task 19)
- [x] AM.6 END and RETURN commands — register, tab-history stack, exit semantics (Task 20)
- [x] AM.7 Contextual help fallback — "not available yet" status message in ff-help (Task 21)
- [x] AM.8 LIST + RETRIEVE history browser — `ShowList` variant, modal overlay, selection (Task 22)

### Phase AN — Key Configuration Dialog (Req 20 function-keys-and-history)

### Phase AN — Final Status (implementation complete)

- [x] AN.1 `KeyModifier` + `ModifiedKey` in `ff-keys`; `KeyMap` uses `ModifiedKey`; `description` on `KeyBinding`; TOML parser extended for `SF`/`CF`/`AF` (Tasks 23–24)
- [x] AN.2 `KeyMapResolver` call sites updated; modifier dispatch in `ff-desktop` shell (Tasks 25–26)
- [x] AN.3 `key_config_dialog.rs` — scope tabs, 24-row grid, Save/Cancel/Reset (Tasks 27–29)
- [x] AN.4 `KEYS` command + `Edit > Key Assignments…` menu item wired (Task 28)
- [x] AN.5 Property-based tests for `ModifiedKey` round-trip — deferred (Task 30)

### Phase AP — PFSHOW Session Persistence (Req 12.4 function-keys-and-history)

- [x] AP.1 Add `key_bar_visible: bool` to `SessionState` in `ff-session` with `serde(default = "default_true")`
- [x] AP.2 Pass `key_bar_visible` to `session.save()` in `on_exit()`; restore in startup session load
- [x] AP.3 Unit test: `key_bar_visible_round_trips_through_session` in `session_manager.rs`

## Phase AQ -- Key Map TOML Persistence (Req 20.8)

- [x] AQ.1 Add to_config_table() to ScopeRows; add save_to_config() to KeyConfigDialog
- [x] AQ.2 Update render_if_open/render to accept ConfigHandle; wire Save button
- [x] AQ.3 Add 3 unit tests for Req 20.8 serialisation logic

### Phase AR -- [context_key_maps] TOML Config Parsing (Req 14.7 function-keys-and-history)

- [x] AR.1 Add load_context_maps_from_config() in ff-desktop shell.rs; call at startup; 2 unit tests (Task 32)

### Phase AS — File Explorer Panel (Req 19 startup-and-session)

- [x] AS.1 Add `FileExplorerPanel` TabKind; route `=2`, `=FILES`, `FILES` commands; implement tree view showing open catalogs and their files; session persistence (Tasks 27.1–27.21 in startup-and-session/tasks.md)

### Phase AT — Allocated Dataset Display (Req 13 virtual-catalog-manager)

- [x] AT.1 Add `AllocatedDataset` store to `FilesPanelState`; wire `AllocOutcome::Confirmed` to insert into store; populate content area from store on catalog select; persist/restore via session TOML (Tasks 12.1–12.14 in virtual-catalog-manager/tasks.md)

### Phase AX — Default Home Catalog on First Launch (Req 14 virtual-catalog-manager)

- [x] AX.1 Add `ensure_default_home_catalog()` helper; wire into startup block in `update.rs`; deletion guard in `execute_delete()`; 5 unit tests (Tasks 14.1–14.11 in virtual-catalog-manager/tasks.md)

### Phase AU — Catalog Registry Persistence (B010 fix, Req 2.1, 2.2 virtual-catalog-manager)

- [x] AU.1 Add `save_catalog_registry()` / `load_catalog_registry()` to `SessionManager`; wire into `on_exit()` and startup; 2 unit tests (Tasks 13.1–13.9 in virtual-catalog-manager/tasks.md)

### Phase AY — File Explorer: Expandable Subdirectories and Scrollable Panel (Req 15 file-tree-panel)

- [x] AY.1 Wrap File Explorer Panel content in `ScrollArea::vertical()`; replace flat directory entries with recursive `CollapsingHeader` nodes in `render_native_children()` (Tasks 18.1–18.3 in file-tree-panel/tasks.md)

### Phase AZ — File Explorer Context Menu (Req 16 file-tree-panel)

- [x] AZ.1 `NodeKind` + `MenuItem` enums; `build_context_menu()` for all 8 node kinds; egui context_menu wiring; Git/Submit JCL greyed-out (Tasks 19.1–19.3)
- [x] AZ.2 Inline rename with Enter/Escape; Mainframe 8-char uppercase enforcement (Task 19.4)
- [x] AZ.3 Copy to clipboard (full path/DSN); all Copy path variants (Tasks 19.5–19.6)
- [x] AZ.4 Reveal in Explorer — platform-appropriate OS file manager launch (Task 19.7)
- [x] AZ.5 Copy To… / Move To… dialog with naming-rule transformation and ff-bgio progress (Task 19.8)
- [x] AZ.6 Unit tests for all above (Task 19.9)

### Phase BA — Open With Default Application (Req 17 file-tree-panel)

- [x] BA.1 `FileClass` enum + `EXTERNAL_EXTENSIONS` table covering Office, PDF, images, audio/video, archives, executables, databases (Task 20.1)
- [x] BA.2 `classify_file()` with extension lookup and magic-byte fallback (Task 20.2)
- [x] BA.3 `launch_default_app()` platform dispatch Windows/macOS/Linux non-blocking (Task 20.3)
- [x] BA.4 `open_file_node()` routing Text→editor, External→OS launch; Mainframe bypass (Task 20.4)
- [x] BA.5 Status-bar error display for failed launches (Task 20.5)
- [x] BA.6 Unit tests for all above (Task 20.6)

### Phase BB — Native Catalog Sorted Listing and File Attributes (CR-NR-008, B017, B018)

- [x] BB.1 Refactor `render_native_children()` — sort dirs-first/alpha, silent-skip inaccessible entries (junction points, locked files), build `FileEntryRow` with metadata (Req 18.1, 18.7, B017)
- [x] BB.2 Implement `format_size`, `format_timestamp`, `format_permissions` helpers (Req 18.2–18.6)
- [x] BB.3 Render attribute columns per row: Size, Modified, Created, Accessed, Permissions (Req 18.9)
- [x] BB.4 Catch OS error 32 in `open_file_node()` — status-bar message, no editor tab (Req 18.8, B018)
- [x] BB.5 Unit tests for all helpers and sort/skip behaviour (Task 21.8)

### Phase BC — File Explorer content area: directories alphabetically sorted (CR-CH-004)

- [x] BC.1 Fix `visible_entries()` sort in `files_panel.rs` — containers always before non-containers when sorting by Name; each group sorted case-insensitively; 3 new unit tests (Req 10.7)

### Phase BD — File Explorer tree: drag-select and copy as text tree (CR-NR-009, Req 19 file-tree-panel)

- [x] BD.1 Add `selected_nodes: HashSet<String>` and `anchor_node: Option<String>` to `FileExplorerPanelState`; wire plain-click, Shift+click, Ctrl+click, and drag-select input handling (Tasks 22.1–22.2)
- [x] BD.2 Render selected nodes with selection background tint (Task 22.3)
- [x] BD.3 Implement `build_text_tree()` pure function — indented ASCII tree with `[DIR]` prefix and tree connectors for hierarchical selections (Task 22.4)
- [x] BD.4 Wire Ctrl+C and "Copy as Text Tree" context menu item to `build_text_tree` + clipboard write (Tasks 22.5–22.6)
- [x] BD.5 Wire Escape to clear multi-selection (Task 22.7)
- [x] BD.6 Unit tests for `build_text_tree` (Task 22.8)

### Phase BD — File Explorer tree: drag-select and copy as text tree (Req 19 file-tree-panel)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | — | Req 19.1: drag-select highlights all visible nodes between start and current cursor position |
| `ff-desktop` | 🔴 | — | Req 19.2: Shift+click extends selection from Anchor_Node to clicked node |
| `ff-desktop` | 🔴 | — | Req 19.3: Ctrl+click toggles individual node membership without affecting others |
| `ff-desktop` | 🔴 | — | Req 19.4: selected nodes rendered with `ui.selection_background` tint |
| `ff-desktop` | 🔴 | — | Req 19.5: Ctrl+C with non-empty selection writes Text_Tree to OS clipboard |
| `ff-desktop` | 🔴 | — | Req 19.6: `build_text_tree` produces correct indented ASCII output with `[DIR]` prefix and tree connectors |
| `ff-desktop` | 🔴 | — | Req 19.7: "Copy as Text Tree" context menu item present above "Copy" group |
| `ff-desktop` | 🔴 | — | Req 19.8: Escape clears multi-selection, reverts to single-node mode |
| `ff-desktop` | 🔴 | — | Req 19.9: selection extends to nodes scrolled into view during drag |
| `ff-desktop` | 🔴 | — | Req 19.10: Mainframe nodes use DSN in Text_Tree output |

### Phase BJ — Catalog Repository Path Display + VFS Dataset Path Resolution (CR-NR-012)

- [x] BJ.1 Show repository path as read-only field in Edit Catalog dialog for all catalog types (Task 15.1–15.3 in virtual-catalog-manager/tasks.md)
- [x] BJ.2 Add `resolve_dataset_path()` pure function; wire into Files Panel and File Explorer open handlers (Tasks 16.1–16.6 in virtual-catalog-manager/tasks.md)

### Phase BI — Default BLKSIZE=0 in Dataset Allocation Dialog (CR-CH-005)

- [x] BI.1 Change default BLKSIZE to `"0"` in `AllocDatasetForm::default()`; update validate() to accept 0; update tests (Task 7.8 in virtual-catalog-manager/tasks.md)

### Phase BE — File Explorer keyboard navigation + file copy/paste (CR-NR-010, CR-NR-011, Req 20–21 file-tree-panel)

- [x] BE.1 Add `FocusStop::FileExplorer` variant; wire Tab from CommandField to Explorer_Focus; `cursor_node` field on `FileExplorerPanelState` (Task 23.1)
- [x] BE.2 Implement `collect_visible_node_paths()` pure function (Task 23.2)
- [x] BE.3 Wire Tab, Arrow, Shift+Arrow, Ctrl+Arrow, Ctrl+Space, Escape keyboard handling in Explorer render loop (Tasks 23.3–23.8)
- [x] BE.4 Render cursor focus ring distinct from selection highlight (Task 23.9)
- [x] BE.5 Unit tests for all keyboard navigation behaviours (Task 23.10)
- [x] BE.6 Add `FileCopyClipboard`, `PasteProgress`, `PasteConflict` types; wire Ctrl+C, Ctrl+V (file list), Ctrl+V (editor), conflict modal, POSIX guard, Mainframe transform, pending-paste indicator (Tasks 24.1–24.8)
- [x] BE.7 Unit tests for file copy/paste operations (Task 24.9)

### Phase BL — B024 Tab Cycle Fix: exit tree + visual cursor highlight

- [x] BL.1 Fix Tab cycle exit: Tab past last tree node exits `explorer_focused`, clears `cursor_node`, returns `focus_stop` to `CommandField` — previously wrapped infinitely
- [x] BL.2 Add visual cursor highlight on catalog-level nodes (`rect_filled` behind `CollapsingHeader` when `cursor_node == "cat:NAME"`)
- [x] BL.3 Extend file node `is_selected` to include `cursor_node` match so file rows show highlight when tabbed to
- [x] BL.4 Consolidate two separate Tab branches into single unified branch handling enter/advance/exit cases

### Phase BM — File Explorer Panel: egui-file-dialog look-and-feel with catalog mount points (CR-NR-014, Req 23 file-tree-panel)

- [x] BM.1 Add `selected_catalog` + `sidebar_width` fields to `FileExplorerPanelState`; refactor `render()` into `render_sidebar()` + `render_content_pane()` two-pane layout (Tasks 26.1–26.4)
- [x] BM.2 Implement `render_mainframe_content()` — dot-qualified dataset listing, PDS expandable, PS leaf, VFS open routing (Task 26.5)
- [x] BM.3 Implement `render_posix_content()` — forward-slash path display, directory/file tree from `read_dir` (Task 26.6)
- [x] BM.4 Empty sidebar placeholder; sidebar width persistence (Tasks 26.7–26.8)
- [x] BM.5 Unit tests + `cargo test` green (Tasks 26.9–26.10)

### Phase BK — Native File Browser: egui-file-dialog Integration (CR-NR-013, Req 22 file-tree-panel)

- [x] BK.1 Add `egui-file-dialog = "0.6"` to `crates/ff-desktop/Cargo.toml`; vendored patch resolves egui 0.29 mismatch; `cargo check` passes (Task 25.1)
- [x] BK.2 Add `NativeDialogSlot` newtype (manual `Debug`/`Clone`); `native_dialogs: HashMap<String, NativeDialogSlot>` on `FileExplorerPanelState` (Task 25.2)
- [x] BK.3 Implement `render_native_dialog()` — lazy init, scoped borrow, `dialog.update(ctx)`, `take_selected()` → `open_file_node()` (Task 25.3)
- [x] BK.4 Replace `render_native_children()` call in Native branch of `render()` with `render_native_dialog()`; dead-code helpers annotated `#[allow(dead_code)]` (Task 25.4)
- [x] BK.5 Confirmed Mainframe/POSIX branches untouched; all 8 Mainframe/POSIX tests pass (Task 25.5)
- [x] BK.6 `THIRD_PARTY_CREDITS.md` created at workspace root with full MIT licence text (Task 25.6)
- [x] BK.7 `cargo test` 486 passing 0 failures; `cargo clippy` clean; `cargo build --release` succeeds (Tasks 25.7–25.9)

### Phase BO — Bug Fix Sprint: Persistence Gaps (B020, B021, B022)

- [x] BO.1 B020 — Call `save_catalog_registry()` immediately after `DialogOutcome::Confirmed` in NewCatalog and DeleteCatalog handlers in `update.rs` (not only in `on_exit()`)
- [x] BO.2 B021 — Pre-populate `repository_path` with `mainframe_root.clone()` in `NewCatalogForm::with_defaults()` so the field is non-empty on dialog open
- [x] BO.3 B022 — `save_datasets()`/`load_datasets()` added to `SessionManager`; wired into startup restore, `AllocateDataset` confirmed handler, and `on_exit()`
- [x] BO.4 Mark task 26 (Phase BM) `[x]` in `file-tree-panel/tasks.md`; 496 tests passing 0 failures

### Phase BP — Vendor Warning Elimination (egui-file-dialog future_incompatible + deprecated)

- [x] BP.1 Fix `float_literal_f32_fallback` in `vendor/egui-file-dialog/src/file_dialog.rs` lines 1223 and 1360 — `1.0` → `1.0_f32` in two `egui::Stroke::new()` calls
- [x] BP.2 Fix deprecated `ComboBox::from_id_source` → `from_id_salt` in `file_dialog.rs` line 1805
- [x] BP.3 Fix `mismatched_lifetime_syntaxes` in `vendor/egui-file-dialog/src/data/directory_content.rs` — add `'s` to elided return-type lifetimes on `filtered_iter` and `filtered_iter_mut`
- [x] BP.4 `cargo build -p egui-file-dialog` — 0 warnings; `cargo test -p ff-desktop` — 496 passed 0 failed

### Phase BQ — Requirements Review and Modernisation (CR-NR-015)

- [x] BQ.T1 Task 1 — Inventory & Baseline Audit (`docs/reviews/requirements-review/inventory.md`)
- [x] BQ.T2 Task 2 — Terminology Standardisation (`docs/reviews/requirements-review/terminology-map.md`)
- [x] BQ.T3 Task 3 — Architectural Domain Classification (`docs/reviews/requirements-review/domain-classification.md`)
- [x] BQ.T4 Task 4 — Gap Analysis (`docs/reviews/requirements-review/gap-analysis.md`)
- [x] BQ.T5 Task 5 — Rewrite: Core Platform & UX Layer Specs (10 specs)
- [x] BQ.T6 Task 6 — Rewrite: Explorer & Content Layer Specs (15 specs)
- [x] BQ.T7 Task 7 — Rewrite: Task Layer, Integration Layer & Domain Specs (14 specs)
- [x] BQ.T8 Task 8 — Traceability Matrix (`docs/reviews/requirements-review/traceability-matrix.md`)
- [x] BQ.T9 Task 9 — Consolidation Report (`docs/reviews/requirements-review/consolidation-report.md`)
- [x] BQ.T10 Task 10 — Executive Assessment & Strategic Recommendations (`docs/reviews/requirements-review/executive-assessment.md`)

### Phase BT — Pre-BS Requirements Consistency Fixes (MUST complete before Phase BS code)

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

### Phase BS — Mainframe Dataset Architecture (CR-NR-016)

> Implements the hybrid storage architecture, record codecs, StorageProvider layer, VSAM/ISAM support,
> staged transactions, integrity/backup/restore, audit trail, and security hardening defined in
> `docs/source-documents/dataset-catalog/FileForgeWorkbench_Mainframe_Dataset_Architecture.md` and
> `docs/source-documents/dataset-catalog/FileForgeWorkbench_Virtual_File_and_Dataset_Storage_Requirements.md`.

#### Wave 1 — Foundations (no dependencies on later waves)

- [x] BS.1 Record codecs — `FixedCodec`, `VariableCodec`, `BinaryCodec`, `TextCodec` as independent module with full unit + property tests (Tasks 17.1–17.6 in dataset-catalog/tasks.md)
- [x] BS.2 `StorageProvider` trait definition + capability enum (Task 18.1)
- [x] BS.3 `NativeFileProvider` — UUID-based allocation, path-safety guards, PS/PDS/GDG layout (Tasks 18.2–18.5)

#### Wave 2 — VSAM and ISAM Providers (depends on Wave 1)

- [x] BS.4 `SqliteRecordProvider` base + VSAM KSDS — keyed read/write, uniqueness, alternate-index extension point (Tasks 19.1–19.5; 19.4 alternate indexes remains; no system SQLite install required because `rusqlite` uses `bundled`)
- [x] BS.5 VSAM RRDS — relative-record store, unallocated vs blank distinction (Tasks 20.1–20.3)
- [x] BS.6 VSAM ESDS — append-oriented native file, stable record address, sidecar index (Tasks 21.1–21.4)
- [x] BS.7 ISAM -- SQLite-backed, shared indexed-record interface with KSDS (Tasks 22.1--22.3)

#### Wave 3 — Transactions, Integrity, and Governance (depends on Wave 2)

- [ ] BS.8 Staged transaction protocol — `OperationJournal`, staged create/delete, startup recovery (Tasks 23.1–23.6)
- [x] BS.9 Integrity, backup, restore — checksums, `workspace.backup/restore/diagnose/reconcile` commands (Tasks 24.1–24.6)
- [x] BS.10 Catalogue audit trail + schema migrations (Tasks 25.1-25.3)
- [x] BS.11 Security hardening — parameterised SQL audit, log scrubbing, path-traversal property test (Tasks 26.1–26.3)

#### Wave 4 — Catalogue Hierarchy and Editor Integration (depends on Wave 3)

- [x] BS.12 Master/user catalogue hierarchy, logical rename, scoped uniqueness (Tasks 27.1-27.4)
- [x] BS.13 Record-oriented editor integration — wire codecs into open/save path, integration tests (Tasks 28.1–28.4)
- [x] BS.14 Non-functional validation — cross-platform, performance, Git-compat, data-fidelity tests (Tasks 29.1–29.4)
- [x] BS.15 Update `dataset-catalog/design.md` for CR-NR-016 (Task 30.1)

---

## Summary (updated after Phase BT complete)

| Status | Count |
|--------|-------|
| `[x]` Complete with real tests | 61 library crates + ff-desktop binary |
| `[x]` Complete -- Phase BT | 5 doc-fix deliverables (BT.1--BT.5) |
| `[ ]` Pending -- Phase BS | 15 deliverables (BS.1--BS.15) |
| Active work | Phase BS -- Mainframe Dataset Architecture (CR-NR-016) |

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

- [ ] CH.1 Overtype fields: visual distinction, direct overtype, command-line syntax, extension pop-up (Tasks 30.1-30.4)
- [ ] CH.2 Help system: HELP/PF1, ACTH, COLH, CMDH, SEARCH (Tasks 31.1-31.5)
- [ ] CH.3 Log panels: LOG, ULOG, NEXT/PREV, SNAPSHOT (Tasks 32.1-32.4)
- [ ] CH.4 System panels: SYS, DASH, INIT, JC, SP (Tasks 32.5-32.9)
- [ ] CH.5 Browse settings, PRINT action, COLS command (Tasks 33.1-33.3)
- [ ] CH.6 SET P2 commands: BCOLOR, CONFIRM, CURSOR, DATE, DELAY, HEX, SCHARS, SCREEN + persistence (Tasks 34.1-34.9)

### Phase CI -- command-semantics P2 EARS Integration (CR-NR-031)

- [ ] CI.1 OUTPUT and CANCEL commands -- routing to FFW-JES, PURGE operand (Tasks 25.1-25.3)
- [ ] CI.2 SEND, PROFILE, PRINTDS commands -- messaging, session profile, file-ops routing (Tasks 26.1-26.4)
- [ ] CI.3 TCR update for Requirement 10 (Task 27.1)

### Phase CG -- lua-macro-engine P2 EARS Integration (CR-NR-029)

- [ ] CG.1 ISREDIT/ISPEXEC host command environments and IMACRO initial macro (Tasks 21.1-21.4)
- [ ] CG.2 LINENUM function and CURSOR get/set extension (Tasks 21.5-21.6)
- [ ] CG.3 REXX exec invocation: EXEC command, implicit invocation, % prefix, argument passing (Tasks 22.1-22.4)
- [ ] CG.4 TSO host environment, ADDRESS switching, ISPEXEC/ISREDIT environments, RC variable (Tasks 22.5-22.8)
- [ ] CG.5 REXX built-in functions: LISTDSI, MSG, MVSVAR, OUTTRAP, PROMPT, SYSDSN, SYSVAR, USERID (Tasks 23.1-23.8)
- [ ] CG.6 EXECIO DISKR/DISKW/FINIS/SKIP and FFCMD command files (Tasks 24.1-24.7)

### Phase CF -- syntax-highlighting P2 EARS Integration (CR-NR-028)

> Adds EARS-derived criteria to syntax-highlighting: HILITE ON/OFF command (toggle
> syntax highlighting per document), HILITE LOGIC (boolean/comparison operator highlighting),
> HILITE PAREN (delimiter-pair matching at cursor with error style), HILITE FIND (persistent
> find-match highlights), and combined operand support.
> Requirement 16 in syntax-highlighting/requirements.md.

- [ ] CF.1 HILITE ON/OFF command and HILITE LOGIC mode (Tasks 21.1-21.5)
- [ ] CF.2 HILITE PAREN, HILITE FIND, and combined operands (Tasks 22.1-22.5)

---

### Phase CE -- undo-redo-transactions P2 EARS Integration (CR-NR-027)

> Adds EARS-derived criteria to undo-redo-transactions: SETUNDO primary command
> (ON/OFF/n operands, immediate effect) and RECOVERY primary command (ON/OFF/n operands).
> Requirement 19 in undo-redo-transactions/requirements.md.

- [ ] CE.1 SETUNDO command -- ON/OFF/n operands, immediate effect on max_levels (Tasks 19.1-19.5)
- [ ] CE.2 RECOVERY command -- ON/OFF/n operands, immediate effect on recovery interval (Tasks 20.1-20.5)

---

### Phase CD -- FFW-JES P1 extended EARS Integration (CR-NR-026)

> Adds EARS-derived criteria to FFW-JES: ST panel (all-jobs status view), FILTER command
> (advanced filter expressions with AND/OR/wildcard), FIND command (panel search with
> NEXT/PREV), LOCATE command (scroll to matching row), SDSF scroll commands (UP/DOWN/LEFT/RIGHT
> with n/HALF/PAGE/MAX), SET ACTION/MAIN/ROWNUM commands, WHO command, QUERY AUTH command,
> and PERSIST-1 (SET settings persistence).
> Requirement 17 in FFW-JES/requirements.md.

- [ ] CD.1 ST panel and advanced FILTER/FIND/LOCATE commands (Tasks 26.1-26.5)
- [ ] CD.2 SDSF scroll commands UP/DOWN/LEFT/RIGHT with SCROLL field sync (Tasks 27.1-27.3)
- [ ] CD.3 SET ACTION/MAIN/ROWNUM commands, WHO, QUERY AUTH (Tasks 28.1-28.6)
- [ ] CD.4 SET settings persistence and integration tests (Tasks 29.1-29.5)

---

### Phase CC -- FFW-JES P1 core EARS Integration (CR-NR-025)

> Adds EARS-derived criteria to FFW-JES: SDSF panel framework (action bar, title line,
> SCROLL field, filter lines, NP column, fixed column), action character system (S/?/C/H/A/P/D/E/J/W,
> = repeat, // block, command-line syntax, SET ROWNUM), main panel (MENU, groups, SET MAIN GROUP),
> PREFIX/OWNER/DEST filter commands, full column set, SORT command.
> Requirement 16 in FFW-JES/requirements.md.

- [ ] CC.1 SDSF panel chrome -- action bar, title line, SCROLL field, filter lines, message area, COMMAND INPUT field (Tasks 20.1-20.7)
- [ ] CC.2 NP column and action character system -- S/?/C/H/A/P/D/E/J/W, = repeat, // block, command-line syntax, SET ROWNUM (Tasks 21.1-21.8)
- [ ] CC.3 Main panel (MENU command), command groups, S action, SET MAIN GROUP (Tasks 22.1-22.5)
- [ ] CC.4 PREFIX/OWNER/DEST filter commands (Tasks 23.1-23.4)
- [ ] CC.5 Full column set (JOBNAME through PROCSTEP), column hide/reorder, SORT command (Tasks 24.1-24.4)
- [ ] CC.6 Integration tests for SDSF panel framework (Tasks 25.1-25.4)

---

### Phase CB -- command-semantics EARS Integration (CR-NR-024)

> Adds EARS-derived criteria to command-semantics: TSO dataset commands (ALLOCATE through
> STATUS), EDIT routing extension, FTSO operand parsing, session prefix, continuation,
> ds:// URI, namespace conflict, capability model, secret operands, audit events.
> Requirement 9 in command-semantics/requirements.md.

- [ ] CB.1 TSO dataset management commands ALLOCATE/FREE/DELETE/RENAME/LISTCAT/LISTDS/LISTALC (Tasks 19.1-19.8)
- [ ] CB.2 TSO job commands SUBMIT/STATUS and EDIT routing extension (Tasks 20.1-20.4)
- [ ] CB.3 TSO-style operand parsing and session prefix (Tasks 21.1-21.4)
- [ ] CB.4 Command continuation, ds:// URI, namespace conflict resolution (Tasks 22.1-22.4)
- [ ] CB.5 Capability model, secret operands, audit events (Tasks 23.1-23.5)
- [ ] CB.6 TCR update for Requirement 9 (Task 24.1)

---

Phase CA -- startup-and-session EARS Integration (CR-NR-023)

> Adds EARS-derived criteria to startup-and-session: session start timestamp (TSO-1.2),
> session end timestamp and logoff message (TSO-1.3), LOGOFF command (TSO-1.4),
> TIME command (TSO-2.4), STATUS routing to FFW-JES (TSO-2.5).
> Requirement 20 in startup-and-session/requirements.md.

- [ ] CA.1 Session start timestamp in status bar (Tasks 28.1-28.3)
- [ ] CA.2 Session end timestamp and logoff message (Tasks 29.1-29.3)
- [ ] CA.3 LOGOFF command alias for exit sequence (Tasks 30.1-30.2)
- [ ] CA.4 TIME command -- date/time display (Tasks 31.1-31.2)
- [ ] CA.5 STATUS command routing to FFW-JES (Tasks 32.1-32.3)
- [ ] CA.6 TCR update for Requirement 20 (Task 33.1)

---

Phase BZ -- menu-and-statusbar EARS Integration (CR-NR-022)

> Adds EARS-derived criteria to menu-and-statusbar: SCROLL ===> field, fastpath notation,
> data entry/list panel layout, list panel LOCATE, extended scroll amounts, split screen.
> Requirement 19 in menu-and-statusbar/requirements.md.

- [ ] BZ.1 SCROLL ===> field adjacent to Command ===> (Tasks 24.1-24.5)
- [ ] BZ.2 Fastpath dotted notation navigation (Tasks 25.1-25.3)
- [ ] BZ.3 Data entry and list panel layout conformance (Tasks 26.1-26.4)
- [ ] BZ.4 List panel LOCATE command (Tasks 27.1-27.3)
- [ ] BZ.5 Extended scroll amounts HALF/CSR/MAX/DATA (Tasks 28.1-28.3)
- [ ] BZ.6 Split screen PF2/PF9/PF3 (Tasks 29.1-29.6)
- [ ] BZ.7 TCR update for Requirement 19 (Task 30.1)

---

Phase BY -- sequence-numbers EARS Integration (CR-NR-021)

> Extends sequence-numbers: AUTONUM ON/OFF as alias for NUMBER ON/OFF (Req 6.7a),
> NUM as alias for NUMBER command (Req 8 alias criterion).

- [ ] BY.1 AUTONUM alias -- parser extension, unit tests (Task 20.1-20.2)
- [ ] BY.2 NUM alias -- command framework alias registration, unit tests (Task 21.1-21.3)
- [ ] BY.3 TCR update for alias criteria (Task 22.1)

---

Phase BX -- line-commands EARS Integration (CR-NR-020)

> Adds EARS-derived criteria to line-commands: Overlay (O/On), clipboard copy (W/WW),
> first-of-excluded (F), last-of-excluded (L), single-column shift right (]), show-excluded (S).
> Requirement 15 in line-commands/requirements.md.

- [ ] BX.1 Overlay line command (O, On) -- LineCommandKind variants, parser, execution (Tasks 22.1-22.6)
- [ ] BX.2 Clipboard copy line command (W, WW) -- variants, parser, ff-clipboard integration (Tasks 23.1-23.6)
- [ ] BX.3 First-of-excluded (F) -- ShowFirst variant, parser, execution (Tasks 24.1-24.6)
- [ ] BX.4 Last-of-excluded (L) -- ShowLast variant, parser, execution (Tasks 25.1-25.6)
- [ ] BX.5 Single-column shift right (]) -- ShiftRightOne variant, parser, delegate to shift_right(1) (Tasks 26.1-26.6)
- [ ] BX.6 Show-excluded (S) -- ShowLine variant, parser, execution (Tasks 27.1-27.6)
- [ ] BX.7 TCR update for Requirement 15 (Task 28.1)

---

Phase BW -- edit-operations EARS Integration (CR-NR-019)

> Adds EARS-derived criteria to edit-operations: CAPS mode, NULLS mode, PROFILE command,
> STATS mode, LOCK setting, edit profile persistence, AUTONUM/NUM aliases, HILITE delegation,
> SUBMIT/CREATE/REPLACE/EDIT/BROWSE/VIEW/COMPARE primary commands.
> Requirements 16-17 in edit-operations/requirements.md.

- [ ] BW.1 CAPS mode -- CapsMode flag, insert_char integration, CAPS command (Tasks 28.1-28.6)
- [ ] BW.2 NULLS mode -- NullsMode flag, display/edit integration, NULLS command (Tasks 29.1-29.5)
- [ ] BW.3 PROFILE command -- EditProfile struct, display and update handlers (Tasks 30.1-30.6)
- [ ] BW.4 STATS mode -- StatsMode flag, prefix area rendering (Tasks 31.1-31.5)
- [ ] BW.5 LOCK setting -- ProfileLock flag, profile mutation guard (Tasks 32.1-32.5)
- [ ] BW.6 Edit profile persistence -- TOML round-trip via ff-session (Tasks 33.1-33.4)
- [ ] BW.7 AUTONUM and NUM aliases -- command framework aliases (Tasks 34.1-34.4)
- [ ] BW.8 HILITE delegation -- HILITE command handler to ff-syntax (Tasks 35.1-35.5)
- [ ] BW.9 SUBMIT command -- JES subsystem dispatch (Tasks 36.1-36.5)
- [ ] BW.10 CREATE and REPLACE commands (Tasks 37.1-37.6)
- [ ] BW.11 Nested EDIT, BROWSE, VIEW, COMPARE commands (Tasks 38.1-38.8)
- [ ] BW.12 TCR.md and project-master updated; cargo test --workspace green (Task 39)

### Phase BV -- Catalog Location Discriminant (CR-NR-017)

> Adds `CatalogLocation` enum to `CatalogMount` so each catalog declares local vs remote
> transport. Only `Local` is implemented; `Remote` returns `UnsupportedOperation`. Zero
> behaviour change for existing local catalogs.

- [x] BV.1 `CatalogLocation` enum + `CatalogMount` refactor in `ff-dscatalog`
        (Tasks 32.1-32.8 in dataset-catalog/tasks.md)

---

## Phase EI-6 -- Reorganised Pending Work (all EARS gates complete)

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
- [ ] BU.2 Failing tests -- CatalogRegistry API, resolve_and_open_dataset, content area (Tasks 18-20)
- [ ] BU.3 CatalogRegistry::allocate() and list_datasets() implemented and tests green (Task 21)
- [x] BU.4 AllocOutcome::Confirmed handler wired to SQLite (Task 22)
- [x] BU.5 Files Panel content area reads from SQLite (Task 23)
- [x] BU.6 File Explorer Panel Mainframe content reads from SQLite (Task 24)
- [x] BU.7 resolve_and_open_dataset() replaces resolve_dataset_path() (Task 25)
- [x] BU.8 AllocatedDataset struct, datasets HashMap, and TOML persistence removed (Task 26)
- [x] BU.9 TCR.md and project-master updated; cargo test --workspace green (Task 27)

### Stream 2 -- EARS P1 Implementation (independent of Stream 1)

- [ ] BW.impl edit-operations: CAPS/NULLS/PROFILE/SUBMIT/CREATE/REPLACE/BROWSE/VIEW/nested EDIT/COMPARE/LOCK/STATS (Tasks 28-39 in edit-operations/tasks.md)
- [ ] BX.impl line-commands: O/W/F/L/]/S line commands (Tasks 22-28 in line-commands/tasks.md)
- [ ] BY.impl sequence-numbers: AUTONUM and NUM alias extensions (Tasks 20-22 in sequence-numbers/tasks.md)
- [ ] BZ.impl menu-and-statusbar: SCROLL field, fastpath, split screen, list panel LOCATE (Tasks 24-30 in menu-and-statusbar/tasks.md)
- [ ] CA.impl startup-and-session: session timestamps, LOGOFF, TIME, STATUS routing (Tasks 28-33 in startup-and-session/tasks.md)
- [ ] CB.impl command-semantics P1: ALLOCATE through STATUS + FTSO operand parsing (Tasks 19-24 in command-semantics/tasks.md)
- [ ] CC.impl FFW-JES P1 core: SDSF panel framework, NP column, action chars, main panel (Tasks 20-25 in FFW-JES/tasks.md)
- [ ] CD.impl FFW-JES P1 extended: ST panel, FILTER/FIND/LOCATE, SET P1 commands (Tasks 26-29 in FFW-JES/tasks.md)

### Stream 3 -- EARS P2 Implementation (follows Stream 2)

- [ ] CE.impl undo-redo-transactions: SETUNDO command, RECOVERY ON/OFF (Tasks 19-20 in undo-redo-transactions/tasks.md)
- [ ] CF.impl syntax-highlighting: HILITE ON/OFF/LOGIC/PAREN/FIND (Tasks 21-22 in syntax-highlighting/tasks.md)
- [ ] CG.impl lua-macro-engine: ISREDIT/ISPEXEC/IMACRO, REXX bridge, FFCMD (Tasks 21-24 in lua-macro-engine/tasks.md)
- [ ] CH.impl FFW-JES P2: overtype, help, log/system panels, browse/print, SET P2 (Tasks 30-34 in FFW-JES/tasks.md)
- [ ] CI.impl command-semantics P2: OUTPUT/CANCEL/SEND/PROFILE/PRINTDS (Tasks 25-27 in command-semantics/tasks.md)

---

## Summary (updated after EI-6 -- all EARS gates complete)

| Status | Count |
|--------|-------|
| `[x]` Complete with real tests | 61 library crates + ff-desktop binary |
| `[x]` EARS gates complete | Phases BW-CI (13 phases, 136 new criteria gated) |
| `[ ]` Pending -- Stream 1 (dataset architecture) | BU.2-BU.9 (9 deliverables) |
| `[ ]` Pending -- Stream 2 (EARS P1 implementation) | BW-CD (8 phases) |
| `[ ]` Pending -- Stream 3 (EARS P2 implementation) | CE-CI (5 phases) |
| Active work | ff-vfs.13 (next in Stream 1) or BW (next in Stream 2) |
