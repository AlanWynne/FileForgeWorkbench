//! Kinds Editor ISPF-style explicit-save convention tests (B081,
//! project-master Requirement 11).
//!
//! These drive the SHARED SAVE / END / RETURN command seam
//! (`try_commands_a` -> `editable_context_kind` -> `editable_persist` /
//! `editable_discard` / `editable_reload`) against the Kinds Editor Context,
//! proving the net behaviour the convention requires:
//!   - typed SAVE persists the working Kind config and STAYS open (11.3);
//!   - END persists then EXITS, and STAYS open reporting an error when persist
//!     fails (11.4);
//!   - RETURN discards edits and EXITS without writing (11.5);
//!   - a bare edit does NOT persist (11.2);
//!   - the in-panel Save button and the typed SAVE command share ONE persist
//!     path (11.8).

#![allow(unused_imports)]
use super::tests_common::*;
use crate::kinds_editor_panel::KindsEditorAction;
use crate::tab_state::KindTag;

/// Build a shell with the Kinds Editor open on the "editor" Kind, its
/// workspace-kinds dir pointed at a fresh TempDir. Returns the shell, the
/// TempDir, and the dir path.
fn open_kinds_on_editor() -> (super::WorkbenchShell, tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let mut shell = make_shell();
    shell.dir_overrides.workspace_kinds = Some(dir.path().to_path_buf());
    shell.dispatch_command_string("KINDS");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::KindsEditor);
    shell.apply_kinds_editor_action(KindsEditorAction::SelectKind("editor".to_string()));
    let path = dir.path().to_path_buf();
    (shell, dir, path)
}

/// Read the persisted title for the editor Kind, if the file exists.
fn persisted_editor_title(kinds_dir: &std::path::Path) -> Option<String> {
    let reloaded = crate::workspace_kind::KindRegistry::load(kinds_dir);
    // The file-backed override, if present, surfaces as the effective title.
    if kinds_dir.join("editor.toml").exists() {
        Some(reloaded.effective("editor").title.clone())
    } else {
        None
    }
}

/// Edit the working Kind's title (as a field edit does).
fn edit_title(shell: &mut super::WorkbenchShell, title: &str) {
    shell
        .kinds_editor_panel
        .working
        .as_mut()
        .expect("working")
        .title = title.to_string();
}

// Validates: project-master Requirement 11.3 -- the typed SAVE command persists
// the working Kind config to its backing file AND leaves the Context open.
#[test]
fn kinds_editor_save_command_persists_and_stays_open() {
    let (mut shell, _dir, kinds_dir) = open_kinds_on_editor();
    edit_title(&mut shell, "[SAVED EDITOR]");
    shell.dispatch_command_string("SAVE");
    assert_eq!(
        persisted_editor_title(&kinds_dir).as_deref(),
        Some("[SAVED EDITOR]"),
        "typed SAVE must persist the edited title to workspace-kinds/editor.toml"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KindsEditor,
        "SAVE must leave the Kinds Editor Context OPEN"
    );
}

// Validates: project-master Requirement 11.4 -- END persists the working Kind
// config AND THEN exits the Context (END implies SAVE).
#[test]
fn kinds_editor_end_persists_then_exits() {
    let (mut shell, _dir, kinds_dir) = open_kinds_on_editor();
    edit_title(&mut shell, "[ENDED EDITOR]");
    shell.dispatch_command_string("END");
    assert_eq!(
        persisted_editor_title(&kinds_dir).as_deref(),
        Some("[ENDED EDITOR]"),
        "END must persist the edited title (END implies SAVE)"
    );
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KindsEditor,
        "END must EXIT the Kinds Editor Context after persisting"
    );
}

// Validates: project-master Requirement 11.4 -- when persistence FAILS, END
// keeps the Context OPEN and reports the error rather than exiting. The failure
// is forced by making the workspace-kinds dir path a regular FILE, so
// create_dir_all (and the subsequent write) cannot succeed.
#[test]
fn kinds_editor_end_on_persist_error_stays_open() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let kinds_as_file = dir.path().join("kinds-as-file");
    std::fs::write(&kinds_as_file, b"blocker").expect("write blocker file");
    let mut shell = make_shell();
    shell.dir_overrides.workspace_kinds = Some(kinds_as_file);
    shell.dispatch_command_string("KINDS");
    assert_eq!(shell.tabs.active_tab().kind.tag(), KindTag::KindsEditor);
    shell.apply_kinds_editor_action(KindsEditorAction::SelectKind("editor".to_string()));
    edit_title(&mut shell, "[WILL FAIL]");
    shell.dispatch_command_string("END");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KindsEditor,
        "END must STAY in the Context when persistence fails"
    );
    assert!(
        shell.kinds_editor_panel.error.is_some(),
        "a failed END persist must surface the error inline"
    );
}

// Validates: project-master Requirement 11.5 -- RETURN discards the working
// edits WITHOUT persisting and exits; the backing store is left as it was on
// entry (no file written).
#[test]
fn kinds_editor_return_discards_and_exits() {
    let (mut shell, _dir, kinds_dir) = open_kinds_on_editor();
    edit_title(&mut shell, "[DISCARD ME]");
    shell.dispatch_command_string("RETURN");
    assert!(
        !kinds_dir.join("editor.toml").exists(),
        "RETURN must NOT persist -- the backing file stays absent"
    );
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KindsEditor,
        "RETURN must exit the Kinds Editor Context"
    );
}

// Validates: project-master Requirement 11.2 -- a bare edit does NOT persist:
// the backing file is unchanged (absent) until an explicit commit.
#[test]
fn kinds_editor_bare_edit_does_not_autosave() {
    let (mut shell, _dir, kinds_dir) = open_kinds_on_editor();
    edit_title(&mut shell, "[NOT SAVED]");
    // No SAVE / END issued: the file must still be absent.
    assert!(
        !kinds_dir.join("editor.toml").exists(),
        "a bare edit must NOT be persisted (no autosave-on-edit)"
    );
}

// Validates: project-master Requirement 11.8 -- the in-panel Save affordance
// (KindsEditorAction::Save) gates persistence through the SAME path as the typed
// SAVE command; both persist the working copy identically (no divergent paths).
#[test]
fn kinds_editor_save_button_matches_save_command_path() {
    let (mut shell, _dir, kinds_dir) = open_kinds_on_editor();
    edit_title(&mut shell, "[BUTTON EDITOR]");
    // The in-panel Save button action.
    shell.apply_kinds_editor_action(KindsEditorAction::Save);
    assert_eq!(
        persisted_editor_title(&kinds_dir).as_deref(),
        Some("[BUTTON EDITOR]"),
        "the in-panel Save button must persist via the shared persist path"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::KindsEditor,
        "the Save button leaves the Context open, like the SAVE command"
    );
}
