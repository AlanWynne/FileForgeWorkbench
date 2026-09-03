# Implementation Plan: Menu & Status Bar (`ff-menu`)

## Overview

This plan covers the complete implementation of the `ff-menu` crate — the menu bar, context menus, status bar, and primary command field for FileForgeWorkbench. The menu system provides hierarchical command invocation through standard menus, right-click context menus, and a configurable multi-segment status bar displaying real-time editor and workbench state. All menu actions route through `ff-command` dispatch — no menu item directly mutates application state.

This is a **Wave 6 (UI and Rendering)** sub-project. It depends on `ff-command` (command framework), `ff-core` (platform core), `ff-layout` (layout and docking), `ff-plugin` (plugin architecture), and `ff-config` (configuration system).

---

## Tasks

- [x] 1. Crate scaffolding and module structure
  - [x] 1.1 Create `crates/ff-menu/Cargo.toml` with dependencies (ff-command, ff-core, ff-logging, ff-config, ff-plugin, egui, thiserror, serde, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-menu/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `menu_bar.rs`, `menu_item.rs`, `menu_model.rs`, `context_menu.rs`, `status_bar.rs`, `status_segment.rs`, `command_field.rs`, `recent_files.rs`, `keyboard_nav.rs`, `extensibility.rs`, `error.rs`
  - [x] 1.4 Add `ff-menu` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [x] 2. Menu model types and data structures
  - [x] 2.1 Define `MenuId` newtype for unique menu identification
  - [x] 2.2 Define `MenuItem` enum with variants: Action(CommandBinding), Separator, Submenu(SubMenu), Toggle(ToggleBinding)
  - [x] 2.3 Define `CommandBinding` struct with fields: command_id, display_name, shortcut_text (Option), icon (Option), access_key (Option<char>)
  - [x] 2.4 Define `ToggleBinding` struct extending CommandBinding with a checked-state predicate
  - [x] 2.5 Define `SubMenu` struct with fields: label, items (Vec<MenuItem>), access_key (Option<char>)
  - [x] 2.6 Define `MenuDefinition` struct representing a complete top-level menu (label, items, access_key)
  - [x] 2.7 Write unit tests for menu model construction and traversal
  - Covers: Requirement 1 (AC 1.1-1.8), Requirement 2 (AC 2.1)

- [x] 3. Menu-command binding and predicate evaluation
  - [x] 3.1 Implement `MenuCommandBinding` struct linking a MenuItem to a Command_ID in the command registry
  - [x] 3.2 Implement shortcut text resolution — query ShortcutRegistry for bound command's shortcut and format as display text
  - [x] 3.3 Implement enabled-state evaluation — query command's EnabledPredicate against current ExecutionContext
  - [x] 3.4 Implement visibility evaluation — query command's VisibilityPredicate against current ExecutionContext
  - [x] 3.5 Implement menu item activation — call `execute_command(command_id, params)` on the command dispatcher when item is clicked
  - [x] 3.6 Write unit tests for binding resolution, disabled rendering, hidden items, and dispatch invocation
  - Covers: Requirement 2 (AC 2.1-2.4, 2.10)

- [x] 4. Default menu bar definitions
  - [x] 4.1 Define File menu structure: New (`file.new`), Open (`file.open`), Open Recent (submenu), separator, Save (`file.save`), Save As (`file.save_as`), separator, Close (`file.close`), separator, Exit (`workbench.exit`)
  - [x] 4.2 Define Edit menu structure: Undo (`edit.undo`), Redo (`edit.redo`), separator, Cut (`edit.cut`), Copy (`edit.copy`), Paste (`edit.paste`), separator, Select All (`edit.select_all`)
  - [x] 4.3 Define Search menu structure: Find (`find.show`), Find Next (`find.next`), Find Previous (`find.previous`), separator, Change (`find.replace_show`), Go to Line (`navigate.goto_line`)
  - [x] 4.4 Define View menu structure: Zoom In (`view.zoom_in`), Zoom Out (`view.zoom_out`), Reset Zoom (`view.zoom_reset`), separator, Word Wrap toggle (`view.word_wrap`), Show Whitespace toggle (`view.show_whitespace`), Show Line Numbers toggle (`view.show_line_numbers`), separator, Theme submenu
  - [x] 4.5 Define Help menu structure: Help Topics (`help.topics`), Keyboard Shortcuts (`help.shortcuts`), About (`help.about`)
  - [x] 4.6 Define Tools menu placeholder (between View and Help) for plugin contributions
  - [x] 4.7 Write unit tests verifying default menu structure contains all required items in correct order
  - Covers: Requirement 1 (AC 1.2-1.7), Requirement 2 (AC 2.5-2.9), Requirement 10 (AC 10.5)

- [x] 5. Menu bar rendering (egui)
  - [x] 5.1 Implement `MenuBarWidget` struct with `render(&self, ui: &mut egui::Ui)` method
  - [x] 5.2 Implement top-level menu heading rendering — each heading opens dropdown on click
  - [x] 5.3 Implement dropdown submenu rendering with items, separators, and nested submenus
  - [x] 5.4 Implement disabled-item greyed-out rendering style
  - [x] 5.5 Implement hidden-item filtering (skip items whose visibility predicate returns false)
  - [x] 5.6 Implement shortcut text display right-aligned in menu entries
  - [x] 5.7 Implement toggle-item checkmark rendering for active toggles
  - [x] 5.8 Write unit tests for menu rendering state logic (mocked egui context)
  - Covers: Requirement 1 (AC 1.1, 1.8), Requirement 2 (AC 2.2-2.4)

- [x] 6. Recent files management
  - [x] 6.1 Define `RecentFilesManager` struct with bounded list storage and configuration reference
  - [x] 6.2 Implement `add_or_promote(path: &Path)` — adds path to top, removes duplicate, trims to max
  - [x] 6.3 Implement configurable max entries from `menu.recent_files_max` (default 10, max 50)
  - [x] 6.4 Implement list query method returning ordered entries for submenu rendering
  - [x] 6.5 Implement stale-path detection: render non-existent paths with visual indication
  - [x] 6.6 Implement stale-path removal after failed open attempt
  - [x] 6.7 Implement `clear_all()` method for the "Clear Recent Files" menu action
  - [x] 6.8 Implement persistence: serialize to/from JSON in workbench data directory
  - [x] 6.9 Implement session-load on startup and session-save on shutdown
  - [x] 6.10 Write unit tests for add/promote, max clamping, stale detection, clear, and persistence round-trip
  - Covers: Requirement 3 (AC 3.1-3.7)

- [x] 7. Context menu system
  - [x] 7.1 Define `ContextMenuDefinition` struct with target context type and items
  - [x] 7.2 Define `ContextType` enum: EditorArea, TabHeader, PanelHeader, FileTreeNode
  - [x] 7.3 Implement editor-area context menu: Cut, Copy, Paste, Select All, separator, Find, Change
  - [x] 7.4 Implement tab-header context menu: Close, Close Others, Close All, Close to the Right, separator, Copy Path, Reveal in File Tree
  - [x] 7.5 Implement context menu rendering via egui popup
  - [x] 7.6 Implement command binding for context menu items (same predicate evaluation as menu bar items)
  - [x] 7.7 Implement plugin extension point for context menu contributions (per context type)
  - [x] 7.8 Write unit tests for context menu construction, item enablement, and plugin contribution
  - Covers: Requirement 4 (AC 4.1-4.5)

- [x] 8. Status bar layout and rendering
  - [x] 8.1 Define `StatusBar` struct with segment collection and layout configuration
  - [x] 8.2 Define `StatusSegment` struct with fields: id (String), content_provider, alignment, min_width, priority
  - [x] 8.3 Define `SegmentAlignment` enum: Left, Center, Right
  - [x] 8.4 Implement `StatusBar::render(&self, ui: &mut egui::Ui)` rendering segments in order by alignment and priority
  - [x] 8.5 Implement segment ID validation: 1-64 ASCII alphanumeric or underscore characters
  - [x] 8.6 Implement fixed-height rendering (single text line at current UI font size)
  - [x] 8.7 Implement full-width spanning at bottom of Primary_Window
  - [x] 8.8 Implement placeholder display when no editor tab is active (mode="--", line/col="--/--", encoding="--")
  - [x] 8.9 Write unit tests for layout ordering, ID validation, and placeholder behavior
  - Covers: Requirement 5 (AC 5.1-5.7)

- [x] 9. Status bar core segments — mode and state
  - [x] 9.1 Implement editor mode segment displaying "Browse", "Edit", or "View"
  - [x] 9.2 Implement mode segment reactive update on Editor_Mode change
  - [x] 9.3 Implement insert/overstrike segment displaying "INS" or "OVR"
  - [x] 9.4 Implement insert/overstrike reactive update on state toggle
  - [x] 9.5 Implement modified indicator segment: "●" when document has unsaved changes, empty when clean
  - [x] 9.6 Implement modified indicator clearing on successful save
  - [x] 9.7 Write unit tests for mode display values, state transitions, and modified indicator lifecycle
  - Covers: Requirement 6 (AC 6.1-6.6)

- [x] 10. Status bar core segments — position and file info
  - [x] 10.1 Implement line/column segment displaying "Ln {line}, Col {col}" (1-based)
  - [x] 10.2 Implement line/column reactive update on cursor movement (within one frame)
  - [x] 10.3 Implement file encoding segment displaying detected encoding string
  - [x] 10.4 Implement total line count segment displaying "{count} lines"
  - [x] 10.5 Implement line count reactive update on document content change
  - [x] 10.6 Implement all-segment update on active tab switch (within one frame)
  - [x] 10.7 Write unit tests for position formatting, encoding display, line count updates, and tab-switch propagation
  - Covers: Requirement 7 (AC 7.1-7.6)

- [x] 11. Status bar extensibility
  - [x] 11.1 Define `StatusSegmentProvider` trait with methods: `segment_id()`, `render()`, `alignment()`, `priority()`
  - [x] 11.2 Implement plugin segment registration in the StatusBar
  - [x] 11.3 Implement segment layout update on plugin registration (insert according to alignment and priority)
  - [x] 11.4 Implement segment removal on plugin unload
  - [x] 11.5 Implement duplicate segment ID rejection with WARN-level log
  - [x] 11.6 Implement configurable segment layout from `statusbar.segments` config table (hide, reorder, resize)
  - [x] 11.7 Write unit tests for provider registration, duplicate rejection, unload cleanup, and config-driven layout
  - Covers: Requirement 8 (AC 8.1-8.6)

- [x] 12. Primary command field
  - [x] 12.1 Implement `PrimaryCommandField` widget with "Command ===>" label and expanding text input
  - [x] 12.2 Implement horizontal expansion to fill available width between label and right panel edge
  - [x] 12.3 Implement Enter key handling: submit text to CommandEngine for parsing and dispatch
  - [x] 12.4 Implement field clearing on successful command dispatch
  - [x] 12.5 Implement error display in status bar for unrecognized commands (field content preserved)
  - [x] 12.6 Implement command history recall: Up Arrow cycles backwards, Down Arrow cycles forwards
  - [x] 12.7 Implement Down Arrow focus-transfer to editor when field is empty
  - [x] 12.8 Write unit tests for submit/clear flow, error preservation, history navigation, and focus transfer
  - Covers: Requirement 9 (AC 9.1-9.7)

- [x] 13. Menu extensibility (plugin contributions)
  - [x] 13.1 Define `MenuContribution` descriptor struct: target_menu_path, command_id, position (Before/After/End), separator spec
  - [x] 13.2 Implement plugin menu item insertion at specified position within target menu
  - [x] 13.3 Implement new top-level menu creation for paths that don't exist (inserted before Help)
  - [x] 13.4 Implement menu item removal on plugin unload, collapsing empty top-level menus
  - [x] 13.5 Implement plugin menu items respecting same enabled/visibility/shortcut rules as built-in items
  - [x] 13.6 Write unit tests for contribution insertion, new menu creation, unload cleanup, and predicate respect
  - Covers: Requirement 10 (AC 10.1-10.6)

- [x] 14. Keyboard navigation
  - [x] 14.1 Implement Alt+letter access key activation for top-level menus (e.g., Alt+F opens File)
  - [x] 14.2 Implement Up/Down Arrow navigation within open dropdown
  - [x] 14.3 Implement Right Arrow to open submenus, Left Arrow to close submenus
  - [x] 14.4 Implement Left/Right Arrow to jump between top-level menus while dropdown is open
  - [x] 14.5 Implement Escape to close open menu and return focus to previously focused element
  - [x] 14.6 Implement per-item access key activation (underlined character) within open menu
  - [x] 14.7 Implement F10 key activation of first top-level menu
  - [x] 14.8 Write unit tests for keyboard navigation state machine transitions
  - Covers: Requirement 11 (AC 11.1-11.6)

- [x] 15. Error types
  - [x] 15.1 Define `MenuError` enum with variants: DuplicateSegmentId, InvalidSegmentId, CommandNotFound, PluginContributionError, RecentFilesIoError
  - [x] 15.2 Implement `Display` and `thiserror::Error` derives with descriptive messages following `[ff-menu] operation: description` format
  - [x] 15.3 Write unit tests for error display output
  - Covers: All requirements (error paths)

- [x] 16. Integration wiring
  - [x] 16.1 Wire MenuBar into Primary_Window layout (below title bar, above command area)
  - [x] 16.2 Wire StatusBar into Primary_Window layout (bottom of window, full width)
  - [x] 16.3 Wire PrimaryCommandField into editor panel layout (above editor content)
  - [x] 16.4 Wire context menu triggers (right-click events) to context menu display
  - [x] 16.5 Wire status segment updates to editor state change events (mode, cursor, encoding, line count, modified)
  - [x] 16.6 Wire recent files update on file open/save events
  - [x] 16.7 Write integration tests verifying end-to-end menu activation -> command dispatch
  - Covers: All requirements (system integration)

- [x] 17. Property-based tests
  - [x] 17.1 Write PBT: Recent files list bounded-size property
  - [x] 17.2 Write PBT: Recent files add-or-promote ordering property
  - [x] 17.3 Write PBT: Status segment ID uniqueness property
  - [x] 17.4 Write PBT: Menu item command binding consistency property
  - [x] 17.5 Write PBT: Context menu predicate evaluation consistency property
  - [x] 17.6 Write PBT: Command field history navigation property
  - Covers: All requirements (property-based validation)

---

## Property-Based Test Definitions

### Property 1: Recent Files Bounded-Size

**Validates: Requirement 3.2, 3.3**

- **Statement:** For any sequence of `add_or_promote` operations and any configured max_entries in [1, 50], the recent files list never exceeds max_entries. When an add would exceed the limit, the oldest entry is evicted.
- **Strategy:** Generate:
  - max_entries: integer in [1, 50]
  - Operation sequence: 10-200 add_or_promote calls with paths drawn from a pool of 5-30 unique paths
- **Invariant:** `recent_files.len() <= max_entries` after every operation; after overflow the oldest non-promoted entry is removed

### Property 2: Recent Files Add-or-Promote Ordering

**Validates: Requirement 3.1, 3.3**

- **Statement:** For any sequence of add_or_promote operations, the most recently added or promoted path is always at index 0, and no path appears more than once in the list.
- **Strategy:** Generate:
  - Sequences of 5-100 add_or_promote calls with paths drawn from a pool of 3-20 unique paths
- **Invariant:** After each operation, `list[0] == last_promoted_path` and `list` contains no duplicate paths

### Property 3: Status Segment ID Uniqueness

**Validates: Requirement 5.4, 8.6**

- **Statement:** For any sequence of segment registrations, the status bar never contains two segments with the same ID. Duplicate registration attempts are rejected.
- **Strategy:** Generate:
  - Sequences of 5-50 registration attempts with IDs drawn from a pool of 3-15 valid segment IDs
- **Invariant:** `status_bar.segment_ids()` has no duplicates; duplicate registrations return an error

### Property 4: Menu Item Command Binding Consistency

**Validates: Requirement 2.1, 2.10**

- **Statement:** For any menu item with a command binding, activating the item always results in exactly one `execute_command` call with the bound Command_ID. No menu item activation mutates state directly.
- **Strategy:** Generate:
  - Menu trees with 5-30 items bound to distinct command IDs
  - Simulate activation of random items
- **Invariant:** Each activation produces exactly one dispatch call matching the bound Command_ID; no state mutation outside command dispatch

### Property 5: Context Menu Predicate Evaluation Consistency

**Validates: Requirement 4.3, 4.4**

- **Statement:** For any context menu and any ExecutionContext, the set of enabled items is exactly the set whose bound command's enabled predicate returns true in that context. No disabled item is activatable.
- **Strategy:** Generate:
  - Context menus with 5-15 items
  - ExecutionContexts with varying active_document, selection, and panel states
  - Commands with various enabled predicates (always-true, always-false, conditional)
- **Invariant:** `item.is_enabled(ctx)` iff `command.enabled_predicate(ctx) == true`

### Property 6: Command Field History Navigation

**Validates: Requirement 9.6**

- **Statement:** For any sequence of submitted commands, pressing Up Arrow N times from an empty field yields the Nth most recent command (clamped at oldest). Pressing Down Arrow from any position moves towards the most recent, stopping at empty.
- **Strategy:** Generate:
  - Command sequences: 1-50 unique command strings
  - Navigation sequences: alternating Up/Down presses (5-100 presses)
- **Invariant:** After K Up presses, displayed text == `history[len - K]` (clamped at 0); after returning to bottom, field is empty

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types and Models", "tasks": ["2", "15"], "dependsOn": [0] },
    { "id": 2, "label": "Command Integration", "tasks": ["3", "4"], "dependsOn": [1] },
    { "id": 3, "label": "Menu Rendering", "tasks": ["5", "6", "14"], "dependsOn": [2] },
    { "id": 4, "label": "Context Menus", "tasks": ["7"], "dependsOn": [2] },
    { "id": 5, "label": "Status Bar", "tasks": ["8", "9", "10", "11"], "dependsOn": [1] },
    { "id": 6, "label": "Command Field", "tasks": ["12"], "dependsOn": [2] },
    { "id": 7, "label": "Extensibility", "tasks": ["13"], "dependsOn": [3, 4] },
    { "id": 8, "label": "Integration and PBT", "tasks": ["16", "17"], "dependsOn": [3, 4, 5, 6, 7] }
  ]
}
```

---

## Notes

- This is a Wave 6 (UI and Rendering) crate depending on Wave 2 platform crates (ff-command, ff-core, ff-plugin, ff-config) and Wave 2 layout (ff-layout)
- The `ff-command` crate provides CommandId, CommandRegistry, CommandDispatch, and ShortcutRegistry — ff-menu consumes these without modification
- The `ff-plugin` crate provides the PluginContext and capability discovery — ff-menu uses this for menu and status bar extensibility
- The `ff-config` crate provides configuration access for `menu.recent_files_max` and `statusbar.segments`
- Menu rendering uses `egui::menu` and `egui::popup` — this crate has a direct dependency on egui (it is a UI crate)
- The Primary Command Field submits text to the `ff-command-semantics` CommandEngine for parsing; if that crate is not yet available, a basic pass-through to `execute_command` is acceptable as an interim implementation
- Status bar segments subscribe to editor state changes; the event/subscription mechanism will be defined by ff-core's messaging interface
- Plugin-contributed menus and status segments follow the same lifecycle as the plugin itself — registration on activate, removal on deactivate/shutdown
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Access keys (underlined characters) for keyboard navigation depend on egui's text rendering capabilities; if egui does not natively support underlined access keys, a custom rendering approach within the menu widget will be needed
- The "Tools" top-level menu is an initially empty placeholder populated exclusively by plugin contributions
- Recent files persistence uses a JSON file in the workbench data directory (same location as other workbench state files managed by ff-config)

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Menu Bar Structure | AC 1.1 | Tasks 5 |
| Req 1: Menu Bar Structure | AC 1.2 | Tasks 4, 5 |
| Req 1: Menu Bar Structure | AC 1.3 | Task 4 |
| Req 1: Menu Bar Structure | AC 1.4 | Task 4 |
| Req 1: Menu Bar Structure | AC 1.5 | Task 4 |
| Req 1: Menu Bar Structure | AC 1.6 | Task 4 |
| Req 1: Menu Bar Structure | AC 1.7 | Task 4 |
| Req 1: Menu Bar Structure | AC 1.8 | Tasks 5, 14 |
| Req 2: Menu-Command Integration | AC 2.1 | Task 3 |
| Req 2: Menu-Command Integration | AC 2.2 | Tasks 3, 5 |
| Req 2: Menu-Command Integration | AC 2.3 | Tasks 3, 5 |
| Req 2: Menu-Command Integration | AC 2.4 | Tasks 3, 5 |
| Req 2: Menu-Command Integration | AC 2.5 | Task 4 |
| Req 2: Menu-Command Integration | AC 2.6 | Task 4 |
| Req 2: Menu-Command Integration | AC 2.7 | Task 4 |
| Req 2: Menu-Command Integration | AC 2.8 | Task 4 |
| Req 2: Menu-Command Integration | AC 2.9 | Task 4 |
| Req 2: Menu-Command Integration | AC 2.10 | Task 3 |
| Req 3: Recent Files | AC 3.1-3.7 | Task 6 |
| Req 4: Context Menus | AC 4.1-4.5 | Task 7 |
| Req 5: Status Bar Layout | AC 5.1-5.7 | Task 8 |
| Req 6: Mode and State Indicators | AC 6.1-6.6 | Task 9 |
| Req 7: Position and File Info | AC 7.1-7.6 | Task 10 |
| Req 8: Status Bar Extensibility | AC 8.1-8.6 | Task 11 |
| Req 9: Primary Command Field | AC 9.1-9.7 | Task 12 |
| Req 10: Menu Extensibility | AC 10.1-10.6 | Task 13 |
| Req 11: Keyboard Navigation | AC 11.1-11.6 | Task 14 |

- [x] 18. Help > About dialog
  - [x] 18.1 Create `crates/ff-desktop/src/about_dialog.rs` with `render(ctx, open: &mut bool)` fn
    - Validates: Requirement 13.1, 13.8
  - [x] 18.2 Display app name, version, description, creator credit, AI assistant credit, copyright
    - Validates: Requirement 13.2, 13.3, 13.4, 13.5, 13.6, 13.7
  - [x] 18.3 Add `show_about: bool` field to `WorkbenchShell`; wire `Help > About` menu item to set it `true`
    - Validates: Requirement 13.1
  - [x] 18.4 Call `about_dialog::render` each frame when `show_about` is true
    - Validates: Requirement 13.1, 13.8
  - [x] 18.5 Write unit tests: `about_dialog_version_is_nonempty`, `about_dialog_contains_creator_credit`, `about_dialog_contains_aws_credit`
    - Validates: Requirement 13.3, 13.4, 13.5

- [x] 19. Tab-Order Focus Cycle (Requirement 16)
  - [x] 19.1 Add `FocusStop` enum to `shell.rs` with variants:
          `CommandField`, `PomOption { index: usize }`, `PomExit`,
          `CalendarPrev`, `CalendarNext`, `MenuBar { index: usize }`
    - Validates: Requirement 16.1, 16.3-16.10
  - [x] 19.2 Add `focus_stop: FocusStop` field to `WorkbenchShell`; initialise to `CommandField`
    - Validates: Requirement 16.1
  - [x] 19.3 On first frame, request egui focus on the command field `Id`
    - Validates: Requirement 16.1, 16.2
  - [x] 19.4 In `update()`, consume Tab / Shift+Tab before egui and advance `focus_stop`
          forward / backward through the full cycle:
          CommandField -> PomOption(0..8) -> PomExit -> CalendarPrev -> CalendarNext
          -> MenuBar(0..N-1) -> CommandField (and reverse for Shift+Tab).
          When active tab is NOT POM, skip PomOption/PomExit/Calendar stops.
    - Validates: Requirement 16.3-16.11, 16.19
  - [x] 19.5 Pass `focused_pom_option: Option<usize>` into `primary_option_menu::render()`;
          render focused option row with reversed colours (bg = option_label colour,
          text = panel_bg colour)
    - Validates: Requirement 16.12
  - [x] 19.6 Handle Enter/Space on focused POM option, exit line, and calendar buttons
          to trigger the same action as a mouse click
    - Validates: Requirement 16.13, 16.14, 16.15, 16.16
  - [x] 19.7 After advancing `focus_stop` to a MenuBar stop, apply a visible focus indicator
          on the targeted menu bar item
    - Validates: Requirement 16.17, 16.18
  - [x] 19.8 Write unit tests:
          `focus_cycle_tab_forward_from_command_field_goes_to_pom_option_0`,
          `focus_cycle_tab_forward_through_all_pom_options`,
          `focus_cycle_tab_forward_from_pom_exit_goes_to_calendar_prev`,
          `focus_cycle_tab_forward_from_calendar_next_goes_to_first_menu`,
          `focus_cycle_tab_forward_from_last_menu_wraps_to_command_field`,
          `focus_cycle_shift_tab_from_command_field_goes_to_last_menu`,
          `focus_cycle_shift_tab_from_first_menu_goes_to_command_field`,
          `focus_cycle_non_pom_tab_skips_pom_stops`,
          `focused_pom_option_renders_with_reversed_colours`
    - Validates: Requirement 16.1-16.19

- [x] 20. Tab-Header Focus Stops and Command Field Focus Reliability (Requirements 16.1, 16.2, 16.10, 16.20, 16.21, 16.22)
  - [x] 20.1 Add `TabHeader { index: usize }` variant to `FocusStop` enum
    - Validates: Requirement 16.20, 16.21
  - [x] 20.2 Update `FocusStop::next()` to accept `tab_count: usize`; after last `MenuBar` stop advance to `TabHeader { index: 0 }` (if tab_count > 0), then through all tab headers, then wrap to `CommandField`
    - Validates: Requirement 16.10, 16.20, 16.21
  - [x] 20.3 Update `FocusStop::prev()` to accept `tab_count: usize`; from `CommandField` go to last `TabHeader` (if tab_count > 0); from `TabHeader { index: 0 }` go to last `MenuBar`; from `MenuBar { index: 0 }` go to last `TabHeader` when not POM active
    - Validates: Requirement 16.11, 16.22
  - [x] 20.4 In `update()`, pass `self.tabs.len()` as `tab_count` to `next()` and `prev()`; when `focus_stop == TabHeader { index }` request egui focus on the tab button's `Id`
    - Validates: Requirement 16.20
  - [x] 20.5 In `render_command_field()` (or at the top of `update()`), call `ctx.memory_mut(|m| m.request_focus(cmd_id))` every frame when `focus_stop == CommandField` — not only on startup
    - Validates: Requirement 16.1, 16.2
  - [x] 20.6 Write unit tests:
          `focus_cycle_tab_forward_from_last_menu_goes_to_first_tab_header`,
          `focus_cycle_tab_forward_through_all_tab_headers`,
          `focus_cycle_tab_forward_from_last_tab_header_wraps_to_command_field`,
          `focus_cycle_shift_tab_from_command_field_goes_to_last_tab_header`,
          `focus_cycle_shift_tab_from_first_tab_header_goes_to_last_menu`,
          `focus_cycle_non_pom_includes_tab_headers`
    - Validates: Requirement 16.10, 16.20, 16.21, 16.22

- [x] 21. Title Line — Tab Window Chrome (Requirement 17)
  - [x] 21.1 Add `render_title_line(ctx: &egui::Context)` method to `WorkbenchShell`; render a `TopBottomPanel` between the tab bar and command field displaying context-dependent text
    - Validates: Requirement 17.1, 17.2
  - [x] 21.2 Derive Title_Line text from active tab kind and path: POM → app name + version; FileEditor with path → full path; FileEditor untitled → `[Untitled]`; FilesPanel → `[FILES]`; SettingsPanel → `[SETTINGS]`; Untitled → `[Untitled]`
    - Validates: Requirement 17.3, 17.4, 17.5, 17.6
  - [x] 21.3 Apply Legacy theme styling to Title_Line: when `palette.mode == VisualMode::Legacy`, use `primary_menu_bg` (#0000AA) background and `menu_bar_fg` (#FFFFFF) text; otherwise use standard panel colours
    - Validates: Requirement 17.7, 17.8
  - [x] 21.4 Write unit tests: `title_line_pom_tab_shows_app_name_and_version`, `title_line_file_editor_shows_path`, `title_line_untitled_shows_placeholder`, `title_line_settings_panel_shows_settings`, `title_line_files_panel_shows_files`
    - Validates: Requirement 17.3, 17.4, 17.5, 17.6

- [x] 22. Detachable Tabs — stub wiring (Requirement 18, partial)
  - [x] 22.1 Document "Move to Other View" context menu item as deferred (Phase AL); ensure stub does not panic and shows a status message "Detachable windows: coming in Phase AL"
    - Validates: Requirement 18.1 (stub acknowledgement)
  - [x] 22.2 Add `title_line_text(tab: &TabState) -> String` helper function used by both `render_title_line` and (future) floating window title bar
    - Validates: Requirement 18.5 (preparation)

## Phase AM — Detachable Tab Windows (Requirement 18)

- [x] 23. Implement detachable tab windows via egui child viewports
  - [x] 23.1 Add `is_floating: bool` field to `Tab` struct in `tab_manager.rs`; default `false`
    - Validates: Requirement 18.4
  - [x] 23.2 Add `FloatingTab` struct to `shell.rs` with fields `viewport_id`, `tab_index`, `origin_index`
    - Validates: Requirement 18.1
  - [x] 23.3 Add `floating_tabs: Vec<FloatingTab>`, `detach_pending: Option<usize>`, `redock_pending: Arc<Mutex<Vec<usize>>>` fields to `WorkbenchShell`
    - Validates: Requirement 18.1, 18.3, 18.7
  - [x] 23.4 Wire "Move to Other View" context menu item to set `detach_pending = Some(i)`; enforce 16-window limit with status message
    - Validates: Requirement 18.1, 18.7
  - [x] 23.5 In `update()`, process `detach_pending`: allocate `ViewportId`, push `FloatingTab`, set `tab.is_floating = true`
    - Validates: Requirement 18.1, 18.4
  - [x] 23.6 In `render_tab_bar()`, skip tabs where `is_floating == true`
    - Validates: Requirement 18.4
  - [x] 23.7 Each frame, call `ctx.show_viewport_deferred()` for each `FloatingTab`; render Title_Line + Command_Field + tab content inside the callback; set OS title bar to `title_line_text(tab) + " — FileForge Workbench"`
    - Validates: Requirement 18.1, 18.2, 18.5
  - [x] 23.8 Inside deferred viewport callback, detect close event and push `origin_index` into `redock_pending`
    - Validates: Requirement 18.3
  - [x] 23.9 In `update()`, process `redock_pending`: clear `is_floating`, remove `FloatingTab`, restore tab to `origin_index` (clamped)
    - Validates: Requirement 18.3
  - [x] 23.10 Write unit tests: `floating_tab_is_floating_flag_set`, `floating_tab_limit_enforced_at_16`, `floating_tab_origin_index_preserved`, `redock_clamps_to_tab_count`, `floating_tab_title_format`
    - Validates: Requirement 18.1, 18.3, 18.5, 18.7
