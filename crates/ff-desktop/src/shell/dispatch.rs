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
//! This file holds the ROUTER (`dispatch_command_string`). The FFEDIT claim is
//! now the registered `FfEditEnvironment::claim` object (CR-CH-053 Task 17, see
//! `environment_registry.rs`); the FFEDIT verb-body handlers (`ffedit_exclude`/
//! `_show`/`_reset`/`_caps`/`_nulls`/`_stats`/`_lock`/`_profile`/`_hilite` and
//! `find_status_to_error`) stay in the sibling `dispatch_ffedit.rs` as
//! `pub(super)` methods the object delegates to, keeping each file under the
//! 400-line rule (`rust-standards.md`).

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
