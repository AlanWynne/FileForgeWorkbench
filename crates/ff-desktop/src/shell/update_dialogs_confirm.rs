//! # Shell confirmation overlays (RESET BARE + Unsaved-workspace)
//!
//! The RESET BARE confirmation and the Unsaved-workspace-changes dialogs, split
//! out of `update_dialogs.rs` to keep that file under the 400-line limit
//! (rust-standards.md) once the on-dialog Copy affordance (screen-snapshot-scrm
//! Req 27.3) was added to every overlay. Behaviour is unchanged -- these are the
//! SAME modal bodies that ran inline in `render_overlays`, called in the same
//! order. Each now carries the direct Copy button, routed to the single
//! `copy_frontmost_surface` path.

use eframe::egui;

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Render the RESET BARE confirmation dialog when it is open and act on its
    /// outcome (CR-CH-021, CR-NR-083). The SAME single dialog is reused for every
    /// target list; it names the profile(s) that will be reset. Escape / a click
    /// on the dimmed background cancels (the safe default). The on-dialog Copy
    /// affordance copies the dialog's redacted text and leaves the modal open.
    ///
    /// Validates: configuration-system Requirement 19.2, 19.3, 19.6, 19.15;
    /// screen-snapshot-scrm Requirement 27.3.
    pub(super) fn render_reset_bare_overlay(&mut self, ctx: &egui::Context) {
        if self.reset_bare_confirm.is_none() {
            return;
        }
        self.modal_open = true;
        let mut confirm_clicked = false;
        let mut cancel_clicked = false;
        let mut copy_clicked = false;
        // Snapshot the display names for the body without holding a borrow
        // across the closure.
        let profile_names: Vec<String> = self
            .reset_bare_confirm
            .as_ref()
            .map(|t| t.profiles.iter().map(|(name, _)| name.clone()).collect())
            .unwrap_or_default();
        // B077: render as an `egui::Modal` so the background is dimmed and
        // input is captured by the dialog (accessibility Req 2.3 -- modal
        // focus trap). Focus lands on Cancel on the first frame; Tab cycles
        // between Cancel and Confirm; Escape cancels.
        let focus_cancel = self.reset_bare_focus_requested;
        self.reset_bare_focus_requested = false;
        let modal = egui::Modal::new(egui::Id::new("reset_bare_modal")).show(ctx, |ui| {
            ui.set_max_width(460.0);
            ui.heading(ff_i18n::t("ff-desktop-shell-dialogs-reset-heading"));
            ui.label(ff_i18n::t("ff-desktop-shell-dialogs-reset-intro"));
            if profile_names.len() == 1 {
                ui.label(ff_i18n::t_args(
                    "ff-desktop-shell-dialogs-reset-one",
                    &ff_i18n::MessageArgs::new().set("name", profile_names[0].clone()),
                ));
            } else {
                // Count passed as TEXT (pre-formatted) to keep byte-identical
                // rendering (no Fluent NUMBER() grouping).
                ui.label(ff_i18n::t_args(
                    "ff-desktop-shell-dialogs-reset-many",
                    &ff_i18n::MessageArgs::new().set("count", profile_names.len().to_string()),
                ));
                for name in &profile_names {
                    ui.label(ff_i18n::t_args(
                        "ff-desktop-shell-dialogs-reset-bullet",
                        &ff_i18n::MessageArgs::new().set("name", name.clone()),
                    ));
                }
            }
            ui.label(ff_i18n::t("ff-desktop-shell-dialogs-reset-archive-note"));
            ui.horizontal(|ui| {
                // Cancel is first and the safe default -- it receives initial
                // focus (B077). Its stable id anchors the modal focus trap.
                let cancel = ui
                    .add(
                        egui::Button::new(ff_i18n::t("ff-desktop-shell-dialogs-cancel"))
                            .min_size(egui::vec2(96.0, 0.0)),
                    )
                    .on_hover_text(ff_i18n::t("ff-desktop-shell-dialogs-reset-cancel-hover"));
                let confirm = ui.add(
                    egui::Button::new(ff_i18n::t("ff-desktop-shell-dialogs-reset-confirm"))
                        .min_size(egui::vec2(120.0, 0.0)),
                );
                if focus_cancel {
                    cancel.request_focus();
                }
                if cancel.clicked() {
                    cancel_clicked = true;
                }
                if confirm.clicked() {
                    confirm_clicked = true;
                }
                // Direct COPY affordance (Req 27.3): copies this dialog's
                // redacted text and leaves it open.
                if ui
                    .add(
                        egui::Button::new(ff_i18n::t("ff-desktop-shell-dialogs-copy"))
                            .min_size(egui::vec2(96.0, 0.0)),
                    )
                    .clicked()
                {
                    copy_clicked = true;
                }
            });
        });
        // Req 23.2a: RESET BARE confirmation is priority 4.
        self.snapshot.note_dialog_rect(4, modal.response.rect);
        // Escape (or clicking the dimmed background) cancels -- the safe
        // default that leaves configuration untouched (Req 19.3).
        if modal.should_close() {
            cancel_clicked = true;
        }
        if confirm_clicked {
            self.modal_open = false;
            if let Some(target) = self.reset_bare_confirm.take() {
                self.execute_reset_bare(&target);
            }
        } else if cancel_clicked {
            // Req 19.3: cancelling leaves all configuration untouched.
            self.reset_bare_confirm = None;
            self.modal_open = false;
            self.open_error = None;
        } else if copy_clicked {
            self.copy_frontmost_surface();
        }
    }

    /// Render the Unsaved-workspace-changes dialog when it is open and act on its
    /// outcome (workspace-model Req 2.5). Save flushes and proceeds, Discard
    /// drops the edits and proceeds, Cancel aborts the pending open. The on-dialog
    /// Copy affordance copies the dialog's redacted text and leaves it open.
    ///
    /// Validates: workspace-model Requirement 2.5; screen-snapshot-scrm
    /// Requirement 27.3.
    pub(super) fn render_unsaved_workspace_overlay(&mut self, ctx: &egui::Context) {
        if !self.show_unsaved_workspace_dialog {
            return;
        }
        let mut save_clicked = false;
        let mut discard_clicked = false;
        let mut cancel_clicked = false;
        let mut copy_clicked = false;
        let ws_name = self
            .active_workspace
            .as_ref()
            .map(|ws| ws.name.clone())
            .unwrap_or_default();
        let unsaved_resp = egui::Window::new(ff_i18n::t("ff-desktop-shell-dialogs-unsaved-title"))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(ff_i18n::t_args(
                    "ff-desktop-shell-dialogs-unsaved-body",
                    &ff_i18n::MessageArgs::new().set("name", ws_name.clone()),
                ));
                ui.horizontal(|ui| {
                    if ui
                        .button(ff_i18n::t("ff-desktop-shell-dialogs-unsaved-save"))
                        .clicked()
                    {
                        save_clicked = true;
                    }
                    if ui
                        .button(ff_i18n::t("ff-desktop-shell-dialogs-unsaved-discard"))
                        .clicked()
                    {
                        discard_clicked = true;
                    }
                    if ui
                        .button(ff_i18n::t("ff-desktop-shell-dialogs-cancel"))
                        .clicked()
                    {
                        cancel_clicked = true;
                    }
                    // Direct COPY affordance (Req 27.3): copies this
                    // dialog's redacted text and leaves it open.
                    if ui
                        .button(ff_i18n::t("ff-desktop-shell-dialogs-copy"))
                        .clicked()
                    {
                        copy_clicked = true;
                    }
                });
            });
        // Req 23.2a: unsaved-workspace dialog is priority 3.
        if let Some(resp) = unsaved_resp {
            self.snapshot.note_dialog_rect(3, resp.response.rect);
        }
        if save_clicked {
            self.save_workspace_to(None);
            self.show_unsaved_workspace_dialog = false;
            if let Some(path) = self.pending_workspace_open.take() {
                self.open_workspace_force(&path);
            }
        } else if discard_clicked {
            self.show_unsaved_workspace_dialog = false;
            if let Some(path) = self.pending_workspace_open.take() {
                if let Some(ws) = self.active_workspace.as_mut() {
                    ws.is_modified = false;
                }
                self.open_workspace_force(&path);
            }
        } else if cancel_clicked {
            self.show_unsaved_workspace_dialog = false;
            self.pending_workspace_open = None;
        } else if copy_clicked {
            self.copy_frontmost_surface();
        }
    }
}
