//! Tab stop list management.
//!
//! Provides the [`TabStopList`] type — an ordered, deduplicated list of positive
//! column positions representing tab stops. This is the foundation for Tab key
//! advancement, shift-to-tab-stop computation, and TABS_Line rendering.

use std::fmt;

/// An ordered, deduplicated list of positive column positions representing tab stops.
///
/// Column positions are 1-based. The list is always sorted in ascending order.
/// Addresses: Requirements 2, 4, 5, 12, 13, 14, 15
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabStopList {
    /// Tab stop column positions, sorted ascending, all > 0, no duplicates.
    stops: Vec<u32>,
}

impl TabStopList {
    /// Creates an empty tab stop list.
    pub fn empty() -> Self {
        Self { stops: Vec::new() }
    }

    /// Creates a tab stop list from an iterator of column positions.
    ///
    /// Invalid values (zero) are filtered out. Duplicates are removed.
    /// Result is sorted in ascending order.
    ///
    /// Addresses: Requirement 2, criterion 2.8; Requirement 4, criterion 4.7
    pub fn from_columns(columns: impl IntoIterator<Item = u32>) -> Self {
        let mut stops: Vec<u32> = columns.into_iter().filter(|&c| c > 0).collect();
        stops.sort_unstable();
        stops.dedup();
        Self { stops }
    }

    /// Creates tab stops at every `interval` columns starting from `interval + 1`.
    ///
    /// Used for the built-in every-8-columns default.
    /// For interval=8, max_column=80: produces [9, 17, 25, 33, 41, 49, 57, 65, 73].
    ///
    /// Addresses: Requirement 4, criterion 4.2
    pub fn every_n_columns(interval: u32, max_column: u32) -> Self {
        if interval == 0 {
            return Self::empty();
        }
        let stops: Vec<u32> = (1..)
            .map(|i| interval * i + 1)
            .take_while(|&col| col <= max_column)
            .collect();
        Self { stops }
    }

    /// Returns the next tab stop column strictly greater than `current_column`.
    ///
    /// If past the last explicit stop, computes by repeating the last interval.
    /// Returns `None` if the list is empty.
    ///
    /// Addresses: Requirement 5, criteria 5.1, 5.2
    pub fn next_stop_after(&self, current_column: u32) -> Option<u32> {
        if self.stops.is_empty() {
            return None;
        }

        // Find the first stop strictly greater than current_column
        if let Some(&stop) = self.stops.iter().find(|&&s| s > current_column) {
            return Some(stop);
        }

        // Past last explicit stop — extend using last interval
        let last_interval = self.last_interval();
        let last_stop = *self.stops.last().unwrap();
        // Compute next stop in the repeating sequence
        let steps_past = (current_column - last_stop) / last_interval + 1;
        Some(last_stop + steps_past * last_interval)
    }

    /// Returns the previous tab stop column strictly less than `current_column`.
    ///
    /// Returns `None` if no stop exists to the left.
    ///
    /// Addresses: Requirement 14, criteria 14.2, 14.3
    pub fn prev_stop_before(&self, current_column: u32) -> Option<u32> {
        if self.stops.is_empty() || current_column <= 1 {
            return None;
        }

        // Find the last stop strictly less than current_column
        self.stops
            .iter()
            .rev()
            .find(|&&s| s < current_column)
            .copied()
    }

    /// Returns the tab stop n positions ahead of `current_column`.
    ///
    /// Addresses: Requirement 14, criterion 14.4
    pub fn nth_stop_after(&self, current_column: u32, n: u32) -> Option<u32> {
        if self.stops.is_empty() || n == 0 {
            return None;
        }
        let mut col = current_column;
        for _ in 0..n {
            col = self.next_stop_after(col)?;
        }
        Some(col)
    }

    /// Returns the tab stop n positions behind `current_column`.
    ///
    /// Addresses: Requirement 14, criterion 14.4
    pub fn nth_stop_before(&self, current_column: u32, n: u32) -> Option<u32> {
        if self.stops.is_empty() || n == 0 {
            return None;
        }
        let mut col = current_column;
        for _ in 0..n {
            col = self.prev_stop_before(col)?;
        }
        Some(col)
    }

    /// Returns the list of stop positions as a slice.
    pub fn stops(&self) -> &[u32] {
        &self.stops
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.stops.is_empty()
    }

    /// Returns the number of explicit tab stops.
    pub fn len(&self) -> usize {
        self.stops.len()
    }

    /// Returns true if the given column is a configured tab stop.
    pub fn contains(&self, column: u32) -> bool {
        self.stops.contains(&column)
    }

    /// Computes the last interval between tab stops for extension past the last stop.
    /// If there's only one stop, uses that stop's value as the interval.
    fn last_interval(&self) -> u32 {
        match self.stops.len() {
            0 => 8, // fallback
            1 => self.stops[0],
            n => self.stops[n - 1] - self.stops[n - 2],
        }
    }
}

impl fmt::Display for TabStopList {
    /// Formats as space-separated column numbers (e.g., "7 12 72").
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: Vec<String> = self.stops.iter().map(|n| n.to_string()).collect();
        write!(f, "{}", s.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_list_has_no_stops() {
        // Validates: Requirement 2.8
        let list = TabStopList::empty();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.stops(), &[]);
    }

    #[test]
    fn from_columns_filters_zeros_and_deduplicates() {
        // Validates: Requirement 2.8, 4.7
        let list = TabStopList::from_columns(vec![0, 5, 3, 5, 0, 10, 3]);
        assert_eq!(list.stops(), &[3, 5, 10]);
    }

    #[test]
    fn from_columns_sorts_ascending() {
        // Validates: Requirement 4.7
        let list = TabStopList::from_columns(vec![20, 5, 15, 10]);
        assert_eq!(list.stops(), &[5, 10, 15, 20]);
    }

    #[test]
    fn every_n_columns_generates_correct_stops() {
        // Validates: Requirement 4.2
        let list = TabStopList::every_n_columns(8, 40);
        assert_eq!(list.stops(), &[9, 17, 25, 33]);
    }

    #[test]
    fn every_n_columns_with_zero_interval_returns_empty() {
        let list = TabStopList::every_n_columns(0, 80);
        assert!(list.is_empty());
    }

    #[test]
    fn next_stop_after_finds_next_explicit_stop() {
        // Validates: Requirement 5.1
        let list = TabStopList::from_columns(vec![5, 10, 15]);
        assert_eq!(list.next_stop_after(3), Some(5));
        assert_eq!(list.next_stop_after(5), Some(10));
        assert_eq!(list.next_stop_after(7), Some(10));
        assert_eq!(list.next_stop_after(10), Some(15));
    }

    #[test]
    fn next_stop_after_extends_past_last_stop() {
        // Validates: Requirement 5.2
        let list = TabStopList::from_columns(vec![5, 10, 15]);
        // Last interval is 5, so after 15 we get 20, 25, etc.
        assert_eq!(list.next_stop_after(15), Some(20));
        assert_eq!(list.next_stop_after(18), Some(20));
        assert_eq!(list.next_stop_after(20), Some(25));
    }

    #[test]
    fn next_stop_after_empty_list_returns_none() {
        let list = TabStopList::empty();
        assert_eq!(list.next_stop_after(5), None);
    }

    #[test]
    fn prev_stop_before_finds_previous_stop() {
        // Validates: Requirement 14.2
        let list = TabStopList::from_columns(vec![5, 10, 15]);
        assert_eq!(list.prev_stop_before(15), Some(10));
        assert_eq!(list.prev_stop_before(12), Some(10));
        assert_eq!(list.prev_stop_before(10), Some(5));
        assert_eq!(list.prev_stop_before(7), Some(5));
    }

    #[test]
    fn prev_stop_before_returns_none_at_or_before_first_stop() {
        // Validates: Requirement 14.3
        let list = TabStopList::from_columns(vec![5, 10, 15]);
        assert_eq!(list.prev_stop_before(5), None);
        assert_eq!(list.prev_stop_before(3), None);
        assert_eq!(list.prev_stop_before(1), None);
    }

    #[test]
    fn nth_stop_after_advances_n_stops() {
        // Validates: Requirement 14.4
        let list = TabStopList::from_columns(vec![5, 10, 15, 20]);
        assert_eq!(list.nth_stop_after(3, 1), Some(5));
        assert_eq!(list.nth_stop_after(3, 2), Some(10));
        assert_eq!(list.nth_stop_after(3, 3), Some(15));
    }

    #[test]
    fn nth_stop_before_retreats_n_stops() {
        // Validates: Requirement 14.4
        let list = TabStopList::from_columns(vec![5, 10, 15, 20]);
        assert_eq!(list.nth_stop_before(20, 1), Some(15));
        assert_eq!(list.nth_stop_before(20, 2), Some(10));
        assert_eq!(list.nth_stop_before(20, 3), Some(5));
        assert_eq!(list.nth_stop_before(20, 4), None);
    }

    #[test]
    fn contains_checks_membership() {
        let list = TabStopList::from_columns(vec![5, 10, 15]);
        assert!(list.contains(5));
        assert!(list.contains(10));
        assert!(!list.contains(7));
    }

    #[test]
    fn display_formats_as_space_separated_numbers() {
        let list = TabStopList::from_columns(vec![7, 12, 72]);
        assert_eq!(format!("{list}"), "7 12 72");
    }

    #[test]
    fn display_empty_list_is_empty_string() {
        let list = TabStopList::empty();
        assert_eq!(format!("{list}"), "");
    }

    #[test]
    fn single_stop_extends_by_its_own_value() {
        // Single stop at 8 means interval=8, so after 8 → 16, 24, etc.
        let list = TabStopList::from_columns(vec![8]);
        assert_eq!(list.next_stop_after(8), Some(16));
        assert_eq!(list.next_stop_after(16), Some(24));
    }
}
