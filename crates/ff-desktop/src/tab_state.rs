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
    /// In-app Workspace Kinds editor Context (configure a Kind's title / menu
    /// bar / key list / profile; create a new Kind modelled on a built-in base;
    /// save to `workspace-kinds/<name>.toml`).
    ///
    /// Validates: workspace-kinds Requirement 6 (CR-NR-090 B.4)
    KindsEditor,
    /// Context-sensitive Help Context -- displays the resolved help topic
    /// (F1 / HELP). Rendered via `HelpContextPanel` (a `WorkspaceContext`).
    ///
    /// Validates: context-help Requirement 18.2 (CR-NR-097)
    HelpContext,
    /// Screen Collection Replay viewer Context (CAPTURE REPLAY). Rendered via
    /// `ScrmViewerState` (a `WorkspaceContext`).
    ///
    /// Validates: screen-snapshot-scrm Requirement 16.1 (CR-NR-098)
    ScrmViewer,
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

// The `base_tab!` macro and all `TabState` constructor fns live in the sibling
// `tab_state_ctors` module (declared at the crate root) to keep this file under
// the 400-line rule; see `tab_state_ctors.rs`.

impl TabState {
    /// Human-readable encoding label derived from the line-end mode.
    pub fn encoding_label(&self) -> &'static str {
        match self.line_end_mode {
            LineEndMode::Default => "UTF-8",
            LineEndMode::Unicode => "UTF-8 (Unicode)",
        }
    }
}
