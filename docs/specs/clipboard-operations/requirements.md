# Requirements Document

## Introduction

The `clipboard-operations` sub-project defines all clipboard interaction behaviour within the FileForgeWorkbench editor. It merges the FileForgeEditor clipboard functionality from two primary sources:

1. **FFE `copy-clipboard-paste`** — The COPY primary command's clipboard-paste mode, file-insert mode, and shell-capture mode (inserting content at `A`/`B` target positions via the command engine)
2. **FFE `mvp-implementation` Requirement 8** — Standard desktop clipboard keyboard shortcuts (Ctrl+C/X/V), context menu integration, selection-based cut/copy/paste, line-copy-when-no-selection, and undo/redo keyboard shortcuts for clipboard operations

These are unified into a single clipboard subsystem that provides:

- System clipboard access abstraction (read/write/availability detection)
- Standard keyboard shortcut and context menu clipboard operations
- COPY command modes: in-document copy, clipboard-paste, file-insert, shell-capture
- Rectangular clipboard content handling
- Multi-caret clipboard distribution
- Line-copy mode (copy entire line when no selection exists)
- Clipboard unavailability and error handling
- Undoable paste operations via the transaction system

**Scope boundaries:**
- Selection creation, extension, and manipulation (stream, rectangular, multi-caret) are defined in `edit-operations` — this spec defines what happens when clipboard operations interact with those selections
- Undo/redo transaction mechanics (TransactionStack, coalescing, save points) are defined in `undo-redo-transactions` — this spec defines what constitutes an undoable clipboard transaction
- The COPY line command (C/CC) for in-document duplication is defined in `line-commands` — this spec defines the COPY primary command's clipboard-paste and file-insert routing
- Shell command execution and output capture are defined in `shell-command` — this spec defines how SHELL capture mode content reaches the clipboard subsystem
- Command registration, dispatch, and shortcut bindings are defined in `command-framework` — this spec defines clipboard command IDs and their semantics
- File reading for file-insert mode uses the VFS abstraction from `virtual-file-system`

**Source references:**
- **[FFE-CLIP]** = FileForgeEditor `copy-clipboard-paste` spec (10 requirements — clipboard paste, file-insert, disambiguation, error handling, compatibility matrix)
- **[FFE-MVP-8]** = FileForgeEditor `mvp-implementation` Requirement 8 (Standard Desktop Editor Interactions — Ctrl+C/X/V, context menu, selection, line-copy)
- **[WB]** = Workbench Platform Architecture Brief (GUI independence, command-driven, VFS-aware file access)
- **[SCI-SEL-4.1]** = Scintilla selection model (rectangular selection clipboard, multi-caret clipboard distribution)

### Cross-References

- **`edit-operations`** — Defines selection model, multi-caret, rectangular selection; clipboard operations consume/produce selection content
- **`command-framework`** — Clipboard commands are registered as Command_IDs; keyboard shortcuts are bound via Shortcut_Registry
- **`shell-command`** — SHELL document-capture mode shares line-insertion mechanics with COPY clipboard-paste mode
- **`undo-redo-transactions`** — All paste/cut operations produce Undo_Records pushed onto the TransactionStack
- **`virtual-file-system`** — File-insert mode reads files through the VFS abstraction layer
- **`line-commands`** — C/CC line commands define in-document copy source; A/B line commands define insertion targets
- **`configuration-system`** — Provides clipboard-related configuration keys

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **Clipboard_Engine** | The workbench subsystem responsible for reading from and writing to the system clipboard. Provides a platform-independent abstraction over OS clipboard APIs. | [FFE-CLIP] |
| **System_Clipboard** | The operating system's clipboard facility (e.g., Win32 clipboard, X11 selections, macOS pasteboard) that holds text content copied from the editor or any other application. | [FFE-CLIP] |
| **Clipboard_Content** | The text currently held in the System_Clipboard at the time a clipboard operation is executed. | [FFE-CLIP] |
| **Clipboard_Entry** | A structured representation of clipboard content including: the text payload, the clipboard mode (stream, line, rectangular), and optional per-line segments for rectangular or multi-caret content. | [SCI-SEL-4.1] |
| **Clipboard_Mode** | An enum indicating how clipboard content was acquired: `Stream` (normal character selection), `Line` (full-line copy with no explicit selection), `Rectangular` (column block selection). Affects paste behaviour. | [SCI-SEL-4.1] |
| **Line_Copy_Mode** | A clipboard mode flag set when the user copies with no active selection — the entire current line is copied, and paste inserts it as a new line rather than inline at the caret. | [FFE-MVP-8] |
| **Rectangular_Clipboard** | Clipboard content acquired from a rectangular (column) selection. Each line segment is stored independently and pasted as a column block at the target position. | [SCI-SEL-4.1] |
| **Multi_Caret_Clipboard** | Clipboard content acquired from a multi-caret selection. Each caret's selection is stored as an independent segment. On paste, segments are distributed one-per-caret if the caret count matches; otherwise, full content is pasted at each caret. | [SCI-SEL-4.1] |
| **COPY** | The primary command that copies content to a target location. Has four modes: in-document (C/CC source + A/B target), clipboard-paste (no args, no source, A/B target), file-insert (path arg + A/B target), shell-capture (SHELL cmd + A/B target). | [FFE-CLIP] |
| **File_Path** | A relative or absolute path to a file on the filesystem supplied as the first argument to `COPY`. Paths containing spaces must be enclosed in double quotes. | [FFE-CLIP] |
| **A (After)** | A line command target marker that designates the insertion point to be immediately after the marked line. | [FFE-CLIP] |
| **B (Before)** | A line command target marker that designates the insertion point to be immediately before the marked line. | [FFE-CLIP] |
| **Target_Line** | The document line carrying an `A` or `B` line command marker at the time a clipboard/file-insert operation is executed. | [FFE-CLIP] |
| **Pending_Source_Command** | A `C` or `CC` line command currently active in the prefix area and awaiting resolution. | [FFE-CLIP] |
| **Logical_Line** | A single record in the editor's document model. | [FFE-CLIP] |
| **Command_Engine** | The workbench command dispatch subsystem that parses, validates, and executes commands (equivalent to `command-framework`). | [FFE-CLIP, WB] |
| **Undo_Record** | An opaque token produced by an undoable command, encapsulating the information needed to reverse the command's effect. Managed by `undo-redo-transactions`. | [WB] |
| **VFS** | Virtual File System abstraction layer (FFW-ARCH-001). File-insert mode reads files through VFS providers. | [WB] |

---

## Requirements

### Requirement 1: Clipboard Engine — System Clipboard Access [FFE-CLIP, WB]

**User Story:** As an editor user, I want the editor to reliably access the system clipboard on all supported platforms, so that I can copy and paste text between the editor and other applications.

#### Acceptance Criteria

1.1 THE Clipboard_Engine SHALL provide a platform-independent trait for clipboard read and write operations, abstracting over Win32 clipboard (Windows), X11/Wayland selections (Linux), and NSPasteboard (macOS). [WB]

1.2 WHEN the Clipboard_Engine writes text to the System_Clipboard, it SHALL store the text as plain UTF-8 content in the platform's standard text clipboard format. [FFE-CLIP]

1.3 WHEN the Clipboard_Engine reads from the System_Clipboard, it SHALL retrieve plain text content only; non-text content (images, binary data, rich text) SHALL be reported as unavailable text. [FFE-CLIP]

1.4 THE Clipboard_Engine SHALL store a Clipboard_Entry alongside the system clipboard write, recording the Clipboard_Mode (Stream, Line, or Rectangular) and optional per-line segments for rectangular or multi-caret content. [SCI-SEL-4.1]

1.5 WHEN the Clipboard_Engine reads clipboard content that was written by an external application (not this editor instance), THE Clipboard_Mode SHALL default to Stream. [SCI-SEL-4.1]

1.6 IF the System_Clipboard cannot be accessed due to a platform or permission error, THEN THE Clipboard_Engine SHALL return a descriptive error without panicking or modifying the document. [FFE-CLIP]

1.7 THE Clipboard_Engine SHALL be implemented as a GUI-independent service with no direct dependency on any rendering framework. [WB]

---

### Requirement 2: Copy Operation — Keyboard Shortcut [FFE-MVP-8]

**User Story:** As a desktop user, I want to press Ctrl+C to copy selected text to the clipboard, so that I can use the familiar keyboard shortcut for clipboard operations.

#### Acceptance Criteria

2.1 WHEN the user presses Ctrl+C with an active stream selection, THE editor SHALL copy the selected text to the System_Clipboard with Clipboard_Mode set to Stream. [FFE-MVP-8]

2.2 WHEN the user presses Ctrl+C with an active rectangular selection, THE editor SHALL copy the rectangular block to the System_Clipboard with Clipboard_Mode set to Rectangular, preserving each line segment independently. [SCI-SEL-4.1]

2.3 WHEN the user presses Ctrl+C with multiple carets each having an active selection, THE editor SHALL copy each caret's selected text as a separate segment in the Clipboard_Entry with per-caret segment storage. [SCI-SEL-4.1]

2.4 WHEN no text is selected and the user presses Ctrl+C, THE editor SHALL copy the entire current line (including its line ending) to the System_Clipboard with Clipboard_Mode set to Line. [FFE-MVP-8]

2.5 WHEN Ctrl+C is pressed, THE editor SHALL NOT modify the document content or the current selection. [FFE-MVP-8]

2.6 THE copy operation SHALL be registered as command ID `"clipboard.copy"` in the Command_Registry with the default shortcut Ctrl+C. [WB]

---

### Requirement 3: Cut Operation — Keyboard Shortcut [FFE-MVP-8]

**User Story:** As a desktop user, I want to press Ctrl+X to cut selected text to the clipboard, so that I can move text using the familiar cut-and-paste workflow.

#### Acceptance Criteria

3.1 WHEN the user presses Ctrl+X with an active stream selection, THE editor SHALL copy the selected text to the System_Clipboard with Clipboard_Mode set to Stream, then delete the selected text from the document. [FFE-MVP-8]

3.2 WHEN the user presses Ctrl+X with an active rectangular selection, THE editor SHALL copy the rectangular block to the System_Clipboard with Clipboard_Mode set to Rectangular, then delete the selected column block from all affected lines. [SCI-SEL-4.1]

3.3 WHEN the user presses Ctrl+X with multiple carets each having an active selection, THE editor SHALL copy each caret's selected text as a separate segment, then delete all selected regions simultaneously as a single Undo_Record. [SCI-SEL-4.1]

3.4 WHEN no text is selected and the user presses Ctrl+X, THE editor SHALL cut the entire current line (including its line ending) to the System_Clipboard with Clipboard_Mode set to Line, removing the line from the document. [FFE-MVP-8]

3.5 WHEN Ctrl+X is executed, THE operation SHALL be recorded as a single Undo_Record in the undo history. [FFE-MVP-8]

3.6 THE cut operation SHALL be registered as command ID `"clipboard.cut"` in the Command_Registry with the default shortcut Ctrl+X. [WB]

---

### Requirement 4: Paste Operation — Keyboard Shortcut [FFE-MVP-8, SCI-SEL-4.1]

**User Story:** As a desktop user, I want to press Ctrl+V to paste clipboard content at the cursor position, with paste behaviour adapting to how the content was originally copied (stream, line, or rectangular).

#### Acceptance Criteria

4.1 WHEN the user presses Ctrl+V and the Clipboard_Mode is Stream, THE editor SHALL insert the clipboard text at the current caret position, replacing any active selection. [FFE-MVP-8]

4.2 WHEN the user presses Ctrl+V and the Clipboard_Mode is Line, THE editor SHALL insert the clipboard content as one or more new lines immediately above the line containing the caret, without splitting the current line. [FFE-MVP-8]

4.3 WHEN the user presses Ctrl+V and the Clipboard_Mode is Rectangular, THE editor SHALL insert each line segment of the rectangular clipboard content as a column block starting at the caret position, with one segment per line downward from the caret line. [SCI-SEL-4.1]

4.4 WHEN the user presses Ctrl+V with multiple carets active and the clipboard contains the same number of segments as active carets, THE editor SHALL distribute one clipboard segment to each caret, pasting each segment at its respective caret position. [SCI-SEL-4.1]

4.5 WHEN the user presses Ctrl+V with multiple carets active and the clipboard segment count does NOT match the caret count, THE editor SHALL paste the full clipboard content at each caret position independently. [SCI-SEL-4.1]

4.6 WHEN the clipboard text contains multiple lines separated by line endings (LF, CRLF, or CR), THE editor SHALL split the text into individual Logical_Lines before insertion. [FFE-CLIP]

4.7 WHEN clipboard text ends with a trailing line ending, THE editor SHALL NOT insert an additional empty Logical_Line for that trailing terminator. [FFE-CLIP]

4.8 THE editor SHALL preserve the exact content of each Logical_Line derived from the clipboard without trimming or modifying whitespace. [FFE-CLIP]

4.9 WHEN Ctrl+V is executed, THE operation SHALL be recorded as a single Undo_Record in the undo history. [FFE-MVP-8]

4.10 THE paste operation SHALL be registered as command ID `"clipboard.paste"` in the Command_Registry with the default shortcut Ctrl+V. [WB]

---

### Requirement 5: Context Menu Clipboard Operations [FFE-MVP-8]

**User Story:** As a desktop user, I want right-click context menu entries for Cut, Copy, and Paste, so that I can access clipboard operations without memorising keyboard shortcuts.

#### Acceptance Criteria

5.1 WHEN the user right-clicks within the document area, THE editor SHALL display a context menu containing at minimum: Cut, Copy, Paste, and Select All. [FFE-MVP-8]

5.2 WHEN the user selects Cut from the context menu with an active selection, THE editor SHALL perform the same operation as the `"clipboard.cut"` command. [FFE-MVP-8]

5.3 WHEN the user selects Copy from the context menu with an active selection, THE editor SHALL perform the same operation as the `"clipboard.copy"` command. [FFE-MVP-8]

5.4 WHEN the user selects Paste from the context menu, THE editor SHALL perform the same operation as the `"clipboard.paste"` command. [FFE-MVP-8]

5.5 WHEN the user selects Select All from the context menu, THE editor SHALL select all text in the current document. [FFE-MVP-8]

5.6 WHEN no selection is active, THE context menu Cut and Copy items SHALL be disabled (greyed out) but Paste SHALL remain enabled if the clipboard contains text. [FFE-MVP-8]

5.7 WHEN the System_Clipboard is empty or contains no text content, THE context menu Paste item SHALL be disabled (greyed out). [FFE-MVP-8]

---

### Requirement 6: Clipboard Unavailability Handling [FFE-CLIP, FFE-MVP-8]

**User Story:** As an editor user, I want clear feedback when the system clipboard is unavailable or empty, so that I understand why a clipboard operation did not succeed.

#### Acceptance Criteria

6.1 IF the System_Clipboard is empty or unavailable when Ctrl+V is pressed, THEN THE editor SHALL display a descriptive message in the status bar and take no other action. [FFE-MVP-8]

6.2 IF the Clipboard_Engine cannot access the System_Clipboard due to a platform or permission error, THEN THE editor SHALL NOT modify the document and SHALL display an error message indicating that clipboard access failed. [FFE-CLIP]

6.3 IF the Clipboard_Engine returns non-text content (such as an image or binary data), THEN THE editor SHALL NOT modify the document and SHALL display an error message indicating that only plain text clipboard content is supported. [FFE-CLIP]

6.4 WHEN a clipboard access error occurs during a copy or cut operation, THE editor SHALL display an error message but SHALL NOT lose the selected text or modify the document. [FFE-CLIP]

6.5 WHEN clipboard access fails, THE editor SHALL retain any pending line commands (A/B targets) in the prefix area so the user can retry after resolving the clipboard issue. [FFE-CLIP]

---

### Requirement 7: COPY Command — Clipboard-Paste Mode [FFE-CLIP]

**User Story:** As an editor user, I want to type `COPY` in the command line with an `A` or `B` target marker and no source line commands pending, so that the clipboard content is pasted at the target location using the ISPF command paradigm.

#### Acceptance Criteria

7.1 WHEN the `COPY` primary command is entered and no `C` or `CC` pending source line commands exist and exactly one `A` or `B` target line command is present and no arguments are supplied, THE Command_Engine SHALL retrieve the current text from the Clipboard_Engine and insert it at the position defined by the target line command. [FFE-CLIP]

7.2 WHEN the `COPY` command resolves to clipboard-paste mode and the `A` target is present, THE Command_Engine SHALL insert the clipboard lines immediately after the Target_Line. [FFE-CLIP]

7.3 WHEN the `COPY` command resolves to clipboard-paste mode and the `B` target is present, THE Command_Engine SHALL insert the clipboard lines immediately before the Target_Line. [FFE-CLIP]

7.4 WHEN the clipboard-paste operation completes successfully, THE Command_Engine SHALL clear the resolved `A` or `B` target line command from the prefix area. [FFE-CLIP]

7.5 WHEN the clipboard-paste operation completes successfully, THE Command_Engine SHALL display a status message indicating the number of lines inserted. [FFE-CLIP]

7.6 WHEN the `COPY` command resolves to clipboard-paste mode and the Clipboard_Engine returns empty content, THE Command_Engine SHALL NOT modify the document and SHALL display an error message indicating that the clipboard is empty. [FFE-CLIP]

7.7 WHEN the `COPY` command resolves to clipboard-paste mode and the clipboard is empty, THE Command_Engine SHALL retain the `A` or `B` target line command in the prefix area so the user can retry after copying content to the clipboard. [FFE-CLIP]

7.8 WHEN a clipboard-paste operation via `COPY` is executed successfully, THE Command_Engine SHALL record the operation as a single Undo_Record in the undo history. [FFE-CLIP]

---

### Requirement 8: COPY Command — Disambiguation and Routing [FFE-CLIP]

**User Story:** As an editor user, I want the editor to unambiguously choose between in-document copy, clipboard-paste, file-insert, and shell-capture modes when I issue the COPY command, so that my intent is always interpreted correctly.

#### Acceptance Criteria

8.1 WHEN the `COPY` primary command is entered and one or more `C` or `CC` pending source line commands are present and an `A` or `B` target is present, THE Command_Engine SHALL execute the existing in-document copy behaviour (route to `line-commands`), regardless of clipboard content. [FFE-CLIP]

8.2 WHEN the `COPY` primary command is entered and no pending source line commands exist and no `A` or `B` target line command is present and no arguments are supplied, THE Command_Engine SHALL display an error message stating that a target line command `A` or `B` is required. [FFE-CLIP]

8.3 THE Command_Engine SHALL NOT enter clipboard-paste mode if any `C` or `CC` source line commands are pending, even if an `A` or `B` target is also present. [FFE-CLIP]

8.4 THE Command_Engine SHALL recognise the combination `COPY` + `A` or `B` (no pending `C`/`CC`, no arguments) as the clipboard-paste mode form. [FFE-CLIP]

8.5 THE Command_Engine SHALL recognise the combination `COPY <path>` + `A` or `B` (no pending `C`/`CC`) as the file-insert mode form. [FFE-CLIP]

8.6 THE Command_Engine SHALL give precedence to file-insert mode over clipboard-paste mode when a File_Path argument is present. [FFE-CLIP]

8.7 THE Command_Engine SHALL treat the combination `COPY <path>` + pending `C`/`CC` as an invalid command and display an error message indicating that source line commands cannot be combined with a file path argument. [FFE-CLIP]

8.8 THE Command_Engine SHALL treat the combination `COPY` + pending `C`/`CC` + no `A`/`B` target as incomplete, retaining pending commands and displaying a status message requesting a target. [FFE-CLIP]

---

### Requirement 9: COPY Command — File-Insert Mode [FFE-CLIP, WB]

**User Story:** As an editor user, I want to type `COPY path/to/file` in the command line with an `A` or `B` target marker, so that the contents of the specified file are inserted at the target location without requiring me to open the file separately.

#### Acceptance Criteria

9.1 WHEN the `COPY` primary command is entered with a File_Path argument and no `C` or `CC` pending source line commands exist and exactly one `A` or `B` target line command is present, THE Command_Engine SHALL read the file at the specified path through the VFS abstraction and insert its content at the position defined by the target line command. [FFE-CLIP, WB]

9.2 WHEN the `COPY` command resolves to file-insert mode and the `A` target is present, THE Command_Engine SHALL insert the file lines immediately after the Target_Line. [FFE-CLIP]

9.3 WHEN the `COPY` command resolves to file-insert mode and the `B` target is present, THE Command_Engine SHALL insert the file lines immediately before the Target_Line. [FFE-CLIP]

9.4 WHEN the File_Path is a relative path, THE Command_Engine SHALL resolve it relative to the directory of the resource currently open in the editor session. [FFE-CLIP]

9.5 WHEN the File_Path is an absolute path, THE Command_Engine SHALL use the path as supplied without modification. [FFE-CLIP]

9.6 WHEN the File_Path contains spaces, THE Command_Engine SHALL require the path to be enclosed in double quotes (e.g., `COPY "path/to/my file.txt"`). [FFE-CLIP]

9.7 WHEN the file-insert operation completes successfully, THE Command_Engine SHALL clear the resolved `A` or `B` target line command from the prefix area. [FFE-CLIP]

9.8 WHEN the file-insert operation completes successfully, THE Command_Engine SHALL display a status message indicating the number of lines inserted and the resolved file path. [FFE-CLIP]

9.9 WHEN a file-insert operation is triggered, THE Command_Engine SHALL split the file content into individual Logical_Lines using the same line-ending rules as paste (LF, CRLF, or CR separators; trailing terminator does not produce an empty line). [FFE-CLIP]

9.10 THE Command_Engine SHALL preserve the exact content of each Logical_Line derived from the file without trimming or modifying whitespace. [FFE-CLIP]

9.11 WHEN a file-insert operation via `COPY` is executed successfully, THE Command_Engine SHALL record the operation as a single Undo_Record in the undo history. [FFE-CLIP]

---

### Requirement 10: File-Insert Error Handling [FFE-CLIP]

**User Story:** As an editor user, I want a clear error message when the file specified in a `COPY` command cannot be read, so that I understand why nothing was inserted and can correct my input.

#### Acceptance Criteria

10.1 WHEN the `COPY` command is entered with a File_Path argument and the file does not exist at the resolved path, THE Command_Engine SHALL NOT modify the document and SHALL display an error message indicating that the file was not found, including the resolved path. [FFE-CLIP]

10.2 WHEN the `COPY` command is entered with a File_Path argument and the file exists but cannot be read due to a permission or I/O error, THE Command_Engine SHALL NOT modify the document and SHALL display an error message indicating the access failure. [FFE-CLIP]

10.3 WHEN the `COPY` command is entered with a File_Path argument and the file is detected as binary or non-text content, THE Command_Engine SHALL NOT modify the document and SHALL display an error message indicating that only plain text files are supported. [FFE-CLIP]

10.4 WHEN a file-insert error occurs, THE Command_Engine SHALL retain the `A` or `B` target line command in the prefix area so the user can correct the path and retry. [FFE-CLIP]

---

### Requirement 11: COPY Command — Shell-Capture Mode [FFE-CLIP, WB]

**User Story:** As an editor user, I want to use the `SHELL <command>` primary command with an `A` or `B` target marker to capture command output directly into my document, following the same insertion semantics as COPY clipboard-paste mode.

#### Acceptance Criteria

11.1 WHEN the `SHELL` primary command is entered with one or more arguments and exactly one `A` or `B` target line command is present, THE Command_Engine SHALL execute the arguments as a shell command and insert the captured stdout at the target position, following the same line-splitting and insertion rules as COPY clipboard-paste mode. [FFE-CLIP]

11.2 WHEN shell-capture mode is active and the `A` target is present, THE Command_Engine SHALL insert the captured lines immediately after the Target_Line. [FFE-CLIP]

11.3 WHEN shell-capture mode is active and the `B` target is present, THE Command_Engine SHALL insert the captured lines immediately before the Target_Line. [FFE-CLIP]

11.4 WHEN shell-capture completes successfully, THE Command_Engine SHALL clear the resolved `A` or `B` target line command from the prefix area and display a status message indicating the number of lines inserted. [FFE-CLIP]

11.5 WHEN shell-capture mode is active and the command produces no stdout output, THE Command_Engine SHALL NOT modify the document and SHALL display a message indicating that the command produced no output. [FFE-CLIP]

11.6 WHEN a shell-capture operation is executed successfully, THE Command_Engine SHALL record the operation as a single Undo_Record in the undo history. [FFE-CLIP]

11.7 THE detailed shell execution mechanics (shell detection, security mode, timeout, error handling) are defined in `shell-command`; this requirement defines only the document-insertion contract shared with other COPY modes. [WB]

---

### Requirement 12: Rectangular Clipboard — Copy and Paste [SCI-SEL-4.1]

**User Story:** As an editor user working with columnar data, I want rectangular (column) selections to be copied and pasted as column blocks, so that I can manipulate tabular content without disturbing surrounding text.

#### Acceptance Criteria

12.1 WHEN a rectangular selection is copied, THE Clipboard_Engine SHALL store each line's selected column segment as an independent entry in the Clipboard_Entry, along with Clipboard_Mode set to Rectangular. [SCI-SEL-4.1]

12.2 WHEN rectangular clipboard content is pasted, THE editor SHALL insert each stored segment on successive lines starting at the caret line and caret column, pushing existing text on each line rightward by the segment's width. [SCI-SEL-4.1]

12.3 WHEN rectangular clipboard content is pasted and the caret is beyond the end of a short line, THE editor SHALL pad the line with spaces up to the caret column before inserting the segment. [SCI-SEL-4.1]

12.4 WHEN rectangular clipboard content has more segments than lines remaining below the caret, THE editor SHALL create new lines as needed to accommodate the remaining segments. [SCI-SEL-4.1]

12.5 WHEN rectangular clipboard content is pasted with an active rectangular selection, THE editor SHALL replace the selected rectangular region with the clipboard content, adjusting for width differences. [SCI-SEL-4.1]

12.6 A rectangular paste operation SHALL be recorded as a single Undo_Record in the undo history. [SCI-SEL-4.1]

---

### Requirement 13: Multi-Caret Clipboard Distribution [SCI-SEL-4.1]

**User Story:** As an editor user working with multiple carets, I want clipboard content from a multi-caret copy to be distributed back to the same number of carets on paste, so that I can perform parallel edits efficiently.

#### Acceptance Criteria

13.1 WHEN text is copied with N active carets each having a selection, THE Clipboard_Engine SHALL store N independent text segments in the Clipboard_Entry. [SCI-SEL-4.1]

13.2 WHEN a paste operation is triggered with N active carets and the Clipboard_Entry contains exactly N segments, THE editor SHALL paste segment[i] at caret[i] for each caret, ordered by document position. [SCI-SEL-4.1]

13.3 WHEN a paste operation is triggered with N active carets and the Clipboard_Entry does NOT contain exactly N segments, THE editor SHALL paste the full concatenated clipboard text at each caret position. [SCI-SEL-4.1]

13.4 A multi-caret paste operation SHALL be recorded as a single Undo_Record wrapping all individual insertions, so that undo reverses all pastes simultaneously. [SCI-SEL-4.1]

13.5 WHEN performing a multi-caret paste, THE editor SHALL process carets in reverse document order to prevent earlier insertions from invalidating later caret positions. [SCI-SEL-4.1]

---

### Requirement 14: Line-Copy Mode [FFE-MVP-8]

**User Story:** As an editor user, I want copying with no selection to capture the entire current line and pasting it to insert a new line rather than inline text, so that I can quickly duplicate lines without manually selecting them.

#### Acceptance Criteria

14.1 WHEN the user triggers a copy operation (Ctrl+C or context menu Copy) with no active selection, THE editor SHALL copy the entire current line (including its line ending) to the System_Clipboard with Clipboard_Mode set to Line. [FFE-MVP-8]

14.2 WHEN Clipboard_Mode is Line and a paste operation is triggered, THE editor SHALL insert the clipboard content as one or more new Logical_Lines immediately above the line containing the caret, without splitting the current line or inserting inline. [FFE-MVP-8]

14.3 WHEN Clipboard_Mode is Line and the clipboard contains multiple lines, THE editor SHALL insert all lines as a block above the caret line. [FFE-MVP-8]

14.4 WHEN the user triggers a cut operation (Ctrl+X or context menu Cut) with no active selection, THE editor SHALL cut the entire current line with Clipboard_Mode set to Line, removing the line from the document. [FFE-MVP-8]

14.5 A line-copy paste operation SHALL be recorded as a single Undo_Record in the undo history. [FFE-MVP-8]

---

### Requirement 15: Undoable Clipboard Operations [FFE-CLIP, FFE-MVP-8]

**User Story:** As an editor user, I want all paste and cut operations to be undoable, so that I can reverse accidental clipboard actions.

#### Acceptance Criteria

15.1 WHEN any paste operation (Ctrl+V, COPY clipboard-paste, COPY file-insert, SHELL capture) is executed successfully, THE editor SHALL record it as a single Undo_Record in the undo history. [FFE-CLIP, FFE-MVP-8]

15.2 WHEN the undo command is issued and the most recent recorded operation is a paste, THE editor SHALL remove all content that was inserted by the paste and restore the document to its pre-paste state. [FFE-CLIP]

15.3 WHEN any cut operation (Ctrl+X, context menu Cut) is executed successfully, THE editor SHALL record it as a single Undo_Record in the undo history. [FFE-MVP-8]

15.4 WHEN the undo command is issued and the most recent recorded operation is a cut, THE editor SHALL restore the deleted text at its original position and restore the selection that existed before the cut. [FFE-MVP-8]

15.5 WHEN the user presses Ctrl+Z, THE editor SHALL perform an UNDO operation (equivalent to the UNDO primary command). [FFE-MVP-8]

15.6 WHEN the user presses Ctrl+Y or Ctrl+Shift+Z, THE editor SHALL perform a REDO operation (equivalent to the REDO primary command). [FFE-MVP-8]

---

### Requirement 16: Clipboard Content Line Handling [FFE-CLIP]

**User Story:** As an editor user, I want clipboard text that spans multiple lines to be correctly split into logical lines regardless of the source platform's line-ending convention, so that pasted content integrates correctly with the editor's line model.

#### Acceptance Criteria

16.1 WHEN a paste or clipboard-insert operation is triggered and the clipboard text contains multiple lines separated by line endings (LF, CRLF, or CR), THE editor SHALL split the text into individual Logical_Lines before insertion. [FFE-CLIP]

16.2 WHEN a paste or clipboard-insert operation is triggered and the clipboard text contains a single line with no line-ending characters, THE editor SHALL insert it as exactly one Logical_Line (or inline at the caret for stream paste). [FFE-CLIP]

16.3 WHEN clipboard text ends with a trailing line ending, THE editor SHALL NOT insert an additional empty Logical_Line for that trailing terminator. [FFE-CLIP]

16.4 THE editor SHALL preserve the exact content of each Logical_Line derived from the clipboard without trimming or modifying whitespace. [FFE-CLIP]

16.5 WHEN the clipboard contains mixed line endings (e.g., some LF and some CRLF), THE editor SHALL split on any of the three standard line-ending sequences and normalise inserted lines to the document's configured line-ending style. [FFE-CLIP]

---

### Requirement 17: Command Registration and Shortcut Bindings [WB]

**User Story:** As a workbench developer, I want all clipboard operations registered as discoverable commands with default keyboard shortcuts, so that they integrate with the command framework's shortcut management, macro recording, and plugin extensibility.

#### Acceptance Criteria

17.1 THE clipboard subsystem SHALL register the following command IDs in the Command_Registry at startup: `"clipboard.copy"`, `"clipboard.cut"`, `"clipboard.paste"`, `"clipboard.copy-command"` (COPY primary command). [WB]

17.2 THE default shortcut bindings SHALL be: Ctrl+C → `"clipboard.copy"`, Ctrl+X → `"clipboard.cut"`, Ctrl+V → `"clipboard.paste"`. [WB, FFE-MVP-8]

17.3 THE shortcut bindings SHALL be overridable via user configuration in the Shortcut_Registry without modifying source code. [WB]

17.4 WHEN a clipboard command is invoked via the scripting bridge (Lua macro), THE command SHALL execute identically to keyboard invocation and SHALL produce the same Undo_Record. [WB]

17.5 WHEN a clipboard command is executed, THE command SHALL be logged in the Command_History for RETRIEVE command access. [WB]

---

### Requirement 18: Selection Interaction with Clipboard Operations [FFE-MVP-8]

**User Story:** As an editor user, I want clipboard operations to interact correctly with the current selection state — replacing selected text on paste, clearing selection after paste or cut — so that clipboard workflows feel natural.

#### Acceptance Criteria

18.1 WHEN a paste operation is triggered with an active stream selection, THE editor SHALL first delete the selected text, then insert the clipboard content at the resulting caret position, recording both as a single Undo_Record. [FFE-MVP-8]

18.2 WHEN a paste operation completes, THE editor SHALL place the caret at the end of the inserted content with no active selection. [FFE-MVP-8]

18.3 WHEN a copy operation completes, THE editor SHALL NOT modify or clear the current selection. [FFE-MVP-8]

18.4 WHEN a cut operation completes, THE editor SHALL place the caret at the position where the deleted text began with no active selection. [FFE-MVP-8]

18.5 WHEN the user starts typing or presses any navigation key after a paste, THE active selection (if any) SHALL be cleared. [FFE-MVP-8]

---

### Requirement 19: Configuration [WB]

**User Story:** As an editor administrator, I want clipboard behaviour to be configurable through the standard configuration system, so that I can control features like line-copy mode or clipboard timeout in managed environments.

#### Acceptance Criteria

19.1 THE Clipboard_Engine SHALL read an optional `clipboard.line_copy_when_no_selection` configuration key (boolean, default `true`); WHEN set to `false`, Ctrl+C with no selection SHALL do nothing rather than copying the current line. [WB]

19.2 THE Clipboard_Engine SHALL read an optional `clipboard.rectangular_paste_adds_lines` configuration key (boolean, default `true`); WHEN set to `false`, rectangular paste SHALL NOT create new lines beyond the end of the document but SHALL silently discard excess segments. [WB]

19.3 THE Clipboard_Engine SHALL read an optional `clipboard.access_timeout_ms` configuration key (positive integer, default `500`); WHEN a clipboard read or write exceeds this timeout, THE operation SHALL fail with a timeout error message rather than blocking indefinitely. [WB]

19.4 WHEN configuration keys contain invalid values, THE Clipboard_Engine SHALL log a warning and fall back to the documented defaults. [WB]
