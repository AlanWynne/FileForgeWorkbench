//! Shift-to-tab-stop computation for `>` and `<` line commands.
//!
//! Computes the target column and delta (spaces to add/remove) when shifting
//! line content rightward or leftward to the nearest tab stop position.

use crate::tab_stops::TabStopList;

/// Describes the result of computing a shift target for >/< line commands.
///
/// Addresses: Requirement 14, criteria 14.1–14.4
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShiftAction {
    /// The target column for the first non-space character after shifting.
    pub target_column: u32,
    /// Number of spaces to add (positive) or remove (negative) from line start.
    pub delta: i32,
}

/// Computes the shift action for a `>` (shift right) command.
///
/// Shifts content rightward to the next tab stop position(s).
///
/// Addresses: Requirement 14, criteria 14.1, 14.4
///
/// # Arguments
///
/// * `tab_stops` - The active tab stop list
/// * `first_nonspace_column` - The current column of the first non-space character (1-based)
/// * `count` - Number of tab stop positions to shift (for `>n` commands)
pub fn compute_shift_right(
    tab_stops: &TabStopList,
    first_nonspace_column: u32,
    count: u32,
) -> ShiftAction {
    if tab_stops.is_empty() || count == 0 {
        return ShiftAction {
            target_column: first_nonspace_column,
            delta: 0,
        };
    }

    let target = tab_stops
        .nth_stop_after(first_nonspace_column, count)
        .unwrap_or(first_nonspace_column);

    let delta = target as i32 - first_nonspace_column as i32;

    ShiftAction {
        target_column: target,
        delta,
    }
}

/// Computes the shift action for a `<` (shift left) command.
///
/// Shifts content leftward to the previous tab stop position(s). Floors at column 1.
///
/// Addresses: Requirement 14, criteria 14.2, 14.3, 14.4
///
/// # Arguments
///
/// * `tab_stops` - The active tab stop list
/// * `first_nonspace_column` - The current column of the first non-space character (1-based)
/// * `count` - Number of tab stop positions to shift (for `<n` commands)
pub fn compute_shift_left(
    tab_stops: &TabStopList,
    first_nonspace_column: u32,
    count: u32,
) -> ShiftAction {
    if tab_stops.is_empty() || count == 0 || first_nonspace_column <= 1 {
        return ShiftAction {
            target_column: first_nonspace_column,
            delta: 0,
        };
    }

    let target = tab_stops
        .nth_stop_before(first_nonspace_column, count)
        .unwrap_or(1)
        .max(1);

    let delta = target as i32 - first_nonspace_column as i32;

    ShiftAction {
        target_column: target,
        delta,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_right_by_one_advances_to_next_stop() {
        // Validates: Requirement 14.1
        let stops = TabStopList::from_columns(vec![5, 10, 15, 20]);
        let action = compute_shift_right(&stops, 5, 1);
        assert_eq!(
            action,
            ShiftAction {
                target_column: 10,
                delta: 5
            }
        );
    }

    #[test]
    fn shift_right_by_n_advances_n_stops() {
        // Validates: Requirement 14.4
        let stops = TabStopList::from_columns(vec![5, 10, 15, 20]);
        let action = compute_shift_right(&stops, 5, 2);
        assert_eq!(
            action,
            ShiftAction {
                target_column: 15,
                delta: 10
            }
        );
    }

    #[test]
    fn shift_left_by_one_retreats_to_previous_stop() {
        // Validates: Requirement 14.2
        let stops = TabStopList::from_columns(vec![5, 10, 15, 20]);
        let action = compute_shift_left(&stops, 15, 1);
        assert_eq!(
            action,
            ShiftAction {
                target_column: 10,
                delta: -5
            }
        );
    }

    #[test]
    fn shift_left_by_n_retreats_n_stops() {
        // Validates: Requirement 14.4
        let stops = TabStopList::from_columns(vec![5, 10, 15, 20]);
        let action = compute_shift_left(&stops, 20, 2);
        assert_eq!(
            action,
            ShiftAction {
                target_column: 10,
                delta: -10
            }
        );
    }

    #[test]
    fn shift_left_past_first_stop_floors_at_column_1() {
        // Validates: Requirement 14.3
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let action = compute_shift_left(&stops, 5, 1);
        // No stop before 5, so floor at 1
        assert_eq!(
            action,
            ShiftAction {
                target_column: 1,
                delta: -4
            }
        );
    }

    #[test]
    fn shift_left_past_all_stops_floors_at_column_1() {
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let action = compute_shift_left(&stops, 10, 5);
        assert_eq!(
            action,
            ShiftAction {
                target_column: 1,
                delta: -9
            }
        );
    }

    #[test]
    fn shift_with_empty_list_returns_no_change() {
        let stops = TabStopList::empty();
        let right = compute_shift_right(&stops, 5, 1);
        assert_eq!(
            right,
            ShiftAction {
                target_column: 5,
                delta: 0
            }
        );

        let left = compute_shift_left(&stops, 5, 1);
        assert_eq!(
            left,
            ShiftAction {
                target_column: 5,
                delta: 0
            }
        );
    }

    #[test]
    fn shift_with_count_zero_returns_no_change() {
        let stops = TabStopList::from_columns(vec![5, 10, 15]);
        let action = compute_shift_right(&stops, 5, 0);
        assert_eq!(
            action,
            ShiftAction {
                target_column: 5,
                delta: 0
            }
        );
    }
}
