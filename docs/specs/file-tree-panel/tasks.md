# Implementation Plan: File Tree Panel (`ff-file-tree-panel`)

## Overview

This task plan implements the `ff-file-tree-panel` crate — the unified resource explorer panel for FileForgeWorkbench. It renders all registered VFS providers as a multi-root tree hierarchy in a dockable panel, supporting asynchronous directory loading, file watching, drag-and-drop, context menus, keyboard navigation, search/filter, dataset catalog browsing, and configurable appearance.

**Crate location:** `crates/ff-file-tree-panel`
**Upstream dependencies:** `ff-vfs` (Wave 3), `ff-layout` (Wave 2), `ff-config` (Wave 2), `ff-command` (Wave 2), `ff-theme` (Wave 6), `ff-connector-local-fs` (Wave 3), `ff-dataset-catalog` (Wave 13)
**Downstream consumers:** Application shell (ff-desktop)

---

## Tasks

- [x] 1. Crate scaffold and core data model
  - [x] 1.1 Create `crates/ff-file-tree-panel/Cargo.toml` with dependencies (ff-vfs, ff-layout, ff-config, ff-command, ff-theme, egui, tokio, async-trait, thiserror, serde, serde_json) and dev-dependencies (proptest, tempfile, pretty_assertions, tokio-test)
  - [x] 1.2 Create `crates/ff-file-tree-panel/src/lib.rs` with crate-level doc comment and public module declarations
  - [x] 1.3 Implement `src/error.rs` — define `FileTreeError` enum with variants (VfsError, ConfigError, InvalidPath, WatchError, NodeNotFound, ConcurrencyLimitReached, OperationCancelled)
  - [x] 1.4 Implement `src/model/mod.rs` — module for tree data model types
  - [x] 1.5 Implement `src/model/tree_node.rs` — define `TreeNode` struct (id, label, icon, node_type, expansion_state, children, parent_id, vfs_uri, file_category, metadata) and `NodeType` enum (RootCategory, Directory, File, Dataset, PdsMember, CatalogRoot, Placeholder)
  - [x] 1.6 Implement `src/model/file_category.rs` — define `FileCategory` enum (NonEditableBinary, FileForgeStructured, StandardText, Unknown, Directory, SymbolicLink) with classification logic by extension and metadata
  - [x] 1.7 Implement `src/model/tree_root.rs` — define `TreeRoot` enum (LocalFiles, Catalogs, Connections) and `RootCategory` struct with label, expansion state, children
  - [x] 1.8 Implement `src/model/sort_order.rs` — define `SortOrder` enum (DirectoriesFirst, Alphabetical, Type, ModifiedDate) with `sort_nodes()` comparator function
  - [x] 1.9 Write unit tests for FileCategory classification, SortOrder comparators, TreeNode construction
    - Validates: Requirement 4 AC 1, AC 2, AC 5

- [x] 2. Panel layout and DockablePanel integration
  - [x] 2.1 Implement `src/panel.rs` — define `FileTreePanel` struct implementing `DockablePanel` trait with `panel_id` returning `"file_tree"`, `default_dock_zone` returning `DockZone::Left`
  - [x] 2.2 Implement default width of 260 logical pixels, resizable range constrained between 120 and 600 logical pixels
  - [x] 2.3 Implement width persistence to layout state (save on resize, restore on startup)
  - [x] 2.4 Implement collapse/expand functionality — minimizes to icon strip (≤28 px width) with title and toggle
  - [x] 2.5 Implement collapse/expand state persistence in layout state
  - [x] 2.6 Implement title bar rendering — display "Explorer" with refresh button and collapse/expand toggle
  - [x] 2.7 Implement configuration gate — read `file_tree.enabled` (default: true); when false, do not register with Panel_Registry
  - [x] 2.8 Write unit tests for DockablePanel trait implementation, width clamping, configuration gate logic
    - Validates: Requirement 1 AC 1–8

- [x] 3. Multi-root tree hierarchy
  - [x] 3.1 Implement `src/roots.rs` — define `RootManager` struct managing three top-level root categories (Local Files, Catalogs, Connections) as expandable section headers
  - [x] 3.2 Implement Local Files root — enumerate all bookmarked root paths from config, each as an expandable directory node served by `connector-local-fs` VFS provider
  - [x] 3.3 Implement fallback root — if no bookmarked roots exist, display process working directory (or `file_tree.default_root`) as single default root
  - [x] 3.4 Implement Catalogs root — enumerate all mounted dataset catalogs from `dataset-catalog` VFS provider as expandable nodes
  - [x] 3.5 Implement Catalogs empty state — display "No catalogs mounted" placeholder node (non-expandable, greyed)
  - [x] 3.6 Implement Connections root — enumerate registered remote VFS providers; display "No connections configured" placeholder when none registered
  - [x] 3.7 Implement independent section expansion/collapse with persisted state per root category
  - [x] 3.8 Implement add bookmarked root — context menu action or toolbar button opening native folder picker
  - [x] 3.9 Implement remove/rename bookmarked root — context menu actions with immediate config persistence
  - [x] 3.10 Implement bookmarked roots persistence at config key `file_tree.bookmarked_roots` as ordered array
  - [x] 3.11 Write unit tests for root enumeration, empty states, bookmark add/remove/persist, section expansion state
    - Validates: Requirement 2 AC 1–10

- [x] 4. Async directory loading
  - [x] 4.1 Implement `src/async_loader.rs` — define `AsyncLoader` struct managing concurrent VFS list operations with a semaphore limiting to 8 simultaneous loads
  - [x] 4.2 Implement async node expansion — on user expand, initiate VFS `list` operation off UI thread, display Loading_Indicator (spinner + "Loading...") as child
  - [x] 4.3 Implement successful load handling — replace Loading_Indicator with sorted directory entries on completion
  - [x] 4.4 Implement error handling — replace Loading_Indicator with error node (muted/error colour) on VFS failure; log at WARN level
  - [x] 4.5 Implement children cache — cache expanded node children in memory; skip VFS call on collapse/re-expand unless invalidated
  - [x] 4.6 Implement collapse behaviour — retain cached children, cancel pending async operations for deeper unexpanded subtrees
  - [x] 4.7 Implement cancellation — if user collapses before load completes, cancel the pending VFS operation via tokio cancellation token
  - [x] 4.8 Implement concurrency limiter — enforce maximum 8 simultaneous async directory loads via semaphore
  - [x] 4.9 Write unit tests for async load lifecycle, Loading_Indicator states, error node rendering, cache hits, cancellation, concurrency limit enforcement
    - Validates: Requirement 3 AC 1–8
  - [x] 4.10 Write property test: concurrency limit invariant (Property 1) — trigger N>8 simultaneous expansions, assert at most 8 are in-flight concurrently
    - Validates: Requirement 3 AC 8

- [x] 5. Tree node rendering and sort order
  - [x] 5.1 Implement `src/rendering/node_renderer.rs` — render tree nodes with type-appropriate icons (folder open/closed, text file, source code, binary, image, config, dataset, PDS, member, catalog, connection)
  - [x] 5.2 Implement file-category colour mapping — obtain foreground colours from theme palette `file_tree` colour group (non_editable_binary, fileforge_structured, standard_text, unknown, directory, symbolic_link)
  - [x] 5.3 Implement Structure_Badge rendering — overlay icon or inline label for FileForgeStructured files with associated structure definitions
  - [x] 5.4 Implement sort logic — default `directories_first` (dirs before files, then alphabetical case-insensitive); support `alphabetical`, `type`, `modified_date` modes via config key
  - [x] 5.5 Implement hidden files filtering — respect `file_tree.show_hidden_files` config (default false); hide files starting with `.` or with hidden attribute
  - [x] 5.6 Implement large directory truncation — for directories with >10,000 visible entries, display first 1,000 followed by "... and N more items" indicator with "Show All" action
  - [x] 5.7 Write unit tests for icon selection by type/extension, colour mapping, sort correctness across modes, hidden file filtering, large directory truncation
    - Validates: Requirement 4 AC 1–8
  - [x] 5.8 Write property test: sort order stability (Property 2) — generate random node lists, sort with each mode, verify result is totally ordered and stable (equal elements preserve insertion order)
    - Validates: Requirement 4 AC 1, AC 2
  - [x] 5.9 Write property test: hidden file filtering completeness (Property 3) — generate nodes with mixed visibility, assert show_hidden=false excludes all hidden nodes and show_hidden=true includes all
    - Validates: Requirement 4 AC 7

- [x] 6. File watching and live updates
  - [x] 6.1 Implement `src/watcher.rs` — define `TreeWatcher` struct managing VFS watch registrations per expanded directory node
  - [x] 6.2 Implement watch registration — when a directory node is expanded, register VFS watch; when collapsed, cancel the watch
  - [x] 6.3 Implement `Created` event handling — insert new entry at correct sorted position within parent's children
  - [x] 6.4 Implement `Deleted` event handling — remove corresponding node from tree
  - [x] 6.5 Implement `Renamed` event handling — update node label and re-sort position within parent
  - [x] 6.6 Implement `Modified` event handling — update node metadata (size, modification time)
  - [x] 6.7 Implement capability check — if VFS provider does not support `watch`, skip registration and log at DEBUG level
  - [x] 6.8 Implement debounce — batch rapid watch events for same directory within 200ms window into single tree update
  - [x] 6.9 Write unit tests for watch registration/cancellation lifecycle, event handling for each type, debounce batching, capability check bypass
    - Validates: Requirement 5 AC 1–8
  - [x] 6.10 Write property test: debounce batching invariant (Property 4) — generate N events within 200ms for same directory, assert exactly one tree update is applied
    - Validates: Requirement 5 AC 8

- [x] 7. Context menus
  - [x] 7.1 Implement `src/context_menu.rs` — define `TreeContextMenu` struct with methods returning command descriptors per node type
  - [x] 7.2 Implement file node menu (Local Files) — Open, Open With..., Rename, Delete, New File (sibling), New Folder (sibling), Copy Path, Reveal in System Explorer
  - [x] 7.3 Implement directory node menu (Local Files) — Expand/Collapse, New File (child), New Folder (child), Rename, Delete, Copy Path, Reveal in System Explorer, Refresh
  - [x] 7.4 Implement dataset node menu (Catalogs) — Open, Rename, Delete, Properties, Copy DSN
  - [x] 7.5 Implement PDS dataset node menu (Catalogs) — Expand/Collapse, New Member, Rename, Delete, Properties, Copy DSN
  - [x] 7.6 Implement PDS member node menu (Catalogs) — Open, Rename, Delete, Copy DSN
  - [x] 7.7 Implement catalog root node menu — Unmount, Refresh, Properties
  - [x] 7.8 Implement Local Files section header menu — Add Root Folder, Refresh All
  - [x] 7.9 Implement command dispatch — all actions dispatched through command framework (file_tree.open, file_tree.rename, file_tree.delete, file_tree.new_file, file_tree.new_folder, file_tree.copy_path, file_tree.reveal_in_explorer, file_tree.refresh)
  - [x] 7.10 Implement delete confirmation dialog — "Delete {name}? This action cannot be undone." before dispatching delete command
  - [x] 7.11 Implement inline rename — activate text editor on node label, pre-filled with current name, confirm with Enter, cancel with Escape
  - [x] 7.12 Write unit tests for menu generation per node type, command dispatch mapping, delete confirmation flow, inline rename activation/confirm/cancel
    - Validates: Requirement 6 AC 1–10

- [x] 8. Drag-and-drop and file opening
  - [x] 8.1 Implement `src/interaction/open.rs` — double-click on file/member node dispatches `file.open` command with VFS URI to active Tab_Group
  - [x] 8.2 Implement Enter key on file node dispatches `file.open` command with VFS URI
  - [x] 8.3 Implement drag initiation — start drag on file node, carry VFS URI as payload
  - [x] 8.4 Implement drop on editor area — dispatch `file.open` targeting the drop Tab_Group
  - [x] 8.5 Implement drop indicator — while drag is in progress over editor area, display drop indicator showing tab placement
  - [x] 8.6 Implement drag cancel — drop outside valid target cancels with no action
  - [x] 8.7 Implement single-click selection — highlight node, display VFS URI/path in status bar
  - [x] 8.8 Implement double-click on directory — toggle expansion state
  - [x] 8.9 Write unit tests for open dispatch (double-click, Enter), drag payload construction, selection state, directory double-click toggle
    - Validates: Requirement 7 AC 1–7

- [x] 9. Keyboard navigation
  - [x] 9.1 Implement `src/interaction/keyboard.rs` — define `KeyboardHandler` processing key events when panel has focus
  - [x] 9.2 Implement Down Arrow — move selection to next visible node below
  - [x] 9.3 Implement Up Arrow — move selection to previous visible node above
  - [x] 9.4 Implement Right Arrow on collapsed directory — expand node (trigger async load if not cached)
  - [x] 9.5 Implement Right Arrow on expanded node — move selection to first child
  - [x] 9.6 Implement Left Arrow on expanded directory — collapse node
  - [x] 9.7 Implement Left Arrow on child node — move selection to parent node
  - [x] 9.8 Implement Enter on file node — open file (same as double-click)
  - [x] 9.9 Implement Enter on directory node — toggle expansion state
  - [x] 9.10 Implement Delete key — trigger delete confirmation dialog
  - [x] 9.11 Implement F2 key — activate inline rename
  - [x] 9.12 Implement Home/End — move to first/last visible node
  - [x] 9.13 Implement type-ahead search — alphanumeric chars jump to next sibling matching typed prefix
  - [x] 9.14 Write unit tests for all keyboard navigation actions, boundary cases (first/last node, root-level Left Arrow), type-ahead matching
    - Validates: Requirement 8 AC 1–12
  - [x] 9.15 Write property test: keyboard navigation order consistency (Property 5) — generate tree with N visible nodes, assert Down Arrow N-1 times visits every node exactly once in display order
    - Validates: Requirement 8 AC 1, AC 2

- [x] 10. Search and filter
  - [x] 10.1 Implement `src/search.rs` — define `TreeSearchBox` component rendered below title bar, above tree content
  - [x] 10.2 Implement text filter — as user types, filter tree to show only nodes whose label contains search text (case-insensitive substring)
  - [x] 10.3 Implement ancestor expansion — auto-expand all ancestors of matching nodes so matches are visible
  - [x] 10.4 Implement non-match hiding — non-matching nodes that are not ancestors of a match are hidden
  - [x] 10.5 Implement filter clear — on backspace-to-empty or clear button, restore full unfiltered tree preserving pre-filter expansion state
  - [x] 10.6 Implement glob pattern support — when input contains `*` or `?`, use glob matching instead of substring (e.g., `*.rs` matches Rust files)
  - [x] 10.7 Implement cached-only filter — filter operates on cached tree data without triggering new VFS operations
  - [x] 10.8 Implement Ctrl+Shift+E shortcut — move focus to Tree_Search_Box
  - [x] 10.9 Implement Escape from search box — return focus to tree, clear filter
  - [x] 10.10 Write unit tests for substring matching, glob matching, ancestor expansion, filter clear state restoration, keyboard shortcuts
    - Validates: Requirement 9 AC 1–9
  - [x] 10.11 Write property test: filter preserves tree invariants (Property 6) — generate tree and filter string, assert every visible node either matches or is ancestor of a match
    - Validates: Requirement 9 AC 2, AC 3, AC 4
  - [x] 10.12 Write property test: glob pattern matching consistency (Property 7) — generate file names and glob patterns, verify `*` matches zero-or-more chars, `?` matches exactly one char
    - Validates: Requirement 9 AC 6

- [x] 11. Dataset catalog browsing
  - [x] 11.1 Implement `src/catalog_tree.rs` — render mounted catalogs as expandable child nodes under "Catalogs" root, labelled with catalog name
  - [x] 11.2 Implement HLQ grouping — on catalog expansion, list datasets grouped by High_Level_Qualifier as intermediate directory-like nodes
  - [x] 11.3 Implement sequential dataset rendering — DSORG=PS as leaf file nodes with sequential-dataset icon
  - [x] 11.4 Implement PDS rendering — DSORG=PO as expandable folder-like nodes with PDS icon; on expand, list all members
  - [x] 11.5 Implement PDS member rendering — leaf file nodes with member icon, labelled with 1–8 character member name
  - [x] 11.6 Implement GDG rendering — DSORG=GDG as expandable nodes with GDG icon; on expand, list active generations sorted newest-first
  - [x] 11.7 Implement dataset tooltip — display RECFM, LRECL, DSORG on hover after 500ms delay
  - [x] 11.8 Implement Properties panel integration — context menu "Properties" opens panel/dialog with full dataset attributes (DSN, DSORG, RECFM, LRECL, BLKSIZE, creation date, modification date, physical path)
  - [x] 11.9 Implement dataset colour — apply `file_tree.fileforge_structured` colour from theme palette to dataset nodes
  - [x] 11.10 Implement PDS member opening — double-click/Enter dispatches `file.open` with VFS URI `vfs://catalog/{catalog-name}/{DSN}({member})`
  - [x] 11.11 Write unit tests for catalog tree rendering, HLQ grouping, dataset type icons, member listing, GDG ordering, tooltip content, URI construction
    - Validates: Requirement 10 AC 1–10

- [x] 12. Path bar and navigation
  - [x] 12.1 Implement `src/path_bar.rs` — define `PathBar` component displayed above/integrated with Tree_Search_Box showing path of focused root or selected node
  - [x] 12.2 Implement path editing — on click, input becomes editable with full path text selected
  - [x] 12.3 Implement path navigation — on Enter, navigate to typed path: expand tree to reveal if under current root, or add as temporary root if not
  - [x] 12.4 Implement not-found handling — if path does not exist (VFS stat returns NotFound), display inline "Path not found" error and revert
  - [x] 12.5 Implement Browse button — folder icon opens native OS folder picker; on selection, navigate to chosen folder
  - [x] 12.6 Implement VFS URI support — path resolution goes through VFS layer, supports `vfs://` URIs in addition to bare local paths
  - [x] 12.7 Write unit tests for path display, edit activation, navigation success/failure, browse button flow, URI parsing
    - Validates: Requirement 11 AC 1–6

- [x] 13. Refresh command
  - [x] 13.1 Implement `src/commands/refresh.rs` — register `file_tree.refresh` command with command framework
  - [x] 13.2 Implement full refresh — invalidate all cached directory listings, re-load all currently expanded nodes from VFS providers
  - [x] 13.3 Implement refresh UI feedback — display Loading_Indicator on each refreshing node during reload
  - [x] 13.4 Implement per-node refresh — context menu "Refresh" reloads only that directory and its expanded descendants
  - [x] 13.5 Implement selection preservation — after refresh, preserve selection if node still exists; if deleted, move to nearest sibling or parent
  - [x] 13.6 Implement refresh triggers — toolbar button, context menu, F5 keyboard shortcut while panel has focus
  - [x] 13.7 Write unit tests for full refresh lifecycle, per-node refresh scope, selection preservation/fallback, trigger mechanisms
    - Validates: Requirement 12 AC 1–5

- [x] 14. Configuration integration
  - [x] 14.1 Implement `src/config.rs` — define `FileTreeConfig` struct reading all `file_tree.*` keys (enabled, default_width, default_root, bookmarked_roots, sort_order, show_hidden_files) from configuration-system
  - [x] 14.2 Implement hot-reload — subscribe to `file_tree.*` config change events; apply new values without application restart
  - [x] 14.3 Implement show_hidden_files hot-reload — immediately re-filter all displayed nodes on change
  - [x] 14.4 Implement sort_order hot-reload — re-sort all displayed directory contents on change
  - [x] 14.5 Implement per-project overrides — support configuration layering (user → project → profile) for all keys
  - [x] 14.6 Write unit tests for config loading, validation, hot-reload trigger for each key, per-project override precedence
    - Validates: Requirement 13 AC 1–5

- [x] 15. Accessibility and visual feedback
  - [x] 15.1 Implement selection highlighting — distinct background colour from theme palette (`ui.selection_background`) meeting WCAG AA contrast
  - [x] 15.2 Implement focus ring — visible focus indicator on focused node, distinct from selection highlight
  - [x] 15.3 Implement tree semantics for assistive technology — tree role, tree-item roles, expanded/collapsed state, level depth, set size/position
  - [x] 15.4 Implement icon text labels — all icons have associated text labels for tooltip and accessibility name
  - [x] 15.5 Implement high-contrast support — in High-Contrast mode, all colours/icons/indicators meet WCAG AAA (7:1 ratio) via theme system
  - [x] 15.6 Implement indent guides — vertical lines connecting parent to child using theme colour `ui.indent_guide`
  - [x] 15.7 Implement open-file indicator — nodes representing files open in editor tabs show subtle "open" indicator (dot or modified background)
  - [x] 15.8 Write unit tests for selection highlight application, focus ring rendering, indent guide depth calculation, open-file indicator state
    - Validates: Requirement 14 AC 1–7

- [x] 16. Command registration
  - [x] 16.1 Implement `src/commands/mod.rs` — module structure for all file tree commands
  - [x] 16.2 Register `file_tree.open` command — open selected file in editor
  - [x] 16.3 Register `file_tree.rename` command — activate inline rename on selected node
  - [x] 16.4 Register `file_tree.delete` command — trigger delete confirmation and dispatch VFS delete
  - [x] 16.5 Register `file_tree.new_file` command — create new file in selected directory (or sibling)
  - [x] 16.6 Register `file_tree.new_folder` command — create new folder in selected directory (or sibling)
  - [x] 16.7 Register `file_tree.copy_path` command — copy VFS URI/path to clipboard
  - [x] 16.8 Register `file_tree.reveal_in_explorer` command — open native file explorer at node location
  - [x] 16.9 Register `file_tree.add_root` command — open folder picker to add bookmarked root
  - [x] 16.10 Write unit tests for command registration, parameter validation, dispatch to correct panel operations
    - Validates: Requirement 6 AC 8; Requirement 12 AC 1

- [x] 17. Integration tests and end-to-end validation
  - [x] 17.1 Write integration test: full panel lifecycle — create panel, verify docking, expand Local Files root, async load directory, verify nodes rendered with correct icons/colours/sort
  - [x] 17.2 Write integration test: multi-root browsing — configure bookmarked roots and mounted catalogs, verify all three root sections display correctly
  - [x] 17.3 Write integration test: file watching — expand directory, simulate VFS watch events (create, delete, rename), verify tree updates correctly with debounce
  - [x] 17.4 Write integration test: search and filter — type search text, verify filter/expand/hide behaviour, clear and verify restoration
  - [x] 17.5 Write integration test: keyboard navigation — simulate key sequences, verify selection traversal, expansion/collapse, open dispatch
  - [x] 17.6 Write integration test: dataset catalog browsing — mount catalog, expand to HLQ/datasets/members, open PDS member, verify VFS URI
  - [x] 17.7 Write integration test: configuration hot-reload — change sort_order and show_hidden_files at runtime, verify immediate tree update
    - Validates: All requirements end-to-end

- [x] 18. Native catalog recursive expansion and scrollable panel (Phase AY)
  - [x] 18.1 Wrap the File Explorer Panel content area in `egui::ScrollArea::vertical()` so the user can scroll through large listings
    - Validates: Requirement 15.3
  - [x] 18.2 Replace flat directory entries in `render_native_children()` with `CollapsingHeader` nodes that recursively call `render_native_children()`, using the full path as `id_salt`
    - Validates: Requirement 15.1, 15.2
  - [x] 18.3 Add unit test confirming that a nested directory structure is readable to at least two levels deep
    - Validates: Requirement 15.2

---

## Acceptance Criteria Coverage

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Panel Layout and Docking | AC 1 (DockablePanel trait) | 2.1, 2.8 |
| Req 1: Panel Layout and Docking | AC 2 (visible on start) | 2.7, 2.8 |
| Req 1: Panel Layout and Docking | AC 3 (default width 260px) | 2.2, 2.8 |
| Req 1: Panel Layout and Docking | AC 4 (width persistence) | 2.3, 2.8 |
| Req 1: Panel Layout and Docking | AC 5 (collapse to icon strip) | 2.4, 2.8 |
| Req 1: Panel Layout and Docking | AC 6 (collapse state persistence) | 2.5, 2.8 |
| Req 1: Panel Layout and Docking | AC 7 (title bar) | 2.6, 2.8 |
| Req 1: Panel Layout and Docking | AC 8 (config gate) | 2.7, 2.8 |
| Req 2: Multi-Root Tree Hierarchy | AC 1 (three root categories) | 3.1, 3.11 |
| Req 2: Multi-Root Tree Hierarchy | AC 2 (Local Files bookmarks) | 3.2, 3.11 |
| Req 2: Multi-Root Tree Hierarchy | AC 3 (default root fallback) | 3.3, 3.11 |
| Req 2: Multi-Root Tree Hierarchy | AC 4 (Catalogs enumeration) | 3.4, 3.11 |
| Req 2: Multi-Root Tree Hierarchy | AC 5 (Catalogs empty state) | 3.5, 3.11 |
| Req 2: Multi-Root Tree Hierarchy | AC 6 (Connections empty state) | 3.6, 3.11 |
| Req 2: Multi-Root Tree Hierarchy | AC 7 (section expand/collapse) | 3.7, 3.11 |
| Req 2: Multi-Root Tree Hierarchy | AC 8 (add bookmark) | 3.8, 3.11 |
| Req 2: Multi-Root Tree Hierarchy | AC 9 (remove/rename bookmark) | 3.9, 3.11 |
| Req 2: Multi-Root Tree Hierarchy | AC 10 (bookmark persistence) | 3.10, 3.11 |
| Req 3: Async Directory Loading | AC 1 (non-blocking VFS list) | 4.1, 4.2, 4.9 |
| Req 3: Async Directory Loading | AC 2 (Loading_Indicator) | 4.2, 4.9 |
| Req 3: Async Directory Loading | AC 3 (replace with entries) | 4.3, 4.9 |
| Req 3: Async Directory Loading | AC 4 (error node on failure) | 4.4, 4.9 |
| Req 3: Async Directory Loading | AC 5 (children cache) | 4.5, 4.9 |
| Req 3: Async Directory Loading | AC 6 (collapse retains cache) | 4.6, 4.9 |
| Req 3: Async Directory Loading | AC 7 (cancellation) | 4.7, 4.9 |
| Req 3: Async Directory Loading | AC 8 (concurrency limit) | 4.8, 4.9, 4.10 |
| Req 4: Tree Node Rendering | AC 1 (directories-first sort) | 5.4, 5.7, 5.8 |
| Req 4: Tree Node Rendering | AC 2 (sort order config) | 5.4, 5.7, 5.8 |
| Req 4: Tree Node Rendering | AC 3 (type-appropriate icons) | 5.1, 5.7 |
| Req 4: Tree Node Rendering | AC 4 (built-in icon set) | 5.1, 5.7 |
| Req 4: Tree Node Rendering | AC 5 (file-category colours) | 5.2, 5.7 |
| Req 4: Tree Node Rendering | AC 6 (Structure_Badge) | 5.3, 5.7 |
| Req 4: Tree Node Rendering | AC 7 (hidden files) | 5.5, 5.7, 5.9 |
| Req 4: Tree Node Rendering | AC 8 (large directory truncation) | 5.6, 5.7 |
| Req 5: File Watching | AC 1 (register watch on expand) | 6.2, 6.9 |
| Req 5: File Watching | AC 2 (Created event insert) | 6.3, 6.9 |
| Req 5: File Watching | AC 3 (Deleted event remove) | 6.4, 6.9 |
| Req 5: File Watching | AC 4 (Renamed event update) | 6.5, 6.9 |
| Req 5: File Watching | AC 5 (Modified event metadata) | 6.6, 6.9 |
| Req 5: File Watching | AC 6 (cancel watch on collapse) | 6.2, 6.9 |
| Req 5: File Watching | AC 7 (capability check) | 6.7, 6.9 |
| Req 5: File Watching | AC 8 (debounce 200ms) | 6.8, 6.9, 6.10 |
| Req 6: Context Menus | AC 1 (file node menu) | 7.2, 7.12 |
| Req 6: Context Menus | AC 2 (directory node menu) | 7.3, 7.12 |
| Req 6: Context Menus | AC 3 (dataset node menu) | 7.4, 7.12 |
| Req 6: Context Menus | AC 4 (PDS dataset menu) | 7.5, 7.12 |
| Req 6: Context Menus | AC 5 (PDS member menu) | 7.6, 7.12 |
| Req 6: Context Menus | AC 6 (catalog root menu) | 7.7, 7.12 |
| Req 6: Context Menus | AC 7 (Local Files header menu) | 7.8, 7.12 |
| Req 6: Context Menus | AC 8 (command dispatch) | 7.9, 7.12, 16.2–16.9 |
| Req 6: Context Menus | AC 9 (delete confirmation) | 7.10, 7.12 |
| Req 6: Context Menus | AC 10 (inline rename) | 7.11, 7.12 |
| Req 7: Drag-and-Drop | AC 1 (double-click opens file) | 8.1, 8.9 |
| Req 7: Drag-and-Drop | AC 2 (Enter opens file) | 8.2, 8.9 |
| Req 7: Drag-and-Drop | AC 3 (drag-drop to editor) | 8.3, 8.4, 8.9 |
| Req 7: Drag-and-Drop | AC 4 (drop indicator) | 8.5 |
| Req 7: Drag-and-Drop | AC 5 (cancel on invalid target) | 8.6 |
| Req 7: Drag-and-Drop | AC 6 (single-click select) | 8.7, 8.9 |
| Req 7: Drag-and-Drop | AC 7 (double-click directory toggle) | 8.8, 8.9 |
| Req 8: Keyboard Navigation | AC 1 (Down Arrow) | 9.2, 9.14, 9.15 |
| Req 8: Keyboard Navigation | AC 2 (Up Arrow) | 9.3, 9.14, 9.15 |
| Req 8: Keyboard Navigation | AC 3 (Right Arrow expand) | 9.4, 9.14 |
| Req 8: Keyboard Navigation | AC 4 (Right Arrow to child) | 9.5, 9.14 |
| Req 8: Keyboard Navigation | AC 5 (Left Arrow collapse) | 9.6, 9.14 |
| Req 8: Keyboard Navigation | AC 6 (Left Arrow to parent) | 9.7, 9.14 |
| Req 8: Keyboard Navigation | AC 7 (Enter opens file) | 9.8, 9.14 |
| Req 8: Keyboard Navigation | AC 8 (Enter toggles directory) | 9.9, 9.14 |
| Req 8: Keyboard Navigation | AC 9 (Delete key) | 9.10, 9.14 |
| Req 8: Keyboard Navigation | AC 10 (F2 rename) | 9.11, 9.14 |
| Req 8: Keyboard Navigation | AC 11 (Home/End) | 9.12, 9.14 |
| Req 8: Keyboard Navigation | AC 12 (type-ahead search) | 9.13, 9.14 |
| Req 9: Search and Filter | AC 1 (search box position) | 10.1, 10.10 |
| Req 9: Search and Filter | AC 2 (substring filter) | 10.2, 10.10, 10.11 |
| Req 9: Search and Filter | AC 3 (ancestor expansion) | 10.3, 10.10, 10.11 |
| Req 9: Search and Filter | AC 4 (non-match hiding) | 10.4, 10.10, 10.11 |
| Req 9: Search and Filter | AC 5 (filter clear) | 10.5, 10.10 |
| Req 9: Search and Filter | AC 6 (glob patterns) | 10.6, 10.10, 10.12 |
| Req 9: Search and Filter | AC 7 (cached-only filter) | 10.7, 10.10 |
| Req 9: Search and Filter | AC 8 (Ctrl+Shift+E) | 10.8, 10.10 |
| Req 9: Search and Filter | AC 9 (Escape clears) | 10.9, 10.10 |
| Req 10: Dataset Catalog Browsing | AC 1 (catalog child nodes) | 11.1, 11.11 |
| Req 10: Dataset Catalog Browsing | AC 2 (HLQ grouping) | 11.2, 11.11 |
| Req 10: Dataset Catalog Browsing | AC 3 (sequential dataset) | 11.3, 11.11 |
| Req 10: Dataset Catalog Browsing | AC 4 (PDS expandable) | 11.4, 11.11 |
| Req 10: Dataset Catalog Browsing | AC 5 (PDS member nodes) | 11.5, 11.11 |
| Req 10: Dataset Catalog Browsing | AC 6 (GDG rendering) | 11.6, 11.11 |
| Req 10: Dataset Catalog Browsing | AC 7 (dataset tooltip) | 11.7, 11.11 |
| Req 10: Dataset Catalog Browsing | AC 8 (Properties panel) | 11.8, 11.11 |
| Req 10: Dataset Catalog Browsing | AC 9 (dataset colour) | 11.9, 11.11 |
| Req 10: Dataset Catalog Browsing | AC 10 (PDS member open) | 11.10, 11.11 |
| Req 11: Path Bar | AC 1 (path display) | 12.1, 12.7 |
| Req 11: Path Bar | AC 2 (click to edit) | 12.2, 12.7 |
| Req 11: Path Bar | AC 3 (Enter navigates) | 12.3, 12.7 |
| Req 11: Path Bar | AC 4 (not-found error) | 12.4, 12.7 |
| Req 11: Path Bar | AC 5 (Browse button) | 12.5, 12.7 |
| Req 11: Path Bar | AC 6 (VFS URI support) | 12.6, 12.7 |
| Req 12: Refresh Command | AC 1 (register command) | 13.1, 13.7, 16.10 |
| Req 12: Refresh Command | AC 2 (full refresh) | 13.2, 13.7 |
| Req 12: Refresh Command | AC 3 (Loading_Indicator during refresh) | 13.3, 13.7 |
| Req 12: Refresh Command | AC 4 (per-node refresh) | 13.4, 13.7 |
| Req 12: Refresh Command | AC 5 (selection preservation) | 13.5, 13.7 |
| Req 13: Configuration | AC 1 (config keys) | 14.1, 14.6 |
| Req 13: Configuration | AC 2 (hot-reload) | 14.2, 14.6 |
| Req 13: Configuration | AC 3 (show_hidden hot-reload) | 14.3, 14.6 |
| Req 13: Configuration | AC 4 (sort_order hot-reload) | 14.4, 14.6 |
| Req 13: Configuration | AC 5 (per-project overrides) | 14.5, 14.6 |
| Req 14: Accessibility | AC 1 (selection highlight) | 15.1, 15.8 |
| Req 14: Accessibility | AC 2 (focus ring) | 15.2, 15.8 |
| Req 14: Accessibility | AC 3 (tree semantics) | 15.3 |
| Req 14: Accessibility | AC 4 (icon text labels) | 15.4 |
| Req 14: Accessibility | AC 5 (high-contrast) | 15.5 |
| Req 14: Accessibility | AC 6 (indent guides) | 15.6, 15.8 |
| Req 14: Accessibility | AC 7 (open-file indicator) | 15.7, 15.8 |

---

## Property-Based Test Summary

| Property | Statement | Task | Validates |
|----------|-----------|------|-----------|
| P1 | Concurrency limit invariant: at most 8 async directory loads in-flight simultaneously | 4.10 | Req 3 AC 8 |
| P2 | Sort order stability: sort with any mode produces a totally ordered, stable result | 5.8 | Req 4 AC 1, AC 2 |
| P3 | Hidden file filtering completeness: show_hidden=false excludes all hidden nodes, true includes all | 5.9 | Req 4 AC 7 |
| P4 | Debounce batching invariant: N events within 200ms for same directory produce exactly one tree update | 6.10 | Req 5 AC 8 |
| P5 | Keyboard navigation order: Down Arrow N-1 times visits every visible node exactly once in display order | 9.15 | Req 8 AC 1, AC 2 |
| P6 | Filter tree invariant: every visible node either matches the filter or is an ancestor of a match | 10.11 | Req 9 AC 2, AC 3, AC 4 |
| P7 | Glob pattern matching: `*` matches zero-or-more chars, `?` matches exactly one char | 10.12 | Req 9 AC 6 |

---

## Notes

- Task 1 (scaffold) has no dependencies and must complete first
- Tasks 2 (panel docking) and 14 (configuration) can proceed in parallel once task 1 is done — both need the core model but are otherwise independent
- Task 3 (multi-root hierarchy) depends on tasks 2 and 14 since it needs the panel container and config keys
- Tasks 4 (async loading) and 6 (file watching) depend on task 3 since they operate on the tree structure
- Task 5 (rendering) depends on task 4 since nodes must be loaded before rendered with icons/colours
- Tasks 7 (context menus), 8 (drag-drop), 9 (keyboard nav), and 10 (search) depend on tasks 4–5 since they require a rendered tree with loaded nodes
- Task 11 (catalog browsing) depends on task 3 and can proceed in parallel with tasks 7–10
- Task 12 (path bar) depends on task 3 since it navigates within the tree structure
- Task 13 (refresh) depends on tasks 4 and 6 since it invalidates caches and re-triggers loads
- Task 15 (accessibility) depends on task 5 since it extends the rendering layer with focus/selection visuals
- Task 16 (commands) can proceed once tasks 7–13 define the operations being dispatched
- Task 17 (integration tests) runs last as it exercises the full stack
- All property tests use the `proptest` crate with a minimum of 100 iterations
- All async tests use `#[tokio::test]` where applicable
- Physical file operations in tests use `tempfile::TempDir` to avoid polluting the real filesystem
- The mock VFS, configuration, and command framework interfaces should be defined in `tests/support/` for integration tests

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Crate scaffold and core data model", "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7", "1.8", "1.9"] },
    { "id": 1, "label": "Panel docking and configuration", "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7", "2.8", "14.1", "14.2", "14.3", "14.4", "14.5", "14.6"], "dependsOn": [0] },
    { "id": 2, "label": "Multi-root tree hierarchy", "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7", "3.8", "3.9", "3.10", "3.11"], "dependsOn": [1] },
    { "id": 3, "label": "Async loading, rendering, and file watching", "tasks": ["4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "4.8", "4.9", "4.10", "5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7", "5.8", "5.9", "6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "6.8", "6.9", "6.10"], "dependsOn": [2] },
    { "id": 4, "label": "Interaction layer — context menus, drag-drop, keyboard, search, path bar", "tasks": ["7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7", "7.8", "7.9", "7.10", "7.11", "7.12", "8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8", "8.9", "9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7", "9.8", "9.9", "9.10", "9.11", "9.12", "9.13", "9.14", "9.15", "10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8", "10.9", "10.10", "10.11", "10.12", "12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7"], "dependsOn": [3] },
    { "id": 5, "label": "Dataset catalog browsing and refresh", "tasks": ["11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7", "11.8", "11.9", "11.10", "11.11", "13.1", "13.2", "13.3", "13.4", "13.5", "13.6", "13.7"], "dependsOn": [3] },
    { "id": 6, "label": "Accessibility, commands, and visual feedback", "tasks": ["15.1", "15.2", "15.3", "15.4", "15.5", "15.6", "15.7", "15.8", "16.1", "16.2", "16.3", "16.4", "16.5", "16.6", "16.7", "16.8", "16.9", "16.10"], "dependsOn": [4, 5] },
    { "id": 7, "label": "Integration tests and end-to-end validation", "tasks": ["17.1", "17.2", "17.3", "17.4", "17.5", "17.6", "17.7"], "dependsOn": [6] }
  ]
}
```

- [x] 19. File Explorer context menu — Phase AZ (Requirement 16)
  - [x] 19.1 Add `NodeKind` enum (`NativeFile`, `NativeDir`, `PosixFile`, `MfPs`, `MfPds`, `MfMember`, `MfGdgBase`, `MfGdgGen`) and `MenuItem` enum (all actions + `Separator` + `Disabled`) to `file_explorer_panel.rs`
    - Validates: Requirement 16.1
  - [x] 19.2 Implement `build_context_menu(catalog_type, node_kind, extension) -> Vec<MenuItem>` consulting `ExtensionRule` table; produce correct item lists for all 8 node kinds per Req 16.2–16.9
    - Validates: Requirement 16.2–16.9
  - [x] 19.3 Wire `ui.response.context_menu()` on each tree node; render items from `build_context_menu`; Git submenu and Submit JCL rendered via `ui.add_enabled(false, ...)`
    - Validates: Requirement 16.1, 16.15, 16.16
  - [x] 19.4 Implement inline rename state in `FileExplorerPanelState` (`rename_state: Option<(String, String)>`); render `TextEdit` in place of label; confirm on Enter (rename on disk / in store), cancel on Escape; enforce 8-char uppercase for Mainframe members
    - Validates: Requirement 16.11
  - [x] 19.5 Implement Copy action — write full path / DSN to OS clipboard via `arboard`
    - Validates: Requirement 16.10 AC 1–2
  - [x] 19.6 Implement Copy path variants — Copy File Name, Copy Relative Path, Copy Full Path, Copy Dataset Name, Copy Member Name, Copy Dataset(Member)
    - Validates: Requirement 16.18
  - [x] 19.7 Implement Reveal in Explorer / Open Containing Folder — platform-appropriate OS file manager launch; platform-specific label
    - Validates: Requirement 16.14
  - [x] 19.8 Implement Copy To… / Move To… dialog (`copy_move_dialog.rs`) — target picker, proposed name with naming-rule transformation, inline validation, dispatch to `ff-bgio`, progress indicator in status bar
    - Validates: Requirement 16.12
  - [x] 19.9 Write unit tests: `build_context_menu` returns correct item sets for all 8 node kinds; inline rename enforces Mainframe 8-char rule; Copy path variants produce correct strings; Copy To naming transformation (Native→Mainframe uppercase/truncate, Mainframe→Native lowercase)
    - Validates: Requirement 16.2–16.9, 16.11, 16.18, 16.12

- [x] 20. Open With Default Application — Phase BA (Requirement 17)
  - [x] 20.1 Add `FileClass` enum (`Text`, `FfwbStructured`, `External`) and `file_class` field to `ExtensionRule`; populate `EXTERNAL_EXTENSIONS` table for all Req 17.8 categories
    - Validates: Requirement 17.8
  - [x] 20.2 Implement `classify_file(path) -> FileClass`: extension lookup then magic-byte fallback (first 512 bytes, null-byte / UTF-8 detection)
    - Validates: Requirement 17.1, 17.2, 17.3
  - [x] 20.3 Implement `launch_default_app(path)`: platform dispatch via `Command::spawn()` (Windows: `cmd /c start`, macOS: `open`, Linux: `xdg-open`); store error in `FileExplorerPanelState::last_error`
    - Validates: Requirement 17.2, 17.4, 17.6
  - [x] 20.4 Replace direct `*open_path = Some(...)` in `handle_menu_action` Open branch with `open_file_node()` that routes Text/FfwbStructured to editor and External to `launch_default_app`; Mainframe nodes bypass classification
    - Validates: Requirement 17.1, 17.2, 17.7
  - [x] 20.5 Wire `last_error` display into status bar (shell reads `file_explorer_panel.last_error` each frame and shows it)
    - Validates: Requirement 17.4
  - [x] 20.6 Write unit tests: `classify_file` returns correct `FileClass` for all Req 17.8 extension categories; magic-byte scan correctly identifies binary vs text; `launch_default_app` uses correct command per platform
    - Validates: Requirement 17.1–17.8

- [x] 21. Native catalog sorted listing and file attributes — Phase BB (Requirement 18)
  - [x] 21.1 Add `FileEntryRow` struct to `file_explorer_panel.rs` holding `name`, `is_dir`, `size_bytes: Option<u64>`, `created: Option<SystemTime>`, `modified: Option<SystemTime>`, `accessed: Option<SystemTime>`, `permissions_str: String`
    - Validates: Requirement 18.2–18.6
  - [x] 21.2 Refactor `render_native_children()` to collect entries via `read_dir()`, call `entry.metadata()` per entry, silently skip entries where metadata returns an error, build `Vec<FileEntryRow>`, sort directories-first then alphabetically case-insensitive
    - Validates: Requirement 18.1, 18.7
  - [x] 21.3 Implement `format_size(bytes: u64) -> String` — `B`, `KB`, `MB`, `GB` with one decimal place; directories return `"<DIR>"`
    - Validates: Requirement 18.2
  - [x] 21.4 Implement `format_timestamp(t: SystemTime) -> String` — `YYYY-MM-DD HH:MM`
    - Validates: Requirement 18.3, 18.4, 18.5
  - [x] 21.5 Implement `format_permissions(meta: &Metadata) -> String` — Windows: `R`/`H`/`S`/`A` flags or `—`; Unix: `rwxr-xr-x` style
    - Validates: Requirement 18.6
  - [x] 21.6 Render each `FileEntryRow` as a horizontal row with columns: Name (expandable), Size (right-aligned ~70px), Modified (~120px), Created (~120px), Accessed (~120px), Permissions (~80px)
    - Validates: Requirement 18.9
  - [x] 21.7 In `open_file_node()`, catch OS error 32 (file in use) and store `"Cannot open '<filename>': file is in use by another process"` in `last_error`; do not open editor tab
    - Validates: Requirement 18.8, B018
  - [x] 21.8 Write unit tests: `format_size` for B/KB/MB/GB boundaries; `format_timestamp` round-trip; `format_permissions` for read-only and full-access cases; sort order (dirs before files, alpha within group); silent-skip of metadata-error entries
    - Validates: Requirement 18.1–18.9

- [x] 22. Drag-select and copy as text tree — Phase BD (Requirement 19)
  - [x] 22.1 Add `selected_nodes: HashSet<String>` and `anchor_node: Option<String>` fields to `FileExplorerPanelState`
    - Validates: Requirement 19.1, 19.2, 19.3
  - [x] 22.2 Wire plain-click, Shift+click, Ctrl+click, and drag-select input handling on each node row response; update `selected_nodes` and `anchor_node` accordingly
    - Validates: Requirement 19.1, 19.2, 19.3
  - [x] 22.3 Render selected nodes with `ui.visuals().selection.bg_fill` background tint
    - Validates: Requirement 19.4
  - [x] 22.4 Implement `build_text_tree(selected_paths: &[&str], all_visible: &[NodeRow]) -> String` — pure function producing indented ASCII tree with `[DIR]` prefix for directories and tree connectors when parent-child relationships exist within the selection
    - Validates: Requirement 19.6
  - [x] 22.5 Wire Ctrl+C detection in the panel render loop: when `selected_nodes` is non-empty, call `build_text_tree` and write result to OS clipboard via `arboard`; store errors in `last_error`
    - Validates: Requirement 19.5
  - [x] 22.6 Add "Copy as Text Tree" item to context menu above the existing "Copy" group; wire it to the same `build_text_tree` path
    - Validates: Requirement 19.7
  - [x] 22.7 Wire Escape key to clear `selected_nodes` to single-element set containing `anchor_node`
    - Validates: Requirement 19.8
  - [x] 22.8 Write unit tests:
    - `build_text_tree_flat_selection` — non-hierarchical selection produces one path per line
    - `build_text_tree_hierarchical_selection` — parent + children produce connector-decorated output
    - `build_text_tree_dir_prefix` — directory nodes are prefixed with `[DIR] `
    - `build_text_tree_relative_depth` — shallowest selected node is at indent level 0
    - `build_text_tree_mainframe_uses_dsn` — Mainframe nodes use DSN not file path
    - Validates: Requirement 19.6, 19.10

- [x] 23. Keyboard navigation and focus transfer — Phase BE (Requirement 20)
  - [x] 23.1 Add `cursor_node: Option<String>` and `explorer_focused: bool` fields to `FileExplorerPanelState`; add `FocusStop::FileExplorer` variant to the shell `FocusStop` enum; wire Tab-cycle so Tab from `CommandField` (when File Explorer tab is active) sets `explorer_focused = true` and positions `cursor_node` on the first visible catalog node
    - Validates: Requirement 20.1
  - [x] 23.2 Implement `collect_visible_node_paths(state: &FileExplorerPanelState) -> Vec<String>` — pure function returning ordered list of all currently visible node paths in display order
    - Validates: Requirement 20.2, 20.4
  - [x] 23.3 Wire Tab key in the panel render loop: when `explorer_focused`, advance `cursor_node` to next in `collect_visible_node_paths()`; if current node is a collapsed container, expand it first
    - Validates: Requirement 20.2, 20.3
  - [x] 23.4 Wire plain Down/Up Arrow keys: move `cursor_node` without touching `selected_nodes`; containers are NOT expanded
    - Validates: Requirement 20.4
  - [x] 23.5 Wire Shift+Down/Up Arrow: move `cursor_node` and add newly visited node to `selected_nodes`; set `anchor_node` on first Shift+Arrow if selection was empty
    - Validates: Requirement 20.6, 20.7, 20.8
  - [x] 23.6 Wire Ctrl+Down/Up Arrow: move `cursor_node` only; `selected_nodes` unchanged
    - Validates: Requirement 20.9
  - [x] 23.7 Wire Ctrl+Space: toggle `cursor_node` path in `selected_nodes`
    - Validates: Requirement 20.10
  - [x] 23.8 Wire Escape: clear `selected_nodes`; `cursor_node` remains
    - Validates: Requirement 20.12
  - [x] 23.9 Render `cursor_node` with a focus ring (distinct from selection fill); selected nodes use `ui.visuals().selection.bg_fill`; a node that is both cursor and selected shows both
    - Validates: Requirement 20.13
  - [x] 23.10 Write unit tests:
    - `tab_from_command_field_sets_explorer_focused` — Tab from CommandField when FileExplorer tab active sets `explorer_focused = true`
    - `tab_advances_cursor_to_next_visible_node` — Tab moves cursor forward in visible order
    - `tab_on_collapsed_container_expands_it` — Tab on collapsed dir expands before advancing
    - `arrow_down_moves_cursor_without_expanding` — Down Arrow on collapsed container does not expand
    - `shift_arrow_adds_to_selection` — Shift+Down adds node to `selected_nodes`
    - `ctrl_arrow_moves_cursor_without_changing_selection` — Ctrl+Down moves cursor, selection unchanged
    - `ctrl_space_toggles_cursor_node_in_selection` — Ctrl+Space toggles membership
    - `escape_clears_selection_preserves_cursor` — Escape clears `selected_nodes`, cursor stays
    - Validates: Requirement 20.1–20.13

- [x] 24. File copy and paste operations — Phase BE (Requirement 21)
  - [x] 24.1 Add `FileCopyClipboard { paths: Vec<String>, operation: CopyOperation }` struct and `CopyOperation { Copy, Cut }` enum to `file_explorer_panel.rs`; add `file_copy_clipboard: Option<FileCopyClipboard>` and `paste_progress: Option<PasteProgress>` and `pending_conflicts: VecDeque<PasteConflict>` fields to `FileExplorerPanelState`
    - Validates: Requirement 21.1, 21.11
  - [x] 24.2 Wire Ctrl+C in the file list: when `selected_nodes` is non-empty, build `FileCopyClipboard` and store it; also write Text_Tree to OS clipboard (reuse Req 19.5 path)
    - Validates: Requirement 21.1
  - [x] 24.3 Implement `determine_paste_target(cursor_node: &str, state: &FileExplorerPanelState) -> Option<String>` — returns the target directory path (container → itself; non-container → parent)
    - Validates: Requirement 21.2
  - [x] 24.4 Wire Ctrl+V in the file list: read `file_copy_clipboard`, call `determine_paste_target`, check POSIX read-only guard, check for name collisions (push to `pending_conflicts`), dispatch `ff-bgio` copy tasks for non-conflicting files, initialise `paste_progress`
    - Validates: Requirement 21.2, 21.3, 21.4, 21.10
  - [x] 24.5 Render `pending_conflicts` queue: show per-file conflict modal with Overwrite / Skip / Rename options; dispatch chosen action; advance queue
    - Validates: Requirement 21.5
  - [x] 24.6 Wire Ctrl+V in the editor: when editor tab is active and `file_copy_clipboard` is non-empty, open Paste_Prompt modal; on "Insert File Names" insert one path per line at caret; on "Insert File Contents" read each file and insert with blank-line separator; skip unreadable files with inline error
    - Validates: Requirement 21.6, 21.7, 21.8
  - [x] 24.7 Implement Mainframe paste support: DSN/member path stored in clipboard; when pasting to Native/POSIX target, apply member-name lowercasing transformation (reuse Req 16.12.4 logic)
    - Validates: Requirement 21.9
  - [x] 24.8 Render "pending paste" dashed border on source nodes whose paths are in `file_copy_clipboard.paths`
    - Validates: Requirement 21.11
  - [x] 24.9 Write unit tests:
    - `ctrl_c_in_file_list_populates_file_copy_clipboard` — Ctrl+C stores paths in clipboard
    - `determine_paste_target_container_returns_self` — container node → itself as target
    - `determine_paste_target_file_returns_parent` — file node → parent dir as target
    - `paste_to_posix_catalog_is_rejected` — Ctrl+V to POSIX target stores error, no copy dispatched
    - `paste_conflict_detection_identifies_existing_file` — collision detected when target path exists
    - `ctrl_v_in_editor_with_clipboard_opens_paste_prompt` — Ctrl+V in editor opens prompt when clipboard non-empty
    - `insert_file_names_produces_one_path_per_line` — "Insert File Names" inserts correct text
    - Validates: Requirement 21.1–21.11

- [x] 26. File Explorer Panel — egui-file-dialog look-and-feel with catalog mount points (Phase BM, Requirement 23)
  - [x] 26.1 Add `selected_catalog: Option<String>` and `sidebar_width: f32` fields to `FileExplorerPanelState`; initialise `sidebar_width` to `200.0` in `Default`
    - Validates: Requirement 23.2, 23.9
  - [x] 26.2 Refactor `render()` into `render_sidebar()` + `render_content_pane()` using `SidePanel::left` / `CentralPanel`; wire `selected_catalog` selection in sidebar
    - Validates: Requirement 23.1, 23.2
  - [x] 26.3 Implement sidebar grouping: three `CollapsingHeader` sections ("Mainframe", "POSIX", "Native") each listing their catalogs as `selectable_label` rows with type icon prefix; highlight selected node
    - Validates: Requirement 23.3
  - [x] 26.4 Implement content pane dispatch: when `selected_catalog` is `Some`, look up catalog type and call `render_native_dialog()` / `render_mainframe_content()` / `render_posix_content()`
    - Validates: Requirement 23.4, 23.5, 23.6
  - [x] 26.5 Implement `render_mainframe_content()`: render datasets from `files_panel.datasets` with dot-separated names; PDS as `CollapsingHeader`, PS as `selectable_label`; double-click routes through existing VFS path resolution
    - Validates: Requirement 23.5
  - [x] 26.6 Implement `render_posix_content()`: read from `std::fs::read_dir` on catalog repository path; display paths with forward-slash separators; directories as `CollapsingHeader`, files as `selectable_label`; double-click opens in editor
    - Validates: Requirement 23.6
  - [x] 26.7 Implement empty sidebar state: when no catalogs registered, show placeholder message in sidebar; content pane empty
    - Validates: Requirement 23.7
  - [x] 26.8 Persist `sidebar_width` in session state; restore on launch; enforce 120px minimum
    - Validates: Requirement 23.9
  - [x] 26.9 Write unit tests:
    - `selected_catalog_field_exists_on_state` — field present and defaults to None
    - `sidebar_width_defaults_to_200` — default width is 200.0
    - `posix_path_uses_forward_slashes` — path normalisation converts backslash to forward slash
    - `mainframe_dataset_node_kind_dispatch` — PO → container, PS → leaf (reuses existing `dataset_node_kind` tests)
    - `empty_registry_shows_placeholder` — zero catalogs triggers placeholder path
    - Validates: Requirement 23.1–23.10
  - [x] 26.10 Run `cargo test` — all tests pass, 0 failures
    - Validates: Requirement 23.10

- [x] 25. Native file browser — egui-file-dialog integration (Phase BK, Requirement 22)
  - [x] 25.1 Add `egui-file-dialog = "0.6"` to `crates/ff-desktop/Cargo.toml` [dependencies]; confirm `cargo check` passes
    - Validates: Requirement 22.4
  - [x] 25.2 Add `native_dialogs: HashMap<String, NativeDialogSlot>` field to `FileExplorerPanelState`; initialise to empty map in `Default`; `NativeDialogSlot` newtype provides `Debug` + `Clone`
    - Validates: Requirement 22.1
  - [x] 25.3 Implement `render_native_dialog(ui, catalog_name, catalog_path, state, open_path)` — lazily creates a `FileDialog` for the catalog if absent, calls `dialog.update(ctx)`, checks `dialog.take_selected()` and sets `*open_path`
    - Validates: Requirement 22.1, 22.2
  - [x] 25.4 Replace the `render_native_children(...)` call in the Native catalog branch of `render()` with `render_native_dialog(...)`
    - Validates: Requirement 22.1, 22.3
  - [x] 25.5 Confirm Mainframe and POSIX branches of `render()` are untouched; `render_dataset_children()` call unchanged
    - Validates: Requirement 22.3
  - [x] 25.6 Create `THIRD_PARTY_CREDITS.md` at workspace root with entry for `egui-file-dialog`
    - Validates: Requirement 22.5
  - [x] 25.7 Run `cargo test` — 486 tests pass, 0 failures
    - Validates: Requirement 22.6
  - [x] 25.8 Run `cargo clippy -- -D warnings` — no new lint violations
  - [x] 25.9 Run `cargo build --release` — binary builds successfully (29s)
