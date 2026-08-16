//! TABS and MASK line command handlers.
//!
//! Handles execution of TABS and MASK line commands — entered in the prefix
//! area to insert display artifact lines at specific document positions.

use crate::artifacts::ArtifactKind;
use crate::error::TabsMaskError;
use crate::state::{ArtifactPosition, TabsMaskState};

/// Handles execution of TABS/MASK line commands.
///
/// Inserts a display artifact line immediately above the target document line.
///
/// Addresses: Requirements 3, 8
///
/// # Arguments
///
/// * `state` - The current tabs/mask session state
/// * `kind` - The kind of artifact to insert (TabsLine or MaskLine)
/// * `anchor_line` - The document line above which to insert the artifact
/// * `_line_width` - The document line width (used for rendering but not positioning)
pub fn execute_line_command(
    state: &mut TabsMaskState,
    kind: ArtifactKind,
    anchor_line: usize,
    _line_width: usize,
) -> Result<(), TabsMaskError> {
    let position = ArtifactPosition {
        anchor_line,
        from_line_command: true,
    };

    match kind {
        ArtifactKind::TabsLine => {
            state.add_tabs_line(position);
        }
        ArtifactKind::MaskLine => {
            state.add_mask_line(position);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mask::MaskLine;
    use crate::state::{MaskState, TabStopSource, TabsMaskState, TabsState};
    use crate::tab_stops::TabStopList;

    fn make_state() -> TabsMaskState {
        TabsMaskState::new(
            TabsState::new(
                TabStopList::from_columns(vec![5, 10, 15]),
                TabStopSource::BuiltIn,
            ),
            MaskState::with_mask(MaskLine::new("      *"), false),
        )
    }

    #[test]
    fn tabs_line_command_inserts_at_position() {
        // Validates: Requirement 3.1, 3.2
        let mut state = make_state();
        execute_line_command(&mut state, ArtifactKind::TabsLine, 7, 80).unwrap();
        assert!(state.has_tabs_lines());
        assert_eq!(state.tabs_lines()[0].anchor_line, 7);
        assert!(state.tabs_lines()[0].from_line_command);
    }

    #[test]
    fn mask_line_command_inserts_at_position() {
        // Validates: Requirement 8.1, 8.2
        let mut state = make_state();
        execute_line_command(&mut state, ArtifactKind::MaskLine, 12, 80).unwrap();
        assert!(state.has_mask_lines());
        assert_eq!(state.mask_lines()[0].anchor_line, 12);
        assert!(state.mask_lines()[0].from_line_command);
    }

    #[test]
    fn multiple_line_commands_add_multiple_artifacts() {
        let mut state = make_state();
        execute_line_command(&mut state, ArtifactKind::TabsLine, 3, 80).unwrap();
        execute_line_command(&mut state, ArtifactKind::TabsLine, 10, 80).unwrap();
        assert_eq!(state.tabs_lines().len(), 2);
    }
}
