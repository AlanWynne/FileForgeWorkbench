//! # Unified locale-aware verb resolver (CR-NR-109, localization Req 11.6).
//!
//! `resolve_locale_verb` applies the EXACT Req 11.6 resolution order to a typed
//! or selected verb token, returning the ROOT (canonical) command. It is called
//! at BOTH command-string seams -- the command-line front door
//! (`dispatch_command_string`) and the menu-selection entry
//! (`activate_menu_option`) -- so a localised surface and a bare English root
//! resolve the SAME way from either seam (single path, Req 11.6 last sentence).
//!
//! The resolver ONLY rewrites the verb TOKEN to its root; it never dispatches
//! and never touches the argument. The ROOT is what the unchanged prelude /
//! `resolve_target` / ladder then dispatch, and what history/session/macro
//! record (Req 11.9, 10.3). At `en` / no overlay the active-locale table is
//! identity-only, so the verb equals its root and behaviour is byte-identical
//! (Req 10.2).
//!
//! ## The Req 11.6 order
//!
//! Given the active environment L (FFCMD for a menu / non-editor Context, FFEDIT
//! for an editor Context) and a verb V:
//!
//! 1. Look up V (uppercased) in the active-locale AliasTable for L. The store's
//!    table already carries the active-locale overlay PLUS the synthesised
//!    identity rows for every root (FEAT-001), so a SINGLE `canonical_verb`
//!    lookup covers both branch 1.1 (a localised surface -> its root) and branch
//!    1.2.1 (a bare root resolving to itself via its identity row).
//!    -> `Root(canonical)`.
//! 2. Otherwise V is neither a localised surface nor a seeded root in L: return
//!    `Unknown(V)` (branch 1.2.2). The caller leaves V unchanged so the existing
//!    not-found path runs -- the resolver never turns a non-match into an error
//!    itself.
//!
//! This folds the "alias table for L" and the "root command table" of Req 11.6
//! into the one keyed AliasTable the store owns (its identity rows ARE the root
//! command table for L), so there is ONE lookup seam and no second dispatcher.

use super::WorkbenchShell;

/// The outcome of resolving a verb token through the active-locale alias layer.
///
/// Carries only the ROOT verb (or the unchanged surface on a miss); the caller
/// reassembles `root + arg` and dispatches through the UNCHANGED front door.
///
/// Validates: localization Requirement 11.6, 11.9
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LocaleResolveOutcome {
    /// V resolved (via a localised alias OR a seeded root identity row) to this
    /// ROOT command -- the dispatched/persisted identity.
    Root(String),
    /// V did not resolve in the active environment's table; the caller leaves it
    /// unchanged so the existing not-found path runs (Req 11.6 branch 1.2.2).
    Unknown(String),
}

impl WorkbenchShell {
    /// The stable NAME of the command environment active for the FOCUSED
    /// Context, derived through the kind exactly as the claim gate does
    /// (`EnvironmentRegistry::active_name`): FFEDIT for an editor Context, FFCMD
    /// for a menu / non-editor Context. This is the L of Req 11.6 -- the
    /// environment whose alias table the resolver reads.
    ///
    /// Validates: localization Requirement 11.6; command-environments Req 2.1
    pub(super) fn active_environment_name(&self) -> &'static str {
        let t = self.tabs.active_tab();
        self.environments.active_name(t.kind.tag(), t.is_home)
    }

    /// Resolve a verb token to its ROOT command under the active-locale alias
    /// layer for environment `env_name`, applying the Req 11.6 order (see the
    /// module docs). Pure wrt the shell (reads the alias store under its lock);
    /// it does NOT dispatch.
    ///
    /// Validates: localization Requirement 11.6, 11.9, 10.3
    pub(super) fn resolve_locale_verb(&self, verb: &str, env_name: &str) -> LocaleResolveOutcome {
        let trimmed = verb.trim();
        if trimmed.is_empty() {
            return LocaleResolveOutcome::Unknown(verb.to_string());
        }
        // Step 1 + 1.2.1: one lookup against the active-locale table for L, whose
        // overlay rows (localised surfaces) AND synthesised identity rows (roots)
        // both resolve through `canonical_verb` to the ROOT (FEAT-001).
        if let Ok(store) = self.i18n.alias_store.lock() {
            if let Some(table) = store.table_for(env_name) {
                if let Some(root) = table.canonical_verb(trimmed) {
                    return LocaleResolveOutcome::Root(root.to_string());
                }
            }
        }
        // Step 1.2.2: not a surface nor a seeded root in L -- leave it unchanged.
        LocaleResolveOutcome::Unknown(trimmed.to_string())
    }

    /// Rewrite a raw command STRING's verb token to its ROOT under the active
    /// environment's active-locale alias table (CR-NR-109 Req 11.6), preserving
    /// the argument and its case (B062). Returns the string with the verb
    /// replaced by its root when the resolver maps it to a DIFFERENT root; on an
    /// Unknown outcome, or when the root already equals the surface (en /
    /// identity), the ORIGINAL string is returned unchanged (allocating a copy),
    /// so the caller's front door is byte-identical at `en` (Req 10.2).
    ///
    /// This is the thin glue the command-line front door
    /// (`dispatch_command_string`) calls; the menu-selection entry calls
    /// `resolve_locale_verb` directly on the option's root (root in -> same root
    /// out) for the single-path property.
    ///
    /// Validates: localization Requirement 11.6, 11.9
    pub(super) fn rewrite_verb_to_root(&self, raw: &str) -> String {
        let (verb, arg) = super::helpers::split_verb_arg(raw);
        if verb.is_empty() {
            return raw.to_string();
        }
        let env_name = self.active_environment_name();
        match self.resolve_locale_verb(verb, env_name) {
            LocaleResolveOutcome::Root(root) if !root.eq_ignore_ascii_case(verb) => {
                if arg.is_empty() {
                    root
                } else {
                    format!("{root} {arg}")
                }
            }
            // Root == surface (en/identity) or Unknown: leave the string as typed.
            _ => raw.to_string(),
        }
    }

    /// Compute the per-option DISPLAYED command-column surfaces for the ACTIVE
    /// Menu Workspace (CR-NR-109 Task 11.5, localization Req 11.7, 11.8),
    /// positionally aligned with the loaded menu's options. For each option the
    /// surface is the active-locale alias surface for the option's ROOT command,
    /// looked up in the active environment's `AliasTable` via
    /// `display_surface_for_canonical`, falling back to the raw root command when
    /// the table has no row for it (Req 11.8). Returns an empty Vec when the
    /// active tab is not a loaded Menu Workspace, so the renderer falls back to
    /// the raw command for every row. At `en` / no overlay every surface equals
    /// its root (identity rows carry no key), so the column is byte-identical to
    /// the pre-CR-NR-109 render (Req 10.2).
    ///
    /// Reads only immutable shell state and returns an owned Vec, so the caller
    /// can compute it before taking the mutable tab borrow the renderer needs.
    ///
    /// Validates: localization Requirement 11.7, 11.8
    pub(super) fn menu_option_command_surfaces(&self) -> Vec<String> {
        let Some(menu) = self
            .tabs
            .active_tab()
            .kind
            .menu_workspace()
            .and_then(|mw| mw.menu.as_ref())
        else {
            return Vec::new();
        };
        let env_name = self.active_environment_name();
        let store = match self.i18n.alias_store.lock() {
            Ok(store) => store,
            Err(_) => return Vec::new(),
        };
        let table = store.table_for(env_name);
        menu.options
            .iter()
            .map(|option| {
                let source = table.and_then(|t| t.display_surface_for_canonical(&option.command));
                crate::menu_workspace::i18n_keys::localised_option_command(
                    option,
                    source.as_deref(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::environment_registry::{FFCMD_NAME, FFEDIT_NAME};
    use crate::shell::tests_common::make_shell;

    /// Overlay an `af` surface -> root mapping onto the given environment's table
    /// in the shell's alias store, so the resolver sees a localised surface.
    fn overlay_env(shell: &super::WorkbenchShell, env: &str, surface: &str, canonical: &str) {
        let mut store = shell.i18n.alias_store.lock().expect("alias store lock");
        store.overlay_row_for_test(env, surface, canonical);
    }

    /// Validates: localization Requirement 11.6 -- the resolver follows the exact
    /// Req 11.6 order: a localised surface (SOEK) resolves to its ROOT [1.1]; a
    /// bare root (FIND) resolves to itself via its identity row [1.2.1]; an
    /// unknown token (ZZZ) is Unknown [1.2.2].
    #[test]
    fn resolve_locale_verb_follows_the_req_11_6_order() {
        let shell = make_shell();
        // Seed an af overlay SOEK -> FIND on the FFEDIT table (FIND is a seeded
        // FFEDIT root via ffedit_english).
        overlay_env(&shell, FFEDIT_NAME, "SOEK", "FIND");

        // 1.1: a localised surface resolves to its root.
        assert_eq!(
            shell.resolve_locale_verb("soek", FFEDIT_NAME),
            LocaleResolveOutcome::Root("FIND".to_string()),
            "a localised surface must resolve to its ROOT (Req 11.6 branch 1.1)"
        );
        // 1.2.1: a bare root resolves to itself (identity row).
        assert_eq!(
            shell.resolve_locale_verb("FIND", FFEDIT_NAME),
            LocaleResolveOutcome::Root("FIND".to_string()),
            "a bare root must resolve to itself (Req 11.6 branch 1.2.1)"
        );
        // 1.2.2: an unknown token is Unknown and carries the surface unchanged.
        assert_eq!(
            shell.resolve_locale_verb("ZZZ", FFEDIT_NAME),
            LocaleResolveOutcome::Unknown("ZZZ".to_string()),
            "an unknown token must be Unknown (Req 11.6 branch 1.2.2)"
        );
    }

    /// Validates: localization Requirement 11.9, 10.3 -- the ROOT command is
    /// returned regardless of the surface typed: both the localised surface SOEK
    /// and the bare root FIND return Root(FIND), so the ROOT is the single
    /// dispatched/persisted identity.
    #[test]
    fn root_command_is_returned_regardless_of_surface() {
        let shell = make_shell();
        overlay_env(&shell, FFEDIT_NAME, "SOEK", "FIND");
        let via_surface = shell.resolve_locale_verb("SOEK", FFEDIT_NAME);
        let via_root = shell.resolve_locale_verb("FIND", FFEDIT_NAME);
        assert_eq!(via_surface, via_root);
        assert_eq!(via_surface, LocaleResolveOutcome::Root("FIND".to_string()));
    }

    /// Validates: localization Requirement 11.6, 11.3 -- EVERY legitimate FFCMD
    /// root resolves to itself via its synthesised identity row, NOT just the two
    /// built-in menus' option roots. A built-in workspace verb that is NOT a
    /// POM/Settings option root (COMMANDS, LOG, PLUGINS, MACROS, SEARCH, GSEARCH,
    /// =FILES, FILE CATALOGS) must still return Root(verb), so Req 11.6 branch
    /// 1.2.1 holds for every root a typed/selected verb can legitimately carry
    /// and that root can carry a localised surface in a menu.
    #[test]
    fn every_registered_ffcmd_root_resolves_to_itself() {
        let shell = make_shell();
        // These are legitimate FFCMD roots classified by
        // `command_config::builtin_workspace_target_for` but NOT present among the
        // two built-in menus' option `command` values, so the pre-fix
        // menu-only seed left them Unknown.
        for root in ["COMMANDS", "LOG", "PLUGINS", "MACROS", "SEARCH", "GSEARCH"] {
            assert_eq!(
                shell.resolve_locale_verb(root, FFCMD_NAME),
                LocaleResolveOutcome::Root(root.to_string()),
                "a legitimate FFCMD root ('{root}') must self-resolve (Req 11.6 branch 1.2.1)"
            );
        }
        // The two-word verb FILE CATALOGS and the =FILES fastpath are also
        // legitimate roots the seed must carry.
        assert_eq!(
            shell.resolve_locale_verb("FILE CATALOGS", FFCMD_NAME),
            LocaleResolveOutcome::Root("FILE CATALOGS".to_string())
        );
        assert_eq!(
            shell.resolve_locale_verb("=FILES", FFCMD_NAME),
            LocaleResolveOutcome::Root("=FILES".to_string())
        );
        // And the menu-option roots still resolve (regression guard).
        assert_eq!(
            shell.resolve_locale_verb("CATALOGS", FFCMD_NAME),
            LocaleResolveOutcome::Root("CATALOGS".to_string())
        );
    }

    /// Validates: localization Requirement 11.6 -- resolution is scoped to the
    /// ACTIVE environment L: an FFEDIT-only overlay does not leak into FFCMD, and
    /// an FFCMD seeded root (CATALOGS) resolves under FFCMD.
    #[test]
    fn resolution_is_scoped_to_the_active_environment() {
        let shell = make_shell();
        overlay_env(&shell, FFEDIT_NAME, "SOEK", "FIND");
        // SOEK is an FFEDIT surface; it must NOT resolve under FFCMD.
        assert_eq!(
            shell.resolve_locale_verb("SOEK", FFCMD_NAME),
            LocaleResolveOutcome::Unknown("SOEK".to_string())
        );
        // A built-in FFCMD root resolves to itself under FFCMD.
        assert_eq!(
            shell.resolve_locale_verb("CATALOGS", FFCMD_NAME),
            LocaleResolveOutcome::Root("CATALOGS".to_string())
        );
    }
}
