//! Tab key cursor advancement logic.
//!
//! Computes the action to take when the Tab key is pressed, based on the
//! active tab stop list, current editing mode, and selection state.

use crate::artifacts::EditorMode;
use crate::tab_stops::TabStopList;

/// Describes the result of a Tab key press, to be executed by edit-operations.
///
/// Addresses: Requirement 5, criteria 5.1–5.6
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabKeyAction {
    /// Advance cursor by inserting spaces from current column to target column (Insert mode).
    ///
    /// Addresses: Requirement 5, criterion 5.5
    InsertSpacesTo {
        /// The target column to advance to.
        target_column: u32,
    },
    /// Move cursor to target column without modifying content (Overstrike mode).
    ///
    /// Addresses: Requirement 5, criterion 5.6
    MoveCursorTo {
        /// The target column to move to.
        target_column: u32,
    },
    /// Delegate to auto-indentation indent command (selection active).
    ///
    /// Addresses: Requirement 5, criterion 5.4
    DelegateToIndent,
    /// Fall back to standard navigation (Browse/View mode).
    ///
    /// Addresses: Requirement 5, criterion 5.4
    StandardNavigation,
    /// Advance by tab_size (no tab stops configured).
    ///
    /// Addresses: Requirement 5, criterion 5.3
    AdvanceBySize {
        /// The number of spaces to advance.
        spaces: u32,
    },
}

/// Computes the Tab key action for the given context.
///
/// Addresses: Requirement 5, criteria 5.1–5.6
///
/// # Arguments
///
/// * `tab_stops` - The active tab stop list
/// * `current_column` - The current cursor column (1-based)
/// * `mode` - The current editing mode
/// * `has_selection` - Whether a selection is active
/// * `tab_size` - The configured tab size (fallback when list is empty)
/// * `line_width` - The maximum line width
pub fn compute_tab_action(
    tab_stops: &TabStopList,
    current_column: u32,
    mode: EditorMode,
    has_selection: bool,
    tab_size: u32,
    line_width: u32,
) -> TabKeyAction {
    // Browse/View mode: standard navigation
    if mode == EditorMode::Browse || mode == EditorMode::View {
        return TabKeyAction::StandardNavigation;
    }

    // Selection active: delegate to indent
    if has_selection {
        return TabKeyAction::DelegateToIndent;
    }

    // Empty tab stop list: advance by tab_size
    if tab_stops.is_empty() {
        return TabKeyAction::AdvanceBySize { spaces: tab_size };
    }

    // Compute target column
    let target = match tab_stops.next_stop_after(current_column) {
        Some(col) => col.min(line_width),
        None => return TabKeyAction::AdvanceBySize { spaces: tab_size },
    };

    // Clamp to line width
    if target > line_width {
        return TabKeyAction::AdvanceBySize { spaces: tab_size };
    }

    match mode {
        EditorMode::Insert => TabKeyAction::InsertSpacesTo {
            target_column: target,
        },
        EditorMode::Overstrike => TabKeyAction::MoveCursorTo {
            target_column: target,
        },
        _ => TabKeyAction::StandardNavigation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_mode_advances_to_next_stop() {
        // Validates: Requirement 5.1, 5.5
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let action = compute_tab_action(&stops, 3, EditorMode::Insert, false, 8, 80);
        assert_eq!(action, TabKeyAction::InsertSpacesTo { target_column: 5 });
    }

    #[test]
    fn overstrike_mode_moves_cursor_without_inserting() {
        // Validates: Requirement 5.6
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let action = compute_tab_action(&stops, 3, EditorMode::Overstrike, false, 8, 80);
        assert_eq!(action, TabKeyAction::MoveCursorTo { target_column: 5 });
    }

    #[test]
    fn browse_mode_returns_standard_navigation() {
        // Validates: Requirement 5.4
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let action = compute_tab_action(&stops, 3, EditorMode::Browse, false, 8, 80);
        assert_eq!(action, TabKeyAction::StandardNavigation);
    }

    #[test]
    fn view_mode_returns_standard_navigation() {
        // Validates: Requirement 5.4
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let action = compute_tab_action(&stops, 3, EditorMode::View, false, 8, 80);
        assert_eq!(action, TabKeyAction::StandardNavigation);
    }

    #[test]
    fn selection_active_delegates_to_indent() {
        // Validates: Requirement 5.4
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let action = compute_tab_action(&stops, 3, EditorMode::Insert, true, 8, 80);
        assert_eq!(action, TabKeyAction::DelegateToIndent);
    }

    #[test]
    fn empty_list_advances_by_tab_size() {
        // Validates: Requirement 5.3
        let stops = TabStopList::empty();
        let action = compute_tab_action(&stops, 3, EditorMode::Insert, false, 4, 80);
        assert_eq!(action, TabKeyAction::AdvanceBySize { spaces: 4 });
    }

    #[test]
    fn past_last_stop_extends_with_interval() {
        // Validates: Requirement 5.2
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let action = compute_tab_action(&stops, 17, EditorMode::Insert, false, 8, 80);
        // Last interval is 5, so next after 17 is 20
        assert_eq!(action, TabKeyAction::InsertSpacesTo { target_column: 20 });
    }

    #[test]
    fn target_clamped_to_line_width() {
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let action = compute_tab_action(&stops, 12, EditorMode::Insert, false, 8, 14);
        // Next stop is 15 but line width is 14, so clamp
        assert_eq!(action, TabKeyAction::InsertSpacesTo { target_column: 14 });
    }
}
