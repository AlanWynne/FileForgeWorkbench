//! `TabState` — per-tab state owned by the desktop shell.
//!
//! Each open tab holds its own `DocumentHandle`, `ViewportModel`, and
//! `CursorModel` so that switching tabs preserves scroll position and cursor.

use ff_document_model::{DocumentHandle, LineEndMode};
use ff_edit_operations::EditProfile;
use ff_viewport_scrolling::{CursorModel, ViewportModel};
use std::collections::HashMap;

/// The kind of content a tab is displaying.
///
/// Drives central-panel dispatch and determines which context-menu items
/// are shown when the user right-clicks the tab header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKind {
    /// ISPF-style Primary Option Menu.
    PrimaryOptionMenu,
    /// A file loaded from the VFS.
    FileEditor,
    /// A new, unsaved buffer with no backing file.
    Untitled,
    /// Virtual Catalog Manager — POM option 1.
    FilesPanel,
    /// Settings Panel — POM option 0.
    ///
    /// Validates: Requirement 15.1, 15.9
    SettingsPanel,
    /// File Explorer Panel — POM option 2 (tree view of catalog contents).
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
}

/// A single undoable edit stored as the inverse operation to apply.
#[derive(Debug)]
pub enum UndoEntry {
    /// Undo an insert: delete `length` bytes at `position`.
    DeleteBytes { position: u64, length: u64 },
    /// Undo a delete: re-insert `bytes` at `position`.
    InsertBytes { position: u64, bytes: Vec<u8> },
}

/// A unique tab identifier (simple counter — no UUID dep needed at this layer).
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
    /// Cached total line count — updated at load time and after edits.
    pub line_count: u64,
    /// Cached line-end / encoding mode — used to derive the encoding label.
    pub line_end_mode: LineEndMode,
    /// Per-tab undo stack (inverse operations, most-recent last).
    pub undo_stack: Vec<UndoEntry>,
    /// Per-line editable prefix area text (line number → current input string).
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
        }
    }

    /// Create a Primary Option Menu tab.
    pub fn pom(id: TabId, document: DocumentHandle) -> Self {
        let mut viewport = ViewportModel::with_line_count(1);
        viewport.set_line_height(16);
        Self {
            id,
            kind: TabKind::PrimaryOptionMenu,
            title: "[POM]".to_string(),
            path: None,
            document,
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
        }
    }

    /// Create a Files Panel (Virtual Catalog Manager) tab.
    pub fn files_panel(id: TabId, document: DocumentHandle) -> Self {
        let mut viewport = ViewportModel::with_line_count(1);
        viewport.set_line_height(16);
        Self {
            id,
            kind: TabKind::FilesPanel,
            title: "[FILES]".to_string(),
            path: None,
            document,
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
        }
    }

    /// Create a Settings Panel tab.
    ///
    /// Validates: Requirement 15.1, 15.9
    pub fn settings_panel(id: TabId, document: DocumentHandle) -> Self {
        let mut viewport = ViewportModel::with_line_count(1);
        viewport.set_line_height(16);
        Self {
            id,
            kind: TabKind::SettingsPanel,
            title: "[SETTINGS]".to_string(),
            path: None,
            document,
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
        }
    }

    /// Create a File Explorer Panel tab (POM option 2).
    ///
    /// Validates: Requirement 19.11, 19.12
    pub fn file_explorer_panel(id: TabId, document: DocumentHandle) -> Self {
        let mut viewport = ViewportModel::with_line_count(1);
        viewport.set_line_height(16);
        Self {
            id,
            kind: TabKind::FileExplorerPanel,
            title: "[FILES]".to_string(),
            path: None,
            document,
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
        }
    }

    /// Create a Search Results panel tab.
    ///
    /// Validates: global-search Requirement 1.1
    pub fn search_results_panel(id: TabId, document: DocumentHandle) -> Self {
        let mut viewport = ViewportModel::with_line_count(1);
        viewport.set_line_height(16);
        Self {
            id,
            kind: TabKind::SearchResults,
            title: "[SEARCH]".to_string(),
            path: None,
            document,
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
        }
    }

    /// Create a Plugin Manager panel tab.
    ///
    /// Validates: plugin-manager-ui Requirement 1.1
    pub fn plugin_manager(id: TabId, document: DocumentHandle) -> Self {
        let mut viewport = ViewportModel::with_line_count(1);
        viewport.set_line_height(16);
        Self {
            id,
            kind: TabKind::PluginManager,
            title: "[PLUGINS]".to_string(),
            path: None,
            document,
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
        }
    }

    /// Create an Event Log panel tab.
    ///
    /// Validates: notification-system Requirement 2.1
    pub fn event_log(id: TabId, document: DocumentHandle) -> Self {
        let mut viewport = ViewportModel::with_line_count(1);
        viewport.set_line_height(16);
        Self {
            id,
            kind: TabKind::EventLog,
            title: "[LOG]".to_string(),
            path: None,
            document,
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
        }
    }

    /// Human-readable encoding label derived from the line-end mode.
    pub fn encoding_label(&self) -> &'static str {
        match self.line_end_mode {
            LineEndMode::Default => "UTF-8",
            LineEndMode::Unicode => "UTF-8 (Unicode)",
        }
    }
}
