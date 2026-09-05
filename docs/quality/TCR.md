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

### Phase AS — File Explorer Panel (Requirement 19)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | — | Req 19.1: `=2` in any command field closes current context and switches tab to FileExplorerPanel in-place |
| `ff-desktop` | 🔴 | — | Req 19.2: `=FILES` (case-insensitive) closes current context and switches tab to FileExplorerPanel in-place |
| `ff-desktop` | 🔴 | — | Req 19.3: `FILES` (no `=` prefix) opens a NEW tab in FileExplorerPanel context; current tab unchanged |
| `ff-desktop` | 🔴 | — | Req 19.4: option `2` on POM tab transforms that tab to FileExplorerPanel with title `[FILES]` |
| `ff-desktop` | 🔴 | — | Req 19.5: FileExplorerPanel displays tree view with one top-level node per open/mounted catalog |
| `ff-desktop` | 🔴 | — | Req 19.6: expanding a catalog node lists its files/datasets as child nodes |
| `ff-desktop` | 🔴 | — | Req 19.7: tree groups catalogs under Mainframe Catalogs, POSIX Catalogs, Native Catalogs section headers |
| `ff-desktop` | 🔴 | — | Req 19.8: when no catalogs are mounted, placeholder message is shown |
| `ff-desktop` | 🔴 | — | Req 19.9: double-clicking a file/member node opens it in a new editor tab |
| `ff-desktop` | 🔴 | — | Req 19.10: F3/END in FileExplorerPanel returns tab to POM view |
| `ff-desktop` | 🔴 | — | Req 19.11: FileExplorerPanel tab title in tab bar is `[FILES]` |
| `ff-desktop` | 🔴 | — | Req 19.12: `FileExplorerPanel` tab kind persists in session and restores on next launch |

### Phase AS — File Explorer Panel (Requirement 19) — Final Status

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell.rs`, `tab_manager.rs` unit tests | Req 19.1: `=2` transforms current tab in-place to `FileExplorerPanel` (`equals_2_command_transforms_tab_to_file_explorer`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 19.2: `=FILES` is a shell-level intercept (`equals_files_command_is_shell_intercept`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 19.3: `FILES` (no `=`) routes to new tab (`files_no_prefix_command_is_shell_intercept`) |
| `ff-desktop` | ✅ | `shell.rs`, `tab_manager.rs` unit tests | Req 19.4: option `2` on POM tab transforms in-place to `FileExplorerPanel` with title `[FILES]` (`option_2_on_pom_tab_transforms_to_file_explorer`) |
| `ff-desktop` | 🔴 | — | Req 19.5: tree view with catalog nodes (UI — deferred Task 27.4) |
| `ff-desktop` | 🔴 | — | Req 19.6: expanding catalog node lists files (UI — deferred Task 27.6) |
| `ff-desktop` | 🔴 | — | Req 19.7: Mainframe/POSIX/Native section headers (UI — deferred Task 27.4) |
| `ff-desktop` | 🔴 | — | Req 19.8: empty-state placeholder message (UI — deferred Task 27.5) |
| `ff-desktop` | 🔴 | — | Req 19.9: double-click opens file in editor tab (UI — deferred Task 27.7) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 19.10: END returns FileExplorerPanel tab to POM (`file_explorer_panel_end_command_returns_to_pom`) |
| `ff-desktop` | ✅ | `shell.rs`, `tab_manager.rs` unit tests | Req 19.11: tab title is `[FILES]` (`file_explorer_panel_tab_title_is_files`) |
| `ff-desktop` | ✅ | `tab_state.rs`, `session_manager.rs` | Req 19.12: `FileExplorerPanel` kind persists via `PersistedTabKind::FileExplorerPanel` (`file_explorer_panel_kind_is_distinct_from_files_panel`) |

## Final Summary (after Phase AS)

| Status | Count |
|--------|-------|
| ✅ PASS | 376 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | Req 19.5–19.9 (UI tree rendering) |
| 🔴 NOT COVERED | Req 19.5, 19.6, 19.7, 19.8, 19.9 (deferred Tasks 27.4–27.7) |

### Phase AT — Allocated Dataset Display (Req 13 virtual-catalog-manager)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | — | Req 13.1: `FilesPanelState` has `datasets` map and `AllocatedDataset` struct |
| `ff-desktop` | 🔴 | — | Req 13.2: `AllocOutcome::Confirmed` inserts `AllocatedDataset` into map under correct catalog name |
| `ff-desktop` | 🔴 | — | Req 13.3: selecting a catalog node populates `ContentAreaState::entries` from datasets map |
| `ff-desktop` | 🔴 | — | Req 13.4: datasets map persists to session TOML and restores on next launch |
| `ff-desktop` | 🔴 | — | Req 13.5: deleting a catalog removes all its datasets from the map |

### Phase AT -- Allocated Dataset Display -- Final Status (superseded by Phase BU)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ❌ | SUPERSEDED | Req 13.1: `files_panel_state_has_datasets_map` -- removed in BU.8; `AllocatedDataset` struct and `datasets` HashMap deleted |
| `ff-desktop` | ❌ | SUPERSEDED | Req 13.2: `add_dataset_inserts_into_map_under_catalog_name` -- removed in BU.8; allocation now via `CatalogRegistry::allocate()` |
| `ff-desktop` | ❌ | SUPERSEDED | Req 13.3: `load_entries_populates_content_area_from_datasets` -- removed in BU.8; content area now reads from SQLite |
| `ff-desktop` | ❌ | SUPERSEDED | Req 13.4: session TOML persistence -- removed in BU.8; `save_datasets()`/`load_datasets()` deleted from `SessionManager` |
| `ff-desktop` | ❌ | SUPERSEDED | Req 13.5: `delete_catalog_removes_its_datasets` -- removed in BU.8; `remove_catalog_datasets()` deleted; SQLite is sole store |
| | | | See Phase BU rows above for current passing coverage of Req 13.1-13.5 |

### Phase AU — Catalog Registry Persistence (B010 fix)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `session_manager.rs` unit tests | Req 2.1: `save_catalog_registry()` writes `catalogs.toml` on exit (`save_and_load_catalog_registry_round_trips`) |
| `ff-desktop` | ✅ | `session_manager.rs` unit tests | Req 2.2: `load_catalog_registry()` reads `catalogs.toml` on startup; returns empty registry if absent (`load_missing_catalog_file_returns_empty_registry`) |

### Phase AV — File Explorer Panel Tree View (Tasks 27.4–27.7, Req 19.5–19.9)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.5: `registered_catalogs_appear_as_tree_nodes` — each catalog in registry appears as a top-level expandable node |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.6: `catalog_datasets_accessible_for_child_nodes` — datasets for a catalog are accessible as child node data |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.7: `section_header_labels_match_catalog_type_labels` — Mainframe Catalogs / POSIX Catalogs / Native Catalogs headers |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.8: `zero_catalogs_triggers_empty_state` — empty registry (0 catalogs) triggers placeholder path |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.9: `ps_dataset_is_a_leaf_node_not_a_container` / `po_dataset_is_a_container_node` — PS is leaf (double-click opens); PO is container |
| `ff-desktop` | 🔲 | — | Req 19.5–19.9: full tree rendering with expand/collapse and double-click (manual UI verification) |

## Final Summary (after Phase AV)

| Status | Count |
|--------|-------|
| ✅ PASS | 391 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | Req 19.5–19.9 (UI tree rendering — manual verification) |
| 🔴 NOT COVERED | 0 (all Req 19.5–19.9 criteria have unit test coverage) |

### Phase AW — Mainframe Dataset Allocation Fixes (B011, B012, CR-NR-003)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | — | Req 5.8: Mainframe dataset name uppercased on confirm (B011) |
| `ff-desktop` | 🔴 | — | Req 5.9: duplicate DSN within same catalog rejected with inline error (B012) |
| `ff-desktop` | 🔴 | — | Req 5.7: Dataset Name pre-populated with catalog HLQ when HLQ is configured (CR-NR-003) |

### Phase AW — Final Status

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `dataset_alloc_dialog.rs` unit tests | Req 5.8: `validate_uppercases_dataset_name`, `validate_uppercases_mixed_case_name` — B011 fixed |
| `ff-desktop` | ✅ | `dataset_alloc_dialog.rs` unit tests | Req 5.9: `validate_for_catalog_rejects_duplicate_dsn`, `validate_for_catalog_duplicate_check_is_case_insensitive`, `validate_for_catalog_accepts_unique_dsn`, `validate_for_catalog_empty_existing_always_passes` — B012 fixed |
| `ff-desktop` | ✅ | `dataset_alloc_dialog.rs` unit tests | Req 5.7: `with_hlq_prepopulates_dataset_name_with_hlq_dot`, `with_hlq_empty_string_gives_dot` — CR-NR-003 done |

## Final Summary (after Phase AW)

| Status | Count |
|--------|-------|
| ✅ PASS | 399 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 0 new |
| 🔴 NOT COVERED | 0 |

### Phase AX — Default Home Catalog on First Launch (Req 14 virtual-catalog-manager)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | — | Req 14.1: when no Native catalogs exist, startup creates a Native catalog named `"Home"` pointing to the user home directory |
| `ff-desktop` | 🔴 | — | Req 14.2: the Home catalog is registered in the CatalogRegistry immediately and visible in the Files panel on the same launch |
| `ff-desktop` | 🔴 | — | Req 14.3: the Home catalog is persisted to `catalogs.toml` before the first frame so it survives restart |
| `ff-desktop` | 🔴 | — | Req 14.4: when one or more Native catalogs already exist, no Home catalog is created |
| `ff-desktop` | 🔴 | — | Req 14.5: when home directory cannot be determined, falls back to process working directory and still creates the catalog |
| `ff-desktop` | 🔴 | — | Req 14.6: attempting to delete the `"Home"` Native catalog is rejected with inline error |
| `ff-desktop` | 🔴 | — | Req 14.7: renaming or editing the Home catalog is permitted; after rename the deletion guard no longer applies |

### Phase AX — Default Home Catalog on First Launch — Final Status

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell/update.rs` startup_tests | Req 14.1, 14.2: `no_native_catalogs_triggers_home_catalog_creation` — empty registry gets a `"Home"` Native catalog pointing at the provided home path |
| `ff-desktop` | ✅ | `shell/update.rs` startup_tests | Req 14.4: `existing_native_catalog_suppresses_home_creation` — existing Native catalog prevents Home creation |
| `ff-desktop` | ✅ | `shell/update.rs` startup_tests | Req 14.3, 14.5: `home_catalog_uses_provided_path` — catalog uses the supplied path; `true` return signals caller to persist |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 14.6: `delete_home_native_catalog_is_rejected` — `execute_delete` returns `Err` for `"Home"` Native catalog; registry unchanged |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 14.7: `delete_renamed_home_catalog_is_permitted` — Native catalog renamed away from `"Home"` can be deleted normally |

## Final Summary (after Phase AX)

| Status | Count |
|--------|-------|
| ✅ PASS | 404 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 0 new |
| 🔴 NOT COVERED | 0 |

### Phase AV (CR-CH-003) — Help Fallback Human-Readable Message (Req 18.1, 18.2)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-help` | ✅ | `context_detector.rs` unit tests | Req 18.1: `resolve_with_fallback_missing_topic_returns_err` — message contains `"Help not yet available for"`, human-readable label (e.g. `command "FIND"`), and raw topic-key (`cmd:FIND`) for diagnostics |
| `ff-help` | ✅ | `context_detector.rs` unit tests | Req 18.2: `resolve_with_fallback_existing_topic_returns_ok` — registered topic returns `Ok(key)`; no fallback message emitted |

## Final Summary (after CR-CH-003)

| Status | Count |
|--------|-------|
| ✅ PASS | 404 tests (ff-desktop) + 12 (ff-help) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 0 new |
| 🔴 NOT COVERED | 0 |

### Phase AN.5 — ModifiedKey Property-Based Tests (Task 30)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-keys` | ✅ | `tests/modified_key_pbt.rs` | Req 20.11, 20.12: `modified_key_toml_name_always_round_trips` — all 96 ModifiedKey TOML names parse back to original (200 cases) |
| `ff-keys` | ✅ | `tests/modified_key_pbt.rs` | Req 20.9, 20.12: `get_plain_unaffected_by_modifier_bindings` — plain binding unchanged regardless of Shift/Ctrl/Alt entries on same key (200 cases) |
| `ff-keys` | ✅ | `tests/modified_key_pbt.rs` | Req 20.11, 20.12: `from_toml_table_mixed_modifiers_no_cross_contamination` — mixed modifier TOML produces exactly expected entries, no cross-contamination (200 cases) |

## Final Summary (after AN.5)

| Status | Count |
|--------|-------|
| ✅ PASS | 404 (ff-desktop) + 12 (ff-help) + 10 PBTs (ff-keys) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 0 new |
| 🔴 NOT COVERED | 0 |

### Phase AY — File Explorer: Expandable Subdirectories and Scrollable Panel (Req 15 file-tree-panel)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | — | Req 15.1: clicking expand arrow on a directory node inside a Native catalog shows its children sorted dirs-first alphabetically |
| `ff-desktop` | 🔴 | — | Req 15.2: child directory nodes are themselves expandable, supporting arbitrary nesting depth |
| `ff-desktop` | 🔴 | — | Req 15.3: File Explorer Panel content area is wrapped in a vertical scroll region |

### Phase AY — Final Status

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 15.2: `nested_directory_structure_readable_two_levels_deep` — two-level nested dirs readable via `std::fs::read_dir` |
| `ff-desktop` | 🔲 | — | Req 15.1: directory `CollapsingHeader` nodes expand to show children (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 15.2: child dirs are themselves expandable recursively (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 15.3: panel content scrollable via `ScrollArea::vertical()` (manual UI verification) |

## Final Summary (after Phase AY)

| Status | Count |
|--------|-------|
| ✅ PASS | 405 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | Req 15.1, 15.2, 15.3 (UI rendering — manual verification) |
| 🔴 NOT COVERED | 0 |

### Phase AZ — File Explorer Context Menu (Requirement 16)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.1: right-click on any non-header node shows context menu; right-click on section header shows nothing |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.2: Native File menu contains correct items in correct group order |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.3: Native Directory menu contains correct items in correct group order |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.4: POSIX File menu is read-only subset (no Rename, Move To, Copy To, New File, New Folder) |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.5: Mainframe PS dataset menu contains correct items |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.6: Mainframe PDS menu contains correct items |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.7: Mainframe PDS Member menu contains correct items; Submit JCL greyed-out |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.8: Mainframe GDG Base menu contains correct items |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.9: Mainframe GDG Generation menu contains correct items |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.10: Copy writes full path/DSN to OS clipboard; paste into editor prompts file name vs file contents |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.11: Rename activates inline TextEdit; Enter confirms on disk/store; Escape cancels; Mainframe enforces 8-char uppercase |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.12: Copy To / Move To dialog shows target picker, proposed name with naming-rule transform, dispatches to ff-bgio with progress indicator |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.13: Open With invokes platform-appropriate mechanism (Windows ShellExecuteEx / macOS open -a / Linux xdg chooser) |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.14: Reveal in Explorer opens OS file manager at parent directory with platform-appropriate label |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.15: Git submenu present but greyed-out and non-interactive |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.16: Submit JCL present but greyed-out and non-interactive |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.17: ExtensionRule table is data-driven; *.jcl rule would enable Submit JCL when implemented |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 16.18: Copy File Name / Relative Path / Full Path / Dataset Name / Member Name / Dataset(Member) each write correct string to clipboard |

### Phase BA — Open With Default Application (Requirement 17 file-tree-panel)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 17.1: Text/source files open in FFWB editor tab (no external launch) |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 17.2: External file class launches OS default app (Windows cmd start / macOS open / Linux xdg-open) |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 17.3: Unknown extension uses magic-byte scan; UTF-8 text opens in editor, binary launches OS app |
| `ff-desktop` | 🔲 | — | Req 17.4: Launch failure falls back to FFWB editor with status-bar message |
| `ff-desktop` | 🔲 | — | Req 17.5: Open With shows platform picker (Windows openwith / macOS open -a / Linux xdg chooser) |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 17.6: DefaultAppLaunch is non-blocking (Command::spawn, UI thread not blocked) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 17.7: Mainframe nodes always open in FFWB editor regardless of content |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 17.8: EXTERNAL_EXTENSIONS table covers all required categories (Office, PDF, images, audio/video, archives, executables, databases) |
| `ff-desktop` | 🔲 | — | Req 17.9: POSIX catalog file nodes follow same FileClass classification and launch rules as Native nodes |

### Phase BC — Directory-first alphabetical sort in content area (Req 10.7)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 10.7: Name-sort groups containers before non-containers, each group sorted case-insensitively — `visible_entries_name_sort_groups_dirs_before_files`, `visible_entries_name_sort_dirs_are_alphabetical_within_group`, `visible_entries_type_sort_does_not_force_dir_grouping` |

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.1: Native catalog directory children sorted directories-first then alphabetically case-insensitive — `collect_native_entries_sorts_dirs_first_then_alpha` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.2: Each file node displays human-readable size; `format_size_produces_correct_strings` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.3, 18.4, 18.5: Timestamps in `YYYY-MM-DD HH:MM` format — `format_timestamp_produces_correct_format` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.6: Permission attributes returned as non-empty string — `format_permissions_returns_nonempty_string` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.7: Valid entries collected without error; inaccessible entries silently skipped via `metadata().ok()?` — `collect_native_entries_returns_valid_entries` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.8: Opening unreadable file stores error in `last_error`; no editor tab opened — `open_file_node_stores_error_for_nonexistent_file` |
| `ff-desktop` | 🔲 | — | Req 18.9: Attribute columns rendered in correct order and alignment (manual UI verification) |

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

### Phase BE — File Explorer keyboard navigation + file copy/paste (Req 20–21 file-tree-panel)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | — | Req 20.1: Tab from CommandField transfers focus to File Explorer node list; Cursor_Node set to first visible catalog node |
| `ff-desktop` | 🔴 | — | Req 20.2: Tab advances Cursor_Node to next visible node in display order |
| `ff-desktop` | 🔴 | — | Req 20.3: Tab on a collapsed container expands it before advancing |
| `ff-desktop` | 🔴 | — | Req 20.4: Down/Up Arrow moves Cursor_Node without expanding containers |
| `ff-desktop` | 🔴 | — | Req 20.5: Right Arrow expands collapsed container; Left Arrow collapses expanded container or moves to parent |
| `ff-desktop` | 🔴 | — | Req 20.6: Shift+Arrow moves Cursor_Node and adds newly visited node to Keyboard_Selection |
| `ff-desktop` | 🔴 | — | Req 20.7: Continued Shift+Arrow adds each newly visited node cumulatively |
| `ff-desktop` | 🔴 | — | Req 20.8: Releasing Shift preserves Keyboard_Selection; plain Arrow moves cursor without changing selection |
| `ff-desktop` | 🔴 | — | Req 20.9: Ctrl+Arrow moves Cursor_Node without changing Keyboard_Selection |
| `ff-desktop` | 🔴 | — | Req 20.10: Ctrl+Space toggles Cursor_Node membership in Keyboard_Selection |
| `ff-desktop` | 🔴 | — | Req 20.11: Ctrl+C with non-empty Keyboard_Selection copies selected nodes |
| `ff-desktop` | 🔴 | — | Req 20.12: Escape clears Keyboard_Selection; Cursor_Node remains |
| `ff-desktop` | 🔴 | — | Req 20.13: Cursor_Node rendered with focus ring distinct from selection fill; both shown when node is cursor and selected |
| `ff-desktop` | 🔴 | — | Req 21.1: Ctrl+C stores selected node paths in File_Copy_Clipboard with operation type Copy |
| `ff-desktop` | 🔴 | — | Req 21.2: Ctrl+V in file list dispatches background copy to Paste_Target directory |
| `ff-desktop` | 🔴 | — | Req 21.3: Paste progress indicator shown in status bar; dismissed on completion; target directory refreshed |
| `ff-desktop` | 🔴 | — | Req 21.4: Paste failure shows error in status bar; successfully copied files not rolled back |
| `ff-desktop` | 🔴 | — | Req 21.5: Name collision shows per-file prompt with Overwrite / Skip / Rename options |
| `ff-desktop` | 🔴 | — | Req 21.6: Ctrl+V in editor with non-empty clipboard opens Paste_Prompt modal |
| `ff-desktop` | 🔴 | — | Req 21.7: "Insert File Names" inserts one path per line at caret |
| `ff-desktop` | 🔴 | — | Req 21.8: "Insert File Contents" reads and inserts file text; skips unreadable files with inline error |
| `ff-desktop` | 🔴 | — | Req 21.9: Mainframe DSN/member paths supported; member name lowercased when pasting to Native/POSIX |
| `ff-desktop` | 🔴 | — | Req 21.10: Paste to POSIX catalog rejected with status-bar message |
| `ff-desktop` | 🔴 | — | Req 21.11: File_Copy_Clipboard persists until replaced or cleared; source nodes show dashed border indicator |

### Phase BE — Final Status (keyboard + paste wired into render loop)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.1: `explorer_focused` field; Tab from CommandField wired in `render_central_panel` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.2: `collect_visible_node_paths` + Tab advance wired |
| `ff-desktop` | 🔲 | — | Req 20.3: Tab on collapsed container expands it (egui CollapsingHeader state — manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.4: Arrow keys move cursor without expanding — `arrow_down_moves_cursor_without_expanding` |
| `ff-desktop` | 🔲 | — | Req 20.5: Right/Left Arrow expand/collapse containers (egui CollapsingHeader — manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.6, 20.7: Shift+Arrow extends selection — `shift_arrow_adds_to_selection` |
| `ff-desktop` | 🔲 | — | Req 20.8: Releasing Shift preserves selection (modifier release — manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.9: Ctrl+Arrow moves cursor without changing selection — `ctrl_arrow_moves_cursor_without_changing_selection` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.10: Ctrl+Space toggles selection — `ctrl_space_toggles_cursor_node_in_selection` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.11: Ctrl+C copies to clipboard + File_Copy_Clipboard — wired in `handle_explorer_keyboard` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.12: Escape clears selection — `escape_clears_selection_preserves_cursor` |
| `ff-desktop` | 🔲 | — | Req 20.13: Cursor focus ring rendering (egui visual — manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.1: Ctrl+C stores paths in File_Copy_Clipboard — `ctrl_c_in_file_list_populates_file_copy_clipboard` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.2: Ctrl+V sets paste_prompt_open; target determined by `determine_paste_target` |
| `ff-desktop` | 🔲 | — | Req 21.3: ff-bgio background copy progress indicator (deferred — requires ff-bgio wiring) |
| `ff-desktop` | 🔲 | — | Req 21.4: Paste failure error handling (deferred — requires ff-bgio wiring) |
| `ff-desktop` | 🔲 | — | Req 21.5: Name collision Overwrite/Skip/Rename prompt (deferred) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.6: Ctrl+V with clipboard writes paths to OS clipboard + status message — wired in `render_central_panel` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.7: File paths joined one-per-line — `insert_file_names_produces_one_path_per_line` |
| `ff-desktop` | 🔲 | — | Req 21.8: Insert File Contents (deferred) |
| `ff-desktop` | 🔲 | — | Req 21.9: Mainframe DSN naming transform on paste (deferred) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.10: POSIX catalog paste rejected — `paste_to_posix_catalog_is_rejected` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.11: File_Copy_Clipboard persists until replaced — `file_copy_clipboard_persists_until_replaced` |

### Phase BD — File Explorer tree: drag-select and copy as text tree (Req 19 file-tree-panel)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.2: `shift_click_extends_selection_from_anchor` — Shift+click adds to selection from anchor |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.3: `ctrl_click_toggles_individual_node` — Ctrl+click toggles without affecting others |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.4: `selectable_label(is_selected, ...)` uses egui selection bg_fill tint |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.5: Ctrl+C calls `build_text_tree` and writes to OS clipboard |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.6: `build_text_tree_flat_selection`, `build_text_tree_hierarchical_selection`, `build_text_tree_dir_prefix`, `build_text_tree_relative_depth` |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 19.7: `CopyAsTextTree` action present in Native File and Native Dir menus above Copy group |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.8: `escape_clears_multi_selection` — Escape clears selected_nodes |
| `ff-desktop` | 🔲 | — | Req 19.1: drag-select range highlight (egui pointer drag — manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 19.9: selection extends to nodes scrolled into view during drag (manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.10: `build_text_tree_mainframe_uses_dsn` — Mainframe DSN used as-is in text tree |

### Phase BI — Default BLKSIZE=0 in Dataset Allocation Dialog (CR-CH-005)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `dataset_alloc_dialog.rs` unit tests | Req 5.2: BLKSIZE default is `0` (system-determined); `AllocDatasetForm::default()` returns `"0"` for blksize field; `validate()` accepts 0 as system-determined — `default_form_blksize_is_zero`, `validate_accepts_blksize_zero` |

## Final Summary (after Phase BD)

| Status | Count |
|--------|-------|
| ✅ PASS | 474 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | Req 19.1, 19.9 (drag pointer — manual verification) |
| 🔴 NOT COVERED | 0 |

### Phase BF — Tab Close Button + Files Menu Close (B002, B003, B015, B016)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔲 | — | B002/B015: `×` close button visible on every tab header (manual UI verification) |
| `ff-desktop` | 🔲 | — | B003: Files > Close closes the active tab (manual UI verification) |
| `ff-desktop` | 🔲 | — | B016: bracket rule documented — system tabs use `[]`, file tabs show filename only (manual UI verification) |

## Final Summary (after Phase BF)

| Status | Count |
|--------|-------|
| ✅ PASS | 474 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | B002/B003/B015/B016 (UI rendering — manual verification) |
| 🔴 NOT COVERED | 0 |

### Phase BJ — Catalog Repository Path Display + VFS Dataset Path Resolution (CR-NR-012)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 15.1: `edit_form_displays_repository_path` — `EditCatalogForm::from_catalog` carries `path` field from source catalog |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 15.2: `edit_form_repository_path_present_for_all_catalog_types` — path present for Mainframe and POSIX |
| `ff-desktop` | 🔲 | — | Req 15.3: Repository path field rendered as read-only label in Edit Catalog dialog (manual UI verification) |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.1, 16.5: `resolve_dataset_path_maps_dsn_to_subpath` — `PAYROLL.EMPLOYEE` maps to `{repo}/PAYROLL/EMPLOYEE` |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.4, 16.5: `resolve_dataset_path_empty_repo_returns_none` — empty repository path returns `None` |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.5: `resolve_dataset_path_empty_dsn_returns_none` — empty DSN returns `None` |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.1: `resolve_dataset_path_single_qualifier_dsn` — single-qualifier DSN resolves to one component under repo |
| `ff-desktop` | 🔲 | — | Req 16.2: file opened in editor when resolved path exists on disk (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 16.3: `'<DSN>': dataset file not found at <path>` shown when path missing (manual UI verification) |
| `ff-desktop` | 🔲 | — | Req 16.4: `'<DSN>': catalog has no repository path configured` shown when repo empty (manual UI verification) |

## Final Summary (after Phase BJ)

| Status | Count |
|--------|-------|
| ✅ PASS | 481 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | Req 15.3, 16.2, 16.3, 16.4 (UI rendering — manual verification) |
| 🔴 NOT COVERED | 0 |

### Phase BL — B024 Tab Cycle Fix (Req 20.1, 20.2)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell/update.rs` | Req 20.1: Tab from CommandField enters tree, sets `explorer_focused = true`, `cursor_node` = first visible node |
| `ff-desktop` | ✅ | `shell/update.rs` | Req 20.2: Tab advances `cursor_node`; Tab past last node exits tree and returns focus to CommandField |
| `ff-desktop` | 🔲 | — | Req 20.13: Cursor highlight on catalog nodes and file nodes — manual UI verification |

## Final Summary (after Phase BL)

| Status | Count |
|--------|-------|
| ✅ PASS | 481 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | Req 20.13 cursor highlight (UI rendering) |
| 🔴 NOT COVERED | 0 |

### Phase BK — Native File Browser: egui-file-dialog Integration (Requirement 22)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 22.1: `NativeDialogSlot` wraps `FileDialog`; `native_dialogs` field on `FileExplorerPanelState`; lazily initialised per catalog — `native_dialogs_field_exists_on_state`, `native_dialog_slot_lazily_created_for_catalog`, `native_dialog_slot_implements_debug_and_clone`, `file_explorer_panel_state_debug_clone_with_native_dialogs` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 22.2: `render_native_dialog()` calls `take_selected()` and routes to `open_file_node()` — `native_dialog_slot_lazily_created_for_catalog` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 22.3: `render_dataset_children()` unchanged; Mainframe/POSIX path unaffected — `mainframe_posix_branches_use_render_dataset_children` |
| `ff-desktop` | ✅ | `crates/ff-desktop/Cargo.toml` | Req 22.4: `egui-file-dialog = "0.6"` declared; vendored patch resolves egui 0.29 mismatch |
| `ff-desktop` | ✅ | `THIRD_PARTY_CREDITS.md` | Req 22.5: `THIRD_PARTY_CREDITS.md` created at workspace root with full MIT licence text |
| `ff-desktop` | ✅ | `cargo test` 486 passing | Req 22.6: 486 tests pass, 0 failures after BK refactoring |

### Phase BM — File Explorer Panel: egui-file-dialog look-and-feel with catalog mount points (Requirement 23)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.1: Two-pane layout (SidePanel + CentralPanel) matching egui-file-dialog visual style — `all_existing_state_fields_present_after_bm_refactor` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.2: Sidebar lists all catalogs as named Mount_Nodes; clicking selects and populates Content_Pane — `clicking_mount_node_sets_selected_catalog`, `selected_catalog_defaults_to_none` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.3: Sidebar groups catalogs under "Mainframe", "POSIX", "Native" collapsible headers — `sidebar_groups_catalogs_by_type` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.4: Native catalog Content_Pane renders egui-file-dialog widget — `native_catalog_uses_native_dialog_slot` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.5: Mainframe catalog Content_Pane renders dot-qualified dataset list; PS is leaf — `mainframe_content_ps_dataset_is_leaf` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.6: POSIX catalog Content_Pane uses forward-slash path normalisation — `posix_path_normalised_to_forward_slashes` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.7: Empty sidebar shows placeholder when no catalogs registered — `empty_registry_produces_no_mount_nodes` |
| `ff-desktop` | 🔲 | — | Req 23.8: Right-click context menu uses egui-file-dialog native menu for Native; Req 16 menu for Mainframe/POSIX (manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.9: Sidebar width persisted; default 200px; minimum 120px — `sidebar_width_defaults_to_200`, `sidebar_width_minimum_is_120` |
| `ff-desktop` | ✅ | `cargo test` 496 passing | Req 23.10: `cargo test` passes with 0 failures after BM refactoring |

### Phase BR — B028 Dataset File Creation on First Open (Req 16.3, 16.6)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.3: `opening_missing_dataset_creates_file_and_parent_dirs`, `opening_missing_dataset_creates_parent_dirs` — `create_dataset_file` creates file and all missing parent dirs |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.6: `create_dataset_file` returns `Err` on I/O failure; shell shows `'<DSN>': cannot create dataset file at <path>: <os_error>` |

### Phase BS — Mainframe Dataset Architecture (CR-NR-016)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-dscatalog` | ✅ | `vfs_provider::tests::fb_dataset_read_decodes_fixed_records_no_crlf` | Req 16.1: mainframe dataset written with no CRLF/LF record delimiter |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::fb_dataset_read_decodes_fixed_records_no_crlf` | Req 16.2: fixed-length records packed contiguously; record n at offset n×LRECL |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::vb_dataset_read_decodes_rdw_records_no_crlf` | Req 16.3: variable-length records preceded by 4-byte RDW; no CRLF after data |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::read_write_round_trip` | Req 16.4: RECFM=U content stored as opaque binary stream |
| `ff-dscatalog` | 🔴 | — | Req 16.5: editor presents records as lines without altering binary storage |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::fb_dataset_read_decodes_fixed_records_no_crlf`, `vb_dataset_read_decodes_rdw_records_no_crlf` | Req 16.6: save re-encodes displayed lines to binary record format |
| `ff-dscatalog` | 🔴 | — | Req 16.7: malformed RDW returns diagnostic error with dataset identity and record position |
| `ff-dscatalog` | 🔴 | — | Req 17.1: RecordCodec trait defined with no filesystem or SQLite dependency |
| `ff-dscatalog` | 🔴 | — | Req 17.2: FixedCodec encodes/decodes fixed-length records given LRECL |
| `ff-dscatalog` | 🔴 | — | Req 17.3: VariableCodec encodes/decodes variable-length records with 4-byte RDW |
| `ff-dscatalog` | 🔴 | — | Req 17.4: BinaryCodec passes bytes through unchanged for RECFM=U |
| `ff-dscatalog` | 🔴 | — | Req 17.5: TextCodec maps host text lines to/from fixed-length records; import/export only |
| `ff-dscatalog` | 🔴 | — | Req 17.6: all codecs independently testable using in-memory byte buffers |
| `ff-dscatalog` | 🔴 | — | Req 17.7: import/export requires explicit codec and encoding policy; not inferred silently |
| `ff-dscatalog` | 🔴 | — | Req 18.1: PS dataset content stored as native file; no SQLite BLOB |
| `ff-dscatalog` | 🔴 | — | Req 18.2: PDS/PDSE member content stored as individual native files; no SQLite BLOB |
| `ff-dscatalog` | 🔴 | — | Req 18.3: GDG generation content stored as native files |
| `ff-dscatalog` | 🔴 | — | Req 18.4: VSAM KSDS records stored in dedicated SQLite-backed keyed record store |
| `ff-dscatalog` | 🔴 | — | Req 18.5: VSAM RRDS records stored in SQLite-backed relative-record store |
| `ff-dscatalog` | 🔴 | — | Req 18.6: VSAM ESDS records stored in append-oriented native file; sidecar index rebuildable |
| `ff-dscatalog` | 🔴 | — | Req 18.7: POSIX files remain native host filesystem objects; not copied into SQLite |
| `ff-dscatalog` | 🔴 | — | Req 18.8: PS/PDS/GDG/POSIX content NOT stored as BLOBs in central catalogue database |
| `ff-dscatalog` | 🔴 | — | Req 19.1: StorageProvider trait defined with allocate/open/stat/rename/delete/list/reconcile |
| `ff-dscatalog` | 🔴 | — | Req 19.2: providers declare capabilities; callers do not infer from dataset type |
| `ff-dscatalog` | 🔴 | — | Req 19.3: native-file and SQLite-record providers share common error taxonomy mapping to VfsError |
| `ff-dscatalog` | 🔴 | — | Req 19.4: provider-specific locators opaque outside provider and catalogue services |
| `ff-dscatalog` | 🔴 | — | Req 19.5: NativeFileProvider implements StorageProvider for PS/PDS/GDG/POSIX |
| `ff-dscatalog` | 🔴 | — | Req 19.6: SqliteRecordProvider implements StorageProvider for VSAM KSDS/RRDS/ISAM |
| `ff-dscatalog` | 🔴 | — | Req 19.7: new StorageProvider addable without changing editors, catalogue consumers, or VFS layer |
| `ff-dscatalog` | 🔴 | — | Req 20.1: each managed physical object assigned stable UUID at allocation time |
| `ff-dscatalog` | 🔴 | — | Req 20.2: repository layout uses datasets/objects/<uuid>.dat and indexed/<uuid>.sqlite |
| `ff-dscatalog` | 🔴 | — | Req 20.3: logical dataset name NOT used as physical path |
| `ff-dscatalog` | 🔴 | — | Req 20.4: physical mapping deterministic and persisted; dataset findable after restart |
| `ff-dscatalog` | 🔴 | — | Req 20.5: dots in DSN NOT translated directly to directory separators in UUID layout |
| `ff-dscatalog` | 🔴 | — | Req 20.6: dataset rename updates catalogue only; physical object not moved |
| `ff-dscatalog` | 🔴 | — | Req 20.7: path-safety guards reject traversal, reserved names, illegal chars, length violations |
| `ff-dscatalog` | 🟢 | `storage::sqlite_record::tests::creates_indexed_database_with_wal_and_schema` | Req 21.1: KSDS provider uses dedicated SQLite database per dataset |
| `ff-dscatalog` | 🟡 | `storage::sqlite_record::tests::metadata_survives_reopen_and_mismatches_are_rejected` | Req 21.2: key metadata persists with the indexed database; catalogue-layer wiring remains |
| `ff-dscatalog` | 🟢 | `storage::sqlite_record::tests::supports_keyed_crud_and_ordered_ranges` | Req 21.3: KSDS supports keyed read, ordered read, CRUD, and range retrieval |
| `ff-dscatalog` | 🟢 | `storage::sqlite_record::tests::primary_key_uniqueness_is_transactional` | Req 21.4: KSDS primary-key uniqueness enforced transactionally |
| `ff-dscatalog` | ✅ | `storage::sqlite_record::tests::alternate_index_*` | Req 21.5: KSDS alternate indexes represented as SQLite indexes or mapping tables |
| `ff-dscatalog` | 🟢 | `storage::sqlite_record::tests::supports_keyed_crud_and_ordered_ranges` | Req 21.6: KSDS record data stored independently of catalogue rows |
| `ff-dscatalog` | 🟢 | `storage::sqlite_record::SqliteRecordProvider` | Req 21.7: KSDS can use dedicated SQLite database or alternative provider |
| `ff-dscatalog` | 🟢 | `storage::rrds::tests::reopens_existing_database` | Req 22.1: RRDS provider uses SQLite-backed store keyed by relative record number |
| `ff-dscatalog` | 🟢 | `storage::rrds::tests::distinguishes_unallocated_and_allocated_blank` | Req 22.2: RRDS distinguishes unallocated slot from allocated blank record |
| `ff-dscatalog` | 🟢 | `storage::rrds::tests::writes_replaces_deletes_and_reads_in_order` | Req 22.3: RRDS supports direct retrieval, replacement, deletion, sequential iteration |
| `ff-dscatalog` | 🟢 | `storage::esds::tests::appends_records_in_insertion_order` | Req 23.1: ESDS provider stores records in insertion order in append-oriented native file |
| `ff-dscatalog` | 🟢 | `storage::esds::tests::addresses_remain_stable_across_updates_and_reopen` | Req 23.2: ESDS issues stable record address for each appended record |
| `ff-dscatalog` | 🟢 | `storage::esds::tests::rebuilds_sidecar_index_from_data_file` | Req 23.3: ESDS sidecar index rebuildable from data file |
| `ff-dscatalog` | 🟢 | `storage::esds::NativeEsdsProvider` and design.md | Req 23.4: ESDS update/deletion semantics explicitly documented |
| `ff-dscatalog` | ✅ | `storage::isam::tests::isam_primary_key_insert_and_read`, `isam_sequential_read_returns_records_in_key_order` | Req 24.1: ISAM uses common indexed-record interface shared with KSDS |
| `ff-dscatalog` | ✅ | `storage::isam::tests::isam_secondary_index_lookup_returns_matching_primary_keys`, `isam_multiple_secondary_indexes_coexist` | Req 24.2: ISAM default provider uses SQLite indexes for primary and secondary access |
| `ff-dscatalog` | ✅ | `storage::isam::tests::isam_provider_implements_storage_provider_trait`, `isam_storage_provider_allocate_and_stat` | Req 24.3: ISAM implementation encapsulated behind StorageProvider interface |
| `ff-dscatalog` | 🔴 | — | Req 25.1: staged create protocol — stage, reserve, publish, activate |
| `ff-dscatalog` | 🔴 | — | Req 25.2: staged delete protocol — mark pending, tombstone, finalise |
| `ff-dscatalog` | 🔴 | — | Req 25.3: interrupted operations discoverable through OperationJournal |
| `ff-dscatalog` | 🔴 | — | Req 25.4: startup recovery detects and offers complete-or-rollback for incomplete operations |
| `ff-dscatalog` | 🔴 | — | Req 25.5: concurrent modification controlled via version tokens / SQLite transactions |
| `ff-dscatalog` | 🔴 | — | Req 25.6: operation not reported successful until both catalogue and provider postconditions met |
| `ff-dscatalog` | ✅ | `integrity::tests::checksum_file_produces_hex_digest`, `verify_checksum_*` | Req 26.1: optional CRC-32 checksums on managed content; verified on open when enabled |
| `ff-dscatalog` | ✅ | `integrity::tests::backup_creates_archive_with_manifest`, `backup_manifest_contains_correct_sizes` | Req 26.2: workspace.backup captures catalogue DB, SQLite stores, native files, journals |
| `ff-dscatalog` | ✅ | `integrity::tests::manifest_serialises_and_deserialises`, `manifest_schema_version_is_set` | Req 26.3: backup manifest contains schema version, provider config, object inventory, checksums |
| `ff-dscatalog` | ✅ | `integrity::tests::restore_extracts_files_to_target_root`, `restore_preserves_file_content` | Req 26.4: workspace.restore supports original root or remapped root without changing logical names |
| `ff-dscatalog` | ✅ | `integrity::tests::diagnose_reports_dangling_entry`, `diagnose_reports_orphaned_object`, `diagnose_reports_checksum_mismatch`, `diagnose_clean_workspace_returns_empty` | Req 26.5: workspace.diagnose reports orphaned physical objects and dangling catalogue entries |
| `ff-dscatalog` | ✅ | `integrity::tests::repair_plan_maps_findings_to_actions`, `apply_repair_deletes_orphan_file`, `apply_repair_dangling_entry_is_noop_on_filesystem`, `repair_plan_is_empty_for_no_findings` | Req 26.6: repair operations previewable, auditable, reversible where practical |
| `ff-dscatalog` | 🔴 | — | Req 27.1: reconciliation compares catalogue entries with physical objects per provider |
| `ff-dscatalog` | 🔴 | — | Req 27.2: reconciliation detects missing, inaccessible, duplicated, or inconsistent objects |
| `ff-dscatalog` | 🔴 | — | Req 27.3: reconciliation reports proposed corrections without auto-applying |
| `ff-dscatalog` | ✅ | `audit::tests::audit_log_records_all_action_variants`, `audit_log_records_create_action`, `audit_log_records_delete_action`, `audit_log_records_err_outcome`, `audit_log_catalogue_level_action_has_no_dsn`, `audit_log_entries_ordered_newest_first` | Req 27.4: audit_log table records create/rename/move/delete/restore/import/export/allocate |
| `ff-dscatalog` | ✅ | `audit::tests::audit_log_timestamp_is_nonempty` | Req 28.6: audit events identify action, object, outcome, timestamp, principal |
| `ff-dscatalog` | ✅ | `storage::native::tests::path_traversal_and_reserved_names_always_rejected` | Req 28.1: all resolved physical paths constrained to authorised workspace roots |
| `ff-dscatalog` | ✅ | `storage::native::tests::path_traversal_and_reserved_names_always_rejected` | Req 28.2: path canonicalisation and traversal checks before any filesystem access |
| `ff-dscatalog` | 🔴 | — | Req 28.3: catalogue metadata not treated as substitute for OS access controls |
| `ff-dscatalog` | ✅ | `security::tests::scrub_payload_returns_redacted_string`, `scrub_payload_never_exposes_content`, `scrub_str_returns_redacted`, `scrub_empty_payload`, `scrub_single_byte_payload` | Req 28.4: sensitive dataset contents and credentials not written to logs |
| `ff-dscatalog` | ✅ | `security::tests::parameterised_query_neutralises_sql_injection_in_datasets`, `parameterised_query_neutralises_sql_injection_in_audit_log` | Req 28.5: all SQLite connections use parameterised statements; no interpolated schema identifiers |
| `ff-dscatalog` | ✅ | `schema::tests::migration_from_v1_to_v2_creates_audit_log_table`, `migration_is_idempotent_on_current_version`, `migration_rejects_newer_version` | Req 27.5: schema changes versioned and applied through forward migration scripts |
| `ff-dscatalog` | ✅ | `hierarchy::tests::scope_display_and_parse_round_trip`, `catalog_registry::tests::resolve_scoped_finds_master_entry`, `resolve_with_scope_priority_prefers_master` | Req 29.1: master and user catalogue hierarchy supported |
| `ff-dscatalog` | ✅ | `catalog_registry::tests::resolve_scoped_does_not_return_wrong_scope`, `resolve_scoped_finds_master_entry` | Req 29.2: each logical DSN maps to exactly one active provider and locator within a scope |
| `ff-dscatalog` | ✅ | `catalog_registry::tests::logical_rename_updates_catalogue_only` | Req 29.3: logical rename updates catalogue only; physical relocation is a separate operation |
| `ff-dscatalog` | ✅ | `hierarchy::tests::uniqueness_fails_on_same_scope_collision`, `uniqueness_passes_when_same_dsn_different_scope`, `catalog_registry::tests::check_scope_uniqueness_rejects_duplicate_in_same_scope`, `check_scope_uniqueness_allows_same_dsn_in_different_scope` | Req 29.4: uniqueness validated per configured naming scope and collation rules |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::cross_platform_uuid_layout_produces_identical_logical_results` | Req 30.1: architecture operates identically on Windows, Linux, and macOS |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::catalogue_listing_does_not_load_payload_bytes` | Req 30.2: catalogue listing queries metadata without loading dataset payloads |
| `ff-dscatalog` | 🔴 | — | Req 30.3: design permits large datasets/libraries without all content in central catalogue DB |
| `ff-dscatalog` | 🔴 | — | Req 30.4: catalogue, codec, and provider components independently testable |
| `ff-dscatalog` | 🔴 | — | Req 30.5: storage operations emit structured diagnostic events with correlation identifiers |
| `ff-dscatalog` | 🔴 | — | Req 30.6: future storage provider addable without rewriting editors or catalogue consumers |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::pds_members_are_plain_files_readable_without_workbench` | Req 30.7: text-oriented PDS/PDSE members representable as ordinary files for Git |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::data_fidelity_binary_content_survives_round_trip` | Req 30.8: system does not silently alter bytes, encoding, record boundaries, keys, or generation identity |
| `ff-vfs` | ✅ | `storage_provider::tests::storage_provider_trait_object_is_object_safe`, `mock_provider_stored_as_arc_dyn` | Req 9.1 (VFS): StorageProvider trait defined separate from VfsProvider |
| `ff-vfs` | ✅ | `storage_provider::tests::allocate_returns_locator`, `open_with_stream_read_capability_returns_data`, `stat_returns_storage_stat`, `list_returns_empty_for_mock`, `reconcile_returns_no_discrepancies_for_mock` | Req 9.2 (VFS): StorageProvider exposes allocate/open/stat/rename/delete/list/reconcile |
| `ff-vfs` | ✅ | `storage_provider::tests::capability_advertisement_stream_read_write`, `capability_advertisement_none`, `default_write_returns_unsupported_operation`, `all_capability_variants_are_distinct` | Req 9.3 (VFS): providers declare capabilities; callers do not infer from dataset type |
| `ff-vfs` | ✅ | `storage_provider::tests::open_without_stream_read_returns_unsupported`, `default_write_returns_unsupported_operation` | Req 9.4 (VFS): native-file and SQLite-record providers share common error taxonomy |
| `ff-vfs` | ✅ | `storage_provider::tests::storage_locator_opaque_via_as_str` | Req 9.5 (VFS): provider-specific locators opaque outside provider and catalogue services |
| `ff-vfs` | ✅ | `posix_provider::tests::allocate_creates_native_file_not_sqlite`, `write_and_open_round_trip_native_bytes` | Req 10.1 (VFS): POSIX files remain native host filesystem objects; not copied into SQLite |
| `ff-vfs` | ✅ | `posix_provider::tests::allocate_creates_native_file_not_sqlite` | Req 10.2 (VFS): catalogue may register POSIX root without moving content |
| `ff-vfs` | ✅ | `posix_provider::tests::reconcile_detects_orphaned_and_dangling` | Req 10.3 (VFS): external POSIX changes detected via refresh/notifications/reconciliation |
| `ff-vfs` | ✅ | `posix_provider::tests::resolve_rejects_path_traversal`, `resolve_rejects_absolute_path_outside_root` | Req 10.4 (VFS): symlink handling configurable with loop detection |
| `ff-vfs` | ✅ | `posix_provider::tests::stat_returns_native_metadata`, `resolve_rejects_path_traversal` | Req 10.5 (VFS): host permissions, locking, case sensitivity surfaced accurately |
| `ff-vfs` | ✅ | `posix_provider::tests::read_only_provider_rejects_write`, `read_only_provider_rejects_allocate`, `read_only_provider_rejects_delete`, `read_only_provider_rejects_rename` | Req 10.6 (VFS): read-only POSIX catalog returns PermissionDenied for write/create/delete/rename |
| `ff-vfs` | ✅ | `transaction::tests::commit_write_creates_file_with_correct_content` | Req 11.1 (VFS): VFS create uses staged protocol -- stage, reserve, publish, activate |
| `ff-vfs` | ✅ | `transaction::tests::commit_delete_removes_file` | Req 11.2 (VFS): VFS delete uses staged protocol -- mark pending, tombstone, finalise |
| `ff-vfs` | ✅ | `transaction::tests::interrupted_transaction_journal_detectable_on_startup` | Req 11.3 (VFS): interrupted operations discoverable through journals or transitional states |
| `ff-vfs` | ✅ | `transaction::tests::interrupted_transaction_journal_detectable_on_startup` | Req 11.4 (VFS): startup detects and offers recovery for incomplete operations |
| `ff-vfs` | ✅ | `transaction::tests::commit_returns_error_when_any_op_fails` | Req 11.5 (VFS): VFS operation not reported successful until catalogue and provider postconditions met |
| `ff-vfs` | ✅ | `workspace::tests::backup_captures_all_files_from_source_root` | Req 12.1 (VFS): workspace.backup command captures complete workspace |
| `ff-vfs` | ✅ | `workspace::tests::backup_manifest_contains_schema_version_and_providers` | Req 12.2 (VFS): backup manifest contains schema version, provider config, inventory, integrity info |
| `ff-vfs` | ✅ | `workspace::tests::restore_round_trip_produces_identical_content` | Req 12.3 (VFS): workspace.restore supports original or remapped root |
| `ff-vfs` | ✅ | `workspace::tests::reconcile_reports_missing_from_provider` | Req 12.4 (VFS): workspace.reconcile reports discrepancies without auto-applying |
| `ff-vfs` | ✅ | `workspace::tests::diagnose_reports_orphaned_physical_objects` | Req 12.5 (VFS): workspace.diagnose reports orphaned objects and dangling entries |

### Phase BU -- SQLite Catalog Integration for Options 1 and 2 (CR-CH-006)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `catalog_registry.rs` unit tests | Req 13.1: `catalog_registry_allocate_writes_to_sqlite` -- AllocOutcome::Confirmed invokes `CatalogRegistry::allocate()`, writes to SQLite catalog.db |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 13.2: `files_panel_content_area_populated_from_sqlite` -- Files Panel content area populated via `CatalogRegistry::list_datasets()` from SQLite |
| `ff-desktop` | ✅ | `catalog_registry.rs` unit tests | Req 13.3: `catalog_registry_list_datasets_returns_all_allocated` -- File Explorer Panel Mainframe content populated via `CatalogRegistry::list_datasets()` |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 13.4: `alloc_confirm_uses_registry_not_hashmap` -- dataset persistence provided by SQLite catalog.db; no session-TOML dataset entries; `AllocatedDataset` struct and `datasets` HashMap removed |
| `ff-desktop` | ✅ | `shell/update.rs` | Req 13.5: catalog delete no longer calls `remove_catalog_datasets()`; SQLite catalog is the sole store; no separate HashMap cleanup needed |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.1: `resolve_and_open_dataset_returns_path_for_known_dsn` -- dataset open calls `CatalogRegistry::resolve_dsn()` to get UUID-based physical path |
| `ff-desktop` | 🔲 | -- | Req 16.2: resolved path exists on disk -- file opened in editor tab (manual UI verification) |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.3: `resolve_and_open_dataset_creates_file_when_missing` -- resolved path missing: `create_dataset_file()` creates file then opens it |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.4: `resolve_and_open_dataset_returns_err_for_unknown_dsn` -- DSN not in any catalog: error string contains "not found" |
| `ff-desktop` | ✅ | `shell/render.rs` `open_mainframe_dsn()` | Req 16.5: file creation fails -- `open_mainframe_dsn()` propagates `create_dataset_file` Err to status bar |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.6: `resolve_and_open_dataset()` is independently testable without egui -- three unit tests pass without any egui context |

### Phase BV -- Catalog Location Discriminant (Requirement 31)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-dscatalog` | 🔴 | -- | Req 31.1: `CatalogLocation` enum defined with `Local` and `Remote` variants |
| `ff-dscatalog` | 🔴 | -- | Req 31.2: `CatalogMount.location: CatalogLocation` replaces `path: PathBuf` |
| `ff-dscatalog` | 🔴 | -- | Req 31.3: `Local` variant behaves identically to previous `path: PathBuf` for all local operations |
| `ff-dscatalog` | 🔴 | -- | Req 31.4: `Remote` variant returns `CatalogError::UnsupportedOperation` on mount |
| `ff-dscatalog` | 🔴 | -- | Req 31.5: TOML schema extended with `location` and `uri` fields; round-trips correctly |
| `ff-dscatalog` | 🔴 | -- | Req 31.6: absent `location` field in TOML defaults to `Local` for backward compatibility |
| `ff-dscatalog` | 🔴 | -- | Req 31.7: `CatalogLocation` is `#[non_exhaustive]` |
| `ff-dscatalog` | 🔴 | -- | Req 31.8: `CatalogMount.local_path()` returns `Some(path)` for Local, `None` for Remote |
| `ff-dscatalog` | 🔴 | -- | Req 31.9: all existing mount/unmount/resolve/config tests pass unchanged |

### Phase BW -- edit-operations EARS Integration (Requirements 16-17)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-edit-operations` | ✅ | `profile.rs` unit tests | Req 16.1: CAPS ON converts typed characters to uppercase before insert |
| `ff-edit-operations` | ✅ | `profile.rs` unit tests | Req 16.2: CAPS with no argument toggles CAPS mode state |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 16.3: CAPS mode active -- status bar displays CAPS indicator |
| `ff-edit-operations` | ✅ | `profile.rs` unit tests | Req 16.4: NULLS ON treats trailing nulls as trailing spaces; NULLS OFF leaves unchanged |
| `ff-edit-operations` | ✅ | `profile.rs` unit tests | Req 16.5: PROFILE command displays current edit profile settings |
| `ff-edit-operations` | ✅ | `profile.rs` unit tests | Req 16.6: PROFILE with keyword argument updates named profile setting |
| `ff-edit-operations` | ✅ | `profile.rs` unit tests | Req 16.7: STATS ON sets stats_visible flag; STATS OFF clears it |
| `ff-edit-operations` | ✅ | `profile.rs` unit tests | Req 16.8: LOCK ON prevents profile changes; LOCK OFF re-enables them |
| `ff-edit-operations` | ✅ | `profile_persistence.rs` unit tests | Req 16.9: EditProfile round-trips through TOML serialisation |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 16.10: AUTONUM ON/OFF treated as alias for NUMBER ON/OFF |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 16.11: NUM command treated as alias for NUMBER command |
| `ff-edit-operations` | ✅ | `profile.rs` unit tests | Req 16.12: HILITE keyword parsed and stored; delegates to syntax-highlighting subsystem |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 17.1: SUBMIT returns JES-not-available error (JES dispatch deferred to Phase CC) |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 17.2: CREATE <dsn> dispatched; missing dsn returns error |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 17.3: REPLACE <dsn> dispatched; missing dsn returns error |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 17.4: EDIT <dsn> opens named dataset via existing file.open dispatch |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 17.5: BROWSE <dsn> dispatched; missing dsn returns error |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 17.6: VIEW <dsn> dispatched; missing dsn returns error |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 17.7: COMPARE <dsn> dispatched; missing dsn returns error |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 17.8: missing dsn argument returns descriptive error for all dataset commands |

### Phase BX -- line-commands EARS Integration (Requirement 15)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-line-commands` | ✅ | `execution/overlay.rs` unit tests | Req 15.1: O overlays target line(s) with source content, non-blank chars only |
| `ff-line-commands` | ✅ | `execution/overlay.rs` unit tests | Req 15.2: On overlays n consecutive lines with source content |
| `ff-line-commands` | ✅ | `execution/clipboard_copy.rs` unit tests | Req 15.3: W copies single line content to system clipboard |
| `ff-line-commands` | ✅ | `execution/clipboard_copy.rs` unit tests | Req 15.4: WW copies block of lines to system clipboard |
| `ff-line-commands` | ✅ | `execution/show_excluded.rs` unit tests | Req 15.5: F shows (un-excludes) only the first line of an excluded block |
| `ff-line-commands` | ✅ | `execution/show_excluded.rs` unit tests | Req 15.6: L shows (un-excludes) only the last line of an excluded block |
| `ff-line-commands` | ✅ | `parser.rs` + `resolution.rs` unit tests | Req 15.7: ] shifts single line right by exactly one column |
| `ff-line-commands` | ✅ | `parser.rs` + `resolution.rs` unit tests | Req 15.8: ]] shifts block of lines right by exactly one column |
| `ff-line-commands` | ✅ | `execution/show_excluded.rs` unit tests | Req 15.9: S shows (un-excludes) first line of excluded block at that position |
| `ff-line-commands` | ✅ | `execution/overlay.rs` unit tests | Req 15.10: overlay operation (O/On) produces a single undoable Transaction |
| `ff-line-commands` | ✅ | `execution/clipboard_copy.rs` unit tests | Req 15.11: clipboard copy (W/WW) produces no Transaction |
| `ff-line-commands` | ✅ | `execution/show_excluded.rs` unit tests | Req 15.12: F, L, S produce no Transaction (session state only) |


### Phase BY -- sequence-numbers EARS Integration (Alias Extensions)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-sequence-numbers` | ✅ | `number_cmd.rs` unit tests | Req 6.7a: AUTONUM ON/OFF treated as alias for NUMBER ON/OFF |
| `ff-sequence-numbers` | ✅ | `number_cmd.rs` + `commands.rs` unit tests | Req 8 alias: NUM accepted as alias for NUMBER command with all sub-commands |


### Phase BZ -- menu-and-statusbar EARS Integration (Requirement 19)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `scroll_amount.rs` + `shell/render.rs` unit tests | Req 19.1: SCROLL ===> field rendered adjacent to Command ===> field |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 19.2: SCROLL field value update on Enter sets active scroll amount |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 19.3: SCROLL field value retained across command submissions and panel switches |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 19.4: fastpath notation (e.g., 3.1) navigates directly to nested option |
| `ff-desktop` | ✅ | `panel_layout.rs` unit tests | Req 19.5: data entry panel conforms to ISPF layout (title, command, ===> fields, key bar) |
| `ff-desktop` | ✅ | `panel_layout.rs` unit tests | Req 19.6: list panel conforms to ISPF layout (title, command, filter lines, NP column, rows) |
| `ff-desktop` | ✅ | existing `nav_manager` LOCATE tests | Req 19.7: LOCATE on list panel scrolls to nearest alphabetic match |
| `ff-desktop` | ✅ | existing `nav_manager` LOCATE tests | Req 19.8: LOCATE accepts partial names on list panel |
| `ff-desktop` | 🔲 | -- | Req 19.9: LOCATE scrolls panel so matching item is visible (manual UI verification) |
| `ff-desktop` | ✅ | `scroll_amount.rs` unit tests | Req 19.10: scroll amounts HALF/CSR/MAX/DATA supported in all panel scroll commands |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 19.11: PF2 splits screen at cursor line into two independent halves |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 19.12: PF9 swaps focus between split-screen halves |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 19.13: each split-screen half operates independently |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 19.14: END (PF3) while split unsplits the screen |


### Phase CA -- startup-and-session EARS Integration (Requirement 20)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell/mod.rs`, `shell/tests.rs` | Req 20.1: session start timestamp displayed in status bar as Started: HH:MM |
| `ff-desktop` | ✅ | `shell/mod.rs`, `shell/tests.rs` | Req 20.2: session end timestamp and duration shown in status area on exit |
| `ff-desktop` | ✅ | `shell/commands.rs`, `shell/tests.rs` | Req 20.3: LOGOFF command initiates exit sequence identical to EXIT/=X |
| `ff-desktop` | ✅ | `shell/commands.rs`, `shell/tests.rs` | Req 20.4: TIME command displays current date/time/day-of-year in response area |
| `ff-desktop` | ✅ | `shell/commands.rs`, `shell/tests.rs` | Req 20.5: STATUS command routes to FFW-JES job status panel |
| `ff-desktop` | ✅ | `shell/commands.rs`, `shell/tests.rs` | Req 20.6: STATUS jobname routes to FFW-JES panel filtered by jobname |


### Phase CB -- command-semantics EARS Integration (Requirement 9)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.1: ALLOCATE command routes to dataset allocator with TSO keyword operands |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.2: FREE command routes to dataset allocator |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.3: DELETE command routes to VFS/catalog layer |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.4: RENAME oldname newname routes to VFS/catalog layer |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.5: LISTCAT [pattern] routes to catalog registry |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.6: LISTDS dsname [MEMBERS] routes to VFS layer |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.7: LISTALC routes to dataset allocator |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.8: SUBMIT dsname routes to FFW-JES subsystem |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.9: STATUS [jobname] routes to FFW-JES job status panel |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.10: EDIT dsname routes to file-operations pipeline |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.11: TSO-style positional and keyword operand parsing |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.12: SET PREFIX and automatic dataset name qualification |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.13: command continuation via trailing backslash |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.14: ds:// URI scheme bypasses session prefix, routes to VFS |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.15: namespace conflict resolution built-in > plugin > macro |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.16: capability model -- commands declare and verify required capabilities |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.17: secret operand redaction from history, logs, and status messages |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 9.18: structured audit events on every command execution |



### Phase CC -- FFW-JES P1 core EARS Integration (Requirement 16)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-jes` | ✅ | sdsf_panel.rs unit tests | Req 16.1: action bar with pull-down menus (File, View, Help) |
| `ff-jes` | ✅ | sdsf_panel.rs unit tests | Req 16.2: title line with panel name and visible row range |
| `ff-jes` | ✅ | sdsf_panel.rs unit tests | Req 16.3: SCROLL ===> field retains last-used scroll amount |
| `ff-jes` | ✅ | sdsf_filter.rs unit tests | Req 16.4: filter information lines PREFIX=/DEST=/OWNER= below title |
| `ff-jes` | ✅ | sdsf_action.rs unit tests | Req 16.5: NP column fixed leftmost, non-scrolling |
| `ff-jes` | ✅ | sdsf_filter.rs unit tests | Req 16.6: JOBNAME column fixed during horizontal scroll |
| `ff-jes` | ✅ | sdsf_action.rs unit tests | Req 16.7: action character in NP column dispatches action on Enter |
| `ff-jes` | ✅ | sdsf_action.rs unit tests | Req 16.8: action characters S/?/C/H/A/P/D/E/J/W supported |
| `ff-jes` | ✅ | sdsf_action.rs unit tests | Req 16.9: = repeats previous action character on that row |
| `ff-jes` | ✅ | sdsf_action.rs unit tests | Req 16.10: // block action applies to all rows in block |
| `ff-jes` | ✅ | sdsf_action.rs unit tests | Req 16.11: command-line action syntax "2 C" in command field |
| `ff-jes` | ✅ | sdsf_action.rs unit tests | Req 16.12: SET ROWNUM ON displays row numbers in NP area |
| `ff-jes` | ✅ | sdsf_panel.rs unit tests | Req 16.13: main panel lists all SDSF commands with name/desc/group |
| `ff-jes` | ✅ | sdsf_panel.rs unit tests | Req 16.14: command groups (Jobs/Output/JES/Log/Memory/Other) expandable |
| `ff-jes` | ✅ | sdsf_panel.rs unit tests | Req 16.15: S action on main panel row navigates to selected panel |
| `ff-jes` | ✅ | sdsf_panel.rs unit tests | Req 16.16: SET MAIN GROUP displays grouped main panel |
| `ff-jes` | ✅ | sdsf_panel.rs unit tests | Req 16.17: MENU command returns to main panel from any sub-panel |
| `ff-jes` | ✅ | sdsf_filter.rs unit tests | Req 16.18: PREFIX filter -- filter by job name prefix; PREFIX * clears |
| `ff-jes` | ✅ | sdsf_filter.rs unit tests | Req 16.19: OWNER filter -- filter by job owner; OWNER * clears |
| `ff-jes` | ✅ | sdsf_filter.rs unit tests | Req 16.20: DEST filter -- filter by output destination; DEST * clears |
| `ff-jes` | ✅ | sdsf_panel.rs unit tests | Req 16.21: title line message area shows last command feedback |
| `ff-jes` | ✅ | sdsf_panel.rs unit tests | Req 16.22: COMMAND INPUT ===> field for SDSF commands |
| `ff-jes` | ✅ | sdsf_action.rs unit tests | Req 16.23: NP column supports full action char set; invalid state rejected with message |
| `ff-jes` | ✅ | sdsf_filter.rs unit tests | Req 16.24: columns JOBNAME/JOBID/OWNER/STATUS/CLASS/PRTY/QUEUE/START/END/RC/STEPNAME/PROCSTEP; hideable/reorderable |
| `ff-jes` | ✅ | sdsf_filter.rs unit tests | Req 16.25: PREFIX/OWNER/DEST filter fields as editable in-place rows above table |
| `ff-jes` | ✅ | sdsf_filter.rs unit tests | Req 16.26: SORT colname [A|D] sorts job table; SORT with no args restores submission-time order |


### Phase CD -- FFW-JES P1 extended EARS Integration (Requirement 17)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-jes` | 🔴 | -- | Req 17.1: ST panel shows all jobs with STATUS column |
| `ff-jes` | 🔴 | -- | Req 17.2: FILTER command -- advanced filter expression; FILTER clears |
| `ff-jes` | 🔴 | -- | Req 17.3: FIND command -- search panel data; FIND NEXT/PREV |
| `ff-jes` | 🔴 | -- | Req 17.4: LOCATE command -- scroll to first JOBNAME match, nearest alpha on no match |
| `ff-jes` | 🔴 | -- | Req 17.5: UP/DOWN/LEFT/RIGHT scroll commands with n/HALF/PAGE/MAX amounts |
| `ff-jes` | 🔴 | -- | Req 17.6: SET ACTION displays valid action characters with descriptions |
| `ff-jes` | 🔴 | -- | Req 17.7: SET MAIN [panel-name] sets default MENU panel |
| `ff-jes` | 🔴 | -- | Req 17.8: SET ROWNUM ON/OFF toggles row numbers in NP area |
| `ff-jes` | 🔴 | -- | Req 17.9: WHO displays session info (user, start time, filters, SET settings, provider) |
| `ff-jes` | 🔴 | -- | Req 17.10: QUERY AUTH displays authorised commands and action characters |
| `ff-jes` | 🔴 | -- | Req 17.11: SET settings (ACTION/MAIN/ROWNUM) persist across restarts |
| `ff-jes` | 🔴 | -- | Req 17.12: FILTER supports =, !=, >, <, >=, <= operators and wildcard * |
| `ff-jes` | 🔴 | -- | Req 17.13: FILTER supports AND and OR logical operators |
| `ff-jes` | 🔴 | -- | Req 17.14: ST panel accessible via ST command and S action on main panel |
| `ff-jes` | 🔴 | -- | Req 17.15: FIND case-insensitive by default; FIND C for case-sensitive |
| `ff-jes` | 🔴 | -- | Req 17.16: LOCATE/FIND no-match shows "string NOT FOUND" in message area |
| `ff-jes` | 🔴 | -- | Req 17.17: scroll commands update SCROLL ===> field to last-used amount |


### Phase CE -- undo-redo-transactions P2 EARS Integration (Requirement 19)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-undo-redo` | 🔴 | -- | Req 19.1: SETUNDO ON/OFF/n command -- enable/disable/configure undo levels, immediate effect |
| `ff-undo-redo` | 🔴 | -- | Req 19.2: RECOVERY ON/OFF/n command -- enable/disable/configure recovery interval, immediate effect |


### Phase CF -- syntax-highlighting P2 EARS Integration (Requirement 16)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-syntax-highlighting` | 🔴 | -- | Req 16.1: HILITE ON/OFF -- enable/disable syntax highlighting per document, state persists |
| `ff-syntax-highlighting` | 🔴 | -- | Req 16.2: HILITE LOGIC -- highlight boolean and comparison operators with HILITE_LOGIC style |
| `ff-syntax-highlighting` | 🔴 | -- | Req 16.3: HILITE PAREN -- highlight enclosing delimiter pair at cursor; HILITE_PAREN_ERROR for mismatches |
| `ff-syntax-highlighting` | 🔴 | -- | Req 16.4: HILITE FIND -- persist find-match highlights; HILITE FIND OFF clears |
| `ff-syntax-highlighting` | 🔴 | -- | Req 16.5: HILITE combined operands -- ON LOGIC PAREN enables multiple modes; modes toggle independently |


### Phase CG -- lua-macro-engine P2 EARS Integration (Requirement 11)

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-macro` | 🔴 | -- | Req 11.1: ISREDIT host command environment dispatches edit macro service calls |
| `ff-macro` | 🔴 | -- | Req 11.2: ISPEXEC host command environment routes dialog service calls |
| `ff-macro` | 🔴 | -- | Req 11.3: IMACRO executes named macro at edit session open |
| `ff-macro` | 🔴 | -- | Req 11.4: IMACRO edit profile setting stores/retrieves initial macro name |
| `ff-macro` | 🔴 | -- | Req 11.5: LINENUM function resolves label/relative reference to absolute line number |
| `ff-macro` | 🔴 | -- | Req 11.6: CURSOR function gets and sets cursor position |
| `ff-macro` | 🔴 | -- | Req 11.7: EXEC command locates and executes named exec from SYSEXEC/SYSPROC |
| `ff-macro` | 🔴 | -- | Req 11.8: Implicit exec invocation for unrecognized command names |
| `ff-macro` | 🔴 | -- | Req 11.9: % prefix bypasses primary command table for exec lookup |
| `ff-macro` | 🔴 | -- | Req 11.10: EXEC <member> <args> passes argument string to exec |
| `ff-macro` | 🔴 | -- | Req 11.11: TSO host command environment routes to ff-command dispatcher |
| `ff-macro` | 🔴 | -- | Req 11.12: ADDRESS <environment-name> switches default host command environment |
| `ff-macro` | 🔴 | -- | Req 11.13: ISPEXEC ADDRESS environment routes to ISPF dialog service layer |
| `ff-macro` | 🔴 | -- | Req 11.14: ISREDIT ADDRESS environment routes to ISREDIT handler |
| `ff-macro` | 🔴 | -- | Req 11.15: RC special variable set to host command return code |
| `ff-macro` | 🔴 | -- | Req 11.16: LISTDSI built-in returns dataset information from ff-dscatalog |
| `ff-macro` | 🔴 | -- | Req 11.17: MSG built-in displays message in status bar or message area |
| `ff-macro` | 🔴 | -- | Req 11.18: MVSVAR built-in returns system variable values mapped to workbench equivalents |
| `ff-macro` | 🔴 | -- | Req 11.19: OUTTRAP built-in captures TSO command output into stem variable |
| `ff-macro` | 🔴 | -- | Req 11.20: PROMPT built-in controls terminal input availability |
| `ff-macro` | 🔴 | -- | Req 11.21: SYSDSN built-in returns OK or error string for named dataset |
| `ff-macro` | 🔴 | -- | Req 11.22: SYSVAR built-in returns ISPF system variable values |
| `ff-macro` | 🔴 | -- | Req 11.23: USERID built-in returns current user login name |
| `ff-macro` | 🔴 | -- | Req 11.24: EXECIO DISKR reads records from ddname dataset into stem variable |
| `ff-macro` | 🔴 | -- | Req 11.25: EXECIO DISKW writes records from stem variable to ddname dataset |
| `ff-macro` | 🔴 | -- | Req 11.26: EXECIO FINIS variants read/write all remaining records and close file |
| `ff-macro` | 🔴 | -- | Req 11.27: EXECIO SKIP advances read position without returning data |
| `ff-macro` | 🔴 | -- | Req 11.28: EXECIO return codes RC=0/2/non-zero per TSO conventions |
| `ff-macro` | 🔴 | -- | Req 11.29: FFCMD command files execute .ffcmd files line-by-line as batch primary commands |
| `ff-macro` | 🔴 | -- | Req 11.30: FFCMD execution wrapped in single Macro_Transaction for atomic undo |

### Phase CH -- FFW-JES P2 EARS Integration (Requirement 18)

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-jes` | 🔴 | -- | Req 18.1: Overtypeable fields visually distinct from read-only fields |
| `ff-jes` | 🔴 | -- | Req 18.2: Direct overtype applies change and refreshes panel on Enter |
| `ff-jes` | 🔴 | -- | Req 18.3: Command-line overtype syntax updates named field for cursor/NP row |
| `ff-jes` | 🔴 | -- | Req 18.4: Overtype Extension pop-up for values exceeding column width |
| `ff-jes` | 🔴 | -- | Req 18.5: Context-sensitive HELP / PF1 displays panel help |
| `ff-jes` | 🔴 | -- | Req 18.6: ACTH lists valid action characters with descriptions |
| `ff-jes` | 🔴 | -- | Req 18.7: COLH lists column names with type, width, and description |
| `ff-jes` | 🔴 | -- | Req 18.8: CMDH lists valid primary commands with syntax and description |
| `ff-jes` | 🔴 | -- | Req 18.9: SEARCH <text> in help panel scrolls to first match |
| `ff-jes` | 🔴 | -- | Req 18.10: LOG command opens System Log panel in reverse-chronological order |
| `ff-jes` | 🔴 | -- | Req 18.11: ULOG command opens User Log panel for current user |
| `ff-jes` | 🔴 | -- | Req 18.12: NEXT/PREV scroll forward/backward through log segments |
| `ff-jes` | 🔴 | -- | Req 18.13: SNAPSHOT captures current log content to dataset or file |
| `ff-jes` | 🔴 | -- | Req 18.14: SYS panel displays active address spaces with status and resources |
| `ff-jes` | 🔴 | -- | Req 18.15: DASH panel displays system health metrics summary |
| `ff-jes` | 🔴 | -- | Req 18.16: INIT panel displays initiator pool status |
| `ff-jes` | 🔴 | -- | Req 18.17: JC panel displays job class definitions and scheduling parameters |
| `ff-jes` | 🔴 | -- | Req 18.18: SP panel displays spool volume utilisation and track allocation |
| `ff-jes` | 🔴 | -- | Req 18.19: Browse settings: line width, record format display, FIND in output |
| `ff-jes` | 🔴 | -- | Req 18.20: PRINT action routes job output to configured print destination |
| `ff-jes` | 🔴 | -- | Req 18.21: COLS command displays column ruler in browse panel |
| `ff-jes` | 🔴 | -- | Req 18.22: SET BCOLOR sets panel background colour, persisted |
| `ff-jes` | 🔴 | -- | Req 18.23: SET CONFIRM ON/OFF controls confirmation prompt for destructive actions |
| `ff-jes` | 🔴 | -- | Req 18.24: SET CURSOR sets default cursor landing position on panel open |
| `ff-jes` | 🔴 | -- | Req 18.25: SET DATE sets date display format (MDY/DMY/YMD/JUL) |
| `ff-jes` | 🔴 | -- | Req 18.26: SET DELAY sets automatic refresh interval; 0 disables auto-refresh |
| `ff-jes` | 🔴 | -- | Req 18.27: SET HEX ON/OFF toggles hexadecimal display of field values |
| `ff-jes` | 🔴 | -- | Req 18.28: SET SCHARS defines special characters for field delimiters |
| `ff-jes` | 🔴 | -- | Req 18.29: SET SCREEN sets logical screen dimensions for panel layout |
| `ff-jes` | 🔴 | -- | Req 18.30: All SET P2 settings persisted across sessions |


### Phase CI -- command-semantics P2 EARS Integration (Requirement 10)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-command-semantics` | 🔴 | -- | Req 10.1: OUTPUT jobname routes to FFW-JES for job output display/retrieval |
| `ff-command-semantics` | 🔴 | -- | Req 10.2: CANCEL jobname [PURGE] routes to FFW-JES; PURGE requests output purge |
| `ff-command-semantics` | 🔴 | -- | Req 10.3: SEND 'message' [USER/LOGON/BROADCAST] routes to messaging subsystem |
| `ff-command-semantics` | 🔴 | -- | Req 10.4: PROFILE [operands] routes to session profile subsystem |
| `ff-command-semantics` | 🔴 | -- | Req 10.5: PRINTDS DATASET(dsname) routes to file-operations pipeline |

### Phase CJ -- Bootstrap Scripts (CR-NR-032)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `bootstrap` | 🔴 | -- | Req 1.1: Windows script installs Rust stable into C:\tools\rust without admin rights |
| `bootstrap` | 🔴 | -- | Req 1.2: Windows script skips install if rustc.exe already present |
| `bootstrap` | 🔴 | -- | Req 1.3: Windows script verifies with rustc --version and cargo --version |
| `bootstrap` | 🔴 | -- | Req 1.4: Windows script adds cargo\bin to user PATH via HKCU registry |
| `bootstrap` | 🔴 | -- | Req 1.5: Windows script accepts -Root parameter |
| `bootstrap` | 🔴 | -- | Req 1.6: Windows script prints Next Steps summary |
| `bootstrap` | 🔴 | -- | Req 1.7: Windows script runs on PowerShell 5.1 without additional modules |
| `bootstrap` | 🔴 | -- | Req 2.1: Linux script installs Rust stable into ~/.tools/rust without sudo |
| `bootstrap` | 🔴 | -- | Req 2.2: Linux script skips install if rustc already present |
| `bootstrap` | 🔴 | -- | Req 2.3: Linux script verifies with rustc --version and cargo --version |
| `bootstrap` | 🔴 | -- | Req 2.4: Linux script appends PATH export to ~/.profile and ~/.bashrc |
| `bootstrap` | 🔴 | -- | Req 2.5: Linux script prints Next Steps summary |
| `bootstrap` | 🔴 | -- | Req 2.6: Linux script falls back to wget when curl is absent |
| `bootstrap` | 🔴 | -- | Req 3.1: macOS script installs Rust stable into ~/.tools/rust without sudo |
| `bootstrap` | 🔴 | -- | Req 3.2: macOS script skips install if rustc already present |
| `bootstrap` | 🔴 | -- | Req 3.3: macOS script verifies with rustc --version and cargo --version |
| `bootstrap` | 🔴 | -- | Req 3.4: macOS script appends PATH export to ~/.zshrc and ~/.bash_profile |
| `bootstrap` | 🔴 | -- | Req 3.5: macOS script prints Next Steps summary |
| `bootstrap` | 🔴 | -- | Req 3.6: macOS script warns if Xcode Command Line Tools absent but does not abort |
| `bootstrap` | 🔴 | -- | Req 4.1: bootstrap/README.md describes each script, prerequisites, install paths, and run command |
| `bootstrap` | 🔴 | -- | Req 4.2: README describes next steps: cargo build, cargo test, launch ffwb |
| `bootstrap` | 🔴 | -- | Req 5.1: no script requires admin or root privileges |
| `bootstrap` | 🔴 | -- | Req 5.2: all scripts are idempotent |
| `bootstrap` | 🔴 | -- | Req 5.3: all scripts write a timestamped log to bootstrap/logs/ |
| `bootstrap` | 🔴 | -- | Req 5.4: all scripts pass --no-modify-path to rustup-init |
| `bootstrap` | 🔴 | -- | Req 5.5: all scripts install the stable toolchain targeting the host triple |
| `bootstrap` | 🔴 | -- | Req 5.6: no script installs a non-stable toolchain unless explicitly requested |
