//! TOML serialization — serialize/deserialize LayoutState with graceful degradation.

use std::path::Path;

use crate::error::LayoutError;
use crate::state::layout_state::LayoutState;
use crate::SCHEMA_VERSION;

/// Serializes a LayoutState to TOML format.
pub fn serialize_layout_state(state: &LayoutState) -> Result<String, LayoutError> {
    toml::to_string_pretty(state).map_err(|e| LayoutError::SerializationFailed {
        operation: "serialize".to_string(),
        reason: e.to_string(),
    })
}

/// Deserializes a LayoutState from TOML format.
///
/// Returns an error if the TOML is invalid or schema version is incompatible.
pub fn deserialize_layout_state(content: &str) -> Result<LayoutState, LayoutError> {
    let state: LayoutState =
        toml::from_str(content).map_err(|e| LayoutError::SerializationFailed {
            operation: "deserialize".to_string(),
            reason: e.to_string(),
        })?;

    // Check schema version
    if state.schema_version > SCHEMA_VERSION {
        return Err(LayoutError::SerializationFailed {
            operation: "deserialize".to_string(),
            reason: format!(
                "schema version {} is newer than supported version {}",
                state.schema_version, SCHEMA_VERSION
            ),
        });
    }

    Ok(state)
}

/// Saves a LayoutState to a TOML file.
///
/// # Errors
///
/// Returns an error if serialization or file I/O fails.
pub fn save_to_file(state: &LayoutState, path: &Path) -> Result<(), LayoutError> {
    let content = serialize_layout_state(state)?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, content)?;
    Ok(())
}

/// Loads a LayoutState from a TOML file.
///
/// # Errors
///
/// Returns an error if the file cannot be read, parsed, or has an incompatible schema.
pub fn load_from_file(path: &Path) -> Result<LayoutState, LayoutError> {
    let content = std::fs::read_to_string(path).map_err(|e| LayoutError::SerializationFailed {
        operation: "load".to_string(),
        reason: format!("failed to read {}: {e}", path.display()),
    })?;

    deserialize_layout_state(&content)
}

/// Attempts to load a LayoutState from file, returning the default on failure.
///
/// Logs a warning (via the returned reason) when falling back to default.
pub fn load_or_default(path: &Path) -> (LayoutState, Option<String>) {
    match load_from_file(path) {
        Ok(state) => (state, None),
        Err(e) => (LayoutState::default(), Some(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn serialize_deserialize_round_trip() {
        // Validates: Requirement 6 criteria 1, 2, 4
        let state = LayoutState::default();
        let serialized = serialize_layout_state(&state).unwrap();
        let deserialized = deserialize_layout_state(&serialized).unwrap();
        assert_eq!(state, deserialized);
    }

    #[test]
    fn save_and_load_file_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("layout_state.toml");

        let state = LayoutState::default();
        save_to_file(&state, &path).unwrap();
        let loaded = load_from_file(&path).unwrap();
        assert_eq!(state, loaded);
    }

    #[test]
    fn deserialize_invalid_toml_returns_error() {
        // Validates: Requirement 6 criterion 3
        let result = deserialize_layout_state("not valid toml {{{");
        assert!(matches!(
            result,
            Err(LayoutError::SerializationFailed { .. })
        ));
    }

    #[test]
    fn load_or_default_falls_back_on_missing_file() {
        // Validates: Requirement 6 criterion 3
        let (state, reason) = load_or_default(Path::new("/nonexistent/path.toml"));
        assert_eq!(state, LayoutState::default());
        assert!(reason.is_some());
    }

    #[test]
    fn schema_version_included_in_serialization() {
        // Validates: Requirement 6 criterion 11
        let state = LayoutState::default();
        let serialized = serialize_layout_state(&state).unwrap();
        assert!(serialized.contains("schema_version"));
    }
}
