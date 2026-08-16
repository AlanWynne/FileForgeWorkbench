//! TOML file loading.
//!
//! Responsible for reading TOML configuration files from disk, parsing them
//! into `ConfigTable` values, and reporting parse or I/O errors.

use std::path::{Path, PathBuf};

use crate::error::ConfigError;
use crate::layer::ConfigLayer;
use crate::value::{ConfigTable, ConfigValue};

/// Known top-level namespace tables for configuration files.
///
/// These represent the valid organizational categories for settings.
pub const KNOWN_NAMESPACES: &[&str] = &[
    "logging", "editor", "theme", "plugins", "vfs", "commands", "layout", "core", "_session",
];

/// Parsed data from a single configuration layer file.
///
/// Holds the layer identity, the source file path, and all parsed
/// key-value pairs from the TOML file.
#[derive(Debug, Clone)]
pub struct LayerData {
    /// Which configuration layer this data belongs to.
    pub layer: ConfigLayer,
    /// The file path this data was loaded from.
    pub source_path: PathBuf,
    /// The parsed configuration values.
    pub values: ConfigTable,
}

/// Load and parse a TOML configuration file into a `ConfigTable`.
///
/// Reads the file at the given path, parses the contents as TOML, and
/// converts the result into the internal `ConfigTable` representation.
///
/// # Errors
///
/// Returns `ConfigError::Io` if the file cannot be read (missing, permission denied, etc.).
/// Returns `ConfigError::ParseError` if the file contains invalid TOML syntax or if
/// the root value is not a table.
pub fn load_toml_file(path: &Path) -> Result<ConfigTable, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
    parse_toml_content(&content, path)
}

/// Parse TOML content string into a `ConfigTable`.
///
/// This is the internal parsing function separated from I/O for testability.
///
/// # Errors
///
/// Returns `ConfigError::ParseError` if the content is not valid TOML or if
/// the root value is not a table.
pub(crate) fn parse_toml_content(
    content: &str,
    source_path: &Path,
) -> Result<ConfigTable, ConfigError> {
    let toml_value: toml::Value =
        content
            .parse()
            .map_err(|e: toml::de::Error| ConfigError::ParseError {
                path: source_path.to_path_buf(),
                details: e.to_string(),
            })?;

    match toml_value {
        toml::Value::Table(table) => Ok(toml_table_to_config_table(&table)),
        _ => Err(ConfigError::ParseError {
            path: source_path.to_path_buf(),
            details: "root value must be a table".to_string(),
        }),
    }
}

/// Validate that all top-level keys in a `ConfigTable` belong to known namespaces.
///
/// Returns a list of unknown top-level keys. These are not errors but should
/// be logged at DEBUG level per Requirement 9.6.
pub fn validate_namespaces(table: &ConfigTable) -> Vec<String> {
    table
        .keys()
        .filter(|key| !KNOWN_NAMESPACES.contains(&key.as_str()))
        .cloned()
        .collect()
}

/// Convert a `toml::map::Map` into our internal `ConfigTable`.
fn toml_table_to_config_table(table: &toml::map::Map<String, toml::Value>) -> ConfigTable {
    let mut result = ConfigTable::new();
    for (key, value) in table {
        result.insert(key.clone(), toml_value_to_config_value(value));
    }
    result
}

/// Convert a single `toml::Value` into our internal `ConfigValue`.
fn toml_value_to_config_value(value: &toml::Value) -> ConfigValue {
    match value {
        toml::Value::String(s) => ConfigValue::String(s.clone()),
        toml::Value::Integer(i) => ConfigValue::Integer(*i),
        toml::Value::Float(f) => ConfigValue::Float(*f),
        toml::Value::Boolean(b) => ConfigValue::Boolean(*b),
        toml::Value::Array(arr) => {
            ConfigValue::Array(arr.iter().map(toml_value_to_config_value).collect())
        }
        toml::Value::Table(t) => ConfigValue::Table(toml_table_to_config_table(t)),
        toml::Value::Datetime(dt) => ConfigValue::String(dt.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    // Validates: Requirement 1.1 — valid TOML files are parsed successfully
    #[test]
    fn load_toml_file_parses_valid_toml() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("config.toml");
        std::fs::write(
            &file_path,
            r#"
[editor]
tab_size = 4
indent_style = "space"

[logging]
level = "info"
"#,
        )
        .unwrap();

        let result = load_toml_file(&file_path);
        assert!(result.is_ok(), "Valid TOML should parse successfully");

        let table = result.unwrap();
        // Check editor table exists
        assert!(table.contains_key("editor"));
        assert!(table.contains_key("logging"));

        if let Some(ConfigValue::Table(editor)) = table.get("editor") {
            assert_eq!(editor.get("tab_size"), Some(&ConfigValue::Integer(4)));
            assert_eq!(
                editor.get("indent_style"),
                Some(&ConfigValue::String("space".to_string()))
            );
        } else {
            panic!("editor should be a table");
        }
    }

    // Validates: Requirement 1.6 — syntax errors produce ParseError
    #[test]
    fn load_toml_file_returns_parse_error_on_invalid_toml() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("bad.toml");
        std::fs::write(&file_path, "this is not = [valid toml\nfoo bar").unwrap();

        let result = load_toml_file(&file_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::ParseError { path, details } => {
                assert_eq!(path, file_path);
                assert!(!details.is_empty(), "Parse error should have details");
            }
            other => panic!("Expected ParseError, got: {:?}", other),
        }
    }

    // Validates: Requirement 5.7 — I/O errors for missing files
    #[test]
    fn load_toml_file_returns_io_error_for_missing_file() {
        let path = Path::new("/nonexistent/path/config.toml");
        let result = load_toml_file(path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::Io(io_err) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::NotFound);
            }
            other => panic!("Expected Io error, got: {:?}", other),
        }
    }

    // Validates: Requirement 5.7 — I/O errors for unreadable files
    #[cfg(unix)]
    #[test]
    fn load_toml_file_returns_io_error_for_unreadable_file() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("locked.toml");
        std::fs::write(&file_path, "[editor]\ntab_size = 4\n").unwrap();

        // Remove read permissions
        let perms = std::fs::Permissions::from_mode(0o000);
        std::fs::set_permissions(&file_path, perms).unwrap();

        let result = load_toml_file(&file_path);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConfigError::Io(io_err) => {
                assert_eq!(io_err.kind(), std::io::ErrorKind::PermissionDenied);
            }
            other => panic!("Expected Io error, got: {:?}", other),
        }

        // Restore permissions for cleanup
        let perms = std::fs::Permissions::from_mode(0o644);
        std::fs::set_permissions(&file_path, perms).unwrap();
    }

    // Validates: Requirement 1.1 — TOML value types are correctly mapped
    #[test]
    fn load_toml_file_maps_all_value_types() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("types.toml");
        std::fs::write(
            &file_path,
            r#"
string_val = "hello"
int_val = 42
float_val = 3.14
bool_val = true
array_val = [1, 2, 3]

[table_val]
nested_key = "nested"
"#,
        )
        .unwrap();

        let table = load_toml_file(&file_path).unwrap();

        assert_eq!(
            table.get("string_val"),
            Some(&ConfigValue::String("hello".to_string()))
        );
        assert_eq!(table.get("int_val"), Some(&ConfigValue::Integer(42)));
        assert_eq!(table.get("float_val"), Some(&ConfigValue::Float(3.14)));
        assert_eq!(table.get("bool_val"), Some(&ConfigValue::Boolean(true)));
        assert_eq!(
            table.get("array_val"),
            Some(&ConfigValue::Array(vec![
                ConfigValue::Integer(1),
                ConfigValue::Integer(2),
                ConfigValue::Integer(3),
            ]))
        );

        if let Some(ConfigValue::Table(nested)) = table.get("table_val") {
            assert_eq!(
                nested.get("nested_key"),
                Some(&ConfigValue::String("nested".to_string()))
            );
        } else {
            panic!("table_val should be a Table");
        }
    }

    // Validates: Requirement 1.3 — settings organized into namespace tables
    #[test]
    fn validate_namespaces_identifies_known_namespaces() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("namespaces.toml");
        std::fs::write(
            &file_path,
            r#"
[logging]
level = "info"

[editor]
tab_size = 4

[theme]
active = "dark"

[plugins]
enabled = true

[vfs]
provider = "local"
"#,
        )
        .unwrap();

        let table = load_toml_file(&file_path).unwrap();
        let unknown = validate_namespaces(&table);
        assert!(
            unknown.is_empty(),
            "All standard namespaces should be recognized, got unknown: {:?}",
            unknown
        );
    }

    // Validates: Requirement 9.6 — unknown keys are identified (not errors)
    #[test]
    fn validate_namespaces_reports_unknown_top_level_keys() {
        let mut table = ConfigTable::new();
        table.insert(
            "logging".to_string(),
            ConfigValue::Table(ConfigTable::new()),
        );
        table.insert(
            "unknown_section".to_string(),
            ConfigValue::Table(ConfigTable::new()),
        );
        table.insert(
            "another_unknown".to_string(),
            ConfigValue::String("val".to_string()),
        );

        let unknown = validate_namespaces(&table);
        assert_eq!(unknown.len(), 2);
        assert!(unknown.contains(&"unknown_section".to_string()));
        assert!(unknown.contains(&"another_unknown".to_string()));
    }

    // Validates: Requirement 1.1 — empty TOML file produces empty table
    #[test]
    fn load_toml_file_handles_empty_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("empty.toml");
        std::fs::write(&file_path, "").unwrap();

        let table = load_toml_file(&file_path).unwrap();
        assert!(
            table.is_empty(),
            "Empty TOML file should produce empty table"
        );
    }

    // Validates: Requirement 1.3 — deeply nested tables are parsed correctly
    #[test]
    fn load_toml_file_handles_nested_tables() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("nested.toml");
        std::fs::write(
            &file_path,
            r#"
[plugins.sql-viewer]
max_rows = 1000
highlight = true

[plugins.sql-viewer.connection]
timeout_ms = 5000
"#,
        )
        .unwrap();

        let table = load_toml_file(&file_path).unwrap();
        if let Some(ConfigValue::Table(plugins)) = table.get("plugins") {
            if let Some(ConfigValue::Table(sql_viewer)) = plugins.get("sql-viewer") {
                assert_eq!(
                    sql_viewer.get("max_rows"),
                    Some(&ConfigValue::Integer(1000))
                );
                assert_eq!(
                    sql_viewer.get("highlight"),
                    Some(&ConfigValue::Boolean(true))
                );

                if let Some(ConfigValue::Table(conn)) = sql_viewer.get("connection") {
                    assert_eq!(conn.get("timeout_ms"), Some(&ConfigValue::Integer(5000)));
                } else {
                    panic!("connection should be a nested table");
                }
            } else {
                panic!("sql-viewer should be a table under plugins");
            }
        } else {
            panic!("plugins should be a table");
        }
    }

    // Validates: Requirement 1.6 — various syntax errors are caught
    #[test]
    fn parse_toml_content_rejects_various_syntax_errors() {
        let test_cases = vec![
            ("= value_without_key", "missing key"),
            ("[unclosed", "unclosed section"),
            ("key = ", "missing value"),
        ];

        for (content, description) in test_cases {
            let result = parse_toml_content(content, Path::new("test.toml"));
            assert!(
                result.is_err(),
                "Should reject TOML with {}: {:?}",
                description,
                content
            );
        }
    }

    // Validates: Task 5.4 — LayerData struct holds identity, source, and values
    #[test]
    fn layer_data_struct_holds_all_fields() {
        let mut values = ConfigTable::new();
        values.insert("key".to_string(), ConfigValue::Integer(42));

        let layer_data = LayerData {
            layer: ConfigLayer::User,
            source_path: PathBuf::from("/home/user/.config/ffworkbench/config.toml"),
            values,
        };

        assert_eq!(layer_data.layer, ConfigLayer::User);
        assert_eq!(
            layer_data.source_path,
            PathBuf::from("/home/user/.config/ffworkbench/config.toml")
        );
        assert_eq!(
            layer_data.values.get("key"),
            Some(&ConfigValue::Integer(42))
        );
    }

    // Validates: Requirement 1.1 — TOML datetime values are converted to strings
    #[test]
    fn load_toml_file_converts_datetime_to_string() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("datetime.toml");
        std::fs::write(&file_path, "created = 2024-01-15T10:30:00Z\n").unwrap();

        let table = load_toml_file(&file_path).unwrap();
        if let Some(ConfigValue::String(s)) = table.get("created") {
            assert!(
                s.contains("2024"),
                "Datetime should be converted to string containing year"
            );
        } else {
            panic!("Datetime should be converted to ConfigValue::String");
        }
    }

    // Validates: Requirement 1.6 — ParseError includes file path
    #[test]
    fn parse_error_includes_source_path() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("syntax_error.toml");
        std::fs::write(&file_path, "[invalid\nno closing bracket").unwrap();

        let result = load_toml_file(&file_path);
        match result {
            Err(ConfigError::ParseError { path, .. }) => {
                assert_eq!(path, file_path);
            }
            other => panic!("Expected ParseError with path, got: {:?}", other),
        }
    }

    // Validates: Requirement 1.3 — mixed arrays are handled
    #[test]
    fn load_toml_file_handles_homogeneous_arrays() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("arrays.toml");
        std::fs::write(
            &file_path,
            r#"
strings = ["a", "b", "c"]
numbers = [1, 2, 3]
booleans = [true, false, true]
"#,
        )
        .unwrap();

        let table = load_toml_file(&file_path).unwrap();
        assert_eq!(
            table.get("strings"),
            Some(&ConfigValue::Array(vec![
                ConfigValue::String("a".to_string()),
                ConfigValue::String("b".to_string()),
                ConfigValue::String("c".to_string()),
            ]))
        );
    }
}
