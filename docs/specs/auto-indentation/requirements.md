# Requirements Document

## Introduction

This feature specifies the auto-indentation subsystem for FileForgeWorkbench (`ff-auto-indent` crate). The auto-indentation subsystem provides **language-aware automatic indentation** when inserting new lines, as well as explicit indent/unindent commands. It adapts SciTE's property-driven indent logic and block-pattern matching to Rust idioms, TOML-based language definitions, and the workbench's GUI-independent, command-driven architecture.

The subsystem covers four related concerns:

1. **Auto-indent on Enter** — when a new line is created, the new line's indentation matches or adjusts relative to the previous line based on language rules.
2. **Indent-increase patterns** — regex patterns that trigger an increase in indentation level for the next line (e.g., lines ending with `{`, `:`, `do`, `then`).
3. **Indent-decrease patterns** — regex patterns that trigger a decrease in indentation level for the current line (e.g., lines starting with `}`, `end`, `else`, `]`).
4. **Explicit Indent/Unindent commands** — Tab and Shift+Tab commands that increase or decrease indentation of selected lines.
5. **Block comment auto-continue** — when Enter is pressed inside a block or line comment, the new line is automatically prefixed with the appropriate comment continuation marker.

All indent rules are defined per-language in TOML language definition files (managed by `language-service`). The auto-indent logic operates on the document model without GUI coupling. Indent/Unindent commands are registered with the `command-framework`. All indent modifications are recorded as undoable transactions via `edit-operations`.

**Source references:**
- **[SCI-STE-INDENT]** = SciTE auto-indent properties: `indent.automatic`, `indent.opening`, `indent.closing`, `indent.maintain.*`, `statement.indent.*`, `statement.end.*`, `block.start.*`, `block.end.*`
- **[WB]** = Workbench Architecture Brief (GUI-independent core, command-driven architecture, configuration as data, per-language TOML definitions)

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `language-service` | **Dependency** | Provides the active language definition for the current document, including indent patterns, comment markers, and block delimiters defined in per-language TOML files. |
| `edit-operations` | **Integration** | The newline insertion (Requirement 2 in `edit-operations`) triggers auto-indent logic. Auto-indent modifications are recorded via the same EditorTransaction mechanism. The Indent/Unindent commands modify line content through the edit-operations API. |
| `document-model` | **Dependency** | Provides line content for pattern matching, line count, and the indent settings (tab size, indent size, use-tabs) that determine how indentation is physically represented. |
| `configuration-system` | **Dependency** | Stores global and per-language indent settings (e.g., `editor.auto_indent`, `editor.indent_size`, `editor.tab_size`, `editor.use_tabs`). Hot-reload propagates changes without restart. Language-specific overrides are defined in `languages/*.toml` files. |
| `command-framework` | **Integration** | The Indent and Unindent commands are registered as commands (`edit.indent`, `edit.unindent`) with default keybindings (Tab, Shift+Tab). |
| `undo-redo-transactions` | **Consumer** | All auto-indent modifications (whether triggered by Enter or by explicit commands) are wrapped in EditorTransactions so they can be undone as a single unit with the newline operation. |

## Glossary

- **Auto_Indent**: The automatic adjustment of indentation on a newly created line, triggered when the user presses Enter. The system determines the correct indent level based on the previous line's content and the language's indent rules. [SCI-STE-INDENT, WB]
- **Indent_Level**: The logical nesting depth of a line, measured in units of `indent_size` characters. Physical representation depends on the `use_tabs` setting. [WB]
- **Indent_Size**: The number of columns that constitute one logical indentation level (e.g., 4 spaces). Configurable per-language. [WB]
- **Tab_Size**: The display width of a tab character in columns. May differ from Indent_Size. [WB]
- **Use_Tabs**: A boolean setting that determines whether indentation is composed of tab characters (true) or space characters (false). [WB]
- **Indent_Increase_Pattern**: A regex pattern that, when matched against the content of the current line (typically its ending), causes the next line's indent to increase by one level. Example: `\{\s*$` matches lines ending with `{`. [SCI-STE-INDENT]
- **Indent_Decrease_Pattern**: A regex pattern that, when matched against the content of the newly typed line (typically its beginning), causes that line's indent to decrease by one level. Example: `^\s*\}` matches lines starting with `}`. [SCI-STE-INDENT]
- **Statement_Indent_Pattern**: A regex pattern identifying lines that begin a multi-line statement requiring indent continuation (e.g., `if`, `while`, `for` without braces). The next line is indented but subsequent lines return to the original level. [SCI-STE-INDENT]
- **Statement_End_Pattern**: A regex pattern identifying the end of a multi-line statement, signalling that subsequent lines should return to the pre-statement indent level. [SCI-STE-INDENT]
- **Block_Start_Pattern**: A language-defined regex identifying the opening of a block structure (e.g., `{`, `begin`, `do`). [SCI-STE-INDENT]
- **Block_End_Pattern**: A language-defined regex identifying the closing of a block structure (e.g., `}`, `end`). [SCI-STE-INDENT]
- **Indent_String**: The physical characters used to represent one indent level — either a single tab character or N space characters (where N = Indent_Size). [WB]
- **Comment_Continue_Marker**: The prefix automatically inserted at the start of a new line when Enter is pressed inside a comment block (e.g., ` * ` for C-style block comments, `// ` for line comment continuation). [SCI-STE-INDENT]
- **Maintain_Indent**: The simplest auto-indent behaviour — the new line receives exactly the same indentation as the previous line, regardless of content patterns. [SCI-STE-INDENT]
- **Smart_Indent**: Language-aware auto-indent that goes beyond Maintain_Indent by consulting Indent_Increase_Pattern and Indent_Decrease_Pattern to adjust the indent level. [SCI-STE-INDENT]
- **Indent_Transaction**: An EditorTransaction that groups the auto-indent adjustment with the newline insertion (or with the Indent/Unindent command) into a single undoable unit. [WB]

---

## Requirements

### Requirement 1: Auto-Indent Mode Configuration [WB, SCI-STE-INDENT]

**User Story:** As a workbench user, I want to control whether and how auto-indentation behaves, so that I can choose between no auto-indent, simple indent maintenance, or full language-aware smart indent depending on my preference and the file type.

#### Acceptance Criteria

1.1 THE auto-indentation subsystem SHALL support the following Auto_Indent modes: `None` (no automatic indentation applied), `Maintain` (new line matches the indentation of the previous line), and `Smart` (new line indentation is adjusted based on language-specific Indent_Increase_Pattern and Indent_Decrease_Pattern rules). [SCI-STE-INDENT, WB]

1.2 THE default Auto_Indent mode SHALL be `Smart` when a language definition with indent patterns is available for the active document, and `Maintain` otherwise. [SCI-STE-INDENT, WB]

1.3 THE Auto_Indent mode SHALL be configurable globally via the configuration-system key `editor.auto_indent` (accepting values `"none"`, `"maintain"`, `"smart"`) and overridable per-language in the language TOML definition file. [WB]

1.4 WHEN the configuration-system hot-reloads a change to `editor.auto_indent` or to the active language's indent settings, THE auto-indentation subsystem SHALL adopt the new mode for subsequent newline insertions without requiring a document close/reopen. [WB]

1.5 THE auto-indentation subsystem SHALL read `editor.indent_size`, `editor.tab_size`, and `editor.use_tabs` from the configuration-system (with per-language overrides) to determine the physical Indent_String used when inserting indentation. [WB]

1.6 WHEN EditorConfig settings are active for a file (detected by the configuration-system), THE EditorConfig values for `indent_style` and `indent_size` SHALL take precedence over the global configuration for that file. [WB]

---

### Requirement 2: Maintain Indent (Basic Auto-Indent) [SCI-STE-INDENT]

**User Story:** As an editor user, I want new lines to automatically receive the same indentation as the line I pressed Enter on, so that I don't have to manually type leading whitespace for every new line.

#### Acceptance Criteria

2.1 WHEN the user presses Enter and the Auto_Indent mode is `Maintain`, THE auto-indentation subsystem SHALL set the new line's indentation to exactly match the leading whitespace of the line where Enter was pressed (the "reference line"). [SCI-STE-INDENT]

2.2 WHEN determining the reference line's indentation, THE system SHALL count all leading whitespace characters (spaces and tabs) up to the first non-whitespace character, interpreting tab characters using the configured Tab_Size for column calculation. [SCI-STE-INDENT]

2.3 WHEN reproducing the indentation on the new line, THE system SHALL generate the Indent_String according to the `use_tabs` setting: if true, use tab characters (with spaces for partial tab stops); if false, use space characters. [WB]

2.4 THE auto-indent whitespace insertion SHALL be part of the same EditorTransaction as the newline insertion in `edit-operations`, so that a single Undo command removes both the new line and its auto-indentation. [WB]

2.5 WHEN Enter is pressed at the beginning of a line (caret at column 0), THE new line SHALL have zero indentation regardless of the reference line's indentation. [SCI-STE-INDENT]

2.6 WHEN Enter is pressed in the middle of indentation (caret within the leading whitespace), THE new line SHALL receive indentation equal to the whitespace before the caret position, not the full indentation of the reference line. [SCI-STE-INDENT]

---

### Requirement 3: Smart Indent — Indent Increase [SCI-STE-INDENT]

**User Story:** As a developer, I want the editor to automatically increase the indent level after I type a block-opening construct (like `{` or `do`), so that the next line starts at the correct nesting depth without manual adjustment.

#### Acceptance Criteria

3.1 WHEN the user presses Enter and the Auto_Indent mode is `Smart`, THE auto-indentation subsystem SHALL examine the content of the reference line (the line where Enter was pressed) and, if it matches the active language's Indent_Increase_Pattern, increase the new line's indentation by one Indent_Level relative to the reference line's indentation. [SCI-STE-INDENT]

3.2 THE Indent_Increase_Pattern SHALL be defined per-language in the language TOML definition file under the key `indent.increase_pattern` as a regex string. [WB, SCI-STE-INDENT]

3.3 WHEN no Indent_Increase_Pattern is defined for the active language, THE system SHALL fall back to Maintain_Indent behaviour for indent-increase detection (no automatic increase). [SCI-STE-INDENT]

3.4 THE Indent_Increase_Pattern SHALL be matched against the non-comment, non-string portion of the line text (using the syntax state from `language-service` to exclude content inside string literals and comments from pattern matching). [SCI-STE-INDENT]

3.5 WHEN the reference line matches both an Indent_Increase_Pattern and an Indent_Decrease_Pattern (e.g., `} else {`), THE system SHALL apply the net effect: one decrease and one increase cancel out, resulting in the same indent level as the reference line. [SCI-STE-INDENT]

3.6 THE system SHALL support a Statement_Indent_Pattern per-language (key `indent.statement_pattern`) that identifies lines beginning a statement continuation (e.g., `if (...)` without `{`). When matched, only the immediately following line is indented by one level; subsequent lines return to the original indent. [SCI-STE-INDENT]

---

### Requirement 4: Smart Indent — Indent Decrease [SCI-STE-INDENT]

**User Story:** As a developer, I want the editor to automatically decrease the indent level when I type a block-closing construct (like `}` or `end`), so that closing delimiters align with their corresponding opening constructs without manual adjustment.

#### Acceptance Criteria

4.1 WHEN the user types a character that completes a match against the active language's Indent_Decrease_Pattern on the current line, THE auto-indentation subsystem SHALL immediately reduce the current line's indentation by one Indent_Level. [SCI-STE-INDENT]

4.2 THE Indent_Decrease_Pattern SHALL be defined per-language in the language TOML definition file under the key `indent.decrease_pattern` as a regex string. [WB, SCI-STE-INDENT]

4.3 WHEN no Indent_Decrease_Pattern is defined for the active language, THE system SHALL not perform any automatic indent decrease. [SCI-STE-INDENT]

4.4 THE indent decrease adjustment SHALL be recorded as an EditorTransaction so that Undo reverses both the typed character and the indent adjustment as a single unit. [WB]

4.5 THE Indent_Decrease_Pattern SHALL be evaluated against the line content including only leading whitespace and the characters typed so far on the line, so that the decrease triggers as soon as the pattern is completed (e.g., typing `}` at the start of a line triggers immediately). [SCI-STE-INDENT]

4.6 WHEN an indent-decrease is triggered, THE system SHALL NOT reduce the indentation below zero (column 0). [SCI-STE-INDENT]

4.7 WHEN the user types a closing delimiter that matches the Indent_Decrease_Pattern and the line already has content before the caret (i.e., it is not a fresh line with only whitespace), THE system SHALL NOT adjust the line's indentation (decrease only applies to lines whose only content prior to the trigger is whitespace). [SCI-STE-INDENT]

---

### Requirement 5: Smart Indent — Enter After Block-Start/End Patterns [SCI-STE-INDENT]

**User Story:** As a developer, I want pressing Enter between an opening and closing brace (e.g., `{|}`) to automatically create a properly indented blank line between them, so that I can immediately start typing inside the block.

#### Acceptance Criteria

5.1 WHEN the user presses Enter and the caret is positioned immediately between a Block_Start_Pattern match and a Block_End_Pattern match on the same line (e.g., `{}`), THE system SHALL: (a) split the line at the caret, (b) indent the new line by one Indent_Level relative to the opening line, and (c) insert a third line with the closing delimiter at the original indent level. [SCI-STE-INDENT]

5.2 THE Block_Start_Pattern and Block_End_Pattern SHALL be defined per-language in the language TOML definition file under keys `indent.block_start` and `indent.block_end`. [WB, SCI-STE-INDENT]

5.3 WHEN the Enter-between-braces expansion creates multiple lines, THE entire operation (line split + indent adjustment + extra line) SHALL be recorded as a single EditorTransaction. [WB]

5.4 WHEN no Block_Start_Pattern or Block_End_Pattern is defined for the active language, THE system SHALL not perform the between-braces expansion (falling back to standard smart indent behaviour). [SCI-STE-INDENT]

5.5 THE caret SHALL be positioned at the end of the indentation on the middle (indented) line after the between-braces expansion, ready for the user to type block content. [SCI-STE-INDENT]

---

### Requirement 6: Block Comment Auto-Continue [SCI-STE-INDENT]

**User Story:** As a developer writing multi-line comments, I want the editor to automatically insert comment continuation markers when I press Enter inside a comment, so that I can write documentation and comments fluidly without manually typing `*` or `//` on each line.

#### Acceptance Criteria

6.1 WHEN the user presses Enter inside a block comment (e.g., between `/*` and `*/`), THE auto-indentation subsystem SHALL insert the language-defined Comment_Continue_Marker (e.g., ` * `) at the beginning of the new line, aligned with the comment above. [SCI-STE-INDENT]

6.2 WHEN the user presses Enter on a line that is a line-comment (e.g., starts with `//`), and the `comment.continue_line` setting is enabled for the active language, THE system SHALL prefix the new line with the same line-comment marker followed by a space (e.g., `// `). [SCI-STE-INDENT]

6.3 THE comment continuation markers SHALL be defined per-language in the language TOML definition file under keys `comment.block_continue` (e.g., `" * "`) and `comment.line_prefix` (e.g., `"// "`). [WB, SCI-STE-INDENT]

6.4 WHEN the user presses Enter on the last line of a block comment (the line containing `*/` or equivalent), THE system SHALL NOT insert a continuation marker on the new line (the comment has ended). [SCI-STE-INDENT]

6.5 THE comment continuation insertion SHALL be part of the same EditorTransaction as the newline, so a single Undo removes both the new line and the continuation marker. [WB]

6.6 WHEN the reference line's only non-whitespace content is the comment continuation marker (e.g., a line containing just ` * ` with no text after it) and the user presses Enter, THE system SHALL insert the new line with the continuation marker AND, if the user immediately presses Enter again (resulting in two blank comment lines), THE system SHALL remove the continuation marker from the previous line (allowing the user to "break out" of comment continuation by pressing Enter twice on an empty comment line). [SCI-STE-INDENT]

6.7 THE comment-continue detection SHALL consult the syntax highlighting state from `language-service` to determine whether the caret is inside a comment, rather than relying solely on text pattern matching. [SCI-STE-INDENT]

---

### Requirement 7: Indent Command (Tab) [WB, SCI-STE-INDENT]

**User Story:** As an editor user, I want to press Tab to increase the indentation of selected lines, so that I can quickly adjust code structure and nesting.

#### Acceptance Criteria

7.1 WHEN one or more complete lines are selected and the user presses Tab (or invokes the `edit.indent` command), THE system SHALL increase the indentation of every selected line by one Indent_Level (prepending one Indent_String to each line). [WB, SCI-STE-INDENT]

7.2 WHEN no selection is active (or the selection does not span multiple lines) and the user presses Tab, THE system SHALL delegate to the normal Tab insertion behaviour defined in `edit-operations` (insert tab character or spaces to next tab stop). [WB]

7.3 THE `edit.indent` command SHALL be registered with the command-framework with the default keybinding `Tab` and the display name "Indent". [WB]

7.4 THE indent operation SHALL be recorded as a single EditorTransaction covering all affected lines, so that one Undo command reverses the entire indent operation. [WB]

7.5 WHEN indenting lines with mixed leading whitespace (tabs and spaces), THE system SHALL normalise the indentation to the current `use_tabs` setting before adding the new indent level. [WB]

7.6 THE indent command SHALL set the modified line marker on every affected line. [WB]

---

### Requirement 8: Unindent Command (Shift+Tab) [WB, SCI-STE-INDENT]

**User Story:** As an editor user, I want to press Shift+Tab to decrease the indentation of selected lines, so that I can quickly adjust code structure and reduce nesting.

#### Acceptance Criteria

8.1 WHEN one or more complete lines are selected and the user presses Shift+Tab (or invokes the `edit.unindent` command), THE system SHALL decrease the indentation of every selected line by one Indent_Level (removing one Indent_String from the beginning of each line). [WB, SCI-STE-INDENT]

8.2 WHEN a line's current indentation is less than one full Indent_Level, THE system SHALL remove all remaining leading whitespace (indentation cannot go below zero). [WB, SCI-STE-INDENT]

8.3 WHEN the caret is on a single line with no selection, the `edit.unindent` command SHALL unindent that single line. [WB]

8.4 THE `edit.unindent` command SHALL be registered with the command-framework with the default keybinding `Shift+Tab` and the display name "Unindent". [WB]

8.5 THE unindent operation SHALL be recorded as a single EditorTransaction covering all affected lines, so that one Undo command reverses the entire unindent operation. [WB]

8.6 THE unindent command SHALL set the modified line marker on every affected line whose indentation was actually changed (lines already at column 0 are not marked). [WB]

8.7 WHEN unindenting, THE system SHALL remove indentation considering both tab characters and space characters: one tab character counts as Tab_Size columns, and spaces are removed up to Indent_Size columns per unindent level. [WB, SCI-STE-INDENT]

---

### Requirement 9: Language-Specific Indent Rules via TOML Definitions [WB, SCI-STE-INDENT]

**User Story:** As a workbench developer or power user, I want indent rules to be defined per-language in TOML files, so that each programming language gets contextually correct auto-indentation without requiring code changes to the editor.

#### Acceptance Criteria

9.1 THE auto-indentation subsystem SHALL read indent rules from the active language's TOML definition file (resolved by `language-service`), under the `[indent]` table. [WB, SCI-STE-INDENT]

9.2 THE language TOML `[indent]` table SHALL support the following keys: `increase_pattern` (regex string), `decrease_pattern` (regex string), `statement_pattern` (regex string, optional), `statement_end_pattern` (regex string, optional), `block_start` (regex string, optional), `block_end` (regex string, optional). [SCI-STE-INDENT]

9.3 THE language TOML `[indent]` table SHALL support override keys: `indent_size` (integer), `tab_size` (integer), `use_tabs` (boolean) that take precedence over the global `editor.*` settings for files of that language. [WB]

9.4 THE language TOML `[comment]` table SHALL support keys: `block_start` (string, e.g., `"/*"`), `block_end` (string, e.g., `"*/"`), `block_continue` (string, e.g., `" * "`), `line_prefix` (string, e.g., `"//"`), `continue_line` (boolean, default false). [WB, SCI-STE-INDENT]

9.5 WHEN the active language for a document changes (e.g., via manual language selection or re-detection by `language-service`), THE auto-indentation subsystem SHALL immediately adopt the new language's indent rules for subsequent operations. [WB]

9.6 WHEN a language definition file does not contain an `[indent]` table, THE system SHALL use the global `editor.*` indent settings with Maintain_Indent behaviour (no pattern-based smart indent). [WB, SCI-STE-INDENT]

9.7 ALL regex patterns in language indent rules SHALL use Rust's `regex` crate syntax and SHALL be compiled and cached at language load time. Invalid patterns SHALL be logged as WARN and the pattern SHALL be treated as non-matching. [WB]

---

### Requirement 10: Integration with Edit Operations and Undo [WB]

**User Story:** As an editor user, I want auto-indentation to be seamlessly integrated with the normal editing flow — undoable, non-disruptive, and invisible when I don't want it — so that it enhances productivity without interfering with manual formatting choices.

#### Acceptance Criteria

10.1 WHEN auto-indent is triggered by a newline insertion, THE auto-indent modification SHALL be grouped into the same EditorTransaction as the newline operation in `edit-operations`, so that Ctrl+Z undoes both the newline and the auto-indent in a single step. [WB]

10.2 WHEN the user immediately edits the auto-indented whitespace after it is inserted (e.g., pressing Backspace to reduce indent), THE system SHALL NOT fight the user — no re-indentation shall be triggered by whitespace edits on the same line within the same editing session. [WB]

10.3 WHEN the Auto_Indent mode is `None`, THE system SHALL not modify line content in response to Enter presses — the new line shall start at column 0 with no leading whitespace. [SCI-STE-INDENT]

10.4 THE auto-indentation subsystem SHALL operate purely on the document model (line content and metadata) without requiring access to any GUI components. The GUI shell triggers auto-indent through the `edit-operations` API; the subsystem returns the indentation to apply. [WB]

10.5 WHEN multiple carets are active and Enter is pressed, THE auto-indentation subsystem SHALL compute and apply the correct indentation independently for each caret's new line, using each caret's reference line context. All caret indentations SHALL be part of the same UndoGroup. [WB]

10.6 WHEN the Indent or Unindent command is invoked and a rectangular selection is active, THE system SHALL indent or unindent all lines spanned by the rectangular selection. [WB]

10.7 THE auto-indentation subsystem SHALL emit a DEBUG-level log record for each auto-indent decision, including the reference line number, matched pattern (if any), and resulting indent level. [WB]
