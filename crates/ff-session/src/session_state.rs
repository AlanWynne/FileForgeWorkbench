//! Session state data model — the complete serialisable snapshot of
//! the user's workspace persisted across restarts.
//!
//! Addresses: Requirement 4 (Session State Persistence), Requirement 5

use serde::{Deserialize, Serialize};

/// Current schema version for the session state format.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// The complete serialisable snapshot of the user's workspace.
///
/// Persisted to `session.toml` and restored on next launch. Contains
/// all state needed to reconstruct the user's workspace: open tabs,
/// viewport positions, layout, window geometry, and recent files.
///
/// Addresses: Requirement 4 AC 4.1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionState {
    /// Schema version for forward/backward compatibility.
    pub schema_version: u32,

    /// Ordered list of open tabs with their per-tab state.
    pub tabs: Vec<TabState>,

    /// The tab ID of the active (focused) tab at save time.
    pub active_tab_id: Option<String>,

    /// The layout state snapshot (panel positions, tab groups, splitters, persona).
    pub layout: Option<LayoutSnapshot>,

    /// Window geometry for primary and floating windows.
    pub windows: Vec<WindowGeometryState>,

    /// Recent files list with timestamps and metadata.
    pub recent_files: Vec<RecentFileEntry>,

    /// Active configuration profile name.
    pub active_profile: Option<String>,

    /// ISO 8601 timestamp of when this session was last saved.
    pub last_saved: Option<String>,

    /// Whether the Primary Option Menu floating window is visible.
    ///
    /// Addresses: Requirement 14.9
    #[serde(default = "default_true")]
    pub show_pom: bool,

    /// Global application zoom offset — applies to all windows and panels.
    ///
    /// Addresses: Requirement 3.1 (view-zoom) — single zoom level carried across all contexts.
    #[serde(default)]
    pub global_zoom_offset: i32,

    /// Whether the Key_Label_Bar is visible in the footer region.
    ///
    /// Addresses: Requirement 12.4 (function-keys-and-history) — PFSHOW visibility persisted.
    #[serde(default = "default_true")]
    pub key_bar_visible: bool,

    /// Width of the File Explorer Panel sidebar in logical pixels.
    ///
    /// Addresses: Requirement 23.9 (file-tree-panel) -- sidebar width persisted.
    #[serde(default = "default_sidebar_width")]
    pub file_explorer_sidebar_width: f32,

    /// Path to the `.ffwb-workspace` file that was active at last exit.
    ///
    /// `None` means no workspace was active. Restored at startup before tabs.
    ///
    /// Addresses: workspace-model Requirement 5.1, 5.2
    #[serde(default)]
    pub active_workspace_path: Option<String>,

    /// Recently-used command IDs executed via the Command Palette (most recent first).
    ///
    /// Capped at 10 entries. Persisted across sessions.
    ///
    /// Addresses: command-palette Requirement 5.2
    #[serde(default)]
    pub recent_palette_commands: Vec<String>,

    /// Search query history for the Global Search panel (most recent first).
    ///
    /// Capped at 20 entries. Persisted across sessions.
    ///
    /// Addresses: global-search Requirement 6.1, 6.2
    #[serde(default)]
    pub search_history: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_sidebar_width() -> f32 {
    200.0
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            tabs: Vec::new(),
            active_tab_id: None,
            layout: None,
            windows: Vec::new(),
            recent_files: Vec::new(),
            active_profile: None,
            last_saved: None,
            show_pom: true,
            global_zoom_offset: 0,
            key_bar_visible: true,
            file_explorer_sidebar_width: 200.0,
            active_workspace_path: None,
            recent_palette_commands: Vec::new(),
            search_history: Vec::new(),
        }
    }
}

impl SessionState {
    /// Create an empty session state (first run or reset).
    pub fn empty() -> Self {
        Self::default()
    }

    /// Attempt to migrate from an older schema version to current.
    ///
    /// If the state is already at the current version, returns it unchanged.
    /// Future versions may add migration logic for schema evolution.
    ///
    /// Addresses: Requirement 4 AC 4.6
    pub fn migrate(mut state: Self) -> Self {
        // Currently only version 1 exists. Future migrations would be
        // handled here as match arms on state.schema_version.
        if state.schema_version < CURRENT_SCHEMA_VERSION {
            // Placeholder for future schema migration logic.
            // Each version bump would have a migration path here.
            state.schema_version = CURRENT_SCHEMA_VERSION;
        }
        state
    }

    /// Check whether this state has any meaningful content to restore.
    pub fn has_content(&self) -> bool {
        !self.tabs.is_empty() || self.layout.is_some() || !self.windows.is_empty()
    }
}

/// The kind of a persisted tab — used to reconstruct the correct tab type on restore.
///
/// Addresses: Requirement 11.3 (FilesPanel tab persistence)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PersistedTabKind {
    /// A file editor tab backed by a URI.
    #[default]
    FileEditor,
    /// The ISPF Primary Option Menu tab.
    PrimaryOptionMenu,
    /// The Virtual Catalog Manager (Files Panel) tab.
    FilesPanel,
    /// The File Explorer Panel tab (POM option 2 — tree view of catalog contents).
    ///
    /// Validates: Requirement 19.12
    FileExplorerPanel,
    /// The Global Search Results panel tab.
    ///
    /// Validates: global-search Requirement 1.1
    SearchResults,
    /// The Plugin Manager panel tab (POM option 8).
    ///
    /// Validates: plugin-manager-ui Requirement 1.1
    PluginManager,
    /// The Event Log panel tab.
    ///
    /// Validates: notification-system Requirement 2.1
    EventLog,
    /// An untitled buffer with no backing file.
    Untitled,
}

/// The data-file enumeration of built-in Contexts a Workspace can display.
///
/// This is the session-layer counterpart of the runtime `TabKind`. It replaces
/// the closed `PersistedTabKind` enum as the primary persistence model; the
/// legacy `PersistedTabKind` is retained only to load older `session.toml`
/// files (see `WorkspaceDescriptor::from_legacy`).
///
/// Serialises in `snake_case` so it reads naturally in a data file.
///
/// Validates: startup-and-session Requirement 21 (Workspace_Kind).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum WorkspaceKind {
    /// A file editor Context backed by a URI.
    Editor,
    /// The Virtual Catalog Manager (Files Panel) Context.
    Files,
    /// The File Explorer tree Context.
    FileExplorer,
    /// The Settings Context (optionally namespace-filtered).
    Settings,
    /// The Global Search Results Context.
    Search,
    /// The Plugin Manager Context.
    PluginManager,
    /// The Event Log Context.
    EventLog,
    /// The Lua Macro Library Context.
    MacroLibrary,
    /// The Command Configurator Context (command-configurator sub-project).
    CommandConfigurator,
    /// The Home Context / Primary Option Menu.
    ///
    /// Present so a POM opened as a Custom Workspace round-trips; note the POM
    /// is normally guaranteed by the always-present rule rather than persisted.
    PrimaryOptionMenu,
    /// An untitled buffer with no backing file.
    Untitled,
}

/// A single value inside a `WorkspaceDescriptor` parameter map.
///
/// A serde-friendly, deterministically-ordered value model so a descriptor
/// round-trips through TOML. Kept independent of `ff-command` so the persisted
/// session format is owned entirely by the session layer.
///
/// Validates: startup-and-session Requirement 21.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DescriptorValue {
    /// A boolean value.
    Boolean(bool),
    /// A 64-bit signed integer value.
    Integer(i64),
    /// A string value.
    String(String),
}

impl From<bool> for DescriptorValue {
    fn from(v: bool) -> Self {
        Self::Boolean(v)
    }
}

impl From<i64> for DescriptorValue {
    fn from(v: i64) -> Self {
        Self::Integer(v)
    }
}

impl From<&str> for DescriptorValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl From<String> for DescriptorValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

/// An ordered, serialisable parameter bag carried by a `CustomWorkspace`
/// descriptor (e.g. a Settings namespace filter, an editor URI). `BTreeMap`
/// gives deterministic ordering for stable TOML output and reproducible tests.
pub type DescriptorParams = std::collections::BTreeMap<String, DescriptorValue>;

/// The persisted description of one visible Workspace.
///
/// Exactly one of `Menu { name }` or `CustomWorkspace { workspace_kind,
/// params }`, mirroring the corresponding Command_Target variants
/// (command-framework Requirement 8). Non-visible actions (Started Tasks,
/// functions, macros) are never persisted, so no variant represents them.
///
/// Validates: startup-and-session Requirement 21.1.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkspaceDescriptor {
    /// Open the Menu Workspace backed by `menus/<name>.toml`.
    Menu {
        /// The menu name (without extension); `pom` is the Home Context.
        name: String,
    },
    /// Open a built-in Context, optionally parameterised.
    CustomWorkspace {
        /// Which built-in Context to open.
        workspace_kind: WorkspaceKind,
        /// Optional typed parameters (e.g. `namespace`, `uri`).
        #[serde(default)]
        params: DescriptorParams,
    },
}

impl WorkspaceDescriptor {
    /// Derive a descriptor from the legacy `PersistedTabKind` + `uri` fields of
    /// a `TabState` that was written before descriptors existed.
    ///
    /// Returns `None` for legacy kinds that were never actually restored and
    /// carry no useful descriptor (e.g. `Untitled` with no uri), so the restore
    /// loop can skip them exactly as before.
    ///
    /// Validates: startup-and-session Requirement 21.10 (backward compatibility).
    pub fn from_legacy(kind: &PersistedTabKind, uri: Option<&str>) -> Option<Self> {
        let ws = |k: WorkspaceKind| Self::CustomWorkspace {
            workspace_kind: k,
            params: DescriptorParams::new(),
        };
        match kind {
            PersistedTabKind::FileEditor => {
                let mut params = DescriptorParams::new();
                params.insert("uri".to_string(), DescriptorValue::from(uri?));
                Some(Self::CustomWorkspace {
                    workspace_kind: WorkspaceKind::Editor,
                    params,
                })
            }
            PersistedTabKind::FilesPanel => Some(ws(WorkspaceKind::Files)),
            PersistedTabKind::FileExplorerPanel => Some(ws(WorkspaceKind::FileExplorer)),
            PersistedTabKind::SearchResults => Some(ws(WorkspaceKind::Search)),
            PersistedTabKind::PluginManager => Some(ws(WorkspaceKind::PluginManager)),
            PersistedTabKind::EventLog => Some(ws(WorkspaceKind::EventLog)),
            PersistedTabKind::PrimaryOptionMenu => Some(ws(WorkspaceKind::PrimaryOptionMenu)),
            // Untitled tabs were never restored (no uri), so no descriptor.
            PersistedTabKind::Untitled => None,
        }
    }
}

/// Per-tab state persisted as part of the session.
///
/// This is the session-layer view of a tab — not the full runtime Tab object.
///
/// Addresses: Requirement 4 AC 4.1, Requirement 5 AC 5.2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabState {
    /// Unique tab identifier (stable across session save/restore).
    pub tab_id: String,

    /// The Workspace_Descriptor for this tab -- the primary persistence model
    /// (startup-and-session Requirement 21). `None` for tabs loaded from a
    /// legacy `session.toml` written before descriptors existed; use
    /// [`TabState::effective_descriptor`] to obtain a descriptor for either
    /// case.
    ///
    /// Validates: startup-and-session Requirement 21.1, 21.10.
    #[serde(default)]
    pub descriptor: Option<WorkspaceDescriptor>,

    /// The kind of this tab — determines how it is reconstructed on restore.
    ///
    /// Legacy field retained for backward-compatible loading of older sessions
    /// (Requirement 21.10). New saves populate `descriptor`; this field is kept
    /// as a fallback and for the legacy mapping in
    /// [`WorkspaceDescriptor::from_legacy`].
    ///
    /// Addresses: Requirement 11.3
    #[serde(default)]
    pub tab_kind: PersistedTabKind,

    /// Resource URI of the open file (None for untitled documents).
    pub uri: Option<String>,

    /// The 1-based line number at the top of the viewport.
    pub viewport_top_line: usize,

    /// Horizontal scroll offset in columns.
    pub viewport_horizontal_offset: usize,

    /// Caret position: line (1-based).
    pub caret_line: usize,

    /// Caret position: column (1-based).
    pub caret_column: usize,

    /// Selection ranges (empty vec = no selection).
    pub selections: Vec<SelectionRange>,

    /// Language override if the user manually set the language.
    pub language_override: Option<String>,

    /// Whether this tab was pinned.
    pub is_pinned: bool,

    /// Zoom offset persisted for this tab.
    ///
    /// Addresses: Requirement 6.1 (view-zoom) — per-document zoom offset persisted.
    #[serde(default)]
    pub zoom_offset: i32,

    /// User-assigned Workspace name (optional).
    ///
    /// When `Some`, displayed in the tab header alongside the content title.
    /// When `None`, only the content-derived title is shown.
    ///
    /// Validates: CX Requirement 1.1, 1.6, Requirement 4.1-4.3
    #[serde(default)]
    pub workspace_name: Option<String>,
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            tab_id: String::new(),
            descriptor: None,
            tab_kind: PersistedTabKind::FileEditor,
            uri: None,
            viewport_top_line: 1,
            viewport_horizontal_offset: 0,
            caret_line: 1,
            caret_column: 1,
            selections: Vec::new(),
            language_override: None,
            is_pinned: false,
            zoom_offset: 0,
            workspace_name: None,
        }
    }
}

impl TabState {
    /// Return the effective Workspace_Descriptor for this tab.
    ///
    /// Prefers the explicit `descriptor` (new format). When it is absent
    /// (a legacy `session.toml` written before descriptors existed), derive one
    /// from the legacy `tab_kind` + `uri` fields. Returns `None` only when the
    /// legacy tab carried no restorable descriptor (e.g. an untitled buffer),
    /// so the restore loop skips it exactly as the old URI-only loop did.
    ///
    /// Validates: startup-and-session Requirement 21.1, 21.5, 21.10.
    pub fn effective_descriptor(&self) -> Option<WorkspaceDescriptor> {
        if let Some(d) = &self.descriptor {
            return Some(d.clone());
        }
        WorkspaceDescriptor::from_legacy(&self.tab_kind, self.uri.as_deref())
    }
}

/// A serialisable selection range within a document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionRange {
    /// Start line (1-based).
    pub start_line: usize,
    /// Start column (1-based).
    pub start_column: usize,
    /// End line (1-based).
    pub end_line: usize,
    /// End column (1-based).
    pub end_column: usize,
}

/// A serialisable snapshot of the layout state for session persistence.
///
/// This is a thin wrapper around the layout-and-docking crate's serialisation
/// format, stored as a TOML-compatible nested structure.
///
/// Addresses: Requirement 4 AC 4.1 (layout portion), Requirement 5 AC 5.1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutSnapshot {
    /// The serialised layout data as a TOML value.
    pub data: toml::Value,
    /// The active persona name at save time.
    pub persona: Option<String>,
}

/// Window geometry state persisted as part of the session.
///
/// Addresses: Requirement 8
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WindowGeometryState {
    /// Unique identifier for this window ("primary" or floating panel key).
    pub window_id: String,
    /// Horizontal position in logical pixels.
    pub x: i32,
    /// Vertical position in logical pixels.
    pub y: i32,
    /// Window width in logical pixels.
    pub width: u32,
    /// Window height in logical pixels.
    pub height: u32,
    /// Whether the window is maximised.
    pub is_maximised: bool,
    /// Whether the window is in fullscreen mode.
    pub is_fullscreen: bool,
    /// Display identifier where the window was last seen.
    pub display_id: Option<String>,
}

impl WindowGeometryState {
    /// The identifier used for the primary application window.
    pub const PRIMARY_WINDOW_ID: &'static str = "primary";

    /// Create geometry for the primary window.
    pub fn primary(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            window_id: Self::PRIMARY_WINDOW_ID.to_string(),
            x,
            y,
            width,
            height,
            is_maximised: false,
            is_fullscreen: false,
            display_id: None,
        }
    }
}

/// An entry in the recent files list persisted with the session.
///
/// Addresses: Requirement 4 AC 4.4, 4.5
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentFileEntry {
    /// The resource URI.
    pub uri: String,
    /// Display name (filename portion).
    pub display_name: String,
    /// Last access timestamp (ISO 8601 string for TOML serialisation).
    pub last_accessed: String,
    /// Last known viewport top line (for restoring position on reopen).
    pub last_viewport_top_line: Option<usize>,
    /// Whether the file was confirmed to exist at last session load.
    pub available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_session_state_has_current_schema_version() {
        // Validates: Requirement 4 AC 4.6
        let state = SessionState::default();
        assert_eq!(state.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn empty_session_state_has_no_content() {
        let state = SessionState::empty();
        assert!(!state.has_content());
        assert!(state.tabs.is_empty());
        assert!(state.layout.is_none());
        assert!(state.windows.is_empty());
        assert!(state.recent_files.is_empty());
        assert!(state.active_tab_id.is_none());
        assert!(state.active_profile.is_none());
    }

    #[test]
    fn session_state_with_tabs_has_content() {
        let mut state = SessionState::empty();
        state.tabs.push(TabState::default());
        assert!(state.has_content());
    }

    #[test]
    fn session_state_with_layout_has_content() {
        let mut state = SessionState::empty();
        state.layout = Some(LayoutSnapshot {
            data: toml::Value::Table(toml::map::Map::new()),
            persona: None,
        });
        assert!(state.has_content());
    }

    #[test]
    fn session_state_with_windows_has_content() {
        let mut state = SessionState::empty();
        state
            .windows
            .push(WindowGeometryState::primary(100, 100, 1024, 768));
        assert!(state.has_content());
    }

    #[test]
    fn migrate_preserves_current_version_state() {
        // Validates: Requirement 4 AC 4.6
        let state = SessionState {
            schema_version: CURRENT_SCHEMA_VERSION,
            tabs: vec![TabState {
                tab_id: "tab1".to_string(),
                uri: Some("file.txt".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let migrated = SessionState::migrate(state.clone());
        assert_eq!(migrated, state);
    }

    #[test]
    fn migrate_upgrades_older_version() {
        // Validates: Requirement 4 AC 4.6
        let state = SessionState {
            schema_version: 0,
            ..Default::default()
        };

        let migrated = SessionState::migrate(state);
        assert_eq!(migrated.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn tab_state_default_has_valid_positions() {
        let tab = TabState::default();
        assert_eq!(tab.viewport_top_line, 1);
        assert_eq!(tab.viewport_horizontal_offset, 0);
        assert_eq!(tab.caret_line, 1);
        assert_eq!(tab.caret_column, 1);
        assert!(tab.selections.is_empty());
        assert!(!tab.is_pinned);
    }

    #[test]
    fn window_geometry_primary_creates_primary_window() {
        let geom = WindowGeometryState::primary(100, 200, 1920, 1080);
        assert_eq!(geom.window_id, "primary");
        assert_eq!(geom.x, 100);
        assert_eq!(geom.y, 200);
        assert_eq!(geom.width, 1920);
        assert_eq!(geom.height, 1080);
        assert!(!geom.is_maximised);
        assert!(!geom.is_fullscreen);
        assert_eq!(geom.display_id, None);
    }

    #[test]
    fn active_workspace_path_round_trips_through_session() {
        // Validates: workspace-model Requirement 5.1, 5.2
        let state = SessionState {
            active_workspace_path: Some("/projects/myapp.ffwb-workspace".to_string()),
            ..Default::default()
        };
        let serialized = toml::to_string_pretty(&state).unwrap();
        let loaded: SessionState = toml::from_str(&serialized).unwrap();
        assert_eq!(loaded.active_workspace_path, state.active_workspace_path);
    }

    #[test]
    fn active_workspace_path_defaults_to_none() {
        // Validates: workspace-model Requirement 5.4
        let state = SessionState::default();
        assert!(state.active_workspace_path.is_none());
    }

    #[test]
    fn schema_version_constant_matches_default() {
        assert_eq!(CURRENT_SCHEMA_VERSION, 1);
        assert_eq!(SessionState::default().schema_version, 1);
    }

    // === Workspace_Descriptor persistence (Requirement 21) =================

    #[test]
    fn menu_descriptor_round_trips_through_toml() {
        // Validates: Requirement 21.1, 21.4
        let state = SessionState {
            tabs: vec![TabState {
                tab_id: "1".to_string(),
                descriptor: Some(WorkspaceDescriptor::Menu {
                    name: "notes".to_string(),
                }),
                ..Default::default()
            }],
            ..Default::default()
        };
        let toml = toml::to_string_pretty(&state).unwrap();
        let loaded: SessionState = toml::from_str(&toml).unwrap();
        assert_eq!(
            loaded.tabs[0].descriptor,
            Some(WorkspaceDescriptor::Menu {
                name: "notes".to_string()
            })
        );
    }

    #[test]
    fn settings_namespace_descriptor_round_trips() {
        // Validates: Requirement 21.3 -- Settings namespace filter persists
        let mut params = DescriptorParams::new();
        params.insert("namespace".to_string(), DescriptorValue::from("editor"));
        let state = SessionState {
            tabs: vec![TabState {
                tab_id: "3".to_string(),
                descriptor: Some(WorkspaceDescriptor::CustomWorkspace {
                    workspace_kind: WorkspaceKind::Settings,
                    params,
                }),
                ..Default::default()
            }],
            ..Default::default()
        };
        let toml = toml::to_string_pretty(&state).unwrap();
        let loaded: SessionState = toml::from_str(&toml).unwrap();
        match &loaded.tabs[0].descriptor {
            Some(WorkspaceDescriptor::CustomWorkspace {
                workspace_kind,
                params,
            }) => {
                assert_eq!(*workspace_kind, WorkspaceKind::Settings);
                assert_eq!(
                    params.get("namespace"),
                    Some(&DescriptorValue::String("editor".to_string()))
                );
            }
            other => panic!("expected settings CustomWorkspace, got {other:?}"),
        }
    }

    #[test]
    fn editor_descriptor_round_trips_with_uri_param() {
        // Validates: Requirement 21.2
        let mut params = DescriptorParams::new();
        params.insert("uri".to_string(), DescriptorValue::from("/a/b.txt"));
        let desc = WorkspaceDescriptor::CustomWorkspace {
            workspace_kind: WorkspaceKind::Editor,
            params,
        };
        let state = SessionState {
            tabs: vec![TabState {
                tab_id: "1".to_string(),
                descriptor: Some(desc.clone()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let toml = toml::to_string_pretty(&state).unwrap();
        let loaded: SessionState = toml::from_str(&toml).unwrap();
        assert_eq!(loaded.tabs[0].descriptor, Some(desc));
    }

    #[test]
    fn effective_descriptor_prefers_explicit_descriptor() {
        // Validates: Requirement 21.1
        let tab = TabState {
            descriptor: Some(WorkspaceDescriptor::Menu {
                name: "pom".to_string(),
            }),
            // Legacy field says something else; explicit descriptor must win.
            tab_kind: PersistedTabKind::FilesPanel,
            ..Default::default()
        };
        assert_eq!(
            tab.effective_descriptor(),
            Some(WorkspaceDescriptor::Menu {
                name: "pom".to_string()
            })
        );
    }

    #[test]
    fn legacy_file_editor_maps_to_editor_descriptor_with_uri() {
        // Validates: Requirement 21.10 -- old FileEditor tab still restorable
        let tab = TabState {
            descriptor: None,
            tab_kind: PersistedTabKind::FileEditor,
            uri: Some("/legacy/file.rs".to_string()),
            ..Default::default()
        };
        match tab.effective_descriptor() {
            Some(WorkspaceDescriptor::CustomWorkspace {
                workspace_kind,
                params,
            }) => {
                assert_eq!(workspace_kind, WorkspaceKind::Editor);
                assert_eq!(
                    params.get("uri"),
                    Some(&DescriptorValue::String("/legacy/file.rs".to_string()))
                );
            }
            other => panic!("expected Editor CustomWorkspace, got {other:?}"),
        }
    }

    #[test]
    fn legacy_files_and_file_explorer_map_to_descriptors() {
        // Validates: Requirement 21.10 -- FilesPanel/FileExplorerPanel restorable
        let files = TabState {
            tab_kind: PersistedTabKind::FilesPanel,
            ..Default::default()
        };
        let explorer = TabState {
            tab_kind: PersistedTabKind::FileExplorerPanel,
            ..Default::default()
        };
        assert_eq!(
            files.effective_descriptor(),
            Some(WorkspaceDescriptor::CustomWorkspace {
                workspace_kind: WorkspaceKind::Files,
                params: DescriptorParams::new(),
            })
        );
        assert_eq!(
            explorer.effective_descriptor(),
            Some(WorkspaceDescriptor::CustomWorkspace {
                workspace_kind: WorkspaceKind::FileExplorer,
                params: DescriptorParams::new(),
            })
        );
    }

    #[test]
    fn legacy_untitled_has_no_descriptor() {
        // Validates: Requirement 21.10 -- untitled tabs were never restored
        let tab = TabState {
            descriptor: None,
            tab_kind: PersistedTabKind::Untitled,
            uri: None,
            ..Default::default()
        };
        assert_eq!(tab.effective_descriptor(), None);
    }

    #[test]
    fn legacy_session_toml_without_descriptor_field_still_loads() {
        // Validates: Requirement 21.10 -- a session.toml written before the
        // descriptor field existed deserialises, with descriptor defaulting to
        // None and the legacy tab_kind preserved.
        let legacy = r#"
schema_version = 1

[[tabs]]
tab_id = "1"
tab_kind = "file_editor"
uri = "/x/y.txt"
viewport_top_line = 1
viewport_horizontal_offset = 0
caret_line = 1
caret_column = 1
selections = []
is_pinned = false
"#;
        let loaded: SessionState = toml::from_str(legacy).unwrap();
        assert_eq!(loaded.tabs.len(), 1);
        assert!(loaded.tabs[0].descriptor.is_none());
        assert_eq!(loaded.tabs[0].tab_kind, PersistedTabKind::FileEditor);
        // And it still yields a restorable descriptor.
        assert!(loaded.tabs[0].effective_descriptor().is_some());
    }
}
