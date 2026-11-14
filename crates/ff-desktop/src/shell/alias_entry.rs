//! # Alias row types: `AliasOrigin` + `AliasEntry` (CR-NR-121 Stage B split)
//!
//! Split out of `environment.rs` to keep that file under the 400-line source
//! limit (rust-standards.md / rust-ffwb-layout.md) once CR-NR-121 Stage B added
//! the `AliasOrigin` discriminator and the menu-display rule. This module holds
//! only the per-row DATA types; the `AliasTable` and its overlay/resolution logic
//! stay in `environment.rs` and reach these fields via the crate-visible struct.
//! No behaviour change: the types are re-exported from `environment.rs` so every
//! existing `super::environment::{AliasEntry, AliasOrigin}` path is unchanged.

/// CR-NR-121 Stage B: the ORIGIN of an alias row -- WHICH overlay layer produced
/// it. Derived at LOAD time (Option 1, the owner-approved choice in
/// `docs/specs/command-resolution-model/design.md`), so no new persisted TOML key
/// is needed: the overlay apply path already KNOWS which layer it is applying.
///
/// This replaces the former bare `is_overlay: bool`, which could not tell a
/// locale-translation overlay apart from a user-shorthand overlay (both were
/// `is_overlay = true`). The discriminator lets the menu-display rule show the
/// canonical root overridden ONLY by a LOCALE translation and NEVER by a USER
/// shorthand (Req 5.4), and (in a later stage) lets locale surfaces resolve at
/// ROOT precedence while user shorthands resolve LAST (Req 5.2, 5.3).
///
/// Validates: command-resolution-model Requirement 5.1, 5.5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AliasOrigin {
    /// A built-in BASE row: the hard-coded English FFEDIT verb aliases
    /// (X -> EXCLUDE, INCLUDE -> SHOW) and the synthesised identity roots. NOT a
    /// localisation / user-customisation alias; never shown in the Alias Editor.
    Base,
    /// A row from the shipped per-locale SEED overlay
    /// (`i18n/<locale>/aliases/<ENV>.toml`): a LOCALE translation. Drives the
    /// menu display surface (Req 5.4) and resolves at ROOT precedence (Req 5.2).
    LocaleSeed,
    /// A row from the USER override store
    /// (`<User_Data_Dir>/.../aliases/<locale>/<ENV>.toml`): a user SHORTHAND. Does
    /// NOT drive the menu display surface (Req 5.4) and resolves LAST (Req 5.3).
    UserOverride,
}

/// One ALIAS TABLE entry: a surface form (the In_Row_Surface) and the canonical
/// (root) verb it resolves to, plus an OPTIONAL parameter template.
///
/// CR-CH-072 (localization Requirement 13.1): the command-surface
/// `localisation_key -> verb-*` indirection is REMOVED. The surface string stored
/// DIRECTLY on the row is the authoritative displayed/typeable text (the
/// In_Row_Surface), resolved by a SINGLE lookup (surface -> root). There is no
/// `t(key)` indirection for command surfaces; DESCRIPTION/TITLE `.ftl`
/// localisation is unchanged and lives elsewhere. The canonical verb is the
/// dispatched/persisted identity; the surface is a DISPLAY/typeable concern only.
///
/// Validates: localization Requirement 13.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AliasEntry {
    /// UPPERCASED typeable/displayed surface form (the In_Row_Surface, the
    /// authoritative display text under CR-CH-072).
    pub(crate) surface: String,
    /// The root (canonical) verb -- the dispatched/persisted identity.
    pub(crate) canonical: String,
    /// CR-NR-112: OPTIONAL parameter template (e.g. `FFWB.COBOL.MASTER(&1)`).
    /// `Some`+non-empty -> invoking the alias expands `canonical` + ` ` + the
    /// substituted template into the ROOT command (Req 14.1/14.2); `None`/empty
    /// is a pure verb alias, byte-identical to today (Req 14.6). Pre-resolution
    /// form only; the ROOT stays the dispatched/persisted identity (Req 19.4).
    /// RETAINED by CR-CH-072 (localization Requirement 13.8).
    ///
    /// Validates: command-semantics Requirement 14.1
    pub(crate) command_template: Option<String>,
    /// CR-NR-121 Stage B: the ORIGIN layer that produced this row -- `Base`
    /// (hard-coded English aliases + synthesised identity roots), `LocaleSeed`
    /// (the shipped `i18n/<locale>/aliases/<ENV>.toml` overlay), or
    /// `UserOverride` (the user override store). REPLACES the former bare
    /// `is_overlay: bool`: the old flag was `true` for BOTH overlay layers and so
    /// could not distinguish a locale translation from a user shorthand
    /// (CR-CH-072). ONLY non-`Base` rows are the localisation / user-customisation
    /// aliases the Alias Editor shows and edits (Req 6.1) -- see
    /// [`AliasEntry::is_overlay`]. Resolution ignores this for now; the menu
    /// display rule (Req 5.4) uses it to prefer a `LocaleSeed` surface over a
    /// `UserOverride` one.
    ///
    /// Validates: command-resolution-model Requirement 5.1;
    /// command-configurator Requirement 6.1
    pub(crate) origin: AliasOrigin,
}

impl AliasEntry {
    /// `true` when this row came from a per-locale OVERLAY (the shipped seed OR
    /// the user override store) rather than a built-in base row. Preserves the
    /// former `is_overlay` field semantics for the Alias Editor listing
    /// (CR-CH-072 Req 6.1): a `Base` row is NOT listed; a `LocaleSeed` or
    /// `UserOverride` row IS. Derived from [`AliasEntry::origin`].
    ///
    /// Validates: command-configurator Requirement 6.1
    pub(crate) fn is_overlay(&self) -> bool {
        !matches!(self.origin, AliasOrigin::Base)
    }

    /// The DISPLAYED (and typeable) surface for this entry: the In_Row_Surface
    /// stored directly on the row.
    ///
    /// CR-CH-072 (localization Requirement 13.1): there is exactly ONE lookup
    /// (surface -> root); the surface is NOT resolved through the
    /// Catalogue_Lookup_Seam (`t(key)` / `verb-*`). The surface string IS the
    /// authoritative display text.
    ///
    /// Validates: localization Requirement 13.1
    // Consumed by `display_surface_for_canonical` (the menu-kind render reader,
    // now called by `menu_option_command_surfaces`).
    pub(crate) fn display_surface(&self) -> String {
        self.surface.clone()
    }
}
