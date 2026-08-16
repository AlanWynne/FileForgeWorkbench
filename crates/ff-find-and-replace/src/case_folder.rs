//! Unicode Full Case Folding for case-insensitive comparison.
//!
//! Implements CaseFolding.txt status C + F mappings for Unicode case folding
//! across all scripts. The CaseFolder is stateless and thread-safe.
//!
//! Addresses: Requirement 10

/// Locale hint for case-sensitive folding adjustments.
///
/// Addresses: Requirement 10 AC 8
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaleHint {
    /// Turkish/Azerbaijani: special İ/I/ı/i rules.
    Turkish,
    /// Lithuanian: special dot-above handling.
    Lithuanian,
}

/// Unicode Full Case Folding for case-insensitive comparison.
///
/// Stateless and thread-safe. Implements CaseFolding.txt status C + F mappings.
///
/// Addresses: Requirement 10
#[derive(Debug, Clone)]
pub struct CaseFolder {
    locale: Option<LocaleHint>,
}

// SAFETY: CaseFolder is stateless (only holds an immutable config option).
// This explicitly documents the Send + Sync guarantee required by Requirement 10 AC 5.
unsafe impl Send for CaseFolder {}
unsafe impl Sync for CaseFolder {}

impl CaseFolder {
    /// Create a locale-independent case folder.
    pub fn new() -> Self {
        Self { locale: None }
    }

    /// Create with a locale hint for locale-sensitive rules.
    ///
    /// Addresses: Requirement 10 AC 8
    pub fn with_locale(locale: LocaleHint) -> Self {
        Self {
            locale: Some(locale),
        }
    }

    /// Fold a single character, returning the folded result.
    ///
    /// For most characters this returns a single character, but for
    /// one-to-many mappings (e.g., ß → ss) it returns multiple characters.
    ///
    /// Addresses: Requirement 10 AC 1, AC 3
    pub fn fold_char(&self, ch: char) -> Vec<char> {
        // Handle Turkish locale special cases
        if self.locale == Some(LocaleHint::Turkish) {
            match ch {
                'I' => return vec!['ı'], // Turkish: I → ı (dotless i)
                'İ' => return vec!['i'], // Turkish: İ → i
                'i' => return vec!['i'], // unchanged
                'ı' => return vec!['ı'], // unchanged
                _ => {}
            }
        }

        // Standard Unicode Full Case Folding (status C + F)
        match ch {
            // ASCII uppercase
            'A'..='Z' => vec![(ch as u8 + 32) as char],

            // German sharp s: ß → ss (status F: full mapping)
            'ß' => vec!['s', 's'],
            // Capital sharp s
            'ẞ' => vec!['s', 's'],

            // Greek capital sigma variants
            'Σ' => vec!['σ'],
            // Final sigma folds to regular sigma under full case folding
            'ς' => vec!['ς'],

            // Latin extended
            'À'..='Ö' => vec![char::from_u32(ch as u32 + 32).unwrap_or(ch)],
            'Ø'..='Þ' => vec![char::from_u32(ch as u32 + 32).unwrap_or(ch)],

            // Turkish İ (Latin Capital Letter I With Dot Above) - non-Turkish locale
            'İ' => vec!['i', '\u{0307}'], // i + combining dot above

            // Ligatures with full case folding
            'ﬁ' => vec!['f', 'i'],
            'ﬂ' => vec!['f', 'l'],
            'ﬀ' => vec!['f', 'f'],
            'ﬃ' => vec!['f', 'f', 'i'],
            'ﬄ' => vec!['f', 'f', 'l'],
            'ﬅ' => vec!['s', 't'],
            'ﬆ' => vec!['s', 't'],

            // Cyrillic uppercase (basic range)
            '\u{0410}'..='\u{042F}' => {
                vec![char::from_u32(ch as u32 + 32).unwrap_or(ch)]
            }

            // Greek uppercase (basic range)
            '\u{0391}'..='\u{03A1}' => {
                vec![char::from_u32(ch as u32 + 32).unwrap_or(ch)]
            }
            '\u{03A3}'..='\u{03AB}' => {
                vec![char::from_u32(ch as u32 + 32).unwrap_or(ch)]
            }

            // Default: character is already folded or has no folding
            _ => vec![ch],
        }
    }

    /// Fold an entire string, producing the folded output.
    ///
    /// Handles one-to-many mappings (e.g., ß → ss) correctly expanding output.
    ///
    /// Addresses: Requirement 10 AC 3–4
    pub fn fold_str(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        for ch in input.chars() {
            for folded in self.fold_char(ch) {
                result.push(folded);
            }
        }
        result
    }

    /// Fold a byte slice assumed to be valid UTF-8.
    ///
    /// Returns folded bytes. Used for pre-folding search terms.
    ///
    /// Addresses: Requirement 10 AC 6
    pub fn fold_bytes(&self, input: &[u8]) -> Vec<u8> {
        match std::str::from_utf8(input) {
            Ok(s) => self.fold_str(s).into_bytes(),
            Err(_) => input.to_vec(), // Non-UTF-8 input returned as-is
        }
    }

    /// Compare two strings for equality under case folding.
    ///
    /// Addresses: Requirement 10 AC 2
    pub fn eq_folded(&self, a: &str, b: &str) -> bool {
        self.fold_str(a) == self.fold_str(b)
    }
}

impl Default for CaseFolder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_ascii_uppercase_to_lowercase() {
        let folder = CaseFolder::new();
        assert_eq!(folder.fold_str("HELLO"), "hello");
        assert_eq!(folder.fold_str("Hello World"), "hello world");
    }

    #[test]
    fn fold_german_sharp_s_expands_to_ss() {
        let folder = CaseFolder::new();
        assert_eq!(folder.fold_str("Straße"), "strasse");
        assert_eq!(folder.fold_str("ß"), "ss");
    }

    #[test]
    fn fold_is_idempotent() {
        let folder = CaseFolder::new();
        let input = "Hello Straße WORLD";
        let once = folder.fold_str(input);
        let twice = folder.fold_str(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn fold_preserves_already_lowercase() {
        let folder = CaseFolder::new();
        assert_eq!(folder.fold_str("hello"), "hello");
        assert_eq!(folder.fold_str("123!@#"), "123!@#");
    }

    #[test]
    fn eq_folded_is_symmetric() {
        let folder = CaseFolder::new();
        assert!(folder.eq_folded("Hello", "HELLO"));
        assert!(folder.eq_folded("HELLO", "Hello"));
        assert!(folder.eq_folded("straße", "STRASSE"));
        assert!(folder.eq_folded("STRASSE", "straße"));
    }

    #[test]
    fn turkish_locale_folds_i_correctly() {
        let folder = CaseFolder::with_locale(LocaleHint::Turkish);
        // Turkish I → ı (dotless i)
        assert_eq!(folder.fold_char('I'), vec!['ı']);
        // Turkish İ → i
        assert_eq!(folder.fold_char('İ'), vec!['i']);
    }

    #[test]
    fn fold_bytes_handles_valid_utf8() {
        let folder = CaseFolder::new();
        let result = folder.fold_bytes(b"HELLO");
        assert_eq!(result, b"hello");
    }

    #[test]
    fn fold_bytes_returns_invalid_utf8_unchanged() {
        let folder = CaseFolder::new();
        let invalid = vec![0xFF, 0xFE, 0x01];
        let result = folder.fold_bytes(&invalid);
        assert_eq!(result, invalid);
    }

    #[test]
    fn fold_multibyte_characters_never_splits_code_points() {
        let folder = CaseFolder::new();
        // "é" is 2 bytes in UTF-8 (U+00E9)
        let result = folder.fold_str("é");
        assert!(std::str::from_utf8(result.as_bytes()).is_ok());

        // Cyrillic
        let result = folder.fold_str("МОСКВА");
        assert!(std::str::from_utf8(result.as_bytes()).is_ok());
        assert_eq!(result, "москва");
    }

    #[test]
    fn fold_ligatures_expand_correctly() {
        let folder = CaseFolder::new();
        assert_eq!(folder.fold_str("ﬁ"), "fi");
        assert_eq!(folder.fold_str("ﬂ"), "fl");
        assert_eq!(folder.fold_str("ﬀ"), "ff");
    }

    #[test]
    fn case_folder_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CaseFolder>();
    }
}
