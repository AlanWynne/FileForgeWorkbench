# Implementation Plan: Viewport and Scrolling (`ff-viewport-and-scrolling`)

## Overview

This plan implements the viewport and scrolling subsystem for FileForgeWorkbench — a GUI-independent model that tracks the visible portion of a document, manages scroll state, caret visibility policies, column affinity, smooth scrolling, and scrollbar mapping. The crate integrates with `ff-document-model` for line counts, `ff-display-line-mapping` for wrapped/folded lines, and `ff-command` for scroll command dispatch.

---

## Tasks

- [x] 1. Crate scaffolding and core types
  - [x] 1.1 Create `crates/ff-viewport-and-scrolling/Cargo.toml` with dependencies on `ff-document-model`, `ff-logging`, `serde`, `thiserror`, and dev-dependencies on `proptest`, `pretty_assertions`
  - [x] 1.2 Create `src/lib.rs` with module declarations and public API re-exports
  - [x] 1.3 Create `src/types.rs` with newtypes: `DisplayLine(u64)`, `PixelOffset(f64)`, `ColumnOffset(u64)`, `ScrollFraction(f64)`, `WheelTicks(i32)`
  - [x] 1.4 Create `src/error.rs` with `ViewportError` enum following `[viewport] operation: description` format
  - [x] 1.5 Register crate in workspace root `Cargo.toml`

- [x] 2. Viewport state model (Requirement 1)
  - [x] 2.1 Create `src/viewport.rs` with `ViewportState` struct containing `top_line`, `visible_count`, `horizontal_offset`, `cursor_line`, `cursor_column`, `column_affinity` fields
  - [x] 2.2 Implement `ViewportState::new()` constructor with default values (top_line=1, visible_count=1, cursor at 1:1)
  - [x] 2.3 Implement `max_top_line` computation: `max(1, total_display_lines - visible_count + 1)`
  - [x] 2.4 Implement `set_visible_count` method that updates visible_count and clamps top_line to max_top_line
  - [x] 2.5 Implement read-only accessor methods for all state fields
  - [x] 2.6 Write unit tests for viewport state construction and clamping on resize
  - [x] 2.7 Write property test: viewport state invariants (top_line always in [1, max_top_line], cursor_line >= 1)

- [x] 3. Vertical scroll commands (Requirement 2)
  - [x] 3.1 Implement `scroll_page_down` method: advance top_line by visible_count, clamp to max_top_line, move cursor to first visible line
  - [x] 3.2 Implement `scroll_page_up` method: retreat top_line by visible_count, clamp to 1, move cursor to first visible line
  - [x] 3.3 Implement `scroll_line_down` method: advance top_line by 1, clamp to max_top_line
  - [x] 3.4 Implement `scroll_line_up` method: retreat top_line by 1, clamp to 1
  - [x] 3.5 Implement `scroll_to_line(target)` method: set top_line clamped to [1, max_top_line]
  - [x] 3.6 Implement `scroll_to_top` and `scroll_to_bottom` convenience methods
  - [x] 3.7 Write unit tests for all vertical scroll operations including boundary clamping
  - [x] 3.8 Write property test: scroll commands never produce out-of-bounds top_line

- [x] 4. Cursor movement and viewport coordination (Requirement 3)
  - [x] 4.1 Create `src/cursor.rs` with cursor movement methods on ViewportState
  - [x] 4.2 Implement `move_cursor_down`: advance cursor_line by 1, scroll viewport if cursor exits visible area
  - [x] 4.3 Implement `move_cursor_up`: retreat cursor_line by 1, scroll viewport if cursor exits visible area
  - [x] 4.4 Implement `move_cursor_left`: retreat cursor_column by 1, clamp to 1
  - [x] 4.5 Implement `move_cursor_right(line_length)`: advance cursor_column by 1, clamp to line_length+1
  - [x] 4.6 Implement `set_cursor_position(line, column)`: click-to-position with viewport adjustment
  - [x] 4.7 Implement horizontal viewport adjustment: update horizontal_offset when cursor_column exceeds visible range
  - [x] 4.8 Write unit tests for cursor movement and auto-scroll coordination
  - [x] 4.9 Write property test: cursor movement with viewport auto-scroll keeps cursor within visible bounds

- [x] 5. Vertical scrollbar mapping (Requirement 4)
  - [x] 5.1 Create `src/scrollbar.rs` with `ScrollbarState` struct
  - [x] 5.2 Implement `scrollbar_position()`: compute fraction `(top_line - 1) / (max_top_line - 1)` mapping [1, max_top_line] → [0.0, 1.0]
  - [x] 5.3 Implement `thumb_size()`: compute ratio `visible_count / total_display_lines`, clamped to [0.0, 1.0]
  - [x] 5.4 Implement `scroll_to_fraction(f)`: set top_line = round(1 + f * (max_top_line - 1)), clamped
  - [x] 5.5 Implement `is_scrollbar_disabled()`: returns true when total_display_lines <= visible_count
  - [x] 5.6 Write unit tests for scrollbar fraction mapping at boundaries (0.0, 0.5, 1.0)
  - [x] 5.7 Write property test: scrollbar round-trip (top_line → fraction → top_line produces original value)

- [x] 6. Caret visibility policies (Requirement 5)
  - [x] 6.1 Create `src/caret_policy.rs` with `CaretPolicy` struct: `slop`, `strict`, `jumps`, `even` flags and `slop_lines`/`slop_pixels` values
  - [x] 6.2 Implement default caret policy (minimal scroll to make caret visible)
  - [x] 6.3 Implement slop zone logic: scroll when caret enters slop_lines margin from viewport edges
  - [x] 6.4 Implement strict mode: always enforce slop zone even when caret is already visible
  - [x] 6.5 Implement jumps mode: scroll by 3× slop value when scrolling is needed
  - [x] 6.6 Implement even mode: symmetric slop application to both edges
  - [x] 6.7 Implement separate vertical and horizontal caret policies
  - [x] 6.8 Integrate caret policy into cursor movement methods (apply_caret_policy on move)
  - [x] 6.9 Write unit tests for each caret policy mode (default, slop, strict, jumps, even)
  - [x] 6.10 Write property test: caret policy always ensures cursor is visible after viewport adjustment

- [x] 7. Column affinity (Requirement 6)
  - [x] 7.1 Create `src/affinity.rs` with column affinity logic
  - [x] 7.2 Implement affinity preservation during vertical movement: use stored column_affinity as target column
  - [x] 7.3 Implement affinity update on horizontal movement: reset column_affinity to new cursor_column
  - [x] 7.4 Implement short-line clamping: place cursor at end of line but preserve affinity value
  - [x] 7.5 Implement long-line restoration: place cursor at affinity column when line is long enough
  - [x] 7.6 Write unit tests for column affinity across lines of varying length
  - [x] 7.7 Write property test: column affinity is preserved across vertical moves through short lines

- [x] 8. Horizontal scrollbar (Requirement 7)
  - [x] 8.1 Add horizontal scrollbar computation to `src/scrollbar.rs`
  - [x] 8.2 Implement `horizontal_scrollbar_position()`: ratio of horizontal_offset / max_horizontal_extent
  - [x] 8.3 Implement `set_horizontal_offset(fraction)`: update horizontal_offset from scrollbar fraction
  - [x] 8.4 Implement `max_horizontal_extent` calculation based on longest visible line minus viewport width
  - [x] 8.5 Implement `is_horizontal_scrollbar_disabled()`: true when all content fits viewport width
  - [x] 8.6 Implement horizontal scrollbar disable when word-wrap is enabled
  - [x] 8.7 Write unit tests for horizontal scrollbar mapping and word-wrap disable
  - [x] 8.8 Write property test: horizontal_offset always in [0, max_horizontal_extent]

- [x] 9. Mouse wheel scrolling (Requirement 8)
  - [x] 9.1 Create `src/wheel.rs` with mouse wheel handling
  - [x] 9.2 Implement `scroll_wheel_vertical(ticks, lines_per_tick)`: adjust top_line by ticks * lines_per_tick, clamped
  - [x] 9.3 Implement `scroll_wheel_horizontal(ticks, pixels_per_tick)`: adjust horizontal_offset, clamped
  - [x] 9.4 Implement configurable lines_per_tick (default 3) with getter/setter
  - [x] 9.5 Implement smooth wheel integration: produce pixel offsets instead of line jumps when smooth mode active
  - [x] 9.6 Write unit tests for wheel scroll with various tick counts and configurations
  - [x] 9.7 Write property test: wheel scrolling never exceeds document bounds

- [x] 10. Smooth scrolling (Requirement 9)
  - [x] 10.1 Create `src/smooth.rs` with `ScrollMode` enum (`Line`, `Smooth`) and pixel offset tracking
  - [x] 10.2 Implement `pixel_offset` field in ViewportState for sub-line vertical position
  - [x] 10.3 Implement scroll mode switching: Line mode rounds to whole lines, Smooth mode allows sub-line
  - [x] 10.4 Implement target position computation for smooth scroll commands (GUI shell performs animation)
  - [x] 10.5 Implement pixel-accurate scrollbar position when smooth mode is active
  - [x] 10.6 Implement hot-reloadable scroll_mode via configuration integration point
  - [x] 10.7 Write unit tests for smooth scroll pixel offset management
  - [x] 10.8 Write property test: pixel_offset is always in [0, line_height) when smooth mode is active

- [x] 11. Command framework integration (Requirement 10)
  - [x] 11.1 Create `src/commands.rs` with scroll command definitions: `ScrollLineUp`, `ScrollLineDown`, `ScrollPageUp`, `ScrollPageDown`, `ScrollToLine`, `ScrollToTop`, `ScrollToBottom`, `ScrollHorizontal`
  - [x] 11.2 Implement command handler that dispatches scroll commands to ViewportState methods
  - [x] 11.3 Implement `ViewportChanged` event emission after all scroll state mutations
  - [x] 11.4 Implement command metadata (display name, description, default shortcut, category) for each scroll command
  - [x] 11.5 Ensure scroll commands are NOT recorded on the undo stack (navigation only)
  - [x] 11.6 Write unit tests for command dispatch and event emission
  - [x] 11.7 Write property test: ViewportChanged event is always emitted after any state mutation

- [x] 12. Display line mapping integration (Requirement 11)
  - [x] 12.1 Create `src/display_mapping.rs` with `DisplayLineMapper` trait definition (or import from upstream crate)
  - [x] 12.2 Implement viewport operations in terms of display lines when a mapper is provided
  - [x] 12.3 Implement identity mapping fallback when no DisplayLineMapper is present (1:1 doc→display)
  - [x] 12.4 Implement folded/excluded line skipping during scroll operations
  - [x] 12.5 Implement wrapped line accounting in visible_count and page size calculations
  - [x] 12.6 Implement max_top_line recalculation on fold/exclusion state change
  - [x] 12.7 Write unit tests with mock DisplayLineMapper for wrapping and folding scenarios
  - [x] 12.8 Write property test: scrollbar total always equals DisplayLineMapper total_display_lines

- [x] 13. Viewport state persistence (Requirement 12)
  - [x] 13.1 Create `src/persistence.rs` with `ViewportSnapshot` struct (serde Serialize/Deserialize)
  - [x] 13.2 Implement `ViewportState::snapshot()` → ViewportSnapshot serialisation
  - [x] 13.3 Implement `ViewportState::restore(snapshot, line_count)` with boundary clamping
  - [x] 13.4 Implement clamping rules: top_line to max_top_line, cursor_line to line_count, cursor_column preserved
  - [x] 13.5 Write unit tests for snapshot/restore round-trip and clamping on shortened documents
  - [x] 13.6 Write property test: restore(snapshot(state)) produces equivalent state when document unchanged

- [x] 14. Scrollbar precision for large files (Requirement 13)
  - [x] 14.1 Implement 64-bit integer arithmetic for scrollbar mapping when line_count > 1_000_000
  - [x] 14.2 Implement monotonically increasing scrollbar-to-top-line mapping (no decreases on forward drag)
  - [x] 14.3 Implement precision drag mode: 1 pixel = 1 line scroll sensitivity scaling
  - [x] 14.4 Implement tooltip feedback mechanism: expose current top_line during drag for GUI tooltip display
  - [x] 14.5 Write unit tests for precision mapping with multi-million line documents
  - [x] 14.6 Write property test: scrollbar mapping is monotonically non-decreasing across entire fraction range

- [x] 15. Integration tests and documentation
  - [x] 15.1 Write integration test: full scroll scenario (open document → page down → cursor move → scroll to bottom → restore)
  - [x] 15.2 Write integration test: viewport with display line mapping (wrapping + folding + scrollbar)
  - [x] 15.3 Add crate-level documentation in `src/lib.rs` with usage examples
  - [x] 15.4 Add `README.md` for the crate with architecture overview

---

## Acceptance Criteria Coverage

| Task | Requirements Covered |
|------|---------------------|
| 1    | Crate structure (Req 7 cross-cutting) |
| 2    | Requirement 1 (AC 1–10) |
| 3    | Requirement 2 (AC 1–11) |
| 4    | Requirement 3 (AC 1–12) |
| 5    | Requirement 4 (AC 1–9) |
| 6    | Requirement 5 (AC 1–9) |
| 7    | Requirement 6 (AC 1–6) |
| 8    | Requirement 7 (AC 1–7) |
| 9    | Requirement 8 (AC 1–6) |
| 10   | Requirement 9 (AC 1–7) |
| 11   | Requirement 10 (AC 1–6) |
| 12   | Requirement 11 (AC 1–6) |
| 13   | Requirement 12 (AC 1–5) |
| 14   | Requirement 13 (AC 1–5) |
| 15   | Integration validation across all requirements |

---

## Property-Based Test Definitions

| ID | Property | Requirement | Strategy |
|----|----------|-------------|----------|
| P1 | `top_line` always in `[1, max_top_line]` after any operation | Req 1 | Generate random sequences of scroll/resize operations; assert bounds |
| P2 | Scroll commands never produce out-of-bounds `top_line` | Req 2 | Generate random (visible_count, line_count, command) triples; assert clamping |
| P3 | After cursor move + viewport adjustment, cursor is always within visible range | Req 3 | Generate random cursor moves; assert `cursor_line ∈ [top_line, top_line + visible_count - 1]` |
| P4 | Scrollbar fraction round-trip: `to_fraction(to_top_line(f)) ≈ f` within ±1 line | Req 4 | Generate random fractions in [0.0, 1.0]; assert round-trip |
| P5 | Caret policy ensures cursor visible after adjustment | Req 5 | Generate random cursor positions and policies; assert visibility post-policy |
| P6 | Column affinity preserved through short-line sequences | Req 6 | Generate line-length sequences; move cursor vertically; assert affinity restoration |
| P7 | `horizontal_offset` always in `[0, max_horizontal_extent]` | Req 7 | Generate random horizontal scroll ops; assert bounds |
| P8 | Mouse wheel never exceeds document bounds | Req 8 | Generate random wheel ticks; assert top_line bounds |
| P9 | `pixel_offset` always in `[0, line_height)` in smooth mode | Req 9 | Generate random smooth scroll ops; assert pixel range |
| P10 | `ViewportChanged` event emitted after every mutation | Req 10 | Generate random command sequences; assert event count matches mutation count |
| P11 | Scrollbar total equals DisplayLineMapper total | Req 11 | Generate random fold/wrap configs; assert scrollbar uses mapper total |
| P12 | `restore(snapshot(s))` produces equivalent state | Req 12 | Generate random viewport states; assert snapshot round-trip |
| P13 | Scrollbar mapping is monotonically non-decreasing | Req 13 | Generate fractions 0.0 to 1.0 in steps; assert non-decreasing top_line |

---

## Notes

- The `ff-viewport-and-scrolling` crate depends on `ff-document-model` for line count queries and on `ff-display-line-mapping` (via trait) for wrapped/folded line calculations
- The viewport model is GUI-independent — it computes positions and targets; the GUI shell is responsible for rendering and animation interpolation
- Scroll commands are navigation-only and are NOT recorded on the undo stack
- The `DisplayLineMapper` trait may be defined in this crate or imported from `ff-display-line-mapping` depending on which crate is implemented first; a local trait definition with adapter pattern is acceptable
- Column affinity uses columns (not pixels) for the initial monospace implementation; pixel-based affinity can be added later for proportional font support
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The caret policy implementation follows Scintilla's `CaretPolicySlop` model with Rust-idiomatic naming

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1.1", "1.2", "1.3", "1.4", "1.5"], "dependsOn": [] },
    { "id": 1, "label": "Viewport State Model", "tasks": ["2.1", "2.2", "2.3", "2.4", "2.5", "2.6", "2.7"], "dependsOn": [0] },
    { "id": 2, "label": "Core Scroll Operations", "tasks": ["3.1", "3.2", "3.3", "3.4", "3.5", "3.6", "3.7", "3.8", "4.1", "4.2", "4.3", "4.4", "4.5", "4.6", "4.7", "4.8", "4.9", "5.1", "5.2", "5.3", "5.4", "5.5", "5.6", "5.7"], "dependsOn": [1] },
    { "id": 3, "label": "Policies and Affinity", "tasks": ["6.1", "6.2", "6.3", "6.4", "6.5", "6.6", "6.7", "6.8", "6.9", "6.10", "7.1", "7.2", "7.3", "7.4", "7.5", "7.6", "7.7"], "dependsOn": [2] },
    { "id": 4, "label": "Horizontal and Wheel Scrolling", "tasks": ["8.1", "8.2", "8.3", "8.4", "8.5", "8.6", "8.7", "8.8", "9.1", "9.2", "9.3", "9.4", "9.5", "9.6", "9.7"], "dependsOn": [2] },
    { "id": 5, "label": "Smooth Scrolling and Commands", "tasks": ["10.1", "10.2", "10.3", "10.4", "10.5", "10.6", "10.7", "10.8", "11.1", "11.2", "11.3", "11.4", "11.5", "11.6", "11.7"], "dependsOn": [2, 4] },
    { "id": 6, "label": "Display Mapping and Persistence", "tasks": ["12.1", "12.2", "12.3", "12.4", "12.5", "12.6", "12.7", "12.8", "13.1", "13.2", "13.3", "13.4", "13.5", "13.6"], "dependsOn": [1, 2] },
    { "id": 7, "label": "Large File Precision", "tasks": ["14.1", "14.2", "14.3", "14.4", "14.5", "14.6"], "dependsOn": [2] },
    { "id": 8, "label": "Integration and Documentation", "tasks": ["15.1", "15.2", "15.3", "15.4"], "dependsOn": [3, 4, 5, 6, 7] }
  ]
}
```
