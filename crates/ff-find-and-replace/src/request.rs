//! Request types for FIND and CHANGE operations.
//!
//! Addresses: Requirements 1–9

use crate::direction::SearchDirection;
use crate::scope::{ColumnRange, ScopeModifier};
use crate::search_mode::SearchMode;
use crate::types::BytePosition;

/// Word-matching mode for boundary constraints.
///
/// Addresses: Requirement 11 AC 1–2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WordMatchMode {
    /// No word boundary constraints.
    #[default]
    None,
    /// Match must be a complete word (boundaries at both ends).
    WholeWord,
    /// Match must start at a word boundary.
    WordStart,
}

/// Complete specification for a single FIND operation.
///
/// Addresses: Requirements 1–5
#[derive(Debug, Clone)]
pub struct FindRequest {
    /// The search term (literal text, regex pattern, or hex string).
    pub term: String,
    /// How to interpret the search term.
    pub mode: SearchMode,
    /// Traversal direction.
    pub direction: SearchDirection,
    /// Scope filter (which lines to search).
    pub scope: ScopeModifier,
    /// Case sensitivity flag (true = case-sensitive, default).
    pub case_sensitive: bool,
    /// Whole-word matching mode.
    pub word_match: WordMatchMode,
    /// Optional explicit column range override.
    pub column_range: Option<ColumnRange>,
    /// Current cursor position (byte offset) for NEXT/PREV.
    pub cursor_position: BytePosition,
}

impl FindRequest {
    /// Create a minimal find request for a literal term at position 0.
    pub fn literal(term: &str) -> Self {
        Self {
            term: term.to_string(),
            mode: SearchMode::Literal,
            direction: SearchDirection::Next,
            scope: ScopeModifier::All,
            case_sensitive: true,
            word_match: WordMatchMode::None,
            column_range: None,
            cursor_position: BytePosition::ZERO,
        }
    }

    /// Create a find request for a regex pattern.
    pub fn regex(pattern: &str) -> Self {
        Self {
            term: pattern.to_string(),
            mode: SearchMode::Regex,
            direction: SearchDirection::Next,
            scope: ScopeModifier::All,
            case_sensitive: true,
            word_match: WordMatchMode::None,
            column_range: None,
            cursor_position: BytePosition::ZERO,
        }
    }

    /// Create a find request for a hex byte pattern.
    pub fn hex(hex_str: &str) -> Self {
        Self {
            term: hex_str.to_string(),
            mode: SearchMode::HexBytes,
            direction: SearchDirection::Next,
            scope: ScopeModifier::All,
            case_sensitive: true,
            word_match: WordMatchMode::None,
            column_range: None,
            cursor_position: BytePosition::ZERO,
        }
    }

    /// Builder: set direction.
    pub fn with_direction(mut self, direction: SearchDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Builder: set scope.
    pub fn with_scope(mut self, scope: ScopeModifier) -> Self {
        self.scope = scope;
        self
    }

    /// Builder: set case sensitivity.
    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Builder: set word match mode.
    pub fn with_word_match(mut self, mode: WordMatchMode) -> Self {
        self.word_match = mode;
        self
    }

    /// Builder: set cursor position.
    pub fn with_cursor(mut self, position: BytePosition) -> Self {
        self.cursor_position = position;
        self
    }

    /// Builder: set column range.
    pub fn with_column_range(mut self, range: ColumnRange) -> Self {
        self.column_range = Some(range);
        self
    }
}

/// Complete specification for a single CHANGE operation.
///
/// Addresses: Requirements 6–9
#[derive(Debug, Clone)]
pub struct ChangeRequest {
    /// The search portion (same semantics as FindRequest).
    pub find: FindRequest,
    /// The replacement text or template.
    pub replacement: String,
}

impl ChangeRequest {
    /// Create a change request from a find request and replacement text.
    pub fn new(find: FindRequest, replacement: &str) -> Self {
        Self {
            find,
            replacement: replacement.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_request_literal_creates_default_forward_search() {
        let req = FindRequest::literal("hello");
        assert_eq!(req.term, "hello");
        assert_eq!(req.mode, SearchMode::Literal);
        assert_eq!(req.direction, SearchDirection::Next);
        assert_eq!(req.scope, ScopeModifier::All);
        assert!(req.case_sensitive);
        assert_eq!(req.word_match, WordMatchMode::None);
        assert_eq!(req.column_range, None);
        assert_eq!(req.cursor_position, BytePosition::ZERO);
    }

    #[test]
    fn find_request_builders_modify_fields_correctly() {
        let req = FindRequest::literal("test")
            .with_direction(SearchDirection::Prev)
            .with_scope(ScopeModifier::Tagged)
            .with_case_sensitive(false)
            .with_word_match(WordMatchMode::WholeWord)
            .with_cursor(BytePosition(100));

        assert_eq!(req.direction, SearchDirection::Prev);
        assert_eq!(req.scope, ScopeModifier::Tagged);
        assert!(!req.case_sensitive);
        assert_eq!(req.word_match, WordMatchMode::WholeWord);
        assert_eq!(req.cursor_position, BytePosition(100));
    }

    #[test]
    fn change_request_wraps_find_request_with_replacement() {
        let find = FindRequest::literal("old");
        let change = ChangeRequest::new(find, "new");
        assert_eq!(change.find.term, "old");
        assert_eq!(change.replacement, "new");
    }
}
