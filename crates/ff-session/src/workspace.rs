//! Workspace model -- `WorkspaceState`, load/save for `.ffwb-workspace` TOML files.
//!
//! A workspace is a named collection of root directories, workspace-scoped
//! configuration overrides, and a per-workspace recent-files list.
//!
//! Addresses: workspace-model Requirement 1 (file format), Requirement 5 (session
//! persistence), Requirement 6 (workspace-scoped recent files).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::SessionError;

// === Data structures =========================================================

/// A workspace-scoped recent-file entry stored in the `.ffwb-workspace` file.
///
/// Distinct from `session_state::RecentFileEntry` which uses URI strings and
/// is stored in `session.toml`. This type uses `PathBuf` and an RFC 3339
/// timestamp string to match the workspace TOML format.
///
/// Addresses: workspace-model Requirement 6.1, 6.2
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceRecentFile {
    /// Absolute path to the file.
    pub path: PathBuf,
    /// RFC 3339 timestamp of when the file was last opened in this workspace.
    pub opened_at: String,
}

impl WorkspaceRecentFile {
    /// Create a new entry with the current UTC time.
    pub fn now(path: PathBuf) -> Self {
        Self {
            path,
            opened_at: Utc::now().to_rfc3339(),
        }
    }
}

/// The in-memory representation of an active workspace.
///
/// Serialised to / deserialised from a `.ffwb-workspace` TOML file.
///
/// Addresses: workspace-model Requirement 1.1, 1.2
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceState {
    /// Human-readable workspace name.
    pub name: String,
    /// Path to the `.ffwb-workspace` file on disk. `None` for an unsaved new workspace.
    pub file_path: Option<PathBuf>,
    /// Ordered list of workspace root directories.
    pub roots: Vec<PathBuf>,
    /// Workspace-scoped configuration overrides (key -> TOML value string).
    pub settings: HashMap<String, String>,
    /// Per-workspace recent-files list (most-recent first).
    pub recent_files: Vec<WorkspaceRecentFile>,
    /// Whether the workspace has unsaved changes.
    pub is_modified: bool,
}

impl WorkspaceState {
    /// Create a new, empty, unsaved workspace with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            file_path: None,
            roots: Vec::new(),
            settings: HashMap::new(),
            recent_files: Vec::new(),
            is_modified: false,
        }
    }

    /// Maximum number of recent files kept in the workspace MRU list.
    ///
    /// Addresses: workspace-model Requirement 6.4 (default depth 50).
    pub const DEFAULT_RECENT_FILES_DEPTH: usize = 50;

    /// Record a file as recently opened, deduplicating and capping the list.
    ///
    /// Addresses: workspace-model Requirement 6.1
    pub fn record_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|e| e.path != path);
        self.recent_files.insert(0, WorkspaceRecentFile::now(path));
        self.recent_files.truncate(Self::DEFAULT_RECENT_FILES_DEPTH);
        self.is_modified = true;
    }
}

// === TOML wire format ========================================================

/// The raw TOML-serialisable form of a workspace file.
///
/// Kept separate from `WorkspaceState` so the public API uses `PathBuf` while
/// TOML serialisation uses plain strings (TOML has no native path type).
#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceToml {
    name: String,
    #[serde(default)]
    roots: Vec<String>,
    #[serde(default)]
    settings: HashMap<String, String>,
    #[serde(default)]
    recent_files: Vec<WorkspaceRecentFileToml>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceRecentFileToml {
    path: String,
    opened_at: String,
}

// === Public API ==============================================================

/// Load a workspace from a `.ffwb-workspace` TOML file.
///
/// Validates that `name` and `roots` are present. Relative root paths are
/// resolved relative to the directory containing the workspace file.
///
/// Addresses: workspace-model Requirement 1.3, 1.4
pub fn load_workspace(path: &Path) -> Result<WorkspaceState, SessionError> {
    let content =
        std::fs::read_to_string(path).map_err(|e| SessionError::WorkspaceFileCorrupt {
            path: path.to_path_buf(),
            reason: format!("cannot read file: {e}"),
        })?;

    let raw: WorkspaceToml =
        toml::from_str(&content).map_err(|e| SessionError::WorkspaceFileCorrupt {
            path: path.to_path_buf(),
            reason: format!("TOML parse error: {e}"),
        })?;

    if raw.name.is_empty() {
        return Err(SessionError::WorkspaceFileCorrupt {
            path: path.to_path_buf(),
            reason: "required field `name` is empty".to_string(),
        });
    }

    let base = path.parent().unwrap_or(Path::new("."));

    let roots = raw
        .roots
        .iter()
        .map(|r| {
            let p = PathBuf::from(r);
            if p.is_absolute() {
                p
            } else {
                base.join(p)
            }
        })
        .collect();

    let recent_files = raw
        .recent_files
        .into_iter()
        .map(|e| WorkspaceRecentFile {
            path: PathBuf::from(e.path),
            opened_at: e.opened_at,
        })
        .collect();

    Ok(WorkspaceState {
        name: raw.name,
        file_path: Some(path.to_path_buf()),
        roots,
        settings: raw.settings,
        recent_files,
        is_modified: false,
    })
}

/// Serialise and write a workspace to a `.ffwb-workspace` TOML file.
///
/// Addresses: workspace-model Requirement 1.1, 2.2
pub fn save_workspace(state: &WorkspaceState, path: &Path) -> Result<(), SessionError> {
    let raw = WorkspaceToml {
        name: state.name.clone(),
        roots: state
            .roots
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect(),
        settings: state.settings.clone(),
        recent_files: state
            .recent_files
            .iter()
            .map(|e| WorkspaceRecentFileToml {
                path: e.path.to_string_lossy().into_owned(),
                opened_at: e.opened_at.clone(),
            })
            .collect(),
    };

    let content =
        toml::to_string_pretty(&raw).map_err(|e| SessionError::WorkspaceFileWriteFailed {
            path: path.to_path_buf(),
            reason: format!("serialisation error: {e}"),
        })?;

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SessionError::WorkspaceFileWriteFailed {
                    path: path.to_path_buf(),
                    reason: format!("cannot create parent directory: {e}"),
                }
            })?;
        }
    }

    let tmp = path.with_extension("ffwb-workspace.tmp");
    std::fs::write(&tmp, content.as_bytes()).map_err(|e| {
        SessionError::WorkspaceFileWriteFailed {
            path: tmp.clone(),
            reason: format!("cannot write temp file: {e}"),
        }
    })?;

    std::fs::rename(&tmp, path).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        SessionError::WorkspaceFileWriteFailed {
            path: path.to_path_buf(),
            reason: format!("cannot rename temp to target: {e}"),
        }
    })?;

    Ok(())
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // -- helpers --

    fn write_workspace_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let p = dir.join(filename);
        std::fs::write(&p, content).unwrap();
        p
    }

    /// Returns a platform-appropriate absolute path string for tests.
    #[cfg(windows)]
    fn abs(suffix: &str) -> String {
        format!("C:{suffix}")
    }
    #[cfg(not(windows))]
    fn abs(suffix: &str) -> String {
        suffix.to_string()
    }

    // -- Task 1.4: round-trip serialisation -----------------------------------

    #[test]
    fn round_trip_minimal_workspace() {
        // Validates: Requirement 1.1, 1.2
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("project.ffwb-workspace");

        let root = PathBuf::from(abs("/home/user/myapp"));
        let mut state = WorkspaceState::new("MyProject");
        state.roots.push(root.clone());

        save_workspace(&state, &path).unwrap();
        let loaded = load_workspace(&path).unwrap();

        assert_eq!(loaded.name, "MyProject");
        assert_eq!(loaded.roots, vec![root]);
        assert!(loaded.settings.is_empty());
        assert!(loaded.recent_files.is_empty());
        assert!(!loaded.is_modified);
        assert_eq!(loaded.file_path, Some(path));
    }

    #[test]
    fn round_trip_with_settings_and_recent_files() {
        // Validates: Requirement 1.1, 6.2
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("ws.ffwb-workspace");

        let root = PathBuf::from(abs("/projects/app"));
        let recent_path = PathBuf::from(abs("/projects/app/src/main.rs"));
        let mut state = WorkspaceState::new("Full");
        state.roots.push(root);
        state
            .settings
            .insert("editor.tab_size".to_string(), "4".to_string());
        state.recent_files.push(WorkspaceRecentFile {
            path: recent_path.clone(),
            opened_at: "2026-01-01T00:00:00+00:00".to_string(),
        });

        save_workspace(&state, &path).unwrap();
        let loaded = load_workspace(&path).unwrap();

        assert_eq!(loaded.settings["editor.tab_size"], "4");
        assert_eq!(loaded.recent_files.len(), 1);
        assert_eq!(loaded.recent_files[0].path, recent_path);
    }

    // -- Task 1.4: missing required field error -------------------------------

    #[test]
    fn load_missing_name_field_returns_error() {
        // Validates: Requirement 1.3
        let tmp = TempDir::new().unwrap();
        let path = write_workspace_file(
            tmp.path(),
            "bad.ffwb-workspace",
            &format!("roots = [\"{}\"]", abs("/home/user")),
        );
        let err = load_workspace(&path).unwrap_err();
        assert!(
            matches!(err, SessionError::WorkspaceFileCorrupt { .. }),
            "expected WorkspaceFileCorrupt, got {err:?}"
        );
    }

    #[test]
    fn load_empty_name_returns_error() {
        // Validates: Requirement 1.3
        let tmp = TempDir::new().unwrap();
        let path = write_workspace_file(
            tmp.path(),
            "empty-name.ffwb-workspace",
            "name = \"\"\nroots = []\n",
        );
        let err = load_workspace(&path).unwrap_err();
        assert!(matches!(err, SessionError::WorkspaceFileCorrupt { .. }));
    }

    #[test]
    fn load_corrupt_toml_returns_error() {
        // Validates: Requirement 1.3
        let tmp = TempDir::new().unwrap();
        let path = write_workspace_file(tmp.path(), "corrupt.ffwb-workspace", "{{not valid toml}}");
        let err = load_workspace(&path).unwrap_err();
        assert!(matches!(err, SessionError::WorkspaceFileCorrupt { .. }));
    }

    // -- Task 1.4: relative path resolution -----------------------------------

    #[test]
    fn relative_root_resolved_relative_to_workspace_file_directory() {
        // Validates: Requirement 1.4
        let tmp = TempDir::new().unwrap();
        let path = write_workspace_file(
            tmp.path(),
            "rel.ffwb-workspace",
            "name = \"Rel\"\nroots = [\"subdir\"]\n",
        );
        let loaded = load_workspace(&path).unwrap();
        let expected = tmp.path().join("subdir");
        assert_eq!(loaded.roots[0], expected);
    }

    #[test]
    fn absolute_root_not_modified() {
        // Validates: Requirement 1.4
        let tmp = TempDir::new().unwrap();
        let abs_root = abs("/absolute/path");
        let path = write_workspace_file(
            tmp.path(),
            "abs.ffwb-workspace",
            &format!("name = \"Abs\"\nroots = [\"{abs_root}\"]"),
        );
        let loaded = load_workspace(&path).unwrap();
        assert_eq!(loaded.roots[0], PathBuf::from(&abs_root));
    }

    // -- WorkspaceState helpers -----------------------------------------------

    #[test]
    fn record_recent_file_deduplicates_and_moves_to_front() {
        // Validates: Requirement 6.1
        let mut ws = WorkspaceState::new("Test");
        let a = PathBuf::from(abs("/a.rs"));
        let b = PathBuf::from(abs("/b.rs"));
        ws.record_recent_file(a.clone());
        ws.record_recent_file(b.clone());
        ws.record_recent_file(a.clone());
        assert_eq!(ws.recent_files[0].path, a);
        assert_eq!(ws.recent_files[1].path, b);
        assert_eq!(ws.recent_files.len(), 2);
    }

    #[test]
    fn record_recent_file_caps_at_default_depth() {
        // Validates: Requirement 6.4
        let mut ws = WorkspaceState::new("Test");
        for i in 0..=WorkspaceState::DEFAULT_RECENT_FILES_DEPTH {
            ws.record_recent_file(PathBuf::from(format!("/file{i}.rs")));
        }
        assert_eq!(
            ws.recent_files.len(),
            WorkspaceState::DEFAULT_RECENT_FILES_DEPTH
        );
    }

    #[test]
    fn record_recent_file_marks_workspace_modified() {
        // Validates: Requirement 6.1
        let mut ws = WorkspaceState::new("Test");
        assert!(!ws.is_modified);
        ws.record_recent_file(PathBuf::from(abs("/x.rs")));
        assert!(ws.is_modified);
    }

    #[test]
    fn new_workspace_is_not_modified_and_has_no_file_path() {
        // Validates: Requirement 1.1
        let ws = WorkspaceState::new("Fresh");
        assert!(!ws.is_modified);
        assert!(ws.file_path.is_none());
        assert!(ws.roots.is_empty());
    }

    // -- DateTime parsing sanity ----------------------------------------------

    #[test]
    fn workspace_recent_file_now_produces_valid_rfc3339() {
        use chrono::DateTime;
        let entry = WorkspaceRecentFile::now(PathBuf::from(abs("/x.rs")));
        DateTime::parse_from_rfc3339(&entry.opened_at).expect("opened_at must be valid RFC 3339");
    }
}
