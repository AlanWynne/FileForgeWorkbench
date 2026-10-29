//! # Theme Editor save-convention seam (B081, project-master Requirement 11)
//!
//! The Theme Editor's implementation of the shared CR-CH-071 editable-Context
//! SEAM (`editable_persist` / `editable_discard` / `editable_reload` in
//! `editable_context.rs`). Split out of `commands_theme.rs` to keep that file
//! under the 400-line rule (rust-standards.md); pure code movement plus the new
//! seam methods -- no behaviour change to the existing Theme Editor commands.
//!
//! These methods make the Theme Editor satisfy the ISPF-style explicit-save
//! convention: typed SAVE persists and stays, END persists then exits (and
//! stays open on a persist error), RETURN discards and reverts the live-preview
//! palette to the persisted theme, and REFRESH reloads the working copy. The
//! in-panel Save button and the typed SAVE command share the ONE persist path
//! here (Req 11.8 -- no divergent save paths).

use super::WorkbenchShell;

impl WorkbenchShell {
    /// The Theme Editor's implementation of the CR-CH-071 editable-Context SEAM
    /// PERSIST operation (project-master Req 11.3, 11.4, 11.8): commit the working
    /// copy to its backing theme file, returning ok/err so the caller's END can
    /// branch save-then-leave vs stay-with-error. This is the SINGLE persist path
    /// shared by the in-panel Save button (`A::Save`) and the typed SAVE / END
    /// commands, so a visible SAVE affordance gates persistence identically to the
    /// SAVE command (Req 11.8 -- no divergent save paths).
    ///
    /// A built-in theme is read-only and code-only (CR-CH-019), so Save redirects
    /// to Save As using the pre-filled New-name buffer; an empty buffer is a
    /// guided error. On ANY failure the error is BOTH surfaced inline AND returned
    /// as the `Err`, and the backing file is left UNCHANGED.
    ///
    /// Validates: project-master Requirement 11.3, 11.4, 11.8
    pub(super) fn theme_editor_persist(&mut self) -> Result<(), String> {
        let (Some(name), Some(palette)) = (
            self.theme_editor_panel.selected.clone(),
            self.theme_editor_panel.working.clone(),
        ) else {
            // Nothing loaded -> nothing to persist; a clean no-op.
            return Ok(());
        };
        // CR-CH-019: a built-in cannot be overwritten -- Save becomes Save As to
        // the pre-filled new name (B081: name_buffer is pre-filled on load).
        if ff_theme::is_builtin_theme(&name) {
            let new_name = self.theme_editor_panel.name_buffer.trim().to_string();
            if new_name.is_empty() {
                let msg = format!(
                    "'{name}' is a built-in theme and cannot be overwritten. Enter a new name and use Save As (or Copy) to keep your changes."
                );
                self.theme_editor_panel.error = Some(msg.clone());
                return Err(msg);
            }
            return self.save_theme_as(&new_name);
        }
        match self.write_theme_file(&name, &palette) {
            Ok(()) => {
                self.theme_editor_panel.error = None;
                Ok(())
            }
            Err(e) => {
                self.theme_editor_panel.error = Some(e.clone());
                Err(e)
            }
        }
    }

    /// Save the working copy under a NEW name (the Save As path), returning
    /// ok/err. Shared by `theme_editor_persist` (built-in redirect) and the
    /// `A::SaveAs` action so there is one Save As behaviour.
    ///
    /// Validates: project-master Requirement 11.3, 11.8
    pub(super) fn save_theme_as(&mut self, new_name: &str) -> Result<(), String> {
        let Some(mut palette) = self.theme_editor_panel.working.clone() else {
            return Ok(());
        };
        palette.name = new_name.to_string();
        match self.write_theme_file(new_name, &palette) {
            Ok(()) => {
                self.theme_editor_panel.name_buffer.clear();
                self.refresh_theme_editor_list();
                self.theme_editor_panel.load_working(new_name, palette);
                self.theme_editor_panel.error = None;
                Ok(())
            }
            Err(e) => {
                self.theme_editor_panel.error = Some(e.clone());
                Err(e)
            }
        }
    }

    /// The Theme Editor's SEAM DISCARD operation (project-master Req 11.5, 11.6):
    /// drop the unsaved working edits WITHOUT writing the backing store, AND
    /// revert the live-preview palette back to the PERSISTED theme, so RETURN
    /// cleanly cancels the preview (B081: the preview must not double as
    /// persistence). After discard the running palette equals the persisted
    /// theme, not the edited preview.
    ///
    /// Validates: project-master Requirement 11.5, 11.6
    pub(super) fn theme_editor_discard(&mut self) {
        let themes_dir = self.themes_dir();
        if let Some(name) = self.theme_editor_panel.selected.clone() {
            // Revert the live preview: re-read the PERSISTED palette for the
            // selected theme (built-in compiled palette or the on-disk user
            // file) and make it the running palette again.
            if let Some(persisted) = crate::theme_defaults::load_theme_by_name(&name, &themes_dir) {
                self.palette = persisted;
            }
        }
        // Drop the working copy + inline error so the Context is clean.
        self.theme_editor_panel.working = None;
        self.theme_editor_panel.error = None;
    }

    /// The Theme Editor's SEAM RELOAD operation (project-master Req 11 -- REFRESH):
    /// re-open the working copy from the PERSISTED theme, dropping any unsaved
    /// edits, mirroring the other editors' REFRESH.
    ///
    /// Validates: project-master Requirement 11.1
    pub(super) fn theme_editor_reload(&mut self) {
        let themes_dir = self.themes_dir();
        if let Some(name) = self.theme_editor_panel.selected.clone() {
            if let Some(persisted) = crate::theme_defaults::load_theme_by_name(&name, &themes_dir) {
                // Revert the running palette to the persisted theme and re-seed
                // the working copy from it.
                self.palette = persisted.clone();
                self.theme_editor_panel.load_working(&name, persisted);
            }
        }
    }
}
