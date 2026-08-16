//! Exclude line command execution (X, Xn, XX).
//!
//! Session-state only — does NOT produce an EditorTransaction.

use ff_display_line_mapping::{DisplayLineMapping, DocLine};

use crate::error::LineCommandError;

/// Execute an exclude operation — set excluded flag on `count` lines starting at `start_line`.
///
/// This is a session-state operation that does NOT produce an EditorTransaction.
pub fn execute_exclude(
    display_mapping: &mut dyn DisplayLineMapping,
    start_line: u64,
    count: u64,
) -> Result<(), LineCommandError> {
    let end_line = start_line + count - 1;
    display_mapping.set_visible(
        DocLine(start_line as usize),
        DocLine(end_line as usize),
        false,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_display_line_mapping::ContractionState;

    #[test]
    fn exclude_single_line_hides_it() {
        let mut state = ContractionState::new(10);
        assert!(state.get_visible(DocLine(3)));

        execute_exclude(&mut state, 3, 1).unwrap();
        assert!(!state.get_visible(DocLine(3)));
    }

    #[test]
    fn exclude_counted_hides_range() {
        let mut state = ContractionState::new(10);
        execute_exclude(&mut state, 2, 3).unwrap();
        assert!(!state.get_visible(DocLine(2)));
        assert!(!state.get_visible(DocLine(3)));
        assert!(!state.get_visible(DocLine(4)));
        assert!(state.get_visible(DocLine(5)));
    }

    #[test]
    fn exclude_does_not_produce_transaction() {
        // This test verifies by absence — the function returns ()
        let mut state = ContractionState::new(5);
        let result = execute_exclude(&mut state, 0, 2);
        assert!(result.is_ok());
        // No EditorTransaction returned — correct by API design
    }
}
