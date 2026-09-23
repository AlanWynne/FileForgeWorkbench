# Design Document: Menu and Status Bar (`ff-menu-statusbar`)

## 1. Overview

The `ff-menu-statusbar` crate provides the **menu bar, context menus, status bar, and primary command field** for the FileForgeWorkbench platform. It bridges the command framework and layout system to deliver a conventional desktop menu hierarchy and a configurable multi-segment status bar -- all without directly mutating application state.

### Purpose

- Render a standard hierarchical Menu_Bar (File, Edit, Search, View, Tools, Help) at the top of the Primary_Window
- Bind every menu item to a `CommandId` in the command framework for consistent dispatch
- Provide context menus for editor areas, tab headers, panel headers, and file tree nodes
- Display a configurable multi-segment Status_Bar at the bottom of the Primary_Window
- Implement the Primary_Command_Field ("Command ===>") for ISPF-style command entry
- Support plugin-contributed menu items, submenus, and status bar segments
- Manage the Recent_Files_List with persistence across sessions

### Position in Architecture

```
Wave 6 -- UI and Rendering (depends on Waves 0–5)

┌─────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)               │
│         Renders menu bar, status bar, command field       │
├─────────────────────────────────────────────────────────┤
│  ff-menu-statusbar (THIS CRATE) -- Wave 6                 │
│  Menu model, status segments, context menus              │
├─────────────────────────────────────────────────────────┤
│  ff-command │ ff-config │ ff-layout │ ff-plugin │ ff-core│
│  (Wave 2 -- Platform Architecture)                        │
├─────────────────────────────────────────────────────────┤
│                     ff-logging (Wave 0)                   │
└─────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **Command-Driven Architecture (Req 4)**: Every menu item dispatches via `execute_command` -- no direct state mutation
- **GUI Independence (Req 2)**: The menu/status model is GUI-independent data; only `render` trait methods accept `egui::Ui`
- **Plugin Architecture (Req 3)**: Plugins contribute menu items and status segments via traits registered through `PluginContext`
- **Configuration Namespace (Req 5)**: Status bar layout and recent files settings live under `menu.*` and `statusbar.*` namespaces
- **Status Bar Layout (Req 9)**: All active indicators visible simultaneously; single row, fixed height
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-menu-statusbar`
- **Error Message Standards (Req 8)**: Errors follow `[menu] operation: description` format

### Upstream Dependencies

| Crate | What It Provides |
|-------|------------------|
| `ff-command` | `CommandId`, `CommandRegistry`, `CommandDispatch`, `CommandMetadata`, `ShortcutBinding`, `ShortcutRegistry` |
| `ff-config` | `ConfigHandle`, typed getters, reload callbacks, `keys` module |
| `ff-layout` | `DockablePanel` trait, `DockZone`, `PanelRegistry` |
| `ff-plugin` | `FileForgePlugin`, `PluginContext`, `Capability_Registry` |
| `ff-logging` | `log_warn!`, `log_info!`, `log_debug!` macros |

### Downstream Consumers

- `ff-desktop` (GUI shell): Renders the menu bar, status bar, and command field using this crate's model and render traits
- `ff-command-semantics`: Registers ISPF commands that the Primary_Command_Field dispatches
- All plugins that contribute menu items or status segments

---

## 2. Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Shell [ff-desktop -- GUI Shell]
        RENDER_MENU[Menu Renderer]
        RENDER_STATUS[Status Bar Renderer]
        RENDER_CMD[Command Field Renderer]
    end

    subgraph ff-menu-statusbar [This Crate]
        MB[MenuBar Model<br/>menus, items, bindings]
        CM[ContextMenuRegistry<br/>per-context type menus]
        SB[StatusBarManager<br/>segments, layout]
        CF[CommandFieldController<br/>input, history]
        RF[RecentFilesManager<br/>list, persistence]
        MC[MenuContributionRegistry<br/>plugin items]
    end

    subgraph Upstream [Platform Services]
        CMD[ff-command<br/>registry, dispatch, shortcuts]
        CFG[ff-config<br/>settings, recent files]
        LAY[ff-layout<br/>panel registration]
        PLG[ff-plugin<br/>contribution API]
        LOG[ff-logging<br/>diagnostics]
    end

    RENDER_MENU -->|reads model| MB
    RENDER_STATUS -->|reads segments| SB
    RENDER_CMD -->|delegates input| CF
    MB -->|dispatches| CMD
    MB -->|reads shortcuts| CMD
    CM -->|dispatches| CMD
    CF -->|dispatches| CMD
    SB -->|reads state| CMD
    RF -->|persists| CFG
    SB -->|reads config| CFG
    MC -->|plugin items| PLG
    SB -->|registers panel| LAY
    MB --> LOG
    SB --> LOG
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **MenuBar Model** | Declarative menu tree -- headings, items, separators, submenus, bindings |
| **ContextMenuRegistry** | Per-context-type (editor, tab, panel, file-tree) menu definitions |
| **StatusBarManager** | Segment registry, ordering, alignment, content provider dispatch |
| **CommandFieldController** | Input buffering, history recall, submit-to-CommandEngine logic |
| **RecentFilesManager** | MRU list, max-size enforcement, persistence, stale-path handling |
| **MenuContributionRegistry** | Plugin menu contributions -- insertion, removal, ordering |

---

## 3. Module Structure

```
crates/ff-menu-statusbar/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports, crate docs
│   ├── menu/
│   │   ├── mod.rs                  # Menu module re-exports
│   │   ├── model.rs                # MenuBar, Menu, MenuItem, MenuSeparator data types
│   │   ├── builder.rs              # MenuBarBuilder -- declarative menu tree construction
│   │   ├── binding.rs              # MenuCommandBinding -- item ↔ CommandId association
│   │   ├── keyboard_nav.rs         # Keyboard navigation state machine (Alt keys, arrows)
│   │   └── renderer.rs             # MenuBar render trait (egui::Ui integration)
│   ├── context_menu/
│   │   ├── mod.rs                  # Context menu re-exports
│   │   ├── registry.rs             # ContextMenuRegistry -- context-type → menu mapping
│   │   ├── types.rs                # ContextType enum (Editor, Tab, Panel, FileTree)
│   │   └── renderer.rs             # Context menu popup render trait
│   ├── status/
│   │   ├── mod.rs                  # Status bar re-exports
│   │   ├── manager.rs              # StatusBarManager -- segment lifecycle, ordering
│   │   ├── segment.rs              # StatusSegment data type, SegmentAlignment
│   │   ├── provider.rs             # StatusSegmentProvider trait
│   │   ├── builtin.rs              # Built-in segment providers (mode, pos, encoding, etc.)
│   │   └── renderer.rs             # Status bar render trait
│   ├── command_field/
│   │   ├── mod.rs                  # Command field re-exports
│   │   ├── controller.rs           # CommandFieldController -- input, submit, history
│   │   ├── history.rs              # Command field history ring buffer
│   │   └── renderer.rs             # Command field render trait
│   ├── recent/
│   │   ├── mod.rs                  # Recent files re-exports
│   │   ├── manager.rs              # RecentFilesManager -- MRU list logic
│   │   └── persistence.rs          # File-based persistence (JSON in data dir)
│   ├── contribution/
│   │   ├── mod.rs                  # Plugin contribution re-exports
│   │   ├── menu_descriptor.rs      # MenuContribution descriptor type
│   │   └── registry.rs             # MenuContributionRegistry -- insert/remove/reorder
│   ├── config_keys.rs              # Compile-time config key constants for this crate
│   └── error.rs                    # MenuStatusBarError enum
└── tests/
    ├── menu_model_tests.rs         # Menu structure property tests
    ├── context_menu_tests.rs       # Context menu registry tests
    ├── status_bar_tests.rs         # Status bar segment ordering property tests
    ├── command_field_tests.rs      # Command field history property tests
    ├── recent_files_tests.rs       # Recent files MRU property tests
    ├── contribution_tests.rs       # Plugin contribution insertion tests
    └── integration.rs              # End-to-end menu dispatch and status update tests
```

---

## 4. Key Data Models and Types

### MenuBar

```rust
/// The complete menu bar model. A list of top-level menus rendered left-to-right.
/// Addresses: Requirement 1, criteria 1/2
#[derive(Debug, Clone)]
pub struct MenuBar {
    /// Ordered list of top-level menus
    pub menus: Vec<Menu>,
    /// Current keyboard navigation state
    pub nav_state: MenuNavState,
}
```

### Menu

```rust
/// A top-level menu or submenu containing ordered items.
/// Addresses: Requirement 1, criteria 3–7
#[derive(Debug, Clone)]
pub struct Menu {
    /// Display label (e.g., "File", "Edit")
    pub label: String,
    /// Access key character (underlined in UI, e.g., 'F' for File)
    pub access_key: Option<char>,
    /// Ordered list of items (menu items, separators, submenus)
    pub items: Vec<MenuEntry>,
    /// Whether this menu is currently open
    pub is_open: bool,
}
```

### MenuEntry

```rust
/// A single entry within a menu -- an item, separator, or submenu.
/// Addresses: Requirement 1, criteria 3–8; Requirement 2, criteria 1–4
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MenuEntry {
    /// A clickable menu item bound to a command
    Item(MenuItem),
    /// A visual separator between groups of items
    Separator,
    /// A nested submenu
    Submenu(Menu),
}
```

### MenuItem

```rust
/// An individual menu item bound to a command in the command framework.
/// Addresses: Requirement 2, criteria 1–4; Requirement 11, criterion 5
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Unique identifier for this menu item (for contribution targeting)
    pub id: String,
    /// Display label (from command metadata or explicit override)
    pub label: String,
    /// Access key character for keyboard navigation
    pub access_key: Option<char>,
    /// The Command_ID this item invokes when activated
    pub command_id: String,
    /// Optional parameters to pass to the command
    pub params: Option<CommandParams>,
    /// Keyboard shortcut display text (read from ShortcutRegistry)
    pub shortcut_text: Option<String>,
    /// Whether the item is currently enabled (from command enabled predicate)
    pub is_enabled: bool,
    /// Whether the item is currently visible (from command visibility predicate)
    pub is_visible: bool,
    /// Whether this item represents a toggle (checkbox-style display)
    pub is_toggle: bool,
    /// Current toggle state (only meaningful if is_toggle is true)
    pub is_checked: bool,
    /// Contributing plugin name (None for built-in items)
    pub contributed_by: Option<String>,
}
```

### MenuNavState

```rust
/// Keyboard navigation state machine for the menu bar.
/// Addresses: Requirement 11, all criteria
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuNavState {
    /// Menu bar is inactive; no menu is open
    Inactive,
    /// Menu bar is focused (e.g., F10 pressed) but no dropdown open yet
    Focused { highlighted_index: usize },
    /// A dropdown menu is open with a highlighted item
    Open {
        menu_index: usize,
        item_index: Option<usize>,
        submenu_stack: Vec<usize>,
    },
}
```

### ContextType

```rust
/// The type of UI element that a context menu is associated with.
/// Addresses: Requirement 4, criteria 1/2/5
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ContextType {
    /// Right-click in the editor text area
    EditorArea,
    /// Right-click on a tab header
    TabHeader,
    /// Right-click on a panel header
    PanelHeader,
    /// Right-click on a file tree node
    FileTreeNode,
}
```

### StatusSegment

```rust
/// A single segment within the status bar.
/// Addresses: Requirement 5, criteria 2/4
#[derive(Debug, Clone)]
pub struct StatusSegment {
    /// Unique identifier (1–64 ASCII alphanumeric/underscore chars)
    pub id: String,
    /// Alignment group within the status bar
    pub alignment: SegmentAlignment,
    /// Ordering priority within the alignment group (lower = renders first)
    pub priority: u32,
    /// Minimum width in logical pixels (0 = auto-size to content)
    pub min_width: f32,
    /// Whether this segment is currently visible
    pub visible: bool,
    /// Contributing plugin name (None for built-in segments)
    pub contributed_by: Option<String>,
}
```

### SegmentAlignment

```rust
/// Alignment grouping for status bar segments.
/// Addresses: Requirement 5, criterion 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SegmentAlignment {
    /// Left-aligned segments (editor mode, insert/overstrike, encoding)
    Left,
    /// Center-aligned segments (rarely used, reserved for extension)
    Center,
    /// Right-aligned segments (line/col, modified indicator, total lines)
    Right,
}
```

### StatusSegmentProvider Trait

```rust
/// Trait for providing content to a status bar segment.
/// Implemented by built-in providers and plugins.
/// Addresses: Requirement 8, criteria 1/2
pub trait StatusSegmentProvider: Send + Sync {
    /// Returns the unique segment identifier.
    fn segment_id(&self) -> &str;

    /// Render the segment content into the given UI region.
    fn render(&self, ui: &mut egui::Ui);

    /// Returns the alignment group for this segment.
    fn alignment(&self) -> SegmentAlignment;

    /// Returns the ordering priority (lower = renders first within group).
    fn priority(&self) -> u32;

    /// Returns whether the segment currently has content to display.
    /// Segments returning false may be collapsed to save space.
    fn has_content(&self) -> bool { true }
}
```

### EditorMode (re-exported from ff-core)

```rust
/// The current interaction mode of the active editor.
/// Addresses: Requirement 6, criteria 1/2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Browse,
    Edit,
    View,
}

impl std::fmt::Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Browse => write!(f, "Browse"),
            Self::Edit => write!(f, "Edit"),
            Self::View => write!(f, "View"),
        }
    }
}
```

### InsertOverstrikeState

```rust
/// Whether typed characters insert or overwrite.
/// Addresses: Requirement 6, criteria 3/4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertOverstrikeState {
    Insert,
    Overstrike,
}

impl std::fmt::Display for InsertOverstrikeState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Insert => write!(f, "INS"),
            Self::Overstrike => write!(f, "OVR"),
        }
    }
}
```

### CommandFieldState

```rust
/// The state of the primary command field.
/// Addresses: Requirement 9, all criteria
#[derive(Debug, Clone)]
pub struct CommandFieldState {
    /// Current text content of the input field
    pub text: String,
    /// Whether the field currently has keyboard focus
    pub has_focus: bool,
    /// Current position in the history ring (-1 = live input, 0 = most recent)
    pub history_position: i32,
    /// Saved live input when browsing history
    pub saved_input: String,
}
```

### RecentFileEntry

```rust
/// A single entry in the recent files list.
/// Addresses: Requirement 3, criteria 1/3/5
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecentFileEntry {
    /// Absolute path to the file
    pub path: String,
    /// Timestamp of last open/save (for ordering)
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Whether the file still exists on disk (checked lazily)
    pub verified_exists: Option<bool>,
}
```

### MenuContribution

```rust
/// Descriptor for a plugin-contributed menu item.
/// Addresses: Requirement 10, criteria 1/2/3
#[derive(Debug, Clone)]
pub struct MenuContribution {
    /// The target menu path (e.g., "File", "Tools", "View > Panels")
    pub menu_path: String,
    /// The Command_ID to bind this menu item to
    pub command_id: String,
    /// Desired position within the target menu
    pub position: MenuInsertPosition,
    /// Whether to insert a separator before this item
    pub separator_before: bool,
    /// Whether to insert a separator after this item
    pub separator_after: bool,
    /// The plugin that contributed this item
    pub plugin_name: String,
}
```

### MenuInsertPosition

```rust
/// Where to insert a contributed menu item within the target menu.
/// Addresses: Requirement 10, criterion 1
#[derive(Debug, Clone)]
pub enum MenuInsertPosition {
    /// Insert at the end of the menu (before any trailing separator/exit)
    End,
    /// Insert before a specific item ID
    Before(String),
    /// Insert after a specific item ID
    After(String),
    /// Insert at a specific zero-based index
    AtIndex(usize),
}
```

---

## 5. Public API Surface

### Menu Bar -- Construction and Lifecycle

```rust
/// Build the default menu bar model with all built-in menus and items.
/// Reads command metadata and shortcut bindings from the registries.
///
/// Addresses: Requirement 1, criteria 2–7; Requirement 2, criteria 1/2
pub fn build_default_menu_bar(
    command_registry: &CommandRegistry,
    shortcut_registry: &ShortcutRegistry,
) -> MenuBar;

/// Refresh the enabled/visible state of all menu items by re-evaluating
/// command predicates against the current execution context.
///
/// Addresses: Requirement 2, criteria 3/4
pub fn refresh_menu_state(
    menu_bar: &mut MenuBar,
    command_registry: &CommandRegistry,
    context: &ExecutionContext,
);
```

### Menu Bar -- Activation and Dispatch

```rust
impl MenuBar {
    /// Activate the menu item at the given path. Invokes the bound command
    /// through the command dispatch. Returns the CommandResult.
    ///
    /// Addresses: Requirement 2, criteria 1/5–10
    pub fn activate_item(
        &self,
        item_id: &str,
        dispatch: &CommandDispatch,
    ) -> CommandResult;

    /// Process a keyboard navigation event. Updates MenuNavState.
    /// Returns true if the event was consumed by menu navigation.
    ///
    /// Addresses: Requirement 11, all criteria
    pub fn handle_key_event(&mut self, event: &KeyEvent) -> bool;

    /// Open a specific top-level menu by index.
    /// Addresses: Requirement 1, criterion 8; Requirement 11, criterion 1
    pub fn open_menu(&mut self, index: usize);

    /// Close all open menus and return to Inactive state.
    /// Addresses: Requirement 11, criterion 4
    pub fn close_all(&mut self);

    /// Returns the currently highlighted menu item path (for accessibility).
    pub fn highlighted_item(&self) -> Option<&MenuItem>;
}
```

### Context Menu Registry

```rust
/// Registry for context-specific popup menus.
/// Addresses: Requirement 4, all criteria
pub struct ContextMenuRegistry { /* ... */ }

impl ContextMenuRegistry {
    /// Create a new registry with default context menus for editor and tabs.
    ///
    /// Addresses: Requirement 4, criteria 1/2
    pub fn new(
        command_registry: &CommandRegistry,
        shortcut_registry: &ShortcutRegistry,
    ) -> Self;

    /// Get the menu for a given context type. Returns a freshly-evaluated menu
    /// with enabled/visible states set per the current context.
    ///
    /// Addresses: Requirement 4, criteria 3/4
    pub fn get_menu(
        &self,
        context_type: ContextType,
        execution_context: &ExecutionContext,
        command_registry: &CommandRegistry,
    ) -> Menu;

    /// Register a plugin-contributed context menu item for a specific context type.
    ///
    /// Addresses: Requirement 4, criterion 5
    pub fn contribute_item(
        &mut self,
        context_type: ContextType,
        contribution: MenuContribution,
    ) -> Result<(), MenuStatusBarError>;

    /// Remove all contributions from a specific plugin.
    pub fn remove_plugin_contributions(&mut self, plugin_name: &str);
}
```

### Status Bar Manager

```rust
/// Manages the status bar segment registry and layout.
/// Addresses: Requirement 5, all criteria; Requirement 8, all criteria
pub struct StatusBarManager { /* ... */ }

impl StatusBarManager {
    /// Create a new manager with default built-in segments.
    ///
    /// Addresses: Requirement 5, criterion 3
    pub fn new(config: &ConfigHandle) -> Self;

    /// Register a segment provider (built-in or plugin-contributed).
    /// Returns Err if a segment with the same ID already exists.
    ///
    /// Addresses: Requirement 8, criteria 1/3/6
    pub fn register_segment(
        &mut self,
        provider: Box<dyn StatusSegmentProvider>,
    ) -> Result<(), MenuStatusBarError>;

    /// Unregister a segment by ID. Used during plugin unload.
    ///
    /// Addresses: Requirement 8, criterion 4
    pub fn unregister_segment(&mut self, segment_id: &str) -> bool;

    /// Get the ordered list of visible segments for rendering.
    /// Segments are sorted by alignment group, then by priority within group.
    ///
    /// Addresses: Requirement 5, criteria 2/3
    pub fn visible_segments(&self) -> Vec<&dyn StatusSegmentProvider>;

    /// Update segment visibility/ordering from configuration.
    ///
    /// Addresses: Requirement 8, criterion 5
    pub fn apply_config(&mut self, config: &ConfigHandle);

    /// Notify that the active editor context changed (tab switch, mode change, etc.).
    /// Triggers segment content refresh.
    ///
    /// Addresses: Requirement 7, criterion 6
    pub fn notify_context_changed(&mut self, context: &EditorStateSnapshot);
}
```

### EditorStateSnapshot

```rust
/// A snapshot of the active editor's state, provided to status bar segments.
/// Addresses: Requirement 6, all criteria; Requirement 7, all criteria
#[derive(Debug, Clone)]
pub struct EditorStateSnapshot {
    /// Current editor mode (Browse/Edit/View), None if no editor active
    pub mode: Option<EditorMode>,
    /// Insert/Overstrike state, None if no editor active
    pub insert_overstrike: Option<InsertOverstrikeState>,
    /// Current cursor line (1-based), None if no editor active
    pub cursor_line: Option<usize>,
    /// Current cursor column (1-based), None if no editor active
    pub cursor_col: Option<usize>,
    /// File encoding string, None if no editor active
    pub encoding: Option<String>,
    /// Whether the active document has unsaved changes
    pub is_modified: Option<bool>,
    /// Total line count of active document, None if no editor active
    pub total_lines: Option<usize>,
    /// Active indicator flags (HEX, ASA, SEQSHOW, etc.)
    pub active_indicators: Vec<String>,
}
```

### Command Field Controller

```rust
/// Controller for the primary command field ("Command ===>").
/// Addresses: Requirement 9, all criteria
pub struct CommandFieldController { /* ... */ }

impl CommandFieldController {
    /// Create a new controller with an empty history.
    pub fn new() -> Self;

    /// Get the current field state for rendering.
    pub fn state(&self) -> &CommandFieldState;

    /// Set the text content (e.g., when the user types).
    pub fn set_text(&mut self, text: String);

    /// Submit the current field content for command dispatch.
    /// Returns Ok(()) if the command was dispatched (even if it failed),
    /// or Err if the field is empty.
    ///
    /// Addresses: Requirement 9, criteria 3/4/5
    pub fn submit(
        &mut self,
        dispatch: &CommandDispatch,
    ) -> Result<SubmitResult, MenuStatusBarError>;

    /// Navigate command history: direction = -1 (older) or +1 (newer).
    ///
    /// Addresses: Requirement 9, criterion 6
    pub fn history_navigate(&mut self, direction: i32);

    /// Focus or unfocus the command field.
    pub fn set_focus(&mut self, focused: bool);

    /// Returns true if focus should transfer to editor (Down arrow on empty field).
    ///
    /// Addresses: Requirement 9, criterion 7
    pub fn should_transfer_focus_down(&self) -> bool;

    /// Load command history from persisted state.
    pub fn load_history(&mut self, entries: Vec<String>);

    /// Get the current history entries for persistence.
    pub fn history_entries(&self) -> &[String];
}

/// Result of a command field submission.
#[derive(Debug, Clone)]
pub enum SubmitResult {
    /// Command was dispatched successfully
    Dispatched,
    /// Command was not recognized -- error message provided
    Unrecognized { error_message: String },
}
```

### Recent Files Manager

```rust
/// Manages the most recently used files list.
/// Addresses: Requirement 3, all criteria
pub struct RecentFilesManager { /* ... */ }

impl RecentFilesManager {
    /// Create a new manager with the given maximum capacity.
    ///
    /// Addresses: Requirement 3, criterion 2
    pub fn new(max_entries: usize) -> Self;

    /// Create from configuration -- reads `menu.recent_files_max` setting.
    /// Clamps to [1, 50] range.
    pub fn from_config(config: &ConfigHandle) -> Self;

    /// Add or promote a file path to the top of the list.
    ///
    /// Addresses: Requirement 3, criterion 3
    pub fn add_or_promote(&mut self, path: &str);

    /// Get the current list of recent files (most recent first).
    ///
    /// Addresses: Requirement 3, criterion 1
    pub fn entries(&self) -> &[RecentFileEntry];

    /// Remove a specific entry by path.
    pub fn remove(&mut self, path: &str);

    /// Clear the entire list.
    ///
    /// Addresses: Requirement 3, criterion 7
    pub fn clear(&mut self);

    /// Mark a path as non-existent (for greyed display).
    ///
    /// Addresses: Requirement 3, criterion 5
    pub fn mark_missing(&mut self, path: &str);

    /// Remove entries marked as missing.
    pub fn purge_missing(&mut self);

    /// Load recent files from persistent storage.
    ///
    /// Addresses: Requirement 3, criterion 6
    pub fn load(data_dir: &Path) -> Result<Self, MenuStatusBarError>;

    /// Persist the current list to storage.
    ///
    /// Addresses: Requirement 3, criterion 6
    pub fn save(&self, data_dir: &Path) -> Result<(), MenuStatusBarError>;
}
```

### Menu Contribution Registry

```rust
/// Registry for plugin-contributed menu items and submenus.
/// Addresses: Requirement 10, all criteria
pub struct MenuContributionRegistry { /* ... */ }

impl MenuContributionRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self;

    /// Register a plugin menu contribution.
    /// The contribution is applied to the MenuBar at the next refresh.
    ///
    /// Addresses: Requirement 10, criteria 1/2/3
    pub fn register(
        &mut self,
        contribution: MenuContribution,
    ) -> Result<(), MenuStatusBarError>;

    /// Remove all contributions from a specific plugin.
    /// Collapses empty top-level menus created solely by that plugin.
    ///
    /// Addresses: Requirement 10, criterion 4
    pub fn remove_plugin(&mut self, plugin_name: &str);

    /// Apply all registered contributions to a MenuBar model.
    /// Creates new top-level menus as needed (inserted before Help).
    ///
    /// Addresses: Requirement 10, criteria 2/3/5
    pub fn apply_to(&self, menu_bar: &mut MenuBar, command_registry: &CommandRegistry);

    /// List all contributions from a specific plugin.
    pub fn contributions_for(&self, plugin_name: &str) -> Vec<&MenuContribution>;
}
```

### Configuration Keys

```rust
/// Compile-time config key constants for the menu-and-statusbar crate.
/// Consumers use these instead of string literals for compile-time checking.
pub mod config_keys {
    /// Maximum number of recent file entries (default: 10, max: 50)
    pub const MENU_RECENT_FILES_MAX: &str = "menu.recent_files_max";

    /// Status bar segment visibility/ordering configuration table
    pub const STATUSBAR_SEGMENTS: &str = "statusbar.segments";

    /// Whether to show the primary command field (default: true)
    pub const MENU_SHOW_COMMAND_FIELD: &str = "menu.show_command_field";
}
```

---

## 6. Error Types

```rust
/// Errors originating from the ff-menu-statusbar crate.
/// Formatted per Error Message Standards: `[menu] operation: description`
///
/// Addresses: Cross-cutting Requirement 8
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MenuStatusBarError {
    /// Menu item references a command that is not registered.
    #[error("[menu] bind: command '{command_id}' is not registered")]
    CommandNotFound { command_id: String },

    /// Attempted to register a status segment with a duplicate ID.
    /// Addresses: Requirement 8, criterion 6
    #[error("[menu] status: segment '{id}' is already registered")]
    DuplicateSegmentId { id: String },

    /// Invalid segment ID format (must be 1–64 ASCII alphanumeric/underscore).
    /// Addresses: Requirement 5, criterion 4
    #[error("[menu] status: invalid segment ID '{id}' -- must be 1-64 ASCII alphanumeric or underscore")]
    InvalidSegmentId { id: String },

    /// Plugin menu contribution targets a menu path that cannot be resolved.
    #[error("[menu] contribute: cannot resolve menu path '{path}' for plugin '{plugin}'")]
    MenuPathNotFound { path: String, plugin: String },

    /// Plugin attempted to insert at a reference item that does not exist.
    #[error("[menu] contribute: reference item '{reference}' not found in '{menu_path}'")]
    ReferenceItemNotFound { reference: String, menu_path: String },

    /// Command field submission with empty text.
    #[error("[menu] command_field: cannot submit empty command")]
    EmptyCommand,

    /// Recent files persistence I/O error.
    /// Addresses: Requirement 3, criterion 6
    #[error("[menu] recent_files: {operation} failed for '{path}': {source}")]
    RecentFilesIo {
        operation: String,
        path: PathBuf,
        source: std::io::Error,
    },

    /// Recent files JSON parse error.
    #[error("[menu] recent_files: parse error in '{path}': {detail}")]
    RecentFilesParseError { path: PathBuf, detail: String },

    /// Configuration value out of range.
    #[error("[menu] config: key '{key}' value {value} out of range [{min}, {max}] -- using default {default}")]
    ConfigOutOfRange {
        key: String,
        value: String,
        min: String,
        max: String,
        default: String,
    },
}
```

---

## 7. Integration Points

### With `ff-command` (Command Framework -- upstream, Wave 2)

- **Dependency direction**: ff-menu-statusbar depends on ff-command
- **API consumed**: `CommandId`, `CommandRegistry::get()`, `CommandRegistry::metadata()`, `CommandDispatch::execute_command()`, `ShortcutRegistry::binding_for()`, `CommandHandler::is_enabled()`, `CommandHandler::is_visible()`, `ExecutionContext`
- **Usage**: Every menu item is bound to a `CommandId`. Activation routes through `execute_command`. Shortcut text is read from `ShortcutRegistry::binding_for()`. Enabled/visible predicates drive menu item rendering state.
- **Menu items register NO commands** -- they only bind to existing commands registered by other crates (file-operations, edit-operations, etc.)

### With `ff-config` (Configuration System -- upstream, Wave 2)

- **Dependency direction**: ff-menu-statusbar depends on ff-config
- **API consumed**: `ConfigHandle::get_int()`, `ConfigHandle::get_table()`, `ConfigHandle::on_reload()`
- **Usage**: Reads `menu.recent_files_max` for MRU list capacity. Reads `statusbar.segments` for segment visibility/ordering configuration. Registers reload callback to apply config changes live.
- **Persistence**: Recent files list is persisted in the workbench data directory (path obtained via `ff-config` platform path resolution)

### With `ff-layout` (Layout & Docking -- upstream, Wave 2)

- **Dependency direction**: ff-menu-statusbar depends on ff-layout
- **API consumed**: `DockablePanel` trait (for status bar panel registration)
- **Usage**: The status bar is registered as a workbench-level panel in the `Bottom` dock zone with special "always visible" semantics. The menu bar integrates with `Primary_Window` through the layout engine's chrome rendering hooks.

### With `ff-plugin` (Plugin Architecture -- upstream, Wave 2)

- **Dependency direction**: ff-menu-statusbar depends on ff-plugin
- **API consumed**: Plugin capability advertisement for `StatusSegmentProvider` and `MenuContribution`
- **Usage**: Plugins register status segments via `StatusBarManager::register_segment()` and menu contributions via `MenuContributionRegistry::register()`. Plugin unload triggers cleanup of contributed items.
- **Extension points exposed**:
  - `StatusSegmentProvider` trait -- plugins implement to contribute custom status bar segments
  - `MenuContribution` descriptor -- plugins submit to contribute menu items

### With `ff-logging` (Logging -- upstream, Wave 0)

- **Dependency direction**: ff-menu-statusbar depends on ff-logging
- **API consumed**: `log_warn!`, `log_info!`, `log_debug!` macros
- **Usage**: WARN on duplicate segment registration (Req 8.6), WARN on command not found during menu activation, DEBUG on menu item activation for audit, INFO on recent files persistence events

### With `ff-command-semantics` (Command Engine -- downstream, Wave 5)

- **Dependency direction**: ff-command-semantics does NOT depend on this crate; it registers commands that menu items bind to
- **Interaction**: The Primary_Command_Field submits text to the CommandEngine (from ff-command-semantics) for ISPF command parsing and dispatch. The field receives success/failure results for clearing or error display.

### With `ff-desktop` (GUI Shell -- downstream)

- **Dependency direction**: ff-desktop depends on ff-menu-statusbar
- **API consumed**: `MenuBar`, `StatusBarManager`, `CommandFieldController`, render trait methods
- **Usage**: The shell renders the menu bar at the window top, the status bar at the window bottom, and the command field above the editor area -- all by calling this crate's render methods with an `egui::Ui` context.

---

## 8. Correctness Properties

These properties define invariants suitable for property-based testing with `proptest`.

### Property 1: Menu items never dispatch without a registered command

For any `MenuBar` and any `item_id` activation attempt, if the item's `command_id` is not present in the `CommandRegistry`, the activation SHALL return `CommandResult::Err(CommandNotFound)` and SHALL NOT produce side effects.

**Validates: Requirement 2, criterion 10**

### Property 2: Recent files list never exceeds configured maximum

For any sequence of `add_or_promote` operations on a `RecentFilesManager` with capacity `N`, the resulting `entries().len()` SHALL always be `<= N`.

**Validates: Requirement 3, criterion 2**

### Property 3: Recent files add_or_promote is idempotent on ordering for duplicate paths

For any `RecentFilesManager` and any path `P` that is already at position 0 (most recent), calling `add_or_promote(P)` SHALL not change the list contents or ordering.

**Validates: Requirement 3, criterion 3**

### Property 4: Status bar segment IDs are unique

For any sequence of `register_segment` calls on a `StatusBarManager`, if two providers have the same `segment_id()`, the second registration SHALL return `Err(DuplicateSegmentId)` and the first provider SHALL remain active.

**Validates: Requirement 8, criterion 6**

### Property 5: Status bar segments are ordered by alignment then priority

For any `StatusBarManager`, the result of `visible_segments()` SHALL be partitioned into alignment groups (Left, Center, Right in that order), and within each group, segments SHALL be sorted by ascending `priority()` value.

**Validates: Requirement 5, criteria 2/3**

### Property 6: Menu contribution removal leaves no orphan menus

For any `MenuContributionRegistry` and any plugin `P`, after calling `remove_plugin(P)` and `apply_to(menu_bar)`, if a top-level menu was created solely by contributions from `P`, that menu SHALL no longer appear in the menu bar.

**Validates: Requirement 10, criterion 4**

### Property 7: Command field history navigation is bounded

For any `CommandFieldController` with history of length `N`, calling `history_navigate(-1)` more than `N` times SHALL clamp at the oldest entry (position `N-1`) and SHALL NOT panic or wrap around.

**Validates: Requirement 9, criterion 6**

### Property 8: Disabled menu items cannot be activated

For any `MenuBar` where a `MenuItem` has `is_enabled == false`, calling `activate_item` for that item SHALL NOT invoke `execute_command` on the command dispatch and SHALL return immediately without side effects.

**Validates: Requirement 2, criterion 3**

### Property 9: Context menu respects command predicates

For any `ContextMenuRegistry::get_menu()` call with a given `ExecutionContext`, every `MenuItem` in the returned `Menu` SHALL have `is_enabled` and `is_visible` values that match the result of calling the bound command's `is_enabled()` and `is_visible()` predicates with the same context.

**Validates: Requirement 4, criteria 3/4**

### Property 10: Status bar placeholder values when no editor active

For any `EditorStateSnapshot` where all fields are `None`, the built-in status segments SHALL render placeholder text ("--" for mode, "--/--" for line/column, "--" for encoding, etc.) and SHALL NOT panic.

**Validates: Requirement 5, criterion 7**

---

## 6. About Dialog

The About dialog is a simple egui modal window rendered in `ff-desktop` as a new module
`about_dialog.rs`. It holds no mutable state -- it is opened by setting a boolean flag in
`WorkbenchShell` and closed by the user.

- No new crate dependency is required.
- The version string is read from `CARGO_PKG_VERSION` at compile time via `env!("CARGO_PKG_VERSION")`.
- The copyright year is a compile-time constant (`2025`).
- The `Help > About` menu item in `shell.rs` sets `show_about: bool` to `true`.
- `about_dialog::render(ctx, &mut show_about)` is called each frame when `show_about` is true.
- Closing via the `Close` button or the window's `×` button sets `show_about` to `false`.

No new crate dependencies. No architectural contradictions with existing decisions.

---

## 9. Tab-Order Focus Cycle (Requirement 16) -- unified model (CR-CH-023)

**Superseded design.** The original Section 9 described a shell-owned focus ring: a `FocusStop`
enum enumerating `CommandField`, `PomOption(0..8)`, `PomExit`, `CalendarPrev`, `CalendarNext`,
`MenuBar{index}`, and `TabHeader{index}`, advanced by `next()`/`prev()` and driven onto widgets
via `request_focus`. CR-CH-023 REMOVES that ring in favour of one shared model that every
Workspace uses without per-Workspace focus code.

### Model

1. **Interior order = egui-native.** Each Workspace renders its interactive controls in
   visual/creation order; egui's built-in Tab traversal walks them. The POM/menu-workspace
   option rows are ALREADY real focusable `egui::Button`s, and the Menus/Theme editors already
   create their fields in visual order, so their interior Tab order is correct with no shell
   involvement. Disabled menu options are non-interactive `Label`s and are naturally skipped.

2. **Shell Boundary_Policy (implemented once, applies to all Workspaces).** The shell handles
   only the three transitions that cross a Workspace boundary. Each frame, for the active
   Workspace, on a Tab/Shift+Tab press the shell:
   - **Enter:** if focus is on the Primary_Command_Field and Tab is pressed, move focus to the
     Workspace's FIRST interior control. The shell learns the first/last interior control ids
     generically -- see "Interior boundary detection" below -- rather than hard-coding them.
   - **Exit to menu bar:** if focus is on the LAST interior control and Tab is pressed, move to
     the first Menu_Bar item.
   - **Wrap:** if focus is on the LAST Menu_Bar item and Tab is pressed, move to the
     Primary_Command_Field.
   Shift+Tab performs the exact reverse at each boundary. Between boundaries the shell consumes
   nothing and lets egui move focus natively.

3. **Chrome not focusable.** The Status_Bar segments (already `ui.label`, B055), the
   `SCROLL ===>` field, the Tab_Bar tab headers, and the Key_Label_Bar F-key buttons are made
   non-focusable so they never appear in the cycle. For egui this means rendering them so their
   `Response`/`Sense` is not focusable (e.g. plain `ui.label`, or a button/`add` with focus
   suppressed), and NOT calling `request_focus` on them.

### Interior boundary detection

To keep the Boundary_Policy generic (no per-Workspace enumeration), the shell needs the FIRST
and LAST focusable interior control of the active Workspace. Two viable mechanisms, to be
finalised in implementation:

- **(a) egui tab-navigation surrogate:** rely on egui's own "focus wrapped" signal. When Tab
  from the command field would leave the command field, the shell requests egui move focus into
  the central panel; when egui reports focus has wrapped past the last central-panel widget, the
  shell redirects to the Menu_Bar / command field. This needs the command field, menu bar, and
  chrome to be arranged so egui's natural wrap coincides with the desired boundaries (achieved
  by making chrome non-focusable and by ordering the panels so the central panel's widgets are a
  contiguous focus run).
- **(b) sentinel ids:** the central panel exposes a stable "first interior" id (the shell
  focuses it on Enter) and the shell detects "last interior" by observing that a Tab press while
  the central panel held focus produced no in-panel focus move (focus left the panel), then
  redirects to the Menu_Bar. This mirrors the existing File Explorer Tab-transfer approach
  (Tab from the command line enters the tree; Tab past the last node exits to the command line).

Mechanism (b) matches an already-proven pattern in the codebase (File Explorer) and is the
preferred starting point; (a) is the fallback if egui 0.31 provides a clean wrap signal.

### Menu-workspace and calendar

The menu-workspace renderer already emits option rows as focusable buttons (Requirement 2.2).
The calendar changes from a single painted click-region widget to two REAL focusable buttons
`<` (previous month) and `>` (next month), created after the option column, so egui-native
traversal reaches them after the last option and before the Menu_Bar (menu-workspace
Requirement 15.5-15.8). The existing month-change side effect (`CalendarNav::Prev`/`Next`) is
driven by the buttons' click/activate instead of a hit-region test.

### Removal

- Delete the `FocusStop` enum and its `next()`/`prev()` methods from `shell/mod.rs`.
- Remove the ring-drive block and the `is_menus_editor` "Option X" special-case from
  `shell/update.rs`; replace with the generic Boundary_Policy.
- Keep `command_field_focus_requested` (used for Req 16.1/16.1a initial and on-entry focus).

Back Tab (Shift+Tab) reverses the sequence exactly. There is no per-Workspace focus code: a new
Workspace obtains correct Tab order solely by rendering its controls in visual order.

---

## 10. Tab Window Chrome -- Title Line (Requirement 17) and Detachable Tabs (Requirement 18)

### Tab Window Chrome layout

Every tab's content area renders three elements at the top, in order:

```
┌─────────────────────────────────────────────────────────┐
│  [POM]  [file.txt]  [SETTINGS]          ← Tab_Bar       │
├─────────────────────────────────────────────────────────┤
│  FileForge Workbench  v0.1.0            ← Title_Line    │
├─────────────────────────────────────────────────────────┤
│  Command ===>  ___________________________← Cmd Field   │
├─────────────────────────────────────────────────────────┤
│                                                         │
│   (tab content area -- POM / editor / settings / etc.)   │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Title_Line implementation in `ff-desktop`

The Title_Line is a new read-only `egui::TopBottomPanel` rendered between the tab bar and
the command field. It is added to `WorkbenchShell::update()` as a call to a new
`render_title_line(ctx)` method, inserted between `render_tab_bar` and `render_command_field`.

Title_Line text is derived from the active tab's `TabKind` and `path`:

| TabKind | Title_Line text |
|---------|----------------|
| `PrimaryOptionMenu` | `"FileForge Workbench  v{CARGO_PKG_VERSION}"` |
| `FileEditor` with path | full absolute path string |
| `FileEditor` without path | `"[Untitled]"` |
| `FilesPanel` | `"[FILES]"` |
| `SettingsPanel` | `"[SETTINGS]"` |
| `Untitled` | `"[Untitled]"` |

### Legacy theme styling

When the active palette is `VisualMode::Legacy`, the Title_Line panel background is set to
`palette.ui.primary_menu_bg` (`#0000AA`) and the text colour to `palette.ui.menu_bar_fg`
(`#FFFFFF`). For all other themes, the Title_Line uses the standard panel background with
the primary text colour.

### Detachable tabs

Detachable tab support is deferred to a future phase (Phase AL). The "Move to Other View"
context menu item currently stubs out to a no-op. When implemented, it will use egui's
`egui::ViewportBuilder` to create a child viewport containing the full Tab_Window_Chrome
and the tab's content. The `ff-layout` `FloatingWindowManager` will track the window state.

No architectural contradictions with existing decisions. The Title_Line is a pure addition
to the rendering pipeline -- it is non-interactive and does not participate in the tab-order
cycle (Section 9, unified model per CR-CH-023); the command field retains its focus behaviour.


### CR-CH-034 delta -- header label derived from live Context state (B050 phantom-stale-title)

The original Section 10 table derived the Title_Line from `TabKind` + `path`, but for the
panel and Menu_Workspace kinds the shell had drifted to rendering the cached `tab.title`
string in BOTH the Tab_Header (`render_tab_bar`) and the Title_Line (`title_line_text`). A
cached string must be rewritten at every in-place context-switch site; when one site mutates
`tab.kind` / the loaded `menu_workspace` but forgets the title (the same class as the
workspace-conformance phantom Tab-stop bug), the header shows the previous Context's label.
B050 is one instance (an in-place `=<key>` fastpath from a Files Context whose header stayed
`files` after the content changed).

Fix (Requirement 17.10): introduce a single helper `context_header_label(tab) -> String` in
`shell/mod.rs` that derives the label from the tab's LIVE state, and route both
`title_line_text` and `render_tab_bar`'s `base_title` through it:

| Live state | Header label |
|------------|--------------|
| `tab.workspace_name == Some(name)` (and kind is a panel/menu, not an editor) | `[name]` (CX Req 1.4, unchanged precedence) |
| `tab.is_home` | app banner (Title_Line) / `[POM]` (Tab_Header) |
| `kind == MenuWorkspace` (non-home) | `tab.menu_workspace.as_ref().map(tab_title).unwrap_or(tab.title)` -- the CURRENTLY loaded menu's title |
| `kind == FileEditor` | `path` or `[Untitled]` (unchanged) |
| `kind == Untitled` | `[Untitled]` |
| other panel kinds (Config, Files, FileExplorer, Search, PluginManager, EventLog, MacroLibrary, CommandConfigurator, Theme/Menus/Keys editors) | the kind's canonical bracket label |

The Menu_Workspace row is the key change: a non-home Menu_Workspace's displayed label now
comes from the loaded `menu_workspace` (Settings, a named user menu, etc.) rather than the
cached `tab.title`, so an in-place reload of the menu can never leave the header stale. The
panel kinds return a fixed per-kind bracket label (they have no variable content title), so
their label is likewise independent of the cached string. `tab.title` remains stored (it is
still the persistence/session field and the editor label source) but is no longer the display
source for the drift-prone Contexts.

No architectural contradiction: the Title_Line stays a pure, non-interactive derivation; only
its INPUT changes from a cached field to the live kind/menu, eliminating the drift class.


### CR-CH-035 delta -- Detached Workspaces render real content (proper B045 fix)

The Phase AL/AO scaffolding wired a LIVE detach path (context menu "Move to Other View" and
the SPLIT/SPLIT DETACH commands set `detach_pending`; the frame loop consumes it, sets
`TabState.is_floating`, and pushes a `FloatingTab { viewport_id, tab_index, origin_index }`;
the primary tab bar skips `is_floating` tabs; closing the OS window pushes `origin_index`
into `redock_pending` which the primary frame drains). The DEFECT (B045) was that the
floating window was drawn with `ctx.show_viewport_deferred`, whose render closure is
`'static` + `move` and therefore cannot borrow `&mut self` -- so it could only draw a
placeholder `ui.label("Tab N -- floating")`.

Fix decisions:

1. **Immediate viewport (Req 18.8).** Replace `show_viewport_deferred` with
   `ctx.show_viewport_immediate`, which runs the child-viewport UI SYNCHRONOUSLY within the
   current `update()` call and CAN borrow `&mut self`. Each floating window then renders the
   tab's real Context through the same code the docked central panel uses. To render a
   SPECIFIC (non-active) tab without disturbing the docked active tab, the central-panel
   render is factored so it can draw a given tab index into a supplied `egui::Ui`
   (`render_tab_context_into(ui, tab_index)`), and the floating loop calls it for each
   `FloatingTab.tab_index`. The docked path continues to render the active tab; because the
   tab bar hides `is_floating` tabs, a detached tab is never also the docked active tab, so
   there is no double-render conflict.

2. **Title truncation (Req 18.5).** The window title is
   `truncate_title(format!("{} -- FileForge Workbench", title_line_text(tab)), 80)` where
   `truncate_title` clamps to 80 chars on a char boundary. Routing through `title_line_text`
   (the shared derivation, CR-CH-034) means the later Workspace Definition model (Wave B)
   changes the title source in one place.

3. **Faithful redock (Req 18.9).** Redock becomes a remove-and-reinsert at the origin index
   rather than a positional `swap`. `TabManager` gains `remove_at(index) -> TabState` and
   `insert_at(index, TabState)` seams (index bookkeeping mirrors `close_tab`: repair
   `active`/`previous_active`). On redock the tab is removed from its current slot and
   reinserted at `origin_index.min(len)` (append when the origin now exceeds the count, per
   18.3). Detach conversely does NOT remove the tab from `TabManager` (its content/cursor/
   profile must survive live in the same `TabState`); it only sets `is_floating` so the bar
   hides it and the floating loop renders it. This keeps a single source of truth for the
   tab's `TabState` (no clone/restore), satisfying the "identity/content/cursor/profile
   survive the round-trip" clause.

4. **Drag-out (Req 18.6).** Detaching by dragging a Tab_Header >20px outside the bar and
   releasing outside the Primary_Window depends on real pointer-vs-OS-window geometry that
   the headless `egui_kittest` harness cannot drive deterministically; it is implemented
   against egui drag deltas where feasible and its acceptance is a justified MANUAL TCR row.

Testability: the headless-testable behaviour -- detach creates a `FloatingTab` + sets
`is_floating` + removes the header from the bar; redock removes the `FloatingTab`, clears
`is_floating`, and reinserts at the origin index; the 16-window guard; the 80-char title
truncation (pure function) -- is covered by full-shell `egui_kittest` `build_eframe` tests
and unit tests. The REAL multi-viewport OS window (its actual separate-window appearance,
taskbar presence, independent move/resize) is a justified MANUAL row per testing.md (real
OS windows / multi-viewport are the documented harness exception). `show_viewport_immediate`
child content CAN be exercised headlessly for the render-without-panic + correct-tab
assertions.

No architectural contradiction: this completes the deferred Phase AL/AO intent using egui's
supported immediate-viewport API; the detach/redock state machine (`detach_pending` /
`FloatingTab` / `redock_pending`) is retained, only its viewport call and redock mechanics
are corrected.


### CR-CH-036 delta -- Detached Workspaces as independent command contexts (B045)

CR-CH-035 made the detached window render real content via `show_viewport_immediate` + the
"active-tab time-slice" (temporarily set the detached tab active, render, restore). But the
shell has ONE `command_text` and ONE active-tab index, and the entire command pipeline
(`run_command_line` -> `handle_command` -> `nav_manager`/`find_manager`/`exclude_manager`,
`nav_stack.rs`, `cmd_engine`, `cursor_context_snapshot`) is hard-bound to
`self.tabs.active_tab()` and `self.command_text`. So the detached window had no command field
and the primary command line drove whatever was active -- the cross-window bleed the owner hit.

Rewriting the dozens of `active_tab()`/`command_text` call sites to thread an explicit target
would be large and risky. Instead, EXTEND the time-slice trick to the whole command context:

1. **Per-window command context.** Introduce `WorkspaceCommandContext { command_text: String,
   scroll_field_text: String, scroll_amount: ScrollAmount, open_error: Option<String>,
   command_field_focus_requested: bool, pending_command_line_outcome: Option<CommandLineOutcome> }`.
   Each `FloatingTab` owns one (its independent Command ===> buffer, SCROLL buffer, status line,
   and focus/outcome latches). The Primary_Window keeps using the shell's existing fields (its
   own implicit context).

2. **Scoped context swap.** Add `WorkbenchShell::with_workspace_context(&mut self, tab_index,
   &mut WorkspaceCommandContext, f: impl FnOnce(&mut Self))` that: saves the shell's active
   index + the six shell fields; installs the detached tab as active and MOVES the FloatingTab's
   buffers into the shell fields; runs `f` (which renders the detached command field and, on
   Enter, calls the UNCHANGED `run_command_line`); then moves the (possibly command-modified)
   buffers back into the FloatingTab and restores the saved active index + shell fields. Because
   `show_viewport_immediate` is synchronous, the entire existing pipeline transparently operates
   on the detached tab with the detached window's buffers -- no pipeline rewrite. `handle_command`,
   the managers, `nav_stack`, and the Command_Line_Outcome application all "just work" because
   `active_tab()` and `command_text` now transiently mean the detached window's.

3. **Full chrome in the child viewport.** The floating loop renders, inside the swapped context:
   Title_Line, the Primary_Command_Field (a `render_command_field`-equivalent), and optionally the
   SCROLL field, all bound to the shell fields (which currently hold the window's buffers). The
   command-field panel id and widget id are SALTED per window (`("command_field", tab_id)` /
   `("command_field_input", tab_id)`) so they never collide with the primary field or other
   detached windows.

4. **Dispatch targeting.** On Enter in a detached command field, the closure calls
   `self.run_command_line(&cmd)` from WITHIN the swap, so the command acts on the detached tab and
   its Command_Line_Outcome applies to the detached window's `command_text`. The primary window's
   field and status are untouched.

5. **Shared vs per-window.** RETRIEVE history (`command_line_history`), the command registry,
   engine, key map, zoom, notifications, and panel states remain single/global (documented). The
   RETRIEVE ring being shared is an accepted allowance (Req 18.10); per-window history is a future
   refinement. `key_map_resolver`/`key_label_bar` stay global for this slice (function keys act on
   the focused window's context via the same swap when a detached window has focus -- a follow-up
   if physical F-keys must target a specific detached window; the command FIELD independence is the
   criterion this slice satisfies).

Testability: the per-window independence is headless-testable through the shell -- drive a command
into a detached window's context (via the swap helper / a test entry that submits to a FloatingTab's
buffer) and assert it changed THAT tab and left the primary `command_text`/active tab untouched, and
vice versa. The real multi-viewport OS windows remain MANUAL (testing.md exception), but the
buffer-isolation + correct-target-dispatch logic is asserted headlessly.

No architectural contradiction: this is the same synchronous-swap technique CR-CH-035 already uses
for rendering, generalised to the command context; it keeps ONE command pipeline (avoids a second
divergent code path) while giving each window an isolated command state.


### B045 bundle 2 delta -- RETURN semantics, close=RETURN, F-keys, DOCK, detached menu bar

Owner refinements after CR-CH-036 landed. All build on the CR-CH-036 `with_workspace_context`
swap so detached and docked behaviour stay identical.

1. **RETURN semantics (CR-CH-038, menu-workspace Req 14.10).** `nav_return` is changed from
   "collapse to the tab's ROOT context" to a POM-targeted rule:
   - non-POM active tab -> `set_active_tab_home()` (navigate to the Home Context / POM in place),
     clearing `nav_stack`;
   - POM active tab -> `close_workspace_or_exit()` (close this one workspace; exit if it is the
     last). END (`nav_end`) is unchanged (pop one level; close at root). This makes RETURN "go to
     the POM" from anywhere and "close one workspace" from a POM (Option A -- not recursive).

2. **Detached close = RETURN (CR-CH-037, Req 18.3).** The floating-viewport `close_requested()`
   handler no longer pushes to `redock_pending`; instead it runs `nav_return` INSIDE the window's
   `with_workspace_context` swap (so RETURN acts on the detached tab). If RETURN returned the
   detached workspace to its POM, the window stays open (the FloatingTab remains); if RETURN closed
   the workspace (it was a POM), the tab is gone -- the floating loop's `index_of_id` lookup then
   fails next frame and the FloatingTab is dropped, so the OS window closes. `CancelClose` is still
   sent on the frame RETURN only navigated (window must stay), and NOT sent when the workspace was
   closed (window is allowed to close). No Alt+F4 binding is added; Alt+F4 stays unassigned.
   The old `redock_pending` path is removed (redock is now the DOCK command).

3. **F-keys in the detached window (B068, Req 18.11).** The primary F-key detection block in
   `update()` reads the primary `ctx`. Add an equivalent F-key detection inside the floating
   viewport's frame, sourced from the child `vctx`, dispatched via `dispatch_key_command` WITHIN
   the `with_workspace_context` swap so the bound command (e.g. F3=END, F4=RETURN) acts on the
   detached tab. Same keymap + same merge-with-command-field behaviour as the primary window.

4. **DOCK command + Shift+F2 (CR-NR-088, Req 18.13).** New `handle_command` arm `DOCK`: if the
   active tab is `is_floating`, re-dock it -- find its FloatingTab, clear `is_floating`, `move_tab`
   it to `origin_index` (the CR-CH-035 faithful redock), and drop the FloatingTab; else a status
   message ("DOCK: the current workspace is not detached"). Typed in the detached window's command
   line it runs under the swap so "the active tab" is the detached one. `ff-keys` default_global:
   the Shift-row F2 entry changes from `SPLIT` to `DOCK` (Base F2 unchanged); the two default-map
   tests update their expected Shift row (still 24 bindings).

5. **Detached menu bar (CR-NR-089, Req 18.12).** Render `render_menu_bar` inside the detached
   viewport under the swap, with a per-window-salted panel id (the shared renderer's fixed
   `"menu_bar"` id is parameterised or a salted wrapper is used) so it does not collide with the
   primary Menu_Bar; menu-item dispatch already routes through `handle_command`, so under the swap
   it acts on the detached tab.

Testability: RETURN semantics, DOCK, and F-key-in-context dispatch are all headless-testable
through the shell (drive the command / a swapped F-key dispatch and assert the tab state). The
real OS-window close gesture and the visual menu bar remain MANUAL (real multi-viewport window).


---

## Design Delta: SPLIT -> DETACH rename + retire inert ISPF split (Requirement 18.14 + 19.11-19.14 revision, CR-CH-040 / B046 Slice 1)

### Problem

The `SPLIT` command is overloaded and misnamed:
- On a NON-editor Workspace, `SPLIT` (and `SPLIT DETACH`) detach the Workspace into a
  Detached_Workspace (OS window); `DOCK` re-attaches. This is the useful, working behaviour, but the
  verb `SPLIT` does not communicate "detach into its own window".
- On an EDITOR Workspace, `SPLIT` sets `scroll_amount::SplitScreenState` at the cursor line, and
  `UNSPLIT` clears it, and bare `SWAP` flips its `active_half`. This state is INERT: a code audit
  confirms NO render path (`render.rs`, `render_chrome.rs`, `editor_panel.rs`) reads `split_screen` or
  any `SplitScreenState` field, so nothing visibly splits. It is dead weight that only confuses the
  command surface.

Separately, a proper VS-Code-style split model already exists but is UNWIRED: `ff-layout::TabGroupTree`
(a binary tree of `Split { direction: SplitDirection, proportion: f32, first, second }` / `Leaf(TabGroup)`)
is fully built and unit-tested in the `ff-layout` crate, but `ff-desktop` does not depend on `ff-layout`
at all. That is the foundation for the real split (Slice 2), not the inert `SplitScreenState`.

### Slice 1 scope (this delta)

Rename + retire only. NO new split feature.

1. **Rename the detach verb.** Add a `DETACH` command in `shell/commands.rs` that runs the exact detach
   path the non-editor `SPLIT` / `SPLIT DETACH` arms run today (16-window limit check -> set
   `detach_pending`). Keep `SPLIT DETACH` as a deprecated alias dispatching the same path. `DOCK` is
   unchanged.
2. **Retire the inert ISPF split.** Remove: the editor-tab branch of the `SPLIT` arm (the one that sets
   `SplitScreenState`), the `UNSPLIT` arm, and the bare-`SWAP` `split_screen.swap_focus()` branch (bare
   `SWAP` now always does the previous-tab toggle / picker). Delete the `split_screen` field from
   `WorkbenchShell` and the `SplitScreenState` type from `scroll_amount.rs` (keep `ScrollAmount`).
   `SWAP n` / `SWAP LIST` / bare-`SWAP` toggle are unchanged.
3. **Reserve `SPLIT`.** After Slice 1 the bare verb `SPLIT` no longer detaches and has no editor-split
   behaviour; it is unhandled at the shell level (falls through to the normal unresolved-command path)
   until Slice 2 gives it the real in-window split. `SPLIT DETACH` still works (alias).
4. **Key map.** In `ff-keys::key_map::default_global`, change Base F2 from `("SPLIT","Split")` to
   `("DETACH","Detach")`. Shift+F2 stays `("DOCK","Dock")`. This preserves "F2 detaches" under the new
   name and keeps the DETACH/DOCK antonym pair on F2 / Shift+F2.

### Command-parity note

`DETACH` and `DOCK` are both real dispatchable commands (command-driven principle); the F2 / Shift+F2
bindings and the tab context-menu "Move to Other View" affordance all route through the same command
path. No UI affordance bypasses the command layer.

### Testing

- `shell/commands.rs` unit tests: `detach_command_sets_detach_pending`,
  `split_detach_alias_still_detaches`, `bare_split_no_longer_detaches`,
  `unsplit_command_removed_is_unhandled` (or updated existing split/unsplit tests to the new behaviour),
  `bare_swap_toggles_previous_tab_without_split`.
- `ff-keys` unit test: `default_global` Base F2 = DETACH, Shift+F2 = DOCK (update the existing
  `default_global` row-assertion tests that currently expect F2 = SPLIT).
- The real OS-window detach appearance stays MANUAL (existing Req 18 exception).

### Deferred (Slice 2, separate future CR)

The real in-window split: make `ff-desktop` depend on `ff-layout`, render the active Workspace area
through a `TabGroupTree`, and give `SPLIT` (or a direction-explicit verb) the divide-into-two-regions
behaviour with relative proportions and a chosen new-region Context. Not in Slice 1.

---

## Design Delta: Single config-driven centered Menu Workspace Title_Line (Requirement 17.3/17.6/17.11, CR-CH-042)

Design delta to Section 10 (Tab Window Chrome -- Title Line). The full design of
the change lives in `menu-workspace/design.md` (Requirement 20 delta); this delta
records the Title_Line-side decisions this spec owns.

### Change

For a Menu_Workspace Context (POM, Settings, user menu), the Title_Line (the
read-only line above the Primary_Command_Field) displays the menu's Menu_Title
(the `MenuFile.title` field, raw) CENTERED, using one uniform format for every
menu. This REPLACES:
- the POM Title_Line hardcoded application banner `FileForge Workbench  vX.Y.Z`
  (Requirement 17.3, revised), and
- the non-Home menu bracketed / left-aligned `[<UPPERCASE TITLE>]` form
  (Requirement 17.6, revised).

The separate centered Menu_Title heading previously drawn INSIDE the menu body
above the option list is removed (owned by menu-workspace Req 20 / its design
delta), so a Menu Workspace shows its title in exactly one position -- the
Title_Line.

### Preserved

- Non-menu Title_Line cases are unchanged: file editor => path / `[Untitled]`
  (17.4/17.5); system/panel Context => Kind title (17.6 non-menu part).
- Title_Line styling (17.7) and the Legacy blue-bg/white-text rule (17.8) are
  unchanged; the POM's black-bg/blue styling is a theme concern and stays.
- The in-place live-derivation guarantee (17.10 / CR-CH-034): the Menu_Title is
  read from the LIVE loaded menu each frame, never a stale cached string.
- The application name/version is not lost: it remains available via the About
  dialog (Section 6 of this design) and/or the status area; it is simply no
  longer the POM Title_Line.

### Implementation note

The shell renders the Title_Line (`ff-desktop` `shell/render.rs
::render_title_line_into_ui`, string from `shell/mod.rs::title_line_text` /
`kind_title`); the centering generalises the existing POM `is_pom` centered
branch to all `TabKind::MenuWorkspace` tabs. No `ff-menu-statusbar` API change.
