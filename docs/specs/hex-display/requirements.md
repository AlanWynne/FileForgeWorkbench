# Requirements Document

## Introduction

This spec defines the **Hex Display Mode** for FileForgeWorkbench — the `ff-hex-display` crate. It provides a complete hexadecimal viewing and editing subsystem, allowing users to inspect and modify raw byte content of any file directly within the editor. The hex display mode presents a three-pane layout: an offset column, hex byte columns, and an ASCII/text pane, with synchronised cursor movement between panes.

Hex mode is an invaluable diagnostic tool for:

- Identifying non-printable characters, control characters, and embedded nulls in data files
- Diagnosing encoding problems (EBCDIC vs ASCII, BOM characters, unexpected byte sequences)
- Inspecting packed decimal fields and binary data in fixed-width flat files (particularly relevant when FileForge_Mode is active)
- Verifying exact byte content of a record before or after editing
- Locating specific byte sequences using hex search (`FIND X'hexdigits'`)
- Editing raw bytes by overwriting hex digit pairs in the hex pane

Unlike the FFE implementation which was display-only with limited hex-line editing, this workbench spec provides a **full hex editor** with overwrite editing, goto-offset navigation, hex dump export, and tight integration with the find-and-replace engine's `FIND X'...'` capability.

The crate is **GUI-independent** — it manages the hex display model, cursor synchronisation, and editing logic. Rendering is delegated to the UI layer.

### Source References

- **[FFE-HEX]** = FileForgeEditor `hex-display` spec (Requirements 1–6: HEX commands, display layout, FileForge integration, navigation, hex search, session state)
- **[WB]** = Workbench Platform Architecture Brief (GUI independence, command framework integration, VFS-aware file access, plugin architecture)

### Cross-References

- **`document-model`** — Hex mode operates over the document's raw byte buffer via BytePosition addressing
- **`viewport-and-scrolling`** — Hex viewport scrolling is coordinated through the viewport model with a hex-specific row-count calculation
- **`find-and-replace`** — Hex search (`FIND X'...'`) is implemented by the find engine; this spec defines the display integration when matches are found
- **`undo-redo-transactions`** — Hex edits are recorded as standard Edit_Operations and participate in the undo/redo transaction system
- **`command-framework`** — HEX ON/OFF/toggle and GOTO OFFSET commands are registered through the command registry
- **`theme-and-appearance`** — Hex mode uses theme tokens for offset column, hex digits, ASCII pane, separator, modified-byte highlights, and non-printable indicators
- **`encoding-and-characters`** — Multi-byte character display in the ASCII pane and EBCDIC mode interplay
- **`fileforge-integration`** — Hex mode shows field boundaries and packed decimal annotations when FileForge_Mode is active
- **`configuration-system`** — Hex mode settings (bytes-per-row, uppercase/lowercase, auto-activate for binary) are loaded from configuration

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **Hex_Mode** | The editor display state in which content is shown in the three-pane hex layout (offset + hex bytes + ASCII). Activated by the `HEX` command. | [FFE-HEX] |
| **Hex_Pane** | The central column region showing hexadecimal digit pairs for each byte, grouped and spaced for readability. | [WB] |
| **ASCII_Pane** | The right-hand column region showing the printable ASCII representation of each byte (non-printable bytes shown as `.`). | [WB] |
| **Offset_Column** | The left-hand column showing the file offset (in hex) of the first byte on each row. | [WB] |
| **Bytes_Per_Row** | The configurable number of bytes displayed on each hex row. Default is 16. Must be a power of 2 (8, 16, 32, 64). | [WB] |
| **Hex_Cursor** | The cursor in hex mode, which can reside in either the Hex_Pane or the ASCII_Pane. Position is always synchronised between panes. | [FFE-HEX] |
| **Nibble_Position** | When editing in the Hex_Pane, the cursor can be on the high nibble (first hex digit) or low nibble (second hex digit) of a byte. | [WB] |
| **Non_Printable_Indicator** | The substitution character (`.`) shown in the ASCII_Pane for bytes with no printable ASCII representation (0x00–0x1F, 0x7F–0xFF). | [FFE-HEX] |
| **Byte_Offset** | The 0-based position of a byte within the document buffer, displayed in hex in the Offset_Column. | [WB] |
| **Modified_Byte** | A byte that has been edited in the current session but not yet saved. Highlighted with a distinct colour in both the Hex_Pane and ASCII_Pane. | [WB] |
| **Hex_Row** | A single display row in hex mode, containing the offset, hex digits for Bytes_Per_Row bytes, and their ASCII representation. | [WB] |
| **Hex_Dump** | An exported text representation of the document in hex format, suitable for external analysis or documentation. | [WB] |

---

## Requirements

### Requirement 1: HEX ON / HEX OFF / HEX Toggle Commands

**User Story:** As an editor user, I want to switch between normal text display and hex display mode using primary commands, so that I can inspect the raw byte content of any file without leaving the editor.

**Source:** [FFE-HEX], [WB]

#### Acceptance Criteria

1. THE Command_Framework SHALL register `HEX ON` as a primary command that activates Hex_Mode for the current editor session. [FFE-HEX]
2. THE Command_Framework SHALL register `HEX OFF` as a primary command that deactivates Hex_Mode and returns the viewport to normal text display. [FFE-HEX]
3. THE Command_Framework SHALL register `HEX` (with no argument) as a primary command that toggles Hex_Mode — activating it if currently off, deactivating it if currently on. [FFE-HEX]
4. WHEN `HEX ON` is issued and Hex_Mode is already active, THE system SHALL display a status message "Hex mode is already active" and SHALL NOT change any state. [FFE-HEX]
5. WHEN `HEX OFF` is issued and Hex_Mode is already inactive, THE system SHALL display a status message "Hex mode is already off" and SHALL NOT change any state. [FFE-HEX]
6. THE `HEX ON`, `HEX OFF`, and `HEX` commands SHALL be valid in Browse mode, Edit mode, and View mode. [FFE-HEX]
7. THE hex mode state change SHALL NOT be added to the Undo_Stack — it is a non-undoable display state change. [FFE-HEX]
8. WHEN Hex_Mode is active, THE status bar SHALL display a `HEX` indicator to clearly show the current display mode. [FFE-HEX]
9. WHEN transitioning from text mode to Hex_Mode, THE system SHALL preserve the current cursor byte position and map it to the corresponding Hex_Row and column in the hex view. [WB]
10. WHEN transitioning from Hex_Mode back to text mode, THE system SHALL restore the cursor to the text line and column corresponding to the current hex cursor byte offset. [WB]

---

### Requirement 2: Hex View Layout

**User Story:** As an editor user, I want the hex view to present content in a clear three-pane layout (offset, hex bytes, ASCII), so that I can quickly correlate byte positions, hex values, and character representations.

**Source:** [FFE-HEX], [WB]

#### Acceptance Criteria

1. WHEN Hex_Mode is active, THE system SHALL render each row with three regions: Offset_Column on the left, Hex_Pane in the centre, and ASCII_Pane on the right. [WB]
2. THE Offset_Column SHALL display the hexadecimal byte offset of the first byte on that row, zero-padded to 8 digits (e.g., `00000000`, `00000010`). For documents exceeding 4 GB, the offset SHALL expand to the minimum number of hex digits needed. [WB]
3. THE Hex_Pane SHALL display each byte as exactly two uppercase hex digits (default), with a single space separating each byte pair. [FFE-HEX]
4. THE Hex_Pane SHALL insert an additional space after every group of 8 bytes (half-row separator) when Bytes_Per_Row is 16 or greater, creating visual groupings for easier reading. [WB]
5. THE ASCII_Pane SHALL display each byte as its printable ASCII character (0x20–0x7E) or as the Non_Printable_Indicator (`.`) for bytes outside the printable range. [FFE-HEX]
6. THE Offset_Column, Hex_Pane, and ASCII_Pane SHALL be visually separated by a column delimiter character or themed separator. [WB]
7. WHEN a row contains fewer bytes than Bytes_Per_Row (final row of document), THE Hex_Pane SHALL pad the remaining positions with spaces to maintain column alignment, and the ASCII_Pane SHALL also be padded. [WB]
8. THE Hex_Pane and ASCII_Pane SHALL use a monospaced font to ensure exact column alignment across all rows. [WB]
9. THE line-number/prefix area used in normal text mode SHALL be replaced by the Offset_Column in hex mode. [FFE-HEX]
10. WHEN the document is empty (zero bytes), THE system SHALL display a single row with offset `00000000` and empty hex/ASCII panes. [WB]

---

### Requirement 3: Configurable Bytes Per Row

**User Story:** As an editor user, I want to configure how many bytes are shown per row, so that I can optimise the hex display for my screen width and analysis needs.

**Source:** [WB]

#### Acceptance Criteria

1. THE system SHALL support a configurable Bytes_Per_Row setting with valid values: 8, 16, 32, and 64. [WB]
2. THE default Bytes_Per_Row value SHALL be 16. [WB]
3. WHEN the user changes Bytes_Per_Row while Hex_Mode is active, THE system SHALL immediately re-render the hex view with the new row width, preserving the current cursor byte offset. [WB]
4. IF an invalid Bytes_Per_Row value is specified (not 8, 16, 32, or 64), THEN THE system SHALL reject the value with an error message and retain the current setting. [WB]
5. THE Bytes_Per_Row setting SHALL be persisted in the configuration system under `editor.hex.bytes_per_row`. [WB]
6. WHEN Bytes_Per_Row changes, THE Offset_Column, Hex_Pane, and ASCII_Pane widths SHALL adjust proportionally to accommodate the new byte count. [WB]

---

### Requirement 4: Hex Editing (Overwrite Mode)

**User Story:** As an editor user, I want to edit raw bytes by overwriting hex digits in the hex pane or characters in the ASCII pane, so that I can make precise byte-level modifications to binary data.

**Source:** [FFE-HEX], [WB]

#### Acceptance Criteria

1. WHEN the cursor is in the Hex_Pane and the user types a valid hex digit (0–9, A–F, a–f), THE system SHALL overwrite the current nibble (high or low) at the Nibble_Position, advance the cursor to the next nibble, and update both the Hex_Pane and the corresponding byte in the ASCII_Pane immediately. [FFE-HEX]
2. WHEN both nibbles of a byte have been entered in the Hex_Pane, THE cursor SHALL advance to the high nibble of the next byte. [WB]
3. WHEN the cursor is in the ASCII_Pane and the user types a printable character (0x20–0x7E), THE system SHALL overwrite the byte at the current position with the character's ASCII value and update the corresponding hex digits in the Hex_Pane. [WB]
4. WHEN the user types an invalid hex digit (not 0–9 or A–F/a–f) while the cursor is in the Hex_Pane, THE system SHALL ignore the input and display a brief status message "Invalid hex digit". [FFE-HEX]
5. EACH byte modification in hex mode SHALL be recorded as a standard Edit_Operation through the undo-redo-transactions system. [FFE-HEX]
6. WHEN hex mode editing is attempted in Browse mode or View mode, THE system SHALL reject the edit and display "Cannot edit in Browse/View mode". [WB]
7. THE system SHALL support consecutive rapid hex digit entries that are coalesced into a single undo transaction (consistent with coalescing rules defined in undo-redo-transactions). [WB]
8. WHEN a byte is modified, THE system SHALL mark that byte as a Modified_Byte with a distinct visual highlight in both the Hex_Pane and ASCII_Pane until the document is saved. [WB]
9. WHEN the editor is in EBCDIC mode (EBCDIC encoding active for the current file), THE system SHALL display a warning "Hex editing on EBCDIC files modifies raw bytes directly — ensure edited values are valid EBCDIC characters" when the cursor first enters the Hex_Pane. [FFE-HEX]

---

### Requirement 5: Hex Search Integration (FIND X'...')

**User Story:** As an editor user, I want to search for specific byte sequences using hex notation, so that I can locate non-printable characters, packed values, or binary markers that cannot be typed directly.

**Source:** [FFE-HEX], [WB]

#### Acceptance Criteria

1. THE find-and-replace engine SHALL support `FIND X'hexdigits'` as a search form, where `hexdigits` is a sequence of hex digit pairs representing the bytes to search for. [FFE-HEX]
2. WHEN a hex pattern match is found and Hex_Mode is not currently active, THE system SHALL automatically activate Hex_Mode so the user can see the matching bytes highlighted in context. [FFE-HEX]
3. WHEN a hex pattern match is found in Hex_Mode, THE system SHALL highlight the matching byte range in both the Hex_Pane (hex digit pairs) and the ASCII_Pane (corresponding characters). [FFE-HEX]
4. THE `FIND X'...'` form SHALL support all existing FIND scope modifiers: ALL, NEXT, PREV, FIRST, LAST. [FFE-HEX]
5. WHEN an odd number of hex digits is provided (e.g., `X'0D0'`), THE system SHALL display a syntax error "Hex pattern must contain an even number of digits" and SHALL NOT execute the search. [FFE-HEX]
6. WHEN `FIND X'0D0A'` is issued, THE system SHALL search for the byte sequence `0x0D 0x0A` at any byte position in the document, regardless of line boundaries. [FFE-HEX]
7. THE hex search SHALL operate on raw bytes without Unicode case folding — it matches exact byte sequences. [FFE-HEX]
8. WHEN hex search finds a match, THE viewport SHALL scroll to reveal the matching row and the cursor SHALL be positioned at the first byte of the match. [WB]

---

### Requirement 6: Cursor Synchronisation Between Panes

**User Story:** As an editor user, I want the cursor position to stay synchronised between the hex pane and ASCII pane, so that I always know which byte I'm looking at regardless of which pane has focus.

**Source:** [FFE-HEX], [WB]

#### Acceptance Criteria

1. WHEN the cursor moves in the Hex_Pane, THE ASCII_Pane SHALL highlight the corresponding byte position to show which character maps to the current hex digits. [WB]
2. WHEN the cursor moves in the ASCII_Pane, THE Hex_Pane SHALL highlight the corresponding hex digit pair to show which hex value maps to the current character. [WB]
3. THE user SHALL be able to switch focus between the Hex_Pane and ASCII_Pane using a configurable key (default: Tab). [FFE-HEX]
4. WHEN switching panes, THE cursor SHALL remain on the same byte offset — only the active editing pane changes. [WB]
5. WHEN the cursor moves in either pane, THE Offset_Column SHALL visually indicate the current row (e.g., with a highlight or a marker). [WB]
6. Arrow key navigation in the Hex_Pane SHALL move by nibbles horizontally (Left/Right move one nibble) and by one full row vertically (Up/Down move by Bytes_Per_Row bytes). [WB]
7. Arrow key navigation in the ASCII_Pane SHALL move by bytes horizontally (Left/Right move one byte) and by one full row vertically (Up/Down move by Bytes_Per_Row bytes). [WB]
8. WHEN navigating past the end of a row, THE cursor SHALL wrap to the beginning of the next row (Right at end) or end of the previous row (Left at beginning). [WB]

---

### Requirement 7: Undo/Redo in Hex Mode

**User Story:** As an editor user, I want full undo/redo support while editing in hex mode, so that I can safely experiment with byte modifications and revert mistakes.

**Source:** [FFE-HEX], [WB]

#### Acceptance Criteria

1. WHEN a byte is modified in hex mode, THE modification SHALL be recorded as a reversible Edit_Operation in the undo-redo-transactions system. [FFE-HEX]
2. WHEN Undo is invoked in hex mode, THE system SHALL reverse the most recent hex edit transaction, restoring the original byte value and updating both the Hex_Pane and ASCII_Pane. [WB]
3. WHEN Redo is invoked in hex mode, THE system SHALL re-apply the most recently undone hex edit transaction. [WB]
4. WHEN multiple consecutive single-nibble edits form a complete byte change (high nibble + low nibble), THE system SHALL coalesce them into a single undo transaction. [WB]
5. WHEN undo/redo changes a byte, THE Modified_Byte indicator SHALL be updated: restored bytes lose the indicator if they match the saved state; re-modified bytes gain it. [WB]
6. THE undo/redo behaviour in hex mode SHALL be identical to undo/redo in text mode — hex edits and text edits share the same undo stack. [WB]

---

### Requirement 8: Modified Byte Indicators

**User Story:** As an editor user, I want to see which bytes have been changed since the last save, so that I can visually track my hex edits.

**Source:** [WB]

#### Acceptance Criteria

1. WHEN a byte has been modified since the last save, THE Hex_Pane SHALL render that byte's hex digits with a distinct highlight colour (using the theme's `hex.modified_byte` token). [WB]
2. WHEN a byte has been modified since the last save, THE ASCII_Pane SHALL render that byte's character with the same modified highlight. [WB]
3. WHEN the document is saved, ALL Modified_Byte indicators SHALL be cleared since the saved state now matches the buffer. [WB]
4. WHEN undo restores a byte to its saved-state value, THE Modified_Byte indicator for that byte SHALL be removed. [WB]
5. THE modified byte tracking SHALL work correctly even when bytes are modified, undone, and re-modified multiple times — the indicator reflects whether the current value differs from the last-saved value. [WB]

---

### Requirement 9: Scrolling and Viewport in Hex Mode

**User Story:** As an editor user, I want scrolling in hex mode to behave predictably with row-based navigation, so that I can efficiently browse large binary files.

**Source:** [FFE-HEX], [WB]

#### Acceptance Criteria

1. THE hex mode viewport SHALL calculate total row count as `ceil(document_byte_length / Bytes_Per_Row)` and SHALL integrate with the viewport-and-scrolling system for scrollbar proportionality. [WB]
2. WHEN Page Down is pressed in hex mode, THE viewport SHALL advance by the number of visible hex rows (viewport height in rows). [WB]
3. WHEN Page Up is pressed in hex mode, THE viewport SHALL move back by the number of visible hex rows, clamped to row 0. [WB]
4. THE vertical scrollbar SHALL map the full hex row range [0, total_rows) onto the scrollbar track with proportional thumb size. [WB]
5. WHEN the cursor moves outside the visible viewport (via editing or navigation), THE viewport SHALL scroll to keep the cursor row visible using the caret-visibility policies defined in viewport-and-scrolling. [WB]
6. THE horizontal scrollbar SHALL be hidden in hex mode when the hex row width (offset + hex + ASCII) fits within the window width. If it does not fit, horizontal scrolling SHALL be enabled. [WB]
7. WHEN Bytes_Per_Row changes, THE system SHALL recalculate total row count and adjust the scrollbar accordingly without changing the byte offset currently at the top of the viewport. [WB]
8. WHEN scrolling, THE system SHALL always display complete hex rows — partial rows SHALL NOT be rendered at the top or bottom of the viewport. [WB]

---

### Requirement 10: Hex View for Binary vs Text Files

**User Story:** As an editor user, I want hex mode to handle both text and binary files appropriately, with automatic detection and optional auto-activation for binary content.

**Source:** [WB]

#### Acceptance Criteria

1. WHEN a file is detected as binary (containing null bytes or non-text byte sequences as determined by the encoding detection in encoding-and-characters), THE system SHALL offer to open it in Hex_Mode automatically. [WB]
2. THE auto-hex-for-binary behaviour SHALL be configurable via `editor.hex.auto_activate_binary` (default: true — prompt user; can be set to "always" or "never"). [WB]
3. WHEN Hex_Mode is active on a text file, THE ASCII_Pane SHALL show the text characters faithfully, including line-ending bytes (CR as `0D`, LF as `0A`) which are normally invisible in text mode. [WB]
4. WHEN Hex_Mode is active on a binary file, THE system SHALL NOT interpret line endings — content is displayed strictly as a byte stream organised into fixed-width rows. [WB]
5. THE system SHALL handle files of any size in hex mode by loading only the visible byte range from the VFS (consistent with streaming/chunked access from document-model). [WB]
6. WHEN hex mode is displaying a file open in text mode, byte offsets SHALL correspond to the actual byte positions in the document buffer (accounting for the gap buffer's gap). [WB]

---

### Requirement 11: Hex Dump Export

**User Story:** As an editor user, I want to export the current file's content as a hex dump to a text file or the clipboard, so that I can share hex data with colleagues or include it in documentation.

**Source:** [WB]

#### Acceptance Criteria

1. THE Command_Framework SHALL register a `HEX DUMP` command that exports the document's content in hex dump format. [WB]
2. THE hex dump output SHALL follow the same three-column layout as the hex view: offset, hex bytes, and ASCII representation — one row per Bytes_Per_Row bytes. [WB]
3. WHEN `HEX DUMP` is issued with no arguments, THE system SHALL export the entire document. [WB]
4. WHEN `HEX DUMP` is issued with a byte range (e.g., `HEX DUMP 0x0000 0x00FF`), THE system SHALL export only the specified byte range. [WB]
5. THE system SHALL support exporting the hex dump to a new editor tab (`HEX DUMP EDIT`), to the clipboard (`HEX DUMP CLIP`), or to a file (`HEX DUMP FILE 'path'`). [WB]
6. THE hex dump output SHALL use the current Bytes_Per_Row and uppercase/lowercase settings. [WB]
7. WHEN a selection exists in hex mode, `HEX DUMP` with no range argument SHALL export only the selected bytes. [WB]

---

### Requirement 12: Goto Offset Command

**User Story:** As an editor user, I want to jump directly to a specific byte offset in the document, so that I can navigate quickly to known positions in large binary files.

**Source:** [WB]

#### Acceptance Criteria

1. THE Command_Framework SHALL register a `GOTO` command that accepts a hexadecimal byte offset (e.g., `GOTO X'1A4F'` or `GOTO 0x1A4F`). [WB]
2. WHEN `GOTO X'offset'` is issued in Hex_Mode, THE system SHALL position the cursor at the specified byte offset and scroll the viewport to make that row visible. [WB]
3. WHEN `GOTO X'offset'` is issued and Hex_Mode is not active, THE system SHALL activate Hex_Mode and then navigate to the specified offset. [WB]
4. IF the specified offset exceeds the document length, THEN THE system SHALL display an error "Offset X'...' exceeds document size (X'size' bytes)" and SHALL NOT move the cursor. [WB]
5. THE `GOTO` command SHALL accept offsets in hexadecimal (prefixed with `X'...'` or `0x`) and decimal (no prefix) formats. [WB]
6. WHEN the GOTO command completes successfully, THE Offset_Column SHALL clearly indicate the target row and the cursor SHALL be positioned at the exact target byte. [WB]

---

### Requirement 13: Configurable Uppercase/Lowercase Hex Digits

**User Story:** As an editor user, I want to choose whether hex digits are displayed in uppercase (A–F) or lowercase (a–f), so that the display matches my preference and industry conventions.

**Source:** [WB]

#### Acceptance Criteria

1. THE system SHALL support a configurable hex digit case setting with values: `uppercase` (default) and `lowercase`. [WB]
2. WHEN `uppercase` is configured, THE Hex_Pane SHALL display all hex digits as A–F for values 10–15. [WB]
3. WHEN `lowercase` is configured, THE Hex_Pane SHALL display all hex digits as a–f for values 10–15. [WB]
4. THE Offset_Column SHALL follow the same case setting as the Hex_Pane. [WB]
5. THE hex digit case setting SHALL be persisted in the configuration system under `editor.hex.digit_case`. [WB]
6. WHEN the setting is changed while Hex_Mode is active, THE display SHALL update immediately without requiring a mode toggle. [WB]
7. HEX DUMP export output SHALL use the current hex digit case setting. [WB]

---

### Requirement 14: Hex Mode with FileForge Structured Files

**User Story:** As a data engineer, I want hex mode to work when viewing fixed-width flat files in FileForge_Mode, so that I can inspect raw byte content of fields and diagnose packed decimal values or non-printable characters at specific offsets.

**Source:** [FFE-HEX]

#### Acceptance Criteria

1. WHEN Hex_Mode is active and FileForge_Mode is also active, THE system SHALL render hex display for all records showing the raw bytes alongside or beneath the structured field representation. [FFE-HEX]
2. WHEN the user selects a cell in the FileForge grid and Hex_Mode is active, THE system SHALL highlight the corresponding byte range in the Hex_Pane to show which hex digits correspond to that field's offset and length. [FFE-HEX]
3. THE field boundary positions (from the active Record_Structure's offset/length definitions) SHALL be indicated in the Hex_Pane by a visual separator or colour change at each field boundary. [FFE-HEX]
4. WHEN a COMP-3 (packed decimal) field is identified in the Record_Structure, THE system SHALL annotate the hex digits with the decoded numeric value as a tooltip or inline annotation. [FFE-HEX]

---

### Requirement 15: Hex Mode Session State

**User Story:** As an editor user, I want the editor to remember my hex mode preferences per file, so that binary files reopen in hex mode and my settings persist across sessions.

**Source:** [FFE-HEX], [WB]

#### Acceptance Criteria

1. THE hex mode state (on or off) SHALL be stored in the session history entry for each file. [FFE-HEX]
2. WHEN a file is reopened and its session history indicates hex mode was previously active, THE system SHALL restore hex mode automatically. [FFE-HEX]
3. THE per-file hex session state SHALL include: Hex_Mode on/off, Bytes_Per_Row, cursor byte offset, viewport top row, and active pane (Hex or ASCII). [WB]
4. THE hex mode state SHALL be shown in the status bar as a persistent `HEX` indicator when active, so the user always knows the current display mode. [FFE-HEX]
5. WHEN a file was previously opened in hex mode and the user opens it again, THE system SHALL restore the cursor to the previously active byte offset. [WB]

---

### Requirement 16: Hex Mode Command Compatibility

**User Story:** As an editor user, I want all existing primary commands to continue working when hex mode is active, so that hex display does not disrupt my workflow.

**Source:** [FFE-HEX]

#### Acceptance Criteria

1. WHEN Hex_Mode is active, ALL existing primary commands (FIND, CHANGE, SORT, EXCLUDE, etc.) SHALL continue to operate normally on the underlying text content. [FFE-HEX]
2. WHEN `FIND 'text'` is executed in Hex_Mode, THE system SHALL highlight the matching text in both the Hex_Pane (corresponding hex digits) and the ASCII_Pane (matching characters). [FFE-HEX]
3. WHEN a line command is issued while Hex_Mode is active and the document is in text mode, THE system SHALL apply the line command to the underlying text line that contains the cursor's current byte offset. [WB]
4. WHEN CHANGE modifies text content while Hex_Mode is active, THE hex display SHALL update immediately to reflect the new byte values. [WB]

