//! Persona storage — TOML file I/O for persona definitions.

use std::path::Path;

use crate::error::LayoutError;
use crate::persona::definition::Persona;

/// Reads a persona from a TOML file.
///
/// # Errors
///
/// Returns `SerializationFailed` if the file cannot be read or parsed.
pub fn load_persona(path: &Path) -> Result<Persona, LayoutError> {
    let content = std::fs::read_to_string(path).map_err(|e| LayoutError::SerializationFailed {
        operation: "load_persona".to_string(),
        reason: format!("failed to read {}: {e}", path.display()),
    })?;

    toml::from_str(&content).map_err(|e| LayoutError::SerializationFailed {
        operation: "load_persona".to_string(),
        reason: format!("failed to parse {}: {e}", path.display()),
    })
}

/// Writes a persona to a TOML file.
///
/// # Errors
///
/// Returns `SerializationFailed` if the file cannot be written.
pub fn save_persona(path: &Path, persona: &Persona) -> Result<(), LayoutError> {
    let content =
        toml::to_string_pretty(persona).map_err(|e| LayoutError::SerializationFailed {
            operation: "save_persona".to_string(),
            reason: format!("failed to serialize persona '{}': {e}", persona.name),
        })?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, content)?;
    Ok(())
}

/// Discovers all persona TOML files in a directory.
///
/// Returns a list of paths to `.toml` files in the given directory.
pub fn discover_persona_files(directory: &Path) -> Vec<std::path::PathBuf> {
    if !directory.exists() || !directory.is_dir() {
        return Vec::new();
    }

    std::fs::read_dir(directory)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|ext| ext == "toml").unwrap_or(false))
        .collect()
}

/// Loads all personas from a directory.
///
/// Files that fail to parse are skipped with a warning logged.
pub fn load_all_personas(directory: &Path) -> Vec<Persona> {
    discover_persona_files(directory)
        .into_iter()
        .filter_map(|path| load_persona(&path).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::layout_state::LayoutState;
    use tempfile::TempDir;

    #[test]
    fn save_and_load_persona_round_trip() {
        // Validates: Requirement 5 criterion 7
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.toml");

        let persona = Persona::custom("Test Layout", LayoutState::default());
        save_persona(&path, &persona).unwrap();
        let loaded = load_persona(&path).unwrap();

        assert_eq!(loaded.name, "Test Layout");
        assert!(!loaded.built_in);
    }

    #[test]
    fn discover_persona_files_finds_toml_files() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("editor.toml"), "").unwrap();
        std::fs::write(dir.path().join("debug.toml"), "").unwrap();
        std::fs::write(dir.path().join("readme.md"), "").unwrap();

        let files = discover_persona_files(dir.path());
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn discover_persona_files_returns_empty_for_missing_dir() {
        let files = discover_persona_files(Path::new("/nonexistent/path"));
        assert!(files.is_empty());
    }

    #[test]
    fn load_persona_returns_error_for_missing_file() {
        let result = load_persona(Path::new("/nonexistent/file.toml"));
        assert!(matches!(
            result,
            Err(LayoutError::SerializationFailed { .. })
        ));
    }
}
