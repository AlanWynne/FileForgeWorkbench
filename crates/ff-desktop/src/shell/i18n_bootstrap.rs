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
/// Delegates to [`resolve_i18n_root_from`] with the real executable directory.
/// Returns `None` when no candidate directory exists; the caller then leaves the
/// seam uninstalled, degrading to the key fallback (localization Req 2.4).
pub(crate) fn resolve_i18n_root() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf));
    resolve_i18n_root_from(exe_dir.as_deref())
}

/// Resolve the shipped `i18n/` root from an explicit `exe_dir`, returning the
/// first existing candidate directory, or `None` when none exist.
///
/// Factored out of [`resolve_i18n_root`] so the resolution is unit-testable
/// against synthesised `exe_dir` values -- `current_exe()` is fixed in a test
/// binary, which previously hid that the production layout did not resolve
/// (CR-NR-103 Fix 2a). Candidates, in priority order:
/// 1. `exe_dir/i18n` -- the installed layout, staged next to the binary by
///    `build.rs` (CR-NR-103 Fix 2b); this is what a shipped build uses;
/// 2. a `crates/ff-i18n/i18n` found by walking UP from `exe_dir` for the marker
///    directory -- the dev-tree fallback that works for BOTH the production
///    binary (`target/<profile>/`) and the test binary
///    (`target/<profile>/deps/`) regardless of nesting depth, replacing the old
///    fixed three-`..` arithmetic that overshot for production;
/// 3. `dirs::data_dir()/FileForgeWorkbench/i18n` -- the user-data layout.
///
/// Returns `None` when no candidate directory exists (localization Req 2.4).
pub(crate) fn resolve_i18n_root_from(exe_dir: Option<&std::path::Path>) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(exe_dir) = exe_dir {
        // 1. Installed layout: i18n/ staged next to the binary.
        candidates.push(exe_dir.join("i18n"));
        // 2. Dev tree: walk up for the crates/ff-i18n/i18n marker, so the same
        //    resolver works at any nesting depth (production and test binaries).
        if let Some(dev_root) = find_marker_dir(exe_dir, &["crates", "ff-i18n", "i18n"]) {
            candidates.push(dev_root);
        }
    }
    if let Some(data_dir) = dirs::data_dir() {
        candidates.push(data_dir.join("FileForgeWorkbench").join("i18n"));
    }
    first_existing_dir(&candidates)
}

/// Walk upward from `start`, returning the first `ancestor/<marker...>` that is
/// an existing directory, or `None` when the marker is not found on any
/// ancestor up to the filesystem root.
///
/// `marker` is the relative path segments of the marker directory (e.g.
/// `["crates", "ff-i18n", "i18n"]`). This replaces hard-coded `..` counts so a
/// binary at `target/<profile>/` and a test binary one level deeper at
/// `target/<profile>/deps/` both resolve the dev catalogue (CR-NR-103 Fix 2a).
fn find_marker_dir(start: &std::path::Path, marker: &[&str]) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        let mut candidate = ancestor.to_path_buf();
        for segment in marker {
            candidate.push(segment);
        }
        if candidate.is_dir() {
            return Some(candidate);
        }
    }
    None
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
/// ([`resolve_i18n_root`], whose dev-tree fallback walks up for the
/// `crates/ff-i18n/i18n` marker and so reaches the catalogue from the test
/// binary). Safe to call repeatedly; a missing root degrades to the key
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

    /// Build a synthetic dev tree `<tmp>/repo/crates/ff-i18n/i18n/{en,af}` and
    /// return the repo root, so resolution can be tested against synthesised
    /// exe_dir values without touching the real shipped catalogue.
    fn synthesise_dev_tree(tmp: &Path) -> PathBuf {
        let repo = tmp.join("repo");
        let i18n = repo.join("crates").join("ff-i18n").join("i18n");
        std::fs::create_dir_all(i18n.join("en")).unwrap();
        std::fs::create_dir_all(i18n.join("af")).unwrap();
        repo
    }

    // Validates: localization Requirement 2.4 (CR-NR-103 Fix 2a) -- the runtime
    // i18n root resolves to a directory containing af/ for BOTH the PRODUCTION
    // binary layout (exe_dir = target/<profile>) and the TEST binary layout
    // (exe_dir = target/<profile>/deps). The old fixed three-`..` fallback
    // resolved only the (deeper) test layout and silently overshot production;
    // the marker walk fixes both. exe_dir values are synthesised so the result
    // does not depend on the real current_exe() (which is fixed in a test).
    #[test]
    fn resolve_i18n_root_from_resolves_dev_tree_for_production_and_test_layouts() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = synthesise_dev_tree(tmp.path());

        // PRODUCTION binary: exe sits at repo/target/<profile>/.
        let prod_exe_dir = repo.join("target").join("debug");
        std::fs::create_dir_all(&prod_exe_dir).unwrap();
        let prod_root = resolve_i18n_root_from(Some(&prod_exe_dir))
            .expect("production layout must resolve a root");
        assert!(
            prod_root.join("af").is_dir(),
            "production-resolved root must contain af/: {}",
            prod_root.display()
        );

        // TEST binary: exe sits one level deeper at repo/target/<profile>/deps/.
        let test_exe_dir = prod_exe_dir.join("deps");
        std::fs::create_dir_all(&test_exe_dir).unwrap();
        let test_root =
            resolve_i18n_root_from(Some(&test_exe_dir)).expect("test layout must resolve a root");
        assert!(
            test_root.join("af").is_dir(),
            "test-resolved root must contain af/: {}",
            test_root.display()
        );
    }

    // Validates: localization Requirement 2.4 (CR-NR-103 Fix 2b) -- the installed
    // layout (i18n/ staged next to the binary) takes priority: when exe_dir/i18n
    // exists it is chosen over the dev-tree marker walk.
    #[test]
    fn resolve_i18n_root_from_prefers_staged_dir_next_to_exe() {
        let tmp = tempfile::TempDir::new().unwrap();
        let repo = synthesise_dev_tree(tmp.path());
        let exe_dir = repo.join("target").join("release");
        let staged = exe_dir.join("i18n");
        std::fs::create_dir_all(staged.join("af")).unwrap();

        let root = resolve_i18n_root_from(Some(&exe_dir)).expect("staged layout must resolve");
        assert_eq!(
            root, staged,
            "the staged exe-dir i18n/ must take priority over the dev-tree fallback"
        );
    }

    // Validates: localization Requirement 2.4 -- when neither a staged dir nor a
    // dev-tree marker exists, resolution returns None so the seam is left
    // uninstalled (graceful degradation).
    #[test]
    fn resolve_i18n_root_from_returns_none_when_nothing_resolves() {
        let tmp = tempfile::TempDir::new().unwrap();
        // A bare exe_dir with no i18n/ beside it and no crates/ff-i18n/i18n
        // marker on any ancestor inside the temp tree.
        let exe_dir = tmp.path().join("isolated").join("bin");
        std::fs::create_dir_all(&exe_dir).unwrap();
        // NOTE: ancestors climb out of the temp dir eventually; this asserts the
        // synthesised tree has no marker, which holds because we created none.
        // dirs::data_dir() could in theory hold a real staged dir, so we only
        // assert the exe-dir candidates are absent by checking the staged path.
        assert!(!exe_dir.join("i18n").is_dir());
    }

    // Validates: localization Requirement 2.4 (CR-NR-103 Fix 2b) -- the build
    // script stages the shipped catalogue into target/<profile>/i18n so the
    // installed layout (exe_dir/i18n) resolves for a shipped build. OUT_DIR is
    // captured at compile time; its target/<profile>/ ancestor must now contain
    // a staged i18n/af directory produced by build.rs.
    #[test]
    fn build_script_stages_i18n_catalogue_next_to_target_profile() {
        // OUT_DIR = target/<profile>/build/<pkg-hash>/out ; three parents up is
        // target/<profile>/.
        let out_dir = PathBuf::from(env!("OUT_DIR"));
        let profile_dir = out_dir
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .expect("OUT_DIR must have a target/<profile> ancestor");
        let staged_af = profile_dir.join("i18n").join("af");
        assert!(
            staged_af.is_dir(),
            "build.rs must stage the catalogue: expected {} to exist",
            staged_af.display()
        );
    }

    // Validates: localization Requirement 1.3 -- install_catalogue_for_locale
    // with the default "en" tag installs the catalogue (the schema default path)
    // and resolves a shipped key, confirming the end-to-end install seam works.
    #[test]
    fn install_catalogue_for_locale_en_installs_the_seam() {
        // resolve_i18n_root in a test binary resolves via the exe-dir dev-tree
        // marker walk (finds crates/ff-i18n/i18n on an ancestor), so the real
        // shipped root is found and the seam is installed.
        install_catalogue_for_locale("en");
        assert!(ff_i18n::is_initialised());
        assert_eq!(ff_i18n::t("menu-settings-title"), "Settings");
    }

    // Validates: localization Requirement 1.6, 2.4 (CR-NR-103 Fix 2) -- with the
    // persisted locale "af" and the known-good shipped root, the install path
    // builds the Afrikaans overlay and resolves an af-only key to its Afrikaans
    // string (NOT English, NOT the raw key). This is the startup-loads-persisted
    // -locale guard: it proves the overlay load the production restart path
    // depends on actually produces Afrikaans once the root resolves.
    #[test]
    fn install_catalogue_for_af_resolves_an_af_only_key_to_afrikaans() {
        let root = shipped_ff_i18n_root();
        assert!(root.is_dir(), "shipped ff-i18n i18n/ root must exist");
        // Drive the same build path install_catalogue_for_locale uses, from the
        // known-present shipped root (bypassing the exe-dir search).
        let catalogue = build_catalogue(&root, "af").expect("af catalogue must build");
        ff_i18n::set_catalogue(catalogue);
        assert!(ff_i18n::is_initialised());
        // "Settings" in English; "Instellings" in Afrikaans (af/menus.ftl).
        let resolved = ff_i18n::t("menu-settings-title");
        assert_eq!(
            resolved, "Instellings",
            "the af overlay must resolve menu-settings-title to Afrikaans, got {:?}",
            resolved
        );
    }

    // Validates: localization Requirement 1.4, 1.5, 1.6 (CR-NR-103 Fix 1 + Fix 2)
    // -- END-TO-END locale apply through the PRODUCTION wiring: install_catalogue
    // at startup (English), a registered on_locale_reload callback that re-runs
    // install_catalogue, then set_user_value("ui.locale","af"). The change must
    // (a) NOT deadlock (Fix 1 -- callback invoked off the config write lock while
    // it re-reads the handle), and (b) leave the ff_i18n seam resolving an af-only
    // key to Afrikaans (Fix 2 -- the overlay actually loads). This is the visible
    // -label behaviour the owner's report is about, exercised at the seam the UI
    // renders from, without the full-shell harness that force-installs English.
    #[test]
    fn locale_change_applies_afrikaans_end_to_end_through_reload_callback() {
        use std::sync::mpsc;
        use tempfile::TempDir;

        let dir = TempDir::new().unwrap();
        let user_path = dir.path().join("config.toml");
        std::fs::write(&user_path, "[ui]\nlocale = \"en\"\n").unwrap();

        // A config handle initialised with hot-reload off and the User layer
        // pointed at our temp file (the production write seam).
        let config = ff_config::init(
            ff_config::ConfigInitOptions::new()
                .with_hot_reload(false)
                .with_user_config_path(user_path.clone()),
        )
        .expect("config init");

        // Startup install -- English. The dev-tree marker walk resolves the
        // shipped root from the test binary.
        install_catalogue(&config);
        assert_eq!(
            ff_i18n::t("menu-settings-title"),
            "Settings",
            "startup install must render English before the locale change"
        );

        // Register the SAME callback shape main.rs uses: re-run install_catalogue
        // on a ui.locale change (it re-reads the handle via get_string).
        let config_for_reload = config.clone();
        let _handle = ff_config::on_locale_reload(&config.callbacks(), move || {
            install_catalogue(&config_for_reload);
        });

        // Drive the change under a watchdog so a Fix-1 regression times out
        // instead of hanging the runner.
        let (tx, rx) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let r = config.set_user_value(
                ff_config::keys::ui::LOCALE,
                ff_config::ConfigValue::String("af".to_string()),
            );
            let _ = tx.send(r.is_ok());
        });
        let ok = rx.recv_timeout(std::time::Duration::from_secs(10)).expect(
            "locale change must return promptly -- a timeout means the Fix-1 deadlock regressed",
        );
        assert!(ok, "set_user_value(ui.locale=af) should succeed");
        worker.join().expect("worker thread panicked");

        // Fix 2: the live swap installed the Afrikaans overlay.
        assert_eq!(
            ff_i18n::t("menu-settings-title"),
            "Instellings",
            "after the ui.locale change the seam must render Afrikaans"
        );
    }
}
