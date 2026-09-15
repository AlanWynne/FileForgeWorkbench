//! Shell-side wiring for the Menus Editor Context (menu-workspace Req 13,
//! CR-NR-075).
//!
//! The panel render is pure (`menus_editor_panel::render`); all side effects
//! (validation, serialisation, file writes, list refresh) live here in the
//! command layer, mirroring the Theme editor (`apply_theme_editor_action`).

use super::WorkbenchShell;
use crate::menu_workspace::{MenuFile, MenuOption};
use crate::menus_editor_panel::{MenusEditorAction, OptionField};

impl WorkbenchShell {
    /// Open the Menus Editor Context and populate its state: list editable
    /// menus and load the current selection (or POM) as the working copy.
    ///
    /// On a POM tab, transforms in place (so END/RETURN returns to the POM);
    /// otherwise opens or activates a dedicated tab.
    ///
    /// Validates: menu-workspace Requirement 13.1, 13.2, 13.3
    pub(super) fn open_menus_editor(&mut self) {
        use crate::tab_state::TabKind;
        self.refresh_menus_editor_list();
        // Origin for END (B053): opened from the Settings menu (a MenuWorkspace
        // tab backed by settings.toml) -> END returns to the Settings menu;
        // otherwise END returns to the POM. Mirrors the Settings_Namespace_View
        // one-level-back rule (cw-requirements Req 10.4).
        let from_settings = self.tabs.active_tab().kind == TabKind::MenuWorkspace
            && self
                .tabs
                .active_tab()
                .menu_workspace
                .as_ref()
                .and_then(|mw| mw.menu.as_ref())
                .map(|m| m.title.eq_ignore_ascii_case("Settings"))
                .unwrap_or(false);
        self.menus_editor_panel.opened_from_settings = from_settings;

        // Default the selection to POM the first time the editor opens.
        let selected = self
            .menus_editor_panel
            .selected
            .clone()
            .unwrap_or_else(|| "POM".to_string());
        let menu = self.load_menu_for_editor(&selected);
        self.menus_editor_panel.load_working(&selected, menu);

        if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu
            || self.tabs.active_tab().kind == TabKind::MenuWorkspace
        {
            // Transform in place from the POM or the Settings menu so END can
            // restore the origin on a single tab (no orphaned tab left behind).
            let tab = self.tabs.active_tab_mut();
            tab.kind = TabKind::MenusEditor;
            tab.title = "[MENUS]".to_string();
        } else {
            self.tabs.open_menus_editor_tab(&self.runtime);
        }
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
            A::EditTitle(title) => {
                if let Some(m) = self.menus_editor_panel.working.as_mut() {
                    m.title = title;
                }
            }
            A::SetShowCalendar(v) => {
                if let Some(m) = self.menus_editor_panel.working.as_mut() {
                    m.show_calendar = v;
                }
            }
            A::SetGroupSeparator(sep) => {
                if let Some(m) = self.menus_editor_panel.working.as_mut() {
                    m.group_separator = sep;
                }
            }
            A::SetGroupHeaders(v) => {
                if let Some(m) = self.menus_editor_panel.working.as_mut() {
                    m.group_headers = v;
                }
            }
            A::EditOption {
                index,
                field,
                value,
            } => {
                if let Some(m) = self.menus_editor_panel.working.as_mut() {
                    if let Some(opt) = m.options.get_mut(index) {
                        match field {
                            // Keys are stored uppercase (loader Req 1.2).
                            OptionField::Key => opt.key = value.trim().to_uppercase(),
                            OptionField::Command => opt.command = value,
                            OptionField::Description => opt.description = value,
                            OptionField::Group => {
                                let g = value.trim();
                                opt.group = if g.is_empty() {
                                    None
                                } else {
                                    Some(g.to_string())
                                };
                            }
                        }
                    }
                }
            }
            A::SetOptionEnabled { index, enabled } => {
                if let Some(m) = self.menus_editor_panel.working.as_mut() {
                    if let Some(opt) = m.options.get_mut(index) {
                        opt.enabled = enabled;
                    }
                }
            }
            A::AddOption => {
                if let Some(m) = self.menus_editor_panel.working.as_mut() {
                    m.options.push(MenuOption {
                        key: String::new(),
                        command: String::new(),
                        description: String::new(),
                        enabled: true,
                        group: None,
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
