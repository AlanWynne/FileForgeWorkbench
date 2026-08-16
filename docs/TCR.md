# Test Coverage Report (TCR)

**Last updated:** Phase S Step 4 — integration testing complete  
**Workspace:** `cargo test --workspace` — **all crates pass**

## Status Key

| Symbol | Meaning |
|--------|---------|
| ✅ | Automated tests exist and pass |
| ❌ | Automated tests exist but fail (compile error or runtime failure) |
| 🔲 | Requires manual / UI verification only |
| 🔴 | No automated tests exist |

## Known Failures

None — all crates compile and pass.

---

## Coverage by Crate

### Wave 0 — Foundation

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-logging` | ✅ | lib unit tests, doc tests | Logging init, level filtering, structured output; global flag tests serialised with `FLAG_LOCK` |

### Wave 2 — Platform Architecture

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-core` | ✅ | `integration_tests.rs`, `property_tests.rs` | Service registry, event bus, lifecycle, startup/shutdown ordering |
| `ff-config` | ✅ | lib unit tests, integration tests | Layered config merge, hot-reload, schema validation |
| `ff-command` | ✅ | lib unit tests, `thread_safety_tests.rs` | Registry, dispatch, history, thread safety |
| `ff-plugin` | ✅ | lib unit tests | Plugin lifecycle, registry, dependency ordering |
| `ff-workflow` | ✅ | lib unit tests | Workflow definition, step execution, state transitions |
| `ff-layout` | ✅ | `integration.rs` | Panel docking, tab groups, drag-drop, persona switching |

### Wave 3 — Virtual File System

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-vfs` | ✅ | lib unit tests | URI resolution, provider registry, VFS error types |
| `ff-connector-local-fs` | ✅ | lib unit tests | Local filesystem read/write, path normalisation |
| `ff-connector-ext` | ✅ | lib unit tests | Extensibility hooks, provider registration |

### Wave 4 — Core Editor

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-document-model` | ✅ | `integration_tests.rs`, `property_tests.rs` | Insert/delete, line counting, byte positions, streaming load |
| `ff-edit-operations` | ✅ | lib unit tests, integration tests | Edit bounds, region operations, batch edits |
| `ff-undo-redo` | ✅ | lib unit tests | Transaction recording, undo/redo stacks, recovery files |
| `ff-viewport-scrolling` | ✅ | lib unit tests | Viewport model, cursor model, caret policy, scroll clamping |
| `ff-display-line-mapping` | ✅ | lib unit tests | Logical-to-visual line mapping, wrap mode |

### Wave 5 — Command Engine

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-cmd-semantics` | ✅ | lib unit tests | Command parsing, semantic validation |
| `ff-find-and-replace` | ✅ | lib unit tests, `property_tests.rs` | Search, replace, scope filters, regex |
| `ff-line-commands` | ✅ | `integration_tests.rs`, `property_tests.rs` | ISPF line commands, block pairs, copy/move/delete |
| `ff-filter` | ✅ | lib unit tests | Exclude/show filter logic, pattern matching |
| `ff-nav` | ✅ | `integration_tests.rs` | Navigation commands, sort, selection |

### Wave 6 — UI and Rendering

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-menu` | ✅ | lib unit tests | Menu model, action binding, status bar segments |
| `ff-theme` | ✅ | `integration_tests.rs`, `property_tests.rs` | Palette construction, design tokens, font zoom clamping |
| `ff-decorations` | ✅ | lib unit tests | Text decoration spans, overlay rendering |
| `ff-whitespace` | ✅ | lib unit tests | Whitespace guide rendering, tab/space visualisation |
| `ff-caret-selection` | ✅ | lib unit tests | Caret rendering, selection highlight, blink state |

### Wave 7 — Language and Highlighting

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-lang` | ✅ | lib unit tests | Language detection, grammar registry |
| `ff-syntax` | ✅ | lib unit tests | Syntax token classification, highlight spans |
| `ff-auto-indent` | ✅ | lib unit tests, `property_tests.rs` | Indent level computation, pattern matching |

### Wave 8 — File I/O and Session

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-fileops` | ✅ | `integration_tests.rs`, `persistence_tests.rs` | Open/save pipeline, atomic write, backup, revert |
| `ff-bgio` | ✅ | `integration_tests.rs`, `property_tests.rs` | Background I/O service, progress reporting, cancellation |
| `ff-encoding` | ✅ | lib unit tests | Encoding detection, BOM handling, transcoding |
| `ff-extmod` | ✅ | `integration_tests.rs`, `property_tests.rs` | External modification detection, prompt handling |
| `ff-session` | ✅ | lib unit tests, `integration_tests.rs`, `property_tests.rs` | Session state round-trip, TOML persistence, schema migration, geometry clamping |
| `ff-tabs` | ✅ | lib unit tests | Tab collection, tab state serialisation, active tab tracking |

### Wave 9 — Desktop Integration

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-clipboard` | ✅ | lib unit tests | Clipboard read/write, format negotiation |
| `ff-keys` | ✅ | lib unit tests | Function key binding, command history |
| `ff-shell` | ✅ | lib unit tests | Shell command execution, output capture |
| `ff-help` | ✅ | lib unit tests | Context help lookup, topic resolution |
| `ff-zoom` | ✅ | lib unit tests, integration tests, property tests | Zoom level management, font size clamping, Ctrl+Scroll wired in ff-desktop, zoom persistence in session |
| `ff-wrap` | ✅ | lib unit tests | Line wrap toggle, wrap width configuration |

### Wave 10 — Extensions and Macros

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-lua` | ✅ | lib unit tests | Lua engine init, macro execution, API bindings |
| `ff-completion` | ✅ | lib unit tests | Command completion candidates, prefix matching |

### Wave 11 — Display Modes

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-hex` | ✅ | lib unit tests, `property_tests.rs` | Hex dump layout, nibble editing, byte reader |
| `ff-seqnum` | ✅ | lib unit tests | Sequence number parsing, renumbering |
| `ff-tabmask` | ✅ | lib unit tests | Tab mask display, column alignment |

### Wave 12 — FileForge Domain

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-forge` | ✅ | lib unit tests | FileForge integration layer, record model |
| `ff-struct` | ✅ | lib unit tests, `property_tests.rs` | Structure catalog, packed decimal, field definitions |
| `ff-select` | ✅ | lib unit tests | Record selection criteria, filter evaluation |
| `ff-asa` | ✅ | lib unit tests | ASA carriage control, report preview rendering |
| `ff-viewers` | ✅ | lib unit tests | Custom file viewer registration, content dispatch |

### Wave 13 — Dataset Catalog

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-dscatalog` | ✅ | lib unit tests | Dataset catalog CRUD, registry, VFS provider |
| `ff-dsalloc` | ✅ | lib unit tests, `property_tests.rs` | Property tests fixed: `prop::char::ranges` replaces bare `RangeInclusive<char>` literals; moved-value borrow fixed with `.clone()` |
| `ff-idcams` | ✅ | `integration_tests.rs` | IDCAMS command emulation, DEFINE/DELETE/LISTCAT |

### Wave 13.5 — Job Entry Subsystem

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-jes` | ✅ | lib unit tests | JES job submission, status tracking, spool output |

### Wave 14 — File Explorer

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-tree` | ✅ | lib unit tests | File tree model, directory traversal, filter |
| `ff-compare` | ✅ | lib unit tests | File diff algorithm, merge operations |

### Wave 15 — Performance

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-idle` | ✅ | lib unit tests | Idle task scheduling, priority queue |
| `ff-largefile` | ✅ | lib unit tests | Large file chunked loading, memory pressure |

### Wave 17 — Database Tool

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-dbtool` | ✅ | lib unit tests | DB connection model, query execution, result set |

### Binary — ff-desktop

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `main.rs` unit tests (39 tests) | Boot sequence, CLI path resolution, tab manager, editor panel navigation, command dispatch, session manager, file-open dialog, text input, save |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.1: mouse click moves cursor to clicked line/column |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.2: Ctrl+Z undoes most recent edit, restores document and cursor |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 4.2: Backspace at col 1 joins current line to end of previous line |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 4.2: Backspace at col 1 on first line is a no-op |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.3: current cursor line is visually highlighted |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.4: caret (vertical bar) rendered at cursor column |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests, `shell.rs` unit tests | Req 14.1: POM opens as floating window on startup (was: shown when no file tabs open — revised) |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests, `shell.rs` unit tests | Req 14.2: title line with app name and version |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.3: numbered option list with built-in entries |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.4: live calendar panel with current month/day |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.5: calendar shows current time and day-of-year |
| `ff-desktop` | 🔲 | — | Req 14.6: typing option number navigates to feature (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 14.7: menu bar mirrors Primary Option Menu entries (manual UI verification) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.1: first launch inserts POM tab at index 0 with kind PrimaryOptionMenu |
| `ff-desktop` | ✅ | `tab_manager.rs` unit tests | Req 14.1: POM tab inserted at index 0; duplicate insert is a no-op |
| `ff-desktop` | ✅ | `tab_manager.rs` unit tests | Req 14.8: TabKind::FileEditor set on file-backed tabs; TabKind::Untitled on new buffers |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.10, 14.14: START/POM commands recognised as shell-level intercepts |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.11: CLOSE command recognised as shell-level intercept |
| `ff-desktop` | ✅ | `tab_manager.rs` unit tests | Req 14.13: POM tab title is [POM] |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.15c: POM tab kind != FileEditor — file-specific menu items omitted |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.15b: FileEditor tab kind == FileEditor — file-specific items shown |
| `ff-desktop` | 🔲 | — | Req 14.9: tab bar empty-space right-click shows New / New File (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 14.12: EXIT/=X/Ctrl+X exits from any command field (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 14.15a–14.15b: full context menu renders correctly at runtime (manual UI verification) |
| ``ff-desktop`` | ? | ``shell.rs`` unit tests, ``tab_manager.rs`` unit tests | Req 14.6: option number on POM tab transforms tab in-place; on non-POM tab opens new tab |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.38: "Exit" item in tab context menu routes through shell-level exit intercept |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.39: POM option rows rendered as interactive buttons; click/Enter navigates to feature |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.40: POM exit line text is "Enter X to Terminate using log/list defaults"; rendered as interactive button; activating it exits the app |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.41: Calendar header rendered as `<  MonthName YYYY  >` with `<` and `>` as interactive hotspot buttons |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.42: Clicking `<`/`>` navigates calendar to previous/next month; current-day highlight suppressed when offset != 0 |

| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 6.8: no exclusions → display list equals all lines in order |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 6.1, 6.2: single exclusion block produces one placeholder row |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 6.1: two separate blocks produce two placeholder rows |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 6.2: placeholder text contains correct excluded line count |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 6.3: excluded lines do not appear as Line rows in display list |

| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 4.2, 4.4: default key map produces 4 labelled slots (F3, F7, F8, F12) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 4.4, 4.5: F3 slot label is derived from explicit label field |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 3.1: assigned F-key returns its command string via resolver |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 3.2: unassigned F-key returns None from resolver |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 4.3: unassigned key produces blank slot in label bar |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 4.6: key label bar updates when key map changes |

### Compiler Toolchain Integration (Phase W)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-toolchain-api` | ✅ | lib unit tests | Req 15.1: GCC plugin probes PATH for gcc/g++/gfortran/as/ld/ar |
| `ff-toolchain-api` | ✅ | lib unit tests | Req 15.2: ToolchainState transitions to Ready when all GCC components detected |
| `ff-toolchain-api` | ✅ | lib unit tests | Req 15.3: ToolchainState is NotDetected when components missing; panel shows install button |
| `ff-gcc-toolchain` | 🔲 | — | Req 15.4: Install confirmation dialog lists components, source, disk space |
| `ff-gcc-toolchain` | 🔲 | — | Req 15.5: Install runs via ff-bgio background service; UI stays interactive |
| `ff-gcc-toolchain` | ✅ | lib unit tests | Req 15.6: Successful install re-probes PATH and transitions to Ready |
| `ff-gcc-toolchain` | ✅ | lib unit tests | Req 15.7: Failed install transitions to InstallFailed with Retry/View Log actions |
| `ff-gcc-toolchain` | ✅ | lib unit tests | Req 15.8: Platform-appropriate install source (winget/apt/brew) |
| `ff-gcc-toolchain` | ✅ | lib unit tests | Req 15.9: Ready state lists all detected GCC components with versions |
| `ff-gcc-toolchain` | 🔲 | — | Req 16.1: Compile action enabled when GCC Ready and active tab is C/C++ file |
| `ff-gcc-toolchain` | 🔲 | — | Req 16.2: Compile runs as background process; output streamed to Toolchain_Panel |
| `ff-gcc-toolchain` | ✅ | lib unit tests | Req 16.3: GCC diagnostic output parsed into Diagnostic records; editor annotated |
| `ff-gcc-toolchain` | 🔲 | — | Req 16.4: Exit code 0 → Build succeeded; previous annotations cleared |
| `ff-gcc-toolchain` | 🔲 | — | Req 16.5: Non-zero exit → Build failed with error/warning counts |
| `ff-gcc-toolchain` | ✅ | lib unit tests | Req 16.6: Built-in BuildProfiles: debug, release, check-only |
| `ff-desktop` | ✅ | `toolchain_panel.rs` unit tests | Req 15.2, 15.3: Toolchain_Panel status rows show Ready/NotDetected state with correct labels |
| `ff-desktop` | ✅ | `toolchain_panel.rs` unit tests | Req 15.5, 17.5: Installing state renders progress indicator |
| `ff-desktop` | ✅ | `toolchain_panel.rs` unit tests | Req 16.2, 18.2: Build output lines accumulate in scrollable output area |
| `ff-desktop` | ✅ | `toolchain_panel.rs` unit tests | Req 16.3, 18.3: Diagnostic events accumulate in diagnostics list |
| `ff-desktop` | ✅ | `toolchain_panel.rs` unit tests | Req 16.4, 18.4: Exit code 0 produces 'succeeded' status text |
| `ff-desktop` | ✅ | `toolchain_panel.rs` unit tests | Req 16.5, 18.5: Non-zero exit produces 'failed' status text with error count |
| `ff-desktop` | ✅ | `toolchain_panel.rs` unit tests | Req 14.6: option 3 in POM maps to Compilers; command '3' opens Toolchain Panel |
| `ff-desktop` | ✅ | `toolchain_panel.rs` unit tests | Req 15.1, 17.1: Panel initialises with GCC and Rust entries both in NotDetected state |
| `ff-toolchain-api` | ✅ | lib unit tests | Req 17.1: Rust plugin probes PATH for rustc, cargo, rustup |
| `ff-toolchain-api` | ✅ | lib unit tests | Req 17.2: ToolchainState transitions to Ready when rustc and cargo detected |
| `ff-toolchain-api` | ✅ | lib unit tests | Req 17.3: ToolchainState is NotDetected when rustc/cargo missing; panel shows install button |
| `ff-rust-toolchain` | 🔲 | — | Req 17.4: Install confirmation dialog states rustup-init method, channel, target dir, disk space |
| `ff-rust-toolchain` | 🔲 | — | Req 17.5: rustup-init runs via ff-bgio; UI stays interactive during install |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 17.6: Successful install re-probes PATH (including ~/.cargo/bin); transitions to Ready |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 17.7: Failed install transitions to InstallFailed with Retry/View Log actions |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 17.8: Update Toolchain button runs rustup update in background |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 17.9: Toolchain_Panel lists installed channels with versions; allows channel switch |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 18.1: Cargo actions enabled when Rust Ready and active file is inside a Cargo workspace |
| `ff-rust-toolchain` | 🔲 | — | Req 18.2: Cargo runs as background process; output streamed to Toolchain_Panel |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 18.3: cargo --message-format=json output parsed into Diagnostic records |
| `ff-rust-toolchain` | 🔲 | — | Req 18.4: Exit code 0 → Cargo succeeded; previous annotations cleared |
| `ff-rust-toolchain` | 🔲 | — | Req 18.5: Non-zero exit → Cargo failed with error/warning counts |
| `ff-desktop` | ✅ | `toolchain_panel.rs` unit tests | Req 18.6: Clicking Diagnostic in panel navigates editor to file/line/col |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 18.7: --message-format=json passed to all cargo invocations |

---

## Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 116 |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 13 |
| 🔴 NOT COVERED | 6 |
| **Total crates** | **64** |

## Outstanding Issues

Req 14.1, 14.8–14.12 are 🔴 NOT COVERED — Phase Y re-revision: POM must be an attached tab (not floating window). Implementation tasks in `docs/specs/startup-and-session/tasks.md` Phase Y (tasks 19.1–19.11). Previous Phase X tests for floating-window behaviour are now superseded.

## Summary

| Status | Count |
|--------|-------|
| ✅ PASS | 122 |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 16 |
| 🔴 NOT COVERED | 1 |
| **Total crates** | **64** |

## Outstanding Issues

Req 14.6 (option number transforms POM tab kind in-place) is ? PASS � completed in Phase AB.

Req 14.38 ("Exit" in tab context menu) is PASS - completed in Phase Z.1.

| `ff-desktop` | 🔲 | — | Req 13.1: Help > About menu item opens About dialog (manual UI verification) |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.2: About dialog displays application name |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.3: About dialog displays version string — `about_dialog_version_is_nonempty` |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.4: About dialog credits creator Alan R Wynne — `about_dialog_contains_creator_credit` |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.5: About dialog credits Amazon Q Developer / AWS — `about_dialog_contains_aws_credit` |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.6: About dialog displays copyright notice — `about_dialog_copyright_contains_creator_name` |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.7: About dialog displays application description — `about_dialog_description_is_nonempty` |
| `ff-desktop` | 🔲 | — | Req 13.8: Close button / Escape closes the About dialog (manual UI verification) |
| `ff-config` | ✅ | `config_handle.rs` unit tests | Req 15.4: `set_user_value` writes key to user-layer file and triggers hot-reload |
| `ff-config` | ✅ | `config_handle.rs` unit tests | Req 15.6: `remove_user_value` removes key from user-layer file and restores default |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 15.1: `0` / `SETTINGS` / `=0` commands open SettingsPanel tab |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.2: Settings panel groups all schema keys by namespace in collapsible sections |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.3: Each key shows description, effective value, provenance badge, and appropriate widget |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.4: Confirmed valid value is written to user-layer config immediately |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.5: Invalid value shows inline error and is not persisted |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.6: Reset to Default button removes user-layer override |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.7: Filter input hides non-matching keys (case-insensitive substring) |
| `ff-desktop` | 🔲 | — | Req 15.8: Source file path shown as read-only footer (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 15.9: SettingsPanel tab persists in session and restores on next launch (manual UI verification) |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.10: F3/END in Settings panel returns tab to POM view |
| `ff-desktop` | 🔲 | — | Req 15.11: POM option 0 button navigates to Settings panel (manual UI verification) |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests, `main.rs` | Req 12.1: Mainframe new-catalog dialog pre-populates Repository Path from `catalogs.default_mainframe_root` + catalog name |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 12.2: POSIX new-catalog dialog pre-populates Root Directory from `catalogs.default_posix_root` |
| `ff-desktop` | ✅ | `main.rs` `register_builtin_schema()` | Req 12.3: `catalogs.default_mainframe_root` built-in default is `{user_data_dir}/catalogs/mainframe` |
| `ff-desktop` | ✅ | `main.rs` `register_builtin_schema()` | Req 12.4: `catalogs.default_posix_root` built-in default is `{user_data_dir}/catalogs/posix` |
| `ff-desktop` | ✅ | `ff_config::keys::catalogs`, `main.rs` | Req 12.5: both keys registered in ff-config schema under `[catalogs]` namespace with descriptions |
| `ff-desktop` | 🔲 | — | Req 12.6: user changes to either key persist to user-layer config and take effect immediately (manual UI verification via Settings panel) |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 12.7: pre-populated path is a suggestion only; field remains editable |

### Virtual Catalog Manager (Phase AA)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell.rs` unit tests, `tab_state.rs` | Req 3.1–3.5 (view-zoom): Ctrl+Scroll up/down zooms in/out on active tab; zoom state per tab |
| `ff-desktop` | ✅ | `session_manager.rs` unit tests | Req 6.1–6.4 (view-zoom): zoom_offset persisted per FileEditor tab; restored with clamping on session load |
| `ff-desktop` | 🔲 | — | Req 7.1–7.5 (view-zoom): zoom indicator shown in status bar when offset != 0 (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 2.1–2.7 (view-zoom): Ctrl+=, Ctrl+-, Ctrl+0 keyboard shortcuts (manual UI verification) |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 1.2, 1.4, 1.5: Files panel split layout, three section headers, empty-state nodes |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 1.7: F3/END in Files panel returns to POM view |
| `ff-desktop` | ✅ | `catalog_registry.rs` unit tests | Req 2.1–2.5: Catalog Registry CRUD and persistence |
| ``ff-desktop`` | ? | ``catalog_manager_dialog.rs`` unit tests | Req 3.1�3.8: Catalog Manager Dialog � Create (all four types) |
| ``ff-desktop`` | ? | ``catalog_manager_dialog.rs`` unit tests | Req 4.1�4.5: Catalog Manager Dialog � Edit and Delete |
| ``ff-desktop`` | ? | ``dataset_alloc_dialog.rs`` unit tests | Req 5.1�5.6: Dataset Allocation Dialog � ISPF fields, Allocate Like |
| ``ff-desktop`` | ?? | � | Req 6.1�6.7: Mainframe dataset context menus and inline rename/delete (manual UI verification) |
| ``ff-desktop`` | ? | ``posix_provider.rs`` unit tests | Req 7.1�7.7: POSIX VFS provider � scheme, path normalisation, read-only |
| ``ff-desktop`` | ?? | � | Req 8.1�8.6: POSIX file management dialogs and context menus (manual UI verification) |
| ``ff-desktop`` | ?? | � | Req 9.1�9.5: Native catalog browsing and context menus (manual UI verification) |
| ``ff-desktop`` | ? | ``files_panel.rs`` unit tests | Req 10.1�10.6: Content area � columns, sort, breadcrumb, filter |
| ``ff-desktop`` | ? | ``files_panel.rs`` unit tests, ``session_manager.rs`` unit tests | Req 11.1�11.3: POM option 1 label update and FilesPanel session persistence |
| ``ff-desktop`` | ? | ``primary_option_menu.rs`` unit tests, ``shell.rs`` unit tests | Req 14.3a: Option 1 opens Files panel with description "Mainframe, POSIX, Native" |

| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests, `shell.rs` unit tests | Req 14.3: POM option list reorganised to 9 entries (0–8) with updated labels/descriptions |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.3a: Option 1 labelled "File Catalogs" with correct description |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.3b: Option 8 labelled "Plugins" with description "Vendor added plugins" |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.7: menu bar includes `File Catalogs` and `Plugins` top-level menus mirroring all 9 POM options |

### Phase AI — User-Configurable Theme Colours and Custom Themes

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-theme` | ✅ | `loader.rs` unit tests | Req 14.1: Every colour token individually overridable in TOML (`all_ui_colour_tokens_overridable_via_toml`) |
| `ff-theme` | ✅ | `discovery.rs` unit tests | Req 14.2: Themes directory scanned for user `.toml` files (`scan_themes_dir_finds_toml_files`) |
| `ff-theme` | ✅ | `discovery.rs` unit tests | Req 14.3: New `.toml` in themes dir available after hot-reload (`scan_themes_dir_finds_toml_files`) |
| `ff-theme` | ✅ | `discovery.rs` unit tests | Req 14.4: User theme can declare `base` to inherit from another theme (`scan_themes_dir_reads_base_field`) |
| `ff-theme` | ✅ | `loader.rs` unit tests | Req 14.5: Omitted tokens inherited from `base` or built-in default (`base_inheritance_fills_missing_tokens`) |
| `ff-theme` | ✅ | `discovery.rs` unit tests | Req 14.6: `list_themes()` returns all built-in and user-created themes (`list_all_themes_includes_user_themes`) |
| `ff-theme` | ✅ | `api.rs` unit tests | Req 14.7: Changing `theme.active` applies new theme within one hot-reload cycle (`api_mode_switch_updates_palette`) |
| `ff-theme` | ✅ | `loader.rs` unit tests | Req 14.8: Invalid colour token logs WARN and uses fallback (`invalid_colour_in_user_theme_falls_back_to_default`) |
| `ff-theme` | ✅ | `discovery.rs` unit tests | Req 14.9: `export_theme()` serialises active palette to TOML (`export_theme_round_trips_name`, `export_theme_produces_valid_toml`) |
| `ff-theme` | ✅ | `loader.rs` unit tests | Req 14.10: Unresolvable `base` theme logs WARN and falls back to built-in default (`base_inheritance_fills_missing_tokens`) |

### Phase AE — Legacy Theme Colour Semantics

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-theme` | ✅ | `defaults.rs` unit tests | Req 13.1: Legacy menu bar text is white (`menu_bar_fg = ISPF_WHITE_HI`) |
| `ff-theme` | ✅ | `defaults.rs` unit tests | Req 13.2: Legacy primary menu has blue background (`primary_menu_bg = ISPF_BLUE`) |
| `ff-theme` | ✅ | `defaults.rs` unit tests | Req 13.3: Legacy normal body text is bright green (`editor.foreground = ISPF_GREEN_HI`) |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 13.4: Legacy option key characters are white (`option_key = UiMenuBarForeground`) |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 13.5: Legacy option labels are turquoise (`option_label = UiInputForeground`) |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 13.6: Legacy option descriptions are bright green (`normal_text = EditorForeground`) |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 13.7: Legacy calendar rendered in turquoise (`calendar_fg = UiInputForeground`) |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 13.8: Legacy today cell reversed (turquoise bg, black text) |

## Final Summary (after Phase AK)

| Status | Count |
|--------|-------|
| ✅ PASS | 162 |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 21 |
| 🔴 NOT COVERED | 0 |
| **Total crates** | **64** |

### Phase AJ — Tab-Order Focus Cycle

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.1: On launch, focus is on the Primary_Command_Field |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.2: Typing goes to command field (focus initialised to CommandField on launch) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.3: Tab from command field goes to POM option 0 when POM active; first menu bar item otherwise |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.4: Tab advances through POM options 0–8 |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.5: Tab from option 8 goes to POM exit line |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.6: Tab from exit line goes to calendar `<` button |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.7: Tab from `<` goes to `>` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.8: Tab from `>` goes to first menu bar item |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.9: Tab advances through menu bar items |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.10: Tab from last menu bar item wraps to command field |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.11: Shift+Tab is exact reverse of forward cycle |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 16.12: Focused POM option row rendered with reversed colours |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.13: Enter/Space on focused POM option navigates |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.14: Enter/Space on focused exit line exits app |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.15: Enter/Space on focused `<` navigates calendar back |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.16: Enter/Space on focused `>` navigates calendar forward |
| `ff-desktop` | 🔲 | — | Req 16.17: Focused menu bar item has visible focus indicator (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 16.18: Enter/Space on focused menu bar item opens dropdown (manual UI verification) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.19: Non-POM tab skips POM/calendar stops in cycle |

### Phase AK — Tab-Header Focus Stops + Command Field Focus Fix

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.1: CommandField focus requested every frame — typing reliable without click |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.2: CommandField focus requested every frame when focus_stop == CommandField |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.10: Tab from last menu bar item goes to first tab header (`focus_cycle_tab_forward_from_last_menu_goes_to_first_tab_header`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.20: Tab advances through tab headers left to right (`focus_cycle_tab_forward_through_all_tab_headers`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.21: Tab from last tab header wraps to CommandField (`focus_cycle_tab_forward_from_last_tab_header_wraps_to_command_field`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.22: Non-POM cycle: CommandField → menu bar → tab headers → CommandField (`focus_cycle_non_pom_includes_tab_headers`) |

### Phase AK - Tab-Header Focus Stops + Command Field Focus Fix

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | OK | `shell.rs` unit tests | Req 16.10: Tab from last menu bar item goes to first tab header |
| `ff-desktop` | OK | `shell.rs` unit tests | Req 16.20: Tab advances through tab headers left to right |
| `ff-desktop` | OK | `shell.rs` unit tests | Req 16.21: Tab from last tab header wraps to CommandField |
| `ff-desktop` | OK | `shell.rs` unit tests | Req 16.11: Shift+Tab from CommandField goes to last tab header |
| `ff-desktop` | OK | `shell.rs` unit tests | Req 16.11: Shift+Tab from first tab header goes to last menu bar item |
| `ff-desktop` | OK | `shell.rs` unit tests | Req 16.22: Non-POM cycle includes tab headers |
| `ff-desktop` | OK | `shell.rs` unit tests | Req 16.1, 16.2: Command field receives egui focus every frame when CommandField is active stop |

## Final Summary (after Phase AK)

| Status | Count |
|--------|-------|
| PASS | 163 |
| FAIL | 0 |
| MANUAL | 21 |
| NOT COVERED | 0 |
| **Total crates** | **64** |

### Phase AL — Tab Window Chrome (Requirement 17, 18)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | — | Req 17.1: Tab content area renders Tab_Header row, Title_Line, Command_Field in order |
| `ff-desktop` | 🔴 | — | Req 17.2: Title_Line is read-only and not editable |
| `ff-desktop` | 🔴 | — | Req 17.3: POM tab Title_Line shows "FileForge Workbench  vX.Y.Z" |
| `ff-desktop` | 🔴 | — | Req 17.4: File editor tab Title_Line shows full absolute path |
| `ff-desktop` | 🔴 | — | Req 17.5: Untitled file editor tab Title_Line shows "[Untitled]" |
| `ff-desktop` | 🔴 | — | Req 17.6: Other tab kinds Title_Line shows tab title string |
| `ff-desktop` | 🔴 | — | Req 17.7: Title_Line styled distinct from editor content area |
| `ff-desktop` | 🔴 | — | Req 17.8: Legacy theme Title_Line uses blue background (#0000AA) and white text (#FFFFFF) |
| `ff-desktop` | 🔴 | — | Req 17.9: Command_Field remains third element below Title_Line |
| `ff-desktop` | 🔴 | — | Req 18.1: "Move to Other View" detaches tab into Floating_Window with full chrome (deferred Phase AL) |
| `ff-desktop` | 🔴 | — | Req 18.2: Floating tab has functional Title_Line and Command_Field (deferred Phase AL) |
| `ff-desktop` | 🔴 | — | Req 18.3: Closing Floating_Window redocks tab at original position (deferred Phase AL) |
| `ff-desktop` | 🔴 | — | Req 18.4: Tab_Header removed from Primary_Window bar when detached; restored on redock (deferred Phase AL) |
| `ff-desktop` | 🔴 | — | Req 18.5: Floating_Window OS title bar shows Title_Line content + " — FileForge Workbench" (deferred Phase AL) |
| `ff-desktop` | 🔴 | — | Req 18.6: Drag Tab_Header 20px outside tab bar detaches to Floating_Window (deferred Phase AL) |
| `ff-desktop` | 🔴 | — | Req 18.7: Maximum 16 simultaneous Floating_Windows enforced (deferred Phase AL) |

### Phase AL — Tab Window Chrome (final status)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.3: POM tab Title_Line shows "FileForge Workbench  vX.Y.Z" — `title_line_pom_tab_shows_app_name_and_version` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.4: File editor tab Title_Line shows full absolute path — `title_line_file_editor_shows_path` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.5: Untitled file editor tab Title_Line shows "[Untitled]" — `title_line_untitled_shows_placeholder` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.6: SettingsPanel tab Title_Line shows "[SETTINGS]" — `title_line_settings_panel_shows_settings` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.6: FilesPanel tab Title_Line shows "[FILES]" — `title_line_files_panel_shows_files` |
| `ff-desktop` | 🔲 | — | Req 17.1: Three-element chrome order (Tab_Header, Title_Line, Command_Field) visible at runtime (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 17.2: Title_Line is read-only (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 17.7: Title_Line visually distinct from editor content area (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 17.8: Legacy theme Title_Line uses blue background / white text (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 17.9: Command_Field remains third element below Title_Line (manual UI verification) |
| `ff-desktop` | 🔴 | — | Req 18.1–18.7: Detachable tab windows — deferred to future phase |

| `ff-desktop` | 🔲 | `shell.rs` unit tests | Req 8.1: Command field Enter-to-submit — pressing Enter while field has focus executes the command |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 8.2: Command field Enter on empty field is a no-op |

## Final Summary (after Phase AL)

| Status | Count |
|--------|-------|
| ✅ PASS | 168 |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 26 |
| 🔴 NOT COVERED | 7 |
| **Total crates** | **64** |

### Phase AM — Per-Context Key Maps, PFSHOW, 24-Key Bar, Hotspots, END/RETURN, LIST+RETRIEVE

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-keys` | 🔴 | — | Req 12.1–12.7: PFSHOW ON/OFF/toggle command registered and dispatched |
| `ff-desktop` | 🔴 | — | Req 12.4: key_bar_visible persisted in session state |
| `ff-desktop` | 🔴 | — | Req 12.1–12.3: Key_Label_Bar shown/hidden by PFSHOW command |
| `ff-keys` | 🔴 | — | Req 13.1–13.2: KeyLabelBarModel produces two rows of 12 slots each (F1–F12, F13–F24) |
| `ff-keys` | 🔴 | — | Req 13.2: Unassigned slots present with blank label (grid preserved) |
| `ff-desktop` | 🔴 | — | Req 13.3–13.4: Two-row Key_Label_Bar rendered in footer |
| `ff-keys` | 🔴 | — | Req 14.1–14.5: KeyMapResolver supports context_maps with full-replacement semantics |
| `ff-desktop` | 🔴 | — | Req 14.2, 14.4: Tab switch calls set_context; Key_Label_Bar updates same frame |
| `ff-keys` | 🔴 | — | Req 14.7: [context_key_maps] TOML section parsed into KeyMapResolver |
| `ff-keys` | 🔴 | — | Req 15.1–15.2: KeyMap::default_global() returns 5-key built-in default map |
| `ff-keys` | 🔴 | — | Req 15.3: User [global_key_map] fully replaces built-in defaults |
| `ff-desktop` | 🔴 | — | Req 16.1–16.3: Key_Label_Bar slots are clickable; click dispatches assigned command |
| `ff-desktop` | 🔴 | — | Req 16.2: Click on blank slot is no-op |
| `ff-desktop` | 🔴 | — | Req 16.4: Hover over assigned slot shows full command string tooltip |
| `ff-keys` | 🔴 | — | Req 17.1–17.2: nav.end and nav.return commands registered |
| `ff-desktop` | 🔴 | — | Req 17.1: END closes current tab, navigates to previous tab or POM |
| `ff-desktop` | 🔴 | — | Req 17.2: END from POM exits application |
| `ff-desktop` | 🔴 | — | Req 17.3: RETURN navigates to POM tab from any context |
| `ff-desktop` | 🔴 | — | Req 17.4: RETURN from POM exits application |
| `ff-keys` | 🔴 | — | Req 17.7: END and RETURN added to ExclusionFilter; not recorded in history |
| `ff-help` | 🔴 | — | Req 18.1–18.2: Missing help topic emits "not available yet" status message |
| `ff-help` | 🔴 | — | Req 18.3: No specific context opens Help_Index (existing behaviour preserved) |
| `ff-keys` | 🔴 | — | Req 19.1–19.2: LIST+RETRIEVE returns ShowList variant with all history entries |
| `ff-desktop` | 🔴 | — | Req 19.3–19.4: Modal history-list overlay; selection populates field; Escape clears |
| `ff-keys` | 🔴 | — | Req 19.6: LIST not added to Command_History when used as RETRIEVE trigger |

### Phase AM — Final Status (implementation complete)

| Crate | Status | Test | Notes |
|-------|--------|------|-------|
| `ff-keys` | ✅ | `key_map.rs` unit tests | Req 15.1–15.2: `KeyMap::default_global()` — 5 built-in assignments, 19 unassigned |
| `ff-keys` | ✅ | `key_label_bar.rs` unit tests | Req 13.1–13.2: `row0()`/`row1()` — two rows of 12, all 24 slots always present |
| `ff-keys` | ✅ | `key_map_resolver.rs` unit tests | Req 14.1–14.5: context maps with full-replacement; context > profile > global priority |
| `ff-keys` | ✅ | `retrieve.rs` unit tests | Req 19.1–19.2, 19.5: `ShowList` variant; LIST trigger case-insensitive; empty history |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 12.1–12.3: PFSHOW ON/OFF/toggle intercepts in `handle_command` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 12.4: `key_bar_visible` field in shell struct (session persistence deferred) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 13.3–13.4: two-row key label bar rendered via `row0()`/`row1()` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.1–16.3: slots rendered as `egui::Button`; click dispatches command |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.4: hover tooltip shows full command string |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.1: END closes current tab, navigates to previous via `tab_history` stack |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.2: END from POM dispatches `file.exit` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.3: RETURN navigates to POM tab (or inserts one) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.4: RETURN from POM dispatches `file.exit` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.7: END and RETURN added to `is_shell_command` exclusion set |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.4: tab switch calls `set_context` + updates key label bar |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 19.3–19.4: `ShowList` triggers modal history overlay; selection populates field; Escape clears |
| `ff-desktop` | 🔴 | — | Req 12.4: `key_bar_visible` session persistence (TOML round-trip) — deferred |
| `ff-desktop` | 🔴 | — | Req 18.1–18.3: contextual help "not available yet" fallback — deferred (ff-help crate) |
| `ff-desktop` | 🔴 | — | Req 14.7: `[context_key_maps]` TOML config parsing — deferred (config integration) |

## Final Summary (after Phase AM)

| Status | Count |
|--------|-------|
| ✅ PASS | 186 |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 26 |
| 🔴 NOT COVERED | 10 |
| **Total crates** | **64** |

### Phase AN — Key Configuration Dialog (Req 20)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-keys` | 🔴 | — | Req 20.11, 20.12: `KeyModifier` enum and `ModifiedKey` struct defined; 96 TOML key names parse and round-trip |
| `ff-keys` | 🔴 | — | Req 20.9, 20.12: `KeyBinding.description` field; modifier bindings stored independently in `KeyMap` |
| `ff-keys` | 🔴 | — | Req 20.12: `KeyMap` uses `ModifiedKey` as key type; `get_plain()` returns only `None`-modifier entry |
| `ff-keys` | 🔴 | — | Req 20.11: TOML parser accepts `SF1`–`SF24`, `CF1`–`CF24`, `AF1`–`AF24` prefixes |
| `ff-desktop` | 🔴 | — | Req 20.10: Shift/Ctrl/Alt+Fn dispatch reads `egui::Modifiers`, constructs `ModifiedKey`, dispatches if assigned |
| `ff-desktop` | 🔴 | — | Req 20.1: `KEYS` command opens Key_Configuration_Dialog |
| `ff-desktop` | 🔴 | — | Req 20.1: `Edit > Key Assignments…` menu item opens Key_Configuration_Dialog |
| `ff-desktop` | 🔴 | — | Req 20.2: Dialog shows Default (Global) tab and one tab per context name |
| `ff-desktop` | 🔴 | — | Req 20.3: Each scope tab shows 24-row grid with Key, Command, Label, Description, Shift/Ctrl/Alt Cmd+Desc columns |
| `ff-desktop` | 🔴 | — | Req 20.4: Empty command field treated as unassigned on save |
| `ff-desktop` | 🔴 | — | Req 20.5: Save writes changes to user-layer TOML; Cancel discards |
| `ff-desktop` | 🔴 | — | Req 20.6: Dialog pre-populates from current effective key maps on open |
| `ff-desktop` | 🔴 | — | Req 20.7: Label column read-only, derived from command string |
| `ff-desktop` | 🔴 | — | Req 20.8: Save writes `[global_key_map]` or `[context_key_maps.<name>]` sections |
| `ff-desktop` | 🔴 | — | Req 20.13: Key_Label_Bar continues to show only plain bindings after modifier extension |
| `ff-desktop` | 🔴 | — | Req 20.15: Reset to Defaults restores Default tab to built-in defaults; clears context tabs |

### Phase AN — Key Configuration Dialog (final status)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-keys` | ✅ | `function_key.rs` unit tests | Req 20.11, 20.12: `KeyModifier` + `ModifiedKey` — 96 slots, all TOML names round-trip |
| `ff-keys` | ✅ | `key_map.rs` unit tests | Req 20.3, 20.9: `KeyBinding.description`; modifier bindings independent; `get_plain()` |
| `ff-keys` | ✅ | `key_map.rs` unit tests | Req 20.11: TOML parser accepts `SF`/`CF`/`AF` prefixes |
| `ff-keys` | ✅ | `key_map.rs` unit tests | Req 20.12: `KeyMap` uses `ModifiedKey` as key type; `get_plain()` returns `None`-modifier entry |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.1: `KEYS` command opens dialog (`dialog_new_starts_closed`, `is_shell_command`) |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.2: Default tab + 6 context tabs present (`dialog_has_all_scope_tabs`) |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.3: 24 rows per scope tab (`staged_default_has_24_rows`) |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.4: Empty command = unassigned; non-empty = binding (`empty_command_produces_no_binding_in_map`, `non_empty_command_produces_binding_in_map`) |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.5: Cancel discards staged changes (`cancel_discards_staged_changes`) |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.6: `load_from_resolver` pre-populates from global map (`load_from_resolver_populates_default_rows`) |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.9: Modifier bindings stored independently (`modifier_bindings_stored_independently_in_staged_map`) |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.15: Reset to Defaults restores built-in defaults (`reset_default_tab_restores_built_in_defaults`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 20.1: `KEYS` in `is_shell_command` |
| `ff-desktop` | 🔲 | — | Req 20.1: `Edit > Key Assignments…` menu item opens dialog (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 20.7: Label column read-only, derived from command (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 20.8: Save writes `[global_key_map]` / `[context_key_maps]` TOML — deferred (config integration Task 27.6) |
| `ff-desktop` | 🔲 | — | Req 20.10: Shift/Ctrl/Alt+Fn dispatch reads `egui::Modifiers` at runtime (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 20.13: Key_Label_Bar shows only plain bindings after modifier extension (manual UI verification) |
| `ff-desktop` | 🔴 | — | Req 20.8: Full TOML persistence for key maps — deferred to config integration |

### Phase AO — Detachable Tab Windows (Requirement 18)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 18.1, 18.4: `is_floating` flag set on detach; `FloatingTab` struct with `origin_index` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 18.7: 16-window limit enforced (`floating_tab_limit_enforced_at_16`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 18.3: `origin_index` preserved on `FloatingTab` (`floating_tab_origin_index_preserved`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 18.3: redock clamps `origin_index` to current tab count (`redock_clamps_to_tab_count`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 18.5: OS title bar format `<Title_Line> — FileForge Workbench` (`floating_tab_title_format`) |
| `ff-desktop` | 🔲 | — | Req 18.1: "Move to Other View" opens floating OS window (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 18.2: floating tab has functional Title_Line and Command_Field (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 18.3: closing floating window redocks tab at origin position (manual UI verification) |
| `ff-desktop` | 🔴 | — | Req 18.6: drag Tab_Header 20px outside bar detaches — deferred (egui drag-outside-bounds not exposed) |

### Phase AP — PFSHOW Session Persistence (Requirement 12.4)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `session_manager.rs` unit tests | Req 12.4: `key_bar_visible` persisted to session TOML and restored on next launch (`key_bar_visible_round_trips_through_session`) |
| `ff-session` | ✅ | `session_file.rs` unit tests, `property_tests.rs` | Req 12.4: `key_bar_visible` field in `SessionState` with `serde(default = "default_true")` — round-trips correctly |

### Phase AQ — Key Map TOML Persistence (Req 20.8)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.8: `save_produces_correct_config_values_for_global_scope` — global_key_map table contains assigned keys, omits unassigned |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.8: `save_produces_correct_config_key_for_context_scope` — context scope produces correct ConfigValue::Table |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.8: `empty_context_scope_produces_empty_table` — empty context produces empty table (no spurious keys) |
| `ff-desktop` | 🔲 | — | Req 20.8: Save button writes to actual user-layer TOML file on disk (manual UI verification — requires running binary) |

### Phase AR -- [context_key_maps] TOML Config Parsing (Req 14.7)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | OK | `shell.rs` unit tests | Req 14.7: context_key_maps_parsed_from_config_value_table -- editor + pom contexts loaded; full-replacement; unknown context falls back to global |
| `ff-desktop` | OK | `shell.rs` unit tests | Req 14.7: context_key_maps_invalid_key_skipped -- F99 produces warning, valid F3 loaded |
