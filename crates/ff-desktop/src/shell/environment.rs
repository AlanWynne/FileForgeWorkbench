//! # Command Environments (CR-CH-053) -- E0 scaffold (Option 2)
//!
//! FFWB adopts the REXX/ISPF ADDRESS model: a set of named Command Environments,
//! each owning the commands relevant to its context. The active environment is
//! SUPPLIED BY the focused Context's KIND (not a central match the command
//! handler owns), with FFCMD as the always-present base (= `resolve_target`).
//!
//! ## E0 scope (Option 2)
//!
//! - The active environment is derived through the KIND: the built
//!   `EnvironmentRegistry` (see `environment_registry.rs`) maps a `BuiltinKind`
//!   (obtained via `BuiltinKind::from_tab_kind`, the kind's OWN mapping) to its
//!   command-environment NAME. The command handler does NOT enumerate `TabKind`
//!   -- it goes through the kind. This is the dependency inversion
//!   (Requirement 2.1, 9.1), now read from the built registry (Req 13.3).
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

//! The former `EnvironmentKind` enum / `environment_for_kind` / `active_environment`
//! derivation now live in the built `EnvironmentRegistry` (`environment_registry.rs`,
//! command-environments Req 13.1-13.4). This file holds the per-environment
//! `AliasTable`; the shared contract (`CommandEnvironment` + `EnvDispatchOutcome`)
//! lives in `environment_contract.rs`, re-exported here (CR-NR-109 400-line split).

/// One ALIAS TABLE entry: a surface form, the canonical (root) verb it resolves
/// to, and an OPTIONAL localisation key (CR-NR-109, localization Requirement
/// 11.1). The canonical verb is the dispatched/persisted identity; the surface
/// and localisation key are DISPLAY/typeable concerns only.
///
/// WHERE `localisation_key` is `Some(k)`, the DISPLAYED (and typeable) surface
/// for this entry is `ff_i18n::t(k)` resolved against the ACTIVE locale, reusing
/// the EXISTING Catalogue_Lookup_Seam with no new lookup mechanism (Req 11.1,
/// 11.2). WHERE it is `None`, the `surface` string itself is both displayed and
/// typeable (the pre-CR-NR-109 behaviour, so the `en` default is unchanged).
///
/// Validates: localization Requirement 11.1, 11.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AliasEntry {
    /// UPPERCASED typeable/displayed surface form (the fallback display when no
    /// localisation key is present).
    pub(crate) surface: String,
    /// The root (canonical) verb -- the dispatched/persisted identity.
    pub(crate) canonical: String,
    /// Optional `ff-i18n` key resolving the displayed surface per active locale.
    pub(crate) localisation_key: Option<String>,
    /// CR-NR-112: OPTIONAL parameter template (e.g. `FFWB.COBOL.MASTER(&1)`).
    /// `Some`+non-empty -> invoking the alias expands `canonical` + ` ` + the
    /// substituted template into the ROOT command (Req 14.1/14.2); `None`/empty
    /// is a pure verb alias, byte-identical to today (Req 14.6). Pre-resolution
    /// form only; the ROOT stays the dispatched/persisted identity (Req 19.4).
    ///
    /// Validates: command-semantics Requirement 14.1
    pub(crate) command_template: Option<String>,
}

impl AliasEntry {
    /// The DISPLAYED (and typeable) surface for this entry under the active
    /// locale: `ff_i18n::t(key)` when a localisation key is present, else the raw
    /// `surface` string (CR-NR-109, localization Requirement 11.1, 11.2). The
    /// `t()` seam returns the key itself on a miss, so an unresolved key falls
    /// back to the key string, and a `None` key always yields the raw surface.
    ///
    /// Validates: localization Requirement 11.1, 11.2
    // Consumed by `display_surface_for_canonical` (the menu-kind render reader,
    // now called by `menu_option_command_surfaces`, CR-NR-109 FEAT-002) and by
    // the Task 11.1 unit tests.
    pub(crate) fn display_surface(&self) -> String {
        match &self.localisation_key {
            Some(key) => ff_i18n::t(key),
            None => self.surface.clone(),
        }
    }
}

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
            // ffedit_english seeds the English base: no localisation key, so the
            // `en` display/typeable surface is byte-identical to the raw surface
            // (localization Req 11.1 behaviour-neutral at en).
            entries.push(AliasEntry {
                surface: surface.to_string(),
                canonical: canonical.to_string(),
                localisation_key: None,
                command_template: None,
            });
        }
        Self { entries }
    }

    /// Build an alias table seeded with an IDENTITY row (`surface == canonical`,
    /// `localisation_key = None`) for every root command in `roots` (CR-NR-109,
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
                localisation_key: None,
                command_template: None,
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
    /// for that canonical. When several rows share a canonical, the FIRST (an
    /// overlaid row, which the keyed overlay places ahead of the synthesised
    /// identity default) wins. This is the display accessor the menu-kind render
    /// path reads (Req 11.7); resolution/persistence still use `canonical_verb`.
    ///
    /// Validates: localization Requirement 11.1, 11.2, 11.7
    // The menu-kind render production caller is `menu_option_command_surfaces`
    // (CR-NR-109 FEAT-002, shell/locale_resolver.rs); also exercised by the Task
    // 11.1 unit tests.
    pub(crate) fn display_surface_for_canonical(&self, canonical: &str) -> Option<String> {
        let c = canonical.trim();
        self.entries
            .iter()
            .find(|e| e.canonical.eq_ignore_ascii_case(c))
            .map(AliasEntry::display_surface)
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
    // KEYLESS shim: the production on-disk path uses `apply_locale_overlay_keyed`
    // (via the alias store); this keyless entry is consumed by the keyless
    // `AliasCatalogue::load_into` and the localization Req 5.x fixture tests, so a
    // non-test build sees no direct production caller. Retained as the parallel
    // keyless path.
    #[allow(dead_code)]
    pub(crate) fn apply_locale_overlay(
        &mut self,
        environment_name: &str,
        rows: &[(String, String)],
    ) -> Result<(), super::alias_overlay::AliasOverlayError> {
        // Delegate to the keyed form with no localisation key per row, so the two
        // entry points share ONE scratch-copy collision discipline (Req 5.6,
        // 11.10). Existing (surface, canonical) callers are unchanged.
        let keyed: Vec<super::alias_overlay::KeyedAliasRow> = rows
            .iter()
            .map(|(s, c)| (s.clone(), c.clone(), None, None))
            .collect();
        self.apply_locale_overlay_keyed(environment_name, &keyed)
    }

    /// Overlay a locale's `(surface, canonical, localisation_key?)` rows ON TOP of
    /// this table's existing base entries, carrying the per-row localisation key
    /// through to the entry (CR-NR-109, localization Requirement 11.1, 11.4). This
    /// is the keyed entry point; [`apply_locale_overlay`] is the keyless shim that
    /// defaults `localisation_key = None`. BOTH reuse the SAME scratch-copy
    /// collision machinery, so a within-environment collision leaves `self`
    /// UNCHANGED and the Identity_Base retained (Req 5.6, 11.10).
    ///
    /// An incoming row whose surface already resolves (case-insensitively) to the
    /// SAME canonical REPLACES that existing row in place -- so an authored row for
    /// a synthesised identity default UPDATES it (e.g. attaching a localisation
    /// key) rather than erroring (Req 11.4), and the replacement is positioned
    /// AHEAD so a canonical lookup prefers the overlaid row's display surface. A
    /// surface resolving to a DIFFERENT canonical is a collision.
    ///
    /// Validates: localization Requirement 5.1, 5.5, 5.6, 10.3, 11.1, 11.4, 11.10
    // Production caller chain: the on-disk alias store's `apply_overlay_from_dir`
    // -> `AliasCatalogue::load_into_keyed` -> here (CR-NR-109), so no dead_code
    // allow is needed.
    pub(crate) fn apply_locale_overlay_keyed(
        &mut self,
        environment_name: &str,
        rows: &[super::alias_overlay::KeyedAliasRow],
    ) -> Result<(), super::alias_overlay::AliasOverlayError> {
        use super::alias_overlay::AliasOverlayError;

        // Validate the WHOLE overlay against a scratch copy first, so a collision
        // leaves `self` untouched (Identity_Base retained, Req 5.6). Only on full
        // success do we commit the scratch copy back.
        let mut scratch = self.entries.clone();
        for (surface, canonical, key, template) in rows {
            let surface_trimmed = surface.trim();
            // CR-NR-112: an empty/whitespace template deserialises/overlays to
            // None (a pure verb alias), so the byte-identical legacy path is
            // preserved (command-semantics Req 14.6).
            let template = template
                .as_ref()
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty());
            if let Some(pos) = scratch
                .iter()
                .position(|e| e.surface.eq_ignore_ascii_case(surface_trimmed))
            {
                if scratch[pos].canonical != *canonical {
                    return Err(AliasOverlayError::Collision {
                        environment: environment_name.to_string(),
                        surface: surface_trimmed.to_string(),
                        existing: scratch[pos].canonical.clone(),
                        incoming: canonical.clone(),
                    });
                }
                // Same canonical: an authored/overlaid row UPDATES the existing
                // (possibly synthesised-identity) row rather than colliding
                // (Req 11.4). Move it to the front and apply the new key and
                // template (CR-NR-112), so a canonical lookup prefers the
                // overlaid display surface and the entry carries the template.
                let mut updated = scratch.remove(pos);
                updated.surface = surface_trimmed.to_string();
                updated.localisation_key = key.clone();
                updated.command_template = template;
                scratch.insert(0, updated);
                continue;
            }
            scratch.insert(
                0,
                AliasEntry {
                    surface: surface_trimmed.to_string(),
                    canonical: canonical.clone(),
                    localisation_key: key.clone(),
                    command_template: template,
                },
            );
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

// The shared Command_Environment contract (`CommandEnvironment` trait +
// `EnvDispatchOutcome` enum) lives in `environment_contract.rs` (split out purely
// to keep this file under the 400-line rule once CR-NR-109 grew the AliasTable).
// Re-exported here so every existing `super::environment::CommandEnvironment` /
// `crate::shell::environment::EnvDispatchOutcome` path is unchanged.
pub(crate) use super::environment_contract::{CommandEnvironment, EnvDispatchOutcome};

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

    /// Install a TEST catalogue defining `verb-find = SOEK` so a localised
    /// display surface resolves end-to-end through the real `ff_i18n::t` seam.
    ///
    /// CRITICAL: builds from a COPY of the SHIPPED English base PLUS the extra
    /// `verb-find` key, NOT a one-key catalogue. The i18n seam is PROCESS-WIDE and
    /// the ff-desktop test binary runs every `shell::*` test (these unit tests AND
    /// the full-shell `get_by_label` tests in `tests_focus`) in ONE process, so a
    /// one-key catalogue would strip every other English label and break unrelated
    /// label-query tests that run later in the same process. Mirrors the shared
    /// `tests_common::install_verb_find_soek_catalogue` helper.
    /// Install the `verb-find = SOEK` test catalogue as a PER-THREAD override
    /// (test isolation, `ff_i18n::set_test_catalogue`), so these global-catalogue
    /// tests are independent under threaded `cargo test` -- each test thread has
    /// its own catalogue and concurrent tests cannot clobber it. No lock, full
    /// parallelism.
    fn install_verb_find_soek_catalogue() {
        let shipped_root = crate::shell::i18n_bootstrap::resolve_i18n_root()
            .expect("shipped i18n/ root must resolve for the verb-find test catalogue");
        let shipped_en = shipped_root.join("en");
        let dir = tempfile::TempDir::new().expect("tempdir");
        let en = dir.path().join("en");
        std::fs::create_dir_all(&en).expect("create en dir");
        if let Ok(entries) = std::fs::read_dir(&shipped_en) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("ftl") {
                    if let Some(name) = path.file_name() {
                        // SKIP the shipped verb catalogue (`verb-find = FIND`,
                        // CR-NR-109 Task 11.7): copying it beside the test's own
                        // `verb-find = SOEK` below would duplicate `verb-find` in
                        // the en bundle (a Fluent duplicate-message error). The
                        // test's zz-test-verbs.ftl is the sole `verb-find` source.
                        if name.eq_ignore_ascii_case("ff-desktop-verbs.ftl") {
                            continue;
                        }
                        let _ = std::fs::copy(&path, en.join(name));
                    }
                }
            }
        }
        std::fs::write(en.join("zz-test-verbs.ftl"), "verb-find = SOEK\n").expect("write ftl");
        let catalogue =
            ff_i18n::Catalogue::load_identity_base(dir.path()).expect("test catalogue loads");
        ff_i18n::set_test_catalogue(catalogue);
    }

    /// Validates: localization Requirement 11.1, 11.2 -- an entry carrying a
    /// localisation key displays the `t(key)` resolution (and that string is the
    /// one typeable surface); a None-key entry returns its raw surface; after a
    /// keyed overlay, `canonical_verb` of the localised surface still resolves to
    /// the SAME canonical (one source of truth, Req 11.2).
    #[test]
    fn display_surface_resolves_localisation_key_else_raw_surface() {
        install_verb_find_soek_catalogue();

        // Keyed entry: surface FIND, canonical FIND, key verb-find -> displays SOEK.
        let keyed = AliasEntry {
            surface: "FIND".to_string(),
            canonical: "FIND".to_string(),
            localisation_key: Some("verb-find".to_string()),
            command_template: None,
        };
        assert_eq!(
            keyed.display_surface(),
            "SOEK",
            "a keyed entry must display the t(key) resolution"
        );

        // None-key entry returns its raw surface unchanged.
        let plain = AliasEntry {
            surface: "CHANGE".to_string(),
            canonical: "CHANGE".to_string(),
            localisation_key: None,
            command_template: None,
        };
        assert_eq!(
            plain.display_surface(),
            "CHANGE",
            "a None-key entry must return its raw surface"
        );

        // After a keyed overlay onto the English base, the localised surface SOEK
        // resolves to the SAME canonical FIND the root yields (Req 11.2).
        let mut table = AliasTable::ffedit_english();
        table
            .apply_locale_overlay_keyed(
                "FFEDIT",
                &[(
                    "SOEK".to_string(),
                    "FIND".to_string(),
                    Some("verb-find".to_string()),
                    None,
                )],
            )
            .expect("keyed overlay applies");
        assert_eq!(table.canonical_verb("SOEK"), Some("FIND"));
        assert_eq!(table.canonical_verb("FIND"), Some("FIND"));
        assert_eq!(
            table.display_surface_for_canonical("FIND"),
            Some("SOEK".to_string()),
            "the overlaid SOEK row's display surface is preferred for canonical FIND"
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
    /// `en` overlay row (surface FIND, canonical NAYTI, key verb-find). After it,
    /// FIND resolves to the root NAYTI, NAYTI still self-resolves, and the display
    /// surface for the row is the t(key) resolution.
    #[test]
    fn overlay_row_replaces_synthesised_default_without_changing_root() {
        install_verb_find_soek_catalogue();

        // Seed the Russian-authored root NAYTI (identity row).
        let mut table = AliasTable::identity_for(&["NAYTI"]);
        assert_eq!(table.canonical_verb("NAYTI"), Some("NAYTI"));

        // Apply the English-surface overlay row for the same canonical NAYTI.
        table
            .apply_locale_overlay_keyed(
                "FFEDIT",
                &[(
                    "FIND".to_string(),
                    "NAYTI".to_string(),
                    Some("verb-find".to_string()),
                    None,
                )],
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
        // The overlaid row's display surface is the localisation-key resolution.
        assert_eq!(
            table.display_surface_for_canonical("NAYTI"),
            Some("SOEK".to_string()),
            "the overlaid row for NAYTI is preferred and displays t(verb-find)"
        );
    }

    /// Validates: localization Requirement 11.4 -- re-stating a row whose surface
    /// matches an existing identity with a COMPATIBLE canonical UPDATES it (not a
    /// collision): attaching a localisation key to an existing identity row keeps
    /// the canonical and gains the key.
    #[test]
    fn overlay_row_matching_identity_surface_updates_without_collision() {
        install_verb_find_soek_catalogue();
        let mut table = AliasTable::identity_for(&["FIND"]);
        // Same surface FIND, same canonical FIND, now with a key -> update in place.
        table
            .apply_locale_overlay_keyed(
                "FFEDIT",
                &[(
                    "FIND".to_string(),
                    "FIND".to_string(),
                    Some("verb-find".to_string()),
                    None,
                )],
            )
            .expect("re-stating a surface with the same canonical must not collide");
        assert_eq!(table.canonical_verb("FIND"), Some("FIND"));
        assert_eq!(
            table.display_surface_for_canonical("FIND"),
            Some("SOEK".to_string()),
            "the identity row gained the localisation key via an update"
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
                    None,
                    Some("FFWB.COBOL.MASTER(&1)".to_string()),
                )],
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
}
