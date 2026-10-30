//! # Command Configurator save-convention seam (B081, project-master
//! Requirement 11 -- the Command Configurator fix).
//!
//! The Command Configurator's implementation of the shared CR-CH-071
//! editable-Context SEAM (`editable_persist` / `editable_discard` /
//! `editable_reload` in `editable_context.rs`). Mirrors the Config / Settings
//! fix (`commands_config_save.rs`) and the Menus / Keys / Kinds seams.
//!
//! The Command Configurator was the LAST non-compliant editable Context: every
//! add / edit / delete used to write `commands.toml` IMMEDIATELY (the autosave
//! model the convention forbids). It now owns a WHOLE-STORE WORKING COPY in
//! `CommandConfiguratorState` (`working: Option<Vec<CommandDefinition>>`): add /
//! edit / DELETE all mutate that buffer only, and the three seam operations
//! below flush / drop / re-snapshot it. The backing `CommandStore` load/save
//! semantics are UNCHANGED -- the working copy lives entirely in the panel.
//!
//! Net behaviour (the convention):
//!   - a bare add/edit/delete does NOT write the store (Req 11.2 -- no autosave);
//!   - SAVE flushes the working copy and STAYS open, buffer == persisted after
//!     (11.3);
//!   - END flushes then EXITS, and STAYS open with an error on flush failure
//!     (11.4);
//!   - RETURN drops the working copy and EXITS, store unchanged from entry
//!     (11.5);
//!   - a deferred DELETE is committed only on SAVE / END, never on the click
//!     (11.2).

use super::WorkbenchShell;

impl WorkbenchShell {
    /// The Command Configurator's implementation of the CR-CH-071
    /// editable-Context SEAM PERSIST operation (project-master Req 11.3, 11.4):
    /// flush the panel's whole-store working copy to `commands.toml` in ONE
    /// commit, returning ok/err so the caller's END can branch save-then-leave
    /// vs stay-with-error. The staged add / edit / DELETE edits (which never
    /// touched the store) are applied atomically here: the working definitions
    /// REPLACE the store's definitions, then the store is written.
    ///
    /// On a write failure the working copy is RETAINED (so RETURN can still
    /// discard it and a retry is possible), the error is surfaced inline, and
    /// `Err` is returned so END stays open (Req 11.4). On success the working
    /// copy is cleared (buffer == persisted, Req 11.3).
    ///
    /// Validates: project-master Requirement 11.3, 11.4
    pub(super) fn command_configurator_persist(&mut self) -> Result<(), String> {
        let Some(working) = self.command_configurator_panel.working.clone() else {
            // Nothing staged -> nothing to persist; a clean no-op.
            self.command_configurator_panel.persist_error = None;
            return Ok(());
        };
        // Apply the working copy to the store, then write it. Keep the previous
        // definitions so a write failure can be rolled back cleanly (the store's
        // in-memory state must not diverge from disk on failure).
        let previous = std::mem::replace(&mut self.command_store.definitions, working);
        match self.command_store.save() {
            Ok(()) => {
                // Flushed: the working copy now equals the persisted store.
                self.command_configurator_panel.working = None;
                self.command_configurator_panel.persist_error = None;
                Ok(())
            }
            Err(e) => {
                // Roll the store's in-memory definitions back to the last-saved
                // state and KEEP the working copy so the user can retry / RETURN.
                self.command_store.definitions = previous;
                let msg = format!("Save failed: {e}");
                self.command_configurator_panel.persist_error = Some(msg.clone());
                Err(msg)
            }
        }
    }

    /// The Command Configurator's SEAM DISCARD operation (project-master
    /// Req 11.5): drop the working copy WITHOUT writing the backing store, so
    /// RETURN reverts every pending add / edit / delete and the store is left
    /// exactly as it was on entry. No edit ever leaked to the store before this
    /// point (add/edit/delete only mutate the working copy), so dropping it is a
    /// complete revert.
    ///
    /// Validates: project-master Requirement 11.5
    pub(super) fn command_configurator_discard(&mut self) {
        self.command_configurator_panel.clear_working();
        self.command_configurator_panel.error = None;
    }

    /// The Command Configurator's SEAM RELOAD operation (project-master Req 11 --
    /// REFRESH): drop the working copy and re-read the persisted store from disk,
    /// so REFRESH shows the last-saved definitions (picking up any out-of-band
    /// change) with no pending edits.
    ///
    /// Validates: project-master Requirement 11.1
    pub(super) fn command_configurator_reload(&mut self) {
        self.command_configurator_panel.clear_working();
        self.command_store.poll_reload();
    }
}
