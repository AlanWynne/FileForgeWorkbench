//! Character classification engine for word navigation.
//!
//! Provides configurable classification of characters into categories
//! (Space, NewLine, Word, Punctuation) used by word and word-part navigation
//! to detect boundaries.

/// Character class categories for word navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterClass {
    /// Whitespace characters (space, tab, etc.).
    Space,
    /// Line ending characters (LF, CR).
    NewLine,
    /// Word characters (alphanumeric + configured extras).
    Word,
    /// Punctuation/symbol characters (everything else).
    Punctuation,
}

/// Configurable character classification table.
///
/// ASCII characters (0x00–0x7F) use a lookup table for fast classification.
/// Unicode characters (>= 0x80) use Unicode category fallback logic.
#[derive(Debug, Clone)]
pub struct CharClassifier {
    /// Classification for each ASCII byte (0–127).
    ascii_table: [CharacterClass; 128],
    /// Additional characters to treat as word characters (from config).
    extra_word_chars: Vec<char>,
}

impl CharClassifier {
    /// Create with default classification.
    ///
    /// - Alphanumeric + underscore = Word
    /// - Space, tab, form feed = Space
    /// - CR, LF = NewLine
    /// - Everything else = Punctuation
    pub fn new() -> Self {
        let mut table = [CharacterClass::Punctuation; 128];

        // Whitespace
        table[b' ' as usize] = CharacterClass::Space;
        table[b'\t' as usize] = CharacterClass::Space;
        table[0x0B] = CharacterClass::Space; // vertical tab
        table[0x0C] = CharacterClass::Space; // form feed

        // Newlines
        table[b'\n' as usize] = CharacterClass::NewLine;
        table[b'\r' as usize] = CharacterClass::NewLine;

        // Word characters: a-z, A-Z, 0-9, _
        for b in b'a'..=b'z' {
            table[b as usize] = CharacterClass::Word;
        }
        for b in b'A'..=b'Z' {
            table[b as usize] = CharacterClass::Word;
        }
        for b in b'0'..=b'9' {
            table[b as usize] = CharacterClass::Word;
        }
        table[b'_' as usize] = CharacterClass::Word;

        Self {
            ascii_table: table,
            extra_word_chars: Vec::new(),
        }
    }

    /// Classify a single character.
    pub fn classify(&self, ch: char) -> CharacterClass {
        if (ch as u32) < 128 {
            self.ascii_table[ch as usize]
        } else {
            // Check extra word chars first
            if self.extra_word_chars.contains(&ch) {
                return CharacterClass::Word;
            }
            // Unicode fallback: alphanumeric = Word, whitespace = Space
            if ch.is_alphanumeric() {
                CharacterClass::Word
            } else if ch.is_whitespace() {
                CharacterClass::Space
            } else {
                CharacterClass::Punctuation
            }
        }
    }

    /// Classify a byte (ASCII fast path).
    pub fn classify_byte(&self, byte: u8) -> CharacterClass {
        if byte < 128 {
            self.ascii_table[byte as usize]
        } else {
            // Non-ASCII bytes — treat as Word by default (part of multi-byte UTF-8)
            CharacterClass::Word
        }
    }

    /// Check if a character is classified as Word.
    pub fn is_word_char(&self, ch: char) -> bool {
        self.classify(ch) == CharacterClass::Word
    }

    /// Check if a character is classified as Space.
    pub fn is_space(&self, ch: char) -> bool {
        self.classify(ch) == CharacterClass::Space
    }

    /// Set custom character classes for a set of characters.
    pub fn set_char_classes(&mut self, chars: &str, class: CharacterClass) {
        for ch in chars.chars() {
            if (ch as u32) < 128 {
                self.ascii_table[ch as usize] = class;
            } else if class == CharacterClass::Word && !self.extra_word_chars.contains(&ch) {
                self.extra_word_chars.push(ch);
            }
        }
    }

    /// Reset to default classification.
    pub fn set_default_classes(&mut self) {
        *self = Self::new();
    }

    /// Add extra word characters from configuration.
    pub fn add_word_characters(&mut self, chars: &str) {
        self.set_char_classes(chars, CharacterClass::Word);
    }
}

impl Default for CharClassifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_letters_classified_as_word() {
        let c = CharClassifier::new();
        assert_eq!(c.classify('a'), CharacterClass::Word);
        assert_eq!(c.classify('Z'), CharacterClass::Word);
        assert_eq!(c.classify('5'), CharacterClass::Word);
        assert_eq!(c.classify('_'), CharacterClass::Word);
    }

    #[test]
    fn space_and_tab_classified_as_space() {
        let c = CharClassifier::new();
        assert_eq!(c.classify(' '), CharacterClass::Space);
        assert_eq!(c.classify('\t'), CharacterClass::Space);
    }

    #[test]
    fn newlines_classified_as_newline() {
        let c = CharClassifier::new();
        assert_eq!(c.classify('\n'), CharacterClass::NewLine);
        assert_eq!(c.classify('\r'), CharacterClass::NewLine);
    }

    #[test]
    fn punctuation_classified_correctly() {
        let c = CharClassifier::new();
        assert_eq!(c.classify('.'), CharacterClass::Punctuation);
        assert_eq!(c.classify('+'), CharacterClass::Punctuation);
        assert_eq!(c.classify('('), CharacterClass::Punctuation);
    }

    #[test]
    fn unicode_alphanumeric_classified_as_word() {
        let c = CharClassifier::new();
        assert_eq!(c.classify('é'), CharacterClass::Word);
        assert_eq!(c.classify('日'), CharacterClass::Word);
    }

    #[test]
    fn custom_char_class_override() {
        let mut c = CharClassifier::new();
        c.set_char_classes("-$", CharacterClass::Word);
        assert_eq!(c.classify('-'), CharacterClass::Word);
        assert_eq!(c.classify('$'), CharacterClass::Word);
    }

    #[test]
    fn reset_restores_defaults() {
        let mut c = CharClassifier::new();
        c.set_char_classes("-", CharacterClass::Word);
        assert_eq!(c.classify('-'), CharacterClass::Word);
        c.set_default_classes();
        assert_eq!(c.classify('-'), CharacterClass::Punctuation);
    }

    #[test]
    fn add_word_characters_extends_classification() {
        let mut c = CharClassifier::new();
        c.add_word_characters("@#");
        assert_eq!(c.classify('@'), CharacterClass::Word);
        assert_eq!(c.classify('#'), CharacterClass::Word);
    }
}
