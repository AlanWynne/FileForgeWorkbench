# Requirements Document

## Introduction

This feature specifies the **ASA Report Preview** subsystem for FileForgeWorkbench (`ff-asa-report-preview` crate). It provides a visual rendering mode that interprets ASA (ANSI) carriage control characters and displays mainframe spool files as they would have appeared on a line printer — complete with page breaks, line spacing, overprint (bold/underline), and green-bar/blue-bar paper simulation.

The ASA Report Preview subsystem provides:

1. **ASA carriage control character interpretation** — space (single space), `0` (double space), `-` (triple space), `1` (page eject/new page), `+` (no advance/overstrike).
2. **Auto-detection** of ASA content from first-column character pattern analysis.
3. **PREVIEW command integration** — rendering paginated output through the `custom-file-viewers` framework.
4. **Page-break rendering** — visual page bands with alternating colours and page numbering.
5. **Overstrike line merging** — combining `+` (overprint) lines with their base line to produce bold and underlined text.
6. **Print preview panel** — paginated view with headers/footers in a dockable panel.
7. **ASA strip/restore on edit** — transparent removal of column 1 control characters during editing, with restoration on save.
8. **Line printer emulation** — 132-column page width, 60-line page depth as defaults, simulating IBM 1403/3211 output.
9. **Configurable page dimensions** — operator-adjustable page width, depth, and margins.
10. **Export to PDF/text** — rendered output export with full ASA interpretation applied.

### Design Principles

1. **GUI-independent** — all ASA parsing, merging, pagination, and export logic operates on the document model without GUI framework dependency. Rendering hints are provided to the UI layer but no egui dependency exists in the core logic. [WB]
2. **Command-framework integrated** — PREVIEW activation, export commands, and page navigation are registered with the command framework, discoverable, and scriptable. [WB]
3. **Custom-viewer compliant** — the ASA report preview is registered as a `Custom_Viewer` with Viewer_Key `"asa-report"` through the `custom-file-viewers` framework. [FFE-ASA]
4. **Read-only display** — Preview_Mode is a rendering transformation; it does not modify the Edit_Buffer or the file on disk. [FFE-ASA]
5. **Sequence-aware** — operates on post-strip content when sequence number stripping is active. [FFE-ASA]
6. **Plugin-extensible** — the viewer is registered via the plugin architecture's viewer trait, allowing future ASA variants or custom report renderers. [WB]

This crate is a Wave 12 (FileForge Domain) component in the workbench architecture. It depends on:
- `ff-document-model` — for edit buffer access and line content
- `ff-command` (command-framework) — for command registration and dispatch
- `ff-layout` (layout-and-docking) — for print preview panel docking
- `ff-config` (configuration-system) — for page dimensions, colours, and preview settings
- `ff-theme` (theme-and-appearance) — for Page_Band colours, line-band shading, font rendering
- `ff-custom-viewers` (custom-file-viewers) — for Viewer_Registry integration and PREVIEW command routing
- `ff-fileforge` (fileforge-integration) — for ASA detection hooks and RECFM metadata

It is consumed by:
- `ff-custom-viewers` — registers the `"asa-report"` viewer in the Viewer_Registry
- `ff-file-ops` (file-operations) — for ASA strip/restore on save behaviour

### Source References

- **[FFE-ASA]** = FileForgeEditor `asa-report-preview` specification (8 requirements)
- **[WB]** = Workbench Platform Architecture Brief (GUI independence, command-driven, multi-crate, plugin architecture)

### Cross-References

- **`fileforge-integration`** — Defines ASA carriage control detection (RECFM "FBA"/"VBA"), EBCDIC encoding support, and flat-file mode activation that triggers ASA preview offers.
- **`document-model`** — Provides the TextBuffer/Document that this subsystem reads for rendering and that ASA strip/restore modifies.
- **`layout-and-docking`** — Provides dockable panel infrastructure for the print preview panel.
- **`command-framework`** — All commands (PREVIEW, LOCATE PAGE, PREVIEW EXPORT) are registered, dispatched, and discoverable through this framework.
- **`theme-and-appearance`** — Provides the colour tokens, font metrics, and design system tokens used for page bands, line shading, and overstrike rendering.
- **`custom-file-viewers`** — Defines the Viewer_Registry, PREVIEW command dispatch, viewer/edit coexistence, and split view infrastructure.

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **Preview_Mode** | The display mode that renders ASA carriage control characters as visual report formatting instead of raw text. Always read-only. | [FFE-ASA] |
| **ASA_Control** | The character in column 1 of each record that defines the printer action before printing the line. Standard characters: space (single space), `0` (double space), `-` (triple space), `1` (page eject), `+` (no advance/overstrike), `H` (halt). | [FFE-ASA] |
| **Page_Band** | The visual element rendered at each `1` (new page) control — a full-width coloured band containing a page number label, simulating a page break. | [FFE-ASA] |
| **Overprint_Line** | A line with `+` in column 1, which the printer would have printed on the same physical line as the previous record, creating bold text or underlines by character superimposition. | [FFE-ASA] |
| **Merged_Line** | The result of combining a base line with one or more Overprint_Lines. Identical characters become bold; dash/underscore overprints become underlined. | [FFE-ASA] |
| **Page_Counter** | The running count of `1` (new page) characters seen in the file, used to label Page_Bands. | [FFE-ASA] |
| **Line_Bands** | The alternating light/dark horizontal shading applied to groups of lines to simulate green-bar or blue-bar fanfold paper used on classic line printers. | [FFE-ASA] |
| **PREVIEW** | The primary command that activates the ASA report preview through the custom-file-viewers framework. | [FFE-ASA] |
| **Line_Printer_Page** | A logical page defined by standard line printer dimensions: 132 columns wide by 60 lines deep (default IBM 1403/3211 standard). | [WB] |
| **Page_Width** | The number of character columns in a logical printer page. Default: 132. Configurable. | [WB] |
| **Page_Depth** | The number of print lines in a logical printer page (excluding channel skips). Default: 60. Configurable. | [WB] |
| **ASA_Detection** | The heuristic process that examines the first-column character pattern of a file to determine whether it contains ASA carriage control characters. | [FFE-ASA], [WB] |
| **Print_Preview_Panel** | A dockable panel that displays the paginated report with headers, footers, and page navigation controls. | [WB] |
| **ASA_Strip** | The process of removing column 1 ASA control characters from the edit buffer for editing, making the data content start at column 1. | [WB] |
| **ASA_Restore** | The process of re-inserting ASA control characters into column 1 when saving, preserving the original control character sequence. | [WB] |

---

## Requirements

### Requirement 1: ASA Carriage Control Characters

**User Story:** As an operator working with mainframe spool files, I want the system to correctly interpret all standard ASA carriage control characters so that the report is rendered with accurate spacing and page structure.

**Source:** [FFE-ASA] Requirements 2–4. Cross-references: `fileforge-integration` (RECFM metadata, ASA detection), `document-model` (line content access).

#### Acceptance Criteria

1. THE system SHALL recognise the following ASA carriage control characters in column 1 of each record: [FFE-ASA]
  - Space (` `) — single space before printing (normal line advance)
  - Zero (`0`) — double space before printing (skip one blank line)
  - Minus (`-`) — triple space before printing (skip two blank lines)
  - One (`1`) — page eject (advance to top of next page before printing)
  - Plus (`+`) — no advance (overstrike/overprint on previous line)
  - `H` — halt (printer halt indication)

2. WHEN Preview_Mode is active and a line has ASA control character space (` `), THE system SHALL render the line with standard single spacing — no additional blank lines inserted before it. [FFE-ASA]

3. WHEN Preview_Mode is active and a line has ASA control character `0`, THE system SHALL insert one blank preview line before the line's content, producing double spacing. [FFE-ASA]

4. WHEN Preview_Mode is active and a line has ASA control character `-`, THE system SHALL insert two blank preview lines before the line's content, producing triple spacing. [FFE-ASA]

5. WHEN Preview_Mode is active and a line has ASA control character `1`, THE system SHALL render a page break (Page_Band) at that position before the line's content. [FFE-ASA]

6. WHEN Preview_Mode is active and a line has ASA control character `+`, THE system SHALL NOT render the line as a separate row but SHALL merge it with the preceding base line (overstrike). [FFE-ASA]

7. WHEN Preview_Mode is active and the `H` (halt) control character is encountered, THE system SHALL render a visually distinct full-width amber warning band labelled `─── PRINTER HALT ───` at that position. [FFE-ASA]

8. THE blank lines inserted for spacing (criteria 1.3, 1.4) SHALL be display artifacts only — they are NOT real document lines and SHALL NOT be editable, selectable as text, or counted in the document's line total. [FFE-ASA]

9. WHEN a line's column 1 character is not one of the recognised ASA control characters, THE system SHALL treat it as a space (single spacing) and render a WARN-level diagnostic in the log. [WB]

---

### Requirement 2: ASA Auto-Detection

**User Story:** As an operator opening a file of unknown origin, I want the editor to automatically detect whether the file contains ASA carriage control characters so that the preview mode can be offered without manual classification.

**Source:** [FFE-ASA] Requirement 1 (criteria 2–3), [WB]. Cross-references: `fileforge-integration` (RECFM "FBA"/"VBA" metadata), `language-service` (language profile detection).

#### Acceptance Criteria

1. THE ASA_Detection algorithm SHALL examine the first column of the first N non-blank lines of a file (default N = 50, configurable) and determine whether the file likely contains ASA carriage control characters. [WB]

2. THE detection SHALL classify a file as ASA-controlled WHEN at least 80% of the sampled first-column characters match the set of valid ASA control characters (space, `0`, `-`, `1`, `+`, `H`) AND at least one `1` (page eject) character is present in the sample. [WB]

3. WHEN the file's RECFM metadata (from `fileforge-integration` config or dataset catalog attributes) is `"FBA"` or `"VBA"`, THE system SHALL bypass heuristic detection and treat the file as ASA-controlled unconditionally. [FFE-ASA]

4. WHEN ASA is detected (by heuristic or RECFM metadata), THE system SHALL offer to activate the ASA report preview via a non-blocking status bar prompt, consistent with the `custom-file-viewers` Requirement 2 language-profile viewer offer. [FFE-ASA]

5. WHEN `PREVIEW ON` is issued and no ASA carriage control characters are detected in the file, THE system SHALL display a warning: `PREVIEW: no ASA carriage control detected — preview may not render correctly` and activate Preview_Mode anyway. [FFE-ASA]

6. THE detection threshold (default 80%) and sample size (default 50 lines) SHALL be configurable in the `[asa_preview]` section of configuration. [WB]

7. THE ASA_Detection SHALL run asynchronously and SHALL NOT block the UI thread for files larger than 1 MB. [WB]

---

### Requirement 3: Preview Mode Activation (PREVIEW Command)

**User Story:** As an operator working with mainframe spool files, I want to activate a report preview mode that renders the file as it would have looked when printed, so that I can read the report content without interpreting control characters manually.

**Source:** [FFE-ASA] Requirement 1. Cross-references: `custom-file-viewers` (PREVIEW command, Viewer_Registry), `command-framework` (command registration).

#### Acceptance Criteria

1. THE ASA report preview SHALL be activated via the unified `PREVIEW` command defined in the `custom-file-viewers` spec, using the Viewer_Key `"asa-report"`. Direct commands `PREVIEW asa-report`, `PREVIEW ON` (when ASA is the language default), and `PREVIEW` (toggle) SHALL all activate this viewer through that framework. [FFE-ASA]

2. THE `PREVIEW` state SHALL be displayed in the status bar as `Viewer: asa-report` when active, consistent with the `custom-file-viewers` Requirement 3.7. [FFE-ASA]

3. WHEN sequence number stripping (per the `sequence-numbers` spec) is active for the current file, THE ASA report preview SHALL operate on the post-strip Edit_Buffer content. Column 1 of the stripped buffer contains ASA control characters, not sequence digits. [FFE-ASA]

4. Preview_Mode SHALL be valid in both Browse mode and Edit mode. In Edit mode, the preview renders the current Edit_Buffer content; in Browse mode, it renders the on-disk content. [FFE-ASA]

5. THE `PREVIEW` command and the `"asa-report"` Viewer_Key SHALL be registered with the command-framework at crate initialisation, making the viewer discoverable via `PREVIEW LIST`. [WB]

6. WHEN Preview_Mode is activated, THE system SHALL parse the entire document to build a page index (mapping page numbers to document line numbers) for efficient page navigation. [WB]

---

### Requirement 4: Page Break Rendering

**User Story:** As an operator, I want page breaks to be rendered as visually distinct full-width bands so that I can immediately see the page structure of the report.

**Source:** [FFE-ASA] Requirement 2. Cross-references: `theme-and-appearance` (colour tokens), `layout-and-docking` (panel rendering area).

#### Acceptance Criteria

1. WHEN Preview_Mode is active and a line with ASA control character `1` is encountered, THE system SHALL render a full-width Page_Band in the viewport at that position instead of the line's data content appearing immediately — the Page_Band appears before the line's data. [FFE-ASA]

2. THE Page_Band SHALL span the full width of the editing area and SHALL display the text `─── PAGE N ───` centred within it, where N is the sequential page number starting from 1. [FFE-ASA]

3. THE Page_Band background colour SHALL alternate between two visually distinct colours on consecutive pages — configurable via theme tokens `asa.page_band_odd` and `asa.page_band_even`. Default: muted blue for odd pages, lighter blue for even pages. [FFE-ASA]

4. THE Page_Band SHALL be a display artifact only — it is NOT a real document line and SHALL NOT be editable, selectable as text, or saved to disk. [FFE-ASA]

5. THE first `1` character in the file (if present) SHALL render a Page_Band before the first line of data content, labelled `PAGE 1`. [FFE-ASA]

6. WHEN the file begins with data lines before the first `1` character, those lines SHALL be treated as belonging to a pre-page-1 section and no Page_Band SHALL be inserted before them. [FFE-ASA]

7. THE Page_Band SHALL include the configurable page dimensions in a subtle annotation (e.g., `132×60`) when the operator has customised page dimensions from the default. [WB]

---

### Requirement 5: Overstrike Line Merging

**User Story:** As an operator, I want overprint lines to be merged with the preceding line so that bold text and underlines are rendered correctly, simulating the way a line printer would have produced them.

**Source:** [FFE-ASA] Requirement 4. Cross-references: `theme-and-appearance` (bold/underline font styles), `document-model` (line content access).

#### Acceptance Criteria

1. WHEN Preview_Mode is active and a line has ASA control character `+` (plus), THE system SHALL NOT render that line as a separate row. Instead, THE system SHALL merge it with the most recently rendered non-overprint line to produce a Merged_Line. [FFE-ASA]

2. WHEN merging an Overprint_Line with its base line, THE system SHALL apply the following rules character by character: [FFE-ASA]
  - IF the overprint character is the same as the base character at that column → render the character in **bold** weight.
  - IF the overprint character is `-` or `_` and the base character is a printable non-space character → render the base character with an **underline** style.
  - IF the overprint character is `-` or `_` and the base character is a space → render a dash or underscore at that position (creates a rule/underline on blank space).
  - IF the overprint character is any other printable character and differs from the base character → render the overprint character (superimposition — last overprint wins).
  - IF the overprint character is a space → leave the base character unchanged.

3. WHEN multiple consecutive `+` lines follow a base line, THE system SHALL merge all of them into the same Merged_Line sequentially, applying each merge pass in document order. [FFE-ASA]

4. THE Merged_Line SHALL be displayed in place of the original base line. The overprint source lines SHALL NOT appear as separate rows in the preview. [FFE-ASA]

5. WHEN Preview_Mode is active and a `+` line appears as the very first line in the file (no preceding base line to merge with), THE system SHALL render it as a regular line with no merging and display a diagnostic `[OVERPRINT — no preceding line]` in the prefix area. [FFE-ASA]

6. THE bold and underline styles applied during overstrike merging SHALL use theme-provided font weight and decoration tokens from `theme-and-appearance`, ensuring consistent rendering across themes. [WB]

---

### Requirement 6: Print Preview Panel (Paginated View)

**User Story:** As an operator reviewing a multi-page report, I want a dedicated print preview panel that shows the report in a paginated layout with headers and footers, so that I can see exactly how each page would have appeared on paper.

**Source:** [WB]. Cross-references: `layout-and-docking` (dockable panel), `theme-and-appearance` (page rendering), `command-framework` (panel activation).

#### Acceptance Criteria

1. THE system SHALL provide a Print_Preview_Panel that renders the report in a paginated, page-at-a-time layout within a dockable panel managed by `layout-and-docking`. [WB]

2. THE Print_Preview_Panel SHALL display one logical page at a time, bounded by the configured Page_Width and Page_Depth dimensions, with a visible page border/frame. [WB]

3. THE Print_Preview_Panel SHALL display a page header area showing the current page number, total page count, and optionally the first line of data on the page (as a report title hint). [WB]

4. THE Print_Preview_Panel SHALL display a page footer area showing navigation controls: previous page, next page, go-to-page input, and page N of M indicator. [WB]

5. THE Print_Preview_Panel SHALL be activated via the command `PREVIEW PANEL` or by activating the ASA preview and opening the panel from the View menu or a toolbar button. [WB]

6. THE Print_Preview_Panel content SHALL update in real time when the Edit_Buffer changes (in Edit mode), reflecting edits to the underlying source. [WB]

7. THE Print_Preview_Panel SHALL support zoom in/out to show the page at different scales (fit width, fit page, 50%–200%). [WB]

8. WHEN the operator double-clicks a line in the Print_Preview_Panel, THE system SHALL navigate the main editor viewport to the corresponding source line in the document. [WB]

---

### Requirement 7: ASA Strip/Restore on Edit

**User Story:** As an operator editing a spool file, I want ASA control characters to be transparently stripped from the editable view so that I can work with clean data content, with the original controls restored when I save.

**Source:** [WB]. Cross-references: `document-model` (edit buffer manipulation), `file-operations` (save pipeline), `undo-redo-transactions` (undoable strip operation).

#### Acceptance Criteria

1. WHEN ASA_Strip mode is activated (via `ASA STRIP` command or auto-activation on file open when configured), THE system SHALL remove the ASA control character from column 1 of every line in the Edit_Buffer, shifting all content one column to the left. [WB]

2. THE system SHALL preserve the original ASA control characters in a parallel metadata structure (ASA_Control_Map), keyed by document line number, enabling restoration on save. [WB]

3. WHEN the operator saves a file that has been ASA-stripped, THE system SHALL restore the original ASA control characters to column 1 of each line before writing to disk (ASA_Restore), producing output identical in structure to the original file. [WB]

4. WHEN new lines are inserted during editing of an ASA-stripped file, THE system SHALL assign a default ASA control character of space (` `) to the new lines in the ASA_Control_Map. [WB]

5. WHEN lines are deleted during editing of an ASA-stripped file, THE system SHALL remove the corresponding entries from the ASA_Control_Map. [WB]

6. THE ASA_Strip operation SHALL be recorded as an undoable transaction. Undoing the strip SHALL restore column 1 control characters to the Edit_Buffer and discard the ASA_Control_Map. [WB]

7. THE status bar SHALL display `ASA:Stripped` when ASA_Strip mode is active, so the operator knows that column 1 control characters have been removed from the visible content. [WB]

8. THE `ASA STRIP` and `ASA RESTORE` commands SHALL be registered with the command-framework. `ASA STRIP` activates strip mode; `ASA RESTORE` explicitly re-inserts the control characters into the Edit_Buffer. [WB]

9. WHEN ASA_Strip mode is active and Preview_Mode is also active, THE preview SHALL use the ASA_Control_Map (not column 1 of the edit buffer) to determine carriage control actions, since column 1 no longer contains control characters. [WB]

---

### Requirement 8: Line Printer Emulation (Page Dimensions)

**User Story:** As an operator, I want the preview to emulate standard line printer page dimensions (132 columns × 60 lines) so that the report pagination matches what the mainframe printer would have produced, with the ability to configure different printer models.

**Source:** [WB]. Cross-references: `configuration-system` (page dimension settings), `theme-and-appearance` (monospace font metrics).

#### Acceptance Criteria

1. THE system SHALL define default line printer page dimensions of 132 columns wide (Page_Width) and 60 lines deep (Page_Depth), emulating the IBM 1403/3211 standard line printer output. [WB]

2. WHEN Preview_Mode is active, THE system SHALL enforce the configured Page_Width by truncating or soft-wrapping lines that exceed the page width, with the behaviour controlled by a `page_overflow` setting (`truncate` or `wrap`, default `truncate`). [WB]

3. WHEN Preview_Mode is active and no explicit `1` (page eject) characters are present in the file, THE system SHALL insert implicit page breaks every Page_Depth lines, simulating continuous-form paper with a fixed page length. [WB]

4. WHEN both explicit `1` page-eject characters and implicit page-depth boundaries apply, explicit `1` characters SHALL take priority — implicit page breaks are only inserted in sections without explicit page control. [WB]

5. THE configurable page dimensions SHALL be settable via the `[asa_preview]` section in configuration: [WB]
  - `page_width`: positive integer, default 132 — character columns per page
  - `page_depth`: positive integer, default 60 — print lines per page
  - `page_overflow`: `"truncate"` or `"wrap"`, default `"truncate"`

6. THE system SHALL support named printer profiles that bundle page dimensions and behaviour: [WB]
  - `"ibm-1403"`: 132 × 60 (default)
  - `"ibm-3800"`: 132 × 60
  - `"ibm-4245"`: 132 × 66
  - `"custom"`: operator-defined dimensions

7. THE `PREVIEW SET PRINTER <profile>` command SHALL switch the active printer profile, immediately re-rendering the preview with the new dimensions. [WB]

8. THE Print_Preview_Panel (Requirement 6) SHALL render pages at the configured dimensions, showing the page boundary as a visible frame and displaying line numbers relative to the page (1 through Page_Depth). [WB]

---

### Requirement 9: Line Band Shading (Green-Bar Paper Simulation)

**User Story:** As an operator, I want the preview to apply alternating shading to groups of lines to simulate the green-bar or blue-bar fanfold paper used on classic line printers, making it easier to track across long report lines.

**Source:** [FFE-ASA] Requirement 5. Cross-references: `theme-and-appearance` (colour tokens), `configuration-system` (band settings).

#### Acceptance Criteria

1. WHEN Preview_Mode is active, THE system SHALL apply alternating background shading to groups of N consecutive data lines (default N = 5), simulating bar paper. [FFE-ASA]

2. THE shading SHALL alternate between a slightly tinted background and the default background colour. The tinted colour SHALL be a subtle, low-contrast shade (e.g., very light green or very light blue) that does not obscure the text. [FFE-ASA]

3. THE shading groups SHALL restart at each page boundary (each `1` control character / Page_Band). Line 1 of each page always starts in the first shading group. [FFE-ASA]

4. Blank lines inserted for spacing (Requirement 1) SHALL participate in the band shading count — a blank spacing line counts as one line for shading purposes. [FFE-ASA]

5. Page_Bands (Requirement 4) SHALL NOT be counted in the shading group — the band counter resets at each page break. [FFE-ASA]

6. THE line band shading colours and band size SHALL be configurable via theme tokens (`asa.line_band_tint`, `asa.line_band_size`) and the `[asa_preview]` configuration section, allowing operators to match their preferred paper style (green-bar, blue-bar, or none). [FFE-ASA], [WB]

---

### Requirement 10: Preview Navigation

**User Story:** As an operator, I want to navigate a report preview by page rather than by raw line number, so that I can jump directly to a specific page of a large report.

**Source:** [FFE-ASA] Requirement 6. Cross-references: `command-framework` (command registration), `navigation-commands` (LOCATE integration).

#### Acceptance Criteria

1. WHEN Preview_Mode is active, THE command-framework SHALL support `LOCATE PAGE n` as a navigation command that scrolls the preview viewport to the Page_Band for page number n. [FFE-ASA]

2. WHEN `LOCATE PAGE n` is issued and page n does not exist (n exceeds the total page count), THE system SHALL display `Page n not found — report has M pages` and leave the viewport position unchanged. [FFE-ASA]

3. THE status bar SHALL display the current page number and total page count when Preview_Mode is active (e.g., `Preview: Page 3 of 47`). [FFE-ASA]

4. THE `UP` and `DOWN` navigation commands SHALL scroll the preview by screen height as in standard mode. Additionally, `UP PAGE` and `DOWN PAGE` SHALL move to the previous/next Page_Band (one report page at a time). [FFE-ASA]

5. WHEN Preview_Mode is active, THE `FIND` command SHALL search the data content of lines (excluding ASA control characters and Page_Band labels) and highlight matching text in the preview. [FFE-ASA]

6. THE `LOCATE PAGE FIRST` and `LOCATE PAGE LAST` shortcut commands SHALL navigate to the first and last pages of the report respectively. [WB]

---

### Requirement 11: Export to PDF/Text with ASA Interpretation

**User Story:** As an operator, I want to export the rendered preview to PDF or plain text so that I can share a readable version of the report without the ASA control characters, preserving the page structure and formatting.

**Source:** [FFE-ASA] Requirement 7, [WB]. Cross-references: `command-framework` (export command), `file-operations` (file writing), `workflow-engine` (export progress).

#### Acceptance Criteria

1. THE command-framework SHALL support `PREVIEW EXPORT TEXT <path>` which writes the rendered preview content to the specified file path as UTF-8 plain text. [FFE-ASA]

2. THE command-framework SHALL support `PREVIEW EXPORT PDF <path>` which writes the rendered preview to the specified file path as a PDF document with pages matching the configured page dimensions. [WB]

3. THE text export SHALL represent page breaks as a row of dashes or form-feed characters (configurable: `"dashes"` → `--- PAGE N ---`, `"formfeed"` → ASCII FF character). [FFE-ASA], [WB]

4. THE text export SHALL represent double spacing as blank lines and triple spacing as two blank lines, consistent with the preview rendering. [FFE-ASA]

5. THE text export SHALL represent overprint merging as plain text — bold and underline markers SHALL NOT be included. The merged character content SHALL be written as plain characters. [FFE-ASA]

6. THE PDF export SHALL render pages at the configured Page_Width × Page_Depth dimensions using a monospace font, preserving: [WB]
  - Page breaks as PDF page boundaries
  - Bold text from overstrike merging as bold font weight
  - Underlined text from overstrike merging as underline decoration
  - Line spacing (double/triple) as vertical whitespace
  - Line band shading as alternating background fills (if enabled)

7. WHEN export succeeds, THE system SHALL display a status message with the output file path and page count. [FFE-ASA]

8. WHEN export fails (I/O error, permission denied, invalid path), THE system SHALL display an error message describing the failure without crashing. [FFE-ASA]

9. FOR large files (> 1000 pages), THE export operation SHALL run asynchronously via the workflow-engine, providing progress reporting and cancellation support. [WB]

---

### Requirement 12: Preview Configuration

**User Story:** As an operator, I want to configure the preview appearance and behaviour in configuration so that I can adjust colours, dimensions, and display options to match my preferences and printer model.

**Source:** [FFE-ASA] Requirement 8, [WB]. Cross-references: `configuration-system` (TOML settings, hot-reload), `theme-and-appearance` (colour tokens).

#### Acceptance Criteria

1. THE configuration-system SHALL accept an `[asa_preview]` section with the following optional keys: [FFE-ASA], [WB]
  - `page_width`: positive integer, default 132 — character columns per page
  - `page_depth`: positive integer, default 60 — print lines per page
  - `page_overflow`: `"truncate"` or `"wrap"`, default `"truncate"` — handling of lines exceeding page width
  - `band_size`: positive integer, default 5 — number of lines per shading band
  - `show_line_bands`: boolean, default `true` — whether to show alternating line shading
  - `auto_detect`: boolean, default `true` — whether to run ASA auto-detection on file open
  - `auto_strip`: boolean, default `false` — whether to automatically strip ASA column on file open
  - `detection_threshold`: float 0.0–1.0, default 0.8 — minimum ratio for ASA detection confidence
  - `detection_sample_size`: positive integer, default 50 — number of lines to sample for detection
  - `printer_profile`: string, default `"ibm-1403"` — named printer profile
  - `export_page_separator`: `"dashes"` or `"formfeed"`, default `"dashes"` — text export page break style
  - `implicit_page_breaks`: boolean, default `true` — whether to insert page breaks at Page_Depth intervals when no explicit `1` controls exist

2. WHEN a configuration key contains an invalid value (negative number, unknown string, out-of-range float), THE system SHALL emit a WARN-level configuration diagnostic and apply the default for that key. [FFE-ASA], [WB]

3. THE theme-and-appearance system SHALL define the following ASA-specific colour tokens with sensible defaults: [WB]
  - `asa.page_band_odd` — Page_Band background for odd pages (default: muted blue)
  - `asa.page_band_even` — Page_Band background for even pages (default: lighter blue)
  - `asa.page_band_text` — Page_Band label text colour (default: white)
  - `asa.line_band_tint` — Line band shading tint (default: very light green)
  - `asa.halt_band` — Printer halt band colour (default: amber)
  - `asa.halt_band_text` — Printer halt band text colour (default: black)

4. WHEN configuration values change at runtime (hot-reload), THE system SHALL re-render the active preview with the updated settings without requiring the operator to deactivate and reactivate Preview_Mode. [WB]
