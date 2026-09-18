//! Shell-side wiring for the Menus Editor Context (menu-workspace Req 13,
//! CR-NR-075).
//!
//! The panel render is pure (`menus_editor_panel::render`); all side effects
//! (validation, serialisation, file writes, list refresh) live here in the
//! command layer, mirroring the Theme editor (`apply_theme_editor_action`).

use super::WorkbenchShell;
use crate::menu_workspace::{MenuFile, MenuOption};
use crate::menus_editor_panel::MenusEditorAction;

impl WorkbenchShell {
    /// Open the Menus Editor Context and populate its state: list editable
    /// menus and load the current selection (or POM) as the working copy.
    ///
    /// On a POM tab, transforms in place (so END/RETURN returns to the POM);
    /// otherwise opens or activates a dedicated tab.
    ///
    /// Validates: menu-workspace Requirement 13.1, 13.2, 13.3
    pub(super) fn open_menus_editor(&mut self) {
        self.refresh_menus_editor_list();
        // Default the selection to POM the first time the editor opens.
        let selected = self
            .menus_editor_panel
            .selected
            .clone()
            .unwrap_or_else(|| "POM".to_string());
        let menu = self.load_menu_for_editor(&selected);
        self.menus_editor_panel.load_working(&selected, menu);

        // CR-CH-022 Req 14.2: navigate the current tab to the Menus editor in
        // place (push onto the Navigation_Stack). END pops back to whatever the
        // editor was opened from (Settings, POM, ...) via the stack -- no
        // per-flag origin tracking needed (supersedes B053's opened_from_settings).
        self.navigate_to(
            ff_session::session_state::WorkspaceDescriptor::CustomWorkspace {
                workspace_kind: ff_session::session_state::WorkspaceKind::CommandConfigurator,
                params: {
                    let mut p = ff_session::session_state::DescriptorParams::new();
                    p.insert(
                        "editor".to_string(),
                        ff_session::session_state::DescriptorValue::from("menus"),
                    );
                    p
                },
            },
            true,
        );
    }

    /// Apply a [`MenusEditorAction`] produced by the Menus Editor render.
    ///
    /// Validates: menu-workspace Requirement 13.4-13.9
    pub(super) fn apply_menus_editor_action(&mut self, action: MenusEditorAction) {
        use MenusEditorAction as A;
        match action {
            A::None => {}
            A::Select(name) => {
                let menu = self.load_menu_for_editor(&name);
                self.menus_editor_panel.load_working(&name, menu);
            }
            // Note: title, display toggles, and per-option field edits are
            // applied DIRECTLY by the render on the mutable working menu (B054),
            // so they do not appear as actions here -- only structural changes do.
            A::AddOption => {
                if let Some(m) = self.menus_editor_panel.working.as_mut() {
                    m.options.push(MenuOption {
                        key: String::new(),
                        command: String::new(),
                        description: String::new(),
                        enabled: true,
                        group: None,
                        show_in_menu_bar: true,
                        target: None,
                    });
                }
            }
            A::DeleteOption(index) => {
                if let Some(m) = self.menus_editor_panel.working.as_mut() {
                    if index < m.options.len() {
                        m.options.remove(index);
                    }
                }
            }
            A::MoveOptionUp(index) => {
                if let Some(m) = self.menus_editor_panel.working.as_mut() {
                    if index > 0 && index < m.options.len() {
                        m.options.swap(index - 1, index);
                    }
                }
            }
            A::MoveOptionDown(index) => {
                if let Some(m) = self.menus_editor_panel.working.as_mut() {
                    if index + 1 < m.options.len() {
                        m.options.swap(index, index + 1);
                    }
                }
            }
            A::Save => {
                if let Some(name) = self.menus_editor_panel.selected.clone() {
                    self.save_working_menu(&name);
                }
            }
            A::SaveAs(new_name) => {
                self.save_working_menu(&new_name);
                self.menus_editor_panel.name_buffer.clear();
            }
        }
    }

    /// Validate then serialise+write the working menu to `menus/<name>.toml`,
    /// refreshing the list and re-selecting the saved menu on success.
    ///
    /// Validates: menu-workspace Requirement 13.7, 13.8, 13.9, 13.13
    fn save_working_menu(&mut self, name: &str) {
        let Some(menu) = self.menus_editor_panel.working.clone() else {
            return;
        };
        let limits = crate::menu_workspace::loader::option_limits_from_config(&self.config_handle);
        // Shared validation: the editor cannot produce a file the loader rejects.
        if let Err(e) = crate::menu_workspace::loader::validate_menu(&menu, limits) {
            self.menus_editor_panel.error = Some(e);
            return;
        }
        if let Err(e) = self.write_menu_file(name, &menu) {
            self.menus_editor_panel.error = Some(e);
            return;
        }
        self.menus_editor_panel.error = None;
        self.refresh_menus_editor_list();
        self.menus_editor_panel.load_working(name, menu);
    }

    /// Serialise `menu` and write it to `<menus_dir>/<file-stem>.toml`.
    ///
    /// The built-in names POM / Settings map to `pom.toml` / `settings.toml`
    /// (a user override the renderer prefers, Req 13.8); any other name is
    /// slugified.
    ///
    /// Validates: menu-workspace Requirement 13.8
    fn write_menu_file(&self, name: &str, menu: &MenuFile) -> Result<(), String> {
        let menus_dir = self.menus_dir();
        std::fs::create_dir_all(&menus_dir)
            .map_err(|e| format!("could not create menus dir: {e}"))?;
        let stem = menu_file_stem(name);
        let path = menus_dir.join(format!("{stem}.toml"));
        let toml = crate::menu_workspace::serialiser::serialise(menu);
        std::fs::write(&path, toml).map_err(|e| format!("could not write menu '{name}': {e}"))
    }

    /// Load a menu by editor name into a `MenuFile` for editing. A built-in name
    /// (POM / Settings) with no user file falls back to the compiled
    /// Recovery_Baseline (Req 13.3); a user file is parsed from disk.
    fn load_menu_for_editor(&self, name: &str) -> MenuFile {
        let menus_dir = self.menus_dir();
        let stem = menu_file_stem(name);
        let path = menus_dir.join(format!("{stem}.toml"));
        if let Ok(menu) = crate::menu_workspace::loader::load_menu_file(&path) {
            return menu;
        }
        // No valid file: built-in names fall back to their Recovery_Baseline.
        match name.to_ascii_uppercase().as_str() {
            "SETTINGS" => crate::menu_workspace::defaults::recovery_settings_menu(),
            _ => crate::menu_workspace::defaults::recovery_pom_menu(),
        }
    }

    /// Rebuild the editor's available-menus list: POM + Settings + every user
    /// `menus/<name>.toml` on disk (deduplicated, POM/Settings first).
    ///
    /// Validates: menu-workspace Requirement 13.2
    fn refresh_menus_editor_list(&mut self) {
        let mut names = vec!["POM".to_string(), "Settings".to_string()];
        let menus_dir = self.menus_dir();
        if let Ok(entries) = std::fs::read_dir(&menus_dir) {
            let mut user: Vec<String> = entries
                .flatten()
                .filter_map(|e| {
                    let path = e.path();
                    if path.extension().and_then(|x| x.to_str()) != Some("toml") {
                        return None;
                    }
                    let stem = path.file_stem().and_then(|s| s.to_str())?.to_string();
                    // pom/settings are already represented by POM/Settings.
                    if stem.eq_ignore_ascii_case("pom") || stem.eq_ignore_ascii_case("settings") {
                        return None;
                    }
                    Some(stem)
                })
                .collect();
            user.sort();
            names.extend(user);
        }
        self.menus_editor_panel.available = names;
    }
}

/// Map an editor menu name to its file stem: POM -> "pom", Settings ->
/// "settings", else a lowercase-hyphenated slug.
fn menu_file_stem(name: &str) -> String {
    match name.to_ascii_uppercase().as_str() {
        "POM" => "pom".to_string(),
        "SETTINGS" => "settings".to_string(),
        _ => name
            .trim()
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect(),
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 13.8 -- built-in names map to their well-known file
    // stems; user names are slugified.
    #[test]
    fn menu_file_stem_maps_built_ins_and_slugs_users() {
        assert_eq!(menu_file_stem("POM"), "pom");
        assert_eq!(menu_file_stem("pom"), "pom");
        assert_eq!(menu_file_stem("Settings"), "settings");
        assert_eq!(menu_file_stem("My Tools"), "my-tools");
        assert_eq!(menu_file_stem("build_release"), "build-release");
    }
}
