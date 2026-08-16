# Requirements Document

## Introduction

This feature specifies the **Exclude/Show Filter** subsystem for FileForgeWorkbench — the `ff-exclude-show-filter` crate. This subsystem implements ISPF-style line visibility management, allowing users to hide and reveal document lines without modifying document content. It provides the logical visibility state engine behind the EXCLUDE, SHOW, RESET primary commands and the X/Xn/XX line commands.

The exclude-show-filter is a **GUI-independent** logical layer. It manages which lines are excluded (hidden) and drives the `display-line-mapping` subsystem to update the physical display-line count. It does not render anything directly — rendering of placeholder lines ("-- N line(s) excluded --") is delegated to the viewport rendering layer, while this crate provides the data model and placeholder text generation.

Key architectural properties:
- EXCLUDE/SHOW/RESET operations are **non-undoable** — they operate on transient session state, not document content.
- All commands are dispatched through the workbench command framework.
- Line exclusion is **flat** (not hierarchical), distinct from code folding which uses nested fold levels.
- The exclusion state coexists with code-folding visibility in the `display-line-mapping` layer, which provides the underlying visibility storage.

This specification merges requirements from two primary sources:

- **FileForgeEditor core-command-semantics** (Requirements 7–9, 28): EXCLUDE/X command with literal text match, ALL, REGEX, TAGGED, range modifiers; SHOW/INCLUDE command with ALL, EXCLUDED, NONEXCLUDED, text, regex modifiers; RESET command with EXCLUDED, TAGS, COMMANDS, ALL variants; X/Xn/XX line commands for per-line and block exclusion.
- **Scintilla ContractionState** (IContractionState interface): Per-line visibility tracking via `SetVisible`/`GetVisible`, `HiddenLines` query, range-based visibility mutation, display-line integration where hidden lines contribute zero display lines, and `ShowAll` for bulk reset.

**Source references:**
- **[FFE-CMD-7]** = FileForgeEditor core-command-semantics Requirement 7: EXCLUDE/X Command
- **[FFE-CMD-8]** = FileForgeEditor core-command-semantics Requirement 8: SHOW/INCLUDE Command
- **[FFE-CMD-9]** = FileForgeEditor core-command-semantics Requirement 9: RESET Command
- **[FFE-CMD-28]** = FileForgeEditor core-command-semantics Requirement 28: Line Commands — Exclude (X, Xn, XX)
- **[SCI-CS-12.1]** = Scintilla ContractionState / IContractionState interface — visibility tracking, SetVisible, GetVisible, HiddenLines, ShowAll, display-line integration
- **[WB]** = Workbench Platform Architecture Brief (GUI-independent core, command-driven architecture, non-destructive session state)

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `display-line-mapping` | **Dependency** | Provides the underlying per-line visibility storage (`set_visible`, `get_visible`, `hidden_lines`, `show_all`). The exclude-show-filter drives visibility changes through this layer. |
| `command-semantics` | **Integration** | All EXCLUDE/SHOW/RESET primary commands are registered in the command framework and dispatched through the standard command execution pipeline. |
| `line-commands` | **Integration** | The X, Xn, XX line commands are parsed and resolved by the line-command subsystem, which delegates the actual exclusion operation to this crate. |
| `find-and-replace` | **Integration** | FIND/CHANGE with EXCLUDED/VISIBLE modifiers use the exclusion state from this subsystem to determine search scope. EXCLUDE-ALL + FIND-ALL is a core filtering workflow. |
| `document-model` | **Dependency** | Provides document line content for text-matching operations (literal and regex) used by EXCLUDE and SHOW commands. |
| `viewport-and-scrolling` | **Consumer** | Renders placeholder lines for contiguous excluded blocks based on data provided by this subsystem. |
| `navigation-commands` | **Consumer** | LOCATE and scroll commands interact with visibility state — navigating to an excluded line may auto-show it depending on configuration. |

## Glossary

- **Exclusion_State**: The per-line boolean attribute tracking whether a document line is excluded (hidden) from the viewport display. Part of transient session state — never saved to disk. [FFE-CMD-7, SCI-CS-12.1]
- **Excluded_Line**: A document line whose `excluded` flag is true, causing it to be hidden from the viewport and contribute zero display lines. [FFE-CMD-7, SCI-CS-12.1]
- **Visible_Line**: A document line whose `excluded` flag is false, displayed normally in the viewport. [FFE-CMD-7]
- **Exclusion_Block**: A contiguous range of one or more consecutive excluded lines. Rendered as a single placeholder line in the viewport. [FFE-CMD-7]
- **Placeholder_Line**: A synthetic, non-editable display line rendered in the viewport to represent a contiguous Exclusion_Block. Shows the count of hidden lines (e.g., "-- 5 line(s) excluded --"). [FFE-CMD-7]
- **Session_State**: Transient, in-memory editor state (excluded lines, tags, bounds, pending commands) that is not persisted to disk and is not undoable. [FFE-CMD-7, FFE-CMD-9]
- **ISPF_Exclusion**: The ISPF/PDF-inspired flat (non-hierarchical) line-hiding mechanism driven by EXCLUDE/SHOW/RESET commands. Distinct from code folding in that it operates on flat line ranges without fold levels. [FFE-CMD-7, SCI-CS-12.1]
- **Scope_Modifier**: A command argument (ALL, VISIBLE, EXCLUDED, TAGGED, NONTAGGED, NONEXCLUDED) that restricts which lines an operation applies to. [FFE-CMD-7, FFE-CMD-8]
- **Display_Line_Mapping**: The subsystem that tracks the relationship between document lines and display lines, including visibility. The exclude-show-filter calls `set_visible` on this layer. [SCI-CS-12.1]
- **Primary_Command**: A command entered on the command line and dispatched by the command engine. EXCLUDE, SHOW, RESET, and their aliases are primary commands. [FFE-CMD-7, FFE-CMD-8, FFE-CMD-9]
- **Line_Command**: A command entered in the prefix area next to a document line. X, Xn, XX are exclusion-related line commands. [FFE-CMD-28]
- **Block_Command**: A paired line command (XX...XX) that marks a range of lines for exclusion. [FFE-CMD-28]

---

## Requirements

---

### Requirement 1: Exclusion State Model

**User Story:** As the exclude-show-filter engine, I need a logical model for tracking which lines are excluded so that exclusion state can be queried by other subsystems (find, show, reset) and driven to the display-line-mapping for viewport hiding.

**Source:** [FFE-CMD-7], [SCI-CS-12.1]

#### Acceptance Criteria

1. THE exclude-show-filter SHALL maintain a logical Exclusion_State per document line, represented as a boolean attribute (`excluded = true` or `excluded = false`), stored in the display-line-mapping layer via the `set_visible` / `get_visible` API. [SCI-CS-12.1]
2. WHEN a line's Exclusion_State is set to `excluded = true`, THE exclude-show-filter SHALL call `display_line_mapping.set_visible(line, line, false)` to hide the line from display output. [SCI-CS-12.1]
3. WHEN a line's Exclusion_State is set to `excluded = false`, THE exclude-show-filter SHALL call `display_line_mapping.set_visible(line, line, true)` to restore the line to display output. [SCI-CS-12.1]
4. THE exclude-show-filter SHALL provide a `is_excluded(doc_line)` method that returns `true` if the given document line is currently excluded, delegating to `display_line_mapping.get_visible(doc_line) == false`. [SCI-CS-12.1]
5. THE exclude-show-filter SHALL provide a `has_excluded_lines()` method that returns `true` if any document line is currently excluded, delegating to `display_line_mapping.hidden_lines()`. [SCI-CS-12.1]
6. THE Exclusion_State SHALL be transient Session_State only: it SHALL NOT be saved to disk, SHALL NOT be stored in the document file, and SHALL NOT be included in undo/redo transactions. [FFE-CMD-7]
7. THE exclude-show-filter SHALL provide an `excluded_line_count()` method returning the total number of currently excluded lines across the entire document. [WB]
8. THE exclude-show-filter SHALL support batch exclusion of contiguous ranges via a single `exclude_range(start_line, end_line)` call that maps to `display_line_mapping.set_visible(start, end, false)` for efficiency. [SCI-CS-12.1]

---

### Requirement 2: EXCLUDE / X Primary Command

**User Story:** As a developer, I want to hide lines from the viewport without deleting them so that I can focus on relevant content and use the EXCLUDE ALL + FIND ALL pattern to filter large files.

**Source:** [FFE-CMD-7]

#### Acceptance Criteria

1. WHEN `EXCLUDE 'text'` is issued, THE exclude-show-filter SHALL set the excluded flag to true on every currently visible line whose content contains the given literal text (case-insensitive by default, or case-sensitive per configuration). [FFE-CMD-7]
2. WHEN `EXCLUDE 'text' ALL` is issued, THE exclude-show-filter SHALL set the excluded flag on every line matching the text regardless of current visibility state (including already-excluded lines, which remain excluded). [FFE-CMD-7]
3. WHEN `EXCLUDE REGEX 'pattern'` is issued, THE exclude-show-filter SHALL interpret the argument as a regular expression and set the excluded flag on every visible line whose content matches the pattern. [FFE-CMD-7]
4. WHEN `EXCLUDE ALL` is issued, THE exclude-show-filter SHALL set the excluded flag on every line in the document. [FFE-CMD-7]
5. WHEN `EXCLUDE TAGGED` is issued, THE exclude-show-filter SHALL set the excluded flag on every line whose `tagged` flag is true. [FFE-CMD-7]
6. WHEN `EXCLUDE n m` is issued with two positive integer arguments, THE exclude-show-filter SHALL set the excluded flag on document lines n through m inclusive (1-based line numbers). [FFE-CMD-7]
7. THE command engine SHALL treat `X` as a full alias for `EXCLUDE`, accepting all the same argument forms (`X 'text'`, `X ALL`, `X REGEX 'pattern'`, `X TAGGED`, `X n m`). [FFE-CMD-7]
8. WHEN an EXCLUDE operation completes, THE exclude-show-filter SHALL report the count of newly excluded lines in the status message (e.g., "12 line(s) excluded"). [WB]
9. WHEN an EXCLUDE operation matches zero lines, THE exclude-show-filter SHALL display "No lines matched" in the status area and SHALL NOT modify exclusion state. [WB]
10. THE EXCLUDE command SHALL NOT record the operation as an undoable Transaction; exclusion is Session_State only. [FFE-CMD-7]

---

### Requirement 3: SHOW / INCLUDE Primary Command

**User Story:** As a developer, I want to make excluded lines visible again by revealing all or a selected subset so that I can restore the full document view or selectively expose relevant hidden lines.

**Source:** [FFE-CMD-8]

#### Acceptance Criteria

1. WHEN `SHOW ALL` is issued, THE exclude-show-filter SHALL clear the excluded flag on every line in the document (all lines become visible). [FFE-CMD-8]
2. WHEN `SHOW EXCLUDED` is issued, THE exclude-show-filter SHALL clear the excluded flag on every currently excluded line. [FFE-CMD-8]
3. WHEN `SHOW NONEXCLUDED` is issued, THE exclude-show-filter SHALL leave excluded lines unchanged and SHALL display a confirmation message "No excluded lines were modified" (this is effectively a no-op that confirms current state). [FFE-CMD-8]
4. WHEN `SHOW 'text'` is issued, THE exclude-show-filter SHALL clear the excluded flag on every currently excluded line whose content contains the given literal text. [FFE-CMD-8]
5. WHEN `SHOW REGEX 'pattern'` is issued, THE exclude-show-filter SHALL clear the excluded flag on every currently excluded line whose content matches the regular expression. [FFE-CMD-8]
6. THE command engine SHALL treat `INCLUDE` as a full alias for `SHOW`, accepting all the same argument forms (`INCLUDE 'text'`, `INCLUDE REGEX 'pattern'`, `INCLUDE ALL`, `INCLUDE EXCLUDED`). [FFE-CMD-8]
7. WHEN a SHOW/INCLUDE operation completes, THE exclude-show-filter SHALL report the count of lines made visible in the status message (e.g., "8 line(s) shown"). [WB]
8. WHEN a SHOW operation matches zero excluded lines, THE exclude-show-filter SHALL display "No excluded lines matched" in the status area and SHALL NOT modify exclusion state. [WB]
9. THE SHOW/INCLUDE command SHALL NOT record the operation as an undoable Transaction; visibility state is Session_State only. [FFE-CMD-8]

---

### Requirement 4: RESET Command (Exclusion Aspects)

**User Story:** As a developer, I want a single command to clear accumulated exclusion state so that I can return the editor to a clean view without closing and reopening the file.

**Source:** [FFE-CMD-9]

#### Acceptance Criteria

1. WHEN `RESET` is issued with no arguments, THE exclude-show-filter SHALL clear all excluded-line visibility state (all lines become visible), clear temporary find/show filters, and delegate to other subsystems to clear pending commands. [FFE-CMD-9]
2. WHEN `RESET EXCLUDED` is issued, THE exclude-show-filter SHALL clear only the excluded flag on all lines, making all lines visible, leaving tag state and pending commands unchanged. [FFE-CMD-9]
3. WHEN `RESET ALL` is issued, THE exclude-show-filter SHALL clear excluded state (all lines visible) as part of the broader RESET ALL operation (which also clears tags and pending commands via their respective subsystems). [FFE-CMD-9]
4. WHEN any RESET variant clears exclusion state, THE exclude-show-filter SHALL call `display_line_mapping.show_all()` or equivalent range-based `set_visible(0, last_line, true)` to restore all lines to visible, returning the display-line-mapping to one-to-one mode where possible. [SCI-CS-12.1]
5. THE RESET command SHALL NOT modify document content, save the file, or remove bookmarks. [FFE-CMD-9]
6. THE RESET command SHALL NOT record the operation as an undoable Transaction; RESET operates on Session_State only. [FFE-CMD-9]
7. WHEN RESET clears exclusion state, THE exclude-show-filter SHALL report the count of lines made visible in the status message (e.g., "RESET: 45 line(s) restored to view"). [WB]

---

### Requirement 5: X / Xn / XX Line Commands

**User Story:** As a developer, I want to exclude individual lines or blocks from the viewport using line commands so that I can quickly hide lines without switching to the command line.

**Source:** [FFE-CMD-28]

#### Acceptance Criteria

1. WHEN `X` is entered in the prefix area of a line, THE exclude-show-filter SHALL set the excluded flag on that single line when the command is resolved. [FFE-CMD-28]
2. WHEN `Xn` is entered (where n is a positive integer), THE exclude-show-filter SHALL set the excluded flag on n consecutive lines starting at the prefixed line. [FFE-CMD-28]
3. WHEN two `XX` markers are entered on different lines, THE exclude-show-filter SHALL set the excluded flag on all lines from the first XX to the second XX inclusive. [FFE-CMD-28]
4. IF only one `XX` marker exists with no matching pair, THEN THE command engine SHALL leave the XX marker visible as a pending command and display "XX requires a matching pair". [FFE-CMD-28]
5. WHEN an exclude line command completes, THE exclude-show-filter SHALL NOT record it as an undoable Transaction; excluded state is Session_State only. [FFE-CMD-28]
6. WHEN the X/Xn/XX line command is resolved (command line is empty or a compatible primary command is submitted), THE exclude-show-filter SHALL execute the exclusion immediately without requiring a primary command. [FFE-CMD-28]
7. THE X/Xn/XX line commands SHALL report the count of excluded lines in the status message upon completion. [WB]

---

### Requirement 6: Placeholder Display Model

**User Story:** As a viewport renderer, I need the exclude-show-filter to provide information about contiguous excluded blocks so that I can render placeholder lines showing how many lines are hidden in each block.

**Source:** [FFE-CMD-7], [SCI-CS-12.1]

#### Acceptance Criteria

1. THE exclude-show-filter SHALL provide a method to enumerate all Exclusion_Blocks in the document, where each block is a contiguous range of excluded lines defined by a start line and end line (inclusive). [FFE-CMD-7]
2. FOR EACH Exclusion_Block, THE exclude-show-filter SHALL provide placeholder text generation in the format `-- N line(s) excluded --` where N is the count of excluded lines in that block. [FFE-CMD-7]
3. THE Placeholder_Line SHALL be a display artifact only: it SHALL NOT be editable as document content, SHALL NOT be saved to disk, and SHALL NOT appear in any command operation's scope. [FFE-CMD-7]
4. THE Placeholder_Line SHALL NOT have a modifiable prefix area — it SHALL display a fixed indicator (e.g., `- - -` or blank) in the prefix column. [FFE-CMD-7]
5. WHEN excluded lines are added or removed adjacent to an existing Exclusion_Block, THE exclude-show-filter SHALL merge or split blocks automatically to maintain the invariant that each Exclusion_Block is maximally contiguous. [SCI-CS-12.1]
6. THE exclude-show-filter SHALL provide a `block_count()` method returning the total number of Exclusion_Blocks currently in the document. [WB]
7. THE exclude-show-filter SHALL provide a `block_at_doc_line(doc_line)` method that, given a document line within an exclusion block, returns the full block range and its placeholder text. [WB]
8. WHEN the document has no excluded lines, THE exclude-show-filter SHALL report zero blocks and no placeholders SHALL be rendered. [SCI-CS-12.1]

---

### Requirement 7: Display-Line Integration

**User Story:** As the viewport and scrollbar subsystems, I need exclusion state to be correctly reflected in the display-line-mapping so that hidden lines contribute zero display lines and scrollbar ranges are accurate.

**Source:** [SCI-CS-12.1], [FFE-CMD-7]

#### Acceptance Criteria

1. WHEN lines are excluded, THE display-line-mapping SHALL subtract the affected lines' display heights from the total Display_Line_Count, causing those lines to occupy zero display lines. [SCI-CS-12.1]
2. WHEN lines are shown (un-excluded), THE display-line-mapping SHALL add the restored lines' display heights back to the total Display_Line_Count. [SCI-CS-12.1]
3. THE scrollbar range SHALL reflect only visible lines (plus one placeholder per Exclusion_Block if placeholder rendering contributes a display line), ensuring the scrollbar accurately represents the visible content extent. [SCI-CS-12.1]
4. WHEN the user scrolls through the viewport, excluded lines SHALL be skipped entirely — the viewport SHALL jump from the last visible line before a block to the first visible line after the block, with the placeholder rendered at the transition. [FFE-CMD-7]
5. THE exclude-show-filter SHALL emit a change notification (or trigger a display-line-mapping notification) when exclusion state changes, enabling the viewport and scrollbar to synchronize. [SCI-CS-12.1]
6. WHEN an Exclusion_Block placeholder is rendered, IT SHALL occupy exactly one display line in the viewport regardless of how many document lines are hidden in the block. [FFE-CMD-7]
7. THE `doc_from_display(display_line)` mapping SHALL never resolve to an excluded document line — it SHALL always return a visible line. [SCI-CS-12.1]

---

### Requirement 8: Scope Integration with Find and Change

**User Story:** As a developer using the EXCLUDE ALL + FIND/SHOW workflow, I need the find and change operations to respect exclusion state through scope modifiers so that I can filter large files by hiding all lines and then selectively revealing matches.

**Source:** [FFE-CMD-7], [FFE-CMD-8]

#### Acceptance Criteria

1. WHEN `FIND 'text' EXCLUDED` is issued, THE find subsystem SHALL restrict the search scope to lines whose excluded flag is true (searching hidden lines only). [FFE-CMD-7]
2. WHEN `FIND 'text' VISIBLE` is issued, THE find subsystem SHALL restrict the search scope to lines whose excluded flag is false (searching visible lines only). [FFE-CMD-7]
3. WHEN `CHANGE 'old' 'new' EXCLUDED` is issued, THE change subsystem SHALL restrict substitutions to lines whose excluded flag is true. [FFE-CMD-7]
4. WHEN `CHANGE 'old' 'new' VISIBLE` is issued, THE change subsystem SHALL restrict substitutions to lines whose excluded flag is false. [FFE-CMD-7]
5. THE exclude-show-filter SHALL expose a `visible_lines_iter()` method providing an iterator over all currently visible line indices, for use by scoped operations. [WB]
6. THE exclude-show-filter SHALL expose an `excluded_lines_iter()` method providing an iterator over all currently excluded line indices, for use by scoped operations. [WB]
7. THE standard FIND/CHANGE workflow `EXCLUDE ALL` → `FIND 'text' ALL` → `SHOW 'text'` SHALL result in only lines containing 'text' being visible, providing ISPF-style filtering. [FFE-CMD-7, FFE-CMD-8]

---

### Requirement 9: Command Framework Integration

**User Story:** As a workbench user, I want EXCLUDE, SHOW, RESET, and X/Xn/XX to be registered in the command framework so that they are accessible from the command line, keyboard shortcuts, menus, and macros through the standard dispatch path.

**Source:** [WB], [FFE-CMD-7], [FFE-CMD-8], [FFE-CMD-9], [FFE-CMD-28]

#### Acceptance Criteria

1. THE EXCLUDE command (and its X alias) SHALL be registered in the command framework with metadata including command name, aliases, syntax help, and argument schema. [WB]
2. THE SHOW command (and its INCLUDE alias) SHALL be registered in the command framework with metadata including command name, aliases, syntax help, and argument schema. [WB]
3. THE RESET command SHALL be registered in the command framework with metadata for its variants (no-arg, EXCLUDED, TAGS, COMMANDS, ALL). [WB]
4. THE X, Xn, XX line commands SHALL be registered in the line-command parser's recognized command set. [FFE-CMD-28]
5. ALL exclude-show-filter commands SHALL be executable from macros (Lua scripting engine) via the standard command dispatch API. [WB]
6. THE exclude-show-filter commands SHALL be valid in both Edit mode and Browse/View mode (excluding lines is non-destructive and applicable regardless of edit permissions). [WB]
7. THE exclude-show-filter commands SHALL NOT be added to undo history — they SHALL be explicitly marked as non-undoable in their command metadata. [FFE-CMD-7, FFE-CMD-8, FFE-CMD-9]
8. WHEN an EXCLUDE or SHOW command receives invalid arguments (unterminated quote, invalid regex, non-numeric range value), THE command engine SHALL display an error message identifying the problem and SHALL NOT modify exclusion state. [WB]

---

### Requirement 10: Performance and Scalability

**User Story:** As a developer working with large files (100K+ lines), I want EXCLUDE ALL and SHOW ALL to execute in sub-second time and I want the exclusion state model to scale linearly without degrading viewport performance.

**Source:** [SCI-CS-12.1], [WB]

#### Acceptance Criteria

1. THE `EXCLUDE ALL` operation SHALL complete in O(n) time where n is the number of document lines, leveraging the display-line-mapping's range-based `set_visible(0, last_line, false)` for bulk updates. [SCI-CS-12.1]
2. THE `SHOW ALL` / `RESET EXCLUDED` operation SHALL complete in O(1) amortized time by calling `display_line_mapping.show_all()` which resets to one-to-one mode and deallocates per-line tracking structures. [SCI-CS-12.1]
3. TEXT-MATCHING exclusion operations (`EXCLUDE 'text'`, `EXCLUDE REGEX 'pattern'`) SHALL execute in O(n × m) time where n is the number of lines in scope and m is the average line length, without requiring a pre-built full-text index. [WB]
4. THE Exclusion_Block enumeration (for placeholder rendering) SHALL execute in O(k) time where k is the number of blocks (not the total number of excluded lines), using run-length or boundary tracking. [SCI-CS-12.1]
5. FOR documents with 1,000,000+ lines, THE `EXCLUDE ALL` followed by `SHOW 'text'` workflow SHALL complete within 2 seconds on modern hardware, enabling interactive filtering of very large files. [WB]
6. THE memory overhead of exclusion tracking SHALL be O(1) when no lines are excluded (leveraging the display-line-mapping's one-to-one mode) and O(n) in the worst case when per-line tracking is allocated. [SCI-CS-12.1]

