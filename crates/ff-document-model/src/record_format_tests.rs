//! Tests for `record_format`: RecordFormat variants and Delimited framing.

use super::*;
use crate::line_index::LineIndex;

#[test]
fn record_format_variants_exist_and_compare() {
    // Validates: Requirement 11.1
    let d = RecordFormat::Delimited {
        terminator: DelimiterTerminator::Crlf,
    };
    let f = RecordFormat::Fixed { lrecl: 80 };
    let v = RecordFormat::Variable {
        max_lrecl: 255,
        rdw: true,
    };
    assert!(d.is_delimited());
    assert!(!f.is_delimited());
    assert_ne!(d, f);
    assert_ne!(f, v);
    // Clone + Debug derive present.
    let d2 = d;
    assert_eq!(d, d2);
    let _ = format!("{d:?} {f:?} {v:?}");
}

#[test]
fn native_generalises_line_end_mode() {
    // Validates: Requirement 11.2
    let fmt = RecordFormat::native(LineEndMode::Default);
    assert!(fmt.is_delimited());
    assert_eq!(RecordFormat::default(), fmt);
}

#[test]
fn terminator_reemit_bytes() {
    // Validates: Requirement 12.12
    assert_eq!(reemit_terminator(DelimiterTerminator::Crlf), b"\r\n");
    assert_eq!(reemit_terminator(DelimiterTerminator::Lf), b"\n");
    assert_eq!(reemit_terminator(DelimiterTerminator::Cr), b"\r");
}

/// Compare framing boundaries against the existing LineIndex line_starts for
/// the same bytes: a Delimited record IS a line (AC 11.3, 11.4).
fn assert_frame_matches_line_index(bytes: &[u8]) {
    let boundaries = frame_records(
        bytes,
        RecordFormat::native(LineEndMode::Default),
        LineEndMode::Default,
    );
    let mut idx = LineIndex::new();
    idx.rebuild(bytes, LineEndMode::Default);
    // Same count.
    assert_eq!(
        boundaries.len() as u64,
        idx.line_count(),
        "record count must equal line count for {bytes:?}"
    );
    // Same starts.
    for (k, b) in boundaries.iter().enumerate() {
        let line_start = idx.line_start(crate::types::LineNumber(k as u64)).value();
        assert_eq!(b.start, line_start, "record {k} start mismatch");
    }
    // Lengths cover the slice with no gaps/overlaps and include terminators.
    let mut cursor = 0u64;
    for b in &boundaries {
        assert_eq!(b.start, cursor, "contiguous records");
        cursor += b.byte_length as u64;
    }
    assert_eq!(cursor, bytes.len() as u64, "records cover all bytes");
}

#[test]
fn delimited_framing_matches_line_index_lf() {
    // Validates: Requirement 11.3, 11.4
    assert_frame_matches_line_index(b"hello\nworld\n");
    assert_frame_matches_line_index(b"a\nb\nc");
    assert_frame_matches_line_index(b"");
    assert_frame_matches_line_index(b"single");
}

#[test]
fn delimited_framing_matches_line_index_crlf() {
    // Validates: Requirement 11.3, 11.4
    assert_frame_matches_line_index(b"line1\r\nline2\r\n");
    assert_frame_matches_line_index(b"a\r\nb\r\nc");
}

#[test]
fn delimited_framing_matches_line_index_cr() {
    // Validates: Requirement 11.3, 11.4
    assert_frame_matches_line_index(b"a\rb\rc\r");
    assert_frame_matches_line_index(b"only\r");
}

#[test]
fn delimited_framing_matches_line_index_mixed() {
    // Validates: Requirement 11.3, 11.4
    assert_frame_matches_line_index(b"a\r\nb\nc\rd");
    assert_frame_matches_line_index(b"x\ny\r\nz\r");
}

#[test]
fn delimited_record_byte_length_includes_terminator() {
    // Validates: Requirement 12.12
    let boundaries = frame_records(
        b"ab\r\ncd\n",
        RecordFormat::native(LineEndMode::Default),
        LineEndMode::Default,
    );
    assert_eq!(boundaries.len(), 3); // "ab\r\n", "cd\n", ""
    assert_eq!(boundaries[0].byte_length, 4); // ab + CRLF
    assert_eq!(boundaries[1].byte_length, 3); // cd + LF
    assert_eq!(boundaries[2].byte_length, 0); // trailing empty record
}

#[test]
fn fixed_framing_splits_arithmetically() {
    // Validates: Requirement 11.5 (Fixed not flattened to delimiters)
    let bytes = b"AAAABBBBCC"; // lrecl 4 -> "AAAA","BBBB","CC"
    let boundaries = frame_records(bytes, RecordFormat::Fixed { lrecl: 4 }, LineEndMode::Default);
    assert_eq!(boundaries.len(), 3);
    assert_eq!(boundaries[0], RecordBoundary { start: 0, byte_length: 4 });
    assert_eq!(boundaries[1], RecordBoundary { start: 4, byte_length: 4 });
    assert_eq!(boundaries[2], RecordBoundary { start: 8, byte_length: 2 });
}
