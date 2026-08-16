//! Per-document aggregate of all active indicator decorations.
//!
//! The `DecorationList` manages the collection of indicator values for a document,
//! providing aggregate queries and delegating to individual `RunStyles<u32>` instances.

use std::collections::HashMap;

use crate::run_styles::RunStyles;
use crate::IndicatorNumber;

/// Per-document aggregate of all active indicator decorations.
///
/// Addresses: Requirement 3 AC 2–9
pub struct DecorationList {
    /// Lazily populated map: indicator_number → RunStyles storage.
    decorations: HashMap<IndicatorNumber, RunStyles<u32>>,
    /// Document length for invariant enforcement.
    document_length: u64,
}

impl DecorationList {
    /// Create a new DecorationList for a document of the given length.
    pub fn new(document_length: u64) -> Self {
        Self {
            decorations: HashMap::new(),
            document_length,
        }
    }

    /// Get the indicator value at a position for a specific indicator.
    /// Returns 0 if no decoration exists for that indicator.
    ///
    /// Addresses: Requirement 3 AC 5
    pub fn value_at(&self, indicator: IndicatorNumber, position: u64) -> u32 {
        match self.decorations.get(&indicator) {
            Some(rs) => rs.value_at(position),
            None => 0,
        }
    }

    /// Get the start of the run containing `position` for the given indicator.
    ///
    /// Addresses: Requirement 3 AC 6
    pub fn start_run(&self, indicator: IndicatorNumber, position: u64) -> u64 {
        match self.decorations.get(&indicator) {
            Some(rs) => rs.run_start(position),
            None => 0,
        }
    }

    /// Get the end (exclusive) of the run containing `position`.
    ///
    /// Addresses: Requirement 3 AC 7
    pub fn end_run(&self, indicator: IndicatorNumber, position: u64) -> u64 {
        match self.decorations.get(&indicator) {
            Some(rs) => rs.run_end(position),
            None => self.document_length,
        }
    }

    /// Set indicator values for a contiguous range.
    /// Creates the Decoration lazily if this is the first non-zero write.
    /// Removes the Decoration if all values become zero.
    ///
    /// Addresses: Requirement 3 AC 3, 4, 8
    pub fn fill_range(
        &mut self,
        indicator: IndicatorNumber,
        position: u64,
        value: u32,
        length: u64,
    ) -> bool {
        if value == 0 {
            // Clearing: only act if decoration exists
            if let Some(rs) = self.decorations.get_mut(&indicator) {
                let changed = rs.fill_range(position, 0, length);
                if rs.is_empty() {
                    self.decorations.remove(&indicator);
                }
                return changed;
            }
            return false;
        }

        // Non-zero write: create lazily
        let rs = self
            .decorations
            .entry(indicator)
            .or_insert_with(|| RunStyles::new(self.document_length));
        rs.fill_range(position, value, length)
    }

    /// Returns a bitmask of all indicator numbers with non-zero values at `position`.
    ///
    /// Addresses: Requirement 3 AC 9
    pub fn all_on_for(&self, position: u64) -> u64 {
        let mut mask: u64 = 0;
        for (&indicator, rs) in &self.decorations {
            if rs.value_at(position) != 0 {
                mask |= 1u64 << indicator.0;
            }
        }
        mask
    }

    /// Insert space at position across all active decorations.
    ///
    /// Addresses: Requirement 4 AC 1
    pub fn insert_space(&mut self, position: u64, length: u64) {
        for rs in self.decorations.values_mut() {
            rs.insert_space(position, length);
        }
        self.document_length += length;
    }

    /// Delete a range across all active decorations.
    ///
    /// Addresses: Requirement 4 AC 2
    pub fn delete_range(&mut self, position: u64, length: u64) {
        let actual = length.min(self.document_length.saturating_sub(position));
        for rs in self.decorations.values_mut() {
            rs.delete_range(position, actual);
        }
        self.document_length -= actual;

        // Remove any decorations that became empty
        self.decorations.retain(|_, rs| !rs.is_empty());
    }

    /// Clear all values for indicators in the lexer range (0–7).
    ///
    /// Addresses: Requirement 13 AC 7
    pub fn delete_lexer_decorations(&mut self) {
        for i in 0..=7 {
            self.decorations.remove(&IndicatorNumber(i));
        }
    }

    /// Query all active indicator ranges intersecting [start, end).
    /// Returns a Vec of (indicator_number, run_start, run_end, value).
    ///
    /// Addresses: Requirement 14 AC 2
    pub fn indicators_in_range(
        &self,
        start: u64,
        end: u64,
    ) -> Vec<(IndicatorNumber, u64, u64, u32)> {
        let mut result = Vec::new();
        for (&indicator, rs) in &self.decorations {
            for (pos, run) in rs.runs_in_range(start, end) {
                if run.value != 0 {
                    let run_end = pos + run.length;
                    result.push((indicator, pos, run_end, run.value));
                }
            }
        }
        result
    }

    /// Number of active (non-empty) decorations.
    pub fn active_count(&self) -> usize {
        self.decorations.len()
    }

    /// Current document length.
    pub fn document_length(&self) -> u64 {
        self.document_length
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_zero_active_decorations() {
        // Validates: Requirement 3 AC 2
        let dl = DecorationList::new(100);
        assert_eq!(dl.active_count(), 0);
    }

    #[test]
    fn value_at_returns_zero_for_no_decoration() {
        // Validates: Requirement 3 AC 5
        let dl = DecorationList::new(100);
        assert_eq!(dl.value_at(IndicatorNumber(5), 50), 0);
    }

    #[test]
    fn fill_range_creates_decoration_lazily() {
        // Validates: Requirement 3 AC 3, Property 5
        let mut dl = DecorationList::new(100);
        assert_eq!(dl.active_count(), 0);
        dl.fill_range(IndicatorNumber(10), 10, 1, 5);
        assert_eq!(dl.active_count(), 1);
        assert_eq!(dl.value_at(IndicatorNumber(10), 12), 1);
    }

    #[test]
    fn fill_range_with_zero_removes_decoration_when_empty() {
        // Validates: Requirement 3 AC 4, Property 5
        let mut dl = DecorationList::new(100);
        dl.fill_range(IndicatorNumber(10), 10, 1, 5);
        assert_eq!(dl.active_count(), 1);
        dl.fill_range(IndicatorNumber(10), 10, 0, 5);
        assert_eq!(dl.active_count(), 0);
    }

    #[test]
    fn all_on_for_returns_correct_bitmask() {
        // Validates: Requirement 3 AC 9, Property 7
        let mut dl = DecorationList::new(100);
        dl.fill_range(IndicatorNumber(5), 10, 1, 10);
        dl.fill_range(IndicatorNumber(8), 10, 2, 10);
        let mask = dl.all_on_for(12);
        assert!(mask & (1 << 5) != 0);
        assert!(mask & (1 << 8) != 0);
        assert!(mask & (1 << 3) == 0);
    }

    #[test]
    fn insert_space_propagates_to_all_decorations() {
        // Validates: Requirement 4 AC 1
        let mut dl = DecorationList::new(100);
        dl.fill_range(IndicatorNumber(5), 10, 1, 10);
        dl.insert_space(15, 5);
        assert_eq!(dl.document_length(), 105);
        assert_eq!(dl.value_at(IndicatorNumber(5), 10), 1);
        assert_eq!(dl.value_at(IndicatorNumber(5), 14), 1);
        assert_eq!(dl.value_at(IndicatorNumber(5), 15), 0); // inserted space
        assert_eq!(dl.value_at(IndicatorNumber(5), 20), 1); // shifted
    }

    #[test]
    fn delete_range_propagates_to_all_decorations() {
        // Validates: Requirement 4 AC 2
        let mut dl = DecorationList::new(100);
        dl.fill_range(IndicatorNumber(5), 10, 1, 20); // [10..30) = 1
        dl.delete_range(15, 5); // delete [15..20)
        assert_eq!(dl.document_length(), 95);
        assert_eq!(dl.value_at(IndicatorNumber(5), 10), 1);
        assert_eq!(dl.value_at(IndicatorNumber(5), 14), 1);
        assert_eq!(dl.value_at(IndicatorNumber(5), 15), 1); // shifted from 20
    }

    #[test]
    fn delete_lexer_decorations_clears_range_0_to_7() {
        // Validates: Requirement 13 AC 7
        let mut dl = DecorationList::new(100);
        dl.fill_range(IndicatorNumber(3), 10, 1, 5);
        dl.fill_range(IndicatorNumber(7), 20, 2, 5);
        dl.fill_range(IndicatorNumber(8), 30, 3, 5);
        dl.delete_lexer_decorations();
        assert_eq!(dl.value_at(IndicatorNumber(3), 12), 0);
        assert_eq!(dl.value_at(IndicatorNumber(7), 22), 0);
        assert_eq!(dl.value_at(IndicatorNumber(8), 32), 3); // not affected
    }

    #[test]
    fn indicators_in_range_returns_active_decorations() {
        // Validates: Requirement 14 AC 2
        let mut dl = DecorationList::new(100);
        dl.fill_range(IndicatorNumber(5), 10, 1, 10);
        dl.fill_range(IndicatorNumber(8), 15, 2, 5);
        let results = dl.indicators_in_range(12, 18);
        assert!(!results.is_empty());
    }
}
