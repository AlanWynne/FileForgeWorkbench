# Implementation Plan: Theme & Appearance (`ff-theme`)

## Overview

This plan covers the complete implementation of the `ff-theme` crate — the central visual identity layer for FileForgeWorkbench. The crate manages colours, fonts, design tokens, style slots, and visual mode switching (Dark/Light/High-Contrast) through a TOML-based theme configuration format. All rendering code obtains colour values, font selections, and spacing metrics through the theme system rather than using hardcoded values.

This is a **Wave 6 (UI and Rendering)** sub-project. It depends on `ff-configuration-system` (Wave 2) for TOML-based configuration loading, layered overrides, and hot-reload notifications. It is consumed by all rendering subsystems: `menu-and-statusbar`, `text-decorations`, `whitespace-and-guides`, `caret-and-selection`, `syntax-highlighting`, `file-tree-panel`, `layout-and-docking`, and the GUI shell.

---

## Tasks

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-theme/Cargo.toml` with dependencies (serde, toml, thiserror, proptest dev-dep) and deps on `ff-configuration-system`, `ff-logging`
  - [ ] 1.2 Create `crates/ff-theme/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `colour.rs`, `palette.rs`, `style_slot.rs`, `font.rs`, `visual_mode.rs`, `design_tokens.rs`, `element.rs`, `extension.rs`, `loader.rs`, `serialiser.rs`, `api.rs`, `error.rs`, `types.rs`
  - [ ] 1.4 Add `ff-theme` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [ ] 2. Core colour types and representation
  - [ ] 2.1 Define `ColourRGBA { r: u8, g: u8, b: u8, a: u8 }` struct with constructors, Display, serde support
  - [ ] 2.2 Implement `from_hex` parser supporting `#RRGGBB` and `#RRGGBBAA` formats with validation
  - [ ] 2.3 Implement `to_hex` serialiser producing `#RRGGBB` for opaque colours, `#RRGGBBAA` for translucent
  - [ ] 2.4 Implement `to_color32` conversion method producing egui-compatible `Color32` value
  - [ ] 2.5 Define `ColourToken` enum with all semantic token names organised by group (editor, syntax, file_tree, tab_bar, chrome, decorations, indicators, ui)
  - [ ] 2.6 Implement compile-time token verification via exhaustive enum matching
  - [ ] 2.7 Write unit tests for hex parsing (valid/invalid), round-trip, Color32 conversion, alpha handling
  - Covers: Requirement 2 (AC 2.9), Requirement 8 (AC 8.7, 8.8)

- [ ] 3. Theme palette structure
  - [ ] 3.1 Define `EditorColours` struct: background, foreground, accent, muted, modified_indicator, current_line_background, selection_secondary_background
  - [ ] 3.2 Define `SyntaxColours` struct: keyword, comment, string, number, operator, type_name, function, macro_name, preprocessor, default_text
  - [ ] 3.3 Define `FileTreeColours` struct: binary, structured, text, unknown, directory, symlink
  - [ ] 3.4 Define `TabBarColours` struct: active_bg, inactive_bg, active_text, inactive_text, modified_indicator, close_button, drop_target
  - [ ] 3.5 Define `ChromeColours` struct: cursor_row_border, cursor_column_indicator, line_number_fg, line_number_bg, fold_margin_bg, fold_margin_fg, margin_separator
  - [ ] 3.6 Define `DecorationColours` struct: search_highlight, error_underline, warning_underline, info_underline, change_added, change_modified, change_deleted, bookmark
  - [ ] 3.7 Define `IndicatorColours` struct: find_match, brace_match, brace_mismatch, hotspot_underline, user_defined (array of 32 slots)
  - [ ] 3.8 Define `UiColours` struct: panel_bg, panel_fg, panel_border, button_bg, button_fg, button_hover, input_bg, input_border, input_fg, scrollbar_track, scrollbar_thumb, tooltip_bg, tooltip_fg
  - [ ] 3.9 Define `ThemePalette` struct composing all colour groups with `allows_translucent` flags per token
  - [ ] 3.10 Implement `ThemePalette::colour(&self, token: ColourToken) -> ColourRGBA` accessor
  - [ ] 3.11 Write unit tests for palette construction, token lookup, translucent flag checking
  - Covers: Requirement 2 (AC 2.1–2.10)

- [ ] 4. Style slot system
  - [ ] 4.1 Define `CaseTransform` enum: None, Upper, Lower, Camel
  - [ ] 4.2 Define `StyleSlot` struct: foreground, background, font_family (Option), bold, italic, underline, case_transform
  - [ ] 4.3 Define `StyleSlotTable` with 256-entry indexed storage and reserved index constants (DEFAULT=32, LINE_NUMBER=33, BRACE_HIGHLIGHT=34, BRACE_MISMATCH=35, CONTROL_CHAR=36, INDENT_GUIDE=37, CALL_TIP=38, FOLD_DISPLAY=39)
  - [ ] 4.4 Implement default-inheritance: undefined slots inherit all attributes from slot 32 (Default)
  - [ ] 4.5 Implement `allocate_extended_range(count: usize) -> Option<u8>` for syntax-highlighting slot allocation
  - [ ] 4.6 Implement font-family resolution through font stack mechanism with fallback
  - [ ] 4.7 Write unit tests for slot lookup, inheritance, allocation, font resolution
  - Covers: Requirement 3 (AC 3.1–3.7)

- [ ] 5. Font configuration system
  - [ ] 5.1 Define `FontStack` struct: families (Vec<String>), base_size_points (f32)
  - [ ] 5.2 Define `FontConfig` struct with monospace and proportional FontStack fields
  - [ ] 5.3 Implement default font sizes: 14.0pt monospace, 13.0pt proportional
  - [ ] 5.4 Implement font size validation: clamp to 6.0–72.0 range with WARN log for out-of-range
  - [ ] 5.5 Implement platform-default fallback when font stack is empty or all families unavailable
  - [ ] 5.6 Implement `ZoomLevel` (i32) with effective size calculation: base_size + zoom_level
  - [ ] 5.7 Implement zoom clamping: effective size clamped to 2.0–128.0 without modifying zoom value
  - [ ] 5.8 Implement font family availability checking with DEBUG-level logging for unavailable fonts
  - [ ] 5.9 Implement `apply_to_egui(ctx: &egui::Context)` method to set FontDefinitions and Style
  - [ ] 5.10 Write unit tests for size validation, zoom clamping, fallback logic, stack resolution
  - Covers: Requirement 4 (AC 4.1–4.10)

- [ ] 6. Visual modes (Dark / Light / High-Contrast)
  - [ ] 6.1 Define `VisualMode` enum: Dark, Light, HighContrast with serde support
  - [ ] 6.2 Implement per-mode palette storage: base palette with mode-specific overrides in TOML sections (`[dark]`, `[light]`, `[high_contrast]`)
  - [ ] 6.3 Implement mode switching: replace active palette with mode-appropriate values
  - [ ] 6.4 Implement `theme.mode` configuration key integration with configuration-system
  - [ ] 6.5 Implement built-in default palettes for all three modes with appropriate colour choices
  - [ ] 6.6 Implement High-Contrast mode WCAG AAA validation: verify 7:1 minimum contrast ratio for all fg/bg pairs
  - [ ] 6.7 Implement runtime mode switch: apply within one frame without restart
  - [ ] 6.8 Implement consumer notification on mode change via callback/event registration
  - [ ] 6.9 Write unit tests for mode switching, contrast ratio validation, default palette completeness
  - Covers: Requirement 5 (AC 5.1–5.7)

- [ ] 7. Design system tokens
  - [ ] 7.1 Define `SpacingLevel` enum: Xs, Sm, Md, Lg, Xl with default values in logical pixels
  - [ ] 7.2 Define `RadiusLevel` enum: None, Sm, Md, Lg, Full with default values
  - [ ] 7.3 Define `ShadowDefinition` struct: offset_x, offset_y, blur_radius, spread, colour
  - [ ] 7.4 Define `ShadowLevel` enum: Sm, Md, Lg with default ShadowDefinition for each
  - [ ] 7.5 Define `AnimationTiming` struct: duration_ms (u32), easing (EasingCurve enum)
  - [ ] 7.6 Define `AnimationLevel` enum: Fast, Normal, Slow with default timings
  - [ ] 7.7 Define `DesignTokens` struct composing spacing scale, border radii, shadows, animations
  - [ ] 7.8 Implement TOML deserialization with fallback to defaults for missing tokens
  - [ ] 7.9 Implement typed accessor methods: `spacing(SpacingLevel) -> f32`, `border_radius(RadiusLevel) -> f32`, etc.
  - [ ] 7.10 Write unit tests for token access, TOML override, fallback to defaults
  - Covers: Requirement 6 (AC 6.1–6.7)

- [ ] 8. Theme loading and configuration integration
  - [ ] 8.1 Implement `ThemeLoader` struct with configuration-system API integration
  - [ ] 8.2 Implement `theme.active` configuration key reading to determine active theme file
  - [ ] 8.3 Implement TOML theme file parsing with structural validation
  - [ ] 8.4 Implement missing-file fallback: load built-in default dark theme with WARN log
  - [ ] 8.5 Implement invalid-TOML-syntax handling: retain previous theme with WARN log
  - [ ] 8.6 Implement partial-definition support: missing tokens inherit from built-in default for active mode
  - [ ] 8.7 Implement per-value validation: out-of-range/invalid values get defaults with per-token WARN log
  - [ ] 8.8 Implement layered override participation: user → project → profile layers via configuration-system
  - [ ] 8.9 Implement startup blocking: theme fully resolved before first frame renders
  - [ ] 8.10 Implement 50ms loading performance target for typical (<10KB) theme files
  - [ ] 8.11 Write unit tests for file loading, fallback, partial definitions, invalid values, layered overrides
  - Covers: Requirement 1 (AC 1.1–1.7), Requirement 7 (AC 7.1, 7.4)

- [ ] 9. Hot-reload and shared palette access
  - [ ] 9.1 Implement `Arc<ThemePalette>` shared reference for thread-safe read-only palette access
  - [ ] 9.2 Implement hot-reload callback registration with configuration-system for theme file changes
  - [ ] 9.3 Implement hot-reload callback for `theme.active` and `theme.mode` configuration key changes
  - [ ] 9.4 Implement atomic palette swap on hot-reload: no mixed old/new values in any frame
  - [ ] 9.5 Implement change event/notification emission for palette consumers to invalidate caches
  - [ ] 9.6 Implement font re-application to egui context on theme change
  - [ ] 9.7 Write unit tests for arc sharing, atomic swap, change notification delivery, reload triggers
  - Covers: Requirement 7 (AC 7.2, 7.3, 7.5–7.7)

- [ ] 10. Element-based colour system
  - [ ] 10.1 Define `Element` enum: SelectionBg, SelectionFg, AdditionalSelectionBg, AdditionalSelectionFg, CaretFg, AdditionalCaretFg, CaretLineBg, WhitespaceFg, WhitespaceBg, FoldLineColour, FoldLineHighlight, HiddenLineIndicator
  - [ ] 10.2 Implement `element_colour(element: Element) -> Option<ColourRGBA>` returning None for unset elements
  - [ ] 10.3 Implement `element_allows_translucent(element: Element) -> bool` per-element translucent flag
  - [ ] 10.4 Implement alpha enforcement: force alpha to 255 for elements not in translucent set
  - [ ] 10.5 Implement user-set vs base element colour distinction: user overrides base
  - [ ] 10.6 Implement `set_element_colour(element: Element, colour: ColourRGBA)` for runtime overrides
  - [ ] 10.7 Implement `reset_element(element: Element)` to clear runtime override and revert to base/theme
  - [ ] 10.8 Write unit tests for element lookup, translucent enforcement, set/reset, None for unset
  - Covers: Requirement 10 (AC 10.1–10.6)

- [ ] 11. Plugin theme extensions
  - [ ] 11.1 Define `ThemeExtension` struct: token_name, defaults per VisualMode (dark, light, high_contrast), description
  - [ ] 11.2 Implement `register_extension(plugin_id: &str, extension: ThemeExtension) -> Result<(), ThemeError>` with namespace scoping
  - [ ] 11.3 Implement collision detection: reject registration if token name collides with core palette names
  - [ ] 11.4 Implement TOML integration: read `[plugins.{plugin-id}]` section for user-defined overrides
  - [ ] 11.5 Implement mode-aware resolution: plugin tokens resolve to mode-specific user value or plugin default
  - [ ] 11.6 Implement deregistration on plugin unload: remove tokens from active palette, preserve theme file values
  - [ ] 11.7 Implement hot-reload participation: plugin tokens update on theme file changes with callback notification
  - [ ] 11.8 Write unit tests for registration, collision rejection, mode resolution, deregistration, hot-reload
  - Covers: Requirement 11 (AC 11.1–11.7)

- [ ] 12. Theme serialisation
  - [ ] 12.1 Implement `serialise(palette: &ThemePalette) -> String` producing valid TOML with section grouping
  - [ ] 12.2 Implement colour output format: `#RRGGBB` for opaque, `#RRGGBBAA` for translucent
  - [ ] 12.3 Implement section comments: descriptive header comment for each colour group section
  - [ ] 12.4 Implement full palette serialisation: colour groups, style slots, font config, design tokens, mode overrides
  - [ ] 12.5 Implement round-trip property: `parse(serialise(palette)) == palette` for all valid palettes
  - [ ] 12.6 Write unit tests for TOML output validity, colour format, round-trip equality, comment presence
  - Covers: Requirement 9 (AC 9.1–9.5)

- [ ] 13. Hardcoded colour replacement API
  - [ ] 13.1 Implement public `ThemeApi` facade exposing: `colour(token)`, `element_colour(element)`, `style_slot(index)`, `font_config()`, `design_tokens()`, `visual_mode()`
  - [ ] 13.2 Implement `colour` method returning egui `Color32` directly (no conversion needed by caller)
  - [ ] 13.3 Implement compile-time token safety via `ColourToken` enum — misspelled tokens are compile errors
  - [ ] 13.4 Implement `ThemeApi` as the single access point consumed by all rendering subsystems
  - [ ] 13.5 Write unit tests verifying API surface completeness and Color32 conversion correctness
  - Covers: Requirement 8 (AC 8.1–8.8)

- [ ] 14. Theme inheritance and extensibility
  - [ ] 14.1 Implement `base` key support in theme TOML: `base = "theme-name"` declares parent theme
  - [ ] 14.2 Implement token resolution chain: current theme → base theme → built-in default
  - [ ] 14.3 Implement missing-base handling: WARN log and fall back to built-in defaults for unresolved tokens
  - [ ] 14.4 Implement unrecognised section preservation: ignore unknown TOML sections without error, preserve in serialisation
  - [ ] 14.5 Implement sub-structure organisation: palette groups as independent sub-structs for backward-compatible extension
  - [ ] 14.6 Implement thread-safe shared access: `Arc<ThemePalette>` accessible from workbench context and PluginContext
  - [ ] 14.7 Write unit tests for inheritance resolution, missing base fallback, unknown section preservation
  - Covers: Requirement 12 (AC 12.1–12.6)

- [ ] 15. Error handling
  - [ ] 15.1 Define `ThemeError` enum: InvalidColourFormat, FileNotFound, ParseError, InvalidFontSize, SlotAllocationExhausted, ExtensionCollision, InvalidBase, ContrastViolation
  - [ ] 15.2 Implement error message formatting per `[theme] operation: description` standard (≤200 chars)
  - [ ] 15.3 Implement WARN-level logging for all recoverable errors (missing files, invalid values, unavailable fonts)
  - [ ] 15.4 Implement graceful degradation: never crash on theme errors, always fall back to built-in defaults
  - [ ] 15.5 Write unit tests for all error variants, message formatting, and fallback behaviour
  - Covers: Cross-cutting Requirement 8 (Error Message Standards)

- [ ] 16. Property-based tests
  - [ ] 16.1 Write PBT: colour hex round-trip correctness
  - [ ] 16.2 Write PBT: theme serialisation round-trip correctness
  - [ ] 16.3 Write PBT: font size clamping correctness
  - [ ] 16.4 Write PBT: zoom level effective size calculation correctness
  - [ ] 16.5 Write PBT: style slot inheritance correctness
  - [ ] 16.6 Write PBT: high-contrast mode WCAG AAA contrast ratio enforcement
  - [ ] 16.7 Write PBT: element colour alpha enforcement correctness
  - Covers: Requirements 2–5, 9, 10 (see Property-Based Test Definitions below)

- [x] 18. Legacy theme colour semantics (Phase AE)
  - [x] 18.1 Update `legacy_ui_colours()` in `defaults.rs`: set `menu_bar_fg` to `ISPF_WHITE_HI`, add `primary_menu_bg` field to `UiColours` set to `ISPF_BLUE`
  - [x] 18.2 Update `UiColours` struct in `palette.rs` to add `primary_menu_bg: ColourRGBA`
  - [x] 18.3 Update `ColourToken` enum to add `UiPrimaryMenuBackground` variant
  - [x] 18.4 Update `ThemePalette::colour()` match arm for `UiPrimaryMenuBackground`
  - [x] 18.5 Update `PomColours` in `primary_option_menu.rs` to carry all required semantic colours
  - [x] 18.6 Wire `PomColours` from shell into `primary_option_menu::render()` for Legacy theme
  - [x] 18.7 Implement per-element colour rendering in `primary_option_menu::render()` using `PomColours`
  - [x] 18.8 Implement reversed today-cell rendering (turquoise bg, black text) in calendar
  - [x] 18.9 Write unit tests for Legacy palette colour values
  - Covers: Requirement 13 (AC 13.1–13.8)

- [x] 19. User-configurable theme colours and custom themes (Phase AI)
  - [x] 19.1 Implement themes directory scanning: discover all `.toml` files in the platform themes directory at startup and on hot-reload
    - Covers: Requirement 14.2, 14.3
  - [x] 19.2 Implement `ThemeInfo` struct and `list_themes()` API on `ThemeHandle`
    - Covers: Requirement 14.6
  - [x] 19.3 Implement directory watch: register a watch on the themes directory so new/modified `.toml` files trigger a theme list refresh
    - Covers: Requirement 14.3
  - [x] 19.4 Implement `export_theme(name)` on `ThemeHandle`: serialise active palette to TOML with the given name field
    - Covers: Requirement 14.9
  - [x] 19.5 Verify all colour tokens are individually overridable via TOML (audit token coverage in `token.rs` and `defaults.rs`)
    - Covers: Requirement 14.1
  - [x] 19.6 Verify `base` inheritance chain resolution works for user-created themes (already in loader; add test with user theme file)
    - Covers: Requirement 14.4, 14.5, 14.10
  - [x] 19.7 Verify invalid colour token handling in user theme files (WARN + fallback to inherited/default)
    - Covers: Requirement 14.8
  - [x] 19.8 Verify `theme.active` hot-reload applies new theme within one cycle
    - Covers: Requirement 14.7
  - [x] 19.9 Write unit tests: `theme_discovery_finds_toml_files`, `list_themes_includes_builtins`, `export_theme_round_trips`, `invalid_colour_falls_back_to_default`, `base_inheritance_resolves_missing_tokens`
    - Covers: Requirement 14.1–14.10

- [ ] 17. Integration tests
  - [ ] 17.1 Write integration test: full theme load from TOML → palette access → colour retrieval lifecycle
  - [ ] 17.2 Write integration test: visual mode switch (Dark → Light → HighContrast) with consumer notification
  - [ ] 17.3 Write integration test: hot-reload cycle (modify file → detect → reload → notify consumers)
  - [ ] 17.4 Write integration test: plugin extension registration → theme override → mode switch → deregistration
  - [ ] 17.5 Write integration test: theme inheritance chain (child → base → built-in defaults)
  - [ ] 17.6 Write integration test: partial theme file with missing tokens inheriting from defaults
  - [ ] 17.7 Write integration test: style slot allocation by syntax-highlighting consumer
  - Covers: End-to-end validation across Requirements 1–12

---

## Property-Based Test Definitions

### Property 1: Colour Hex Round-Trip Correctness

**Validates: Requirements 2.9, 9.5**

- **Statement:** For any valid ColourRGBA value (r, g, b in 0–255, a in 0–255), serialising to hex and parsing back SHALL produce an identical ColourRGBA value. Opaque colours (a=255) SHALL serialise as `#RRGGBB` and translucent colours (a<255) SHALL serialise as `#RRGGBBAA`.
- **Strategy:** Generate:
  - r: [0, 255], g: [0, 255], b: [0, 255], a: [0, 255]
- **Invariant:** `parse_hex(to_hex(colour)) == colour` AND opaque colours produce 7-char hex AND translucent produce 9-char hex

### Property 2: Theme Serialisation Round-Trip Correctness

**Validates: Requirements 9.1, 9.2**

- **Statement:** For any valid ThemePalette containing all colour groups, style slots, font configuration, and design tokens, serialising to TOML and parsing back SHALL produce an equivalent ThemePalette (`parse(serialise(palette)) == palette`).
- **Strategy:** Generate:
  - Random ThemePalette with all groups populated with random valid ColourRGBA values
  - Random FontConfig with valid sizes in [6.0, 72.0] and random family names
  - Random DesignTokens with valid spacing, radius, shadow, and animation values
  - Random StyleSlot entries for 5–20 slots beyond defaults
- **Invariant:** `parse(serialise(palette)) == palette` with no data loss

### Property 3: Font Size Clamping Correctness

**Validates: Requirements 4.6**

- **Statement:** For any configured font size value (including extreme values), the validated font size SHALL always be within [6.0, 72.0] points. Values below 6.0 are clamped to 6.0, values above 72.0 are clamped to 72.0.
- **Strategy:** Generate:
  - font_size: f32 in [-100.0, 200.0]
- **Invariant:** `6.0 <= validated_size <= 72.0` AND `validated_size == font_size.clamp(6.0, 72.0)`

### Property 4: Zoom Level Effective Size Calculation Correctness

**Validates: Requirements 4.7, 4.8**

- **Statement:** For any base font size in [6.0, 72.0] and any zoom_level (i32), the effective size SHALL equal `(base_size + zoom_level as f32).clamp(2.0, 128.0)`. The zoom_level value itself SHALL never be modified by the clamping operation.
- **Strategy:** Generate:
  - base_size: f32 in [6.0, 72.0]
  - zoom_level: i32 in [-100, 100]
- **Invariant:** `effective_size == (base_size + zoom_level).clamp(2.0, 128.0)` AND `stored_zoom_level == input_zoom_level`

### Property 5: Style Slot Inheritance Correctness

**Validates: Requirements 3.4**

- **Statement:** For any StyleSlotTable where the Default slot (index 32) is fully defined, any undefined slot index in [0, 255] SHALL return attributes identical to the Default slot. Any explicitly defined slot SHALL return its own attributes without inheriting.
- **Strategy:** Generate:
  - Default slot: random full StyleSlot (fg, bg, bold, italic, underline, case_transform)
  - Defined slots: random subset of [0, 255] (excluding 32) with random StyleSlot values
  - Query index: random [0, 255]
- **Invariant:** If query index is in defined set → returns defined values; otherwise → returns Default slot values

### Property 6: High-Contrast Mode WCAG AAA Contrast Ratio Enforcement

**Validates: Requirements 5.6**

- **Statement:** For all foreground/background colour pairs in a High-Contrast mode palette produced by the built-in defaults, the contrast ratio SHALL be at least 7:1 as calculated by the WCAG 2.0 relative luminance formula.
- **Strategy:** Generate:
  - Random selection of fg/bg colour token pairs from the full palette
  - Apply the High-Contrast built-in defaults
- **Invariant:** For each pair, `contrast_ratio(fg, bg) >= 7.0` where contrast_ratio uses WCAG relative luminance

### Property 7: Element Colour Alpha Enforcement Correctness

**Validates: Requirements 10.4**

- **Statement:** For any Element and any ColourRGBA set via the element colour API, if the element does NOT allow translucent rendering, the returned colour SHALL always have alpha=255 regardless of the input alpha. Elements that DO allow translucent rendering SHALL preserve the input alpha unchanged.
- **Strategy:** Generate:
  - element: random Element variant
  - colour: random ColourRGBA with random alpha in [0, 255]
- **Invariant:** If `!allows_translucent(element)` → `returned.a == 255`; if `allows_translucent(element)` → `returned.a == input.a`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types", "tasks": ["2", "15"], "dependsOn": [0] },
    { "id": 2, "label": "Palette and Styles", "tasks": ["3", "4", "5", "7"], "dependsOn": [1] },
    { "id": 3, "label": "Visual Modes and Elements", "tasks": ["6", "10"], "dependsOn": [2] },
    { "id": 4, "label": "Loading and Hot-Reload", "tasks": ["8", "9"], "dependsOn": [3] },
    { "id": 5, "label": "Extensions and Serialisation", "tasks": ["11", "12"], "dependsOn": [4] },
    { "id": 6, "label": "Public API and Inheritance", "tasks": ["13", "14"], "dependsOn": [5] },
    { "id": 7, "label": "Validation", "tasks": ["16", "17"], "dependsOn": [6] }
  ]
}
```

---

## Notes

- This is a Wave 6 (UI and Rendering) crate depending on `ff-configuration-system` (Wave 2) for all config I/O
- The built-in default themes (dark, light, high-contrast) are compiled into the binary for zero-dependency startup
- All consumer subsystems obtain colours through the `ThemeApi` facade — no direct file reading
- The 256 style-slot system is adapted from Scintilla's architecture for efficient syntax-highlighting integration
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Hot-reload leverages the configuration-system file watcher — `ff-theme` does not implement its own watcher
- The WCAG AAA contrast ratio (7:1) requirement applies only to the built-in High-Contrast palette defaults
- Plugin extensions are namespaced under `plugins.{plugin-id}` in TOML to avoid collision with core tokens
- The design.md for this crate may be generated concurrently; task structure is derived from requirements.md

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Theme Configuration File | AC 1.1–1.7 | Task 8 |
| Req 2: Theme Palette Structure | AC 2.1–2.10 | Tasks 2, 3 |
| Req 3: Style Slots | AC 3.1–3.7 | Task 4 |
| Req 4: Font Configuration | AC 4.1–4.10 | Task 5 |
| Req 5: Visual Modes | AC 5.1–5.7 | Task 6 |
| Req 6: Design System Tokens | AC 6.1–6.7 | Task 7 |
| Req 7: Theme Loading and Startup | AC 7.1–7.7 | Tasks 8, 9 |
| Req 8: Replacing Hardcoded Colours | AC 8.1–8.8 | Tasks 2, 13 |
| Req 9: Theme Serialisation Round-Trip | AC 9.1–9.5 | Task 12 |
| Req 10: Element-Based Colour System | AC 10.1–10.6 | Task 10 |
| Req 11: Plugin Theme Extensions | AC 11.1–11.7 | Task 11 |
| Req 12: Extensibility and Forward Compatibility | AC 12.1–12.6 | Task 14 |
| Cross-cutting Req 8: Error Message Standards | All | Task 15 |
