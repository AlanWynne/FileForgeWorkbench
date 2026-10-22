//! Startup bootstrap for the localization message catalogue (CR-NR-103).
//!
//! This is the FIRST production caller of [`ff_i18n::set_catalogue`]: until now
//! the Catalogue_Lookup_Seam was installed nowhere, so every `t()` lookup fell
//! back to the key string. At startup [`install_catalogue`] resolves the shipped
//! `i18n/` tree, loads the English Identity_Base (plus a non-English overlay when
//! `ui.locale` selects one), and installs it on the process-wide seam. It is
//! re-invoked by the `ui.locale` hot-reload callback so a locale change swaps the
//! active catalogue with no new mechanism (localization Req 1.4, 1.5, 3.4).
//!
//! No new config or reload path is introduced: the root is resolved by the same
//! exe-dir search-path pattern the help loader uses (construct.rs), and the
//! reload wiring goes through the existing `ff_config::on_locale_reload`. Every
//! failure degrades gracefully -- a missing root leaves the seam uninstalled
//! (key fallback), and a bad overlay keeps the English base (localization Req
//! 2.4).

use std::path::PathBuf;

use ff_i18n::{Catalogue, Locale};

/// Resolve the shipped `i18n/` root directory, returning the first candidate
/// that exists on disk, or `None` when none exist.
///
/// Mirrors the help-registry search-path pattern in `construct.rs`. Candidates,
/// in priority order:
/// 1. exe-dir `i18n` (installed layout, next to the binary);
/// 2. exe-dir `..\..\i18n` (an alternative installed layout);
/// 3. exe-dir `..\..\..\crates\ff-i18n\i18n` (dev fallback from
///    `target/<profile>/` back to the crate's shipped catalogue);
/// 4. `dirs::data_dir()/FileForgeWorkbench/i18n` (user-data layout).
///
/// Returns `None` when no candidate directory exists; the caller then leaves the
/// seam uninstalled, degrading to the key fallback (localization Req 2.4).
pub(crate) fn resolve_i18n_root() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("i18n"));
            candidates.push(exe_dir.join("..").join("..").join("i18n"));
            candidates.push(
                exe_dir
                    .join("..")
                    .join("..")
                    .join("..")
                    .join("crates")
                    .join("ff-i18n")
                    .join("i18n"),
            );
        }
    }
    if let Some(data_dir) = dirs::data_dir() {
        candidates.push(data_dir.join("FileForgeWorkbench").join("i18n"));
    }
    first_existing_dir(&candidates)
}

/// Return the first path in `candidates` that is an existing directory, or
/// `None` when none exist. Factored out so the search is unit-testable without
/// depending on the real executable location.
fn first_existing_dir(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates.iter().find(|p| p.is_dir()).cloned()
}

/// Install the active message catalogue from the shipped `i18n/` tree, selecting
/// the locale from `ui.locale` (localization Req 1.3, 1.6, 2.4, 3.4).
///
/// Reads `ui.locale` (the schema default `"en"` applies when unset, Req 1.3),
/// resolves the `i18n/` root, loads the English Identity_Base, applies a
/// non-English overlay when selected, and installs the result on the
/// process-wide seam. Every failure is non-fatal: a missing root leaves the seam
/// uninstalled, and a bad overlay keeps the English base (Req 2.4).
pub(crate) fn install_catalogue(config: &ff_config::ConfigHandle) {
    // The schema registers ui.locale with default "en", so get_string resolves
    // to "en" when the key is unset (localization Req 1.3).
    let locale_tag = config
        .get_string(ff_config::keys::ui::LOCALE)
        .unwrap_or_else(|_| ff_i18n::IDENTITY_BASE_TAG.to_string());
    install_catalogue_for_locale(&locale_tag);
}

/// Resolve the root and install the catalogue for an explicit `locale_tag`.
///
/// Separated from [`install_catalogue`] so the install-and-resolve behaviour is
/// unit-testable against a known locale string without a live config handle.
/// When no catalogue can be built the seam is left as-is (uninstalled at
/// startup, degrading to the key fallback -- localization Req 2.4).
fn install_catalogue_for_locale(locale_tag: &str) {
    let root = match resolve_i18n_root() {
        Some(root) => root,
        None => {
            // No shipped i18n/ tree: leave the seam uninstalled so lookups
            // degrade to the key fallback (localization Req 2.4).
            ff_logging::log_warn!(
                "[i18n] no i18n/ catalogue directory found on any search path -- \
                 leaving the lookup seam uninstalled (keys will render as their fallback)"
            );
            return;
        }
    };

    if let Some(catalogue) = build_catalogue(&root, locale_tag) {
        ff_i18n::set_catalogue(catalogue);
    }
}

/// Build the active catalogue for `locale_tag` from the `root` i18n tree.
///
/// Returns the English Identity_Base for an English (or unparseable) locale, the
/// base with a non-English overlay on success, or the base alone when the
/// overlay fails (localization Req 1.6, 2.4). Returns `None` only when even the
/// English base cannot be loaded, so the caller leaves the seam untouched.
fn build_catalogue(root: &std::path::Path, locale_tag: &str) -> Option<Catalogue> {
    // English (or an unparseable tag): the Identity_Base is the active catalogue.
    let Some(locale) = parse_overlay_locale(locale_tag) else {
        return load_base(root);
    };

    // Non-English overlay. load_overlay consumes the base, so on failure we
    // reload the base to keep the English Identity_Base active (Req 2.4).
    let base = load_base(root)?;
    match base.load_overlay(root, locale.clone()) {
        Ok(with_overlay) => Some(with_overlay),
        Err(e) => {
            ff_logging::log_warn!(
                "[i18n] failed to load the \"{}\" locale overlay -- {} -- \
                 using the English Identity_Base",
                locale.tag(),
                e
            );
            load_base(root)
        }
    }
}

/// Parse `locale_tag` into the overlay locale to apply, or `None` when the
/// English Identity_Base should be used directly (English selected, or an
/// invalid tag that falls back to English -- localization Req 1.6).
fn parse_overlay_locale(locale_tag: &str) -> Option<Locale> {
    match Locale::parse(locale_tag) {
        Ok(locale) if locale.is_identity_base() => None,
        Ok(locale) => Some(locale),
        Err(e) => {
            ff_logging::log_warn!(
                "[i18n] ui.locale \"{}\" is not a valid BCP-47 tag ({}) -- \
                 using the English Identity_Base",
                locale_tag,
                e
            );
            None
        }
    }
}

/// Load the English Identity_Base from `root`, logging a WARN and returning
/// `None` on failure (the caller then leaves the seam uninstalled, Req 2.4).
fn load_base(root: &std::path::Path) -> Option<Catalogue> {
    match Catalogue::load_identity_base(root) {
        Ok(base) => Some(base),
        Err(e) => {
            ff_logging::log_warn!(
                "[i18n] failed to load the English Identity_Base from {} -- {} -- \
                 leaving the lookup seam uninstalled",
                root.display(),
                e
            );
            None
        }
    }
}

/// Install the English Identity_Base catalogue for the TEST harness, so the
/// full-shell `egui_kittest` tests see the same English labels production shows
/// (where [`install_catalogue`] runs at startup). Without this the process-wide
/// seam is uninstalled in a fresh test process (nextest runs each test in its
/// own process), so every `t()` renders its key instead of the English string
/// and a `get_by_label("Save")`-style query fails.
///
/// Resolves the shipped root via the same exe-dir search path as production
/// ([`resolve_i18n_root`], whose dev fallback reaches `crates/ff-i18n/i18n` from
/// the test binary). Safe to call repeatedly; a missing root degrades to the key
/// fallback exactly as production does.
#[cfg(test)]
pub(crate) fn install_catalogue_for_tests() {
    install_catalogue_for_locale(ff_i18n::IDENTITY_BASE_TAG);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// The crate-relative path to the shipped ff-i18n `i18n/` tree, used so the
    /// install test resolves against the real English catalogue DATA.
    fn shipped_ff_i18n_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("ff-i18n")
            .join("i18n")
    }

    // Validates: localization Requirement 2.4 -- when no i18n/ directory exists
    // on any candidate path, root resolution returns None so the seam is left
    // uninstalled (graceful degradation to the key fallback).
    #[test]
    fn first_existing_dir_returns_none_when_no_candidate_exists() {
        let candidates = vec![
            PathBuf::from("/this/path/does/not/exist/i18n"),
            PathBuf::from("/another/missing/i18n"),
        ];
        assert_eq!(first_existing_dir(&candidates), None);
    }

    // Validates: localization Requirement 2.4 -- the first existing candidate
    // directory is selected in priority order.
    #[test]
    fn first_existing_dir_returns_first_existing_candidate() {
        let root = shipped_ff_i18n_root();
        assert!(root.is_dir(), "shipped ff-i18n i18n/ root must exist");
        let candidates = vec![
            PathBuf::from("/missing/first/i18n"),
            root.clone(),
            PathBuf::from("/missing/last/i18n"),
        ];
        assert_eq!(first_existing_dir(&candidates), Some(root));
    }

    // Validates: localization Requirement 3.2, 10.2 -- installing the catalogue
    // with ui.locale=en makes the free seam ff_i18n::t resolve a shipped key to
    // its English value (behaviour-neutral English baseline).
    #[test]
    fn install_catalogue_for_en_resolves_a_shipped_key_to_english() {
        let root = shipped_ff_i18n_root();
        // Install directly from the known-present shipped root (bypassing the
        // exe-dir search, which does not point at the crate in a test binary).
        let base = Catalogue::load_identity_base(&root).expect("English base loads");
        ff_i18n::set_catalogue(base);
        assert!(ff_i18n::is_initialised());
        // A key from main.ftl and one from the new menus.ftl both resolve.
        assert_eq!(ff_i18n::t("files-panel-title"), "Files");
        assert_eq!(
            ff_i18n::t("menu-pom-title"),
            "FileForge Workbench -- Primary Option Menu"
        );
    }

    // Validates: localization Requirement 1.3 -- install_catalogue_for_locale
    // with the default "en" tag installs the catalogue (the schema default path)
    // and resolves a shipped key, confirming the end-to-end install seam works.
    #[test]
    fn install_catalogue_for_locale_en_installs_the_seam() {
        // resolve_i18n_root in a test binary resolves via the exe-dir fallback
        // (target/<profile>/deps -> ../../../crates/ff-i18n/i18n), so the real
        // shipped root is found and the seam is installed.
        install_catalogue_for_locale("en");
        assert!(ff_i18n::is_initialised());
        assert_eq!(ff_i18n::t("menu-settings-title"), "Settings");
    }
}
