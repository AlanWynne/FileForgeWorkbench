# Requirements Document

## Introduction

This feature specifies the Viewport and Scrolling subsystem for FileForgeWorkbench — the `ff-viewport-and-scrolling` crate. The viewport model is the **GUI-independent component** that manages the visible window into a document, vertical and horizontal scroll state, caret visibility policies, and scroll behaviour.

The viewport-and-scrolling crate is responsible for:
- **Viewport state** — tracking which portion of the document is currently visible (`top_line`, `visible_count`, `horizontal_offset`)
- **Vertical scrollbar mapping** — a full-range scrollbar that maps the entire document line range [1, line_count] onto the scrollbar track, with proportional thumb size reflecting viewport-to-document ratio
- **Horizontal scrollbar mapping** — tracking horizontal scroll offset relative to the longest visible line (or longest line in the document)
- **Scroll commands** — handling Page Up, Page Down, Line Up, Line Down, and scroll-to-position operations with proper clamping
- **Caret visibility policies** — configurable rules governing how the viewport scrolls to keep the caret visible (slop, strict, jumps, even modes)
- **Scroll policies** — configurable rules for vertical and horizontal viewport movement in response to programmatic or user-driven scroll requests
- **Smooth scrolling** — support for both line-level scrolling (traditional) and pixel-level scrolling (smooth) for viewport transitions
- **Cursor-viewport coordination** — ensuring cursor movement commands scroll the viewport when the cursor would otherwise leave the visible area
- **Display-line awareness** — integration with display-line-mapping for correct scrolling when lines are wrapped, folded, or excluded

The viewport model is **owned by the editor session**, NOT by the GUI. This ensures testability and enables headless operation. GUI renderers query the viewport model to determine what to paint.

**Source references:**
- **[FFE-MVP-2]** = FileForgeEditor `mvp-implementation` Requirement 2 (viewport scrolling, cursor movement, scrollbar behaviour)
- **[FFE-SCROLL]** = FileForgeEditor `scrollbar-full-file-range` spec (full-range scrollbar bugfix, proportional thumb, precision interaction)
- **[SCI-EDIT-2.2]** = Scintilla Editor Requirement 2.2 (keyboard command handling, caret movement, PageUp/PageDown, lastXChosen column affinity, MovePositionTo with caret policies)
- **[SCI-EDIT-2.4]** = Scintilla EditModel Requirement 2.4 (xOffset for horizontal scroll, TopLineOfMain, LinesOnScreen, shared viewport state)
- **[WB]** = Workbench Platform Architecture Brief (GUI independence, command framework integration, crate separation)

## Glossary

- **Viewport**: The logical window into the document. Defined by `top_line` (first visible line), `visible_count` (number of lines that fit vertically), and `horizontal_offset` (horizontal scroll in columns or pixels). The viewport model is GUI-independent. [FFE-MVP-2, SCI-EDIT-2.4]
- **Top_Line**: A 1-based line number identifying the first document line currently visible at the top of the viewport. [FFE-MVP-2]
- **Visible_Count**: The number of display lines that fit vertically within the viewport at the current window geometry. Determined by the GUI shell and communicated to the viewport model. [FFE-MVP-2]
- **Horizontal_Offset**: The horizontal scroll position in pixels (for smooth scrolling) or columns (for character-grid mode), measured from the left edge of the text area. [FFE-MVP-2, SCI-EDIT-2.4]
- **Cursor_Line**: The 1-based document line where the editing cursor (caret) currently resides. [FFE-MVP-2]
- **Cursor_Column**: The 1-based column position of the caret within its line. [FFE-MVP-2]
- **Column_Affinity**: The remembered horizontal position (in pixels or columns) that the caret returns to when moving vertically through lines of varying length. Equivalent to Scintilla's `lastXChosen`. [SCI-EDIT-2.2]
- **Caret_Policy**: A set of flags (Slop, Strict, Jumps, Even) and a slop value that control how the viewport scrolls to keep the caret visible after movement. [SCI-EDIT-2.2]
- **Scroll_Policy**: Vertical and horizontal policy configuration controlling viewport movement magnitude and margins. [SCI-EDIT-2.2]
- **Slop_Zone**: A margin (in lines for vertical, pixels/columns for horizontal) near the viewport edge. When the caret enters this zone, the viewport scrolls to push the caret back toward the centre. [SCI-EDIT-2.2]
- **Smooth_Scrolling**: Pixel-level viewport transitions (as opposed to line-level jumps), providing visually fluid scroll animations. [SCI-EDIT-2.2]
- **Full_Range_Scrollbar**: A scrollbar whose track represents the entire document, not just the currently rendered content. The thumb position maps proportionally to `top_line` within `[1, line_count]`. [FFE-SCROLL]
- **Proportional_Thumb**: A scrollbar thumb whose size reflects the ratio `visible_count / line_count`, visually indicating how much of the document is currently visible. [FFE-SCROLL]
- **Display_Line**: A line as rendered on screen, which may differ from a document line when word-wrapping or folding is active. One document line may produce multiple display lines (wrapping) or zero display lines (folding/exclusion). [WB]
- **Line_Count**: The total number of document lines (from `document-model`). [FFE-MVP-2]
- **Max_Top_Line**: The maximum valid value for `top_line`, computed as `max(1, line_count - visible_count + 1)`, ensuring the viewport never scrolls past the last page. [FFE-SCROLL]
- **Scroll_Command**: A command dispatched through the command framework that modifies viewport state (e.g., `ScrollPageDown`, `ScrollLineUp`, `ScrollToLine`). [WB]
- **Resource_URI**: The VFS resource identifier for the document, used to associate viewport state with a specific file/buffer. [WB]

---

## Requirements

### Requirement 1: Viewport State Management

**User Story:** As a workbench editor component, I want a GUI-independent viewport model that tracks which portion of the document is visible, so that the viewport logic can be unit-tested without any GUI framework.

**Source:** [FFE-MVP-2] criteria 1; [SCI-EDIT-2.4] criteria 7, 17; [WB] GUI-independence principle.

#### Acceptance Criteria

1. THE viewport model SHALL maintain a `top_line` field (1-based) identifying the first document line currently visible at the top of the viewport. [FFE-MVP-2]
2. THE viewport model SHALL maintain a `visible_count` field representing the number of display lines that fit vertically in the viewport, as reported by the GUI shell. [FFE-MVP-2]
3. THE viewport model SHALL maintain a `horizontal_offset` field representing the horizontal scroll position in pixels. [SCI-EDIT-2.4]
4. THE viewport model SHALL maintain a `cursor_line` field (1-based) identifying the document line where the caret resides. [FFE-MVP-2]
5. THE viewport model SHALL maintain a `cursor_column` field (1-based) identifying the column position of the caret within its line. [FFE-MVP-2]
6. THE viewport model SHALL maintain a `column_affinity` field that records the preferred horizontal position for vertical cursor movement, equivalent to Scintilla's `lastXChosen`. [SCI-EDIT-2.2]
7. THE viewport model SHALL be owned by the editor session and SHALL NOT depend on any GUI framework type. [WB]
8. WHEN the GUI shell reports a change in available display height, THE viewport model SHALL update `visible_count` accordingly, and IF `top_line` now exceeds `max_top_line`, THE viewport model SHALL clamp `top_line` to `max_top_line`. [FFE-MVP-2]
9. THE viewport model SHALL expose accessor methods for all state fields, enabling GUI renderers to query the current viewport without mutation. [WB]
10. THE viewport model SHALL compute `max_top_line` as `max(1, total_display_lines - visible_count + 1)` where `total_display_lines` is obtained from `display-line-mapping` (or equals the document line count when no mapping is active). [FFE-SCROLL]

---

### Requirement 2: Vertical Scroll Commands

**User Story:** As a user navigating a document, I want Page Down, Page Up, Line Down, and Line Up to scroll the viewport predictably with proper clamping, so that I can traverse documents of any size without overshooting boundaries.

**Source:** [FFE-MVP-2] criteria 2–5, 12–17; [SCI-EDIT-2.2] criteria 10, 12, 13.

#### Acceptance Criteria

1. WHEN the Page Down scroll command is issued, THE viewport model SHALL advance `top_line` by `visible_count`, clamped to `max_top_line`. [FFE-MVP-2]
2. WHEN the Page Up scroll command is issued, THE viewport model SHALL retreat `top_line` by `visible_count`, clamped to 1. [FFE-MVP-2]
3. WHEN the Line Down scroll command is issued, THE viewport model SHALL advance `top_line` by 1, clamped to `max_top_line`. [FFE-MVP-2]
4. WHEN the Line Up scroll command is issued, THE viewport model SHALL retreat `top_line` by 1, clamped to 1. [FFE-MVP-2]
5. WHEN a scroll-to-line command is issued with a target line number, THE viewport model SHALL set `top_line` to the target, clamped to `[1, max_top_line]`. [FFE-SCROLL]
6. WHEN Page Down is issued, THE cursor SHALL move to the first visible line of the new page. [FFE-MVP-2]
7. WHEN Page Up is issued, THE cursor SHALL move to the first visible line of the new page. [FFE-MVP-2]
8. WHEN `top_line` is at `max_top_line`, THE Page Down command SHALL have no further effect on viewport position. [FFE-MVP-2]
9. WHEN `top_line` is at 1, THE Page Up command SHALL have no further effect on viewport position. [FFE-MVP-2]
10. ALL vertical scroll commands SHALL be dispatched through the command framework, allowing binding to configurable keys and scripting invocation. [WB]
11. WHEN `display-line-mapping` is active (wrapping or folding enabled), THE scroll commands SHALL operate on display lines rather than document lines, so that "one page" means `visible_count` display lines. [WB]

---

### Requirement 3: Cursor Movement and Viewport Coordination

**User Story:** As a user moving the cursor with arrow keys, I want the viewport to scroll automatically when the cursor would leave the visible area, so that I never lose sight of the editing position.

**Source:** [FFE-MVP-2] criteria 11–22; [SCI-EDIT-2.2] criteria 4, 12, 13.

#### Acceptance Criteria

1. WHEN the user presses the Down Arrow key, THE cursor SHALL move down one line. IF the cursor would move below the last visible line, THE viewport SHALL scroll down to keep the cursor visible. [FFE-MVP-2]
2. WHEN the user presses the Up Arrow key, THE cursor SHALL move up one line. IF the cursor would move above the first visible line, THE viewport SHALL scroll up to keep the cursor visible. [FFE-MVP-2]
3. WHEN the cursor reaches the last line of the document, THE Down Arrow key SHALL have no further effect on cursor position. [FFE-MVP-2]
4. WHEN the cursor is on the first line of the document, THE Up Arrow key SHALL have no further effect on cursor position. [FFE-MVP-2]
5. WHEN the user clicks a line within the viewport, THE viewport model SHALL move the cursor to that line. [FFE-MVP-2]
6. WHEN the user presses the Left Arrow key, THE cursor column (`cursor_column`) SHALL retreat by 1, clamped to column 1. [FFE-MVP-2]
7. WHEN the user presses the Right Arrow key, THE cursor column (`cursor_column`) SHALL advance by 1, clamped to the length of the current line plus 1 (end-of-line position). [FFE-MVP-2]
8. WHEN the cursor moves to a different line via an arrow key or click, THE cursor column SHALL reset to 1 on the new line, UNLESS column affinity is active (see Requirement 5). [FFE-MVP-2]
9. WHEN the cursor column changes, THE viewport model SHALL emit a state-change event so that the status bar column display can update. [FFE-MVP-2]
10. WHEN a cursor move gives text field keyboard focus, THE viewport model SHALL record this in its state so the GUI shell can transfer focus to the appropriate text row. [FFE-MVP-2]
11. WHEN the cursor moves vertically, THE viewport model SHALL apply the configured caret policy (Requirement 5) to determine the exact `top_line` adjustment needed. [SCI-EDIT-2.2]
12. WHEN the cursor moves horizontally and the caret column exceeds the visible horizontal range, THE viewport model SHALL adjust `horizontal_offset` to keep the cursor column visible. [SCI-EDIT-2.4]

---

### Requirement 4: Vertical Scrollbar — Full File Range

**User Story:** As a user working with a large file, I want the vertical scrollbar to represent the entire document range and have a proportional thumb, so that I can quickly jump to any position in the file by dragging the scrollbar.

**Source:** [FFE-MVP-2] criteria 6–7; [FFE-SCROLL] Expected Behaviour 2.1–2.3 and Properties 1–3.

#### Acceptance Criteria

1. THE viewport model SHALL compute a scrollbar position as a fraction: `(top_line - 1) / (max_top_line - 1)`, mapping `top_line = 1` to fraction `0.0` and `top_line = max_top_line` to fraction `1.0`. [FFE-SCROLL]
2. THE viewport model SHALL compute a scrollbar thumb size as a ratio: `visible_count / total_display_lines`, visually representing the fraction of the document currently visible. WHEN the entire document fits in the viewport (`total_display_lines <= visible_count`), THE thumb size ratio SHALL be `1.0`. [FFE-SCROLL]
3. WHEN the user drags the vertical scrollbar to a position (expressed as a fraction `f` in `[0.0, 1.0]`), THE viewport model SHALL set `top_line` to `round(1 + f * (max_top_line - 1))`, clamped to `[1, max_top_line]`. [FFE-SCROLL]
4. WHEN the user drags the vertical scrollbar to the maximum position (fraction `1.0`), THE viewport model SHALL set `top_line` to `max_top_line`, allowing full navigation to the last page of the file. [FFE-SCROLL]
5. WHEN the user drags the vertical scrollbar to the minimum position (fraction `0.0`), THE viewport model SHALL set `top_line` to 1. [FFE-SCROLL]
6. THE scrollbar mapping SHALL be a pure function of `(top_line, max_top_line, visible_count, total_display_lines)` with no dependency on GUI state. [WB]
7. WHEN `total_display_lines <= visible_count` (entire document fits in viewport), THE viewport model SHALL indicate that the scrollbar should be disabled or hidden, and `top_line` SHALL remain 1. [FFE-SCROLL]
8. THE scrollbar fraction-to-top-line mapping SHALL be invertible: converting `top_line` to a fraction and back SHALL produce the original `top_line` (round-trip property). [FFE-SCROLL]
9. WHEN `display-line-mapping` is active, THE scrollbar SHALL map against `total_display_lines` (which accounts for wrapped and folded lines) rather than raw document line count. [WB]

---

### Requirement 5: Caret Visibility Policies

**User Story:** As a workbench developer, I want configurable caret policies that control how aggressively the viewport scrolls to keep the caret visible, so that different editing workflows (code reading vs. active editing) can have appropriate scroll behaviour.

**Source:** [SCI-EDIT-2.2] criteria 12, 13; Scintilla `CaretPolicySlop` struct and `XYScrollToMakeVisible`.

#### Acceptance Criteria

1. THE viewport model SHALL support a `CaretPolicy` configuration with four boolean flags: `slop`, `strict`, `jumps`, `even`, and an integer `slop_lines` value (vertical) or `slop_pixels` value (horizontal). [SCI-EDIT-2.2]
2. WHEN `slop` is true, THE viewport model SHALL define a visibility zone of `slop_lines` lines from the top and bottom edges of the viewport. IF the caret enters this zone, THE viewport SHALL scroll to push the caret back toward the interior. [SCI-EDIT-2.2]
3. WHEN `strict` is true, THE viewport model SHALL enforce the slop zone strictly — the viewport SHALL always scroll to ensure the caret is outside the slop zone, even if the caret is already visible. [SCI-EDIT-2.2]
4. WHEN `jumps` is true AND the viewport needs to scroll, THE viewport model SHALL scroll by a larger amount (3× the slop value) to reduce the frequency of subsequent scrolling. [SCI-EDIT-2.2]
5. WHEN `even` is true, THE viewport model SHALL apply the same slop zone symmetrically to both top and bottom (vertical) or left and right (horizontal). [SCI-EDIT-2.2]
6. WHEN no caret policy flags are set (default minimal policy), THE viewport model SHALL perform the minimal scroll needed to bring the caret into the visible area — one line for vertical, minimal offset for horizontal. [SCI-EDIT-2.2]
7. THE caret policy SHALL be configurable separately for vertical and horizontal axes, allowing different behaviours for each direction. [SCI-EDIT-2.2]
8. WHEN the caret moves and `MovePositionTo` (or its Rust equivalent) is invoked, THE viewport model SHALL apply the current caret policy to compute the new `top_line` and `horizontal_offset`. [SCI-EDIT-2.2]
9. THE caret policy configuration SHALL be persisted as part of the workbench configuration (references `configuration-system`). [WB]

---

### Requirement 6: Column Affinity (Vertical Movement Memory)

**User Story:** As a user moving the cursor vertically through lines of varying length, I want the cursor to return to my preferred column when passing through shorter lines, so that vertical navigation feels natural and predictable.

**Source:** [SCI-EDIT-2.2] criteria 12 — `lastXChosen` column affinity.

#### Acceptance Criteria

1. WHEN the cursor moves vertically (Up Arrow, Down Arrow, Page Up, Page Down), THE viewport model SHALL use the stored `column_affinity` value to determine the target column on the new line. [SCI-EDIT-2.2]
2. WHEN the cursor moves horizontally (Left Arrow, Right Arrow, click, Home, End), THE viewport model SHALL update `column_affinity` to the new cursor column. [SCI-EDIT-2.2]
3. IF the target line is shorter than the `column_affinity` value, THE cursor SHALL be placed at the end of the line (last valid column), but `column_affinity` SHALL be preserved for future vertical movements. [SCI-EDIT-2.2]
4. IF the target line is long enough to accommodate the `column_affinity` value, THE cursor SHALL be placed at that column. [SCI-EDIT-2.2]
5. WHEN the cursor is explicitly positioned (by click, Home, End, or any horizontal command), THE `column_affinity` SHALL be reset to the new cursor column. [SCI-EDIT-2.2]
6. THE `column_affinity` value SHALL be stored in pixels (for proportional fonts) or columns (for monospace/character-grid mode), matching the configured font metric mode. [SCI-EDIT-2.2]

---

### Requirement 7: Horizontal Scrollbar

**User Story:** As a user viewing lines wider than the viewport, I want a horizontal scrollbar that reflects my horizontal scroll position relative to the longest visible line, so that I can pan across wide content.

**Source:** [FFE-MVP-2] criteria 8–9; [SCI-EDIT-2.4] criteria 7.

#### Acceptance Criteria

1. THE viewport model SHALL compute the horizontal scrollbar position as a ratio: `horizontal_offset / max_horizontal_extent`, where `max_horizontal_extent` is the width of the longest visible line (or longest line in the document, depending on configuration) minus the viewport width. [FFE-MVP-2]
2. WHEN the user drags the horizontal scrollbar, THE viewport model SHALL update `horizontal_offset` to the proportional position within `[0, max_horizontal_extent]`. [FFE-MVP-2]
3. WHEN all visible lines fit within the viewport width, THE viewport model SHALL indicate that the horizontal scrollbar should be disabled or hidden, and `horizontal_offset` SHALL remain 0. [FFE-MVP-2]
4. WHEN the user scrolls horizontally (via scrollbar drag, keyboard, or mouse wheel), THE `horizontal_offset` SHALL be clamped to `[0, max_horizontal_extent]`. [FFE-MVP-2]
5. THE `max_horizontal_extent` SHALL be recalculated whenever the visible content changes (scroll, edit, resize) or when the longest-line metric changes. [SCI-EDIT-2.4]
6. THE horizontal scrollbar mapping SHALL be a pure function of `(horizontal_offset, max_horizontal_extent, viewport_width)` with no GUI dependency. [WB]
7. WHEN word-wrap is enabled (via `display-line-mapping`), THE horizontal scrollbar SHALL be disabled since all content is wrapped to fit the viewport width. [WB]

---

### Requirement 8: Mouse Wheel Scrolling

**User Story:** As a user scrolling with the mouse wheel, I want the viewport to scroll smoothly by a configurable number of lines per wheel tick, so that mouse-wheel navigation feels responsive and natural.

**Source:** [FFE-SCROLL] design — mouse wheel events replacing ScrollArea; [SCI-EDIT-2.2] smooth scroll concepts.

#### Acceptance Criteria

1. WHEN the user scrolls the mouse wheel vertically, THE viewport model SHALL adjust `top_line` by a configurable number of lines per wheel tick (default: 3 lines per tick). [FFE-SCROLL]
2. WHEN scrolling the mouse wheel down, THE viewport model SHALL advance `top_line`, clamped to `max_top_line`. [FFE-SCROLL]
3. WHEN scrolling the mouse wheel up, THE viewport model SHALL retreat `top_line`, clamped to 1. [FFE-SCROLL]
4. WHEN the user scrolls the mouse wheel horizontally (Shift+wheel or horizontal wheel on supported hardware), THE viewport model SHALL adjust `horizontal_offset` by a configurable number of pixels/columns per tick. [SCI-EDIT-2.4]
5. THE lines-per-wheel-tick value SHALL be configurable through the configuration system (references `configuration-system`). [WB]
6. WHEN smooth scrolling is enabled (Requirement 9), THE mouse wheel SHALL produce pixel-level scroll increments rather than whole-line jumps. [SCI-EDIT-2.2]

---

### Requirement 9: Smooth Scrolling

**User Story:** As a user who prefers fluid visual transitions, I want an option for pixel-level smooth scrolling rather than jumping by whole lines, so that scroll animations feel polished and visually continuous.

**Source:** Scintilla smooth scrolling concepts; [WB] GUI-independence (logic computes target positions, GUI shell animates).

#### Acceptance Criteria

1. THE viewport model SHALL support a `scroll_mode` configuration with two values: `Line` (traditional whole-line scrolling) and `Smooth` (pixel-level sub-line scrolling). [SCI-EDIT-2.2]
2. WHEN `scroll_mode` is `Line`, THE viewport model SHALL round all vertical scroll positions to whole line boundaries (integer `top_line` values). [FFE-MVP-2]
3. WHEN `scroll_mode` is `Smooth`, THE viewport model SHALL maintain an additional `pixel_offset` field representing the sub-line vertical scroll position in pixels (range `[0, line_height)`). [SCI-EDIT-2.2]
4. WHEN smooth scrolling is active AND a scroll command targets a specific line, THE viewport model SHALL compute the target pixel position and expose it for the GUI shell to animate toward. [WB]
5. THE smooth scrolling logic SHALL remain GUI-independent — the viewport model computes target positions and velocities; the GUI shell performs the actual animation interpolation. [WB]
6. WHEN smooth scrolling is active, THE scrollbar position SHALL reflect the pixel-accurate scroll position (not just the line-level approximation). [SCI-EDIT-2.2]
7. THE `scroll_mode` SHALL be configurable through the configuration system and hot-reloadable without restarting the editor. [WB]

---

### Requirement 10: Scroll Commands Integration with Command Framework

**User Story:** As a workbench developer, I want all scroll operations to be routable through the command framework, so that scroll actions can be bound to keys, invoked from scripts, and integrated with undo/redo where appropriate.

**Source:** [WB] command-driven architecture; [SCI-EDIT-2.2] KeyCommand dispatch for scroll messages.

#### Acceptance Criteria

1. THE viewport crate SHALL define the following scroll commands for registration with the command framework: `ScrollLineUp`, `ScrollLineDown`, `ScrollPageUp`, `ScrollPageDown`, `ScrollToLine(line)`, `ScrollToTop`, `ScrollToBottom`, `ScrollHorizontal(offset)`. [WB]
2. WHEN a scroll command is dispatched through the command framework, THE viewport model SHALL execute the corresponding state mutation and emit appropriate state-change events. [WB]
3. THE scroll commands SHALL be bindable to configurable keyboard shortcuts via the command framework's key-mapping system. [WB]
4. THE scroll commands SHALL be invocable from Lua macros via `editor.command("ScrollPageDown")` or equivalent scripting API. [WB]
5. THE viewport model SHALL emit a `ViewportChanged` event (or equivalent notification) after any scroll state mutation, allowing UI renderers, status bars, and other observers to react. [WB]
6. SCROLL commands SHALL NOT be recorded on the undo stack — scroll position changes are navigation, not document modifications. [WB]

---

### Requirement 11: Display Line Mapping Integration

**User Story:** As a workbench developer, I want the viewport model to correctly handle wrapped, folded, and excluded lines, so that scrolling and caret visibility work correctly regardless of the display mode.

**Source:** [WB] integration with `display-line-mapping`; [SCI-EDIT-2.4] IContractionState.

#### Acceptance Criteria

1. THE viewport model SHALL accept a reference (trait object) to a `DisplayLineMapper` (from the `display-line-mapping` crate) that translates between document lines and display lines. [WB]
2. WHEN `display-line-mapping` reports that a document line is folded or excluded, THE viewport model SHALL skip that line when scrolling (it does not consume a visible row). [WB]
3. WHEN a document line is word-wrapped into multiple display lines, THE viewport model SHALL account for all display sub-lines when computing `visible_count` usage, page sizes, and scrollbar metrics. [WB]
4. WHEN no `DisplayLineMapper` is provided (or mapping is identity), THE viewport model SHALL treat each document line as exactly one display line (1:1 mapping). [WB]
5. THE `total_display_lines` used for scrollbar calculations (Requirement 4) SHALL come from the `DisplayLineMapper` rather than the raw document line count. [WB]
6. WHEN a fold or exclusion state changes, THE viewport model SHALL recalculate `max_top_line` and clamp `top_line` if necessary to prevent scrolling past the new document end. [WB]

---

### Requirement 12: Viewport State Persistence and Restoration

**User Story:** As a user who closes and reopens files, I want the viewport position and cursor location to be restored when I reopen a document, so that I can resume editing exactly where I left off.

**Source:** [WB] session restore; [FFE-MVP-2] viewport as part of editor session state.

#### Acceptance Criteria

1. THE viewport model SHALL expose a serialisable snapshot of its state (`top_line`, `cursor_line`, `cursor_column`, `horizontal_offset`, `column_affinity`) for session persistence. [WB]
2. WHEN a document is reopened and a persisted viewport snapshot is available, THE viewport model SHALL restore the saved state, clamped to the current document boundaries (in case the document has been externally modified). [WB]
3. IF the persisted `top_line` exceeds the current document's `max_top_line`, THE viewport model SHALL clamp to `max_top_line`. [WB]
4. IF the persisted `cursor_line` exceeds the current document's line count, THE viewport model SHALL clamp to the last line. [WB]
5. THE viewport state snapshot SHALL be serialisable to a format compatible with the session persistence mechanism (references `startup-and-session`). [WB]

---

### Requirement 13: Scrollbar Precision for Large Files

**User Story:** As a user working with very large files (millions of lines), I want the scrollbar to provide precision interaction even when the file-to-pixel ratio is extreme, so that I can navigate to specific regions without overshooting.

**Source:** [FFE-SCROLL] precision scrollbar interaction; [SCI-EDIT-2.2] fine-grained scroll control.

#### Acceptance Criteria

1. WHEN the document has more lines than there are physical pixels in the scrollbar track, THE viewport model SHALL map scrollbar positions using 64-bit arithmetic to avoid precision loss. [FFE-SCROLL]
2. THE scrollbar-to-top-line mapping SHALL be monotonically increasing: moving the scrollbar thumb by even one pixel SHALL always produce a distinct (or unchanged) `top_line` — never a decrease. [FFE-SCROLL]
3. WHEN the user holds Shift while dragging the scrollbar (or the platform equivalent), THE viewport model SHALL enter a "precision drag" mode that scales the drag sensitivity (e.g., 1 pixel of mouse movement = 1 line of scroll), enabling fine-grained positioning in large files. [FFE-SCROLL]
4. THE scrollbar computation SHALL not use floating-point for the final `top_line` determination when the document exceeds 1 million lines — integer arithmetic with proper rounding SHALL be used to prevent cumulative precision errors. [FFE-SCROLL]
5. THE viewport model SHALL support a tooltip-style feedback mechanism during scrollbar drag, providing the current `top_line` value to the GUI shell for display (e.g., "Line 1,234,567 of 5,000,000"). [FFE-SCROLL]

