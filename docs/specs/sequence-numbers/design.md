# Design Document: Sequence Numbers (`ff-seqnum`)

## Overview

The `ff-seqnum` crate implements the **sequence number detection, stripping, numbering, and display** subsystem for FileForgeWorkbench. It handles the legacy punched-card-era sequence numbers found in mainframe source files (COBOL, JCL, FORTRAN, PL/I) where fixed column ranges carry sequence data that is not part of the source logic. The crate is **GUI-independent** — all detection, stripping, numbering, and state management operate on the document model without GUI framework dependency.

### Purpose

- Detect sequence numbers in fixed-format files using configurable heuristic sampling
- Automatically strip detected sequence numbers on file open (Auto-Unnum)
- Provide explicit `UNNUM` command to remove sequence numbers from any file or scoped range
- Provide explicit `NUMBER` command to write sequential numbers into defined column positions
- Provide `NUMBER SHOW` display overlay mode for inspecting original sequence numbers
- Preserve/restore sequence numbers on save based on configuration
- Support COBOL, JCL, FORTRAN, PL/I, and assembler format detection
- Support configurable detection thresholds, sample sizes, and format options

### Position in Architecture

```
Wave 11 — Display Modes

┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
│   Status bar indicators, column shading, overlay render      │
├─────────────────────────────────────────────────────────────┤
│         ff-seqnum (THIS CRATE — Wave 11)                     │
│   Detection, stripping, numbering, NUMBER SHOW state         │
├─────────────────────────────────────────────────────────────┤
│  ff-document-model (Wave 4) — edit buffer read/write         │
│  ff-language-service (Wave 7) — language profile columns     │
│  ff-command (Wave 2) — command registration + dispatch       │
│  ff-undo (Wave 4) — Sequence_Transaction recording           │
│  ff-config (Wave 2) — detection/save/display settings        │
│  ff-file-ops (Wave 8) — save pipeline integration            │
│  ff-line-commands (Wave 5) — CC block range for scoping      │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging (Wave 0)            │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **GUI Independence (FFW-ARCH-001)**: All detection, stripping, numbering, and state tracking operate on the document model. Visual indicators (status bar, column shading, overlay) are data models consumed by the shell.
- **Command-Driven**: UNNUM (`sequence.unnum`), NUMBER (`sequence.number`), and NUMBER SHOW (`sequence.number_show`) are registered with the command framework.
- **Multi-Crate Workspace**: Crate at `crates/ff-seqnum`
- **Error Message Standards**: All errors follow `[seqnum] operation: description` format
- **Non-Blocking**: Detection sampling is non-blocking; defers to background step for slow file access
- **BOUNDS-aware**: Sequence column operations never alter active BOUNDS settings

### Upstream Dependencies

| Crate | Purpose |
|-------|---------|
| `ff-document-model` | Edit buffer access — reading line content for detection, mutating lines for strip/number operations |
| `ff-language-service` | Language profile registry — provides `sequence_cols_front`, `sequence_cols_back`, `auto_unnum` per language |
| `ff-command` | Command registration for UNNUM, NUMBER, NUMBER SHOW; dispatch integration |
| `ff-undo` | Transaction API for recording Sequence_Transactions (UNNUM and NUMBER are undoable) |
| `ff-config` | Configuration namespace `editor.sequence_numbers.*` — thresholds, formats, save behaviour |
| `ff-line-commands` | CC block range resolution for scoped UNNUM/NUMBER operations |
| `ff-logging` | WARN-level diagnostics for invalid column ranges, config clamping, overflow warnings |

### Downstream Consumers

| Consumer | Integration |
|----------|-------------|
| `ff-file-ops` | Save pipeline hook — restore/strip sequence numbers based on `restore_on_save` setting |
| `ff-viewport` | NUMBER SHOW overlay rendering — reads side-table data for display |
| `ff-edit-operations` | BOUNDS interaction — queries active sequence column state for constraint semantics |
| `ff-desktop` (GUI shell) | Status bar indicators (`SEQNUM`, `SEQNUM?`, `SEQSHOW`), column shading |

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph "Trigger Events"
        FO[File Open<br/>auto-detect + auto-strip]
        UC[UNNUM Command<br/>explicit strip]
        NC[NUMBER Command<br/>explicit sequencing]
        NS[NUMBER SHOW<br/>overlay toggle]
        SV[SAVE Event<br/>restore/strip hook]
    end

    subgraph "ff-seqnum"
        SD[SequenceDetector<br/>heuristic sampling]
        SS[SequenceStripper<br/>column clearing + side-table]
        SN[SequenceNumberer<br/>number generation + insertion]
        SM[SeqNumStateManager<br/>per-document state tracking]
        SO[SeqNumOverlay<br/>NUMBER SHOW data model]
        SC[SeqNumConfig<br/>typed config access]
        CF[ColumnFormat<br/>numeric/alpha format logic]
        SH[SaveHook<br/>restore_on_save logic]
    end

    subgraph "Upstream Crates"
        DM[ff-document-model<br/>TextBuffer / line access]
        LS[ff-language-service<br/>LanguageProfile]
        CMD[ff-command<br/>CommandRegistry]
        UNDO[ff-undo<br/>Transaction API]
        CFG[ff-config<br/>editor.sequence_numbers.*]
        LC[ff-line-commands<br/>CC block range]
    end

    FO --> SD
    SD --> DM
    SD --> LS
    SD --> SC
    SC --> CFG
    SD --> SS
    SS --> DM
    SS --> SM
    SS --> UNDO

    UC --> CMD
    CMD --> SS
    SS --> LC

    NC --> CMD
    CMD --> SN
    SN --> DM
    SN --> CF
    SN --> UNDO
    SN --> LC

    NS --> CMD
    CMD --> SO
    SO --> SM

    SV --> SH
    SH --> SM
    SH --> DM
end
```

### Layer Placement

| Layer | Components | Role |
|-------|-----------|------|
| **Detection Layer** | `SequenceDetector`, `SeqNumConfig` | Samples file content and determines presence of sequence numbers using heuristic rules |
| **Strip Layer** | `SequenceStripper`, `SeqNumStateManager` | Clears sequence columns, stores originals in side-table, tracks per-document state |
| **Numbering Layer** | `SequenceNumberer`, `ColumnFormat` | Generates and inserts sequential numbers in configured formats |
| **Display Layer** | `SeqNumOverlay` | Provides NUMBER SHOW data model for viewport overlay rendering |
| **Save Layer** | `SaveHook` | Intercepts save pipeline to restore or strip sequence numbers based on config |
| **Command Layer** | Command handlers | Bridges command framework dispatch to detection/strip/number operations |

### Data Flow (File Open Lifecycle)

```
1. File opened in Standard_Text_Mode
2. Language service resolves active LanguageProfile for the file
3. SeqNumConfig loads detection settings (threshold, sample_size)
4. SequenceDetector samples up to N non-blank lines from document model
5. For each defined column range (front, back): evaluate numeric pattern presence
6. DetectionResult produced: { front_detected, back_detected, alphanumeric_prefix }
7. IF auto_unnum=true AND sequence numbers detected:
   a. SequenceStripper reads original column content → stores in side-table
   b. SequenceStripper replaces column bytes with spaces in edit buffer
   c. SeqNumStateManager records stripped state for this document
   d. Status message emitted: "SEQUENCE NUMBERS REMOVED: COLS x-y[, x-y]"
8. IF auto_unnum=false AND detected:
   a. Status message emitted: "SEQUENCE NUMBERS DETECTED — not removed"
   b. SeqNumStateManager records detected-but-not-stripped state
9. Shell reads SeqNumStateManager for status bar indicator (SEQNUM / SEQNUM?)
```

---

## Module Structure

```
crates/ff-seqnum/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API re-exports, crate docs
│   ├── detector.rs             # SequenceDetector — heuristic sampling logic
│   ├── stripper.rs             # SequenceStripper — column clearing + side-table
│   ├── numberer.rs             # SequenceNumberer — number generation + insertion
│   ├── state.rs                # SeqNumStateManager — per-document state tracking
│   ├── overlay.rs              # SeqNumOverlay — NUMBER SHOW data model
│   ├── columns.rs              # SeqNumColumns — column range parsing + validation
│   ├── format.rs               # SeqNumFormat — numeric/alpha format logic
│   ├── config.rs               # SeqNumConfig — typed config access
│   ├── save_hook.rs            # SaveHook — restore/strip on save logic
│   ├── commands/
│   │   ├── mod.rs              # Re-exports for command handlers
│   │   ├── unnum.rs            # UNNUM command handler (sequence.unnum)
│   │   ├── number.rs           # NUMBER command handler (sequence.number)
│   │   └── number_show.rs      # NUMBER SHOW command handler (sequence.number_show)
│   └── error.rs                # SeqNumError enum
└── tests/
    ├── detector_tests.rs       # Detection heuristic property tests
    ├── stripper_tests.rs       # Strip + restore correctness tests
    ├── numberer_tests.rs       # Number generation property tests
    ├── columns_tests.rs        # Column range parsing property tests
    ├── format_tests.rs         # Format generation property tests
    ├── config_tests.rs         # Configuration validation tests
    ├── save_hook_tests.rs      # Save pipeline integration tests
    └── undo_tests.rs           # Undo/redo round-trip tests
```

---

## Data Models

### SeqNumColumns

```rust
/// Represents a validated column range for sequence numbers.
/// Column numbers are 1-based, matching ISPF conventions.
/// Addresses: Requirements 1.1, 1.2, 1.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqNumColumns {
    /// The starting column (1-based, inclusive).
    start: u16,
    /// The ending column (1-based, inclusive).
    end: u16,
}

impl SeqNumColumns {
    /// Parse a column range from a "start-end" string (e.g., "1-6", "73-80").
    /// Returns None if the format is invalid (non-numeric, start > end, or zero values).
    /// Addresses: Requirement 1.4
    pub fn parse(s: &str) -> Option<Self>;

    /// Create a column range from explicit start and end values.
    /// Returns None if start > end or either value is zero.
    pub fn new(start: u16, end: u16) -> Option<Self>;

    /// Returns the starting column (1-based).
    pub fn start(&self) -> u16;

    /// Returns the ending column (1-based).
    pub fn end(&self) -> u16;

    /// Returns the width of the column range (end - start + 1).
    pub fn width(&self) -> u16;

    /// Returns the 0-based byte offset for the start of this range within a line.
    pub fn start_offset(&self) -> usize;

    /// Returns the 0-based byte offset for the end of this range within a line (exclusive).
    pub fn end_offset(&self) -> usize;
}
```

### SeqNumFormat

```rust
/// The format specification for generated sequence numbers.
/// Addresses: Requirements 7.1, 7.2, 7.3
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeqNumFormat {
    /// Pure numeric format: zero-padded decimal filling the entire column width.
    /// Example: "000100" for value 100 in a 6-column range.
    Numeric,
    /// Alphanumeric prefix format: fixed alphabetic prefix followed by zero-padded digits.
    /// Example: "ABC001" for prefix "ABC", value 1, in a 6-column range.
    AlphaPrefix {
        /// The alphabetic prefix string (uppercase).
        prefix: String,
    },
}

impl SeqNumFormat {
    /// Format a sequence value into a string of the specified width.
    /// Returns None if the value overflows the available digit positions.
    /// Addresses: Requirements 6.11, 7.1, 7.2
    pub fn format_value(&self, value: u64, column_width: u16) -> Option<String>;

    /// Returns the number of digit positions available for the given column width.
    pub fn digit_width(&self, column_width: u16) -> u16;

    /// Returns the maximum sequence value representable in the given column width.
    pub fn max_value(&self, column_width: u16) -> u64;

    /// Validates that this format can produce at least one digit in the given width.
    /// Addresses: Requirement 7.4
    pub fn validate_for_width(&self, column_width: u16) -> bool;
}
```

### DetectionResult

```rust
/// The outcome of sequence number detection for a single document.
/// Produced by SequenceDetector after sampling file content.
/// Addresses: Requirements 2.1–2.9
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectionResult {
    /// Whether sequence numbers were detected in the front column range.
    pub front_detected: bool,
    /// Whether sequence numbers were detected in the back column range.
    pub back_detected: bool,
    /// The front column range that was checked (from language profile or config override).
    pub front_columns: Option<SeqNumColumns>,
    /// The back column range that was checked (from language profile or config override).
    pub back_columns: Option<SeqNumColumns>,
    /// If detected, the format classification for front columns.
    pub front_format: Option<DetectedFormat>,
    /// If detected, the format classification for back columns.
    pub back_format: Option<DetectedFormat>,
    /// Number of non-blank lines sampled.
    pub lines_sampled: usize,
    /// Number of lines matching the numeric criterion (front).
    pub front_match_count: usize,
    /// Number of lines matching the numeric criterion (back).
    pub back_match_count: usize,
}

/// The format classification detected during sampling.
/// Addresses: Requirement 2.9
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedFormat {
    /// Pure numeric sequence (all digits or spaces with at least one all-digit line).
    Numeric,
    /// Alphanumeric sequence with a consistent prefix.
    AlphaPrefix {
        /// The detected alphabetic prefix.
        prefix: String,
    },
}
```

### SequenceNumberState

```rust
/// Per-document state tracking for sequence number processing.
/// Stored by SeqNumStateManager, keyed by document ID.
/// Addresses: Requirements 3.9, 4.1, 4.2, 8.1
#[derive(Debug, Clone)]
pub struct SequenceNumberState {
    /// The detection result from file open (or re-detection).
    pub detection: DetectionResult,
    /// Whether auto-strip was performed on open.
    pub was_stripped: bool,
    /// The original sequence number values, keyed by line number.
    /// Populated when stripping occurs; used for NUMBER SHOW overlay and restore_on_save.
    pub side_table: SideTable,
    /// Whether NUMBER SHOW overlay mode is active.
    pub show_mode_active: bool,
    /// Whether NUMBER ON auto-numbering mode is active.
    pub auto_numbering_active: bool,
    /// The current auto-numbering state (next value, increment, target columns).
    pub auto_number_state: Option<AutoNumberState>,
    /// The display status for the status bar.
    pub status_indicator: SeqNumStatusIndicator,
}

/// The status indicator displayed in the status bar.
/// Addresses: Requirements 4.1, 4.2, 4.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqNumStatusIndicator {
    /// No sequence columns detected or defined — no indicator shown.
    None,
    /// Sequence numbers detected and stripped. Shows "SEQNUM x-y[,x-y]".
    Stripped {
        has_front: bool,
        has_back: bool,
    },
    /// Sequence numbers detected but NOT stripped. Shows "SEQNUM?".
    DetectedNotStripped,
    /// NUMBER SHOW overlay is active. Shows "SEQSHOW".
    ShowMode,
}

/// Auto-numbering state for NUMBER ON mode.
/// Addresses: Requirements 6.7, 6.8
#[derive(Debug, Clone)]
pub struct AutoNumberState {
    /// The next sequence value to assign.
    pub next_value: u64,
    /// The increment between values.
    pub increment: u64,
    /// The target column range.
    pub target_columns: SeqNumColumns,
    /// The format to use for generated numbers.
    pub format: SeqNumFormat,
}
```

### SideTable

```rust
/// Stores original sequence number column content stripped from the edit buffer.
/// Enables NUMBER SHOW overlay rendering and restore_on_save functionality.
/// Addresses: Requirements 3.9, 8.2, 11.5
#[derive(Debug, Clone)]
pub struct SideTable {
    /// Front column content by 0-based line index. None entries indicate
    /// lines that were shorter than the column range or had no content stripped.
    front_entries: Vec<Option<String>>,
    /// Back column content by 0-based line index.
    back_entries: Vec<Option<String>>,
    /// The column range used for front entries.
    front_columns: Option<SeqNumColumns>,
    /// The column range used for back entries.
    back_columns: Option<SeqNumColumns>,
}

impl SideTable {
    /// Create a new empty side-table.
    pub fn new() -> Self;

    /// Store the original front column content for a line.
    pub fn set_front(&mut self, line_index: usize, content: String);

    /// Store the original back column content for a line.
    pub fn set_back(&mut self, line_index: usize, content: String);

    /// Retrieve the original front column content for a line.
    pub fn get_front(&self, line_index: usize) -> Option<&str>;

    /// Retrieve the original back column content for a line.
    pub fn get_back(&self, line_index: usize) -> Option<&str>;

    /// Returns the number of lines tracked.
    pub fn len(&self) -> usize;

    /// Returns true if no entries are stored.
    pub fn is_empty(&self) -> bool;

    /// Adjust line indices after a line insertion at the given index.
    /// Shifts entries at and after the index down by count.
    pub fn on_lines_inserted(&mut self, at_index: usize, count: usize);

    /// Adjust line indices after lines are deleted.
    /// Removes entries in the deleted range and shifts subsequent entries up.
    pub fn on_lines_deleted(&mut self, at_index: usize, count: usize);
}
```

### SeqNumConfig

```rust
/// Typed representation of all `editor.sequence_numbers.*` configuration keys.
/// Read from ff-config at initialization and on hot-reload.
/// Addresses: Requirements 2.8, 7.5, 11.5, 12.1, 12.2
#[derive(Debug, Clone)]
pub struct SeqNumConfig {
    /// Detection threshold percentage (50–100, default 80).
    /// Minimum percentage of sampled lines that must match numeric pattern.
    pub detection_threshold: u8,
    /// Sample size — maximum non-blank lines to sample (5–100, default 20).
    pub sample_size: u8,
    /// Whether to highlight sequence columns with background shading.
    pub highlight_columns: bool,
    /// Default sequence number format ("numeric" or "alpha:PREFIX").
    pub default_format: SeqNumFormat,
    /// Whether to restore sequence numbers on save.
    pub restore_on_save: bool,
}

/// Per-language configuration override.
/// Addresses: Requirements 12.2, 12.3, 12.4
#[derive(Debug, Clone)]
pub struct SeqNumLanguageOverride {
    /// Override auto_unnum setting for this language.
    pub auto_unnum: Option<bool>,
    /// Override front sequence columns.
    pub sequence_cols_front: Option<SeqNumColumns>,
    /// Override back sequence columns.
    pub sequence_cols_back: Option<SeqNumColumns>,
    /// Override detection threshold.
    pub detection_threshold: Option<u8>,
    /// Override sample size.
    pub sample_size: Option<u8>,
}

impl SeqNumConfig {
    /// Load configuration from ff-config, applying validation and clamping.
    /// Out-of-range values are clamped and a WARN log is emitted.
    /// Addresses: Requirements 2.8, 12.1
    pub fn from_config(config: &dyn ConfigProvider) -> Self;

    /// Load per-language override for the specified language ID.
    /// Returns None if no override exists.
    /// Addresses: Requirement 12.2
    pub fn language_override(config: &dyn ConfigProvider, language_id: &str)
        -> Option<SeqNumLanguageOverride>;
}
```

---

## Public API Surface

### SequenceDetector

```rust
/// Samples file content to detect sequence number presence.
/// Read-only operation — never modifies the edit buffer.
/// Addresses: Requirements 2.1–2.9
pub struct SequenceDetector { /* ... */ }

impl SequenceDetector {
    /// Create a new detector with the given configuration.
    pub fn new(config: &SeqNumConfig) -> Self;

    /// Detect sequence numbers in the given document.
    /// Reads line content from the document model and evaluates
    /// defined column ranges against the detection heuristics.
    /// Addresses: Requirements 2.1–2.5, 2.9
    pub fn detect(
        &self,
        document: &dyn DocumentAccess,
        front_columns: Option<SeqNumColumns>,
        back_columns: Option<SeqNumColumns>,
    ) -> DetectionResult;

    /// Update configuration (on hot-reload). New settings apply to
    /// subsequent detect() calls only.
    pub fn update_config(&mut self, config: &SeqNumConfig);
}
```

### SequenceStripper

```rust
/// Removes sequence numbers from the edit buffer and records originals
/// in the side-table for potential restoration or overlay display.
/// Addresses: Requirements 3.1–3.9, 5.1–5.11
pub struct SequenceStripper { /* ... */ }

impl SequenceStripper {
    /// Create a new stripper.
    pub fn new() -> Self;

    /// Strip sequence columns from all lines in the document.
    /// Replaces column content with spaces. Stores originals in the side-table.
    /// Returns the number of lines actually modified (skips already-blank columns).
    /// Addresses: Requirements 3.1, 3.2, 5.8
    pub fn strip_all(
        &self,
        document: &mut dyn DocumentMutate,
        columns: &[SeqNumColumns],
        side_table: &mut SideTable,
    ) -> StripResult;

    /// Strip sequence columns from a range of lines (CC block scoped).
    /// Addresses: Requirement 5.7
    pub fn strip_range(
        &self,
        document: &mut dyn DocumentMutate,
        columns: &[SeqNumColumns],
        start_line: usize,
        end_line: usize,
        side_table: &mut SideTable,
    ) -> StripResult;

    /// Strip an explicit column range (UNNUM COLS start end).
    /// Does not use language profile — uses the caller-specified range.
    /// Addresses: Requirement 5.3
    pub fn strip_explicit(
        &self,
        document: &mut dyn DocumentMutate,
        columns: SeqNumColumns,
        line_range: Option<(usize, usize)>,
        side_table: &mut SideTable,
    ) -> StripResult;

    /// Restore previously stripped content from the side-table back into
    /// the document. Used for UNDO reversal of strip operations.
    /// Addresses: Requirements 9.1, 9.5
    pub fn restore_from_side_table(
        &self,
        document: &mut dyn DocumentMutate,
        side_table: &SideTable,
    ) -> usize;
}

/// Result of a strip operation.
#[derive(Debug, Clone)]
pub struct StripResult {
    /// Number of lines modified (had non-blank content cleared).
    pub lines_modified: usize,
    /// Total lines examined.
    pub lines_examined: usize,
    /// Column ranges that were stripped.
    pub columns_stripped: Vec<SeqNumColumns>,
}
```

### SequenceNumberer

```rust
/// Writes sequential numbers into defined column positions in the edit buffer.
/// Addresses: Requirements 6.1–6.12, 7.1–7.4
pub struct SequenceNumberer { /* ... */ }

impl SequenceNumberer {
    /// Create a new numberer.
    pub fn new() -> Self;

    /// Write sequence numbers to all lines using the specified parameters.
    /// Returns the result including any overflow warnings.
    /// Addresses: Requirements 6.3, 6.4, 6.6, 6.11
    pub fn number_all(
        &self,
        document: &mut dyn DocumentMutate,
        columns: SeqNumColumns,
        start_value: u64,
        increment: u64,
        format: &SeqNumFormat,
    ) -> NumberResult;

    /// Write sequence numbers to a range of lines (CC block scoped).
    /// The sequence counter restarts from start_value for the block.
    /// Addresses: Requirement 6.12
    pub fn number_range(
        &self,
        document: &mut dyn DocumentMutate,
        columns: SeqNumColumns,
        start_line: usize,
        end_line: usize,
        start_value: u64,
        increment: u64,
        format: &SeqNumFormat,
    ) -> NumberResult;

    /// Assign the next auto-number to a newly inserted line.
    /// Called by the insert operation hook when NUMBER ON is active.
    /// Addresses: Requirements 6.7, 9.4
    pub fn auto_number_line(
        &self,
        document: &mut dyn DocumentMutate,
        line_index: usize,
        state: &mut AutoNumberState,
    ) -> Result<(), SeqNumError>;
}

/// Result of a numbering operation.
#[derive(Debug, Clone)]
pub struct NumberResult {
    /// Number of lines numbered.
    pub lines_numbered: usize,
    /// Whether any sequence values overflowed the column width.
    pub overflow_occurred: bool,
    /// The line index where overflow first occurred (if any).
    pub overflow_at_line: Option<usize>,
}
```

### SeqNumStateManager

```rust
/// Manages per-document sequence number state.
/// One instance exists per open document. Provides the data needed by
/// the status bar, NUMBER SHOW overlay, save hook, and BOUNDS interaction.
/// Addresses: Requirements 3.9, 4.1–4.5, 8.1–8.7, 10.1–10.4
pub struct SeqNumStateManager { /* ... */ }

impl SeqNumStateManager {
    /// Create a new state manager for a document.
    pub fn new(document_id: DocumentId) -> Self;

    /// Returns the current state for this document.
    pub fn state(&self) -> &SequenceNumberState;

    /// Returns the status indicator for the status bar.
    pub fn status_indicator(&self) -> SeqNumStatusIndicator;

    /// Returns the side-table reference for overlay rendering.
    pub fn side_table(&self) -> &SideTable;

    /// Returns true if NUMBER SHOW overlay mode is active.
    pub fn is_show_mode_active(&self) -> bool;

    /// Toggle NUMBER SHOW mode on or off.
    /// Addresses: Requirements 8.1, 8.4, 8.6
    pub fn toggle_show_mode(&mut self);

    /// Returns true if NUMBER ON auto-numbering is active.
    pub fn is_auto_numbering_active(&self) -> bool;

    /// Enable auto-numbering mode with the specified parameters.
    /// Addresses: Requirement 6.7
    pub fn enable_auto_numbering(
        &mut self,
        columns: SeqNumColumns,
        start_value: u64,
        increment: u64,
        format: SeqNumFormat,
    );

    /// Disable auto-numbering mode.
    /// Addresses: Requirement 6.8
    pub fn disable_auto_numbering(&mut self);

    /// Record that detection was performed.
    pub fn set_detection_result(&mut self, result: DetectionResult);

    /// Record that stripping was performed on open.
    pub fn set_stripped(&mut self, side_table: SideTable);

    /// Returns the defined sequence column ranges (from profile + overrides).
    /// Used by BOUNDS interaction to determine exclusion zones.
    /// Addresses: Requirements 10.1–10.4
    pub fn active_columns(&self) -> (Option<SeqNumColumns>, Option<SeqNumColumns>);

    /// Notify the state manager that lines were inserted.
    /// Updates the side-table to maintain correct line alignment.
    pub fn on_lines_inserted(&mut self, at_index: usize, count: usize);

    /// Notify the state manager that lines were deleted.
    pub fn on_lines_deleted(&mut self, at_index: usize, count: usize);
}
```

### SaveHook

```rust
/// Integrates with the file-operations save pipeline to handle
/// sequence number restore/strip behaviour on save.
/// Addresses: Requirements 11.1–11.6
pub struct SaveHook { /* ... */ }

impl SaveHook {
    /// Create a new save hook with access to state and config.
    pub fn new() -> Self;

    /// Called by the save pipeline before writing to disk.
    /// Returns the content to save — either the edit buffer as-is (default)
    /// or with sequence numbers restored from the side-table.
    /// Addresses: Requirements 11.1, 11.2, 11.5, 11.6
    pub fn prepare_save_content(
        &self,
        document: &dyn DocumentAccess,
        state: &SequenceNumberState,
        config: &SeqNumConfig,
    ) -> SaveContentDecision;
}

/// The decision on how to handle save content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveContentDecision {
    /// Save the edit buffer content as-is (sequence columns contain spaces).
    SaveAsIs,
    /// Restore sequence numbers before saving (inject side-table content).
    RestoreAndSave {
        /// Lines to modify with restored content, keyed by line index.
        restorations: Vec<LineRestoration>,
    },
}

/// A single line restoration entry for save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineRestoration {
    /// The 0-based line index.
    pub line_index: usize,
    /// Content to insert at the front column range.
    pub front_content: Option<String>,
    /// Content to insert at the back column range.
    pub back_content: Option<String>,
}
```

### Command Handlers

```rust
/// UNNUM command handler — registered as Command_ID "sequence.unnum".
/// Addresses: Requirements 5.1–5.11
pub struct UnnumCommand { /* ... */ }

impl UnnumCommand {
    /// Execute the UNNUM command with the given arguments.
    /// Dispatches to the appropriate strip variant based on arguments.
    ///
    /// Supported forms:
    /// - `UNNUM` — strip using language profile columns
    /// - `UNNUM COLS start end` — strip explicit column range
    /// - `UNNUM FRONT` — strip front columns only
    /// - `UNNUM BACK` — strip back columns only
    /// - `UNNUM ALL` — strip both front and back
    ///
    /// When combined with CC block: restricts to block range.
    /// Records a Sequence_Transaction for undo.
    pub fn execute(
        &self,
        args: &CommandArgs,
        context: &mut CommandContext,
    ) -> Result<CommandOutput, SeqNumError>;
}

/// NUMBER command handler — registered as Command_ID "sequence.number".
/// Addresses: Requirements 6.1–6.12, 7.1–7.4
pub struct NumberCommand { /* ... */ }

impl NumberCommand {
    /// Execute the NUMBER command with the given arguments.
    /// Dispatches to the appropriate variant based on arguments.
    ///
    /// Supported forms:
    /// - `NUMBER` — display usage summary
    /// - `NUMBER COLS start end [FORMAT format]` — explicit column numbering
    /// - `NUMBER STD [start_val increment]` — language profile column numbering
    /// - `NUMBER ON` — enable auto-numbering
    /// - `NUMBER OFF` — disable auto-numbering
    ///
    /// Sequencing forms (COLS, STD) require confirmation before modifying.
    /// Records a Sequence_Transaction for undo.
    pub fn execute(
        &self,
        args: &CommandArgs,
        context: &mut CommandContext,
    ) -> Result<CommandOutput, SeqNumError>;
}

/// NUMBER SHOW command handler — registered as Command_ID "sequence.number_show".
/// Addresses: Requirements 8.1–8.7
pub struct NumberShowCommand { /* ... */ }

impl NumberShowCommand {
    /// Toggle the NUMBER SHOW overlay mode.
    /// Non-undoable display state change.
    pub fn execute(
        &self,
        args: &CommandArgs,
        context: &mut CommandContext,
    ) -> Result<CommandOutput, SeqNumError>;
}
```

---

## Error Handling

```rust
/// Errors produced by the sequence numbers subsystem.
/// Addresses: Cross-cutting error format: [seqnum] operation: description
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SeqNumError {
    /// No sequence columns defined for the active language.
    #[error("[seqnum] unnum: no sequence columns defined for this language — use UNNUM COLS to specify a range")]
    NoColumnsDefinedForUnnum,

    /// No sequence columns defined for NUMBER STD.
    #[error("[seqnum] number_std: no sequence columns defined for this language")]
    NoColumnsDefinedForNumber,

    /// Front columns not defined (UNNUM FRONT / NUMBER with front expectation).
    #[error("[seqnum] {operation}: front sequence columns not defined for this language")]
    FrontColumnsNotDefined { operation: String },

    /// Back columns not defined (UNNUM BACK / NUMBER with back expectation).
    #[error("[seqnum] {operation}: back sequence columns not defined for this language")]
    BackColumnsNotDefined { operation: String },

    /// Invalid column range specification.
    #[error("[seqnum] columns: invalid range '{value}' — start must be <= end, both > 0")]
    InvalidColumnRange { value: String },

    /// Invalid start value or increment for NUMBER command.
    #[error("[seqnum] number: start value and increment must be positive integers")]
    InvalidNumberParams,

    /// Prefix too long for the column width.
    #[error("[seqnum] number: prefix too long for column range")]
    PrefixTooLong,

    /// Sequence number overflow — value exceeds column width capacity.
    #[error("[seqnum] number: sequence overflow — numbers truncated to fit COLS {start}-{end}")]
    SequenceOverflow { start: u16, end: u16 },

    /// Command not applicable in Grid Edit Mode.
    #[error("[seqnum] {command}: not applicable in Grid Edit Mode")]
    NotApplicableInGridMode { command: String },

    /// Auto-numbering conflict with active BOUNDS.
    #[error("[seqnum] number_on: sequence columns overlap with active BOUNDS — auto-numbering disabled for overlapping range")]
    BoundsOverlap,

    /// Configuration value out of valid range (value was clamped).
    #[error("[seqnum] config '{key}': value {value} out of range [{min}–{max}], clamped to {clamped}")]
    ConfigClamped {
        key: String,
        value: String,
        min: String,
        max: String,
        clamped: String,
    },

    /// Document model access error.
    #[error("[seqnum] document: {0}")]
    DocumentAccess(String),

    /// Undo system error.
    #[error("[seqnum] undo: failed to record transaction — {0}")]
    UndoRecordFailed(String),
}
```

---

## Integration Points

### Integration with `ff-document-model` (Document Model)

| Integration | Detail |
|-------------|--------|
| **DocumentAccess trait** | `SequenceDetector` reads line content and line count through this read-only trait |
| **DocumentMutate trait** | `SequenceStripper` and `SequenceNumberer` mutate line content via this trait |
| **Line content slicing** | Column operations work on byte slices within lines; requires fixed-width character assumption (EBCDIC/ASCII single-byte) |
| **Line count queries** | Detector uses `line_count()` to determine sample boundaries and short-file thresholds |
| **Modification tracking** | Strip/number operations mark affected lines as modified via the document model |

### Integration with `ff-language-service` (Language Service)

| Integration | Detail |
|-------------|--------|
| **LanguageProfile access** | On file open, queries the active profile for `sequence_cols_front`, `sequence_cols_back`, and `auto_unnum` values |
| **Profile TOML schema** | Defines the optional keys this crate reads: `sequence_cols_front`, `sequence_cols_back`, `auto_unnum` |
| **Language ID resolution** | Used to look up per-language config overrides from `editor.sequence_numbers.languages.<id>` |
| **No-columns early exit** | When profile defines neither column range, the entire detection/strip pipeline is skipped (Req 1.9) |

### Integration with `ff-command` (Command Framework)

| Integration | Detail |
|-------------|--------|
| **Command registration** | Registers `sequence.unnum` (Edit + Browse), `sequence.number` (Edit only), `sequence.number_show` (Edit + Browse) |
| **Command dispatch** | Command handlers receive `CommandArgs` and `CommandContext`; return `CommandOutput` for status messages |
| **Confirmation protocol** | NUMBER sequencing forms use the command framework's confirmation mechanism (YES/NO prompt) before modifying the buffer |
| **CC block context** | When commands are combined with CC line commands, the block range is passed via `CommandContext` |

### Integration with `ff-undo` (Undo/Redo Transactions)

| Integration | Detail |
|-------------|--------|
| **Sequence_Transaction** | UNNUM and NUMBER (sequencing forms) wrap all line modifications in a single transaction pushed to the Undo_Stack |
| **Non-undoable classification** | Auto-strip on file open is NOT recorded in the Undo_Stack (session initialisation) |
| **Restoration on UNDO** | When UNDO reverses a strip, the exact original byte content is restored (not just re-inserted blanks) |
| **Auto-number co-transaction** | NUMBER ON insertions join the same transaction as the triggering line-insert operation |

### Integration with `ff-config` (Configuration System)

| Integration | Detail |
|-------------|--------|
| **Namespace `editor.sequence_numbers.*`** | Global settings: `detection_threshold`, `sample_size`, `highlight_columns`, `default_format`, `restore_on_save` |
| **Per-language overrides** | `editor.sequence_numbers.languages.<lang_id>.*` tables override global and profile settings |
| **Hot-reload callback** | Display settings (`highlight_columns`, NUMBER SHOW style) apply immediately; detection settings apply to newly opened files only |
| **Validation + clamping** | Out-of-range values are clamped with WARN log (threshold 50–100, sample_size 5–100) |

### Integration with `ff-file-ops` (File Operations)

| Integration | Detail |
|-------------|--------|
| **Save pipeline hook** | `SaveHook::prepare_save_content()` is called before writing; returns either pass-through or restoration instructions |
| **Default behaviour** | Save writes edit buffer as-is (stripped columns contain spaces — no sequence numbers in output) |
| **Restore mode** | When `restore_on_save=true`, injects side-table content into save output without modifying the edit buffer |
| **New-line numbering on save** | If lines were inserted since open and `restore_on_save=true`, generates new numbers for those lines |

### Integration with `ff-line-commands` (Line Commands)

| Integration | Detail |
|-------------|--------|
| **CC block range** | UNNUM and NUMBER support scoping via CC...CC block pairs; the line command system resolves the range |
| **Range validation** | If CC block is invalid or spans zero lines, the command reports an error through the standard error channel |

### Integration with `ff-edit-operations` (Edit Operations / BOUNDS)

| Integration | Detail |
|-------------|--------|
| **BOUNDS exclusion query** | Edit operations may query `SeqNumStateManager::active_columns()` to determine which columns are outside editable bounds |
| **No BOUNDS mutation** | Sequence operations NEVER alter active BOUNDS settings; BOUNDS are session state owned by navigation-commands |
| **Overlap warning** | When NUMBER ON is active and sequence columns overlap BOUNDS, a warning is displayed and auto-numbering is disabled for the overlap |

---

## Correctness Properties

These properties define invariants suitable for property-based testing with `proptest`.

### Property 1: Column Range Parse Roundtrip

**Statement:** For any valid column range with start in [1, 999] and end in [start, 999], formatting as `"{start}-{end}"` and re-parsing via `SeqNumColumns::parse()` yields the same start and end values.

**Validates: Requirements 1.1, 1.2, 1.4**

### Property 2: Detection Is Read-Only

**Statement:** For any document content and any column range parameters, calling `SequenceDetector::detect()` does not modify any line content in the document. The document state before and after detection is byte-identical.

**Validates: Requirement 2.7**

### Property 3: Detection Threshold Correctness

**Statement:** For any set of N non-blank lines (N >= 5) and a threshold T (50–100), the detector reports "detected" for a column range if and only if at least ⌈N × T / 100⌉ lines have that range fully populated with digits or spaces, with at least one line containing all digits.

**Validates: Requirements 2.2, 2.8**

### Property 4: Short-File 100% Threshold

**Statement:** For any document with fewer than 5 non-blank lines, the detector reports "detected" if and only if 100% of sampled non-blank lines match the numeric criterion (not the configurable threshold).

**Validates: Requirement 2.3**

### Property 5: Strip Preserves Non-Sequence Content

**Statement:** For any document and any column range [s, e], after stripping, every character outside positions [s-1, e-1] (0-based) on every line is unchanged. Only characters within the column range are modified.

**Validates: Requirements 3.2, 5.8**

### Property 6: Strip Produces Spaces in Range

**Statement:** After stripping column range [s, e] from a line of length >= e, positions [s-1, e-1] (0-based) contain only space characters (0x20).

**Validates: Requirement 3.2**

### Property 7: Side-Table Faithfully Records Originals

**Statement:** For any line that is modified by a strip operation, the side-table entry for that line contains the exact byte content that occupied the column range before stripping.

**Validates: Requirements 3.9, 9.5**

### Property 8: Undo Reversal Restores Exact Content

**Statement:** For any UNNUM operation that modifies N lines, undoing the Sequence_Transaction restores each of those N lines to their exact pre-strip byte content. The document state after undo is identical to the state before the UNNUM command.

**Validates: Requirements 9.1, 9.5**

### Property 9: NUMBER Format Width Invariant

**Statement:** For any SeqNumFormat (Numeric or AlphaPrefix) and any column width W > 0, `format_value(v, W)` returns a string of exactly W characters for all v where v <= max_value(W), and returns None for v > max_value(W).

**Validates: Requirements 6.6, 6.11, 7.1, 7.2**

### Property 10: NUMBER Sequence Monotonicity

**Statement:** For any start_value S > 0 and increment I > 0, numbering lines 0..N produces values S, S+I, S+2I, ..., S+(N-1)*I in line order, until overflow occurs.

**Validates: Requirements 6.5, 6.6**

### Property 11: NUMBER SHOW Does Not Modify Buffer

**Statement:** For any document with NUMBER SHOW active, the edit buffer content is identical before and after toggling NUMBER SHOW on. Toggling off also leaves the buffer unchanged. Save while NUMBER SHOW is active writes the edit buffer — not the overlay values.

**Validates: Requirements 8.3, 8.4**

### Property 12: Save Without Restore Preserves Strip

**Statement:** When `restore_on_save=false` (default), the saved output for any line equals the current edit buffer content for that line. No sequence numbers are injected into the save output.

**Validates: Requirements 11.1, 11.2**

### Property 13: Save With Restore Injects Side-Table

**Statement:** When `restore_on_save=true` and stripping occurred, the saved output for each unmodified line has the original side-table content restored into the defined column positions. The edit buffer remains unchanged (still stripped).

**Validates: Requirements 11.5, 11.6**

### Property 14: Config Clamping Idempotence

**Statement:** For any detection_threshold value V, clamping to [50, 100] yields `V.clamp(50, 100)`. For any sample_size value V, clamping to [5, 100] yields `V.clamp(5, 100)`. The clamped value is always within the valid range.

**Validates: Requirements 2.8, 12.1**

### Property 15: Front and Back Independence

**Statement:** The detection, stripping, and numbering of front columns is independent of back columns and vice versa. Modifying or detecting one range does not affect the content or detection status of the other range.

**Validates: Requirement 2.4**

### Property 16: Already-Blank Lines Are Unmodified

**Statement:** For any line where the target column range is already entirely spaces, the strip operation reports that line as NOT modified and does not create a side-table entry with meaningful content changes.

**Validates: Requirement 5.8**

---

## Testing Strategy

- **Crate:** `proptest` (already in workspace `[dev-dependencies]`)
- **Minimum iterations:** 256 per property
- **Generators:** Custom strategies for document content (ASCII lines with configurable length), column ranges, detection thresholds, sequence formats, and line counts
- **Regression files:** Committed alongside tests in `tests/` directory
- **Unit tests:** Co-located in each module's `#[cfg(test)] mod tests` block
- **Integration tests:** `tests/` directory exercising full file-open lifecycle (detect → strip → NUMBER SHOW → save)
- **Coverage target:** Every acceptance criterion from requirements.md has at least one automated test
- **Fixture files:** COBOL, JCL, FORTRAN, and PL/I sample files in `tests/fixtures/` for realistic detection testing
