# Requirements Document

## Introduction

This feature specifies the **Text Decorations** subsystem for FileForgeWorkbench — the `ff-text-decorations` crate. Text decorations are visual overlays applied on top of (or underneath) the rendered text to communicate semantic information such as search matches, diagnostic errors, change history, and bookmarks. Unlike syntax highlighting (which assigns style classes to text ranges), decorations are transient, overlapping, and independently managed by multiple producers.

The subsystem adapts Scintilla's indicator and line marker systems into a Rust-native architecture with the following key differences:

- **Run-length-encoded storage** for per-character indicator ranges, adapted from Scintilla's `RunStyles` / `Decoration` for memory-efficient sparse coverage across large documents.
- **Indicator styles** for inline text decorations (underlines, boxes, squiggles, colour overrides) — the 23 Scintilla indicator styles adapted to egui rendering primitives.
- **Line markers** for gutter/margin annotations (bookmarks, change history, modified indicators) — adapted from Scintilla's `LineMarker` geometric shapes.
- **Theme integration** for all decoration colours and style parameters, with full dark/light/high-contrast theme support.
- **High-DPI rendering** with pixel-aligned coordinates for crisp decoration output at any scale factor.

This is a Wave 6 (UI and Rendering) component. It is NEW from the Scintilla gap analysis — FileForgeEditor did not have an explicit decoration subsystem.

**Source references:**
- **[SCI-IND-10.1]** = Scintilla `Indicator` class — 23 indicator styles, ValueFore, hover state, fillAlpha/outlineAlpha, strokeWidth, under property, pixel-aligned drawing
- **[SCI-MRK-10.3]** = Scintilla `LineMarker` class — geometric margin shapes, fold markers, custom colours, alpha/layer support
- **[SCI-DEC]** = Scintilla `Decoration` / `DecorationList` / `RunStyles` — run-length-encoded per-character indicator values, InsertSpace/DeleteRange for edit tracking
- **[WB]** = Workbench Platform Architecture Brief — GUI-independent core, theme system, command-driven architecture

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `find-and-replace` | **Producer** | Search match highlighting (current match, all matches) uses indicators to mark match ranges in the document. |
| `theme-and-appearance` | **Dependency** | All decoration colours, alpha values, and stroke widths are sourced from the active theme definition. |
| `document-model` | **Dependency** | Decoration storage is indexed by character position within the document buffer; edit operations require decoration position adjustment. |
| `undo-redo-transactions` | **Integration** | Decoration changes triggered by edits (InsertSpace/DeleteRange) are synchronized with undo/redo operations so decorations remain consistent after undo. |
| `display-line-mapping` | **Consumer** | Line markers (bookmarks, change indicators) reference document lines; the display-line-mapping translates these to viewport positions for rendering. |
| `syntax-highlighting` | **Peer** | Syntax styles and indicators coexist on the same text; the `under` property determines whether indicators render below or above syntax-coloured text. |
| `viewport-and-scrolling` | **Consumer** | The viewport renderer queries active decorations within the visible range to draw overlays during paint. |
| `configuration-system` | **Integration** | Per-indicator style defaults and user overrides are stored in configuration; hot-reload updates decoration appearance without restart. |

## Glossary

- **Indicator**: A named text decoration style that can be applied to arbitrary character ranges in a document. Each indicator has a visual style, colour, and rendering properties. Multiple indicators may overlap on the same text range. [SCI-IND-10.1]
- **Indicator_Style**: The visual appearance of an indicator — one of 23 predefined styles (Plain, Squiggle, Box, etc.) that determine how the decoration is drawn relative to the text. [SCI-IND-10.1]
- **Indicator_Value**: An integer value associated with each character position for a given indicator. Value 0 means "no decoration"; any non-zero value activates the indicator. When ValueFore is enabled, the value encodes an RGB colour. [SCI-IND-10.1, SCI-DEC]
- **Decoration**: The per-document storage of indicator values for a single indicator number. Internally uses run-length encoding for efficient sparse storage. [SCI-DEC]
- **Decoration_List**: The collection of all active decorations for a document, indexed by indicator number. Provides aggregate queries (all indicators active at a position). [SCI-DEC]
- **Run_Length_Encoding**: A compression technique that stores consecutive positions with the same indicator value as a single (value, length) pair, reducing memory for sparse decorations from O(document_length) to O(number_of_transitions). [SCI-DEC]
- **Line_Marker**: A visual symbol displayed in the gutter margin adjacent to a document line, indicating state such as bookmarks, change history, or fold structure. [SCI-MRK-10.3]
- **Marker_Symbol**: The geometric shape or image used to render a line marker (Circle, Arrow, Bookmark, RoundRect, custom pixmap, etc.). [SCI-MRK-10.3]
- **Marker_Mask**: A bitmask indicating which marker numbers (0–31) are active on a given line. Each bit position corresponds to a marker number. [SCI-MRK-10.3]
- **Hover_State**: The distinction between normal and hover appearance for an indicator, enabling interactive feedback when the mouse cursor moves over a decorated range. [SCI-IND-10.1]
- **Fill_Alpha**: The opacity (0–255) of the interior fill for box-style indicators (default 30). [SCI-IND-10.1]
- **Outline_Alpha**: The opacity (0–255) of the border/outline for box-style indicators (default 50). [SCI-IND-10.1]
- **Stroke_Width**: The line thickness in logical pixels for line-based indicators (underlines, squiggles, etc.), default 1.0. Supports fractional values for high-DPI scaling. [SCI-IND-10.1]
- **Under_Property**: A boolean per indicator that determines whether the indicator renders below the text (between background and text glyphs) or above the text (on top of rendered glyphs). [SCI-IND-10.1]
- **ValueFore_Mode**: A mode where the indicator colour is derived from the indicator value at each position rather than the indicator's configured foreground colour, enabling per-range colour variation within a single indicator number. [SCI-IND-10.1]
- **Change_History_Marker**: A line marker used to indicate edit history state — whether a line has been modified, saved, reverted to origin, or reverted to a previously-saved state. [SCI-MRK-10.3]
- **Bookmark_Marker**: A line marker placed by the user to mark lines of interest for quick navigation. [SCI-MRK-10.3]

## Requirements

### Requirement 1: Indicator Style Catalogue

**User Story:** As a theme designer or plugin author, I need a comprehensive set of indicator visual styles to represent different semantic meanings (errors, warnings, search matches, spelling, composition) with distinct and recognizable appearances.

**Source:** [SCI-IND-10.1] — 23 `IndicatorStyle` variants adapted from Scintilla.

#### Acceptance Criteria

1. THE text-decorations crate SHALL define an `IndicatorStyle` enum with the following variants: Plain, Squiggle, TT, Diagonal, Strike, Hidden, Box, RoundBox, StraightBox, Dash, Dots, SquiggleLow, DotBox, SquigglePixmap, CompositionThick, CompositionThin, FullBox, TextFore, Point, PointCharacter, Gradient, GradientCentre, PointTop.
2. THE `Plain` style SHALL render as a solid horizontal underline of configurable stroke width beneath the decorated text range.
3. THE `Squiggle` style SHALL render as a wavy (zigzag) underline beneath the decorated text range, with peak-to-peak pitch proportional to stroke width.
4. THE `TT` style SHALL render as a series of small "T" shapes (horizontal line with a descending tick) beneath the decorated text range.
5. THE `Diagonal` style SHALL render as repeated diagonal line segments (lower-left to upper-right) beneath the decorated text range.
6. THE `Strike` style SHALL render as a horizontal line through the vertical centre of the text line (strikethrough), not beneath the text.
7. THE `Hidden` style SHALL render nothing visually but SHALL still occupy indicator storage, allowing programmatic queries without visual output.
8. THE `Box` style SHALL render a rectangular outline around the full text height of the decorated range, using the configured outline alpha for border opacity.
9. THE `RoundBox` style SHALL render a rounded-corner rectangle with semi-transparent fill (using fill alpha) and semi-transparent border (using outline alpha) spanning the full text height.
10. THE `StraightBox` style SHALL render as `RoundBox` but with square corners (no corner radius).
11. THE `FullBox` style SHALL render as `StraightBox` but extending from the very top of the line to the very bottom (full line height, not just text height).
12. THE `Dash` style SHALL render as a dashed horizontal underline beneath the decorated text range.
13. THE `Dots` style SHALL render as a dotted horizontal underline (individual square dots) beneath the decorated text range.
14. THE `SquiggleLow` style SHALL render as a low-amplitude squiggle (half the height of the standard Squiggle) beneath the decorated text range.
15. THE `DotBox` style SHALL render a dotted rectangular outline around the full line height of the decorated range (dotted border, alternating opaque and transparent pixels).
16. THE `SquigglePixmap` style SHALL render a pre-computed anti-aliased squiggle pattern using an RGBA pixel image for smooth appearance at all DPI levels.
17. THE `CompositionThick` style SHALL render a thick underline (2 pixels at standard DPI) at the bottom of the line, suitable for indicating IME composition ranges.
18. THE `CompositionThin` style SHALL render a thin underline (1 pixel at standard DPI) at the bottom of the line, suitable for indicating confirmed IME composition.
19. THE `TextFore` style SHALL override the text foreground colour of the decorated range to the indicator's configured colour, without drawing any additional graphical element.
20. THE `Point` style SHALL render a small downward-pointing triangle at the left edge of the first decorated character, suitable for marking a position.
21. THE `PointCharacter` style SHALL render a small downward-pointing triangle at the horizontal centre of the first decorated character.
22. THE `PointTop` style SHALL render a small downward-pointing triangle at the top-left of the first decorated character (above the text line).
23. THE `Gradient` style SHALL render a top-to-bottom gradient fill from the indicator colour (at fill alpha) at the top to fully transparent at the bottom, spanning the full line height.
24. THE `GradientCentre` style SHALL render a gradient fill that is transparent at the top, reaches the indicator colour (at fill alpha) at the vertical centre, and fades back to transparent at the bottom.

---

### Requirement 2: Indicator Properties and Configuration

**User Story:** As a workbench developer, I need each indicator to have configurable properties (colour, alpha, stroke width, layer ordering, hover behaviour) so that different producers can tailor their indicators to be visually distinct and semantically clear.

**Source:** [SCI-IND-10.1] — `Indicator` class properties: sacNormal, sacHover, under, fillAlpha, outlineAlpha, strokeWidth, IndicFlag.

#### Acceptance Criteria

1. EACH Indicator SHALL have a configurable foreground colour (`fore`) used as the primary drawing colour for both line-based and fill-based styles.
2. EACH Indicator SHALL have an `under` property (boolean, default `false`). WHEN `under` is `true`, THE indicator SHALL render below the text glyphs (between background fill and text rendering). WHEN `under` is `false`, THE indicator SHALL render above the text glyphs.
3. EACH Indicator SHALL have a `fill_alpha` property (integer 0–255, default 30) controlling the opacity of the interior fill for box-style indicators (RoundBox, StraightBox, FullBox, DotBox, Gradient, GradientCentre).
4. EACH Indicator SHALL have an `outline_alpha` property (integer 0–255, default 50) controlling the opacity of the border/outline for box-style indicators (Box, RoundBox, StraightBox, DotBox, CompositionThick).
5. EACH Indicator SHALL have a `stroke_width` property (floating-point, default 1.0) controlling the line thickness for all line-based styles (Plain, Squiggle, TT, Diagonal, Strike, Dash, Dots, SquiggleLow).
6. EACH Indicator SHALL support a normal-state style/colour (`sac_normal`) and a hover-state style/colour (`sac_hover`). WHEN the hover state differs from the normal state, the indicator is considered "dynamic" and requires redraw on hover transitions.
7. THE text-decorations crate SHALL provide an `is_dynamic()` predicate that returns `true` when an indicator's hover state differs from its normal state, enabling the renderer to track mouse position for dynamic indicators only.
8. EACH Indicator SHALL support ValueFore mode via a flag (`IndicFlag::ValueFore`). WHEN ValueFore is enabled, THE indicator colour SHALL be derived from the lower 24 bits of the indicator value at each position (interpreted as RGB), overriding the configured `fore` colour.
9. ALL indicator properties SHALL be configurable via the theme system (cross-ref: `theme-and-appearance`), allowing themes to override default colours, alpha values, and stroke widths per indicator number.
10. WHEN the active theme changes, ALL visible indicators SHALL update their rendering immediately to reflect the new theme's colour and style definitions without requiring document re-decoration.

---

### Requirement 3: Decoration Storage (Run-Length Encoded Indicator Values)

**User Story:** As a document with sparse decorations (a few error underlines among thousands of lines), I need indicator storage to be memory-efficient so that having many potential indicator numbers does not consume memory proportional to document length for each unused indicator.

**Source:** [SCI-DEC] — `Decoration` class with `RunStyles<POS, int>` storage, `DecorationList` aggregate.

#### Acceptance Criteria

1. THE Decoration storage for each indicator number SHALL use run-length encoding, storing consecutive positions with the same indicator value as a single (value, run_length) pair rather than per-character storage.
2. WHEN no positions in the document have a non-zero value for a given indicator, THE Decoration for that indicator SHALL be empty (consuming O(1) memory) and SHALL be omittable from the Decoration_List.
3. THE Decoration_List SHALL create Decoration storage for an indicator lazily — only when the first non-zero value is set for that indicator number.
4. WHEN all values for a given indicator are reset to zero (the decoration becomes empty), THE Decoration_List SHALL remove that indicator's storage from the active list, freeing memory.
5. THE Decoration_List SHALL provide a `value_at(indicator, position)` method returning the indicator value at the given character position (0 if no decoration exists).
6. THE Decoration_List SHALL provide a `start_run(indicator, position)` method returning the start position of the run containing the given position for the specified indicator.
7. THE Decoration_List SHALL provide an `end_run(indicator, position)` method returning the end position (exclusive) of the run containing the given position for the specified indicator.
8. THE Decoration_List SHALL provide a `fill_range(indicator, position, value, length)` method that sets the indicator value for a contiguous range of characters, returning whether any values actually changed.
9. THE Decoration_List SHALL provide an `all_on_for(position)` method returning a bitmask of all indicator numbers that have non-zero values at the given position, for efficient aggregate queries during rendering.
10. THE memory consumption of Decoration storage SHALL scale with O(number_of_transitions) — the count of value changes — rather than O(document_length), ensuring that a 1MB document with 10 error underlines uses roughly the same memory as the same document with 5 error underlines (not proportional to document size).

---

### Requirement 4: Decoration Edit Synchronization

**User Story:** As a user editing a document with active decorations (search highlights, error underlines), I need decorations to remain correctly positioned after text insertions and deletions so that indicators always correspond to the intended text ranges.

**Source:** [SCI-DEC] — `InsertSpace`, `DeleteRange` on `DecorationList`; [WB] — undo/redo integration.

#### Acceptance Criteria

1. WHEN text is inserted at a position within the document, THE Decoration_List SHALL call `insert_space(position, insert_length)` on all active decorations, shifting all indicator values at or after the insertion point rightward by `insert_length` characters.
2. WHEN text is deleted from the document, THE Decoration_List SHALL call `delete_range(position, delete_length)` on all active decorations, removing indicator values in the deleted range and shifting subsequent values leftward by `delete_length` characters.
3. WHEN text is inserted at the end of a decorated run, THE insertion SHALL NOT extend the decoration into the newly inserted text — new characters SHALL receive value 0 (no decoration) by default.
4. WHEN text is inserted in the middle of a decorated run, THE run SHALL be split: the portion before the insertion retains the original value, the inserted characters receive value 0, and the portion after the insertion retains the original value at their new positions.
5. WHEN an undo operation reverses a text insertion, THE Decoration_List's position adjustments SHALL be reversed correspondingly (via the matching `delete_range`), restoring decorations to their pre-insertion positions.
6. WHEN an undo operation reverses a text deletion, THE Decoration_List's position adjustments SHALL be reversed correspondingly (via the matching `insert_space`), restoring decorations to their pre-deletion positions.
7. THE `insert_space` and `delete_range` operations SHALL execute in O(k × log n) time where k is the number of active indicators with non-empty decoration storage and n is the number of runs in each decoration.
8. AFTER any edit synchronization, THE invariant that the total length of all runs in a Decoration equals the document length SHALL hold.

---

### Requirement 5: Search Match Highlighting

**User Story:** As a user performing a FIND operation, I need the current match and all other matches to be visually highlighted in the document so that I can see where matches occur relative to my cursor position.

**Source:** [WB] — search match highlighting; cross-ref: `find-and-replace` spec.

#### Acceptance Criteria

1. THE text-decorations crate SHALL define a dedicated indicator number for the **current search match** — the match currently focused/selected by the find engine.
2. THE text-decorations crate SHALL define a separate dedicated indicator number for **all other matches** — all matches found by the current search that are not the currently focused match.
3. THE current-match indicator SHALL use a visually prominent style (default: StraightBox with a distinct highlight colour such as bright yellow/orange) that clearly distinguishes it from other matches.
4. THE all-matches indicator SHALL use a less prominent style (default: RoundBox with a subdued highlight colour such as pale yellow) that is visible but does not compete with the current match for attention.
5. WHEN the find engine reports match positions, THE find-and-replace subsystem SHALL call `fill_range` on the all-matches indicator for every match range, and `fill_range` on the current-match indicator for only the focused match.
6. WHEN the user navigates to the next/previous match (RFIND), THE current-match indicator SHALL move from the old match to the new match position, and the old position SHALL revert to the all-matches indicator style.
7. WHEN the search is cancelled or the search term is cleared, ALL search-related indicator values SHALL be reset to zero (decorations removed).
8. WHEN incremental search is active (search-as-you-type), THE match highlighting SHALL update in real time as the user types, adding and removing indicator ranges as the match set changes.
9. THE search highlighting colours SHALL be configurable via the theme system, with distinct defaults for light, dark, and high-contrast themes.
10. WHEN the document is edited while search highlighting is active, THE search decorations SHALL be invalidated and the find engine SHALL re-execute the search to update match positions (or the producer SHALL clear stale decorations).

---

### Requirement 6: Diagnostic Underlines (Error/Warning Indicators)

**User Story:** As a developer viewing a file with syntax errors or warnings, I need errors to be underlined with a distinct squiggle or underline style so that I can quickly identify and navigate to problematic code locations.

**Source:** [SCI-IND-10.1] — Squiggle indicator style for errors; [WB] — diagnostic integration.

#### Acceptance Criteria

1. THE text-decorations crate SHALL define dedicated indicator numbers for diagnostic severity levels: Error, Warning, Information, and Hint.
2. THE Error indicator SHALL default to a red Squiggle style underline, clearly signalling a compilation or syntax error.
3. THE Warning indicator SHALL default to a yellow/amber Squiggle style underline, signalling a potential issue that is not a hard error.
4. THE Information indicator SHALL default to a blue Plain underline, signalling informational diagnostics or suggestions.
5. THE Hint indicator SHALL default to a grey Dots underline, signalling low-priority hints or style suggestions.
6. WHEN a diagnostic producer (language service, linter plugin) reports diagnostic ranges, IT SHALL call `fill_range` on the appropriate severity indicator with the character range of each diagnostic.
7. WHEN diagnostics are resolved (e.g., the user fixes an error), THE diagnostic producer SHALL clear the indicator values for the resolved range by calling `fill_range` with value 0.
8. WHEN the user hovers over a diagnostic-decorated range, THE indicator SHALL transition to its hover state (if configured as dynamic), providing visual feedback that additional information is available.
9. DIAGNOSTIC indicators SHALL be rendered with `under = true` (below text) by default so that the squiggle/underline does not obscure the text content.
10. ALL diagnostic indicator colours and styles SHALL be overridable via the theme system.

---

### Requirement 7: Change History Markers

**User Story:** As a user editing a file, I need to see which lines have been modified, saved, or reverted so that I can understand the edit history at a glance without needing to diff against a saved version.

**Source:** [SCI-MRK-10.3] — `MarkerOutline::HistoryModified`, `HistorySaved`, `HistoryRevertedToOrigin`, `HistoryRevertedToModified`; [SCI-IND-10.1] — `IndicatorNumbers::History*` indicators for character-level change tracking; [WB] — change history markers.

#### Acceptance Criteria

1. THE text-decorations crate SHALL define line marker numbers for four change history states: Modified (unsaved changes), Saved (modified then saved), Reverted_To_Origin (reverted to the original file content), and Reverted_To_Modified (reverted to a previously modified state).
2. THE Modified marker SHALL display in the change-history gutter margin with a distinct colour (default: orange/amber bar) indicating the line has unsaved changes.
3. THE Saved marker SHALL display in the change-history gutter margin with a distinct colour (default: green bar) indicating the line was modified and then saved.
4. THE Reverted_To_Origin marker SHALL display in the change-history gutter margin with a distinct colour (default: blue bar) indicating the line was reverted to its original file content.
5. THE Reverted_To_Modified marker SHALL display in the change-history gutter margin with a distinct colour (default: yellow bar) indicating the line was reverted to a previously modified (but not saved) state.
6. IN ADDITION to line markers, THE text-decorations crate SHALL define character-level change history indicators (using the indicator system) for insertion and deletion tracking within lines, with dedicated indicator numbers for each combination of history state and change type (insertion vs deletion).
7. WHEN a line is edited, THE change-history system SHALL set the Modified marker on that line and clear any previous history marker.
8. WHEN the document is saved, ALL lines with the Modified marker SHALL transition to the Saved marker.
9. WHEN an undo operation reverts a line to its original content, THE marker SHALL transition to Reverted_To_Origin.
10. WHEN an undo operation reverts a line to a previously modified (but not original) state, THE marker SHALL transition to Reverted_To_Modified.
11. ALL change history marker colours SHALL be configurable via the theme system.

---

### Requirement 8: Bookmark Markers

**User Story:** As a user navigating a large file, I need to place and remove bookmarks on lines of interest so that I can quickly jump between marked locations using keyboard shortcuts or a bookmark list.

**Source:** [SCI-MRK-10.3] — `MarkerSymbol::Bookmark`, `VerticalBookmark`; [WB] — bookmark margin markers.

#### Acceptance Criteria

1. THE text-decorations crate SHALL define a dedicated line marker number for bookmarks.
2. THE bookmark marker SHALL display in a bookmark margin (symbol margin) using a recognizable bookmark shape (default: `Bookmark` symbol — a flag or page-corner shape).
3. WHEN the user toggles a bookmark on a line (via command or keyboard shortcut), THE bookmark marker SHALL be added to that line if not present, or removed if already present.
4. THE bookmark system SHALL support multiple simultaneous bookmarks across a document — there is no limit on the number of bookmarked lines.
5. THE text-decorations crate SHALL provide a method to query all lines with active bookmarks, enabling a "bookmark list" panel or "next bookmark" / "previous bookmark" navigation commands.
6. THE text-decorations crate SHALL provide `next_bookmark(from_line)` and `previous_bookmark(from_line)` methods that return the next/previous line with a bookmark marker relative to the given starting line, wrapping around the document if necessary.
7. WHEN a bookmarked line is deleted, THE bookmark marker SHALL be removed. WHEN lines are inserted above a bookmarked line, THE bookmark SHALL move with its document line (marker positions track document line numbers).
8. THE bookmark marker symbol, colours (fore, back), and margin width SHALL be configurable via the theme system.
9. ALL bookmark operations (toggle, next, previous, clear all) SHALL be registered as commands in the command-framework for keyboard shortcut and menu access.
10. THE text-decorations crate SHALL provide a `clear_all_bookmarks()` method that removes all bookmark markers from the document in a single operation.

---

### Requirement 9: Line Marker System

**User Story:** As a workbench developer, I need a general-purpose line marker system that can display various symbols in gutter margins so that plugins and subsystems can annotate lines with visual markers (breakpoints, code coverage, bookmarks, change history, etc.) without coupling to specific rendering code.

**Source:** [SCI-MRK-10.3] — `LineMarker` class, `MarkerSymbol` enum, marker fore/back/backSelected colours, alpha/layer support.

#### Acceptance Criteria

1. THE line marker system SHALL support up to 32 marker numbers (0–31), each with independently configurable symbol, colours, and rendering properties.
2. EACH line marker SHALL have a `symbol` property specifying its visual shape from the supported set: Circle, RoundRect, Arrow, SmallRect, ShortArrow, Empty, ArrowDown, Minus, Plus, VLine, LCorner, TCorner, BoxPlus, BoxPlusConnected, BoxMinus, BoxMinusConnected, LCornerCurve, TCornerCurve, CirclePlus, CirclePlusConnected, CircleMinus, CircleMinusConnected, Background, DotDotDot, Arrows, FullRect, LeftRect, Underline, Bookmark, VerticalBookmark, Bar.
3. EACH line marker SHALL have configurable `fore` (foreground), `back` (background), and `back_selected` (background when line is selected) colours.
4. EACH line marker SHALL have an `alpha` property (0–255) controlling the marker's opacity, and a `layer` property determining whether the marker renders in the base layer or an overlay layer.
5. EACH line marker SHALL have a `stroke_width` property for geometric shapes that are drawn with outlines.
6. THE line marker system SHALL support custom pixmap/RGBA image markers in addition to the geometric symbol set, for maximum extensibility.
7. THE line marker system SHALL provide methods to add a marker to a line (`marker_add(line, marker_number)`), remove a marker from a line (`marker_delete(line, marker_number)`), and query which markers are active on a line (`marker_get(line)` returning a Marker_Mask bitmask).
8. THE line marker system SHALL provide a `marker_next(from_line, marker_mask)` method that returns the next line at or after `from_line` that has any marker in the given mask set.
9. THE line marker system SHALL provide a `marker_previous(from_line, marker_mask)` method that returns the previous line at or before `from_line` that has any marker in the given mask set.
10. WHEN lines are inserted or deleted in the document, ALL marker assignments SHALL update to track their document lines — markers do not stay at fixed line indices, they move with their logical line content.

---

### Requirement 10: High-DPI Rendering and Pixel Alignment

**User Story:** As a user on a high-DPI display (Retina, 4K), I need text decorations to render crisply without blurring, so that underlines, squiggles, and box borders appear sharp at any display scaling factor.

**Source:** [SCI-IND-10.1] — `PixelAlignOutside`, `PixelAlign`, `pixelDivisions` for sub-pixel rendering; [WB] — high-DPI support.

#### Acceptance Criteria

1. ALL indicator drawing operations SHALL use pixel-aligned coordinates, snapping line positions and box edges to device-pixel boundaries to prevent anti-aliasing blur on straight edges.
2. THE text-decorations renderer SHALL query the current display's pixel divisions (scale factor) and adjust coordinate alignment accordingly — on a 2x display, positions SHALL align to half logical pixels.
3. THE `stroke_width` property SHALL scale with the display DPI factor so that a configured stroke width of 1.0 produces a visually consistent line thickness regardless of display scaling.
4. FOR box-style indicators (Box, RoundBox, StraightBox, FullBox, DotBox), THE bounding rectangle SHALL be pixel-aligned outward (expanded to the nearest device pixel boundary) to ensure clean rectangular edges.
5. FOR line-based indicators (Plain, Squiggle, Dash, Dots, SquiggleLow, TT, Diagonal), THE vertical position (y-coordinate) SHALL be pixel-aligned to ensure consistent baseline rendering across characters of different widths.
6. THE Squiggle and SquiggleLow styles SHALL adjust their peak-to-peak pitch based on stroke width so that the zigzag pattern remains visually balanced at all DPI levels.
7. THE SquigglePixmap style SHALL generate its anti-aliased pixel image at the appropriate resolution for the current display scale factor, regenerating if the scale factor changes.
8. THE Point, PointCharacter, and PointTop triangle markers SHALL have pixel-height dimensions derived from the available indicator area, ensuring proportional rendering at all DPI levels.

---

### Requirement 11: Hover Interaction

**User Story:** As a user hovering over a decorated text range (e.g., an error underline or a hyperlink indicator), I need the decoration to visually respond to hover so that I know I can interact with it (click for details, navigate to definition, etc.).

**Source:** [SCI-IND-10.1] — `sacHover` state, `IsDynamic()` predicate, click notification.

#### Acceptance Criteria

1. WHEN the mouse cursor enters a character range decorated by a dynamic indicator (one where `is_dynamic()` returns `true`), THE indicator SHALL transition from its normal appearance (style + colour) to its hover appearance (hover style + hover colour).
2. WHEN the mouse cursor leaves a dynamic indicator's range, THE indicator SHALL transition back from hover appearance to normal appearance.
3. THE hover transition SHALL occur without delay (immediate visual feedback on mouse move, not debounced).
4. THE Decoration_List SHALL provide a `click_notified` flag that tracks whether a click notification has been dispatched for the current hover position, enabling consuming code to distinguish hover-only from hover-then-click interactions.
5. WHEN the user clicks on a character position with one or more active indicators, THE text-decorations crate SHALL emit a decoration-click event containing the indicator number(s) and the character position, allowing consumers (diagnostic tooltip, hyperlink navigation) to respond.
6. ONLY dynamic indicators (those with differing normal vs hover states) SHALL trigger redraw on mouse movement; non-dynamic indicators SHALL NOT cause unnecessary repaint during mouse tracking.
7. THE renderer SHALL track the current hover position and efficiently determine which indicators are dynamic at that position using the `all_on_for` aggregate query combined with per-indicator `is_dynamic()` checks.

---

### Requirement 12: Modified Line Indicator in Gutter

**User Story:** As a user making edits to a file, I need a simple visual indicator in the gutter showing which lines have been modified since the file was last opened or saved, providing immediate feedback about the scope of my changes.

**Source:** [WB] — modified line indicators in gutter; [SCI-MRK-10.3] — LeftRect marker style for gutter bars.

#### Acceptance Criteria

1. THE text-decorations crate SHALL support a dedicated gutter column (or margin) for displaying modified-line indicators — a narrow colour bar adjacent to the line number column.
2. WHEN a document line has been modified since the last save, THE modified-line indicator SHALL display a coloured bar (default: amber/orange) in the change margin for that line.
3. WHEN a document line has been modified and subsequently saved, THE modified-line indicator SHALL transition to a different colour (default: green) indicating "saved changes".
4. WHEN a document line is reverted to its original content (via undo), THE modified-line indicator SHALL be removed from that line.
5. THE modified-line indicator margin SHALL be independently hideable via configuration, for users who prefer a cleaner gutter appearance.
6. THE modified-line indicator colours SHALL be configurable via the theme system with distinct defaults for dark and light themes.
7. THE modified-line indicators SHALL be consistent with the change history markers (Requirement 7) — they represent the same underlying state and SHALL be driven by the same data source, displayed in the same margin column.

---

### Requirement 13: Indicator Number Allocation and Namespaces

**User Story:** As a workbench platform with multiple decoration producers (search, diagnostics, language service, plugins), I need a clear allocation scheme for indicator numbers so that different producers do not conflict with each other.

**Source:** [SCI-IND-10.1] — `IndicatorNumbers::Container` (8), `Ime` (32–35), `History*` (36–43); [WB] — plugin extensibility.

#### Acceptance Criteria

1. THE text-decorations crate SHALL define indicator number ranges with the following allocation: indicators 0–7 for lexer/syntax-highlighting use, indicators 8–31 for container/application use (search, diagnostics, user plugins), indicators 32–35 reserved for IME composition, and indicators 36–43 reserved for change history tracking.
2. THE maximum indicator number SHALL be 43 (matching Scintilla's `IndicatorNumbers::Max`), providing a total of 44 indicator slots.
3. THE text-decorations crate SHALL define named constants for well-known indicator allocations: `INDICATOR_SEARCH_CURRENT`, `INDICATOR_SEARCH_ALL`, `INDICATOR_ERROR`, `INDICATOR_WARNING`, `INDICATOR_INFO`, `INDICATOR_HINT`, `INDICATOR_IME_*`, `INDICATOR_HISTORY_*`.
4. THE text-decorations crate SHALL provide a registry or allocation API that plugins can use to request an available indicator number from the container range (8–31), preventing number collisions between independent plugins.
5. WHEN a plugin requests an indicator number and all container-range numbers are allocated, THE allocation SHALL return an error indicating no available indicator slots remain.
6. THE lexer range (0–7) SHALL be managed exclusively by the syntax-highlighting subsystem; container code and plugins SHALL NOT write to indicator numbers below 8.
7. THE crate SHALL provide a `delete_lexer_decorations()` method that clears all indicator values in the lexer range (0–7) without affecting container or history indicators, for use when the lexer is re-run.

---

### Requirement 14: Rendering Pipeline Integration

**User Story:** As the viewport renderer, I need a clear contract for querying active decorations within a visible range and rendering them in the correct layer order (background → under-indicators → text → over-indicators → margin markers) so that all decorations compose correctly.

**Source:** [SCI-IND-10.1] — draw ordering (under property), layer composition; [SCI-MRK-10.3] — layer/alpha for margin markers; [WB] — rendering architecture.

#### Acceptance Criteria

1. THE rendering pipeline SHALL draw decorations in the following layer order per line: (1) line background markers (Background-symbol markers), (2) indicators with `under = true`, (3) text glyphs with syntax highlighting, (4) indicators with `under = false`, (5) selection overlay (if any), (6) gutter/margin markers.
2. THE text-decorations crate SHALL provide a method to retrieve all active indicator ranges intersecting a given character range (the visible viewport range), returning an iterator of (indicator_number, start, end, value) tuples for the renderer to draw.
3. THE text-decorations crate SHALL provide a method to retrieve the marker mask for a given document line, enabling the margin renderer to draw the appropriate symbols for that line.
4. WHEN multiple indicators overlap on the same character range, EACH indicator SHALL be drawn independently in indicator-number order (lower numbers first), allowing all overlapping indicators to be visible simultaneously.
5. THE text-decorations crate SHALL NOT perform rendering itself — it provides data and configuration; the actual drawing is performed by the viewport/editor-view layer using the platform's graphics API (egui/painter).
6. THE public API SHALL expose a `DecorationRenderer` trait defining the methods needed by the viewport to query and draw decorations, decoupling the decoration data model from the specific rendering technology.
7. WHEN the viewport requests decorations for a range, THE query SHALL complete in O(k × log n) time where k is the number of active indicators and n is the number of runs in each decoration, ensuring rendering is not bottlenecked by decoration queries.

---

### Requirement 15: Theme Integration

**User Story:** As a user switching between light, dark, and high-contrast themes, I need all text decorations and line markers to adapt their colours and rendering parameters automatically so that decorations remain visible and aesthetically consistent with the active theme.

**Source:** [WB] — configurable via theme system; cross-ref: `theme-and-appearance`.

#### Acceptance Criteria

1. THE text-decorations crate SHALL read indicator and marker colour/style definitions from the active theme configuration (cross-ref: `theme-and-appearance`), applying theme-defined values as overrides to the compiled defaults.
2. THE theme system SHALL define decoration colours in a dedicated `[decorations]` or `[indicators]` section of the theme TOML file, providing named entries for each well-known indicator and marker.
3. WHEN the active theme changes at runtime (hot-reload or user switch), THE text-decorations crate SHALL reload all indicator and marker colour/style properties from the new theme and trigger a viewport repaint to reflect the updated appearance.
4. THE theme SHALL provide per-indicator overrides for: `fore`, `fill_alpha`, `outline_alpha`, `stroke_width`, and optionally `style` (allowing a theme to change an indicator from Squiggle to Plain, for example).
5. THE theme SHALL provide per-marker overrides for: `fore`, `back`, `back_selected`, `alpha`, and optionally `symbol`.
6. FOR high-contrast themes, THE default indicator and marker colours SHALL use fully opaque, high-saturation colours with maximum contrast against the editor background, ensuring accessibility for users with low vision.
7. IF a theme does not define overrides for a given indicator or marker, THE text-decorations crate SHALL fall back to compiled default values (as specified in Requirements 1, 5, 6, 7, 8).
8. THE text-decorations crate SHALL validate theme-provided values (alpha clamped to 0–255, stroke_width clamped to 0.5–10.0, style must be a valid enum variant) and fall back to defaults for invalid entries, logging a warning.
