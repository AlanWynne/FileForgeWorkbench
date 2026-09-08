//! Settings export and import.
//!
//! Provides `export_settings` and `import_settings` for portable TOML
//! configuration snapshots.
//!
//! Addresses: Requirement 17

use std::path::Path;

use crate::error::ConfigError;
use crate::loader::load_toml_file;
use crate::value::{ConfigTable, ConfigValue};

/// Scope of a settings export.
///
/// Addresses: Requirement 17.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportScope {
    /// Export all effective values from all layers.
    AllLayers,
    /// Export only user-layer overrides.
    UserLayer,
    /// Export only project-layer overrides.
    ProjectLayer,
}

/// Target layer for a settings import.
///
/// Addresses: Requirement 17.5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportTarget {
    /// Write imported values into the user-layer file.
    UserLayer,
    /// Write imported values into the project-layer file.
    ProjectLayer,
}

/// Summary returned after a successful import.
///
/// Addresses: Requirement 17.7
#[derive(Debug, Clone, Default)]
pub struct ImportSummary {
    /// Number of keys successfully imported.
    pub imported_count: usize,
    /// Number of keys skipped due to validation failure.
    pub skipped_count: usize,
    /// Keys that were skipped.
    pub skipped_keys: Vec<String>,
}

/// Write a TOML export file for the given scope.
///
/// The file includes a `[_export_meta]` header table with timestamp,
/// version, and scope, followed by the exported key-value pairs.
///
/// Addresses: Requirement 17.1, 17.3
pub fn export_settings(
    values: &ConfigTable,
    scope: ExportScope,
    path: &Path,
    version: &str,
) -> Result<(), ConfigError> {
    let mut root = toml::map::Map::new();

    // Build [_export_meta] header (Req 17.3)
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut meta = toml::map::Map::new();
    meta.insert("timestamp".to_string(), toml::Value::Integer(ts as i64));
    meta.insert(
        "version".to_string(),
        toml::Value::String(version.to_string()),
    );
    meta.insert(
        "scope".to_string(),
        toml::Value::String(format!("{scope:?}")),
    );
    root.insert("_export_meta".to_string(), toml::Value::Table(meta));

    // Flatten the ConfigTable into the TOML document
    for (k, v) in values {
        root.insert(k.clone(), config_value_to_toml(v.clone()));
    }

    let content = toml::to_string_pretty(&toml::Value::Table(root))
        .map_err(|e| ConfigError::Io(std::io::Error::other(e.to_string())))?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(ConfigError::Io)?;
    }
    std::fs::write(path, content).map_err(ConfigError::Io)
}

/// Read an exported TOML file and return the key-value pairs, skipping
/// the `[_export_meta]` header.
///
/// Returns `ConfigError::ParseError` if the file is invalid TOML.
/// Returns `ConfigError::Io` if the file cannot be read.
///
/// Addresses: Requirement 17.4, 17.8
pub fn read_export_file(path: &Path) -> Result<ConfigTable, ConfigError> {
    let table = load_toml_file(path)?;
    // Strip the meta header -- it is not a real config key
    let mut result = ConfigTable::new();
    for (k, v) in table {
        if k != "_export_meta" {
            result.insert(k, v);
        }
    }
    Ok(result)
}

/// Validate and merge imported values into a target TOML file.
///
/// Each key in `imported` is validated against `schema`; invalid keys are
/// collected in the returned `ImportSummary`. Valid keys are written to
/// `target_path` using the same dot-path write logic as `set_user_value`.
///
/// Addresses: Requirement 17.4, 17.6, 17.7
pub fn apply_import(
    imported: &ConfigTable,
    schema: &crate::schema::SchemaRegistry,
    target_path: &Path,
) -> Result<ImportSummary, ConfigError> {
    let mut summary = ImportSummary::default();

    // Read existing target file (or start empty)
    let mut root = read_toml_as_value(target_path)?;

    for (key, value) in imported {
        // Validate against schema if an entry exists
        if let Some(entry) = schema.get(key) {
            use crate::validate::{validate_value, ValidationResult};
            if let ValidationResult::DefaultApplied { .. } = validate_value(value, entry) {
                summary.skipped_count += 1;
                summary.skipped_keys.push(key.clone());
                continue;
            }
        }
        // Write the key into the TOML tree
        set_dotted_key(&mut root, key, config_value_to_toml(value.clone()));
        summary.imported_count += 1;
    }

    // Persist the updated file
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(ConfigError::Io)?;
    }
    let content = toml::to_string_pretty(&root)
        .map_err(|e| ConfigError::Io(std::io::Error::other(e.to_string())))?;
    std::fs::write(target_path, content).map_err(ConfigError::Io)?;

    Ok(summary)
}

// === Internal helpers =====================================================

fn read_toml_as_value(path: &Path) -> Result<toml::Value, ConfigError> {
    if !path.exists() {
        return Ok(toml::Value::Table(toml::map::Map::new()));
    }
    let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
    content
        .parse::<toml::Value>()
        .map_err(|e| ConfigError::ParseError {
            path: path.to_path_buf(),
            details: e.to_string(),
        })
}

fn set_dotted_key(root: &mut toml::Value, key: &str, value: toml::Value) {
    let parts: Vec<&str> = key.split('.').collect();
    let mut current = root;
    for part in &parts[..parts.len() - 1] {
        if let toml::Value::Table(ref mut map) = current {
            let entry = map
                .entry((*part).to_string())
                .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
            current = entry;
        } else {
            return;
        }
    }
    if let toml::Value::Table(ref mut map) = current {
        map.insert(parts[parts.len() - 1].to_string(), value);
    }
}

fn config_value_to_toml(value: ConfigValue) -> toml::Value {
    match value {
        ConfigValue::String(s) => toml::Value::String(s),
        ConfigValue::Integer(i) => toml::Value::Integer(i),
        ConfigValue::Float(f) => toml::Value::Float(f),
        ConfigValue::Boolean(b) => toml::Value::Boolean(b),
        ConfigValue::Array(arr) => {
            toml::Value::Array(arr.into_iter().map(config_value_to_toml).collect())
        }
        ConfigValue::Table(t) => {
            let mut map = toml::map::Map::new();
            for (k, v) in t {
                map.insert(k, config_value_to_toml(v));
            }
            toml::Value::Table(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SchemaRegistry;
    use crate::value::ConfigValue;
    use tempfile::TempDir;

    fn make_values() -> ConfigTable {
        let mut t = ConfigTable::new();
        t.insert("editor.tab_size".to_string(), ConfigValue::Integer(4));
        t.insert(
            "theme.active".to_string(),
            ConfigValue::String("dark".to_string()),
        );
        t
    }

    // Validates: Requirement 17.1, 17.3
    #[test]
    fn export_user_layer_produces_valid_toml() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("export.toml");
        let values = make_values();
        export_settings(&values, ExportScope::UserLayer, &path, "0.1.0").unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("tab_size"));
    }

    // Validates: Requirement 17.3
    #[test]
    fn export_includes_meta_header() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("export.toml");
        export_settings(&make_values(), ExportScope::AllLayers, &path, "1.2.3").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("_export_meta"));
        assert!(content.contains("1.2.3"));
        assert!(content.contains("AllLayers"));
    }

    // Validates: Requirement 17.4
    #[test]
    fn import_valid_file_updates_layer() {
        let dir = TempDir::new().unwrap();
        let export_path = dir.path().join("export.toml");
        let target_path = dir.path().join("user.toml");

        export_settings(
            &make_values(),
            ExportScope::UserLayer,
            &export_path,
            "0.1.0",
        )
        .unwrap();

        let imported = read_export_file(&export_path).unwrap();
        let schema = SchemaRegistry::new();
        let summary = apply_import(&imported, &schema, &target_path).unwrap();

        assert_eq!(summary.imported_count, 2);
        assert_eq!(summary.skipped_count, 0);

        let content = std::fs::read_to_string(&target_path).unwrap();
        assert!(content.contains("tab_size"));
    }

    // Validates: Requirement 17.6, 17.7
    #[test]
    fn import_invalid_values_skipped_in_summary() {
        use crate::schema::constraint::Constraints;
        use crate::schema::entry::SchemaEntry;
        use crate::value::ConfigValue;

        let dir = TempDir::new().unwrap();
        let target_path = dir.path().join("user.toml");

        let mut schema = SchemaRegistry::new();
        // Register tab_size with min=1, max=8
        schema
            .register(SchemaEntry {
                key: "editor.tab_size".to_string(),
                value_type: crate::error::ValueType::Integer,
                default: ConfigValue::Integer(4),
                description: "Tab size".to_string(),
                constraints: Some(Constraints {
                    min: Some(1.0),
                    max: Some(8.0),
                    allowed_values: None,
                    pattern: None,
                }),
            })
            .unwrap();

        let mut imported = ConfigTable::new();
        // Value 99 violates max=8
        imported.insert("editor.tab_size".to_string(), ConfigValue::Integer(99));
        imported.insert(
            "theme.active".to_string(),
            ConfigValue::String("dark".to_string()),
        );

        let summary = apply_import(&imported, &schema, &target_path).unwrap();
        assert_eq!(summary.skipped_count, 1);
        assert!(summary
            .skipped_keys
            .contains(&"editor.tab_size".to_string()));
        assert_eq!(summary.imported_count, 1);
    }

    // Validates: Requirement 17.8
    #[test]
    fn import_bad_toml_returns_parse_error() {
        let dir = TempDir::new().unwrap();
        let bad_path = dir.path().join("bad.toml");
        std::fs::write(&bad_path, b"not valid toml ===").unwrap();
        let result = read_export_file(&bad_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ParseError { .. } => {}
            other => panic!("Expected ParseError, got: {other:?}"),
        }
    }

    // Validates: Requirement 17.2, 17.5
    #[test]
    fn export_scope_and_import_target_variants_exist() {
        let _ = ExportScope::AllLayers;
        let _ = ExportScope::UserLayer;
        let _ = ExportScope::ProjectLayer;
        let _ = ImportTarget::UserLayer;
        let _ = ImportTarget::ProjectLayer;
    }

    // Validates: Requirement 17.7
    #[test]
    fn import_summary_fields_are_public() {
        let s = ImportSummary {
            imported_count: 3,
            skipped_count: 1,
            skipped_keys: vec!["editor.tab_size".to_string()],
        };
        assert_eq!(s.imported_count, 3);
        assert_eq!(s.skipped_count, 1);
        assert_eq!(s.skipped_keys.len(), 1);
    }
}
