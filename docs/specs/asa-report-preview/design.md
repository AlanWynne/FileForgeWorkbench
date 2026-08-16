# Design Document: ASA Report Preview (`ff-asa-report-preview`)

## Overview

The `ff-asa-report-preview` crate provides **ASA (ANSI) carriage control interpretation and print preview rendering** for the FileForgeWorkbench editor. It transforms mainframe spool files into a visual representation that simulates how the report would have appeared on a line printer — complete with page breaks, line spacing, overprint (bold/underline), and green-bar paper simulation.

### Purpose

- Parse and interpret ASA carriage control characters (space, 0, -, 1, +, H)
- Auto-detect ASA content through heuristic first-column analysis
- Render paginated preview output with Page_Bands and line band shading
- Merge overstrike lines into bold/underlined display text
- Provide a dockable print preview panel with page navigation
- Support ASA strip/restore for transparent editing of spool files
- Emulate line printer dimensions (132×60 default, configurable profiles)
- Export rendered output to PDF and plain text formats

### Position in Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
├─────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-asa-report-preview ← Wave 12                │
│         (registered as Custom_Viewer "asa-report")           │
├─────────────────────────────────────────────────────────────┤
│  Peer Crates: ff-custom-viewers, ff-fileforge                │
│         (viewer registry, ASA detection hooks)               │
├─────────────────────────────────────────────────────────────┤
│  Core Feature Crates: ff-document-model, ff-command,         │
│    ff-layout, ff-config, ff-theme                            │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence**: All ASA parsing, merging, pagination, and export logic operates on the document model without GUI framework dependency. Rendering hints are provided to the UI layer but no egui dependency exists in core logic.
- **Command-Framework Integrated**: PREVIEW activation, export commands, and page navigation are registered with the command framework, discoverable, and scriptable.
- **Custom-Viewer Compliant**: Registered as `Custom_Viewer` with Viewer_Key `"asa-report"` through the `custom-file-viewers` framework.
- **Read-Only Display**: Preview_Mode is a rendering transformation; it does not modify the Edit_Buffer or the file on disk.
- **Sequence-Aware**: Operates on post-strip content when sequence number stripping is active.
- **Plugin-Extensible**: Registered via the plugin architecture's viewer trait, allowing future ASA variants or custom report renderers.

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Shell [Shell Layer]
        DESKTOP[ff-desktop / egui]
    end

    subgraph ff-asa-report-preview [ff-asa-report-preview Crate]
        PARSER[ASA Parser]
        DETECT[ASA Detector]
        PAGINATOR[Page Paginator]
        MERGER[Overstrike Merger]
        INDEX[Page Index]
        PANEL[Print Preview Panel Logic]
        STRIP[ASA Strip/Restore]
        EXPORT_TEXT[Text Exporter]
        EXPORT_PDF[PDF Exporter]
        SHADING[Line Band Shader]
        PROFILE[Printer Profile]
        VIEWER[Viewer Registration]
    end

    subgraph Upstream [Upstream Crates]
        DOC[ff-document-model]
        CMD[ff-command]
        LAYOUT[ff-layout]
        CONFIG[ff-config]
        THEME[ff-theme]
        VIEWERS[ff-custom-viewers]
        FFG[ff-fileforge]
    end

    DESKTOP -->|render hints| PANEL
    VIEWER -->|register| VIEWERS
    PARSER -->|read lines| DOC
    STRIP -->|mutate buffer| DOC
    PANEL -->|dock panel| LAYOUT
    VIEWER -->|commands| CMD
    PROFILE -->|read settings| CONFIG
    SHADING -->|colour tokens| THEME
    DETECT -->|RECFM metadata| FFG
    PAGINATOR --> INDEX
    MERGER --> PARSER
    EXPORT_TEXT --> PAGINATOR
    EXPORT_PDF --> PAGINATOR
end
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **ASA Parser** | Extracts the ASA control character from column 1 and classifies it into the `AsaControl` enum |
| **ASA Detector** | Heuristic analysis of first-column characters to determine ASA presence |
| **Page Paginator** | Builds a paginated view by interpreting control characters, inserting spacing, and tracking page boundaries |
| **Overstrike Merger** | Combines `+` (overprint) lines with their base line to produce `MergedLine` with bold/underline attributes |
| **Page Index** | Maps page numbers to document line numbers for O(1) page navigation |
| **Print Preview Panel Logic** | GUI-independent page layout and navigation state for the dockable preview panel |
| **ASA Strip/Restore** | Removes/restores column 1 control characters with a parallel `AsaControlMap` metadata structure |
| **Text Exporter** | Writes rendered preview content to UTF-8 plain text with page separators |
| **PDF Exporter** | Writes rendered preview to PDF with monospace font, page boundaries, and formatting |
| **Line Band Shader** | Computes alternating background shading groups for green-bar paper simulation |
| **Printer Profile** | Named printer dimension configurations (IBM 1403, 3800, 4245, custom) |
| **Viewer Registration** | Registers the `"asa-report"` viewer with the `custom-file-viewers` Viewer_Registry |

### Data Flow: Preview Activation

```
1. Operator issues PREVIEW command (or PREVIEW asa-report)
2. Viewer Registration routes to this crate via custom-file-viewers framework
3. ASA Parser scans entire document, extracting AsaControl per line
4. Overstrike Merger identifies consecutive `+` lines and merges them with base lines
5. Page Paginator processes the parsed lines:
   a. Inserts Page_Bands at `1` characters
   b. Inserts spacing lines for `0` and `-` characters
   c. Tracks page depth for implicit page breaks
   d. Applies Line Band Shading to groups of N lines
6. Page Index is built (page number → document line mapping)
7. Rendering hints are emitted for the UI layer (Page_Bands, merged lines, shading)
8. Status bar updated: "Viewer: asa-report"
```

---

## Components and Interfaces

```
crates/ff-asa-report-preview/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── parser.rs               # ASA control character parsing
│   ├── detector.rs             # ASA auto-detection heuristic
│   ├── paginator.rs            # Page pagination logic
│   ├── merger.rs               # Overstrike line merging
│   ├── page_index.rs           # Page number → line number mapping
│   ├── preview_panel.rs        # Print preview panel state and logic
│   ├── strip.rs                # ASA strip/restore operations
│   ├── export/
│   │   ├── mod.rs              # Export re-exports
│   │   ├── text.rs             # Plain text export
│   │   └── pdf.rs              # PDF export
│   ├── shading.rs              # Line band shading computation
│   ├── profile.rs              # Printer profile definitions
│   ├── viewer.rs               # Viewer_Registry integration and commands
│   ├── config.rs               # Configuration parsing and defaults
│   ├── types.rs                # Core data types and newtypes
│   └── error.rs                # AsaPreviewError enum
└── tests/
    ├── parser_tests.rs         # ASA parsing unit + property tests
    ├── detector_tests.rs       # Detection heuristic tests
    ├── paginator_tests.rs      # Pagination logic property tests
    ├── merger_tests.rs         # Overstrike merging property tests
    ├── page_index_tests.rs     # Page index lookup tests
    ├── strip_tests.rs          # Strip/restore round-trip tests
    ├── shading_tests.rs        # Line band computation tests
    ├── export_tests.rs         # Text/PDF export integration tests
    └── integration.rs          # End-to-end preview activation flow
```

---

## Data Models

### Core Enums

```rust
/// ASA carriage control character classification.
/// Represents the printer action encoded in column 1 of each record.
///
/// Addresses: Requirement 1 AC 1.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AsaControl {
    /// Space — single space before printing (normal line advance)
    SingleSpace,
    /// `0` — double space before printing (skip one blank line)
    DoubleSpace,
    /// `-` — triple space before printing (skip two blank lines)
    TripleSpace,
    /// `1` — page eject (advance to top of next page before printing)
    PageEject,
    /// `+` — no advance (overstrike/overprint on previous line)
    Overstrike,
    /// `H` — halt (printer halt indication)
    Halt,
}

impl AsaControl {
    /// Parse a single character into an ASA control classification.
    /// Returns None for unrecognised characters (caller handles fallback).
    pub fn from_char(ch: char) -> Option<Self>;

    /// Number of blank lines to insert before this line's content.
    pub fn spacing_lines(&self) -> usize;

    /// Whether this control starts a new page.
    pub fn is_page_break(&self) -> bool;

    /// Whether this control indicates overstrike merging.
    pub fn is_overstrike(&self) -> bool;
}

/// The set of valid ASA control characters for detection purposes.
pub const ASA_VALID_CHARS: &[char] = &[' ', '0', '-', '1', '+', 'H'];
```

### Character Styling

```rust
/// Styling attributes for a single character in a merged line.
///
/// Addresses: Requirement 5 AC 5.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CharStyle {
    /// Whether this character should be rendered in bold weight
    pub bold: bool,
    /// Whether this character should be rendered with underline decoration
    pub underline: bool,
}

/// A single character with its associated rendering style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyledChar {
    /// The character to display
    pub character: char,
    /// Rendering attributes (bold, underline)
    pub style: CharStyle,
}
```

### Merged Line

```rust
/// The result of merging a base line with one or more overprint lines.
/// Contains character-level styling information for bold and underline rendering.
///
/// Addresses: Requirement 5 AC 5.1–5.4
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergedLine {
    /// The styled characters after all overprint passes have been applied
    pub characters: Vec<StyledChar>,
    /// The original document line number of the base line
    pub source_line: usize,
    /// Number of overprint lines that were merged into this line
    pub overprint_count: usize,
}

impl MergedLine {
    /// Create a MergedLine from a base line (no overprinting applied yet).
    pub fn from_base(content: &str, source_line: usize) -> Self;

    /// Apply an overprint line to this merged line.
    /// Implements the character-by-character merging rules from Req 5.2.
    pub fn apply_overprint(&mut self, overprint_content: &str);

    /// The plain-text content (without styling) for export purposes.
    pub fn plain_text(&self) -> String;

    /// Whether any character in this line has bold styling.
    pub fn has_bold(&self) -> bool;

    /// Whether any character in this line has underline styling.
    pub fn has_underline(&self) -> bool;
}
```

### Page Index

```rust
/// Mapping from page numbers to document line numbers for efficient navigation.
/// Built during initial preview parse; rebuilt when document changes.
///
/// Addresses: Requirement 3 AC 3.6, Requirement 10 AC 10.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageIndex {
    /// Ordered list of page entries: (page_number, first_document_line)
    entries: Vec<PageEntry>,
}

/// A single entry in the page index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageEntry {
    /// 1-based page number
    pub page_number: usize,
    /// 0-based document line number where this page starts
    pub document_line: usize,
    /// Whether this page break is explicit (from `1` control) or implicit (page depth)
    pub is_explicit: bool,
}

impl PageIndex {
    /// Create an empty page index.
    pub fn new() -> Self;

    /// Total number of pages in the index.
    pub fn page_count(&self) -> usize;

    /// Look up the document line for a given page number.
    /// Returns None if page_number is out of range.
    pub fn document_line_for_page(&self, page_number: usize) -> Option<usize>;

    /// Find which page a given document line belongs to.
    pub fn page_for_document_line(&self, document_line: usize) -> usize;

    /// Add a page entry to the index.
    pub fn push(&mut self, entry: PageEntry);

    /// Get all entries as a slice.
    pub fn entries(&self) -> &[PageEntry];
}
```

### Printer Profile

```rust
/// Named printer profile bundling page dimensions and behaviour.
///
/// Addresses: Requirement 8 AC 8.5, 8.6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrinterProfile {
    /// Profile name (e.g., "ibm-1403", "ibm-3800", "custom")
    pub name: String,
    /// Character columns per page
    pub page_width: usize,
    /// Print lines per page
    pub page_depth: usize,
    /// How to handle lines exceeding page width
    pub page_overflow: PageOverflow,
}

/// Behaviour for lines that exceed the configured page width.
///
/// Addresses: Requirement 8 AC 8.2
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageOverflow {
    /// Truncate lines at page width boundary
    Truncate,
    /// Soft-wrap lines that exceed page width
    Wrap,
}

impl PrinterProfile {
    /// IBM 1403 standard: 132×60
    pub fn ibm_1403() -> Self;

    /// IBM 3800 laser: 132×60
    pub fn ibm_3800() -> Self;

    /// IBM 4245 printer: 132×66
    pub fn ibm_4245() -> Self;

    /// Custom profile with user-specified dimensions.
    pub fn custom(page_width: usize, page_depth: usize, overflow: PageOverflow) -> Self;

    /// Look up a profile by name. Returns None for unknown names.
    pub fn from_name(name: &str) -> Option<Self>;
}
```

### Preview State

```rust
/// The parsed representation of a single source line with its ASA control.
///
/// Addresses: Requirement 1 AC 1.1–1.9
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLine {
    /// The ASA control character for this line
    pub control: AsaControl,
    /// The data content (column 2 onwards, or full line if stripped)
    pub content: String,
    /// 0-based document line number in the source buffer
    pub source_line: usize,
}

/// A single element in the rendered preview output.
/// The paginator produces a sequence of these for the UI to render.
///
/// Addresses: Requirements 1, 4, 5, 9
#[derive(Debug, Clone, PartialEq)]
pub enum PreviewElement {
    /// A rendered data line (possibly merged from overstrikes)
    DataLine {
        /// The styled content for rendering
        content: MergedLine,
        /// Which line band shading group this line belongs to (0 or 1)
        band_group: u8,
        /// Page-relative line number (1-based within current page)
        page_line: usize,
    },
    /// A blank spacing line (display artifact, not a real document line)
    SpacingLine {
        /// Which line band shading group this line belongs to (0 or 1)
        band_group: u8,
    },
    /// A page break band
    PageBand {
        /// 1-based page number
        page_number: usize,
        /// Whether this is an explicit (from `1`) or implicit (page depth) break
        is_explicit: bool,
    },
    /// A printer halt warning band
    HaltBand {
        /// 0-based document line number of the halt control
        source_line: usize,
    },
}

/// Complete preview state for a document.
/// Built by the paginator from the parsed document; consumed by the UI layer.
///
/// Addresses: Requirement 3 AC 3.6
#[derive(Debug, Clone)]
pub struct PreviewState {
    /// Ordered sequence of preview elements for rendering
    pub elements: Vec<PreviewElement>,
    /// Page index for navigation
    pub page_index: PageIndex,
    /// Total page count
    pub total_pages: usize,
    /// Active printer profile
    pub printer_profile: PrinterProfile,
    /// Whether implicit page breaks were inserted
    pub has_implicit_breaks: bool,
}
```

### ASA Strip/Restore State

```rust
/// Parallel metadata structure preserving original ASA control characters
/// when column 1 has been stripped for editing.
///
/// Addresses: Requirement 7 AC 7.2
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsaControlMap {
    /// Map from 0-based document line number to original ASA control character.
    entries: Vec<AsaControl>,
}

impl AsaControlMap {
    /// Create from a document by extracting column 1 of each line.
    pub fn from_document(lines: &[&str]) -> Self;

    /// Get the control character for a given line.
    pub fn get(&self, line: usize) -> Option<AsaControl>;

    /// Insert a new entry at line position (for line insertion during edit).
    /// Defaults to SingleSpace per Req 7.4.
    pub fn insert_line(&mut self, line: usize);

    /// Remove an entry at line position (for line deletion during edit).
    pub fn remove_line(&mut self, line: usize);

    /// Total number of entries.
    pub fn len(&self) -> usize;

    /// Whether the map is empty.
    pub fn is_empty(&self) -> bool;

    /// Restore control characters to a set of line contents.
    /// Returns lines with column 1 control characters prepended.
    pub fn restore(&self, lines: &[&str]) -> Vec<String>;
}
```

### Detection Result

```rust
/// Result of the ASA auto-detection heuristic.
///
/// Addresses: Requirement 2 AC 2.1–2.3
#[derive(Debug, Clone, PartialEq)]
pub struct DetectionResult {
    /// Whether the file is classified as ASA-controlled
    pub is_asa: bool,
    /// Confidence ratio (0.0–1.0) of valid ASA characters in column 1
    pub confidence: f64,
    /// Number of lines sampled
    pub lines_sampled: usize,
    /// Whether detection was forced by RECFM metadata
    pub forced_by_recfm: bool,
    /// Whether at least one page eject (`1`) was found
    pub has_page_eject: bool,
}

/// Configuration for the detection heuristic.
///
/// Addresses: Requirement 2 AC 2.6
#[derive(Debug, Clone, PartialEq)]
pub struct DetectionConfig {
    /// Minimum ratio of valid ASA chars for positive detection (default 0.8)
    pub threshold: f64,
    /// Number of non-blank lines to sample (default 50)
    pub sample_size: usize,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            threshold: 0.8,
            sample_size: 50,
        }
    }
}
```

### Preview Configuration

```rust
/// Complete configuration for the ASA preview subsystem.
/// Parsed from the `[asa_preview]` section of the workbench configuration.
///
/// Addresses: Requirement 12 AC 12.1
#[derive(Debug, Clone, PartialEq)]
pub struct AsaPreviewConfig {
    /// Character columns per page (default 132)
    pub page_width: usize,
    /// Print lines per page (default 60)
    pub page_depth: usize,
    /// How to handle lines exceeding page width (default Truncate)
    pub page_overflow: PageOverflow,
    /// Number of lines per shading band (default 5)
    pub band_size: usize,
    /// Whether to show alternating line shading (default true)
    pub show_line_bands: bool,
    /// Whether to run ASA auto-detection on file open (default true)
    pub auto_detect: bool,
    /// Whether to automatically strip ASA column on file open (default false)
    pub auto_strip: bool,
    /// Detection confidence threshold (default 0.8)
    pub detection_threshold: f64,
    /// Number of lines to sample for detection (default 50)
    pub detection_sample_size: usize,
    /// Named printer profile (default "ibm-1403")
    pub printer_profile: String,
    /// Text export page break style (default Dashes)
    pub export_page_separator: ExportPageSeparator,
    /// Whether to insert implicit page breaks at Page_Depth (default true)
    pub implicit_page_breaks: bool,
}

/// Style for page separators in text export.
///
/// Addresses: Requirement 11 AC 11.3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportPageSeparator {
    /// `--- PAGE N ---` separator line
    Dashes,
    /// ASCII form-feed character (0x0C)
    FormFeed,
}

impl Default for AsaPreviewConfig {
    fn default() -> Self {
        Self {
            page_width: 132,
            page_depth: 60,
            page_overflow: PageOverflow::Truncate,
            band_size: 5,
            show_line_bands: true,
            auto_detect: true,
            auto_strip: false,
            detection_threshold: 0.8,
            detection_sample_size: 50,
            printer_profile: "ibm-1403".to_string(),
            export_page_separator: ExportPageSeparator::Dashes,
            implicit_page_breaks: true,
        }
    }
}
```

### Print Preview Panel State

```rust
/// GUI-independent state for the print preview panel.
/// Drives rendering without knowledge of the UI framework.
///
/// Addresses: Requirement 6 AC 6.1–6.8
#[derive(Debug, Clone)]
pub struct PreviewPanelState {
    /// Currently displayed page number (1-based)
    pub current_page: usize,
    /// Total page count
    pub total_pages: usize,
    /// Current zoom level as a percentage (50–200, default 100)
    pub zoom_percent: u32,
    /// Whether the panel is currently visible/docked
    pub is_visible: bool,
    /// Page width in characters for layout calculation
    pub page_width: usize,
    /// Page depth in lines for layout calculation
    pub page_depth: usize,
}

impl PreviewPanelState {
    /// Navigate to a specific page. Returns false if page is out of range.
    pub fn go_to_page(&mut self, page: usize) -> bool;

    /// Navigate to the next page. Returns false if already at last page.
    pub fn next_page(&mut self) -> bool;

    /// Navigate to the previous page. Returns false if already at first page.
    pub fn previous_page(&mut self) -> bool;

    /// Navigate to the first page.
    pub fn first_page(&mut self);

    /// Navigate to the last page.
    pub fn last_page(&mut self);

    /// Set zoom level, clamped to [50, 200].
    pub fn set_zoom(&mut self, percent: u32);

    /// Zoom to fit page width in the available viewport.
    pub fn fit_width(&mut self, viewport_width: f32, font_char_width: f32);

    /// Zoom to fit entire page in the available viewport.
    pub fn fit_page(&mut self, viewport_width: f32, viewport_height: f32,
                    font_char_width: f32, font_line_height: f32);
}
```

### Export Types

```rust
/// Options for text export.
///
/// Addresses: Requirement 11 AC 11.1, 11.3–11.5
#[derive(Debug, Clone)]
pub struct TextExportOptions {
    /// How to represent page breaks in the output
    pub page_separator: ExportPageSeparator,
    /// Output file path
    pub path: std::path::PathBuf,
}

/// Options for PDF export.
///
/// Addresses: Requirement 11 AC 11.2, 11.6
#[derive(Debug, Clone)]
pub struct PdfExportOptions {
    /// Output file path
    pub path: std::path::PathBuf,
    /// Whether to include line band shading in PDF
    pub include_shading: bool,
    /// Monospace font size in points (default 10)
    pub font_size: f32,
}

/// Result of a successful export operation.
///
/// Addresses: Requirement 11 AC 11.7
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// Output file path
    pub path: std::path::PathBuf,
    /// Total pages exported
    pub pages_exported: usize,
    /// Total bytes written
    pub bytes_written: u64,
}
```

---

## Public API Surface

### ASA Parser

```rust
/// Parse a single line's column 1 character into an ASA control.
/// If the character is not recognised, returns AsaControl::SingleSpace
/// (the fallback) and the caller should log a WARN diagnostic.
///
/// Addresses: Requirement 1 AC 1.1, 1.9
pub fn parse_control_char(ch: char) -> (AsaControl, bool);

/// Parse a full document (slice of line strings) into ParsedLines.
/// Each line's first character is interpreted as the ASA control;
/// the remainder is the data content.
///
/// Addresses: Requirement 1 AC 1.1–1.9
pub fn parse_document(lines: &[&str]) -> Vec<ParsedLine>;

/// Parse a full document using an AsaControlMap (for stripped files).
/// Control characters are taken from the map rather than column 1.
///
/// Addresses: Requirement 7 AC 7.9
pub fn parse_document_stripped(
    lines: &[&str],
    control_map: &AsaControlMap,
) -> Vec<ParsedLine>;
```

### ASA Detector

```rust
/// Run the ASA auto-detection heuristic on a set of lines.
/// Examines column 1 of the first N non-blank lines.
///
/// Addresses: Requirement 2 AC 2.1–2.2
pub fn detect_asa(lines: &[&str], config: &DetectionConfig) -> DetectionResult;

/// Check if a file should be treated as ASA based on RECFM metadata.
/// Returns true for "FBA" or "VBA" record formats.
///
/// Addresses: Requirement 2 AC 2.3
pub fn is_asa_by_recfm(recfm: &str) -> bool;
```

### Paginator

```rust
/// Build a complete PreviewState from parsed lines.
/// Interprets spacing, page breaks, overstrike merging, and shading.
///
/// Addresses: Requirements 1, 4, 5, 8, 9
pub fn paginate(
    parsed_lines: &[ParsedLine],
    config: &AsaPreviewConfig,
) -> PreviewState;

/// Re-paginate a subset of the document (incremental update).
/// Used when the edit buffer changes to avoid full re-parse.
///
/// Addresses: Requirement 6 AC 6.6
pub fn paginate_range(
    parsed_lines: &[ParsedLine],
    config: &AsaPreviewConfig,
    start_line: usize,
    end_line: usize,
    existing_state: &PreviewState,
) -> PreviewState;
```

### Overstrike Merger

```rust
/// Merge a sequence of parsed lines, combining overprint lines with
/// their base lines. Returns MergedLines with styling applied.
///
/// Addresses: Requirement 5 AC 5.1–5.5
pub fn merge_overstrikes(parsed_lines: &[ParsedLine]) -> Vec<MergeResult>;

/// Result of merging a group of lines (base + overprints).
#[derive(Debug, Clone, PartialEq)]
pub enum MergeResult {
    /// A data line (possibly merged with overprints)
    Merged(MergedLine),
    /// An overprint with no preceding base line (diagnostic case)
    OrphanOverprint {
        content: String,
        source_line: usize,
    },
}
```

### ASA Strip/Restore

```rust
/// Strip ASA control characters from column 1 of all lines.
/// Returns the modified lines and the control map for restoration.
///
/// Addresses: Requirement 7 AC 7.1, 7.2
pub fn strip_asa(lines: &[&str]) -> (Vec<String>, AsaControlMap);

/// Restore ASA control characters to column 1 using the control map.
/// Returns lines with the original control characters prepended.
///
/// Addresses: Requirement 7 AC 7.3
pub fn restore_asa(lines: &[&str], control_map: &AsaControlMap) -> Vec<String>;
```

### Page Navigation

```rust
/// Navigate to a specific page in the preview.
/// Returns the element index in PreviewState::elements for that page.
///
/// Addresses: Requirement 10 AC 10.1
pub fn locate_page(
    state: &PreviewState,
    page_number: usize,
) -> Result<usize, AsaPreviewError>;

/// Find which page a given element index belongs to.
///
/// Addresses: Requirement 10 AC 10.3
pub fn current_page_for_element(
    state: &PreviewState,
    element_index: usize,
) -> usize;
```

### Line Band Shading

```rust
/// Compute band group assignments for a page of lines.
/// Returns alternating 0/1 group numbers for each line position,
/// resetting at each page boundary.
///
/// Addresses: Requirement 9 AC 9.1–9.5
pub fn compute_band_groups(
    elements: &[PreviewElement],
    band_size: usize,
) -> Vec<u8>;
```

### Export

```rust
/// Export the preview as plain text to the specified path.
///
/// Addresses: Requirement 11 AC 11.1, 11.3–11.5
pub fn export_text(
    state: &PreviewState,
    options: &TextExportOptions,
) -> Result<ExportResult, AsaPreviewError>;

/// Export the preview as PDF to the specified path.
///
/// Addresses: Requirement 11 AC 11.2, 11.6
pub fn export_pdf(
    state: &PreviewState,
    options: &PdfExportOptions,
) -> Result<ExportResult, AsaPreviewError>;
```

### Viewer Registration

```rust
/// Register the ASA report preview viewer with the custom-file-viewers framework.
/// This is called during crate initialisation.
///
/// Addresses: Requirement 3 AC 3.5
pub fn register_viewer(registry: &mut ViewerRegistry, command_registry: &mut CommandRegistry);

/// The viewer key used for registration and command dispatch.
pub const VIEWER_KEY: &str = "asa-report";
```

---

## Error Handling

```rust
/// Errors originating from the ff-asa-report-preview crate.
/// Formatted per Error Message Standards: `[asa-preview] operation: description`
///
/// Addresses: Cross-cutting error handling
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AsaPreviewError {
    /// Page number is out of valid range.
    #[error("[asa-preview] navigate: page {page} not found — report has {total} pages")]
    PageNotFound {
        page: usize,
        total: usize,
    },

    /// Export I/O failure.
    #[error("[asa-preview] export: I/O error writing to {path}: {source}")]
    ExportIo {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Export path is invalid or inaccessible.
    #[error("[asa-preview] export: invalid path {path}: {reason}")]
    InvalidExportPath {
        path: String,
        reason: String,
    },

    /// Configuration value is invalid.
    #[error("[asa-preview] config: invalid value for {key}: {reason}")]
    InvalidConfig {
        key: String,
        reason: String,
    },

    /// Preview cannot be activated (no document loaded).
    #[error("[asa-preview] activate: no document available for preview")]
    NoDocument,

    /// Strip/restore operation failed due to inconsistent state.
    #[error("[asa-preview] {operation}: control map has {map_size} entries but document has {doc_lines} lines")]
    ControlMapMismatch {
        operation: String,
        map_size: usize,
        doc_lines: usize,
    },

    /// Export cancelled by operator.
    #[error("[asa-preview] export: cancelled after {pages_exported} pages")]
    ExportCancelled {
        pages_exported: usize,
    },

    /// PDF rendering error.
    #[error("[asa-preview] export pdf: rendering failed: {reason}")]
    PdfRenderError {
        reason: String,
    },
}
```

---

## Integration Points

### With `ff-document-model` (Core Feature — upstream)

- **Dependency direction**: ff-asa-report-preview depends on ff-document-model
- **API consumed**: Line content access (`Document::line_count()`, `Document::get_range()`), line start/end positions, watcher registration for edit notifications
- **Usage pattern**: Parser reads all lines from the document buffer to extract ASA controls and content. Strip/restore modifies the edit buffer through insert/delete operations.
- **Watcher integration**: Registers a `DocumentWatcher` to receive edit notifications and trigger incremental preview re-rendering

### With `ff-command` (Command Framework — upstream)

- **Dependency direction**: ff-asa-report-preview depends on ff-command
- **API consumed**: `CommandRegistry::register()` for all ASA commands
- **Commands registered**:
  - `PREVIEW asa-report` / `PREVIEW ON` / `PREVIEW` — activate preview
  - `PREVIEW PANEL` — open print preview panel
  - `PREVIEW EXPORT TEXT <path>` — text export
  - `PREVIEW EXPORT PDF <path>` — PDF export
  - `PREVIEW SET PRINTER <profile>` — switch printer profile
  - `LOCATE PAGE <n>` — page navigation
  - `LOCATE PAGE FIRST` / `LOCATE PAGE LAST` — first/last page
  - `ASA STRIP` — strip control characters
  - `ASA RESTORE` — restore control characters

### With `ff-layout` (Layout and Docking — upstream)

- **Dependency direction**: ff-asa-report-preview depends on ff-layout
- **API consumed**: Panel registration and docking for the Print_Preview_Panel
- **Usage pattern**: Preview panel is registered as a dockable panel; layout system manages its position and visibility

### With `ff-config` (Configuration System — upstream)

- **Dependency direction**: ff-asa-report-preview depends on ff-config
- **API consumed**: Configuration reading from `[asa_preview]` TOML section, hot-reload notification
- **Usage pattern**: Reads `AsaPreviewConfig` values on activation and subscribes to hot-reload events to re-render with updated settings

### With `ff-theme` (Theme and Appearance — upstream)

- **Dependency direction**: ff-asa-report-preview depends on ff-theme
- **API consumed**: Colour token resolution for Page_Band backgrounds, line band tint, halt band, bold/underline font styles
- **Tokens consumed**: `asa.page_band_odd`, `asa.page_band_even`, `asa.page_band_text`, `asa.line_band_tint`, `asa.halt_band`, `asa.halt_band_text`

### With `ff-custom-viewers` (Custom File Viewers — peer)

- **Dependency direction**: ff-asa-report-preview depends on ff-custom-viewers
- **API consumed**: `ViewerRegistry::register()` for Viewer_Key `"asa-report"`, `CustomViewer` trait implementation, PREVIEW command dispatch routing
- **Usage pattern**: At crate init, registers the viewer. When PREVIEW is activated, the custom-viewers framework routes to this crate's viewer implementation. Split view and coexistence with the editor are managed by the custom-viewers framework.

### With `ff-fileforge` (FileForge Integration — peer)

- **Dependency direction**: ff-asa-report-preview depends on ff-fileforge
- **API consumed**: RECFM metadata access for unconditional ASA detection (FBA/VBA), flat-file mode detection hooks
- **Usage pattern**: When a file is opened, the detector queries ff-fileforge for RECFM metadata. If RECFM is "FBA" or "VBA", heuristic detection is bypassed and ASA mode is confirmed immediately.

### With `ff-file-ops` (File Operations — downstream consumer)

- **Dependency direction**: ff-file-ops may consume ff-asa-report-preview
- **Integration**: During save, ff-file-ops calls `restore_asa()` to re-insert column 1 control characters if ASA_Strip mode is active

### Dependency Direction Summary

```
ff-document-model ← ff-asa-report-preview → ff-custom-viewers
ff-command        ← ff-asa-report-preview → ff-fileforge
ff-layout         ← ff-asa-report-preview
ff-config         ← ff-asa-report-preview
ff-theme          ← ff-asa-report-preview

ff-asa-report-preview ← ff-file-ops (save-time ASA restore)
```

---

## Configuration

ff-asa-report-preview owns the `[asa_preview]` namespace in the workbench TOML configuration file.

### TOML Schema

```toml
[asa_preview]
# Character columns per page. Range: 60–255. Default: 132
page_width = 132

# Print lines per page. Range: 10–120. Default: 60
page_depth = 60

# Handling of lines exceeding page width: "truncate" or "wrap". Default: "truncate"
page_overflow = "truncate"

# Number of lines per shading band. Range: 1–20. Default: 5
band_size = 5

# Whether to show alternating line band shading. Default: true
show_line_bands = true

# Whether to run ASA auto-detection on file open. Default: true
auto_detect = true

# Whether to automatically strip ASA column on file open. Default: false
auto_strip = false

# Minimum ratio for ASA detection confidence. Range: 0.5–1.0. Default: 0.8
detection_threshold = 0.8

# Number of non-blank lines to sample for detection. Range: 10–500. Default: 50
detection_sample_size = 50

# Named printer profile: "ibm-1403", "ibm-3800", "ibm-4245", or "custom". Default: "ibm-1403"
printer_profile = "ibm-1403"

# Text export page break style: "dashes" or "formfeed". Default: "dashes"
export_page_separator = "dashes"

# Whether to insert implicit page breaks at page_depth intervals. Default: true
implicit_page_breaks = true
```

### Config Resolution Rules

| Setting | Absent | Invalid Value | Out of Range |
|---------|--------|---------------|--------------|
| `page_width` | Default to 132 | Default to 132 + WARN | Clamp to [60–255] + WARN |
| `page_depth` | Default to 60 | Default to 60 + WARN | Clamp to [10–120] + WARN |
| `page_overflow` | Default to "truncate" | Default to "truncate" + WARN | N/A |
| `band_size` | Default to 5 | Default to 5 + WARN | Clamp to [1–20] + WARN |
| `show_line_bands` | Default to true | Default to true + WARN | N/A |
| `auto_detect` | Default to true | Default to true + WARN | N/A |
| `auto_strip` | Default to false | Default to false + WARN | N/A |
| `detection_threshold` | Default to 0.8 | Default to 0.8 + WARN | Clamp to [0.5–1.0] + WARN |
| `detection_sample_size` | Default to 50 | Default to 50 + WARN | Clamp to [10–500] + WARN |
| `printer_profile` | Default to "ibm-1403" | Default to "ibm-1403" + WARN | N/A |
| `export_page_separator` | Default to "dashes" | Default to "dashes" + WARN | N/A |
| `implicit_page_breaks` | Default to true | Default to true + WARN | N/A |

---

## Design Decisions

### Decision 1: GUI-Independent Core with Rendering Hints

**Chosen**: All parsing, pagination, merging, and export logic is GUI-independent. The core produces a `PreviewState` containing `PreviewElement` items that the UI layer interprets for rendering.

Rationale:
1. **Testability**: Core logic can be tested without a GUI framework
2. **Portability**: Supports future UI frameworks beyond egui
3. **Separation of concerns**: Business logic (ASA interpretation) is cleanly separated from presentation
4. **Property-based testing**: Pure functions operating on data structures are ideal for proptest

### Decision 2: Eager Full-Document Parse on Activation

**Chosen**: When PREVIEW is activated, the entire document is parsed to build the complete `PreviewState` and `PageIndex`.

Rationale:
1. **Navigation performance**: O(1) page lookups require a pre-built index
2. **Page count accuracy**: Total page count is needed for status bar and panel display
3. **Simplicity**: Avoids complex incremental parsing state machines for initial implementation
4. **Acceptable cost**: Even large reports (100K+ lines) parse in milliseconds since ASA parsing is character-level extraction

Trade-off: Memory usage for the preview element list. For a 100K-line document this is approximately 10–50 MB — acceptable for workbench use.

### Decision 3: MergedLine as Separate Type (Not In-Place Mutation)

**Chosen**: Overstrike merging produces a new `MergedLine` type rather than modifying the source document lines.

Rationale:
1. **Read-only display**: Preview does not modify the edit buffer (design constraint)
2. **Rich styling**: `StyledChar` carries per-character bold/underline attributes that don't exist in the source
3. **Export compatibility**: Plain-text export can call `MergedLine::plain_text()` while PDF export uses the styled characters
4. **Reversibility**: Original document is untouched; closing preview discards all MergedLine data

### Decision 4: AsaControlMap as Parallel Vector

**Chosen**: The strip/restore metadata is a `Vec<AsaControl>` indexed by line number, rather than a `HashMap` or BTreeMap.

Rationale:
1. **O(1) lookup**: Line-number-indexed access is constant time
2. **Memory efficiency**: Enum variants are 1 byte each; 100K lines = 100 KB
3. **Line insert/delete**: Vec `insert()` and `remove()` operations are O(n) but document edits are infrequent relative to reads
4. **Simplicity**: No key hashing, no tree balancing — just a parallel array

---

## Correctness Properties

The following properties are suitable for property-based testing with the `proptest` crate. Each property is universal — it must hold for all valid inputs.

### Property 1: ASA Control Character Parse Round-Trip

**Statement:** For any valid ASA control character, parsing it with `AsaControl::from_char` and then converting back produces the original character.

```
∀ ch ∈ {' ', '0', '-', '1', '+', 'H'}:
    AsaControl::from_char(ch).is_some()
    ∧ to_char(AsaControl::from_char(ch).unwrap()) == ch
```

**Validates: Requirement 1.1**

### Property 2: Spacing Lines Count Invariant

**Statement:** For any document with N lines where M lines have control character `0` (double space) and K lines have control character `-` (triple space), the total number of SpacingLine elements in the preview equals M + 2K.

```
∀ document D with parsed lines P:
    let M = P.iter().filter(|l| l.control == AsaControl::DoubleSpace).count();
    let K = P.iter().filter(|l| l.control == AsaControl::TripleSpace).count();
    let spacing_count = paginate(P, config).elements.iter()
        .filter(|e| matches!(e, PreviewElement::SpacingLine { .. })).count();
    spacing_count == M + 2 * K
```

**Validates: Requirements 1.2, 1.3, 1.4**

### Property 3: Page Break Count Equals Page Index Size

**Statement:** The number of PageBand elements in the preview output equals the page count in the PageIndex (both explicit and implicit breaks).

```
∀ document D, ∀ config C:
    let state = paginate(D, C);
    let band_count = state.elements.iter()
        .filter(|e| matches!(e, PreviewElement::PageBand { .. })).count();
    band_count == state.page_index.page_count()
```

**Validates: Requirements 4.1, 4.5, 4.6**

### Property 4: Overstrike Lines Never Appear as Separate Rows

**Statement:** For any document with overprint lines (`+` control), no PreviewElement::DataLine has a source_line that was an overprint line — all overprint lines are absorbed into their preceding base line's MergedLine.

```
∀ document D:
    let parsed = parse_document(D);
    let overprint_lines: HashSet<usize> = parsed.iter()
        .filter(|l| l.control == AsaControl::Overstrike)
        .map(|l| l.source_line)
        .collect();
    let state = paginate(parsed, config);
    ∀ element ∈ state.elements:
        if let PreviewElement::DataLine { content, .. } = element {
            ¬overprint_lines.contains(&content.source_line)
        }
```

**Validates: Requirements 5.1, 5.4**

### Property 5: Overstrike Merge Preserves Base Line Length (Minimum)

**Statement:** A MergedLine's character count is at least as long as the base line — overprint can extend but never shrink the merged result.

```
∀ base_line B, ∀ overprint_lines [O₁, ..., Oₙ]:
    let merged = MergedLine::from_base(B);
    for O in overprint_lines { merged.apply_overprint(O); }
    merged.characters.len() >= B.chars().count()
```

**Validates: Requirement 5.2**

### Property 6: Strip-Restore Round-Trip

**Statement:** For any document with valid ASA control characters, stripping and then restoring produces output identical to the original.

```
∀ document D where all lines have a valid ASA char in column 1:
    let (stripped, map) = strip_asa(D);
    let restored = restore_asa(&stripped, &map);
    restored == D
```

**Validates: Requirements 7.1, 7.2, 7.3**

### Property 7: AsaControlMap Length Invariant After Edits

**Statement:** After inserting N lines and deleting M lines from a stripped document, the AsaControlMap length equals the original length plus N minus M.

```
∀ control_map C, ∀ inserts [i₁, ..., iₙ], ∀ deletes [d₁, ..., dₘ]:
    apply_inserts(&mut C, inserts);
    apply_deletes(&mut C, deletes);
    C.len() == original_len + N - M
```

**Validates: Requirements 7.4, 7.5**

### Property 8: Detection Confidence Is a Valid Ratio

**Statement:** For any set of input lines, the detection confidence is always in the range [0.0, 1.0].

```
∀ lines L, ∀ config C:
    let result = detect_asa(L, C);
    result.confidence >= 0.0 ∧ result.confidence <= 1.0
```

**Validates: Requirement 2.1**

### Property 9: Detection Requires Page Eject for Positive Classification

**Statement:** If no `1` character is present in the sampled lines, detection never classifies the file as ASA (unless forced by RECFM).

```
∀ lines L where no line starts with '1', ∀ config C:
    let result = detect_asa(L, C);
    result.is_asa == false ∨ result.forced_by_recfm == true
```

**Validates: Requirement 2.2**

### Property 10: Page Index Navigation Round-Trip

**Statement:** For any valid page number P, looking up the document line for page P and then finding which page that line belongs to returns P.

```
∀ state S, ∀ P ∈ [1, S.page_index.page_count()]:
    let line = S.page_index.document_line_for_page(P).unwrap();
    S.page_index.page_for_document_line(line) == P
```

**Validates: Requirements 10.1, 10.2**

### Property 11: Line Band Shading Resets at Page Boundaries

**Statement:** In the computed band groups, the first data line after every PageBand element always starts in band group 0 (the first shading group).

```
∀ elements E, ∀ band_size B:
    let groups = compute_band_groups(E, B);
    for each PageBand at index i in E:
        let next_data = first DataLine or SpacingLine after index i;
        groups[next_data] == 0
```

**Validates: Requirement 9.3, 9.5**

### Property 12: Implicit Page Breaks Respect Page Depth

**Statement:** When implicit page breaks are enabled and no explicit `1` controls exist, pages never exceed the configured page_depth in data lines.

```
∀ document D with no '1' controls, ∀ config C where C.implicit_page_breaks == true:
    let state = paginate(D, C);
    for each page in state:
        data_lines_in_page <= C.page_depth
```

**Validates: Requirements 8.3, 8.4**

### Property 13: Export Text Preserves Page Count

**Statement:** A text export contains exactly as many page separators as there are page breaks in the preview state (total_pages - 1 separators for total_pages pages).

```
∀ state S:
    let export = export_text(S, options);
    count_separators(export) == S.total_pages - 1
```

**Validates: Requirement 11.3**

### Property 14: Preview Elements Never Reference Invalid Source Lines

**Statement:** Every DataLine element's source_line field is within the valid range [0, document_line_count).

```
∀ state S produced from a document with N lines:
    ∀ element ∈ S.elements:
        if let PreviewElement::DataLine { content, .. } = element {
            content.source_line < N
        }
```

**Validates: Requirements 1.1–1.9 (structural integrity)**

---

## Testing Strategy

### Unit Tests

- `parser_tests.rs`: All 6 control characters parsed correctly, unrecognised character fallback, empty lines, whitespace-only lines
- `detector_tests.rs`: 100% ASA files detected, mixed files at threshold boundary, no-page-eject rejection, RECFM bypass, configurable thresholds
- `paginator_tests.rs`: Spacing insertion counts, implicit page breaks at page depth, explicit page breaks, mixed explicit/implicit, pre-page-1 content
- `merger_tests.rs`: Single overprint bold, underscore overprint, multiple consecutive overprints, orphan overprint diagnostic, space-in-overprint passthrough
- `page_index_tests.rs`: Lookup forward/reverse, out-of-range page, empty document, single-page document
- `strip_tests.rs`: Strip extracts correct characters, restore rebuilds exact original, insert/delete line updates map, stripped preview uses map
- `shading_tests.rs`: Band group alternation, reset at page boundary, configurable band size, spacing lines count in band
- `export_tests.rs`: Text export page separators, PDF export page count, spacing lines as blank lines, bold/underline stripped in text export

### Property-Based Tests (proptest)

- ASA control character parse round-trip (Property 1)
- Spacing lines count invariant (Property 2)
- Page break count equals page index size (Property 3)
- Overstrike lines never appear as separate rows (Property 4)
- Overstrike merge preserves base line length (Property 5)
- Strip-restore round-trip (Property 6)
- AsaControlMap length invariant after edits (Property 7)
- Detection confidence valid ratio (Property 8)
- Detection requires page eject for positive (Property 9)
- Page index navigation round-trip (Property 10)
- Line band shading resets at page boundaries (Property 11)
- Implicit page breaks respect page depth (Property 12)
- Export text preserves page count (Property 13)
- Preview elements never reference invalid source lines (Property 14)

### Integration Tests

- End-to-end: load sample ASA file → detect → activate preview → verify page count and element sequence
- Strip-edit-restore: strip → insert new lines → delete lines → save → verify original structure preserved
- Export round-trip: activate preview → export text → verify page separators and content match
- Configuration hot-reload: change band_size → verify preview re-renders with new shading groups
- Large file: generate 10K-line ASA document → verify pagination completes in < 100ms

### Test Infrastructure

- **Sample fixtures**: Pre-built ASA files with known page counts, overprint patterns, and spacing sequences in `tests/fixtures/`
- **Testing framework**: `proptest` for property-based tests, standard `#[test]` for unit tests
- **Minimum proptest iterations**: 100 per property
- **Strategy generators**: Custom proptest strategies for generating valid ASA documents with controlled distributions of control characters
