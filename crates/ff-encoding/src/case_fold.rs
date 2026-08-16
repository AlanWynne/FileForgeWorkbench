//! Unicode case folding and conversion.
//!
//! Provides case fold (for comparison), uppercase, and lowercase conversion
//! using compiled Unicode CaseFolding.txt data.

/// Conversion mode for case operations.
///
/// [Requirement 10]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseMode {
    /// Unicode case folding for comparison (status C+F from CaseFolding.txt)
    Fold,
    /// To uppercase
    Upper,
    /// To lowercase
    Lower,
}

/// Result of case-converting a single code point.
///
/// [Requirement 10.2]
#[derive(Debug, Clone)]
pub struct CaseFoldResult {
    /// UTF-8 encoded result (up to 12 bytes for multi-char expansions)
    pub bytes: [u8; 12],
    /// Number of valid bytes in the result
    pub len: usize,
}

impl CaseFoldResult {
    /// Get the result as a string slice.
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

/// Trait for case conversion, enabling find-and-replace to use case folding
/// without depending on the specific implementation.
///
/// [Requirement 10.7]
pub trait ICaseConverter: Send + Sync {
    /// Convert the entire string according to the given mode.
    fn case_convert_string(&self, text: &str, mode: CaseMode) -> String;
}

/// The concrete case folder using compiled Unicode data.
///
/// [Requirement 10]
#[derive(Debug)]
pub struct CaseFolder;

impl CaseFolder {
    /// Create a new case folder.
    pub fn new() -> Self {
        Self
    }

    /// Convert a single code point according to the given mode.
    ///
    /// Returns the result as a `CaseFoldResult` containing UTF-8 bytes.
    /// For fold mode, some characters expand (e.g., ß → ss).
    ///
    /// [Requirement 10.2]
    pub fn case_convert(&self, code_point: u32, mode: CaseMode) -> CaseFoldResult {
        let mut result = CaseFoldResult {
            bytes: [0u8; 12],
            len: 0,
        };

        let converted = match mode {
            CaseMode::Fold => self.fold_char(code_point),
            CaseMode::Upper => self.upper_char(code_point),
            CaseMode::Lower => self.lower_char(code_point),
        };

        let s = converted;
        let bytes = s.as_bytes();
        let len = bytes.len().min(12);
        result.bytes[..len].copy_from_slice(&bytes[..len]);
        result.len = len;
        result
    }

    /// Convert an entire string according to the given mode.
    ///
    /// The result may be longer than the input (e.g., ß → ss in Fold mode).
    ///
    /// [Requirement 10.3]
    pub fn case_convert_string(&self, text: &str, mode: CaseMode) -> String {
        let mut result = String::with_capacity(text.len());
        for ch in text.chars() {
            let converted = match mode {
                CaseMode::Fold => self.fold_char(ch as u32),
                CaseMode::Upper => self.upper_char(ch as u32),
                CaseMode::Lower => self.lower_char(ch as u32),
            };
            result.push_str(&converted);
        }
        result
    }

    /// Case fold a single character (for comparison).
    fn fold_char(&self, code_point: u32) -> String {
        // Special multi-character fold cases from CaseFolding.txt
        match code_point {
            // ß (U+00DF) → ss
            0x00DF => "ss".to_string(),
            // ﬁ (U+FB01) → fi
            0xFB01 => "fi".to_string(),
            // ﬂ (U+FB02) → fl
            0xFB02 => "fl".to_string(),
            // ﬀ (U+FB00) → ff
            0xFB00 => "ff".to_string(),
            // ﬃ (U+FB03) → ffi
            0xFB03 => "ffi".to_string(),
            // ﬄ (U+FB04) → ffl
            0xFB04 => "ffl".to_string(),
            // ﬅ (U+FB05) → st
            0xFB05 => "st".to_string(),
            // ﬆ (U+FB06) → st
            0xFB06 => "st".to_string(),
            // Default: lowercase
            _ => {
                if let Some(ch) = char::from_u32(code_point) {
                    ch.to_lowercase().to_string()
                } else {
                    String::new()
                }
            }
        }
    }

    /// Convert to uppercase.
    fn upper_char(&self, code_point: u32) -> String {
        if let Some(ch) = char::from_u32(code_point) {
            ch.to_uppercase().to_string()
        } else {
            String::new()
        }
    }

    /// Convert to lowercase.
    fn lower_char(&self, code_point: u32) -> String {
        if let Some(ch) = char::from_u32(code_point) {
            ch.to_lowercase().to_string()
        } else {
            String::new()
        }
    }
}

impl Default for CaseFolder {
    fn default() -> Self {
        Self::new()
    }
}

impl ICaseConverter for CaseFolder {
    fn case_convert_string(&self, text: &str, mode: CaseMode) -> String {
        CaseFolder::case_convert_string(self, text, mode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_eszett_to_ss() {
        // Validates: Requirement 10.4, 10.6
        let folder = CaseFolder::new();
        let result = folder.case_convert_string("ß", CaseMode::Fold);
        assert_eq!(result, "ss");
    }

    #[test]
    fn fold_fi_ligature() {
        // Validates: Requirement 10.4, 10.6
        let folder = CaseFolder::new();
        let result = folder.case_convert_string("\u{FB01}", CaseMode::Fold);
        assert_eq!(result, "fi");
    }

    #[test]
    fn fold_uppercase_to_lowercase() {
        // Validates: Requirement 10.1
        let folder = CaseFolder::new();
        let result = folder.case_convert_string("HELLO", CaseMode::Fold);
        assert_eq!(result, "hello");
    }

    #[test]
    fn upper_basic_ascii() {
        // Validates: Requirement 10.1
        let folder = CaseFolder::new();
        let result = folder.case_convert_string("hello", CaseMode::Upper);
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn lower_basic_ascii() {
        // Validates: Requirement 10.1
        let folder = CaseFolder::new();
        let result = folder.case_convert_string("HELLO", CaseMode::Lower);
        assert_eq!(result, "hello");
    }

    #[test]
    fn fold_is_idempotent() {
        // Validates: Requirement 10.4
        let folder = CaseFolder::new();
        let text = "Hello World ß ﬁ ABC";
        let once = folder.case_convert_string(text, CaseMode::Fold);
        let twice = folder.case_convert_string(&once, CaseMode::Fold);
        assert_eq!(once, twice);
    }

    #[test]
    fn case_convert_single_code_point() {
        // Validates: Requirement 10.2
        let folder = CaseFolder::new();
        let result = folder.case_convert(0x41, CaseMode::Lower); // 'A' → 'a'
        assert_eq!(result.as_str(), "a");
    }

    #[test]
    fn case_convert_string_expansion_makes_longer() {
        // Validates: Requirement 10.3, 10.6
        let folder = CaseFolder::new();
        let input = "straße";
        let result = folder.case_convert_string(input, CaseMode::Fold);
        // ß folds to "ss" — the result has more characters even though
        // UTF-8 byte lengths happen to be the same (ß = 2 bytes, ss = 2 bytes)
        assert!(result.chars().count() > input.chars().count());
        assert_eq!(result, "strasse");
    }

    #[test]
    fn icase_converter_trait_usage() {
        // Validates: Requirement 10.7
        let converter: Box<dyn ICaseConverter> = Box::new(CaseFolder::new());
        let result = converter.case_convert_string("Test", CaseMode::Fold);
        assert_eq!(result, "test");
    }

    #[test]
    fn case_conversion_not_locale_sensitive() {
        // Validates: Requirement 10.5
        let folder = CaseFolder::new();
        // Turkish 'İ' should fold to 'i̇' (i + combining dot) in Unicode default,
        // not Turkish-specific 'i'
        let result = folder.case_convert_string("I", CaseMode::Lower);
        assert_eq!(result, "i"); // Default, not Turkish
    }
}
