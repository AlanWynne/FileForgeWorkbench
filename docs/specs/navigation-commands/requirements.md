# Requirements Document

## Introduction

This spec defines the **Navigation Commands** subsystem for FileForgeWorkbench (`ff-navigation-commands` crate). It covers all primary commands and keyboard operations that move the viewport or caret without modifying document content, plus the SORT command which reorders lines, and the display/session-state commands COLS and BOUNDS.

The navigation-commands crate is responsible for:
- **LOCATE command** — jump to a line number or named label
- **SORT command** — reorder lines by column key (undoable document modification)
- **COLS command** — display/toggle a column ruler overlay
- **BOUNDS/BNDS command** — set/clear active column boundaries for column-sensitive operations
- **Viewport navigation commands** — UP, DOWN, LEFT, RIGHT, TOP, BOTTOM (page/line scroll)
- **Paragraph navigation** — move caret to previous/next paragraph boundary
- **Word navigation** — move caret by word boundaries, word-part (camelCase) boundaries

All commands are registered with the command framework and dispatched through the standard command execution pipeline. SORT is the only undoable command in this crate; LOCATE, viewport navigation, COLS, BOUNDS, paragraph nav, and word nav are non-undoable (viewport/session state changes).

### Design Principles

1. **GUI-independent** — all navigation logic operates on the viewport model and document model without GUI framework dependency. [WB]
2. **Command-framework registered** — every command is registered with metadata, help text, and undo classification. [WB]
3. **Viewport model delegation** — scroll operations delegate to `viewport-and-scrolling` for actual viewport state mutation and clamping. [WB]
4. **Bounds integration** — SORT respects active column bounds; BOUNDS state is shared across command specs. [FFE-CMD-20]
5. **Character classification** — word and word-part navigation uses the document model's configurable character class tables. [SCI-DOC-16]

### Source References

- **[FFE-CMD-10]** = FileForgeEditor `core-command-semantics` Requirement 10 (SORT command)
- **[FFE-CMD-11]** = FileForgeEditor `core-command-semantics` Requirement 11 (SAVE, CANCEL, END — referenced for session commands; not owned here)
- **[FFE-CMD-12]** = FileForgeEditor `core-command-semantics` Requirement 12 (LOAD, RELOAD — referenced; not owned here)
- **[FFE-CMD-13]** = FileForgeEditor `core-command-semantics` Requirement 13 (DELETE — referenced; owned by `edit-operations`)
- **[FFE-CMD-14]** = FileForgeEditor `core-command-semantics` Requirement 14 (COPY in-document — referenced; owned by `edit-operations`)
- **[FFE-CMD-15]** = FileForgeEditor `core-command-semantics` Requirement 15 (MOVE — referenced; owned by `edit-operations`)
- **[FFE-CMD-16]** = FileForgeEditor `core-command-semantics` Requirement 16 (LOCATE command)
- **[FFE-CMD-17]** = FileForgeEditor `core-command-semantics` Requirement 17 (Navigation: UP, DOWN, LEFT, RIGHT, TOP, BOTTOM)
- **[FFE-CMD-18]** = FileForgeEditor `core-command-semantics` Requirement 18 (MACRO/EXEC/RUN — referenced; owned by `lua-macro-engine`)
- **[FFE-CMD-19]** = FileForgeEditor `core-command-semantics` Requirement 19 (COLS command)
- **[FFE-CMD-20]** = FileForgeEditor `core-command-semantics` Requirement 20 (BOUNDS/BNDS command)
- **[FFE-CMD-21]** = FileForgeEditor `core-command-semantics` Requirement 21 (UNDO/REDO delegation — referenced; owned by `undo-redo-transactions`)
- **[SCI-EDIT-2.2]** = Scintilla Editor Requirement 2.2 criteria 8–12 (DocumentStart/End, PageUp/PageDown, ParaUp/ParaDown, word movement, CursorUpOrDown with lastXChosen)
- **[SCI-DOC-16]** = Scintilla Document Requirement 16 (Word Navigation — ExtendWordSelect, NextWordStart, NextWordEnd, WordPartLeft/Right, character class boundaries, camelCase detection)
- **[WB]** = Workbench Platform Architecture Brief (GUI-independent, command-framework integration, crate separation)

### Cross-References

- **`command-semantics`** — Defines the command execution pipeline, scope resolution, and error handling that all commands in this crate pass through.
- **`viewport-and-scrolling`** — Owns viewport state (top_line, visible_count, horizontal_offset, cursor_line, cursor_column, column_affinity). Navigation commands delegate scroll operations to this crate.
- **`document-model`** — Provides line count, line content, character classification tables, and paragraph detection.
- **`undo-redo-transactions`** — SORT produces an undoable transaction; this crate records it via the transaction API.
- **`edit-operations`** — DELETE, COPY, MOVE commands are specified there, not here (FFE-CMD-13/14/15 are not owned by this spec).
- **`configuration-system`** — Provides configurable values for default scroll amounts, bounds_affect_find, and word-character classification.

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **LOCATE** | A primary command that scrolls the viewport to a specific line number or named label without modifying document content. | [FFE-CMD-16] |
| **SORT** | A primary command that reorders lines within a resolved scope by a column-key comparison. The only undoable command in this crate. | [FFE-CMD-10] |
| **COLS_Line** | A synthetic, non-editable display line showing a column ruler. Not a real document line; never saved to disk. | [FFE-CMD-19] |
| **BNDS_Line** | A synthetic, non-editable display line showing the active column boundary positions. Not a real document line; never saved to disk. | [FFE-CMD-20] |
| **Bounds** | The active left and right column numbers constraining column-sensitive operations (CHANGE, FIND, SORT, shift). Stored in Session_State. | [FFE-CMD-20] |
| **Session_State** | Transient in-memory editor state (bounds, COLS markers, BNDS marker, excluded lines, tags) that is not persisted to disk. | [FFE-CMD-19], [FFE-CMD-20] |
| **Viewport** | The logical window into the document defined by top_line, visible_count, and horizontal_offset. Owned by `viewport-and-scrolling`. | [FFE-CMD-17] |
| **Column_Affinity** | The remembered preferred horizontal pixel/column position maintained during vertical caret movement (Scintilla's `lastXChosen`). | [SCI-EDIT-2.2] |
| **Paragraph_Boundary** | A blank line (or a line consisting solely of whitespace) that separates paragraphs. Used by paragraph navigation. | [SCI-EDIT-2.2] |
| **Word_Boundary** | A transition between character classes (space, word, punctuation) used by word navigation. | [SCI-DOC-16] |
| **Word_Part_Boundary** | A transition within a word at camelCase boundaries or punctuation separators (e.g., `getValue` → `get` + `Value`). | [SCI-DOC-16] |
| **Character_Class** | A classification of characters into categories: space, newLine, word, punctuation. Configurable per document. | [SCI-DOC-16] |
| **Stable_Sort** | A sort algorithm where elements with equal keys retain their original relative order. | [FFE-CMD-10] |
| **Scope** | The set of lines to which a command applies, resolved by the command execution pipeline. | [FFE-CMD-10] |

---

## Requirements

### Requirement 1: LOCATE Command

**User Story:** As a developer editing a large file, I want to jump directly to a specific line number or named label so that I can navigate instantly without manual scrolling.

**Source:** [FFE-CMD-16]

#### Acceptance Criteria

1.1. WHEN `LOCATE n` is issued with a positive integer `n`, THE system SHALL scroll the Viewport so that document line `n` is the topmost visible line. [FFE-CMD-16]

1.2. IF the line number supplied to `LOCATE` is less than 1 or greater than the document line count, THEN THE system SHALL display "Line number out of range" in the status area and SHALL leave the Viewport position unchanged. [FFE-CMD-16]

1.3. WHEN `LOCATE label` is issued with a non-numeric argument, THE system SHALL interpret the argument as a named label and scroll the Viewport to the line associated with that label. [FFE-CMD-16]

1.4. IF the label supplied to `LOCATE` does not exist in the current document, THEN THE system SHALL display "Label not found: <label>" in the status area and SHALL leave the Viewport position unchanged. [FFE-CMD-16]

1.5. THE LOCATE command SHALL be registered with the command framework as a non-undoable command (viewport state only). [WB]

1.6. WHEN LOCATE successfully navigates, THE system SHALL update `cursor_line` to the target line and reset `cursor_column` to 1. [FFE-CMD-16], [WB]

---

### Requirement 2: SORT Command

**User Story:** As a developer working with data files, I want to sort a range of lines by specified column positions so that I can reorder records without leaving the editor.

**Source:** [FFE-CMD-10]

#### Acceptance Criteria

2.1. WHEN `SORT` is issued with no arguments, THE system SHALL sort all visible lines using the full line content as the sort key, in ascending (A) order. [FFE-CMD-10]

2.2. WHEN `SORT col1 col2` is issued with two positive integers, THE system SHALL sort the resolved Scope using characters in columns `col1` through `col2` inclusive as the sort key. [FFE-CMD-10]

2.3. WHEN `SORT col1 col2 A` is issued, THE system SHALL sort in ascending order. [FFE-CMD-10]

2.4. WHEN `SORT col1 col2 D` is issued, THE system SHALL sort in descending order. [FFE-CMD-10]

2.5. WHEN `SORT TAGGED` is issued, THE system SHALL sort only lines whose `tagged` flag is true, leaving non-tagged lines in their original positions relative to non-tagged lines. [FFE-CMD-10]

2.6. WHEN `SORT VISIBLE` is issued, THE system SHALL sort only currently visible (non-excluded) lines. [FFE-CMD-10]

2.7. WHEN a `CC...CC` block is pending and `SORT` is issued, THE system SHALL restrict the sort scope to the lines within the CC block. [FFE-CMD-10]

2.8. THE SORT operation SHALL be stable: lines with equal sort keys SHALL retain their original relative order. [FFE-CMD-10]

2.9. WHEN active Bounds are set and no explicit column range is given in the SORT command, THE system SHALL use the active Bounds as the default sort key column range. [FFE-CMD-10], [FFE-CMD-20]

2.10. WHEN active Bounds are set and an explicit column range `col1 col2` is given, THE system SHALL use the intersection of [col1, col2] and the active Bounds as the effective sort key column range. [FFE-CMD-10], [FFE-CMD-20]

2.11. WHEN a SORT operation completes successfully, THE system SHALL record it as a single undoable Transaction via the `undo-redo-transactions` API. [FFE-CMD-10]

2.12. WHEN `SORT` is issued with no scope qualifier and no CC block is pending, THE system SHALL default to sorting all visible lines in the document. [FFE-CMD-10]

2.13. WHEN `SORT` is issued and the resolved scope contains zero or one lines, THE system SHALL display "Nothing to sort" and SHALL NOT record a transaction. [WB]

---

### Requirement 3: Navigation Commands (UP, DOWN, LEFT, RIGHT, TOP, BOTTOM)

**User Story:** As a developer navigating a document, I want keyboard-accessible commands for scrolling the viewport so that I can traverse any file from the command line without relying solely on scroll bars or arrow keys.

**Source:** [FFE-CMD-17], [SCI-EDIT-2.2] criteria 8–10

#### Acceptance Criteria

3.1. WHEN `UP` is issued with no argument, THE system SHALL scroll the Viewport up by one screen height (one page of `visible_count` lines). [FFE-CMD-17]

3.2. WHEN `UP n` is issued with a positive integer, THE system SHALL scroll the Viewport up by `n` lines. [FFE-CMD-17]

3.3. WHEN `DOWN` is issued with no argument, THE system SHALL scroll the Viewport down by one screen height (one page of `visible_count` lines). [FFE-CMD-17]

3.4. WHEN `DOWN n` is issued with a positive integer, THE system SHALL scroll the Viewport down by `n` lines. [FFE-CMD-17]

3.5. WHEN `LEFT` is issued with no argument, THE system SHALL scroll the Viewport left by the configured default horizontal scroll amount. [FFE-CMD-17]

3.6. WHEN `LEFT n` is issued with a positive integer, THE system SHALL scroll the Viewport left by `n` columns. [FFE-CMD-17]

3.7. WHEN `RIGHT` is issued with no argument, THE system SHALL scroll the Viewport right by the configured default horizontal scroll amount. [FFE-CMD-17]

3.8. WHEN `RIGHT n` is issued with a positive integer, THE system SHALL scroll the Viewport right by `n` columns. [FFE-CMD-17]

3.9. WHEN `TOP` is issued, THE system SHALL scroll the Viewport to the first line of the document (top_line = 1). [FFE-CMD-17]

3.10. WHEN `BOTTOM` is issued, THE system SHALL scroll the Viewport so that the last line of the document is visible (top_line = max_top_line). [FFE-CMD-17]

3.11. WHEN any navigation command would scroll past the beginning of the document, THE system SHALL clamp `top_line` at 1 and SHALL NOT produce an error. [FFE-CMD-17]

3.12. WHEN any navigation command would scroll past the end of the document, THE system SHALL clamp `top_line` at `max_top_line` and SHALL NOT produce an error. [FFE-CMD-17]

3.13. WHEN a horizontal scroll command would result in a negative `horizontal_offset`, THE system SHALL clamp `horizontal_offset` at 0. [FFE-CMD-17]

3.14. ALL navigation commands (UP, DOWN, LEFT, RIGHT, TOP, BOTTOM) SHALL be registered with the command framework as non-undoable commands (viewport state only). [WB]

3.15. THE default horizontal scroll amount SHALL be configurable via `editor.navigation.horizontal_scroll_columns` in the configuration system. The default SHALL be 8 columns. [WB]

3.16. WHEN `TOP` or `BOTTOM` is issued, THE system SHALL also update `cursor_line` to the first or last line of the document respectively, and reset `cursor_column` to 1. [FFE-CMD-17], [SCI-EDIT-2.2]

---

### Requirement 4: COLS Command

**User Story:** As a developer working with fixed-width data, I want to display a non-editable column ruler line in the viewport so that I can visually identify column positions without counting characters.

**Source:** [FFE-CMD-19]

#### Acceptance Criteria

4.1. WHEN `COLS` is issued as a primary command, THE system SHALL insert a COLS_Line into the Viewport at the current cursor position (or at the top of the visible area if no cursor line is defined). [FFE-CMD-19]

4.2. THE COLS_Line SHALL be formatted as: `----+----1----+----2----+----3----+----4----+----5----+----6----+----7----+----8` with column position indicators at each tenth column, preceded by a `COLS` prefix indicator. [FFE-CMD-19]

4.3. THE COLS_Line SHALL be a display artifact only: it SHALL NOT be a real document line, SHALL NOT be saved to disk, and SHALL NOT appear in any document operation's Scope. [FFE-CMD-19]

4.4. WHEN `COLS` is issued a second time while a COLS_Line is already displayed at the same position, THE system SHALL remove that COLS_Line from the Viewport (toggle behaviour). [FFE-CMD-19]

4.5. WHEN `RESET` or `RESET ALL` is issued, THE system SHALL remove all COLS_Lines from the Viewport. [FFE-CMD-19]

4.6. THE COLS_Line SHALL scroll with the document such that it remains visually anchored to the document lines it was inserted between. [FFE-CMD-19]

4.7. WHEN the `COLS` line command is entered in the prefix area next to a document line, THE system SHALL insert a COLS_Line immediately above that document line. [FFE-CMD-19]

4.8. WHEN multiple `COLS` primary commands are issued at different cursor positions, THE system SHALL display a separate COLS_Line at each requested position. [FFE-CMD-19]

4.9. THE prefix area cell adjacent to a COLS_Line SHALL be non-editable and SHALL display the fixed indicator `COLS`. [FFE-CMD-19]

4.10. WHEN `RESET COMMANDS` is issued, THE system SHALL clear all COLS_Lines along with other pending command markers. [FFE-CMD-19]

4.11. THE COLS command SHALL be registered with the command framework as a non-undoable command (display artifact only). [WB]

---

### Requirement 5: BOUNDS / BNDS Command

**User Story:** As a developer working with fixed-width records or columnar data, I want to set active column boundaries so that column-sensitive operations (CHANGE, SORT, FIND, shift) act only within a defined column range.

**Source:** [FFE-CMD-20]

#### Acceptance Criteria

5.1. WHEN `BOUNDS left right` is issued with two positive integers (e.g., `BOUNDS 1 72`), THE system SHALL store `left` and `right` as the active Bounds in Session_State. [FFE-CMD-20]

5.2. WHEN `BNDS left right` is issued, THE system SHALL treat it identically to `BOUNDS left right`. [FFE-CMD-20]

5.3. WHEN Bounds are set, THE system SHALL insert a BNDS_Line into the Viewport showing the boundary positions, formatted with `<` and `>` characters placed at the configured left and right column positions respectively. [FFE-CMD-20]

5.4. WHEN `BOUNDS` is issued with no arguments, THE system SHALL clear the active Bounds from Session_State and remove the BNDS_Line from the Viewport. [FFE-CMD-20]

5.5. WHEN `BNDS` is issued with no arguments, THE system SHALL treat it identically to `BOUNDS` with no arguments (clear active Bounds). [FFE-CMD-20]

5.6. THE BNDS_Line SHALL be a display artifact only: it SHALL NOT be a real document line, SHALL NOT be saved to disk, and SHALL NOT appear in any document operation's Scope. [FFE-CMD-20]

5.7. WHEN active Bounds are set, THE system SHALL restrict CHANGE operations to the active column range on every affected line. [FFE-CMD-20]

5.8. WHEN active Bounds are set and `bounds_affect_find` configuration is true, THE system SHALL restrict FIND operations to the active column range. [FFE-CMD-20]

5.9. WHEN active Bounds are set, THE system SHALL restrict the `)` and `((` bounds-aware shift commands to operate within the Bounds, preserving characters outside the Bounds. [FFE-CMD-20]

5.10. WHEN active Bounds are set, THE system SHALL use the active Bounds as the default column range for SORT when no explicit column range is given (see Requirement 2.9). [FFE-CMD-20]

5.11. WHEN Bounds are cleared, THE system SHALL remove the BNDS_Line from the Viewport and restore all column-sensitive commands to their full-line default behaviour. [FFE-CMD-20]

5.12. THE system SHALL NOT record Bounds changes as undoable Transactions; Bounds are Session_State only. [FFE-CMD-20]

5.13. IF `left` is less than 1, or `right` is less than `left`, or either value is not a positive integer, THEN THE system SHALL display "Invalid bounds: left must be >= 1 and right must be > left" in the status area and SHALL NOT update the active Bounds. [FFE-CMD-20]

5.14. THE BOUNDS/BNDS command SHALL be registered with the command framework as a non-undoable command (session state only). [WB]

5.15. THE system SHALL expose the current active Bounds state via a public query API so that other command executors (CHANGE, FIND, SORT, shift) can read and apply the bounds constraint. [WB]

---

### Requirement 6: Paragraph Navigation

**User Story:** As a developer editing prose or structured text, I want to move the caret forward or backward by paragraph boundaries so that I can quickly skip between logical sections of content.

**Source:** [SCI-EDIT-2.2] criterion 11

#### Acceptance Criteria

6.1. WHEN paragraph-up navigation is triggered (e.g., via key binding or `PARA_UP` command), THE system SHALL move the caret to the beginning of the previous paragraph boundary (the first line after the previous blank line). [SCI-EDIT-2.2]

6.2. WHEN paragraph-down navigation is triggered (e.g., via key binding or `PARA_DOWN` command), THE system SHALL move the caret to the beginning of the next paragraph boundary (the first line after the next blank line). [SCI-EDIT-2.2]

6.3. A paragraph boundary SHALL be defined as a line that is empty or contains only whitespace characters. [SCI-EDIT-2.2]

6.4. WHEN paragraph-up is triggered and the caret is already at the first paragraph boundary (beginning of document or first line after first blank line group), THE system SHALL move the caret to document position 0 (first character of first line). [SCI-EDIT-2.2]

6.5. WHEN paragraph-down is triggered and no more paragraph boundaries exist below the caret, THE system SHALL move the caret to the last line of the document. [SCI-EDIT-2.2]

6.6. WHEN paragraph navigation moves the caret, THE system SHALL scroll the Viewport if necessary to keep the caret visible, following the configured caret visibility policy. [SCI-EDIT-2.2]

6.7. THE paragraph navigation commands SHALL support selection extension: when issued with the Extend modifier, THE system SHALL extend the selection from the anchor to the new caret position rather than collapsing it. [SCI-EDIT-2.2]

6.8. THE paragraph navigation commands SHALL be registered with the command framework as non-undoable commands (caret/viewport state only). [WB]

6.9. WHEN paragraph navigation is triggered, THE system SHALL skip over excluded (hidden) lines when traversing paragraph boundaries, treating contiguous excluded lines as not present for boundary detection purposes. [WB]

---

### Requirement 7: Word Navigation

**User Story:** As a developer editing source code, I want to move the caret forward or backward by word boundaries so that I can navigate efficiently within lines without character-by-character movement.

**Source:** [SCI-DOC-16], [SCI-EDIT-2.2] criterion 7

#### Acceptance Criteria

7.1. THE system SHALL classify characters into CharacterClass categories: space, newLine, word, and punctuation. ASCII characters use the configurable CharClassify table; Unicode code points >= 0x80 use Unicode category tables. [SCI-DOC-16]

7.2. WHEN word-left navigation is triggered, THE system SHALL move the caret to the start of the previous word by skipping whitespace backwards then skipping characters of the same class backwards until a class transition is reached. [SCI-DOC-16]

7.3. WHEN word-right navigation is triggered, THE system SHALL move the caret to the start of the next word by skipping characters of the current class forwards then skipping whitespace forwards until a non-whitespace character is reached. [SCI-DOC-16]

7.4. WHEN word-end-right navigation is triggered, THE system SHALL move the caret to the end of the current or next word by skipping whitespace forwards then skipping word characters forwards until a class transition is reached. [SCI-DOC-16]

7.5. WHEN word navigation reaches the beginning or end of a line, THE system SHALL cross the line boundary and continue navigation on the adjacent line. [SCI-DOC-16]

7.6. WHEN word navigation reaches the beginning of the document (position 0), THE system SHALL clamp the caret at position 0 without error. [SCI-DOC-16]

7.7. WHEN word navigation reaches the end of the document, THE system SHALL clamp the caret at the document end position without error. [SCI-DOC-16]

7.8. THE word navigation commands SHALL support selection extension: when issued with the Extend modifier, THE system SHALL extend the selection from the anchor to the new caret position. [SCI-EDIT-2.2]

7.9. THE system SHALL support configurable word-character classification via `SetCharClasses` and `SetDefaultCharClasses` APIs, allowing applications to customize which characters are treated as word characters. [SCI-DOC-16]

7.10. WHEN word navigation moves the caret, THE system SHALL scroll the Viewport if necessary to keep the caret visible, following the configured caret visibility policy. [SCI-EDIT-2.2]

7.11. THE word navigation commands SHALL be registered with the command framework as non-undoable commands (caret/viewport state only). [WB]

---

### Requirement 8: Word-Part Navigation (camelCase / Sub-Word)

**User Story:** As a developer editing camelCase or snake_case identifiers, I want to move the caret by sub-word boundaries so that I can navigate within compound identifiers efficiently.

**Source:** [SCI-DOC-16] criterion 6

#### Acceptance Criteria

8.1. WHEN word-part-left navigation is triggered, THE system SHALL move the caret to the previous sub-word boundary within the current word, detecting boundaries at: [SCI-DOC-16]
   - Transitions from lowercase to uppercase (camelCase: `getValue` → `get|Value`)
   - Transitions between alphanumeric and punctuation/separator characters (e.g., underscores in `get_value` → `get|_|value`)
   - Start of the word (if no internal boundary exists)

8.2. WHEN word-part-right navigation is triggered, THE system SHALL move the caret to the next sub-word boundary within the current word, detecting the same transition types as word-part-left but in the forward direction. [SCI-DOC-16]

8.3. WHEN the caret is at the beginning of a word and word-part-left is triggered, THE system SHALL move to the end of the previous word (crossing the word boundary to reach the last sub-word part of the preceding word). [SCI-DOC-16]

8.4. WHEN the caret is at the end of a word and word-part-right is triggered, THE system SHALL move to the beginning of the next word (crossing the word boundary to reach the first sub-word part of the following word). [SCI-DOC-16]

8.5. THE word-part navigation SHALL detect the following boundary patterns: [SCI-DOC-16]
   - `lowerUpper` — boundary before the uppercase letter (e.g., `my|Method`)
   - `UPPER_UPPER_lower` — boundary before the last uppercase in a run preceding a lowercase (e.g., `XML|Parser`)
   - `alpha_nonalpha` — boundary at transitions between alphanumeric and non-alphanumeric characters
   - `digit_alpha` and `alpha_digit` — boundaries between digits and letters

8.6. THE word-part navigation commands SHALL support selection extension: when issued with the Extend modifier, THE system SHALL extend the selection from the anchor to the new caret position. [SCI-DOC-16]

8.7. WHEN word-part navigation moves the caret, THE system SHALL scroll the Viewport if necessary to keep the caret visible. [SCI-EDIT-2.2]

8.8. THE word-part navigation commands SHALL be registered with the command framework as non-undoable commands (caret/viewport state only). [WB]

---

### Requirement 9: Vertical Caret Movement and Column Affinity

**User Story:** As a developer moving the caret up and down through lines of varying length, I want the caret to maintain its preferred horizontal column position so that vertical movement feels natural and predictable.

**Source:** [SCI-EDIT-2.2] criterion 12

#### Acceptance Criteria

9.1. WHEN the caret moves vertically (line up, line down, page up, page down), THE system SHALL compute the target line position using the stored Column_Affinity value (last chosen X coordinate) rather than the current caret column. [SCI-EDIT-2.2]

9.2. WHEN the caret moves horizontally (character left/right, word left/right, home, end), THE system SHALL update Column_Affinity to reflect the new caret position. [SCI-EDIT-2.2]

9.3. WHEN the target line is shorter than the Column_Affinity value, THE system SHALL place the caret at the end of the target line (clamped to line length) without modifying Column_Affinity, so that subsequent vertical movement returns to the preferred column on longer lines. [SCI-EDIT-2.2]

9.4. WHEN the caret moves to a line that is at least as long as the Column_Affinity value, THE system SHALL place the caret at the column indicated by Column_Affinity. [SCI-EDIT-2.2]

9.5. WHEN line-up or line-down is triggered and moves the caret off the visible viewport, THE system SHALL scroll the Viewport to keep the caret visible, delegating to the `viewport-and-scrolling` caret visibility policy. [SCI-EDIT-2.2]

9.6. WHEN page-up navigation is triggered, THE system SHALL move the caret up by one page (visible_count lines) while maintaining Column_Affinity. [SCI-EDIT-2.2]

9.7. WHEN page-down navigation is triggered, THE system SHALL move the caret down by one page (visible_count lines) while maintaining Column_Affinity. [SCI-EDIT-2.2]

9.8. WHEN the caret would move above line 1 (via line-up or page-up), THE system SHALL clamp the caret at line 1. [SCI-EDIT-2.2]

9.9. WHEN the caret would move below the last document line (via line-down or page-down), THE system SHALL clamp the caret at the last line. [SCI-EDIT-2.2]

9.10. THE vertical caret movement commands SHALL support selection extension: when issued with the Extend modifier, THE system SHALL extend the selection from the anchor to the new caret position. [SCI-EDIT-2.2]

---

### Requirement 10: Document Start and End Navigation

**User Story:** As a developer, I want single-keystroke commands to jump to the very beginning or very end of the document so that I can navigate to extremes instantly.

**Source:** [SCI-EDIT-2.2] criteria 8–9

#### Acceptance Criteria

10.1. WHEN document-start navigation is triggered (e.g., Ctrl+Home key binding or `DOC_START` command), THE system SHALL move the caret to position 0 (first character of the first line) and scroll the Viewport to show line 1 at the top. [SCI-EDIT-2.2]

10.2. WHEN document-end navigation is triggered (e.g., Ctrl+End key binding or `DOC_END` command), THE system SHALL move the caret to the end of the last line and scroll the Viewport to show the last page. [SCI-EDIT-2.2]

10.3. THE document-start and document-end commands SHALL support selection extension: when issued with the Extend modifier, THE system SHALL extend the selection from the anchor to the new caret position. [SCI-EDIT-2.2]

10.4. THE document-start and document-end commands SHALL be registered with the command framework as non-undoable commands (caret/viewport state only). [WB]

10.5. WHEN document-start navigation is triggered, THE system SHALL reset Column_Affinity to column 1. [SCI-EDIT-2.2]

10.6. WHEN document-end navigation is triggered, THE system SHALL update Column_Affinity to reflect the caret's position on the last line. [SCI-EDIT-2.2]

---

### Requirement 11: SAVE, CANCEL, and END Commands (Delegation)

**User Story:** As a developer, I want explicit commands for persisting, discarding, and closing my edit session so that I never lose work accidentally.

**Source:** [FFE-CMD-11]

*Note: Full implementation of these commands is specified in `file-operations`. This requirement documents the command registration and dispatch interface only.*

#### Acceptance Criteria

11.1. WHEN `SAVE` is issued, THE system SHALL delegate execution to the `file-operations` module which performs atomic file write, clears modified markers, and updates the status area. [FFE-CMD-11]

11.2. WHEN `CANCEL` is issued, THE system SHALL delegate to the `file-operations` module which handles the unsaved-changes prompt and session close logic. [FFE-CMD-11]

11.3. WHEN `END` is issued, THE system SHALL delegate to the `file-operations` module which applies the configured end-of-session behaviour (save-and-exit or confirm-exit). [FFE-CMD-11]

11.4. THE SAVE, CANCEL, and END commands SHALL be registered with the command framework with appropriate metadata. SAVE is undoable (it resets the save point); CANCEL and END terminate the session. [FFE-CMD-11], [WB]

---

### Requirement 12: LOAD and RELOAD Commands (Delegation)

**User Story:** As a developer, I want to open a different file or refresh the current file from disk so that I can quickly switch context or pick up external changes.

**Source:** [FFE-CMD-12]

*Note: Full implementation is specified in `file-operations`. This requirement documents the command registration and dispatch interface only.*

#### Acceptance Criteria

12.1. WHEN `LOAD path` is issued, THE system SHALL delegate to the `file-operations` module which opens the file at the given path in a new or replacement document session. [FFE-CMD-12]

12.2. IF the path supplied to `LOAD` does not exist or is not readable, THEN THE system SHALL display "File not found: <path>" and SHALL leave the current session unchanged. [FFE-CMD-12]

12.3. WHEN `RELOAD` is issued and the current session has no unsaved changes, THE system SHALL delegate to the `file-operations` module which discards the edit buffer and re-reads the source file from disk. [FFE-CMD-12]

12.4. WHEN `RELOAD` is issued and the current session has unsaved changes, THE system SHALL prompt the user to confirm discarding changes before reloading. [FFE-CMD-12]

12.5. THE LOAD and RELOAD commands SHALL be registered with the command framework. Neither is undoable (they reset the document state entirely). [FFE-CMD-12], [WB]

---

### Requirement 13: DELETE Command (Delegation)

**User Story:** As a developer, I want to delete lines by primary command so that I can remove large ranges or tag-scoped lines in one step.

**Source:** [FFE-CMD-13]

*Note: Full implementation is specified in `edit-operations`. This requirement documents the command registration and dispatch interface only.*

#### Acceptance Criteria

13.1. WHEN `DELETE` is issued with a `D` or `DD` line command pending, THE system SHALL delegate to `edit-operations` which deletes the identified lines. [FFE-CMD-13]

13.2. WHEN `DELETE` is issued with a `CC...CC` block pending, THE system SHALL delegate to `edit-operations` which deletes the lines within the CC block. [FFE-CMD-13]

13.3. WHEN `DELETE TAGGED` is issued, THE system SHALL delegate to `edit-operations` which deletes all lines whose `tagged` flag is true. [FFE-CMD-13]

13.4. WHEN a DELETE operation completes, THE system SHALL record it as a single undoable Transaction (handled by `edit-operations`). [FFE-CMD-13]

13.5. IF DELETE is issued with no scope, THEN THE system SHALL display "DELETE requires a scope: use D/DD line commands, TAGGED, or a line range" and SHALL NOT modify the document. [FFE-CMD-13]

---

### Requirement 14: COPY Command — In-Document Mode (Delegation)

**User Story:** As a developer, I want to copy lines within the document using C/CC source markers and A/B target markers so that I can duplicate content to another location.

**Source:** [FFE-CMD-14]

*Note: Full implementation is specified in `edit-operations`. This requirement documents the command registration and dispatch interface only.*

#### Acceptance Criteria

14.1. WHEN a `C` or `CC...CC` source marker is pending and an `A` or `B` target marker is entered, THE system SHALL delegate to `edit-operations` which copies the marked source lines to the target position. [FFE-CMD-14]

14.2. WHEN `COPY` is issued as a primary command with source and target markers pending, THE system SHALL delegate to `edit-operations` to resolve and execute the in-document copy. [FFE-CMD-14]

14.3. WHEN a COPY operation completes, THE system SHALL record it as a single undoable Transaction (handled by `edit-operations`). [FFE-CMD-14]

14.4. IF `COPY path` is issued while `C`/`CC` source markers are pending, THEN THE system SHALL display "Source line commands cannot be combined with a file path argument" and SHALL NOT execute the copy. [FFE-CMD-14]

---

### Requirement 15: MOVE Command (Delegation)

**User Story:** As a developer, I want to move lines from one position to another using M/MM source markers and A/B target markers so that I can reorganise document content.

**Source:** [FFE-CMD-15]

*Note: Full implementation is specified in `edit-operations`. This requirement documents the command registration and dispatch interface only.*

#### Acceptance Criteria

15.1. WHEN a `M` or `MM...MM` source marker is pending and an `A` or `B` target marker is entered, THE system SHALL delegate to `edit-operations` which removes the source lines and inserts them at the target position. [FFE-CMD-15]

15.2. WHEN `MOVE` is issued as a primary command with source and target markers pending, THE system SHALL delegate to `edit-operations` to resolve and execute the move. [FFE-CMD-15]

15.3. IF the target line falls inside the source block, THEN THE system SHALL display "Target cannot be inside the source block" and SHALL NOT modify the document. [FFE-CMD-15]

15.4. WHEN a MOVE operation completes, THE system SHALL record it as a single undoable Transaction (handled by `edit-operations`). [FFE-CMD-15]

---

### Requirement 16: MACRO / EXEC / RUN Commands (Delegation)

**User Story:** As a developer, I want to invoke editor macros from the command line using familiar aliases so that I can automate repetitive editing tasks.

**Source:** [FFE-CMD-18]

*Note: Full implementation is specified in `lua-macro-engine`. This requirement documents the command registration and dispatch interface only.*

#### Acceptance Criteria

16.1. WHEN `MACRO name` is issued, THE system SHALL delegate execution to the Macro Engine, passing the macro name and any additional arguments. [FFE-CMD-18]

16.2. WHEN `EXEC name` is issued, THE system SHALL treat it identically to `MACRO name`. [FFE-CMD-18]

16.3. WHEN `RUN name` is issued, THE system SHALL treat it identically to `MACRO name`. [FFE-CMD-18]

16.4. WHEN a macro completes successfully, THE system SHALL record the entire macro execution as a single undoable Transaction (handled by `lua-macro-engine`). [FFE-CMD-18]

16.5. IF the macro name cannot be resolved, THEN THE system SHALL display "Macro not found: <name>" and SHALL NOT modify the document. [FFE-CMD-18]

16.6. IF the macro raises an error during execution, THEN THE system SHALL roll back any partial document mutations made within that macro's Transaction where rollback is possible. [FFE-CMD-18]

---

### Requirement 17: UNDO and REDO Commands (Delegation)

**User Story:** As a developer, I want UNDO and REDO available as primary commands so that I can reverse and re-apply edit operations via the command line.

**Source:** [FFE-CMD-21]

*Note: Full implementation is specified in `undo-redo-transactions`. This requirement documents the command registration and dispatch interface only.*

#### Acceptance Criteria

17.1. WHEN `UNDO` is issued, THE system SHALL delegate to the Transaction service and display the result message returned by that service. [FFE-CMD-21]

17.2. WHEN `REDO` is issued, THE system SHALL delegate to the Transaction service and display the result message returned by that service. [FFE-CMD-21]

17.3. THE system SHALL NOT add `UNDO` or `REDO` to the command history. [FFE-CMD-21]

17.4. THE system SHALL NOT record `UNDO` or `REDO` themselves as undoable Transactions. [FFE-CMD-21]

---

### Requirement 18: Configuration Options

**User Story:** As a developer, I want key navigation behaviours to be configurable so that I can adapt navigation to my preferred workflow.

**Source:** [WB], [FFE-CMD-17], [SCI-DOC-16]

#### Acceptance Criteria

18.1. THE configuration system SHALL support `editor.navigation.horizontal_scroll_columns` (positive integer, default 8) controlling how many columns the LEFT and RIGHT commands scroll when issued without an explicit count. [FFE-CMD-17], [WB]

18.2. THE configuration system SHALL support `editor.navigation.page_overlap_lines` (non-negative integer, default 2) controlling how many lines of overlap are retained when scrolling by page (UP/DOWN without arguments). [WB]

18.3. THE configuration system SHALL support `editor.bounds.affect_find` (boolean, default false) controlling whether active Bounds restrict FIND operations. [FFE-CMD-20]

18.4. THE configuration system SHALL support `editor.navigation.word_characters` (string of additional characters to treat as word characters, default empty) extending the default word-character classification. [SCI-DOC-16]

18.5. WHEN a configuration key is missing or contains an invalid value, THE system SHALL fall back to the documented default and SHALL emit a warning via the logging subsystem. [WB]

---

### Requirement 19: Command Registration and Metadata

**User Story:** As a workbench developer, I want all navigation commands to be properly registered with the command framework so that they are discoverable, have help text, and are correctly classified for undo/redo.

**Source:** [WB]

#### Acceptance Criteria

19.1. THE following commands SHALL be registered as non-undoable primary commands: LOCATE, UP, DOWN, LEFT, RIGHT, TOP, BOTTOM, COLS, BOUNDS, BNDS, PARA_UP, PARA_DOWN, WORD_LEFT, WORD_RIGHT, WORD_PART_LEFT, WORD_PART_RIGHT, DOC_START, DOC_END. [WB]

19.2. THE following commands SHALL be registered as undoable primary commands: SORT. [FFE-CMD-10], [WB]

19.3. THE following commands SHALL be registered as delegation-only (owned by other specs): SAVE, CANCEL, END, LOAD, RELOAD, DELETE, COPY, MOVE, MACRO, EXEC, RUN, UNDO, REDO. [WB]

19.4. EACH registered command SHALL include a `help_text` field providing syntax and description for the HELP command. [WB]

19.5. EACH registered command SHALL include the canonical name and any aliases (e.g., BOUNDS/BNDS, MACRO/EXEC/RUN). [WB]

19.6. THE command metadata SHALL declare whether the command is valid in Browse mode, Edit mode, or both. Navigation commands (LOCATE, UP, DOWN, LEFT, RIGHT, TOP, BOTTOM, COLS, BOUNDS, paragraph, word, doc-start/end) SHALL be valid in both Browse and Edit modes. SORT SHALL be valid only in Edit mode. [WB]
