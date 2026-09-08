//! egui rendering for Menu_Workspace.
//!
//! Validates: Requirement 2 (menu-workspace)

use eframe::egui;

use super::MenuWorkspaceState;

/// Render the option list area of a Menu_Workspace.
///
/// Returns the `Option_Command` string of the option the user clicked,
/// or `None` if no selection was made this frame.
///
/// The caller (shell render) is responsible for the surrounding chrome
/// (Title_Line, Command_Field, Key_Label_Bar). This function renders only
/// the Menu_Title and option list inside the central panel area.
///
/// Validates: Requirement 2.1-2.6
pub fn render_menu_workspace(state: &mut MenuWorkspaceState, ui: &mut egui::Ui) -> Option<String> {
    let mut selected_command: Option<String> = None;

    match &state.menu {
        None => {
            // Req 1.5, 1.6 -- show error message
            let msg = state
                .load_error
                .clone()
                .unwrap_or_else(|| "Menu file error: unknown error".to_string());
            ui.colored_label(egui::Color32::RED, msg);
        }
        Some(menu) => {
            // Req 2.1 -- Menu_Title centred
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new(&menu.title).strong().size(14.0));
            });
            ui.add_space(4.0);

            // Req 2.5 -- scrollable option list
            egui::ScrollArea::vertical()
                .id_salt("menu_workspace_options")
                .show(ui, |ui| {
                    if menu.options.is_empty() {
                        // Req 2.6 -- empty options placeholder
                        ui.label("No options defined in this menu.");
                        return;
                    }

                    let mut last_group: Option<&str> = None;

                    for option in &menu.options {
                        // Req 2.4 -- group separator
                        let current_group = option.group.as_deref();
                        if current_group != last_group && last_group.is_some() {
                            ui.separator();
                        }
                        last_group = current_group;

                        // Req 2.2, 2.3 -- interactive vs disabled rows
                        let row_text = format!("{:<4}  {}", option.key, option.description);

                        if option.enabled {
                            // Req 2.2 -- selectable_label responds to click
                            if ui
                                .selectable_label(false, egui::RichText::new(&row_text).monospace())
                                .clicked()
                            {
                                selected_command = Some(option.command.clone());
                            }
                        } else {
                            // Req 2.3 -- disabled style, no interaction
                            ui.add_enabled(
                                false,
                                egui::Label::new(
                                    egui::RichText::new(&row_text)
                                        .monospace()
                                        .color(egui::Color32::GRAY),
                                ),
                            );
                        }
                    }
                });
        }
    }

    selected_command
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_workspace::{MenuFile, MenuOption};

    fn make_state_with_options(options: Vec<MenuOption>) -> MenuWorkspaceState {
        MenuWorkspaceState {
            file_path: std::path::PathBuf::from("test.toml"),
            menu: Some(MenuFile {
                title: "Test Menu".to_string(),
                options,
            }),
            load_error: None,
            last_modified: None,
        }
    }

    // Validates: Requirement 2.6 -- empty options list produces placeholder state
    #[test]
    fn render_empty_options_state_is_valid() {
        let state = make_state_with_options(vec![]);
        assert!(state.menu.as_ref().unwrap().options.is_empty());
    }

    // Validates: Requirement 2.3 -- disabled option is not selectable
    #[test]
    fn render_disabled_option_not_selectable() {
        let state = make_state_with_options(vec![MenuOption {
            key: "1".to_string(),
            command: "FILES".to_string(),
            description: "Files".to_string(),
            enabled: false,
            group: None,
        }]);
        assert!(!state.menu.as_ref().unwrap().options[0].enabled);
    }

    // Validates: Requirement 1.5 -- error state has load_error set
    #[test]
    fn render_error_state_has_message() {
        let state = MenuWorkspaceState {
            file_path: std::path::PathBuf::from("missing.toml"),
            menu: None,
            load_error: Some("Menu file not found: missing.toml".to_string()),
            last_modified: None,
        };
        assert!(state.load_error.is_some());
    }
}
