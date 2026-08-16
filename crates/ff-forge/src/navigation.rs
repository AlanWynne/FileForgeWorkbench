//! Record navigation engine.
//!
//! Provides O(1) record access via the ByteOffsetIndex: go-to-record,
//! page up/down, first/last, and position reporting.

use crate::byte_index::ByteOffsetIndex;
use crate::error::FileForgeError;

/// Manages navigation state within a file's records.
#[derive(Debug, Clone)]
pub struct RecordNavigator {
    /// Current record index (0-based).
    current_record: usize,
    /// Total records in the file.
    total_records: usize,
    /// Records per page (window size).
    window_size: usize,
}

impl RecordNavigator {
    /// Creates a new navigator for the given index and window size.
    pub fn new(index: &ByteOffsetIndex, window_size: usize) -> Self {
        Self {
            current_record: 0,
            total_records: index.record_count(),
            window_size,
        }
    }

    /// Returns the current 0-based record index.
    pub fn current_record(&self) -> usize {
        self.current_record
    }

    /// Returns the total number of records.
    pub fn total_records(&self) -> usize {
        self.total_records
    }

    /// Returns the window size (page size).
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Returns the current position as a percentage (0.0–100.0).
    pub fn position_percent(&self) -> f64 {
        if self.total_records == 0 {
            0.0
        } else {
            (self.current_record as f64 / self.total_records as f64) * 100.0
        }
    }

    /// Returns position info: (current_record_1based, total_records, percent).
    pub fn position_info(&self) -> (usize, usize, f64) {
        (
            self.current_record + 1,
            self.total_records,
            self.position_percent(),
        )
    }

    /// Navigates to a specific 1-based record number.
    ///
    /// # Errors
    ///
    /// Returns `FileForgeError::RecordOutOfRange` if the record number is invalid.
    pub fn go_to_record(&mut self, record_number: usize) -> Result<usize, FileForgeError> {
        if record_number == 0 || record_number > self.total_records {
            return Err(FileForgeError::RecordOutOfRange {
                requested: record_number,
                total: self.total_records,
            });
        }
        self.current_record = record_number - 1;
        Ok(self.current_record)
    }

    /// Moves to the first record.
    pub fn first_record(&mut self) -> usize {
        self.current_record = 0;
        self.current_record
    }

    /// Moves to the last record.
    pub fn last_record(&mut self) -> usize {
        if self.total_records > 0 {
            self.current_record = self.total_records - 1;
        }
        self.current_record
    }

    /// Moves forward by one page (window_size records).
    ///
    /// Clamps to the last record if the page would exceed the file.
    pub fn page_down(&mut self) -> usize {
        let new_pos = self.current_record + self.window_size;
        self.current_record = new_pos.min(self.total_records.saturating_sub(1));
        self.current_record
    }

    /// Moves backward by one page (window_size records).
    ///
    /// Clamps to the first record (0) if the page would go negative.
    pub fn page_up(&mut self) -> usize {
        self.current_record = self.current_record.saturating_sub(self.window_size);
        self.current_record
    }

    /// Returns the byte offset for the current record position.
    pub fn current_offset(&self, index: &ByteOffsetIndex) -> Option<u64> {
        index.offset_of(self.current_record)
    }

    /// Updates the total record count (after insert/delete operations).
    pub fn update_total(&mut self, new_total: usize) {
        self.total_records = new_total;
        if self.current_record >= new_total && new_total > 0 {
            self.current_record = new_total - 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_index(count: usize) -> ByteOffsetIndex {
        ByteOffsetIndex::FixedWidth {
            lrecl: 80,
            record_count: count,
        }
    }

    // Validates: Requirement 10.1
    #[test]
    fn go_to_record_seeks_directly() {
        let index = make_index(1000);
        let mut nav = RecordNavigator::new(&index, 200);
        nav.go_to_record(500).unwrap();
        assert_eq!(nav.current_record(), 499); // 0-based
    }

    #[test]
    fn go_to_record_out_of_range_returns_error() {
        let index = make_index(100);
        let mut nav = RecordNavigator::new(&index, 20);
        let result = nav.go_to_record(101);
        assert!(matches!(
            result,
            Err(FileForgeError::RecordOutOfRange { .. })
        ));
    }

    #[test]
    fn go_to_record_zero_returns_error() {
        let index = make_index(100);
        let mut nav = RecordNavigator::new(&index, 20);
        let result = nav.go_to_record(0);
        assert!(matches!(
            result,
            Err(FileForgeError::RecordOutOfRange { .. })
        ));
    }

    // Validates: Requirement 10.2
    #[test]
    fn page_down_advances_by_window_size() {
        let index = make_index(1000);
        let mut nav = RecordNavigator::new(&index, 200);
        assert_eq!(nav.page_down(), 200);
        assert_eq!(nav.page_down(), 400);
    }

    #[test]
    fn page_down_clamps_at_last_record() {
        let index = make_index(50);
        let mut nav = RecordNavigator::new(&index, 200);
        let pos = nav.page_down();
        assert_eq!(pos, 49); // Last record (0-based)
    }

    #[test]
    fn page_up_retreats_by_window_size() {
        let index = make_index(1000);
        let mut nav = RecordNavigator::new(&index, 200);
        nav.go_to_record(501).unwrap(); // position at 500
        assert_eq!(nav.page_up(), 300);
    }

    #[test]
    fn page_up_clamps_at_first_record() {
        let index = make_index(1000);
        let mut nav = RecordNavigator::new(&index, 200);
        nav.go_to_record(50).unwrap(); // position at 49
        assert_eq!(nav.page_up(), 0);
    }

    // Validates: Requirement 10.3
    #[test]
    fn first_record_goes_to_beginning() {
        let index = make_index(1000);
        let mut nav = RecordNavigator::new(&index, 200);
        nav.go_to_record(500).unwrap();
        assert_eq!(nav.first_record(), 0);
    }

    #[test]
    fn last_record_goes_to_end() {
        let index = make_index(1000);
        let mut nav = RecordNavigator::new(&index, 200);
        assert_eq!(nav.last_record(), 999);
    }

    // Validates: Requirement 10.4
    #[test]
    fn position_info_reports_correctly() {
        let index = make_index(1000);
        let mut nav = RecordNavigator::new(&index, 200);
        nav.go_to_record(500).unwrap();
        let (current, total, percent) = nav.position_info();
        assert_eq!(current, 500); // 1-based
        assert_eq!(total, 1000);
        assert!((percent - 49.9).abs() < 0.1);
    }

    #[test]
    fn position_percent_empty_file() {
        let index = make_index(0);
        let nav = RecordNavigator::new(&index, 200);
        assert_eq!(nav.position_percent(), 0.0);
    }

    #[test]
    fn current_offset_uses_index() {
        let index = make_index(100);
        let mut nav = RecordNavigator::new(&index, 20);
        nav.go_to_record(10).unwrap();
        assert_eq!(nav.current_offset(&index), Some(9 * 80)); // record 9, 0-based
    }

    #[test]
    fn update_total_clamps_current_position() {
        let index = make_index(1000);
        let mut nav = RecordNavigator::new(&index, 200);
        nav.go_to_record(999).unwrap();
        nav.update_total(500);
        assert_eq!(nav.current_record(), 499); // Clamped to new last
    }
}
