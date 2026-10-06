# Requirements Document

## Introduction

This feature specifies the theme and appearance subsystem for FileForgeWorkbench (`ff-theme` crate). The theme system is the **central visual identity layer** for the entire workbench platform. It manages colours, fonts, design tokens, and visual mode switching (dark/light/high-contrast) through a TOML-based theme configuration format. All rendering code obtains colour values, font selections, and spacing metrics through the theme system rather than using hardcoded values.

The theme system replaces all hardcoded colour values with semantic token lookups, provides a structured palette covering every visual element (editor, syntax, file tree, tab bar, chrome, decorations, indicators), supports multiple font stacks (monospace for editor content, proportional for UI elements), and exposes a design system for consistent spacing, border radii, shadows, and animations across the entire workbench.

The `ff-theme` crate is a Wave 6 (UI and Rendering) component. It depends on `configuration-system` for TOML-based configuration loading, layered overrides, and hot-reload. It is consumed by all rendering subsystems: `menu-and-statusbar`, `text-decorations`, `whitespace-and-guides`, `caret-and-selection`, `syntax-highlighting`, `file-tree-panel`, `layout-and-docking`, and the GUI shell.

**Source references:**
- **[FFE-THEME-1]** through **[FFE-THEME-7]** = FileForgeEditor `theme-and-appearance` specification (7 requirements -- theme file, palette, fonts, loading, colour replacement, serialisation, extensibility)
- **[SCI-STYLE]** = Scintilla `ViewStyle` / `Style` / `ElementMap` -- 256 style slots with font/fore/back/bold/italic/underline/case, element-based colour system (selection, caret, whitespace, fold, etc.), zoom level, alpha/transparency support
- **[WB]** = Workbench Architecture Brief -- dark/light/high-contrast modes, design system (spacing, radii, shadows, animations), plugin-provided theme extensions, hot-reload, multiple font stacks

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `configuration-system` | **Dependency** | Provides TOML loading, layered overrides, hot-reload notifications, and schema validation for theme configuration files. Theme loading uses the configuration-system API. |
| `syntax-highlighting` | **Consumer** | Obtains syntax style definitions (colours, font attributes) for each token type from the theme palette. |
| `caret-and-selection` | **Consumer** | Obtains caret colour, selection background/foreground, selection alpha, virtual-space background from the theme. |
| `text-decorations` | **Consumer** | Obtains indicator colours, underline styles, change-marker colours, bookmark colours from the theme palette. |
| `whitespace-and-guides` | **Consumer** | Obtains whitespace dot colour, indent-guide colour, edge-column colour, wrap-marker colour from the theme. |
| `menu-and-statusbar` | **Consumer** | Obtains menu colours, status bar background/foreground, mode indicator colours from the theme. |
| `layout-and-docking` | **Consumer** | Obtains panel border colours, tab-group colours, drag-overlay colours, resize-handle colours from the theme. |
| `plugin-architecture` | **Integration** | Plugins register additional colour tokens and theme extensions through the plugin trait interface. |
| `file-tree-panel` | **Consumer** | Obtains file-category colours, selection highlight, tree-node colours from the theme palette. |

## Glossary

- **Theme_System**: The `ff-theme` crate responsible for loading, validating, storing, hot-reloading, and providing access to all visual appearance settings (colours, fonts, spacing, and design tokens) throughout the workbench. [FFE-THEME-1, WB]
- **Theme_File**: A TOML file defining the complete set of visual tokens for a named theme. Located in the themes directory managed by the configuration-system. [FFE-THEME-1]
- **Theme_Palette**: The in-memory data structure representing the full set of resolved colour values organised into semantic groups (editor, syntax, file_tree, tab_bar, chrome, decorations, indicators, UI). [FFE-THEME-2, SCI-STYLE]
- **Colour_Token**: A named reference to a specific colour within the Theme_Palette (e.g., `editor.background`, `syntax.keyword`, `chrome.line_number_foreground`). Tokens are the sole interface for rendering code to obtain colours. [FFE-THEME-2]
- **Design_Token**: A named reference to a non-colour visual property: spacing value, border radius, shadow definition, or animation timing. Part of the design system. [WB]
- **Font_Stack**: An ordered list of font family names with fallback semantics. The theme defines separate stacks for monospace (editor) and proportional (UI) contexts. [FFE-THEME-3, WB]
- **Visual_Mode**: One of three appearance modes -- Dark, Light, or High-Contrast -- that determines which set of palette values is active. [WB]
- **Style_Slot**: An indexed slot (0-255) defining a combination of font, foreground colour, background colour, and text attributes (bold, italic, underline, case) for a specific syntax or UI element. Adapted from Scintilla's 256-style system. [SCI-STYLE]
- **Element_Colour**: A named colour associated with a UI element (selection background, caret, whitespace, fold margin, etc.) that can optionally support alpha transparency. Adapted from Scintilla's element-based colour system. [SCI-STYLE]
- **Zoom_Level**: An integer offset applied to all font sizes, increasing or decreasing the effective rendered size without modifying the base theme configuration. [SCI-STYLE]
- **Theme_Extension**: A set of additional colour tokens registered by a plugin to extend the palette with plugin-specific visual elements. [WB]
- **Hot_Reload**: The ability to detect changes to theme files on disk and apply updated colours/fonts without restarting the workbench. Leverages configuration-system hot-reload. [WB, FFE-THEME-1]
- **Design_System**: The collection of design tokens (spacing scale, border radii, shadow definitions, animation curves) that ensure visual consistency across all workbench UI components. [WB]
- **Rendering_Code**: Any function or method that draws UI elements and requires colour, font, or design-token values from the Theme_System. [FFE-THEME-5]

## Requirements

### Requirement 1: Theme Configuration File

**User Story:** As a workbench user, I want to define my colour scheme, fonts, and visual preferences in a TOML theme file, so that I can customise the workbench appearance without modifying source code and share themes with other users.

**Source:** [FFE-THEME-1] Theme Configuration File; [WB] TOML-based theming.

#### Acceptance Criteria

1. THE Theme_System SHALL load theme definitions from TOML files located in the themes directory managed by the configuration-system (e.g., `themes/dark.toml`, `themes/light.toml`).
2. THE Theme_System SHALL identify the active theme by reading the `theme.active` configuration key from the configuration-system, which names the theme file to load.
3. WHEN the specified theme file does not exist, THE Theme_System SHALL fall back to a built-in default dark theme and emit a WARN-level log record identifying the missing file.
4. WHEN a theme file contains invalid TOML syntax, THE Theme_System SHALL log a warning identifying the file and parse error, retain the previously active theme (or fall back to the built-in default if no previous theme exists), and continue operating.
5. WHEN a theme file contains a valid TOML structure with individual invalid values (out-of-range colour components, unknown colour format, invalid font size), THE Theme_System SHALL log a warning for each invalid value and use the corresponding default for that specific token.
6. THE theme file format SHALL support partial definitions where any omitted token inherits its value from the built-in default for the active Visual_Mode.
7. THE Theme_System SHALL load theme settings through the configuration-system API, participating in the layered override model so that user-layer, project-layer, and profile-layer theme overrides function correctly.

**REWORDED by CR-CH-056:** The theme file is now a VERSIONED TOML file whose chrome is an
embedded `egui::Style` sub-table (Requirement 25). Criterion 1.6's partial-definition /
default-fill behaviour is PRESERVED and extended to the egui `Style` sub-table (missing or
extra egui fields are tolerated on load, Requirement 25.3). The configuration-system
loading path (1.1, 1.2, 1.7) is unchanged.

---

### Requirement 2: Theme Palette Structure

**SUPERSEDED IN PART by CR-CH-056 (Requirement 23, egui-Native Theme Model).** The
CHROME portions of this requirement -- the `tab_bar` and `ui` colour groups and the
chrome-adjacent `editor` fields (background, foreground, accent) -- are REPLACED by the
egui-native chrome layer of Requirement 23, which configures the full `egui::Style` /
`Visuals` surface rather than a flat fixed set of chrome colours. The DOMAIN colour
groups below -- `syntax` (2.2), `file_tree` (2.3), the editor-domain fields of 2.1
(modified_indicator, current_line_background, selection_secondary_background), the editor
gutter group (renamed `chrome` -> `gutter`, 2.5), `decorations` (2.6), `indicators`
(2.7), and the RGBA colour representation and translucency rules (2.9, 2.10) -- are
RETAINED by Requirement 23 because egui does not model them. See Requirement 23 for the
replacement chrome contract. The criteria below are kept for history; the chrome criteria
(2.4, 2.8, and the chrome fields of 2.1) no longer bind.

**User Story:** As a workbench user, I want a comprehensive colour palette covering all parts of the UI, so that I have fine-grained control over the visual appearance and every element respects my chosen theme.

**Source:** [FFE-THEME-2] Theme Palette Structure; [SCI-STYLE] Element-based colour system.

#### Acceptance Criteria

1. THE Theme_Palette SHALL define an **editor** colour group containing at minimum: background, foreground, accent, muted/disabled text, modified indicator, current-line background, and selection-secondary background.
2. THE Theme_Palette SHALL define a **syntax** colour group containing at minimum: keyword, comment, string, number, operator, type, function, macro, preprocessor, and default text colours.
3. THE Theme_Palette SHALL define a **file_tree** colour group containing at minimum: non-editable binary, FileForge structured, standard text, unknown file type, directory, and symbolic link colours.
4. THE Theme_Palette SHALL define a **tab_bar** colour group containing at minimum: active tab background, inactive tab background, active tab text, inactive tab text, modified indicator, close-button colour, and drop-target highlight.
5. THE Theme_Palette SHALL define a **chrome** colour group containing at minimum: cursor row border, cursor column indicator, line number gutter foreground, line number gutter background, fold margin background, fold margin foreground, and margin separator.
6. THE Theme_Palette SHALL define a **decorations** colour group containing at minimum: search highlight, error underline, warning underline, info underline, change-added marker, change-modified marker, change-deleted marker, and bookmark indicator.
7. THE Theme_Palette SHALL define an **indicators** colour group containing at minimum: find-match highlight, brace-match highlight, brace-mismatch highlight, hotspot underline, and up to 32 user-defined indicator colours indexed by slot number.
8. THE Theme_Palette SHALL define a **ui** colour group containing at minimum: panel background, panel foreground, panel border, button background, button foreground, button hover, input background, input border, input foreground, scrollbar track, scrollbar thumb, tooltip background, and tooltip foreground.
9. FOR ALL Colour_Token values in the Theme_Palette, THE Theme_System SHALL represent each colour as an RGBA quadruplet with red, green, blue components in the range 0-255 and an alpha component in the range 0-255 (where 255 is fully opaque).
10. THE Theme_Palette SHALL support alpha/transparency on tokens where translucent rendering is semantically meaningful (selection background, indicator overlays, caret-line background), as indicated by a per-token `allows_translucent` flag.

---

### Requirement 3: Style Slots

**User Story:** As a syntax-highlighting engine, I need indexed style slots that define the visual attributes for each token type, so that I can efficiently map lexer output to rendering instructions.

**Source:** [SCI-STYLE] 256 style slots with font, fore, back, bold, italic, underline, case.

#### Acceptance Criteria

1. THE Theme_System SHALL provide a style-slot table containing up to 256 indexed Style_Slot entries (indices 0-255).
2. EACH Style_Slot SHALL define: foreground colour, background colour, font family (optional override of the default monospace stack), bold flag, italic flag, underline flag, and case transformation (none, upper, lower, camel).
3. THE Theme_System SHALL define reserved style indices for: Default (index 32), Line Number (index 33), Brace Highlight (index 34), Brace Mismatch (index 35), Control Character (index 36), Indent Guide (index 37), Call Tip (index 38), and Fold Display Text (index 39).
4. ALL Style_Slot entries not explicitly defined in the theme file SHALL inherit all attributes from the Default style slot (index 32).
5. THE Theme_System SHALL allow the syntax-highlighting subsystem to allocate extended style ranges beyond the base styles, returning the starting index of a contiguous block of available slots.
6. WHEN a Style_Slot's font family is set, THE Theme_System SHALL resolve it through the font stack mechanism, falling back to the editor monospace stack if the specified family is unavailable.
7. WHEN rendering text using a Style_Slot, THE Rendering_Code SHALL apply all defined attributes (foreground, background, bold, italic, underline, case) as a combined visual effect.

---

### Requirement 4: Font Configuration

**User Story:** As a workbench user, I want to configure separate font stacks for editor content and UI elements, with size control and fallback behaviour, so that I can choose typefaces suited to each context.

**Source:** [FFE-THEME-3] Font Configuration; [WB] Multiple font stacks; [SCI-STYLE] Zoom level.

#### Acceptance Criteria

1. THE Theme_System SHALL define a **monospace** Font_Stack for editor content, specifying an ordered list of font family names with automatic fallback to the next family if a font is unavailable.
2. THE Theme_System SHALL define a **proportional** Font_Stack for UI elements (menus, panels, status bar, dialogs), specifying an ordered list of font family names with automatic fallback.
3. WHEN a Font_Stack does not specify any font families (empty list or missing configuration), THE Theme_System SHALL default to the platform's built-in monospace font for the editor stack and the platform's built-in proportional font for the UI stack.
4. THE Theme_System SHALL specify a base font size as a floating-point value in points, independently configurable for the monospace and proportional stacks.
5. WHEN a font size is not specified, THE Theme_System SHALL default to 14.0 points for the monospace stack and 13.0 points for the proportional stack.
6. WHEN a configured font size is outside the valid range of 6.0-72.0 points, THE Theme_System SHALL log a warning and clamp the value to the nearest boundary (6.0 or 72.0).
7. THE Theme_System SHALL support a Zoom_Level integer offset (positive or negative) that is added to the base font size of the monospace stack for all editor rendering, without modifying the stored base size in the theme configuration.
8. WHEN a Zoom_Level adjustment would result in an effective font size below 2.0 or above 128.0 points, THE Theme_System SHALL clamp the effective size to the boundary without modifying the Zoom_Level value itself.
9. WHEN the first font family in a Font_Stack is not available on the system, THE Theme_System SHALL attempt each subsequent family in order, log a DEBUG-level record for each unavailable font, and fall back to the platform default if no family in the stack is available.
10. THE Theme_System SHALL apply the resolved monospace font and size to the egui/rendering context before the first frame is rendered.

---

### Requirement 5: Visual Modes (Dark / Light / High-Contrast)

**User Story:** As a workbench user, I want to switch between dark, light, and high-contrast appearance modes, so that I can choose the mode that best suits my environment, preference, or accessibility needs.

**Source:** [WB] Dark mode, light mode, high-contrast mode support.

#### Acceptance Criteria

1. THE Theme_System SHALL support three Visual_Modes: Dark, Light, and High-Contrast.
2. EACH theme file SHALL define palette values for all three Visual_Modes, either in separate sections (`[dark]`, `[light]`, `[high_contrast]`) or through a base palette with per-mode overrides.
3. THE Theme_System SHALL store the active Visual_Mode as a configuration key (`theme.mode`) managed through the configuration-system.
4. WHEN the active Visual_Mode changes, THE Theme_System SHALL replace the active Theme_Palette with the palette values corresponding to the new mode and notify all registered consumers.
5. THE built-in default themes SHALL provide visually appropriate defaults for all three modes: dark backgrounds with light text for Dark mode, light backgrounds with dark text for Light mode, and maximum-contrast colours meeting WCAG AAA contrast ratios for High-Contrast mode.
6. WHEN High-Contrast mode is active, THE Theme_System SHALL ensure that all foreground/background colour pairs in the palette achieve a minimum contrast ratio of 7:1 (WCAG AAA level).
7. THE Theme_System SHALL allow users to switch Visual_Mode at runtime without restarting the workbench, with the change taking effect within one frame.

**REWORDED by CR-CH-056:** The three Visual_Modes are retained; a Theme now carries its
`VisualMode` as metadata (Dark / Light / High-Contrast / Legacy) and produces an
`egui::Style` for chrome plus the domain groups. The built-in Dark / Light instances
become Solarized (Requirement 18, revised); High-Contrast is unchanged (5.6 still binds).
The WCAG AA contrast advisory (consumed by the Theme Editor, Requirement 20.9) is
re-expressed against the egui `Visuals` foreground/background pairs in Requirement 23.11.

---

### Requirement 6: Design System Tokens

**User Story:** As a UI developer, I want a consistent set of design tokens for spacing, border radii, shadows, and animations, so that all workbench panels and components share a unified visual language.

**Source:** [WB] Design system (consistent spacing, border radii, shadows, animations).

#### Acceptance Criteria

1. THE Theme_System SHALL define a **spacing scale** as an array of design tokens providing consistent spacing values (e.g., `spacing.xs`, `spacing.sm`, `spacing.md`, `spacing.lg`, `spacing.xl`) measured in logical pixels.
2. THE Theme_System SHALL define **border radius** tokens (e.g., `radius.none`, `radius.sm`, `radius.md`, `radius.lg`, `radius.full`) for consistent corner rounding across all UI components.
3. THE Theme_System SHALL define **shadow** tokens specifying offset, blur radius, spread, and colour for consistent elevation effects (e.g., `shadow.sm`, `shadow.md`, `shadow.lg`).
4. THE Theme_System SHALL define **animation** tokens specifying duration and easing curve names (e.g., `animation.fast`, `animation.normal`, `animation.slow`) for consistent motion timing.
5. ALL Design_Token values SHALL be configurable through the theme TOML file, using the same override and fallback semantics as colour tokens.
6. WHEN a Design_Token is not defined in the active theme file, THE Theme_System SHALL use the built-in default value for that token.
7. THE Theme_System SHALL expose Design_Token values through typed accessor methods that return the appropriate numeric or structured type (e.g., `spacing(SpacingLevel) -> f32`, `border_radius(RadiusLevel) -> f32`).

**REWORDED by CR-CH-056:** The Design_Tokens (spacing scale, border radii, shadows) are
now WIRED onto the egui `Style` at the apply seam -- `spacing` -> `Style.spacing`
(item_spacing / button_padding / window_margin / indent), `border_radius` -> the
`CornerRadius` fields of `Visuals` / `WidgetVisuals`, and `shadows` -> `Visuals`
`window_shadow` / `popup_shadow` (Requirement 23.6). Before CR-CH-056 these tokens were
defined but unused at the egui seam; CR-CH-056 makes them effective.

---

### Requirement 7: Theme Loading and Startup

**User Story:** As a workbench developer, I want the theme to be loaded once at startup and made available to all rendering code through a shared reference, so that colour and font lookups are efficient and consistent across the entire UI.

**Source:** [FFE-THEME-4] Theme Loading at Startup; [WB] Hot-reload.

#### Acceptance Criteria

1. WHEN the workbench starts, THE Theme_System SHALL load and validate the active theme before any UI rendering occurs, blocking the first frame until the palette and font configuration are resolved.
2. THE Theme_System SHALL make the validated Theme_Palette accessible to all Rendering_Code through a shared, read-only reference (`Arc<ThemePalette>` or equivalent) without requiring each component to load or parse configuration independently.
3. WHEN the theme is successfully loaded, THE Theme_System SHALL apply the resolved font families and sizes to the rendering context (egui `FontDefinitions` and `Style`) before the first frame is rendered.
4. THE Theme_System SHALL complete initial theme loading within 50 milliseconds for a typical theme file (under 10 KB), exclusive of font discovery time on the operating system.
5. THE Theme_System SHALL register a hot-reload callback with the configuration-system so that changes to theme files or the `theme.active` / `theme.mode` configuration keys trigger a palette reload without application restart.
6. WHEN a hot-reload is triggered, THE Theme_System SHALL atomically swap the shared Theme_Palette reference so that all subsequent rendering operations use the new palette, with no frame rendered using a mix of old and new values.
7. THE Theme_System SHALL emit an event/notification when the palette changes (due to hot-reload, mode switch, or theme switch) so that consumers can invalidate caches or trigger re-renders.

---

### Requirement 8: Replacing Hardcoded Colours

**User Story:** As a workbench developer, I want all rendering code to obtain colours exclusively from the theme palette, so that the entire UI respects the user's chosen theme and no hardcoded values bypass the theming system.

**Source:** [FFE-THEME-5] Replacing Hardcoded Colours.

#### Acceptance Criteria

1. ALL Rendering_Code for syntax highlighting SHALL obtain token colours from the Theme_Palette syntax group or Style_Slot table; no syntax colour values SHALL be hardcoded in rendering functions.
2. ALL Rendering_Code for the editor chrome (cursor row border, cursor column indicator, line numbers, fold margins) SHALL obtain colours from the Theme_Palette chrome group; no chrome colour values SHALL be hardcoded.
3. ALL Rendering_Code for the file tree panel SHALL obtain file-category colours from the Theme_Palette file_tree group; no file-type colour values SHALL be hardcoded.
4. ALL Rendering_Code for the tab bar SHALL obtain colours from the Theme_Palette tab_bar group; no tab-bar colour values SHALL be hardcoded.
5. ALL Rendering_Code for UI panels, buttons, inputs, tooltips, and scrollbars SHALL obtain colours from the Theme_Palette ui group; no UI-component colour values SHALL be hardcoded.
6. ALL Rendering_Code for text decorations and indicators SHALL obtain colours from the Theme_Palette decorations and indicators groups; no decoration colour values SHALL be hardcoded.
7. WHEN a Colour_Token lookup is performed, THE Theme_System SHALL return a valid rendering-compatible colour value (e.g., egui `Color32`) that can be used directly without conversion by the caller.
8. THE Theme_System SHALL provide a compile-time-verifiable token API (using Rust enums or const identifiers) so that misspelled or non-existent token names produce compilation errors rather than runtime failures.

**REWORDED by CR-CH-056:** The "no hardcoded colours" rule is PRESERVED, but the chrome
sourcing changes: criteria 8.4 (tab bar) and 8.5 (UI panels / buttons / inputs / tooltips
/ scrollbars) are now satisfied by the egui `Style` / `Visuals` the chrome layer applies
at the single `apply_to_egui` seam (Requirement 23.5), not by reading flat `tab_bar.*` /
`ui.*` palette fields. The hardcoded Legacy slider-colour injection currently living at
the apply seam is REMOVED by CR-CH-056 (it becomes part of the Legacy instance's egui
`Style`). The domain-group sourcing (8.1 syntax, 8.2 gutter, 8.3 file tree, 8.6
decorations/indicators) is unchanged.

---

### Requirement 9: Theme Serialisation Round-Trip

**User Story:** As a workbench developer, I want the theme configuration to survive a parse-then-serialise cycle without data loss, so that settings UI, theme editors, and export/import tools can safely read and write theme files.

**Source:** [FFE-THEME-6] Theme Serialisation Round-Trip.

#### Acceptance Criteria

1. THE Theme_System SHALL provide a serialiser that writes a Theme_Palette (including all colour groups, style slots, font configuration, design tokens, and per-mode overrides) to valid TOML format.
2. FOR ALL valid Theme_Palette values, parsing the serialised TOML output and comparing to the original Theme_Palette SHALL produce an equivalent result (round-trip property: `parse(serialise(palette)) == palette`).
3. THE serialiser SHALL preserve the semantic grouping structure (sections for editor, syntax, file_tree, tab_bar, chrome, decorations, indicators, ui, font, design, modes) in the output TOML.
4. THE serialiser SHALL include descriptive comments in the output TOML file for each section and each colour group, explaining the purpose of the section.
5. THE serialiser SHALL output colour values in a consistent, human-readable format (`"#RRGGBB"` for opaque colours, `"#RRGGBBAA"` for colours with non-255 alpha).

---

### Requirement 10: Element-Based Colour System

**User Story:** As a rendering subsystem, I need to query colours for specific UI elements (selection, caret, whitespace, fold markers) with optional transparency support, so that I can render overlapping visual elements with correct blending.

**Source:** [SCI-STYLE] Element-based colour system -- selection, caret, whitespace, fold, etc.; alpha/transparency support.

#### Acceptance Criteria

1. THE Theme_System SHALL provide an element-colour API: `element_colour(element: Element) -> Option<ColourRGBA>` that returns the colour for a named UI element, or `None` if no colour is set for that element (indicating the element should not be rendered or should use a computed default).
2. THE Theme_System SHALL define elements for at minimum: selection background, selection foreground, additional-selection background, additional-selection foreground, caret foreground, additional-caret foreground, caret-line background, whitespace foreground, whitespace background, fold-line colour, fold-line-highlight colour, and hidden-line indicator colour.
3. WHEN an element colour has an alpha component less than 255, THE Rendering_Code SHALL use alpha-blended rendering for that element, compositing over the underlying content.
4. THE Theme_System SHALL track which elements allow translucent rendering (via `element_allows_translucent(element) -> bool`); elements not in the translucent set SHALL have their alpha forced to 255.
5. THE Theme_System SHALL support both user-set element colours (defined in the theme file) and base element colours (derived from the palette); user-set colours override base colours.
6. THE Theme_System SHALL provide `set_element_colour(element, colour)` and `reset_element(element)` methods for runtime element colour overrides (e.g., per-document overrides driven by plugin logic).

---

### Requirement 11: Plugin Theme Extensions

**User Story:** As a plugin developer, I want to register additional colour tokens with the theme system, so that my plugin's custom UI elements respect the user's theme and participate in mode switching and hot-reload.

**Source:** [WB] Plugin-provided theme extensions (register new colour tokens).

#### Acceptance Criteria

1. THE Theme_System SHALL provide a `register_extension(plugin_id, extension: ThemeExtension)` method that allows plugins to register additional Colour_Token names scoped to the plugin's namespace (e.g., `plugins.sql-viewer.result_grid_header`).
2. EACH ThemeExtension registration SHALL include: the token name, a default colour for each Visual_Mode (dark, light, high-contrast), and a human-readable description.
3. WHEN a theme file defines colour values for a registered plugin token (under `[plugins.{plugin-id}]` in the theme TOML), THE Theme_System SHALL use the theme-defined value instead of the plugin-provided default.
4. WHEN the active Visual_Mode changes, THE Theme_System SHALL resolve plugin extension tokens to the appropriate mode-specific value (user-defined override or plugin default for that mode).
5. WHEN a plugin is unloaded, THE Theme_System SHALL deregister the plugin's extension tokens from the active palette; previously defined values in theme files SHALL be preserved but not actively served.
6. THE Theme_System SHALL prevent plugin token names from colliding with core palette token names; IF a collision is detected during registration, THEN THE Theme_System SHALL reject the registration and return an error.
7. PLUGIN-registered theme extensions SHALL participate in hot-reload: WHEN the theme file is modified and contains changes to a plugin's colour tokens, THE Theme_System SHALL update the palette and notify the plugin's registered callback.

---

### Requirement 13: Legacy Theme Colour Semantics

**User Story:** As a user running the Legacy (ISPF 3270) theme, I want the screen colours to faithfully reproduce the ISPF semantic colour assignments so that the workbench looks and feels like a real 3270 terminal session.

**Source:** ISPF 3270 terminal colour conventions; user requirement (Phase AE).

**REWORDED by CR-CH-056:** The ISPF semantic colour contract below is RETAINED but
RE-FRAMED: Legacy is now ONE INSTANCE of the general egui-native theme model (Requirement
23), not the model's defining shape. The ISPF roles (white headings, green body,
turquoise labels, white keys, blue structure) live in the Legacy instance's egui `Style`
chrome plus the retained domain groups. ONE value changes: the primary option-menu /
title-band background is TONED DOWN from the full-intensity ISPF structural blue
(`#0000AA`) to a muted deep navy (`#000060`); criterion 13.2 is amended accordingly below.
All other Legacy colours remain byte-identical, so the authentic look is preserved.

#### Acceptance Criteria

1. WHEN the Legacy theme is active, THE menu bar top-level item text SHALL be rendered in white (`#FFFFFF`).
2. WHEN the Legacy theme is active, THE primary menu (screen title / heading row on any screen) SHALL be rendered with a toned-down muted navy background (`#000060`, amended by CR-CH-056 from the former `#0000AA`) and white text. (The former `#0000AA` is the pre-CR-CH-056 value; `#000060` is the current contract.)
3. WHEN the Legacy theme is active, ALL normal body text SHALL be rendered in bright green (`#00FF00`).
4. WHEN the Legacy theme is active, option item numbers or key characters SHALL be rendered in white (`#FFFFFF`).
5. WHEN the Legacy theme is active, option item names (labels) SHALL be rendered in turquoise (`#00AAAA`).
6. WHEN the Legacy theme is active, option item descriptions SHALL be rendered as normal text in bright green (`#00FF00`).
7. WHEN the Legacy theme is active, THE calendar widget SHALL be rendered in turquoise (`#00AAAA`).
8. WHEN the Legacy theme is active AND the calendar is displaying the current month, THE cell for today's date SHALL be rendered in reversed colours: turquoise background (`#00AAAA`) with black text (`#000000`).

---

### Requirement 14: User-Configurable Theme Colours and Custom Themes

**User Story:** As a workbench user, I want to configure every theme colour setting and create entirely new themes via TOML configuration files, so that I can fully personalise the workbench appearance without modifying source code.

**Source:** User requirement (Phase AI). Extends Requirement 1 (Theme Configuration File) and Requirement 5 (Visual Modes).

#### Acceptance Criteria

1. EVERY colour token in the Theme_Palette (all groups: editor, syntax, file_tree, tab_bar, chrome, decorations, indicators, ui) SHALL be individually overridable in a theme TOML file using the `#RRGGBB` or `#RRGGBBAA` hex format.
2. THE Theme_System SHALL discover all `.toml` files in the themes directory (`themes/` under the user config path) and make them available as selectable themes, in addition to the four built-in themes (dark, light, high-contrast, legacy).
3. WHEN a user creates a new `.toml` file in the themes directory, THE Theme_System SHALL make it available as a selectable theme on the next hot-reload cycle or application restart, without requiring any code change.
4. A user-created theme file SHALL be able to declare `base = "<theme-name>"` to inherit all tokens from a named built-in or previously defined theme, overriding only the tokens it explicitly specifies.
5. WHEN a user-created theme file omits any colour token, THE Theme_System SHALL inherit that token's value from the declared `base` theme, or from the built-in default for the active Visual_Mode if no `base` is declared.
6. THE Theme_System SHALL expose the list of all available themes (built-in and user-created) through a queryable API so that the Settings Context and View menu can present them as selectable options.
7. WHEN the user changes the active theme via the `theme.active` configuration key (through the Settings Context or by editing the config file), THE Theme_System SHALL load and apply the new theme within one hot-reload cycle without application restart.
8. THE Theme_System SHALL validate every colour token value in a user-created theme file; WHEN an invalid colour format is encountered, THE Theme_System SHALL log a WARN, use the inherited or default value for that token, and continue loading the remainder of the theme.
9. THE Theme_System SHALL provide a `serialise_theme` function that writes the current active palette to a TOML file in the themes directory, enabling users to export and share their customised theme.
10. WHEN a user-created theme file specifies a `base` theme that cannot be resolved, THE Theme_System SHALL emit a WARN-level log record and fall back to the built-in default theme for all unresolved tokens.

**REWORDED by CR-CH-056:** "every colour token overridable" (14.1) now covers the egui
`Style` chrome fields embedded in the theme file in addition to the retained domain
groups. The `base` inheritance key (14.4, 14.5) MUST be ACTUALLY RESOLVED by the loader --
today it is read and discarded, so inherited themes silently fall back to defaults;
Requirement 25.4 makes `base` resolution a binding criterion. The built-in count in 14.2
changes from four to FIVE (Requirement 18, revised): Default Dark, Default Light, Default
High Contrast, Default Legacy, Legacy Soft.

---

### Requirement 16: OS Dark/Light Mode Follow

**User Story:** As a workbench user, I want the workbench to automatically follow the operating system dark/light mode preference, so that the theme switches without manual intervention when I change my OS appearance setting.

**Source:** Gap analysis medium-priority item: "System theme follow (OS dark/light mode)" -- `theme-and-appearance` gap.

#### Acceptance Criteria

1. THE Theme_System SHALL register a `theme.follow_os` configuration key (boolean, default `false`) that controls whether the workbench automatically follows the OS dark/light preference.
2. WHEN `theme.follow_os` is `true` AND the OS reports a dark preference, THE Theme_System SHALL set the active Visual_Mode to `Dark` if it is not already `Dark`.
3. WHEN `theme.follow_os` is `true` AND the OS reports a light preference, THE Theme_System SHALL set the active Visual_Mode to `Light` if it is not already `Light`.
4. WHEN `theme.follow_os` is `false`, THE Theme_System SHALL NOT change the Visual_Mode in response to OS preference changes; the user-configured `theme.mode` value is used exclusively.
5. WHEN the OS dark/light preference changes while the workbench is running AND `theme.follow_os` is `true`, THE Theme_System SHALL detect the change within one egui frame and apply the corresponding Visual_Mode switch.
6. THE OS preference SHALL be detected via `eframe`/`egui`'s `visuals.dark_mode` field on the egui context, which reflects the platform system preference.
7. WHEN `theme.follow_os` is enabled and the OS preference is applied, THE Theme_System SHALL NOT persist the auto-applied mode to the `theme.mode` config key, so that disabling `theme.follow_os` restores the user's last manually chosen mode.
8. THE Settings Context SHALL expose `theme.follow_os` as a checkbox widget labelled "Follow OS dark/light mode".

---

### Requirement 15: Extensibility and Forward Compatibility

**User Story:** As a workbench developer, I want the theme system to be extensible so that new UI features can introduce additional tokens without modifying the core theme structure, and old theme files remain loadable.

**Source:** [FFE-THEME-7] Extensibility for Future Features.

#### Acceptance Criteria

1. THE Theme_Palette data structure SHALL be organised into clearly named sub-structures (one per colour group) so that new groups can be added without modifying existing groups.
2. WHEN a theme TOML file contains sections or keys not recognised by the current version of the Theme_System, THE Theme_System SHALL ignore unrecognised sections without error, preserving them during serialisation round-trips.
3. THE Theme_System SHALL expose the Theme_Palette through a shared, thread-safe reference accessible from any rendering component, any subsystem holding a reference to the workbench context, and any plugin via its PluginContext.
4. THE Theme_System SHALL provide a stable public API surface such that adding new token groups or design tokens does not require changes to existing consumers (backward-compatible extension).
5. THE Theme_System SHALL support theme inheritance: a theme file MAY declare a `base` theme name, and all tokens not explicitly defined in the file SHALL be inherited from the base theme rather than from the built-in defaults.
6. WHEN a theme file specifies a `base` theme that cannot be found, THE Theme_System SHALL emit a WARN-level log record and fall back to the built-in default theme for unresolved tokens.


---

### Requirement 17: THEME Command (Command Parity)

**User Story:** As a workbench user, I want a single `THEME` command: `THEME <name>` switches to a named theme from the command line exactly as from the Settings menu, and bare `THEME` opens the Theme Editor -- so theme switching honours the command-driven principle (every user action is a command), is scriptable/automatable, and there is no redundant second command.

**Source:** `docs/specs/workbench-requirements-merge/architecture-brief.md` Principle 2 (Command Driven -- "Every user action is a command. Menus, toolbars, keyboard shortcuts, automation scripts and future AI agents invoke commands through the same dispatcher."). Closes the gap where the Settings theme menu called the theme setter directly, bypassing the command layer, and no `THEME` command existed.

#### Acceptance Criteria

**Revised by CR-CH-024 (name-based THEME command).** `THEME <mode>` (mode-keyword-only)
is replaced by `THEME <name>`, a general selector over the full theme list (built-ins
plus user themes). The old mode keywords (`dark`/`light`/`high_contrast`/`legacy`) still
work because they are built-in shorthands, so nothing regresses. The separate `THEMES`
command is REMOVED (redundant): `THEME` with no argument now opens the Theme Editor
context that `THEMES` used to open, so there is ONE theme command.

1. THE workbench SHALL provide a `THEME` command invocable from the command line in any context.
     THE `THEMES` command SHALL be removed; `THEME` is the sole theme command. Any prior
     `THEMES` call site (Settings menu "Theme Editor" item, the Settings `T` option in
     `menus/settings.toml`, the command palette) SHALL dispatch `THEME` instead.

2. WHEN `THEME <name>` is issued, THE workbench SHALL resolve `<name>` against the available
     themes list (built-ins plus user themes, Requirement 14.6) case-insensitively, using this
     order and selecting the FIRST match:
     (a) an EXACT case-insensitive match against a real theme NAME (built-in or user);
     (b) otherwise a BUILT-IN SHORTHAND that omits the `Default ` prefix: `dark` -> `Default Dark`,
         `light` -> `Default Light`, `high contrast` (also `high_contrast` / `high-contrast`)
         -> `Default High Contrast`, `legacy` -> `Default Legacy`, and (added by CR-CH-056)
         `legacy soft` (also `legacy-soft` / `legacy_soft`) -> `Legacy Soft`.
     WHEN a match is found, THE workbench SHALL set it as the active theme and persist the
     selection (Requirement 19.7), identical to selecting it from the Settings menu.

3. WHEN two or more user themes match `<name>` case-insensitively (e.g. differing only by
     letter case), THE workbench SHALL resolve to the FIRST match in the available-themes list
     and SHALL NOT error.

4. WHEN `THEME` is issued with NO argument, THE workbench SHALL navigate the current Workspace
     in place to the Theme Editor context (the same context reached by the Settings `T` /
     Themes option, i.e. `0.T`, and formerly by the removed `THEMES` command), pushing the
     per-tab Navigation_Stack so that END returns one level to the previous context and RETURN
     collapses to the tab root (CR-CH-022). Bare `THEME` SHALL NOT change the active theme. It
     SHALL route through the SAME open-Theme-Editor code path as the Settings Themes option
     (command parity, one code path).

5. WHEN `THEME <name>` is issued and no theme matches by exact name or built-in shorthand, THE
     workbench SHALL LEAVE the active theme unchanged and display the message
     `THEME: '<name>' does not exist` (echoing the entered value).

6. THE Settings menu theme actions SHALL invoke the `THEME` command through the same dispatch
     path used by the typed command (command parity). Until the menu-bar redesign: the
     theme-selection items dispatch `THEME <name>` for their respective themes (the former
     `Legacy (ISPF 3270)` item is relabelled `Default Legacy` and dispatches
     `THEME Default Legacy`); and the "Theme Editor" menu item -- which previously dispatched
     the removed `THEMES` -- dispatches bare `THEME` (opening the Theme Editor per criterion 4).
     The Settings `T` option in `menus/settings.toml` (previously `THEMES`) SHALL likewise
     dispatch `THEME`.

7. WHEN setting the theme via the `THEME` command or the menu fails to persist (e.g. the user
     config location is unavailable or the key is locked), THE workbench SHALL apply the theme
     for the current session AND surface a non-silent message that the change could not be saved
     (no silent revert).

**Added by CR-NR-077 (THEME LIST popup selector).** A `THEME LIST` sub-command opens
an arrow-navigable popup of the available themes, and the Settings menu gains a
Themes submenu. Both are command-parity affordances over the SAME `set_active_theme`
apply path as `THEME <name>` (criterion 17.2).

8. WHEN `THEME LIST` is issued from the command line (matched case-insensitively, BEFORE the
     general `THEME <name>` selector of criterion 17.2 so `LIST` is never treated as a theme
     name), THE workbench SHALL open a Theme_List_Popup: a centred modal overlay listing the
     available themes (built-ins plus user themes, Requirement 14.6) in list order (the order
     returned by the theme-list source, built-ins first). The popup SHALL NOT change the active
     theme merely by opening.

9. WHEN the Theme_List_Popup opens, THE workbench SHALL pre-select the entry whose name equals
     the currently active theme; WHERE the active theme is not in the list, the first entry
     SHALL be selected.

10. WHILE the Theme_List_Popup is open, THE workbench SHALL move the selection with the Down and
     Up arrow keys (wrapping at the ends), SHALL apply the selected theme and close the popup
     when Enter is pressed, and SHALL close the popup WITHOUT changing the active theme when
     Escape is pressed or the user clicks outside the popup. Clicking an entry SHALL apply it
     and close the popup.

11. WHEN a theme is chosen from the Theme_List_Popup, THE workbench SHALL apply and persist it
     through the SAME activation path as `THEME <name>` (criterion 17.2, `set_active_theme`):
     the chosen entry's exact name is used (the popup lists real names, so no shorthand
     resolution is needed), the palette is applied for the session, and the selection is
     persisted (Requirement 19.7). A persistence failure SHALL surface the same non-silent
     message as criterion 17.7.

12. WHILE the Theme_List_Popup is open, THE popup SHALL be treated as a modal overlay for input
     purposes: the shell Tab-order cycle, function-key dispatch, and Ctrl+S SHALL be suppressed
     (consistent with the existing Command_Palette / dialog modal behaviour) so keyboard input
     is consumed by the popup.

13. THE Settings menu SHALL provide a Themes submenu listing the available themes; selecting an
     entry SHALL dispatch the `THEME <name>` command (command parity, architecture-brief
     Principle 2) -- the SAME code path as typing it -- rather than calling the theme setter
     directly. (The submenu is the menu-bar equivalent of the `THEME LIST` popup; it does NOT
     splice into the CR-CH-023 Boundary_Policy Tab ring -- it is an egui-native nested menu.)
---

### Requirement 18: Default Legacy Palette and Reset-to-Default

**User Story:** As a workbench user, I want a canonical built-in theme I can always fall back to (based on the Legacy ISPF look), and a way to reset any theme back to its built-in baseline, so that after experimenting with colours I can always return to a known-good appearance.

**Source:** User requirement (CR-NR-074). "a hardcoded internal theme based on legacy, possibly called 'Default'... people make bad choices so we want them to be able to go back to Default."

**REWORDED by CR-CH-056 (built-in set 4 -> 5; Solarized Dark/Light; Legacy toned +
Legacy Soft).** The built-in set grows to FIVE, and each built-in is an INSTANCE of the
egui-native theme model (Requirement 23): a chrome `egui::Style` plus the retained domain
groups. The five built-ins and their nature:
- `Default Dark` -- now a SOLARIZED DARK instance (replacing Catppuccin Mocha; SAME name
  and `VisualMode::Dark`). Backgrounds base03 `#002B36` / base02 `#073642`; foregrounds
  base0 `#839496` / base1 `#93A1A1`; shared Solarized accents (blue `#268BD2`, cyan
  `#2AA198`, green `#859900`, yellow `#B58900`, orange `#CB4B16`, red `#DC322F`, magenta
  `#D33682`, violet `#6C71C4`).
- `Default Light` -- now a SOLARIZED LIGHT instance (replacing Catppuccin Latte; SAME name
  and `VisualMode::Light`). Backgrounds base3 `#FDF6E3` / base2 `#EEE8D5`; foregrounds
  base00 `#657B83` / base01 `#586E75`; shared Solarized accents.
- `Default High Contrast` -- UNCHANGED (retains its AAA 7:1 contract, Requirement 5.6).
- `Default Legacy` -- the ISPF 3270 retrofit onto the egui `Style`, with its primary
  option-menu background TONED DOWN to `#000060` (Requirement 13.2, amended); all other
  Legacy colours byte-identical. Remains the canonical Fallback_Theme.
- `Legacy Soft` -- NEW: a softer phosphor variant of Legacy that keeps the ISPF semantic
  roles in the domain groups but softens the harshest pure-saturated values (e.g. body
  green `#00FF00` -> `#33FF66`), for comfortable long sessions, WITHOUT altering the
  authentic `Default Legacy`.

Criterion 18.3 is amended below to enumerate FIVE. Criteria 18.1, 18.2, 18.4, 18.5, 18.6
(code-only, read-only, fallback, reset baseline) are UNCHANGED and apply to all five.

#### Acceptance Criteria

**Revised by CR-CH-024 (built-in set consolidated 5 -> 4; superseded for the count by
CR-CH-056, which sets it to 5 -- see the CR-CH-056 banner above).** The separate
`Legacy (ISPF 3270)` built-in is REMOVED; the ISPF 3270 legacy look now lives ONLY under the
name `Default Legacy`. The built-in set is FOUR: `Default Dark`, `Default Light`,
`Default High Contrast`, `Default Legacy`. Rationale: the two entries were byte-identical, so
a separate selectable `Legacy (ISPF 3270)` is redundant -- any theme (including `Default
Legacy`) can be copied to a new named, editable, selectable user theme (Requirement 20.4). No
legacy colour changes; only the redundant name is dropped.

1. THE Theme_System SHALL provide a compiled-in built-in palette named `Default Legacy` carrying
     the ISPF 3270 legacy colours (as defined by Requirement 13 and the legibility fix of
     Requirement 22). It is always available and cannot be removed from the selectable list by
     deleting a disk file. (This is the same palette formerly also exposed under the removed name
     `Legacy (ISPF 3270)`.)
2. THE `Default Legacy` palette SHALL be the canonical Fallback_Theme: WHEN the configured active theme cannot be resolved (missing file, invalid TOML, or unresolved `base`), THE Theme_System SHALL fall back to `Default Legacy` (rather than `Default Dark`) and emit a WARN-level log record naming the unresolved theme. This supersedes Requirement 1.3's "built-in default dark theme" fallback for the file-backed path (Requirement 19); Requirement 1.3 remains the contract for the legacy mode-only path until Requirement 19 is implemented.

   **(CR-CH-019, Option 1; count revised to four by CR-CH-024)** THE built-in palettes are PERMANENT, read-only, COMPILED themes and SHALL NOT be materialised as `.toml` files in the themes directory. They exist only in code. This supersedes any earlier requirement to write built-in theme files to disk (see Requirement 19.2 as revised). Built-ins are the reset baseline and the fallback; the user customises a built-in by copying it to a new named user theme (Requirement 20.4), never by editing the built-in itself.
3. **(Amended by CR-CH-056 to FIVE.)** THE FIVE built-in palettes (`Default Dark`
     [Solarized Dark], `Default Light` [Solarized Light], `Default High Contrast`,
     `Default Legacy` [toned primary-menu], `Legacy Soft`) SHALL all appear in the
     available-themes list (Requirement 14.6) and each SHALL be selectable as the active theme.
     (CR-CH-024: `Legacy (ISPF 3270)` is no longer a separate built-in; `THEME Legacy` and the
     former menu item resolve to `Default Legacy` via the built-in shorthand of Requirement
     17.2b. CR-CH-056: `Legacy Soft` is reachable via its own shorthand, Requirement 17.2b.)
4. THE Theme_System SHALL provide a Reset_Theme operation, given a theme identified by name. **(CR-CH-019, Option 1)** For a BUILT-IN theme, Reset SHALL re-select the compiled built-in palette as the working/active theme (there is no on-disk file to restore, because built-ins are code-only). For a USER theme with a resolvable `base`, Reset SHALL restore the theme's colours to the `base` theme's content. Reset SHALL require confirmation before discarding edits / overwriting a user file.
5. THE compiled built-in palettes SHALL be the immutable reset baseline: resetting to a built-in always yields a palette equal to the compiled built-in palette (a built-in cannot be permanently altered, so a default always stays a default). (Consistent with Requirement 9.2 round-trip when a built-in is copied to a user file.)
6. THE `Default Legacy` name SHALL be stable and reserved: a user-created theme file SHALL NOT be able to shadow or replace the compiled `Default Legacy` fallback used in criterion 2, even if a `default-legacy.toml` on disk is malformed.

---

### Requirement 19: File-Backed Active Theme (Startup and Switch)

**User Story:** As a workbench user, I want the workbench to read which theme I have selected from configuration, load that theme's file, and use it to drive all rendering from then on, so that my saved custom theme is actually used and edits to the file take effect.

**Source:** User requirement (CR-NR-074). "When FFWB starts up it should look for what the configuration says about which theme is in use, and then go looking for the theme file, and from then on use the theme file to guide all rendering." Wires Requirement 1 (Theme_File loading), Requirement 7 (startup loading), and Requirement 14 (custom themes) into the running application.

#### Acceptance Criteria

1. THE Theme_System SHALL manage a `<User_Data_Dir>/themes/` directory. WHEN the workbench starts and this directory does not exist, THE workbench SHALL create it, mirroring the first-launch creation of `<User_Data_Dir>/menus/` (menu-workspace Requirement 4.6). The directory MAY be empty; it holds ONLY user-created themes.
2. **(REVISED by CR-CH-019, Option 1.)** THE workbench SHALL NOT materialise the built-in palettes as `.toml` files. The `themes/` directory contains ONLY user themes created via Copy / Save As (Requirement 20.4/20.5). Built-in themes are compiled-in and read-only (Requirement 18.1/18.2). (This supersedes the earlier requirement to write `default-dark.toml`/`legacy.toml`/etc. on first launch, which caused each built-in to appear twice in the theme list -- once compiled, once as its file -- and made "saving a built-in" ambiguous.)
2a. THE available-themes list (Requirement 14.6) SHALL be de-duplicated by theme NAME: each name appears at most once. WHERE a user `.toml` in `themes/` has the same name as a built-in, the built-in (compiled) entry wins and the user file of that name is ignored for listing (a user cannot shadow a built-in name). No theme SHALL appear more than once in the list.
3. THE configuration SHALL distinguish the active theme (which named theme/file) from the Visual_Mode (dark/light/high-contrast/legacy). The active theme SHALL be identified by a configuration key holding a theme NAME (resolving to a `themes/<slug>.toml` file or a built-in). WHERE the existing `theme.active` key currently holds a MODE string (dark/light/high_contrast/legacy), the gate's design (Requirement 19, design) SHALL define the key(s) so that: (a) the `THEME <name>` command (Requirement 17, name-based per CR-CH-024 -- mode words remain valid as built-in shorthands) and `theme.follow_os` (Requirement 16) continue to work, and (b) no existing persisted config value causes a startup failure (a mode string SHALL resolve to the corresponding built-in theme; a persisted `legacy` resolves to `Default Legacy`).
4. WHEN the workbench starts, THE Theme_System SHALL resolve the active theme name and set `self.palette` from it BEFORE the first frame is rendered (Requirement 7.1). **(CR-CH-019, Option 1.)** WHERE the active theme name is a BUILT-IN name, the compiled built-in palette is used directly (no file read). WHERE it names a USER theme, `themes/<slug>.toml` is loaded and validated via the loader (Requirement 1, resolving `base` per Requirement 14.4/15.5). Either way, all rendering is driven by the resolved palette.
5. WHEN the active theme file is missing or invalid at startup, THE Theme_System SHALL fall back to the `Default Legacy` built-in (Requirement 18.2), apply it for the session, and emit a WARN naming the unresolved theme, WITHOUT crashing.
6. WHEN the active theme's `.toml` file changes on disk while the workbench is running, THE Theme_System SHALL reload it and atomically swap the active palette within one hot-reload cycle (Requirement 7.5/7.6), so edits made in an external editor or by the Theme editor (Requirement 20) take effect without restart.
7. WHEN the user selects a different theme (by name) as active, THE Theme_System SHALL load that theme's file, set it as the active palette, and persist the selection so the same theme is active on the next launch.
8. ALL existing rendering that reads `self.palette` (chrome, menus, POM, editor, focus ring) SHALL be driven by the file-loaded palette with no code change at the call sites (the palette field remains the single source of truth; only its population changes from compiled-only to file-backed).
9. THE `THEME <mode>` command (Requirement 17.2) SHALL continue to select the corresponding built-in-mode theme; when file-backed themes are active, `THEME <mode>` SHALL switch to the built-in theme for that mode (loading its `themes/` file if present, else the compiled palette), preserving command parity and existing behaviour.

---

### Requirement 20: Theme Editor Context

**User Story:** As a workbench user, I want a simple, fast workspace where I can copy an existing theme, change its colours, save it under its own name, and select it for future use, so that I can personalise the appearance without editing TOML by hand or restarting.

**Source:** User requirement (CR-NR-074). "we need to create a workspace where i can copy/change/Save theme's. Once i have saved a theme with it's own name i should be able to select it so that FFWB will use it in the future." Owner directed: keep it simple and fast first; a richer graphical editor can come later.

**REWORKED by CR-CH-056 (edits the egui Style surface + domain groups; drives
import/export; folds in B081).** The Theme Editor is REBUILT to edit the egui-native theme
model (Requirement 23): the editable surface is DERIVED from the egui `Style` / `Visuals`
chrome fields plus the retained domain groups, rather than a hand-written fixed list of
14 tokens. The editor additionally drives IMPORT and EXPORT (Requirement 24) and MAKES THE
CREATE+SAVE FLOW DISCOVERABLE AND FUNCTIONAL, resolving open bug B081 (a custom theme
cannot currently be saved because the Save button is disabled with a pre-filled name and a
built-in selection refuses to write with no prompt). The Theme Editor REMAINS a
`WorkspaceContext` dispatched via `render_workspace_context` (command parity and the single
focus-latch path are unchanged). Criteria 20.3 and 20.5 are amended below; new criteria
20.11-20.13 cover the egui-surface editing, import/export, and the B081 Save fix.

#### Acceptance Criteria

1. THE workbench SHALL provide a Theme_Editor Context (a Workspace, opened via a command so it honours command parity -- architecture-brief Principle 2) that lists the available themes (Requirement 14.6) and lets the user select one to edit.
2. THE Theme_Editor SHALL open via a `THEMES` (or equivalently named) command from any context, and the Settings menu Themes affordance SHALL invoke that same command (same code path as the typed command). WHEN opened from the Home Context (POM), it MAY transform the active POM tab in place (consistent with the Settings/Commands Contexts) so END/RETURN returns to the POM.
3. **(Amended by CR-CH-056.)** THE Theme_Editor SHALL present the selected theme's editable
     surface DERIVED from the egui `Style` / `Visuals` chrome fields (Requirement 23) plus the
     retained domain groups (syntax, gutter, file_tree, decorations, indicators), with each
     colour value shown as `#RRGGBB`/`#RRGGBBAA`, and SHALL allow the user to change a value by
     entering a hex colour (or an appropriate control for non-colour egui fields such as
     rounding/spacing where exposed). Invalid input SHALL be rejected with an inline message and
     SHALL NOT corrupt the theme. (Replaces the former fixed 14-token `ui`/`editor` list.)
4. THE Theme_Editor SHALL provide a Copy_Theme action that creates a new theme initialised from the currently selected theme's colours, prompting for a new unique name; the copy becomes the edit target. This is how a user derives a custom theme from a built-in without altering the built-in.
5. THE Theme_Editor SHALL provide a Save action that writes the edited theme to its `themes/<slug>.toml` file via the serialiser (Requirement 9), and a Save_As action that writes to a new named file. **(REVISED by CR-CH-019, Option 1.)** A BUILT-IN theme cannot be saved over (built-ins are read-only, code-only): WHEN the selected theme is a built-in, Save SHALL be disallowed and SHALL behave as Save_As (prompting for a new user-theme name) OR be disabled with a message directing the user to Copy / Save As. Save and Save_As only ever write USER theme files; they never create or overwrite a built-in. **(Amended by CR-CH-056 / B081.)** The "behave as Save_As (prompting for a new user-theme name)" half SHALL be IMPLEMENTED, not merely specified: pressing Save with a built-in selected SHALL prompt for (or focus a pre-filled, editable) new name and perform Save_As, so creating a user theme from a built-in is a single discoverable step and never a silent no-op (see criterion 20.13).
6. THE Theme_Editor SHALL provide a Set_Active action that makes the selected/edited theme the active theme (Requirement 19.7), applying it immediately (within one frame) and persisting the selection for future launches.
7. THE Theme_Editor SHALL provide a Reset action (Requirement 18.4) that discards in-progress edits and restores the selected theme to its baseline, with confirmation. **(CR-CH-019, Option 1.)** For a built-in theme this re-selects the compiled built-in palette (no file involved); for a user theme with a `base` it restores the `base` colours.
8. WHEN the user edits a colour in the Theme_Editor, THE editor MAY show a live preview by applying the in-progress palette; edits are not persisted until Save/Save_As. Closing the editor without saving SHALL discard unsaved in-progress edits and leave the on-disk file and the active theme unchanged.
9. THE Theme_Editor SHALL surface a contrast advisory (using `check_theme_contrast`, Requirement 5.6/accessibility) for foreground/background pairs that fall below the WCAG AA threshold, as a non-blocking warning, so users are guided away from unreadable combinations.
10. EVERY Theme_Editor action (open, copy, save, save-as, set-active, reset) SHALL be expressible as a command routed through the shell dispatcher (command parity); UI affordances (menu items, buttons) SHALL invoke those commands rather than calling the underlying logic directly.

**Added by CR-CH-056 (Theme Editor rebuild for the egui-native model, import/export, B081).**

11. THE Theme_Editor editable surface SHALL be DERIVED from the egui `Style` / `Visuals`
     chrome fields plus the retained domain groups (Requirement 23), NOT a hand-maintained
     fixed token list; adding or changing an egui chrome field or a domain group SHALL surface
     in the editor without a bespoke per-field edit to the editor's control list. Each editable
     control SHALL round-trip (edit -> working copy -> serialise -> load) without data loss.
12. THE Theme_Editor SHALL provide Export and Import actions (Requirement 24): Export writes
     the active/selected theme to a native FFWB theme file at a user-chosen location; Import
     reads a native FFWB theme file and makes it a selectable user theme. Both actions SHALL be
     expressible as commands routed through the dispatcher (command parity, criterion 20.10);
     the UI affordances SHALL invoke those commands.
13. WHEN the user, with a BUILT-IN theme selected, edits a value and presses Save, THE
     Theme_Editor SHALL make creating a persisted user theme reachable in ONE obvious step --
     by prompting for (or focusing a pre-filled, editable) new unique name and performing
     Save_As -- so the created theme is written to `themes/<slug>.toml`, appears in the theme
     list, and can be Set Active (resolving B081). Save SHALL NEVER be a silent no-op: it either
     writes a user theme or shows a clear, actionable message. A full-shell `egui_kittest` test
     SHALL type a name and press Save/Save_As and assert the user theme file is written and the
     theme becomes selectable (the previously untested interactive render-to-action seam).
---

### Requirement 21: Non-Monochrome Chrome for Dark and Light Themes

**SUPERSEDED by CR-CH-056 (Requirement 23, egui-Native Theme Model).** This requirement
was written entirely in terms of the fixed `ui` / `tab_bar` palette fields
(`ui.panel_bg` / `button_bg` / `input_bg` hierarchy, `tab_bar.active_bg`,
`ui.primary_menu_bg`, `ui.focus_ring`). Those flat fields are replaced by the egui-native
chrome layer, so the contract is RE-EXPRESSED in egui `Visuals` terms in Requirement 23
(criteria 23.7-23.11): the three-level background hierarchy maps onto
`Visuals::panel_fill` / `widgets.*.weak_bg_fill` / `extreme_bg_color`; the accent focus
ring onto `Visuals::selection` / `widgets.active.bg_stroke`; the accent active tab onto
the tab-bar chrome drawn from `Visuals`; and the accent primary-menu band onto the
chrome-layer header fill. The Solarized Dark / Light built-in instances (Requirement 18,
revised by CR-CH-056) MUST satisfy that re-expressed contract and the WCAG AA advisory.
High-Contrast is unchanged (Requirement 23 keeps 21.9). The criteria below are retained
for history and no longer bind as written.

**User Story:** As a workbench user, I want the Dark and Light built-in themes to use accent colour and a sense of depth in the application chrome (the POM, menus, panels, tab bar, title line), so that the interface looks designed and layered rather than a flat wash of one grey.

**Source:** User requirement (CR-CH-020). "the default themes apart from the legacy theme appear very Monochrome". UX evaluation established the cause: the accent is defined but applied only inside the editor; every chrome surface is a near-identical grey. Keeps the Catppuccin base and calm feel.

#### Acceptance Criteria

1. THE Dark and Light built-in palettes SHALL define a THREE-LEVEL background hierarchy so that layered surfaces are visually distinct: a window/base level (`ui.panel_bg`), a raised-surface level (`ui.button_bg`), and an input/inset level (`ui.input_bg`). The three SHALL be perceptibly different (not the near-identical values used before this change) while remaining within the theme's family.
2. THE Dark and Light palettes SHALL apply the theme accent to the keyboard focus ring (`ui.focus_ring` = the theme accent colour), so the focused element is clearly indicated.
3. THE Dark and Light palettes SHALL give the ACTIVE tab an accent-tinted background (`tab_bar.active_bg`) distinct from the inactive-tab background (`tab_bar.inactive_bg`), so the focused Workspace stands out in the tab bar.
4. THE Dark and Light palettes SHALL provide an accent-tinted primary-menu / title-bar background (`ui.primary_menu_bg`) distinct from the base panel background, so the Title_Line reads as a header band rather than blending into the body.
5. WHEN the active theme is NOT the Legacy theme AND NOT the POM's own hardcoded header, THE Title_Line SHALL be painted with the `ui.primary_menu_bg` background and `ui.menu_bar_fg` text (mirroring the Legacy title-line branch), so criterion 4's colour is actually visible. (Render tweak: the non-Legacy title-line branch previously painted no background.)
6. THE tab bar SHALL derive its active/inactive tab background and text from the `tab_bar` palette group (`tab_bar.active_bg`, `tab_bar.inactive_bg`, `tab_bar.active_text`, `tab_bar.inactive_text`) rather than reusing `ui.input_bg`/`ui.panel_bg`/`editor.foreground`, so per-theme tab colours (criterion 3) are honoured. (Render tweak: `render_tab_bar` previously ignored the `tab_bar` group.)
7. FOR ALL foreground/background pairs changed by this requirement (title-bar text on `primary_menu_bg`, active/inactive tab text on their backgrounds), THE contrast SHALL meet WCAG AA: normal text >= 4.5:1; the intentionally-muted inactive-tab text MAY use the >= 3:1 UI-element threshold.
8. THE syntax colour groups, editor colours, and the overall Catppuccin identity of Dark and Light SHALL be preserved; this requirement changes chrome/`ui`/`tab_bar` colours and the accent's reach into the chrome, not the editor content palette.
9. THE High-Contrast theme SHALL be unchanged by this requirement and SHALL continue to meet its AAA (7:1) contrast contract (Requirement 5.6).

---

### Requirement 22: Legacy Theme Blue-on-Black Legibility

**User Story:** As a user of the Legacy (ISPF 3270) theme, I want the blue informational text (line numbers, comments, unknown files, margins) to be readable on the black background, while the theme keeps its authentic ISPF look everywhere else.

**Source:** User requirement (CR-CH-020). The normal-intensity ISPF blue `#0000AA` on black is ~1.58:1 -- effectively unreadable. Real 3270 sessions used the brighter blue for legibility.

#### Acceptance Criteria

1. WHERE the Legacy palette uses the normal-intensity ISPF blue (`#0000AA`, `ISPF_BLUE`) as a FOREGROUND on the black background -- specifically `chrome.line_number_fg`, `chrome.fold_margin_fg`, `chrome.margin_separator`, `syntax.comment`, and `file_tree.unknown` -- THE Legacy palette SHALL instead use the bright ISPF blue (`#7878FF`, `ISPF_BLUE_HI`), which achieves ~5.93:1 on black (WCAG AA).
2. ALL other Legacy colours SHALL remain byte-identical (the ISPF semantic attribute mapping, turquoise input fields, yellow commands, green body text, white headings, blue primary-menu background, etc. are unchanged). This is a targeted legibility fix, not a re-theme.
3. THE `Default Legacy` palette (Requirement 18.1), being a copy of the Legacy palette, SHALL inherit the same legibility fix automatically.
4. THE Legacy `primary_menu_bg` (blue `#0000AA` as a BACKGROUND with white text) SHALL be unchanged -- white on `#0000AA` is a background pairing, not the blue-on-black foreground problem this requirement addresses. **(Note: CR-CH-056 subsequently tones this background down from `#0000AA` to `#000060` per Requirement 13.2 as amended; white text on `#000060` remains a legible background pairing and is not affected by this requirement.)**

---

### Requirement 23: egui-Native Theme Model (CR-CH-056)

**User Story:** As a workbench user and theme author, I want a Theme to configure the full range of the application's actual egui appearance (window and panel fills, per-state widget fills, strokes, rounding, selection, hyperlink, text styles, spacing, shadows) rather than a small fixed slice of chrome colours, so that themes drive the real look of the Windows egui application and the ISPF look is one retrofitted instance rather than the model's defining shape.

**Source:** User requirement (CR-CH-056). "this is not an ISPF application, it is a windows egui application. We need to re-work the whole Theme context and configuration to support all the egui theme functionality. We can retro fit the ISPF look and feel onto it." Scoping report: `.agents/tasks/theme-egui-rework/findings.md`.

**Framework note:** This is an owner-confirmed framework change (framework-conformance.md). It reshapes the public `ThemePalette` type into a HYBRID (an egui-native chrome layer + the retained domain groups) and supersedes Requirement 2 (chrome portions) and Requirement 21. It builds ON the existing single-command-dispatch, per-tab Navigation_Stack, and `WorkspaceContext` mechanisms, which are UNCHANGED.

**Terminology:** egui itself has NO concept of a "theme". Its vocabulary is `egui::Style`, which contains `Visuals` (colours per widget state), `WidgetVisuals`, `Spacing`, text styles, and corner-radius / shadow fields. The user-facing concept remains the word "Theme"; internally a Theme PRODUCES an `egui::Style` for chrome plus the retained domain groups. See the Glossary note at the end of this document.

#### Acceptance Criteria

1. THE Theme model SHALL include a CHROME layer that configures the full themable surface of `egui::Style` / `Visuals` / `WidgetVisuals`: window/panel fills, per-state widget fills (noninteractive / inactive / hovered / active / open) with their `bg_fill`, `weak_bg_fill`, `bg_stroke`, `fg_stroke`, corner radius and expansion; `selection` (fill + stroke); `hyperlink_color`; `extreme_bg_color`, `faint_bg_color`, `code_bg_color`; `warn_fg_color` and `error_fg_color`; window/menu/popup corner radius and shadow; and the `dark_mode` flag matching the Theme's Visual_Mode.
2. THE CHROME layer SHALL be serialised and deserialised using egui's OWN serde `Serialize`/`Deserialize` derives on `Style`/`Visuals` (enabling egui's `serde` feature in the workspace), so the theme file embeds egui's native representation rather than a hand-written mirror that could drift from egui's type.
3. THE Theme model SHALL RETAIN, as first-class groups that egui does NOT model, the DOMAIN groups: `syntax` (token colours), `gutter` (the editor line-number / fold-margin / cursor-row gutter -- the group formerly named `chrome`, renamed to avoid clashing with the new chrome layer), `file_tree` (file-category colours), `decorations` (search/error/warning underlines, change markers, bookmark), `indicators` (find/brace match + user-defined), `style_slots` (the 256-entry Scintilla style table), and `elements` (selection/caret/whitespace/fold element colours with alpha). These groups are read directly by the editor / file-tree / syntax / decoration subsystems and are unaffected by the egui chrome layer.
4. THE user-facing concept SHALL remain the word "Theme". A Theme SHALL carry metadata (`name`, `VisualMode`) and PRODUCE an `egui::Style` (from the chrome layer) plus the domain groups; the application SHALL NOT expose egui's `Style`/`Visuals` vocabulary as the user-facing concept.
5. THE application SHALL apply a Theme to egui through a SINGLE seam -- `WorkbenchShell::apply_theme` (crates/ff-desktop/src/shell/render_theme.rs) -- which SHALL perform a WHOLESALE `apply_to_egui(&mut egui::Style)` using the Theme's chrome layer, replacing the current hand-written body that maps only ~15 of egui's fields and injects hardcoded Legacy slider colours. The hardcoded Legacy slider-colour injection at the seam SHALL be REMOVED (its values become part of the Legacy instance's chrome `Style`).
6. THE Design_Tokens (Requirement 6) spacing scale, border radii, and shadows SHALL be WIRED onto the egui `Style` at the apply seam: spacing -> `Style.spacing` (item_spacing / button_padding / window_margin / indent), radii -> the corner-radius fields of `Visuals` / `WidgetVisuals`, and shadows -> `Visuals.window_shadow` / `popup_shadow`.
7. THE Dark and Light built-in instances (Solarized, Requirement 18 as revised) SHALL present a THREE-LEVEL background hierarchy expressed in egui `Visuals` terms -- a window/base level (`Visuals::panel_fill` / `window_fill`), a raised-surface level (the widget `weak_bg_fill` / inactive `bg_fill`), and an inset level (`extreme_bg_color` for text inputs) -- that are perceptibly distinct (restating old Req 21.1 in egui terms).
8. THE Dark and Light instances SHALL apply the theme accent to the keyboard focus ring expressed in egui terms (`Visuals::selection.stroke` and/or `widgets.active.bg_stroke`), so the focused element is clearly indicated (restating old Req 21.2).
9. THE Dark and Light instances SHALL give the ACTIVE tab an accent-tinted background distinct from the inactive-tab background, drawn from the chrome `Visuals` the tab bar reads, so the focused Workspace stands out (restating old Req 21.3).
10. THE Dark and Light instances SHALL provide an accent-tinted primary-menu / title band distinct from the base panel fill, from the chrome layer, so the Title_Line reads as a header band rather than blending into the body (restating old Req 21.4).
11. FOR ALL foreground/background pairs introduced by the chrome layer (title-band text on the band fill; active/inactive tab text on their fills; body text on `panel_fill`), THE Theme_System SHALL preserve the WCAG AA contrast advisory (normal text >= 4.5:1; the intentionally-muted inactive-tab text MAY use the >= 3:1 UI-element threshold), surfaced through the existing `check_theme_contrast` advisory consumed by the Theme Editor (Requirement 20.9). The Solarized Dark/Light instances SHALL satisfy this advisory with no NEW below-AA text pair; High-Contrast SHALL continue to meet its AAA 7:1 contract unchanged (old Req 21.9 / Req 5.6).

---

### Requirement 24: Theme Import/Export (CR-CH-056)

**User Story:** As a workbench user, I want to export a theme I have made and import a theme that FileForgeWorkbench produced, so that I can back up, move, and share FFWB themes without hand-editing files, using FFWB's own format.

**Source:** User requirement (CR-CH-056). "we stick to Export our theme, import others saved and exported from FFWB." Closed loop in FFWB's native format only; external-format mapping (base16, VS Code, tmTheme) is explicitly OUT of scope here (recorded as a FUTURE possibility).

#### Acceptance Criteria

1. THE Theme_System SHALL provide an EXPORT operation that writes the active (or a selected) Theme to a native FFWB theme file (the versioned TOML format of Requirement 25) at a user-chosen location, using the existing serialiser.
2. THE Theme_System SHALL provide an IMPORT operation that reads a native FFWB theme file (one produced by this application's export or Save) and makes it a selectable USER theme (adding it to the available-themes list, Requirement 14.6), without altering any built-in.
3. WHEN an imported file is invalid, foreign, or not a native FFWB theme file (unparseable, missing required metadata, or an unrecognised structure), THE Theme_System SHALL REJECT it with a clear, non-silent message identifying the problem, SHALL NOT corrupt the existing themes or the active theme, and SHALL leave the themes directory unchanged except for a successful import.
4. EXTERNAL theme-format import (base16 / VS Code / tmTheme or any non-FFWB format) is EXPLICITLY OUT OF SCOPE for CR-CH-056 and SHALL NOT be implemented in this gate; it is recorded as a possible FUTURE requirement.
5. THE Export and Import operations SHALL each be expressible as a command routed through the shell dispatcher (command parity); the Theme Editor UI affordances for Import/Export (Requirement 20.12) SHALL invoke those commands rather than calling the underlying logic directly.

---

### Requirement 25: Versioned Theme File Format and base Resolution (CR-CH-056)

**User Story:** As a workbench user and the maintainer of the theme format, I want theme files to carry a version and to tolerate missing or extra egui fields, and I want `base` theme inheritance to actually work, so that themes written by one version of FFWB keep loading in later versions and inherited themes resolve correctly instead of silently falling back to defaults.

**Source:** User requirement (CR-CH-056) and findings sections D/G. Resolves the discarded-`base` defect (loader.rs reads and drops `base`) and adds the version field for the embedded egui `Style` blob.

#### Acceptance Criteria

1. THE theme file format SHALL carry a top-level `version` field (an integer) recording the format version, and the design SHALL record the egui version the embedded `Style` blob was written against so a future egui upgrade can be detected.
2. WHEN a pre-version theme file (no `version` field -- treated as v1, the legacy section layout) is loaded, THE Theme_System SHALL load it backward-compatibly by mapping the keys it has and FILLING absent chrome/egui fields from the built-in default for the active Visual_Mode (the loader's existing per-token default-fill behaviour, Requirement 1.6), WITHOUT failing.
3. THE embedded egui `Style` sub-table SHALL be deserialised version-tolerantly: missing egui fields SHALL take egui's own defaults (or the Theme's mode default) and EXTRA / unrecognised egui fields SHALL be ignored without error (consistent with Requirement 15.2's unrecognised-section tolerance), so a `Style` written by a different egui version still loads.
4. THE `base` inheritance key SHALL be RESOLVED by the loader (not read and discarded as today): WHEN a theme declares `base = "<theme-name>"`, every token not explicitly defined in the file SHALL be inherited from the named base theme (built-in or previously defined), and only tokens absent from BOTH the file and the base SHALL fall back to the mode default. This makes Requirements 14.4, 14.5, and 15.5 effective rather than aspirational.
5. WHEN a declared `base` theme cannot be resolved, THE Theme_System SHALL emit a WARN-level log record naming the unresolved base and fall back to the built-in default for the unresolved tokens (consistent with Requirements 14.10 and 15.6), without failing the load.
6. THE theme file format SHALL be versioned TOML (the design records that TOML was chosen over JSON for consistency with the existing `menus/*.toml` and config files; this choice is flaggable at gate review). The egui `Style` chrome SHALL be embedded as a TOML sub-table within the theme file.
7. FOR ALL valid Themes in the new model, serialising to the versioned TOML and parsing it back SHALL round-trip to an equivalent Theme (extending Requirement 9.2 to the chrome `Style` sub-table and the `version` / `base` metadata).

---

## Glossary Addendum (CR-CH-056): Theme vs egui Style / Visuals

To keep the specification honest to egui's own terminology:

- **Theme** -- the FFWB USER-FACING concept: a named, selectable, versioned set of visual
  settings. A Theme carries metadata (`name`, `VisualMode`) and PRODUCES an `egui::Style`
  for chrome plus the retained domain groups. "Theme" is the word shown in the UI and used
  in commands (`THEME <name>`).
- **egui `Style`** -- egui's top-level appearance struct. It contains `Visuals`, `Spacing`,
  text styles, and corner-radius / shadow fields. egui has NO "theme" concept; `Style` is
  the closest egui equivalent and is what FFWB's chrome layer configures and applies.
- **`Visuals`** -- the colour portion of `egui::Style`: window/panel fills, per-state
  `WidgetVisuals`, selection, hyperlink, extreme/faint/code backgrounds, warn/error
  colours, shadows, and the `dark_mode` flag.
- **`WidgetVisuals`** -- the per-interaction-state (noninteractive / inactive / hovered /
  active / open) fills, strokes, corner radius, and expansion within `Visuals`.
- **Chrome layer** -- the part of a FFWB Theme that configures `egui::Style` (serialised
  via egui's own serde). "Chrome" here means the application's egui-painted surface.
- **Domain groups** -- the FFWB-specific colour groups egui does NOT model (syntax, gutter,
  file_tree, decorations, indicators, style_slots, elements), retained alongside the chrome
  layer.
- **gutter group** -- the editor line-number / fold-margin / cursor-row gutter colours;
  this is the domain group formerly named `chrome`, RENAMED to `gutter` by CR-CH-056 so it
  does not clash with the new egui chrome layer.
