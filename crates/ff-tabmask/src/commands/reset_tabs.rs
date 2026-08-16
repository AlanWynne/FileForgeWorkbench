//! RESET TABS command handler.
//!
//! Restores default tab stops per precedence rules and updates displayed TABS_Lines.

use crate::error::TabsMaskError;
use crate::state::TabsMaskState;

/// Handles execution of the RESET TABS command.
///
/// Replaces the active tab stop list with the default tab stops (determined by
/// the precedence rules: Language_Definition > global config > built-in every-8-columns).
///
/// Addresses: Requirement 12, criteria 12.1–12.4
pub fn execute_reset_tabs(
    state: &mut TabsMaskState,
    _line_width: usize,
) -> Result<(), TabsMaskError> {
    state.tabs_mut().reset_to_defaults();
    // TABS_Lines remain displayed but now reflect the restored defaults (Req 12.3)
    Ok(())
}

/// Handles execution of RESET — clears display artifacts only.
///
/// Removes all TABS_Lines and MASK_Lines from the viewport but preserves
/// the tab stop list and mask content in session state.
///
/// Addresses: Requirement 11, criteria 11.1–11.4
pub fn handle_reset(state: &mut TabsMaskState) {
    state.remove_all_tabs_lines();
    state.remove_all_mask_lines();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mask::MaskLine;
    use crate::state::{ArtifactPosition, MaskState, TabStopSource, TabsState};
    use crate::tab_stops::TabStopList;

    #[test]
    fn reset_tabs_restores_defaults() {
        // Validates: Requirement 12.1
        let defaults = TabStopList::from_columns(vec![7, 12, 72]);
        let mut state = TabsMaskState::new(
            TabsState::new(defaults.clone(), TabStopSource::LanguageDefinition),
            MaskState::empty(),
        );
        state
            .tabs_mut()
            .set_tab_stops(TabStopList::from_columns(vec![5, 10]));

        execute_reset_tabs(&mut state, 80).unwrap();
        assert_eq!(state.tabs().tab_stops(), &defaults);
    }

    #[test]
    fn reset_tabs_does_not_remove_tabs_lines() {
        // Validates: Requirement 12.3
        let mut state = TabsMaskState::new(
            TabsState::new(
                TabStopList::from_columns(vec![8, 16]),
                TabStopSource::BuiltIn,
            ),
            MaskState::empty(),
        );
        state.add_tabs_line(ArtifactPosition {
            anchor_line: 5,
            from_line_command: false,
        });

        execute_reset_tabs(&mut state, 80).unwrap();
        assert!(state.has_tabs_lines());
    }

    #[test]
    fn handle_reset_removes_all_display_lines() {
        // Validates: Requirement 11.1, 11.2
        let mut state = TabsMaskState::new(
            TabsState::new(
                TabStopList::from_columns(vec![5, 10, 15]),
                TabStopSource::BuiltIn,
            ),
            MaskState::with_mask(MaskLine::new("test"), false),
        );
        state.add_tabs_line(ArtifactPosition {
            anchor_line: 3,
            from_line_command: false,
        });
        state.add_mask_line(ArtifactPosition {
            anchor_line: 7,
            from_line_command: true,
        });

        handle_reset(&mut state);
        assert!(!state.has_tabs_lines());
        assert!(!state.has_mask_lines());
    }

    #[test]
    fn handle_reset_preserves_tab_stops() {
        // Validates: Requirement 11.3
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let mut state = TabsMaskState::new(
            TabsState::new(stops.clone(), TabStopSource::BuiltIn),
            MaskState::empty(),
        );
        state.add_tabs_line(ArtifactPosition {
            anchor_line: 0,
            from_line_command: false,
        });

        handle_reset(&mut state);
        assert_eq!(state.tabs().tab_stops(), &stops);
    }

    #[test]
    fn handle_reset_preserves_mask_content() {
        // Validates: Requirement 11.4
        let mut state = TabsMaskState::new(
            TabsState::new(TabStopList::empty(), TabStopSource::BuiltIn),
            MaskState::with_mask(MaskLine::new("      *"), true),
        );
        state.add_mask_line(ArtifactPosition {
            anchor_line: 0,
            from_line_command: false,
        });

        handle_reset(&mut state);
        assert!(state.mask().is_active());
        assert_eq!(state.mask().mask().unwrap().content(), "      *");
    }
}
