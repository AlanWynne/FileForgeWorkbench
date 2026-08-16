//! Fixed-length record reader and LRECL auto-detection.
//!
//! Handles RECFM F/FB/FBA files where every record has the same byte length.
//! Record N starts at byte offset N × LRECL — O(1) direct position calculation.

use crate::byte_index::ByteOffsetIndex;
use crate::error::FileForgeError;

/// Result of LRECL auto-detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LreclDetection {
    /// All sampled lines have the same byte length.
    Uniform(usize),
    /// Line lengths vary — use variable-length mode.
    Variable,
}

/// Auto-detects the logical record length by sampling lines.
///
/// Examines up to `sample_size` lines and checks for uniform byte length.
/// Lines are split by newline characters (\n or \r\n).
///
/// # Returns
///
/// - `LreclDetection::Uniform(len)` if all sampled lines have the same length.
/// - `LreclDetection::Variable` if line lengths differ.
pub fn detect_lrecl(data: &[u8], sample_size: usize) -> LreclDetection {
    if data.is_empty() {
        return LreclDetection::Variable;
    }

    let mut lines_checked = 0;
    let mut uniform_length: Option<usize> = None;
    let mut pos = 0;

    while pos < data.len() && lines_checked < sample_size {
        // Find end of line
        let line_start = pos;
        while pos < data.len() && data[pos] != b'\n' {
            pos += 1;
        }

        let line_end = if pos > line_start && pos > 0 && data[pos - 1] == b'\r' {
            pos - 1
        } else {
            pos
        };

        let line_length = line_end - line_start;

        // Skip truly empty lines (consecutive newlines)
        if line_length > 0 {
            match uniform_length {
                None => uniform_length = Some(line_length),
                Some(expected) => {
                    if line_length != expected {
                        return LreclDetection::Variable;
                    }
                }
            }
            lines_checked += 1;
        }

        // Advance past newline
        if pos < data.len() {
            pos += 1;
        }
    }

    match uniform_length {
        Some(len) => LreclDetection::Uniform(len),
        None => LreclDetection::Variable,
    }
}

/// Builds a byte-offset index for a variable-length text file.
///
/// Scans for newlines and records the start offset of each line.
pub fn build_variable_index(data: &[u8]) -> ByteOffsetIndex {
    if data.is_empty() {
        return ByteOffsetIndex::for_variable(vec![]);
    }

    let mut offsets = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        offsets.push(pos as u64);

        // Find end of line
        while pos < data.len() && data[pos] != b'\n' {
            pos += 1;
        }

        // Advance past newline
        if pos < data.len() {
            pos += 1;
        }
    }

    ByteOffsetIndex::for_variable(offsets)
}

/// Builds a fixed-width index for a file with known LRECL.
///
/// # Errors
///
/// Returns `FileForgeError::LreclDetectionFailed` if `lrecl` is 0.
pub fn build_fixed_index(file_size: u64, lrecl: usize) -> Result<ByteOffsetIndex, FileForgeError> {
    if lrecl == 0 {
        return Err(FileForgeError::LreclDetectionFailed { sample_size: 0 });
    }
    Ok(ByteOffsetIndex::for_fixed_width(file_size, lrecl))
}

/// Reads a single fixed-width record from a data buffer.
///
/// # Errors
///
/// Returns `FileForgeError::RecordOutOfRange` if the record index is out of bounds.
/// Returns `FileForgeError::UnexpectedEof` if the record extends past the data.
pub fn read_fb_record(
    data: &[u8],
    record_index: usize,
    lrecl: usize,
) -> Result<&[u8], FileForgeError> {
    let offset = record_index * lrecl;
    let record_count = data.len() / lrecl;

    if record_index >= record_count {
        return Err(FileForgeError::RecordOutOfRange {
            requested: record_index,
            total: record_count,
        });
    }

    if offset + lrecl > data.len() {
        return Err(FileForgeError::UnexpectedEof {
            byte_offset: offset as u64,
            expected: lrecl,
        });
    }

    Ok(&data[offset..offset + lrecl])
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.9
    #[test]
    fn detect_lrecl_uniform_lines() {
        let data = b"AAAAAAAA\nBBBBBBBB\nCCCCCCCC\n";
        let result = detect_lrecl(data, 100);
        assert_eq!(result, LreclDetection::Uniform(8));
    }

    // Validates: Requirement 2.11
    #[test]
    fn detect_lrecl_variable_lines() {
        let data = b"SHORT\nMUCH LONGER LINE\nMED\n";
        let result = detect_lrecl(data, 100);
        assert_eq!(result, LreclDetection::Variable);
    }

    #[test]
    fn detect_lrecl_empty_data() {
        let result = detect_lrecl(b"", 100);
        assert_eq!(result, LreclDetection::Variable);
    }

    #[test]
    fn detect_lrecl_single_line() {
        let data = b"ONE LINE ONLY\n";
        let result = detect_lrecl(data, 100);
        assert_eq!(result, LreclDetection::Uniform(13));
    }

    #[test]
    fn detect_lrecl_samples_only_first_n_lines() {
        // First 3 lines uniform, 4th different — but sample_size = 3
        let data = b"AAAA\nBBBB\nCCCC\nDDDDDDD\n";
        let result = detect_lrecl(data, 3);
        assert_eq!(result, LreclDetection::Uniform(4));
    }

    #[test]
    fn detect_lrecl_handles_crlf() {
        let data = b"AAAA\r\nBBBB\r\nCCCC\r\n";
        let result = detect_lrecl(data, 100);
        assert_eq!(result, LreclDetection::Uniform(4));
    }

    // Validates: Requirement 2.2
    #[test]
    fn read_fb_record_direct_seek() {
        let data = b"RECORD01RECORD02RECORD03";
        let record = read_fb_record(data, 1, 8).unwrap();
        assert_eq!(record, b"RECORD02");
    }

    #[test]
    fn read_fb_record_first() {
        let data = b"FIRSTSECTHIRD";
        let record = read_fb_record(data, 0, 5).unwrap();
        assert_eq!(record, b"FIRST");
    }

    #[test]
    fn read_fb_record_out_of_range() {
        let data = b"RECORD01RECORD02";
        let result = read_fb_record(data, 5, 8);
        assert!(matches!(
            result,
            Err(FileForgeError::RecordOutOfRange { .. })
        ));
    }

    #[test]
    fn build_fixed_index_correct_count() {
        let index = build_fixed_index(8000, 80).unwrap();
        assert_eq!(index.record_count(), 100);
        assert_eq!(index.offset_of(0), Some(0));
        assert_eq!(index.offset_of(99), Some(7920));
    }

    #[test]
    fn build_variable_index_from_text() {
        let data = b"Line1\nLine22\nLine333\n";
        let index = build_variable_index(data);
        assert_eq!(index.record_count(), 3);
        assert_eq!(index.offset_of(0), Some(0));
        assert_eq!(index.offset_of(1), Some(6));
        assert_eq!(index.offset_of(2), Some(13));
    }
}
