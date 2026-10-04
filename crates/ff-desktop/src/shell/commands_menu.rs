//! # Shell Command Dispatch -- Menu resolution and opening
//!
//! Menu-name resolution, Option_Key lookup, chained fastpath, and the
//! menu-opening commands (POM / Settings / Config). Split out of `commands.rs`
//! (TASK 2.2, pure code movement, no behaviour change). All methods are on
//! `WorkbenchShell` and keep their exact signatures and visibility.

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Open the Search Results panel, or focus it if already open.
    ///
    /// Validates: global-search Requirement 1.1, 1.3
    pub(super) fn open_or_focus_search_panel(&mut self) {
        use crate::tab_state::TabKind;
        for i in 0..self.tabs.len() {
            if self.tabs.tabs()[i].kind == TabKind::SearchResults {
                self.tabs.set_active(i);
                return;
            }
        }
        self.tabs.open_search_results_tab(&self.runtime);
    }

    /// Resolve the `menus/` directory under the User Data Dir.
    ///
    /// Falls back to `dirs::data_dir()/FileForgeWorkbench/menus` so the command
    /// still resolves a path even when the session layer is unavailable.
    pub(super) fn menus_dir(&self) -> std::path::PathBuf {
        // Test override (menu-workspace Req 13, CR-NR-075): isolates Menus editor
        // file operations to a TempDir. Production leaves this None.
        if let Some(dir) = &self.dir_overrides.menus {
            return dir.clone();
        }
        if let Ok(udd) = ff_session::UserDataDir::resolve(None) {
            return udd.path().join("menus");
        }
        dirs::data_dir()
            .map(|base| base.join("FileForgeWorkbench").join("menus"))
            .unwrap_or_else(|| std::path::PathBuf::from("menus"))
    }
    /// Open a resolved menu by name, honouring the built-in special cases
    /// (B075). This is the SINGLE routing point shared by the typed menu-name
    /// path (`try_menu_name_dispatch`) and the menu-option CLICK / command-target
    /// path (`dispatch_command_target`'s `Menu` arm), so a clicked option and a
    /// typed command open the SAME workspace the SAME way:
    ///
    /// - `settings` -> `open_settings_menu()` (navigate IN PLACE via the
    ///   Navigation_Stack, reconstructing the Settings menu on the current tab),
    /// - `pom` -> the Home Context,
    /// - any other name -> the generic `open_menu_by_name`.
    ///
    /// Before B075 the click path called `open_menu_by_name("settings")` directly,
    /// which opened a GENERIC new menu tab instead of the in-place Settings menu.
    ///
    /// Open (or return to) a menu by name.
    ///
    /// This is the single menu-OPENING command: it OWNS the in-place-vs-new-tab
    /// placement for each menu (CR-CH-043, menu-workspace Req 19.5,
    /// command-framework Req 14.3/14.4). An empty name or `POM` returns to the
    /// Home Context in place; `SETTINGS` navigates the current Workspace to the
    /// Settings menu in place (the B075 behaviour, now owned here rather than by
    /// a dispatcher-level name router); any other name opens the data-driven
    /// Menu_Workspace backed by `menus/<name>.toml` in a new tab, with a missing
    /// file shown in the load-error state. Because placement lives HERE, the
    /// menu-option CLICK seam, the typed menu-name path, and the
    /// `CommandTarget::Menu` dispatch all route through this one command and get
    /// identical placement -- no separate `open_named_menu` router.
    ///
    /// Validates: menu-workspace Requirement 10.4, 11.1, 11.2, 11.4, 11.5, 19.5;
    /// command-framework Requirement 14.3, 14.4; B075
    pub(super) fn open_menu_by_name(&mut self, name: &str) {
        let lower = name.trim().to_lowercase();
        // Req 11.1 / 11.2: bare MENU and MENU POM go to the Home Context.
        if lower.is_empty() || lower == "pom" {
            if !self.tabs.active_tab().is_home {
                self.tabs.insert_pom_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }
        // CR-CH-043 (Req 19.5, B075): the Settings menu navigates IN PLACE. This
        // command owns that effect; the launching menu / affordance does not.
        if lower == "settings" {
            self.open_settings_menu();
            return;
        }
        // Req 11.2: open menus/<name>.toml (SETTINGS -> settings.toml by file name).
        let menus_dir = self.menus_dir();
        let limits = crate::menu_workspace::loader::option_limits_from_config(&self.config_handle);
        self.tabs
            .open_menu_workspace_tab(&lower, &menus_dir, limits, &self.runtime);
        // Surface the load-error message when the backing file is missing (11.4).
        if let Some(mw) = self.tabs.active_tab().menu_workspace.as_ref() {
            self.open_error = mw.load_error.clone();
        } else {
            self.open_error = None;
        }
    }

    /// Stage 1 of the command-resolution chain (CR-CH-025, command-framework
    /// Req 8.3): when the active Workspace is a Menu_Workspace and `cmd` matches
    /// an Option_Key of the CURRENT menu, activate that option and return `true`.
    /// Returns `false` when the active tab is not a menu OR the token is not an
    /// Option_Key of it, so the caller falls through to the rest of the chain
    /// (menu-workspace Req 3.6 -- a non-matching token is NOT a terminal error).
    ///
    /// Validates: menu-workspace Requirement 3.1, 3.6, 10.1, 10.3, 10.6
    pub(super) fn try_current_menu_option(&mut self, cmd: &str) -> bool {
        if self.tabs.active_tab().kind != crate::tab_state::TabKind::MenuWorkspace {
            return false;
        }
        // CR-CH-043 (menu-workspace Req 19.2): this is the ONE current-menu
        // Option_Key resolver, applied to the ACTIVE menu regardless of
        // `is_home`. The Home Context (POM) is a MenuWorkspace (CR-NR-082 Slice
        // 1), so a bare Option_Key typed while the POM is active is resolved
        // HERE, exactly as for the Settings menu or any user menu -- there is no
        // longer a separate POM-only resolver at this stage. (The
        // `resolve_pom_option_key` path remains for the Navigation_Origin `=`
        // fastpath -- e.g. `=0.K` chains resolving against the POM from ANOTHER
        // workspace -- which is a distinct concern from the active-menu lookup.)
        //
        // Extract the option's inline target + command (clone what we need)
        // without holding the borrow, then activate it through the single
        // Option-Selection path shared with the click seam.
        // Validates: menu-workspace Requirement 3.1, 18.4, 19.1, 19.2
        let resolved: Option<(Option<ff_command::CommandTarget>, String, String)> = self
            .tabs
            .active_tab()
            .menu_workspace
            .as_ref()
            .and_then(|mw| mw.menu.as_ref())
            .and_then(|menu| {
                crate::menu_workspace::commands::find_option(cmd.trim(), menu)
                    .ok()
                    .map(|opt| (opt.target.clone(), opt.command.clone(), opt.key.clone()))
            });
        let Some((target, option_cmd, option_key)) = resolved else {
            // Not an Option_Key of this menu: fall through (Req 3.6). A disabled
            // option (find_option Err) also falls through; the chain will report
            // an unresolved command if nothing else matches.
            return false;
        };
        // Guard against a self-referential loop: an option whose command is its
        // own key (e.g. a menu with `key = "X"`, `command = "X"`) would recurse
        // into this same resolver forever. When the resolved command equals the
        // option key, treat it as "no command command" and fall through so a
        // built-in / menu-name stage can claim it instead. (Mirrors the guard
        // the POM fastpath applied.)
        if target.is_none() && option_cmd.trim().eq_ignore_ascii_case(option_key.trim()) {
            return false;
        }
        self.activate_menu_option(target.as_ref(), &option_cmd);
        true
    }

    /// The single Option-Selection activation step (CR-CH-043, menu-workspace
    /// Req 19.1/19.3/19.4; command-framework Req 14.1). Given a selected menu
    /// option's inline `target` (if any) and its `command` string, execute it:
    /// an inline `[options.target]` (Req 10.6) is dispatched through the command
    /// pipeline; otherwise the option's command string is resolved-and-dispatched
    /// (falling through to `handle_command`). This is shared by the current-menu
    /// Option_Key resolver (typed / Tab+Enter) and the option-CLICK seam so that
    /// selecting an option is observably identical to executing its command --
    /// the Menu Workspace is a dumb dispatcher and does not choose placement.
    pub(super) fn activate_menu_option(
        &mut self,
        target: Option<&ff_command::CommandTarget>,
        command: &str,
    ) {
        // Req 10.6: an inline [options.target] wins over `command`.
        if let Some(target) = target {
            self.dispatch_command_target(target);
            return;
        }
        // Req 10.1/10.3: resolve the option's command to a user-owned target and
        // dispatch it; otherwise handle the raw command string (Req 10.2) through
        // `handle_command`, which (as of B080 Step 2) runs the stage-1 Option_Key
        // lookup, the fastpaths, and `resolve_target` in the single ordered
        // chain, so an Option_Command that is itself an in-scope built-in verb or
        // a chained path resolves the SAME way a typed command would (D7/D8).
        match self.resolve_and_dispatch_command(command) {
            super::target_dispatch::ResolveOutcome::Dispatched => {}
            super::target_dispatch::ResolveOutcome::FallThrough => {
                self.handle_command(command);
            }
        }
    }

    /// Stage 3 of the command-resolution chain (CR-CH-025, command-framework
    /// Req 8.11, menu-workspace Req 11.11): when the FIRST token of `cmd` names a
    /// resolvable menu (user `menus/<name>.toml` or built-in `pom`/`settings`),
    /// open that Menu_Workspace -- forwarding a trailing token as an Option_Key
    /// (Req 11.7) -- and return `true`. Returns `false` when the first token is
    /// not a resolvable menu name, so the caller falls through to the error
    /// stage. Built-in commands are matched earlier in `handle_command`, so a
    /// built-in always shadows a same-named menu (Req 8.10 / 11.12).
    pub(super) fn try_menu_name_dispatch(&mut self, cmd: &str) -> bool {
        let mut tokens = cmd.split_whitespace();
        let Some(first) = tokens.next() else {
            return false;
        };
        let rest: String = tokens.collect::<Vec<_>>().join(" ");
        // Ask the shared resolver whether the first token names a menu.
        let resolver = crate::command_config::ShellTargetResolver::new(
            &self.command_store.definitions,
            &self.cmd_registry,
            self.menus_dir(),
        );
        let name = match ff_command::TargetResolver::menu_name_target(&resolver, first) {
            Some(ff_command::CommandTarget::Menu { name }) => name,
            _ => return false,
        };
        // Open the named menu through the single menu-opening command, which
        // OWNS per-menu placement (pom/settings in place, others new tab --
        // CR-CH-043 Req 19.5). Shared with the menu-option CLICK seam and the
        // `CommandTarget::Menu` dispatch, so all three get identical placement
        // with no dispatcher-level name router (replaces `open_named_menu`).
        self.open_menu_by_name(&name);
        // Trailing token: activate the option keyed by it on the now-open menu
        // (Req 11.7). Re-dispatch through `handle_command` so it hits the stage-1
        // Option_Key lookup (the single ordered chain, B080 Step 2).
        let trailing = rest.trim();
        if !trailing.is_empty() {
            self.handle_command(trailing);
        }
        true
    }

    /// Open the Settings_Menu -- the data-driven Menu_Workspace backed by
    /// `menus/settings.toml`.
    ///
    /// Transforms the active POM tab in place when opened from the Home Context
    /// (so F3/END returns to the POM), otherwise opens a dedicated tab.
    ///
    /// Validates: cw-requirements.md Requirement 9.1; configuration-system
    /// Requirement 15.1
    pub(super) fn open_settings_menu(&mut self) {
        // CR-CH-022 Req 14.2: navigate the current tab to the Settings menu in
        // place (push onto the Navigation_Stack); never a new tab. The
        // reconstruct path (nav_stack.rs) loads settings.toml with the compiled
        // Recovery_Baseline fallback (CR-CH-021 Req 12.4/12.5).
        self.navigate_to(
            ff_session::session_state::WorkspaceDescriptor::Menu {
                name: "settings".to_string(),
            },
            true,
        );
        // Surface a non-blocking notice when a present settings.toml failed to
        // parse (a bypassed user file), matching the prior behaviour.
        let idx = self.tabs.active_index();
        if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
            if let Some(mw) = tab.menu_workspace.as_ref() {
                if let Some(err) = mw
                    .load_error
                    .as_deref()
                    .filter(|e| e.starts_with("Menu file error"))
                    .map(str::to_string)
                {
                    self.notify_menu_fallback("settings.toml", &err);
                }
            }
        }
        self.open_error = None;
    }

    /// Push a non-blocking notice that a user Menu_File was bypassed in favour of
    /// the compiled Recovery_Baseline because it failed to parse (CR-CH-021
    /// Req 12.5; startup-and-session Req 11.8).
    pub(super) fn notify_menu_fallback(&self, file: &str, reason: &str) {
        use crate::notification::{Notification, NotificationLevel};
        if let Ok(mut queue) = self.notification_queue.lock() {
            queue.push(Notification::new(
                NotificationLevel::Warning,
                format!("{file}: using built-in menu"),
                Some(format!("{reason} -- your {file} was bypassed.")),
            ));
        }
    }

    /// Open the Config panel (flat config-key browser), optionally filtered to a
    /// namespace (the `CONFIG [<namespace>]` command).
    ///
    /// When `namespace` is `Some(ns)` the flat-list filter is pre-populated
    /// with `<ns>.` and the tab title becomes `[CONFIG:<ns>]`. When `None`
    /// the unfiltered all-keys Config view is shown with title `[CONFIG]`.
    ///
    /// Validates: cw-requirements.md Requirement 10.1, 10.2, 10.5, 9.4
    pub(super) fn open_config_view(&mut self, namespace: Option<String>) {
        // CR-CH-022 Req 14.2 + cw-requirements Req 10.4: navigate the current tab
        // in place. For a NAMESPACE view, the parent on the Navigation_Stack must
        // be the Settings MENU (so END returns to the menu, not straight to the
        // grandparent). Achieve this by first navigating to the Settings menu
        // (pushing the current Context), then to the namespace view (pushing the
        // Settings menu). The unfiltered All-Settings view (namespace None) is a
        // single hop from the current Context.
        if let Some(ns) = &namespace {
            // Only insert the Settings-menu parent when we are not already on it
            // (avoid a redundant [Settings, Settings] pair when opened via the
            // Settings menu option).
            let already_on_settings_menu = self.tabs.active_tab().kind
                == crate::tab_state::TabKind::MenuWorkspace
                && self
                    .tabs
                    .active_tab()
                    .menu_workspace
                    .as_ref()
                    .and_then(|mw| mw.menu.as_ref())
                    .map(|m| m.title.eq_ignore_ascii_case("Settings"))
                    .unwrap_or(false);
            if !already_on_settings_menu {
                self.open_settings_menu();
            }
            self.config_panel.filter = format!("{ns}.");
            self.config_panel.namespace_filter = Some(ns.clone());
            let mut params = ff_session::session_state::DescriptorParams::new();
            params.insert(
                "namespace".to_string(),
                ff_session::session_state::DescriptorValue::from(ns.as_str()),
            );
            self.navigate_to(
                ff_session::session_state::WorkspaceDescriptor::CustomWorkspace {
                    workspace_kind: ff_session::session_state::WorkspaceKind::Config,
                    params,
                },
                true,
            );
        } else {
            self.config_panel.filter = String::new();
            self.config_panel.namespace_filter = None;
            self.navigate_to(
                ff_session::session_state::WorkspaceDescriptor::CustomWorkspace {
                    workspace_kind: ff_session::session_state::WorkspaceKind::Config,
                    params: ff_session::session_state::DescriptorParams::new(),
                },
                true,
            );
        }
    }
}
