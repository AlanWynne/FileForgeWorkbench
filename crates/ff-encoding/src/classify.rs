//! Character classification for word-boundary detection.
//!
//! Provides a 256-entry lookup table mapping each byte value to one of four
//! classes: Space, NewLine, Word, or Punctuation.

/// Classification of a byte value for word-boundary detection.
///
/// [Requirement 6]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterClass {
    /// Space characters (whitespace excluding newlines)
    Space,
    /// Line ending characters (CR, LF)
    NewLine,
    /// Word characters (alphanumeric, underscore, configurable)
    Word,
    /// Punctuation and other non-word, non-space characters
    Punctuation,
}

/// A 256-entry lookup table mapping byte values to character classes.
///
/// Provides O(1) classification for the ASCII/Latin-1 byte range.
///
/// [Requirement 6]
#[derive(Debug, Clone)]
pub struct CharClassify {
    classes: [CharacterClass; 256],
}

impl CharClassify {
    /// Create with default classifications.
    ///
    /// If `include_word_class` is true: alphanumeric + underscore + 0x80–0xFF = Word.
    /// If false: all non-space/non-newline = Punctuation.
    ///
    /// [Requirement 6.2, 6.3]
    pub fn new(include_word_class: bool) -> Self {
        let mut classes = [CharacterClass::Punctuation; 256];

        // Set newline characters
        classes[b'\n' as usize] = CharacterClass::NewLine;
        classes[b'\r' as usize] = CharacterClass::NewLine;

        // Set space/control characters
        classes[b' ' as usize] = CharacterClass::Space;
        classes[b'\t' as usize] = CharacterClass::Space;
        classes[0x0B] = CharacterClass::Space; // Vertical tab
        classes[0x0C] = CharacterClass::Space; // Form feed

        // Control characters (excluding CR, LF, TAB, VT, FF)
        for i in 0..=0x08u8 {
            classes[i as usize] = CharacterClass::Space;
        }
        classes[0x0E] = CharacterClass::Space;
        classes[0x0F] = CharacterClass::Space;
        for i in 0x10..=0x1Fu8 {
            classes[i as usize] = CharacterClass::Space;
        }
        classes[0x7F] = CharacterClass::Space; // DEL

        if include_word_class {
            // Alphanumeric + underscore = Word
            for i in b'0'..=b'9' {
                classes[i as usize] = CharacterClass::Word;
            }
            for i in b'A'..=b'Z' {
                classes[i as usize] = CharacterClass::Word;
            }
            for i in b'a'..=b'z' {
                classes[i as usize] = CharacterClass::Word;
            }
            classes[b'_' as usize] = CharacterClass::Word;

            // High bytes 0x80–0xFF = Word (covers extended Latin, etc.)
            for i in 0x80..=0xFFu16 {
                classes[i as usize] = CharacterClass::Word;
            }
        }

        Self { classes }
    }

    /// Classify a byte value. O(1) array lookup.
    ///
    /// [Requirement 6.5]
    pub fn classify(&self, byte: u8) -> CharacterClass {
        self.classes[byte as usize]
    }

    /// Fast predicate: is this byte classified as Word?
    ///
    /// [Requirement 6.6]
    pub fn is_word(&self, byte: u8) -> bool {
        self.classes[byte as usize] == CharacterClass::Word
    }

    /// Set classification for a set of byte values.
    ///
    /// [Requirement 6.4]
    pub fn set_char_classes(&mut self, chars: &[u8], class: CharacterClass) {
        for &ch in chars {
            self.classes[ch as usize] = class;
        }
    }

    /// Configure word characters from a set of byte values.
    ///
    /// [Requirement 13.2]
    pub fn set_word_chars(&mut self, chars: &[u8]) {
        self.set_char_classes(chars, CharacterClass::Word);
    }

    /// Configure whitespace characters.
    ///
    /// [Requirement 13.3]
    pub fn set_whitespace_chars(&mut self, chars: &[u8]) {
        self.set_char_classes(chars, CharacterClass::Space);
    }

    /// Configure punctuation characters.
    ///
    /// [Requirement 13.4]
    pub fn set_punctuation_chars(&mut self, chars: &[u8]) {
        self.set_char_classes(chars, CharacterClass::Punctuation);
    }

    /// Reset to default word-character classification.
    ///
    /// [Requirement 13.5]
    pub fn reset_word_chars(&mut self) {
        *self = Self::new(true);
    }

    /// Get all byte values currently assigned to a class.
    ///
    /// [Requirement 6.7]
    pub fn get_chars_of_class(&self, class: CharacterClass) -> Vec<u8> {
        (0..=255u16)
            .filter(|&i| self.classes[i as usize] == class)
            .map(|i| i as u8)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_classification_with_word_class() {
        // Validates: Requirement 6.1, 6.2
        let classify = CharClassify::new(true);
        assert_eq!(classify.classify(b'A'), CharacterClass::Word);
        assert_eq!(classify.classify(b'z'), CharacterClass::Word);
        assert_eq!(classify.classify(b'0'), CharacterClass::Word);
        assert_eq!(classify.classify(b'_'), CharacterClass::Word);
        assert_eq!(classify.classify(0x80), CharacterClass::Word);
        assert_eq!(classify.classify(0xFF), CharacterClass::Word);
        assert_eq!(classify.classify(b' '), CharacterClass::Space);
        assert_eq!(classify.classify(b'\n'), CharacterClass::NewLine);
        assert_eq!(classify.classify(b'\r'), CharacterClass::NewLine);
        assert_eq!(classify.classify(b'!'), CharacterClass::Punctuation);
        assert_eq!(classify.classify(b'.'), CharacterClass::Punctuation);
    }

    #[test]
    fn default_classification_without_word_class() {
        // Validates: Requirement 6.3
        let classify = CharClassify::new(false);
        assert_eq!(classify.classify(b'A'), CharacterClass::Punctuation);
        assert_eq!(classify.classify(b'z'), CharacterClass::Punctuation);
        assert_eq!(classify.classify(b' '), CharacterClass::Space);
        assert_eq!(classify.classify(b'\n'), CharacterClass::NewLine);
    }

    #[test]
    fn is_word_returns_correct_values() {
        // Validates: Requirement 6.6
        let classify = CharClassify::new(true);
        assert!(classify.is_word(b'A'));
        assert!(classify.is_word(b'_'));
        assert!(!classify.is_word(b' '));
        assert!(!classify.is_word(b'!'));
    }

    #[test]
    fn set_char_classes_overrides_classification() {
        // Validates: Requirement 6.4
        let mut classify = CharClassify::new(true);
        assert_eq!(classify.classify(b'$'), CharacterClass::Punctuation);
        classify.set_char_classes(b"$#", CharacterClass::Word);
        assert_eq!(classify.classify(b'$'), CharacterClass::Word);
        assert_eq!(classify.classify(b'#'), CharacterClass::Word);
    }

    #[test]
    fn set_word_chars_adds_to_word_class() {
        // Validates: Requirement 13.2
        let mut classify = CharClassify::new(true);
        classify.set_word_chars(b"$-");
        assert!(classify.is_word(b'$'));
        assert!(classify.is_word(b'-'));
    }

    #[test]
    fn reset_word_chars_restores_defaults() {
        // Validates: Requirement 13.5
        let mut classify = CharClassify::new(true);
        classify.set_word_chars(b"$");
        assert!(classify.is_word(b'$'));
        classify.reset_word_chars();
        assert!(!classify.is_word(b'$'));
    }

    #[test]
    fn get_chars_of_class_returns_correct_set() {
        // Validates: Requirement 6.7
        let classify = CharClassify::new(true);
        let newlines = classify.get_chars_of_class(CharacterClass::NewLine);
        assert!(newlines.contains(&b'\n'));
        assert!(newlines.contains(&b'\r'));
        assert_eq!(newlines.len(), 2);
    }

    #[test]
    fn every_byte_has_exactly_one_class() {
        // Validates: Requirement 6.1
        let classify = CharClassify::new(true);
        for i in 0..=255u16 {
            let class = classify.classify(i as u8);
            assert!(
                matches!(
                    class,
                    CharacterClass::Space
                        | CharacterClass::NewLine
                        | CharacterClass::Word
                        | CharacterClass::Punctuation
                ),
                "Byte {} has invalid class {:?}",
                i,
                class
            );
        }
    }
}
