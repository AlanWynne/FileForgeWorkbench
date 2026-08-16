//! Integration tests for CharClassify and CharacterCategoryMap.

use ff_encoding::*;

#[test]
fn classify_and_category_map_agree_on_word_chars() {
    // Validates: Requirement 6.1, 7.6
    let classify = CharClassify::new(true);
    let map = CharacterCategoryMap::new();

    // ASCII letters should be Word in both
    for byte in b'A'..=b'Z' {
        assert!(classify.is_word(byte));
        assert!(map.is_word_char(byte as u32));
    }
    for byte in b'a'..=b'z' {
        assert!(classify.is_word(byte));
        assert!(map.is_word_char(byte as u32));
    }
    // Digits
    for byte in b'0'..=b'9' {
        assert!(classify.is_word(byte));
        assert!(map.is_word_char(byte as u32));
    }
    // Underscore
    assert!(classify.is_word(b'_'));
    assert!(map.is_word_char('_' as u32));
}

#[test]
fn classify_space_and_punctuation_consistent() {
    // Validates: Requirement 6.1
    let classify = CharClassify::new(true);

    // Space is Space
    assert_eq!(classify.classify(b' '), CharacterClass::Space);
    // Common punctuation
    assert_eq!(classify.classify(b'!'), CharacterClass::Punctuation);
    assert_eq!(classify.classify(b'.'), CharacterClass::Punctuation);
    assert_eq!(classify.classify(b','), CharacterClass::Punctuation);
}

#[test]
fn custom_word_chars_for_php() {
    // Validates: Requirement 13.1, 13.2
    let mut classify = CharClassify::new(true);
    // PHP uses $ as part of variable names
    assert!(!classify.is_word(b'$'));
    classify.set_word_chars(b"$");
    assert!(classify.is_word(b'$'));
}

#[test]
fn custom_word_chars_for_lisp() {
    // Validates: Requirement 13.1, 13.2
    let mut classify = CharClassify::new(true);
    // Lisp uses - in identifiers
    assert!(!classify.is_word(b'-'));
    classify.set_word_chars(b"-");
    assert!(classify.is_word(b'-'));
}

#[test]
fn category_map_word_boundary_across_scripts() {
    // Validates: Requirement 7.6
    let map = CharacterCategoryMap::new();

    // CJK characters are word-like (Lo category)
    assert!(map.is_word_char(0x4E2D)); // 中
    assert!(map.is_word_char(0x65E5)); // 日

    // Cyrillic
    assert!(map.is_word_char(0x0410)); // А
    assert!(map.is_word_char(0x0430)); // а

    // Space is not word
    assert!(!map.is_word_char(0x20));

    // Punctuation is not word
    assert!(!map.is_word_char(0x21)); // !
}
