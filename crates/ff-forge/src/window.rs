//! Window management and record streaming.
//!
//! Manages on-demand loading of record windows — contiguous subsets of
//! records loaded for display, avoiding full-file memory load.

/// A window of records currently loaded for display.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordWindow {
    /// 0-based index of the first record in this window.
    pub start_index: usize,
    /// The loaded record content bytes.
    pub records: Vec<Vec<u8>>,
    /// Maximum window size (records per page).
    pub window_size: usize,
}

/// Default window size in records.
pub const DEFAULT_WINDOW_SIZE: usize = 200;

impl RecordWindow {
    /// Creates a new empty window with the given start and size.
    pub fn new(start_index: usize, window_size: usize) -> Self {
        Self {
            start_index,
            records: Vec::new(),
            window_size,
        }
    }

    /// Returns the number of records currently in the window.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true if the window contains no records.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns the end index (exclusive) of this window.
    pub fn end_index(&self) -> usize {
        self.start_index + self.records.len()
    }

    /// Returns true if the given record index is within this window.
    pub fn contains(&self, record_index: usize) -> bool {
        record_index >= self.start_index && record_index < self.end_index()
    }

    /// Returns the record at the given absolute index, if within this window.
    pub fn get(&self, record_index: usize) -> Option<&[u8]> {
        if self.contains(record_index) {
            Some(&self.records[record_index - self.start_index])
        } else {
            None
        }
    }
}

/// Loads a window of fixed-width records from a data buffer.
///
/// Reads `window_size` records starting at `start_index` using direct
/// byte offset calculation (O(1) seek for each record).
pub fn load_fixed_window(
    data: &[u8],
    lrecl: usize,
    start_index: usize,
    window_size: usize,
) -> RecordWindow {
    let total_records = data.len().checked_div(lrecl).unwrap_or(0);
    let actual_start = start_index.min(total_records);
    let actual_count = window_size.min(total_records.saturating_sub(actual_start));

    let mut records = Vec::with_capacity(actual_count);
    for i in 0..actual_count {
        let offset = (actual_start + i) * lrecl;
        let end = offset + lrecl;
        if end <= data.len() {
            records.push(data[offset..end].to_vec());
        }
    }

    RecordWindow {
        start_index: actual_start,
        records,
        window_size,
    }
}

/// Loads a window of variable-length records using a byte-offset index.
///
/// Each record is terminated by a newline in the source data.
pub fn load_variable_window(
    data: &[u8],
    offsets: &[u64],
    start_index: usize,
    window_size: usize,
) -> RecordWindow {
    let total_records = offsets.len();
    let actual_start = start_index.min(total_records);
    let actual_count = window_size.min(total_records.saturating_sub(actual_start));

    let mut records = Vec::with_capacity(actual_count);
    for i in 0..actual_count {
        let idx = actual_start + i;
        let offset = offsets[idx] as usize;

        // Find end of record (next newline or end of data)
        let end = data[offset..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|pos| offset + pos)
            .unwrap_or(data.len());

        // Trim trailing \r if present
        let end = if end > offset && data[end - 1] == b'\r' {
            end - 1
        } else {
            end
        };

        records.push(data[offset..end].to_vec());
    }

    RecordWindow {
        start_index: actual_start,
        records,
        window_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.7
    #[test]
    fn load_fixed_window_default_200_records() {
        let lrecl = 10;
        let data = vec![b'A'; lrecl * 500]; // 500 records
        let window = load_fixed_window(&data, lrecl, 0, DEFAULT_WINDOW_SIZE);
        assert_eq!(window.len(), 200);
        assert_eq!(window.start_index, 0);
        assert_eq!(window.window_size, DEFAULT_WINDOW_SIZE);
    }

    // Validates: Requirement 2.8
    #[test]
    fn load_fixed_window_no_full_file_load() {
        let lrecl = 80;
        let data = vec![b'X'; lrecl * 10000]; // 10000 records
        let window = load_fixed_window(&data, lrecl, 5000, 200);
        assert_eq!(window.len(), 200);
        assert_eq!(window.start_index, 5000);
        // Window only holds 200 records, not all 10000
    }

    #[test]
    fn load_fixed_window_at_end_of_file() {
        let lrecl = 10;
        let data = vec![b'A'; lrecl * 50]; // 50 records
        let window = load_fixed_window(&data, lrecl, 45, 200);
        assert_eq!(window.len(), 5); // Only 5 remaining
        assert_eq!(window.start_index, 45);
    }

    #[test]
    fn load_fixed_window_start_beyond_end() {
        let lrecl = 10;
        let data = vec![b'A'; lrecl * 10];
        let window = load_fixed_window(&data, lrecl, 100, 200);
        assert_eq!(window.len(), 0);
    }

    #[test]
    fn window_contains_check() {
        let window = RecordWindow {
            start_index: 10,
            records: vec![vec![0; 80]; 5],
            window_size: 200,
        };
        assert!(!window.contains(9));
        assert!(window.contains(10));
        assert!(window.contains(14));
        assert!(!window.contains(15));
    }

    #[test]
    fn window_get_returns_record_by_absolute_index() {
        let window = RecordWindow {
            start_index: 5,
            records: vec![vec![1; 10], vec![2; 10], vec![3; 10]],
            window_size: 200,
        };
        assert_eq!(window.get(5).unwrap(), &[1; 10]);
        assert_eq!(window.get(7).unwrap(), &[3; 10]);
        assert!(window.get(4).is_none());
        assert!(window.get(8).is_none());
    }

    #[test]
    fn load_variable_window_from_offsets() {
        let data = b"Line0\nLine01\nLine002\n";
        let offsets: Vec<u64> = vec![0, 6, 13];
        let window = load_variable_window(data, &offsets, 0, 200);
        assert_eq!(window.len(), 3);
        assert_eq!(window.records[0], b"Line0");
        assert_eq!(window.records[1], b"Line01");
        assert_eq!(window.records[2], b"Line002");
    }

    #[test]
    fn configurable_window_size() {
        let lrecl = 10;
        let data = vec![b'A'; lrecl * 100];
        let window = load_fixed_window(&data, lrecl, 0, 50);
        assert_eq!(window.len(), 50);
        assert_eq!(window.window_size, 50);
    }
}
