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
use crate::tab_state::KindTag;

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
        match tab.kind.tag() {
            KindTag::MenuWorkspace => {
                // The Home Context (POM) always maps to `Menu{name:"pom"}`
                // regardless of the loaded menu's title (menu-workspace Req 18.8).
                let name = if tab.is_home {
                    "pom".to_string()
                } else {
                    tab.kind
                        .menu_workspace()
                        .and_then(|mw| mw.menu.as_ref())
                        .map(|m| m.title.to_lowercase())
                        .unwrap_or_else(|| "pom".to_string())
                };
                WorkspaceDescriptor::Menu { name }
            }
            KindTag::ConfigPanel => {
                let mut params = DescriptorParams::new();
                if let Some(ns) = self.config_panel.namespace_filter.as_deref() {
                    params.insert("namespace".to_string(), DescriptorValue::from(ns));
                }
                custom(WorkspaceKind::Config, params)
            }
            KindTag::FilesPanel => custom(WorkspaceKind::Files, DescriptorParams::new()),
            KindTag::FileExplorerPanel => {
                custom(WorkspaceKind::FileExplorer, DescriptorParams::new())
            }
            KindTag::SearchResults => custom(WorkspaceKind::Search, DescriptorParams::new()),
            KindTag::PluginManager => custom(WorkspaceKind::PluginManager, DescriptorParams::new()),
            KindTag::EventLog => custom(WorkspaceKind::EventLog, DescriptorParams::new()),
            KindTag::ScrmViewer => custom(WorkspaceKind::ScrmViewer, DescriptorParams::new()),
            KindTag::MacroLibrary => custom(WorkspaceKind::MacroLibrary, DescriptorParams::new()),
            KindTag::CommandConfigurator => {
                custom(WorkspaceKind::CommandConfigurator, DescriptorParams::new())
            }
            KindTag::FileEditor | KindTag::Untitled => {
                let mut params = DescriptorParams::new();
                if let Some(uri) = tab.path.as_ref() {
                    params.insert("uri".to_string(), DescriptorValue::from(uri.clone()));
                }
                custom(WorkspaceKind::Editor, params)
            }
            // Transient editors: kind-only descriptor (editing state is
            // shell-global and re-derived on reconstruct).
            KindTag::ThemeEditor => custom(WorkspaceKind::CommandConfigurator, {
                // Reuse a distinct marker param so reconstruct routes to THEMES.
                let mut p = DescriptorParams::new();
                p.insert("editor".to_string(), DescriptorValue::from("theme"));
                p
            }),
            KindTag::MenusEditor => custom(WorkspaceKind::CommandConfigurator, {
                let mut p = DescriptorParams::new();
                p.insert("editor".to_string(), DescriptorValue::from("menus"));
                p
            }),
            KindTag::KeysEditor => custom(WorkspaceKind::CommandConfigurator, {
                let mut p = DescriptorParams::new();
                p.insert("editor".to_string(), DescriptorValue::from("keys"));
                p
            }),
            KindTag::KindsEditor => custom(WorkspaceKind::CommandConfigurator, {
                let mut p = DescriptorParams::new();
                p.insert("editor".to_string(), DescriptorValue::from("kinds"));
                p
            }),
            // The Help Context is transient (CR-NR-097): it is not persisted as
            // its own descriptor. If a descriptor is ever requested for it (it
            // should not be, since session persistence skips it), map to the Home
            // Context so a restore lands on the POM rather than an empty tab.
            KindTag::HelpContext => WorkspaceDescriptor::Menu {
                name: "pom".to_string(),
            },
        }
    }

    // The Context-reconstruction method group (reconstruct_context,
    // set_active_tab_context, set_active_tab_home, reconstruct_settings_menu,
    // reconstruct_named_menu, reconstruct_custom) now lives in
    // `nav_reconstruct.rs` as part of the Phase 2 task 2.2 file-size split.

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
        // CR-NR-098 Wave 2 (Req 9.1, 9.5): a Context transition is the single
        // "screen changed" choke point. When automatic capture is enabled, snap
        // the DEPARTING Context before it is reconstructed. This was previously
        // placed AFTER reconstruct_context, where it relied on a stale
        // `menu_workspace` field that survived the kind change; now that the
        // state lives inside `TabKind::MenuWorkspace`, it must run before the
        // kind is replaced.
        self.auto_capture_active_context();
        self.reconstruct_context(&descriptor);
        // CR-CH-023 Req 16.1a: entering a Workspace context places focus on the
        // command field.
        self.focus.command_field_focus_requested = true;
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

    /// RETURN: collapse the active tab's Navigation_Stack to its own
    /// Tab_Visual_Root (the bottom of its stack) in one step. When already at the
    /// root (empty stack), close the Workspace (exit when last, CR-CH-016).
    ///
    /// CR-CH-052 (supersedes CR-CH-038): RETURN targets the TAB'S VISUAL ROOT,
    /// not the global POM. The Tab_Visual_Root is the context the tab was STARTed
    /// at (POM by default, or `START <ctx>`), i.e. `nav_stack[0]`. RETURN now
    /// converges with bare `X` on a non-empty stack. The `=` family remains the
    /// only way to reinitialise the stack to the POM (FFCMD_Root).
    ///
    /// Validates: menu-workspace Requirement 14.10 (revised, CR-CH-052)
    pub(super) fn nav_return(&mut self) {
        if self.tabs.active_tab().nav_stack.is_empty() {
            self.close_workspace_or_exit();
        } else {
            self.nav_collapse_to_visual_root();
        }
    }

    /// Collapse the active tab's Navigation_Stack to its Tab_Visual_Root (the
    /// bottom of the stack, `nav_stack[0]`) in one step: reconstruct that root
    /// Context in place, then clear the stack. A no-op when the stack is already
    /// empty (the tab is already AT its visual root). Shared by bare `X` and
    /// RETURN on a non-empty stack (CR-CH-052).
    ///
    /// Validates: menu-workspace Requirement 14.10, 14.13 (revised, CR-CH-052)
    pub(super) fn nav_collapse_to_visual_root(&mut self) {
        let root = self.tabs.active_tab().nav_stack.first().cloned();
        if let Some(root) = root {
            self.reconstruct_context(&root);
            self.tabs.active_tab_mut().nav_stack.clear();
            // Match nav_return's focus behaviour: entering the root Context places
            // focus on the command field.
            self.focus.command_field_focus_requested = true;
        }
    }

    /// Uniform bare `X` (CR-CH-052): the FFCMD close verb. When the active tab's
    /// Navigation_Stack is non-empty, collapse to the Tab_Visual_Root in one
    /// action; when empty (already at the root), close the Workspace (exit when
    /// it is the last tab, CR-CH-016). Identical for EVERY context including the
    /// POM -- there is NO POM special-casing. (The editor Command_Environment
    /// claims bare `X` as EXCLUDE first, so uniform `X` is not reached there;
    /// the `=X` escape hatch still routes to this verb at the POM root.)
    ///
    /// Validates: menu-workspace Requirement 14.13, 14.14 (CR-CH-052)
    pub(super) fn nav_x(&mut self) {
        if self.tabs.active_tab().nav_stack.is_empty() {
            self.close_workspace_or_exit();
        } else {
            self.nav_collapse_to_visual_root();
        }
    }

    /// CR-CH-052 front-door `=` step: reinitialise the active tab's
    /// Navigation_Stack to the POM (FFCMD_Root). Clears the stack, then
    /// reconstructs the Home Context (POM) in place via `set_active_tab_home`
    /// (which re-seeds pom.toml / the barebones fallback). After this the
    /// stripped remainder of the `=`-command runs against FFCMD from the POM, so
    /// e.g. `=1` resolves as POM option 1 from ANY tab.
    ///
    /// Validates: command-framework Requirement 10.2, 10.14 (revised, CR-CH-052)
    pub(super) fn reinitialise_active_tab_to_pom(&mut self) {
        self.tabs.active_tab_mut().nav_stack.clear();
        self.set_active_tab_home();
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
        self.focus.command_field_focus_requested = true;

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
        // CR-CH-052 (B080 reroute): dispatch through the single front door
        // `dispatch_command_string` (not `handle_command`) so an in-scope START
        // argument verb reaches `resolve_target` and the CustomWorkspace dispatch
        // arm -- the same shell open method as the (now-deleted) ladder arm --
        // navigating + transforming in place on the active new tab. The resolved
        // argument is already `=`-stripped by `start_new_workspace`, so the front
        // door's `=` step is not re-triggered here.
        self.dispatch_command_string(&resolved);
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
