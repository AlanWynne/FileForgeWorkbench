//! # Unified Command Dispatch -- the single shell-side front door (B080).
//!
//! `dispatch_command_string` is the ONE shell-side entry point that every
//! command-string seam (the typed `Command ===>` line and the key/menu
//! FallThrough path) delegates to. It exists so Steps 1-7 of the dispatch-unify
//! migration (command-framework design.md "Unified Command Dispatch -- one front
//! door") can grow a single ordered Target_Resolution chain in ONE place instead
//! of the three seams converging by accident at `handle_command`.
//!
//! Scope reached by Step 2: `dispatch_command_string` is the LIVE single ordered
//! chain for the command-string seams (the typed `Command ===>` line and the
//! key/menu FallThrough). It runs, in order: the dispatch prelude
//! (`run_command_prelude`: history record, Req 8.3 stage-1 current-menu
//! Option_Key, the EXIT family, the POM / chained fastpaths), then
//! `ff_command::resolve_target` (whose `builtin_workspace_target` now classifies
//! the CustomWorkspace / navigation family -- Step 2), and finally the stage-2+
//! ladder (`run_command_ladder`). The prelude and the ladder are the SAME methods
//! `handle_command` runs, so there is ONE definition of each and no divergence.
//!
//! `resolve_target` lives HERE and NOT in `handle_command` because
//! `handle_command` is also the Function-execution terminal
//! (`dispatch_command_target` executes a resolved `Function` by calling
//! `handle_command(id)`); putting `resolve_target` in `handle_command` would make
//! a resolved registered Command_ID re-resolve to a `Function` and recurse
//! forever. Keeping `resolve_target` in the front door means every STRING seam
//! reaches it once, while a resolved Command_ID executes via `handle_command`
//! without re-resolution. Steps 3-7 (Function family, manager families,
//! split/detach, standalone, ladder retirement) remain TODO.
//!
//! This file holds the ROUTER (`dispatch_command_string` + the FFEDIT claim
//! `ffedit_claim`). The FFEDIT verb-body handlers (`ffedit_exclude`/`_show`/
//! `_reset`/`_caps`/`_nulls`/`_stats`/`_lock`/`_profile`/`_hilite` and
//! `find_status_to_error`) live in the sibling `dispatch_ffedit.rs`, split out to
//! keep both files under the 400-line rule (`rust-standards.md`).

use super::WorkbenchShell;

impl WorkbenchShell {
    /// The single shell-side front door for a command STRING: prelude ->
    /// `resolve_target` -> ladder.
    ///
    /// The typed line (`run_command_line`) and the key/menu FallThrough
    /// (`dispatch_bound_command`) both call this. It reproduces the ladder's
    /// EXISTING precedence exactly -- the prelude runs first (so e.g. `X` / `=X`
    /// from a non-menu context still exit via the EXIT family), THEN
    /// `resolve_target` classifies the in-scope CustomWorkspace / navigation
    /// family (Step 2), THEN the stage-2+ ladder handles everything else, exactly
    /// as today. `run_command_prelude` and `run_command_ladder` are shared with
    /// `handle_command`, so direct callers and this front door never diverge.
    ///
    /// Validates: command-framework Requirement 2.1, 8.3, 8.4
    pub(super) fn dispatch_command_string(&mut self, raw: &str) {
        // CR-CH-052 front door `=` step (applied ONCE, BEFORE the prelude,
        // resolve_target, and the Active_Environment): a `=`-prefixed command
        // reinitialises the active tab's Navigation_Stack to the POM (FFCMD_Root),
        // then runs the stripped remainder against FFCMD from the POM. This is the
        // SINGLE place `=` is interpreted -- the former three ad-hoc `=` sites
        // (the `=X` EXIT-family literal, the chained-fastpath origin pop, and the
        // `resolve_pom_option_key` strip-`=`) become consumers of an
        // already-stripped remainder. `=1` therefore resolves as POM option 1
        // from ANY tab, and `=X` falls out naturally (reinit to POM, then `X` at
        // the empty root closes the Workspace).
        if let Some(rest) = raw.trim_start().strip_prefix('=') {
            let rest = rest.trim().to_string();
            self.reinitialise_active_tab_to_pom();
            self.dispatch_command_string(&rest);
            return;
        }

        let upper = raw.trim().to_uppercase();

        // Prelude first (A1 ordering: stage 1 / EXIT family / POM / chained before
        // the active environment and resolve_target).
        if self.run_command_prelude(raw, &upper) {
            return;
        }

        // The single Target_Resolution stage (Req 8.3 stage 2-5). Built as
        // `resolve_and_dispatch_command` builds it so the typed path reaches the
        // SAME classification the key/menu path uses (one front door, Req 2.1).
        // `builtin_workspace_target` now owns the in-scope CustomWorkspace /
        // navigation family; everything else returns Err and falls through to the
        // ladder exactly as today (Req 8.4).
        let resolver = crate::command_config::ShellTargetResolver::new(
            &self.command_store.definitions,
            &self.cmd_registry,
            self.menus_dir(),
        );
        if let Ok(target) = ff_command::resolve_target(raw, &resolver) {
            self.dispatch_command_target(&target);
            return;
        }

        // Fall through to the stage-2+ ladder (not-yet-migrated verbs + the
        // editor engine terminal). The prelude already ran, so the ladder is
        // entered directly without re-running it.
        self.run_command_ladder(raw, &upper);
    }

    /// The FFEDIT (editor) Command_Environment claim step (CR-CH-053).
    ///
    /// Called from the front door's active-env step ONLY when the focused Context
    /// is an editor Context. Returns `true` iff FFEDIT owns `raw` and handled it
    /// (the caller then returns); `false` to fall through to FFCMD
    /// (`resolve_target`) and then the ladder.
    ///
    /// E1 migrates the NAVIGATION family (LOCATE/TOP/BOTTOM/UP/DOWN/LEFT/RIGHT/
    /// SORT) into FFEDIT, delegating to `nav_manager` with IDENTICAL observable
    /// results to the former ladder arms (Req 6.2, 6.3). The surface verb is
    /// resolved to its CANONICAL verb via the FFEDIT alias table (Req 6a) before
    /// matching; the nav verbs are identity-mapped (no aliases), so canonical ==
    /// verb, but routing through the table now establishes the pattern E3 (X ->
    /// EXCLUDE) relies on. Remaining families (exclude/show, find, profile,
    /// scroll) still fall through to their ladder arms until E2-E5 migrate them.
    /// FFEDIT claims BEFORE FFCMD and the Command_Engine terminal (Req 6.2a).
    ///
    /// Validates: command-environments Requirement 6.1, 6.2, 6.2a, 6.3, 6a.1
    pub(super) fn ffedit_claim(&mut self, raw: &str, upper: &str) -> bool {
        use super::environment::AliasTable;
        use super::helpers::{parse_optional_u64, verb_arg};

        // Resolve the surface verb token to its canonical verb (Req 6a). The
        // handler below matches on the CANONICAL verb; the alias table is the
        // single point localized surface forms (CR-NR-103) will plug into.
        let aliases = AliasTable::ffedit_english();
        let verb_token = raw.split_whitespace().next().unwrap_or("");
        let Some(canonical) = aliases.canonical_verb(verb_token) else {
            // Not an FFEDIT verb -- fall through to FFCMD / the ladder.
            return false;
        };

        // NAVIGATION family (E1). Each arm is the former `try_commands_b2` arm,
        // moved verbatim, keyed on the canonical verb. Non-nav canonical verbs
        // (exclude/find/profile/scroll) are NOT handled yet -> fall through.
        match canonical {
            "LOCATE" => {
                let arg = verb_arg(raw, "LOCATE").filter(|a| !a.is_empty());
                let Some(arg) = arg else { return false };
                let status = self.nav_manager.locate(arg, &mut self.tabs);
                self.open_error = if status.is_empty() {
                    None
                } else {
                    Some(status)
                };
                true
            }
            "TOP" => {
                self.nav_manager.top(&mut self.tabs);
                self.open_error = None;
                true
            }
            "BOTTOM" => {
                self.nav_manager.bottom(&mut self.tabs);
                self.open_error = None;
                true
            }
            "UP" => {
                // CR-NR-087: numeric `UP n` overrides SCROLL; bare `UP` uses the
                // active SCROLL amount.
                let Some(arg) = verb_arg(raw, "UP") else {
                    return false;
                };
                match parse_optional_u64(arg) {
                    Some(n) => self.nav_manager.up(Some(n), &mut self.tabs),
                    None => self
                        .nav_manager
                        .up_by_amount(&self.scroll_amount, &mut self.tabs),
                }
                self.open_error = None;
                true
            }
            "DOWN" => {
                let Some(arg) = verb_arg(raw, "DOWN") else {
                    return false;
                };
                match parse_optional_u64(arg) {
                    Some(n) => self.nav_manager.down(Some(n), &mut self.tabs),
                    None => self
                        .nav_manager
                        .down_by_amount(&self.scroll_amount, &mut self.tabs),
                }
                self.open_error = None;
                true
            }
            "LEFT" => {
                let Some(arg) = verb_arg(raw, "LEFT") else {
                    return false;
                };
                self.nav_manager
                    .left(parse_optional_u64(arg), &mut self.tabs);
                self.open_error = None;
                true
            }
            "RIGHT" => {
                let Some(arg) = verb_arg(raw, "RIGHT") else {
                    return false;
                };
                self.nav_manager
                    .right(parse_optional_u64(arg), &mut self.tabs);
                self.open_error = None;
                true
            }
            "SORT" => {
                let Some(rest) = verb_arg(raw, "SORT") else {
                    return false;
                };
                let args: Vec<&str> = rest.split_whitespace().collect();
                let status = self.nav_manager.sort(&args, &mut self.tabs, &self.runtime);
                self.open_error = if status.is_empty() {
                    None
                } else {
                    Some(status)
                };
                true
            }

            // EXCLUDE / SHOW / RESET family (E2). X -> EXCLUDE and INCLUDE -> SHOW
            // are resolved by the alias table, so the match keys on the CANONICAL
            // verb; the sub-parse re-derives the argument text from `raw` (which
            // carries the surface form the user typed) to preserve EXACT behaviour
            // incl. the `ALL` suffix.
            "EXCLUDE" => {
                self.ffedit_exclude(raw);
                true
            }
            "SHOW" => {
                self.ffedit_show(raw);
                true
            }
            "RESET" => {
                // Ownership boundary (CR-CH-053): FFEDIT's RESET owns the editor
                // exclusion-reset forms (bare RESET, RESET ALL, RESET EXCLUDED).
                // `RESET BARE [..]` is a DIFFERENT, FFCMD-owned command (reset a
                // profile to the compiled baseline, configuration-system Req 19),
                // not an editor verb -- FFEDIT DECLINES it so it falls through to
                // FFCMD's `RESET BARE` ladder arm. Without this, FFEDIT would
                // shadow `RESET BARE` on an editor Context now that the env claim
                // runs ahead of the FFCMD arms (E8).
                let rest = verb_arg(raw, "RESET").unwrap_or("");
                let first_arg = rest.split_whitespace().next().unwrap_or("");
                if first_arg.eq_ignore_ascii_case("BARE") {
                    return false;
                }
                self.ffedit_reset(raw);
                true
            }

            // FIND / RFIND / CHANGE / RCHANGE family (E3). CHANGE's two-argument
            // quoting (`parse_two_args`) and the B062 case-preserved term are
            // carried verbatim from the former ladder arms.
            "RFIND" => {
                let status = self.find_manager.rfind(&mut self.tabs, &self.runtime);
                self.open_error = Self::find_status_to_error(status);
                true
            }
            "RCHANGE" => {
                let status = self.find_manager.rchange(&mut self.tabs, &self.runtime);
                self.open_error = Self::find_status_to_error(status);
                true
            }
            "FIND" => {
                let Some(term) = verb_arg(raw, "FIND").filter(|a| !a.is_empty()) else {
                    return false;
                };
                let status = self.find_manager.find(term, &mut self.tabs, &self.runtime);
                self.open_error = Self::find_status_to_error(status);
                true
            }
            "CHANGE" => {
                let Some(rest) = verb_arg(raw, "CHANGE").filter(|a| !a.is_empty()) else {
                    return false;
                };
                if let Some((old, new)) = super::helpers::parse_two_args(rest) {
                    let status =
                        self.find_manager
                            .change(&old, &new, &mut self.tabs, &self.runtime);
                    self.open_error = Self::find_status_to_error(status);
                } else {
                    self.open_error =
                        Some("CHANGE requires two arguments: CHANGE 'old' 'new'".to_string());
                }
                true
            }

            // PROFILE family (E4): CAPS / NULLS / STATS / LOCK / PROFILE / HILITE.
            "CAPS" => {
                self.ffedit_caps(upper);
                true
            }
            "NULLS" => {
                self.ffedit_nulls(upper);
                true
            }
            "STATS" => {
                self.ffedit_stats(upper);
                true
            }
            "LOCK" => {
                self.ffedit_lock(upper);
                true
            }
            "PROFILE" => {
                self.ffedit_profile(raw);
                true
            }
            "HILITE" => {
                self.ffedit_hilite(raw);
                true
            }

            // SCROLL field update (E5).
            "SCROLL" => {
                let Some(arg) = verb_arg(raw, "SCROLL").filter(|a| !a.is_empty()) else {
                    return false;
                };
                if let Some(amount) = crate::scroll_amount::ScrollAmount::parse(arg) {
                    self.scroll_amount = amount;
                    self.scroll_field_text = self.scroll_amount.display_string();
                    self.open_error = None;
                } else {
                    self.open_error = Some(format!(
                        "SCROLL: '{arg}' is not a valid scroll amount \
                         (PAGE/HALF/CSR/MAX/DATA/n)"
                    ));
                }
                true
            }

            // Editor-buffer SAVE (E9, Req 10.1): dirty-aware, STAYS in the editor.
            "SAVE" => {
                self.ffedit_save();
                true
            }

            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::shell::helpers::split_verb_arg;

    /// Validates: command-framework Requirement 9.7 -- the Argument_String keeps
    /// its original case (B062); only the verb token is case-insensitive.
    #[test]
    fn verb_arg_split_preserves_argument_case() {
        let (verb, arg) = split_verb_arg("LOCATE Foo");
        assert!(
            verb.eq_ignore_ascii_case("LOCATE"),
            "verb token is matched case-insensitively"
        );
        assert_eq!(arg, "Foo", "argument case is preserved (B062)");
    }
}
