//! # `AliasTable` locale/user overlay application (CR-NR-121 Stage B split)
//!
//! Split out of `environment.rs` to keep that file under the 400-line source
//! limit (rust-standards.md / rust-ffwb-layout.md). As a CHILD module of
//! `environment` it legitimately reaches `AliasTable`'s private `entries` field
//! and the re-exported `AliasEntry` / `AliasOrigin` types. No behaviour change:
//! these are the same two `AliasTable` methods, now in their own `impl` block.

use super::{AliasEntry, AliasOrigin, AliasTable};
use crate::shell::alias_overlay::{AliasOverlayError, KeyedAliasRow};

impl AliasTable {
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
    ) -> Result<(), AliasOverlayError> {
        // Delegate to the keyed form with no template per row, so the two entry
        // points share ONE scratch-copy collision discipline (Req 5.6, 11.10).
        // Existing (surface, canonical) callers are unchanged. CR-NR-121: the
        // keyless shim models a SHIPPED locale catalogue, so its rows are
        // `LocaleSeed` origin.
        let keyed: Vec<KeyedAliasRow> = rows
            .iter()
            .map(|(s, c)| (s.clone(), c.clone(), None))
            .collect();
        self.apply_locale_overlay_keyed(environment_name, &keyed, AliasOrigin::LocaleSeed)
    }

    /// Overlay a locale's `(surface, canonical, command_template?)` rows ON TOP
    /// of this table's existing base entries, carrying the per-row parameter
    /// template through to the entry (CR-NR-112; CR-CH-072 localization
    /// Requirement 13.1, 13.8). This is the keyed entry point;
    /// [`AliasTable::apply_locale_overlay`] is the keyless shim that defaults the
    /// template to `None`. BOTH reuse the SAME scratch-copy collision machinery,
    /// so a within-environment collision leaves `self` UNCHANGED and the
    /// Identity_Base retained (Req 5.6, 11.10).
    ///
    /// An incoming row whose surface already resolves (case-insensitively) to the
    /// SAME canonical REPLACES that existing row in place -- so an authored row for
    /// a synthesised identity default UPDATES it (e.g. attaching a localisation
    /// key) rather than erroring (Req 11.4), and the replacement is positioned
    /// AHEAD so a canonical lookup prefers the overlaid row's display surface. A
    /// surface resolving to a DIFFERENT canonical is a collision.
    ///
    /// CR-NR-121 Stage B: `origin` records WHICH overlay layer produced these
    /// rows -- `AliasOrigin::LocaleSeed` for the shipped per-locale seed, or
    /// `AliasOrigin::UserOverride` for the user override store. It is stamped onto
    /// every row this call inserts or updates, so the menu-display rule can prefer
    /// a locale translation over a user shorthand (Req 5.4).
    ///
    /// Validates: command-resolution-model Requirement 5.1, 5.2, 5.3;
    /// localization Requirement 5.1, 5.5, 5.6, 10.3, 13.1, 11.4, 11.10
    // Production caller chain: the on-disk alias store's `apply_overlay_from_dir`
    // -> `AliasCatalogue::load_into_keyed` -> here (CR-NR-109), so no dead_code
    // allow is needed.
    pub(crate) fn apply_locale_overlay_keyed(
        &mut self,
        environment_name: &str,
        rows: &[KeyedAliasRow],
        origin: AliasOrigin,
    ) -> Result<(), AliasOverlayError> {
        // Validate the WHOLE overlay against a scratch copy first, so a collision
        // leaves `self` untouched (Identity_Base retained, Req 5.6). Only on full
        // success do we commit the scratch copy back.
        let mut scratch = self.entries.clone();
        for (surface, canonical, template) in rows {
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
                // (Req 11.4). Move it to the front and apply the new surface and
                // template (CR-NR-112), so a canonical lookup prefers the
                // overlaid display surface and the entry carries the template.
                let mut updated = scratch.remove(pos);
                updated.surface = surface_trimmed.to_string();
                updated.command_template = template;
                // CR-CH-072: an overlaid row (locale seed or user override) is a
                // localisation/customisation alias the Alias Editor lists.
                // CR-NR-121: stamp WHICH layer produced it so the menu-display
                // rule can tell a locale translation from a user shorthand.
                updated.origin = origin;
                scratch.insert(0, updated);
                continue;
            }
            scratch.insert(
                0,
                AliasEntry {
                    surface: surface_trimmed.to_string(),
                    canonical: canonical.clone(),
                    command_template: template,
                    origin,
                },
            );
        }
        self.entries = scratch;
        Ok(())
    }
}
