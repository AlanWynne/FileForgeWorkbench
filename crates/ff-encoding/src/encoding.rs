//! Core encoding types: `Encoding`, `EncodingFamily`, and `EncodingMetadata`.

/// A specific character encoding identified by name and code page.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Encoding {
    /// Canonical name (e.g., "utf-8", "shift-jis", "iso-8859-1")
    pub name: &'static str,
    /// Windows code page number (65001 for UTF-8)
    pub code_page: u32,
    /// The encoding family this belongs to
    pub family: EncodingFamily,
    /// Human-readable display name (e.g., "UTF-8", "Shift-JIS")
    pub display_name: &'static str,
    /// Alternative names for this encoding
    pub aliases: &'static [&'static str],
}

/// Classification of encodings into families that determine
/// how character boundaries are detected.
///
/// [Requirement 11]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EncodingFamily {
    /// ASCII, ISO-8859-x, Windows-125x, EBCDIC — 1 byte = 1 character
    SingleByte,
    /// UTF-8 — 1–4 bytes per character, lead/trail byte logic
    Utf8,
    /// Shift-JIS, GBK, Big5, Korean — 1–2 bytes per character
    Dbcs,
    /// UTF-16LE/BE — used for stream processing before conversion to UTF-8
    Utf16,
}

/// Metadata about a document's encoding state.
///
/// [Requirement 14]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodingMetadata {
    /// The active encoding
    pub encoding: Encoding,
    /// Whether a BOM was present in the original file
    pub has_bom: bool,
    /// Whether the content needs reload after encoding change
    pub needs_reload: bool,
}

/// Returns the `EncodingFamily` for a given code page number.
///
/// [Requirement 11.2]
pub fn encoding_family(code_page: u32) -> EncodingFamily {
    match code_page {
        65001 => EncodingFamily::Utf8,
        1200 | 1201 => EncodingFamily::Utf16,
        12000 | 12001 => EncodingFamily::Utf16, // UTF-32 treated as Utf16 family for processing
        932 | 936 | 949 | 950 | 1361 | 20932 => EncodingFamily::Dbcs,
        _ => EncodingFamily::SingleByte,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_family_utf8() {
        // Validates: Requirement 11.1, 11.2
        assert_eq!(encoding_family(65001), EncodingFamily::Utf8);
    }

    #[test]
    fn encoding_family_utf16() {
        // Validates: Requirement 11.1, 11.2
        assert_eq!(encoding_family(1200), EncodingFamily::Utf16);
        assert_eq!(encoding_family(1201), EncodingFamily::Utf16);
    }

    #[test]
    fn encoding_family_utf32_maps_to_utf16_processing() {
        // Validates: Requirement 11.1, 11.2
        assert_eq!(encoding_family(12000), EncodingFamily::Utf16);
        assert_eq!(encoding_family(12001), EncodingFamily::Utf16);
    }

    #[test]
    fn encoding_family_dbcs_code_pages() {
        // Validates: Requirement 11.1, 11.2
        assert_eq!(encoding_family(932), EncodingFamily::Dbcs);
        assert_eq!(encoding_family(936), EncodingFamily::Dbcs);
        assert_eq!(encoding_family(949), EncodingFamily::Dbcs);
        assert_eq!(encoding_family(950), EncodingFamily::Dbcs);
        assert_eq!(encoding_family(1361), EncodingFamily::Dbcs);
    }

    #[test]
    fn encoding_family_single_byte_iso_8859() {
        // Validates: Requirement 11.1, 11.2
        assert_eq!(encoding_family(28591), EncodingFamily::SingleByte); // ISO-8859-1
        assert_eq!(encoding_family(28592), EncodingFamily::SingleByte); // ISO-8859-2
        assert_eq!(encoding_family(28605), EncodingFamily::SingleByte); // ISO-8859-15
    }

    #[test]
    fn encoding_family_single_byte_windows() {
        // Validates: Requirement 11.1, 11.2
        assert_eq!(encoding_family(1250), EncodingFamily::SingleByte);
        assert_eq!(encoding_family(1252), EncodingFamily::SingleByte);
        assert_eq!(encoding_family(1258), EncodingFamily::SingleByte);
    }

    #[test]
    fn encoding_family_single_byte_ebcdic() {
        // Validates: Requirement 11.1, 11.2
        assert_eq!(encoding_family(37), EncodingFamily::SingleByte); // CP037
        assert_eq!(encoding_family(500), EncodingFamily::SingleByte); // CP500
        assert_eq!(encoding_family(1047), EncodingFamily::SingleByte); // CP1047
    }

    #[test]
    fn encoding_struct_fields() {
        // Validates: Requirement 14.7, 14.8
        let enc = Encoding {
            name: "utf-8",
            code_page: 65001,
            family: EncodingFamily::Utf8,
            display_name: "UTF-8",
            aliases: &["utf8", "unicode-1-1-utf-8"],
        };
        assert_eq!(enc.name, "utf-8");
        assert_eq!(enc.code_page, 65001);
        assert_eq!(enc.family, EncodingFamily::Utf8);
        assert_eq!(enc.display_name, "UTF-8");
        assert_eq!(enc.aliases, &["utf8", "unicode-1-1-utf-8"]);
    }

    #[test]
    fn encoding_metadata_construction() {
        // Validates: Requirement 14.7
        let meta = EncodingMetadata {
            encoding: Encoding {
                name: "utf-8",
                code_page: 65001,
                family: EncodingFamily::Utf8,
                display_name: "UTF-8",
                aliases: &[],
            },
            has_bom: true,
            needs_reload: false,
        };
        assert!(meta.has_bom);
        assert!(!meta.needs_reload);
    }
}
