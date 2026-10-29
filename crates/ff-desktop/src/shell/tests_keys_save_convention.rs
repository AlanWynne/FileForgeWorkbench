//! Keys Editor ISPF-style explicit-save convention tests (B081,
//! project-master Requirement 11).
//!
//! These drive the SHARED SAVE / END / RETURN command seam
//! (`try_commands_a` -> `editable_context_kind` -> `editable_persist` /
//! `editable_discard` / `editable_reload`) against the Keys Editor Context,
//! proving the net behaviour the convention requires:
//!   - typed SAVE persists the working key list and STAYS open (11.3);
//!   - END persists then EXITS, and STAYS open reporting an error when persist
//!     fails (11.4);
//!   - RETURN discards edits and EXITS without writing (11.5);
//!   - a bare edit does NOT persist (11.2);
//!   - the in-panel Save button and the typed SAVE command share ONE persist
//!     path (11.8).

#![allow(unused_imports)]
use super::tests_common::*;
use crate::keys_editor_panel::KeysEditorAction;
use crate::tab_state::KindTag;
use ff_keys::FunctionKey;

/// Build a shell with the Keys Editor open on the "editor" kind, its keymaps dir
/// pointed at a fresh TempDir. Returns the shell, the TempDir, and the keymaps
/// directory path.
fn open_keys_on_editor() -> (super::WorkbenchShell, tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let keymaps = dir.path().join("keymaps");
    let mut shell = make_shell();
    shell.dir_overrides.keymaps = Some(keymaps.clone());
    shell.dispatch_command_string("KEYS editor");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::KeysEditor);
    (shell, dir, keymaps)
}

/// Read the persisted F5-plain command for the editor kind, if the file exists.
fn persisted_f5_command(keymaps: &std::path::Path) -> Option<String> {
    let text = std::fs::read_to_string(keymaps.join("editor.toml")).ok()?;
    let table: toml::Table = toml::from_str(&text).ok()?;
    let (map, _w) = ff_keys::KeyMap::from_toml_table(&table, "editor");
    map.get_plain(FunctionKey::F5)
        .map(|b| b.command().to_string())
}

/// Set the F5-plain command in the working rows (as a field edit does).
fn edit_f5(shell: &mut super::WorkbenchShell, cmd: &str) {
    let row = shell
        .keys_editor_panel
        .rows
        .iter_mut()
        .find(|r| r.key == FunctionKey::F5)
        .expect("F5 row");
    row.commands[0] = cmd.to_string();
}

// Validates: project-master Requirement 11.3 -- the typed SAVE command persists
// the working key list to its backing file AND leaves the Context open.
#[test]
fn keys_editor_save_command_persists_and_stays_open() {
    let (mut shell, _dir, keymaps) = open_keys_on_editor();
    edit_f5(&mut shell, "FIND");
    shell.dispatch_command_string("SAVE");
    assert_eq!(
        persisted_f5_command(&keymaps).as_deref(),
        Some("FIND"),
        "typed SAVE must persist the edited binding to keymaps/editor.toml"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KeysEditor,
        "SAVE must leave the Keys Editor Context OPEN"
    );
}

// Validates: project-master Requirement 11.4 -- END persists the working key
// list AND THEN exits the Context (END implies SAVE).
#[test]
fn keys_editor_end_persists_then_exits() {
    let (mut shell, _dir, keymaps) = open_keys_on_editor();
    edit_f5(&mut shell, "CHANGE");
    shell.dispatch_command_string("END");
    assert_eq!(
        persisted_f5_command(&keymaps).as_deref(),
        Some("CHANGE"),
        "END must persist the edited binding (END implies SAVE)"
    );
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KeysEditor,
        "END must EXIT the Keys Editor Context after persisting"
    );
}

// Validates: project-master Requirement 11.4 -- when persistence FAILS, END
// keeps the Context OPEN and reports the error rather than exiting. The failure
// is forced by making the keymaps dir path a regular FILE, so create_dir_all
// (and the subsequent write) cannot succeed.
#[test]
fn keys_editor_end_on_persist_error_stays_open() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    // A regular file where the keymaps DIRECTORY should be: dir creation fails.
    let keymaps = dir.path().join("keymaps-as-file");
    std::fs::write(&keymaps, b"blocker").expect("write blocker file");
    let mut shell = make_shell();
    shell.dir_overrides.keymaps = Some(keymaps);
    shell.dispatch_command_string("KEYS editor");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::KeysEditor);
    edit_f5(&mut shell, "FIND");
    shell.dispatch_command_string("END");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KeysEditor,
        "END must STAY in the Context when persistence fails"
    );
    assert!(
        shell.keys_editor_panel.error.is_some(),
        "a failed END persist must surface the error inline"
    );
}

// Validates: project-master Requirement 11.5 -- RETURN discards the working
// edits WITHOUT persisting and exits; the backing store is left as it was on
// entry (no file written).
#[test]
fn keys_editor_return_discards_and_exits() {
    let (mut shell, _dir, keymaps) = open_keys_on_editor();
    edit_f5(&mut shell, "DISCARDME");
    shell.dispatch_command_string("RETURN");
    assert!(
        !keymaps.join("editor.toml").exists(),
        "RETURN must NOT persist -- the backing file stays absent"
    );
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KeysEditor,
        "RETURN must exit the Keys Editor Context"
    );
}

// Validates: project-master Requirement 11.2 -- a bare edit does NOT persist:
// the backing file is unchanged (absent) until an explicit commit.
#[test]
fn keys_editor_bare_edit_does_not_autosave() {
    let (mut shell, _dir, keymaps) = open_keys_on_editor();
    edit_f5(&mut shell, "NOTSAVED");
    // No SAVE / END issued: the file must still be absent.
    assert!(
        !keymaps.join("editor.toml").exists(),
        "a bare edit must NOT be persisted (no autosave-on-edit)"
    );
}

// Validates: project-master Requirement 11.8 -- the in-panel Save affordance
// (KeysEditorAction::Save) gates persistence through the SAME path as the typed
// SAVE command; both persist the working copy identically (no divergent paths).
#[test]
fn keys_editor_save_button_matches_save_command_path() {
    let (mut shell, _dir, keymaps) = open_keys_on_editor();
    edit_f5(&mut shell, "LOCATE");
    // The in-panel Save button action.
    shell.apply_keys_editor_action(KeysEditorAction::Save);
    assert_eq!(
        persisted_f5_command(&keymaps).as_deref(),
        Some("LOCATE"),
        "the in-panel Save button must persist via the shared persist path"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KeysEditor,
        "the Save button leaves the Context open, like the SAVE command"
    );
}
