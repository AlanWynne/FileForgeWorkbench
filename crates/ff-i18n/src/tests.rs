//! Unit tests for the ff-i18n catalogue mechanism and lookup seam (CR-NR-103).
//!
//! Each test is annotated with the acceptance criterion it validates. All
//! catalogue tests build bundles from a temp `i18n/<locale>/` tree so they do
//! not depend on filesystem iteration order or shared process state; only the
//! single seam test touches the process-wide handle (deterministic because it
//! is the one test that installs a catalogue, and it does not assert absence).

use std::fs;
use std::path::{Path, PathBuf};

use pretty_assertions::assert_eq;
use tempfile::TempDir;

use crate::{Catalogue, I18nError, Locale, MessageArgs};

/// Path to the crate's shipped English Identity_Base tree (`i18n/`).
fn shipped_i18n_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("i18n")
}

/// Create a temp `i18n/<locale>/main.ftl` tree with the given contents and
/// return the i18n root plus the TempDir guard (kept alive by the caller).
fn write_locale(locale: &str, ftl: &str) -> (TempDir, PathBuf) {
    let tmp = TempDir::new().expect("create temp dir");
    let dir = tmp.path().join(locale);
    fs::create_dir_all(&dir).expect("create locale dir");
    fs::write(dir.join("main.ftl"), ftl).expect("write ftl");
    let root = tmp.path().to_path_buf();
    (tmp, root)
}

// === Locale ==========================================================

/// Validates: Requirement 2.5 -- English `en` is the always-present identity base.
#[test]
fn identity_base_locale_is_english() {
    let base = Locale::identity_base();
    assert_eq!(base.tag(), "en");
    assert!(base.is_identity_base());
}

/// Validates: Requirement 1.1 -- a `ui.locale` string parses into a Locale.
#[test]
fn locale_parses_from_a_bcp47_tag() {
    let fr = Locale::parse("fr").expect("fr parses");
    assert_eq!(fr.tag(), "fr");
    assert!(!fr.is_identity_base());
}

// === Identity_Base catalogue ========================================

/// Validates: Requirement 2.5 -- the English base loads with no other locale present.
#[test]
fn identity_base_loads_from_shipped_english_catalogue() {
    let catalogue = Catalogue::load_identity_base(&shipped_i18n_root())
        .expect("English Identity_Base must load");
    assert_eq!(catalogue.active_locale().tag(), "en");
}

/// Validates: Requirement 2.2 -- messages come from `.ftl` DATA, resolved by key.
#[test]
fn identity_base_resolves_a_shipped_key_from_ftl_data() {
    let catalogue = Catalogue::load_identity_base(&shipped_i18n_root())
        .expect("English Identity_Base must load");
    assert_eq!(catalogue.resolve("ok-button"), "OK");
    assert_eq!(catalogue.resolve("files-panel-title"), "Files");
}

/// Validates: Requirement 2.2, 2.3 -- `.ftl` files load into a bundle keyed by locale.
#[test]
fn catalogue_loads_ftl_file_into_a_bundle() {
    let (_tmp, root) = write_locale("en", "greeting = Hello");
    let catalogue = Catalogue::load_identity_base(&root).expect("temp English base must load");
    assert_eq!(catalogue.resolve("greeting"), "Hello");
}

// === Lookup seam (t / t_args) =======================================

/// Validates: Requirement 3.1 -- the seam exposes a no-argument `t`-style lookup.
#[test]
fn resolve_returns_the_message_string_for_a_present_key() {
    let (_tmp, root) = write_locale("en", "title = My Title");
    let catalogue = Catalogue::load_identity_base(&root).expect("base loads");
    assert_eq!(catalogue.resolve("title"), "My Title");
}

/// Validates: Requirement 3.1, 9.1 -- `t_args` interpolates named placeables.
#[test]
fn resolve_args_interpolates_a_named_placeable() {
    let (_tmp, root) = write_locale("en", "welcome = Welcome, { $name }.");
    let catalogue = Catalogue::load_identity_base(&root).expect("base loads");
    let args = MessageArgs::new().set("name", "Ada");
    assert_eq!(catalogue.resolve_args("welcome", &args), "Welcome, Ada.");
}

/// Validates: Requirement 9.2 -- an embedded command verb stays English (not translated).
#[test]
fn resolve_args_keeps_an_embedded_verb_as_a_non_translated_argument() {
    let (_tmp, root) = write_locale(
        "en",
        "profile-locked = Profile is locked -- use { $verb } to unlock.",
    );
    let catalogue = Catalogue::load_identity_base(&root).expect("base loads");
    let args = MessageArgs::new().set("verb", "LOCK OFF");
    assert_eq!(
        catalogue.resolve_args("profile-locked", &args),
        "Profile is locked -- use LOCK OFF to unlock."
    );
}

/// Validates: Requirement 3.2 -- a key present in the active locale returns the
/// active-locale string; a key absent in the active locale falls back to English.
#[test]
fn active_locale_overrides_base_and_missing_overlay_key_falls_back_to_english() {
    let tmp = TempDir::new().expect("temp dir");
    let root = tmp.path().to_path_buf();
    fs::create_dir_all(root.join("en")).expect("en dir");
    fs::write(
        root.join("en").join("main.ftl"),
        "shared = English Shared\nenglish-only = English Only",
    )
    .expect("write en");
    fs::create_dir_all(root.join("fr")).expect("fr dir");
    fs::write(
        root.join("fr").join("main.ftl"),
        "shared = Partage Francais",
    )
    .expect("write fr");

    let catalogue = Catalogue::load_identity_base(&root)
        .expect("base loads")
        .load_overlay(&root, Locale::parse("fr").expect("fr"))
        .expect("fr overlay loads");

    // Present in the active (fr) locale -> French string (Req 3.2 first clause).
    assert_eq!(catalogue.resolve("shared"), "Partage Francais");
    // Absent in fr, present in English base -> English fallback (Req 3.2 second clause).
    assert_eq!(catalogue.resolve("english-only"), "English Only");
}

/// Validates: Requirement 3.3 -- a key absent in both the active locale and the
/// Identity_Base returns the key itself (a stable visible fallback), never blank.
#[test]
fn missing_key_in_both_falls_back_to_the_key_itself() {
    let (_tmp, root) = write_locale("en", "present = Present");
    let catalogue = Catalogue::load_identity_base(&root).expect("base loads");
    assert_eq!(catalogue.resolve("totally-missing"), "totally-missing");
}

/// Validates: Requirement 3.4 -- the seam yields an owned String (what egui
/// widgets already accept), so a render-site literal-to-`t()` swap needs no API
/// change. Asserted structurally: the return type is `String`.
#[test]
fn resolve_returns_an_owned_string() {
    let (_tmp, root) = write_locale("en", "label = Label");
    let catalogue = Catalogue::load_identity_base(&root).expect("base loads");
    let value: String = catalogue.resolve("label");
    assert_eq!(value, "Label");
}

// === Recoverable load errors ========================================

/// Validates: Requirement 2.4 -- a `.ftl` parse error is surfaced as a
/// recoverable error (never a panic); callers retain the prior catalogue.
#[test]
fn parse_error_is_a_recoverable_error_not_a_panic() {
    // A bare term reference with no value is invalid Fluent syntax.
    let tmp = TempDir::new().expect("temp dir");
    let root = tmp.path().to_path_buf();
    fs::create_dir_all(root.join("en")).expect("en dir");
    fs::write(root.join("en").join("main.ftl"), "ok = fine\n= broken line").expect("write en base");
    fs::create_dir_all(root.join("de")).expect("de dir");
    fs::write(root.join("de").join("main.ftl"), "= this is not valid").expect("write de");

    // The English base here is itself malformed, but the point under test is
    // that building a bundle for a malformed file returns an Err, not a panic.
    let result = Catalogue::load_identity_base(&root);
    assert!(matches!(result, Err(I18nError::Parse { .. })));
}

/// Validates: Requirement 2.4 -- a missing locale directory is a recoverable
/// error (so a caller can retain the Identity_Base), not a panic.
#[test]
fn missing_overlay_directory_is_a_recoverable_error() {
    let (_tmp, root) = write_locale("en", "k = v");
    let base = Catalogue::load_identity_base(&root).expect("base loads");
    let result = base.load_overlay(&root, Locale::parse("fr").expect("fr"));
    assert!(matches!(result, Err(I18nError::CatalogueDirMissing { .. })));
}

// === Seam global handle (single global-state test) ==================

/// Validates: Requirement 3.4 -- the free `t`/`t_args` seam resolves against a
/// swappable installed catalogue with no per-call catalogue threading (Task 1.5).
#[test]
fn seam_free_functions_resolve_against_the_installed_catalogue() {
    let (_tmp, root) = write_locale("en", "seam-key = Seam Value\nseam-arg = Hi { $who }.");
    let catalogue = Catalogue::load_identity_base(&root).expect("base loads");
    crate::set_catalogue(catalogue);

    assert!(crate::is_initialised());
    assert_eq!(crate::t("seam-key"), "Seam Value");

    let args = MessageArgs::new().set("who", "there");
    assert_eq!(crate::t_args("seam-arg", &args), "Hi there.");

    // A key missing everywhere falls back to the key itself (Req 3.3) via the seam.
    assert_eq!(crate::t("no-such-key"), "no-such-key");
}
