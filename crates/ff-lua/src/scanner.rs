//! Macro directory scanning and name resolution.
//!
//! Discovers `.lua` files in configured directories and resolves macro names.
//! Addresses: Requirement 9 (all criteria)

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::LuaEngineError;

/// Priority levels for macro directory sources (higher = preferred on conflict).
///
/// Addresses: Requirement 9 AC 4
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DirectoryPriority {
    /// User-level macros (~/.config/ffworkbench/macros/).
    User = 0,
    /// Workspace-level macros (workspace_root/macros/).
    Workspace = 1,
}

/// Metadata about a discovered macro script file.
///
/// Addresses: Requirement 9 AC 3
#[derive(Debug, Clone)]
pub struct MacroScript {
    /// Absolute filesystem path to the .lua file.
    pub path: PathBuf,
    /// Macro name (filename without extension).
    pub name: String,
    /// The macro directory this script was discovered from.
    pub source_directory: PathBuf,
    /// Priority level (workspace > user) for shadowing resolution.
    pub priority: DirectoryPriority,
}

/// Maximum recursive depth for directory scanning.
const MAX_SCAN_DEPTH: usize = 3;

/// Scans configured directories for .lua files (recursive, max 3 levels).
///
/// Returns discovered macro names with resolved paths.
/// Higher-priority directories override lower-priority ones on name collision.
///
/// Addresses: Requirement 9 AC 1, AC 2, AC 3, AC 4
pub fn scan_directories(
    directories: &[(PathBuf, DirectoryPriority)],
) -> Result<HashMap<String, MacroScript>, LuaEngineError> {
    let mut all_scripts: Vec<MacroScript> = Vec::new();

    for (dir, priority) in directories {
        if dir.exists() && dir.is_dir() {
            let scripts = scan_directory_recursive(dir, dir, *priority, 0)?;
            all_scripts.extend(scripts);
        }
    }

    // Resolve name conflicts: higher priority wins
    let mut resolved: HashMap<String, MacroScript> = HashMap::new();
    for script in all_scripts {
        let name = script.name.clone();
        match resolved.get(&name) {
            Some(existing) if existing.priority >= script.priority => {
                // Existing has higher or equal priority, keep it
            }
            _ => {
                resolved.insert(name, script);
            }
        }
    }

    Ok(resolved)
}

/// Resolves a macro name to its file path from the scanned results.
pub fn resolve_name<'a>(
    available_macros: &'a HashMap<String, MacroScript>,
    name: &str,
) -> Option<&'a Path> {
    available_macros.get(name).map(|s| s.path.as_path())
}

/// Recursively scan a directory for .lua files up to MAX_SCAN_DEPTH levels.
fn scan_directory_recursive(
    root_dir: &Path,
    current_dir: &Path,
    priority: DirectoryPriority,
    depth: usize,
) -> Result<Vec<MacroScript>, LuaEngineError> {
    if depth > MAX_SCAN_DEPTH {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(current_dir).map_err(|e| LuaEngineError::ScanError {
        path: current_dir.display().to_string(),
        reason: e.to_string(),
    })?;

    let mut scripts = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| LuaEngineError::ScanError {
            path: current_dir.display().to_string(),
            reason: e.to_string(),
        })?;

        let path = entry.path();

        if path.is_dir() {
            let sub_scripts = scan_directory_recursive(root_dir, &path, priority, depth + 1)?;
            scripts.extend(sub_scripts);
        } else if path.extension().is_some_and(|ext| ext == "lua") {
            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                scripts.push(MacroScript {
                    path: path.clone(),
                    name: name.to_string(),
                    source_directory: root_dir.to_path_buf(),
                    priority,
                });
            }
        }
    }

    Ok(scripts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_directory_structure(dir: &Path) {
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("format.lua"), "-- format macro").unwrap();
        std::fs::write(dir.join("sort.lua"), "-- sort macro").unwrap();
        std::fs::write(dir.join("sub/helper.lua"), "-- helper macro").unwrap();
    }

    // Validates: Requirement 9.1, 9.3
    #[test]
    fn scan_discovers_lua_files_recursively() {
        let tmp = TempDir::new().unwrap();
        create_test_directory_structure(tmp.path());

        let dirs = vec![(tmp.path().to_path_buf(), DirectoryPriority::User)];
        let result = scan_directories(&dirs).unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.contains_key("format"));
        assert!(result.contains_key("sort"));
        assert!(result.contains_key("helper"));
    }

    // Validates: Requirement 9.3
    #[test]
    fn scan_keys_by_filename_without_extension() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("my_macro.lua"), "-- test").unwrap();

        let dirs = vec![(tmp.path().to_path_buf(), DirectoryPriority::User)];
        let result = scan_directories(&dirs).unwrap();

        assert!(result.contains_key("my_macro"));
        assert!(!result.contains_key("my_macro.lua"));
    }

    // Validates: Requirement 9.4
    #[test]
    fn workspace_priority_overrides_user_on_name_collision() {
        let user_dir = TempDir::new().unwrap();
        let workspace_dir = TempDir::new().unwrap();

        std::fs::write(user_dir.path().join("format.lua"), "-- user version").unwrap();
        std::fs::write(
            workspace_dir.path().join("format.lua"),
            "-- workspace version",
        )
        .unwrap();

        let dirs = vec![
            (user_dir.path().to_path_buf(), DirectoryPriority::User),
            (
                workspace_dir.path().to_path_buf(),
                DirectoryPriority::Workspace,
            ),
        ];
        let result = scan_directories(&dirs).unwrap();

        let format_script = result.get("format").unwrap();
        assert_eq!(format_script.priority, DirectoryPriority::Workspace);
        assert!(format_script.path.starts_with(workspace_dir.path()));
    }

    // Validates: Requirement 9.1
    #[test]
    fn scan_respects_max_depth_limit() {
        let tmp = TempDir::new().unwrap();
        let deep_path = tmp.path().join("a/b/c/d");
        std::fs::create_dir_all(&deep_path).unwrap();
        std::fs::write(deep_path.join("deep.lua"), "-- too deep").unwrap();
        // Level 3 should work
        let level3_path = tmp.path().join("a/b/c");
        std::fs::write(level3_path.join("level3.lua"), "-- level 3").unwrap();

        let dirs = vec![(tmp.path().to_path_buf(), DirectoryPriority::User)];
        let result = scan_directories(&dirs).unwrap();

        assert!(result.contains_key("level3"));
        assert!(!result.contains_key("deep"));
    }

    #[test]
    fn scan_ignores_non_lua_files() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("readme.md"), "# docs").unwrap();
        std::fs::write(tmp.path().join("config.toml"), "[settings]").unwrap();
        std::fs::write(tmp.path().join("macro.lua"), "-- lua macro").unwrap();

        let dirs = vec![(tmp.path().to_path_buf(), DirectoryPriority::User)];
        let result = scan_directories(&dirs).unwrap();

        assert_eq!(result.len(), 1);
        assert!(result.contains_key("macro"));
    }

    #[test]
    fn scan_handles_nonexistent_directory_gracefully() {
        let dirs = vec![(PathBuf::from("/nonexistent/path"), DirectoryPriority::User)];
        let result = scan_directories(&dirs).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn resolve_name_returns_path_for_known_macro() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("test.lua"), "-- test").unwrap();

        let dirs = vec![(tmp.path().to_path_buf(), DirectoryPriority::User)];
        let macros = scan_directories(&dirs).unwrap();

        let path = resolve_name(&macros, "test");
        assert!(path.is_some());
        assert!(path.unwrap().ends_with("test.lua"));
    }

    #[test]
    fn resolve_name_returns_none_for_unknown_macro() {
        let macros = HashMap::new();
        assert!(resolve_name(&macros, "unknown").is_none());
    }
}
