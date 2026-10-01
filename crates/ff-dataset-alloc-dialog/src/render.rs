//! Dataset allocation dialog egui rendering.
//!
//! Renders the modal Allocate Dataset window and returns the per-frame
//! `AllocOutcome`. The only egui-dependent module in the crate.
//!
//! Validates: Requirement 5.1-5.5

use crate::form::{AllocDatasetForm, AllocOutcome, Dsorg, Recfm};
use crate::validate::validate;

// === Render =================================================================

/// Render the Dataset Allocation modal dialog.
///
/// Returns `AllocOutcome::Confirmed` (with validated params stored in `form`)
/// when the user confirms, `AllocOutcome::Cancelled` when they cancel, or
/// `AllocOutcome::Open` while the dialog remains active.
///
/// Validates: Requirement 5.1-5.5
pub fn render(ctx: &egui::Context, form: &mut AllocDatasetForm) -> AllocOutcome {
    let mut outcome = AllocOutcome::Open;

    egui::Window::new("Allocate Dataset")
        .collapsible(false)
        .resizable(false)
        .min_width(460.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_min_width(440.0);

            // === Dataset Name ===================================================
            ui.horizontal(|ui| {
                ui.label("Dataset Name:      ");
                ui.text_edit_singleline(&mut form.dataset_name);
            });

            // === DSORG selector =================================================
            ui.horizontal(|ui| {
                ui.label("Dataset Org (DSORG):");
                for dsorg in [Dsorg::Ps, Dsorg::Po, Dsorg::Pdse, Dsorg::Gdg] {
                    ui.selectable_value(&mut form.dsorg, dsorg, dsorg.label());
                }
            });

            // === RECFM selector =================================================
            ui.horizontal(|ui| {
                ui.label("Record Format:     ");
                for recfm in [Recfm::Fb, Recfm::F, Recfm::Vb, Recfm::V, Recfm::U] {
                    ui.selectable_value(&mut form.recfm, recfm, recfm.label());
                }
            });

            // === LRECL / BLKSIZE ================================================
            ui.horizontal(|ui| {
                ui.label("LRECL:             ");
                ui.add(
                    egui::TextEdit::singleline(&mut form.lrecl)
                        .desired_width(60.0)
                        .font(egui::TextStyle::Monospace),
                );
                ui.label("  Block Size:");
                ui.add(
                    egui::TextEdit::singleline(&mut form.blksize)
                        .desired_width(80.0)
                        .font(egui::TextStyle::Monospace),
                );
            });

            // === Conditional fields =============================================
            match form.dsorg {
                Dsorg::Po | Dsorg::Pdse => {
                    ui.horizontal(|ui| {
                        ui.label("Directory Blocks:  ");
                        ui.add(
                            egui::TextEdit::singleline(&mut form.dir_blocks)
                                .desired_width(60.0)
                                .font(egui::TextStyle::Monospace),
                        );
                    });
                }
                Dsorg::Gdg => {
                    ui.horizontal(|ui| {
                        ui.label("GDG Limit (1-255): ");
                        ui.add(
                            egui::TextEdit::singleline(&mut form.gdg_limit)
                                .desired_width(60.0)
                                .font(egui::TextStyle::Monospace),
                        );
                    });
                    ui.checkbox(&mut form.scratch, "Scratch on roll-off");
                }
                _ => {}
            }

            // === Description ====================================================
            ui.horizontal(|ui| {
                ui.label("Description:       ");
                ui.text_edit_singleline(&mut form.description);
            });

            // === Inline error -- Req 5.5 ========================================
            if let Some(err) = &form.error {
                ui.colored_label(egui::Color32::RED, err);
            }

            ui.separator();

            // === Buttons ========================================================
            ui.horizontal(|ui| {
                if ui.button("Allocate").clicked() {
                    match validate(form) {
                        Ok(_) => {
                            form.error = None;
                            outcome = AllocOutcome::Confirmed;
                        }
                        Err(e) => {
                            form.error = Some(e);
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    outcome = AllocOutcome::Cancelled;
                }
            });
        });

    // Validates: accessibility Requirement 2.1, 2.3 -- Escape closes the dialog.
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        outcome = AllocOutcome::Cancelled;
    }

    outcome
}
