//! Theme TOML loading, inheritance resolution, and validation.
//!
//! The loader reads theme files via the configuration system, validates
//! colour formats and font sizes, resolves inheritance chains, and
//! builds the final `ThemePalette`.

use crate::chrome_style::ChromeStyle;
use crate::defaults;
use crate::element::ElementColourMap;
use crate::error::ThemeError;
use crate::loader_parse::{
    parse_decoration_colours, parse_design_tokens, parse_editor_colours, parse_file_tree_colours,
    parse_font_config, parse_gutter_colours, parse_indicator_colours, parse_style_slots,
    parse_syntax_colours, parse_tab_bar_colours, parse_ui_colours,
};
use crate::mode::VisualMode;
use crate::palette::ThemePalette;

/// Load a theme palette from a TOML string.
///
/// Missing tokens inherit from the built-in default for the specified mode.
/// Invalid values are logged and replaced with defaults.
///
/// # Errors
///
/// Returns `ThemeError::ParseError` if the TOML is completely invalid syntax.
pub fn load_from_toml(toml_str: &str, mode: VisualMode) -> Result<ThemePalette, ThemeError> {
    load_from_toml_with_base_resolver(toml_str, mode, None)
}

/// Load a theme palette, resolving a declared `base` against the built-in themes
/// AND an optional `user_resolver` for previously-loaded USER themes
/// (CR-CH-056 Requirement 25.4).
///
/// This is the full loader; [`load_from_toml`] is the convenience wrapper that
/// resolves only built-in bases. The `version` field is read for format
/// branching (Requirement 25.2) and the embedded egui `Style` sub-table (if any)
/// is parsed version-tolerantly (Requirement 25.3); the flat authoring groups
/// remain the authoritative chrome source (Phase-1 derive-on-load design), so
/// the chrome `Style` is always re-derived from the resolved groups.
///
/// # Errors
///
/// Returns `ThemeError::ParseError` if the TOML is completely invalid syntax.
pub fn load_from_toml_with_base_resolver(
    toml_str: &str,
    mode: VisualMode,
    user_resolver: Option<&crate::format_version::UserBaseResolver<'_>>,
) -> Result<ThemePalette, ThemeError> {
    let table: toml::Table =
        toml_str
            .parse()
            .map_err(|e: toml::de::Error| ThemeError::ParseError {
                path: "<string>".to_string(),
                detail: e.to_string(),
            })?;

    // The effective mode is the file's `mode` field when present (B063), else
    // the mode passed by the caller (backward compatible with older files that
    // did not persist a mode). Using it for the default palette + style slots +
    // the final `mode` keeps token fallbacks and the visual mode consistent, so
    // a user theme saved from Legacy reloads as Legacy.
    let mode = table
        .get("mode")
        .and_then(|v| v.as_str())
        .and_then(VisualMode::from_str_loose)
        .unwrap_or(mode);

    // Format version: absent or 1 => legacy layout; 2 => egui-native. Both load
    // through the SAME flat-group + default-fill path, so an old (v1) file never
    // fails -- the version only records provenance and gates the embedded-Style
    // read below (Requirement 25.1, 25.2).
    let _version = table
        .get("version")
        .and_then(|v| v.as_integer())
        .map(|v| v as u32)
        .unwrap_or(crate::format_version::LEGACY_FORMAT_VERSION);

    // Resolve `base` (Requirement 25.4): the per-token fallback becomes the named
    // base palette when it resolves, else the built-in mode default. A base-chain
    // cycle or an unresolvable base WARNs and falls back to the mode default
    // (Requirement 25.5) WITHOUT failing the load.
    let default = resolve_fallback_palette(&table, mode, user_resolver);

    let name = table
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&default.name)
        .to_string();

    let editor = parse_editor_colours(&table, &default.editor);
    let syntax = parse_syntax_colours(&table, &default.syntax);
    let file_tree = parse_file_tree_colours(&table, &default.file_tree);
    let tab_bar = parse_tab_bar_colours(&table, &default.tab_bar);
    // The gutter group is still parsed from the `[chrome]` TOML section in Phase
    // 1 (the file format is unchanged until Phase 4); only the in-memory field
    // was renamed `chrome` -> `gutter` (CR-CH-056 Req 23.3).
    let gutter = parse_gutter_colours(&table, &default.gutter);
    let decorations = parse_decoration_colours(&table, &default.decorations);
    let indicators = parse_indicator_colours(&table, &default.indicators);
    let ui = parse_ui_colours(&table, &default.ui);
    let fonts = parse_font_config(&table);
    let design = parse_design_tokens(&table);
    let style_slots = parse_style_slots(&table, mode);

    // CR-CH-056 Req 23.1/23.3: derive the egui-native chrome layer from the
    // parsed chrome groups + design + mode (Phase 1 keeps the flat groups as the
    // authoritative authoring surface, so the chrome layer stays consistent with
    // the file). The DERIVED chrome is authoritative.
    let derived_chrome = ChromeStyle::from_palette_parts(&ui, &tab_bar, &editor, &design, mode);

    // CR-CH-056 Req 25.3: when the file embeds a `[chrome_style]` sub-table (a
    // v2 native file), parse it VERSION-TOLERANTLY -- missing egui fields take
    // the derived chrome's values and extra/unknown fields are ignored -- so a
    // `Style` written by a different egui version still loads WITHOUT failing.
    // The FLAT GROUPS remain authoritative and the chrome is DERIVED from them
    // (the Phase-1 design, kept so built-in appearance never changes); the
    // embedded read is validation-only + forward-compat and NEVER overrides the
    // derived chrome, so the derived chrome is returned.
    read_embedded_chrome_tolerant(&table, &derived_chrome);
    let chrome_style = derived_chrome;

    Ok(ThemePalette {
        name,
        mode,
        editor,
        syntax,
        file_tree,
        tab_bar,
        gutter,
        decorations,
        indicators,
        ui,
        style_slots,
        fonts,
        design,
        elements: ElementColourMap::new(),
        chrome_style,
    })
}

/// Read an embedded `[chrome_style]` sub-table VERSION-TOLERANTLY (CR-CH-056
/// Req 25.3) when present. Missing egui fields take the derived chrome's values
/// (via a deep merge over the derived chrome's own serialised form) and
/// extra/unknown fields are ignored. The load NEVER fails on a malformed or
/// foreign-version embed: on any error a WARN is logged and the derived chrome
/// is used. Returns `true` when an embed was present and parsed, `false`
/// otherwise (used only by tests to confirm tolerance).
fn read_embedded_chrome_tolerant(table: &toml::Table, derived: &ChromeStyle) -> bool {
    let embedded = match table.get("chrome_style").and_then(|v| v.as_table()) {
        Some(t) => t,
        None => return false,
    };

    // Serialise the derived chrome to a TOML table, deep-merge the embedded
    // fields over it (so any MISSING embedded field keeps the derived value),
    // then deserialise. Unknown fields in `embedded` are ignored by serde.
    let base_value = match toml::Value::try_from(derived) {
        Ok(toml::Value::Table(t)) => t,
        _ => return true, // derived should always serialise; be defensive
    };
    let mut merged = base_value;
    deep_merge_table(&mut merged, embedded);

    match toml::Value::Table(merged).try_into::<ChromeStyle>() {
        Ok(_parsed) => true,
        Err(e) => {
            ff_logging::log(
                ff_logging::LogLevel::Warn,
                module_path!(),
                &format!(
                    "[theme] load: embedded chrome Style could not be parsed ({e}); \
                     using the derived chrome instead"
                ),
            );
            true
        }
    }
}

/// Recursively merge `overlay` into `base`: scalar/array keys from `overlay`
/// replace those in `base`; nested tables are merged key-by-key. Keys present
/// only in `base` are kept (fills missing fields); keys present only in
/// `overlay` are added (tolerated, later ignored by serde if unknown).
fn deep_merge_table(base: &mut toml::Table, overlay: &toml::Table) {
    for (key, ov) in overlay {
        match (base.get_mut(key), ov) {
            (Some(toml::Value::Table(bt)), toml::Value::Table(ot)) => deep_merge_table(bt, ot),
            _ => {
                base.insert(key.clone(), ov.clone());
            }
        }
    }
}

/// Resolve the per-token FALLBACK palette for a theme (CR-CH-056 Req 25.4/25.5).
///
/// - No `base` field -> the built-in mode default (the historical behaviour).
/// - `base = "<name>"` that resolves (built-in or via `user_resolver`) -> that
///   base palette, so tokens absent from the file inherit the base's values.
/// - `base` that cannot be resolved -> WARN + the mode default (Req 25.5).
/// - a `base` chain that cycles -> WARN + the mode default (Req 25.4 guard; the
///   cycle is actually detected and broken in [`load_from_sources`], which is
///   the entry point that can see the whole chain of raw user files).
fn resolve_fallback_palette(
    table: &toml::Table,
    mode: VisualMode,
    user_resolver: Option<&crate::format_version::UserBaseResolver<'_>>,
) -> ThemePalette {
    let mode_default = defaults::default_palette_for_mode(mode);
    let base_name = match table.get("base").and_then(|v| v.as_str()) {
        Some(b) if !b.trim().is_empty() => b.trim(),
        _ => return mode_default,
    };

    match crate::format_version::resolve_base_palette(base_name, user_resolver) {
        Some(base) => base,
        None => {
            crate::format_version::warn_unresolvable_base(base_name, mode);
            mode_default
        }
    }
}

/// Load a theme by NAME from a map of raw theme-file sources, resolving its
/// `base` chain against the SAME source map (and the built-ins) with CYCLE
/// DETECTION (CR-CH-056 Requirement 25.4).
///
/// `sources` maps a theme name to its raw TOML. The named theme is loaded; when
/// it declares `base = "<other>"`, the base is resolved from `sources` (or a
/// built-in) and its tokens become the fallback. A `base` chain that loops
/// (A -> B -> A) is DETECTED and BROKEN with a WARN, falling back to the mode
/// default rather than recursing forever (Requirement 25.4). An unresolvable
/// base WARNs and falls back (Requirement 25.5).
///
/// # Errors
///
/// Returns `ThemeError::ParseError` if the named source is invalid TOML.
pub fn load_from_sources(
    name: &str,
    sources: &std::collections::HashMap<String, String>,
    mode: VisualMode,
) -> Result<ThemePalette, ThemeError> {
    // Walk the base chain first, collecting names, to detect a cycle up front.
    let cyclic = crate::format_version::base_chain_has_cycle(name, |n| {
        sources.get(n).and_then(|src| {
            src.parse::<toml::Table>()
                .ok()
                .and_then(|t| t.get("base").and_then(|v| v.as_str()).map(str::to_string))
        })
    });

    let source = sources.get(name).ok_or_else(|| ThemeError::FileNotFound {
        path: name.to_string(),
    })?;

    if cyclic {
        crate::format_version::warn_base_cycle(name);
        // Break the cycle: load the theme WITHOUT resolving its base (base
        // tokens fall back to the mode default instead of chasing the loop).
        let stripped = strip_base_field(source);
        return load_from_toml(&stripped, mode);
    }

    // Acyclic: resolve a user base recursively through this same function.
    let resolver = |base_name: &str| -> Option<ThemePalette> {
        sources
            .get(base_name)
            .and_then(|_| load_from_sources(base_name, sources, mode).ok())
    };
    load_from_toml_with_base_resolver(source, mode, Some(&resolver))
}

/// Return `source` with any top-level `base = "..."` line removed, so a theme
/// whose base chain was found to cycle loads without re-entering the loop.
fn strip_base_field(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("base"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colour::ColourRGBA;

    #[test]
    fn all_ui_colour_tokens_overridable_via_toml() {
        // Validates: Requirement 14.1 -- every colour token individually overridable
        let toml = r##"
[ui]
menu_bar_foreground = "#FF0000"
primary_menu_background = "#0000FF"
panel_background = "#111111"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        assert_eq!(palette.ui.menu_bar_fg, ColourRGBA::rgb(255, 0, 0));
        assert_eq!(palette.ui.primary_menu_bg, ColourRGBA::rgb(0, 0, 255));
        assert_eq!(palette.ui.panel_bg, ColourRGBA::rgb(0x11, 0x11, 0x11));
    }

    #[test]
    fn invalid_colour_in_user_theme_falls_back_to_default() {
        // Validates: Requirement 14.8 -- invalid colour uses fallback, rest loads fine
        let toml = r##"
[ui]
menu_bar_foreground = "not-a-colour"
panel_background = "#ABCDEF"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        let default = defaults::dark_palette();
        // Invalid token falls back to default
        assert_eq!(palette.ui.menu_bar_fg, default.ui.menu_bar_fg);
        // Valid token is applied
        assert_eq!(palette.ui.panel_bg, ColourRGBA::rgb(0xAB, 0xCD, 0xEF));
    }

    #[test]
    fn base_inheritance_fills_missing_tokens() {
        // Validates: Requirement 14.4, 14.5 -- omitted tokens inherit from base/default
        // A theme that only overrides one editor token should inherit all others
        let toml = r##"
name = "Partial"

[editor]
background = "#FF0000"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        let default = defaults::dark_palette();
        // Overridden token is applied
        assert_eq!(palette.editor.background, ColourRGBA::rgb(255, 0, 0));
        // All other tokens inherit from the default
        assert_eq!(palette.editor.foreground, default.editor.foreground);
        assert_eq!(palette.syntax.keyword, default.syntax.keyword);
        assert_eq!(palette.ui.panel_bg, default.ui.panel_bg);
    }

    #[test]
    fn load_empty_toml_returns_defaults() {
        // Validates: Requirement 1.6
        let palette = load_from_toml("", VisualMode::Dark).unwrap();
        let default = defaults::dark_palette();
        assert_eq!(palette.editor.background, default.editor.background);
        assert_eq!(palette.syntax.keyword, default.syntax.keyword);
    }

    #[test]
    fn load_partial_toml_overrides_specified_values() {
        // Validates: Requirement 1.6
        let toml = r##"
name = "Custom"

[editor]
background = "#FF0000"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        assert_eq!(palette.name, "Custom");
        assert_eq!(palette.editor.background, ColourRGBA::rgb(255, 0, 0));
        // Unspecified values should be defaults
        let default = defaults::dark_palette();
        assert_eq!(palette.editor.foreground, default.editor.foreground);
    }

    #[test]
    fn load_invalid_colour_uses_default() {
        // Validates: Requirement 1.5
        let toml = r##"
[editor]
background = "not-a-colour"
foreground = "#CDD6F4"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        let default = defaults::dark_palette();
        // Invalid colour should fall back to default
        assert_eq!(palette.editor.background, default.editor.background);
        // Valid colour should be parsed
        assert_eq!(palette.editor.foreground, ColourRGBA::rgb(0xCD, 0xD6, 0xF4));
    }

    #[test]
    fn load_invalid_toml_syntax_returns_error() {
        // Validates: Requirement 1.4
        let result = load_from_toml("this is not valid { toml [", VisualMode::Dark);
        assert!(result.is_err());
    }

    #[test]
    fn load_font_config_from_toml() {
        // Validates: Requirement 4.1, 4.4
        let toml = r##"
[font.monospace]
families = ["JetBrains Mono", "Fira Code", "Consolas"]
size = 16.0

[font.proportional]
families = ["Inter", "Segoe UI"]
size = 13.0
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        assert_eq!(palette.fonts.monospace.families.len(), 3);
        assert_eq!(palette.fonts.monospace.base_size_pt, 16.0);
        assert_eq!(palette.fonts.proportional.families.len(), 2);
    }

    #[test]
    fn load_design_tokens_from_toml() {
        // Validates: Requirement 6.5
        let toml = r##"
[design.spacing]
xs = 4.0
sm = 8.0
md = 16.0
lg = 24.0
xl = 48.0
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        assert_eq!(palette.design.spacing.xs, 4.0);
        assert_eq!(palette.design.spacing.sm, 8.0);
        assert_eq!(palette.design.spacing.md, 16.0);
    }

    #[test]
    fn v1_file_without_version_loads_with_default_fill() {
        // Validates: Requirement 25.2 -- a pre-version (no `version` key) file is
        // treated as v1 and loads backward-compatibly with per-token default-fill
        // and does NOT fail.
        let toml = r##"
name = "Legacy v1 file"

[editor]
background = "#010203"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).expect("v1 file must not fail");
        assert_eq!(palette.name, "Legacy v1 file");
        assert_eq!(palette.editor.background, ColourRGBA::rgb(1, 2, 3));
        // New/absent tokens are default-filled from the mode default.
        let default = defaults::dark_palette();
        assert_eq!(palette.editor.foreground, default.editor.foreground);
    }

    #[test]
    fn v2_file_with_embedded_style_loads() {
        // Validates: Requirement 25.2, 25.3 -- a v2 file (version = 2 + embedded
        // [chrome_style]) loads successfully.
        let toml_str = crate::serialiser::serialise(&defaults::dark_palette());
        let table: toml::Table = toml_str.parse().expect("valid TOML");
        assert_eq!(table.get("version").and_then(|v| v.as_integer()), Some(2));
        assert!(
            table
                .get("chrome_style")
                .map(|v| v.is_table())
                .unwrap_or(false),
            "v2 file embeds a [chrome_style] sub-table"
        );
        let palette = load_from_toml(&toml_str, VisualMode::Dark).expect("v2 file loads");
        assert_eq!(palette.name, "Default Dark");
    }

    #[test]
    fn embedded_style_with_missing_field_loads_defaulted() {
        // Validates: Requirement 25.3 -- an embedded Style MISSING egui fields
        // loads (the missing fields take the derived chrome's values) WITHOUT
        // failing the load.
        let toml = r##"
version = 2
name = "Sparse Chrome"

[editor]
background = "#101010"

[chrome_style]
title_band_bg = "#123456"

[chrome_style.style]
# intentionally almost-empty: nearly every egui Style field is MISSING
"##;
        let palette =
            load_from_toml(toml, VisualMode::Dark).expect("missing Style fields must not fail");
        assert_eq!(palette.editor.background, ColourRGBA::rgb(0x10, 0x10, 0x10));
    }

    #[test]
    fn embedded_style_with_unknown_field_is_ignored() {
        // Validates: Requirement 25.3 -- an embedded Style with an EXTRA /
        // unrecognised field (e.g. from a different egui version) is ignored and
        // the load succeeds.
        let toml = r##"
version = 2
name = "Future Chrome"

[editor]
background = "#202020"

[chrome_style]
title_band_bg = "#654321"
some_future_field_that_does_not_exist = "whatever"

[chrome_style.style]
future_only_egui_field = 42
"##;
        let palette =
            load_from_toml(toml, VisualMode::Dark).expect("unknown Style fields must not fail");
        assert_eq!(palette.editor.background, ColourRGBA::rgb(0x20, 0x20, 0x20));
    }

    #[test]
    fn base_resolves_inherited_tokens_from_named_base_not_bare_default() {
        // Validates: Requirement 25.4 -- a child theme with base = "Default Dark"
        // and only one overridden token inherits the REST from Default Dark.
        // Default Dark == the Dark mode default here, so to prove inheritance
        // comes from the BASE (not the bare mode default) we base on a built-in
        // whose values differ from the loader's passed mode default: use
        // "Default Legacy" while passing VisualMode::Dark.
        let toml = r##"
name = "Child Of Legacy"
base = "Default Legacy"

[editor]
background = "#000001"
"##;
        // Pass Dark as the mode arg; the file declares no `mode`, so the loader
        // uses Dark. Without base resolution, inherited tokens would come from the
        // Dark default. With base resolution they must come from Default Legacy.
        let palette = load_from_toml(toml, VisualMode::Dark).expect("loads");
        let legacy = defaults::default_legacy_palette();
        // Overridden token applied.
        assert_eq!(palette.editor.background, ColourRGBA::rgb(0, 0, 1));
        // Inherited token comes from the BASE (Legacy), not the Dark default.
        assert_eq!(
            palette.syntax.keyword, legacy.syntax.keyword,
            "inherited token must come from the resolved base theme"
        );
        assert_eq!(palette.editor.foreground, legacy.editor.foreground);
    }

    #[test]
    fn unresolvable_base_warns_and_falls_back_without_error() {
        // Validates: Requirement 25.5 -- a base naming a theme that cannot be
        // found WARNs and falls back to the mode default WITHOUT failing.
        let toml = r##"
name = "Orphan"
base = "NoSuchTheme"

[editor]
background = "#030303"
"##;
        let palette = load_from_toml(toml, VisualMode::Dark)
            .expect("unresolvable base must not fail the load");
        let default = defaults::dark_palette();
        assert_eq!(palette.editor.background, ColourRGBA::rgb(3, 3, 3));
        // Unspecified tokens fall back to the mode default.
        assert_eq!(palette.editor.foreground, default.editor.foreground);
    }

    #[test]
    fn base_cycle_terminates_with_warn_not_hang() {
        // Validates: Requirement 25.4 -- a base chain A -> B -> A is detected and
        // broken (does NOT infinite-loop); the load terminates and falls back to
        // the mode default for inherited tokens.
        let mut sources = std::collections::HashMap::new();
        sources.insert(
            "A".to_string(),
            "name = \"A\"\nbase = \"B\"\n[editor]\nbackground = \"#0A0A0A\"\n".to_string(),
        );
        sources.insert(
            "B".to_string(),
            "name = \"B\"\nbase = \"A\"\n[editor]\nforeground = \"#0B0B0B\"\n".to_string(),
        );
        // This must return (not hang) and must succeed.
        let palette = load_from_sources("A", &sources, VisualMode::Dark)
            .expect("cyclic base must still load");
        assert_eq!(palette.name, "A");
        assert_eq!(palette.editor.background, ColourRGBA::rgb(0x0A, 0x0A, 0x0A));
    }

    #[test]
    fn base_resolves_through_user_sources_when_acyclic() {
        // Validates: Requirement 25.4 -- a user theme can base on ANOTHER user
        // theme (acyclic), inheriting its tokens.
        let mut sources = std::collections::HashMap::new();
        sources.insert(
            "Parent".to_string(),
            "name = \"Parent\"\n[editor]\nforeground = \"#AABBCC\"\n".to_string(),
        );
        sources.insert(
            "Kid".to_string(),
            "name = \"Kid\"\nbase = \"Parent\"\n[editor]\nbackground = \"#112233\"\n".to_string(),
        );
        let palette = load_from_sources("Kid", &sources, VisualMode::Dark).expect("loads");
        assert_eq!(palette.editor.background, ColourRGBA::rgb(0x11, 0x22, 0x33));
        // Inherited from the Parent user theme.
        assert_eq!(palette.editor.foreground, ColourRGBA::rgb(0xAA, 0xBB, 0xCC));
    }

    #[test]
    fn load_style_slots_from_toml() {
        // Validates: Requirement 3.1, 3.2
        let toml = r##"
[style_slots.50]
foreground = "#FF0000"
bold = true
italic = true
"##;
        let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
        let slot = palette.style_slots.get(50);
        assert_eq!(slot.foreground, ColourRGBA::rgb(255, 0, 0));
        assert!(slot.bold);
        assert!(slot.italic);
    }
}
