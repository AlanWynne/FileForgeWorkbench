//! Tests for `TextBuffer` (moved out of `text_buffer.rs` for the 400-line rule,
//! CR-CH-058 F1) plus the new record-API and piece-table-spine (save round-trip)
//! tests. The pre-CR-CH-058 tests are UNCHANGED and must stay byte-identical.

use super::*;
use crate::original_index::OriginalIndex;
use crate::record_format::{DelimiterTerminator, RecordFormat};
use crate::types::RecordNumber;

#[test]
fn empty_buffer_has_one_line() {
    let buf = TextBuffer::new();
    assert_eq!(buf.line_count(), 1);
    assert_eq!(buf.length(), 0);
}

#[test]
fn insert_text_without_line_endings() {
    let mut buf = TextBuffer::new();
    let result = buf.insert(BytePosition(0), b"hello").unwrap();
    assert_eq!(result.bytes_inserted, 5);
    assert_eq!(result.lines_added, 0);
    assert_eq!(buf.length(), 5);
    assert_eq!(buf.line_count(), 1);
}

#[test]
fn insert_text_with_newline() {
    let mut buf = TextBuffer::new();
    buf.insert(BytePosition(0), b"hello\nworld").unwrap();
    assert_eq!(buf.line_count(), 2);
    assert_eq!(buf.line_start(LineNumber(0)), BytePosition(0));
    assert_eq!(buf.line_start(LineNumber(1)), BytePosition(6));
}

#[test]
fn insert_text_with_crlf() {
    let mut buf = TextBuffer::new();
    buf.insert(BytePosition(0), b"hello\r\nworld").unwrap();
    assert_eq!(buf.line_count(), 2);
    assert_eq!(buf.line_start(LineNumber(1)), BytePosition(7));
}

#[test]
fn delete_removes_line_endings() {
    let mut buf = TextBuffer::new();
    buf.insert(BytePosition(0), b"a\nb\nc").unwrap();
    assert_eq!(buf.line_count(), 3);
    // Delete the first newline at position 1
    let result = buf.delete(BytePosition(1), 1).unwrap();
    assert_eq!(result.lines_removed, 1);
    assert_eq!(buf.line_count(), 2);
    let content = buf.get_range(BytePosition(0), buf.length()).unwrap();
    assert_eq!(content, b"ab\nc");
}

#[test]
fn read_only_blocks_insert() {
    let mut buf = TextBuffer::new();
    buf.set_read_only(true);
    let err = buf.insert(BytePosition(0), b"hello").unwrap_err();
    assert!(matches!(err, DocumentError::ReadOnly { .. }));
}

#[test]
fn read_only_blocks_delete() {
    let mut buf = TextBuffer::new();
    buf.insert(BytePosition(0), b"hello").unwrap();
    buf.set_read_only(true);
    let err = buf.delete(BytePosition(0), 1).unwrap_err();
    assert!(matches!(err, DocumentError::ReadOnly { .. }));
}

#[test]
fn position_out_of_range_on_insert() {
    let mut buf = TextBuffer::new();
    buf.insert(BytePosition(0), b"abc").unwrap();
    let err = buf.insert(BytePosition(10), b"x").unwrap_err();
    assert!(matches!(err, DocumentError::PositionOutOfRange { .. }));
}

#[test]
fn line_from_position_round_trip() {
    let mut buf = TextBuffer::new();
    buf.insert(BytePosition(0), b"abc\ndef\nghi").unwrap();
    for line_num in 0..buf.line_count() {
        let ln = LineNumber(line_num);
        let start = buf.line_start(ln);
        assert_eq!(buf.line_from_position(start), ln);
    }
}

#[test]
fn line_end_position() {
    let mut buf = TextBuffer::new();
    buf.insert(BytePosition(0), b"abc\ndef\nghi").unwrap();
    assert_eq!(buf.line_end(LineNumber(0)), BytePosition(3));
    assert_eq!(buf.line_end(LineNumber(1)), BytePosition(7));
    assert_eq!(buf.line_end(LineNumber(2)), BytePosition(11)); // end of doc
}

#[test]
fn line_end_mode_change_rebuilds_index() {
    let mut buf = TextBuffer::new();
    // NEL = 0xC2 0x85
    let content: Vec<u8> = [b"hello".as_slice(), &[0xC2, 0x85], b"world"].concat();
    buf.insert(BytePosition(0), &content).unwrap();
    assert_eq!(buf.line_count(), 1); // Default mode doesn't recognize NEL

    buf.set_line_end_mode(LineEndMode::Unicode);
    assert_eq!(buf.line_count(), 2); // Now NEL is recognized
}

#[test]
fn crlf_split_handling() {
    let mut buf = TextBuffer::new();
    // Start with CR followed by LF -> CRLF = 1 line ending
    buf.insert(BytePosition(0), b"a\r\nb").unwrap();
    assert_eq!(buf.line_count(), 2);
    // Insert between CR and LF
    buf.insert(BytePosition(2), b"x").unwrap();
    // Now it's "a\rx\nb" - CR and LF are separate = 2 line endings
    assert_eq!(buf.line_count(), 3);
}

#[test]
fn crlf_merge_handling() {
    let mut buf = TextBuffer::new();
    // "a\r" + "x" + "\nb" - CR and LF separated
    buf.insert(BytePosition(0), b"a\rx\nb").unwrap();
    assert_eq!(buf.line_count(), 3); // lines: "a\r", "x\n", "b"
                                     // Delete 'x' between CR and LF
    buf.delete(BytePosition(2), 1).unwrap();
    // Now "a\r\nb" - CRLF merged = 2 lines
    assert_eq!(buf.line_count(), 2);
}

#[test]
fn contains_line_end_check() {
    let buf = TextBuffer::new();
    assert!(buf.contains_line_end(b"hello\nworld"));
    assert!(!buf.contains_line_end(b"hello world"));
}

#[test]
fn split_view_matches_contiguous() {
    let mut buf = TextBuffer::new();
    buf.insert(BytePosition(0), b"hello world").unwrap();
    let split = buf.split_view();
    let mut combined: Vec<u8> = split.before_gap;
    combined.extend_from_slice(&split.after_gap);
    let contiguous = buf.contiguous_view().to_vec();
    assert_eq!(combined, contiguous);
}

// === CR-CH-058 F1: record API (a Delimited record IS a line) ================

/// Build a TextBuffer from bytes via the public insert path (whole-file
/// resident in F1), returning it ready for record-API assertions.
fn buf_from(bytes: &[u8]) -> TextBuffer {
    let mut buf = TextBuffer::new();
    if !bytes.is_empty() {
        buf.insert(BytePosition(0), bytes).unwrap();
    }
    buf
}

#[test]
fn default_record_format_is_native_delimited() {
    // Validates: Requirement 11.2
    let buf = TextBuffer::new();
    assert!(buf.record_format().is_delimited());
}

#[test]
fn total_records_equals_line_count_for_delimited() {
    // Validates: Requirement 11.3, 11.4
    for fixture in [
        b"a\nb\nc".as_slice(),
        b"a\r\nb\r\nc",
        b"a\rb\rc",
        b"only one line",
        b"trailing\n",
        b"",
    ] {
        let buf = buf_from(fixture);
        assert_eq!(
            buf.total_records(),
            buf.line_count(),
            "record count must equal line count for Delimited: {fixture:?}"
        );
    }
}

#[test]
fn record_start_equals_line_start_for_delimited() {
    // Validates: Requirement 11.3, 11.4
    let buf = buf_from(b"abc\ndef\nghi");
    for k in 0..buf.total_records() {
        assert_eq!(
            buf.record_start(RecordNumber(k)),
            buf.line_start(LineNumber(k)),
            "record_start must equal line_start for record {k}"
        );
    }
}

#[test]
fn record_from_position_equals_line_from_position_for_delimited() {
    // Validates: Requirement 11.3, 11.4
    let buf = buf_from(b"abc\r\ndef\r\nghi");
    for k in 0..buf.total_records() {
        let start = buf.record_start(RecordNumber(k));
        assert_eq!(
            buf.record_from_position(start).0,
            buf.line_from_position(start).0,
            "record_from_position must agree with line_from_position at record {k}"
        );
    }
}

#[test]
fn record_byte_length_includes_terminator_and_tiles_the_image() {
    // Validates: Requirement 11.4, 12.12 (byte lengths tile the whole image so
    // re-emit is byte-identical).
    for fixture in [
        b"a\nbb\nccc".as_slice(),
        b"a\r\nbb\r\nccc",
        b"x\ry\rz",
        b"no-newline",
        b"ends-with-nl\n",
    ] {
        let buf = buf_from(fixture);
        let mut sum = 0u64;
        for k in 0..buf.total_records() {
            sum += buf.record_byte_length(RecordNumber(k));
        }
        assert_eq!(
            sum,
            buf.length(),
            "record byte lengths must tile the whole image: {fixture:?}"
        );
    }
}

#[test]
fn set_line_end_mode_tracks_native_record_format() {
    // Validates: Requirement 11.2 (native RecordFormat generalises LineEndMode)
    let mut buf = TextBuffer::new();
    assert!(buf.record_format().is_delimited());
    buf.set_line_end_mode(LineEndMode::Unicode);
    assert!(
        buf.record_format().is_delimited(),
        "a Delimited document stays Delimited across a mode change"
    );
}

#[test]
fn non_delimited_format_is_not_flattened() {
    // Validates: Requirement 11.5 (Fixed is not flattened to delimiters)
    let mut buf = TextBuffer::new();
    buf.set_record_format(RecordFormat::Fixed { lrecl: 4 });
    assert!(!buf.record_format().is_delimited());
}

// === CR-CH-058 F1: piece-table spine + byte-identical SAVE (re-baseline) ====

#[test]
fn baseline_piece_list_is_single_original_span() {
    // Validates: Requirement 12.1
    let buf = buf_from(b"a\nb\nc\n");
    let pl = buf.baseline_piece_list();
    assert_eq!(pl.total_records(), buf.total_records());
    assert_eq!(
        pl.pieces().len(),
        1,
        "a freshly opened document is one Original piece"
    );
    assert!(!pl.has_dirty(), "a baseline piece list is not dirty");
}

#[test]
fn original_index_matches_record_api() {
    // Validates: Requirement 12.1 (the index built from the resident image
    // agrees with the record API).
    let buf = buf_from(b"alpha\nbeta\r\ngamma");
    let idx = buf.build_original_index();
    assert_eq!(idx.total_records(), buf.total_records());
    for k in 0..buf.total_records() {
        assert_eq!(
            idx.record_start(RecordNumber(k)),
            buf.record_start(RecordNumber(k)).0
        );
    }
}

#[test]
fn save_image_is_byte_identical_to_resident_content() {
    // Validates: Requirement 12.12 (non-negotiable safety rule) -- the SAVE
    // image of a Delimited document is byte-identical to its content, across
    // every terminator style, with and without an edit.
    for fixture in [
        b"line one\nline two\nline three\n".as_slice(),
        b"crlf one\r\ncrlf two\r\n",
        b"cr one\rcr two\r",
        b"no trailing newline",
        b"",
        b"mixed\na\r\nb\rc\n",
    ] {
        // No-edit round-trip: save image == original bytes.
        let buf = buf_from(fixture);
        assert_eq!(
            buf.save_image(),
            fixture.to_vec(),
            "no-edit SAVE image must be byte-identical to the original: {fixture:?}"
        );
    }
}

#[test]
fn save_image_byte_identical_after_edits() {
    // Validates: Requirement 12.12 -- after a representative edit sequence the
    // SAVE image equals the same edits applied to a naive byte model.
    let mut buf = buf_from(b"one\ntwo\nthree\n");
    let mut model: Vec<u8> = b"one\ntwo\nthree\n".to_vec();

    // Insert a line at the start.
    buf.insert(BytePosition(0), b"zero\n").unwrap();
    model.splice(0..0, b"zero\n".iter().copied());
    assert_eq!(buf.save_image(), model);

    // Delete "two\n" (now after "zero\none\n" = 9 bytes, length 4).
    buf.delete(BytePosition(9), 4).unwrap();
    model.splice(9..13, std::iter::empty());
    assert_eq!(buf.save_image(), model);

    // Overtype within a record: replace the 'r' of "three" (not strictly an
    // atomic op here; emulate by delete+insert) -- byte image must still match.
    let three_pos = model
        .windows(5)
        .position(|w| w == b"three")
        .expect("three present") as u64;
    buf.delete(BytePosition(three_pos), 5).unwrap();
    buf.insert(BytePosition(three_pos), b"THREE").unwrap();
    let idx = three_pos as usize;
    model.splice(idx..idx + 5, b"THREE".iter().copied());
    assert_eq!(buf.save_image(), model);
}

#[test]
fn rebaseline_preserves_content_and_resets_cleanly() {
    // Validates: Requirement 12.1, 12.12 (re-baseline keeps the saved content
    // and yields a single clean Original piece again).
    let mut buf = buf_from(b"a\nb\n");
    buf.insert(BytePosition(0), b"x\n").unwrap();
    let image = buf.save_image();
    buf.rebaseline();
    assert_eq!(buf.save_image(), image, "re-baseline must preserve content");
    let pl = buf.baseline_piece_list();
    assert_eq!(pl.pieces().len(), 1);
    assert!(!pl.has_dirty());
}

#[test]
fn terminator_bytes_are_exact() {
    // Validates: Requirement 11.1 (terminator re-emit bytes are correct)
    assert_eq!(DelimiterTerminator::Crlf.bytes(), b"\r\n");
    assert_eq!(DelimiterTerminator::Lf.bytes(), b"\n");
    assert_eq!(DelimiterTerminator::Cr.bytes(), b"\r");
}
