# Implementation Plan: Multi-Tab Editor (`ff-tabs`)

## Overview

This plan covers the complete implementation of the `ff-tabs` crate — the multi-tab editor subsystem for FileForgeWorkbench. The crate manages Tab_Collections, per-tab state isolation, MRU (Most Recently Used) ordering, tab bar display and overflow handling, tab lifecycle (open/close/activate), drag-and-drop reordering, pinned tabs, split editor views, context menus, keyboard navigation, duplicate detection, command framework integration, and session serialisation/deserialisation.

This is a **Wave 8 (File I/O and Session)** sub-project. It depends on `ff-command` (command registration and dispatch), `ff-document` (document model and DocumentHandle), `ff-layout` (Tab_Group containers from Layout_Engine), `ff-vfs` (ResourceUri and VFS access), `ff-config` (configuration system), `ff-undo` (TransactionStack), and `ff-logging` (diagnostics).

---

## Tasks

- [x] 1. Crate scaffolding and core types
  - [x] 1.1 Create `crates/ff-tabs/Cargo.toml` with dependencies (ff-command, ff-document, ff-layout, ff-vfs, ff-config, ff-undo, ff-logging, thiserror, serde, uuid, proptest dev-dep)
  - [x] 1.2 Create `crates/ff-tabs/src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create module files: `tab.rs`, `tab_collection.rs`, `tab_bar.rs`, `mru_stack.rs`, `pinned.rs`, `overflow.rs`, `context_menu.rs`, `drag_drop.rs`, `split_view.rs`, `duplicate_detection.rs`, `session.rs`, `commands.rs`, `keyboard_nav.rs`, `error.rs`
  - [x] 1.4 Add `ff-tabs` to workspace `Cargo.toml` members list
  - [x] 1.5 Define `TabsError` enum with variants: TabNotFound, TabGroupNotFound, DuplicateResource, MaxTabsReached, AllTabsModified, ResourceOpenFailed, SessionDeserializeFailed, SessionMigrationFailed, InvalidTabId, SplitFailed, DragCancelled
  - [x] 1.6 Implement `Display` and `thiserror::Error` derives with descriptive messages for all error variants
  - [x] 1.7 Define `TabId` newtype wrapping `uuid::Uuid` with `Display`, `Clone`, `Copy`, `Hash`, `Eq`, `Serialize`, `Deserialize` derives
  - Covers: Structural foundation for all requirements

- [x] 2. Tab and TabState types
  - [x] 2.1 Define `Tab` struct with fields: id (TabId), document_handle (DocumentHandle), resource_uri (Option<ResourceUri>), state (TabState), pinned (bool), created_at (Instant)
  - [x] 2.2 Define `TabState` struct with fields: viewport_top_line (usize), horizontal_scroll (usize), cursor_line (usize), cursor_column (usize), selections (Vec<SelectionRange>), language (Option<LanguageId>), modified (bool), command_line (String), status_message (String), fold_state (FoldState), bookmarks (BTreeSet<usize>)
  - [x] 2.3 Define `FoldState` struct holding collapsed fold region set
  - [x] 2.4 Implement `Tab::new(document_handle, resource_uri)` constructor generating a fresh TabId
  - [x] 2.5 Implement `Tab::new_untitled(document_handle)` constructor for untitled documents
  - [x] 2.6 Implement `TabState::default()` with viewport at line 0, cursor at (0,0), empty selections
  - [x] 2.7 Implement `Tab::is_modified()` delegating to document dirty flag
  - [x] 2.8 Implement `Serialize`/`Deserialize` for `Tab` and `TabState` (serde derives with custom handling for DocumentHandle)
  - [x] 2.9 Write unit tests for Tab construction, default state, and modified flag delegation
  - Covers: Requirement 1 (AC 1.1), Requirement 2 (AC 2.1, 2.4, 2.5, 2.6, 2.7, 2.8)

- [x] 3. TabCollection data structure
  - [x] 3.1 Define `TabCollection` struct with fields: tabs (Vec<Tab>), active_tab_id (Option<TabId>), mru_stack (MruStack), max_tab_count (usize), tab_group_id (TabGroupId)
  - [x] 3.2 Implement `TabCollection::new(tab_group_id, max_count)` constructor
  - [x] 3.3 Implement `TabCollection::insert(tab, position)` — insert at end or specific index, respecting pinned ordering invariant
  - [x] 3.4 Implement `TabCollection::remove(tab_id) -> Option<Tab>` — remove tab and update MRU stack
  - [x] 3.5 Implement `TabCollection::activate(tab_id)` — set active tab and push to MRU top
  - [x] 3.6 Implement `TabCollection::get(tab_id) -> Option<&Tab>` and `get_mut(tab_id) -> Option<&mut Tab>`
  - [x] 3.7 Implement `TabCollection::active_tab() -> Option<&Tab>` accessor
  - [x] 3.8 Implement `TabCollection::len()`, `is_empty()`, `iter()` convenience methods
  - [x] 3.9 Implement `TabCollection::find_by_uri(uri) -> Option<&Tab>` for duplicate detection across collection
  - [x] 3.10 Implement `TabCollection::tab_at_position(index) -> Option<&Tab>` for positional access (Ctrl+1..9)
  - [x] 3.11 Implement pinned ordering invariant enforcement — pinned tabs always precede unpinned tabs in the Vec
  - [x] 3.12 Write unit tests for insert, remove, activate, find_by_uri, positional access, and pinned ordering
  - Covers: Requirement 1 (AC 1.1, 1.4, 1.5, 1.6), Requirement 10 (AC 10.3)

- [x] 4. Tab lifecycle — open, close, activate
  - [x] 4.1 Implement `open_tab(resource_uri, document_handle, tab_group_id)` — create Tab, check max count, evict LRU if needed, insert, activate
  - [x] 4.2 Implement max-tab-count enforcement — close least-recently-used non-pinned, unmodified tab to make room
  - [x] 4.3 Implement max-tab-count error path — return `AllTabsModified` error when no evictable tab exists
  - [x] 4.4 Implement `open_new_tab(document_handle)` — create untitled tab with empty document
  - [x] 4.5 Implement `close_tab(tab_id)` — remove tab, handle post-close activation (MRU or sequential)
  - [x] 4.6 Implement post-close activation logic — activate next MRU tab, or right neighbour, or left neighbour
  - [x] 4.7 Implement last-tab-closed behaviour — create new empty tab when Tab_Group becomes empty
  - [x] 4.8 Implement `activate_tab(tab_id)` — persist departing tab state, restore arriving tab state, update MRU
  - [x] 4.9 Implement tab-switch state persistence — save viewport, cursor, selections, scroll offset, command line on depart
  - [x] 4.10 Implement tab-switch state restoration — restore all per-tab state fields on arrival
  - [x] 4.11 Implement open-error handling — return error with ResourceUri and failure reason, leave collection unchanged
  - [x] 4.12 Write unit tests for open, close, activate, max-count eviction, last-tab-closed, and error paths
  - Covers: Requirement 1 (AC 1.2, 1.3, 1.5, 1.6, 1.7, 1.8), Requirement 2 (AC 2.2, 2.3), Requirement 5 (AC 5.1, 5.6, 5.7)

- [x] 5. MRU stack tracking
  - [x] 5.1 Define `MruStack` struct with internal `Vec<TabId>` maintaining activation order (most recent first)
  - [x] 5.2 Implement `MruStack::push(tab_id)` — move to top, dedup if already present
  - [x] 5.3 Implement `MruStack::remove(tab_id)` — remove from stack, preserve relative order of remaining
  - [x] 5.4 Implement `MruStack::next(current)` — return next tab in MRU order (deeper into history)
  - [x] 5.5 Implement `MruStack::prev(current)` — return previous tab in MRU order (towards more recent)
  - [x] 5.6 Implement `MruStack::iter()` — iterate in MRU order for popup display
  - [x] 5.7 Implement MRU navigation session state — track session active flag, current position in stack
  - [x] 5.8 Implement `MruStack::begin_session()` — start navigation, set position to second entry
  - [x] 5.9 Implement `MruStack::commit_session()` — end navigation, push final selection to top
  - [x] 5.10 Implement `Serialize`/`Deserialize` for `MruStack` for session persistence
  - [x] 5.11 Write unit tests for push, remove, next/prev cycling, session begin/commit, and serialisation round-trip
  - Covers: Requirement 7 (AC 7.1, 7.2, 7.3, 7.4, 7.5, 7.8, 7.9)

- [x] 6. Tab reordering and drag-and-drop
  - [x] 6.1 Define `DragState` struct with fields: dragged_tab_id (TabId), origin_index (usize), current_hover_index (Option<usize>), source_tab_group (TabGroupId)
  - [x] 6.2 Implement `begin_drag(tab_id, cursor_pos)` — initiate drag after 5px dead zone threshold
  - [x] 6.3 Implement `update_drag(cursor_pos)` — compute hover insertion index, respecting pinned constraints
  - [x] 6.4 Implement `complete_drag()` — move tab to target index within same TabCollection
  - [x] 6.5 Implement `cancel_drag()` — reset state, return tab to original position
  - [x] 6.6 Implement cross-group drag — detect hover over different Tab_Group's Tab_Bar
  - [x] 6.7 Implement cross-group drop — remove from source TabCollection, insert into target TabCollection
  - [x] 6.8 Implement split-on-drop — create new Tab_Group when dropped between existing groups (delegate to Layout_Engine)
  - [x] 6.9 Implement pinned-tab drag constraint — restrict drop to pinned region (before all unpinned tabs)
  - [x] 6.10 Implement `move_tab_left(tab_id)` — swap with left neighbour in TabCollection (command-driven reorder)
  - [x] 6.11 Implement `move_tab_right(tab_id)` — swap with right neighbour in TabCollection
  - [x] 6.12 Implement drag invariant — MRU position unchanged after reorder
  - [x] 6.13 Write unit tests for begin/update/complete/cancel drag, cross-group move, pinned constraint, move left/right, and MRU preservation
  - Covers: Requirement 9 (AC 9.1–9.10)

- [x] 7. Tab groups and split views
  - [x] 7.1 Define `SplitDirection` enum with variants: Right, Down
  - [x] 7.2 Implement `split_tab(tab_id, direction)` — request split from Layout_Engine, create new Tab sharing DocumentHandle
  - [x] 7.3 Implement shared DocumentHandle semantics — new Tab references same `Arc<RwLock<Document>>` with independent TabState
  - [x] 7.4 Implement edit synchronisation — when Document is modified via one view, content reflects in all views sharing the handle
  - [x] 7.5 Implement shared save-point — when Document is saved, clear Modified_Indicator on all Tabs referencing that DocumentHandle
  - [x] 7.6 Implement reference-counted Document lifecycle — Document released only when last referencing Tab is closed
  - [x] 7.7 Implement split view title decoration — optional "[1]", "[2]" suffix on Tab_Headers sharing same resource
  - [x] 7.8 Implement `find_all_tabs_for_document(document_handle) -> Vec<TabId>` across all Tab_Groups
  - [x] 7.9 Write unit tests for split creation, shared state, edit sync, save-point propagation, reference counting, and title decoration
  - Covers: Requirement 12 (AC 12.1–12.7)

- [x] 8. Pinned tabs
  - [x] 8.1 Implement `pin_tab(tab_id)` — set pinned flag, move tab to rightmost pinned position
  - [x] 8.2 Implement `unpin_tab(tab_id)` — clear pinned flag, move tab to leftmost unpinned position
  - [x] 8.3 Implement pinned ordering invariant — all pinned tabs precede all unpinned tabs, relative pin order by pin sequence
  - [x] 8.4 Implement pinned tab close protection — `close_tab` on pinned tab unpins instead of closing
  - [x] 8.5 Implement `close_pinned_tab(tab_id)` — explicit close that bypasses protection (context menu / command)
  - [x] 8.6 Implement bulk-close immunity — `close_all` and `close_others` skip pinned tabs
  - [x] 8.7 Implement pinned tab Modified_Indicator — display alongside pin icon when document dirty
  - [x] 8.8 Implement pinned state serialisation — include in session data
  - [x] 8.9 Implement duplicate-from-pinned — `tabs.duplicate` on pinned tab creates unpinned copy
  - [x] 8.10 Write unit tests for pin/unpin, ordering invariant, close protection, bulk-close immunity, and serialisation
  - Covers: Requirement 10 (AC 10.1–10.8), Requirement 5 (AC 5.8, 5.14)

- [x] 9. Tab close operations
  - [x] 9.1 Implement `close_tab_with_confirmation(tab_id)` — check modified flag, return `NeedsSavePrompt` when dirty
  - [x] 9.2 Implement `close_all(tab_group_id)` — iterate non-pinned tabs left-to-right, prompt for each modified
  - [x] 9.3 Implement `close_others(target_tab_id)` — close all non-pinned except target, prompt for each modified
  - [x] 9.4 Implement `close_to_left(tab_id)` — close non-pinned tabs left of target in order
  - [x] 9.5 Implement `close_to_right(tab_id)` — close non-pinned tabs right of target in order
  - [x] 9.6 Implement bulk-close abort semantics — when user cancels any tab, abort entire bulk operation, leave remaining unchanged
  - [x] 9.7 Implement exit-sequence close — iterate all modified tabs across all Tab_Groups in Tab_Bar order, prompt for each
  - [x] 9.8 Implement exit abort — cancel at any point returns to workbench with all tabs intact
  - [x] 9.9 Implement `CloseDecision` enum: Save, Discard, Cancel — contract for unsaved-changes dialog integration
  - [x] 9.10 Write unit tests for single close, close_all, close_others, close_to_left/right, bulk abort, and exit sequence
  - Covers: Requirement 5 (AC 5.1–5.14)

- [x] 10. Tab bar display and title formatting
  - [x] 10.1 Define `TabTitleFormat` enum with variants: FilenameOnly, FilenameWithDirectory, AutoDisambiguate
  - [x] 10.2 Implement `compute_tab_title(tab, collection, format) -> String` — generate display title per format config
  - [x] 10.3 Implement filename extraction — final path segment from ResourceUri
  - [x] 10.4 Implement untitled naming — "Untitled", "Untitled-2", "Untitled-3" with sequential disambiguation
  - [x] 10.5 Implement auto-disambiguation — append minimum parent directory segments when filenames collide
  - [x] 10.6 Implement `TabHeaderModel` struct with fields: title (String), is_active (bool), is_modified (bool), is_pinned (bool), close_button_visible (bool)
  - [x] 10.7 Implement `build_tab_bar_model(collection) -> Vec<TabHeaderModel>` — produce render-ready header list
  - [x] 10.8 Implement close button visibility rules — hidden on pinned (unless hovered), muted on inactive, prominent on active/hovered
  - [x] 10.9 Write unit tests for title formatting (all modes), untitled naming, disambiguation, and header model generation
  - Covers: Requirement 3 (AC 3.1–3.12)

- [x] 11. Tab overflow handling
  - [x] 11.1 Define `OverflowState` struct with fields: is_overflow (bool), visible_range (Range<usize>), scroll_offset (usize)
  - [x] 11.2 Implement overflow detection — compare rendered tab widths against available Tab_Bar width
  - [x] 11.3 Implement scroll left/right — shift visible_range by one tab position
  - [x] 11.4 Implement active-tab-always-visible — auto-scroll to bring active tab into view when activated
  - [x] 11.5 Implement overflow dropdown model — list of all tabs with title, modified indicator, active highlight
  - [x] 11.6 Implement type-ahead filter — case-insensitive title substring filtering on dropdown content
  - [x] 11.7 Implement dropdown activation — activate selected tab, close dropdown, scroll into view
  - [x] 11.8 Write unit tests for overflow detection, scroll logic, auto-scroll on activate, dropdown model, and type-ahead filter
  - Covers: Requirement 4 (AC 4.1–4.7)

- [x] 12. Context menu
  - [x] 12.1 Define `TabContextAction` enum with all context menu actions: Close, CloseOthers, CloseAll, CloseToLeft, CloseToRight, PinTab, UnpinTab, CopyFileName, CopyRelativePath, CopyAbsolutePath, RevealInFileTree, SplitRight, SplitDown, MoveTabLeft, MoveTabRight
  - [x] 12.2 Implement `build_context_menu(tab_id, collection) -> Vec<ContextMenuItem>` — generate menu items with correct enabled/disabled states
  - [x] 12.3 Implement disabled-state logic: CloseToLeft disabled when no non-pinned tabs to left, CloseToRight disabled when no tabs to right, CloseOthers disabled when only one tab, MoveLeft disabled when leftmost, MoveRight disabled when rightmost
  - [x] 12.4 Implement path menu disabled state — CopyFileName, CopyRelativePath, CopyAbsolutePath, RevealInFileTree disabled for untitled tabs
  - [x] 12.5 Implement PinTab/UnpinTab toggle — show appropriate item based on current pinned state
  - [x] 12.6 Implement `execute_context_action(action, tab_id)` dispatcher — route each action to the corresponding operation
  - [x] 12.7 Implement CopyFileName — extract and return final path segment for clipboard
  - [x] 12.8 Implement CopyRelativePath — compute path relative to workspace root
  - [x] 12.9 Implement CopyAbsolutePath — return full canonical path or ResourceUri string
  - [x] 12.10 Implement RevealInFileTree — emit event for file tree panel to expand and highlight
  - [x] 12.11 Write unit tests for menu construction, disabled states, toggle logic, and action dispatch
  - Covers: Requirement 6 (AC 6.1–6.21)

- [x] 13. Duplicate detection
  - [x] 13.1 Implement `DuplicateDetector` struct holding reference to all Tab_Groups
  - [x] 13.2 Implement `find_existing_tab(uri) -> Option<(TabGroupId, TabId)>` — search across all Tab_Groups
  - [x] 13.3 Implement ResourceUri normalization — resolve symlinks, normalize case on case-insensitive FS, resolve relative segments
  - [x] 13.4 Implement cross-Tab_Group detection — find duplicates in any group, not just active
  - [x] 13.5 Implement focus-on-duplicate — when duplicate found in different group, focus that group and activate the tab
  - [x] 13.6 Implement split-view exception — allow explicit split requests to bypass duplicate detection
  - [x] 13.7 Write unit tests for URI normalization, cross-group detection, focus behaviour, and split exception
  - Covers: Requirement 11 (AC 11.1–11.5)

- [x] 14. Keyboard navigation
  - [x] 14.1 Implement Ctrl+Tab handler — switch to next tab per configured mode (MRU or sequential)
  - [x] 14.2 Implement Ctrl+Shift+Tab handler — switch to previous tab per configured mode
  - [x] 14.3 Implement MRU navigation session — show popup, commit on Ctrl release
  - [x] 14.4 Implement MRU popup model — MRU-ordered list with current selection highlighted
  - [x] 14.5 Implement sequential mode — Ctrl+Tab moves right (wrapping), Ctrl+Shift+Tab moves left (wrapping)
  - [x] 14.6 Implement Ctrl+W handler — close active tab following close rules
  - [x] 14.7 Implement Ctrl+1 through Ctrl+9 — activate tab at position N (Ctrl+9 always last tab)
  - [x] 14.8 Implement position-shortcut no-op — do nothing when collection has fewer than N tabs
  - [x] 14.9 Implement Ctrl+Shift+T — reopen most recently closed tab from closed-tab URI stack
  - [x] 14.10 Implement recently-closed stack — bounded stack of up to 20 ResourceUris from closed tabs
  - [x] 14.11 Implement configurable navigation mode — `mru` (default) or `sequential` from configuration-system
  - [x] 14.12 Write unit tests for MRU cycling, sequential cycling, positional shortcuts, reopen-closed, and mode configuration
  - Covers: Requirement 7 (AC 7.3–7.7), Requirement 8 (AC 8.1–8.8)

- [x] 15. Session serialisation and deserialisation
  - [x] 15.1 Define `SerializedTabCollection` struct with versioned schema (version field, tabs list, mru order, active tab id)
  - [x] 15.2 Define `SerializedTab` struct with fields: tab_id, resource_uri, viewport, cursor, selections, pinned, language_override
  - [x] 15.3 Implement `serialize_tab_collection(collection) -> SerializedTabCollection` — capture full state
  - [x] 15.4 Implement `deserialize_tab_collection(data, vfs) -> Result<TabCollection>` — reconstruct from serialised data
  - [x] 15.5 Implement resource-open-on-restore — load each resource via VFS during deserialization
  - [x] 15.6 Implement skip-on-failure — when a resource cannot be opened, log warning and continue with remaining tabs
  - [x] 15.7 Implement MRU stack restoration from serialised order
  - [x] 15.8 Implement active tab re-activation after restore
  - [x] 15.9 Implement version migration — detect older format versions, attempt data migration
  - [x] 15.10 Implement migration failure fallback — discard stored state, start with single empty tab, log warning
  - [x] 15.11 Write unit tests for serialisation round-trip, skip-on-failure, MRU restoration, version migration, and fallback
  - Covers: Requirement 14 (AC 14.1–14.6), Requirement 2 (AC 2.8)

- [x] 16. Command registration and framework integration
  - [x] 16.1 Register all tab commands with IDs: `tabs.close`, `tabs.close_all`, `tabs.close_others`, `tabs.close_to_left`, `tabs.close_to_right`, `tabs.close_pinned`, `tabs.next`, `tabs.previous`, `tabs.next_mru`, `tabs.previous_mru`, `tabs.pin`, `tabs.unpin`, `tabs.move_left`, `tabs.move_right`, `tabs.goto_1` through `tabs.goto_9`, `tabs.split_right`, `tabs.split_down`, `tabs.reopen_closed`, `tabs.duplicate`
  - [x] 16.2 Implement CommandHandler for each registered command — dispatch to appropriate TabCollection/lifecycle method
  - [x] 16.3 Implement command metadata — display name, description, category "Tabs", default shortcuts
  - [x] 16.4 Implement default keyboard shortcuts: Ctrl+Tab=tabs.next_mru, Ctrl+Shift+Tab=tabs.previous_mru, Ctrl+W=tabs.close, Ctrl+1..9=tabs.goto_N, Ctrl+Shift+T=tabs.reopen_closed
  - [x] 16.5 Implement enabled predicates — disable commands when they cannot meaningfully execute (e.g., close_to_left when leftmost)
  - [x] 16.6 Implement target TabId parameter — commands operate on Active_Tab unless explicit TabId provided
  - [x] 16.7 Implement `tabs.duplicate` — create new Tab with same DocumentHandle (open from VFS) as unpinned tab
  - [x] 16.8 Write unit tests for command registration, metadata correctness, predicate evaluation, and dispatch
  - Covers: Requirement 13 (AC 13.1–13.8)

- [x] 17. Startup and CLI argument handling
  - [x] 17.1 Implement CLI file argument processing — create one Tab per argument in order, set last as Active_Tab
  - [x] 17.2 Implement CLI argument error handling — log error, skip failed argument, continue with remaining
  - [x] 17.3 Implement no-argument startup — create single empty Tab when no CLI args and no session to restore
  - [x] 17.4 Write unit tests for multi-argument open, error-skip behaviour, and empty startup
  - Covers: Requirement 1 (AC 1.9, 1.10)

- [x] 18. Property-based tests
  - [x] 18.1 Write PBT: TabCollection insertion-order preservation property
  - [x] 18.2 Write PBT: MRU stack consistency property
  - [x] 18.3 Write PBT: Pinned tab ordering invariant property
  - [x] 18.4 Write PBT: Tab close post-activation correctness property
  - [x] 18.5 Write PBT: Duplicate detection idempotency property
  - [x] 18.6 Write PBT: Drag-and-drop reorder MRU independence property
  - [x] 18.7 Write PBT: Session serialisation round-trip fidelity property
  - [x] 18.8 Write PBT: Tab title disambiguation completeness property
  - [x] 18.9 Write PBT: Max tab count bounded-collection invariant property
  - [x] 18.10 Write PBT: Bulk close pinned immunity property
  - Covers: All requirements (property-based validation)

- [x] 19. Integration tests
  - [x] 19.1 Write integration test: full open-activate-close lifecycle with MRU tracking
  - [x] 19.2 Write integration test: max tab count enforcement with LRU eviction
  - [x] 19.3 Write integration test: pin tab, bulk close_all, verify pinned tabs remain
  - [x] 19.4 Write integration test: split view shared document edits and save-point propagation
  - [x] 19.5 Write integration test: duplicate detection across multiple Tab_Groups
  - [x] 19.6 Write integration test: drag-and-drop reorder within same Tab_Group
  - [x] 19.7 Write integration test: drag-and-drop move between Tab_Groups
  - [x] 19.8 Write integration test: session serialize, shutdown, deserialize restore cycle
  - [x] 19.9 Write integration test: keyboard navigation MRU session (Ctrl+Tab cycling)
  - [x] 19.10 Write integration test: context menu action dispatch for all actions
  - [x] 19.11 Write integration test: command registration and execution for all tab commands
  - [x] 19.12 Write integration test: tab title disambiguation with colliding filenames
  - [x] 19.13 Write integration test: reopen recently closed tab with URI restoration
  - [x] 19.14 Write integration test: CLI multi-argument startup with partial failures
  - Covers: Cross-requirement interaction validation

---

## Property-Based Test Definitions

### Property 1: TabCollection Insertion-Order Preservation

**Validates: Requirement 1.4**

- **Statement:** For any sequence of tab insertions (without reordering operations), the Tab_Bar displays tabs in exactly the order they were inserted, with pinned tabs preceding unpinned tabs within their respective groups.
- **Strategy:** Generate:
  - Tab count: 1–50 tabs
  - Insertion sequence: random mix of pinned and unpinned tabs
  - No reorder operations applied
- **Invariant:** `collection.iter().map(|t| t.id).collect()` preserves insertion order within the pinned group and within the unpinned group separately.

### Property 2: MRU Stack Consistency

**Validates: Requirement 7.1, 7.2, 7.8**

- **Statement:** The MRU stack always contains exactly the set of open TabIds (no more, no less), with the most recently activated tab at position 0. Closing a tab removes it from the stack without disturbing the relative order of remaining entries.
- **Strategy:** Generate:
  - Operation sequence: 20–200 operations from {Open(tab), Close(tab), Activate(tab)}
  - Track expected MRU state in a model
- **Invariant:** After every operation, `mru_stack.iter().collect::<HashSet<_>>() == open_tabs` AND `mru_stack[0] == last_activated_tab`

### Property 3: Pinned Tab Ordering Invariant

**Validates: Requirement 10.3**

- **Statement:** For any sequence of pin/unpin/insert/remove/reorder operations, all pinned tabs always appear before all unpinned tabs in the TabCollection ordering. No unpinned tab ever precedes a pinned tab.
- **Strategy:** Generate:
  - Initial tabs: 2–30 tabs
  - Operation sequence: 10–100 operations from {Pin(tab), Unpin(tab), Insert(tab), Remove(tab), MoveLeft(tab), MoveRight(tab)}
- **Invariant:** For all `i < j` where `collection[i].pinned == false`, `collection[j].pinned == false`. Equivalently: once an unpinned tab appears, all subsequent tabs are also unpinned.

### Property 4: Tab Close Post-Activation Correctness

**Validates: Requirement 5.7**

- **Statement:** When the active tab is closed in MRU mode, the tab that becomes active is always the next entry in MRU order (the most recently used remaining tab). In sequential mode, it is the right neighbour (or left if rightmost).
- **Strategy:** Generate:
  - Tab count: 2–20 tabs
  - Random activation sequence (builds MRU history)
  - Close the active tab
  - Mode: MRU or Sequential
- **Invariant:** In MRU mode: `new_active == mru_stack_after_remove[0]`. In Sequential mode: `new_active == right_neighbour OR (was_rightmost AND new_active == left_neighbour)`.

### Property 5: Duplicate Detection Idempotency

**Validates: Requirement 11.1, 11.2**

- **Statement:** Opening a resource that already exists in any TabCollection always results in activation of the existing tab — never creates a second tab for the same ResourceUri. The total tab count does not increase.
- **Strategy:** Generate:
  - URI pool: 3–15 unique URIs
  - Tab_Groups: 1–4
  - Open sequence: 20–50 opens from the URI pool (with repeats)
- **Invariant:** After every open, for each URI in the pool, at most one Tab with that URI exists across all Tab_Groups. `total_tab_count <= unique_uris_opened`.

### Property 6: Drag-and-Drop Reorder MRU Independence

**Validates: Requirement 9.9**

- **Statement:** Drag-and-drop reordering within a TabCollection never modifies the MRU stack order. The MRU positions of all tabs remain identical before and after any reorder operation.
- **Strategy:** Generate:
  - Tab count: 3–20 tabs
  - Random activation sequence (builds MRU)
  - Reorder operations: 1–10 random drag-and-drops (source_idx, target_idx)
- **Invariant:** `mru_stack_before == mru_stack_after` for every reorder operation.

### Property 7: Session Serialisation Round-Trip Fidelity

**Validates: Requirement 14.1, 14.4, 14.5**

- **Statement:** For any valid TabCollection state, serialising and then deserialising produces an equivalent TabCollection: same tabs in same order, same MRU ordering, same active tab, same per-tab state (viewport, cursor, selections, pinned flag).
- **Strategy:** Generate:
  - Tab count: 0–30 tabs
  - Random per-tab state: viewport (0–10000), cursor (0–10000, 0–200), selections (0–5 ranges), pinned (bool), language (Option from pool)
  - Random MRU ordering (permutation of tab ids)
  - Random active tab selection
- **Invariant:** `deserialize(serialize(collection)) == collection` (structural equality on all fields except DocumentHandle internals).

### Property 8: Tab Title Disambiguation Completeness

**Validates: Requirement 3.4**

- **Statement:** In `auto_disambiguate` mode, no two tabs in the same TabCollection ever display the same title string. When filenames collide, parent directories are appended until all titles are unique.
- **Strategy:** Generate:
  - Tab count: 2–20 tabs
  - ResourceUri pool with deliberate filename collisions: same filename with 2–5 different parent paths
  - Title format: AutoDisambiguate
- **Invariant:** `titles.len() == titles.iter().collect::<HashSet<_>>().len()` — all computed titles are unique within the collection.

### Property 9: Max Tab Count Bounded-Collection Invariant

**Validates: Requirement 1.6**

- **Statement:** For any sequence of open operations on a TabCollection with a configured maximum of N, the collection never exceeds N tabs. When at capacity, the least-recently-used non-pinned unmodified tab is evicted.
- **Strategy:** Generate:
  - Max count: integer in [1, 50]
  - Tab sequence: 20–100 open operations with random URIs
  - Random subset marked as pinned or modified (non-evictable)
- **Invariant:** `collection.len() <= max_count` after every open operation.

### Property 10: Bulk Close Pinned Immunity

**Validates: Requirement 10.4, Requirement 5.8**

- **Statement:** Bulk close operations (`close_all`, `close_others`) never close pinned tabs. After any bulk close, all previously pinned tabs remain in the collection with their pinned state unchanged.
- **Strategy:** Generate:
  - Tab count: 3–20 tabs
  - Random subset pinned (1–5 tabs)
  - Bulk operation: CloseAll or CloseOthers(random_target)
  - All unpinned tabs have no unsaved changes (to avoid dialog complexity)
- **Invariant:** After bulk close, `collection.iter().filter(|t| t.pinned).collect::<Vec<_>>() == pinned_tabs_before`.

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types", "tasks": ["2", "3"], "dependsOn": [0] },
    { "id": 2, "label": "Tab Lifecycle", "tasks": ["4", "5"], "dependsOn": [1] },
    { "id": 3, "label": "Reordering and Splits", "tasks": ["6", "7"], "dependsOn": [2] },
    { "id": 4, "label": "Pinned Tabs and Close Operations", "tasks": ["8", "9"], "dependsOn": [2] },
    { "id": 5, "label": "Display and Overflow", "tasks": ["10", "11"], "dependsOn": [2] },
    { "id": 6, "label": "Context Menu and Detection", "tasks": ["12", "13"], "dependsOn": [3, 4, 5] },
    { "id": 7, "label": "Keyboard Navigation", "tasks": ["14"], "dependsOn": [2, 4] },
    { "id": 8, "label": "Session Persistence", "tasks": ["15"], "dependsOn": [2, 4, 5] },
    { "id": 9, "label": "Command Registration", "tasks": ["16"], "dependsOn": [3, 4, 6, 7] },
    { "id": 10, "label": "Startup", "tasks": ["17"], "dependsOn": [2, 8] },
    { "id": 11, "label": "Property-Based Tests", "tasks": ["18"], "dependsOn": [1, 2, 3, 4, 5, 6, 7, 8] },
    { "id": 12, "label": "Integration Tests", "tasks": ["19"], "dependsOn": [9, 10, 11] }
  ]
}
```

---

## Notes

- This is a Wave 8 (File I/O and Session) crate depending on `ff-command` (Wave 2), `ff-document` (Wave 4), `ff-layout` (Wave 2), `ff-vfs` (Wave 3), `ff-config` (Wave 2), `ff-undo` (Wave 4), and `ff-logging` (Wave 0)
- All tab operations are dispatched through the command framework — no direct state mutation from UI code (FFW-ARCH cross-cutting Requirement 4)
- Tab identity is based on `ResourceUri` (VFS addresses), not raw filesystem paths — duplicate detection normalizes URIs before comparison
- The `DocumentHandle` type is `Arc<RwLock<Document>>` — split views share the same handle, enabling real-time edit synchronisation
- The Layout_Engine (`ff-layout`) owns Tab_Group spatial arrangement; `ff-tabs` owns per-tab content and state within those containers
- The GUI shell crate renders Tab_Headers and handles mouse/keyboard events — `ff-tabs` provides the data model and logic only (GUI independence principle)
- The unsaved-changes dialog is an abstracted contract — `ff-tabs` defines the decision enum, GUI shell provides the presentation
- The `file-operations` crate (sibling Wave 8) handles open/save I/O; `ff-tabs` integrates via tab creation/activation APIs
- The `startup-and-session` crate (sibling Wave 8) invokes `serialize_tab_collection`/`deserialize_tab_collection` during session lifecycle
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The recently-closed-tab stack is bounded to 20 entries — only ResourceUris are stored, not full document state
- MRU navigation popup is a transient UI element — `ff-tabs` provides the model (`MruStack::iter()`), GUI shell renders the popup
- Configurable settings are sourced from the `configuration-system` (`ff-config`): `tabs.max_count`, `tabs.navigation_mode`, `tabs.title_format`, `tabs.close_button_on_inactive`, `tabs.pinned_position_policy`

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Tab Collection | AC 1.1 | Tasks 2, 3 |
| Req 1: Tab Collection | AC 1.2 | Task 4 |
| Req 1: Tab Collection | AC 1.3 | Task 4 |
| Req 1: Tab Collection | AC 1.4 | Task 3 |
| Req 1: Tab Collection | AC 1.5 | Tasks 3, 4 |
| Req 1: Tab Collection | AC 1.6 | Tasks 3, 4 |
| Req 1: Tab Collection | AC 1.7 | Task 4 |
| Req 1: Tab Collection | AC 1.8 | Task 4 |
| Req 1: Tab Collection | AC 1.9 | Task 17 |
| Req 1: Tab Collection | AC 1.10 | Task 17 |
| Req 2: Per-Tab State | AC 2.1 | Task 2 |
| Req 2: Per-Tab State | AC 2.2 | Task 4 |
| Req 2: Per-Tab State | AC 2.3 | Task 4 |
| Req 2: Per-Tab State | AC 2.4 | Task 2 |
| Req 2: Per-Tab State | AC 2.5 | Task 2 |
| Req 2: Per-Tab State | AC 2.6 | Task 2 |
| Req 2: Per-Tab State | AC 2.7 | Task 2 |
| Req 2: Per-Tab State | AC 2.8 | Tasks 2, 15 |
| Req 3: Tab Bar Display | AC 3.1 | Task 10 |
| Req 3: Tab Bar Display | AC 3.2 | Task 10 |
| Req 3: Tab Bar Display | AC 3.3 | Task 10 |
| Req 3: Tab Bar Display | AC 3.4 | Task 10 |
| Req 3: Tab Bar Display | AC 3.5 | Task 10 |
| Req 3: Tab Bar Display | AC 3.6 | Task 10 |
| Req 3: Tab Bar Display | AC 3.7 | Task 10 |
| Req 3: Tab Bar Display | AC 3.8 | Task 10 |
| Req 3: Tab Bar Display | AC 3.9 | Task 10 |
| Req 3: Tab Bar Display | AC 3.10 | Task 10 |
| Req 3: Tab Bar Display | AC 3.11 | Task 10 |
| Req 3: Tab Bar Display | AC 3.12 | Task 10 |
| Req 4: Tab Overflow | AC 4.1 | Task 11 |
| Req 4: Tab Overflow | AC 4.2 | Task 11 |
| Req 4: Tab Overflow | AC 4.3 | Task 11 |
| Req 4: Tab Overflow | AC 4.4 | Task 11 |
| Req 4: Tab Overflow | AC 4.5 | Task 11 |
| Req 4: Tab Overflow | AC 4.6 | Task 11 |
| Req 4: Tab Overflow | AC 4.7 | Task 11 |
| Req 5: Tab Close | AC 5.1 | Tasks 4, 9 |
| Req 5: Tab Close | AC 5.2 | Task 9 |
| Req 5: Tab Close | AC 5.3 | Task 9 |
| Req 5: Tab Close | AC 5.4 | Task 9 |
| Req 5: Tab Close | AC 5.5 | Task 9 |
| Req 5: Tab Close | AC 5.6 | Task 4 |
| Req 5: Tab Close | AC 5.7 | Task 4 |
| Req 5: Tab Close | AC 5.8 | Tasks 8, 9 |
| Req 5: Tab Close | AC 5.9 | Task 9 |
| Req 5: Tab Close | AC 5.10 | Task 9 |
| Req 5: Tab Close | AC 5.11 | Task 9 |
| Req 5: Tab Close | AC 5.12 | Task 9 |
| Req 5: Tab Close | AC 5.13 | Task 9 |
| Req 5: Tab Close | AC 5.14 | Task 8 |
| Req 6: Context Menu | AC 6.1 | Task 12 |
| Req 6: Context Menu | AC 6.2 | Task 12 |
| Req 6: Context Menu | AC 6.3 | Tasks 9, 12 |
| Req 6: Context Menu | AC 6.4 | Tasks 9, 12 |
| Req 6: Context Menu | AC 6.5 | Tasks 9, 12 |
| Req 6: Context Menu | AC 6.6 | Tasks 9, 12 |
| Req 6: Context Menu | AC 6.7 | Tasks 9, 12 |
| Req 6: Context Menu | AC 6.8 | Tasks 8, 12 |
| Req 6: Context Menu | AC 6.9 | Tasks 8, 12 |
| Req 6: Context Menu | AC 6.10 | Task 12 |
| Req 6: Context Menu | AC 6.11 | Task 12 |
| Req 6: Context Menu | AC 6.12 | Task 12 |
| Req 6: Context Menu | AC 6.13 | Task 12 |
| Req 6: Context Menu | AC 6.14 | Tasks 7, 12 |
| Req 6: Context Menu | AC 6.15 | Tasks 7, 12 |
| Req 6: Context Menu | AC 6.16 | Tasks 6, 12 |
| Req 6: Context Menu | AC 6.17 | Tasks 6, 12 |
| Req 6: Context Menu | AC 6.18 | Task 12 |
| Req 6: Context Menu | AC 6.19 | Task 12 |
| Req 6: Context Menu | AC 6.20 | Task 12 |
| Req 6: Context Menu | AC 6.21 | Task 12 |
| Req 7: MRU Ordering | AC 7.1 | Task 5 |
| Req 7: MRU Ordering | AC 7.2 | Task 5 |
| Req 7: MRU Ordering | AC 7.3 | Tasks 5, 14 |
| Req 7: MRU Ordering | AC 7.4 | Tasks 5, 14 |
| Req 7: MRU Ordering | AC 7.5 | Tasks 5, 14 |
| Req 7: MRU Ordering | AC 7.6 | Task 14 |
| Req 7: MRU Ordering | AC 7.7 | Task 14 |
| Req 7: MRU Ordering | AC 7.8 | Task 5 |
| Req 7: MRU Ordering | AC 7.9 | Tasks 5, 15 |
| Req 8: Keyboard Nav | AC 8.1 | Task 14 |
| Req 8: Keyboard Nav | AC 8.2 | Task 14 |
| Req 8: Keyboard Nav | AC 8.3 | Task 14 |
| Req 8: Keyboard Nav | AC 8.4 | Task 16 |
| Req 8: Keyboard Nav | AC 8.5 | Task 14 |
| Req 8: Keyboard Nav | AC 8.6 | Task 14 |
| Req 8: Keyboard Nav | AC 8.7 | Task 14 |
| Req 8: Keyboard Nav | AC 8.8 | Task 16 |
| Req 9: Drag-and-Drop | AC 9.1 | Task 6 |
| Req 9: Drag-and-Drop | AC 9.2 | Task 6 |
| Req 9: Drag-and-Drop | AC 9.3 | Task 6 |
| Req 9: Drag-and-Drop | AC 9.4 | Task 6 |
| Req 9: Drag-and-Drop | AC 9.5 | Task 6 |
| Req 9: Drag-and-Drop | AC 9.6 | Task 6 |
| Req 9: Drag-and-Drop | AC 9.7 | Task 6 |
| Req 9: Drag-and-Drop | AC 9.8 | Task 6 |
| Req 9: Drag-and-Drop | AC 9.9 | Task 6 |
| Req 9: Drag-and-Drop | AC 9.10 | Task 6 |
| Req 10: Pinned Tabs | AC 10.1 | Task 8 |
| Req 10: Pinned Tabs | AC 10.2 | Task 8 |
| Req 10: Pinned Tabs | AC 10.3 | Task 8 |
| Req 10: Pinned Tabs | AC 10.4 | Task 8 |
| Req 10: Pinned Tabs | AC 10.5 | Tasks 8, 10 |
| Req 10: Pinned Tabs | AC 10.6 | Task 8 |
| Req 10: Pinned Tabs | AC 10.7 | Tasks 8, 15 |
| Req 10: Pinned Tabs | AC 10.8 | Task 8 |
| Req 11: Duplicate Detection | AC 11.1 | Task 13 |
| Req 11: Duplicate Detection | AC 11.2 | Task 13 |
| Req 11: Duplicate Detection | AC 11.3 | Task 13 |
| Req 11: Duplicate Detection | AC 11.4 | Task 13 |
| Req 11: Duplicate Detection | AC 11.5 | Tasks 7, 13 |
| Req 12: Split Editor | AC 12.1 | Task 7 |
| Req 12: Split Editor | AC 12.2 | Task 7 |
| Req 12: Split Editor | AC 12.3 | Task 7 |
| Req 12: Split Editor | AC 12.4 | Task 7 |
| Req 12: Split Editor | AC 12.5 | Task 7 |
| Req 12: Split Editor | AC 12.6 | Task 7 |
| Req 12: Split Editor | AC 12.7 | Task 7 |
| Req 13: Command Integration | AC 13.1 | Task 16 |
| Req 13: Command Integration | AC 13.2 | Task 16 |
| Req 13: Command Integration | AC 13.3 | Task 16 |
| Req 13: Command Integration | AC 13.4 | Task 16 |
| Req 13: Command Integration | AC 13.5 | Task 16 |
| Req 13: Command Integration | AC 13.6 | Task 16 |
| Req 13: Command Integration | AC 13.7 | Task 16 |
| Req 13: Command Integration | AC 13.8 | Task 16 |
| Req 14: Session Persistence | AC 14.1 | Task 15 |
| Req 14: Session Persistence | AC 14.2 | Task 15 |
| Req 14: Session Persistence | AC 14.3 | Task 15 |
| Req 14: Session Persistence | AC 14.4 | Task 15 |
| Req 14: Session Persistence | AC 14.5 | Task 15 |
| Req 14: Session Persistence | AC 14.6 | Task 15 |
