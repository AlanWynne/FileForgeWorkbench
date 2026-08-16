//! Per-project configuration management.
//!
//! Handles detection, loading, and lifecycle of project-layer configuration
//! files (`.ffworkbench/config.toml` in the project root directory).

use std::path::Path;

use crate::error::ConfigError;
use crate::layer::ConfigLayer;
use crate::loader::LayerData;
use crate::paths::project_config_path;

/// Load the project-layer configuration file if it exists.
///
/// Detects `.ffworkbench/config.toml` in the given project root directory,
/// loads and parses it, and returns a `LayerData` at `ConfigLayer::Project`
/// priority.
///
/// Returns `Ok(Some(LayerData))` if the file exists and was loaded successfully.
/// Returns `Ok(None)` if the file does not exist (project config is optional).
/// Returns `Err(ConfigError)` if the file exists but cannot be read or parsed.
///
/// # Errors
///
/// - `ConfigError::Io` if the file exists but cannot be read (permissions, etc.)
/// - `ConfigError::ParseError` if the file contains invalid TOML syntax
pub fn load_project_config(project_root: &Path) -> Result<Option<LayerData>, ConfigError> {
    let config_path = project_config_path(project_root);

    if !config_path.exists() {
        return Ok(None);
    }

    let values = crate::loader::load_toml_file(&config_path)?;

    Ok(Some(LayerData {
        layer: ConfigLayer::Project,
        source_path: config_path,
        values,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{ConfigTable, ConfigValue};
    use tempfile::TempDir;

    // Validates: Requirement 5.1 — project config loads successfully
    #[test]
    fn load_project_config_returns_layer_data_when_config_exists() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            r#"
[editor]
tab_size = 2
indent_style = "space"
"#,
        )
        .unwrap();

        let result = load_project_config(dir.path());
        assert!(result.is_ok(), "Should succeed when config exists");

        let layer_data = result.unwrap();
        assert!(
            layer_data.is_some(),
            "Should return Some when config exists"
        );

        let layer_data = layer_data.unwrap();
        assert_eq!(layer_data.layer, ConfigLayer::Project);

        if let Some(ConfigValue::Table(editor)) = layer_data.values.get("editor") {
            assert_eq!(editor.get("tab_size"), Some(&ConfigValue::Integer(2)));
            assert_eq!(
                editor.get("indent_style"),
                Some(&ConfigValue::String("space".to_string()))
            );
        } else {
            panic!("editor table should exist in loaded project config");
        }
    }

    // Validates: Requirement 5.2 — returns None when no project config exists
    #[test]
    fn load_project_config_returns_none_when_no_config_file() {
        let dir = TempDir::new().unwrap();
        // No .ffworkbench directory at all

        let result = load_project_config(dir.path());
        assert!(result.is_ok(), "Should succeed even without config");
        assert!(
            result.unwrap().is_none(),
            "Should return None when config file does not exist"
        );
    }

    // Validates: Requirement 5.7 — returns error for invalid TOML in project config
    #[test]
    fn load_project_config_returns_error_for_invalid_toml() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        std::fs::write(
            ffworkbench_dir.join("config.toml"),
            "this is not valid = [toml content\nbroken",
        )
        .unwrap();

        let result = load_project_config(dir.path());
        assert!(result.is_err(), "Should return error for invalid TOML");

        match result.unwrap_err() {
            ConfigError::ParseError { path, details } => {
                assert!(
                    path.ends_with("config.toml"),
                    "ParseError path should reference config.toml, got: {}",
                    path.display()
                );
                assert!(!details.is_empty(), "ParseError should have details");
            }
            other => panic!("Expected ParseError, got: {:?}", other),
        }
    }

    // Validates: Requirement 5.1, 5.2 — LayerData has correct layer and source_path
    #[test]
    fn load_project_config_layer_data_has_correct_layer_and_source() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        let config_file = ffworkbench_dir.join("config.toml");
        std::fs::write(&config_file, "[core]\nkey = \"value\"\n").unwrap();

        let result = load_project_config(dir.path()).unwrap().unwrap();

        assert_eq!(result.layer, ConfigLayer::Project, "Layer must be Project");
        assert_eq!(
            result.source_path, config_file,
            "source_path must point to the actual config file"
        );
    }

    // Validates: Requirement 5.2 — returns None when .ffworkbench dir exists but config.toml is missing
    #[test]
    fn load_project_config_returns_none_when_dir_exists_without_config() {
        let dir = TempDir::new().unwrap();
        let ffworkbench_dir = dir.path().join(".ffworkbench");
        std::fs::create_dir_all(&ffworkbench_dir).unwrap();
        // Directory exists but no config.toml file

        let result = load_project_config(dir.path());
        assert!(result.is_ok());
        assert!(
            result.unwrap().is_none(),
            "Should return None when .ffworkbench dir exists but config.toml does not"
        );
    }
}
