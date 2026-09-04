# Requirements Document

## Introduction

The `edit-operations` sub-project defines all text editing behaviour within the FileForgeWorkbench editor. It merges the FileForgeEditor MVP editing model (insert/overstrike modes, character insertion/deletion, transaction recording, modified line markers) with Scintilla's comprehensive editing commands, selection model, and multi-caret coordination concepts — all adapted to Rust idioms and the workbench's GUI-independent, command-driven architecture.

This spec covers:
- How characters enter and leave the document buffer (insert, overstrike, delete at multiple granularities)
- How selections are created, extended, and manipulated (stream, rectangular, multi-caret)
- How multiple carets operate simultaneously with coordinated edits
- How edit boundaries (BOUNDS) constrain the editable area — an ISPF/PDF heritage concept
- Line manipulation commands (transpose, duplicate, case change)
- Tab/indent handling
- Selection position adjustment when the document is modified externally
- Integration points with undo-redo, clipboard, and command framework

**Scope boundaries:**
- Undo/redo transaction mechanics (TransactionStack, coalescing, save points) are defined in `undo-redo-transactions` — this spec defines what constitutes a transaction unit
- Clipboard system-level access is defined in `clipboard-operations` — this spec defines the edit-side cut/copy/paste semantics
- Caret visual appearance (blink, width, colour) is defined in `caret-and-selection` — this spec defines the logical caret/selection model
- Navigation (word movement, paragraph movement, LOCATE, caret motion keys) is defined in `navigation-commands`
- The document buffer and line storage are defined in `document-model`
- Modified line marker *rendering* is defined in `caret-and-selection`; this spec defines when the marker is set

**Source references:**
- **[FFE-MVP-3]** = FileForgeEditor mvp-implementation Requirement 3 (Edit Mode, Undo, TransactionStack, modified markers, Save)
- **[FFE-MVP-8]** = FileForgeEditor mvp-implementation Requirement 8 (Standard Desktop Interactions)
- **[SCI-EDIT-2.2]** = Scintilla editor-editmodel Requirement 2.2 (Keyboard command handling, editing commands)
- **[SCI-EDIT-2.3]** = Scintilla editor-editmodel Requirement 2.3 (Multi-caret coordination)
- **[SCI-SEL-4.1]** = Scintilla selection model Requirement 4.1 (Selection model, SelectionPosition, SelectionRange)
- **[WB]** = Workbench Platform Architecture Brief (GUI independence, command-driven, multi-crate)

---

## Glossary

- **Caret**: The logical insertion point within the document. Each caret has a position (line + column offset) and may have an associated anchor for selection. [SCI-SEL-4.1]
- **Anchor**: The fixed end of a selection range. The selected region spans from anchor to caret. [SCI-SEL-4.1]
- **SelectionPosition**: A document position that includes both a real position (byte offset or line+column) and a virtual space offset for positions beyond line ends. Adjusts automatically on document modification. [SCI-SEL-4.1]
- **SelectionRange**: An ordered pair (anchor, caret) defining a contiguous selected region. May include virtual space at either end. [SCI-SEL-4.1]
- **Selection Container**: The top-level structure holding all active SelectionRanges, with operations: Add, Drop, Trim, MovePositions, and a designated main range. [SCI-SEL-4.1]
- **Stream Selection**: A selection that flows across line boundaries — from a position on one line through all intermediate lines to a position on another line. [SCI-SEL-4.1]
- **Rectangular Selection**: A column-oriented selection defined by a rectangle of (top-line, left-column) to (bottom-line, right-column), producing one selection segment per line. [SCI-SEL-4.1]
- **Multi-Caret**: Multiple independent carets active simultaneously, each with its own SelectionRange, receiving the same typed input. [SCI-EDIT-2.3]
- **Insert Mode**: The default editing mode where typed characters are inserted at the caret, pushing existing text rightward. [FFE-MVP-3]
- **Overstrike Mode**: An alternative editing mode where typed characters replace the character at the caret position. ISPF/mainframe heritage. [FFE-MVP-3, SCI-EDIT-2.2]
- **Grapheme Cluster**: A user-perceived character that may consist of multiple Unicode code points (e.g., base + combining diacritics, emoji ZWJ sequences).
- **BOUNDS**: An ISPF concept that defines left and right column limits constraining where edits may be applied within a line. [FFE-MVP-3]
- **Virtual Space**: Positions beyond the end of a line's text content, where the caret can be placed but no characters exist yet. When an edit occurs in virtual space, the space is "realised" by padding with actual space characters. [SCI-SEL-4.1, SCI-EDIT-2.3]
- **Edit Transaction**: A unit of work recorded for undo/redo. This spec defines transaction boundaries; mechanics are in `undo-redo-transactions`. [FFE-MVP-3]
- **UndoGroup**: A composite transaction wrapping multiple sub-operations (e.g., multi-caret insert) into a single undoable unit. [SCI-EDIT-2.3]
- **EditorTransaction**: A transaction value storing before/after line snapshots for modified lines. [FFE-MVP-3]
- **Protected Range**: A document region marked as read-only that multi-caret operations must skip rather than error on. [SCI-EDIT-2.3]
- **Modified Line Marker**: A visual indicator (`*`) displayed in the prefix area for lines that have been modified since the last save. [FFE-MVP-3]

---

## Requirements

### Requirement 1: Insert Mode — Character Insertion [FFE-MVP-3, SCI-EDIT-2.2]

**User Story:** As an editor user, I want to type characters that are inserted at the caret position, so that I can compose and extend text content naturally.

#### Acceptance Criteria

1. WHEN the editor is in Insert Mode and the user types a printable character, THE editor SHALL insert that character at the current caret position, shifting all subsequent characters on the same line one position to the right. [FFE-MVP-3]

2. WHEN a character is inserted, THE editor SHALL advance the caret one position to the right (past the newly inserted character). [FFE-MVP-3]

3. WHEN the user types a character that forms part of a multi-code-point grapheme cluster (e.g., combining diacritical mark, emoji ZWJ sequence), THE editor SHALL treat the complete grapheme cluster as a single character unit for cursor movement and deletion purposes. [SCI-EDIT-2.2]

4. WHEN the editor starts or a new document is opened, THE default editing mode SHALL be Insert Mode. [FFE-MVP-3]

5. WHEN a character is inserted in Insert Mode, THE editor SHALL push an EditorTransaction onto the TransactionStack recording the before-snapshot and after-snapshot of the affected line. [FFE-MVP-3]

6. WHEN a character is inserted, THE editor SHALL set the modified line marker on the affected line. [FFE-MVP-3]

7. WHEN the caret is positioned in virtual space (beyond the end of the line) and the user types a character in Insert Mode, THE editor SHALL realise the virtual space by padding the line with space characters up to the caret position, then insert the character. [SCI-EDIT-2.3, SCI-SEL-4.1]

8. WHEN a Tab key is pressed in Insert Mode, THE editor SHALL insert either a literal tab character or the configured number of space characters (per the indentation settings), advancing the caret to the next tab stop. [SCI-EDIT-2.2]

---

### Requirement 2: NewLine Handling [FFE-MVP-3, SCI-EDIT-2.2]

**User Story:** As an editor user, I want pressing Enter to behave correctly based on the current editing mode — splitting lines in Insert Mode and moving to the next line in Overstrike Mode — so that line manipulation matches my expectation based on the editing paradigm.

#### Acceptance Criteria

1. WHEN the user presses Enter (or Return) in Insert Mode, THE editor SHALL split the current line at the caret position — text before the caret remains on the current line; text from the caret onward becomes a new line inserted immediately below. [FFE-MVP-3, SCI-EDIT-2.2]

2. WHEN a line is split by Enter in Insert Mode, THE caret SHALL move to column 1 of the newly created line. [FFE-MVP-3]

3. WHEN Enter is pressed in Overstrike Mode, THE editor SHALL move the caret to the beginning of the next line without splitting the current line (mainframe terminal behaviour). [FFE-MVP-3]

4. WHEN Enter is pressed in Insert Mode with a selection active, THE editor SHALL first delete the selected text, then perform the line split at the resulting caret position. [SCI-EDIT-2.2]

5. WHEN a new line is created by Enter, THE editor SHALL record the operation as an EditorTransaction with before/after line snapshots of all affected lines. [FFE-MVP-3]

6. WHEN a new line is created, THE new line's line ending SHALL match the document's configured line ending style (LF, CRLF, or CR). [SCI-EDIT-2.2]

---

### Requirement 3: Overstrike Mode [FFE-MVP-3, SCI-EDIT-2.2]

**User Story:** As an editor user familiar with mainframe terminals, I want an overstrike mode where typing replaces existing characters rather than inserting, so that I can edit fixed-format records without disturbing column alignment.

#### Acceptance Criteria

1. WHEN the editor is in Overstrike Mode and the user types a printable character, THE editor SHALL replace the character at the current caret position with the typed character (the line length does not change unless the caret is at or beyond the end of the line). [FFE-MVP-3, SCI-EDIT-2.2]

2. WHEN the caret is at or beyond the end of the current line in Overstrike Mode, THE editor SHALL append the typed character (equivalent to insert behaviour at end-of-line). [FFE-MVP-3]

3. WHEN the user presses the Insert key, THE editor SHALL toggle between Insert Mode and Overstrike Mode. [FFE-MVP-8, SCI-EDIT-2.2]

4. WHEN the editing mode changes, THE editor SHALL update the mode indicator to display "INSERT" or "OVERSTRIKE" respectively (status bar integration via `menu-and-statusbar`). [FFE-MVP-3]

5. WHEN a character is replaced in Overstrike Mode, THE editor SHALL record the replacement as an EditorTransaction preserving the original character in the before-snapshot. [FFE-MVP-3]

6. WHEN a character is replaced in Overstrike Mode, THE editor SHALL set the modified line marker on the affected line. [FFE-MVP-3]

7. WHEN a selection is active and the user types a character in Overstrike Mode, THE editor SHALL delete the selected text and insert the typed character at the former selection start (same as Insert Mode behaviour when a selection exists). [SCI-EDIT-2.2]

8. THE editing mode (Insert or Overstrike) SHALL be a per-editor-instance setting that persists for the lifetime of the editor session. [FFE-MVP-3]

---

### Requirement 4: Delete Operations [FFE-MVP-3, SCI-EDIT-2.2]

**User Story:** As an editor user, I want multiple ways to delete text (character, word, line, to end/start of line), so that I can efficiently remove content at various granularities.

#### Acceptance Criteria

1. WHEN the user presses Backspace (DeleteBack) with no active selection, THE editor SHALL delete the grapheme cluster immediately before the caret and move the caret one position to the left. [FFE-MVP-3, SCI-EDIT-2.2]

2. WHEN the user presses Backspace at the beginning of a line (column 1) with no active selection, THE editor SHALL join the current line to the end of the previous line, moving the caret to the junction point. [FFE-MVP-3, SCI-EDIT-2.2]

3. WHEN the user presses Delete (DelChar) with no active selection, THE editor SHALL delete the grapheme cluster at the caret position without moving the caret. [FFE-MVP-3, SCI-EDIT-2.2]

4. WHEN the user presses Delete at the end of a line with no active selection, THE editor SHALL join the next line to the end of the current line at the caret position. [FFE-MVP-3, SCI-EDIT-2.2]

5. WHEN the user presses Ctrl+Backspace (DelWordLeft), THE editor SHALL delete the word (contiguous sequence of word characters) immediately before the caret. [SCI-EDIT-2.2]

6. WHEN the user presses Ctrl+Delete (DelWordRight), THE editor SHALL delete the word immediately after the caret. [SCI-EDIT-2.2]

7. WHEN the user invokes "Delete Line" (LineDelete, Ctrl+Shift+K), THE editor SHALL delete the entire current line (including its line ending) and move the caret to the same column on the next line (or previous line if the deleted line was the last). [SCI-EDIT-2.2]

8. WHEN the user invokes "Delete to End of Line" (DelLineRight, Ctrl+Shift+Delete), THE editor SHALL delete all text from the caret position to the end of the current line without removing the line itself. [SCI-EDIT-2.2]

9. WHEN the user invokes "Delete to Start of Line" (DelLineLeft, Ctrl+Shift+Backspace), THE editor SHALL delete all text from the beginning of the current line up to (but not including) the caret position. [SCI-EDIT-2.2]

10. WHEN a selection is active and the user presses Backspace or Delete, THE editor SHALL delete the entire selected text and collapse the caret to the start of the former selection. [FFE-MVP-8, SCI-EDIT-2.2]

11. WHEN any delete operation is performed, THE editor SHALL record it as an EditorTransaction with before/after line snapshots and set the modified line marker on affected lines. [FFE-MVP-3]

12. WHEN the caret is in virtual space and Backspace is pressed, THE editor SHALL move the caret to the end of the actual line content without modifying the document. [SCI-EDIT-2.2]

---

### Requirement 5: Line Manipulation Commands [SCI-EDIT-2.2]

**User Story:** As an editor user, I want commands to transpose, duplicate, and change the case of lines or selections, so that I can restructure and transform text without manual re-typing.

#### Acceptance Criteria

1. WHEN the user invokes "Line Transpose" (Ctrl+T with no selection), THE editor SHALL swap the current line with the line above it and move the caret to the swapped line (maintaining column position). [SCI-EDIT-2.2]

2. WHEN the user invokes "Line Duplicate" (Ctrl+Shift+D), THE editor SHALL insert a copy of the current line (or all lines touched by the selection) immediately below the original, placing the caret on the duplicated content. [SCI-EDIT-2.2]

3. WHEN the user invokes "Uppercase" (Ctrl+Shift+U) with a selection active, THE editor SHALL convert all characters in the selection to their Unicode uppercase equivalents. [SCI-EDIT-2.2]

4. WHEN the user invokes "Lowercase" (Ctrl+U) with a selection active, THE editor SHALL convert all characters in the selection to their Unicode lowercase equivalents. [SCI-EDIT-2.2]

5. WHEN the user invokes "Toggle Case" with a selection active, THE editor SHALL invert the case of each alphabetic character in the selection (uppercase → lowercase, lowercase → uppercase). [SCI-EDIT-2.2]

6. WHEN any line manipulation command is invoked without a selection, THE editor SHALL operate on the entire current line. [SCI-EDIT-2.2]

7. WHEN any line manipulation command is performed, THE editor SHALL record it as a single EditorTransaction and set modified line markers on all affected lines. [FFE-MVP-3, SCI-EDIT-2.2]

8. IF Line Transpose is invoked on the first line of the document (no line above), THE editor SHALL take no action and SHALL NOT record a transaction. [SCI-EDIT-2.2]

---

### Requirement 6: Selection Model [FFE-MVP-8, SCI-SEL-4.1]

**User Story:** As an editor user, I want to select text using keyboard and mouse so that I can operate on regions of text (delete, replace, copy, cut).

#### Acceptance Criteria

1. THE selection model SHALL represent each selection as a SelectionRange containing an anchor (SelectionPosition) and a caret (SelectionPosition). The selected region is all text between anchor and caret regardless of document order. [SCI-SEL-4.1]

2. EACH SelectionPosition SHALL consist of a real document position (line + column offset) and a virtual space offset (non-negative integer representing columns beyond the line end). [SCI-SEL-4.1]

3. THE editor SHALL maintain a Selection container that holds one or more SelectionRanges, with exactly one designated as the "main" range. [SCI-SEL-4.1]

4. WHEN the user holds Shift and presses an arrow key (Left, Right, Up, Down), THE editor SHALL extend the current selection by moving the caret in the specified direction while keeping the anchor fixed. [FFE-MVP-8]

5. WHEN the user holds Shift and presses Home, THE editor SHALL extend the selection to the beginning of the current line. [FFE-MVP-8]

6. WHEN the user holds Shift and presses End, THE editor SHALL extend the selection to the end of the current line. [FFE-MVP-8]

7. WHEN the user holds Shift+Ctrl and presses Left or Right, THE editor SHALL extend the selection by one word in the corresponding direction. [FFE-MVP-8]

8. WHEN the user holds Shift and presses Page Up or Page Down, THE editor SHALL extend the selection by one viewport page in the corresponding direction. [FFE-MVP-8]

9. WHEN the user presses Ctrl+A, THE editor SHALL select all text in the document (anchor at document start, caret at document end). [FFE-MVP-8]

10. WHEN a selection is active and the user types a printable character, THE editor SHALL delete the selected text and insert the typed character at the former selection start (selection replacement). [FFE-MVP-8, SCI-EDIT-2.2]

11. WHEN the user presses an arrow key without Shift, THE active selection SHALL be collapsed — the caret moves to the appropriate end of the former selection and the anchor is reset to match the caret. [FFE-MVP-8]

12. WHEN the user clicks (without Shift or Ctrl) at a position in the document, THE editor SHALL place the caret at that position and clear any existing selection (anchor = caret). [FFE-MVP-8]

13. WHEN the user clicks with Shift held, THE editor SHALL extend the selection from the existing anchor to the clicked position. [FFE-MVP-8]

14. WHEN the user clicks and drags, THE editor SHALL create a stream selection from the click-down position (anchor) to the current drag position (caret), updating in real time. [FFE-MVP-8]

15. WHEN the user double-clicks a word, THE editor SHALL select the entire word (as defined by word character classification from `encoding-and-characters`). [FFE-MVP-8]

16. WHEN the user triple-clicks, THE editor SHALL select the entire line including the line ending. [FFE-MVP-8]

17. THE editor SHALL visually highlight selected text using a distinct selection background colour (rendering details in `caret-and-selection`). [FFE-MVP-8]

---

### Requirement 7: Selection Position Adjustment [SCI-SEL-4.1]

**User Story:** As a workbench developer, I want selection positions to automatically adjust when the document is modified (by any source — typing, undo, external reload), so that selections remain semantically correct after edits.

#### Acceptance Criteria

1. WHEN text is inserted before a SelectionPosition, THE position SHALL be shifted forward by the length of the inserted text. [SCI-SEL-4.1]

2. WHEN text is inserted at the exact location of a SelectionPosition, THE position SHALL shift forward (insert-before semantics) unless the position is an anchor at the start of a selection (anchor stays, selection grows). [SCI-SEL-4.1]

3. WHEN text is deleted that spans a SelectionPosition, THE position SHALL be moved to the start of the deleted range. [SCI-SEL-4.1]

4. WHEN text is deleted entirely before a SelectionPosition, THE position SHALL be shifted backward by the length of the deleted text. [SCI-SEL-4.1]

5. THE Selection container SHALL provide a `MovePositions` operation that adjusts all SelectionPositions in all ranges when a document modification occurs, accepting the modification offset, length inserted, and length deleted. [SCI-SEL-4.1]

6. WHEN position adjustment causes a SelectionRange's anchor and caret to become equal, THE selection SHALL be treated as collapsed (no selection, just a caret). [SCI-SEL-4.1]

7. WHEN position adjustment causes two SelectionRanges to overlap or become identical, THE Selection container SHALL merge them into a single range (Trim operation). [SCI-SEL-4.1]

---

### Requirement 8: Multi-Caret Editing [SCI-EDIT-2.3, SCI-SEL-4.1]

**User Story:** As a power user, I want to place multiple carets in the document simultaneously, so that I can make the same edit at several locations at once.

#### Acceptance Criteria

1. THE editor SHALL support multiple simultaneous carets, each represented as an independent SelectionRange within the Selection container. [SCI-SEL-4.1]

2. WHEN the user holds Ctrl (or Cmd on macOS) and clicks at a new position, THE editor SHALL add a new caret at that position (Add operation on the Selection container) without removing existing carets. [SCI-SEL-4.1]

3. WHEN the user holds Ctrl and clicks on an existing caret position, THE editor SHALL remove that caret (Drop operation), provided at least one caret remains. [SCI-SEL-4.1]

4. WHEN multiple carets are active and the user types a character, THE editor SHALL insert (or overstrike) that character at every caret simultaneously, processing carets in reverse document order (last-to-first) so that earlier insertions do not shift positions for later ones. [SCI-EDIT-2.3]

5. WHEN multiple carets are active and the user presses Backspace or Delete, THE editor SHALL perform the delete operation at every caret simultaneously, processing in reverse document order. [SCI-EDIT-2.3]

6. WHEN multiple carets are active, THE Selection container SHALL designate one as the "main" range — this determines viewport auto-scroll position and status bar display. [SCI-SEL-4.1]

7. WHEN multiple carets are active and the user performs a navigation operation (arrow keys, Home, End), THE editor SHALL move all carets in the same direction simultaneously. [SCI-EDIT-2.3]

8. WHEN multiple carets would collapse to the same position after an operation, THE Selection container SHALL merge them (Trim operation), reducing to a single caret at that position. [SCI-EDIT-2.3, SCI-SEL-4.1]

9. WHEN the user presses Escape while multiple carets are active, THE editor SHALL reduce to a single caret (the main range), removing all additional carets via ClearSelection. [SCI-EDIT-2.3]

10. WHEN the user invokes "Add Caret Above" (Ctrl+Alt+Up), THE editor SHALL add a new caret one line above the main caret at the same column. [SCI-EDIT-2.3]

11. WHEN the user invokes "Add Caret Below" (Ctrl+Alt+Down), THE editor SHALL add a new caret one line below the main caret at the same column. [SCI-EDIT-2.3]

12. WHEN multiple carets exist, EACH caret MAY have its own independent selection range — Shift+Arrow extends selection at all carets independently. [SCI-EDIT-2.3]

13. WHEN multiple carets exist, all edit operations within a single user action SHALL be recorded as a single UndoGroup — one Undo command reverses the operation at all caret positions. [SCI-EDIT-2.3]

14. WHEN the user invokes "Select Next Occurrence" (Ctrl+D), THE editor SHALL find the next occurrence of the currently selected text (or word at caret) and add a new caret+selection at that occurrence. [SCI-EDIT-2.3]

15. WHEN a multi-caret insert encounters a protected range at one caret position, THE editor SHALL skip that caret's insertion (leaving the protected content unchanged) and continue processing remaining carets. [SCI-EDIT-2.3]

16. WHEN virtual space exists at any caret position during a multi-caret edit, THE editor SHALL realise the virtual space (pad with spaces) at that specific caret before performing the edit. [SCI-EDIT-2.3]

---

### Requirement 9: Rectangular/Column Selection [SCI-SEL-4.1]

**User Story:** As a user editing columnar data (fixed-format files, tables), I want to select and edit rectangular regions of text, so that I can insert or delete content in a specific column across multiple lines simultaneously.

#### Acceptance Criteria

1. WHEN the user holds Alt and drags the mouse, THE editor SHALL create a rectangular (column) selection defined by the drag start position (top-left corner) and current mouse position (bottom-right corner). [SCI-SEL-4.1]

2. WHEN a rectangular selection is active, THE editor SHALL display the selection as a column highlight — one selection segment per line between the top and bottom rows, each spanning the same left-to-right column range. [SCI-SEL-4.1]

3. WHEN the user holds Alt+Shift and presses an arrow key, THE editor SHALL extend the rectangular selection in the corresponding direction. [SCI-SEL-4.1]

4. WHEN the user types a character with a rectangular selection active, THE editor SHALL insert that character at the left edge of the rectangular selection on every line within the selection (column insert). [SCI-SEL-4.1, SCI-EDIT-2.2]

5. WHEN the user presses Delete or Backspace with a rectangular selection active, THE editor SHALL delete the selected column region on every affected line, shifting remaining text leftward. [SCI-SEL-4.1, SCI-EDIT-2.2]

6. WHEN a rectangular selection is copied, THE clipboard content SHALL preserve the rectangular structure (one segment per line, separated by line endings), tagged with rectangular metadata. [SCI-SEL-4.1]

7. WHEN rectangular clipboard content is pasted, THE editor SHALL insert each line of the clipboard content as a column at the caret position, one segment per document line starting from the caret's line. [SCI-SEL-4.1]

8. WHEN a rectangular selection spans lines of differing lengths and the right column exceeds a line's length, THE editor SHALL treat the missing columns as virtual space for selection display and edit purposes. [SCI-SEL-4.1]

9. WHEN the user presses Escape while a rectangular selection is active, THE editor SHALL collapse the selection to a single caret at the position of the main selection's caret. [SCI-SEL-4.1]

10. WHEN the user invokes "Column Select Mode" toggle, THE editor SHALL switch the selection mode between stream and rectangular for subsequent keyboard-driven selection extensions. [SCI-SEL-4.1]

---

### Requirement 10: Clipboard Integration (Edit-Side Semantics) [FFE-MVP-8, SCI-SEL-4.1]

**User Story:** As an editor user, I want cut, copy, and paste operations to work correctly with all selection types (stream, rectangular, multi-caret), so that I can efficiently move and duplicate text.

#### Acceptance Criteria

1. WHEN the user invokes Copy (Ctrl+C) with a single stream selection active, THE editor SHALL copy the selected text to the system clipboard. [FFE-MVP-8]

2. WHEN the user invokes Cut (Ctrl+X) with a single stream selection active, THE editor SHALL copy the selected text to the system clipboard, delete it from the document, and record the deletion as an EditorTransaction. [FFE-MVP-8]

3. WHEN the user invokes Paste (Ctrl+V) with a single caret (no selection), THE editor SHALL insert the clipboard content at the caret position and record the insertion as an EditorTransaction. [FFE-MVP-8]

4. WHEN the user invokes Paste with an active selection, THE editor SHALL replace the selected text with the clipboard content. [FFE-MVP-8]

5. WHEN no selection is active and the user invokes Copy, THE editor SHALL copy the entire current line (including line ending) to the clipboard as "line copy" mode. [SCI-EDIT-2.2]

6. WHEN line-copy content is pasted, THE editor SHALL insert it as a new line above the current caret line rather than inline at the caret position. [SCI-EDIT-2.2]

7. WHEN multiple carets are active and the user invokes Copy, THE editor SHALL copy text from each caret's selection (or each caret's full line if no selection), concatenated with the configured copy separator (default: newline). [SCI-EDIT-2.3]

8. WHEN multiple carets are active and clipboard content contains the same number of segments as there are carets, Paste SHALL distribute one segment to each caret position rather than pasting the full content at every caret. [SCI-EDIT-2.3]

9. WHEN a rectangular selection is copied, THE editor SHALL place the content on the clipboard with metadata indicating rectangular format. [SCI-SEL-4.1]

10. WHEN rectangular clipboard content is pasted at a single caret, THE editor SHALL insert the content as a column block starting at the caret position. [SCI-SEL-4.1]

11. THE editor SHALL provide a context menu containing Cut, Copy, Paste, and Select All items that invoke the same commands as their keyboard equivalents. [FFE-MVP-8]

12. WHEN the clipboard operation fails (e.g., system clipboard unavailable), THE editor SHALL display a descriptive error message in the status bar and SHALL NOT modify the document. [FFE-MVP-8]

---

### Requirement 11: Transaction Recording and Modified Line Markers [FFE-MVP-3]

**User Story:** As an editor user, I want every edit to be recorded for undo/redo and I want a visual indicator showing which lines I have modified since the last save, so that I can track my changes and revert them if needed.

#### Acceptance Criteria

1. THE editor SHALL maintain a TransactionStack that stores EditorTransaction values, each containing before-snapshot and after-snapshot of the affected lines. [FFE-MVP-3]

2. WHEN a character is typed (insert or overstrike), THE editor SHALL push an EditorTransaction recording the modification onto the TransactionStack. [FFE-MVP-3]

3. WHEN a character is deleted, THE editor SHALL push an EditorTransaction recording the deletion onto the TransactionStack. [FFE-MVP-3]

4. WHEN the user invokes UNDO (Ctrl+Z), THE editor SHALL pop the most recent EditorTransaction from the TransactionStack and restore the before-snapshot of the affected lines. [FFE-MVP-3, FFE-MVP-8]

5. WHEN the user invokes REDO (Ctrl+Y or Ctrl+Shift+Z), THE editor SHALL re-apply the most recently undone EditorTransaction (the after-snapshot). [FFE-MVP-3, FFE-MVP-8]

6. THE editor SHALL display a modified line marker (`*`) in the prefix area for every line that has been modified since the last successful save. [FFE-MVP-3]

7. WHEN a SAVE operation completes successfully, THE editor SHALL clear all modified line markers. [FFE-MVP-3]

8. WHEN UNDO restores a line to its saved state, THE modified line marker for that line SHALL be cleared. WHEN UNDO causes a line to differ from its saved state, THE marker SHALL be set. [FFE-MVP-3]

9. WHEN a multi-caret operation is performed, all sub-operations SHALL be wrapped in a single UndoGroup so that one UNDO command reverses all of them. [SCI-EDIT-2.3]

---

### Requirement 12: Save Operations [FFE-MVP-3]

**User Story:** As an editor user, I want to save my work reliably with atomic file writes, clear visual confirmation of success, and retained state on failure, so that I never lose data due to interrupted saves.

#### Acceptance Criteria

1. WHEN the user invokes SAVE (Ctrl+S), THE editor SHALL write the document content to a temporary file in the same directory as the target, then perform an atomic rename to replace the target file. [FFE-MVP-3]

2. WHEN SAVE completes successfully, THE editor SHALL clear all modified line markers and display a confirmation message (e.g., in the status bar). [FFE-MVP-3]

3. IF a SAVE operation fails (I/O error, permission denied, disk full), THE editor SHALL preserve all modified line markers unchanged, display a descriptive error message, and leave the document in its current modified state. [FFE-MVP-3]

4. WHEN SAVE completes successfully, THE TransactionStack SHALL record a save point so that the undo system can correctly determine modified state relative to the saved version. [FFE-MVP-3]

5. THE SAVE command SHALL be registered with the command framework and invokable from keyboard shortcut, menu, and scripting. [WB]

---

### Requirement 13: Edit Boundaries (BOUNDS) [FFE-MVP-3]

**User Story:** As an ISPF user, I want to set left and right column boundaries that restrict where edits can be applied, so that I can protect fixed columns (sequence numbers, identification fields) from accidental modification.

#### Acceptance Criteria

1. WHEN the BOUNDS primary command is issued with two column numbers (left, right), THE editor SHALL set the left boundary and right boundary for the current editing session. [FFE-MVP-3]

2. WHEN BOUNDS is active and the user types in Insert Mode, THE editor SHALL only allow character insertion within the bounded column range — characters typed at positions outside the bounds SHALL be ignored. [FFE-MVP-3]

3. WHEN BOUNDS is active and the user types in Overstrike Mode, THE editor SHALL only allow character replacement within the bounded column range. [FFE-MVP-3]

4. WHEN BOUNDS is active and a line-split (Enter in Insert Mode) would affect columns outside the bounded range, THE editor SHALL restrict the split to operate only on content within the bounds — content outside the bounds remains on the original line unchanged. [FFE-MVP-3]

5. WHEN BOUNDS is active, delete operations (Backspace, Delete, Ctrl+Backspace, Ctrl+Delete) SHALL only remove characters within the bounded range. Attempting to delete outside the bounds SHALL have no effect. [FFE-MVP-3]

6. WHEN the BOUNDS primary command is issued with no arguments, THE editor SHALL reset (clear) the boundaries, allowing edits across the full line width. [FFE-MVP-3]

7. WHEN BOUNDS is active, THE editor SHALL display visual indicators (vertical guide lines or column shading) showing the left and right boundary positions. [FFE-MVP-3]

8. WHEN BOUNDS is active, THE status bar SHALL display the current BOUNDS column range. [FFE-MVP-3]

9. WHEN a selection extends beyond the bounded columns, edit operations on that selection SHALL only affect the portion within the bounds — text outside the bounds SHALL remain unchanged. [FFE-MVP-3]

10. WHEN BOUNDS is active and a paste operation is performed, THE pasted content SHALL be clipped to fit within the bounded column range — excess characters beyond the right boundary SHALL be truncated. [FFE-MVP-3]

11. THE BOUNDS setting SHALL be per-document (each open document/tab maintains its own BOUNDS state). [FFE-MVP-3]

12. WHEN BOUNDS values are set, THE left boundary SHALL be >= 1 and the right boundary SHALL be > left boundary. IF invalid values are supplied, THE editor SHALL display an error message and retain the previous BOUNDS state. [FFE-MVP-3]

---

### Requirement 14: Selection Container Operations [SCI-SEL-4.1]

**User Story:** As a workbench developer, I want a well-defined API for managing the set of active selections, so that all edit operations, plugins, and macros can manipulate selections consistently.

#### Acceptance Criteria

1. THE Selection container SHALL provide an `Add` operation that inserts a new SelectionRange, maintaining ranges sorted by document position. [SCI-SEL-4.1]

2. THE Selection container SHALL provide a `Drop` operation that removes a specified SelectionRange by index, failing gracefully if it would leave zero ranges (at least one must remain). [SCI-SEL-4.1]

3. THE Selection container SHALL provide a `Trim` operation that removes duplicate or overlapping SelectionRanges, merging any ranges that overlap into a single range covering the union. [SCI-SEL-4.1]

4. THE Selection container SHALL provide a `MovePositions` operation that adjusts all positions in all ranges given a document modification descriptor (offset, inserted length, deleted length). [SCI-SEL-4.1]

5. THE Selection container SHALL expose a `main_range` accessor that returns the designated main SelectionRange (used for scroll/status bar). [SCI-SEL-4.1]

6. THE Selection container SHALL expose a `set_main_range` operation that changes which range is the main range by index. [SCI-SEL-4.1]

7. THE Selection container SHALL provide a `ranges()` iterator yielding all SelectionRanges in document order. [SCI-SEL-4.1]

8. THE Selection container SHALL provide a `count()` method returning the number of active selections. [SCI-SEL-4.1]

9. THE Selection container SHALL be GUI-independent — it SHALL NOT reference any rendering or platform types. [WB]

---

### Requirement 15: Command-Driven Edit Dispatch [WB, SCI-EDIT-2.2]

**User Story:** As a workbench architect, I want all edit operations to be registered commands dispatched through the command framework, so that menus, keyboard shortcuts, macros, and plugins can all invoke editing functionality uniformly.

#### Acceptance Criteria

1. EACH edit operation (insert character, delete, line transpose, line duplicate, case change, toggle mode, select all, cut, copy, paste) SHALL be registered as a named command in the command framework's CommandRegistry. [WB]

2. THE edit-operations crate SHALL NOT directly handle keyboard input — it SHALL expose command handlers that the command framework invokes after key-to-command resolution. [WB]

3. EACH edit command SHALL declare its metadata (name, display label, default key binding, category) for discoverability by menus, keymaps, and the command palette. [WB]

4. EACH edit command that modifies the document SHALL return a result indicating success or failure (e.g., BOUNDS rejection, read-only document), enabling the command framework to report status. [WB]

5. THE edit-operations crate SHALL have no GUI dependency — it SHALL operate solely on the logical document model, selection container, and transaction stack. [WB]

6. EACH edit command SHALL be invokable from the Lua macro engine via the scripting bridge (integration point with `lua-macro-engine`). [WB]

---

## Cross-References

| Dependency | Relationship |
|---|---|
| `document-model` | Provides the gap buffer, line index, and content access API that edit operations modify. All edits go through the document model's mutation API. |
| `undo-redo-transactions` | Records all edit operations as EditorTransactions on the TransactionStack. Defines coalescing, save points, and UndoGroup semantics — this spec defines transaction boundaries. |
| `command-framework` | All edit operations are registered commands dispatched via CommandRegistry. Key bindings resolved by command framework before reaching edit handlers. |
| `caret-and-selection` | Defines visual rendering of carets, selection highlights, and modified line markers. This spec defines the logical selection/caret model. |
| `clipboard-operations` | Defines system clipboard access and the COPY command clipboard-paste mode. This spec defines edit-side cut/copy/paste logic. |
| `navigation-commands` | Defines caret movement commands (arrow, word, line, page). This spec defines how selection extends during Shift+navigation. Also defines BOUNDS command parsing. |
| `encoding-and-characters` | Provides grapheme cluster boundary detection and word character classification used by delete/insert/selection operations. |
| `find-and-replace` | BOUNDS also restricts FIND/CHANGE scope (defined in `find-and-replace`). Select Next Occurrence (Ctrl+D) uses find-and-replace search logic. |
| `menu-and-statusbar` | Displays mode indicator (INSERT/OVERSTRIKE), BOUNDS range, and clipboard error messages. |
| `lua-macro-engine` | Edit commands are invokable from Lua scripts via the scripting bridge. |

---

## Notes

- **Priority:** FileForgeEditor requirements take precedence on all conflicts with Scintilla concepts. Where Scintilla defines a different behaviour (e.g., Enter in overtype mode), FFE's ISPF-heritage behaviour is authoritative.
- **BOUNDS** is an ISPF heritage feature not found in Scintilla — it provides column-range protection for fixed-format file editing common in mainframe environments.
- **Virtual space** support (caret positioned beyond line end) is included to support rectangular selection on short lines and multi-caret column editing, per Scintilla's `virtualSpaceOptions`.
- **Multi-caret reverse-order processing** is essential to avoid position drift: when inserting at multiple positions, processing from last-to-first ensures earlier positions are not invalidated by later insertions within the same operation.
- **Protected range skipping** allows multi-caret operations to gracefully handle read-only regions (e.g., sequence number columns protected by BOUNDS or explicit read-only markers) without aborting the entire operation.
- **Enter key behaviour** differs between Insert Mode (line split) and Overstrike Mode (move to next line) — this is intentional ISPF/mainframe terminal behaviour preserved from FFE.
- **Keyboard shortcuts** listed (Ctrl+D, Ctrl+Shift+K, Ctrl+T, etc.) are defaults registered with the command framework's shortcut registry; users may remap them via `configuration-system`.
- **Platform-specific rendering**, C++ ABI details, and Scintilla's message-passing API (`WM_*`, `SCI_*` messages) are excluded — all concepts are adapted to Rust traits and method calls.
- **GUI independence**: The `edit-operations` crate has zero GUI dependencies. It operates on abstract types from `document-model` and produces transaction records for `undo-redo-transactions`. Visual feedback (caret rendering, selection highlighting, modified markers) is the responsibility of the GUI layer via `caret-and-selection`.

---

### Requirement 16: ISPF Edit Profile Commands [EARS: Edit-CAPS-mode, Edit-NULLS-mode, Edit-PROFILE-command, Edit-STATS-mode, Edit-LOCK-setting, Edit-profile-persist]

**User Story:** As an ISPF user, I want to view and change edit profile settings (CAPS, NULLS, PROFILE, STATS, LOCK) from within the editor, so that the editing environment matches my preferences and persists across sessions.

#### Acceptance Criteria

1. WHEN the user issues the CAPS ON primary command, THE editor SHALL convert all subsequently typed characters to uppercase before inserting them into the document. WHEN CAPS OFF is issued, THE editor SHALL revert to case-preserving input. [EARS: Edit-CAPS-mode]

2. WHEN the user issues the CAPS command with no argument, THE editor SHALL toggle the current CAPS mode state. [EARS: Edit-CAPS-mode]

3. WHEN CAPS mode is active, THE status bar SHALL display a CAPS indicator. [EARS: Edit-CAPS-mode]

4. WHEN the user issues the NULLS ON primary command, THE editor SHALL treat trailing null characters (0x00) on a line as equivalent to trailing spaces for display and editing purposes. WHEN NULLS OFF is issued, THE editor SHALL display null characters as visible placeholders. [EARS: Edit-NULLS-mode]

5. WHEN the user issues the PROFILE primary command, THE editor SHALL display the current edit profile settings (CAPS, NULLS, RECOVERY, AUTONUM, NUM, HILITE, STATS, LOCK, IMACRO) in a status overlay or panel. [EARS: Edit-PROFILE-command]

6. WHEN the user issues PROFILE with keyword arguments (e.g. PROFILE CAPS ON), THE editor SHALL update the named profile setting and confirm the change. [EARS: Edit-PROFILE-command]

7. WHEN the user issues STATS ON, THE editor SHALL display member statistics (creation date, modification date, modification count, user ID) in the prefix area or a dedicated column. WHEN STATS OFF is issued, THE statistics display SHALL be hidden. [EARS: Edit-STATS-mode]

8. WHEN the user issues LOCK ON, THE editor SHALL prevent further changes to the edit profile settings for the current session. WHEN LOCK OFF is issued, THE profile SHALL become editable again. [EARS: Edit-LOCK-setting]

9. WHEN the user closes and reopens a file, THE edit profile settings (CAPS, NULLS, AUTONUM, NUM, HILITE, STATS) SHALL be restored to the values they had when the file was last closed, persisted via the configuration system. [EARS: Edit-profile-persist]

10. WHEN the user issues AUTONUM ON, THE editor SHALL treat this as equivalent to NUMBER ON, enabling automatic line numbering. WHEN AUTONUM OFF is issued, THE editor SHALL treat this as equivalent to NUMBER OFF. [EARS: Edit-AUTONUM-mode -- extends sequence-numbers Req 6.7]

11. WHEN the user issues the NUM command, THE editor SHALL treat it as an alias for the NUMBER command, accepting the same arguments (ON, OFF, SHOW, STD, COBOL, etc.). [EARS: Edit-NUM-mode -- extends sequence-numbers Req 8]

12. WHEN the user issues HILITE followed by a mode keyword (ON, OFF, LOGIC, FIND, PAREN), THE editor SHALL delegate to the syntax-highlighting subsystem to apply the requested highlighting mode. [EARS: Edit-HILITE-setting -- extends syntax-highlighting]

---

### Requirement 17: Editor-Context Dataset Commands [EARS: PC-SUBMIT, PC-CREATE, PC-REPLACE, PC-EDIT-nested, PC-BROWSE, PC-VIEW, PC-COMPARE]

**User Story:** As an ISPF user, I want to issue dataset-level commands (SUBMIT, CREATE, REPLACE, EDIT, BROWSE, VIEW, COMPARE) from within the editor, so that I can manage and navigate datasets without leaving the editing context.

#### Acceptance Criteria

1. WHEN the user issues the SUBMIT primary command from within the editor, THE editor SHALL submit the current document buffer as a batch job via the JES subsystem (ff-jes) and display the assigned job ID in the status bar. [EARS: PC-SUBMIT]

2. WHEN the user issues the CREATE primary command with a dataset name argument, THE editor SHALL create a new dataset containing the lines currently selected (or all lines if no selection is active) and confirm the creation in the status bar. [EARS: PC-CREATE]

3. WHEN the user issues the REPLACE primary command with a dataset name argument, THE editor SHALL replace the content of the named dataset with the lines currently selected (or all lines if no selection is active) and confirm in the status bar. [EARS: PC-REPLACE]

4. WHEN the user issues the EDIT primary command with a dataset name argument from within an active editor session, THE editor SHALL open the named dataset in a new editor tab, leaving the current session open. [EARS: PC-EDIT-nested]

5. WHEN the user issues the BROWSE primary command with a dataset name argument, THE editor SHALL open the named dataset in a read-only browse tab. [EARS: PC-BROWSE]

6. WHEN the user issues the VIEW primary command with a dataset name argument, THE editor SHALL open the named dataset in a view tab (read-only with limited edit profile). [EARS: PC-VIEW]

7. WHEN the user issues the COMPARE primary command with a dataset name argument, THE editor SHALL open a compare view showing the differences between the current document and the named dataset. [EARS: PC-COMPARE]

8. WHEN any of the above commands (SUBMIT, CREATE, REPLACE, EDIT, BROWSE, VIEW, COMPARE) is issued with a missing or invalid dataset name argument, THE editor SHALL display a descriptive error message in the status bar and SHALL NOT modify the document or open any new tab. [EARS: PC-SUBMIT through PC-COMPARE]
