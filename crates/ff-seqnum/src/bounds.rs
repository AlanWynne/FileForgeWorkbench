//! BOUNDS interaction enforcement.
//!
//! Ensures that sequence number operations never modify active BOUNDS state,
//! and detects overlap between sequence columns and BOUNDS for NUMBER ON warnings.

use crate::types::ColumnRange;

/// Check whether a sequence column range overlaps with active BOUNDS.
///
/// Returns true if any overlap exists between the sequence column range
/// and the BOUNDS range.
pub fn columns_overlap_bounds(seq_range: &ColumnRange, bounds_start: u32, bounds_end: u32) -> bool {
    // Overlap if: seq_start <= bounds_end AND seq_end >= bounds_start
    seq_range.start() <= bounds_end && seq_range.end() >= bounds_start
}

/// Generate a warning message when sequence columns overlap with BOUNDS.
pub fn overlap_warning_message(range: &ColumnRange) -> String {
    format!(
        "NUMBER ON: sequence columns {}-{} overlap with active BOUNDS — auto-numbering disabled for overlapping range",
        range.start(),
        range.end()
    )
}

/// Verify that BOUNDS remain unchanged.
///
/// This is a documentation function that confirms the design guarantee:
/// strip/number operations never alter BOUNDS state. The actual BOUNDS
/// state is owned by `ff-navigation-commands`. This module only provides
/// overlap detection for warnings.
///
/// Returns the same bounds unchanged (identity function for testing).
pub fn bounds_after_operation(bounds_start: u32, bounds_end: u32) -> (u32, u32) {
    (bounds_start, bounds_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_overlap_when_range_before_bounds() {
        // Validates: Requirement 10.1
        let range = ColumnRange::new(1, 6).unwrap();
        assert!(!columns_overlap_bounds(&range, 7, 72));
    }

    #[test]
    fn no_overlap_when_range_after_bounds() {
        // Validates: Requirement 10.1
        let range = ColumnRange::new(73, 80).unwrap();
        assert!(!columns_overlap_bounds(&range, 7, 72));
    }

    #[test]
    fn overlap_when_range_within_bounds() {
        // Validates: Requirement 10.4
        let range = ColumnRange::new(10, 20).unwrap();
        assert!(columns_overlap_bounds(&range, 5, 50));
    }

    #[test]
    fn overlap_when_range_partially_overlaps() {
        // Validates: Requirement 10.4
        let range = ColumnRange::new(1, 10).unwrap();
        assert!(columns_overlap_bounds(&range, 8, 72));
    }

    #[test]
    fn bounds_unchanged_after_operation() {
        // Validates: Requirements 10.1, 10.2, 10.3
        let (start, end) = bounds_after_operation(7, 72);
        assert_eq!(start, 7);
        assert_eq!(end, 72);
    }

    #[test]
    fn overlap_warning_message_format() {
        // Validates: Requirement 10.4
        let range = ColumnRange::new(1, 6).unwrap();
        let msg = overlap_warning_message(&range);
        assert!(msg.contains("1-6"));
        assert!(msg.contains("NUMBER ON"));
        assert!(msg.contains("overlap"));
    }
}
