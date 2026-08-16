# Requirements Document

## Introduction

This feature specifies the display-line-mapping subsystem for FileForgeWorkbench (`ff-display-line-mapping` crate). The display-line-mapping layer is a **core editor infrastructure component** that maintains the bidirectional relationship between document lines (the logical lines stored in the document buffer) and display lines (the visual lines rendered in the viewport). This relationship is complex because:

- **Hidden lines** (excluded via EXCLUDE/SHOW or code-folding) have zero display lines.
- **Wrapped lines** (when word wrap is active) have multiple display lines per document line.
- **Both** can be active simultaneously, making the mapping a many-to-many relationship when considered across the entire document.

The subsystem adapts concepts from Scintilla's `ContractionState` and `Partitioning` data structures into Rust, providing O(log n) lookup performance for both forward (doc→display) and reverse (display→doc) conversions. It supports incremental updates so that edits, fold toggles, and wrap changes invalidate only the affected mapping ranges rather than triggering a full rebuild.

The display-line-mapping crate is a Wave 4 (Core Editor) component. It is consumed by `viewport-and-scrolling` (for rendering and scroll calculations), `exclude-show-filter` (for line hiding commands), the code-folding UI (gutter indicators), and `idle-processing` (for background wrap recalculation). It depends on `document-model` for line count and line content, and integrates with the `command-framework` for fold/unfold commands.

**Source references:**
- **[SCI-CS-12.1]** = Scintilla `ContractionState` / `IContractionState` interface — visible/hidden line tracking, fold level storage, display-line mapping, one-to-one optimization, lazy allocation, 64-bit line indexing
- **[SCI-CS-12.3]** = Scintilla expansion/collapse algorithm and line numbering effects
- **[FFE-EXCL]** = FileForgeEditor exclude-show concepts (EXCLUDE/SHOW/RESET commands, ISPF-style line exclusion)
- **[WB]** = Workbench Architecture Brief (GUI-independent core, command-driven architecture, large file support)

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `document-model` | **Dependency** | Provides line count, line content, and document modification notifications (insert/delete events) that trigger mapping updates. |
| `viewport-and-scrolling` | **Consumer** | Uses display-line-mapping for scroll position translation, viewport bounds calculation, and scrollbar range determination. |
| `exclude-show-filter` | **Consumer** | Drives ISPF-style line visibility via `set_visible` calls; the display-line-mapping provides the underlying visibility storage. |
| `syntax-highlighting` | **Consumer** | Provides fold-level detection (indent-based or keyword-based) that identifies fold headers and body extents for code folding. |
| `idle-processing` | **Consumer** | Performs background wrap height recalculation and updates the mapping via `set_height` as wrap results become available. |
| `command-framework` | **Integration** | All fold/unfold/exclude/show operations are registered as commands for keyboard shortcuts, menus, and macros. |
| `line-wrap-toggle` | **Consumer** | Toggles word wrap mode on/off, triggering bulk `set_height` updates across all document lines. |

## Glossary

- **Document_Line**: A logical line in the text buffer, identified by a zero-based index. Document lines are the lines stored in the `document-model` crate. [SCI-CS-12.1, FFE-EXCL]
- **Display_Line**: A visual line as rendered in the viewport. Display lines are numbered contiguously from zero. A single Document_Line may map to zero display lines (if hidden) or multiple display lines (if wrapped). [SCI-CS-12.1]
- **Display_Line_Count**: The total number of display lines across the entire document, which equals the sum of the height (in display lines) of all visible document lines. [SCI-CS-12.1]
- **Line_Height**: The number of display lines occupied by a single Document_Line. A visible, unwrapped line has height 1. A visible, wrapped line has height ≥ 2. A hidden line has effective height 0. [SCI-CS-12.1]
- **Line_Visibility**: A boolean attribute per Document_Line indicating whether the line is visible (contributes to display output) or hidden (excluded from display output). [SCI-CS-12.1, FFE-EXCL]
- **Fold_Region**: A contiguous range of Document_Lines that can be collapsed (hidden) or expanded (shown) as a unit, identified by a header line that acts as the fold point. [SCI-CS-12.1, SCI-CS-12.3]
- **Fold_State**: Whether a Fold_Region is currently expanded (all lines in the region visible) or collapsed (all lines except the header hidden). [SCI-CS-12.1, SCI-CS-12.3]
- **Fold_Display_Text**: Optional summary text displayed on the fold header line when the fold region is collapsed (e.g., "{ ... }" or a line count indicator). [SCI-CS-12.1]
- **Wrap_Height**: The number of display lines needed to render a Document_Line when word wrap is active. Equals 1 when the line fits within the viewport width; equals ⌈line_width / viewport_width⌉ when it does not. [SCI-CS-12.1]
- **Contraction_State**: The aggregate mapping state for the entire document, tracking visibility, fold state, and per-line display heights. Named after the Scintilla concept. [SCI-CS-12.1]
- **Partitioning**: An internal data structure that maps document line indices to cumulative display line positions, enabling O(log n) lookup in both directions. [SCI-CS-12.1]
- **Incremental_Update**: A mapping update that modifies only the affected lines/ranges rather than recomputing the entire document-to-display mapping from scratch. [SCI-CS-12.1]
- **Sub_Line**: A specific display line within a wrapped Document_Line, identified by a zero-based offset from the first display line of that document line (0 = first sub-line, 1 = second sub-line, etc.). [SCI-CS-12.1]
- **One_To_One_Mode**: An optimized state where every document line maps directly to exactly one display line (no hidden lines, no wrapping). In this mode, the mapping requires no heap allocation and all lookups are O(1). [SCI-CS-12.1]
- **Lazy_Allocation**: The strategy of deferring allocation of per-line tracking data structures until the first non-trivial operation (first hide, first fold, or first wrap height > 1). The system starts in One_To_One_Mode and transitions to full tracking on demand. [SCI-CS-12.1]
- **Large_Document_Mode**: A mode that uses 64-bit line indexing (instead of 32-bit) to support documents exceeding 2^31 lines. Activated based on document size at creation or when line count exceeds 32-bit limits. [SCI-CS-12.1, WB]
- **ISPF_Exclusion**: A line-hiding mechanism driven by the EXCLUDE/SHOW/RESET commands (inherited from ISPF editing), distinct from code folding in that it operates on flat line ranges without hierarchical fold levels. [FFE-EXCL]

## Requirements

### Requirement 1: Document-to-Display Line Mapping

**User Story:** As a viewport renderer, I need to convert document line numbers to display line numbers and vice versa, so that I can correctly position content in the viewport and translate user interactions (clicks, scroll positions) back to document coordinates.

**Source:** [SCI-CS-12.1] `IContractionState::DisplayFromDoc`, `DocFromDisplay`, `Partitioning`.

#### Acceptance Criteria

1. THE Contraction_State SHALL provide a `display_from_doc(doc_line)` method that returns the first display line index corresponding to the given Document_Line, where the result is the cumulative sum of display heights of all preceding visible lines.
2. THE Contraction_State SHALL provide a `display_from_doc_sub(doc_line, sub_line)` method that returns the display line index for a specific Sub_Line within a wrapped Document_Line, clamping `sub_line` to `height - 1` if it exceeds the line's current display height.
3. THE Contraction_State SHALL provide a `display_last_from_doc(doc_line)` method that returns the last display line index occupied by the given Document_Line (equal to `display_from_doc(doc_line) + height - 1` for visible lines).
4. THE Contraction_State SHALL provide a `doc_from_display(display_line)` method that returns the Document_Line that contains the given display line, where the returned document line is always a visible line.
5. WHEN `doc_from_display` is called with a display line index less than 0, THE method SHALL return document line 0.
6. WHEN `doc_from_display` is called with a display line index greater than or equal to Display_Line_Count, THE method SHALL return the last visible document line.
7. THE Contraction_State SHALL provide a `lines_in_doc()` method returning the total number of Document_Lines currently tracked.
8. THE Contraction_State SHALL provide a `lines_displayed()` method returning the total Display_Line_Count (sum of heights of all visible lines).
9. WHEN no lines are hidden and no lines are wrapped (the trivial case), THE Contraction_State SHALL operate in an optimized one-to-one mode where `display_from_doc(n)` returns `n` and `doc_from_display(n)` returns `n` without allocating data structures for per-line tracking.
10. FOR ALL valid document lines `d` where `0 ≤ d < lines_in_doc()`, THE invariant `doc_from_display(display_from_doc(d)) == d` SHALL hold whenever the line is visible.

---

### Requirement 2: Line Exclusion and Hiding

**User Story:** As the exclude-show-filter subsystem, I need to hide ranges of document lines so they are not displayed in the viewport, reducing visual clutter and allowing users to focus on relevant sections of a file.

**Source:** [SCI-CS-12.1] `IContractionState::SetVisible`, `GetVisible`, `HiddenLines`; [FFE-EXCL] EXCLUDE/SHOW/RESET commands.

#### Acceptance Criteria

1. THE Contraction_State SHALL provide a `set_visible(start_line, end_line, is_visible)` method that sets the visibility of all Document_Lines in the range `[start_line, end_line]` (inclusive) to the given boolean value, returning `true` if any line's visibility actually changed.
2. THE Contraction_State SHALL provide a `get_visible(doc_line)` method that returns `true` if the Document_Line is visible, `false` if hidden.
3. WHEN a Document_Line is set to hidden, THE Contraction_State SHALL subtract that line's display height from the Display_Line_Count, causing subsequent display line indices to shift down accordingly.
4. WHEN a hidden Document_Line is set to visible, THE Contraction_State SHALL add that line's current display height (1 for unwrapped, or the wrap height for wrapped) to the Display_Line_Count, causing subsequent display line indices to shift up accordingly.
5. THE Contraction_State SHALL provide a `hidden_lines()` method that returns `true` if any Document_Line is currently hidden, `false` if all lines are visible.
6. THE Contraction_State SHALL provide a `show_all()` method that makes all lines visible and resets the state to the optimized one-to-one mode (deallocating per-line tracking structures), equivalent to Scintilla's `ShowAll`.
7. WHEN `set_visible` is called with `start_line > end_line` or with line indices outside the valid range `[0, lines_in_doc())`, THE method SHALL return `false` without modifying state.
8. THE Display_Line_Count SHALL always equal the sum of display heights of all visible lines; hidden lines SHALL contribute zero to the Display_Line_Count regardless of their wrap height.

---

### Requirement 3: Code Folding

**User Story:** As a user, I want to collapse and expand regions of code so that I can navigate large files efficiently by hiding implementation details behind fold points.

**Source:** [SCI-CS-12.1] `IContractionState::SetExpanded`, `GetExpanded`, `ContractedNext`, `SetFoldDisplayText`; [SCI-CS-12.3] expansion/collapse algorithm.

#### Acceptance Criteria

1. THE Contraction_State SHALL provide a `set_expanded(doc_line, is_expanded)` method that sets the fold state of the given Document_Line (acting as a fold header), returning `true` if the state changed.
2. THE Contraction_State SHALL provide a `get_expanded(doc_line)` method that returns `true` if the fold at the given Document_Line is expanded (or if the line is not a fold header), `false` if collapsed.
3. THE Contraction_State SHALL provide an `expand_all()` method that sets all fold headers to expanded state, returning `true` if any fold state changed.
4. THE Contraction_State SHALL provide a `contracted_next(start_line)` method that returns the document line index of the next collapsed fold header at or after `start_line`, or `None` (sentinel -1) if no contracted fold exists beyond that point.
5. WHEN a fold is collapsed (expanded → contracted), THE consuming code (exclude-show-filter or fold engine) SHALL call `set_visible(body_start, body_end, false)` on the fold body lines to hide them from the display; THE Contraction_State tracks fold expanded/collapsed state separately from line visibility to support nested folds.
6. WHEN a fold is expanded (contracted → expanded), THE consuming code SHALL call `set_visible(body_start, body_end, true)` on the fold body lines to restore them to the display, except for body lines that belong to a nested fold that is itself still collapsed.
7. THE Contraction_State SHALL provide a `set_fold_display_text(doc_line, text)` method that sets optional summary text to display on a collapsed fold header line, returning `true` if the text changed.
8. THE Contraction_State SHALL provide a `get_fold_display_text(doc_line)` method that returns the fold display text for a given line, or `None` if no text is set.
9. NESTED FOLDS SHALL be supported: a fold region may contain other fold regions. Collapsing an outer fold hides all inner fold headers and bodies; expanding the outer fold restores inner fold headers and their bodies according to each inner fold's own expanded/collapsed state.
10. THE gutter rendering layer SHALL display a fold indicator (e.g., ▶ for collapsed, ▼ for expanded) adjacent to each fold header line, allowing users to toggle fold state by clicking the indicator.

---

### Requirement 4: Word Wrap Mapping

**User Story:** As the viewport renderer, I need to know how many display lines each document line occupies when word wrap is active, so that I can correctly lay out wrapped text and position the caret on the correct sub-line.

**Source:** [SCI-CS-12.1] `IContractionState::SetHeight`, `GetHeight`, `DisplayFromDocSub`.

#### Acceptance Criteria

1. THE Contraction_State SHALL provide a `set_height(doc_line, height)` method that sets the number of display lines (Sub_Lines) for the given Document_Line, returning `true` if the height changed.
2. THE Contraction_State SHALL provide a `get_height(doc_line)` method that returns the current display height of the given Document_Line (minimum 1 for visible lines in one-to-one mode).
3. WHEN word wrap is disabled, ALL Document_Lines SHALL have a height of 1 (one document line = one display line).
4. WHEN word wrap is enabled and a Document_Line's content exceeds the viewport width, THE wrap calculation engine SHALL call `set_height(doc_line, n)` where `n` is the number of visual sub-lines needed to display the full line content within the viewport width.
5. WHEN `set_height` is called on a visible Document_Line, THE Contraction_State SHALL adjust the Display_Line_Count by the difference between the new height and the old height (`new_height - old_height`).
6. WHEN `set_height` is called on a hidden Document_Line, THE Contraction_State SHALL store the new height but SHALL NOT adjust the Display_Line_Count (hidden lines contribute zero display lines regardless of their wrap height).
7. WHEN `set_height` is called with a `doc_line` outside the valid range `[0, lines_in_doc())`, THE method SHALL return `false` without modifying state.
8. FOR ALL visible Document_Lines with height `h > 1`, THE mapping `display_from_doc_sub(doc_line, sub)` for `sub` in `[0, h-1]` SHALL return contiguous display line indices.

---

### Requirement 5: Efficient Lookup Performance

**User Story:** As a viewport renderer processing scroll events and mouse interactions, I need document-to-display and display-to-document conversions to complete in sub-linear time, so that the editor remains responsive even for documents with millions of lines.

**Source:** [SCI-CS-12.1] `Partitioning` data structure (prefix-sum tree / partition table).

#### Acceptance Criteria

1. THE `display_from_doc(doc_line)` method SHALL execute in O(log n) time or better, where n is the number of Document_Lines in the mapping.
2. THE `doc_from_display(display_line)` method SHALL execute in O(log n) time or better, where n is the number of Document_Lines in the mapping.
3. THE Contraction_State SHALL use an internal partitioning data structure (prefix-sum array, Fenwick tree, segment tree, or equivalent) that supports O(log n) point queries and O(log n) range updates to maintain the cumulative display-line positions.
4. WHEN in one-to-one mode (no hidden lines, no wrapping), THE `display_from_doc` and `doc_from_display` methods SHALL execute in O(1) time with no data structure traversal, returning the input value directly.
5. THE `set_visible` and `set_height` methods SHALL update the internal partitioning data structure in O(log n) time per affected line, without requiring a full scan of all document lines.
6. FOR documents with 1,000,000 or more lines, THE `display_from_doc` and `doc_from_display` methods SHALL complete within 1 microsecond on modern hardware (target: no more than ~20 tree traversal steps for a 1M-line document).

---

### Requirement 6: Incremental Updates

**User Story:** As a document editor processing text insertions and deletions, I need the display-line mapping to update incrementally when lines are added or removed, so that each keystroke does not trigger a full O(n) rebuild of the mapping state.

**Source:** [SCI-CS-12.1] `IContractionState::InsertLines`, `DeleteLines`, incremental height updates.

#### Acceptance Criteria

1. THE Contraction_State SHALL provide an `insert_lines(doc_line, count)` method that inserts `count` new Document_Lines at the specified position, each initialized as visible with height 1 (unwrapped), updating the internal partitioning data structure incrementally.
2. THE Contraction_State SHALL provide a `delete_lines(doc_line, count)` method that removes `count` Document_Lines starting at the specified position, adjusting the Display_Line_Count by subtracting the heights of any visible deleted lines, and updating the internal data structure incrementally.
3. WHEN lines are inserted, THE Contraction_State SHALL update the partitioning data structure in O(count × log n) time, where n is the total number of document lines after insertion.
4. WHEN lines are deleted, THE Contraction_State SHALL update the partitioning data structure in O(count × log n) time, where n is the total number of document lines before deletion.
5. WHEN a Document_Line's wrap height changes (due to content change or viewport resize), THE consuming code SHALL call `set_height(doc_line, new_height)` which SHALL update only that line's entry in O(log n) time without affecting other entries.
6. WHEN a fold is toggled (collapsed or expanded), THE visibility changes SHALL be applied to the affected range via `set_visible`, which SHALL update the Display_Line_Count incrementally in O(range_size × log n) time — not by scanning the entire document.
7. AFTER any incremental update (insert, delete, set_visible, set_height), THE invariant `lines_displayed() == sum of get_height(d) for all visible d` SHALL hold.

---

### Requirement 7: Integration Points

**User Story:** As a workbench developer, I want the display-line-mapping to integrate cleanly with the viewport, scrollbar, find system, and gutter rendering, so that all components present a consistent view of the document.

**Source:** [SCI-CS-12.1] viewport/scrollbar integration; [SCI-CS-12.3] line numbering effects; [FFE-EXCL] exclude-show display integration; [WB] command-driven architecture.

#### Acceptance Criteria

1. THE viewport rendering subsystem SHALL use `display_from_doc` and `doc_from_display` to determine which document lines are visible within the current scroll position, rendering only those lines that map to display lines within the viewport bounds.
2. THE scrollbar SHALL reflect the total Display_Line_Count (from `lines_displayed()`) as its range, so that the scrollbar thumb size and position accurately represent the visible content extent excluding hidden lines.
3. WHEN the user scrolls, THE viewport SHALL translate the scroll position (a display line offset) to a document line via `doc_from_display`, enabling correct rendering of the target position.
4. WHEN the Find/Replace subsystem navigates to a match on a hidden line, THE consuming code SHALL make that line visible (by expanding folds or clearing exclusion) before scrolling to it, ensuring the user can always see search results.
5. WHEN the Find/Replace subsystem reports match positions to the user (e.g., "match at line 42"), IT SHALL report the Document_Line number (1-based), but the viewport SHALL scroll to the corresponding display line position.
6. THE line-number gutter SHALL display Document_Line numbers (not display line numbers) adjacent to each rendered line, so that users always see document-relative line numbers even when lines are hidden or wrapped.
7. WHEN a Document_Line is wrapped into multiple Sub_Lines, THE gutter SHALL display the line number only on the first Sub_Line; subsequent Sub_Lines SHALL display a continuation marker or blank in the line-number column.
8. ALL fold/unfold operations SHALL be registered as commands in the command-framework, enabling keyboard shortcuts, menu items, and macros to trigger fold state changes through the standard command dispatch path.
9. THE Contraction_State SHALL emit change notifications (via a callback, event, or observer pattern) when the Display_Line_Count changes, enabling the scrollbar and viewport to synchronize without polling.
10. THE display-line-mapping crate SHALL expose a public trait (`DisplayLineMapping`) defining the full lookup and mutation API, allowing the viewport, scrollbar, gutter, and find subsystems to depend on the trait rather than a concrete implementation.

---

### Requirement 8: Large Document Support

**User Story:** As a developer working with very large files (millions of lines), I need the display-line-mapping to support 64-bit line indexing so that documents exceeding 2 billion lines can be mapped without overflow or truncation.

**Source:** [SCI-CS-12.1] `ContractionStateCreate(bool largeDocument)` — 32-bit vs 64-bit line indexing template parameter; [WB] large file support.

#### Acceptance Criteria

1. THE display-line-mapping crate SHALL support two internal line index sizes: a 32-bit mode (default, for documents with fewer than 2^31 lines) and a 64-bit mode (for documents exceeding 2^31 lines), selectable at creation time.
2. WHEN a Contraction_State is created with large-document mode enabled, ALL internal line indices and partition positions SHALL use 64-bit integers (`u64` or `i64`), preventing overflow for documents exceeding 2^31 lines.
3. WHEN a Contraction_State is created with standard mode (large-document mode disabled), ALL internal line indices SHALL use 32-bit integers (`u32` or `i32`) to minimize memory consumption for typical documents.
4. THE public API SHALL use a platform-width line index type (e.g., `usize`) regardless of internal storage mode, abstracting the internal representation from consumers.
5. THE selection of 32-bit vs 64-bit mode SHALL be determined at construction time based on the document's line count or a caller-provided hint, and SHALL NOT require rebuilding the mapping to switch modes after construction.
6. IN 32-bit mode, THE memory consumption of the per-line tracking data structures SHALL be approximately 50% of the equivalent 64-bit mode structures, providing a measurable memory saving for typical documents (< 10M lines).

---

### Requirement 9: Lazy Allocation and One-to-One Optimization

**User Story:** As the editor opening many files simultaneously (e.g., in a multi-tab session), I need the display-line-mapping to consume near-zero memory for files that have no hidden lines and no word wrapping, so that the per-document overhead does not accumulate across many open buffers.

**Source:** [SCI-CS-12.1] One-to-one optimization mode, `EnsureData()` lazy allocation pattern.

#### Acceptance Criteria

1. WHEN a Contraction_State is first created, IT SHALL start in One_To_One_Mode with no heap-allocated per-line data structures; only the document line count SHALL be stored.
2. WHEN the first non-trivial operation occurs (any of: `set_visible(_, _, false)`, `set_expanded(_, false)`, `set_height(_, h)` where `h != 1`), THE Contraction_State SHALL lazily allocate the full per-line tracking data structures (visibility, expanded state, heights, fold display texts, and the partitioning structure) initialized to the default visible/expanded/height-1 state for all existing lines.
3. THE `show_all()` method SHALL deallocate all per-line tracking data structures and return the Contraction_State to One_To_One_Mode, recovering the memory used by visibility/fold/height tracking.
4. IN One_To_One_Mode, THE memory footprint of the Contraction_State SHALL be O(1) — independent of the number of document lines.
5. IN One_To_One_Mode, THE methods `display_from_doc`, `doc_from_display`, `get_visible`, `get_expanded`, and `get_height` SHALL return their trivial values (identity mapping, `true`, `true`, `1` respectively) without any branching on per-line data.
6. WHEN `insert_lines` or `delete_lines` is called in One_To_One_Mode, THE Contraction_State SHALL simply update the line count without allocating per-line structures.
7. THE transition from One_To_One_Mode to full tracking mode SHALL complete in O(n) time where n is the current line count (one-time cost to initialize per-line arrays).

---

### Requirement 10: Dual Hiding Mechanism Support

**User Story:** As a workbench user, I want both ISPF-style EXCLUDE/SHOW line hiding and hierarchical code folding to coexist in the same document, so that I can use whichever mechanism suits my current task without one interfering with the other.

**Source:** [FFE-EXCL] ISPF exclusion model; [SCI-CS-12.1] fold expanded state; [SCI-CS-12.3] expansion algorithm; [WB] architectural requirement for both mechanisms.

#### Acceptance Criteria

1. THE Contraction_State SHALL maintain line visibility and fold expanded/collapsed state as independent attributes: a line may be hidden due to ISPF exclusion, due to being inside a collapsed fold, or both simultaneously.
2. THE `set_visible` method SHALL be usable by BOTH the exclude-show-filter (for ISPF exclusion) AND the fold engine (for collapsing fold bodies); the mapping layer SHALL NOT distinguish the reason a line is hidden — it only tracks the boolean visibility.
3. THE `set_expanded` / `get_expanded` state SHALL be orthogonal to visibility: a fold header may be marked as collapsed (`expanded = false`) even if the fold body lines are currently visible (e.g., the user manually showed them via SHOW command). The consuming fold engine uses both attributes together to determine correct behavior.
4. WHEN the EXCLUDE command hides lines that are inside a collapsed fold (already hidden), THE visibility SHALL remain hidden and `set_visible` SHALL return `false` (no change).
5. WHEN the SHOW command makes lines visible that are inside a collapsed fold, THE fold engine SHALL re-evaluate whether those lines should remain hidden based on the fold's expanded state, potentially overriding the SHOW for lines within a contracted fold.
6. THE `show_all()` method SHALL reset BOTH exclusion-based hiding AND fold-based hiding, making all lines visible and marking all folds as expanded, providing a clean "reset everything" operation.
7. THE display-line-mapping layer SHALL NOT store fold levels, fold nesting depth, or fold region extents — those are the responsibility of the syntax-highlighting / language-service layer. The mapping layer only stores per-line visibility and per-line expanded/collapsed flags.
8. ISPF EXCLUDE/SHOW operations SHALL be flat (not hierarchical): excluding a range simply hides those lines, with no concept of nested exclusion levels. This contrasts with code folding, which IS hierarchical.

