//! Shell-side side effects for the Keys Editor Context (function-keys
//! Requirement 22, CR-CH-029).
//!
//! The panel render is pure (`keys_editor_panel::render`); all side effects
//! (load a kind's key list, serialise+write `keymaps/<kind>.toml`, reset, reload
//! the resolver's context map) live here, mirroring the Menus editor
//! (`apply_menus_editor_action`).

use std::path::PathBuf;

use ff_keys::KeyMap;

use super::WorkbenchShell;
use crate::keys_editor_panel::{rows_to_map, KeysEditorAction};

impl WorkbenchShell {
    /// Resolve the `keymaps/` directory under the User Data Dir. Mirrors
    /// `menus_dir()` / `themes_dir()`: a test override when set, else the real
    /// `<User_Data_Dir>/keymaps/` (falling back to the platform data dir).
    pub(super) fn keymaps_dir(&self) -> PathBuf {
        if let Some(dir) = &self.keymaps_dir_override {
            return dir.clone();
        }
        if let Ok(udd) = ff_session::UserDataDir::resolve(None) {
            return udd.path().join("keymaps");
        }
        dirs::data_dir()
            .map(|base| base.join("FileForgeWorkbench").join("keymaps"))
            .unwrap_or_else(|| PathBuf::from("keymaps"))
    }

    /// Open the Keys Editor Context and populate its state for a kind.
    ///
    /// Navigates the current tab to the Keys editor IN PLACE (Navigation_Stack
    /// push, so END/RETURN return to wherever it was opened from), mirroring the
    /// Theme and Menus editors.
    ///
    /// `initial_kind` pre-selects a workspace kind when given and valid; an
    /// unknown kind leaves the current selection and sets a status message.
    ///
    /// Validates: function-keys-and-history Requirement 22.1, 22.2, 22.5
    pub(super) fn open_keys_editor(&mut self, initial_kind: Option<&str>) {
        // Choose the kind to load: a valid requested kind, else the current
        // selection, else the active workspace's own kind, else "editor".
        let requested = initial_kind
            .map(|k| k.trim().to_lowercase())
            .filter(|k| crate::keys_editor_panel::KIND_NAMES.contains(&k.as_str()));
        let current_ctx = super::helpers::context_name_for_kind(self.tabs.active_tab().kind)
            .filter(|k| crate::keys_editor_panel::KIND_NAMES.contains(k))
            .map(|k| k.to_string());
        let kind = requested
            .clone()
            .or_else(|| self.keys_editor_panel.selected_kind.clone())
            .or(current_ctx)
            .unwrap_or_else(|| "editor".to_string());

        let map = self.load_key_map_for_kind(&kind);
        self.keys_editor_panel.load_kind(&kind, &map);
        if initial_kind.is_some() && requested.is_none() {
            self.keys_editor_panel.error = Some(format!(
                "Unknown workspace kind '{}' -- showing '{kind}'.",
                initial_kind.unwrap_or("").trim()
            ));
        }

        // CR-CH-022: navigate in place (push onto the Navigation_Stack) so END
        // pops back to whatever the editor was opened from.
        self.navigate_to(
            ff_session::session_state::WorkspaceDescriptor::CustomWorkspace {
                workspace_kind: ff_session::session_state::WorkspaceKind::CommandConfigurator,
                params: {
                    let mut p = ff_session::session_state::DescriptorParams::new();
                    p.insert(
                        "editor".to_string(),
                        ff_session::session_state::DescriptorValue::from("keys"),
                    );
                    p
                },
            },
            true,
        );
    }

    /// Load the effective key map for a workspace kind: the `keymaps/<kind>.toml`
    /// override file when present and parseable, else the compiled default map.
    fn load_key_map_for_kind(&self, kind: &str) -> KeyMap {
        let path = self.keymaps_dir().join(format!("{kind}.toml"));
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(table) = toml::from_str::<toml::Table>(&text) {
                let (map, _warnings) = KeyMap::from_toml_table(&table, kind);
                return map;
            }
        }
        KeyMap::default_global()
    }

    /// Apply a [`KeysEditorAction`] produced by the Keys Editor render.
    ///
    /// Validates: function-keys-and-history Requirement 22.2, 22.4
    pub(super) fn apply_keys_editor_action(&mut self, action: KeysEditorAction) {
        use KeysEditorAction as A;
        match action {
            A::None => {}
            A::SelectKind(kind) => {
                let map = self.load_key_map_for_kind(&kind);
                self.keys_editor_panel.load_kind(&kind, &map);
            }
            A::Save => {
                if let Some(kind) = self.keys_editor_panel.selected_kind.clone() {
                    self.save_keys_for_kind(&kind);
                }
            }
            A::Reset => {
                if let Some(kind) = self.keys_editor_panel.selected_kind.clone() {
                    let map = KeyMap::default_global();
                    self.keys_editor_panel.load_kind(&kind, &map);
                }
            }
        }
    }

    /// Serialise the working rows to `keymaps/<kind>.toml` and reload that kind's
    /// context map into the resolver so the new bindings take effect at once.
    ///
    /// Validates: function-keys-and-history Requirement 22.4
    fn save_keys_for_kind(&mut self, kind: &str) {
        let map = rows_to_map(&self.keys_editor_panel.rows, kind);
        let toml_text = key_map_to_toml(&map);
        let dir = self.keymaps_dir();
        if let Err(e) = std::fs::create_dir_all(&dir) {
            self.keys_editor_panel.error = Some(format!("Could not create keymaps dir: {e}"));
            return;
        }
        let path = dir.join(format!("{kind}.toml"));
        if let Err(e) = std::fs::write(&path, toml_text) {
            self.keys_editor_panel.error = Some(format!("Save failed: {e}"));
            return;
        }
        // Reload this kind's context map from the freshly written file so the
        // binding change is live without a relaunch.
        self.key_map_resolver.set_context_map(kind.to_string(), map);
        self.keys_editor_panel.error = Some(format!("Saved keymaps/{kind}.toml"));
    }
}

/// Serialise a [`KeyMap`] to the `keymaps/<kind>.toml` schema: one top-level key
/// per assigned binding, named by its canonical TOML name (`F3`, `SF3`, `CF12`),
/// with a plain-string command value (or a `{ command, label?, description? }`
/// table when metadata is present). Matches `KeyMap::from_toml_table`.
fn key_map_to_toml(map: &KeyMap) -> String {
    let mut table = toml::map::Map::new();
    for (mk, binding) in map.iter() {
        let key_name = mk.toml_name();
        let value = if binding.label().is_some() || binding.description().is_some() {
            let mut entry = toml::map::Map::new();
            entry.insert(
                "command".to_string(),
                toml::Value::String(binding.command().to_string()),
            );
            if let Some(lbl) = binding.label() {
                entry.insert("label".to_string(), toml::Value::String(lbl.to_string()));
            }
            if let Some(desc) = binding.description() {
                entry.insert(
                    "description".to_string(),
                    toml::Value::String(desc.to_string()),
                );
            }
            toml::Value::Table(entry)
        } else {
            toml::Value::String(binding.command().to_string())
        };
        table.insert(key_name, value);
    }
    toml::to_string(&toml::Value::Table(table)).unwrap_or_default()
}
