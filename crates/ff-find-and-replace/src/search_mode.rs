//! Search mode enumeration controlling term interpretation.
//!
//! Addresses: Requirements 1, 3, 4

use std::fmt;

/// How the search term is interpreted.
///
/// Addresses: Requirements 1, 3, 4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SearchMode {
    /// Plain text matching (default).
    #[default]
    Literal,
    /// Regular expression pattern.
    Regex,
    /// Raw hex byte sequence (e.g., X'4A5B').
    HexBytes,
}

impl fmt::Display for SearchMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal => write!(f, "Literal"),
            Self::Regex => write!(f, "Regex"),
            Self::HexBytes => write!(f, "HexBytes"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_mode_display_shows_human_readable_name() {
        assert_eq!(SearchMode::Literal.to_string(), "Literal");
        assert_eq!(SearchMode::Regex.to_string(), "Regex");
        assert_eq!(SearchMode::HexBytes.to_string(), "HexBytes");
    }

    #[test]
    fn search_mode_default_is_literal() {
        assert_eq!(SearchMode::default(), SearchMode::Literal);
    }
}
