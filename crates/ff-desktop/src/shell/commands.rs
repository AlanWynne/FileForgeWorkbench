//! # Shell Command Dispatch
//!
//! `handle_command()` -- parses and routes every primary command entered in the
//! Command ===> field or dispatched programmatically.

use ff_command_semantics::StatusKind;

use super::helpers::*;

impl WorkbenchShell {
    /// Run a command submitted from the `Command ===>` field (Enter or a
    /// key-forwarded F-key), then apply its Command_Line_Outcome to the field.
    ///
    /// This is the SINGLE decision point for what the command line holds after
    /// an invocation, shared by both input paths (command-framework Req 13.1,
    /// 9.9 revised, 9.10). The field is left INTACT during dispatch (so commands
    /// that read it -- notably RETRIEVE, Req 19.1 -- still work), then set to the
    /// effective outcome:
    /// - an explicit outcome a command stashed in `pending_command_line_outcome`
    ///   (RETRIEVE -> `Set(recalled)` / `Leave`); else
    /// - the DEFAULT: `Clear` on success, `Restore` (the executed text) when the
    ///   command left an `open_error` -- which covers BOTH an unresolved command
    ///   (typo kept for correction, Req 13.2) and a resolved-but-failed command
    ///   (e.g. FIND-not-found comes back, Req 13.3).
    ///
    /// Validates: command-framework Requirement 13.1, 13.2, 13.3, 9.9, 9.10
    pub(super) fn run_command_line(&mut self, cmd: &str) {
        let original = cmd.trim().to_string();
        self.begin_command_line();
        self.dispatch_command_string(&original);
        self.finish_command_line(&original);
    }

    /// Reset the per-dispatch Command_Line_Outcome state before a command-line
    /// invocation: no stashed outcome, no stale error. Shared by the Enter path
    /// (`run_command_line`) and the key-forward path (`dispatch_key_command`) so
    /// the outcome is computed from THIS invocation only.
    ///
    /// Validates: command-framework Requirement 13.1
    pub(super) fn begin_command_line(&mut self) {
        self.pending_command_line_outcome = None;
        self.open_error = None;
    }

    /// Apply the effective Command_Line_Outcome to the `Command ===>` field after
    /// a command-line invocation dispatched with the field left intact. An
    /// explicit stash wins; else the default is `Clear` on success and `Restore`
    /// (the executed `original`) when the command left an `open_error` -- which
    /// covers BOTH an unresolved typo (kept for correction, Req 13.2) and a
    /// resolved-but-failed command (e.g. FIND-not-found comes back, Req 13.3).
    ///
    /// Validates: command-framework Requirement 13.1, 13.2, 13.3
    pub(super) fn finish_command_line(&mut self, original: &str) {
        use crate::shell::command_line_outcome::CommandLineOutcome;
        let effective = self.pending_command_line_outcome.take().unwrap_or_else(|| {
            if self.open_error.is_some() {
                CommandLineOutcome::Restore
            } else {
                CommandLineOutcome::Clear
            }
        });
        match effective {
            CommandLineOutcome::Clear => self.command_text.clear(),
            CommandLineOutcome::Restore => self.command_text = original.to_string(),
            CommandLineOutcome::Set(text) => self.command_text = text,
            // Keep whatever the command itself left in the field.
            CommandLineOutcome::Leave => {}
        }
    }

    /// Parse and route a command string through the prelude + ladder, WITHOUT
    /// the Step-2 `resolve_target` stage.
    ///
    /// This is BOTH the direct-caller entry (menu-button clicks, Command Palette,
    /// `ShellRequest::Command`, `start_new_workspace`, chained segments, the POM
    /// recursion) AND the Function-execution terminal: `dispatch_command_target`
    /// executes a resolved `Function` target by calling `handle_command(id)` for
    /// an observable result identical to typing it. For that reason the
    /// `resolve_target` stage does NOT live here -- if it did, a resolved
    /// registered Command_ID (e.g. `file.exit`) would re-resolve to a `Function`
    /// target and recurse forever. The single `resolve_target` stage lives in the
    /// front door `dispatch_command_string` (dispatch.rs), between the prelude and
    /// the ladder, so every seam reaches it once at the correct precedence (B080
    /// Step 2).
    pub(super) fn handle_command(&mut self, cmd: &str) {
        let upper = cmd.trim().to_uppercase();
        if self.run_command_prelude(cmd, &upper) {
            return;
        }
        self.run_command_ladder(cmd, &upper);
    }

    /// The dispatch prelude: history record, Req 8.3 stage-1 current-menu
    /// Option_Key, the EXIT family, and the POM / chained fastpaths, in their
    /// ORIGINAL ladder precedence. Returns `true` when a stage handled the
    /// command (the caller must return); `false` to continue to `resolve_target`
    /// (front door) or the ladder (direct `handle_command`).
    ///
    /// Shared by `handle_command` (direct callers) and the front door
    /// `dispatch_command_string` so the one prelude is defined once and runs at
    /// the same point for every seam (B080 Step 2).
    ///
    /// Validates: command-framework Requirement 8.3, 8.4
    pub(super) fn run_command_prelude(&mut self, cmd: &str, upper: &str) -> bool {
        // Record every submitted command in the RETRIEVE history exactly once,
        // BEFORE the branch handlers run, so that shell-intercept commands
        // (THEME, CAPS, NULLS, STATS, LOCK, ...) are recallable via F12 just like
        // command-engine commands. RETRIEVE itself is excluded; empty input and
        // consecutive duplicates are de-duplicated by the ring. `record` handles
        // the RETRIEVE-verb exclusion and resets the retrieve pointer for any
        // non-RETRIEVE line (CR-NR-084, Req 19.5).
        self.command_line_history.record(cmd);

        // Stage 1 (CR-CH-025, command-framework Req 8.3): current-menu Option_Key
        // lookup. When the active Workspace is a Menu_Workspace and the typed
        // string matches an Option_Key of the CURRENT menu, activate that option
        // (menu-workspace Req 3.1). A NON-matching token falls through (Req 3.6).
        if self.try_current_menu_option(cmd) {
            return true;
        }

        // EXIT family (EXIT / QUIT / X / RETURN / LOGOFF, and `=X`) -- the FFCMD
        // exit verbs. These resolve here BEFORE `resolve_pom_option_key` so the
        // POM's `X` -> Return option never shadows the base exit (the locked Req
        // 8.4 precedence), EXCEPT when the active environment should get first
        // crack at a BARE editor verb:
        //
        // CR-CH-053 E8 (active-wins): WHEN an editor Context is focused AND the
        // command is NOT `=`-prefixed, the EXIT family is SKIPPED here so bare `X`
        // falls through to the ladder's FFEDIT claim (FFEDIT owns X -> EXCLUDE,
        // Req 5.1). A `=`-prefixed form (e.g. `=X`) is the universal escape hatch
        // (Req 3.2/3.2a): it is NOT skipped, so `=X` always reaches FFCMD's exit
        // regardless of the focused environment. On every non-editor Context the
        // EXIT family runs exactly as before (FFEDIT is not active), preserving
        // today's precedence over the POM Option_Key fastpath below.
        let is_equals_prefixed = cmd.trim_start().starts_with('=');
        let editor_env_active = {
            let t = self.tabs.active_tab();
            crate::shell::environment::active_environment(t.kind.tag(), t.is_home)
                == crate::shell::environment::EnvironmentKind::FfEdit
        };
        if (!editor_env_active || is_equals_prefixed) && self.try_exit_family(upper) {
            return true;
        }

        // Config-driven POM fastpath (menu-workspace Req 2.1e, 2.1i): a bare
        // option key (e.g. `1`, `S`) or `=<key>` (e.g. `=1`) is resolved against
        // the loaded pom.toml option list to the option's Option_Command, which
        // is then dispatched. There is NO behaviour keyed to the digit itself --
        // editing pom.toml's `command` is the only thing that changes what a key
        // does. Only the option's command re-enters `handle_command` below (which
        // runs its own prelude + ladder; a migrated verb there is handled by its
        // superseded ladder arm, the same observable result as `resolve_target`).
        if let Some(pom_command) = self.resolve_pom_option_key(upper) {
            // Guard against a self-referential loop (an option whose command is
            // its own key): only recurse when the resolved command differs.
            if pom_command.to_uppercase() != *upper {
                // CR-CH-052 (B080 reroute): route the resolved Option_Command
                // through the single front door `dispatch_command_string` (not
                // `handle_command`) so an in-scope verb reaches `resolve_target`
                // and the CustomWorkspace dispatch arm -- the same shell open
                // method as the (now-deleted) ladder arm. Recursion safety is
                // preserved because the Function terminal still re-enters via
                // `handle_command`, not the front door.
                self.dispatch_command_string(&pom_command);
                return true;
            }
        }

        // Chained fastpath navigation (menu-workspace Requirement 5): a
        // dotted/semicolon path like `=0.K`, `3.1`, or `=0;E.T` walks a menu
        // option chain. Each segment re-enters `handle_command`.
        if let Some(handled) = self.try_chained_fastpath(upper) {
            if handled {
                return true;
            }
        }

        false
    }

    /// The stage-2+ dispatch ladder: built-in intercepts, NAME, menu-name
    /// resolution, and the CommandEngine terminal. Runs AFTER the prelude and
    /// (on the front-door path) AFTER `resolve_target`. Migrated in-scope verbs
    /// (KEYS/KINDS/CONFIG/FILES/.../THEME-bare) still have their arms here as a
    /// superseded, now-unreachable fallback (B080 Step 2; see the arm comments).
    pub(super) fn run_command_ladder(&mut self, cmd: &str, upper: &str) {
        // CR-CH-053 E8: the ACTIVE COMMAND ENVIRONMENT gets FIRST CRACK, before
        // ANY FFCMD member. When an editor Context is focused the active
        // environment is FFEDIT, so its verbs are claimed here BEFORE
        // `try_commands_a` (which owns the FFCMD exit verbs X/RETURN/EXIT/QUIT/
        // LOGOFF) and before `try_commands_b1` / `resolve_target`. This is why bare
        // `X` on an editor Context is FFEDIT EXCLUDE (active-wins, Req 5.1), while
        // on a non-editor Context the gate is false and `try_commands_a` handles
        // `X` as a close/exit exactly as before (Req 4.1).
        //
        // NARROW `=` RULE (Req 3.2/3.2a): a `=`-prefixed command is NOT offered to
        // the environment -- `=` is the universal escape hatch that routes past the
        // active environment to FFCMD/POM. So `=X` in the editor is NOT claimed by
        // FFEDIT; it falls through to `try_commands_a` and reaches FFCMD's exit,
        // i.e. `=X` does return/exit (not EXCLUDE). (The FULL `=`-unification --
        // one universal `=` step owning drop-ladder-then-run for =X/=0/=0.K/=FILES
        // -- is deferred to E8b; here we only guarantee the environment skips
        // `=`-prefixed input, which is enough for the editor correction.)
        //
        // The gate lives in this SHARED ladder path (not the front door) so EVERY
        // `handle_command` caller reaches FFEDIT identically (the E1 reachability
        // correction); it is simply ordered ahead of `try_commands_a` now.
        if !cmd.trim_start().starts_with('=') {
            let (active_kind, active_is_home) = {
                let t = self.tabs.active_tab();
                (t.kind.tag(), t.is_home)
            };
            if crate::shell::environment::active_environment(active_kind, active_is_home)
                == crate::shell::environment::EnvironmentKind::FfEdit
                && self.ffedit_claim(cmd, upper)
            {
                return;
            }
        }

        // Stage 2 built-in intercepts (EXIT..RETURN). Extracted verbatim to
        // `commands_ladder_a.rs` (TASK 2.2, pure code movement); the FFCMD exit
        // verbs (X/RETURN/EXIT/QUIT/LOGOFF) live here and are reached AFTER the
        // active-environment claim above (CR-CH-053 E8).
        if self.try_commands_a(cmd, upper) {
            return;
        }

        // CR-CH-025: the hardcoded `SETTINGS`, `SETTINGS <ns>`, and `A`
        // intercepts are REMOVED. Opening the Settings menu is now keyword-less
        // menu-name resolution of the token `SETTINGS` (stage 3, see
        // `try_menu_name_dispatch`), identical to `POM` or any user menu; the
        // flat config-key browser and namespace filtering moved to the `CONFIG
        // [<namespace>]` command below (configuration-system Req 20).

        // Workspace-open / navigation / snapshot / capture built-ins
        // (CONFIG..RETRIEVE). Extracted verbatim to `commands_ladder_b.rs`
        // (TASK 2.2, pure code movement); dispatch order and behaviour unchanged.
        if self.try_commands_b1(cmd, upper) {
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
        //
        // THEME is NOT an FFEDIT verb -- it is an FFCMD command. It is the sole
        // arm that remained after the CR-CH-053 editor-verb migration emptied the
        // old `try_commands_b2` segment, so E7 folded it back inline here (the
        // `commands_ladder_b2.rs` file + its call were retired). The nav / exclude
        // / find / profile / scroll families it used to sit beside are now claimed
        // by `ffedit_claim` above when an editor Context is active.
        // CR-CH-052 (B080 Step 2 follow-up): the bare-`THEME` branch is DELETED.
        // Bare `THEME` is resolved via the front door's `resolve_target` ->
        // CustomWorkspace("theme_editor") (opening the Theme Editor Context)
        // before this ladder is reached, and the nav callers were rerouted to the
        // front door, so the bare branch is truly dead. Only `THEME <name>` (theme
        // APPLY) remains here -- `builtin_workspace_target` returns None for a
        // non-empty arg, so a named apply still falls through to this arm. A bare
        // `THEME` that somehow reaches here with no arg now simply falls through
        // (no editor open), which cannot happen on the live front-door path.
        if let Some(arg) = verb_arg(cmd, "THEME") {
            if arg.is_empty() {
                // Bare THEME is handled by the front door (CustomWorkspace); fall
                // through here rather than special-casing an editor open.
                return;
            }
            // CR-CH-056 (Req 24.5): THEME EXPORT / THEME IMPORT are the native
            // Export/Import commands. The Theme Editor UI affordances invoke
            // these same commands (apply_theme_editor_action routes Export/Import
            // here), so the typed and clicked paths share one code path.
            match arg.to_ascii_uppercase().as_str() {
                "EXPORT" => {
                    self.export_theme_command();
                    return;
                }
                "IMPORT" => {
                    self.import_theme_command();
                    return;
                }
                _ => {}
            }
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
        if let Some(handled) = self.try_chained_fastpath(upper) {
            if handled {
                return;
            }
        }

        // ── NAME -- Validates: CX Requirement 1.2, 1.3 ──────────────────────
        if let Some(name) = verb_arg(cmd, "NAME") {
            if name.is_empty() {
                // Clear workspace name.
                self.tabs.active_tab_mut().workspace_name = None;
            } else {
                // Set workspace name (max 32 chars).
                let name = if name.len() > 32 { &name[..32] } else { name };
                self.tabs.active_tab_mut().workspace_name = Some(name.to_string());
            }
            self.open_error = None;
            return;
        }

        // Detach / split / swap / dataset-stub / WORKSPACE built-ins
        // (DETACH..WORKSPACE). Extracted verbatim to `commands_ladder_c.rs`
        // (TASK 2.2, pure code movement); dispatch order and behaviour unchanged.
        if self.try_commands_c(cmd, upper) {
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
        // (The retrieve pointer was already reset for this non-RETRIEVE line by
        // `command_line_history.record` in the prelude, CR-NR-084.)
        let status = self.cmd_engine.execute_command_line(cmd);
        match status.kind {
            StatusKind::Info => {
                self.open_error = None;
            }
            StatusKind::SyntaxError | StatusKind::StructureError | StatusKind::RuntimeError => {
                self.open_error = Some(status.text.clone());
            }
        }
    }
}
use super::WorkbenchShell;
