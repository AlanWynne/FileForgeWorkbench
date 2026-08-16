//! Highlight-all-matches: viewport-scoped match computation.
//!
//! Addresses: Requirement 15

use crate::types::MatchRange;

#[cfg(test)]
use crate::types::BytePosition;

/// Result of a highlight-all computation.
///
/// Addresses: Requirement 15 AC 1, AC 6–7
#[derive(Debug, Clone)]
pub struct HighlightAllResult {
    /// Matches found within the viewport.
    pub matches: Vec<MatchRange>,
    /// Whether the total match count exceeds the configured maximum.
    pub truncated: bool,
    /// Total match count (may exceed matches.len() if truncated).
    pub total_count: u64,
}

impl HighlightAllResult {
    /// Create a new result with no matches.
    pub fn empty() -> Self {
        Self {
            matches: Vec::new(),
            truncated: false,
            total_count: 0,
        }
    }

    /// Create from a vec of match ranges with a max threshold.
    pub fn from_matches(matches: Vec<MatchRange>, max_matches: u64) -> Self {
        let total_count = matches.len() as u64;
        let truncated = total_count > max_matches;
        let limited = if truncated {
            matches.into_iter().take(max_matches as usize).collect()
        } else {
            matches
        };
        Self {
            matches: limited,
            truncated,
            total_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_result_has_no_matches() {
        let result = HighlightAllResult::empty();
        assert!(result.matches.is_empty());
        assert!(!result.truncated);
        assert_eq!(result.total_count, 0);
    }

    #[test]
    fn from_matches_truncates_when_exceeds_max() {
        let matches: Vec<MatchRange> = (0..10)
            .map(|i| MatchRange::new(BytePosition(i * 10), BytePosition(i * 10 + 5)))
            .collect();
        let result = HighlightAllResult::from_matches(matches, 5);
        assert!(result.truncated);
        assert_eq!(result.matches.len(), 5);
        assert_eq!(result.total_count, 10);
    }

    #[test]
    fn from_matches_no_truncation_within_limit() {
        let matches: Vec<MatchRange> = (0..3)
            .map(|i| MatchRange::new(BytePosition(i * 10), BytePosition(i * 10 + 5)))
            .collect();
        let result = HighlightAllResult::from_matches(matches, 1000);
        assert!(!result.truncated);
        assert_eq!(result.matches.len(), 3);
        assert_eq!(result.total_count, 3);
    }
}
