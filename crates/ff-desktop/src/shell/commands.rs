//! # Shell Command Dispatch
//!
//! `handle_command()` — parses and routes every primary command entered in the
//! Command ===> field or dispatched programmatically.

use ff_command::{CommandParams, CommandResult};
use ff_command_semantics::StatusKind;
use ff_edit_operations::ProfileError;
use ff_help::{ContextDetector, EditorContext, EditorMode, HelpTopicRegistry};
use ff_keys::RetrieveResult;

use crate::tab_state::TabKind;

use super::helpers::*;

impl WorkbenchShell {
    /// Close the current tab and navigate to the tab that was active immediately
    /// before it was opened (the top of `tab_history`), clamped to the remaining
    /// range. Shared by END from any Context, including a POM when other
    /// Workspaces remain open (Requirement 17.1, 17.2).
    ///
    /// `TabManager::close_tab` keeps at least one tab open, so callers must
    /// decide the last-Workspace case (terminate) before calling this.
    pub(super) fn close_current_and_navigate_back(&mut self) {
        let current = self.tabs.active_index();
        self.tabs.close_tab(current);
        if let Some(prev) = self.tab_history.pop() {
            let clamped = prev.min(self.tabs.len().saturating_sub(1));
            self.tabs.set_active(clamped);
        }
    }

    pub(super) fn handle_command(&mut self, cmd: &str) {
        let upper = cmd.trim().to_uppercase();

        // Record every submitted command in the RETRIEVE history exactly once,
        // BEFORE the branch handlers run, so that shell-intercept commands
        // (THEME, CAPS, NULLS, STATS, LOCK, ...) are recallable via F12 just like
        // command-engine commands. Previously each handler had to remember to
        // call `cmd_history.add`, and the intercepts did not -- so e.g.
        // `THEME legacy` could not be retrieved (only engine-routed commands
        // like `LOCATE 1` were). RETRIEVE itself is excluded (it is the recall
        // action, not a recallable command); empty input is skipped by
        // `CommandHistory::add`, which also de-duplicates.
        if upper != "RETRIEVE" {
            self.cmd_history.add(cmd);
        }

        // ── Shell-level intercepts ───────────────────────────────────────
        if upper == "EXIT" || upper == "QUIT" || upper == "=X" || upper == "X" || upper == "LOGOFF"
        {
            // Validates: Requirement 20.3 -- LOGOFF is an alias for EXIT
            if upper == "LOGOFF" {
                let msg = self.format_logoff_message();
                self.open_error = Some(msg);
            }
            let result = self
                .dispatch
                .execute_command("file.exit", CommandParams::new());
            if let CommandResult::Err(e) = result {
                self.open_error = Some(e.to_string());
            }
            return;
        }

        if upper.starts_with("EDIT") && (upper == "EDIT" || upper.starts_with("EDIT ")) {
            let rest = cmd.trim().split_once(' ').map(|x| x.1.trim()).unwrap_or("");
            if rest.is_empty() {
                self.open_error = Some("EDIT requires a file path".to_string());
            } else {
                let mut p = CommandParams::new();
                p.insert("path", rest);
                let result = self.dispatch.execute_command("file.open", p);
                if let CommandResult::Err(e) = result {
                    self.open_error = Some(e.to_string());
                } else {
                    self.open_error = None;
                }
            }
            return;
        }

        if upper == "START" || upper == "POM" {
            // Validates: Requirement 14.10, 14.14 — START/POM opens a new POM tab
            self.tabs.insert_pom_tab(&self.runtime);
            self.open_error = None;
            return;
        }

        // MENU / MENU <name> and the menu.open Command_ID form.
        // Validates: menu-workspace Requirement 11.1, 11.2, 11.4, 11.6
        if upper == "MENU" || upper.starts_with("MENU ") {
            let arg = cmd.trim()[4..].trim();
            self.open_menu_by_name(arg);
            return;
        }
        if upper == "MENU.OPEN" || upper.starts_with("MENU.OPEN ") {
            let arg = cmd.trim()[9..].trim();
            self.open_menu_by_name(arg);
            return;
        }

        if upper == "CLOSE" {
            // Validates: Requirement 14.11 — CLOSE closes the current tab
            let idx = self.tabs.active_index();
            self.tabs.close_tab(idx);
            self.open_error = None;
            return;
        }

        // ── HELP / F1 fallback — Validates: Requirement 18.1, 18.2 ————————
        if upper == "HELP" {
            let registry = HelpTopicRegistry::new(); // empty registry — no topics loaded yet
            let ctx = EditorContext {
                command_line_text: self.command_text.clone(),
                command_line_has_focus: true,
                prefix_area_text: None,
                prefix_area_has_focus: false,
                active_mode: EditorMode::Edit,
                help_panel_open: false,
                current_help_topic: None,
            };
            if let Err(msg) = ContextDetector::resolve_with_fallback(&ctx, &registry) {
                self.open_error = Some(msg);
            }
            return;
        }

        // ── KEYS -- Validates: Requirement 20.1, CX Requirement 2.1-2.4 ──────
        if upper == "KEYS" {
            self.key_config_dialog.open = true;
            self.key_config_dialog.initial_scope = None;
            self.open_error = None;
            return;
        }
        if upper.starts_with("KEYS ") {
            // Validates: CX Requirement 2.2, 2.3, 2.4
            let name = cmd.trim()[5..].trim().to_lowercase();
            self.key_config_dialog.open = true;
            self.key_config_dialog.initial_scope = Some(name.clone());
            // Status message if name not found -- dialog will show Default scope
            let known = ["pom", "editor", "settings", "files", "hex", "toolchain"];
            if !known.contains(&name.as_str()) {
                self.open_error = Some(format!(
                    "Key map '{}' not found -- showing Default map.",
                    name
                ));
            } else {
                self.open_error = None;
            }
            return;
        }

        // ── PFSHOW — Validates: Requirement 12.1–12.3 ——————————————————————
        if upper == "PFSHOW" {
            self.key_bar_visible = !self.key_bar_visible;
            self.open_error = None;
            return;
        }
        if upper == "PFSHOW ON" {
            self.key_bar_visible = true;
            self.open_error = None;
            return;
        }
        if upper == "PFSHOW OFF" {
            self.key_bar_visible = false;
            self.open_error = None;
            return;
        }

        // ── END — Validates: Requirement 17.1, 17.2 ———————————————————————
        if upper == "END" {
            let kind = self.tabs.active_tab().kind;
            if kind == TabKind::PrimaryOptionMenu {
                // Validates: Requirement 17.2 / 17.2a -- END from a POM closes
                // only that POM Workspace and navigates to the previously-active
                // tab when other Workspaces remain open; it terminates the app
                // only when the POM is the last Workspace open (equivalent to
                // EXIT). (CR-CH-016.)
                if self.tabs.len() <= 1 {
                    // Last Workspace -> terminate (Req 17.2a).
                    let result = self
                        .dispatch
                        .execute_command("file.exit", CommandParams::new());
                    if let CommandResult::Err(e) = result {
                        self.open_error = Some(e.to_string());
                    }
                } else {
                    // Other Workspaces remain -> close-and-navigate (Req 17.2,
                    // same behaviour as criterion 17.1).
                    self.close_current_and_navigate_back();
                }
            } else if kind == TabKind::FileExplorerPanel
                || kind == TabKind::CommandConfigurator
                || kind == TabKind::MenuWorkspace
            {
                // Validates: Requirement 19.10 (file-explorer),
                // command-configurator Requirement 2.8, and cw-requirements.md
                // Requirement 10.4 / 15.10 -- END/F3 from a Menu_Workspace (incl.
                // the Settings_Menu) or these Contexts returns to the POM.
                self.pending_return_to_pom = true;
            } else if kind == TabKind::SettingsPanel
                && self.settings_panel.namespace_filter.is_some()
            {
                // Validates: cw-requirements.md Requirement 10.4 -- END from a
                // Settings_Namespace_View returns to the Settings_Menu, not the
                // flat All-Settings view and not straight to the POM.
                self.open_settings_menu();
            } else {
                // Validates: Requirement 17.1 -- close current tab, go to previous
                self.close_current_and_navigate_back();
            }
            self.open_error = None;
            return;
        }

        // ── RETURN — Validates: Requirement 17.3, 17.4 ————————————————————
        if upper == "RETURN" {
            let is_pom = self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu;
            if is_pom {
                // Validates: Requirement 17.4 (REVISED, CR-CH-016) -- RETURN from
                // a POM behaves like END: close only that POM Workspace and
                // navigate back when other Workspaces remain open; terminate the
                // app only when the POM is the last Workspace open.
                if self.tabs.len() <= 1 {
                    let result = self
                        .dispatch
                        .execute_command("file.exit", CommandParams::new());
                    if let CommandResult::Err(e) = result {
                        self.open_error = Some(e.to_string());
                    }
                } else {
                    self.close_current_and_navigate_back();
                }
            } else {
                // Validates: Requirement 17.3 — navigate to POM tab
                if let Some(pom_idx) = self
                    .tabs
                    .tabs()
                    .iter()
                    .position(|t| t.kind == TabKind::PrimaryOptionMenu)
                {
                    self.tabs.set_active(pom_idx);
                } else {
                    self.tabs.insert_pom_tab(&self.runtime);
                }
            }
            self.open_error = None;
            return;
        }

        // Config-driven POM fastpath (menu-workspace Req 2.1e, 2.1i): a bare
        // option key (e.g. `1`, `S`) or `=<key>` (e.g. `=1`) is resolved against
        // the loaded pom.toml option list to the option's Option_Command, which
        // is then dispatched. There is NO behaviour keyed to the digit itself --
        // editing pom.toml's `command` is the only thing that changes what a key
        // does. Only the option's command re-enters handle_command below.
        if let Some(pom_command) = self.resolve_pom_option_key(&upper) {
            // Guard against a self-referential loop (an option whose command is
            // its own key): only recurse when the resolved command differs.
            if pom_command.to_uppercase() != upper {
                self.handle_command(&pom_command);
                return;
            }
        }

        // Settings navigation (two-level, cw-requirements.md Req 9, 10, 15.1).
        // Bare SETTINGS opens the data-driven Settings_Menu (Menu_Workspace
        // backed by menus/settings.toml). Option A opens the unfiltered flat
        // list; SETTINGS <ns> opens a filtered namespace view.
        if upper == "SETTINGS" {
            self.open_settings_menu();
            self.open_error = None;
            return;
        }
        if upper == "A" {
            // Validates: cw-requirements.md Req 9.4, 10.x -- All Settings flat list.
            self.open_settings_view(None);
            self.open_error = None;
            return;
        }
        if upper.starts_with("SETTINGS ") {
            let ns = cmd.trim()[9..].trim().to_lowercase();
            if ns.is_empty() {
                self.open_settings_menu();
            } else {
                self.open_settings_view(Some(ns));
            }
            self.open_error = None;
            return;
        }

        if upper == "=FILES" {
            // Validates: Requirement 19.1, 19.2 -- =FILES transforms the current
            // POM tab in place (fastpath from the Home Context).
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::FileExplorerPanel, "[FILES]");
            } else {
                self.tabs.open_file_explorer_panel_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }

        if upper == "FILES" {
            // Validates: Requirement 19.3; menu-workspace Req 2.1e -- FILES opens
            // the File Explorer Context (config-driven by command name). On a POM
            // tab it transforms in place; elsewhere it opens a new tab.
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::FileExplorerPanel, "[FILES]");
            } else {
                self.tabs.open_file_explorer_panel_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }

        if upper == "GSEARCH" || upper == "SEARCH" {
            // Validates: global-search Requirement 1.2
            self.open_or_focus_search_panel();
            self.open_error = None;
            return;
        }

        if upper == "COMMANDS" {
            // Validates: command-configurator Requirement 2.1, 2.7 -- open the
            // Command Configurator Context (title [COMMANDS]).
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::CommandConfigurator, "[COMMANDS]");
            } else {
                self.tabs.open_command_configurator_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }

        if upper == "THEMES" {
            // Validates: theme-and-appearance Requirement 20.1, 20.2 -- open the
            // Theme Editor Context. On a POM tab, transform in place; else open a
            // dedicated tab. (Distinct from `THEME <mode>` which switches theme.)
            self.open_theme_editor();
            self.open_error = None;
            return;
        }

        if upper == "LOG" {
            // Validates: notification-system Requirement 2.1
            self.tabs.open_event_log_tab(&self.runtime);
            self.notification_queue
                .lock()
                .expect("queue")
                .mark_all_read();
            self.open_error = None;
            return;
        }

        if upper == "FILE CATALOGS" || upper == "CATALOGS" {
            // Validates: Requirement 1.1, 14.6; menu-workspace Req 2.1e/2.1h --
            // File Catalogs Context, resolved by command name (config-driven).
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::FilesPanel, "[FILES]");
            } else {
                self.tabs.open_files_panel_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }

        if upper == "PLUGINS" {
            // Validates: plugin-manager-ui Requirement 1.1; menu-workspace Req
            // 2.1e -- PLUGINS opens the Plugin Manager (config-driven by name).
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::PluginManager, "[PLUGINS]");
            } else {
                self.tabs.open_plugin_manager_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }

        // Phase CV -- Extended POM options
        // Validates: Requirement 6.2 (cv-requirements.md)
        if upper == "MACROS" {
            // Validates: lua-macro-engine Requirement 12.1; menu-workspace Req 2.1e
            // -- MACROS opens the Macro Library Context (config-driven by name).
            if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
                self.tabs
                    .transform_active_pom_tab(TabKind::MacroLibrary, "[MACROS]");
            } else {
                self.tabs.open_macro_library_tab(&self.runtime);
            }
            self.open_error = None;
            return;
        }

        if upper == "RETRIEVE" {
            let cmd_text = self.command_text.clone();
            match self.retrieve_state.retrieve(&self.cmd_history, &cmd_text) {
                RetrieveResult::Recalled { command } => {
                    self.command_text = command;
                }
                RetrieveResult::ShowList { entries } => {
                    // Validates: Requirement 19.1 — show history list overlay
                    self.show_history_list = Some(entries);
                    self.command_text.clear();
                }
                RetrieveResult::HistoryEmpty | RetrieveResult::NoOlderHistory => {}
            }
            return;
        }

        // ── LOCATE / SORT / UP / DOWN / LEFT / RIGHT / TOP / BOTTOM ────────
        if upper.starts_with("LOCATE ") {
            let arg = cmd.trim()[7..].trim();
            let status = self.nav_manager.locate(arg, &mut self.tabs);
            self.open_error = if status.is_empty() {
                None
            } else {
                Some(status)
            };
            return;
        }

        if upper == "TOP" {
            self.nav_manager.top(&mut self.tabs);
            self.open_error = None;
            return;
        }

        if upper == "BOTTOM" {
            self.nav_manager.bottom(&mut self.tabs);
            self.open_error = None;
            return;
        }

        if upper == "UP" || upper.starts_with("UP ") {
            let n = parse_optional_u64(cmd.trim().get(2..).unwrap_or("").trim());
            self.nav_manager.up(n, &mut self.tabs);
            self.open_error = None;
            return;
        }

        if upper == "DOWN" || upper.starts_with("DOWN ") {
            let n = parse_optional_u64(cmd.trim().get(4..).unwrap_or("").trim());
            self.nav_manager.down(n, &mut self.tabs);
            self.open_error = None;
            return;
        }

        if upper == "LEFT" || upper.starts_with("LEFT ") {
            let n = parse_optional_u64(cmd.trim().get(4..).unwrap_or("").trim());
            self.nav_manager.left(n, &mut self.tabs);
            self.open_error = None;
            return;
        }

        if upper == "RIGHT" || upper.starts_with("RIGHT ") {
            let n = parse_optional_u64(cmd.trim().get(5..).unwrap_or("").trim());
            self.nav_manager.right(n, &mut self.tabs);
            self.open_error = None;
            return;
        }

        if upper == "SORT" || upper.starts_with("SORT ") {
            let rest = cmd.trim().get(4..).unwrap_or("").trim();
            let args: Vec<&str> = rest.split_whitespace().collect();
            let status = self.nav_manager.sort(&args, &mut self.tabs, &self.runtime);
            self.open_error = if status.is_empty() {
                None
            } else {
                Some(status)
            };
            return;
        }

        // ── EXCLUDE / SHOW / RESET ────────────────────────────────────────────
        if upper == "EXCLUDE ALL" || upper == "X ALL" {
            let msg = self
                .exclude_manager
                .exclude_all(&mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            return;
        }

        if upper.starts_with("EXCLUDE ") || upper.starts_with("X ") {
            // EXCLUDE 'text' [ALL]  or  X 'text' [ALL]
            let rest = if upper.starts_with("EXCLUDE ") {
                cmd.trim()[8..].trim()
            } else {
                cmd.trim()[2..].trim()
            };
            let (text, all_flag) = strip_all_suffix(rest);
            let msg = if all_flag {
                self.exclude_manager
                    .exclude_text_all(text, &mut self.tabs, &self.runtime)
            } else {
                self.exclude_manager
                    .exclude_text(text, &mut self.tabs, &self.runtime)
            };
            self.open_error = info_or_error(&msg);
            return;
        }

        if upper == "SHOW ALL" || upper == "INCLUDE ALL" {
            let msg = self.exclude_manager.show_all(&mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            return;
        }

        if upper.starts_with("SHOW ") || upper.starts_with("INCLUDE ") {
            let rest = if upper.starts_with("SHOW ") {
                cmd.trim()[5..].trim()
            } else {
                cmd.trim()[8..].trim()
            };
            let msg = self
                .exclude_manager
                .show_text(rest, &mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            return;
        }

        if upper == "RESET" || upper == "RESET EXCLUDED" || upper == "RESET ALL" {
            use ff_exclude_show_filter::ResetVariant;
            let variant = if upper == "RESET ALL" {
                ResetVariant::All
            } else if upper == "RESET EXCLUDED" {
                ResetVariant::Excluded
            } else {
                ResetVariant::Default
            };
            let msg = self
                .exclude_manager
                .reset(variant, &mut self.tabs, &self.runtime);
            self.open_error = info_or_error(&msg);
            return;
        }

        // ── FIND / RFIND / CHANGE / RCHANGE ─────────────────────────────────
        if upper == "RFIND" {
            let status = self.find_manager.rfind(&mut self.tabs, &self.runtime);
            self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                Some(status)
            } else {
                self.open_error = None;
                None
            };
            return;
        }

        if upper == "RCHANGE" {
            let status = self.find_manager.rchange(&mut self.tabs, &self.runtime);
            self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                Some(status)
            } else {
                None
            };
            return;
        }

        if upper.starts_with("FIND ") {
            let term = cmd.trim()[5..].trim();
            let status = self.find_manager.find(term, &mut self.tabs, &self.runtime);
            self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                Some(status)
            } else {
                None
            };
            return;
        }

        if upper.starts_with("CHANGE ") {
            // Parse: CHANGE 'old' 'new'  (single-quoted or bare words)
            let rest = cmd.trim()[7..].trim();
            if let Some((old, new)) = parse_two_args(rest) {
                let status = self
                    .find_manager
                    .change(&old, &new, &mut self.tabs, &self.runtime);
                self.open_error = if status.contains("NOT FOUND") || status.contains("error") {
                    Some(status)
                } else {
                    None
                };
            } else {
                self.open_error =
                    Some("CHANGE requires two arguments: CHANGE 'old' 'new'".to_string());
            }
            return;
        }

        // ── CAPS — Validates: Requirement 16.1, 16.2 ────────────────────────
        if upper == "CAPS ON" {
            self.tabs.active_tab_mut().edit_profile.caps = ff_edit_operations::CapsMode::On;
            self.open_error = None;
            return;
        }
        if upper == "CAPS OFF" {
            self.tabs.active_tab_mut().edit_profile.caps = ff_edit_operations::CapsMode::Off;
            self.open_error = None;
            return;
        }
        // == THEME -- command parity for theme switching ======================
        // Validates: theme-and-appearance Requirement 17 (architecture-brief
        // Principle 2: every user action is a command). `THEME <mode>` sets the
        // theme (same code path as the Settings menu, which dispatches this
        // command); bare `THEME` reports the current mode; an invalid mode errors.
        if upper == "THEME" {
            self.open_error = Some(format!(
                "Current theme: {}. Usage: THEME dark|light|high_contrast|legacy",
                self.palette.mode.section_name()
            ));
            return;
        }
        if upper.starts_with("THEME ") {
            let arg = cmd.trim().get(6..).unwrap_or("").trim();
            match ff_theme::mode::VisualMode::from_str_loose(arg) {
                Some(mode) => {
                    // Clear any stale error first; set_theme applies + persists
                    // and re-sets open_error only if persistence fails (Req 17.6).
                    self.open_error = None;
                    self.set_theme(mode);
                }
                None => {
                    self.open_error = Some(format!(
                        "Unknown theme '{arg}'. Valid: dark, light, high_contrast, legacy"
                    ));
                }
            }
            return;
        }

        if upper == "CAPS" {
            let tab = self.tabs.active_tab_mut();
            tab.edit_profile.caps = tab.edit_profile.caps.toggle();
            self.open_error = None;
            return;
        }

        // ── NULLS — Validates: Requirement 16.4 ──────────────────────────────
        if upper == "NULLS ON" {
            self.tabs.active_tab_mut().edit_profile.nulls = ff_edit_operations::NullsMode::On;
            self.open_error = None;
            return;
        }
        if upper == "NULLS OFF" {
            self.tabs.active_tab_mut().edit_profile.nulls = ff_edit_operations::NullsMode::Off;
            self.open_error = None;
            return;
        }

        // ── STATS — Validates: Requirement 16.7 ──────────────────────────────
        if upper == "STATS ON" {
            self.tabs.active_tab_mut().edit_profile.stats = ff_edit_operations::StatsMode::On;
            self.open_error = None;
            return;
        }
        if upper == "STATS OFF" {
            self.tabs.active_tab_mut().edit_profile.stats = ff_edit_operations::StatsMode::Off;
            self.open_error = None;
            return;
        }

        // ── LOCK — Validates: Requirement 16.8 ───────────────────────────────
        if upper == "LOCK ON" {
            self.tabs.active_tab_mut().edit_profile.lock = ff_edit_operations::ProfileLock::On;
            self.open_error = None;
            return;
        }
        if upper == "LOCK OFF" {
            self.tabs.active_tab_mut().edit_profile.lock = ff_edit_operations::ProfileLock::Off;
            self.open_error = None;
            return;
        }

        // ── PROFILE — Validates: Requirement 16.5, 16.6 ──────────────────────
        if upper == "PROFILE" {
            let summary = self.tabs.active_tab().edit_profile.display_summary();
            self.open_error = Some(summary);
            return;
        }
        if upper.starts_with("PROFILE ") {
            let rest = cmd.trim()[8..].trim();
            let mut parts = rest.splitn(2, ' ');
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("").trim();
            let result = self
                .tabs
                .active_tab_mut()
                .edit_profile
                .apply_keyword(key, val);
            match result {
                Ok(()) => self.open_error = None,
                Err(ProfileError::Locked) => {
                    self.open_error =
                        Some("Profile is locked -- use LOCK OFF to unlock".to_string());
                }
                Err(e) => self.open_error = Some(e.to_string()),
            }
            return;
        }

        // ── HILITE — Validates: Requirement 16.12 ────────────────────────────
        if upper == "HILITE" || upper.starts_with("HILITE ") {
            let keyword = cmd.trim().get(6..).unwrap_or("").trim();
            let mode = if keyword.is_empty() {
                Some(ff_edit_operations::HiliteMode::On)
            } else {
                ff_edit_operations::HiliteMode::from_keyword(keyword)
            };
            match mode {
                Some(m) => {
                    self.tabs.active_tab_mut().edit_profile.hilite = m;
                    self.open_error = None;
                }
                None => {
                    self.open_error = Some(format!("HILITE: unknown mode '{keyword}'"));
                }
            }
            return;
        }

        // ── SCROLL field update via command — Validates: Requirement 19.2 ──────
        if upper.starts_with("SCROLL ") {
            let arg = cmd.trim()[7..].trim();
            if let Some(amount) = crate::scroll_amount::ScrollAmount::parse(arg) {
                self.scroll_amount = amount;
                self.scroll_field_text = self.scroll_amount.display_string();
                self.open_error = None;
            } else {
                self.open_error = Some(format!(
                    "SCROLL: '{}' is not a valid scroll amount (PAGE/HALF/CSR/MAX/DATA/n)",
                    arg
                ));
            }
            return;
        }

        // ── Fastpath dotted notation (e.g. 3.1) — Validates: Requirement 19.4 ──
        // A dotted path like "3.1" navigates to POM option 3 sub-option 1.
        // For now, resolve the first segment as a POM option and record the
        // sub-option for future nested navigation.
        if upper.contains('.') && !upper.starts_with('.') {
            let parts: Vec<&str> = upper.splitn(2, '.').collect();
            if parts.len() == 2 {
                let first = parts[0].trim();
                let rest = parts[1].trim();
                // Only treat as fastpath if first segment is a single digit
                if first.len() == 1 && first.chars().all(|c| c.is_ascii_digit()) {
                    // Navigate to the top-level option first
                    self.handle_command(first);
                    // Then navigate to the sub-option if non-empty
                    if !rest.is_empty() {
                        self.handle_command(rest);
                    }
                    return;
                }
            }
        }

        // ── NAME -- Validates: CX Requirement 1.2, 1.3 ──────────────────────
        if upper == "NAME" {
            // Clear workspace name
            self.tabs.active_tab_mut().workspace_name = None;
            self.open_error = None;
            return;
        }
        if upper.starts_with("NAME ") {
            // Set workspace name (max 32 chars)
            let name = cmd.trim()[5..].trim();
            let name = if name.len() > 32 { &name[..32] } else { name };
            self.tabs.active_tab_mut().workspace_name = Some(name.to_string());
            self.open_error = None;
            return;
        }

        // ── SPLIT DETACH -- Validates: CX Requirement 3.1, 3.4 ──────────────
        if upper == "SPLIT DETACH" {
            let idx = self.tabs.active_index();
            let floating_count = self.tabs.tabs().iter().filter(|t| t.is_floating).count();
            if floating_count >= 16 {
                self.open_error =
                    Some("Maximum number of detached Workspaces (16) reached.".to_string());
            } else {
                self.tabs.tabs_mut()[idx].is_floating = true;
                self.detach_pending = Some(idx);
                self.open_error = None;
            }
            return;
        }
        // ── SPLIT (no arg) -- Validates: CX Requirement 3.2, 3.3 ──────────────
        // Non-editor tabs: detach (ISPF SPLIT heritage)
        // Editor tabs: split-screen (Req 19.11 backward compat)
        if upper == "SPLIT" {
            let kind = self.tabs.active_tab().kind;
            let is_editor = matches!(kind, TabKind::FileEditor | TabKind::Untitled);
            if !is_editor {
                let idx = self.tabs.active_index();
                let floating_count = self.tabs.tabs().iter().filter(|t| t.is_floating).count();
                if floating_count >= 16 {
                    self.open_error =
                        Some("Maximum number of detached Workspaces (16) reached.".to_string());
                } else {
                    self.tabs.tabs_mut()[idx].is_floating = true;
                    self.detach_pending = Some(idx);
                    self.open_error = None;
                }
                return;
            }
            // Editor tab: split-screen behaviour
            // Split at current cursor line
            let cursor_line = self.tabs.active_tab().cursor.cursor_line();
            self.split_screen = Some(crate::scroll_amount::SplitScreenState::new(
                (cursor_line as usize).saturating_sub(1),
            ));
            self.open_error = None;
            return;
        }
        if upper == "SWAP" || upper.starts_with("SWAP ") {
            // SWAP is primarily the tab/workspace switcher (multi-tab-editor
            // Req 18), and also retains the split-screen focus-swap behaviour
            // for the no-argument case when a split is active (menu-and-statusbar
            // Req 19.12). Parse the argument to decide.
            let arg = cmd.trim().get(4..).unwrap_or("").trim().to_string();
            let arg_upper = arg.to_uppercase();

            if arg.is_empty() {
                // Bare SWAP: swap split focus if a split is active, else open the
                // tab picker (Req 18.6, 18.7).
                if let Some(ref mut ss) = self.split_screen {
                    ss.swap_focus();
                    self.open_error = None;
                } else {
                    self.show_swap_list = Some(());
                    self.open_error = None;
                }
            } else if arg_upper == "LIST" {
                // SWAP LIST: open the tab picker (Req 18.3).
                self.show_swap_list = Some(());
                self.open_error = None;
            } else if let Ok(n) = arg.parse::<usize>() {
                // SWAP n: activate the n-th tab, 1-based (Req 18.1, 18.2).
                let count = self.tabs.len();
                if n >= 1 && n <= count {
                    self.tabs.set_active(n - 1);
                    self.open_error = None;
                } else {
                    self.open_error = Some(format!(
                        "SWAP: tab number {n} out of range (valid: 1 to {count})"
                    ));
                }
            } else {
                // Non-numeric, non-LIST argument (Req 18.2).
                self.open_error = Some(format!(
                    "SWAP: invalid argument '{arg}'. Usage: SWAP <n>, SWAP LIST"
                ));
            }
            return;
        }
        if upper == "UNSPLIT" {
            // Validates: Requirement 19.14 -- unsplit restores single-panel view
            self.split_screen = None;
            self.open_error = None;
            return;
        }

        // ── AUTONUM / NUM aliases — Validates: Requirement 16.10, 16.11 ──────
        if upper == "AUTONUM ON" || upper == "AUTONUM OFF" {
            let rest = &cmd.trim()[7..];
            let redirected = format!("NUMBER{rest}");
            self.handle_command(&redirected);
            return;
        }
        if upper == "NUM" || upper.starts_with("NUM ") {
            let rest = cmd.trim().get(3..).unwrap_or("").trim();
            let redirected = if rest.is_empty() {
                "NUMBER".to_string()
            } else {
                format!("NUMBER {rest}")
            };
            self.handle_command(&redirected);
            return;
        }

        // ── SUBMIT — Validates: Requirement 17.1 ─────────────────────────────
        if upper == "SUBMIT" {
            // Stub: JES subsystem dispatch deferred to Phase CC/CD.
            self.open_error = Some("SUBMIT: JES subsystem not yet available".to_string());
            return;
        }

        // ── TIME — Validates: Requirement 20.4 ───────────────────────────────
        if upper == "TIME" {
            let now = chrono::Local::now();
            let msg = format!(
                "Date: {}  Time: {}  Day: {}",
                now.format("%Y-%m-%d"),
                now.format("%H:%M:%S"),
                now.format("%j")
            );
            self.open_error = Some(msg);
            return;
        }

        // ── STATUS — Validates: Requirement 20.5, 20.6 ───────────────────────
        if upper == "STATUS" || upper.starts_with("STATUS ") {
            let jobname = if upper.starts_with("STATUS ") {
                let j = cmd.trim()[7..].trim();
                if j.is_empty() {
                    None
                } else {
                    Some(j.to_string())
                }
            } else {
                None
            };
            let msg = match jobname {
                Some(ref j) => format!("STATUS: routing to JES panel (filter: {})", j),
                None => "STATUS: routing to JES job status panel".to_string(),
            };
            self.open_error = Some(msg);
            return;
        }

        // ── CREATE — Validates: Requirement 17.2 ─────────────────────────────
        if upper.starts_with("CREATE ") {
            let dsn = cmd.trim()[7..].trim();
            if dsn.is_empty() {
                self.open_error = Some("CREATE requires a dataset name argument".to_string());
            } else {
                // Stub: dataset creation deferred to Phase BU/CB.
                self.open_error = Some(format!("CREATE {dsn}: dataset creation not yet available"));
            }
            return;
        }

        // ── REPLACE — Validates: Requirement 17.3 ────────────────────────────
        if upper.starts_with("REPLACE ") {
            let dsn = cmd.trim()[8..].trim();
            if dsn.is_empty() {
                self.open_error = Some("REPLACE requires a dataset name argument".to_string());
            } else {
                self.open_error = Some(format!("REPLACE {dsn}: dataset replace not yet available"));
            }
            return;
        }

        // ── BROWSE — Validates: Requirement 17.5 ─────────────────────────────
        if upper.starts_with("BROWSE ") {
            let dsn = cmd.trim()[7..].trim();
            if dsn.is_empty() {
                self.open_error = Some("BROWSE requires a dataset name argument".to_string());
            } else {
                // Open as read-only editor tab (full browse mode deferred).
                let mut p = CommandParams::new();
                p.insert("path", dsn);
                let result = self.dispatch.execute_command("file.open", p);
                if let CommandResult::Err(e) = result {
                    self.open_error = Some(e.to_string());
                } else {
                    self.open_error = None;
                }
            }
            return;
        }

        // ── VIEW — Validates: Requirement 17.6 ───────────────────────────────
        if upper.starts_with("VIEW ") {
            let dsn = cmd.trim()[5..].trim();
            if dsn.is_empty() {
                self.open_error = Some("VIEW requires a dataset name argument".to_string());
            } else {
                let mut p = CommandParams::new();
                p.insert("path", dsn);
                let result = self.dispatch.execute_command("file.open", p);
                if let CommandResult::Err(e) = result {
                    self.open_error = Some(e.to_string());
                } else {
                    self.open_error = None;
                }
            }
            return;
        }

        // ── COMPARE — Validates: Requirement 17.7 ────────────────────────────
        if upper.starts_with("COMPARE ") {
            let dsn = cmd.trim()[8..].trim();
            if dsn.is_empty() {
                self.open_error = Some("COMPARE requires a dataset name argument".to_string());
            } else {
                // Stub: compare view deferred to Phase BX/ff-compare.
                self.open_error = Some(format!("COMPARE {dsn}: compare view not yet available"));
            }
            return;
        }

        // ── WORKSPACE commands -- Validates: workspace-model Requirement 2.1-2.4 ──
        if upper.starts_with("WORKSPACE ") || upper == "WORKSPACE" {
            let rest = cmd.trim().get(9..).unwrap_or("").trim();
            let rest_upper = rest.to_uppercase();
            if rest_upper.starts_with("OPEN ") {
                let path = rest.get(5..).unwrap_or("").trim();
                if path.is_empty() {
                    self.open_error = Some("WORKSPACE OPEN requires a path".to_string());
                } else {
                    self.open_workspace(std::path::Path::new(path));
                }
            } else if rest_upper == "SAVE" {
                self.save_workspace_to(None);
            } else if rest_upper.starts_with("SAVE AS ") {
                let path = rest.get(8..).unwrap_or("").trim();
                if path.is_empty() {
                    self.open_error = Some("WORKSPACE SAVE AS requires a path".to_string());
                } else {
                    self.save_workspace_to(Some(std::path::Path::new(path)));
                }
            } else if rest_upper == "CLOSE" {
                self.close_workspace();
            } else if rest_upper.starts_with("ADD ROOT ") {
                let path = rest.get(9..).unwrap_or("").trim();
                if path.is_empty() {
                    self.open_error = Some("WORKSPACE ADD ROOT requires a path".to_string());
                } else if let Some(ws) = self.active_workspace.as_mut() {
                    let p = std::path::PathBuf::from(path);
                    if !ws.roots.contains(&p) {
                        ws.roots.push(p.clone());
                        ws.is_modified = true;
                    }
                    let cat = crate::catalog_registry::VirtualCatalog {
                        name: p
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_else(|| p.to_string_lossy().into_owned()),
                        catalog_type: crate::catalog_registry::CatalogType::Native,
                        path: p.to_string_lossy().into_owned(),
                        description: Some("Workspace root".to_string()),
                        auto_mount: true,
                        default_hlq: None,
                        mount_point: None,
                        read_only: false,
                    };
                    let _ = self.files_panel.registry.register(cat);
                    self.open_error = None;
                } else {
                    self.open_error =
                        Some("No active workspace -- use WORKSPACE OPEN first".to_string());
                }
            } else if rest_upper.starts_with("REMOVE ROOT ") {
                let path = rest.get(12..).unwrap_or("").trim();
                if path.is_empty() {
                    self.open_error = Some("WORKSPACE REMOVE ROOT requires a path".to_string());
                } else if let Some(ws) = self.active_workspace.as_mut() {
                    let p = std::path::PathBuf::from(path);
                    ws.roots.retain(|r| r != &p);
                    ws.is_modified = true;
                    let name = p
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| p.to_string_lossy().into_owned());
                    let _ = self.files_panel.registry.remove(&name);
                    self.open_error = None;
                } else {
                    self.open_error = Some("No active workspace".to_string());
                }
            } else {
                self.open_error = Some(
                    "WORKSPACE: unknown subcommand. Use OPEN/SAVE/SAVE AS/CLOSE/ADD ROOT/REMOVE ROOT"
                        .to_string(),
                );
            }
            return;
        }

        // Menu_Workspace option key lookup + Target_Resolution.
        // Validates: menu-workspace Requirement 3.1, 3.6, 3.7, 10.1, 10.3, 10.6
        if self.tabs.active_tab().kind == crate::tab_state::TabKind::MenuWorkspace {
            if let Some(mw) = self.tabs.active_tab().menu_workspace.as_ref() {
                if let Some(menu) = mw.menu.as_ref() {
                    match crate::menu_workspace::commands::find_option(cmd.trim(), menu) {
                        Ok(option) => {
                            // Req 10.6: an inline [options.target] wins over `command`.
                            if let Some(target) = option.target.clone() {
                                self.dispatch_command_target(&target);
                                return;
                            }
                            // Req 10.1/10.3: resolve the option's command to a
                            // user-owned target and dispatch it; otherwise fall
                            // through to the existing pipeline (Req 10.2).
                            let option_cmd = option.command.clone();
                            match self.resolve_and_dispatch_command(&option_cmd) {
                                super::target_dispatch::ResolveOutcome::Dispatched => return,
                                super::target_dispatch::ResolveOutcome::FallThrough => {
                                    self.handle_command(&option_cmd);
                                    return;
                                }
                            }
                        }
                        Err(msg) => {
                            self.open_error = Some(msg);
                            return;
                        }
                    }
                }
            }
        }

        // ── Route through CommandEngine ──────────────────────────────────
        self.retrieve_state.reset();
        let status = self.cmd_engine.execute_command_line(cmd);
        match status.kind {
            StatusKind::Info => {
                self.open_error = None;
            }
            StatusKind::SyntaxError | StatusKind::StructureError | StatusKind::RuntimeError => {
                self.open_error = Some(status.text.clone());
            }
        }
        // (History is recorded once at the top of handle_command for every
        // submitted command, so no per-path recording is needed here.)
    }
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
        if let Ok(udd) = ff_session::UserDataDir::resolve(None) {
            return udd.path().join("menus");
        }
        dirs::data_dir()
            .map(|base| base.join("FileForgeWorkbench").join("menus"))
            .unwrap_or_else(|| std::path::PathBuf::from("menus"))
    }

    /// The themes directory: the test override when set, else the real
    /// `<User_Data_Dir>/themes/` (via `theme_defaults::themes_dir`). All Theme
    /// editor / active-theme file operations route through this so tests can
    /// isolate them to a TempDir (CR-NR-074).
    pub(super) fn themes_dir(&self) -> std::path::PathBuf {
        self.themes_dir_override
            .clone()
            .unwrap_or_else(crate::theme_defaults::themes_dir)
    }

    /// Open (or return to) a menu by name.
    ///
    /// An empty name or `POM` opens/returns to the Home Context (POM); any other
    /// name opens the data-driven Menu_Workspace backed by `menus/<name>.toml`,
    /// with a missing file shown in the load-error state.
    ///
    /// Validates: menu-workspace Requirement 11.1, 11.2, 11.4, 11.5
    pub(super) fn open_menu_by_name(&mut self, name: &str) {
        use crate::tab_state::TabKind;
        let lower = name.trim().to_lowercase();
        // Req 11.1 / 11.2: bare MENU and MENU POM go to the Home Context.
        if lower.is_empty() || lower == "pom" {
            if self.tabs.active_tab().kind != TabKind::PrimaryOptionMenu {
                self.tabs.insert_pom_tab(&self.runtime);
            }
            self.open_error = None;
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

    /// Ensure the active POM tab carries a loaded `MenuWorkspaceState` backed by
    /// `menus/pom.toml`, loading it lazily on first render. The POM keeps its
    /// `TabKind::PrimaryOptionMenu` identity, `[POM]` title, and Title_Line
    /// styling; only its option list becomes data-driven (menu-workspace Req
    /// 2.1c, 2.1d). Idempotent: does nothing if the state is already present or
    /// the active tab is not a POM.
    ///
    /// Validates: menu-workspace Requirement 2.1c, 2.1d
    pub(super) fn ensure_pom_menu_loaded(&mut self) {
        use crate::tab_state::TabKind;
        if self.tabs.active_tab().kind != TabKind::PrimaryOptionMenu {
            return;
        }
        if self.tabs.active_tab().menu_workspace.is_some() {
            return;
        }
        let pom_path = self.menus_dir().join("pom.toml");
        let limits = crate::menu_workspace::loader::option_limits_from_config(&self.config_handle);
        let mut state =
            crate::menu_workspace::MenuWorkspaceState::load_with_limits(&pom_path, limits);
        // Config-driven fallback (menu-workspace Req 2.1h): if the on-disk
        // pom.toml is missing or invalid, fall back to the built-in default POM
        // content so the Home Context always has its options. This also keeps
        // the POM deterministic in tests that do not seed a menus directory.
        if state.menu.is_none() {
            if let Ok(menu) = crate::menu_workspace::loader::parse_menu_str(
                crate::menu_workspace::defaults::DEFAULT_POM_TOML,
            ) {
                state.menu = Some(menu);
                state.load_error = None;
            }
        }
        let idx = self.tabs.active_index();
        if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
            tab.menu_workspace = Some(state);
        }
    }

    /// Resolve a POM fastpath key to its Option_Command using the loaded
    /// `pom.toml` option list. Accepts a bare Option_Key (e.g. `1`, `S`) or the
    /// `=<key>` fastpath form (e.g. `=1`). Returns the option's `command` when a
    /// matching, enabled option exists in the POM menu; otherwise `None`.
    ///
    /// This is how `=<key>` and bare keys become config-driven (menu-workspace
    /// Req 2.1e, 2.1i): the key selects a row, and that row's command drives the
    /// behaviour -- no digit is coupled to a destination in code.
    ///
    /// The lookup uses the POM tab's menu regardless of which Workspace is
    /// active, so `=1` from any context resolves against the POM (Navigation
    /// Origin, menu-workspace Req 5.7).
    ///
    /// Validates: menu-workspace Requirement 2.1e, 2.1i
    pub(super) fn resolve_pom_option_key(&mut self, upper: &str) -> Option<String> {
        use crate::tab_state::TabKind;
        // Normalise: strip a single leading '=' for the fastpath form.
        let key = upper.strip_prefix('=').unwrap_or(upper);
        // Only single short keys are POM option keys (1-4 chars, no spaces).
        if key.is_empty() || key.len() > 4 || key.contains(' ') {
            return None;
        }
        // Ensure the active POM tab's menu is loaded so a fastpath resolves even
        // before the POM has rendered.
        if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
            self.ensure_pom_menu_loaded();
        }
        // Prefer a loaded POM tab's menu (respects user edits / hot-reload);
        // otherwise consult the on-disk pom.toml, then the built-in default, so
        // the fastpath is config-driven from any Workspace (Navigation Origin =
        // POM, menu-workspace Req 5.7). The default fallback also keeps the
        // resolver deterministic when no POM tab exists yet.
        let menu = self
            .tabs
            .tabs()
            .iter()
            .find(|t| t.kind == TabKind::PrimaryOptionMenu)
            .and_then(|t| t.menu_workspace.as_ref())
            .and_then(|mw| mw.menu.clone())
            .or_else(|| {
                let pom_path = self.menus_dir().join("pom.toml");
                crate::menu_workspace::loader::load_menu_file(&pom_path).ok()
            })
            .or_else(|| {
                crate::menu_workspace::loader::parse_menu_str(
                    crate::menu_workspace::defaults::DEFAULT_POM_TOML,
                )
                .ok()
            })?;
        let option = menu
            .options
            .iter()
            .find(|o| o.key.eq_ignore_ascii_case(key) && o.enabled)?;
        Some(option.command.clone())
    }

    /// Open the Theme Editor Context and populate its state: list available
    /// themes, select the current active theme, and load it as the working copy.
    ///
    /// On a POM tab, transforms in place (so END/RETURN returns to the POM);
    /// otherwise opens a dedicated tab.
    ///
    /// Validates: theme-and-appearance Requirement 20.1, 20.2
    pub(super) fn open_theme_editor(&mut self) {
        use crate::tab_state::TabKind;
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

        if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
            self.tabs
                .transform_active_pom_tab(TabKind::ThemeEditor, "[THEME]");
        } else {
            self.tabs.open_theme_editor_tab(&self.runtime);
        }
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
        }
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

    /// Open the Settings_Menu -- the data-driven Menu_Workspace backed by
    /// `menus/settings.toml`.
    ///
    /// Transforms the active POM tab in place when opened from the Home Context
    /// (so F3/END returns to the POM), otherwise opens a dedicated tab.
    ///
    /// Validates: cw-requirements.md Requirement 9.1; configuration-system
    /// Requirement 15.1
    pub(super) fn open_settings_menu(&mut self) {
        let menus_dir = self.menus_dir();
        let limits = crate::menu_workspace::loader::option_limits_from_config(&self.config_handle);
        self.tabs
            .open_menu_workspace_here("settings", &menus_dir, limits, &self.runtime);
        // Surface the load-error message when settings.toml is missing.
        if let Some(mw) = self.tabs.active_tab().menu_workspace.as_ref() {
            self.open_error = mw.load_error.clone();
        } else {
            self.open_error = None;
        }
    }

    /// Open the Settings panel, optionally as a Settings_Namespace_View.
    ///
    /// When `namespace` is `Some(ns)` the flat-list filter is pre-populated
    /// with `<ns>.` and the tab title becomes `[SETTINGS:<ns>]`. When `None`
    /// the unfiltered All-Settings view is shown with title `[SETTINGS]`.
    ///
    /// Validates: cw-requirements.md Requirement 10.1, 10.2, 10.5, 9.4
    pub(super) fn open_settings_view(&mut self, namespace: Option<String>) {
        use crate::tab_state::TabKind;
        let title = match &namespace {
            Some(ns) => format!("[SETTINGS:{ns}]"),
            None => "[SETTINGS]".to_string(),
        };
        // Pre-populate the flat-list filter with the namespace prefix so only
        // matching keys are visible immediately (Req 10.2).
        self.settings_panel.filter = match &namespace {
            Some(ns) => format!("{ns}."),
            None => String::new(),
        };
        self.settings_panel.namespace_filter = namespace;
        if self.tabs.active_tab().kind == TabKind::PrimaryOptionMenu {
            self.tabs
                .transform_active_pom_tab(TabKind::SettingsPanel, &title);
        } else {
            self.tabs.open_settings_panel_tab(&self.runtime);
            self.tabs.active_tab_mut().title = title;
        }
    }

    /// Reconstruct open Workspaces from persisted Workspace_Descriptors on
    /// session restore. Re-opens every visible Workspace in tab order, not only
    /// file-backed tabs. Unknown descriptors are skipped so one bad entry does
    /// not abort the whole restore (graceful degradation).
    ///
    /// Validates: startup-and-session Requirement 21.2, 21.3, 21.4, 21.5, 21.9.
    pub(super) fn restore_workspace_descriptors(
        &mut self,
        descriptors: &[ff_session::WorkspaceDescriptor],
    ) {
        use ff_session::session_state::{DescriptorValue, WorkspaceDescriptor, WorkspaceKind};

        for descriptor in descriptors {
            match descriptor {
                WorkspaceDescriptor::Menu { name: _name } => {
                    // Menu Workspaces open via the MENU command (menu-workspace
                    // Requirement 11), wired in DB.4. Until then, a persisted
                    // menu (other than the POM, which is guaranteed separately)
                    // is skipped rather than dropped incorrectly.
                    // Req 21.4 restore lands with the MENU command wiring (DB.4).
                }
                WorkspaceDescriptor::CustomWorkspace {
                    workspace_kind,
                    params,
                } => match workspace_kind {
                    WorkspaceKind::Editor => {
                        if let Some(DescriptorValue::String(uri)) = params.get("uri") {
                            if let Err(e) = self.tabs.open_file(uri, &self.runtime) {
                                self.open_error = Some(format!("Could not restore: {e}"));
                            }
                        }
                    }
                    WorkspaceKind::Files => {
                        self.tabs.open_files_panel_tab(&self.runtime);
                    }
                    WorkspaceKind::FileExplorer => {
                        self.tabs.open_file_explorer_panel_tab(&self.runtime);
                    }
                    WorkspaceKind::Settings => {
                        let namespace = match params.get("namespace") {
                            Some(DescriptorValue::String(ns)) => Some(ns.clone()),
                            _ => None,
                        };
                        self.tabs.open_settings_panel_tab(&self.runtime);
                        self.settings_panel.filter = match &namespace {
                            Some(ns) => format!("{ns}."),
                            None => String::new(),
                        };
                        let title = match &namespace {
                            Some(ns) => format!("[SETTINGS:{ns}]"),
                            None => "[SETTINGS]".to_string(),
                        };
                        self.tabs.active_tab_mut().title = title;
                        self.settings_panel.namespace_filter = namespace;
                    }
                    WorkspaceKind::Search => {
                        self.tabs.open_search_results_tab(&self.runtime);
                    }
                    WorkspaceKind::PluginManager => {
                        self.tabs.open_plugin_manager_tab(&self.runtime);
                    }
                    WorkspaceKind::EventLog => {
                        self.tabs.open_event_log_tab(&self.runtime);
                    }
                    WorkspaceKind::MacroLibrary => {
                        self.tabs.open_macro_library_tab(&self.runtime);
                    }
                    WorkspaceKind::CommandConfigurator => {
                        // Validates: startup-and-session Requirement 21.2, 21.3;
                        // command-configurator Requirement 2.1.
                        self.tabs.open_command_configurator_tab(&self.runtime);
                    }
                    WorkspaceKind::PrimaryOptionMenu => {
                        // POM presence is guaranteed by ensure_pom_tab_present;
                        // no explicit open needed here.
                    }
                    // Untitled and any future kind: skip (Req 21.9).
                    _ => {}
                },
            }
        }
    }
}
use super::WorkbenchShell;
