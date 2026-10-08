//! Tests for the Document-level record model + SAVE image (CR-CH-058 F1,
//! Req 11 and Req 12.12). These assert the record API is exposed on the public
//! `Document` surface and agrees with the line API for Delimited documents, and
//! that the SAVE image is byte-identical (the non-negotiable safety rule).

use crate::document::Document;
use crate::types::{BytePosition, LineNumber, RecordNumber};

fn doc_from(bytes: &[u8]) -> Document {
    let mut d = Document::new();
    if !bytes.is_empty() {
        d.insert(BytePosition(0), bytes).unwrap();
    }
    d
}

#[test]
fn document_default_record_format_is_delimited() {
    // Validates: Requirement 11.2
    let d = Document::new();
    assert!(d.record_format().is_delimited());
}

#[test]
fn document_total_records_equals_line_count() {
    // Validates: Requirement 11.3, 11.4
    for fixture in [
        b"a\nb\nc".as_slice(),
        b"a\r\nb\r\nc",
        b"a\rb\rc",
        b"single",
        b"",
    ] {
        let d = doc_from(fixture);
        assert_eq!(d.total_records(), d.line_count(), "fixture {fixture:?}");
    }
}

#[test]
fn document_record_start_and_position_agree_with_line_api() {
    // Validates: Requirement 11.3, 11.4
    let d = doc_from(b"abc\ndef\r\nghi");
    for k in 0..d.total_records() {
        let rs = d.record_start(RecordNumber(k));
        assert_eq!(rs, d.line_start(LineNumber(k)));
        assert_eq!(
            d.record_from_position(rs).0,
            d.line_from_position(rs).0,
            "record/line position agreement at {k}"
        );
    }
}

#[test]
fn document_record_byte_lengths_tile_the_image() {
    // Validates: Requirement 11.4, 12.12
    let d = doc_from(b"a\nbb\r\nccc\rd");
    let sum: u64 = (0..d.total_records())
        .map(|k| d.record_byte_length(RecordNumber(k)))
        .sum();
    assert_eq!(sum, d.length());
}

#[test]
fn document_save_image_byte_identical_no_edit() {
    // Validates: Requirement 12.12 (non-negotiable safety rule)
    for fixture in [
        b"one\ntwo\nthree\n".as_slice(),
        b"crlf\r\nlines\r\n",
        b"cr\rlines\r",
        b"no newline",
        b"",
    ] {
        let d = doc_from(fixture);
        assert_eq!(d.save_image(), fixture.to_vec(), "fixture {fixture:?}");
    }
}

#[test]
fn document_save_image_byte_identical_after_edit_then_rebaseline() {
    // Validates: Requirement 12.12, 12.1
    let mut d = doc_from(b"alpha\nbeta\n");
    d.insert(BytePosition(0), b"zero\n").unwrap();
    let mut model: Vec<u8> = b"zero\nalpha\nbeta\n".to_vec();
    assert_eq!(d.save_image(), model);

    // Delete "alpha\n" (after "zero\n" = 5 bytes, length 6).
    d.delete(BytePosition(5), 6).unwrap();
    model.splice(5..11, std::iter::empty());
    assert_eq!(d.save_image(), model);

    // Re-baseline preserves content.
    let image = d.save_image();
    d.rebaseline();
    assert_eq!(d.save_image(), image);
}

#[test]
fn document_set_record_format_not_flattened() {
    // Validates: Requirement 11.5
    use crate::record_format::RecordFormat;
    let mut d = Document::new();
    d.set_record_format(RecordFormat::Fixed { lrecl: 80 });
    assert!(!d.record_format().is_delimited());
}
