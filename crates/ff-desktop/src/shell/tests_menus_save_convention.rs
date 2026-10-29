//! Menus Editor ISPF-style explicit-save convention tests (B081,
//! project-master Requirement 11).
//!
//! These drive the SHARED SAVE / END / RETURN command seam
//! (`try_commands_a` -> `editable_context_kind` -> `editable_persist` /
//! `editable_discard` / `editable_reload`) against the Menus Editor Context,
//! proving the net behaviour the convention requires:
//!   - typed SAVE persists the working menu and STAYS open (11.3);
//!   - END persists then EXITS, and STAYS open reporting an error when persist
//!     fails (11.4);
//!   - RETURN discards edits and EXITS without writing (11.5);
//!   - a bare edit does NOT persist (11.2);
//!   - the in-panel Save button and the typed SAVE command share ONE persist
//!     path (11.8).

#![allow(unused_imports)]
use super::tests_common::*;
use crate::menus_editor_panel::MenusEditorAction;
use crate::tab_state::KindTag;

/// Load the persisted POM menu title from the editor's menus dir, if the file
/// exists yet.
fn persisted_pom_title(dir: &tempfile::TempDir) -> Option<String> {
    let path = dir.path().join("menus").join("pom.toml");
    let menu = crate::menu_workspace::loader::load_menu_file(&path).ok()?;
    Some(menu.title)
}

// Validates: project-master Requirement 11.3 -- the typed SAVE command persists
// the working menu to its backing file AND leaves the Context open.
#[test]
fn menus_editor_save_command_persists_and_stays_open() {
    let (mut shell, dir) = make_shell_with_menus_editor();
    shell.menus_editor_panel.working.as_mut().unwrap().title = "Saved POM".to_string();
    shell.dispatch_command_string("SAVE");
    assert_eq!(
        persisted_pom_title(&dir).as_deref(),
        Some("Saved POM"),
        "typed SAVE must persist the edited menu title to the backing file"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::MenusEditor,
        "SAVE must leave the Menus Editor Context OPEN"
    );
    assert!(
        shell.menus_editor_panel.error.is_none(),
        "a successful SAVE clears any inline error"
    );
}

// Validates: project-master Requirement 11.4 -- END persists the working menu
// AND THEN exits the Context (END implies SAVE).
#[test]
fn menus_editor_end_persists_then_exits() {
    let (mut shell, dir) = make_shell_with_menus_editor();
    shell.menus_editor_panel.working.as_mut().unwrap().title = "Ended POM".to_string();
    shell.dispatch_command_string("END");
    assert_eq!(
        persisted_pom_title(&dir).as_deref(),
        Some("Ended POM"),
        "END must persist the edited menu title (END implies SAVE)"
    );
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::MenusEditor,
        "END must EXIT the Menus Editor Context after persisting"
    );
}

// Validates: project-master Requirement 11.4 -- when persistence FAILS
// (validation rejects an empty command), END keeps the Context OPEN and reports
// the error rather than exiting; nothing is written.
#[test]
fn menus_editor_end_on_persist_error_stays_open() {
    let (mut shell, dir) = make_shell_with_menus_editor();
    // Blank out an option's command -> invalid (the loader would reject it).
    shell.menus_editor_panel.working.as_mut().unwrap().options[0].command = "   ".to_string();
    shell.dispatch_command_string("END");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::MenusEditor,
        "END must STAY in the Context when persistence fails"
    );
    assert!(
        shell.menus_editor_panel.error.is_some(),
        "a failed END persist must surface the error inline"
    );
    assert!(
        !dir.path().join("menus").join("pom.toml").exists(),
        "a failed persist must NOT write the backing file"
    );
}

// Validates: project-master Requirement 11.5 -- RETURN discards the working
// edits WITHOUT persisting and exits; the backing store is left as it was on
// entry (no file written).
#[test]
fn menus_editor_return_discards_and_exits() {
    let (mut shell, dir) = make_shell_with_menus_editor();
    shell.menus_editor_panel.working.as_mut().unwrap().title = "Discard me".to_string();
    shell.dispatch_command_string("RETURN");
    assert!(
        !dir.path().join("menus").join("pom.toml").exists(),
        "RETURN must NOT persist -- the backing file stays absent"
    );
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::MenusEditor,
        "RETURN must exit the Menus Editor Context"
    );
}

// Validates: project-master Requirement 11.2 -- a bare edit does NOT persist:
// the backing file is unchanged (absent) until an explicit commit.
#[test]
fn menus_editor_bare_edit_does_not_autosave() {
    let (mut shell, dir) = make_shell_with_menus_editor();
    shell.menus_editor_panel.working.as_mut().unwrap().title = "Not saved".to_string();
    // No SAVE / END issued: the file must still be absent.
    assert!(
        !dir.path().join("menus").join("pom.toml").exists(),
        "a bare edit must NOT be persisted (no autosave-on-edit)"
    );
}

// Validates: project-master Requirement 11.8 -- the in-panel Save affordance
// (MenusEditorAction::Save) gates persistence through the SAME path as the typed
// SAVE command; both persist the working copy identically (no divergent paths).
#[test]
fn menus_editor_save_button_matches_save_command_path() {
    let (mut shell, dir) = make_shell_with_menus_editor();
    shell.menus_editor_panel.working.as_mut().unwrap().title = "Button POM".to_string();
    // The in-panel Save button action.
    shell.apply_menus_editor_action(MenusEditorAction::Save);
    assert_eq!(
        persisted_pom_title(&dir).as_deref(),
        Some("Button POM"),
        "the in-panel Save button must persist via the shared persist path"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::MenusEditor,
        "the Save button leaves the Context open, like the SAVE command"
    );
}
