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
                    // A chrome token writes a flat authoring field (ui/tab_bar/
                    // editor); re-derive the egui chrome layer so the live
                    // preview reflects the edit (CR-CH-056 Req 20.11). Domain
                    // edits leave the chrome unchanged; the re-derive is cheap
                    // and keeps the preview consistent either way.
                    p.rederive_chrome_style();
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
            A::Export => {
                // Command parity (Req 24.5): the UI affordance invokes the SAME
                // command the typed `THEME EXPORT` runs, rather than calling the
                // export logic directly.
                self.handle_command("THEME EXPORT");
            }
            A::Import => {
                // Command parity (Req 24.5): invoke the `THEME IMPORT` command.
                self.handle_command("THEME IMPORT");
            }
        }
    }

    /// Export the Theme Editor's selected (or working) theme to a native FFWB
    /// theme file at `path`. The pure write is separated from the file-picker
    /// path I/O (`export_theme_command`) so it is testable without a dialog.
    ///
    /// Validates: theme-and-appearance Requirement 24.1
    pub(super) fn export_theme_to_path(&mut self, path: &std::path::Path) -> Result<(), String> {
        // Prefer the working copy (reflects unsaved edits); fall back to the
        // active palette. Name the export after the selected theme.
        let palette = self
            .theme_editor_panel
            .working
            .clone()
            .unwrap_or_else(|| self.palette.clone());
        let name = self
            .theme_editor_panel
            .selected
            .clone()
            .unwrap_or_else(|| palette.name.clone());
        let toml = ff_theme::export_theme(&palette, &name)
            .map_err(|e| format!("could not serialise theme '{name}': {e}"))?;
        std::fs::write(path, toml)
            .map_err(|e| format!("could not write export '{}': {e}", path.display()))
    }

    /// Import a native FFWB theme file from `path` into the themes directory as
    /// a selectable user theme. Validates the file first (Req 24.3): a foreign /
    /// invalid file is rejected with a clear message and NOTHING is written, so
    /// the existing themes and the active theme are never corrupted. The pure
    /// read/validate/write is separated from the file-picker path I/O
    /// (`import_theme_command`) so it is testable without a dialog.
    ///
    /// Validates: theme-and-appearance Requirement 24.2, 24.3
    pub(super) fn import_theme_from_path(
        &mut self,
        path: &std::path::Path,
    ) -> Result<String, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("could not read '{}': {e}", path.display()))?;
        // Validate it is a native FFWB theme BEFORE touching the themes dir.
        let palette = ff_theme::parse_native_theme(&source, ff_theme::mode::VisualMode::Dark)
            .map_err(|e| format!("import rejected: {e}"))?;
        // A user import must never shadow a built-in name (Req 24.2 / 19.2a).
        if ff_theme::is_builtin_theme(&palette.name) {
            return Err(format!(
                "import rejected: '{}' is a built-in theme name; rename the file's name field",
                palette.name
            ));
        }
        self.write_theme_file(&palette.name, &palette)?;
        self.refresh_theme_editor_list();
        Ok(palette.name)
    }

    /// `THEME EXPORT` command: pick a destination path (rfd save dialog) and
    /// export the selected/active theme there. Command parity (Req 24.5).
    ///
    /// Validates: theme-and-appearance Requirement 24.1, 24.5
    pub(super) fn export_theme_command(&mut self) {
        let default_name = self
            .theme_editor_panel
            .selected
            .clone()
            .unwrap_or_else(|| self.palette.name.clone());
        let picked = rfd::FileDialog::new()
            .set_title("Export FFWB theme")
            .set_file_name(format!(
                "{}.toml",
                crate::theme_defaults::theme_slug(&default_name)
            ))
            .add_filter("FFWB theme", &["toml"])
            .save_file();
        if let Some(path) = picked {
            match self.export_theme_to_path(&path) {
                Ok(()) => self.theme_editor_panel.error = None,
                Err(e) => self.theme_editor_panel.error = Some(e),
            }
        }
    }

    /// `THEME IMPORT` command: pick a source path (rfd open dialog) and import
    /// it as a selectable user theme. Command parity (Req 24.5).
    ///
    /// Validates: theme-and-appearance Requirement 24.2, 24.3, 24.5
    pub(super) fn import_theme_command(&mut self) {
        let picked = rfd::FileDialog::new()
            .set_title("Import FFWB theme")
            .add_filter("FFWB theme", &["toml"])
            .pick_file();
        if let Some(path) = picked {
            match self.import_theme_from_path(&path) {
                Ok(name) => {
                    self.theme_editor_panel.error = None;
                    self.theme_editor_panel.load_working(
                        &name,
                        self.palette_for_name(&name)
                            .unwrap_or_else(|| self.palette.clone()),
                    );
                }
                Err(e) => self.theme_editor_panel.error = Some(e),
            }
        }
    }

    /// Resolve a theme NAME to its palette (built-in or user file), for
    /// re-targeting the editor after an import.
    fn palette_for_name(&self, name: &str) -> Option<ff_theme::ThemePalette> {
        crate::theme_defaults::load_theme_by_name(name, &self.themes_dir())
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
