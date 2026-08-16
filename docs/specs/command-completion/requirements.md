# Requirements Document

## Introduction

This feature specifies the **Command Completion** subsystem for FileForgeWorkbench (`ff-command-completion` crate). The command completion system provides an **auto-complete popup** for the primary command field and the line-command prefix area, offering context-sensitive suggestions as the user types. It adapts the Scintilla AutoComplete concepts to the ISPF-inspired command-driven workbench architecture.

The command completion subsystem is responsible for:

1. **Primary command name completion** — prefix-matching against all registered commands in the `command-framework` Command_Registry
2. **Argument completion** — context-sensitive completion for command arguments: file paths (via VFS), command modifiers/keywords, macro names, and configurable keyword lists
3. **Line command completion** — prefix-area auto-complete for ISPF line command abbreviations
4. **Popup positioning** — intelligent placement of the completion popup above or below the command field based on available screen space
5. **Navigation and selection** — keyboard-driven list navigation with Arrow keys, Tab, and Enter for selection; Escape for dismiss
6. **Fuzzy matching** — optional fuzzy/subsequence matching mode as an alternative to strict prefix matching
7. **Trigger control** — configurable activation: manual (Ctrl+Space) and/or automatic after N characters typed

The crate is **GUI-independent** in its core logic (candidate generation, filtering, ranking, and selection management). The popup rendering and positioning coordinate with the GUI shell (egui) but the completion engine can be tested without a running UI.

### Design Principles

1. **GUI-independent core.** The completion engine (candidate generation, matching, ranking, selection state) has no GUI dependency. Only the popup positioning and rendering adapt to the GUI shell. [WB]
2. **Command-framework integration.** Completions for command names are sourced directly from the `CommandRegistry`, ensuring they always reflect the current set of registered commands (including plugin-contributed commands). [WB]
3. **VFS-aware path completion.** File path arguments are resolved through the VFS abstraction, supporting any registered provider (local, catalog, future remotes). [WB, FFW-ARCH-001]
4. **Non-blocking.** Completion candidates are generated asynchronously where necessary (e.g., VFS directory listing). The popup never blocks the UI thread. [WB]
5. **Configurable activation.** The user controls whether completion triggers automatically or only on demand, and can tune the trigger threshold. [SCI-AC]
6. **Extensible providers.** Plugins and macros can register custom completion providers for their own command arguments via the plugin architecture. [WB]

### Source References

- **[SCI-AC]** = Scintilla AutoComplete concepts — list box positioning, stop chars, fill-up chars, case-insensitive matching, sort order, auto-hide behaviour
- **[WB]** = Workbench Architecture Brief — command-driven architecture, GUI-independence, VFS principle, plugin extensibility
- **[FFE-CMD-1]** = FileForgeEditor core-command-semantics — command execution pipeline (primary command field context)
- **[FFE-CMD-37]** = FileForgeEditor core-command-semantics — line command parser (prefix-area context)

### Cross-References

- **`command-framework`** — Provides the Command_Registry from which command name completions are sourced; provides Command_Metadata (display name, category, description) for tooltip enrichment in the popup.
- **`command-semantics`** — Defines the primary command field, command parsing, and scope modifiers that completion must understand for argument context.
- **`virtual-file-system`** — Supplies file path completions via async directory listing through VFS provider-agnostic API.
- **`lua-macro-engine`** — Provides the list of registered macro names for macro name completion.
- **`configuration-system`** — Stores completion behaviour settings (trigger mode, threshold, fuzzy mode, max items, popup dimensions) under the `completion.*` namespace.
- **`line-commands`** — Defines the set of valid line commands (kinds, block forms) used as the candidate list for prefix-area completion.

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **Completion_Engine** | The core logic component that manages candidate generation, filtering, ranking, and selection state for auto-complete. GUI-independent. | [SCI-AC], [WB] |
| **Completion_Popup** | The visual overlay widget that displays the filtered list of completion candidates near the command field or prefix area. | [SCI-AC] |
| **Completion_Candidate** | A single item in the completion list, comprising at minimum a label (display text) and an insertion value. May also include a category, icon, and description. | [SCI-AC] |
| **Completion_Provider** | A trait that supplies completion candidates for a given context. Built-in providers cover command names, file paths, keywords, line commands, and macro names. Plugins may register custom providers. | [WB] |
| **Completion_Context** | The state at the moment completion is triggered: the field being edited (primary command or prefix area), the text typed so far, the cursor position, and the parsed command name (if arguments are being completed). | [SCI-AC] |
| **Prefix_Match** | A matching mode where candidates must start with the typed prefix (case-insensitive by default). | [SCI-AC] |
| **Fuzzy_Match** | A matching mode where candidates are matched by subsequence — all typed characters must appear in order within the candidate, but not necessarily consecutively. | [SCI-AC] |
| **Trigger_Mode** | The configured activation mode: `manual` (Ctrl+Space only), `automatic` (after N characters), or `both` (automatic triggers and manual override). | [SCI-AC] |
| **Trigger_Threshold** | The number of characters the user must type before automatic completion triggers. Configurable via `completion.auto_trigger_chars`. | [SCI-AC] |
| **Stop_Char** | A character that, when typed, causes the completion popup to dismiss automatically. | [SCI-AC] |
| **Fill_Up_Char** | A character that, when typed while a candidate is selected, accepts the candidate AND inserts the typed character. | [SCI-AC] |
| **Popup_Anchor** | The screen position (x, y) at which the completion popup is anchored, typically the start of the prefix being completed. | [SCI-AC] |
| **Max_Visible_Items** | The maximum number of candidates displayed in the popup before scrolling is required. Configurable. | [SCI-AC] |

---

## Requirements

### Requirement 1: Primary Command Name Completion

**User Story:** As an editor user, I want the completion system to suggest matching command names as I type in the primary command field, so that I can discover and invoke commands quickly without memorizing exact names.

**Source:** [SCI-AC], [WB]

#### Acceptance Criteria

1.1. WHEN completion is triggered in the primary command field AND the typed text represents the first token (command name position), THE Completion_Engine SHALL query the `command-framework` Command_Registry for all registered commands and filter them against the typed prefix.

1.2. WHEN filtering command names, THE Completion_Engine SHALL perform case-insensitive Prefix_Match by default (e.g., typing `fi` SHALL match `FIND`, `FILE.SAVE`, `FILTER`).

1.3. EACH Completion_Candidate for a command name SHALL include: the command name as the label, the Command_ID as the insertion value, the category (from Command_Metadata), and the display name or description (from Command_Metadata) for tooltip display.

1.4. THE Completion_Engine SHALL rank command name candidates by relevance: exact prefix matches before substring matches, shorter names before longer names at equal prefix length, and recently-used commands before rarely-used commands (frequency weighting from Command_History if available).

1.5. WHEN a command name candidate is accepted (selected and confirmed), THE Completion_Engine SHALL replace the typed prefix in the command field with the canonical command name (uppercase form as registered in Command_Registry) and position the cursor after the inserted name with a trailing space.

1.6. THE Completion_Engine SHALL update the candidate list dynamically as the user continues typing — each additional character SHALL re-filter the list without requiring a new trigger action.

1.7. IF no registered command names match the typed prefix, THEN THE Completion_Popup SHALL auto-hide (dismiss itself) rather than displaying an empty list.

---

### Requirement 2: Argument Completion

**User Story:** As an editor user, I want context-sensitive completions for command arguments (file paths, modifiers, keyword values), so that I can construct complex commands efficiently without looking up valid argument values in documentation.

**Source:** [SCI-AC], [WB]

#### Acceptance Criteria

2.1. WHEN completion is triggered in the primary command field AND the cursor is positioned after the command name (argument position), THE Completion_Engine SHALL determine the argument context by consulting the parsed command name and its argument schema (if available from Command_Metadata or a registered Completion_Provider).

2.2. WHEN the argument context indicates a file path is expected, THE Completion_Engine SHALL query the VFS abstraction layer for matching file and directory entries, using the typed text as a path prefix. The query SHALL be asynchronous and SHALL NOT block the UI thread.

2.3. WHEN completing VFS file paths, THE Completion_Engine SHALL support both bare paths (routed to the default local provider) and full Resource_URIs (`vfs://provider/path`), filtering the provider portion first and then the path portion.

2.4. WHEN the argument context indicates a command modifier or keyword is expected (e.g., scope modifiers VISIBLE/EXCLUDED/ALL, FIND modifiers CHARS/PREFIX/SUFFIX/WORD), THE Completion_Engine SHALL offer the valid keyword set for that argument position as candidates.

2.5. WHEN the argument context indicates a macro name is expected (e.g., `MACRO RUN <name>`), THE Completion_Engine SHALL query the Lua macro engine for the list of registered/available macro names and offer them as candidates.

2.6. PLUGINS SHALL be able to register custom Completion_Provider implementations for their own commands via the plugin architecture. THE Completion_Engine SHALL invoke registered providers based on the Command_ID of the command being completed.

2.7. WHEN multiple Completion_Providers are applicable for an argument position (e.g., a command accepts either a file path or a keyword), THE Completion_Engine SHALL merge candidates from all applicable providers, de-duplicate by insertion value, and present a unified list ranked by relevance.

2.8. IF no Completion_Provider is registered for a command's arguments, THEN THE Completion_Engine SHALL offer no argument completions for that command and the popup SHALL NOT appear in argument position.

---

### Requirement 3: Popup Positioning

**User Story:** As an editor user, I want the completion popup to appear near the text I'm typing without obscuring the command field or extending off-screen, so that I can read both my typed input and the suggested completions simultaneously.

**Source:** [SCI-AC]

#### Acceptance Criteria

3.1. THE Completion_Popup SHALL be anchored horizontally at the Popup_Anchor — the x-coordinate corresponding to the start of the prefix being completed within the command field.

3.2. THE Completion_Popup SHALL be positioned vertically below the command field by default, with the top edge of the popup adjacent to the bottom edge of the command field.

3.3. IF positioning the popup below the command field would cause it to extend beyond the bottom edge of the application window, THEN THE Completion_Popup SHALL flip to appear above the command field (bottom edge of popup adjacent to top edge of command field).

3.4. IF positioning the popup in the flipped (above) direction would also extend beyond the top edge of the window, THEN THE Completion_Popup SHALL choose whichever direction (above or below) provides more visible area and SHALL clip to the available space with internal scrolling.

3.5. THE Completion_Popup SHALL NOT overlap or obscure the command field text where the user is actively typing.

3.6. THE Completion_Popup width SHALL be at least as wide as the longest visible candidate label, bounded by a configurable maximum width (`completion.popup_max_width`). IF candidates are wider than the maximum width, THEN candidate labels SHALL be truncated with an ellipsis.

3.7. THE Completion_Popup height SHALL display up to `completion.popup_max_items` candidates (configurable, default 10), using vertical scrolling if more candidates are available.

3.8. WHEN the application window is resized or moved while the popup is visible, THE Completion_Popup SHALL recompute its position to remain correctly anchored and within bounds.

---

### Requirement 4: Selection and Navigation

**User Story:** As an editor user, I want to navigate the completion list with keyboard shortcuts and accept a selection efficiently, so that completion accelerates my workflow rather than interrupting it.

**Source:** [SCI-AC]

#### Acceptance Criteria

4.1. WHEN the Completion_Popup is visible, THE Down Arrow key SHALL move the selection highlight to the next candidate in the list (wrapping from last to first if `completion.wrap_navigation` is true, stopping at the last item otherwise).

4.2. WHEN the Completion_Popup is visible, THE Up Arrow key SHALL move the selection highlight to the previous candidate in the list (wrapping from first to last if `completion.wrap_navigation` is true, stopping at the first item otherwise).

4.3. WHEN the Completion_Popup is visible, THE Tab key SHALL accept the currently highlighted candidate — inserting its value into the command field, replacing the typed prefix, and dismissing the popup.

4.4. WHEN the Completion_Popup is visible, THE Enter key SHALL accept the currently highlighted candidate (same behaviour as Tab) AND submit the command for execution if the cursor is at the end of the command field and no further arguments are expected.

4.5. WHEN the Completion_Popup is visible, THE Escape key SHALL dismiss the popup without accepting any candidate, leaving the typed text unchanged.

4.6. WHEN the Completion_Popup is visible AND the user types a Stop_Char (configurable via `completion.stop_chars`, default: space, semicolon), THE popup SHALL dismiss without accepting a candidate.

4.7. WHEN the Completion_Popup is visible AND the user types a Fill_Up_Char (configurable via `completion.fill_up_chars`, default: none), THE popup SHALL accept the currently highlighted candidate AND insert the Fill_Up_Char after the accepted text.

4.8. WHEN the Completion_Popup is visible, THE Page Down key SHALL move the selection by one page (Max_Visible_Items count) downward; THE Page Up key SHALL move the selection by one page upward.

4.9. WHEN only a single candidate matches the typed prefix AND `completion.choose_single` is true (configurable, default false), THE Completion_Engine SHALL auto-accept that candidate without showing the popup.

4.10. WHEN a candidate is accepted, THE insertion SHALL replace only the prefix portion that was used to filter — any text after the cursor in the command field SHALL be preserved.

---

### Requirement 5: Dismiss Behaviour

**User Story:** As an editor user, I want the completion popup to disappear automatically when it's no longer relevant (e.g., I've moved past the completion point, clicked elsewhere, or the field lost focus), so that it never obstructs my work.

**Source:** [SCI-AC]

#### Acceptance Criteria

5.1. WHEN the Escape key is pressed while the Completion_Popup is visible, THE popup SHALL dismiss immediately without modifying the command field content.

5.2. WHEN the command field loses keyboard focus (e.g., user clicks in the editor area, switches tabs, or navigates to another panel), THE Completion_Popup SHALL dismiss.

5.3. WHEN the cursor moves to a position at or before the original Popup_Anchor position (start of the prefix when completion was triggered) AND `completion.cancel_at_start_pos` is true (configurable, default true), THE Completion_Popup SHALL dismiss.

5.4. WHEN the user deletes characters such that the remaining prefix produces zero matching candidates AND `completion.auto_hide` is true (configurable, default true), THE Completion_Popup SHALL dismiss.

5.5. WHEN the user submits the command (presses Enter to execute), THE Completion_Popup SHALL dismiss regardless of selection state.

5.6. WHEN a different completion is triggered (e.g., user moves to a different argument position and triggers completion again), THE existing popup SHALL close and a new popup SHALL open for the new context.

---

### Requirement 6: Fuzzy Matching

**User Story:** As an editor user, I want an optional fuzzy matching mode that finds commands and arguments even when I don't remember the exact prefix, so that I can locate items by typing any subsequence of characters in their name.

**Source:** [SCI-AC]

#### Acceptance Criteria

6.1. WHEN `completion.matching_mode` is set to `"fuzzy"`, THE Completion_Engine SHALL use Fuzzy_Match: a candidate matches if all characters of the typed text appear in the candidate label in the same order, but not necessarily consecutively (e.g., typing `fs` matches `file.save`, `find.scope`).

6.2. WHEN `completion.matching_mode` is set to `"prefix"` (the default), THE Completion_Engine SHALL use strict Prefix_Match: a candidate matches only if it starts with the typed text.

6.3. WHEN fuzzy matching is active, THE Completion_Engine SHALL highlight the matched character positions within each candidate's label in the popup, providing visual feedback about why each candidate matched.

6.4. WHEN fuzzy matching is active, THE Completion_Engine SHALL rank candidates by match quality: candidates where matched characters are closer together (fewer gaps) SHALL rank higher; candidates where the match starts at the beginning of the label SHALL rank higher than mid-word matches.

6.5. THE `completion.matching_mode` configuration key SHALL accept values `"prefix"` and `"fuzzy"`. IF an invalid value is provided, THE Completion_Engine SHALL fall back to `"prefix"` and log a WARN-level record.

6.6. WHEN `completion.matching_mode` is `"prefix"`, THE Completion_Engine SHALL still perform case-insensitive matching (uppercase/lowercase treated as equivalent) by default; case sensitivity SHALL be controlled by `completion.case_sensitive` (boolean, default false).

---

### Requirement 7: Line Command Completion

**User Story:** As an editor user, I want auto-complete suggestions in the prefix area when typing line commands, so that I can discover and accurately enter line commands (D, DD, CC, M5, etc.) without memorizing all valid forms.

**Source:** [SCI-AC], [FFE-CMD-37]

#### Acceptance Criteria

7.1. WHEN completion is triggered in the prefix area (line command input), THE Completion_Engine SHALL offer all valid line command kinds (C, CC, M, MM, D, DD, R, RR, X, XX, I, A, B, O, W, S, T, TT, U, UU, >, >>, <, <<, ), )), (, (( ) as candidates, filtered by the typed prefix.

7.2. EACH line command Completion_Candidate SHALL include the command kind as the label and a short description of its action (e.g., `CC` → "Copy block start/end", `D5` → "Delete 5 lines").

7.3. WHEN the user has typed alphabetic characters in the prefix area, THE Completion_Engine SHALL filter the line command list to show only commands whose kind starts with the typed characters (case-insensitive).

7.4. WHEN a line command candidate is accepted, THE Completion_Engine SHALL replace the prefix area content with the selected command kind, preserving any numeric count already typed after the kind.

7.5. THE line command completion trigger SHALL follow the same Trigger_Mode settings as primary command completion (`completion.trigger_mode`, `completion.auto_trigger_chars`).

7.6. IF `completion.line_command_completion` is set to `false` (configurable, default true), THEN THE Completion_Engine SHALL NOT offer completion in the prefix area.

---

### Requirement 8: Macro Name Completion

**User Story:** As an editor user, I want auto-complete for macro names when invoking macros from the command line, so that I can quickly find and run registered Lua macros without remembering exact file names.

**Source:** [WB]

#### Acceptance Criteria

8.1. WHEN the parsed command is a macro invocation (e.g., the command name matches a registered macro name pattern, or the user is typing after `MACRO RUN` or equivalent), THE Completion_Engine SHALL query the Lua macro engine for all available macro names and offer them as candidates.

8.2. THE macro name candidate list SHALL include: the macro name (without file extension) as the label, the macro file path as supplementary info, and a brief description if the macro provides one in its metadata.

8.3. WHEN the Lua macro engine reports that macros have been added, removed, or reloaded, THE Completion_Engine SHALL refresh its cached macro name list to reflect the current state.

8.4. THE macro name completion SHALL apply the same matching mode (prefix or fuzzy) as configured for command name completion via `completion.matching_mode`.

8.5. IF no macros are registered with the Lua macro engine, THEN macro name completion SHALL produce an empty candidate list and the popup SHALL NOT appear.

---

### Requirement 9: Configurable Trigger Behaviour

**User Story:** As an editor user, I want to control when completion activates — either manually on demand or automatically as I type — so that I can balance discoverability against minimal disruption to my typing flow.

**Source:** [SCI-AC]

#### Acceptance Criteria

9.1. THE Configuration_System SHALL support the following completion-related keys under the `completion.*` namespace:
- `completion.trigger_mode` (string: `"manual"` | `"automatic"` | `"both"`) — activation mode. Default: `"both"`.
- `completion.auto_trigger_chars` (integer, 1–10) — character count threshold for automatic triggering. Default: `2`.
- `completion.matching_mode` (string: `"prefix"` | `"fuzzy"`) — matching algorithm. Default: `"prefix"`.
- `completion.case_sensitive` (boolean) — whether matching is case-sensitive. Default: `false`.
- `completion.popup_max_items` (integer, 3–50) — maximum visible candidates in popup. Default: `10`.
- `completion.popup_max_width` (integer, 100–1000 logical pixels) — maximum popup width. Default: `400`.
- `completion.auto_hide` (boolean) — dismiss when no matches. Default: `true`.
- `completion.cancel_at_start_pos` (boolean) — dismiss when cursor retreats past anchor. Default: `true`.
- `completion.choose_single` (boolean) — auto-accept lone match. Default: `false`.
- `completion.wrap_navigation` (boolean) — wrap arrow navigation. Default: `true`.
- `completion.stop_chars` (string) — characters that dismiss the popup. Default: `" ;"`.
- `completion.fill_up_chars` (string) — characters that accept selection. Default: `""` (none).
- `completion.line_command_completion` (boolean) — enable prefix-area completion. Default: `true`.
- `completion.drop_rest_of_word` (boolean) — whether accepting a candidate removes text after the cursor up to the next word boundary. Default: `false`.

9.2. WHEN `completion.trigger_mode` is `"manual"`, THE Completion_Engine SHALL only activate when the user explicitly presses the trigger shortcut (Ctrl+Space by default, configurable via the Shortcut_Registry).

9.3. WHEN `completion.trigger_mode` is `"automatic"`, THE Completion_Engine SHALL activate automatically after the user has typed at least `completion.auto_trigger_chars` consecutive characters in the command field without a pause, AND SHALL also respond to the manual trigger shortcut.

9.4. WHEN `completion.trigger_mode` is `"both"`, THE Completion_Engine SHALL support both automatic triggering (after threshold) and manual triggering (Ctrl+Space), equivalent to `"automatic"` mode.

9.5. WHEN a configuration value is invalid (out of range, wrong type, or unknown enum variant), THE Completion_Engine SHALL fall back to the defined default for that key and SHALL write a WARN-level log record via the logging subsystem indicating the invalid value and the fallback being used.

9.6. THE Completion_Engine SHALL re-read configuration values when the Configuration_System emits a hot-reload notification, applying updated settings to subsequent completion activations without requiring application restart.

9.7. THE manual trigger shortcut (default Ctrl+Space) SHALL be registered in the command-framework Shortcut_Registry as Command_ID `"completion.trigger"` and SHALL be user-customizable through the standard key map configuration.

---

### Requirement 10: Completion Provider Extensibility

**User Story:** As a plugin developer, I want to register custom completion providers for my plugin's commands, so that users get context-aware auto-complete for arguments specific to my plugin without modifying the core completion engine.

**Source:** [WB]

#### Acceptance Criteria

10.1. THE Completion_Engine SHALL define a `CompletionProvider` trait with the following method: `fn provide_candidates(context: &CompletionContext) -> Vec<CompletionCandidate>` (or its async equivalent), where `CompletionContext` includes the Command_ID being completed, the argument index, the typed prefix, and the current Execution_Context.

10.2. PLUGINS SHALL register `CompletionProvider` implementations with the Completion_Engine during their `initialize` lifecycle phase, specifying which Command_IDs or argument patterns they provide completions for.

10.3. WHEN a plugin is unloaded or deactivated, THE Completion_Engine SHALL deregister all CompletionProvider instances associated with that plugin, ensuring no stale providers remain.

10.4. THE Completion_Engine SHALL invoke all applicable providers for a given context in parallel (where possible) and merge their results into a single de-duplicated, ranked candidate list.

10.5. IF a CompletionProvider fails (panics or returns an error), THE Completion_Engine SHALL catch the failure, log a WARN-level record identifying the provider, and continue with candidates from other providers — a single provider failure SHALL NOT prevent completion from functioning.

10.6. BUILT-IN providers (command names, file paths, keywords, line commands, macro names) SHALL be implemented using the same `CompletionProvider` trait as plugin providers, ensuring a uniform internal architecture.

