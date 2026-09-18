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

/// Names of the built-in themes (CR-CH-024: four total). `Default Legacy` carries
/// the ISPF 3270 legacy colours and is BOTH a selectable built-in and the
/// compiled Fallback_Theme (Req 18.1/18.2/18.3). The former separate
/// `Legacy (ISPF 3270)` built-in was removed as redundant -- the legacy look now
/// lives only under `Default Legacy`; `THEME Legacy` resolves to it via shorthand.
pub const BUILTIN_THEME_NAMES: &[&str] = &[
    "Default Dark",
    "Default Light",
    "Default High Contrast",
    "Default Legacy",
];

/// Return `ThemeInfo` entries for all built-in themes.
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

/// True when `name` is one of the compiled built-in theme names. Built-ins are
/// read-only and code-only; a user theme cannot share a built-in name in the
/// available-themes list (theme-and-appearance Req 19.2a, CR-CH-019).
pub fn is_builtin_theme(name: &str) -> bool {
    BUILTIN_THEME_NAMES.contains(&name)
}

/// Return all available themes: the compiled built-ins first, then user-created
/// themes from `themes_dir`, DE-DUPLICATED by name with the built-in winning.
///
/// A user `.toml` whose `name` matches a built-in is ignored for listing (a user
/// cannot shadow a built-in name). No theme appears more than once.
///
/// Does not fail if the themes directory is absent — returns only built-ins.
///
/// Validates: theme-and-appearance Requirement 19.2a (CR-CH-019)
pub fn list_all_themes(themes_dir: &Path) -> Vec<ThemeInfo> {
    let mut all = builtin_themes();
    let mut seen: std::collections::HashSet<String> = all.iter().map(|t| t.name.clone()).collect();
    if let Ok(user) = scan_themes_dir(themes_dir) {
        for info in user {
            // Skip a user theme whose name collides with a built-in or another
            // already-listed user theme (built-in / first-seen wins).
            if seen.insert(info.name.clone()) {
                all.push(info);
            }
        }
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
        // Validates: Requirement 14.2, 14.6; theme-and-appearance Req 18.3
        // (CR-CH-024: four built-ins -- Default Dark/Light/High Contrast/Legacy;
        // the separate Legacy (ISPF 3270) built-in was removed).
        let themes = builtin_themes();
        assert_eq!(themes.len(), 4);
        assert!(themes.iter().all(|t| t.is_builtin));
        assert!(themes.iter().all(|t| t.path.is_none()));
        assert!(
            !themes.iter().any(|t| t.name == "Legacy (ISPF 3270)"),
            "Legacy (ISPF 3270) is no longer a built-in (CR-CH-024)"
        );
    }

    // Validates: theme-and-appearance Requirement 18.3 -- Default Legacy is a
    // listed, selectable built-in theme.
    #[test]
    fn builtin_themes_includes_default_legacy() {
        let themes = builtin_themes();
        assert!(
            themes.iter().any(|t| t.name == "Default Legacy"),
            "Default Legacy must be a built-in theme"
        );
    }

    #[test]
    fn list_all_themes_includes_builtins_when_dir_absent() {
        // Validates: Requirement 14.6 — built-ins always present; Req 18.3.
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
        assert_eq!(themes.len(), 5); // 4 built-in (incl. Default Legacy) + 1 user
        assert!(themes.iter().any(|t| t.name == "Custom" && !t.is_builtin));
        // No theme appears more than once (Req 19.2a).
        let mut names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        names.sort_unstable();
        let unique = names.len();
        names.dedup();
        assert_eq!(names.len(), unique, "no duplicate theme names");
        assert!(themes
            .iter()
            .any(|t| t.name == "Default Dark" && t.is_builtin));
    }

    // Validates: Requirement 19.2a (CR-CH-019) -- a user file whose name matches
    // a built-in is ignored for listing (built-in wins); no duplicate appears.
    #[test]
    fn list_all_themes_dedups_builtin_named_user_file() {
        let dir = TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("legacy.toml"),
            "name = \"Default Legacy\"\n[editor]\nbackground = \"#123456\"\n",
        )
        .unwrap();
        let themes = list_all_themes(dir.path());
        let matches: Vec<&ThemeInfo> = themes
            .iter()
            .filter(|t| t.name == "Default Legacy")
            .collect();
        assert_eq!(matches.len(), 1, "built-in name must not be duplicated");
        assert!(
            matches[0].is_builtin,
            "the built-in entry wins over the user file"
        );
        assert_eq!(themes.len(), 4, "shadowing user file adds nothing");
    }

    // Validates: Requirement 19.2a -- is_builtin_theme identifies built-in names.
    #[test]
    fn is_builtin_theme_identifies_builtins() {
        assert!(is_builtin_theme("Default Dark"));
        // CR-CH-024: Legacy (ISPF 3270) is no longer a built-in name.
        assert!(!is_builtin_theme("Legacy (ISPF 3270)"));
        assert!(is_builtin_theme("Default Legacy"));
        assert!(!is_builtin_theme("My Custom Theme"));
        assert!(!is_builtin_theme(""));
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
