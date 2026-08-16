//! Toggle command implementations for whitespace, indent guides, and edge column.

use crate::modes::{EdgeMode, IndentGuideMode, WhitespaceVisibility};

/// Cycle whitespace visibility to the next mode.
///
/// Order: Invisible → VisibleAlways → VisibleAfterIndent → VisibleOnlyInIndent → Invisible
///
/// Addresses: Requirement 8 AC 8.1
pub fn toggle_whitespace(current: WhitespaceVisibility) -> WhitespaceVisibility {
    current.next()
}

/// Cycle indent guide mode to the next mode.
///
/// Order: None → Real → LookForward → LookBoth → None
///
/// Addresses: Requirement 8 AC 8.2
pub fn toggle_indent_guides(current: IndentGuideMode) -> IndentGuideMode {
    current.next()
}

/// Toggle edge column between None and the previous non-None mode.
///
/// If the current mode is `None`, switch to `previous_mode` (defaulting to `Line`).
/// If the current mode is non-None, switch to `None`.
///
/// Returns the new mode and the "previous" mode to remember.
///
/// Addresses: Requirement 8 AC 8.3
pub fn toggle_edge_column(current: EdgeMode, previous_non_none: Option<EdgeMode>) -> EdgeMode {
    match current {
        EdgeMode::None => previous_non_none.unwrap_or(EdgeMode::Line),
        _ => EdgeMode::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_cycles_through_all_four_modes() {
        // Validates: Requirement 8.1
        let start = WhitespaceVisibility::Invisible;
        let a = toggle_whitespace(start);
        assert_eq!(a, WhitespaceVisibility::VisibleAlways);
        let b = toggle_whitespace(a);
        assert_eq!(b, WhitespaceVisibility::VisibleAfterIndent);
        let c = toggle_whitespace(b);
        assert_eq!(c, WhitespaceVisibility::VisibleOnlyInIndent);
        let d = toggle_whitespace(c);
        assert_eq!(d, WhitespaceVisibility::Invisible);
    }

    #[test]
    fn whitespace_toggle_four_times_returns_to_start() {
        // Validates: Requirement 8.1
        for &start in WhitespaceVisibility::variants() {
            let mut current = start;
            for _ in 0..4 {
                current = toggle_whitespace(current);
            }
            assert_eq!(current, start);
        }
    }

    #[test]
    fn indent_guides_cycles_through_all_four_modes() {
        // Validates: Requirement 8.2
        let start = IndentGuideMode::None;
        let a = toggle_indent_guides(start);
        assert_eq!(a, IndentGuideMode::Real);
        let b = toggle_indent_guides(a);
        assert_eq!(b, IndentGuideMode::LookForward);
        let c = toggle_indent_guides(b);
        assert_eq!(c, IndentGuideMode::LookBoth);
        let d = toggle_indent_guides(c);
        assert_eq!(d, IndentGuideMode::None);
    }

    #[test]
    fn indent_guides_toggle_four_times_returns_to_start() {
        // Validates: Requirement 8.2
        for &start in IndentGuideMode::variants() {
            let mut current = start;
            for _ in 0..4 {
                current = toggle_indent_guides(current);
            }
            assert_eq!(current, start);
        }
    }

    #[test]
    fn edge_toggle_from_none_defaults_to_line() {
        // Validates: Requirement 8.3
        let result = toggle_edge_column(EdgeMode::None, None);
        assert_eq!(result, EdgeMode::Line);
    }

    #[test]
    fn edge_toggle_from_none_remembers_previous() {
        // Validates: Requirement 8.3
        let result = toggle_edge_column(EdgeMode::None, Some(EdgeMode::Background));
        assert_eq!(result, EdgeMode::Background);
    }

    #[test]
    fn edge_toggle_from_non_none_goes_to_none() {
        // Validates: Requirement 8.3
        let result = toggle_edge_column(EdgeMode::Line, None);
        assert_eq!(result, EdgeMode::None);

        let result = toggle_edge_column(EdgeMode::MultiLine, Some(EdgeMode::Background));
        assert_eq!(result, EdgeMode::None);
    }
}
