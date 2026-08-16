//! Grapheme cluster boundary detection (UAX #29).
//!
//! Provides boundary testing, forward/backward iteration, and a
//! grapheme cluster iterator supporting both strict and simplified modes.

/// Mode for grapheme cluster detection.
///
/// [Requirement 9.8]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphemeMode {
    /// Full UAX #29 grapheme clustering
    Strict,
    /// Code-point-level navigation only (performance mode)
    Simplified,
}

/// Iterator over grapheme cluster boundaries in a UTF-8 string.
///
/// [Requirement 9]
#[derive(Debug)]
pub struct GraphemeIterator<'a> {
    text: &'a str,
    position: usize,
    mode: GraphemeMode,
}

impl<'a> GraphemeIterator<'a> {
    /// Create a new grapheme iterator over the given text.
    pub fn new(text: &'a str, mode: GraphemeMode) -> Self {
        Self {
            text,
            position: 0,
            mode,
        }
    }
}

impl<'a> Iterator for GraphemeIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.text.len() {
            return None;
        }

        let start = self.position;
        let next_boundary = next_grapheme_boundary_impl(self.text, start, self.mode);
        self.position = next_boundary;

        Some(&self.text[start..next_boundary])
    }
}

/// Is the byte offset a grapheme cluster boundary in the given text?
///
/// [Requirement 9.2]
pub fn is_grapheme_boundary(text: &str, byte_offset: usize) -> bool {
    if byte_offset == 0 || byte_offset >= text.len() {
        return true;
    }

    // Must be on a char boundary first
    if !text.is_char_boundary(byte_offset) {
        return false;
    }

    // In strict mode, check if the preceding character and following character
    // form a grapheme cluster (combining marks, etc.)
    let before = &text[..byte_offset];
    let after = &text[byte_offset..];

    if let (Some(prev_char), Some(next_char)) = (before.chars().next_back(), after.chars().next()) {
        // Combining marks (Mn, Mc, Me) do not form boundaries after base chars
        if is_combining_mark(next_char) {
            return false;
        }

        // Regional indicators come in pairs
        if is_regional_indicator(prev_char) && is_regional_indicator(next_char) {
            // Count preceding regional indicators
            let ri_count = before
                .chars()
                .rev()
                .take_while(|&c| is_regional_indicator(c))
                .count();
            if ri_count % 2 == 1 {
                return false; // Odd count means we're in the middle of a pair
            }
        }

        // ZWJ sequences
        if prev_char == '\u{200D}' {
            // ZWJ joins the surrounding characters
            return false;
        }
        if next_char == '\u{200D}' && byte_offset + next_char.len_utf8() < text.len() {
            // ZWJ followed by something — not a boundary at ZWJ itself
            // (boundary check is at the position before ZWJ, which IS a boundary check target)
        }

        // Extend/SpacingMark characters (simplified)
        if is_extend_char(next_char) {
            return false;
        }
    }

    true
}

/// Return the byte offset of the next grapheme cluster boundary.
///
/// [Requirement 9.3]
pub fn next_grapheme_boundary(text: &str, byte_offset: usize) -> usize {
    next_grapheme_boundary_impl(text, byte_offset, GraphemeMode::Strict)
}

/// Return the byte offset of the previous grapheme cluster boundary.
///
/// [Requirement 9.4]
pub fn prev_grapheme_boundary(text: &str, byte_offset: usize) -> usize {
    prev_grapheme_boundary_impl(text, byte_offset, GraphemeMode::Strict)
}

/// Implementation of next_grapheme_boundary with mode support.
fn next_grapheme_boundary_impl(text: &str, byte_offset: usize, mode: GraphemeMode) -> usize {
    if byte_offset >= text.len() {
        return text.len();
    }

    match mode {
        GraphemeMode::Simplified => {
            // Just advance by one code point
            let remaining = &text[byte_offset..];
            if let Some(ch) = remaining.chars().next() {
                byte_offset + ch.len_utf8()
            } else {
                text.len()
            }
        }
        GraphemeMode::Strict => {
            // Advance past at least one code point, then find the next boundary
            let remaining = &text[byte_offset..];
            let mut chars = remaining.char_indices();

            // Skip the first char
            if let Some((_, first_ch)) = chars.next() {
                let mut pos = byte_offset + first_ch.len_utf8();

                // Keep advancing while we're not at a boundary
                while pos < text.len() && !is_grapheme_boundary(text, pos) {
                    if let Some(ch) = text[pos..].chars().next() {
                        pos += ch.len_utf8();
                    } else {
                        break;
                    }
                }

                pos
            } else {
                text.len()
            }
        }
    }
}

/// Implementation of prev_grapheme_boundary with mode support.
fn prev_grapheme_boundary_impl(text: &str, byte_offset: usize, mode: GraphemeMode) -> usize {
    if byte_offset == 0 {
        return 0;
    }

    let offset = byte_offset.min(text.len());

    match mode {
        GraphemeMode::Simplified => {
            // Just go back one code point
            let before = &text[..offset];
            if let Some(ch) = before.chars().next_back() {
                offset - ch.len_utf8()
            } else {
                0
            }
        }
        GraphemeMode::Strict => {
            // Go back at least one code point, then keep going while not at boundary
            let before = &text[..offset];
            let mut pos = offset;

            if let Some(ch) = before.chars().next_back() {
                pos -= ch.len_utf8();
            }

            // Keep going back while we're not at a boundary
            while pos > 0 && !is_grapheme_boundary(text, pos) {
                let before = &text[..pos];
                if let Some(ch) = before.chars().next_back() {
                    pos -= ch.len_utf8();
                } else {
                    break;
                }
            }

            pos
        }
    }
}

/// Check if a character is a combining mark (simplified).
fn is_combining_mark(ch: char) -> bool {
    let cp = ch as u32;
    // Combining Diacritical Marks (0300-036F)
    // Combining Diacritical Marks Extended (1AB0-1AFF)
    // Combining Diacritical Marks Supplement (1DC0-1DFF)
    // Combining Half Marks (FE20-FE2F)
    (0x0300..=0x036F).contains(&cp)
        || (0x1AB0..=0x1AFF).contains(&cp)
        || (0x1DC0..=0x1DFF).contains(&cp)
        || (0xFE20..=0xFE2F).contains(&cp)
        || (0x20D0..=0x20FF).contains(&cp) // Combining Diacritical Marks for Symbols
}

/// Check if a character is a regional indicator.
fn is_regional_indicator(ch: char) -> bool {
    let cp = ch as u32;
    (0x1F1E6..=0x1F1FF).contains(&cp)
}

/// Check if a character is an Extend grapheme property character (simplified).
fn is_extend_char(ch: char) -> bool {
    if is_combining_mark(ch) {
        return true;
    }
    let cp = ch as u32;
    // Emoji modifiers (skin tones)
    if (0x1F3FB..=0x1F3FF).contains(&cp) {
        return true;
    }
    // Variation selectors
    if (0xFE00..=0xFE0F).contains(&cp) || (0xE0100..=0xE01EF).contains(&cp) {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_each_char_is_boundary() {
        // Validates: Requirement 9.2
        let text = "Hello";
        assert!(is_grapheme_boundary(text, 0));
        assert!(is_grapheme_boundary(text, 1));
        assert!(is_grapheme_boundary(text, 2));
    }

    #[test]
    fn combining_mark_not_boundary() {
        // Validates: Requirement 9.5
        // 'e' + combining acute accent (U+0301) = "é"
        let text = "e\u{0301}";
        assert!(is_grapheme_boundary(text, 0)); // Before 'e'
        assert!(!is_grapheme_boundary(text, 1)); // Between 'e' and combining mark
    }

    #[test]
    fn next_grapheme_boundary_skips_combining_marks() {
        // Validates: Requirement 9.3, 9.5
        let text = "e\u{0301}x"; // é + x
        let next = next_grapheme_boundary(text, 0);
        // Should skip past 'e' + combining mark to 'x'
        assert_eq!(next, 3); // 'e' (1 byte) + U+0301 (2 bytes)
    }

    #[test]
    fn prev_grapheme_boundary_skips_combining_marks() {
        // Validates: Requirement 9.4, 9.5
        let text = "xe\u{0301}"; // x + é
        let prev = prev_grapheme_boundary(text, text.len());
        // Should go back past combining mark to start of 'e'
        assert_eq!(prev, 1); // After 'x'
    }

    #[test]
    fn next_boundary_always_advances() {
        // Validates: Requirement 9.3
        let text = "Hello";
        let mut pos = 0;
        while pos < text.len() {
            let next = next_grapheme_boundary(text, pos);
            assert!(next > pos, "next_grapheme_boundary must advance");
            pos = next;
        }
    }

    #[test]
    fn prev_boundary_always_retreats() {
        // Validates: Requirement 9.4
        let text = "Hello";
        let mut pos = text.len();
        while pos > 0 {
            let prev = prev_grapheme_boundary(text, pos);
            assert!(prev < pos, "prev_grapheme_boundary must retreat");
            pos = prev;
        }
    }

    #[test]
    fn grapheme_iterator_ascii() {
        // Validates: Requirement 9.1
        let text = "Hi";
        let graphemes: Vec<&str> = GraphemeIterator::new(text, GraphemeMode::Strict).collect();
        assert_eq!(graphemes, vec!["H", "i"]);
    }

    #[test]
    fn grapheme_iterator_combining_marks() {
        // Validates: Requirement 9.5
        let text = "e\u{0301}"; // é as base + combining
        let graphemes: Vec<&str> = GraphemeIterator::new(text, GraphemeMode::Strict).collect();
        assert_eq!(graphemes, vec!["e\u{0301}"]);
    }

    #[test]
    fn simplified_mode_navigates_by_code_point() {
        // Validates: Requirement 9.8
        let text = "e\u{0301}"; // é as base + combining
        let graphemes: Vec<&str> = GraphemeIterator::new(text, GraphemeMode::Simplified).collect();
        assert_eq!(graphemes.len(), 2); // Each code point is separate
    }

    #[test]
    fn regional_indicator_pairs_form_cluster() {
        // Validates: Requirement 9.6
        // US flag: U+1F1FA U+1F1F8
        let text = "\u{1F1FA}\u{1F1F8}";
        let next = next_grapheme_boundary(text, 0);
        assert_eq!(next, text.len()); // Whole flag is one cluster
    }

    #[test]
    fn boundary_at_start_and_end() {
        // Validates: Requirement 9.2
        let text = "test";
        assert!(is_grapheme_boundary(text, 0));
        assert!(is_grapheme_boundary(text, text.len()));
    }
}
