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
    /// The ordered Tab focus ring for the Menus Editor: the shell command line,
    /// then each working option's key/command/description/group fields in order.
    /// The ids MUST match those the panel render assigns
    /// (`menus_editor_panel::render`) so `request_focus` lands on the real
    /// widgets. Returns just the command line when no menu is loaded.
    ///
    /// Validates: menu-workspace Requirement 13.4 (editable fields reachable by Tab).
    pub(super) fn menus_editor_focus_ring(&self) -> Vec<egui::Id> {
        // The shell command line is the first stop; the remainder is the ordered
        // list of EVERY interactive control the panel render captured this frame
        // (menu selector, title, checkboxes, separator selectables, and per
        // option: key/command/description/group/enabled/up/down/delete, then the
        // footer buttons). Capturing the real egui ids in render -- rather than
        // predicting them -- keeps every control keyboard-reachable
        // (accessibility) without fragile id guessing.
        let mut ring = vec![egui::Id::new("command_field_input")];
        ring.extend(self.menus_editor_panel.focus_ids.iter().copied());
        ring
    }

    /// Compute the next focus-ring index for a Tab / Shift+Tab press.
    ///
    /// Pure helper so the Tab-walk order is unit-testable without a live egui
    /// frame (this is where the B054 "double-advance skips every other widget"
    /// bug lived). Given the ring length, the index the walk is anchored on
    /// (the position of the id we last drove focus to, if any), and the shift
    /// state, it returns the next index to focus:
    ///
    /// - Anchored on `Some(i)`: advance +1 (Tab) or -1 (Shift+Tab), wrapping.
    /// - Not anchored (`None`, e.g. first Tab into the editor): enter at index
    ///   0 for Tab, or the last index for Shift+Tab.
    ///
    /// Returns `None` only when the ring is empty (nothing to focus).
    ///
    /// Validates: menu-workspace Requirement 13.4 (every editable field is
    /// reachable in order by Tab, none skipped).
    pub(super) fn next_ring_index(
        ring_len: usize,
        anchor_idx: Option<usize>,
        shift: bool,
    ) -> Option<usize> {
        if ring_len == 0 {
            return None;
        }
        let next = match anchor_idx {
            Some(i) if shift => (i + ring_len - 1) % ring_len,
            Some(i) => (i + 1) % ring_len,
            None if shift => ring_len - 1,
            None => 0,
        };
        Some(next)
    }

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

    /// Walk a ring of `len` widgets by repeatedly applying `next_ring_index`,
    /// anchoring each step on the index the previous step returned (this mirrors
    /// the shell handler which anchors on the id it last drove focus to). Starts
    /// unanchored (`None`) like the first Tab into the editor.
    fn walk(len: usize, shift: bool, presses: usize) -> Vec<usize> {
        let mut anchor: Option<usize> = None;
        let mut visited = Vec::new();
        for _ in 0..presses {
            let idx = WorkbenchShell::next_ring_index(len, anchor, shift)
                .expect("non-empty ring yields an index");
            visited.push(idx);
            anchor = Some(idx);
        }
        visited
    }

    // Validates: menu-workspace Requirement 13.4 -- pressing Tab N times over an
    // N-widget ring visits every index exactly once in order 0,1,..,N-1 then
    // wraps to 0. This is the regression guard for the B054 double-advance bug
    // where every OTHER widget (Title, Group headers, separator Line) was
    // skipped because the walk advanced by two.
    #[test]
    fn tab_walk_visits_every_ring_index_in_order_without_skips() {
        let len = 6;
        let visited = walk(len, false, len + 1);
        assert_eq!(
            visited,
            vec![0, 1, 2, 3, 4, 5, 0],
            "Tab must step through every index once then wrap -- no skips, no double-advance"
        );
    }

    // Validates: menu-workspace Requirement 13.4 -- Shift+Tab walks the ring in
    // reverse, entering at the last index, visiting each once, then wrapping.
    #[test]
    fn shift_tab_walk_visits_every_ring_index_in_reverse_without_skips() {
        let len = 6;
        let visited = walk(len, true, len + 1);
        assert_eq!(
            visited,
            vec![5, 4, 3, 2, 1, 0, 5],
            "Shift+Tab must step backwards through every index once then wrap"
        );
    }

    // Validates: menu-workspace Requirement 13.4 -- a full forward walk of any
    // ring length reaches every distinct index before repeating any (no widget
    // is unreachable). Exhaustive over small ring sizes.
    #[test]
    fn tab_walk_reaches_every_index_for_all_small_ring_sizes() {
        for len in 1..=20 {
            let visited = walk(len, false, len);
            let mut sorted = visited.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(
                sorted.len(),
                len,
                "ring of {len}: a full walk must reach all {len} indices, got {visited:?}"
            );
        }
    }

    // Validates: menu-workspace Requirement 13.4 -- entry behaviour: the first
    // Tab into the editor (unanchored) lands on index 0; the first Shift+Tab
    // lands on the last index.
    #[test]
    fn unanchored_entry_lands_on_first_for_tab_and_last_for_shift_tab() {
        assert_eq!(WorkbenchShell::next_ring_index(5, None, false), Some(0));
        assert_eq!(WorkbenchShell::next_ring_index(5, None, true), Some(4));
    }

    // Validates: menu-workspace Requirement 13.4 -- an empty ring (no menu
    // loaded) yields no target rather than panicking.
    #[test]
    fn empty_ring_yields_no_index() {
        assert_eq!(WorkbenchShell::next_ring_index(0, None, false), None);
        assert_eq!(WorkbenchShell::next_ring_index(0, Some(0), true), None);
    }
}
