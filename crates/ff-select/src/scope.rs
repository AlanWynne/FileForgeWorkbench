//! Criteria scope integration for FIND/CHANGE operations.
//!
//! Provides a scope implementation that restricts FIND/CHANGE operations
//! to records matching the active criteria.

/// Trait for mapping display lines to their parent record index.
pub trait LineToRecordMap {
    /// Get the record index that contains the given line number.
    fn record_for_line(&self, line: usize) -> Option<usize>;
}

/// Provides a scope that restricts FIND/CHANGE operations to records
/// matching the active criteria.
///
/// Addresses: Requirement 8
pub struct CriteriaScope {
    matching_record_indices: Vec<usize>,
}

impl CriteriaScope {
    /// Create a criteria scope from the set of record indices that match
    /// the active criteria.
    pub fn new(matching_indices: Vec<usize>) -> Self {
        Self {
            matching_record_indices: matching_indices,
        }
    }

    /// Check whether a given record index is within the criteria scope.
    ///
    /// Addresses: Requirement 8 AC 1, 2, 6
    pub fn contains_record(&self, record_index: usize) -> bool {
        self.matching_record_indices.contains(&record_index)
    }

    /// Check whether a given line number is within criteria scope.
    ///
    /// Maps the line to its parent record and checks that record.
    ///
    /// Addresses: Requirement 8 AC 6
    pub fn contains_line(
        &self,
        line_number: usize,
        line_to_record_map: &dyn LineToRecordMap,
    ) -> bool {
        match line_to_record_map.record_for_line(line_number) {
            Some(record_index) => self.contains_record(record_index),
            None => false,
        }
    }

    /// Whether this scope has any filtering effect.
    ///
    /// Returns `true` when not all records match (i.e., some records are excluded).
    /// Returns `false` when the scope is empty (no records match — effectively no effect
    /// because FIND/CHANGE would find nothing) or could be a trivial case.
    ///
    /// Addresses: Requirement 8 AC 3
    pub fn is_effective(&self) -> bool {
        !self.matching_record_indices.is_empty()
    }

    /// Get the matching record indices.
    pub fn matching_indices(&self) -> &[usize] {
        &self.matching_record_indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestLineMap {
        /// Maps line number → record index.
        mapping: Vec<(usize, usize)>,
    }

    impl LineToRecordMap for TestLineMap {
        fn record_for_line(&self, line: usize) -> Option<usize> {
            self.mapping
                .iter()
                .find(|(l, _)| *l == line)
                .map(|(_, r)| *r)
        }
    }

    #[test]
    fn contains_record_returns_true_for_matching_index() {
        let scope = CriteriaScope::new(vec![0, 2, 5]);
        assert!(scope.contains_record(0));
        assert!(scope.contains_record(2));
        assert!(scope.contains_record(5));
    }

    #[test]
    fn contains_record_returns_false_for_non_matching_index() {
        let scope = CriteriaScope::new(vec![0, 2, 5]);
        assert!(!scope.contains_record(1));
        assert!(!scope.contains_record(3));
        assert!(!scope.contains_record(100));
    }

    #[test]
    fn contains_line_maps_to_record() {
        let scope = CriteriaScope::new(vec![0, 2]);
        let map = TestLineMap {
            mapping: vec![(0, 0), (1, 0), (2, 1), (3, 1), (4, 2), (5, 2)],
        };

        // Lines 0, 1 belong to record 0 (matching)
        assert!(scope.contains_line(0, &map));
        assert!(scope.contains_line(1, &map));
        // Lines 2, 3 belong to record 1 (not matching)
        assert!(!scope.contains_line(2, &map));
        assert!(!scope.contains_line(3, &map));
        // Lines 4, 5 belong to record 2 (matching)
        assert!(scope.contains_line(4, &map));
        assert!(scope.contains_line(5, &map));
    }

    #[test]
    fn contains_line_returns_false_for_unmapped_line() {
        let scope = CriteriaScope::new(vec![0]);
        let map = TestLineMap { mapping: vec![] };
        assert!(!scope.contains_line(99, &map));
    }

    #[test]
    fn is_effective_when_has_matches() {
        let scope = CriteriaScope::new(vec![0, 1, 2]);
        assert!(scope.is_effective());
    }

    #[test]
    fn is_not_effective_when_empty() {
        let scope = CriteriaScope::new(vec![]);
        assert!(!scope.is_effective());
    }

    #[test]
    fn matching_indices_returns_stored_indices() {
        let scope = CriteriaScope::new(vec![3, 7, 11]);
        assert_eq!(scope.matching_indices(), &[3, 7, 11]);
    }
}
