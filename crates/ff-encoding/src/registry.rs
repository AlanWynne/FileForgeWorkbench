//! Encoding registry providing lookup by name, code page, and alias.
//!
//! Pre-populated with all required encodings per the design specification.

use crate::encoding::{Encoding, EncodingFamily};

/// Registry of all supported encodings with lookup by name, code page, and alias.
///
/// [Requirement 14]
#[derive(Debug)]
pub struct EncodingRegistry {
    encodings: Vec<Encoding>,
}

impl EncodingRegistry {
    /// Create a new registry pre-populated with all supported encodings.
    pub fn new() -> Self {
        Self {
            encodings: all_encodings(),
        }
    }

    /// Look up an encoding by its canonical name (case-insensitive).
    pub fn by_name(&self, name: &str) -> Option<&Encoding> {
        let lower = name.to_ascii_lowercase();
        self.encodings
            .iter()
            .find(|e| e.name == lower || e.aliases.contains(&lower.as_str()))
    }

    /// Look up an encoding by its Windows code page number.
    pub fn by_code_page(&self, code_page: u32) -> Option<&Encoding> {
        self.encodings.iter().find(|e| e.code_page == code_page)
    }

    /// Look up an encoding by any alias (case-insensitive).
    pub fn by_alias(&self, alias: &str) -> Option<&Encoding> {
        let lower = alias.to_ascii_lowercase();
        self.encodings.iter().find(|e| {
            e.name == lower
                || e.display_name.to_ascii_lowercase() == lower
                || e.aliases.contains(&lower.as_str())
        })
    }

    /// Returns an iterator over all registered encodings.
    pub fn all(&self) -> impl Iterator<Item = &Encoding> {
        self.encodings.iter()
    }

    /// Returns the number of registered encodings.
    pub fn count(&self) -> usize {
        self.encodings.len()
    }
}

impl Default for EncodingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Construct the complete list of supported encodings.
fn all_encodings() -> Vec<Encoding> {
    vec![
        // UTF-8
        Encoding {
            name: "utf-8",
            code_page: 65001,
            family: EncodingFamily::Utf8,
            display_name: "UTF-8",
            aliases: &["utf8", "unicode-1-1-utf-8"],
        },
        // UTF-16LE
        Encoding {
            name: "utf-16le",
            code_page: 1200,
            family: EncodingFamily::Utf16,
            display_name: "UTF-16 LE",
            aliases: &["utf16le", "ucs-2le"],
        },
        // UTF-16BE
        Encoding {
            name: "utf-16be",
            code_page: 1201,
            family: EncodingFamily::Utf16,
            display_name: "UTF-16 BE",
            aliases: &["utf16be", "ucs-2be"],
        },
        // UTF-32LE
        Encoding {
            name: "utf-32le",
            code_page: 12000,
            family: EncodingFamily::Utf16,
            display_name: "UTF-32 LE",
            aliases: &["utf32le", "ucs-4le"],
        },
        // UTF-32BE
        Encoding {
            name: "utf-32be",
            code_page: 12001,
            family: EncodingFamily::Utf16,
            display_name: "UTF-32 BE",
            aliases: &["utf32be", "ucs-4be"],
        },
        // ISO-8859-1 through ISO-8859-15
        Encoding {
            name: "iso-8859-1",
            code_page: 28591,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-1 (Latin-1)",
            aliases: &["latin1", "iso8859-1", "iso88591"],
        },
        Encoding {
            name: "iso-8859-2",
            code_page: 28592,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-2 (Latin-2)",
            aliases: &["latin2", "iso8859-2"],
        },
        Encoding {
            name: "iso-8859-3",
            code_page: 28593,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-3 (Latin-3)",
            aliases: &["latin3", "iso8859-3"],
        },
        Encoding {
            name: "iso-8859-4",
            code_page: 28594,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-4 (Latin-4)",
            aliases: &["latin4", "iso8859-4"],
        },
        Encoding {
            name: "iso-8859-5",
            code_page: 28595,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-5 (Cyrillic)",
            aliases: &["iso8859-5", "cyrillic"],
        },
        Encoding {
            name: "iso-8859-6",
            code_page: 28596,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-6 (Arabic)",
            aliases: &["iso8859-6", "arabic"],
        },
        Encoding {
            name: "iso-8859-7",
            code_page: 28597,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-7 (Greek)",
            aliases: &["iso8859-7", "greek"],
        },
        Encoding {
            name: "iso-8859-8",
            code_page: 28598,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-8 (Hebrew)",
            aliases: &["iso8859-8", "hebrew"],
        },
        Encoding {
            name: "iso-8859-9",
            code_page: 28599,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-9 (Turkish)",
            aliases: &["latin5", "iso8859-9"],
        },
        Encoding {
            name: "iso-8859-10",
            code_page: 28600,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-10 (Nordic)",
            aliases: &["latin6", "iso8859-10"],
        },
        Encoding {
            name: "iso-8859-11",
            code_page: 28601,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-11 (Thai)",
            aliases: &["iso8859-11", "thai"],
        },
        Encoding {
            name: "iso-8859-13",
            code_page: 28603,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-13 (Baltic)",
            aliases: &["latin7", "iso8859-13"],
        },
        Encoding {
            name: "iso-8859-14",
            code_page: 28604,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-14 (Celtic)",
            aliases: &["latin8", "iso8859-14"],
        },
        Encoding {
            name: "iso-8859-15",
            code_page: 28605,
            family: EncodingFamily::SingleByte,
            display_name: "ISO 8859-15 (Latin-9)",
            aliases: &["latin9", "iso8859-15"],
        },
        // Windows code pages 1250-1258
        Encoding {
            name: "windows-1250",
            code_page: 1250,
            family: EncodingFamily::SingleByte,
            display_name: "Windows-1250 (Central European)",
            aliases: &["cp1250", "win1250"],
        },
        Encoding {
            name: "windows-1251",
            code_page: 1251,
            family: EncodingFamily::SingleByte,
            display_name: "Windows-1251 (Cyrillic)",
            aliases: &["cp1251", "win1251"],
        },
        Encoding {
            name: "windows-1252",
            code_page: 1252,
            family: EncodingFamily::SingleByte,
            display_name: "Windows-1252 (Western)",
            aliases: &["cp1252", "win1252"],
        },
        Encoding {
            name: "windows-1253",
            code_page: 1253,
            family: EncodingFamily::SingleByte,
            display_name: "Windows-1253 (Greek)",
            aliases: &["cp1253", "win1253"],
        },
        Encoding {
            name: "windows-1254",
            code_page: 1254,
            family: EncodingFamily::SingleByte,
            display_name: "Windows-1254 (Turkish)",
            aliases: &["cp1254", "win1254"],
        },
        Encoding {
            name: "windows-1255",
            code_page: 1255,
            family: EncodingFamily::SingleByte,
            display_name: "Windows-1255 (Hebrew)",
            aliases: &["cp1255", "win1255"],
        },
        Encoding {
            name: "windows-1256",
            code_page: 1256,
            family: EncodingFamily::SingleByte,
            display_name: "Windows-1256 (Arabic)",
            aliases: &["cp1256", "win1256"],
        },
        Encoding {
            name: "windows-1257",
            code_page: 1257,
            family: EncodingFamily::SingleByte,
            display_name: "Windows-1257 (Baltic)",
            aliases: &["cp1257", "win1257"],
        },
        Encoding {
            name: "windows-1258",
            code_page: 1258,
            family: EncodingFamily::SingleByte,
            display_name: "Windows-1258 (Vietnamese)",
            aliases: &["cp1258", "win1258"],
        },
        // DBCS encodings
        Encoding {
            name: "shift-jis",
            code_page: 932,
            family: EncodingFamily::Dbcs,
            display_name: "Shift-JIS",
            aliases: &["shift_jis", "sjis", "cp932", "ms932"],
        },
        Encoding {
            name: "gbk",
            code_page: 936,
            family: EncodingFamily::Dbcs,
            display_name: "GBK",
            aliases: &["cp936", "ms936", "gb2312"],
        },
        Encoding {
            name: "euc-kr",
            code_page: 949,
            family: EncodingFamily::Dbcs,
            display_name: "EUC-KR",
            aliases: &["cp949", "korean", "ksc5601"],
        },
        Encoding {
            name: "big5",
            code_page: 950,
            family: EncodingFamily::Dbcs,
            display_name: "Big5",
            aliases: &["cp950", "big-5", "csbig5"],
        },
        Encoding {
            name: "euc-jp",
            code_page: 20932,
            family: EncodingFamily::Dbcs,
            display_name: "EUC-JP",
            aliases: &["eucjp", "x-euc-jp"],
        },
        Encoding {
            name: "johab",
            code_page: 1361,
            family: EncodingFamily::Dbcs,
            display_name: "Johab (Korean)",
            aliases: &["cp1361"],
        },
        // EBCDIC code pages
        Encoding {
            name: "ebcdic-037",
            code_page: 37,
            family: EncodingFamily::SingleByte,
            display_name: "EBCDIC CP037 (US/Canada)",
            aliases: &["cp037", "ibm037"],
        },
        Encoding {
            name: "ebcdic-500",
            code_page: 500,
            family: EncodingFamily::SingleByte,
            display_name: "EBCDIC CP500 (International)",
            aliases: &["cp500", "ibm500"],
        },
        Encoding {
            name: "ebcdic-1047",
            code_page: 1047,
            family: EncodingFamily::SingleByte,
            display_name: "EBCDIC CP1047 (Latin-1/Open Systems)",
            aliases: &["cp1047", "ibm1047"],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_lookup_by_name_utf8() {
        // Validates: Requirement 14.7, 14.8
        let registry = EncodingRegistry::new();
        let enc = registry.by_name("utf-8").expect("utf-8 should exist");
        assert_eq!(enc.code_page, 65001);
        assert_eq!(enc.family, EncodingFamily::Utf8);
    }

    #[test]
    fn registry_lookup_by_name_case_insensitive() {
        // Validates: Requirement 14.8
        let registry = EncodingRegistry::new();
        assert!(registry.by_name("UTF-8").is_some());
        assert!(registry.by_name("Shift-JIS").is_some());
    }

    #[test]
    fn registry_lookup_by_alias() {
        // Validates: Requirement 14.8
        let registry = EncodingRegistry::new();
        let enc = registry
            .by_alias("sjis")
            .expect("sjis alias should resolve");
        assert_eq!(enc.name, "shift-jis");
        assert_eq!(enc.code_page, 932);
    }

    #[test]
    fn registry_lookup_by_code_page() {
        // Validates: Requirement 14.7
        let registry = EncodingRegistry::new();
        let enc = registry.by_code_page(1252).expect("CP1252 should exist");
        assert_eq!(enc.name, "windows-1252");
        assert_eq!(enc.family, EncodingFamily::SingleByte);
    }

    #[test]
    fn registry_lookup_by_code_page_dbcs() {
        // Validates: Requirement 14.7
        let registry = EncodingRegistry::new();
        let enc = registry.by_code_page(932).expect("CP932 should exist");
        assert_eq!(enc.name, "shift-jis");
        assert_eq!(enc.family, EncodingFamily::Dbcs);
    }

    #[test]
    fn registry_lookup_by_code_page_ebcdic() {
        // Validates: Requirement 14.7
        let registry = EncodingRegistry::new();
        let enc = registry.by_code_page(37).expect("CP037 should exist");
        assert_eq!(enc.name, "ebcdic-037");
        assert_eq!(enc.family, EncodingFamily::SingleByte);
    }

    #[test]
    fn registry_lookup_nonexistent_returns_none() {
        let registry = EncodingRegistry::new();
        assert!(registry.by_name("nonexistent").is_none());
        assert!(registry.by_code_page(99999).is_none());
    }

    #[test]
    fn registry_contains_all_required_encodings() {
        // Validates: Requirement 14.7
        let registry = EncodingRegistry::new();
        // UTF family
        assert!(registry.by_name("utf-8").is_some());
        assert!(registry.by_name("utf-16le").is_some());
        assert!(registry.by_name("utf-16be").is_some());
        assert!(registry.by_name("utf-32le").is_some());
        assert!(registry.by_name("utf-32be").is_some());
        // ISO-8859 family
        assert!(registry.by_name("iso-8859-1").is_some());
        assert!(registry.by_name("iso-8859-15").is_some());
        // Windows family
        assert!(registry.by_name("windows-1250").is_some());
        assert!(registry.by_name("windows-1258").is_some());
        // DBCS
        assert!(registry.by_name("shift-jis").is_some());
        assert!(registry.by_name("gbk").is_some());
        assert!(registry.by_name("euc-kr").is_some());
        assert!(registry.by_name("big5").is_some());
        assert!(registry.by_name("euc-jp").is_some());
        // EBCDIC
        assert!(registry.by_name("ebcdic-037").is_some());
        assert!(registry.by_name("ebcdic-500").is_some());
        assert!(registry.by_name("ebcdic-1047").is_some());
    }

    #[test]
    fn registry_encoding_family_classification_consistent() {
        // Validates: Requirement 11.1, 11.2
        let registry = EncodingRegistry::new();
        for enc in registry.all() {
            let computed = crate::encoding::encoding_family(enc.code_page);
            assert_eq!(
                enc.family, computed,
                "Family mismatch for '{}' (CP{}): stored {:?} vs computed {:?}",
                enc.name, enc.code_page, enc.family, computed
            );
        }
    }
}
