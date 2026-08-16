//! Hex view model.
//!
//! Pre-computed renderable data for the hex display. The shell layer
//! reads this to render the hex grid without performing formatting logic.

use crate::types::{HexMode, HexPane, NibblePosition};

/// Pre-computed renderable data for a single hex row.
///
/// The shell layer reads this to render the hex grid without
/// performing any formatting logic.
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
