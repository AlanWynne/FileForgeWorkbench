//! Menu Workspace pattern -- data model and public API.
//!
//! A Menu_Workspace is a Workspace whose Context is a list of options loaded
//! from a TOML file at runtime. The POM and Settings Context are instances of
//! this pattern backed by named TOML files.
//!
//! Validates: Requirement 1 (menu-workspace)

pub mod commands;
pub mod defaults;
pub mod loader;
pub mod render;

use std::path::{Path, PathBuf};
use std::time::SystemTime;

// === MenuOption =============================================================

/// A single option entry in a Menu_File.
///
/// Validates: Requirement 1.2, 1.3
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuOption {
    /// 1-4 character key, stored uppercase.
    pub key: String,
    /// Primary command executed when this option is selected.
    pub command: String,
    /// One-line description shown next to the key.
    pub description: String,
    /// When false the option is displayed disabled and cannot be selected.
    pub enabled: bool,
    /// Optional group label for visual separation.
    pub group: Option<String>,
}

// === MenuFile ===============================================================

/// The parsed content of a Menu_File.
///
/// Validates: Requirement 1.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuFile {
    /// Title displayed at the top of the Menu_Workspace.
    pub title: String,
    /// Ordered list of options.
    pub options: Vec<MenuOption>,
}

// === MenuWorkspaceState =====================================================

/// Per-tab state for a Menu_Workspace.
///
/// Stored on `TabState` and serialised to session TOML as
/// `PersistedTabKind::MenuWorkspace { file_path }`.
///
/// Validates: Requirement 1, 4 (menu-workspace)
#[derive(Debug, Clone)]
pub struct MenuWorkspaceState {
    /// Absolute path to the backing Menu_File.
    pub file_path: PathBuf,
    /// Successfully loaded menu, or `None` when a load error occurred.
    pub menu: Option<MenuFile>,
    /// Human-readable load error message, populated when `menu` is `None`.
    pub load_error: Option<String>,
    /// Modification time of the file at last successful load, used for hot-reload.
    pub last_modified: Option<SystemTime>,
}

impl MenuWorkspaceState {
    /// Create a new state by loading the menu file at `path`.
    ///
    /// Validates: Requirement 1.5, 1.6
    #[allow(dead_code)]
    pub fn load(path: impl AsRef<Path>) -> Self {
        let file_path = path.as_ref().to_path_buf();
        let last_modified = std::fs::metadata(&file_path)
            .ok()
            .and_then(|m| m.modified().ok());
        match loader::load_menu_file(&file_path) {
            Ok(menu) => Self {
                file_path,
                menu: Some(menu),
                load_error: None,
                last_modified,
            },
            Err(e) => Self {
                file_path,
                menu: None,
                load_error: Some(e),
                last_modified,
            },
        }
    }

    /// Poll for file changes and reload if the modification time has changed.
    ///
    /// Validates: Requirement 4.3, 4.5
    pub fn poll_reload(&mut self) {
        if let Ok(meta) = std::fs::metadata(&self.file_path) {
            if let Ok(modified) = meta.modified() {
                if Some(modified) != self.last_modified {
                    self.last_modified = Some(modified);
                    match loader::load_menu_file(&self.file_path) {
                        Ok(menu) => {
                            self.menu = Some(menu);
                            self.load_error = None;
                        }
                        Err(e) => {
                            // Req 4.5: retain previous menu on parse error
                            self.load_error = Some(e);
                        }
                    }
                }
            }
        }
    }

    /// Return the tab title derived from the loaded menu title, or a fallback.
    #[allow(dead_code)]
    pub fn tab_title(&self) -> String {
        self.menu
            .as_ref()
            .map(|m| format!("[{}]", m.title.to_uppercase()))
            .unwrap_or_else(|| "[MENU]".to_string())
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_toml(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("tempfile");
        f.write_all(content.as_bytes()).expect("write");
        f
    }

    // Validates: Requirement 1.1 -- MenuWorkspaceState::load parses a valid file
    #[test]
    fn menu_workspace_state_load_valid_file() {
        let toml = r#"
title = "Test Menu"
[[options]]
key = "1"
command = "FILES"
description = "File Explorer"
"#;
        let f = write_toml(toml);
        let state = MenuWorkspaceState::load(f.path());
        assert!(state.menu.is_some());
        assert!(state.load_error.is_none());
        let menu = state.menu.unwrap();
        assert_eq!(menu.title, "Test Menu");
        assert_eq!(menu.options.len(), 1);
        assert_eq!(menu.options[0].key, "1");
    }

    // Validates: Requirement 1.5 -- missing file produces load_error
    #[test]
    fn menu_workspace_state_load_missing_file_sets_error() {
        let state = MenuWorkspaceState::load("/nonexistent/path/menu.toml");
        assert!(state.menu.is_none());
        assert!(state.load_error.is_some());
        let err = state.load_error.unwrap();
        assert!(err.contains("Menu file not found"));
    }

    // Validates: Requirement 4.5 -- poll_reload retains previous menu on parse error
    #[test]
    fn poll_reload_retains_previous_menu_on_parse_error() {
        let toml = r#"
title = "Good Menu"
[[options]]
key = "1"
command = "FILES"
description = "Files"
"#;
        let mut f = NamedTempFile::new().expect("tempfile");
        f.write_all(toml.as_bytes()).expect("write");
        let mut state = MenuWorkspaceState::load(f.path());
        assert!(state.menu.is_some());

        // Overwrite with invalid TOML -- force a different mtime
        std::thread::sleep(std::time::Duration::from_millis(10));
        f.reopen().expect("reopen");
        std::fs::write(f.path(), b"not valid toml [[[").expect("write bad");
        state.poll_reload();

        // Previous menu retained; error set
        assert!(state.menu.is_some(), "previous menu must be retained");
        assert!(state.load_error.is_some());
    }

    // Validates: Requirement 1.1 -- tab_title derived from menu title
    #[test]
    fn tab_title_derived_from_menu_title() {
        let toml = "title = \"Primary Option Menu\"\n[[options]]\nkey=\"0\"\ncommand=\"SETTINGS\"\ndescription=\"Settings\"\n";
        let f = write_toml(toml);
        let state = MenuWorkspaceState::load(f.path());
        assert_eq!(state.tab_title(), "[PRIMARY OPTION MENU]");
    }

    // Validates: Requirement 1.5 -- tab_title fallback when no menu loaded
    #[test]
    fn tab_title_fallback_when_no_menu() {
        let state = MenuWorkspaceState::load("/nonexistent/menu.toml");
        assert_eq!(state.tab_title(), "[MENU]");
    }
}
