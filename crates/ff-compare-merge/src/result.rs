//! DiffResult, DiffHunk, DiffStatistics, InlineChange types.

/// A character-level difference within a changed line pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineChange {
    /// Byte offset range in the left line that changed.
    pub left_range: std::ops::Range<usize>,
    /// Byte offset range in the right line that changed.
    pub right_range: std::ops::Range<usize>,
}

/// A contiguous region describing the relationship between left and right inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffHunk {
    /// Lines identical in both inputs.
    Equal {
        left_start: usize,
        right_start: usize,
        count: usize,
    },
    /// Lines present only in the right input (added).
    Added { right_start: usize, count: usize },
    /// Lines present only in the left input (removed).
    Removed { left_start: usize, count: usize },
    /// Lines that differ between left and right inputs.
    Changed {
        left_start: usize,
        left_count: usize,
        right_start: usize,
        right_count: usize,
        inline_changes: Vec<InlineChange>,
    },
}

impl DiffHunk {
    /// Returns true if this is an Equal hunk.
    pub fn is_equal(&self) -> bool {
        matches!(self, DiffHunk::Equal { .. })
    }

    /// Number of lines this hunk covers on the left side.
    pub fn left_line_count(&self) -> usize {
        match self {
            DiffHunk::Equal { count, .. } => *count,
            DiffHunk::Removed { count, .. } => *count,
            DiffHunk::Changed { left_count, .. } => *left_count,
            DiffHunk::Added { .. } => 0,
        }
    }

    /// Number of lines this hunk covers on the right side.
    pub fn right_line_count(&self) -> usize {
        match self {
            DiffHunk::Equal { count, .. } => *count,
            DiffHunk::Added { count, .. } => *count,
            DiffHunk::Changed { right_count, .. } => *right_count,
            DiffHunk::Removed { .. } => 0,
        }
    }

    /// Starting line index on the left side (0-based).
    pub fn left_start(&self) -> usize {
        match self {
            DiffHunk::Equal { left_start, .. } => *left_start,
            DiffHunk::Removed { left_start, .. } => *left_start,
            DiffHunk::Changed { left_start, .. } => *left_start,
            DiffHunk::Added { .. } => 0,
        }
    }

    /// Starting line index on the right side (0-based).
    pub fn right_start(&self) -> usize {
        match self {
            DiffHunk::Equal { right_start, .. } => *right_start,
            DiffHunk::Added { right_start, .. } => *right_start,
            DiffHunk::Changed { right_start, .. } => *right_start,
            DiffHunk::Removed { .. } => 0,
        }
    }
}

/// Summary statistics for a diff computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffStatistics {
    /// Total lines present only in right input.
    pub lines_added: usize,
    /// Total lines present only in left input.
    pub lines_removed: usize,
    /// Total line pairs that differ (Changed hunks, counted by left side).
    pub lines_changed: usize,
    /// Total lines identical in both inputs.
    pub lines_unchanged: usize,
    /// Total number of difference hunks (non-Equal).
    pub hunks_count: usize,
}

impl DiffStatistics {
    /// Compute statistics from a slice of hunks.
    pub fn from_hunks(hunks: &[DiffHunk]) -> Self {
        let mut stats = DiffStatistics {
            lines_added: 0,
            lines_removed: 0,
            lines_changed: 0,
            lines_unchanged: 0,
            hunks_count: 0,
        };
        for hunk in hunks {
            match hunk {
                DiffHunk::Equal { count, .. } => stats.lines_unchanged += count,
                DiffHunk::Added { count, .. } => {
                    stats.lines_added += count;
                    stats.hunks_count += 1;
                }
                DiffHunk::Removed { count, .. } => {
                    stats.lines_removed += count;
                    stats.hunks_count += 1;
                }
                DiffHunk::Changed {
                    left_count,
                    right_count,
                    ..
                } => {
                    stats.lines_changed += left_count;
                    stats.hunks_count += 1;
                    // right_count lines are "added" in the changed region
                    let _ = right_count;
                }
            }
        }
        stats
    }

    /// Format as a concise summary string.
    pub fn summary(&self) -> String {
        format!(
            "+{} −{} ~{} unchanged: {}",
            self.lines_added, self.lines_removed, self.lines_changed, self.lines_unchanged
        )
    }
}

/// The complete result of a diff computation between two text inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffResult {
    /// Ordered sequence of diff hunks covering the entire input.
    pub hunks: Vec<DiffHunk>,
    /// Summary statistics computed from the hunks.
    pub statistics: DiffStatistics,
}

impl DiffResult {
    /// Create a DiffResult from hunks, computing statistics automatically.
    pub fn new(hunks: Vec<DiffHunk>) -> Self {
        let statistics = DiffStatistics::from_hunks(&hunks);
        Self { hunks, statistics }
    }

    /// Returns only the non-Equal (difference) hunks.
    pub fn diff_hunks(&self) -> impl Iterator<Item = &DiffHunk> {
        self.hunks.iter().filter(|h| !h.is_equal())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statistics_from_empty_hunks() {
        // Validates: Requirement 12.1 — statistics for empty diff
        let stats = DiffStatistics::from_hunks(&[]);
        assert_eq!(stats.lines_added, 0);
        assert_eq!(stats.lines_removed, 0);
        assert_eq!(stats.lines_changed, 0);
        assert_eq!(stats.lines_unchanged, 0);
        assert_eq!(stats.hunks_count, 0);
    }

    #[test]
    fn statistics_from_equal_hunk() {
        // Validates: Requirement 12.1 — equal lines counted as unchanged
        let hunks = vec![DiffHunk::Equal {
            left_start: 0,
            right_start: 0,
            count: 5,
        }];
        let stats = DiffStatistics::from_hunks(&hunks);
        assert_eq!(stats.lines_unchanged, 5);
        assert_eq!(stats.hunks_count, 0);
    }

    #[test]
    fn statistics_from_mixed_hunks() {
        // Validates: Requirement 12.1 — mixed hunk statistics
        let hunks = vec![
            DiffHunk::Equal {
                left_start: 0,
                right_start: 0,
                count: 3,
            },
            DiffHunk::Added {
                right_start: 3,
                count: 2,
            },
            DiffHunk::Removed {
                left_start: 3,
                count: 1,
            },
            DiffHunk::Changed {
                left_start: 4,
                left_count: 2,
                right_start: 5,
                right_count: 2,
                inline_changes: vec![],
            },
        ];
        let stats = DiffStatistics::from_hunks(&hunks);
        assert_eq!(stats.lines_unchanged, 3);
        assert_eq!(stats.lines_added, 2);
        assert_eq!(stats.lines_removed, 1);
        assert_eq!(stats.lines_changed, 2);
        assert_eq!(stats.hunks_count, 3);
    }

    #[test]
    fn diff_hunk_left_right_counts() {
        // Validates: Requirement 2.3 — hunk line counts
        let equal = DiffHunk::Equal {
            left_start: 0,
            right_start: 0,
            count: 4,
        };
        assert_eq!(equal.left_line_count(), 4);
        assert_eq!(equal.right_line_count(), 4);

        let added = DiffHunk::Added {
            right_start: 0,
            count: 3,
        };
        assert_eq!(added.left_line_count(), 0);
        assert_eq!(added.right_line_count(), 3);

        let removed = DiffHunk::Removed {
            left_start: 0,
            count: 2,
        };
        assert_eq!(removed.left_line_count(), 2);
        assert_eq!(removed.right_line_count(), 0);
    }

    #[test]
    fn diff_result_new_computes_statistics() {
        // Validates: Requirement 12.1 — DiffResult auto-computes stats
        let hunks = vec![
            DiffHunk::Equal {
                left_start: 0,
                right_start: 0,
                count: 10,
            },
            DiffHunk::Added {
                right_start: 10,
                count: 5,
            },
        ];
        let result = DiffResult::new(hunks);
        assert_eq!(result.statistics.lines_unchanged, 10);
        assert_eq!(result.statistics.lines_added, 5);
        assert_eq!(result.statistics.hunks_count, 1);
    }

    #[test]
    fn statistics_summary_format() {
        // Validates: Requirement 12.2 — summary string format
        let stats = DiffStatistics {
            lines_added: 42,
            lines_removed: 17,
            lines_changed: 8,
            lines_unchanged: 1203,
            hunks_count: 5,
        };
        let s = stats.summary();
        assert!(s.contains("42"));
        assert!(s.contains("17"));
        assert!(s.contains("8"));
        assert!(s.contains("1203"));
    }
}
