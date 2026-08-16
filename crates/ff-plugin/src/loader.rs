//! Plugin discovery and directory scanning.
//!
//! Provides utility functions for scanning plugin directories and
//! parsing manifests. The primary discovery logic lives in `PluginRegistry`.

use std::path::Path;

use crate::metadata::{parse_manifest, PluginMetadata};

/// Scan a directory for plugin manifests and parse them.
///
/// Each subdirectory containing a `plugin.toml` file is treated as a plugin.
/// Returns a list of successfully parsed metadata entries.
/// Malformed or missing manifests are skipped with warnings.
pub fn scan_plugin_directory(directory: &Path) -> Vec<(PluginMetadata, std::path::PathBuf)> {
    let mut results = Vec::new();

    if !directory.exists() || !directory.is_dir() {
        return results;
    }

    let entries = match std::fs::read_dir(directory) {
        Ok(e) => e,
        Err(_) => return results,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("plugin.toml");
        if !manifest_path.exists() {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            if let Ok(meta) = parse_manifest(&content) {
                results.push((meta, path));
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn scan_empty_directory_returns_empty() {
        // Validates: Requirement 3.1
        let dir = TempDir::new().unwrap();
        let results = scan_plugin_directory(dir.path());
        assert!(results.is_empty());
    }

    #[test]
    fn scan_nonexistent_directory_returns_empty() {
        // Validates: Requirement 3.1
        let results = scan_plugin_directory(Path::new("/nonexistent/path"));
        assert!(results.is_empty());
    }

    #[test]
    fn scan_directory_with_valid_manifest() {
        // Validates: Requirement 3.1
        let dir = TempDir::new().unwrap();
        let plugin_dir = dir.path().join("my-plugin");
        std::fs::create_dir(&plugin_dir).unwrap();
        std::fs::write(
            plugin_dir.join("plugin.toml"),
            r#"
[plugin]
name = "my-plugin"
version = "1.0.0"
required_api_version = "1.0.0"
"#,
        )
        .unwrap();

        let results = scan_plugin_directory(dir.path());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.name, "my-plugin");
    }

    #[test]
    fn scan_skips_directories_without_manifest() {
        // Validates: Requirement 3.1
        let dir = TempDir::new().unwrap();
        let no_manifest = dir.path().join("no-manifest");
        std::fs::create_dir(&no_manifest).unwrap();
        std::fs::write(no_manifest.join("readme.md"), "hello").unwrap();

        let results = scan_plugin_directory(dir.path());
        assert!(results.is_empty());
    }

    #[test]
    fn scan_skips_malformed_manifest() {
        // Validates: Requirement 3.1
        let dir = TempDir::new().unwrap();
        let plugin_dir = dir.path().join("bad-plugin");
        std::fs::create_dir(&plugin_dir).unwrap();
        std::fs::write(plugin_dir.join("plugin.toml"), "not valid toml {{{}").unwrap();

        let results = scan_plugin_directory(dir.path());
        assert!(results.is_empty());
    }
}
