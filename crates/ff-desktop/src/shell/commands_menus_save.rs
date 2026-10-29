//! # Menus Editor save-convention seam (B081, project-master Requirement 11)
//!
//! The Menus Editor's implementation of the shared CR-CH-071 editable-Context
//! SEAM (`editable_persist` / `editable_discard` / `editable_reload` in
//! `editable_context.rs`). Split out of `menus_editor.rs` to keep the seam ops
//! grouped with the sibling editors and mirror the Theme Editor fix
//! (`commands_theme_save.rs`), so no file breaches the 400-line rule
//! (rust-standards.md).
//!
//! These methods make the Menus Editor satisfy the ISPF-style explicit-save
//! convention: typed SAVE persists and stays, END persists then exits (and
//! stays open on a persist error), RETURN discards the working menu, and
//! REFRESH re-opens the working copy. The in-panel Save button and the typed
//! SAVE command share the ONE persist path (`save_working_menu`), so a visible
//! SAVE affordance gates persistence identically to the SAVE command (Req 11.8
//! -- no divergent save paths).

use super::WorkbenchShell;

impl WorkbenchShell {
    /// The Menus Editor's implementation of the CR-CH-071 editable-Context SEAM
    /// PERSIST operation (project-master Req 11.3, 11.4, 11.8): commit the working
    /// menu to its backing file, returning ok/err so the caller's END can branch
    /// save-then-leave vs stay-with-error. THE single persist path shared by the
    /// in-panel Save button (`A::Save`) and the typed SAVE / END commands.
    ///
    /// Validates: project-master Requirement 11.3, 11.4, 11.8
    pub(super) fn menus_editor_persist(&mut self) -> Result<(), String> {
        let Some(name) = self.menus_editor_panel.selected.clone() else {
            // Nothing selected -> nothing to persist; a clean no-op.
            return Ok(());
        };
        self.save_working_menu(&name)
    }

    /// The Menus Editor's SEAM DISCARD operation (project-master Req 11.5): drop
    /// the unsaved working edits WITHOUT writing the backing store, re-opening
    /// the PERSISTED menu so the Context is left clean (no partially-applied
    /// state) and re-entry reloads cleanly.
    ///
    /// Validates: project-master Requirement 11.5
    pub(super) fn menus_editor_discard(&mut self) {
        if let Some(name) = self.menus_editor_panel.selected.clone() {
            let menu = self.load_menu_for_editor(&name);
            self.menus_editor_panel.load_working(&name, menu);
        } else {
            self.menus_editor_panel.working = None;
        }
        self.menus_editor_panel.error = None;
    }

    /// The Menus Editor's SEAM RELOAD operation (project-master Req 11 -- REFRESH):
    /// re-open the working copy from the PERSISTED menu, dropping any unsaved
    /// edits. Identical effect to discard for this Context (no live preview to
    /// revert).
    ///
    /// Validates: project-master Requirement 11.1
    pub(super) fn menus_editor_reload(&mut self) {
        self.menus_editor_discard();
    }
}
