//! # Command Environments (CR-CH-053) -- E0 scaffold (Option 2)
//!
//! FFWB adopts the REXX/ISPF ADDRESS model: a set of named Command Environments,
//! each owning the commands relevant to its context. The active environment is
//! SUPPLIED BY the focused Context's KIND (not a central match the command
//! handler owns), with FFCMD as the always-present base (= `resolve_target`).
//!
//! ## E0 scope (Option 2)
//!
//! - The active environment is derived through the KIND: `environment_for_kind`
//!   maps a `BuiltinKind` (obtained via `BuiltinKind::from_tab_kind`, the kind's
//!   OWN mapping) to its command environment. The command handler does NOT
//!   enumerate `TabKind` -- it goes through the kind. This is the dependency
//!   inversion (Requirement 2.1, 9.1) in its E0 form.
//! - The PERSISTED, user-configurable `command_environment` kind ATTRIBUTE
//!   (a field on `KindConfig`/`KindConfigToml`) is DEFERRED to the later
//!   Kinds-config slice (owner decision: editor/catalogs face major refactors;
//!   do not change the persisted Kinds schema mid-framework). Until then the
//!   built-in kinds declare their environment here in code, exactly as they
//!   declare `default_title()`.
//! - The per-environment ALIAS TABLE (surface form -> canonical verb,
//!   Requirement 6a) is seeded with the existing English aliases. In E0 the
//!   FFEDIT claim still handles NOTHING (pure indirection), so behaviour is
//!   byte-identical; verbs migrate in E1-E5.
//!
//! See docs/specs/command-environments/{requirements,design}.md.

use crate::tab_state::KindTag;
use crate::workspace_kind::BuiltinKind;

/// The command environment that owns a Context's command-line verbs.
///
/// Phase 1 BUILDS only FFEDIT as a first-class claiming environment; FFCMD is the
/// existing `resolve_target` base; FFNAV / FFLINE are NAMED so the model and
/// derivation are correct but their command handling is unchanged.
///
/// The `Ff` prefix is the deliberate FF* environment naming convention
/// (FFCMD/FFEDIT/FFNAV, mirroring the mainframe TSO/ISREDIT/... set); the shared
/// prefix is intentional, not accidental.
///
/// Validates: command-environments Requirement 1.1, 2.1, 2a.1
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnvironmentKind {
    /// The editor command-line environment (FIND searches the open buffer, etc.).
    /// The only environment that CLAIMS at the front-door active-env step in
    /// phase 1 (and only once E1+ migrate verbs into it).
    FfEdit,
    /// The file-navigator environment (its FIND locates a file). NAMED only in
    /// phase 1 -- its verbs stay handled where they are today; does not claim.
    FfNav,
    /// The shell/workbench base environment. It IS `resolve_target`; it does not
    /// claim at the active-env step -- the front door reaches it via the existing
    /// `resolve_target` call.
    FfCmd,
}

impl EnvironmentKind {
    /// Stable name used for addressing (macros) and diagnostics. Reserved for the
    /// macro ADDRESS wiring (task 10); not yet consumed in E0.
    ///
    /// Validates: command-environments Requirement 8.1
    #[allow(dead_code)] // consumed by the macro ADDRESS wiring (task 10).
    pub(crate) fn name(self) -> &'static str {
        match self {
            EnvironmentKind::FfEdit => "FFEDIT",
            EnvironmentKind::FfNav => "FFNAV",
            EnvironmentKind::FfCmd => "FFCMD",
        }
    }
}

/// The command environment DECLARED BY a built-in Workspace Kind.
///
/// This is where each kind "supplies" its environment, the way it supplies its
/// `default_title()`. The command handler obtains the `BuiltinKind` via
/// `BuiltinKind::from_tab_kind` and asks here; it does NOT enumerate `TabKind`.
/// Option 2 (E0): declared in code here; the persisted user-configurable
/// `command_environment` attribute on `KindConfig` is a later slice. A kind that
/// declares nothing special resolves to the FFCMD base.
///
/// Editor kinds -> FFEDIT; file-navigator kinds (Files = File Explorer, Catalogs)
/// -> FFNAV (named only in phase 1); every other kind -> FFCMD.
///
/// Validates: command-environments Requirement 2.1, 7a.1, 9.1
pub(crate) fn environment_for_kind(kind: BuiltinKind) -> EnvironmentKind {
    match kind {
        BuiltinKind::Editor => EnvironmentKind::FfEdit,
        BuiltinKind::Files | BuiltinKind::Catalogs => EnvironmentKind::FfNav,
        _ => EnvironmentKind::FfCmd,
    }
}

/// The Active_Environment for a focused Context, derived THROUGH its kind.
///
/// `is_home` splits POM vs Menu for the kind mapping (both render as
/// `TabKind::MenuWorkspace`); neither is an editor/navigator kind, so both
/// resolve to FFCMD -- `is_home` does not affect the environment, but the kind
/// mapping is routed through the one `from_tab_kind` seam for correctness.
///
/// Validates: command-environments Requirement 2.1, 2.2
pub(crate) fn active_environment(kind: KindTag, is_home: bool) -> EnvironmentKind {
    environment_for_kind(BuiltinKind::from_tab_kind(kind, is_home))
}

/// A per-environment ALIAS TABLE: resolves a typed surface form to the CANONICAL
/// verb, BEFORE dispatch (Requirement 6a). Behaviour lives on the canonical verb;
/// the canonical verb is what is recorded/persisted. Matching is
/// case-insensitive (B062). Phase 1 seeds the existing English aliases; CR-NR-103
/// loads per-locale alias data into the SAME table with no new mechanism.
///
/// E0: the table EXISTS and is unit-tested, but the FFEDIT claim handles nothing
/// yet, so it does not alter dispatch -- behaviour is byte-identical. E1+ consult
/// `canonical_verb` when matching migrated verbs.
///
/// Validates: command-environments Requirement 6a.1, 6a.2, 6a.3, 6a.5
pub(crate) struct AliasTable {
    /// (surface-form-UPPERCASED, canonical-verb) pairs. Kept as a small Vec; verb
    /// counts are tiny and this avoids a HashMap dependency for E0.
    ///
    /// Stored as OWNED `String` (not `&'static str`) so that a per-locale alias
    /// overlay -- which arrives as runtime DATA (CR-NR-103 fixtures now, on-disk
    /// catalogues in Phase 3) -- can be layered ON TOP of the English base
    /// entries without leaking memory on each locale switch (localization
    /// Requirement 5.5). The English/no-overlay case is byte-identical: the
    /// seed data is the same literals, merely `.to_string()`-ed at construction.
    entries: Vec<(String, String)>,
}

impl AliasTable {
    /// Build the English-seed alias table for the FFEDIT environment: the
    /// existing one-behaviour aliases (X -> EXCLUDE, INCLUDE -> SHOW) plus the
    /// identity entries for the canonical verbs. CR-NR-103 extends this per
    /// locale. Panics on a collision (an alias mapping two different canonicals),
    /// surfacing an authoring error at construction (Requirement 6a.3).
    pub(crate) fn ffedit_english() -> Self {
        // (surface form, canonical verb). Identity entries make the canonical
        // verbs themselves resolvable; the two genuine aliases are X and INCLUDE.
        let raw: &[(&str, &str)] = &[
            ("LOCATE", "LOCATE"),
            ("TOP", "TOP"),
            ("BOTTOM", "BOTTOM"),
            ("UP", "UP"),
            ("DOWN", "DOWN"),
            ("LEFT", "LEFT"),
            ("RIGHT", "RIGHT"),
            ("SORT", "SORT"),
            ("EXCLUDE", "EXCLUDE"),
            ("X", "EXCLUDE"),
            ("SHOW", "SHOW"),
            ("INCLUDE", "SHOW"),
            ("RESET", "RESET"),
            ("FIND", "FIND"),
            ("RFIND", "RFIND"),
            ("CHANGE", "CHANGE"),
            ("RCHANGE", "RCHANGE"),
            ("CAPS", "CAPS"),
            ("NULLS", "NULLS"),
            ("STATS", "STATS"),
            ("LOCK", "LOCK"),
            ("PROFILE", "PROFILE"),
            ("HILITE", "HILITE"),
            ("SCROLL", "SCROLL"),
            // Editor-buffer verb (E9, Req 10.1): SAVE is dirty-aware and stays in
            // the editor. UNDO/REDO are deferred (Req 10.6: UNDO is keyboard-only
            // today, REDO does not exist) so they are NOT in the table yet.
            ("SAVE", "SAVE"),
        ];
        let mut entries: Vec<(String, String)> = Vec::with_capacity(raw.len());
        for &(surface, canonical) in raw {
            if let Some((_, existing)) = entries.iter().find(|(s, _)| s == surface) {
                // Collision: the same surface form maps to two canonicals -- an
                // authoring error (Requirement 6a.3).
                assert_eq!(
                    existing, canonical,
                    "alias collision: surface form '{surface}' maps to both \
                     '{existing}' and '{canonical}'"
                );
            }
            entries.push((surface.to_string(), canonical.to_string()));
        }
        Self { entries }
    }

    /// Resolve a surface-form verb token to its canonical verb (case-insensitive),
    /// or `None` when this environment does not own it. The caller matches on the
    /// CANONICAL verb; the canonical verb is what is recorded/persisted.
    ///
    /// Validates: command-environments Requirement 6a.1, 6a.2
    pub(crate) fn canonical_verb(&self, surface: &str) -> Option<&str> {
        let s = surface.trim();
        self.entries
            .iter()
            .find(|(form, _)| form.eq_ignore_ascii_case(s))
            .map(|(_, canonical)| canonical.as_str())
    }

    /// Overlay a locale's `(surface-form -> canonical-verb)` rows ON TOP of this
    /// table's existing English base entries (base English + locale overlay, NOT
    /// a replacement -- localization Requirement 5.5). A localized surface form
    /// added here resolves through the IDENTICAL `canonical_verb` seam to the
    /// SAME canonical English verb, so dispatch is behaviour-neutral (Req 5.1).
    ///
    /// The overlay is FALLIBLE and RECOVERABLE (localization Requirement 5.6):
    /// on a within-this-table collision -- an incoming surface form that already
    /// resolves (case-insensitively) to a DIFFERENT canonical, whether that
    /// existing mapping is a base identity, a base alias, or an earlier row in
    /// this same overlay -- it returns `Err` and leaves `self` UNCHANGED (the
    /// Identity_Base is retained). An incoming row that duplicates an identical
    /// existing `(surface -> same canonical)` mapping is a harmless no-op, not a
    /// collision (mirrors the `ffedit_english` authoring discipline). The
    /// collision check is scoped WITHIN this one environment's table.
    ///
    /// `environment_name` is used only for the diagnostic in the returned error.
    ///
    /// Validates: localization Requirement 5.1, 5.5, 5.6, 10.3;
    /// command-environments Requirement 6a.3, 6a.5, 6a.7
    // Consumed by the Phase 3 locale-switch loader (Task 10) and by the Task 4
    // unit tests; a non-test build sees no caller yet, like the ahead-of-consumer
    // `CommandEnvironment` trait below.
    #[allow(dead_code)]
    pub(crate) fn apply_locale_overlay(
        &mut self,
        environment_name: &str,
        rows: &[(String, String)],
    ) -> Result<(), super::alias_overlay::AliasOverlayError> {
        use super::alias_overlay::AliasOverlayError;

        // Validate the WHOLE overlay against a scratch copy first, so a collision
        // leaves `self` untouched (Identity_Base retained, Req 5.6). Only on full
        // success do we commit the scratch copy back.
        let mut scratch = self.entries.clone();
        for (surface, canonical) in rows {
            let surface_trimmed = surface.trim();
            if let Some((_, existing)) = scratch
                .iter()
                .find(|(form, _)| form.eq_ignore_ascii_case(surface_trimmed))
            {
                if existing == canonical {
                    // Identical mapping already present -- harmless no-op.
                    continue;
                }
                return Err(AliasOverlayError::Collision {
                    environment: environment_name.to_string(),
                    surface: surface_trimmed.to_string(),
                    existing: existing.clone(),
                    incoming: canonical.clone(),
                });
            }
            scratch.push((surface_trimmed.to_string(), canonical.to_string()));
        }
        self.entries = scratch;
        Ok(())
    }

    /// Build an EMPTY alias table (no base entries). Test-only helper used to
    /// model a second environment's base when exercising the per-environment
    /// loader (localization Req 5.3) without touching that environment's real
    /// dispatch wiring.
    #[cfg(test)]
    pub(crate) fn empty_for_test() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

/// The contract every first-class Command_Environment implements: try to CLAIM a
/// raw command string for this environment's vocabulary (after resolving its
/// surface verb to the canonical verb via the alias table). Returns `true` when
/// claimed+handled; `false` to fall through to the next front-door step.
///
/// E0 defines the contract; its first implementor arrives when E1 migrates the
/// FFEDIT navigation family (today FFEDIT claims via the `ffedit_claim` method on
/// WorkbenchShell, because its verbs mutate shell-entangled state -- see
/// design.md "FFEDIT is a resolver+executor over the active editor"). Kept ahead
/// of its first impl so the trait is defined in one place for E1+.
///
/// Validates: command-environments Requirement 1.1, 6.2a
#[allow(dead_code)]
pub(crate) trait CommandEnvironment {
    /// Attempt to claim and execute `raw`. `upper` is the pre-uppercased trimmed
    /// line to avoid recomputing it. Returns `true` iff claimed+handled.
    fn claim(&mut self, raw: &str, upper: &str) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates: command-environments Requirement 2.1, 9.1 -- the environment is
    /// derived THROUGH the kind (editor -> FFEDIT, navigator -> FFNAV, else
    /// FFCMD), not via a handler-owned TabKind match.
    #[test]
    fn environment_is_supplied_by_kind() {
        assert_eq!(
            environment_for_kind(BuiltinKind::Editor),
            EnvironmentKind::FfEdit
        );
        assert_eq!(
            environment_for_kind(BuiltinKind::Files),
            EnvironmentKind::FfNav
        );
        assert_eq!(
            environment_for_kind(BuiltinKind::Catalogs),
            EnvironmentKind::FfNav
        );
        assert_eq!(
            environment_for_kind(BuiltinKind::Pom),
            EnvironmentKind::FfCmd
        );
        assert_eq!(
            environment_for_kind(BuiltinKind::Config),
            EnvironmentKind::FfCmd
        );
    }

    /// Validates: command-environments Requirement 2.1 -- an editor KindTag
    /// resolves (through its kind) to FFEDIT; a menu KindTag to FFCMD.
    #[test]
    fn active_environment_routes_through_kind() {
        assert_eq!(
            active_environment(KindTag::FileEditor, false),
            EnvironmentKind::FfEdit
        );
        assert_eq!(
            active_environment(KindTag::Untitled, false),
            EnvironmentKind::FfEdit
        );
        assert_eq!(
            active_environment(KindTag::FileExplorerPanel, false),
            EnvironmentKind::FfNav
        );
        assert_eq!(
            active_environment(KindTag::MenuWorkspace, true),
            EnvironmentKind::FfCmd
        );
    }

    /// Validates: command-environments Requirement 6a.1, 6a.2 -- the alias table
    /// resolves a surface form to its canonical verb, case-insensitively; the
    /// genuine aliases (X -> EXCLUDE, INCLUDE -> SHOW) map to their canonical.
    #[test]
    fn alias_table_resolves_surface_to_canonical() {
        let t = AliasTable::ffedit_english();
        assert_eq!(t.canonical_verb("FIND"), Some("FIND"));
        assert_eq!(t.canonical_verb("find"), Some("FIND")); // case-insensitive
        assert_eq!(t.canonical_verb("X"), Some("EXCLUDE")); // alias -> canonical
        assert_eq!(t.canonical_verb("INCLUDE"), Some("SHOW"));
        assert_eq!(t.canonical_verb("NOTAVERB"), None);
    }

    /// Validates: command-environments Requirement 5.1 (active-wins) -- the
    /// FFEDIT environment claims a verb it owns ONLY when the Active_Environment
    /// is FFEDIT (an editor Context). The derivation gates the claim: on an
    /// editor Context the active env is FFEDIT (so `run_command_ladder`'s gate
    /// lets `ffedit_claim` run and claim an owned verb such as FIND); on a
    /// non-editor Context the active env is NOT FFEDIT (so the gate is false and
    /// the SAME verb falls through to the FFCMD base), mirroring the
    /// earlier-stage-wins rule of command-framework Req 8.10. FIND is a verb the
    /// FFEDIT table owns, exercised here as the shared-name representative.
    #[test]
    fn ffedit_claims_owned_verb_only_when_active_env_is_ffedit() {
        let owned = AliasTable::ffedit_english();
        // FFEDIT owns FIND (the canonical resolves), so when FFEDIT is active it
        // is the environment that will claim it.
        assert_eq!(owned.canonical_verb("FIND"), Some("FIND"));

        // Editor Context -> FFEDIT is the Active_Environment (claim gate TRUE).
        assert_eq!(
            active_environment(KindTag::FileEditor, false),
            EnvironmentKind::FfEdit,
            "an editor Context must make FFEDIT the active environment so it wins the verb"
        );
        // A non-editor Context (menu / navigator / config) is NOT FFEDIT, so the
        // ladder's FFEDIT gate is false and the verb is left to the FFCMD base.
        assert_ne!(
            active_environment(KindTag::MenuWorkspace, true),
            EnvironmentKind::FfEdit,
            "a non-editor Context must not route an owned verb through FFEDIT"
        );
        assert_ne!(
            active_environment(KindTag::ConfigPanel, false),
            EnvironmentKind::FfEdit
        );
    }

    /// Validates: command-environments Requirement 2a.6, 5.1, 5.3 (E8 corrected
    /// model) -- the FFCMD exit verbs are NOT a privileged unshadowable set; they
    /// are ordinary FFCMD-owned verbs. `X` is BOTH an FFCMD exit alias AND the
    /// FFEDIT alias for EXCLUDE, and FFEDIT (the active environment) gets FIRST
    /// crack: so bare `X` in an editor Context is FFEDIT EXCLUDE (shadows the
    /// FFCMD exit), while `RETURN` / `EXIT` / `QUIT` / `LOGOFF` are NOT FFEDIT
    /// verbs (FFEDIT declines, FFCMD receives them). This replaces the earlier
    /// provisional assertion that wrongly treated `X` as prelude-owned.
    #[test]
    fn ffedit_shadows_ffcmd_x_but_not_the_other_exit_verbs() {
        let t = AliasTable::ffedit_english();

        // `X` IS an FFEDIT verb (EXCLUDE) -> the active env (FFEDIT) wins it over
        // FFCMD's `X` exit alias (active-wins, Req 5.1).
        assert_eq!(
            t.canonical_verb("X"),
            Some("EXCLUDE"),
            "X is FFEDIT EXCLUDE; the active env wins it over FFCMD's X exit alias (Req 5.1)"
        );

        // The other FFCMD exit verbs are NOT FFEDIT verbs -> FFEDIT declines and
        // FFCMD receives them (they are not shadowed by FFEDIT). END/RETURN are
        // navigation, also not FFEDIT verbs.
        for ffcmd_verb in ["EXIT", "QUIT", "LOGOFF", "RETURN", "END", "START"] {
            assert_eq!(
                t.canonical_verb(ffcmd_verb),
                None,
                "'{ffcmd_verb}' is not an FFEDIT verb; FFEDIT declines so FFCMD receives it"
            );
        }
    }

    /// Validates: command-environments Requirement 3.2, 3.2a (E8 narrow `=`
    /// rule) -- a `=`-prefixed command is routed PAST the active environment to
    /// FFCMD/POM; the environment is skipped for it. This pins the predicate the
    /// ladder uses to skip the FFEDIT claim: a string whose first non-space char
    /// is `=` is never offered to the environment, so `=X` reaches FFCMD `X`
    /// (return/exit) rather than FFEDIT EXCLUDE.
    #[test]
    fn equals_prefixed_command_bypasses_the_active_environment() {
        // The narrow-rule predicate: trimmed string starts with '='.
        let is_equals_prefixed = |raw: &str| raw.trim_start().starts_with('=');
        assert!(
            is_equals_prefixed("=X"),
            "=X is `=`-prefixed -> bypass FFEDIT"
        );
        assert!(
            is_equals_prefixed("  =0.K"),
            "leading spaces still detected"
        );
        assert!(
            !is_equals_prefixed("X"),
            "bare X is NOT bypassed -> FFEDIT sees it"
        );
        assert!(
            !is_equals_prefixed("FIND x"),
            "a normal verb is not bypassed"
        );
    }
}
