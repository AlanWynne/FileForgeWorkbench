//! # Keys Editor save-convention seam (B081, project-master Requirement 11)
//!
//! The Keys Editor's implementation of the shared CR-CH-071 editable-Context
//! SEAM (`editable_persist` / `editable_discard` / `editable_reload` in
//! `editable_context.rs`). Mirrors the Theme Editor fix
//! (`commands_theme_save.rs`) and the Menus seam (`commands_menus_save.rs`).
//!
//! These methods make the Keys Editor satisfy the ISPF-style explicit-save
//! convention: typed SAVE persists and stays, END persists then exits (and
//! stays open on a persist error), RETURN discards the working key rows, and
//! REFRESH re-opens the working copy from the persisted key map. The in-panel
//! Save button and the typed SAVE command share the ONE persist path
//! (`save_keys_for_kind`), so a visible SAVE affordance gates persistence
//! identically to the SAVE command (Req 11.8 -- no divergent save paths).

use super::WorkbenchShell;

impl WorkbenchShell {
    /// The Keys Editor's implementation of the CR-CH-071 editable-Context SEAM
    /// PERSIST operation (project-master Req 11.3, 11.4, 11.8): commit the working
    /// key rows to `keymaps/<kind>.toml`, returning ok/err so the caller's END can
    /// branch save-then-leave vs stay-with-error. THE single persist path shared
    /// by the in-panel Save button (`A::Save`) and the typed SAVE / END commands.
    ///
    /// Validates: project-master Requirement 11.3, 11.4, 11.8
    pub(super) fn keys_editor_persist(&mut self) -> Result<(), String> {
        let Some(kind) = self.keys_editor_panel.selected_kind.clone() else {
            // Nothing selected -> nothing to persist; a clean no-op.
            return Ok(());
        };
        self.save_keys_for_kind(&kind)
    }

    /// The Keys Editor's SEAM DISCARD operation (project-master Req 11.5): drop
    /// the unsaved working rows WITHOUT writing the backing store, re-opening the
    /// PERSISTED key map for the selected kind so the Context is left clean (no
    /// partially-applied state) and re-entry reloads cleanly.
    ///
    /// Validates: project-master Requirement 11.5
    pub(super) fn keys_editor_discard(&mut self) {
        if let Some(kind) = self.keys_editor_panel.selected_kind.clone() {
            let map = self.load_key_map_for_kind(&kind);
            self.keys_editor_panel.load_kind(&kind, &map);
        } else {
            self.keys_editor_panel.rows.clear();
        }
        self.keys_editor_panel.error = None;
    }

    /// The Keys Editor's SEAM RELOAD operation (project-master Req 11 -- REFRESH):
    /// re-open the working rows from the PERSISTED key map, dropping any unsaved
    /// edits. Identical effect to discard for this Context (no live preview to
    /// revert).
    ///
    /// Validates: project-master Requirement 11.1
    pub(super) fn keys_editor_reload(&mut self) {
        self.keys_editor_discard();
    }
}
