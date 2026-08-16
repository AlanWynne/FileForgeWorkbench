//! Integration tests for ff-theme crate.
//!
//! End-to-end validation across the theme loading, access, and lifecycle.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ff_theme::api::ThemeApi;
use ff_theme::colour::ColourRGBA;
use ff_theme::defaults;
use ff_theme::design_tokens::SpacingLevel;
use ff_theme::element::Element;
use ff_theme::event::ThemeEvent;
use ff_theme::extension::{ExtensionToken, ThemeExtension};
use ff_theme::font::ZoomLevel;
use ff_theme::loader::load_from_toml;
use ff_theme::mode::VisualMode;
use ff_theme::palette::ThemePalette;
use ff_theme::serialiser;
use ff_theme::style_slot::{StyleSlot, StyleSlotTable};
use ff_theme::token::ColourToken;

// ─── Integration Test 1: Full lifecycle ─────────────────────────────────────

#[test]
fn full_theme_load_to_colour_retrieval_lifecycle() {
    // Validates: Requirement 1.1, 7.1, 7.2, 8.7
    let toml = r##"
name = "Integration Test Theme"

[editor]
background = "#282C34"
foreground = "#ABB2BF"

[syntax]
keyword = "#C678DD"
comment = "#5C6370"

[font.monospace]
families = ["JetBrains Mono", "Consolas"]
size = 15.0
"##;

    // Load theme from TOML
    let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
    assert_eq!(palette.name, "Integration Test Theme");
    assert_eq!(palette.editor.background, ColourRGBA::rgb(0x28, 0x2C, 0x34));
    assert_eq!(palette.syntax.keyword, ColourRGBA::rgb(0xC6, 0x78, 0xDD));

    // Create API and verify access
    let api = ThemeApi::with_palette(palette);
    assert_eq!(
        api.colour(ColourToken::EditorBackground),
        ColourRGBA::rgb(0x28, 0x2C, 0x34)
    );
    assert_eq!(
        api.colour(ColourToken::SyntaxKeyword),
        ColourRGBA::rgb(0xC6, 0x78, 0xDD)
    );

    // Font config should be loaded
    let fonts = api.font_config();
    assert_eq!(fonts.monospace.families.len(), 2);
    assert_eq!(fonts.monospace.base_size_pt, 15.0);
}

// ─── Integration Test 2: Visual mode switching ──────────────────────────────

#[test]
fn visual_mode_switch_with_consumer_notification() {
    // Validates: Requirement 5.4, 5.7, 7.7
    let api = ThemeApi::new();
    let change_count = Arc::new(AtomicUsize::new(0));
    let change_count_clone = change_count.clone();

    api.on_change(move |event| {
        if matches!(event, ThemeEvent::ModeChanged { .. }) {
            change_count_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    // Start in Dark mode
    assert_eq!(api.mode(), VisualMode::Dark);

    // Switch to Light
    api.set_mode(VisualMode::Light);
    assert_eq!(api.mode(), VisualMode::Light);
    assert_eq!(change_count.load(Ordering::SeqCst), 1);

    // Verify colours reflect light mode
    let light = defaults::light_palette();
    assert_eq!(
        api.colour(ColourToken::EditorBackground),
        light.editor.background
    );

    // Switch to HighContrast
    api.set_mode(VisualMode::HighContrast);
    assert_eq!(api.mode(), VisualMode::HighContrast);
    assert_eq!(change_count.load(Ordering::SeqCst), 2);

    let hc = defaults::high_contrast_palette();
    assert_eq!(
        api.colour(ColourToken::EditorBackground),
        hc.editor.background
    );
}

// ─── Integration Test 3: Hot-reload cycle ───────────────────────────────────

#[test]
fn hot_reload_cycle_modifies_palette_and_notifies() {
    // Validates: Requirement 7.5, 7.6, 7.7
    let api = ThemeApi::new();
    let notified = Arc::new(AtomicUsize::new(0));
    let notified_clone = notified.clone();

    api.on_change(move |event| {
        if matches!(event, ThemeEvent::PaletteChanged { .. }) {
            notified_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    // Simulate hot-reload by loading a new palette
    let new_toml = r##"
name = "Hot Reloaded"

[editor]
background = "#000000"
"##;
    let new_palette = load_from_toml(new_toml, VisualMode::Dark).unwrap();
    api.set_palette(new_palette);

    // Verify the palette was atomically swapped
    assert_eq!(api.theme_name(), "Hot Reloaded");
    assert_eq!(
        api.colour(ColourToken::EditorBackground),
        ColourRGBA::rgb(0, 0, 0)
    );
    assert_eq!(notified.load(Ordering::SeqCst), 1);
}

// ─── Integration Test 4: Plugin extension lifecycle ─────────────────────────

#[test]
fn plugin_extension_registration_override_mode_switch_deregistration() {
    // Validates: Requirement 11.1, 11.3, 11.4, 11.5
    let api = ThemeApi::new();

    // Register a plugin extension
    let extension = ThemeExtension {
        plugin_id: "sql-viewer".to_string(),
        tokens: vec![ExtensionToken {
            name: "result_header".to_string(),
            dark_default: ColourRGBA::rgb(100, 150, 200),
            light_default: ColourRGBA::rgb(50, 80, 120),
            high_contrast_default: ColourRGBA::rgb(255, 255, 0),
            description: "Result grid header".to_string(),
        }],
    };
    api.register_extension(extension).unwrap();

    // Verify default dark mode resolution
    assert_eq!(
        api.extension_colour("sql-viewer", "result_header"),
        Some(ColourRGBA::rgb(100, 150, 200))
    );

    // Switch to light mode and verify mode-appropriate default
    api.set_mode(VisualMode::Light);
    assert_eq!(
        api.extension_colour("sql-viewer", "result_header"),
        Some(ColourRGBA::rgb(50, 80, 120))
    );

    // Deregister
    api.deregister_extension("sql-viewer");
    assert_eq!(api.extension_colour("sql-viewer", "result_header"), None);
}

// ─── Integration Test 5: Theme inheritance ──────────────────────────────────

#[test]
fn theme_inheritance_chain_resolves_tokens() {
    // Validates: Requirement 12.5, 12.6
    // Partial theme inherits unspecified values from defaults
    let toml = r##"
name = "Child Theme"

[editor]
background = "#111111"
"##;

    let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
    let default = defaults::dark_palette();

    // Specified value is overridden
    assert_eq!(palette.editor.background, ColourRGBA::rgb(0x11, 0x11, 0x11));

    // Unspecified values fall back to defaults
    assert_eq!(palette.editor.foreground, default.editor.foreground);
    assert_eq!(palette.syntax.keyword, default.syntax.keyword);
    assert_eq!(palette.ui.panel_bg, default.ui.panel_bg);
}

// ─── Integration Test 6: Partial theme file ─────────────────────────────────

#[test]
fn partial_theme_file_inherits_missing_tokens_from_defaults() {
    // Validates: Requirement 1.6
    // Only specify one section — everything else should have defaults
    let toml = r##"
name = "Minimal"

[syntax]
keyword = "#FF00FF"
"##;

    let palette = load_from_toml(toml, VisualMode::Dark).unwrap();
    let default = defaults::dark_palette();

    // Syntax keyword is overridden
    assert_eq!(palette.syntax.keyword, ColourRGBA::rgb(255, 0, 255));
    // Everything else is default
    assert_eq!(palette.editor, default.editor);
    assert_eq!(palette.chrome, default.chrome);
    assert_eq!(palette.ui, default.ui);
    assert_eq!(palette.file_tree, default.file_tree);
}

// ─── Integration Test 7: Style slot allocation ──────────────────────────────

#[test]
fn style_slot_allocation_by_syntax_highlighting_consumer() {
    // Validates: Requirement 3.5
    let mut table = StyleSlotTable::default();

    // Syntax highlighter requests 10 slots
    let start = table.allocate_range(10).unwrap();
    assert_eq!(start, 40); // First allocatable index after reserved range

    // Set up some slots
    for i in start..(start + 10) {
        table.set(
            i,
            StyleSlot {
                foreground: ColourRGBA::rgb(i.wrapping_mul(5), 0, 0),
                ..StyleSlot::default()
            },
        );
    }

    // Verify slots are independently set
    assert_eq!(
        table.get(start).foreground,
        ColourRGBA::rgb(start.wrapping_mul(5), 0, 0)
    );
    assert!(table.is_defined(start));
    assert!(table.is_defined(start + 9));

    // Second allocation should continue from where first left off
    let start2 = table.allocate_range(5).unwrap();
    assert_eq!(start2, 50);
}
