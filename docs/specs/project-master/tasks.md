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

- [ ] AM.1 PFSHOW command — register `keys.pfshow`, session persistence, shell wiring (Task 15)
- [ ] AM.2 Two-row Key Label Bar — 24-slot model, two-row render in ff-desktop (Task 16)
- [ ] AM.3 Per-context key map — `KeyMapResolver` context support, tab-switch wiring, TOML config (Task 17)
- [ ] AM.4 Built-in default 24-key set — `KeyMap::default_global()`, fallback wiring (Task 18)
- [ ] AM.5 Key Label Bar hotspots — clickable slots, tooltip, dispatch on click (Task 19)
- [ ] AM.6 END and RETURN commands — register, tab-history stack, exit semantics (Task 20)
- [ ] AM.7 Contextual help fallback — "not available yet" status message in ff-help (Task 21)
- [ ] AM.8 LIST + RETRIEVE history browser — `ShowList` variant, modal overlay, selection (Task 22)

### Phase AN — Key Configuration Dialog (Req 20 function-keys-and-history)

- [ ] AN.1 Add `KeyModifier` enum and `ModifiedKey` struct to `ff-keys`; update `KeyMap` to use `ModifiedKey` as key type; add `description` field to `KeyBinding`; update TOML parser for `SF`/`CF`/`AF` prefixes (Tasks 23–24)
- [ ] AN.2 Update `KeyMapResolver` and `ff-desktop` modifier dispatch — read `egui::Modifiers`, construct `ModifiedKey`, dispatch modifier bindings (Tasks 25–26)
- [ ] AN.3 Create `key_config_dialog.rs` in `ff-desktop` — scope tabs, 10-column grid, Save/Cancel/Reset, TOML write (Tasks 27–29)
- [ ] AN.4 Wire `KEYS` command and `Edit > Key Assignments…` menu item (Task 28)
- [ ] AN.5 Property-based tests for `ModifiedKey` round-trip and binding isolation (Task 30)

### Phase AN — Final Status (implementation complete)

- [x] AN.1 `KeyModifier` + `ModifiedKey` in `ff-keys`; `KeyMap` uses `ModifiedKey`; `description` on `KeyBinding`; TOML parser extended for `SF`/`CF`/`AF` (Tasks 23–24)
- [x] AN.2 `KeyMapResolver` call sites updated; modifier dispatch in `ff-desktop` shell (Tasks 25–26)
- [x] AN.3 `key_config_dialog.rs` — scope tabs, 24-row grid, Save/Cancel/Reset (Tasks 27–29)
- [x] AN.4 `KEYS` command + `Edit > Key Assignments…` menu item wired (Task 28)
- [ ] AN.5 Property-based tests for `ModifiedKey` round-trip — deferred (Task 30)

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

- [ ] AS.1 Add `FileExplorerPanel` TabKind; route `=2`, `=FILES`, `FILES` commands; implement tree view showing open catalogs and their files; session persistence (Tasks 27.1–27.21 in startup-and-session/tasks.md)

### Phase AT — Allocated Dataset Display (Req 13 virtual-catalog-manager)

- [ ] AT.1 Add `AllocatedDataset` store to `FilesPanelState`; wire `AllocOutcome::Confirmed` to insert into store; populate content area from store on catalog select; persist/restore via session TOML (Tasks 12.1–12.14 in virtual-catalog-manager/tasks.md)

### Phase AX — Default Home Catalog on First Launch (Req 14 virtual-catalog-manager)

- [x] AX.1 Add `ensure_default_home_catalog()` helper; wire into startup block in `update.rs`; deletion guard in `execute_delete()`; 5 unit tests (Tasks 14.1–14.11 in virtual-catalog-manager/tasks.md)

### Phase AU — Catalog Registry Persistence (B010 fix, Req 2.1, 2.2 virtual-catalog-manager)

- [x] AU.1 Add `save_catalog_registry()` / `load_catalog_registry()` to `SessionManager`; wire into `on_exit()` and startup; 2 unit tests (Tasks 13.1–13.9 in virtual-catalog-manager/tasks.md)
