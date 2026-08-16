//! Hex dump export.
//!
//! Formats document content as a hex dump and exports it to clipboard,
//! file, or new editor tab.

use crate::layout::HexLayout;

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

/// Formats document content as a hex dump.
///
/// Produces a three-column text output matching the hex view layout:
/// offset, hex bytes, and ASCII representation — one row per
/// bytes_per_row bytes.
#[derive(Debug)]
pub struct HexDumpExporter;

impl HexDumpExporter {
    /// Export a hex dump of the given bytes (or a range).
    ///
    /// Returns the formatted hex dump as a string.
    pub fn export(data: &[u8], range: Option<HexDumpRange>, layout: &HexLayout) -> String {
        let (slice, start_offset) = match range {
            Some(r) => {
                let start = r.start as usize;
                let end = (r.end as usize).min(data.len());
                if start >= data.len() {
                    return String::new();
                }
                (&data[start..end], r.start)
            }
            None => (data, 0u64),
        };

        if slice.is_empty() {
            return layout.format_row(start_offset, &[]);
        }

        let bpr = layout.bytes_per_row().as_usize();
        let mut lines: Vec<String> = Vec::new();

        for (i, chunk) in slice.chunks(bpr).enumerate() {
            let offset = start_offset + (i * bpr) as u64;
            lines.push(layout.format_row(offset, chunk));
        }

        lines.join("\n")
    }

    /// Format a single row of hex dump output.
    pub fn format_row(offset: u64, bytes: &[u8], layout: &HexLayout) -> String {
        layout.format_row(offset, bytes)
    }

    /// Parse a hex dump back into bytes (for round-trip verification).
    ///
    /// Extracts the hex pane content from each line and decodes it.
    pub fn parse_hex_dump(dump: &str, _layout: &HexLayout) -> Vec<u8> {
        let mut result = Vec::new();

        for line in dump.lines() {
            // Find the hex content between the separators " │ "
            // The format is: offset │ hex_pane │ ascii_pane
            let parts: Vec<&str> = line.splitn(3, " │ ").collect();
            if parts.len() < 2 {
                continue;
            }
            let hex_part = parts[1];

            // Parse hex digits, skipping spaces
            let mut chars = hex_part.chars().peekable();
            while let Some(&ch) = chars.peek() {
                if ch == ' ' {
                    chars.next();
                    continue;
                }
                if ch.is_ascii_hexdigit() {
                    let high = chars.next().unwrap();
                    if let Some(&low) = chars.peek() {
                        if low.is_ascii_hexdigit() {
                            chars.next();
                            let byte = (hex_char_value(high) << 4) | hex_char_value(low);
                            result.push(byte);
                        }
                    }
                } else {
                    chars.next();
                }
            }
        }

        result
    }
}

/// Convert a hex character to its nibble value.
fn hex_char_value(ch: char) -> u8 {
    match ch {
        '0'..='9' => ch as u8 - b'0',
        'A'..='F' => ch as u8 - b'A' + 10,
        'a'..='f' => ch as u8 - b'a' + 10,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BytesPerRow, HexDigitCase};
    use pretty_assertions::assert_eq;

    fn layout_16() -> HexLayout {
        HexLayout::new(256, BytesPerRow::Sixteen)
    }

    // Validates: Requirement 11 AC 2
    #[test]
    fn export_full_document_produces_correct_format() {
        let layout = layout_16();
        let data: Vec<u8> = (0..32).collect();
        let dump = HexDumpExporter::export(&data, None, &layout);

        let lines: Vec<&str> = dump.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("00000000"));
        assert!(lines[1].starts_with("00000010"));
    }

    // Validates: Requirement 11 AC 4
    #[test]
    fn export_byte_range() {
        let layout = layout_16();
        let data: Vec<u8> = (0..64).collect();
        let range = HexDumpRange { start: 16, end: 48 };
        let dump = HexDumpExporter::export(&data, Some(range), &layout);

        let lines: Vec<&str> = dump.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("00000010")); // offset 16
        assert!(lines[1].starts_with("00000020")); // offset 32
    }

    // Validates: Requirement 11 AC 6
    #[test]
    fn export_respects_digit_case() {
        let mut layout = HexLayout::new(256, BytesPerRow::Sixteen);
        layout.set_digit_case(HexDigitCase::Lowercase);
        let data = vec![0xAB, 0xCD, 0xEF];
        let dump = HexDumpExporter::export(&data, None, &layout);
        assert!(dump.contains("ab cd ef"));
    }

    // Validates: Requirement 11 AC 2
    #[test]
    fn export_round_trip_preserves_content() {
        let layout = layout_16();
        let data: Vec<u8> = (0..48).collect();
        let dump = HexDumpExporter::export(&data, None, &layout);
        let parsed = HexDumpExporter::parse_hex_dump(&dump, &layout);
        assert_eq!(parsed, data);
    }

    // Validates: Requirement 11 AC 2, 11.6
    #[test]
    fn export_with_8_bytes_per_row() {
        let layout = HexLayout::new(256, BytesPerRow::Eight);
        let data: Vec<u8> = (0..16).collect();
        let dump = HexDumpExporter::export(&data, None, &layout);

        let lines: Vec<&str> = dump.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    // Validates: Requirement 11 AC 3
    #[test]
    fn export_empty_data() {
        let layout = layout_16();
        let dump = HexDumpExporter::export(&[], None, &layout);
        assert!(dump.starts_with("00000000"));
    }
}
