//! Theme file FORMAT VERSION metadata and `base` inheritance resolution
//! (CR-CH-056 Phase 4, Requirement 25).
//!
//! This module owns the small pieces of the versioned theme file format that
//! would otherwise push `loader.rs` / `serialiser.rs` over the 400-line limit:
//!
//! - the format version constant (`THEME_FORMAT_VERSION`) and the egui version
//!   the embedded `Style` blob was written against (`EMBEDDED_EGUI_VERSION`,
//!   Requirement 25.1);
//! - resolution of a declared `base` theme into the palette whose tokens an
//!   inheriting theme falls back to (Requirement 25.4), including cycle
//!   detection (a base chain that loops is broken with a WARN rather than
//!   recursing forever) and the unresolvable-base WARN + mode-default fallback
//!   (Requirement 25.5).

use crate::defaults;
use crate::mode::VisualMode;
use crate::palette::ThemePalette;

/// The current theme file format version written by the serialiser.
///
/// `version = 2` is the egui-native format (embedded `Style` sub-table + the
/// flat authoring groups). A file with NO `version` field, or `version = 1`, is
/// a legacy file and loads backward-compatibly (Requirement 25.2).
pub const THEME_FORMAT_VERSION: u32 = 2;

/// The legacy (pre-CR-CH-056) format version. Files with no `version` field are
/// treated as this version.
pub const LEGACY_FORMAT_VERSION: u32 = 1;

/// The egui version the embedded `Style` blob is written against (Requirement
/// 25.1). Recorded in the serialised file (`egui_version`) so a future egui
/// upgrade that changes the `Style` serde shape can be detected; the loader
/// stays version-tolerant regardless (Requirement 25.3).
pub const EMBEDDED_EGUI_VERSION: &str = "0.33";

/// The maximum `base` chain depth followed before the resolver assumes a cycle
/// and stops (Requirement 25.4 cycle guard). The built-in set is tiny, so any
/// chain deeper than this is a loop or a pathological user file.
const MAX_BASE_DEPTH: usize = 16;

/// Resolve the compiled BUILT-IN palette for a theme NAME, or `None` when the
/// name is not a built-in. Mirrors the shell's `builtin_palette_by_name` but
/// lives in `ff-theme` so the loader can resolve a built-in `base` without a
/// dependency on `ff-desktop`.
///
/// Validates: Requirement 25.4
pub fn builtin_palette_by_name(name: &str) -> Option<ThemePalette> {
    match name {
        "Default Dark" => Some(defaults::dark_palette()),
        "Default Light" => Some(defaults::light_palette()),
        "Default High Contrast" => Some(defaults::high_contrast_palette()),
        "Default Legacy" => Some(defaults::default_legacy_palette()),
        "Legacy Soft" => Some(defaults::legacy_soft_palette()),
        _ => None,
    }
}

/// A resolver that maps a theme NAME to its already-loaded palette, used to let
/// a `base` reference a previously-loaded USER theme in addition to the
/// built-ins. Returns `None` for an unknown name.
pub type UserBaseResolver<'a> = dyn Fn(&str) -> Option<ThemePalette> + 'a;

/// Resolve the palette an inheriting theme should fall back to for tokens it
/// does not define, given its declared `base` name.
///
/// Resolution order (Requirement 25.4):
/// 1. a BUILT-IN theme with that name;
/// 2. otherwise a previously-loaded USER theme via `user_resolver` (if any);
/// 3. otherwise `None` -> the caller emits a WARN and uses the mode default
///    (Requirement 25.5).
///
/// `MAX_BASE_DEPTH` bounds the chain: if the named base ITSELF declares a base,
/// the resolver does NOT recurse (the resolved palette is already a fully
/// default-filled `ThemePalette`, so its own inheritance was applied when it was
/// loaded). The depth guard exists so a hostile `user_resolver` that returns
/// palettes referencing each other cannot cause unbounded work; see
/// [`base_chain_has_cycle`].
///
/// Returns the resolved base palette, or `None` when the base cannot be found.
///
/// Validates: Requirement 25.4, 25.5
pub fn resolve_base_palette(
    base_name: &str,
    user_resolver: Option<&UserBaseResolver<'_>>,
) -> Option<ThemePalette> {
    if let Some(p) = builtin_palette_by_name(base_name) {
        return Some(p);
    }
    if let Some(resolver) = user_resolver {
        return resolver(base_name);
    }
    None
}

/// Detect whether following the `base` chain starting at `start` loops, using
/// `next_base` to look up each theme's declared base name. Returns `true` when a
/// cycle (or an over-long chain) is detected, in which case the caller breaks
/// the chain and WARNs rather than recursing forever (Requirement 25.4).
///
/// Built-in themes declare no base, so a chain that reaches a built-in or an
/// unresolved name terminates cleanly (`false`).
pub fn base_chain_has_cycle(start: &str, next_base: impl Fn(&str) -> Option<String>) -> bool {
    let mut seen: Vec<String> = Vec::new();
    let mut current = start.to_string();
    for _ in 0..MAX_BASE_DEPTH {
        if seen.iter().any(|s| s == &current) {
            return true;
        }
        seen.push(current.clone());
        match next_base(&current) {
            Some(next) => current = next,
            None => return false,
        }
    }
    // Exceeded the depth bound without terminating -> treat as a cycle.
    true
}

/// Emit a WARN-level log that a declared `base` could not be resolved, naming it
/// (Requirement 25.5). The load continues with the mode default.
pub fn warn_unresolvable_base(base_name: &str, mode: VisualMode) {
    ff_logging::log(
        ff_logging::LogLevel::Warn,
        module_path!(),
        &format!(
            "[theme] resolve base: '{base_name}' not found; falling back to the {} default",
            mode.section_name()
        ),
    );
}

/// Emit a WARN-level log that a `base` chain cycle was detected and broken
/// (Requirement 25.4). The load continues with the mode default for the
/// unresolved tokens.
pub fn warn_base_cycle(base_name: &str) {
    ff_logging::log(
        ff_logging::LogLevel::Warn,
        module_path!(),
        &format!(
            "[theme] resolve base: cycle detected in base chain starting at '{base_name}'; \
             breaking and falling back to defaults"
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_version_is_two() {
        // Validates: Requirement 25.1 -- the current format version is 2.
        assert_eq!(THEME_FORMAT_VERSION, 2);
        assert_eq!(LEGACY_FORMAT_VERSION, 1);
    }

    #[test]
    fn embedded_egui_version_is_recorded() {
        // Validates: Requirement 25.1 -- the egui version the Style blob was
        // written against is recorded as a non-empty constant.
        assert!(!EMBEDDED_EGUI_VERSION.is_empty());
    }

    #[test]
    fn builtin_base_resolves_to_compiled_palette() {
        // Validates: Requirement 25.4 -- a built-in base name resolves.
        let p = resolve_base_palette("Default Dark", None).expect("Default Dark resolves");
        assert_eq!(p.name, "Default Dark");
        assert!(resolve_base_palette("Default Legacy", None).is_some());
        assert!(resolve_base_palette("Legacy Soft", None).is_some());
    }

    #[test]
    fn unknown_base_without_resolver_is_none() {
        // Validates: Requirement 25.5 -- an unresolvable base yields None (the
        // caller then WARNs and uses the mode default).
        assert!(resolve_base_palette("NoSuchTheme", None).is_none());
    }

    #[test]
    fn user_base_resolves_via_resolver() {
        // Validates: Requirement 25.4 -- a previously-loaded user theme resolves
        // through the user resolver when it is not a built-in.
        let resolver = |name: &str| -> Option<ThemePalette> {
            if name == "My User Theme" {
                let mut p = defaults::dark_palette();
                p.name = "My User Theme".to_string();
                Some(p)
            } else {
                None
            }
        };
        let p = resolve_base_palette("My User Theme", Some(&resolver)).expect("user base resolves");
        assert_eq!(p.name, "My User Theme");
    }

    #[test]
    fn base_chain_cycle_is_detected() {
        // Validates: Requirement 25.4 -- a base chain A -> B -> A terminates with
        // a cycle verdict rather than looping forever.
        let next = |name: &str| -> Option<String> {
            match name {
                "A" => Some("B".to_string()),
                "B" => Some("A".to_string()),
                _ => None,
            }
        };
        assert!(base_chain_has_cycle("A", next));
    }

    #[test]
    fn acyclic_base_chain_is_not_a_cycle() {
        // Validates: Requirement 25.4 -- a terminating chain is not flagged.
        let next = |name: &str| -> Option<String> {
            match name {
                "Child" => Some("Default Dark".to_string()),
                _ => None, // Default Dark declares no base
            }
        };
        assert!(!base_chain_has_cycle("Child", next));
    }
}
