//! `TabState` -- per-tab state owned by the desktop shell.
//!
//! Each open tab holds its own `DocumentHandle`, `ViewportModel`, and
//! `CursorModel` so that switching tabs preserves scroll position and cursor.

use ff_document_model::{DocumentHandle, LineEndMode};
use ff_edit_operations::EditProfile;
use ff_viewport_scrolling::{CursorModel, ViewportModel};
use std::collections::HashMap;

use crate::menu_workspace::MenuWorkspaceState;

/// The kind of content a tab is displaying.
///
/// Drives central-panel dispatch and determines which context-menu items
/// are shown when the user right-clicks the tab header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKind {
    /// A file loaded from the VFS.
    FileEditor,
    /// A new, unsaved buffer with no backing file.
    Untitled,
    /// Virtual Catalog Manager -- POM option 1.
    FilesPanel,
    /// Config Panel -- the flat config-key browser opened by `CONFIG`.
    ///
    /// Validates: Requirement 15.1, 15.9
    ConfigPanel,
    /// File Explorer Panel -- POM option 2 (tree view of catalog contents).
    ///
    /// Validates: Requirement 19.11, 19.12
    FileExplorerPanel,
    /// Global Search Results panel.
    ///
    /// Validates: global-search Requirement 1.1
    SearchResults,
    /// Plugin Manager panel -- POM option 8.
    ///
    /// Validates: plugin-manager-ui Requirement 1.1
    PluginManager,
    /// Event Log panel -- notification history.
    ///
    /// Validates: notification-system Requirement 2.1
    EventLog,
    /// Macro Library panel -- POM option 6 / MACROS / =6.
    ///
    /// Validates: lua-macro-engine Requirement 12.1
    MacroLibrary,
    /// A data-driven menu loaded from a TOML file.
    ///
    /// Validates: menu-workspace Requirement 1, 2
    MenuWorkspace,
    /// Command Configurator -- lists and edits user-defined command definitions.
    ///
    /// Validates: command-configurator Requirement 2.1, 2.7
    CommandConfigurator,
    /// Theme Editor -- copy/edit/save/set-active themes.
    ///
    /// Validates: theme-and-appearance Requirement 20.1
    ThemeEditor,
    /// In-app Menus editor Context (create/edit/reorder/save menu TOMLs).
    ///
    /// Validates: menu-workspace Requirement 13.1 (CR-NR-075)
    MenusEditor,
    /// In-app Keys editor Context (edit/save per-workspace-kind key lists to
    /// `keymaps/<kind>.toml`). Replaces the retired modal Key Configuration
    /// dialog.
    ///
    /// Validates: function-keys-and-history Requirement 22 (CR-CH-029)
    KeysEditor,
}

/// A single undoable edit stored as the inverse operation to apply.
#[derive(Debug)]
pub enum UndoEntry {
    /// Undo an insert: delete `length` bytes at `position`.
    DeleteBytes { position: u64, length: u64 },
    /// Undo a delete: re-insert `bytes` at `position`.
    InsertBytes { position: u64, bytes: Vec<u8> },
}

/// A unique tab identifier (simple counter -- no UUID dep needed at this layer).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub u64);

/// All state associated with a single open tab.
pub struct TabState {
    /// Stable identity.
    #[allow(dead_code)]
    pub id: TabId,
    /// What kind of content this tab is displaying.
    pub kind: TabKind,
    /// Display title shown in the tab header (filename or "Untitled").
    pub title: String,
    /// Full path if backed by a file, None for untitled buffers.
    pub path: Option<String>,
    /// Shared document handle.
    pub document: DocumentHandle,
    /// Independent viewport state for this tab.
    pub viewport: ViewportModel,
    /// Independent cursor state for this tab.
    pub cursor: CursorModel,
    /// True when the document has unsaved changes.
    pub is_modified: bool,
    /// Cached total line count -- updated at load time and after edits.
    pub line_count: u64,
    /// Cached line-end / encoding mode -- used to derive the encoding label.
    pub line_end_mode: LineEndMode,
    /// Per-tab undo stack (inverse operations, most-recent last).
    pub undo_stack: Vec<UndoEntry>,
    /// Per-line editable prefix area text (line number -> current input string).
    pub prefix_inputs: HashMap<u64, String>,
    /// True when this tab has been detached into a floating OS window.
    ///
    /// Validates: Requirement 18.4
    #[allow(dead_code)]
    pub is_floating: bool,
    /// ISPF edit profile for this tab (CAPS, NULLS, STATS, LOCK, HILITE).
    ///
    /// Validates: Requirement 16.1-16.12
    pub edit_profile: EditProfile,
    /// Active mouse text selection on the editor canvas.
    ///
    /// Stored as (anchor_line, anchor_col, end_line, end_col) in 1-based coordinates.
    /// `None` means no selection is active.
    ///
    /// Validates: Requirement 13.1-13.10 (caret-and-selection)
    pub canvas_selection: Option<(u64, u64, u64, u64)>,
    /// User-assigned Workspace name (optional).
    ///
    /// When `Some`, displayed in the tab header alongside the content title.
    /// When `None`, only the content-derived title is shown (existing behaviour).
    ///
    /// Validates: CX Requirement 1.1, 1.2, 1.3, 1.4
    pub workspace_name: Option<String>,
    /// Menu Workspace state -- populated when `kind == TabKind::MenuWorkspace`.
    ///
    /// Validates: menu-workspace Requirement 1, 2
    pub menu_workspace: Option<MenuWorkspaceState>,
    /// True when this Menu Workspace is the Home Context (the POM). After the
    /// CR-NR-082 Slice 1 unification the POM is just a `MenuWorkspace` tab whose
    /// menu is `pom`; this flag is the stable Home identity (survives before the
    /// menu is lazily loaded, and independent of a user `workspace_name`). It
    /// drives the barebones pom-seed, the Title_Line Home styling, the `pom`
    /// keymap context, and the persistence descriptor.
    ///
    /// Validates: menu-workspace Requirement 18.1, 18.2, 18.5, 18.6
    pub is_home: bool,
    /// Per-tab Navigation_Stack: the ordered ancestors of the current Context,
    /// most-recent last. The current Context is NOT on the stack; an empty stack
    /// means this tab is at its root (END closes the Workspace).
    ///
    /// Validates: menu-workspace Requirement 14.1 (CR-CH-022)
    pub nav_stack: Vec<ff_session::WorkspaceDescriptor>,
}

// === Helper macro to reduce constructor boilerplate =========================

macro_rules! base_tab {
    ($id:expr, $kind:expr, $title:expr, $doc:expr) => {{
        let mut viewport = ViewportModel::with_line_count(1);
        viewport.set_line_height(16);
        TabState {
            id: $id,
            kind: $kind,
            title: $title,
            path: None,
            document: $doc,
            viewport,
            cursor: CursorModel::new(),
            is_modified: false,
            line_count: 1,
            line_end_mode: LineEndMode::Default,
            undo_stack: Vec::new(),
            prefix_inputs: HashMap::new(),
            is_floating: false,
            edit_profile: EditProfile::new(),
            canvas_selection: None,
            workspace_name: None,
            menu_workspace: None,
            is_home: false,
            nav_stack: Vec::new(),
        }
    }};
}

impl TabState {
    /// Create an untitled tab wrapping an existing document handle.
    pub fn untitled(id: TabId, document: DocumentHandle, line_count: u64) -> Self {
        let mut viewport = ViewportModel::with_line_count(line_count);
        viewport.set_line_height(16);
        Self {
            id,
            kind: TabKind::Untitled,
            title: "Untitled".to_string(),
            path: None,
            document,
            viewport,
            cursor: CursorModel::new(),
            is_modified: false,
            line_count,
            line_end_mode: LineEndMode::Default,
            undo_stack: Vec::new(),
            prefix_inputs: HashMap::new(),
            is_floating: false,
            edit_profile: EditProfile::new(),
            canvas_selection: None,
            workspace_name: None,
            menu_workspace: None,
            is_home: false,
            nav_stack: Vec::new(),
        }
    }

    /// Create a tab for a file that has been loaded into `document`.
    pub fn for_file(
        id: TabId,
        path: String,
        document: DocumentHandle,
        line_count: u64,
        line_end_mode: LineEndMode,
    ) -> Self {
        let title = std::path::Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.clone());
        let mut viewport = ViewportModel::with_line_count(line_count);
        viewport.set_line_height(16);
        Self {
            id,
            kind: TabKind::FileEditor,
            title,
            path: Some(path),
            document,
            viewport,
            cursor: CursorModel::new(),
            is_modified: false,
            line_count,
            line_end_mode,
            undo_stack: Vec::new(),
            prefix_inputs: HashMap::new(),
            is_floating: false,
            edit_profile: EditProfile::new(),
            canvas_selection: None,
            workspace_name: None,
            menu_workspace: None,
            is_home: false,
            nav_stack: Vec::new(),
        }
    }

    /// Create the Home Context (the Primary Option Menu).
    ///
    /// After the CR-NR-082 Slice 1 unification the POM is a `MenuWorkspace`
    /// tab whose menu is `pom`; `is_home` is the stable Home identity. The
    /// menu itself is seeded lazily on first render (barebones fallback when
    /// `pom.toml` is absent) via `ensure_pom_menu_loaded`.
    ///
    /// Validates: menu-workspace Requirement 18.1, 18.2
    pub fn pom(id: TabId, document: DocumentHandle) -> Self {
        let mut tab = base_tab!(id, TabKind::MenuWorkspace, "[POM]".to_string(), document);
        tab.is_home = true;
        tab
    }

    /// Create a Files Panel (Virtual Catalog Manager) tab.
    pub fn files_panel(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(id, TabKind::FilesPanel, "[FILES]".to_string(), document)
    }

    /// Create a Config Panel tab (the flat config-key browser).
    ///
    /// Validates: Requirement 15.1, 15.9
    pub fn config_panel(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(id, TabKind::ConfigPanel, "[CONFIG]".to_string(), document)
    }

    /// Create a File Explorer Panel tab (POM option 2).
    ///
    /// Validates: Requirement 19.11, 19.12
    pub fn file_explorer_panel(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(
            id,
            TabKind::FileExplorerPanel,
            "[FILES]".to_string(),
            document
        )
    }

    /// Create a Search Results panel tab.
    ///
    /// Validates: global-search Requirement 1.1
    pub fn search_results_panel(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(id, TabKind::SearchResults, "[SEARCH]".to_string(), document)
    }

    /// Create a Plugin Manager panel tab.
    ///
    /// Validates: plugin-manager-ui Requirement 1.1
    pub fn plugin_manager(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(
            id,
            TabKind::PluginManager,
            "[PLUGINS]".to_string(),
            document
        )
    }

    /// Create an Event Log panel tab.
    ///
    /// Validates: notification-system Requirement 2.1
    pub fn event_log(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(id, TabKind::EventLog, "[LOG]".to_string(), document)
    }

    /// Create a Macro Library panel tab.
    ///
    /// Validates: lua-macro-engine Requirement 12.1
    pub fn macro_library(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(id, TabKind::MacroLibrary, "[MACROS]".to_string(), document)
    }

    /// Create a Command Configurator tab.
    ///
    /// Validates: command-configurator Requirement 2.1, 2.7
    pub fn command_configurator(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(
            id,
            TabKind::CommandConfigurator,
            "[COMMANDS]".to_string(),
            document
        )
    }

    /// Create a Theme Editor tab.
    ///
    /// Validates: theme-and-appearance Requirement 20.1
    // CR-CH-022: only used by the retained open_theme_editor_tab (in-place
    // navigation transforms an existing tab rather than constructing a new one).
    #[allow(dead_code)]
    pub fn theme_editor(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(id, TabKind::ThemeEditor, "[THEME]".to_string(), document)
    }

    /// Create a Menus Editor tab.
    ///
    /// Validates: menu-workspace Requirement 13.1 (CR-NR-075)
    // CR-CH-022: only used by the retained open_menus_editor_tab.
    #[allow(dead_code)]
    pub fn menus_editor(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(id, TabKind::MenusEditor, "[MENUS]".to_string(), document)
    }

    /// Create a Menu Workspace tab backed by a TOML file at `file_path`.
    ///
    /// Validates: menu-workspace Requirement 1.1, 1.7
    pub fn menu_workspace_tab(
        id: TabId,
        document: DocumentHandle,
        mw_state: MenuWorkspaceState,
    ) -> Self {
        let title = mw_state.tab_title();
        let mut tab = base_tab!(id, TabKind::MenuWorkspace, title, document);
        tab.menu_workspace = Some(mw_state);
        tab
    }

    /// Human-readable encoding label derived from the line-end mode.
    pub fn encoding_label(&self) -> &'static str {
        match self.line_end_mode {
            LineEndMode::Default => "UTF-8",
            LineEndMode::Unicode => "UTF-8 (Unicode)",
        }
    }
}
