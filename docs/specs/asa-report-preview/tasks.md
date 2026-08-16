# Implementation Plan: ASA Report Preview (`ff-asa-report-preview`)

## Overview

This plan covers the complete implementation of the `ff-asa-report-preview` crate — the ASA carriage control interpretation and print preview subsystem for FileForgeWorkbench. The crate provides visual rendering of mainframe spool files as they would have appeared on a line printer, including page breaks, line spacing, overprint merging (bold/underline), green-bar paper simulation, paginated preview panels, strip/restore editing, and export to PDF/text.

This is a **Wave 12 (FileForge Domain)** sub-project that depends on:
- `ff-document-model` (Wave 4) for edit buffer access and line content
- `ff-command` (Wave 5) for command registration and dispatch
- `ff-layout` (Wave 2) for print preview panel docking
- `ff-config` (Wave 2) for page dimensions, colours, and preview settings
- `ff-theme` (Wave 6) for page band colours, line-band shading, font rendering
- `ff-custom-viewers` (Wave 12) for Viewer_Registry integration and PREVIEW command routing
- `ff-fileforge` (Wave 12) for ASA detection hooks and RECFM metadata

The crate is **GUI-independent** — all ASA parsing, merging, pagination, and export logic operates on the document model without GUI framework dependency. Rendering hints are provided to the UI layer.

---

## Tasks

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-asa-report-preview/Cargo.toml` with dependencies (thiserror, proptest dev-dep) and dependencies on `ff-document-model`, `ff-command`, `ff-layout`, `ff-config`, `ff-theme`, `ff-custom-viewers`, `ff-fileforge`, `ff-logging`
  - [ ] 1.2 Create `crates/ff-asa-report-preview/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `control.rs`, `detection.rs`, `page_model.rs`, `page_index.rs`, `merge.rs`, `preview_hints.rs`, `strip_restore.rs`, `printer.rs`, `printer_config.rs`, `line_bands.rs`, `navigation.rs`, `export_text.rs`, `export_pdf.rs`, `commands.rs`, `config.rs`, `error.rs`, `types.rs`
  - [ ] 1.4 Add `ff-asa-report-preview` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [ ] 2. ASA control types and error definitions
  - [ ] 2.1 Define `AsaControl` enum with variants: Space, DoubleSpace, TripleSpace, PageEject, Overstrike, Halt, Unknown(char)
  - [ ] 2.2 Implement `AsaControl::from_char(c: char) -> Self` parser mapping column-1 characters to control variants
  - [ ] 2.3 Implement `AsaControl::spacing_lines(&self) -> u8` returning the number of blank lines to insert before the data line (0, 1, 2)
  - [ ] 2.4 Define `PageNumber(u32)` newtype with Display, arithmetic, and From<u32>
  - [ ] 2.5 Define `PageDepth(u16)` and `PageWidth(u16)` validated newtypes with TryFrom, enforcing minimum values
  - [ ] 2.6 Define `AsaError` enum with variants: UnrecognisedControl { ch: char, line: usize }, OverprintNoBaseLine { line: usize }, DetectionTimeout, ExportIoError(std::io::Error), InvalidConfiguration { key: String, reason: String }
  - [ ] 2.7 Write unit tests for AsaControl parsing (all valid chars + unknown), spacing calculation, newtype validation
  - Covers: Requirement 1 (AC 1.1, 1.9), Requirement 8 (AC 8.1, 8.5)

- [ ] 3. ASA detection engine
  - [ ] 3.1 Implement `AsaDetector` struct holding configuration: sample_size (default 50), threshold (default 0.8)
  - [ ] 3.2 Implement `detect(lines: &[&str]) -> AsaDetectionResult` that examines column 1 of first N non-blank lines and calculates ASA confidence ratio
  - [ ] 3.3 Implement classification logic: file is ASA-controlled when confidence >= threshold AND at least one `1` (page eject) is present in the sample
  - [ ] 3.4 Implement RECFM metadata bypass: when RECFM is "FBA" or "VBA", return ASA-controlled unconditionally without heuristic scan
  - [ ] 3.5 Implement `AsaDetectionResult` struct with fields: is_asa (bool), confidence (f64), page_eject_found (bool), sample_size_used (usize), bypass_reason (Option<String>)
  - [ ] 3.6 Implement async detection wrapper for files > 1 MB that runs the scan on a background thread and returns a Future
  - [ ] 3.7 Implement configurable threshold and sample size from `[asa_preview]` config section
  - [ ] 3.8 Write unit tests for detection at various confidence levels, threshold boundary, RECFM bypass, empty files, files with no page ejects
  - Covers: Requirement 2 (AC 2.1–2.7), Requirement 12 (AC 12.1)

- [ ] 4. Page model and page index
  - [ ] 4.1 Define `PageEntry` struct with fields: page_number (PageNumber), start_line (usize), end_line (usize), has_explicit_break (bool)
  - [ ] 4.2 Define `PageIndex` struct holding Vec<PageEntry> and total_page_count, with O(1) page-number-to-line lookup via binary search
  - [ ] 4.3 Implement `PageIndex::build(lines: &[AsaControl], page_depth: PageDepth) -> Self` that scans all ASA controls to build the page index
  - [ ] 4.4 Implement explicit page break handling: each `1` control starts a new PageEntry
  - [ ] 4.5 Implement implicit page break insertion: when no explicit `1` exists in a section, insert breaks every page_depth lines
  - [ ] 4.6 Implement mixed mode: explicit `1` breaks take priority; implicit breaks only fill gaps between explicit breaks
  - [ ] 4.7 Implement pre-page-1 section handling: data lines before the first `1` belong to a pre-page section (no Page_Band before them)
  - [ ] 4.8 Implement `page_for_line(line: usize) -> Option<PageNumber>` reverse lookup
  - [ ] 4.9 Implement `line_range_for_page(page: PageNumber) -> Option<Range<usize>>` forward lookup
  - [ ] 4.10 Implement incremental page index update on buffer change (insert/delete lines) without full rebuild
  - [ ] 4.11 Write unit tests for page index construction, explicit/implicit/mixed page breaks, pre-page-1 handling, lookups, incremental update
  - Covers: Requirement 3 (AC 3.6), Requirement 4 (AC 4.1, 4.5, 4.6), Requirement 8 (AC 8.3, 8.4)

- [ ] 5. Overstrike merge engine
  - [ ] 5.1 Define `MergeStyle` enum with variants: Normal, Bold, Underline, Overwritten(char)
  - [ ] 5.2 Define `MergedChar` struct with fields: character (char), style (MergeStyle)
  - [ ] 5.3 Define `MergedLine` struct holding Vec<MergedChar> representing the final merged output
  - [ ] 5.4 Implement `merge_overprint(base: &str, overprint: &str) -> MergedLine` applying character-by-character merge rules:
    - Same char as base → Bold
    - `-` or `_` over printable non-space → Underline on base char
    - `-` or `_` over space → dash/underscore at that position
    - Different printable char over base → Overwritten (last wins)
    - Space over base → leave base unchanged
  - [ ] 5.5 Implement multi-overprint merge: `merge_all(base: &str, overprints: &[&str]) -> MergedLine` applying each overprint sequentially
  - [ ] 5.6 Implement edge case: overprint line longer than base — extend MergedLine to overprint length with overprint chars
  - [ ] 5.7 Implement edge case: overprint line shorter than base — leave remaining base chars unchanged
  - [ ] 5.8 Implement first-line overprint handling: `+` as first line in file renders as normal line with diagnostic prefix `[OVERPRINT — no preceding line]`
  - [ ] 5.9 Write unit tests for all merge rules, multi-overprint, length mismatches, first-line edge case
  - Covers: Requirement 5 (AC 5.1–5.6)

- [ ] 6. Preview rendering hints (GUI-independent display model)
  - [ ] 6.1 Define `PreviewElement` enum with variants: DataLine { content: MergedLine, line_band_group: u32 }, SpacingLine { line_band_group: u32 }, PageBand { page_number: PageNumber, is_odd: bool, dimensions_annotation: Option<String> }, HaltBand
  - [ ] 6.2 Define `PreviewPage` struct holding Vec<PreviewElement> for a single logical page
  - [ ] 6.3 Implement `build_preview(lines: &[String], controls: &[AsaControl], page_index: &PageIndex, config: &AsaPreviewConfig) -> Vec<PreviewElement>` that transforms raw document lines into the full preview element sequence
  - [ ] 6.4 Implement spacing line insertion: 1 blank PreviewElement for `0`, 2 blanks for `-`
  - [ ] 6.5 Implement overstrike suppression: `+` lines are consumed by merge engine, not emitted as separate PreviewElements
  - [ ] 6.6 Implement Page_Band generation with alternating odd/even flag and centred `─── PAGE N ───` label
  - [ ] 6.7 Implement Halt band generation for `H` control with `─── PRINTER HALT ───` label
  - [ ] 6.8 Implement page dimension annotation on Page_Band when dimensions differ from default (132×60)
  - [ ] 6.9 Implement unknown control character fallback: treat as Space, emit WARN diagnostic
  - [ ] 6.10 Verify spacing lines and page bands are marked as non-editable, non-selectable display artifacts
  - [ ] 6.11 Write unit tests for complete preview generation with mixed controls, overstrike suppression, page band alternation, halt band, artifact markers
  - Covers: Requirement 1 (AC 1.2–1.9), Requirement 4 (AC 4.1–4.7), Requirement 5 (AC 5.4)

- [ ] 7. ASA strip/restore engine
  - [ ] 7.1 Define `AsaControlMap` struct (BTreeMap<usize, AsaControl>) storing the stripped control character for each document line number
  - [ ] 7.2 Implement `strip(buffer: &mut EditBuffer) -> AsaControlMap` that removes column 1 from every line, shifts content left, and records original controls in the map
  - [ ] 7.3 Implement `restore(buffer: &mut EditBuffer, map: &AsaControlMap)` that re-inserts the control character at column 1 of each line using the map
  - [ ] 7.4 Implement new-line insertion handling: assign default Space control to inserted lines in the map
  - [ ] 7.5 Implement line deletion handling: remove the corresponding entry from the AsaControlMap
  - [ ] 7.6 Implement line reordering propagation: update line-number keys when lines are moved (block move, sort)
  - [ ] 7.7 Implement undo integration: strip is recorded as an undoable transaction; undo restores column 1 chars and discards the map
  - [ ] 7.8 Implement status bar indicator query: `is_stripped() -> bool` for `ASA:Stripped` display
  - [ ] 7.9 Implement preview interaction: when stripped, preview reads controls from AsaControlMap instead of column 1
  - [ ] 7.10 Write unit tests for strip/restore round-trip, new line defaults, deletion cleanup, undo cycle, preview-from-map
  - Covers: Requirement 7 (AC 7.1–7.9)

- [ ] 8. Printer profiles and page dimension configuration
  - [ ] 8.1 Define `PrinterProfile` struct with fields: name (String), page_width (PageWidth), page_depth (PageDepth), description (String)
  - [ ] 8.2 Implement built-in profiles: `"ibm-1403"` (132×60), `"ibm-3800"` (132×60), `"ibm-4245"` (132×66), `"custom"` (operator-defined)
  - [ ] 8.3 Implement `PageOverflow` enum with variants: Truncate, Wrap (default Truncate)
  - [ ] 8.4 Implement page width enforcement in preview: truncate lines exceeding PageWidth or soft-wrap based on PageOverflow setting
  - [ ] 8.5 Implement `PREVIEW SET PRINTER <profile>` handler that switches active profile and triggers preview re-render
  - [ ] 8.6 Implement configuration loading from `[asa_preview]` section: page_width, page_depth, page_overflow, printer_profile
  - [ ] 8.7 Implement invalid configuration value handling: emit WARN diagnostic and apply defaults
  - [ ] 8.8 Write unit tests for profile loading, page width enforcement (truncate and wrap), profile switching, invalid config handling
  - Covers: Requirement 8 (AC 8.1–8.8), Requirement 12 (AC 12.1, 12.2)

- [ ] 9. Line band shading engine
  - [ ] 9.1 Define `LineBandConfig` struct with fields: band_size (u8, default 5), show_bands (bool, default true), tint_token (String)
  - [ ] 9.2 Implement `assign_band_groups(elements: &mut [PreviewElement], band_size: u8)` that assigns line_band_group values to data lines and spacing lines within each page
  - [ ] 9.3 Implement page-boundary reset: band counter restarts at 0 on each PageBand element
  - [ ] 9.4 Implement spacing line participation: blank spacing lines count toward band group progression
  - [ ] 9.5 Implement PageBand exclusion: page bands do not participate in band counting
  - [ ] 9.6 Implement `band_group_is_tinted(group: u32) -> bool` returning true for alternating groups (group % 2 == 1)
  - [ ] 9.7 Implement configurable band size and tint colour from config and theme tokens
  - [ ] 9.8 Write unit tests for band assignment, page reset, spacing participation, alternation, configurable band size
  - Covers: Requirement 9 (AC 9.1–9.6), Requirement 12 (AC 12.1)

- [ ] 10. Print preview panel (paginated view model)
  - [ ] 10.1 Define `PrintPreviewState` struct with fields: current_page (PageNumber), total_pages (u32), zoom_level (f32), page_elements (Vec<PreviewElement>)
  - [ ] 10.2 Implement `render_page(page: PageNumber) -> PreviewPage` that extracts elements for a single logical page bounded by PageWidth and PageDepth
  - [ ] 10.3 Implement page header model: current page number, total page count, first-line title hint
  - [ ] 10.4 Implement page footer model: navigation state (prev/next enabled), page N of M text
  - [ ] 10.5 Implement zoom level support: fit_width, fit_page, 50%–200% scale factors
  - [ ] 10.6 Implement real-time update: re-render preview page when edit buffer changes
  - [ ] 10.7 Implement double-click-to-source mapping: preview element index → source document line number
  - [ ] 10.8 Implement `PREVIEW PANEL` command handler that activates the dockable panel via layout-and-docking
  - [ ] 10.9 Implement page dimension display in panel frame showing line numbers relative to page (1 through PageDepth)
  - [ ] 10.10 Write unit tests for page extraction, header/footer content, zoom, source mapping, real-time update triggers
  - Covers: Requirement 6 (AC 6.1–6.8), Requirement 8 (AC 8.8)

- [ ] 11. Preview navigation
  - [ ] 11.1 Implement `locate_page(page: PageNumber, page_index: &PageIndex) -> Result<usize, AsaError>` that returns the document line for a given page number
  - [ ] 11.2 Implement `LOCATE PAGE n` command handler that scrolls viewport to the Page_Band for page n
  - [ ] 11.3 Implement page-not-found error: display `Page n not found — report has M pages` when n exceeds total
  - [ ] 11.4 Implement `LOCATE PAGE FIRST` and `LOCATE PAGE LAST` shortcut commands
  - [ ] 11.5 Implement `UP PAGE` / `DOWN PAGE` navigation that moves to previous/next Page_Band
  - [ ] 11.6 Implement status bar page indicator: `Preview: Page N of M` when preview mode is active
  - [ ] 11.7 Implement FIND command integration: search data content excluding ASA controls and Page_Band labels, highlight matches in preview
  - [ ] 11.8 Write unit tests for page location, boundary navigation, error messages, status bar state, FIND filtering
  - Covers: Requirement 10 (AC 10.1–10.6)

- [ ] 12. Export to plain text
  - [ ] 12.1 Implement `export_text(elements: &[PreviewElement], config: &ExportConfig) -> String` that renders the preview as UTF-8 plain text
  - [ ] 12.2 Implement page break representation: configurable as dashes (`--- PAGE N ---`) or form-feed (ASCII FF char)
  - [ ] 12.3 Implement spacing representation: double spacing → 1 blank line, triple spacing → 2 blank lines
  - [ ] 12.4 Implement overprint merge output: plain text characters only — no bold/underline markers; merged content written as plain chars
  - [ ] 12.5 Implement `PREVIEW EXPORT TEXT <path>` command handler that writes export to file and reports success with page count
  - [ ] 12.6 Implement export failure handling: display error message without crashing on I/O error, permission denied, or invalid path
  - [ ] 12.7 Write unit tests for text export format, page separators (both modes), spacing, merge flattening, error handling
  - Covers: Requirement 11 (AC 11.1, 11.3–11.5, 11.7, 11.8)

- [ ] 13. Export to PDF
  - [ ] 13.1 Implement `export_pdf(elements: &[PreviewElement], config: &ExportConfig, path: &Path) -> Result<(), AsaError>` using a PDF generation library (e.g., `printpdf` or `genpdf`)
  - [ ] 13.2 Implement page rendering: each logical page becomes a PDF page at configured PageWidth × PageDepth using monospace font
  - [ ] 13.3 Implement bold text rendering from overstrike merge as bold font weight
  - [ ] 13.4 Implement underline text rendering from overstrike merge as underline decoration
  - [ ] 13.5 Implement line spacing as vertical whitespace in PDF output
  - [ ] 13.6 Implement line band shading as alternating background fills when enabled
  - [ ] 13.7 Implement `PREVIEW EXPORT PDF <path>` command handler with success/failure messaging
  - [ ] 13.8 Implement async export with progress reporting for large files (> 1000 pages) via workflow-engine
  - [ ] 13.9 Write unit tests for PDF structure (page count, font usage), error handling, progress reporting trigger
  - Covers: Requirement 11 (AC 11.2, 11.6–11.9)

- [ ] 14. Command registration and PREVIEW integration
  - [ ] 14.1 Register `"asa-report"` Viewer_Key with the custom-file-viewers Viewer_Registry at crate initialisation
  - [ ] 14.2 Implement PREVIEW activation via `PREVIEW asa-report`, `PREVIEW ON` (when ASA default), and `PREVIEW` (toggle) through custom-file-viewers framework
  - [ ] 14.3 Implement status bar indicator: `Viewer: asa-report` when preview mode is active
  - [ ] 14.4 Implement preview availability in both Browse mode (renders on-disk content) and Edit mode (renders edit buffer)
  - [ ] 14.5 Implement sequence-number awareness: preview operates on post-strip buffer when sequence stripping is active
  - [ ] 14.6 Implement `PREVIEW LIST` discoverability: viewer appears in viewer list
  - [ ] 14.7 Implement `ASA STRIP` command: activates strip mode
  - [ ] 14.8 Implement `ASA RESTORE` command: explicitly re-inserts controls into buffer
  - [ ] 14.9 Implement `PREVIEW SET PRINTER <profile>` command routing
  - [ ] 14.10 Implement detection offer: non-blocking status bar prompt when ASA detected, consistent with custom-file-viewers Requirement 2
  - [ ] 14.11 Implement no-ASA warning: display `PREVIEW: no ASA carriage control detected — preview may not render correctly` when PREVIEW ON issued without detected ASA
  - [ ] 14.12 Write unit tests for viewer registration, command dispatch, status bar states, mode availability, detection offer trigger
  - Covers: Requirement 2 (AC 2.4, 2.5), Requirement 3 (AC 3.1–3.5), Requirement 7 (AC 7.7, 7.8)

- [ ] 15. Configuration and theme integration
  - [ ] 15.1 Define `AsaPreviewConfig` struct holding all configurable fields from `[asa_preview]` section with defaults
  - [ ] 15.2 Implement config loading from configuration-system with validation and WARN on invalid values
  - [ ] 15.3 Implement hot-reload: re-render active preview when config changes at runtime without deactivate/reactivate
  - [ ] 15.4 Define theme colour tokens: `asa.page_band_odd`, `asa.page_band_even`, `asa.page_band_text`, `asa.line_band_tint`, `asa.halt_band`, `asa.halt_band_text`
  - [ ] 15.5 Implement theme token resolution at render-hint generation time, providing colour values in PreviewElement metadata
  - [ ] 15.6 Write unit tests for config loading, default application, invalid value fallback, hot-reload trigger
  - Covers: Requirement 12 (AC 12.1–12.4)

- [ ] 16. Property-based tests
  - [ ] 16.1 Write PBT: ASA control character parsing completeness
  - [ ] 16.2 Write PBT: detection confidence threshold boundary correctness
  - [ ] 16.3 Write PBT: page index construction consistency
  - [ ] 16.4 Write PBT: overstrike merge idempotence and character coverage
  - [ ] 16.5 Write PBT: strip/restore round-trip fidelity
  - [ ] 16.6 Write PBT: line band shading assignment correctness
  - [ ] 16.7 Write PBT: text export spacing and page break fidelity
  - [ ] 16.8 Write PBT: preview element count conservation
  - Covers: Requirements 1, 2, 4, 5, 7, 8, 9, 11 (see Property-Based Test Definitions below)

- [ ] 17. Integration tests
  - [ ] 17.1 Write integration test: full preview lifecycle (detect → activate → navigate → deactivate)
  - [ ] 17.2 Write integration test: overstrike merge with multi-line overprint sequences in complete document
  - [ ] 17.3 Write integration test: strip/restore round-trip preserving all ASA controls through edit/save cycle
  - [ ] 17.4 Write integration test: export text and verify output matches rendered preview content
  - [ ] 17.5 Write integration test: page navigation with LOCATE PAGE, UP PAGE, DOWN PAGE commands
  - [ ] 17.6 Write integration test: printer profile switch triggers page index rebuild and re-render
  - [ ] 17.7 Write integration test: config hot-reload updates line band shading and page dimensions live
  - Covers: End-to-end validation across Requirements 1–12

---

## Property-Based Test Definitions

### Property 1: ASA Control Character Parsing Completeness

**Validates: Requirement 1.1, 1.9**

- **Statement:** For any character in the full ASCII range (0x00–0x7F), `AsaControl::from_char` SHALL return: the correct variant for known ASA characters (` `, `0`, `-`, `1`, `+`, `H`), and `Unknown(ch)` for all other characters. The `spacing_lines()` method SHALL return 0 for Space/Overstrike/PageEject/Halt/Unknown, 1 for DoubleSpace, and 2 for TripleSpace.
- **Strategy:** Generate:
  - Input character: any u8 cast to char (0x00–0x7F)
- **Invariant:** Known chars produce their expected variant; unknown chars produce Unknown; spacing values match spec; from_char is total (never panics)

### Property 2: Detection Confidence Threshold Boundary Correctness

**Validates: Requirement 2.1, 2.2, 2.6**

- **Statement:** For any set of lines with a known distribution of valid/invalid ASA first-column characters, the detection algorithm SHALL classify the file as ASA-controlled if and only if: (a) the confidence ratio >= threshold AND (b) at least one `1` character is present in the sample. The classification SHALL be deterministic for the same input.
- **Strategy:** Generate:
  - Line count: [1, 200]
  - Valid ASA ratio: uniform float [0.0, 1.0] determining proportion of valid first-column chars
  - Page eject present: bool (whether at least one line has `1`)
  - Threshold: float [0.5, 1.0]
  - Sample size: [10, 100]
- **Invariant:** `is_asa == (confidence >= threshold && page_eject_found)`; result is deterministic for same input

### Property 3: Page Index Construction Consistency

**Validates: Requirement 4.1, 4.5, 8.3, 8.4**

- **Statement:** For any sequence of ASA controls and any valid page_depth, the constructed PageIndex SHALL satisfy: (a) every document line belongs to exactly one PageEntry, (b) page entries are non-overlapping and cover the entire document, (c) explicit `1` controls always start a new page, and (d) implicit breaks occur at most every page_depth lines.
- **Strategy:** Generate:
  - Line count: [0, 500]
  - ASA controls: random sequence from valid set with configurable `1` frequency
  - Page depth: [10, 80]
- **Invariant:** Union of all PageEntry ranges == [0, line_count); no overlaps; each `1` starts a new entry; max consecutive non-`1` lines in one page <= page_depth

### Property 4: Overstrike Merge Idempotence and Character Coverage

**Validates: Requirement 5.1, 5.2, 5.3**

- **Statement:** For any base line and any sequence of overprint lines, the merge operation SHALL: (a) produce a MergedLine with length >= max(base_len, max overprint_len), (b) be deterministic for the same input sequence, and (c) when the same overprint is applied twice in succession, the second application SHALL not change the result (idempotence of style — double bold is still bold).
- **Strategy:** Generate:
  - Base line: arbitrary printable ASCII string [0, 132] chars
  - Overprint count: [1, 5]
  - Overprint content: arbitrary ASCII strings including `-`, `_`, spaces, and printable chars
- **Invariant:** `merge_all(base, overprints).len() >= max(base.len(), max(overprint.len()))`; `merge_all(base, [op, op]) == merge_all(base, [op])` for style (idempotence); result is deterministic

### Property 5: Strip/Restore Round-Trip Fidelity

**Validates: Requirement 7.1, 7.2, 7.3**

- **Statement:** For any document with valid ASA control characters in column 1, stripping followed by restoring SHALL produce byte-for-byte identical output to the original document. The AsaControlMap SHALL contain exactly one entry per line, and the restored content SHALL match the pre-strip state.
- **Strategy:** Generate:
  - Line count: [1, 300]
  - Line content: random printable ASCII with a random valid ASA control in column 1
- **Invariant:** `restore(strip(document)) == document`; `control_map.len() == line_count`; every line's column 1 after restore matches original

### Property 6: Line Band Shading Assignment Correctness

**Validates: Requirement 9.1, 9.3, 9.4, 9.5**

- **Statement:** For any sequence of PreviewElements with configurable band_size, the band group assignment SHALL satisfy: (a) data lines and spacing lines within a page are grouped into consecutive blocks of band_size, (b) groups alternate (even/odd) for tinting, (c) page bands reset the counter, and (d) page bands themselves do not carry a band group.
- **Strategy:** Generate:
  - Element sequence: random mix of DataLine, SpacingLine, PageBand (10–200 elements)
  - Band size: [1, 10]
- **Invariant:** Within each page section: first band_size data/spacing lines share group 0, next band_size share group 1, etc.; groups increment monotonically within page; page boundary resets to 0; PageBand elements have no group

### Property 7: Text Export Spacing and Page Break Fidelity

**Validates: Requirement 11.3, 11.4, 11.5**

- **Statement:** For any preview element sequence, the text export SHALL: (a) represent each PageBand as exactly one page separator line, (b) represent each SpacingLine as exactly one blank line in output, (c) represent merged content without bold/underline markers (plain chars only), and (d) the output line count SHALL equal: data_lines + spacing_lines + page_bands.
- **Strategy:** Generate:
  - Preview elements: random sequence of DataLine/SpacingLine/PageBand (10–300 elements)
  - Page separator style: Dashes or FormFeed
- **Invariant:** Output lines counted matches formula; no bold/underline escape sequences present; page separators match configured style; spacing lines produce empty lines

### Property 8: Preview Element Count Conservation

**Validates: Requirement 1.2–1.6, 4.4, 5.4**

- **Statement:** For any document of N lines with known ASA controls, the preview element generation SHALL produce exactly: N - overprint_count data lines + spacing_insertions blank lines + page_band_count page bands + halt_count halt bands. No source line SHALL be lost or duplicated.
- **Strategy:** Generate:
  - Line count: [1, 200]
  - ASA controls: random sequence from full valid set
- **Invariant:** `data_elements == N - count_of('+')`, `spacing_elements == sum(spacing_for_each_line)`, `page_bands == count_of('1')`, `halt_bands == count_of('H')`; total accounts for all source lines

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Types", "tasks": ["2"], "dependsOn": [0] },
    { "id": 2, "label": "Detection and Page Model", "tasks": ["3", "4"], "dependsOn": [1] },
    { "id": 3, "label": "Merge and Rendering", "tasks": ["5", "6"], "dependsOn": [2] },
    { "id": 4, "label": "Strip/Restore and Printer", "tasks": ["7", "8"], "dependsOn": [1] },
    { "id": 5, "label": "Shading and Preview Panel", "tasks": ["9", "10"], "dependsOn": [3, 4] },
    { "id": 6, "label": "Navigation", "tasks": ["11"], "dependsOn": [5] },
    { "id": 7, "label": "Export", "tasks": ["12", "13"], "dependsOn": [3, 5] },
    { "id": 8, "label": "Commands and Config", "tasks": ["14", "15"], "dependsOn": [6, 7] },
    { "id": 9, "label": "Validation and PBT", "tasks": ["16", "17"], "dependsOn": [8] }
  ]
}
```

---

## Notes

- This is a Wave 12 (FileForge Domain) crate depending on multiple upstream crates from Waves 2–12
- The crate is GUI-independent — all ASA parsing, merging, pagination, and export logic operates on the document model without egui dependency
- Rendering hints (PreviewElement) are consumed by the UI layer for actual visual rendering
- The PREVIEW command integration uses the `custom-file-viewers` framework — this crate registers itself as a viewer, not as a standalone command
- Strip/Restore is an editing transformation; Preview is a rendering transformation — they can coexist (preview reads from AsaControlMap when stripped)
- PDF export uses a Rust PDF generation crate (e.g., `genpdf` or `printpdf`) — the choice is deferred to implementation
- Async detection (Requirement 2.7) and async export (Requirement 11.9) delegate to the workflow-engine for progress/cancellation
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The `PageNumber(u32)` type supports reports up to ~4 billion pages — more than sufficient for any real spool file
- Line band shading is a display enhancement only — it does not modify document content
- The merge engine handles arbitrary-length overprint chains; there is no artificial limit on consecutive `+` lines
- Configuration hot-reload (Requirement 12.4) uses the configuration-system's change notification mechanism

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: ASA Carriage Control Characters | AC 1.1–1.9 | Tasks 2, 6 |
| Req 2: ASA Auto-Detection | AC 2.1–2.7 | Tasks 3, 14 |
| Req 3: Preview Mode Activation (PREVIEW Command) | AC 3.1–3.6 | Tasks 4, 14 |
| Req 4: Page Break Rendering | AC 4.1–4.7 | Tasks 4, 6 |
| Req 5: Overstrike Line Merging | AC 5.1–5.6 | Tasks 5, 6 |
| Req 6: Print Preview Panel | AC 6.1–6.8 | Task 10 |
| Req 7: ASA Strip/Restore on Edit | AC 7.1–7.9 | Tasks 7, 14 |
| Req 8: Line Printer Emulation (Page Dimensions) | AC 8.1–8.8 | Tasks 4, 8, 10 |
| Req 9: Line Band Shading | AC 9.1–9.6 | Task 9 |
| Req 10: Preview Navigation | AC 10.1–10.6 | Task 11 |
| Req 11: Export to PDF/Text | AC 11.1–11.9 | Tasks 12, 13 |
| Req 12: Preview Configuration | AC 12.1–12.4 | Tasks 8, 15 |
