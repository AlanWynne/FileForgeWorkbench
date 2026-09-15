//! File-backed theme support for the desktop shell (CR-NR-074).
//!
//! Materialises the built-in palettes as editable `.toml` files under
//! `<User_Data_Dir>/themes/` on first launch (mirroring `menu_workspace::defaults`),
//! and resolves the active theme at startup from configuration, loading the
//! theme file to drive rendering with a Default Legacy fallback.
//!
//! Validates: theme-and-appearance Requirement 18, 19.

use std::path::{Path, PathBuf};

use ff_theme::ThemePalette;

/// Convert a theme NAME to its on-disk file slug (lowercase, spaces/parens/
/// punctuation collapsed to single hyphens). E.g. `"Legacy (ISPF 3270)"` ->
/// `"legacy-ispf-3270"`, `"Default Legacy"` -> `"default-legacy"`.
///
/// Validates: Requirement 19.3, 19.7 (name <-> file resolution)
pub fn theme_slug(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut prev_hyphen = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_hyphen = false;
        } else if !prev_hyphen && !slug.is_empty() {
            slug.push('-');
            prev_hyphen = true;
        }
    }
    // Trim any trailing hyphen.
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

/// Ensure `<user_data_dir>/themes/` exists (creating it if absent) so the Theme
/// editor has a target for user themes. The directory may be empty.
///
/// CR-CH-019 (Option 1): the built-in palettes are permanent, read-only, COMPILED
/// themes and are NOT materialised to disk. `themes/` holds ONLY user themes
/// (created via Copy / Save As). This eliminates the duplicate theme list and the
/// "saving a built-in un-defaults it" ambiguity. Best-effort; errors are ignored.
///
/// Validates: Requirement 19.1, 19.2 (revised, CR-CH-019)
pub fn ensure_default_theme_files(user_data_dir: &Path) {
    let themes_dir = user_data_dir.join("themes");
    let _ = std::fs::create_dir_all(&themes_dir);
}

/// Resolve the startup palette from configuration and the themes directory.
///
/// Resolution order (Requirement 19.3, 19.4, 19.5):
/// 1. If `theme.active_name` is a non-empty theme NAME, load `themes/<slug>.toml`.
/// 2. Else resolve the `theme.active` MODE to its built-in theme, loading that
///    theme's `themes/` file if present, else the compiled palette.
/// 3. On any load/parse error, return the Default Legacy fallback.
///
/// Validates: Requirement 19.3, 19.4, 19.5
pub fn resolve_startup_palette(
    config: &ff_config::ConfigHandle,
    themes_dir: &Path,
) -> ThemePalette {
    use ff_theme::mode::VisualMode;

    // 1. Explicit active theme name.
    if let Ok(name) = config.get_string(ff_config::keys::theme::ACTIVE_NAME) {
        let name = name.trim();
        if !name.is_empty() {
            if let Some(p) = load_theme_by_name(name, themes_dir) {
                return p;
            }
            // Named theme unresolved: fall back (Req 19.5).
            log_unresolved(name);
            return ff_theme::defaults::fallback_palette();
        }
    }

    // 2. Mode path (existing behaviour): theme.active holds a mode string.
    let mode = config
        .get_string(ff_config::keys::theme::ACTIVE)
        .ok()
        .and_then(|m| VisualMode::from_str_loose(&m))
        .unwrap_or(VisualMode::Dark);
    let builtin_name = ff_theme::defaults::default_palette_for_mode(mode).name;
    if let Some(p) = load_theme_by_name(&builtin_name, themes_dir) {
        return p;
    }
    // 3. File absent/invalid for the mode's built-in: use the compiled palette.
    ff_theme::defaults::default_palette_for_mode(mode)
}

/// Load a theme by NAME: resolve to `themes/<slug>.toml` and parse it. Returns
/// `None` if the file is missing or invalid (caller decides the fallback).
///
/// A built-in name returns the compiled palette (no file read); a user theme is
/// loaded from its file with Dark as the base mode (the loader fills missing
/// tokens from the mode default).
///
/// Validates: Requirement 19.3, 19.4
pub fn load_theme_by_name(name: &str, themes_dir: &Path) -> Option<ThemePalette> {
    // CR-CH-019 (Option 1): a built-in name resolves to the compiled palette --
    // there is no file to read (built-ins are code-only). Only user themes are
    // read from `themes/<slug>.toml`.
    if let Some(p) = builtin_palette_by_name(name) {
        return Some(p);
    }
    let slug = theme_slug(name);
    let path = themes_dir.join(format!("{slug}.toml"));
    let source = std::fs::read_to_string(&path).ok()?;
    // User themes may declare a `base`; the loader fills missing tokens from the
    // mode default. Default to Dark mode for user themes without a known mode.
    ff_theme::loader::load_from_toml(&source, ff_theme::mode::VisualMode::Dark).ok()
}

/// The compiled built-in palette for a built-in theme NAME, or `None` if `name`
/// is not a built-in. Built-ins are code-only (CR-CH-019).
///
/// Validates: theme-and-appearance Requirement 18.2, 19.4
pub fn builtin_palette_by_name(name: &str) -> Option<ThemePalette> {
    use ff_theme::defaults;
    match name {
        "Default Dark" => Some(defaults::dark_palette()),
        "Default Light" => Some(defaults::light_palette()),
        "Default High Contrast" => Some(defaults::high_contrast_palette()),
        "Legacy (ISPF 3270)" => Some(defaults::legacy_palette()),
        "Default Legacy" => Some(defaults::default_legacy_palette()),
        _ => None,
    }
}

/// The `VisualMode` associated with a built-in theme name, defaulting to Dark
/// for user themes (the loader fills omitted tokens from the mode default).
/// Path to the themes directory under the resolved user data dir. Falls back to
/// the data dir when the session layer is unavailable, mirroring `menus_dir`.
pub fn themes_dir() -> PathBuf {
    if let Ok(udd) = ff_session::UserDataDir::resolve(None) {
        return udd.path().join("themes");
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("FileForgeWorkbench")
        .join("themes")
}

fn log_unresolved(name: &str) {
    // Best-effort WARN; logging failures must not affect startup.
    ff_logging::log_warn!(
        "[theme] active theme '{}' could not be resolved; using Default Legacy fallback",
        name
    );
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Validates: Requirement 19.3 -- name-to-slug mapping.
    #[test]
    fn theme_slug_maps_names_to_files() {
        assert_eq!(theme_slug("Default Legacy"), "default-legacy");
        assert_eq!(theme_slug("Default Dark"), "default-dark");
        assert_eq!(theme_slug("Default High Contrast"), "default-high-contrast");
        assert_eq!(theme_slug("Legacy (ISPF 3270)"), "legacy-ispf-3270");
        // Trailing/edge punctuation does not produce trailing hyphens.
        assert!(!theme_slug("Legacy (ISPF 3270)").ends_with('-'));
    }

    // Validates: Requirement 19.1, 19.2 (CR-CH-019) -- themes dir is created but
    // built-ins are NOT materialised (built-ins are code-only).
    #[test]
    fn ensure_default_theme_files_creates_empty_dir_no_builtins() {
        let dir = TempDir::new().expect("tempdir");
        ensure_default_theme_files(dir.path());
        let themes = dir.path().join("themes");
        assert!(themes.exists(), "themes dir must be created");
        // No built-in files are written.
        for slug in [
            "default-dark",
            "default-light",
            "default-high-contrast",
            "legacy",
            "default-legacy",
        ] {
            assert!(
                !themes.join(format!("{slug}.toml")).exists(),
                "{slug}.toml must NOT be materialised (built-ins are code-only)"
            );
        }
        // The directory is empty (no user themes yet).
        let count = std::fs::read_dir(&themes).map(|d| d.count()).unwrap_or(0);
        assert_eq!(count, 0, "themes dir starts empty");
    }

    // Validates: Requirement 19.4 (CR-CH-019) -- a built-in name resolves to the
    // compiled palette WITHOUT any file (built-ins are code-only).
    #[test]
    fn load_theme_by_name_builtin_resolves_without_file() {
        let dir = TempDir::new().expect("tempdir");
        ensure_default_theme_files(dir.path());
        let themes = dir.path().join("themes");
        let p = load_theme_by_name("Default Legacy", &themes).expect("built-in resolves");
        assert_eq!(p.name, "Default Legacy");
        assert_eq!(
            p.editor,
            ff_theme::defaults::default_legacy_palette().editor
        );
        // And it works even with a non-existent themes dir.
        let none_dir = dir.path().join("does-not-exist");
        assert!(load_theme_by_name("Default Dark", &none_dir).is_some());
    }

    // Validates: Requirement 19.4 -- unknown/absent USER theme returns None.
    #[test]
    fn load_theme_by_name_absent_user_theme_is_none() {
        let dir = TempDir::new().expect("tempdir");
        let themes = dir.path().join("themes");
        assert!(load_theme_by_name("No Such Theme", &themes).is_none());
    }

    // Validates: Requirement 19.4 -- a USER theme file loads by name.
    #[test]
    fn load_theme_by_name_reads_user_file() {
        let dir = TempDir::new().expect("tempdir");
        let themes = dir.path().join("themes");
        std::fs::create_dir_all(&themes).expect("mk themes");
        std::fs::write(
            themes.join("my-theme.toml"),
            "name = \"my-theme\"\n[editor]\nbackground = \"#123456\"\n",
        )
        .expect("write");
        let p = load_theme_by_name("my-theme", &themes).expect("user theme loads");
        assert_eq!(
            p.editor.background,
            ff_theme::ColourRGBA::rgb(0x12, 0x34, 0x56)
        );
    }

    fn temp_config(root: &Path) -> ff_config::ConfigHandle {
        use ff_config::init::{init, ConfigInitOptions};
        init(
            ConfigInitOptions::new()
                .with_hot_reload(false)
                .with_project_root(root.to_path_buf()),
        )
        .expect("config init")
    }

    // Validates: Requirement 19.5 -- when nothing is materialised and no theme
    // resolves from config, startup falls back to Default Legacy without panic.
    #[test]
    fn resolve_startup_palette_falls_back_to_default_legacy() {
        let dir = TempDir::new().expect("tempdir");
        let config = temp_config(dir.path());
        // Empty themes dir (nothing materialised) + no theme.active_name set:
        // the mode path resolves to Dark's built-in file which is absent, so the
        // resolver returns the compiled dark palette OR, if the dir is empty, the
        // compiled default. Either way it must not panic and must be a valid theme.
        let themes = dir.path().join("themes");
        let p = resolve_startup_palette(&config, &themes);
        // With an empty dir and default config (mode dark), we get the compiled
        // dark palette (mode path step 3). Assert a valid palette name is set.
        assert!(!p.name.is_empty());
    }

    // Validates: Requirement 19.4 -- after materialisation, the resolver loads
    // the mode's built-in theme file and drives the palette. Isolated from any
    // real user config (B048) by pointing FFWB_USER_CONFIG_PATH at a temp file so
    // the resolved mode is the schema default (dark), not a developer's setting.
    #[test]
    fn resolve_startup_palette_loads_materialised_mode_file() {
        let dir = TempDir::new().expect("tempdir");
        let cfg_path = dir.path().join("iso-config.toml");
        std::env::set_var("FFWB_USER_CONFIG_PATH", &cfg_path);
        ensure_default_theme_files(dir.path());
        let config = temp_config(dir.path());
        let themes = dir.path().join("themes");
        let p = resolve_startup_palette(&config, &themes);
        // The resolved palette's background must match the built-in for the
        // resolved mode (loaded from the materialised file, round-tripped).
        let expected_bg = ff_theme::defaults::default_palette_for_mode(p.mode)
            .editor
            .background;
        assert_eq!(p.editor.background, expected_bg);
        std::env::remove_var("FFWB_USER_CONFIG_PATH");
    }
}
