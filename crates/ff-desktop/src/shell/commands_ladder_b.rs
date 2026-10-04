//! # Shell Command Dispatch -- workspace-open / navigation ladder (CONFIG..RETRIEVE)
//!
//! An extracted contiguous segment of the `handle_command` dispatch ladder
//! (TASK 2.2, pure code movement -- no behaviour change). `try_commands_b1`
//! (CONFIG..RETRIEVE) returns `true` when it handled the command. Branch order
//! is identical to the original ladder. It was formerly dispatched before
//! `try_commands_b2`; that segment was retired (CR-CH-053 E7) once its editor
//! verbs migrated to the FFEDIT Command_Environment and its sole remaining arm
//! (THEME) folded back inline into `run_command_ladder`.

use ff_keys::RetrieveResult;

use super::helpers::*;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Extracted ladder segment of `handle_command` (TASK 2.2, pure code
    /// movement -- no behaviour change). Returns `true` when a branch
    /// handled the command (the caller then returns), `false` to fall
    /// through to the next segment. Branch order is preserved exactly.
    pub(super) fn try_commands_b1(&mut self, cmd: &str, upper: &str) -> bool {
        // SUPERSEDED-ARM NOTE (B080 Step 2): the CustomWorkspace / navigation
        // verbs in this segment -- CONFIG, FILES/=FILES, GSEARCH/SEARCH, COMMANDS,
        // MENUS, LOG, FILE CATALOGS/CATALOGS, PLUGINS, MACROS -- are now resolved
        // by `builtin_workspace_target` through the front door's `resolve_target`
        // BEFORE this ladder is reached, so each of those arms is an unreachable
        // fallback kept for per-step rollback safety (deletion is a Step-2
        // follow-up). The `COMMAND` / `COMMAND <pos>`, SNAPSHOT, CAPTURE, RESET
        // BARE, and RETRIEVE arms are NOT migrated and remain reachable.
        // ── CONFIG [<namespace>] -- flat configuration-key browser (Req 20) ──
        // SUPERSEDED (B080 Step 2) -> CustomWorkspace("config"); body unchanged.
        if let Some(arg) = verb_arg(cmd, "CONFIG") {
            // Bare CONFIG -> unfiltered All-Settings flat list; CONFIG <namespace>
            // -> the flat list pre-filtered to `<namespace>.` (lowercased).
            let ns = arg.to_lowercase();
            if ns.is_empty() {
                self.open_config_view(None);
            } else {
                self.open_config_view(Some(ns));
            }
            self.open_error = None;
            return true;
        }

        if upper == "=FILES" || upper == "FILES" {
            // Validates: Requirement 19.1-19.3; menu-workspace Req 14.2
            // (CR-CH-022) -- navigate the CURRENT tab in place; never a new tab.
            self.nav_to_kind(ff_session::session_state::WorkspaceKind::FileExplorer);
            self.open_error = None;
            return true;
        }

        if upper == "GSEARCH" || upper == "SEARCH" {
            // Validates: global-search Requirement 1.2
            self.open_or_focus_search_panel();
            self.open_error = None;
            return true;
        }

        // CR-NR-095 (Req 8.9-8.13): the ISPF `COMMAND` command moves the active
        // Workspace's command line. `COMMAND TOP` / `COMMAND BOTTOM` set the
        // position; bare `COMMAND` toggles it. It updates the active instance's
        // Kind Command_Line_Position via the SAME position-setting seam the Kinds
        // Editor uses (persists to the Kind file), so it takes effect next frame
        // and survives restart. Checked BEFORE `COMMANDS` (plural); the two are
        // disjoint (`"COMMANDS" != "COMMAND"` and does not start with `"COMMAND "`),
        // so neither shadows the other (Req 8.12).
        if upper == "COMMAND" {
            self.toggle_active_command_line_position();
            return true;
        }
        if let Some(arg) = upper.strip_prefix("COMMAND ") {
            use crate::workspace_kind::CommandLinePosition;
            let arg = arg.trim();
            match arg {
                "TOP" => self.set_active_command_line_position(CommandLinePosition::Top),
                "BOTTOM" => self.set_active_command_line_position(CommandLinePosition::Bottom),
                other => {
                    // Req 8.9: unknown argument -> non-blocking error, unchanged.
                    self.open_error = Some(format!(
                        "COMMAND: '{other}' is not a valid position (TOP or BOTTOM)"
                    ));
                }
            }
            return true;
        }

        if upper == "COMMANDS" {
            // Validates: command-configurator Requirement 2.1, 2.7; menu-workspace
            // Req 14.2 (CR-CH-022) -- navigate the current tab in place.
            self.nav_to_kind(ff_session::session_state::WorkspaceKind::CommandConfigurator);
            self.open_error = None;
            return true;
        }

        // CR-CH-024: the `THEMES` command is removed; bare `THEME` (handled
        // below) now opens the Theme Editor. No `THEMES` intercept remains.

        if upper == "MENUS" {
            // Validates: menu-workspace Requirement 13.1 (CR-NR-075) -- open the
            // Menus Editor Context. Both the POM `M` row and the Settings `M` row
            // dispatch this command, so both entry points open the editor.
            self.open_menus_editor();
            self.open_error = None;
            return true;
        }

        if upper == "SNAPSHOT" || upper.starts_with("SNAPSHOT ") {
            // Validates: screen-snapshot-scrm Requirement 4.1-4.6, 6.1-6.3, 2.3
            // (CR-NR-098, Wave 1). SNAPSHOT [TEXT|ANSI|MARKDOWN|MD|HTML|YAML|AI]
            // renders the active Context's logical screen to selectable text and
            // copies it to the clipboard. Routed through the single handle_command
            // path like every other built-in verb (command parity).
            let arg = cmd.trim().get("SNAPSHOT".len()..).unwrap_or("").trim();
            self.handle_snapshot(arg);
            return true;
        }

        if upper == "CAPTURE" || upper.starts_with("CAPTURE ") {
            // Validates: screen-snapshot-scrm Requirement 7, 8, 9.1, 11.1
            // (CR-NR-098, Wave 2). CAPTURE START/STOP/SCREEN/STATUS/LIST/PURGE/
            // REPLAY drive the SCRM session. Routed through the single
            // handle_command path like every other built-in verb.
            let rest = cmd.trim().get("CAPTURE".len()..).unwrap_or("").trim();
            self.handle_capture(rest);
            return true;
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
                    // B077: focus the Cancel button on the first frame the dialog
                    // is shown (modal focus trap, accessibility Req 2.3).
                    self.reset_bare_focus_requested = true;
                    self.open_error = None;
                }
                Err(message) => {
                    // Req 19.12: unknown profile -> error, no dialog.
                    self.reset_bare_confirm = None;
                    self.open_error = Some(message);
                }
            }
            return true;
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
            return true;
        }

        if upper == "FILE CATALOGS" || upper == "CATALOGS" {
            // Validates: Requirement 1.1, 14.6; menu-workspace Req 14.2
            // (CR-CH-022) -- navigate the current tab in place.
            self.nav_to_kind(ff_session::session_state::WorkspaceKind::Files);
            self.open_error = None;
            return true;
        }

        if upper == "PLUGINS" {
            // Validates: plugin-manager-ui Requirement 1.1; menu-workspace Req
            // 14.2 (CR-CH-022) -- navigate the current tab in place.
            self.nav_to_kind(ff_session::session_state::WorkspaceKind::PluginManager);
            self.open_error = None;
            return true;
        }

        // Phase CV -- Extended POM options
        // Validates: Requirement 6.2 (cv-requirements.md)
        if upper == "MACROS" {
            // Validates: lua-macro-engine Requirement 12.1; menu-workspace Req
            // 14.2 (CR-CH-022) -- navigate the current tab in place.
            self.nav_to_kind(ff_session::session_state::WorkspaceKind::MacroLibrary);
            self.open_error = None;
            return true;
        }

        // Match the RETRIEVE VERB by prefix (not exact string): B066's
        // key-dispatch may append the command-field content, so F12 with a
        // non-empty field arrives as `RETRIEVE <field>` and must still trigger
        // recall (B067). The handler reads `self.command_text` (the actual
        // field) -- the source of truth for the LIST trigger / empty check
        // (Req 19.1) -- rather than the merged argument.
        if upper == "RETRIEVE" || upper.starts_with("RETRIEVE ") {
            // Drive recall through the command-processor-owned history
            // (CR-NR-084). The handler still reads `self.command_text` (the
            // actual field) as the LIST/empty trigger source of truth (Req 19.1).
            let cmd_text = self.command_text.clone();
            match self.command_line_history.retrieve(&cmd_text) {
                RetrieveResult::Recalled { command } => {
                    // CR-CH-033 (Req 13.4): RETRIEVE returns Set(<recalled>) so the
                    // recalled command is placed in the field by the outcome pass.
                    self.pending_command_line_outcome = Some(
                        crate::shell::command_line_outcome::CommandLineOutcome::Set(command),
                    );
                }
                RetrieveResult::ShowList { entries } => {
                    // Validates: Requirement 19.1 — show history list overlay
                    self.show_history_list = Some(entries);
                    self.command_text.clear();
                    // The field was cleared and the picker opened; keep it as-is.
                    self.pending_command_line_outcome =
                        Some(crate::shell::command_line_outcome::CommandLineOutcome::Leave);
                }
                RetrieveResult::HistoryEmpty | RetrieveResult::NoOlderHistory => {
                    // Nothing to recall: leave the field untouched (do not clear).
                    self.pending_command_line_outcome =
                        Some(crate::shell::command_line_outcome::CommandLineOutcome::Leave);
                }
            }
            return true;
        }
        false
    }
}
