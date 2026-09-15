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

**Source:** [SCI-STYLE] Element-based colour system -- selection, caret, whitespace, fold, etc.; alpha/transparency support.

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
6. THE Theme_System SHALL expose the list of all available themes (built-in and user-created) through a queryable API so that the Settings Context and View menu can present them as selectable options.
7. WHEN the user changes the active theme via the `theme.active` configuration key (through the Settings Context or by editing the config file), THE Theme_System SHALL load and apply the new theme within one hot-reload cycle without application restart.
8. THE Theme_System SHALL validate every colour token value in a user-created theme file; WHEN an invalid colour format is encountered, THE Theme_System SHALL log a WARN, use the inherited or default value for that token, and continue loading the remainder of the theme.
9. THE Theme_System SHALL provide a `serialise_theme` function that writes the current active palette to a TOML file in the themes directory, enabling users to export and share their customised theme.
10. WHEN a user-created theme file specifies a `base` theme that cannot be resolved, THE Theme_System SHALL emit a WARN-level log record and fall back to the built-in default theme for all unresolved tokens.

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

**User Story:** As a workbench user, I want to change the theme by typing a command on the command line exactly as I can from the Settings menu, so that theme switching honours the command-driven principle (every user action is a command) and is scriptable/automatable.

**Source:** `docs/specs/workbench-requirements-merge/architecture-brief.md` Principle 2 (Command Driven -- "Every user action is a command. Menus, toolbars, keyboard shortcuts, automation scripts and future AI agents invoke commands through the same dispatcher."). Closes the gap where the Settings theme menu called the theme setter directly, bypassing the command layer, and no `THEME` command existed.

#### Acceptance Criteria

1. THE workbench SHALL provide a `THEME` command invocable from the command line in any context.
2. WHEN `THEME <mode>` is issued with a recognised mode name, THE workbench SHALL set the active Visual_Mode to that mode and persist it via `theme.active`, identical to selecting the mode from the Settings menu. Recognised mode names SHALL be `dark`, `light`, `high_contrast` (also accepting `high-contrast`), and `legacy`, matched case-insensitively (consistent with `VisualMode::from_str_loose`).
3. WHEN `THEME` is issued with no argument, THE workbench SHALL report the currently active mode in the status area and SHALL NOT change the theme.
4. WHEN `THEME <arg>` is issued with an unrecognised mode name, THE workbench SHALL display a clear error naming the invalid value and listing the valid modes, and SHALL NOT change the theme.
5. THE Settings menu theme actions (Dark / Light / High Contrast / Legacy) SHALL invoke the `THEME` command through the same dispatch path used by the typed command, so that the menu action and the typed command are the same code path (command parity).
6. WHEN setting the theme via the `THEME` command or the menu fails to persist (e.g. the user config location is unavailable or the key is locked), THE workbench SHALL apply the theme for the current session AND surface a non-silent message that the change could not be saved (no silent revert).
---

### Requirement 18: Default Legacy Palette and Reset-to-Default

**User Story:** As a workbench user, I want a canonical built-in theme I can always fall back to (based on the Legacy ISPF look), and a way to reset any theme back to its built-in baseline, so that after experimenting with colours I can always return to a known-good appearance.

**Source:** User requirement (CR-NR-074). "a hardcoded internal theme based on legacy, possibly called 'Default'... people make bad choices so we want them to be able to go back to Default."

#### Acceptance Criteria

1. THE Theme_System SHALL provide a fifth built-in palette named `Default Legacy` whose colours are identical to the existing `Legacy (ISPF 3270)` palette at the time of definition. It is a compiled-in palette (like the other built-ins) and cannot be deleted from disk in a way that removes it from the selectable list; it is always available.
2. THE `Default Legacy` palette SHALL be the canonical Fallback_Theme: WHEN the configured active theme cannot be resolved (missing file, invalid TOML, or unresolved `base`), THE Theme_System SHALL fall back to `Default Legacy` (rather than `Default Dark`) and emit a WARN-level log record naming the unresolved theme. This supersedes Requirement 1.3's "built-in default dark theme" fallback for the file-backed path (Requirement 19); Requirement 1.3 remains the contract for the legacy mode-only path until Requirement 19 is implemented.

   **(CR-CH-019, Option 1)** THE five built-in palettes are PERMANENT, read-only, COMPILED themes and SHALL NOT be materialised as `.toml` files in the themes directory. They exist only in code. This supersedes any earlier requirement to write built-in theme files to disk (see Requirement 19.2 as revised). Built-ins are the reset baseline and the fallback; the user customises a built-in by copying it to a new named user theme (Requirement 20.4), never by editing the built-in itself.
3. THE five built-in palettes (`Default Dark`, `Default Light`, `Default High Contrast`, `Legacy (ISPF 3270)`, `Default Legacy`) SHALL all appear in the available-themes list (Requirement 14.6) and each SHALL be selectable as the active theme.
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
3. THE configuration SHALL distinguish the active theme (which named theme/file) from the Visual_Mode (dark/light/high-contrast/legacy). The active theme SHALL be identified by a configuration key holding a theme NAME (resolving to a `themes/<slug>.toml` file or a built-in). WHERE the existing `theme.active` key currently holds a MODE string (dark/light/high_contrast/legacy), the gate's design (Requirement 19, design) SHALL define the key(s) so that: (a) the existing `THEME <mode>` command (Requirement 17) and `theme.follow_os` (Requirement 16) continue to work unchanged, and (b) no existing persisted config value causes a startup failure (a mode string SHALL resolve to the corresponding built-in theme).
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

#### Acceptance Criteria

1. THE workbench SHALL provide a Theme_Editor Context (a Workspace, opened via a command so it honours command parity -- architecture-brief Principle 2) that lists the available themes (Requirement 14.6) and lets the user select one to edit.
2. THE Theme_Editor SHALL open via a `THEMES` (or equivalently named) command from any context, and the Settings menu Themes affordance SHALL invoke that same command (same code path as the typed command). WHEN opened from the Home Context (POM), it MAY transform the active POM tab in place (consistent with the Settings/Commands Contexts) so END/RETURN returns to the POM.
3. THE Theme_Editor SHALL present the selected theme's editable colour tokens (at minimum the `ui`, `editor`, and Legacy-semantic colours that drive the visible chrome) with their current values shown as `#RRGGBB`/`#RRGGBBAA`, and SHALL allow the user to change a token's value by entering a hex colour. Invalid hex input SHALL be rejected with an inline message and SHALL NOT corrupt the theme.
4. THE Theme_Editor SHALL provide a Copy_Theme action that creates a new theme initialised from the currently selected theme's colours, prompting for a new unique name; the copy becomes the edit target. This is how a user derives a custom theme from a built-in without altering the built-in.
5. THE Theme_Editor SHALL provide a Save action that writes the edited theme to its `themes/<slug>.toml` file via the serialiser (Requirement 9), and a Save_As action that writes to a new named file. **(REVISED by CR-CH-019, Option 1.)** A BUILT-IN theme cannot be saved over (built-ins are read-only, code-only): WHEN the selected theme is a built-in, Save SHALL be disallowed and SHALL behave as Save_As (prompting for a new user-theme name) OR be disabled with a message directing the user to Copy / Save As. Save and Save_As only ever write USER theme files; they never create or overwrite a built-in.
6. THE Theme_Editor SHALL provide a Set_Active action that makes the selected/edited theme the active theme (Requirement 19.7), applying it immediately (within one frame) and persisting the selection for future launches.
7. THE Theme_Editor SHALL provide a Reset action (Requirement 18.4) that discards in-progress edits and restores the selected theme to its baseline, with confirmation. **(CR-CH-019, Option 1.)** For a built-in theme this re-selects the compiled built-in palette (no file involved); for a user theme with a `base` it restores the `base` colours.
8. WHEN the user edits a colour in the Theme_Editor, THE editor MAY show a live preview by applying the in-progress palette; edits are not persisted until Save/Save_As. Closing the editor without saving SHALL discard unsaved in-progress edits and leave the on-disk file and the active theme unchanged.
9. THE Theme_Editor SHALL surface a contrast advisory (using `check_theme_contrast`, Requirement 5.6/accessibility) for foreground/background pairs that fall below the WCAG AA threshold, as a non-blocking warning, so users are guided away from unreadable combinations.
10. EVERY Theme_Editor action (open, copy, save, save-as, set-active, reset) SHALL be expressible as a command routed through the shell dispatcher (command parity); UI affordances (menu items, buttons) SHALL invoke those commands rather than calling the underlying logic directly.
---

### Requirement 21: Non-Monochrome Chrome for Dark and Light Themes

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
4. THE Legacy `primary_menu_bg` (blue `#0000AA` as a BACKGROUND with white text) SHALL be unchanged -- white on `#0000AA` is a background pairing, not the blue-on-black foreground problem this requirement addresses.
