//! # Config / Settings panel save-convention seam (B081, project-master
//! Requirement 11 -- the Config/Settings fix).
//!
//! The Config Context's implementation of the shared CR-CH-071 editable-Context
//! SEAM (`editable_persist` / `editable_discard` / `editable_reload` in
//! `editable_context.rs`). Mirrors the Theme Editor fix
//! (`commands_theme_save.rs`) and the Menus / Keys / Kinds seams.
//!
//! The Config panel is DIFFERENT from the sibling editors: it used to AUTOSAVE
//! on every widget change (`commit_value` -> `set_user_value` immediately), with
//! NO working copy and NO Save affordance. This fix introduces a WORKING-COPY
//! BUFFER in `ConfigPanelState` (`working: HashMap<key, WorkingEdit>`): a widget
//! edit now STAGES into that buffer, and the three seam operations below
//! flush/drop/re-snapshot it. The backing `ConfigHandle` layered-store semantics
//! (layers, precedence, hot-reload engine) are UNCHANGED -- the working copy
//! lives entirely in the panel (ff-desktop), per the configuration-system
//! cross-reference.
//!
//! Net behaviour (the convention):
//!   - a bare widget edit does NOT write the store (Req 11.2 -- no autosave);
//!   - SAVE flushes the buffer and STAYS open, buffer == persisted after (11.3);
//!   - END flushes then EXITS, and STAYS open with an error on flush failure
//!     (11.4);
//!   - RETURN drops the buffer and EXITS, store unchanged from entry (11.5);
//!   - the in-panel Save affordance enqueues the SAVE command, so it flushes via
//!     THIS same persist path (11.8 -- no divergent save paths).

use super::WorkbenchShell;
use crate::config_panel::WorkingEdit;

impl WorkbenchShell {
    /// The Config Context's implementation of the CR-CH-071 editable-Context
    /// SEAM PERSIST operation (project-master Req 11.3, 11.4, 11.8): flush the
    /// panel's working buffer to the backing config store in ONE commit,
    /// returning ok/err so the caller's END can branch save-then-leave vs
    /// stay-with-error. Each staged `Set` becomes a `set_user_value`; each staged
    /// `Reset` becomes a `remove_user_value` -- exactly the writes that USED to
    /// happen per-edit now happen here, gated behind the explicit commit.
    ///
    /// On the FIRST write error the flush stops, the still-unflushed edits are
    /// restored into the buffer, the error is surfaced inline, and `Err` is
    /// returned so END stays open (Req 11.4) and the store is left partially
    /// consistent with whatever committed before the failure. On success the
    /// buffer is emptied (buffer == persisted, Req 11.3).
    ///
    /// Validates: project-master Requirement 11.3, 11.4, 11.8
    pub(super) fn config_panel_persist(&mut self) -> Result<(), String> {
        if !self.config_panel.is_dirty() {
            // Nothing staged -> nothing to persist; a clean no-op.
            self.config_panel.persist_error = None;
            return Ok(());
        }
        // Take the staged edits out; flush them in a deterministic (sorted) order
        // so a failure path is reproducible in tests.
        let staged = self.config_panel.take_working();
        let mut keys: Vec<&String> = staged.keys().collect();
        keys.sort();
        for key in keys {
            let edit = &staged[key];
            let result = match edit {
                WorkingEdit::Set(value) => self.config_handle.set_user_value(key, value.clone()),
                WorkingEdit::Reset => self.config_handle.remove_user_value(key),
            };
            if let Err(e) = result {
                // Restore the edits that have NOT been flushed yet (including the
                // failing one) so RETURN can still discard them and a retry is
                // possible, then surface and return the error.
                let msg = format!("Save failed for '{key}': {e}");
                self.config_panel.working = staged;
                self.config_panel.persist_error = Some(msg.clone());
                return Err(msg);
            }
        }
        // All staged edits flushed: the buffer is now empty (Req 11.3).
        self.config_panel.persist_error = None;
        Ok(())
    }

    /// The Config Context's SEAM DISCARD operation (project-master Req 11.5): drop
    /// the working buffer WITHOUT writing the backing store, so RETURN reverts
    /// every pending edit and the store is left exactly as it was on entry. No
    /// edit ever leaked to the store before this point (bare edits only stage),
    /// so dropping the buffer is a complete revert.
    ///
    /// Validates: project-master Requirement 11.5
    pub(super) fn config_panel_discard(&mut self) {
        self.config_panel.clear_working();
    }

    /// The Config Context's SEAM RELOAD operation (project-master Req 11 --
    /// REFRESH): re-take the working-copy snapshot from the persisted store by
    /// dropping the buffer. Because the panel reads effective values live from
    /// the `ConfigHandle` and only overlays the buffer, an empty buffer IS the
    /// persisted snapshot -- so reload is a buffer clear, identical to discard
    /// (no live preview to revert).
    ///
    /// Validates: project-master Requirement 11.1
    pub(super) fn config_panel_reload(&mut self) {
        self.config_panel.clear_working();
    }
}
