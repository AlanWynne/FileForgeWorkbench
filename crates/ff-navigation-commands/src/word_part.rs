//! Word-part (camelCase / sub-word) navigation implementation.
//!
//! Detects sub-word boundaries within compound identifiers:
//! - camelCase transitions (lowerUpper)
//! - UPPER runs before lowercase (XMLParser → XML|Parser)
//! - alpha↔non-alpha transitions
//! - digit↔alpha transitions

use crate::types::{SelectionModifier, WordPartBoundary};

/// Word-part navigation executor.
pub struct WordPartNav;

impl WordPartNav {
    /// Move to the previous sub-word boundary (word-part-left).
    ///
    /// Detects boundaries at camelCase transitions, UPPER runs,
    /// alpha↔non-alpha, and digit↔alpha transitions.
    ///
    /// Returns the new column position (1-based) within the same word,
    /// or crosses to the previous word if at word start.
    pub fn word_part_left(text: &str, position: usize, _selection: SelectionModifier) -> usize {
        let chars: Vec<char> = text.chars().collect();
        if position == 0 || chars.is_empty() {
            return 0;
        }

        let pos = position.min(chars.len());

        // Move back at least one character
        let mut i = pos.saturating_sub(1);

        // Skip any separator/space characters at position
        while i > 0 && !chars[i].is_alphanumeric() {
            i -= 1;
        }

        if i == 0 {
            return 0;
        }

        // Now scan backwards to find a sub-word boundary
        while i > 0 {
            if Self::is_boundary_at(&chars, i) {
                return i;
            }
            i -= 1;
        }

        0
    }

    /// Move to the next sub-word boundary (word-part-right).
    ///
    /// Returns the new position (0-based offset into the string).
    pub fn word_part_right(text: &str, position: usize, _selection: SelectionModifier) -> usize {
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        if position >= len || chars.is_empty() {
            return len;
        }

        let mut i = position + 1;

        // Skip any non-alphanumeric at current position
        while i < len && !chars[i].is_alphanumeric() && !chars[i - 1].is_alphanumeric() {
            i += 1;
        }

        // Scan forward to find a sub-word boundary
        while i < len {
            if Self::is_boundary_at(&chars, i) {
                return i;
            }
            i += 1;
        }

        len
    }

    /// Detect the type of sub-word boundary at a given position.
    ///
    /// A boundary exists at `pos` if there's a transition between `chars[pos-1]`
    /// and `chars[pos]` matching one of the defined patterns.
    pub fn detect_boundary_type(chars: &[char], pos: usize) -> WordPartBoundary {
        if pos == 0 || pos >= chars.len() {
            return WordPartBoundary::WordEdge;
        }

        let prev = chars[pos - 1];
        let curr = chars[pos];

        // alpha↔non-alpha
        if prev.is_alphanumeric() != curr.is_alphanumeric() {
            return WordPartBoundary::AlphaToNonAlpha;
        }

        // digit↔alpha
        if prev.is_ascii_digit() && curr.is_alphabetic() {
            return WordPartBoundary::DigitAlphaTransition;
        }
        if prev.is_alphabetic() && curr.is_ascii_digit() {
            return WordPartBoundary::DigitAlphaTransition;
        }

        // lowerUpper (camelCase)
        if prev.is_lowercase() && curr.is_uppercase() {
            return WordPartBoundary::LowerToUpper;
        }

        // UPPER_UPPER_lower: boundary before the last uppercase in a run preceding lowercase
        // e.g., "XMLParser" → boundary at 'P' (between 'L' and 'P')
        if pos + 1 < chars.len()
            && prev.is_uppercase()
            && curr.is_uppercase()
            && chars[pos + 1].is_lowercase()
        {
            // The boundary is at the NEXT position (before 'P' in "XMLParser")
            // Actually, the boundary IS at pos+1 in this check — but we detect
            // it here because we look one ahead.
        }

        // Check if we're at the start of a lowercase run after an uppercase run
        if prev.is_uppercase() && curr.is_lowercase() && pos >= 2 && chars[pos - 2].is_uppercase() {
            return WordPartBoundary::UpperRunBeforeLower;
        }

        WordPartBoundary::WordEdge
    }

    /// Check if a sub-word boundary exists at position `pos`.
    fn is_boundary_at(chars: &[char], pos: usize) -> bool {
        if pos == 0 || pos >= chars.len() {
            return false;
        }

        let prev = chars[pos - 1];
        let curr = chars[pos];

        // alpha↔non-alpha boundary
        if prev.is_alphanumeric() != curr.is_alphanumeric() {
            return true;
        }

        // digit↔alpha boundary
        if (prev.is_ascii_digit() && curr.is_alphabetic())
            || (prev.is_alphabetic() && curr.is_ascii_digit())
        {
            return true;
        }

        // lowerUpper (camelCase boundary)
        if prev.is_lowercase() && curr.is_uppercase() {
            return true;
        }

        // UPPER run before lowercase: "XMLParser" boundary between 'L' and 'P'
        // Detected when prev is uppercase and curr is uppercase and the char AFTER curr is lowercase.
        // The boundary is at `pos` (before curr starts a new sub-word with the following lowercase).
        if pos + 1 < chars.len()
            && prev.is_uppercase()
            && curr.is_uppercase()
            && chars[pos + 1].is_lowercase()
        {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camel_case_boundary_get_value() {
        // Validates: Requirement 8.1, 8.5 (lowerUpper)
        let text = "getValue";
        // Boundary at position 3 (between 't' and 'V')
        let new_pos = WordPartNav::word_part_right(text, 0, SelectionModifier::Move);
        assert_eq!(new_pos, 3); // "get" boundary
    }

    #[test]
    fn camel_case_boundary_left() {
        // Validates: Requirement 8.1
        let text = "getValue";
        let new_pos = WordPartNav::word_part_left(text, 8, SelectionModifier::Move);
        assert_eq!(new_pos, 3); // Back to start of "Value"
    }

    #[test]
    fn upper_run_before_lower_xml_parser() {
        // Validates: Requirement 8.5 (UPPER_UPPER_lower)
        let text = "XMLParser";
        // From start, first boundary should be at position 3 ('P')
        let new_pos = WordPartNav::word_part_right(text, 0, SelectionModifier::Move);
        assert_eq!(new_pos, 3); // "XML" | "Parser"
    }

    #[test]
    fn digit_alpha_boundary() {
        // Validates: Requirement 8.5 (digit_alpha)
        let text = "test123abc";
        let chars: Vec<char> = text.chars().collect();
        let boundary = WordPartNav::detect_boundary_type(&chars, 7);
        assert_eq!(boundary, WordPartBoundary::DigitAlphaTransition);
    }

    #[test]
    fn alpha_nonalpha_boundary() {
        // Validates: Requirement 8.5 (alpha_nonalpha)
        let text = "get_value";
        let chars: Vec<char> = text.chars().collect();
        let boundary = WordPartNav::detect_boundary_type(&chars, 3);
        assert_eq!(boundary, WordPartBoundary::AlphaToNonAlpha);
    }

    #[test]
    fn snake_case_navigation() {
        // Validates: Requirement 8.1, 8.2
        let text = "get_value_fast";
        // From start, first boundary at '_' (position 3)
        let pos1 = WordPartNav::word_part_right(text, 0, SelectionModifier::Move);
        assert_eq!(pos1, 3); // "get" | "_value_fast"
    }

    #[test]
    fn word_part_left_from_end() {
        let text = "getValueFast";
        let pos = WordPartNav::word_part_left(text, 12, SelectionModifier::Move);
        assert_eq!(pos, 8); // Start of "Fast"
    }

    #[test]
    fn word_part_at_start_returns_zero() {
        let text = "hello";
        let pos = WordPartNav::word_part_left(text, 0, SelectionModifier::Move);
        assert_eq!(pos, 0);
    }

    #[test]
    fn word_part_at_end_returns_len() {
        let text = "hello";
        let pos = WordPartNav::word_part_right(text, 5, SelectionModifier::Move);
        assert_eq!(pos, 5);
    }
}
