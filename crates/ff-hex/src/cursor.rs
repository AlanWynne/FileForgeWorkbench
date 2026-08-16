//! Hex cursor and navigation.
//!
//! Tracks the current byte offset, active pane, nibble position,
//! and provides navigation logic with boundary clamping and wrapping.

use crate::layout::HexLayout;
use crate::types::{HexPane, NibblePosition};

/// The cursor state in hex display mode.
///
/// Tracks the current byte offset, active pane, nibble position,
/// and provides navigation with synchronisation between panes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexCursor {
    /// Absolute byte offset in the document (0-based).
    byte_offset: u64,
    /// Which pane has focus.
    active_pane: HexPane,
    /// When in Hex_Pane: which nibble is selected.
    nibble: NibblePosition,
}

impl Default for HexCursor {
    fn default() -> Self {
        Self::new()
    }
}

impl HexCursor {
    /// Create a new cursor at byte offset 0, Hex pane, high nibble.
    pub fn new() -> Self {
        Self {
            byte_offset: 0,
            active_pane: HexPane::Hex,
            nibble: NibblePosition::High,
        }
    }

    /// Create a cursor positioned at a specific byte offset.
    pub fn at_offset(offset: u64) -> Self {
        Self {
            byte_offset: offset,
            active_pane: HexPane::Hex,
            nibble: NibblePosition::High,
        }
    }

    /// Current byte offset.
    pub fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    /// Current active pane.
    pub fn active_pane(&self) -> HexPane {
        self.active_pane
    }

    /// Current nibble position (only meaningful when active_pane is Hex).
    pub fn nibble(&self) -> NibblePosition {
        self.nibble
    }

    /// Switch focus between Hex and ASCII panes (Tab key).
    /// Preserves byte offset.
    pub fn switch_pane(&mut self) {
        self.active_pane = match self.active_pane {
            HexPane::Hex => HexPane::Ascii,
            HexPane::Ascii => HexPane::Hex,
        };
        // Reset nibble to High when switching to hex pane
        if self.active_pane == HexPane::Hex {
            self.nibble = NibblePosition::High;
        }
    }

    /// Move cursor right.
    ///
    /// - In Hex_Pane: advance by one nibble (wraps to next byte/row).
    /// - In ASCII_Pane: advance by one byte (wraps at row end to next row).
    pub fn move_right(&mut self, _layout: &HexLayout, document_length: u64) {
        if document_length == 0 {
            return;
        }
        let max_offset = document_length - 1;

        match self.active_pane {
            HexPane::Hex => {
                let (next_nibble, advance_byte) = self.nibble.advance();
                if advance_byte && self.byte_offset < max_offset {
                    self.byte_offset += 1;
                }
                self.nibble = next_nibble;
                // Clamp: if we're at max_offset and tried to advance past low nibble
                if advance_byte && self.byte_offset >= max_offset {
                    self.byte_offset = max_offset;
                    self.nibble = NibblePosition::High;
                }
            }
            HexPane::Ascii => {
                if self.byte_offset < max_offset {
                    self.byte_offset += 1;
                }
            }
        }
    }

    /// Move cursor left.
    ///
    /// - In Hex_Pane: retreat by one nibble (wraps to previous byte/row).
    /// - In ASCII_Pane: retreat by one byte (wraps at row start to previous row).
    pub fn move_left(&mut self, _layout: &HexLayout) {
        match self.active_pane {
            HexPane::Hex => {
                let (prev_nibble, retreat_byte) = self.nibble.retreat();
                if retreat_byte {
                    if self.byte_offset > 0 {
                        self.byte_offset -= 1;
                    } else {
                        // At offset 0, high nibble, can't go further left
                        return;
                    }
                }
                self.nibble = prev_nibble;
            }
            HexPane::Ascii => {
                if self.byte_offset > 0 {
                    self.byte_offset -= 1;
                }
            }
        }
    }

    /// Move cursor up by one row (byte_offset -= bytes_per_row).
    /// Clamped at offset 0.
    pub fn move_up(&mut self, layout: &HexLayout) {
        let bpr = layout.bytes_per_row().as_u64();
        if self.byte_offset >= bpr {
            self.byte_offset -= bpr;
        } else {
            self.byte_offset = 0;
        }
    }

    /// Move cursor down by one row (byte_offset += bytes_per_row).
    /// Clamped at document_length - 1.
    pub fn move_down(&mut self, layout: &HexLayout, document_length: u64) {
        if document_length == 0 {
            return;
        }
        let bpr = layout.bytes_per_row().as_u64();
        let max_offset = document_length - 1;
        let new_offset = self.byte_offset + bpr;
        self.byte_offset = new_offset.min(max_offset);
    }

    /// Jump to a specific byte offset. Resets nibble to High.
    ///
    /// Returns `true` if the offset was valid and cursor was moved,
    /// `false` if the offset was out of range.
    pub fn goto_offset(&mut self, offset: u64, document_length: u64) -> bool {
        if document_length == 0 {
            return false;
        }
        if offset >= document_length {
            return false;
        }
        self.byte_offset = offset;
        self.nibble = NibblePosition::High;
        true
    }

    /// Set byte offset from a text-mode cursor position.
    /// Used when transitioning into hex mode.
    pub fn set_from_text_position(&mut self, byte_offset: u64) {
        self.byte_offset = byte_offset;
        self.nibble = NibblePosition::High;
        self.active_pane = HexPane::Hex;
    }

    /// Get the byte offset for restoring to text-mode cursor.
    pub fn to_text_position(&self) -> u64 {
        self.byte_offset
    }

    /// Set the nibble position directly (used after editing advances the nibble).
    pub fn set_nibble(&mut self, nibble: NibblePosition) {
        self.nibble = nibble;
    }

    /// Advance the cursor after a hex edit nibble input.
    ///
    /// After entering a nibble: High → Low (same byte), Low → High (next byte).
    pub fn advance_after_hex_edit(&mut self, document_length: u64) {
        let (next_nibble, advance_byte) = self.nibble.advance();
        self.nibble = next_nibble;
        if advance_byte && document_length > 0 {
            let max_offset = document_length - 1;
            if self.byte_offset < max_offset {
                self.byte_offset += 1;
            }
        }
    }

    /// Advance the cursor after an ASCII edit (move to next byte).
    pub fn advance_after_ascii_edit(&mut self, document_length: u64) {
        if document_length > 0 && self.byte_offset < document_length - 1 {
            self.byte_offset += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BytesPerRow;
    use pretty_assertions::assert_eq;

    fn layout_16() -> HexLayout {
        HexLayout::new(256, BytesPerRow::Sixteen)
    }

    // Validates: Requirement 6 AC 3-4
    #[test]
    fn switch_pane_toggles_between_hex_and_ascii_preserving_offset() {
        let mut cursor = HexCursor::at_offset(42);
        assert_eq!(cursor.active_pane(), HexPane::Hex);

        cursor.switch_pane();
        assert_eq!(cursor.active_pane(), HexPane::Ascii);
        assert_eq!(cursor.byte_offset(), 42);

        cursor.switch_pane();
        assert_eq!(cursor.active_pane(), HexPane::Hex);
        assert_eq!(cursor.byte_offset(), 42);
    }

    // Validates: Requirement 6 AC 6
    #[test]
    fn move_right_in_hex_pane_advances_nibble() {
        let layout = layout_16();
        let mut cursor = HexCursor::at_offset(0);
        assert_eq!(cursor.nibble(), NibblePosition::High);

        cursor.move_right(&layout, 256);
        assert_eq!(cursor.nibble(), NibblePosition::Low);
        assert_eq!(cursor.byte_offset(), 0);

        cursor.move_right(&layout, 256);
        assert_eq!(cursor.nibble(), NibblePosition::High);
        assert_eq!(cursor.byte_offset(), 1);
    }

    // Validates: Requirement 6 AC 7
    #[test]
    fn move_right_in_ascii_pane_advances_byte() {
        let layout = layout_16();
        let mut cursor = HexCursor::at_offset(0);
        cursor.switch_pane(); // ASCII pane

        cursor.move_right(&layout, 256);
        assert_eq!(cursor.byte_offset(), 1);

        cursor.move_right(&layout, 256);
        assert_eq!(cursor.byte_offset(), 2);
    }

    // Validates: Requirement 6 AC 6
    #[test]
    fn move_left_in_hex_pane_retreats_nibble() {
        let layout = layout_16();
        let mut cursor = HexCursor::at_offset(1);
        cursor.set_nibble(NibblePosition::Low);

        cursor.move_left(&layout);
        assert_eq!(cursor.nibble(), NibblePosition::High);
        assert_eq!(cursor.byte_offset(), 1);

        cursor.move_left(&layout);
        assert_eq!(cursor.nibble(), NibblePosition::Low);
        assert_eq!(cursor.byte_offset(), 0);
    }

    // Validates: Requirement 6 AC 7
    #[test]
    fn move_left_in_ascii_pane_retreats_byte() {
        let layout = layout_16();
        let mut cursor = HexCursor::at_offset(5);
        cursor.switch_pane();

        cursor.move_left(&layout);
        assert_eq!(cursor.byte_offset(), 4);
    }

    // Validates: Requirement 6 AC 6-7
    #[test]
    fn move_up_decrements_by_bytes_per_row() {
        let layout = layout_16();
        let mut cursor = HexCursor::at_offset(32);

        cursor.move_up(&layout);
        assert_eq!(cursor.byte_offset(), 16);

        cursor.move_up(&layout);
        assert_eq!(cursor.byte_offset(), 0);
    }

    // Validates: Requirement 6 AC 6-7
    #[test]
    fn move_up_clamps_at_zero() {
        let layout = layout_16();
        let mut cursor = HexCursor::at_offset(5);

        cursor.move_up(&layout);
        assert_eq!(cursor.byte_offset(), 0);
    }

    // Validates: Requirement 6 AC 6-7
    #[test]
    fn move_down_increments_by_bytes_per_row() {
        let layout = layout_16();
        let mut cursor = HexCursor::at_offset(0);

        cursor.move_down(&layout, 256);
        assert_eq!(cursor.byte_offset(), 16);
    }

    // Validates: Requirement 6 AC 6-7
    #[test]
    fn move_down_clamps_at_document_end() {
        let layout = layout_16();
        let mut cursor = HexCursor::at_offset(250);

        cursor.move_down(&layout, 256);
        assert_eq!(cursor.byte_offset(), 255); // clamped to last byte
    }

    // Validates: Requirement 6 AC 8
    #[test]
    fn move_right_at_document_end_clamps() {
        let layout = layout_16();
        let mut cursor = HexCursor::at_offset(255);
        cursor.switch_pane(); // ASCII

        cursor.move_right(&layout, 256);
        assert_eq!(cursor.byte_offset(), 255); // clamped
    }

    // Validates: Requirement 6 AC 8
    #[test]
    fn move_left_at_document_start_clamps() {
        let layout = layout_16();
        let mut cursor = HexCursor::at_offset(0);

        cursor.move_left(&layout);
        assert_eq!(cursor.byte_offset(), 0);
        assert_eq!(cursor.nibble(), NibblePosition::High);
    }

    // Validates: Requirement 12
    #[test]
    fn goto_offset_positions_cursor_and_resets_nibble() {
        let mut cursor = HexCursor::at_offset(0);
        cursor.set_nibble(NibblePosition::Low);

        assert!(cursor.goto_offset(100, 256));
        assert_eq!(cursor.byte_offset(), 100);
        assert_eq!(cursor.nibble(), NibblePosition::High);
    }

    // Validates: Requirement 12 AC 4
    #[test]
    fn goto_offset_rejects_out_of_range() {
        let mut cursor = HexCursor::at_offset(0);
        assert!(!cursor.goto_offset(256, 256)); // offset == length is out of range
        assert_eq!(cursor.byte_offset(), 0); // unchanged
    }

    // Validates: Requirement 1 AC 9
    #[test]
    fn set_from_text_position_maps_into_hex_mode() {
        let mut cursor = HexCursor::new();
        cursor.set_from_text_position(42);
        assert_eq!(cursor.byte_offset(), 42);
        assert_eq!(cursor.active_pane(), HexPane::Hex);
        assert_eq!(cursor.nibble(), NibblePosition::High);
    }

    // Validates: Requirement 1 AC 10
    #[test]
    fn to_text_position_returns_byte_offset() {
        let cursor = HexCursor::at_offset(99);
        assert_eq!(cursor.to_text_position(), 99);
    }

    // Validates: Requirement 4 AC 2
    #[test]
    fn advance_after_hex_edit_moves_nibble_correctly() {
        let mut cursor = HexCursor::at_offset(5);
        // High nibble → Low nibble (same byte)
        cursor.advance_after_hex_edit(256);
        assert_eq!(cursor.byte_offset(), 5);
        assert_eq!(cursor.nibble(), NibblePosition::Low);

        // Low nibble → High nibble (next byte)
        cursor.advance_after_hex_edit(256);
        assert_eq!(cursor.byte_offset(), 6);
        assert_eq!(cursor.nibble(), NibblePosition::High);
    }
}
