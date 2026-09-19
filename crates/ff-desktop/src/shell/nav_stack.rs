//! Per-tab Navigation_Stack: uniform in-place navigation and END/RETURN
//! (menu-workspace Req 14, CR-CH-022).
//!
//! Replaces the three former ad-hoc END mechanisms (`pending_return_to_pom`,
//! the Settings `namespace_filter` END branch, and the Menus editor
//! `opened_from_settings` bool). Each tab owns a `nav_stack` of
//! `WorkspaceDescriptor`s (its ancestors, most-recent last). Navigation ALWAYS
//! transforms the current tab in place, pushing the previous Context onto the
//! stack; END pops one level; an empty-stack END closes the Workspace (exits
//! when last, preserving CR-CH-016).

use ff_session::session_state::{
    DescriptorParams, DescriptorValue, WorkspaceDescriptor, WorkspaceKind,
};

use super::WorkbenchShell;
use crate::tab_state::TabKind;

impl WorkbenchShell {
    /// Derive the `WorkspaceDescriptor` for the active tab's CURRENT Context, so
    /// it can be pushed onto the Navigation_Stack. Mirrors the session
    /// descriptor mapping, plus kind-only descriptors for the transient editors
    /// (Theme / Menus) so they can sit on a stack.
    ///
    /// Validates: menu-workspace Requirement 14.6
    pub(super) fn descriptor_for_current_context(&self) -> WorkspaceDescriptor {
        let tab = self.tabs.active_tab();
        let custom =
            |kind: WorkspaceKind, params: DescriptorParams| WorkspaceDescriptor::CustomWorkspace {
                workspace_kind: kind,
                params,
            };
        match tab.kind {
            TabKind::MenuWorkspace => {
                // The Home Context (POM) always maps to `Menu{name:"pom"}`
                // regardless of the loaded menu's title (menu-workspace Req 18.8).
                let name = if tab.is_home {
                    "pom".to_string()
                } else {
                    tab.menu_workspace
                        .as_ref()
                        .and_then(|mw| mw.menu.as_ref())
                        .map(|m| m.title.to_lowercase())
                        .unwrap_or_else(|| "pom".to_string())
                };
                WorkspaceDescriptor::Menu { name }
            }
            TabKind::ConfigPanel => {
                let mut params = DescriptorParams::new();
                if let Some(ns) = self.config_panel.namespace_filter.as_deref() {
                    params.insert("namespace".to_string(), DescriptorValue::from(ns));
                }
                custom(WorkspaceKind::Config, params)
            }
            TabKind::FilesPanel => custom(WorkspaceKind::Files, DescriptorParams::new()),
            TabKind::FileExplorerPanel => {
                custom(WorkspaceKind::FileExplorer, DescriptorParams::new())
            }
            TabKind::SearchResults => custom(WorkspaceKind::Search, DescriptorParams::new()),
            TabKind::PluginManager => custom(WorkspaceKind::PluginManager, DescriptorParams::new()),
            TabKind::EventLog => custom(WorkspaceKind::EventLog, DescriptorParams::new()),
            TabKind::MacroLibrary => custom(WorkspaceKind::MacroLibrary, DescriptorParams::new()),
            TabKind::CommandConfigurator => {
                custom(WorkspaceKind::CommandConfigurator, DescriptorParams::new())
            }
            TabKind::FileEditor | TabKind::Untitled => {
                let mut params = DescriptorParams::new();
                if let Some(uri) = tab.path.as_ref() {
                    params.insert("uri".to_string(), DescriptorValue::from(uri.clone()));
                }
                custom(WorkspaceKind::Editor, params)
            }
            // Transient editors: kind-only descriptor (editing state is
            // shell-global and re-derived on reconstruct).
            TabKind::ThemeEditor => custom(WorkspaceKind::CommandConfigurator, {
                // Reuse a distinct marker param so reconstruct routes to THEMES.
                let mut p = DescriptorParams::new();
                p.insert("editor".to_string(), DescriptorValue::from("theme"));
                p
            }),
            TabKind::MenusEditor => custom(WorkspaceKind::CommandConfigurator, {
                let mut p = DescriptorParams::new();
                p.insert("editor".to_string(), DescriptorValue::from("menus"));
                p
            }),
            TabKind::KeysEditor => custom(WorkspaceKind::CommandConfigurator, {
                let mut p = DescriptorParams::new();
                p.insert("editor".to_string(), DescriptorValue::from("keys"));
                p
            }),
        }
    }

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
    fn set_active_tab_context(&mut self, kind: TabKind, title: &str) {
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
    fn set_active_tab_home(&mut self) {
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
            WorkspaceKind::Files => self.set_active_tab_context(TabKind::FilesPanel, "[FILES]"),
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

    /// Navigate the active tab to `descriptor` IN PLACE. When `push` is true, the
    /// current Context's descriptor is pushed onto the tab's Navigation_Stack
    /// first (a `;` PUSH / bare navigation); when false the current Context is
    /// collapsed (a `.` STOP -- no push).
    ///
    /// Validates: menu-workspace Requirement 14.2, 14.3
    pub(super) fn navigate_to(&mut self, descriptor: WorkspaceDescriptor, push: bool) {
        if push {
            let current = self.descriptor_for_current_context();
            self.tabs.active_tab_mut().nav_stack.push(current);
        }
        self.reconstruct_context(&descriptor);
        // CR-CH-023 Req 16.1a: entering a Workspace context places focus on the
        // command field.
        self.command_field_focus_requested = true;
    }

    /// Convenience: navigate the current tab to a parameterless CustomWorkspace
    /// kind in place (push onto the stack). Used by the simple navigation arms.
    ///
    /// Validates: menu-workspace Requirement 14.2
    pub(super) fn nav_to_kind(&mut self, kind: WorkspaceKind) {
        self.navigate_to(
            WorkspaceDescriptor::CustomWorkspace {
                workspace_kind: kind,
                params: DescriptorParams::new(),
            },
            true,
        );
    }

    /// END: pop one level of the active tab's Navigation_Stack and reconstruct
    /// the parent Context in place. When the stack is empty, close the Workspace
    /// (terminate the app when it is the last open tab, CR-CH-016).
    ///
    /// Validates: menu-workspace Requirement 14.4, 14.5
    pub(super) fn nav_end(&mut self) {
        if let Some(parent) = self.tabs.active_tab_mut().nav_stack.pop() {
            self.reconstruct_context(&parent);
        } else {
            self.close_workspace_or_exit();
        }
    }

    /// RETURN: collapse the active tab's Navigation_Stack to its ROOT Context in
    /// one step. When already at the root (empty stack), behaves as END-at-root.
    ///
    /// Validates: menu-workspace Requirement 14.10 (CR-CH-038)
    pub(super) fn nav_return(&mut self) {
        // CR-CH-038: RETURN targets the POM, not the tab's arbitrary root.
        // - non-POM active tab -> navigate to the Home Context (POM) in one step,
        //   clearing the Navigation_Stack, regardless of stack depth or whether
        //   the workspace was rooted directly (START <arg>). The workspace stays
        //   open, now showing the POM.
        // - POM active tab -> close this one workspace (Option A: one workspace
        //   per RETURN; exit when it is the last, CR-CH-016). This behaviour is
        //   identical in a docked and a Detached_Workspace.
        if self.tabs.active_tab().is_home {
            self.close_workspace_or_exit();
        } else {
            self.tabs.active_tab_mut().nav_stack.clear();
            self.set_active_tab_home();
            // Entering the POM places focus on the command field.
            self.command_field_focus_requested = true;
        }
    }

    /// START: create a NEW Workspace (tab) and root it per the argument form
    /// (menu-workspace Req 14.8/14.9). START is the only tab-creator.
    ///
    /// - empty: new POM tab, empty stack.
    /// - `=<path>`: new POM tab, then drill along `<path>` (POM on the stack).
    /// - `<arg>`: new tab rooted DIRECTLY at the resolved Context (empty stack);
    ///   an unresolved arg leaves the new tab at the POM with a status message.
    ///
    /// Validates: menu-workspace Requirement 14.8, 14.9
    pub(super) fn start_new_workspace(&mut self, arg: &str) {
        // Always begin with a fresh POM tab (the only tab-creation point).
        self.tabs.insert_pom_tab(&self.runtime);
        self.ensure_pom_menu_loaded();
        // CR-CH-023 Req 16.1a: a new Workspace places focus on the command field.
        self.command_field_focus_requested = true;

        let arg = arg.trim();
        if arg.is_empty() {
            return; // START -> POM, empty stack.
        }

        if let Some(path) = arg.strip_prefix('=') {
            // START =<path>: drill from the POM origin. The `=` origin means the
            // POM is the root; a single segment pushes the POM onto the stack.
            self.apply_start_command(path.trim(), true);
            return;
        }

        // START <arg>: root DIRECTLY at the resolved Context (no POM beneath).
        // Navigate with push, then clear the stack so END ends the Workspace.
        self.apply_start_command(arg, true);
        self.tabs.active_tab_mut().nav_stack.clear();
    }

    /// Resolve a START argument to a navigation command and dispatch it on the
    /// (already-created) new tab. Reuses POM option-key resolution and the
    /// standard command routing.
    fn apply_start_command(&mut self, arg: &str, _push: bool) {
        if arg.is_empty() {
            return;
        }
        // Resolve a POM option key (e.g. "0", "S") to its command, else use the
        // argument verbatim as a command (e.g. "Settings", "FILES").
        let resolved = self
            .resolve_pom_option_key(&arg.to_uppercase())
            .unwrap_or_else(|| arg.to_string());
        // Dispatch through handle_command so the navigation arms (which now push
        // + transform in place on the active new tab) do the work.
        self.handle_command(&resolved);
    }

    /// Close the active Workspace (tab); terminate the application when it is the
    /// last open tab (CR-CH-016).
    ///
    /// Validates: menu-workspace Requirement 14.5
    pub(super) fn close_workspace_or_exit(&mut self) {
        if self.tabs.len() <= 1 {
            let result = self
                .dispatch
                .execute_command("file.exit", ff_command::CommandParams::new());
            if let ff_command::CommandResult::Err(e) = result {
                self.open_error = Some(e.to_string());
            }
        } else {
            self.close_current_and_navigate_back();
        }
    }
}
