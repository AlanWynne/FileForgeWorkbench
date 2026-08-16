//! Hex search integration.
//!
//! Bridge between the find-and-replace engine and hex display.
//! Handles FIND X'...' pattern validation, auto-activation of hex mode
//! on match, and match highlight coordination.

use crate::error::HexError;

/// A highlighted byte range from a hex search match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexMatchHighlight {
    /// Start byte offset (inclusive).
    pub start: u64,
    /// End byte offset (exclusive).
    pub end: u64,
}

/// Bridge between the find-and-replace engine and hex display.
///
/// Handles auto-activation of hex mode on hex search matches and
/// coordinates highlight rendering in the hex panes.
#[derive(Debug, Clone)]
pub struct HexSearchBridge {
    /// Currently highlighted match ranges (byte offsets).
    active_highlights: Vec<HexMatchHighlight>,
}

impl Default for HexSearchBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl HexSearchBridge {
    /// Create a new search bridge with no active highlights.
    pub fn new() -> Self {
        Self {
            active_highlights: Vec::new(),
        }
    }

    /// Called when a hex search match is found.
    ///
    /// Returns `true` if hex mode needs to be activated (was not active).
    pub fn on_hex_match_found(
        &mut self,
        match_start: u64,
        match_end: u64,
        hex_mode_active: bool,
    ) -> bool {
        self.active_highlights.push(HexMatchHighlight {
            start: match_start,
            end: match_end,
        });
        !hex_mode_active
    }

    /// Get current match highlights for rendering.
    pub fn active_highlights(&self) -> &[HexMatchHighlight] {
        &self.active_highlights
    }

    /// Clear all active highlights.
    pub fn clear_highlights(&mut self) {
        self.active_highlights.clear();
    }

    /// Check if a byte offset falls within any active highlight.
    pub fn is_highlighted(&self, offset: u64) -> bool {
        self.active_highlights
            .iter()
            .any(|h| offset >= h.start && offset < h.end)
    }

    /// Validate a hex search pattern string.
    ///
    /// The pattern must contain an even number of valid hex digit characters.
    /// Returns the decoded byte sequence, or an error for invalid patterns.
    pub fn validate_hex_pattern(pattern: &str) -> Result<Vec<u8>, HexError> {
        // Check for invalid characters first
        for ch in pattern.chars() {
            if !ch.is_ascii_hexdigit() {
                return Err(HexError::InvalidHexPatternChar(ch));
            }
        }

        // Check for odd length
        if !pattern.len().is_multiple_of(2) {
            return Err(HexError::OddHexPatternLength);
        }

        // Parse digit pairs into bytes
        let bytes: Vec<u8> = (0..pattern.len())
            .step_by(2)
            .map(|i| {
                let high = hex_char_to_nibble(pattern.as_bytes()[i]);
                let low = hex_char_to_nibble(pattern.as_bytes()[i + 1]);
                (high << 4) | low
            })
            .collect();

        Ok(bytes)
    }

    /// Search for a byte pattern in a document's raw bytes.
    ///
    /// Returns all match positions (start offsets) where the pattern
    /// is found as an exact byte-for-byte match.
    pub fn find_all_matches(data: &[u8], pattern: &[u8]) -> Vec<u64> {
        if pattern.is_empty() || data.len() < pattern.len() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        for i in 0..=(data.len() - pattern.len()) {
            if &data[i..i + pattern.len()] == pattern {
                matches.push(i as u64);
            }
        }
        matches
    }
}

/// Convert an ASCII hex digit byte to its nibble value.
fn hex_char_to_nibble(ch: u8) -> u8 {
    match ch {
        b'0'..=b'9' => ch - b'0',
        b'A'..=b'F' => ch - b'A' + 10,
        b'a'..=b'f' => ch - b'a' + 10,
        _ => 0, // unreachable after validation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 5 AC 1
    #[test]
    fn validate_hex_pattern_parses_valid_pairs() {
        let bytes = HexSearchBridge::validate_hex_pattern("0D0A").unwrap();
        assert_eq!(bytes, vec![0x0D, 0x0A]);

        let bytes = HexSearchBridge::validate_hex_pattern("DEADBEEF").unwrap();
        assert_eq!(bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    // Validates: Requirement 5 AC 1
    #[test]
    fn validate_hex_pattern_accepts_lowercase() {
        let bytes = HexSearchBridge::validate_hex_pattern("ff00ab").unwrap();
        assert_eq!(bytes, vec![0xFF, 0x00, 0xAB]);
    }

    // Validates: Requirement 5 AC 5
    #[test]
    fn validate_hex_pattern_rejects_odd_length() {
        let result = HexSearchBridge::validate_hex_pattern("0D0");
        assert_eq!(result.unwrap_err(), HexError::OddHexPatternLength);
    }

    // Validates: Requirement 5 AC 5
    #[test]
    fn validate_hex_pattern_rejects_invalid_characters() {
        let result = HexSearchBridge::validate_hex_pattern("0G0A");
        assert_eq!(result.unwrap_err(), HexError::InvalidHexPatternChar('G'));
    }

    // Validates: Requirement 5 AC 6
    #[test]
    fn find_all_matches_finds_byte_sequence() {
        let data = vec![0x01, 0x0D, 0x0A, 0x02, 0x0D, 0x0A, 0x03];
        let pattern = vec![0x0D, 0x0A];
        let matches = HexSearchBridge::find_all_matches(&data, &pattern);
        assert_eq!(matches, vec![1, 4]);
    }

    // Validates: Requirement 5 AC 7
    #[test]
    fn find_all_matches_is_case_sensitive_bytes() {
        // Byte search doesn't do Unicode case folding
        let data = vec![0x41, 0x42, 0x61, 0x62]; // "ABab"
        let pattern = vec![0x41, 0x42]; // "AB"
        let matches = HexSearchBridge::find_all_matches(&data, &pattern);
        assert_eq!(matches, vec![0]); // Only exact match at position 0
    }

    // Validates: Requirement 5 AC 6
    #[test]
    fn find_all_matches_empty_pattern_returns_empty() {
        let data = vec![0x01, 0x02, 0x03];
        let matches = HexSearchBridge::find_all_matches(&data, &[]);
        assert!(matches.is_empty());
    }

    // Validates: Requirement 5 AC 6
    #[test]
    fn find_all_matches_pattern_longer_than_data_returns_empty() {
        let data = vec![0x01, 0x02];
        let pattern = vec![0x01, 0x02, 0x03];
        let matches = HexSearchBridge::find_all_matches(&data, &pattern);
        assert!(matches.is_empty());
    }

    // Validates: Requirement 5 AC 2
    #[test]
    fn on_hex_match_found_returns_true_when_hex_inactive() {
        let mut bridge = HexSearchBridge::new();
        let needs_activate = bridge.on_hex_match_found(10, 12, false);
        assert!(needs_activate);
    }

    // Validates: Requirement 5 AC 3
    #[test]
    fn on_hex_match_found_records_highlight() {
        let mut bridge = HexSearchBridge::new();
        bridge.on_hex_match_found(10, 12, true);
        assert_eq!(bridge.active_highlights().len(), 1);
        assert_eq!(bridge.active_highlights()[0].start, 10);
        assert_eq!(bridge.active_highlights()[0].end, 12);
    }

    // Validates: Requirement 5 AC 3
    #[test]
    fn is_highlighted_checks_range_correctly() {
        let mut bridge = HexSearchBridge::new();
        bridge.on_hex_match_found(10, 14, true);

        assert!(!bridge.is_highlighted(9));
        assert!(bridge.is_highlighted(10));
        assert!(bridge.is_highlighted(13));
        assert!(!bridge.is_highlighted(14)); // exclusive end
    }

    #[test]
    fn clear_highlights_removes_all() {
        let mut bridge = HexSearchBridge::new();
        bridge.on_hex_match_found(0, 5, true);
        bridge.on_hex_match_found(10, 15, true);
        assert_eq!(bridge.active_highlights().len(), 2);

        bridge.clear_highlights();
        assert!(bridge.active_highlights().is_empty());
    }
}
