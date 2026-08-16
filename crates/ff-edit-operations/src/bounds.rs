//! Edit boundaries (BOUNDS) — ISPF-style column-range protection.
//!
//! BOUNDS define left and right column limits constraining where edits
//! may be applied within a line. This is an ISPF/mainframe heritage feature.

use crate::error::EditError;

/// ISPF-style column boundaries that restrict where edits can be applied.
///
/// Both `left` and `right` are 1-based and inclusive.
/// Invariant: `left >= 1` and `right > left`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditBounds {
    /// Left boundary column (1-based, inclusive). Must be >= 1.
    pub left: u64,
    /// Right boundary column (1-based, inclusive). Must be > left.
    pub right: u64,
}

impl EditBounds {
    /// Creates new bounds with validation.
    ///
    /// Returns `None` if `left < 1` or `right <= left`.
    pub fn new(left: u64, right: u64) -> Option<Self> {
        if left >= 1 && right > left {
            Some(Self { left, right })
        } else {
            None
        }
    }

    /// Check if a column position (1-based) is within the bounds (inclusive).
    pub fn contains_column(&self, col: u64) -> bool {
        col >= self.left && col <= self.right
    }

    /// Clamp a range to fit within bounds.
    ///
    /// Returns the clamped (start, end) columns. If the range is entirely
    /// outside bounds, returns (left, left) (zero-width).
    pub fn clamp_range(&self, start_col: u64, end_col: u64) -> (u64, u64) {
        let clamped_start = start_col.max(self.left);
        let clamped_end = end_col.min(self.right);
        if clamped_start > clamped_end {
            (self.left, self.left)
        } else {
            (clamped_start, clamped_end)
        }
    }

    /// Returns the width of the bounded region.
    pub fn width(&self) -> u64 {
        self.right - self.left + 1
    }
}

/// Enforces edit boundary constraints on all edit operations.
///
/// When bounds are active, operations outside the [left, right] column range
/// are rejected or clipped.
#[derive(Debug, Clone)]
pub struct BoundsEnforcer {
    bounds: Option<EditBounds>,
}

impl BoundsEnforcer {
    /// Creates a new enforcer with no bounds active.
    pub fn new() -> Self {
        Self { bounds: None }
    }

    /// Sets the active bounds.
    ///
    /// # Errors
    ///
    /// Returns `EditError::InvalidBounds` if the bounds are invalid.
    pub fn set_bounds(&mut self, left: u64, right: u64) -> Result<(), EditError> {
        match EditBounds::new(left, right) {
            Some(bounds) => {
                self.bounds = Some(bounds);
                Ok(())
            }
            None => Err(EditError::InvalidBounds { left, right }),
        }
    }

    /// Clears the active bounds, allowing unrestricted editing.
    pub fn clear_bounds(&mut self) {
        self.bounds = None;
    }

    /// Returns the current bounds, if active.
    pub fn bounds(&self) -> Option<&EditBounds> {
        self.bounds.as_ref()
    }

    /// Returns true if bounds are currently active.
    pub fn is_active(&self) -> bool {
        self.bounds.is_some()
    }

    /// Check if an edit at the given column (1-based) is permitted.
    ///
    /// When no bounds are active, all edits are permitted.
    pub fn allows_edit_at(&self, column: u64) -> bool {
        match &self.bounds {
            None => true,
            Some(bounds) => bounds.contains_column(column),
        }
    }

    /// Clip paste content to fit within bounds, truncating at right boundary.
    ///
    /// Returns the clipped content. If no bounds are active, returns the
    /// original content unchanged.
    pub fn clip_paste_content(&self, content: &str, start_col: u64) -> String {
        match &self.bounds {
            None => content.to_string(),
            Some(bounds) => {
                if start_col > bounds.right {
                    return String::new();
                }
                let available_width = bounds.right.saturating_sub(start_col) + 1;
                content.chars().take(available_width as usize).collect()
            }
        }
    }
}

impl Default for BoundsEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_bounds_new_valid() {
        let bounds = EditBounds::new(1, 72);
        assert!(bounds.is_some());
        let b = bounds.unwrap();
        assert_eq!(b.left, 1);
        assert_eq!(b.right, 72);
    }

    #[test]
    fn edit_bounds_new_invalid_left_zero() {
        assert!(EditBounds::new(0, 72).is_none());
    }

    #[test]
    fn edit_bounds_new_invalid_right_equals_left() {
        assert!(EditBounds::new(5, 5).is_none());
    }

    #[test]
    fn edit_bounds_new_invalid_right_less_than_left() {
        assert!(EditBounds::new(10, 5).is_none());
    }

    #[test]
    fn contains_column_within_bounds() {
        let bounds = EditBounds::new(5, 20).unwrap();
        assert!(bounds.contains_column(5));
        assert!(bounds.contains_column(10));
        assert!(bounds.contains_column(20));
    }

    #[test]
    fn contains_column_outside_bounds() {
        let bounds = EditBounds::new(5, 20).unwrap();
        assert!(!bounds.contains_column(4));
        assert!(!bounds.contains_column(21));
    }

    #[test]
    fn clamp_range_within_bounds() {
        let bounds = EditBounds::new(5, 20).unwrap();
        assert_eq!(bounds.clamp_range(7, 15), (7, 15));
    }

    #[test]
    fn clamp_range_exceeding_bounds() {
        let bounds = EditBounds::new(5, 20).unwrap();
        assert_eq!(bounds.clamp_range(3, 25), (5, 20));
    }

    #[test]
    fn clamp_range_entirely_outside_bounds() {
        let bounds = EditBounds::new(5, 20).unwrap();
        assert_eq!(bounds.clamp_range(25, 30), (5, 5));
    }

    #[test]
    fn width_calculates_correctly() {
        let bounds = EditBounds::new(1, 72).unwrap();
        assert_eq!(bounds.width(), 72);
    }

    #[test]
    fn enforcer_no_bounds_allows_all() {
        let enforcer = BoundsEnforcer::new();
        assert!(!enforcer.is_active());
        assert!(enforcer.allows_edit_at(1));
        assert!(enforcer.allows_edit_at(1000));
    }

    #[test]
    fn enforcer_set_bounds_restricts_edits() {
        let mut enforcer = BoundsEnforcer::new();
        enforcer.set_bounds(5, 20).unwrap();
        assert!(enforcer.is_active());
        assert!(!enforcer.allows_edit_at(4));
        assert!(enforcer.allows_edit_at(5));
        assert!(enforcer.allows_edit_at(20));
        assert!(!enforcer.allows_edit_at(21));
    }

    #[test]
    fn enforcer_set_bounds_invalid_returns_error() {
        let mut enforcer = BoundsEnforcer::new();
        let result = enforcer.set_bounds(0, 5);
        assert!(result.is_err());
    }

    #[test]
    fn enforcer_clear_bounds_removes_restriction() {
        let mut enforcer = BoundsEnforcer::new();
        enforcer.set_bounds(5, 20).unwrap();
        enforcer.clear_bounds();
        assert!(!enforcer.is_active());
        assert!(enforcer.allows_edit_at(1));
    }

    #[test]
    fn clip_paste_content_no_bounds_returns_original() {
        let enforcer = BoundsEnforcer::new();
        assert_eq!(enforcer.clip_paste_content("hello world", 1), "hello world");
    }

    #[test]
    fn clip_paste_content_truncates_at_right_boundary() {
        let mut enforcer = BoundsEnforcer::new();
        enforcer.set_bounds(1, 5).unwrap();
        // Starting at col 1 with width 5, can fit 5 chars
        assert_eq!(enforcer.clip_paste_content("hello world", 1), "hello");
    }

    #[test]
    fn clip_paste_content_past_right_boundary_returns_empty() {
        let mut enforcer = BoundsEnforcer::new();
        enforcer.set_bounds(1, 5).unwrap();
        assert_eq!(enforcer.clip_paste_content("hello", 10), "");
    }
}
