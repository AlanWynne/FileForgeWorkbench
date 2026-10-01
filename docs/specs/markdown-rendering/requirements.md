# Requirements Document

## Introduction

This feature specifies the **Markdown Rendering configuration** for FileForgeWorkbench. It defines how Markdown elements are styled when a Markdown document is rendered, through a TOML-based configuration that is editable, discoverable, and hot-reloadable using the same configuration machinery as the theme system.

Today Markdown is rendered with bare library defaults: `ff-mdx-app` draws documents through `egui_commonmark::CommonMarkViewer::new()` with no style configuration, and the HTML path (`ff-md-viewer::render_to_html`, consumed by `ff-html-export` and `ff-pdf-export`) emits an HTML fragment with no attached styling. The result is a plain, unconfigurable appearance. This feature introduces a single source of truth for Markdown presentation that drives **both** render paths so they cannot drift.

The configuration follows a **"compose, do not absorb"** relationship with the theme system (`theme-and-appearance`): Markdown-specific **colours** reference existing `ff-theme` Colour_Tokens so the Markdown view stays coherent with the active theme and participates in Visual_Mode switching (dark / light / high-contrast / legacy), while the Markdown configuration **owns** its Markdown-specific **typography** and **layout** tokens (heading scale, element spacing, list indent, table borders, maximum content width) that have no editor-palette equivalent.

The configuration is consumed by **both** FileForgeWorkbench (the in-shell Markdown viewer) and the standalone Markdown Explorer `ffmdx` (`ff-mdx-app`). The proposed implementation crate is `ff-md-style` (name to be confirmed in `design.md`), holding the schema, built-in default, loader, egui-style mapping, and CSS emitter.

**Source references:**
- **[OWNER-MDR]** = Owner prompt (Phase markdown-rendering): "looking at a markdown file in ffmdx, i am disappointed by the look ... create a toml file to configure how ffmdx renders the markdown files ... create a new markdown-rendering sub-project ... ultimately used in both FFWB and FFMDX."
- **[THEME]** = `theme-and-appearance` requirements -- Colour_Token model, Visual_Modes, configuration loading, hot-reload, serialisation round-trip.
- **[CFG]** = `configuration-system` requirements -- TOML loading, layered overrides, hot-reload notifications, schema validation.
- **[CMARK]** = `egui_commonmark` / `pulldown-cmark` -- the two existing render engines whose output this configuration styles.

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `theme-and-appearance` | **Dependency** | Markdown colour tokens reference `ff-theme` Colour_Tokens; the Markdown view participates in Visual_Mode switching and hot-reload through the theme/palette-change notification. |
| `configuration-system` | **Dependency** | Provides TOML loading, layered overrides, hot-reload notifications, and schema validation for the Markdown rendering configuration file. |
| `ffmdx-app` | **Consumer** | The standalone `ffmdx` binary applies this configuration to its egui `CommonMarkViewer` and its HTML/PDF export. |
| `custom-file-viewers` | **Consumer** | The in-shell Markdown viewer (`ff-viewers` / `ff-mdx-plugin`) applies this configuration to its egui and HTML render paths. |
| `fileforge-integration` | **Consumer** | FFWB's embedded Markdown rendering (help, previews) obtains element styling from this configuration. |
| `context-help` | **Related** | If help content is rendered via the Markdown viewer (CR-NR-100), it inherits this element styling. |

## Glossary

- **Markdown_Style_Config**: The in-memory data structure holding the complete set of resolved Markdown element styling values (typography, layout, colour-token references), loaded from the Markdown_Style_File. [OWNER-MDR]
- **Markdown_Style_File**: A TOML file defining how Markdown elements are styled. Discoverable and editable, loaded through the configuration-system with layered-override and hot-reload semantics. [OWNER-MDR, CFG]
- **Element_Style**: The styling attributes for one class of Markdown element (e.g. a heading level, blockquote, code block, table), covering typography and, where applicable, a Colour_Token reference. [OWNER-MDR]
- **Heading_Scale**: The ordered set of font sizes (and optional weight/colour) for heading levels H1 through H6. [OWNER-MDR]
- **Colour_Token_Reference**: A named reference to an `ff-theme` Colour_Token (e.g. `editor.accent`) used wherever a Markdown element needs a colour, so the colour tracks the active theme and Visual_Mode rather than being hardcoded in the Markdown configuration. [THEME]
- **Layout_Token**: A Markdown-specific non-colour value owned by this configuration: maximum content width, paragraph/element spacing, list indent, table cell padding, code-block padding. [OWNER-MDR]
- **Render_Path**: One of the two engines that turns Markdown into output: the egui path (`egui_commonmark::CommonMarkViewer`) and the HTML path (`ff-md-viewer::render_to_html`). [CMARK]
- **Egui_Style_Mapping**: The transformation of a Markdown_Style_Config into the style inputs the egui `CommonMarkViewer` consumes for a frame. [CMARK]
- **CSS_Emission**: The transformation of a Markdown_Style_Config into a CSS stylesheet attached to the HTML produced by the HTML Render_Path (for HTML and PDF export). [CMARK]
- **Built_In_Default**: The compiled-in default Markdown_Style_Config used when no Markdown_Style_File is present or when individual tokens are omitted. [OWNER-MDR]
- **Code_Block_Theme**: The styling applied to fenced code blocks, including background (a Colour_Token_Reference), padding, font, and an optional syntax-highlighting colour set for recognised languages. [OWNER-MDR]

## Requirements

### Requirement 1: Markdown Style Configuration File

**User Story:** As a workbench user, I want to define how Markdown elements are rendered in a TOML file, so that I can control the appearance of Markdown documents without modifying source code and share the configuration between FFWB and ffmdx.

**Source:** [OWNER-MDR]; [CFG] TOML-based configuration.

#### Acceptance Criteria

1. THE Markdown rendering subsystem SHALL load a Markdown_Style_Config from a TOML Markdown_Style_File managed by the configuration-system (e.g. `markdown.toml` under the config path).
2. WHEN no Markdown_Style_File exists, THE subsystem SHALL use the Built_In_Default Markdown_Style_Config and emit a DEBUG-level log record indicating the default was used.
3. WHEN a Markdown_Style_File contains invalid TOML syntax, THE subsystem SHALL log a warning identifying the file and parse error, retain the previously active configuration (or the Built_In_Default if none), and continue operating.
4. WHEN a Markdown_Style_File contains a valid TOML structure with individual invalid values (out-of-range size, unknown Colour_Token name, negative spacing), THE subsystem SHALL log a warning for each invalid value and use the Built_In_Default for that specific token.
5. THE Markdown_Style_File format SHALL support partial definitions, where any omitted token inherits its value from the Built_In_Default.
6. THE subsystem SHALL load the Markdown_Style_Config through the configuration-system API, participating in the layered override model (user-layer, project-layer, profile-layer) so that overrides function correctly.
7. THE Markdown_Style_File SHALL be editable by hand and SHALL round-trip (parse-then-serialise) without loss of defined values, so that a settings UI or theme editor can safely read and write it.

---

### Requirement 2: Heading Typography

**User Story:** As a workbench user, I want to configure the size, weight, and colour of each heading level, so that document structure is visually clear and matches my preferences.

**Source:** [OWNER-MDR].

#### Acceptance Criteria

1. THE Markdown_Style_Config SHALL define a Heading_Scale with an independently configurable Element_Style for each heading level H1 through H6.
2. EACH heading Element_Style SHALL support at minimum: font size (points), bold flag, italic flag, and an optional Colour_Token_Reference for the heading text colour.
3. WHEN a heading level's Colour_Token_Reference is omitted, THE subsystem SHALL render that heading using the body text colour token.
4. WHEN a heading level's font size is omitted, THE subsystem SHALL use the Built_In_Default size for that level, where the Built_In_Default sizes decrease monotonically from H1 to H6.
5. WHEN a configured heading font size is outside the valid range of 6.0 to 72.0 points, THE subsystem SHALL log a warning and clamp the value to the nearest boundary.
6. THE subsystem SHALL support an optional per-level configuration to render a horizontal rule beneath H1 and H2 (a common Markdown convention), defaulting to enabled for H1 and H2 and disabled for H3 through H6.

---

### Requirement 3: Body, Paragraph, and Inline Text

**User Story:** As a workbench user, I want to configure the base body font, size, and inline text decorations, so that running text is comfortable to read.

**Source:** [OWNER-MDR]; [THEME] font stacks.

#### Acceptance Criteria

1. THE Markdown_Style_Config SHALL define a body Element_Style specifying font family (or a reference to the theme proportional Font_Stack), base font size, and a Colour_Token_Reference for body text colour (defaulting to the editor foreground token).
2. WHEN the body font family is omitted, THE subsystem SHALL use the theme proportional Font_Stack.
3. THE Markdown_Style_Config SHALL define Element_Styles for inline emphasis (italic), strong emphasis (bold), and strikethrough, each applying the corresponding text attribute to the body style.
4. THE Markdown_Style_Config SHALL define an inline-code Element_Style specifying a monospace font (or a reference to the theme monospace Font_Stack), a background Colour_Token_Reference, and a foreground Colour_Token_Reference.
5. THE Markdown_Style_Config SHALL define a link Element_Style specifying a Colour_Token_Reference (defaulting to the editor accent token) and an underline flag.
6. WHEN a configured body or inline font size is outside the valid range of 6.0 to 72.0 points, THE subsystem SHALL log a warning and clamp the value to the nearest boundary.

---

### Requirement 4: Lists and Blockquotes

**User Story:** As a workbench user, I want to configure list indentation and blockquote styling, so that nested and quoted content is clearly distinguished.

**Source:** [OWNER-MDR].

#### Acceptance Criteria

1. THE Markdown_Style_Config SHALL define a Layout_Token for list indent width (logical pixels) applied per nesting level for both ordered and unordered lists.
2. THE Markdown_Style_Config SHALL define a Layout_Token for vertical spacing between list items.
3. THE Markdown_Style_Config SHALL define a blockquote Element_Style specifying a left border width, a border Colour_Token_Reference, a background Colour_Token_Reference, inner padding, and an optional italic flag for quoted text.
4. WHEN a Layout_Token for indent or spacing is omitted or negative, THE subsystem SHALL use the Built_In_Default value and, for a negative value, log a warning.
5. THE subsystem SHALL render task-list checkboxes (`- [ ]` / `- [x]`) using a configurable Element_Style, defaulting to the body text colour for the marker.

---

### Requirement 5: Fenced Code Blocks and Syntax Highlighting

**User Story:** As a workbench user, I want fenced code blocks to be visually distinct and optionally syntax-highlighted, so that embedded code is readable within a Markdown document.

**Source:** [OWNER-MDR].

#### Acceptance Criteria

1. THE Markdown_Style_Config SHALL define a Code_Block_Theme specifying a background Colour_Token_Reference (defaulting to the editor background token), a foreground Colour_Token_Reference, inner padding, and a monospace font (or a reference to the theme monospace Font_Stack).
2. THE Code_Block_Theme SHALL support an optional syntax-highlighting colour set keyed by token kind (keyword, comment, string, number, type, function, operator) using Colour_Token_References into the theme syntax group.
3. WHEN syntax highlighting is enabled AND a fenced code block declares a recognised language, THE subsystem SHALL apply the syntax-highlighting colour set to that block.
4. WHEN syntax highlighting is disabled OR the fenced code block declares no language OR the language is unrecognised, THE subsystem SHALL render the block using the Code_Block_Theme foreground colour without per-token colouring.
5. THE subsystem SHALL expose a configuration flag to enable or disable code-block syntax highlighting, defaulting to enabled.
6. THE subsystem SHALL render a code block identically in intent across both Render_Paths (egui and HTML), subject to each engine's capabilities (see Requirement 9).

---

### Requirement 6: Tables and Horizontal Rules

**User Story:** As a workbench user, I want to configure table borders, header styling, and cell padding, so that tabular data is legible.

**Source:** [OWNER-MDR].

#### Acceptance Criteria

1. THE Markdown_Style_Config SHALL define a table Element_Style specifying border width, a border Colour_Token_Reference, cell padding (Layout_Token), and a header-row Element_Style (bold flag and an optional background Colour_Token_Reference).
2. THE subsystem SHALL support an optional zebra-striping flag for alternate table rows, specifying the stripe background as a Colour_Token_Reference, defaulting to disabled.
3. THE Markdown_Style_Config SHALL define a horizontal-rule Element_Style specifying thickness and a Colour_Token_Reference.
4. WHEN a table border width or cell padding is omitted or negative, THE subsystem SHALL use the Built_In_Default value and, for a negative value, log a warning.

---

### Requirement 7: Layout and Content Width

**User Story:** As a workbench user, I want to control the maximum content width and element spacing, so that long documents do not stretch uncomfortably wide and vertical rhythm is consistent.

**Source:** [OWNER-MDR].

#### Acceptance Criteria

1. THE Markdown_Style_Config SHALL define a Layout_Token for maximum content width in logical pixels, beyond which rendered content is horizontally constrained (and centred, per Requirement 7.4).
2. WHEN the maximum content width is unset or zero, THE subsystem SHALL render content using the full available width of the viewport.
3. THE Markdown_Style_Config SHALL define Layout_Tokens for vertical spacing between block-level elements (paragraphs, headings, lists, code blocks, tables, blockquotes).
4. WHEN a maximum content width is set and the viewport is wider than it, THE subsystem SHALL centre the rendered content within the available width.
5. WHEN a Layout_Token value is outside a sane range (negative, or a width below a documented minimum of 200 logical pixels when non-zero), THE subsystem SHALL log a warning and clamp to the nearest valid boundary.

---

### Requirement 8: Theme Colour Sourcing (Compose, Do Not Absorb)

**User Story:** As a workbench developer, I want Markdown element colours to come from the theme palette by reference, so that the Markdown view stays coherent with the active theme and Visual_Mode without duplicating colour definitions.

**Source:** [OWNER-MDR] "compose, do not absorb"; [THEME] Colour_Token model, Visual_Modes.

#### Acceptance Criteria

1. WHEREVER a Markdown Element_Style requires a colour, THE Markdown_Style_Config SHALL express it as a Colour_Token_Reference naming an `ff-theme` Colour_Token, NOT as a literal colour value.
2. WHEN resolving a Colour_Token_Reference, THE subsystem SHALL obtain the current RGBA value from the active theme palette for the active Visual_Mode.
3. WHEN the active Visual_Mode changes (dark / light / high-contrast / legacy), THE subsystem SHALL re-resolve all Colour_Token_References so Markdown rendering tracks the mode change within the frame budget defined by the theme system.
4. WHEN a Colour_Token_Reference names a token that does not exist in the theme palette, THE subsystem SHALL log a warning identifying the invalid reference and fall back to a documented default token (body foreground, or editor background for backgrounds).
5. THE Markdown_Style_Config SHALL NOT define a parallel colour palette, Visual_Mode set, or theme-loading mechanism; it SHALL rely entirely on the theme system for colour values and mode switching.
6. THE Markdown_Style_Config MAY define Markdown-specific NON-colour tokens (Heading_Scale sizes, Layout_Tokens, border widths, flags) directly, as these have no theme-palette equivalent.

---

### Requirement 9: Both Render Paths Driven by One Configuration

**User Story:** As a workbench developer, I want a single Markdown_Style_Config to drive both the egui viewer and the HTML export, so that the two render paths cannot drift apart in appearance.

**Source:** [OWNER-MDR]; [CMARK] two render engines.

#### Acceptance Criteria

1. THE subsystem SHALL provide an Egui_Style_Mapping that transforms a Markdown_Style_Config into the style inputs consumed by the egui `CommonMarkViewer` Render_Path used by `ff-mdx-app` and the in-shell Markdown viewer.
2. THE subsystem SHALL provide a CSS_Emission that transforms a Markdown_Style_Config into a CSS stylesheet attached to the HTML produced by the `ff-md-viewer` Render_Path, consumed by `ff-html-export` and `ff-pdf-export`.
3. FOR ALL element classes defined in the Markdown_Style_Config (headings, body, inline, lists, blockquotes, code blocks, tables, rules, links, layout), BOTH the Egui_Style_Mapping and the CSS_Emission SHALL derive their output from the SAME Markdown_Style_Config instance, with no element styled by only one path.
4. WHERE a styling attribute cannot be faithfully reproduced by a given Render_Path (an egui or CSS capability gap), THE subsystem SHALL document the limitation and apply the nearest supported approximation rather than silently ignoring the attribute.
5. WHEN the Markdown_Style_Config changes (edit, hot-reload, or Visual_Mode change), BOTH Render_Paths SHALL use the updated configuration on their next render without an application restart.
6. THE CSS_Emission SHALL produce a self-contained stylesheet (no external resource references) so that exported HTML and PDF render consistently offline.

---

### Requirement 10: Hot-Reload and Change Notification

**User Story:** As a workbench user, I want changes to the Markdown style file to take effect without restarting the application, so that I can iterate on the appearance quickly.

**Source:** [OWNER-MDR]; [CFG] hot-reload; [THEME] palette-change notification.

#### Acceptance Criteria

1. THE subsystem SHALL register a hot-reload callback with the configuration-system so that changes to the Markdown_Style_File trigger a reload of the Markdown_Style_Config without an application restart.
2. WHEN a hot-reload is triggered, THE subsystem SHALL atomically swap the active Markdown_Style_Config so that no frame is rendered using a mix of old and new values.
3. THE subsystem SHALL re-resolve Colour_Token_References on every theme palette-change notification, so that a Visual_Mode or theme switch updates Markdown colours without a Markdown_Style_File change.
4. THE subsystem SHALL emit a change notification when the active Markdown_Style_Config changes (edit, hot-reload, or re-resolution) so that consumers can invalidate caches or trigger a re-render.
5. WHEN a hot-reload loads an invalid Markdown_Style_File, THE subsystem SHALL retain the previously active configuration and log a warning, per Requirement 1.3.

---

### Requirement 11: Built-In Default and Discoverability

**User Story:** As a workbench user, I want a good-looking default out of the box and an easy way to discover and edit the configuration, so that I benefit immediately and can customise when I choose to.

**Source:** [OWNER-MDR].

#### Acceptance Criteria

1. THE subsystem SHALL ship a compiled Built_In_Default Markdown_Style_Config that produces a visually appropriate rendering without any Markdown_Style_File present.
2. THE Built_In_Default SHALL reference theme Colour_Tokens for all colours, so the default rendering is coherent with the active theme and Visual_Mode.
3. THE subsystem SHALL provide a mechanism to write the current effective Markdown_Style_Config to a Markdown_Style_File (a "export defaults" affordance), with descriptive comments for each section, so a user can start from a complete, documented file.
4. THE subsystem SHALL discover a user-provided Markdown_Style_File in the config path and make it the active configuration on the next hot-reload cycle or application start, without any code change.
5. THE Built_In_Default SHALL NOT be written to disk automatically on first launch (consistent with the workbench code-only-built-in convention); it is used in memory unless the user explicitly exports or creates a file.
