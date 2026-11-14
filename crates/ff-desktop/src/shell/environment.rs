//! # Command Environments (CR-CH-053) -- E0 scaffold (Option 2)
//!
//! FFWB adopts the REXX/ISPF ADDRESS model: a set of named Command Environments,
//! each owning the commands relevant to its context, with FFCMD as the
//! always-present base (= `resolve_target`). The active environment is derived
//! through the focused Context's KIND via the built `EnvironmentRegistry`
//! (`environment_registry.rs`, command-environments Req 13.1-13.4).
//!
//! This file holds the per-environment `AliasTable` (surface form -> canonical
//! verb, Requirement 6a). The per-row DATA types `AliasEntry` / `AliasOrigin`
//! live in `alias_entry.rs` (CR-NR-121 Stage B 400-line split, re-exported
//! below); the shared contract (`CommandEnvironment` + `EnvDispatchOutcome`)
//! lives in `environment_contract.rs`, re-exported here (CR-NR-109 400-line
//! split).
//!
//! See docs/specs/command-environments/{requirements,design}.md.

// CR-NR-121 Stage B: the per-row alias DATA types (`AliasOrigin` + `AliasEntry`)
// were split into `alias_entry.rs` to keep this file under the 400-line limit.
// Re-exported here so every existing `super::environment::{AliasEntry,
// AliasOrigin}` path is unchanged.
pub(crate) use super::alias_entry::{AliasEntry, AliasOrigin};

// CR-NR-121 Stage B: the `AliasTable` locale/user overlay-application methods
// live in the child module `environment/alias_table_overlay.rs` (same 400-line
// split). As a child module it reaches `AliasTable`'s private `entries` field.
mod alias_table_overlay;

/// A per-environment ALIAS TABLE: resolves a typed surface form to the CANONICAL
/// verb, BEFORE dispatch (Requirement 6a). Behaviour lives on the canonical verb;
/// the canonical verb is what is recorded/persisted. Matching is
/// case-insensitive (B062). Phase 1 seeds the existing English aliases; CR-NR-103
/// loads per-locale alias data into the SAME table with no new mechanism;
/// CR-NR-109 grows each entry to carry an optional localisation key.
///
/// E0: the table EXISTS and is unit-tested, but the FFEDIT claim handles nothing
/// yet, so it does not alter dispatch -- behaviour is byte-identical. E1+ consult
/// `canonical_verb` when matching migrated verbs.
///
/// Validates: command-environments Requirement 6a.1, 6a.2, 6a.3, 6a.5;
/// localization Requirement 11.1
#[derive(Debug, Clone)]
pub(crate) struct AliasTable {
    /// `AliasEntry` rows. Kept as a small Vec; verb counts are tiny and this
    /// avoids a HashMap dependency for E0.
    ///
    /// Stored as OWNED `String` (not `&'static str`) so that a per-locale alias
    /// overlay -- which arrives as runtime DATA (CR-NR-103 fixtures now, on-disk
    /// catalogues in Phase 3) -- can be layered ON TOP of the English base
    /// entries without leaking memory on each locale switch (localization
    /// Requirement 5.5). The English/no-overlay case is byte-identical: the
    /// seed data is the same literals, merely `.to_string()`-ed at construction.
    entries: Vec<AliasEntry>,
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
            // CR-CH-069 (Req 21.1): the in-buffer line-renumbering verbs AUTONUM
            // and NUM are FFEDIT roots -- they act on the open buffer's sequence
            // columns (CR-CH-053 Req 2a.4 / Req 10), moved out of the FFCMD
            // ladder. Their eventual target verb NUMBER is NOT wired on main
            // (the NUMBER/UNNUM sequence-number engine is the deferred
            // ff-sequence-numbers effort); these remain the SAME redirect/
            // placeholder they are today, now claimed only when an editor Context
            // is active.
            ("AUTONUM", "AUTONUM"),
            ("NUM", "NUM"),
        ];
        let mut entries: Vec<AliasEntry> = Vec::with_capacity(raw.len());
        for &(surface, canonical) in raw {
            if let Some(existing) = entries.iter().find(|e| e.surface == surface) {
                // Collision: the same surface form maps to two canonicals -- an
                // authoring error (Requirement 6a.3).
                assert_eq!(
                    existing.canonical, canonical,
                    "alias collision: surface form '{surface}' maps to both \
                     '{}' and '{canonical}'",
                    existing.canonical
                );
            }
            // ffedit_english seeds the English base: the surface IS the display
            // text (In_Row_Surface), byte-identical at `en` (localization
            // Req 13.1 behaviour-neutral at en).
            entries.push(AliasEntry {
                surface: surface.to_string(),
                canonical: canonical.to_string(),
                command_template: None,
                origin: AliasOrigin::Base,
            });
        }
        Self { entries }
    }

    /// Build an alias table seeded with an IDENTITY row (`surface == canonical`)
    /// for every root command in `roots` (CR-NR-109,
    /// localization Requirement 11.3). This makes the synthesised identity rows
    /// EXPLICIT and universal: the root command is always resolvable in whatever
    /// source language it was authored, WITHOUT an author hand-writing an identity
    /// row. A duplicate root in `roots` is deduplicated (a harmless no-op), so a
    /// caller may pass overlapping root sets. An authored/overlaid row for the
    /// SAME canonical later REPLACES the synthesised default via the keyed overlay
    /// (Req 11.4), without changing the root.
    ///
    /// Validates: localization Requirement 11.3
    pub(crate) fn identity_for(roots: &[&str]) -> Self {
        let mut entries: Vec<AliasEntry> = Vec::with_capacity(roots.len());
        for &root in roots {
            let root = root.trim();
            if root.is_empty() || entries.iter().any(|e| e.surface.eq_ignore_ascii_case(root)) {
                continue;
            }
            entries.push(AliasEntry {
                surface: root.to_string(),
                canonical: root.to_string(),
                command_template: None,
                origin: AliasOrigin::Base,
            });
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
            .find(|e| e.surface.eq_ignore_ascii_case(s))
            .map(|e| e.canonical.as_str())
    }

    /// Resolve a surface-form verb token to its FULL alias entry
    /// (case-insensitive), or `None` when unowned. The CR-NR-112 pre-pass needs
    /// the entry's `command_template`, not just its canonical verb.
    /// `canonical_verb` is RETAINED for the verb-only/display callers.
    ///
    /// Validates: command-semantics Requirement 14.1
    pub(crate) fn resolve_alias_entry(&self, surface: &str) -> Option<&AliasEntry> {
        let s = surface.trim();
        self.entries
            .iter()
            .find(|e| e.surface.eq_ignore_ascii_case(s))
    }

    /// The distinct canonical (root) verbs this table resolves to, as owned
    /// strings in first-seen order. Used by the `LocaleAliasStore` to read back
    /// the seeded root set for a locale reload without the caller re-deriving it
    /// (CR-NR-109). The result is the set of ROOT commands, which is exactly what
    /// `identity_for` re-seeds from.
    pub(crate) fn canonical_verbs_owned(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for e in &self.entries {
            if !out.iter().any(|c| c.eq_ignore_ascii_case(&e.canonical)) {
                out.push(e.canonical.clone());
            }
        }
        out
    }

    /// The DISPLAYED (and typeable) surface for the entry whose canonical verb is
    /// `canonical` (case-insensitive), resolved under the active locale via
    /// [`AliasEntry::display_surface`]. Returns `None` when this table has no row
    /// for that canonical. This is the display accessor the menu-kind render path
    /// reads (Req 11.7); resolution/persistence still use `canonical_verb`.
    ///
    /// CR-NR-121 Stage B MENU-DISPLAY RULE (command-resolution-model Req 5.4):
    /// the menu command column shows the CANONICAL root overridden ONLY by a
    /// LOCALE translation (a `LocaleSeed` overlay), NEVER by a USER shorthand (a
    /// `UserOverride` overlay). Concretely, among the rows for a canonical this
    /// prefers, in order: a `LocaleSeed` row's surface (a locale translation
    /// still shows the translated surface, e.g. an `af` SOEK for FIND); otherwise
    /// the canonical's own surface -- a `Base` identity row if present, else the
    /// canonical itself -- so a `UserOverride` row (e.g. SS -> SNAPSHOT) does NOT
    /// replace the canonical in the menu column (the menu shows SNAPSHOT). The
    /// `UserOverride` row remains typeable and is still listed in the Alias
    /// Editor; only the MENU display surface ignores it.
    ///
    /// Validates: command-resolution-model Requirement 5.4;
    /// localization Requirement 11.1, 11.2, 11.7
    // The menu-kind render production caller is `menu_option_command_surfaces`
    // (CR-NR-109 FEAT-002, shell/locale_resolver.rs); also exercised by the Task
    // 11.1 unit tests.
    pub(crate) fn display_surface_for_canonical(&self, canonical: &str) -> Option<String> {
        let c = canonical.trim();
        let matching = || {
            self.entries
                .iter()
                .filter(|e| e.canonical.eq_ignore_ascii_case(c))
        };
        // Rule 5.4: a LOCALE translation (LocaleSeed) drives the menu display.
        if let Some(locale_row) = matching().find(|e| matches!(e.origin, AliasOrigin::LocaleSeed)) {
            return Some(locale_row.display_surface());
        }
        // Otherwise prefer the canonical's own surface (a Base identity row),
        // NEVER a UserOverride shorthand. Fall back to the canonical itself when
        // only a UserOverride row exists (so the menu shows the canonical root).
        if let Some(base_row) = matching().find(|e| matches!(e.origin, AliasOrigin::Base)) {
            return Some(base_row.display_surface());
        }
        // No Base/LocaleSeed row: the only row is a UserOverride. Show the
        // canonical root (not the user shorthand). If there is no row at all for
        // this canonical, return None so the caller falls back to option.command.
        matching().next().map(|_| c.to_string())
    }

    /// The OVERLAY alias rows of this table as `(surface, canonical,
    /// command_template)` triples -- the per-locale localisation surfaces (the
    /// shipped seed) and the user-override customisations the Alias Editor shows
    /// and edits (CR-CH-072 Req 6.1). The built-in base rows -- the hard-coded
    /// English FFEDIT verb aliases (X -> EXCLUDE, INCLUDE -> SHOW) and the
    /// synthesised identity roots -- are EXCLUDED: they are command shorthands,
    /// not localisation, so an `en` install with no seed/override shows an empty
    /// editor. Owned strings, first-seen order.
    ///
    /// Validates: command-configurator Requirement 6.1
    pub(crate) fn display_rows(&self) -> Vec<(String, String, Option<String>)> {
        self.entries
            .iter()
            .filter(|e| e.is_overlay())
            .map(|e| {
                (
                    e.surface.clone(),
                    e.canonical.clone(),
                    e.command_template.clone(),
                )
            })
            .collect()
    }

    // CR-NR-121 Stage B: the two `apply_locale_overlay` / `apply_locale_overlay_keyed`
    // methods were split into the child module `alias_table_overlay.rs` to keep
    // this file under the 400-line limit (declared via `mod alias_table_overlay;`
    // below). They are still `impl AliasTable` methods -- no behaviour change.

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

// The shared Command_Environment contract (`CommandEnvironment` trait +
// `EnvDispatchOutcome` enum) lives in `environment_contract.rs` (split out purely
// to keep this file under the 400-line rule once CR-NR-109 grew the AliasTable).
// Re-exported here so every existing `super::environment::CommandEnvironment` /
// `crate::shell::environment::EnvDispatchOutcome` path is unchanged.
// core-001 strip: the `CommandEnvironment` trait is no longer consumed (the
// FFEDIT environment object that implemented it is parked); only
// `EnvDispatchOutcome` is still used, by `dispatch_to_environment`.
pub(crate) use super::environment_contract::EnvDispatchOutcome;

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: the former `environment_is_supplied_by_kind`,
    // `active_environment_routes_through_kind`, and
    // `ffedit_claims_owned_verb_only_when_active_env_is_ffedit` tests moved to
    // `environment_registry.rs` (CR-CH-053 Task 17): the kind-supplied active-name
    // derivation and the active-wins gate now read from the built
    // `EnvironmentRegistry` (Req 13.3), so those behaviours are proven against the
    // registry (`active_name_is_supplied_by_kind`,
    // `is_ffedit_active_matches_editor_contexts_only`). The AliasTable tests
    // below stay here with the table they exercise.

    // CR-CH-072: the former `install_verb_find_soek_catalogue` helper (which
    // installed a `verb-find = SOEK` Fluent catalogue so `display_surface` could
    // resolve `t(verb-find)`) is removed. The command-surface `verb-*` indirection
    // no longer exists: the In_Row_Surface stored on the row IS the display text,
    // so these tests now assert the in-row surface directly without any catalogue.

    /// Validates: localization Requirement 13.1 -- the In_Row_Surface stored on
    /// the row IS the display text (one lookup, no `t(key)`); after a keyed
    /// overlay the localised in-row surface resolves to the SAME canonical the
    /// root yields (one source of truth), and `display_surface_for_canonical`
    /// returns the overlaid in-row surface.
    #[test]
    fn display_surface_returns_in_row_surface() {
        // CR-CH-072: an entry's display surface IS its in-row `surface` string.
        let entry = AliasEntry {
            surface: "SOEK".to_string(),
            canonical: "FIND".to_string(),
            command_template: None,
            origin: AliasOrigin::LocaleSeed,
        };
        assert_eq!(
            entry.display_surface(),
            "SOEK",
            "the display surface is the in-row surface string (Req 13.1)"
        );

        let plain = AliasEntry {
            surface: "CHANGE".to_string(),
            canonical: "CHANGE".to_string(),
            command_template: None,
            origin: AliasOrigin::Base,
        };
        assert_eq!(plain.display_surface(), "CHANGE");

        // After a keyed overlay onto the English base, the localised surface SOEK
        // resolves to the SAME canonical FIND the root yields (one source of
        // truth), and its in-row surface is the preferred display for FIND.
        let mut table = AliasTable::ffedit_english();
        table
            .apply_locale_overlay_keyed(
                "FFEDIT",
                &[("SOEK".to_string(), "FIND".to_string(), None)],
                AliasOrigin::LocaleSeed,
            )
            .expect("keyed overlay applies");
        assert_eq!(table.canonical_verb("SOEK"), Some("FIND"));
        assert_eq!(table.canonical_verb("FIND"), Some("FIND"));
        assert_eq!(
            table.display_surface_for_canonical("FIND"),
            Some("SOEK".to_string()),
            "the overlaid SOEK row's in-row surface is preferred for canonical FIND"
        );
    }

    /// Validates: localization Requirement 11.3 -- an identity row (surface ==
    /// canonical) is synthesised for every root command, so each resolves to
    /// itself; duplicates are deduplicated.
    #[test]
    fn synthesised_identity_row_exists_for_every_root() {
        let table = AliasTable::identity_for(&["CATALOGS", "FILES", "SETTINGS", "CATALOGS"]);
        assert_eq!(table.canonical_verb("CATALOGS"), Some("CATALOGS"));
        assert_eq!(table.canonical_verb("FILES"), Some("FILES"));
        assert_eq!(table.canonical_verb("SETTINGS"), Some("SETTINGS"));
        // Case-insensitive resolution of a synthesised identity row.
        assert_eq!(table.canonical_verb("files"), Some("FILES"));
        // An un-seeded root does not resolve.
        assert_eq!(table.canonical_verb("NOTAROOT"), None);
    }

    /// Validates: localization Requirement 11.4 -- an overlay row REPLACES the
    /// synthesised identity default for the SAME canonical WITHOUT changing the
    /// root: the Russian-root / English-surface case. Seed root NAYTI; apply an
    /// overlay row (surface FIND, canonical NAYTI). After it, FIND resolves to the
    /// root NAYTI, NAYTI still self-resolves, and the in-row surface for the row
    /// is FIND (CR-CH-072: the surface IS the display text, Req 13.1).
    #[test]
    fn overlay_row_replaces_synthesised_default_without_changing_root() {
        // Seed the Russian-authored root NAYTI (identity row).
        let mut table = AliasTable::identity_for(&["NAYTI"]);
        assert_eq!(table.canonical_verb("NAYTI"), Some("NAYTI"));

        // Apply the English-surface overlay row for the same canonical NAYTI.
        table
            .apply_locale_overlay_keyed(
                "FFEDIT",
                &[("FIND".to_string(), "NAYTI".to_string(), None)],
                AliasOrigin::LocaleSeed,
            )
            .expect("overlay row for a new surface of the same root applies");

        // FIND now resolves to the root NAYTI; the root itself still self-resolves.
        assert_eq!(
            table.canonical_verb("FIND"),
            Some("NAYTI"),
            "the English surface FIND resolves to the Russian root NAYTI"
        );
        assert_eq!(
            table.canonical_verb("NAYTI"),
            Some("NAYTI"),
            "the root is unchanged and still self-resolves"
        );
        // The overlaid row's display surface is its in-row surface FIND.
        assert_eq!(
            table.display_surface_for_canonical("NAYTI"),
            Some("FIND".to_string()),
            "the overlaid row for NAYTI is preferred and displays its in-row surface FIND"
        );
    }

    /// Validates: localization Requirement 11.4 -- re-stating a row whose surface
    /// matches an existing identity with a COMPATIBLE canonical UPDATES it (not a
    /// collision): the canonical is kept (CR-CH-072: the in-row surface is updated
    /// in place; there is no separate localisation key to attach).
    #[test]
    fn overlay_row_matching_identity_surface_updates_without_collision() {
        let mut table = AliasTable::identity_for(&["FIND"]);
        // Same surface FIND, same canonical FIND -> update in place (no collision).
        table
            .apply_locale_overlay_keyed(
                "FFEDIT",
                &[("FIND".to_string(), "FIND".to_string(), None)],
                AliasOrigin::LocaleSeed,
            )
            .expect("re-stating a surface with the same canonical must not collide");
        assert_eq!(table.canonical_verb("FIND"), Some("FIND"));
        assert_eq!(
            table.display_surface_for_canonical("FIND"),
            Some("FIND".to_string()),
            "the identity row's in-row surface is unchanged after a same-root update"
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

    /// Validates: command-environments Requirement 21.1 (CR-CH-069) -- the
    /// in-buffer line-renumbering verbs AUTONUM and NUM are seeded as FFEDIT
    /// canonical roots (moved out of the FFCMD ladder), so the FFEDIT alias table
    /// resolves them to themselves and `canonical_verbs_owned` lists them (the
    /// source `EnvironmentRegistry::root_commands("FFEDIT")` reads).
    #[test]
    fn ffedit_english_owns_autonum_and_num_roots() {
        let t = AliasTable::ffedit_english();
        assert_eq!(t.canonical_verb("AUTONUM"), Some("AUTONUM"));
        assert_eq!(t.canonical_verb("NUM"), Some("NUM"));
        // Case-insensitive, like every other seeded root.
        assert_eq!(t.canonical_verb("autonum"), Some("AUTONUM"));
        let owned = t.canonical_verbs_owned();
        assert!(
            owned.iter().any(|c| c == "AUTONUM") && owned.iter().any(|c| c == "NUM"),
            "ffedit_english canonical roots must include AUTONUM and NUM, got {owned:?}"
        );
    }

    /// Validates: command-semantics Requirement 14.1 -- a seeded entry carries
    /// `command_template = None` (a pure verb alias, back-compat), and
    /// `resolve_alias_entry` returns the whole entry case-insensitively (None for
    /// an unknown surface).
    #[test]
    fn resolve_alias_entry_returns_entry_with_template_field() {
        let t = AliasTable::ffedit_english();
        let entry = t
            .resolve_alias_entry("find")
            .expect("FIND resolves case-insensitively");
        assert_eq!(entry.canonical, "FIND");
        assert_eq!(
            entry.command_template, None,
            "a seeded entry is a pure verb alias (no template, Req 14.6)"
        );
        assert!(
            t.resolve_alias_entry("NOTAVERB").is_none(),
            "an unknown surface resolves to no entry"
        );
    }

    /// Validates: command-semantics Requirement 14.1 -- a KEYED overlay row
    /// carrying a parameter template produces an `AliasEntry` whose
    /// `command_template` is `Some(...)`, while `canonical_verb` and
    /// `resolve_alias_entry` still resolve the surface to its canonical.
    #[test]
    fn keyed_overlay_row_with_template_sets_command_template() {
        let mut table = AliasTable::ffedit_english();
        table
            .apply_locale_overlay_keyed(
                "FFEDIT",
                &[(
                    "ECOB".to_string(),
                    "EDIT".to_string(),
                    Some("FFWB.COBOL.MASTER(&1)".to_string()),
                )],
                AliasOrigin::LocaleSeed,
            )
            .expect("keyed overlay with a template applies");
        assert_eq!(table.canonical_verb("ECOB"), Some("EDIT"));
        let entry = table
            .resolve_alias_entry("ecob")
            .expect("ECOB resolves case-insensitively");
        assert_eq!(
            entry.command_template.as_deref(),
            Some("FFWB.COBOL.MASTER(&1)"),
            "the overlaid entry carries the parameter template"
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

    // === CR-NR-121 Stage B: overlay seed-vs-user discriminator ==============

    /// Validates: command-resolution-model Requirement 5.1 -- a LocaleSeed
    /// overlay row and a UserOverride overlay row are now DISTINGUISHABLE by their
    /// `origin` after applying each through `apply_locale_overlay_keyed` (both
    /// were indistinguishable `is_overlay = true` before Stage B). Both are still
    /// listed by `is_overlay()` for the Alias Editor; a Base identity row is not.
    #[test]
    fn locale_seed_and_user_override_rows_are_distinguishable_by_origin() {
        let mut table = AliasTable::identity_for(&["FIND", "SNAPSHOT"]);
        // A locale-translation overlay (the shipped seed) for FIND.
        table
            .apply_locale_overlay_keyed(
                "FFCMD",
                &[("SOEK".to_string(), "FIND".to_string(), None)],
                AliasOrigin::LocaleSeed,
            )
            .expect("locale seed overlay applies");
        // A user-shorthand overlay for SNAPSHOT.
        table
            .apply_locale_overlay_keyed(
                "FFCMD",
                &[("SS".to_string(), "SNAPSHOT".to_string(), None)],
                AliasOrigin::UserOverride,
            )
            .expect("user override overlay applies");

        let soek = table.resolve_alias_entry("SOEK").expect("SOEK row present");
        let ss = table.resolve_alias_entry("SS").expect("SS row present");
        assert_eq!(
            soek.origin,
            AliasOrigin::LocaleSeed,
            "the shipped-seed row is LocaleSeed origin"
        );
        assert_eq!(
            ss.origin,
            AliasOrigin::UserOverride,
            "the user-override row is UserOverride origin"
        );
        assert_ne!(
            soek.origin, ss.origin,
            "a locale translation and a user shorthand are now distinguishable (Req 5.1)"
        );
        // Both overlay layers are listed for the Alias Editor; the Base identity
        // rows are not.
        assert!(soek.is_overlay() && ss.is_overlay());
        let find_identity = table
            .resolve_alias_entry("FIND")
            .expect("FIND identity row present");
        // FIND's front row is the LocaleSeed overlay (SOEK moved identity aside),
        // so look at a pure Base identity (SNAPSHOT's identity is shadowed by SS,
        // so use the collection directly).
        let base_count = table.display_rows().len();
        assert_eq!(
            base_count, 2,
            "exactly the two overlay rows are listed (identity Base rows excluded)"
        );
        let _ = find_identity; // FIND resolves; identity retained under the overlay
        assert_eq!(table.canonical_verb("FIND"), Some("FIND"));
    }

    /// Validates: command-resolution-model Requirement 5.4 -- when the ONLY
    /// overlay for a canonical is a USER shorthand, the menu display surface is
    /// the CANONICAL root, NOT the user shorthand (SS -> SNAPSHOT displays
    /// SNAPSHOT). The shorthand stays typeable (`canonical_verb` still resolves
    /// SS) and listed in the Alias Editor (`display_rows` still carries it).
    #[test]
    fn user_override_does_not_drive_menu_display_surface() {
        let mut table = AliasTable::identity_for(&["SNAPSHOT"]);
        table
            .apply_locale_overlay_keyed(
                "FFCMD",
                &[("SS".to_string(), "SNAPSHOT".to_string(), None)],
                AliasOrigin::UserOverride,
            )
            .expect("user override applies");

        assert_eq!(
            table.display_surface_for_canonical("SNAPSHOT"),
            Some("SNAPSHOT".to_string()),
            "a user shorthand must NOT replace the canonical in the menu column (Req 5.4)"
        );
        // The shorthand is still typeable and still listed for the Alias Editor.
        assert_eq!(table.canonical_verb("SS"), Some("SNAPSHOT"));
        assert!(table
            .display_rows()
            .iter()
            .any(|(s, c, _)| s == "SS" && c == "SNAPSHOT"));
    }

    /// Validates: command-resolution-model Requirement 5.4 -- when there is NO
    /// Base identity row for a canonical and the ONLY overlay is a USER shorthand,
    /// the menu display surface still falls back to the canonical ROOT (never the
    /// shorthand).
    #[test]
    fn user_override_only_no_identity_still_shows_canonical() {
        let mut table = AliasTable::empty_for_test();
        table
            .apply_locale_overlay_keyed(
                "FFCMD",
                &[("SS".to_string(), "SNAPSHOT".to_string(), None)],
                AliasOrigin::UserOverride,
            )
            .expect("user override applies");
        assert_eq!(
            table.display_surface_for_canonical("SNAPSHOT"),
            Some("SNAPSHOT".to_string()),
            "with no Base row, a lone user shorthand still shows the canonical root (Req 5.4)"
        );
    }

    /// Validates: command-resolution-model Requirement 5.4 -- a LOCALE
    /// translation (a LocaleSeed overlay) DOES drive the menu display surface: an
    /// af-locale SOEK for FIND shows the translated surface SOEK in the menu
    /// column, even though a user shorthand never would.
    #[test]
    fn locale_seed_drives_menu_display_surface() {
        let mut table = AliasTable::identity_for(&["FIND"]);
        table
            .apply_locale_overlay_keyed(
                "FFEDIT",
                &[("SOEK".to_string(), "FIND".to_string(), None)],
                AliasOrigin::LocaleSeed,
            )
            .expect("locale seed applies");
        assert_eq!(
            table.display_surface_for_canonical("FIND"),
            Some("SOEK".to_string()),
            "a locale translation drives the menu display surface (Req 5.4)"
        );
    }

    /// Validates: command-resolution-model Requirement 5.4 -- a LocaleSeed
    /// translation WINS the menu display even when a UserOverride shorthand also
    /// exists for the same canonical: the translated surface is shown, not the
    /// shorthand.
    #[test]
    fn locale_seed_wins_over_user_override_for_menu_display() {
        let mut table = AliasTable::identity_for(&["FIND"]);
        table
            .apply_locale_overlay_keyed(
                "FFEDIT",
                &[("SOEK".to_string(), "FIND".to_string(), None)],
                AliasOrigin::LocaleSeed,
            )
            .expect("locale seed applies");
        table
            .apply_locale_overlay_keyed(
                "FFEDIT",
                &[("SK".to_string(), "FIND".to_string(), None)],
                AliasOrigin::UserOverride,
            )
            .expect("user override applies");
        assert_eq!(
            table.display_surface_for_canonical("FIND"),
            Some("SOEK".to_string()),
            "the LocaleSeed translation drives the menu display over a user shorthand (Req 5.4)"
        );
    }
}
