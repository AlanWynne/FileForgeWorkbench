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


