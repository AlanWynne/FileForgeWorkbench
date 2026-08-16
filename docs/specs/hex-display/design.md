# Design Document: Hex Display Mode (`ff-hex`)

## Overview

The `ff-hex` crate implements the **hexadecimal display and editing subsystem** for FileForgeWorkbench. It provides a complete hex editor experience: toggling between text and hex modes, three-pane layout rendering (offset/hex/ASCII), in-place byte editing, hex search integration, cursor synchronisation between panes, hex dump export, and goto-offset navigation.

### Purpose

- Toggle between normal text display and hex display mode via `HEX ON`/`HEX OFF`/`HEX` commands
- Manage the hex view layout model: Offset_Column, Hex_Pane, ASCII_Pane with configurable bytes-per-row
- Provide byte-level overwrite editing in both hex and ASCII panes
- Synchronise cursor position between panes and maintain nibble-level precision
- Integrate with the find-and-replace engine for `FIND X'...'` hex byte searches
- Track modified bytes for visual highlighting until save
- Calculate hex-mode viewport dimensions and coordinate with the scrolling subsystem
- Export hex dumps to clipboard, file, or new editor tab
- Navigate to arbitrary byte offsets via `GOTO X'...'` command
- Persist hex mode session state per file for restore on reopen

### Position in Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              Shell Layer: ff-desktop (egui)                   │
│  Renders hex grid from HexViewModel; routes key events       │
├─────────────────────────────────────────────────────────────┤
│  Peer Feature Crates:                                        │
│    ff-find-and-replace (hex search FIND X'...')              │
│    ff-fileforge-integration (field boundaries in hex)        │
├─────────────────────────────────────────────────────────────┤
│  THIS CRATE: ff-hex ← Wave 11 (Display Modes)               │
├─────────────────────────────────────────────────────────────┤
│  Upstream:                                                   │
│    ff-document-model (raw byte buffer, BytePosition)         │
│    ff-edit-operations (overwrite mutation primitives)         │
│    ff-undo-redo-transactions (transaction recording)         │
│    ff-command (command registration/dispatch)                 │
│    ff-viewport-scrolling (viewport coordination)             │
│    ff-config (hex settings persistence)                      │
│    ff-clipboard-operations (hex copy/paste)                  │
├─────────────────────────────────────────────────────────────┤
│              Foundation Layer: ff-logging                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Constraints (Cross-Cutting)

- **Command-Driven (Req 4)**: `HEX ON`, `HEX OFF`, `HEX`, `HEX DUMP`, `GOTO X'...'` registered in `ff-command`
- **GUI Independence (Req 2)**: Zero GUI dependencies — the hex layout model is pure data; rendering is the shell's concern
- **Multi-Crate Workspace (Req 7)**: Crate at `crates/ff-hex`
- **Error Message Standards (Req 8)**: All errors follow `[hex] operation: description` format
- **Async I/O (Req 6)**: Large hex dump export supports cancellation tokens
- **Configuration (Req 5)**: Settings under `editor.hex.*` namespace in `ff-config`

---

## Architecture

### High-Level Architecture Diagram

```mermaid
graph TD
    subgraph Consumers [Consuming Crates / Shell]
        DESKTOP[ff-desktop<br/>egui hex grid renderer]
        FFI[ff-fileforge-integration<br/>field boundary annotations]
        SESSION[ff-startup-and-session<br/>hex state persistence]
    end

    subgraph ff-hex [ff-hex Crate]
        HM[HexModeController<br/>activate/deactivate/toggle]
        HL[HexLayout<br/>row geometry, pane widths]
        HC[HexCursor<br/>position, pane, nibble]
        HE[HexEditState<br/>byte modification engine]
        HS[HexSearchBridge<br/>FIND X integration]
        HV[HexViewportAdapter<br/>row-based scrolling]
        HD[HexDumpExporter<br/>dump to text/clip/file]
        HG[HexGotoHandler<br/>offset navigation]
        HMB[ModifiedByteTracker<br/>save-state diff]
        HVM[HexViewModel<br/>renderable row data]
        HCFG[HexConfig<br/>settings binding]
        HCMD[HexCommands<br/>command registration]
        HSESS[HexSessionState<br/>per-file persistence]
    end

    subgraph Upstream [Upstream Crates]
        DOC[ff-document-model<br/>TextBuffer / BytePosition]
        EDIT[ff-edit-operations<br/>overwrite mutation]
        UNDO[ff-undo-redo-transactions<br/>transaction recording]
        CMD[ff-command<br/>registry + dispatch]
        VP[ff-viewport-scrolling<br/>viewport model]
        CFG[ff-config<br/>settings access]
        FIND[ff-find-and-replace<br/>hex byte search]
        CLIP[ff-clipboard-operations<br/>copy/paste]
        LOG[ff-logging]
    end

    DESKTOP -->|render from| HVM
    DESKTOP -->|key events| HC
    FFI -->|field boundaries| HL
    SESSION -->|restore/save| HSESS

    HM --> HL
    HM --> HC
    HM --> HE
    HM --> HV
    HM --> HVM
    HM --> HCMD

    HC --> HL
    HE --> HMB
    HE --> DOC
    HE --> EDIT
    HE --> UNDO
    HS --> FIND
    HS --> HM
    HV --> VP
    HV --> HL
    HD --> DOC
    HD --> HL
    HD --> CLIP
    HG --> HC
    HG --> HV
    HCFG --> CFG
    HCMD --> CMD
    HM --> LOG
end
```

### Layer Placement

| Component | Responsibility |
|-----------|---------------|
| **HexModeController** | Top-level orchestrator: activates/deactivates hex mode, coordinates state transitions, owns all hex sub-components |
| **HexLayout** | Computes row geometry: column widths for offset/hex/ASCII panes, group separators, padding for partial rows |
| **HexCursor** | Cursor state: byte offset, active pane (Hex or ASCII), nibble position (high/low), navigation logic |
| **HexEditState** | Editing engine: validates input, applies nibble/byte overwrites, delegates mutations to edit-operations |
| **HexSearchBridge** | Integrates with ff-find-and-replace: auto-activates hex mode on match, highlights byte ranges in both panes |
| **HexViewportAdapter** | Adapts hex row-based scrolling to the viewport model: row count calculation, page size, scroll clamping |
| **HexDumpExporter** | Formats and exports hex dump: full document or byte range, to clipboard/file/new-tab |
| **HexGotoHandler** | Parses offset arguments, validates bounds, positions cursor and scrolls viewport |
| **ModifiedByteTracker** | Tracks which byte offsets differ from last-saved state; updates on edit, undo, redo, and save |
| **HexViewModel** | Produces renderable row data: pre-computed strings for offset, hex digits, ASCII characters, highlights |
| **HexConfig** | Reads and watches `editor.hex.*` settings: bytes_per_row, digit_case, auto_activate_binary |
| **HexCommands** | Registers HEX ON/OFF/toggle, HEX DUMP, GOTO commands with the command framework |
| **HexSessionState** | Serialisable per-file state: mode on/off, bytes_per_row, cursor offset, viewport top row, active pane |

### Data Flow: HEX ON Activation

```
1. User issues "HEX ON" → command framework dispatches to HexCommands
2. HexCommands checks current state via HexModeController
3. If already active → emit status "Hex mode is already active", return
4. HexModeController.activate() called:
   a. Maps current text cursor byte position to hex row/column via HexLayout
   b. Initialises HexCursor at the mapped position, Hex_Pane, high nibble
   c. Calculates total hex rows via HexLayout.total_rows(doc_length)
   d. Notifies HexViewportAdapter to switch scrollbar to hex row mode
   e. Builds initial HexViewModel for visible rows
   f. Emits mode-change event for status bar update
5. Shell renders hex grid from HexViewModel
```

### Data Flow: Hex Byte Edit (Hex Pane)

```
1. User types hex digit (e.g., 'A') while cursor in Hex_Pane
2. Shell forwards key event to HexEditState via HexModeController
3. HexEditState validates: is hex digit? is Edit mode active?
4. HexEditState computes new byte value:
   - If nibble_position == High: new_byte = (digit << 4) | (old_byte & 0x0F)
   - If nibble_position == Low: new_byte = (old_byte & 0xF0) | digit
5. HexEditState delegates overwrite to ff-edit-operations at BytePosition
6. ff-edit-operations records Edit_Operation in undo-redo-transactions
7. ModifiedByteTracker marks the byte offset as modified
8. HexCursor advances: Low nibble → next byte High; High → same byte Low
9. HexViewModel regenerates affected row
10. Shell re-renders the updated row
```

### Data Flow: FIND X'0D0A' Hex Search

```
1. User issues "FIND X'0D0A'" → command-semantics parses as hex search
2. ff-find-and-replace executes raw byte search for [0x0D, 0x0A]
3. On match found: HexSearchBridge receives notification
4. If Hex_Mode not active: HexSearchBridge triggers HexModeController.activate()
5. HexSearchBridge computes match byte range → hex row range
6. HexCursor positioned at first byte of match
7. HexViewportAdapter scrolls to reveal match row
8. HexViewModel marks match range as highlighted in both Hex_Pane and ASCII_Pane
9. Shell renders highlight overlays
```

---

## Components and Interfaces

### Module Structure

```
crates/ff-hex/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Public API re-exports, crate docs
│   ├── controller.rs               # HexModeController: top-level orchestrator
│   ├── layout.rs                   # HexLayout: row geometry computation
│   ├── cursor.rs                   # HexCursor: position, pane, nibble, navigation
│   ├── edit.rs                     # HexEditState: byte modification engine
│   ├── search_bridge.rs            # HexSearchBridge: FIND X'...' integration
│   ├── viewport_adapter.rs         # HexViewportAdapter: row-based scrolling
│   ├── dump.rs                     # HexDumpExporter: hex dump formatting/export
│   ├── goto.rs                     # HexGotoHandler: offset navigation
│   ├── modified_tracker.rs         # ModifiedByteTracker: save-state diffing
│   ├── view_model.rs               # HexViewModel: renderable row data
│   ├── config.rs                   # HexConfig: settings binding
│   ├── commands.rs                 # HexCommands: command framework registration
│   ├── session.rs                  # HexSessionState: per-file persistence
│   ├── types.rs                    # Shared types: HexMode, HexPane, NibblePosition
│   └── error.rs                    # HexError enum
└── tests/
    ├── layout_tests.rs             # Row geometry and pane width calculations
    ├── cursor_tests.rs             # Cursor navigation and synchronisation
    ├── edit_tests.rs               # Hex/ASCII editing correctness
    ├── search_bridge_tests.rs      # Hex search highlight integration
    ├── viewport_tests.rs           # Hex viewport row calculations
    ├── dump_tests.rs               # Hex dump export format verification
    ├── goto_tests.rs               # Goto offset parsing and navigation
    ├── modified_tracker_tests.rs   # Modified byte tracking across edit/undo/save
    ├── view_model_tests.rs         # Row rendering correctness
    ├── config_tests.rs             # Configuration binding and validation
    ├── session_tests.rs            # Session state serialisation round-trip
    └── property_tests.rs           # Cross-cutting proptest properties
```

---

## Data Models

### Core Enums and Newtypes

```rust
/// The current hex display mode state.
///
/// Addresses: Requirement 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HexMode {
    /// Normal text display (hex mode inactive).
    #[default]
    Off,
    /// Hex display mode is active.
    On,
}

impl HexMode {
    /// Toggle the mode.
    pub fn toggle(&self) -> Self {
        match self {
            Self::Off => Self::On,
            Self::On => Self::Off,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::On)
    }
}

/// Which pane currently has editing focus.
///
/// Addresses: Requirement 6
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HexPane {
    /// Focus is in the hex digit pane (editing nibbles).
    #[default]
    Hex,
    /// Focus is in the ASCII character pane.
    Ascii,
}

/// Position within a byte when editing in the Hex_Pane.
///
/// Addresses: Requirement 4 AC 1–2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NibblePosition {
    /// High nibble (first hex digit, bits 7–4).
    #[default]
    High,
    /// Low nibble (second hex digit, bits 3–0).
    Low,
}

impl NibblePosition {
    /// Advance to the next nibble position.
    /// High → Low (same byte), Low → High (next byte).
    pub fn advance(&self) -> (Self, bool) {
        match self {
            Self::High => (Self::Low, false),   // stay on same byte
            Self::Low => (Self::High, true),    // advance to next byte
        }
    }
}
```

### Configurable Values

```rust
/// Valid values for bytes displayed per hex row.
///
/// Addresses: Requirement 3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BytesPerRow {
    Eight = 8,
    Sixteen = 16,
    ThirtyTwo = 32,
    SixtyFour = 64,
}

impl Default for BytesPerRow {
    fn default() -> Self {
        Self::Sixteen
    }
}

impl BytesPerRow {
    /// Attempt to create from a raw u32 value.
    /// Returns None for invalid (non-power-of-2 or out-of-range) values.
    pub fn from_value(value: u32) -> Option<Self> {
        match value {
            8 => Some(Self::Eight),
            16 => Some(Self::Sixteen),
            32 => Some(Self::ThirtyTwo),
            64 => Some(Self::SixtyFour),
            _ => None,
        }
    }

    pub fn as_usize(&self) -> usize {
        *self as usize
    }
}

/// Hex digit display case preference.
///
/// Addresses: Requirement 13
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HexDigitCase {
    /// Display hex digits A–F in uppercase (default).
    #[default]
    Uppercase,
    /// Display hex digits a–f in lowercase.
    Lowercase,
}

/// Configuration for auto-activating hex mode on binary files.
///
/// Addresses: Requirement 10 AC 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutoActivateBinary {
    /// Prompt the user when binary content is detected (default).
    #[default]
    Prompt,
    /// Always activate hex mode for binary files without prompting.
    Always,
    /// Never auto-activate; user must manually invoke HEX ON.
    Never,
}
```

### HexLayout

```rust
/// Computes the geometry for the hex display: pane widths, separators,
/// row structure, and total row count.
///
/// Addresses: Requirement 2, Requirement 3
#[derive(Debug, Clone)]
pub struct HexLayout {
    /// Number of bytes shown per row.
    bytes_per_row: BytesPerRow,
    /// Width of the offset column in characters.
    /// 8 digits for files ≤ 4GB, expands for larger files.
    offset_width: u8,
    /// Whether to insert extra space at the half-row boundary.
    half_row_separator: bool,
    /// Hex digit case for display formatting.
    digit_case: HexDigitCase,
}

impl HexLayout {
    /// Create a new layout for a document of the given byte length.
    ///
    /// Addresses: Requirement 2 AC 2 (offset width auto-expansion)
    pub fn new(document_byte_length: u64, bytes_per_row: BytesPerRow) -> Self;

    /// Total number of hex rows needed to display the document.
    ///
    /// Addresses: Requirement 9 AC 1
    /// Formula: ceil(document_byte_length / bytes_per_row)
    pub fn total_rows(&self, document_byte_length: u64) -> u64;

    /// The byte offset of the first byte on the given row.
    pub fn row_start_offset(&self, row: u64) -> u64;

    /// Which row contains the given byte offset.
    pub fn row_for_offset(&self, byte_offset: u64) -> u64;

    /// Column position of a byte within the Hex_Pane (character column).
    pub fn hex_column_for_byte(&self, byte_index_in_row: usize) -> usize;

    /// Column position of a byte within the ASCII_Pane.
    pub fn ascii_column_for_byte(&self, byte_index_in_row: usize) -> usize;

    /// Total width in characters of one complete hex row
    /// (offset + separator + hex pane + separator + ASCII pane).
    pub fn total_row_width(&self) -> usize;

    /// Width of the hex pane region in characters.
    /// Each byte = 2 hex digits + 1 space; plus half-row separator spaces.
    pub fn hex_pane_width(&self) -> usize;

    /// Width of the ASCII pane region in characters (= bytes_per_row).
    pub fn ascii_pane_width(&self) -> usize;

    /// Update bytes_per_row, recalculating geometry.
    pub fn set_bytes_per_row(&mut self, bpr: BytesPerRow, document_byte_length: u64);

    /// Update digit case setting.
    pub fn set_digit_case(&mut self, case: HexDigitCase);
}
```

### HexCursor

```rust
/// The cursor state in hex display mode. Tracks the current byte offset,
/// active pane, nibble position, and provides navigation.
///
/// Addresses: Requirement 6
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexCursor {
    /// Absolute byte offset in the document (0-based).
    byte_offset: u64,
    /// Which pane has focus.
    active_pane: HexPane,
    /// When in Hex_Pane: which nibble is selected.
    nibble: NibblePosition,
}

impl HexCursor {
    /// Create a new cursor at byte offset 0, Hex pane, high nibble.
    pub fn new() -> Self;

    /// Create a cursor positioned at a specific byte offset.
    pub fn at_offset(offset: u64) -> Self;

    /// Current byte offset.
    pub fn byte_offset(&self) -> u64;

    /// Current active pane.
    pub fn active_pane(&self) -> HexPane;

    /// Current nibble position (only meaningful when active_pane is Hex).
    pub fn nibble(&self) -> NibblePosition;

    /// Switch focus between Hex and ASCII panes (Tab key).
    /// Preserves byte offset.
    ///
    /// Addresses: Requirement 6 AC 3–4
    pub fn switch_pane(&mut self);

    /// Move cursor right.
    /// - In Hex_Pane: advance by one nibble (wraps to next byte at row end → next row start).
    /// - In ASCII_Pane: advance by one byte (wraps at row end → next row start).
    ///
    /// Addresses: Requirement 6 AC 6–8
    pub fn move_right(&mut self, layout: &HexLayout, document_length: u64);

    /// Move cursor left.
    /// - In Hex_Pane: retreat by one nibble (wraps to previous byte at row start → prev row end).
    /// - In ASCII_Pane: retreat by one byte (wraps at row start → prev row end).
    ///
    /// Addresses: Requirement 6 AC 6–8
    pub fn move_left(&mut self, layout: &HexLayout);

    /// Move cursor up by one row (byte_offset -= bytes_per_row).
    /// Clamped at offset 0.
    ///
    /// Addresses: Requirement 6 AC 6–7
    pub fn move_up(&mut self, layout: &HexLayout);

    /// Move cursor down by one row (byte_offset += bytes_per_row).
    /// Clamped at document_length - 1.
    ///
    /// Addresses: Requirement 6 AC 6–7
    pub fn move_down(&mut self, layout: &HexLayout, document_length: u64);

    /// Jump to a specific byte offset. Resets nibble to High.
    ///
    /// Addresses: Requirement 12
    pub fn goto_offset(&mut self, offset: u64, document_length: u64) -> bool;

    /// Set byte offset from a text-mode cursor position.
    /// Used when transitioning into hex mode.
    ///
    /// Addresses: Requirement 1 AC 9
    pub fn set_from_text_position(&mut self, byte_offset: u64);

    /// Get the byte offset for restoring to text-mode cursor.
    ///
    /// Addresses: Requirement 1 AC 10
    pub fn to_text_position(&self) -> u64;
}
```

### HexEditState

```rust
/// Manages byte-level editing in hex mode. Validates input, computes
/// new byte values, and delegates mutations to the edit-operations layer.
///
/// Addresses: Requirement 4, Requirement 7
#[derive(Debug)]
pub struct HexEditState {
    /// Whether editing is permitted (false in Browse/View mode).
    editing_enabled: bool,
    /// EBCDIC warning shown flag (per-session, per-file).
    ebcdic_warning_shown: bool,
}

impl HexEditState {
    pub fn new() -> Self;

    /// Process a hex digit keystroke in the Hex_Pane.
    /// Returns the byte modification to apply, or an error.
    ///
    /// Addresses: Requirement 4 AC 1–2, 4
    pub fn input_hex_digit(
        &self,
        digit: char,
        cursor: &HexCursor,
        current_byte: u8,
    ) -> Result<HexEditAction, HexError>;

    /// Process a character keystroke in the ASCII_Pane.
    /// Returns the byte modification to apply, or an error.
    ///
    /// Addresses: Requirement 4 AC 3
    pub fn input_ascii_char(
        &self,
        ch: char,
        cursor: &HexCursor,
    ) -> Result<HexEditAction, HexError>;

    /// Set editing enabled/disabled based on editor mode.
    ///
    /// Addresses: Requirement 4 AC 6
    pub fn set_editing_enabled(&mut self, enabled: bool);

    /// Check if EBCDIC warning should be shown.
    ///
    /// Addresses: Requirement 4 AC 9
    pub fn check_ebcdic_warning(&mut self, is_ebcdic: bool) -> Option<&str>;
}

/// The result of a validated hex edit input.
///
/// Addresses: Requirement 4 AC 1, 5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexEditAction {
    /// The byte offset being modified.
    pub byte_offset: u64,
    /// The new byte value to write.
    pub new_value: u8,
    /// The previous byte value (for undo).
    pub old_value: u8,
}
```

### HexSearchBridge (formerly HexSearchEngine)

```rust
/// Bridge between the find-and-replace engine and hex display.
/// Handles auto-activation of hex mode on hex search matches and
/// coordinates highlight rendering in the hex panes.
///
/// Addresses: Requirement 5
#[derive(Debug)]
pub struct HexSearchBridge {
    /// Currently highlighted match ranges (byte offsets).
    active_highlights: Vec<HexMatchHighlight>,
}

/// A highlighted byte range from a hex search match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexMatchHighlight {
    /// Start byte offset (inclusive).
    pub start: u64,
    /// End byte offset (exclusive).
    pub end: u64,
}

impl HexSearchBridge {
    pub fn new() -> Self;

    /// Called when a hex search match is found.
    /// Returns true if hex mode needs to be activated.
    ///
    /// Addresses: Requirement 5 AC 2
    pub fn on_hex_match_found(
        &mut self,
        match_start: u64,
        match_end: u64,
        hex_mode_active: bool,
    ) -> bool;

    /// Get current match highlights for rendering.
    ///
    /// Addresses: Requirement 5 AC 3
    pub fn active_highlights(&self) -> &[HexMatchHighlight];

    /// Clear all active highlights.
    pub fn clear_highlights(&mut self);

    /// Validate a hex search pattern string.
    /// Returns error if odd number of digits or invalid characters.
    ///
    /// Addresses: Requirement 5 AC 5
    pub fn validate_hex_pattern(pattern: &str) -> Result<Vec<u8>, HexError>;
}
```

### ModifiedByteTracker

```rust
/// Tracks which byte offsets have been modified since the last save.
/// Compares current buffer values against a snapshot taken at save time.
///
/// Addresses: Requirement 8
#[derive(Debug, Clone)]
pub struct ModifiedByteTracker {
    /// Set of byte offsets that differ from the last-saved state.
    modified_offsets: BTreeSet<u64>,
}

impl ModifiedByteTracker {
    pub fn new() -> Self;

    /// Mark a byte offset as modified.
    ///
    /// Addresses: Requirement 8 AC 1–2
    pub fn mark_modified(&mut self, offset: u64);

    /// Check if a byte is currently marked as modified.
    pub fn is_modified(&self, offset: u64) -> bool;

    /// Called when undo restores a byte to its saved value.
    /// Removes the modified indicator.
    ///
    /// Addresses: Requirement 8 AC 4
    pub fn mark_restored(&mut self, offset: u64);

    /// Called on document save: clears all modified indicators.
    ///
    /// Addresses: Requirement 8 AC 3
    pub fn on_save(&mut self);

    /// Get all modified offsets within a byte range (for rendering).
    pub fn modified_in_range(&self, start: u64, end: u64) -> Vec<u64>;

    /// Recalculate modification state for a byte after undo/redo.
    /// Compares current value against saved value.
    ///
    /// Addresses: Requirement 8 AC 5
    pub fn recalculate(&mut self, offset: u64, current_value: u8, saved_value: u8);
}
```

### HexViewModel

```rust
/// Pre-computed renderable data for a single hex row.
/// The shell layer reads this to render the hex grid without
/// performing any formatting logic.
///
/// Addresses: Requirement 2
#[derive(Debug, Clone)]
pub struct HexRow {
    /// The hex row index (0-based).
    pub row_index: u64,
    /// Formatted offset string (e.g., "0000001A").
    pub offset_text: String,
    /// Formatted hex digit pairs (e.g., "4A 5B 6C ...").
    /// Includes group separator spaces.
    pub hex_text: String,
    /// Formatted ASCII representation (e.g., "Hello...").
    pub ascii_text: String,
    /// Per-byte metadata for this row (for highlighting).
    pub byte_metadata: Vec<HexByteMetadata>,
}

/// Per-byte rendering metadata within a hex row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HexByteMetadata {
    /// Whether this byte has been modified since last save.
    pub is_modified: bool,
    /// Whether this byte is part of a search match highlight.
    pub is_search_match: bool,
    /// Whether this byte is under the cursor.
    pub is_cursor: bool,
    /// Whether this byte is part of a selection.
    pub is_selected: bool,
    /// Optional field boundary indicator (for FileForge integration).
    pub is_field_boundary: bool,
}

/// The complete view model for the visible hex viewport.
///
/// Addresses: Requirements 2, 9
#[derive(Debug, Clone)]
pub struct HexViewModel {
    /// Rows currently visible in the viewport.
    pub visible_rows: Vec<HexRow>,
    /// Total number of rows in the document.
    pub total_rows: u64,
    /// The first visible row index.
    pub top_row: u64,
    /// Current cursor state (for cursor rendering).
    pub cursor: HexCursorRenderState,
    /// Active pane indicator.
    pub active_pane: HexPane,
    /// Whether hex mode is active.
    pub mode: HexMode,
}

/// Cursor rendering state for the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexCursorRenderState {
    /// Row containing the cursor.
    pub row: u64,
    /// Byte index within the row (0-based).
    pub byte_in_row: usize,
    /// Nibble position (for Hex_Pane cursor shape).
    pub nibble: NibblePosition,
    /// Active pane.
    pub pane: HexPane,
}
```

### HexSessionState

```rust
/// Serialisable per-file hex mode session state.
/// Stored in the session history system for restore on reopen.
///
/// Addresses: Requirement 15
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct HexSessionState {
    /// Whether hex mode was active when the file was last closed.
    pub mode: HexMode,
    /// Bytes per row setting.
    pub bytes_per_row: u32,
    /// Cursor byte offset.
    pub cursor_offset: u64,
    /// Top visible row (viewport).
    pub viewport_top_row: u64,
    /// Which pane had focus.
    pub active_pane: HexPane,
}

impl HexSessionState {
    /// Create from current hex mode state.
    pub fn capture(
        mode: HexMode,
        layout: &HexLayout,
        cursor: &HexCursor,
        viewport_top_row: u64,
    ) -> Self;

    /// Restore hex mode state from a saved session.
    pub fn restore(&self, controller: &mut HexModeController) -> Result<(), HexError>;
}
```

### HexConfig

```rust
/// Typed access to hex display configuration settings.
/// Reads from the configuration system under `editor.hex.*`.
///
/// Addresses: Requirements 3, 10, 13
#[derive(Debug, Clone)]
pub struct HexConfig {
    /// Bytes per row (default: 16).
    pub bytes_per_row: BytesPerRow,
    /// Hex digit case (default: uppercase).
    pub digit_case: HexDigitCase,
    /// Auto-activate hex for binary files (default: prompt).
    pub auto_activate_binary: AutoActivateBinary,
}

impl Default for HexConfig {
    fn default() -> Self {
        Self {
            bytes_per_row: BytesPerRow::default(),
            digit_case: HexDigitCase::default(),
            auto_activate_binary: AutoActivateBinary::default(),
        }
    }
}

impl HexConfig {
    /// Load configuration from the configuration system.
    /// Keys: editor.hex.bytes_per_row, editor.hex.digit_case,
    ///       editor.hex.auto_activate_binary
    pub fn from_config(provider: &dyn ConfigProvider) -> Self;

    /// Validate and apply a bytes_per_row change.
    ///
    /// Addresses: Requirement 3 AC 4
    pub fn set_bytes_per_row(&mut self, value: u32) -> Result<(), HexError>;
}
```

### HexDumpExporter

```rust
/// Formats document content as a hex dump and exports it
/// to clipboard, file, or new editor tab.
///
/// Addresses: Requirement 11
#[derive(Debug)]
pub struct HexDumpExporter;

/// Target destination for a hex dump export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexDumpTarget {
    /// Open the dump in a new editor tab.
    NewTab,
    /// Copy the dump to the system clipboard.
    Clipboard,
    /// Write the dump to a file at the given path.
    File(String),
}

/// A byte range for partial hex dump export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexDumpRange {
    /// Start byte offset (inclusive).
    pub start: u64,
    /// End byte offset (exclusive).
    pub end: u64,
}

impl HexDumpExporter {
    /// Export a hex dump of the document (or a byte range).
    ///
    /// Addresses: Requirement 11 AC 1–7
    pub fn export(
        document_bytes: &dyn ByteReader,
        range: Option<HexDumpRange>,
        layout: &HexLayout,
        target: HexDumpTarget,
    ) -> Result<String, HexError>;

    /// Format a single row of hex dump output.
    pub fn format_row(
        offset: u64,
        bytes: &[u8],
        layout: &HexLayout,
    ) -> String;
}
```

### HexGotoHandler

```rust
/// Handles GOTO offset command parsing and navigation.
///
/// Addresses: Requirement 12
#[derive(Debug)]
pub struct HexGotoHandler;

/// Parsed offset value from a GOTO command argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedOffset {
    /// The byte offset value.
    pub value: u64,
}

impl HexGotoHandler {
    /// Parse an offset string in supported formats:
    /// - `X'1A4F'` (ISPF hex literal)
    /// - `0x1A4F` (C-style hex prefix)
    /// - `6735` (decimal, no prefix)
    ///
    /// Addresses: Requirement 12 AC 5
    pub fn parse_offset(input: &str) -> Result<ParsedOffset, HexError>;

    /// Execute the goto: validate bounds, position cursor, scroll viewport.
    ///
    /// Addresses: Requirement 12 AC 2–4
    pub fn execute(
        offset: ParsedOffset,
        document_length: u64,
        cursor: &mut HexCursor,
        viewport: &mut HexViewportAdapter,
        hex_mode: &mut HexModeController,
    ) -> Result<(), HexError>;
}
```

### HexViewportAdapter

```rust
/// Adapts the hex row model to the viewport-and-scrolling system.
/// Provides row-based scrolling, page calculations, and scroll clamping.
///
/// Addresses: Requirement 9
#[derive(Debug)]
pub struct HexViewportAdapter {
    /// Current top visible row.
    top_row: u64,
    /// Number of visible rows in the viewport.
    visible_rows: u64,
    /// Total rows in the document (computed from document length / bytes_per_row).
    total_rows: u64,
}

impl HexViewportAdapter {
    pub fn new(total_rows: u64, visible_rows: u64) -> Self;

    /// Recalculate total rows when document length or bytes_per_row changes.
    ///
    /// Addresses: Requirement 9 AC 7
    pub fn recalculate(&mut self, document_byte_length: u64, bytes_per_row: BytesPerRow);

    /// Scroll down by one page.
    ///
    /// Addresses: Requirement 9 AC 2
    pub fn page_down(&mut self);

    /// Scroll up by one page.
    ///
    /// Addresses: Requirement 9 AC 3
    pub fn page_up(&mut self);

    /// Scroll to ensure the given row is visible.
    ///
    /// Addresses: Requirement 9 AC 5
    pub fn ensure_row_visible(&mut self, row: u64);

    /// Set viewport size (on resize).
    pub fn set_visible_rows(&mut self, count: u64);

    /// Get scrollbar position as a fraction [0.0, 1.0].
    ///
    /// Addresses: Requirement 9 AC 4
    pub fn scrollbar_fraction(&self) -> f64;

    /// Set top row from scrollbar fraction.
    pub fn scroll_to_fraction(&mut self, fraction: f64);

    /// Current top row.
    pub fn top_row(&self) -> u64;

    /// Whether horizontal scrolling is needed.
    ///
    /// Addresses: Requirement 9 AC 6
    pub fn needs_horizontal_scroll(&self, layout: &HexLayout, viewport_width: usize) -> bool;
}
```

---

## Public API Surface

### HexModeController — Main Entry Point

```rust
/// Top-level orchestrator for hex display mode.
/// Owns all hex sub-components and coordinates state transitions.
///
/// Addresses: Requirements 1, 16
#[derive(Debug)]
pub struct HexModeController {
    mode: HexMode,
    layout: HexLayout,
    cursor: HexCursor,
    edit_state: HexEditState,
    search_bridge: HexSearchBridge,
    viewport: HexViewportAdapter,
    modified_tracker: ModifiedByteTracker,
    config: HexConfig,
    session: Option<HexSessionState>,
}

impl HexModeController {
    /// Create a new controller with default configuration.
    pub fn new(config: HexConfig) -> Self;

    // --- Mode Control ---

    /// Activate hex mode. Maps current text cursor to hex position.
    /// Returns error if already active (with status message).
    ///
    /// Addresses: Requirement 1 AC 1, 4, 9
    pub fn activate(
        &mut self,
        text_cursor_byte_offset: u64,
        document_byte_length: u64,
    ) -> Result<(), HexError>;

    /// Deactivate hex mode. Returns the byte offset for text cursor restore.
    /// Returns error if already inactive.
    ///
    /// Addresses: Requirement 1 AC 2, 5, 10
    pub fn deactivate(&mut self) -> Result<u64, HexError>;

    /// Toggle hex mode.
    ///
    /// Addresses: Requirement 1 AC 3
    pub fn toggle(
        &mut self,
        text_cursor_byte_offset: u64,
        document_byte_length: u64,
    ) -> Result<Option<u64>, HexError>;

    /// Whether hex mode is currently active.
    pub fn is_active(&self) -> bool;

    /// Current hex mode state.
    pub fn mode(&self) -> HexMode;

    // --- Cursor Access ---

    /// Get a reference to the hex cursor.
    pub fn cursor(&self) -> &HexCursor;

    /// Get mutable access to the hex cursor (for navigation).
    pub fn cursor_mut(&mut self) -> &mut HexCursor;

    // --- Layout Access ---

    /// Get the current layout configuration.
    pub fn layout(&self) -> &HexLayout;

    /// Change bytes per row. Preserves cursor byte offset.
    ///
    /// Addresses: Requirement 3 AC 3
    pub fn set_bytes_per_row(
        &mut self,
        bpr: BytesPerRow,
        document_byte_length: u64,
    ) -> Result<(), HexError>;

    // --- Editing ---

    /// Process a key input in hex mode.
    /// Delegates to HexEditState based on active pane.
    ///
    /// Addresses: Requirement 4
    pub fn handle_input(
        &mut self,
        input: HexInput,
        document: &dyn ByteReader,
    ) -> Result<Option<HexEditAction>, HexError>;

    // --- Viewport ---

    /// Get the viewport adapter.
    pub fn viewport(&self) -> &HexViewportAdapter;

    /// Get mutable viewport adapter (for scroll operations).
    pub fn viewport_mut(&mut self) -> &mut HexViewportAdapter;

    // --- View Model ---

    /// Build the view model for the currently visible rows.
    ///
    /// Addresses: Requirement 2
    pub fn build_view_model(
        &self,
        document: &dyn ByteReader,
    ) -> HexViewModel;

    // --- Modified Tracking ---

    /// Get the modified byte tracker.
    pub fn modified_tracker(&self) -> &ModifiedByteTracker;

    /// Notify that the document was saved.
    pub fn on_document_saved(&mut self);

    // --- Session ---

    /// Capture current state for session persistence.
    ///
    /// Addresses: Requirement 15
    pub fn capture_session(&self) -> HexSessionState;

    /// Restore from a saved session.
    pub fn restore_session(&mut self, state: &HexSessionState) -> Result<(), HexError>;

    // --- Search Bridge ---

    /// Get the search bridge for hex search integration.
    pub fn search_bridge(&self) -> &HexSearchBridge;

    /// Get mutable search bridge.
    pub fn search_bridge_mut(&mut self) -> &mut HexSearchBridge;
}
```

### ByteReader Trait

```rust
/// Trait for read-only byte-level access to the document buffer.
/// Implemented by ff-document-model's Document/TextBuffer.
/// Enables hex mode to read raw bytes without owning the document.
///
/// Addresses: Cross-cutting (GUI independence, buffer abstraction)
pub trait ByteReader: Send + Sync {
    /// Read a single byte at the given offset.
    /// Returns None if offset >= document length.
    fn byte_at(&self, offset: u64) -> Option<u8>;

    /// Read a contiguous range of bytes.
    /// Returns a Vec with actual bytes read (may be shorter at EOF).
    fn bytes_in_range(&self, start: u64, end: u64) -> Vec<u8>;

    /// Total byte length of the document.
    fn byte_length(&self) -> u64;
}
```

### HexInput

```rust
/// Input event types handled by the hex mode controller.
///
/// Addresses: Requirements 4, 6
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexInput {
    /// A hex digit typed in the Hex_Pane (0-9, A-F, a-f).
    HexDigit(char),
    /// A printable ASCII character typed in the ASCII_Pane.
    AsciiChar(char),
    /// Arrow key navigation.
    Arrow(ArrowDirection),
    /// Tab key: switch panes.
    SwitchPane,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
}

/// Arrow key direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}
```

---

## Error Types

```rust
/// Errors produced by the ff-hex crate.
/// All error messages follow the `[hex] operation: description` format.
///
/// Addresses: Cross-cutting Req 8
#[derive(Debug, thiserror::Error)]
pub enum HexError {
    /// HEX ON issued when hex mode is already active.
    #[error("[hex] activate: hex mode is already active")]
    AlreadyActive,

    /// HEX OFF issued when hex mode is already inactive.
    #[error("[hex] deactivate: hex mode is already off")]
    AlreadyInactive,

    /// Invalid hex digit typed in Hex_Pane.
    #[error("[hex] input: invalid hex digit '{0}'")]
    InvalidHexDigit(char),

    /// Non-printable character typed in ASCII_Pane.
    #[error("[hex] input: character 0x{0:02X} is not printable ASCII")]
    NonPrintableAscii(u8),

    /// Editing attempted in Browse or View mode.
    #[error("[hex] edit: cannot edit in {0} mode")]
    EditNotAllowed(String),

    /// Invalid bytes_per_row value.
    #[error("[hex] config: invalid bytes_per_row value {0} (must be 8, 16, 32, or 64)")]
    InvalidBytesPerRow(u32),

    /// GOTO offset exceeds document size.
    #[error("[hex] goto: offset 0x{offset:X} exceeds document size (0x{size:X} bytes)")]
    OffsetOutOfRange { offset: u64, size: u64 },

    /// Invalid offset format in GOTO command.
    #[error("[hex] goto: invalid offset format '{0}'")]
    InvalidOffsetFormat(String),

    /// Hex pattern has odd number of digits.
    #[error("[hex] search: hex pattern must contain an even number of digits")]
    OddHexPatternLength,

    /// Hex pattern contains invalid characters.
    #[error("[hex] search: invalid character '{0}' in hex pattern")]
    InvalidHexPatternChar(char),

    /// Hex dump export failed.
    #[error("[hex] dump: export failed: {0}")]
    DumpExportFailed(String),

    /// Session state restore failed.
    #[error("[hex] session: failed to restore hex state: {0}")]
    SessionRestoreFailed(String),
}
```

---

## Integration Points

### document-model (`ff-document-model`)

| Integration | Direction | Description |
|-------------|-----------|-------------|
| `ByteReader` trait | hex ← doc | Hex mode reads raw bytes from the document buffer for display and editing |
| `BytePosition` type | shared | Hex cursor and edits reference byte positions defined in document-model |
| Document length | hex ← doc | Required for total row calculation, offset validation, cursor clamping |
| Save-point events | hex ← doc | `ModifiedByteTracker.on_save()` triggered when document reports save completion |

### edit-operations (`ff-edit-operations`)

| Integration | Direction | Description |
|-------------|-----------|-------------|
| Overwrite byte | hex → edit | `HexEditAction` translated to a single-byte overwrite operation |
| Edit mode query | hex ← edit | `HexEditState.set_editing_enabled()` reads current Insert/Browse/View mode |
| Transaction boundary | hex → edit | Hex nibble pairs coalesced into single-byte edit transactions |

### undo-redo-transactions (`ff-undo-redo-transactions`)

| Integration | Direction | Description |
|-------------|-----------|-------------|
| Record edit | hex → undo | Each byte modification recorded as a reversible Edit_Operation |
| Coalescing | hex → undo | High+Low nibble edits coalesced into one undo entry (Req 7 AC 4) |
| Undo/Redo notify | hex ← undo | On undo/redo, `ModifiedByteTracker.recalculate()` updates byte indicators |
| Shared undo stack | design | Hex edits and text edits share the same undo stack (Req 7 AC 6) |

### command-framework (`ff-command`)

| Integration | Direction | Description |
|-------------|-----------|-------------|
| `HEX ON` command | hex → cmd | Registered as primary command; valid in Browse, Edit, View modes |
| `HEX OFF` command | hex → cmd | Registered; returns to text mode |
| `HEX` toggle command | hex → cmd | Registered; toggles based on current state |
| `HEX DUMP` command | hex → cmd | Registered with sub-variants (EDIT, CLIP, FILE) |
| `GOTO X'...'` command | hex → cmd | Registered; accepts hex and decimal offset formats |
| Command metadata | hex → cmd | Display names, descriptions, keyboard shortcuts for all hex commands |

### find-and-replace (`ff-find-and-replace`)

| Integration | Direction | Description |
|-------------|-----------|-------------|
| `FIND X'...'` result | hex ← find | When hex match found, `HexSearchBridge.on_hex_match_found()` called |
| Auto-activate | hex ← find | If match found while hex mode inactive, bridge returns true to trigger activation |
| Match highlighting | hex ← find | Match byte range stored in `HexSearchBridge.active_highlights` for rendering |
| Existing FIND in hex | hex ← find | Text FIND in hex mode highlights matching bytes in both panes (Req 16 AC 2) |
| Hex pattern validation | shared | `HexSearchBridge::validate_hex_pattern()` used by find engine for `X'...'` parsing |

### viewport-and-scrolling (`ff-viewport-scrolling`)

| Integration | Direction | Description |
|-------------|-----------|-------------|
| Row-based viewport | hex → vp | `HexViewportAdapter` provides hex-specific row count to viewport model |
| Scrollbar mapping | hex → vp | Total rows and top_row map to scrollbar fraction/position |
| Page size | hex → vp | Visible rows = viewport height; page up/down moves by visible row count |
| Caret visibility | hex ← vp | Viewport scrolls to keep cursor row visible (delegates to VP caret policy) |
| Resize handling | hex ← vp | On viewport resize, `HexViewportAdapter.set_visible_rows()` recalculates |

### configuration-system (`ff-config`)

| Integration | Direction | Description |
|-------------|-----------|-------------|
| `editor.hex.bytes_per_row` | hex ← cfg | Loaded at start, hot-reloaded on change |
| `editor.hex.digit_case` | hex ← cfg | Uppercase/lowercase preference |
| `editor.hex.auto_activate_binary` | hex ← cfg | Auto-activation behaviour for binary files |
| Hot-reload | hex ← cfg | Settings changes applied immediately when hex mode is active (Req 3 AC 3, Req 13 AC 6) |

### clipboard-operations (`ff-clipboard-operations`)

| Integration | Direction | Description |
|-------------|-----------|-------------|
| Hex dump to clipboard | hex → clip | `HEX DUMP CLIP` delegates formatted text to clipboard |
| Copy in hex mode | hex → clip | Selected byte range copied as hex text or raw bytes |
| Paste in hex mode | hex ← clip | Clipboard content parsed as hex or applied as ASCII bytes |

### startup-and-session (`ff-startup-and-session`)

| Integration | Direction | Description |
|-------------|-----------|-------------|
| Save session state | hex → session | `HexSessionState` serialised and stored in per-file session entry |
| Restore session | hex ← session | On file reopen, hex state restored from session history |

### fileforge-integration (`ff-fileforge-integration`)

| Integration | Direction | Description |
|-------------|-----------|-------------|
| Field boundaries | hex ← forge | Record_Structure field offsets/lengths provided for boundary rendering |
| COMP-3 annotations | hex ← forge | Packed decimal field byte ranges annotated with decoded values |
| Cell-to-byte mapping | hex ← forge | Grid cell selection maps to byte range highlight in hex pane |

### encoding-and-characters (`ff-encoding-and-characters`)

| Integration | Direction | Description |
|-------------|-----------|-------------|
| EBCDIC detection | hex ← enc | `HexEditState.check_ebcdic_warning()` triggered when encoding is EBCDIC |
| Binary detection | hex ← enc | Auto-activation uses encoding system's binary file detection signal |

---

## Correctness Properties

These properties are suitable for property-based testing with `proptest`.

### Property 1: Layout Row Count Consistency

**Statement:** For any document of byte length `L` and any valid `BytesPerRow` value `B`, the total row count equals `ceil(L / B)`. When `L == 0`, total rows equals 1 (empty document displays one empty row).

**Validates:** Requirement 2 AC 10, Requirement 9 AC 1

```
∀ L: u64, B ∈ {8, 16, 32, 64}:
  total_rows(L, B) == if L == 0 { 1 } else { (L + B - 1) / B }
```

### Property 2: Cursor Byte Offset Invariant

**Statement:** After any sequence of cursor movement operations (move_left, move_right, move_up, move_down), the cursor's byte_offset is always within `[0, document_length)` for non-empty documents, or equals 0 for empty documents.

**Validates:** Requirement 6 AC 6–8

```
∀ ops: Vec<CursorOp>, doc_len: u64:
  apply_ops(cursor, ops, layout, doc_len) ⟹
    cursor.byte_offset() < max(1, doc_len)
```

### Property 3: Pane Synchronisation

**Statement:** After switching panes (Tab), the cursor byte offset remains unchanged. The only state that changes is `active_pane`.

**Validates:** Requirement 6 AC 3–4

```
∀ cursor: HexCursor:
  let offset_before = cursor.byte_offset();
  cursor.switch_pane();
  cursor.byte_offset() == offset_before
  ∧ cursor.active_pane() != pane_before
```

### Property 4: Nibble Edit Produces Correct Byte

**Statement:** For any byte value `B` and any hex digit `D` (0–15), editing the high nibble produces `(D << 4) | (B & 0x0F)` and editing the low nibble produces `(B & 0xF0) | D`.

**Validates:** Requirement 4 AC 1–2

```
∀ B: u8, D: u8 where D < 16:
  edit_high_nibble(B, D) == (D << 4) | (B & 0x0F)
  edit_low_nibble(B, D) == (B & 0xF0) | D
```

### Property 5: Hex Row Formatting Round-Trip

**Statement:** For any byte sequence `bytes` of length ≤ `bytes_per_row`, formatting the hex text and then parsing it back produces the original bytes.

**Validates:** Requirement 2 AC 3, Requirement 11 AC 2

```
∀ bytes: Vec<u8> where bytes.len() <= 64:
  parse_hex_text(format_hex_text(bytes, layout)) == bytes
```

### Property 6: Offset Column Formatting

**Statement:** The offset column text for any row always represents the correct byte offset (`row_index * bytes_per_row`) and uses the configured digit case.

**Validates:** Requirement 2 AC 2, Requirement 13 AC 4

```
∀ row: u64, B ∈ {8, 16, 32, 64}:
  parse_hex(offset_text(row, B)) == row * B
```

### Property 7: ASCII Pane Printability Invariant

**Statement:** Every character in the ASCII pane text is either a printable ASCII character (0x20–0x7E) or the non-printable indicator (`.`). No other characters appear.

**Validates:** Requirement 2 AC 5

```
∀ bytes: Vec<u8>:
  format_ascii(bytes).chars().all(|c| c == '.' || (c >= ' ' && c <= '~'))
```

### Property 8: Modified Byte Tracker Consistency

**Statement:** After saving (clearing all indicators), then applying edits and undos, a byte is marked modified if and only if its current value differs from its saved value.

**Validates:** Requirement 8 AC 3–5

```
∀ edits: Vec<(u64, u8)>, undos: Vec<u64>, saved: HashMap<u64, u8>:
  tracker.on_save();
  apply_edits(tracker, edits);
  apply_undos(tracker, undos, saved);
  ∀ offset ∈ touched_offsets:
    tracker.is_modified(offset) == (current[offset] != saved[offset])
```

### Property 9: Goto Offset Bounds Validation

**Statement:** `HexGotoHandler::execute` succeeds if and only if the offset is less than the document length. If offset ≥ document length, it returns `HexError::OffsetOutOfRange`.

**Validates:** Requirement 12 AC 4

```
∀ offset: u64, doc_len: u64:
  (offset < doc_len) ⟹ execute(offset, doc_len).is_ok()
  (offset >= doc_len) ⟹ execute(offset, doc_len) == Err(OffsetOutOfRange)
```

### Property 10: Bytes Per Row Change Preserves Cursor Offset

**Statement:** When `bytes_per_row` is changed, the cursor's absolute byte offset is preserved; only the row/column mapping changes.

**Validates:** Requirement 3 AC 3

```
∀ cursor_offset: u64, old_bpr: BytesPerRow, new_bpr: BytesPerRow:
  let before = controller.cursor().byte_offset();
  controller.set_bytes_per_row(new_bpr, doc_len);
  controller.cursor().byte_offset() == before
```

### Property 11: Hex Pattern Validation

**Statement:** `validate_hex_pattern` accepts a string if and only if it has even length and all characters are valid hex digits (0–9, A–F, a–f). The returned byte vector has length equal to input length / 2.

**Validates:** Requirement 5 AC 5

```
∀ pattern: String:
  let valid = pattern.len() % 2 == 0 && pattern.chars().all(|c| c.is_ascii_hexdigit());
  valid ⟹ validate_hex_pattern(&pattern).unwrap().len() == pattern.len() / 2
  !valid ⟹ validate_hex_pattern(&pattern).is_err()
```

### Property 12: Session State Round-Trip

**Statement:** Capturing hex session state and then restoring it produces an equivalent controller state (mode, bytes_per_row, cursor offset, viewport top row, active pane).

**Validates:** Requirement 15 AC 3

```
∀ state: HexSessionState:
  let captured = controller.capture_session();
  let mut new_controller = HexModeController::new(config);
  new_controller.restore_session(&captured);
  new_controller.capture_session() == captured
```
