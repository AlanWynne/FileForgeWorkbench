//! Active filter state tracking and status bar indicators.
//!
//! Tracks the currently applied CriteriaSet, match counts, and provides
//! formatted strings for the status bar Criteria_Active_Indicator.

use crate::model::CriteriaSet;

/// Tracks the active filter state for one file session.
///
/// Addresses: Requirement 7 AC 12, Requirement 13
#[derive(Debug, Clone, PartialEq)]
pub struct FilterState {
    /// The currently applied CriteriaSet, or None if no criteria active.
    active_criteria: Option<CriteriaSet>,
    /// Number of records matching the active criteria (visible count).
    visible_count: usize,
    /// Total number of records in the file.
    total_count: usize,
}

impl FilterState {
    /// Create a new inactive filter state.
    pub fn inactive() -> Self {
        Self {
            active_criteria: None,
            visible_count: 0,
            total_count: 0,
        }
    }

    /// Apply a CriteriaSet, transitioning to active state.
    pub fn apply(&mut self, criteria: CriteriaSet, visible: usize, total: usize) {
        self.active_criteria = Some(criteria);
        self.visible_count = visible;
        self.total_count = total;
    }

    /// Clear the active criteria, returning to inactive state.
    pub fn clear(&mut self) {
        self.active_criteria = None;
        self.visible_count = 0;
        self.total_count = 0;
    }

    /// Whether criteria are currently active.
    pub fn is_active(&self) -> bool {
        self.active_criteria.is_some()
    }

    /// Get the active criteria set (if any).
    pub fn active_criteria(&self) -> Option<&CriteriaSet> {
        self.active_criteria.as_ref()
    }

    /// Format the status bar indicator text.
    ///
    /// Returns `Some("Criteria: <name>")` when a named set is active,
    /// `Some("Criteria: active")` when an unnamed set is active,
    /// or `None` when no criteria are active.
    ///
    /// Addresses: Requirement 13 AC 1, 2
    pub fn format_indicator(&self) -> Option<String> {
        let criteria = self.active_criteria.as_ref()?;

        let name_part = criteria.name.as_deref().unwrap_or("active");

        let mut indicator = format!("Criteria: {name_part}");

        // Append record type scope if present
        if let Some(scope) = &criteria.record_type_scope {
            indicator.push_str(&format!(" | Scope: {scope}"));
        }

        Some(indicator)
    }

    /// Format the record count display.
    ///
    /// Returns `Some("Showing N of M records")` when active, `None` when inactive.
    ///
    /// Addresses: Requirement 7 AC 12
    pub fn format_count(&self) -> Option<String> {
        if !self.is_active() {
            return None;
        }
        Some(format!(
            "Showing {} of {} records",
            self.visible_count, self.total_count
        ))
    }

    /// Get the visible record count.
    pub fn visible_count(&self) -> usize {
        self.visible_count
    }

    /// Get the total record count.
    pub fn total_count(&self) -> usize {
        self.total_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CriteriaOperator;

    #[test]
    fn inactive_state_has_no_indicator() {
        let fs = FilterState::inactive();
        assert!(!fs.is_active());
        assert!(fs.format_indicator().is_none());
        assert!(fs.format_count().is_none());
    }

    #[test]
    fn apply_transitions_to_active() {
        let mut fs = FilterState::inactive();
        let cs = CriteriaSet::single("NAME", CriteriaOperator::Eq, "Alice");
        fs.apply(cs, 42, 1000);

        assert!(fs.is_active());
        assert_eq!(fs.visible_count(), 42);
        assert_eq!(fs.total_count(), 1000);
    }

    #[test]
    fn clear_transitions_to_inactive() {
        let mut fs = FilterState::inactive();
        let cs = CriteriaSet::single("NAME", CriteriaOperator::Eq, "Alice");
        fs.apply(cs, 42, 1000);
        fs.clear();

        assert!(!fs.is_active());
        assert!(fs.format_indicator().is_none());
    }

    #[test]
    fn indicator_shows_name_when_named() {
        let mut fs = FilterState::inactive();
        let mut cs = CriteriaSet::single("NAME", CriteriaOperator::Eq, "Alice");
        cs.name = Some("my_filter".to_string());
        fs.apply(cs, 42, 1000);

        assert_eq!(
            fs.format_indicator(),
            Some("Criteria: my_filter".to_string())
        );
    }

    #[test]
    fn indicator_shows_active_when_unnamed() {
        let mut fs = FilterState::inactive();
        let cs = CriteriaSet::single("NAME", CriteriaOperator::Eq, "Alice");
        fs.apply(cs, 42, 1000);

        assert_eq!(fs.format_indicator(), Some("Criteria: active".to_string()));
    }

    #[test]
    fn indicator_shows_scope_when_present() {
        let mut fs = FilterState::inactive();
        let mut cs = CriteriaSet::single("NAME", CriteriaOperator::Eq, "Alice");
        cs.record_type_scope = Some("Detail".to_string());
        fs.apply(cs, 42, 1000);

        assert_eq!(
            fs.format_indicator(),
            Some("Criteria: active | Scope: Detail".to_string())
        );
    }

    #[test]
    fn count_format_when_active() {
        let mut fs = FilterState::inactive();
        let cs = CriteriaSet::single("NAME", CriteriaOperator::Eq, "Alice");
        fs.apply(cs, 142, 10000);

        assert_eq!(
            fs.format_count(),
            Some("Showing 142 of 10000 records".to_string())
        );
    }
}
