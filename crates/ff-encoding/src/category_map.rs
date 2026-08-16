//! Unicode General Category classification (CharacterCategoryMap).
//!
//! Provides lookup of Unicode General Category for any code point,
//! using a dense array for BMP and binary search for supplementary planes.

/// Unicode General Category (30 categories per Unicode standard).
///
/// [Requirement 7]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum CharacterCategory {
    Lu = 0,
    Ll,
    Lt,
    Lm,
    Lo,
    Mn,
    Mc,
    Me,
    Nd,
    Nl,
    No,
    Pc,
    Pd,
    Ps,
    Pe,
    Pi,
    Pf,
    Po,
    Sm,
    Sc,
    Sk,
    So,
    Zs,
    Zl,
    Zp,
    Cc,
    Cf,
    Cs,
    Co,
    Cn,
}

/// Optimized lookup structure: dense array for BMP, binary search for
/// supplementary planes.
///
/// [Requirement 7]
pub struct CharacterCategoryMap {
    /// Dense array for U+0000..U+FFFF (65536 entries)
    bmp_table: Vec<CharacterCategory>,
    /// Sorted ranges for supplementary planes: (start, end, category)
    supplementary_ranges: Vec<(u32, u32, CharacterCategory)>,
}

impl CharacterCategoryMap {
    /// Create a new category map with basic Unicode data.
    ///
    /// Uses simplified built-in tables covering the most important ranges.
    /// Full Unicode data will be generated at build time in the production version.
    pub fn new() -> Self {
        let mut bmp_table = vec![CharacterCategory::Cn; 65536];

        // ASCII range
        // Control chars C0
        for i in 0x00..=0x1Fu32 {
            bmp_table[i as usize] = CharacterCategory::Cc;
        }
        bmp_table[0x7F] = CharacterCategory::Cc; // DEL

        // Space
        bmp_table[0x20] = CharacterCategory::Zs;

        // Digits
        for i in 0x30..=0x39u32 {
            bmp_table[i as usize] = CharacterCategory::Nd;
        }

        // Uppercase Latin
        for i in 0x41..=0x5Au32 {
            bmp_table[i as usize] = CharacterCategory::Lu;
        }

        // Lowercase Latin
        for i in 0x61..=0x7Au32 {
            bmp_table[i as usize] = CharacterCategory::Ll;
        }

        // Underscore is Pc (Connector Punctuation)
        bmp_table[0x5F] = CharacterCategory::Pc;

        // Common punctuation
        for &cp in &[
            0x21u32, 0x22, 0x23, 0x25, 0x26, 0x27, 0x2A, 0x2C, 0x2E, 0x2F, 0x3A, 0x3B, 0x3F, 0x40,
            0x5C,
        ] {
            bmp_table[cp as usize] = CharacterCategory::Po;
        }

        // Brackets/parens
        bmp_table[0x28] = CharacterCategory::Ps; // (
        bmp_table[0x29] = CharacterCategory::Pe; // )
        bmp_table[0x5B] = CharacterCategory::Ps; // [
        bmp_table[0x5D] = CharacterCategory::Pe; // ]
        bmp_table[0x7B] = CharacterCategory::Ps; // {
        bmp_table[0x7D] = CharacterCategory::Pe; // }

        // Math symbols
        bmp_table[0x2B] = CharacterCategory::Sm; // +
        bmp_table[0x3C] = CharacterCategory::Sm; // <
        bmp_table[0x3D] = CharacterCategory::Sm; // =
        bmp_table[0x3E] = CharacterCategory::Sm; // >
        bmp_table[0x7C] = CharacterCategory::Sm; // |
        bmp_table[0x7E] = CharacterCategory::Sm; // ~
        bmp_table[0x5E] = CharacterCategory::Sk; // ^

        // Currency
        bmp_table[0x24] = CharacterCategory::Sc; // $

        // Dash
        bmp_table[0x2D] = CharacterCategory::Pd; // -

        // Latin-1 Supplement (U+0080-U+00FF)
        for i in 0x80..=0x9Fu32 {
            bmp_table[i as usize] = CharacterCategory::Cc; // C1 controls
        }
        bmp_table[0xA0] = CharacterCategory::Zs; // NBSP
        bmp_table[0xAD] = CharacterCategory::Cf; // Soft hyphen

        // Latin Extended (simplified - uppercase/lowercase blocks)
        for i in 0xC0..=0xD6u32 {
            bmp_table[i as usize] = CharacterCategory::Lu;
        }
        for i in 0xD8..=0xDEu32 {
            bmp_table[i as usize] = CharacterCategory::Lu;
        }
        bmp_table[0xD7] = CharacterCategory::Sm; // ×
        for i in 0xDF..=0xF6u32 {
            bmp_table[i as usize] = CharacterCategory::Ll;
        }
        for i in 0xF8..=0xFFu32 {
            bmp_table[i as usize] = CharacterCategory::Ll;
        }
        bmp_table[0xF7] = CharacterCategory::Sm; // ÷

        // CJK Unified Ideographs (U+4E00-U+9FFF)
        for i in 0x4E00..=0x9FFFu32 {
            bmp_table[i as usize] = CharacterCategory::Lo;
        }

        // Hiragana (U+3040-U+309F)
        for i in 0x3041..=0x3096u32 {
            bmp_table[i as usize] = CharacterCategory::Lo;
        }

        // Katakana (U+30A0-U+30FF)
        for i in 0x30A1..=0x30FAu32 {
            bmp_table[i as usize] = CharacterCategory::Lo;
        }

        // Cyrillic uppercase (U+0400-U+042F)
        for i in 0x0410..=0x042Fu32 {
            bmp_table[i as usize] = CharacterCategory::Lu;
        }
        // Cyrillic lowercase (U+0430-U+044F)
        for i in 0x0430..=0x044Fu32 {
            bmp_table[i as usize] = CharacterCategory::Ll;
        }

        // Arabic letters (U+0621-U+064A)
        for i in 0x0621..=0x064Au32 {
            bmp_table[i as usize] = CharacterCategory::Lo;
        }

        // Combining marks (a few key ranges)
        for i in 0x0300..=0x036Fu32 {
            bmp_table[i as usize] = CharacterCategory::Mn;
        }

        // Hangul Syllables (U+AC00-U+D7A3)
        for i in 0xAC00..=0xD7A3u32 {
            bmp_table[i as usize] = CharacterCategory::Lo;
        }

        // Hangul Jamo leading (U+1100-U+1159)
        for i in 0x1100..=0x1159u32 {
            bmp_table[i as usize] = CharacterCategory::Lo;
        }

        // Surrogates
        for i in 0xD800..=0xDFFFu32 {
            bmp_table[i as usize] = CharacterCategory::Cs;
        }

        // Private Use Area
        for i in 0xE000..=0xF8FFu32 {
            bmp_table[i as usize] = CharacterCategory::Co;
        }

        // Line/Paragraph separators
        bmp_table[0x2028] = CharacterCategory::Zl;
        bmp_table[0x2029] = CharacterCategory::Zp;

        // Supplementary ranges (simplified) - MUST be sorted by start for binary search
        let supplementary_ranges = vec![
            // CJK Extension B
            (0x1D400, 0x1D7FF, CharacterCategory::Lu), // Mathematical Alphanumeric Symbols
            (0x1F300, 0x1F5FF, CharacterCategory::So), // Misc symbols
            (0x1F600, 0x1F64F, CharacterCategory::So), // Emoticons
            (0x1F680, 0x1F6FF, CharacterCategory::So), // Transport symbols
            (0x1F900, 0x1F9FF, CharacterCategory::So), // Supplemental symbols
            (0x20000, 0x2A6DF, CharacterCategory::Lo), // CJK Extension B
        ];

        Self {
            bmp_table,
            supplementary_ranges,
        }
    }

    /// Optimize the map by pre-allocating dense storage up to `count_characters`.
    ///
    /// [Requirement 7.3]
    pub fn optimize(&mut self, count_characters: usize) {
        if count_characters > self.bmp_table.len() {
            self.bmp_table
                .resize(count_characters, CharacterCategory::Cn);
            // Fill from supplementary ranges
            for &(start, end, cat) in &self.supplementary_ranges {
                for cp in start..=end {
                    if (cp as usize) < self.bmp_table.len() {
                        self.bmp_table[cp as usize] = cat;
                    }
                }
            }
        }
    }

    /// Return the Unicode General Category for a code point.
    ///
    /// O(1) for BMP, O(log n) for supplementary planes.
    ///
    /// [Requirement 7.1]
    pub fn category_for(&self, code_point: u32) -> CharacterCategory {
        if (code_point as usize) < self.bmp_table.len() {
            return self.bmp_table[code_point as usize];
        }

        // Binary search in supplementary ranges
        match self
            .supplementary_ranges
            .binary_search_by(|&(start, end, _)| {
                if code_point < start {
                    std::cmp::Ordering::Greater
                } else if code_point > end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            }) {
            Ok(idx) => self.supplementary_ranges[idx].2,
            Err(_) => CharacterCategory::Cn,
        }
    }

    /// UAX #31: Is this code point valid at the start of an identifier?
    ///
    /// ID_Start = Lu | Ll | Lt | Lm | Lo | Nl
    ///
    /// [Requirement 7.4]
    pub fn is_id_start(&self, code_point: u32) -> bool {
        matches!(
            self.category_for(code_point),
            CharacterCategory::Lu
                | CharacterCategory::Ll
                | CharacterCategory::Lt
                | CharacterCategory::Lm
                | CharacterCategory::Lo
                | CharacterCategory::Nl
        )
    }

    /// UAX #31: Is this code point valid as a continuation of an identifier?
    ///
    /// ID_Continue = ID_Start | Mn | Mc | Nd | Pc
    ///
    /// [Requirement 7.4]
    pub fn is_id_continue(&self, code_point: u32) -> bool {
        matches!(
            self.category_for(code_point),
            CharacterCategory::Lu
                | CharacterCategory::Ll
                | CharacterCategory::Lt
                | CharacterCategory::Lm
                | CharacterCategory::Lo
                | CharacterCategory::Nl
                | CharacterCategory::Mn
                | CharacterCategory::Mc
                | CharacterCategory::Nd
                | CharacterCategory::Pc
        )
    }

    /// UAX #31 extended: XID_Start property.
    ///
    /// Simplified: same as `is_id_start` for this implementation.
    ///
    /// [Requirement 7.5]
    pub fn is_xid_start(&self, code_point: u32) -> bool {
        self.is_id_start(code_point)
    }

    /// UAX #31 extended: XID_Continue property.
    ///
    /// Simplified: same as `is_id_continue` for this implementation.
    ///
    /// [Requirement 7.5]
    pub fn is_xid_continue(&self, code_point: u32) -> bool {
        self.is_id_continue(code_point)
    }

    /// Is this code point word-like?
    ///
    /// True for categories L* (Lu, Ll, Lt, Lm, Lo), Nd, Nl, Pc.
    ///
    /// [Requirement 7.6]
    pub fn is_word_char(&self, code_point: u32) -> bool {
        matches!(
            self.category_for(code_point),
            CharacterCategory::Lu
                | CharacterCategory::Ll
                | CharacterCategory::Lt
                | CharacterCategory::Lm
                | CharacterCategory::Lo
                | CharacterCategory::Nd
                | CharacterCategory::Nl
                | CharacterCategory::Pc
        )
    }
}

impl Default for CharacterCategoryMap {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for CharacterCategoryMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CharacterCategoryMap")
            .field("bmp_table_len", &self.bmp_table.len())
            .field("supplementary_ranges", &self.supplementary_ranges.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_for_ascii_letters() {
        // Validates: Requirement 7.1
        let map = CharacterCategoryMap::new();
        assert_eq!(map.category_for(0x41), CharacterCategory::Lu); // 'A'
        assert_eq!(map.category_for(0x5A), CharacterCategory::Lu); // 'Z'
        assert_eq!(map.category_for(0x61), CharacterCategory::Ll); // 'a'
        assert_eq!(map.category_for(0x7A), CharacterCategory::Ll); // 'z'
    }

    #[test]
    fn category_for_digits() {
        // Validates: Requirement 7.1
        let map = CharacterCategoryMap::new();
        assert_eq!(map.category_for(0x30), CharacterCategory::Nd); // '0'
        assert_eq!(map.category_for(0x39), CharacterCategory::Nd); // '9'
    }

    #[test]
    fn category_for_cjk() {
        // Validates: Requirement 7.1
        let map = CharacterCategoryMap::new();
        assert_eq!(map.category_for(0x4E2D), CharacterCategory::Lo); // '中'
        assert_eq!(map.category_for(0x65E5), CharacterCategory::Lo); // '日'
    }

    #[test]
    fn category_for_cyrillic() {
        // Validates: Requirement 7.1
        let map = CharacterCategoryMap::new();
        assert_eq!(map.category_for(0x0410), CharacterCategory::Lu); // 'А'
        assert_eq!(map.category_for(0x0430), CharacterCategory::Ll); // 'а'
    }

    #[test]
    fn category_for_supplementary_emoji() {
        // Validates: Requirement 7.2
        let map = CharacterCategoryMap::new();
        assert_eq!(map.category_for(0x1F600), CharacterCategory::So); // 😀
    }

    #[test]
    fn is_id_start_letters_only() {
        // Validates: Requirement 7.4
        let map = CharacterCategoryMap::new();
        assert!(map.is_id_start(0x41)); // 'A'
        assert!(map.is_id_start(0x61)); // 'a'
        assert!(!map.is_id_start(0x30)); // '0' - digit not valid at start
        assert!(!map.is_id_start(0x5F)); // '_' - Pc not in ID_Start
    }

    #[test]
    fn is_id_continue_includes_digits_and_underscore() {
        // Validates: Requirement 7.4
        let map = CharacterCategoryMap::new();
        assert!(map.is_id_continue(0x41)); // 'A'
        assert!(map.is_id_continue(0x30)); // '0'
        assert!(map.is_id_continue(0x5F)); // '_'
    }

    #[test]
    fn is_word_char_classification() {
        // Validates: Requirement 7.6
        let map = CharacterCategoryMap::new();
        assert!(map.is_word_char(0x41)); // 'A'
        assert!(map.is_word_char(0x61)); // 'a'
        assert!(map.is_word_char(0x30)); // '0'
        assert!(map.is_word_char(0x5F)); // '_'
        assert!(!map.is_word_char(0x20)); // ' '
        assert!(!map.is_word_char(0x21)); // '!'
    }

    #[test]
    fn optimize_extends_dense_array() {
        // Validates: Requirement 7.3
        let mut map = CharacterCategoryMap::new();
        assert_eq!(map.bmp_table.len(), 65536);
        map.optimize(0x20000);
        assert!(map.bmp_table.len() >= 0x20000);
    }
}
