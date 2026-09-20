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
    // CR-NR-090 B.4b: user Workspace Kind files are archived alongside the other
    // config so RESET BARE returns to the compiled built-in Kind defaults.
    "workspace-kinds",
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

/// Display name used for the DEFAULT_PROFILE in RESET BARE target lists and the
/// confirmation dialog (CR-NR-083). The default profile has no `profiles/<slug>/`
/// directory; its User_Data_Dir is the base itself.
pub(super) const DEFAULT_PROFILE_DISPLAY: &str = "(default)";

/// A resolved RESET BARE target: the list of profiles to archive/reset and
/// whether the running process's Active_Profile is among them (CR-NR-083).
///
/// Every command form (bare, one-or-more named, ALL) resolves to one of these,
/// so the confirmation dialog and the execute path handle them uniformly
/// (Requirement 19.9-19.15). `profiles` is `(display_name, user_data_dir)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResetBareTarget {
    /// The profiles that will be archived and reset, in display order.
    pub profiles: Vec<(String, PathBuf)>,
    /// True when the running process's Active_Profile is in `profiles`, so the
    /// live in-memory state must also be reset (Requirement 19.11, 19.13).
    pub includes_active: bool,
}

impl ResetBareTarget {
    /// A single-profile target (bare RESET BARE or a one-name list).
    pub fn single(display: String, udd: PathBuf, includes_active: bool) -> Self {
        Self {
            profiles: vec![(display, udd)],
            includes_active,
        }
    }
}

/// Build a named profile's User_Data_Dir path under a profiles root, using the
/// same slug rule as `ff_session::profile_slug` (CR-NR-083, Requirement 19.10).
pub(super) fn profile_udd_path(profiles_root: &Path, name: &str) -> PathBuf {
    profiles_root.join(ff_session::profile_slug(name))
}

/// Enumerate every Application_Profile under a config base: the DEFAULT_PROFILE
/// (the base itself) plus each `profiles/<slug>/` child directory. Returns
/// `(display_name, user_data_dir)` pairs; the default is first and named
/// profiles follow in sorted slug order for a stable dialog listing.
///
/// This takes the base as an argument (rather than calling `ff_session`) so it
/// is unit-testable against a `TempDir` layout (Requirement 19.13).
///
/// Validates: configuration-system Requirement 19.13
pub(super) fn enumerate_profiles_under(base: &Path) -> Vec<(String, PathBuf)> {
    let mut out: Vec<(String, PathBuf)> =
        vec![(DEFAULT_PROFILE_DISPLAY.to_string(), base.to_path_buf())];

    let profiles_root = base.join("profiles");
    if let Ok(entries) = std::fs::read_dir(&profiles_root) {
        let mut named: Vec<(String, PathBuf)> = entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| (e.file_name().to_string_lossy().to_string(), e.path()))
            .collect();
        named.sort_by(|a, b| a.0.cmp(&b.0));
        out.extend(named);
    }
    out
}

/// Pure resolution of a RESET BARE argument string into a target profile list,
/// against an explicit config `base` and `active_slug` (CR-NR-083, Requirement
/// 19.9-19.14). Separated from the shell method so it is fully unit-testable
/// against a `TempDir` layout. `args` is the case-preserving text after
/// `RESET BARE`. See `resolve_reset_bare_target` for the semantics.
///
/// Validates: configuration-system Requirement 19.9, 19.10, 19.12, 19.13, 19.14
pub(super) fn resolve_reset_bare_target_under(
    base: &Path,
    active_slug: Option<&str>,
    args: &str,
) -> Result<ResetBareTarget, String> {
    let profiles_root = base.join("profiles");

    // Bare RESET BARE -> the running process's profile only (19.9).
    if args.is_empty() {
        let (display, udd) = match active_slug {
            Some(slug) => (slug.to_string(), profile_udd_path(&profiles_root, slug)),
            None => (DEFAULT_PROFILE_DISPLAY.to_string(), base.to_path_buf()),
        };
        return Ok(ResetBareTarget::single(display, udd, true));
    }

    // `ALL` as the SOLE argument -> every profile (19.13, 19.14).
    if args.eq_ignore_ascii_case("ALL") {
        let profiles = enumerate_profiles_under(base);
        // The active profile is always within the ALL set.
        return Ok(ResetBareTarget {
            profiles,
            includes_active: true,
        });
    }

    // Otherwise: a whitespace-separated list of named profiles (19.10). Slug
    // each, de-duplicate preserving first-seen display order, and verify each
    // exists (19.12, all-or-nothing).
    let mut seen: Vec<String> = Vec::new();
    let mut profiles: Vec<(String, PathBuf)> = Vec::new();
    let mut unknown: Vec<String> = Vec::new();
    let mut includes_active = false;
    for raw in args.split_whitespace() {
        let slug = ff_session::profile_slug(raw);
        if slug.is_empty() || seen.contains(&slug) {
            continue;
        }
        seen.push(slug.clone());
        let udd = profile_udd_path(&profiles_root, &slug);
        if !udd.is_dir() {
            unknown.push(raw.to_string());
            continue;
        }
        if active_slug == Some(slug.as_str()) {
            includes_active = true;
        }
        profiles.push((slug, udd));
    }

    if !unknown.is_empty() {
        return Err(format!(
            "RESET BARE: unknown profile(s): {}. No profile was reset.",
            unknown.join(", ")
        ));
    }
    if profiles.is_empty() {
        return Err("RESET BARE: no valid profile named.".to_string());
    }
    Ok(ResetBareTarget {
        profiles,
        includes_active,
    })
}

impl WorkbenchShell {
    /// Resolve the RESET BARE argument string into a target profile list
    /// (CR-NR-083, Requirement 19.9-19.14). `args` is the case-preserving text
    /// after `RESET BARE`:
    /// - empty          -> the running process's profile only (19.9).
    /// - `ALL` (sole arg, case-insensitive) -> every profile (19.13, 19.14).
    /// - one or more names -> each named profile, slug-matched + de-duplicated
    ///   (19.10); any unknown name makes the WHOLE command fail (19.12).
    ///
    /// Returns `Err(message)` when a named profile does not exist (all-or-
    /// nothing); the caller shows the message and opens NO dialog.
    ///
    /// Validates: configuration-system Requirement 19.9, 19.10, 19.12, 19.13, 19.14
    pub(super) fn resolve_reset_bare_target(&self, args: &str) -> Result<ResetBareTarget, String> {
        let base = ff_session::default_base()
            .map_err(|e| format!("could not resolve the configuration directory: {e}"))?;
        let active_slug = ff_session::active_profile().map(|n| ff_session::profile_slug(&n));
        resolve_reset_bare_target_under(&base, active_slug.as_deref(), args)
    }

    /// Execute a confirmed RESET BARE against a resolved target list: archive
    /// (move-not-delete) EACH listed profile best-effort, then -- iff the list
    /// includes the running process's Active_Profile -- reset the live in-memory
    /// state to the compiled baselines and reopen the Recovery_Baseline POM. No
    /// process relaunch is required.
    ///
    /// Validates: configuration-system Requirement 19.4, 19.5, 19.6, 19.11, 19.13
    pub(super) fn execute_reset_bare(&mut self, target: &ResetBareTarget) {
        use crate::notification::{Notification, NotificationLevel};

        // 1. Archive each targeted profile (best-effort; one failure does not
        //    abort the rest -- Req 19.5, 19.13).
        let mut archived: Vec<String> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        for (display, udd) in &target.profiles {
            match archive_config(udd) {
                Ok(path) => archived.push(format!("{display} -> {}", path.display())),
                Err(errs) => {
                    for e in errs {
                        errors.push(format!("{display}: {e}"));
                    }
                }
            }
        }

        if let Ok(mut q) = self.notification_queue.lock() {
            if !archived.is_empty() {
                q.push(Notification::new(
                    NotificationLevel::Info,
                    format!(
                        "RESET BARE: archived {} profile(s) to barebones.",
                        archived.len()
                    ),
                    Some(archived.join("; ")),
                ));
            }
            if !errors.is_empty() {
                q.push(Notification::new(
                    NotificationLevel::Warning,
                    "RESET BARE completed with some archive errors.".to_string(),
                    Some(errors.join("; ")),
                ));
            }
        }

        // 2. Reset the live in-memory state ONLY when the running process's
        //    profile was among the targets (Req 19.11); otherwise the archived
        //    profiles are other-process/on-disk only and the running shell is
        //    left untouched.
        if target.includes_active {
            self.reset_in_memory_to_baseline();
        }
    }

    /// Reset the in-memory configuration, menus, theme, catalog, and Workspace
    /// Kind state to the compiled baselines and reopen the Recovery_Baseline POM
    /// (Req 19.6; workspace-kinds Req 7).
    fn reset_in_memory_to_baseline(&mut self) {
        // Workspace Kinds -> compiled built-in defaults (CR-NR-090 B.4b). The
        // archived workspace-kinds/ files are moved aside above; the live
        // registry must also drop any loaded user Kinds so titles / menu bars /
        // key lists / profiles all recompute from the built-in defaults.
        self.kind_registry = crate::workspace_kind::KindRegistry::with_builtin_defaults();

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
        let pom_idx = self.tabs.tabs().iter().position(|t| t.is_home);
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

    // Validates: workspace-kinds Req 7 (CR-NR-090 B.4b) -- user Workspace Kind
    // files are in the archived set, so RESET BARE moves them aside and the
    // built-in Kind defaults are restored.
    #[test]
    fn archived_items_includes_workspace_kinds() {
        assert!(
            ARCHIVED_ITEMS.contains(&"workspace-kinds"),
            "RESET BARE must archive the workspace-kinds/ directory"
        );
    }

    // Validates: workspace-kinds Req 7 (CR-NR-090 B.4b) -- a user workspace-kinds
    // file present at reset time is MOVED into the archive (not left live).
    #[test]
    fn archive_config_moves_workspace_kinds_dir() {
        let dir = TempDir::new().expect("tempdir");
        let udd = dir.path();
        write_file(
            &udd.join("workspace-kinds").join("mainframe-editor.toml"),
            "name=\"mainframe-editor\"\nmodelled_on=\"editor\"\ntitle=\"[MF]\"",
        );
        let archive = archive_config(udd).expect("archive ok");
        assert!(archive
            .join("workspace-kinds")
            .join("mainframe-editor.toml")
            .exists());
        assert!(
            !udd.join("workspace-kinds").exists(),
            "workspace-kinds/ must be moved out of the live dir"
        );
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

    // Validates: configuration-system Req 19.13 (CR-NR-083) -- enumerate returns
    // the DEFAULT_PROFILE (base) first, then each profiles/<slug>/ child in
    // sorted order.
    #[test]
    fn enumerate_profiles_lists_default_first_then_named_sorted() {
        let dir = TempDir::new().expect("tempdir");
        let base = dir.path();
        std::fs::create_dir_all(base.join("profiles").join("rust")).expect("mk");
        std::fs::create_dir_all(base.join("profiles").join("ispf")).expect("mk");
        // A stray file (not a dir) under profiles/ must be ignored.
        write_file(&base.join("profiles").join("notes.txt"), "x");

        let profiles = enumerate_profiles_under(base);
        assert_eq!(profiles.len(), 3, "default + ispf + rust");
        assert_eq!(profiles[0].0, DEFAULT_PROFILE_DISPLAY);
        assert_eq!(profiles[0].1, base.to_path_buf());
        assert_eq!(profiles[1].0, "ispf");
        assert_eq!(profiles[1].1, base.join("profiles").join("ispf"));
        assert_eq!(profiles[2].0, "rust");
    }

    // Validates: configuration-system Req 19.13 -- with no profiles/ dir, only
    // the DEFAULT_PROFILE is returned.
    #[test]
    fn enumerate_profiles_with_no_profiles_dir_returns_only_default() {
        let dir = TempDir::new().expect("tempdir");
        let profiles = enumerate_profiles_under(dir.path());
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].0, DEFAULT_PROFILE_DISPLAY);
    }

    // Validates: configuration-system Req 19.10 -- profile_udd_path slugs the
    // name (case-insensitive, non-alphanumeric -> '-') under the profiles root.
    #[test]
    fn profile_udd_path_slugs_the_name() {
        let root = Path::new("/base/ffworkbench/profiles");
        assert_eq!(profile_udd_path(root, "ISPF"), root.join("ispf"));
        assert_eq!(
            profile_udd_path(root, "My Profile"),
            root.join("my-profile")
        );
    }

    /// Create a base with the given named profile dirs under `profiles/`.
    fn base_with_profiles(names: &[&str]) -> TempDir {
        let dir = TempDir::new().expect("tempdir");
        for n in names {
            std::fs::create_dir_all(dir.path().join("profiles").join(n)).expect("mk");
        }
        dir
    }

    // Validates: configuration-system Req 19.9 -- bare RESET BARE targets the
    // running process's profile only; default profile when none active.
    #[test]
    fn resolve_bare_targets_default_when_no_active_profile() {
        let dir = base_with_profiles(&[]);
        let t = resolve_reset_bare_target_under(dir.path(), None, "").expect("ok");
        assert_eq!(t.profiles.len(), 1);
        assert_eq!(t.profiles[0].0, DEFAULT_PROFILE_DISPLAY);
        assert_eq!(t.profiles[0].1, dir.path().to_path_buf());
        assert!(t.includes_active, "bare always includes the active profile");
    }

    // Validates: configuration-system Req 19.9 -- bare RESET BARE under an active
    // profile targets that profile's dir.
    #[test]
    fn resolve_bare_targets_active_profile_dir() {
        let dir = base_with_profiles(&["ispf"]);
        let t = resolve_reset_bare_target_under(dir.path(), Some("ispf"), "").expect("ok");
        assert_eq!(t.profiles.len(), 1);
        assert_eq!(t.profiles[0].0, "ispf");
        assert_eq!(t.profiles[0].1, dir.path().join("profiles").join("ispf"));
        assert!(t.includes_active);
    }

    // Validates: configuration-system Req 19.10, 19.11 -- a named subset resolves
    // each existing profile; includes_active is false when the active profile is
    // not in the list.
    #[test]
    fn resolve_named_subset_resolves_each_and_excludes_active() {
        let dir = base_with_profiles(&["ispf", "rust"]);
        // Active profile is a THIRD profile not in the list.
        let t = resolve_reset_bare_target_under(dir.path(), Some("web"), "ispf rust").expect("ok");
        assert_eq!(t.profiles.len(), 2);
        assert_eq!(t.profiles[0].0, "ispf");
        assert_eq!(t.profiles[1].0, "rust");
        assert!(
            !t.includes_active,
            "active profile not in the list -> running shell untouched"
        );
    }

    // Validates: configuration-system Req 19.11 -- when the active profile IS in
    // the named list, includes_active is true.
    #[test]
    fn resolve_named_subset_including_active_sets_flag() {
        let dir = base_with_profiles(&["ispf", "rust"]);
        let t = resolve_reset_bare_target_under(dir.path(), Some("rust"), "ispf rust").expect("ok");
        assert!(t.includes_active);
    }

    // Validates: configuration-system Req 19.10, 19.14 -- names are slug-matched
    // (case-insensitive) and de-duplicated.
    #[test]
    fn resolve_named_list_slugs_and_deduplicates() {
        let dir = base_with_profiles(&["ispf"]);
        let t = resolve_reset_bare_target_under(dir.path(), None, "ISPF ispf").expect("ok");
        assert_eq!(
            t.profiles.len(),
            1,
            "duplicate slug collapses to one target"
        );
        assert_eq!(t.profiles[0].0, "ispf");
    }

    // Validates: configuration-system Req 19.12 -- any unknown name in the list
    // makes the WHOLE command fail (all-or-nothing), naming the unknown one.
    #[test]
    fn resolve_named_list_with_unknown_errors_all_or_nothing() {
        let dir = base_with_profiles(&["ispf"]);
        let err = resolve_reset_bare_target_under(dir.path(), None, "ispf nonesuch")
            .expect_err("unknown must error");
        assert!(
            err.contains("nonesuch"),
            "error names the unknown profile: {err}"
        );
    }

    // Validates: configuration-system Req 19.13, 19.14 -- `ALL` (sole arg,
    // case-insensitive) targets every profile (default + named) and always
    // includes the active profile.
    #[test]
    fn resolve_all_targets_every_profile() {
        let dir = base_with_profiles(&["ispf", "rust"]);
        let t = resolve_reset_bare_target_under(dir.path(), Some("ispf"), "all").expect("ok");
        assert_eq!(t.profiles.len(), 3, "default + ispf + rust");
        assert_eq!(t.profiles[0].0, DEFAULT_PROFILE_DISPLAY);
        assert!(t.includes_active);
    }

    // Validates: configuration-system Req 19.14 -- `ALL` is only the keyword as
    // the SOLE argument; mixed with another name it is an ordinary (unknown)
    // profile name, so the command errors rather than resetting everything.
    #[test]
    fn resolve_all_mixed_with_a_name_is_not_the_keyword() {
        let dir = base_with_profiles(&["ispf"]);
        let err = resolve_reset_bare_target_under(dir.path(), None, "all ispf")
            .expect_err("'all ispf' treats 'all' as an unknown profile name");
        assert!(err.contains("all"), "error names 'all' as unknown: {err}");
    }
}
