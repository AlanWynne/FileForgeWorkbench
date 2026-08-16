//! BinaryComparator — byte-level comparison for non-text resources.

/// Result of a binary (non-text) comparison.
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryCompareResult {
    /// Resources are byte-for-byte identical.
    Identical { size: u64 },
    /// Resources differ at the byte level.
    Different {
        /// Byte offset of first divergence.
        first_difference_offset: u64,
        /// Size of left resource in bytes.
        left_size: u64,
        /// Size of right resource in bytes.
        right_size: u64,
        /// Percentage similarity (matching bytes / max size × 100).
        similarity_percent: f64,
    },
}

/// Streaming byte-level comparator for non-text resources.
pub struct BinaryComparator;

impl BinaryComparator {
    /// Compare two byte slices and produce a binary comparison result.
    pub fn compare(left: &[u8], right: &[u8]) -> BinaryCompareResult {
        let left_size = left.len() as u64;
        let right_size = right.len() as u64;

        if left == right {
            return BinaryCompareResult::Identical { size: left_size };
        }

        // Find first difference
        let first_diff = left
            .iter()
            .zip(right.iter())
            .position(|(a, b)| a != b)
            .unwrap_or(left.len().min(right.len())) as u64;

        // Count matching bytes
        let matching = left
            .iter()
            .zip(right.iter())
            .filter(|(a, b)| a == b)
            .count() as f64;
        let max_size = left_size.max(right_size) as f64;
        let similarity_percent = if max_size > 0.0 {
            (matching / max_size) * 100.0
        } else {
            100.0
        };

        BinaryCompareResult::Different {
            first_difference_offset: first_diff,
            left_size,
            right_size,
            similarity_percent,
        }
    }

    /// Detect whether content is binary by scanning for null bytes in the first 8 KB.
    pub fn is_binary(content: &[u8]) -> bool {
        let scan_len = content.len().min(8192);
        content[..scan_len].contains(&0u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_bytes_returns_identical() {
        // Validates: Requirement 10.2 — identical binary resources
        let data = b"hello world";
        let result = BinaryComparator::compare(data, data);
        assert!(matches!(
            result,
            BinaryCompareResult::Identical { size: 11 }
        ));
    }

    #[test]
    fn different_bytes_returns_different_with_offset() {
        // Validates: Requirement 10.3 — first divergence offset reported
        let left = b"hello world";
        let right = b"hello WORLD";
        let result = BinaryComparator::compare(left, right);
        match result {
            BinaryCompareResult::Different {
                first_difference_offset,
                ..
            } => {
                assert_eq!(first_difference_offset, 6); // 'w' vs 'W'
            }
            _ => panic!("expected Different"),
        }
    }

    #[test]
    fn different_sizes_reported_correctly() {
        // Validates: Requirement 10.3 — sizes reported
        let left = b"hello";
        let right = b"hello world";
        let result = BinaryComparator::compare(left, right);
        match result {
            BinaryCompareResult::Different {
                left_size,
                right_size,
                ..
            } => {
                assert_eq!(left_size, 5);
                assert_eq!(right_size, 11);
            }
            _ => panic!("expected Different"),
        }
    }

    #[test]
    fn similarity_percent_for_identical_is_100() {
        // Validates: Requirement 10.3 — 100% similarity for identical
        let data = b"test data";
        let result = BinaryComparator::compare(data, data);
        assert!(matches!(result, BinaryCompareResult::Identical { .. }));
    }

    #[test]
    fn similarity_percent_for_completely_different() {
        // Validates: Requirement 10.3 — low similarity for different data
        let left = b"aaaa";
        let right = b"bbbb";
        let result = BinaryComparator::compare(left, right);
        match result {
            BinaryCompareResult::Different {
                similarity_percent, ..
            } => {
                assert!(similarity_percent < 1.0);
            }
            _ => panic!("expected Different"),
        }
    }

    #[test]
    fn is_binary_detects_null_byte() {
        // Validates: Property 12 — null byte detection
        let binary = b"hello\x00world";
        assert!(BinaryComparator::is_binary(binary));
    }

    #[test]
    fn is_binary_false_for_text() {
        // Validates: Property 12 — text content not binary
        let text = b"hello world\nno null bytes here\n";
        assert!(!BinaryComparator::is_binary(text));
    }

    #[test]
    fn is_binary_only_scans_first_8kb() {
        // Validates: Requirement 10.1 — only first 8 KB scanned
        let mut data = vec![b'a'; 9000];
        data[8500] = 0u8; // null byte beyond 8 KB
        assert!(!BinaryComparator::is_binary(&data));
        data[100] = 0u8; // null byte within 8 KB
        assert!(BinaryComparator::is_binary(&data));
    }

    #[test]
    fn empty_inputs_are_identical() {
        // Validates: Requirement 10.2 — empty inputs are identical
        let result = BinaryComparator::compare(b"", b"");
        assert!(matches!(result, BinaryCompareResult::Identical { size: 0 }));
    }

    #[test]
    fn binary_comparison_symmetric() {
        // Validates: Property 16 — comparison is symmetric
        let a = b"hello";
        let b = b"world";
        let ab = BinaryComparator::compare(a, b);
        let ba = BinaryComparator::compare(b, a);
        let ab_identical = matches!(ab, BinaryCompareResult::Identical { .. });
        let ba_identical = matches!(ba, BinaryCompareResult::Identical { .. });
        assert_eq!(ab_identical, ba_identical);
    }
}
