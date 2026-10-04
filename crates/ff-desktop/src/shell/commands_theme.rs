//! # Shell Command Dispatch -- Theme Editor
//!
//! The themes directory resolver and the Theme Editor Context commands (open,
//! apply action, save/copy/reset). Split out of `commands.rs` (TASK 2.2, pure
//! code movement, no behaviour change). All methods are on `WorkbenchShell` and
//! keep their exact signatures and visibility.

use super::WorkbenchShell;

impl WorkbenchShell {
    /// The themes directory: the test override when set, else the real
    /// `<User_Data_Dir>/themes/` (via `theme_defaults::themes_dir`). All Theme
    /// editor / active-theme file operations route through this so tests can
    /// isolate them to a TempDir (CR-NR-074).
    pub(super) fn themes_dir(&self) -> std::path::PathBuf {
        self.dir_overrides
            .themes
            .clone()
            .unwrap_or_else(crate::theme_defaults::themes_dir)
    }
    /// Open the Theme Editor Context and populate its state: list available
    /// themes, select the current active theme, and load it as the working copy.
    ///
    /// On a POM tab, transforms in place (so END/RETURN returns to the POM);
    /// otherwise opens a dedicated tab.
    ///
    /// Validates: theme-and-appearance Requirement 20.1, 20.2
    pub(super) fn open_theme_editor(&mut self) {
        let themes_dir = self.themes_dir();
        // Available themes (built-in + user).
        let available: Vec<String> = ff_theme::list_all_themes(&themes_dir)
            .into_iter()
            .map(|t| t.name)
            .collect();
        // Selected = the current active palette's name (the running theme).
        let selected = self.palette.name.clone();
        self.theme_editor_panel.available = available;
        // Load the current palette as the working copy so edits start from what
        // is on screen.
        self.theme_editor_panel
            .load_working(&selected, self.palette.clone());

        // CR-CH-022 Req 14.2: navigate the current tab in place (push). The
        // Theme editor's working state is already set on the shell above.
        self.navigate_to(
            ff_session::session_state::WorkspaceDescriptor::CustomWorkspace {
                workspace_kind: ff_session::session_state::WorkspaceKind::CommandConfigurator,
                params: {
                    let mut p = ff_session::session_state::DescriptorParams::new();
                    p.insert(
                        "editor".to_string(),
                        ff_session::session_state::DescriptorValue::from("theme"),
                    );
                    p
                },
            },
            true,
        );
    }

    /// Apply a `ThemeEditorAction` produced by the Theme Editor render. Side
    /// effects (file writes, palette swap, config persist) live here in the
    /// command layer, keeping the panel render pure.
    ///
    /// Validates: theme-and-appearance Requirement 20.4-20.8
    pub(super) fn apply_theme_editor_action(
        &mut self,
        action: crate::theme_editor_panel::ThemeEditorAction,
    ) {
        use crate::theme_editor_panel::ThemeEditorAction as A;
        let themes_dir = self.themes_dir();
        match action {
            A::None => {}
            A::Select(name) => {
                // Load the selected theme as the new working copy (Req 20.1).
                if let Some(p) = crate::theme_defaults::load_theme_by_name(&name, &themes_dir) {
                    self.theme_editor_panel.load_working(&name, p);
                } else {
                    self.theme_editor_panel.error =
                        Some(format!("Theme '{name}' could not be loaded"));
                }
            }
            A::EditToken(token, colour) => {
                // Update the working copy and live-preview it (Req 20.3, 20.8).
                if let Some(p) = self.theme_editor_panel.working.as_mut() {
                    token.set(p, colour);
                    let preview = p.clone();
                    self.theme_editor_panel.recompute_advisories();
                    // Live preview: apply the working copy to the active palette.
                    self.palette = preview;
                }
            }
            A::Copy(new_name) => {
                // New named theme initialised from the working copy (Req 20.4).
                if let Some(mut p) = self.theme_editor_panel.working.clone() {
                    p.name = new_name.clone();
                    if let Err(e) = self.write_theme_file(&new_name, &p) {
                        self.theme_editor_panel.error = Some(e);
                    } else {
                        self.theme_editor_panel.name_buffer.clear();
                        self.refresh_theme_editor_list();
                        self.theme_editor_panel.load_working(&new_name, p);
                    }
                }
            }
            A::Save => {
                // Save the working copy to the selected theme's file (Req 20.5).
                if let (Some(name), Some(p)) = (
                    self.theme_editor_panel.selected.clone(),
                    self.theme_editor_panel.working.clone(),
                ) {
                    // CR-CH-019: a built-in is read-only and code-only -- Save
                    // redirects to Save As. Use the typed new name if present,
                    // else guide the user to enter one.
                    if ff_theme::is_builtin_theme(&name) {
                        let new_name = self.theme_editor_panel.name_buffer.trim().to_string();
                        if new_name.is_empty() {
                            self.theme_editor_panel.error = Some(format!(
                                "'{name}' is a built-in theme and cannot be overwritten. Enter a new name and use Save As (or Copy) to keep your changes."
                            ));
                        } else {
                            self.apply_theme_editor_action(A::SaveAs(new_name));
                        }
                    } else if let Err(e) = self.write_theme_file(&name, &p) {
                        self.theme_editor_panel.error = Some(e);
                    } else {
                        self.theme_editor_panel.error = None;
                    }
                }
            }
            A::SaveAs(new_name) => {
                if let Some(mut p) = self.theme_editor_panel.working.clone() {
                    p.name = new_name.clone();
                    if let Err(e) = self.write_theme_file(&new_name, &p) {
                        self.theme_editor_panel.error = Some(e);
                    } else {
                        self.theme_editor_panel.name_buffer.clear();
                        self.refresh_theme_editor_list();
                        self.theme_editor_panel.load_working(&new_name, p);
                    }
                }
            }
            A::SetActive(name) => {
                // Apply immediately + persist (Req 20.6). Uses the shared helper.
                self.set_active_theme(&name);
            }
            A::Reset(name) => {
                // Re-select the baseline for this theme (Req 20.7/18.4). For a
                // built-in this re-selects the compiled palette (no file write).
                self.reset_theme_reselect(&name);
            }
        }
    }

    /// Serialise `palette` and write it to `<themes>/<slug>.toml`.
    fn write_theme_file(&self, name: &str, palette: &ff_theme::ThemePalette) -> Result<(), String> {
        let themes_dir = self.themes_dir();
        std::fs::create_dir_all(&themes_dir)
            .map_err(|e| format!("could not create themes dir: {e}"))?;
        let path = themes_dir.join(format!("{}.toml", crate::theme_defaults::theme_slug(name)));
        let toml = ff_theme::serialiser::serialise(palette);
        std::fs::write(&path, toml).map_err(|e| format!("could not write theme '{name}': {e}"))
    }

    /// Reset a theme to its baseline (Requirement 18.4 / 20.7; CR-CH-019).
    /// For a BUILT-IN name, re-selects the compiled built-in palette into the
    /// working copy -- no file is written (built-ins are code-only). For a user
    /// theme with a resolvable `base`, restores the base colours; otherwise
    /// reports that there is no baseline to reset to.
    fn reset_theme_reselect(&mut self, name: &str) {
        if let Some(p) = crate::theme_defaults::builtin_palette_by_name(name) {
            // Built-in: pure in-memory re-select, no file touched.
            self.theme_editor_panel.load_working(name, p);
            return;
        }
        // User theme: reload from disk (discards unsaved edits). A future
        // enhancement could restore from a declared `base`; for now reloading
        // the saved file is the baseline for a user theme.
        let themes_dir = self.themes_dir();
        match crate::theme_defaults::load_theme_by_name(name, &themes_dir) {
            Some(p) => self.theme_editor_panel.load_working(name, p),
            None => {
                self.theme_editor_panel.error =
                    Some(format!("'{name}' has no saved baseline to reset to"));
            }
        }
    }

    /// Refresh the Theme Editor's available-themes list from disk + built-ins.
    fn refresh_theme_editor_list(&mut self) {
        let themes_dir = self.themes_dir();
        self.theme_editor_panel.available = ff_theme::list_all_themes(&themes_dir)
            .into_iter()
            .map(|t| t.name)
            .collect();
    }
}
