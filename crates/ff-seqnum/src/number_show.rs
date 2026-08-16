//! NUMBER SHOW display mode.
//!
//! Provides the overlay data model for rendering original sequence numbers
//! in the viewport without modifying the edit buffer.

use crate::state::SeqNumState;

/// An overlay entry for a single line in NUMBER SHOW mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayEntry {
    /// Content to display in the front column range (if any).
    pub front_text: Option<String>,
    /// Content to display in the back column range (if any).
    pub back_text: Option<String>,
}

/// Get the overlay content for a specific line in NUMBER SHOW mode.
///
/// Returns `None` if:
/// - NUMBER SHOW mode is not active
/// - No stripping occurred (nothing to overlay)
/// - No side-table entry exists for this line
pub fn get_overlay_content(state: &SeqNumState, line_index: usize) -> Option<OverlayEntry> {
    // Must be in show mode
    if !state.number_show_active {
        return None;
    }

    // Must have stripped something
    if state.stripped_front.is_none() && state.stripped_back.is_none() {
        return None;
    }

    // Look up side-table entry
    let entry = state.side_table.get_original_values(line_index)?;

    Some(OverlayEntry {
        front_text: entry.front_content.clone(),
        back_text: entry.back_content.clone(),
    })
}

/// Toggle NUMBER SHOW mode on or off.
///
/// This is a non-undoable display state change (Requirement 8.6).
/// Returns the new state of the mode.
pub fn toggle_show_mode(state: &mut SeqNumState) -> bool {
    state.number_show_active = !state.number_show_active;
    state.number_show_active
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ColumnRange;

    #[test]
    fn toggle_show_mode_on_and_off() {
        // Validates: Requirement 8.1
        let mut state = SeqNumState::new();
        assert!(!state.number_show_active);

        let result = toggle_show_mode(&mut state);
        assert!(result);
        assert!(state.number_show_active);

        let result = toggle_show_mode(&mut state);
        assert!(!result);
        assert!(!state.number_show_active);
    }

    #[test]
    fn overlay_returns_none_when_mode_inactive() {
        // Validates: Requirement 8.7
        let mut state = SeqNumState::new();
        state.stripped_front = Some(ColumnRange::new(1, 6).unwrap());
        state
            .side_table
            .store_stripped_values(0, Some("000100"), None);

        let result = get_overlay_content(&state, 0);
        assert!(result.is_none());
    }

    #[test]
    fn overlay_returns_values_when_mode_active() {
        // Validates: Requirement 8.2
        let mut state = SeqNumState::new();
        state.number_show_active = true;
        state.stripped_front = Some(ColumnRange::new(1, 6).unwrap());
        state.stripped_back = Some(ColumnRange::new(73, 80).unwrap());
        state
            .side_table
            .store_stripped_values(0, Some("000100"), Some("00000100"));

        let result = get_overlay_content(&state, 0).unwrap();
        assert_eq!(result.front_text.as_deref(), Some("000100"));
        assert_eq!(result.back_text.as_deref(), Some("00000100"));
    }

    #[test]
    fn overlay_returns_none_when_no_stripping_occurred() {
        // Validates: Requirement 8.7
        let mut state = SeqNumState::new();
        state.number_show_active = true;
        // No stripped ranges set

        let result = get_overlay_content(&state, 0);
        assert!(result.is_none());
    }

    #[test]
    fn overlay_returns_none_for_missing_line() {
        // Validates: Requirement 8.2
        let mut state = SeqNumState::new();
        state.number_show_active = true;
        state.stripped_front = Some(ColumnRange::new(1, 6).unwrap());
        state
            .side_table
            .store_stripped_values(0, Some("000100"), None);

        let result = get_overlay_content(&state, 5); // Line 5 not in table
        assert!(result.is_none());
    }

    #[test]
    fn toggle_does_not_affect_side_table() {
        // Validates: Requirement 8.6 (non-undoable display change)
        let mut state = SeqNumState::new();
        state.stripped_front = Some(ColumnRange::new(1, 6).unwrap());
        state
            .side_table
            .store_stripped_values(0, Some("000100"), None);

        toggle_show_mode(&mut state);
        toggle_show_mode(&mut state);

        // Side-table unchanged
        let entry = state.side_table.get_original_values(0).unwrap();
        assert_eq!(entry.front_content.as_deref(), Some("000100"));
    }
}
