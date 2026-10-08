//! Tests for the Immutable_Original_Index.

use super::*;
use crate::line_index::LineIndex;
use crate::types::LineNumber;
use std::mem::size_of;

#[test]
fn entry_is_sixteen_bytes() {
    // Validates: Requirement 12.1 (lean ~16-byte entry)
    assert_eq!(size_of::<OriginalIndexEntry>(), 16);
}

#[test]
fn from_bytes_one_entry_per_record() {
    // Validates: Requirement 11.4, 12.1
    let bytes = b"ab\ncd\nef\n";
    let idx = ResidentOriginalIndex::from_bytes(
        bytes,
        RecordFormat::native(LineEndMode::Default),
        LineEndMode::Default,
    );
    // "ab\n","cd\n","ef\n","" -> 4 records (matches line_index)
    let mut li = LineIndex::new();
    li.rebuild(bytes, LineEndMode::Default);
    assert_eq!(idx.total_records(), li.line_count());
    assert_eq!(idx.total_records(), 4);
    assert_eq!(idx.record_start(RecordNumber(0)), 0);
    assert_eq!(idx.record_byte_length(RecordNumber(0)), 3);
    assert_eq!(idx.record_start(RecordNumber(1)), 3);
    assert_eq!(idx.record_start(RecordNumber(2)), 6);
    assert_eq!(idx.record_start(RecordNumber(3)), 9); // trailing empty
    assert_eq!(idx.record_byte_length(RecordNumber(3)), 0);
}

#[test]
fn record_starts_match_line_index() {
    // Validates: Requirement 11.4
    for bytes in [
        b"hello\nworld\n".as_slice(),
        b"a\r\nb\r\nc",
        b"a\rb\rc\r",
        b"x\ny\r\nz\r",
        b"",
        b"single",
    ] {
        let idx = ResidentOriginalIndex::from_bytes(
            bytes,
            RecordFormat::native(LineEndMode::Default),
            LineEndMode::Default,
        );
        let mut li = LineIndex::new();
        li.rebuild(bytes, LineEndMode::Default);
        assert_eq!(idx.total_records(), li.line_count(), "count for {bytes:?}");
        for k in 0..idx.total_records() {
            assert_eq!(
                idx.record_start(RecordNumber(k)),
                li.line_start(LineNumber(k)).value(),
                "start of record {k} for {bytes:?}"
            );
        }
    }
}

#[test]
fn record_from_offset_round_trips() {
    // Validates: Requirement 12.1
    let bytes = b"ab\ncd\nef\n";
    let idx = ResidentOriginalIndex::from_bytes(
        bytes,
        RecordFormat::native(LineEndMode::Default),
        LineEndMode::Default,
    );
    for k in 0..idx.total_records() {
        let start = idx.record_start(RecordNumber(k));
        assert_eq!(
            idx.record_from_offset(start),
            Some(RecordNumber(k)),
            "round trip for record {k}"
        );
    }
    // A position inside a record resolves to that record.
    assert_eq!(idx.record_from_offset(1), Some(RecordNumber(0)));
    assert_eq!(idx.record_from_offset(4), Some(RecordNumber(1)));
    // Beyond the image -> None.
    assert_eq!(idx.record_from_offset(1000), None);
}

#[test]
fn empty_document_is_single_zero_length_record() {
    // Validates: Requirement 4.8
    let idx = ResidentOriginalIndex::empty();
    assert_eq!(idx.total_records(), 1);
    assert_eq!(idx.record_byte_length(RecordNumber(0)), 0);
    assert_eq!(idx.image_length(), 0);
    assert_eq!(idx.record_from_offset(0), Some(RecordNumber(0)));
}

#[test]
fn out_of_range_queries_are_graceful() {
    // Validates: Requirement 12.1
    let idx = ResidentOriginalIndex::from_bytes(
        b"a\n",
        RecordFormat::native(LineEndMode::Default),
        LineEndMode::Default,
    );
    assert_eq!(idx.entry(RecordNumber(99)), None);
    assert_eq!(idx.record_byte_length(RecordNumber(99)), 0);
    assert_eq!(idx.record_start(RecordNumber(99)), idx.image_length());
}

#[test]
fn index_is_usable_as_trait_object() {
    // Validates: Requirement 12.11 (behind the OriginalIndex trait)
    let boxed: Box<dyn OriginalIndex> = Box::new(ResidentOriginalIndex::from_bytes(
        b"a\nb\n",
        RecordFormat::native(LineEndMode::Default),
        LineEndMode::Default,
    ));
    assert_eq!(boxed.total_records(), 3);
    assert_eq!(boxed.record_start(RecordNumber(1)), 2);
}
