//! # Per-frame theme file hot-reload
//!
//! The file-backed active-theme tracking extracted verbatim from `update.rs`
//! (rust-standards.md 400-line rule). Behaviour is unchanged.

use super::WorkbenchShell;

impl WorkbenchShell {
    /// File-backed active theme + hot-reload (CR-NR-074 Req 19.6). Resolve the
    /// active theme file, and reload the palette when the file changes on disk
    /// or when the configured active theme changes. This keeps the palette
    /// driven by the theme file (not just the compiled mode default).
    pub(super) fn reload_active_theme_file(&mut self) {
        let themes_dir = self.themes_dir();
        // Determine the active theme's file path: prefer theme.active_name,
        // else the built-in for the current theme.active mode.
        let active_name = self
            .config_handle
            .get_string(ff_config::keys::theme::ACTIVE_NAME)
            .unwrap_or_default();
        let active_name = active_name.trim();
        let theme_path = if !active_name.is_empty() {
            Some(themes_dir.join(format!(
                "{}.toml",
                crate::theme_defaults::theme_slug(active_name)
            )))
        } else {
            self.config_handle
                .get_string(ff_config::keys::theme::ACTIVE)
                .ok()
                .and_then(|m| ff_theme::mode::VisualMode::from_str_loose(&m))
                .map(|mode| {
                    let name = ff_theme::defaults::default_palette_for_mode(mode).name;
                    themes_dir.join(format!("{}.toml", crate::theme_defaults::theme_slug(&name)))
                })
        };

        if let Some(path) = theme_path {
            let disk_mtime = std::fs::metadata(&path)
                .ok()
                .and_then(|m| m.modified().ok());
            let tracked = self.active_theme_file.as_ref();
            let path_changed = tracked.map(|(p, _)| p != &path).unwrap_or(true);
            let mtime_changed = match (tracked, disk_mtime) {
                (Some((p, t)), Some(dm)) => p == &path && *t != dm,
                _ => false,
            };
            if path_changed || mtime_changed {
                // Reload from the resolver so a missing/invalid file falls
                // back to Default Legacy (Req 19.5) without crashing.
                self.palette = crate::theme_defaults::resolve_startup_palette(
                    &self.config_handle,
                    &themes_dir,
                );
                self.active_theme_file = disk_mtime.map(|dm| (path, dm));
            }
        }
    }
}
