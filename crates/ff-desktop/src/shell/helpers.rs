//! # Shell Helper Functions
//!
//! Pure free functions used by the WorkbenchShell — no shell state access.

use eframe::egui;
use ff_keys::FunctionKey;
use ff_theme::ColourRGBA;

use crate::tab_state::TabKind;

pub(super) fn config_value_to_toml_value(v: ff_config::ConfigValue) -> Option<toml::Value> {
    use ff_config::ConfigValue;
    match v {
        ConfigValue::String(s) => Some(toml::Value::String(s)),
        ConfigValue::Integer(i) => Some(toml::Value::Integer(i)),
        ConfigValue::Float(f) => Some(toml::Value::Float(f)),
        ConfigValue::Boolean(b) => Some(toml::Value::Boolean(b)),
        ConfigValue::Array(arr) => {
            let items: Vec<toml::Value> = arr
                .into_iter()
                .filter_map(config_value_to_toml_value)
                .collect();
            Some(toml::Value::Array(items))
        }
        ConfigValue::Table(t) => {
            let mut map = toml::map::Map::new();
            for (k, val) in t {
                if let Some(tv) = config_value_to_toml_value(val) {
                    map.insert(k, tv);
                }
            }
            Some(toml::Value::Table(map))
        }
        // ConfigValue is #[non_exhaustive] — future variants are silently skipped.
        _ => None,
    }
}

/// Map a `TabKind` to its context name for key map resolution.
///
/// Validates: Requirement 14.6
pub(super) fn context_name_for_kind(kind: TabKind) -> Option<&'static str> {
    match kind {
        TabKind::PrimaryOptionMenu => Some("pom"),
        TabKind::FileEditor | TabKind::Untitled => Some("editor"),
        TabKind::SettingsPanel => Some("settings"),
        TabKind::FilesPanel => Some("files"),
        TabKind::FileExplorerPanel => Some("files"),
    }
}

/// Convert a `ColourRGBA` to an `egui::Color32`.
#[inline]
pub(super) fn to_egui_color(c: ColourRGBA) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a)
}

/// Build the default global key map used at startup.
///
/// Provides ISPF-standard bindings: F3=END, F7=UP, F8=DOWN, F12=RETRIEVE.
/// Map a `FunctionKey` to the corresponding `egui::Key`, if supported.
///
/// egui exposes F1–F20; F21–F24 are not available on most platforms.
pub(super) fn egui_fkey(fk: FunctionKey) -> Option<egui::Key> {
    match fk {
        FunctionKey::F1 => Some(egui::Key::F1),
        FunctionKey::F2 => Some(egui::Key::F2),
        FunctionKey::F3 => Some(egui::Key::F3),
        FunctionKey::F4 => Some(egui::Key::F4),
        FunctionKey::F5 => Some(egui::Key::F5),
        FunctionKey::F6 => Some(egui::Key::F6),
        FunctionKey::F7 => Some(egui::Key::F7),
        FunctionKey::F8 => Some(egui::Key::F8),
        FunctionKey::F9 => Some(egui::Key::F9),
        FunctionKey::F10 => Some(egui::Key::F10),
        FunctionKey::F11 => Some(egui::Key::F11),
        FunctionKey::F12 => Some(egui::Key::F12),
        FunctionKey::F13 => Some(egui::Key::F13),
        FunctionKey::F14 => Some(egui::Key::F14),
        FunctionKey::F15 => Some(egui::Key::F15),
        FunctionKey::F16 => Some(egui::Key::F16),
        FunctionKey::F17 => Some(egui::Key::F17),
        FunctionKey::F18 => Some(egui::Key::F18),
        FunctionKey::F19 => Some(egui::Key::F19),
        FunctionKey::F20 => Some(egui::Key::F20),
        // F21–F24 not available in egui
        FunctionKey::F21 | FunctionKey::F22 | FunctionKey::F23 | FunctionKey::F24 => None,
    }
}

/// Parse an optional u64 from a trimmed string (empty → None, valid int → Some).
pub(super) fn parse_optional_u64(s: &str) -> Option<u64> {
    if s.is_empty() {
        None
    } else {
        s.parse().ok()
    }
}

/// Which shell to use when opening a containing folder.
pub(super) enum FolderOpenMode {
    Explorer,
    Cmd,
    PowerShell,
    Terminal,
}

/// Open the folder containing `file_path` in the requested shell.
///
/// Validates: Requirement 14.23–14.26
pub(super) fn open_containing_folder(file_path: &str, mode: FolderOpenMode) {
    let folder = std::path::Path::new(file_path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string());
    let _ = match mode {
        FolderOpenMode::Explorer => std::process::Command::new("explorer").arg(&folder).spawn(),
        FolderOpenMode::Cmd => std::process::Command::new("cmd")
            .args(["/k", "cd", "/d", &folder])
            .spawn(),
        FolderOpenMode::PowerShell => std::process::Command::new("powershell")
            .args(["-NoExit", "-Command", &format!("Set-Location '{folder}'")])
            .spawn(),
        FolderOpenMode::Terminal => {
            // Windows Terminal if available, else cmd fallback.
            std::process::Command::new("wt")
                .args(["--startingDirectory", &folder])
                .spawn()
                .or_else(|_| {
                    std::process::Command::new("cmd")
                        .args(["/k", "cd", "/d", &folder])
                        .spawn()
                })
        }
    };
}

/// Strip a trailing ` ALL` suffix (case-insensitive) from an EXCLUDE argument.
/// Returns `(text_without_all, had_all_flag)`.
pub(super) fn strip_all_suffix(s: &str) -> (&str, bool) {
    let upper = s.to_uppercase();
    if upper.ends_with(" ALL") {
        (s[..s.len() - 4].trim_end(), true)
    } else {
        (s, false)
    }
}

/// Map an operation message to `open_error`: None for success/info, Some for errors.
pub(super) fn info_or_error(msg: &str) -> Option<String> {
    // Messages that indicate no change or an error get surfaced; pure counts are info.
    if msg.is_empty()
        || msg.contains("line(s) excluded")
        || msg.contains("line(s) shown")
        || msg.contains("RESET:")
    {
        None
    } else {
        Some(msg.to_string())
    }
}

/// Parse two single-quoted or bare-word arguments from a CHANGE command tail.
/// Handles: `'old text' 'new text'`, `old new`, `'old' new`, `old 'new'`.
pub(super) fn parse_two_args(s: &str) -> Option<(String, String)> {
    let s = s.trim();
    let (first, rest) = extract_arg(s)?;
    let (second, _) = extract_arg(rest.trim())?;
    Some((first, second))
}

/// Extract one argument (single-quoted or bare word) from the front of `s`.
/// Returns `(arg, remainder)`.
pub(super) fn extract_arg(s: &str) -> Option<(String, &str)> {
    let s = s.trim_start();
    if s.is_empty() {
        return None;
    }
    if let Some(inner) = s.strip_prefix('\'') {
        // Single-quoted: find the closing quote
        let close = inner.find('\'')?;
        Some((inner[..close].to_string(), &inner[close + 1..]))
    } else {
        // Bare word: up to next whitespace
        let pos = s.find(char::is_whitespace).unwrap_or(s.len());
        Some((s[..pos].to_string(), &s[pos..]))
    }
}
