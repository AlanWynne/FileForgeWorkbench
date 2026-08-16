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
}

fn default_true() -> bool {
    true
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
    /// An untitled buffer with no backing file.
    Untitled,
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

    /// The kind of this tab — determines how it is reconstructed on restore.
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
}

impl Default for TabState {
    fn default() -> Self {
        Self {
            tab_id: String::new(),
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
        }
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
    fn schema_version_constant_matches_default() {
        assert_eq!(CURRENT_SCHEMA_VERSION, 1);
        assert_eq!(SessionState::default().schema_version, 1);
    }
}
