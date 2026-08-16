# Requirements Document

## Introduction

This feature specifies the **line wrap toggle** subsystem for FileForgeWorkbench (`ff-line-wrap-toggle` crate). Line wrapping controls whether long document lines are rendered as a single display row (requiring horizontal scrolling) or broken across multiple visual rows to fit within a specified width. The feature provides three wrap modes, a WRAP primary command with multiple sub-commands, configurable wrap boundaries (viewport width or fixed column), and wrap indent options for continuation lines.

Line wrapping is a **per-document display property** — it does not modify document content, does not insert line breaks, and is not an undoable operation. Each editor instance maintains its own wrap mode independently. When wrap is active, a single document line may occupy multiple display lines (sub-lines), directly affecting the display-line-mapping layer that tracks document-to-display line relationships.

The feature merges concepts from three sources:

1. **FileForgeEditor** (`line-wrap-toggle`) — per-tab boolean wrap state, WRAP command (ON/OFF/toggle), View menu toggle, horizontal scrollbar interaction, configuration default.
2. **Scintilla** — multi-mode wrapping (None/Word/Character/Whitespace), wrap indent modes (Fixed/Same/Indent/DeepIndent), wrap visual flags (start/end markers), wrap-at-viewport-width semantics, display line mapping integration.
3. **Workbench Architecture Brief** — command-driven operations, per-editor-instance state, configuration-as-data, session persistence.

The Scintilla `SC_WRAP_WHITESPACE` mode is excluded from initial scope (it wraps only at whitespace boundaries, similar to Word mode but stricter). It may be added in a future phase if user demand warrants it.

**Source references:**
- **[FFE-WRAP]** = FileForgeEditor `line-wrap-toggle` specification (7 requirements — per-tab state, WRAP command, View menu toggle, rendering ON/OFF, scrollbar interaction, configuration)
- **[SCI-VS-10]** = Scintilla `WrapAppearance` struct — `Wrap` enum (None/Word/Char/Whitespace), `WrapVisualFlag`, `WrapVisualLocation`, `WrapIndentMode` (Fixed/Same/Indent/DeepIndent), `visualStartIndent`
- **[WB]** = Workbench Architecture Brief — command-driven architecture, per-editor-instance state, configuration-as-data, session persistence

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `display-line-mapping` | **Dependency** | Wrap mode changes trigger bulk `set_height` updates across all document lines; wrapped lines have height ≥ 2 in the contraction state. The display-line-mapping layer tracks sub-lines and provides doc↔display coordinate translation. |
| `whitespace-and-guides` | **Integration** | Wrap visual markers (continuation indicators at line start/end) are rendered by the whitespace-and-guides system. Edge column indicator interacts with wrap boundary when wrapping at a fixed column. |
| `viewport-and-scrolling` | **Consumer** | Wrap changes affect viewport line count, scrollbar range, and scroll position. When wrap is ON, the horizontal scrollbar is disabled. Viewport uses display-line-mapping for layout. |
| `configuration-system` | **Dependency** | Provides TOML-based configuration for default wrap mode, wrap column, wrap indent mode, and visual flags. Uses layered override model. |
| `command-framework` | **Integration** | WRAP primary command is registered in the command registry. Wrap shortcuts are registered in the shortcut registry. |
| `menu-and-statusbar` | **Consumer** | Status bar displays wrap mode indicator. View menu exposes wrap mode submenu. |
| `multi-tab-editor` | **Integration** | Each tab maintains its own wrap mode as per-editor-instance state. Tab switching updates UI indicators. |
| `startup-and-session` | **Integration** | Per-document wrap mode is persisted in session state for restore on reopen. |
| `idle-processing` | **Consumer** | Background wrap height recalculation for large files — wrap layout is computed incrementally in idle time rather than blocking the UI thread. |

## Glossary

- **Wrap_Mode**: An enumeration defining how (or whether) long lines are visually broken. Values: `None` (no wrapping), `Word` (wrap at word boundaries), `Character` (wrap at any character boundary). [SCI-VS-10, FFE-WRAP]
- **Wrap_Boundary**: The column or pixel position at which wrapping occurs. Either the current viewport width (dynamic) or a fixed column number (static). [SCI-VS-10]
- **Wrap_At_Viewport**: The default wrap boundary mode where lines wrap at the current text area width. As the window is resized, wrap positions adjust dynamically. [SCI-VS-10]
- **Wrap_At_Column**: An alternative wrap boundary mode where lines wrap at a fixed column number regardless of viewport width. Useful for enforcing line-length conventions. [SCI-VS-10]
- **Wrap_Indent_Mode**: An enumeration controlling the indentation of continuation lines (sub-lines). Values: `Fixed` (indent by a fixed amount), `Same` (align with start of first sub-line), `Indent` (same + one indent level), `DeepIndent` (same + two indent levels). [SCI-VS-10]
- **Wrap_Visual_Flag**: Visual indicators rendered at continuation line boundaries to show where a logical line has been wrapped. Can appear at end of the sub-line, start of the next sub-line, or in the margin. [SCI-VS-10]
- **Continuation_Line**: A display sub-line that is not the first sub-line of a document line — it exists because the document line was too long to fit in one display row. [SCI-VS-10]
- **Sub_Line**: A specific display line within a wrapped document line, identified by a zero-based offset from the first display line of that document line. [SCI-VS-10]
- **Display_Line_Height**: The number of display lines (sub-lines) a single document line occupies. Height 1 means unwrapped; height ≥ 2 means the line wraps onto additional rows. [SCI-VS-10]
- **Editor_Instance**: A single editor pane associated with one open document/tab. Each instance has its own independent Wrap_Mode. [WB]
- **WRAP_Command**: The primary command (`WRAP`, `WRAP ON`, `WRAP OFF`, `WRAP TOGGLE`, `WRAP WORD`, `WRAP CHAR`) controlling wrap state. [FFE-WRAP]
- **Horizontal_Scrollbar**: The scrollbar rendered below the editing area. Visible and functional when wrap is None; hidden/disabled when wrap is active. [FFE-WRAP]
- **Wrap_Indicator**: The status bar element displaying the current wrap mode when wrapping is active. [FFE-WRAP]

---

## Requirements

### Requirement 1: Wrap Mode Enumeration

**User Story:** As a user, I want to choose between no wrapping, word-boundary wrapping, and character-boundary wrapping, so that I can control exactly how long lines are broken for different file types and use cases.

**Source:** [SCI-VS-10] `Wrap` enum (None/Word/Char); [FFE-WRAP] Req 1 (per-tab boolean, adapted to multi-mode enum).

#### Acceptance Criteria

1. THE system SHALL define a `WrapMode` enum with exactly three variants: `None`, `Word`, and `Character`.
2. WHEN Wrap_Mode is `None`, THE Editor_Instance SHALL render each document line as exactly one display row regardless of line length — no visual line breaking is applied.
3. WHEN Wrap_Mode is `Word`, THE Editor_Instance SHALL break long lines at word boundaries (whitespace, punctuation adjacent to alphanumeric characters) so that whole words are preserved on each sub-line where possible.
4. WHEN Wrap_Mode is `Word` and a single word exceeds the Wrap_Boundary width, THE Editor_Instance SHALL break the word at the Wrap_Boundary position (falling back to character-level breaking for that segment).
5. WHEN Wrap_Mode is `Character`, THE Editor_Instance SHALL break long lines at the exact character position that fills the Wrap_Boundary width, without regard to word boundaries.
6. THE Wrap_Mode `Word` SHALL be the default wrapping style when the user enables wrapping without specifying a mode (via `WRAP ON` or `WRAP TOGGLE`).

---

### Requirement 2: Per-Document Wrap State

**User Story:** As a user working with multiple files, I want each open document to have its own wrap mode, so that I can view source code unwrapped and log files word-wrapped simultaneously.

**Source:** [FFE-WRAP] Req 1 (per-tab wrap state); [WB] per-editor-instance state.

#### Acceptance Criteria

1. EACH Editor_Instance SHALL store a Wrap_Mode value initialised to the configured default wrap mode from the configuration-system.
2. WHEN a new Editor_Instance is created (new file or opened file), ITS Wrap_Mode SHALL be initialised to the `default_mode` configuration value (default: `None`).
3. WHEN the user changes the Wrap_Mode on one Editor_Instance, THE system SHALL leave the Wrap_Mode of all other open Editor_Instances unchanged.
4. WHEN the user switches between tabs, THE status bar Wrap_Indicator and View menu checkmark SHALL update to reflect the Wrap_Mode of the newly active Editor_Instance.
5. WHEN the configuration `default_mode` is absent or invalid, THE system SHALL initialise new Editor_Instances with Wrap_Mode `None`.

---

### Requirement 3: WRAP Primary Command

**User Story:** As a keyboard-driven user, I want a WRAP primary command with sub-commands to control line wrapping from the command line, so that I can manage wrap mode without leaving the keyboard.

**Source:** [FFE-WRAP] Req 2 (WRAP ON/OFF/toggle); extended with WRAP WORD/CHAR sub-commands for multi-mode control.

#### Acceptance Criteria

1. THE command-framework SHALL register `WRAP` as a primary command with Command_ID `"view.wrap"`.
2. WHEN `WRAP ON` is issued, THE active Editor_Instance SHALL set its Wrap_Mode to `Word` (the default enabled mode) if currently `None`, or leave it unchanged if already `Word` or `Character`.
3. WHEN `WRAP OFF` is issued, THE active Editor_Instance SHALL set its Wrap_Mode to `None`.
4. WHEN `WRAP TOGGLE` is issued, THE active Editor_Instance SHALL toggle its Wrap_Mode: if currently `None`, set to `Word`; if currently `Word` or `Character`, set to `None`.
5. WHEN `WRAP` is issued with no arguments, THE system SHALL behave as `WRAP TOGGLE` (toggle between None and the last-used active mode).
6. WHEN `WRAP WORD` is issued, THE active Editor_Instance SHALL set its Wrap_Mode to `Word`.
7. WHEN `WRAP CHAR` is issued, THE active Editor_Instance SHALL set its Wrap_Mode to `Character`.
8. THE WRAP command SHALL return a status message indicating the new wrap state (e.g., "Wrap: Word", "Wrap: Character", "Wrap: Off").
9. WHEN `WRAP ON` is executed while wrap is already active (Word or Character), THE system SHALL return a confirmation message indicating the current mode and take no other action.
10. WHEN `WRAP OFF` is executed while wrap is already None, THE system SHALL return a confirmation message "Wrap is already off" and take no other action.
11. THE WRAP command SHALL be valid in Browse mode, Edit mode, View mode, and all special modes.
12. THE WRAP command SHALL NOT be recorded as an undoable transaction — wrap is a display-only state change.
13. THE WRAP command SHALL NOT be added to command history (display-only operation with no semantic significance to editing).
14. WHEN an invalid sub-command is provided (e.g., `WRAP BANANA`), THE system SHALL display an error message listing valid sub-commands: ON, OFF, TOGGLE, WORD, CHAR.

---

### Requirement 4: Wrap Boundary — Viewport Width vs Fixed Column

**User Story:** As a user, I want to choose whether lines wrap at the current viewport edge or at a fixed column number, so that I can enforce line-length conventions independent of window size.

**Source:** [SCI-VS-10] `wrapWidth` computed from viewport width; adapted to support fixed column as an alternative.

#### Acceptance Criteria

1. THE system SHALL support two wrap boundary modes: `Viewport` (dynamic, wraps at current text area width) and `Column(n)` (static, wraps at column n regardless of viewport width).
2. WHEN wrap boundary is `Viewport`, THE Editor_Instance SHALL recompute wrap positions whenever the text area width changes (window resize, panel dock/undock, margin width change). Wrapped line heights SHALL update to reflect the new width.
3. WHEN wrap boundary is `Column(n)`, THE Editor_Instance SHALL wrap lines at column n regardless of viewport width. IF the viewport is wider than column n, excess space to the right of column n SHALL remain empty.
4. WHEN wrap boundary is `Column(n)` and the viewport is narrower than column n, THE Editor_Instance SHALL still wrap at column n — content beyond the viewport edge is accessible via horizontal scrolling even when wrap is active.
5. THE configuration-system SHALL accept a `wrap_column` key (integer, 0 means Viewport mode, positive integer means Column mode). Default: 0 (Viewport).
6. THE WRAP command SHALL support `WRAP COL n` to set a fixed wrap column, and `WRAP COL 0` to revert to viewport-width wrapping.
7. WHEN `wrap_column` is negative or exceeds 10000, THE system SHALL treat it as invalid, apply the default (Viewport mode), and emit a configuration warning.

---

### Requirement 5: Wrap Indent for Continuation Lines

**User Story:** As a user reading wrapped code, I want continuation lines to be indented so that I can visually distinguish them from new logical lines, making the wrap structure clear.

**Source:** [SCI-VS-10] `WrapIndentMode` (Fixed/Same/Indent/DeepIndent), `visualStartIndent`.

#### Acceptance Criteria

1. THE system SHALL support four wrap indent modes: `Fixed`, `Same`, `Indent`, and `DeepIndent`.
2. WHEN Wrap_Indent_Mode is `Fixed`, continuation lines SHALL be indented by a fixed number of characters from the left margin, defined by the `wrap_indent_amount` configuration value (default: 0 — flush left).
3. WHEN Wrap_Indent_Mode is `Same`, continuation lines SHALL be indented to the same column as the first non-whitespace character of the first sub-line (matching the source line's indentation level).
4. WHEN Wrap_Indent_Mode is `Indent`, continuation lines SHALL be indented to the same position as `Same` mode plus one additional indent level (one tab stop or `indent_width` spaces).
5. WHEN Wrap_Indent_Mode is `DeepIndent`, continuation lines SHALL be indented to the same position as `Same` mode plus two additional indent levels.
6. THE configuration-system SHALL accept a `wrap_indent_mode` key with valid values: `"fixed"`, `"same"`, `"indent"`, `"deep_indent"`. Default: `"fixed"`.
7. THE configuration-system SHALL accept a `wrap_indent_amount` key (integer, 0–40) specifying the fixed indent in characters when using `Fixed` mode. Default: 0.
8. WHEN `wrap_indent_amount` is outside the valid range (0–40), THE system SHALL clamp it and emit a configuration warning.
9. THE wrap indent SHALL reduce the effective width available for text on continuation lines — the wrap position for subsequent sub-lines accounts for the indent offset.

---

### Requirement 6: Display-Line-Mapping Integration

**User Story:** As a viewport renderer, I need wrapped lines to report their correct display height (sub-line count) to the display-line-mapping layer, so that scroll calculations, line numbering, and coordinate translation work correctly.

**Source:** [SCI-VS-10] `wrapWidth`, multi-line layout, display line count; [SCI-CS-12.1] `ContractionState` height tracking.

#### Acceptance Criteria

1. WHEN Wrap_Mode changes from `None` to `Word` or `Character`, THE system SHALL compute the display height (number of sub-lines) for every visible document line and update the display-line-mapping via `set_height(doc_line, height)` for each affected line.
2. WHEN Wrap_Mode changes from `Word` or `Character` to `None`, THE system SHALL set the display height of every visible document line to 1 via `set_height(doc_line, 1)`.
3. WHEN a document line is edited while wrap is active, THE system SHALL recompute the display height of that line and update the display-line-mapping if the height changed.
4. WHEN the Wrap_Boundary changes (viewport resize or column change) while wrap is active, THE system SHALL recompute display heights for all visible lines and update the display-line-mapping.
5. FOR large documents, wrap height recalculation SHALL be performed incrementally via the idle-processing system — only visible and near-viewport lines are computed immediately; remaining lines are computed in background idle cycles.
6. UNTIL a line's wrap height has been computed by the idle-processing system, THE display-line-mapping SHALL assume a provisional height of 1 for that line.
7. THE total Display_Line_Count (sum of all visible line heights) SHALL be used by viewport-and-scrolling to determine the vertical scrollbar range when wrap is active.
8. THE display-line-mapping `doc_from_display` and `display_from_doc` methods SHALL correctly account for wrapped line heights, enabling click-to-position and scroll-to-line operations to target the correct sub-line within a wrapped document line.

---

### Requirement 7: Horizontal Scrollbar Interaction

**User Story:** As a user, I want the horizontal scrollbar to appear only when lines are not being wrapped, so that screen space is not wasted on a scrollbar that serves no purpose during wrapping.

**Source:** [FFE-WRAP] Reqs 4–6 (scrollbar interaction); [SCI-VS-10] `xOffset` reset on wrap enable.

#### Acceptance Criteria

1. WHEN Wrap_Mode changes from `None` to `Word` or `Character` (with boundary mode `Viewport`), THE Editor_Instance SHALL hide the Horizontal_Scrollbar and reset `horizontal_offset` to 0.
2. WHEN Wrap_Mode changes from `Word` or `Character` to `None`, THE Editor_Instance SHALL display the Horizontal_Scrollbar.
3. WHILE Wrap_Mode is `None`, THE Horizontal_Scrollbar SHALL be visible and functional, with its thumb position reflecting `horizontal_offset` relative to the longest visible line width.
4. WHILE Wrap_Mode is `Word` or `Character` with boundary `Viewport`, THE Horizontal_Scrollbar SHALL be hidden — all content is guaranteed to fit within the viewport width.
5. WHEN Wrap_Mode is `Word` or `Character` with boundary `Column(n)` and the viewport is narrower than column n, THE Horizontal_Scrollbar SHALL remain visible to allow horizontal panning of the wrapped content.

---

### Requirement 8: Status Bar Wrap Indicator

**User Story:** As a user, I want to see the current wrap mode in the status bar, so that I always know whether and how wrapping is applied to the active document.

**Source:** [FFE-WRAP] Req 3 (View menu checkmark, adapted to status bar indicator).

#### Acceptance Criteria

1. WHEN the active Editor_Instance has Wrap_Mode `Word`, THE status bar SHALL display a Wrap_Indicator showing "Wrap: Word".
2. WHEN the active Editor_Instance has Wrap_Mode `Character`, THE status bar SHALL display a Wrap_Indicator showing "Wrap: Char".
3. WHEN the active Editor_Instance has Wrap_Mode `None`, THE status bar SHALL NOT display a Wrap_Indicator — it is omitted to reduce clutter at the default state.
4. THE Wrap_Indicator SHALL be positioned in the status bar after the line/column display and before any other mode indicators.
5. WHEN the user clicks the Wrap_Indicator in the status bar, THE Editor SHALL cycle through wrap modes: None → Word → Character → None.
6. WHEN the user switches tabs, THE Wrap_Indicator SHALL update to reflect the Wrap_Mode of the newly active Editor_Instance.

---

### Requirement 9: View Menu Integration

**User Story:** As a mouse-oriented user, I want a "Word Wrap" submenu in the View menu showing available wrap modes, so that I can change the wrap mode with familiar menu interactions.

**Source:** [FFE-WRAP] Req 3 (View menu toggle, expanded to submenu for multi-mode).

#### Acceptance Criteria

1. THE View menu SHALL contain a "Word Wrap" submenu with items: "Off", "Word", "Character".
2. THE submenu SHALL display a radio-style indicator (bullet or checkmark) next to the currently active Wrap_Mode for the active Editor_Instance.
3. WHEN the user selects "Off" from the submenu, THE active Editor_Instance SHALL set its Wrap_Mode to `None`.
4. WHEN the user selects "Word" from the submenu, THE active Editor_Instance SHALL set its Wrap_Mode to `Word`.
5. WHEN the user selects "Character" from the submenu, THE active Editor_Instance SHALL set its Wrap_Mode to `Character`.
6. WHEN the user switches to a different tab, THE submenu radio indicator SHALL update to reflect the Wrap_Mode of the newly active Editor_Instance.

---

### Requirement 10: Wrap Visual Flags (Continuation Markers)

**User Story:** As a user reading wrapped content, I want optional visual markers at the beginning or end of continuation lines, so that I can clearly see where a logical line has been broken.

**Source:** [SCI-VS-10] `WrapVisualFlag` (None/End/Start/Margin), `WrapVisualLocation`.

#### Acceptance Criteria

1. THE system SHALL support wrap visual flags that indicate where wrapping has occurred. Flags can appear at: the end of a sub-line (before the wrap break), the start of a continuation line (after the wrap break), or in the margin adjacent to a continuation line.
2. WHEN wrap visual flag `End` is enabled, THE Editor_Instance SHALL render a small wrap indicator glyph (e.g., a bent arrow or pilcrow) at the right edge of each sub-line that continues onto the next display row.
3. WHEN wrap visual flag `Start` is enabled, THE Editor_Instance SHALL render a small wrap indicator glyph at the left side of each continuation line (not the first sub-line of a document line).
4. WHEN wrap visual flag `Margin` is enabled, THE Editor_Instance SHALL render a wrap indicator in the line-number margin adjacent to each continuation line.
5. WHEN no wrap visual flags are enabled (`None`), THE Editor_Instance SHALL render no visual markers at wrap break points — continuation lines appear seamlessly after the first sub-line.
6. THE configuration-system SHALL accept a `wrap_visual_flags` key with valid values: `"none"`, `"end"`, `"start"`, `"start_end"`, `"margin"`. Default: `"none"`.
7. THE visual flag indicators SHALL be rendered using the whitespace-and-guides rendering infrastructure, using the configured foreground colour for wrap markers.

---

### Requirement 11: Wrap Persistence in Session State

**User Story:** As a user, I want my per-document wrap mode to be remembered when I reopen the editor, so that I don't have to re-enable wrapping every time I open a file.

**Source:** [FFE-WRAP] Req 7 (configuration default); [WB] session persistence pattern (following view-zoom model).

#### Acceptance Criteria

1. WHEN the workbench exits normally, THE session system SHALL persist the Wrap_Mode of each open Editor_Instance, associated with the document's resource URI.
2. WHEN the workbench restores a session on startup, THE session system SHALL restore the persisted Wrap_Mode for each reopened document. IF no persisted value exists for a document, THE Editor_Instance SHALL use the `default_mode` configuration value.
3. IF a persisted Wrap_Mode value is not a recognized variant (e.g., from a future version), THE system SHALL fall back to `None` and emit a warning.
4. THE session system SHALL store per-document wrap mode in its existing session state format (the same store used by startup-and-session for cursor positions, scroll state, zoom offsets, etc.).
5. THE session system SHALL also persist the per-document wrap boundary mode (Viewport or Column(n)) when it differs from the global configuration default.

---

### Requirement 12: Configuration Defaults

**User Story:** As an administrator or power user, I want to configure the default wrap behaviour in TOML configuration, so that all new documents start with my preferred wrap settings without manual toggling.

**Source:** [FFE-WRAP] Req 7 (configuration); [SCI-VS-10] wrap configuration surface; [WB] configuration-as-data.

#### Acceptance Criteria

1. THE configuration-system SHALL accept a `[view.wrap]` table in configuration files with the following keys:
   - `default_mode` (string): The initial Wrap_Mode for new editor instances. Values: `"none"`, `"word"`, `"character"`. Default: `"none"`.
   - `wrap_column` (integer): Wrap boundary column. 0 = viewport width (dynamic). Positive integer = fixed column. Default: 0.
   - `indent_mode` (string): Wrap indent mode. Values: `"fixed"`, `"same"`, `"indent"`, `"deep_indent"`. Default: `"fixed"`.
   - `indent_amount` (integer): Fixed indent amount in characters (used when `indent_mode` is `"fixed"`). Range: 0–40. Default: 0.
   - `visual_flags` (string): Wrap visual flags. Values: `"none"`, `"end"`, `"start"`, `"start_end"`, `"margin"`. Default: `"none"`.
2. WHEN any configuration key contains an invalid value, THE system SHALL apply the default for that key and emit a configuration warning via the logging-subsystem.
3. WHEN wrap configuration keys are changed via hot-reload, THE system SHALL apply the new defaults to newly opened documents only — already-open documents retain their current wrap settings.
4. THE `[view.wrap]` configuration SHALL support the layered override model: workspace-level overrides user-level, project-level overrides workspace-level.
5. IF `default_mode` is set to `"word"` or `"character"`, new documents SHALL open with wrap already active — the user does not need to manually enable it.

---

### Requirement 13: Rendering Behaviour When Wrap Is Active

**User Story:** As a user viewing wrapped content, I want the editor to correctly render multi-line wrapped text with proper line numbering, cursor placement, and selection highlighting across sub-lines.

**Source:** [FFE-WRAP] Req 5 (rendering when ON); [SCI-VS-10] wrapped line layout, sub-line rendering.

#### Acceptance Criteria

1. WHILE Wrap_Mode is `Word` or `Character`, THE Editor_Instance SHALL render each document line across as many display rows as needed to fit the content within the Wrap_Boundary width.
2. THE line number gutter SHALL display the document line number only on the first sub-line of each document line. Continuation sub-lines SHALL display no line number (blank gutter) or a continuation marker if configured.
3. WHEN the cursor is positioned within a wrapped line, THE cursor SHALL appear at the correct character position within the correct sub-line, navigable with arrow keys across sub-line boundaries.
4. WHEN the user presses the Down arrow while on a continuation sub-line that is not the last sub-line of the document line, THE cursor SHALL move to the same visual column on the next sub-line of the same document line.
5. WHEN text selection spans across sub-line boundaries within the same wrapped document line, THE selection highlight SHALL render correctly across all affected sub-lines without gaps or overlaps.
6. WHEN Wrap_Mode is active, THE Editor_Instance SHALL NOT apply `horizontal_offset` to line rendering (all content is visible within the wrap boundary, except in Column(n) mode with narrow viewport per Req 4.4).

