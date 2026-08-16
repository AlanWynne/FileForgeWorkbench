//! Integration tests for grapheme cluster boundary detection.

use ff_encoding::*;

#[test]
fn grapheme_iteration_ascii() {
    // Validates: Requirement 9.1
    let text = "Hello";
    let graphemes: Vec<&str> = GraphemeIterator::new(text, GraphemeMode::Strict).collect();
    assert_eq!(graphemes, vec!["H", "e", "l", "l", "o"]);
}

#[test]
fn grapheme_iteration_combining_marks() {
    // Validates: Requirement 9.5
    // 'e' + combining acute (U+0301) + 'x'
    let text = "e\u{0301}x";
    let graphemes: Vec<&str> = GraphemeIterator::new(text, GraphemeMode::Strict).collect();
    assert_eq!(graphemes, vec!["e\u{0301}", "x"]);
}

#[test]
fn grapheme_iteration_multiple_combining_marks() {
    // Validates: Requirement 9.5
    // 'a' + combining diaeresis (U+0308) + combining acute (U+0301)
    let text = "a\u{0308}\u{0301}";
    let graphemes: Vec<&str> = GraphemeIterator::new(text, GraphemeMode::Strict).collect();
    assert_eq!(graphemes.len(), 1); // All one grapheme cluster
    assert_eq!(graphemes[0], "a\u{0308}\u{0301}");
}

#[test]
fn grapheme_regional_indicators_form_flag() {
    // Validates: Requirement 9.6
    // US flag: Regional Indicator U (U+1F1FA) + Regional Indicator S (U+1F1F8)
    let text = "\u{1F1FA}\u{1F1F8}";
    let graphemes: Vec<&str> = GraphemeIterator::new(text, GraphemeMode::Strict).collect();
    assert_eq!(graphemes.len(), 1);
    assert_eq!(graphemes[0], "\u{1F1FA}\u{1F1F8}");
}

#[test]
fn grapheme_simplified_mode_per_code_point() {
    // Validates: Requirement 9.8
    let text = "e\u{0301}"; // é as base + combining
    let graphemes: Vec<&str> = GraphemeIterator::new(text, GraphemeMode::Simplified).collect();
    // Simplified mode treats each code point separately
    assert_eq!(graphemes.len(), 2);
}

#[test]
fn grapheme_mixed_scripts() {
    // Validates: Requirement 9.1
    let text = "aé中"; // ASCII + combining mark sequence + CJK
    let graphemes: Vec<&str> = GraphemeIterator::new(text, GraphemeMode::Strict).collect();
    // 'a' is one grapheme, 'é' (if precomposed) is one grapheme, '中' is one
    assert_eq!(graphemes.len(), 3);
}

#[test]
fn grapheme_boundary_at_string_boundaries() {
    // Validates: Requirement 9.2
    let text = "test";
    assert!(is_grapheme_boundary(text, 0));
    assert!(is_grapheme_boundary(text, text.len()));
}

#[test]
fn next_and_prev_grapheme_cover_entire_string() {
    // Validates: Requirement 9.3, 9.4
    let text = "He\u{0301}llo";

    // Forward traversal
    let mut positions = vec![0];
    let mut pos = 0;
    while pos < text.len() {
        pos = next_grapheme_boundary(text, pos);
        positions.push(pos);
    }

    // Backward traversal
    let mut rev_positions = vec![text.len()];
    pos = text.len();
    while pos > 0 {
        pos = prev_grapheme_boundary(text, pos);
        rev_positions.push(pos);
    }
    rev_positions.reverse();

    // Forward and backward should produce the same boundaries
    assert_eq!(positions, rev_positions);
}
