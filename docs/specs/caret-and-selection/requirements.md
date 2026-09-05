# Requirements Document

## Introduction

The `caret-and-selection` sub-project defines the **visual presentation** of the caret (cursor), selection highlighting, caret-line highlighting, and virtual space rendering within FileForgeWorkbench. This spec covers the rendering-side concerns: how the caret is drawn, how selected text is visually distinguished, how the caret line is highlighted, and how virtual space is displayed. The logical selection model (SelectionPosition, SelectionRange, Selection container, multi-caret coordination) is defined in `edit-operations`; this spec consumes that model and specifies its visual representation.

This spec covers:
- **Caret shape and style** — Invisible, Line, Block caret modes with configurable width and colour
- **Caret blink** — Blink period managed by the GUI shell; the model is blink-agnostic
- **Caret colour** — Primary caret colour and additional caret colour for multi-caret display
- **Caret line highlight** — Whole-line or sub-line background/frame highlighting for the current caret line
- **Selection display** — Selection background/foreground colours, layer modes (base vs translucent over-text)
- **Selection element colours** — Primary, additional, secondary, and inactive selection colour elements
- **Selection EOL fill** — Whether selection colouring extends past line-end to the right edge
- **Virtual space display** — Visual caret positioning beyond line-end content
- **Rectangular selection display** — Column-highlight rendering for rectangular selections
- **Multi-caret display** — Rendering multiple visible carets simultaneously
- **Modified line marker rendering** — Visual rendering of the `*` marker from edit-operations logical state
- **Theme integration** — All visual settings configurable via the theme system

**Scope boundaries:**
- The logical selection model (positions, ranges, adjustment, multi-caret coordination) is defined in `edit-operations`
- Colour palette definitions and theme file format are defined in `theme-and-appearance`
- Configuration loading/hot-reload mechanics are defined in `configuration-system`
- Viewport scroll-to-caret policies are defined in `viewport-and-scrolling`
- The caret blink timer is owned by the GUI shell — this spec defines the blink period setting but not the timer implementation

**Source references:**
- **[FFE-MVP-2]** = FileForgeEditor mvp-implementation Requirement 2 (cursor row visual distinction — border/outline, no background fill for cursor indication)
- **[FFE-MVP-2.18]** = FileForgeEditor mvp-implementation Requirement 2 criterion 18 (cursor move gives text field keyboard focus)
- **[FFE-MVP-8]** = FileForgeEditor mvp-implementation Requirement 8 (selection highlighting with distinct colour, not obscuring text)
- **[SCI-VS-7]** = Scintilla ViewStyle Requirement 7 (selection appearance — visible, layer, eolFilled, element colours, translucent)
- **[SCI-VS-8]** = Scintilla ViewStyle Requirement 8 (caret appearance — style, width, colour, caret-line highlight)
- **[SCI-SEL-4.1]** = Scintilla Selection model Requirement 4.1 (virtual space, rectangular selection, multi-caret, selection types)
- **[WB]** = Workbench Platform Architecture Brief (GUI independence, theme-configurable, model-rendering separation)

---

## Glossary

- **Caret**: The visual cursor rendered at the logical caret position from `edit-operations`. Drawn as a line, block, or invisible shape depending on configuration. [SCI-VS-8]
- **Caret Style**: The shape of the caret — Invisible (not drawn), Line (vertical bar), Block (solid rectangle spanning one character cell). [SCI-VS-8]
- **Caret Width**: The pixel width of the Line-style caret. Default 1px. Configurable in the range [1, 20]. [SCI-VS-8]
- **Caret Line**: The entire display line containing the primary caret. May be highlighted with a background colour or frame border. [SCI-VS-8]
- **Caret Line Frame**: An outline/border drawn around the caret line instead of a solid fill. Specified as a pixel width. [FFE-MVP-2, SCI-VS-8]
- **Overstrike Block Caret**: A block-shaped caret displayed when the editor is in Overstrike Mode, indicating that typed characters replace rather than insert. [SCI-VS-8]
- **Blink Period**: The total duration (on-time + off-time) of one caret blink cycle, in milliseconds. A period of 0 means no blinking (always visible). [WB]
- **Selection Display**: The visual highlighting applied to text that is currently selected (between anchor and caret). [FFE-MVP-8, SCI-VS-7]
- **Layer Mode**: Controls how the selection colour is composited — Base (opaque, drawn under text) or OverText (translucent, alpha-blended over text). [SCI-VS-7]
- **EOL Fill**: Whether selection colouring extends beyond the last character of a line to the right edge of the text area. [SCI-VS-7]
- **Element Colour**: A named colour slot that can be configured independently via the theme. Elements include SelectionText, SelectionBack, Caret, CaretAdditional, CaretLineBack, etc. [SCI-VS-7, SCI-VS-8]
- **Translucent Selection**: A selection drawn with alpha-blended colours, allowing underlying text and decorations to remain partially visible through the selection. [SCI-VS-7]
- **Virtual Space**: Display positions beyond the end of a line's text content where the caret can be placed visually but no characters exist. [SCI-SEL-4.1]
- **Rectangular Selection**: A column-oriented selection rendered as a vertical band spanning the same left-right column range across multiple lines. [SCI-SEL-4.1]
- **Multi-Caret Display**: Rendering multiple simultaneous carets, each with its own colour (primary vs additional). [SCI-SEL-4.1]
- **Modified Line Marker**: A visual `*` indicator displayed in the prefix area for lines that have been modified since the last save. The logical state is managed by `edit-operations`; this spec defines the rendering. [FFE-MVP-2]
- **Primary Caret**: The main caret (from the main SelectionRange) — uses the primary caret colour element. [SCI-VS-8]
- **Additional Caret**: Non-main carets in a multi-caret scenario — uses the additional caret colour element. [SCI-VS-8]
- **Inactive Selection**: A selection in a pane/view that does not currently have keyboard focus — rendered with muted colours. [SCI-VS-7]
- **Secondary Selection**: Additional (non-primary) selection ranges in a multi-selection — rendered with secondary colours. [SCI-VS-7]

---

## Requirements

### Requirement 1: Caret Shape and Style [SCI-VS-8]

**User Story:** As an editor user, I want a configurable caret shape (line, block, or invisible), so that the cursor appearance matches my preferences and editing mode.

#### Acceptance Criteria

1. THE caret renderer SHALL support three caret styles: Invisible (not drawn), Line (vertical bar), and Block (solid rectangle spanning one character cell width and line height). [SCI-VS-8]

2. THE default caret style SHALL be Line. [SCI-VS-8]

3. WHEN the editor is in Overstrike Mode, THE caret renderer SHALL display a Block-shaped caret (overstrike block) to visually distinguish the mode from Insert Mode. [SCI-VS-8]

4. WHEN the caret style is Line, THE caret renderer SHALL draw the caret as a vertical bar with a configurable width in pixels. [SCI-VS-8]

5. THE default caret width SHALL be 1 pixel. [SCI-VS-8]

6. WHEN the caret width is configured, THE caret renderer SHALL accept values in the range [1, 20] pixels, clamping out-of-range values to the nearest bound. [SCI-VS-8]

7. WHEN the caret style is Block, THE caret renderer SHALL draw the caret as a filled rectangle covering the full character cell at the caret position. [SCI-VS-8]

8. WHEN the caret style is Block and the caret is at the end of a line (no character underneath), THE caret renderer SHALL draw the block with the width of a space character. [SCI-VS-8]

9. WHEN the caret style is Invisible, THE caret renderer SHALL not draw any caret graphic. [SCI-VS-8]

10. THE caret style setting SHALL be configurable via the theme system (cross-reference: `theme-and-appearance`). [WB]

---

### Requirement 2: Caret Colour [SCI-VS-8]

**User Story:** As an editor user, I want configurable caret colours so that the cursor is clearly visible against any background theme.

#### Acceptance Criteria

1. THE caret renderer SHALL use the `Caret` element colour for rendering the primary caret (the caret of the main SelectionRange). [SCI-VS-8]

2. THE default `Caret` element colour SHALL be black (#000000). [SCI-VS-8]

3. THE caret renderer SHALL use the `CaretAdditional` element colour for rendering additional carets (non-main carets in a multi-caret scenario). [SCI-VS-8]

4. THE default `CaretAdditional` element colour SHALL be grey (#7F7F7F). [SCI-VS-8]

5. WHEN the `Caret` element colour is configured via the theme, THE caret renderer SHALL use the theme-specified colour for the primary caret. [WB]

6. WHEN the `CaretAdditional` element colour is configured via the theme, THE caret renderer SHALL use the theme-specified colour for additional carets. [WB]

7. WHEN a Block-style caret is drawn, THE caret renderer SHALL render the character underneath the block using the inverse of the caret colour (or the configured selection text colour) to maintain legibility. [SCI-VS-8]

---

### Requirement 3: Caret Blink [WB, SCI-VS-8]

**User Story:** As an editor user, I want the caret to blink at a configurable rate (or not at all), so that the cursor draws my eye without being distracting.

#### Acceptance Criteria

1. THE caret-and-selection model SHALL expose a `blink_period_ms` configuration value representing the total blink cycle duration in milliseconds. [WB]

2. THE default `blink_period_ms` SHALL be 530 milliseconds (matching typical desktop editor defaults). [WB]

3. WHEN `blink_period_ms` is set to 0, THE caret SHALL remain permanently visible (no blinking). [WB]

4. THE blink timer SHALL be owned and driven by the GUI shell — the caret-and-selection model SHALL expose only the period value and a `visible_phase` query method. The model SHALL NOT contain a timer implementation. [WB]

5. WHEN the GUI shell queries the blink state, THE model SHALL report whether the caret is in the visible phase or hidden phase based on elapsed time modulo `blink_period_ms`. [WB]

6. WHEN the caret position changes (user typed, navigated, or clicked), THE blink cycle SHALL reset to the visible phase to ensure the caret is immediately visible after movement. [WB]

7. THE `blink_period_ms` value SHALL be configurable via the configuration system and overridable by the theme (cross-reference: `configuration-system`). [WB]

---

### Requirement 4: Caret Line Highlight [FFE-MVP-2, SCI-VS-8]

**User Story:** As an editor user, I want the line containing my cursor to be visually distinguished, so that I can quickly locate my editing position within the document.

#### Acceptance Criteria

1. THE caret renderer SHALL support highlighting the current caret line using either a background fill OR a frame (border/outline), but not both simultaneously. [FFE-MVP-2, SCI-VS-8]

2. THE default caret-line highlight mode SHALL be Frame (border/outline) with no background fill, consistent with the FFE visual design. [FFE-MVP-2]

3. WHEN caret-line highlighting is configured as Frame mode, THE renderer SHALL draw a border/outline around the entire caret line row. The frame width SHALL be configurable, defaulting to 1 pixel. [FFE-MVP-2, SCI-VS-8]

4. WHEN caret-line highlighting is configured as Fill mode, THE renderer SHALL draw the `CaretLineBack` element colour as the background for the entire caret line. Text colour SHALL remain unchanged. [SCI-VS-8]

5. THE caret-line frame width SHALL be clamped to the range [1, lineHeight / 3] to prevent visual overflow. [SCI-VS-8]

6. THE caret-line highlight SHALL support a `layer` setting: Base (drawn under text, opaque) or OverText (drawn over text, translucent). [SCI-VS-8]

7. THE default caret-line layer SHALL be Base. [SCI-VS-8]

8. THE caret-line highlight SHALL support an `always_show` flag. WHEN `always_show` is true, THE highlight SHALL be visible even when the editor pane does not have keyboard focus. WHEN `always_show` is false, THE highlight SHALL only appear when the pane is focused. [SCI-VS-8]

9. THE default `always_show` value SHALL be false. [SCI-VS-8]

10. THE caret-line highlight SHALL support a `sub_line` flag. WHEN `sub_line` is true and word-wrap is active, THE highlight SHALL apply only to the wrapped sub-line containing the caret, not the entire document line. WHEN `sub_line` is false, THE highlight SHALL span the full document line (all wrapped sub-lines). [SCI-VS-8]

11. THE default `sub_line` value SHALL be false (highlight the full line). [SCI-VS-8]

12. THE `CaretLineBack` element colour SHALL be configurable via the theme system, defaulting to a subtle highlight appropriate for the current colour scheme. [WB]

13. WHEN the editor has multiple carets, THE caret-line highlight SHALL apply to the line containing the main (primary) caret only. [SCI-VS-8]

---

### Requirement 5: Selection Display — Colours and Layers [FFE-MVP-8, SCI-VS-7]

**User Story:** As an editor user, I want selected text to be visually highlighted with distinct colours that do not obscure the text, so that I can clearly see what is selected while still reading the content.

#### Acceptance Criteria

1. THE selection renderer SHALL display selected text using a distinct background colour that contrasts with the default text background. [FFE-MVP-8]

2. THE selection renderer SHALL NOT obscure selected text — text SHALL remain legible through the selection highlighting. [FFE-MVP-8]

3. THE selection renderer SHALL support a `visible` flag. WHEN `visible` is false, THE selection SHALL not be rendered visually (though the logical selection remains active in `edit-operations`). [SCI-VS-7]

4. THE default selection visibility SHALL be true. [SCI-VS-7]

5. THE selection renderer SHALL support a `layer` setting controlling compositing mode: Base (opaque background drawn under text, replacing the default background) or OverText (translucent, alpha-blended over both text and background). [SCI-VS-7]

6. THE default selection layer SHALL be Base (opaque). [SCI-VS-7]

7. WHEN the selection layer is Base, THE renderer SHALL draw the selection background colour underneath the text, and optionally override the text foreground colour with the SelectionText element. [SCI-VS-7]

8. WHEN the selection layer is OverText, THE renderer SHALL draw the selection as a translucent overlay above the text, using the alpha channel of the selection colour elements. [SCI-VS-7]

9. THE selection renderer SHALL support an `eol_filled` flag. WHEN `eol_filled` is true, THE selection background SHALL extend from the last character on a selected line to the right edge of the text area. WHEN false, THE selection background SHALL end at the last selected character. [SCI-VS-7]

10. THE default `eol_filled` value SHALL be false. [SCI-VS-7]

---

### Requirement 6: Selection Element Colours [SCI-VS-7]

**User Story:** As a theme designer, I want fine-grained control over selection colours for different selection contexts (primary, additional, secondary, inactive), so that all selection states are visually distinguishable.

#### Acceptance Criteria

1. THE selection renderer SHALL support the following element colour pairs, each with a text (foreground) and back (background) component: [SCI-VS-7]
- `SelectionText` / `SelectionBack` — primary selection colours
- `SelectionAdditionalText` / `SelectionAdditionalBack` — additional (non-primary) multi-selection colours
- `SelectionSecondaryText` / `SelectionSecondaryBack` — secondary selection colours (e.g., find-all highlights)
- `SelectionInactiveText` / `SelectionInactiveBack` — inactive pane selection colours

2. THE default `SelectionBack` colour SHALL be grey (#C0C0C0) fully opaque. [SCI-VS-7]

3. THE default `SelectionAdditionalBack` colour SHALL be grey (#D7D7D7) fully opaque. [SCI-VS-7]

4. THE default `SelectionSecondaryBack` colour SHALL be grey (#B0B0B0) fully opaque. [SCI-VS-7]

5. THE default `SelectionInactiveBack` colour SHALL be grey (#808080) with alpha 0x3F (translucent). [SCI-VS-7]

6. WHEN a SelectionText element colour is set, THE renderer SHALL override the text foreground colour within the selection with that element colour. WHEN no SelectionText colour is set, THE text SHALL retain its original syntax-highlighted foreground colour. [SCI-VS-7]

7. ALL selection element colours SHALL support translucent (alpha-blended) values. [SCI-VS-7]

8. ALL selection element colours SHALL be configurable via the theme system (cross-reference: `theme-and-appearance`). [WB]

9. WHEN a secondary selection exists (e.g., find-all match highlights that are not the primary selection), THE renderer SHALL use the `SelectionSecondary*` colour pair for display. [SCI-VS-7]

10. WHEN the editor pane loses keyboard focus and a selection is active, THE renderer SHALL switch to the `SelectionInactive*` colour pair for that selection. [SCI-VS-7]

---

### Requirement 7: Virtual Space Display [SCI-SEL-4.1]

**User Story:** As an editor user working with fixed-format data or column selections, I want the caret to be positionable beyond the end of a line, so that I can place content in column-aligned positions without first padding with spaces.

#### Acceptance Criteria

1. WHEN the caret's virtual space offset (from the logical SelectionPosition in `edit-operations`) is greater than zero, THE caret renderer SHALL display the caret at a horizontal position computed as: end-of-line-content + (virtual_space × space_width). [SCI-SEL-4.1]

2. THE virtual space area between line-end and the caret SHALL be rendered as empty space (no visible characters) with the default line background colour. [SCI-SEL-4.1]

3. WHEN a rectangular selection spans virtual space (columns beyond a line's content), THE selection renderer SHALL display the selection highlight in the virtual space area as if space characters existed there. [SCI-SEL-4.1]

4. WHEN virtual space is part of a selection range, THE selection highlight SHALL extend through the virtual space region between line-end and the selection boundary. [SCI-SEL-4.1]

5. THE caret SHALL visually occupy virtual space identically to how it occupies real text positions — same style, width, and colour apply. [SCI-SEL-4.1]

6. THE virtual space display SHALL NOT render any visible whitespace indicators in the virtual region (whitespace visibility applies only to real content). [SCI-SEL-4.1]

---

### Requirement 8: Rectangular Selection Display [SCI-SEL-4.1]

**User Story:** As an editor user editing columnar data, I want rectangular selections displayed as a consistent column band across multiple lines, so that I can clearly see the column boundaries I'm operating on.

#### Acceptance Criteria

1. WHEN a rectangular selection is active (selType is rectangle or thin in `edit-operations`), THE selection renderer SHALL display the selection as a vertical column band — one selection segment per line spanning the same left-to-right column range. [SCI-SEL-4.1]

2. THE rectangular selection highlight SHALL use the same `SelectionBack` colour as stream selections, drawn with the same layer mode. [SCI-SEL-4.1]

3. WHEN the rectangular selection spans lines of differing lengths and the right column exceeds a line's content length, THE renderer SHALL display the selection highlight extending into virtual space for that line. [SCI-SEL-4.1]

4. WHEN a rectangular selection of type "thin" is active (zero-width column selection), THE renderer SHALL display a thin vertical line at the column position on each affected line, indicating the insertion point for column operations. [SCI-SEL-4.1]

5. THE rectangular selection SHALL display one caret per line at the caret-column edge of the selection (right edge for left-to-right selections, left edge for right-to-left). [SCI-SEL-4.1]

---

### Requirement 9: Multi-Caret Display [SCI-SEL-4.1, SCI-VS-8]

**User Story:** As a power user with multiple carets active, I want each caret rendered distinctly, so that I can see all my editing positions simultaneously.

#### Acceptance Criteria

1. WHEN multiple carets are active (multiple SelectionRanges in the Selection container from `edit-operations`), THE renderer SHALL draw a caret at each caret position. [SCI-SEL-4.1]

2. THE primary caret (from the main SelectionRange) SHALL be rendered using the `Caret` element colour. [SCI-VS-8]

3. ALL additional carets (non-main SelectionRanges) SHALL be rendered using the `CaretAdditional` element colour. [SCI-VS-8]

4. ALL carets (primary and additional) SHALL use the same caret style (Line, Block, or Invisible) as configured globally. [SCI-VS-8]

5. WHEN each additional caret has its own selection range (anchor ≠ caret), THE renderer SHALL display selection highlighting for each range using `SelectionAdditionalBack` / `SelectionAdditionalText` colours. [SCI-VS-7]

6. THE caret blink cycle SHALL apply identically to all visible carets — all carets blink in phase (simultaneously visible, simultaneously hidden). [WB]

---

### Requirement 10: Modified Line Marker Rendering [FFE-MVP-2]

**User Story:** As an editor user, I want a visual indicator on lines I have modified since the last save, so that I can quickly identify my changes while editing.

#### Acceptance Criteria

1. WHEN a line's modified flag is set (logical state from `edit-operations`), THE renderer SHALL display a `*` character in the prefix area for that line. [FFE-MVP-2]

2. THE modified line marker SHALL be rendered using the theme-configured marker colour (cross-reference: `theme-and-appearance`). [WB]

3. THE modified line marker position SHALL be fixed within the prefix area and SHALL NOT shift when line numbers change width. [FFE-MVP-2]

4. WHEN a SAVE operation clears all modified flags (via `edit-operations`), THE renderer SHALL immediately remove all `*` markers from the display. [FFE-MVP-2]

5. THE modified line marker SHALL remain visible regardless of caret-line highlighting — the marker SHALL NOT be obscured by the caret-line background or frame. [FFE-MVP-2, WB]

---

### Requirement 11: Theme Integration and Configuration [WB]

**User Story:** As a workbench platform, I want all caret and selection visual settings configurable via the theme system, so that themes can provide a cohesive appearance across all visual elements.

#### Acceptance Criteria

1. THE following caret and selection settings SHALL be configurable through the theme system (cross-reference: `theme-and-appearance`, `configuration-system`): [WB]
- Caret style (Invisible, Line, Block)
- Caret width (pixels)
- Caret colour (element: Caret)
- Additional caret colour (element: CaretAdditional)
- Caret-line highlight mode (None, Frame, Fill)
- Caret-line frame width (pixels)
- Caret-line background colour (element: CaretLineBack)
- Caret-line layer (Base, OverText)
- Caret-line always-show flag
- Caret-line sub-line flag
- Blink period (milliseconds)
- Selection visibility flag
- Selection layer (Base, OverText)
- Selection EOL-fill flag
- All selection element colours (SelectionText, SelectionBack, SelectionAdditionalText, SelectionAdditionalBack, SelectionSecondaryText, SelectionSecondaryBack, SelectionInactiveText, SelectionInactiveBack)
- Modified line marker colour

2. WHEN the theme is hot-reloaded (cross-reference: `configuration-system`), THE caret and selection renderer SHALL apply the new visual settings on the next frame without requiring a restart. [WB]

3. WHEN a theme does not specify a particular caret/selection setting, THE renderer SHALL use the default values defined in this specification. [WB]

4. THE caret-and-selection model SHALL be GUI-independent — it SHALL store configuration and expose query methods without depending on any rendering framework type. GUI shells consume the model to perform actual drawing. [WB]

5. WHEN configuration values are changed programmatically (e.g., via a settings dialog or command), THE changes SHALL take effect immediately on the next render frame. [WB]

---

### Requirement 12: Caret Keyboard Focus Integration [FFE-MVP-2]

**User Story:** As an editor user, I want the caret line's text field to receive keyboard focus when the cursor moves, so that I can immediately type at the new position without an extra click.

#### Acceptance Criteria

1. WHEN the caret moves to a new line via arrow key, Page Up/Down, mouse click, or command, THE GUI shell SHALL give keyboard focus to the text content at the caret position, making the caret visible and input-ready. [FFE-MVP-2]

2. WHEN the caret is positioned within the viewport and the editor pane has focus, THE caret SHALL always be visible (not obscured by other UI elements). [FFE-MVP-2]

3. WHEN the editor pane receives keyboard focus (e.g., user clicks in the editor area), THE caret SHALL immediately become visible in its current position and the blink cycle SHALL reset to the visible phase. [WB]

---

---

### Requirement 13: Mouse Text Selection in the Editor Canvas [CR-NR-034]

**User Story:** As an editor user, I want to click and drag the mouse to select text in the editor canvas, so that I can copy any visible text to the OS clipboard and paste it into other applications.

#### Acceptance Criteria

13.1 WHEN the user presses the primary mouse button inside the editor text area, THE editor SHALL record the click position as the selection anchor, converting the screen coordinate to a (line, column) document position.

13.2 WHEN the user holds the primary mouse button and moves the pointer, THE editor SHALL continuously update the selection end position to the current pointer coordinate, extending the selection in real time.

13.3 WHEN the user releases the primary mouse button after a drag, THE editor SHALL finalise the selection range from anchor to release position.

13.4 WHEN a selection is active, THE editor SHALL render a highlight rectangle behind the selected text on each affected line using the `SelectionBack` element colour.

13.5 WHEN the user clicks without dragging (pointer does not move more than 2 pixels), THE editor SHALL clear any active selection and position the caret at the click position.

13.6 WHEN the user presses Escape while a selection is active, THE editor SHALL clear the selection.

13.7 WHEN the user presses Ctrl+C with an active selection in the editor canvas, THE editor SHALL copy the selected text to the OS clipboard and display a brief status message confirming the copy.

13.8 WHEN the user presses Ctrl+C with no active selection, THE editor SHALL NOT copy anything (no line-copy-mode in the canvas -- that is handled by the clipboard-operations COPY command).

13.9 THE selection state SHALL be per-tab -- switching tabs clears the selection in the previous tab.

13.10 WHEN the document is scrolled while a selection is active, THE selection highlight SHALL remain correctly positioned relative to the document lines (not the screen).

---

### Requirement 14: Selectable Text in Read-Only Panels [CR-NR-034]

**User Story:** As a user, I want to be able to select and copy text from read-only panels (POM option descriptions, Settings panel values, status bar messages), so that I can paste panel content into other tools.

#### Acceptance Criteria

14.1 WHEN text is rendered in the Primary Option Menu panel (option labels, descriptions, calendar text), THE text SHALL be rendered using egui selectable labels so the user can click-drag to select and Ctrl+C to copy.

14.2 WHEN text is rendered in the Settings panel (key names, values, descriptions), THE text SHALL be rendered using egui selectable labels.

14.3 WHEN text is rendered in the status bar (file path, line/column, encoding, messages), THE text SHALL be rendered using egui selectable labels.

14.4 WHEN the user selects text in a read-only panel and presses Ctrl+C, THE OS clipboard SHALL receive the selected text via egui's built-in clipboard integration.

14.5 THE selectable label behaviour SHALL NOT interfere with existing click-to-navigate interactions (POM option buttons, Settings edit fields).

---

## Cross-References

- **`edit-operations`** — Defines the logical selection model (SelectionPosition, SelectionRange, Selection container, multi-caret, modified line flags) consumed by this spec for rendering
- **`theme-and-appearance`** — Defines the colour palette, TOML theme file format, and semantic colour tokens used by element colours in this spec
- **`viewport-and-scrolling`** — Defines scroll-to-caret policies that ensure the caret remains visible after movement
- **`configuration-system`** — Defines configuration loading, hot-reload, and per-project override mechanics used by caret/selection settings
- **`whitespace-and-guides`** — Defines whitespace visibility rendering (excluded from virtual space areas per Requirement 7.6)
- **`display-line-mapping`** — Provides the wrapped sub-line information needed for the `sub_line` caret-line highlight (Requirement 4.10)
- **`clipboard-operations`** -- Defines the clipboard write contract consumed by Requirement 13.7
