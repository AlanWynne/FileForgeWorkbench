//! Shell-side side effects for the Kinds Editor Context (workspace-kinds Req 6,
//! CR-NR-090 B.4).
//!
//! The panel render is pure (`kinds_editor_panel::render`); all side effects
//! (load a Kind's config, create a new Kind, serialise + write
//! `workspace-kinds/<name>.toml`, reload the registry) live here, mirroring the
//! Keys editor (`apply_keys_editor_action`).

use std::path::PathBuf;

use super::WorkbenchShell;
use crate::kinds_editor_panel::KindsEditorAction;
use crate::workspace_kind::{BaseKind, KindConfig, KindConfigToml, KindRegistry};

impl WorkbenchShell {
    /// Resolve the `workspace-kinds/` directory. Test override when set, else the
    /// real `<User_Data_Dir>/workspace-kinds/` (falling back to the platform data
    /// dir). Mirrors `keymaps_dir()` / `menus_dir()`.
    pub(super) fn workspace_kinds_dir(&self) -> PathBuf {
        if let Some(dir) = &self.workspace_kinds_dir_override {
            return dir.clone();
        }
        if let Ok(udd) = ff_session::UserDataDir::resolve(None) {
            return udd.path().join("workspace-kinds");
        }
        dirs::data_dir()
            .map(|base| base.join("FileForgeWorkbench").join("workspace-kinds"))
            .unwrap_or_else(|| PathBuf::from("workspace-kinds"))
    }

    /// Open the Kinds Editor Context: refresh the Kind list from the registry,
    /// default the selection, and navigate the current tab to the editor IN
    /// PLACE (Navigation_Stack push, so END/RETURN return to the origin),
    /// mirroring the Keys/Menus/Theme editors.
    ///
    /// Validates: workspace-kinds Requirement 6.1, 6.4
    pub(super) fn open_kinds_editor(&mut self) {
        self.refresh_kinds_editor_list();
        // Default the selection to the active workspace's own Kind, else the
        // first name, the first time the editor opens.
        if self.kinds_editor_panel.selected.is_none() {
            let default_name = {
                let t = self.tabs.active_tab();
                crate::workspace_kind::BuiltinKind::from_tab_kind(t.kind, t.is_home).stable_name()
            };
            let cfg = self.kind_registry.effective(default_name).clone();
            self.kinds_editor_panel.load_kind(default_name, cfg);
        }
        self.navigate_to(
            ff_session::session_state::WorkspaceDescriptor::CustomWorkspace {
                workspace_kind: ff_session::session_state::WorkspaceKind::CommandConfigurator,
                params: {
                    let mut p = ff_session::session_state::DescriptorParams::new();
                    p.insert(
                        "editor".to_string(),
                        ff_session::session_state::DescriptorValue::from("kinds"),
                    );
                    p
                },
            },
            true,
        );
    }

    /// Refresh the editor's Kind-name list from the current registry (built-in
    /// defaults plus any loaded user Kinds), sorted for determinism.
    fn refresh_kinds_editor_list(&mut self) {
        let mut names: Vec<String> = crate::workspace_kind::BuiltinKind::ALL
            .iter()
            .map(|k| k.stable_name().to_string())
            .collect();
        // Include any user-defined names present in the registry (dir scan).
        for cfg in self.load_user_kind_configs() {
            if !names.contains(&cfg.name) {
                names.push(cfg.name);
            }
        }
        names.sort();
        names.dedup();
        self.kinds_editor_panel.kind_names = names;
    }

    /// Load the user Kind configs currently on disk (for listing user Kinds).
    fn load_user_kind_configs(&self) -> Vec<KindConfig> {
        let dir = self.workspace_kinds_dir();
        let mut out = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                    continue;
                }
                if let Ok(text) = std::fs::read_to_string(&path) {
                    if let Ok(t) = toml::from_str::<KindConfigToml>(&text) {
                        out.push(KindConfig::from(t));
                    }
                }
            }
        }
        out
    }

    /// Apply a [`KindsEditorAction`] produced by the Kinds Editor render.
    ///
    /// Validates: workspace-kinds Requirement 6.2, 6.3
    pub(super) fn apply_kinds_editor_action(&mut self, action: KindsEditorAction) {
        use KindsEditorAction as A;
        match action {
            A::None => {}
            A::SelectKind(name) => {
                let cfg = self.kind_registry.effective(&name).clone();
                self.kinds_editor_panel.load_kind(&name, cfg);
            }
            A::NewKind { name, base } => {
                let trimmed = name.trim().to_string();
                if trimmed.is_empty() {
                    self.kinds_editor_panel.error = Some("New Kind needs a name".to_string());
                    return;
                }
                // Seed a copy of the base's default config with the new name and
                // `modelled_on = Builtin(base)` (Req 6.2).
                let mut cfg = KindConfig::builtin_default(base);
                cfg.name = trimmed.clone();
                cfg.modelled_on = BaseKind::Builtin(base);
                self.kinds_editor_panel.new_name.clear();
                self.kinds_editor_panel.load_kind(&trimmed, cfg);
                if !self.kinds_editor_panel.kind_names.contains(&trimmed) {
                    self.kinds_editor_panel.kind_names.push(trimmed);
                    self.kinds_editor_panel.kind_names.sort();
                }
            }
            A::Save => self.save_selected_kind(),
        }
    }

    /// Serialise the working Kind config to `workspace-kinds/<name>.toml` and
    /// reload the registry so the change is live (title / menu bar / key list /
    /// profile-on-open all recompute from the registry).
    ///
    /// Validates: workspace-kinds Requirement 6.3
    fn save_selected_kind(&mut self) {
        let Some(cfg) = self.kinds_editor_panel.working.clone() else {
            self.kinds_editor_panel.error = Some("No Kind selected".to_string());
            return;
        };
        match self.persist_kind_config(&cfg) {
            Ok(()) => {
                self.kinds_editor_panel.error =
                    Some(format!("Saved workspace-kinds/{}.toml", cfg.name));
            }
            Err(e) => self.kinds_editor_panel.error = Some(e),
        }
    }

    /// Serialise `cfg` to `workspace-kinds/<name>.toml` and reload the registry so
    /// the change is live. THE single write+reload seam shared by the Kinds Editor
    /// Save (Req 6.3) and the `COMMAND` command's position change (CR-NR-095
    /// Req 8.11) so both persist through one code path. Returns a human-readable
    /// error string on failure (the caller decides how to surface it).
    pub(super) fn persist_kind_config(&mut self, cfg: &KindConfig) -> Result<(), String> {
        let dir = self.workspace_kinds_dir();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Could not create workspace-kinds dir: {e}"))?;
        let toml_text = toml::to_string(&KindConfigToml::from(cfg))
            .map_err(|e| format!("Serialise failed: {e}"))?;
        let path = dir.join(format!("{}.toml", cfg.name));
        std::fs::write(&path, toml_text).map_err(|e| format!("Save failed: {e}"))?;
        // Reload the registry from the dir so the change is live.
        self.kind_registry = KindRegistry::load(&dir);
        Ok(())
    }

    /// Set the ACTIVE Workspace's Kind `Command_Line_Position` to `position`
    /// (CR-NR-095, Req 8.9/8.10/8.11) and persist it. Resolves the active tab's
    /// Kind stable name (the same seam as `command_line_position_for`), updates
    /// that Kind's effective config's profile, and writes+reloads via
    /// `persist_kind_config` so the change takes effect next frame AND survives a
    /// restart. Because dispatch runs under `with_workspace_context` for a
    /// detached window / split region, the "active" tab is THAT instance's tab
    /// (Req 8.13). One shared seam with the Kinds Editor (Req 8.12).
    pub(super) fn set_active_command_line_position(
        &mut self,
        position: crate::workspace_kind::CommandLinePosition,
    ) {
        let kind_name = {
            let t = self.tabs.active_tab();
            crate::workspace_kind::BuiltinKind::from_tab_kind(t.kind, t.is_home).stable_name()
        };
        let mut cfg = self.kind_registry.effective(kind_name).clone();
        cfg.profile.command_line_position = position;
        if let Err(e) = self.persist_kind_config(&cfg) {
            self.open_error = Some(e);
        } else {
            self.open_error = None;
        }
    }

    /// Toggle the ACTIVE Workspace's Kind `Command_Line_Position` (bare `COMMAND`,
    /// Req 8.9). Reads the current effective position and sets the opposite.
    pub(super) fn toggle_active_command_line_position(&mut self) {
        let current = self.command_line_position_for(self.tabs.active_index());
        self.set_active_command_line_position(current.toggled());
    }
}
