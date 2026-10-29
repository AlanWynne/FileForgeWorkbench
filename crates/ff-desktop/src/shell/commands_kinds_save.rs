//! # Kinds Editor save-convention seam (B081, project-master Requirement 11)
//!
//! The Kinds Editor's implementation of the shared CR-CH-071 editable-Context
//! SEAM (`editable_persist` / `editable_discard` / `editable_reload` in
//! `editable_context.rs`). Mirrors the Theme Editor fix
//! (`commands_theme_save.rs`) and the Menus/Keys seams.
//!
//! These methods make the Kinds Editor satisfy the ISPF-style explicit-save
//! convention: typed SAVE persists and stays, END persists then exits (and
//! stays open on a persist error), RETURN discards the working Kind config, and
//! REFRESH re-opens the working copy from the registry. The in-panel Save button
//! and the typed SAVE command share the ONE persist path (`save_selected_kind`),
//! so a visible SAVE affordance gates persistence identically to the SAVE
//! command (Req 11.8 -- no divergent save paths).

use super::WorkbenchShell;

impl WorkbenchShell {
    /// The Kinds Editor's implementation of the CR-CH-071 editable-Context SEAM
    /// PERSIST operation (project-master Req 11.3, 11.4, 11.8): commit the working
    /// Kind config to `workspace-kinds/<name>.toml` + reload the registry,
    /// returning ok/err so the caller's END can branch save-then-leave vs
    /// stay-with-error. THE single persist path shared by the in-panel Save button
    /// (`A::Save`) and the typed SAVE / END commands.
    ///
    /// Validates: project-master Requirement 11.3, 11.4, 11.8
    pub(super) fn kinds_editor_persist(&mut self) -> Result<(), String> {
        if self.kinds_editor_panel.working.is_none() {
            // Nothing loaded -> nothing to persist; a clean no-op.
            return Ok(());
        }
        self.save_selected_kind()
    }

    /// The Kinds Editor's SEAM DISCARD operation (project-master Req 11.5): drop
    /// the unsaved working edits WITHOUT writing the backing store, re-opening the
    /// PERSISTED effective config for the selected Kind so the Context is left
    /// clean (no partially-applied state) and re-entry reloads cleanly.
    ///
    /// Validates: project-master Requirement 11.5
    pub(super) fn kinds_editor_discard(&mut self) {
        if let Some(name) = self.kinds_editor_panel.selected.clone() {
            let cfg = self.kind_registry.effective(&name).clone();
            self.kinds_editor_panel.load_kind(&name, cfg);
        } else {
            self.kinds_editor_panel.working = None;
        }
        self.kinds_editor_panel.error = None;
    }

    /// The Kinds Editor's SEAM RELOAD operation (project-master Req 11 -- REFRESH):
    /// re-open the working copy from the registry's persisted effective config,
    /// dropping any unsaved edits. Identical effect to discard for this Context
    /// (no live preview to revert).
    ///
    /// Validates: project-master Requirement 11.1
    pub(super) fn kinds_editor_reload(&mut self) {
        self.kinds_editor_discard();
    }
}
