//! Search direction enumeration for FIND/CHANGE traversal.
//!
//! Addresses: Requirement 1 AC 2–5, Requirement 5 AC 4–5

use std::fmt;

/// Direction of traversal for FIND/CHANGE commands.
///
/// Addresses: Requirement 1 AC 2–5
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SearchDirection {
    /// Next match after cursor (default for FIND).
    #[default]
    Next,
    /// Previous match before cursor.
    Prev,
    /// First match from document start.
    First,
    /// Last match from document end.
    Last,
}

impl SearchDirection {
    /// Normalise direction for RFIND/RCHANGE: FIRST becomes NEXT, LAST becomes PREV.
    ///
    /// Addresses: Requirement 5 AC 4–5
    pub fn normalise_for_repeat(self) -> Self {
        match self {
            Self::First => Self::Next,
            Self::Last => Self::Prev,
            Self::Next | Self::Prev => self,
        }
    }

    /// Whether this direction searches forward (toward higher byte positions).
    pub fn is_forward(self) -> bool {
        matches!(self, Self::Next | Self::First)
    }

    /// Whether this direction searches backward (toward lower byte positions).
    pub fn is_backward(self) -> bool {
        matches!(self, Self::Prev | Self::Last)
    }

    /// Parse from a command token string (case-insensitive).
    pub fn from_token(token: &str) -> Option<Self> {
        match token.to_ascii_uppercase().as_str() {
            "NEXT" => Some(Self::Next),
            "PREV" => Some(Self::Prev),
            "FIRST" => Some(Self::First),
            "LAST" => Some(Self::Last),
            _ => None,
        }
    }
}

impl fmt::Display for SearchDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Next => write!(f, "NEXT"),
            Self::Prev => write!(f, "PREV"),
            Self::First => write!(f, "FIRST"),
            Self::Last => write!(f, "LAST"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_normalise_converts_first_to_next_and_last_to_prev() {
        assert_eq!(
            SearchDirection::First.normalise_for_repeat(),
            SearchDirection::Next
        );
        assert_eq!(
            SearchDirection::Last.normalise_for_repeat(),
            SearchDirection::Prev
        );
        assert_eq!(
            SearchDirection::Next.normalise_for_repeat(),
            SearchDirection::Next
        );
        assert_eq!(
            SearchDirection::Prev.normalise_for_repeat(),
            SearchDirection::Prev
        );
    }

    #[test]
    fn direction_is_forward_and_is_backward_are_mutually_exclusive() {
        assert!(SearchDirection::Next.is_forward());
        assert!(SearchDirection::First.is_forward());
        assert!(!SearchDirection::Next.is_backward());
        assert!(!SearchDirection::First.is_backward());

        assert!(SearchDirection::Prev.is_backward());
        assert!(SearchDirection::Last.is_backward());
        assert!(!SearchDirection::Prev.is_forward());
        assert!(!SearchDirection::Last.is_forward());
    }

    #[test]
    fn direction_from_token_parses_case_insensitively() {
        assert_eq!(
            SearchDirection::from_token("NEXT"),
            Some(SearchDirection::Next)
        );
        assert_eq!(
            SearchDirection::from_token("next"),
            Some(SearchDirection::Next)
        );
        assert_eq!(
            SearchDirection::from_token("Prev"),
            Some(SearchDirection::Prev)
        );
        assert_eq!(
            SearchDirection::from_token("FIRST"),
            Some(SearchDirection::First)
        );
        assert_eq!(
            SearchDirection::from_token("last"),
            Some(SearchDirection::Last)
        );
        assert_eq!(SearchDirection::from_token("unknown"), None);
    }

    #[test]
    fn direction_display_shows_uppercase_token() {
        assert_eq!(SearchDirection::Next.to_string(), "NEXT");
        assert_eq!(SearchDirection::Prev.to_string(), "PREV");
        assert_eq!(SearchDirection::First.to_string(), "FIRST");
        assert_eq!(SearchDirection::Last.to_string(), "LAST");
    }
}
