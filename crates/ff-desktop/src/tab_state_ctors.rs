//! `TabState` constructors + the `base_tab!` boilerplate macro.
//!
//! Split out of `tab_state.rs` for the 400-line rule; behaviour unchanged. The
//! `macro_rules! base_tab` is defined above the `impl TabState` block in this
//! same file so textual macro scoping is satisfied with no `#[macro_export]`.

use ff_document_model::{DocumentHandle, LineEndMode};
use ff_edit_operations::EditProfile;
use ff_viewport_scrolling::{CursorModel, ViewportModel};
use std::collections::HashMap;

use crate::menu_workspace::MenuWorkspaceState;
use crate::tab_state::{TabId, TabKind, TabState, DEFAULT_OWNING_ENVIRONMENT};

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
            is_home: false,
            nav_stack: Vec::new(),
            owning_environment: DEFAULT_OWNING_ENVIRONMENT.to_string(),
            store_identity: None,
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
            is_home: false,
            nav_stack: Vec::new(),
            owning_environment: DEFAULT_OWNING_ENVIRONMENT.to_string(),
            store_identity: None,
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
            is_home: false,
            nav_stack: Vec::new(),
            owning_environment: DEFAULT_OWNING_ENVIRONMENT.to_string(),
            store_identity: None,
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
        let mut tab = base_tab!(
            id,
            TabKind::MenuWorkspace(None),
            "[POM]".to_string(),
            document
        );
        tab.is_home = true;
        tab
    }

    /// Create a Files Panel (Virtual Catalog Manager / Catalog Explorer) tab.
    ///
    /// CR-NR-090 B.1: this is the Catalogs Kind; its title is `[CATALOGS]`,
    /// distinct from the File Explorer's `[FILES]` (the display label is derived
    /// from the Kind, but the cached title is kept consistent).
    pub fn files_panel(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(id, TabKind::FilesPanel, "[CATALOGS]".to_string(), document)
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

    /// Create a SCRM Replay viewer tab.
    ///
    /// Validates: screen-snapshot-scrm Requirement 16.1 (CR-NR-098)
    pub fn scrm_viewer(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(id, TabKind::ScrmViewer, "[REPLAY]".to_string(), document)
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
    // CR-CH-022: retained constructor, currently uncalled. The in-place
    // navigate_to model transforms an existing tab rather than constructing a
    // new Theme Editor tab, so the former open_theme_editor_tab opener was
    // removed; this constructor is kept for the navigable-kind path.
    #[allow(dead_code)]
    pub fn theme_editor(id: TabId, document: DocumentHandle) -> Self {
        base_tab!(id, TabKind::ThemeEditor, "[THEME]".to_string(), document)
    }

    /// Create a Menus Editor tab.
    ///
    /// Validates: menu-workspace Requirement 13.1 (CR-NR-075)
    // CR-CH-022: retained constructor, currently uncalled. The former
    // open_menus_editor_tab opener was removed in favour of in-place navigate_to.
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
        base_tab!(id, TabKind::MenuWorkspace(Some(mw_state)), title, document)
    }
}
