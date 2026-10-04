//! Navigation_Stack Context reconstruction helpers (menu-workspace Req 14,
//! CR-CH-022). Moved out of `nav_stack.rs` verbatim as part of the Phase 2 task
//! 2.2 file-size split; behaviour, method names, signatures, and visibility are
//! unchanged. These `impl WorkbenchShell` methods re-derive a Context on the
//! ACTIVE tab in place (no new tab) from a `WorkspaceDescriptor`.

use ff_session::session_state::{
    DescriptorParams, DescriptorValue, WorkspaceDescriptor, WorkspaceKind,
};

use super::WorkbenchShell;
use crate::tab_state::TabKind;

impl WorkbenchShell {
    /// Reconstruct a `WorkspaceDescriptor` as the active tab's Context, IN PLACE
    /// (no new tab). Re-derives shell-global Context state exactly as opening the
    /// Context does.
    ///
    /// Validates: menu-workspace Requirement 14.4, 14.6
    pub(super) fn reconstruct_context(&mut self, descriptor: &WorkspaceDescriptor) {
        match descriptor {
            WorkspaceDescriptor::Menu { name } => {
                // POM is the Home Context; any other name is a Menu_Workspace.
                if name.eq_ignore_ascii_case("pom") {
                    self.set_active_tab_home();
                } else if name.eq_ignore_ascii_case("settings") {
                    self.reconstruct_settings_menu();
                } else {
                    self.reconstruct_named_menu(name);
                }
            }
            WorkspaceDescriptor::CustomWorkspace {
                workspace_kind,
                params,
            } => self.reconstruct_custom(workspace_kind.clone(), params),
        }
    }

    /// Set the active tab's kind + title in place (no push, no new tab).
    pub(super) fn set_active_tab_context(&mut self, kind: TabKind, title: &str) {
        let tab = self.tabs.active_tab_mut();
        tab.kind = kind;
        tab.title = title.to_string();
        // Only the dedicated Home reconstruction sets `is_home`; any other
        // Context reset clears it so a former Home tab becomes a plain Context.
        tab.is_home = false;
    }

    /// Reconstruct the Home Context (POM) on the active tab in place: a
    /// `MenuWorkspace` tab flagged `is_home`, with the menu cleared so
    /// `ensure_pom_menu_loaded` re-seeds `pom.toml` (or the barebones fallback).
    ///
    /// Validates: menu-workspace Requirement 18.2, 18.4
    pub(super) fn set_active_tab_home(&mut self) {
        {
            let tab = self.tabs.active_tab_mut();
            tab.kind = TabKind::MenuWorkspace;
            tab.title = "[POM]".to_string();
            tab.is_home = true;
            tab.menu_workspace = None;
        }
        self.ensure_pom_menu_loaded();
    }

    /// Reconstruct the Settings menu (Menu_Workspace backed by settings.toml,
    /// with the compiled fallback) on the active tab in place.
    fn reconstruct_settings_menu(&mut self) {
        let menus_dir = self.menus_dir();
        let limits = crate::menu_workspace::loader::option_limits_from_config(&self.config_handle);
        let file_path = menus_dir.join("settings.toml");
        let mut mw =
            crate::menu_workspace::MenuWorkspaceState::load_with_limits(&file_path, limits);
        if mw.menu.is_none() {
            // Fall back to the compiled Recovery_Baseline (CR-CH-021 Req 12.4/
            // 12.5). Retain a PARSE error in load_error so the caller
            // (open_settings_menu) can surface a bypassed-file notice; clear a
            // mere "not found" (absent file is silent).
            let keep_parse_error = mw
                .load_error
                .as_deref()
                .map(|e| e.starts_with("Menu file error"))
                .unwrap_or(false);
            mw.menu = Some(crate::menu_workspace::defaults::recovery_settings_menu());
            if !keep_parse_error {
                mw.load_error = None;
            }
        }
        let tab = self.tabs.active_tab_mut();
        tab.kind = TabKind::MenuWorkspace;
        tab.title = "[SETTINGS]".to_string();
        tab.is_home = false;
        tab.menu_workspace = Some(mw);
    }

    /// Reconstruct a named Menu_Workspace on the active tab in place.
    fn reconstruct_named_menu(&mut self, name: &str) {
        let menus_dir = self.menus_dir();
        let limits = crate::menu_workspace::loader::option_limits_from_config(&self.config_handle);
        let file_path = menus_dir.join(format!("{name}.toml"));
        let mw = crate::menu_workspace::MenuWorkspaceState::load_with_limits(&file_path, limits);
        let title = mw.tab_title();
        let tab = self.tabs.active_tab_mut();
        tab.kind = TabKind::MenuWorkspace;
        tab.title = title;
        tab.is_home = false;
        tab.menu_workspace = Some(mw);
    }

    /// Reconstruct a CustomWorkspace kind on the active tab in place, re-deriving
    /// any shell-global state from params.
    fn reconstruct_custom(&mut self, kind: WorkspaceKind, params: &DescriptorParams) {
        match kind {
            // Legacy sessions persisted the POM as this custom kind; route it to
            // the unified Home Context (menu-workspace Req 18.8).
            WorkspaceKind::PrimaryOptionMenu => {
                self.set_active_tab_home();
            }
            WorkspaceKind::Config => {
                let namespace = match params.get("namespace") {
                    Some(DescriptorValue::String(ns)) => Some(ns.clone()),
                    _ => None,
                };
                let title = match &namespace {
                    Some(ns) => format!("[CONFIG:{ns}]"),
                    None => "[CONFIG]".to_string(),
                };
                self.config_panel.filter = match &namespace {
                    Some(ns) => format!("{ns}."),
                    None => String::new(),
                };
                self.config_panel.namespace_filter = namespace;
                self.set_active_tab_context(TabKind::ConfigPanel, &title);
            }
            WorkspaceKind::Files => self.set_active_tab_context(TabKind::FilesPanel, "[CATALOGS]"),
            WorkspaceKind::FileExplorer => {
                self.set_active_tab_context(TabKind::FileExplorerPanel, "[FILES]")
            }
            WorkspaceKind::Search => {
                self.set_active_tab_context(TabKind::SearchResults, "[SEARCH]")
            }
            WorkspaceKind::PluginManager => {
                self.set_active_tab_context(TabKind::PluginManager, "[PLUGINS]")
            }
            WorkspaceKind::EventLog => self.set_active_tab_context(TabKind::EventLog, "[LOG]"),
            WorkspaceKind::ScrmViewer => {
                self.set_active_tab_context(TabKind::ScrmViewer, "[REPLAY]")
            }
            WorkspaceKind::MacroLibrary => {
                self.set_active_tab_context(TabKind::MacroLibrary, "[MACROS]")
            }
            WorkspaceKind::CommandConfigurator => {
                // The transient editors are encoded as CommandConfigurator with an
                // `editor` marker param (see descriptor_for_current_context). Set
                // the kind/title directly; the editors' shell-global working
                // state is still present, so no re-open (which would re-navigate)
                // is needed.
                match params.get("editor") {
                    Some(DescriptorValue::String(e)) if e == "theme" => {
                        self.set_active_tab_context(TabKind::ThemeEditor, "[THEME]")
                    }
                    Some(DescriptorValue::String(e)) if e == "menus" => {
                        self.set_active_tab_context(TabKind::MenusEditor, "[MENUS]")
                    }
                    Some(DescriptorValue::String(e)) if e == "keys" => {
                        self.set_active_tab_context(TabKind::KeysEditor, "[KEYS]")
                    }
                    Some(DescriptorValue::String(e)) if e == "kinds" => {
                        self.set_active_tab_context(TabKind::KindsEditor, "[KINDS]")
                    }
                    _ => self.set_active_tab_context(TabKind::CommandConfigurator, "[COMMANDS]"),
                }
            }
            WorkspaceKind::Editor | WorkspaceKind::Untitled => {
                // An editor Context cannot be reconstructed without its document;
                // fall back to the POM rather than leave a blank editor.
                self.set_active_tab_home();
            }
            // WorkspaceKind is #[non_exhaustive]; any future kind falls back to
            // the POM so navigation never lands on an unhandled Context.
            _ => {
                self.set_active_tab_home();
            }
        }
    }
}
