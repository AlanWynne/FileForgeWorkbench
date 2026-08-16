# Requirements Document

## Introduction

This feature specifies the whitespace-and-guides subsystem for FileForgeWorkbench (`ff-whitespace-guides` crate). The whitespace-and-guides subsystem provides **visual indicators for invisible characters, structural indentation, column boundaries, and line-wrap continuation**. These are non-content visual elements rendered over or alongside the text to aid readability, code navigation, and adherence to formatting standards.

The subsystem covers four related concerns:

1. **Whitespace visibility** — rendering dots for spaces and arrows/strikeouts for tabs, with configurable visibility modes.
2. **Indent guides** — vertical guide lines at each indentation level, with active-block highlighting.
3. **Edge column indicator** — vertical line(s) or background shading at configurable column positions (e.g., column 80) to signal line-length boundaries.
4. **Wrap markers** — small visual indicators drawn at the start or end of wrapped sub-lines to distinguish soft wraps from hard line breaks.

All settings are stored in the `configuration-system` and themed via `theme-and-appearance`. The model is GUI-independent: this crate defines the settings, modes, and per-line metadata; rendering is delegated to the GUI shell. Toggle commands are registered with the `command-framework`.

**Source references:**
- **[SCI-VS-9]** = Scintilla `ViewStyle` — `viewWhitespace`, `tabDrawMode`, `whitespaceSize`, WhiteSpace element colour/alpha
- **[SCI-VS-10]** = Scintilla `ViewStyle` / `WrapAppearance` — `WrapVisualFlag`, `WrapVisualLocation`, `WrapIndentMode`, `visualStartIndent`
- **[SCI-EV-GUIDE]** = Scintilla `EditView` — `DrawIndentGuide`, `DrawIndentGuidesOverEmpty`, `pixmapIndentGuide`, `pixmapIndentGuideHighlight`, `IndentView` enum, `STYLE_INDENTGUIDE`, `SetHighlightGuide`
- **[SCI-VS-EDGE]** = Scintilla `ViewStyle` — `edgeState` (`EdgeVisualStyle`), `theEdge`, `theMultiEdge`, `EdgeProperties`, `SetEdgeColumn`, `MultiEdgeAddLine`
- **[WB]** = Workbench Architecture Brief (GUI-independent core, configuration as data, command-driven architecture)

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `theme-and-appearance` | **Dependency** | Provides element colours, alpha, and style definitions for whitespace glyphs, indent guide lines, edge indicator colour, and wrap marker colour. All visual attributes are resolved from the active theme. |
| `configuration-system` | **Dependency** | Stores all whitespace, indent guide, edge, and wrap marker settings as TOML configuration keys (e.g., `editor.whitespace_mode`, `editor.edge_column`). Hot-reload propagates changes without restart. |
| `display-line-mapping` | **Consumer** | Provides wrap height information and sub-line identification used to determine where wrap markers are rendered and how indent guides span wrapped sub-lines. |
| `document-model` | **Dependency** | Provides line content (for indent level computation), tab size, and indent size used to position indent guides and edge column indicators. |
| `command-framework` | **Integration** | Toggle commands for whitespace visibility, indent guide visibility, and edge column on/off are registered as commands with keybindings and menu entries. |
| `line-wrap-toggle` | **Related** | Controls whether word wrap is active; wrap markers are only meaningful when word wrap is enabled. |

## Glossary

- **Whitespace_Visibility**: A mode controlling whether invisible whitespace characters (spaces and tabs) are rendered with visible glyphs. Modes: Invisible, VisibleAlways, VisibleAfterIndent, VisibleOnlyInIndent. [SCI-VS-9]
- **Whitespace_Glyph**: The visual marker drawn for a whitespace character — a centred dot for a space, an arrow or strikeout for a tab. [SCI-VS-9]
- **Whitespace_Size**: A configurable integer (in pixels or logical units) controlling the size of the dot/arrow glyph rendered for visible whitespace. [SCI-VS-9]
- **Tab_Draw_Mode**: The style used to render visible tab characters. Modes: LongArrow (arrow spanning the full tab width) or Strikeout (horizontal line through the tab span). [SCI-VS-9]
- **Indent_Guide**: A thin vertical line drawn at each tab-stop column within the indentation area of a line, indicating structural nesting depth. [SCI-EV-GUIDE]
- **Indent_Guide_Mode**: Controls which lines display indent guides. Modes: None, Real (only on lines with actual indentation), LookForward (extend guides through blank lines by looking ahead), LookBoth (extend guides through blank lines by looking forward and backward). [SCI-EV-GUIDE]
- **Active_Indent_Guide**: The indent guide column currently highlighted because the caret or a brace-match falls within that indentation block. Drawn in a distinct highlight colour. [SCI-EV-GUIDE]
- **Edge_Column**: A column position at which a visual indicator is drawn to show a line-length boundary (e.g., 80 or 120 characters). [SCI-VS-EDGE]
- **Edge_Mode**: The rendering style for the edge indicator. Modes: None, Line (vertical line at the column), Background (columns beyond the edge are shaded), MultiLine (multiple edge columns, each potentially with its own colour). [SCI-VS-EDGE]
- **Edge_Properties**: A pairing of a column position and a colour, used for multi-edge configurations. [SCI-VS-EDGE]
- **Wrap_Marker**: A small visual indicator drawn at the start or end of a wrapped sub-line to signal that the line continues from/to the adjacent sub-line. [SCI-VS-10]
- **Wrap_Visual_Flag**: A bitfield controlling which wrap markers are displayed: None, End (marker at end of sub-line), Start (marker at start of continuation sub-line), Margin (marker in the margin area). [SCI-VS-10]
- **Wrap_Visual_Location**: Controls whether wrap markers are positioned at the text boundary (near the last/first character) or at the edge of the display area. Values: Default (at edge), EndByText (end marker near text), StartByText (start marker near text). [SCI-VS-10]
- **Wrap_Indent_Mode**: Controls how continuation sub-lines are indented relative to the first sub-line. Modes: Fixed (a fixed offset), Same (same indentation as first sub-line), Indent (one extra indent level), DeepIndent (two extra indent levels). [SCI-VS-10]
- **Wrap_Start_Indent**: The number of characters (or width units) of additional indentation applied to continuation sub-lines when Wrap_Indent_Mode is Fixed. [SCI-VS-10]

## Requirements

### Requirement 1: Whitespace Visibility Modes

**User Story:** As a developer, I want to toggle the visibility of space and tab characters so that I can verify indentation style, detect trailing whitespace, and inspect formatting without modifying the document content.

**Source:** [SCI-VS-9] `ViewStyle::viewWhitespace`, `WhiteSpace` enum, `WhiteSpaceVisible()`.

#### Acceptance Criteria

1. THE whitespace-and-guides subsystem SHALL support the following Whitespace_Visibility modes: `Invisible` (no whitespace glyphs rendered), `VisibleAlways` (all spaces and tabs rendered), `VisibleAfterIndent` (only spaces/tabs after the first non-whitespace character on each line), and `VisibleOnlyInIndent` (only leading spaces/tabs before the first non-whitespace character). [SCI-VS-9]
2. THE default Whitespace_Visibility mode SHALL be `Invisible`. [SCI-VS-9, WB]
3. WHEN Whitespace_Visibility is set to `VisibleAlways`, THE system SHALL render a Whitespace_Glyph for every space and tab character in the document, including leading indentation, inline spacing, and trailing whitespace. [SCI-VS-9]
4. WHEN Whitespace_Visibility is set to `VisibleAfterIndent`, THE system SHALL render Whitespace_Glyphs only for space and tab characters that occur after the first non-whitespace character on each line. [SCI-VS-9]
5. WHEN Whitespace_Visibility is set to `VisibleOnlyInIndent`, THE system SHALL render Whitespace_Glyphs only for space and tab characters that occur before the first non-whitespace character on each line. [SCI-VS-9]
6. THE Whitespace_Visibility mode SHALL be stored in the configuration-system under the key `editor.whitespace_mode` and SHALL respond to hot-reload changes. [WB]

---

### Requirement 2: Whitespace Glyph Appearance

**User Story:** As a user, I want control over how whitespace characters are drawn — their colour, size, and tab style — so that visible whitespace is informative without being distracting.

**Source:** [SCI-VS-9] `whitespaceSize`, `tabDrawMode`, WhiteSpace element colour with alpha.

#### Acceptance Criteria

1. THE system SHALL render visible space characters as a centred dot glyph, positioned vertically and horizontally at the midpoint of the character cell. [SCI-VS-9]
2. THE system SHALL support the following Tab_Draw_Mode values for visible tab characters: `LongArrow` (a rightward arrow spanning the full tab width) and `Strikeout` (a horizontal line through the vertical centre of the tab span). [SCI-VS-9]
3. THE default Tab_Draw_Mode SHALL be `LongArrow`. [SCI-VS-9]
4. THE Tab_Draw_Mode SHALL be stored in the configuration-system under the key `editor.tab_draw_mode`. [WB]
5. THE system SHALL support a configurable Whitespace_Size integer (minimum 1) controlling the diameter of the space dot glyph and the stroke width of tab arrows/strikeouts, stored under the configuration key `editor.whitespace_size`. [SCI-VS-9, WB]
6. THE default Whitespace_Size SHALL be 1. [SCI-VS-9]
7. THE whitespace foreground colour SHALL be resolved from the active theme via the `theme-and-appearance` subsystem, supporting an optional alpha channel for translucent rendering. [SCI-VS-9, WB]
8. THE whitespace background colour SHALL be independently configurable from the foreground, also resolved from the theme, allowing whitespace to be rendered on a distinct background (e.g., a subtle highlight for trailing spaces). [SCI-VS-9, WB]
9. WHEN the theme does not define a whitespace-specific colour, THE system SHALL fall back to the default text foreground colour for whitespace glyphs. [SCI-VS-9]

---

### Requirement 3: Indent Guide Display

**User Story:** As a developer working with indentation-based structure (Python, YAML) or deeply nested code, I want vertical guide lines at each indent level so that I can visually trace the scope and nesting depth of code blocks.

**Source:** [SCI-EV-GUIDE] `DrawIndentGuide`, `DrawIndentGuidesOverEmpty`, `STYLE_INDENTGUIDE`, `IndentView` enum.

#### Acceptance Criteria

1. THE system SHALL support the following Indent_Guide_Mode values: `None` (no guides drawn), `Real` (guides drawn only on lines that have actual indentation at or beyond that guide column), `LookForward` (guides drawn through empty or short-indented lines by examining the next non-empty line's indent), and `LookBoth` (guides drawn through empty lines by examining both preceding and following non-empty lines and using the higher indent level). [SCI-EV-GUIDE]
2. THE default Indent_Guide_Mode SHALL be `None`. [SCI-EV-GUIDE, WB]
3. WHEN Indent_Guide_Mode is `Real`, THE system SHALL draw a vertical guide line at every tab-stop column that falls within the leading whitespace of each line, using the document's configured tab size to determine column positions. [SCI-EV-GUIDE]
4. WHEN Indent_Guide_Mode is `LookForward`, THE system SHALL extend indent guides through blank lines or lines with less indentation by scanning forward to the next line with equal or greater indentation. [SCI-EV-GUIDE]
5. WHEN Indent_Guide_Mode is `LookBoth`, THE system SHALL extend indent guides through blank lines by taking the maximum indent level from scanning both forward and backward to the nearest non-blank lines. [SCI-EV-GUIDE]
6. THE indent guide lines SHALL be drawn using the `IndentGuide` style (colour, line pattern) defined in the `theme-and-appearance` subsystem. [SCI-EV-GUIDE, WB]
7. THE Indent_Guide_Mode SHALL be stored in the configuration-system under the key `editor.indent_guides` and SHALL respond to hot-reload changes. [WB]
8. WHEN word wrap is active and a document line wraps to multiple sub-lines, THE indent guides SHALL continue vertically through all sub-lines of that document line. [SCI-EV-GUIDE]

---

### Requirement 4: Active Indent Guide Highlighting

**User Story:** As a developer navigating deeply nested code, I want the indent guide at my current scope level highlighted so that I can instantly identify which block I'm editing within.

**Source:** [SCI-EV-GUIDE] `SetHighlightGuide`, `pixmapIndentGuideHighlight`, active block detection.

#### Acceptance Criteria

1. THE system SHALL support highlighting a single Active_Indent_Guide at a specified column, rendering it in a distinct highlight colour/style that visually differentiates it from inactive guides. [SCI-EV-GUIDE]
2. THE Active_Indent_Guide column SHALL be determined by the indentation level of the innermost brace-matched block or scope boundary containing the caret position. [SCI-EV-GUIDE]
3. WHEN no scope boundary or brace match is active (e.g., caret at column 0 or no matching braces found), THE Active_Indent_Guide SHALL be suppressed (no guide highlighted). [SCI-EV-GUIDE]
4. THE highlight colour for the Active_Indent_Guide SHALL be resolved from the `theme-and-appearance` subsystem, distinct from the default indent guide colour. [SCI-EV-GUIDE, WB]
5. WHEN the caret moves, THE Active_Indent_Guide SHALL update to reflect the new caret scope without requiring a full viewport repaint. [SCI-EV-GUIDE, WB]

---

### Requirement 5: Edge Column Indicator

**User Story:** As a developer adhering to line-length conventions (e.g., 80 or 120 columns), I want a visual indicator at the configured column position so that I can see when lines approach or exceed the limit without manually counting characters.

**Source:** [SCI-VS-EDGE] `edgeState`, `EdgeVisualStyle`, `theEdge`, `theMultiEdge`, `SetEdgeColumn`, `MultiEdgeAddLine`.

#### Acceptance Criteria

1. THE system SHALL support the following Edge_Mode values: `None` (no edge indicator), `Line` (a thin vertical line drawn at the edge column), `Background` (all text beyond the edge column is drawn with a shaded background colour), and `MultiLine` (multiple vertical line indicators, each at a different column with its own colour). [SCI-VS-EDGE]
2. THE default Edge_Mode SHALL be `None`. [SCI-VS-EDGE, WB]
3. WHEN Edge_Mode is `Line`, THE system SHALL draw a single vertical line at the column specified by the `editor.edge_column` configuration key, spanning the full viewport height. [SCI-VS-EDGE, WB]
4. WHEN Edge_Mode is `Background`, THE system SHALL render all character cells at or beyond the configured edge column with the edge background colour, providing a shaded band indicating the overflow region. [SCI-VS-EDGE]
5. WHEN Edge_Mode is `MultiLine`, THE system SHALL support an ordered list of Edge_Properties (column + colour pairs), drawing a vertical line at each specified column in the corresponding colour. [SCI-VS-EDGE]
6. THE multi-edge list SHALL be stored in the configuration-system under the key `editor.edge_columns` as an array of `{column, colour}` entries. [WB]
7. THE single-edge column SHALL be stored under `editor.edge_column` (integer) and the edge colour under `editor.edge_colour`. [WB]
8. THE Edge_Mode SHALL be stored under `editor.edge_mode` and SHALL respond to hot-reload changes. [WB]
9. WHEN Edge_Mode is `MultiLine`, THE system SHALL provide a command to clear all multi-edge entries, resetting to an empty list. [SCI-VS-EDGE, WB]
10. THE edge column indicator SHALL be drawn behind text content (as a background decoration) so that it does not obscure readable text. [SCI-VS-EDGE]

---

### Requirement 6: Wrap Visual Markers

**User Story:** As a user working with word wrap enabled, I want small visual indicators at the start and/or end of wrapped sub-lines so that I can distinguish soft wraps from actual line endings.

**Source:** [SCI-VS-10] `WrapVisualFlag`, `WrapVisualLocation`, `WrapAppearance`.

#### Acceptance Criteria

1. THE system SHALL support the following Wrap_Visual_Flag values (combinable as a bitfield): `None` (no markers), `End` (marker at the end of each sub-line that continues), `Start` (marker at the start of each continuation sub-line), and `Margin` (marker in the line-number margin area for wrapped lines). [SCI-VS-10]
2. THE default Wrap_Visual_Flag SHALL be `None`. [SCI-VS-10, WB]
3. WHEN `End` is set, THE system SHALL render a small glyph (e.g., a curved arrow or continuation symbol) at the rightmost position of each sub-line whose content continues on the next sub-line. [SCI-VS-10]
4. WHEN `Start` is set, THE system SHALL render a small glyph at the leftmost position of each continuation sub-line (sub-line index > 0). [SCI-VS-10]
5. WHEN `Margin` is set, THE system SHALL render a wrap indicator in the margin area adjacent to wrapped lines. [SCI-VS-10]
6. THE Wrap_Visual_Location SHALL control positioning of wrap markers: `Default` places markers at the display edge, `EndByText` places the end marker adjacent to the last character, and `StartByText` places the start marker adjacent to the first character of the continuation sub-line. [SCI-VS-10]
7. THE Wrap_Visual_Flag and Wrap_Visual_Location SHALL be stored in the configuration-system under the keys `editor.wrap_visual_flags` and `editor.wrap_visual_location`. [WB]
8. THE wrap marker colour SHALL be resolved from the `theme-and-appearance` subsystem. [WB]
9. WHEN word wrap is not active (Wrap mode = None), THE system SHALL NOT render any wrap markers regardless of the Wrap_Visual_Flag setting. [SCI-VS-10]

---

### Requirement 7: Wrap Indentation for Continuation Sub-Lines

**User Story:** As a user reading wrapped content, I want continuation sub-lines indented relative to the first sub-line so that I can visually distinguish the start of a logical line from its continuation.

**Source:** [SCI-VS-10] `WrapIndentMode`, `wrapVisualStartIndent`, `WrapAppearance`.

#### Acceptance Criteria

1. THE system SHALL support the following Wrap_Indent_Mode values: `Fixed` (continuation sub-lines are indented by a fixed number of characters defined by Wrap_Start_Indent), `Same` (continuation sub-lines use the same indentation as the first sub-line), `Indent` (continuation sub-lines are indented one additional tab stop beyond the first sub-line's indentation), and `DeepIndent` (continuation sub-lines are indented two additional tab stops beyond the first sub-line's indentation). [SCI-VS-10]
2. THE default Wrap_Indent_Mode SHALL be `Fixed` with a Wrap_Start_Indent of 0 (no additional indentation). [SCI-VS-10]
3. THE Wrap_Start_Indent SHALL be a non-negative integer specifying the number of character widths of additional indentation for `Fixed` mode, stored under the configuration key `editor.wrap_start_indent`. [SCI-VS-10, WB]
4. THE Wrap_Indent_Mode SHALL be stored under the configuration key `editor.wrap_indent_mode` and SHALL respond to hot-reload changes. [WB]
5. WHEN Wrap_Indent_Mode is `Same`, `Indent`, or `DeepIndent`, THE system SHALL compute the indentation of continuation sub-lines relative to the leading whitespace of the document line's first sub-line. [SCI-VS-10]
6. THE total indentation of a continuation sub-line SHALL NOT exceed 3/4 of the viewport width, to ensure readable content always remains visible. [SCI-VS-10]

---

### Requirement 8: Toggle Commands

**User Story:** As a user, I want quick toggle commands to show/hide whitespace, indent guides, and edge indicators without navigating to settings, so that I can switch between clean and annotated views during editing.

**Source:** [WB] Command-driven architecture, toggle commands.

#### Acceptance Criteria

1. THE system SHALL register a `ToggleWhitespace` command with the `command-framework` that cycles the Whitespace_Visibility mode through its values in order: Invisible → VisibleAlways → VisibleAfterIndent → VisibleOnlyInIndent → Invisible. [WB]
2. THE system SHALL register a `ToggleIndentGuides` command that cycles the Indent_Guide_Mode through: None → Real → LookForward → LookBoth → None. [WB]
3. THE system SHALL register a `ToggleEdgeColumn` command that toggles Edge_Mode between `None` and the previously configured non-None mode (defaulting to `Line` if no prior mode was set). [WB]
4. ALL toggle commands SHALL persist their resulting state to the user layer of the configuration-system so that the setting survives application restart. [WB]
5. ALL toggle commands SHALL be assignable to keyboard shortcuts and accessible from menus via the `command-framework`. [WB]
6. WHEN a toggle command changes a visual setting, THE system SHALL emit a configuration-change notification so that the viewport repaints immediately. [WB]

---

### Requirement 9: GUI-Independent Model

**User Story:** As a platform architect, I want the whitespace-and-guides settings model to be completely independent of any GUI framework so that different rendering shells (egui, terminal, headless test) can consume the same configuration state.

**Source:** [WB] GUI-independent platform-core principle.

#### Acceptance Criteria

1. THE `ff-whitespace-guides` crate SHALL NOT depend on any GUI framework crate (e.g., `egui`, `winit`, `wgpu`). It SHALL expose only data types, enums, configuration accessors, and per-line metadata queries. [WB]
2. THE crate SHALL provide a `WhitespaceSettings` struct (or equivalent) aggregating the current effective values of all whitespace, indent guide, edge, and wrap marker settings resolved from the configuration-system. [WB]
3. THE rendering of glyphs, guide lines, edge indicators, and wrap markers SHALL be the responsibility of the GUI shell crate, which reads `WhitespaceSettings` and per-line metadata to drive drawing calls. [WB]
4. THE crate SHALL provide query methods that, given a document line's content and the current settings, return the positions and types of visual elements to render (e.g., list of guide columns, edge column hit, whitespace glyph positions). [WB]
5. THE crate SHALL be testable in a headless environment without any windowing system or display server. [WB]
