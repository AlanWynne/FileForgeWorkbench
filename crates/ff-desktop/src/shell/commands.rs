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

    /// Build the current [`CursorContext`](ff_command::CursorContext) from live
    /// focus/selection state (CR-CH-028, command-framework Requirement 12.2).
    ///
    /// Start scope (Requirement 12.2, focused-control identity): the command
    /// line and the focused Menu_Option; other workspaces populate the workspace
    /// context name and editor cursor/selection where applicable and MAY add
    /// EXTRAS later. This is a per-invocation SNAPSHOT (Requirement 12.5).
    pub(super) fn capture_cursor_context(&self, ctx: &egui::Context) -> ff_command::CursorContext {
        let mut b = ff_command::CursorContext::builder();

        // (a) Focused Workspace context name.
        if let Some(name) = context_name_for_kind(self.tabs.active_tab().kind) {
            b = b.workspace_context(name);
        }

        // (b)/(c) Focused-control identity + text. The command field id mirrors
        // `cmd_field_id()` (the shell's "Command ===>" TextEdit). A focused Menu
        // _Option is recorded during render in `focused_menu_option`.
        let focused = ctx.memory(|m| m.focused());
        let cmd_field = egui::Id::new("command_field_input");
        if focused == Some(cmd_field) {
            b = b.focused_identity("command-line");
            if !self.command_text.trim().is_empty() {
                b = b.focused_text(self.command_text.clone());
            }
        } else if let Some((id, command, label)) = &self.focused_menu_option {
            if focused == Some(*id) {
                // The option's command is its semantic identity (e.g. "FILES");
                // the label is the human text.
                b = b.focused_identity(command.clone());
                if !label.trim().is_empty() {
                    b = b.focused_text(label.clone());
                }
            }
        }

        // (d) Editor cursor + selection when an editor document is active.
        if matches!(
            self.tabs.active_tab().kind,
            TabKind::FileEditor | TabKind::Untitled
        ) {
            let tab = self.tabs.active_tab();
            b = b
                .cursor_line(tab.cursor.cursor_line() as usize)
                .cursor_column(tab.cursor.cursor_column() as usize);
        }

        // (e) Active scroll setting (for the deferred CSR consumer).
        if !self.scroll_field_text.trim().is_empty() {
            b = b.scroll_setting(self.scroll_field_text.clone());
        }

        b.build()
    }

    /// Refresh the shared Cursor_Context snapshot so both the registry provider
    /// and the string-path commands see the same per-invocation package
    /// (CR-CH-028, command-framework Requirement 12.4). Called once per frame
    /// before any dispatch.
    pub(super) fn refresh_cursor_context_snapshot(&mut self, ctx: &egui::Context) {
        let cc = self.capture_cursor_context(ctx);
        if let Ok(mut guard) = self.cursor_context_snapshot.lock() {
            *guard = cc;
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
        // RETRIEVE is excluded (it is the recall action, not a recallable
        // command). Match the VERB, not the exact string: B066's key-dispatch
        // merges the command-field content, so an F12 press with a non-empty
        // field arrives here as `RETRIEVE <field>` -- that must also be excluded
        // (B067). Empty input is skipped by `CommandHistory::add`.
        let is_retrieve = upper == "RETRIEVE" || upper.starts_with("RETRIEVE ");
        if !is_retrieve {
            self.cmd_history.add(cmd);
        }

        // Stage 1 (CR-CH-025, command-framework Req 8.3): current-menu Option_Key
        // lookup. When the active Workspace is a Menu_Workspace and the typed
        // string matches an Option_Key of the CURRENT menu, activate that option
        // (menu-workspace Req 3.1). A NON-matching token falls through to the
        // rest of the chain (Req 3.6) -- it is NOT a terminal error here.
        if self.try_current_menu_option(cmd) {
            return;
        }

        // ── Shell-level intercepts (stage 2: built-in command / Command_ID) ──
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

        if upper == "POM" {
            // Validates: Requirement 14.10, 14.14 -- POM opens a new POM tab.
            self.tabs.insert_pom_tab(&self.runtime);
            self.open_error = None;
            return;
        }

        if upper == "START" || upper.starts_with("START ") {
            // Validates: menu-workspace Requirement 14.8, 14.9 (CR-CH-022) --
            // START is the ONLY tab-creator. Forms:
            //   START            -> new tab rooted at the POM (empty stack)
            //   START =<path>    -> new POM tab, then drill along <path> (POM on
            //                       the stack via the `=` origin rule)
            //   START <arg>      -> new tab rooted DIRECTLY at <arg> (empty stack)
            let arg = cmd.trim().get(5..).map(str::trim).unwrap_or("");
            self.start_new_workspace(arg);
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

        // ── HELP / F1 fallback — Validates: Requirement 18.1, 18.2;
        //    command-framework Requirement 12.8 (CR-NR-079) ————————————————————
        if upper == "HELP" {
            let registry = HelpTopicRegistry::new(); // empty registry — no topics loaded yet
                                                     // CR-NR-079 (Req 12.8): build the HELP EditorContext FROM the
                                                     // Cursor_Context snapshot so a focused Menu_Option resolves that
                                                     // option's help topic (F1 on FILES -> cmd:FILES). The Cursor_Context
                                                     // records a focused option's COMMAND as `focused_identity` (and
                                                     // "command-line" when the command field is focused). We feed that
                                                     // command through the command-line path of ContextDetector so it
                                                     // resolves `cmd:<OPTION_COMMAND>`; absent a specific focused control
                                                     // we fall back to today's behaviour (the command-line text).
            let cc = self
                .cursor_context_snapshot
                .lock()
                .map(|g| g.clone())
                .unwrap_or_default();
            let is_menu_option = cc
                .focused_identity
                .as_deref()
                .map(|id| id != "command-line")
                .unwrap_or(false);
            let command_line_text = if is_menu_option {
                cc.focused_identity.clone().unwrap_or_default()
            } else {
                self.command_text.clone()
            };
            let ctx = EditorContext {
                command_line_text,
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
            // Validates: function-keys Requirement 22.1, 22.5 (CR-CH-029) --
            // KEYS opens the Keys Workspace in place (replaces the modal).
            self.open_keys_editor(None);
            self.open_error = None;
            return;
        }
        if upper.starts_with("KEYS ") {
            // Validates: function-keys Requirement 22.5 -- KEYS <kind> opens the
            // Keys Workspace with that workspace kind pre-selected.
            let kind = cmd.trim()[5..].trim();
            self.open_keys_editor(Some(kind));
            self.open_error = None;
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
            // Validates: menu-workspace Requirement 14.4, 14.5 (CR-CH-022) --
            // END pops one level of the active tab's Navigation_Stack and
            // reconstructs the parent Context in place; an empty stack closes the
            // Workspace (terminates when last, preserving CR-CH-016). This
            // replaces the former per-kind END rules and the three ad-hoc
            // mechanisms (pending_return_to_pom / namespace_filter /
            // opened_from_settings).
            self.nav_end();
            self.open_error = None;
            return;
        }

        // ── RETURN -- Validates: menu-workspace Requirement 14.10 (CR-CH-022) ──
        if upper == "RETURN" {
            // RETURN collapses the whole Navigation_Stack to the tab's ROOT
            // Context in one step (distinct from END's one-level pop); at the
            // root it behaves as END-at-root (close / exit when last, CR-CH-016).
            self.nav_return();
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

        // CR-CH-025: the hardcoded `SETTINGS`, `SETTINGS <ns>`, and `A`
        // intercepts are REMOVED. Opening the Settings menu is now keyword-less
        // menu-name resolution of the token `SETTINGS` (stage 3, see
        // `try_menu_name_dispatch`), identical to `POM` or any user menu; the
        // flat config-key browser and namespace filtering moved to the `CONFIG
        // [<namespace>]` command below (configuration-system Req 20).

        // ── CONFIG [<namespace>] -- flat configuration-key browser (Req 20) ──
        if upper == "CONFIG" {
            // Bare CONFIG: the unfiltered All-Settings flat list.
            self.open_config_view(None);
            self.open_error = None;
            return;
        }
        if upper.starts_with("CONFIG ") {
            // CONFIG <namespace>: the flat list pre-filtered to `<namespace>.`.
            let ns = cmd.trim()[7..].trim().to_lowercase();
            if ns.is_empty() {
                self.open_config_view(None);
            } else {
                self.open_config_view(Some(ns));
            }
            self.open_error = None;
            return;
        }

        if upper == "=FILES" || upper == "FILES" {
            // Validates: Requirement 19.1-19.3; menu-workspace Req 14.2
            // (CR-CH-022) -- navigate the CURRENT tab in place; never a new tab.
            self.nav_to_kind(ff_session::session_state::WorkspaceKind::FileExplorer);
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
            // Validates: command-configurator Requirement 2.1, 2.7; menu-workspace
            // Req 14.2 (CR-CH-022) -- navigate the current tab in place.
            self.nav_to_kind(ff_session::session_state::WorkspaceKind::CommandConfigurator);
            self.open_error = None;
            return;
        }

        // CR-CH-024: the `THEMES` command is removed; bare `THEME` (handled
        // below) now opens the Theme Editor. No `THEMES` intercept remains.

        if upper == "MENUS" {
            // Validates: menu-workspace Requirement 13.1 (CR-NR-075) -- open the
            // Menus Editor Context. Both the POM `M` row and the Settings `M` row
            // dispatch this command, so both entry points open the editor.
            self.open_menus_editor();
            self.open_error = None;
            return;
        }

        if upper == "RESET BARE" || upper.starts_with("RESET BARE ") {
            // Validates: configuration-system Requirement 19.1, 19.2, 19.9-19.15
            // (CR-CH-021, CR-NR-083) -- resolve the target profile list from the
            // argument(s), then open the confirmation dialog. Take no action
            // until confirmed. An unknown named profile blocks with an error and
            // no dialog (Req 19.12). The argument is taken from the ORIGINAL
            // (case-preserving) command so profile names keep their case for
            // display; matching is slug-based.
            let args = cmd.trim().get("RESET BARE".len()..).unwrap_or("").trim();
            match self.resolve_reset_bare_target(args) {
                Ok(target) => {
                    self.reset_bare_confirm = Some(target);
                    self.open_error = None;
                }
                Err(message) => {
                    // Req 19.12: unknown profile -> error, no dialog.
                    self.reset_bare_confirm = None;
                    self.open_error = Some(message);
                }
            }
            return;
        }

        if upper == "LOG" {
            // Validates: notification-system Requirement 2.1; menu-workspace Req
            // 14.2 (CR-CH-022) -- navigate the current tab in place.
            self.nav_to_kind(ff_session::session_state::WorkspaceKind::EventLog);
            self.notification_queue
                .lock()
                .expect("queue")
                .mark_all_read();
            self.open_error = None;
            return;
        }

        if upper == "FILE CATALOGS" || upper == "CATALOGS" {
            // Validates: Requirement 1.1, 14.6; menu-workspace Req 14.2
            // (CR-CH-022) -- navigate the current tab in place.
            self.nav_to_kind(ff_session::session_state::WorkspaceKind::Files);
            self.open_error = None;
            return;
        }

        if upper == "PLUGINS" {
            // Validates: plugin-manager-ui Requirement 1.1; menu-workspace Req
            // 14.2 (CR-CH-022) -- navigate the current tab in place.
            self.nav_to_kind(ff_session::session_state::WorkspaceKind::PluginManager);
            self.open_error = None;
            return;
        }

        // Phase CV -- Extended POM options
        // Validates: Requirement 6.2 (cv-requirements.md)
        if upper == "MACROS" {
            // Validates: lua-macro-engine Requirement 12.1; menu-workspace Req
            // 14.2 (CR-CH-022) -- navigate the current tab in place.
            self.nav_to_kind(ff_session::session_state::WorkspaceKind::MacroLibrary);
            self.open_error = None;
            return;
        }

        // Match the RETRIEVE VERB by prefix (not exact string): B066's
        // key-dispatch may append the command-field content, so F12 with a
        // non-empty field arrives as `RETRIEVE <field>` and must still trigger
        // recall (B067). The handler reads `self.command_text` (the actual
        // field) -- the source of truth for the LIST trigger / empty check
        // (Req 19.1) -- rather than the merged argument.
        if upper == "RETRIEVE" || upper.starts_with("RETRIEVE ") {
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
        // == THEME -- single name-based theme command (CR-CH-024) =============
        // Validates: theme-and-appearance Requirement 17 (architecture-brief
        // Principle 2: every user action is a command).
        //   - bare `THEME` opens the Theme Editor Context (in place; the removed
        //     `THEMES` command's behaviour, Req 17.4).
        //   - `THEME <name>` selects by exact case-insensitive name, else by
        //     built-in shorthand (Req 17.2); unknown leaves the theme unchanged
        //     and reports it (Req 17.5).
        if upper == "THEME" {
            self.open_theme_editor();
            self.open_error = None;
            return;
        }
        if upper.starts_with("THEME ") {
            let arg = cmd.trim().get(6..).unwrap_or("").trim();
            let available: Vec<String> = ff_theme::list_all_themes(&self.themes_dir())
                .into_iter()
                .map(|t| t.name)
                .collect();
            match crate::theme_defaults::resolve_theme_arg(arg, &available) {
                Some(name) => {
                    // set_active_theme applies + persists and re-sets open_error
                    // only if persistence fails (Req 17.7).
                    self.open_error = None;
                    self.set_active_theme(&name);
                }
                None => {
                    self.open_error = Some(format!("THEME: '{arg}' does not exist"));
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

        // ── Chained fastpath navigation (menu-workspace Requirement 5) ───────
        // A dotted/semicolon path like `=0.K`, `3.1`, or `=0;E.T` walks a menu
        // option chain. The leading `=` is the Navigation_Origin (Req 5.7): it
        // pops to the POM before resolving segment 1, so `=0.K` resolves option
        // `0` on the POM regardless of the active Workspace. A bare dotted path
        // (no `=`) resolves its first segment against the CURRENT menu (the
        // legacy `3.1` behaviour). Each `.`/`;` separator delimits the next
        // segment; every segment is dispatched as an Option_Key through
        // handle_command, which opens the sub-menu and activates the option
        // (the same path `MENU <name> <key>` uses). Fixes B061.
        if let Some(handled) = self.try_chained_fastpath(&upper) {
            if handled {
                return;
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

        // Stage 3 (CR-CH-025): keyword-less MENU-NAME resolution. A bare token
        // that reaches here (not claimed by the current-menu Option_Key stage at
        // the top, nor by any built-in intercept above) may name a resolvable
        // menu (user `menus/<name>.toml` or built-in `pom`/`settings`). If so,
        // open it -- forwarding a trailing token as an Option_Key (Req 11.7,
        // 11.11) -- exactly as `MENU <name>` does, with no `MENU` keyword.
        if self.try_menu_name_dispatch(cmd.trim()) {
            return;
        }

        // ── Route through CommandEngine (stage 5: unresolved) ────────────
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
        // Test override (menu-workspace Req 13, CR-NR-075): isolates Menus editor
        // file operations to a TempDir. Production leaves this None.
        if let Some(dir) = &self.menus_dir_override {
            return dir.clone();
        }
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

    /// Stage 1 of the command-resolution chain (CR-CH-025, command-framework
    /// Req 8.3): when the active Workspace is a Menu_Workspace and `cmd` matches
    /// an Option_Key of the CURRENT menu, activate that option and return `true`.
    /// Returns `false` when the active tab is not a menu OR the token is not an
    /// Option_Key of it, so the caller falls through to the rest of the chain
    /// (menu-workspace Req 3.6 -- a non-matching token is NOT a terminal error).
    ///
    /// Validates: menu-workspace Requirement 3.1, 3.6, 10.1, 10.3, 10.6
    fn try_current_menu_option(&mut self, cmd: &str) -> bool {
        if self.tabs.active_tab().kind != crate::tab_state::TabKind::MenuWorkspace {
            return false;
        }
        // Extract the option (clone what we need) without holding the borrow.
        let resolved: Option<(Option<ff_command::CommandTarget>, String)> = self
            .tabs
            .active_tab()
            .menu_workspace
            .as_ref()
            .and_then(|mw| mw.menu.as_ref())
            .and_then(|menu| {
                crate::menu_workspace::commands::find_option(cmd.trim(), menu)
                    .ok()
                    .map(|opt| (opt.target.clone(), opt.command.clone()))
            });
        let Some((target, option_cmd)) = resolved else {
            // Not an Option_Key of this menu: fall through (Req 3.6). A disabled
            // option (find_option Err) also falls through; the chain will report
            // an unresolved command if nothing else matches.
            return false;
        };
        // Req 10.6: an inline [options.target] wins over `command`.
        if let Some(target) = target {
            self.dispatch_command_target(&target);
            return true;
        }
        // Req 10.1/10.3: resolve the option's command to a user-owned target and
        // dispatch it; otherwise handle the raw command string (Req 10.2).
        match self.resolve_and_dispatch_command(&option_cmd) {
            super::target_dispatch::ResolveOutcome::Dispatched => {}
            super::target_dispatch::ResolveOutcome::FallThrough => {
                self.handle_command(&option_cmd);
            }
        }
        true
    }

    /// Stage 3 of the command-resolution chain (CR-CH-025, command-framework
    /// Req 8.11, menu-workspace Req 11.11): when the FIRST token of `cmd` names a
    /// resolvable menu (user `menus/<name>.toml` or built-in `pom`/`settings`),
    /// open that Menu_Workspace -- forwarding a trailing token as an Option_Key
    /// (Req 11.7) -- and return `true`. Returns `false` when the first token is
    /// not a resolvable menu name, so the caller falls through to the error
    /// stage. Built-in commands are matched earlier in `handle_command`, so a
    /// built-in always shadows a same-named menu (Req 8.10 / 11.12).
    fn try_menu_name_dispatch(&mut self, cmd: &str) -> bool {
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
        // Open the named menu through its proper opener so the Navigation_Stack
        // (CR-CH-022) and Settings/POM chrome are preserved.
        match name.as_str() {
            "pom" => self.open_menu_by_name("pom"),
            "settings" => self.open_settings_menu(),
            other => self.open_menu_by_name(other),
        }
        // Trailing token: activate the option keyed by it on the now-open menu
        // (Req 11.7). Re-dispatch so it hits the stage-1 Option_Key lookup.
        let trailing = rest.trim();
        if !trailing.is_empty() {
            self.handle_command(trailing);
        }
        true
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
        // Code-only fallback (menu-workspace Req 12.4/12.5, CR-CH-021): if no
        // valid user pom.toml produced a menu, use the compiled Recovery_Baseline
        // so the Home Context always has its options. A file that EXISTED but
        // failed to PARSE gets a non-blocking notice; a merely absent file is
        // silent. Keeps the POM deterministic in tests with no menus dir.
        if state.menu.is_none() {
            let parse_error = state
                .load_error
                .as_deref()
                .filter(|e| e.starts_with("Menu file error"))
                .map(str::to_string);
            state.menu = Some(crate::menu_workspace::defaults::recovery_pom_menu());
            state.load_error = None;
            if let Some(err) = parse_error {
                self.notify_menu_fallback("pom.toml", &err);
            }
        }
        let idx = self.tabs.active_index();
        if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
            tab.menu_workspace = Some(state);
        }
    }

    /// Handle a chained fastpath navigation path (menu-workspace Requirement 5).
    ///
    /// A chained path walks a menu Option_Key chain across one or more levels,
    /// e.g. `=0.K` (POM option 0 -> Settings, then option K -> KEYS), `3.1`, or
    /// `=0;E.T` (mixed separators). Segments are separated by `.` (collapse /
    /// STOP) or `;` (push / PUSH); this method splits on BOTH.
    ///
    /// Semantics:
    /// - A leading `=` is the Navigation_Origin (Req 5.7): the shell pops to the
    ///   POM before resolving the first segment, so `=<k>...` always resolves
    ///   `<k>` against the POM regardless of the active Workspace.
    /// - A bare dotted path (no `=`) resolves its first segment against the
    ///   CURRENT menu (the legacy `3.1` behaviour, Requirement 19.4). To avoid
    ///   hijacking ordinary dotted input (dataset names, `abc.def`), a bare path
    ///   is only treated as a fastpath when its first segment is a single digit.
    /// - Each segment is dispatched as an Option_Key through `handle_command`,
    ///   which opens the target sub-menu and activates the option (the same code
    ///   path `MENU <name> <key>` and the single-segment fastpath use).
    /// - Depth is capped at 4 segments (Req 5.2); deeper paths report an error.
    ///
    /// Returns `Some(true)` when the input was a chained path and was handled
    /// (the caller must return), `Some(false)`/`None` when the input is NOT a
    /// chained path and the caller should fall through to the rest of the chain.
    ///
    /// Validates: menu-workspace Requirement 5.1, 5.2, 5.4, 5.7; Requirement 19.4
    pub(super) fn try_chained_fastpath(&mut self, upper: &str) -> Option<bool> {
        let has_sep = upper.contains('.') || upper.contains(';');
        let is_origin = upper.starts_with('=');
        // Not a chained path unless it has a separator (or is a bare `=<key>`,
        // which the single-segment fastpath below already handles -- so require
        // a separator here). A leading `.`/`;` is not a path.
        if !has_sep || upper.starts_with('.') || upper.starts_with(';') {
            return None;
        }

        // Strip the Navigation_Origin marker; split into segments on either
        // separator, preserving order. splitn-style cap at 5 so >4 is an error.
        let body = upper.strip_prefix('=').unwrap_or(upper);
        let segments: Vec<&str> = body
            .split(['.', ';'])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if segments.is_empty() {
            return None;
        }
        if segments.len() > 4 {
            self.open_error = Some("Chained path exceeds maximum depth of 4 segments.".to_string());
            return Some(true);
        }

        // A bare dotted path (no `=`) is only a fastpath when the first segment
        // is a single digit -- otherwise ordinary dotted text (`abc.def`, DSNs)
        // would be hijacked. An `=`-origin path is always a fastpath.
        let first = segments[0];
        if !is_origin && !(first.len() == 1 && first.chars().all(|c| c.is_ascii_digit())) {
            return None;
        }

        // Navigation_Origin: `=` pops to the POM before resolving segment 1.
        if is_origin && self.tabs.active_tab().kind != crate::tab_state::TabKind::PrimaryOptionMenu
        {
            self.tabs.insert_pom_tab(&self.runtime);
            self.ensure_pom_menu_loaded();
        }

        // Dispatch each segment as an Option_Key. Every re-entry routes through
        // the stage-1 current-menu Option_Key lookup / stage-3 menu-name path,
        // which opens the sub-menu and activates the option in place.
        for segment in segments {
            self.handle_command(segment);
        }
        Some(true)
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

        // CR-CH-022 Req 14.2: navigate the current tab in place (push). The
        // Theme editor's working state is already set on the shell above.
        self.navigate_to(
            ff_session::session_state::WorkspaceDescriptor::CustomWorkspace {
                workspace_kind: ff_session::session_state::WorkspaceKind::CommandConfigurator,
                params: {
                    let mut p = ff_session::session_state::DescriptorParams::new();
                    p.insert(
                        "editor".to_string(),
                        ff_session::session_state::DescriptorValue::from("theme"),
                    );
                    p
                },
            },
            true,
        );
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
                    WorkspaceKind::Config => {
                        let namespace = match params.get("namespace") {
                            Some(DescriptorValue::String(ns)) => Some(ns.clone()),
                            _ => None,
                        };
                        self.tabs.open_config_panel_tab(&self.runtime);
                        self.config_panel.filter = match &namespace {
                            Some(ns) => format!("{ns}."),
                            None => String::new(),
                        };
                        let title = match &namespace {
                            Some(ns) => format!("[CONFIG:{ns}]"),
                            None => "[CONFIG]".to_string(),
                        };
                        self.tabs.active_tab_mut().title = title;
                        self.config_panel.namespace_filter = namespace;
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
