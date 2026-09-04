//! `SessionManager` — thin shell-side wrapper around `ff-session` persistence.
//!
//! Owns the `UserDataDir` and `SessionFile` handles. Called by `WorkbenchShell`
//! on startup (restore) and on exit (save).
//!
//! Addresses: Requirement 18.10 — session save/restore wired into ff-desktop.

use ff_session::session_state::{PersistedTabKind, TabState as SessionTabState};
use ff_session::{SessionFile, SessionState, UserDataDir};

use crate::catalog_registry::CatalogRegistry;
use crate::tab_manager::TabManager;
use crate::tab_state::TabKind;

/// Manages session persistence for the desktop shell.
pub struct SessionManager {
    session_file: SessionFile,
}

impl SessionManager {
    /// Initialise the User Data Directory and return a ready `SessionManager`.
    ///
    /// On failure (permissions, etc.) returns `None` — the shell continues
    /// without session persistence (graceful degradation, Req 11 AC 1).
    pub fn try_init() -> Option<Self> {
        let mut udd = UserDataDir::resolve(None).ok()?;
        udd.initialise().ok()?;
        let session_file = SessionFile::new(udd.session_file_path());
        Some(Self { session_file })
    }

    /// Create a `SessionManager` pointing at an explicit path (used in tests).
    #[cfg(test)]
    pub fn with_path(path: std::path::PathBuf) -> Self {
        Self {
            session_file: SessionFile::new(path),
        }
    }

    /// Load the persisted session state.
    ///
    /// Returns an empty state on first run or if the file is corrupt.
    pub fn load(&self) -> SessionState {
        self.session_file
            .load()
            .unwrap_or_else(|_| SessionState::empty())
    }

    /// Capture the current tab list and global zoom from the shell and persist them.
    ///
    /// Validates: Requirement 14.1 — session captures file tabs for restore.
    /// Validates: Requirement 11.3 — FilesPanel tab is persisted and restored.
    /// Validates: Requirement 3.1 (view-zoom) — global zoom offset persisted.
    /// Validates: Requirement 12.4 (function-keys-and-history) — key_bar_visible persisted.
    /// Validates: Requirement 23.9 (file-tree-panel) — sidebar_width persisted.
    pub fn save(
        &self,
        tabs: &TabManager,
        zoom_offset: i32,
        key_bar_visible: bool,
        file_explorer_sidebar_width: f32,
    ) {
        let session_tabs: Vec<SessionTabState> = tabs
            .tabs()
            .iter()
            .filter_map(|t| match t.kind {
                TabKind::FileEditor => t.path.as_ref().map(|path| SessionTabState {
                    tab_id: format!("{}", t.id.0),
                    tab_kind: PersistedTabKind::FileEditor,
                    uri: Some(path.clone()),
                    viewport_top_line: t.viewport.top_line() as usize,
                    viewport_horizontal_offset: 0,
                    caret_line: t.cursor.cursor_line() as usize,
                    caret_column: t.cursor.cursor_column() as usize,
                    selections: Vec::new(),
                    language_override: None,
                    is_pinned: false,
                    zoom_offset: 0,
                }),
                TabKind::FilesPanel => Some(SessionTabState {
                    tab_id: format!("{}", t.id.0),
                    tab_kind: PersistedTabKind::FilesPanel,
                    uri: None,
                    viewport_top_line: 1,
                    viewport_horizontal_offset: 0,
                    caret_line: 1,
                    caret_column: 1,
                    selections: Vec::new(),
                    language_override: None,
                    is_pinned: false,
                    zoom_offset: 0,
                }),
                TabKind::FileExplorerPanel => Some(SessionTabState {
                    tab_id: format!("{}", t.id.0),
                    tab_kind: PersistedTabKind::FileExplorerPanel,
                    uri: None,
                    viewport_top_line: 1,
                    viewport_horizontal_offset: 0,
                    caret_line: 1,
                    caret_column: 1,
                    selections: Vec::new(),
                    language_override: None,
                    is_pinned: false,
                    zoom_offset: 0,
                }),
                TabKind::PrimaryOptionMenu | TabKind::Untitled | TabKind::SettingsPanel => None,
            })
            .collect();

        let active_tab_id = {
            let active = tabs.active_tab();
            match active.kind {
                TabKind::FileEditor => active.path.as_ref().map(|_| format!("{}", active.id.0)),
                TabKind::FilesPanel => Some(format!("{}", active.id.0)),
                TabKind::PrimaryOptionMenu
                | TabKind::Untitled
                | TabKind::SettingsPanel
                | TabKind::FileExplorerPanel => None,
            }
        };
        // Note: FileExplorerPanel active_tab_id is None (no URI to track)

        let state = SessionState {
            tabs: session_tabs,
            active_tab_id,
            global_zoom_offset: zoom_offset,
            key_bar_visible,
            file_explorer_sidebar_width,
            ..SessionState::empty()
        };

        // Best-effort — ignore write errors (graceful degradation)
        let _ = self.session_file.save(&state);
    }

    /// Persist the catalog registry to `catalogs.toml` next to `session.toml`.
    ///
    /// Best-effort — write errors are silently ignored (graceful degradation).
    /// Validates: Requirement 2.1 (virtual-catalog-manager)
    pub fn save_catalog_registry(&self, registry: &CatalogRegistry) {
        let path = self.catalogs_path();
        let toml_str = registry.save_to_toml();
        // Best-effort — ignore write errors (graceful degradation)
        let _ = std::fs::write(&path, toml_str);
    }

    /// Load the catalog registry from `catalogs.toml` next to `session.toml`.
    ///
    /// Returns an empty registry if the file is absent or unreadable.
    /// Validates: Requirement 2.2 (virtual-catalog-manager)
    pub fn load_catalog_registry(&self) -> CatalogRegistry {
        let path = self.catalogs_path();
        match std::fs::read_to_string(&path) {
            Ok(contents) => CatalogRegistry::load_from_toml(&contents),
            Err(_) => CatalogRegistry::new(),
        }
    }

    /// Path to `catalogs.toml` — sibling of `session.toml`.
    fn catalogs_path(&self) -> std::path::PathBuf {
        let session_path = self.session_file.path();
        session_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("catalogs.toml")
    }

    /// Extract the ordered list of file URIs from a `SessionState`.
    ///
    /// Returns only tabs that have a URI (skips untitled placeholders).
    pub fn tab_uris(state: &SessionState) -> Vec<String> {
        state.tabs.iter().filter_map(|t| t.uri.clone()).collect()
    }

    /// Returns the tab IDs of all persisted `FilesPanel` tabs.
    ///
    /// Validates: Requirement 11.3
    // Used in session restore (Task 10.3) and tests.
    #[allow(dead_code)]
    pub fn files_panel_tab_ids(state: &SessionState) -> Vec<String> {
        state
            .tabs
            .iter()
            .filter(|t| t.tab_kind == PersistedTabKind::FilesPanel)
            .map(|t| t.tab_id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio::runtime::Runtime;

    use crate::tab_manager::TabManager;

    fn make_session_file(dir: &TempDir) -> PathBuf {
        dir.path().join("session.toml")
    }

    /// Validates: Requirement 18.10 — saving an empty tab list persists an
    /// empty session (no file-backed tabs → no URIs in session).
    #[test]
    fn save_with_no_file_tabs_produces_empty_uri_list() {
        // Validates: Requirement 4 AC 1 — only file-backed tabs are persisted
        let tmp = TempDir::new().expect("tempdir");
        let mgr = SessionManager::with_path(make_session_file(&tmp));
        let runtime = Runtime::new().expect("runtime");
        let tabs = TabManager::new(&runtime, "welcome\n");

        mgr.save(&tabs, 0, true, 200.0);

        let loaded = mgr.load();
        assert!(
            SessionManager::tab_uris(&loaded).is_empty(),
            "untitled tab must not be persisted"
        );
    }

    /// Validates: Requirement 18.10 — loading from a non-existent session file
    /// returns an empty state (first-run behaviour, Req 4 AC 7).
    #[test]
    fn load_from_missing_file_returns_empty_state() {
        // Validates: Requirement 4 AC 7 — absent session file → empty state
        let tmp = TempDir::new().expect("tempdir");
        let mgr = SessionManager::with_path(make_session_file(&tmp));

        let state = mgr.load();
        assert!(state.tabs.is_empty());
        assert!(state.active_tab_id.is_none());
    }

    /// Validates: Requirement 18.10 — tab_uris extracts URIs in order.
    #[test]
    fn tab_uris_returns_uris_in_tab_order() {
        // Validates: Requirement 4 AC 1 — ordered list of open file URIs
        let state = SessionState {
            tabs: vec![
                ff_session::session_state::TabState {
                    tab_id: "1".to_string(),
                    uri: Some("/a/file.txt".to_string()),
                    ..Default::default()
                },
                ff_session::session_state::TabState {
                    tab_id: "2".to_string(),
                    uri: Some("/b/other.rs".to_string()),
                    ..Default::default()
                },
                ff_session::session_state::TabState {
                    tab_id: "3".to_string(),
                    uri: None, // untitled — must be skipped
                    ..Default::default()
                },
            ],
            ..SessionState::empty()
        };

        let uris = SessionManager::tab_uris(&state);
        assert_eq!(uris.len(), 2);
        assert_eq!(uris[0], "/a/file.txt");
        assert_eq!(uris[1], "/b/other.rs");
    }

    /// Validates: Requirement 18.10 — save then load round-trips a file-backed
    /// tab's URI correctly.
    #[test]
    fn save_and_load_round_trips_file_uri() {
        // Validates: Requirement 4 AC 1, 2 — session file stores tab URIs
        let tmp = TempDir::new().expect("tempdir");
        let mgr = SessionManager::with_path(make_session_file(&tmp));

        // Build a minimal SessionState with one file tab directly
        let state = SessionState {
            tabs: vec![ff_session::session_state::TabState {
                tab_id: "42".to_string(),
                uri: Some("/some/file.rs".to_string()),
                viewport_top_line: 10,
                caret_line: 15,
                caret_column: 3,
                ..Default::default()
            }],
            active_tab_id: Some("42".to_string()),
            ..SessionState::empty()
        };
        mgr.session_file.save(&state).expect("save must succeed");

        let loaded = mgr.load();
        let uris = SessionManager::tab_uris(&loaded);
        assert_eq!(uris, vec!["/some/file.rs"]);
        assert_eq!(loaded.active_tab_id, Some("42".to_string()));
    }

    /// Validates: Requirement 11.3 — FilesPanel tab round-trips through session.
    #[test]
    fn files_panel_tab_round_trips_through_session() {
        // Validates: Requirement 11.3
        let tmp = TempDir::new().expect("tempdir");
        let mgr = SessionManager::with_path(make_session_file(&tmp));

        let state = SessionState {
            tabs: vec![ff_session::session_state::TabState {
                tab_id: "99".to_string(),
                tab_kind: ff_session::session_state::PersistedTabKind::FilesPanel,
                uri: None,
                ..Default::default()
            }],
            active_tab_id: Some("99".to_string()),
            ..SessionState::empty()
        };
        mgr.session_file.save(&state).expect("save must succeed");

        let loaded = mgr.load();
        let fp_ids = SessionManager::files_panel_tab_ids(&loaded);
        assert_eq!(fp_ids, vec!["99"]);
        // FilesPanel has no URI
        assert!(SessionManager::tab_uris(&loaded).is_empty());
    }

    /// Validates: Requirement 11.3 — files_panel_tab_ids returns empty when no FilesPanel tabs.
    #[test]
    fn files_panel_tab_ids_empty_when_no_files_panel_tabs() {
        // Validates: Requirement 11.3
        let state = SessionState {
            tabs: vec![ff_session::session_state::TabState {
                tab_id: "1".to_string(),
                tab_kind: ff_session::session_state::PersistedTabKind::FileEditor,
                uri: Some("/a.txt".to_string()),
                ..Default::default()
            }],
            ..SessionState::empty()
        };
        assert!(SessionManager::files_panel_tab_ids(&state).is_empty());
    }

    /// Validates: Requirement 11.3 — mixed tabs: file and FilesPanel both persisted correctly.
    #[test]
    fn mixed_tabs_file_and_files_panel_both_persisted() {
        // Validates: Requirement 11.3
        let tmp = TempDir::new().expect("tempdir");
        let mgr = SessionManager::with_path(make_session_file(&tmp));

        let state = SessionState {
            tabs: vec![
                ff_session::session_state::TabState {
                    tab_id: "1".to_string(),
                    tab_kind: ff_session::session_state::PersistedTabKind::FileEditor,
                    uri: Some("/a.txt".to_string()),
                    ..Default::default()
                },
                ff_session::session_state::TabState {
                    tab_id: "2".to_string(),
                    tab_kind: ff_session::session_state::PersistedTabKind::FilesPanel,
                    uri: None,
                    ..Default::default()
                },
            ],
            ..SessionState::empty()
        };
        mgr.session_file.save(&state).expect("save");

        let loaded = mgr.load();
        assert_eq!(SessionManager::tab_uris(&loaded), vec!["/a.txt"]);
        assert_eq!(SessionManager::files_panel_tab_ids(&loaded), vec!["2"]);
    }

    /// Validates: Requirement 2.1 (virtual-catalog-manager) — save_catalog_registry
    /// writes catalogs.toml and load_catalog_registry reads it back.
    #[test]
    fn save_and_load_catalog_registry_round_trips() {
        // Validates: Requirement 2.1
        use crate::catalog_registry::{CatalogRegistry, CatalogType, VirtualCatalog};
        let tmp = TempDir::new().expect("tempdir");
        let mgr = SessionManager::with_path(make_session_file(&tmp));

        let mut reg = CatalogRegistry::new();
        reg.register(VirtualCatalog {
            name: "PAYROLL".to_string(),
            catalog_type: CatalogType::Mainframe,
            path: "/catalogs/payroll".to_string(),
            description: Some("Payroll datasets".to_string()),
            auto_mount: true,
            default_hlq: Some("PAYROLL".to_string()),
            mount_point: None,
            read_only: false,
        })
        .expect("register");

        mgr.save_catalog_registry(&reg);
        let loaded = mgr.load_catalog_registry();

        assert_eq!(loaded.list().len(), 1);
        assert_eq!(loaded.list()[0].name, "PAYROLL");
    }

    /// Validates: Requirement 2.2 (virtual-catalog-manager) — load_catalog_registry
    /// returns an empty registry when catalogs.toml does not exist.
    #[test]
    fn load_missing_catalog_file_returns_empty_registry() {
        // Validates: Requirement 2.2
        let tmp = TempDir::new().expect("tempdir");
        let mgr = SessionManager::with_path(make_session_file(&tmp));

        let loaded = mgr.load_catalog_registry();
        assert!(
            loaded.list().is_empty(),
            "absent catalogs.toml must yield empty registry"
        );
    }

    /// Validates: Requirement 12.4 (function-keys-and-history) — key_bar_visible
    /// is persisted and restored across sessions.
    #[test]
    fn key_bar_visible_round_trips_through_session() {
        // Validates: Requirement 12.4
        let tmp = TempDir::new().expect("tempdir");
        let mgr = SessionManager::with_path(make_session_file(&tmp));

        // Save with key_bar_visible = false
        let state = SessionState {
            key_bar_visible: false,
            ..SessionState::empty()
        };
        mgr.session_file.save(&state).expect("save");
        let loaded = mgr.load();
        assert!(
            !loaded.key_bar_visible,
            "key_bar_visible false must survive round-trip"
        );

        // Save with key_bar_visible = true
        let state2 = SessionState {
            key_bar_visible: true,
            ..SessionState::empty()
        };
        mgr.session_file.save(&state2).expect("save");
        let loaded2 = mgr.load();
        assert!(
            loaded2.key_bar_visible,
            "key_bar_visible true must survive round-trip"
        );
    }

    /// Validates: Requirement 23.9 (file-tree-panel) — sidebar_width round-trips through session.
    #[test]
    fn file_explorer_sidebar_width_round_trips_through_session() {
        // Validates: Requirement 23.9
        let tmp = TempDir::new().expect("tempdir");
        let mgr = SessionManager::with_path(make_session_file(&tmp));

        let state = ff_session::SessionState {
            file_explorer_sidebar_width: 350.0,
            ..ff_session::SessionState::empty()
        };
        mgr.session_file.save(&state).expect("save");
        let loaded = mgr.load();
        assert!(
            (loaded.file_explorer_sidebar_width - 350.0).abs() < f32::EPSILON,
            "sidebar_width must survive round-trip"
        );
    }
}
