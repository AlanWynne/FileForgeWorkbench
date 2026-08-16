//! Integration tests for encoding detection.

use ff_encoding::*;

#[test]
fn detect_utf8_with_bom() {
    // Validates: Requirement 1.1, 2.1
    let content = b"\xEF\xBB\xBFHello, world!";
    let result = detect_encoding(content, None);
    assert_eq!(result.encoding.name, "utf-8");
    assert_eq!(result.confidence, DetectionConfidence::High);
    assert!(result.bom.is_some());
    assert_eq!(result.bom.unwrap().length, 3);
}

#[test]
fn detect_utf16le_with_bom() {
    // Validates: Requirement 1.1, 2.1
    let content = b"\xFF\xFEH\x00e\x00l\x00l\x00o\x00";
    let result = detect_encoding(content, None);
    assert_eq!(result.encoding.name, "utf-16le");
    assert_eq!(result.confidence, DetectionConfidence::High);
}

#[test]
fn detect_valid_utf8_multibyte() {
    // Validates: Requirement 1.4
    let content = "こんにちは世界".as_bytes(); // Japanese in UTF-8
    let result = detect_encoding(content, None);
    assert_eq!(result.encoding.name, "utf-8");
    assert_eq!(result.confidence, DetectionConfidence::Medium);
    assert!(result.bom.is_none());
}

#[test]
fn detect_pure_ascii() {
    // Validates: Requirement 1.4
    let content = b"Hello, world! This is a plain ASCII file.\n";
    let result = detect_encoding(content, None);
    assert_eq!(result.encoding.name, "utf-8"); // ASCII is valid UTF-8
}

#[test]
fn detect_empty_file() {
    let result = detect_encoding(&[], None);
    // Empty file falls through to fallback
    assert_eq!(result.encoding.name, "utf-8");
}
