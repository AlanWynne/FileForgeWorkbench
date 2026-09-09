//! Command Configurator Context -- render the definitions table and dispatch
//! Add / Edit / Delete actions.
//!
//! The render function is a free function (mirroring `settings_panel::render`)
//! that draws the list of `CommandDefinition`s and any open edit form or
//! delete-confirmation dialog, returning a [`ConfiguratorAction`] the shell
//! applies against the `CommandStore`.
//!
//! Validates: command-configurator Requirement 2.2, 2.3, 2.6.

use eframe::egui;

use super::edit::EditForm;
use super::store::CommandStore;

/// An action produced by the Command Configurator render, applied by the shell.
///
/// Validates: command-configurator Requirement 2.3, 2.4, 2.5.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfiguratorAction {
    /// No action this frame.
    None,
    /// The user asked to commit the currently open edit form (add or edit).
    Save,
    /// The user asked to delete the definition with this id (confirmed).
    Delete(String),
}

/// Per-Context UI state for the Command Configurator.
///
/// Lives on the shell (like `SettingsPanelState`), not on the `TabState`.
///
/// Validates: command-configurator Requirement 2.1, 2.3.
#[derive(Debug, Clone, Default)]
pub struct CommandConfiguratorState {
    /// The open Add/Edit form, when the user is creating or editing.
    pub form: Option<EditForm>,
    /// The id pending delete confirmation, when a delete was requested.
    pub pending_delete: Option<String>,
    /// The most recent validation or save error, shown inline.
    pub error: Option<String>,
}

impl CommandConfiguratorState {
    /// Create an empty state.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Render the Command Configurator Context.
///
/// Validates: command-configurator Requirement 2.2, 2.3, 2.6.
pub fn render(
    ui: &mut egui::Ui,
    state: &mut CommandConfiguratorState,
    store: &CommandStore,
) -> ConfiguratorAction {
    let mut action = ConfiguratorAction::None;

    // Header + Add action (Requirement 2.3).
    ui.horizontal(|ui| {
        ui.heading("Command Configurator");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Add").clicked() && state.form.is_none() {
                state.form = Some(EditForm::new_add());
                state.error = None;
            }
        });
    });

    // Load diagnostics (Requirement 1.6).
    if let Some(err) = &store.load_error {
        ui.colored_label(egui::Color32::from_rgb(0xC8, 0x8A, 0x00), err);
    }
    // Inline action error (validation / save failure, Requirement 2.4).
    if let Some(err) = &state.error {
        ui.colored_label(egui::Color32::RED, err);
    }
    ui.separator();

    // Definitions table (Requirement 2.2).
    egui::ScrollArea::vertical()
        .id_salt("command_configurator_list")
        .show(ui, |ui| {
            if store.definitions.is_empty() {
                ui.label("No user-defined commands. Use Add to create one.");
            } else {
                egui::Grid::new("command_configurator_grid")
                    .num_columns(5)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Id");
                        ui.strong("Label");
                        ui.strong("Variant");
                        ui.strong("Mode");
                        ui.strong("Actions");
                        ui.end_row();

                        for def in &store.definitions {
                            ui.monospace(&def.id);
                            ui.label(&def.label);
                            ui.label(def.target.variant_name());
                            ui.label(external_mode_label(&def.target));
                            ui.horizontal(|ui| {
                                if ui.small_button("Edit").clicked() && state.form.is_none() {
                                    state.form = Some(EditForm::from_definition(def));
                                    state.error = None;
                                }
                                if ui.small_button("Delete").clicked() {
                                    // Req 2.3: delete requires confirmation.
                                    state.pending_delete = Some(def.id.clone());
                                }
                            });
                            ui.end_row();
                        }
                    });
            }
        });

    // Delete confirmation (Requirement 2.3).
    if let Some(id) = state.pending_delete.clone() {
        egui::Window::new("Confirm delete")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label(format!("Delete command definition '{id}'?"));
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        action = ConfiguratorAction::Delete(id.clone());
                        state.pending_delete = None;
                    }
                    if ui.button("Cancel").clicked() {
                        state.pending_delete = None;
                    }
                });
            });
    }

    // Add/Edit form (Requirement 2.6). Returns Save when the user commits.
    if state.form.is_some() {
        let mut save_clicked = false;
        let mut cancel_clicked = false;
        egui::Window::new(if state.form.as_ref().map(|f| f.is_edit).unwrap_or(false) {
            "Edit command"
        } else {
            "Add command"
        })
        .collapsible(false)
        .resizable(true)
        .show(ui.ctx(), |ui| {
            if let Some(form) = state.form.as_mut() {
                form.render(ui);
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    save_clicked = true;
                }
                if ui.button("Cancel").clicked() {
                    cancel_clicked = true;
                }
            });
        });
        if save_clicked {
            action = ConfiguratorAction::Save;
        } else if cancel_clicked {
            state.form = None;
            state.error = None;
        }
    }

    action
}

/// Display label for the External execution mode column; blank for non-External.
fn external_mode_label(target: &ff_command::CommandTarget) -> &'static str {
    match target {
        ff_command::CommandTarget::External {
            mode: ff_command::ExternalMode::Detached,
            ..
        } => "detached",
        ff_command::CommandTarget::External {
            mode: ff_command::ExternalMode::Captured,
            ..
        } => "captured",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.2 -- external mode label reflects the mode.
    #[test]
    fn external_mode_label_reflects_mode() {
        use ff_command::{CommandTarget, ExternalMode};
        let detached = CommandTarget::External {
            program: "p".to_string(),
            args: vec![],
            working_dir: None,
            mode: ExternalMode::Detached,
        };
        let captured = CommandTarget::External {
            program: "p".to_string(),
            args: vec![],
            working_dir: None,
            mode: ExternalMode::Captured,
        };
        assert_eq!(external_mode_label(&detached), "detached");
        assert_eq!(external_mode_label(&captured), "captured");
    }

    // Validates: Requirement 2.2 -- non-External targets have a blank mode label.
    #[test]
    fn non_external_target_has_blank_mode_label() {
        use ff_command::{CommandTarget, TargetParams};
        let func = CommandTarget::Function {
            command_id: "file.save".to_string(),
            params: TargetParams::new(),
        };
        assert_eq!(external_mode_label(&func), "");
    }
}
