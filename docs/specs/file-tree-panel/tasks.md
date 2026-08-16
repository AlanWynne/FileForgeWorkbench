# Implementation Plan: File Tree Panel (`ff-file-tree-panel`)

## Overview

This task plan implements the `ff-file-tree-panel` crate — the unified resource explorer panel for FileForgeWorkbench. It renders all registered VFS providers as a multi-root tree hierarchy in a dockable panel, supporting asynchronous directory loading, file watching, drag-and-drop, context menus, keyboard navigation, search/filter, dataset catalog browsing, and configurable appearance.

**Crate location:** `crates/ff-file-tree-panel`
**Upstream dependencies:** `ff-vfs` (Wave 3), `ff-layout` (Wave 2), `ff-config` (Wave 2), `ff-command` (Wave 2), `ff-theme` (Wave 6), `ff-connector-local-fs` (Wave 3), `ff-dataset-catalog` (Wave 13)
**Downstream consumers:** Application shell (ff-desktop)

---

## Tasks

- [ ] 1. Crate scaffold and core data model
  - [ ] 1.1 Create `crates/ff-file-tree-panel/Cargo.toml` with dependencies (ff-vfs, ff-layout, ff-config, ff-command, ff-theme, egui, tokio, async-trait, thiserror, serde, serde_json) and dev-dependencies (proptest, tempfile, pretty_assertions, tokio-test)
  - [ ] 1.2 Create `crates/ff-file-tree-panel/src/lib.rs` with crate-level doc comment and public module declarations
  - [ ] 1.3 Implement `src/error.rs` — define `FileTreeError` enum with variants (VfsError, ConfigError, InvalidPath, WatchError, NodeNotFound, ConcurrencyLimitReached, OperationCancelled)
  - [ ] 1.4 Implement `src/model/mod.rs` — module for tree data model types
  - [ ] 1.5 Implement `src/model/tree_node.rs` — define `TreeNode` struct (id, label, icon, node_type, expansion_state, children, parent_id, vfs_uri, file_category, metadata) and `NodeType` enum (RootCategory, Directory, File, Dataset, PdsMember, CatalogRoot, Placeholder)
  - [ ] 1.6 Implement `src/model/file_category.rs` — define `FileCategory` enum (NonEditableBinary, FileForgeStructured, StandardText, Unknown, Directory, SymbolicLink) with classification logic by extension and metadata
  - [ ] 1.7 Implement `src/model/tree_root.rs` — define `TreeRoot` enum (LocalFiles, Catalogs, Connections) and `RootCategory` struct with label, expansion state, children
  - [ ] 1.8 Implement `src/model/sort_order.rs` — define `SortOrder` enum (DirectoriesFirst, Alphabetical, Type, ModifiedDate) with `sort_nodes()` comparator function
  - [ ] 1.9 Write unit tests for FileCategory classification, SortOrder comparators, TreeNode construction
    - Validates: Requirement 4 AC 1, AC 2, AC 5

- [ ] 2. Panel layout and DockablePanel integration
  - [ ] 2.1 Implement `src/panel.rs` — define `FileTreePanel` struct implementing `DockablePanel` trait with `panel_id` returning `"file_tree"`, `default_dock_zone` returning `DockZone::Left`
  - [ ] 2.2 Implement default width of 260 logical pixels, resizable range constrained between 120 and 600 logical pixels
  - [ ] 2.3 Implement width persistence to layout state (save on resize, restore on startup)
  - [ ] 2.4 Implement collapse/expand functionality — minimizes to icon strip (≤28 px width) with title and toggle
  - [ ] 2.5 Implement collapse/expand state persistence in layout state
  - [ ] 2.6 Implement title bar rendering — display "Explorer" with refresh button and collapse/expand toggle
  - [ ] 2.7 Implement configuration gate — read `file_tree.enabled` (default: true); when false, do not register with Panel_Registry
  - [ ] 2.8 Write unit tests for DockablePanel trait implementation, width clamping, configuration gate logic
    - Validates: Requirement 1 AC 1–8

- [ ] 3. Multi-root tree hierarchy
  - [ ] 3.1 Implement `src/roots.rs` — define `RootManager` struct managing three top-level root categories (Local Files, Catalogs, Connections) as expandable section headers
  - [ ] 3.2 Implement Local Files root — enumerate all bookmarked root paths from config, each as an expandable directory node served by `connector-local-fs` VFS provider
  - [ ] 3.3 Implement fallback root — if no bookmarked roots exist, display process working directory (or `file_tree.default_root`) as single default root
  - [ ] 3.4 Implement Catalogs root — enumerate all mounted dataset catalogs from `dataset-catalog` VFS provider as expandable nodes
  - [ ] 3.5 Implement Catalogs empty state — display "No catalogs mounted" placeholder node (non-expandable, greyed)
  - [ ] 3.6 Implement Connections root — enumerate registered remote VFS providers; display "No connections configured" placeholder when none registered
  - [ ] 3.7 Implement independent section expansion/collapse with persisted state per root category
  - [ ] 3.8 Implement add bookmarked root — context menu action or toolbar button opening native folder picker
  - [ ] 3.9 Implement remove/rename bookmarked root — context menu actions with immediate config persistence
  - [ ] 3.10 Implement bookmarked roots persistence at config key `file_tree.bookmarked_roots` as ordered array
  - [ ] 3.11 Write unit tests for root enumeration, empty states, bookmark add/remove/persist, section expansion state
    - Validates: Requirement 2 AC 1–10

- [ ] 4. Async directory loading
  - [ ] 4.1 Implement `src/async_loader.rs` — define `AsyncLoader` struct managing concurrent VFS list operations with a semaphore limiting to 8 simultaneous loads
  - [ ] 4.2 Implement async node expansion — on user expand, initiate VFS `list` operation off UI thread, display Loading_Indicator (spinner + "Loading...") as child
  - [ ] 4.3 Implement successful load handling — replace Loading_Indicator with sorted directory entries on completion
  - [ ] 4.4 Implement error handling — replace Loading_Indicator with error node (muted/error colour) on VFS failure; log at WARN level
  - [ ] 4.5 Implement children cache — cache expanded node children in memory; skip VFS call on collapse/re-expand unless invalidated
  - [ ] 4.6 Implement collapse behaviour — retain cached children, cancel pending async operations for deeper unexpanded subtrees
  - [ ] 4.7 Implement cancellation — if user collapses before load completes, cancel the pending VFS operation via tokio cancellation token
  - [ ] 4.8 Implement concurrency limiter — enforce maximum 8 simultaneous async directory loads via semaphore
  - [ ] 4.9 Write unit tests for async load lifecycle, Loading_Indicator states, error node rendering, cache hits, cancellation, concurrency limit enforcement
    - Validates: Requirement 3 AC 1–8
  - [ ] 4.10 Write property test: concurrency limit invariant (Property 1) — trigger N>8 simultaneous expansions, assert at most 8 are in-flight concurrently
    - Validates: Requirement 3 AC 8

- [ ] 5. Tree node rendering and sort order
  - [ ] 5.1 Implement `src/rendering/node_renderer.rs` — render tree nodes with type-appropriate icons (folder open/closed, text file, source code, binary, image, config, dataset, PDS, member, catalog, connection)
  - [ ] 5.2 Implement file-category colour mapping — obtain foreground colours from theme palette `file_tree` colour group (non_editable_binary, fileforge_structured, standard_text, unknown, directory, symbolic_link)
  - [ ] 5.3 Implement Structure_Badge rendering — overlay icon or inline label for FileForgeStructured files with associated structure definitions
  - [ ] 5.4 Implement sort logic — default `directories_first` (dirs before files, then alphabetical case-insensitive); support `alphabetical`, `type`, `modified_date` modes via config key
  - [ ] 5.5 Implement hidden files filtering — respect `file_tree.show_hidden_files` config (default false); hide files starting with `.` or with hidden attribute
  - [ ] 5.6 Implement large directory truncation — for directories with >10,000 visible entries, display first 1,000 followed by "... and N more items" indicator with "Show All" action
  - [ ] 5.7 Write unit tests for icon selection by type/extension, colour mapping, sort correctness across modes, hidden file filtering, large directory truncation
    - Validates: Requirement 4 AC 1–8
  - [ ] 5.8 Write property test: sort order stability (Property 2) — generate random node lists, sort with each mode, verify result is totally ordered and stable (equal elements preserve insertion order)
    - Validates: Requirement 4 AC 1, AC 2
  - [ ] 5.9 Write property test: hidden file filtering completeness (Property 3) — generate nodes with mixed visibility, assert show_hidden=false excludes all hidden nodes and show_hidden=true includes all
    - Validates: Requirement 4 AC 7

- [ ] 6. File watching and live updates
  - [ ] 6.1 Implement `src/watcher.rs` — define `TreeWatcher` struct managing VFS watch registrations per expanded directory node
  - [ ] 6.2 Implement watch registration — when a directory node is expanded, register VFS watch; when collapsed, cancel the watch
  - [ ] 6.3 Implement `Created` event handling — insert new entry at correct sorted position within parent's children
  - [ ] 6.4 Implement `Deleted` event handling — remove corresponding node from tree
  - [ ] 6.5 Implement `Renamed` event handling — update node label and re-sort position within parent
  - [ ] 6.6 Implement `Modified` event handling — update node metadata (size, modification time)
  - [ ] 6.7 Implement capability check — if VFS provider does not support `watch`, skip registration and log at DEBUG level
  - [ ] 6.8 Implement debounce — batch rapid watch events for same directory within 200ms window into single tree update
  - [ ] 6.9 Write unit tests for watch registration/cancellation lifecycle, event handling for each type, debounce batching, capability check bypass
    - Validates: Requirement 5 AC 1–8
  - [ ] 6.10 Write property test: debounce batching invariant (Property 4) — generate N events within 200ms for same directory, assert exactly one tree update is applied
    - Validates: Requirement 5 AC 8

- [ ] 7. Context menus
  - [ ] 7.1 Implement `src/context_menu.rs` — define `TreeContextMenu` struct with methods returning command descriptors per node type
  - [ ] 7.2 Implement file node menu (Local Files) — Open, Open With..., Rename, Delete, New File (sibling), New Folder (sibling), Copy Path, Reveal in System Explorer
  - [ ] 7.3 Implement directory node menu (Local Files) — Expand/Collapse, New File (child), New Folder (child), Rename, Delete, Copy Path, Reveal in System Explorer, Refresh
  - [ ] 7.4 Implement dataset node menu (Catalogs) — Open, Rename, Delete, Properties, Copy DSN
  - [ ] 7.5 Implement PDS dataset node menu (Catalogs) — Expand/Collapse, New Member, Rename, Delete, Properties, Copy DSN
  - [ ] 7.6 Implement PDS member node menu (Catalogs) — Open, Rename, Delete, Copy DSN
  - [ ] 7.7 Implement catalog root node menu — Unmount, Refresh, Properties
  - [ ] 7.8 Implement Local Files section header menu — Add Root Folder, Refresh All
  - [ ] 7.9 Implement command dispatch — all actions dispatched through command framework (file_tree.open, file_tree.rename, file_tree.delete, file_tree.new_file, file_tree.new_folder, file_tree.copy_path, file_tree.reveal_in_explorer, file_tree.refresh)
  - [ ] 7.10 Implement delete confirmation dialog — "Delete {name}? This action cannot be undone." before dispatching delete command
  - [ ] 7.11 Implement inline rename — activate text editor on node label, pre-filled with current name, confirm with Enter, cancel with Escape
  - [ ] 7.12 Write unit tests for menu generation per node type, command dispatch mapping, delete confirmation flow, inline rename activation/confirm/cancel
    - Validates: Requirement 6 AC 1–10

- [ ] 8. Drag-and-drop and file opening
  - [ ] 8.1 Implement `src/interaction/open.rs` — double-click on file/member node dispatches `file.open` command with VFS URI to active Tab_Group
  - [ ] 8.2 Implement Enter key on file node dispatches `file.open` command with VFS URI
  - [ ] 8.3 Implement drag initiation — start drag on file node, carry VFS URI as payload
  - [ ] 8.4 Implement drop on editor area — dispatch `file.open` targeting the drop Tab_Group
  - [ ] 8.5 Implement drop indicator — while drag is in progress over editor area, display drop indicator showing tab placement
  - [ ] 8.6 Implement drag cancel — drop outside valid target cancels with no action
  - [ ] 8.7 Implement single-click selection — highlight node, display VFS URI/path in status bar
  - [ ] 8.8 Implement double-click on directory — toggle expansion state
  - [ ] 8.9 Write unit tests for open dispatch (double-click, Enter), drag payload construction, selection state, directory double-click toggle
    - Validates: Requirement 7 AC 1–7

- [ ] 9. Keyboard navigation
  - [ ] 9.1 Implement `src/interaction/keyboard.rs` — define `KeyboardHandler` processing key events when panel has focus
  - [ ] 9.2 Implement Down Arrow — move selection to next visible node below
  - [ ] 9.3 Implement Up Arrow — move selection to previous visible node above
  - [ ] 9.4 Implement Right Arrow on collapsed directory — expand node (trigger async load if not cached)
  - [ ] 9.5 Implement Right Arrow on expanded node — move selection to first child
  - [ ] 9.6 Implement Left Arrow on expanded directory — collapse node
  - [ ] 9.7 Implement Left Arrow on child node — move selection to parent node
  - [ ] 9.8 Implement Enter on file node — open file (same as double-click)
  - [ ] 9.9 Implement Enter on directory node — toggle expansion state
  - [ ] 9.10 Implement Delete key — trigger delete confirmation dialog
  - [ ] 9.11 Implement F2 key — activate inline rename
  - [ ] 9.12 Implement Home/End — move to first/last visible node
  - [ ] 9.13 Implement type-ahead search — alphanumeric chars jump to next sibling matching typed prefix
  - [ ] 9.14 Write unit tests for all keyboard navigation actions, boundary cases (first/last node, root-level Left Arrow), type-ahead matching
    - Validates: Requirement 8 AC 1–12
  - [ ] 9.15 Write property test: keyboard navigation order consistency (Property 5) — generate tree with N visible nodes, assert Down Arrow N-1 times visits every node exactly once in display order
    - Validates: Requirement 8 AC 1, AC 2

- [ ] 10. Search and filter
  - [ ] 10.1 Implement `src/search.rs` — define `TreeSearchBox` component rendered below title bar, above tree content
  - [ ] 10.2 Implement text filter — as user types, filter tree to show only nodes whose label contains search text (case-insensitive substring)
  - [ ] 10.3 Implement ancestor expansion — auto-expand all ancestors of matching nodes so matches are visible
  - [ ] 10.4 Implement non-match hiding — non-matching nodes that are not ancestors of a match are hidden
  - [ ] 10.5 Implement filter clear — on backspace-to-empty or clear button, restore full unfiltered tree preserving pre-filter expansion state
  - [ ] 10.6 Implement glob pattern support — when input contains `*` or `?`, use glob matching instead of substring (e.g., `*.rs` matches Rust files)
  - [ ] 10.7 Implement cached-only filter — filter operates on cached tree data without triggering new VFS operations
  - [ ] 10.8 Implement Ctrl+Shift+E shortcut — move focus to Tree_Search_Box
  - [ ] 10.9 Implement Escape from search box — return focus to tree, clear filter
  - [ ] 10.10 Write unit tests for substring matching, glob matching, ancestor expansion, filter clear state restoration, keyboard shortcuts
    - Validates: Requirement 9 AC 1–9
  - [ ] 10.11 Write property test: filter preserves tree invariants (Property 6) — generate tree and filter string, assert every visible node either matches or is ancestor of a match
    - Validates: Requirement 9 AC 2, AC 3, AC 4
  - [ ] 10.12 Write property test: glob pattern matching consistency (Property 7) — generate file names and glob patterns, verify `*` matches zero-or-more chars, `?` matches exactly one char
    - Validates: Requirement 9 AC 6

- [ ] 11. Dataset catalog browsing
  - [ ] 11.1 Implement `src/catalog_tree.rs` — render mounted catalogs as expandable child nodes under "Catalogs" root, labelled with catalog name
  - [ ] 11.2 Implement HLQ grouping — on catalog expansion, list datasets grouped by High_Level_Qualifier as intermediate directory-like nodes
  - [ ] 11.3 Implement sequential dataset rendering — DSORG=PS as leaf file nodes with sequential-dataset icon
  - [ ] 11.4 Implement PDS rendering — DSORG=PO as expandable folder-like nodes with PDS icon; on expand, list all members
  - [ ] 11.5 Implement PDS member rendering — leaf file nodes with member icon, labelled with 1–8 character member name
  - [ ] 11.6 Implement GDG rendering — DSORG=GDG as expandable nodes with GDG icon; on expand, list active generations sorted newest-first
  - [ ] 11.7 Implement dataset tooltip — display RECFM, LRECL, DSORG on hover after 500ms delay
  - [ ] 11.8 Implement Properties panel integration — context menu "Properties" opens panel/dialog with full dataset attributes (DSN, DSORG, RECFM, LRECL, BLKSIZE, creation date, modification date, physical path)
  - [ ] 11.9 Implement dataset colour — apply `file_tree.fileforge_structured` colour from theme palette to dataset nodes
  - [ ] 11.10 Implement PDS member opening — double-click/Enter dispatches `file.open` with VFS URI `vfs://catalog/{catalog-name}/{DSN}({member})`
  - [ ] 11.11 Write unit tests for catalog tree rendering, HLQ grouping, dataset type icons, member listing, GDG ordering, tooltip content, URI construction
    - Validates: Requirement 10 AC 1–10

- [ ] 12. Path bar and navigation
  - [ ] 12.1 Implement `src/path_bar.rs` — define `PathBar` component displayed above/integrated with Tree_Search_Box showing path of focused root or selected node
  - [ ] 12.2 Implement path editing — on click, input becomes editable with full path text selected
  - [ ] 12.3 Implement path navigation — on Enter, navigate to typed path: expand tree to reveal if under current root, or add as temporary root if not
  - [ ] 12.4 Implement not-found handling — if path does not exist (VFS stat returns NotFound), display inline "Path not found" error and revert
  - [ ] 12.5 Implement Browse button — folder icon opens native OS folder picker; on selection, navigate to chosen folder
  - [ ] 12.6 Implement VFS URI support — path resolution goes through VFS layer, supports `vfs://` URIs in addition to bare local paths
  - [ ] 12.7 Write unit tests for path display, edit activation, navigation success/failure, browse button flow, URI parsing
    - Validates: Requirement 11 AC 1–6

- [ ] 13. Refresh command
  - [ ] 13.1 Implement `src/commands/refresh.rs` — register `file_tree.refresh` command with command framework
  - [ ] 13.2 Implement full refresh — invalidate all cached directory listings, re-load all currently expanded nodes from VFS providers
  - [ ] 13.3 Implement refresh UI feedback — display Loading_Indicator on each refreshing node during reload
  - [ ] 13.4 Implement per-node refresh — context menu "Refresh" reloads only that directory and its expanded descendants
  - [ ] 13.5 Implement selection preservation — after refresh, preserve selection if node still exists; if deleted, move to nearest sibling or parent
  - [ ] 13.6 Implement refresh triggers — toolbar button, context menu, F5 keyboard shortcut while panel has focus
  - [ ] 13.7 Write unit tests for full refresh lifecycle, per-node refresh scope, selection preservation/fallback, trigger mechanisms
    - Validates: Requirement 12 AC 1–5

- [ ] 14. Configuration integration
  - [ ] 14.1 Implement `src/config.rs` — define `FileTreeConfig` struct reading all `file_tree.*` keys (enabled, default_width, default_root, bookmarked_roots, sort_order, show_hidden_files) from configuration-system
  - [ ] 14.2 Implement hot-reload — subscribe to `file_tree.*` config change events; apply new values without application restart
  - [ ] 14.3 Implement show_hidden_files hot-reload — immediately re-filter all displayed nodes on change
  - [ ] 14.4 Implement sort_order hot-reload — re-sort all displayed directory contents on change
  - [ ] 14.5 Implement per-project overrides — support configuration layering (user → project → profile) for all keys
  - [ ] 14.6 Write unit tests for config loading, validation, hot-reload trigger for each key, per-project override precedence
    - Validates: Requirement 13 AC 1–5

- [ ] 15. Accessibility and visual feedback
  - [ ] 15.1 Implement selection highlighting — distinct background colour from theme palette (`ui.selection_background`) meeting WCAG AA contrast
  - [ ] 15.2 Implement focus ring — visible focus indicator on focused node, distinct from selection highlight
  - [ ] 15.3 Implement tree semantics for assistive technology — tree role, tree-item roles, expanded/collapsed state, level depth, set size/position
  - [ ] 15.4 Implement icon text labels — all icons have associated text labels for tooltip and accessibility name
  - [ ] 15.5 Implement high-contrast support — in High-Contrast mode, all colours/icons/indicators meet WCAG AAA (7:1 ratio) via theme system
  - [ ] 15.6 Implement indent guides — vertical lines connecting parent to child using theme colour `ui.indent_guide`
  - [ ] 15.7 Implement open-file indicator — nodes representing files open in editor tabs show subtle "open" indicator (dot or modified background)
  - [ ] 15.8 Write unit tests for selection highlight application, focus ring rendering, indent guide depth calculation, open-file indicator state
    - Validates: Requirement 14 AC 1–7

- [ ] 16. Command registration
  - [ ] 16.1 Implement `src/commands/mod.rs` — module structure for all file tree commands
  - [ ] 16.2 Register `file_tree.open` command — open selected file in editor
  - [ ] 16.3 Register `file_tree.rename` command — activate inline rename on selected node
  - [ ] 16.4 Register `file_tree.delete` command — trigger delete confirmation and dispatch VFS delete
  - [ ] 16.5 Register `file_tree.new_file` command — create new file in selected directory (or sibling)
  - [ ] 16.6 Register `file_tree.new_folder` command — create new folder in selected directory (or sibling)
  - [ ] 16.7 Register `file_tree.copy_path` command — copy VFS URI/path to clipboard
  - [ ] 16.8 Register `file_tree.reveal_in_explorer` command — open native file explorer at node location
  - [ ] 16.9 Register `file_tree.add_root` command — open folder picker to add bookmarked root
  - [ ] 16.10 Write unit tests for command registration, parameter validation, dispatch to correct panel operations
    - Validates: Requirement 6 AC 8; Requirement 12 AC 1

- [ ] 17. Integration tests and end-to-end validation
  - [ ] 17.1 Write integration test: full panel lifecycle — create panel, verify docking, expand Local Files root, async load directory, verify nodes rendered with correct icons/colours/sort
  - [ ] 17.2 Write integration test: multi-root browsing — configure bookmarked roots and mounted catalogs, verify all three root sections display correctly
  - [ ] 17.3 Write integration test: file watching — expand directory, simulate VFS watch events (create, delete, rename), verify tree updates correctly with debounce
  - [ ] 17.4 Write integration test: search and filter — type search text, verify filter/expand/hide behaviour, clear and verify restoration
  - [ ] 17.5 Write integration test: keyboard navigation — simulate key sequences, verify selection traversal, expansion/collapse, open dispatch
  - [ ] 17.6 Write integration test: dataset catalog browsing — mount catalog, expand to HLQ/datasets/members, open PDS member, verify VFS URI
  - [ ] 17.7 Write integration test: configuration hot-reload — change sort_order and show_hidden_files at runtime, verify immediate tree update
    - Validates: All requirements end-to-end

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
