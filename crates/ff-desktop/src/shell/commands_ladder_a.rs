//! # Shell Command Dispatch -- built-in intercepts (stage 2, EXIT..RETURN)
//!
//! An extracted contiguous segment of the `handle_command` dispatch ladder
//! (TASK 2.2, pure code movement -- no behaviour change). `try_commands_a`
//! returns `true` when it handled the command. Branch order is identical to
//! the original ladder.

use ff_command::{CommandParams, CommandResult};

use super::helpers::*;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Extracted ladder segment of `handle_command` (TASK 2.2, pure code
    /// movement -- no behaviour change). Returns `true` when a branch
    /// handled the command (the caller then returns), `false` to fall
    /// through to the next segment. Branch order is preserved exactly.
    /// The EXIT family (EXIT / QUIT / LOGOFF) classification -- the UNCONDITIONAL
    /// app-exit verbs (CR-CH-052). Extracted so the single front door
    /// (`dispatch_command_string`) can run it at its ORIGINAL ladder precedence --
    /// BEFORE the POM Option_Key fastpath -- without re-ordering the chain. The
    /// front door calls this before `resolve_pom_option_key`; `try_commands_a`
    /// also calls it first (so the fall-through ladder path is unchanged), but the
    /// front door returns before reaching the ladder, so it never double-runs.
    ///
    /// `X` and `=X` are NO LONGER members of this family (CR-CH-052): `X` is the
    /// uniform FFCMD close verb (`nav_x`), and `=X` is handled by the front-door
    /// `=` step followed by `X` at the POM root. Only EXIT / QUIT / LOGOFF are
    /// unconditional app-exit.
    ///
    /// Validates: command-framework Requirement 8.4; Requirement 20.3
    pub(super) fn try_exit_family(&mut self, upper: &str) -> bool {
        // CR-CH-052: the EXIT family is now EXIT / QUIT / LOGOFF only -- the
        // UNCONDITIONAL app-exit verbs. `X` is NO LONGER an exit verb: it is the
        // uniform FFCMD close verb (`nav_x`, collapse-to-visual-root / close
        // workspace, exit only when last). `=X` is likewise gone: the front-door
        // `=` step reinitialises to the POM and the trailing `X` closes the
        // Workspace at the empty root -- `=X` is NOT app-exit.
        if upper == "EXIT" || upper == "QUIT" || upper == "LOGOFF" {
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
            return true;
        }
        false
    }

    pub(super) fn try_commands_a(&mut self, cmd: &str, upper: &str) -> bool {
        // ── Shell-level intercepts (stage 2: built-in command / Command_ID) ──
        // EXIT family first, at its original precedence. (The front door also
        // runs this before the POM fastpath; it returns before reaching here, so
        // this is not a double-run -- it keeps the fall-through ladder faithful.)
        if self.try_exit_family(upper) {
            return true;
        }

        // EDIT <path>: DEFERRED from the Step 3 Function family (B080). The open
        // runs via `dispatch.execute_command("file.open", { path })`, which
        // carries the path param to the registered FileOpenHandler. The
        // Function-target dispatch arm calls `handle_command(command_id)` with NO
        // params, so classifying EDIT to `Function { "file.open" }` would drop the
        // path and never reach the handler (Req 8.4 regression). EDIT therefore
        // stays on the ladder; revisit if the Function arm learns to carry params.
        if let Some(rest) = verb_arg(cmd, "EDIT") {
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
            return true;
        }

        // CR-CH-044 (command-framework Req 15.2; menu-workspace Req 20.5 revised):
        // the hardcoded `POM` menu-open intercept is RETIRED. `POM` now resolves
        // as a Menu_Name (`Menu { name: "pom" }`) through the single
        // Target_Resolution classifier -- it falls through to the stage-3
        // `try_menu_name_dispatch` below, which opens/returns to the Home Context
        // via `open_menu_by_name("pom")` (in place, no duplicate tab), exactly as
        // `SETTINGS` resolves to the settings menu. This makes opening the POM one
        // path with every other menu, with no POM special case in dispatch. The
        // POM tab short label / Title_Line still derive from the menu name/title
        // (CR-CH-042). `START` (below) remains the sole tab-creator.

        if let Some(arg) = verb_arg(cmd, "START") {
            // Validates: menu-workspace Requirement 14.8, 14.9 (CR-CH-022) --
            // START is the ONLY tab-creator. Forms:
            //   START            -> new tab rooted at the POM (empty stack)
            //   START =<path>    -> new POM tab, then drill along <path> (POM on
            //                       the stack via the `=` origin rule)
            //   START <arg>      -> new tab rooted DIRECTLY at <arg> (empty stack)
            self.start_new_workspace(arg);
            self.open_error = None;
            return true;
        }

        // MENU / MENU <name> and the menu.open Command_ID form.
        // Validates: menu-workspace Requirement 11.1, 11.2, 11.4, 11.6
        if let Some(arg) = verb_arg(cmd, "MENU") {
            self.open_menu_by_name(arg);
            return true;
        }
        if let Some(arg) = verb_arg(cmd, "MENU.OPEN") {
            self.open_menu_by_name(arg);
            return true;
        }

        // CLOSE: DEFERRED from the Step 3 Function family (B080). CLOSE is a bare
        // shell op (`tabs.close_tab`), not a registered Command_ID, so it does not
        // round-trip through the Function arm's `handle_command(command_id)`.
        // Stays on the ladder; a later step can model it as a Function once a
        // close Command_ID + param-carrying Function dispatch exist.
        if upper == "CLOSE" {
            // Validates: Requirement 14.11 — CLOSE closes the current tab
            let idx = self.tabs.active_index();
            self.tabs.close_tab(idx);
            self.open_error = None;
            return true;
        }

        // === DOCK -- Validates: menu-and-statusbar Req 18.13 (CR-NR-088) ======
        // Re-dock the CURRENT Detached_Workspace into the Primary_Window's tab
        // bar at its origin index (the faithful remove+reinsert from Req 18.9),
        // clearing its floating state and dropping its FloatingTab. Typed in the
        // detached window's own command line it runs under the CR-CH-036 swap, so
        // "the active tab" is the detached one. A no-op with a status message when
        // the active workspace is not detached. This is the explicit re-attach
        // (the window Close button now runs RETURN, not redock -- CR-CH-037).
        if upper == "DOCK" {
            let active_id = self.tabs.active_tab().id;
            if !self.tabs.active_tab().is_floating {
                self.open_error = Some("DOCK: the current workspace is not detached".to_string());
                return true;
            }
            if let Some(ft_pos) = self
                .detach_split
                .floating_tabs
                .iter()
                .position(|ft| ft.tab_id == active_id)
            {
                let ft = self.detach_split.floating_tabs.remove(ft_pos);
                if let Some(tab_idx) = self.tabs.index_of_id(active_id) {
                    self.tabs.tabs_mut()[tab_idx].is_floating = false;
                    // CR-NR-093 Slice 2c.4 (Req 14.16): when the Workspace is
                    // split, re-attach into a tree LEAF (the origin leaf if the
                    // tab still sits there, else the focused leaf) rather than a
                    // flat origin index. When unsplit, keep the exact prior flat
                    // re-dock (move to the recorded origin index).
                    if self.tabs.is_split() {
                        self.tabs.dock_tab_into_leaf(active_id);
                    } else {
                        self.tabs.move_tab(tab_idx, ft.origin_index);
                    }
                }
                self.open_error = None;
            } else {
                // is_floating but no FloatingTab record: clear the flag defensively.
                let idx = self.tabs.active_index();
                self.tabs.tabs_mut()[idx].is_floating = false;
                self.open_error = None;
            }
            return true;
        }

        // ── HELP / F1 — display context-sensitive help (CR-NR-097).
        //    Resolves against the single shell-owned Help_Topic_Registry loaded
        //    once at startup, opens the Help Context on the resolved topic, and
        //    on a miss records it + shows the index with a message. Bare HELP /
        //    F1 uses the Cursor_Context (CR-NR-079: a focused option resolves
        //    that option's topic). Routed through the single command path; not
        //    recorded in history / undo (Req 18.6, 18.7).
        //    Validates: context-help Requirement 18.1-18.7, 19.1-19.4.
        if upper == "HELP" {
            self.open_help("");
            return true;
        }
        if let Some(arg) = verb_arg(cmd, "HELP") {
            self.open_help(arg);
            return true;
        }

        // ── KEYS / KINDS arms DELETED (CR-CH-052, B080 Step 2 follow-up) ─────
        // KEYS -> CustomWorkspace("keys") and KINDS -> CustomWorkspace("kinds")
        // are now resolved by `builtin_workspace_target` through the front door's
        // `resolve_target` BEFORE this ladder is reached (the CustomWorkspace
        // dispatch arm calls the identical shell open method). The nav callers
        // that formerly reached these via `handle_command` were rerouted to the
        // front door, so these arms are truly dead and have been removed.

        // ── PFSHOW — Validates: Requirement 12.1-12.3, 12.8-12.9, 12.13 ————————
        // CR-CH-046: bare PFSHOW cycles Off -> Base -> Shift -> Ctrl -> Alt -> Off;
        // PFSHOW BASE/SHIFT/CTRL/ALT jump to a scope (and show); ON/OFF keep their
        // meaning. Unknown args leave the mode unchanged with a non-fatal error.
        if upper == "PFSHOW" || upper.starts_with("PFSHOW ") {
            let arg = upper.strip_prefix("PFSHOW").unwrap_or("").trim();
            self.handle_pfshow(arg);
            return true;
        }

        // ── END — Validates: Requirement 17.1, 17.2 ———————————————————————
        if upper == "END" {
            // CR-NR-092 (B046 Slice 2b, Req 13.9): when the Workspace area is
            // split, END collapses the split back to a single Tab_Group (the
            // focused group survives) BEFORE the normal Navigation_Stack pop.
            // This makes END the keyboard-friendly "close this region" verb while
            // split; once unsplit, END resumes its usual per-tab behaviour.
            if self.tabs.is_split() {
                self.tabs.unsplit();
                self.open_error = None;
                return true;
            }
            // Validates: menu-workspace Requirement 14.4, 14.5 (CR-CH-022) --
            // END pops one level of the active tab's Navigation_Stack and
            // reconstructs the parent Context in place; an empty stack closes the
            // Workspace (terminates when last, preserving CR-CH-016). This
            // replaces the former per-kind END rules and the three ad-hoc
            // mechanisms (pending_return_to_pom / namespace_filter /
            // opened_from_settings).
            self.nav_end();
            self.open_error = None;
            return true;
        }

        // ── RETURN -- Validates: menu-workspace Requirement 14.10 (CR-CH-052) ──
        if upper == "RETURN" {
            // CR-CH-052 (supersedes CR-CH-038): RETURN collapses the whole
            // Navigation_Stack to the TAB'S VISUAL ROOT in one step (converges
            // with bare X); at the root it closes the Workspace (exit when last,
            // CR-CH-016). It no longer jumps to the global POM -- the `=` family
            // is now the only way to reinitialise the stack to the POM.
            self.nav_return();
            self.open_error = None;
            return true;
        }

        // ── X -- Validates: menu-workspace Requirement 14.13, 14.14 (CR-CH-052) ─
        if upper == "X" {
            // Uniform FFCMD close verb: non-empty stack collapses to the
            // Tab_Visual_Root; empty stack closes the Workspace (exit when last).
            // Identical for every context including the POM -- NO special case.
            // The editor Command_Environment claims bare `X` as EXCLUDE first
            // (CR-CH-053 E8), so this arm is only reached on non-editor contexts
            // (and for `=X`, where the front-door `=` step already reinitialised
            // to the POM root so `X` closes the Workspace).
            self.nav_x();
            self.open_error = None;
            return true;
        }
        false
    }
}
