//! # Per-environment, per-locale alias-catalogue LOADER (CR-NR-103, localization
//! Requirement 5.1-5.6, 10.3)
//!
//! This module is the LOADER MECHANISM that layers a selected locale's surface
//! forms onto the EXISTING per-environment `AliasTable` (see
//! `shell/environment.rs`). It adds NO new alias mechanism and NO new dispatch
//! path: a localized surface form loaded here resolves through the IDENTICAL
//! `AliasTable::canonical_verb` seam to the SAME canonical English verb, so
//! dispatch stays behaviour-neutral (Req 5.1, command-environments Req 6a.1/6a.5).
//!
//! ## Model
//!
//! - ONE `AliasCatalogue` per LOCALE (Req 5.2). Selecting a different locale
//!   loads a DIFFERENT catalogue; only the active locale's surface forms become
//!   resident, never a union of all locales.
//! - A catalogue is organised PER ENVIRONMENT (Req 5.3): e.g. a French FFEDIT
//!   set and a French FFCMD set, each loaded into the CORRESPONDING environment's
//!   `AliasTable`. The SAME surface form MAY map to a different canonical in a
//!   different environment -- the collision check is scoped WITHIN one
//!   environment's table (Req 5.6).
//! - Alias rows are language-agnostic DATA with NO per-row locale tag (Req 5.4);
//!   the single `ui.locale` selection picks which catalogue is loaded.
//! - English CANONICAL verbs remain resolvable in EVERY locale (Req 5.5): the
//!   catalogue is LAYERED ON TOP of the English Identity_Base via
//!   `AliasTable::apply_locale_overlay`, not a replacement.
//!
//! ## Scope (Task 4)
//!
//! This ships the IN-MEMORY loader + overlay logic, proven with FIXTURE data.
//! The concrete on-disk `.ftl`/TOML catalogue reader (design.md names
//! `i18n/<locale>/aliases/<environment>.toml`) is DEFERRED to Phase 3; Task 4
//! does NOT author real fr/de catalogue DATA.
//!
//! See docs/specs/localization/{requirements,design}.md.

// Task 4 ships the loader MECHANISM; its runtime consumer (the locale-switch
// path that reads an on-disk catalogue and loads it into the live environment
// tables) arrives in Phase 3 (Task 10). Until then these items are exercised by
// unit tests only, so a non-test build sees them as unused -- mirror the
// `#[allow(dead_code)]` already used for the ahead-of-consumer `CommandEnvironment`
// trait and `EnvironmentKind::name` in `environment.rs`.
#![allow(dead_code)]

use super::environment::AliasTable;

/// Recoverable overlay failure (NOT a panic): a within-environment alias
/// collision. The loader rejects the catalogue FOR THAT ENVIRONMENT and retains
/// the Identity_Base (localization Requirement 5.6), using the same conflict
/// discipline as command-environments Requirement 6a.3 but as a FALLIBLE result
/// rather than a construction-time panic.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub(crate) enum AliasOverlayError {
    /// A surface form in the overlay maps to a different canonical than the one
    /// already resident in this environment's table (a base identity, a base
    /// alias, or an earlier overlay row in the same apply).
    #[error(
        "alias overlay collision in environment '{environment}': surface form \
         '{surface}' maps to both '{existing}' and '{incoming}'"
    )]
    Collision {
        /// The environment whose table rejected the overlay.
        environment: String,
        /// The colliding surface form (trimmed).
        surface: String,
        /// The canonical already resident for that surface form.
        existing: String,
        /// The canonical the overlay row tried to introduce.
        incoming: String,
    },
}

/// A per-environment set of language-agnostic `(surface-form -> canonical-verb)`
/// rows for ONE locale, carrying NO per-row locale tag (localization
/// Requirement 5.4). The `ui.locale` selection picks which catalogue is loaded
/// (Req 5.2); within the catalogue the rows are grouped by environment name
/// (e.g. "FFEDIT", "FFCMD") so each group loads into the CORRESPONDING
/// environment's `AliasTable` (Req 5.3).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct AliasCatalogue {
    /// Keyed by environment name; each value is the set of
    /// `(surface-UPPERCASED, canonical)` overlay rows for that environment.
    per_environment: Vec<(String, Vec<(String, String)>)>,
}

impl AliasCatalogue {
    /// Build an empty catalogue (no environments).
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Add (or replace) the overlay row set for one environment. Rows are
    /// language-agnostic `(surface, canonical)` pairs with no locale tag
    /// (Req 5.4).
    pub(crate) fn with_environment(
        mut self,
        environment_name: impl Into<String>,
        rows: Vec<(String, String)>,
    ) -> Self {
        let name = environment_name.into();
        if let Some(slot) = self.per_environment.iter_mut().find(|(n, _)| *n == name) {
            slot.1 = rows;
        } else {
            self.per_environment.push((name, rows));
        }
        self
    }

    /// The overlay rows this catalogue carries for `environment_name`, if any.
    pub(crate) fn rows_for(&self, environment_name: &str) -> Option<&[(String, String)]> {
        self.per_environment
            .iter()
            .find(|(n, _)| n == environment_name)
            .map(|(_, rows)| rows.as_slice())
    }

    /// Load this catalogue's rows for `environment_name` INTO the given
    /// environment `table`, layering the locale's surface forms on top of the
    /// English Identity_Base (localization Requirement 5.3, 5.5). A
    /// within-environment collision rejects THIS environment's overlay and
    /// retains its Identity_Base (Req 5.6, scoped per environment) -- other
    /// environments loaded by separate `load_into` calls are unaffected. When the
    /// catalogue carries no rows for this environment, the base table is left
    /// unchanged (`Ok`).
    ///
    /// Validates: localization Requirement 5.2, 5.3, 5.5, 5.6
    pub(crate) fn load_into(
        &self,
        environment_name: &str,
        table: &mut AliasTable,
    ) -> Result<(), AliasOverlayError> {
        match self.rows_for(environment_name) {
            Some(rows) => table.apply_locale_overlay(environment_name, rows),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convenience: an owned `(surface, canonical)` row from `&str` literals.
    fn row(surface: &str, canonical: &str) -> (String, String) {
        (surface.to_string(), canonical.to_string())
    }

    /// A synthetic `fr`-like FFEDIT overlay (NOT real French data -- fixture only,
    /// Phase 3 ships real catalogues). CHERCHER -> FIND, CHANGER -> CHANGE.
    fn fr_ffedit_rows() -> Vec<(String, String)> {
        vec![row("CHERCHER", "FIND"), row("CHANGER", "CHANGE")]
    }

    /// Validates: localization Requirement 5.1, 5.5 -- a locale overlay layers its
    /// surface forms on top of the English base; both the localized surface
    /// (CHERCHER) and the base English canonical (FIND) resolve to the SAME
    /// canonical FIND, case-insensitively, through the identical seam.
    #[test]
    fn overlay_layers_locale_surface_on_top_of_english_base() {
        let mut table = AliasTable::ffedit_english();
        table
            .apply_locale_overlay("FFEDIT", &fr_ffedit_rows())
            .expect("non-colliding fr overlay applies cleanly");

        assert_eq!(table.canonical_verb("CHERCHER"), Some("FIND"));
        assert_eq!(table.canonical_verb("FIND"), Some("FIND")); // base retained
        assert_eq!(table.canonical_verb("chercher"), Some("FIND")); // case-insensitive
        assert_eq!(table.canonical_verb("CHANGER"), Some("CHANGE"));
    }

    /// Validates: localization Requirement 5.1 -- a localized surface form yields
    /// the SAME canonical a bare English verb yields, so dispatch reaches the
    /// identical canonical match arm (behaviour-neutral).
    #[test]
    fn overlay_resolves_through_identical_canonical_code_path() {
        let mut table = AliasTable::ffedit_english();
        table
            .apply_locale_overlay("FFEDIT", &fr_ffedit_rows())
            .expect("overlay applies");

        assert_eq!(
            table.canonical_verb("CHERCHER"),
            table.canonical_verb("FIND"),
            "localized surface must resolve to the same canonical as the English verb"
        );
    }

    /// Validates: localization Requirement 5.2 -- selecting a different locale
    /// loads a DIFFERENT catalogue; only the active locale's surface forms are
    /// resident (NOT a union). A fr overlay knows CHERCHER but not the de-only
    /// SUCHEN; a separately-built de overlay knows SUCHEN but not CHERCHER.
    #[test]
    fn selecting_a_different_locale_loads_a_different_catalogue_not_a_union() {
        let mut fr = AliasTable::ffedit_english();
        fr.apply_locale_overlay("FFEDIT", &fr_ffedit_rows())
            .expect("fr overlay applies");
        assert_eq!(fr.canonical_verb("CHERCHER"), Some("FIND"));
        assert_eq!(
            fr.canonical_verb("SUCHEN"),
            None,
            "fr table has no de surface"
        );

        let mut de = AliasTable::ffedit_english();
        de.apply_locale_overlay("FFEDIT", &[row("SUCHEN", "FIND")])
            .expect("de overlay applies");
        assert_eq!(de.canonical_verb("SUCHEN"), Some("FIND"));
        assert_eq!(
            de.canonical_verb("CHERCHER"),
            None,
            "de table has no fr surface -- no union across locales"
        );
    }

    /// Validates: localization Requirement 5.3 -- a catalogue is organised per
    /// environment; the SAME surface form may map to a DIFFERENT canonical in a
    /// different environment, each loading into its corresponding table. This is
    /// NOT a collision because the check is scoped within one environment.
    #[test]
    fn per_locale_catalogue_is_organised_per_environment() {
        // FFEDIT: X is the base alias for EXCLUDE. A second (FFCMD-like)
        // environment maps the SAME surface X to a DIFFERENT canonical EXIT.
        let catalogue = AliasCatalogue::new()
            .with_environment("FFEDIT", vec![row("CHERCHER", "FIND")])
            .with_environment("FFCMD", vec![row("X", "EXIT")]);

        let mut ffedit = AliasTable::ffedit_english();
        catalogue
            .load_into("FFEDIT", &mut ffedit)
            .expect("ffedit overlay applies");
        assert_eq!(ffedit.canonical_verb("CHERCHER"), Some("FIND"));
        assert_eq!(ffedit.canonical_verb("X"), Some("EXCLUDE")); // base unchanged

        // Model the second environment's base as an empty identity table so the
        // loader logic is exercised without touching FFCMD dispatch.
        let mut ffcmd = AliasTable::empty_for_test();
        catalogue
            .load_into("FFCMD", &mut ffcmd)
            .expect("ffcmd overlay applies -- same surface, different env, no collision");
        assert_eq!(ffcmd.canonical_verb("X"), Some("EXIT"));
    }

    /// Validates: localization Requirement 5.4 -- overlay rows carry no per-row
    /// locale tag; a row is exactly `(surface, canonical)` and the catalogue is
    /// selected by locale, not per row. Asserted structurally via the row shape.
    #[test]
    fn overlay_rows_carry_no_per_row_locale_tag() {
        let rows = fr_ffedit_rows();
        // Each row destructures into exactly two fields: surface and canonical.
        // There is no third (locale) field -- selection is by catalogue, Req 5.4.
        let (surface, canonical) = &rows[0];
        assert_eq!(surface, "CHERCHER");
        assert_eq!(canonical, "FIND");
    }

    /// Validates: localization Requirement 5.5 -- every base English canonical
    /// still resolves after a locale overlay; the overlay is a layer, not a
    /// replacement.
    #[test]
    fn english_canonical_remains_resolvable_in_every_locale() {
        let mut table = AliasTable::ffedit_english();
        table
            .apply_locale_overlay("FFEDIT", &fr_ffedit_rows())
            .expect("overlay applies");

        for (surface, expected) in [
            ("FIND", "FIND"),
            ("CHANGE", "CHANGE"),
            ("EXCLUDE", "EXCLUDE"),
            ("X", "EXCLUDE"),
            ("INCLUDE", "SHOW"),
        ] {
            assert_eq!(
                table.canonical_verb(surface),
                Some(expected),
                "base English '{surface}' must stay resolvable under a locale overlay"
            );
        }
    }

    /// Validates: localization Requirement 5.6 -- a within-environment collision
    /// (same surface mapping to two canonicals) causes the overlay to be
    /// REJECTED and the Identity_Base RETAINED; no panic (fallible path).
    #[test]
    fn within_environment_collision_is_rejected_and_identity_base_retained() {
        let mut table = AliasTable::ffedit_english();
        // Capture pre-overlay resolutions.
        let find_before = table.canonical_verb("FIND").map(str::to_string);
        let x_before = table.canonical_verb("X").map(str::to_string);

        // CHERCHER mapped to two different canonicals within one overlay.
        let colliding = vec![row("CHERCHER", "FIND"), row("CHERCHER", "CHANGE")];
        let err = table
            .apply_locale_overlay("FFEDIT", &colliding)
            .expect_err("colliding overlay must be rejected");
        assert_eq!(
            err,
            AliasOverlayError::Collision {
                environment: "FFEDIT".to_string(),
                surface: "CHERCHER".to_string(),
                existing: "FIND".to_string(),
                incoming: "CHANGE".to_string(),
            }
        );

        // Identity_Base retained: pre-overlay resolutions unchanged, and the
        // overlay surface does NOT resolve.
        assert_eq!(
            table.canonical_verb("FIND").map(str::to_string),
            find_before
        );
        assert_eq!(table.canonical_verb("X").map(str::to_string), x_before);
        assert_eq!(
            table.canonical_verb("CHERCHER"),
            None,
            "a rejected overlay must leave no partial surface forms resident"
        );
    }

    /// Validates: localization Requirement 5.6 -- a surface colliding with an
    /// existing BASE canonical to a DIFFERENT target is rejected; the base is
    /// retained.
    #[test]
    fn overlay_surface_colliding_with_base_canonical_is_rejected() {
        let mut table = AliasTable::ffedit_english();
        // FIND already resolves to FIND; mapping it to CHANGE is a collision.
        let err = table
            .apply_locale_overlay("FFEDIT", &[row("FIND", "CHANGE")])
            .expect_err("remapping an existing canonical is a collision");
        assert!(matches!(err, AliasOverlayError::Collision { .. }));
        assert_eq!(table.canonical_verb("FIND"), Some("FIND"), "base retained");
    }

    /// Validates: localization Requirement 5.6 -- the collision check is scoped
    /// WITHIN one environment. The same conflicting surface loaded into TWO
    /// DIFFERENT environments is accepted (no cross-environment collision); a
    /// conflicting pair WITHIN one environment is rejected for THAT environment
    /// only, while other environments still load.
    #[test]
    fn collision_check_is_scoped_within_one_environment() {
        // Same surface SRCH mapping to different canonicals in two environments.
        let catalogue = AliasCatalogue::new()
            .with_environment("FFEDIT", vec![row("SRCH", "FIND")])
            .with_environment("FFCMD", vec![row("SRCH", "EXIT")]);

        let mut ffedit = AliasTable::ffedit_english();
        catalogue
            .load_into("FFEDIT", &mut ffedit)
            .expect("ffedit SRCH->FIND loads");
        let mut ffcmd = AliasTable::empty_for_test();
        catalogue
            .load_into("FFCMD", &mut ffcmd)
            .expect("ffcmd SRCH->EXIT loads -- different env, not a collision");
        assert_eq!(ffedit.canonical_verb("SRCH"), Some("FIND"));
        assert_eq!(ffcmd.canonical_verb("SRCH"), Some("EXIT"));

        // A conflicting pair WITHIN one environment IS rejected, and a separate
        // clean environment in the same catalogue still loads.
        let mixed = AliasCatalogue::new()
            .with_environment("FFEDIT", vec![row("SRCH", "FIND"), row("SRCH", "CHANGE")])
            .with_environment("FFCMD", vec![row("SRCH", "EXIT")]);
        let mut bad = AliasTable::ffedit_english();
        assert!(
            mixed.load_into("FFEDIT", &mut bad).is_err(),
            "within-env collision rejected for FFEDIT"
        );
        let mut good = AliasTable::empty_for_test();
        assert!(
            mixed.load_into("FFCMD", &mut good).is_ok(),
            "the non-colliding FFCMD environment still loads"
        );
        assert_eq!(good.canonical_verb("SRCH"), Some("EXIT"));
    }

    /// Validates: localization Requirement 10.3 -- the canonical English verb is
    /// the recorded/persisted value regardless of locale; a localized surface
    /// form resolves to the canonical, never to itself.
    #[test]
    fn canonical_verb_is_the_recorded_persisted_value_regardless_of_locale() {
        let mut table = AliasTable::ffedit_english();
        table
            .apply_locale_overlay("FFEDIT", &fr_ffedit_rows())
            .expect("overlay applies");
        // The value a caller would record/persist is the canonical, not CHERCHER.
        assert_eq!(table.canonical_verb("CHERCHER"), Some("FIND"));
        assert_ne!(table.canonical_verb("CHERCHER"), Some("CHERCHER"));
    }

    /// Validates: localization Requirement 5.1 (and 10.2) -- with NO overlay the
    /// table resolves exactly as before; the owned-String storage change is
    /// behaviour-neutral for the English case.
    #[test]
    fn english_only_no_overlay_is_behaviour_identical() {
        let table = AliasTable::ffedit_english();
        assert_eq!(table.canonical_verb("X"), Some("EXCLUDE"));
        assert_eq!(table.canonical_verb("INCLUDE"), Some("SHOW"));
        assert_eq!(table.canonical_verb("FIND"), Some("FIND"));
        assert_eq!(table.canonical_verb("NOTAVERB"), None);
    }

    /// Validates: localization Requirement 5.6 -- a catalogue with no rows for an
    /// environment leaves that environment's base table unchanged.
    #[test]
    fn catalogue_without_rows_for_environment_leaves_base_unchanged() {
        let catalogue = AliasCatalogue::new().with_environment("FFCMD", vec![row("X", "EXIT")]);
        let mut ffedit = AliasTable::ffedit_english();
        catalogue
            .load_into("FFEDIT", &mut ffedit)
            .expect("no FFEDIT rows -> base unchanged");
        assert_eq!(ffedit.canonical_verb("X"), Some("EXCLUDE"));
        assert_eq!(ffedit.canonical_verb("CHERCHER"), None);
    }
}
