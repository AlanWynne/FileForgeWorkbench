//! # Shell Free-Function Helpers (startup + title derivation)
//!
//! Free functions that were defined at the bottom of `mod.rs` and are moved here
//! verbatim as part of the Phase 2 task 2.2 file-size split. They are
//! re-exported from `mod.rs` via `pub(crate) use mod_helpers::*;`, so every
//! existing `super::title_line_text` / `super::truncate_title` /
//! `super::line_end_from_name` reference in sibling modules and tests keeps
//! resolving unchanged. Behaviour, signatures, and visibility are identical.

use ff_config::ConfigHandle;
use ff_keys::{KeyMap, KeyMapResolver};

use super::helpers::config_value_to_toml_value;

/// Map a `Kind_Profile.line_end_mode` stored name to a `LineEndMode`
/// (CR-NR-090 B.3). `"unicode"` -> Unicode; anything else -> Default.
pub(crate) fn line_end_from_name(name: &str) -> ff_document_model::LineEndMode {
    match name.trim().to_ascii_lowercase().as_str() {
        "unicode" => ff_document_model::LineEndMode::Unicode,
        _ => ff_document_model::LineEndMode::Default,
    }
}

/// - POM tab -> app name + version
/// - FileEditor with path -> full path
/// - FileEditor without path (Untitled) -> "[Untitled]"
/// - All other kinds -> tab title string
///
/// Validates: Requirement 17.3, 17.4, 17.5, 17.6
pub(crate) fn title_line_text(tab: &crate::tab_state::TabState) -> String {
    use crate::tab_state::TabKind;
    // CR-CH-042 (menu-workspace Req 20.2/20.3; menu-and-statusbar Req 17.3/17.11):
    // the Home Context (POM) is a Menu Workspace and its Title_Line is its loaded
    // Menu_Title (from pom.toml), NOT the hardcoded application banner. It shares
    // the SAME derivation as every other Menu Workspace below (the raw
    // Menu_Title), so the POM and Settings are one uniform title source. The
    // application name/version now lives in the About dialog / status area.
    match tab.kind {
        TabKind::FileEditor => tab
            .path
            .as_deref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "[Untitled]".to_string()),
        TabKind::Untitled => "[Untitled]".to_string(),
        TabKind::FilesPanel
        | TabKind::ConfigPanel
        | TabKind::FileExplorerPanel
        | TabKind::SearchResults
        | TabKind::PluginManager
        | TabKind::EventLog
        | TabKind::MacroLibrary
        | TabKind::CommandConfigurator
        | TabKind::ThemeEditor
        | TabKind::MenusEditor
        | TabKind::KeysEditor
        | TabKind::KindsEditor => {
            // CR-NR-090 B.1: the label for a system/panel Kind is the Kind's
            // compiled default title, NOT the cached `tab.title`. This fixes the
            // Catalog Explorer (FilesPanel -> [CATALOGS]) vs File Explorer
            // (FileExplorerPanel -> [FILES]) shared-label smell. A user Kind's
            // configured title (registry override) is applied by the shell's
            // `kind_title` (which this free function cannot reach without a shell;
            // the shell render path prefers `kind_title`).
            crate::workspace_kind::BuiltinKind::from_tab_kind(tab.kind.tag(), tab.is_home)
                .default_title()
                .to_string()
        }
        // CR-CH-034 / B050 (menu-and-statusbar Req 17.10) + CR-CH-042 (Req
        // 17.11): a Menu_Workspace's Title_Line -- the POM, Settings, or any user
        // menu -- is the RAW loaded Menu_Title (single config-driven source), not
        // bracketed/uppercased and not a hardcoded banner. It is derived from the
        // CURRENTLY loaded menu each frame so an in-place context switch can never
        // leave it stale. Falls back to the cached `tab.title` only when no menu
        // is loaded yet (e.g. a fresh POM before its menu loads -> "[POM]").
        TabKind::MenuWorkspace(ref mw) => mw
            .as_ref()
            .and_then(|mw| mw.menu_title())
            .unwrap_or_else(|| tab.title.clone()),
        // The Help Context uses its cached title ("[HELP]"). CR-NR-097.
        TabKind::HelpContext => tab.title.clone(),
        // The SCRM Replay viewer uses its cached title ("[REPLAY]"). CR-NR-098.
        TabKind::ScrmViewer => tab.title.clone(),
    }
}

/// Truncate a Detached_Workspace OS-window title to at most `max` characters,
/// clamping on a char boundary so multi-byte characters are never split. When
/// the title is longer than `max`, the last character of the kept prefix is
/// replaced with an ellipsis marker so the truncation is visible.
///
/// Validates: menu-and-statusbar Requirement 18.5 (CR-CH-035, B045)
pub(crate) fn truncate_title(title: &str, max: usize) -> String {
    if title.chars().count() <= max {
        return title.to_string();
    }
    if max == 0 {
        return String::new();
    }
    // Keep `max - 1` chars and append a single-char ellipsis marker ("~") so the
    // result is exactly `max` chars and the cut is visible without a non-ASCII
    // ellipsis (documentation.md: plain ASCII in .rs).
    let kept: String = title.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}~")
}

/// Load `[context_key_maps]` from the workbench configuration into the resolver.
///
/// Reads the `context_key_maps` top-level table from `config_handle`.
/// Each sub-table key is a context name (e.g. `"editor"`, `"pom"`) and its
/// value is a key-map table using the same schema as `[global_key_map]`.
/// Invalid entries are silently skipped (warnings are not surfaced at startup).
///
/// Validates: Requirement 14.7
pub(crate) fn load_context_maps_from_config(config: &ConfigHandle, resolver: &mut KeyMapResolver) {
    use ff_config::ConfigValue;

    let Ok(ConfigValue::Table(outer)) = config.get("context_key_maps") else {
        return;
    };
    for (ctx_name, ctx_value) in outer {
        if let ConfigValue::Table(ctx_table) = ctx_value {
            // Convert ConfigTable (BTreeMap<String, ConfigValue>) to toml::Table
            // so we can reuse KeyMap::from_toml_table.
            let mut toml_map = toml::map::Map::new();
            for (k, v) in ctx_table {
                if let Some(tv) = config_value_to_toml_value(v) {
                    toml_map.insert(k, tv);
                }
            }
            let (map, _warnings) = KeyMap::from_toml_table(&toml_map, &ctx_name);
            resolver.set_context_map(ctx_name, map);
        }
    }
}

/// Resolve the command-line history file path (function-keys-and-history
/// Requirement 6.4): `<User_Data_Dir>/command_history.toml` by default, honouring
/// the profile-aware `UserDataDir`. The `FFWB_HISTORY_PATH` environment variable
/// overrides it verbatim -- a test-isolation seam (B048) mirroring
/// `FFWB_USER_CONFIG_PATH`, so tests never read/write the developer's real
/// command history. Returns `None` when no path can be resolved.
///
/// Validates: function-keys-and-history Requirement 6.4
pub(crate) fn resolve_history_path() -> Option<std::path::PathBuf> {
    if let Some(p) = std::env::var_os("FFWB_HISTORY_PATH") {
        return Some(std::path::PathBuf::from(p));
    }
    ff_session::UserDataDir::resolve(None)
        .ok()
        .map(|udd| udd.path().join(ff_keys::DEFAULT_HISTORY_FILE))
}

/// Ensure the `keymaps/` directory exists under `user_data_dir` (creating it if
/// absent), so a user has a place to author per-context key-map override files
/// (`keymaps/<context>.toml`). The directory may be empty.
///
/// Mirrors `ensure_menus_dir` / `ensure_default_theme_files`: built-in default
/// key maps are CODE-ONLY (compiled `KeyMap::default_global`) and are NEVER
/// written here.
///
/// Validates: function-keys-and-history Requirement 14.11 (CR-CH-027)
pub(crate) fn ensure_keymaps_dir(user_data_dir: &std::path::Path) {
    let keymaps_dir = user_data_dir.join("keymaps");
    // Best-effort -- ignore errors (graceful degradation).
    let _ = std::fs::create_dir_all(&keymaps_dir);
}

/// Load per-context key-map override FILES from `<user_data_dir>/keymaps/*.toml`
/// into the resolver, keyed by the file stem (the context name).
///
/// Each `keymaps/<context>.toml` uses the same key-name schema as
/// `[global_key_map]` (Base `F1`-`F12` plus `SF`/`CF`/`AF`/`GF`/`XF` prefixes).
/// A present file is registered as the Context_Key_Map for that context
/// (full-replacement over the compiled default); a file that cannot be read or
/// parsed is skipped (DEBUG record) and the context falls back to the compiled
/// default.
///
/// Called at startup AFTER [`load_context_maps_from_config`], so a
/// `keymaps/<context>.toml` FILE takes precedence over a
/// `[context_key_maps.<name>]` config section for the same context
/// (Requirement 14.12).
///
/// Validates: function-keys-and-history Requirement 14.9, 14.10, 14.12 (CR-CH-027)
pub(crate) fn load_context_maps_from_keymaps_dir(
    keymaps_dir: &std::path::Path,
    resolver: &mut KeyMapResolver,
) {
    let Ok(entries) = std::fs::read_dir(keymaps_dir) else {
        return; // Absent/unreadable dir: every context keeps the compiled default.
    };
    for entry in entries.flatten() {
        let path = entry.path();
        // Only `*.toml` files; the stem is the context name.
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let Some(ctx_name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            ff_logging::log_debug!("[keys] could not read keymaps file {}", path.display());
            continue;
        };
        let table: toml::Table = match toml::from_str(&text) {
            Ok(t) => t,
            Err(e) => {
                // Req 14.10: malformed file skipped; context keeps the default.
                ff_logging::log_debug!(
                    "[keys] skipping malformed keymaps file {}: {e}",
                    path.display()
                );
                continue;
            }
        };
        let (map, _warnings) = KeyMap::from_toml_table(&table, ctx_name);
        resolver.set_context_map(ctx_name.to_string(), map);
    }
}
