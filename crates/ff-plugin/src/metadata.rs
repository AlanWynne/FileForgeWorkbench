//! Plugin metadata and dependency declarations.
//!
//! Defines `PluginMetadata` and `PluginDependency` structs, and provides
//! TOML manifest parsing from `plugin.toml` files.

use crate::version::{Version, VersionReq};

/// Metadata describing a plugin: identity, versioning, and dependencies.
///
/// This information is declared in the plugin's `plugin.toml` manifest
/// and is accessible at runtime via `FileForgePlugin::metadata()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMetadata {
    /// Unique plugin name (kebab-case identifier).
    pub name: String,
    /// Plugin version (semantic versioning).
    pub version: Version,
    /// Author or organization name.
    pub author: String,
    /// Human-readable description.
    pub description: String,
    /// Dependencies on other plugins.
    pub dependencies: Vec<PluginDependency>,
    /// Minimum plugin API version this plugin requires.
    pub required_api_version: Version,
}

/// A dependency declaration within a plugin's metadata.
///
/// Declares that this plugin requires another plugin to be loaded and active
/// before it can be initialized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginDependency {
    /// Name of the required plugin.
    pub name: String,
    /// Version requirement (semver range expression).
    pub version_req: VersionReq,
}

/// Parses a `plugin.toml` manifest string into `PluginMetadata`.
///
/// # Format
///
/// ```toml
/// [plugin]
/// name = "my-plugin"
/// version = "1.0.0"
/// author = "Author Name"
/// description = "A description"
/// required_api_version = "1.0.0"
///
/// [[dependencies]]
/// name = "other-plugin"
/// version_req = "1.0.0"
/// ```
///
/// # Errors
///
/// Returns a string error if the TOML is malformed or required fields are missing.
pub fn parse_manifest(toml_content: &str) -> Result<PluginMetadata, String> {
    let table: toml::Table = toml_content
        .parse()
        .map_err(|e| format!("TOML parse error: {e}"))?;

    let plugin_table = table
        .get("plugin")
        .and_then(|v| v.as_table())
        .ok_or_else(|| "missing [plugin] section".to_string())?;

    let name = plugin_table
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing plugin.name".to_string())?
        .to_string();

    let version_str = plugin_table
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing plugin.version".to_string())?;
    let version: Version = version_str
        .parse()
        .map_err(|e| format!("invalid plugin.version: {e}"))?;

    let author = plugin_table
        .get("author")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let description = plugin_table
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let required_api_str = plugin_table
        .get("required_api_version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing plugin.required_api_version".to_string())?;
    let required_api_version: Version = required_api_str
        .parse()
        .map_err(|e| format!("invalid plugin.required_api_version: {e}"))?;

    let mut dependencies = Vec::new();
    if let Some(deps_array) = table.get("dependencies").and_then(|v| v.as_array()) {
        for dep_value in deps_array {
            let dep_table = dep_value
                .as_table()
                .ok_or_else(|| "dependency entry must be a table".to_string())?;

            let dep_name = dep_table
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "missing dependency.name".to_string())?
                .to_string();

            let dep_version_str = dep_table
                .get("version_req")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0");

            let dep_version: Version = dep_version_str
                .parse()
                .map_err(|e| format!("invalid dependency version_req: {e}"))?;

            dependencies.push(PluginDependency {
                name: dep_name,
                version_req: VersionReq::new(dep_version, true),
            });
        }
    }

    Ok(PluginMetadata {
        name,
        version,
        author,
        description,
        dependencies,
        required_api_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_manifest_with_all_fields() {
        // Validates: Requirement 1.2, Requirement 6.2
        let toml = r#"
[plugin]
name = "test-plugin"
version = "1.2.3"
author = "Test Author"
description = "A test plugin"
required_api_version = "1.0.0"

[[dependencies]]
name = "dep-a"
version_req = "1.0.0"
"#;
        let meta = parse_manifest(toml).unwrap();
        assert_eq!(meta.name, "test-plugin");
        assert_eq!(meta.version, Version::new(1, 2, 3));
        assert_eq!(meta.author, "Test Author");
        assert_eq!(meta.description, "A test plugin");
        assert_eq!(meta.required_api_version, Version::new(1, 0, 0));
        assert_eq!(meta.dependencies.len(), 1);
        assert_eq!(meta.dependencies[0].name, "dep-a");
    }

    #[test]
    fn parse_manifest_minimal() {
        // Validates: Requirement 1.2
        let toml = r#"
[plugin]
name = "minimal"
version = "0.1.0"
required_api_version = "1.0.0"
"#;
        let meta = parse_manifest(toml).unwrap();
        assert_eq!(meta.name, "minimal");
        assert_eq!(meta.author, "");
        assert_eq!(meta.description, "");
        assert!(meta.dependencies.is_empty());
    }

    #[test]
    fn parse_manifest_missing_name_fails() {
        // Validates: Requirement 1.2
        let toml = r#"
[plugin]
version = "1.0.0"
required_api_version = "1.0.0"
"#;
        assert!(parse_manifest(toml).is_err());
    }

    #[test]
    fn parse_manifest_missing_plugin_section_fails() {
        // Validates: Requirement 1.2
        let toml = r#"
[other]
name = "test"
"#;
        assert!(parse_manifest(toml).is_err());
    }

    #[test]
    fn parse_manifest_invalid_version_fails() {
        // Validates: Requirement 6.2
        let toml = r#"
[plugin]
name = "bad-version"
version = "not.a.version"
required_api_version = "1.0.0"
"#;
        assert!(parse_manifest(toml).is_err());
    }

    #[test]
    fn parse_manifest_multiple_dependencies() {
        // Validates: Requirement 3.3
        let toml = r#"
[plugin]
name = "multi-dep"
version = "1.0.0"
required_api_version = "1.0.0"

[[dependencies]]
name = "dep-a"
version_req = "1.0.0"

[[dependencies]]
name = "dep-b"
version_req = "2.1.0"
"#;
        let meta = parse_manifest(toml).unwrap();
        assert_eq!(meta.dependencies.len(), 2);
        assert_eq!(meta.dependencies[0].name, "dep-a");
        assert_eq!(meta.dependencies[1].name, "dep-b");
    }

    #[test]
    fn plugin_metadata_equality() {
        // Validates: Requirement 1.2
        let meta1 = PluginMetadata {
            name: "test".to_string(),
            version: Version::new(1, 0, 0),
            author: "Author".to_string(),
            description: "Desc".to_string(),
            dependencies: vec![],
            required_api_version: Version::new(1, 0, 0),
        };
        let meta2 = meta1.clone();
        assert_eq!(meta1, meta2);
    }
}
