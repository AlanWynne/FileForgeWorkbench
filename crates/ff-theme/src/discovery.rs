//! Theme discovery: scanning the themes directory for user-created theme files
//! and exposing the full list of available themes (built-in + user-created).

use std::path::{Path, PathBuf};

use crate::error::ThemeError;
use crate::palette::ThemePalette;
use crate::serialiser;

/// Metadata for a single available theme.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeInfo {
    /// Display name (from `name` field in TOML, or filename stem for user themes).
    pub name: String,
    /// Whether this theme is compiled into the binary.
    pub is_builtin: bool,
    /// Path to the TOML file (`None` for built-in themes).
    pub path: Option<PathBuf>,
    /// Base theme declared in the file (`base = "..."`) if any.
    pub base: Option<String>,
}

/// Names of the four built-in themes.
pub const BUILTIN_THEME_NAMES: &[&str] = &[
    "Default Dark",
    "Default Light",
    "Default High Contrast",
    "Legacy (ISPF 3270)",
];

/// Return `ThemeInfo` entries for all four built-in themes.
pub fn builtin_themes() -> Vec<ThemeInfo> {
    BUILTIN_THEME_NAMES
        .iter()
        .map(|name| ThemeInfo {
            name: name.to_string(),
            is_builtin: true,
            path: None,
            base: None,
        })
        .collect()
}

/// Scan `themes_dir` for `.toml` files and return a `ThemeInfo` for each.
///
/// Files that cannot be read or parsed are silently skipped (a WARN would be
/// emitted in production; here we just omit them so callers get a clean list).
///
/// # Errors
///
/// Returns `ThemeError::Io` only if the directory itself cannot be read.
pub fn scan_themes_dir(themes_dir: &Path) -> Result<Vec<ThemeInfo>, ThemeError> {
    if !themes_dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(themes_dir).map_err(|e| ThemeError::Io {
        path: themes_dir.to_string_lossy().into_owned(),
        detail: e.to_string(),
    })?;

    let mut infos = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            let info = theme_info_from_toml(&content, &path);
            infos.push(info);
        }
    }

    // Sort by name for deterministic ordering.
    infos.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(infos)
}

/// Return all available themes: built-ins first, then user-created (sorted by name).
///
/// Does not fail if the themes directory is absent — returns only built-ins.
pub fn list_all_themes(themes_dir: &Path) -> Vec<ThemeInfo> {
    let mut all = builtin_themes();
    if let Ok(user) = scan_themes_dir(themes_dir) {
        all.extend(user);
    }
    all
}

/// Serialise `palette` to a TOML string suitable for saving as a user theme file.
///
/// The `name` field in the output is set to the provided `name` argument.
///
/// # Errors
///
/// Propagates any `ThemeError` from the serialiser.
pub fn export_theme(palette: &ThemePalette, name: &str) -> Result<String, ThemeError> {
    let mut p = palette.clone();
    p.name = name.to_string();
    Ok(serialiser::serialise(&p))
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn theme_info_from_toml(content: &str, path: &Path) -> ThemeInfo {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let (name, base) = if let Ok(table) = content.parse::<toml::Table>() {
        let name = table
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&stem)
            .to_string();
        let base = table
            .get("base")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        (name, base)
    } else {
        (stem, None)
    };

    ThemeInfo {
        name,
        is_builtin: false,
        path: Some(path.to_path_buf()),
        base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn builtin_themes_returns_four_entries() {
        // Validates: Requirement 14.2, 14.6
        let themes = builtin_themes();
        assert_eq!(themes.len(), 4);
        assert!(themes.iter().all(|t| t.is_builtin));
        assert!(themes.iter().all(|t| t.path.is_none()));
    }

    #[test]
    fn list_all_themes_includes_builtins_when_dir_absent() {
        // Validates: Requirement 14.6 — built-ins always present
        let dir = PathBuf::from("/nonexistent/themes/dir");
        let themes = list_all_themes(&dir);
        assert_eq!(themes.len(), 4);
        assert!(themes.iter().all(|t| t.is_builtin));
    }

    #[test]
    fn scan_themes_dir_finds_toml_files() {
        // Validates: Requirement 14.2, 14.3
        let dir = TempDir::new().unwrap();
        let toml_path = dir.path().join("my-theme.toml");
        std::fs::write(
            &toml_path,
            "name = \"My Theme\"\n[editor]\nbackground = \"#FF0000\"\n",
        )
        .unwrap();

        let infos = scan_themes_dir(dir.path()).unwrap();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].name, "My Theme");
        assert!(!infos[0].is_builtin);
        assert_eq!(infos[0].path.as_deref(), Some(toml_path.as_path()));
    }

    #[test]
    fn scan_themes_dir_ignores_non_toml_files() {
        // Validates: Requirement 14.2 — only .toml files are picked up
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("notes.txt"), "not a theme").unwrap();
        std::fs::write(dir.path().join("theme.json"), "{}").unwrap();
        std::fs::write(dir.path().join("valid.toml"), r#"name = "Valid""#).unwrap();

        let infos = scan_themes_dir(dir.path()).unwrap();
        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].name, "Valid");
    }

    #[test]
    fn scan_themes_dir_reads_base_field() {
        // Validates: Requirement 14.4 — base inheritance declared in file
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("child.toml"),
            "name = \"Child Theme\"\nbase = \"Default Dark\"\n[editor]\nbackground = \"#123456\"\n",
        )
        .unwrap();

        let infos = scan_themes_dir(dir.path()).unwrap();
        assert_eq!(infos[0].base.as_deref(), Some("Default Dark"));
    }

    #[test]
    fn list_all_themes_includes_user_themes() {
        // Validates: Requirement 14.6 — list includes both built-in and user themes
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("custom.toml"), r#"name = "Custom""#).unwrap();

        let themes = list_all_themes(dir.path());
        assert_eq!(themes.len(), 5); // 4 built-in + 1 user
        assert!(themes.iter().any(|t| t.name == "Custom" && !t.is_builtin));
        assert!(themes
            .iter()
            .any(|t| t.name == "Default Dark" && t.is_builtin));
    }

    #[test]
    fn export_theme_round_trips_name() {
        // Validates: Requirement 14.9 — export sets the name field
        let palette = crate::defaults::dark_palette();
        let toml = export_theme(&palette, "My Export").unwrap();
        // Re-parse and check name
        let table: toml::Table = toml.parse().unwrap();
        assert_eq!(
            table.get("name").and_then(|v| v.as_str()),
            Some("My Export")
        );
    }

    #[test]
    fn export_theme_produces_valid_toml() {
        // Validates: Requirement 14.9 — output is valid TOML
        let palette = crate::defaults::dark_palette();
        let toml = export_theme(&palette, "Test").unwrap();
        assert!(toml.parse::<toml::Table>().is_ok());
    }

    #[test]
    fn scan_themes_dir_returns_empty_for_absent_dir() {
        // Validates: Requirement 14.2 — no error when dir doesn't exist
        let infos = scan_themes_dir(Path::new("/nonexistent/path/themes")).unwrap();
        assert!(infos.is_empty());
    }

    #[test]
    fn theme_info_uses_filename_stem_when_name_absent() {
        // Validates: Requirement 14.2 — filename stem used as fallback name
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("my-custom-theme.toml"),
            "[editor]\nbackground = \"#000000\"\n",
        )
        .unwrap();

        let infos = scan_themes_dir(dir.path()).unwrap();
        assert_eq!(infos[0].name, "my-custom-theme");
    }
}
