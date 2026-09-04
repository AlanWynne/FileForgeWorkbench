//! Show-excluded line command execution (F, L, S).
//!
//! These commands un-exclude individual lines from an excluded block.
//! They are session-state operations -- they do NOT produce EditorTransactions.

use ff_display_line_mapping::{DisplayLineMapping, DocLine};

use crate::error::LineCommandError;

/// Execute F -- show (un-exclude) only the first line of an excluded block.
///
/// Session-state only, no transaction produced.
pub fn execute_show_first(
    display_mapping: &mut dyn DisplayLineMapping,
    block_start: u64,
    _block_end: u64,
) -> Result<(), LineCommandError> {
    display_mapping.set_visible(
        DocLine(block_start as usize),
        DocLine(block_start as usize),
        true,
    );
    Ok(())
}

/// Execute L -- show (un-exclude) only the last line of an excluded block.
///
/// Session-state only, no transaction produced.
pub fn execute_show_last(
    display_mapping: &mut dyn DisplayLineMapping,
    _block_start: u64,
    block_end: u64,
) -> Result<(), LineCommandError> {
    display_mapping.set_visible(
        DocLine(block_end as usize),
        DocLine(block_end as usize),
        true,
    );
    Ok(())
}

/// Execute S -- show (un-exclude) the first line of an excluded block.
///
/// Equivalent to F for the purposes of this command.
/// Session-state only, no transaction produced.
pub fn execute_show_line(
    display_mapping: &mut dyn DisplayLineMapping,
    block_start: u64,
    _block_end: u64,
) -> Result<(), LineCommandError> {
    display_mapping.set_visible(
        DocLine(block_start as usize),
        DocLine(block_start as usize),
        true,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_display_line_mapping::ContractionState;

    fn make_excluded_state(total: usize, start: usize, end: usize) -> ContractionState {
        let mut state = ContractionState::new(total);
        state.set_visible(DocLine(start), DocLine(end), false);
        state
    }

    #[test]
    fn show_first_un_excludes_only_first_line() {
        // Validates: Requirement 15.5, 15.12
        let mut state = make_excluded_state(5, 1, 3);
        assert!(!state.get_visible(DocLine(1)));
        assert!(!state.get_visible(DocLine(2)));
        assert!(!state.get_visible(DocLine(3)));

        execute_show_first(&mut state, 1, 3).unwrap();

        assert!(state.get_visible(DocLine(1)));
        assert!(!state.get_visible(DocLine(2)));
        assert!(!state.get_visible(DocLine(3)));
    }

    #[test]
    fn show_last_un_excludes_only_last_line() {
        // Validates: Requirement 15.6, 15.12
        let mut state = make_excluded_state(5, 1, 3);

        execute_show_last(&mut state, 1, 3).unwrap();

        assert!(!state.get_visible(DocLine(1)));
        assert!(!state.get_visible(DocLine(2)));
        assert!(state.get_visible(DocLine(3)));
    }

    #[test]
    fn show_line_un_excludes_first_line_of_block() {
        // Validates: Requirement 15.9, 15.12
        let mut state = make_excluded_state(5, 2, 4);

        execute_show_line(&mut state, 2, 4).unwrap();

        assert!(state.get_visible(DocLine(2)));
        assert!(!state.get_visible(DocLine(3)));
        assert!(!state.get_visible(DocLine(4)));
    }

    #[test]
    fn show_first_on_single_line_block_un_excludes_it() {
        // Validates: Requirement 15.5 (single-line excluded block)
        let mut state = make_excluded_state(3, 1, 1);
        assert!(!state.get_visible(DocLine(1)));

        execute_show_first(&mut state, 1, 1).unwrap();
        assert!(state.get_visible(DocLine(1)));
    }

    #[test]
    fn show_last_on_single_line_block_un_excludes_it() {
        // Validates: Requirement 15.6 (single-line excluded block)
        let mut state = make_excluded_state(3, 1, 1);
        execute_show_last(&mut state, 1, 1).unwrap();
        assert!(state.get_visible(DocLine(1)));
    }

    #[test]
    fn show_excluded_produces_no_transaction() {
        // Validates: Requirement 15.12 -- all three return ()
        let mut state = make_excluded_state(5, 0, 4);
        assert!(execute_show_first(&mut state, 0, 4).is_ok());
        assert!(execute_show_last(&mut state, 0, 4).is_ok());
        assert!(execute_show_line(&mut state, 0, 4).is_ok());
    }
}
