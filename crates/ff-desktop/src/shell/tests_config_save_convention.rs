//! Config / Settings panel ISPF-style explicit-save convention tests (B081,
//! project-master Requirement 11 -- the Config/Settings fix).
//!
//! These drive the SHARED SAVE / END / RETURN command seam
//! (`try_commands_a` -> `editable_context_kind` -> `editable_persist` /
//! `editable_discard` / `editable_reload`) against the Config Context, proving
//! the net behaviour the convention requires once the autosave model is
//! replaced by a working-copy buffer in `ConfigPanelState`:
//!   - a bare widget edit does NOT write the backing config store (11.2);
//!   - typed SAVE flushes the buffered edits and STAYS open, buffer == persisted
//!     afterwards (11.3);
//!   - END persists the buffered edits then EXITS (11.4), and STAYS open
//!     reporting an error when persist fails (11.4);
//!   - RETURN discards the buffered edits and EXITS, leaving the store exactly
//!     as it was on entry (11.5);
//!   - the in-panel Save affordance shares the ONE persist path (11.8).

#![allow(unused_imports)]
use super::tests_common::*;
use crate::config_panel::WorkingEdit;
use crate::tab_state::KindTag;
use ff_config::value::ConfigValue;

const KEY: &str = "editor.tab_size";

/// Open the Config Context on a fresh shell whose user-config path is an
/// isolated temp file (see `make_shell`). Returns the shell.
fn open_config() -> super::WorkbenchShell {
    let mut shell = make_shell();
    shell.dispatch_command_string("CONFIG");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "setup: CONFIG must open the Config Context"
    );
    shell
}

/// The effective integer value of KEY straight from the persisted config store.
fn persisted_tab_size(shell: &super::WorkbenchShell) -> i64 {
    shell
        .config_handle
        .get_int(KEY)
        .expect("editor.tab_size is a registered integer key")
}

// Validates: project-master Requirement 11.2 -- a bare widget edit mutates the
// working buffer only; the backing config store is NOT written (no autosave).
#[test]
fn config_bare_edit_does_not_write_store() {
    let mut shell = open_config();
    let original = persisted_tab_size(&shell);
    assert_ne!(original, 7, "setup: edit must change the value");
    // Stage an edit the way a widget's resp.changed() now does -- into the
    // working buffer, NOT straight to the store.
    shell.config_panel.stage_set(KEY, ConfigValue::Integer(7));
    assert_eq!(
        persisted_tab_size(&shell),
        original,
        "a bare edit must NOT be persisted (no autosave-on-edit)"
    );
    assert!(
        shell.config_panel.is_dirty(),
        "a staged edit marks the working buffer dirty"
    );
}

// Validates: project-master Requirement 11.3 -- the typed SAVE command flushes
// the buffered edits to the backing store AND leaves the Context open; after
// SAVE the buffer equals the persisted state (no dirty state).
#[test]
fn config_save_command_persists_and_stays_open() {
    let mut shell = open_config();
    shell.config_panel.stage_set(KEY, ConfigValue::Integer(7));
    shell.dispatch_command_string("SAVE");
    assert_eq!(
        persisted_tab_size(&shell),
        7,
        "typed SAVE must flush the buffered edit to the backing store"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "SAVE must leave the Config Context OPEN"
    );
    assert!(
        !shell.config_panel.is_dirty(),
        "after a successful SAVE the working buffer is clean (buffer == persisted)"
    );
}

// Validates: project-master Requirement 11.4 -- END persists the buffered edits
// AND THEN exits the Context (END implies SAVE).
#[test]
fn config_end_persists_then_exits() {
    let mut shell = open_config();
    shell.config_panel.stage_set(KEY, ConfigValue::Integer(9));
    shell.dispatch_command_string("END");
    assert_eq!(
        persisted_tab_size(&shell),
        9,
        "END must persist the buffered edit (END implies SAVE)"
    );
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "END must EXIT the Config Context after persisting"
    );
}

// Validates: project-master Requirement 11.4 -- when persistence FAILS, END
// keeps the Context OPEN and reports the error rather than exiting.
#[test]
fn config_end_on_persist_error_stays_open() {
    let mut shell = open_config();
    // Force the persist to fail: lock the key so set_user_value returns
    // KeyLocked. `update_locked_keys` reads `[_locked].locked_keys` from a
    // system-layer table (Requirement 18.1).
    let mut locked_table = ff_config::value::ConfigTable::new();
    locked_table.insert(
        "locked_keys".to_string(),
        ConfigValue::Array(vec![ConfigValue::String(KEY.to_string())]),
    );
    let mut sys_values = ff_config::value::ConfigTable::new();
    sys_values.insert("_locked".to_string(), ConfigValue::Table(locked_table));
    shell.config_handle.update_locked_keys(&sys_values);
    shell.config_panel.stage_set(KEY, ConfigValue::Integer(11));
    shell.dispatch_command_string("END");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "END must STAY in the Context when persistence fails"
    );
    assert!(
        shell.config_panel.persist_error.is_some(),
        "a failed END persist must surface an error"
    );
}

// Validates: project-master Requirement 11.5 -- RETURN discards the buffered
// edits WITHOUT persisting AND exits; the backing store is left exactly as it
// was on entry.
#[test]
fn config_return_discards_buffered_edits_and_exits() {
    let mut shell = open_config();
    let original = persisted_tab_size(&shell);
    assert_ne!(original, 13, "setup: edit must change the value");
    shell.config_panel.stage_set(KEY, ConfigValue::Integer(13));
    shell.dispatch_command_string("RETURN");
    assert_eq!(
        persisted_tab_size(&shell),
        original,
        "RETURN must leave the backing store exactly as it was on entry"
    );
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "RETURN must exit the Config Context"
    );
    assert!(
        !shell.config_panel.is_dirty(),
        "RETURN must drop the working buffer"
    );
}

// Validates: project-master Requirement 11.5 -- a Reset staged in the working
// buffer is also discarded by RETURN (the backing store keeps the saved value).
#[test]
fn config_return_discards_staged_reset() {
    let mut shell = open_config();
    // First persist a user value so there is something a Reset would remove.
    shell.config_panel.stage_set(KEY, ConfigValue::Integer(6));
    shell.dispatch_command_string("SAVE");
    assert_eq!(
        persisted_tab_size(&shell),
        6,
        "setup: SAVE persisted the value"
    );
    // Re-open so a fresh working buffer is taken from the persisted state.
    shell.dispatch_command_string("CONFIG");
    // Stage a Reset (would restore the schema default 4) but then RETURN.
    shell.config_panel.stage_reset(KEY);
    shell.dispatch_command_string("RETURN");
    assert_eq!(
        persisted_tab_size(&shell),
        6,
        "RETURN must discard a staged Reset -- the persisted user value survives"
    );
}

// Validates: project-master Requirement 11.8 -- the in-panel Save affordance
// gates persistence through the SAME path as the typed SAVE command; both flush
// the working buffer identically (no divergent save paths).
#[test]
fn config_save_affordance_matches_save_command_path() {
    let mut shell = open_config();
    shell.config_panel.stage_set(KEY, ConfigValue::Integer(10));
    // The in-panel Save affordance flushes via the SAME seam persist the SAVE
    // command uses.
    let result = shell.editable_persist();
    assert!(result.is_ok(), "the Save affordance persist must succeed");
    assert_eq!(
        persisted_tab_size(&shell),
        10,
        "the in-panel Save affordance must flush the working buffer via the shared persist path"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ConfigPanel,
        "the Save affordance leaves the Context open, like the SAVE command"
    );
}

// Validates: project-master Requirement 11.1 -- on entry the Context operates on
// a working copy taken from the persisted state: with no edits, the displayed
// value equals the persisted value (nothing pending).
#[test]
fn config_working_value_defaults_to_persisted() {
    let shell = open_config();
    let persisted = ConfigValue::Integer(persisted_tab_size(&shell));
    let shown = shell.config_panel.working_value(KEY, &persisted);
    assert_eq!(
        shown, persisted,
        "with no staged edit the working copy shows the persisted value"
    );
    // A staged edit is what the user then sees.
    let mut shell = shell;
    shell.config_panel.stage_set(KEY, ConfigValue::Integer(5));
    let shown = shell.config_panel.working_value(KEY, &persisted);
    assert_eq!(
        shown,
        ConfigValue::Integer(5),
        "a staged edit is reflected in the working copy the user edits"
    );
}
