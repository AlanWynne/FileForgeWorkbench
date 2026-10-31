//! # SCRM Save-As overlay dispatch (Stream SCRM Phase 2C FEAT-002)
//!
//! The capture-STOP Save-As modal's shell-side dispatch, split out of
//! `update_dialogs.rs` to keep that file under the 400-line limit. The modal
//! render body itself lives in `crate::scrm_save_as_dialog`; this method only
//! wires the outcome to the EXISTING front-door command and the modal guard.

use eframe::egui;

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Render the capture-STOP Save-As modal when it is open and act on its
    /// outcome. On SAVE the shell dispatches the EXISTING `CAPTURE SAVE` /
    /// `CAPTURE EXPORT` verbs through the single front door (`handle_command`),
    /// introducing NO new persistence path (Req 24.4). On CANCEL / Escape /
    /// dimmed-background click nothing is written and the capture stays stopped
    /// (Req 24.5). A no-op when the modal is closed.
    ///
    /// Validates: screen-snapshot-scrm Requirement 24.1, 24.4, 24.5.
    pub(super) fn render_scrm_save_as_overlay(&mut self, ctx: &egui::Context) {
        if self.scrm_ui.save_as.is_none() {
            return;
        }
        self.modal_open = true;
        let outcome = {
            let state = self.scrm_ui.save_as.as_mut().expect("save_as is Some");
            crate::scrm_save_as_dialog::render(ctx, state)
        };
        match outcome {
            crate::scrm_save_as_dialog::Outcome::Save => {
                let command = self
                    .scrm_ui
                    .save_as
                    .take()
                    .expect("save_as is Some")
                    .save_command();
                self.modal_open = false;
                self.handle_command(&command);
            }
            crate::scrm_save_as_dialog::Outcome::Cancel => {
                // Req 24.5: cancelling writes nothing; the capture stays stopped.
                self.scrm_ui.save_as = None;
                self.modal_open = false;
            }
            crate::scrm_save_as_dialog::Outcome::Open => {}
        }
    }

    /// Render the saved-session PICKER modal when it is open and act on its
    /// outcome (Stream SCRM Phase 2C FEAT-003). The shell enumerates the
    /// `*.ffscrm` files in `scrm_dir()` fresh each frame and passes them to the
    /// picker render. On selection the shell dispatches `CAPTURE REPLAY <absolute
    /// path>` through the single front door (`handle_command`), which loads the
    /// archive and opens the ScrmViewer -- NO new load or dispatch path (Req
    /// 10.12). On Cancel / Escape / dimmed-background click nothing happens. A
    /// no-op when the modal is closed.
    ///
    /// Validates: screen-snapshot-scrm Requirement 10.12, 24.6.
    pub(super) fn render_scrm_replay_picker_overlay(&mut self, ctx: &egui::Context) {
        if self.scrm_ui.replay_picker.is_none() {
            return;
        }
        self.modal_open = true;
        let entries = self.enumerate_saved_scrm_sessions();
        let outcome = {
            let state = self
                .scrm_ui
                .replay_picker
                .as_mut()
                .expect("replay_picker is Some");
            crate::scrm_session_picker::render(ctx, state, &entries)
        };
        match outcome {
            crate::scrm_session_picker::Outcome::Selected(path) => {
                self.scrm_ui.replay_picker = None;
                self.modal_open = false;
                let command = format!("CAPTURE REPLAY {}", path.display());
                self.handle_command(&command);
            }
            crate::scrm_session_picker::Outcome::Cancel => {
                self.scrm_ui.replay_picker = None;
                self.modal_open = false;
            }
            crate::scrm_session_picker::Outcome::Open => {}
        }
    }

    /// Enumerate the saved sessions -- the `*.ffscrm` archives in `scrm_dir()` --
    /// as picker entries, sorted by display stem for a stable listing. A missing
    /// or unreadable directory yields an empty list (the picker shows its
    /// empty-state label).
    ///
    /// Validates: screen-snapshot-scrm Requirement 10.12, 24.6.
    pub(super) fn enumerate_saved_scrm_sessions(
        &self,
    ) -> Vec<crate::scrm_session_picker::SessionEntry> {
        let dir = self.scrm_dir();
        let mut entries: Vec<crate::scrm_session_picker::SessionEntry> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .map(|x| x.eq_ignore_ascii_case("ffscrm"))
                    .unwrap_or(false)
            })
            .map(crate::scrm_session_picker::SessionEntry::from_path)
            .collect();
        entries.sort_by(|a, b| a.stem.cmp(&b.stem));
        entries
    }
}
