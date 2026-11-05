//! First-run write-hook tests (CR-CH-116, FEAT-002).
//!
//! These drive the ONE first-run hook through real `WorkbenchShell` construction
//! (`make_shell` / `make_shell_first_run`), proving the write-through config
//! model's startup behaviour: absent built-in menu files and the FFCMD X -> EXIT
//! alias seed are written on first run, a pre-existing or corrupt file is left
//! alone, and the subsequent POM load reads the written file.

use super::tests_common::{make_shell, make_shell_first_run};
use crate::menu_workspace::serialiser::serialise_complete;

// Validates: startup-and-session Requirement 23.1, configuration-system
// Requirement 23.2 -- constructing the shell with an empty isolated user dir
// writes pom/settings/scrm (under menus_dir) and the FFCMD alias seed, through
// the dir seams, during construction.
#[test]
fn construction_writes_builtin_menus_and_alias_seed() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let menus = dir.path().join("menus");
    let _shell = make_shell_first_run(&menus);

    assert!(
        menus.join("pom.toml").exists(),
        "pom.toml written on first run"
    );
    assert!(
        menus.join("settings.toml").exists(),
        "settings.toml written on first run"
    );
    assert!(
        menus.join("scrm.toml").exists(),
        "scrm.toml written on first run"
    );
    // The alias seed goes to the shared isolated alias base; after ANY shell
    // construction the active-locale FFCMD.toml exists and carries X -> EXIT.
    let base = super::user_aliases::user_alias_override_base().expect("alias base");
    let ffcmd = super::user_aliases::user_alias_override_file(&base, "en", "FFCMD");
    assert!(ffcmd.exists(), "aliases/en/FFCMD.toml written on first run");
    let rows = super::user_aliases::parse_user_alias_toml(
        &std::fs::read_to_string(&ffcmd).expect("read seed"),
    )
    .expect("seed parses");
    assert!(
        rows.iter()
            .any(|r| r.surface == "X" && r.canonical == "EXIT"),
        "the FFCMD seed carries the X -> EXIT row: {rows:?}"
    );
}

// Validates: startup-and-session Requirement 23.1, configuration-system
// Requirement 23.2 -- the first-run write is COMPLETE (every flag explicit) and
// the subsequent POM load reads the written file.
#[test]
fn first_run_pom_file_is_complete_and_loads() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let menus = dir.path().join("menus");
    let _shell = make_shell_first_run(&menus);

    let text = std::fs::read_to_string(menus.join("pom.toml")).expect("read pom");
    assert!(
        text.contains("show_calendar = true"),
        "first-run POM is a Complete_Config_File with explicit show_calendar: {text}"
    );
    // The written file loads back to the Recovery_Source POM content.
    let loaded =
        crate::menu_workspace::loader::load_menu_file(&menus.join("pom.toml")).expect("load pom");
    let recovery = crate::menu_workspace::defaults::recovery_pom_menu();
    assert_eq!(
        loaded, recovery,
        "first-run POM equals the Recovery_Source POM"
    );
}

// Validates: configuration-system Requirement 23.2 (per-file idempotent),
// startup-and-session Requirement 23.1 -- a pre-existing user pom.toml is NOT
// overwritten by the first-run write.
#[test]
fn first_run_does_not_overwrite_existing_pom() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let menus = dir.path().join("menus");
    std::fs::create_dir_all(&menus).expect("mkdir menus");
    // A valid but custom user POM already on disk.
    let custom = crate::menu_workspace::defaults::recovery_settings_menu();
    let custom_text = serialise_complete(&custom);
    std::fs::write(menus.join("pom.toml"), &custom_text).expect("seed pom");

    let _shell = make_shell_first_run(&menus);

    let after = std::fs::read_to_string(menus.join("pom.toml")).expect("read pom");
    assert_eq!(
        after, custom_text,
        "a pre-existing pom.toml must be left byte-unchanged on first run"
    );
    // The absent settings.toml WAS written (per-file detection).
    assert!(menus.join("settings.toml").exists());
}

// Validates: configuration-system Requirement 23.7 -- a present-but-corrupt
// pom.toml is NOT treated as absent and NOT overwritten (recovery fallback
// happens at load time, not by clobbering the user's file).
#[test]
fn first_run_leaves_corrupt_pom_untouched() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let menus = dir.path().join("menus");
    std::fs::create_dir_all(&menus).expect("mkdir menus");
    std::fs::write(menus.join("pom.toml"), b"not valid toml [[[").expect("seed corrupt");

    let _shell = make_shell_first_run(&menus);

    let after = std::fs::read_to_string(menus.join("pom.toml")).expect("read pom");
    assert_eq!(
        after, "not valid toml [[[",
        "a corrupt present pom.toml must not be overwritten or treated as absent"
    );
}

// Validates: startup-and-session Requirement 23.1 -- the standard test shell
// (make_shell) runs the first-run hook during construction, materialising the
// built-in menus into its isolated menus dir (the write-through model, which
// reverses the former code-only never-materialise invariant).
#[test]
fn make_shell_materialises_builtin_menus_on_first_run() {
    let shell = make_shell();
    let menus = shell.menus_dir();
    assert!(
        menus.join("pom.toml").exists(),
        "make_shell's first-run hook writes pom.toml into the isolated menus dir"
    );
    assert!(menus.join("settings.toml").exists());
    assert!(menus.join("scrm.toml").exists());
}
