//! Command Configurator ISPF-style explicit-save convention tests (B081,
//! project-master Requirement 11 -- the Command Configurator fix, the LAST
//! non-compliant editable Context).
//!
//! These drive the SHARED SAVE / END / RETURN command seam
//! (`try_commands_a` -> `editable_context_kind` -> `editable_persist` /
//! `editable_discard` / `editable_reload`) against the Command Configurator
//! Context, proving the net behaviour the convention requires once the autosave
//! model (each add/edit/delete wrote `commands.toml` immediately) is replaced by
//! a WHOLE-STORE working copy in `CommandConfiguratorState`:
//!   - a bare add/edit/delete does NOT write the backing store (11.2);
//!   - typed SAVE flushes the working copy and STAYS open (11.3);
//!   - END persists then EXITS (11.4), and STAYS open on a persist error (11.4);
//!   - RETURN discards the working copy and EXITS, store unchanged (11.5);
//!   - a DELETE is deferred to SAVE/END -- the click alone does not write (11.2).

#![allow(unused_imports)]
use super::tests_common::*;
use crate::command_config::render::ConfiguratorAction;
use crate::command_config::store::CommandStore;
use crate::command_config::CommandDefinition;
use crate::tab_state::KindTag;
use ff_command::{CommandTarget, TargetParams};

/// A Function definition with the given id (valid Command_ID form).
fn func_def(id: &str) -> CommandDefinition {
    CommandDefinition {
        id: id.to_string(),
        label: format!("Label for {id}"),
        description: None,
        category: "user".to_string(),
        target: CommandTarget::Function {
            command_id: "file.save".to_string(),
            params: TargetParams::new(),
        },
    }
}

/// Open the Command Configurator Context on a fresh shell whose `CommandStore`
/// is an isolated temp file. `seed` definitions are PERSISTED (written to the
/// temp store) so the Context starts from a known on-disk baseline. Returns the
/// shell and the TempDir (kept alive for the test).
fn open_configurator(seed: &[CommandDefinition]) -> (super::WorkbenchShell, tempfile::TempDir) {
    let mut shell = make_shell();
    let dir = point_store_at_temp(&mut shell);
    if !seed.is_empty() {
        shell.command_store.definitions = seed.to_vec();
        shell.command_store.save().expect("seed save");
    }
    shell.dispatch_command_string("COMMANDS");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::CommandConfigurator,
        "setup: COMMANDS must open the Command Configurator Context"
    );
    (shell, dir)
}

/// The ids currently PERSISTED to the backing store file (re-read from disk).
fn persisted_ids(shell: &super::WorkbenchShell) -> Vec<String> {
    let loaded = CommandStore::load(&shell.command_store.file_path);
    loaded.definitions.into_iter().map(|d| d.id).collect()
}

// Validates: project-master Requirement 11.2 -- a bare ADD mutates the working
// copy only; the backing store file is NOT written (no autosave-on-edit).
#[test]
fn configurator_bare_add_does_not_write_store() {
    let (mut shell, _dir) = open_configurator(&[]);
    // Stage an Add through the SAME action the render produces, then apply it.
    set_add_function_form(&mut shell, "build.release", "file.save");
    shell.apply_configurator_action(ConfiguratorAction::Save);
    // The working copy now has the definition...
    assert!(
        shell
            .command_configurator_panel
            .is_dirty(&shell.command_store.definitions),
        "a staged add marks the working copy dirty"
    );
    // ...but the backing store FILE is still empty (not autosaved).
    assert!(
        persisted_ids(&shell).is_empty(),
        "a bare add must NOT be persisted to commands.toml (no autosave)"
    );
}

// Validates: project-master Requirement 11.3 -- the typed SAVE command flushes
// the working copy to the backing store AND leaves the Context open; after SAVE
// the working copy equals the persisted store (no dirty state).
#[test]
fn configurator_save_command_persists_and_stays_open() {
    let (mut shell, _dir) = open_configurator(&[]);
    set_add_function_form(&mut shell, "build.release", "file.save");
    shell.apply_configurator_action(ConfiguratorAction::Save);
    shell.dispatch_command_string("SAVE");
    assert_eq!(
        persisted_ids(&shell),
        vec!["build.release".to_string()],
        "typed SAVE must flush the working copy to commands.toml"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::CommandConfigurator,
        "SAVE must leave the Command Configurator Context OPEN"
    );
    assert!(
        !shell
            .command_configurator_panel
            .is_dirty(&shell.command_store.definitions),
        "after a successful SAVE the working copy is clean (buffer == persisted)"
    );
}

// Validates: project-master Requirement 11.4 -- END persists the working copy
// AND THEN exits the Context (END implies SAVE).
#[test]
fn configurator_end_persists_then_exits() {
    let (mut shell, _dir) = open_configurator(&[]);
    set_add_function_form(&mut shell, "run.tool", "file.save");
    shell.apply_configurator_action(ConfiguratorAction::Save);
    shell.dispatch_command_string("END");
    assert_eq!(
        persisted_ids(&shell),
        vec!["run.tool".to_string()],
        "END must persist the working copy (END implies SAVE)"
    );
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::CommandConfigurator,
        "END must EXIT the Command Configurator Context after persisting"
    );
}

// Validates: project-master Requirement 11.4 -- when persistence FAILS, END
// keeps the Context OPEN and reports the error rather than exiting.
#[test]
fn configurator_end_on_persist_error_stays_open() {
    let (mut shell, _dir) = open_configurator(&[]);
    set_add_function_form(&mut shell, "build.release", "file.save");
    shell.apply_configurator_action(ConfiguratorAction::Save);
    // Force the backing-store write to fail: point the store file at a path whose
    // PARENT is a FILE, so create_dir_all / write cannot succeed.
    let bad_parent = _dir.path().join("not-a-dir");
    std::fs::write(&bad_parent, b"x").expect("make a file where a dir is needed");
    shell.command_store.file_path = bad_parent.join("commands.toml");
    shell.dispatch_command_string("END");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::CommandConfigurator,
        "END must STAY in the Context when persistence fails"
    );
    assert!(
        shell.command_configurator_panel.persist_error.is_some(),
        "a failed END persist must surface an error"
    );
    assert!(
        shell.command_configurator_panel.working.is_some(),
        "a failed persist must RETAIN the working copy for retry / RETURN"
    );
}

// Validates: project-master Requirement 11.5 -- RETURN discards the working copy
// WITHOUT persisting AND exits; the backing store is left exactly as it was on
// entry.
#[test]
fn configurator_return_discards_edits_and_exits() {
    let (mut shell, _dir) = open_configurator(&[func_def("existing.one")]);
    // Stage an add on top of the seeded store, then RETURN.
    set_add_function_form(&mut shell, "build.release", "file.save");
    shell.apply_configurator_action(ConfiguratorAction::Save);
    shell.dispatch_command_string("RETURN");
    assert_eq!(
        persisted_ids(&shell),
        vec!["existing.one".to_string()],
        "RETURN must leave the backing store exactly as it was on entry"
    );
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::CommandConfigurator,
        "RETURN must exit the Command Configurator Context"
    );
    assert!(
        shell.command_configurator_panel.working.is_none(),
        "RETURN must drop the working copy"
    );
}

// Validates: project-master Requirement 11.2 -- a DELETE is a WORKING-COPY edit
// deferred to SAVE/END: the click alone does NOT write the store, and RETURN
// after a staged delete leaves the definition on disk.
#[test]
fn configurator_delete_is_deferred_to_save() {
    let (mut shell, _dir) = open_configurator(&[func_def("keep.me"), func_def("drop.me")]);
    // Click Delete on drop.me: this stages the removal into the working copy only.
    shell.apply_configurator_action(ConfiguratorAction::Delete("drop.me".to_string()));
    assert_eq!(
        persisted_ids(&shell),
        vec!["keep.me".to_string(), "drop.me".to_string()],
        "a DELETE click must NOT write the store (deferred to SAVE/END)"
    );
    assert!(
        shell
            .command_configurator_panel
            .is_dirty(&shell.command_store.definitions),
        "a staged delete marks the working copy dirty"
    );
    // SAVE now commits the deletion.
    shell.dispatch_command_string("SAVE");
    assert_eq!(
        persisted_ids(&shell),
        vec!["keep.me".to_string()],
        "SAVE must commit the deferred delete"
    );
}

// Validates: project-master Requirement 11.5 -- a staged DELETE discarded by
// RETURN leaves the backing store untouched (the definition survives).
#[test]
fn configurator_return_discards_staged_delete() {
    let (mut shell, _dir) = open_configurator(&[func_def("keep.me"), func_def("drop.me")]);
    shell.apply_configurator_action(ConfiguratorAction::Delete("drop.me".to_string()));
    shell.dispatch_command_string("RETURN");
    assert_eq!(
        persisted_ids(&shell),
        vec!["keep.me".to_string(), "drop.me".to_string()],
        "RETURN must discard a staged delete -- both definitions survive on disk"
    );
}
