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
    /// CR-CH-116 (navigation-commands Req 21): `X` is the FFCMD seed alias of
    /// EXIT, so a bare `X` on a non-editor Context is rewritten to `EXIT` by the
    /// front door BEFORE this family runs and terminates the app here. `=X`
    /// reinitialises to the POM (FFCMD) then runs `X` -> EXIT (also app-exit).
    /// The literal verbs EXIT / QUIT / LOGOFF remain unconditional app-exit. EXIT
    /// terminates FFWB on every non-editor Context; PF3 stays END.
    ///
    /// Validates: command-framework Requirement 8.4; Requirement 20.3;
    /// navigation-commands Requirement 21.3
    pub(super) fn try_exit_family(&mut self, upper: &str) -> bool {
        // The EXIT family is EXIT / QUIT / LOGOFF -- the UNCONDITIONAL app-exit
        // verbs. CR-CH-116: `X` is NOT a member here; it reaches EXIT via the
        // FFCMD X -> EXIT seed rewritten at the front door, so by the time this
        // runs a non-editor `X` has already become `EXIT`.
        if upper == "EXIT" || upper == "QUIT" || upper == "LOGOFF" {
            // Validates: Requirement 20.3 -- LOGOFF is an alias for EXIT
            if upper == "LOGOFF" {
                let msg = self.format_logoff_message();
                // Req 7.28: the logoff report is informational (Message_Line).
                self.post_info(msg);
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
                self.open_error = Some(ff_i18n::t("ff-desktop-shell-msg-edit-requires-path"));
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
                self.open_error = Some(ff_i18n::t("ff-desktop-shell-msg-dock-not-detached"));
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

        // ── SAVE / CANCEL / REFRESH -- CR-CH-071 editable-Context verbs ──────
        // Validates: command-environments Requirement 20a.1-20a.4, 20a.6, 20a.14.
        //
        // These reach the FFCMD base (this ladder) only AFTER the active-env
        // claim in `run_command_ladder`: on an editor BUFFER the active env is
        // FFEDIT, which claims SAVE first (active-wins, Req 20a.7), so SAVE here
        // is only reached on a non-buffer Context. SAVE persists + STAYS; REFRESH
        // discards + reloads + STAYS; CANCEL discards then leaves via `nav_end`.
        // On a NON-editable Context (`editable_context_kind()` is None) SAVE /
        // REFRESH are no-ops and CANCEL is plain navigation (Req 20a.14).
        if upper == "SAVE" {
            if self.editable_context_kind().is_some() {
                // Persist and STAY. The error (if any) is surfaced inline by the
                // Context's persist; do not navigate.
                let _ = self.editable_persist();
            }
            // Non-editable: SAVE is a no-op here (no FFCMD-base save), matching
            // today's behaviour (Req 20a.14).
            self.open_error = None;
            return true;
        }
        if upper == "REFRESH" {
            if self.editable_context_kind().is_some() {
                // Discard unsaved edits, then reload the last-saved state, and
                // STAY in the Context (Req 20a.4).
                self.editable_discard();
                self.editable_reload();
            }
            // Non-editable: no-op (Req 20a.14).
            self.open_error = None;
            return true;
        }
        if upper == "CANCEL" {
            // CANCEL = discard the working changes (NO save) then leave one level
            // up via the per-tab Navigation_Stack (Req 20a.1, 20a.3). On a
            // non-editable Context `editable_discard` is a no-op, so CANCEL is
            // plain `nav_end` navigation exactly as before (Req 20a.14).
            self.editable_discard();
            self.nav_end();
            self.open_error = None;
            return true;
        }

        // ── END — Validates: Requirement 17.1, 17.2; CR-CH-071 Req 20a.1 ─────
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
            // CR-CH-071 (Req 20a.1): on an editable Context END = SAVE the working
            // changes THEN leave. If the save FAILS validation, STAY in the
            // Context with the error surfaced inline and the store UNCHANGED (do
            // NOT navigate). On a non-editable Context `editable_context_kind()`
            // is None, so END is plain navigation exactly as before (Req 20a.14).
            if self.editable_context_kind().is_some() {
                match self.editable_persist() {
                    Ok(()) => {
                        self.nav_end();
                        self.open_error = None;
                    }
                    Err(_) => {
                        // Save failed: stay put; the Context surfaced the error.
                    }
                }
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

        // ── RETURN -- Validates: menu-workspace Requirement 14.10 (CR-CH-052);
        //    CR-CH-071 Req 20a.3 ──────────────────────────────────────────────
        if upper == "RETURN" {
            // CR-CH-071 (Req 20a.1, 20a.3): RETURN on an editable Context is a
            // CANCEL-shaped verb -- DISCARD the working changes (no save) then
            // leave. On a non-editable Context `editable_discard` is a no-op.
            self.editable_discard();
            // CR-CH-116 (navigation-commands Req 21.2): RETURN unwinds the whole
            // Navigation_Stack to the tab's PRIMARY PANEL (Tab_Visual_Root) in one
            // step; at the root it closes the Workspace (exit when last,
            // CR-CH-016). Bare `X` no longer converges here -- `X` is now EXIT.
            // The `=` family remains the only way to reinitialise to the POM.
            self.nav_return();
            self.open_error = None;
            return true;
        }

        // CR-CH-116 (navigation-commands Req 21.4, menu-workspace Req 14.13): the
        // bare `X` ladder arm (which called `nav_x`) is RETIRED. `X` is now the
        // FFCMD seed alias of EXIT: the front door's `rewrite_verb_to_root`
        // rewrites `X` -> `EXIT` against the active (FFCMD) alias table, and the
        // prelude's `try_exit_family` dispatches `file.exit`. On an editor the
        // FFEDIT environment claims bare `X` as EXCLUDE first (active-wins), so
        // the X -> EXIT seed is not reached there. `=X` reinitialises to the POM
        // (FFCMD) then runs `X` -> EXIT. There is no command named `X`.
        false
    }
}
