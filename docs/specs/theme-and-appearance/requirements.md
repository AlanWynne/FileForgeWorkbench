# Requirements Document

## Introduction

This feature specifies the theme and appearance subsystem for FileForgeWorkbench (`ff-theme` crate). The theme system is the **central visual identity layer** for the entire workbench platform. It manages colours, fonts, design tokens, and visual mode switching (dark/light/high-contrast) through a TOML-based theme configuration format. All rendering code obtains colour values, font selections, and spacing metrics through the theme system rather than using hardcoded values.

The theme system replaces all hardcoded colour values with semantic token lookups, provides a structured palette covering every visual element (editor, syntax, file tree, tab bar, chrome, decorations, indicators), supports multiple font stacks (monospace for editor content, proportional for UI elements), and exposes a design system for consistent spacing, border radii, shadows, and animations across the entire workbench.

The `ff-theme` crate is a Wave 6 (UI and Rendering) component. It depends on `configuration-system` for TOML-based configuration loading, layered overrides, and hot-reload. It is consumed by all rendering subsystems: `menu-and-statusbar`, `text-decorations`, `whitespace-and-guides`, `caret-and-selection`, `syntax-highlighting`, `file-tree-panel`, `layout-and-docking`, and the GUI shell.

**Source references:**
- **[FFE-THEME-1]** through **[FFE-THEME-7]** = FileForgeEditor `theme-and-appearance` specification (7 requirements — theme file, palette, fonts, loading, colour replacement, serialisation, extensibility)
- **[SCI-STYLE]** = Scintilla `ViewStyle` / `Style` / `ElementMap` — 256 style slots with font/fore/back/bold/italic/underline/case, element-based colour system (selection, caret, whitespace, fold, etc.), zoom level, alpha/transparency support
- **[WB]** = Workbench Architecture Brief — dark/light/high-contrast modes, design system (spacing, radii, shadows, animations), plugin-provided theme extensions, hot-reload, multiple font stacks

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
- **Visual_Mode**: One of three appearance modes — Dark, Light, or High-Contrast — that determines which set of palette values is active. [WB]
- **Style_Slot**: An indexed slot (0–255) defining a combination of font, foreground colour, background colour, and text attributes (bold, italic, underline, case) for a specific syntax or UI element. Adapted from Scintilla's 256-style system. [SCI-STYLE]
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

---

### Requirement 2: Theme Palette Structure

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
9. FOR ALL Colour_Token values in the Theme_Palette, THE Theme_System SHALL represent each colour as an RGBA quadruplet with red, green, blue components in the range 0–255 and an alpha component in the range 0–255 (where 255 is fully opaque).
10. THE Theme_Palette SHALL support alpha/transparency on tokens where translucent rendering is semantically meaningful (selection background, indicator overlays, caret-line background), as indicated by a per-token `allows_translucent` flag.

---

### Requirement 3: Style Slots

**User Story:** As a syntax-highlighting engine, I need indexed style slots that define the visual attributes for each token type, so that I can efficiently map lexer output to rendering instructions.

**Source:** [SCI-STYLE] 256 style slots with font, fore, back, bold, italic, underline, case.

#### Acceptance Criteria

1. THE Theme_System SHALL provide a style-slot table containing up to 256 indexed Style_Slot entries (indices 0–255).
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
6. WHEN a configured font size is outside the valid range of 6.0–72.0 points, THE Theme_System SHALL log a warning and clamp the value to the nearest boundary (6.0 or 72.0).
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
7. THE Theme_System SHALL expose Design_Token values through typed accessor methods that return the appropriate numeric or structured type (e.g., `spacing(SpacingLevel) → f32`, `border_radius(RadiusLevel) → f32`).

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

**Source:** [SCI-STYLE] Element-based colour system — selection, caret, whitespace, fold, etc.; alpha/transparency support.

#### Acceptance Criteria

1. THE Theme_System SHALL provide an element-colour API: `element_colour(element: Element) → Option<ColourRGBA>` that returns the colour for a named UI element, or `None` if no colour is set for that element (indicating the element should not be rendered or should use a computed default).
2. THE Theme_System SHALL define elements for at minimum: selection background, selection foreground, additional-selection background, additional-selection foreground, caret foreground, additional-caret foreground, caret-line background, whitespace foreground, whitespace background, fold-line colour, fold-line-highlight colour, and hidden-line indicator colour.
3. WHEN an element colour has an alpha component less than 255, THE Rendering_Code SHALL use alpha-blended rendering for that element, compositing over the underlying content.
4. THE Theme_System SHALL track which elements allow translucent rendering (via `element_allows_translucent(element) → bool`); elements not in the translucent set SHALL have their alpha forced to 255.
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

#### Acceptance Criteria

1. WHEN the Legacy theme is active, THE menu bar top-level item text SHALL be rendered in white (`#FFFFFF`).
2. WHEN the Legacy theme is active, THE primary menu (screen title / heading row on any screen) SHALL be rendered with a blue background (`#0000AA`) and white text.
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
6. THE Theme_System SHALL expose the list of all available themes (built-in and user-created) through a queryable API so that the Settings panel and View menu can present them as selectable options.
7. WHEN the user changes the active theme via the `theme.active` configuration key (through the Settings panel or by editing the config file), THE Theme_System SHALL load and apply the new theme within one hot-reload cycle without application restart.
8. THE Theme_System SHALL validate every colour token value in a user-created theme file; WHEN an invalid colour format is encountered, THE Theme_System SHALL log a WARN, use the inherited or default value for that token, and continue loading the remainder of the theme.
9. THE Theme_System SHALL provide a `serialise_theme` function that writes the current active palette to a TOML file in the themes directory, enabling users to export and share their customised theme.
10. WHEN a user-created theme file specifies a `base` theme that cannot be resolved, THE Theme_System SHALL emit a WARN-level log record and fall back to the built-in default theme for all unresolved tokens.

---

### Requirement 12: Extensibility and Forward Compatibility

**User Story:** As a workbench developer, I want the theme system to be extensible so that new UI features can introduce additional tokens without modifying the core theme structure, and old theme files remain loadable.

**Source:** [FFE-THEME-7] Extensibility for Future Features.

#### Acceptance Criteria

1. THE Theme_Palette data structure SHALL be organised into clearly named sub-structures (one per colour group) so that new groups can be added without modifying existing groups.
2. WHEN a theme TOML file contains sections or keys not recognised by the current version of the Theme_System, THE Theme_System SHALL ignore unrecognised sections without error, preserving them during serialisation round-trips.
3. THE Theme_System SHALL expose the Theme_Palette through a shared, thread-safe reference accessible from any rendering component, any subsystem holding a reference to the workbench context, and any plugin via its PluginContext.
4. THE Theme_System SHALL provide a stable public API surface such that adding new token groups or design tokens does not require changes to existing consumers (backward-compatible extension).
5. THE Theme_System SHALL support theme inheritance: a theme file MAY declare a `base` theme name, and all tokens not explicitly defined in the file SHALL be inherited from the base theme rather than from the built-in defaults.
6. WHEN a theme file specifies a `base` theme that cannot be found, THE Theme_System SHALL emit a WARN-level log record and fall back to the built-in default theme for unresolved tokens.

