//! Result types for FIND and CHANGE operations.
//!
//! Addresses: Requirements 1, 6, 20

use crate::types::{BytePosition, LineNumber, MatchRange};

/// The result of a successful FIND operation.
///
/// Addresses: Requirement 1 AC 1, Requirement 4 AC 9
#[derive(Debug, Clone)]
pub struct FindResult {
    /// The byte range of the match in the document.
    pub match_range: MatchRange,
    /// The document line containing the match start.
    pub line: LineNumber,
    /// Captured groups (index 0 = entire match, 1–9 = sub-groups).
    /// Empty for literal and hex searches.
    pub captures: Vec<MatchRange>,
}

impl FindResult {
    /// Create a simple result with no captures (literal/hex match).
    pub fn simple(start: BytePosition, end: BytePosition, line: LineNumber) -> Self {
        Self {
            match_range: MatchRange::new(start, end),
            line,
            captures: Vec::new(),
        }
    }

    /// Create a result with captures (regex match).
    pub fn with_captures(
        start: BytePosition,
        end: BytePosition,
        line: LineNumber,
        captures: Vec<MatchRange>,
    ) -> Self {
        Self {
            match_range: MatchRange::new(start, end),
            line,
            captures,
        }
    }
}

/// The result of a successful CHANGE operation.
///
/// Addresses: Requirement 6 AC 1–2
#[derive(Debug, Clone)]
pub struct ChangeResult {
    /// Number of replacements made.
    pub replacement_count: u64,
    /// The position after the last replacement (for cursor placement).
    pub final_position: BytePosition,
    /// The line of the last replacement.
    pub final_line: LineNumber,
}

/// The outcome of a find operation.
///
/// Addresses: Requirement 1 AC 7, Requirement 20
#[derive(Debug, Clone)]
pub enum FindOutcome {
    /// A single match was found.
    Found(FindResult),
    /// Multiple matches found (for ALL direction).
    FoundAll { count: u64, first: FindResult },
    /// No match found.
    NotFound {
        /// The search term that was not found (for error message).
        term: String,
    },
}

impl FindOutcome {
    /// Whether the outcome represents a successful match.
    pub fn is_found(&self) -> bool {
        matches!(self, Self::Found(_) | Self::FoundAll { .. })
    }

    /// Get the first result if found.
    pub fn first_result(&self) -> Option<&FindResult> {
        match self {
            Self::Found(r) => Some(r),
            Self::FoundAll { first, .. } => Some(first),
            Self::NotFound { .. } => None,
        }
    }
}

/// The outcome of a change operation.
#[derive(Debug, Clone)]
pub enum ChangeOutcome {
    /// Replacements were made.
    Changed(ChangeResult),
    /// No match found to replace.
    NotFound { term: String },
    /// Document is read-only.
    ReadOnly,
}

impl ChangeOutcome {
    /// Whether the outcome represents a successful change.
    pub fn is_changed(&self) -> bool {
        matches!(self, Self::Changed(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_result_simple_creates_result_with_no_captures() {
        let result = FindResult::simple(BytePosition(5), BytePosition(10), LineNumber(0));
        assert_eq!(result.match_range.start, BytePosition(5));
        assert_eq!(result.match_range.end, BytePosition(10));
        assert_eq!(result.line, LineNumber(0));
        assert!(result.captures.is_empty());
    }

    #[test]
    fn find_outcome_is_found_returns_true_for_found_variants() {
        let found = FindOutcome::Found(FindResult::simple(
            BytePosition(0),
            BytePosition(5),
            LineNumber(0),
        ));
        assert!(found.is_found());

        let not_found = FindOutcome::NotFound {
            term: "x".to_string(),
        };
        assert!(!not_found.is_found());
    }

    #[test]
    fn find_outcome_first_result_extracts_result_when_present() {
        let result = FindResult::simple(BytePosition(0), BytePosition(5), LineNumber(0));
        let outcome = FindOutcome::Found(result);
        assert!(outcome.first_result().is_some());

        let not_found = FindOutcome::NotFound {
            term: "x".to_string(),
        };
        assert!(not_found.first_result().is_none());
    }
}
