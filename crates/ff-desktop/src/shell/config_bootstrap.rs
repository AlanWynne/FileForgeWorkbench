//! The Config_Rebuild_Helper (CR-CH-116, configuration-system Req 23.6).
//!
//! The write-through config model keeps the built-in menus (POM, Settings,
//! SCRM) and the FFCMD `X -> EXIT` alias seed as a RECOVERY source in code
//! (`menu_workspace::defaults`), writes COMPLETE config files on first run, and
//! runs from the on-disk files thereafter. Reset bare rebuilds those same files.
//!
//! This module owns the ONE shared write engine both paths call so their output
//! cannot diverge: [`write_builtin_defaults`]. It is a PURE free function -- the
//! caller resolves the three directory seams and passes them in -- so it is unit
//! -testable against `TempDir`s and identical for first-run and Reset bare. The
//! only difference is the [`BootstrapMode`]: `FirstRunMissingOnly` writes a file
//! only when it is absent (per-file), `RebuildOverwrite` always writes.
//!
//! The thin first-run hook ([`WorkbenchShell::first_run_write_builtin_config`])
//! resolves the seams, skips in Degraded_Mode, and calls the engine in
//! `FirstRunMissingOnly` mode. The Reset-bare rebuild (FEAT-003) calls the same
//! engine in `RebuildOverwrite` mode.

use std::path::{Path, PathBuf};

use crate::menu_workspace::{defaults, serialiser, MenuFile};

use super::environment_registry::FFCMD_NAME;
use super::user_aliases::{self, user_alias_override_file, AliasEnvironment, UserAliasRowToml};
use super::WorkbenchShell;

/// How [`write_builtin_defaults`] treats an already-present config file.
///
/// Validates: configuration-system Requirement 23.2, 23.6, 19.17
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BootstrapMode {
    /// First-run write: write a file ONLY when it does not already exist
    /// (per-file detection). A present file (even a corrupt one) is left alone.
    FirstRunMissingOnly,
    /// Reset-bare rebuild: always (re)write the file, overwriting any existing
    /// content. Reset bare has already archived the old files.
    RebuildOverwrite,
}

/// Write the COMPLETE built-in config files from the Recovery_Source through the
/// passed directory seams (CR-CH-116). Writes `pom.toml`, `settings.toml` AND
/// `scrm.toml` under `menus_dir` (each a Complete_Config_File via
/// [`serialiser::serialise_complete`]), and the single FFCMD `X -> EXIT` alias
/// seed row to `<alias_base>/<locale>/FFCMD.toml`. Parent directories are
/// created as needed.
///
/// `scrm.toml` is a MENU file read from `menus_dir()/scrm.toml` by the SCRM menu
/// open path and the Menus editor (gate: `menus/scrm.toml`), NOT the unrelated
/// `scrm_dir()` screen-collections directory. All three menus therefore share
/// the one `menus_dir`.
///
/// This is the SINGLE Config_Rebuild_Helper both the first-run write and the
/// Reset-bare rebuild call (Req 23.6), so the two cannot diverge. It is pure:
/// all paths come in as arguments, nothing is read from `self`.
///
/// # Errors
///
/// Returns the first write/serialise error as a human-readable string. Callers
/// treat this best-effort (log a WARN; the Recovery_Source still backs runtime).
///
/// Validates: configuration-system Requirement 23.2, 23.4, 23.5, 23.6, 19.17;
/// command-configurator Requirement 8.1, 8.2, 8.3; screen-snapshot-scrm Req 22.8
pub(super) fn write_builtin_defaults(
    menus_dir: &Path,
    alias_base: &Path,
    locale: &str,
    mode: BootstrapMode,
) -> Result<(), String> {
    write_menu(
        &menus_dir.join("pom.toml"),
        &defaults::recovery_pom_menu(),
        mode,
    )?;
    write_menu(
        &menus_dir.join("settings.toml"),
        &defaults::recovery_settings_menu(),
        mode,
    )?;
    write_menu(
        &menus_dir.join("scrm.toml"),
        &defaults::recovery_scrm_menu(),
        mode,
    )?;
    write_alias_seed(alias_base, locale, mode)?;
    Ok(())
}

/// Write one COMPLETE menu file, honouring the mode's write-if-absent guard.
fn write_menu(path: &Path, menu: &MenuFile, mode: BootstrapMode) -> Result<(), String> {
    if mode == BootstrapMode::FirstRunMissingOnly && path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let toml = serialiser::serialise_complete(menu);
    std::fs::write(path, toml).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Write the single `X -> EXIT` FFCMD alias seed row to the per-locale override
/// file, honouring the mode's write-if-absent guard. The row is a user-override
/// row (editable in the Alias Editor) targeting the FFCMD environment.
fn write_alias_seed(alias_base: &Path, locale: &str, mode: BootstrapMode) -> Result<(), String> {
    let path = user_alias_override_file(alias_base, locale, FFCMD_NAME);
    if mode == BootstrapMode::FirstRunMissingOnly && path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let rows = vec![UserAliasRowToml {
        surface: "X".to_string(),
        canonical: "EXIT".to_string(),
        environment: AliasEnvironment::Ffcmd,
        command_template: String::new(),
        description: String::new(),
    }];
    let text = user_aliases::serialise_user_aliases(&rows)?;
    std::fs::write(&path, text).map_err(|e| format!("write {}: {e}", path.display()))
}

// === First-run hook =========================================================

// Test-only THREAD-LOCAL override for the first-run menus dir.
//
// `make_shell`/`harness_shell` set `dir_overrides.menus` AFTER construction, but
// the first-run write runs DURING construction, so the per-shell override is not
// yet in place. A test sets this seam on its OWN thread just before building the
// shell so the construction-time first-run write targets an isolated temp dir
// rather than the developer's real User_Data_Dir. It is THREAD-LOCAL (not a
// process global) so parallel tests under `cargo test` cannot clobber each
// other's value between the set and the construction-time read. Production never
// sets it.
#[cfg(test)]
thread_local! {
    static TEST_FIRST_RUN_MENUS_DIR: std::cell::RefCell<Option<PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(super) fn set_test_first_run_menus_dir(dir: PathBuf) {
    TEST_FIRST_RUN_MENUS_DIR.with(|cell| *cell.borrow_mut() = Some(dir));
}

impl WorkbenchShell {
    /// First-run write hook (CR-CH-116, startup-and-session Req 23): on
    /// construction, write any ABSENT built-in menu file (pom/settings/scrm) and
    /// the absent FFCMD `X -> EXIT` alias seed through the dir seams, so the app
    /// then runs from the on-disk files. This is the ONE first-run hook.
    ///
    /// Skips entirely in Degraded_Mode (the User_Data_Dir cannot be resolved):
    /// the compiled Recovery_Source still backs runtime (Req 23.8). A present but
    /// unparseable file is left alone (the engine only writes when the path is
    /// absent, so a corrupt-but-present file is not treated as absent and not
    /// overwritten -- Req 23.7). Any write error is logged at WARN and is
    /// non-blocking (the Recovery_Source remains the fallback).
    ///
    /// Validates: startup-and-session Requirement 23.1-23.6; configuration-system
    /// Requirement 23.2, 23.3, 23.5, 23.7, 23.8
    pub(super) fn first_run_write_builtin_config(&mut self) {
        // Degraded_Mode: if the session layer cannot resolve a User_Data_Dir and
        // no test/menus override points elsewhere, skip the write and fall back
        // to the Recovery_Source (Req 23.8).
        if self.dir_overrides.menus.is_none()
            && self.first_run_menus_dir_override().is_none()
            && ff_session::UserDataDir::resolve(None).is_err()
        {
            ff_logging::log_warn!(
                "[config] first-run write skipped: User_Data_Dir unavailable (Degraded_Mode); using Recovery_Source"
            );
            return;
        }

        let menus_dir = self.first_run_menus_dir();
        let Some(alias_base) = self.user_alias_override_base() else {
            ff_logging::log_warn!(
                "[config] first-run write skipped: no alias override base; using Recovery_Source"
            );
            return;
        };
        let locale = self
            .config_handle
            .get_string(ff_config::keys::ui::LOCALE)
            .unwrap_or_else(|_| "en".to_string());

        if let Err(e) = write_builtin_defaults(
            &menus_dir,
            &alias_base,
            &locale,
            BootstrapMode::FirstRunMissingOnly,
        ) {
            ff_logging::log_warn!("[config] first-run write error (non-blocking): {e}");
        }
    }

    /// The menus dir the first-run write targets: the per-shell override when set
    /// (unusual at construction time), else the test seam, else `menus_dir()`.
    fn first_run_menus_dir(&self) -> PathBuf {
        if let Some(dir) = &self.dir_overrides.menus {
            return dir.clone();
        }
        if let Some(dir) = self.first_run_menus_dir_override() {
            return dir;
        }
        self.menus_dir()
    }

    /// The test-only first-run menus dir override (always `None` in production).
    fn first_run_menus_dir_override(&self) -> Option<PathBuf> {
        let _ = self;
        #[cfg(test)]
        {
            TEST_FIRST_RUN_MENUS_DIR.with(|cell| cell.borrow().clone())
        }
        #[cfg(not(test))]
        {
            None
        }
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn read(path: &Path) -> String {
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    // Validates: configuration-system Requirement 23.2, 23.6 (CR-CH-116) --
    // FirstRunMissingOnly writes pom/settings/scrm (all under menus_dir) +
    // aliases/en/FFCMD.toml when all are absent.
    #[test]
    fn first_run_missing_only_writes_all_absent_files() {
        let menus = TempDir::new().expect("menus");
        let aliases = TempDir::new().expect("aliases");
        write_builtin_defaults(
            menus.path(),
            aliases.path(),
            "en",
            BootstrapMode::FirstRunMissingOnly,
        )
        .expect("write ok");
        assert!(menus.path().join("pom.toml").exists(), "pom.toml written");
        assert!(
            menus.path().join("settings.toml").exists(),
            "settings.toml written"
        );
        assert!(
            menus.path().join("scrm.toml").exists(),
            "scrm.toml written under menus_dir"
        );
        assert!(
            aliases.path().join("en").join("FFCMD.toml").exists(),
            "aliases/en/FFCMD.toml written"
        );
    }

    // Validates: configuration-system Requirement 23.2, 23.6 -- FirstRunMissingOnly
    // leaves a PRE-EXISTING file byte-unchanged (per-file write-if-absent).
    #[test]
    fn first_run_missing_only_leaves_existing_file_untouched() {
        let menus = TempDir::new().expect("menus");
        let aliases = TempDir::new().expect("aliases");
        let pom = menus.path().join("pom.toml");
        std::fs::write(&pom, b"user custom content").expect("seed");
        write_builtin_defaults(
            menus.path(),
            aliases.path(),
            "en",
            BootstrapMode::FirstRunMissingOnly,
        )
        .expect("write ok");
        assert_eq!(
            read(&pom),
            "user custom content",
            "a pre-existing file must not be overwritten on first run"
        );
        // The other absent files were still written.
        assert!(menus.path().join("settings.toml").exists());
    }

    // Validates: configuration-system Requirement 19.17 -- RebuildOverwrite
    // replaces an existing file with the Recovery_Source content.
    #[test]
    fn rebuild_overwrite_replaces_existing_file() {
        let menus = TempDir::new().expect("menus");
        let aliases = TempDir::new().expect("aliases");
        let pom = menus.path().join("pom.toml");
        std::fs::write(&pom, b"stale content").expect("seed");
        write_builtin_defaults(
            menus.path(),
            aliases.path(),
            "en",
            BootstrapMode::RebuildOverwrite,
        )
        .expect("write ok");
        let content = read(&pom);
        assert_ne!(
            content, "stale content",
            "RebuildOverwrite replaces the file"
        );
        assert!(
            content.contains("Primary Option Menu"),
            "rebuilt pom.toml carries the Recovery_Source POM: {content}"
        );
    }

    // Validates: configuration-system Requirement 23.5, screen-snapshot-scrm
    // Req 22.8 -- scrm.toml lands under menus_dir (the SCRM menu read path), not
    // a separate screen-collections dir; every write lands under the PASSED dirs.
    #[test]
    fn scrm_menu_is_written_under_menus_dir() {
        let menus = TempDir::new().expect("menus");
        let aliases = TempDir::new().expect("aliases");
        write_builtin_defaults(
            menus.path(),
            aliases.path(),
            "en",
            BootstrapMode::FirstRunMissingOnly,
        )
        .expect("write ok");
        let scrm = menus.path().join("scrm.toml");
        assert!(scrm.exists(), "scrm.toml under menus_dir");
        // And it parses as the SCRM menu (Capture / Snapshot options present).
        let text = read(&scrm);
        assert!(
            text.contains("SNAPSHOT"),
            "scrm.toml carries the SCRM menu: {text}"
        );
    }

    // Validates: configuration-system Requirement 23.4 -- written menu files are
    // Complete_Config_File with show_calendar explicit (POM true, Settings false).
    #[test]
    fn written_menu_files_are_complete() {
        let menus = TempDir::new().expect("menus");
        let aliases = TempDir::new().expect("aliases");
        write_builtin_defaults(
            menus.path(),
            aliases.path(),
            "en",
            BootstrapMode::FirstRunMissingOnly,
        )
        .expect("write ok");
        let pom = read(&menus.path().join("pom.toml"));
        assert!(pom.contains("show_calendar = true"), "POM complete: {pom}");
        assert!(pom.contains("config_version"), "POM stamped: {pom}");
        let settings = read(&menus.path().join("settings.toml"));
        assert!(
            settings.contains("show_calendar = false"),
            "Settings complete with explicit show_calendar=false: {settings}"
        );
    }

    // Validates: command-configurator Requirement 8.1, 8.2 (CR-CH-116) --
    // aliases/<locale>/FFCMD.toml contains a parseable X -> EXIT row.
    #[test]
    fn alias_seed_contains_x_to_exit_row() {
        let menus = TempDir::new().expect("menus");
        let aliases = TempDir::new().expect("aliases");
        write_builtin_defaults(
            menus.path(),
            aliases.path(),
            "en",
            BootstrapMode::FirstRunMissingOnly,
        )
        .expect("write ok");
        let text = read(&aliases.path().join("en").join("FFCMD.toml"));
        let rows = user_aliases::parse_user_alias_toml(&text).expect("seed parses");
        assert_eq!(rows.len(), 1, "exactly the one seed row");
        assert_eq!(rows[0].surface, "X");
        assert_eq!(rows[0].canonical, "EXIT");
    }

    // Validates: command-configurator Requirement 8.2 -- the seed is written to
    // the ACTIVE locale subfolder (e.g. a non-en locale).
    #[test]
    fn alias_seed_respects_locale_subfolder() {
        let menus = TempDir::new().expect("menus");
        let aliases = TempDir::new().expect("aliases");
        write_builtin_defaults(
            menus.path(),
            aliases.path(),
            "af",
            BootstrapMode::FirstRunMissingOnly,
        )
        .expect("write ok");
        assert!(
            aliases.path().join("af").join("FFCMD.toml").exists(),
            "seed written under the active locale subfolder"
        );
        assert!(!aliases.path().join("en").join("FFCMD.toml").exists());
    }
}
