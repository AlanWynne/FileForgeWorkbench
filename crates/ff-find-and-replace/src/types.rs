//! Core newtypes shared across the find-and-replace crate.
//!
//! Re-exports `BytePosition` and `LineNumber` from `ff-document-model`
//! and defines `MatchRange` for representing match byte spans.

pub use ff_document_model::{BytePosition, Direction, LineNumber};

/// A byte range within the document representing a match.
///
/// The range is half-open: [start, end).
///
/// Addresses: Requirement 1 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MatchRange {
    /// Start byte position (inclusive).
    pub start: BytePosition,
    /// End byte position (exclusive).
    pub end: BytePosition,
}

impl MatchRange {
    /// Create a new match range.
    pub fn new(start: BytePosition, end: BytePosition) -> Self {
        Self { start, end }
    }

    /// Length of the match in bytes.
    pub fn length(&self) -> u64 {
        self.end - self.start
    }

    /// Whether this range contains the given position.
    pub fn contains(&self, position: BytePosition) -> bool {
        position >= self.start && position < self.end
    }

    /// Whether this range is empty (zero length).
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_range_length_is_difference_of_start_and_end() {
        let range = MatchRange::new(BytePosition(5), BytePosition(15));
        assert_eq!(range.length(), 10);
    }

    #[test]
    fn match_range_contains_positions_within_half_open_interval() {
        let range = MatchRange::new(BytePosition(10), BytePosition(20));
        assert!(range.contains(BytePosition(10)));
        assert!(range.contains(BytePosition(15)));
        assert!(range.contains(BytePosition(19)));
        assert!(!range.contains(BytePosition(9)));
        assert!(!range.contains(BytePosition(20)));
    }

    #[test]
    fn match_range_is_empty_when_start_equals_end() {
        let range = MatchRange::new(BytePosition(5), BytePosition(5));
        assert!(range.is_empty());
        assert_eq!(range.length(), 0);
    }
}
