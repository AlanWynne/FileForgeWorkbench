//! Property-based tests for ff-theme crate.
//!
//! These tests verify universal properties that must hold across all inputs.

use proptest::prelude::*;

use ff_theme::colour::ColourRGBA;
use ff_theme::design_tokens::DesignTokens;
use ff_theme::element::{Element, ElementColourMap};
use ff_theme::font::{
    clamp_font_size, FontConfig, FontStack, ZoomLevel, MAX_EFFECTIVE_SIZE_PT, MAX_FONT_SIZE_PT,
    MIN_EFFECTIVE_SIZE_PT, MIN_FONT_SIZE_PT,
};
use ff_theme::mode::VisualMode;
use ff_theme::style_slot::{CaseTransform, StyleSlot, StyleSlotTable};
use ff_theme::{defaults, loader, serialiser};

// ─── Property 1: Colour Hex Round-Trip Correctness ──────────────────────────
// **Validates: Requirements 2.9, 9.5**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// For any valid ColourRGBA, serialising to hex and parsing back produces
    /// an identical value. Opaque colours serialize as #RRGGBB, translucent as #RRGGBBAA.
    #[test]
    fn colour_hex_round_trip_correctness(r in 0u8..=255, g in 0u8..=255, b in 0u8..=255, a in 0u8..=255) {
        // Feature: ff-theme, Property 1: colour hex round-trip correctness
        let colour = ColourRGBA::rgba(r, g, b, a);
        let hex = colour.to_hex();
        let parsed = ColourRGBA::from_hex(&hex).unwrap();

        // Round-trip produces identical colour
        prop_assert_eq!(colour, parsed);

        // Format correctness
        if a == 255 {
            // Opaque → 7 chars (#RRGGBB)
            prop_assert_eq!(hex.len(), 7, "Opaque colour should produce 7-char hex, got: {}", hex);
        } else {
            // Translucent → 9 chars (#RRGGBBAA)
            prop_assert_eq!(hex.len(), 9, "Translucent colour should produce 9-char hex, got: {}", hex);
        }
    }
}

// ─── Property 2: Theme Serialisation Round-Trip Correctness ─────────────────
// **Validates: Requirements 9.1, 9.2**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// For any valid ThemePalette, serialising to TOML and parsing back produces
    /// equivalent colour values for all palette groups.
    #[test]
    fn theme_serialisation_round_trip_correctness(mode in prop_oneof![
        Just(VisualMode::Dark),
        Just(VisualMode::Light),
        Just(VisualMode::HighContrast),
    ]) {
        // Feature: ff-theme, Property 2: theme serialisation round-trip correctness
        let original = defaults::default_palette_for_mode(mode);
        let toml_str = serialiser::serialise(&original);
        let round_tripped = loader::load_from_toml(&toml_str, mode).unwrap();

        // All colour groups should round-trip correctly
        prop_assert_eq!(&original.editor, &round_tripped.editor);
        prop_assert_eq!(&original.syntax, &round_tripped.syntax);
        prop_assert_eq!(&original.file_tree, &round_tripped.file_tree);
        prop_assert_eq!(&original.tab_bar, &round_tripped.tab_bar);
        prop_assert_eq!(&original.chrome, &round_tripped.chrome);
        prop_assert_eq!(&original.decorations, &round_tripped.decorations);
        prop_assert_eq!(&original.ui, &round_tripped.ui);
        prop_assert_eq!(&original.indicators.find_match, &round_tripped.indicators.find_match);
        prop_assert_eq!(&original.indicators.brace_match, &round_tripped.indicators.brace_match);
    }
}

// ─── Property 3: Font Size Clamping Correctness ─────────────────────────────
// **Validates: Requirements 4.6**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// For any configured font size, the validated value is always within [6.0, 72.0].
    #[test]
    fn font_size_clamping_correctness(size in -100.0f32..200.0) {
        // Feature: ff-theme, Property 3: font size clamping correctness
        let validated = clamp_font_size(size);

        // Result is always within valid range
        prop_assert!(validated >= MIN_FONT_SIZE_PT, "Validated size {} is below minimum {}", validated, MIN_FONT_SIZE_PT);
        prop_assert!(validated <= MAX_FONT_SIZE_PT, "Validated size {} is above maximum {}", validated, MAX_FONT_SIZE_PT);

        // Result equals the expected clamping
        let expected = size.clamp(MIN_FONT_SIZE_PT, MAX_FONT_SIZE_PT);
        prop_assert!((validated - expected).abs() < f32::EPSILON,
            "Validated {} != expected clamped {}", validated, expected);
    }
}

// ─── Property 4: Zoom Level Effective Size Calculation Correctness ──────────
// **Validates: Requirements 4.7, 4.8**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// For any base font size and zoom level, the effective size equals
    /// (base + zoom).clamp(2.0, 128.0) and the zoom level is never modified.
    #[test]
    fn zoom_level_effective_size_correctness(
        base_size in 6.0f32..72.0,
        zoom_level in -100i32..100
    ) {
        // Feature: ff-theme, Property 4: zoom level effective size calculation correctness
        let zoom = ZoomLevel::new(zoom_level);
        let effective = zoom.effective_size(base_size);

        // Effective size is clamped to [2.0, 128.0]
        prop_assert!(effective >= MIN_EFFECTIVE_SIZE_PT,
            "Effective size {} is below minimum {}", effective, MIN_EFFECTIVE_SIZE_PT);
        prop_assert!(effective <= MAX_EFFECTIVE_SIZE_PT,
            "Effective size {} is above maximum {}", effective, MAX_EFFECTIVE_SIZE_PT);

        // Effective size equals expected calculation
        let expected = (base_size + zoom_level as f32).clamp(MIN_EFFECTIVE_SIZE_PT, MAX_EFFECTIVE_SIZE_PT);
        prop_assert!((effective - expected).abs() < f32::EPSILON,
            "Effective {} != expected {}", effective, expected);

        // Zoom level is not modified
        prop_assert_eq!(zoom.level(), zoom_level,
            "Zoom level was modified from {} to {}", zoom_level, zoom.level());
    }
}

// ─── Property 5: Style Slot Inheritance Correctness ─────────────────────────
// **Validates: Requirements 3.4**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// For any StyleSlotTable with a defined Default slot, undefined slots
    /// return Default slot values. Explicitly defined slots return their own values.
    #[test]
    fn style_slot_inheritance_correctness(
        default_fg_r in 0u8..=255,
        default_fg_g in 0u8..=255,
        default_fg_b in 0u8..=255,
        default_bold in proptest::bool::ANY,
        defined_index in 0u8..=255,
        custom_fg_r in 0u8..=255,
        query_index in 0u8..=255,
    ) {
        // Feature: ff-theme, Property 5: style slot inheritance correctness
        // Skip if defined_index is the default slot itself
        prop_assume!(defined_index != 32);

        let default_slot = StyleSlot {
            foreground: ColourRGBA::rgb(default_fg_r, default_fg_g, default_fg_b),
            background: ColourRGBA::rgb(0, 0, 0),
            font_family: None,
            bold: default_bold,
            italic: false,
            underline: false,
            case_transform: CaseTransform::None,
        };

        let mut table = StyleSlotTable::new(default_slot.clone());

        let custom_slot = StyleSlot {
            foreground: ColourRGBA::rgb(custom_fg_r, 0, 0),
            background: ColourRGBA::rgb(255, 255, 255),
            font_family: Some("Custom".to_string()),
            bold: !default_bold,
            italic: true,
            underline: true,
            case_transform: CaseTransform::Upper,
        };
        table.set(defined_index, custom_slot.clone());

        let result = table.get(query_index);

        if query_index == defined_index {
            // Defined slot returns its own values
            prop_assert_eq!(result, &custom_slot);
        } else if query_index == 32 {
            // Default slot returns the default values
            prop_assert_eq!(result, &default_slot);
        } else {
            // Undefined slot inherits from Default
            prop_assert_eq!(result.foreground, default_slot.foreground);
            prop_assert_eq!(result.bold, default_slot.bold);
        }
    }
}

// ─── Property 6: High-Contrast Mode WCAG AAA Contrast Ratio ────────────────
// **Validates: Requirements 5.6**

#[test]
fn high_contrast_mode_wcag_aaa_contrast_ratio_enforcement() {
    // Feature: ff-theme, Property 6: high-contrast mode WCAG AAA contrast ratio
    // **Validates: Requirements 5.6**
    let palette = defaults::high_contrast_palette();
    let bg = &palette.editor.background;

    // All syntax colours against editor background must have 7:1 ratio
    let syntax_colours = [
        ("keyword", palette.syntax.keyword),
        ("comment", palette.syntax.comment),
        ("string", palette.syntax.string),
        ("number", palette.syntax.number),
        ("operator", palette.syntax.operator),
        ("type_name", palette.syntax.type_name),
        ("function", palette.syntax.function),
        ("macro_name", palette.syntax.macro_name),
        ("preprocessor", palette.syntax.preprocessor),
        ("default_text", palette.syntax.default_text),
    ];

    for (name, colour) in &syntax_colours {
        let ratio = colour.contrast_ratio(bg);
        assert!(
            ratio >= 7.0,
            "High-contrast syntax.{} has ratio {:.2}:1 (minimum 7:1) against background",
            name,
            ratio
        );
    }

    // Editor foreground against background
    let ratio = palette.editor.foreground.contrast_ratio(bg);
    assert!(
        ratio >= 7.0,
        "High-contrast editor.foreground has ratio {:.2}:1 (minimum 7:1)",
        ratio
    );

    // UI colours
    let ui_bg = &palette.ui.panel_bg;
    let ratio = palette.ui.panel_fg.contrast_ratio(ui_bg);
    assert!(
        ratio >= 7.0,
        "High-contrast ui.panel_fg has ratio {:.2}:1 (minimum 7:1)",
        ratio
    );
}

// ─── Property 7: Element Colour Alpha Enforcement Correctness ───────────────
// **Validates: Requirements 10.4**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// For any element and colour, if the element does NOT allow translucent
    /// rendering, the returned colour has alpha=255. If it DOES allow, alpha is preserved.
    #[test]
    fn element_colour_alpha_enforcement_correctness(
        element_idx in 0usize..12,
        r in 0u8..=255,
        g in 0u8..=255,
        b in 0u8..=255,
        a in 0u8..=255,
    ) {
        // Feature: ff-theme, Property 7: element colour alpha enforcement correctness
        let elements = Element::all();
        let element = elements[element_idx];
        let colour = ColourRGBA::rgba(r, g, b, a);

        let mut map = ElementColourMap::new();
        map.set(element, colour);
        let returned = map.get(element).unwrap();

        if element.allows_translucent() {
            // Translucent elements preserve input alpha
            prop_assert_eq!(returned.a, a,
                "Translucent element {:?} should preserve alpha {} but got {}",
                element, a, returned.a);
        } else {
            // Non-translucent elements force alpha to 255
            prop_assert_eq!(returned.a, 255,
                "Non-translucent element {:?} should force alpha to 255 but got {}",
                element, returned.a);
        }

        // RGB components are always preserved
        prop_assert_eq!(returned.r, r);
        prop_assert_eq!(returned.g, g);
        prop_assert_eq!(returned.b, b);
    }
}
