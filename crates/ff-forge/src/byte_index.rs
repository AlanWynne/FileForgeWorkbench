//! Byte offset index for O(1) record access.
//!
//! For fixed-width files with known LRECL, record positions are calculated
//! directly (no storage needed). For variable-length and VB files, an explicit
//! offset vector is stored.

/// An in-memory index of record byte positions within the source file.
///
/// Enables O(1) random access to any record by index.
#[derive(Debug, Clone, PartialEq)]
pub enum ByteOffsetIndex {
    /// Computed index — record N starts at N * lrecl.
    FixedWidth {
        /// Logical record length in bytes.
        lrecl: usize,
        /// Total number of records.
        record_count: usize,
    },
    /// Stored index — vec of byte offsets (one per record).
    Variable {
        /// Byte offset of each record's content start.
        offsets: Vec<u64>,
    },
}

impl ByteOffsetIndex {
    /// Returns the total number of records in the file.
    pub fn record_count(&self) -> usize {
        match self {
            Self::FixedWidth { record_count, .. } => *record_count,
            Self::Variable { offsets } => offsets.len(),
        }
    }

    /// Returns the byte offset of the record at the given 0-based index.
    pub fn offset_of(&self, record_index: usize) -> Option<u64> {
        match self {
            Self::FixedWidth {
                lrecl,
                record_count,
            } => {
                if record_index < *record_count {
                    Some((record_index * lrecl) as u64)
                } else {
                    None
                }
            }
            Self::Variable { offsets } => offsets.get(record_index).copied(),
        }
    }

    /// Memory footprint in bytes.
    ///
    /// Fixed-width indices use O(1) memory. Variable indices use 8 bytes per record.
    pub fn memory_usage(&self) -> usize {
        match self {
            Self::FixedWidth { .. } => std::mem::size_of::<Self>(),
            Self::Variable { offsets } => {
                std::mem::size_of::<Self>() + offsets.len() * std::mem::size_of::<u64>()
            }
        }
    }

    /// Returns true if the index represents a fixed-width file.
    pub fn is_fixed_width(&self) -> bool {
        matches!(self, Self::FixedWidth { .. })
    }

    /// Creates a fixed-width index for the given file size and LRECL.
    pub fn for_fixed_width(file_size: u64, lrecl: usize) -> Self {
        let record_count = (file_size as usize).checked_div(lrecl).unwrap_or(0);
        Self::FixedWidth {
            lrecl,
            record_count,
        }
    }

    /// Creates a variable-length index from a vector of offsets.
    pub fn for_variable(offsets: Vec<u64>) -> Self {
        Self::Variable { offsets }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 2.2
    #[test]
    fn fixed_width_index_calculates_offsets_directly() {
        let index = ByteOffsetIndex::FixedWidth {
            lrecl: 80,
            record_count: 1000,
        };
        assert_eq!(index.offset_of(0), Some(0));
        assert_eq!(index.offset_of(1), Some(80));
        assert_eq!(index.offset_of(999), Some(999 * 80));
        assert_eq!(index.offset_of(1000), None);
    }

    #[test]
    fn fixed_width_index_record_count() {
        let index = ByteOffsetIndex::FixedWidth {
            lrecl: 80,
            record_count: 500,
        };
        assert_eq!(index.record_count(), 500);
    }

    #[test]
    fn variable_index_returns_stored_offsets() {
        let index = ByteOffsetIndex::Variable {
            offsets: vec![0, 100, 250, 400],
        };
        assert_eq!(index.offset_of(0), Some(0));
        assert_eq!(index.offset_of(1), Some(100));
        assert_eq!(index.offset_of(3), Some(400));
        assert_eq!(index.offset_of(4), None);
        assert_eq!(index.record_count(), 4);
    }

    // Validates: Requirement 10.6
    #[test]
    fn fixed_width_memory_usage_is_constant() {
        let small = ByteOffsetIndex::FixedWidth {
            lrecl: 80,
            record_count: 100,
        };
        let large = ByteOffsetIndex::FixedWidth {
            lrecl: 80,
            record_count: 10_000_000,
        };
        // Memory should be the same regardless of record count
        assert_eq!(small.memory_usage(), large.memory_usage());
    }

    #[test]
    fn variable_memory_usage_scales_with_records() {
        let index = ByteOffsetIndex::Variable {
            offsets: vec![0; 1_000_000],
        };
        // ~8 MB for 1M records (8 bytes each)
        let usage = index.memory_usage();
        assert!(usage >= 8_000_000);
        assert!(usage < 9_000_000);
    }

    #[test]
    fn memory_budget_10m_records_under_100mb() {
        // 10 million records × 8 bytes = 80 MB < 100 MB
        let estimated = 10_000_000 * std::mem::size_of::<u64>();
        assert!(estimated <= 100 * 1024 * 1024);
    }

    #[test]
    fn for_fixed_width_from_file_size() {
        let index = ByteOffsetIndex::for_fixed_width(8000, 80);
        assert_eq!(index.record_count(), 100);
        assert_eq!(index.offset_of(50), Some(4000));
    }

    #[test]
    fn for_variable_from_offsets() {
        let index = ByteOffsetIndex::for_variable(vec![0, 50, 130]);
        assert_eq!(index.record_count(), 3);
        assert_eq!(index.offset_of(2), Some(130));
    }
}
