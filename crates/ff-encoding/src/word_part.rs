//! Word-part (sub-word) boundary detection for camelCase/snake_case navigation.
//!
//! Supports boundaries at: underscores, lower→upper transitions,
//! uppercase-sequence→lowercase transitions, and letter↔digit transitions.

use crate::classify::CharClassify;

/// Is this code point a word-part separator?
///
/// True for underscore characters. Case transitions are detected by context,
/// not by this single-character predicate.
///
/// [Requirement 12.1]
pub fn is_word_part_separator(code_point: u32) -> bool {
    code_point == '_' as u32
}

/// Find the start of the previous word-part to the left of `position`.
///
/// Respects camelCase, snake_case, PascalCase, and digit transitions.
/// Characters not classified as Word act as hard boundaries.
///
/// [Requirement 12.2]
pub fn word_part_left(text: &str, position: usize, classify: &CharClassify) -> usize {
    if position == 0 {
        return 0;
    }

    let bytes = text.as_bytes();
    let mut pos = position.min(text.len());

    // If immediately to the left are non-word characters, skip them
    if pos > 0 && !is_word_byte(bytes[pos - 1], classify) {
        while pos > 0 && !is_word_byte(bytes[pos - 1], classify) {
            pos -= 1;
        }
        return pos;
    }

    if pos == 0 {
        return 0;
    }

    // Now we're at a word character. Determine what kind and find the start of this part.
    let end_byte = bytes[pos - 1];

    if end_byte == b'_' {
        // Skip underscores
        while pos > 0 && bytes[pos - 1] == b'_' {
            pos -= 1;
        }
        return pos;
    }

    if end_byte.is_ascii_digit() {
        // Skip digits
        while pos > 0 && bytes[pos - 1].is_ascii_digit() {
            pos -= 1;
        }
        return pos;
    }

    if end_byte.is_ascii_lowercase() || is_unicode_lowercase(end_byte) {
        // Skip lowercase letters
        while pos > 0
            && (bytes[pos - 1].is_ascii_lowercase() || is_unicode_lowercase(bytes[pos - 1]))
        {
            pos -= 1;
        }

        // If preceded by exactly one uppercase (camelCase boundary), include it
        if pos > 0 && (bytes[pos - 1].is_ascii_uppercase() || is_unicode_uppercase(bytes[pos - 1]))
        {
            // Check if there's a run of uppercase before
            let upper_start = pos - 1;
            let mut check = upper_start;
            while check > 0
                && (bytes[check - 1].is_ascii_uppercase() || is_unicode_uppercase(bytes[check - 1]))
            {
                check -= 1;
            }
            let upper_run_len = upper_start - check + 1;
            if upper_run_len == 1 {
                // Single uppercase before lowercase = camelCase start (e.g., 'G' in "getDoc")
                pos -= 1;
            } else {
                // Multiple uppercase before lowercase = acronym end
                // e.g., "XMLParser" — 'P' starts a new part, boundary is before 'P'
                pos -= 1; // Include the last uppercase of the acronym that transitions
            }
        }

        return pos;
    }

    if end_byte.is_ascii_uppercase() || is_unicode_uppercase(end_byte) {
        // Skip uppercase letters
        while pos > 0
            && (bytes[pos - 1].is_ascii_uppercase() || is_unicode_uppercase(bytes[pos - 1]))
        {
            pos -= 1;
        }
        return pos;
    }

    // High bytes (0x80+) — treat as word chars, skip them
    while pos > 0 && bytes[pos - 1] >= 0x80 {
        pos -= 1;
    }

    pos
}

/// Find the start of the next word-part to the right of `position`.
///
/// [Requirement 12.3]
pub fn word_part_right(text: &str, position: usize, classify: &CharClassify) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut pos = position.min(len);

    if pos >= len {
        return len;
    }

    // If starting at non-word characters, skip them and return
    if !is_word_byte(bytes[pos], classify) {
        while pos < len && !is_word_byte(bytes[pos], classify) {
            pos += 1;
        }
        return pos;
    }

    let start_byte = bytes[pos];

    if start_byte == b'_' {
        // Skip underscores
        while pos < len && bytes[pos] == b'_' {
            pos += 1;
        }
        return pos;
    }

    if start_byte.is_ascii_digit() {
        // Skip digits
        while pos < len && bytes[pos].is_ascii_digit() {
            pos += 1;
        }
        return pos;
    }

    if start_byte.is_ascii_uppercase() || is_unicode_uppercase(start_byte) {
        // Check for acronym (run of uppercase)
        let mut upper_count = 0;
        while pos < len && (bytes[pos].is_ascii_uppercase() || is_unicode_uppercase(bytes[pos])) {
            pos += 1;
            upper_count += 1;
        }

        if upper_count > 1
            && pos < len
            && (bytes[pos].is_ascii_lowercase() || is_unicode_lowercase(bytes[pos]))
        {
            // Acronym followed by lowercase: boundary before the last uppercase
            // e.g., "XMLParser" → "XML" then "Parser"
            pos -= 1;
            return pos;
        }

        if upper_count == 1
            && pos < len
            && (bytes[pos].is_ascii_lowercase() || is_unicode_lowercase(bytes[pos]))
        {
            // Single uppercase followed by lowercase: this is a camelCase word part start
            // e.g., "Document" in "getDocumentModel"
            while pos < len {
                let b = bytes[pos];
                if b.is_ascii_uppercase()
                    || is_unicode_uppercase(b)
                    || b.is_ascii_digit()
                    || b == b'_'
                    || !is_word_byte(b, classify)
                {
                    break;
                }
                pos += 1;
            }
        }

        return pos;
    }

    if start_byte.is_ascii_lowercase() || is_unicode_lowercase(start_byte) {
        // Skip lowercase until uppercase, digit, underscore, or non-word
        while pos < len {
            let b = bytes[pos];
            if b.is_ascii_uppercase()
                || is_unicode_uppercase(b)
                || b.is_ascii_digit()
                || b == b'_'
                || !is_word_byte(b, classify)
            {
                break;
            }
            pos += 1;
        }
        return pos;
    }

    // High bytes — skip
    while pos < len && bytes[pos] >= 0x80 {
        pos += 1;
    }

    pos
}

/// Check if a byte is classified as a word character.
fn is_word_byte(byte: u8, classify: &CharClassify) -> bool {
    classify.is_word(byte)
}

/// Simplified check for Unicode uppercase (ASCII range only for byte-level).
fn is_unicode_uppercase(byte: u8) -> bool {
    byte.is_ascii_uppercase()
}

/// Simplified check for Unicode lowercase (ASCII range only for byte-level).
fn is_unicode_lowercase(byte: u8) -> bool {
    byte.is_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_classify() -> CharClassify {
        CharClassify::new(true)
    }

    #[test]
    fn is_word_part_separator_underscore() {
        // Validates: Requirement 12.1
        assert!(is_word_part_separator('_' as u32));
        assert!(!is_word_part_separator('A' as u32));
        assert!(!is_word_part_separator(' ' as u32));
    }

    #[test]
    fn word_part_right_camel_case() {
        // Validates: Requirement 12.3, 12.4
        let text = "getDocumentModel";
        let classify = default_classify();

        let p1 = word_part_right(text, 0, &classify);
        assert_eq!(&text[0..p1], "get");

        let p2 = word_part_right(text, p1, &classify);
        assert_eq!(&text[p1..p2], "Document");

        let p3 = word_part_right(text, p2, &classify);
        assert_eq!(&text[p2..p3], "Model");
    }

    #[test]
    fn word_part_right_snake_case() {
        // Validates: Requirement 12.3, 12.4
        let text = "get_document_model";
        let classify = default_classify();

        let p1 = word_part_right(text, 0, &classify);
        assert_eq!(&text[0..p1], "get");

        let p2 = word_part_right(text, p1, &classify);
        assert_eq!(&text[p1..p2], "_");

        let p3 = word_part_right(text, p2, &classify);
        assert_eq!(&text[p2..p3], "document");
    }

    #[test]
    fn word_part_right_pascal_case_acronym() {
        // Validates: Requirement 12.4
        let text = "XMLParser";
        let classify = default_classify();

        let p1 = word_part_right(text, 0, &classify);
        assert_eq!(&text[0..p1], "XML");

        let p2 = word_part_right(text, p1, &classify);
        assert_eq!(&text[p1..p2], "Parser");
    }

    #[test]
    fn word_part_right_digit_transitions() {
        // Validates: Requirement 12.4
        let text = "line42count";
        let classify = default_classify();

        let p1 = word_part_right(text, 0, &classify);
        assert_eq!(&text[0..p1], "line");

        let p2 = word_part_right(text, p1, &classify);
        assert_eq!(&text[p1..p2], "42");

        let p3 = word_part_right(text, p2, &classify);
        assert_eq!(&text[p2..p3], "count");
    }

    #[test]
    fn word_part_left_camel_case() {
        // Validates: Requirement 12.2
        let text = "getDocumentModel";
        let classify = default_classify();
        let len = text.len();

        let p1 = word_part_left(text, len, &classify);
        assert_eq!(&text[p1..len], "Model");

        let p2 = word_part_left(text, p1, &classify);
        assert_eq!(&text[p2..p1], "Document");

        let p3 = word_part_left(text, p2, &classify);
        assert_eq!(&text[p3..p2], "get");
        assert_eq!(p3, 0);
    }

    #[test]
    fn word_part_left_at_zero_returns_zero() {
        // Validates: Requirement 12.2
        let classify = default_classify();
        assert_eq!(word_part_left("test", 0, &classify), 0);
    }

    #[test]
    fn word_part_right_at_end_returns_end() {
        // Validates: Requirement 12.3
        let text = "test";
        let classify = default_classify();
        assert_eq!(word_part_right(text, text.len(), &classify), text.len());
    }

    #[test]
    fn word_part_respects_non_word_boundaries() {
        // Validates: Requirement 12.5
        let text = "hello world";
        let classify = default_classify();

        let p1 = word_part_right(text, 0, &classify);
        assert_eq!(&text[0..p1], "hello");

        // Skip space (non-word) and find next word part
        let p2 = word_part_right(text, p1, &classify);
        // After skipping the space, we're at "world" — returns end of "world"
        assert_eq!(&text[p1..p2], " ");

        let p3 = word_part_right(text, p2, &classify);
        assert_eq!(&text[p2..p3], "world");
    }

    #[test]
    fn word_part_left_always_le_position() {
        // Validates: Requirement 12.2
        let text = "getDocumentModel";
        let classify = default_classify();
        for pos in 0..=text.len() {
            let result = word_part_left(text, pos, &classify);
            assert!(
                result <= pos,
                "word_part_left({pos}) = {result}, expected <= {pos}"
            );
        }
    }

    #[test]
    fn word_part_right_always_ge_position() {
        // Validates: Requirement 12.3
        let text = "getDocumentModel";
        let classify = default_classify();
        for pos in 0..=text.len() {
            let result = word_part_right(text, pos, &classify);
            assert!(
                result >= pos,
                "word_part_right({pos}) = {result}, expected >= {pos}"
            );
        }
    }
}
