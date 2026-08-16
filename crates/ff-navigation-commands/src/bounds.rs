//! BOUNDS / BNDS command implementation.
//!
//! Manages active column boundaries for column-sensitive operations
//! and the BNDS_Line display artifact.

use crate::error::NavigationError;
use crate::types::ActiveBounds;

/// Session-level bounds state manager.
///
/// Tracks the active column boundaries and whether they affect FIND operations.
/// Provides a public query API for other command executors.
#[derive(Debug, Clone)]
pub struct BoundsManager {
    /// Currently active bounds (None = no bounds set).
    active_bounds: Option<ActiveBounds>,
    /// Whether to affect FIND operations.
    affect_find: bool,
    /// Positions of BNDS_Lines in the display (anchored line numbers).
    bnds_line_positions: Vec<u64>,
}

impl BoundsManager {
    /// Create with no active bounds.
    pub fn new() -> Self {
        Self {
            active_bounds: None,
            affect_find: false,
            bnds_line_positions: Vec::new(),
        }
    }

    /// Set active bounds.
    ///
    /// # Errors
    ///
    /// Returns `NavigationError::InvalidBounds` if `left < 1` or `right <= left`.
    pub fn set_bounds(&mut self, left: u64, right: u64) -> Result<(), NavigationError> {
        let bounds = ActiveBounds::new(left, right).ok_or(NavigationError::InvalidBounds)?;
        self.active_bounds = Some(bounds);
        Ok(())
    }

    /// Clear active bounds and remove BNDS_Line.
    pub fn clear_bounds(&mut self) {
        self.active_bounds = None;
        self.bnds_line_positions.clear();
    }

    /// Query current active bounds (public API for other crates).
    pub fn active_bounds(&self) -> Option<ActiveBounds> {
        self.active_bounds
    }

    /// Whether bounds should affect FIND operations.
    pub fn bounds_affect_find(&self) -> bool {
        self.affect_find && self.active_bounds.is_some()
    }

    /// Update configuration (called on config reload).
    pub fn update_config(&mut self, affect_find: bool) {
        self.affect_find = affect_find;
    }

    /// Computes the effective column range for a SORT operation.
    ///
    /// - If an explicit range is given and bounds are set, returns the intersection.
    /// - If no explicit range is given and bounds are set, returns the bounds range.
    /// - If no bounds are set, returns the explicit range (or None).
    pub fn effective_sort_range(&self, explicit: Option<(u64, u64)>) -> Option<(u64, u64)> {
        match (self.active_bounds, explicit) {
            (Some(bounds), Some((col1, col2))) => bounds.intersect(col1, col2),
            (Some(bounds), None) => Some((bounds.left, bounds.right)),
            (None, explicit) => explicit,
        }
    }

    /// Format the BNDS_Line display string.
    ///
    /// Shows `<` at the left column position and `>` at the right column position.
    pub fn format_bnds_line(left: u64, right: u64) -> String {
        let width = right as usize + 1;
        let mut line = vec![b' '; width];
        if (left as usize) <= width && left >= 1 {
            line[left as usize - 1] = b'<';
        }
        if (right as usize) <= width {
            line[right as usize - 1] = b'>';
        }
        String::from_utf8_lossy(&line).to_string()
    }
}

impl Default for BoundsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_valid_bounds() {
        // Validates: Requirement 5.1
        let mut mgr = BoundsManager::new();
        assert!(mgr.set_bounds(1, 72).is_ok());
        assert_eq!(
            mgr.active_bounds(),
            Some(ActiveBounds { left: 1, right: 72 })
        );
    }

    #[test]
    fn set_invalid_bounds_left_zero() {
        // Validates: Requirement 5.13
        let mut mgr = BoundsManager::new();
        assert_eq!(mgr.set_bounds(0, 72), Err(NavigationError::InvalidBounds));
        assert_eq!(mgr.active_bounds(), None);
    }

    #[test]
    fn set_invalid_bounds_right_equals_left() {
        // Validates: Requirement 5.13
        let mut mgr = BoundsManager::new();
        assert_eq!(mgr.set_bounds(5, 5), Err(NavigationError::InvalidBounds));
    }

    #[test]
    fn set_invalid_bounds_right_less_than_left() {
        // Validates: Requirement 5.13
        let mut mgr = BoundsManager::new();
        assert_eq!(mgr.set_bounds(10, 5), Err(NavigationError::InvalidBounds));
    }

    #[test]
    fn clear_bounds_removes_active_bounds() {
        // Validates: Requirement 5.4, 5.11
        let mut mgr = BoundsManager::new();
        mgr.set_bounds(1, 72).unwrap();
        mgr.clear_bounds();
        assert_eq!(mgr.active_bounds(), None);
    }

    #[test]
    fn bounds_affect_find_when_configured() {
        // Validates: Requirement 5.8
        let mut mgr = BoundsManager::new();
        mgr.update_config(true);
        mgr.set_bounds(1, 72).unwrap();
        assert!(mgr.bounds_affect_find());
    }

    #[test]
    fn bounds_do_not_affect_find_without_bounds_set() {
        let mut mgr = BoundsManager::new();
        mgr.update_config(true);
        assert!(!mgr.bounds_affect_find());
    }

    #[test]
    fn effective_sort_range_with_bounds_no_explicit() {
        // Validates: Requirement 2.9
        let mut mgr = BoundsManager::new();
        mgr.set_bounds(5, 20).unwrap();
        assert_eq!(mgr.effective_sort_range(None), Some((5, 20)));
    }

    #[test]
    fn effective_sort_range_intersection() {
        // Validates: Requirement 2.10
        let mut mgr = BoundsManager::new();
        mgr.set_bounds(5, 20).unwrap();
        assert_eq!(mgr.effective_sort_range(Some((10, 30))), Some((10, 20)));
        assert_eq!(mgr.effective_sort_range(Some((1, 15))), Some((5, 15)));
    }

    #[test]
    fn effective_sort_range_empty_intersection() {
        let mut mgr = BoundsManager::new();
        mgr.set_bounds(5, 10).unwrap();
        assert_eq!(mgr.effective_sort_range(Some((11, 20))), None);
    }

    #[test]
    fn effective_sort_range_no_bounds() {
        let mgr = BoundsManager::new();
        assert_eq!(mgr.effective_sort_range(Some((1, 10))), Some((1, 10)));
        assert_eq!(mgr.effective_sort_range(None), None);
    }
}
