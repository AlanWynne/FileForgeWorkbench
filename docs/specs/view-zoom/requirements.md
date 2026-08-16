# Requirements Document

## Introduction

This feature specifies the **view zoom** subsystem for FileForgeWorkbench. Zoom adjusts the effective font size of the editor content area by applying an integer point offset to the base editor font size defined in the theme. Unlike percentage-based zoom systems, this model directly modifies typographical point size, providing predictable, font-metric-aligned scaling.

Zoom is a **display-only** operation — it does not modify document content, does not affect file output, and is not recorded as an undoable transaction. The zoom level is maintained **per editor instance** (per document tab), allowing the user to keep different documents at different magnifications. Zoom affects only the editor font rendering; workbench chrome (menus, status bars, panels, file tree) remains at the system/theme-defined size.

The feature provides four interaction methods:

1. **Keyboard shortcuts** — Ctrl+= zoom in, Ctrl+- zoom out, Ctrl+0 reset.
2. **Mouse wheel** — Ctrl+Scroll to zoom in/out by one step.
3. **ZOOM primary command** — `ZOOM n`, `ZOOM IN`, `ZOOM OUT`, `ZOOM RESET`.
4. **Status bar indicator** — displays current offset when non-zero; clickable for quick access.

The zoom model is adapted from Scintilla's integer-offset design [SCI-VS-ZOOM] where `SCI_SETZOOM` applies a point offset. FileForgeEditor's percentage-based zoom model [FFE-ZOOM] is reinterpreted: the integer offset approach provides finer control at typical editing sizes without floating-point rounding in font metrics.

**Source references:**
- **[FFE-ZOOM]** = FileForgeEditor `view-zoom` specification (9 requirements — zoom model, menu, shortcuts, mouse wheel, indicator, persistence, ZOOM command, configuration, DPI)
- **[SCI-VS-ZOOM]** = Scintilla ViewStyle zoom — integer point offset model (`SCI_SETZOOM`, `SCI_GETZOOM`), range -10 to +20 default
- **[WB]** = Workbench Architecture Brief — command-driven architecture, per-editor-instance state, configuration-as-data

## Cross-References

| Sub-Project | Relationship | Description |
|---|---|---|
| `theme-and-appearance` | **Dependency** | Provides the base editor font size (monospace font stack point size) to which the zoom offset is applied. Zoom reads font metrics from the theme system. |
| `configuration-system` | **Dependency** | Provides TOML-based configuration for zoom defaults, step size, and range limits. Uses layered override model. |
| `viewport-and-scrolling` | **Integration** | Zoom changes affect `visible_count` (lines fitting in viewport). Viewport recalculates layout when zoom offset changes. |
| `command-framework` | **Integration** | ZOOM primary command is registered in the command registry. Zoom shortcuts are registered as reserved shortcuts in the shortcut registry. |
| `menu-and-statusbar` | **Consumer** | Status bar displays zoom indicator. View menu exposes zoom submenu items. |
| `multi-tab-editor` | **Integration** | Each tab maintains its own zoom offset as per-editor-instance state. |
| `startup-and-session` | **Integration** | Per-document zoom offsets are persisted in session state for restore on reopen. |

## Glossary

- **Zoom_Offset**: A signed integer representing the number of points added to or subtracted from the base editor font size. Positive values enlarge text; negative values shrink it. Zero means no zoom (default rendering). [SCI-VS-ZOOM]
- **Base_Font_Size**: The monospace editor font point size defined in the theme configuration (e.g., 12pt). The effective rendered size is `Base_Font_Size + Zoom_Offset`. [FFE-ZOOM, SCI-VS-ZOOM]
- **Effective_Font_Size**: The computed font size used for editor text rendering: `Base_Font_Size + Zoom_Offset`, clamped to a minimum of 1pt. [SCI-VS-ZOOM]
- **Zoom_Step**: The number of points added or removed per zoom in/out operation. Default: 1 point. Configurable. [FFE-ZOOM]
- **Minimum_Offset**: The lowest permitted Zoom_Offset value. Default: -10. [SCI-VS-ZOOM]
- **Maximum_Offset**: The highest permitted Zoom_Offset value. Default: +60. [SCI-VS-ZOOM, extended from Scintilla's +20 to accommodate large-display use cases]
- **Zoom_Indicator**: The status bar element displaying the current Zoom_Offset when it differs from zero (e.g., `+3`, `-2`). [FFE-ZOOM]
- **Editor_Instance**: A single editor pane associated with one open document/tab. Each instance has its own independent Zoom_Offset. [WB]

---

## Requirements

### Requirement 1: Zoom Offset Model

**User Story:** As an editor user, I want the editor to apply a point-size offset to the editor font, so that I can make text larger for readability or smaller to see more content without affecting the rest of the workbench UI.

**Source:** [SCI-VS-ZOOM] Integer offset model; [FFE-ZOOM] Requirement 1 (adapted from percentage to offset).

#### Acceptance Criteria

1. EACH Editor_Instance SHALL maintain a Zoom_Offset expressed as a signed integer (i32) representing the point offset applied to the Base_Font_Size.
2. THE Effective_Font_Size SHALL be computed as `max(1, Base_Font_Size + Zoom_Offset)` — the rendered font size is never less than 1 point regardless of the offset value.
3. THE Zoom_Offset SHALL affect only the editor text content area: document line text, prefix area (line numbers), and the command input field. It SHALL NOT affect workbench chrome including menus, status bar text, file tree panel, tab headers, dockable panel headers, or any non-editor UI element.
4. WHEN the Zoom_Offset is zero, THE Editor_Instance SHALL render at the theme-defined Base_Font_Size with no additional scaling applied.
5. THE Zoom_Offset SHALL be constrained to the range [Minimum_Offset, Maximum_Offset] inclusive. Any operation that would set the offset outside this range SHALL clamp it to the nearest bound.
6. WHEN the Zoom_Offset changes, THE Editor_Instance SHALL recalculate font metrics (glyph widths, line heights, character advance) and re-layout all visible content within the same rendering frame — there SHALL be no visible flicker or intermediate state.
7. WHEN the Zoom_Offset changes, THE Editor_Instance SHALL preserve the current cursor position (line and column) and keep the cursor row visible in the viewport by adjusting `top_line` if necessary.
8. WHEN the Zoom_Offset increases, THE number of visible lines (`visible_count`) in the viewport SHALL decrease (larger text). WHEN the Zoom_Offset decreases, `visible_count` SHALL increase (smaller text).
9. THE Zoom_Offset SHALL NOT affect the logical content of the document — it is a display-only transformation. Zoom does not modify line text, character positions, column numbers, or any data written to disk on SAVE.
10. EACH Editor_Instance SHALL maintain its Zoom_Offset independently — changing zoom in one tab SHALL NOT affect the zoom offset of any other tab.

---

### Requirement 2: Keyboard Shortcuts

**User Story:** As a keyboard-driven user, I want standard keyboard shortcuts to zoom in, zoom out, and reset zoom, so that I can adjust the view quickly without reaching for the menu.

**Source:** [FFE-ZOOM] Requirement 3 (keyboard shortcuts); [SCI-VS-ZOOM] key binding pattern.

#### Acceptance Criteria

1. WHEN the user presses Ctrl+= (Ctrl+Plus on US keyboards), THE active Editor_Instance SHALL increase its Zoom_Offset by one Zoom_Step, clamped at Maximum_Offset.
2. WHEN the user presses Ctrl+- (Ctrl+Minus), THE active Editor_Instance SHALL decrease its Zoom_Offset by one Zoom_Step, clamped at Minimum_Offset.
3. WHEN the user presses Ctrl+0 (zero), THE active Editor_Instance SHALL reset its Zoom_Offset to zero.
4. THE keyboard shortcuts SHALL function only when an Editor_Instance has focus. They SHALL NOT apply when focus is in a non-editor panel (file tree, help panel, terminal).
5. THE keyboard shortcuts SHALL be registered as reserved shortcuts in the command-framework shortcut registry. Ctrl+=, Ctrl+-, and Ctrl+0 SHALL NOT be reassignable by user configuration or plugins.
6. WHEN the Zoom_Offset is already at Maximum_Offset and the user presses Ctrl+=, THE Editor_Instance SHALL take no action and display a brief status message: "Maximum zoom reached (+{Maximum_Offset})".
7. WHEN the Zoom_Offset is already at Minimum_Offset and the user presses Ctrl+-, THE Editor_Instance SHALL take no action and display a brief status message: "Minimum zoom reached ({Minimum_Offset})".

---

### Requirement 3: Ctrl+Mouse Wheel Zoom

**User Story:** As a desktop user, I want to hold Ctrl and scroll the mouse wheel to zoom in and out, so that I can adjust the view with a familiar gesture used in web browsers and other desktop applications.

**Source:** [FFE-ZOOM] Requirement 4 (Ctrl+mouse wheel); [SCI-VS-ZOOM] scroll-zoom.

#### Acceptance Criteria

1. WHEN the user holds the Ctrl key and scrolls the mouse wheel up (away from the user) while the cursor is over an Editor_Instance, THE Editor_Instance SHALL increase its Zoom_Offset by one Zoom_Step, clamped at Maximum_Offset.
2. WHEN the user holds the Ctrl key and scrolls the mouse wheel down (toward the user) while the cursor is over an Editor_Instance, THE Editor_Instance SHALL decrease its Zoom_Offset by one Zoom_Step, clamped at Minimum_Offset.
3. WHEN the Ctrl key is NOT held, mouse wheel scrolling SHALL perform its normal function (vertical document scrolling) and SHALL NOT affect the Zoom_Offset.
4. THE Ctrl+Scroll gesture SHALL apply to the Editor_Instance under the mouse cursor, regardless of which editor has keyboard focus. IF the mouse cursor is not over any Editor_Instance, the gesture SHALL be ignored.
5. WHEN multiple scroll events arrive in rapid succession (fast scrolling), THE Editor_Instance SHALL apply each zoom step individually — there is no debouncing or acceleration for zoom scroll events.

---

### Requirement 4: Zoom Range Limits

**User Story:** As an administrator, I want configurable minimum and maximum zoom offsets, so that I can restrict the zoom range for deployments or allow extended ranges for accessibility.

**Source:** [FFE-ZOOM] Requirement 8 (configuration); [SCI-VS-ZOOM] range limits.

#### Acceptance Criteria

1. THE configuration-system SHALL accept a `[view.zoom]` table in configuration files with the following keys:
   - `default_offset` (integer): The initial Zoom_Offset for new editor instances. Default: 0.
   - `step` (integer): The Zoom_Step applied per zoom in/out operation. Default: 1. Valid range: 1–10.
   - `min_offset` (integer): The Minimum_Offset value. Default: -10. Valid range: -20 to 0.
   - `max_offset` (integer): The Maximum_Offset value. Default: +60. Valid range: 0 to +100.
2. WHEN `min_offset` is greater than or equal to `max_offset`, THE configuration-system SHALL emit a configuration warning and apply the defaults (-10 and +60).
3. WHEN `default_offset` is outside the [`min_offset`, `max_offset`] range, THE configuration-system SHALL clamp it to the nearest bound and emit a configuration warning.
4. WHEN `step` is outside the valid range (1–10), THE configuration-system SHALL clamp it and emit a configuration warning.
5. WHEN any zoom configuration key contains an invalid value type, THE configuration-system SHALL apply the default for that key and emit a configuration warning.
6. WHEN zoom configuration keys are changed via hot-reload, THE configuration-system SHALL apply the new limits immediately. IF any active Editor_Instance has a Zoom_Offset outside the new range, its offset SHALL be clamped to the nearest new bound.

---

### Requirement 5: Per-Editor-Instance Zoom

**User Story:** As a user working with multiple documents, I want each editor tab to remember its own zoom level independently, so that I can keep source code zoomed to one size and a log file at another.

**Source:** [WB] per-editor-instance state; adapted from [FFE-ZOOM] Requirement 6 (which was global).

#### Acceptance Criteria

1. EACH Editor_Instance (document tab) SHALL store its own Zoom_Offset independently from all other editor instances.
2. WHEN a new Editor_Instance is created (new file or opened file), ITS Zoom_Offset SHALL be initialised to the `default_offset` configuration value (default: 0).
3. WHEN the user switches between tabs, THE status bar Zoom_Indicator SHALL update to reflect the Zoom_Offset of the newly active Editor_Instance.
4. WHEN an Editor_Instance is split (if supported by layout-and-docking), EACH split view SHALL maintain its own independent Zoom_Offset initialised from the source instance's current offset.
5. ZOOM primary command operations and keyboard shortcuts SHALL apply only to the currently active Editor_Instance.

---

### Requirement 6: Zoom Persistence in Session

**User Story:** As an editor user, I want my per-document zoom levels to be remembered when I reopen the editor, so that I don't have to re-adjust the zoom every time.

**Source:** [FFE-ZOOM] Requirement 6 (persistence, adapted to per-document); [WB] session state.

#### Acceptance Criteria

1. WHEN the workbench exits normally, THE session system SHALL persist the Zoom_Offset of each open Editor_Instance, associated with the document's resource URI.
2. WHEN the workbench restores a session on startup, THE session system SHALL restore the persisted Zoom_Offset for each reopened document. IF no persisted value exists for a document, THE Editor_Instance SHALL use the `default_offset` configuration value.
3. IF a persisted Zoom_Offset is outside the currently configured [Minimum_Offset, Maximum_Offset] range (e.g., because configuration changed between sessions), THE Editor_Instance SHALL clamp the value to the nearest bound and apply it without error.
4. THE session system SHALL store per-document zoom offsets in its existing session state format (the same store used by startup-and-session for cursor positions, scroll state, etc.).

---

### Requirement 7: Status Bar Zoom Indicator

**User Story:** As an editor user, I want to see the current zoom offset in the status bar when it differs from zero, so that I always know whether the view is scaled.

**Source:** [FFE-ZOOM] Requirement 5 (zoom indicator, adapted to offset display).

#### Acceptance Criteria

1. WHEN the active Editor_Instance has a Zoom_Offset that is not zero, THE status bar SHALL display a Zoom_Indicator showing the current offset with sign (e.g., `Zoom: +3`, `Zoom: -2`).
2. WHEN the active Editor_Instance has a Zoom_Offset of zero, THE status bar SHALL NOT display the Zoom_Indicator — it is omitted to reduce clutter at the default state.
3. THE Zoom_Indicator SHALL be positioned in the status bar after the encoding display and before the line/column display.
4. WHEN the user clicks the Zoom_Indicator in the status bar, THE Editor SHALL display a zoom popup or dropdown allowing quick selection of common offsets (e.g., -5, -2, 0, +2, +5, +10) and a "Reset to 0" action.
5. THE Zoom_Indicator text SHALL use the format `Zoom: +N` for positive offsets and `Zoom: -N` for negative offsets, where N is the absolute integer value.

---

### Requirement 8: ZOOM Primary Command

**User Story:** As a command-line user, I want a ZOOM primary command to set the zoom offset from the command line, so that I can adjust the view without reaching for the mouse or menu.

**Source:** [FFE-ZOOM] Requirement 7 (ZOOM command, adapted to offset model).

#### Acceptance Criteria

1. THE command-framework SHALL register `ZOOM` as a primary command with Command_ID `"view.zoom"`.
2. WHEN `ZOOM n` is issued where n is a signed integer, THE active Editor_Instance SHALL set its Zoom_Offset to n, clamped to the [Minimum_Offset, Maximum_Offset] range.
3. WHEN `ZOOM IN` is issued, THE active Editor_Instance SHALL increase its Zoom_Offset by one Zoom_Step, clamped at Maximum_Offset.
4. WHEN `ZOOM OUT` is issued, THE active Editor_Instance SHALL decrease its Zoom_Offset by one Zoom_Step, clamped at Minimum_Offset.
5. WHEN `ZOOM RESET` is issued, THE active Editor_Instance SHALL set its Zoom_Offset to zero.
6. WHEN `ZOOM` is issued with no arguments, THE Editor SHALL display the current Zoom_Offset and Effective_Font_Size in the status message area (e.g., "Zoom offset: +3 (effective size: 15pt)").
7. THE `ZOOM` command SHALL be valid in Browse mode, Edit mode, View mode, and all special modes.
8. THE `ZOOM` command SHALL NOT be added to command history (it is a display-only operation with no semantic significance to the editing session).
9. THE `ZOOM` command SHALL NOT be recorded as an undoable transaction — zoom is a display-only state change.

---

### Requirement 9: Zoom Interaction with DPI and Multi-Monitor

**User Story:** As a user with multiple monitors at different DPI scales, I want zoom to work correctly regardless of which monitor the editor is on, so that the content is always readable at my chosen scale.

**Source:** [FFE-ZOOM] Requirement 9 (DPI interaction); [SCI-VS-ZOOM] point-size model inherently DPI-aware.

#### Acceptance Criteria

1. THE Zoom_Offset SHALL be applied as a typographical point offset — the rendering engine computes physical pixels using the operating system's DPI scale for the target monitor. A +3 offset means +3 points regardless of DPI.
2. WHEN the editor window is moved to a different monitor with a different DPI scale, THE Editor_Instance SHALL maintain its Zoom_Offset unchanged. The physical pixel rendering adapts to the new DPI but the offset remains the same.
3. WHEN undocked panels containing editor instances (per layout-and-docking) are on different monitors, EACH Editor_Instance SHALL use its own Zoom_Offset and render correctly for the DPI of the monitor it is displayed on.
4. THE Effective_Font_Size in points SHALL remain constant across monitors — only the physical pixel rendering changes based on DPI. The status bar Zoom_Indicator SHALL continue to show the same offset value.
