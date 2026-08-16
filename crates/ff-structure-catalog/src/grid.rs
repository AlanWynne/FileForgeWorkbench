//! Grid browse/edit mode — data models.
//!
//! Provides the data models for grid browse mode (read-only column-per-field view)
//! and grid edit mode (editable cells with validation and buffering).

use std::collections::HashMap;

use crate::field::{decode_packed_decimal, format_numeric_decimal, FieldDefinition, FieldType};

/// A decoded field value with display metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldValue {
    /// The display string for the field value.
    pub display: String,
    /// Whether the field has a validation warning.
    pub has_warning: bool,
    /// Optional warning message.
    pub warning: Option<String>,
}

/// A single row in the grid view.
#[derive(Debug, Clone, PartialEq)]
pub enum GridRow {
    /// Record matched the structure — fields extracted successfully.
    Matched {
        /// Decoded field values for each column.
        fields: Vec<FieldValue>,
    },
    /// Record did not match the structure — display raw text.
    Unmatched {
        /// Raw record text.
        raw_text: String,
    },
}

/// Grid browse state — read-only view of records.
#[derive(Debug)]
pub struct GridBrowseState {
    /// Rows of decoded record data.
    rows: Vec<GridRow>,
    /// Column definitions derived from the active record structure.
    column_names: Vec<String>,
    /// Current scroll position (top row index).
    scroll_position: usize,
}

impl GridBrowseState {
    /// Create a new grid browse state from record bytes and a record structure.
    pub fn new(column_names: Vec<String>) -> Self {
        Self {
            rows: Vec::new(),
            column_names,
            scroll_position: 0,
        }
    }

    /// Parse records from raw bytes using field definitions.
    ///
    /// Each record is `record_length` bytes. Fields that extend beyond the
    /// record boundary produce an unmatched row.
    pub fn load_records(&mut self, data: &[u8], record_length: usize, fields: &[FieldDefinition]) {
        self.rows.clear();
        if record_length == 0 {
            return;
        }

        for chunk in data.chunks(record_length) {
            let row = Self::parse_record(chunk, fields);
            self.rows.push(row);
        }
    }

    /// Parse a single record into a grid row.
    fn parse_record(record_bytes: &[u8], fields: &[FieldDefinition]) -> GridRow {
        let mut field_values = Vec::with_capacity(fields.len());

        for field in fields {
            let start = field.offset as usize;
            let end = start + field.length as usize;

            if end > record_bytes.len() {
                // Field extends beyond record — mark as unmatched
                let raw = String::from_utf8_lossy(record_bytes).to_string();
                return GridRow::Unmatched { raw_text: raw };
            }

            let bytes = &record_bytes[start..end];
            let value = decode_field_value(bytes, field);
            field_values.push(value);
        }

        GridRow::Matched {
            fields: field_values,
        }
    }

    /// Get all rows.
    pub fn rows(&self) -> &[GridRow] {
        &self.rows
    }

    /// Get column names.
    pub fn column_names(&self) -> &[String] {
        &self.column_names
    }

    /// Get scroll position.
    pub fn scroll_position(&self) -> usize {
        self.scroll_position
    }

    /// Set scroll position.
    pub fn set_scroll_position(&mut self, pos: usize) {
        self.scroll_position = pos.min(self.rows.len().saturating_sub(1));
    }

    /// Get the total number of rows (records).
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

/// Grid edit state — extends browse with an edit buffer.
#[derive(Debug)]
pub struct GridEditState {
    /// The underlying browse state.
    browse: GridBrowseState,
    /// Edit buffer: (row, col) → edited display value.
    edit_buffer: HashMap<(usize, usize), String>,
}

impl GridEditState {
    /// Create a new edit state from a browse state.
    pub fn from_browse(browse: GridBrowseState) -> Self {
        Self {
            browse,
            edit_buffer: HashMap::new(),
        }
    }

    /// Get the browse state.
    pub fn browse(&self) -> &GridBrowseState {
        &self.browse
    }

    /// Set a cell value in the edit buffer.
    pub fn set_cell(&mut self, row: usize, col: usize, value: String) {
        self.edit_buffer.insert((row, col), value);
    }

    /// Get a cell value from the edit buffer, or the original if not edited.
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&str> {
        if let Some(edited) = self.edit_buffer.get(&(row, col)) {
            return Some(edited.as_str());
        }

        // Fall back to original decoded value
        if let Some(GridRow::Matched { fields }) = self.browse.rows.get(row) {
            fields.get(col).map(|fv| fv.display.as_str())
        } else {
            None
        }
    }

    /// Check if a cell has been edited.
    pub fn is_cell_edited(&self, row: usize, col: usize) -> bool {
        self.edit_buffer.contains_key(&(row, col))
    }

    /// Check if a row has any edited cells.
    pub fn is_row_modified(&self, row: usize) -> bool {
        self.edit_buffer.keys().any(|(r, _)| *r == row)
    }

    /// Check if there are any unsaved edits.
    pub fn has_edits(&self) -> bool {
        !self.edit_buffer.is_empty()
    }

    /// Clear the edit buffer (discard all edits).
    pub fn clear_edits(&mut self) {
        self.edit_buffer.clear();
    }

    /// Get all edited cells as (row, col, value) tuples.
    pub fn edited_cells(&self) -> Vec<(usize, usize, &str)> {
        self.edit_buffer
            .iter()
            .map(|((r, c), v)| (*r, *c, v.as_str()))
            .collect()
    }

    /// Check if a row is unmatched (read-only in edit mode).
    pub fn is_row_unmatched(&self, row: usize) -> bool {
        matches!(self.browse.rows.get(row), Some(GridRow::Unmatched { .. }))
    }
}

/// Decode raw bytes into a display string using the field definition.
pub fn decode_field_value(bytes: &[u8], field: &FieldDefinition) -> FieldValue {
    match field.field_type {
        FieldType::Alphanumeric => {
            let display = String::from_utf8_lossy(bytes).trim_end().to_string();
            FieldValue {
                display,
                has_warning: false,
                warning: None,
            }
        }
        FieldType::Numeric => {
            let digits: String = bytes
                .iter()
                .map(|b| if b.is_ascii_digit() { *b as char } else { '0' })
                .collect();
            let display = format_numeric_decimal(&digits, field.decimals);
            FieldValue {
                display,
                has_warning: false,
                warning: None,
            }
        }
        FieldType::PackedDecimal => match decode_packed_decimal(bytes, field.decimals) {
            Ok(display) => FieldValue {
                display,
                has_warning: false,
                warning: None,
            },
            Err(byte_idx) => {
                let hex: String = bytes.iter().map(|b| format!("{b:02X}")).collect();
                FieldValue {
                    display: hex,
                    has_warning: true,
                    warning: Some(format!("invalid packed-decimal nibble at byte {byte_idx}")),
                }
            }
        },
        FieldType::Binary | FieldType::Hex => {
            let display: String = bytes.iter().map(|b| format!("{b:02X}")).collect();
            FieldValue {
                display,
                has_warning: false,
                warning: None,
            }
        }
    }
}

/// Encode a display value back to raw bytes for a given field.
///
/// Pads or truncates to match the field length.
pub fn encode_field_value(value: &str, field: &FieldDefinition) -> Vec<u8> {
    let length = field.length as usize;
    match field.field_type {
        FieldType::Alphanumeric => {
            let mut bytes: Vec<u8> = value.as_bytes().to_vec();
            // Right-pad with spaces
            bytes.resize(length, b' ');
            bytes.truncate(length);
            bytes
        }
        FieldType::Numeric => {
            // Left-pad with zeros
            let clean: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
            let padded = format!("{:0>width$}", clean, width = length);
            padded.as_bytes()[..length].to_vec()
        }
        FieldType::PackedDecimal => {
            // Simplified: encode digits with sign nibble
            encode_packed_decimal(value, length)
        }
        FieldType::Binary | FieldType::Hex => {
            // Parse hex string back to bytes
            let mut result = Vec::with_capacity(length);
            let hex_chars: Vec<char> = value.chars().filter(|c| c.is_ascii_hexdigit()).collect();
            for chunk in hex_chars.chunks(2) {
                if chunk.len() == 2 {
                    let s: String = chunk.iter().collect();
                    if let Ok(byte) = u8::from_str_radix(&s, 16) {
                        result.push(byte);
                    }
                }
            }
            result.resize(length, 0);
            result.truncate(length);
            result
        }
    }
}

/// Encode a decimal string value to COMP-3 packed-decimal bytes.
fn encode_packed_decimal(value: &str, length: usize) -> Vec<u8> {
    let is_negative = value.starts_with('-');
    let clean: String = value.chars().filter(|c| c.is_ascii_digit()).collect();

    // Maximum digits that can fit: (length * 2) - 1 (last nibble is sign)
    let max_digits = length * 2 - 1;
    let padded = format!("{:0>width$}", clean, width = max_digits);
    let digits: Vec<u8> = padded
        .chars()
        .take(max_digits)
        .map(|c| c as u8 - b'0')
        .collect();

    let sign = if is_negative { 0x0D } else { 0x0C };

    let mut result = Vec::with_capacity(length);
    let mut i = 0;
    while i < digits.len() {
        if i + 1 < digits.len() {
            result.push((digits[i] << 4) | digits[i + 1]);
            i += 2;
        } else {
            // Last digit + sign
            result.push((digits[i] << 4) | sign);
            i += 1;
        }
    }

    // If we have an even number of digits, the sign goes in the last byte
    #[allow(clippy::manual_is_multiple_of)]
    if digits.len() % 2 == 0 {
        // Replace last byte's low nibble with sign
        if let Some(last) = result.last_mut() {
            *last = (*last & 0xF0) | sign;
        }
    }

    result.resize(length, 0);
    result.truncate(length);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{FieldDefinition, FieldType};

    // Validates: Requirement 12.3 — alphanumeric field extraction
    #[test]
    fn decode_alphanumeric_field() {
        let field = FieldDefinition::new("NAME", 0, 10, FieldType::Alphanumeric);
        let bytes = b"JOHN      ";
        let value = decode_field_value(bytes, &field);
        assert_eq!(value.display, "JOHN");
        assert!(!value.has_warning);
    }

    // Validates: Requirement 12.3 — numeric field extraction
    #[test]
    fn decode_numeric_field_no_decimals() {
        let field = FieldDefinition::new("COUNT", 0, 5, FieldType::Numeric);
        let bytes = b"00123";
        let value = decode_field_value(bytes, &field);
        assert_eq!(value.display, "00123");
    }

    // Validates: Requirement 12.4 — numeric with implied decimal
    #[test]
    fn decode_numeric_field_with_decimals() {
        let mut field = FieldDefinition::new("AMOUNT", 0, 5, FieldType::Numeric);
        field.decimals = 2;
        let bytes = b"12345";
        let value = decode_field_value(bytes, &field);
        assert_eq!(value.display, "123.45");
    }

    // Validates: Requirement 12.4 — packed-decimal extraction
    #[test]
    fn decode_packed_decimal_field() {
        let mut field = FieldDefinition::new("BALANCE", 0, 3, FieldType::PackedDecimal);
        field.decimals = 2;
        let bytes = [0x12, 0x34, 0x5C]; // +12345 with 2 decimals = 123.45
        let value = decode_field_value(&bytes, &field);
        assert_eq!(value.display, "123.45");
        assert!(!value.has_warning);
    }

    // Validates: Requirement 12.5 — invalid packed-decimal shows hex with warning
    #[test]
    fn decode_invalid_packed_decimal_shows_hex_with_warning() {
        let field = FieldDefinition::new("BAD", 0, 2, FieldType::PackedDecimal);
        let bytes = [0xAB, 0xCD]; // Invalid: A is > 9
        let value = decode_field_value(&bytes, &field);
        assert!(value.has_warning);
        assert_eq!(value.display, "ABCD");
    }

    // Validates: Requirement 12.3 — binary field as hex
    #[test]
    fn decode_binary_field_as_hex() {
        let field = FieldDefinition::new("RAW", 0, 3, FieldType::Binary);
        let bytes = [0xDE, 0xAD, 0xBE];
        let value = decode_field_value(&bytes, &field);
        assert_eq!(value.display, "DEADBE");
    }

    // Validates: Requirement 12.5 — unmatched record when field exceeds length
    #[test]
    fn load_records_marks_short_record_as_unmatched() {
        let fields = vec![FieldDefinition::new("F1", 0, 20, FieldType::Alphanumeric)];
        let column_names = vec!["F1".to_string()];
        let mut state = GridBrowseState::new(column_names);

        let short_record = b"SHORT"; // Only 5 bytes, field needs 20
        state.load_records(short_record, 5, &fields);

        assert_eq!(state.row_count(), 1);
        assert!(matches!(state.rows()[0], GridRow::Unmatched { .. }));
    }

    // Validates: Requirement 12.1 — grid displays records as rows
    #[test]
    fn load_records_parses_multiple_records() {
        let fields = vec![
            FieldDefinition::new("F1", 0, 5, FieldType::Alphanumeric),
            FieldDefinition::new("F2", 5, 5, FieldType::Alphanumeric),
        ];
        let column_names = vec!["F1".to_string(), "F2".to_string()];
        let mut state = GridBrowseState::new(column_names);

        let data = b"AAAAA11111BBBBB22222";
        state.load_records(data, 10, &fields);

        assert_eq!(state.row_count(), 2);
        if let GridRow::Matched { fields: fv } = &state.rows()[0] {
            assert_eq!(fv[0].display, "AAAAA");
            assert_eq!(fv[1].display, "11111");
        } else {
            panic!("Expected matched row");
        }
    }

    // Validates: Requirement 13.1 — grid edit cell modification
    #[test]
    fn edit_state_set_and_get_cell() {
        let fields = vec![FieldDefinition::new("F1", 0, 5, FieldType::Alphanumeric)];
        let mut browse = GridBrowseState::new(vec!["F1".to_string()]);
        browse.load_records(b"HELLO", 5, &fields);

        let mut edit = GridEditState::from_browse(browse);
        assert_eq!(edit.get_cell(0, 0), Some("HELLO"));

        edit.set_cell(0, 0, "WORLD".to_string());
        assert_eq!(edit.get_cell(0, 0), Some("WORLD"));
        assert!(edit.is_cell_edited(0, 0));
        assert!(edit.is_row_modified(0));
        assert!(edit.has_edits());
    }

    // Validates: Requirement 13.5 — unmatched rows are read-only
    #[test]
    fn unmatched_rows_detected_as_read_only() {
        let fields = vec![FieldDefinition::new("F1", 0, 20, FieldType::Alphanumeric)];
        let mut browse = GridBrowseState::new(vec!["F1".to_string()]);
        browse.load_records(b"SHORT", 5, &fields);

        let edit = GridEditState::from_browse(browse);
        assert!(edit.is_row_unmatched(0));
    }

    // Validates: Requirement 13.9 — alphanumeric right-pad spaces
    #[test]
    fn encode_alphanumeric_right_pads_spaces() {
        let field = FieldDefinition::new("F1", 0, 10, FieldType::Alphanumeric);
        let encoded = encode_field_value("HI", &field);
        assert_eq!(encoded.len(), 10);
        assert_eq!(&encoded[..2], b"HI");
        assert_eq!(&encoded[2..], b"        ");
    }

    // Validates: Requirement 13.9 — numeric left-pad zeros
    #[test]
    fn encode_numeric_left_pads_zeros() {
        let field = FieldDefinition::new("F1", 0, 8, FieldType::Numeric);
        let encoded = encode_field_value("123", &field);
        assert_eq!(encoded.len(), 8);
        assert_eq!(&encoded, b"00000123");
    }

    // Validates: Requirement 13.10 — truncation preserves length
    #[test]
    fn encode_truncates_to_field_length() {
        let field = FieldDefinition::new("F1", 0, 5, FieldType::Alphanumeric);
        let encoded = encode_field_value("TOOLONGVALUE", &field);
        assert_eq!(encoded.len(), 5);
    }

    // Validates: Requirement 13.3 — clear edits
    #[test]
    fn clear_edits_removes_all_modifications() {
        let fields = vec![FieldDefinition::new("F1", 0, 5, FieldType::Alphanumeric)];
        let mut browse = GridBrowseState::new(vec!["F1".to_string()]);
        browse.load_records(b"HELLO", 5, &fields);

        let mut edit = GridEditState::from_browse(browse);
        edit.set_cell(0, 0, "NEW".to_string());
        assert!(edit.has_edits());

        edit.clear_edits();
        assert!(!edit.has_edits());
    }
}
