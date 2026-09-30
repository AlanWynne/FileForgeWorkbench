//! Screen snapshot wiring for the desktop shell (CR-NR-098, Wave 1).
//!
//! This module bridges the shell's live Context state to the GUI-independent
//! [`ff_screen_model`] capture engine. It builds a logical [`ScreenModel`] from a
//! Context's state (so capture is copy/paste-able TEXT, never a raster) and
//! renders it to the requested [`SnapshotFormat`].
//!
//! Wave 1 provides the FIRST `ScreenProvider`: the Home Context (POM), whose
//! content is a data-driven Menu_Workspace. Later waves add providers for other
//! Contexts and the collection/replay wiring.
//!
//! Validates: screen-snapshot-scrm Requirement 2.1, 2.2, 3.1, 3.8, 4.1-4.6.

use ff_screen_model::{Field, RenderOptions, ScreenModel, SnapshotFormat};

use crate::menu_workspace::MenuWorkspaceState;

/// Build a logical [`ScreenModel`] from a Menu_Workspace's loaded state (the POM
/// and every other menu Context share this shape). Each menu option becomes a
/// [`Field`] whose label is the option key and whose value is
/// "description (command)". A load error is surfaced as a message so the capture
/// still reflects what the user sees.
///
/// `command_line` is the current `Command ===>` field content, threaded through
/// so the snapshot matches the live screen (Requirement 3.8).
///
/// Validates: Requirement 2.1, 3.1, 3.8.
pub fn menu_workspace_screen_model(state: &MenuWorkspaceState, command_line: &str) -> ScreenModel {
    let title = state.menu_title().unwrap_or_else(|| "Menu".to_string());
    let mut model = ScreenModel::new(title);
    model.command_line = command_line.to_string();

    match &state.menu {
        Some(menu) => {
            for opt in &menu.options {
                // label = key; value = "description (command)". Both are visible,
                // copy/paste-able text on the screen.
                let value = if opt.command.trim().is_empty() {
                    opt.description.clone()
                } else {
                    format!("{} ({})", opt.description, opt.command)
                };
                model = model.with_field(Field::new(opt.key.clone(), value));
            }
        }
        None => {
            let msg = state
                .load_error
                .clone()
                .unwrap_or_else(|| "Menu not loaded".to_string());
            model.messages.push(msg);
        }
    }
    model
}

/// Render a [`ScreenModel`] to the requested format's text, ready for the
/// clipboard. Thin wrapper over [`ff_screen_model::render`] with default options
/// (Unicode box drawing).
///
/// Validates: Requirement 4.1-4.6.
pub fn render_snapshot(model: &ScreenModel, format: SnapshotFormat) -> String {
    ff_screen_model::render(model, format, RenderOptions::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_menu(toml: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().expect("tempfile");
        f.write_all(toml.as_bytes()).expect("write");
        f
    }

    fn pom_like() -> MenuWorkspaceState {
        let toml = r#"
title = "Primary Option Menu"
[[options]]
key = "1"
command = "FILES"
description = "File Explorer"
[[options]]
key = "0"
command = "SETTINGS"
description = "Settings"
"#;
        let f = write_menu(toml);
        MenuWorkspaceState::load(f.path())
    }

    // Validates: Requirement 2.1, 3.1 -- model carries the menu title + one
    // field per option with visible key/description/command text.
    #[test]
    fn menu_model_has_title_and_option_fields() {
        let state = pom_like();
        let model = menu_workspace_screen_model(&state, "");
        assert_eq!(model.title, "Primary Option Menu");
        assert_eq!(model.fields.len(), 2);
        assert_eq!(model.fields[0].label, "1");
        assert!(model.fields[0].value.contains("File Explorer"));
        assert!(model.fields[0].value.contains("FILES"));
    }

    // Validates: Requirement 3.8 -- the command line is threaded into the model.
    #[test]
    fn command_line_is_captured() {
        let state = pom_like();
        let model = menu_workspace_screen_model(&state, "SNAPSHOT");
        assert_eq!(model.command_line, "SNAPSHOT");
    }

    // Validates: Requirement 2.1 -- a load error surfaces as a message.
    #[test]
    fn load_error_surfaces_as_message() {
        let state = MenuWorkspaceState::load("/nonexistent/menu.toml");
        let model = menu_workspace_screen_model(&state, "");
        assert!(!model.messages.is_empty());
    }

    // Validates: Requirement 1.1, 4.2 -- rendered snapshot is selectable text
    // containing the option descriptions.
    #[test]
    fn render_snapshot_contains_option_text() {
        let state = pom_like();
        let model = menu_workspace_screen_model(&state, "");
        let text = render_snapshot(&model, SnapshotFormat::PlainText);
        assert!(text.contains("Primary Option Menu"));
        assert!(text.contains("File Explorer"));
        assert!(text.contains("Settings"));
    }
}
