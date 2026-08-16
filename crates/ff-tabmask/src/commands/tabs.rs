//! TABS primary command handler.
//!
//! Handles execution of the TABS primary command — display/toggle tab stops
//! and set custom tab stop positions.

use crate::error::TabsMaskError;
use crate::state::{ArtifactPosition, TabsMaskState};
use crate::tab_stops::TabStopList;

/// Result of executing a TABS primary command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabsCommandResult {
    /// TABS_Line(s) added to viewport.
    LinesAdded {
        /// Number of lines added.
        count: usize,
    },
    /// TABS_Line(s) removed from viewport (toggle off).
    LinesRemoved {
        /// Number of lines removed.
        count: usize,
    },
    /// Tab stops updated and TABS_Line(s) refreshed.
    StopsUpdated {
        /// The new tab stop list.
        stops: TabStopList,
        /// Number of lines refreshed.
        lines_refreshed: usize,
    },
}

/// Handles execution of the TABS primary command.
///
/// Addresses: Requirements 1, 2
///
/// # Arguments
///
/// * `state` - The current tabs/mask session state
/// * `args` - Command arguments (empty for toggle, column numbers to set stops)
/// * `cursor_line` - The current cursor line position (None defaults to 0)
/// * `line_width` - The document line width
pub fn execute_tabs_command(
    state: &mut TabsMaskState,
    args: &[&str],
    cursor_line: Option<usize>,
    _line_width: usize,
) -> Result<TabsCommandResult, TabsMaskError> {
    if args.is_empty() {
        // No arguments: toggle TABS_Lines
        if state.has_tabs_lines() {
            // Remove all TABS_Lines (toggle off)
            let count = state.tabs_lines().len();
            state.remove_all_tabs_lines();
            Ok(TabsCommandResult::LinesRemoved { count })
        } else {
            // Insert a TABS_Line at cursor position
            let anchor = cursor_line.unwrap_or(0);
            state.add_tabs_line(ArtifactPosition {
                anchor_line: anchor,
                from_line_command: false,
            });
            Ok(TabsCommandResult::LinesAdded { count: 1 })
        }
    } else {
        // Column arguments: parse and set tab stops
        let stops = parse_tab_stops(args)?;
        let lines_refreshed = state.tabs_lines().len();
        state.tabs_mut().set_tab_stops(stops.clone());

        // If no TABS_Lines displayed, add one
        if !state.has_tabs_lines() {
            let anchor = cursor_line.unwrap_or(0);
            state.add_tabs_line(ArtifactPosition {
                anchor_line: anchor,
                from_line_command: false,
            });
        }

        Ok(TabsCommandResult::StopsUpdated {
            stops,
            lines_refreshed,
        })
    }
}

/// Parses and validates column arguments from a TABS command.
///
/// Returns `Ok` with a `TabStopList` on success, or an error if any argument is invalid.
///
/// Addresses: Requirement 2, criteria 2.1, 2.7, 2.8
pub fn parse_tab_stops(args: &[&str]) -> Result<TabStopList, TabsMaskError> {
    let mut invalid_values = Vec::new();
    let mut columns = Vec::new();

    for &arg in args {
        match arg.parse::<u32>() {
            Ok(0) => {
                invalid_values.push(arg.to_string());
            }
            Ok(col) => {
                columns.push(col);
            }
            Err(_) => {
                invalid_values.push(arg.to_string());
            }
        }
    }

    if !invalid_values.is_empty() {
        return Err(TabsMaskError::InvalidTabStops { invalid_values });
    }

    Ok(TabStopList::from_columns(columns))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{MaskState, TabStopSource, TabsState};

    fn make_state() -> TabsMaskState {
        TabsMaskState::new(
            TabsState::new(
                TabStopList::from_columns(vec![5, 10, 15]),
                TabStopSource::BuiltIn,
            ),
            MaskState::empty(),
        )
    }

    #[test]
    fn no_args_toggle_on_adds_tabs_line() {
        // Validates: Requirement 1.1
        let mut state = make_state();
        let result = execute_tabs_command(&mut state, &[], Some(3), 80).unwrap();
        assert_eq!(result, TabsCommandResult::LinesAdded { count: 1 });
        assert!(state.has_tabs_lines());
        assert_eq!(state.tabs_lines()[0].anchor_line, 3);
    }

    #[test]
    fn no_args_toggle_off_removes_tabs_lines() {
        // Validates: Requirement 1.4
        let mut state = make_state();
        state.add_tabs_line(ArtifactPosition {
            anchor_line: 5,
            from_line_command: false,
        });
        let result = execute_tabs_command(&mut state, &[], Some(3), 80).unwrap();
        assert_eq!(result, TabsCommandResult::LinesRemoved { count: 1 });
        assert!(!state.has_tabs_lines());
    }

    #[test]
    fn column_args_set_stops_and_add_line() {
        // Validates: Requirement 2.1, 2.3
        let mut state = make_state();
        let result = execute_tabs_command(&mut state, &["7", "12", "72"], Some(0), 80).unwrap();
        match result {
            TabsCommandResult::StopsUpdated { stops, .. } => {
                assert_eq!(stops, TabStopList::from_columns(vec![7, 12, 72]));
            }
            _ => panic!("Expected StopsUpdated"),
        }
        assert!(state.has_tabs_lines());
        assert_eq!(
            state.tabs().tab_stops(),
            &TabStopList::from_columns(vec![7, 12, 72])
        );
    }

    #[test]
    fn invalid_column_args_rejected() {
        // Validates: Requirement 2.7
        let mut state = make_state();
        let result = execute_tabs_command(&mut state, &["5", "abc", "0"], Some(0), 80);
        assert!(result.is_err());
        match result.unwrap_err() {
            TabsMaskError::InvalidTabStops { invalid_values } => {
                assert!(invalid_values.contains(&"abc".to_string()));
                assert!(invalid_values.contains(&"0".to_string()));
            }
            _ => panic!("Expected InvalidTabStops error"),
        }
    }

    #[test]
    fn duplicate_columns_deduplicated() {
        // Validates: Requirement 2.8
        let mut state = make_state();
        let result =
            execute_tabs_command(&mut state, &["5", "10", "5", "10"], Some(0), 80).unwrap();
        match result {
            TabsCommandResult::StopsUpdated { stops, .. } => {
                assert_eq!(stops, TabStopList::from_columns(vec![5, 10]));
            }
            _ => panic!("Expected StopsUpdated"),
        }
    }

    #[test]
    fn multiple_tabs_lines_at_different_positions() {
        // Validates: Requirement 1.7
        let mut state = make_state();
        execute_tabs_command(&mut state, &[], Some(3), 80).unwrap();
        // Add another at a different position by calling add_tabs_line directly
        state.add_tabs_line(ArtifactPosition {
            anchor_line: 10,
            from_line_command: false,
        });
        assert_eq!(state.tabs_lines().len(), 2);
    }

    #[test]
    fn parse_tab_stops_valid_input() {
        let result = parse_tab_stops(&["5", "10", "15"]).unwrap();
        assert_eq!(result, TabStopList::from_columns(vec![5, 10, 15]));
    }

    #[test]
    fn parse_tab_stops_rejects_zero() {
        let result = parse_tab_stops(&["0", "5"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_tab_stops_rejects_non_integer() {
        let result = parse_tab_stops(&["abc"]);
        assert!(result.is_err());
    }
}
