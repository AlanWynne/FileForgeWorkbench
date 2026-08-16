//! Hex view layout model.
//!
//! Computes the geometry for the hex display: pane widths, separators,
//! row structure, and total row count.

use crate::types::{ByteOffset, BytesPerRow, HexDigitCase};

/// Separator character placed between offset column, hex pane, and ASCII pane.
pub const SEPARATOR: &str = " │ ";
/// Width of the separator in characters.
pub const SEPARATOR_WIDTH: usize = 3;

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
    /// Hex digit case for display formatting.
    digit_case: HexDigitCase,
}

impl HexLayout {
    /// Create a new layout for a document of the given byte length.
    ///
    /// The offset column width auto-expands beyond 8 hex digits for
    /// files larger than 4 GB (0xFFFF_FFFF).
    pub fn new(document_byte_length: u64, bytes_per_row: BytesPerRow) -> Self {
        let offset_width = Self::compute_offset_width(document_byte_length);
        Self {
            bytes_per_row,
            offset_width,
            digit_case: HexDigitCase::default(),
        }
    }

    /// Compute the minimum offset column width needed for the document.
    fn compute_offset_width(document_byte_length: u64) -> u8 {
        if document_byte_length <= 0xFFFF_FFFF {
            8
        } else if document_byte_length <= 0x00FF_FFFF_FFFF {
            10
        } else if document_byte_length <= 0xFFFF_FFFF_FFFF {
            12
        } else if document_byte_length <= 0x00FF_FFFF_FFFF_FFFF {
            14
        } else {
            16
        }
    }

    /// Total number of hex rows needed to display the document.
    ///
    /// Formula: `ceil(document_byte_length / bytes_per_row)`, minimum 1.
    pub fn total_rows(&self, document_byte_length: u64) -> u64 {
        if document_byte_length == 0 {
            1
        } else {
            document_byte_length.div_ceil(self.bytes_per_row.as_u64())
        }
    }

    /// The byte offset of the first byte on the given row.
    pub fn row_start_offset(&self, row: u64) -> u64 {
        row * self.bytes_per_row.as_u64()
    }

    /// Which row contains the given byte offset.
    pub fn row_for_offset(&self, byte_offset: u64) -> u64 {
        byte_offset / self.bytes_per_row.as_u64()
    }

    /// Byte index within its row for a given absolute byte offset.
    pub fn byte_index_in_row(&self, byte_offset: u64) -> usize {
        (byte_offset % self.bytes_per_row.as_u64()) as usize
    }

    /// Column position of a byte within the Hex_Pane (character column).
    ///
    /// Each byte takes 2 hex digits + 1 space. Groups of 8 get an extra space.
    pub fn hex_column_for_byte(&self, byte_index_in_row: usize) -> usize {
        let base = byte_index_in_row * 3; // 2 digits + 1 space per byte
        let group_separators = if self.bytes_per_row.as_usize() >= 16 {
            byte_index_in_row / 8
        } else {
            0
        };
        base + group_separators
    }

    /// Column position of a byte within the ASCII_Pane.
    pub fn ascii_column_for_byte(&self, byte_index_in_row: usize) -> usize {
        byte_index_in_row
    }

    /// Total width in characters of one complete hex row
    /// (offset + separator + hex pane + separator + ASCII pane).
    pub fn total_row_width(&self) -> usize {
        self.offset_width as usize
            + SEPARATOR_WIDTH
            + self.hex_pane_width()
            + SEPARATOR_WIDTH
            + self.ascii_pane_width()
    }

    /// Width of the hex pane region in characters.
    ///
    /// Each byte = 2 hex digits + 1 space; plus additional group separator
    /// spaces after every 8 bytes when bytes_per_row >= 16.
    pub fn hex_pane_width(&self) -> usize {
        let bpr = self.bytes_per_row.as_usize();
        let base = bpr * 3 - 1; // 2 digits + space per byte, minus trailing space
        let group_separators = if bpr >= 16 { (bpr / 8) - 1 } else { 0 };
        base + group_separators
    }

    /// Width of the ASCII pane region in characters (= bytes_per_row).
    pub fn ascii_pane_width(&self) -> usize {
        self.bytes_per_row.as_usize()
    }

    /// Width of the offset column in characters.
    pub fn offset_width(&self) -> usize {
        self.offset_width as usize
    }

    /// Current bytes per row setting.
    pub fn bytes_per_row(&self) -> BytesPerRow {
        self.bytes_per_row
    }

    /// Current hex digit case.
    pub fn digit_case(&self) -> HexDigitCase {
        self.digit_case
    }

    /// Update bytes_per_row, recalculating geometry.
    pub fn set_bytes_per_row(&mut self, bpr: BytesPerRow, document_byte_length: u64) {
        self.bytes_per_row = bpr;
        self.offset_width = Self::compute_offset_width(document_byte_length);
    }

    /// Update digit case setting.
    pub fn set_digit_case(&mut self, case: HexDigitCase) {
        self.digit_case = case;
    }

    /// Whether the half-row separator is active (bytes_per_row >= 16).
    pub fn has_half_row_separator(&self) -> bool {
        self.bytes_per_row.as_usize() >= 16
    }

    /// Format the offset value for display.
    pub fn format_offset(&self, offset: u64) -> String {
        self.digit_case
            .format_offset(offset, self.offset_width as usize)
    }

    /// Format a slice of bytes as hex text for the hex pane.
    ///
    /// Inserts an additional space after every 8-byte group when
    /// bytes_per_row >= 16.
    pub fn format_hex_pane(&self, bytes: &[u8]) -> String {
        let bpr = self.bytes_per_row.as_usize();
        let mut result = String::with_capacity(self.hex_pane_width());

        for (i, &byte) in bytes.iter().enumerate() {
            if i > 0 {
                result.push(' ');
                // Extra space at 8-byte group boundary
                if bpr >= 16 && i % 8 == 0 {
                    result.push(' ');
                }
            }
            result.push_str(&self.digit_case.format_byte(byte));
        }

        // Pad remaining positions with spaces for incomplete rows
        if bytes.len() < bpr {
            let formatted_len = if bytes.is_empty() {
                0
            } else {
                self.hex_column_for_byte(bytes.len() - 1) + 2
            };
            let target_width = self.hex_pane_width();
            if formatted_len < target_width {
                // Add space separator before padding
                if !bytes.is_empty() {
                    result.push(' ');
                    if bpr >= 16 && bytes.len().is_multiple_of(8) {
                        result.push(' ');
                    }
                }
                let current_len = result.len();
                for _ in current_len..target_width {
                    result.push(' ');
                }
            }
        }

        result
    }

    /// Format a slice of bytes as ASCII text for the ASCII pane.
    ///
    /// Printable bytes (0x20–0x7E) are shown directly; non-printable bytes
    /// are shown as `.`.
    pub fn format_ascii_pane(&self, bytes: &[u8]) -> String {
        let bpr = self.bytes_per_row.as_usize();
        let mut result = String::with_capacity(bpr);

        for &byte in bytes {
            if (0x20..=0x7E).contains(&byte) {
                result.push(byte as char);
            } else {
                result.push('.');
            }
        }

        // Pad with spaces for incomplete rows
        while result.len() < bpr {
            result.push(' ');
        }

        result
    }

    /// Format a complete hex row from offset and bytes.
    pub fn format_row(&self, offset: u64, bytes: &[u8]) -> String {
        format!(
            "{}{}{}{}{}",
            self.format_offset(offset),
            SEPARATOR,
            self.format_hex_pane(bytes),
            SEPARATOR,
            self.format_ascii_pane(bytes),
        )
    }

    /// Map an absolute byte offset to its (row, byte_index_in_row) coordinates.
    pub fn offset_to_row_col(&self, byte_offset: u64) -> (u64, usize) {
        let row = self.row_for_offset(byte_offset);
        let col = self.byte_index_in_row(byte_offset);
        (row, col)
    }

    /// Map (row, byte_index_in_row) back to absolute byte offset.
    pub fn row_col_to_offset(&self, row: u64, col: usize) -> ByteOffset {
        ByteOffset::new(row * self.bytes_per_row.as_u64() + col as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 2 AC 2
    #[test]
    fn offset_column_width_is_8_for_small_files() {
        let layout = HexLayout::new(1000, BytesPerRow::Sixteen);
        assert_eq!(layout.offset_width(), 8);
    }

    // Validates: Requirement 2 AC 2
    #[test]
    fn offset_column_width_expands_for_large_files() {
        let layout = HexLayout::new(0x1_0000_0000, BytesPerRow::Sixteen);
        assert_eq!(layout.offset_width(), 10);
    }

    // Validates: Requirement 9 AC 1
    #[test]
    fn total_rows_calculates_correctly() {
        let layout = HexLayout::new(256, BytesPerRow::Sixteen);
        assert_eq!(layout.total_rows(256), 16); // 256 / 16 = 16

        let layout = HexLayout::new(257, BytesPerRow::Sixteen);
        assert_eq!(layout.total_rows(257), 17); // ceil(257 / 16) = 17
    }

    // Validates: Requirement 2 AC 10
    #[test]
    fn total_rows_for_empty_document_is_one() {
        let layout = HexLayout::new(0, BytesPerRow::Sixteen);
        assert_eq!(layout.total_rows(0), 1);
    }

    // Validates: Requirement 2 AC 1
    #[test]
    fn row_start_offset_increments_by_bytes_per_row() {
        let layout = HexLayout::new(100, BytesPerRow::Sixteen);
        assert_eq!(layout.row_start_offset(0), 0);
        assert_eq!(layout.row_start_offset(1), 16);
        assert_eq!(layout.row_start_offset(2), 32);
    }

    // Validates: Requirement 2 AC 3
    #[test]
    fn format_hex_pane_full_row_16_bytes() {
        let layout = HexLayout::new(100, BytesPerRow::Sixteen);
        let bytes: Vec<u8> = (0..16).collect();
        let hex = layout.format_hex_pane(&bytes);
        assert_eq!(hex, "00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F");
    }

    // Validates: Requirement 2 AC 4
    #[test]
    fn format_hex_pane_has_group_separator_at_8_byte_boundary() {
        let layout = HexLayout::new(100, BytesPerRow::Sixteen);
        let bytes: Vec<u8> = (0..16).collect();
        let hex = layout.format_hex_pane(&bytes);
        // There should be a double space between groups of 8
        assert!(hex.contains("07  08"));
    }

    // Validates: Requirement 2 AC 4
    #[test]
    fn format_hex_pane_no_group_separator_for_8_bytes_per_row() {
        let layout = HexLayout::new(100, BytesPerRow::Eight);
        let bytes: Vec<u8> = (0..8).collect();
        let hex = layout.format_hex_pane(&bytes);
        assert_eq!(hex, "00 01 02 03 04 05 06 07");
        // No double spaces
        assert!(!hex.contains("  "));
    }

    // Validates: Requirement 2 AC 5
    #[test]
    fn format_ascii_pane_printable_and_non_printable() {
        let layout = HexLayout::new(100, BytesPerRow::Sixteen);
        let bytes = vec![
            0x48, 0x65, 0x6C, 0x6C, 0x6F, // "Hello"
            0x00, 0x01, 0x1F, // non-printable
            0x7E, 0x7F, 0x80, 0xFF, // boundary values
            0x20, 0x41, 0x5A, 0x61, // space, A, Z, a
        ];
        let ascii = layout.format_ascii_pane(&bytes);
        // 0x7E='~' printable, 0x7F=non-printable, 0x80=non-printable, 0xFF=non-printable
        // 0x20=' ' printable, 0x41='A', 0x5A='Z', 0x61='a'
        assert_eq!(ascii, "Hello...~... AZa");
    }

    // Validates: Requirement 2 AC 5
    #[test]
    fn format_ascii_pane_shows_dot_for_non_printable() {
        let layout = HexLayout::new(100, BytesPerRow::Eight);
        let bytes = vec![0x00, 0x1F, 0x20, 0x7E, 0x7F, 0x80, 0xFF, 0x41];
        let ascii = layout.format_ascii_pane(&bytes);
        assert_eq!(ascii, ".. ~...A");
    }

    // Validates: Requirement 2 AC 7
    #[test]
    fn format_ascii_pane_pads_incomplete_row() {
        let layout = HexLayout::new(100, BytesPerRow::Sixteen);
        let bytes = vec![0x41, 0x42, 0x43]; // "ABC"
        let ascii = layout.format_ascii_pane(&bytes);
        assert_eq!(ascii.len(), 16); // Padded to full row width
        assert_eq!(&ascii[..3], "ABC");
    }

    // Validates: Requirement 2 AC 10
    #[test]
    fn format_row_for_empty_document() {
        let layout = HexLayout::new(0, BytesPerRow::Sixteen);
        let row = layout.format_row(0, &[]);
        assert!(row.starts_with("00000000"));
    }

    // Validates: Requirement 2 AC 2
    #[test]
    fn format_offset_zero_padded_8_digits() {
        let layout = HexLayout::new(100, BytesPerRow::Sixteen);
        assert_eq!(layout.format_offset(0), "00000000");
        assert_eq!(layout.format_offset(16), "00000010");
        assert_eq!(layout.format_offset(0xFF), "000000FF");
    }

    // Validates: Requirement 13 AC 4
    #[test]
    fn format_offset_respects_digit_case() {
        let mut layout = HexLayout::new(100, BytesPerRow::Sixteen);
        layout.set_digit_case(HexDigitCase::Lowercase);
        assert_eq!(layout.format_offset(0xABCD), "0000abcd");
    }

    // Validates: Requirement 2 AC 1, 3
    #[test]
    fn hex_pane_width_for_16_bytes() {
        let layout = HexLayout::new(100, BytesPerRow::Sixteen);
        // 16 bytes * 3 - 1 = 47, plus 1 group separator = 48
        assert_eq!(layout.hex_pane_width(), 48);
    }

    // Validates: Requirement 2 AC 1
    #[test]
    fn ascii_pane_width_equals_bytes_per_row() {
        let layout = HexLayout::new(100, BytesPerRow::Sixteen);
        assert_eq!(layout.ascii_pane_width(), 16);

        let layout = HexLayout::new(100, BytesPerRow::ThirtyTwo);
        assert_eq!(layout.ascii_pane_width(), 32);
    }

    // Validates: Requirement 3 AC 3, 3.6
    #[test]
    fn set_bytes_per_row_updates_layout() {
        let mut layout = HexLayout::new(100, BytesPerRow::Sixteen);
        assert_eq!(layout.bytes_per_row(), BytesPerRow::Sixteen);
        layout.set_bytes_per_row(BytesPerRow::ThirtyTwo, 100);
        assert_eq!(layout.bytes_per_row(), BytesPerRow::ThirtyTwo);
    }

    // Validates: Requirement 2 AC 1
    #[test]
    fn offset_to_row_col_mapping() {
        let layout = HexLayout::new(100, BytesPerRow::Sixteen);
        assert_eq!(layout.offset_to_row_col(0), (0, 0));
        assert_eq!(layout.offset_to_row_col(15), (0, 15));
        assert_eq!(layout.offset_to_row_col(16), (1, 0));
        assert_eq!(layout.offset_to_row_col(33), (2, 1));
    }

    // Validates: Requirement 2 AC 1
    #[test]
    fn row_col_to_offset_mapping() {
        let layout = HexLayout::new(100, BytesPerRow::Sixteen);
        assert_eq!(layout.row_col_to_offset(0, 0), ByteOffset::new(0));
        assert_eq!(layout.row_col_to_offset(1, 0), ByteOffset::new(16));
        assert_eq!(layout.row_col_to_offset(2, 5), ByteOffset::new(37));
    }
}
