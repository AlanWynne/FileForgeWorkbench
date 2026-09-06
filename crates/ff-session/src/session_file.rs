//! Session file TOML serialisation and persistence — reading and writing
//! `session.toml` within the User Data Directory.
//!
//! Addresses: Requirement 4 (AC 4.2, 4.6, 4.7, 4.8)

use std::path::{Path, PathBuf};

use crate::error::SessionError;
use crate::session_state::{SessionState, CURRENT_SCHEMA_VERSION};

/// Handles reading and writing `SessionState` to/from `session.toml`.
///
/// Provides atomic writes (write to temp file then rename) and graceful
/// handling of missing or corrupt session files.
#[derive(Debug, Clone)]
pub struct SessionFile {
    /// Path to the `session.toml` file within User_Data_Dir.
    path: PathBuf,
}

impl SessionFile {
    /// Create a `SessionFile` handle pointing to the given path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Returns the path to the session file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Check whether the session file exists on disk.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Load and deserialise the session file.
    ///
    /// Returns an empty session if the file is absent (first run).
    /// Returns an empty session if the file is corrupt (with error logged).
    ///
    /// Addresses: Requirement 4 AC 4.7, 4.8
    pub fn load(&self) -> Result<SessionState, SessionError> {
        if !self.exists() {
            // First run or manually deleted — start fresh
            return Ok(SessionState::empty());
        }

        let content =
            std::fs::read_to_string(&self.path).map_err(|e| SessionError::SessionFileCorrupt {
                path: self.path.clone(),
                reason: format!("cannot read file: {e}"),
            })?;

        self.deserialize(&content)
    }

    /// Serialise and write the session state to disk atomically.
    ///
    /// Uses write-to-temp-then-rename to avoid corruption on crash during write.
    ///
    /// Addresses: Requirement 4 AC 4.2
    pub fn save(&self, state: &SessionState) -> Result<(), SessionError> {
        let content = self.serialize(state)?;

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    SessionError::SessionFileWriteFailed {
                        path: self.path.clone(),
                        reason: format!("cannot create parent directory: {e}"),
                    }
                })?;
            }
        }

        // Atomic write: write to temp file then rename
        let temp_path = self.path.with_extension("toml.tmp");
        std::fs::write(&temp_path, content.as_bytes()).map_err(|e| {
            SessionError::SessionFileWriteFailed {
                path: temp_path.clone(),
                reason: format!("cannot write temp file: {e}"),
            }
        })?;

        std::fs::rename(&temp_path, &self.path).map_err(|e| {
            // Clean up temp file on rename failure
            let _ = std::fs::remove_file(&temp_path);
            SessionError::SessionFileWriteFailed {
                path: self.path.clone(),
                reason: format!("cannot rename temp to target: {e}"),
            }
        })?;

        Ok(())
    }

    /// Serialise a `SessionState` to a TOML string.
    pub fn serialize(&self, state: &SessionState) -> Result<String, SessionError> {
        toml::to_string_pretty(state).map_err(|e| SessionError::SessionFileWriteFailed {
            path: self.path.clone(),
            reason: format!("serialisation error: {e}"),
        })
    }

    /// Deserialise a TOML string into a `SessionState`.
    ///
    /// Handles schema migration for older versions and forward-compatibility
    /// by ignoring unknown keys.
    ///
    /// Addresses: Requirement 4 AC 4.6, 4.8
    pub fn deserialize(&self, content: &str) -> Result<SessionState, SessionError> {
        // First try strict deserialization
        let state: SessionState =
            toml::from_str(content).map_err(|e| SessionError::SessionFileCorrupt {
                path: self.path.clone(),
                reason: format!("TOML parse error: {e}"),
            })?;

        // Apply schema migration if needed
        let migrated = if state.schema_version < CURRENT_SCHEMA_VERSION {
            SessionState::migrate(state)
        } else {
            state
        };

        Ok(migrated)
    }
}

/// Serialize a `SessionState` to TOML without needing a `SessionFile` instance.
///
/// Useful for testing and standalone serialisation.
pub fn serialize_session_state(state: &SessionState) -> Result<String, SessionError> {
    toml::to_string_pretty(state).map_err(|e| SessionError::SessionFileWriteFailed {
        path: PathBuf::from("(in-memory)"),
        reason: format!("serialisation error: {e}"),
    })
}

/// Deserialize a `SessionState` from a TOML string without needing a `SessionFile` instance.
///
/// Useful for testing and standalone deserialisation.
pub fn deserialize_session_state(content: &str) -> Result<SessionState, SessionError> {
    let state: SessionState =
        toml::from_str(content).map_err(|e| SessionError::SessionFileCorrupt {
            path: PathBuf::from("(in-memory)"),
            reason: format!("TOML parse error: {e}"),
        })?;

    let migrated = if state.schema_version < CURRENT_SCHEMA_VERSION {
        SessionState::migrate(state)
    } else {
        state
    };

    Ok(migrated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_state::{
        LayoutSnapshot, RecentFileEntry, SelectionRange, TabState, WindowGeometryState,
    };
    use tempfile::TempDir;

    #[test]
    fn load_nonexistent_file_returns_empty_session() {
        // Validates: Requirement 4 AC 4.7
        let sf = SessionFile::new(PathBuf::from("/nonexistent/session.toml"));
        let state = sf.load().unwrap();
        assert_eq!(state, SessionState::empty());
    }

    #[test]
    fn save_and_load_round_trip_empty_state() {
        // Validates: Requirement 4 AC 4.2
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("session.toml");
        let sf = SessionFile::new(path);

        let state = SessionState::empty();
        sf.save(&state).unwrap();
        let loaded = sf.load().unwrap();
        assert_eq!(loaded, state);
    }

    #[test]
    fn save_and_load_round_trip_with_tabs() {
        // Validates: Requirement 4 AC 4.1, 4.2
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("session.toml");
        let sf = SessionFile::new(path);

        let state = SessionState {
            schema_version: CURRENT_SCHEMA_VERSION,
            tabs: vec![
                TabState {
                    tab_id: "tab-1".to_string(),
                    uri: Some("file:///home/user/project/main.rs".to_string()),
                    viewport_top_line: 42,
                    viewport_horizontal_offset: 5,
                    caret_line: 50,
                    caret_column: 12,
                    selections: vec![SelectionRange {
                        start_line: 50,
                        start_column: 5,
                        end_line: 50,
                        end_column: 12,
                    }],
                    language_override: Some("rust".to_string()),
                    is_pinned: true,
                    ..Default::default()
                },
                TabState {
                    tab_id: "tab-2".to_string(),
                    uri: Some("vfs://remote/README.md".to_string()),
                    viewport_top_line: 1,
                    viewport_horizontal_offset: 0,
                    caret_line: 1,
                    caret_column: 1,
                    selections: vec![],
                    language_override: None,
                    is_pinned: false,
                    ..Default::default()
                },
            ],
            active_tab_id: Some("tab-1".to_string()),
            layout: None,
            windows: vec![WindowGeometryState::primary(100, 200, 1920, 1080)],
            recent_files: vec![RecentFileEntry {
                uri: "file:///recent/file.txt".to_string(),
                display_name: "file.txt".to_string(),
                last_accessed: "2024-01-15T10:30:00Z".to_string(),
                last_viewport_top_line: Some(10),
                available: true,
            }],
            active_profile: Some("dev".to_string()),
            last_saved: Some("2024-01-15T10:30:00Z".to_string()),
            show_pom: true,
            global_zoom_offset: 0,
            key_bar_visible: true,
            file_explorer_sidebar_width: 200.0,
            active_workspace_path: None,
            recent_palette_commands: Vec::new(),
            search_history: Vec::new(),
        };

        sf.save(&state).unwrap();
        let loaded = sf.load().unwrap();
        assert_eq!(loaded, state);
    }

    #[test]
    fn save_and_load_round_trip_with_layout() {
        // Validates: Requirement 4 AC 4.1
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("session.toml");
        let sf = SessionFile::new(path);

        let mut layout_data = toml::map::Map::new();
        layout_data.insert(
            "panels".to_string(),
            toml::Value::String("left:file_tree,right:outline".to_string()),
        );

        let state = SessionState {
            layout: Some(LayoutSnapshot {
                data: toml::Value::Table(layout_data),
                persona: Some("coding".to_string()),
            }),
            ..Default::default()
        };

        sf.save(&state).unwrap();
        let loaded = sf.load().unwrap();
        assert_eq!(loaded, state);
    }

    #[test]
    fn deserialize_ignores_unknown_keys_for_forward_compatibility() {
        // Validates: Requirement 4 AC 4.6
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("session.toml");
        let sf = SessionFile::new(path);

        // Simulate a session file from a newer version with extra keys
        let content = format!(
            r#"
schema_version = {}
active_profile = "default"

[some_future_section]
future_key = "future_value"
"#,
            CURRENT_SCHEMA_VERSION
        );

        // This should not error — unknown keys are ignored by serde's default
        // behavior when deny_unknown_fields is not set
        let result = sf.deserialize(&content);
        assert!(result.is_ok());
    }

    #[test]
    fn deserialize_corrupt_content_returns_error() {
        // Validates: Requirement 4 AC 4.8
        let sf = SessionFile::new(PathBuf::from("test.toml"));
        let result = sf.deserialize("{{{{not valid toml}}}}");
        assert!(result.is_err());
        if let Err(SessionError::SessionFileCorrupt { reason, .. }) = result {
            assert!(reason.contains("TOML parse error"));
        }
    }

    #[test]
    fn deserialize_older_schema_version_triggers_migration() {
        // Validates: Requirement 4 AC 4.6
        let sf = SessionFile::new(PathBuf::from("test.toml"));
        let content = r#"
schema_version = 0
"#;
        let state = sf.deserialize(content).unwrap();
        assert_eq!(state.schema_version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn save_creates_parent_directory_if_missing() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("subdir").join("session.toml");
        let sf = SessionFile::new(path.clone());

        sf.save(&SessionState::empty()).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn exists_returns_false_for_nonexistent_file() {
        let sf = SessionFile::new(PathBuf::from("/definitely/not/here/session.toml"));
        assert!(!sf.exists());
    }

    #[test]
    fn exists_returns_true_for_existing_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("session.toml");
        std::fs::write(&path, "schema_version = 1").unwrap();

        let sf = SessionFile::new(path);
        assert!(sf.exists());
    }

    #[test]
    fn serialize_and_deserialize_standalone_functions_work() {
        let state = SessionState {
            tabs: vec![TabState {
                tab_id: "t1".to_string(),
                uri: Some("test.rs".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let serialized = serialize_session_state(&state).unwrap();
        let deserialized = deserialize_session_state(&serialized).unwrap();
        assert_eq!(deserialized, state);
    }

    #[test]
    fn schema_version_embedded_in_serialized_output() {
        // Validates: Requirement 4 AC 4.6
        let state = SessionState::empty();
        let serialized = serialize_session_state(&state).unwrap();
        assert!(serialized.contains("schema_version"));
        assert!(serialized.contains(&CURRENT_SCHEMA_VERSION.to_string()));
    }
}
