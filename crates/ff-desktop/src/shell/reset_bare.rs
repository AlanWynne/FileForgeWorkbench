//! RESET BARE: archive the current configuration and reopen in barebones mode.
//!
//! CR-CH-021, configuration-system Requirement 19. The archive step is
//! non-destructive (move, not delete) so the operator can recover their old
//! configuration from `<User_Data_Dir>/config-archive/<timestamp>/`.

use std::path::{Path, PathBuf};

use super::WorkbenchShell;

/// The items archived by RESET BARE, relative to the User_Data_Dir. Each is
/// moved into the timestamped archive directory when it exists.
///
/// Validates: configuration-system Requirement 19.4
const ARCHIVED_ITEMS: &[&str] = &[
    "menus",
    "themes",
    "keymaps",
    "session.toml",
    "config.toml",
    "catalogs.toml",
];

/// Compute a filesystem-safe UTC timestamp of the form `YYYYMMDD-HHMMSS`.
fn utc_timestamp() -> String {
    // Use chrono if available in this crate; otherwise fall back to a
    // SystemTime-based epoch-seconds label. chrono is a workspace dependency
    // used elsewhere in ff-desktop (the POM calendar), so prefer it for a
    // human-readable, sortable label.
    chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string()
}

/// Archive the current configuration under `user_data_dir` into
/// `config-archive/<timestamp>/`, MOVING (not deleting) each present item.
///
/// Returns the archive directory on success. Per-item failures are collected
/// into the `Err` vector (best-effort): items that moved successfully remain in
/// the archive; the operation does not abort on the first failure. A move that
/// fails a cross-volume `rename` is retried as copy-then-remove.
///
/// Missing items are skipped without error. Previous archive directories are
/// never touched (Requirement 19.8).
///
/// Validates: configuration-system Requirement 19.4, 19.5, 19.8
pub(super) fn archive_config(user_data_dir: &Path) -> Result<PathBuf, Vec<String>> {
    let archive_root = user_data_dir.join("config-archive").join(utc_timestamp());
    let mut errors: Vec<String> = Vec::new();

    if let Err(e) = std::fs::create_dir_all(&archive_root) {
        return Err(vec![format!(
            "could not create archive directory {}: {e}",
            archive_root.display()
        )]);
    }

    for item in ARCHIVED_ITEMS {
        let src = user_data_dir.join(item);
        if !src.exists() {
            continue; // Missing items are skipped (Req 19.4).
        }
        let dst = archive_root.join(item);
        if let Err(e) = move_path(&src, &dst) {
            errors.push(format!("could not archive '{item}': {e}"));
        }
    }

    if errors.is_empty() {
        Ok(archive_root)
    } else {
        Err(errors)
    }
}

/// Move `src` to `dst`, falling back to copy-then-remove when a direct rename
/// fails (e.g. across volumes).
fn move_path(src: &Path, dst: &Path) -> Result<(), String> {
    if std::fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    // Fallback: recursive copy, then remove the source.
    copy_recursive(src, dst).map_err(|e| format!("copy failed: {e}"))?;
    if src.is_dir() {
        std::fs::remove_dir_all(src)
            .map_err(|e| format!("copied but could not remove source dir: {e}"))?;
    } else {
        std::fs::remove_file(src)
            .map_err(|e| format!("copied but could not remove source file: {e}"))?;
    }
    Ok(())
}

/// Recursively copy a file or directory tree from `src` to `dst`.
fn copy_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        std::fs::create_dir_all(dst)?;
        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let child_dst = dst.join(entry.file_name());
            copy_recursive(&entry.path(), &child_dst)?;
        }
        Ok(())
    } else {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, dst).map(|_| ())
    }
}

impl WorkbenchShell {
    /// Execute a confirmed RESET BARE: archive the current configuration, then
    /// reset in-memory state to the compiled baselines and reopen the
    /// Recovery_Baseline POM. No process relaunch is required.
    ///
    /// Validates: configuration-system Requirement 19.4, 19.5, 19.6
    pub(super) fn execute_reset_bare(&mut self) {
        use crate::notification::{Notification, NotificationLevel};

        // 1. Archive (move-not-delete) under the resolved User_Data_Dir.
        let archive_result = match ff_session::UserDataDir::resolve(None) {
            Ok(udd) => archive_config(udd.path()),
            Err(e) => Err(vec![format!("could not resolve user data dir: {e}")]),
        };

        match &archive_result {
            Ok(path) => {
                if let Ok(mut q) = self.notification_queue.lock() {
                    q.push(Notification::new(
                        NotificationLevel::Info,
                        "Configuration archived; reset to barebones.".to_string(),
                        Some(format!(
                            "Previous configuration moved to {}",
                            path.display()
                        )),
                    ));
                }
            }
            Err(errs) => {
                if let Ok(mut q) = self.notification_queue.lock() {
                    q.push(Notification::new(
                        NotificationLevel::Warning,
                        "RESET BARE completed with some archive errors.".to_string(),
                        Some(errs.join("; ")),
                    ));
                }
            }
        }

        // 2. Reset in-memory state to compiled baselines.
        self.reset_in_memory_to_baseline();
    }

    /// Reset the in-memory configuration, menus, theme, and catalog state to the
    /// compiled baselines and reopen the Recovery_Baseline POM (Req 19.6).
    fn reset_in_memory_to_baseline(&mut self) {
        use crate::tab_state::TabKind;

        // Theme -> Default Legacy baseline (the ISPF barebones aesthetic). B064:
        // clearing the archived files alone does NOT reset the theme, because the
        // running config_handle still holds the old theme keys in memory (and a
        // built-in theme name resolves with no file). So explicitly CLEAR the
        // user theme overrides for the ACTIVE profile, then select the baseline.
        // remove_user_value writes to the active profile's config.toml only, so
        // other profiles are untouched.
        let _ = self
            .config_handle
            .remove_user_value(ff_config::keys::theme::ACTIVE_NAME);
        let _ = self
            .config_handle
            .remove_user_value(ff_config::keys::theme::ACTIVE);
        let _ = self
            .config_handle
            .remove_user_value(ff_config::keys::theme::FOLLOW_OS);
        // Apply + persist the Default Legacy baseline via the shared activation
        // path (also opts out of follow_os). This lands the palette on Default
        // Legacy regardless of what was active before the reset.
        self.set_active_theme("Default Legacy");

        // Catalogs -> clear and re-seed the default Home catalog (CR-NR-004).
        self.files_panel.registry = crate::catalog_registry::CatalogRegistry::new();
        let home_path = dirs::home_dir()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        let _ =
            super::update::ensure_default_home_catalog(&mut self.files_panel.registry, home_path);
        if let Some(session) = &self.session {
            session.save_catalog_registry(&self.files_panel.registry);
        }

        // Menus -> drop any loaded POM menu so it reloads from the compiled
        // Recovery_Baseline on the next render. Reopen the Home Context (POM).
        let pom_idx = self
            .tabs
            .tabs()
            .iter()
            .position(|t| t.kind == TabKind::PrimaryOptionMenu);
        match pom_idx {
            Some(idx) => {
                if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
                    tab.menu_workspace = None; // forces reload from baseline
                }
                self.tabs.set_active(idx);
            }
            None => {
                self.tabs.insert_pom_tab(&self.runtime);
            }
        }
        self.ensure_pom_menu_loaded();
        self.open_error = None;
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        let mut f = std::fs::File::create(path).expect("create");
        f.write_all(contents.as_bytes()).expect("write");
    }

    // Validates: configuration-system Requirement 19.4, 19.5 -- archive moves
    // present items into config-archive/<timestamp>/ and removes the originals.
    #[test]
    fn archive_config_moves_present_items_and_removes_originals() {
        let dir = TempDir::new().expect("tempdir");
        let udd = dir.path();
        write_file(&udd.join("menus").join("pom.toml"), "title=\"x\"");
        write_file(&udd.join("themes").join("mine.toml"), "name=\"mine\"");
        // CR-CH-027: keymaps/ is archived alongside menus/themes.
        write_file(&udd.join("keymaps").join("editor.toml"), "F5=\"FIND\"");
        write_file(&udd.join("session.toml"), "a=1");
        write_file(&udd.join("config.toml"), "b=2");
        write_file(&udd.join("catalogs.toml"), "c=3");

        let archive = archive_config(udd).expect("archive ok");

        // Every item now lives under the archive and NOT at its original path.
        assert!(archive.join("menus").join("pom.toml").exists());
        assert!(archive.join("themes").join("mine.toml").exists());
        assert!(archive.join("keymaps").join("editor.toml").exists());
        assert!(archive.join("session.toml").exists());
        assert!(archive.join("config.toml").exists());
        assert!(archive.join("catalogs.toml").exists());

        assert!(!udd.join("menus").exists(), "menus/ must be moved");
        assert!(!udd.join("themes").exists(), "themes/ must be moved");
        assert!(!udd.join("keymaps").exists(), "keymaps/ must be moved");
        assert!(!udd.join("session.toml").exists());
        assert!(!udd.join("config.toml").exists());
        assert!(!udd.join("catalogs.toml").exists());
    }

    // Validates: configuration-system Requirement 19.4 -- missing items are
    // skipped without error.
    #[test]
    fn archive_config_skips_missing_items() {
        let dir = TempDir::new().expect("tempdir");
        let udd = dir.path();
        // Only session.toml present; everything else absent.
        write_file(&udd.join("session.toml"), "a=1");

        let archive = archive_config(udd).expect("archive ok despite missing items");
        assert!(archive.join("session.toml").exists());
        assert!(!archive.join("menus").exists());
        assert!(!archive.join("config.toml").exists());
    }

    // Validates: configuration-system Requirement 19.8 -- a second RESET BARE
    // creates a distinct archive and never touches the first.
    #[test]
    fn archive_config_does_not_prune_previous_archives() {
        let dir = TempDir::new().expect("tempdir");
        let udd = dir.path();
        write_file(&udd.join("config.toml"), "b=2");
        let first = archive_config(udd).expect("first archive");
        assert!(first.join("config.toml").exists());

        // Recreate a config, archive again; the first archive must survive.
        write_file(&udd.join("config.toml"), "b=3");
        let second = archive_config(udd).expect("second archive");
        assert!(second.join("config.toml").exists());
        assert!(
            first.join("config.toml").exists(),
            "the first archive must not be pruned by a later RESET BARE"
        );
    }

    // Validates: configuration-system Requirement 19.8 -- archive root persists
    // under config-archive/ (not deleted).
    #[test]
    fn archive_root_is_under_config_archive() {
        let dir = TempDir::new().expect("tempdir");
        let udd = dir.path();
        write_file(&udd.join("session.toml"), "a=1");
        let archive = archive_config(udd).expect("archive");
        assert!(archive.starts_with(udd.join("config-archive")));
    }
}
