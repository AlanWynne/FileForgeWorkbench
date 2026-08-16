//! DBCS (Double-Byte Character Set) code page support.
//!
//! Provides lead/trail byte detection, safe segmentation, and fold maps
//! for Shift-JIS, GBK, EUC-KR, Big5, and Johab.

use std::collections::HashMap;

/// Supported DBCS code pages.
///
/// [Requirement 8]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DbcsCodePage {
    /// Shift-JIS (CP932)
    ShiftJis = 932,
    /// GBK (CP936)
    Gbk = 936,
    /// Korean Wansung (CP949)
    KoreanWansung = 949,
    /// Big5 (CP950)
    Big5 = 950,
    /// Korean Johab (CP1361)
    KoreanJohab = 1361,
}

/// DBCS code page definition with lead/trail byte ranges.
///
/// [Requirement 8]
#[derive(Debug, Clone)]
pub struct DbcsCodePageDef {
    /// The code page this definition is for
    pub code_page: DbcsCodePage,
    /// Inclusive byte ranges that are valid lead bytes
    pub lead_byte_ranges: &'static [(u8, u8)],
    /// Inclusive byte ranges that are valid trail bytes
    pub trail_byte_ranges: &'static [(u8, u8)],
    /// Bytes that are valid single-byte characters in the DBCS encoding
    pub single_byte_ranges: &'static [(u8, u8)],
}

/// Lead/trail byte range definitions for each code page.
static SHIFT_JIS_DEF: DbcsCodePageDef = DbcsCodePageDef {
    code_page: DbcsCodePage::ShiftJis,
    lead_byte_ranges: &[(0x81, 0x9F), (0xE0, 0xFC)],
    trail_byte_ranges: &[(0x40, 0x7E), (0x80, 0xFC)],
    single_byte_ranges: &[(0xA1, 0xDF)], // Half-width katakana
};

static GBK_DEF: DbcsCodePageDef = DbcsCodePageDef {
    code_page: DbcsCodePage::Gbk,
    lead_byte_ranges: &[(0x81, 0xFE)],
    trail_byte_ranges: &[(0x40, 0x7E), (0x80, 0xFE)],
    single_byte_ranges: &[],
};

static KOREAN_WANSUNG_DEF: DbcsCodePageDef = DbcsCodePageDef {
    code_page: DbcsCodePage::KoreanWansung,
    lead_byte_ranges: &[(0x81, 0xFE)],
    trail_byte_ranges: &[(0x41, 0x5A), (0x61, 0x7A), (0x81, 0xFE)],
    single_byte_ranges: &[],
};

static BIG5_DEF: DbcsCodePageDef = DbcsCodePageDef {
    code_page: DbcsCodePage::Big5,
    lead_byte_ranges: &[(0x81, 0xFE)],
    trail_byte_ranges: &[(0x40, 0x7E), (0xA1, 0xFE)],
    single_byte_ranges: &[],
};

static KOREAN_JOHAB_DEF: DbcsCodePageDef = DbcsCodePageDef {
    code_page: DbcsCodePage::KoreanJohab,
    lead_byte_ranges: &[(0x84, 0xD3), (0xD8, 0xDE), (0xE0, 0xF9)],
    trail_byte_ranges: &[(0x31, 0x7E), (0x81, 0xFE)],
    single_byte_ranges: &[],
};

/// Get the code page definition for a given DBCS code page.
fn get_def(code_page: DbcsCodePage) -> &'static DbcsCodePageDef {
    match code_page {
        DbcsCodePage::ShiftJis => &SHIFT_JIS_DEF,
        DbcsCodePage::Gbk => &GBK_DEF,
        DbcsCodePage::KoreanWansung => &KOREAN_WANSUNG_DEF,
        DbcsCodePage::Big5 => &BIG5_DEF,
        DbcsCodePage::KoreanJohab => &KOREAN_JOHAB_DEF,
    }
}

/// Is the given code page a supported DBCS code page?
///
/// [Requirement 8.2]
pub fn is_dbcs_code_page(code_page: u32) -> bool {
    matches!(code_page, 932 | 936 | 949 | 950 | 1361)
}

/// Is the byte a lead byte for the given DBCS code page?
///
/// [Requirement 8.3]
pub fn dbcs_is_lead_byte(code_page: DbcsCodePage, byte: u8) -> bool {
    let def = get_def(code_page);
    def.lead_byte_ranges
        .iter()
        .any(|&(lo, hi)| byte >= lo && byte <= hi)
}

/// Is the byte a trail byte for the given DBCS code page?
///
/// [Requirement 8.4]
pub fn dbcs_is_trail_byte(code_page: DbcsCodePage, byte: u8) -> bool {
    let def = get_def(code_page);
    def.trail_byte_ranges
        .iter()
        .any(|&(lo, hi)| byte >= lo && byte <= hi)
}

/// Is the byte a valid single-byte character in the DBCS encoding?
///
/// [Requirement 8.5]
pub fn is_dbcs_valid_single_byte(code_page: DbcsCodePage, byte: u8) -> bool {
    // ASCII is always single-byte
    if byte < 0x80 {
        return true;
    }

    let def = get_def(code_page);
    def.single_byte_ranges
        .iter()
        .any(|&(lo, hi)| byte >= lo && byte <= hi)
}

/// Return the longest prefix of `data` that ends on a character boundary.
///
/// [Requirement 8.6]
pub fn safe_segment(data: &[u8], code_page: DbcsCodePage) -> &[u8] {
    if data.is_empty() {
        return data;
    }

    let mut pos = 0;
    let mut last_safe = 0;

    while pos < data.len() {
        let byte = data[pos];
        if byte < 0x80 {
            // ASCII — always single byte
            pos += 1;
            last_safe = pos;
        } else if dbcs_is_lead_byte(code_page, byte) {
            if pos + 1 < data.len() && dbcs_is_trail_byte(code_page, data[pos + 1]) {
                pos += 2;
                last_safe = pos;
            } else {
                // Incomplete double-byte character at end
                break;
            }
        } else if is_dbcs_valid_single_byte(code_page, byte) {
            pos += 1;
            last_safe = pos;
        } else {
            // Unknown byte — treat as single byte
            pos += 1;
            last_safe = pos;
        }
    }

    &data[..last_safe]
}

/// DBCS fold map for case-insensitive search in DBCS content.
///
/// Maps full-width uppercase Latin letters to their lowercase equivalents.
///
/// [Requirement 8.7]
#[derive(Debug, Clone)]
pub struct DBCSFoldMap {
    /// Mapping from double-byte sequence to its case-folded equivalent
    map: HashMap<[u8; 2], [u8; 2]>,
}

impl DBCSFoldMap {
    /// Create a new fold map for the given code page.
    pub fn new(_code_page: DbcsCodePage) -> Self {
        // Simplified: empty map for now
        // Full implementation would contain full-width Latin uppercase → lowercase mappings
        Self {
            map: HashMap::new(),
        }
    }

    /// Look up the case-folded equivalent of a double-byte sequence.
    pub fn fold(&self, bytes: [u8; 2]) -> [u8; 2] {
        self.map.get(&bytes).copied().unwrap_or(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_dbcs_code_page_returns_true_for_supported() {
        // Validates: Requirement 8.2
        assert!(is_dbcs_code_page(932));
        assert!(is_dbcs_code_page(936));
        assert!(is_dbcs_code_page(949));
        assert!(is_dbcs_code_page(950));
        assert!(is_dbcs_code_page(1361));
    }

    #[test]
    fn is_dbcs_code_page_returns_false_for_unsupported() {
        // Validates: Requirement 8.2
        assert!(!is_dbcs_code_page(1252));
        assert!(!is_dbcs_code_page(65001));
        assert!(!is_dbcs_code_page(28591));
        assert!(!is_dbcs_code_page(0));
    }

    #[test]
    fn shift_jis_lead_byte_ranges() {
        // Validates: Requirement 8.3
        assert!(dbcs_is_lead_byte(DbcsCodePage::ShiftJis, 0x81));
        assert!(dbcs_is_lead_byte(DbcsCodePage::ShiftJis, 0x9F));
        assert!(dbcs_is_lead_byte(DbcsCodePage::ShiftJis, 0xE0));
        assert!(dbcs_is_lead_byte(DbcsCodePage::ShiftJis, 0xFC));
        assert!(!dbcs_is_lead_byte(DbcsCodePage::ShiftJis, 0x80));
        assert!(!dbcs_is_lead_byte(DbcsCodePage::ShiftJis, 0xA0));
        assert!(!dbcs_is_lead_byte(DbcsCodePage::ShiftJis, 0x7F));
    }

    #[test]
    fn shift_jis_trail_byte_ranges() {
        // Validates: Requirement 8.4
        assert!(dbcs_is_trail_byte(DbcsCodePage::ShiftJis, 0x40));
        assert!(dbcs_is_trail_byte(DbcsCodePage::ShiftJis, 0x7E));
        assert!(dbcs_is_trail_byte(DbcsCodePage::ShiftJis, 0x80));
        assert!(dbcs_is_trail_byte(DbcsCodePage::ShiftJis, 0xFC));
        assert!(!dbcs_is_trail_byte(DbcsCodePage::ShiftJis, 0x3F));
    }

    #[test]
    fn shift_jis_half_width_katakana_is_single_byte() {
        // Validates: Requirement 8.5
        assert!(is_dbcs_valid_single_byte(DbcsCodePage::ShiftJis, 0xA1));
        assert!(is_dbcs_valid_single_byte(DbcsCodePage::ShiftJis, 0xDF));
        assert!(!is_dbcs_valid_single_byte(DbcsCodePage::ShiftJis, 0xA0));
    }

    #[test]
    fn ascii_is_always_valid_single_byte() {
        // Validates: Requirement 8.5
        for cp in [
            DbcsCodePage::ShiftJis,
            DbcsCodePage::Gbk,
            DbcsCodePage::Big5,
        ] {
            for byte in 0..0x80u8 {
                assert!(
                    is_dbcs_valid_single_byte(cp, byte),
                    "ASCII byte 0x{byte:02X} should be valid single byte for {cp:?}"
                );
            }
        }
    }

    #[test]
    fn gbk_lead_byte_ranges() {
        // Validates: Requirement 8.3
        assert!(dbcs_is_lead_byte(DbcsCodePage::Gbk, 0x81));
        assert!(dbcs_is_lead_byte(DbcsCodePage::Gbk, 0xFE));
        assert!(!dbcs_is_lead_byte(DbcsCodePage::Gbk, 0x80));
        assert!(!dbcs_is_lead_byte(DbcsCodePage::Gbk, 0xFF));
    }

    #[test]
    fn no_ascii_byte_is_lead_byte() {
        // Validates: Requirement 8.2, 8.3
        for cp in [
            DbcsCodePage::ShiftJis,
            DbcsCodePage::Gbk,
            DbcsCodePage::KoreanWansung,
            DbcsCodePage::Big5,
            DbcsCodePage::KoreanJohab,
        ] {
            for byte in 0..0x80u8 {
                assert!(
                    !dbcs_is_lead_byte(cp, byte),
                    "ASCII byte 0x{byte:02X} should NOT be a lead byte for {cp:?}"
                );
            }
        }
    }

    #[test]
    fn safe_segment_complete_sequences() {
        // Validates: Requirement 8.6
        // "AB" in ASCII + a Shift-JIS double-byte char (0x82 0xA0 = 'あ')
        let data = [b'A', b'B', 0x82, 0xA0];
        let segment = safe_segment(&data, DbcsCodePage::ShiftJis);
        assert_eq!(segment, &data); // All complete
    }

    #[test]
    fn safe_segment_truncates_incomplete() {
        // Validates: Requirement 8.6
        // "A" + incomplete double-byte (lead byte only)
        let data = [b'A', 0x82];
        let segment = safe_segment(&data, DbcsCodePage::ShiftJis);
        assert_eq!(segment, b"A"); // Truncate the incomplete pair
    }

    #[test]
    fn safe_segment_empty() {
        // Validates: Requirement 8.6
        let data: &[u8] = &[];
        let segment = safe_segment(data, DbcsCodePage::ShiftJis);
        assert!(segment.is_empty());
    }
}
