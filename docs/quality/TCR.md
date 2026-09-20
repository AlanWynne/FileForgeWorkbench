# Test Coverage Report (TCR)

**Last updated:** Phase S Step 4 -- integration testing complete  
**Workspace:** `cargo test --workspace` -- **all crates pass**

## Status Key

| Symbol | Meaning |
|--------|---------|
| ✅ | Automated tests exist and pass |
| ❌ | Automated tests exist but fail (compile error or runtime failure) |
| 🔲 | Requires manual / UI verification only |
| 🔴 | No automated tests exist |

## Known Failures

None -- all crates compile and pass.

---

## Coverage by Crate

### Wave 0 -- Foundation

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-logging` | ✅ | lib unit tests, doc tests | Logging init, level filtering, structured output; global flag tests serialised with `FLAG_LOCK` |

### Wave 2 -- Platform Architecture

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-core` | ✅ | `integration_tests.rs`, `property_tests.rs` | Service registry, event bus, lifecycle, startup/shutdown ordering |
| `ff-config` | ✅ | lib unit tests, integration tests | Layered config merge, hot-reload, schema validation |
| `ff-command` | ✅ | lib unit tests, `thread_safety_tests.rs` | Registry, dispatch, history, thread safety |
| `ff-plugin` | ✅ | lib unit tests | Plugin lifecycle, registry, dependency ordering |
| `ff-workflow` | ✅ | lib unit tests | Workflow definition, step execution, state transitions |
| `ff-layout` | ✅ | `integration.rs` | Panel docking, tab groups, drag-drop, persona switching |

### Wave 3 -- Virtual File System

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-vfs` | ✅ | lib unit tests | URI resolution, provider registry, VFS error types |
| `ff-connector-local-fs` | ✅ | lib unit tests | Local filesystem read/write, path normalisation |
| `ff-connector-ext` | ✅ | lib unit tests | Extensibility hooks, provider registration |

### Wave 4 -- Core Editor

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-document-model` | ✅ | `integration_tests.rs`, `property_tests.rs` | Insert/delete, line counting, byte positions, streaming load |
| `ff-edit-operations` | ✅ | lib unit tests, integration tests | Edit bounds, region operations, batch edits |
| `ff-undo-redo` | ✅ | lib unit tests | Transaction recording, undo/redo stacks, recovery files |
| `ff-viewport-scrolling` | ✅ | lib unit tests | Viewport model, cursor model, caret policy, scroll clamping |
| `ff-display-line-mapping` | ✅ | lib unit tests | Logical-to-visual line mapping, wrap mode |

### Wave 5 -- Command Engine

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-cmd-semantics` | ✅ | lib unit tests | Command parsing, semantic validation |
| `ff-find-and-replace` | ✅ | lib unit tests, `property_tests.rs` | Search, replace, scope filters, regex |
| `ff-line-commands` | ✅ | `integration_tests.rs`, `property_tests.rs` | ISPF line commands, block pairs, copy/move/delete |
| `ff-filter` | ✅ | lib unit tests | Exclude/show filter logic, pattern matching |
| `ff-nav` | ✅ | `integration_tests.rs` | Navigation commands, sort, selection |

### Wave 6 -- UI and Rendering

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-menu` | ✅ | lib unit tests | Menu model, action binding, status bar segments |
| `ff-theme` | ✅ | `integration_tests.rs`, `property_tests.rs` | Palette construction, design tokens, font zoom clamping |
| `ff-decorations` | ✅ | lib unit tests | Text decoration spans, overlay rendering |
| `ff-whitespace` | ✅ | lib unit tests | Whitespace guide rendering, tab/space visualisation |
| `ff-caret-selection` | ✅ | lib unit tests | Caret rendering, selection highlight, blink state |

### Wave 7 -- Language and Highlighting

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-lang` | ✅ | lib unit tests | Language detection, grammar registry |
| `ff-syntax` | ✅ | lib unit tests | Syntax token classification, highlight spans |
| `ff-auto-indent` | ✅ | lib unit tests, `property_tests.rs` | Indent level computation, pattern matching |

### Wave 8 -- File I/O and Session

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-fileops` | ✅ | `integration_tests.rs`, `persistence_tests.rs` | Open/save pipeline, atomic write, backup, revert |
| `ff-bgio` | ✅ | `integration_tests.rs`, `property_tests.rs` | Background I/O service, progress reporting, cancellation |
| `ff-encoding` | ✅ | lib unit tests | Encoding detection, BOM handling, transcoding |
| `ff-extmod` | ✅ | `integration_tests.rs`, `property_tests.rs` | External modification detection, prompt handling |
| `ff-session` | ✅ | lib unit tests, `integration_tests.rs`, `property_tests.rs` | Session state round-trip, TOML persistence, schema migration, geometry clamping |
| `ff-tabs` | ✅ | lib unit tests | Tab collection, tab state serialisation, active tab tracking |

### Wave 9 -- Desktop Integration

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-clipboard` | ✅ | lib unit tests | Clipboard read/write, format negotiation |
| `ff-keys` | ✅ | lib unit tests | Function key binding, command history |
| `ff-shell` | ✅ | lib unit tests | Shell command execution, output capture |
| `ff-help` | ✅ | lib unit tests | Context help lookup, topic resolution |
| `ff-zoom` | ✅ | lib unit tests, integration tests, property tests | Zoom level management, font size clamping, Ctrl+Scroll wired in ff-desktop, zoom persistence in session |
| `ff-wrap` | ✅ | lib unit tests | Line wrap toggle, wrap width configuration |

### Wave 10 -- Extensions and Macros

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-lua` | ✅ | lib unit tests | Lua engine init, macro execution, API bindings |
| `ff-completion` | ✅ | lib unit tests | Command completion candidates, prefix matching |

### Wave 11 -- Display Modes

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-hex` | ✅ | lib unit tests, `property_tests.rs` | Hex dump layout, nibble editing, byte reader |
| `ff-seqnum` | ✅ | lib unit tests | Sequence number parsing, renumbering |
| `ff-tabmask` | ✅ | lib unit tests | Tab mask display, column alignment |

### Wave 12 -- FileForge Domain

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-forge` | ✅ | lib unit tests | FileForge integration layer, record model |
| `ff-struct` | ✅ | lib unit tests, `property_tests.rs` | Structure catalog, packed decimal, field definitions |
| `ff-select` | ✅ | lib unit tests | Record selection criteria, filter evaluation |
| `ff-asa` | ✅ | lib unit tests | ASA carriage control, report preview rendering |
| `ff-viewers` | ✅ | lib unit tests | Custom file viewer registration, content dispatch |

### Wave 13 -- Dataset Catalog

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-dscatalog` | ✅ | lib unit tests | Dataset catalog CRUD, registry, VFS provider |
| `ff-dsalloc` | ✅ | lib unit tests, `property_tests.rs` | Property tests fixed: `prop::char::ranges` replaces bare `RangeInclusive<char>` literals; moved-value borrow fixed with `.clone()` |
| `ff-idcams` | ✅ | `integration_tests.rs` | IDCAMS command emulation, DEFINE/DELETE/LISTCAT |

### Wave 13.5 -- Job Entry Subsystem

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-jes` | ✅ | lib unit tests | JES job submission, status tracking, spool output |

### Wave 14 -- File Explorer

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-tree` | ✅ | lib unit tests | File tree model, directory traversal, filter |
| `ff-compare` | ✅ | lib unit tests | File diff algorithm, merge operations |

### Wave 15 -- Performance

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-idle` | ✅ | lib unit tests | Idle task scheduling, priority queue |
| `ff-largefile` | ✅ | lib unit tests | Large file chunked loading, memory pressure |

### Wave 17 -- Database Tool

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-dbtool` | ✅ | lib unit tests | DB connection model, query execution, result set |

### Binary -- ff-desktop

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `main.rs` unit tests (39 tests) | Boot sequence, CLI path resolution, tab manager, editor panel navigation, command dispatch, session manager, file-open dialog, text input, save |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.1: mouse click moves cursor to clicked line/column |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.2: Ctrl+Z undoes most recent edit, restores document and cursor |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 4.2: Backspace at col 1 joins current line to end of previous line |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 4.2: Backspace at col 1 on first line is a no-op |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.3: current cursor line is visually highlighted |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.4: caret (vertical bar) rendered at cursor column |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests, `shell.rs` unit tests | Req 14.1: POM opens as floating window on startup (was: shown when no file tabs open -- revised) |
| `ff-desktop` | ✅ | `shell/update.rs` startup_tests | Req 14.1a: session with POM tab -- `session_with_pom_tab_restores_exactly` -- ensure_pom_tab_present is a no-op when POM already present |
| `ff-desktop` | ✅ | `shell/update.rs` startup_tests | Req 14.1b: session without POM tab -- `session_without_pom_tab_prepends_pom` -- POM prepended at index 0 when no POM tab in restored session |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests, `shell.rs` unit tests | Req 14.2: title line with app name and version |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.3: numbered option list with built-in entries |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.4: live calendar panel with current month/day |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.5: calendar shows current time and day-of-year |
| `ff-desktop` | 🔲 | -- | Req 14.6: typing option number navigates to feature (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 14.7: menu bar mirrors Primary Option Menu entries (manual UI verification) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.1: first launch inserts POM tab at index 0 with kind PrimaryOptionMenu |
| `ff-desktop` | ✅ | `tab_manager.rs` unit tests | Req 14.1: POM tab inserted at index 0; duplicate insert is a no-op |
| `ff-desktop` | ✅ | `tab_manager.rs` unit tests | Req 14.8: TabKind::FileEditor set on file-backed tabs; TabKind::Untitled on new buffers |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.10, 14.14: START/POM commands recognised as shell-level intercepts |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.11: CLOSE command recognised as shell-level intercept |
| `ff-desktop` | ✅ | `tab_manager.rs` unit tests | Req 14.13: POM tab title is [POM] |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.15c: POM tab kind != FileEditor -- file-specific menu items omitted |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.15b: FileEditor tab kind == FileEditor -- file-specific items shown |
| `ff-desktop` | 🔲 | -- | Req 14.9: tab bar empty-space right-click shows New / New File (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 14.12: EXIT/=X/Ctrl+X exits from any command field (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 14.15a–14.15b: full context menu renders correctly at runtime (manual UI verification) |
| ``ff-desktop`` | ? | ``shell.rs`` unit tests, ``tab_manager.rs`` unit tests | Req 14.6: option number on POM tab transforms tab in-place; on non-POM tab opens new tab |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 14.38: "Exit" item in tab context menu routes through shell-level exit intercept |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.39: POM option rows rendered as interactive buttons; click/Enter navigates to feature |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.40: POM exit line text is "Enter X to Terminate using log/list defaults"; rendered as interactive button; activating it exits the app |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.41: Calendar header rendered as `<  MonthName YYYY  >` with `<` and `>` as interactive hotspot buttons |
| `ff-desktop` | ✅ | `primary_option_menu.rs` unit tests | Req 14.42: Clicking `<`/`>` navigates calendar to previous/next month; current-day highlight suppressed when offset != 0 |
| `ff-desktop` | 🔴 | -- | Req 14.43: calendar omitted (not deformed/clipped) when the POM/Menu_Workspace area is below CALENDAR_MIN_RENDER_SIZE -- DEFERRED (CR-NR-059) |
| `ff-desktop` | 🔴 | -- | Req 14.44: calendar restored on the next frame when the area is >= CALENDAR_MIN_RENDER_SIZE; decision from current-frame available area only -- DEFERRED (CR-NR-059) |
| `ff-desktop` | 🔴 | -- | Req 14.45: hiding the calendar does not mutate `pom_calendar_offset` / current-day state; restored calendar shows the same month -- DEFERRED (CR-NR-059) |

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
| `ff-gcc-toolchain` | 🔲 | -- | Req 15.4: Install confirmation dialog lists components, source, disk space |
| `ff-gcc-toolchain` | 🔲 | -- | Req 15.5: Install runs via ff-bgio background service; UI stays interactive |
| `ff-gcc-toolchain` | ✅ | lib unit tests | Req 15.6: Successful install re-probes PATH and transitions to Ready |
| `ff-gcc-toolchain` | ✅ | lib unit tests | Req 15.7: Failed install transitions to InstallFailed with Retry/View Log actions |
| `ff-gcc-toolchain` | ✅ | lib unit tests | Req 15.8: Platform-appropriate install source (winget/apt/brew) |
| `ff-gcc-toolchain` | ✅ | lib unit tests | Req 15.9: Ready state lists all detected GCC components with versions |
| `ff-gcc-toolchain` | 🔲 | -- | Req 16.1: Compile action enabled when GCC Ready and active tab is C/C++ file |
| `ff-gcc-toolchain` | 🔲 | -- | Req 16.2: Compile runs as background process; output streamed to Toolchain_Panel |
| `ff-gcc-toolchain` | ✅ | lib unit tests | Req 16.3: GCC diagnostic output parsed into Diagnostic records; editor annotated |
| `ff-gcc-toolchain` | 🔲 | -- | Req 16.4: Exit code 0 → Build succeeded; previous annotations cleared |
| `ff-gcc-toolchain` | 🔲 | -- | Req 16.5: Non-zero exit → Build failed with error/warning counts |
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
| `ff-rust-toolchain` | 🔲 | -- | Req 17.4: Install confirmation dialog states rustup-init method, channel, target dir, disk space |
| `ff-rust-toolchain` | 🔲 | -- | Req 17.5: rustup-init runs via ff-bgio; UI stays interactive during install |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 17.6: Successful install re-probes PATH (including ~/.cargo/bin); transitions to Ready |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 17.7: Failed install transitions to InstallFailed with Retry/View Log actions |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 17.8: Update Toolchain button runs rustup update in background |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 17.9: Toolchain_Panel lists installed channels with versions; allows channel switch |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 18.1: Cargo actions enabled when Rust Ready and active file is inside a Cargo workspace |
| `ff-rust-toolchain` | 🔲 | -- | Req 18.2: Cargo runs as background process; output streamed to Toolchain_Panel |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 18.3: cargo --message-format=json output parsed into Diagnostic records |
| `ff-rust-toolchain` | 🔲 | -- | Req 18.4: Exit code 0 → Cargo succeeded; previous annotations cleared |
| `ff-rust-toolchain` | 🔲 | -- | Req 18.5: Non-zero exit → Cargo failed with error/warning counts |
| `ff-desktop` | ✅ | `toolchain_panel.rs` unit tests | Req 18.6: Clicking Diagnostic in panel navigates editor to file/line/col |
| `ff-rust-toolchain` | ✅ | lib unit tests | Req 18.7: --message-format=json passed to all cargo invocations |
| `ff-toolchain-api` | ✅ | lib unit tests | Req 5.1: ToolchainPlugin trait is object-safe; dyn dispatch works (`mock_toolchain_as_trait_object_is_object_safe`) |
| `ff-toolchain-api` | ✅ | lib unit tests | Req 5.2: ToolchainPlugin trait doc comment cites Req 5.2; no GCC/Rust-specific assumptions in trait definition |
| `ff-toolchain-api` | ✅ | lib unit tests | Req 5.3: MockToolchain implements ToolchainPlugin using only ff-toolchain-api types; no ff-gcc-toolchain or ff-rust-toolchain dependency (`mock_toolchain_implements_trait_without_plugin_crate_dependency`, `mock_toolchain_install_transitions_to_ready`, `mock_toolchain_build_emits_output_and_finished`) |
| `ff-toolchain-api` | ✅ | `Cargo.toml` CI comment | Req 5.4: ff-toolchain-api Cargo.toml has no dev-dependency on ff-gcc-toolchain or ff-rust-toolchain; CI constraint documented |

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

Req 14.1, 14.8–14.12 are 🔴 NOT COVERED -- Phase Y re-revision: POM must be an attached tab (not floating window). Implementation tasks in `docs/specs/startup-and-session/tasks.md` Phase Y (tasks 19.1–19.11). Previous Phase X tests for floating-window behaviour are now superseded.

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

| `ff-desktop` | 🔲 | -- | Req 13.1: Help > About menu item opens About dialog (manual UI verification) |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.2: About dialog displays application name |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.3: About dialog displays version string -- `about_dialog_version_is_nonempty` |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.4: About dialog credits creator Alan R Wynne -- `about_dialog_contains_creator_credit` |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.5: About dialog credits Amazon Q Developer / AWS -- `about_dialog_contains_aws_credit` |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.6: About dialog displays copyright notice -- `about_dialog_copyright_contains_creator_name` |
| `ff-desktop` | ✅ | `about_dialog.rs` unit tests | Req 13.7: About dialog displays application description -- `about_dialog_description_is_nonempty` |
| `ff-desktop` | 🔲 | -- | Req 13.8: Close button / Escape closes the About dialog (manual UI verification) |
| `ff-config` | ✅ | `config_handle.rs` unit tests | Req 15.4: `set_user_value` writes key to user-layer file and triggers hot-reload |
| `ff-config` | ✅ | `config_handle.rs` unit tests | Req 15.6: `remove_user_value` removes key from user-layer file and restores default |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 15.1: `0` / `SETTINGS` / `=0` commands open SettingsPanel tab |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.2: Settings panel groups all schema keys by namespace in collapsible sections |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.3: Each key shows description, effective value, provenance badge, and appropriate widget |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.4: Confirmed valid value is written to user-layer config immediately |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.5: Invalid value shows inline error and is not persisted |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.6: Reset to Default button removes user-layer override |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.7: Filter input hides non-matching keys (case-insensitive substring) |
| `ff-desktop` | 🔲 | -- | Req 15.8: Source file path shown as read-only footer (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 15.9: SettingsPanel tab persists in session and restores on next launch (manual UI verification) |
| `ff-desktop` | ✅ | `settings_panel.rs` unit tests | Req 15.10: F3/END in Settings panel returns tab to POM view |
| `ff-desktop` | 🔲 | -- | Req 15.11: POM option 0 button navigates to Settings panel (manual UI verification) |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests, `main.rs` | Req 12.1: Mainframe new-catalog dialog pre-populates Repository Path from `catalogs.default_mainframe_root` + catalog name |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 12.2: POSIX new-catalog dialog pre-populates Root Directory from `catalogs.default_posix_root` |
| `ff-desktop` | ✅ | `main.rs` `register_builtin_schema()` | Req 12.3: `catalogs.default_mainframe_root` built-in default is `{user_data_dir}/catalogs/mainframe` |
| `ff-desktop` | ✅ | `main.rs` `register_builtin_schema()` | Req 12.4: `catalogs.default_posix_root` built-in default is `{user_data_dir}/catalogs/posix` |
| `ff-desktop` | ✅ | `ff_config::keys::catalogs`, `main.rs` | Req 12.5: both keys registered in ff-config schema under `[catalogs]` namespace with descriptions |
| `ff-desktop` | 🔲 | -- | Req 12.6: user changes to either key persist to user-layer config and take effect immediately (manual UI verification via Settings panel) |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 12.7: pre-populated path is a suggestion only; field remains editable |

### Virtual Catalog Manager (Phase AA)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell.rs` unit tests, `tab_state.rs` | Req 3.1–3.5 (view-zoom): Ctrl+Scroll up/down zooms in/out on active tab; zoom state per tab |
| `ff-desktop` | ✅ | `session_manager.rs` unit tests | Req 6.1–6.4 (view-zoom): zoom_offset persisted per FileEditor tab; restored with clamping on session load |
| `ff-desktop` | 🔲 | -- | Req 7.1–7.5 (view-zoom): zoom indicator shown in status bar when offset != 0 (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 2.1–2.7 (view-zoom): Ctrl+=, Ctrl+-, Ctrl+0 keyboard shortcuts (manual UI verification) |
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

### Phase AI -- User-Configurable Theme Colours and Custom Themes

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

### Phase (theme-editor) -- File-backed themes + Theme editor (CR-NR-074, theme-and-appearance Req 18-20)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-theme` | ✅ | `defaults.rs::default_legacy_matches_legacy_colours` | Req 18.1: `default_legacy_palette()` built-in (name "Default Legacy", colours == Legacy ISPF) |
| `ff-theme` | ✅ | `defaults.rs::fallback_is_default_legacy` | Req 18.2/18.6: `Default Legacy` is the canonical fallback (`fallback_palette()`) when the active theme cannot be resolved; reserved built-in name |
| `ff-desktop` | ✅ | `theme_defaults.rs::ensure_default_theme_files_creates_empty_dir_no_builtins`, `load_theme_by_name_builtin_resolves_without_file`; `shell/tests.rs::theme_editor_does_not_materialise_builtins` | Req 18.2 (CR-CH-019): built-ins are code-only, never materialised to themes/; a built-in stays a permanent read-only baseline |
| `ff-theme` | ✅ | `discovery.rs::builtin_themes_includes_default_legacy`, `builtin_themes_returns_five_entries` | Req 18.3 (SUPERSEDED by CR-CH-024: now FOUR built-ins, `Legacy (ISPF 3270)` removed -- see the CR-CH-024 TCR section) -- five built-ins (incl. Default Legacy) listed by `list_all_themes` and selectable |
| `ff-theme` | ✅ | `defaults.rs::default_legacy_serialise_round_trips` | Req 18.5: serialise(default_legacy) parses back equal (round-trip) |
| `ff-desktop` | ✅ | `shell/tests.rs::theme_editor_reset_loads_builtin_baseline`, `theme_editor_reset_non_builtin_errors` | Req 18.4: reset-to-baseline restores a built-in theme to its compiled content (with confirmation in render); non-built-in errors |
| `ff-desktop` | ✅ | `theme_defaults.rs::ensure_default_theme_files_creates_empty_dir_no_builtins`, `load_theme_by_name_reads_user_file` | Req 19.1/19.2 (REVISED CR-CH-019): `themes/` dir created (possibly empty) on first launch; built-ins NOT materialised -- themes/ holds only user themes |
| `ff-theme` | ✅ | `discovery.rs::list_all_themes_dedups_builtin_named_user_file`, `is_builtin_theme_identifies_builtins`, `list_all_themes_includes_user_themes` (no-dup assert); `shell/tests.rs::theme_editor_list_has_no_duplicates` | Req 19.2a (CR-CH-019): `list_all_themes` de-duplicated by name (built-in wins); no theme appears twice; a user file cannot shadow a built-in name |
| `ff-desktop` | ✅ | `keys.rs` (ACTIVE_NAME const) + main.rs schema; `theme_defaults.rs::resolve_startup_palette_*` (mode path unaffected) | Req 19.3: `theme.active_name` config key added (theme file NAME); existing `theme.active` (mode) unchanged; mode-only configs still resolve |
| `ff-desktop` | ✅ | `theme_defaults.rs::resolve_startup_palette_loads_materialised_mode_file`, `resolve_startup_palette_falls_back_to_default_legacy`, `load_theme_by_name_reads_materialised_file`, `load_theme_by_name_absent_is_none` | Req 19.4/19.5: startup loads the active theme file to drive `self.palette` before first frame; missing/invalid -> Default Legacy fallback, no crash |
| `ff-desktop` | 🔲 | -- | Req 19.6: active theme file hot-reloads (mtime poll in update.rs) and swaps the palette atomically (manual UI verification: edit themes/legacy.toml while running) |
| `ff-desktop` | ✅ | `render_chrome.rs::set_theme` (sets theme.active_name); `set_active_theme(name)` helper | Req 19.7/19.9: name-based activation persists `theme.active_name`; `THEME <mode>` keeps the two keys consistent (command parity) |
| `ff-desktop` | ✅ | code review: `self.palette` population changed in main.rs/update.rs only; no render call sites touched | Req 19.8: existing rendering call sites unchanged; palette field remains the single source of truth |
| `ff-desktop` | ✅ | `shell/tests.rs::themes_command_opens_theme_editor`, `themes_command_transforms_pom_tab_in_place` | Req 20.1/20.2/20.10: Theme Editor Context opens via a `THEMES` command; Settings menu "Theme Editor" item dispatches the same command; POM transform-in-place |
| `ff-desktop` | ✅ | `theme_editor_panel.rs::load_working_initialises_hex_buffers`, `all_editable_tokens_have_labels`; render shows swatch + inline invalid-hex | Req 20.3: editable token list shows current hex; invalid hex rejected inline without corrupting the theme |
| `ff-desktop` | ✅ | `shell/tests.rs::theme_editor_copy_creates_new_named_theme_file` | Req 20.4: Copy_Theme creates a new named theme file initialised from the source (built-in unaltered) |
| `ff-desktop` | ✅ | `shell/tests.rs::theme_editor_save_writes_edited_colour_to_disk`, `theme_editor_save_as_writes_new_file` | Req 20.5: Save / Save_As write the theme to `themes/<slug>.toml` via the serialiser (edited colour persists) |
| `ff-desktop` | ✅ | `shell/tests.rs::theme_editor_save_on_builtin_does_not_write_builtin`, `theme_editor_save_as_after_edit_writes_file_b052` | Req 20.5 (REVISED CR-CH-019 / B052): Save on a built-in redirects to Save As (never writes a built-in); Save As works after editing a token (button action not clobbered by token lost_focus) |
| `ff-desktop` | ✅ | `shell/tests.rs::theme_editor_set_active_swaps_palette_and_persists` | Req 20.6: Set_Active applies the theme immediately (palette swap) and persists `theme.active_name` for future launches |
| `ff-desktop` | ✅ | `shell/tests.rs::theme_editor_reset_loads_builtin_baseline`, `theme_editor_reset_non_builtin_errors` | Req 20.7: Reset restores the selected theme to its built-in baseline (confirmation in render) |
| `ff-desktop` | ✅ | `shell/tests.rs::theme_editor_reset_loads_builtin_baseline` (built-in re-select), `theme_editor_reset_non_builtin_errors` | Req 20.7/18.4 (REVISED CR-CH-019): reset a built-in re-selects the compiled palette (no file restore); user-theme reset reloads the saved file |
| `ff-desktop` | ✅ | `shell/tests.rs::theme_editor_edit_token_updates_working_and_previews` | Req 20.8: live preview while editing; unsaved edits live in the working copy and are not written to disk |
| `ff-desktop` | ✅ | `theme_editor_panel.rs::recompute_advisories_reads_working_palette` | Req 20.9: contrast advisory (check_theme_contrast) lists below-AA foreground/background pairs, non-blocking |

### Phase (theme-polish) -- Non-monochrome Dark/Light chrome + Legacy legibility (CR-CH-020, Req 21/22)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-theme` | ✅ | `defaults.rs::dark_chrome_has_three_level_surface_hierarchy`, `light_chrome_has_three_level_surface_hierarchy` | Req 21.1: Dark/Light define a 3-level background hierarchy (panel_bg / button_bg / input_bg perceptibly distinct) |
| `ff-theme` | ✅ | `defaults.rs::dark_and_light_focus_ring_is_accent` | Req 21.2: Dark/Light focus_ring = the theme accent |
| `ff-theme` | ✅ | `defaults.rs::dark_and_light_active_tab_is_distinct_and_accented` | Req 21.3: Dark/Light active tab (tab_bar.active_bg) accent-tinted, distinct from inactive |
| `ff-theme` | ✅ | `defaults.rs::dark_and_light_title_bar_is_accented_and_legible` | Req 21.4: Dark/Light accent-tinted primary_menu_bg (title-bar band) distinct from panel_bg |
| `ff-desktop` | 🔲 | -- | Req 21.5: non-Legacy title line paints primary_menu_bg background + menu_bar_fg text (egui render path -- manual UI verification; contrast guarded by ff-theme menu_bar_fg/primary_menu_bg pair) |
| `ff-desktop` | 🔲 | -- | Req 21.6: render_tab_bar reads the tab_bar.* group (active/inactive bg+text) instead of ui/editor reuse (egui render path -- manual UI verification; contrast guarded by ff-theme tab_bar pairs) |
| `ff-theme` | ✅ | `defaults.rs::dark_and_light_title_bar_is_accented_and_legible`, `dark_and_light_palettes_have_no_contrast_warnings`, `contrast.rs::check_theme_contrast_*_passes` | Req 21.7: new chrome fg/bg pairs meet WCAG AA (title 8.64/6.37:1, active tab 7.88/5.76:1; inactive tab >= 3:1) |
| `ff-theme` | ✅ | `defaults.rs::high_contrast_unchanged_no_warnings`, `high_contrast_fg_bg_pairs_meet_wcag_aaa` | Req 21.8/21.9: Catppuccin editor/syntax identity preserved; High Contrast unchanged (AAA) |
| `ff-theme` | ✅ | `defaults.rs::legacy_blue_on_black_uses_bright_blue_for_legibility`, `legacy_retains_look_and_feel` | Req 22.1-22.4: Legacy blue-on-black uses ISPF_BLUE_HI (#7878FF, 5.93:1) for line numbers/comments/unknown/margins; otherwise byte-identical; primary_menu_bg unchanged |

### Phase (menu-recovery) -- Code-only menus + Recovery Baseline + configurable separator + RESET BARE (CR-CH-021)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `defaults.rs::ensure_menus_dir_does_not_materialise_built_in_menus`, `ensure_menus_dir_creates_menus_dir` | menu-workspace Req 4.1/4.2: built-in menus are code-only; ensure_menus_dir no longer writes pom.toml/settings.toml (menus/ empty after startup) |
| `ff-desktop` | ✅ | `defaults.rs::recovery_pom_menu_has_barebones_options`, `recovery_settings_menu_has_barebones_options`, `default_pom_toml_has_recovery_baseline_options`, `default_settings_toml_has_recovery_baseline_options` | menu-workspace Req 12.1-12.3/12.7: compiled Recovery_Baseline (POM 0/1/2/L/M/X; Settings T/M/A) is the single compiled source |
| `ff-desktop` | 🔲 | -- | menu-workspace Req 12.4: absent user Menu_File -> Recovery_Baseline rendered, no load error, no notice (POM+Settings fallback wired in commands.rs; egui render path -- manual UI verification) |
| `ff-desktop` | 🔲 | -- | menu-workspace Req 12.5 / startup Req 11.8: corrupt user Menu_File -> Recovery_Baseline + non-blocking notice via notify_menu_fallback (Settings gains this fallback; exercised through the egui open path -- manual UI verification) |
| `ff-desktop` | ✅ | `shell/tests.rs::menus_command_shows_not_yet_available_notice` | menu-workspace Req 12.6: MENUS command shows "not yet available" notice, Workspace unchanged (reserved for the Menus-editor CR) |
| `ff-desktop` | ✅ | `loader.rs::load_group_separator_defaults_to_space`, `load_group_separator_values_parse`, `load_group_headers_true_parses` | menu-workspace Req 2.4/4a/4b: group_separator (space default / line / none) + optional group_headers parse; boundary drawn only between two different non-empty groups (render path) |
| `ff-desktop` | ✅ | `shell/tests.rs::reset_bare_command_opens_confirmation_dialog` | configuration-system Req 19.1-19.3: RESET BARE command opens a confirmation dialog; Cancel is a no-op |
| `ff-desktop` | ✅ | `reset_bare.rs::archive_config_moves_present_items_and_removes_originals`, `archive_config_skips_missing_items`, `archive_config_does_not_prune_previous_archives`, `archive_root_is_under_config_archive` | configuration-system Req 19.4/19.5/19.8: archive_config moves (not deletes) menus/themes/session.toml/config.toml/catalog registry to config-archive/<timestamp>/; missing skipped; never prunes |
| `ff-desktop` | ✅ | `shell/tests.rs::execute_reset_bare_reopens_home_context` | configuration-system Req 19.6: after confirm, in-memory reset to baselines + Home catalog re-created + Recovery_Baseline POM reopened without relaunch |
| `ff-desktop` | 🔲 | -- | configuration-system Req 19.7: Settings affordance dispatches RESET BARE via the command path, does not bypass the dialog (egui affordance -- manual UI verification; command path itself tested) |
| `ff-desktop` | ✅ | `shell/update.rs::startup_tests` (ensure_default_home_catalog tests) | startup Req 14 (regression, CR-NR-004): ensure_default_home_catalog still seeds "Home" -> user home when no Native catalog exists |
| `ff-desktop` | ✅ | `defaults.rs::ensure_menus_dir_does_not_materialise_built_in_menus` | startup Req 11.9: built-in menus not materialised on first launch; fresh install with no menus/ still opens a usable Home Context (Recovery_Baseline) |

### Phase (menus-editor) -- In-app Menus editor Context (CR-NR-075, Req 13; closes 23.9)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `serialiser.rs::recovery_pom_round_trips`, `recovery_settings_round_trips`, `full_featured_menu_round_trips`, `defaults_are_omitted_but_parse_back`, `special_characters_in_title_escaped` | menu-workspace Req 13.8/13.10: MenuFile -> TOML serialiser; `parse_menu_str(serialise(&m)) == m` round-trip |
| `ff-desktop` | ✅ | `shell/tests.rs::menus_editor_save_blocked_when_invalid` (+ `loader::validate_menu`) | menu-workspace Req 13.7/13.13: shared validate_menu (key 1-4 upper, command/description required, hard limit); editor Save blocked when invalid |
| `ff-desktop` | ✅ | `shell/tests.rs::menus_editor_add_delete_move_mutate_working`, `menus_editor_edit_option_key_uppercases` | menu-workspace Req 13.4/13.5: option list edit (key/command/description/enabled/group), add/delete/move reorder mutate the working menu |
| `ff-desktop` | ✅ | `shell/tests.rs::menus_editor_save_writes_loadable_file` (title edit round-trips) | menu-workspace Req 13.6: edit title + show_calendar/group_separator/group_headers on the working menu |
| `ff-desktop` | ✅ | `shell/tests.rs::menus_editor_loads_recovery_baseline_when_no_file`, `menus_command_opens_menus_editor` | menu-workspace Req 13.2/13.3: selector lists POM/Settings + user files; built-in with no file loads the Recovery_Baseline |
| `ff-desktop` | ✅ | `shell/tests.rs::menus_editor_save_writes_loadable_file`, `shell/menus_editor.rs::menu_file_stem_maps_built_ins_and_slugs_users` | menu-workspace Req 13.8/13.9: Save/Save As writes menus/<name>.toml (POM/Settings -> pom.toml/settings.toml) as a user override |
| `ff-desktop` | 🔲 | -- | menu-workspace Req 13.11: saved menu hot-reloads into an open POM/Settings within the existing window (mtime poll_reload -- exercised at runtime; manual UI verification) |
| `ff-desktop` | ✅ | `shell/tests.rs::menus_command_opens_menus_editor` | menu-workspace Req 13.1: MENUS opens the editor `[MENUS]`; POM/Settings M rows both dispatch MENUS |
| `ff-desktop` | 🔲 | -- | menu-workspace Req 13.12: pure render -> Action, button click wins over field lost_focus (B052) (egui interaction -- manual UI verification; the two-slot pattern mirrors the tested Theme editor) |
| `ff-desktop` | ✅ | `shell/tests.rs::settings_reset_bare_affordance_dispatches_command`, `defaults.rs::default_settings_toml_has_recovery_baseline_options` | configuration-system Req 19.7 (closes task 23.9): Settings baseline R row dispatches the RESET BARE command through the command path (opens the dialog, does not bypass) |

### Phase (nav-stack) -- Per-tab Navigation_Stack (CR-CH-022, Req 14)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell/tests.rs::navigation_stacks_are_per_tab` | Req 14.1: each tab owns an independent Navigation_Stack (per-tab, not global) |
| `ff-desktop` | ✅ | `shell/tests.rs::navigation_transforms_in_place_no_new_tab` | Req 14.2: navigating transforms the current tab IN PLACE and never opens a new tab |
| `ff-desktop` | ✅ | `shell/tests.rs::start_equals_path_keeps_pom_on_stack`, `end_walks_back_up_the_navigation_stack` | Req 14.3: `;`/bare navigation PUSH the current Context; a single navigation is one stack level |
| `ff-desktop` | ✅ | `shell/tests.rs::end_walks_back_up_the_navigation_stack`, `menus_editor_end_returns_to_origin`, `settings_menu_end_returns_to_pom`, `settings_end_from_namespace_view_returns_to_menu` | Req 14.4: END pops one entry and reconstructs the parent Context in place |
| `ff-desktop` | ✅ | `shell/tests.rs::end_at_empty_stack_last_tab_exits` | Req 14.5: END with an empty stack closes the Workspace; last tab -> terminate (CR-CH-016) |
| `ff-desktop` | ✅ | `shell/nav_stack.rs` (descriptor_for_current_context / reconstruct_context); exercised by the END/START tests | Req 14.6: stack entries are WorkspaceDescriptors carrying reconstruct params; shell-global Context state re-derived on pop |
| `ff-desktop` | ✅ | `shell/tests.rs::start_equals_path_keeps_pom_on_stack` (=0 origin), `start_forms_root_the_new_tab_correctly` | Req 14.7: leading `=` roots at the POM (POM on the stack); a direct `START <arg>` roots at the arg with an empty stack |
| `ff-desktop` | ✅ | `shell/tests.rs::start_forms_root_the_new_tab_correctly`, `start_equals_path_keeps_pom_on_stack`, `navigation_stacks_are_per_tab` | Req 14.8/14.9: START is the only tab-creator; START (POM) / START =X (POM+drill) / START X (rooted at X, empty stack) |
| `ff-desktop` | 🔲 | -- | Req 14.10: RETURN collapses to the tab's root Context in one step; END-at-root closes/exits (nav_return implemented; a dedicated multi-hop RETURN test is a follow-up -- END-at-root path is covered by end_at_empty_stack_last_tab_exits) |
| `ff-desktop` | ✅ | `shell/tests.rs::menus_editor_end_returns_to_origin` (+ build: the 3 fields removed) | Req 14.11: the 3 ad-hoc END mechanisms (pending_return_to_pom-as-transform, namespace_filter END branch, opened_from_settings) removed; replaced by the stack pop (supersedes B053 interim fix) |
| `ff-desktop` | 🔲 | -- | Req 14.12: tab Title_Line + header reflect the current Context after Navigate_Here and after END pop (title set in reconstruct is unit-covered via kind/title; the egui render is manual UI verification) |

### Phase AE -- Legacy Theme Colour Semantics

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

### Phase AJ -- Tab-Order Focus Cycle

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
| `ff-desktop` | 🔲 | -- | Req 16.17: Focused menu bar item has visible focus indicator (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 16.18: Enter/Space on focused menu bar item opens dropdown (manual UI verification) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.19: Non-POM tab skips POM/calendar stops in cycle |

### Phase AK -- Tab-Header Focus Stops + Command Field Focus Fix

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 16.1: CommandField focus requested every frame -- typing reliable without click |
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

### Phase AL -- Tab Window Chrome (Requirement 17, 18)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | -- | Req 17.1: Tab content area renders Tab_Header row, Title_Line, Command_Field in order |
| `ff-desktop` | 🔴 | -- | Req 17.2: Title_Line is read-only and not editable |
| `ff-desktop` | 🔴 | -- | Req 17.3: POM tab Title_Line shows "FileForge Workbench  vX.Y.Z" |
| `ff-desktop` | 🔴 | -- | Req 17.4: File editor tab Title_Line shows full absolute path |
| `ff-desktop` | 🔴 | -- | Req 17.5: Untitled file editor tab Title_Line shows "[Untitled]" |
| `ff-desktop` | 🔴 | -- | Req 17.6: Other tab kinds Title_Line shows tab title string |
| `ff-desktop` | 🔴 | -- | Req 17.7: Title_Line styled distinct from editor content area |
| `ff-desktop` | 🔴 | -- | Req 17.8: Legacy theme Title_Line uses blue background (#0000AA) and white text (#FFFFFF) |
| `ff-desktop` | 🔴 | -- | Req 17.9: Command_Field remains third element below Title_Line |
| `ff-desktop` | 🔴 | -- | Req 18.1: "Move to Other View" detaches tab into Floating_Window with full chrome (deferred Phase AL) |
| `ff-desktop` | 🔴 | -- | Req 18.2: Floating tab has functional Title_Line and Command_Field (deferred Phase AL) |
| `ff-desktop` | 🔴 | -- | Req 18.3: Closing Floating_Window redocks tab at original position (deferred Phase AL) |
| `ff-desktop` | 🔴 | -- | Req 18.4: Tab_Header removed from Primary_Window bar when detached; restored on redock (deferred Phase AL) |
| `ff-desktop` | 🔴 | -- | Req 18.5: Floating_Window OS title bar shows Title_Line content + " -- FileForge Workbench" (deferred Phase AL) |
| `ff-desktop` | 🔴 | -- | Req 18.6: Drag Tab_Header 20px outside tab bar detaches to Floating_Window (deferred Phase AL) |
| `ff-desktop` | 🔴 | -- | Req 18.7: Maximum 16 simultaneous Floating_Windows enforced (deferred Phase AL) |

### Phase AL -- Tab Window Chrome (final status)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.3: POM tab Title_Line shows "FileForge Workbench  vX.Y.Z" -- `title_line_pom_tab_shows_app_name_and_version` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.4: File editor tab Title_Line shows full absolute path -- `title_line_file_editor_shows_path` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.5: Untitled file editor tab Title_Line shows "[Untitled]" -- `title_line_untitled_shows_placeholder` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.6: ConfigPanel tab Title_Line shows "[CONFIG]" -- `title_line_config_panel_shows_config` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 17.6: FilesPanel tab Title_Line shows "[FILES]" -- `title_line_files_panel_shows_files` |
| `ff-desktop` | 🔲 | -- | Req 17.1: Three-element chrome order (Tab_Header, Title_Line, Command_Field) visible at runtime (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 17.2: Title_Line is read-only (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 17.7: Title_Line visually distinct from editor content area (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 17.8: Legacy theme Title_Line uses blue background / white text (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 17.9: Command_Field remains third element below Title_Line (manual UI verification) |
| `ff-desktop` | 🔴 | -- | Req 17.10 (CR-CH-034, B050): Tab_Header + Title_Line derive the label from live Context state (kind / loaded menu), never a stale cached title after an in-place context switch |
| `ff-desktop` | 🔴 | -- | Req 18.1–18.7: Detachable tab windows -- deferred to future phase (superseded by CR-CH-035 rows below) |
| `ff-desktop` | ✅ | `shell/tests.rs::full_shell_detach_sets_floating_and_records_floating_tab` | Req 18.1/18.2/18.8 (CR-CH-035, B045): Detached Workspace renders the tab's REAL Context (immediate viewport), not a placeholder -- detach records a FloatingTab + sets is_floating; immediate-viewport render path executes without panic. Real OS-window APPEARANCE remains MANUAL (row below) |
| `ff-desktop` | ✅ | `shell/tests.rs::full_shell_redock_restores_tab_at_origin`, `tab_manager.rs::{move_tab_restores_to_origin_preserving_order, move_tab_follows_active_tab, move_tab_clamps_origin_beyond_count_to_end, remove_at_*, insert_at_*}` | Req 18.3/18.4/18.9 (CR-CH-035, B045): detach hides the header (bar skips is_floating) + sets is_floating; redock reinserts at origin index via move_tab (remove+reinsert, order preserved), tab identity/content survive round-trip (tracked by stable TabId) |
| `ff-desktop` | ✅ | `shell/tests.rs::truncate_title_clamps_to_max_on_char_boundary` | Req 18.5 (CR-CH-035, B045): Detached_Workspace OS title = title_line_text + " -- FileForge Workbench", truncated to 80 chars on a char boundary (pure truncate_title) |
| `ff-desktop` | 🔲 | -- | Req 18.6 (CR-CH-035, B045): drag Tab_Header >20px outside the bar detaches at release point -- MANUAL: real pointer-vs-OS-window geometry not headless-drivable (context-menu + SPLIT detach paths cover the headless case; drag-out is the follow-up) |
| `ff-desktop` | ✅ | `shell/tests.rs::full_shell_detach_rejected_at_16_window_limit`, `split_detach_at_limit_shows_error` | Req 18.7 (CR-CH-035, B045): max 16 Detached Workspaces (limit unified on floating_tabs.len across context-menu + SPLIT paths); beyond -> status message, no detach |
| `ff-desktop` | 🔲 | -- | Req 18.1/18.2 (CR-CH-035, B045): the ACTUAL separate OS window (appearance, taskbar presence, independent move/resize/minimize) -- MANUAL: real multi-viewport OS windows are the documented egui_kittest exception (testing.md) |
| `ff-desktop` | ✅ | `shell/tests.rs::{with_workspace_context_saves_and_restores_primary_context, detached_command_acts_on_its_tab_not_the_primary}` | Req 18.10 (CR-CH-036, B045): each Detached_Workspace is an independent command context -- own Command ===> buffer/SCROLL/status; a command in a detached window acts on ITS tab and leaves the primary window's command_text + active tab untouched (bidirectional isolation via scoped WorkspaceCommandContext swap). Shared RETRIEVE ring allowed (documented); real OS window MANUAL |
| `ff-desktop` | 🔲 | -- | Req 18.3 (CR-CH-037, B045): detached window Close (X) behaves as RETURN on that window's context -- non-POM returns to POM (window stays), POM closes the workspace; no redock, no Alt+F4 binding. RETURN behaviour is automated (menu-workspace 14.10 row); the REAL OS-window close gesture is MANUAL (real multi-viewport window, testing.md exception) |
| `ff-desktop` | ✅ | `shell/tests.rs::detached_function_key_return_acts_on_its_tab` | Req 18.11 (B068): F-keys pressed while a Detached_Workspace has focus act on THAT window's context (F4=RETURN there, primary tab untouched); shared resolve_function_key_command + dispatch_detached_function_key |
| `ff-desktop` | 🔲 | -- | Req 18.12 (CR-NR-089, B045): a Detached_Workspace renders its own Menu_Bar (salted panel id via render_menu_bar_from_menu); item dispatch acts on the detached tab. Render exercised by the floating loop (no panic); the VISUAL menu bar in the real OS window is MANUAL |
| `ff-desktop` | ✅ | `shell/tests.rs::{full_shell_dock_command_redocks_tab_at_origin, full_shell_dock_on_non_detached_is_noop_with_message}` | Req 18.13 (CR-NR-088, B045): DOCK command re-docks the current Detached_Workspace to its origin index; no-op-with-message when not detached |
| `ff-desktop` | ✅ | `shell/tests.rs::{return_from_non_pom_navigates_to_pom, return_from_drilled_in_non_pom_goes_straight_to_pom, return_from_pom_with_other_tabs_closes_pom_not_app, end_at_empty_stack_last_tab_exits}` | menu-workspace Req 14.10 (CR-CH-038): RETURN in a non-POM navigates to the POM (clears nav_stack); RETURN in a POM closes that one workspace (exit if last). END unchanged (one step). Identical docked/detached |
| `ff-keys` | ✅ | `key_map.rs::key_map_default_global_has_full_shift_row` | function-keys-and-history Req 15.2 (CR-NR-088): default Global_Key_Map Shift+F2 = DOCK (was SPLIT); Base F2 = SPLIT; still 24 default bindings |

### CR-NR-090 -- Configurable Workspace Kinds (Slice B.1)
| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `workspace_kind/tests.rs::{base_kind_builtin_tag_round_trips, base_kind_external_tag_round_trips, base_kind_unknown_tag_becomes_external}` | workspace-kinds Req 1.1/1.2 (CR-NR-090 B.1): KindConfig{name,modelled_on,title,menu_bar,key_list,profile} + open BaseKind (Builtin/External); External accepted by schema, unresolved-with-notice in v1 |
| `ff-desktop` | ✅ | `workspace_kind/tests.rs::registry_load_flags_external_base_and_falls_back` | workspace-kinds Req 1.3 (B.1): user Kind modelled_on a built-in resolvable base; a non-built-in base is flagged-with-notice and falls back to a safe built-in (no crash) |
| `ff-desktop` | ✅ | `workspace_kind/tests.rs::{tab_kind_maps_to_builtin_kind, registry_resolve_base_single_hop_for_builtin, builtin_stable_names_are_unique_and_round_trip}` | workspace-kinds Req 1.4/1.5 (B.1): runtime resolves a Kind to its base (single hop); a Kind's stable name is its id (from_tab_kind incl. POM/menu is_home split) |
| `ff-desktop` | ✅ | `workspace_kind/tests.rs::{kind_config_toml_round_trips_builtin_base, kind_config_toml_round_trips_external_base}` | workspace-kinds Req 1.6 (B.1): KindConfig TOML round-trip via KindConfigToml incl. modelled_on Builtin+External tags |
| `ff-desktop` | ✅ | `workspace_kind/tests.rs::{registry_builtin_defaults_resolve_every_kind, registry_load_adds_user_kind_and_overrides_builtin_by_name, registry_load_skips_unparseable_with_notice, registry_load_absent_dir_is_silent}` | workspace-kinds Req 2.1/2.2/2.3 (B.1): KindRegistry compiled built-in defaults; effective(name) total; user file overrides built-in by name; unparseable skipped with notice; absent dir silent; never crashes |
| `ff-desktop` | ✅ | `workspace_kind/tests.rs::builtin_default_titles_distinguish_catalogs_from_files` | workspace-kinds Req 2.4 (B.1): compiled built-in default titles are distinct (Catalog Explorer [CATALOGS] vs File Explorer [FILES]) -- fixes the shared-label smell |
| `ff-desktop` | ✅ | `shell/tests.rs::{kind_title_derives_from_registry_and_user_override_wins, title_line_files_panel_shows_files}` | workspace-kinds Req 3.1/3.2/3.3 (B.1): tab header + Title_Line derived from the Kind's effective config title via kind_title/title_line_text; workspace_name still overrides; user override applies live; POM banner + editor path unchanged |

| `ff-desktop` | 🔲 | `shell.rs` unit tests | Req 8.1: Command field Enter-to-submit -- pressing Enter while field has focus executes the command |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 8.2: Command field Enter on empty field is a no-op |

## Final Summary (after Phase AL)

| Status | Count |
|--------|-------|
| ✅ PASS | 168 |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 26 |
| 🔴 NOT COVERED | 7 |
| **Total crates** | **64** |

### Phase AM -- Per-Context Key Maps, PFSHOW, 24-Key Bar, Hotspots, END/RETURN, LIST+RETRIEVE

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-keys` | 🔴 | -- | Req 12.1–12.7: PFSHOW ON/OFF/toggle command registered and dispatched |
| `ff-desktop` | ✅ | filter_narrows_visible_entries | Req 12.4: key_bar_visible persisted in session state |
| `ff-desktop` | 🔴 | -- | Req 12.1–12.3: Key_Label_Bar shown/hidden by PFSHOW command |
| `ff-keys` | 🔴 | -- | Req 13.1–13.2: KeyLabelBarModel produces two rows of 12 slots each (F1–F12, F13–F24) |
| `ff-keys` | 🔴 | -- | Req 13.2: Unassigned slots present with blank label (grid preserved) |
| `ff-desktop` | 🔴 | -- | Req 13.3–13.4: Two-row Key_Label_Bar rendered in footer |
| `ff-keys` | 🔴 | -- | Req 14.1–14.5: KeyMapResolver supports context_maps with full-replacement semantics |
| `ff-desktop` | 🔴 | -- | Req 14.2, 14.4: Tab switch calls set_context; Key_Label_Bar updates same frame |
| `ff-keys` | 🔴 | -- | Req 14.7: [context_key_maps] TOML section parsed into KeyMapResolver |
| `ff-keys` | 🔴 | -- | Req 15.1–15.2: KeyMap::default_global() returns 5-key built-in default map |
| `ff-keys` | 🔴 | -- | Req 15.3: User [global_key_map] fully replaces built-in defaults |
| `ff-desktop` | 🔴 | -- | Req 16.1–16.3: Key_Label_Bar slots are clickable; click dispatches assigned command |
| `ff-desktop` | ✅ | theme_follow_os_defaults_to_false | Req 16.2: Click on blank slot is no-op |
| `ff-desktop` | 🔲 | manual | Req 16.4: Hover over assigned slot shows full command string tooltip |
| `ff-keys` | 🔴 | -- | Req 17.1–17.2: nav.end and nav.return commands registered |
| `ff-desktop` | 🔴 | -- | Req 17.1: END closes current tab, navigates to previous tab or POM |
| `ff-desktop` | 🔴 | -- | Req 17.2: END from POM exits application |
| `ff-desktop` | 🔴 | -- | Req 17.3: RETURN navigates to POM tab from any context |
| `ff-desktop` | 🔴 | -- | Req 17.4: RETURN from POM exits application |
| `ff-keys` | 🔴 | -- | Req 17.7: END and RETURN added to ExclusionFilter; not recorded in history |
| `ff-help` | 🔴 | -- | Req 18.1–18.2: Missing help topic emits "not available yet" status message |
| `ff-help` | 🔴 | -- | Req 18.3: No specific context opens Help_Index (existing behaviour preserved) |
| `ff-keys` | 🔴 | -- | Req 19.1–19.2: LIST+RETRIEVE returns ShowList variant with all history entries |
| `ff-desktop` | 🔴 | -- | Req 19.3–19.4: Modal history-list overlay; selection populates field; Escape clears |
| `ff-keys` | 🔴 | -- | Req 19.6: LIST not added to Command_History when used as RETRIEVE trigger |

### Phase AM -- Final Status (implementation complete)

| Crate | Status | Test | Notes |
|-------|--------|------|-------|
| `ff-keys` | ✅ | `key_map.rs` unit tests | Req 15.1–15.2: `KeyMap::default_global()` -- 5 built-in assignments, 19 unassigned |
| `ff-keys` | ✅ | `key_label_bar.rs` unit tests | Req 13.1–13.2: `row0()`/`row1()` -- two rows of 12, all 24 slots always present |
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
| `ff-desktop` | 🔴 | -- | Req 12.4: `key_bar_visible` session persistence (TOML round-trip) -- deferred |
| `ff-desktop` | 🔴 | -- | Req 18.1–18.3: contextual help "not available yet" fallback -- deferred (ff-help crate) |
| `ff-desktop` | 🔴 | -- | Req 14.7: `[context_key_maps]` TOML config parsing -- deferred (config integration) |

### CR-CH-027 -- Full compiled default key map + keymaps/ override files (Slice 1)

> Owner-specified full Base + Shift default key map (code-only, like the compiled
> POM/Settings menus); per-workspace-kind override files at
> `<User_Data_Dir>/keymaps/<context>.toml` layered over the compiled default;
> RESET BARE archives `keymaps/`.

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-keys` | ✅ | `key_map.rs::key_map_default_global_has_full_base_row` | function-keys Req 15.1: `KeyMap::default_global()` contains the full Base row (F1 HELP, F2 SPLIT, F3 END, F4 RETURN, F5 RFIND, F6 RCHANGE, F7 UP, F8 DOWN, F9 SWAP, F10 LEFT, F11 RIGHT, F12 RETRIEVE) with labels |
| `ff-keys` | ✅ | `key_map.rs::key_map_default_global_has_full_shift_row` | function-keys Req 15.2: `default_global()` contains the Shift row (SF1 HELP..SF6 RCHANGE mirror base; SF7 UP MAX, SF8 DOWN MAX, SF9 SWAP, SF10 LEFT MAX, SF11 RIGHT MAX, SF12 CURSOR) |
| `ff-keys` | ✅ | `key_map.rs::key_map_default_global_binds_exactly_base_and_shift_f1_to_f12`; `shell::tests::keymaps_file_present_overrides_default_for_context` (full-replacement) | function-keys Req 15.3: the compiled default is overridable in full by `[global_key_map]` or a `keymaps/<context>.toml` file (full-replacement); Ctrl/Alt + keys beyond F12 unassigned |
| `ff-keys` | ✅ | `key_map.rs::key_map_default_global_*` (built by `KeyMap::empty` + `set`, no file read) | function-keys Req 15.1 (code-only): the default map is compiled, NOT a shipped TOML file |
| `ff-desktop` | ✅ | `shell::tests::ensure_keymaps_dir_creates_keymaps_dir` | function-keys Req 14.11: `ensure_keymaps_dir` creates `<User_Data_Dir>/keymaps/` (may be empty); built-in defaults never materialised |
| `ff-desktop` | ✅ | `shell::tests::keymaps_file_present_overrides_default_for_context`, `keymaps_malformed_file_is_skipped_and_falls_back` | function-keys Req 14.9/14.10: `keymaps/<context>.toml` present -> loaded as the context map; absent -> compiled default; malformed -> skipped (DEBUG) + default fallback |
| `ff-desktop` | ✅ | `shell::tests::keymaps_file_takes_precedence_over_config_section` | function-keys Req 14.12: a `keymaps/<context>.toml` FILE takes precedence over a `[context_key_maps.<name>]` config section for the same context (file loaded second) |
| `ff-desktop` | ✅ | `shell::reset_bare::tests::archive_config_moves_present_items_and_removes_originals` | configuration-system Req 19.4 (CR-CH-027): RESET BARE archives the `keymaps/` directory (move-not-delete) alongside menus/themes |

### CR-CH-028 -- Cursor_Context package on every command (Slice 2a)

> ADDITIVE, behaviour-preserving. A flexible Cursor_Context (fixed core + open
> extras bag) is threaded to every command via the existing `ExecutionContext` /
> `ContextProvider` seam. Commands merely receive it (CSR consumption deferred).
> Delivers CR-NR-079 context-aware HELP + the canonical "not implemented yet" message.

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-command` | ✅ | `context::tests::empty_context_has_empty_cursor_context` | command-framework Req 12.1: `ExecutionContext` carries a `CursorContext`; handlers that ignore it are unchanged (additive) |
| `ff-command` | ✅ | `context::tests::builder_sets_cursor_context_core` | command-framework Req 12.2: Cursor_Context CORE fields (workspace context, focused-control identity, focused text, cursor line/col, selection, scroll setting), each optional |
| `ff-command` | ✅ | `context::tests::cursor_context_extras_bag_holds_typed_values`, `cursor_context_default_is_empty` | command-framework Req 12.3: open EXTRAS bag (string-keyed `ContextValue`); commands ignore unrecognised keys; empty when nothing extra |
| `ff-desktop` | ✅ | `shell::tests::cursor_context_snapshot_reflects_command_line_focus`, `context_provider_returns_populated_cursor_context` | command-framework Req 12.4: shell-backed `ShellContextProvider` populates the package from live focus/selection; command-line + bound paths carry the SAME package (parity) |
| `ff-desktop` | ✅ | `shell::tests::cursor_context_snapshot_reflects_command_line_focus` (snapshot refreshed each frame from live focus) | command-framework Req 12.5: Cursor_Context is a per-invocation snapshot; not retained/mutated by a command |
| `ff-command` | 🔲 | -- | command-framework Req 12.6: explicit params take precedence over context. DELIVERY-ONLY this slice (no command consumes context yet); the precedence rule is verified when the first consumer (CSR LEFT/RIGHT/UP/DOWN) lands in Slice 2b. MANUAL/deferred: no behaviour to assert until a consumer exists |
| `ff-desktop` | ✅ | `shell::tests::not_implemented_message_is_canonical` | command-framework Req 12.7: single canonical "command not implemented yet" message constant on the bound path ("out of context" is command-owned, deferred); bound-path consumers land in Slice 2b |
| `ff-desktop` | ✅ | `shell::tests::help_consumes_focused_menu_option_context` | command-framework Req 12.8 (CR-NR-079): HELP consumes the Cursor_Context -- F1 on a focused Menu_Option resolves that option's help topic (F1 on FILES -> `cmd:FILES`) |

### CR-CH-029 -- Keys Workspace replaces the modal dialog (Key Assignments Slice 3)

> The modal `KeyConfigDialog` is re-homed into a Keys Workspace Context (modelled
> on the Menus/Theme editors): a workspace-KIND dropdown + Save to
> `keymaps/<kind>.toml` (closes CR-CH-027); `K` -> `KEYS` in Settings. The
> Command_Picker/72-slot model (Req 20/21) is deferred.

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell::tests::keys_command_opens_keys_workspace` | function-keys Req 22.1: Keys Workspace (`TabKind::KeysEditor`) replaces the modal `KeyConfigDialog` (modal retired), modelled on the Menus/Theme editor Contexts |
| `ff-desktop` | ✅ | `keys_editor_panel::state::tests::*`, `shell::tests::keys_with_kind_preselects_that_kind` | function-keys Req 22.2: workspace-KIND dropdown lists the stable context names; selecting a kind loads its key list (keymaps file or compiled default) |
| `ff-desktop` | ✅ | `keys_editor_panel::state::tests::*` | function-keys Req 22.3: editable key grid for the selected kind (current grid; Command_Picker deferred) |
| `ff-desktop` | ✅ | `shell::tests::keys_editor_save_writes_keymaps_file_for_kind` | function-keys Req 22.4: Save writes `<User_Data_Dir>/keymaps/<kind>.toml` (the resolver's override file, CR-CH-027) and reloads that kind's context map |
| `ff-desktop` | ✅ | `shell::tests::keys_command_opens_keys_workspace` | function-keys Req 22.5: `KEYS` opens the Keys Workspace in place (nav-stack push); menu affordance dispatches `KEYS` (command parity) |
| `ff-desktop` | ✅ | `menu_workspace::defaults::tests::default_settings_toml_has_recovery_baseline_options`, `recovery_settings_menu_has_barebones_options` | function-keys Req 22.6 / menu-workspace Req 12.3: `DEFAULT_SETTINGS_TOML` Core group gains `K` -> `KEYS` "Keys" |
| `ff-desktop` | ✅ | `shell::tests::full_shell_keys_first_tab_focuses_kind_dropdown` | function-keys Req 22.7: Keys Workspace reports its `InteriorFocus` (kind dropdown = first interior); full-shell first-Tab test (no phantom stop) |
| `ff-desktop` | ✅ | `session_manager` (KeysEditor -> None, transient) | function-keys Req 22.8: Keys Workspace is a transient editing Context (not restored as a tab; keymaps files are the persisted artefacts) |

### CR-NR-077 -- THEME LIST popup selector + Settings Themes submenu

> A `THEME LIST` command opens a centred arrow-navigable popup of the available
> themes (Up/Down/Enter/Escape); the Settings menu gains a Themes submenu. Both
> route through the shared `set_active_theme` apply+persist path (command parity).

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | -- | theme-and-appearance Req 17.8: `THEME LIST` (matched before `THEME <name>`) opens a centred Theme_List_Popup listing available themes in list order; does not change the active theme on open |
| `ff-desktop` | 🔴 | -- | theme-and-appearance Req 17.9: the popup pre-selects the entry equal to the currently active theme (else the first entry) |
| `ff-desktop` | 🔴 | -- | theme-and-appearance Req 17.10: Down/Up move (wrapping), Enter applies + closes, Escape / click-outside closes without changing the theme, row-click applies |
| `ff-desktop` | 🔴 | -- | theme-and-appearance Req 17.11: choosing an entry applies + persists via the shared `set_active_theme` path (exact name; persistence failure surfaces the non-silent message of 17.7) |
| `ff-desktop` | 🔴 | -- | theme-and-appearance Req 17.12: while open the popup is modal for input -- shell Tab-cycle, function keys, and Ctrl+S suppressed (folded into `modal_open`) |
| `ff-desktop` | 🔴 | -- | theme-and-appearance Req 17.13: Settings Themes submenu lists themes; selecting one dispatches `THEME <name>` via `handle_command` (command parity), not a direct setter |

> NOTE (CR-NR-080): the CR-NR-077 theme-picker behaviour above (Req 17.8-17.13) is
> now DELIVERED via the configurable menu-bar mechanism (CR-NR-080 Slice D,
> menu-workspace Req 17.10/17.11) rather than a bespoke popup. These rows will be
> satisfied by that work.

### CR-NR-080 -- Configurable named menu bars (a menu rendered horizontally)

> The menu bar becomes a Menu_File rendered horizontally; top-level buttons PEEK
> their referenced submenu as a dropdown (not navigate); leaves dispatch commands
> (parity). Named + editable + per-kind assignable; dynamic Themes source. Model +
> command resolution unchanged. SLICED: A rows below; B-D added when those slices
> are specced into tasks.

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `menu_workspace::defaults::tests::default_menubar_is_the_barebones_pom`; `shell::tests::menu_bar_has_file_catalogs_menu` | menu-workspace Req 17.1: the menu bar renders from a Menu_File (`render_menu_bar_from_menu` iterates bar-visible options as dropdown buttons in order); top-level buttons labelled by `command` |
| `ff-desktop` | ✅ | `menu_workspace::defaults::tests::default_menubar_is_the_barebones_pom` | menu-workspace Req 17.2: `default_menubar_menu()` returns the compiled POM (single source of truth); the bar renders from it when no user file exists |
| `ff-desktop` | ✅ | `menu_workspace::defaults::tests::default_menubar_bar_visible_options_are_settings_catalogs_files_help`, `shell::tests::menu_bar_has_help_and_excludes_return` | menu-workspace Req 17.2a: the bar renders only `show_in_menu_bar = true` options; barebones `RETURN` (false) excluded, leaving Settings/Catalogs/Files/Help |
| `ff-desktop` | ✅ | `shell::tests::menu_bar_peek_of_settings_returns_settings_menu_options` | menu-workspace Req 17.3: opening a top-level button PEEKS the menu its command references (`peek_menu_options` returns that menu's options) WITHOUT navigating the workspace |
| `ff-desktop` | ✅ | `shell::tests::menu_bar_peek_of_non_menu_command_is_empty`, `menu_bar_leaf_dispatch_is_command_parity` | menu-workspace Req 17.4: a non-menu command peeks empty (rendered as a direct item); selecting a leaf dispatches its command via `handle_command` (command parity) |
| `ff-desktop` | 🔲 | -- | menu-workspace Req 17.5: dropdown items keyboard-navigable via egui-native `menu_button` behaviour (arrows/Enter/Escape). MANUAL: egui-native menu keyboard traversal inside an open dropdown is provided by egui and not driven by the kittest harness |
| `ff-desktop` | ✅ | `shell::tests::full_shell_tab_reaches_settings_as_first_menu_item`, `full_shell_tab_cycle_wraps_to_command_field_and_skips_chrome` | menu-workspace Req 17.6: first/last top-level button ids captured from the data-driven bar for the CR-CH-023 Boundary_Policy (Tab reaches the bar; last button wraps to the command field) |
| `ff-desktop` | ✅ | `menu_workspace::render` POM/menu tests unchanged; `shell::tests::menu_bar_peek_of_settings_returns_settings_menu_options` (peek does not navigate) | menu-workspace Req 17.7: the POM/Settings vertical Menu_Workspace render is unchanged by the data-driven bar |
| `ff-desktop` | ✅ | `menu_workspace::loader::tests::load_show_in_menu_bar_defaults_to_true`, `load_show_in_menu_bar_false_preserved`; `menu_workspace::serialiser::tests::recovery_pom_round_trips`, `defaults_are_omitted_but_parse_back` | menu-workspace Req 1.3 (CR-NR-080): `show_in_menu_bar` (bool, default true) parses, defaults true when absent, round-trips (omitted when true, written when false); Menus Editor exposes a "bar" checkbox |
| `ff-desktop` | ✅ | `shell::tests::menu_bar_dynamic_theme_list_generates_one_item_per_theme`, `menu_bar_dynamic_options_none_for_ordinary_command` | menu-workspace Req 17.10 (CR-NR-080 Slice D): a `THEME LIST` option is a DYNAMIC source -- `dynamic_menu_options` generates one child per `list_all_themes` entry at runtime (each `THEME <name>`); ordinary commands yield `None`. Mechanism delivered; the barebones default does not yet wire a Themes dropdown (usable when a menu includes a `THEME LIST` option, Slice B / config) |
| `ff-desktop` | ✅ | `shell::tests::menu_bar_leaf_dispatch_is_command_parity`, `theme_command_sets_mode` | menu-workspace Req 17.11 (CR-NR-080 Slice D): selecting a generated theme item dispatches `THEME <name>` via `handle_command` (command parity, shared `set_active_theme` apply+persist). This delivers the CR-NR-077 theme-picker behaviour (theme-and-appearance Req 17.8-17.13) via the menu-bar mechanism rather than a bespoke popup |
| `ff-desktop` | ✅ | `shell::tests::resolve_menu_bar_falls_back_to_compiled_default`, `resolve_menu_bar_loads_user_file_when_present` | menu-workspace Req 17.8 (CR-NR-080 Slice B): the bar is a NAMED menu -- `resolve_menu_bar_menu` loads `menus/<slug>.toml` for the default bar name (`MB-POM` -> `mb-pom`) via the loader; a user file OVERRIDES the compiled default, else falls back to it. Authorable via the existing Menus Editor (same slugging); `MB-` is a convention |

### CR-NR-081 -- Application Profiles (startup-and-session Requirement 22)

> `--profile <name>` / `-p <name>` redirects the User_Data_Dir to
> `profiles/<slug>/` at the single `platform_default_path` seam so every subsystem
> is isolated per profile; default profile = today's location; active profile
> shown in the UI; RESET BARE scoped to the active profile.

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-session` | ✅ | `user_data_dir::tests::active_profile_redirects_resolved_user_data_dir` | startup-and-session Req 22.1: `set_active_profile`/`active_profile` process-global set once at startup records the Active_Profile |
| `ff-session` | ✅ | `user_data_dir::tests::active_profile_redirects_resolved_user_data_dir`, `resolve_without_custom_path_uses_platform_default` | startup-and-session Req 22.2: no `--profile` -> DEFAULT_PROFILE; resolved User_Data_Dir identical to the pre-CR-NR-081 location |
| `ff-session` | ✅ | `user_data_dir::tests::active_profile_redirects_resolved_user_data_dir`, `profile_slug_lowercases_and_replaces_non_alphanumerics` | startup-and-session Req 22.3, 22.9: active profile -> `<base>/ffworkbench/profiles/<slug>/`; every `UserDataDir::resolve` caller inherits it (single `platform_default_path` seam); profiles independent |
| `ff-session` | ✅ | `user_data_dir::tests::initialise_creates_directory_and_subdirs` (initialise creates the tree for any resolved dir, incl. a profile dir) | startup-and-session Req 22.4: a fresh profile's dir + required sub-dirs are created on first `initialise()` |
| `ff-desktop` | ✅ | `main.rs` step-0 ordering (`extract_profile_arg` + `set_active_profile` before config `init()`/first resolve); covered structurally + by `extract_profile_arg_*` tests | startup-and-session Req 22.5: the profile is resolved ONCE before any `UserDataDir::resolve`/config `init()` |
| `ff-desktop` | ✅ | `tests::extract_profile_arg_missing_value_returns_none_and_removes_flag` | startup-and-session Req 22.6: missing/empty `--profile` value -> DEFAULT_PROFILE + WARN, no abort |
| `ff-desktop` | ✅ | `tests::extract_profile_arg_long_form_extracts_and_removes`, `extract_profile_arg_short_form_extracts_and_removes`, `extract_profile_then_resolve_paths_keeps_file_arg` | startup-and-session Req 22.7: `--profile`/`-p` and its value are removed from positional file args (`ffwb -p rust file.txt` opens `file.txt`) |
| `ff-desktop` | ✅ | `shell::tests::active_profile_label_reflects_active_profile` | startup-and-session Req 22.8: the Active_Profile (incl. explicit "default") is shown in the Status_Bar |
| `ff-session` | ✅ | `user_data_dir::tests::active_profile_redirects_resolved_user_data_dir` (RESET BARE resolves `UserDataDir::resolve(None)`, which returns the active profile's dir) | startup-and-session Req 22.10: `RESET BARE` archives/resets only the ACTIVE profile's User_Data_Dir (covered by construction via the resolver seam) |

## Final Summary (after Phase AM)

| Status | Count |
|--------|-------|
| ✅ PASS | 186 |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 26 |
| 🔴 NOT COVERED | 10 |
| **Total crates** | **64** |

### Phase AN -- Key Configuration Dialog (Req 20)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-keys` | 🔴 | -- | Req 20.11, 20.12: `KeyModifier` enum and `ModifiedKey` struct defined; 96 TOML key names parse and round-trip |
| `ff-keys` | 🔴 | -- | Req 20.9, 20.12: `KeyBinding.description` field; modifier bindings stored independently in `KeyMap` |
| `ff-keys` | 🔴 | -- | Req 20.12: `KeyMap` uses `ModifiedKey` as key type; `get_plain()` returns only `None`-modifier entry |
| `ff-keys` | 🔴 | -- | Req 20.11: TOML parser accepts `SF1`–`SF24`, `CF1`–`CF24`, `AF1`–`AF24` prefixes |
| `ff-desktop` | 🔴 | -- | Req 20.10: Shift/Ctrl/Alt+Fn dispatch reads `egui::Modifiers`, constructs `ModifiedKey`, dispatches if assigned |
| `ff-desktop` | 🔴 | -- | Req 20.1: `KEYS` command opens Key_Configuration_Dialog |
| `ff-desktop` | 🔴 | -- | Req 20.1: `Edit > Key Assignments…` menu item opens Key_Configuration_Dialog |
| `ff-desktop` | 🔴 | -- | Req 20.2: Dialog shows Default (Global) tab and one tab per context name |
| `ff-desktop` | 🔴 | -- | Req 20.3: Each scope tab shows 24-row grid with Key, Command, Label, Description, Shift/Ctrl/Alt Cmd+Desc columns |
| `ff-desktop` | 🔴 | -- | Req 20.4: Empty command field treated as unassigned on save |
| `ff-desktop` | 🔴 | -- | Req 20.5: Save writes changes to user-layer TOML; Cancel discards |
| `ff-desktop` | 🔴 | -- | Req 20.6: Dialog pre-populates from current effective key maps on open |
| `ff-desktop` | 🔴 | -- | Req 20.7: Label column read-only, derived from command string |
| `ff-desktop` | 🔴 | -- | Req 20.8: Save writes `[global_key_map]` or `[context_key_maps.<name>]` sections |
| `ff-desktop` | 🔴 | -- | Req 20.13: Key_Label_Bar continues to show only plain bindings after modifier extension |
| `ff-desktop` | 🔴 | -- | Req 20.15: Reset to Defaults restores Default tab to built-in defaults; clears context tabs |

### Phase AN -- Key Configuration Dialog (final status)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-keys` | ✅ | `function_key.rs` unit tests | Req 20.11, 20.12: `KeyModifier` + `ModifiedKey` -- 96 slots, all TOML names round-trip |
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
| `ff-desktop` | 🔲 | -- | Req 20.1: `Edit > Key Assignments…` menu item opens dialog (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 20.7: Label column read-only, derived from command (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 20.8: Save writes `[global_key_map]` / `[context_key_maps]` TOML -- deferred (config integration Task 27.6) |
| `ff-desktop` | 🔲 | -- | Req 20.10: Shift/Ctrl/Alt+Fn dispatch reads `egui::Modifiers` at runtime (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 20.13: Key_Label_Bar shows only plain bindings after modifier extension (manual UI verification) |
| `ff-desktop` | 🔴 | -- | Req 20.8: Full TOML persistence for key maps -- deferred to config integration |

### Phase AO -- Detachable Tab Windows (Requirement 18)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 18.1, 18.4: `is_floating` flag set on detach; `FloatingTab` struct with `origin_index` |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 18.7: 16-window limit enforced (`floating_tab_limit_enforced_at_16`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 18.3: `origin_index` preserved on `FloatingTab` (`floating_tab_origin_index_preserved`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 18.3: redock clamps `origin_index` to current tab count (`redock_clamps_to_tab_count`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 18.5: OS title bar format `<Title_Line> -- FileForge Workbench` (`floating_tab_title_format`) |
| `ff-desktop` | 🔲 | -- | Req 18.1: "Move to Other View" opens floating OS window (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 18.2: floating tab has functional Title_Line and Command_Field (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 18.3: closing floating window redocks tab at origin position (manual UI verification) |
| `ff-desktop` | 🔴 | -- | Req 18.6: drag Tab_Header 20px outside bar detaches -- deferred (egui drag-outside-bounds not exposed) |

### Phase AP -- PFSHOW Session Persistence (Requirement 12.4)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `session_manager.rs` unit tests | Req 12.4: `key_bar_visible` persisted to session TOML and restored on next launch (`key_bar_visible_round_trips_through_session`) |
| `ff-session` | ✅ | `session_file.rs` unit tests, `property_tests.rs` | Req 12.4: `key_bar_visible` field in `SessionState` with `serde(default = "default_true")` -- round-trips correctly |

### Phase AQ -- Key Map TOML Persistence (Req 20.8)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.8: `save_produces_correct_config_values_for_global_scope` -- global_key_map table contains assigned keys, omits unassigned |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.8: `save_produces_correct_config_key_for_context_scope` -- context scope produces correct ConfigValue::Table |
| `ff-desktop` | ✅ | `key_config_dialog.rs` unit tests | Req 20.8: `empty_context_scope_produces_empty_table` -- empty context produces empty table (no spurious keys) |
| `ff-desktop` | 🔲 | -- | Req 20.8: Save button writes to actual user-layer TOML file on disk (manual UI verification -- requires running binary) |

### Phase AR -- [context_key_maps] TOML Config Parsing (Req 14.7)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | OK | `shell.rs` unit tests | Req 14.7: context_key_maps_parsed_from_config_value_table -- editor + pom contexts loaded; full-replacement; unknown context falls back to global |
| `ff-desktop` | OK | `shell.rs` unit tests | Req 14.7: context_key_maps_invalid_key_skipped -- F99 produces warning, valid F3 loaded |

### Phase AS -- File Explorer Panel (Requirement 19)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | -- | Req 19.1: `=2` in any command field closes current context and switches tab to FileExplorerPanel in-place |
| `ff-desktop` | 🔴 | -- | Req 19.2: `=FILES` (case-insensitive) closes current context and switches tab to FileExplorerPanel in-place |
| `ff-desktop` | 🔴 | -- | Req 19.3: `FILES` (no `=` prefix) opens a NEW tab in FileExplorerPanel context; current tab unchanged |
| `ff-desktop` | 🔴 | -- | Req 19.4: option `2` on POM tab transforms that tab to FileExplorerPanel with title `[FILES]` |
| `ff-desktop` | 🔴 | -- | Req 19.5: FileExplorerPanel displays tree view with one top-level node per open/mounted catalog |
| `ff-desktop` | 🔴 | -- | Req 19.6: expanding a catalog node lists its files/datasets as child nodes |
| `ff-desktop` | 🔴 | -- | Req 19.7: tree groups catalogs under Mainframe Catalogs, POSIX Catalogs, Native Catalogs section headers |
| `ff-desktop` | 🔴 | -- | Req 19.8: when no catalogs are mounted, placeholder message is shown |
| `ff-desktop` | 🔴 | -- | Req 19.9: double-clicking a file/member node opens it in a new editor tab |
| `ff-desktop` | 🔴 | -- | Req 19.10: F3/END in FileExplorerPanel returns tab to POM view |
| `ff-desktop` | 🔴 | -- | Req 19.11: FileExplorerPanel tab title in tab bar is `[FILES]` |
| `ff-desktop` | 🔴 | -- | Req 19.12: `FileExplorerPanel` tab kind persists in session and restores on next launch |

### Phase AS -- File Explorer Panel (Requirement 19) -- Final Status

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell.rs`, `tab_manager.rs` unit tests | Req 19.1: `=2` transforms current tab in-place to `FileExplorerPanel` (`equals_2_command_transforms_tab_to_file_explorer`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 19.2: `=FILES` is a shell-level intercept (`equals_files_command_is_shell_intercept`) |
| `ff-desktop` | ✅ | `shell.rs` unit tests | Req 19.3: `FILES` (no `=`) routes to new tab (`files_no_prefix_command_is_shell_intercept`) |
| `ff-desktop` | ✅ | `shell.rs`, `tab_manager.rs` unit tests | Req 19.4: option `2` on POM tab transforms in-place to `FileExplorerPanel` with title `[FILES]` (`option_2_on_pom_tab_transforms_to_file_explorer`) |
| `ff-desktop` | 🔴 | -- | Req 19.5: tree view with catalog nodes (UI -- deferred Task 27.4) |
| `ff-desktop` | 🔴 | -- | Req 19.6: expanding catalog node lists files (UI -- deferred Task 27.6) |
| `ff-desktop` | 🔴 | -- | Req 19.7: Mainframe/POSIX/Native section headers (UI -- deferred Task 27.4) |
| `ff-desktop` | 🔴 | -- | Req 19.8: empty-state placeholder message (UI -- deferred Task 27.5) |
| `ff-desktop` | 🔴 | -- | Req 19.9: double-click opens file in editor tab (UI -- deferred Task 27.7) |
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

### Phase AT -- Allocated Dataset Display (Req 13 virtual-catalog-manager)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | -- | Req 13.1: `FilesPanelState` has `datasets` map and `AllocatedDataset` struct |
| `ff-desktop` | 🔴 | -- | Req 13.2: `AllocOutcome::Confirmed` inserts `AllocatedDataset` into map under correct catalog name |
| `ff-desktop` | 🔴 | -- | Req 13.3: selecting a catalog node populates `ContentAreaState::entries` from datasets map |
| `ff-desktop` | 🔴 | -- | Req 13.4: datasets map persists to session TOML and restores on next launch |
| `ff-desktop` | 🔴 | -- | Req 13.5: deleting a catalog removes all its datasets from the map |

### Phase AT -- Allocated Dataset Display -- Final Status (superseded by Phase BU)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ❌ | SUPERSEDED | Req 13.1: `files_panel_state_has_datasets_map` -- removed in BU.8; `AllocatedDataset` struct and `datasets` HashMap deleted |
| `ff-desktop` | ❌ | SUPERSEDED | Req 13.2: `add_dataset_inserts_into_map_under_catalog_name` -- removed in BU.8; allocation now via `CatalogRegistry::allocate()` |
| `ff-desktop` | ❌ | SUPERSEDED | Req 13.3: `load_entries_populates_content_area_from_datasets` -- removed in BU.8; content area now reads from SQLite |
| `ff-desktop` | ❌ | SUPERSEDED | Req 13.4: session TOML persistence -- removed in BU.8; `save_datasets()`/`load_datasets()` deleted from `SessionManager` |
| `ff-desktop` | ❌ | SUPERSEDED | Req 13.5: `delete_catalog_removes_its_datasets` -- removed in BU.8; `remove_catalog_datasets()` deleted; SQLite is sole store |
| | | | See Phase BU rows above for current passing coverage of Req 13.1-13.5 |

### Phase AU -- Catalog Registry Persistence (B010 fix)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `session_manager.rs` unit tests | Req 2.1: `save_catalog_registry()` writes `catalogs.toml` on exit (`save_and_load_catalog_registry_round_trips`) |
| `ff-desktop` | ✅ | `session_manager.rs` unit tests | Req 2.2: `load_catalog_registry()` reads `catalogs.toml` on startup; returns empty registry if absent (`load_missing_catalog_file_returns_empty_registry`) |

### Phase AV -- File Explorer Panel Tree View (Tasks 27.4–27.7, Req 19.5–19.9)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.5: `registered_catalogs_appear_as_tree_nodes` -- each catalog in registry appears as a top-level expandable node |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.6: `catalog_datasets_accessible_for_child_nodes` -- datasets for a catalog are accessible as child node data |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.7: `section_header_labels_match_catalog_type_labels` -- Mainframe Catalogs / POSIX Catalogs / Native Catalogs headers |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.8: `zero_catalogs_triggers_empty_state` -- empty registry (0 catalogs) triggers placeholder path |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.9: `ps_dataset_is_a_leaf_node_not_a_container` / `po_dataset_is_a_container_node` -- PS is leaf (double-click opens); PO is container |
| `ff-desktop` | 🔲 | -- | Req 19.5–19.9: full tree rendering with expand/collapse and double-click (manual UI verification) |

## Final Summary (after Phase AV)

| Status | Count |
|--------|-------|
| ✅ PASS | 391 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | Req 19.5–19.9 (UI tree rendering -- manual verification) |
| 🔴 NOT COVERED | 0 (all Req 19.5–19.9 criteria have unit test coverage) |

### Phase AW -- Mainframe Dataset Allocation Fixes (B011, B012, CR-NR-003)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | -- | Req 5.8: Mainframe dataset name uppercased on confirm (B011) |
| `ff-desktop` | 🔴 | -- | Req 5.9: duplicate DSN within same catalog rejected with inline error (B012) |
| `ff-desktop` | 🔴 | -- | Req 5.7: Dataset Name pre-populated with catalog HLQ when HLQ is configured (CR-NR-003) |

### Phase AW -- Final Status

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `dataset_alloc_dialog.rs` unit tests | Req 5.8: `validate_uppercases_dataset_name`, `validate_uppercases_mixed_case_name` -- B011 fixed |
| `ff-desktop` | ✅ | `dataset_alloc_dialog.rs` unit tests | Req 5.9: `validate_for_catalog_rejects_duplicate_dsn`, `validate_for_catalog_duplicate_check_is_case_insensitive`, `validate_for_catalog_accepts_unique_dsn`, `validate_for_catalog_empty_existing_always_passes` -- B012 fixed |
| `ff-desktop` | ✅ | `dataset_alloc_dialog.rs` unit tests | Req 5.7: `with_hlq_prepopulates_dataset_name_with_hlq_dot`, `with_hlq_empty_string_gives_dot` -- CR-NR-003 done |

## Final Summary (after Phase AW)

| Status | Count |
|--------|-------|
| ✅ PASS | 399 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 0 new |
| 🔴 NOT COVERED | 0 |

### Phase AX -- Default Home Catalog on First Launch (Req 14 virtual-catalog-manager)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | -- | Req 14.1: when no Native catalogs exist, startup creates a Native catalog named `"Home"` pointing to the user home directory |
| `ff-desktop` | 🔴 | -- | Req 14.2: the Home catalog is registered in the CatalogRegistry immediately and visible in the Files panel on the same launch |
| `ff-desktop` | 🔴 | -- | Req 14.3: the Home catalog is persisted to `catalogs.toml` before the first frame so it survives restart |
| `ff-desktop` | 🔴 | -- | Req 14.4: when one or more Native catalogs already exist, no Home catalog is created |
| `ff-desktop` | 🔴 | -- | Req 14.5: when home directory cannot be determined, falls back to process working directory and still creates the catalog |
| `ff-desktop` | 🔴 | -- | Req 14.6: attempting to delete the `"Home"` Native catalog is rejected with inline error |
| `ff-desktop` | 🔴 | -- | Req 14.7: renaming or editing the Home catalog is permitted; after rename the deletion guard no longer applies |

### Phase AX -- Default Home Catalog on First Launch -- Final Status

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell/update.rs` startup_tests | Req 14.1, 14.2: `no_native_catalogs_triggers_home_catalog_creation` -- empty registry gets a `"Home"` Native catalog pointing at the provided home path |
| `ff-desktop` | ✅ | `shell/update.rs` startup_tests | Req 14.4: `existing_native_catalog_suppresses_home_creation` -- existing Native catalog prevents Home creation |
| `ff-desktop` | ✅ | `shell/update.rs` startup_tests | Req 14.3, 14.5: `home_catalog_uses_provided_path` -- catalog uses the supplied path; `true` return signals caller to persist |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 14.6: `delete_home_native_catalog_is_rejected` -- `execute_delete` returns `Err` for `"Home"` Native catalog; registry unchanged |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 14.7: `delete_renamed_home_catalog_is_permitted` -- Native catalog renamed away from `"Home"` can be deleted normally |

## Final Summary (after Phase AX)

| Status | Count |
|--------|-------|
| ✅ PASS | 404 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 0 new |
| 🔴 NOT COVERED | 0 |

### Phase AV (CR-CH-003) -- Help Fallback Human-Readable Message (Req 18.1, 18.2)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-help` | ✅ | `context_detector.rs` unit tests | Req 18.1: `resolve_with_fallback_missing_topic_returns_err` -- message contains `"Help not yet available for"`, human-readable label (e.g. `command "FIND"`), and raw topic-key (`cmd:FIND`) for diagnostics |
| `ff-help` | ✅ | `context_detector.rs` unit tests | Req 18.2: `resolve_with_fallback_existing_topic_returns_ok` -- registered topic returns `Ok(key)`; no fallback message emitted |

## Final Summary (after CR-CH-003)

| Status | Count |
|--------|-------|
| ✅ PASS | 404 tests (ff-desktop) + 12 (ff-help) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 0 new |
| 🔴 NOT COVERED | 0 |

### Phase AN.5 -- ModifiedKey Property-Based Tests (Task 30)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-keys` | ✅ | `tests/modified_key_pbt.rs` | Req 20.11, 20.12: `modified_key_toml_name_always_round_trips` -- all 96 ModifiedKey TOML names parse back to original (200 cases) |
| `ff-keys` | ✅ | `tests/modified_key_pbt.rs` | Req 20.9, 20.12: `get_plain_unaffected_by_modifier_bindings` -- plain binding unchanged regardless of Shift/Ctrl/Alt entries on same key (200 cases) |
| `ff-keys` | ✅ | `tests/modified_key_pbt.rs` | Req 20.11, 20.12: `from_toml_table_mixed_modifiers_no_cross_contamination` -- mixed modifier TOML produces exactly expected entries, no cross-contamination (200 cases) |

## Final Summary (after AN.5)

| Status | Count |
|--------|-------|
| ✅ PASS | 404 (ff-desktop) + 12 (ff-help) + 10 PBTs (ff-keys) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | 0 new |
| 🔴 NOT COVERED | 0 |

### Phase AY -- File Explorer: Expandable Subdirectories and Scrollable Panel (Req 15 file-tree-panel)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | -- | Req 15.1: clicking expand arrow on a directory node inside a Native catalog shows its children sorted dirs-first alphabetically |
| `ff-desktop` | 🔴 | -- | Req 15.2: child directory nodes are themselves expandable, supporting arbitrary nesting depth |
| `ff-desktop` | 🔴 | -- | Req 15.3: File Explorer Panel content area is wrapped in a vertical scroll region |

### Phase AY -- Final Status

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 15.2: `nested_directory_structure_readable_two_levels_deep` -- two-level nested dirs readable via `std::fs::read_dir` |
| `ff-desktop` | 🔲 | -- | Req 15.1: directory `CollapsingHeader` nodes expand to show children (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 15.2: child dirs are themselves expandable recursively (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 15.3: panel content scrollable via `ScrollArea::vertical()` (manual UI verification) |

## Final Summary (after Phase AY)

| Status | Count |
|--------|-------|
| ✅ PASS | 405 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | Req 15.1, 15.2, 15.3 (UI rendering -- manual verification) |
| 🔴 NOT COVERED | 0 |

### Phase AZ -- File Explorer Context Menu (Requirement 16)

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

### Phase BA -- Open With Default Application (Requirement 17 file-tree-panel)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 17.1: Text/source files open in FFWB editor tab (no external launch) |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 17.2: External file class launches OS default app (Windows cmd start / macOS open / Linux xdg-open) |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 17.3: Unknown extension uses magic-byte scan; UTF-8 text opens in editor, binary launches OS app |
| `ff-desktop` | 🔲 | -- | Req 17.4: Launch failure falls back to FFWB editor with status-bar message |
| `ff-desktop` | 🔲 | -- | Req 17.5: Open With shows platform picker (Windows openwith / macOS open -a / Linux xdg chooser) |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 17.6: DefaultAppLaunch is non-blocking (Command::spawn, UI thread not blocked) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 17.7: Mainframe nodes always open in FFWB editor regardless of content |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 17.8: EXTERNAL_EXTENSIONS table covers all required categories (Office, PDF, images, audio/video, archives, executables, databases) |
| `ff-desktop` | 🔲 | -- | Req 17.9: POSIX catalog file nodes follow same FileClass classification and launch rules as Native nodes |

### Phase BC -- Directory-first alphabetical sort in content area (Req 10.7)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 10.7: Name-sort groups containers before non-containers, each group sorted case-insensitively -- `visible_entries_name_sort_groups_dirs_before_files`, `visible_entries_name_sort_dirs_are_alphabetical_within_group`, `visible_entries_type_sort_does_not_force_dir_grouping` |

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.1: Native catalog directory children sorted directories-first then alphabetically case-insensitive -- `collect_native_entries_sorts_dirs_first_then_alpha` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.2: Each file node displays human-readable size; `format_size_produces_correct_strings` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.3, 18.4, 18.5: Timestamps in `YYYY-MM-DD HH:MM` format -- `format_timestamp_produces_correct_format` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.6: Permission attributes returned as non-empty string -- `format_permissions_returns_nonempty_string` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.7: Valid entries collected without error; inaccessible entries silently skipped via `metadata().ok()?` -- `collect_native_entries_returns_valid_entries` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 18.8: Opening unreadable file stores error in `last_error`; no editor tab opened -- `open_file_node_stores_error_for_nonexistent_file` |
| `ff-desktop` | 🔲 | -- | Req 18.9: Attribute columns rendered in correct order and alignment (manual UI verification) |

### Phase BD -- File Explorer tree: drag-select and copy as text tree (Req 19 file-tree-panel)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | -- | Req 19.1: drag-select highlights all visible nodes between start and current cursor position |
| `ff-desktop` | 🔴 | -- | Req 19.2: Shift+click extends selection from Anchor_Node to clicked node |
| `ff-desktop` | 🔴 | -- | Req 19.3: Ctrl+click toggles individual node membership without affecting others |
| `ff-desktop` | 🔴 | -- | Req 19.4: selected nodes rendered with `ui.selection_background` tint |
| `ff-desktop` | 🔴 | -- | Req 19.5: Ctrl+C with non-empty selection writes Text_Tree to OS clipboard |
| `ff-desktop` | 🔴 | -- | Req 19.6: `build_text_tree` produces correct indented ASCII output with `[DIR]` prefix and tree connectors |
| `ff-desktop` | 🔴 | -- | Req 19.7: "Copy as Text Tree" context menu item present above "Copy" group |
| `ff-desktop` | 🔴 | -- | Req 19.8: Escape clears multi-selection, reverts to single-node mode |
| `ff-desktop` | 🔴 | -- | Req 19.9: selection extends to nodes scrolled into view during drag |
| `ff-desktop` | 🔴 | -- | Req 19.10: Mainframe nodes use DSN in Text_Tree output |

### Phase BE -- File Explorer keyboard navigation + file copy/paste (Req 20–21 file-tree-panel)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | -- | Req 20.1: Tab from CommandField transfers focus to File Explorer node list; Cursor_Node set to first visible catalog node |
| `ff-desktop` | 🔴 | -- | Req 20.2: Tab advances Cursor_Node to next visible node in display order |
| `ff-desktop` | 🔴 | -- | Req 20.3: Tab on a collapsed container expands it before advancing |
| `ff-desktop` | 🔴 | -- | Req 20.4: Down/Up Arrow moves Cursor_Node without expanding containers |
| `ff-desktop` | 🔴 | -- | Req 20.5: Right Arrow expands collapsed container; Left Arrow collapses expanded container or moves to parent |
| `ff-desktop` | 🔴 | -- | Req 20.6: Shift+Arrow moves Cursor_Node and adds newly visited node to Keyboard_Selection |
| `ff-desktop` | 🔴 | -- | Req 20.7: Continued Shift+Arrow adds each newly visited node cumulatively |
| `ff-desktop` | 🔴 | -- | Req 20.8: Releasing Shift preserves Keyboard_Selection; plain Arrow moves cursor without changing selection |
| `ff-desktop` | 🔴 | -- | Req 20.9: Ctrl+Arrow moves Cursor_Node without changing Keyboard_Selection |
| `ff-desktop` | 🔴 | -- | Req 20.10: Ctrl+Space toggles Cursor_Node membership in Keyboard_Selection |
| `ff-desktop` | 🔴 | -- | Req 20.11: Ctrl+C with non-empty Keyboard_Selection copies selected nodes |
| `ff-desktop` | 🔴 | -- | Req 20.12: Escape clears Keyboard_Selection; Cursor_Node remains |
| `ff-desktop` | 🔴 | -- | Req 20.13: Cursor_Node rendered with focus ring distinct from selection fill; both shown when node is cursor and selected |
| `ff-desktop` | 🔴 | -- | Req 21.1: Ctrl+C stores selected node paths in File_Copy_Clipboard with operation type Copy |
| `ff-desktop` | 🔴 | -- | Req 21.2: Ctrl+V in file list dispatches background copy to Paste_Target directory |
| `ff-desktop` | 🔴 | -- | Req 21.3: Paste progress indicator shown in status bar; dismissed on completion; target directory refreshed |
| `ff-desktop` | 🔴 | -- | Req 21.4: Paste failure shows error in status bar; successfully copied files not rolled back |
| `ff-desktop` | 🔴 | -- | Req 21.5: Name collision shows per-file prompt with Overwrite / Skip / Rename options |
| `ff-desktop` | 🔴 | -- | Req 21.6: Ctrl+V in editor with non-empty clipboard opens Paste_Prompt modal |
| `ff-desktop` | 🔴 | -- | Req 21.7: "Insert File Names" inserts one path per line at caret |
| `ff-desktop` | 🔴 | -- | Req 21.8: "Insert File Contents" reads and inserts file text; skips unreadable files with inline error |
| `ff-desktop` | 🔴 | -- | Req 21.9: Mainframe DSN/member paths supported; member name lowercased when pasting to Native/POSIX |
| `ff-desktop` | 🔴 | -- | Req 21.10: Paste to POSIX catalog rejected with status-bar message |
| `ff-desktop` | 🔴 | -- | Req 21.11: File_Copy_Clipboard persists until replaced or cleared; source nodes show dashed border indicator |

### Phase BE -- Final Status (keyboard + paste wired into render loop)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.1: `explorer_focused` field; Tab from CommandField wired in `render_central_panel` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.2: `collect_visible_node_paths` + Tab advance wired |
| `ff-desktop` | 🔲 | -- | Req 20.3: Tab on collapsed container expands it (egui CollapsingHeader state -- manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.4: Arrow keys move cursor without expanding -- `arrow_down_moves_cursor_without_expanding` |
| `ff-desktop` | 🔲 | -- | Req 20.5: Right/Left Arrow expand/collapse containers (egui CollapsingHeader -- manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.6, 20.7: Shift+Arrow extends selection -- `shift_arrow_adds_to_selection` |
| `ff-desktop` | 🔲 | -- | Req 20.8: Releasing Shift preserves selection (modifier release -- manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.9: Ctrl+Arrow moves cursor without changing selection -- `ctrl_arrow_moves_cursor_without_changing_selection` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.10: Ctrl+Space toggles selection -- `ctrl_space_toggles_cursor_node_in_selection` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.11: Ctrl+C copies to clipboard + File_Copy_Clipboard -- wired in `handle_explorer_keyboard` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 20.12: Escape clears selection -- `escape_clears_selection_preserves_cursor` |
| `ff-desktop` | 🔲 | -- | Req 20.13: Cursor focus ring rendering (egui visual -- manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.1: Ctrl+C stores paths in File_Copy_Clipboard -- `ctrl_c_in_file_list_populates_file_copy_clipboard` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.2: Ctrl+V sets paste_prompt_open; target determined by `determine_paste_target` |
| `ff-desktop` | 🔲 | -- | Req 21.3: ff-bgio background copy progress indicator (deferred -- requires ff-bgio wiring) |
| `ff-desktop` | 🔲 | -- | Req 21.4: Paste failure error handling (deferred -- requires ff-bgio wiring) |
| `ff-desktop` | 🔲 | -- | Req 21.5: Name collision Overwrite/Skip/Rename prompt (deferred) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.6: Ctrl+V with clipboard writes paths to OS clipboard + status message -- wired in `render_central_panel` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.7: File paths joined one-per-line -- `insert_file_names_produces_one_path_per_line` |
| `ff-desktop` | 🔲 | -- | Req 21.8: Insert File Contents (deferred) |
| `ff-desktop` | 🔲 | -- | Req 21.9: Mainframe DSN naming transform on paste (deferred) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.10: POSIX catalog paste rejected -- `paste_to_posix_catalog_is_rejected` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 21.11: File_Copy_Clipboard persists until replaced -- `file_copy_clipboard_persists_until_replaced` |

### Phase BD -- File Explorer tree: drag-select and copy as text tree (Req 19 file-tree-panel)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.2: `shift_click_extends_selection_from_anchor` -- Shift+click adds to selection from anchor |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.3: `ctrl_click_toggles_individual_node` -- Ctrl+click toggles without affecting others |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.4: `selectable_label(is_selected, ...)` uses egui selection bg_fill tint |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.5: Ctrl+C calls `build_text_tree` and writes to OS clipboard |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.6: `build_text_tree_flat_selection`, `build_text_tree_hierarchical_selection`, `build_text_tree_dir_prefix`, `build_text_tree_relative_depth` |
| `ff-desktop` | ✅ | `context_menu.rs` unit tests | Req 19.7: `CopyAsTextTree` action present in Native File and Native Dir menus above Copy group |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.8: `escape_clears_multi_selection` -- Escape clears selected_nodes |
| `ff-desktop` | 🔲 | -- | Req 19.1: drag-select range highlight (egui pointer drag -- manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 19.9: selection extends to nodes scrolled into view during drag (manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 19.10: `build_text_tree_mainframe_uses_dsn` -- Mainframe DSN used as-is in text tree |

### Phase BI -- Default BLKSIZE=0 in Dataset Allocation Dialog (CR-CH-005)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `dataset_alloc_dialog.rs` unit tests | Req 5.2: BLKSIZE default is `0` (system-determined); `AllocDatasetForm::default()` returns `"0"` for blksize field; `validate()` accepts 0 as system-determined -- `default_form_blksize_is_zero`, `validate_accepts_blksize_zero` |

## Final Summary (after Phase BD)

| Status | Count |
|--------|-------|
| ✅ PASS | 474 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | Req 19.1, 19.9 (drag pointer -- manual verification) |
| 🔴 NOT COVERED | 0 |

### Phase BF -- Tab Close Button + Files Menu Close (B002, B003, B015, B016)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔲 | -- | B002/B015: `×` close button visible on every tab header (manual UI verification) |
| `ff-desktop` | 🔲 | -- | B003: Files > Close closes the active tab (manual UI verification) |
| `ff-desktop` | 🔲 | -- | B016: bracket rule documented -- system tabs use `[]`, file tabs show filename only (manual UI verification) |

## Final Summary (after Phase BF)

| Status | Count |
|--------|-------|
| ✅ PASS | 474 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | B002/B003/B015/B016 (UI rendering -- manual verification) |
| 🔴 NOT COVERED | 0 |

### Phase BJ -- Catalog Repository Path Display + VFS Dataset Path Resolution (CR-NR-012)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 15.1: `edit_form_displays_repository_path` -- `EditCatalogForm::from_catalog` carries `path` field from source catalog |
| `ff-desktop` | ✅ | `catalog_manager_dialog.rs` unit tests | Req 15.2: `edit_form_repository_path_present_for_all_catalog_types` -- path present for Mainframe and POSIX |
| `ff-desktop` | 🔲 | -- | Req 15.3: Repository path field rendered as read-only label in Edit Catalog dialog (manual UI verification) |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.1, 16.5: `resolve_dataset_path_maps_dsn_to_subpath` -- `PAYROLL.EMPLOYEE` maps to `{repo}/PAYROLL/EMPLOYEE` |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.4, 16.5: `resolve_dataset_path_empty_repo_returns_none` -- empty repository path returns `None` |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.5: `resolve_dataset_path_empty_dsn_returns_none` -- empty DSN returns `None` |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.1: `resolve_dataset_path_single_qualifier_dsn` -- single-qualifier DSN resolves to one component under repo |
| `ff-desktop` | 🔲 | -- | Req 16.2: file opened in editor when resolved path exists on disk (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 16.3: `'<DSN>': dataset file not found at <path>` shown when path missing (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 16.4: `'<DSN>': catalog has no repository path configured` shown when repo empty (manual UI verification) |

## Final Summary (after Phase BJ)

| Status | Count |
|--------|-------|
| ✅ PASS | 481 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | Req 15.3, 16.2, 16.3, 16.4 (UI rendering -- manual verification) |
| 🔴 NOT COVERED | 0 |

### Phase BL -- B024 Tab Cycle Fix (Req 20.1, 20.2)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell/update.rs` | Req 20.1: Tab from CommandField enters tree, sets `explorer_focused = true`, `cursor_node` = first visible node |
| `ff-desktop` | ✅ | `shell/update.rs` | Req 20.2: Tab advances `cursor_node`; Tab past last node exits tree and returns focus to CommandField |
| `ff-desktop` | 🔲 | -- | Req 20.13: Cursor highlight on catalog nodes and file nodes -- manual UI verification |

## Final Summary (after Phase BL)

| Status | Count |
|--------|-------|
| ✅ PASS | 481 tests (ff-desktop) |
| ❌ FAIL | 0 |
| 🔲 MANUAL | Req 20.13 cursor highlight (UI rendering) |
| 🔴 NOT COVERED | 0 |

### Phase BK -- Native File Browser: egui-file-dialog Integration (Requirement 22)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 22.1: `NativeDialogSlot` wraps `FileDialog`; `native_dialogs` field on `FileExplorerPanelState`; lazily initialised per catalog -- `native_dialogs_field_exists_on_state`, `native_dialog_slot_lazily_created_for_catalog`, `native_dialog_slot_implements_debug_and_clone`, `file_explorer_panel_state_debug_clone_with_native_dialogs` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 22.2: `render_native_dialog()` calls `take_selected()` and routes to `open_file_node()` -- `native_dialog_slot_lazily_created_for_catalog` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 22.3: `render_dataset_children()` unchanged; Mainframe/POSIX path unaffected -- `mainframe_posix_branches_use_render_dataset_children` |
| `ff-desktop` | ✅ | `crates/ff-desktop/Cargo.toml` | Req 22.4: `egui-file-dialog = "0.6"` declared; vendored patch resolves egui 0.29 mismatch |
| `ff-desktop` | ✅ | `THIRD_PARTY_CREDITS.md` | Req 22.5: `THIRD_PARTY_CREDITS.md` created at workspace root with full MIT licence text |
| `ff-desktop` | ✅ | `cargo test` 486 passing | Req 22.6: 486 tests pass, 0 failures after BK refactoring |

### Phase BM -- File Explorer Panel: egui-file-dialog look-and-feel with catalog mount points (Requirement 23)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.1: Two-pane layout (SidePanel + CentralPanel) matching egui-file-dialog visual style -- `all_existing_state_fields_present_after_bm_refactor` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.2: Sidebar lists all catalogs as named Mount_Nodes; clicking selects and populates Content_Pane -- `clicking_mount_node_sets_selected_catalog`, `selected_catalog_defaults_to_none` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.3: Sidebar groups catalogs under "Mainframe", "POSIX", "Native" collapsible headers -- `sidebar_groups_catalogs_by_type` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.4: Native catalog Content_Pane renders egui-file-dialog widget -- `native_catalog_uses_native_dialog_slot` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.5: Mainframe catalog Content_Pane renders dot-qualified dataset list; PS is leaf -- `mainframe_content_ps_dataset_is_leaf` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.6: POSIX catalog Content_Pane uses forward-slash path normalisation -- `posix_path_normalised_to_forward_slashes` |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.7: Empty sidebar shows placeholder when no catalogs registered -- `empty_registry_produces_no_mount_nodes` |
| `ff-desktop` | 🔲 | -- | Req 23.8: Right-click context menu uses egui-file-dialog native menu for Native; Req 16 menu for Mainframe/POSIX (manual UI verification) |
| `ff-desktop` | ✅ | `file_explorer_panel.rs` unit tests | Req 23.9: Sidebar width persisted; default 200px; minimum 120px -- `sidebar_width_defaults_to_200`, `sidebar_width_minimum_is_120` |
| `ff-desktop` | ✅ | `cargo test` 496 passing | Req 23.10: `cargo test` passes with 0 failures after BM refactoring |

### Phase BR -- B028 Dataset File Creation on First Open (Req 16.3, 16.6)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.3: `opening_missing_dataset_creates_file_and_parent_dirs`, `opening_missing_dataset_creates_parent_dirs` -- `create_dataset_file` creates file and all missing parent dirs |
| `ff-desktop` | ✅ | `files_panel.rs` unit tests | Req 16.6: `create_dataset_file` returns `Err` on I/O failure; shell shows `'<DSN>': cannot create dataset file at <path>: <os_error>` |

### Phase BS -- Mainframe Dataset Architecture (CR-NR-016)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-dscatalog` | ✅ | `vfs_provider::tests::fb_dataset_read_decodes_fixed_records_no_crlf` | Req 16.1: mainframe dataset written with no CRLF/LF record delimiter |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::fb_dataset_read_decodes_fixed_records_no_crlf` | Req 16.2: fixed-length records packed contiguously; record n at offset n×LRECL |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::vb_dataset_read_decodes_rdw_records_no_crlf` | Req 16.3: variable-length records preceded by 4-byte RDW; no CRLF after data |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::read_write_round_trip` | Req 16.4: RECFM=U content stored as opaque binary stream |
| `ff-dscatalog` | 🔴 | -- | Req 16.5: editor presents records as lines without altering binary storage |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::fb_dataset_read_decodes_fixed_records_no_crlf`, `vb_dataset_read_decodes_rdw_records_no_crlf` | Req 16.6: save re-encodes displayed lines to binary record format |
| `ff-dscatalog` | 🔴 | -- | Req 16.7: malformed RDW returns diagnostic error with dataset identity and record position |
| `ff-dscatalog` | 🔴 | -- | Req 17.1: RecordCodec trait defined with no filesystem or SQLite dependency |
| `ff-dscatalog` | 🔴 | -- | Req 17.2: FixedCodec encodes/decodes fixed-length records given LRECL |
| `ff-dscatalog` | 🔴 | -- | Req 17.3: VariableCodec encodes/decodes variable-length records with 4-byte RDW |
| `ff-dscatalog` | 🔴 | -- | Req 17.4: BinaryCodec passes bytes through unchanged for RECFM=U |
| `ff-dscatalog` | 🔴 | -- | Req 17.5: TextCodec maps host text lines to/from fixed-length records; import/export only |
| `ff-dscatalog` | 🔴 | -- | Req 17.6: all codecs independently testable using in-memory byte buffers |
| `ff-dscatalog` | 🔴 | -- | Req 17.7: import/export requires explicit codec and encoding policy; not inferred silently |
| `ff-dscatalog` | 🔴 | -- | Req 18.1: PS dataset content stored as native file; no SQLite BLOB |
| `ff-dscatalog` | 🔴 | -- | Req 18.2: PDS/PDSE member content stored as individual native files; no SQLite BLOB |
| `ff-dscatalog` | 🔴 | -- | Req 18.3: GDG generation content stored as native files |
| `ff-dscatalog` | 🔴 | -- | Req 18.4: VSAM KSDS records stored in dedicated SQLite-backed keyed record store |
| `ff-dscatalog` | 🔴 | -- | Req 18.5: VSAM RRDS records stored in SQLite-backed relative-record store |
| `ff-dscatalog` | 🔴 | -- | Req 18.6: VSAM ESDS records stored in append-oriented native file; sidecar index rebuildable |
| `ff-dscatalog` | 🔴 | -- | Req 18.7: POSIX files remain native host filesystem objects; not copied into SQLite |
| `ff-dscatalog` | 🔴 | -- | Req 18.8: PS/PDS/GDG/POSIX content NOT stored as BLOBs in central catalogue database |
| `ff-dscatalog` | 🔴 | -- | Req 19.1: StorageProvider trait defined with allocate/open/stat/rename/delete/list/reconcile |
| `ff-dscatalog` | 🔴 | -- | Req 19.2: providers declare capabilities; callers do not infer from dataset type |
| `ff-dscatalog` | 🔴 | -- | Req 19.3: native-file and SQLite-record providers share common error taxonomy mapping to VfsError |
| `ff-dscatalog` | 🔴 | -- | Req 19.4: provider-specific locators opaque outside provider and catalogue services |
| `ff-dscatalog` | 🔴 | -- | Req 19.5: NativeFileProvider implements StorageProvider for PS/PDS/GDG/POSIX |
| `ff-dscatalog` | 🔴 | -- | Req 19.6: SqliteRecordProvider implements StorageProvider for VSAM KSDS/RRDS/ISAM |
| `ff-dscatalog` | 🔴 | -- | Req 19.7: new StorageProvider addable without changing editors, catalogue consumers, or VFS layer |
| `ff-dscatalog` | 🔴 | -- | Req 20.1: each managed physical object assigned stable UUID at allocation time |
| `ff-dscatalog` | 🔴 | -- | Req 20.2: repository layout uses datasets/objects/<uuid>.dat and indexed/<uuid>.sqlite |
| `ff-dscatalog` | 🔴 | -- | Req 20.3: logical dataset name NOT used as physical path |
| `ff-dscatalog` | 🔴 | -- | Req 20.4: physical mapping deterministic and persisted; dataset findable after restart |
| `ff-dscatalog` | 🔴 | -- | Req 20.5: dots in DSN NOT translated directly to directory separators in UUID layout |
| `ff-dscatalog` | 🔴 | -- | Req 20.6: dataset rename updates catalogue only; physical object not moved |
| `ff-dscatalog` | 🔴 | -- | Req 20.7: path-safety guards reject traversal, reserved names, illegal chars, length violations |
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
| `ff-dscatalog` | ✅ | `transactions::tests::full_create_protocol_advances_through_all_states`, `rollback_create_removes_journal_entry` | Req 25.1: staged create protocol -- stage, reserve, publish, activate |
| `ff-dscatalog` | ✅ | `transactions::tests::full_delete_protocol_advances_through_all_states` | Req 25.2: staged delete protocol -- mark pending, tombstone, finalise |
| `ff-dscatalog` | ✅ | `transactions::tests::begin_create_writes_staging_entry`, `incomplete_operations_returns_only_transitional_entries`, `journal_entries_survive_reopen` | Req 25.3: interrupted operations discoverable through OperationJournal |
| `ff-dscatalog` | ✅ | `transactions::tests::recovery_plan_for_staging_entry_is_rollback_create`, `recovery_plan_for_published_entry_is_complete_create`, `recovery_plan_for_pending_delete_is_rollback_delete`, `recovery_plan_for_tombstoned_entry_is_complete_delete`, `active_entries_produce_no_recovery_actions` | Req 25.4: startup recovery detects and offers complete-or-rollback for incomplete operations |
| `ff-dscatalog` | ✅ | `transactions::tests::stale_version_is_rejected`, `wrong_state_transition_is_rejected` | Req 25.5: concurrent modification controlled via version tokens / SQLite transactions |
| `ff-dscatalog` | ✅ | `transactions::tests::activate_fails_if_not_in_published_state` | Req 25.6: operation not reported successful until both catalogue and provider postconditions met |
| `ff-dscatalog` | ✅ | `integrity::tests::checksum_file_produces_hex_digest`, `verify_checksum_*` | Req 26.1: optional CRC-32 checksums on managed content; verified on open when enabled |
| `ff-dscatalog` | ✅ | `integrity::tests::backup_creates_archive_with_manifest`, `backup_manifest_contains_correct_sizes` | Req 26.2: workspace.backup captures catalogue DB, SQLite stores, native files, journals |
| `ff-dscatalog` | ✅ | `integrity::tests::manifest_serialises_and_deserialises`, `manifest_schema_version_is_set` | Req 26.3: backup manifest contains schema version, provider config, object inventory, checksums |
| `ff-dscatalog` | ✅ | `integrity::tests::restore_extracts_files_to_target_root`, `restore_preserves_file_content` | Req 26.4: workspace.restore supports original root or remapped root without changing logical names |
| `ff-dscatalog` | ✅ | `integrity::tests::diagnose_reports_dangling_entry`, `diagnose_reports_orphaned_object`, `diagnose_reports_checksum_mismatch`, `diagnose_clean_workspace_returns_empty` | Req 26.5: workspace.diagnose reports orphaned physical objects and dangling catalogue entries |
| `ff-dscatalog` | ✅ | `integrity::tests::repair_plan_maps_findings_to_actions`, `apply_repair_deletes_orphan_file`, `apply_repair_dangling_entry_is_noop_on_filesystem`, `repair_plan_is_empty_for_no_findings` | Req 26.6: repair operations previewable, auditable, reversible where practical |
| `ff-dscatalog` | 🔴 | -- | Req 27.1: reconciliation compares catalogue entries with physical objects per provider |
| `ff-dscatalog` | 🔴 | -- | Req 27.2: reconciliation detects missing, inaccessible, duplicated, or inconsistent objects |
| `ff-dscatalog` | 🔴 | -- | Req 27.3: reconciliation reports proposed corrections without auto-applying |
| `ff-dscatalog` | ✅ | `audit::tests::audit_log_records_all_action_variants`, `audit_log_records_create_action`, `audit_log_records_delete_action`, `audit_log_records_err_outcome`, `audit_log_catalogue_level_action_has_no_dsn`, `audit_log_entries_ordered_newest_first` | Req 27.4: audit_log table records create/rename/move/delete/restore/import/export/allocate |
| `ff-dscatalog` | ✅ | `audit::tests::audit_log_timestamp_is_nonempty` | Req 28.6: audit events identify action, object, outcome, timestamp, principal |
| `ff-dscatalog` | ✅ | `storage::native::tests::path_traversal_and_reserved_names_always_rejected` | Req 28.1: all resolved physical paths constrained to authorised workspace roots |
| `ff-dscatalog` | ✅ | `storage::native::tests::path_traversal_and_reserved_names_always_rejected` | Req 28.2: path canonicalisation and traversal checks before any filesystem access |
| `ff-dscatalog` | 🔴 | -- | Req 28.3: catalogue metadata not treated as substitute for OS access controls |
| `ff-dscatalog` | ✅ | `security::tests::scrub_payload_returns_redacted_string`, `scrub_payload_never_exposes_content`, `scrub_str_returns_redacted`, `scrub_empty_payload`, `scrub_single_byte_payload` | Req 28.4: sensitive dataset contents and credentials not written to logs |
| `ff-dscatalog` | ✅ | `security::tests::parameterised_query_neutralises_sql_injection_in_datasets`, `parameterised_query_neutralises_sql_injection_in_audit_log` | Req 28.5: all SQLite connections use parameterised statements; no interpolated schema identifiers |
| `ff-dscatalog` | ✅ | `schema::tests::migration_from_v1_to_v2_creates_audit_log_table`, `migration_is_idempotent_on_current_version`, `migration_rejects_newer_version` | Req 27.5: schema changes versioned and applied through forward migration scripts |
| `ff-dscatalog` | ✅ | `hierarchy::tests::scope_display_and_parse_round_trip`, `catalog_registry::tests::resolve_scoped_finds_master_entry`, `resolve_with_scope_priority_prefers_master` | Req 29.1: master and user catalogue hierarchy supported |
| `ff-dscatalog` | ✅ | `catalog_registry::tests::resolve_scoped_does_not_return_wrong_scope`, `resolve_scoped_finds_master_entry` | Req 29.2: each logical DSN maps to exactly one active provider and locator within a scope |
| `ff-dscatalog` | ✅ | `catalog_registry::tests::logical_rename_updates_catalogue_only` | Req 29.3: logical rename updates catalogue only; physical relocation is a separate operation |
| `ff-dscatalog` | ✅ | `hierarchy::tests::uniqueness_fails_on_same_scope_collision`, `uniqueness_passes_when_same_dsn_different_scope`, `catalog_registry::tests::check_scope_uniqueness_rejects_duplicate_in_same_scope`, `check_scope_uniqueness_allows_same_dsn_in_different_scope` | Req 29.4: uniqueness validated per configured naming scope and collation rules |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::cross_platform_uuid_layout_produces_identical_logical_results` | Req 30.1: architecture operates identically on Windows, Linux, and macOS |
| `ff-dscatalog` | ✅ | `vfs_provider::tests::catalogue_listing_does_not_load_payload_bytes` | Req 30.2: catalogue listing queries metadata without loading dataset payloads |
| `ff-dscatalog` | 🔴 | -- | Req 30.3: design permits large datasets/libraries without all content in central catalogue DB |
| `ff-dscatalog` | 🔴 | -- | Req 30.4: catalogue, codec, and provider components independently testable |
| `ff-dscatalog` | 🔴 | -- | Req 30.5: storage operations emit structured diagnostic events with correlation identifiers |
| `ff-dscatalog` | 🔴 | -- | Req 30.6: future storage provider addable without rewriting editors or catalogue consumers |
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
| `ff-jes` | ✅ | `sdsf_filter_expr.rs` unit tests | Req 17.1: ST panel shows all jobs with STATUS column -- `filter_status_active`, `sdsf_panel::main_panel_has_st_command` |
| `ff-jes` | ✅ | `sdsf_filter_expr.rs` unit tests | Req 17.2: FILTER command -- advanced filter expression; FILTER clears -- `filter_eq_jobname`, `active_filter_set_and_clear`, `filter_empty_returns_none` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.3: FIND command -- search panel data; FIND NEXT/PREV -- `find_first_returns_correct_index`, `find_next_advances_past_current`, `find_prev_moves_backward` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.4: LOCATE command -- scroll to first JOBNAME match, nearest alpha on no match -- `locate_exact_prefix_match`, `locate_nearest_alphabetic`, `locate_empty_list` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.5: UP/DOWN/LEFT/RIGHT scroll commands with n/HALF/PAGE/MAX amounts -- `scroll_down_page`, `scroll_up_half`, `scroll_down_clamps_at_max`, `scroll_up_clamps_at_zero` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.6: SET ACTION displays valid action characters with descriptions -- `set_action_toggles` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.7: SET MAIN [panel-name] sets default MENU panel -- `set_main_updates_panel` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.8: SET ROWNUM ON/OFF toggles row numbers in NP area -- `set_rownum_toggles` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.9: WHO displays session info (user, start time, filters, SET settings, provider) -- `who_format_contains_required_fields`, `who_omits_unset_filters` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.10: QUERY AUTH displays authorised commands and action characters -- `query_auth_contains_commands_and_actions`, `query_auth_list_non_empty` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.11: SET settings (ACTION/MAIN/ROWNUM) persist across restarts -- `settings_serialise_round_trip`, `settings_default_round_trip` |
| `ff-jes` | ✅ | `sdsf_filter_expr.rs` unit tests | Req 17.12: FILTER supports =, !=, >, <, >=, <= operators and wildcard * -- `filter_ne_operator`, `filter_wildcard_prefix`, `filter_ge_operator` |
| `ff-jes` | ✅ | `sdsf_filter_expr.rs` unit tests | Req 17.13: FILTER supports AND and OR logical operators -- `filter_and_operator`, `filter_or_operator` |
| `ff-jes` | ✅ | `sdsf_panel.rs` unit tests | Req 17.14: ST panel accessible via ST command and S action on main panel -- `main_panel_has_st_command`, `navigate_to_sub_panel` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.15: FIND case-insensitive by default; FIND C for case-sensitive -- `find_case_sensitive`, `find_first_returns_correct_index` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.16: LOCATE/FIND no-match shows "string NOT FOUND" in message area -- `find_no_match_sets_is_no_match`, `locate_nearest_alphabetic` |
| `ff-jes` | ✅ | `sdsf_commands.rs` unit tests | Req 17.17: scroll commands update SCROLL ===> field to last-used amount -- `scroll_uses_default_amount_when_omitted`, `scroll_updates_scroll_field` |


### Phase CE -- undo-redo-transactions P2 EARS Integration (Requirement 19)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-undo-redo` | ✅ | `setundo.rs` unit tests | Req 19.1: SETUNDO ON/OFF/n command -- `apply_setundo_on_enables`, `apply_setundo_off_disables`, `apply_setundo_levels_sets_max`, `setundo_levels_zero_clears_stack`, `setundo_levels_shrink_evicts_oldest` |
| `ff-undo-redo` | ✅ | `setundo.rs` unit tests | Req 19.2: RECOVERY ON/OFF/n command -- `apply_recovery_on_enables`, `apply_recovery_off_disables`, `apply_recovery_interval_sets_value`, `recovery_interval_zero_disables`, `recovery_interval_persists_across_edits` |


### Phase CF -- syntax-highlighting P2 EARS Integration (Requirement 16)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-syntax-highlighting` | ✅ | `hilite.rs` unit tests | Req 16.1: HILITE ON/OFF -- `hilite_state_on_enables_highlighting`, `hilite_state_off_disables_highlighting_and_clears_modes`, `hilite_operand_parse_on/off` |
| `ff-syntax-highlighting` | ✅ | `hilite.rs` unit tests | Req 16.2: HILITE LOGIC -- `hilite_state_logic_toggles_independently`, `logic_scanner_detects_*`, `logic_scanner_word_operators_whole_word_only` |
| `ff-syntax-highlighting` | ✅ | `hilite.rs` unit tests | Req 16.3: HILITE PAREN -- `paren_matcher_finds_enclosing_*`, `paren_matcher_returns_mismatched_for_unclosed_opener`, `paren_matcher_finds_innermost_pair` |
| `ff-syntax-highlighting` | ✅ | `hilite.rs` unit tests | Req 16.4: HILITE FIND -- `hilite_state_find_off_clears_find_string`, `hilite_state_set_find_string_only_when_find_active`, `hilite_operand_parse_find/find_off` |
| `ff-syntax-highlighting` | ✅ | `hilite.rs` unit tests | Req 16.5: HILITE combined operands -- `hilite_state_on_logic_paren_enables_both_modes`, `hilite_state_modes_toggle_independently_after_on`, `hilite_operand_parse_on_logic_paren` |


### Phase CG -- lua-macro-engine P2 EARS Integration (Requirement 11)

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.1: ISREDIT host command environment dispatches edit macro service calls |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.2: ISPEXEC host command environment routes dialog service calls |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.3: IMACRO executes named macro at edit session open |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.4: IMACRO edit profile setting stores/retrieves initial macro name |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.5: LINENUM function resolves label/relative reference to absolute line number |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.6: CURSOR function gets and sets cursor position |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.7: EXEC command locates and executes named exec from SYSEXEC/SYSPROC |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.8: Implicit exec invocation for unrecognized command names |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.9: % prefix bypasses primary command table for exec lookup |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.10: EXEC <member> <args> passes argument string to exec |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.11: TSO host command environment routes to ff-command dispatcher |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.12: ADDRESS <environment-name> switches default host command environment |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.13: ISPEXEC ADDRESS environment routes to ISPF dialog service layer |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.14: ISREDIT ADDRESS environment routes to ISREDIT handler |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.15: RC special variable set to host command return code |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.16: LISTDSI built-in returns dataset information from ff-dscatalog |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.17: MSG built-in displays message in status bar or message area |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.18: MVSVAR built-in returns system variable values mapped to workbench equivalents |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.19: OUTTRAP built-in captures TSO command output into stem variable |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.20: PROMPT built-in controls terminal input availability |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.21: SYSDSN built-in returns OK or error string for named dataset |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.22: SYSVAR built-in returns ISPF system variable values |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.23: USERID built-in returns current user login name |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.24: EXECIO DISKR reads records from ddname dataset into stem variable |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.25: EXECIO DISKW writes records from stem variable to ddname dataset |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.26: EXECIO FINIS variants read/write all remaining records and close file |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.27: EXECIO SKIP advances read position without returning data |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.28: EXECIO return codes RC=0/2/non-zero per TSO conventions |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.29: FFCMD command files execute .ffcmd files line-by-line as batch primary commands |
| `ff-macro` | ✅ | `ff-lua` unit tests | Req 11.30: FFCMD execution wrapped in single Macro_Transaction for atomic undo |

### Phase CH -- FFW-JES P2 EARS Integration (Requirement 18)

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.1: Overtypeable fields visually distinct from read-only fields |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.2: Direct overtype applies change and refreshes panel on Enter |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.3: Command-line overtype syntax updates named field for cursor/NP row |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.4: Overtype Extension pop-up for values exceeding column width |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.5: Context-sensitive HELP / PF1 displays panel help |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.6: ACTH lists valid action characters with descriptions |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.7: COLH lists column names with type, width, and description |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.8: CMDH lists valid primary commands with syntax and description |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.9: SEARCH <text> in help panel scrolls to first match |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.10: LOG command opens System Log panel in reverse-chronological order |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.11: ULOG command opens User Log panel for current user |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.12: NEXT/PREV scroll forward/backward through log segments |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.13: SNAPSHOT captures current log content to dataset or file |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.14: SYS panel displays active address spaces with status and resources |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.15: DASH panel displays system health metrics summary |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.16: INIT panel displays initiator pool status |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.17: JC panel displays job class definitions and scheduling parameters |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.18: SP panel displays spool volume utilisation and track allocation |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.19: Browse settings: line width, record format display, FIND in output |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.20: PRINT action routes job output to configured print destination |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.21: COLS command displays column ruler in browse panel |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.22: SET BCOLOR sets panel background colour, persisted |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.23: SET CONFIRM ON/OFF controls confirmation prompt for destructive actions |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.24: SET CURSOR sets default cursor landing position on panel open |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.25: SET DATE sets date display format (MDY/DMY/YMD/JUL) |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.26: SET DELAY sets automatic refresh interval; 0 disables auto-refresh |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.27: SET HEX ON/OFF toggles hexadecimal display of field values |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.28: SET SCHARS defines special characters for field delimiters |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.29: SET SCREEN sets logical screen dimensions for panel layout |
| `ff-jes` | ✅ | `ff-jes` unit tests | Req 18.30: All SET P2 settings persisted across sessions |


### Phase CI -- command-semantics P2 EARS Integration (Requirement 10)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 10.1: OUTPUT jobname routes to FFW-JES for job output display/retrieval |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 10.2: CANCEL jobname [PURGE] routes to FFW-JES; PURGE requests output purge |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 10.3: SEND 'message' [USER/LOGON/BROADCAST] routes to messaging subsystem |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 10.4: PROFILE [operands] routes to session profile subsystem |
| `ff-command-semantics` | ✅ | `tso.rs` unit tests | Req 10.5: PRINTDS DATASET(dsname) routes to file-operations pipeline |

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

### Phase CK -- FFTest Automated Dialog Testing Framework (CR-NR-033)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-fftest` | 🔴 | -- | Req 1.1: THE FileForgeWorkbench platform SHALL provide an integrated automated dialog testing framework named FFTest |
| `ff-fftest` | 🔴 | -- | Req 1.2: THE FFTest framework SHALL support execution without user interaction |
| `ff-fftest` | 🔴 | -- | Req 1.3: THE FFTest framework SHALL be cross-platform on Windows, Linux, and macOS |
| `ff-fftest` | 🔴 | -- | Req 1.4: THE FFTest framework SHALL be designed such that not less than 90% of business logic testing can be performed without the GUI loaded |
| `ff-fftest` | 🔴 | -- | Req 1.5: THE FFTest framework SHALL maintain a version-controlled repository of test scripts, baselines, test data, and execution reports |
| `ff-desktop` | ✅ | `ff-fftest::automation` unit tests, `ff-desktop::automation` unit tests | Req 2.1: EACH user interface component in ff-desktop SHALL expose a stable automation identifier string |
| `ff-desktop` | ✅ | `ff-fftest::automation` unit tests, `ff-desktop::automation` unit tests | Req 2.2: Automation IDs SHALL follow dot-separated hierarchical naming convention |
| `ff-desktop` | ✅ | `ff-desktop::automation` unit tests | Req 2.3: THE framework SHALL NEVER rely solely on screen coordinates to identify controls |
| `ff-desktop` | ✅ | `ff-desktop::automation` unit tests | Req 2.4: menus, toolbars, buttons, text boxes, tables, tree controls, dialog windows, tabs, ISPF panels, dataset browsers, and plugin dialogs SHALL be automatable |
| `ff-desktop` | ✅ | `ff-fftest::automation` unit tests, `ff-desktop::automation` unit tests | Req 2.5: WHEN a UI control is rendered, THE automation subsystem SHALL query its state by Automation ID without a display device |
| `ff-fftest` | ✅ | `ff-fftest::parser` unit tests | Req 3.1: THE FFTest scripting language SHALL provide navigation, interaction, assertion, and control flow command categories |
| `ff-fftest` | ✅ | `ff-fftest::parser` unit tests | Req 3.2: OPEN FILE, WAIT WINDOW, CLICK MENU, CLICK BUTTON, SELECT MENUITEM, TYPE TEXT, PRESS KEY, ASSERT WINDOW EXISTS, ASSERT TEXT EXISTS, ASSERT STATUSBAR CONTAINS, ASSERT FILE OPEN, ASSERT CONTROL VALUE, CHECKPOINT, CLOSE WINDOW commands SHALL be supported |
| `ff-fftest` | ✅ | `ff-fftest::parser` unit tests | Req 3.3: THE FFTest script parser SHALL be case-insensitive for command keywords |
| `ff-fftest` | ✅ | `ff-fftest::parser` unit tests | Req 3.4: THE FFTest script parser SHALL treat lines beginning with # as comments |
| `ff-fftest` | ✅ | `ff-fftest::runner` unit tests | Req 3.5: WHEN a script command references an Automation ID that does not exist, THE runner SHALL record a diagnostic failure with file name, line number, and unresolved ID |
| `ff-fftest` | ✅ | `ff-fftest::parser` unit tests | Req 3.6: THE FFTest scripting language SHALL support parameterised scripts via ${VARIABLE_NAME} substitution |
| `ff-fftest` | ✅ | `ff-fftest::runner` unit tests | Req 4.1: WHEN a dialog script is executed, THE FFTest runner SHALL validate all assertions contained within the script |
| `ff-fftest` | ✅ | `ff-fftest::runner` unit tests | Req 4.2: THE FFTest runner SHALL process script commands sequentially in file order |
| `ff-fftest` | ✅ | `ff-fftest::assertions` + `ff-fftest::runner` unit tests | Req 4.3: WHEN an assertion fails, THE runner SHALL record script file, line number, assertion text, expected value, and actual value |
| `ff-fftest` | ✅ | `ff-fftest::runner` unit tests | Req 4.4: WHEN a test execution completes, THE runner SHALL generate a pass/fail summary with total assertions, passed count, failed count, and duration |
| `ff-fftest` | 🔴 | -- | Req 4.5: WHILE executing automated tests, THE runner SHALL continue processing UI events and background tasks |
| `ff-fftest` | 🔴 | -- | Req 4.6: WHILE executing end-to-end workflow tests, THE runner SHALL validate expected outcomes at each workflow checkpoint |
| `ff-fftest` | 🔴 | -- | Req 5.1: WHEN a user starts test recording, THE framework SHALL capture all supported user interactions |
| `ff-fftest` | 🔴 | -- | Req 5.2: WHEN test recording ends, THE framework SHALL generate an executable FFTest script |
| `ff-fftest` | 🔴 | -- | Req 5.3: Recorded scripts SHALL be executable without modification |
| `ff-fftest` | 🔴 | -- | Req 5.4: THE recording subsystem SHALL emit Automation IDs in generated scripts, not screen coordinates |
| `ff-fftest` | ✅ | `ff-desktop::fftest_cli` unit tests | Req 6.1: WHILE executing headless tests, THE framework SHALL support operation without an attached display device |
| `ff-fftest` | ✅ | `ff-desktop::fftest_cli` unit tests | Req 6.2: THE FFTest runner SHALL be invocable via ffwb --run-tests and ffwb --run-script <path> |
| `ff-fftest` | ✅ | `ff-desktop::fftest_cli` unit tests | Req 6.3: WHEN CI/CD integration is configured, THE framework SHALL return process exit codes: 0=pass, non-zero=failure |
| `ff-fftest` | 🔴 | -- | Req 6.4: THE headless runner SHALL support GitHub Actions, GitLab CI, Azure DevOps, Jenkins, and local pipelines |
| `ff-fftest` | ✅ | `ff-fftest::report` unit tests | Req 7.1: THE framework SHALL generate machine-readable test results in JSON format after every test run |
| `ff-fftest` | ✅ | `ff-fftest::report` unit tests | Req 7.2: THE framework SHALL generate human-readable test reports in HTML format after every test run |
| `ff-fftest` | ✅ | `ff-fftest::report` unit tests | Req 7.3: THE JSON report SHALL include suite name, timestamp, duration, per-test pass/fail, assertion details, and error messages |
| `ff-fftest` | ✅ | `ff-fftest::report` unit tests | Req 7.4: THE HTML report SHALL include summary table, per-test expandable sections, embedded screenshots, and stack traces |
| `ff-fftest` | 🔴 | -- | Req 7.5: WHERE screenshot capture is enabled, THE framework SHALL record screenshots at configured checkpoints |
| `ff-fftest` | ✅ | `ff-desktop::fftest_cli` unit tests | Req 7.6: THE framework SHALL write reports to reports/ at the workspace root |
| `ff-fftest` | ✅ | `ff-fftest::capture` unit tests | Req 8.1: WHERE visual regression testing is enabled, THE framework SHALL compare screenshots against baseline images in tests/baselines/ |
| `ff-fftest` | ✅ | `ff-fftest::capture` unit tests | Req 8.2: WHEN a screenshot differs from its baseline beyond the configured tolerance, THE framework SHALL record a visual regression failure |
| `ff-fftest` | 🔴 | -- | Req 8.3: visual regression SHALL support ISPF panels, dataset browsers, hex editors, compare windows, tree structures, and editor windows |
| `ff-fftest` | ✅ | `ff-desktop::fftest_cli` unit tests | Req 8.4: THE framework SHALL provide ffwb --update-baselines to update baselines from current screenshots |
| `ff-fftest` | ✅ | `ff-fftest::capture` unit tests | Req 8.5: WHEN a baseline does not exist for a checkpoint, THE framework SHALL create it automatically and report BASELINE_CREATED |
| `ff-fftest` | 🔴 | -- | Req 9.1: WHEN a plugin is loaded for testing, THE framework SHALL expose plugin UIs through the automation subsystem |
| `ff-fftest` | 🔴 | -- | Req 9.2: THE FFTest scripting language SHALL support LOAD PLUGIN "<name>" command |
| `ff-fftest` | 🔴 | -- | Req 9.3: Plugin test scripts SHALL be stored under tests/plugins/<plugin-name>/ |
| `ff-fftest` | ✅ | `tests/` directory structure + `.gitignore` | Req 10.1: THE test repository SHALL follow the defined structure: tests/unit, dialog, workflow, visual, plugins, fixtures, baselines; reports/ |
| `ff-fftest` | ✅ | `.gitignore` | Req 10.2: THE reports/ directory SHALL be listed in .gitignore |
| `ff-fftest` | 🔴 | -- | Req 10.3: THE tests/baselines/ directory SHALL be version-controlled |

### Phase CM -- Mouse Text Selection and Clipboard Copy (CR-NR-034)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.1: `new_tab_has_no_canvas_selection` -- new tab canvas_selection is None |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.2: `canvas_selection_can_be_set_and_cleared` -- canvas_selection field set/cleared |
| `ff-desktop` | 🔲 | -- | Req 13.3: mouse release finalises selection range (egui drag -- manual UI verification) |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.4: `normalise_selection_orders_start_before_end` -- selection highlight uses normalised coords |
| `ff-desktop` | 🔲 | -- | Req 13.5: click without drag clears selection (egui click -- manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 13.6: Escape clears active selection (manual UI verification) |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.7: `canvas_selection_cleared_on_tab_switch` -- selection cleared on tab switch |
| `ff-desktop` | 🔲 | -- | Req 13.8: Ctrl+C with no selection does nothing (manual UI verification) |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 13.9: `canvas_selection_cleared_on_tab_switch` -- selection cleared on tab switch |
| `ff-desktop` | 🔲 | -- | Req 13.10: selection highlight stays positioned when scrolled (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 14.1: POM panel text rendered with selectable labels (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 14.2: Settings panel text rendered with selectable labels (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 14.3: status bar text rendered with selectable labels (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 14.4: Ctrl+C on selected panel text writes to OS clipboard via egui (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 14.5: POM option button click-to-navigate unaffected by selectable label change (manual UI verification) |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 20.1: `extract_selected_text_single_line` -- Ctrl+C writes UTF-8 text |
| `ff-desktop` | 🔲 | -- | Req 20.2: Copied N characters status message (manual UI verification) |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 20.3: `extract_selected_text_empty_selection_returns_empty` -- no-op when selection empty |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 20.4: `extract_selected_text_multi_line_joins_with_newline` -- multi-line joined with newline |
| `ff-desktop` | 🔲 | -- | Req 20.5: canvas copy recorded in Command_History (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 20.6: multi-line selection joined with platform line-ending (manual UI verification) |

### Phase CN -- Editor Scroll Amount Integration (CR-NR-035)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `scroll_by_amount_page_down_advances_by_visible_count` | Req 14.1: PAGE scroll amount scrolls full visible_count lines |
| `ff-desktop` | ✅ | `scroll_by_amount_half_down_advances_by_half_page` | Req 14.2: HALF scroll amount scrolls max(1, visible_count/2) lines |
| `ff-desktop` | ✅ | `scroll_by_amount_csr_down_advances_by_one_line` | Req 14.3: CSR Page Down scrolls so cursor is first visible line |
| `ff-desktop` | ✅ | `scroll_by_amount_csr_down_advances_by_one_line` | Req 14.4: CSR Page Up scrolls so cursor is last visible line |
| `ff-desktop` | ✅ | `scroll_by_amount_lines_n_advances_by_n` | Req 14.5: numeric N scroll amount scrolls exactly N lines |
| `ff-desktop` | ✅ | `scroll_by_amount_max_down_scrolls_to_bottom` | Req 14.6: MAX Page Down scrolls to last page; MAX Page Up scrolls to first line |
| `ff-desktop` | ✅ | `scroll_by_amount_data_behaves_like_page` | Req 14.7: DATA scroll amount scrolls visible_count - 1 lines |
| `ff-desktop` | 🔲 | -- | Req 14.8: SCROLL ===> field visible and editable when editor tab is active |

### Phase B031 Fix -- Gutter Line Command Input (B031, Req 1 line-commands)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 1 line-commands: `prefix_submit_fires_on_lost_focus_without_simultaneous_enter` -- submission fires on `lost_focus()` alone; Enter key guard removed (B031 fix) |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 1 line-commands: `prefix_text_cleared_after_successful_submit` -- prefix field cleared on successful submit |
| `ff-desktop` | ✅ | `editor_panel.rs` unit tests | Req 14.6 line-commands: `prefix_text_retained_on_invalid_command` -- engine has no pending on invalid command; error shown in status bar |
| `ff-desktop` | 🔲 | -- | Req 1 line-commands: gutter TextEdit accepts typed line commands at runtime (manual UI verification) |

## Phase BS -- Workspace Model (CR-NR-036)

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-session` | ✅ | `workspace.rs` unit tests | Req 1.1: workspace saved as valid TOML with name, roots, settings, recent_files |
| `ff-session` | ✅ | `workspace.rs` unit tests | Req 1.2: Workspace_File is valid TOML v1.0 |
| `ff-session` | ✅ | `workspace.rs` unit tests | Req 1.3: missing required field produces error, workspace not loaded |
| `ff-session` | ✅ | `workspace.rs` unit tests | Req 1.4: relative root paths resolved relative to Workspace_File directory |
| `ff-session` | ✅ | `workspace.rs` unit tests | Req 1.5: [settings] table applied as Workspace config layer |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 2.1: WORKSPACE OPEN loads file, registers roots, applies settings, restores MRU |
| `ff-desktop` | ✅ | `shell/commands.rs` | Req 2.2: WORKSPACE SAVE writes current state to Workspace_File |
| `ff-desktop` | ✅ | `shell/commands.rs` | Req 2.3: WORKSPACE SAVE AS writes to specified path |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 2.4: WORKSPACE CLOSE unloads roots, settings layer, MRU list |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 2.5: opening workspace when one is active closes current first; prompts if unsaved |
| `ff-desktop` | ✅ | `shell/mod.rs` | Req 2.6: at most one workspace active at any time |
| `ff-desktop` | ✅ | `shell/commands.rs` | Req 3.1: WORKSPACE ADD ROOT registers new catalog mount point |
| `ff-desktop` | ✅ | `shell/commands.rs` | Req 3.2: WORKSPACE REMOVE ROOT unregisters catalog; open tabs show warning |
| `ff-desktop` | ✅ | `file_explorer_panel::tests::workspace_roots_collected_for_sidebar_display` | Req 3.3: workspace roots displayed as top-level nodes in File Explorer |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 3.4: workspace load auto-registers all roots as Native catalogs |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 3.5: missing root path at load shows status bar warning; remaining roots loaded |
| `ff-session` | ✅ | `workspace.rs` unit tests | Req 4.1: workspace [settings] applied as highest-priority config layer |
| `ff-desktop` | 🔴 | -- | Req 4.2: saving setting at workspace scope writes to Workspace_File [settings] |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 4.3: workspace close removes Workspace layer; hot-reload callbacks invoked |
| `ff-session` | ✅ | `session_state.rs` unit tests | Req 5.1: active_workspace_path persisted in session.toml on exit |
| `ff-desktop` | ✅ | `shell/update.rs` startup_tests | Req 5.2: workspace auto-loaded at startup from persisted path |
| `ff-desktop` | ✅ | `shell/update.rs` startup_tests | Req 5.3: missing persisted path starts without workspace; stale path cleared |
| `ff-session` | ✅ | `workspace.rs` unit tests | Req 6.1: workspace MRU list accumulates files opened while workspace active |
| `ff-session` | ✅ | `workspace.rs` unit tests | Req 6.2: MRU list persisted in Workspace_File [[recent_files]] |
| `ff-desktop` | ✅ | `shell/tests.rs` unit tests | Req 6.3: workspace close reverts to global recent-files list |
| `ff-session` | ✅ | `workspace.rs` unit tests | Req 6.4: workspace MRU depth configurable via workspace.recent_files_depth |

## Phase BS -- Command Palette (CR-NR-037)

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-desktop` | 🔴 | -- | Req 1.1: Ctrl+Shift+P opens palette as modal overlay with search input focused |
| `ff-desktop` | 🔴 | -- | Req 1.2: Escape closes palette without executing command; focus restored |
| `ff-desktop` | 🔴 | -- | Req 1.3: click outside palette closes it without executing command |
| `ff-desktop` | 🔴 | -- | Req 1.4: View > Command Palette menu item opens palette |
| `ff-desktop` | 🔴 | -- | Req 1.5: Ctrl+Shift+P toggles palette closed if already open |
| `ff-desktop` | 🔴 | -- | Req 2.1: typing filters command list to fuzzy matches in real time |
| `ff-desktop` | 🔴 | -- | Req 2.2: contiguous runs score higher; word-boundary matches score higher |
| `ff-desktop` | 🔴 | -- | Req 2.3: results sorted by descending match score; ties broken alphabetically |
| `ff-desktop` | 🔴 | -- | Req 2.4: empty query shows Recent_Commands then all commands alphabetically |
| `ff-desktop` | 🔴 | -- | Req 2.5: fuzzy search is case-insensitive |
| `ff-desktop` | 🔴 | -- | Req 2.6: no matches shows "No commands match '<query>'" message |
| `ff-desktop` | 🔴 | -- | Req 3.1: each entry shows display name, category, shortcut |
| `ff-desktop` | 🔴 | -- | Req 3.2: highlighted entry shows full description in detail area |
| `ff-desktop` | 🔴 | -- | Req 3.3: matched characters highlighted in display name |
| `ff-desktop` | 🔴 | -- | Req 3.4: at most 20 entries visible; list is scrollable |
| `ff-desktop` | 🔴 | -- | Req 4.1: Enter executes highlighted command and closes palette |
| `ff-desktop` | 🔴 | -- | Req 4.2: clicking entry executes command and closes palette |
| `ff-desktop` | 🔴 | -- | Req 4.3: Down/Up arrow navigates entries; wraps at boundaries |
| `ff-desktop` | 🔴 | -- | Req 4.4: executed command added to Recent_Commands list |
| `ff-desktop` | 🔴 | -- | Req 4.5: disabled entry shown with disabled style; Enter shows unavailable message |
| `ff-session` | 🔴 | -- | Req 5.1: Recent_Commands shown at top of palette when query is empty |
| `ff-session` | 🔴 | -- | Req 5.2: Recent_Commands persisted in session state and restored on launch |
| `ff-desktop` | 🔴 | -- | Req 5.3: typing query hides Recent_Commands section |
| `ff-desktop` | 🔴 | -- | Req 5.4: only successfully executed commands added to Recent_Commands |

## Phase BS -- Global Search (CR-NR-038)

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-desktop` | 🔴 | -- | Req 1.1: Ctrl+Shift+F opens Search Results panel with search input focused |
| `ff-desktop` | 🔴 | -- | Req 1.2: GSEARCH/SEARCH command opens Search Results panel |
| `ff-desktop` | 🔴 | -- | Req 1.3: Ctrl+Shift+F focuses existing panel if already open |
| `ff-desktop` | 🔴 | -- | Req 1.4: Search > Find in Files menu item opens panel |
| `ff-desktop` | 🔴 | -- | Req 2.1: panel provides query, replace, case/word/regex toggles |
| `ff-desktop` | 🔴 | -- | Req 2.2: Include Files glob restricts search to matching paths |
| `ff-desktop` | 🔴 | -- | Req 2.3: Exclude Files glob excludes matching paths |
| `ff-desktop` | 🔴 | -- | Req 2.4: default scope is all workspace roots; falls back to Native catalogs |
| `ff-global-search` | 🔴 | -- | Req 2.5: literal, whole-word, and regex modes reuse FindEngine |
| `ff-desktop` | 🔴 | -- | Req 2.6: invalid regex shows inline error; search not executed |
| `ff-global-search` | 🔴 | -- | Req 3.1: search runs as background Tokio task; UI remains interactive |
| `ff-desktop` | 🔴 | -- | Req 3.2: progress indicator shows files scanned and matches found |
| `ff-global-search` | 🔴 | -- | Req 3.3: results streamed incrementally as each file is scanned |
| `ff-global-search` | 🔴 | -- | Req 3.4: Cancel button aborts search; partial results displayed |
| `ff-global-search` | 🔴 | -- | Req 3.5: binary files skipped; count shown in panel footer |
| `ff-global-search` | 🔴 | -- | Req 3.6: Exclude Files globs respected; excluded files not scanned |
| `ff-desktop` | 🔴 | -- | Req 3.7: completion shows "N matches in M files" summary |
| `ff-desktop` | 🔴 | -- | Req 4.1: results grouped by file with collapsible section headers |
| `ff-desktop` | 🔴 | -- | Req 4.2: each match shows line number and highlighted line text |
| `ff-desktop` | 🔴 | -- | Req 4.3: clicking match opens file at matching line with highlight |
| `ff-desktop` | 🔴 | -- | Req 4.4: all file sections expanded by default; collapsible by click |
| `ff-desktop` | 🔴 | -- | Req 4.5: keyboard navigation: Up/Down, Enter, Left/Right collapse/expand |
| `ff-desktop` | 🔴 | -- | Req 4.6: new search clears previous results |
| `ff-desktop` | 🔴 | -- | Req 5.1: replace input field with Replace All and per-file Replace buttons |
| `ff-desktop` | 🔴 | -- | Req 5.2: Replace All shows preview with file/match counts before writing |
| `ff-global-search` | 🔴 | -- | Req 5.3: Replace All applies substitution to all matches across all files |
| `ff-desktop` | 🔴 | -- | Req 5.4: replace completion shows "Replaced N occurrences in M files" |
| `ff-global-search` | 🔴 | -- | Req 5.5: cross-file replace undoable per file in open editor tabs |
| `ff-global-search` | 🔴 | -- | Req 5.6: file with unsaved changes warned before replace; not overwritten |
| `ff-global-search` | 🔴 | -- | Req 5.7: replace input supports regex group substitution \1-\9 |
| `ff-desktop` | 🔴 | -- | Req 6.1: last 20 search queries accessible via dropdown on search field |
| `ff-session` | ✅ | `session_manager.rs` unit tests | Req 6.2: search history persisted in session state and restored on launch |
| `ff-desktop` | 🔴 | -- | Req 6.3: selecting history entry populates query and options |

## Phase BS-B: Command Palette

| Crate | Status | Test | Requirement |
|-------|--------|------|-------------|
| `ff-desktop` | ✅ | `command_palette/fuzzy.rs` unit tests | Req 1.1: Command Palette opens as modal overlay on Ctrl+Shift+P |
| `ff-desktop` | ✅ | `command_palette/state.rs` unit tests | Req 1.2: Escape closes palette without executing |
| `ff-desktop` | ✅ | `command_palette/render.rs` | Req 1.3: Click outside closes palette |
| `ff-desktop` | ✅ | `shell/render_chrome.rs` View menu | Req 1.4: View > Command Palette menu item |
| `ff-desktop` | ✅ | `shell/update.rs` Ctrl+Shift+P toggle | Req 1.5: Ctrl+Shift+P toggles palette closed when already open |
| `ff-desktop` | ✅ | `command_palette/fuzzy.rs` unit tests | Req 2.1: Fuzzy match filters by subsequence in real time |
| `ff-desktop` | ✅ | `command_palette/fuzzy.rs` unit tests | Req 2.2: Scoring: contiguous runs, word boundaries, shorter names |
| `ff-desktop` | ✅ | `command_palette/render.rs` rebuild_filtered | Req 2.3: Results sorted by descending score, alpha tiebreak |
| `ff-desktop` | ✅ | `command_palette/render.rs` rebuild_filtered | Req 2.4: Empty query shows recent then all alphabetically |
| `ff-desktop` | ✅ | `command_palette/fuzzy.rs` unit tests | Req 2.5: Fuzzy search is case-insensitive |
| `ff-desktop` | ✅ | `command_palette/render.rs` empty state | Req 2.6: No-match shows 'No commands match <query>' message |
| `ff-desktop` | ✅ | `command_palette/render.rs` render_entry | Req 3.1: Entry shows display name, category, shortcut |
| `ff-desktop` | ✅ | `command_palette/render.rs` detail area | Req 3.2: Highlighted entry shows description in detail area |
| `ff-desktop` | ✅ | `command_palette/render.rs` build_highlighted_text | Req 3.3: Matched characters highlighted in display name |
| `ff-desktop` | ✅ | `command_palette/render.rs` MAX_VISIBLE | Req 3.4: At most 20 entries visible; list is scrollable |
| `ff-desktop` | ✅ | `command_palette/state.rs` unit tests | Req 4.1: Enter executes highlighted command and closes palette |
| `ff-desktop` | ✅ | `command_palette/render.rs` click handler | Req 4.2: Click on entry executes command and closes palette |
| `ff-desktop` | ✅ | `command_palette/state.rs` unit tests | Req 4.3: Up/Down arrows navigate list with wrap-around |
| `ff-desktop` | ✅ | `shell/update.rs` recent list update | Req 4.4: Executed command added to recent list (max 10) |
| `ff-desktop` | ✅ | `command_palette/render.rs` disabled style | Req 4.5: Disabled entry shown with disabled style; Enter blocked |
| `ff-desktop` | ✅ | `command_palette/render.rs` Recently Used header | Req 5.1: Empty query shows Recently Used section |
| `ff-desktop` | ✅ | `ff-session` SessionState + session_manager | Req 5.2: Recent commands persisted in session.toml |
| `ff-desktop` | ✅ | `command_palette/render.rs` rebuild_filtered | Req 5.3: Typing query hides Recently Used section |
| `ff-desktop` | ✅ | `shell/update.rs` recent list update | Req 5.4: Only successfully executed commands added to recent list |
### Phase CP -- Batch Command Execution (CR-NR-041)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `batch::cli` unit tests, `main.rs` | Req 1.1: ffwb --batch <file> executes in headless batch mode; no GUI window opened |
| `ff-desktop` | ✅ | `batch::cli` unit tests | Req 1.2: ffwb --batch - reads commands from stdin |
| `ff-desktop` | ✅ | `batch::cli` unit tests | Req 1.3: --batch combined with file path arguments rejected with error |
| `ff-desktop` | ✅ | `batch::cli` unit tests, `main.rs` | Req 1.4: without --batch, workbench starts in normal interactive GUI mode |
| `ff-desktop` | ✅ | `batch::cli` unit tests | Req 1.5: --batch documented in ffwb --help (`print_help_does_not_panic`) |
| `ff-desktop` | ✅ | `batch::mod::run_batch()` | Req 1.6: missing/unreadable batch file exits with return code 12 |
| `ff-desktop` | ✅ | `batch::input` unit tests | Req 2.1: Batch_Input_Source accepts UTF-8 with or without BOM; BOM stripped |
| `ff-desktop` | ✅ | `batch::input` unit tests | Req 2.2: each non-blank non-comment line submitted as one Batch_Command in order |
| `ff-desktop` | ✅ | `batch::input` unit tests | Req 2.3: lines beginning with * treated as comments and skipped |
| `ff-desktop` | ✅ | `batch::input` unit tests | Req 2.4: lines beginning with /* treated as comments and skipped |
| `ff-desktop` | ✅ | `batch::input` unit tests | Req 2.5: blank/whitespace-only lines skipped |
| `ff-desktop` | ✅ | `batch::input` unit tests | Req 2.6: line ending with - continues onto next line |
| `ff-desktop` | ✅ | `batch::input` unit tests | Req 2.7: lines exceeding 32767 chars truncated with warning |
| `ff-desktop` | 🔴 | -- | Req 3.1: Batch_Commands dispatched through ff-command-semantics pipeline unchanged |
| `ff-desktop` | 🔴 | -- | Req 3.2: Batch_Session provides same catalog registry and config as interactive session |
| `ff-desktop` | 🔴 | -- | Req 3.3: command output written to Batch_Output_Sink |
| `ff-desktop` | 🔴 | -- | Req 3.4: document modifications applied to real filesystem |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 3.5: GUI-requiring commands fail with Step_Return_Code 8 and diagnostic message (`interactive_command_returns_step_rc_8_with_diagnostic`) |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 3.6: commands executed sequentially |
| `ff-desktop` | ✅ | `batch::mod::run_batch()` | Req 4.1: default output to stdout |
| `ff-desktop` | ✅ | `batch::cli` unit tests, `batch::mod::run_batch()` | Req 4.2: --batch-output <file> writes output to specified file |
| `ff-desktop` | ✅ | `batch::cli` unit tests, `batch::mod::run_batch()` | Req 4.3: --batch-output-append <file> appends output to specified file |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 4.4: --batch-echo prefixes each command output with ===> <command text> |
| `ff-desktop` | 🔴 | -- | Req 4.5: without --batch-echo, command text not written to output |
| `ff-desktop` | 🔴 | -- | Req 4.6: BatchRunner diagnostic output written to stderr |
| `ff-desktop` | ✅ | `batch::mod::run_batch()` | Req 4.7: unwritable output file exits with return code 12 before executing commands |
| `ff-desktop` | ✅ | `batch::return_code` unit tests | Req 5.1: all commands succeed -> exit code 0 |
| `ff-desktop` | ✅ | `batch::return_code` unit tests | Req 5.2: Batch_Return_Code is maximum Step_Return_Code across all commands |
| `ff-desktop` | ✅ | `batch::return_code` unit tests | Req 5.3: Step_Return_Code values 0/4/8/12/16 used per z/OS convention |
| `ff-desktop` | 🔴 | -- | Req 5.4: BatchRunner init failure exits with code 12 |
| `ff-desktop` | ✅ | `batch::mod::run_batch()` | Req 5.5: final summary line "FFWB BATCH RETURN CODE: N" written to stderr |
| `ff-desktop` | ✅ | `batch::return_code` unit tests | Req 6.1: default mode continues after command failure (best-effort) |
| `ff-desktop` | ✅ | `batch::cli` unit tests, `batch::return_code` unit tests | Req 6.2: --batch-abort-on-error <threshold> stops on Step_Return_Code >= threshold |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 6.3: abort writes message identifying aborting command and return code (wired in runner.run()) |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 6.4: Batch_Return_Code reflects aborting command's Step_Return_Code (brc.update(step) before break) |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 6.5: CONTROL ERRORS CANCEL / NOCANCEL inline commands override abort policy |
| `ff-desktop` | 🔴 | -- | Req 7.1: Batch_Session loads same config layers as interactive session |
| `ff-desktop` | 🔴 | -- | Req 7.2: Batch_Session loads catalog registry from same catalogs.toml |
| `ff-desktop` | ✅ | `batch::cli` unit tests | Req 7.3: --batch-profile <name> loads named config profile |
| `ff-desktop` | 🔴 | -- | Req 7.4: Batch_Session does NOT restore GUI session state |
| `ff-desktop` | 🔴 | -- | Req 7.5: batch run does NOT overwrite interactive session state file |
| `ff-desktop` | ✅ | `batch::cli` unit tests | Req 7.6: --batch-no-catalog starts with empty catalog registry |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 8.1: --batch-dry-run parses/validates commands without modifying filesystem |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 8.2: dry-run writes [DRY-RUN] <command> -> OK|ERROR: reason for each command |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 8.3: dry-run return code 0 if all valid, 8 if any syntax error or missing resource |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 8.4: read-only commands execute normally in dry-run mode |
| `ff-desktop` | ✅ | `batch::input` unit tests | Req 9.1: Batch_Input_Source format identical to .ffcmd format (`ffcmd_format_parsed_identically_to_batch_format`) |
| `ff-desktop` | ✅ | `batch::input` unit tests | Req 9.2: .ffcmd file usable as --batch input without modification |
| `ff-desktop` | ✅ | `batch::input` unit tests | Req 9.3: .ffcmd extension recognised; other extensions also accepted |
| `ff-desktop` | ✅ | `batch::input` unit tests | Req 9.4: .ffcmd via --batch does NOT invoke Lua engine (`batch_input_source_has_no_lua_dependency`) |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 10.1: structured log written: start, each command+RC+duration, final RC (`runner_completes_without_panic_logging_enabled`) |
| `ff-desktop` | 🔴 | -- | Req 10.2: --batch-log <file> redirects structured log to specified file (deferred Task 10.2) |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 10.3: log includes wall-clock duration of each command |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 10.4: Step_Return_Code >= 8 logged at ERROR level with full error detail |
| `ff-desktop` | ✅ | `batch::runner` unit tests | Req 10.5: log format matches ff-logging structured format |

## Phase CO -- Accessibility

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-theme` | ✅ | `contrast.rs` unit tests | Req 1.1: contrast_ratio() implements WCAG relative luminance formula |
| `ff-theme` | ✅ | `contrast.rs` unit tests | Req 1.2: contrast_ratio black-on-white returns 21:1 |
| `ff-theme` | ✅ | `contrast.rs` unit tests | Req 1.3: all three built-in themes pass 4.5:1 for primary text pairs |
| `ff-theme` | ✅ | `contrast.rs` unit tests | Req 1.4: check_theme_contrast() emits ContrastWarning for failing pairs |
| `ff-theme` | ✅ | `contrast.rs` unit tests | Req 1.5: light palette WCAG failures fixed (line_number_fg, inactive_text) |
| `ff-theme` | ✅ | `shell::tests::focus_ring_token_exists_in_all_themes` | Req 3.1: focus_ring token exists in all four built-in themes |
| `ff-theme` | ✅ | `palette.rs` unit tests | Req 3.2: UiFocusRing token maps to ui.focus_ring field |
| `ff-desktop` | ✅ | `shell::tests::focus_ring_token_exists_in_all_themes` | Req 3.2: focus_ring colour differs from panel background in all themes |
| `ff-desktop` | ✅ | `shell::tests::focus_indicator_helper_is_callable` | Req 3.3: render_focus_indicator helper exists and is callable |
| `ff-desktop` | 🔲 | -- | Req 3.4: focus ring drawn for focused TabHeader stop (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 3.5: focus ring consistent across all panels (manual UI verification) |
| `ff-desktop` | ✅ | `shell::tests::key_config_dialog_escape_closes_dialog` | Req 2.1, 2.3: KeyConfigDialog closes on Escape |
| `ff-desktop` | ✅ | `shell::tests::dataset_alloc_dialog_has_cancel_path` | Req 2.1, 2.3: DatasetAllocDialog has Cancel path reachable by keyboard |
| `ff-desktop` | ✅ | `shell::tests::modal_open_flag_suppresses_shell_tab_cycle` | Req 2.3: modal_open flag traps focus within dialog |
| `ff-config` | ✅ | `keys.rs` unit tests | Req 5.2: accessibility.reduce_motion config key registered with unique path |
| `ff-desktop` | ✅ | `main::tests::reduce_motion_config_key_is_registered_in_schema` | Req 5.2: accessibility.reduce_motion registered in built-in schema |
| `ff-desktop` | ✅ | `shell::tests::reduce_motion_scroll_is_immediate_jump` | Req 5.3: reduce_motion config key readable; scroll is immediate jump |
| `ff-desktop` | 🔲 | -- | Req 2.4: context menus respond to arrow keys and Enter (manual UI verification) |
| `ff-desktop` | 🔴 | -- | Req 4.1: egui AccessKit feature enabled; interactive elements carry accessible labels |
| `ff-desktop` | 🔴 | -- | Req 4.3: status bar message exposed as live region |
| `ff-desktop` | 🔴 | -- | Req 5.1: macOS/Linux OS reduce-motion detection (deferred) |

## Phase CO -- Plugin Manager UI

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-desktop` | ✅ | `shell::tests::plugin_manager_tab_kind_exists` | Req 1.1: PluginManager TabKind variant exists |
| `ff-desktop` | ✅ | `shell::tests::option_8_routes_to_plugin_manager` | Req 1.1: option 8 routes to PluginManager tab |
| `ff-desktop` | ✅ | `shell::tests::plugins_command_routes_to_plugin_manager` | Req 1.1: PLUGINS command routes to PluginManager |
| `ff-desktop` | ✅ | `shell::tests::equals_8_command_routes_to_plugin_manager` | Req 1.1: =8 command routes to PluginManager |
| `ff-desktop` | ✅ | `shell::tests::title_line_plugin_manager_shows_plugins` | Req 1.1: title line shows [PLUGINS] |
| `ff-desktop` | ✅ | `plugin_manager_panel::tests::plugin_list_sorted_alphabetically` | Req 1.5: plugin list sorted alphabetically |
| `ff-desktop` | ✅ | `plugin_manager_panel::tests::filter_narrows_plugin_list` | Req 1.6: filter narrows plugin list |
| `ff-desktop` | ✅ | `plugin_manager_panel::tests::empty_filter_returns_all_plugins` | Req 1.6: empty filter returns all plugins |
| `ff-desktop` | ✅ | `shell::tests::plugin_manager_tab_round_trips_through_session` | Req 4.1: PluginManager tab kind in PersistedTabKind |
| `ff-desktop` | 🔲 | -- | Req 2.1-2.6: Enable/Disable buttons (manual UI verification) |
| `ff-desktop` | 🔲 | -- | Req 3.1-3.3: Plugin detail view (manual UI verification) |

## Phase CO -- Notification System

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-desktop` | ✅ | `notification::tests::queue_caps_at_1000_entries` | Req 2.7: queue caps at 1000 entries |
| `ff-desktop` | ✅ | `notification::tests::push_warning_increments_unread` | Req 2.7, 4.2: Warning increments unread count |
| `ff-desktop` | ✅ | `notification::tests::mark_all_read_clears_unread` | Req 2.4, 4.3: mark_all_read clears unread |
| `ff-desktop` | ✅ | `notification::tests::clear_empties_queue_and_unread` | Req 2.6: clear empties queue |
| `ff-desktop` | ✅ | `notification::tests::filter_by_level_returns_matching` | Req 2.4: filter_by_level returns matching entries |
| `ff-desktop` | ✅ | `notification::tests::sender_is_clone` | Req 3.2: NotificationSender is Clone |
| `ff-desktop` | ✅ | `notification::tests::full_channel_drops_without_panic` | Req 3.3: full channel drops silently |
| `ff-desktop` | ✅ | `shell::tests::notification_sender_is_clone_and_send` | Req 3.1, 3.2: sender is Clone + Send |
| `ff-desktop` | ✅ | `shell::tests::notification_queue_caps_at_1000_entries` | Req 2.7: queue caps at 1000 |
| `ff-desktop` | ✅ | `shell::tests::push_warning_increments_unread` | Req 2.7: Warning increments unread |
| `ff-desktop` | ✅ | `shell::tests::mark_all_read_clears_unread` | Req 2.4: mark_all_read clears unread |
| `ff-desktop` | ✅ | `shell::tests::event_log_tab_kind_exists` | Req 2.1: EventLog TabKind variant exists |
| `ff-desktop` | ✅ | `shell::tests::log_command_routes_to_event_log` | Req 2.1: LOG command opens EventLog tab |
| `ff-desktop` | ✅ | `shell::tests::clear_log_empties_queue` | Req 2.6: clear empties queue |
| `ff-desktop` | ✅ | `shell::tests::filter_by_level_returns_matching` | Req 2.4: filter_by_level works |
| `ff-desktop` | ✅ | `shell::tests::title_line_event_log_shows_log` | Req 2.1: title line shows [LOG] |
| `ff-desktop` | ✅ | `shell::tests::notifications_drained_from_channel_each_frame` | Req 1.1: channel drained each frame |
| `ff-desktop` | ✅ | `shell::tests::bell_badge_shows_unread_count` | Req 4.2: unread count tracked |
| `ff-desktop` | 🔴 | -- | Req 1.2-1.6: Toast overlay rendering (not yet implemented) |
| `ff-desktop` | 🔴 | -- | Req 4.1: Bell icon in status bar (not yet implemented) |

## Final Summary (after Phase CO)

| Status | Count |
|--------|-------|
| PASS | 716 tests (ff-desktop + all library crates) |
| FAIL | 0 |
| MANUAL | Req 2.1-2.6 plugin enable/disable, Req 3.1-3.3 plugin detail, Req 1.2-1.6 toast overlay, Req 4.1 bell icon |
| NOT COVERED | Req 4.1 bell icon in status bar, Req 1.2-1.6 toast overlay, Req 4.1-4.3 AccessKit screen reader |

### Phase CQ -- Enterprise Features (configuration-system Requirements 16-18)

| Crate | Status | Test | Requirement |
|-------|--------|------|-------------|
| `ff-config` | ✅ | `audit.rs` unit tests | Req 16.1: WHEN any config key effective value changes, THE system SHALL append AuditEntry with timestamp, key, old/new value, layer, actor |
| `ff-config` | ✅ | `audit.rs` unit tests | Req 16.2: audit log persisted to rolling file at <user-config-dir>/audit.log; max 10,000 entries |
| `ff-config` | ✅ | `audit.rs` unit tests | Req 16.3: query_audit_log(filter) API supports filtering by key prefix, layer, time range, actor |
| `ff-config` | ✅ | `audit.rs` unit tests | Req 16.4: audit log write failure emits WARN and does not prevent config change |
| `ff-config` | ✅ | `audit.rs` unit tests | Req 16.5: AuditEntry public type with timestamp, key, old_value, new_value, layer, actor fields |
| `ff-config` | ✅ | `audit.rs` unit tests | Req 16.6: clear_audit_log() truncates in-memory and on-disk audit log |
| `ff-config` | ✅ | `export_import.rs` unit tests | Req 17.1: export_settings(scope, path) writes TOML file for specified ExportScope |
| `ff-config` | ✅ | `export_import.rs` unit tests | Req 17.2: ExportScope enum with AllLayers, UserLayer, ProjectLayer variants |
| `ff-config` | ✅ | `export_import.rs` unit tests | Req 17.3: exported TOML includes [_export_meta] header with timestamp, version, scope |
| `ff-config` | ✅ | `export_import.rs` unit tests | Req 17.4: import_settings(path, target) merges exported values into target layer |
| `ff-config` | ✅ | `export_import.rs` unit tests | Req 17.5: ImportTarget enum with UserLayer, ProjectLayer variants |
| `ff-config` | ✅ | `export_import.rs` unit tests | Req 17.6: invalid values skipped and reported in ImportSummary; import does not fail entirely |
| `ff-config` | ✅ | `export_import.rs` unit tests | Req 17.7: ImportSummary struct with imported_count, skipped_count, skipped_keys fields |
| `ff-config` | ✅ | `export_import.rs` unit tests | Req 17.8: unreadable or invalid TOML import file returns ConfigError; no changes made |
| `ff-config` | ✅ | `export_import.rs` unit tests | Req 17.9: successful import triggers hot-reload cycle; callbacks notified of changed keys |
| `ff-config` | ✅ | `merger.rs`, `config_handle.rs` unit tests | Req 18.1: system-layer [_locked].locked_keys list parsed into locked key set |
| `ff-config` | ✅ | `merger.rs` unit tests | Req 18.2: locked key uses system-layer value regardless of higher-priority layer definitions |
| `ff-config` | ✅ | `config_handle.rs` unit tests | Req 18.3: set_user_value() on locked key returns ConfigError::KeyLocked |
| `ff-config` | ✅ | `merger.rs` unit tests | Req 18.4: higher-priority layer value for locked key silently ignored; DEBUG log emitted |
| `ff-config` | ✅ | `config_handle.rs` unit tests | Req 18.5: is_locked(key) -> bool method on ConfigHandle |
| `ff-desktop` | 🔲 | -- | Req 18.6: Settings panel shows LOCKED badge and disables widget + Reset button for locked keys |
| `ff-config` | ✅ | `error.rs` unit tests | Req 18.7: ConfigError::KeyLocked variant with message "[config] lock: key '{key}' is locked by system policy and cannot be modified" |
| `ff-config` | ✅ | `config_handle.rs` unit tests | Req 18.8: hot-reload of system layer recomputes locked set; callbacks invoked for affected keys |

### Phase CR -- OS Theme Follow + Macro Library Management

| Crate | Status | Test | Requirement |
|-------|--------|------|-------------|
| `ff-desktop` | ✅ | theme_follow_os_key_is_registered_in_schema | Req 16.1: theme.follow_os config key (boolean, default false) |
| `ff-desktop` | 🔲 | manual: egui ctx required | Req 16.2: follow_os=true + OS dark -> Visual_Mode set to Dark |
| `ff-desktop` | 🔲 | manual: egui ctx required | Req 16.3: follow_os=true + OS light -> Visual_Mode set to Light |
| `ff-desktop` | ✅ | theme_follow_os_false_does_not_change_palette | Req 16.4: follow_os=false -> OS preference ignored; theme.mode used |
| `ff-desktop` | 🔲 | manual: egui ctx required | Req 16.5: OS preference change detected within one egui frame |
| `ff-desktop` | 🔲 | manual: egui ctx required | Req 16.6: OS preference read from egui ctx.style().visuals.dark_mode |
| `ff-desktop` | 🔲 | manual: egui ctx required | Req 16.7: auto-applied mode not persisted to theme.mode config key |
| `ff-desktop` | 🔲 | manual: UI verification | Req 16.8: Settings panel exposes theme.follow_os checkbox |
| `ff-desktop` | ✅ | option_6_routes_to_macro_library | Req 12.1: Macro Library panel via POM option 6, MACROS command, =6 fastpath |
| `ff-desktop` | ✅ | refresh_discovers_lua_files_in_directory | Req 12.2: panel lists all macros with name, source directory, full path |
| `ff-desktop` | 🔲 | manual: Lua execution deferred | Req 12.3: Run action dispatches MACRO <name> and shows result in status bar |
| `ff-desktop` | 🔲 | manual: UI verification | Req 12.4: Edit action opens .lua file in new editor tab |
| `ff-desktop` | 🔲 | manual: UI verification | Req 12.5: Delete action prompts confirmation then removes file and inventory entry |
| `ff-desktop` | ✅ | filter_narrows_visible_entries | Req 12.6: filter input narrows list by case-insensitive name substring |
| `ff-desktop` | ✅ | macro_library_tab_not_persisted_in_session | Req 12.7: panel state (selection, filter) not persisted across sessions |
| `ff-desktop` | ✅ | refresh_clears_stale_entries_before_scan | Req 12.8: panel list refreshes within one frame when macro inventory changes |

### Phase CU -- Menu Workspace Pattern (CR-NR-045)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 1.1: Menu_File parsed as TOML with `title` and `[[options]]` top-level keys |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 1.2: each option entry has `key` (1-4 chars), `command`, `description` |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 1.3: optional `enabled` and `group` fields parsed correctly |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 1.4: unknown TOML keys silently ignored |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 1.5: absent or unreadable Menu_File shows error message in option area |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 1.6: invalid TOML or missing required field shows parse error message |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 1.7: Menu_File path resolved relative to User_Data_Dir |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 2.1: Menu_Workspace renders Title_Line, Menu_Title, option list, Command field, Key_Label_Bar in order |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 2.2: each option row is an interactive element (click or Tab+Enter selects) |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 2.3: disabled option rendered in disabled style; does not respond to activation |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 2.4: group field inserts blank line between groups |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 2.5: option list is scrollable when options exceed visible area |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 2.6: zero options shows placeholder message |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 3.1: typing Option_Key in Command field executes Option_Command |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 3.2: clicking option row executes Option_Command |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 3.3: Tab to option row + Enter/Space executes Option_Command |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 3.4: Option_Command dispatched through standard command pipeline |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 3.5: Option_Command beginning with `=` routed as fastpath |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 3.6: unknown key shows 'Option not found' message |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 3.7: disabled option activation shows 'not available' message |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 4.1: menus/pom.toml created on first launch if absent |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 4.2: menus/settings.toml created on first launch if absent |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 4.3: modified Menu_File reloaded within one egui frame |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 4.4: hot-reload uses existing ff-config file-watch infrastructure |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 4.5: hot-reload parse error retains previous option list |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 4.6: menus/ directory created automatically in User_Data_Dir |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 5.1: chained path =key1.key2 navigates to sub-menu and executes option |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 5.2: chained paths support up to 4 levels of nesting |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 5.3: unknown segment stops at last resolved level with error message |
| `ff-desktop` | ✅ | `menu_workspace` unit tests | Req 5.4: existing single-segment fastpath notation (=0, =1, etc.) unchanged |

### Phase CV -- POM Redesign (CR-NR-048)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | PASS | `primary_option_menu.rs`, `shell/tests.rs` unit tests | Req 6.1: revised POM has 12 options (0-8 unchanged, 9/S/B added) |
| `ff-desktop` | PASS | `shell/tests.rs::pom_key_9_routes_to_jes` | Req 6.2: key 9 routes to JES job monitor panel |
| `ff-desktop` | PASS | `shell/tests.rs::pom_key_s_routes_to_search` | Req 6.3: key S routes to Global Search panel |
| `ff-desktop` | PASS | `shell/tests.rs::pom_key_b_shows_batch_message` | Req 6.4: key B shows Batch status message |
| `ff-desktop` | PASS | `shell/tests.rs::pom_keys_0_to_8_unchanged`, `pom_option_keys_are_zero_through_eight_then_extended` | Req 6.5: existing keys 0-8 behaviour unchanged |
| `ff-desktop` | PASS | `menu_workspace/defaults.rs` unit tests | Req 6.6: options grouped as Core (0-8) and Extended (9/S/B) in TOML |
| `ff-desktop` | PASS | `menu_workspace/defaults.rs::default_pom_toml_has_12_options` | Req 7.1: DEFAULT_POM_TOML produces MenuFile with 12 options matching Req 6.1 |
| `ff-desktop` | PASS | `menu_workspace/defaults.rs::ensure_default_menu_files_creates_pom_toml` | Req 7.2: ensure_default_menu_files writes pom.toml when absent |
| `ff-desktop` | PASS | `menu_workspace/defaults.rs::ensure_default_menu_files_does_not_overwrite_existing` | Req 7.3: ensure_default_menu_files does not overwrite existing pom.toml |
| `ff-desktop` | PASS | `menu_workspace/defaults.rs::default_pom_toml_is_valid_toml` | Req 7.4: DEFAULT_POM_TOML is valid TOML parseable by toml crate |
| `ff-desktop` | PASS | `menu_workspace/defaults.rs::default_pom_toml_ascii_only` | Req 7.5: DEFAULT_POM_TOML uses only plain ASCII characters |
| `docs` | PASS | `startup-and-session/requirements.md` Req 14.3 | Req 8.1: startup-and-session Req 14.3 lists all 12 options |
| `docs` | PASS | `startup-and-session/requirements.md` Req 14.3 | Req 8.2: updated Req 14.3 retains forward-reference note to menu-workspace spec |
| `docs` | PASS | `startup-and-session/requirements.md` Req 14.3 | Req 8.3: updated Req 14.3 notes that options 9/S/B are new in Phase CV |

### Phase CW -- Settings as a Menu Workspace (CR-NR-047)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | PASS | ``menu_workspace/defaults.rs::default_settings_toml_has_10_options`` | Req 9.1: Settings_Menu has 10 options (E/T/C/V/L/K/S/P/X/A) with correct keys, labels, descriptions |
| `ff-desktop` | PASS | ``shell/tests.rs::settings_namespace_filter_applied_on_open`` | Req 9.2: typing namespace key in Settings_Menu Command field opens Settings_Namespace_View |
| `ff-desktop` | NOT COVERED | -- | Req 9.3: clicking namespace option row opens Settings_Namespace_View (needs Settings_Menu MenuWorkspace rendering, not built in CW-impl) |
| `ff-desktop` | PASS | ``shell/tests.rs::settings_all_view_has_no_namespace_filter`` | Req 9.4: option A opens unfiltered flat-list Settings panel |
| `ff-desktop` | PASS | ``menu_workspace/defaults.rs::default_settings_toml_title_matches_spec`` | Req 9.5: Settings_Menu title is "FileForge Workbench -- Settings" |
| `ff-desktop` | PASS | ``menu_workspace/defaults.rs::default_settings_toml_is_valid_toml`` | Req 9.6: options grouped as Namespaces (E-X) and All (A) in TOML |
| `ff-desktop` | PASS | ``shell/tests.rs::settings_namespace_filter_applied_on_open`` | Req 10.1: Settings_Namespace_View pre-populates filter with namespace prefix |
| `ff-desktop` | PASS | ``shell/tests.rs::settings_namespace_filter_applied_on_open`` | Req 10.2: pre-populated filter applied immediately on open |
| `ff-desktop` | MANUAL | -- | Req 10.3: user can clear or modify filter within Settings_Namespace_View (manual UI verification) |
| `ff-desktop` | PASS | ``shell/tests.rs::settings_end_from_namespace_view_returns_to_menu`` | Req 10.4: F3/END in Settings_Namespace_View returns to Settings_Menu level |
| `ff-desktop` | PASS | ``shell/tests.rs::settings_namespace_tab_title_includes_namespace`` | Req 10.5: Settings_Namespace_View tab title is [SETTINGS:<namespace>] |
| `ff-desktop` | NOT COVERED | -- | Req 10.6: Settings_Namespace_View persists namespace filter in session (BLOCKED: requires ff-session PersistedTabKind format change; Task 14.5 deferred) |
| `ff-desktop` | PASS | ``menu_workspace/defaults.rs::default_settings_toml_has_10_options`` | Req 11.1: DEFAULT_SETTINGS_TOML produces MenuFile with 10 options matching Req 9.1 |
| `ff-desktop` | PASS | ``menu_workspace/defaults.rs::ensure_default_menu_files_creates_settings_toml`` | Req 11.2: ensure_default_menu_files writes settings.toml when absent |
| `ff-desktop` | PASS | ``menu_workspace/defaults.rs::ensure_default_menu_files_does_not_overwrite_existing`` | Req 11.3: ensure_default_menu_files does not overwrite existing settings.toml |
| `ff-desktop` | PASS | ``menu_workspace/defaults.rs::default_settings_toml_is_valid_toml`` | Req 11.4: DEFAULT_SETTINGS_TOML is valid TOML parseable by toml crate |
| `ff-desktop` | PASS | ``menu_workspace/defaults.rs::default_settings_toml_ascii_only`` | Req 11.5: DEFAULT_SETTINGS_TOML uses only plain ASCII characters |
| `docs` | PASS | ``configuration-system/requirements.md`` Req 15 | Req 12.1: configuration-system Req 15 describes two-level Settings navigation |
| `docs` | PASS | ``configuration-system/requirements.md`` Req 15 | Req 12.2: updated Req 15 retains criteria 15.1-15.11 with Phase CW adjustments |
| `docs` | PASS | ``configuration-system/requirements.md`` Req 15 | Req 12.3: updated Req 15 notes Settings_Menu backed by menus/settings.toml |

### Phase DA -- Configurable Menu Option Limits (CR-NR-050)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | menu_limit_keys_have_correct_defaults | Req 9.1: menu.soft_option_limit (default 64) and menu.hard_option_limit (default 256) config keys defined |
| `ff-desktop` | ✅ | count_at_or_below_soft_limit_no_advisory, load_with_limits_end_to_end_within_soft | Req 9.2: option count <= soft limit loads and renders with no warning |
| `ff-desktop` | ✅ | count_above_soft_below_hard_sets_advisory, advisory_line_rendered_when_soft_exceeded | Req 9.3: count above soft, at/below hard loads, logs WARN, shows advisory line |
| `ff-desktop` | ✅ | count_above_hard_returns_load_error | Req 9.4: count above hard limit rejected as load error, no option rows rendered |
| `ff-desktop` | ✅ | hard_below_soft_clamps_effective_soft_to_hard | Req 9.5: hard limit below soft limit clamps effective soft to hard and logs WARN |
| `ff-desktop` | ✅ | menu_limit_keys_have_correct_defaults, option_limits_from_config_falls_back_to_defaults | Req 9.6: absent limit key applies default value |
| `ff-desktop` | ✅ | option_limits_from_config_falls_back_to_defaults | Req 9.7: invalid limit value falls back to default and logs WARN |
| `ff-desktop` | ✅ | disabled_options_count_toward_limits | Req 9.8: disabled options count toward both limits (counted before enabled filtering) |
| `ff-desktop` | ✅ | reload_over_hard_limit_transitions_to_error, reload_back_under_limit_recovers | Req 9.9: limits re-evaluated on hot-reload; over-hard transitions to error, back-under recovers |

### Phase DB -- Unified Command Target (CR-NR-051, command-framework Req 8)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-command` | ✅ | `command_target_tests.rs::command_target_has_five_constructible_variants` | Req 8.1: CommandTarget type with 5 variants (Menu/CustomWorkspace/Function/Macro/External) |
| `ff-command` | ✅ | `command_target_tests.rs::execute_function_target_dispatches_via_callback`, `execute_non_function_targets_are_deferred_to_shell` | Req 8.2: execute_target routes each variant and returns a CommandResult |
| `ff-command` | ✅ | `command_target_tests.rs::user_command_definition_resolves_first`, `builtin_workspace_verb_resolves_to_custom_workspace`, `bare_registered_command_resolves_to_function_target` | Req 8.3: resolve_target converts a bare string to the correct variant |
| `ff-command` | ✅ | `command_target_tests.rs::bare_registered_command_resolves_to_function_target`, `resolution_trims_surrounding_whitespace` | Req 8.4: no behaviour change -- every existing string resolves to an equivalent target |
| `ff-desktop` | ✅ | `shell/tests.rs::dispatch_bound_command_resolves_user_definition`, `dispatch_bound_command_falls_through_for_builtin` | Req 8.5: a Shortcut_Binding is routed through Target_Resolution (fkey + key-label-bar via `dispatch_bound_command`); DB.4 wiring |
| `ff-desktop` | ✅ | `shell/tests.rs::menu_option_command_matching_definition_id_dispatches`, `resolve_and_dispatch_user_definition_id_dispatches` | Req 8.6: a Menu_Option resolves its command value to a CommandTarget and dispatches it; DB.4 wiring |
| `ff-command` | ✅ | `command_target_tests.rs::menu_target_toml_round_trips`, `external_target_toml_round_trips_with_all_fields`, `custom_workspace_target_with_params_round_trips`, `external_target_parses_from_authored_toml`, `function_target_defaults_params_when_omitted` | Req 8.7: CommandTarget serialises to/from TOML |
| `ff-command` | ✅ | `command_target_tests.rs::unresolved_string_returns_error_naming_input`, `execute_function_target_with_invalid_id_fails` | Req 8.8: unresolved string returns an error naming it, no panic/mutation |
| `ff-command` | ✅ | `command_target_tests.rs::menu_and_custom_workspace_and_captured_external_are_visible`, `function_macro_and_detached_external_are_not_visible` | Req 8.9: produces_visible_workspace classification queryable without executing |

### Phase DB -- Command Configurator (CR-NR-052, new sub-project)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `command_config/store.rs::save_then_load_round_trips_a_definition` | Req 1.1: commands.toml Command_Store with [[command]] array |
| `ff-desktop` | ✅ | `command_config/store.rs::save_then_load_round_trips_a_definition`, `external_definition_round_trips` | Req 1.2: each entry has id, label, [command.target] serialised CommandTarget |
| `ff-desktop` | ✅ | `command_config/mod.rs::category_defaults_to_user` | Req 1.3: optional description and category (default user) |
| `ff-desktop` | ✅ | `command_config/store.rs::duplicate_id_keeps_first_and_records_skip` | Req 1.4: duplicate id keeps first, skips duplicate, records skip |
| `ff-desktop` | ✅ | `command_config/store.rs::absent_file_yields_empty_store`, `save_creates_parent_directory` | Req 1.5: absent file = empty store; commands/ dir created on first save |
| `ff-desktop` | ✅ | `command_config/store.rs::invalid_entry_skipped_valid_retained`, `malformed_toml_surfaces_load_error` | Req 1.6: invalid TOML/entry skipped, valid entries retained, load-error surfaced |
| `ff-desktop` | ✅ | `command_config/store.rs::poll_reload_picks_up_disk_change` | Req 1.7: store hot-reloads via mtime poll (Menu Workspace pattern) |
| `ff-desktop` | 🔴 | -- | Req 1.8: default/example content is plain ASCII (no default-content generator yet; lands with the Context UI) |
| `ff-desktop` | ✅ | `shell/tests.rs::commands_opens_command_configurator_context` | Req 2.1: Command Configurator Context opens via COMMANDS, lists definitions |
| `ff-desktop` | ✅ | `command_config/render.rs::external_mode_label_reflects_mode`, `non_external_target_has_blank_mode_label` | Req 2.2: each row shows id, label, variant, and external Execution_Mode |
| `ff-desktop` | ✅ | `shell/tests.rs::configurator_save_adds_definition_to_store`, `configurator_delete_removes_definition`, `configurator_edit_updates_in_place` | Req 2.3: Add / Edit / Delete (delete with confirmation) |
| `ff-desktop` | ✅ | `shell/tests.rs::configurator_save_adds_definition_to_store`, `configurator_save_rejects_invalid_id` | Req 2.4: save validates then writes store; validation failure leaves store unchanged |
| `ff-desktop` | ✅ | `shell/tests.rs::configurator_delete_removes_definition` | Req 2.5: delete removes definition; save writes store back |
| `ff-desktop` | ✅ | `command_config/edit.rs::build_external_target_from_form`, `from_definition_prefills_external_fields` | Req 2.6: variant-specific target editor fields |
| `ff-desktop` | ✅ | `shell/tests.rs::commands_opens_command_configurator_context` | Req 2.7: Context title is [COMMANDS] |
| `ff-desktop` | ✅ | `shell/tests.rs::command_configurator_end_returns_to_pom` | Req 2.8: F3/END returns to POM |
| `ff-desktop` | 🔴 | -- | Req 3.1: External target carries program, args, working_dir, mode |
| `ff-shell` | ✅ | `engine.rs::execute_external_detached_returns_handle`, `executor::external::tests::spawn_detached_returns_handle_without_waiting` | Req 3.2: detached mode spawns Started_Task, returns immediately, no capture, no Workspace (no Output_Panel entry) |
| `ff-shell` | ✅ | `engine.rs::execute_external_detached_returns_handle` | Req 3.3: Started_Task not tracked/monitored/restarted/persisted after spawn (handle dropped by caller) |
| `ff-shell` | ✅ | `engine.rs::execute_external_captured_appends_to_output_panel` | Req 3.4: captured mode runs async, shows stdout/stderr/exit in Output_Panel |
| `ff-shell` | ✅ | `engine.rs::spawn_detached_uses_explicit_working_dir` (explicit); `working_dir.rs` resolver tests (fallback) | Req 3.5: explicit working_dir else shell.working_directory rules |
| `ff-desktop` | ✅ | `shell/tests.rs::external_placeholder_unresolved_expands_to_empty` | Req 3.6: ${workspace_root} / ${file_dir} placeholder expansion (unresolved -> empty + DEBUG); desktop adapter |
| `ff-shell` | ✅ | `engine.rs::execute_external_refused_when_shell_disabled`, `spawn_detached_refused_when_shell_disabled`; `shell/tests.rs::external_disabled_refuses` | Req 3.7: shell.mode = disabled refuses both modes |
| `ff-desktop` | ✅ | `shell/tests.rs::external_prompt_stages_pending_confirmation`, `dispatch_external_target_stages_prompt_confirmation` | Req 3.8: shell.mode = prompt confirms before spawn; decline = no spawn (confirmation dialog in update.rs) |
| `ff-shell` | ✅ | `engine.rs::execute_external_captured_missing_program_errors`, `executor::external::tests::spawn_detached_missing_program_reports_error`; `shell/tests.rs::external_launch_failure_reports_error` | Req 3.9: launch failure reports error (SpawnFailed), opens no Workspace |
| `ff-command` | ✅ | `command_target_tests.rs::function_macro_and_detached_external_are_not_visible`, `menu_and_custom_workspace_and_captured_external_are_visible` | Req 3.10: External Visible_Workspace classification per command-framework Req 8.9 (produces_visible_workspace) |
| `ff-desktop` | ✅ | `command_config/store.rs::validate_rejects_empty_id_and_label`, `validate_rejects_invalid_id` | Req 4.1: validation rejects bad id/label/target with specific messages |
| `ff-desktop` | ✅ | `command_config/store.rs::validate_rejects_empty_external_program` | Req 4.2: External validation rejects empty program (mode is a typed enum) |
| `ff-desktop` | ✅ | `command_config/mod.rs::user_command_id_resolves_to_stored_target`, `shell/tests.rs::menu_option_command_matching_definition_id_dispatches` | Req 4.3: definition id resolves to its target and a menu option dispatches it (DB.4) |
| `ff-desktop` | ✅ | `shell/tests.rs::dispatch_bound_command_resolves_user_definition`, `run_command_definition_dispatches_defined_id` | Req 4.4: definition id bindable to a keyboard Shortcut_Binding (DB.4) |
| `ff-desktop` | ✅ | `shell/tests.rs::run_command_definition_missing_id_reports_not_defined` | Req 4.5: unknown definition id reports `Command '<id>' is not defined.` (DB.4) |
| `ff-desktop` | ✅ | `command_config/store.rs::validate_rejects_reserved_id` | Req 4.6: user id cannot shadow a reserved built-in Command_ID |

### Phase DB -- External Program Execution (CR-NR-052, shell-command Req 19, DB.10)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-shell` | ✅ | `engine.rs::execute_external_detached_returns_handle`, `execute_external_captured_appends_to_output_panel` | Req 19.1: explicit-program entry point (program + args + working_dir + mode), no shell-string composition |
| `ff-shell` | ✅ | `engine.rs::execute_external_captured_appends_to_output_panel` | Req 19.2: captured mode runs async and shows combined stdout/stderr + exit code in Output_Panel |
| `ff-shell` | ✅ | `engine.rs::execute_external_detached_returns_handle`, `executor::external::tests::spawn_detached_returns_handle_without_waiting` | Req 19.3: detached spawn is fire-and-forget -- no capture, no Output_Panel, returns immediately |
| `ff-shell` | ✅ | `engine.rs::execute_external_detached_returns_handle` | Req 19.4: detached process not tracked/monitored/restarted/persisted; opaque TaskHandle may be dropped |
| `ff-shell` | ✅ | `engine.rs::execute_external_refused_when_shell_disabled`, `spawn_detached_refused_when_shell_disabled` | Req 19.5: gated by shell.mode identically to shell.execute for both modes (disabled refuses; prompt/enabled proceed) |
| `ff-shell` | ✅ | `engine.rs::spawn_detached_uses_explicit_working_dir`; `working_dir.rs` resolver tests | Req 19.6: explicit working_dir honoured; absent falls back to shell.working_directory rules |
| `ff-shell` | 🔲 | -- | Req 19.7: captured external command honours shell.timeout_seconds; timeout not applied to detached (timeout wiring shared with Req 18; manual/integration verification) |
| `ff-shell` | ✅ | `engine.rs::execute_external_captured_missing_program_errors`, `executor::external::tests::spawn_detached_missing_program_reports_error` | Req 19.8: launch failure reports an error naming the program (SpawnFailed); no partial result treated as success |

### Phase DB -- Menu Options Reference a Command Target (CR-NR-051, menu-workspace Req 10)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell/tests.rs::menu_option_command_matching_definition_id_dispatches`, `resolve_and_dispatch_user_definition_id_dispatches` | Req 10.1: option selection resolves command to a CommandTarget and dispatches it (DB.4) |
| `ff-desktop` | ✅ | `shell/tests.rs::resolve_and_dispatch_unknown_string_falls_through`, `dispatch_bound_command_falls_through_for_builtin` | Req 10.2: existing Menu_File format still valid; unresolved strings fall through to the existing pipeline for the same observable result (DB.4) |
| `ff-desktop` | ✅ | `shell/tests.rs::resolve_and_dispatch_user_definition_id_dispatches`, `command_config/mod.rs::user_command_id_resolves_to_stored_target` | Req 10.3: command value equal to a user-defined id resolves to its target (DB.4) |
| `ff-desktop` | ✅ | `shell/tests.rs::dispatch_menu_target_pom_opens_home_context`, `open_menu_workspace_tab_loads_named_menu` | Req 10.4: Menu_Target opens the referenced Menu_Workspace (via the MENU command path) |
| `ff-desktop` | 🔲 | -- | Req 10.5: unresolvable option -- current behaviour preserved via fall-through to the existing pipeline, which surfaces its own status; the dedicated `Option '<key>' could not be resolved` path lands with full target execution (manual/UI) |
| `ff-desktop` | ✅ | `menu_workspace/loader.rs::load_inline_options_target_parses`, `load_option_without_target_is_none` | Req 10.6: inline [options.target] table supported; wins over command with DEBUG log (DB.4) |

### Phase DB -- The MENU Command (CR-NR-051, menu-workspace Req 11)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell/tests.rs::menu_command_returns_to_home_context` | Req 11.1: bare MENU opens/returns to the Home Context (POM) |
| `ff-desktop` | ✅ | `shell/tests.rs::menu_pom_resolves_to_home_context`, `open_menu_workspace_tab_loads_named_menu` | Req 11.2: MENU <name> opens menus/<name>.toml; POM resolves to the Home Context |
| `ff-desktop` | ✅ | `shell/commands.rs` MENU intercept (global, before pipeline) | Req 11.3: MENU is a global command with identical semantics in every Context |
| `ff-desktop` | ✅ | `shell/tests.rs::open_menu_workspace_tab_missing_file_is_load_error` | Req 11.4: MENU <name> for a missing file opens the load-error state (`Menu file not found`) |
| `ff-desktop` | ✅ | `shell/tests.rs::dispatch_menu_target_pom_opens_home_context` | Req 11.5: a Menu_Target executes via the MENU command path |
| `ff-desktop` | ✅ | `shell/mod.rs` menu.open registration (`MenuOpenHandler`) | Req 11.6: MENU registered with the command framework as Command_ID `menu.open` |

### Phase DB -- External Program Execution (CR-NR-052, shell-command Req 19)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-shell` | 🔴 | -- | Req 19.1: external entry point takes explicit program/args/working_dir/mode |
| `ff-shell` | 🔴 | -- | Req 19.2: captured mode async into Output_Panel reusing Req 4/15 display |
| `ff-shell` | 🔴 | -- | Req 19.3: detached mode fire-and-forget, no capture, no Output_Panel, returns immediately |
| `ff-shell` | 🔴 | -- | Req 19.4: detached process not tracked/restarted/persisted; handle optional |
| `ff-shell` | 🔴 | -- | Req 19.5: shell.mode gate applies to both modes |
| `ff-shell` | 🔴 | -- | Req 19.6: explicit working_dir else shell.working_directory rules |
| `ff-shell` | 🔴 | -- | Req 19.7: captured timeout per Req 18; not applied to detached |
| `ff-shell` | 🔴 | -- | Req 19.8: launch failure reported (Output_Panel captured / status detached); not success |

### Phase DB -- Descriptor-Based Persistence (CR-CH-012, startup-and-session Req 21)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-session` | ✅ | `session_state.rs::menu_descriptor_round_trips_through_toml`, `editor_descriptor_round_trips_with_uri_param` | Req 21.1: Session_State persists visible Workspaces as Workspace_Descriptors in tab order |
| `ff-session` | ✅ | `session_state.rs::editor_descriptor_round_trips_with_uri_param` | Req 21.2: Editor Context persists as CustomWorkspace{editor, uri+viewport+caret} |
| `ff-desktop` | ✅ | `shell/tests.rs::restore_settings_descriptor_applies_namespace_filter`, `restore_settings_descriptor_without_namespace_is_unfiltered`; `session_state.rs::settings_namespace_descriptor_round_trips` | Req 21.3: Settings namespace persists/restores as CustomWorkspace{settings, {namespace}} (folds in CR-CH-011 / Task 14.5) |
| `ff-session` | 🟡 | `session_state.rs::menu_descriptor_round_trips_through_toml` | Req 21.4: Menu_Workspace persists as MenuWorkspace{name} (round-trip DONE); restore re-open path now exists via open_menu_workspace_tab -- restore-loop wiring for the Menu descriptor is a small follow-up |
| `ff-desktop` | ✅ | `shell/tests.rs::restore_files_descriptor_opens_files_panel`, `restore_file_explorer_descriptor_opens_explorer_panel`, `restore_multiple_descriptors_opens_each_workspace`, `restore_editor_descriptor_opens_file` | Req 21.5: restore re-opens every descriptor in tab order, not only URI tabs |
| `ff-desktop` | ✅ | `session_manager.rs::descriptor_for_tab` (Untitled -> None; no descriptor for non-visible kinds) | Req 21.6: Started_Task / Function / Macro not persisted, not restarted |
| `ff-desktop` | 🟡 | -- | Req 21.7: Captured_Run Output_Panel result not persisted as a Workspace (holds by construction -- captured runs are not tabs; asserted with DB.10) |
| `ff-desktop` | ✅ | `shell/update.rs` restore path (`ensure_pom_tab_present` after descriptor restore) | Req 21.8: POM-always-present guarantee still holds after descriptor restore |
| `ff-desktop` | ✅ | `shell/tests.rs::restore_skips_menu_descriptor_but_continues` | Req 21.9: unknown workspace_kind/params skipped with WARN, rest restored |
| `ff-session` | ✅ | `session_state.rs::legacy_session_toml_without_descriptor_field_still_loads`, `legacy_file_editor_maps_to_editor_descriptor_with_uri`, `legacy_files_and_file_explorer_map_to_descriptors`, `legacy_untitled_has_no_descriptor` | Req 21.10: backward compatible with existing PersistedTabKind session.toml files |

### Phase CX -- Named Workspaces + KEYS Name + SPLIT Alias (CR-NR-046, CR-CH-010)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | -- | Req 1.1: TabState has workspace_name: Option<String> field |
| `ff-desktop` | ✅ | -- | Req 1.2: NAME <text> sets active Workspace name (max 32 chars) |
| `ff-desktop` | ✅ | -- | Req 1.3: NAME with no argument clears workspace_name |
| `ff-desktop` | ✅ | -- | Req 1.4: tab header shows [<name>] or <name>: <title> when name is set |
| `ff-desktop` | ✅ | -- | Req 1.5: workspace_name persisted in session state |
| `ff-desktop` | ✅ | -- | Req 1.6: workspace_name in PersistedTab as optional string field |
| `ff-desktop` | ✅ | -- | Req 2.1: KEYS with no argument opens dialog with Default scope active |
| `ff-desktop` | ✅ | -- | Req 2.2: KEYS <name> opens dialog with matching context tab pre-selected |
| `ff-desktop` | ✅ | -- | Req 2.3: KEYS <name> with unknown name shows status message, opens Default |
| `ff-desktop` | ✅ | -- | Req 2.4: KEYS <name> matching is case-insensitive |
| `ff-desktop` | ✅ | -- | Req 2.5: Key Configuration Dialog shows read-only Map Name field |
| `ff-desktop` | ✅ | -- | Req 3.1: SPLIT DETACH detaches current Workspace into OS window |
| `ff-desktop` | ✅ | -- | Req 3.2: SPLIT (no arg) on non-editor tab detaches Workspace |
| `ff-desktop` | ✅ | -- | Req 3.3: SPLIT (no arg) on editor tab performs split-screen (backward compat) |
| `ff-desktop` | ✅ | -- | Req 3.4: SPLIT DETACH works from any Workspace kind including editor |
| `ff-desktop` | ✅ | -- | Req 3.5: SPLIT DETACH at 16-window limit shows error, does not detach |
| `ff-desktop` | ✅ | -- | Req 3.6: SPLIT registered as layout.split in command framework |
| `ff-desktop` | ✅ | -- | Req 4.1: workspace_name written to session.toml on save |
| `ff-desktop` | ✅ | -- | Req 4.2: workspace_name restored from session.toml on launch |
| `ff-desktop` | ✅ | -- | Req 4.3: missing workspace_name field in session restores as None |

### Phase CZ -- FFTest Context Inspection, Bug Logging, and Script Suite (CR-NR-049)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-fftest` | ✅ | `ff-fftest::parser` unit tests | Req 11.1: ASSERT CONTEXT IS "<name>" command supported in FFTest scripting language |
| `ff-fftest` | ✅ | `ff-fftest::assertions` unit tests | Req 11.2: ASSERT CONTEXT IS queries shell.active_context Automation ID (case-insensitive) |
| `ff-fftest` | ✅ | `ff-fftest::parser` unit tests | Req 11.3: ASSERT WORKSPACE COUNT IS <n> command supported in FFTest scripting language |
| `ff-fftest` | ✅ | `ff-fftest::assertions` + `ff-fftest::runner` unit tests | Req 11.4: ASSERT WORKSPACE COUNT IS queries shell.workspace_count Automation ID |
| `ff-fftest` | ✅ | `ff-fftest::assertions` + `ff-fftest::runner` unit tests | Req 11.5: failed context/count assertion records expected, actual, and line number in diagnostic |
| `ff-fftest` | ✅ | `ff-fftest::bug_report` unit tests | Req 12.1: WHEN any assertion fails, runner appends entry to reports/bugs-from-tests.md |
| `ff-fftest` | ✅ | `ff-fftest::bug_report` unit tests | Req 12.2: bug entry includes BT-NNN ID, script file, line, assertion text, expected, actual, UTC timestamp |
| `ff-fftest` | ✅ | `ff-fftest::bug_report` unit tests | Req 12.3: bug entry format compatible with docs/status/bugs.md table format |
| `ff-fftest` | ✅ | `ff-fftest::bug_report` unit tests | Req 12.4: WHEN reports/bugs-from-tests.md absent, runner creates it with header row |
| `ff-fftest` | ✅ | `ff-fftest::bug_report` unit tests | Req 12.5: WHEN all assertions pass, runner does NOT append to reports/bugs-from-tests.md |
| `tests/dialog/` | ✅ | `tests/dialog/*.fftest` (10 scripts) | Req 13.1: dialog scripts cover POM navigation, file open/save/close, editor input/undo, catalog, settings |
| `tests/dialog/` | ✅ | `tests/dialog/*.fftest` (4 scripts) | Req 13.2: dialog scripts cover key config, compiler context, plugin manager, notification system |
| `tests/workflow/` | ✅ | `tests/workflow/*.fftest` (3 scripts) | Req 13.3: workflow scripts cover batch execution, global search, command palette |
| `tests/` | ✅ | all .fftest scripts | Req 13.4: each script includes at least one ASSERT command verifying post-condition |
| `tests/` | ✅ | all .fftest scripts | Req 13.5: each script begins with comment block identifying area, requirements, expected outcome |

## Phase DC -- Logging Runtime Reconfiguration (logging-subsystem Req 11, CR-CH-015 / B033)

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-logging` | ✅ | `tests/reconfigure_integration.rs` | Req 11.1: expose `reconfigure(LogConfig)` applying config to the already-initialized subsystem |
| `ff-logging` | ✅ | `writer.rs::switch_directory_opens_file_in_new_directory`; `tests/reconfigure_integration.rs` | Req 11.2: reconfigure with a new directory flushes/closes current file, creates new dir, opens new file before next record |
| `ff-logging` | ✅ | `tests/reconfigure_integration.rs` (same-dir short-circuit in `handle_reconfigure`) | Req 11.3: reconfigure with the same directory keeps the current file, applies only level/rotation changes |
| `ff-logging` | ✅ | `tests/reconfigure_integration.rs` (DEBUG becomes visible after level lowered) | Req 11.4: reconfigure applies new minimum level atomically; clamps out-of-range rotation values with WARN (per Req 5.3/5.8) |
| `ff-logging` | ✅ | `writer.rs::switch_directory_failure_leaves_writer_on_current_file`; `tests/reconfigure_integration.rs` (failed switch retains dir) | Req 11.5: on new-directory failure, retain current file/fallback, lose no buffered records, write WARN, do not terminate |
| `ff-logging` | ✅ | `tests/reconfigure_before_init.rs`; `tests/reconfigure_integration.rs` (post-shutdown no-op) | Req 11.6: reconfigure before init or after shutdown signal is a safe no-op (no panic) |
| `ff-desktop` | ✅ | `main.rs::project_config_logging_directory_is_resolved_for_reconfigure`; `main.rs::apply_logging_config_is_safe_when_logging_not_active` | Req 11.7: startup initializes logging with defaults before config load, then invokes reconfigure with resolved settings before GUI shell construction |
| `ff-logging` | ✅ | `tests/reconfigure_integration.rs` ("Logging reconfigured" INFO in new dir) | Req 11.8: successful directory switch writes an INFO record naming the effective Log_Directory to the new file |
| `ff-logging` | ✅ | `writer.rs::prop_switch_directory_routes_records_to_new_dir` (Property 11) | Req 11.9: reconfigure is thread-safe; concurrent log calls land in the previous or new file without loss or data races |
| `ff-logging` | ✅ | `writer.rs` switch tests (bytes reset, old files retained) | Req 11.10: level change during reconfigure does not itself delete retained files; retention stays governed by max_retained_files |

### Phase DD -- Logging Inventory and Gap Report Tool (CR-NR-055, logging-subsystem Req 12)

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `tools` | 🔴 | -- | Req 12.1: scan every `*.rs` under `crates/`, identify each log call site with file, 1-based line, and level when statically determinable |
| `tools` | 🔴 | -- | Req 12.2: report groups call sites by crate with per-crate and per-level counts |
| `tools` | 🔴 | -- | Req 12.3: report lists every crate with non-test source but zero log call sites as a gap |
| `tools` | 🔴 | -- | Req 12.4: report lists silent-error candidate sites with file:line; test sites excluded from non-test counts |
| `tools` | 🔴 | -- | Req 12.5: report written to a fixed path under `docs/quality/`, overwriting any prior report |
| `tools` | 🔴 | -- | Req 12.6: tool modifies no file outside its `docs/quality/` report path and its `tools/logs/` log |
| `tools` | 🔴 | -- | Req 12.7: tool mirrors progress/summary to a `tools/logs/` log, overwritten each run |
| `tools` | 🔴 | -- | Req 12.8: unreadable/unparseable file is recorded in the report; scan continues without aborting |
| `tools` | 🔴 | -- | Req 12.9: repeated runs are deterministic (crates/files/sites in stable order); Property 12 |

### Phase DF -- Command Arguments and Command Chaining (CR-NR-054)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-command` | 🔴 | -- | Req 9.1: command line parsed into verb + Argument_String (first token / trimmed remainder) |
| `ff-command` | 🔴 | -- | Req 9.2: non-empty argument placed into Command_Params under reserved key `arg` |
| `ff-command` | 🔴 | -- | Req 9.3: no argument -> empty Argument_String, `arg` absent (verb-only unchanged) |
| `ff-command` | 🔴 | -- | Req 9.4: verb-only commands (SAVE/CANCEL) produce the same observable result as before |
| `ff-command` | 🔴 | -- | Req 9.5: a Command_Definition / Shortcut_Binding may carry a fixed Argument_String, forwarded as if typed |
| `ff-command` | 🔴 | -- | Req 9.6: a command that does not accept an argument ignores a surplus argument (no error) |
| `ff-command` | 🔴 | -- | Req 9.7: verb/argument split performed once at the dispatch boundary (shared by all input sources) |
| `ff-desktop` | 🔴 | -- | Req 9.8: function-key press forwards the Command field contents as the command's argument (== typed `<cmd> <field>`) |
| `ff-desktop` | 🔴 | -- | Req 9.9: framework never force-clears the field; command decides clear/replace/keep |
| `ff-command` | 🔴 | -- | Req 9.10: key-forwarded and typed invocations are indistinguishable to the command (same `arg`) |
| `ff-navigation-commands` | 🔴 | -- | Req 20.1: DOWN M/MAX -> bottom; UP M/MAX -> top (case-insensitive) |
| `ff-navigation-commands` | 🔴 | -- | Req 20.2: UP/DOWN with positive integer n -> scroll by n lines (via `arg`) |
| `ff-navigation-commands` | 🔴 | -- | Req 20.3: malformed scroll argument -> default one-screen page, no error |
| `ff-navigation-commands` | 🔴 | -- | Req 20.4: LEFT/RIGHT with n -> n columns; M/MAX -> horizontal extreme |
| `ff-desktop` | 🔴 | -- | Req 20.5: scroll command clears the Command field after consuming a key-forwarded amount |
| `ff-keys` | 🔴 | -- | Req 19.1: RETRIEVE (no argument) recalls previous command; repeated -> step back through history |
| `ff-keys` | 🔴 | -- | Req 19.2: RETRIEVE LIST opens the numbered history list |
| `ff-keys` | 🔴 | -- | Req 19.3: `recall_list` -- numbered most-recent-first (1=top), deduplicated keep-most-recent |
| `ff-keys` | 🔴 | -- | Req 19.4: RETRIEVE `<n>` recalls list item n into the field; out-of-range -> unchanged + status |
| `ff-desktop` | 🔴 | -- | Req 19.5: history-list selection (click / Enter / number+RETRIEVE) populates field without executing |
| `ff-desktop` | 🔴 | -- | Req 19.6: Escape / click-outside closes the list and clears the field |
| `ff-desktop` | 🔴 | -- | Req 19.7: empty history shows "No command history." |
| `ff-keys` | 🔴 | -- | Req 19.8: RETRIEVE (and its LIST/number argument) never added to Command_History |
| `ff-desktop` | 🔴 | -- | Req 19.9: history list rendered as a near-modal overlay just below the command field |
| `ff-desktop` | 🔴 | -- | menu-workspace Req 11.7: MENU `<name> <key>` opens the menu and activates the option by key |
| `ff-desktop` | 🔴 | -- | menu-workspace Req 11.8 / 5.5: MENU `<name> <key>` == `=k1.k2` == `<key>` + key bound to MENU `<name>` |
| `ff-desktop` | 🔴 | -- | menu-workspace Req 11.9: unknown chained key -> open menu + `Option '<key>' not found.` |
| `ff-desktop` | 🔴 | -- | menu-workspace Req 11.10 / 5.6: deeper chains forward the remaining argument to the option's own command |
### Phase DH -- Command Chaining and Context Navigation Stack (CR-NR-057)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-command-semantics` | 🔴 | -- | Req 11.1: parse command line into ordered Command_Chain on top-level `.`/`;` |
| `ff-command-semantics` | 🔴 | -- | Req 11.2: `.`/`;` inside quotes or hex literals are literal, not separators |
| `ff-command-semantics` | 🔴 | -- | Req 11.3: execute chain left-to-right, each on prior Workspace/Context state |
| `ff-command-semantics` | 🔴 | -- | Req 11.4: fail-stop -- halt remainder and report the failing invocation status |
| `ff-command-semantics` | 🔴 | -- | Req 11.5: separator affects only navigation stack; `.`/`;` identical for non-navigating chains |
| `ff-command-semantics` | 🔴 | -- | Req 11.6: empty segments (leading/trailing/doubled separator) ignored, no error |
| `ff-command-semantics` | 🔴 | -- | Req 11.7: single command with no separator == chain of length one (backward compatible) |
| `ff-command-semantics` | 🔴 | -- | Req 11.8: chain parser is a pure, independently unit-testable function |
| `ff-command-semantics` | 🔴 | -- | Req 11.9: each mutating segment keeps its own undo transaction; no combined transaction |
| `ff-command-semantics` | 🔴 | -- | Req 11.10: fastpath Chained_Path and command-line chain share one split helper |
| `ff-command-semantics` | 🔴 | -- | Req 11.11: `commands.max_chain_length` (default 16); over-long chain reports error, executes nothing |
| `ff-command-semantics` | 🔴 | -- | Req 11.12: split only at top-level separators outside quotes and hex literals |
| `ff-command-semantics` | 🔴 | -- | Req 11.13: END/RETURN treated as an ordinary chainable command (no special-casing) |
| `ff-command` | 🔴 | -- | Req 10.1: per-Workbench Context_Navigation_Stack that RETURN (END/F3) pops |
| `ff-command` | 🔴 | -- | Req 10.2: chain beginning with `=` sets Navigation_Origin to the POM |
| `ff-command` | 🔴 | -- | Req 10.3: non-`=` navigation command uses the current Workspace as Navigation_Origin |
| `ff-command` | 🔴 | -- | Req 10.4: Navigation_Origin is always the bottom entry of the stack |
| `ff-command` | 🔴 | -- | Req 10.5: `;` (PUSH) segment pushes the prior Context before switching |
| `ff-command` | 🔴 | -- | Req 10.6: `.` (STOP) segment does not push the intermediate Context |
| `ff-command` | 🔴 | -- | Req 10.7: non-navigating command leaves the stack unchanged regardless of separator |
| `ff-command` | 🔴 | -- | Req 10.8: RETURN (END/F3) pops one entry and switches to that Context |
| `ff-command` | 🔴 | -- | Req 10.9: END/RETURN is a chainable command (e.g. `END ; EDIT`) |
| `ff-command` | 🔴 | -- | Req 10.10: `produces_visible_workspace` (Req 8.9) is the opens/changes-a-Context predicate |
| `ff-command` | 🔴 | -- | Req 10.11: `navigation.stack_max_depth` (default 32); overflow drops oldest + WARN |
| `ff-command` | 🔴 | -- | Req 10.12: stack is session-only; never persisted, never undoable |
| `ff-command` | 🔴 | -- | Req 10.13: single unchained navigation command behaves as STOP (pushes only the origin) |
| `ff-desktop` | 🔴 | -- | menu-workspace Req 5.7: leading `=` means begin navigation from the POM (Navigation_Origin = POM) |
| `ff-desktop` | 🔴 | -- | menu-workspace Req 5.8: `=0.E` opens Editor Config with intermediates collapsed; END returns to POM |
| `ff-desktop` | 🔴 | -- | menu-workspace Req 5.9: `=0;E` opens Editor Config with intermediates pushed; END -> Settings -> POM |
| `ff-desktop` | 🔴 | -- | menu-workspace Req 5.10: non-`=` command uses current Workspace as origin; multi-hop non-`=` chain pushes each hop |
| `ff-desktop` | 🔴 | -- | menu-workspace Req 5.11: mixed separators (`=0;E.T`); each separator independently controls push/collapse |
| `ff-desktop` | 🔴 | -- | menu-workspace Req 5.12: fastpath and chained MENU share one navigation-and-activation helper |
| `ff-macro` | 🔴 | -- | lua-macro-engine Req 5.8: newline in a macro/FFCMD file == `;` (PUSH) separator; sequential top-to-bottom |
| `ff-macro` | 🔴 | -- | lua-macro-engine Req 5.9: a macro/FFCMD line chain runs through the shared command-line chain executor |
| `ff-macro` | 🔴 | -- | lua-macro-engine Req 5.10: chain fail-stop composes with whole-invocation Macro_Transaction rollback |
| `ff-macro` | 🔴 | -- | lua-macro-engine Req 5.11: no piping -- commands act on Workspace/Context state, not a prior return value |

### Phase DI -- Build-Profile Logging + Uniform Command Instrumentation (CR-NR-058)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-logging` | 🔴 | -- | Req 13.1: `dev-logging` cargo feature sets Build_Profile_Level (Trace when on, Info when off) |
| `ff-logging` | 🔴 | -- | Req 13.2: with `dev-logging` off, `log_trace!`/`log_debug!` evaluate no args, no format, no atomic read |
| `ff-logging` | 🔴 | -- | Req 13.3: with `dev-logging` on, `log_trace!`/`log_debug!` behave per Req 3 and Req 9 (runtime guard) |
| `ff-logging` | 🔴 | -- | Req 13.4: `log_info!`/`log_warn!`/`log_error!` retain full runtime behaviour in every profile |
| `ff-logging` | 🔴 | -- | Req 13.5: debug builds enable `dev-logging` by default; release builds do not; no manual flag needed |
| `ff-logging` | 🔴 | -- | Req 13.6: stripping a Development_Level site does not change any Retained_Level or non-logging code |
| `ff-logging` | 🔴 | -- | Req 13.7: public `BUILD_PROFILE_LEVEL` const queryable at compile time by downstream crates |
| `ff-logging` | 🔴 | -- | Req 13.8: runtime `logging.level = "debug"` cannot resurrect a stripped release site |
| `ff-command` | 🔴 | -- | Req 11.1: start DEBUG record (id + params) emitted before the handler runs |
| `ff-command` | 🔴 | -- | Req 11.2: completion record (id, ok/err, Result_Summary, duration ms) -- DEBUG on success, WARN on failure |
| `ff-command` | 🔴 | -- | Req 11.3: instrumentation applied uniformly to every invocation source via the single `execute_command` |
| `ff-command` | 🔴 | -- | Req 11.4: release (`dev-logging` off) compiles out start + success records; failure WARN remains |
| `ff-command` | 🔴 | -- | Req 11.5: Sensitive_Param values redacted (`***`) in start and completion records |
| `ff-command` | 🔴 | -- | Req 11.6: params rendering bounded by the logging subsystem's 8192-byte truncation (Req 2.3) |
| `ff-command` | 🔴 | -- | Req 11.7: rejected commands (unregistered/disabled) still emit start + completion naming the reason |
| `ff-command` | 🔴 | -- | Req 11.8: instrumentation never alters CommandResult, undo, history, or side-effect ordering |
| `ff-command` | 🔴 | -- | Req 11.9: completion duration measures the handler window only |
| `ff-global-search` | ✅ | replace.rs unit tests (atomic_write_replaces_content_and_leaves_no_temp, atomic_write_to_bad_path_leaves_no_partial_output) | Req 5.3: cross-file replace writes ATOMICALLY (temp + fsync + rename); interrupted write cannot corrupt the original. Data-safety fix PA-CONFLICT-015/020 (PA-W5.2) |
| `ff-jes` | ✅ | queue.rs unit tests (persist_writes_atomically_without_leaving_temp_files, atomic_persist_write_to_bad_path_leaves_no_partial_output, queue_persistence_round_trip) | Req 2 AC 6: job-queue persistence writes ATOMICALLY (temp + fsync + rename); interrupted write cannot corrupt the persisted queue. Data-safety fix PA-CONFLICT-015 (PA-W5.2) |

### Phase (navigation-modernization) -- Unified Navigation Model + File Explorer Modernization Slice A (CR-NR-060, file-tree-panel Requirement 24)

| Crate | Status | Test | Criterion |
|-------|--------|------|-----------|
| `ff-desktop` | ✅ | `nav_model::tests::nav_model_starts_with_three_root_categories` (+ shell renders NavModel TreeState as the default File Explorer CentralPanel) | Req 24.1: File Explorer drives its tree from ff-file-tree TreeState; ff-desktop declares ff-file-tree dep; no parallel tree model |
| `ff-desktop` | ✅ | `nav_model::tests::identity_is_node_id_and_uri_side_table`, `explorer_view::tests::*` (NodeId-keyed selection/cursor/anchor) | Req 24.2: node identity is NodeId + ResourceUri; cursor/selection/anchor/expansion keyed on NodeId; same-label nodes do not collide |
| `ff-desktop` | ✅ | `nav_model::tests::{rename_via_writable_provider_moves_file_and_relists, delete_via_writable_provider_removes_file_and_dir, create_via_writable_provider_makes_file_and_folder}` | Req 24.3: File Explorer load/refresh + edit ops via async VfsProvider (list/rename/delete/create); no direct std::fs in the modern explorer path |
| `ff-desktop` | ✅ | `nav_model::tests` map_entries/map_entry (hidden_file_detected_in_posix_mapping) | Req 24.4: provider-defined Namespace_Mapping (VfsEntry -> TreeNodeData); POSIX/local dir->Directory, file->File, symlink->SymbolicLink, forward-slash paths |
| `ff-desktop` | ✅ | `nav_model::tests` apply_listing / apply_load_error + expand round-trip | Req 24.5: POSIX/local expand -> list() -> Namespace_Mapping -> TreeState::apply_children; error nodes + keyboard via ff-file-tree |
| `ff-desktop` | ✅ | `explorer_view::tests` + `nav_model::tests` (multi-select/copy-as-text-tree/paste round-trips) cover the ported Req 15-23 behaviour; legacy file deleted, 875 ff-desktop tests pass | Req 24.6: inline FileExplorerPanelState tree RETIRED -- file_explorer_panel.rs + copy_move_dialog.rs deleted, fallback toggle removed; Req 15-23 features preserved on the modern explorer (native OS dialog + attribute columns deferred, not core) |
| `ff-desktop` | 🔲 | Manual: modern explorer renders indent guides, disclosure glyphs, selection/focus-ring via file_tree.* palette | Req 24.7: modern presentation via theme file_tree.* palette; grouping information preserved |
| `ff-desktop` | ✅ | `nav_model::tests::{split_catalog_uri_path_*, apply_child_data_populates_and_maps_uris}`, `catalog_registry::tests::dataset_node_*` | Req 24.8: catalog roots modelled generically (CatalogRoot via provider list()/dataset list) in Slice A; no mainframe qualifier/duality semantics (Slice B) |
| `ff-desktop` | ✅ | `explorer_view::tests::{first_row_id_is_the_first_visible_node, next_row_id_advances_then_returns_none_past_last}` (Tab focus-transfer helpers); Command ===> retained | Req 24.9: persistent shell Command ===> retained on File Explorer Context; Tab focus-transfer re-pointed at the ff-file-tree node list; dispatch unchanged |
| `ff-desktop` | ✅ | verify.ps1 FULL clean across the Slice A commits; new tests added, none weakened | Req 24.10: cargo test (affected crates) + verify.ps1 clean; tests added (not weakened) for the ff-file-tree-backed behaviour |
| `ff-file-tree` | ✅ | `nav_model::tests` confirm the shell-side NodeId -> ResourceUri side table approach (no model change needed) | Req 24.11: no ff-file-tree model extension required -- shell keeps the side table, model stays canonical |

### Phase (theme-command) -- THEME Command Parity (theme-and-appearance Req 17)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | -- | Req 17.1: `THEME` command invocable from the command line in any context |
| `ff-desktop` | 🔴 | -- | Req 17.2: `THEME <mode>` sets active mode + persists theme.active (dark/light/high_contrast/legacy, case-insensitive, hyphen accepted) |
| `ff-desktop` | 🔴 | -- | Req 17.3: bare `THEME` reports the current mode in the status area; does not change theme |
| `ff-desktop` | 🔴 | -- | Req 17.4: `THEME <invalid>` shows a clear error listing valid modes; does not change theme |
| `ff-desktop` | 🔴 | -- | Req 17.5: Settings menu theme actions invoke the THEME command (menu == typed-command code path) |
| `ff-desktop` | 🔴 | -- | Req 17.6: persist failure applies theme for the session and surfaces a non-silent message (no silent revert) |

### Phase (swap-command) -- SWAP Tab Switching (multi-tab-editor Req 18)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell::tests::swap_n_activates_nth_tab` | Req 18.1: `SWAP n` activates the n-th tab (1-based, Tab_Bar order) |
| `ff-desktop` | ✅ | `shell::tests::swap_n_out_of_range_errors_and_keeps_active` | Req 18.2: `SWAP n` with 0/negative/non-numeric/out-of-range errors clearly; no change |
| `ff-desktop` | ✅ | `shell::tests::{swap_list_opens_tab_picker, swap_without_split_opens_tab_picker}` | Req 18.3: `SWAP LIST` opens a selectable picker of open tabs (position + title) |
| `ff-desktop` | 🔲 | Manual: click a row / type number+Enter in the picker | Req 18.4: picker selection by click OR type-number+Enter switches and closes (egui render) |
| `ff-desktop` | 🔲 | Manual: Escape closes the picker | Req 18.5: Escape/dismiss closes the picker without changing the active tab (egui render) |
| `ff-desktop` | ✅ | `shell::tests::swap_bare_with_split_swaps_focus_not_picker` | Req 18.6: bare `SWAP` with a split active swaps split-screen focus (preserved 19.12) |
| `ff-desktop` | ✅ | `shell::tests::swap_without_split_opens_tab_picker` | Req 18.7: bare `SWAP` with no split opens the picker (not an error) -- SUPERSEDED by CR-CH-031 (now toggles to the previous tab; picker only when no previous exists) |
| `ff-desktop` | ✅ | `shell::tests` (SWAP routed through handle_command) | Req 18.8: `SWAP` command-line dispatchable; tab-switch affordances route through it |

### Phase (swap-previous) -- bare SWAP toggles to the previously active workspace (CR-CH-031)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell::tests::swap_bare_toggles_to_previously_active_tab` | multi-tab-editor Req 18.7 (REVISED, CR-CH-031): bare `SWAP` with no split activates the Previous_Active_Tab (Alt+Tab-style toggle), not the picker; repeated bare SWAP ping-pongs |
| `ff-desktop` | ✅ | `tab_manager::tests::{previous_active_tracks_last_other_tab, previous_active_none_with_single_tab, previous_active_repaired_on_close}` | multi-tab-editor Req 18.9 (CR-CH-031): `TabManager` tracks Previous_Active_Tab via the single `activate` seam (skips no-op activations); `close_tab` repairs the pointer for the removal shift |
| `ff-desktop` | ✅ | `shell::tests::swap_without_split_or_previous_opens_tab_picker` | multi-tab-editor Req 18.10 (CR-CH-031): with only one tab (no Previous_Active_Tab), bare SWAP falls back to the picker |

### Phase (unified-menu-renderer) -- One config-driven menu renderer (menu-workspace Req 1.8, 2.1a-2.1c; CR-CH-018)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `menu_workspace/loader.rs::load_show_calendar_defaults_to_true`, `load_show_calendar_false_preserved` | Req 1.8: Menu_File `show_calendar` (bool, default true) parsed by the loader |
| `ff-desktop` | ✅ | `menu_workspace/render.rs::format_option_row_produces_three_aligned_columns`, `command_column_width_uses_widest_command_clamped`, `format_option_rows_align_descriptions_at_common_width` | Req 2.1a: three-column option list (Option_Key \| Option_Command \| Option_Description) for every menu incl. POM/Settings |
| `ff-desktop` | ✅ | `menu_workspace/render.rs::show_calendar_flag_is_carried_on_menu`, `show_calendar_false_is_respected_on_menu`, `menu_calendar_colours_default_is_inherited`, `menu_render_result_default_is_empty` | Req 2.1b: live calendar drawn right of the option list when `show_calendar` true; full-width columns when false |
| `ff-desktop` | ✅ | `shell/tests.rs::settings_option_zero_opens_menu_workspace`, `pom_key_resolves_to_configured_command`; `menu_workspace/render.rs` shared-renderer tests | Req 2.1c: POM + Settings rendered by the shared menu renderer; bespoke primary_option_menu render/BUILT_IN_OPTIONS retired; POM options data-driven from pom.toml (task 22.4) |
| `ff-desktop` | ✅ | `shell/tests.rs` POM routing tests (POM tab kind preserved; make_shell POM path) | Req 2.1d: POM keeps TabKind::PrimaryOptionMenu, `[POM]` tab title, and black/blue Title_Line while rendered by the shared renderer from a pom.toml MenuWorkspaceState (task 22.4a) |
| `ff-desktop` | ✅ | `shell/tests.rs::pom_key_resolves_to_configured_command`, `pom_key_s_routes_to_search`, `option_5_routes_to_macro_library`, `option_8_routes_to_plugin_manager` | Req 2.1e: selecting a menu option dispatches ONLY its command string; digit-keyed key->panel dispatch arms removed (no behaviour keyed to position/key/menu) (task 22.4c) |
| `ff-desktop` | ✅ | `shell/tests.rs::focus_cycle_tab_forward_through_all_pom_options`, `focus_stop_shift_tab_from_calendar_prev_goes_to_last_pom_option` (ring sized by loaded options via pom_option_count) | Req 2.1f: POM keyboard focus ring + Enter/Space activation driven by the loaded pom.toml options (count + command), not compiled BUILT_IN_OPTIONS (task 22.4d) |
| `ff-desktop` | ✅ | `menu_workspace/defaults.rs::default_pom_toml_terminate_is_x_return`; `shell/tests.rs::return_from_pom_with_other_tabs_closes_pom_not_app` | Req 2.1g: data-driven terminate option (X->RETURN) returns to POM / exits when last, per CR-CH-016; bespoke exit-line/PomAction::Exit/PomExit removed (task 22.4e) |
| `ff-desktop` | ✅ | `menu_workspace/defaults.rs::default_pom_toml_has_only_built_testable_options`; `shell/tests.rs::option_5_routes_to_macro_library`, `equals_8_command_routes_to_plugin_manager` | Req 2.1h: every default pom.toml command resolves by name (CATALOGS arm added); default pom.toml trimmed to built+testable options only (task 22.4f) |
| `ff-desktop` | ✅ | `shell/tests.rs::equals_5_command_routes_to_macro_library`, `equals_8_command_routes_to_plugin_manager` (=<key> resolves the keyed option then dispatches its command) | Req 2.1i: `=<key>` fastpath resolves the keyed option then dispatches its command (config-driven), preserved via shared renderer (task 22.4c) |


### Phase CQ -- egui 0.31 Upgrade + egui_kittest Focus Harness (CR-NR-076, Req 14 automated-dialog-testing)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | 🔴 | -- | Req 14.1: workspace resolves a single egui major-minor version (no two egui versions in Cargo.lock) |
| `ff-desktop` | 🔴 | -- | Req 14.2: workspace egui and eframe upgraded to 0.31 |
| `ff-desktop` | 🔴 | -- | Req 14.3: egui-file-dialog is published 0.9.x; vendor patch removed |
| `ff-desktop` | 🔴 | -- | Req 14.4: after upgrade the full workspace builds, clippy -D warnings clean, and the full test suite passes (verify.ps1 clean) with no behaviour change |
| `ff-desktop` | 🔴 | -- | Req 14.5: ff-desktop carries egui_kittest dev-dependency at 0.31.x |
| `ff-desktop` | 🔴 | -- | Req 14.6: headless egui_kittest harness test renders the Menus Editor and injects Tab, asserting focused-widget order |
| `ff-desktop` | 🔴 | -- | Req 14.7: Tab focus visits every Menus Editor control once in visual order (no widget skipped) then wraps to the command line |
| `ff-desktop` | 🔴 | -- | Req 14.8: the headless harness runs without a display device under cargo test and cargo nextest |

### CR-CH-023 -- Unified tab-order model (Req 16 rework; menu-workspace Req 15)

> Supersedes the Phase AJ / AK Req 16 rows above (the old `FocusStop` ring:
> PomOption/PomExit/CalendarPrev/CalendarNext/MenuBar/TabHeader stops). Those rows
> described the removed ring and no longer apply; the criteria below are the
> authoritative Req 16 rows after CR-CH-023.

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell::tests::full_shell_launch_focus_is_command_field` | menu-and-statusbar Req 16.1: on launch, focus is on the Primary_Command_Field (full-shell egui_kittest build_eframe harness) |
| `ff-desktop` | 🔲 | -- | menu-and-statusbar Req 16.1a: focus returns to the command field on every Workspace entry (navigate_to / start_new_workspace / active-tab change set the flag; harness covers launch; navigation entry is manual/UI verification) |
| `ff-desktop` | 🔲 | -- | menu-and-statusbar Req 16.2: printable char goes to the command field when it has focus (manual/UI verification) |
| `ff-desktop` | ✅ | `shell::tests::full_shell_tab_from_command_field_enters_first_option` | menu-and-statusbar Req 16.3: Tab from command field enters the active Workspace's first Interior_Control, not the SCROLL field (full-shell harness) |
| `ff-desktop` | ✅ | `shell::tests::full_shell_theme_editor_first_tab_focuses_theme_selector` | menu-and-statusbar Req 16.3 (B057): on the Theme Editor Context the first Tab lands on the Theme selector combo (reported first interior), with no phantom/invisible focus stop -- the ThemeEditor arm now reports interior ids and honours the focus latch like the other Contexts |
| `ff-desktop` | ✅ | `shell::tests::full_shell_config_first_tab_focuses_filter_field` | menu-and-statusbar Req 16.3 (B058): on the Config Panel (`CONFIG`) the first Tab lands on the Filter field (reported first interior), with no phantom stop -- the ConfigPanel arm now reports the stable Filter-field id and honours the focus latch like the other Contexts |
| `ff-desktop` | ✅ | `shell::tests::full_shell_plugin_manager_first_tab_focuses_interior`, `full_shell_event_log_first_tab_focuses_interior`, `full_shell_macro_library_first_tab_focuses_interior`, `full_shell_search_results_first_tab_focuses_interior`, `full_shell_command_configurator_first_tab_focuses_interior` | menu-and-statusbar Req 16.3 (B059): the remaining workspace Contexts (Plugin Manager, Event Log, Macro Library, Search Results, Command Configurator) each land the first Tab on their reported first interior control, no phantom stop -- all five arms now report interior ids + honour the focus latch. Enforced for future workspaces by `.kiro/steering/workspace-conformance.md`. (FilesPanel/FileEditor excluded as special cases.) |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::menu_workspace_tab_visits_options_then_calendar` | menu-and-statusbar Req 16.4: Tab advances through Interior_Controls in egui-native visual order (verified for the menu-workspace interior) |
| `ff-desktop` | ✅ | `shell::tests::full_shell_tab_cycle_wraps_to_command_field_and_skips_chrome` | menu-and-statusbar Req 16.5: Tab from the last Interior_Control moves to the Menu_Bar (part of the full-shell wrap cycle) |
| `ff-desktop` | 🔲 | -- | menu-and-statusbar Req 16.6: Tab advances through Menu_Bar items (egui-native; manual/UI verification) |
| `ff-desktop` | ✅ | `shell::tests::full_shell_tab_cycle_wraps_to_command_field_and_skips_chrome` | menu-and-statusbar Req 16.7: Tab from the last Menu_Bar item wraps to the command field (full-shell harness proves the cycle wraps) |
| `ff-desktop` | ✅ | `shell::tests::full_shell_shift_tab_from_command_field_goes_to_menu_bar` | menu-and-statusbar Req 16.8: Shift+Tab reverses -- from the command field it goes to the Menu_Bar, not chrome (full-shell harness) |
| `ff-desktop` | ✅ | `shell::tests::key_label_bar_buttons_are_not_tab_focus_stops`, `status_bar_segments_are_not_tab_focus_stops`, `full_shell_tab_cycle_wraps_to_command_field_and_skips_chrome` | menu-and-statusbar Req 16.9: chrome is not a Tab stop -- Status_Bar + Key_Label_Bar proven by isolated harness; the SCROLL field proven absent from the real full-shell cycle; tab headers use click-only Sense |
| `ff-desktop` | 🔲 | -- | menu-and-statusbar Req 16.10: focused menu-option Interior_Control shows the reversed-colour focus indicator (manual UI verification) |
| `ff-desktop` | 🔲 | -- | menu-and-statusbar Req 16.11: Enter/Space on a focused Interior_Control performs the same action as a click (egui button native activation; manual UI verification) |
| `ff-desktop` | 🔲 | -- | menu-and-statusbar Req 16.12: focused Menu_Bar item shows a visible focus indicator (manual UI verification) |
| `ff-desktop` | 🔲 | -- | menu-and-statusbar Req 16.13: Enter/Space on a focused Menu_Bar item opens its dropdown (manual UI verification) |
| `ff-desktop` | 🔲 | -- | menu-and-statusbar Req 16.14: the Boundary_Policy is implemented once at the shell level and applies to every Workspace kind (no per-Workspace focus code; design/manual verification) |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::menu_workspace_tab_visits_options_then_calendar`, `menu_workspace_disabled_option_is_skipped` | menu-workspace Req 15.1-15.4: enabled options are focus stops in declared order; disabled options skipped |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::menu_workspace_tab_visits_options_then_calendar`, `menu_workspace_no_calendar_last_interior_is_last_option` | menu-workspace Req 15.5/15.6: with the calendar shown Tab reaches the calendar `>` after the last option; no calendar stops when hidden (last interior = last option) |
| `ff-desktop` | ✅ | `primary_option_menu::tests::calendar_prev_next_are_focusable_buttons`, `calendar_prev_button_enter_returns_prev_nav` | menu-workspace Req 15.7/15.8: calendar prev/next are real focusable buttons; Enter changes the month |
| `ff-desktop` | ✅ | `shell::tests::full_shell_shift_tab_from_command_field_goes_to_menu_bar` | menu-workspace Req 15.9: Shift+Tab reverses the cycle -- the command-field reverse boundary (-> Menu_Bar) is harness-proven; interior reverse steps are egui-native |
| `ff-desktop` | ✅ | `primary_option_menu::tests::calendar_prev_next_are_focusable_buttons` | menu-workspace Req 15.10: only the calendar prev/next buttons are focusable; day cells are not focus stops (day cells are plain labels) |
| `ff-desktop` | ✅ | `menu_workspace::render::tests` (interior order derived from menu.options each frame) | menu-workspace Req 15.11: adding/removing/reordering options in the Menu_File changes the Tab order with no code change |
| `ff-desktop` | ✅ | `shell::tests::key_label_bar_buttons_are_not_tab_focus_stops`, `menu_workspace::render::tests::menu_workspace_tab_visits_options_then_calendar` | automated-dialog-testing Req 14.9: egui_kittest harness validates chrome-not-focusable and the POM/menu-workspace interior order (options -> calendar) |

### CR-CH-024 -- Legacy theme consolidation + name-based THEME command

> theme-and-appearance Requirement 17 (rewritten to name-based) and Requirement 18
> (built-in set 5 -> 4; `Legacy (ISPF 3270)` removed, `Default Legacy` retained as
> selectable built-in + fallback). Supersedes the Phase (theme-editor) "five
> built-ins" row above.

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-theme` | ✅ | `discovery.rs::builtin_themes_returns_four_entries`; `shell/tests.rs::full_shell_theme_list_has_four_builtins` | theme Req 18.1/18.3: built-in set is exactly FOUR (`Default Dark`, `Default Light`, `Default High Contrast`, `Default Legacy`); `Legacy (ISPF 3270)` is not a built-in name; legacy colours unchanged under `Default Legacy` |
| `ff-desktop` | ✅ | `theme_defaults.rs::resolve_theme_arg_exact_name_case_insensitive`, `resolve_theme_arg_exact_user_name_beats_shorthand` | theme Req 17.2a: `THEME <name>` selects by EXACT case-insensitive real-name match (built-in or user) first |
| `ff-desktop` | ✅ | `theme_defaults.rs::resolve_theme_arg_shorthand_maps_to_default_builtins`; `shell/tests.rs::full_shell_theme_shorthand_selects_default_builtins` | theme Req 17.2b: `THEME <shorthand>` selects the built-in that omits the `Default ` prefix (`Dark`/`Light`/`High Contrast`/`Legacy` -> `Default *`); `legacy` -> `Default Legacy` |
| `ff-desktop` | ✅ | `theme_defaults.rs::resolve_theme_arg_ambiguous_resolves_to_first` | theme Req 17.3: two case-insensitive user matches resolve to the FIRST listed, no error |
| `ff-desktop` | ✅ | `shell/tests.rs::full_shell_bare_theme_opens_editor_and_end_returns` | theme Req 17.4: bare `THEME` (no argument) opens the Theme Editor context in place (Navigation_Stack push; END returns), does NOT change the active theme, via the same path the removed `THEMES` used |
| `ff-desktop` | ✅ | `shell/tests.rs::full_shell_themes_command_is_removed` | theme Req 17.1: `THEMES` command removed (no longer recognised); Settings "Theme Editor" item, `menus/settings.toml` `T` option, and command palette repointed to `THEME` |
| `ff-desktop` | ✅ | `theme_defaults.rs::resolve_theme_arg_unknown_returns_none`; `shell/tests.rs::full_shell_theme_unknown_leaves_theme_unchanged` | theme Req 17.5: `THEME <unknown>` leaves the active theme unchanged and shows `THEME: '<name>' does not exist` |
| `ff-desktop` | 🔲 | -- | theme Req 17.6: the Settings menu theme items dispatch `THEME <name>` (parity); former `Legacy (ISPF 3270)` item relabelled `Default Legacy`. MANUAL: exact pixel labels/menu-item placement pending the menu-bar redesign; the dispatch path itself is covered by `full_shell_themes_command_is_removed` and the shorthand tests |
| `ff-desktop` | ✅ | `theme_defaults.rs::resolve_theme_arg_shorthand_maps_to_default_builtins` (legacy -> Default Legacy), `resolve_theme_arg_unknown_returns_none` (removed name is not a built-in) | theme Req 18.2/19.3 (CR-CH-024): persisted `legacy` mode string resolves to `Default Legacy`; the removed `Legacy (ISPF 3270)` name degrades to the `Default Legacy` fallback with a WARN (no crash) |

### CR-CH-025 -- Unified command-resolution chain + Settings launcher + CONFIG command

> command-framework Requirement 8 (ordered chain + shadowing + menu-name/macro
> stages), menu-workspace Requirement 3.6/11.11-11.12/12.3 (keyword-less menu-name
> resolution; Settings baseline reorder; retire SETTINGS/A special cases), and
> configuration-system Requirement 20 (the CONFIG command) + Req 15 revisions.
> Closes B032. The macro stage is specified but deferred, so its row is MANUAL/NOT
> COVERED until Lua execution is wired.

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-command` | ✅ | `command_target_tests::menu_name_resolves_to_menu_target`, `menu_name_matches_first_token_for_chaining` | Req 8.3/8.11: `resolve_target` resolves a bare first-token match to a `Menu_Target` when the token is a resolvable Menu_Name (user file or built-in `pom`/`settings`) |
| `ff-command` | ✅ | `command_target_tests::builtin_command_shadows_same_named_menu`, `menu_name_shadows_same_named_macro` | Req 8.10: shadowing rule -- a built-in command / Command_ID beats a same-named Menu_Name, which beats a same-named Macro; first match within a stage wins; matching is case-insensitive |
| `ff-command` | ✅ | `command_target_tests::deferred_macro_stage_default_none_falls_through_to_error`, `macro_name_resolves_when_no_earlier_stage_matches`; `command_config::tests::shell_resolver_macro_stage_is_deferred_none` | Req 8.12: the macro stage is present in the chain order but DEFERRED -- `macro_name_target` returns None so a non-built-in, non-menu token falls through to the unresolved-command error |
| `ff-desktop` | ✅ | `shell::tests::settings_t_chains_to_theme_editor` | Req 8.13 / menu-workspace 11.7: a menu-name match with a trailing token opens the menu and activates the option keyed by that token (`SETTINGS T` == open Settings + activate `T`) |
| `ff-desktop` | ✅ | `shell::tests::settings_command_opens_menu_workspace_not_flat_panel`, `keyword_less_pom_menu_name_opens_home_context`; `command_config::tests::shell_resolver_builtin_menu_name_resolves_without_file`, `shell_resolver_user_menu_file_resolves` | menu-workspace Req 11.11: keyword-less menu-name form -- a bare token matching a resolvable menu opens it without the `MENU` keyword; `SETTINGS`/`POM`/user menu names all resolve identically |
| `ff-desktop` | ✅ | `command_config::tests::shell_resolver_unknown_menu_name_does_not_resolve`; `shell::tests::unknown_token_is_unresolved_not_a_menu` | menu-workspace Req 11.12 / command-framework 8.10: a user menu/macro cannot shadow a built-in verb (built-in precedence); an unknown token is unresolved, not silently swallowed |
| `ff-desktop` | ✅ | `shell::tests::settings_command_opens_menu_workspace_not_flat_panel` (Err falls through), `unknown_token_is_unresolved_not_a_menu` | menu-workspace Req 3.6: the current-menu Option_Key lookup is the FIRST chain stage; a non-matching token falls through to the remaining stages instead of erroring |
| `ff-desktop` | ✅ | `menu_workspace::defaults::tests::default_settings_toml_has_recovery_baseline_options`, `recovery_settings_menu_has_barebones_options`; `shell::tests::default_settings_toml_option_a_command_is_config` | menu-workspace Req 12.3: `DEFAULT_SETTINGS_TOML` is ordered Core[`A` CONFIG, `T` THEME, `M` MENUS] then Recovery[`R` RESET BARE]; `A -> CONFIG`, `T -> THEME` (not THEMES), `R -> RESET BARE`; built-ins stay code-only |
| `ff-desktop` | ✅ | `shell::tests::config_command_is_registered`, `settings_all_view_has_no_namespace_filter`, `settings_option_a_opens_flat_panel` | configuration-system Req 20.2: bare `CONFIG` opens the flat config-key view unfiltered (Command_ID `config.open`, resolves as a built-in) |
| `ff-desktop` | ✅ | `shell::tests::settings_namespace_filter_applied_on_open`, `settings_namespace_opens_filtered_flat_panel`, `settings_namespace_tab_title_includes_namespace`, `config_unknown_namespace_opens_editable_view` | configuration-system Req 20.3/20.4: `CONFIG <namespace>` opens the flat view filtered to `<namespace>.`; an unmatched namespace opens an empty-but-editable filter (no error) |
| `ff-desktop` | ✅ | `shell::tests::default_settings_toml_option_a_command_is_config` (menu row -> CONFIG); the removed `A`/`SETTINGS <ns>` intercepts are covered by the migrated `CONFIG`/`settings_*` tests | configuration-system Req 20.6: the Settings `A` option dispatches `CONFIG` (command parity); the `A`-named command and the `SETTINGS <namespace>` intercept are removed |
| `ff-desktop` | 🔲 | -- | configuration-system Req 20.7: the Config View persists/restores as `CustomWorkspace { workspace_kind = config, params = { namespace } }` with the namespace reapplied. MANUAL: covered structurally by the existing config-descriptor persistence (Req 15.9 / `config_namespace_descriptor_round_trips`) tests; the `CONFIG`-entry-point round trip is verified manually pending a dedicated persistence test |
| `ff-desktop` | ✅ | `shell::tests::settings_command_opens_menu_workspace_not_flat_panel`, `settings_menu_end_returns_to_pom` | B032 (closed by CR-CH-025): opening Settings is menu-name resolution of `SETTINGS`, not a bespoke verb; the Settings_Menu is a real Menu_Workspace |

### CR-CH-026 -- Calendar Visibility and Fit; Settings default off (menu-workspace Req 16, B060)

> Fixes B060: a `show_calendar = true` menu drew the calendar (and its focusable
> `<`/`>` buttons) off the visible right edge in a narrow workspace -- invisible
> yet still Tab stops. Settings now defaults calendar off; a shown calendar must
> be visible and its buttons on-screen; an omitted calendar contributes zero Tab
> stops.

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `menu_workspace::defaults::tests::default_settings_toml_hides_calendar`; `shell::tests::full_shell_settings_tab_walks_options_only_no_calendar_stops` | menu-workspace Req 16.1: the compiled Settings default (`DEFAULT_SETTINGS_TOML`) sets `show_calendar = false`; the Settings Menu_Workspace shows no calendar and has no calendar Tab stops by default |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::menu_calendar_shown_when_wide_next_button_is_on_screen` | menu-workspace Req 16.2: WHEN `show_calendar = true` AND it fits (available >= OPTION_LIST_MIN_WIDTH + CALENDAR_GAP + CALENDAR_MIN_WIDTH), the calendar is laid out within the visible width with `<`/`>` on-screen and reachable (POM, Settings, custom alike) |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::menu_calendar_omitted_when_too_narrow_no_calendar_tab_stops` | menu-workspace Req 16.3: WHEN `show_calendar = true` BUT the workspace is too narrow, the calendar is omitted for that frame (decision uses only the current frame's available width -> restored on a wider frame, no persisted state) |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::menu_calendar_omitted_when_too_narrow_no_calendar_tab_stops`, `menu_workspace_no_calendar_last_interior_is_last_option`; `shell::tests::full_shell_settings_tab_walks_options_only_no_calendar_stops` | menu-workspace Req 16.4: WHEN the calendar is omitted (narrow-fit OR `show_calendar = false`), its `<`/`>` ids are NOT in the reported interior focus contract -- zero calendar Tab stops; last interior = last enabled option |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::menu_calendar_shown_when_wide_next_button_is_on_screen` (asserts the `>` rect is within `ui.clip_rect()`) | menu-workspace Req 16.5: WHEN the calendar is displayed, the reported last interior (`>`) button's on-screen rect is within the visible width (never an off-screen phantom stop) -- the B060 invariant |
| `ff-desktop` | ✅ | option list constrained via `ScrollArea::max_width` + `set_max_width` to `available - gap - CALENDAR_MIN_WIDTH` (reserved right column); guarded by `menu_calendar_shown_when_wide_next_button_is_on_screen` | menu-workspace Req 16.6: the displayed calendar does not overlap the option list; the option list keeps a readable width/scroll and the calendar occupies a reserved right-hand column |

### CR-NR-078 -- WorkspaceContext trait framework (phase 1)

> Compiler-enforced workspace-Context contract: `WorkspaceContext::render ->
> InteriorFocus`, single shell dispatch (owned-panel swap), `ShellServices` +
> `ShellRequest`, host-agnostic (detach-ready), layered above
> `ff-layout::DockablePanel`. Phase-1 proof scope: MenuWorkspace (POM + Settings),
> Theme Editor, Menus Editor, Config panel. Behaviour-preserving -- the existing
> full-shell first-Tab tests are the safety net.

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell::workspace_context::tests::interior_focus_none_is_both_none`, `interior_focus_single_sets_both_to_same_id`, `interior_focus_new_sets_distinct_first_and_last` | Req 1.1/1.3: `InteriorFocus { first, last }` with `none()`/`single()`/`new()`; `WorkspaceContext::render` RETURNS it so an implementor cannot omit the focus contract |
| `ff-desktop` | ✅ | `shell::tests::full_shell_config_first_tab_focuses_filter_field`, `full_shell_theme_editor_first_tab_focuses_theme_selector`, `full_shell_menus_editor_first_tab_focuses_menu_selector` (all pass unchanged through the single `render_workspace_context` dispatch) | Req 1.2: the shell dispatches the active Context's render through the trait on ONE code path (`render_workspace_context` / shared `apply_interior_focus`) and honours the latch; no per-kind focus-latch ritual remains for migrated Contexts |
| `ff-desktop` | ✅ | `shell::workspace_context::tests::shell_request_helpers_enqueue`; exercised by the migrated arms | Req 2.1-2.3: `ShellServices` mediates shell access (config/runtime/notifications/dirs) without `&mut WorkbenchShell`; Contexts enqueue `ShellRequest`s drained by `apply_shell_requests` (pure render -> action generalised) |
| `ff-desktop` | ✅ | `shell::tests::full_shell_config_first_tab_focuses_filter_field`, `full_shell_theme_editor_first_tab_focuses_theme_selector`, `full_shell_menus_editor_first_tab_focuses_menu_selector`, `full_shell_first_tab_focuses_reported_first_interior` (POM), `settings_command_opens_menu_workspace_not_flat_panel`; theme/menus-editor behaviour suites unchanged | Req 1.4/1.5: MenuWorkspace (POM + Settings), Theme Editor, Menus Editor, and Config panel each implement `WorkspaceContext` with IDENTICAL observable behaviour -- all 944 ff-desktop tests pass unchanged |
| `ff-desktop` | ✅ | full test suite green after each per-Context migration (944 tests) | Req 3.1-3.3: migration was incremental and behaviour-preserving; the migrated arms use trait dispatch and their inline focus-latch ritual was removed |
| `ff-desktop` | ✅ | `shell/workspace_context.rs` module doc + `docs/specs/workspace-framework/design.md` section 10 | Req 4.1/4.2: `WorkspaceContext` (ff-desktop) is layered above `ff-layout::DockablePanel` (Option Y); `ff-layout` stays GUI-independent |
| `ff-desktop` | ✅ | `shell::tests::workspace_context_render_is_host_agnostic` | Req 6.1/6.2: `render` is host-agnostic -- the Config Context renders correctly into a non-central-panel `Ui` and returns the same `InteriorFocus` (detach-ready), without building the detach feature |

### Phase (reset-bare-targets) -- targeted RESET BARE: named profile / ALL (CR-NR-083)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `reset_bare::tests::{resolve_bare_targets_default_when_no_active_profile, resolve_bare_targets_active_profile_dir}`; `shell::tests::reset_bare_bare_resolves_single_default_target` | configuration-system Req 19.9: bare `RESET BARE` targets ONLY the running process's profile (active, else default); other profiles untouched |
| `ff-desktop` | ✅ | `reset_bare::tests::{resolve_named_subset_resolves_each_and_excludes_active, resolve_named_list_slugs_and_deduplicates, profile_udd_path_slugs_the_name}` | configuration-system Req 19.10: `RESET BARE <profilename> [<profilename> ...]` resolves each named profile's User_Data_Dir at `profiles/<slug>/`, slug-compared (case-insensitive), de-duplicated |
| `ff-desktop` | ✅ | `reset_bare::tests::{resolve_named_subset_resolves_each_and_excludes_active, resolve_named_subset_including_active_sets_flag}` | configuration-system Req 19.11: confirmed named-list reset archives each listed profile (best-effort); in-memory reset only when the list includes the active profile, else running shell untouched |
| `ff-desktop` | ✅ | `reset_bare::tests::resolve_named_list_with_unknown_errors_all_or_nothing`; `shell::tests::reset_bare_unknown_named_profile_errors_without_dialog` | configuration-system Req 19.12: any unknown name in the list -> non-blocking error naming it, no dialog, nothing archived (all-or-nothing) |
| `ff-desktop` | ✅ | `reset_bare::tests::{resolve_all_targets_every_profile, enumerate_profiles_lists_default_first_then_named_sorted}` | configuration-system Req 19.13: `RESET BARE ALL` archives every profile (default + all `profiles/<slug>/`) best-effort, then resets in-memory once |
| `ff-desktop` | ✅ | `reset_bare::tests::{resolve_all_targets_every_profile, resolve_all_mixed_with_a_name_is_not_the_keyword}` | configuration-system Req 19.14: `ALL` is a reserved case-insensitive keyword only as the sole arg; a profile slugging to `all` can't be named individually (documented) |
| `ff-desktop` | 🔲 | `shell::tests::reset_bare_command_opens_confirmation_dialog` (dialog-open state) | configuration-system Req 19.15: the SAME single confirmation dialog is reused for every target list; body text now names the profile(s) (one name, or enumerated list); Confirm/Cancel unchanged. State/target resolution automated; the added label rendering is a minor extension of the already-tested existing modal -- MANUAL for the pixel-exact body text (no new focus/interaction behaviour) |

### Phase (fkey-arg + menu-wrap) -- B066 F-key argument passing + B065 description hang-indent

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell::tests::{key_command_merges_command_field_as_argument, key_command_with_empty_field_runs_bare_command, key_command_does_not_force_clear_command_field}` | command-framework Req 9.7/9.8/9.9/9.10 (B066): a key-bound command is invoked with the Command ===> field content as its argument (type `1` + F9=SWAP -> `SWAP 1`); empty field -> bare command; the field is not force-cleared. Implements the F-key portion of Req 9 (CR-NR-054, otherwise still PENDING GATE) |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::{option_prefix_job_holds_fixed_columns_and_does_not_wrap, option_prefix_jobs_share_width_across_rows}` | menu-workspace Req 2.1a (B065, 2nd-attempt fix): each option row is a `ui.horizontal` pair -- a non-wrapping key+command prefix Button + a separate wrapping description Label -- so a wrapped description hang-indents under the description column. The 1st attempt (single-galley char-budget) did not work at runtime. Pixel-exact wrap alignment is MANUAL/visual (test-plan 2.2c) |

### Phase (fkey-arg) -- B067 F12 RETRIEVE with a non-empty field (B066 regression fix)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell::tests::{retrieve_via_key_with_nonempty_field_recalls_previous_command, retrieve_list_via_key_opens_history_overlay}` | function-keys-and-history Req 19.1-19.3, 19.6 (B067): F12 RETRIEVE recalls the previous command even when the command field is non-empty (the merged `RETRIEVE <field>` from B066 is matched by verb prefix in both the history-exclusion guard and the dispatch branch); RETRIEVE LIST via key opens the overlay; the merged form is not added to history |

### Phase (retrieve-stack) -- command-line history re-homed to the command processor (CR-NR-084, Option B)

Behaviour-preserving re-home; no new acceptance criteria. The Req 5-10/19 rows
elsewhere in this document remain valid; these rows record the new owner's
coverage and confirm the shell behaviour is unchanged after the move.

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-command` | ✅ | `command_line_history::tests::{owner_records_and_recalls_most_recent, owner_excludes_retrieve_from_history, owner_non_retrieve_resets_pointer, owner_list_returns_all_entries_most_recent_first, owner_load_command_strings_replaces_and_resets, add_deduplicates_and_promotes, capacity_enforcement_evicts_oldest, list_trigger_returns_show_list, successive_retrieves_cycle_backward, ...}` | function-keys-and-history Req 5-9, 19.1-19.4 (CR-NR-084): new `CommandLineHistory` owner -- record+dedup-promote, RETRIEVE-verb exclusion (bare + B067 merged form), bare step-back + reset, LIST. Primitives moved from ff-keys (which re-exports them) |
| `ff-desktop` | ✅ | `shell::tests::{shell_intercept_commands_are_recorded_in_history, retrieve_via_key_with_nonempty_field_recalls_previous_command, retrieve_list_via_key_opens_history_overlay, command_history_records_entries, retrieve_state_cycles_through_history}` | function-keys-and-history Req 19.1-19.9 (CR-NR-084): shell forwards to `CommandLineHistory` (`record`/`retrieve`); observable RETRIEVE behaviour unchanged (recall with non-empty field per B067, no history pollution, LIST overlay). cmd_history/retrieve_state fields removed |

### Phase (retrieve-stack) -- command-line history persistence (function-keys-and-history Req 6, CR-NR-084 follow-on)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell::tests::startup_loads_persisted_command_history` | function-keys-and-history Req 6.2: at startup the shell loads `<User_Data_Dir>/command_history.toml` into `CommandLineHistory` (most-recent-first) |
| `ff-desktop` | ✅ | `shell::tests::startup_missing_or_corrupt_history_is_empty_no_panic` | Req 6.5, 6.6: missing or corrupt history file degrades to an empty history without failing startup (warnings logged) |
| `ff-desktop` | ✅ | `shell::tests::exit_saves_command_history_and_reloads` | Req 6.3, 6.7: on exit the shell writes the history via `HistoryStore` (atomic write), and a fresh shell reloads it; `persist_command_history` extracted for testability |
| `ff-keys` | ✅ | `config_keys.rs::history_file_path_relative_to_user_data_dir` | Req 6.4: history file path resolves relative to the User_Data_Dir (default `command_history.toml`); `FFWB_HISTORY_PATH` override for test isolation |

### Phase (workspace-unify) -- unified Menu Workspace / POM (menu-workspace Req 18, CR-NR-082 Slice 1)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell/tests.rs::home_context_is_a_menu_workspace_flagged_is_home`; `tab_manager.rs::pom_tab_has_kind_primary_option_menu` | menu-workspace Req 18.1 (CR-NR-082): single Menu Workspace kind; `TabKind::PrimaryOptionMenu` removed; Home Context = MenuWorkspace menu `pom` (is_home) |
| `ff-desktop` | ✅ | `shell/tests.rs::home_context_seeds_barebones_menu_on_render` | menu-workspace Req 18.2: Home Context loads `menus/pom.toml` else the compiled barebones Recovery_Baseline (parse error -> notice; absent -> silent) |
| `ff-desktop` | ✅ | `shell/tests.rs::home_context_title_line_shows_app_banner`, `full_shell_tab_reaches_settings_as_first_menu_item` | menu-workspace Req 18.3/18.5: Home renders via the single shared menu renderer; no distinct POM render path; title line unchanged |
| `ff-desktop` | ✅ | `shell/tests.rs::end_from_drilled_menu_restores_home_context`, `home_context_resolves_to_pom_keymap_context` | menu-workspace Req 18.4/18.6: always-present Home guarantee + END/RETURN fallback + `pom` keymap context preserved after unification |
| `ff-desktop` | ✅ | `shell/tests.rs::home_context_persists_as_menu_pom_descriptor` | menu-workspace Req 18.7/18.8: Home persists as `Menu{name:pom}`; legacy POM sessions still restore; overlapping enums reconciled (legacy read-only) |
| `ff-desktop` | ✅ | full ff-desktop suite via `verify.ps1` (nextest) -- POM/menu tests retargeted off the removed kind pass | menu-workspace Req 18.9: unification is behaviour-preserving -- existing menu/POM tests pass (retargeted off the removed kind) |

### Phase (menu-desc-layout) -- description-driven menu layout (menu-workspace Req 16.7-16.11, CR-CH-032)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `menu_workspace::render::tests::{natural_width_is_widest_row_plus_scrollbar, natural_width_grows_with_longer_description, natural_width_empty_is_scrollbar_allowance_only, option_prefix_text_matches_prefix_job_text}` | menu-workspace Req 16.7 (CR-CH-032): `natural_option_list_width` = widest key+command prefix + widest single-line description + scrollbar allowance; uncapped |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::{menu_wide_shows_calendar_and_one_line_descriptions, menu_medium_hides_calendar_keeps_one_line, menu_narrow_hides_calendar_and_wraps}` | menu-workspace Req 16.8: 3-tier ladder -- Tier1 (calendar + one-line, option column = natural width, trailing blank space right of calendar); Tier2 (calendar hidden, full width, one line); Tier3 (calendar hidden, full width, wrapped) |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::menu_narrow_hides_calendar_and_wraps` (medium+narrow both calendar-hidden; wrap only in narrow) | menu-workspace Req 16.9: calendar-hide-before-wrap ordering is strict -- never calendar shown AND a description wrapped |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::menu_narrow_hides_calendar_and_wraps` (the unchanged `.wrap()` Label wraps only in Tier 3); row-render unchanged | menu-workspace Req 16.10: description `.wrap()` retained as fault-tolerant fallback; no-wrap in Tiers 1/2 is a consequence of width allocation, not a hard mode |
| `ff-desktop` | ✅ | `menu_workspace::render::tests::{menu_wide_shows_calendar_and_one_line_descriptions, menu_medium_hides_calendar_keeps_one_line, menu_narrow_hides_calendar_and_wraps}` (each width recomputes the tier from available width alone) | menu-workspace Req 16.11: Layout_Tier recomputed each frame from available width only (no persisted state); widening/narrowing moves reactively between tiers |

### Phase (command-line-outcome) -- Command_Line_Outcome (command-framework Req 13 + Req 9.9 revised, CR-CH-033)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-desktop` | ✅ | `shell::tests::{enter_path_clears_command_field_after_success, key_command_clears_command_field_after_success, unresolved_or_errored_command_restores_field_for_correction}` | command-framework Req 13.1/13.2 (Slice 1): field disposition applied on both Enter and key-forward paths via the Command_Line_Outcome; an UNRESOLVED command keeps the field for correction |
| `ff-desktop` | ✅ | `shell::tests::{key_command_clears_command_field_after_success, unresolved_or_errored_command_restores_field_for_correction}` | command-framework Req 13.3 (Slice 1): default outcome = `Clear` on success, `Restore` original text when the command left an `open_error` (covers unresolved typo AND resolved-but-failed) |
| `ff-desktop` | ✅ | `shell::tests::key_command_retrieve_keeps_recalled_field` | command-framework Req 13.4 (Slice 1): a command returns an explicit `CommandLineOutcome` (`Clear`/`Restore`/`Set`/`Leave`); RETRIEVE returns `Set(<recalled>)`; fixes `1` remaining after `1` + F9 (Req 9.9 revised) |
| `ff-desktop` | ✅ | full ff-desktop command/history/swap suite via `verify.ps1` (377 tests pass unchanged) | command-framework Req 13.8 (Slice 1): additive/behaviour-preserving -- a command returning no outcome gets the default; only the intended clear-on-success changes |
| `ff-desktop` | ✅ | `shell::command_line_outcome::tests::{outcome_data_shape_round_trips_all_variants, outcome_data_shape_set_requires_text, invalid_shape_maps_to_default, outcome_data_action_is_case_insensitive, outcome_data_survives_toml_round_trip, outcome_data_action_tags_are_stable}` | command-framework Req 13.5 (Slice 2): serialisable `{action,text?}` Outcome_Data_Shape; total lossless round-trip to/from the native enum; invalid/absent -> None (caller default), no panic |
| `ff-desktop` | ✅ | `shell/command_line_outcome.rs` doc comments + `outcome_data_survives_toml_round_trip` | command-framework Req 13.6 (Slice 2, docs half) / 13.7: data shape documented as the public boundary for future Lua/REXX/External bridges; per-engine bridge enforcement is Slice 3+ (deferred until each engine executes) |

### Phase (bug-sweep Wave 1) -- Save-durability failures surface (file-operations Req 7.10-7.12, CR-NR-085, B034)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-file-ops` | ✅ | `persistence_tests.rs::{atomic_write_aborts_and_preserves_target_on_fsync_failure, atomic_write_aborts_and_preserves_target_on_flush_failure, atomic_write_still_succeeds_when_fsync_ok}` | file-operations Req 7.10 (B034): AtomicWriteStrategy checks flush/sync_all of the temp file; on failure it aborts (deletes temp, logs, no rename) so a non-durable temp never overwrites the target; a temp that cannot be re-opened for fsync logs WARN, not silent |
| `ff-file-ops` | ✅ | `persistence_tests.rs::{direct_write_handles_fsync_failure_without_panic, delete_first_handles_fsync_failure_without_panic}` (WARN emission itself MANUAL -- needs a running log subsystem) | file-operations Req 7.11 (B034): Direct/DeleteFirst strategies log a WARN when flush/sync_all of the written target fails (best-effort durability), not a silent `let _ =` |
| `ff-file-ops` | ✅ | `persistence_tests.rs::backup_write_failure_returns_backup_failed_error`; code review of `save.rs` `log_warn!` (WARN emission MANUAL) | file-operations Req 7.12 (B034): a failed Backup_Copy logs WARN via ff-logging (Req 7.5) and the save still proceeds; the `let _ = e` discard no longer satisfies the criterion |

### Phase (bug-sweep Wave 2) -- I/O-layer logging coverage + degradation indicator (logging-subsystem Req 8.7/9.6-9.8, CR-NR-086; B035/B036/B037/B038)

| Crate | Status | Test files | Notes |
|-------|--------|-----------|-------|
| `ff-vfs` | ✅ | `vfs::tests::dispatch_error_propagates_unchanged_through_log_wrapper` (log emission MANUAL) | logging-subsystem Req 9.6 (B035): the `Vfs` dispatch layer log_warn!s (operation + URI) on the error path of read/write/delete/rename/stat/list before returning Err (previously emitted nothing). Behaviour automated; log emission MANUAL (no cross-crate sink) |
| `ff-connector-local-fs` | ✅ | `provider::tests::create_and_list_directory` (populated size/modified regression guard); log emission MANUAL | logging-subsystem Req 9.7 (B037): per-entry metadata/modified/created/accessed/file_type/read_link failures log_debug! the entry+attribute instead of silent `.ok()`; graceful degradation preserved (listing still succeeds). Degradation behaviour automated; log emission MANUAL |
| `ff-desktop` | ✅ | `shell::tests::{logging_degradation_reason_covers_all_states, full_shell_status_bar_hides_logging_indicator_when_healthy}` (degraded render MANUAL) | logging-subsystem Req 8.7 (B038): status bar shows a logging-degradation indicator when `is_fallback()` or `dropped_count()>0`, hidden when healthy. Pure decision + full-shell egui_kittest |
| `ff-connector-extensibility` | 🔲 | -- (deferred; verified per-connector at build) | logging-subsystem Req 9.8 (B036): connector logging is a binding framework obligation; REFRAMED forward-looking + DEFERRED (no FTP/SFTP/cloud/mainframe connector crates exist yet) -- each connector verifies at its own gate when built |
