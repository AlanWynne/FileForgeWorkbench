# Requirements Document

## Introduction

This feature specifies the **Line Commands** subsystem for FileForgeWorkbench — the set of prefix-area commands that operate on individual lines or blocks of lines in the ISPF/PDF editing model. Line commands are entered in the prefix area adjacent to document lines and provide rapid, keystroke-efficient operations for deletion, insertion, duplication, copying, moving, exclusion, tagging, and shifting.

The line commands subsystem is **GUI-independent** — it defines the command parsing, validation, pairing, pending-state management, and execution semantics without any rendering dependency. The prefix-area visual representation is the responsibility of the UI layer; this spec covers the underlying command engine behaviour.

This specification is derived from FileForgeEditor core-command-semantics Requirements 22–35, adapted to the workbench architecture:

- **Delete commands** (D, Dn, DD) — remove lines, undoable
- **Insert commands** (I, In) — add blank lines, undoable
- **Repeat commands** (R, Rn, RR) — duplicate lines, undoable
- **Copy markers** (C, CC) — mark source for copy, pending until target supplied
- **Move markers** (M, MM) — mark source for move, pending until target supplied
- **After/Before targets** (A, B) — mark insertion point, resolve pending source
- **Exclude commands** (X, Xn, XX) — hide lines from viewport, NOT undoable (session state)
- **Tag/Untag commands** (T, TT, U, UU) — mark/unmark lines for scoped operations, NOT undoable
- **Shift Right** (>, >n, >>) — indent content, undoable
- **Shift Left** (<, <n, <<) — de-indent content, undoable
- **Bounds-Aware Shift** (), )), (, (( — shift within column bounds, undoable
- **Block command pairing** — validation, normalization, pending retention
- **Compatibility validation** — interaction with primary commands
- **Pending command state management** — storage, display, clearing

**Source references:**
- **[FFE-CMD-22]** = FFE core-command-semantics Requirement 22: Line Commands — Delete
- **[FFE-CMD-23]** = FFE core-command-semantics Requirement 23: Line Commands — Insert
- **[FFE-CMD-24]** = FFE core-command-semantics Requirement 24: Line Commands — Repeat
- **[FFE-CMD-25]** = FFE core-command-semantics Requirement 25: Line Commands — Copy Markers
- **[FFE-CMD-26]** = FFE core-command-semantics Requirement 26: Line Commands — Move Markers
- **[FFE-CMD-27]** = FFE core-command-semantics Requirement 27: Line Commands — After/Before Targets
- **[FFE-CMD-28]** = FFE core-command-semantics Requirement 28: Line Commands — Exclude
- **[FFE-CMD-29]** = FFE core-command-semantics Requirement 29: Line Commands — Tag/Untag
- **[FFE-CMD-30]** = FFE core-command-semantics Requirement 30: Line Commands — Shift Right
- **[FFE-CMD-31]** = FFE core-command-semantics Requirement 31: Line Commands — Shift Left
- **[FFE-CMD-32]** = FFE core-command-semantics Requirement 32: Line Commands — Bounds-Aware Shift
- **[FFE-CMD-33]** = FFE core-command-semantics Requirement 33: Block Command Pairing
- **[FFE-CMD-34]** = FFE core-command-semantics Requirement 34: Command Compatibility Validation
- **[FFE-CMD-35]** = FFE core-command-semantics Requirement 35: Pending Command State Management
- **[WB]** = Workbench Platform Architecture Brief

## Glossary

- **LineCommand**: A command entered in the prefix area next to a document line, parsed by the line command parser into a kind and optional count. [FFE-CMD-22]
- **BlockCommand**: A paired line command (e.g., `CC...CC`, `DD...DD`, `>>...>>`) that marks a range of lines as the operation scope. Requires exactly two matching markers. [FFE-CMD-33]
- **PendingCommand**: A line command or block marker that has been entered but not yet resolved into an executed operation. Stored in DocumentSession until resolved or cleared. [FFE-CMD-35]
- **ImmediateCommand**: A line command that can execute without a partner or target (D, Dn, I, In, R, Rn, X, Xn, T, U, >, >n, <, <n, ), (). Resolves on the next command cycle without requiring a primary command. [FFE-CMD-34]
- **SourceMarker**: A C, CC, M, or MM line command that marks lines as the source of a copy or move operation. Requires a target (A or B) to resolve. [FFE-CMD-25, FFE-CMD-26]
- **TargetMarker**: An A or B line command that designates the insertion point for a pending copy or move operation. [FFE-CMD-27]
- **PrefixArea**: The non-editable column(s) to the left of each document line where line commands are entered and pending command indicators are displayed. [FFE-CMD-35]
- **ShiftWidth**: The configurable number of columns by which `>` and `<` commands shift content when no explicit count is provided. Default: 2. [FFE-CMD-30, FFE-CMD-31]
- **Bounds**: The active left and right column numbers set by the BOUNDS/BNDS command. When set, bounds-aware shift commands operate within this range only. [FFE-CMD-32]
- **DocumentSession**: The editor session struct that holds line storage, visibility state, tag state, pending prefix commands, and transient session state. [FFE-CMD-35]
- **Transaction**: The unit of undo/redo. Undoable line commands wrap their mutations in a single transaction. [FFE-CMD-22]
- **SessionState**: Transient editor state (excluded lines, tags) that is in-memory only and not persisted to disk. Operations on session state bypass the undo stack. [FFE-CMD-28, FFE-CMD-29]
- **CommandCompatibilityMatrix**: The set of rules defining which primary commands can coexist with which line commands, and which combinations are errors. [FFE-CMD-34]

---

## Requirements

### Requirement 1: Delete Line Commands (D, Dn, DD)

**User Story:** As a developer, I want to delete individual lines, counted ranges, or blocks using prefix-area commands so that I can remove content with minimal keystrokes.

**Source:** [FFE-CMD-22]

#### Acceptance Criteria

1. WHEN `D` is entered in the prefix area of a line, THE system SHALL delete that single line when the command is resolved. [FFE-CMD-22]
2. WHEN `Dn` is entered (where n is a positive integer), THE system SHALL delete n consecutive lines starting at the prefixed line. [FFE-CMD-22]
3. WHEN two `DD` markers are entered on different lines, THE system SHALL delete all lines from the first DD to the second DD inclusive. [FFE-CMD-22]
4. WHEN a delete operation (D, Dn, or DD) completes successfully, THE system SHALL record it as a single undoable Transaction. [FFE-CMD-22]
5. IF only one `DD` marker exists with no matching pair, THEN THE system SHALL retain the DD marker as a PendingCommand and display "DD requires a matching pair". [FFE-CMD-22]
6. WHEN `D` or `Dn` is entered with no primary command pending, THE system SHALL execute the deletion immediately on the next command cycle (immediate command). [FFE-CMD-22, WB]

---

### Requirement 2: Insert Line Commands (I, In)

**User Story:** As a developer, I want to insert blank lines after a specific document line so that I can create space for new content without leaving the prefix area.

**Source:** [FFE-CMD-23]

#### Acceptance Criteria

1. WHEN `I` is entered in the prefix area, THE system SHALL insert one blank line immediately after the prefixed line. [FFE-CMD-23]
2. WHEN `In` is entered (where n is a positive integer), THE system SHALL insert n blank lines immediately after the prefixed line. [FFE-CMD-23]
3. WHEN an insert operation completes successfully, THE system SHALL record it as a single undoable Transaction. [FFE-CMD-23]
4. WHEN `I` or `In` is entered with no primary command pending, THE system SHALL execute the insertion immediately on the next command cycle (immediate command). [FFE-CMD-23, WB]

---

### Requirement 3: Repeat Line Commands (R, Rn, RR)

**User Story:** As a developer, I want to duplicate one or more lines in place so that I can quickly create repeated content without copy-paste workflows.

**Source:** [FFE-CMD-24]

#### Acceptance Criteria

1. WHEN `R` is entered in the prefix area, THE system SHALL insert one duplicate of the prefixed line immediately after it. [FFE-CMD-24]
2. WHEN `Rn` is entered (where n is a positive integer), THE system SHALL insert n duplicates of the prefixed line immediately after it. [FFE-CMD-24]
3. WHEN two `RR` markers are entered on different lines, THE system SHALL duplicate the entire block defined by the two RR markers and insert the copy immediately after the second RR line. [FFE-CMD-24]
4. IF only one `RR` marker exists with no matching pair, THEN THE system SHALL retain the RR marker as a PendingCommand and display "RR requires a matching pair". [FFE-CMD-24]
5. WHEN a repeat operation (R, Rn, or RR) completes successfully, THE system SHALL record it as a single undoable Transaction. [FFE-CMD-24]
6. WHEN `R` or `Rn` is entered with no primary command pending, THE system SHALL execute the duplication immediately on the next command cycle (immediate command). [FFE-CMD-24, WB]

---

### Requirement 4: Copy Markers (C, CC)

**User Story:** As a developer, I want to mark one or more lines as a copy source in the prefix area so that I can then place them at a target position using an A or B marker.

**Source:** [FFE-CMD-25]

#### Acceptance Criteria

1. WHEN `C` is entered in the prefix area, THE system SHALL mark that line as a single-line copy source and store it as a PendingCommand. [FFE-CMD-25]
2. WHEN two `CC` markers are entered on different lines, THE system SHALL mark all lines from the first CC to the second CC inclusive as a copy source block. [FFE-CMD-25]
3. WHEN a `C` or `CC` source marker is pending and an `A` or `B` target is supplied, THE system SHALL execute the in-document copy (inserting copied lines at the target position) and clear all consumed markers. [FFE-CMD-25]
4. WHEN a `C` or `CC` marker is pending and no target has been entered, THE system SHALL retain the PendingCommand and display "Waiting for A or B target". [FFE-CMD-25]
5. IF only one `CC` marker exists with no matching pair, THEN THE system SHALL retain the CC marker as a PendingCommand and display "CC requires a matching pair". [FFE-CMD-25]
6. WHEN a copy operation completes successfully, THE system SHALL record it as a single undoable Transaction. [FFE-CMD-25, WB]

---

### Requirement 5: Move Markers (M, MM)

**User Story:** As a developer, I want to mark one or more lines as a move source in the prefix area so that I can relocate them to a target position using an A or B marker.

**Source:** [FFE-CMD-26]

#### Acceptance Criteria

1. WHEN `M` is entered in the prefix area, THE system SHALL mark that line as a single-line move source and store it as a PendingCommand. [FFE-CMD-26]
2. WHEN two `MM` markers are entered on different lines, THE system SHALL mark all lines from the first MM to the second MM inclusive as a move source block. [FFE-CMD-26]
3. WHEN an `M` or `MM` source marker is pending and an `A` or `B` target is supplied, THE system SHALL execute the move (removing source lines and inserting them at the target position) and clear all consumed markers. [FFE-CMD-26]
4. IF the target line (A or B) falls inside the MM source block, THEN THE system SHALL display "Target cannot be inside the source block" and SHALL NOT modify the document. [FFE-CMD-26]
5. WHEN an `M` or `MM` marker is pending and no target has been entered, THE system SHALL retain the PendingCommand and display "Waiting for A or B target". [FFE-CMD-26]
6. IF only one `MM` marker exists with no matching pair, THEN THE system SHALL retain the MM marker as a PendingCommand and display "MM requires a matching pair". [FFE-CMD-26]
7. WHEN a move operation completes successfully, THE system SHALL record it as a single undoable Transaction. [FFE-CMD-26, WB]

---

### Requirement 6: After/Before Target Markers (A, B)

**User Story:** As a developer, I want to mark an insertion point in the prefix area so that pending copy or move operations know exactly where to place their content.

**Source:** [FFE-CMD-27]

#### Acceptance Criteria

1. WHEN `A` is entered in the prefix area, THE system SHALL designate that line as an after-insertion target (content will be placed after this line). [FFE-CMD-27]
2. WHEN `B` is entered in the prefix area, THE system SHALL designate that line as a before-insertion target (content will be placed before this line). [FFE-CMD-27]
3. WHEN an `A` or `B` target is entered and a compatible source marker (C, CC, M, or MM) is already pending, THE system SHALL immediately resolve and execute the copy or move operation. [FFE-CMD-27]
4. WHEN an `A` or `B` target is entered and no compatible source marker is pending, THE system SHALL store the target as a PendingCommand and await a source marker or a `COPY`/`MOVE` primary command. [FFE-CMD-27]
5. IF more than one `A` or more than one `B` target is pending for a single source operation, THEN THE system SHALL display "Only one target marker is permitted per operation" and SHALL NOT execute the copy or move. [FFE-CMD-27]

---

### Requirement 7: Exclude Line Commands (X, Xn, XX)

**User Story:** As a developer, I want to exclude individual lines or blocks from the viewport using line commands so that I can quickly hide lines without switching to the primary command line.

**Source:** [FFE-CMD-28]

#### Acceptance Criteria

1. WHEN `X` is entered in the prefix area of a line, THE system SHALL set the `excluded` flag on that line, hiding it from the viewport. [FFE-CMD-28]
2. WHEN `Xn` is entered (where n is a positive integer), THE system SHALL set the `excluded` flag on n consecutive lines starting at the prefixed line. [FFE-CMD-28]
3. WHEN two `XX` markers are entered on different lines, THE system SHALL set the `excluded` flag on all lines from the first XX to the second XX inclusive. [FFE-CMD-28]
4. IF only one `XX` marker exists with no matching pair, THEN THE system SHALL retain the XX marker as a PendingCommand and display "XX requires a matching pair". [FFE-CMD-28]
5. WHEN an exclude line command completes, THE system SHALL NOT record it as an undoable Transaction — excluded state is SessionState only and bypasses the undo stack. [FFE-CMD-28]
6. WHEN `X` or `Xn` is entered with no primary command pending, THE system SHALL execute the exclusion immediately on the next command cycle (immediate command). [FFE-CMD-28, WB]

---

### Requirement 8: Tag and Untag Line Commands (T, TT, U, UU)

**User Story:** As a developer, I want to tag and untag individual lines or blocks using line commands so that I can mark sets of lines for subsequent scope-qualified operations (FIND TAGGED, CHANGE TAGGED, DELETE TAGGED, etc.).

**Source:** [FFE-CMD-29]

#### Acceptance Criteria

1. WHEN `T` is entered in the prefix area, THE system SHALL set the `tagged` flag on that line. [FFE-CMD-29]
2. WHEN two `TT` markers are entered on different lines, THE system SHALL set the `tagged` flag on all lines from the first TT to the second TT inclusive. [FFE-CMD-29]
3. WHEN `U` is entered in the prefix area, THE system SHALL clear the `tagged` flag on that line. [FFE-CMD-29]
4. WHEN two `UU` markers are entered on different lines, THE system SHALL clear the `tagged` flag on all lines from the first UU to the second UU inclusive. [FFE-CMD-29]
5. IF only one `TT` marker exists with no matching pair, THEN THE system SHALL retain the TT marker as a PendingCommand and display "TT requires a matching pair". [FFE-CMD-29]
6. IF only one `UU` marker exists with no matching pair, THEN THE system SHALL retain the UU marker as a PendingCommand and display "UU requires a matching pair". [FFE-CMD-29]
7. WHEN a tag or untag operation completes, THE system SHALL NOT record it as an undoable Transaction — tag state is SessionState only and bypasses the undo stack. [FFE-CMD-29]
8. WHEN `T` or `U` is entered with no primary command pending, THE system SHALL execute the tag/untag immediately on the next command cycle (immediate command). [FFE-CMD-29, WB]

---

### Requirement 9: Shift Right Line Commands (>, >n, >>)

**User Story:** As a developer, I want to shift line content to the right by a configurable number of columns so that I can quickly indent code or align data using the prefix area.

**Source:** [FFE-CMD-30]

#### Acceptance Criteria

1. WHEN `>` is entered in the prefix area, THE system SHALL shift the content of that line right by the configured default ShiftWidth. [FFE-CMD-30]
2. WHEN `>n` is entered (where n is a positive integer), THE system SHALL shift that line's content right by n columns. [FFE-CMD-30]
3. WHEN two `>>` markers are entered on different lines, THE system SHALL shift all lines in the block right by the configured default ShiftWidth. [FFE-CMD-30]
4. WHEN a right-shift operation (>, >n, or >>) completes successfully, THE system SHALL record it as a single undoable Transaction. [FFE-CMD-30]
5. IF only one `>>` marker exists with no matching pair, THEN THE system SHALL retain the >> marker as a PendingCommand and display ">> requires a matching pair". [FFE-CMD-30]
6. WHEN `>` or `>n` is entered with no primary command pending, THE system SHALL execute the shift immediately on the next command cycle (immediate command). [FFE-CMD-30, WB]
7. THE default ShiftWidth SHALL be configurable via the configuration system, with a default value of 2 columns. [FFE-CMD-30, WB]

---

### Requirement 10: Shift Left Line Commands (<, <n, <<)

**User Story:** As a developer, I want to shift line content to the left by a configurable number of columns so that I can quickly de-indent code or remove leading whitespace using the prefix area.

**Source:** [FFE-CMD-31]

#### Acceptance Criteria

1. WHEN `<` is entered in the prefix area, THE system SHALL shift the content of that line left by the configured default ShiftWidth. [FFE-CMD-31]
2. WHEN `<n` is entered (where n is a positive integer), THE system SHALL shift that line's content left by n columns. [FFE-CMD-31]
3. WHEN two `<<` markers are entered on different lines, THE system SHALL shift all lines in the block left by the configured default ShiftWidth. [FFE-CMD-31]
4. WHEN a left-shift operation (<, <n, or <<) completes successfully, THE system SHALL record it as a single undoable Transaction. [FFE-CMD-31]
5. IF only one `<<` marker exists with no matching pair, THEN THE system SHALL retain the << marker as a PendingCommand and display "<< requires a matching pair". [FFE-CMD-31]
6. WHEN `<` or `<n` is entered with no primary command pending, THE system SHALL execute the shift immediately on the next command cycle (immediate command). [FFE-CMD-31, WB]
7. THE default ShiftWidth SHALL be configurable via the configuration system, with a default value of 2 columns. [FFE-CMD-31, WB]
8. WHEN a left-shift would remove non-whitespace characters, THE system SHALL truncate from the left only up to the first non-whitespace character, preventing data loss. [FFE-CMD-31, WB]

---

### Requirement 11: Bounds-Aware Shift Line Commands (), )), (, ((

**User Story:** As a developer working with fixed-width records, I want to shift line content right or left within the active column bounds so that characters outside the bounds are preserved exactly.

**Source:** [FFE-CMD-32]

#### Acceptance Criteria

1. WHEN `)` is entered in the prefix area and active Bounds are set, THE system SHALL shift the content within the active Bounds one position to the right, preserving characters outside the Bounds unchanged. [FFE-CMD-32]
2. WHEN two `))` markers are entered and active Bounds are set, THE system SHALL apply the bounds-aware right shift to every line in the block. [FFE-CMD-32]
3. WHEN `(` is entered in the prefix area and active Bounds are set, THE system SHALL shift the content within the active Bounds one position to the left, preserving characters outside the Bounds unchanged. [FFE-CMD-32]
4. WHEN two `((` markers are entered and active Bounds are set, THE system SHALL apply the bounds-aware left shift to every line in the block. [FFE-CMD-32]
5. IF active Bounds are not set when `)`, `))`, `(`, or `((` is entered, THEN THE system SHALL display "Bounds-aware shift requires active BOUNDS" and SHALL NOT modify the document. [FFE-CMD-32]
6. WHEN a bounds-aware shift operation completes successfully, THE system SHALL record it as a single undoable Transaction. [FFE-CMD-32]
7. IF only one `))` marker exists with no matching pair, THEN THE system SHALL retain the )) marker as a PendingCommand and display ")) requires a matching pair". [FFE-CMD-32]
8. IF only one `((` marker exists with no matching pair, THEN THE system SHALL retain the (( marker as a PendingCommand and display "(( requires a matching pair". [FFE-CMD-32]

---

### Requirement 12: Block Command Pairing

**User Story:** As a developer, I want block line commands to be validated as complete pairs before execution so that I never accidentally operate on an unintended range due to a missing or mismatched marker.

**Source:** [FFE-CMD-33]

#### Acceptance Criteria

1. THE system SHALL treat the following as block commands requiring exactly two matching markers: `CC`, `MM`, `DD`, `RR`, `XX`, `TT`, `UU`, `>>`, `<<`, `))`, `((`. [FFE-CMD-33]
2. WHEN a block command has exactly two matching markers, THE system SHALL normalize the range so that `start_line = min(marker1_line, marker2_line)` and `end_line = max(marker1_line, marker2_line)` regardless of the order they were entered. [FFE-CMD-33]
3. WHEN a block command has only one marker and no compatible second marker is present, THE system SHALL retain the single marker as a PendingCommand and display the appropriate "requires a matching pair" message. [FFE-CMD-33]
4. WHEN more than two markers of the same block command type are present, THE system SHALL display "Only two <CMD> markers are permitted" and SHALL NOT execute the operation. [FFE-CMD-33]
5. WHEN two block command markers of different types have overlapping line ranges, THE system SHALL display a compatibility error and SHALL NOT execute either operation. [FFE-CMD-33]
6. WHEN a block command pair executes successfully, THE system SHALL clear both markers from the PendingCommands list. [FFE-CMD-33]
7. WHEN a block command pair fails validation, THE system SHALL retain the markers so the user can correct them. [FFE-CMD-33]

---

### Requirement 13: Command Compatibility Validation

**User Story:** As a developer, I want the editor to reject incompatible combinations of primary commands and line commands before execution so that I never get unexpected results from a mistyped command combination.

**Source:** [FFE-CMD-34]

#### Acceptance Criteria

1. THE system SHALL implement a CommandCompatibilityMatrix defining which primary commands are compatible with which pending line commands. [FFE-CMD-34]
2. WHEN a primary command and pending line commands are incompatible according to the matrix, THE system SHALL display a compatibility error message and SHALL NOT execute the operation. [FFE-CMD-34]
3. WHEN a primary command is compatible with pending line commands, THE system SHALL use the line commands to resolve the operation scope. [FFE-CMD-34]
4. WHEN the primary command line is blank and only ImmediateCommands (D, Dn, R, Rn, I, In, X, Xn, T, U, >, >n, <, <n, ), () are pending, THE system SHALL execute those commands without requiring a primary command. [FFE-CMD-34]
5. WHEN `COPY` is issued as a primary command with `C`/`CC` source markers and `A`/`B` target markers pending, THE system SHALL resolve and execute the in-document copy. [FFE-CMD-34]
6. WHEN `MOVE` is issued as a primary command with `M`/`MM` source markers and `A`/`B` target markers pending, THE system SHALL resolve and execute the move. [FFE-CMD-34]
7. WHEN `COPY path` is issued while `C`/`CC` source markers are pending, THE system SHALL display "Source line commands cannot be combined with a file path argument" and SHALL NOT execute the operation. [FFE-CMD-34]

---

### Requirement 14: Pending Command State Management

**User Story:** As a developer, I want the editor to maintain a clear visual record of unresolved line commands so that I can see at a glance which operations are waiting to be completed.

**Source:** [FFE-CMD-35]

#### Acceptance Criteria

1. THE system SHALL store all unresolved PendingCommands in DocumentSession and expose them via a `pending_prefix_commands()` accessor. [FFE-CMD-35]
2. WHEN a PendingCommand is successfully resolved and executed, THE system SHALL remove it from the pending commands list. [FFE-CMD-35]
3. WHEN a command execution cycle fails validation, THE system SHALL retain the PendingCommands that were involved, enabling the user to correct and re-submit. [FFE-CMD-35]
4. THE system SHALL provide visual indication of every line that has a PendingCommand — the prefix area SHALL display the pending command text for that line. [FFE-CMD-35]
5. WHEN `RESET COMMANDS` or `RESET ALL` is issued, THE system SHALL clear all PendingCommands regardless of their state or type. [FFE-CMD-35]
6. WHEN an invalid line command string is entered in the prefix area, THE system SHALL retain the invalid text in the prefix area so the user can correct it, and SHALL display an error describing the unrecognised command. [FFE-CMD-35]
7. THE pending command store SHALL support querying by command type (e.g., all pending source markers, all pending target markers) to enable resolution logic and compatibility checking. [FFE-CMD-35, WB]
8. ALL line command operations SHALL be dispatched through the workbench command framework — line commands SHALL NOT bypass the command dispatch path. [WB]

---

## Cross-References

- **`command-semantics`**: The line commands subsystem integrates with the primary command execution pipeline. Line commands are collected during the "collect line commands" step of the command execution cycle defined in command-semantics. Line command parsing (the line command parser) is shared infrastructure between both specs. [FFE-CMD-34]
- **`undo-redo-transactions`**: Undoable line commands (D, DD, I, R, RR, C/CC+A/B, M/MM+A/B, >, >>, <, <<, ), )), (, (() wrap their mutations in a single undo Transaction. The transaction system is authoritative for coalescing, recovery, and redo semantics. [FFE-CMD-22]
- **`exclude-show-filter`**: The X/Xn/XX line commands set the `excluded` flag that controls line visibility in the viewport. The exclude-show-filter spec is authoritative for SHOW/INCLUDE restoration, placeholder rendering, and RESET EXCLUDED behaviour. [FFE-CMD-28]
- **`document-model`**: Line commands operate on DocumentLines within the document model — deletions remove lines, insertions add lines, shifts modify line content. The document model provides the mutation primitives. [FFE-CMD-22]
- **`navigation-commands`**: The BOUNDS/BNDS command (defined in navigation-commands) establishes the active column bounds that bounds-aware shift commands (), )), (, (( depend on. [FFE-CMD-32]
- **`configuration-system`**: The default ShiftWidth and `invalid_line_command_policy` configuration keys are managed by the configuration system. [FFE-CMD-30, FFE-CMD-31]

### Requirement 15: Additional ISPF Line Commands (O, W, F, L, ], S)

**User Story:** As a developer using ISPF-style editing, I want the full set of ISPF line commands including overlay, clipboard copy, first/last excluded, label assignment, single-column shift right, and show-excluded so that the workbench matches the ISPF line command repertoire.

**Source:** EARS integration Phase BX (coverage-classification.md B03)

#### Acceptance Criteria

1. WHEN `O` is entered in the prefix area of a line and a pending copy source (C or CC) exists, THE system SHALL overlay the target line(s) with the source content, replacing characters only where the source is non-blank. [LC-O]
2. WHEN `On` is entered (where n is a positive integer) and a pending copy source exists, THE system SHALL overlay n consecutive lines starting at the prefixed line with the source content. [LC-O]
3. WHEN `W` is entered in the prefix area of a line, THE system SHALL copy that line's content to the system clipboard. [LC-W]
4. WHEN `WW` markers are entered on two different lines, THE system SHALL copy all lines from the first WW to the second WW inclusive to the system clipboard. [LC-W]
5. WHEN `F` is entered in the prefix area of an excluded-block placeholder, THE system SHALL show (un-exclude) only the first line of that excluded block. [LC-F]
6. WHEN `L` is entered in the prefix area of an excluded-block placeholder, THE system SHALL show (un-exclude) only the last line of that excluded block. [LC-L]
7. WHEN `]` is entered in the prefix area of a line, THE system SHALL shift that line's content right by exactly one column (equivalent to `>1`). [LC-bracket-right]
8. WHEN `]]` markers are entered on two different lines, THE system SHALL shift all lines in the block right by exactly one column. [LC-bracket-right]
9. WHEN `S` is entered in the prefix area of an excluded-block placeholder, THE system SHALL show (un-exclude) that single excluded line or the first line of the excluded block, equivalent to the SHOW primary command scoped to that line. [LC-S]
10. WHEN an overlay operation (O or On) completes successfully, THE system SHALL record it as a single undoable Transaction. [LC-O]
11. WHEN a clipboard copy operation (W or WW) completes, THE system SHALL NOT record it as an undoable Transaction -- clipboard state is external to the document. [LC-W]
12. WHEN `F`, `L`, or `S` completes, THE system SHALL NOT record it as an undoable Transaction -- excluded state is SessionState only and bypasses the undo stack. [LC-F, LC-L, LC-S]
