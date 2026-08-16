//! MASK primary command handler (including MASK OFF).
//!
//! Handles execution of the MASK primary command — display/toggle mask line,
//! and MASK OFF to clear the active insert mask.

use crate::error::TabsMaskError;
use crate::state::{ArtifactPosition, TabsMaskState};

/// Result of executing a MASK primary command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaskCommandResult {
    /// MASK_Line(s) added to viewport.
    LinesAdded {
        /// Number of lines added.
        count: usize,
    },
    /// MASK_Line(s) removed from viewport (toggle off).
    LinesRemoved {
        /// Number of lines removed.
        count: usize,
    },
    /// Mask cleared (MASK OFF).
    MaskCleared,
    /// No active mask to display.
    NoActiveMask,
    /// No active mask to clear.
    NoMaskToClear,
}

/// Handles execution of the MASK primary command.
///
/// Addresses: Requirements 6, 7
///
/// # Arguments
///
/// * `state` - The current tabs/mask session state
/// * `args` - Command arguments ("OFF" to clear, empty for toggle)
/// * `cursor_line` - The current cursor line position (None defaults to 0)
/// * `line_width` - The document line width
pub fn execute_mask_command(
    state: &mut TabsMaskState,
    args: &[&str],
    cursor_line: Option<usize>,
    _line_width: usize,
) -> Result<MaskCommandResult, TabsMaskError> {
    // Check for MASK OFF
    if args.len() == 1 && args[0].eq_ignore_ascii_case("OFF") {
        return execute_mask_off(state);
    }

    // No arguments: toggle MASK_Lines
    if !state.mask().is_active() {
        return Ok(MaskCommandResult::NoActiveMask);
    }

    if state.has_mask_lines() {
        // Remove all MASK_Lines (toggle off)
        let count = state.mask_lines().len();
        state.remove_all_mask_lines();
        Ok(MaskCommandResult::LinesRemoved { count })
    } else {
        // Insert a MASK_Line at cursor position
        let anchor = cursor_line.unwrap_or(0);
        state.add_mask_line(ArtifactPosition {
            anchor_line: anchor,
            from_line_command: false,
        });
        Ok(MaskCommandResult::LinesAdded { count: 1 })
    }
}

/// Handles execution of MASK OFF — clears the active insert mask.
///
/// Addresses: Requirement 7, criteria 7.1–7.4
fn execute_mask_off(state: &mut TabsMaskState) -> Result<MaskCommandResult, TabsMaskError> {
    if !state.mask().is_active() {
        return Ok(MaskCommandResult::NoMaskToClear);
    }

    state.mask_mut().clear();
    state.remove_all_mask_lines();
    Ok(MaskCommandResult::MaskCleared)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mask::MaskLine;
    use crate::state::{MaskState, TabStopSource, TabsMaskState, TabsState};
    use crate::tab_stops::TabStopList;

    fn make_state_with_mask() -> TabsMaskState {
        TabsMaskState::new(
            TabsState::new(TabStopList::empty(), TabStopSource::BuiltIn),
            MaskState::with_mask(MaskLine::new("      *"), true),
        )
    }

    fn make_state_without_mask() -> TabsMaskState {
        TabsMaskState::new(
            TabsState::new(TabStopList::empty(), TabStopSource::BuiltIn),
            MaskState::empty(),
        )
    }

    #[test]
    fn toggle_with_active_mask_adds_mask_line() {
        // Validates: Requirement 6.1
        let mut state = make_state_with_mask();
        let result = execute_mask_command(&mut state, &[], Some(5), 80).unwrap();
        assert_eq!(result, MaskCommandResult::LinesAdded { count: 1 });
        assert!(state.has_mask_lines());
    }

    #[test]
    fn toggle_off_removes_mask_lines() {
        // Validates: Requirement 6.5
        let mut state = make_state_with_mask();
        state.add_mask_line(ArtifactPosition {
            anchor_line: 3,
            from_line_command: false,
        });
        let result = execute_mask_command(&mut state, &[], Some(5), 80).unwrap();
        assert_eq!(result, MaskCommandResult::LinesRemoved { count: 1 });
        assert!(!state.has_mask_lines());
    }

    #[test]
    fn no_active_mask_returns_no_active_mask() {
        // Validates: Requirement 6.2
        let mut state = make_state_without_mask();
        let result = execute_mask_command(&mut state, &[], Some(5), 80).unwrap();
        assert_eq!(result, MaskCommandResult::NoActiveMask);
    }

    #[test]
    fn mask_off_clears_mask_and_removes_lines() {
        // Validates: Requirement 7.1, 7.2
        let mut state = make_state_with_mask();
        state.add_mask_line(ArtifactPosition {
            anchor_line: 3,
            from_line_command: false,
        });
        let result = execute_mask_command(&mut state, &["OFF"], None, 80).unwrap();
        assert_eq!(result, MaskCommandResult::MaskCleared);
        assert!(!state.mask().is_active());
        assert!(!state.has_mask_lines());
    }

    #[test]
    fn mask_off_with_no_mask_returns_no_mask_to_clear() {
        // Validates: Requirement 7.3
        let mut state = make_state_without_mask();
        let result = execute_mask_command(&mut state, &["OFF"], None, 80).unwrap();
        assert_eq!(result, MaskCommandResult::NoMaskToClear);
    }

    #[test]
    fn mask_off_case_insensitive() {
        let mut state = make_state_with_mask();
        let result = execute_mask_command(&mut state, &["off"], None, 80).unwrap();
        assert_eq!(result, MaskCommandResult::MaskCleared);
    }
}
