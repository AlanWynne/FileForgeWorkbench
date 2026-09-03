# Requirements Document

## Introduction

This feature specifies the File Tree Panel for FileForgeWorkbench (`ff-file-tree` crate). The File Tree Panel is a **unified resource explorer** that renders all registered VFS providers as a multi-root tree hierarchy in a dockable panel. It provides visual browsing of local files, mounted dataset catalogs, and future remote connections through a single, consistent tree interface.

The panel is a `DockablePanel` implementation (registered with the layout-and-docking system) that defaults to the left dock zone. It renders a multi-root tree with three top-level sections:

1. **Local Files** — workspace and project directories served by the `connector-local-fs` provider
2. **Catalogs** — mounted dataset catalogs served by the `dataset-catalog` VFS provider, displaying datasets as a tree with PDS member navigation
3. **Connections** — placeholder node for future remote VFS providers (FTP, SFTP, z/OS, cloud)

The tree supports asynchronous directory loading (non-blocking node expansion), file watching for live updates, drag-and-drop to the editor (open file), context menus, keyboard navigation, a search/filter box, file icons by type/extension, and file category colouring from the theme palette.

All resource access flows through the VFS abstraction layer (FFW-ARCH-001) — the panel never performs direct filesystem I/O. Tree operations (open, rename, delete, new file, new folder) are dispatched as commands through the command framework.

**Source references:**
- **[FFE-TREE]** = FileForgeEditor `file-tree-panel` specification (left-docked panel, async loading, colour coding, context menu, keyboard nav, search)
- **[DSC]** = Dataset Catalog Brief (catalog tree, PDS member browsing, dataset properties)
- **[WB]** = Workbench Architecture Brief (VFS-unified explorer, dockable panel, command-driven operations)

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `virtual-file-system` | **Dependency** | All resource browsing, listing, stat, watch, and search operations go through the VFS API. The panel never calls `std::fs` or provider-specific APIs directly. |
| `connector-local-fs` | **Provider** | Supplies the Local Files tree root with directory listing, file watching, and metadata for local filesystem resources. |
| `dataset-catalog` | **Provider** | Supplies the Catalogs tree root with catalog listing, dataset enumeration, PDS member navigation, and dataset properties. |
| `connector-extensibility` | **Integration** | Future providers register with the VFS and automatically appear under the Connections root node. |
| `layout-and-docking` | **Integration** | The panel implements `DockablePanel` trait, participates in dock/undock/show/hide/minimize operations. |
| `theme-and-appearance` | **Consumer** | Obtains file-category colours, selection highlight, tree-node foreground/background, icon tinting from the theme palette `file_tree` colour group. |
| `command-framework` | **Integration** | All tree operations (open, rename, delete, new file, new folder, copy path, reveal, refresh) are registered commands invoked through the command dispatch. |
| `configuration-system` | **Consumer** | Reads panel configuration (enabled, default width, default roots, sort order, show hidden files) from layered configuration. |

## Glossary

- **File_Tree_Panel**: The dockable panel component (`ff-file-tree` crate) that renders a multi-root hierarchical tree of resources from all registered VFS providers. [FFE-TREE, WB]
- **Tree_Root**: A top-level node in the tree representing a VFS provider category or a specific mounted root path. The panel defines three static root categories: Local Files, Catalogs, and Connections. [WB]
- **Tree_Node**: A single entry in the tree hierarchy, representing a file, directory, dataset, PDS member, catalog, or placeholder. Each node has a label, icon, expansion state, and optional children. [FFE-TREE]
- **Local_Files_Root**: The tree root section containing workspace/project directories sourced from the `connector-local-fs` VFS provider. Supports multiple bookmarked root paths. [FFE-TREE, WB]
- **Catalogs_Root**: The tree root section containing all mounted dataset catalogs sourced from the `dataset-catalog` VFS provider. Each mounted catalog appears as a child node with its datasets as sub-tree. [DSC, WB]
- **Connections_Root**: A placeholder tree root section for future remote VFS providers (FTP, SFTP, z/OS, cloud). Displays a "No connections configured" message until providers register. [WB]
- **Bookmarked_Root**: A user-defined directory path saved as a persistent root under Local Files. Multiple bookmarks can coexist, each showing its own subtree. [FFE-TREE]
- **Node_Expansion**: The act of loading and displaying a node's children. For directories, this triggers an async VFS list operation. [FFE-TREE]
- **Loading_Indicator**: A visual spinner or animated icon displayed on a node while its children are being asynchronously loaded. [FFE-TREE]
- **File_Category**: A classification of files for colour-coding purposes: NonEditableBinary, FileForgeStructured, StandardText, Unknown, Directory, SymbolicLink. [FFE-TREE]
- **Context_Menu**: A right-click popup menu offering operations contextual to the selected tree node type. [FFE-TREE, DSC]
- **Tree_Search_Box**: A text input at the top of the panel that filters visible tree nodes by name pattern, auto-expanding ancestors of matching nodes. [FFE-TREE]
- **Path_Bar**: A text input showing the current root path, editable for direct navigation, with a browse button for native folder picker. [FFE-TREE]
- **Structure_Badge**: A small visual indicator (icon overlay or label) on FileForgeStructured files indicating they have an associated structure definition. [FFE-TREE]
- **Refresh_Command**: A command that forces the panel to re-read directory contents from VFS providers, discarding cached state. [WB]

## Requirements

### Requirement 1: Panel Layout and Docking

**User Story:** As a user, I want the file tree to appear as a resizable, dockable panel in the left area of the workbench, so that I can browse resources alongside my editor without switching views.

**Source:** [FFE-TREE] Left-docked resizable panel; [WB] DockablePanel integration.

#### Acceptance Criteria

1. THE File_Tree_Panel SHALL implement the `DockablePanel` trait from the `layout-and-docking` crate, with `panel_id` returning `"file_tree"` and `default_dock_zone` returning `DockZone::Left`.
2. WHEN the workbench starts with the File_Tree_Panel enabled (configuration key `file_tree.enabled` is `true`, default: `true`), THE panel SHALL be visible in its assigned dock zone.
3. THE File_Tree_Panel SHALL have a default width of 260 logical pixels and a resizable range constrained between 120 and 600 logical pixels.
4. THE File_Tree_Panel SHALL persist its width to the layout state so that the user's preferred width is restored on subsequent startups.
5. WHEN the user collapses the File_Tree_Panel, THE panel SHALL minimize to an icon strip (panel title and collapse/expand toggle) occupying no more than 28 logical pixels of width.
6. THE collapse/expand state SHALL be persisted in the layout state and restored on startup.
7. THE File_Tree_Panel SHALL render a title bar displaying "Explorer" with a refresh button and a collapse/expand toggle.
8. IF the configuration key `file_tree.enabled` is set to `false`, THEN THE File_Tree_Panel SHALL not register with the Panel_Registry at startup.

---

### Requirement 2: Multi-Root Tree Hierarchy

**User Story:** As a user, I want a unified tree view with multiple root categories (Local Files, Catalogs, Connections) so that I can browse all my resources from a single panel regardless of their storage backend.

**Source:** [WB] VFS-unified explorer; [DSC] Catalog tree; [FFE-TREE] Bookmarked roots.

#### Acceptance Criteria

1. THE File_Tree_Panel SHALL display three top-level root categories, rendered as expandable section headers: "Local Files", "Catalogs", and "Connections".
2. THE "Local Files" root SHALL enumerate all Bookmarked_Root paths configured by the user, each rendered as an expandable directory node showing its filesystem subtree via the `connector-local-fs` VFS provider.
3. IF no Bookmarked_Root paths are configured, THE "Local Files" root SHALL display the process working directory (or the configured `file_tree.default_root` path) as a single default root.
4. THE "Catalogs" root SHALL enumerate all currently mounted dataset catalogs from the `dataset-catalog` VFS provider, each rendered as an expandable node with its datasets as children.
5. IF no catalogs are mounted, THE "Catalogs" root SHALL display a child node with the label "No catalogs mounted" (non-expandable, greyed).
6. THE "Connections" root SHALL enumerate any registered remote VFS providers. IF no remote providers are registered, THE node SHALL display a child node with the label "No connections configured" (non-expandable, greyed).
7. EACH root category section header SHALL be independently expandable/collapsible, with persisted expansion state.
8. THE user SHALL be able to add a new Bookmarked_Root under "Local Files" via a context menu action or toolbar button, which opens a native folder picker dialog.
9. THE user SHALL be able to remove or rename a Bookmarked_Root via context menu, with the change persisted immediately.
10. Bookmarked_Root paths SHALL be persisted in the configuration system under the key `file_tree.bookmarked_roots` as an ordered array of path strings.

---

### Requirement 3: Async Directory Loading

**User Story:** As a user, I want directory expansion to load content asynchronously so that the UI remains responsive even when browsing slow directories or remote resources.

**Source:** [FFE-TREE] Lazy async directory loading; [WB] Async I/O principle.

#### Acceptance Criteria

1. WHEN the user expands a directory/container node, THE File_Tree_Panel SHALL initiate an async VFS `list` operation and SHALL NOT block the UI thread while waiting for results.
2. WHILE the async list operation is in progress, THE expanded node SHALL display a Loading_Indicator (animated spinner icon with the text "Loading...") as its only visible child.
3. WHEN the async list operation completes successfully, THE File_Tree_Panel SHALL replace the Loading_Indicator with the returned directory entries, sorted according to the current sort order.
4. IF the async list operation fails (VFS error), THE File_Tree_Panel SHALL replace the Loading_Indicator with an error node displaying the error message in a muted/error colour, and log the error at WARN level.
5. THE File_Tree_Panel SHALL cache the children of expanded nodes in memory to avoid re-loading on collapse/re-expand within the same session, unless a refresh or file-watch event invalidates the cache.
6. WHEN the user collapses a node, THE File_Tree_Panel SHALL retain the cached children but free any associated pending async operations for deeper unexpanded subtrees.
7. THE async loading SHALL support cancellation: IF the user collapses a node before its loading completes, THE File_Tree_Panel SHALL cancel the pending VFS operation.
8. THE File_Tree_Panel SHALL limit concurrent async directory loads to a maximum of 8 simultaneous operations to prevent resource exhaustion.

---

### Requirement 4: Tree Node Rendering and Sort Order

**User Story:** As a user, I want tree nodes displayed with appropriate icons, labels, and colours so that I can quickly identify file types and find what I need.

**Source:** [FFE-TREE] File colour coding, sort order; [WB] Theme palette integration.

#### Acceptance Criteria

1. THE File_Tree_Panel SHALL sort child nodes within each directory with directories listed before files, and within each group alphabetically case-insensitive.
2. THE user SHALL be able to change the sort order via a configuration key `file_tree.sort_order` supporting values: `directories_first` (default), `alphabetical`, `type`, and `modified_date`.
3. EACH Tree_Node SHALL display an icon appropriate to its type: folder icon for directories, file-type icon for files (determined by extension), catalog icon for catalog roots, dataset icon for datasets, member icon for PDS members, and a generic icon for unknown types.
4. THE File_Tree_Panel SHALL obtain file icons from a built-in icon set with at minimum distinct icons for: folder (open/closed), text file, source code file (by language family), binary file, image file, configuration file, dataset (sequential), dataset (partitioned), PDS member, catalog, and connection.
5. EACH file node SHALL be rendered with a foreground colour determined by its File_Category, obtained from the `file_tree` colour group in the Theme_Palette:
   - `file_tree.non_editable_binary` for binary/non-editable files
   - `file_tree.fileforge_structured` for files with associated structure definitions
   - `file_tree.standard_text` for regular text files
   - `file_tree.unknown` for unrecognised file types
   - `file_tree.directory` for directory nodes
   - `file_tree.symbolic_link` for symbolic link nodes
6. FILES classified as FileForgeStructured SHALL display a Structure_Badge (small overlay icon or inline label) indicating they have an associated structure definition from the File_Association_Map.
7. THE File_Tree_Panel SHALL respect the configuration key `file_tree.show_hidden_files` (default: `false`). WHEN `false`, files and directories whose names start with `.` (Unix) or have the hidden attribute (Windows) SHALL be omitted from the tree display.
8. WHEN a directory contains more than 10,000 visible entries, THE File_Tree_Panel SHALL display only the first 1,000 entries followed by a "... and N more items" indicator node, with a "Show All" action to reveal the remainder.

---

### Requirement 5: File Watching and Live Updates

**User Story:** As a user, I want the tree to automatically reflect filesystem changes (new files, deletions, renames) so that I always see an accurate view without manually refreshing.

**Source:** [FFE-TREE] Live file watching; [WB] VFS file-watcher integration.

#### Acceptance Criteria

1. WHEN a directory node is expanded (visible in the tree), THE File_Tree_Panel SHALL register a VFS watch on that directory to receive change notifications.
2. WHEN a `Created` watch event is received for a watched directory, THE File_Tree_Panel SHALL insert the new entry into the correct sorted position within that node's children and update the display.
3. WHEN a `Deleted` watch event is received for a resource in a watched directory, THE File_Tree_Panel SHALL remove the corresponding node from the tree.
4. WHEN a `Renamed` watch event is received, THE File_Tree_Panel SHALL update the node's label to the new name and re-sort its position within the parent.
5. WHEN a `Modified` watch event is received for a file, THE File_Tree_Panel SHALL update the node's metadata (size, modification time) if displayed.
6. WHEN a directory node is collapsed, THE File_Tree_Panel SHALL cancel the VFS watch for that directory to conserve system resources.
7. IF the VFS provider for a root does not support the `watch` capability, THE File_Tree_Panel SHALL not attempt to register watches for that subtree and SHALL log a DEBUG-level message.
8. THE File_Tree_Panel SHALL apply a debounce window of 200 milliseconds to watch events for the same directory, batching rapid changes into a single tree update to avoid excessive re-rendering.

---

### Requirement 6: Context Menus

**User Story:** As a user, I want right-click context menus on tree nodes offering relevant operations so that I can perform file management tasks directly from the tree.

**Source:** [FFE-TREE] Context menus (Open, Copy path, Reveal); [DSC] Dataset context menus; [WB] Command-driven operations.

#### Acceptance Criteria

1. WHEN the user right-clicks a file node under "Local Files", THE File_Tree_Panel SHALL display a context menu with the following items: Open, Open With..., Rename, Delete, New File (sibling), New Folder (sibling), Copy Path, Reveal in System Explorer.
2. WHEN the user right-clicks a directory node under "Local Files", THE File_Tree_Panel SHALL display a context menu with: Expand/Collapse, New File (child), New Folder (child), Rename, Delete, Copy Path, Reveal in System Explorer, Refresh.
3. WHEN the user right-clicks a dataset node under "Catalogs", THE File_Tree_Panel SHALL display a context menu with: Open, Rename, Delete, Properties, Copy DSN.
4. WHEN the user right-clicks a PDS dataset node under "Catalogs", THE File_Tree_Panel SHALL display an extended context menu including: Expand/Collapse, New Member, Rename, Delete, Properties, Copy DSN.
5. WHEN the user right-clicks a PDS member node under "Catalogs", THE File_Tree_Panel SHALL display a context menu with: Open, Rename, Delete, Copy DSN.
6. WHEN the user right-clicks a catalog root node, THE File_Tree_Panel SHALL display a context menu with: Unmount, Refresh, Properties.
7. WHEN the user right-clicks the "Local Files" section header, THE File_Tree_Panel SHALL display a context menu with: Add Root Folder, Refresh All.
8. ALL context menu actions SHALL be dispatched as commands through the command framework (e.g., `file_tree.open`, `file_tree.rename`, `file_tree.delete`, `file_tree.new_file`, `file_tree.new_folder`, `file_tree.copy_path`, `file_tree.reveal_in_explorer`, `file_tree.refresh`).
9. WHEN the user selects "Delete" from a context menu, THE File_Tree_Panel SHALL display a confirmation dialog ("Delete {name}? This action cannot be undone.") before dispatching the delete command.
10. WHEN the user selects "Rename" from a context menu, THE File_Tree_Panel SHALL activate an inline text editor on the node's label, pre-filled with the current name, allowing the user to type a new name and confirm with Enter or cancel with Escape.

---

### Requirement 7: Drag-and-Drop to Editor

**User Story:** As a user, I want to drag a file from the tree into the editor area to open it, so that I can use natural gesture-based file opening.

**Source:** [FFE-TREE] Double-click/Enter to open; [WB] Drag-to-editor integration.

#### Acceptance Criteria

1. WHEN the user double-clicks a file node (or PDS member node), THE File_Tree_Panel SHALL dispatch the `file.open` command with the resource's VFS URI, opening the file in the active Tab_Group.
2. WHEN the user presses Enter on a selected file node, THE File_Tree_Panel SHALL dispatch the `file.open` command with the resource's VFS URI.
3. WHEN the user initiates a drag on a file node and drops it onto the editor area (a Tab_Group tab bar or an editor view), THE File_Tree_Panel SHALL dispatch the `file.open` command targeting the drop Tab_Group.
4. WHILE a drag from the tree is in progress over the editor area, THE editor area SHALL display a drop indicator showing where the file tab will appear.
5. WHEN the user initiates a drag on a file node and drops it outside any valid target (editor area, tab bar), THE drag SHALL be cancelled with no action taken.
6. WHEN the user single-clicks a file node, THE File_Tree_Panel SHALL select that node (highlight it) and display the resource's full VFS URI or path in the status bar.
7. WHEN the user double-clicks a directory node, THE File_Tree_Panel SHALL toggle its expansion state (expand if collapsed, collapse if expanded).

---

### Requirement 8: Keyboard Navigation

**User Story:** As a user, I want full keyboard navigation within the tree panel so that I can browse and open files efficiently without the mouse.

**Source:** [FFE-TREE] Keyboard navigation (arrows, Enter, expand/collapse).

#### Acceptance Criteria

1. WHEN the File_Tree_Panel has keyboard focus and the user presses the Down Arrow key, THE selection SHALL move to the next visible node below the current selection.
2. WHEN the File_Tree_Panel has keyboard focus and the user presses the Up Arrow key, THE selection SHALL move to the previous visible node above the current selection.
3. WHEN the user presses the Right Arrow key on a collapsed directory/container node, THE File_Tree_Panel SHALL expand that node (triggering async loading if not cached).
4. WHEN the user presses the Right Arrow key on an already-expanded node, THE selection SHALL move to the first child node.
5. WHEN the user presses the Left Arrow key on an expanded directory/container node, THE File_Tree_Panel SHALL collapse that node.
6. WHEN the user presses the Left Arrow key on a child node (file or collapsed directory), THE selection SHALL move to the parent node.
7. WHEN the user presses Enter on a file node, THE File_Tree_Panel SHALL open the file (equivalent to double-click).
8. WHEN the user presses Enter on a directory node, THE File_Tree_Panel SHALL toggle its expansion state.
9. WHEN the user presses Delete on a selected node, THE File_Tree_Panel SHALL trigger the delete confirmation dialog (same as context menu Delete).
10. WHEN the user presses F2 on a selected node, THE File_Tree_Panel SHALL activate inline rename (same as context menu Rename).
11. WHEN the user presses Home, THE selection SHALL move to the first visible node in the tree. WHEN the user presses End, THE selection SHALL move to the last visible node.
12. WHEN the user types alphanumeric characters while the tree has focus (and the search box is not focused), THE File_Tree_Panel SHALL perform incremental type-ahead search, jumping to the next sibling node whose label starts with the typed prefix.

---

### Requirement 9: Search and Filter

**User Story:** As a user, I want to filter the tree by name so that I can quickly locate files in large directory structures without manual browsing.

**Source:** [FFE-TREE] Tree_Search_Box: live filter by name, auto-expand matching ancestors.

#### Acceptance Criteria

1. THE File_Tree_Panel SHALL display a Tree_Search_Box input field at the top of the panel (below the title bar, above the tree content).
2. WHEN the user types into the Tree_Search_Box, THE File_Tree_Panel SHALL filter the tree to show only nodes whose label contains the search text (case-insensitive substring match).
3. WHEN a filter is active, THE File_Tree_Panel SHALL automatically expand all ancestor nodes of matching nodes, so that matches are always visible.
4. WHEN a filter is active, non-matching nodes that are not ancestors of a match SHALL be hidden from view.
5. WHEN the user clears the Tree_Search_Box (backspace to empty or click the clear button), THE File_Tree_Panel SHALL restore the full unfiltered tree, preserving the expansion state that existed before the filter was applied.
6. THE Tree_Search_Box SHALL support glob pattern matching when the input contains `*` or `?` characters (e.g., `*.rs` matches all Rust source files).
7. THE filter SHALL operate on the cached tree data without triggering new VFS operations for unexpanded directories.
8. WHEN the user presses Ctrl+Shift+E while the workbench has focus, THE keyboard focus SHALL move to the Tree_Search_Box within the File_Tree_Panel.
9. WHEN the user presses Escape while the Tree_Search_Box has focus, THE focus SHALL return to the tree node list and the search filter SHALL be cleared.

---

### Requirement 10: Dataset Catalog Browsing

**User Story:** As a mainframe developer, I want to browse mounted dataset catalogs in the tree panel, expanding PDS datasets to see their members, so that I can work with mainframe-style datasets using the same explorer interface as local files.

**Source:** [DSC] PDS member browsing, dataset tree, catalog mount; [WB] VFS unified explorer.

#### Acceptance Criteria

1. EACH mounted catalog SHALL appear as an expandable child node under the "Catalogs" root, labelled with the catalog name.
2. WHEN the user expands a catalog node, THE File_Tree_Panel SHALL list all datasets in that catalog via the `dataset-catalog` VFS provider, grouped by High_Level_Qualifier (HLQ) as intermediate directory-like nodes.
3. Sequential datasets (DSORG=PS) SHALL be rendered as leaf file nodes with a sequential-dataset icon.
4. Partitioned datasets (DSORG=PO) SHALL be rendered as expandable folder-like nodes with a PDS icon. WHEN expanded, THE File_Tree_Panel SHALL list all members of the PDS.
5. PDS member nodes SHALL be rendered as leaf file nodes with a member icon, labelled with the 1–8 character member name.
6. Generation Data Groups (DSORG=GDG) SHALL be rendered as expandable nodes with a GDG icon. WHEN expanded, THE File_Tree_Panel SHALL list all active generations sorted by generation number (newest first).
7. THE File_Tree_Panel SHALL display dataset properties (RECFM, LRECL, DSORG) as a tooltip when the user hovers over a dataset node for at least 500 milliseconds.
8. WHEN the user selects "Properties" from a dataset's context menu, THE File_Tree_Panel SHALL open a Properties panel (or dialog) displaying full dataset attributes: DSN, DSORG, RECFM, LRECL, BLKSIZE, creation date, modification date, physical path.
9. DATASET nodes SHALL use the `file_tree.fileforge_structured` colour from the theme palette to distinguish them from regular local files.
10. THE File_Tree_Panel SHALL support opening a PDS member by double-click or Enter, dispatching `file.open` with the VFS URI `vfs://catalog/{catalog-name}/{DSN}({member})`.

---

### Requirement 11: Path Bar and Navigation

**User Story:** As a user, I want a path bar where I can type a directory path to jump directly to it, so that I can navigate quickly without expanding the tree manually.

**Source:** [FFE-TREE] Path_Bar: type to navigate, Browse button.

#### Acceptance Criteria

1. THE File_Tree_Panel SHALL display a Path_Bar above (or integrated with) the Tree_Search_Box, showing the path of the currently focused root or selected node.
2. WHEN the user clicks the Path_Bar, THE input SHALL become editable with the full path text selected, allowing the user to type a new path.
3. WHEN the user presses Enter after editing the Path_Bar, THE File_Tree_Panel SHALL navigate to the typed path: expanding the tree to reveal that location if it exists under a current root, or adding it as a temporary root if it doesn't.
4. IF the path entered in the Path_Bar does not exist (VFS stat returns NotFound), THE File_Tree_Panel SHALL display a brief inline error message "Path not found" and revert to the previous path.
5. THE Path_Bar SHALL display a Browse button (folder icon) that opens the native OS folder picker dialog. WHEN the user selects a folder, THE File_Tree_Panel SHALL navigate to that folder.
6. THE Path_Bar path resolution SHALL go through the VFS layer, supporting `vfs://` URIs in addition to bare local paths.

---

### Requirement 12: Refresh Command

**User Story:** As a user, I want a refresh action that forces the tree to reload from the VFS providers, so that I can manually update the view if file watching misses an external change.

**Source:** [WB] Refresh command; [FFE-TREE] Manual refresh.

#### Acceptance Criteria

1. THE File_Tree_Panel SHALL register a `file_tree.refresh` command with the command framework that invalidates all cached directory listings and re-loads all currently expanded nodes from their VFS providers.
2. WHEN the user triggers the refresh command (via toolbar button, context menu, or keyboard shortcut F5 while the panel has focus), THE File_Tree_Panel SHALL clear the children cache for all expanded nodes and re-issue async list operations.
3. DURING a refresh, nodes that are currently expanded SHALL display a Loading_Indicator on each refreshing node while their content is being reloaded.
4. THE File_Tree_Panel SHALL also support per-node refresh: WHEN the user selects "Refresh" from a directory node's context menu, ONLY that directory and its expanded descendants SHALL be reloaded (not the entire tree).
5. AFTER a refresh completes, THE File_Tree_Panel SHALL preserve the user's selection position if the previously selected node still exists. IF the selected node was deleted, THE selection SHALL move to the nearest sibling or parent.

---

### Requirement 13: Configuration

**User Story:** As a user, I want configurable tree panel behaviour (default root, sort order, hidden files, width) so that the panel adapts to my workflow and project structure.

**Source:** [FFE-TREE] Config keys (enabled, width, default root); [WB] Configuration-system integration.

#### Acceptance Criteria

1. THE File_Tree_Panel SHALL read the following configuration keys from the configuration-system:
   - `file_tree.enabled` (bool, default: `true`) — whether the panel is registered at startup
   - `file_tree.default_width` (integer, default: `260`) — initial panel width in logical pixels
   - `file_tree.default_root` (string, optional) — initial root path used when no bookmarked roots exist
   - `file_tree.bookmarked_roots` (array of strings, default: `[]`) — persisted bookmark paths
   - `file_tree.sort_order` (string, default: `"directories_first"`) — sort mode for tree nodes
   - `file_tree.show_hidden_files` (bool, default: `false`) — whether to show hidden files/directories
2. THE File_Tree_Panel SHALL participate in configuration hot-reload: WHEN any `file_tree.*` configuration key changes at runtime, THE panel SHALL apply the new value without requiring application restart.
3. WHEN the `file_tree.show_hidden_files` value changes, THE File_Tree_Panel SHALL immediately re-filter all currently displayed nodes to show or hide hidden entries.
4. WHEN the `file_tree.sort_order` value changes, THE File_Tree_Panel SHALL re-sort all currently displayed directory contents according to the new order.
5. THE File_Tree_Panel SHALL support per-project overrides for all configuration keys, following the configuration-system layering (user → project → profile).

---

### Requirement 14: Accessibility and Visual Feedback

**User Story:** As a user (including those using assistive technology), I want the tree panel to provide clear visual focus indicators, proper ARIA semantics, and high-contrast support so that the panel is usable by all.

**Source:** [WB] Workbench accessibility; [FFE-TREE] Visual feedback.

#### Acceptance Criteria

1. THE currently selected Tree_Node SHALL be highlighted with a distinct background colour obtained from the theme palette (`ui.selection_background`) that meets WCAG AA contrast requirements against adjacent nodes.
2. WHEN the File_Tree_Panel has keyboard focus, THE focused node SHALL display a visible focus ring or border distinct from the selection highlight.
3. THE File_Tree_Panel SHALL expose tree semantics to assistive technology: tree role, tree-item roles, expanded/collapsed state, level depth, and set size/position within set.
4. ALL icons in the tree SHALL have associated text labels (used for tooltip and accessibility name) describing the node type and state.
5. WHEN in High-Contrast visual mode, ALL tree node colours, icons, and selection indicators SHALL meet WCAG AAA contrast ratios (7:1 minimum) as enforced by the theme system.
6. THE File_Tree_Panel SHALL render indent guides (vertical lines connecting parent to child nodes) using the theme colour `ui.indent_guide` to visually communicate hierarchy depth.
7. TREE nodes representing resources currently open in an editor tab SHALL display a subtle "open" indicator (e.g., a dot or modified background) so the user can identify which files are already open.

---

### Requirement 15: Native Catalog Recursive Directory Expansion and Scrollable Panel

**User Story:** As a user, I want to expand subdirectory nodes within a Native catalog in the File Explorer so that I can browse nested folder structures, and I want the panel to be scrollable so that I can page through large directory listings.

**Source:** CR-NR-005 — user request Phase AY.

#### Acceptance Criteria

1. WHEN the user clicks the expand arrow on a directory node inside a Native catalog, THE File_Explorer_Panel SHALL display that directory's children (subdirectories and files) as nested child nodes, sorted directories-first then alphabetically.
2. WHEN a directory node is expanded, THE child nodes SHALL themselves be expandable if they are directories, supporting arbitrary nesting depth.
3. THE File_Explorer_Panel content area SHALL be wrapped in a vertical scroll region so that the user can scroll to see entries that extend beyond the visible panel height.

---

### Requirement 16: File Explorer Context Menu

**User Story:** As a user, I want a right-click context menu on any node in the File Explorer Panel so that I can perform file operations directly from the tree without leaving the workbench.

**Source:** CR-NR-006 — Phase AZ.

#### Glossary additions

- **Context_Menu_Group**: A set of related menu items separated from adjacent groups by a horizontal divider.
- **Greyed_Out**: A menu item rendered in a disabled/muted style that is visible but not clickable. Used for deferred features.
- **Inline_Rename**: An editable text field that replaces the node label in the tree, pre-filled with the current name.
- **Copy_To_Dialog**: A modal dialog that lets the user pick a target catalog/directory and confirm or edit the proposed name after naming-rule transformation.
- **Move_To_Dialog**: Identical to Copy_To_Dialog but deletes the source after a successful copy.
- **Progress_Indicator**: A non-blocking overlay or status-bar entry showing background I/O progress via `ff-bgio`.
- **Extension_Rule**: A data-driven record mapping a glob pattern (e.g. `*.jcl`) to a set of menu item overrides (enable, disable, add).

#### Acceptance Criteria

**1. Trigger**

1. WHEN the user right-clicks any non-section-header node in the File Explorer Panel, THE panel SHALL display a Context_Menu appropriate to that node's catalog type and node kind.
2. WHEN the user right-clicks a section header node ("Mainframe Catalogs", "POSIX Catalogs", "Native Catalogs"), THE panel SHALL display no context menu (section headers are not actionable).
3. THE Context_Menu SHALL be dismissed when the user clicks outside it, presses Escape, or selects an item.

**2. Native File context menu**

WHEN the user right-clicks a file node inside a Native catalog, THE panel SHALL display the following items in order, with dividers as shown:

```
Open
Open in New Tab
Open in New Window
Open With…
────────────────
Copy
────────────────
Rename
Move To…
Copy To…
────────────────
New File
New Folder
────────────────
Copy File Name
Copy Relative Path
Copy Full Path
────────────────
Open Containing Folder
Reveal in Explorer
────────────────
Git ▶  [Greyed_Out — deferred]
────────────────
Properties
```

**3. Native Directory context menu**

WHEN the user right-clicks a directory node inside a Native catalog, THE panel SHALL display:

```
Open in New Tab
────────────────
New File
New Folder
────────────────
Copy
────────────────
Rename
Move To…
Copy To…
────────────────
Copy Full Path
────────────────
Reveal in Explorer
────────────────
Git ▶  [Greyed_Out — deferred]
────────────────
Properties
```

**4. POSIX File context menu**

WHEN the user right-clicks a file node inside a POSIX catalog (read-only), THE panel SHALL display:

```
Open
Open in New Tab
Open in New Window
Open With…
────────────────
Copy
────────────────
Copy File Name
Copy Relative Path
Copy Full Path
────────────────
Properties
```

No Rename, Move To, Copy To, New File, New Folder — POSIX catalogs are read-only.

**5. Mainframe Sequential Dataset (PS) context menu**

WHEN the user right-clicks a PS dataset node inside a Mainframe catalog, THE panel SHALL display:

```
Open
Open in New Tab
────────────────
Compare…
────────────────
Copy To…
────────────────
Copy Dataset Name
Copy Full Path
────────────────
Dataset Properties
────────────────
Refresh
```

**6. Mainframe PDS / Library context menu**

WHEN the user right-clicks a PDS dataset node inside a Mainframe catalog, THE panel SHALL display:

```
New Member
────────────────
Copy To…
────────────────
Copy Dataset Name
────────────────
Dataset Properties
────────────────
Refresh
```

**7. Mainframe PDS Member context menu**

WHEN the user right-clicks a PDS member node inside a Mainframe catalog, THE panel SHALL display:

```
Open
Open in New Tab
────────────────
Submit JCL  [Greyed_Out — deferred pending SDSF]
Compare…
────────────────
Copy Member
Rename Member
────────────────
Copy Member Name
Copy Dataset Name
Copy Dataset(Member)
────────────────
Member Properties
Dataset Properties
────────────────
Refresh
```

**8. Mainframe GDG Base context menu**

WHEN the user right-clicks a GDG Base node inside a Mainframe catalog, THE panel SHALL display:

```
New Generation
────────────────
Copy Dataset Name
────────────────
Dataset Properties
────────────────
Refresh
```

**9. Mainframe GDG Generation context menu**

WHEN the user right-clicks a GDG Generation node inside a Mainframe catalog, THE panel SHALL display:

```
Open
Open in New Tab
────────────────
Copy Dataset Name
────────────────
Dataset Properties
────────────────
Refresh
```

**10. Copy action**

1. WHEN the user selects "Copy" from any context menu, THE panel SHALL write the node's full path (Native/POSIX) or fully-qualified dataset name (Mainframe) as plain UTF-8 text to the OS clipboard.
2. WHEN the user subsequently pastes into the FFWB Command_Field, THE full path string SHALL be inserted as plain text.
3. WHEN the user subsequently pastes into an open FFWB editor tab, THE panel SHALL display a modal prompt with two options: **"Insert File Name"** and **"Insert File Contents"**.
4. WHEN the user selects "Insert File Name" from the paste prompt, THE full path string SHALL be inserted at the caret position.
5. WHEN the user selects "Insert File Contents" from the paste prompt, THE file's text content SHALL be read and inserted at the caret position as if via the COPY file-insert command.
6. WHEN the user selects "Insert File Contents" and the file cannot be read, THE panel SHALL display an inline error and make no document change.

**11. Rename (inline)**

1. WHEN the user selects "Rename" or "Rename Member" from a context menu, THE panel SHALL replace the node's label with an editable text field pre-filled with the current name.
2. WHEN the user presses Enter in the inline rename field, THE panel SHALL attempt to rename the resource: for Native files/directories this renames on disk; for Mainframe PDS members this renames within the dataset store.
3. WHEN the user presses Escape in the inline rename field, THE rename SHALL be cancelled and the original label restored with no change to disk or store.
4. WHEN renaming a Mainframe PDS member, THE panel SHALL enforce 8-character uppercase naming: if the entered name exceeds 8 characters or contains invalid characters, THE panel SHALL display an inline error beneath the field and SHALL NOT confirm the rename until the name is valid.
5. WHEN a rename completes successfully, THE tree node label SHALL update to the new name in place.

**12. Move To… and Copy To… dialogs**

1. WHEN the user selects "Move To…" or "Copy To…", THE panel SHALL open a modal dialog containing: a target catalog/directory picker, a proposed new name field (pre-filled and transformed per target naming rules), and Confirm / Cancel buttons.
2. WHEN the target is a Mainframe PDS, THE dialog SHALL automatically uppercase and truncate the proposed name to 8 characters, stripping invalid characters, and display the transformed name to the user before confirmation.
3. WHEN the target is a Native or POSIX directory, THE dialog SHALL apply OS filename rules with no automatic transformation.
4. WHEN the target is a Native directory and the source is a Mainframe member, THE dialog SHALL propose the member name lowercased with no extension, and allow the user to edit it.
5. THE user SHALL be able to edit the proposed name in the dialog before confirming.
6. WHEN the edited name violates the target's naming rules, THE dialog SHALL display an inline error and SHALL NOT allow confirmation until the name is valid.
7. WHEN the user confirms, THE operation SHALL be dispatched to `ff-bgio` as a background task with a Progress_Indicator shown in the status bar.
8. WHEN the background operation completes successfully, THE Progress_Indicator SHALL be dismissed and the tree SHALL refresh the affected nodes.
9. WHEN the background operation fails, THE panel SHALL display an error message and leave the source node unchanged.
10. For "Move To…", THE source node SHALL only be removed from the tree after the background copy confirms success.

**13. Open With…**

1. WHEN the user selects "Open With…" on Windows, THE panel SHALL invoke the native Windows "Open with" dialog via `ShellExecuteEx` with verb `"openwith"`.
2. WHEN the user selects "Open With…" on macOS, THE panel SHALL invoke `open -a` to present the Finder application picker.
3. WHEN the user selects "Open With…" on Linux, THE panel SHALL display a built-in FFWB "Choose Application" dialog listing applications discovered via `xdg-mime query default` and `/usr/share/applications/*.desktop` entries, with a text field for a custom command.

**14. Reveal in Explorer / Open Containing Folder**

1. WHEN the user selects "Reveal in Explorer" (Windows) / "Reveal in Finder" (macOS) / "Open Containing Folder" (Linux), THE panel SHALL open the OS file manager at the parent directory of the selected node with the node highlighted where the platform supports it.
2. THE label for this item SHALL be platform-appropriate: "Reveal in Explorer" on Windows, "Reveal in Finder" on macOS, "Open Containing Folder" on Linux.

**15. Git submenu (deferred)**

1. THE context menu for Native file and directory nodes SHALL include a "Git ▶" submenu item rendered as Greyed_Out.
2. THE Git submenu SHALL NOT be interactive in this release; clicking it SHALL have no effect.
3. The submenu items (Diff, History, Commit, Blame) SHALL be visible but greyed-out to communicate future intent.

**16. Submit JCL (deferred)**

1. THE context menu for Mainframe PDS member nodes SHALL include a "Submit JCL" item rendered as Greyed_Out.
2. "Submit JCL" SHALL NOT be interactive in this release; clicking it SHALL have no effect.

**17. Extension rules**

1. THE panel SHALL maintain a data-driven `Vec<ExtensionRule>` table mapping glob patterns to menu item overrides (enable, disable, add extra items).
2. In this release the table SHALL be defined in code; no TOML configuration is required yet.
3. The table SHALL be structured so that future TOML-driven configuration can replace or extend it without code changes.
4. WHEN a file's extension matches an Extension_Rule that enables a previously Greyed_Out item (e.g. `*.jcl` enabling Submit JCL), THE item SHALL become active for that node.

**18. Copy path variants**

1. "Copy File Name" SHALL write only the file's base name (no directory) to the OS clipboard.
2. "Copy Relative Path" SHALL write the path relative to the catalog's root path to the OS clipboard.
3. "Copy Full Path" SHALL write the absolute path to the OS clipboard.
4. "Copy Dataset Name" SHALL write the fully-qualified dataset name (e.g. `PAYROLL.DATA`) to the OS clipboard.
5. "Copy Member Name" SHALL write only the member name (e.g. `MYJOB`) to the OS clipboard.
6. "Copy Dataset(Member)" SHALL write the combined form (e.g. `PAYROLL.JCL(MYJOB)`) to the OS clipboard.

---

### Requirement 17: Open With Default Application

**User Story:** As a user, I want double-clicking a file node or selecting "Open" from the context menu to launch the platform-appropriate default application for that file type, so that non-text files (Word documents, spreadsheets, PDFs, images, etc.) open in the correct program rather than in the FFWB text editor.

**Source:** CR-NR-007 — Phase BA.

#### Glossary additions

- **FileClass**: A classification of a file node that determines whether it opens inside FFWB (`Text`, `FfwbStructured`) or in the OS default application (`External`).
- **MagicByteScan**: Reading the first 512 bytes of a file to detect binary content (presence of null bytes or high proportion of non-UTF-8 bytes).
- **DefaultAppLaunch**: A non-blocking `std::process::Command::spawn()` call that hands the file to the OS for opening.

#### Acceptance Criteria

**1. Text files open in FFWB editor**

WHEN the user opens a file whose extension maps to `FileClass::Text` or `FileClass::FfwbStructured`, THE panel SHALL open the file in a new FFWB editor tab (existing behaviour, no change).

**2. External files launch OS default application**

WHEN the user opens a file whose extension maps to `FileClass::External`, THE panel SHALL perform a DefaultAppLaunch using the platform mechanism:
- Windows: `ShellExecuteEx` with verb `"open"` and the file path, via `std::process::Command::new("cmd").args(["/c", "start", "", path])`
- macOS: `std::process::Command::new("open").arg(path).spawn()`
- Linux: `std::process::Command::new("xdg-open").arg(path).spawn()`

**3. Unknown extensions use magic-byte fallback**

WHEN a file's extension has no matching `FileClass` rule, THE panel SHALL perform a MagicByteScan of the first 512 bytes. IF the file appears to be valid UTF-8 text (no null bytes, fewer than 5% non-UTF-8 bytes), THE panel SHALL open it in the FFWB editor. OTHERWISE THE panel SHALL perform a DefaultAppLaunch.

**4. Launch failure falls back to FFWB editor**

WHEN a DefaultAppLaunch fails (spawn returns an error, or on Linux `xdg-open` exits non-zero), THE panel SHALL open the file in the FFWB editor tab and display a status-bar message: `"No application registered for .<ext> — opened in editor"`.

**5. Open With… shows platform picker**

WHEN the user selects "Open With…" from the context menu:
- Windows: `std::process::Command::new("cmd").args(["/c", "start", "", path])` with the `openwith` shell verb via `ShellExecuteEx`
- macOS: `std::process::Command::new("open").args(["-a", path]).spawn()`
- Linux: THE panel SHALL display a built-in FFWB "Choose Application" dialog listing applications from `xdg-mime query default` and `/usr/share/applications/*.desktop` entries, with a free-text field for a custom command

**6. Non-blocking launch**

THE DefaultAppLaunch SHALL use `Command::spawn()` (fire-and-forget). THE panel SHALL NOT wait for the launched application to exit. THE UI thread SHALL NOT block.

**7. Mainframe datasets always open in FFWB**

WHEN the user opens any Mainframe catalog node (PS dataset, PDS member, GDG generation), THE panel SHALL always open the content in a FFWB editor tab regardless of the node's name or content. DefaultAppLaunch does not apply to Mainframe nodes.

**8. FileClass extension table**

THE `ExtensionRule` table  SHALL include a `file_class` field on each rule. The following extensions SHALL be pre-classified as `FileClass::External`:

| Category | Extensions |
|---|---|
| Microsoft Office | `docx`, `xlsx`, `pptx`, `doc`, `xls`, `ppt`, `odt`, `ods`, `odp` |
| PDF / eBook | `pdf`, `epub`, `mobi` |
| Images | `png`, `jpg`, `jpeg`, `gif`, `bmp`, `tiff`, `webp`, `svg`, `ico` |
| Audio / Video | `mp3`, `mp4`, `wav`, `flac`, `avi`, `mkv`, `mov`, `wmv` |
| Archives | `zip`, `tar`, `gz`, `bz2`, `xz`, `7z`, `rar` |
| Executables | `exe`, `dll`, `so`, `dylib`, `app` |
| Database | `db`, `sqlite`, `mdb`, `accdb` |

All other extensions default to `FileClass::Text` unless the magic-byte scan detects binary content (Req 17.3).

**9. POSIX catalog nodes follow same rules**

WHEN the user opens a POSIX catalog file node, THE same FileClass classification and DefaultAppLaunch rules SHALL apply as for Native catalog file nodes.



---

### Requirement 18: Native Catalog File Listing — Sorted Order and File Attributes

**User Story:** As a user, I want the files in a Native catalog directory to be sorted alphabetically and to see file attributes (size, timestamps, permissions) alongside each file name, so that I can quickly find files and understand their state without leaving the workbench.

**Source:** CR-NR-008 — Phase BB.

#### Acceptance Criteria

**1. Sort order**

WHEN a Native catalog directory node is expanded, THE File_Explorer_Panel SHALL display its children sorted directories-first then files, with each group sorted alphabetically case-insensitive by file name.

**2. File size**

EACH file node in a Native catalog SHALL display the file size in a human-readable format (e.g. `1.2 KB`, `3.4 MB`, `512 B`). Directories SHALL display `<DIR>` in the size column.

**3. Created timestamp**

EACH file and directory node in a Native catalog SHALL display the file creation timestamp in `YYYY-MM-DD HH:MM` format.

**4. Modified timestamp**

EACH file and directory node in a Native catalog SHALL display the last-modified timestamp in `YYYY-MM-DD HH:MM` format.

**5. Accessed timestamp**

EACH file and directory node in a Native catalog SHALL display the last-accessed timestamp in `YYYY-MM-DD HH:MM` format.

**6. Permission attributes**

EACH file and directory node in a Native catalog SHALL display permission attributes in a user-friendly format:
- On Windows: a compact flag string showing `R` (read-only), `H` (hidden), `S` (system), `A` (archive) where set; e.g. `RH`, `A`, or `—` if none.
- On Linux/macOS: a compact Unix-style string e.g. `rwxr-xr-x`.

**7. Inaccessible entries silently skipped**

WHEN `std::fs::metadata()` returns an error for an entry (e.g. permission denied on a junction point or locked file), THE File_Explorer_Panel SHALL silently skip that entry — it SHALL NOT appear in the listing and SHALL NOT display an error message for that individual entry. The remaining entries in the directory SHALL still be listed normally.

**8. Locked-file open error**

WHEN the user attempts to open a file that is locked by another process (OS error 32 on Windows), THE File_Explorer_Panel SHALL display a status-bar message: `"Cannot open '<filename>': file is in use by another process"` and SHALL NOT open an editor tab for that file.

**9. Column layout**

THE file attribute columns SHALL be rendered to the right of the file name in the following order: Size, Modified, Created, Accessed, Permissions. Columns SHALL be right-aligned for Size and left-aligned for timestamps and permissions.

---

### Requirement 19: File Explorer Tree — Drag-Select and Copy as Text Tree

**User Story:** As a user, I want to drag-select a range of nodes in the File Explorer tree and copy them as a formatted plain-text tree structure to the clipboard, so that I can paste the file listing into a text file, editor tab, or external tool.

**Source:** CR-NR-009 — Phase BD.

#### Glossary additions

- **Drag_Selection**: A contiguous range of visible tree nodes selected by pressing and holding the left mouse button on a start node and dragging to an end node.
- **Text_Tree**: A plain-text representation of the selected nodes rendered with ASCII indentation that mirrors the tree hierarchy.
- **Anchor_Node**: The node where a drag-selection or Shift-click selection begins.

#### Acceptance Criteria

**1. Drag-select gesture**

WHEN the user presses and holds the left mouse button on a tree node and drags to another node, THE File_Explorer_Panel SHALL highlight all visible nodes between the start node and the current cursor position (inclusive) as a Drag_Selection.

**2. Shift-click extend**

WHEN the user holds Shift and clicks a tree node, THE File_Explorer_Panel SHALL extend the current selection from the Anchor_Node to the clicked node, replacing any previous selection.

**3. Ctrl-click toggle**

WHEN the user holds Ctrl and clicks a tree node, THE File_Explorer_Panel SHALL toggle that node's membership in the current selection without affecting other selected nodes.

**4. Selection highlight**

EACH selected node SHALL be rendered with the theme selection background colour (`ui.selection_background`) so the selection is clearly visible.

**5. Copy selection to clipboard (Ctrl+C)**

WHEN one or more nodes are selected and the user presses Ctrl+C (or selects "Copy as Text Tree" from the context menu), THE File_Explorer_Panel SHALL build a Text_Tree string from the selected nodes and write it to the OS clipboard as plain UTF-8 text.

**6. Text_Tree format**

THE Text_Tree string SHALL be formatted as follows:
- Each node is rendered on its own line.
- Indentation uses two spaces per depth level relative to the shallowest selected node (depth 0 = no indent).
- Directory nodes are prefixed with `[DIR] ` followed by their name.
- File nodes are rendered with their name only (no prefix).
- When the selection includes both a parent directory and its children, tree connector characters (`├── `, `└── `, `│   `) SHALL be used to show the hierarchy.
- When only leaf nodes are selected with no parent-child relationship among the selection, each is listed on its own line with its full relative path from the catalog root.

**7. Context menu item**

WHEN one or more nodes are selected and the user right-clicks, THE context menu SHALL include a "Copy as Text Tree" item above the existing "Copy" group. Selecting it SHALL perform the same action as Ctrl+C (Req 19.5).

**8. Escape clears selection**

WHEN the user presses Escape, THE current multi-node selection SHALL be cleared and the panel SHALL revert to single-node selection mode (the most recently clicked node remains selected).

**9. Selection survives scroll**

WHEN the user scrolls the panel while a drag-selection is active, THE selection SHALL extend to include nodes scrolled into view under the cursor.

**10. Mainframe and POSIX nodes included**

THE drag-select and copy behaviour SHALL apply equally to Native, POSIX, and Mainframe catalog nodes. For Mainframe nodes the Text_Tree SHALL use the fully-qualified dataset name (DSN) in place of a file path.

---

### Requirement 20: File Explorer — Keyboard Navigation and Focus Transfer from Command Line

**User Story:** As a user, I want to press Tab from the command line to move focus into the File Explorer tree, then navigate with arrow keys and select items with Shift+Arrow and Ctrl+Space, so that I can operate the file list entirely from the keyboard.

**Source:** CR-NR-010 — Phase BE.

#### Glossary additions

- **Explorer_Focus**: The keyboard focus state where the File Explorer Panel's node list is the active input target.
- **Keyboard_Selection**: One or more nodes highlighted as selected via keyboard gestures (Shift+Arrow, Ctrl+Space).
- **Cursor_Node**: The node that currently has the keyboard cursor (focus ring), independent of the selection set.

#### Acceptance Criteria

**1. Tab from command line enters the file list**

WHEN the File Explorer Panel is the active tab and the user presses Tab while the Command_Field has focus, THE keyboard focus SHALL transfer to the File Explorer node list and the Cursor_Node SHALL be set to the first visible catalog name node.

**2. Tab advances through nodes**

WHEN the File Explorer node list has Explorer_Focus and the user presses Tab, THE Cursor_Node SHALL advance to the next visible node in display order (wrapping from last to first).

**3. Tab on a container node expands it**

WHEN the user presses Tab and the Cursor_Node is a container (directory, catalog, PDS dataset, GDG base) that is currently collapsed, THE container SHALL be expanded to show its children before the Cursor_Node advances to the next node.

**4. Arrow keys move the cursor without expanding**

WHEN the File Explorer node list has Explorer_Focus and the user presses the Down Arrow or Up Arrow key, THE Cursor_Node SHALL move one position in the corresponding direction. Container nodes SHALL NOT be expanded by arrow key movement alone.

**5. Left and Right Arrow keys on containers**

WHEN the user presses the Right Arrow key on a collapsed container node, THE container SHALL expand (same as existing Req 8.3). WHEN the user presses the Left Arrow key on an expanded container node, THE container SHALL collapse (same as existing Req 8.5). WHEN the user presses the Left Arrow key on a non-container or collapsed node, THE Cursor_Node SHALL move to the parent node (same as existing Req 8.6).

**6. Shift+Arrow extends the keyboard selection**

WHEN the user holds Shift and presses Down Arrow or Up Arrow, THE Cursor_Node SHALL move one position in the corresponding direction AND the node moved onto SHALL be added to the Keyboard_Selection. The Anchor_Node is set on the first Shift+Arrow press if no selection exists.

**7. Shift+Arrow selection is cumulative**

WHEN the user continues to hold Shift and press Arrow keys, EACH newly visited node SHALL be added to the Keyboard_Selection. Nodes already in the selection SHALL remain selected.

**8. Releasing Shift preserves the selection**

WHEN the user releases the Shift key, THE Keyboard_Selection SHALL remain highlighted. Subsequent plain Arrow key presses SHALL move the Cursor_Node without changing the selection.

**9. Ctrl+Arrow moves cursor without changing selection**

WHEN the user holds Ctrl and presses Down Arrow or Up Arrow, THE Cursor_Node SHALL move one position in the corresponding direction WITHOUT adding or removing any node from the Keyboard_Selection.

**10. Ctrl+Space toggles selection of the cursor node**

WHEN the user holds Ctrl and presses Space, THE Cursor_Node's membership in the Keyboard_Selection SHALL toggle: if it was not selected it becomes selected; if it was selected it becomes deselected. All other selected nodes are unaffected.

**11. Ctrl+C copies selected nodes**

WHEN one or more nodes are in the Keyboard_Selection and the user presses Ctrl+C, THE File_Explorer_Panel SHALL copy the selected nodes' information to the OS clipboard (as per Req 19.5 — Text_Tree format for text paste, and as per Req 21.1 for file-level copy).

**12. Escape clears keyboard selection**

WHEN the user presses Escape while the File Explorer node list has Explorer_Focus, THE Keyboard_Selection SHALL be cleared. The Cursor_Node SHALL remain on the current node as a single-node selection.

**13. Visual distinction between cursor and selection**

THE Cursor_Node SHALL be rendered with a focus ring or border distinct from the selection highlight. Selected nodes SHALL use the theme `ui.selection_background` fill. A node that is both the Cursor_Node and selected SHALL show both indicators simultaneously.

---

### Requirement 21: File Explorer — File Copy and Paste Operations

**User Story:** As a user, I want to select files in the File Explorer, press Ctrl+C to copy them, navigate to a new directory, and press Ctrl+V to copy the files there. I also want to be able to paste the list of selected file names into a file I am editing.

**Source:** CR-NR-011 — Phase BE.

#### Glossary additions

- **File_Copy_Clipboard**: An internal clipboard payload (separate from the OS text clipboard) that holds a list of source file paths and the operation type (Copy or Cut).
- **Paste_Target**: The directory node that is currently selected or focused when Ctrl+V is pressed.
- **Paste_Prompt**: A modal dialog shown when the user pastes into an editor tab, offering the choice between inserting file names or file contents.

#### Acceptance Criteria

**1. Ctrl+C copies selected file paths to the internal clipboard**

WHEN one or more nodes are selected in the File Explorer and the user presses Ctrl+C, THE File_Explorer_Panel SHALL store the selected nodes' full paths (Native/POSIX) or fully-qualified DSNs (Mainframe) in the File_Copy_Clipboard with operation type `Copy`.

**2. Ctrl+V in the file list pastes files to the current directory**

WHEN the File_Copy_Clipboard is non-empty and the user presses Ctrl+V while the File Explorer node list has Explorer_Focus, THE File_Explorer_Panel SHALL determine the Paste_Target as follows: if the Cursor_Node is a directory/container, use it as the target; otherwise use the Cursor_Node's parent directory. THE panel SHALL then dispatch a background copy operation (via `ff-bgio`) for each source path to the Paste_Target directory.

**3. Paste progress indicator**

WHEN a paste operation is in progress, THE File_Explorer_Panel SHALL display a Progress_Indicator in the status bar showing the number of files copied and the total. WHEN all copies complete, THE Progress_Indicator SHALL be dismissed and the Paste_Target directory SHALL be refreshed in the tree.

**4. Paste failure handling**

WHEN a background copy fails for one or more files (e.g. permission denied, disk full, name collision), THE File_Explorer_Panel SHALL display an error message in the status bar listing the failed file names and reasons. Successfully copied files SHALL NOT be rolled back.

**5. Name collision on paste**

WHEN a file being pasted has the same name as an existing file in the Paste_Target directory, THE File_Explorer_Panel SHALL display a per-file prompt with options: **Overwrite**, **Skip**, **Rename** (appends `_copy` suffix or increments a counter). The user's choice applies to that file only; subsequent collisions prompt again.

**6. Paste into editor inserts file list**

WHEN the File_Copy_Clipboard is non-empty and the user presses Ctrl+V while an editor tab has focus, THE File_Explorer_Panel SHALL display a Paste_Prompt modal with two options: **\"Insert File Names\"** and **\"Insert File Contents\"**.

**7. Insert File Names**

WHEN the user selects \"Insert File Names\" from the Paste_Prompt, THE full path (or DSN) of each selected file SHALL be inserted at the caret position in the editor, one path per line.

**8. Insert File Contents**

WHEN the user selects \"Insert File Contents\" from the Paste_Prompt, THE text content of each selected file SHALL be read and inserted at the caret position in the editor, files separated by a blank line. If any file cannot be read, that file is skipped and an inline error is shown; other files are still inserted.

**9. Mainframe datasets supported**

THE copy and paste operations SHALL support Mainframe catalog nodes. Copying a Mainframe PS dataset or PDS member SHALL store the DSN/member path in the File_Copy_Clipboard. Pasting to a Native or POSIX directory SHALL use the member name (lowercased, no extension) as the target file name, following the same naming-rule transformation as the Copy To… dialog (Req 16.12.4).

**10. POSIX catalogs are read-only for paste**

WHEN the Paste_Target is within a POSIX catalog, THE File_Explorer_Panel SHALL reject the paste operation and display a status-bar message: `\"Cannot paste: POSIX catalog '<name>' is read-only\"`.

**11. File_Copy_Clipboard persists until replaced or cleared**

THE File_Copy_Clipboard SHALL persist across navigation within the File Explorer until the user performs a new Ctrl+C, presses Escape to clear the selection, or closes the application. A visual indicator (e.g. dashed border on the source nodes) SHALL show which nodes are pending paste.

---

### Requirement 23: File Explorer Panel — egui-file-dialog Look-and-Feel with Catalog Mount Points

**User Story:** As a user, I want the File Explorer Panel (POM option 2) to look and work like the egui-file-dialog widget, with each catalog appearing as a mounted node in a left sidebar, so that I can browse all catalog types through a single, consistent, polished interface.

**Source:** CR-NR-014 — Phase BM.

#### Glossary additions

- **Sidebar**: The left pane of the File Explorer Panel listing all mounted catalogs as named nodes, analogous to the "Places" panel in egui-file-dialog.
- **Content_Pane**: The right pane of the File Explorer Panel showing the files/datasets belonging to the currently selected catalog node.
- **Mount_Node**: A catalog entry rendered in the Sidebar. Clicking it populates the Content_Pane with that catalog’s contents.
- **Mainframe_Listing**: A flat or hierarchical list of dataset names rendered with dot-separated qualifiers (e.g. `PAYROLL.EMPLOYEE`) in the Content_Pane for Mainframe catalogs.
- **POSIX_Listing**: A file/folder tree rendered with forward-slash paths (e.g. `/home/user/docs`) in the Content_Pane for POSIX catalogs.
- **Native_Browser**: The egui-file-dialog widget embedded in the Content_Pane for Native catalogs (unchanged from Requirement 22).

#### Acceptance Criteria

**1. Two-pane layout**

WHEN the File Explorer Panel is open, THE panel SHALL render a two-pane layout: a fixed-width left Sidebar and a resizable right Content_Pane, separated by a visible splitter. The overall visual style SHALL match the egui-file-dialog widget as closely as possible.

**2. Sidebar lists all catalogs as mount nodes**

THE Sidebar SHALL list every registered catalog (Mainframe, POSIX, and Native) as a named Mount_Node. Each node SHALL display the catalog name and a type icon (mainframe icon, POSIX icon, or folder icon for Native). Clicking a Mount_Node SHALL select it and populate the Content_Pane with that catalog’s contents.

**3. Sidebar groups by catalog type**

Mount_Nodes in the Sidebar SHALL be grouped under three collapsible section headers: "Mainframe", "POSIX", and "Native". Each section SHALL be independently collapsible. The selected Mount_Node SHALL be highlighted with the theme selection colour.

**4. Native catalog: egui-file-dialog in Content_Pane**

WHEN a Native catalog Mount_Node is selected, THE Content_Pane SHALL render the `egui-file-dialog` widget initialised to that catalog’s repository path (reusing the existing `render_native_dialog()` implementation from Requirement 22). File selection SHALL open the file in a new editor tab.

**5. Mainframe catalog: dot-qualified dataset listing in Content_Pane**

WHEN a Mainframe catalog Mount_Node is selected, THE Content_Pane SHALL display the catalog’s allocated datasets as a list. Each dataset SHALL be rendered with its fully-qualified name using dot separators (e.g. `PAYROLL.EMPLOYEE`). PDS datasets (DSORG=PO) SHALL be rendered as expandable nodes; sequential datasets (DSORG=PS) as leaf nodes. Double-clicking a PS dataset or PDS member SHALL attempt to open it in the editor via the existing VFS path resolution.

**6. POSIX catalog: file/folder tree with forward-slash paths in Content_Pane**

WHEN a POSIX catalog Mount_Node is selected, THE Content_Pane SHALL display the catalog’s files and directories as a tree. Directory nodes SHALL be expandable. All paths displayed SHALL use forward slashes as separators regardless of the host OS (e.g. `/home/user/docs/file.txt`). Files SHALL be rendered as leaf nodes. Double-clicking a file SHALL open it in the editor.

**7. Empty sidebar state**

WHEN no catalogs are registered, THE Sidebar SHALL display a placeholder message: "No catalogs mounted — use File Catalogs (option 1) to create or mount a catalog". The Content_Pane SHALL be empty.

**8. Right-click context menu uses egui-file-dialog native menu**

FOR this release, right-clicking any node in the Content_Pane SHALL use the context menu provided natively by the egui-file-dialog widget for Native catalogs. For Mainframe and POSIX nodes, the existing context menu from Requirement 16 SHALL continue to apply. No new custom context menu items are added in this requirement.

**9. Sidebar width persistence**

THE Sidebar width SHALL be persisted in the session state and restored on next launch. The default Sidebar width SHALL be 200 logical pixels. The minimum Sidebar width SHALL be 120 logical pixels.

**10. All existing tests continue to pass**

WHEN the refactoring is complete, `cargo test` SHALL pass with 0 failures. No existing test SHALL be removed or modified to accommodate the change.

---

### Requirement 22: Native File Browser — egui-file-dialog Integration

> **Note:** Requirement 22 was added after Requirement 23 during Phase BK. The numbering reflects implementation order and is preserved for test annotation compatibility.

**User Story:** As a user, I want the Native catalog file browser to use a polished, feature-complete file dialog widget so that I can navigate local directories with breadcrumbs, bookmarks, search, and keyboard shortcuts without the workbench having to maintain a custom tree renderer.

**Source:** CR-NR-013 — Phase BK.

#### Glossary additions

- **egui_file_dialog**: The third-party egui widget crate (`egui-file-dialog`) that provides a ready-made file/folder picker rendered inside an egui panel or window.
- **Native_Browser**: The portion of the File Explorer Panel that renders the contents of a Native catalog. Replaced by the `egui-file-dialog` widget in this requirement.
- **Dataset_Browser**: The portion of the File Explorer Panel that renders Mainframe and POSIX catalog datasets. Unchanged by this requirement.

#### Acceptance Criteria

**1. egui-file-dialog replaces render_native_children**

WHEN the user expands a Native catalog node in the File Explorer Panel, THE panel SHALL render the `egui-file-dialog` widget initialised to the catalog's repository path, replacing the previous `render_native_children()` recursive tree renderer.

**2. File selection opens in editor**

WHEN the user selects a file in the `egui-file-dialog` widget and confirms (double-click or Enter), THE File_Explorer_Panel SHALL dispatch the selected file's absolute path to the existing `open_path` handler, opening the file in a new editor tab (same behaviour as the previous double-click handler).

**3. Mainframe and POSIX browsing unchanged**

THE `render_dataset_children()` function and all Mainframe/POSIX catalog rendering logic SHALL remain unchanged. The `egui-file-dialog` widget SHALL only be used for Native catalog nodes.

**4. Dependency declared in Cargo.toml**

THE `ff-desktop` crate's `Cargo.toml` SHALL declare `egui-file-dialog` as a dependency with a pinned version. The workspace `Cargo.toml` SHALL NOT need to be modified (the dependency is local to `ff-desktop`).

**5. Licence credited**

THE `THIRD_PARTY_CREDITS.md` file at the workspace root SHALL contain an entry for `egui-file-dialog` listing its crate name, version, author, and licence (MIT).

**6. All existing tests continue to pass**

WHEN the refactoring is complete, `cargo test` SHALL pass with 0 failures. No existing test SHALL be removed or modified to accommodate the change.
